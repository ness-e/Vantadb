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
}

impl Default for LabelIntern {
    fn default() -> Self {
        Self::new()
    }
}
