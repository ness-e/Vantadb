# MCP-15: S5 — Stack overflow del child vantadb-server durante search_semantic/search_memory

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 1 — hallazgo post-batería)
- **Fuente:** Backlog P22 `MCP-15` (agregado 2026-08-17 por MCP-03/MCP-04)
- **Esfuerzo:** 🔴 1d
- **Prioridad:** 🔴
- **Tipo:** Rust core (bug, stack overflow / recursión)
- **Turns estimados:** 20-40
- **Creado:** 2026-08-17T19:00
- **last-synced:** 2026-08-17T22:10
- **Estado:** ✅ DONE (implementación + verificación) — GATE review vanta-audit pendiente
- **Incógnitas (uphill):** 0 — root cause identificada
- **Pendientes (downhill):** 4 steps (todos DONE)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | batería test-busqueda.py completa (T17/T18/T19 ahora pasan), cualquier cliente que use search_semantic/search_memory tras N queries |
| Callees | child `vantadb-server` (tokio-rt-worker), `src/storage/engine/get.rs` (`get()` → `prefetch_related` → `self.get()`), cache warmer |
| Implicaciones | server ya NO crashea con pipe roto (`[Errno 22]`); búsquedas válidas funcionan indefinidamente |

## Evidencia (detectado independientemente por MCP-03 y MCP-04)
- Crash: `thread 'tokio-rt-worker' (18480) has overflowed its stack` en stderr del child (capturado en `C:\Users\Eros\AppData\Local\Temp\opencode\crash-stderr.txt`)
- Ocurre tras secuencia T09-T16 con dim VÁLIDA (4-dim) — NO es falta de validación (MCP-04 ya corta dims inválidas)
- Bump `thread_stack_size(8 MiB)` probado por MCP-03 → NO resuelve (recursión real, no stack chico)
- Repro 100%: `python C:\Users\Eros\AppData\Local\Temp\opencode\repro-full.py`
- Runs 2/3 originales (pre-fix) ya crasheaban idéntico → pre-existente, no regresión de MCP-01..04
- Aislamiento adicional (2026-08-17): `sem` (search_semantic solo) PASA; `sem15` (tras 9 search_memory) CRASHA; `mem` (solo search_memory) PASA; `RUST_MIN_STACK=512MB` y stack de 1GB en test NO resuelven → **recursión infinita, no stack chico**. Test Rust directo (sin tokio) reproduce el crash → el bug está en el core, no en el worker tokio.

## Root cause (CONFIRMADA 2026-08-17)
- **Archivo:** `src/storage/engine/get.rs`
- **Mecanismo:** `get(id)` (cache miss) → `prefetch_related(id)` → para cada warm id llama recursivamente `self.get(warm_id)`. Si un par co-accesado (A↔B) tiene AMBOS nodos en cache miss, la cadena es `get(A)→prefetch(A)→get(B)→prefetch(B)→get(A)→…` sin término: `get()` NUNCA inserta el nodo que materializa (solo la cola de `prefetch_related` lo hace, tras desenrollar la recursión), así que A y B quedan mutuamente no cacheados toda la cadena → stack overflow en el worker.
- **Fix:** guard de re-entrancia `PrefetchGuard` (thread_local `Cell<bool>` + RAII) en `prefetch_related` → prefetch de UN solo nivel (contrato OLD-20: prefetch los co-accesados del nodo accedido, no transitivamente). RAII libera la flag en unwind (panic mid-prefetch no deja el prefetch deshabilitado).
- **No aplica** la hipótesis original de recursión en serialización del node payload — el node (`VantaNodeRecord`) es plano; la recursión es del warm-up de cache, no de serde.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** el fix NO rompe MCP-01..04 (34/34 tests mcp_tests verdes); no toca semántica de distance (MCP-03) ni validación de dims (MCP-04); prefetch sigue funcionando (test verifica que el co-accesado queda cacheado)
- **Comandos de verificación:** `python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` → **20/20** ✅; `cargo test -p vantadb-mcp --test mcp_tests` → **34/34** ✅ (33 de MCP-01..04 + test 34º de T15); `cargo check -p vantadb-mcp -p vantadb-server` ✅
- **Deuda pendiente:** T15 (explain shape) resuelta en paralelo — NO es falla del server; `test-busqueda.py` T15 fue actualizado al shape canónico (explanation anidado con identity/snippet/bm25_terms/rrf_*, sin route/fusion_report top-level)

## Steps

### Step 1: Repro directo y trazar la recursión
- **Archivos:** `repro-full.py`, `crash-stderr.txt`, stderr del child, `vantadb-mcp/tests/mcp15_repro.rs` (diagnóstico temporal, eliminado tras verificación)
- **Acción:** repro directo; backtrace; aislamiento por secuencia (sem/sem15/mem); test Rust directo sin tokio con stack 1GB → recursión infinita en core
- **Verify:** frame repetido identificado — la recursión es `get()→prefetch_related→self.get()` (no serialización)
- **Estado:** ✅ DONE

### Step 2: Root cause — romper la recursión
- **Archivos:** `src/storage/engine/get.rs` (fix `PrefetchGuard`, 47 líneas); `src/storage/engine/tests/engine.rs` (regression test)
- **Acción:** guard de re-entrancia single-level en `prefetch_related`; regression test `test_get_prefetch_does_not_recurse_forever` (cold-tier A↔B, 3 co-access records)
- **Verify:** `cargo check -p vantadb-mcp -p vantadb-server` limpio ✅; repro-full.py ya no crashea ✅; regression test pasa ✅
- **Estado:** ✅ DONE

### Step 3: Batería completa
- **Archivos:** test-busqueda.py
- **Acción:** re-ejecutar `python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` → 20/20
- **Verify:** **20/20** ✅ (T17/T18/T19 pasan; T15 pasa con shape canónico — script actualizado a explanation anidado)
- **Estado:** ✅ DONE

### Step 4: Regresión MCP
- **Archivos:** vantadb-mcp/tests/mcp_tests.rs (suite completa)
- **Acción:** `cargo test -p vantadb-mcp --test mcp_tests` → 34/34 (33 MCP-01..04 + 1 T15); regression test del crash vive en el core (`src/storage/engine/tests/engine.rs`), no en mcp_tests
- **Verify:** **34/34** ✅
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (MCP-01..04 ya merged; T15 resuelta en paralelo)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit (código core, recursión)
- **Enfoque:** ¿la recursión está realmente rota o solo mitigada? (el guard corta la cadena A↔B — verify que no hay otro path de re-entrancia); ¿el thread_local + RAII introduce riesgo en otros paths (multi-thread, panic, spawn_blocking)?; ¿prefetch single-level degrada co-acceso legítimo multi-nivel?
- **Veredicto:** ⏳ pendiente

## Notas
- Contexto de MCP-03 (sesión que detectó el bug): sesión `ses_fef477842ffeRjAYwesSL1dU1z` tiene el diagnóstico previo — reusar su task_id si el ejecutor necesita el contexto
- `vantadb-mcp/tests/mcp15_repro.rs` fue diagnóstico TEMPORAL y se eliminó tras verificación (el regression test permanente vive en `src/storage/engine/tests/engine.rs`; la batería 20/20 + suite 34/34 prueban el fix end-to-end)
- `test-busqueda.py` (T15) fue actualizado del shape viejo (`explanation.route`) al shape canónico de T15 (explanation anidado) — el server no cambió en ese aspecto

## Context Save Point
- **Fecha:** 2026-08-17T22:10
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** fix = guard de re-entrancia single-level (prefetch deja de ser transitivo); NO cambiar serialización
- **Problemas conocidos:** ninguno
- **Próxima tarea:** GATE review vanta-audit → merge