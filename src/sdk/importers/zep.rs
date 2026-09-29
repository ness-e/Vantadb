//! Zep / Graphiti → VantaDB importer.
//!
//! Accepts the graph dump a Zep user can produce with the SDK's bulk list
//! methods (`graph.episode.get_by_user_id`, `graph.edge.get_by_user_id`, …):
//! `{"user_id" | "graph_id", "episodes": [...], "facts": [...] | "edges": [...]}`.
//!
//! Fidelity highlights:
//! - episodes → **asserted** records carrying their creation time;
//! - facts → **derived** records with `derived_from` = source episode uuids
//!   (provenance) and `valid_at`/`invalid_at` mapped to the v2 validity window;
//! - `expired_at` (transaction time) has no v2 destination → declared discard.
//!
//! Mapping table and declared discards: `docs/api/MEMORY_INTERCHANGE_FORMAT.md`.

use serde_json::Value as JsonValue;

use super::{
    base_line, insert_meta, parse_json, preserve_unknown, read_source_file, scoped_namespace,
    string_field, timestamp_ms, Conversion, ConversionStats,
};
use crate::error::{Error, Result};
use crate::sdk::types::{default_confidence, ConfidenceClass, Value, DERIVATION_DISCOUNT};
use std::collections::BTreeSet;
use std::path::Path;

/// Retrieval-time fields with no v2 destination: discarded and counted.
const SEARCH_TIME: &[&str] = &["score", "relevance", "selection_rank"];

/// Fact discards on top of the search-time set: `expired_at` is Zep's
/// transaction-time invalidation, which v2 does not model.
const FACT_DISCARDED: &[&str] = &["score", "relevance", "selection_rank", "expired_at"];

const EPISODE_CONSUMED: &[&str] = &[
    "uuid",
    "episode_id",
    "content",
    "created_at",
    "source",
    "source_description",
    "role",
    "role_type",
    "thread_id",
    "document_id",
    "metadata",
];

const FACT_CONSUMED: &[&str] = &[
    "uuid",
    "edge_uuid",
    "fact",
    "name",
    "created_at",
    "valid_at",
    "invalid_at",
    "episodes",
    "attributes",
    "source_node_uuid",
    "target_node_uuid",
    "source_node_name",
    "target_node_name",
    "source_node_labels",
    "target_node_labels",
    "scope",
    "hyperedge_uuid",
];

/// Root-level artifact groups outside the VER-05 scope (entities, communities,
/// observations, thread summaries are not memory records). Counted, not imported.
const OUT_OF_SCOPE_GROUPS: &[&str] = &[
    "nodes",
    "entity_nodes",
    "communities",
    "observations",
    "thread_summaries",
    "document_summaries",
];

/// Convert a Zep/Graphiti graph export (JSON text) into v2 interchange lines.
pub fn convert_str(source: &str) -> Result<Conversion> {
    let root = parse_json(source)?;
    let obj = root.as_object().ok_or_else(|| Error::Validation {
        field: "root".into(),
        reason: "expected a Zep graph export object with `episodes` and `facts`/`edges` arrays"
            .into(),
    })?;

    let scope = string_field(obj, &["user_id", "graph_id"]);
    let namespace = scoped_namespace("zep", scope.as_deref());
    let mut stats = ConversionStats::default();
    let mut lines = Vec::new();
    // Uuids of episodes effectively converted; facts are constrained to these
    // so `derived_from` never references a key that was skipped/absent.
    let mut converted_episodes: BTreeSet<String> = BTreeSet::new();

    if let Some(episodes) = obj.get("episodes").and_then(JsonValue::as_array) {
        for item in episodes {
            if let Some(uuid) = convert_episode(item, &namespace, &mut lines, &mut stats) {
                converted_episodes.insert(uuid);
            }
        }
    }
    // `facts` wins over the `edges` alias. When both arrays are present the
    // `edges` one is droppable data with no destination → counted explicitly.
    let facts = obj.get("facts").and_then(JsonValue::as_array);
    let edges = obj.get("edges").and_then(JsonValue::as_array);
    if facts.is_some() {
        if let Some(edges) = edges {
            stats.discard_many("edges", edges.len());
        }
    }
    if let Some(facts) = facts.or(edges) {
        for item in facts {
            convert_fact(
                item,
                &namespace,
                &converted_episodes,
                &mut lines,
                &mut stats,
            );
        }
    }
    for group in OUT_OF_SCOPE_GROUPS {
        if let Some(array) = obj.get(*group).and_then(JsonValue::as_array) {
            stats.discard_many(group, array.len());
        }
    }

    Ok(Conversion { lines, stats })
}

/// Convert a Zep/Graphiti export file into v2 interchange lines.
pub fn convert_file(path: impl AsRef<Path>) -> Result<Conversion> {
    convert_str(&read_source_file(path.as_ref())?)
}

fn convert_episode(
    item: &JsonValue,
    namespace: &str,
    lines: &mut Vec<crate::sdk::types::MemoryExportLine>,
    stats: &mut ConversionStats,
) -> Option<String> {
    let Some(obj) = item.as_object() else {
        stats.skipped += 1;
        return None;
    };
    let Some(uuid) = string_field(obj, &["uuid", "episode_id"]) else {
        stats.skipped += 1;
        return None;
    };
    let Some(content) = string_field(obj, &["content"]) else {
        stats.skipped += 1;
        return None;
    };

    let created_at_ms = obj.get("created_at").and_then(timestamp_ms).unwrap_or(0);
    let mut line = base_line(
        namespace.to_string(),
        uuid.clone(),
        content,
        created_at_ms,
        created_at_ms,
    );
    line.metadata
        .insert("source".to_string(), Value::String("zep".to_string()));
    line.metadata
        .insert("source_id".to_string(), Value::String(uuid.clone()));
    line.metadata
        .insert("zep.kind".to_string(), Value::String("episode".to_string()));
    for field in [
        "source",
        "source_description",
        "role",
        "role_type",
        "thread_id",
        "document_id",
    ] {
        if let Some(value) = obj.get(field) {
            insert_meta(&mut line.metadata, &format!("zep.{field}"), value);
        }
    }
    if let Some(meta) = obj.get("metadata") {
        insert_meta(&mut line.metadata, "zep.metadata", meta);
    }
    preserve_unknown(
        obj,
        EPISODE_CONSUMED,
        SEARCH_TIME,
        "zep",
        &mut line.metadata,
        stats,
    );

    lines.push(line);
    stats.converted += 1;
    Some(uuid)
}

fn convert_fact(
    item: &JsonValue,
    namespace: &str,
    converted_episodes: &BTreeSet<String>,
    lines: &mut Vec<crate::sdk::types::MemoryExportLine>,
    stats: &mut ConversionStats,
) {
    let Some(obj) = item.as_object() else {
        stats.skipped += 1;
        return;
    };
    let Some(uuid) = string_field(obj, &["uuid", "edge_uuid"]) else {
        stats.skipped += 1;
        return;
    };
    let Some(fact) = string_field(obj, &["fact"]) else {
        stats.skipped += 1;
        return;
    };

    let created_at_ms = obj.get("created_at").and_then(timestamp_ms).unwrap_or(0);
    let valid_at_ms = obj
        .get("valid_at")
        .and_then(timestamp_ms)
        .unwrap_or(created_at_ms);
    let invalid_at_ms = obj.get("invalid_at").and_then(timestamp_ms);

    let mut line = base_line(
        namespace.to_string(),
        uuid.clone(),
        fact,
        created_at_ms,
        created_at_ms,
    );
    line.valid_at_ms = valid_at_ms;
    line.invalid_at_ms = invalid_at_ms;

    // Provenance: a fact is a derivation over its source episodes. Parents are
    // constrained to the episodes actually converted in this run, so
    // `derived_from` never references a key that was skipped or absent. A fact
    // left without parents cannot be `Derived` (v2 requires ≥1 parent) →
    // declared downgrade to `Asserted` + counter.
    let source_episodes: Vec<String> = obj
        .get("episodes")
        .and_then(JsonValue::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(JsonValue::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let mut unresolved = 0usize;
    let episodes: Vec<String> = source_episodes
        .iter()
        .filter(|uuid| {
            if converted_episodes.contains(*uuid) {
                true
            } else {
                unresolved += 1;
                false
            }
        })
        .cloned()
        .collect();
    if source_episodes.is_empty() {
        stats.discard("fact.episodes_missing");
    } else if unresolved > 0 {
        stats.discard_many("fact.episodes_unresolved", unresolved);
    }
    if episodes.is_empty() {
        // No resolvable parents: import as a direct claim (counted above when
        // the source declared parents).
    } else {
        line.confidence_class = ConfidenceClass::Derived;
        line.derived_from = episodes;
        line.confidence = default_confidence() * DERIVATION_DISCOUNT;
    }

    line.metadata
        .insert("source".to_string(), Value::String("zep".to_string()));
    line.metadata
        .insert("source_id".to_string(), Value::String(uuid));
    line.metadata
        .insert("zep.kind".to_string(), Value::String("fact".to_string()));
    for field in [
        "name",
        "source_node_uuid",
        "target_node_uuid",
        "source_node_name",
        "target_node_name",
        "source_node_labels",
        "target_node_labels",
        "scope",
        "hyperedge_uuid",
    ] {
        if let Some(value) = obj.get(field) {
            insert_meta(&mut line.metadata, &format!("zep.{field}"), value);
        }
    }
    if let Some(attributes) = obj.get("attributes") {
        insert_meta(&mut line.metadata, "zep.attributes", attributes);
    }
    preserve_unknown(
        obj,
        FACT_CONSUMED,
        FACT_DISCARDED,
        "zep",
        &mut line.metadata,
        stats,
    );

    lines.push(line);
    stats.converted += 1;
}
