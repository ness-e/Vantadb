//! Persistent textual index primitives for memory payloads.
//!
//! The text index is a derived materialization. Canonical memory records remain
//! the source of truth; this module owns only tokenization, key shape, compact
//! posting/stat values, and write-op construction.

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
#[cfg(feature = "advanced-tokenizer")]
use crate::tokenizer::{tokenize_advanced, AdvancedTokenizerConfig};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Schema version with advanced tokenizer support.
#[cfg(feature = "advanced-tokenizer")]
pub(crate) const TEXT_INDEX_SCHEMA_VERSION: u32 = 4;
/// Schema version with basic tokenizer support.
#[cfg(not(feature = "advanced-tokenizer"))]
pub(crate) const TEXT_INDEX_SCHEMA_VERSION: u32 = 3;
/// Name of the built-in tokenizer.
pub(crate) const TOKENIZER_NAME: &str = "lowercase-ascii-alnum";
/// Version of the built-in tokenizer.
pub(crate) const TOKENIZER_VERSION: u32 = 1;
/// Name of the advanced (tantivy-multilingual) tokenizer.
#[cfg(feature = "advanced-tokenizer")]
pub(crate) const ADVANCED_TOKENIZER_NAME: &str = "tantivy-multilingual";
/// Version of the advanced tokenizer.
#[cfg(feature = "advanced-tokenizer")]
pub(crate) const ADVANCED_TOKENIZER_VERSION: u32 = 1;
/// Storage key format description.
pub(crate) const KEY_FORMAT: &str = "namespace\\0token\\0key";
/// BM25 k1 parameter.
pub(crate) const BM25_K1: f32 = 1.2;
/// BM25 b parameter.
pub(crate) const BM25_B: f32 = 0.75;

const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
const TERM_STATS_TAG: &[u8] = b"term\0";
const DOC_STATS_TAG: &[u8] = b"doc\0";
const NAMESPACE_STATS_TAG: &[u8] = b"ns\0";

/// Specification for a text tokenizer implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextTokenizerSpec {
    /// Tokenizer name identifier.
    pub name: &'static str,
    /// Tokenizer version.
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

#[cfg(feature = "advanced-tokenizer")]
impl TextTokenizerSpec {
    pub(crate) fn advanced() -> Self {
        Self {
            name: ADVANCED_TOKENIZER_NAME,
            version: ADVANCED_TOKENIZER_VERSION,
        }
    }
}

/// Specification for the text index schema and tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextIndexSpec {
    /// Schema version of the text index.
    pub schema_version: u32,
    /// Tokenizer specification.
    pub tokenizer: TextTokenizerSpec,
    /// Key format string for posting entries.
    pub key_format: &'static str,
}

impl Default for TextIndexSpec {
    fn default() -> Self {
        #[cfg(feature = "advanced-tokenizer")]
        let tokenizer = TextTokenizerSpec::advanced();
        #[cfg(not(feature = "advanced-tokenizer"))]
        let tokenizer = TextTokenizerSpec::default();

        Self {
            schema_version: TEXT_INDEX_SCHEMA_VERSION,
            tokenizer,
            key_format: KEY_FORMAT,
        }
    }
}

/// A posting entry linking a node ID to a token with term frequency and positions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextPosting {
    /// Node ID referenced by this posting.
    pub node_id: u128,
    /// Term frequency within the document.
    pub tf: u32,
    /// Positions of the token in the document.
    pub positions: Vec<u32>,
}

/// Document-level statistics for the text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextDocStats {
    /// Node ID.
    pub node_id: u128,
    /// Total document length in tokens.
    pub doc_len: u32,
}

/// Term-level statistics for the text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextTermStats {
    /// Document frequency (number of docs containing this term).
    pub df: u64,
}

/// Namespace-level statistics for the text index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextNamespaceStats {
    /// Total document count in the namespace.
    pub doc_count: u64,
    /// Sum of all document lengths in the namespace.
    pub total_doc_len: u64,
}

/// Token counts and positions extracted from a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextRecordTerms {
    /// Token to count mapping.
    pub token_counts: BTreeMap<String, u32>,
    /// Token to positions mapping.
    pub token_positions: BTreeMap<String, Vec<u32>>,
    /// Total document length in tokens.
    pub doc_len: u32,
}

/// A query plan for text search containing terms and phrase groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextQueryPlan {
    /// Unique terms to search.
    pub terms: BTreeSet<String>,
    /// Phrase groups (ordered sequences of terms).
    pub phrases: Vec<Vec<String>>,
}

/// Tokenize text using the default tokenizer specification.
#[allow(dead_code)]
pub(crate) fn tokenize(text: &str) -> Vec<String> {
    tokenize_with_spec(&TextTokenizerSpec::default(), text)
}

/// Tokenize text using a given tokenizer specification.
#[allow(dead_code)]
pub(crate) fn tokenize_with_spec(spec: &TextTokenizerSpec, text: &str) -> Vec<String> {
    #[cfg(feature = "advanced-tokenizer")]
    {
        if spec.name == ADVANCED_TOKENIZER_NAME {
            debug_assert_eq!(spec.version, ADVANCED_TOKENIZER_VERSION);
            return crate::tokenizer::tokenize_advanced_default(text);
        }
    }

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

/// Tokenize text and return counts per unique token.
pub(crate) fn token_counts(text: &str) -> BTreeMap<String, u32> {
    token_counts_with_config(text, None)
}

#[cfg(feature = "advanced-tokenizer")]
/// Tokenize text and return counts, with optional advanced tokenizer config.
pub(crate) fn token_counts_with_config(
    text: &str,
    config: Option<&AdvancedTokenizerConfig>,
) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    let tokens = if let Some(cfg) = config {
        tokenize_advanced(text, cfg)
    } else {
        crate::tokenizer::tokenize_advanced_default(text)
    };

    for token in tokens {
        counts
            .entry(token)
            .and_modify(|count: &mut u32| *count = count.saturating_add(1))
            .or_insert(1);
    }
    counts
}

#[cfg(not(feature = "advanced-tokenizer"))]
/// Tokenize text and return counts (simple path, no advanced tokenizer).
pub(crate) fn token_counts_with_config(text: &str, _config: Option<&()>) -> BTreeMap<String, u32> {
    // Simple token counting without advanced tokenizer support
    let mut counts = BTreeMap::new();
    let tokens = tokenize(text);

    for token in tokens {
        counts
            .entry(token)
            .and_modify(|count: &mut u32| *count = count.saturating_add(1))
            .or_insert(1);
    }
    counts
}

/// Extract token counts, positions and doc length from a payload string.
pub(crate) fn record_terms(payload: &str) -> TextRecordTerms {
    record_terms_with_config(payload, None)
}

#[cfg(feature = "advanced-tokenizer")]
/// Extract record terms with optional advanced tokenizer config.
pub(crate) fn record_terms_with_config(
    payload: &str,
    config: Option<&AdvancedTokenizerConfig>,
) -> TextRecordTerms {
    let tokens = if let Some(cfg) = config {
        tokenize_advanced(payload, cfg)
    } else {
        crate::tokenizer::tokenize_advanced_default(payload)
    };
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

/// Extract record terms (simple path, no advanced tokenizer).
#[cfg(not(feature = "advanced-tokenizer"))]
pub(crate) fn record_terms_with_config(payload: &str, _config: Option<&()>) -> TextRecordTerms {
    // Simple record terms without advanced tokenizer support
    let tokens = tokenize(payload);
    let mut token_counts = BTreeMap::new();
    let mut token_positions = BTreeMap::new();

    for (position, token) in (0_u32..).zip(tokens.iter()) {
        *token_counts.entry(token.clone()).or_insert(0) += 1;
        token_positions
            .entry(token.clone())
            .or_insert_with(Vec::new)
            .push(position);
    }

    TextRecordTerms {
        token_counts,
        token_positions,
        doc_len: tokens.len() as u32,
    }
}

/// Build a query plan (terms and phrases) from a search query string.
pub(crate) fn query_plan(query: &str) -> TextQueryPlan {
    query_plan_with_config(query, None)
}

/// Build a query plan with an optional advanced tokenizer config.
#[cfg(feature = "advanced-tokenizer")]
pub(crate) fn query_plan_with_config(
    query: &str,
    config: Option<&AdvancedTokenizerConfig>,
) -> TextQueryPlan {
    let mut terms = BTreeSet::new();
    let mut phrases = Vec::new();
    let mut outside = String::new();
    let mut quoted = String::new();
    let mut in_quote = false;

    for ch in query.chars() {
        if ch == '"' {
            if in_quote {
                let phrase = if let Some(cfg) = config {
                    tokenize_advanced(&quoted, cfg)
                } else {
                    crate::tokenizer::tokenize_advanced_default(&quoted)
                };
                if !phrase.is_empty() {
                    terms.extend(phrase.iter().cloned());
                    phrases.push(phrase);
                }
                quoted.clear();
                in_quote = false;
            } else {
                let outside_tokens = if let Some(cfg) = config {
                    tokenize_advanced(&outside, cfg)
                } else {
                    crate::tokenizer::tokenize_advanced_default(&outside)
                };
                terms.extend(outside_tokens);
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
    let outside_tokens = if let Some(cfg) = config {
        tokenize_advanced(&outside, cfg)
    } else {
        crate::tokenizer::tokenize_advanced_default(&outside)
    };
    terms.extend(outside_tokens);

    TextQueryPlan { terms, phrases }
}

/// Build a query plan (simple path, no advanced tokenizer).
#[cfg(not(feature = "advanced-tokenizer"))]
pub(crate) fn query_plan_with_config(query: &str, _config: Option<&()>) -> TextQueryPlan {
    // Simple query plan without advanced tokenizer support
    let mut terms = BTreeSet::new();
    let mut phrases = Vec::new();
    let mut outside = String::new();
    let mut quoted = String::new();
    let mut in_quote = false;

    for ch in query.chars() {
        if ch == '"' {
            if in_quote {
                let phrase: Vec<String> = quoted
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect();
                if !phrase.is_empty() {
                    terms.extend(phrase.iter().cloned());
                    phrases.push(phrase);
                }
                quoted.clear();
                in_quote = false;
            } else {
                let outside_tokens: Vec<String> = outside
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect();
                terms.extend(outside_tokens);
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
    let outside_tokens: Vec<String> = outside
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    terms.extend(outside_tokens);

    TextQueryPlan { terms, phrases }
}

/// Return the set of unique tokens in text.
pub(crate) fn unique_tokens(text: &str) -> BTreeSet<String> {
    token_counts(text).into_keys().collect()
}

/// Build a posting index key from namespace, token, and record key.
pub(crate) fn posting_key(namespace: &str, token: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(namespace.len() + token.len() + key.len() + 2);
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(token.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

/// Build a posting prefix for scanning all entries of a namespace+token.
pub(crate) fn posting_prefix(namespace: &str, token: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + token.len() + 2);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(token.as_bytes());
    prefix.push(0);
    prefix
}

/// Build a posting prefix for scanning all tokens in a namespace.
pub(crate) fn posting_namespace_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + 1);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

/// Extract the record key from a posting key for a given namespace and token.
pub(crate) fn posting_record_key(namespace: &str, token: &str, index_key: &[u8]) -> Option<String> {
    let prefix = posting_prefix(namespace, token);
    let key_bytes = index_key.strip_prefix(prefix.as_slice())?;
    String::from_utf8(key_bytes.to_vec()).ok()
}

/// Check whether a key belongs to an internal (stats) prefix.
pub(crate) fn is_internal_key(key: &[u8]) -> bool {
    key.starts_with(INTERNAL_PREFIX)
}

/// Check whether a key is a term stats key.
pub(crate) fn is_term_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(TERM_STATS_TAG))
}

/// Check whether a key is a doc stats key.
pub(crate) fn is_doc_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(DOC_STATS_TAG))
}

/// Check whether a key is a namespace stats key.
pub(crate) fn is_namespace_stats_key(key: &[u8]) -> bool {
    key.starts_with(&internal_key_prefix(NAMESPACE_STATS_TAG))
}

fn internal_key_prefix(tag: &[u8]) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(INTERNAL_PREFIX.len() + tag.len());
    prefix.extend_from_slice(INTERNAL_PREFIX);
    prefix.extend_from_slice(tag);
    prefix
}

/// Build a prefix for scanning term stats entries in a namespace.
pub(crate) fn term_stats_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = internal_key_prefix(TERM_STATS_TAG);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

/// Build a prefix for scanning doc stats entries in a namespace.
pub(crate) fn doc_stats_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = internal_key_prefix(DOC_STATS_TAG);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

/// Check whether a text index key belongs to the given namespace.
pub(crate) fn text_index_key_belongs_to_namespace(key: &[u8], namespace: &str) -> bool {
    if !is_internal_key(key) {
        return key.starts_with(&posting_namespace_prefix(namespace));
    }

    key.starts_with(&term_stats_prefix(namespace))
        || key.starts_with(&doc_stats_prefix(namespace))
        || key == namespace_stats_key(namespace)
}

/// Build a term stats key for a given namespace and token.
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

/// Build a doc stats key for a given namespace and record.
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

/// Build a namespace stats key for a given namespace.
pub(crate) fn namespace_stats_key(namespace: &str) -> Vec<u8> {
    let mut key =
        Vec::with_capacity(INTERNAL_PREFIX.len() + NAMESPACE_STATS_TAG.len() + namespace.len());
    key.extend_from_slice(INTERNAL_PREFIX);
    key.extend_from_slice(NAMESPACE_STATS_TAG);
    key.extend_from_slice(namespace.as_bytes());
    key
}

/// Count the number of tokens in a payload.
pub(crate) fn posting_count(payload: &str) -> u64 {
    token_counts(payload).len() as u64
}

fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    postcard::to_allocvec(value).map_err(VantaError::serialization)
}

fn deserialize<T: for<'de> Deserialize<'de>>(bytes: &[u8], label: &str) -> Result<T> {
    let val: T = postcard::from_bytes(bytes).map_err(|err| {
        VantaError::SerializationError(Box::new(crate::error::SerdeMsgError::new(
            format!("{label} decode error: {err}"),
            err,
        )))
    })?;
    Ok(val)
}

/// Encode a posting entry into bytes.
pub(crate) fn posting_value(node_id: u128, tf: u32, positions: &[u32]) -> Result<Vec<u8>> {
    serialize(&TextPosting {
        node_id,
        tf,
        positions: positions.to_vec(),
    })
}

/// Decode posting bytes into a `TextPosting` struct.
pub(crate) fn decode_posting(bytes: &[u8]) -> Result<TextPosting> {
    deserialize(bytes, "text posting")
}

/// Encode doc stats into bytes.
pub(crate) fn doc_stats_value(node_id: u128, doc_len: u32) -> Result<Vec<u8>> {
    serialize(&TextDocStats { node_id, doc_len })
}

/// Decode doc stats bytes into a `TextDocStats` struct.
pub(crate) fn decode_doc_stats(bytes: &[u8]) -> Result<TextDocStats> {
    deserialize(bytes, "text doc stats")
}

/// Encode term stats into bytes.
pub(crate) fn term_stats_value(df: u64) -> Result<Vec<u8>> {
    serialize(&TextTermStats { df })
}

/// Decode term stats bytes into a `TextTermStats` struct.
pub(crate) fn decode_term_stats(bytes: &[u8]) -> Result<TextTermStats> {
    deserialize(bytes, "text term stats")
}

/// Encode namespace stats into bytes.
pub(crate) fn namespace_stats_value(doc_count: u64, total_doc_len: u64) -> Result<Vec<u8>> {
    serialize(&TextNamespaceStats {
        doc_count,
        total_doc_len,
    })
}

/// Decode namespace stats bytes into a `TextNamespaceStats` struct.
pub(crate) fn decode_namespace_stats(bytes: &[u8]) -> Result<TextNamespaceStats> {
    deserialize(bytes, "text namespace stats")
}

/// Build write operations to upsert a posting entry.
pub(crate) fn posting_put_ops(
    namespace: &str,
    key: &str,
    payload: &str,
    node_id: u128,
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

/// Build write operations to delete postings for a given record.
pub(crate) fn posting_delete_ops(namespace: &str, key: &str, payload: &str) -> Vec<BackendWriteOp> {
    unique_tokens(payload)
        .into_iter()
        .map(|token| BackendWriteOp::Delete {
            partition: BackendPartition::TextIndex,
            key: posting_key(namespace, &token, key),
        })
        .collect()
}

/// Build a write operation to upsert doc stats.
pub(crate) fn doc_stats_put_op(
    namespace: &str,
    key: &str,
    payload: &str,
    node_id: u128,
) -> Result<BackendWriteOp> {
    Ok(BackendWriteOp::Put {
        partition: BackendPartition::TextIndex,
        key: doc_stats_key(namespace, key),
        value: doc_stats_value(node_id, record_terms(payload).doc_len)?,
    })
}

/// Build a write operation to delete doc stats.
pub(crate) fn doc_stats_delete_op(namespace: &str, key: &str) -> BackendWriteOp {
    BackendWriteOp::Delete {
        partition: BackendPartition::TextIndex,
        key: doc_stats_key(namespace, key),
    }
}

/// Build a write operation to upsert term stats.
pub(crate) fn term_stats_put_op(namespace: &str, token: &str, df: u64) -> Result<BackendWriteOp> {
    Ok(BackendWriteOp::Put {
        partition: BackendPartition::TextIndex,
        key: term_stats_key(namespace, token),
        value: term_stats_value(df)?,
    })
}

/// Build a write operation to delete term stats.
pub(crate) fn term_stats_delete_op(namespace: &str, token: &str) -> BackendWriteOp {
    BackendWriteOp::Delete {
        partition: BackendPartition::TextIndex,
        key: term_stats_key(namespace, token),
    }
}

/// Build a write operation to upsert namespace stats.
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

/// Build a write operation to delete namespace stats.
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

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_integration_record_terms() {
        let payload = "The quick brown fox jumps over the lazy dog";
        let terms = record_terms(payload);

        // Should tokenize with advanced features (stopwords removal, stemming)
        assert!(!terms.token_counts.is_empty());
        // With stopwords removal, should have fewer tokens than the full text
        assert!(terms.token_counts.len() < 9); // "The quick brown fox jumps over the lazy dog" has 9 words
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_integration_query_plan() {
        let query = "The jumping fox runs quickly";
        let plan = query_plan(query);

        // Should tokenize query with advanced features
        assert!(!plan.terms.is_empty());
        // Stopwords like "the" should be removed
        assert!(!plan.terms.iter().any(|t| t == "the"));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_integration_token_counts() {
        let text = "The quick brown fox jumps over the lazy dog";
        let counts = token_counts(text);

        // Should have fewer tokens due to stopwords removal
        assert!(!counts.is_empty());
        assert!(counts.len() < 9);
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_advanced_tokenizer_integration_unicode() {
        let text = "Café naïve résumé";
        let counts = token_counts(text);

        // Should handle Unicode with folding
        assert!(!counts.is_empty());
        // "café" should be folded to "cafe" or similar
        assert!(counts.keys().any(|k| k.contains("cafe")));
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_token_counts_with_config_none() {
        let text = "The jumping fox runs quickly";

        // Test with None config (should use default)
        let counts = token_counts_with_config(text, None);

        assert!(!counts.is_empty());
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_record_terms_with_config_none() {
        let payload = "The quick brown fox jumps over the lazy dog";

        // Test with None config (should use default)
        let terms = record_terms_with_config(payload, None);

        assert!(!terms.token_counts.is_empty());
    }

    #[cfg(feature = "advanced-tokenizer")]
    #[test]
    fn test_query_plan_with_config_none() {
        let query = "The jumping fox runs quickly";

        // Test with None config (should use default)
        let plan = query_plan_with_config(query, None);

        assert!(!plan.terms.is_empty());
    }

    #[test]
    fn spec_declares_phrase_ready_text_index_v3() {
        let spec = TextIndexSpec::default();
        #[cfg(feature = "advanced-tokenizer")]
        {
            assert_eq!(spec.schema_version, 4);
            assert_eq!(spec.tokenizer.name, "tantivy-multilingual");
            assert_eq!(spec.tokenizer.version, 1);
        }
        #[cfg(not(feature = "advanced-tokenizer"))]
        {
            assert_eq!(spec.schema_version, 3);
            assert_eq!(spec.tokenizer.name, "lowercase-ascii-alnum");
            assert_eq!(spec.tokenizer.version, 1);
        }
        assert_eq!(spec.key_format, "namespace\\0token\\0key");
    }
}
