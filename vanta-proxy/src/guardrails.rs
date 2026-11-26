//! Per-key model allowlists (PRX-10 guardrails).
//!
//! Disabled by default so the wire stays a transparent proxy unless
//! explicitly opted in (parity: PRX-07 redact, PRX-13 context). A key
//! with no entry — or an empty entry — may use any model (additive,
//! never a surprise deny). Extension point: a pluggable moderation
//! provider would hook the same gate 1c in `server.rs`.

use std::collections::HashMap;

/// Per-key model allowlist policy. All keys default so a TOML without
/// `[guardrails]` parses unchanged (legacy compat).
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct GuardrailConfig {
    /// When false (default), every check allows and no request is denied.
    pub enabled: bool,
    /// Virtual-key id (D34 `user_id`) → allowed model names. Absent key
    /// or empty vec = all models allowed for that identity.
    pub allowed_models: HashMap<String, Vec<String>>,
}

/// Outcome of one allowlist check.
#[derive(Debug, Clone, PartialEq)]
pub enum GuardrailDecision {
    Allowed,
    Denied { key: String, model: String },
}

impl GuardrailConfig {
    /// Pure allowlist check: disabled config, unknown key, or empty entry
    /// all allow (fail-open defaults — denies only come from explicit
    /// operator policy).
    pub fn check(&self, key: &str, model: &str) -> GuardrailDecision {
        if !self.enabled {
            return GuardrailDecision::Allowed;
        }
        match self.allowed_models.get(key) {
            None => GuardrailDecision::Allowed,
            Some(allowed) if allowed.is_empty() => GuardrailDecision::Allowed,
            Some(allowed) if allowed.iter().any(|m| m == model) => GuardrailDecision::Allowed,
            Some(_) => GuardrailDecision::Denied {
                key: key.to_string(),
                model: model.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_entry_means_no_policy() {
        let cfg = GuardrailConfig {
            enabled: true,
            allowed_models: [("k".to_string(), vec![])].into(),
        };
        assert_eq!(cfg.check("k", "anything"), GuardrailDecision::Allowed);
    }
}
