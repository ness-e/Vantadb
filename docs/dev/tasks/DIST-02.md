---
title: "TASK DIST-02: Exponer la capa cognitiva en Python (memory_recall / memory_capture)"
kind: task
description: "Binding Python de la capa cognitiva (vanta-memory): memory_capture (L0) + memory_recall (L1/persona) end-to-end con wheel local; stubs + PYTHON_SDK; scope mínimo, dream follow-up."
---

# TASK DIST-02: Exponer la capa cognitiva en Python (`memory_recall`/`memory_capture`)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 15, F0)
- **Fuente:** DELTA 2026-09-30 (P0) — "un usuario Python tiene una BD, no memoria" (research §4; verificado 2026-09-30: 0 refs `vanta_memory` en el binding)
- **Esfuerzo:** 🟡 1-2d | **Appetite:** max 3d | **Prioridad:** 🔴
- **Tipo:** Feature-add / bindings (PyO3 glue sobre `vanta-memory`)
- **Creado:** 2026-10-04 | **last-synced:** 2026-10-04
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE — `ses_ef9569455ffeOawsQHJX4n3kV9`; commits `2a3bccbc` + `ee731884` + cierre)
- **Campaign ID:** master-plan-0.9.0-20261004
- **Incógnitas (uphill):** 0 abiertas — API de `vanta-memory` verificada en código (`perform_auto_recall`, `AutoCaptureHook::capture`); superficie sync/async decidida en Spec
- **Pendientes (downhill):** 0 steps (4/4 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Usuarios del wheel `vantadb-py` (nuevos métodos `Client.memory_capture`/`memory_recall` + `AsyncClient`); `tests/test_stub_drift.py` (paridad stub↔nativo); `scripts/docs/check-api-docs.mjs` (fingerprint de superficie Python) |
| Callees | `vanta-memory` (`core::hooks::{AutoCaptureHook, AutoCaptureConfig, RawMessage, perform_auto_recall, AutoRecallParams, RecallConfig, RecallScope, RecallError}` + `core::conversation::L0Error`) → `vantadb::sdk::Embedded` (mismo engine que el `Client` ya posee) |
| Implicaciones | (1) `vantadb-python/Cargo.toml` gana dep path a `vanta-memory` (default features: LLM-free; el recall degrada a keyword — contrato D38); (2) `Cargo.lock` cambia (dep nueva); (3) wheel crece (crate ~28k LOC) — aceptado por contrato (la feature ES el producto); (4) sin cambios en `vanta-memory`, `src/**` del core, TS/WASM |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-python/src/lib.rs` (secciones: imports 1-170, macro `forward_to_db!` 208-317, `#[pymethods] impl Client` 572-1990, módulo 2500-2535), `vantadb-python/Cargo.toml` (29L), `vantadb-python/pyproject.toml` (70L), `vantadb-python/vantadb_py/__init__.py` (593L), `vantadb-python/vantadb_py/vantadb_py.pyi` (571L), `vantadb-python/vantadb_py/__init__.pyi` (287L), `vantadb-python/tests/{test_stub_drift,test_subclients,test_w1_surface,conftest}.py`, `vanta-memory/src/core/hooks/{auto_capture.rs,auto_recall.rs}` (secciones), `vanta-memory/src/core/hooks/mod.rs` (14L), `vanta-memory/src/core/conversation/l0_recorder.rs` (secciones), `vanta-memory/src/core/abstractions/types.rs` (secciones), `vanta-memory/tests/{l0_capture.rs,recall.rs}` (secciones), `desktop/src-tauri/src/commands/memory.rs` (precedente de mapeo), `vantadb-mcp/src/context.rs` (precedente), `docs/api/VANTA_MEMORY.md` (374L), `docs/api/PYTHON_SDK.md` (secciones), `scripts/docs/check-api-docs.mjs` (secciones)
- **Archivos referenciados hacia dentro (imports/deps):**
  - `vantadb-python/src/lib.rs` ← `pyproject.toml` (`module-name = vantadb_py`), tests del binding (importan `vantadb_py.vantadb_py`), stubs `.pyi` (paridad por `test_stub_drift.py`)
  - `vanta-memory` ← path deps en `vantadb-mcp`, `vanta-proxy`, `desktop` (intactos); DIST-01 lo dejó publicable con hold release-plz
- **Archivos que referencian a los editados (referencias entrantes):** `test_stub_drift.py` (paridad de métodos/params/requiredness del stub vs módulo compilado); `test_subclients.py` (set de métodos de subclients); `check-api-docs.mjs` (fingerprint `.pyi` + `__all__`); `docs/api/BINDINGS_NAMESPACES.md:277` (conteo "46 pyclass methods" → quedará stale 48; NO se toca — archivo de DIST-03 en vuelo, handoff en deuda)
- **Veredicto impacto:** **MEDIO** — cambio aditivo (2 métodos nuevos + 2 wrappers async + stubs + docs), sin modificar métodos existentes. Único riesgo estructural: la dep nueva `vanta-memory` en el wheel (compila el crate en el binding; features default LLM-free). `vanta-memory` no muta el engine más allá de writes normales (L0/L1 namespaces) — sin cambios de formato on-disk.

## Contrato

> Del plan (Task 15) + prompt del orquestador. Scope mínimo escrito ANTES de codear (pre-mortem #1).

1. `db.memory_capture(session_id, messages)` — captura L0 LLM-free; devuelve `{"recorded_count", "filtered_messages", "cursor_ms"}`; idempotente por cursor; visible luego vía `db.memory.list("l0/<session>")` — ⬜ Step 2/3
2. `db.memory_recall(user_text, session_key, scope=None, max_results=None)` — recall LLM-free (degrada a keyword sin embedding hook, `effective_mode` lo reporta); `None` cuando no hay nada que inyectar (nunca bloque vacío); dict con `prepend_context`/`append_system_context`/`recalled_memories`/`persona`/`effective_mode` — ⬜ Step 2/3
3. Smoke e2e con wheel local (venv limpio): capture → L0 visible + recall de un L1 sembrado → hit; stubs `.pyi` actualizados y `test_stub_drift` verde — ⬜ Step 3
4. `docs/api/PYTHON_SDK.md` documenta el scope (sync + async; `dream` follow-up); gate `check-api-docs` verde (docs + llms.txt movidos con la superficie) + suite Python verde — ⬜ Step 3/4

## Spec (SDD — feature-add: símbolos públicos nuevos pre-especificados por el contrato)

> Gate D: el plan nombra los símbolos exactos (`memory_recall`/`memory_capture`) y el orquestador dio GO con ese contrato → los símbolos NO son invención del implementador. Micro-decisiones de forma abajo (A = default recomendado, decidido por evidencia).

| # | Decisión | Opciones (+tradeoff) | Default | Resuelto |
|---|----------|----------------------|---------|----------|
| 1 | Nombres | A) flat `Client.memory_capture`/`memory_recall` (contrato del plan; paridad con desktop `vanta_memory_*`) / B) solo `db.memory.capture/recall` | A | ✅ decidido-por-contrato: el plan fija los nombres; B se descarta para no crear paridad TS inexistente (DIST-03 decide TS/WASM) |
| 2 | Return shapes | A) dicts planos (patrón del binding para reportes: `capabilities()`, `export_all()`) / B) pyclasses nuevas | A | ✅ decidido-por-evidencia: pyclasses solo para records/hits con property access; un dict documentado es suficiente para reportes |
| 3 | Superficies sync/async | A) sync (nativo) + `AsyncClient` wrapper / B) solo sync | A | ✅ decidido-por-evidencia: `AsyncClient` envuelve la superficie (search/put/...); `test_stub_drift` pinea el wrapper; coste 2 métodos triviales |
| 4 | Knobs de recall | A) `scope` + `max_results` opcionales / B) config completa (mode, budgets, isolation) | A | ✅ decidido-por-evidencia: `scope` es el knob D22 documentado (Session\|Agent\|Team); `max_results` (default 5) es el control más pedido; budgets/isolation quedan para follow-up |
| 5 | `dream` | A) follow-up declarado / B) incluirlo ahora | A | ✅ decidido-por-contrato: el plan lo marca "opcional/follow-up"; stop condition de scope |
| 6 | Errores | A) `L0Error::Vanta`/`RecallError::Vanta` → jerarquía tipada existente (`map_vanta_error`); resto ValueError/RuntimeError/CorruptError descriptivo / B) todo RuntimeError | A | ✅ decidido-por-evidencia: patrón `map_vanta_error` (convert.rs) + `mem_err` del desktop (downcast del chain) |
| 7 | Feature gate | A) incondicional (sin feature nueva) / B) `#[cfg(feature = "memory")]` | A | ✅ decidido-por-evidencia: el binding no tiene features de cara al usuario; el contrato es exponerla end-to-end; la degradación LLM-free es el default seguro |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) la lógica vive en `vanta-memory`/core — el binding es glue + mapping (api-contract R-8); (2) GIL liberado en las ops (`py.detach`) y closure cerrado en el binding (python-bindings R-1/R-2); (3) `None` de recall = "nada que inyectar" (nunca bloque vacío); (4) sin embedding hook adjunto → degradación a keyword reportada en `effective_mode` (D38/P4 — no "arreglar" en el binding); (5) métodos existentes intactos (aditivo); (6) `vantadb-ts/**`/`vantadb-wasm/**` NO se tocan (DIST-03 en vuelo); (7) `vanta-memory/Cargo.toml`/`release-plz.toml` NO se tocan (DIST-01 cerrada); (8) `opencode.jsonc`/plan file WIP ajeno intacto.
- **Comandos de verificación:** `cargo check -p vantadb_py` · `cargo fmt -p vantadb_py --check` · `cargo clippy -p vantadb_py --all-targets -- -D warnings` · `target/dist02-venv/Scripts/python -m pytest vantadb-python/tests/ -q` · smoke `target/dist02-venv/Scripts/python target/tmp/dist02-smoke.py` · `node scripts/docs/check-api-docs.mjs --changed <ref>..HEAD` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`.
- **Deuda pendiente:** `dream` (consolidación idle) = follow-up declarado; conteo/matriz en `BINDINGS_NAMESPACES.md:277` (46→48) queda para DIST-03/DIST-04 (archivo en vuelo); async wrappers no cubren `AsyncMemoryClient` (solo flat — consistente con `search`/`put`); FIND nuevo por CRLF (ver Notas).

## Deuda técnica (Regla 6)

- **Deuda nueva:** ninguna (aditivo, sin `unsafe`, sin clones en hot paths — los métodos son ops de storage normales).
- **Pago:** n/a — la regla aplica a PRs con deuda neta positiva; aquí el saldo es cero.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 1-4 ✅ + fmt/clippy/check scoped + suite Python + smoke wheel local + stubs con paridad (`test_stub_drift`) + gates docs |
| **Commit** | Commit local `feat(python):` atómico, verificación mecánica (nunca auto-reporte), sin archivos fuera del blast radius |
| **Release** | Entrada de changelog (feature pública → minor); la gestiona release-plz — n/a en este task |

## Herramientas necesarias

- `cargo check/clippy/fmt -p vantadb_py` (iteración Rust)
- `maturin build` + venv limpio (`target/dist02-venv`) + `pip install <wheel>` (smoke e2e)
- `python -m pytest` (suite del binding)
- `codegraph_codegraph_explore` + `codebase-memory-mcp_check_index_coverage` (discovery)
- `campaign_verify_cmd` (checks mecánicos) + `pwsh dev-tools/ocr-review.ps1` (cierre)

**Skills cargadas (SDP v3):** `security-and-hardening` (pinned) · `documentation-and-adrs` (pinned) · `api-and-interface-design` (pinned) · `source-driven-development` · `incremental-implementation` · `test-driven-development` · `documentation-skill` (regla AGENTS.md para docs/) · `rust-write-tests` (sugerida del plan). Base auto: campaign-executor · progreso.

## Investigation Notes

- **API real de `vanta-memory` (verificada en código, no docs):** `AutoCaptureHook::new(db: Embedded, AutoCaptureConfig)` + `.capture(session_id, Vec<RawMessage>) -> Result<AutoCaptureResult, L0Error>` (`auto_capture.rs:67-138`); `perform_auto_recall(db: &Embedded, AutoRecallParams, Option<&EmbedFn>) -> Result<Option<RecallResult>, RecallError>` (`auto_recall.rs`); `RecallConfig { mode, scope, max_results, min_overlap, budgets }` default `Hybrid/Agent/5`; `RecallResult { prepend_context, append_system_context, recalled_memories, persona, effective_mode, governance }` — no `Serialize`, se mapea a mano (precedente desktop `RecallOutcome`).
- **Precedente de mapeo:** `desktop/src-tauri/src/commands/memory.rs` (`run_capture`/`run_recall` + mirrors) y `vantadb-mcp/src/context.rs` (`perform_auto_recall_governed`) — mismo shape, sin `Serialize` en los tipos de `vanta-memory`.
- **Degradación (contrato P4/D38):** sin embedding hook, `Hybrid`→`Keyword` y `effective_mode` lo reporta. El binding NO adjunta hook (coherente con desktop/MCP). `Ok(None)` = nada que inyectar.
- **Idempotencia L0:** cursor persistido por sesión (`l0_cursor/<session>`); replay no duplica (`l0_capture.rs` tests (a)/(c)).
- **Wire shape L1 (para el smoke):** `MemoryRecord` JSON con `id/content/type/priority/scene_name/source_message_ids/created_at/updated_at/session_key` requeridos; `serde(default)` en el resto (`abstractions/types.rs:71-130`).
- **`vanta-memory` publicable (DIST-01):** path dep local sigue resolviendo al workspace; el hold de release-plz no afecta al binding (que no se publica a crates.io; PyPI va por maturin).
- **Estado Python (baseline):** `import vantadb` OK (build stale 0.7.0 en el paquete); `Client.memory_recall` → `False` (RED confirmable). `maturin 1.15.0`; sin `target/audit-venv` → venv propio `target/dist02-venv`.
- **Gate api-docs (baseline):** `--changed HEAD..WORKTREE` falla HOY con 53 "removals" fantasma del surface Rust — causa raíz: worktree CRLF en 57 archivos `src/*.rs` (`src/gds.rs`, `graph.rs`, `accumulator.rs`, `schema.rs`, …) vs blobs LF; el parser lee worktree crudo (regex de `impl` anclada a `$`) y git devuelve LF. `git diff` lo oculta (normaliza por `.gitattributes`). CI compara refs git (consistente). → verificación local de cierre con rango de refs git + FIND de entorno.
- **`check-api-docs` lógica:** superficie Python = `.pyi` + `__all__`; superficie cambiada exige diff en `docs/` **y** `llms.txt` (default budgets 0). → la descripción de `PYTHON_SDK.md` (fuente de `llms.txt` vía `gen-index.mjs`) debe actualizarse para que llms.txt se mueva (además es la descripción stale "Note: ..." — anti-pattern §8 de documentation-skill).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — API y superficies decididas en Spec |
| Pendientes de ejecución (downhill) | 0 steps (4/4 ✅) |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- **SECURITY:** toca trust boundary FFI (input de usuario Python → Rust). Checklist `security-and-hardening`: validación en la frontera (mensajes: tipo/rol/contenido validados con errores descriptivos; scope whitelisted fail-loud), sin secretos, sin nuevos endpoints de red, sin `unsafe` nuevo, errores tipados sin filtrar internals. Dep nueva = misma crate workspace ya auditada (`cargo-deny` vigente; sin features de red por default). Gate: clippy -D warnings + revisión P2-01.
- **PERFORMANCE:** no toca hot paths del core (HNSW/métricas/serialización). Ops de storage normales; GIL liberado (`py.detach`). Sin benchmark requerido (Regla 9 no dispara: no hay optimización declarada).

## Steps

### Step 1 — DISCOVERY completo (task file + evidencia) — ✅
- **Archivos:** (lectura) los del Impacto Regla 0; (escritura) `docs/dev/tasks/DIST-02.md`
- **Acción:** verificar 0 refs `vanta_memory`, API real de `vanta-memory`, superficie/patrones del binding, stubs y tests de paridad, gates baseline, entorno de build
- **Verify:** ✅ `rg "vanta[_-]memory" vantadb-python` → 0 matches · `hasattr(Client,'memory_recall')` → False · `codegraph_explore` de ambos lados · `check_index_coverage` sin gaps registrados · baseline api-docs diagnosticado (CRLF)
- **Estado:** ✅

### Step 2 — Rust: dep `vanta-memory` + `memory_capture`/`memory_recall` + error mapping — ✅
- **Archivos:** `vantadb-python/Cargo.toml`, `vantadb-python/src/lib.rs`
- **Acción:** dep path (default features); helpers `parse_raw_message`/`parse_recall_scope`/`map_l0_error`/`map_recall_error` + conversores a dict; 2 métodos en `impl Client` con `py.detach` + `enter(&op_gate)`
- **Verify:** ✅ `cargo check -p vantadb_py` exit 0 · `cargo fmt -p vantadb_py --check` exit 0 · `cargo clippy -p vantadb_py --all-targets -- -D warnings` exit 0 · `cargo tree -p vantadb_py -i reqwest` → 101 (sin red en el grafo; reviewer)
- **Estado:** ✅

### Step 3 — Tests (RED→GREEN) + stubs + wrapper async + docs + smoke wheel — ✅
- **Archivos:** `vantadb-python/tests/test_memory_layer.py` (nuevo), `vantadb-python/vantadb_py/vantadb_py.pyi`, `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/vantadb_py/__init__.pyi`, `docs/api/PYTHON_SDK.md`, `llms.txt` (regen)
- **Acción:** test RED contra el módulo stale (AttributeError) → build wheel (`maturin build` + venv `target/dist02-venv`) → GREEN; smoke e2e (capture→L0 visible; L1 sembrado→recall hit; None sin contenido); stubs con paridad exacta; docs
- **Verify:** ✅ RED 10 fallos (`AttributeError: 'Client' object has no attribute 'memory_capture'`) → GREEN 11/11 · suite completa **173 passed, 4 skipped, 4 deselected** (re-ejecutada por el reviewer) · smoke OK (`vantadb 0.8.0` desde site-packages del venv; capture→reopen→recall→None) · `test_stub_drift` 7/7 · `check-api-docs --changed HEAD~1..HEAD` exit 0
- **Estado:** ✅

### Step 4 — Cierre: gates docs + OCR + review P2-01 + commit local — ✅
- **Archivos:** `docs/dev/tasks/DIST-02.md` (+ los anteriores), `docs/dev/Backlog.md` (FIND-257/258/259), `docs/dev/avance/activo/bindings.md`
- **Acción:** gates docs (check-links/check-docs/gen-index --check/api-docs por rango git); OCR delegation; review por agente distinto; commits locales; registro en avance + fila Backlog removida
- **Verify:** ✅ gates docs exit 0 (check-links within budget · check-docs GATING all clear · gen-index --check 0 · `validate-docs-coverage` 0 gaps · `check-avance-coverage` OK) · OCR 0 Critical/High/Medium (5 archivos, rule groups) · review P2-01: ronda 1 changes-required (R1) → fix `ee731884` → **APPROVE** · commits `2a3bccbc` + `ee731884` + cierre (local, sin push)
- **Estado:** ✅

## Dependencias

- **Bloqueantes:** ninguno — DIST-01 ✅ (path dep funciona).
- **Dependientes:** Task 17 (DIST-04) documenta la superficie publicada post 01/02/03.
- **Coordinación:** DIST-03 en vuelo (TS/WASM) — no tocar `vantadb-ts/**`, `vantadb-wasm/**` ni `BINDINGS_NAMESPACES.md`. `nextTask`: DIST-03 (ya en vuelo) / DIST-04 (bloqueada hasta cerrar 02/03).

## Review (GATE — agente distinto, P2-01)

> Tier risk-based: paths del diff — `vantadb-python/src/lib.rs` (binding PyO3, no listado en globs adversariales del plan) + `docs/api/PYTHON_SDK.md` (**matchea `docs/api/**` → adversarial**). Diff mixto = adversarial → `vanta-review` (contexto fresco).

- **Revisor:** `vanta-review` (contexto fresco — subagent `ses_ef9569455ffeOawsQHJX4n3kV9`; sin participación en la implementación)
- **Enfoque:** contrato 1-4 re-ejecutado (pytest 10→11, suite 172→173, smoke wheel local, `check-api-docs` por rango git, `test_stub_drift` 7/7, fmt/clippy) + sondas adversariales propias: import desde site-packages (no in-tree stale), `None` vs bloque vacío (`max_results=0`), wire L1 (`type:"fact"` → invisible), límites (`max_results=-1` → OverflowError; `id=""` → ValidationError), cross-session isolation, `cargo tree` sin reqwest.
- **Veredicto:** ✅ **APPROVE** (ronda 1: **changes-required** con 1 Required — R1: el ejemplo de docs prometía un loop capture→recall inexistente; ronda 2 tras `ee731884`: APPROVE). R1 resuelto (nota "Recall pool" + ejemplo honesto + test `test_memory_capture_alone_does_not_create_recallable_memories`); M1/M2 → FIND-258/259; L1/N1/N2/N3 Low/Nit declarados no bloqueantes (L3 = este cierre).

## Notas

- **Scope mínimo:** recall + capture (sync + async wrapper). `dream`, budgets de recall, `isolation` team/agent custom y paridad `db.memory.capture/recall` quedan como follow-ups declarados.
- **FIND entorno (CRLF) → FIND-257:** `check-api-docs --changed A..WORKTREE` da falsos positivos en Windows cuando el worktree tiene CRLF en archivos `src/*.rs` (57 archivos hoy; git normaliza, el parser no). No se renormaliza el árbol (fuera de scope). Verificación local del gate: rango de refs git (`HEAD~1..HEAD`).
- **Hallazgos del review → FIND-258** (código `VANTADB_*` perdido en wrappers anidados de `RecallError`) y **FIND-259** (L1 `type` inválido invisible + enum sin enumerar en VANTA_MEMORY; DIST-02 agregó la advertencia parcial en §Cognitive Layer).
- **R1 (Required) resuelto:** el ejemplo original de §Cognitive Layer prometía capture→recall directo; recall lee L1/L2/L3 y la captura LLM-free solo escribe L0 → ejemplo reescrito + nota "Recall pool" + test que pinea el `None` (`ee731884`).
- **BINDINGS_NAMESPACES.md stale:** el conteo "46 pyclass methods" pasa a 48; la matriz `conversation` ("not exposed") queda desactualizada para Python. Handoff a DIST-03/DIST-04 (archivo en vuelo — no se tocó).
- **`last_reviewed` (N3):** no se actualiza — `documentation-skill` §0 lo declara clave retirada (fecha auto-reportada; freshness = `git log`). Se deja como está.
- **Push:** diferido al final del plan (instrucción owner) — commits locales.

## RESULTADO §7

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4
PROXIMO_STEP: ninguno
COMMIT_HASH: 2a3bccbc (impl) + ee731884 (fix R1 + FINDs) + cierre docs (este commit)
ARCHIVOS: vantadb-python/Cargo.toml · vantadb-python/src/lib.rs · vantadb-python/tests/test_memory_layer.py · vantadb-python/vantadb_py/{__init__.py,__init__.pyi,vantadb_py.pyi} · docs/api/PYTHON_SDK.md · docs/api/index.md · docs/index.md · llms.txt · Cargo.lock · docs/dev/tasks/DIST-02.md · docs/dev/Backlog.md (FIND-257/258/259; fila DIST-02 removida) · docs/dev/avance/activo/bindings.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:disparado V:no C:no | D: símbolos públicos nuevos pre-especificados por el contrato del plan (Task 15) + GO del orquestador; micro-decisiones documentadas en §Spec. P: sin cambios de plan/arquitectura. V: sin retries agotados (review ronda 1 → fix dirigido). C: hallazgos Low/Nit declarados (L1/L2/N1/N2/N3) sin fila por valor marginal; Medium/Required ruteados (FIND-258/259) o resueltos (R1).
SKILLS_CARGADAS: security-and-hardening · documentation-and-adrs · api-and-interface-design · source-driven-development · incremental-implementation · test-driven-development · documentation-skill · rust-write-tests · progreso (Trigger 1) (base auto: campaign-executor)
```
