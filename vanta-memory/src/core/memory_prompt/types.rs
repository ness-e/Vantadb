//! Data contracts for the memory-prompt layer (custom strategy prompts that
//! tune how the agent treats L1/L2/L3 memory content).
//!
//! Source: TDAM `MC/core/memory-prompt/types.ts` (reference only — ids are
//! deterministic sanitized strings here, no crypto hash, no new deps).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::conversation::sanitize_component;

/// Which pipeline layer a custom strategy tunes (TDAM `MemoryPromptLayer`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPromptLayer {
    /// L1 memories.
    L1,
    /// L2 scenes.
    L2,
    /// L3 persona / doctrine.
    L3,
}

impl MemoryPromptLayer {
    /// Lowercase wire tag (`"l1"`), also used in setting ids.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::L1 => "l1",
            Self::L2 => "l2",
            Self::L3 => "l3",
        }
    }
}

/// Resolution scope of a prompt (TDAM `MemoryPromptSource`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPromptSource {
    Agent,
    Team,
    Instance,
    System,
}

/// Lifecycle status of a prompt record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptStatus {
    Active,
    Deleting,
}

/// A custom strategy prompt (TDAM `MemoryPromptRecord`, persisted shape).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryPromptRecord {
    pub memory_prompt_id: String,
    pub name: String,
    pub layer: MemoryPromptLayer,
    pub prompt: String,
    pub version: u32,
    pub status: PromptStatus,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

/// Binding of a prompt to a resolution target (TDAM
/// `MemoryPromptSettingRecord`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryPromptSettingRecord {
    pub setting_id: String,
    /// `"agent" | "team" | "instance"` — mirrors [`MemoryPromptSource`]
    /// minus the read-only `system` variant.
    pub target_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub layer: MemoryPromptLayer,
    pub memory_prompt_id: String,
    pub updated_at_ms: u64,
}

/// The outcome of resolving which custom strategy applies to a target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedMemoryPrompt {
    pub memory_prompt_id: String,
    pub prompt: String,
    pub layer: MemoryPromptLayer,
    pub source: MemoryPromptSource,
    pub version: u32,
}

/// Errors surfaced by the memory-prompt layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MemoryPromptError {
    /// An agent-scoped setting was requested without a team id (TDAM throws
    /// `teamId is required for an agent prompt setting`).
    #[error("team id is required for an agent prompt setting")]
    AgentRequiresTeam,
    /// Underlying VantaDB storage error (store implementations).
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
}

/// Deterministic setting id for a target + layer. TDAM hashes the target with
/// sha256; here the sanitized concatenation is enough (ids are not
/// security-sensitive and no new dependency is allowed).
pub fn build_memory_prompt_setting_id(
    team_id: Option<&str>,
    agent_id: Option<&str>,
    layer: MemoryPromptLayer,
) -> Result<String, MemoryPromptError> {
    if let Some(agent) = agent_id {
        let Some(team) = team_id else {
            return Err(MemoryPromptError::AgentRequiresTeam);
        };
        return Ok(format!(
            "mps:a/{}/{}/{}",
            sanitize_component(team, 128, true),
            sanitize_component(agent, 128, true),
            layer.as_str()
        ));
    }
    if let Some(team) = team_id {
        return Ok(format!(
            "mps:t/{}/{}",
            sanitize_component(team, 128, true),
            layer.as_str()
        ));
    }
    Ok(format!("mps:i/{}", layer.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_wire_tags_are_lowercase() {
        assert_eq!(MemoryPromptLayer::L1.as_str(), "l1");
        assert_eq!(MemoryPromptLayer::L3.as_str(), "l3");
    }

    #[test]
    fn setting_ids_are_deterministic_per_target() {
        let instance = build_memory_prompt_setting_id(None, None, MemoryPromptLayer::L1).unwrap();
        assert_eq!(instance, "mps:i/l1");

        let team =
            build_memory_prompt_setting_id(Some("team a"), None, MemoryPromptLayer::L2).unwrap();
        assert_eq!(team, "mps:t/team_a/l2");

        let agent =
            build_memory_prompt_setting_id(Some("team a"), Some("agent 1"), MemoryPromptLayer::L3)
                .unwrap();
        assert_eq!(agent, "mps:a/team_a/agent_1/l3");
    }

    #[test]
    fn agent_setting_requires_team() {
        assert!(matches!(
            build_memory_prompt_setting_id(None, Some("agent-1"), MemoryPromptLayer::L1),
            Err(MemoryPromptError::AgentRequiresTeam)
        ));
    }
}
