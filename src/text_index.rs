//! Minimal textual index scaffolding.
//!
//! This module intentionally does not power public search yet. It defines the
//! low-level token and key contract needed before BM25/RRF can be implemented.

use crate::backend::{BackendPartition, BackendWriteOp};
use std::collections::BTreeSet;

pub(crate) const TEXT_INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextIndexSpec {
    pub schema_version: u32,
    pub tokenizer: &'static str,
    pub key_format: &'static str,
}

impl Default for TextIndexSpec {
    fn default() -> Self {
        Self {
            schema_version: TEXT_INDEX_SCHEMA_VERSION,
            tokenizer: "lowercase-ascii-alnum",
            key_format: "namespace\\0token\\0key",
        }
    }
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
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

pub(crate) fn unique_tokens(text: &str) -> BTreeSet<String> {
    tokenize(text).into_iter().collect()
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

pub(crate) fn posting_count(payload: &str) -> u64 {
    unique_tokens(payload).len() as u64
}

pub(crate) fn posting_put_ops(
    namespace: &str,
    key: &str,
    payload: &str,
    node_id: u64,
) -> Vec<BackendWriteOp> {
    unique_tokens(payload)
        .into_iter()
        .map(|token| BackendWriteOp::Put {
            partition: BackendPartition::TextIndex,
            key: posting_key(namespace, &token, key),
            value: node_id.to_le_bytes().to_vec(),
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
    fn unique_tokens_deduplicate_postings_per_record() {
        let tokens: Vec<_> = unique_tokens("Alpha alpha beta").into_iter().collect();
        assert_eq!(tokens, vec!["alpha", "beta"]);
        assert_eq!(posting_count("Alpha alpha beta"), 2);
    }

    #[test]
    fn spec_keeps_text_index_pre_bm25() {
        let spec = TextIndexSpec::default();
        assert_eq!(spec.schema_version, 1);
        assert_eq!(spec.tokenizer, "lowercase-ascii-alnum");
        assert_eq!(spec.key_format, "namespace\\0token\\0key");
    }
}
