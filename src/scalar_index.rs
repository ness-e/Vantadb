use crate::node::FieldValue;
use dashmap::DashMap;
use std::collections::HashMap;

/// Concurrent scalar index mapping field values to node IDs.
///
/// `field → value → [node_id]` hash map that turns
/// [`filter_field`](crate::storage::engine::StorageEngine::filter_field) from
/// a full table scan into an O(1) lookup (PERF-08).
pub(crate) struct ScalarIndex {
    /// Per-field maps of value to node ID list.
    indexes: DashMap<String, HashMap<FieldValue, Vec<u128>>>,
}

impl ScalarIndex {
    /// Create a new empty scalar index.
    pub fn new() -> Self {
        Self {
            indexes: DashMap::new(),
        }
    }

    /// Insert a node ID for a given field/value pair.
    pub fn insert(&self, field: &str, value: &FieldValue, node_id: u128) {
        let mut entry = self.indexes.entry(field.to_string()).or_default();
        entry.entry(value.clone()).or_default().push(node_id);
    }

    /// Remove a node ID from a given field/value pair.
    pub fn remove(&self, field: &str, value: &FieldValue, node_id: u128) {
        if let Some(mut entry) = self.indexes.get_mut(field) {
            if let Some(values) = entry.get_mut(value) {
                values.retain(|&id| id != node_id);
            }
        }
    }

    /// Look up node IDs by field and value.
    pub fn lookup(&self, field: &str, value: &FieldValue) -> Vec<u128> {
        self.indexes
            .get(field)
            .and_then(|entry| entry.get(value).cloned())
            .unwrap_or_default()
    }

    /// Remove a node from all index entries.
    pub fn remove_node(&self, node_id: u128) {
        for mut entry in self.indexes.iter_mut() {
            entry.retain(|_, ids| {
                ids.retain(|&id| id != node_id);
                !ids.is_empty()
            });
        }
    }

    /// Remove all entries for a given field.
    #[cfg(test)]
    pub fn clear_field(&self, field: &str) {
        self.indexes.remove(field);
    }

    /// Return all indexed field names.
    #[cfg(test)]
    pub fn field_names(&self) -> Vec<String> {
        self.indexes.iter().map(|e| e.key().clone()).collect()
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::FieldValue;

    #[test]
    fn test_scalar_index_insert_and_lookup() {
        let idx = ScalarIndex::new();
        idx.insert("color", &FieldValue::String("red".into()), 1);
        idx.insert("color", &FieldValue::String("red".into()), 2);
        idx.insert("color", &FieldValue::String("blue".into()), 3);

        let reds = idx.lookup("color", &FieldValue::String("red".into()));
        assert_eq!(reds.len(), 2);
        assert!(reds.contains(&1));
        assert!(reds.contains(&2));

        let blues = idx.lookup("color", &FieldValue::String("blue".into()));
        assert_eq!(blues, vec![3]);
    }

    #[test]
    fn test_scalar_index_lookup_missing_field() {
        let idx = ScalarIndex::new();
        let result = idx.lookup("nonexistent", &FieldValue::String("x".into()));
        assert!(result.is_empty());
    }

    #[test]
    fn test_scalar_index_remove() {
        let idx = ScalarIndex::new();
        idx.insert("size", &FieldValue::Int(10), 1);
        idx.insert("size", &FieldValue::Int(10), 2);
        idx.remove("size", &FieldValue::Int(10), 1);

        let result = idx.lookup("size", &FieldValue::Int(10));
        assert_eq!(result, vec![2]);
    }

    #[test]
    fn test_scalar_index_remove_last_entry_drops_key() {
        let idx = ScalarIndex::new();
        idx.insert("tag", &FieldValue::String("a".into()), 1);
        idx.remove("tag", &FieldValue::String("a".into()), 1);

        let result = idx.lookup("tag", &FieldValue::String("a".into()));
        assert!(result.is_empty());
    }

    #[test]
    fn test_scalar_index_remove_node() {
        let idx = ScalarIndex::new();
        idx.insert("a", &FieldValue::Int(1), 1);
        idx.insert("a", &FieldValue::Int(2), 2);
        idx.insert("b", &FieldValue::Int(1), 1);
        idx.remove_node(1);

        assert!(idx.lookup("a", &FieldValue::Int(1)).is_empty());
        assert!(idx.lookup("b", &FieldValue::Int(1)).is_empty());
        assert_eq!(idx.lookup("a", &FieldValue::Int(2)), vec![2]);
    }

    #[test]
    fn test_scalar_index_clear_field() {
        let idx = ScalarIndex::new();
        idx.insert("x", &FieldValue::Bool(true), 1);
        idx.insert("y", &FieldValue::Bool(false), 2);
        idx.clear_field("x");

        assert!(idx.lookup("x", &FieldValue::Bool(true)).is_empty());
        assert_eq!(idx.lookup("y", &FieldValue::Bool(false)), vec![2]);
    }

    #[test]
    fn test_scalar_index_field_names() {
        let idx = ScalarIndex::new();
        idx.insert("a", &FieldValue::Null, 1);
        idx.insert("b", &FieldValue::Null, 2);

        let mut names = idx.field_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_scalar_index_empty_new() {
        let idx = ScalarIndex::new();
        assert!(idx.field_names().is_empty());
    }

    #[test]
    fn test_scalar_index_insert_multiple_fields() {
        let idx = ScalarIndex::new();
        idx.insert("name", &FieldValue::String("alice".into()), 1);
        idx.insert("age", &FieldValue::Int(30), 1);
        idx.insert("age", &FieldValue::Int(25), 2);

        assert_eq!(
            idx.lookup("name", &FieldValue::String("alice".into())),
            vec![1]
        );
        assert_eq!(idx.lookup("age", &FieldValue::Int(30)), vec![1]);
        assert_eq!(idx.lookup("age", &FieldValue::Int(25)), vec![2]);
    }
}
