# Plan de Ejecución: Batch Colaterales + Deuda + Desktop (2026-08-25)

> **Campaign ID:** a226e72e-eb3d-4b5b-b43e-64bc698064a5
> **Inicio:** 2026-08-25
> **Estado:** ✅ COMPLETADO
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
- **Estado:** ✅ COMPLETED (verificado — ya resuelto por MOD-13 `00a85294`, `_ns` en cli_server.rs:1330; 0 diff)

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
- **Estado:** ✅ COMMITTED `fix(storage)` — text index usa include_expired, 2061/2061

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
- **Estado:** ✅ COMMITTED `test(server)` — tests rate-limit alineados a burst=5, 3/3 pass

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
- **Estado:** ✅ COMMITTED `feat(mcp)` — snapshot_create wrapper, 71/71, hash SAME

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
- **Estado:** ✅ COMMITTED `refactor(storage)` — WAL nits, 76/76 wal+txn, PITR documentado

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
- **Estado:** ✅ COMMITTED `fix(mcp)` — clamp k, config list limit, threat model docs, 72/72

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
- **Estado:** ✅ COMMITTED `fix(python)` — graph_bfs direction, clamp_top_k, connect read_only/backend, pytest 132

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
- **Estado:** ✅ COMMITTED `feat(vanta-proxy)` — executor `vanta_memory_capture/search` wired en stream (a9b65224, D46-D48, cap 3 iter)

  **Pre-mortem:**
  - Fallo 1: integrar executor en el stream LLM es invasivo
  - Fallo 2: mem-command como única vía cambia el contrato del agente
  - **Stop conditions:** si requiere rediseño del stream LLM grande → DEFER. **Cynefin:** 🟧 complejo — requiere experimentar. **Top 3 riesgos:** (1) invasivo; (2) contrato; (3) scope.
  - **Verificación 2026-08-25 (MEM-51 batch colaterales):** DISCOVERY confirmó approach (a) YA implementado y commiteado (`a9b65224` "MEM-51 O2 interceptor stream + loop agéntico memory-tools (D46-D48, cap 3 iter)"). El hallazgo del backlog ("inject.rs anuncia tools pero nada intercepta el tool_call") fue el finding ORIGINAL de la Última Milla, resuelto antes de este batch. Contrato verificado: `cargo test -p vanta-proxy` 92/92 ✅ (72 lib + 5 tool_loop + 10 wire + 5 pipeline) · fmt 0 · clippy -D warnings 0 · check 0. Doc auditoría docs/reviews/modulos/vanta-proxy.md:28 confirma "ejecutadas server-side en el loop SSE". 0 diff — patrón FIND-30.

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
- **Estado:** ✅ COMMITTED `feat(node)` — 12 napi graph/explain, npm 8/8

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
- **Estado:** ✅ COMMITTED `docs` — stats CodeGraph 20.5K/71.4K (verificado: codegraph status + sqlite)

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
- **Estado:** ✅ COMMITTED \docs\ - P2 refs verificadas (4 resueltas, 2 actualizadas)

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
- **Estado:** ✅ COMMITTED chore - 96 corrupt/tmp eliminados, GC + script clean-opencode-loop.ps1

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
- **Estado:** ✅ COMMITTED \chore\ - check-agents-refs.ps1 + hook verify_changed, 22 refs OK

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
- **Estado:** ✅ COMMITTED `fix(desktop)` — lucide-react ^1.34.0 declarado, npm ci + build exit 0

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

=== RECITATION FIND-30 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: FIND-30: unused var ns en cli_server.rs:1302 rompe clippy -D warnings bajo feature server
Estado: completed
Última acción: DISCOVERY: codegraph + Read cli_server.rs:1270-1339 confirmaron closure options_for con param _ns ya commiteado por MOD-13 (00a85294). Verifiqué contrato clippy + check con --features server.
Resultado: OK
Próxima acción: Task 2 del plan: FIND-31 (purge_expired tras reopen)
Contrato: verificacion: cargo clippy -p vantadb --features server --all-targets -- -D warnings -> Finished sin warnings (✅); cargo check -p vantadb --features server --all-targets -> Finished (✅). evidencia: claim: fix presente y clippy server pasa | evidencia: git log -S _ns -> 00a85294 + clippy output Finished | confianza: alta. artefactos: .opencode/skills/campaign-executor/tasks/FIND-30.md. invariantes: ninguna (cambio cosmético ya commiteado, sin edición nueva). deuda: ninguna. queda_pendiente: lead marca task DO con 0 diff; fix real entró con MOD-13
Próxima tarea si completa: FIND-31
=== END RECITATION ===

=== RECITATION UX-16 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: UX-16: declarar dependencia fantasma lucide-react en desktop/package.json
Estado: completed
Última acción: DISCOVERY: 14 archivos en desktop/src importan lucide-react (WorkspaceShell.tsx:19, DataExplorer.tsx:38, +12), package.json sin declaración, lockfile sin entry, node_modules hoisted 1.34.0. Liberé lock stale de FIND-06 (ya commiteado). ACT: agregué lucide-react@^1.34.0 a dependencies, regeneré lockfile, verifiqué npm ci + npm run build exit 0.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea desktop/package.json + desktop/package-lock.json (sub-agente NO commitea). Review P2-01 delegado a vanta-review.
Contrato: verificacion: cd desktop && npm ci (added 484 packages, exit 0) && npm run build (tsc+vite, 2861 modules, exit 0) — ✅. evidencia: [{claim: lucide-react declarado en desktop/package.json, evidencia: desktop/package.json:28 (^1.34.0), confianza: alta}, {claim: npm ci limpio + build pasan tras declarar la dep, evidencia: salida real exit 0, confianza: alta}, {claim: version = hoisted 1.34.0, evidencia: desktop/node_modules/lucide-react/package.json, confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/UX-16.md. invariantes: no tocar imports/logica en desktop/src; no alterar otras deps. deuda: ninguna. queda_pendiente: lead commitea los 2 archivos + skill progreso
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION MCP-34a ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: MCP-34a: wrapper MCP snapshot_create
Estado: completed
Última acción: Liberación de lock stale: MCP-34a ya commiteado por lead (0dc57a60 feat(mcp): snapshot_create tool). Plan file registra COMMITTED. El task system retenía in-progress bloqueando la siguiente tarea (MOD-11).
Resultado: OK
Próxima acción: MOD-11 nits MCP H4-H8
Contrato: verificacion: git log muestra 0dc57a60 (commit MCP-34a); plan file Task 4 Estado: COMMITTED. evidencia: [{claim: MCP-34a commiteado, evidencia: git log 0dc57a60 + plan file línea 84, confianza: alta}]. artefactos: ninguno nuevo. invariantes: ninguna. deuda: ninguna. queda_pendiente: ejecutar MOD-11
Próxima tarea si completa: MOD-11
=== END RECITATION ===

=== RECITATION FIND-31 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: FIND-31: purge_expired tras reopen falla 'text index df would go negative'
Estado: completed
Última acción: Root cause real encontrado y fix aplicado: rebuild/reconcile del text index usaba memory_record_from_node (aplica lazy TTL eviction), descartando records expirados-no-purgados y sus term-stats; luego purge los decrementa y subdesborda. Fix: memory_record_from_node_include_expired en rebuild_text_index_with_report, expected_text_index_counts_from y expected_text_index_entries. Test de regresión put->flush->reopen->purge pasa. Verificación en worktree limpio HEAD (WIP de MOD-06 rompe lib test en el árbol principal).
Resultado: OK
Próxima acción: Lead: verifica mecánico (git diff de los 3 archivos + suite), delega review P2-01 a vanta-review, y commitea los 3 archivos (api.rs, impl_rebuild.rs, impl_text_index.rs). Aviso: wal_sharded.rs/engine/insert/delete/txn tienen WIP sin commit de MOD-06 que rompe cargo nextest -p vantadb en el árbol principal.
Contrato: {"verificacion": "cargo nextest run -p vantadb --build-jobs 2 -> 2061 passed, 1 skipped (worktree limpio); test_purge_expired_after_reopen -> PASS (RED fallaba pre-fix con ValidationError df negative); cargo check -p vantadb --all-targets -> Finished; cargo fmt --check -> exit 0; cargo clippy -p vantadb --all-targets -- -D warnings -> exit 0", "evidencia": [{"claim": "root cause: memory_record_from_node aplica lazy TTL (serialization/mod.rs:359-368) y descarta expirados-no-purgados en el rebuild del text index", "evidencia": "src/sdk/serialization/mod.rs:322-425 + diagnóstico empírico (session1 TextIndex=10, raw reopen=10, sdk reopen=0, manual rebuild record=0)", "confianza": "alta"}, {"claim": "fix: usar memory_record_from_node_include_expired en los 3 paths del text index", "evidencia": "src/sdk/serialization/impl_rebuild.rs:105,194 + impl_text_index.rs:125", "confianza": "alta"}, {"claim": "test RED -> GREEN", "evidencia": "src/sdk/api.rs:2014 test_purge_expired_after_reopen_with_indexed_payload (pre-fix ValidationError, post-fix PASS)", "confianza": "alta"}], "artefactos": [".opencode/skills/campaign-executor/tasks/FIND-31.md", "src/sdk/api.rs", "src/sdk/serialization/impl_rebuild.rs", "src/sdk/serialization/impl_text_index.rs"], "invariantes": "records sin payload indexado siguen purgados; firma purge_expired intacta; no tocar wal.rs/storage/vector/", "deuda": "derived index (namespace/payload) y sparse index también usan memory_record_from_node en sus rebuilds; omiten expirados-no-purgados pero no causan crash (deletes idempotentes) — follow-up de consistencia opcional", "queda_pendiente": "lead verifica combinado + review P2-01 + commit; verificar que el árbol principal compila cuando MOD-06 commitee su WIP (wal_sharded.rs actualmente rompe lib test)"}
Próxima tarea si completa: Task 4 del plan: MCP-34a
=== END RECITATION ===

=== RECITATION MOD-06 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: MOD-06: nits agrupados WAL
Estado: completed
Última acción: Implementación completa: 5 archivos editados, todos los verify pasan (wal+txn 76/76, recovery 14/14, cardinality 28/28, clippy, fmt)
Resultado: OK
Próxima acción: Lead: verificar mecánico y commitear
Contrato: verificacion: cargo nextest run -p vantadb -E 'test(wal)|test(txn)' -> 76/76 pass; cargo check -p vantadb -> Finished; rustfmt --check (5 archivos) -> exit 0; cargo clippy -p vantadb --all-targets -- -D warnings -> Finished; recovery/durab 14/14; cardinality+batch+traverse 28/28. evidencia: [{claim: flush_all secuencial sin spawn de threads, evidencia: src/wal_sharded.rs:285, confianza: alta}, {claim: batch_append mueve Vec<WalRecord> sin clones, evidencia: src/wal_sharded.rs:215 + 3 callers actualizados, confianza: alta}, {claim: write_shard_meta atómico temp+rename, evidencia: src/wal_sharded.rs:107, confianza: alta}, {claim: lookup hoisted del BFS loop, evidencia: src/engine.rs:383, confianza: alta}, {claim: bump_cardinality dedup, evidencia: src/storage/engine/insert.rs:186, confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/MOD-06.md. invariantes: semántica round-robin intacta; WalWriter público NO tocado; orden por shard preservado (moves). deuda: PITR sin wiring — ADR humano pendiente, documentado en task file + Backlog PRO-03. queda_pendiente: lead verifica mecánico y commitea los 5 archivos (src/wal_sharded.rs, src/engine.rs, src/storage/engine/{insert,txn,delete}.rs); api.rs diff en worktree es de FIND-31, NO incluirlo
Próxima tarea si completa: MOD-11
=== END RECITATION ===

=== RECITATION BND-05 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: BND-05: exponer graph/explain en vantadb-node para paridad con wasm/ts
Estado: completed
Última acción: Implementé 12 métodos napi (insert_node, get_node, delete_node, add_edge, remove_edge, graph_bfs, graph_dfs, graph_topological_sort, graph_is_dag, graph_filtered_traversal, graph_degree, explain_search) + helpers de parseo estricto; rebuild napi; tests vitest nuevos (5). Fix colaterales de compilación: parse_search_request + exclude_superseded/search_profile (campos nuevos SDK) y get_opt_u64 clippy pre-existente.
Resultado: OK
Próxima acción: Lead: verifica mecánico (cargo check/fmt/clippy + npm test) y commitea los 5 archivos; delega review P2-01 a vanta-review/vanta-audit antes de cerrar
Contrato: verificacion: cargo check --manifest-path vantadb-node/Cargo.toml -> Finished (exit 0); cargo fmt --manifest-path vantadb-node/Cargo.toml --check -> exit 0; cargo clippy --manifest-path vantadb-node/Cargo.toml --all-targets -- -D warnings -> exit 0; cd vantadb-node && npm run build (cjs+esm) -> Finished release exit 0; cd vantadb-node && npm test -> vitest 2 files / 8 tests pass. evidencia: [{claim: 12 métodos graph/explain expuestos en lib.rs, evidencia: vantadb-node/src/lib.rs + index.d.ts regenerado (graphBfs/graphDfs/graphTopologicalSort/graphIsDag/graphFilteredTraversal/graphDegree/explainSearch/insertNode/getNode/deleteNode/addEdge/removeEdge), confianza: alta}, {claim: tests graph/explain pasan en runtime nativo, evidencia: npm test -> 8/8 pass (5 nuevos graph.test.ts), confianza: alta}, {claim: ids u128 como strings decimales (serde_json no serializa u128>u64; JS pierde precisión >2^53), evidencia: helpers parse_node_id/parse_node_ids + tests, confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/BND-05.md; vantadb-node/src/lib.rs; vantadb-node/tests/graph.test.ts; vantadb-node/index.cjs; vantadb-node/index.js; vantadb-node/index.d.ts. invariantes: métodos existentes intactos; op-gate + spawn_blocking en toda op nueva; ids como strings; sin unsafe/unwrap; no tocar core/wasm/ts. deuda: vantadb-ts/src/native.ts sin graph (fuera de scope); docs narrativos node (vanta-docs). queda_pendiente: lead verifica + review P2-01 + commit (sub-agente NO commitea); .node binario gitignored se regenera local, no commitea
Próxima tarea si completa: AGT-02
=== END RECITATION ===

=== RECITATION MOD-11 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: MOD-11: nits agrupados MCP server (H4-H8 de P32)
Estado: completed
Última acción: Implementación completa de 5 nits: H4 clamp k en search_semantic (min(config.max_top_k)) + test test_mcp_search_semantic_clamps_k; H5 comentario limitación timeout spawn_blocking en server.rs (sin abort, invasivo); H6 comentario total_bytes como estimación deliberada en collection_stats; H7 resources.rs usa config.default_list_limit en vez de hardcode 100; H8 nota threat model LLM06 en SKILL.md § Security. Docs x2 editadas idénticas. Verify: 72 passed (era 71 + clamp test), fmt 0, clippy -D warnings 0, check 0, hash SAME DF1A68FA.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea los 6 archivos
Contrato: verificacion: cargo test -p vantadb-mcp --test mcp_tests -> 72 passed ✅; cargo fmt --check -p vantadb-mcp -> exit 0 ✅; cargo clippy -p vantadb-mcp --all-targets -- -D warnings -> Finished ✅; cargo check -p vantadb-mcp -> Finished ✅; Get-FileHash x2 SKILL.md -> DF1A68FAAFEDEDC61E13284696001BE5921E7FDBBA SAME ✅. evidencia: [{claim: k clamp aplicado, evidencia: tools.rs:1002 min(config.max_top_k) + test pasa, confianza: alta}, {claim: docs x2 hash SAME, evidencia: SHA256 DF1A68FA ambos, confianza: alta}, {claim: H5/H6/H7/H8 documentados sin cambio de comportamiento, evidencia: diffs solo comments + default_list_limit (mismo valor 100), confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/MOD-11.md; vantadb-mcp/src/handlers/tools.rs; vantadb-mcp/src/server.rs; vantadb-mcp/src/handlers/resources.rs; vantadb-mcp/tests/mcp_tests.rs; skills/vantadb-mcp/SKILL.md; .opencode/skills/vantadb-mcp/SKILL.md. invariantes: firma pública intacta; semántica search_vector/search_memory intacta; no toqué wal/vector/storage. deuda: H5 cancelación real de spawn_blocking queda como limitación documentada (cooperative CancellationToken = feature core futura). queda_pendiente: lead commitea por tarea (git add 6 archivos: 3 rust src + 1 test + 2 SKILL.md); NOTA: task system reporta lock in-progress BND-05 (otro sub-agente paralelo en vantadb-node) — no bloquee el commit de MOD-11, son archivos disjuntos
Próxima tarea si completa: MOD-21
=== END RECITATION ===

=== RECITATION MOD-21 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: MOD-21: nits agrupados Python — 3 fixes (connect read_only/backend, async direction, MAX_K clamp warning) + 2 ya-resueltos documentados
Estado: completed
Última acción: 3 fixes aplicados + stubs + docs PYTHON_SDK + anti-drift test actualizado; suite pytest completa 132 passed tras maturin develop --release; fmt/clippy/check verdes
Resultado: OK
Próxima acción: Lead: verifica mecánico (git diff 6 archivos) y commitea; cerrar MCP-34a en state machine para desbloquear MOD-21 (bloqueo one-task-at-a-time)
Contrato: verificacion: python -m pytest tests/ -> 132 passed; cargo check -p vantadb_py -> Finished; cargo fmt --check -p vantadb_py -> exit 0; cargo clippy -p vantadb_py --all-targets -- -D warnings -> Finished sin warnings
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION AGT-02 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: AGT-02: números de CodeGraph verificados/actualizados en AGENTS.md § CodeGraph
Estado: completed
Última acción: Verifiqué números reales del index con 2 fuentes independientes (codegraph status CLI: nodes 20.496/edges 71.446; sqlite read-only: nodes 20496/edges 71446) y actualicé línea 67 de .opencode/AGENTS.md: 7.3K/24.7K -> 20.5K/71.4K con fecha+comando de verificación. Línea 368 (§ MCP Servers) fuera de scope, intacta, documentada como deuda.
Resultado: OK
Próxima acción: Lead: verifica mecánico (grep 20.5K/71.4K en línea 67 + codegraph status) y commitea .opencode/AGENTS.md; decidir sobre línea 368 stale
Contrato: {"verificacion": "rg -n '20\\.5K|71\\.4K|7\\.3K|24\\.7K' .opencode/AGENTS.md -> línea 67 actualizada, línea 368 fuera de scope intacta; codegraph status -> Nodes 20.496 / Edges 71.446 (index up to date); python cg_stats.py (sqlite ro) -> nodes 20496, edges 71446", "evidencia": [{"claim": "Index real: 20,496 nodes / 71,446 edges", "evidencia": "codegraph status (CLI) + SELECT COUNT(*) FROM nodes/edges (sqlite read-only) — ambos coinciden", "confianza": "alta"}, {"claim": "AGENTS.md § CodeGraph actualizado con números reales", "evidencia": ".opencode/AGENTS.md:67 (20.5K símbolos, 71.4K edges — verificado 2026-08-25 vía codegraph status)", "confianza": "alta"}, {"claim": "Solo § CodeGraph tocado; otras secciones intactas", "evidencia": ".opencode/AGENTS.md:368 conserva 7.3K (fuera de scope, deuda documentada); git diff limitado a 1 línea", "confianza": "alta"}], "artefactos": [".opencode/AGENTS.md (línea 67)", ".opencode/skills/campaign-executor/tasks/AGT-02.md"], "invariantes": "solo § CodeGraph editado; línea 368 y demás secciones intactas; .codegraph/codegraph.db no modificado", "deuda": "línea 368 § MCP Servers conserva '7.3K símbolos' stale — fuera de scope, lead decide micro-fix (o quitar número: drift inevitable con cada indexación)", "queda_pendiente": "lead verifica mecánico y commitea .opencode/AGENTS.md (sub-agente NO commitea — regla plan); AGT-03 siguiente"}
Próxima tarea si completa: AGT-03
=== END RECITATION ===

=== RECITATION AGT-04 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: AGT-04: limpiar .opencode/opencode-loop/ de corrupt/tmp + agregar rotación al loop server
Estado: completed
Última acción: Step 1: borrados 96 archivos (88 .corrupt-* + 8 .tmp), 0 residuales, sesiones vivas intactas (1261). Step 2: GC agregado al plugin global opencode-loop.ts (cleanupLoopStateDir L407 + llamada en init L1990, matchea solo *.corrupt-* y *.tmp) + script nuevo dev-tools/clean-opencode-loop.ps1 (dry-run/apply, idempotente, 2x corrido -> 0 residuales)
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea dev-tools/clean-opencode-loop.ps1 + task file AGT-04.md (parche del plugin es global, no-commiteable: vive en ~/.config/opencode/plugins/opencode-loop.ts)
Contrato: {"verificacion": "Get-ChildItem .opencode/opencode-loop -File | Where Name -match '\.corrupt-[\\d]+$|\.tmp
 | Measure-Object -> 0 residuales (antes 96); ses_*.json = 1261 intactos; plugin L407+L1990 cleanupLoopStateDir presente; dev-tools/clean-opencode-loop.ps1 dry-run+apply x2 exit 0", "evidencia": [{"claim": "96 corrupt/tmp eliminados, 0 residuales", "evidencia": "conteo regex .corrupt-[\d]+$ y .tmp$ antes 96 / despues 0", "confianza": "alta"}, {"claim": "sesiones vivas no tocadas", "evidencia": "ses_*.json = 1260 antes, 1261 despues (sesion activa del loop creo su estado)", "confianza": "alta"}, {"claim": "GC en loop server (plugin opencode-loop)", "evidencia": "~/.config/opencode/plugins/opencode-loop.ts L407 funcion cleanupLoopStateDir + L1990 llamada en OpenCodeLoopPlugin init", "confianza": "alta"}, {"claim": "script cleanup idempotente", "evidencia": "dev-tools/clean-opencode-loop.ps1 dry-run exit 0, apply #1 exit 0 residual 0, apply #2 exit 0 residual 0", "confianza": "alta"}], "artefactos": ["dev-tools/clean-opencode-loop.ps1", ".opencode/skills/campaign-executor/tasks/AGT-04.md"], "invariantes": "GC solo matchea *.corrupt-* y *.tmp; nunca ses_*.json vivos, loop.log, goals/; semantica writeState/readState intacta", "deuda": "plugin global no-commiteable; si oc plugin install sobreescribe opencode-loop.ts el GC se pierde — dev-tools/clean-opencode-loop.ps1 queda como fallback manual", "queda_pendiente": "lead verifica y commitea los 2 archivos del repo; parche plugin documentado en task file"}
Próxima tarea si completa: AGT-06
=== END RECITATION ===

=== RECITATION AGT-03 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: AGT-03: verificar vigencia de refs file:line de deuda P2 (Regla 6) en .opencode/AGENTS.md y actualizar o migrar
Estado: completed
Última acción: DISCOVERY: verifiqué cada ref P2 contra código real (grep/codegraph). ACT: actualicé tabla Regla 6 en .opencode/AGENTS.md:474-480 — P2-1, P2-3, P2-6, P2-7 marcadas RESUELTO (patrón P2-2); P2-5 ref 312→494 (dual API sigue); P2-8 ref 402-433→564-596 (O(n) sigue). Decisión ponytail: mantener tabla (solo 2 items vigentes), no migrar a issues. Creé task file AGT-03.md con Impacto mapeado. VERIFY: refs vigentes apuntan a código real.
Resultado: OK
Próxima acción: Lead: verifica mecánico y commitea .opencode/AGENTS.md + task file AGT-03.md (sub-agente NO commitea). AGT-06 siguiente.
Contrato: verificacion: rg -n 'fn put_batch' vantadb-python/src/lib.rs -> 494; rg -n 'fn collect_all_deduped' vantadb-wasm/src/lib.rs -> 564; rg -n 'pub async fn delete' vantadb-wasm/src/opfs.rs -> 101 (implementado, no stub); rg -n 'LruCache|AUD-039' convert.rs -> 48/700 (O(1)); rg -n '_ => VantaError::new_err' convert.rs -> 818 (catch-all); rg -n 'AUD-023 (P2-7)' src/sdk/serialization/mod.rs -> 298/1620. Todos apuntan a código real. evidencia: [{claim: P2-1 resuelto, evidencia: opfs.rs:101-104 delete() implementado con remove, confianza: alta}, {claim: P2-3 resuelto, evidencia: convert.rs:48 LruCache::new + comentario AUD-039 O(1) en 700, confianza: alta}, {claim: P2-6 resuelto, evidencia: convert.rs:818 catch-all _ en map_vanta_error, confianza: alta}, {claim: P2-7 resuelto por refactor, evidencia: mod.rs:298/1620 comentarios AUD-023 (P2-7), región 227-294 ahora encoding tipado por campo, confianza: media}, {claim: P2-5 vigente ref actualizada, evidencia: lib.rs:494 fn put_batch, dual API 507-560+, confianza: alta}, {claim: P2-8 vigente ref actualizada, evidencia: lib.rs:564 fn collect_all_deduped, confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/AGT-03.md; .opencode/AGENTS.md (tabla 474-480). invariantes: solo tabla Regla 6 editada; Regla 4 menciones genéricas P2 intactas (sin file:line stale); no borrada la tabla (Regla 0). deuda: ninguna. queda_pendiente: lead verifica y commitea los 2 archivos; AGT-06 siguiente.
Próxima tarea si completa: AGT-06
=== END RECITATION ===

=== RECITATION MEM-51 ===
Campaign ID: a226e72e-eb3d-4b5b-b43e-64bc698064a5
Objetivo activo: MEM-51: implementar executor de vanta_memory_capture/search en el stream de vanta-proxy (o documentar mem-command)
Estado: completed
Última acción: DISCOVERY completo: leí inject.rs, memory_tools.rs, server.rs, forward.rs, task file previo, doc auditoría y git log. Decisión de diseño tomada: approach (a) — executor en el stream — YA implementado y commiteado en a9b65224 (2026-08-22, batch Última Milla). El hallazgo del backlog (tools anunciadas sin interceptor) fue el finding ORIGINAL ya resuelto. Verifiqué contrato mecánico completo: cargo test -p vanta-proxy 92/92, fmt 0, clippy -D warnings 0, check 0. Actualicé plan file (Task 8 → COMMITTED con evidencia) y task file (sección Verificación 2026-08-25). 0 diff — nada que commitear.
Resultado: OK
Próxima acción: Lead: revisar evidencia en plan file Task 8 + task file; 0 diff así que no hay commit pendiente; cerrar tarea en state machine
Contrato: verificacion: cargo test -p vanta-proxy -> 92/92 pass (72 lib + 5 tool_loop + 10 proxy_wire + 5 pipeline) ✅; cargo fmt -p vanta-proxy --check -> exit 0 ✅; cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings -> exit 0 ✅; cargo check -p vanta-proxy --all-targets -> exit 0 ✅. evidencia: [{claim: executor vanta_memory_capture/search existe y está wired en el stream, evidencia: vanta-proxy/src/memory_tools.rs (execute/extract/append_exchange) + server.rs:247-342 forward_with_tool_loop + inject.rs:26-35 TOOL_SPECS + commit a9b65224, confianza: alta}, {claim: approach (a) es el correcto según contrato de producto (agente captura memoria vía tool), evidencia: docs/reviews/modulos/vanta-proxy.md:28 'ejecutadas server-side en el loop SSE' + tests tool_loop.rs (a) capture loop y (b) search loop, confianza: alta}, {claim: contrato cargo test -p vanta-proxy verde, evidencia: salida real 92 tests pass, confianza: alta}, {claim: 0 diff, tarea ya commiteada, evidencia: git status --short -- vanta-proxy/ vacío + git log a9b65224, confianza: alta}]. artefactos: .opencode/skills/campaign-executor/tasks/MEM-51.md (sección Verificación 2026-08-25 agregada); docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 8 estado COMMITTED). invariantes: no tocar wal.rs/storage/vector; vanta-proxy core intocable (crate-local). deuda: M-4 mezcla our-tool+client-tool sin responder upstream (→ MOD-39 backlog); S-2 drain SSE sin cap de memoria (→ FIND backlog) — ambas pre-existentes, documentadas en auditoría. queda_pendiente: ninguna — task cerrada con evidencia; próximo batch puede ejecutar AGT-03/AGT-06
Próxima tarea si completa: AGT-03
=== END RECITATION ===
