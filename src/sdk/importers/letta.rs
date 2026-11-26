//! Letta Agent File (`.af`) → VantaDB importer.
//!
//! `.af` is Letta's open serialization format for stateful agents. This
//! importer maps the two memory-bearing components — **memory blocks**
//! (in-context segments: persona, human, …) and the **message history** — plus
//! their agent linkage. Agent configuration (model settings, tools, tool
//! rules, secrets) is not memory and is counted as a declared discard.
//!
//! Known format limits (declared, not silently dropped): `.af` does not
//! include archival-memory passages nor data-source file contents upstream
//! ("We currently do not support Passages", agent-file README); a `files` /
//! `sources` array is a folder reference list and is counted as a discard.
//!
//! Mapping table and declared discards: `docs/api/MEMORY_INTERCHANGE_FORMAT.md`.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value as JsonValue;

use super::{
    base_line, content_key, insert_meta, parse_json, preserve_unknown, read_source_file,
    scoped_namespace, string_field, timestamp_ms, Conversion, ConversionStats,
};
use crate::error::{Error, Result};
use crate::sdk::types::Value;

const BLOCK_CONSUMED: &[&str] = &[
    "id",
    "block_id",
    "value",
    "label",
    "description",
    "limit",
    "read_only",
    "metadata",
];

const MESSAGE_CONSUMED: &[&str] = &[
    "id",
    "content",
    "created_at",
    "role",
    "type",
    "model",
    "in_context",
];

/// Agent fields consumed by the mapping (the rest is agent configuration, not
/// memory: counted per field as `agent.<field>`).
const AGENT_CONSUMED: &[&str] = &["name", "id", "messages", "block_ids", "memory_blocks"];

/// Top-level groups outside the memory scope (folder references and tooling
/// definitions): counted, not imported.
const OUT_OF_SCOPE_GROUPS: &[&str] = &["files", "sources", "tools", "groups", "mcp_servers"];

/// Convert a Letta `.af` agent file (JSON text) into v2 interchange lines.
pub fn convert_str(source: &str) -> Result<Conversion> {
    let root = parse_json(source)?;
    let obj = root.as_object().ok_or_else(|| Error::Validation {
        field: "root".into(),
        reason: "expected a Letta `.af` object with an `agents` array".into(),
    })?;

    let agents: &[JsonValue] = obj
        .get("agents")
        .and_then(JsonValue::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut stats = ConversionStats::default();
    let mut lines = Vec::new();

    // 1. Block → agent linkage (a block is emitted once, referenced by all).
    let mut block_agents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for agent in agents {
        let Some(agent) = agent.as_object() else {
            continue;
        };
        let Some(name) = agent_name(agent) else {
            continue;
        };
        if let Some(ids) = agent.get("block_ids").and_then(JsonValue::as_array) {
            for id in ids.iter().filter_map(JsonValue::as_str) {
                let agents = block_agents.entry(id.to_string()).or_default();
                if !agents.contains(&name) {
                    agents.push(name.clone());
                }
            }
        }
    }

    // 2. Top-level blocks (one record per block, in file order).
    if let Some(blocks) = obj.get("blocks").and_then(JsonValue::as_array) {
        for block in blocks {
            convert_block(block, &block_agents, &mut lines, &mut stats);
        }
    }

    // 3. Message history per agent + agent configuration discards.
    for agent in agents {
        let Some(agent) = agent.as_object() else {
            continue;
        };
        let Some(name) = agent_name(agent) else {
            continue;
        };
        convert_messages(agent, &name, &mut lines, &mut stats);
        for (field, value) in agent {
            if AGENT_CONSUMED.contains(&field.as_str()) {
                continue;
            }
            match value {
                JsonValue::Null => {}
                JsonValue::Array(items) => {
                    stats.discard_many(&format!("agent.{field}"), items.len());
                }
                JsonValue::Object(map) if map.is_empty() => {}
                _ => stats.discard(&format!("agent.{field}")),
            }
        }
    }

    // 4. Declared out-of-scope groups.
    for group in OUT_OF_SCOPE_GROUPS {
        if let Some(array) = obj.get(*group).and_then(JsonValue::as_array) {
            stats.discard_many(group, array.len());
        }
    }

    Ok(Conversion { lines, stats })
}

/// Convert a Letta `.af` file into v2 interchange lines.
pub fn convert_file(path: impl AsRef<Path>) -> Result<Conversion> {
    convert_str(&read_source_file(path.as_ref())?)
}

fn agent_name(agent: &serde_json::Map<String, JsonValue>) -> Option<String> {
    string_field(agent, &["name", "id"])
}

fn convert_block(
    item: &JsonValue,
    block_agents: &BTreeMap<String, Vec<String>>,
    lines: &mut Vec<crate::sdk::types::MemoryExportLine>,
    stats: &mut ConversionStats,
) {
    let Some(obj) = item.as_object() else {
        stats.skipped += 1;
        return;
    };
    let Some(value) = string_field(obj, &["value"]) else {
        stats.skipped += 1;
        return;
    };
    let id = string_field(obj, &["id", "block_id"]);
    let label = string_field(obj, &["label"]);
    let key = id
        .clone()
        .or_else(|| label.clone())
        .unwrap_or_else(|| content_key("letta", "letta/blocks", &value));

    // `.af` carries no block timestamps: the documented `0` sentinel applies.
    let mut line = base_line("letta/blocks".to_string(), key, value, 0, 0);
    line.metadata
        .insert("source".to_string(), Value::String("letta".to_string()));
    line.metadata
        .insert("letta.kind".to_string(), Value::String("block".to_string()));
    if let Some(id) = &id {
        line.metadata
            .insert("source_id".to_string(), Value::String(id.clone()));
    }
    if let Some(label) = &label {
        line.metadata
            .insert("letta.label".to_string(), Value::String(label.clone()));
    }
    for field in ["description", "limit", "read_only", "metadata"] {
        if let Some(field_value) = obj.get(field) {
            let meta_key = if field == "metadata" {
                "letta.block_metadata".to_string()
            } else {
                format!("letta.{field}")
            };
            insert_meta(&mut line.metadata, &meta_key, field_value);
        }
    }
    if let Some(id) = &id {
        if let Some(agent_names) = block_agents.get(id) {
            if !agent_names.is_empty() {
                line.metadata.insert(
                    "letta.agents".to_string(),
                    Value::ListString(agent_names.clone()),
                );
            }
        }
    }
    preserve_unknown(obj, BLOCK_CONSUMED, &[], "letta", &mut line.metadata, stats);

    lines.push(line);
    stats.converted += 1;
}

fn convert_messages(
    agent: &serde_json::Map<String, JsonValue>,
    agent_name: &str,
    lines: &mut Vec<crate::sdk::types::MemoryExportLine>,
    stats: &mut ConversionStats,
) {
    let Some(messages) = agent.get("messages").and_then(JsonValue::as_array) else {
        return;
    };
    let namespace = scoped_namespace("letta", Some(&format!("{agent_name}/messages")));
    for message in messages {
        let Some(obj) = message.as_object() else {
            stats.skipped += 1;
            continue;
        };
        let payload = match obj.get("content") {
            Some(JsonValue::String(text)) if !text.trim().is_empty() => text.clone(),
            Some(JsonValue::Array(parts)) => {
                // Letta structured content is an array of content parts. Text
                // parts are the payload; non-text parts (images, tool
                // artifacts) have no v2 destination and are counted. Empty or
                // text-less arrays are tool-call messages — real `.af` exports
                // use `content: []` + `tool_calls` — so they are skipped, as
                // declared in the mapping tables.
                let mut texts: Vec<&str> = Vec::new();
                let mut non_text_parts = 0usize;
                for part in parts {
                    match part
                        .get("text")
                        .and_then(JsonValue::as_str)
                        .filter(|text| !text.trim().is_empty())
                    {
                        Some(text) => texts.push(text),
                        None => non_text_parts += 1,
                    }
                }
                stats.discard_many("message.content_parts", non_text_parts);
                if texts.is_empty() {
                    stats.skipped += 1;
                    continue;
                }
                texts.join("\n")
            }
            // Tool-call / tool-return messages without textual content are
            // not memory.
            _ => {
                stats.skipped += 1;
                continue;
            }
        };
        let id = string_field(obj, &["id", "message_id"]);
        let key = id
            .clone()
            .unwrap_or_else(|| content_key("letta", &namespace, &payload));
        let created_at_ms = obj.get("created_at").and_then(timestamp_ms).unwrap_or(0);

        let mut line = base_line(
            namespace.clone(),
            key,
            payload,
            created_at_ms,
            created_at_ms,
        );
        line.metadata
            .insert("source".to_string(), Value::String("letta".to_string()));
        line.metadata.insert(
            "letta.kind".to_string(),
            Value::String("message".to_string()),
        );
        line.metadata.insert(
            "letta.agent".to_string(),
            Value::String(agent_name.to_string()),
        );
        if let Some(id) = &id {
            line.metadata
                .insert("source_id".to_string(), Value::String(id.clone()));
        }
        for field in ["role", "type", "model", "in_context"] {
            if let Some(field_value) = obj.get(field) {
                insert_meta(&mut line.metadata, &format!("letta.{field}"), field_value);
            }
        }
        preserve_unknown(
            obj,
            MESSAGE_CONSUMED,
            &[],
            "letta",
            &mut line.metadata,
            stats,
        );

        lines.push(line);
        stats.converted += 1;
    }
}
