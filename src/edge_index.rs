use crate::error::Result;
use crate::storage::StorageEngine;
use dashmap::DashSet;
use std::sync::Arc;

pub(crate) struct EdgeIndex {
    edges: DashSet<(u64, u64)>,
}

impl EdgeIndex {
    pub fn new() -> Self {
        Self {
            edges: DashSet::new(),
        }
    }

    pub fn insert(&self, from: u64, to: u64) {
        self.edges.insert((from, to));
    }

    pub fn remove_from(&self, from: u64) {
        self.edges.retain(|(f, _)| *f != from);
    }

    pub fn remove_edge(&self, from: u64, to: u64) {
        self.edges.remove(&(from, to));
    }

    pub fn has_edge(&self, from: u64, to: u64) -> bool {
        self.edges.contains(&(from, to))
    }

    pub fn outgoing(&self, from: u64) -> Vec<u64> {
        self.edges
            .iter()
            .filter(|e| e.0 == from)
            .map(|e| e.1)
            .collect()
    }

    pub fn incoming(&self, to: u64) -> Vec<u64> {
        self.edges
            .iter()
            .filter(|e| e.1 == to)
            .map(|e| e.0)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn verify_referential_integrity(&self) -> Result<()> {
        Ok(())
    }
}
