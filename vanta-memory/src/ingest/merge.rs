//! Candidate parsing + serial per-page merge (MEM-30 — TDAM `file-protocol.ts`
//! and `merge.ts` port).
//!
//! [`commit`] is the heart of the contract: candidates aggregated by relPath,
//! merged **serially per page** under the configured global LLM limit (the
//! sync worker runs one call at a time — see module docs in `mod.rs`), a
//! failure on page N never blocks pages N+1.., structural files are never
//! overwritten, and every written page carries its `sources` frontmatter.

use crate::core::abstractions::{LlmError, LlmRunner};
use crate::ingest::{
    build_page, clamp_llm_concurrency, ensure_sources, parse_frontmatter, prompts, IngestConfig,
    STRUCTURAL_FILES,
};

/// One candidate page produced by extraction (TDAM ParsedFile).
#[derive(Debug, Clone, PartialEq)]
pub struct CandidatePage {
    /// Normalized wiki-relative path (`wiki/...`, forward slashes).
    pub rel_path: String,
    /// Full page content including frontmatter.
    pub content: String,
}

/// Outcome of a single merge decision (TDAM MergeDecision).
#[derive(Debug, Clone, PartialEq)]
pub enum MergeDecision {
    /// Replace the page with this content.
    Write(String),
    /// Do not touch the page; reason documents why.
    Skip(String),
}

/// Report of a commit pass over all candidates (TDAM CommitResult parity).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommitReport {
    /// relPaths whose final content was written.
    pub written: Vec<String>,
    /// Per-entry failures/skips that did NOT stop the run.
    pub merge_errors: Vec<MergeEntryError>,
}

/// One non-fatal commit outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeEntryError {
    pub rel_path: String,
    pub source: String,
    pub error: String,
}

/// Normalize an LLM-provided wiki path or return `None` when unsafe
/// (TDAM normalizeWikiPath): forward slashes, no absolute/drive paths,
/// no `.` / `..` segments, must live under `wiki/`.
pub fn normalize_wiki_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim().replace('\\', "/");
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('/') {
        return None;
    }
    // Windows drive letter (`C:/...`) guard.
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return None;
    }
    let cleaned = trimmed.trim_start_matches("./");
    let segments: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return None;
    }
    for seg in &segments {
        if *seg == ".." || *seg == "." {
            return None;
        }
    }
    let normalized = segments.join("/");
    if normalized == "wiki" || !normalized.starts_with("wiki/") {
        return None;
    }
    Some(normalized)
}

/// Parse `<<<FILE path="...">>> ... <<<END>>>` blocks out of LLM output
/// (TDAM parseFileBlocks). Malformed blocks are skipped silently —
/// warnings are not part of the Rust contract surface.
pub fn parse_file_blocks(text: &str) -> Vec<CandidatePage> {
    let mut files = Vec::new();
    let mut rest = text;
    while let Some(open) = find_file_open(rest) {
        let (path_raw, body_start) = open;
        let after = &rest[body_start..];
        let Some(close_off) = after.find("<<<END>>>") else {
            break;
        };
        let body = after[..close_off].trim();
        if let Some(rel) = normalize_wiki_path(&path_raw) {
            if !body.is_empty() {
                files.push(CandidatePage {
                    rel_path: rel,
                    content: body.to_string(),
                });
            }
        }
        rest = &after[close_off + "<<<END>>>".len()..];
    }
    files
}

/// Find the next `<<<FILE path="..." >>>` marker; returns `(path, offset_after)`.
fn find_file_open(text: &str) -> Option<(String, usize)> {
    let start = text.find("<<<FILE")?;
    let after_marker = &text[start + "<<<FILE".len()..];
    // Require `path="..."` before the closing `>>>` of this marker.
    let end = after_marker.find(">>>")?;
    let head = &after_marker[..end];
    let path_idx = head.find("path=")?;
    let quoted = &head[path_idx + "path=".len()..];
    let quoted = quoted.trim_start();
    let quoted = quoted.strip_prefix('"')?;
    let close_quote = quoted.find('"')?;
    Some((
        quoted[..close_quote].to_string(),
        start + "<<<FILE".len() + end + 3,
    ))
}

/// Merge one candidate entry into the page currently stored at `rel_path`
/// (TDAM mergePage). Deterministic fallback without an LLM runner (P4):
/// new pages are written verbatim (no LLM needed), existing pages needing a
/// real merge are skipped with a documented reason.
///
/// Returns the decision for the entry. `existing` is `None` when the page
/// does not exist yet.
pub fn merge_page<R: LlmRunner>(
    existing_content: Option<&str>,
    candidate_content: &str,
    runner: Option<&R>,
    full_rewrite_max_chars: usize,
) -> MergeDecision {
    // New page: write verbatim — no LLM needed (TDAM mergePage short-circuit).
    let Some(existing) = existing_content else {
        return MergeDecision::Write(candidate_content.to_string());
    };
    {
        let (fm, _) = parse_frontmatter(existing);
        if fm.locked {
            return MergeDecision::Skip("page is locked".into());
        }
        let (_, cand_body) = parse_frontmatter(candidate_content);
        let (_, old_body) = parse_frontmatter(existing);
        // Redundant candidate (already contained in old body): union sources only.
        let cand_norm = collapse_ws(cand_body);
        let old_norm = collapse_ws(old_body);
        if !cand_norm.is_empty() && old_norm.contains(&cand_norm) {
            let mut union_fm = fm;
            let (cand_fm, _) = parse_frontmatter(candidate_content);
            for s in cand_fm.sources {
                if !union_fm.sources.contains(&s) {
                    union_fm.sources.push(s);
                }
            }
            return MergeDecision::Write(build_page(&union_fm, old_body));
        }
    }

    match runner {
        None => MergeDecision::Skip("LLM unavailable for merge (LLM-free fallback)".into()),
        Some(runner) => {
            let (system, user) =
                prompts::merge_prompts(existing_content, candidate_content, full_rewrite_max_chars);
            let mut params = crate::core::abstractions::LlmRunParams::new(user, "ingest-merge");
            params.system_prompt = Some(system);
            match runner.run(&params) {
                Ok(output) if output.trim().is_empty() => {
                    MergeDecision::Skip("LLM returned empty merge".into())
                }
                Ok(output) => MergeDecision::Write(output),
                Err(LlmError::NotConfigured) => {
                    MergeDecision::Skip("LLM not configured for merge (LLM-free fallback)".into())
                }
                Err(e) => MergeDecision::Skip(format!("merge failed: {e}")),
            }
        }
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Is this candidate targeting a protected structural file?
pub fn is_structural(rel_path: &str) -> bool {
    STRUCTURAL_FILES
        .iter()
        .any(|f| f.eq_ignore_ascii_case(rel_path))
}

/// Aggregate per-source candidates by relPath preserving discovery order
/// (contract test a). Input order defines page merge order.
pub fn aggregate_by_rel_path(
    sources: Vec<(String, Vec<CandidatePage>)>,
) -> Vec<(String, Vec<(String, String)>)> {
    let mut order: Vec<String> = Vec::new();
    let mut by_page: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (source_name, pages) in sources {
        for page in pages {
            let slot = match by_page.iter_mut().find(|(p, _)| *p == page.rel_path) {
                Some(slot) => slot,
                None => {
                    order.push(page.rel_path.clone());
                    by_page.push((page.rel_path.clone(), Vec::new()));
                    // last_mut() is Some right after push (infallible).
                    #[allow(clippy::expect_used)]
                    {
                        by_page.last_mut().expect("just pushed")
                    }
                }
            };
            slot.1.push((source_name.clone(), page.content.clone()));
        }
    }
    // Stable ordering by first appearance.
    by_page.sort_by_key(|(p, _)| order.iter().position(|x| x == p).unwrap_or(usize::MAX));
    by_page
}

/// Serial commit pass (TDAM commitCandidates :211-283):
/// - pages merged in aggregation order, **one at a time**;
/// - the global LLM limit from `config` bounds concurrency (serial worker ⇒
///   ceiling of 1 in-flight call; limit kept configurable for future pools);
/// - a failure/skip on one entry records it and continues to the next;
/// - structural files are skipped before any merge work happens;
/// - every written page passes through `ensure_sources`.
///
/// `read_page(path)` fetches current stored content (None = new page) and
/// `write_page(path, content)` persists it — injected so this stays
/// storage-agnostic (worker binds them to `WikiStore`).
pub fn commit<L, R, W>(
    by_page: Vec<(String, Vec<(String, String)>)>,
    runner: Option<&L>,
    config: &IngestConfig,
    mut read_page: R,
    mut write_page: W,
) -> Result<CommitReport, crate::ingest::IngestError>
where
    L: LlmRunner,
    R: FnMut(&str) -> Result<Option<String>, crate::ingest::IngestError>,
    W: FnMut(&str, &str) -> Result<(), crate::ingest::IngestError>,
{
    // The configured cap bounds any future concurrent pool; document intent.
    let _llm_cap = clamp_llm_concurrency(Some(config.global_llm_concurrency));

    let mut report = CommitReport::default();
    for (rel_path, entries) in by_page {
        if is_structural(&rel_path) {
            for (source, _) in entries {
                report.merge_errors.push(MergeEntryError {
                    rel_path: rel_path.clone(),
                    source,
                    error: "structural file is never overwritten".into(),
                });
            }
            continue;
        }
        let mut existing = read_page(&rel_path)?;
        for (source, content) in entries {
            let decision = merge_page(existing.as_deref(), &content, runner, 4000);
            match decision {
                MergeDecision::Write(new_content) => {
                    let final_content = ensure_sources(&new_content, &source);
                    write_page(&rel_path, &final_content)?;
                    existing = Some(final_content);
                    if !report.written.contains(&rel_path) {
                        report.written.push(rel_path.clone());
                    }
                }
                MergeDecision::Skip(reason) => {
                    report.merge_errors.push(MergeEntryError {
                        rel_path: rel_path.clone(),
                        source,
                        error: reason,
                    });
                }
            }
        }
    }
    Ok(report)
}
