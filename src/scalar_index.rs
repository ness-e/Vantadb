#![allow(dead_code)]
use crate::node::FieldValue;
use dashmap::DashMap;
use std::collections::HashMap;

pub(crate) struct ScalarIndex {
    indexes: DashMap<String, HashMap<FieldValue, Vec<u64>>>,
}

impl ScalarIndex {
    pub fn new() -> Self {
        Self {
            indexes: DashMap::new(),
        }
    }

    pub fn insert(&self, field: &str, value: &FieldValue, node_id: u64) {
        let mut entry = self.indexes.entry(field.to_string()).or_default();
        entry.entry(value.clone()).or_default().push(node_id);
    }

    pub fn remove(&self, field: &str, value: &FieldValue, node_id: u64) {
        if let Some(mut entry) = self.indexes.get_mut(field) {
            if let Some(values) = entry.get_mut(value) {
                values.retain(|&id| id != node_id);
            }
        }
    }

    pub fn lookup(&self, field: &str, value: &FieldValue) -> Vec<u64> {
        self.indexes
            .get(field)
            .and_then(|entry| entry.get(value).cloned())
            .unwrap_or_default()
    }

    pub fn remove_node(&self, node_id: u64) {
        for mut entry in self.indexes.iter_mut() {
            entry.retain(|_, ids| {
                ids.retain(|&id| id != node_id);
                !ids.is_empty()
            });
        }
    }

    pub fn clear_field(&self, field: &str) {
        self.indexes.remove(field);
    }

    pub fn field_names(&self) -> Vec<String> {
        self.indexes.iter().map(|e| e.key().clone()).collect()
    }
}
