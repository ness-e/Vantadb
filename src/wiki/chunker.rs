//! Chunking of oversized wiki sources (MEM-29 — TDAM ingest-v2 parity).
//!
//! Port of `MemoryKnowledge/src/engines/wiki/ingest-v2/chunker.ts`: when a
//! source exceeds the model context budget it is split into chunks with a
//! small inter-chunk overlap. Splitting prefers markdown heading boundaries
//! (`#`–`######`) so semantic sections stay whole; an oversized section falls
//! back to blank-line paragraphs; an oversized paragraph is finally hard-cut.

/// Per-chunk target size in characters (chunker.ts:19).
pub const DEFAULT_TARGET_CHARS: usize = 12_000;

/// Overlap characters between consecutive chunks (chunker.ts:20).
pub const DEFAULT_OVERLAP_CHARS: usize = 400;

/// Split `text` into chunks of at most `target_chars`, aggregating markdown
/// sections and carrying an `overlap_chars` tail between consecutive chunks.
///
/// Empty input yields `[]`; text within the target yields a single chunk.
pub fn chunk_text(text: &str, target_chars: usize, overlap_chars: usize) -> Vec<String> {
    let target = target_chars.max(1000);
    let overlap = overlap_chars.min(target / 2);

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.chars().count() <= target {
        return vec![trimmed.to_string()];
    }

    let units = split_into_units(trimmed, target);

    let mut chunks: Vec<String> = Vec::new();
    let mut buf = String::new();
    for unit in units {
        let sep = usize::from(!buf.is_empty()) * 2;
        let candidate_len = buf.chars().count() + sep + unit.chars().count();
        if candidate_len > target && !buf.is_empty() {
            // Overlap: the tail of the finished buffer opens the next chunk
            // (chunker.ts:86-88).
            let tail = if overlap > 0 {
                tail_chars(&buf, overlap)
            } else {
                String::new()
            };
            chunks.push(std::mem::take(&mut buf));
            if !tail.is_empty() {
                buf.push_str(&tail);
                buf.push_str("\n\n");
            }
            buf.push_str(&unit);
        } else {
            if !buf.is_empty() {
                buf.push_str("\n\n");
            }
            buf.push_str(&unit);
        }
    }
    if !buf.is_empty() {
        chunks.push(buf);
    }
    chunks
}

/// Split into "units": each is preferably one whole markdown section (from a
/// heading line up to the next heading). Oversized sections fall back to
/// blank-line paragraphs, still-oversized paragraphs are hard-cut
/// (chunker.ts:27-63).
fn split_into_units(text: &str, target: usize) -> Vec<String> {
    // First pass: group lines into heading-delimited sections.
    let mut sections: Vec<String> = Vec::new();
    let mut cur = String::new();
    for line in text.lines() {
        if is_heading(line) && !cur.is_empty() {
            sections.push(std::mem::take(&mut cur));
        }
        cur.push_str(line);
        cur.push('\n');
    }
    if !cur.is_empty() {
        sections.push(cur);
    }

    // Second pass: subdivide oversized sections.
    let mut units: Vec<String> = Vec::new();
    for section in sections {
        let s = section.trim();
        if s.is_empty() {
            continue;
        }
        if s.chars().count() <= target {
            units.push(s.to_string());
            continue;
        }
        for para in paragraphs(s) {
            let p = para.trim();
            if p.is_empty() {
                continue;
            }
            if p.chars().count() <= target {
                units.push(p.to_string());
            } else {
                let chars: Vec<char> = p.chars().collect();
                for piece in chars.chunks(target) {
                    units.push(piece.iter().collect());
                }
            }
        }
    }
    units
}

/// Markdown ATX heading: `^#{1,6}\s+\S` (chunker.ts:32).
fn is_heading(line: &str) -> bool {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&hashes) {
        return false;
    }
    let rest = &line[hashes..];
    let mut after_ws = rest.char_indices().skip_while(|&(_, c)| c.is_whitespace());
    match after_ws.next() {
        Some((_, c)) => !c.is_whitespace(),
        None => false,
    }
}

/// Split on blank-line boundaries (`/\n\s*\n/`, chunker.ts:52).
fn paragraphs(s: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let mut start: Option<usize> = None;
    let mut end = 0;
    for line in s.split_inclusive('\n') {
        let offset = end;
        end += line.len();
        if line.trim().is_empty() {
            if let Some(st) = start.take() {
                out.push(&s[st..offset]);
            }
        } else if start.is_none() {
            start = Some(offset);
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out
}

/// Last `n` characters of `s`.
fn tail_chars(s: &str, n: usize) -> String {
    let skip = s.chars().count().saturating_sub(n);
    s.chars().skip(skip).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A paragraph of `n` filler characters.
    fn filler(n: usize, tag: &str) -> String {
        let word = format!("{tag} ");
        let mut s = String::new();
        while s.chars().count() < n {
            s.push_str(&word);
        }
        s.chars().take(n).collect()
    }

    // ── (b) chunker 12000/400 produce chunks esperados ──

    #[test]
    fn short_text_is_single_chunk() {
        let chunks = chunk_text("# Title\n\nhello world", 12_000, 400);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "# Title\n\nhello world");
    }

    #[test]
    fn empty_text_yields_no_chunks() {
        assert!(chunk_text("", 12_000, 400).is_empty());
        assert!(chunk_text("   \n\t ", 12_000, 400).is_empty());
    }

    #[test]
    fn defaults_split_two_sections_into_two_chunks_with_overlap() {
        let text = format!(
            "# One\n\n{}\n\n# Two\n\n{}\n",
            filler(8_000, "a"),
            filler(8_000, "b")
        );
        let chunks = chunk_text(&text, DEFAULT_TARGET_CHARS, DEFAULT_OVERLAP_CHARS);
        assert_eq!(
            chunks.len(),
            2,
            "~16k chars across two 8k sections → two chunks"
        );
        for chunk in &chunks {
            assert!(
                chunk.chars().count() <= DEFAULT_TARGET_CHARS,
                "every chunk respects the 12000 target"
            );
        }
        // Overlap: the second chunk opens with the 400-char tail of the first.
        let tail: String = chunks[0]
            .chars()
            .skip(chunks[0].chars().count() - 400)
            .collect();
        assert!(
            chunks[1].starts_with(&tail),
            "second chunk must carry the 400-char overlap"
        );
        // Order preserved: 'a' filler only in chunk 1, 'b' only introduced in 2.
        assert!(chunks[0].contains("# One"));
        assert!(!chunks[0].contains("# Two"));
    }

    // ── (d) boundaries sin corromper estructura ──

    #[test]
    fn headings_survive_verbatim_across_chunks() {
        let text = [
            "# Alpha\n\n".to_string() + &filler(5_000, "α"),
            "## Beta\n\n".to_string() + &filler(5_000, "β"),
            "### Gamma\n\n".to_string() + &filler(5_000, "γ"),
        ]
        .join("\n");
        let chunks = chunk_text(&text, 6_000, 200);
        assert!(chunks.len() >= 2);
        for heading in ["# Alpha", "## Beta", "### Gamma"] {
            let intact = chunks.iter().any(|c| c.contains(&format!("{heading}\n")));
            assert!(intact, "heading `{heading}` must survive uncut");
        }
    }

    #[test]
    fn hard_cut_preserves_all_characters_in_order() {
        // No edge whitespace: chunk_text trims, so filler must be boundary-clean.
        let para: String = "x".repeat(25_000); // single paragraph > target → hard-cut
        let chunks = chunk_text(&para, 12_000, 0);
        assert!(chunks.len() >= 3);
        let rejoined: String = chunks.join("");
        assert_eq!(
            rejoined,
            "x".repeat(25_000),
            "hard-cut chunks reassemble the original content"
        );
    }

    #[test]
    fn heading_detection_matches_tdam_regex_semantics() {
        // `^#{1,6}\s+\S` (chunker.ts:32).
        assert!(is_heading("# H"));
        assert!(is_heading("###### Deep"));
        assert!(is_heading("#\tTabbed"));
        assert!(!is_heading("####### seven hashes"));
        assert!(!is_heading("no hash"));
        assert!(!is_heading("#"));
        assert!(!is_heading("   # indented"));
    }
}
