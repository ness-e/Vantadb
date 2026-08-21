//! L2 scene extraction strategy (MEM-14, F4) — UPDATE>MERGE>CREATE + heat +
//! soft-delete.
//!
//! TDAM leaves the strategy decision to the LLM agent via sandboxed tools
//! (`scene-extraction.ts` Phase 2: UPDATE preferred > MERGE > CREATE last
//! resort). This port separates the **decision** ([`decide_strategy`], pure,
//! D19-testable) from the **execution** ([`apply_strategy`]): the LLM emits
//! extractions as JSON `{scene_name, summary, content, merge_sources}` (its
//! overlap judgement) and the deterministic layer applies heat/soft-delete.
//!
//! Execution rules:
//! - UPDATE/CREATE go through [`execute_scene_tool`] (MEM-13 dispatcher —
//!   boundary validation + `upsert_scene` heat semantics reused, no
//!   duplication).
//! - MERGE cannot express `heat = sum + 1` through the write tool (it would
//!   bump `old + 1`); it uses the scene_index primitives
//!   ([`write_scene_block`] + [`soft_delete_scene`]) with explicit heat.
//! - Soft-delete marks the block (flag + `[DELETED]` marker content); nothing
//!   is ever physically removed (recoverable via `get_scene`).
//!
//! `emptyExtraction` (TDAM `scene-extractor.ts:509-516`): an empty batch — or
//! a failed LLM run (Principio 4) — never overwrites the store.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` + TDAM
//! `scene-extractor.ts` (604) + `scene-extraction.ts` (572).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::abstractions::{LlmRunParams, LlmRunner, SceneIndexEntry, SceneMeta};
use crate::core::conversation::now_ms;
use crate::core::prompts::l1_extraction::epoch_ms_to_rfc3339;
use crate::core::prompts::scene_extraction::{
    build_scene_extraction_prompt, SceneExtractionPromptParams,
};
use crate::core::scene::filename_normalizer::normalize_scene_name;
use crate::core::scene::scene_format::SceneBlock;
use crate::core::scene::scene_index::{
    get_scene, list_scenes, soft_delete_scene, write_scene_block, SceneError,
};
use crate::core::scene::scene_tools::{
    execute_scene_tool, validate_content, validate_scene_name, validate_text, SceneToolCall,
    SceneToolError, SceneToolResult, MAX_SUMMARY_BYTES,
};

/// A single L2 scene extraction decision, as emitted by the LLM.
///
/// Serde snake_case so the L2 prompt's JSON contract deserializes directly
/// (LLM output is untrusted input — every field is revalidated downstream).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SceneExtraction {
    /// Target scene name (may already exist → UPDATE, else CREATE/MERGE).
    pub scene_name: String,
    /// Narrative summary of the scene.
    pub summary: String,
    /// Scene content. `"[DELETED]"` (the marker) requests a soft-delete;
    /// empty/whitespace-only content is skipped.
    pub content: String,
    /// Scene names this extraction merges into `scene_name` (overlap
    /// judgement by the LLM).
    #[serde(default)]
    pub merge_sources: Vec<String>,
}

/// Input memory for an L2 extraction run (TDAM `extract()` L141 parity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMemoryInput {
    /// Memory record ID.
    pub id: String,
    /// Memory content.
    pub content: String,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
}

/// Strategy decided for one extraction (pure decision — D19-testable).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SceneStrategy {
    /// Name matches an existing scene → in-place update (heat = old + 1).
    Update { scene_name: String },
    /// New scene → create (heat = 1).
    Create { scene_name: String },
    /// Overlapping sources declared → merge into target
    /// (heat = target + Σ sources + 1, sources soft-deleted).
    Merge {
        scene_name: String,
        sources: Vec<String>,
    },
    /// Content is the `[DELETED]` marker → soft-delete the named scene.
    SoftDelete { scene_name: String },
    /// Empty/whitespace content, marker for a missing scene, or a merge with
    /// no valid sources → do nothing.
    Skip,
}

/// What `apply_strategy` did with one extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SceneAction {
    Updated {
        scene_name: String,
    },
    Created {
        scene_name: String,
    },
    Merged {
        scene_name: String,
        sources: Vec<String>,
    },
    SoftDeleted {
        scene_name: String,
    },
    Skipped,
}

/// Result of one [`apply_strategy`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneApplyResult {
    /// The action applied.
    pub action: SceneAction,
    /// The written block, when the strategy wrote one (`None` for
    /// SoftDelete/Skip).
    pub scene: Option<SceneBlock>,
}

/// Result of a batch [`extract_scenes`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneExtractionResult {
    /// Whether the run succeeded (an LLM-free failure sets `false`).
    pub success: bool,
    /// `true` when nothing was extracted (empty batch or failed LLM run) —
    /// the store was NOT touched.
    pub empty_extraction: bool,
    /// The actions applied, in batch order.
    pub applied: Vec<SceneAction>,
    /// Human-readable error, when `success` is `false`.
    pub error: Option<String>,
}

/// Errors surfaced by the L2 scene strategy.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SceneExtractorError {
    /// Underlying scene index / storage error.
    #[error("scene index: {0}")]
    Scene(#[from] SceneError),
    /// Sandboxed scene tool error (UPDATE/CREATE path).
    #[error("scene tool: {0}")]
    Tool(#[from] SceneToolError),
    /// Input rejected at the strategy boundary.
    #[error("invalid scene extraction input: {0}")]
    Invalid(String),
}

/// Decide the L2 strategy for one extraction against the existing scene index.
///
/// Deterministic (pure): UPDATE when the (normalized) name matches, MERGE
/// when `merge_sources` name live scenes other than the target, CREATE
/// otherwise. Content equal to the `[DELETED]` marker → SoftDelete; empty or
/// whitespace-only content → Skip.
pub fn decide_strategy(
    extraction: &SceneExtraction,
    existing: &[SceneIndexEntry],
) -> SceneStrategy {
    let name = normalize_scene_name(&extraction.scene_name);
    let live: Vec<&str> = existing.iter().map(|e| e.filename.as_str()).collect();

    // Marker content → soft-delete (TDAM: writing [DELETED] deletes a file).
    if extraction.content.trim() == "[DELETED]" {
        return if live.contains(&name.as_str()) {
            SceneStrategy::SoftDelete { scene_name: name }
        } else {
            SceneStrategy::Skip
        };
    }
    // Empty/whitespace-only content → skip (no empty scenes; parity with the
    // write-tool rejection of blank content).
    if extraction.content.trim().is_empty() {
        return SceneStrategy::Skip;
    }
    // Name match → UPDATE is the default strategy (TDAM: prefer UPDATE over
    // CREATE when in doubt).
    if live.contains(&name.as_str()) {
        return SceneStrategy::Update { scene_name: name };
    }
    // Declared overlap → MERGE, but only against sources that actually exist
    // and are not the target itself.
    if !extraction.merge_sources.is_empty() {
        let sources: Vec<String> = extraction
            .merge_sources
            .iter()
            .map(|s| normalize_scene_name(s))
            .filter(|s| s != &name && live.contains(&s.as_str()))
            .collect();
        if !sources.is_empty() {
            return SceneStrategy::Merge {
                scene_name: name,
                sources,
            };
        }
    }
    SceneStrategy::Create { scene_name: name }
}

/// Apply a decided strategy to the store (UPDATE/CREATE via the sandboxed
/// tools; MERGE/soft-delete via scene_index primitives).
pub fn apply_strategy(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    extraction: &SceneExtraction,
    strategy: &SceneStrategy,
) -> Result<SceneApplyResult, SceneExtractorError> {
    match strategy {
        SceneStrategy::Update { scene_name } | SceneStrategy::Create { scene_name } => {
            let result = execute_scene_tool(
                db,
                session_key,
                &SceneToolCall::Write {
                    scene_name: scene_name.clone(),
                    summary: extraction.summary.clone(),
                    content: extraction.content.clone(),
                },
            )?;
            let SceneToolResult::Write { scene } = result else {
                return Err(SceneExtractorError::Invalid(
                    "write tool returned an unexpected result".into(),
                ));
            };
            let action = if matches!(strategy, SceneStrategy::Update { .. }) {
                SceneAction::Updated {
                    scene_name: scene_name.clone(),
                }
            } else {
                SceneAction::Created {
                    scene_name: scene_name.clone(),
                }
            };
            Ok(SceneApplyResult {
                action,
                scene: Some(scene),
            })
        }
        SceneStrategy::Merge {
            scene_name,
            sources,
        } => {
            // Boundary validation (MERGE bypasses the tools, so it reuses the
            // tool validators — no duplicated logic).
            validate_scene_name(scene_name)?;
            validate_text("summary", &extraction.summary, MAX_SUMMARY_BYTES)?;
            validate_content(&extraction.content)?;

            // Heat = target_heat + Σ(source_heat) + 1 (TDAM: sum of all
            // related blocks + 1), saturating so an overflow never panics.
            let target = get_scene(db, session_key, scene_name)?;
            let mut heat = 0u32;
            if let Some(block) = &target {
                heat = heat.saturating_add(block.meta.heat);
            }
            for source in sources {
                if let Some(block) = get_scene(db, session_key, source)? {
                    heat = heat.saturating_add(block.meta.heat);
                }
            }
            heat = heat.saturating_add(1);

            let now = epoch_ms_to_rfc3339(now_ms());
            let created = target
                .map(|b| b.meta.created)
                .unwrap_or_else(|| now.clone());
            let meta = SceneMeta {
                created,
                updated: now,
                summary: extraction.summary.clone(),
                heat,
            };
            let block = SceneBlock::new(scene_name.clone(), meta, extraction.content.clone());
            write_scene_block(db, session_key, &block)?;

            // TDAM order: write the target first, then soft-delete sources — a
            // source-delete failure leaves the merged target intact (no loss).
            for source in sources {
                soft_delete_scene(db, session_key, source)?;
            }
            Ok(SceneApplyResult {
                action: SceneAction::Merged {
                    scene_name: scene_name.clone(),
                    sources: sources.clone(),
                },
                scene: Some(block),
            })
        }
        SceneStrategy::SoftDelete { scene_name } => {
            soft_delete_scene(db, session_key, scene_name)?;
            Ok(SceneApplyResult {
                action: SceneAction::SoftDeleted {
                    scene_name: scene_name.clone(),
                },
                scene: None,
            })
        }
        SceneStrategy::Skip => Ok(SceneApplyResult {
            action: SceneAction::Skipped,
            scene: None,
        }),
    }
}

/// Run a batch of scene extractions against a session (LLM-free entry point).
///
/// An empty batch is `emptyExtraction`: it returns `empty_extraction: true`
/// and never touches the store (TDAM parity — an empty extraction must not
/// overwrite the scene index).
pub fn extract_scenes(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    extractions: &[SceneExtraction],
) -> Result<SceneExtractionResult, SceneExtractorError> {
    if extractions.is_empty() {
        return Ok(SceneExtractionResult {
            success: true,
            empty_extraction: true,
            applied: vec![],
            error: None,
        });
    }

    let existing = list_scenes(db, session_key)?;
    let mut applied = Vec::with_capacity(extractions.len());
    for extraction in extractions {
        let strategy = decide_strategy(extraction, &existing);
        let result = apply_strategy(db, session_key, extraction, &strategy)?;
        applied.push(result.action);
    }

    Ok(SceneExtractionResult {
        success: true,
        empty_extraction: false,
        applied,
        error: None,
    })
}

/// LLM entry point: memories → prompt → `complete_json::<Vec<SceneExtraction>>`
/// → [`extract_scenes`].
///
/// Generic over `R: LlmRunner` because the trait is not dyn-compatible
/// (`complete_json` is generic). Degrades per Principio 4: a runner failure or
/// invalid JSON returns `success: false, empty_extraction: true` and writes
/// NOTHING (an LLM failure never loses or overwrites stored data).
pub fn extract_scenes_with_llm<R: LlmRunner>(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    runner: &R,
    memories: &[SceneMemoryInput],
    previous_scene_name: Option<&str>,
) -> SceneExtractionResult {
    // No memories → emptyExtraction without a pointless LLM call.
    if memories.is_empty() {
        return SceneExtractionResult {
            success: true,
            empty_extraction: true,
            applied: vec![],
            error: None,
        };
    }

    let prompt = build_scene_extraction_prompt(SceneExtractionPromptParams {
        memories: memories.to_vec(),
        previous_scene_name: previous_scene_name.map(str::to_string),
        mode: crate::core::prompts::l1_extraction::PromptMode::Chat,
    });
    let params = LlmRunParams {
        prompt: prompt.user_prompt,
        system_prompt: Some(prompt.system_prompt),
        task_id: "l2-scene-extraction".into(),
        timeout: None,
        max_tokens: None,
        workspace_dir: None,
        instance_id: None,
    };
    let extractions: Vec<SceneExtraction> = match runner.complete_json(&params) {
        Ok(extractions) => extractions,
        Err(err) => {
            return SceneExtractionResult {
                success: false,
                empty_extraction: true,
                applied: vec![],
                error: Some(format!("LLM scene extraction failed: {err}")),
            };
        }
    };

    // LLM output is untrusted input: extract_scenes normalizes names and
    // revalidates every boundary before writing.
    extract_scenes(db, session_key, &extractions).unwrap_or_else(|err| SceneExtractionResult {
        success: false,
        empty_extraction: true,
        applied: vec![],
        error: Some(format!("scene strategy failed: {err}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extraction(scene_name: &str, content: &str) -> SceneExtraction {
        SceneExtraction {
            scene_name: scene_name.into(),
            summary: "s".into(),
            content: content.into(),
            merge_sources: vec![],
        }
    }

    fn entry(filename: &str, heat: u32) -> SceneIndexEntry {
        SceneIndexEntry {
            filename: filename.into(),
            summary: "s".into(),
            heat,
            created: "2026-08-20T10:00:00.000Z".into(),
            updated: "2026-08-20T10:00:00.000Z".into(),
        }
    }

    #[test]
    fn existing_name_prefers_update() {
        let existing = vec![entry("deploy-runbook", 3)];
        let strategy = decide_strategy(&extraction("deploy-runbook", "content"), &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Update {
                scene_name: "deploy-runbook".into()
            }
        );
    }

    #[test]
    fn normalized_name_matches_existing() {
        let existing = vec![entry("Deploy-Runbook", 1)];
        let strategy = decide_strategy(&extraction("Deploy Runbook!", "content"), &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Update {
                scene_name: "Deploy-Runbook".into()
            },
            "name normalized before matching"
        );
    }

    #[test]
    fn new_name_creates() {
        let existing = vec![entry("other-scene", 1)];
        let strategy = decide_strategy(&extraction("brand-new", "content"), &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Create {
                scene_name: "brand-new".into()
            }
        );
    }

    #[test]
    fn merge_sources_name_live_scenes() {
        let existing = vec![entry("a", 2), entry("b", 4)];
        let mut extraction = extraction("merged", "content");
        extraction.merge_sources = vec!["a".into(), "ghost".into()];
        let strategy = decide_strategy(&extraction, &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Merge {
                scene_name: "merged".into(),
                sources: vec!["a".into()]
            },
            "only live sources survive"
        );
    }

    #[test]
    fn merge_into_existing_target_is_not_update() {
        // Target name exists → UPDATE wins over MERGE (TDAM default).
        let existing = vec![entry("merged", 7), entry("a", 2)];
        let mut extraction = extraction("merged", "content");
        extraction.merge_sources = vec!["a".into()];
        let strategy = decide_strategy(&extraction, &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Update {
                scene_name: "merged".into()
            }
        );
    }

    #[test]
    fn empty_content_skips() {
        let existing = vec![];
        assert_eq!(
            decide_strategy(&extraction("x", "   "), &existing),
            SceneStrategy::Skip
        );
    }

    #[test]
    fn marker_soft_deletes_live_scene() {
        let existing = vec![entry("old-scene", 1)];
        let strategy = decide_strategy(&extraction("old-scene", "[DELETED]"), &existing);
        assert_eq!(
            strategy,
            SceneStrategy::SoftDelete {
                scene_name: "old-scene".into()
            }
        );
    }

    #[test]
    fn marker_for_missing_scene_skips() {
        let existing = vec![];
        let strategy = decide_strategy(&extraction("ghost", "[DELETED]"), &existing);
        assert_eq!(strategy, SceneStrategy::Skip);
    }

    #[test]
    fn merge_sources_rejecting_self_or_empty() {
        let existing = vec![entry("a", 1)];
        let mut extraction = extraction("a", "content");
        extraction.merge_sources = vec!["a".into()];
        // Target exists → Update; self-source never reaches Merge.
        let strategy = decide_strategy(&extraction, &existing);
        assert_eq!(
            strategy,
            SceneStrategy::Update {
                scene_name: "a".into()
            }
        );
    }

    #[test]
    fn extraction_serde_wire_contract() {
        let json = r#"{"scene_name":"deploy","summary":"s","content":"c","merge_sources":["old"]}"#;
        let extraction: SceneExtraction = serde_json::from_str(json).expect("parse");
        assert_eq!(extraction.scene_name, "deploy");
        assert_eq!(extraction.merge_sources, vec!["old"]);
        // merge_sources optional (serde default).
        let json2 = r#"{"scene_name":"deploy","summary":"s","content":"c"}"#;
        let extraction: SceneExtraction = serde_json::from_str(json2).expect("parse no merge");
        assert!(extraction.merge_sources.is_empty());
    }

    #[test]
    fn empty_batch_is_empty_extraction() {
        use vantadb::config::VantaConfig;
        use vantadb::storage::BackendKind;

        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..VantaConfig::default()
        };
        let db = vantadb::sdk::VantaEmbedded::open_with_config(config).expect("open in-memory db");
        let result = extract_scenes(&db, "sess-1", &[]).expect("empty batch");
        assert!(result.empty_extraction);
        assert!(result.applied.is_empty());
        assert_eq!(
            list_scenes(&db, "sess-1").expect("list").len(),
            0,
            "store untouched"
        );
    }
}
