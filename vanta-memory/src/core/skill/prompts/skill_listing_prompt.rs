//! Skill listing prompt boilerplate (MEM-17, F4) — frames the
//! `<available_skills>` block injected into agent system prompts.
//!
//! Rewritten in English from TDAM `MC/core/skill/prompts/skill-listing-prompt.ts`
//! (Principio 7: restated, not translated verbatim — wording kept close to the
//! design-doc source so host integrations stay interchangeable).

/// Header injected above the rendered skill list.
pub const SKILL_LISTING_HEADER: &str = "## Skills (mandatory)\n\
Before replying, scan the skills below. If a skill matches or is even partially relevant \
to your task, you MUST load it with skill_view(name) and follow its instructions. \
Err on the side of loading — it is always better to have context you don't need \
than to miss critical steps, pitfalls, or established workflows. \
Skills contain specialized knowledge — API endpoints, tool-specific commands, \
and proven workflows that outperform general-purpose approaches. Load the skill \
even if you think you could handle the task with basic tools like web_search or terminal. \
Skills also encode the user's preferred approach, conventions, and quality standards \
for tasks like code review, planning, and testing — load them even for tasks you \
already know how to do, because the skill defines how it should be done here.\n\
If a skill has issues, fix it with skill_manage(action='patch').\n\
After difficult/iterative tasks, offer to save as a skill. \
If a skill you loaded was missing steps, had wrong commands, or needed \
pitfalls you discovered, update it before finishing.\n";

/// Footer injected below the rendered skill list.
pub const SKILL_LISTING_FOOTER: &str =
    "\nOnly proceed without loading a skill if genuinely none are relevant to the task.";

/// Tool-usage guidance, attached separately from the skill listing.
pub const SKILLS_GUIDANCE: &str =
    "After completing a complex task (5+ tool calls), fixing a tricky error, \
or discovering a non-trivial workflow, save the approach as a \
skill with skill_manage so you can reuse it next time.\n\
When using a skill and finding it outdated, incomplete, or wrong, \
patch it immediately with skill_manage(action='patch') — don't wait to be asked. \
Skills that aren't maintained become liabilities.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_non_empty_and_english() {
        assert!(SKILL_LISTING_HEADER.contains("## Skills (mandatory)"));
        assert!(SKILL_LISTING_FOOTER.contains("Only proceed without loading a skill"));
        assert!(SKILLS_GUIDANCE.contains("save the approach as a"));
    }
}
