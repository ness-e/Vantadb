//! L1 extraction response parser — turns the LLM's JSON array of scene
//! segments into typed [`SceneSegment`]s (port of TDAM
//! `l1-extractor.ts:parseExtractionResult` + `normalizeType`).
//!
//! Tolerant by design: a malformed item or an unknown/legacy type drops only
//! that item — never the whole batch.

use serde_json::Value;

use crate::core::abstractions::{ExtractedMemory, MemoryType, SceneSegment};
use crate::offload::local_llm::parsers::json_utils::extract_json;

/// Scene name used when the LLM omits it.
const DEFAULT_SCENE: &str = "unknown-scene";

/// Parse the L1 extraction response into typed scene segments.
///
/// Returns an empty vec when no JSON array can be recovered. On the first
/// failure a targeted repair of bare `"priority"` scalars is attempted
/// (TDAM `repairExtractionJson`), so one bad scalar doesn't drop the batch.
pub fn parse_l1_extraction(raw: &str) -> Vec<SceneSegment> {
    let items = match extract_json::<Vec<Value>>(raw) {
        Some(items) => items,
        None => {
            let repaired = repair_priority_scalars(raw);
            if repaired == raw {
                return Vec::new();
            }
            match extract_json::<Vec<Value>>(&repaired) {
                Some(items) => items,
                None => return Vec::new(),
            }
        }
    };
    let mut scenes = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let scene_name = obj
            .get("scene_name")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_SCENE)
            .to_string();
        let message_ids = string_array(obj.get("message_ids"));
        let memories = obj
            .get("memories")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| memory_from_value(m, &scene_name))
                    .collect()
            })
            .unwrap_or_default();
        scenes.push(SceneSegment {
            scene_name,
            message_ids,
            memories,
        });
    }
    scenes
}

/// Repair a bare identifier in a `"priority"` value — e.g. `"priority": sheet`
/// — replacing it with the default `50` (port of TDAM
/// `l1-extractor.ts:repairExtractionJson`, 592-598; string-aware instead of
/// regex so string values and nested occurrences are untouched).
fn repair_priority_scalars(raw: &str) -> String {
    const KEY: &str = "\"priority\"";
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            let key_len = KEY.chars().count();
            if i + key_len <= chars.len() && chars[i..i + key_len].iter().collect::<String>() == KEY
            {
                out.push_str(KEY);
                i += key_len;
                while i < chars.len() && chars[i].is_whitespace() {
                    out.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == ':' {
                    out.push(':');
                    i += 1;
                    while i < chars.len() && chars[i].is_whitespace() {
                        out.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() {
                        let v = chars[i];
                        let valid_start = matches!(v, '"' | '{' | '[' | '-' | '0'..='9');
                        if !valid_start {
                            out.push_str("50");
                            while i < chars.len() && !matches!(chars[i], ',' | '}' | ']') {
                                i += 1;
                            }
                            continue;
                        }
                    }
                }
                continue;
            }
            out.push(c);
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Normalize a raw LLM type string against [`MemoryType`], folding legacy
/// aliases (TDAM `l1-extractor.ts:728-737`).
pub fn normalize_type(raw: &str) -> Option<MemoryType> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "persona" | "preference" => Some(MemoryType::Persona),
        "episodic" | "episode" => Some(MemoryType::Episodic),
        "instruction" | "instruct" => Some(MemoryType::Instruction),
        "work_fact" => Some(MemoryType::WorkFact),
        "work_task" => Some(MemoryType::WorkTask),
        "work_method" => Some(MemoryType::WorkMethod),
        "work_artifact" => Some(MemoryType::WorkArtifact),
        _ => None,
    }
}

fn string_array(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn memory_from_value(v: &Value, scene_name: &str) -> Option<ExtractedMemory> {
    let obj = v.as_object()?;
    let content = obj
        .get("content")
        .and_then(Value::as_str)?
        .trim()
        .to_string();
    if content.is_empty() {
        return None;
    }
    let raw_type = obj
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("episodic");
    let memory_type = normalize_type(raw_type)?;
    let priority = obj
        .get("priority")
        .and_then(|p| {
            p.as_i64()
                .map(|v| v as i32)
                .or_else(|| p.as_f64().map(|v| v as i32))
        })
        .unwrap_or(50);
    let source_message_ids = string_array(obj.get("source_message_ids"));
    let metadata = obj
        .get("metadata")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Some(ExtractedMemory {
        content,
        memory_type,
        priority,
        source_message_ids,
        scene_name: scene_name.to_string(),
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use crate::core::abstractions::MemoryType;

    use super::{normalize_type, parse_l1_extraction};

    #[test]
    fn parses_valid_batch() {
        let raw = r#"[
          {"scene_name": "Debugging the auth flow", "message_ids": ["m1", "m2"], "memories": [
            {"content": "User prefers dark mode", "type": "persona", "priority": 70, "source_message_ids": ["m1"], "metadata": {}}
          ]}
        ]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_name, "Debugging the auth flow");
        assert_eq!(scenes[0].message_ids, vec!["m1", "m2"]);
        assert_eq!(scenes[0].memories.len(), 1);
        assert_eq!(scenes[0].memories[0].content, "User prefers dark mode");
        assert_eq!(scenes[0].memories[0].scene_name, "Debugging the auth flow");
        assert_eq!(scenes[0].memories[0].priority, 70);
    }

    #[test]
    fn legacy_types_are_normalized() {
        assert_eq!(normalize_type("episode"), Some(MemoryType::Episodic));
        assert_eq!(normalize_type("instruct"), Some(MemoryType::Instruction));
        assert_eq!(normalize_type("preference"), Some(MemoryType::Persona));
        assert_eq!(normalize_type("Work_Fact"), Some(MemoryType::WorkFact));
        assert_eq!(normalize_type("bogus"), None);
    }

    #[test]
    fn invalid_type_drops_only_that_memory() {
        let raw = r#"[
          {"scene_name": "s1", "message_ids": [], "memories": [
            {"content": "valid one", "type": "episodic", "priority": 60},
            {"content": "invalid type", "type": "quantum_state", "priority": 60}
          ]}
        ]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].memories.len(), 1);
        assert_eq!(scenes[0].memories[0].content, "valid one");
    }

    #[test]
    fn missing_fields_get_defaults() {
        let raw = r#"[{"memories": [{"content": "  "}]}]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_name, "unknown-scene");
        // Empty content is dropped; priority/type default otherwise.
        assert!(scenes[0].memories.is_empty());
    }

    #[test]
    fn missing_metadata_defaults_to_empty_object() {
        let raw = r#"[{"scene_name": "s", "message_ids": [], "memories": [
          {"content": "no metadata", "type": "episodic", "priority": 50}
        ]}]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes[0].memories[0].metadata, serde_json::json!({}));
    }

    #[test]
    fn prose_wrapped_response_parses() {
        let raw = "Here you go:\n```json\n[{\"scene_name\": \"s\", \"message_ids\": [], \"memories\": []}]\n```\nLet me know if you need more.";
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].scene_name, "s");
    }

    #[test]
    fn bare_priority_identifier_repaired_not_dropped() {
        // "priority": sheet is invalid JSON; the batch must survive (TDAM
        // repairExtractionJson) and the memory keeps the default priority.
        let raw = r#"[{"scene_name": "s", "message_ids": [], "memories": [
          {"content": "good one", "type": "episodic", "priority": sheet}
        ]}]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].memories.len(), 1);
        assert_eq!(scenes[0].memories[0].content, "good one");
        assert_eq!(scenes[0].memories[0].priority, 50);
    }

    #[test]
    fn priority_string_value_left_alone() {
        let raw = r#"[{"scene_name": "s", "message_ids": [], "memories": [
          {"content": "string prio", "type": "episodic", "priority": "high"}
        ]}]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes[0].memories[0].content, "string prio");
        assert_eq!(scenes[0].memories[0].priority, 50);
    }

    #[test]
    fn priority_inside_string_not_repaired() {
        // The key must be a real JSON key, not text inside a string value.
        let raw = r#"[{"scene_name": "s", "message_ids": [], "memories": [
          {"content": "mentions \"priority\": sheet literally", "type": "episodic"}
        ]}]"#;
        let scenes = parse_l1_extraction(raw);
        assert_eq!(scenes[0].memories.len(), 1);
        assert_eq!(
            scenes[0].memories[0].content,
            "mentions \"priority\": sheet literally"
        );
    }

    #[test]
    fn non_json_returns_empty() {
        assert!(parse_l1_extraction("I cannot do that.").is_empty());
        assert!(parse_l1_extraction("").is_empty());
    }
}
