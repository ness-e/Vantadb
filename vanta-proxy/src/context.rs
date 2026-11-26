//! Context optimization in transit (PRX-13).
//!
//! Trims oversized conversation histories to a token budget BEFORE the
//! forward, so the upstream never pays for stale turns. Runs between
//! redaction (5a) and cache lookup (5b): trimmed bytes are what get
//! cached and forwarded.
//!
//! Safety design (pre-mortem): resizing degrades `tool_calls` that reference
//! images, so [`ContextMode::Performance`] (the default) passes bodies with
//! tool activity or image blocks through untouched; [`ContextMode::Balanced`]
//! trims text but keeps images; only [`ContextMode::Economy`] drops image
//! blocks. Trimming never fails the request: unparseable bodies, oversize
//! bodies, and serialization errors all fail open (transparent proxy
//! invariant). [`ApplyOutcome`] has a single `Pass` variant by construction
//! — optimization never blocks.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::handlers::auxiliary::estimate_tokens;
use crate::inject::Protocol;

/// Default token budget for the resize: conservative, operator-tunable.
fn default_max_input_tokens() -> u64 {
    8000
}

/// Default scan cap: bodies larger than this fail open (unchanged + warn).
fn default_max_scan_bytes() -> usize {
    2 * 1024 * 1024
}

/// Placeholder keeping turn structure when Economy drops an image block.
const IMAGE_REMOVED_PLACEHOLDER: &str = "[image removed: context budget]";

/// Aggressiveness of the trim. Default is `Performance`: the safe mode from
/// the pre-mortem (tool/image bodies pass through untouched).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    /// Safe default: skip bodies with tool activity or images; trim
    /// pure-text histories only.
    #[default]
    Performance,
    /// Trim text histories; image blocks are preserved.
    Balanced,
    /// Trim text + drop image blocks (placeholder keeps turn structure).
    Economy,
}

/// Context optimization configuration. Disabled by default so the wire stays
/// a transparent proxy unless explicitly opted in (same invariant as
/// cache/routing/cost-enforce/redact).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    /// Master switch. `false` (default) → apply is the identity function.
    pub enabled: bool,
    /// Global mode. Default `Performance` (safe).
    pub mode: ContextMode,
    /// Histories estimating above this (chars/4 heuristic) are resized.
    #[serde(default = "default_max_input_tokens")]
    pub max_input_tokens: u64,
    /// Per-virtual-key mode override (plan: "performance/balanced/economy
    /// por key"). Wins over the global `mode` for that key.
    pub modes_by_key: HashMap<String, ContextMode>,
    /// Bodies larger than this fail open. Default 2 MiB.
    #[serde(default = "default_max_scan_bytes")]
    pub max_scan_bytes: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: ContextMode::Performance,
            max_input_tokens: default_max_input_tokens(),
            modes_by_key: HashMap::new(),
            max_scan_bytes: default_max_scan_bytes(),
        }
    }
}

/// Outcome of [`ContextOptimizer::apply`]. Single variant by design:
/// optimization trims, never blocks.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Bytes to forward (trimmed, or original when disabled/under
    /// budget/guarded/fail-open).
    Pass(Vec<u8>),
}

/// Compiled optimizer: build once from [`ContextConfig`], reuse per request.
/// Stateless beyond config (no regex, no I/O) — construction is infallible.
pub struct ContextOptimizer {
    enabled: bool,
    mode: ContextMode,
    max_input_tokens: u64,
    modes_by_key: HashMap<String, ContextMode>,
    max_scan_bytes: usize,
}

impl ContextOptimizer {
    /// Build from config. Infallible: no patterns to compile.
    #[must_use]
    pub fn new(cfg: &ContextConfig) -> Self {
        Self {
            enabled: cfg.enabled,
            mode: cfg.mode,
            max_input_tokens: cfg.max_input_tokens,
            modes_by_key: cfg.modes_by_key.clone(),
            max_scan_bytes: cfg.max_scan_bytes,
        }
    }

    /// Effective mode for a virtual key: per-key override wins, else global.
    /// Pure and total.
    #[must_use]
    pub fn mode_for(&self, user_key: &str) -> ContextMode {
        self.modes_by_key
            .get(user_key)
            .copied()
            .unwrap_or(self.mode)
    }

    /// Trim `body` to the token budget. Never blocks: every early return
    /// forwards the original bytes (disabled, oversize, non-JSON, guarded,
    /// under budget, serialization failure).
    #[must_use]
    pub fn apply(&self, body: &[u8], _protocol: Protocol, user_key: &str) -> ApplyOutcome {
        if !self.enabled || body.len() > self.max_scan_bytes {
            return ApplyOutcome::Pass(body.to_vec());
        }
        let Ok(mut value) = serde_json::from_slice::<Value>(body) else {
            return ApplyOutcome::Pass(body.to_vec());
        };
        let mode = self.mode_for(user_key);
        let flags = message_flags(&value);
        match mode {
            ContextMode::Performance if flags.tools || flags.images => {
                return ApplyOutcome::Pass(body.to_vec())
            }
            ContextMode::Balanced if flags.tools => return ApplyOutcome::Pass(body.to_vec()),
            ContextMode::Performance | ContextMode::Balanced | ContextMode::Economy => {}
        }
        let mut changed = false;
        if mode == ContextMode::Economy {
            changed |= drop_image_blocks(&mut value);
        }
        if estimate_tokens(&value) > self.max_input_tokens {
            // Balanced conserva bloques imagen: solo caen turns de texto puro.
            changed |= trim_history(
                &mut value,
                self.max_input_tokens,
                mode == ContextMode::Balanced,
            );
        }
        if !changed {
            return ApplyOutcome::Pass(body.to_vec());
        }
        match serde_json::to_vec(&value) {
            Ok(out) => {
                tracing::info!(
                    mode = ?mode,
                    from_bytes = body.len(),
                    to_bytes = out.len(),
                    "context trimmed to budget"
                );
                ApplyOutcome::Pass(out)
            }
            Err(_) => ApplyOutcome::Pass(body.to_vec()),
        }
    }
}

/// Tool/image activity inside the conversation turns (not top-level `tools`:
/// injection (D29) always adds its own L0/L1 specs post-step-5, so the array
/// alone says nothing about risky content).
#[derive(Debug, Default)]
struct BodyFlags {
    tools: bool,
    images: bool,
}

/// Scan `messages` (OpenAI/Anthropic) or `input` (Responses) turns for tool
/// activity and image blocks. Conservative: unknown shapes simply report
/// no flags (trim proceeds, fail-open elsewhere covers the rest).
fn message_flags(value: &Value) -> BodyFlags {
    let mut flags = BodyFlags::default();
    for key in ["messages", "input"] {
        if let Some(turns) = value.get(key).and_then(Value::as_array) {
            for turn in turns {
                scan_turn(turn, &mut flags);
            }
        }
    }
    flags
}

fn scan_turn(turn: &Value, flags: &mut BodyFlags) {
    if turn
        .get("tool_calls")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        flags.tools = true;
    }
    if turn.get("role").and_then(Value::as_str) == Some("tool") {
        flags.tools = true;
    }
    if let Some(kind) = turn.get("type").and_then(Value::as_str) {
        if matches!(
            kind,
            "function_call" | "function_call_output" | "tool_use" | "tool_result"
        ) {
            flags.tools = true;
        }
    }
    if let Some(parts) = turn.get("content").and_then(Value::as_array) {
        for part in parts {
            match part.get("type").and_then(Value::as_str) {
                Some("image_url") | Some("image") => flags.images = true,
                Some("tool_use") | Some("tool_result") => flags.tools = true,
                _ => {}
            }
        }
    }
}

/// Mutable turn list: `messages` (OpenAI/Anthropic) or array `input`
/// (Responses). `None` when absent or not an array (passthrough).
fn turns_mut(value: &mut Value) -> Option<&mut Vec<Value>> {
    for key in ["messages", "input"] {
        if value.get(key).is_some_and(|v| v.is_array()) {
            return value.get_mut(key).and_then(Value::as_array_mut);
        }
    }
    None
}

/// Drop image blocks (OpenAI `image_url` parts, Anthropic `image` blocks),
/// leaving a placeholder so turn structure survives. Returns true when at
/// least one block was removed.
fn drop_image_blocks(value: &mut Value) -> bool {
    let Some(turns) = turns_mut(value) else {
        return false;
    };
    let mut removed = false;
    for turn in turns.iter_mut() {
        let mut emptied = false;
        if let Some(parts) = turn.get_mut("content").and_then(Value::as_array_mut) {
            let before = parts.len();
            parts.retain(|p| {
                !matches!(
                    p.get("type").and_then(Value::as_str),
                    Some("image_url") | Some("image")
                )
            });
            if parts.len() < before {
                removed = true;
                emptied = parts.is_empty();
            }
        }
        if emptied {
            turn["content"] = Value::String(IMAGE_REMOVED_PLACEHOLDER.to_string());
        }
    }
    removed
}

/// True cuando el turn contiene un bloque imagen (OpenAI `image_url`,
/// Anthropic `image`).
fn turn_has_image(turn: &Value) -> bool {
    turn.get("content")
        .and_then(Value::as_array)
        .is_some_and(|parts| {
            parts.iter().any(|p| {
                matches!(
                    p.get("type").and_then(Value::as_str),
                    Some("image_url") | Some("image")
                )
            })
        })
}

/// Resize the history to `budget`: drop oldest turns first, always keeping
/// the first (system-ish) plus the last two (current turn pair). With
/// `keep_images` (Balanced), turns carrying image blocks are never removal
/// candidates. Stops when under budget, only 3 turns remain, or (Balanced)
/// nothing but image turns is left to drop. Returns true when at least one
/// turn was dropped.
fn trim_history(value: &mut Value, budget: u64, keep_images: bool) -> bool {
    let mut changed = false;
    loop {
        if estimate_tokens(value) <= budget {
            break;
        }
        let victim = turns_mut(value).and_then(|turns| {
            let len = turns.len();
            if len <= 3 {
                return None;
            }
            (1..len - 2).find(|&i| !keep_images || !turn_has_image(&turns[i]))
        });
        match victim {
            Some(i) => {
                if let Some(turns) = turns_mut(value) {
                    turns.remove(i);
                    changed = true;
                } else {
                    break;
                }
            }
            None => break,
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_transparent() {
        let cfg = ContextConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.mode, ContextMode::Performance);
        assert_eq!(cfg.max_input_tokens, 8000);
        assert!(cfg.modes_by_key.is_empty());
    }

    #[test]
    fn per_key_override_falls_back_to_global() {
        let cfg = ContextConfig {
            enabled: true,
            mode: ContextMode::Balanced,
            modes_by_key: HashMap::from([("usr-1".to_string(), ContextMode::Economy)]),
            ..ContextConfig::default()
        };
        let o = ContextOptimizer::new(&cfg);
        assert_eq!(o.mode_for("usr-1"), ContextMode::Economy);
        assert_eq!(o.mode_for("unknown"), ContextMode::Balanced);
    }

    #[test]
    fn top_level_tools_alone_do_not_flag() {
        // Injection (D29) always adds its own specs — the array alone must
        // not force the Performance guard (else the hook would be a no-op).
        let v: Value = serde_json::from_str(
            r#"{"messages": [{"role": "user", "content": "hi"}],
                "tools": [{"type": "function", "function": {"name": "vanta_memory_search"}}]}"#,
        )
        .expect("fixture parses");
        let flags = message_flags(&v);
        assert!(!flags.tools && !flags.images);
    }
}
