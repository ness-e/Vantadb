---
title: "ADR-040: auto-consolidación conversacional local-first (MCP-41)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, vanta-memory, mcp]
created: 2026-09-10
last_reviewed: 2026-09-10
---

# ADR-040: auto-consolidación conversacional local-first (MCP-41)

## Context

Brecha mem0/graphiti: las escenas existen (MEM-12 índice LLM-free, MEM-13 tools
sandboxed, MEM-14 estrategia UPDATE>MERGE>CREATE + heat + soft-delete) y el recall
es local-first (`scene_query` keyword, `perform_auto_recall` modo keyword), pero el
`extract` exigía un `LlmRunner` (`extract_scenes_with_llm`) — es decir, una LLM key
obligatoria. El plan (Task 17, appetite max 3d frente a esfuerzo 1-2 semanas)
exige slice DISCOVERY+diseño+vertical mínimo, con stop condition DEFER si no hay
diseño viable local-first.

## Decision

Pipeline extract→consolidate→recall 100% local-first sin LLM key:

- **Extract (nuevo):** `vanta-memory/src/core/scene/auto_consolidate.rs` —
  `extract_local(&[LocalTurn]) -> Vec<SceneExtraction>` heurístico y determinista
  (1 escena viva por turno no vacío; nombre = top-3 términos significativos
  ordenados + fallback `turn-{n}`, siempre por `normalize_scene_name`; truncado a
  `MAX_*_BYTES`; sin `merge_sources` — el juicio de overlap necesita LLM).
- **Consolidate (reutilizado, sin cambios):** `extract_scenes` (decide+apply MEM-14;
  re-consolidar los mismos turnos da UPDATE con heat+1, paridad TDAM) y, para L1
  profunda, `consolidate_session` (dream, `dream/<s>/<run_id>`, originales intactos).
- **Recall (reutilizado, sin cambios):** `scene_list` / `scene_query` keyword +
  MCP `scene_read`/`scene_list`/`scene_query` existentes.
- **NO se añade MCP tool en este slice:** un tool 77 rompería los conteos fijados
  en tests (`config.rs`, `handlers/tools.rs`, `mcp_tests.rs` 76 tools) y los perfiles
  MCP-37. Follow-up diseñado: `scene_consolidate(session_key, turns[{role,content}])`
  write-tool (readOnlyHint false, idempotentHint true), validación en boundary con
  `validate_identifier`/`validate_payload`, perfil Full, conteos 76→77 + tests
  `scene_tests.rs`.

## Consequences

- Pros: sin LLM key por diseño (pre-mortem cumplido); blast radius = 1 archivo nuevo
  + 3 líneas en `mod.rs`; API aditiva (`LocalTurn` `#[non_exhaustive]` + `::new`);
  superficie Hyrum declarada débil a propósito (nombres heurísticos no garantizados
  estables); `cargo test -p vanta-memory` 0 failed + clippy 0.
- Cons: granularidad por turno (sin detección de tópicos — requiere LLM o
  embeddings, futuro); nombres heurísticos pueden colisionar y fusionar por UPDATE
  (documentado, paridad TDAM update-first); promoción dream→L1 sigue en MEM-65.
