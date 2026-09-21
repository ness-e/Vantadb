//! Ingest prompts (MEM-30, P7: English). Port of TDAM
//! `ingest-v2/prompts.ts` (generation, one-stage) and `merge.ts`
//! (`MERGE_SYSTEM` / `APPEND_SYSTEM`), trimmed to what the Rust pipeline uses.

use crate::ingest::Frontmatter;

/// System prompt for the extraction stage: the model reads one source chunk
/// and emits candidate pages using the FILE protocol (TDAM buildGeneratePrompt).
pub fn extraction_system_prompt(purpose: &str) -> String {
    format!(
        "You are a knowledge base writer. Read the provided source document and write \
markdown wiki pages that capture its durable knowledge.\n\n\
## Wiki Purpose\n{purpose}\n\n\
## Output Protocol (mandatory)\n\
Emit every page as a FILE block:\n\
<<<FILE path=\"wiki/<category>/<page-name>.md\">>>\n\
---\n\
title: <page title>\n\
type: <entity | concept | source | ...>\n\
description: <one sentence>\n\
---\n\
<page body in markdown>\n\
<<<END>>>\n\n\
Rules:\n- Paths are relative, forward-slash separated, and MUST start with `wiki/`.\n\
- One topic per page; split large topics across pages and cross-reference them with [[wikilinks]].\n\
- Preserve facts verbatim where possible; do not invent content.\n\
- Output ONLY FILE blocks - no commentary before or after."
    )
}

/// User prompt for the extraction stage: existing pages (dedup context) +
/// source name + chunk text.
pub fn extraction_user_prompt(
    source_name: &str,
    chunk_text: &str,
    existing_pages: &[String],
) -> String {
    let existing = if existing_pages.is_empty() {
        "(wiki is empty - this is the first source)".to_string()
    } else {
        existing_pages.join("\n")
    };
    format!(
        "## Existing wiki pages (update/extend these when appropriate)\n{existing}\n\n\
## Source document: {source_name}\n\n{chunk_text}"
    )
}

/// System prompt for full-page merge (TDAM MERGE_SYSTEM, merge.ts).
pub const MERGE_SYSTEM: &str = "You are a knowledge base maintainer. Merge two markdown pages on the same topic into one.
Merge principles:
- Preserve facts from the old page that still hold true - do not lose information.
- Incorporate new information from the new page.
- If old and new conflict, keep both and explicitly note the disagreement.
- Maintain YAML frontmatter format (type is required). Do NOT output a `locked` field.
- Preserve and merge [[wikilink]] cross-references in the body.
- Output the complete merged page directly (including frontmatter) - no extra commentary, no FILE blocks.";

/// System prompt for append merge of oversized pages
/// (TDAM APPEND_SYSTEM, merge.ts): new material goes into a section,
/// never a full rewrite.
pub const APPEND_SYSTEM: &str =
    "You are a knowledge base maintainer. Given an [existing page body] and [new material],
integrate the new material into the existing page WITHOUT rewriting it wholesale:
- Keep all existing sections intact; add or extend sections only where the new material belongs.
- Do not remove any existing information.
- Output the complete updated page directly (including frontmatter) - no extra commentary.";

/// Build an [`crate::core::abstractions::LlmRunParams`]-shaped prompt pair for
/// merging `existing_content` with `candidate_content`. Returns
/// `(system_prompt, user_prompt)`.
pub fn merge_prompts(
    existing_content: Option<&str>,
    candidate_content: &str,
    full_rewrite_max_chars: usize,
) -> (String, String) {
    let Some(existing) = existing_content else {
        // Caller short-circuits this case; defensive fallback = rewrite prompt.
        return (MERGE_SYSTEM.to_string(), candidate_content.to_string());
    };
    let (_, old_body) = crate::ingest::parse_frontmatter(existing);
    if old_body.chars().count() > full_rewrite_max_chars {
        (
            APPEND_SYSTEM.to_string(),
            format!("[existing page body]\n{old_body}\n\n[new material]\n{candidate_content}"),
        )
    } else {
        (
            MERGE_SYSTEM.to_string(),
            format!("## Old page\n{existing}\n\n## New page\n{candidate_content}"),
        )
    }
}

/// Format the existing-pages list used inside extraction prompts
/// (`- [type] relPath - title` lines, TDAM formatExistingPages).
pub fn format_existing_pages(pages: &[(String, Frontmatter)]) -> Vec<String> {
    pages
        .iter()
        .map(|(rel_path, fm)| {
            let t = fm.title.as_deref().unwrap_or("");
            let ty = fm.page_type.as_deref().unwrap_or("unknown");
            format!("- [{ty}] {rel_path} {t}")
        })
        .collect()
}
