//! Sandboxed scene tools — read/write/edit over the per-session scene store
//! (MEM-13, F4).
//!
//! This is the TOOL layer the L2 LLM strategy (MEM-14) operates through. The
//! TDAM original runs an LLM agent whose tools are sandboxed to the
//! `scene_blocks/` directory: system files (checkpoint, scene index, persona)
//! are physically invisible to it and it has no `exec` tool
//! (`scene-extractor.ts:7-9`, `:300`). The sandbox translates to a record
//! store as:
//!
//! - **Session confinement** — no tool accepts a namespace. Every operation
//!   derives `scene/<session>` through [`scene_index`] (the namespace is as
//!   invisible to the caller as TDAM's system files).
//! - **No destructive tool** — there is no delete; the L2 soft-delete
//!   strategy belongs to MEM-14.
//! - **Boundary validation** — the caller (an LLM, or any host) is untrusted
//!   input (OWASP LLM05): scene names are checked for emptiness/NUL/size and
//!   content is capped, mirroring the TDAM write-tool parameter validation
//!   that rejects empty/whitespace-only content (`scene-extractor.ts:302`).
//! - **No exec analog** — tools are pure store operations.
//!
//! There is no agent loop here and no UPDATE>MERGE>CREATE strategy decision:
//! this layer exposes the primitives; MEM-14 decides, MEM-16 orchestrates.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` + TDAM
//! `scene-extractor.ts` (604).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::scene::scene_format::SceneBlock;
use crate::core::scene::scene_index::{get_scene, upsert_scene, SceneError};

/// Maximum bytes of a scene name (VantaDB key limit, l0_recorder pattern).
pub const MAX_SCENE_NAME_BYTES: usize = 512;
/// Maximum bytes of a scene summary.
pub const MAX_SUMMARY_BYTES: usize = 4096;
/// Maximum bytes of a scene content payload (1 MiB per block).
pub const MAX_CONTENT_BYTES: usize = 1_048_576;

/// A sandboxed scene tool call, as issued by the L2 caller.
///
/// Serde internally-tagged on `tool` so an LLM tool call arrives as
/// `{"tool": "read", "scene_name": "..."}`, `{"tool": "write", ...}` or
/// `{"tool": "edit", "summary": "...", "content": "..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum SceneToolCall {
    /// Read a scene block by name (missing → `SceneToolResult::Read(None)`).
    Read { scene_name: String },
    /// Create or fully replace a scene block (CREATE/UPDATE semantics of
    /// [`upsert_scene`]: heat bump, `created` preserved on update).
    Write {
        scene_name: String,
        summary: String,
        content: String,
    },
    /// Patch one or both fields of an existing scene block. Errors with
    /// [`SceneToolError::NotFound`] if the scene does not exist and
    /// [`SceneToolError::Invalid`] if neither field is provided.
    Edit {
        scene_name: String,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        content: Option<String>,
    },
}

/// Typed result of a sandboxed scene tool call.
///
/// Serde internally-tagged on `result` so it can be serialized back to the
/// caller (e.g. fed to the LLM as tool output).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum SceneToolResult {
    /// Result of [`SceneToolCall::Read`].
    Read {
        /// The block, or `None` when the scene does not exist.
        scene: Option<SceneBlock>,
    },
    /// Result of [`SceneToolCall::Write`].
    Write { scene: SceneBlock },
    /// Result of [`SceneToolCall::Edit`].
    Edit { scene: SceneBlock },
}

/// Errors surfaced by the sandboxed scene tools.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SceneToolError {
    /// Underlying scene index / storage error.
    #[error("scene tool: {0}")]
    Scene(#[from] SceneError),
    /// Input rejected at the sandbox boundary (empty name, NUL, oversize,
    /// empty/whitespace-only content, edit without fields).
    #[error("invalid scene tool input: {0}")]
    Invalid(String),
    /// [`SceneToolCall::Edit`] targeted a scene that does not exist.
    #[error("scene not found: {0}")]
    NotFound(String),
}

/// Read a scene block by name through the sandbox (session-confined).
pub fn read_scene_tool(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
) -> Result<Option<SceneBlock>, SceneToolError> {
    validate_scene_name(scene_name)?;
    Ok(get_scene(db, session_key, scene_name)?)
}

/// Create or fully replace a scene block through the sandbox.
///
/// Delegates to [`upsert_scene`] (CREATE: heat 1; UPDATE: heat +1, `created`
/// preserved) after boundary validation.
pub fn write_scene_tool(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
    summary: &str,
    content: &str,
) -> Result<SceneBlock, SceneToolError> {
    validate_scene_name(scene_name)?;
    validate_text("summary", summary, MAX_SUMMARY_BYTES)?;
    validate_content(content)?;
    Ok(upsert_scene(db, session_key, scene_name, summary, content)?)
}

/// Patch `summary` and/or `content` of an existing scene block.
///
/// Fetches the current block, merges the provided fields, then writes back
/// through [`upsert_scene`] (heat bumps, `created` preserved). Errors with
/// [`SceneToolError::NotFound`] when the scene is missing.
pub fn edit_scene_tool(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    scene_name: &str,
    summary: Option<&str>,
    content: Option<&str>,
) -> Result<SceneBlock, SceneToolError> {
    validate_scene_name(scene_name)?;
    if summary.is_none() && content.is_none() {
        return Err(SceneToolError::Invalid(
            "edit requires at least one of summary or content".into(),
        ));
    }
    if let Some(s) = summary {
        validate_text("summary", s, MAX_SUMMARY_BYTES)?;
    }
    if let Some(c) = content {
        validate_content(c)?;
    }

    let existing = get_scene(db, session_key, scene_name)?
        .ok_or_else(|| SceneToolError::NotFound(scene_name.to_string()))?;
    let merged_summary = summary.unwrap_or(&existing.meta.summary);
    let merged_content = content.unwrap_or(&existing.content);
    Ok(upsert_scene(
        db,
        session_key,
        scene_name,
        merged_summary,
        merged_content,
    )?)
}

/// Dispatch a sandboxed tool call against the session scene store.
pub fn execute_scene_tool(
    db: &vantadb::sdk::VantaEmbedded,
    session_key: &str,
    call: &SceneToolCall,
) -> Result<SceneToolResult, SceneToolError> {
    match call {
        SceneToolCall::Read { scene_name } => Ok(SceneToolResult::Read {
            scene: read_scene_tool(db, session_key, scene_name)?,
        }),
        SceneToolCall::Write {
            scene_name,
            summary,
            content,
        } => Ok(SceneToolResult::Write {
            scene: write_scene_tool(db, session_key, scene_name, summary, content)?,
        }),
        SceneToolCall::Edit {
            scene_name,
            summary,
            content,
        } => Ok(SceneToolResult::Edit {
            scene: edit_scene_tool(
                db,
                session_key,
                scene_name,
                summary.as_deref(),
                content.as_deref(),
            )?,
        }),
    }
}

// ── boundary validation (LLM output / host input is untrusted) ──
//
// `pub(crate)`: reused by the L2 strategy (MEM-14) for the MERGE branch,
// which bypasses the tools (heat = sum + 1 cannot go through `upsert_scene`).

pub(crate) fn validate_scene_name(name: &str) -> Result<(), SceneToolError> {
    if name.is_empty() {
        return Err(SceneToolError::Invalid(
            "scene_name must not be empty".into(),
        ));
    }
    if name.len() > MAX_SCENE_NAME_BYTES {
        return Err(SceneToolError::Invalid(format!(
            "scene_name exceeds {MAX_SCENE_NAME_BYTES} bytes"
        )));
    }
    if name.contains('\0') {
        return Err(SceneToolError::Invalid(
            "scene_name must not contain NUL".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_text(
    field: &str,
    value: &str,
    max_bytes: usize,
) -> Result<(), SceneToolError> {
    if value.len() > max_bytes {
        return Err(SceneToolError::Invalid(format!(
            "{field} exceeds {max_bytes} bytes"
        )));
    }
    Ok(())
}

/// Rejects empty/whitespace-only content, mirroring the TDAM write-tool
/// validation that prevents the LLM from "deleting" a file by writing
/// whitespace (`scene-extractor.ts:302`).
pub(crate) fn validate_content(content: &str) -> Result<(), SceneToolError> {
    if content.trim().is_empty() {
        return Err(SceneToolError::Invalid(
            "content must not be empty or whitespace-only".into(),
        ));
    }
    validate_text("content", content, MAX_CONTENT_BYTES)
}
