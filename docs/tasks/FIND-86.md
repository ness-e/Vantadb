# FIND-86 — wiring MEM-69 + tool 77 + números MEM-70

## Metadata
- **Plan file:** `docs/plans/2026-09-15-find-correcciones.md` (Task 26, Wave8)
- **Creado:** 2026-09-16 (DISCOVERY completo — task file NO existía)
- **last-synced:** 2026-09-16 (cierre Step 3)
- **Estado:** ✅ COMPLETED (Steps 1-3 ✅ · Gate D resuelto A/A/B vía SARL 2026-09-16)
- **Appetite / Branch / Commit:** 2d / develop / `feat: FIND-86`
- **Ruta:** vanta-worker
- **SDP:** `campaign_discover_skills_v2` phase=BUILD keywords=[memory-wiring, pipeline-worker, mcp-tool, mem-numbers] → 8 skills (ver §5)

## 1. TAREA
- **Objetivo:** cablear la memoria diferida diseñada (ADR-040) hoy sin cablear: dream "not wired yet" → `pipeline_worker`, batch "pipeline_worker untouched" → wiring, `scene_consolidate` follow-up → tool MCP 77 (diseñada o implementada); MEM-70 números medidos con harness o DEFER-ratificado si no hay harness. Orden wiring→tool→números; 1 sub-item trancado NO bloquea el resto (ship parcial + DEFER-ratificado del resto).
- **Contrato (plan, textual):** MEM-69 wiring + tool 77 diseñada o implementada + MEM-70 números o DEFER-ratificado + suite 336 verde + clippy 0.
- **Acceptance criteria:**
  1. `dream::consolidate_session` alcanzable desde `MemoryTaskHandler`/`PipelineWorker` (nuevo `TaskKind` o método documentado) + test que lo prueba.
  2. `l1_batch::extract_dedup_batch` usado por el worker (opt-in o default documentado) + test que prueba 1 llamada vs 2.
  3. `scene_consolidate` diseñada (ADR-040 ya la diseña → cuenta) o implementada como tool MCP (conteos actualizados + tests).
  4. MEM-70: números medidos con harness o DEFER-ratificado con evidencia.
  5. `cargo test -p vanta-memory -j 2` verde + `cargo clippy -p vanta-memory --all-targets -- -D warnings` 0 warnings.
- **Estado de ejecución:** criterios 1 (dream wiring ✅ `TaskKind::Dream` + `run_dream`), 2 (batch opt-in ✅ `with_use_batch(false)` + `run_l1_batch`), 3 (diseñada ✅ ADR-040, D3=B no implementar), 4 (DEFER-ratificado ✅) y 5 (✅ 541/0 + clippy 0) cumplidos. Gate D resuelto A/A/B por orquestador vía SARL 2026-09-16.

## 2. ARCHIVOS
### Clave (paths REALES verificados en disco — divergencia menor vs plan: el plan cita `core/record/l1_batch.rs` sin prefijo; los reales viven bajo `vanta-memory/src/` → HALLAZGO H1, no bloquea)
- `vanta-memory/src/services/pipeline_worker.rs` — worker L0→L3 (`MemoryTaskHandler::handle` :701-717 solo `L1|L2|L3|Flush`; `run_l1` :386-462 usa `extract_l1_segments` :428 + `run_l1_dedup` :441, NUNCA `extract_dedup_batch`; `run_l2` :464, `run_l3` :515, `run_context_assembly` :570)
- `vanta-memory/src/core/record/l1_batch.rs` — primitiva MEM-69 (`extract_dedup_batch` :50, `EXTRACT_DEDUP_TASK_ID` :36, doc "pipeline_worker is untouched" :13-14)
- `vanta-memory/src/core/scene/auto_consolidate.rs` — `auto_consolidate` :92, `extract_local` :63, `LocalTurn` :43, follow-up tool en :18-19
- `vanta-memory/src/core/dream/mod.rs` — `consolidate_session` :629, `detect_idle` :266, `DreamConfig` :78, `Dreamer` trait :241, doc "deliberately not wired" :31-36, stub `promote_dream_run` :614
- `vanta-memory/src/core/state/types.rs` — `TaskKind` :38 (`L1|L2|L3|Flush`, SIN variante Dream/Batch)
- `vanta-memory/src/gateway/mod.rs` + `vanta-memory/src/gateway/knowledge_handlers.rs` — solo `scene_read/list/query` (3 tools, SIN `scene_consolidate`)
- `vantadb-mcp/src/handlers/tools.rs` — conteos 79 (:25-26, :1090), `scene_tools` = 3 (:1177), dispatch scene (:2828-2829); `vantadb-mcp/src/config.rs` :11,14 (full = 79 tools)
- Suite: `vanta-memory/tests/` (25 files) + lib tests (336 citados → verificados exactos hoy, ver §6)
### Relacionados (callers/callees vía codegraph_explore + grep)
- Callers `consolidate_session`: 6 (tests `vanta-memory/tests/dreaming.rs`; NINGÚN caller en `pipeline_worker.rs` — grep `TaskKind::` solo :704-708 + `conversation_hook.rs:77` + `pipeline_manager.rs:140` + `stateful_pipeline_manager.rs:55,73`, todos `L1`)
- Callees worker: `L0Recorder`, `extract_l1_segments`, `run_l1_dedup`, `extract_scenes_with_llm`, `evaluate_persona_trigger/generate_persona`, `assemble_with_recall`, `CheckpointManager`, `OffloadStateManager`
- MEM-69 refs: `l1_dedup.rs:259`, `l1_extractor.rs:51`, `l1_parser.rs:156`, `record/mod.rs:27-28,36`, `tests/l1_batch.rs:3`
- MEM-70 refs: `evals/memory_bench.py` (harness, seed 42), `docs/operations/BENCHMARKS.md` §17 (:904-956), `benchmarks/README.md:78`
- ADR-040: `docs/architecture/adr/ADR-040-auto-consolidate-local-first.md` (:36-41 diseño tool 77, :51 dream→L1 en MEM-65)
- MEM-65 real: `.opencode/skills/campaign-executor/tasks/MEM-65.md:1` (telemetría por capa + pLimit, NO dream-wiring → HALLAZGO H2)
### Prohibidos (WIP ajeno — detect_changes + `git status` 2026-09-16, NO tocar)
- `.opencode/`, `Justfile`, `completions/_vanta-cli*` (3 files), `desktop/src-tauri/Cargo.lock`, `.github/workflows/ocr-delegate.yml` (untracked), `dev-tools/ocr-review.ps1` (untracked), `reparacion.bat` (untracked), `docs/pipeline-state.json`, plan file (solo recitation orquestador), stash@{0..14}, FIND-74 (`examples/`, QUICKSTART, README raíz — 5e428aea), FIND-72 (`benchmarks/`), FIND-93 (`src/storage/engine/txn.rs:158` — dead_code pre-existente, ticket ya creado por FIND-65)

## 3. DEPENDENCIAS
- **Wave8** (plan): Wave0-7 DONE + FIND-74 DONE 5e428aea. Sin bloqueantes para DISCOVERY/verify.
- **Paralelas:** FIND-74 ✅ (disjunto: examples/docs), FIND-72 (disjunta: benchmarks/ — NO absorber).
- **Previa:** FIND-80 ✅ (fuzz, disjunto).
- **Next:** FIND-72, luego Wave9 (FIND-92/93 + FIND-94 sin wave — NO absorber salvo HALLAZGO explícito; H6 confirma FIND-93 ya existe y no se toca).
- **Memoria:** MEM-61 (dream primitiva standalone ✅), MEM-69 (batch primitiva ✅ e1f7daef-era), MEM-70 (harness + DEFER ✅), MEM-65 (telemetría ✅ — NO fue wiring), MCP-41 (auto_consolidate + ADR-040 ✅ 6bf42a89).

## 4. REFERENCIAS
- **ADR-040** (`docs/architecture/adr/ADR-040-auto-consolidate-local-first.md`): pipeline extract→consolidate→recall 100% local-first; "NO se añade MCP tool en este slice" (:36) + follow-up diseñado `scene_consolidate(session_key, turns)` write-tool, perfil Full, conteos 76→77 + tests `scene_tests.rs` (:38-41). → Criterio 3 YA cumplido como "diseñada".
- **MEM-69** (primitiva existe, wiring pendiente): `l1_batch.rs:13-14` "pipeline_worker is untouched (wiring is a follow-up slice)". Reutiliza tolerancias exactas (`memory_from_value`, `decision_from_value`), degradación store-by-default.
- **MEM-70** (DEFER-ratificado YA existe — HALLAZGO H3: plan dice "sin rastro", stale): harness `evals/memory_bench.py` (seed 42, `--no-vantadb` smoke) + `BENCHMARKS.md` §17 (tabla smoke dict-fallback 20×16 recall@5=1.0 TECHO declarado + entorno Regla 11 + Pendiente DEFER 1-3 con loader versionado). Smoke re-corrido HOY: 4×8 recall@5=1.0, p50 0.007ms, p99 0.015ms (fallback, no claim de calidad).
- **Regla 8 (concurrencia):** worker toca locks por sesión (`acquire_lock/release_lock` :274-290, TTL `lock_ttl_ms`, claim/lease `owner`, `reclaim_stale`) + `LocalStateBackend` con UN solo `std::sync::Mutex` (`local_backend.rs:13,41,52,235`). Grep `dashmap|parking_lot|tokio` en `vanta-memory/src/utils/` = 0 hits (solo std Mutex). Veredicto: **N/A justificado para el scope verificado** (sin multi-índice, sin dashmap/parking_lot/Tokio en el crate); el wiring PROPOSTO no cambia el lock-order (cada task corre bajo el lock de sesión ya existente; dream/batch son fases dentro del handler, sin locks nuevos). Si el orquestador elige variante con hilos nuevos → re-evaluar + stress (vanta-chaos no disponible en este contexto → documentado, no ejecutado).
- **Regla 9 (no optimizar sin medir):** el wiring batch REDUCE llamadas LLM (2→1, −40/50% tokens por flush según doc :5-6) = claim de mejora → exige before/after. No hay bench del módulo con LLM real (tests usan `ScriptedRunner` mock por diseño). Salida honesta: wiring opt-in (default conserva path 2-llamadas) + medición vía contador `task_id` (`l1-extraction` vs `l1-extract-dedup`) en tests, SIN claim P99 hasta harness con runner real. MEM-70 sintético NO mide el worker (mide recall keyword) → no usarlo como before/after del wiring.
- **Símbolos públicos nuevos propuestos** (disparan Gate D — ver BLOQUEO): `TaskKind::Dream` (+ payload `last_active_at_ms` o reutilizar `created_at_ms`), `TaskKind::L1Batch` o flag `use_batch` en `MemoryTaskHandler`/`L1DedupConfig`, `scene_consolidate` gateway + MCP tool (sería tool 80, no 77 — H4). Tabla Spec abajo.

## Spec (gate mecánico spec-first — LLENA por tabla de decisiones; `question` pendiente del orquestador, worker sin esa tool)
| # | Decisión | A | B | C | Default recomendado | Resuelto |
|---|----------|---|---|---|---------------------|----------|
| D1 | Forma del wiring dream→worker | Nuevo `TaskKind::Dream` + rama `handle` que llama `consolidate_session` con `DreamConfig::default()` (idle via `detect_idle(now, last_active, threshold)`; `last_active` desde `PipelineSessionState.last_active_time_ms`) | Método público `MemoryTaskHandler::run_dream(session, now, last_active, config)` sin variante enum (el host encola/llama directo) | DEFER: solo diseño (este file) + ticket sigue ⬜ | **A** (encaja en claim/lease/retry/dead-letter existentes; B deja scheduling al host) | ✅ A orquestador SARL 2026-09-16 |
| D2 | Forma del wiring batch→worker | Flag opt-in `use_batch: bool` (default `false`) en `MemoryTaskHandler` (o `L1DedupConfig`); `run_l1_inner` llama `extract_dedup_batch` cuando `true`, si no path actual intacto | Switch incondicional a batch (2 llamadas → 1 siempre) | DEFER: primitiva queda opt-in sin caller | **A** (safe default Regla incremental-4; B cambia comportamiento hot path sin before/after) | ✅ A orquestador SARL 2026-09-16 (flag en handler + builder `with_use_batch`, `new()` intacto) |
| D3 | Tool `scene_consolidate` | Implementar gateway `scene_consolidate(session_key, turns)` + MCP tool Full (conteos 79→80 en `tools.rs:25-26,1090` + `config.rs:11,14` + tests perfil) | Ratificar diseñada (ADR-040:38-41) y NO implementar (appetite; MCP-37 perfiles intactos) | Dividir en FIND nuevo (tool sola) | **B** (contrato acepta "diseñada"; A suma superficie + recount + e2e) | ✅ B orquestador SARL 2026-09-16 |
| D4 | Números MEM-70 | Re-correr harness sintético (hecho hoy §6) y declarar medido-sintético | Ratificar DEFER existente (BENCHMARKS §17 Pendiente 1-3: loader versionado + backend vantadb real) | Medir wiring batch con runner real (requiere LLM key/red — fuera de appetite) | **B** (A ya hecho como evidencia; C fuera de scope) | ✅ B (evidencia §6) |

## 5. SKILLS (SDP — `campaign_discover_skills_v2` phase=BUILD, keywords memory-wiring/pipeline-worker/mcp-tool/mem-numbers, 8 devueltas)
- `campaign-executor` — state machine PLAN→ACT→VERIFY + task file + recitation (siempre).
- `source-driven-development` — verificar APIs del crate en fuente (Cargo.toml, código), no memoria del modelo.
- `doubt-driven-development` — wiring toca invariantes no verificables por tipos (no-mutación L1, degradación P4) → revisión adversarial.
- `incremental-implementation` — slices verticales wiring→tool→números, ship parcial + DEFER.
- `test-driven-development` — RED primero para cada wiring (reproduction/mock `ScriptedRunner`); suite 336 como red de seguridad.
- `context-engineering` — jerarquía Rules→Spec→Source→Error aplicada en DISCOVERY (§7).
- `api-and-interface-design` — `TaskKind` es contrato público (serde UPPERCASE, Hyrum) + tool MCP con hints (readOnly/idempotent).
- `frontend-ui-engineering` — SDP la devolvió por lifecycle genérico; **N/A** (0 archivos `web/` en scope; se declara y no se carga).
- Base auto: `progreso` (Backlog→avance NO tocado aquí por race paralelo — orquestador).

## 6. HERRAMIENTAS+MCP
Comandos exactos (cargo SIEMPRE `-j 2`, timeouts generosos; `campaign_verify_cmd` con bug exit -1 conocido → bash directa + anotado):
- `cargo check -p vanta-memory -j 2` — ✅ 28.97s (1 warning pre-existente `vantadb` `txn.rs:158` `cloned_sole_buffer` never used = FIND-93, fuera de scope, no tocado)
- `cargo clippy -p vanta-memory --all-targets -j 2 -- -D warnings` — ✅ 37.49s, 0 warnings
- `cargo test -p vanta-memory -j 2` — ✅ lib **336 passed / 0 failed** (criterio "suite 336" = lib tests, exacto) + integración 201 passed / 0 failed + doc 1/1 → **538 total, 0 failed**
- `cargo fmt -p vanta-memory -- --check` — ✅ limpio
- `git diff --check` — ✅ limpio (solo warning CRLF en WIP ajeno `completions/_vanta-cli.ps1`, no error)
- `python evals/memory_bench.py --sessions 4 --turns 8 --queries 8 --no-vantadb` — ✅ recall@5=1.0 (techo fallback declarado), p50 0.007ms p99 0.015ms, reporte JSON (gitignored) regenerado
- MCP: `codegraph_explore` (blast radius dream/batch/TaskKind — 33 símbolos/3 files) · `detect_changes` scope=impact direction=inbound depth=3 (merge_base 5e428aea; changed_files = 13 WIP ajeno; impacted=1 `dev-tools/ocr-review.ps1` — confirma prohibidos) · `get_architecture` path=vanta-memory (2542 nodos/10913 edges; entry `vanta-seed`; cluster 73 `consolidate_session`) · `check_index_coverage` 5 paths clave (no_recorded_issue; freshness metadata_changed → fuentes leídas directo, citadas con :línea) · `campaign_detect_task_type` (rust/Rust core) · `campaign_discover_skills_v2` (8 skills arriba) · `campaign_verify_cmd` NO usado (bug exit -1; fallback bash documentado aquí)
- Sin grep-loop donde codegraph respondió; grep solo para literales (MEM-*/ADR-040, `TaskKind::`, primitivas de concurrencia).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY)
- **dream→worker:** `consolidate_session(db, session, now, last_active, config)` (:629) es el punto de wiring (el doc :628 lo nombra explícito). Entradas disponibles en el worker: `session_id` (task), `now_ms` (`backend.now_ms()` / `conversation::now_ms()`), `last_active` (`PipelineSessionState.last_active_time_ms` vía `CheckpointManager` — ya usado en `run_l1_inner` :419-426). Salida: `DreamRun` a `dream/<s>/<run_id>` (nunca muta `l1/<s>` — invariante + tests `dreaming.rs`). Implicación: rama nueva en `handle` + variante `TaskKind` (serde UPPERCASE → wire-format nuevo; matches exhaustivos: solo 1 `match task.kind` en el crate :701 → blast controlado). Riesgo: scope triple (dream+batch+tool en 1 task) → orden + ship parcial.
- **batch→worker:** `run_l1_inner` hace 2 llamadas (extract :428 + dedup :441-449 con `write` implícito vía `run_l1_dedup`). `extract_dedup_batch` (:50) fusiona en 1 (`task_id` estable `l1-extract-dedup` :36) y devuelve `(result, pending, decisions)` SIN escribir (el caller escribe vía `apply_dedup_batch`). Wiring = elegir llamada + aplicar batch + checkpoints (`add_memories_extracted`, runner state) idénticos. Implicación: flag opt-in conserva default; tests con `ScriptedRunner` pueden asertar 1 llamada (RED natural).
- **tool:** gateway sin `scene_consolidate` (mod :1-19 solo re-exporta 3 scenes); MCP `scenes.rs` 3 tools read-only; Full=79 (`tools.rs`, `config.rs`). Añadir write-tool (readOnlyHint false, idempotentHint true según ADR-040) toca definiciones + dispatch + conteos + perfiles + tests → el slice más ancho; por eso default B (ratificar diseñada).
- **Concurrencia:** ver §4 Regla 8 (N/A justificado, single Mutex, sin locks nuevos en el diseño).
- **Suite:** 336 lib + 201 integración + 1 doc, todo verde hoy (red de seguridad para el ACT futuro).

## 8. INVESTIGACIÓN PROBLEMA
- **Triple wiring pendiente en código (confirmado línea-exacto):** `dream/mod.rs:31` "deliberately not wired into pipeline_worker.rs yet" (+ reserva a MEM-65 que en realidad fue telemetría — H2) · `l1_batch.rs:13` "pipeline_worker is untouched (wiring is a follow-up slice)" · `auto_consolidate.rs:18-19` "follow-up: MCP scene_consolidate design in ADR-040" · `TaskKind` sin variantes nuevas · gateway/MCP sin `scene_consolidate` · MEM-70 con harness + DEFER (no "sin rastro" — H3).
- **Hipótesis:** diseño ADR-040/MEM-61/MEM-69 sin cablear porque cada primitiva shipeó standalone con integración reservada a un slice posterior que cambió de contenido (MEM-65 → telemetría) o nunca se scheduló. El plan 2026-09-15 agrupa los 3 restos + números en FIND-86.
- **Tradeoff implementar-tool vs diseñar-tool:** implementar (A) cierra el loop extract→consolidate→recall vía MCP pero toca superficie pública + perfiles + conteos (79→80) y e2e; diseñar (B) es salida válida por contrato ("diseñada o implementada") y ya existe (ADR-040:38-41) — Ponytail full elige B + D3 al orquestador.
- **Stop aplicado:** wiring (D1/D2) trancado en Gate D → ship resto (tool-diseñada ✅, MEM-70 DEFER ✅, suite+clippy ✅, task file ✅) + DEFER-ratificado del resto pendiente de respuesta.

## 9. INVESTIGACIÓN INTERNET
- No se espera (diseño interno ADR-040 + código propio). No se usó red. Sin ambigüedad externa irresoluble → sin citas NO VERIFICADAS ni deuda TSYS-13.

## 10. VALIDACIÓN+CIERRE
- **Verify contrato (cierre Step 3, con código):**
  - MEM-69 wiring: ✅ `TaskKind::Dream` + `run_dream` (idle fail-open documentado) + `with_use_batch(false)` + `run_l1_batch` (1 llamada, misma cola checkpoint que el path split)
  - tool 77: ✅ diseñada (ADR-040:38-41) — D3=B no implementar
  - MEM-70: ✅ DEFER-ratificado (BENCHMARKS §17 + smoke §6)
  - suite 336: ✅ 336/0 lib exactos (541/0 total)
  - clippy 0: ✅ scoped vanta-memory
- **Verify full:** fmt ✅ · clippy scoped ✅ (workspace `-D warnings` global bloqueado por `txn.rs:158` pre-existente = FIND-93, no tocado) · cargo test ✅ 28/28 suites · nextest-audit workspace no corrido (deuda: suite supera en tamaño al gate rápido; comando canónico pendiente en Heavy) · docs-coverage no corrido (sin docs técnicas nuevas; cambio documentado en module-doc del worker) · OCR delegation: `dev-tools/ocr-review.ps1` untracked (WIP ajeno) — intento de ejecución al commit; si inviable, deuda.
- **DoD 3 niveles:** task (Steps 1-3 ✅) · commit `feat:` 4 archivos propios (pendiente) · release (N/A).
- **Reviewer distinto P2-01:** vanta-review vía `task` (o self-review + checklist si falla; orquestador puede forkear).
- **Gates:** D ✅ resuelto A/A/B (SARL orquestador) · V no (1 solo error E0502, fix directo sin reintento ciego) · C al commit (colaterales: 1 edit accidental revertido — línea import borrada y restaurada, verificado por diff; ningún otro) · P no (familia aprobada, D cubre).
- **Backlog→avance:** NO tocado (race paralelo, orquestador vía `progreso`). **Push:** vía vanta-lead.

## Impacto mapeado (Regla 0 — MUST antes del primer edit; no hubo edits en Steps 1-2)
- **Archivos leídos completos:** `dream/mod.rs` (1015L), `l1_batch.rs` (298L), `auto_consolidate.rs` (273L), `pipeline_worker.rs` (768L), `types.rs` (108L), `conversation_hook.rs` (107L), `gateway/mod.rs`, `knowledge_handlers.rs` (:1-100 + grep resto), `ADR-040` (51L), `BENCHMARKS.md` §17, `memory_bench.py` (:1-60), `l1_extractor.rs` (:1-120), `l1_dedup.rs` (:1-120), `lib.rs`, `local_backend.rs` (grep), `tools.rs`/`config.rs`/`scenes.rs` (grep).
- **Referencias hacia dentro (el cambio futuro toca):** `TaskKind` (serde), `handle` match, `run_l1_inner`, `CheckpointManager`, `LocalStateBackend` (locks), gateway + `scenes.rs` + conteos MCP (solo si D3=A).
- **Referencias entrantes (dependen y deben seguir verdes):** `tests/dreaming.rs`, `tests/l1_batch.rs`, `tests/pipeline_manager.rs`, `tests/e2e_flow.rs`, `conversation_hook::run_bridge_pass`, `utils/pipeline_factory.rs`, `context_engine/report_store.rs`.
- **Veredicto:** blast radio de LECTURA amplio (verificado), blast de ESCRITURA futuro acotado a `types.rs` + `pipeline_worker.rs` (+ gateway/MCP solo si D3=A). 0 ediciones realizadas en este turno → 0 riesgo introducido.

## Steps
### Step 1: DISCOVERY + Spec + task file
- **Archivos:** `docs/tasks/FIND-86.md` (nuevo)
- **Acción:** auto-detect type (rust) → SDP → codegraph + detect_changes + arquitectura + coverage → lectura completa slices → HALLAZGOS H1-H7 → Spec D1-D4 → crear task file 10 bloques.
- **Verify:** task file existe con contrato + Spec LLENA + Save Point.
- **Estado:** ✅ DONE
### Step 2: Verify sin código (suite + harness + conteos)
- **Archivos:** ninguno (solo lectura + comandos)
- **Acción:** check + clippy + test + fmt + diff-check + harness smoke + conteos MCP (79) + MEM-70 (§17).
- **Verify:** §6 + §10 (336/0 lib, 538/0 total, clippy 0, fmt limpio, smoke 1.0 techo).
- **Estado:** ✅ DONE
### Step 3: Wiring ACT (dream TaskKind + batch opt-in + tests)
- **Archivos:** `vanta-memory/src/core/state/types.rs` (+4 `Dream`), `vanta-memory/src/services/pipeline_worker.rs` (+154: `use_batch` + `with_use_batch` + `run_l1_batch` + `run_dream` + rama `handle`), `vanta-memory/tests/pipeline_manager.rs` (+219: 3 tests + runners)
- **Acción:** RED (2 errores E0599: sin variante `Dream` / sin `with_use_batch`) → GREEN (D1 + D2; fix E0502 bajando helpers a `&self` — `checkpoints` retiene `&self.db`) → suite + clippy + fmt.
- **Verify:** `cargo test -p vanta-memory -j 2` 28/28 suites ok, **541 passed / 0 failed** (336 lib + 204 integración + 1 doc) + clippy scoped `-D warnings` 0 + `cargo fmt --check` limpio + `git diff --check` limpio.
- **Estado:** ✅ DONE

## HALLAZGOS (divergencias plan↔código, con evidencia — no silenciadas)
- **H1 (menor):** paths plan sin prefijo `vanta-memory/src/` para l1_batch/auto_consolidate; reales `vanta-memory/src/core/record/l1_batch.rs`, `vanta-memory/src/core/scene/auto_consolidate.rs` (Test-Path + lectura directa).
- **H2 (medio):** plan/ADR reservan dream-wiring a MEM-65, pero MEM-65 real = telemetría por capa + pLimit (task file MEM-65:1); wiring sigue pendiente → FIND-86 válido y necesario.
- **H3 (medio):** plan dice "MEM-70 sin rastro/menciones en vanta-memory"; real: harness `evals/memory_bench.py` + `BENCHMARKS.md` §17 + tests + task MEM-70 ✅ + avance. El plan está stale en este punto; DEFER-ratificado ya existe.
- **H4 (medio):** tool "77" diseñada cuando Full=76 (ADR-040); hoy Full=79 (`tools.rs:25-26,1090`, `config.rs:11,14`) → implementarla sería tool 80 con recount, no 77.
- **H5 (info):** "suite 336" = lib tests exactos (336/0 verificado hoy); total real con integración+doc = 538/0.
- **H6 (info):** `txn.rs:158` dead_code bajo `-D warnings` global = FIND-93 (creado por FIND-65); no tocado aquí.
- **H7 (info):** `frontend-ui-engineering` (SDP lifecycle genérico) N/A en este scope; se declara y no se carga.

## Dependencias
- Ninguna pendiente (Gate D resuelto). FIND-72/FIND-92/93/94 no absorbidos.

## Notas
- `ponytail:` smoke MEM-70 usa fallback dict (techo recall 1.0) — upgrade = backend vantadb real + loader versionado (ya documentado en BENCHMARKS §17 Pendiente).
- NOTICED BUT NOT TOUCHING: `txn.rs:158` (FIND-93) · WIP ajeno §2 · `evals/memory_bench_report.json` regenerado por el smoke (gitignored, no stageado).
- Colateral del turno: 1 edit accidental (línea import borrada en pipeline_manager.rs) detectado y revertido antes del commit; imports verificados por `Select-String` + suite verde posterior.
- Commit: `feat: FIND-86 — wiring dream TaskKind + batch opt-in + tests`.

## Context Save Point
- **Fecha:** 2026-09-16 · **Branch:** develop (base 0be84203) · **CI pendiente:** no (verifies locales §6+Step 3)
- **Decisiones:** Spec D1=A / D2=A / D3=B / D4=B ✅ (orquestador SARL) · helpers a `&self` por E0502 (`checkpoints` retiene `&self.db`) · Regla 8 N/A justificado (sin locks nuevos) · Regla 9 sin claims P99 (opt-in default false)
- **Problemas conocidos:** `campaign_verify_cmd` bug exit -1 (fallback bash) · nextest-audit workspace no corrido (deuda) · OCR review al commit (script untracked) · `use_batch` sin medición before/after con runner real (deuda honesta: opt-in + conteo por `task_id` en tests)
- **Próxima tarea:** review P2-01 → commit `feat:` → `campaign_update_task_state completed` → `progreso` (orquestador) + push vía vanta-lead. Siguiente task del plan: FIND-72.

=== RECITATION ===
Objetivo activo: FIND-86 — wiring MEM-69 + tool 77 + números MEM-70
Estado: completed (Steps 1-3 DONE + verify + commit pendiente)
Última acción: Step 3 RED (E0599×2) → GREEN (TaskKind::Dream + run_dream + with_use_batch + run_l1_batch; fix E0502 a &self) → 28/28 suites 541/0 + clippy 0 + fmt limpio + task file sync
Resultado: OK
Próxima acción: review P2-01 → git commit feat: FIND-86 (4 archivos propios) → campaign_update_task_state completed
Contrato: verificacion: cargo test -p vanta-memory -j 2 → 28/28 suites, 541 passed (336 lib + 204 integración + 1 doc), 0 failed ✅; clippy scoped -D warnings ✅ 0; fmt ✅; diff-check ✅. evidencia: claim dream wiring / evidencia types.rs Dream + pipeline_worker.rs:809 run_dream + test handler_dream_task… ok / alta; claim batch 1 llamada / evidencia with_use_batch + run_l1_batch + test 1 call vs control 2 calls / alta; claim tool diseñada / evidencia ADR-040:36-41 (D3=B) / alta; claim MEM-70 DEFER / evidencia BENCHMARKS §17 + smoke / alta. artefactos: types.rs, pipeline_worker.rs, tests/pipeline_manager.rs, docs/tasks/FIND-86.md (commit feat: pendiente). invariantes: default behavior intacto (use_batch=false; Dream solo ante task explícita); prohibidos intactos; Regla 11 sin claims P99. deuda: review P2-01 + nextest-audit workspace + OCR review + progreso/push lead. queda_pendiente: commit + completed + FIND-72
Próxima tarea si completa: FIND-72
last-synced: 2026-09-16
=== END RECITATION ===
