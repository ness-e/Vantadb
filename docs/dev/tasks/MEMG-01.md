---
title: MEMG-01 — Detección de contradicción en ingesta L1
kind: task
description: "El juicio de dedup L1 detecta contradicciones explícitas (campo contradicts) y marca el registro viejo con superseded_by vía mark_contradiction — nunca delete; test dedicado."
---

# MEMG-01: Detección de contradicción en ingesta L1

## Metadata
- **Plan file:** docs/dev/plans/2026-10-04-master-plan-0.9.0.md (Task 37, F2 — Memoria I)
- **Fuente:** plan file Task 37 (bloque expandido a nivel F0, 2026-10-05)
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🟠
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-10-05T02:00
- **last-synced:** 2026-10-05T02:38
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas (la única incógnita del plan — "¿el juicio de contradicción viaja en el prompt de dedup sin degradar la extracción?" — resuelta en DISCOVERY: sí, como campo opcional tolerante; ver Notas)
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `core::record::l1_dedup` (run_l1_dedup → apply_dedup_batch), `core::record::l1_batch` (extract_dedup_batch MEM-69 → apply_dedup_batch), `services::pipeline_worker` (run_l1_inner dos-call + run_l1_batch fusionado), `core::record::approval` (apply_dedup_batch), tests: `l1_dedup.rs`, `generation_log.rs`, `recall.rs`, `pipeline_manager.rs`, `capture_approval.rs`, `vantadb-mcp/tests/test_query_embed.rs` |
| Callees | `core::record::lifecycle::mark_contradiction` (MEM-60 — reutilizada, sin cambios), `core::record::l1_reader::{read_record, read_session_records, recall_candidates}`, `core::record::l1_writer::put_record`, `core::prompts::l1_dedup` (system prompts), core SDK `vantadb::sdk::Embedded` |
| Implicaciones | (1) `DedupDecision` gana un campo público `contradicts: Vec<String>` (`#[serde(default)]`) → literales de construcción en workspace actualizados (compilador verifica exhaustividad; 11 sitios enumerados por grep); wire JSON retrocompatible. (2) `write_memory` marca targets contradichos vía `mark_contradiction` — misma semántica que dream (supersede, nunca delete). (3) Sin cambios de firma pública en funciones. (4) Sin migración de datos (campo serde default). (5) Performance: sin cambio en hot path de búsqueda; costo extra = 1 `is_empty()` en el path de escritura y trabajo O(contradicts) solo cuando el LLM marca una contradicción explícita. (6) Tests existentes de dedup intactos (semántica store/update/skip sin cambio). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/record/l1_dedup.rs` (534L), `vanta-memory/src/core/prompts/l1_dedup.rs` (259L), `vanta-memory/src/core/record/l1_batch.rs` (298L), `vanta-memory/src/core/record/l1_reader.rs` (322L), `vanta-memory/src/core/record/l1_writer.rs` (394L), `vanta-memory/src/core/record/mod.rs` (50L), `vanta-memory/src/core/prompts/mod.rs` (31L), `vanta-memory/tests/l1_dedup.rs` (594L), `vanta-memory/src/core/abstractions/types.rs` (secciones 100-379 + sitios enumerados), `vanta-memory/src/core/dream/mod.rs` (secciones 46-371: reuso de `mark_contradiction`/`ContradictionProvenance`), `vanta-memory/src/services/pipeline_worker.rs` (secciones 460-589: ambos paths de ingesta)
- **Archivos referenciados hacia dentro (imports/deps):** `l1_writer` ← `l1_dedup` (apply_dedup_batch, generate_memory_id, EmbedFn, L1Error), `l1_batch`, `approval`, `pipeline_worker`; `l1_reader::{read_record, read_session_records}`; `lifecycle::mark_contradiction`; `prompts::l1_dedup` (format/prompt); `abstractions::{DedupDecision, DedupAction, MemoryRecord}`; SDK `vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata, Value}`
- **Archivos que referencian a los editados (referencias entrantes):** grep workspace `DedupDecision {` → 18 matches en 10 archivos (todos enumerados arriba); grep `apply_dedup_batch|write_memory(` → 30 matches (callers productivos: `l1_dedup.rs:233`, `l1_batch` vía `pipeline_worker.rs:523`, `approval.rs:137`, `l1_writer.rs:278`; resto tests). `write_memory` tiene 12 callers (codegraph), `apply_dedup_batch` 9 callers (codegraph).
- **Veredicto impacto:** **medio** — cambio aditivo de tipo público + lógica nueva en el choke point de escritura (`write_memory`), que ambos paths de ingesta comparten. Se rompe solo la construcción por literal de `DedupDecision` (compilador la detecta; 11 sitios actualizados). Sin cambio de comportamiento para registros no-contradictorios. Dream intacto (reutiliza la misma función, punto de llamada distinto).

## Contrato

"`cargo nextest run -p vanta-memory --profile audit` pasa y `cargo nextest run -p vanta-memory --test l1_contradiction` (test dedicado) prueba: (a) caso canónico `me gusta X` → `ya no me gusta X` marca el registro viejo con `superseded_by` (no lo borra) y emite provenance vía `mark_contradiction`; (b) registros no-contradictorios conservan semántica store/update/skip idéntica; (c) señal conservadora — ya-superseded, ids desconocidos y self-referencias se saltan; (d) el juicio viaja en el prompt de dedup existente (un solo call LLM, sin llamada nueva)."

## Spec (SDD — feature-add detectado: campo público nuevo + capability nueva de ingesta)

> Phase 1b: señal "firma pública nueva" (campo `contradicts` en `DedupDecision` público) + capability nueva (detección al ingerir). Spec resuelta por evidencia (válida según question-gates §"Contenido válido": justificación por-evidencia por ítem).

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Mecanismo de juicio | A: extender el juicio de dedup existente (campo inline, 0 llamadas nuevas) / B: segunda llamada LLM dedicada (mejor foco, +costo por flush) / C: heurística LLM-free (barato, no detecta negaciones explícitas reales) | A | ✅ decidido-por-evidencia (plan Task 37 pre-mortem #1: "extender el juicio de dedup existente (patrón `l1_batch` MEM-69) antes que uno nuevo"; ref: `l1_dedup.rs:145-190` batch_dedup, `l1_batch.rs:50-136`) |
| 2 | Forma de la señal | A: campo `contradicts: Vec<String>` en `DedupDecision` / B: nuevo `DedupAction::Supersede` (duplica semántica update, rompe "mismos store/update/skip") | A | ✅ decidido-por-evidencia (contrato: "registros no-contradictorios sin cambio de semántica"; reuso literal de `mark_contradiction` — ref: `lifecycle.rs:86-108`) |
| 3 | Punto de aplicación | A: `write_memory` (choke point único de ambos paths) / B: `run_l1_dedup` solamente (dejaría fuera el path fusionado MEM-69) | A | ✅ decidido-por-evidencia (ambos paths convergen en `apply_dedup_batch` → `write_memory`: ref: `l1_dedup.rs:233`, `pipeline_worker.rs:523`, `approval.rs:137`) |
| 4 | Semántica del marcado | A: `superseded_by` + provenance, nunca delete, saltar ya-superseded (conservadora, revisable) / B: delete del viejo (pérdida de auditoría) / C: cuarentena (MGR-13 §3.4 la difiere a v1.0 — fuera de scope) | A | ✅ decidido-por-evidencia (contrato + pre-mortem #2/#3; `mark_contradiction` es la base reutilizable) |
| 5 | Degradación LLM-free | A: sin runner/candidatos → store-all sin `contradicts` (comportamiento actual intacto) / B: fallar | A | ✅ decidido-por-evidencia (Principio 4, ref: `l1_dedup.rs:143-190`: "never fails and never drops memories") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) el registro viejo NUNCA se borra por contradicción — solo gana `superseded_by` (auditoría preservada); (2) registros no-contradictorios conservan EXACTA semántica store/update/merge/skip; (3) `mark_contradiction` es la única semántica de marcado (cero semántica paralela); (4) degradación LLM-free intacta: nunca pierde datos, nunca bloquea; (5) dream (`resolve_contradictions`) intacto — misma función, punto de llamada distinto; (6) cuarentena por contradicción FUERA de scope (MGR-13 §3.4).
- **Comandos de verificación:** `cargo nextest run -p vanta-memory --test l1_contradiction` (test dedicado) + `cargo nextest run -p vanta-memory --profile audit` (suite scoped) + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` + `cargo fmt --check`
- **Deuda pendiente:** contradicciones cross-sesión (el pool de candidatos es intra-sesión — `read_session_records`); auditoría persistente de provenance más allá del tracing event de `mark_contradiction` (la deuda ya declarada en `lifecycle.rs:100`); detección de contradicciones implícitas (solo explícitas por diseño conservador); **consumo read-side del flag (`superseded_by` no se filtra en recall) → `FIND-277`**.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (cambio aditivo con test dedicado; reutiliza infraestructura existente; cero `unwrap`/`expect` nuevos en código productivo).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable del task file se cumple + capa determinista (fmt, clippy, nextest scoped) + test dedicado RED→GREEN |
| **Commit** | Commit atómico, conventional commit `feat(memory):`, `git diff` limpio, verificación mecánica (nunca auto-reporte) |
| **Release** | Changelog vía release-plz (vanta-memory → minor por `feat:`); sin cambios manuales de versión; push NO (política owner) |

## Herramientas necesarias
- Terminal cargo (check, clippy, fmt, nextest — MCPs de Rust deshabilitados por default)
- codegraph_codegraph_explore (blast radius — usado en DISCOVERY)
- codebase-memory-mcp (coverage/index — usado en DISCOVERY)

**Skills cargadas (SDP):** `campaign-executor` (base type Rust core), `progreso` (base), `ponytail` (base), `test-driven-development` (pinned policy tests/fix — RED→GREEN del test dedicado), `systematic-debugging` (pinned policy tests/fix), `source-driven-development` (base Rust core), `doubt-driven-development` (base Rust core), `incremental-implementation` (lifecycle BUILD — slices verticales). Adicionales del prompt del orquestador: `rust-write-tests` (calidad de tests Rust), `documentation-skill` (edita `docs/api/VANTA_MEMORY.md`).

## Investigation Notes

- **Código real (re-verificado HEAD 2026-10-05):** la detección de contradicciones existe SOLO del lado dream: `resolve_contradictions` (`core/dream/mod.rs:335-376`, LLM-free, prioridad+timestamp) llama `mark_contradiction` (`core/record/lifecycle.rs:86-108` — setea `superseded_by` en el viejo, nunca borra, emite `ContradictionProvenance` + tracing) sobre clones transitorios; `consolidate_session` estampa los copies dream-side (`dream/mod.rs:1013`). La ingesta L1 (`apply_dedup_batch` → `write_memory`) solo decide store/update/merge/skip: cero detección.
- **Punto de extensión natural confirmado:** ambos paths de ingesta (`run_l1_dedup` dos-call y `extract_dedup_batch` MEM-69 fusionado) convergen en `apply_dedup_batch` → `write_memory`. El juicio de dedup ya viaja en UNA llamada LLM (batch) y el parse es tolerante (`decision_from_value`). Extender ese juicio con `contradicts` = 0 llamadas nuevas (pre-mortem #1 resuelto).
- **Incógnita del plan resuelta:** "¿el juicio de contradicción viaja en el prompt de dedup sin degradar la extracción?" → Sí: (a) en el path dos-call es un campo opcional del output del mismo prompt de conflicto; (b) en el path fusionado es un campo inline del objeto `dedup` por memoria (alineación por construcción, `l1_batch.rs:220-250`); (c) parse tolerante: campo ausente/malformado → `[]` (jamás degrada a fallo); (d) regla conservadora explícita en el prompt (solo negación explícita; duda → `[]`).
- **Sin ambigüedad de APIs externas** — no se requiere web research (todo es diseño interno sobre patrones existentes del repo).
- **Nota de árbol:** el working tree tiene WIP de otras sesiones (`docs/dev/plans/2026-10-04-master-plan-0.9.0.md`, `opencode.jsonc`) — PROHIBIDO tocarlos; commit con pathspec solo a archivos de esta tarea.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (ver Investigation Notes) |
| Pendientes de ejecución (downhill) | 0 steps |
| % completado | 100% |

## Resultados de verificación (mecánica, no auto-reporte)

| Gate | Comando | Resultado |
|------|---------|-----------|
| RED | `cargo nextest run -p vanta-memory --test l1_contradiction` (pre-implementación) | ❌ compile error `no field 'contradicts'` (E0609/E0560) — falla por la razón correcta |
| GREEN dedicado | `cargo nextest run -p vanta-memory --test l1_contradiction` | ✅ 11/11 passed (9 originales + 2 del review F3) |
| Suite scoped | `cargo nextest run -p vanta-memory --profile audit --build-jobs 2` | ✅ 587/587 passed (post-review-fixes) |
| Clippy | `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` | ✅ exit 0 |
| Fmt | `cargo fmt -p vanta-memory -- --check` | ✅ exit 0 |
| MCP compile | `cargo check -p vantadb-mcp --tests` | ✅ exit 0 |
| Docs | `node scripts/docs/check-links.mjs` · `check-docs.mjs` · `gen-index.mjs --check` · `scripts/validate-docs-coverage.ps1` | ✅ 0/0/all-clear/0 gaps |
| OCR | `pwsh dev-tools/ocr-review.ps1 -Format json` | ✅ advisory, reglas delegadas (14 archivos; sin sección de findings) |

## Steps

### Step 1: RED — test dedicado de contradicción en ingesta
- **Archivos:** `vanta-memory/tests/l1_contradiction.rs` (nuevo)
- **Acción:** escribir los tests del contrato: caso canónico (`write_memory` con `contradicts` marca el viejo con `superseded_by`, no lo borra), no-contradictorio intacto, ya-superseded saltado, skip no marca, self-ref ignorada, parse de `contradicts`, prompt documenta la regla conservadora, y e2e `run_l1_dedup` con runner scripted.
- **Verify:** `cargo nextest run -p vanta-memory --test l1_contradiction` → RED esperado (compile error: `no field 'contradicts'` — la funcionalidad no existe)
- **Estado:** ✅ COMPLETED

### Step 2: GREEN-A — campo `contradicts` en `DedupDecision` + literales
- **Archivos:** `vanta-memory/src/core/abstractions/types.rs`, `vanta-memory/src/core/record/l1_dedup.rs`, `vanta-memory/src/core/record/l1_batch.rs`, `vanta-memory/src/core/record/l1_writer.rs`, `vanta-memory/tests/{l1_dedup,generation_log,recall,pipeline_manager}.rs`, `vantadb-mcp/tests/test_query_embed.rs`
- **Acción:** agregar `#[serde(default)] pub contradicts: Vec<String>` (doc: ids explícitamente contradichos; flag, nunca delete) y `contradicts: vec![]` en los 11 sitios de construcción enumerados.
- **Verify:** `cargo check -p vanta-memory --all-targets` + `cargo check -p vantadb-mcp --tests`
- **Estado:** ✅ COMPLETED

### Step 3: GREEN-B — parse + prompts (un solo juicio LLM)
- **Archivos:** `vanta-memory/src/core/record/l1_dedup.rs` (`decision_from_value`), `vanta-memory/src/core/prompts/l1_dedup.rs` (system prompts chat/work), `vanta-memory/src/core/record/l1_batch.rs` (spec inline del objeto `dedup` + addendum)
- **Acción:** parsear `contradicts` con `string_array` (tolerante, default `[]`); agregar campo + regla conservadora ("solo negación EXPLÍCITA; duda → `[]`; un contradicho nunca va también en `target_ids`") a ambos system prompts y al spec inline del path fusionado.
- **Verify:** `cargo nextest run -p vanta-memory --test l1_contradiction` → tests de parse/prompt GREEN; tests de marcado aún RED
- **Estado:** ✅ COMPLETED

### Step 4: GREEN-C — marcado en `write_memory` (choke point)
- **Archivos:** `vanta-memory/src/core/record/l1_writer.rs`
- **Acción:** helper `mark_contradicted_targets` (reusa `lifecycle::mark_contradiction`; salta self, ids desconocidos y ya-superseded; persiste el viejo con su vector preservado vía `put_record`) + llamada en las ramas Store y Update/Merge tras persistir el nuevo registro (Skip no marca — no hay registro persistido que apunte).
- **Verify:** `cargo nextest run -p vanta-memory --test l1_contradiction` → GREEN
- **Estado:** ✅ COMPLETED

### Step 5: Docs — VANTA_MEMORY.md (Regla 3)
- **Archivos:** `docs/api/VANTA_MEMORY.md`
- **Acción:** documentar la detección de contradicción en ingesta L1 (campo `contradicts` del juicio de dedup; marcado `superseded_by` vía MEM-60; conservador; cuarentena diferida) en la sección de capas/operational modules.
- **Verify:** `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs`
- **Estado:** ✅ COMPLETED

### Step 6: VERIFY — suite scoped + fmt + clippy
- **Archivos:** (sin edición)
- **Acción:** correr el gate mecánico completo del scope.
- **Verify:** `cargo fmt --check` + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` + `cargo nextest run -p vanta-memory --profile audit` + `cargo check -p vantadb-mcp --tests`
- **Estado:** ✅ COMPLETED

### Step 7: CIERRE — OCR + review P2-01 + commit + campaign
- **Archivos:** task file + commit
- **Acción:** `pwsh dev-tools/ocr-review.ps1 -Format json` (advisory; Critical/High bloquean), review P2-01 por agente distinto (`vanta-review`), commit LOCAL `feat(memory):` con pathspec (⛔ nunca push), cierre campaign taskId `37`.
- **Verify:** `git log -1 --stat` (commit local) + veredicto review registrado
- **Estado:** ✅ COMPLETED

## Dependencias
- MEM-60 (`mark_contradiction`, `superseded_by`) — ✅ hecha, reutilizada.
- MEM-69 (`l1_batch` fused path) — ✅ hecha, punto de extensión del prompt.
- SCH-02/04 (confianza/validación) — ✅ hechas (contexto; no bloquean).
- MGR-13 §3.4 (cuarentena por contradicción → diferida a v1.0) — respetada (fuera de scope).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, contexto fresco — sesión `ses_ef5488be8ffeQFpNJUXFP3yc3Z`; reviewer_context ≠ author_context ✅)
- **Enfoque:** approach correcto (extender el juicio de dedup, 0 llamadas LLM nuevas; `mark_contradiction` sin semántica paralela; conservadurismo real verificado); alternativas evaluadas y descartadas con evidencia.
- **Cómo se probó:** re-ejecutó el test dedicado (9/9 al momento del review); verificó código directo (writer/prompts/lifecycle/dream intactos); gates docs. Suite/clippy no re-ejecutables en su corrida por disco lleno transitorio (ambiental) — re-ejecutados por el autor post-fixes: 587/587 + clippy 0.
- **Hallazgos:** 0 Critical / 0 Required; Low aplicados → F1 (semántica de fallo documentada), F2 (guarda self sanitizada + test), F3 (2 tests añadidos), F4 (precisión de docs), F5 (nota de índices). F6 no aplicado (modelo de confianza pre-existente de `target_ids`). F7 → `FIND-277` en Backlog.
- **Checklist anti-hábitos tóxicos** (verificado por el revisor — sin hallazgos tóxicos; única inconsistencia aceptable: Steps/Estado aún sin actualizar al momento del review):
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3): `SKILLS_CARGADAS` cubre el dominio (pinned test-driven-development + systematic-debugging incluidos).
- **Veredicto:** ✅ approve (2026-10-05)

## Notas
- `ponytail:` la detección es deliberadamente conservadora y heurística-LLM: solo contradicción explícita, marca revisable (no destructiva). Techo conocido: negaciones implícitas no se detectan; cross-sesión fuera (pool intra-sesión). Upgrade path: candidatos cross-sesión vía `list_namespaces` + recall semántico si aparece necesidad.
- Decisión de diseño: `contradicts` es ORTOGONAL a la acción; un contradicho nunca debe ir también en `target_ids` (si va, el delete del update/merge gana y el puntero no se escribe — el prompt lo prohíbe y el writer lo tolera saltando ids desconocidos tras el delete).
- SECURITY: ids de `contradicts` vienen del LLM (entrada no confiable) → usados como claves de lectura sanitizadas (`read_record` → `sanitize_key`); ids desconocidos/self se saltan; sin nueva red/deps/superficie de ejecución. No security-sensitive más allá de eso.
- PERFORMANCE: no toca hot path de búsqueda; costo extra en escritura = `is_empty()` (camino común) y O(contradicts) solo con señal explícita. Sin baseline requerido (Regla 9 no dispara: no hay optimización, no hay cambio de rendimiento en hot path).
- Fase SECURITY/PERFORMANCE evaluadas: aplica documentación mínima (arriba), sin cambios de dependencias → sin `cargo audit` requerido.
- Índices regenerados (`docs/index.md`, `llms.txt`, 2026-10-05): incluyen el catch-up pre-existente de MGR-12/13/19/MKT-05 (committed sin regenerar) y la entrada de `MEMG-02.md` (task file de otra sesión en vuelo, untracked al momento de este commit). El lead debe asegurar que `MEMG-02.md` se commitee; si no, regenerar índices en el push-prep (`check-links` verifica la referencia).
- Review P2-01 (vanta-review, contexto fresco, 2026-10-05): ✅ approve. Hallazgos Low aplicados: F2 (guarda self por storage-key sanitizada + test de variante "m/new"), F3 (2 tests: rama Update con contradict externo; vector del viejo preservado), F1 (semántica de fallo documentada en el helper), F4 (docs: provenance = tracing event, no persistencia audit), F5 (esta nota). F6 (validación `contradicts ⊆ pool`) no aplicado: mismo modelo de confianza pre-existente que `target_ids`. F7 → `FIND-277` (consumo read-side).
