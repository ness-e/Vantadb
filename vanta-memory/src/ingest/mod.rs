//! Wiki ingest (MEM-30 — TDAM `engines/wiki/ingest-v2` port, merge stage).
//!
//! Pipeline: scan local sources → chunk → extract candidate pages →
//! **serial** per-page merge → persist via the core [`vantadb::wiki::WikiStore`].
//!
//! Concurrency decision (D1: sync crate): merges run **serially per page** —
//! TDAM's `commitCandidates` also iterates pages serially; its `pLimit(5)`
//! exists only because multiple *sources* are extracted concurrently in JS.
//! The configurable global LLM limit ([`IngestConfig::global_llm_concurrency`],
//! default 5, clamped 1..=20) is an upper bound honored trivially by the
//! single-threaded worker; a semaphore would only pay off with concurrent
//! source extraction (deferred).
//!
//! LLM optional (P4): without a runner (or with [`LlmError::NotConfigured`])
//! extraction yields no candidates and commit writes new pages verbatim while
//! recording skips for pages needing real merging. It never blocks and never
//! loses data.

pub mod merge;
pub mod prompts;
pub mod worker;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Paths never overwritten by ingest candidates (TDAM ingest-v2/index.ts:69-75).
/// These structural files are managed by the wiki itself.
pub const STRUCTURAL_FILES: [&str; 5] = [
    "wiki/index.md",
    "wiki/schema.md",
    "wiki/purpose.md",
    "wiki/log.md",
    "wiki/overview.md",
];

/// Default global LLM concurrency for extract+merge calls (TDAM config.ts:105).
pub const DEFAULT_GLOBAL_LLM_CONCURRENCY: usize = 5;

/// Errors surfaced by the wiki ingest pipeline.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IngestError {
    /// Storage / state-machine error from the core wiki store.
    #[error(transparent)]
    Store(#[from] vantadb::error::VantaError),
    /// Input rejected at the ingest boundary (bad root path, empty slug...).
    #[error("invalid ingest request: {0}")]
    Invalid(String),
    /// The wiki does not exist yet.
    #[error("wiki not found: {namespace}:{slug}")]
    NotFound { namespace: String, slug: String },
}

/// Ingest pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestConfig {
    /// Global cap on concurrent LLM calls (extract + merge). Default 5,
    /// always clamped to 1..=20 on construction (TDAM config.ts:104-107).
    pub global_llm_concurrency: usize,
    /// Per-chunk target size in characters (core chunker default 12_000).
    pub chunk_target_chars: usize,
    /// Overlap characters between consecutive chunks (chunker default 400).
    pub chunk_overlap_chars: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            global_llm_concurrency: clamp_llm_concurrency(None),
            chunk_target_chars: vantadb::wiki::DEFAULT_TARGET_CHARS,
            chunk_overlap_chars: vantadb::wiki::DEFAULT_OVERLAP_CHARS,
        }
    }
}

impl IngestConfig {
    /// Build a config, clamping the requested LLM concurrency into 1..=20.
    pub fn new(global_llm_concurrency: Option<usize>) -> Self {
        Self {
            global_llm_concurrency: clamp_llm_concurrency(global_llm_concurrency),
            ..Self::default()
        }
    }
}

/// Clamp an optional raw concurrency value into 1..=20 (TDAM config.ts:104-107:
/// default 5 when unset/invalid). The configured limit bounds merge calls;
/// with the serial worker it is an enforced ceiling of one call at a time.
pub fn clamp_llm_concurrency(raw: Option<usize>) -> usize {
    match raw {
        None | Some(0) => DEFAULT_GLOBAL_LLM_CONCURRENCY,
        Some(n) => n.clamp(1, 20),
    }
}

// ── minimal YAML frontmatter ──

/// Parsed frontmatter of a wiki page (subset TDAM actually relies on:
/// string scalars + `sources` list + `locked` flag).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frontmatter {
    pub fields: Vec<(String, String)>,
    /// Values of the `sources` list (`sources: [a, b]` or `- item` lines).
    pub sources: Vec<String>,
    /// True only when `locked: true`.
    pub locked: bool,
    /// Raw scalar values for selected keys (`title`, `type`, `description`).
    pub title: Option<String>,
    pub page_type: Option<String>,
    pub description: Option<String>,
}

/// Split a page into `(frontmatter fields, body)`. Pages without a leading
/// `---` block return empty frontmatter and the whole text as body.
pub fn parse_frontmatter(content: &str) -> (Frontmatter, &str) {
    let mut fm = Frontmatter::default();
    if !content.starts_with("---") {
        return (fm, content);
    }
    let Some(rest) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
        return (fm, content);
    };
    let end = rest.find("\n---");
    let Some(end) = end else { return (fm, content) };
    let header = &rest[..end];
    // Body starts after the closing delimiter line.
    let body = &rest[end..];
    let body = body.strip_prefix("\n---").unwrap_or(body);
    let body = body.strip_prefix("\r\n").unwrap_or(body);
    let body = body.strip_prefix('\n').unwrap_or(body);

    let mut in_sources_list = false;
    for line in header.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if in_sources_list {
                fm.sources.push(item.trim().to_string());
            }
            continue;
        }
        in_sources_list = false;
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key == "sources" {
            // Inline list form `[a, b]` or empty list opening block form.
            let inner = value.trim_start_matches('[').trim_end_matches(']');
            if !value.contains('[') {
                in_sources_list = true;
                continue;
            }
            for s in inner.split(',') {
                let s = s.trim().trim_matches('"').trim_matches('\'');
                if !s.is_empty() {
                    fm.sources.push(s.to_string());
                }
            }
            continue;
        }
        if key == "locked" {
            fm.locked = value.eq_ignore_ascii_case("true");
            continue;
        }
        let unquoted = value.trim_matches('"').trim_matches('\'');
        match key {
            "title" => fm.title = Some(unquoted.to_string()),
            "type" => fm.page_type = Some(unquoted.to_string()),
            "description" => fm.description = Some(unquoted.to_string()),
            _ => {}
        }
        fm.fields.push((key.to_string(), unquoted.to_string()));
    }
    (fm, body)
}

/// Rebuild a page from frontmatter + body (TDAM buildPage parity).
pub fn build_page(fm: &Frontmatter, body: &str) -> String {
    let mut out = String::from("---\n");
    let mut wrote_keys: Vec<&str> = Vec::new();
    for (key, value) in &fm.fields {
        out.push_str(key);
        out.push_str(": ");
        out.push_str(value);
        out.push('\n');
        wrote_keys.push(key.as_str());
    }
    for key in ["title", "type", "description"] {
        let value = match key {
            "title" => fm.title.as_deref(),
            "type" => fm.page_type.as_deref(),
            _ => fm.description.as_deref(),
        };
        if let (Some(v), false) = (value, wrote_keys.contains(&key)) {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(v);
            out.push('\n');
        }
    }
    out.push_str("sources: [");
    out.push_str(
        &fm.sources
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n");
    if fm.locked {
        out.push_str("locked: true\n");
    }
    out.push_str("---\n");
    out.push_str(body);
    out
}

/// Force the `sources` frontmatter list to contain `source_name`
/// (TDAM ensureSources :368-375). Idempotent.
pub fn ensure_sources(content: &str, source_name: &str) -> String {
    let (mut fm, body) = parse_frontmatter(content);
    if fm.sources.iter().any(|s| s == source_name) {
        return content.to_string();
    }
    fm.sources.push(source_name.to_string());
    build_page(&fm, body)
}
