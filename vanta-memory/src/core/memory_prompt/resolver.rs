//! Resolve which custom memory strategy applies to a target (TDAM
//! `MC/core/memory-prompt/resolver.ts`): candidate chain agent → team →
//! instance, first active match wins.

use crate::core::memory_prompt::types::{
    build_memory_prompt_setting_id, MemoryPromptError, MemoryPromptLayer, MemoryPromptRecord,
    MemoryPromptSettingRecord, MemoryPromptSource, PromptStatus, ResolvedMemoryPrompt,
};

/// Read-side store contract for memory prompts. Implementations persist the
/// records in VantaDB namespaces; tests use in-memory fakes.
pub trait MemoryPromptStore {
    fn get_setting(
        &self,
        setting_id: &str,
    ) -> Result<Option<MemoryPromptSettingRecord>, MemoryPromptError>;
    fn get_prompt(&self, prompt_id: &str) -> Result<Option<MemoryPromptRecord>, MemoryPromptError>;
}

/// Resolution target: who is asking and for which layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveTarget<'a> {
    pub team_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub layer: MemoryPromptLayer,
}

/// Resolve the effective custom strategy for `target`: agent-scoped beats
/// team-scoped beats instance-scoped. Only settings whose target fields match
/// exactly and prompts with [`PromptStatus::Active`] on the same layer count.
pub fn resolve_memory_prompt(
    store: &dyn MemoryPromptStore,
    target: ResolveTarget<'_>,
) -> Result<Option<ResolvedMemoryPrompt>, MemoryPromptError> {
    // Candidate chain (TDAM `candidates()`): agent needs team+agent.
    let mut candidates: Vec<(MemoryPromptSource, Option<&str>, Option<&str>)> = Vec::new();
    if let (Some(team), Some(agent)) = (target.team_id, target.agent_id) {
        candidates.push((MemoryPromptSource::Agent, Some(team), Some(agent)));
    }
    if let Some(team) = target.team_id {
        candidates.push((MemoryPromptSource::Team, Some(team), None));
    }
    candidates.push((MemoryPromptSource::Instance, None, None));

    for (source, team_id, agent_id) in candidates {
        let setting_id = build_memory_prompt_setting_id(team_id, agent_id, target.layer)?;
        let Some(setting) = store.get_setting(&setting_id)? else {
            continue;
        };
        if !setting_matches(&setting, source, team_id, agent_id, target.layer) {
            continue;
        }
        let Some(prompt) = store.get_prompt(&setting.memory_prompt_id)? else {
            continue;
        };
        if prompt.status != PromptStatus::Active || prompt.layer != target.layer {
            continue;
        }
        return Ok(Some(ResolvedMemoryPrompt {
            memory_prompt_id: prompt.memory_prompt_id,
            prompt: prompt.prompt,
            layer: prompt.layer,
            source,
            version: prompt.version,
        }));
    }
    Ok(None)
}

/// Exact-match validation of a setting against its candidate slot (TDAM
/// resolver's field checks).
fn setting_matches(
    setting: &MemoryPromptSettingRecord,
    source: MemoryPromptSource,
    team_id: Option<&str>,
    agent_id: Option<&str>,
    layer: MemoryPromptLayer,
) -> bool {
    setting.layer == layer
        && setting.target_type == source_tag(source)
        && setting.team_id.as_deref().unwrap_or("") == team_id.unwrap_or("")
        && setting.agent_id.as_deref().unwrap_or("") == agent_id.unwrap_or("")
}

fn source_tag(source: MemoryPromptSource) -> &'static str {
    match source {
        MemoryPromptSource::Agent => "agent",
        MemoryPromptSource::Team => "team",
        MemoryPromptSource::Instance => "instance",
        MemoryPromptSource::System => "system",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeStore {
        settings: HashMap<String, MemoryPromptSettingRecord>,
        prompts: HashMap<String, MemoryPromptRecord>,
    }

    impl FakeStore {
        fn with_agent_prompt(layer: MemoryPromptLayer, status: PromptStatus) -> Self {
            let mut store = Self::default();
            store.prompts.insert(
                "mp_1".into(),
                MemoryPromptRecord {
                    memory_prompt_id: "mp_1".into(),
                    name: "test".into(),
                    layer,
                    prompt: "focus on rules".into(),
                    version: 2,
                    status,
                    created_at_ms: 1,
                    updated_at_ms: 1,
                },
            );
            store.settings.insert(
                build_memory_prompt_setting_id(Some("t"), Some("a"), layer).unwrap(),
                MemoryPromptSettingRecord {
                    setting_id: build_memory_prompt_setting_id(Some("t"), Some("a"), layer)
                        .unwrap(),
                    target_type: "agent".into(),
                    team_id: Some("t".into()),
                    agent_id: Some("a".into()),
                    layer,
                    memory_prompt_id: "mp_1".into(),
                    updated_at_ms: 1,
                },
            );
            store
        }
    }

    impl MemoryPromptStore for FakeStore {
        fn get_setting(
            &self,
            setting_id: &str,
        ) -> Result<Option<MemoryPromptSettingRecord>, MemoryPromptError> {
            Ok(self.settings.get(setting_id).cloned())
        }

        fn get_prompt(
            &self,
            prompt_id: &str,
        ) -> Result<Option<MemoryPromptRecord>, MemoryPromptError> {
            Ok(self.prompts.get(prompt_id).cloned())
        }
    }

    fn target<'a>(layer: MemoryPromptLayer) -> ResolveTarget<'a> {
        ResolveTarget {
            team_id: Some("t"),
            agent_id: Some("a"),
            layer,
        }
    }

    #[test]
    fn resolves_active_agent_prompt() {
        let store = FakeStore::with_agent_prompt(MemoryPromptLayer::L1, PromptStatus::Active);
        let out = resolve_memory_prompt(&store, target(MemoryPromptLayer::L1)).unwrap();
        assert!(out.is_some());
        let resolved = out.unwrap();
        assert_eq!(resolved.source, MemoryPromptSource::Agent);
        assert_eq!(resolved.prompt, "focus on rules");
        assert_eq!(resolved.version, 2);
    }

    #[test]
    fn skips_deleting_prompts_and_layer_mismatch() {
        let deleting = FakeStore::with_agent_prompt(MemoryPromptLayer::L1, PromptStatus::Deleting);
        assert!(
            resolve_memory_prompt(&deleting, target(MemoryPromptLayer::L1))
                .unwrap()
                .is_none()
        );
        // Setting bound to l1 but asked for l2 → no match.
        let store = FakeStore::with_agent_prompt(MemoryPromptLayer::L1, PromptStatus::Active);
        assert!(resolve_memory_prompt(&store, target(MemoryPromptLayer::L2))
            .unwrap()
            .is_none());
    }

    #[test]
    fn empty_store_resolves_none() {
        let store = FakeStore::default();
        assert!(resolve_memory_prompt(&store, target(MemoryPromptLayer::L3))
            .unwrap()
            .is_none());
    }
}
