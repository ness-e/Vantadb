// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-69 batch extract+dedup tests: split+dedup agrupados en 1 llamada LLM.
//!
//! Usa un `ScriptedRunner` local (contador de llamadas + script por `task_id`,
//! patrón `CapturingRunner` de `tests/l1_extractor.rs`) — el conteo de llamadas
//! es mock-dependiente por diseño (pre-mortem MEM-69) y el `ScriptedRunner` lo
//! hace determinista sin la feature `mock` ni LLM real.

use std::collections::HashMap;
use std::sync::Mutex;

use vanta_memory::core::abstractions::{
    DedupAction, DedupDecision, ExtractedMemory, LlmError, LlmRunParams, LlmRunner, MemoryRecord,
    MemoryType,
};
use vanta_memory::core::conversation::{L0Message, L0Role};
use vanta_memory::core::record::{
    batch_dedup, extract_dedup_batch, extract_l1_segments, prepare_pending, L1DedupConfig,
    L1ExtractorConfig, PendingMemory, EXTRACT_DEDUP_TASK_ID,
};

/// Runner scriptado: enruta respuestas por `task_id`, cuenta llamadas y
/// captura prompts. Sin script para un `task_id` → error (llamada inesperada).
struct ScriptedRunner {
    calls: Mutex<Vec<String>>,
    prompts: Mutex<Vec<String>>,
    script: Mutex<HashMap<String, Result<String, LlmError>>>,
}

impl ScriptedRunner {
    fn new(script: Vec<(&str, Result<String, LlmError>)>) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            prompts: Mutex::new(Vec::new()),
            script: Mutex::new(
                script
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            ),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    fn last_prompt(&self) -> String {
        self.prompts.lock().unwrap().last().unwrap().clone()
    }
}

impl LlmRunner for ScriptedRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        self.calls.lock().unwrap().push(params.task_id.clone());
        self.prompts.lock().unwrap().push(params.prompt.clone());
        let mut script = self.script.lock().unwrap();
        match script.remove(&params.task_id) {
            Some(result) => result,
            None => Err(LlmError::Other(format!(
                "unexpected call: {}",
                params.task_id
            ))),
        }
    }
}

fn msg(id: &str, content: &str, ts: u64) -> L0Message {
    L0Message {
        id: Some(id.to_string()),
        role: L0Role::User,
        content: content.to_string(),
        timestamp_ms: ts,
    }
}

fn existing(id: &str, content: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        content: content.into(),
        memory_type: MemoryType::Persona,
        priority: 80,
        scene_name: "s".into(),
        source_message_ids: vec![],
        metadata: serde_json::Value::Null,
        timestamps: vec!["2026-08-20T10:00:00.000Z".into()],
        created_at: "2026-08-20T10:00:00.000Z".into(),
        updated_at: "2026-08-20T10:00:00.000Z".into(),
        version: 1,
        session_key: "sk".into(),
        session_id: String::new(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 0,
        superseded_by: None,
    }
}

fn extract_config() -> L1ExtractorConfig {
    L1ExtractorConfig::default()
}

fn dedup_config() -> L1DedupConfig {
    L1DedupConfig::default()
}

const NOW: u64 = 1_700_000_000_000;

// Una memoria extraída con juicio dedup inline (duplicado de un existente).
fn combined_response() -> String {
    r#"[
      {"scene_name": "Setting up the project", "message_ids": ["m2"], "memories": [
        {"content": "User prefers dark mode", "type": "persona", "priority": 80,
         "source_message_ids": ["m2"], "metadata": {},
         "dedup": {"action": "skip", "target_ids": ["m1"]}}
      ]}
    ]"#
    .to_string()
}

// (a) CONTRATO: split+dedup en exactamente 1 llamada con task_id propio.
#[test]
fn batch_groups_split_and_dedup_in_one_call() {
    let runner = ScriptedRunner::new(vec![(EXTRACT_DEDUP_TASK_ID, Ok(combined_response()))]);
    let messages = vec![msg("m2", "I prefer dark mode", 1200)];
    let existing_records = vec![existing("m1", "user prefers dark mode")];

    let (result, pending, decisions) = extract_dedup_batch(
        &runner,
        &messages,
        &existing_records,
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert_eq!(runner.call_count(), 1, "split+dedup en 1 sola llamada");
    assert!(result.success);
    assert_eq!(result.extracted_count, 1);
    assert_eq!(pending.len(), 1);
    assert_eq!(decisions.len(), 1, "1 memoria → exactamente 1 decisión");
    assert_eq!(pending[0].record_id, decisions[0].record_id);
    assert_eq!(decisions[0].action, DedupAction::Skip);
    assert_eq!(decisions[0].target_ids, vec!["m1".to_string()]);
    assert_eq!(pending[0].record_id, "m_1700000000000_0");
}

// (b) Quality gate intacto: solo ruido → 0 llamadas, resultado vacío success.
#[test]
fn batch_noise_only_makes_no_call() {
    let runner = ScriptedRunner::new(vec![]);
    let messages = vec![
        msg("n1", "/clear", 1000),
        msg("n2", "!!!", 1100),
        msg("n3", "(session bootstrap)", 1200),
    ];

    let (result, pending, decisions) = extract_dedup_batch(
        &runner,
        &messages,
        &[existing("m1", "user prefers dark mode")],
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert_eq!(runner.call_count(), 0, "ruido no debe llamar al LLM");
    assert!(result.success);
    assert!(pending.is_empty());
    assert!(decisions.is_empty());
}

// (b2) Quality gate a nivel prompt: el ruido no viaja en el prompt.
#[test]
fn batch_prompt_excludes_noise_content() {
    let runner = ScriptedRunner::new(vec![(EXTRACT_DEDUP_TASK_ID, Ok(combined_response()))]);
    let messages = vec![
        msg("n1", "/clear", 1000),
        msg("m2", "I prefer dark mode", 1200),
    ];

    let _ = extract_dedup_batch(
        &runner,
        &messages,
        &[],
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert_eq!(runner.call_count(), 1);
    let prompt = runner.last_prompt();
    assert!(!prompt.contains("/clear"), "ruido fuera del prompt");
    assert!(prompt.contains("I prefer dark mode"));
}

// (c) COMPARATIVO (pre-mortem/stop condition): batch ≡ split en calidad,
// con la mitad de llamadas. Si diverge → DEFER con estos números.
#[test]
fn batch_matches_split_path_quality() {
    let extraction_json = r#"[
      {"scene_name": "Setting up the project", "message_ids": ["m2"], "memories": [
        {"content": "User prefers dark mode", "type": "persona", "priority": 80,
         "source_message_ids": ["m2"], "metadata": {}}
      ]}
    ]"#
    .to_string();
    // Path split: extracción (1 llamada) + dedup (1 llamada) = 2.
    let split_runner = ScriptedRunner::new(vec![
        ("l1-extraction", Ok(extraction_json)),
        (
            "l1-conflict-detection",
            Ok(
                r#"[{"record_id": "m_1700000000000_0", "action": "skip", "target_ids": ["m1"]}]"#
                    .to_string(),
            ),
        ),
    ]);
    let messages = vec![msg("m2", "I prefer dark mode", 1200)];
    let existing_records = vec![existing("m1", "user prefers dark mode")];

    let (split_result, split_memories) =
        extract_l1_segments(&split_runner, &messages, None, &extract_config());
    let split_pending = prepare_pending(&split_memories, NOW);
    let split_decisions = batch_dedup(
        &split_runner,
        &split_pending,
        &existing_records,
        &dedup_config(),
    );
    assert_eq!(split_runner.call_count(), 2, "path split usa 2 llamadas");

    // Path batch: 1 llamada.
    let batch_runner = ScriptedRunner::new(vec![(EXTRACT_DEDUP_TASK_ID, Ok(combined_response()))]);
    let (batch_result, batch_pending, batch_decisions) = extract_dedup_batch(
        &batch_runner,
        &messages,
        &existing_records,
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );
    assert_eq!(batch_runner.call_count(), 1, "path batch usa 1 llamada");

    // Calidad: mismas memorias (contenido/tipo/prioridad) + mismas acciones.
    assert!(split_result.success && batch_result.success);
    assert_eq!(batch_result.extracted_count, split_result.extracted_count);
    let split_contents: Vec<(&str, MemoryType, i32)> = split_memories
        .iter()
        .map(|m| (m.content.as_str(), m.memory_type, m.priority))
        .collect();
    let batch_contents: Vec<(&str, MemoryType, i32)> = batch_pending
        .iter()
        .map(|p| {
            (
                p.memory.content.as_str(),
                p.memory.memory_type,
                p.memory.priority,
            )
        })
        .collect();
    assert_eq!(batch_contents, split_contents, "memorias idénticas");
    let split_actions: Vec<DedupAction> = split_decisions.iter().map(|d| d.action).collect();
    let batch_actions: Vec<DedupAction> = batch_decisions.iter().map(|d| d.action).collect();
    assert_eq!(batch_actions, split_actions, "juicios idénticos");
}

// (d) Principio 4: fallo del runner → success:false, L0 intacto (vacío, sin pánico).
#[test]
fn batch_runner_failure_degrades_without_data_loss() {
    let runner = ScriptedRunner::new(vec![(
        EXTRACT_DEDUP_TASK_ID,
        Err(LlmError::Other("boom".into())),
    )]);
    let messages = vec![msg("m2", "I prefer dark mode", 1200)];

    let (result, pending, decisions) = extract_dedup_batch(
        &runner,
        &messages,
        &[existing("m1", "user prefers dark mode")],
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert!(!result.success);
    assert!(pending.is_empty());
    assert!(decisions.is_empty());
}

// (e) Juicio ausente/malformado → `store` (safe default), la memoria se conserva.
#[test]
fn batch_missing_dedup_judgment_falls_back_to_store() {
    let runner = ScriptedRunner::new(vec![(
        EXTRACT_DEDUP_TASK_ID,
        Ok(r#"[
          {"scene_name": "s", "message_ids": ["m2"], "memories": [
            {"content": "User prefers dark mode", "type": "persona", "priority": 80,
             "source_message_ids": ["m2"], "metadata": {}},
            {"content": "Second memory", "type": "episodic", "priority": 70,
             "source_message_ids": ["m2"], "metadata": {},
             "dedup": {"action": "explode", "target_ids": []}}
          ]}
        ]"#
        .to_string()),
    )]);
    let messages = vec![msg("m2", "I prefer dark mode", 1200)];

    let (result, pending, decisions) = extract_dedup_batch(
        &runner,
        &messages,
        &[existing("m1", "user prefers dark mode")],
        None,
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert!(result.success);
    assert_eq!(pending.len(), 2, "ninguna memoria se pierde");
    assert_eq!(decisions.len(), 2);
    assert!(
        decisions.iter().all(|d| d.action == DedupAction::Store),
        "juicio ausente/inválido → store"
    );
}

// Contrato de tipos: el resultado espeja `extract_l1_segments` (records vacío,
// stored_count 0 — escritura vive en el caller, como MEM-10/MEM-11).
#[test]
fn batch_result_shape_mirrors_extractor() {
    let runner = ScriptedRunner::new(vec![(EXTRACT_DEDUP_TASK_ID, Ok(combined_response()))]);
    let messages = vec![msg("m2", "I prefer dark mode", 1200)];

    let (result, pending, _): (
        vanta_memory::core::abstractions::L1ExtractionResult,
        Vec<PendingMemory>,
        Vec<DedupDecision>,
    ) = extract_dedup_batch(
        &runner,
        &messages,
        &[],
        Some("prev scene"),
        &extract_config(),
        &dedup_config(),
        NOW,
    );

    assert!(result.records.is_empty());
    assert_eq!(result.stored_count, 0);
    assert_eq!(
        result.last_scene_name.as_deref(),
        Some("Setting up the project")
    );
    assert_eq!(
        pending[0].memory,
        ExtractedMemory {
            content: "User prefers dark mode".into(),
            memory_type: MemoryType::Persona,
            priority: 80,
            source_message_ids: vec!["m2".into()],
            scene_name: "Setting up the project".into(),
            // `{}` preserved verbatim by `memory_from_value` (same as split path).
            metadata: serde_json::json!({}),
        }
    );
}
