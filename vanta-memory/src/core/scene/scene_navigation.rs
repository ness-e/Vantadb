//! Scene navigation: summary index appended to the persona document
//! (MEM-15, F4).
//!
//! Port of TDAM `scene-navigation.ts` (76) adapted to the record store:
//! there are no file paths — a scene's "location" is its name under the
//! `scene/<session>` namespace, so the navigation lists scene names and the
//! host loads them via memory recall/search instead of a read tool.
//!
//! [`NAV_HEADER`] is byte-identical to the TDAM header so
//! [`strip_scene_navigation`] removes exactly what [`generate_scene_navigation`]
//! appends (and anything an LLM copied from it).
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §52-53.

use crate::core::abstractions::SceneIndexEntry;

/// Header of the navigation section (TDAM parity — strip depends on it).
pub const NAV_HEADER: &str = "---\n## Scene Navigation";

/// Build the fire-emoji string for a heat value (visual priority cue).
///
/// Thresholds are the TDAM ones (`scene-navigation.ts:26-33`).
pub fn heat_emoji(heat: u32) -> &'static str {
    match heat {
        h if h >= 1000 => " 🔥🔥🔥🔥🔥",
        h if h >= 500 => " 🔥🔥🔥🔥",
        h if h >= 200 => " 🔥🔥🔥",
        h if h >= 100 => " 🔥🔥",
        h if h >= 50 => " 🔥",
        _ => "",
    }
}

/// Generate the scene navigation Markdown section.
///
/// Entries are sorted by heat descending (navigation order). Empty input →
/// empty string (no navigation section at all).
pub fn generate_scene_navigation(entries: &[SceneIndexEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut sorted: Vec<&SceneIndexEntry> = entries.iter().collect();
    sorted.sort_by_key(|e| std::cmp::Reverse(e.heat));

    let blocks = sorted
        .iter()
        .map(|e| {
            let updated = if e.updated.is_empty() {
                String::new()
            } else {
                format!(" | **Updated**: {}", e.updated)
            };
            format!(
                "### Scene: {}\n**Heat**: {}{}{}\nSummary: {}",
                e.filename,
                e.heat,
                heat_emoji(e.heat),
                updated,
                e.summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "{NAV_HEADER}\n*Index of the current scene memory; load any scene on demand via memory recall/search.*\n\n{blocks}\n\n📌 Usage:\n- Heat: how often the scene has been recalled — higher means more important\n- Summary: the core points of the scene"
    )
}

/// Strip the scene navigation section from persona content.
///
/// Everything from [`NAV_HEADER`] onwards is removed (the navigation is
/// always appended last by the generator).
pub fn strip_scene_navigation(persona_content: &str) -> String {
    match persona_content.find(NAV_HEADER) {
        Some(idx) => persona_content[..idx].trim_end().to_string(),
        None => persona_content.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, heat: u32) -> SceneIndexEntry {
        SceneIndexEntry {
            filename: name.into(),
            summary: format!("summary of {name}"),
            heat,
            created: "2026-08-20T10:00:00.000Z".into(),
            updated: "2026-08-20T11:00:00.000Z".into(),
        }
    }

    #[test]
    fn empty_entries_produce_no_navigation() {
        assert_eq!(generate_scene_navigation(&[]), "");
    }

    #[test]
    fn navigation_sorts_by_heat_descending() {
        let nav = generate_scene_navigation(&[entry("cold", 1), entry("hot", 900)]);
        let hot = nav.find("### Scene: hot").expect("hot block");
        let cold = nav.find("### Scene: cold").expect("cold block");
        assert!(hot < cold, "higher heat first");
    }

    #[test]
    fn navigation_includes_header_heat_and_summary() {
        let nav = generate_scene_navigation(&[entry("deploy", 120)]);
        assert!(nav.starts_with(NAV_HEADER));
        assert!(nav.contains("**Heat**: 120 🔥🔥"));
        assert!(nav.contains("| **Updated**: 2026-08-20T11:00:00.000Z"));
        assert!(nav.contains("Summary: summary of deploy"));
    }

    #[test]
    fn heat_emoji_thresholds_match_tdam() {
        assert_eq!(heat_emoji(0), "");
        assert_eq!(heat_emoji(49), "");
        assert_eq!(heat_emoji(50), " 🔥");
        assert_eq!(heat_emoji(100), " 🔥🔥");
        assert_eq!(heat_emoji(200), " 🔥🔥🔥");
        assert_eq!(heat_emoji(500), " 🔥🔥🔥🔥");
        assert_eq!(heat_emoji(1000), " 🔥🔥🔥🔥🔥");
    }

    #[test]
    fn strip_removes_everything_from_header() {
        let persona = "body text\n\nmore body";
        let nav = generate_scene_navigation(&[entry("s", 1)]);
        let full = format!("{persona}\n\n{nav}\n");
        assert_eq!(strip_scene_navigation(&full), persona);
    }

    #[test]
    fn strip_is_identity_without_header() {
        assert_eq!(strip_scene_navigation("plain persona"), "plain persona");
    }

    #[test]
    fn roundtrip_generate_then_strip_recovers_body() {
        let body = "# User Narrative Profile\n\nArchetype: pragmatic idealist.";
        let nav = generate_scene_navigation(&[entry("a", 3), entry("b", 7)]);
        let full = format!("{body}\n\n{nav}");
        assert_eq!(strip_scene_navigation(&full), body);
    }
}
