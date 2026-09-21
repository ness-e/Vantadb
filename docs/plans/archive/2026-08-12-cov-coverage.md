# Plan de Ejecución: COV Test Coverage (2026-08-12)

> **Inicio:** 2026-08-12
> **Estado:** ✅ COMPLETO (4/4)
> **Fuente:** docs/Backlog.md § Phase 3 (líneas 99-104), entrada `/pipeline task COV-001 COV-002 COV-003 COV-004`
> **Gate:** 4 tareas DO, independientes (sin dependencias entre sí)

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 4  | 0     | 0    | 0          |

## Contrato global
Cada tarea verifica con su propio comando (pytest / vitest+c8 / cargo nextest / ADR grep). Sin romper APIs existentes ni cambiar lógica de producto.

### Task 1: COV-001 — Python AsyncVantaDB smoke test
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Ruta:** vanta-worker (bindings Python)
- **Archivos clave:** `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/tests/`
- **Gate Justificación:** coverage del wrapper Python 69%; el path async (`AsyncVantaDB`) solo ejercita sync hoy.
- **Gate Result:** ✅ DO
- **Contrato:** test async de humo que ejercita `flush`, `purge_expired`, `query`, `graph_*`, `put`, `delete`, `export_*` (las ~37 líneas faltantes); `pytest` pasa. NO cambiar la API pública.
- **Task file:** `.opencode/skills/campaign-executor/tasks/COV-001.md` (creado + completado)
- **Estado:** ✅ COMPLETED (smoke test en `vantadb-python/tests/test_async_smoke.py` — 3 tests pasan; async ya cubierto en test_sdk.py)

### Task 2: COV-002 — TS coverage measurement
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker (bindings TS/WASM)
- **Archivos clave:** `vantadb-ts/vitest.config.ts`, `vantadb-ts/src/`, `vantadb-ts/test-runner.mjs`
- **Gate Justificación:** runtime de `src/` (vantadb.ts, native.ts, errors.ts, guards.ts) está 0% medible por incompat `vite-plugin-wasm` ↔ `vitest`.
- **Gate Result:** ✅ DO
- **Contrato:** resolver la incompatibilidad (o reportar coverage con `c8` desde `test-runner.mjs`) de modo que `src/` TS tenga un número de coverage reportado; 25/26 tests siguen pasando. Sin romper el runner alterno existente.
- **Task file:** `.opencode/skills/campaign-executor/tasks/COV-002.md` (creado en DISCOVERY)
- **Estado:** ✅ COMPLETED

### Task 3: COV-003 — Rust CLI binary tests
- **Esfuerzo:** 🟡 | **Prioridad:** 🟢 | **Ruta:** vanta-worker (Rust core)
- **Archivos clave:** `src/cli_handlers/*`, `src/bin/vanta-cli.rs`, `src/sdk/gds.rs`, `tests/`
- **Gate Justificación:** ~2.500 ln de handlers CLI al 0%; root coverage 81.40% → ~88%.
- **Gate Result:** ✅ DO
- **Contrato:** tests unit/integración de subcomandos CLI (crud/search/diagnostics/server/migrate) que levantan root coverage ~7 puntos (hacia ~88%); `cargo nextest` pasa. Alcance acotado: suficientes asserts para el salto, no 100% de 2.500 ln.
- **Task file:** `.opencode/skills/campaign-executor/tasks/COV-003.md` (a crear en DISCOVERY)
- **Estado:** ✅ DONE

### Task 4: COV-004 — ADR coverage gate policy
- **Esfuerzo:** 📖 | **Prioridad:** 🟡 | **Ruta:** vanta-arch (decisión ADR)
- **Archivos clave:** `.github/workflows/ci-rust-10.yml`, ADR en `docs/architecture/adr/`
- **Gate Justificación:** políticas de gate de coverage no documentadas (root 81.40% hoy pasa vs --workspace 72.76%).
- **Gate Result:** ✅ DO
- **Contrato:** ADR que decide root-crate vs --workspace para el gate de CI, con la migración de medición de bindings a runners nativos si aplica. DOC-ONLY: NO modificar `ci-rust-10.yml` más allá de documentar; la implementación del gate queda fuera de scope.
- **Task file:** `.opencode/skills/campaign-executor/tasks/COV-004.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (ADR-018 creada, DOC-ONLY)

## Dependencias
- Ninguna entre las 4 (independientes: Python / TS / Rust CLI / ADR).

## Notas
- Git sucio con cambios de sesión(es) previa(s) (HEAD `972c13a7` + otros); los sub-agentes commitean SOLO sus archivos de la tarea.
- Ponytail full: alcance mínimo que cumple el contrato; no perseguir 100% de coverage si el salto de gate no lo exige.

=== RECITATION ===
Campaign ID: f0dc0d31-7c45-4438-8708-71b796932849
Objetivo activo: añadir smoke test async que ejercite los métodos faltantes de AsyncVantaDB y pase pytest
Estado: completed
Última acción: creado vantadb-python/tests/test_async_smoke.py (3 tests: crud/flush/purge, query/graph/delete, export) y verificado con pytest → 3 passed
Resultado: OK
Próxima acción: ninguno (tarea completa; vanta-lead commitea el archivo nuevo)
Contrato: {"verificacion": "target/audit-venv/Scripts/python.exe -m pytest vantadb-python/tests/test_async_smoke.py -q → 3 passed in 1.41s", "evidencia": [{"claim": "AsyncVantaDB smoke test pasa ejercitando flush/purge_expired/query/graph_*/put/delete/export_*", "evidencia": "pytest run: 3 passed in 1.41s", "confianza": "alta"}, {"claim": "TestAsyncVantaDB ya commiteado en test_sdk.py pasa 16 tests", "evidencia": "pytest run previo: 16 passed", "confianza": "alta"}], "artefactos": ["vantadb-python/tests/test_async_smoke.py"], "invariantes": "API pública AsyncVantaDB intacta; no se tocó __init__.py ni test_sdk.py", "deuda": "ninguna", "queda_pendiente": "vanta-lead commitea el nuevo archivo tras verificar"}
Próxima tarea si completa: 
=== END RECITATION ===

## Retrospectiva (al archivar)

- **Start:** delegación por waves (3 concurrentes + 1 ADR) con sub-agentes vanta-worker/vanta-arch; contratos acotados por ponytail.
- **Stop:** no perseguir 100% de coverage (COV-003 ~76.5% de handlers alcanza el salto de gate; COV-002 usa c8 fallback en vez de parchear vitest).
- **Continue:** medir coverage de bindings con sus runners nativos (ADR-018 lo fija como política).
- **Acción medida:** 4 tareas cerradas en 2 waves, 0 regresiones, 0 falsos positivos (baseline RULES: >90% primer intento). Verificación mecánica por sub-agente + pre-commit hook verde en COV-003 (fmt+clippy).
- **Nota:** COV-004 es DOC-ONLY; la implementación del gate (scopar `-p vantadb` en `ci-rust-10.yml`) queda como tarea de CI aparte.
