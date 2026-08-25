# Plan de Ejecución: Batch Colaterales + Deuda + Desktop (2026-08-25)

> **Inicio:** 2026-08-25
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md (selección del lead + confirmación del usuario 2026-08-25)
> **Modo:** FAIL_MODE=parallel, MAX_CONCURRENT=3

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 14 |
| 🟡 DEFER | 2 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 1 (MEM-51 requiere decisión de diseño) · ⬇️ downhill = 13

> **Nota:** el usuario confirmó que las sesiones paralelas P34/P37 ya no existen y pidió **incluir desktop** y eliminar el registro de colisiones (adenda P34, H2, AGT-05 limpiados del backlog). El árbol de desktop está limpio.

## Tasks

### Task 1: FIND-30 — unused var `ns` en cli_server.rs:1302 (clippy -D warnings blocker)

- **Appetite:** max 30m
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `src/cli_server.rs:1302`
- **Verificación real:** ✅ CÓDIGO-REAL — hallazgo REVIEW-17 2026-08-25: closure `move |ns: String|` ignora `ns` → rompe `cargo clippy --workspace --all-targets -- -D warnings` bajo feature `server`. Pre-existente. Fix mecánico 1 línea: `_ns`.
- **Gate Justificación:** desbloquea clippy full-workspace, effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo clippy --workspace --all-targets -- -D warnings` pasa (o `cargo clippy -p vantadb --features server --all-targets -- -D warnings`)
- **Task file:** `skills/campaign-executor/tasks/FIND-30.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** — trivial. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) renombrar param rompe callers (no debería).

### Task 2: FIND-31 — purge_expired tras reopen falla "text index df would go negative"

- **Appetite:** max 2h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟡
- **Archivos clave:** `src/sdk/api.rs:1008-1061`, `src/storage/engine/init.rs` (recover_state)
- **Verificación real:** ✅ CÓDIGO-REAL — `purge_expired` (api.rs:911) computa term_deltas sobre `load_text_term_stats`; tras reopen `replay_write_node` NO reconstruye text index, solo HNSW → los deltas pueden volverse negativos ("text index df would go negative"). Reproducido con put→flush→reopen→purge. Hallazgo MOD-04 2026-08-25.
- **Gate Justificación:** bug core real de durabilidad/consistencia tras reopen; effort 🟡.
- **Gate Result:** ✅ DO
- **Contrato:** test put→flush→reopen→purge_expired pasa sin error; `cargo nextest run -p vantadb` verde
- **Task file:** `skills/campaign-executor/tasks/FIND-31.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: reconstruir text index en reopen es costoso (impacta cold-start)
  - Fallo 2: guard de deltas enmascara el bug real de fondo
  - **Stop conditions:** si el fix exige reconstrucción de text index en recover_state (cambio grande), evaluar guard de deltas como fix mínimo primero (ponytail).
  - **Cynefin:** 🟨 complicado — storage/text index. **Top 3 riesgos:** (1) cold-start cost; (2) fix superficial; (3) consistencia.

### Task 3: FIND-32 — tests rate-limit obsoletos en server.rs

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-server/tests/server.rs:223,235`
- **Verificación real:** ✅ CÓDIGO-REAL — hallazgo MOD-13 2026-08-25: `test_rate_limit_enforces_after_burst` (:223) y `test_rate_limit_health_unaffected` (:235) asumen burst=1 con rpm=5, pero `rate_limit_burst` devuelve rpm completo sin auth (burst=5) → 2ª request pasa 200. Fallan en base. Excluidos del default-filter.
- **Gate Justificación:** arreglar tests obsoletos (alinear a burst=rpm o usar auth para burst=rpm/10); effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-server --test server test_rate_limit` pasa (o el comando exacto del test)
- **Task file:** `skills/campaign-executor/tasks/FIND-32.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** — test flaky por timing. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) timing; (2) alineación incorrecta del burst.

### Task 4: MCP-34a — wrapper MCP snapshot_create

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟡
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `src/storage/engine/mod.rs:540`, `src/sdk/builder.rs:253`
- **Verificación real:** ✅ CÓDIGO-REAL — `create_snapshot` SÍ existe (`StorageEngine::create_snapshot` mod.rs + SDK `VantaEmbedded::create_snapshot` builder.rs:253). Falta solo el wrapper MCP (desglose de MCP-34 DEFER). snapshot_restore NO existe (queda DEFER como feature core).
- **Gate Justificación:** wrapper fino sobre API pública existente; cierra la parte viable de MCP-34.
- **Gate Result:** ✅ DO
- **Contrato:** tool `snapshot_create` (name + result `{"path","created_at"}`); `cargo test -p vantadb-mcp --test mcp_tests` pasa; docs ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MCP-34a.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: FsSnapshot no Serialize → construir result manual
  - Fallo 2: hash SAME docs
  - **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) serialización FsSnapshot; (2) hash SAME.

### Task 5: MOD-06 — nits agrupados WAL

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🟢
- **Archivos clave:** `src/wal_sharded.rs`, `src/storage/engine/insert.rs:158-204`
- **Verificación real:** 🟡 VERIFICAR — backlog: nits agrupados (flush thread-per-shard, clones batch_append, lookup intern en loop, cardinality dup, write_shard_meta no atómico). Confirmar en DISCOVERY.
- **Gate Justificación:** higiene de WAL; effort 🟡, impacto 🟢. Varios micro-fixes acotados.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb wal` + txn + check/clippy/fmt; sin cambio de comportamiento público
- **Task file:** `skills/campaign-executor/tasks/MOD-06.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: write_shard_meta no atómico → crash consistency (evaluar si es fix o documentar)
  - Fallo 2: refactor de clones toca hot path sin bench
  - **Stop conditions:** si un nit es en realidad un fix de durabilidad grande, separarlo (Regla: no cambiar WAL semantics sin ADR). **Cynefin:** 🟨 complicado — WAL. **Top 3 riesgos:** (1) durabilidad; (2) hot path; (3) scope.

### Task 6: MOD-11 — nits agrupados MCP

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs:879,1028,1389`
- **Verificación real:** 🟡 VERIFICAR — backlog: k sin clamp en search_semantic, timeout no cancela spawn_blocking, total_bytes aproximada, namespace:// limit 100, rutas LLM06 sin threat-model doc. Confirmar en DISCOVERY.
- **Gate Justificación:** higiene MCP; varios micro-fixes. k clamp y timeout son los de mayor valor.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-mcp --test mcp_tests` pasa; k clamp aplicado; docs ×2 hash SAME
- **Task file:** `skills/campaign-executor/tasks/MOD-11.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: cancelar spawn_blocking correctamente (abort vs cooperative)
  - Fallo 2: hash SAME docs
  - **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) cancelación spawn_blocking; (2) hash SAME.

### Task 7: MOD-21 — nits agrupados Python

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-python/src/convert.rs:36-147`, `dist/`
- **Verificación real:** 🟡 VERIFICAR — backlog: artefactos stale commiteados (wheels/.pyd/.pdb), async graph_bfs pierde direction, validación inconsistente, MAX_K clamp silencioso, connect sin read_only/backend. Confirmar en DISCOVERY.
- **Gate Justificación:** higiene Python; effort 🟢. Ahora secuencial (MOD-18/20 ya commiteados en vantadb-python).
- **Gate Result:** ✅ DO
- **Contrato:** pytest pasa; artefactos stale gitignored/removidos; graph_bfs direction expuesto; docs PYTHON_SDK
- **Task file:** `skills/campaign-executor/tasks/MOD-21.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) eliminar artefactos stale rompe alguien; (2) direction async.

### Task 8: MEM-51 — tools L0/L1 sin ejecutor en vanta-proxy

- **Appetite:** max 3h
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴 (porcierto: prioridad original)
- **Archivos clave:** `vanta-proxy/src/{inject,handlers}*.rs`
- **Verificación real:** 🟡 VERIFICAR — backlog: inject.rs anuncia `vanta_memory_capture/search` al modelo pero nada intercepta el tool_call para ejecutarlo; cliente recibe tool call huérfano. Confirmar en DISCOVERY.
- **Gate Justificación:** ⬆️ UP-HILL — requiere decisión de diseño (implementar executor en el stream vs documentar mem-command como única vía). Confirmar approach en DISCOVERY con question al usuario si ambiguo.
- **Gate Result:** ✅ DO
- **Contrato:** tool call `vanta_memory_capture/search` interceptado y ejecutado (o documentado mem-command); `cargo test -p vanta-proxy` verde
- **Task file:** `skills/campaign-executor/tasks/MEM-51.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:**
  - Fallo 1: integrar executor en el stream LLM es invasivo
  - Fallo 2: mem-command como única vía cambia el contrato del agente
  - **Stop conditions:** si requiere rediseño del stream LLM grande → DEFER. **Cynefin:** 🟧 complejo — requiere experimentar. **Top 3 riesgos:** (1) invasivo; (2) contrato; (3) scope.

### Task 9: BND-05 — vantadb-node superficie mínima

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `vantadb-node/src/lib.rs`
- **Verificación real:** 🟡 VERIFICAR — backlog: expone put/get/search/list; faltan graph/explain para paridad con wasm/ts. Confirmar en DISCOVERY.
- **Gate Justificación:** paridad de bindings Node; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** graph/explain expuestos en vantadb-node (o equivalente); build/test node pasa
- **Task file:** `skills/campaign-executor/tasks/BND-05.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) API napi vs wasm.

### Task 10: AGT-02 — verificar stats CodeGraph

- **Appetite:** max 30m
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `.opencode/AGENTS.md` § CodeGraph, `.codegraph/codegraph.db`
- **Verificación real:** 🟡 VERIFICAR — backlog: dice "7.3K símbolos, 24.7K edges"; contrastar contra codegraph.db. Refrescar números o quitarlos.
- **Gate Justificación:** exactitud de docs; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** números de CodeGraph verificados/actualizados en AGENTS.md
- **Task file:** `skills/campaign-executor/tasks/AGT-02.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio.

### Task 11: AGT-03 — spot-check refs deuda P2 (Regla 6)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `.opencode/AGENTS.md` Regla 6 (P2-1, P2-3, P2-5, P2-6, P2-7, P2-8)
- **Verificación real:** 🟡 VERIFICAR — verificar vigencia de refs `file:line`; actualizar o migrar a issues `debt`.
- **Gate Justificación:** exactitud de refs; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** refs P2 verificadas contra código; actualizadas o migradas
- **Task file:** `skills/campaign-executor/tasks/AGT-03.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio.

### Task 12: AGT-04 — limpieza .opencode/opencode-loop/

- **Appetite:** max 30m
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `.opencode/opencode-loop/`
- **Verificación real:** 🟡 VERIFICAR — backlog: 30+ archivos `ses_*.json.corrupt-*` y `.tmp`; borrar corrupt/tmp + rotación.
- **Gate Justificación:** limpieza; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** corrupt/tmp eliminados; rotación agregada al loop server
- **Task file:** `skills/campaign-executor/tasks/AGT-04.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio.

### Task 13: AGT-06 — script anti-drift de referencias

- **Appetite:** max 2h
- **Esfuerzo:** 🟢
- **Prioridad:** 🟢
- **Archivos clave:** `dev-tools/check-agents-refs.ps1` (nuevo)
- **Verificación real:** 🟡 VERIFICAR — backlog: script que valide existencia de rutas citadas en AGENTS.md; enganchar a verify_changed.ps1 o CI Fast Gate.
- **Gate Justificación:** previene stale refs; effort 🟢.
- **Gate Result:** ✅ DO
- **Contrato:** script existe, valida refs, enganchado a verify_changed.ps1
- **Task file:** `skills/campaign-executor/tasks/AGT-06.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio.

### Task 14: UX-16 — dependencia fantasma lucide-react

- **Appetite:** max 30m
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Archivos clave:** `desktop/package.json`, `desktop/src/components/layout/WorkspaceShell.tsx:19`, `desktop/src/components/DataExplorer.tsx:38`
- **Verificación real:** ✅ CÓDIGO-REAL — `codegraph_explore` confirma `import { Trash2, TriangleAlert } from "lucide-react"` (DataExplorer.tsx:38) y uso en WorkspaceShell, pero NO está en `desktop/package.json` (resuelve solo por hoisting). Romperá `npm ci` limpio.
- **Gate Justificación:** bug de packaging real (rompe build limpio/CI); effort 🟢, prioridad 🔴. **El usuario pidió incluir desktop.**
- **Gate Result:** ✅ DO
- **Contrato:** `lucide-react` en desktop/package.json; `npm ci` limpio + `npm run build` exit 0
- **Task file:** `skills/campaign-executor/tasks/UX-16.md`
- **Estado:** ⬜ PENDING

  **Pre-mortem:** —. **Stop conditions:** —. **Cynefin:** 🟦 obvio. **Top 3 riesgos:** (1) versión correcta de lucide-react.

## DEFER

| ID | Motivo |
|----|--------|
| MCP-34 (resto) | `snapshot_restore` NO existe en core — feature core nueva (Arch/Engine), fuera de wrappers MCP |
| FIND-20/21 | Persistencia ventana Tauri + menú contextual — requieren investigación Tauri y decisiones (los dejo para un batch desktop dedicado tras UX-16; effort > impacto inmediato) |

## SKIP

| ID | Motivo |
|----|--------|
| — | — |

## BLOQUEADO

| ID | Motivo |
|----|--------|
| — | — |

## Waves

- **Wave 0** (independientes): FIND-30 · UX-16 · FIND-32
- **Wave 1**: FIND-31 · MCP-34a · MOD-06
- **Wave 2**: MOD-11 · MOD-21 · BND-05
- **Wave 3**: AGT-02 · AGT-03 · AGT-04
- **Wave 4**: AGT-06 · MEM-51 (en solitario, requiere decisión)

> MAX_CONCURRENT = 3. Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea. MOD-21 secuencial tras MOD-18/20 (mismo dir vantadb-python) — ya commiteados, libre.

## Notas

- plan-adjust [2026-08-25]: creado desde docs/Backlog.md. El usuario confirmó: (a) sesiones P34/P37 ya no existen → incluir desktop; (b) eliminar registros de colisión (adenda P34, H2, AGT-05 → ✅ Resuelto).
- Colaterales de batches previos incluidos: FIND-30/31/32 (de REVIEW-17/MOD-04/MOD-13).
- ⬆️ uphill = 1 (MEM-51 decisión de diseño) · ⬇️ downhill = 13
