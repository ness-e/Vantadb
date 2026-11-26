use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bidirectional map: String ↔ u32 for edge labels.
/// Cardinalidad típica: decenas/cientos, no miles. HashMap alcanza.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LabelIntern {
    map: HashMap<String, u32>,
    strings: Vec<String>,
}

impl LabelIntern {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    /// Get or create a label ID.
    pub fn intern(&mut self, label: &str) -> u32 {
        if let Some(&id) = self.map.get(label) {
            return id;
        }
        let id = self.strings.len() as u32;
        self.map.insert(label.to_string(), id);
        self.strings.push(label.to_string());
        id
    }

    /// Resolve an ID back to a label string.
    pub fn resolve(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    /// Look up a label string without creating a new ID.
    #[allow(dead_code)]
    pub fn lookup(&self, label: &str) -> Option<u32> {
        self.map.get(label).copied()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Serialize the intern state to bytes (JSON).
    pub fn to_bytes(&self) -> std::result::Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Hydrate the intern state from bytes (JSON).
    pub fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl Default for LabelIntern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_intern_basic() {
        let mut intern = LabelIntern::new();
        assert!(intern.is_empty());
        assert_eq!(intern.len(), 0);

        let id0 = intern.intern("related_to");
        let id1 = intern.intern("supersedes");
        let id0_again = intern.intern("related_to");

        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id0_again, 0);
        assert_eq!(intern.len(), 2);
        assert!(!intern.is_empty());

        assert_eq!(intern.resolve(0), Some("related_to"));
        assert_eq!(intern.resolve(1), Some("supersedes"));
        assert_eq!(intern.resolve(2), None);

        assert_eq!(intern.lookup("related_to"), Some(0));
        assert_eq!(intern.lookup("supersedes"), Some(1));
        assert_eq!(intern.lookup("nonexistent"), None);
    }

    #[test]
    fn test_label_intern_serialization_roundtrip() {
        let mut intern = LabelIntern::new();
        intern.intern("parent");
        intern.intern("child");
        intern.intern("sibling");

        let bytes = intern.to_bytes().expect("serialize");
        let restored = LabelIntern::from_bytes(&bytes).expect("deserialize");

        assert_eq!(restored.len(), 3);
        assert_eq!(restored.resolve(0), Some("parent"));
        assert_eq!(restored.resolve(1), Some("child"));
        assert_eq!(restored.resolve(2), Some("sibling"));
        assert_eq!(restored.lookup("child"), Some(1));
    }
}
