//! Persona trigger: decides whether to regenerate the persona (MEM-15, F4).
//!
//! LLM-free heuristics implementing the five TDAM trigger conditions
//! (`persona-trigger.ts:35-96`) as a pure function: the checkpoint counters
//! (`scenes_processed`, `memories_since_last_persona`, …) are INPUTS here —
//! reading them from the persistent checkpoint is MEM-16 orchestration.
//!
//! Priorities (highest wins, matching the existing
//! [`PersonaTriggerPriority`] ordering from MEM-08b):
//! P1 explicit request > P2 cold start > P2 recovery > P3 first scene >
//! P4 memory-count threshold.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` §41.

use crate::core::abstractions::PersonaTriggerPriority;

/// Inputs for one trigger evaluation (all derivable from the store +
/// checkpoint by the caller).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonaTriggerInput {
    /// The agent explicitly requested a persona update (LLM signal
    /// `[PERSONA_UPDATE_REQUEST]`).
    pub request_persona_update: bool,
    /// The agent's reason for the explicit request (P1 detail).
    pub request_reason: Option<String>,
    /// Scenes processed so far (0 before the first L2 extraction).
    pub scenes_processed: usize,
    /// Memories recorded since the last persona generation.
    pub memories_since_last_persona: usize,
    /// At least one live scene block exists in the store.
    pub has_scene_blocks: bool,
    /// A persona was generated before (checkpoint `last_persona_at > 0`).
    pub previously_generated: bool,
    /// The stored persona has a non-empty body (navigation stripped).
    pub has_persona_body: bool,
}

/// Outcome of [`evaluate_persona_trigger`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerResult {
    /// Whether persona generation should run.
    pub should: bool,
    /// The winning priority (`None` when `should` is `false`).
    pub priority: Option<PersonaTriggerPriority>,
    /// Human-readable reason (English — Principio 7).
    pub reason: String,
}

/// Evaluate the TDAM trigger priorities for one generation decision.
///
/// Pure and deterministic: same input + interval → same result (D19).
pub fn evaluate_persona_trigger(
    input: &PersonaTriggerInput,
    trigger_every_n: usize,
) -> TriggerResult {
    // P1: the agent explicitly requested an update.
    if input.request_persona_update {
        return TriggerResult {
            should: true,
            priority: Some(PersonaTriggerPriority::P1Request),
            reason: input
                .request_reason
                .clone()
                .unwrap_or_else(|| "explicit agent request".to_string()),
        };
    }

    let has_generated_persona = input.previously_generated || input.has_persona_body;

    // P2 cold start: first extraction done, no persona yet, scene blocks exist.
    if input.scenes_processed > 0 && !has_generated_persona && input.has_scene_blocks {
        return TriggerResult {
            should: true,
            priority: Some(PersonaTriggerPriority::P2ColdStart),
            reason: "cold start: first extraction completed with scene blocks and no persona"
                .to_string(),
        };
    }

    // P2 recovery: generated before but the body is now empty/corrupted.
    if input.previously_generated && input.has_scene_blocks && !input.has_persona_body {
        return TriggerResult {
            should: true,
            priority: Some(PersonaTriggerPriority::P2Recovery),
            reason: "recovery: persona body missing or empty, regenerating".to_string(),
        };
    }

    // P3: the very first scene block was just extracted.
    if input.scenes_processed == 1 && input.memories_since_last_persona > 0 {
        return TriggerResult {
            should: true,
            priority: Some(PersonaTriggerPriority::P3FirstScene),
            reason: "first scene block extraction completed".to_string(),
        };
    }

    // P4: memory-count threshold reached.
    if input.memories_since_last_persona >= trigger_every_n {
        return TriggerResult {
            should: true,
            priority: Some(PersonaTriggerPriority::P4MemoryCount),
            reason: format!(
                "threshold reached: {count} >= {n}",
                count = input.memories_since_last_persona,
                n = trigger_every_n
            ),
        };
    }

    TriggerResult {
        should: false,
        priority: None,
        reason: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PersonaTriggerInput {
        PersonaTriggerInput {
            request_persona_update: false,
            request_reason: None,
            scenes_processed: 5,
            memories_since_last_persona: 3,
            has_scene_blocks: true,
            previously_generated: true,
            has_persona_body: true,
        }
    }

    #[test]
    fn p1_explicit_request_wins_over_everything() {
        let mut input = base();
        input.request_persona_update = true;
        input.request_reason = Some("user asked".into());
        let result = evaluate_persona_trigger(&input, 50);
        assert_eq!(result.priority, Some(PersonaTriggerPriority::P1Request));
        assert_eq!(result.reason, "user asked");

        // Without a reason there is still a default one.
        input.request_reason = None;
        let result = evaluate_persona_trigger(&input, 50);
        assert!(result.should);
        assert_eq!(result.reason, "explicit agent request");
    }

    #[test]
    fn p2_cold_start_fires_once() {
        let mut input = base();
        input.previously_generated = false;
        input.has_persona_body = false;
        input.memories_since_last_persona = 0; // would not fire P4
        let result = evaluate_persona_trigger(&input, 50);
        assert_eq!(result.priority, Some(PersonaTriggerPriority::P2ColdStart));

        // No scene blocks → no cold start.
        input.has_scene_blocks = false;
        assert!(!evaluate_persona_trigger(&input, 50).should);
    }

    #[test]
    fn p2_recovery_when_body_lost() {
        let mut input = base();
        input.has_persona_body = false; // generated before, body gone
        input.memories_since_last_persona = 0;
        let result = evaluate_persona_trigger(&input, 50);
        assert_eq!(result.priority, Some(PersonaTriggerPriority::P2Recovery));
    }

    #[test]
    fn p3_first_scene_extraction() {
        let mut input = base();
        // A persona already exists (body intact) — P2 cold start/recovery do
        // not apply; the very first scene extraction still triggers.
        input.scenes_processed = 1;
        input.memories_since_last_persona = 2;
        let result = evaluate_persona_trigger(&input, 50);
        assert_eq!(result.priority, Some(PersonaTriggerPriority::P3FirstScene));
    }

    #[test]
    fn p4_threshold_at_interval() {
        let mut input = base();
        input.memories_since_last_persona = 50;
        let result = evaluate_persona_trigger(&input, 50);
        assert_eq!(result.priority, Some(PersonaTriggerPriority::P4MemoryCount));
        assert!(result.reason.contains("50 >= 50"));

        // Below threshold → no trigger.
        input.memories_since_last_persona = 49;
        let result = evaluate_persona_trigger(&input, 50);
        assert!(!result.should);
        assert_eq!(result.priority, None);
    }

    #[test]
    fn quiet_state_does_not_trigger() {
        let result = evaluate_persona_trigger(&base(), 50);
        assert!(!result.should);
        assert_eq!(result.reason, "");
    }
}
