use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Typed relational field value
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldValue {
    /// A UTF-8 string value.
    String(String),
    /// A 64-bit signed integer value.
    Int(i64),
    /// A 64-bit floating point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// A UTC date-time value.
    DateTime(chrono::DateTime<chrono::Utc>),
    /// A list of UTF-8 string values.
    ListString(Vec<String>),
    /// A list of 64-bit signed integer values.
    ListInt(Vec<i64>),
    /// A list of 64-bit floating point values.
    ListFloat(Vec<f64>),
    /// A list of boolean values.
    ListBool(Vec<bool>),
    /// A list of UTC date-time values.
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    /// Absent / null value.
    Null,
}

impl Eq for FieldValue {}

impl Hash for FieldValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            FieldValue::String(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            FieldValue::Int(i) => {
                1u8.hash(state);
                i.hash(state);
            }
            FieldValue::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
            FieldValue::Bool(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            FieldValue::DateTime(dt) => {
                4u8.hash(state);
                dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
            }
            FieldValue::ListString(v) => {
                5u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListInt(v) => {
                6u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListFloat(v) => {
                7u8.hash(state);
                for f in v {
                    f.to_bits().hash(state);
                }
            }
            FieldValue::ListBool(v) => {
                8u8.hash(state);
                v.hash(state);
            }
            FieldValue::ListDateTime(v) => {
                9u8.hash(state);
                for dt in v {
                    dt.timestamp_nanos_opt().unwrap_or(0).hash(state);
                }
            }
            FieldValue::Null => {
                10u8.hash(state);
            }
        }
    }
}

impl FieldValue {
    /// Returns the inner `&str` if this is a `String` variant.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }
    /// Returns the inner `i64` if this is an `Int` variant.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FieldValue::Int(i) => Some(*i),
            _ => None,
        }
    }
    /// Returns the inner `bool` if this is a `Bool` variant.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns a list of string representations of the values.
    /// This is used for indexing and cardinality tracking.
    pub fn to_cardinality_keys(&self) -> Vec<String> {
        match self {
            FieldValue::String(s) => vec![s.clone()],
            FieldValue::Int(i) => vec![i.to_string()],
            FieldValue::Float(f) => vec![f.to_string()],
            FieldValue::Bool(b) => vec![b.to_string()],
            FieldValue::DateTime(dt) => vec![dt.to_rfc3339()],
            FieldValue::ListString(vec) => vec.clone(),
            FieldValue::ListInt(vec) => vec.iter().map(|i| i.to_string()).collect(),
            FieldValue::ListFloat(vec) => vec.iter().map(|f| f.to_string()).collect(),
            FieldValue::ListBool(vec) => vec.iter().map(|b| b.to_string()).collect(),
            FieldValue::ListDateTime(vec) => vec.iter().map(|dt| dt.to_rfc3339()).collect(),
            FieldValue::Null => vec!["null".to_string()],
        }
    }
}

/// Relational fields: ordered key-value map
pub type RelFields = BTreeMap<String, FieldValue>;

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_field_value_as_str() {
        assert_eq!(FieldValue::String("hello".into()).as_str(), Some("hello"));
        assert_eq!(FieldValue::Int(42).as_str(), None);
        assert_eq!(FieldValue::Null.as_str(), None);
    }

    #[test]
    fn test_field_value_as_int() {
        assert_eq!(FieldValue::Int(42).as_int(), Some(42));
        assert_eq!(FieldValue::String("x".into()).as_int(), None);
    }

    #[test]
    fn test_field_value_as_bool() {
        assert_eq!(FieldValue::Bool(true).as_bool(), Some(true));
        assert_eq!(FieldValue::Int(0).as_bool(), None);
    }

    #[test]
    fn test_field_value_cardinality_keys() {
        assert_eq!(
            FieldValue::String("test".into()).to_cardinality_keys(),
            vec!["test"]
        );
        assert_eq!(FieldValue::Int(42).to_cardinality_keys(), vec!["42"]);
        assert_eq!(FieldValue::Float(42.5).to_cardinality_keys(), vec!["42.5"]);
        assert_eq!(FieldValue::Bool(true).to_cardinality_keys(), vec!["true"]);
        assert_eq!(FieldValue::Null.to_cardinality_keys(), vec!["null"]);
        let dt = chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert!(FieldValue::DateTime(dt).to_cardinality_keys()[0].contains("2024"));
        assert_eq!(
            FieldValue::ListString(vec!["a".into()]).to_cardinality_keys(),
            vec!["a"]
        );
        assert_eq!(
            FieldValue::ListInt(vec![1, 2]).to_cardinality_keys(),
            vec!["1", "2"]
        );
        assert_eq!(
            FieldValue::ListBool(vec![true]).to_cardinality_keys(),
            vec!["true"]
        );
    }

    #[test]
    fn test_field_value_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(FieldValue::Int(42));
        assert!(set.contains(&FieldValue::Int(42)));
        assert!(!set.contains(&FieldValue::Int(43)));
        set.insert(FieldValue::String("hello".into()));
        assert!(set.contains(&FieldValue::String("hello".into())));
        set.insert(FieldValue::Bool(true));
        assert!(set.contains(&FieldValue::Bool(true)));
        set.insert(FieldValue::Null);
        assert!(set.contains(&FieldValue::Null));
    }
}
