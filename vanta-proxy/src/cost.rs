//! Cost tracking + virtual keys (PRX-03).
//!
//! Request-side token accounting with a configurable price table, plus a
//! [`CostTracker`] ledger keyed by (virtual key × session × model) with
//! log-first budget enforcement (plan pre-mortem: enforcement never blocks
//! legitimate traffic by default).
//!
//! Token counting reuses the canonical heuristic
//! ([`crate::handlers::auxiliary::estimate_tokens`], ~4 chars/token).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::handlers::auxiliary::estimate_tokens;

/// Token usage of one turn (input estimated locally, output parsed from
/// upstream `usage` when the body was buffered).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// USD per 1K tokens, input × output.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelPrice {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

impl Default for ModelPrice {
    fn default() -> Self {
        Self {
            input_per_1k: 0.002,
            output_per_1k: 0.008,
        }
    }
}

/// Per-model price table with a `__default__` fallback for unknown models
/// (pre-mortem: prices go stale → configurable + documented fallback).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PriceTable {
    pub models: HashMap<String, ModelPrice>,
    pub default: ModelPrice,
}

impl Default for PriceTable {
    fn default() -> Self {
        let models = [
            ("gpt-4o", 0.0025, 0.010),
            ("gpt-4o-mini", 0.00015, 0.0006),
            ("claude-3-5-sonnet", 0.003, 0.015),
            ("claude-3-haiku", 0.00025, 0.00125),
        ]
        .into_iter()
        .map(|(name, input_per_1k, output_per_1k)| {
            (
                name.to_string(),
                ModelPrice {
                    input_per_1k,
                    output_per_1k,
                },
            )
        })
        .collect();
        Self {
            models,
            default: ModelPrice::default(),
        }
    }
}

impl PriceTable {
    /// USD cost of one usage record for `model` (fallback: default price).
    /// Pure and total.
    pub fn cost_usd(&self, model: &str, usage: &Usage) -> f64 {
        let price = self.models.get(model).unwrap_or(&self.default);
        usage.input_tokens as f64 / 1000.0 * price.input_per_1k
            + usage.output_tokens as f64 / 1000.0 * price.output_per_1k
    }
}

// ponytail: heuristic ceil(len/4) over raw byte length, no tokenizer dep.
// Ceiling: ±30% vs real tokenizers — fine for budget guardrails, not billing.
pub fn estimate_text_tokens(len: usize) -> u64 {
    len.div_ceil(4) as u64
}

/// Request-side token estimate: canonical `estimate_tokens` over the parsed
/// body; 0 when the body is not JSON (nothing accountable to count).
pub fn tokens_from_request_body(body: &[u8]) -> u64 {
    serde_json::from_slice::<serde_json::Value>(body)
        .map(|value| estimate_tokens(&value))
        .unwrap_or(0)
}

/// Upstream `usage` from a buffered response body: OpenAI
/// (`prompt_tokens`/`completion_tokens`) or Anthropic
/// (`input_tokens`/`output_tokens`). `None` when absent/unparseable
/// (streaming bodies are never buffered — caller records input side only).
pub fn usage_from_response_body(body: &[u8]) -> Option<Usage> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let usage = value.get("usage")?;
    let input = usage
        .get("prompt_tokens")
        .or_else(|| usage.get("input_tokens"))
        .and_then(serde_json::Value::as_u64)?;
    let output = usage
        .get("completion_tokens")
        .or_else(|| usage.get("output_tokens"))
        .and_then(serde_json::Value::as_u64)?;
    Some(Usage {
        input_tokens: input,
        output_tokens: output,
    })
}

/// A virtual key: one accountable identity (PRX-03 Spec #1: the D34
/// `user_id` from [`crate::auth`]). `budget_usd=None` means untracked spend
/// (log-first default — never blocks). Extensible toward PRX-10 allowlists.
#[derive(Debug, Clone)]
pub struct VirtualKey {
    pub id: String,
    pub budget_usd: Option<f64>,
    pub enforce: bool,
}

/// Outcome of one budget check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetDecision {
    Allowed,
    Limited { spent_usd: f64, budget_usd: f64 },
}

/// Hard cap on tracked ledger keys (parity: `rate_limit::MAX_BUCKETS`).
pub const MAX_LEDGER_KEYS: usize = 10_000;

/// In-memory spend ledger keyed by (virtual key × session × model).
/// Single-instance by design (D37 parity with [`crate::rate_limit`]).
pub struct CostTracker {
    prices: PriceTable,
    default_budget_usd: Option<f64>,
    default_enforce: bool,
    ledger: std::sync::Mutex<HashMap<(String, String, String), (Usage, f64)>>,
}

impl CostTracker {
    pub fn new(prices: PriceTable, default_budget_usd: Option<f64>, default_enforce: bool) -> Self {
        Self {
            prices,
            default_budget_usd,
            default_enforce,
            ledger: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Record one turn. Returns the turn's USD cost. Fail-open on a poisoned
    /// mutex (parity: rate limiter) — accounting must never break the wire.
    pub fn record(&self, key: &str, session: &str, model: &str, usage: &Usage) -> f64 {
        let cost = self.prices.cost_usd(model, usage);
        let Ok(mut ledger) = self.ledger.lock() else {
            tracing::warn!("cost ledger poisoned — turn not recorded (fail-open)");
            return cost;
        };
        if ledger.len() >= MAX_LEDGER_KEYS
            && !ledger.contains_key(&(key.to_string(), session.to_string(), model.to_string()))
        {
            tracing::warn!("cost ledger full — turn not recorded (fail-open)");
            return cost;
        }
        let entry = ledger
            .entry((key.to_string(), session.to_string(), model.to_string()))
            .or_insert((Usage::default(), 0.0));
        entry.0.input_tokens += usage.input_tokens;
        entry.0.output_tokens += usage.output_tokens;
        entry.1 += cost;
        cost
    }

    /// Convenience for the wire path: estimate request tokens from the raw
    /// body and record them (output side unknown until buffered — see
    /// [`CostTracker::record_response_usage`]).
    pub fn record_request(&self, key: &str, session: &str, model: &str, body: &[u8]) -> f64 {
        self.record(
            key,
            session,
            model,
            &Usage {
                input_tokens: tokens_from_request_body(body),
                output_tokens: 0,
            },
        )
    }

    /// Add response-side usage once a buffered body is available (tool-loop /
    /// SSE drain wiring calls this; streaming passthrough never buffers).
    /// Returns the added USD cost, or 0.0 when the body carries no `usage`.
    pub fn record_response_usage(&self, key: &str, session: &str, model: &str, body: &[u8]) -> f64 {
        let Some(usage) = usage_from_response_body(body) else {
            return 0.0;
        };
        self.record(
            key,
            session,
            model,
            &Usage {
                input_tokens: 0,
                output_tokens: usage.output_tokens,
            },
        )
    }

    /// Budget gate for one virtual key. No budget → always allowed (log-first
    /// default). Over budget + enforce → [`BudgetDecision::Limited`]; over
    /// budget without enforce → warn + allowed (pre-mortem anti-bloqueo).
    pub fn check_budget(&self, key: &VirtualKey) -> BudgetDecision {
        let Some(budget) = key.budget_usd.or(self.default_budget_usd) else {
            return BudgetDecision::Allowed;
        };
        let spent = self.spent_by_key(&key.id);
        if spent < budget {
            return BudgetDecision::Allowed;
        }
        let enforce = key.enforce || self.default_enforce;
        if enforce {
            BudgetDecision::Limited {
                spent_usd: spent,
                budget_usd: budget,
            }
        } else {
            tracing::warn!(
                key = %key.id, spent_usd = spent, budget_usd = budget,
                "virtual key over budget — allowing (log-first, enforce=false)"
            );
            BudgetDecision::Allowed
        }
    }

    /// Total USD spent by one virtual key across sessions × models.
    pub fn spent_by_key(&self, key: &str) -> f64 {
        self.ledger
            .lock()
            .map(|ledger| {
                ledger
                    .iter()
                    .filter(|((k, _, _), _)| k == key)
                    .map(|(_, (_, cost))| *cost)
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Total USD spent in one session across keys × models.
    pub fn spent_by_session(&self, session: &str) -> f64 {
        self.ledger
            .lock()
            .map(|ledger| {
                ledger
                    .iter()
                    .filter(|((_, s, _), _)| s == session)
                    .map(|(_, (_, cost))| *cost)
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Total USD spent on one model across keys × sessions.
    pub fn spent_by_model(&self, model: &str) -> f64 {
        self.ledger
            .lock()
            .map(|ledger| {
                ledger
                    .iter()
                    .filter(|((_, _, m), _)| m == model)
                    .map(|(_, (_, cost))| *cost)
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Totals for `/snapshot`: entries held + USD across the whole ledger.
    /// Fail-open on a poisoned mutex (zeros, never an error).
    pub fn snapshot(&self) -> CostSnapshot {
        self.ledger
            .lock()
            .map(|ledger| CostSnapshot {
                entries: ledger.len(),
                total_usd: ledger.values().map(|(_, cost)| *cost).sum(),
            })
            .unwrap_or(CostSnapshot {
                entries: 0,
                total_usd: 0.0,
            })
    }
}

/// Whole-ledger totals served by `/snapshot` (PRX-08 shape, additive).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CostSnapshot {
    pub entries: usize,
    pub total_usd: f64,
}

#[cfg(test)]
mod tracker_tests {
    use super::*;

    fn tracker() -> CostTracker {
        CostTracker::new(PriceTable::default(), None, false)
    }

    #[test]
    fn accounting_splits_by_key_session_model() {
        let t = tracker();
        let body_a = br#"{"model":"gpt-4o","messages":[{"role":"user","content":"hello"}]}"#;
        let body_b = br#"{"model":"gpt-4o","messages":[{"role":"user","content":"world!"}]}"#;
        t.record_request("key-1", "sess-1", "gpt-4o", body_a);
        t.record_request("key-1", "sess-2", "gpt-4o", body_b);
        t.record_request("key-2", "sess-1", "gpt-4o-mini", body_a);
        assert!(t.spent_by_key("key-1") > 0.0);
        assert!(t.spent_by_key("key-2") > 0.0);
        assert!(t.spent_by_session("sess-1") > 0.0);
        assert!(t.spent_by_session("sess-2") > 0.0);
        assert!(t.spent_by_model("gpt-4o") > t.spent_by_model("gpt-4o-mini"));
        assert_eq!(t.spent_by_key("nobody"), 0.0);
    }

    #[test]
    fn no_budget_is_always_allowed() {
        let t = tracker();
        let key = VirtualKey {
            id: "free".into(),
            budget_usd: None,
            enforce: true,
        };
        assert_eq!(t.check_budget(&key), BudgetDecision::Allowed);
    }

    #[test]
    fn over_budget_without_enforce_allows_with_warning() {
        // Log-first default (pre-mortem): over budget, enforce=false → allow.
        let t = tracker();
        t.record(
            "key-1",
            "s",
            "gpt-4o",
            &Usage {
                input_tokens: 1_000_000,
                output_tokens: 0,
            },
        );
        let key = VirtualKey {
            id: "key-1".into(),
            budget_usd: Some(0.000_001),
            enforce: false,
        };
        assert_eq!(t.check_budget(&key), BudgetDecision::Allowed);
    }

    #[test]
    fn over_budget_with_enforce_limits_429() {
        let t = tracker();
        t.record(
            "key-1",
            "s",
            "gpt-4o",
            &Usage {
                input_tokens: 1_000_000,
                output_tokens: 0,
            },
        );
        let key = VirtualKey {
            id: "key-1".into(),
            budget_usd: Some(0.000_001),
            enforce: true,
        };
        assert!(matches!(
            t.check_budget(&key),
            BudgetDecision::Limited { .. }
        ));
    }

    #[test]
    fn under_budget_is_allowed_even_with_enforce() {
        let t = tracker();
        let key = VirtualKey {
            id: "key-1".into(),
            budget_usd: Some(100.0),
            enforce: true,
        };
        assert_eq!(t.check_budget(&key), BudgetDecision::Allowed);
    }

    #[test]
    fn response_usage_adds_output_side_only() {
        let t = tracker();
        let added = t.record_response_usage(
            "k",
            "s",
            "gpt-4o",
            br#"{"usage":{"prompt_tokens":50,"completion_tokens":25}}"#,
        );
        assert!(added > 0.0);
        assert_eq!(
            t.record_response_usage("k", "s", "gpt-4o", br#"{"no":"usage"}"#),
            0.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_model_uses_table_price() {
        let table = PriceTable::default();
        let usage = Usage {
            input_tokens: 1_000,
            output_tokens: 500,
        };
        let cost = table.cost_usd("gpt-4o", &usage);
        assert!(cost > 0.0, "known model must price > 0, got {cost}");
    }

    #[test]
    fn unknown_model_falls_back_to_default_price() {
        let table = PriceTable::default();
        let usage = Usage {
            input_tokens: 1_000,
            output_tokens: 0,
        };
        assert_eq!(
            table.cost_usd("model-from-the-future", &usage),
            table.cost_usd("__default__", &usage)
        );
    }

    #[test]
    fn estimate_text_tokens_scales_with_length() {
        assert_eq!(estimate_text_tokens(0), 0);
        assert_eq!(estimate_text_tokens(4), 1);
        assert_eq!(estimate_text_tokens(8), 2);
        assert!(estimate_text_tokens(4000) >= 900);
    }

    #[test]
    fn request_tokens_extracted_from_openai_messages() {
        let body = br#"{"model":"gpt-4o","messages":[{"role":"user","content":"hello world"}]}"#;
        let tokens = tokens_from_request_body(body);
        assert!(tokens > 0, "message text must count");
    }

    #[test]
    fn request_tokens_zero_on_non_json() {
        assert_eq!(tokens_from_request_body(b"not json"), 0);
        assert_eq!(tokens_from_request_body(b""), 0);
    }

    #[test]
    fn response_usage_parses_openai_shape() {
        let body = br#"{"usage":{"prompt_tokens":10,"completion_tokens":20}}"#;
        let usage = usage_from_response_body(body).expect("openai usage");
        assert_eq!(
            usage,
            Usage {
                input_tokens: 10,
                output_tokens: 20
            }
        );
    }

    #[test]
    fn response_usage_parses_anthropic_shape() {
        let body = br#"{"usage":{"input_tokens":7,"output_tokens":3}}"#;
        let usage = usage_from_response_body(body).expect("anthropic usage");
        assert_eq!(
            usage,
            Usage {
                input_tokens: 7,
                output_tokens: 3
            }
        );
    }

    #[test]
    fn response_usage_none_when_absent() {
        assert!(usage_from_response_body(br#"{"ok":true}"#).is_none());
        assert!(usage_from_response_body(b"garbage").is_none());
    }
}
