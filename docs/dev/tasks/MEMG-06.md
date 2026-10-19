---
title: "TASK MEMG-06: Spill a disco con recall"
kind: task
description: "Spill a disco del contenido compactado (payload completo antes del stub `[compacted N chars]`, patrón offload/storage) + recall explícito por id/sesión (`spill/<session>`); enganchado al path real `assemble_with_recall`/worker como op opt-in (`ContextAssemblyConfig.spill_enabled`, default false); GC por retención reutilizando reglas del reclaimer; test round-trip sin pérdida; review P2-01"
---

# TASK MEMG-06: Spill a disco con recall

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 51, bloque F0-expandido L1464-1490)
- **Fuente:** plan Task 51 + Backlog fila MEMG-06 (research §7.5; Master #11) + `docs/dev/tasks/MEMG-20.md` (offload, patrón reutilizado)
- **Esfuerzo:** 🟠 3-5d | **Appetite:** max 1sem | **Stop (plan L1475):** 3-5d → spill+recall por sesión como op explícita + test + FIND del wiring automático en el worker
- **Prioridad:** 🟡
- **Tipo:** Rust — **feature-add** (vanta-memory; símbolos públicos nuevos sancionados por plan F0)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `51` en el campaign server)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY (§Spec: punto de enganche = path `assemble`, decidido por evidencia)
- **Pendientes (downhill):** 5 steps (1-5)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `51`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `assemble_with_recall`: 6 call sites (todos actualizados mecánicamente por la firma +1 param): `vanta-memory/src/services/pipeline_worker.rs:743` (worker — cablea el sink real), `desktop/src-tauri/src/commands/memory.rs:225` (None), `vantadb-mcp/src/context.rs:131` (None), `vanta-memory/tests/context_engine.rs:256,318` (None), `vanta-memory/tests/e2e_flow.rs:365` (None). `ContextAssemblyConfig`: literales en `vanta-memory/tests/e2e_flow.rs:488,555` (campo nuevo). `stub_message`: 1 caller (`mild_cascade`, engine.rs) + 1 test unitario (engine.rs:477). `iso_to_epoch_secs`/`SECS_PER_DAY`: 1 caller (`OffloadReclaimer::reclaim_as_of`) — pasan a `pub(crate)` (sin cambio de comportamiento). |
| Callees | `vanta-memory/src/context_engine/engine.rs` (`assemble_inner` nuevo privado; `mild_cascade`; `stub_message`), `context_engine/spill.rs` (NUEVO — `SpillStorage` sobre `vantadb::sdk::Embedded`, patrón `offload/storage.rs`), `context_engine/mod.rs` (exports), `services/pipeline_worker.rs` (`ContextAssemblyConfig` + `run_context_assembly_inner`), `offload/reclaimer.rs` (`MIN_RETENTION_DAYS` + `iso_to_epoch_secs` reutilizados), `core::conversation::now_ms` + `core::prompts::l1_extraction::epoch_ms_to_rfc3339` (timestamp ISO datable). |
| Implicaciones | +1 param en `assemble_with_recall` (API pública de vanta-memory, crate 0.x, pre-release — el plan F0 sanciona "enganchado al path real (`assemble_with_recall`/worker)"). `assemble` **NO cambia firma** (delega a `assemble_inner(..., None)` — cero ripple en sus callers). `IntegratedContext` **NO cambia** (sin payloads en el wire; sin riesgo serde de registros `__assembled` viejos). `ContextAssemblyConfig.spill_enabled` aditivo (default false → opt-in, sin cambio de semántica). 0 migración de datos, 0 deps nuevas. Escrituras nuevas solo si el host activa el opt-in. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `b66c4703`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `vanta-memory/src/context_engine/engine.rs` (:1-64 tipos+doc de pureza; :65-170 `assemble` 4 pasadas; :172-294 `IntegratedContext` + `assemble_with_recall` + `inject_recall_block`; :300-461 `unit_score`/`stub_message` (:312-322) / `mild_cascade` (:332-398, loop de stubs :382-385) / `aggressive_one_shot`; :463-486 tests unitarios — `stub_message` sin tests de sink).
  - `vanta-memory/src/context_engine/mod.rs` (:1-33 — exports; engine puro, mmd/report_store con SDK), `types.rs` (:1-118 — `ChatMessage{role,content,id}`, `ChatRole`, `ContextError`), `compressor.rs` (:35-74 — `msg_fingerprint`), `token_estimator.rs` (:40-129 — `estimate_*`, `truncate_content`, `build_units`, `emergency_truncate`), `mmd.rs` (:30-69 — `fingerprint` len+prefix, `save_active`), `report_store.rs` (:1-80 — patrón namespace `context/<s>/...`, append-only sin payload).
  - `vanta-memory/src/offload/storage.rs` (:1-97 COMPLETO — `OffloadStorage` patrón: namespace `offload/<s>`, key sanitizada, get-before-put dedup, read paginado skip-corrupt), `state_manager.rs` (:1-105 — `OffloadError` shape, cursor), `hooks/after_tool_call.rs` (:1-115 — hook sin callers productivos), `reclaimer.rs` (:1-143 — `MIN_RETENTION_DAYS=3`, `reclaim_as_of` cursor-safe, `iso_to_epoch_secs` :148-198, `SECS_PER_DAY` :35), `offload/mod.rs` (:1-23).
  - `vanta-memory/src/services/pipeline_worker.rs` (:1-100 imports; :160-196 `ContextAssemblyConfig`; :663-802 `run_context_assembly_inner` — caller real :743; :805-822 `TaskHandler::handle` L3→assembly :815).
  - `vanta-memory/tests/context_engine.rs` (:1-120 helpers + tests a/b/c; :240-288 MEM-37 aggressive; :290-344 cursor), `tests/e2e_flow.rs` (:350-377 assemble directo; :460-569 worker D19 con `ContextAssemblyConfig`), `vanta-memory/src/lib.rs` (:1-69 — exports), `vanta-memory/Cargo.toml` (:18-60 — deps: serde/serde_json/thiserror/tracing; sin deps nuevas necesarias), `vanta-memory/src/utils/sanitize.rs` (:17-31 — `sanitize_key`/`sanitize_component`), `core/prompts/l1_extraction.rs` (:52-75 `epoch_ms_to_rfc3339` con `Z` datable).
  - Docs/specs: plan Task 51 (L1464-1490), `docs/api/VANTA_MEMORY.md` (:586-596 Context engine — se actualiza en Step 4), `.opencode/rules/api-contract.md` (R-8), `.opencode/references/clean-code-clean-architecture.md` (Apéndice V), `question-gates.md` (Gate D), `prompts/findings.md`.
- **Archivos referenciados hacia dentro (imports/deps):** `assemble_with_recall` ← pipeline_worker/desktop/mcp/tests; `stub_message` ← `mild_cascade`; `iso_to_epoch_secs` ← `OffloadReclaimer`; `MIN_RETENTION_DAYS` ← `reclaim_as_of`; `ContextAssemblyConfig` ← worker + tests; `ChatMessage`/`ChatRole` ← spill.rs (nuevo).
- **Referencias entrantes (grep HEAD):** `spill` en `vanta-memory/src` = **0 hits** (solo comentario en `src/index/diskann.rs:17`, otro dominio); `AfterToolCallHook` = callers solo en su módulo (tests propios) → FIND-295; `assemble_with_recall(` = 6 call sites (listados arriba); `stub_message(` = 1 caller + 1 test; `ContextAssemblyConfig {` = 2 literales en tests.
- **Veredicto impacto:** **MEDIO** — feature-add en vanta-memory con superficie pública nueva **sancionada por plan F0** (Gate D pre-respondido: "enganchado al path real (`assemble_with_recall`/worker) o hook documentado; opt-in"; precedente MEMG-11/MEMG-13) + 3 call sites externos actualizados mecánicamente (desktop/mcp) + 2 tests existentes tocados por firma/literal. Sin cambio de wire persistido (`IntegratedContext` intacto), sin migración, sin deps. Pre-mortems del plan mitigados: (1) GC → `reclaim_as_of` reutiliza reglas del reclaimer (retención mínima + skip undatable) y se documenta por qué el cursor offload no aplica (contenido spilled ya consumido por definición); (2) solape con recall L1 → `recall`/`recall_session` son ops explícitas de `SpillStorage`, jamás en el path de `perform_auto_recall`; (3) hook sin callers → se elige el path `assemble` (evidencia: la pérdida ocurre en `stub_message`, solo `assemble` tiene el contenido original en ese instante) y el hook queda como FIND-295.

## Contrato

"Spill a disco del contenido compactado (persistir payload completo **antes** del stub `[compacted N chars]`, reutilizando el patrón `offload/storage`: records `spill/<session>`, key sanitizada, get-before-put dedup) + recuperación (recall de lo spilled **por id y por sesión**) con test round-trip (spill → recall devuelve contenido íntegro; sin pérdida de datos); enganchado al path real (`assemble_with_recall`/worker) con `ContextAssemblyConfig.spill_enabled` **opt-in default false** (sin cambio de semántica para quien no lo configura); GC por retención reutilizando las reglas del reclaimer (`MIN_RETENTION_DAYS` + `iso_to_epoch_secs`). Verify: `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` + focused (`--test context_engine`, `--test e2e_flow`) + `cargo fmt --check` + clippy scoped + `cargo check -p vantadb-mcp` y `-p vanta-desktop`(si aplica) por el call site. **Review P2-01 por agente distinto (vanta-review) con veredicto; hallazgos → FINDs (no fixes silenciosos).**"

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — el contrato del plan (Task 51, Gate Result ✅ DO, L1473) sanciona las superficies: spill a disco del contenido compactado + recall por id/sesión + enganche al path real (`assemble_with_recall`/worker) + opt-in. Los símbolos nuevos SON el mecanismo sancionado; nombres/firmas siguen convenciones del crate (micro-decisiones decidido-por-evidencia abajo). Precedente de campaña idéntico: MEMG-11 (`core_search` — MEMG-11.md:211 "pre-respondido por el plan F0"), MEMG-13 (MEMG-13.md:52 "los símbolos nuevos SON el mecanismo de consumo sancionado"). Sin símbolos fuera de la sanción.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Punto de enganche | A) **Path `assemble`** (sink en el instante del stub) (pro: `stub_message` es el ÚNICO frame que tiene el contenido original; determinista; cubre worker + cualquier host) / B) Hook `after_tool_call` (pro: ya existe; contra: opera sobre `ToolPair` pre-contexto — mecanismo distinto; 0 callers productivos; el contenido compactado no pasa por ahí) | ✅ **A** — decidido-por-evidencia: pérdida ocurre en `stub_message` (engine.rs:312-322) + plan L1486 ("punto de enganche: `assemble` vs hook → DISCOVERY") + contrato "enganchado al path real (`assemble_with_recall`/worker)". El hook queda FIND-295. |
| 2 | Mecanismo de captura | A) **`SpillSink` (trait puerto, definido en engine.rs) pasado como `Option<&mut dyn SpillSink>`; el engine reporta al sink el original en el instante del reemplazo (`stub_message` extrae el original con `mem::take` y lo entrega; el payload se persiste antes de que el stub sea durable — el registro `__assembled` se escribe después)** (pro: engine sigue puro (invariante engine.rs:10-14: no importa offload/DB); payloads NO viajan en `IntegratedContext` → wire persistido intacto) / B) colectar spilled en `AssembleOutput`/`IntegratedContext` y persistir post-hoc (pro: sin trait; contra: payloads en el wire `__assembled` (bloat) + `IntegratedContext` es serde persistido → riesgo de compat con registros viejos; persistencia post-stub) / C) diff input/output post-assemble (contra: ambigüedad de matching tras deletes aggressive; mensajes sin id irrecuperables) | ✅ **A** — decidido-por-evidencia: invariante de pureza (engine.rs:10-14) + `IntegratedContext` serde persistido (engine.rs:178-186) + patrón puerto/adaptador del crate. |
| 3 | Firma pública | A) **`assemble_with_recall` +1 param `spill: Option<&mut dyn SpillSink>`; `assemble` intacto** (delega a `assemble_inner` privado) (pro: ripple mínimo — solo los 6 call sites ya listados; `assemble` con 10 callers intacto) / B) param en ambos (contra: ripple 10+ callers sin consumidor nuevo) / C) funciones `_and_spill` duplicadas (contra: dual API — deuda P2-5) | ✅ **A** — decidido-por-evidencia: el contrato sanciona el path `assemble_with_recall`/worker (no `assemble` raw); R-8 (lógica en el crate, no en hosts); sin dual API. |
| 4 | Storage y claves | A) **`context_engine/spill.rs` — `SpillStorage` sobre SDK: namespace `spill/<sanitized-session>`, key = id sanitizado o `anon-<FNV1a64(content)>` para mensajes sin id; get-before-put (D19); payload JSON `SpilledMessage{message_id, session_key, role, content, spilled_at}`** (pro: espeja `offload/storage.rs` probado; dedup idempotente entre re-runs; sin deps) / B) archivo externo JSONL (contra: la crate persiste TODO en el store — lib.rs:10-13; prohibido storage externo) / C) key por índice posicional (contra: colisiona entre runs) | ✅ **A** — decidido-por-evidencia: patrón `offload/storage.rs:38-59` + lib.rs:10-13 ("All persistence lives in the VantaDB store — never external storage") + re-runs del worker son idempotentes (get-before-put). |
| 5 | Recall (semántica) | A) **Ops explícitas `recall(session,id)` + `recall_session(session)` en `SpillStorage`** (pro: separado del recall L1 — pre-mortem 2 del plan; el caller decide) / B) mezclar en `perform_auto_recall` (contra: pre-mortem 2 "no mezclar paths"; cambiaría semántica del recall de memoria) | ✅ **A** — decidido-por-evidencia: plan L1474 ("op separada/explícita (no mezclar en el path de recall de memoria)"). |
| 6 | GC | A) **`SpillStorage::reclaim_as_of`/`reclaim` reutilizando `MIN_RETENTION_DAYS` + `iso_to_epoch_secs` (pub(crate)) del reclaimer; sin gate de cursor** (pro: reutiliza reglas probadas; el cursor offload protege buffers NO consumidos — lo spilled YA fue consumido por definición; skip undatable conservador) / B) generalizar `OffloadReclaimer` a trait de entry (contra: refactor de módulo probado sin consumidor que lo pida) | ✅ **A** — decidido-por-evidencia: plan L1474/L1480 ("reutilizar `offload/reclaimer.rs` (cursor-safe) + política declarada") + reclaimer.rs:83-89/:100-116 (retención mínima + conservadurismo). Política declarada en el doc del módulo. |
| 7 | Opt-in | A) **`ContextAssemblyConfig.spill_enabled: bool` default false; el worker construye `DbSpillSink` solo si true** (pro: contrato "opt-in sin cambio de semántica"; ningún host existente cambia) / B) default true (contra: escrituras nuevas para todos los hosts sin pedirlo — no es opt-in) | ✅ **A** — decidido-por-evidencia: contrato del plan L1473 ("opt-in sin cambio de semántica para quien no lo configura"). |
| 8 | Alcance de captura | A) **Solo stubs mild (`stub_message`)** (pro: es el mecanismo que el contrato nombra "antes del stub"; deterministic) / B) + aggressive/emergency (contra: en el snapshot post-mild los mensajes ya stubeados tienen contenido stub — re-spill inútil; requiere refactor del pipeline de pasadas; expande test surface) | ✅ **A** — decidido-por-evidencia: contrato L1473 ("persistir payload completo antes del stub") + stop condition L1475 (entrega explícita + FIND). Techo declarado → **FIND-294**. |
| 9 | Fallo de spill | A) **warn-and-continue** (el sink loguea y el assembly sigue; la entrega de contexto no falla por el spill) (pro: spill es aid de recuperación, no path crítico; `run_context_assembly` mantiene "nothing partial is written") / B) propagar error → falla L3 (contra: un fallo de disco bloquearía la compactación entera) | ✅ **A** — decidido-por-evidencia: doc worker (pipeline_worker.rs:669-672 "the only failures are store-level") + report_store precedent (best-effort, :781-800). |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Engine puro:** `engine.rs` NO importa `offload` ni el SDK (invariante documentado engine.rs:10-14). El sink es un puerto (`SpillSink`) definido en el engine; la infraestructura (DB) vive en `spill.rs`/worker.
  2. **Wire persistido intacto:** `IntegratedContext` NO gana campos con payloads (registros `__assembled` viejos siguen deserializando); `assemble` conserva firma.
  3. **Semántica de contexto intacta:** con `spill_enabled=false` (default) el comportamiento es byte-idéntico (ningún record `spill/` se escribe; mismo `AssembledOutput`).
  4. **Recall separado:** `SpillStorage::recall*` jamás se invoca desde `perform_auto_recall` ni se inyecta en bloques de recall (pre-mortem 2).
  5. **Idempotencia D19:** re-spill del mismo mensaje (mismo id / mismo contenido anon) = no-op (get-before-put); re-runs del worker no duplican.
  6. **Conservadurismo GC:** retención < `MIN_RETENTION_DAYS` → no reclaim; timestamp no datable → no borrar (espeja reclaimer).
  7. **No tocar** `src/wal.rs`, `src/vector/`, `src/storage/` (dominio Arch/Engine); **no tocar** serialization/record/graph (MEMG-03 en vuelo).
  8. **WIP ajeno** (`opencode.jsonc`, master plan) NO se stagea; PROHIBIDO tocar `docs/pipeline-state.json`; commit con **pathspec**.
  9. **Sin `unwrap`/`expect`/`unsafe` en producción**; errores con `Result` + `thiserror`; tests pueden `unwrap/expect` con invariantes documentados (convención del repo).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --test context_engine --build-jobs 2` · `cargo nextest run --profile audit -p vanta-memory --test e2e_flow --build-jobs 2` · `cargo fmt --check` · `cargo clippy -p vanta-memory --all-targets -- -D warnings` · `cargo check -p vantadb-mcp`.
- **Deuda pendiente:** ninguna del contrato. Teccho declarado: aggressive/emergency no capturados (FIND-294); hook offload sin callers (FIND-295).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — sin `unsafe`, sin deps nuevas, sin wire roto. El cambio **elimina** la deuda "compactación irreversible / spill inexistente" (MEMG-06, 0 hits de `spill`) y agrega tests. Deuda aceptada y declarada: +1 arg a `assemble_with_recall` (11 args — la función ya excedía la heurística ≤3 del Apéndice V.4; agrupar params sería refactor fuera de scope con ripple mayor — se declara, no se paga en este PR; `ponytail:` tag en el doc de la firma); GC como op explícita sin scheduler in-repo (mismo estado que `OffloadReclaimer`, también sin callers — wiring futuro del host). `NOTICED BUT NOT TOUCHING`: aggressive/emergency sin captura → **FIND-294**; `AfterToolCallHook` sin callers productivos → **FIND-295**.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: spill→recall round-trip por id y por sesión (contenido íntegro), captura ANTES del stub (sink delivery test), opt-in default false (byte-idéntico sin config), worker cableado (e2e con `spill_enabled:true`), GC por retención reutilizando reglas del reclaimer; tests RED→GREEN + suite scoped + fmt/clippy verdes + review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | Changelog (minor) vía release-plz (`feat:`); doc API `docs/api/VANTA_MEMORY.md` actualizado en el mismo PR (Regla 3) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius: `assemble_with_recall`/`stub_message`/`mild_cascade`/`ContextAssemblyConfig`) + `codebase-memory-mcp_check_index_coverage` (paths clave: `no_recorded_issue` ✅; freshness `metadata_changed` → fuentes leídas directo) + grep puntual
- `cargo nextest` scoped por crate (`-p vanta-memory`, focused por `--test`) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs (`node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --check`) — task file + VANTA_MEMORY.md editados

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `source-driven-development` · `doubt-driven-development` · `incremental-implementation` · `test-driven-development` (SDP phase=BUILD) + `deprecation-and-migration` (pinned storage/schema) + rol: `rust-write-tests` (tests RED→GREEN + determinismo), `performance-optimization` (hot path `engine.rs` — Regla 9), `documentation-skill` (task file + docs/api bajo `docs/`). `security-and-hardening` excluida con justificación (ver Fase SECURITY).

## Steps atómicos (PLAN → ACT → VERIFY)

> ~100 líneas por step; cada step deja el repo compilable y testeable. TDD: RED → GREEN → VERIFY.

### Step 1 — Engine: puerto `SpillSink` + captura antes del stub (RED→GREEN)
- **PLAN:** firma objetivo: `pub trait SpillSink { fn spill(&mut self, message: &ChatMessage, original: &str); }` (engine.rs); `stub_message` → `Option<String>` (original si reemplazó); `assemble` delega a `assemble_inner(..., spill)` privado; `assemble_with_recall` + param `spill: Option<&mut dyn SpillSink>`; actualizar 6 call sites con `None`.
- **ACT (RED):** tests unitarios en engine.rs: (a) `spill_sink_receives_original_before_stub` — collector sink recibe, por cada mensaje stubeado, el contenido original exacto (y el output contiene el stub); (b) sin compaction (ratio gate) el sink no recibe nada; (c) `stub_message` devuelve `Some(original)`/`None` (reemplaza el test :477). Deben fallar por la razón correcta (API no existe → no compila).
- **ACT (GREEN):** implementar; actualizar `stub_guard_reverts_when_stub_not_shorter`; call sites `None`.
- **VERIFY:** `cargo nextest run --profile audit -p vanta-memory --lib context_engine --build-jobs 2` ✅ + `cargo check -p vanta-memory --all-targets` ✅.

### Step 2 — `context_engine/spill.rs`: storage + recall + GC (RED→GREEN)
- **PLAN:** `SpilledMessage{message_id, session_key, role: ChatRole, content, spilled_at}`; `SpillStorage::{new, spill, recall, recall_session, reclaim_as_of, reclaim}`; `DbSpillSink` (impl `SpillSink`, warn-and-continue, contador); `SpillError`; key = `sanitize_key(id)` o `anon-<fnv1a64>`; namespace `spill/<sanitized-session>`; exports en mod.rs.
- **ACT (RED):** tests en spill.rs: round-trip por id; recall_session lista; dedup D19; aislamiento entre sesiones; payload corrupto se saltea; `reclaim_as_of` respeta `MIN_RETENTION_DAYS`, borra solo vencidos, conserva undatables; sink persiste + contador. Deben fallar por la razón correcta.
- **ACT (GREEN):** implementar; `iso_to_epoch_secs`/`SECS_PER_DAY` → `pub(crate)` en reclaimer.rs (sin cambio de lógica).
- **VERIFY:** `cargo nextest run --profile audit -p vanta-memory --lib context_engine::spill --build-jobs 2` ✅.

### Step 3 — Worker: opt-in + cableado del sink real (RED→GREEN)
- **PLAN:** `ContextAssemblyConfig.spill_enabled: bool` (default false); `run_context_assembly_inner` construye `SpillStorage`+`DbSpillSink` solo si enabled y lo pasa a `assemble_with_recall`; log debug con `spilled_count`.
- **ACT (RED):** test e2e en `tests/e2e_flow.rs`: (a) worker con `spill_enabled:true` + historial gordo → `recall_session` devuelve los originales (round-trip; contenido íntegro matchea los L0) y el assembled tiene stubs; (b) default false → 0 records `spill/`. Debe fallar por la razón correcta.
- **ACT (GREEN):** implementar + actualizar literal `ContextAssemblyConfig` :488.
- **VERIFY:** `cargo nextest run --profile audit -p vanta-memory --test e2e_flow --build-jobs 2` ✅.

### Step 4 — Docs API + FINDs + deuda declarada
- **PLAN:** `docs/api/VANTA_MEMORY.md` §Context engine: firma actualizada + `SpillSink`/`SpillStorage`/recall/opt-in/GC; Backlog: FIND-294 (aggressive/emergency sin captura) + FIND-295 (hook offload sin callers).
- **ACT:** editar; verificar formato de tabla (backlog-format.md) y links docs.
- **VERIFY:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` ✅.

### Step 5 — Cierre: verify full + OCR + review P2-01 + commit
- **PLAN:** verify full del contrato; OCR delegation; review P2-01 (vanta-review, agente distinto); commit local pathspec; campaign completed (taskId 51) con payload review.
- **ACT/VERIFY:** comandos del cierre (abajo). 

## Verification contract (cierre)

1. `cargo fmt --check` ✅
2. `cargo clippy -p vanta-memory --all-targets -- -D warnings` ✅ (+ `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` por call site)
3. `cargo nextest run --profile audit -p vanta-memory --build-jobs 2` ✅ (suite scoped del crate completo)
4. `cargo check -p vantadb-mcp` ✅ (call site externo; desktop solo si el crate compila en el entorno — si el linker/tauri falla, `dev-tools/target-cleanup.ps1 -Clean -Yes` y reintento; si persiste → documentar y FIND)
5. `pwsh dev-tools/ocr-review.ps1 -Format json` — Critical/High bloquean; Medium → FIND
6. Review P2-01: `vanta-review` (fresh context, agente distinto) con veredicto approve/changes
7. Commit **LOCAL** `feat(memory): MEMG-06 — spill a disco con recall` con pathspec de archivos propios
8. Campaign: `campaign_update_task_state(taskId:"51", completed, recitation + review payload)`

## Review P2-01 (plan)

- **Reviewer:** `vanta-review` (fresh context; reviewer_context ≠ author_context).
- **Foco:** (a) captura ANTES del stub (no pérdida); (b) wire/serde intactos (`IntegratedContext` sin cambios; registros viejos deserializan); (c) opt-in default false byte-idéntico; (d) dedup D19 e idempotencia entre re-runs; (e) GC conservador (retención mínima + undatable); (f) recall separado del path L1; (g) sin unwrap/unsafe en producción; (h) spec válida (Gate D pre-respondido con evidencia).
- **Fallback sin subagentes:** `doubt-driven-development` degradado (marcado como degradado) + escalado al owner.

### Resultado (2026-10-05, sesión `ses_ef2d63433ffeQP02M1S5njyUWV`)

- **VERDICT: ✅ APPROVE.** OCR: ninguno Critical/High. Spot-check re-ejecutado por el reviewer: `--lib spill` 12/12 ✅ · `--test e2e_flow -E test(memg06)` 2/2 ✅ · `cargo fmt -p vanta-memory -- --check` ✅ · clippy lib ✅ · docs gates ✅.
- **Findings (5 Low, ninguno bloquea):**
  1. [Low] `engine.rs:446-449` — orden literal sink↔stub (el sink se invoca con el stub ya en `msg.content`; el original viaja por valor y se persiste antes de que el stub sea durable) → **corregida la redacción de §Spec #2** (precisión; sin cambio de código — el trait doc ya era consistente).
  2. [Low] `spill.rs` doc de `scanned` heredaba wording del reclaimer → **corregida** ("corrupt payloads were already skipped during the read").
  3. [Low] `spill.rs` naming `message_id` = effective key (incluye `anon-<hash>`) → documentado, se mantiene (Low descartado por rúbrica).
  4. [Low] e2e sin assert de completitud (spilled.len() vs stubs) → cubierto a nivel unit (`spill_sink_receives_original_before_stub`: events == stub_count); gap residual descartado por rúbrica.
  5. [Low] `reclaim` sin call site productivo → espeja `OffloadReclaimer` (sin callers); declarado en Deuda técnica (wiring futuro del host).
- **Nota del reviewer:** `docs/api/EMBEDDED_SDK.md` modificado en el worktree es de DX-04 (ajeno) — el commit con pathspec NO lo barre.

## Context Save Point

- **DISCOVERY ✅ (2026-10-05):** plan Task 51 leído; task file creado; evidencia verificada HEAD (`stub_message` :312-322, `spill` 0 hits, `AfterToolCallHook` 0 callers, `assemble_with_recall` 6 call sites); Gate D pre-respondido (precedente MEMG-11/13); decisiones #1-#9 en §Spec.
- **Steps:** 1 ✅ · 2 ✅ · 3 ✅ · 4 ✅ · 5 ⏳ (review + commit + campaign)
- **Step 1 ✅ (RED→GREEN):** RED = E0405 `SpillSink` no existe; GREEN = trait `SpillSink` + `assemble_inner` privado + `stub_message -> Option<String>` + `mild_cascade(..., spill)` + `assemble_with_recall` +1 param + 6 call sites `None`. Evidencia: `cargo check -p vanta-memory --all-targets` ✅; lib tests 388/388 ✅; focused `spill_sink` 2/2 ✅, `stub_guard` 1/1 ✅.
- **Step 2 ✅ (RED→GREEN):** RED = E0603/E0425/E0433 (módulo faltante); GREEN = `context_engine/spill.rs` (`SpilledMessage`, `SpillError`, `SpillReclaimStats`, `SpillStorage::{spill,recall,recall_session,reclaim,reclaim_as_of}`, `DbSpillSink`, FNV-1a anon keys) + `iso_to_epoch_secs`/`SECS_PER_DAY` → `pub(crate)` + exports. Evidencia: focused `spill` 12/12 ✅.
- **Step 3 ✅ (RED→GREEN):** RED = E0560 `spill_enabled` no existe; GREEN = `ContextAssemblyConfig.spill_enabled` (default false) + sink cableado en `run_context_assembly_inner` + log `spilled` + literal D19 actualizado. Evidencia: `--test e2e_flow` 8/8 ✅ (memg06 on/off incluidos); `--features precise-tokens` memg06 2/2 ✅.
- **Step 4 ✅:** `docs/api/VANTA_MEMORY.md` (sección Spill) + FIND-294/FIND-295 en Backlog + index regenerado (`gen-index --write` → `--check` exit 0) + links/docs gates exit 0.
- **Gates hasta ahora:** `cargo check -p vanta-memory --all-targets` ✅ · suite scoped `-p vanta-memory` 684/684 (2 skipped) ✅ · `cargo clippy -p vanta-memory --all-targets -- -D warnings` ✅ · `cargo check -p vantadb-mcp` ✅ · `cargo check --manifest-path desktop/src-tauri/Cargo.toml` ✅ (3m09s) · fmt scoped de archivos propios ✅ (repo-wide falla por archivos de MEMG-03 en vuelo — fuera de mi pathspec).
- **Próximo paso:** Step 5 — review P2-01 (`vanta-review`) + commit LOCAL + campaign completed (taskId 51).

## RESULTADO (§7 — al cierre)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: a76890f4 (feat) + commit docs(task) de este archivo
ARCHIVOS: vanta-memory/src/context_engine/{engine,mod,spill}.rs · vanta-memory/src/offload/reclaimer.rs · vanta-memory/src/services/pipeline_worker.rs · vanta-memory/tests/{context_engine,e2e_flow}.rs · vantadb-mcp/src/context.rs · desktop/src-tauri/src/commands/memory.rs · docs/api/VANTA_MEMORY.md · docs/dev/Backlog.md (FIND-294/295) · docs/dev/tasks/MEMG-06.md · docs/index.md · llms.txt
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(plan F0 sancionó superficie) D:no(pre-respondido por plan F0; precedente MEMG-11/13) V:no(verify sin fallas) C:no(sin colaterales; WIP ajeno respetado con pathspec)
SKILLS_CARGADAS: campaign-executor · progreso · ponytail (base auto) · source-driven-development · doubt-driven-development · incremental-implementation · test-driven-development · deprecation-and-migration (pinned) · rust-write-tests · performance-optimization · documentation-skill
```
