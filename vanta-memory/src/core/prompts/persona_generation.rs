//! L3 persona generation prompts (MEM-15, F4).
//!
//! REWRITTEN in English from the TDAM persona-generation principles
//! (Principio 7: reescribir, no traducir — the TDAM originals are Chinese).
//! Two families via the existing [`PromptMode`]: chat (user persona,
//! four-layer deep scan) and work/team (operating doctrine).
//!
//! Documented divergence: TDAM runs an LLM agent that writes `persona.md`
//! via sandboxed write/edit tools; this port emits a **JSON contract**
//! `{"persona": "<full markdown document>"}` that the deterministic layer
//! ([`crate::core::persona::persona_generator`]) validates, escapes and
//! persists. The tool-call agent loop is MEM-16 orchestration.

use crate::core::abstractions::PersonaMode;
use crate::core::prompts::l1_extraction::PromptMode;

/// Hard character limit of the persona body in chat mode (TDAM system prompt:
/// "persona.md must not exceed 2000 characters").
pub const MAX_PERSONA_CHARS_CHAT: usize = 2000;

/// Hard character limit of the doctrine body in work mode (TDAM: "no more
/// than 1200 characters").
pub const MAX_PERSONA_CHARS_WORK: usize = 1200;

/// Mechanical character limit for a prompt family.
pub fn persona_char_limit(prompt_mode: PromptMode) -> usize {
    match prompt_mode {
        PromptMode::Chat => MAX_PERSONA_CHARS_CHAT,
        PromptMode::Code => MAX_PERSONA_CHARS_WORK,
    }
}

/// Parameters for building one L3 persona generation prompt.
#[derive(Debug, Clone)]
pub struct PersonaPromptParams {
    /// Generation mode: first (from scratch) or incremental.
    pub mode: PersonaMode,
    /// Prompt family: chat persona vs work doctrine.
    pub prompt_mode: PromptMode,
    /// Current time (ISO 8601).
    pub current_time: String,
    /// Total memories processed so far.
    pub total_processed: usize,
    /// Total live scenes in the index.
    pub scene_count: usize,
    /// Scenes changed since the last generation.
    pub changed_scene_count: usize,
    /// Pre-loaded full content of the changed scenes (markdown blocks).
    pub changed_scenes_content: String,
    /// Existing persona body (navigation already stripped), when updating.
    pub existing_persona: Option<String>,
    /// Why the generation was triggered (optional context for the LLM).
    pub trigger_info: Option<String>,
}

/// The built prompt pair.
#[derive(Debug, Clone)]
pub struct PersonaPromptResult {
    /// System prompt: role + constraints + logic + template + output contract.
    pub system_prompt: String,
    /// User prompt: dynamic data (stats, changed scenes, existing persona).
    pub user_prompt: String,
}

/// Build the L3 persona generation prompt.
pub fn build_persona_prompt(params: PersonaPromptParams) -> PersonaPromptResult {
    let system_prompt = match params.prompt_mode {
        PromptMode::Chat => PERSONA_SYSTEM_PROMPT,
        PromptMode::Code => DOCTRINE_SYSTEM_PROMPT,
    };

    let mode_label = match params.mode {
        PersonaMode::First => "FIRST GENERATION",
        PersonaMode::Incremental => "INCREMENTAL UPDATE",
    };
    let trigger_section = params
        .trigger_info
        .map(|info| format!("\n### Trigger\n{info}\n"))
        .unwrap_or_default();

    let existing_section = match &params.existing_persona {
        Some(existing) => format!(
            "\n## Current document (pre-loaded, {} chars)\n\nUpdate from it; keep the result within the character limit:\n\n```markdown\n{existing}\n```\n\n---\n",
            existing.chars().count()
        ),
        None => String::new(),
    };

    let iteration_guide = match (params.mode, params.prompt_mode) {
        (PersonaMode::Incremental, PromptMode::Chat) => {
            "\n## Iteration guide\n\nFor each changed scene decide autonomously: reinforce (confirms an existing insight) / extend (new dimension) / revise (contradicts) / restructure (document drifted) / no-change (nothing useful). Do not append every change as a new entry — keep compressing.\n"
        }
        (PersonaMode::Incremental, PromptMode::Code) => {
            "\n## Iteration guide\n\nFor each changed scene decide autonomously: reinforce / supplement (new reusable SOP, boundary or rule) / revise (principle overturned) / refactor (document grew long or project-specific) / no-change (only project state). Keep compressing; precision over volume.\n"
        }
        (PersonaMode::First, _) => "",
    };

    let user_prompt = format!(
        "**Output language**: use the dominant language of the changed scene content below. Markdown syntax and JSON field names stay English.\n\n\
         **Time**: {time}\n\
         **Mode**: {mode_label}\n\
         {trigger_section}\
         ## Stats\n\
         - **Total memories**: {total}\n\
         - **Total scenes**: {scenes}\n\
         - **Changed scenes**: {changed} (since the last update)\n\n\
         ---\n\
         {changed_scenes}\
         \n{existing_section}{iteration_guide}",
        time = params.current_time,
        mode_label = mode_label,
        trigger_section = trigger_section,
        total = params.total_processed,
        scenes = params.scene_count,
        changed = params.changed_scene_count,
        changed_scenes = params.changed_scenes_content,
        existing_section = existing_section,
        iteration_guide = iteration_guide,
    );

    PersonaPromptResult {
        system_prompt: system_prompt.to_string(),
        user_prompt,
    }
}

const PERSONA_SYSTEM_PROMPT: &str = r#"You are the Persona Architect for a personal AI memory system. Combine the existing persona document (if any) with the new/changed scene blocks and produce an updated user persona using the four-layer deep scan model.

OUTPUT CONTRACT — return ONLY a valid JSON object:
{"persona": "<the complete markdown persona document>"}
No markdown code fences, no explanatory text outside the JSON.

HARD CONSTRAINTS:
1. LENGTH: the persona document must not exceed 2000 characters. Summarize aggressively and drop unimportant details.
2. NO SPECULATION: do not invent facts absent from the scene data. During cold start, restraint beats hallucination — leave fields empty rather than guess.
3. SCENE DATA ONLY: every statement must come from the provided scene data. Never extract personal information from workspace structure, file paths, or system metadata.
4. NO NAVIGATION: do not append any scene navigation/index section — the engine adds it automatically.

CORE LOGIC — connect & synthesize (narrative coherence, no bullet-point spamming). Run the four-layer deep scan:

- Layer 1 — Base & Facts: confirmed facts, demographics, current state. Value: ice-breakers and context awareness.
- Layer 2 — Interest Graph: what the user invests time, money or attention in. Distinguish active hobbies / passive consumption / dormant interests. Value: quality chit-chat and recommendations.
- Layer 3 — Interaction Protocol: communication habits, landmines, workflow preferences. Value: how to speak and deliver results without friction.
- Layer 4 — Cognitive Core: decision logic, contradictions, ultimate drivers. Value: act as a co-pilot that can decide for the user.

OUTPUT TEMPLATE (adjust chapters freely when information is scarce; keep markdown):

# User Narrative Profile

> **Archetype**: [one sentence]

> **Basic info**
-

> **Long-term preferences**
-

## Chapter 1: Context & Current State
[coherent narrative]

## Chapter 2: The Texture of Life
[interests, consumption, habits]

## Chapter 3: Interaction & Cognitive Protocol
### How to Speak
### How to Think

## Chapter 4: Deep Insights & Evolution
- **Contradictions**: [conflicting but coherent traits]
- **Trajectory**: [recent changes over time]
- **Emerging traits**: 3-7 core trait tags, one per line with a short note"#;

const DOCTRINE_SYSTEM_PROMPT: &str = r#"You are the Team Operating Doctrine Architect for an AI memory system embedded in a work environment. Combine the existing doctrine (if any) with the new/changed L2 scene blocks and produce a highly distilled team operating doctrine.

This document is NOT a project summary, progress log, scene index or fact dump: it is the reusable Operating Doctrine the agent applies to future tasks — how to judge, how to execute, how to avoid mistakes.

OUTPUT CONTRACT — return ONLY a valid JSON object:
{"persona": "<the complete markdown doctrine document>"}
No markdown code fences, no explanatory text outside the JSON.

HARD CONSTRAINTS:
1. LENGTH: at most 1200 characters. Precision over volume.
2. NO PROJECT FRAGMENTS: nothing only understandable inside one project's context ("v2 must optimize X", "module Y continues").
3. NO LOGS: never record what happened, who did what, or task status unless abstracted into a general method.
4. NO LOW-LEVEL FACT PILES: project names, versions, PRs, issues stay out unless they represent a reusable pattern.
5. SEMANTIC COMPLETENESS: every principle must include its action object, applicability condition or judgement logic.
6. NO PERSON PROFILES: no member personality, private preferences or emotional judgements.
7. NO SPECULATION: no invention beyond scene evidence.
8. NO NAVIGATION: the engine appends the scene navigation automatically.

EXTRACT (reusable across work contexts): SOPs, Principles, Decision Logic, Boundaries, Anti-patterns, Agent Rules.

FILTER before writing — if any answer is no, do not write the item:
1. General? (applies to multiple projects/tasks) 2. Complete? (understandable out of context) 3. Actionable? (changes future behavior) 4. Stable? (long-lived, not one-off state) 5. Distilled? (could it merge into an existing principle?)

INCREMENTAL STRATEGY: reinforce (new scenes only confirm) / supplement (new general rule) / revise (evidence overturned a principle) / refactor (document grew long or project-specific — compress wholesale) / no-change (only project state arrived).

OUTPUT TEMPLATE (keep markdown):

# Team Operating Doctrine

> **Operating Thesis**: [one sentence]

## Core Principles
- [principle]: [condition / logic / why]

## Reusable SOPs
- [name]: when [trigger], first [step], then [step], finally [acceptance].

## Decision Logic
- When [situation], prefer [A] over [B] because [reason].

## Boundaries & Anti-patterns
- Do not [mistake]; instead [practice], because [reason].

## Agent Rules
- Agent should [rule], avoiding [risk]."#;

#[cfg(test)]
mod tests {
    use super::*;

    fn params(mode: PersonaMode, prompt_mode: PromptMode) -> PersonaPromptParams {
        PersonaPromptParams {
            mode,
            prompt_mode,
            current_time: "2026-08-20T12:00:00.000Z".into(),
            total_processed: 42,
            scene_count: 7,
            changed_scene_count: 2,
            changed_scenes_content: "### [1] deploy-runbook\n\n```markdown\nc\n```".into(),
            existing_persona: None,
            trigger_info: None,
        }
    }

    #[test]
    fn chat_system_covers_layers_constraints_and_contract() {
        let result = build_persona_prompt(params(PersonaMode::First, PromptMode::Chat));
        assert!(result.system_prompt.contains("four-layer deep scan"));
        assert!(result.system_prompt.contains("2000 characters"));
        assert!(result.system_prompt.contains("NO SPECULATION"));
        assert!(result.system_prompt.contains("\"persona\""));
        assert!(
            !result.system_prompt.contains("persona.md"),
            "record store: no file target"
        );
    }

    #[test]
    fn work_system_is_doctrine_with_own_limit() {
        let result = build_persona_prompt(params(PersonaMode::First, PromptMode::Code));
        assert!(result.system_prompt.contains("Operating Doctrine"));
        assert!(result.system_prompt.contains("1200"));
        assert_eq!(persona_char_limit(PromptMode::Code), MAX_PERSONA_CHARS_WORK);
        assert_eq!(persona_char_limit(PromptMode::Chat), MAX_PERSONA_CHARS_CHAT);
    }

    #[test]
    fn user_prompt_carries_stats_and_changed_scenes() {
        let result = build_persona_prompt(params(PersonaMode::First, PromptMode::Chat));
        assert!(result.user_prompt.contains("**Mode**: FIRST GENERATION"));
        assert!(result.user_prompt.contains("- **Total memories**: 42"));
        assert!(result.user_prompt.contains("- **Changed scenes**: 2"));
        assert!(result.user_prompt.contains("deploy-runbook"));
        assert!(!result.user_prompt.contains("Current document"));
    }

    #[test]
    fn incremental_includes_existing_persona_and_guide() {
        let mut p = params(PersonaMode::Incremental, PromptMode::Chat);
        p.existing_persona = Some("# User Narrative Profile\nold".into());
        p.trigger_info = Some("threshold reached".into());
        let result = build_persona_prompt(p);
        assert!(result.user_prompt.contains("**Mode**: INCREMENTAL UPDATE"));
        assert!(result
            .user_prompt
            .contains("### Trigger\nthreshold reached"));
        assert!(result
            .user_prompt
            .contains("Current document (pre-loaded, 28 chars)"));
        assert!(result.user_prompt.contains("Iteration guide"));
    }

    #[test]
    fn first_mode_has_no_iteration_guide() {
        let result = build_persona_prompt(params(PersonaMode::First, PromptMode::Chat));
        assert!(!result.user_prompt.contains("Iteration guide"));
    }
}
