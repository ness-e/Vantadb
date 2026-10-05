//! VER-04: injection governance — namespace ACL policy + injection-audit
//! emission shared by the proxy's memory surfaces (the `<vanta-memory>`
//! system-prompt block and the LLM-facing `mem:search` tool).
//!
//! Both knobs are opt-in: an empty policy allows every namespace and an unset
//! audit path is a no-op, so the defaults keep the wire behavior
//! byte-identical. Audit events are metadata-only (`ns`/`key`/`score`/
//! `budget`/`truncated`/`acl`) — never payload values.

use std::sync::Arc;

use vanta_memory::core::hooks::{InjectionPolicy, RecallResult};

use crate::inject::InjectionBlock;

/// Open the injection-audit JSONL when `path` is configured. A failed open is
/// logged and treated as disabled — audit never blocks the proxy (same policy
/// as the core `init_audit`).
pub fn open_audit(path: &str) -> Option<Arc<vantadb::audit::AuditLogger>> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    match vantadb::audit::AuditLogger::with_rotation(
        path,
        crate::config::DEFAULT_INJECTION_AUDIT_MAX_BYTES,
        crate::config::DEFAULT_INJECTION_AUDIT_MAX_FILES,
    ) {
        Ok(logger) => Some(Arc::new(logger)),
        Err(e) => {
            tracing::warn!(
                path,
                error = %e,
                "injection audit disabled: cannot open audit_log_path"
            );
            None
        }
    }
}

/// Per-process governance context (ACL policy + optional audit sink).
#[derive(Clone, Debug)]
pub struct Governance {
    policy: InjectionPolicy,
    audit: Option<Arc<vantadb::audit::AuditLogger>>,
}

impl Governance {
    /// Build from an explicit policy and audit sink.
    pub fn new(policy: InjectionPolicy, audit: Option<Arc<vantadb::audit::AuditLogger>>) -> Self {
        Self { policy, audit }
    }

    /// Build from raw config values (empty prefixes = allow-all, empty
    /// tainted list = every namespace trusted, `include_tainted=false`, empty
    /// path = audit disabled).
    pub fn from_config(
        prefixes: &[String],
        tainted_namespaces: &[String],
        include_tainted: bool,
        audit_log_path: &str,
    ) -> Self {
        Self::new(
            InjectionPolicy::from_parts(
                prefixes.iter().cloned(),
                tainted_namespaces.iter().cloned(),
                include_tainted,
            ),
            open_audit(audit_log_path),
        )
    }

    /// The ACL policy applied to every injection read.
    pub fn policy(&self) -> &InjectionPolicy {
        &self.policy
    }

    /// True when an audit sink is configured.
    pub fn audit_enabled(&self) -> bool {
        self.audit.is_some()
    }

    fn record(&self, event: vantadb::audit::AuditEvent) {
        let Some(logger) = &self.audit else {
            return;
        };
        if let Err(e) = logger.record(&event) {
            tracing::warn!(op = %event.op, error = %e, "injection audit record failed");
        }
    }

    fn deny_reason(&self, surface: &str, tool: &str, session: &str, budget: &str) -> String {
        format!("surface={surface};tool={tool};session={session};budget={budget};acl=deny")
    }

    /// One event per injected source plus one per ACL denial (never silent).
    pub fn audit_block(&self, session: &str, block: &InjectionBlock) {
        if self.audit.is_none() {
            return;
        }
        let budget = format!("{}/{}", block.used_tokens, block.budget_tokens);
        for source in &block.sources {
            let reason = format!(
                "surface=proxy;tool=prompt_block;session={session};kind={};budget={budget};truncated={};acl=allow",
                source.kind, block.truncated
            );
            self.record(vantadb::audit::AuditEvent::injection(
                &source.namespace,
                &source.key,
                "ok",
                Some(reason),
            ));
        }
        for namespace in &block.denied {
            let reason = format!(
                "{};kind=prompt_block;truncated={}",
                self.deny_reason("proxy", "prompt_block", session, &budget),
                block.truncated
            );
            self.record(vantadb::audit::AuditEvent::injection(
                namespace,
                "N/A",
                "denied",
                Some(reason),
            ));
        }
    }

    /// One event per recalled memory (score included) plus one per ACL denial.
    /// `budget_chars` is the recall char cap applied (0 = unbounded).
    pub fn audit_recall(
        &self,
        tool: &str,
        session: &str,
        result: &RecallResult,
        budget_chars: u64,
    ) {
        if self.audit.is_none() {
            return;
        }
        let budget = if budget_chars == 0 {
            "unbounded".to_string()
        } else {
            budget_chars.to_string()
        };
        for memory in &result.recalled_memories {
            let namespace = if memory.source_namespace.is_empty() {
                "l1/unknown"
            } else {
                memory.source_namespace.as_str()
            };
            let key = if memory.source_key.is_empty() {
                "N/A"
            } else {
                memory.source_key.as_str()
            };
            let reason = format!(
                "surface=proxy;tool={tool};session={session};kind=l1;score={};budget={budget};acl=allow",
                memory.score
            );
            self.record(vantadb::audit::AuditEvent::injection(
                namespace,
                key,
                "ok",
                Some(reason),
            ));
        }
        for namespace in &result.governance.denied {
            let reason = format!(
                "{};kind=recall",
                self.deny_reason("proxy", tool, session, &budget)
            );
            self.record(vantadb::audit::AuditEvent::injection(
                namespace,
                "N/A",
                "denied",
                Some(reason),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inject::InjectedSource;
    use vanta_memory::core::hooks::{RecallGovernance, RecalledMemory};

    fn reading(path: &std::path::Path) -> Vec<serde_json::Value> {
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).expect("jsonl line"))
            .collect()
    }

    #[test]
    fn injection_block_audit_records_sources_and_denials_metadata_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("audit.jsonl");
        let governance = Governance::new(
            InjectionPolicy::allow_all(),
            open_audit(path.to_string_lossy().as_ref()),
        );
        assert!(governance.audit_enabled());

        let block = InjectionBlock {
            block: "<vanta-memory>secret payload</vanta-memory>".into(),
            sources: vec![InjectedSource {
                namespace: "persona/sess-1".into(),
                key: "persona.md".into(),
                kind: "persona",
            }],
            denied: vec!["scene/sess-1".into()],
            budget_tokens: 2000,
            used_tokens: 42,
            truncated: false,
        };
        governance.audit_block("sess-1", &block);

        let rows = reading(&path);
        assert_eq!(rows.len(), 2, "one source event + one denial event");
        assert_eq!(rows[0]["op"], "injection");
        assert_eq!(rows[0]["namespace"], "persona/sess-1");
        assert_eq!(rows[0]["key"], "persona.md");
        assert_eq!(rows[0]["outcome"], "ok");
        let reason = rows[0]["reason"].as_str().unwrap_or_default();
        assert!(reason.contains("budget=42/2000"), "reason: {reason}");
        assert!(reason.contains("acl=allow"));

        assert_eq!(rows[1]["outcome"], "denied");
        assert_eq!(rows[1]["namespace"], "scene/sess-1");
        assert!(rows[1]["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("acl=deny"));

        // Metadata only: the block payload never reaches the audit file.
        let raw = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(!raw.contains("secret payload"), "payload leaked: {raw}");
    }

    #[test]
    fn recall_audit_carries_score_and_source_identity() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("audit.jsonl");
        let governance = Governance::new(
            InjectionPolicy::allow_all(),
            open_audit(path.to_string_lossy().as_ref()),
        );

        let result = RecallResult {
            prepend_context: None,
            append_system_context: None,
            recalled_memories: vec![RecalledMemory {
                content: "user prefers dark mode".into(),
                score: 3,
                memory_type: "episodic".into(),
                source_namespace: "l1/sess-1".into(),
                source_key: "m1".into(),
            }],
            persona: None,
            effective_mode: vanta_memory::core::hooks::RecallMode::Keyword,
            governance: RecallGovernance {
                sources: vec!["l1/sess-1".into()],
                denied: vec!["l1/sess-2".into()],
            },
        };
        governance.audit_recall("mem:search", "sess-1", &result, 8000);

        let rows = reading(&path);
        assert_eq!(rows.len(), 2, "one hit event + one denial event");
        assert_eq!(rows[0]["namespace"], "l1/sess-1");
        assert_eq!(rows[0]["key"], "m1");
        assert!(rows[0]["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("score=3"));
        assert_eq!(rows[1]["outcome"], "denied");
        assert_eq!(rows[1]["namespace"], "l1/sess-2");
    }

    #[test]
    fn disabled_audit_is_a_noop() {
        let governance = Governance::from_config(&[], &[], false, "");
        assert!(!governance.audit_enabled());
        let block = InjectionBlock {
            block: "x".into(),
            sources: vec![],
            denied: vec!["l1/nope".into()],
            budget_tokens: 10,
            used_tokens: 1,
            truncated: false,
        };
        // No sink, no file, no panic.
        governance.audit_block("sess", &block);
    }

    #[test]
    fn tainted_namespaces_deny_injection_and_include_tainted_opts_in() {
        let governance = Governance::from_config(&[], &["l1/evil".into()], false, "");
        assert!(!governance.policy().allows("l1/evil"));
        assert!(governance.policy().allows("l1/ok"));

        let opted_in = Governance::from_config(&[], &["l1/evil".into()], true, "");
        assert!(opted_in.policy().allows("l1/evil"));
    }
}
