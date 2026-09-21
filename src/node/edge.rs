use serde::{Deserialize, Serialize};
use web_time::{SystemTime, UNIX_EPOCH};

/// Labeled directed edge with optional weight and reverse flag.
///
/// Label stored as `label_id: u32` referencing a `LabelIntern` map.
/// Saves ~20-28 bytes per edge vs storing a `String` inline.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Edge {
    /// Target node ID.
    pub target: u128,
    /// Interned edge label (use LabelIntern to resolve).
    pub label_id: u32,
    /// Edge weight (defaults to 1.0).
    pub weight: f32,
    /// Whether this is a reverse edge.
    #[serde(default)]
    pub reverse: bool,
    /// Unix epoch milliseconds when this edge was created.
    /// Postcard records written before this field existed end the buffer
    /// here; the manual `Deserialize` below reads it as `0` for them.
    pub created_at_ms: u64,
}

/// Manual `Deserialize` for [`Edge`].
///
/// postcard's `deserialize_struct` → `deserialize_tuple(fields.len())` fixes
/// the element count to the *current* struct shape (5 fields). When reading a
/// legacy record that predates `created_at_ms` (4 fields), postcard's
/// `SeqAccess::next_element_seed` returns `Err(DeserializeUnexpectedEnd)`
/// instead of `Ok(None)` once the buffer is exhausted, so `#[serde(default)]`
/// is never consulted. We therefore default the trailing field to `0`.
impl<'de> Deserialize<'de> for Edge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EdgeVisitor;

        impl<'de> serde::de::Visitor<'de> for EdgeVisitor {
            type Value = Edge;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("struct Edge")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let target = seq.next_element()?.unwrap_or_default();
                let label_id = seq.next_element()?.unwrap_or_default();
                let weight = seq.next_element()?.unwrap_or_default();
                let reverse = seq.next_element()?.unwrap_or_default();
                // Legacy records end here; postcard errors instead of `None`,
                // so swallow that and default to 0.
                let created_at_ms = seq.next_element::<u64>().ok().flatten().unwrap_or(0);
                Ok(Edge {
                    target,
                    label_id,
                    weight,
                    reverse,
                    created_at_ms,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut target = None;
                let mut label_id = None;
                let mut weight = None;
                let mut reverse = None;
                let mut created_at_ms = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "target" => target = Some(map.next_value()?),
                        "label_id" => label_id = Some(map.next_value()?),
                        "weight" => weight = Some(map.next_value()?),
                        "reverse" => reverse = Some(map.next_value()?),
                        "created_at_ms" => created_at_ms = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(Edge {
                    target: target.unwrap_or_default(),
                    label_id: label_id.unwrap_or_default(),
                    weight: weight.unwrap_or_default(),
                    reverse: reverse.unwrap_or_default(),
                    created_at_ms: created_at_ms.unwrap_or(0),
                })
            }
        }

        deserializer.deserialize_struct(
            "Edge",
            &["target", "label_id", "weight", "reverse", "created_at_ms"],
            EdgeVisitor,
        )
    }
}

/// Weights for computing per-node eviction scores.
/// Used by `StorageEngine::evict_cold_nodes()` to decide which nodes
/// to evict when under memory pressure.
#[derive(Debug, Clone, Copy)]
pub struct EvictionWeights {
    /// Weight for hit count.
    pub hits: f64,
    /// Weight for confidence score.
    pub confidence: f64,
    /// Weight for importance score.
    pub importance: f64,
    /// Weight for recency score.
    pub recency: f64,
}

impl Edge {
    /// Create an edge with default weight (1.0) and `reverse: false`.
    pub fn new(target: u128, label_id: u32) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: false,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create an edge with a custom weight.
    pub fn with_weight(target: u128, label_id: u32, weight: f32) -> Self {
        Self {
            target,
            label_id,
            weight,
            reverse: false,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create a reverse edge (used for bidirectional traversal).
    pub fn reverse(target: u128, label_id: u32) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: true,
            created_at_ms: edge_created_at_now(),
        }
    }

    /// Create a forward edge with an explicit creation timestamp (Unix ms).
    pub fn with_timestamp(target: u128, label_id: u32, created_at_ms: u64) -> Self {
        Self {
            target,
            label_id,
            weight: 1.0,
            reverse: false,
            created_at_ms,
        }
    }
}

/// Current Unix epoch time in milliseconds, used as the default edge creation
/// timestamp. Falls back to `0` if the system clock predates the epoch.
fn edge_created_at_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new() {
        let edge = Edge::new(42, 0);
        assert_eq!(edge.target, 42);
        assert_eq!(edge.label_id, 0);
        assert!((edge.weight - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_edge_with_weight() {
        let edge = Edge::with_weight(99, 1, 0.5);
        assert_eq!(edge.target, 99);
        assert_eq!(edge.label_id, 1);
        assert!((edge.weight - 0.5).abs() < f32::EPSILON);
    }

    // ── Temporal edge tests (COMP-021) ──

    /// Legacy 4-field shape persisted before `created_at_ms` existed.
    /// WAL/storage use postcard, so this byte-level round-trip proves
    /// `#[serde(default)]` keeps old datasets readable (`created_at_ms == 0`).
    #[derive(Serialize, Deserialize)]
    struct LegacyEdge {
        target: u128,
        label_id: u32,
        weight: f32,
        reverse: bool,
    }

    #[test]
    fn test_edge_backward_compat_postcard_default() {
        let legacy = LegacyEdge {
            target: 7,
            label_id: 3,
            weight: 0.5,
            reverse: false,
        };
        let bytes = postcard::to_allocvec(&legacy).unwrap();
        let edge: Edge = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(edge.created_at_ms, 0, "missing field must default to 0");
        assert_eq!(edge.target, 7);
        assert_eq!(edge.label_id, 3);
        assert!((edge.weight - 0.5).abs() < f32::EPSILON);
        assert!(!edge.reverse);
    }

    #[test]
    fn test_edge_with_timestamp() {
        let edge = Edge::with_timestamp(5, 2, 1_700_000_000_000);
        assert_eq!(edge.created_at_ms, 1_700_000_000_000);
        assert_eq!(edge.target, 5);
        assert!(!edge.reverse);
    }

    #[test]
    fn test_edge_default_timestamp_is_now() {
        let edge = Edge::new(1, 0);
        assert!(
            edge.created_at_ms > 0,
            "default created_at_ms must be wall-clock now, got {}",
            edge.created_at_ms
        );
    }
}
