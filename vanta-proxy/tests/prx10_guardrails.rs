//! PRX-10: per-key model allowlists (guardrails) + 403 shape + TOML compat.
//!
//! RED: fails with E0432 until `vanta_proxy::guardrails` exists.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use axum::response::IntoResponse;
use vanta_proxy::config::ProxyConfig;
use vanta_proxy::error::ProxyError;
use vanta_proxy::guardrails::{GuardrailConfig, GuardrailDecision};

fn enabled_cfg() -> GuardrailConfig {
    let mut allowed = std::collections::HashMap::new();
    allowed.insert("key-1".to_string(), vec!["gpt-4o-mini".to_string()]);
    GuardrailConfig {
        enabled: true,
        allowed_models: allowed,
    }
}

#[test]
fn disabled_default_allows_everything() {
    let cfg = GuardrailConfig::default();
    assert!(!cfg.enabled);
    assert_eq!(cfg.check("any-key", "gpt-4o"), GuardrailDecision::Allowed);
}

#[test]
fn allowlist_hit_allows() {
    assert_eq!(
        enabled_cfg().check("key-1", "gpt-4o-mini"),
        GuardrailDecision::Allowed
    );
}

#[test]
fn allowlist_miss_denies_with_identity() {
    assert_eq!(
        enabled_cfg().check("key-1", "gpt-4o"),
        GuardrailDecision::Denied {
            key: "key-1".to_string(),
            model: "gpt-4o".to_string(),
        }
    );
}

#[test]
fn unknown_key_allows_all_models() {
    // Absent key = no policy for that identity (safe additive default).
    assert_eq!(
        enabled_cfg().check("key-9", "gpt-4o"),
        GuardrailDecision::Allowed
    );
}

#[test]
fn empty_allowlist_vec_allows_all_models() {
    let cfg = GuardrailConfig {
        enabled: true,
        allowed_models: [("key-1".to_string(), vec![])].into(),
    };
    assert_eq!(cfg.check("key-1", "gpt-4o"), GuardrailDecision::Allowed);
}

#[test]
fn guardrail_blocked_answers_403_without_secrets() {
    let response = ProxyError::GuardrailBlocked {
        key: "key-1".to_string(),
        model: "gpt-4o".to_string(),
    }
    .into_response();
    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
}

#[test]
fn legacy_toml_without_guardrails_parses_disabled() {
    let cfg: ProxyConfig = toml::from_str(
        "[server]\nhost = \"127.0.0.1\"\nport = 8096\n\
         [upstream]\nurl = \"https://api.anthropic.com\"\napi_key = \"k\"\n",
    )
    .expect("legacy TOML must parse");
    assert!(!cfg.guardrails.enabled);
}

#[test]
fn full_toml_with_allowlist_parses() {
    let cfg: ProxyConfig = toml::from_str(
        "[upstream]\nurl = \"https://api.anthropic.com\"\n\
         [guardrails]\nenabled = true\n\
         [guardrails.allowed_models]\nkey-1 = [\"gpt-4o-mini\"]\n",
    )
    .expect("guardrails TOML must parse");
    assert!(cfg.guardrails.enabled);
    assert_eq!(
        cfg.guardrails.allowed_models.get("key-1"),
        Some(&vec!["gpt-4o-mini".to_string()])
    );
}
