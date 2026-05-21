//! Persistent textual index primitives for memory payloads.
//!
//! The text index is a derived materialization. Canonical memory records remain
//! the source of truth; this module owns only tokenization, key shape, compact
//! posting/stat values, and write-op construction.

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const TEXT_INDEX_SCHEMA_VERSION: u32 = 3;
pub(crate) const TOKENIZER_NAME: &str = "lowercase-ascii-alnum";
pub(crate) const TOKENIZER_VERSION: u32 = 1;
pub(crate) const KEY_FORMAT: &str = "namespace\\0token\\0key";
pub(crate) const BM25_K1: f32 = 1.2;
pub(crate) const BM25_B: f32 = 0.75;

const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
const TERM_STATS_TAG: &[u8] = b"term\0";
const DOC_STATS_TAG: &[u8] = b"doc\0";
const NAMESPACE_STATS_TAG: &[u8] = b"ns\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextTokenizerSpec {
    pub name: &'static str,
    pub version: u32,
}

impl Default for TextTokenizerSpec {
    fn default() -> Self {
        Self {
            name: TOKENIZER_NAME,
            version: TOKENIZER_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextIndexSpec {
    pub schema_version: u32,
    pub tokenizer: TextTokenizerSpec,
    pub key_format: &'static str,
}

impl Default for TextIndexSpec {
    fn default() -> Self {
        Self {
            schema_version: TEXT_INDEX_SCHEMA_VERSION,
            tokenizer: TextTokenizerSpec::default(),
            key_format: KEY_FORMAT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextPosting {
    pub node_id: u64,
    pub tf: u32,
    pub positions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextDocStats {
    pub node_id: u64,
    pub doc_len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextTermStats {
    pub df: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextNamespaceStats {
    pub doc_count: u64,
    pub total_doc_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextRecordTerms {
    pub token_counts: BTreeMap<String, u32>,
    pub token_positions: BTreeMap<String, Vec<u32>>,
    pub doc_len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextQueryPlan {
    pub terms: BTreeSet<String>,
    pub phrases: Vec<Vec<String>>,
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    tokenize_with_spec(&TextTokenizerSpec::default(), text)
}

pub(crate) fn tokenize_with_spec(spec: &TextTokenizerSpec, text: &str) -> Vec<String> {
    debug_assert_eq!(spec.name, TOKENIZER_NAME);
    debug_assert_eq!(spec.version, TOKENIZER_VERSION);

    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

pub(crate) fn token_counts(text: &str) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for token in tokenize(text) {
        counts
            .entry(token)
            .and_modify(|count: &mut u32| *count = count.saturating_add(1))
            .or_insert(1);
    }
    counts
}

pub(crate) fn record_terms(payload: &str) -> TextRecordTerms {
    let tokens = tokenize(payload);
    let doc_len = tokens.len().min(u32::MAX as usize) as u32;
    let mut token_counts = BTreeMap::new();
    let mut token_positions: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for (position, token) in tokens.into_iter().enumerate() {
        token_counts
            .entry(token.clone())
            .and_modify(|count: &mut u32| *count = count.saturating_add(1))
            .or_insert(1);
        token_positions
            .entry(token)
            .or_default()
            .push(position.min(u32::MAX as usize) as u32);
    }
    TextRecordTerms {
        token_counts,
        token_positions,
        doc_len,
    }
}

pub(crate) fn query_plan(query: &str) -> TextQueryPlan {
    let mut terms = BTreeSet::new();
    let mut phrases = Vec::new();
    let mut outside = String::new();
    let mut quoted = String::new();
    let mut in_quote = false;

    for ch in query.chars() {
        if ch == '"' {
            if in_quote {
                let phrase = tokenize(&quoted);
                if !phrase.is_empty() {
                    terms.extend(phrase.iter().cloned());
                    phrases.push(phrase);
                }
                quoted.clear();
                in_quote = false;
            } else {
                terms.extend(tokenize(&outside));
                outside.clear();
                in_quote = true;
            }
        } else if in_quote {
            quoted.push(ch);
        } else {
            outside.push(ch);
        }
    }

    if in_quote {
        outside.push_str(&quoted);
    }
    terms.extend(tokenize(&outside));

    TextQueryPlan { terms, phrases }
}

pub(crate) fn unique_tokens(text: &str) -> BTreeSet<String> {
    token_counts(text).into_keys().collect()
}

pub(crate) fn posting_key(namespace: &str, token: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(namespace.len() + token.len() + key.len() + 2);
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(token.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

pub(crate) fn posting_prefix(namespace: &str, token: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + token.len() + 2);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(token.as_bytes());
    prefix.push(0);
    prefix
}

pub(crate) fn posting_namespace_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + 1);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

pub(crate) fn posting_record_key(namespace: &str, token: &str, index_key: &[u8]) -> Option<String> {
    let prefix = posting_prefix(namespace, token);
    let key_bytes = index_key.strip_prefix(prefix.as_slice())?;
    String::from_utf8(key_bytes.to_vec()).ok()
}

pub(crate) fn is_internal_key(key: &[u8]) -> bool {
    key.starts_with(INTERNAL_PREFIX)
}

pub(crate) fn is_term_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(TERM_STATS_TAG))
}

pub(crate) fn is_doc_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(DOC_STATS_TAG))
}

pub(crate) fn is_namespace_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(NAMESPACE_STATS_TAG))
}

fn internal_key_prefix(tag: &[u8]) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(INTERNAL_PREFIX.len() + tag.len());
    prefix.extend_from_slice(INTERNAL_PREFIX);
    prefix.extend_from_slice(tag);
    prefix
}

pub(crate) fn term_stats_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = internal_key_prefix(TERM_STATS_TAG);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

pub(crate) fn doc_stats_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = internal_key_prefix(DOC_STATS_TAG);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

pub(crate) fn text_index_key_belongs_to_namespace(key: &[u8], namespace: &str) -> bool {
    if !is_internal_key(key) {
        return key.starts_with(&posting_namespace_prefix(namespace));
    }

    key.starts_with(&term_stats_prefix(namespace))
        || key.starts_with(&doc_stats_prefix(namespace))
        || key == namespace_stats_key(namespace)
}

pub(crate) fn term_stats_key(namespace: &str, token: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(
        INTERNAL_PREFIX.len() + TERM_STATS_TAG.len() + namespace.len() + token.len() + 1,
    );
    key.extend_from_slice(INTERNAL_PREFIX);
    key.extend_from_slice(TERM_STATS_TAG);
    key.extend_from_slice(namespace.as_bytes());
    key.push(0);
    key.extend_from_slice(token.as_bytes());
    key
}

pub(crate) fn doc_stats_key(namespace: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(
        INTERNAL_PREFIX.len() + DOC_STATS_TAG.len() + namespace.len() + key.len() + 1,
    );
    index_key.extend_from_slice(INTERNAL_PREFIX);
    index_key.extend_from_slice(DOC_STATS_TAG);
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

pub(crate) fn namespace_stats_key(namespace: &str) -> Vec<u8> {
    let mut key =
        Vec::with_capacity(INTERNAL_PREFIX.len() + NAMESPACE_STATS_TAG.len() + namespace.len());
    key.extend_from_slice(INTERNAL_PREFIX);
    key.extend_from_slice(NAMESPACE_STATS_TAG);
    key.extend_from_slice(namespace.as_bytes());
    key
}

pub(crate) fn posting_count(payload: &str) -> u64 {
    token_counts(payload).len() as u64
}

fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|err| VantaError::SerializationError(err.to_string()))
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    bincode::deserialize(bytes)
        .map_err(|err| VantaError::SerializationError(format!("{label} decode error: {err}")))
}

pub(crate) fn posting_value(node_id: u64, tf: u32, positions: &[u32]) -> Result<Vec<u8>> {
    serialize(&TextPosting {
        node_id,
        tf,
        positions: positions.to_vec(),
    })
}

pub(crate) fn decode_posting(bytes: &[u8]) -> Result<TextPosting> {
    deserialize(bytes, "text posting")
}

pub(crate) fn doc_stats_value(node_id: u64, doc_len: u32) -> Result<Vec<u8>> {
    serialize(&TextDocStats { node_id, doc_len })
}

pub(crate) fn decode_doc_stats(bytes: &[u8]) -> Result<TextDocStats> {
    deserialize(bytes, "text doc stats")
}

pub(crate) fn term_stats_value(df: u64) -> Result<Vec<u8>> {
    serialize(&TextTermStats { df })
}

pub(crate) fn decode_term_stats(bytes: &[u8]) -> Result<TextTermStats> {
    deserialize(bytes, "text term stats")
}

pub(crate) fn namespace_stats_value(doc_count: u64, total_doc_len: u64) -> Result<Vec<u8>> {
    serialize(&TextNamespaceStats {
        doc_count,
        total_doc_len,
    })
}

pub(crate) fn decode_namespace_stats(bytes: &[u8]) -> Result<TextNamespaceStats> {
    deserialize(bytes, "text namespace stats")
}

pub(crate) fn posting_put_ops(
    namespace: &str,
    key: &str,
    payload: &str,
    node_id: u64,
) -> Result<Vec<BackendWriteOp>> {
    let terms = record_terms(payload);
    let token_positions = terms.token_positions;
    terms
        .token_counts
        .into_iter()
        .map(|(token, tf)| {
            let positions = token_positions
                .get(&token)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            Ok(BackendWriteOp::Put {
                partition: BackendPartition::TextIndex,
                key: posting_key(namespace, &token, key),
                value: posting_value(node_id, tf, positions)?,
            })
        })
        .collect()
}

pub(crate) fn posting_delete_ops(namespace: &str, key: &str, payload: &str) -> Vec<BackendWriteOp> {
    unique_tokens(payload)
        .into_iter()
        .map(|token| BackendWriteOp::Delete {
            partition: BackendPartition::TextIndex,
            key: posting_key(namespace, &token, key),
        })
        .collect()
}

pub(crate) fn doc_stats_put_op(
    namespace: &str,
    key: &str,
    payload: &str,
    node_id: u64,
) -> Result<BackendWriteOp> {
    Ok(BackendWriteOp::Put {
        partition: BackendPartition::TextIndex,
        key: doc_stats_key(namespace, key),
        value: doc_stats_value(node_id, record_terms(payload).doc_len)?,
    })
}

pub(crate) fn doc_stats_delete_op(namespace: &str, key: &str) -> BackendWriteOp {
    BackendWriteOp::Delete {
        partition: BackendPartition::TextIndex,
        key: doc_stats_key(namespace, key),
    }
}

pub(crate) fn term_stats_put_op(namespace: &str, token: &str, df: u64) -> Result<BackendWriteOp> {
    Ok(BackendWriteOp::Put {
        partition: BackendPartition::TextIndex,
        key: term_stats_key(namespace, token),
        value: term_stats_value(df)?,
    })
}

pub(crate) fn term_stats_delete_op(namespace: &str, token: &str) -> BackendWriteOp {
    BackendWriteOp::Delete {
        partition: BackendPartition::TextIndex,
        key: term_stats_key(namespace, token),
    }
}

pub(crate) fn namespace_stats_put_op(
    namespace: &str,
    stats: &TextNamespaceStats,
) -> Result<BackendWriteOp> {
    Ok(BackendWriteOp::Put {
        partition: BackendPartition::TextIndex,
        key: namespace_stats_key(namespace),
        value: namespace_stats_value(stats.doc_count, stats.total_doc_len)?,
    })
}

pub(crate) fn namespace_stats_delete_op(namespace: &str) -> BackendWriteOp {
    BackendWriteOp::Delete {
        partition: BackendPartition::TextIndex,
        key: namespace_stats_key(namespace),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_is_lowercase_ascii_alnum() {
        let tokens = tokenize("Hello, VantaDB! Agent-42 memory.");
        assert_eq!(tokens, vec!["hello", "vantadb", "agent", "42", "memory"]);
    }

    #[test]
    fn posting_key_preserves_namespace_token_key_boundaries() {
        let key = posting_key("agent/main", "memory", "item-1");
        assert_eq!(key, b"agent/main\0memory\0item-1".to_vec());
    }

    #[test]
    fn token_counts_preserve_tf_and_unique_count() {
        let counts = token_counts("Alpha alpha beta");
        assert_eq!(counts.get("alpha"), Some(&2));
        assert_eq!(counts.get("beta"), Some(&1));
        assert_eq!(posting_count("Alpha alpha beta"), 2);
    }

    #[test]
    fn posting_value_stores_node_id_tf_and_positions() {
        let value = posting_value(42, 3, &[0, 2, 4]).expect("encode posting");
        let posting = decode_posting(&value).expect("decode posting");
        assert_eq!(posting.node_id, 42);
        assert_eq!(posting.tf, 3);
        assert_eq!(posting.positions, vec![0, 2, 4]);
    }

    #[test]
    fn query_plan_extracts_phrases_and_terms() {
        let plan = query_plan(r#"alpha "beta gamma" delta"#);
        assert_eq!(
            plan.terms.into_iter().collect::<Vec<_>>(),
            vec!["alpha", "beta", "delta", "gamma"]
        );
        assert_eq!(plan.phrases, vec![vec!["beta", "gamma"]]);
    }

    #[test]
    fn spec_declares_phrase_ready_text_index_v3() {
        let spec = TextIndexSpec::default();
        assert_eq!(spec.schema_version, 3);
        assert_eq!(spec.tokenizer.name, "lowercase-ascii-alnum");
        assert_eq!(spec.tokenizer.version, 1);
        assert_eq!(spec.key_format, "namespace\\0token\\0key");
    }
}
