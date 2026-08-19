//! Version-history retention for persistent memory records (VS-CORE-07).
//!
//! Snapshot = the record as written on each `put`, serialized with postcard
//! and stored under [`BackendPartition::Versions`] with key
//! `ns_len(u32 LE) ‖ ns ‖ key_len(u32 LE) ‖ key ‖ version(u64 BE)`.
//! The version sits at the end in big-endian so a `scan_prefix` over
//! `ns_len‖ns‖key_len‖key` yields ascending version order (v1 < v2 < v10),
//! and `get_version(vN)` is a single point-read.
//!
//! Durability class: **best-effort post-commit**, same as `ShreddedRowStore`
//! and the derived indexes — a crash between the WAL commit and the snapshot
//! write leaves a version gap, never corruption. Callers use `let _ =`.
//! (Hardening to a `WalRecord::VersionSnapshot` variant is deferred debt for
//! P27 if crash-exact history is ever required.)

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::node::SparseVector;
use crate::sdk::types::{VantaMemoryMetadata, VantaMemoryRecord};
use crate::storage::engine::StorageEngine;
use serde::{Deserialize, Serialize};

/// Size in bytes of the trailing big-endian version field.
const VERSION_LEN: usize = 8;

/// Build the exact key for a (namespace, key, version) snapshot.
pub(crate) fn version_key(namespace: &str, key: &str, version: u64) -> Vec<u8> {
    let mut k = Vec::with_capacity(4 + namespace.len() + 4 + key.len() + VERSION_LEN);
    k.extend_from_slice(&(namespace.len() as u32).to_le_bytes());
    k.extend_from_slice(namespace.as_bytes());
    k.extend_from_slice(&(key.len() as u32).to_le_bytes());
    k.extend_from_slice(key.as_bytes());
    k.extend_from_slice(&version.to_be_bytes());
    k
}

/// Build the shared prefix for every version of a (namespace, key).
/// A strict prefix of [`version_key`] for any version.
pub(crate) fn version_prefix(namespace: &str, key: &str) -> Vec<u8> {
    let mut k = Vec::with_capacity(4 + namespace.len() + 4 + key.len());
    k.extend_from_slice(&(namespace.len() as u32).to_le_bytes());
    k.extend_from_slice(namespace.as_bytes());
    k.extend_from_slice(&(key.len() as u32).to_le_bytes());
    k.extend_from_slice(key.as_bytes());
    k
}

/// Postcard-safe mirror of [`VantaMemoryRecord`].
///
/// The public record carries `node_id: u128` through `u128_serde`, whose
/// `#[serde(untagged)]` deserializer requires `deserialize_any` — which
/// postcard intentionally does not implement ("structures not known at
/// compile time"). This private mirror serializes `node_id` as a plain
/// string instead, so the snapshot wire format roundtrips 1:1 through
/// postcard without touching the public struct's serde behavior (JSON /
/// export / binding compat unchanged). Converting back yields an identical
/// `VantaMemoryRecord`.
#[derive(Serialize, Deserialize)]
struct SnapshotRecord {
    namespace: String,
    key: String,
    payload: String,
    metadata: VantaMemoryMetadata,
    created_at_ms: u64,
    updated_at_ms: u64,
    version: u64,
    #[serde(with = "node_id_str")]
    node_id: u128,
    vector: Option<Vec<f32>>,
    sparse_vector: Option<SparseVector>,
    expires_at_ms: Option<u64>,
}

/// `node_id` as decimal string — postcard-safe (no untagged enums).
mod node_id_str {
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(v: &u128, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl From<&VantaMemoryRecord> for SnapshotRecord {
    fn from(r: &VantaMemoryRecord) -> Self {
        Self {
            namespace: r.namespace.clone(),
            key: r.key.clone(),
            payload: r.payload.clone(),
            metadata: r.metadata.clone(),
            created_at_ms: r.created_at_ms,
            updated_at_ms: r.updated_at_ms,
            version: r.version,
            node_id: r.node_id,
            vector: r.vector.clone(),
            sparse_vector: r.sparse_vector.clone(),
            expires_at_ms: r.expires_at_ms,
        }
    }
}

impl From<SnapshotRecord> for VantaMemoryRecord {
    fn from(s: SnapshotRecord) -> Self {
        Self {
            namespace: s.namespace,
            key: s.key,
            payload: s.payload,
            metadata: s.metadata,
            created_at_ms: s.created_at_ms,
            updated_at_ms: s.updated_at_ms,
            version: s.version,
            node_id: s.node_id,
            vector: s.vector,
            sparse_vector: s.sparse_vector,
            expires_at_ms: s.expires_at_ms,
        }
    }
}

fn encode_snapshot(record: &VantaMemoryRecord) -> Result<Vec<u8>> {
    postcard::to_allocvec(&SnapshotRecord::from(record)).map_err(VantaError::serialization)
}

fn decode_snapshot(bytes: &[u8]) -> Result<VantaMemoryRecord> {
    postcard::from_bytes::<SnapshotRecord>(bytes)
        .map(VantaMemoryRecord::from)
        .map_err(VantaError::serialization)
}

/// Write the snapshot of a single record (used by `put_one`), then evict
/// oldest versions beyond `limit` (FIFO). Best-effort: propagates errors so
/// callers can choose to swallow them (`let _ =`).
pub(crate) fn write_snapshot(
    engine: &StorageEngine,
    record: &VantaMemoryRecord,
    limit: Option<usize>,
) -> Result<()> {
    let key = version_key(&record.namespace, &record.key, record.version);
    let value = encode_snapshot(record)?;
    engine.put_to_partition(BackendPartition::Versions, &key, &value)?;
    evict_overflow(engine, &record.namespace, &record.key, limit)
}

/// Write the snapshots of a whole `put_batch` chunk in ONE atomic
/// `write_batch`, then evict overflow per distinct key. Best-effort.
pub(crate) fn write_snapshot_batch(
    engine: &StorageEngine,
    records: &[VantaMemoryRecord],
    limit: Option<usize>,
) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }
    let mut ops = Vec::with_capacity(records.len());
    let mut keys: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    for record in records {
        ops.push(BackendWriteOp::Put {
            partition: BackendPartition::Versions,
            key: version_key(&record.namespace, &record.key, record.version),
            value: encode_snapshot(record)?,
        });
        keys.insert((record.namespace.clone(), record.key.clone()));
    }
    engine.write_backend_batch(ops)?;
    for (ns, key) in keys {
        evict_overflow(engine, &ns, &key, limit)?;
    }
    Ok(())
}

/// Delete every retained version of a key (used by `delete` and
/// `purge_expired`). Best-effort class like the rest of this module.
pub(crate) fn purge_key(engine: &StorageEngine, namespace: &str, key: &str) -> Result<()> {
    let prefix = version_prefix(namespace, key);
    let entries = engine.scan_partition_prefix(BackendPartition::Versions, &prefix)?;
    if entries.is_empty() {
        return Ok(());
    }
    let ops = entries
        .into_iter()
        .map(|(k, _)| BackendWriteOp::Delete {
            partition: BackendPartition::Versions,
            key: k,
        })
        .collect();
    engine.write_backend_batch(ops)
}

/// Fetch the record as it was at `version` (single point-read).
pub(crate) fn get_version(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
    version: u64,
) -> Result<Option<VantaMemoryRecord>> {
    let k = version_key(namespace, key, version);
    let Some(bytes) = engine.get_from_partition(BackendPartition::Versions, &k)? else {
        return Ok(None);
    };
    decode_snapshot(&bytes).map(Some)
}

/// List every retained version of a key, ascending (v1..vN).
pub(crate) fn versions(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
) -> Result<Vec<VantaMemoryRecord>> {
    let prefix = version_prefix(namespace, key);
    let entries = engine.scan_partition_prefix(BackendPartition::Versions, &prefix)?;
    entries
        .into_iter()
        .map(|(_, bytes)| decode_snapshot(&bytes))
        .collect()
}

/// FIFO eviction: once `limit` entries exist for a key, drop the oldest
/// (scan_prefix is ascending, so the first entries are the oldest).
fn evict_overflow(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
    limit: Option<usize>,
) -> Result<()> {
    let Some(limit) = limit else {
        return Ok(());
    };
    let prefix = version_prefix(namespace, key);
    let entries = engine.scan_partition_prefix(BackendPartition::Versions, &prefix)?;
    let overflow = entries.len().saturating_sub(limit);
    if overflow == 0 {
        return Ok(());
    }
    let ops = entries
        .into_iter()
        .take(overflow)
        .map(|(k, _)| BackendWriteOp::Delete {
            partition: BackendPartition::Versions,
            key: k,
        })
        .collect();
    engine.write_backend_batch(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendKind;
    use crate::config::VantaConfig;
    use crate::node::SparseVector;
    use crate::sdk::builder::VantaEmbedded;
    use crate::sdk::types::{VantaMemoryInput, VantaMemoryMetadata};
    use std::collections::BTreeMap;

    fn open_db(limit: Option<usize>) -> VantaEmbedded {
        VantaEmbedded::open_with_config(VantaConfig {
            storage_path: ":memory:".into(),
            backend_kind: BackendKind::InMemory,
            version_history_limit: limit,
            ..Default::default()
        })
        .expect("open in-memory database")
    }

    #[test]
    fn version_key_roundtrips() {
        let k = version_key("docs", "greeting", 1);
        let p = version_prefix("docs", "greeting");
        assert!(k.starts_with(&p));
        // trailing 8 bytes = version BE
        let ver = u64::from_be_bytes(k[k.len() - 8..].try_into().unwrap());
        assert_eq!(ver, 1);
        // prefixes are unambiguous: "a" vs "ab" keys do not collide
        let ka = version_prefix("ns", "a");
        let kab = version_prefix("ns", "ab");
        assert!(!kab.starts_with(&ka) || kab.len() != ka.len() + 1);
    }

    #[test]
    fn put_snapshots_each_version_and_get_version_roundtrips() {
        let db = open_db(None);
        db.put(VantaMemoryInput::new("docs", "greeting", "hello"))
            .expect("put v1");
        db.put(VantaMemoryInput::new("docs", "greeting", "hola"))
            .expect("put v2");
        db.put(VantaMemoryInput::new("docs", "greeting", "bonjour"))
            .expect("put v3");

        let all = db.versions("docs", "greeting").expect("versions");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].payload, "hello");
        assert_eq!(all[1].payload, "hola");
        assert_eq!(all[2].payload, "bonjour");
        assert_eq!(
            all.iter().map(|r| r.version).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );

        let live = db.get("docs", "greeting").expect("get").expect("live");
        assert_eq!(live.version, 3);

        let v2 = db
            .get_version("docs", "greeting", 2)
            .expect("get_version")
            .expect("v2");
        assert_eq!(v2.payload, "hola");
    }

    #[test]
    fn get_version_missing_returns_none() {
        let db = open_db(None);
        db.put(VantaMemoryInput::new("docs", "greeting", "hello"))
            .expect("put");
        assert!(db
            .get_version("docs", "greeting", 999)
            .expect("get_version")
            .is_none());
        assert!(db
            .get_version("docs", "missing", 1)
            .expect("get_version")
            .is_none());
        assert!(db.versions("docs", "missing").expect("versions").is_empty());
    }

    #[test]
    fn put_batch_with_duplicate_keys_snapshots_bump_sequence() {
        let db = open_db(None);
        db.put_batch(vec![
            VantaMemoryInput::new("docs", "k", "one"),
            VantaMemoryInput::new("docs", "k", "two"),
            VantaMemoryInput::new("docs", "other", "x"),
        ])
        .expect("put_batch");

        let all = db.versions("docs", "k").expect("versions");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].payload, "one");
        assert_eq!(all[1].payload, "two");
        assert_eq!(all[1].version, 2);
    }

    #[test]
    fn cap_evicts_oldest_fifo() {
        let db = open_db(Some(2));
        for i in 1..=3 {
            db.put(VantaMemoryInput::new("docs", "k", format!("v{i}")))
                .expect("put");
        }
        let all = db.versions("docs", "k").expect("versions");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].version, 2);
        assert_eq!(all[1].version, 3);
        // evicted v1 is gone
        assert!(db
            .get_version("docs", "k", 1)
            .expect("get_version")
            .is_none());
    }

    #[test]
    fn delete_purges_history() {
        let db = open_db(None);
        db.put(VantaMemoryInput::new("docs", "k", "one"))
            .expect("put v1");
        db.put(VantaMemoryInput::new("docs", "k", "two"))
            .expect("put v2");
        assert_eq!(db.versions("docs", "k").expect("versions").len(), 2);

        assert!(db.delete("docs", "k").expect("delete"));
        assert!(db.versions("docs", "k").expect("versions").is_empty());
        assert!(db.get("docs", "k").expect("get").is_none());
    }

    #[test]
    fn purge_expired_removes_snapshots() {
        let db = open_db(None);
        db.put(VantaMemoryInput {
            namespace: "docs".into(),
            key: "exp".into(),
            payload: "doomed".into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: Some(1),
        })
        .expect("put with ttl");
        db.put(VantaMemoryInput::new("docs", "keep", "stays"))
            .expect("put no ttl");
        assert_eq!(db.versions("docs", "exp").expect("versions").len(), 1);

        std::thread::sleep(std::time::Duration::from_millis(5));
        assert_eq!(db.purge_expired().expect("purge"), 1);
        assert!(db.versions("docs", "exp").expect("versions").is_empty());
        assert_eq!(db.versions("docs", "keep").expect("versions").len(), 1);
    }

    #[test]
    fn import_does_not_generate_snapshots() {
        let db = open_db(None);
        let record = db
            .put(VantaMemoryInput::new("docs", "src", "one"))
            .expect("put source");
        assert_eq!(db.versions("docs", "src").expect("versions").len(), 1);

        // import via put_record_exact writes exact versions without snapshots
        let mut imported = record.clone();
        imported.key = "dst".into();
        imported.node_id = crate::sdk::serialization::memory_node_id("docs", "dst");
        db.put_record_exact(imported).expect("import exact");
        assert!(db.versions("docs", "dst").expect("versions").is_empty());
    }

    #[test]
    fn postcard_roundtrip_with_vector_sparse_metadata() {
        let rec = VantaMemoryRecord {
            namespace: "docs".into(),
            key: "vec".into(),
            payload: "sparse + dense".into(),
            metadata: [(
                "lang".to_string(),
                crate::node::FieldValue::String("en".into()).into(),
            )]
            .into_iter()
            .collect(),
            created_at_ms: 1,
            updated_at_ms: 2,
            version: 7,
            node_id: 42,
            vector: Some(vec![0.5; 1536]),
            sparse_vector: Some(SparseVector(BTreeMap::from([(0, 1.0), (5, 0.25)]))),
            expires_at_ms: Some(1_700_000_000_000),
        };
        let bytes = encode_snapshot(&rec).expect("serialize");
        let back = decode_snapshot(&bytes).expect("deserialize");
        assert_eq!(back, rec);
    }

    #[test]
    fn versions_backend_partition_roundtrip() {
        // InMemory + Fjall share the trait; verify the partition is reachable
        // through both backends via a fresh DB (backward-compat: opening a DB
        // that predates the feature must not error, history starts empty).
        for kind in [BackendKind::InMemory, BackendKind::Fjall] {
            let cfg = VantaConfig {
                storage_path: if kind == BackendKind::Fjall {
                    std::env::temp_dir()
                        .join(format!("vantadb-versions-test-{}", std::process::id()))
                        .to_string_lossy()
                        .into_owned()
                } else {
                    ":memory:".into()
                },
                backend_kind: kind,
                ..Default::default()
            };
            if kind == BackendKind::Fjall {
                let _ = std::fs::remove_dir_all(&cfg.storage_path);
            }
            let db = VantaEmbedded::open_with_config(cfg).expect("open");
            // pre-feature DB: partition exists but is empty
            assert!(db.versions("docs", "nope").expect("versions").is_empty());
            db.put(VantaMemoryInput::new("docs", "k", "v1"))
                .expect("put");
            assert_eq!(db.versions("docs", "k").expect("versions").len(), 1);
            if kind == BackendKind::Fjall {
                db.close().expect("close");
                let _ = std::fs::remove_dir_all(
                    std::env::temp_dir()
                        .join(format!("vantadb-versions-test-{}", std::process::id())),
                );
            }
        }
    }

    #[test]
    fn version_key_prefixes_do_not_collide_across_keys() {
        // Key format must keep "a" and "ab" (and namespaces "x" vs "xy")
        // separate: length prefixes make the boundary unambiguous.
        let db = open_db(None);
        db.put(VantaMemoryInput::new("x", "a", "one")).expect("put");
        db.put(VantaMemoryInput::new("x", "ab", "two"))
            .expect("put");
        db.put(VantaMemoryInput::new("xy", "a", "three"))
            .expect("put");
        assert_eq!(db.versions("x", "a").expect("versions").len(), 1);
        assert_eq!(db.versions("x", "ab").expect("versions").len(), 1);
        assert_eq!(db.versions("xy", "a").expect("versions").len(), 1);
    }
}
