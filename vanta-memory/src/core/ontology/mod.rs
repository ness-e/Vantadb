//! Ontological bitemporal triplet extraction and graph management (MEM0-04).
//!
//! Extracts structured triples `(subject, predicate, object, valid_at_ms, invalid_at_ms)`
//! during L2/L3 consolidation and inserts them as edges in VantaDB's unified graph.
//!
//! Invariants:
//! 1. When a new triplet contradicts a prior `(subject, predicate)` edge/record,
//!    the previous record's `invalid_at_ms` is stamped with `now_ms` without physical deletion.
//! 2. Historical queries via [`query_triplets_as_of`] support bitemporal `AS OF` point-in-time reads.

use crate::core::abstractions::MemoryRecord;
use crate::core::conversation::sanitize_component;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vantadb::sdk::{Embedded, MemoryInput, MemoryListOptions, NodeInput, Value};

/// A bitemporal structured knowledge triple (MEM0-04).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitemporalTriplet {
    /// Subject entity
    pub subject: String,
    /// Predicate relationship
    pub predicate: String,
    /// Object value or entity
    pub object: String,
    /// Unix millisecond timestamp when the fact became valid
    pub valid_at_ms: u64,
    /// Unix millisecond timestamp when the fact was invalidated (None = currently valid)
    pub invalid_at_ms: Option<u64>,
}

impl BitemporalTriplet {
    /// True when the triplet is valid as of `as_of_ms`
    pub fn is_valid_as_of(&self, as_of_ms: u64) -> bool {
        self.valid_at_ms <= as_of_ms && self.invalid_at_ms.is_none_or(|inv| inv > as_of_ms)
    }
}

/// Namespace where structured triplets are persisted for a session: `triplets/<session>`.
pub fn triplets_namespace(session_id: &str) -> String {
    format!("triplets/{}", sanitize_component(session_id, 128, false))
}

/// Deterministic node id for an ontological entity in the graph.
pub fn entity_node_id(entity: &str) -> u128 {
    let mut hash = 0xcbf29ce484222325u128;
    for byte in entity.trim().to_lowercase().bytes() {
        hash ^= u128::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Extract triplets heuristically from text or memory records.
pub fn extract_triplets_from_text(content: &str, now_ms: u64) -> Vec<BitemporalTriplet> {
    let mut triplets = Vec::new();
    let lower = content.trim().to_lowercase();

    // Standard pattern extractors
    let patterns = [
        ("prefers", "prefers"),
        ("likes", "likes"),
        ("dislikes", "dislikes"),
        ("lives in", "lives_in"),
        ("works as", "works_as"),
        ("works at", "works_at"),
        ("uses", "uses"),
        ("speaks", "speaks"),
        ("is a", "is_a"),
        ("is an", "is_a"),
        ("is", "is"),
    ];

    for line in lower.lines() {
        let trimmed = line.trim();
        for (pattern, pred) in &patterns {
            if let Some(idx) = trimmed.find(pattern) {
                let subj = trimmed[..idx]
                    .trim()
                    .trim_matches(|c: char| !c.is_alphanumeric());
                let obj = trimmed[idx + pattern.len()..]
                    .trim()
                    .trim_matches(|c: char| !c.is_alphanumeric());
                if !subj.is_empty() && !obj.is_empty() {
                    triplets.push(BitemporalTriplet {
                        subject: subj.to_string(),
                        predicate: pred.to_string(),
                        object: obj.to_string(),
                        valid_at_ms: now_ms,
                        invalid_at_ms: None,
                    });
                    break;
                }
            }
        }
    }
    triplets
}

/// Read all triplets in a session namespace.
pub fn read_session_triplets(
    db: &Embedded,
    session_id: &str,
) -> Result<Vec<(String, BitemporalTriplet)>, String> {
    let ns = triplets_namespace(session_id);
    let options = MemoryListOptions {
        limit: 1000,
        ..Default::default()
    };
    let page = db.list(&ns, options).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for rec in page.records {
        if let Ok(trip) = serde_json::from_str::<BitemporalTriplet>(&rec.payload) {
            result.push((rec.key, trip));
        }
    }
    Ok(result)
}

/// Query triplets valid as of `as_of_ms` (AS OF historical query support, MEM0-04).
pub fn query_triplets_as_of(
    db: &Embedded,
    session_id: &str,
    as_of_ms: u64,
) -> Result<Vec<BitemporalTriplet>, String> {
    let all = read_session_triplets(db, session_id)?;
    Ok(all
        .into_iter()
        .map(|(_, t)| t)
        .filter(|t| t.is_valid_as_of(as_of_ms))
        .collect())
}

/// Apply a new triplet to the database and graph:
/// If a prior triplet with same `(subject, predicate)` exists with open validity,
/// mark `invalid_at_ms = now_ms` without deleting it physically, and insert the new triplet.
pub fn apply_triplet(
    db: &Embedded,
    session_id: &str,
    triplet: BitemporalTriplet,
    now_ms: u64,
) -> Result<(), String> {
    apply_triplet_with_source(db, session_id, triplet, now_ms, None)
}

/// Apply a new triplet with explicit source episode ID for lineage tracking (TEC-15).
pub fn apply_triplet_with_source(
    db: &Embedded,
    session_id: &str,
    triplet: BitemporalTriplet,
    now_ms: u64,
    source_episode_id: Option<&str>,
) -> Result<(), String> {
    let ns = triplets_namespace(session_id);
    let existing = read_session_triplets(db, session_id)?;

    // Check for prior contradicting triplets with same (subject, predicate)
    for (key, mut prior) in existing {
        if prior.subject.eq_ignore_ascii_case(&triplet.subject)
            && prior.predicate.eq_ignore_ascii_case(&triplet.predicate)
            && prior.invalid_at_ms.is_none()
            && !prior.object.eq_ignore_ascii_case(&triplet.object)
        {
            // Stamp invalid_at_ms on prior triplet
            prior.invalid_at_ms = Some(now_ms);
            let payload = serde_json::to_string(&prior).map_err(|e| e.to_string())?;
            let mut metadata = BTreeMap::new();
            metadata.insert("invalid_at_ms".to_string(), Value::Int(now_ms as i64));
            db.put(MemoryInput {
                namespace: ns.clone(),
                key: key.clone(),
                payload,
                metadata: metadata.into(),
                valid_at_ms: Some(prior.valid_at_ms),
                ..Default::default()
            })
            .map_err(|e| e.to_string())?;
            let _ = db.forget(
                &ns,
                vantadb::sdk::ForgetTarget::Key(key),
                vantadb::sdk::ForgetMode::Soft,
            );
        }
    }

    // Insert new triplet record
    let new_key = format!(
        "{}_{}_{}",
        crate::core::conversation::sanitize_key(&triplet.subject),
        crate::core::conversation::sanitize_key(&triplet.predicate),
        now_ms
    );
    let payload = serde_json::to_string(&triplet).map_err(|e| e.to_string())?;
    let mut metadata = BTreeMap::new();
    if let Some(src) = source_episode_id {
        metadata.insert(
            "source_episode_ids".to_string(),
            Value::ListString(vec![src.to_string()]),
        );
        metadata.insert(
            "derived_from".to_string(),
            Value::ListString(vec![src.to_string()]),
        );
    }
    db.put(MemoryInput {
        namespace: ns.clone(),
        key: new_key,
        payload,
        metadata: metadata.into(),
        valid_at_ms: Some(triplet.valid_at_ms),
        ..Default::default()
    })
    .map_err(|e| e.to_string())?;

    // Ensure subject and object nodes exist in the unified graph
    let subj_id = entity_node_id(&triplet.subject);
    let obj_id = entity_node_id(&triplet.object);

    let mut subj_fields = BTreeMap::new();
    subj_fields.insert("name".to_string(), Value::String(triplet.subject.clone()));
    let _ = db.insert_node(NodeInput {
        id: subj_id,
        content: Some(triplet.subject.clone()),
        vector: None,
        fields: subj_fields,
    });

    let mut obj_fields = BTreeMap::new();
    obj_fields.insert("name".to_string(), Value::String(triplet.object.clone()));
    let _ = db.insert_node(NodeInput {
        id: obj_id,
        content: Some(triplet.object.clone()),
        vector: None,
        fields: obj_fields,
    });

    // Add directed edge in graph with creation timestamp
    let _ = db.add_edge(
        subj_id,
        obj_id,
        &triplet.predicate,
        None,
        Some(triplet.valid_at_ms),
    );

    Ok(())
}

/// Extract and apply bitemporal triplets from memory records during L2 consolidation.
pub fn extract_and_apply_bitemporal_triplets(
    db: &Embedded,
    session_id: &str,
    records: &[MemoryRecord],
    now_ms: u64,
) -> Result<usize, String> {
    let mut count = 0;
    for rec in records {
        let extracted = extract_triplets_from_text(&rec.content, now_ms);
        for trip in extracted {
            apply_triplet_with_source(db, session_id, trip, now_ms, Some(&rec.id))?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vantadb::config::Config;
    use vantadb::storage::BackendKind;

    fn open_test_db() -> Embedded {
        Embedded::open_with_config(Config {
            backend_kind: BackendKind::InMemory,
            ..Config::default()
        })
        .unwrap()
    }

    #[test]
    fn test_bitemporal_triplet_contradiction_and_as_of() {
        let db = open_test_db();
        let session = "sess_triplets_test";

        let t0 = 1000u64;
        let t1 = 2000u64;

        // Triplet 1 at t0: user lives_in Madrid
        let trip1 = BitemporalTriplet {
            subject: "user".to_string(),
            predicate: "lives_in".to_string(),
            object: "Madrid".to_string(),
            valid_at_ms: t0,
            invalid_at_ms: None,
        };
        apply_triplet(&db, session, trip1, t0).unwrap();

        // Query AS OF t0: should find Madrid
        let as_of_t0 = query_triplets_as_of(&db, session, t0).unwrap();
        assert_eq!(as_of_t0.len(), 1);
        assert_eq!(as_of_t0[0].object, "Madrid");
        assert!(as_of_t0[0].invalid_at_ms.is_none());

        // Triplet 2 at t1: user lives_in London (contradicts Madrid)
        let trip2 = BitemporalTriplet {
            subject: "user".to_string(),
            predicate: "lives_in".to_string(),
            object: "London".to_string(),
            valid_at_ms: t1,
            invalid_at_ms: None,
        };
        apply_triplet(&db, session, trip2, t1).unwrap();

        // Historical query AS OF t0: STILL finds Madrid (not physically deleted!)
        let hist = query_triplets_as_of(&db, session, t0).unwrap();
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].object, "Madrid");

        // Current query AS OF t1: finds London
        let current = query_triplets_as_of(&db, session, t1).unwrap();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].object, "London");

        // Raw records in triplets namespace must contain BOTH records
        let all = read_session_triplets(&db, session).unwrap();
        assert_eq!(
            all.len(),
            2,
            "both historical and current triplets preserved on disk"
        );
        let madrid_rec = all.iter().find(|(_, t)| t.object == "Madrid").unwrap();
        assert_eq!(
            madrid_rec.1.invalid_at_ms,
            Some(t1),
            "Madrid was invalidated at t1"
        );
    }
}
