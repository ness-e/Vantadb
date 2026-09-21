# Plan de Ejecución: Python Subclients Paridad TS (cierre excepción `_memory`)

> **Campaign ID:** af650ef2-83ac-46e5-9842-33a7da271b36
> **Inicio:** 2026-09-11
> **Estado:** ⏳ LISTO PARA RUN
> **Fuente:** auditoría 7 remanentes (ítem 2, hazard flat) + decisión usuario GO + `test_subclients.py` SDKB-03 existente
> **Autonomous:** false

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 1 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 0 · ⬇️ downhill = 1
SDP: `code-review-and-quality, code-simplification, deprecation-and-migration, git-workflow-and-versioning, test-driven-development, documentation-and-adrs`
Regla: rename DIRECTO (sin usuarios). Paridad TS: `db.memory.get/list/delete/search`, `db.graph.*` limpios; plano de nodos intacto.

## Tasks

### Task 1: AST-012 — Subclientes Python sin apellido + plano limpio

- **Appetite:** max 1d
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/tests/test_subclients.py`, `vantadb-python/vantadb_py/vantadb_py.pyi`, `docs/api/PYTHON_SDK.md`, `docs/api/BINDINGS_NAMESPACES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — subclientes `db.memory/graph/system/wiki` existen (SDKB-03); `db.memory.get_memory` repite contenedor; plano tiene `get/delete` nodo-u128 + `get_memory` memoria (hazard); TS `MemoryClient.put/get/delete/search` es el modelo.
- **Gate Justificación:** última excepción viva a la regla; sin usuarios, directo.
- **Gate Result:** ✅ DO
- **Contrato:** `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_subclients.py -q` verde (o runner equivalente con evidencia) + `rg -n "get_memory|list_memory|delete_memory|search_memory" vantadb-python/vantadb_py/__init__.py` → `0`
- **Task file:** `docs/tasks/AST-012.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** 29b64ef8 (refactor!: AST-012, 14 files, sin push)

  **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|------------|---------------|
  | 🟡×🔴 | se rompe paridad plano/subcliente | plano delega a subcliente o viceversa, 1 sola implementación | test rojo |
  | 🟢×🟡 | docs con nombres viejos | actualizar PYTHON_SDK + BINDINGS en mismo PR (Regla 3) | rg hits |

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | 1 | Discovery + task file canónico (blast radius, Spec 5/5, Regla 0) | 0 uphill / 9 steps | codegraph, trace_path, rg, git log |
  | 2 | Step 1 RED: test_subclients a cortos + guard eliminación | 21 pass + 1 fail-guard (razón correcta) | pytest |
  | 3 | Step 2-3 GREEN nativo: macro 3er brazo `with { }` (E0119: PyO3 rechaza 2 bloques #[pymethods]), cuerpos get/list/delete movidos, flat `*_memory` fuera, docstrings | cargo check + fmt + clippy verdes | cargo, edit |
  | 4 | Rebuild `maturin develop` (desde vantadb-python/) + sonda hasattr | flat-stutter [] + roundtrip OK | maturin, python |
  | 5 | Steps 4-6: migración tests (sdk 46, subclients, async_smoke, migration, close_concurrency, perf) + verify_wheel | test_sdk 75 passed | pytest |
  | 6 | Step 5: AsyncMemoryClient (get/list/delete) + property memory; `__init__.pyi` espejo | rg→0 en `__init__.py` (contrato) | edit, rg |
  | 7 | Step 7: `vantadb_py.pyi` (Client −3, MemoryClient tipadas reales) | stub_drift 7 passed | pytest |
  | 8 | Step 8 docs Regla 3: PYTHON_SDK + BINDINGS + search_vector saldado | coverage 0 gaps (pwsh7) | edit, validate-docs-coverage |
  | 9 | Step 9: review vanta-review (changes-required condicional → atendido R1/R2/nits) + commit 29b64ef8 | ✅ COMPLETO | task, git |

  **Notas:**
  - **Pre-mortem:** 1) `db.memory.get` choca con algo interno del subcliente; 2) plano y subcliente divergen; 3) `.pyi` drift.
  - **Stop conditions:** colisión real dentro de subcliente → Gate P antes de inventar nombres.
  - **Cynefin:** 🟨 complicado.
  - **Top 3 riesgos:** divergencia plano/subcliente; pyi drift; docs.
  - **Uphill/Downhill:** ⬆️ 0 / ⬇️ renames + tests + docs.
  - **DoD task:** contrato + `test_sdk.py` verde + stub drift verde.

=== RECITATION AST-012 ===
Campaign ID: af650ef2-83ac-46e5-9842-33a7da271b36
Objetivo activo: AST-012 — Subclientes Python sin apellido + plano limpio
Estado: completed
Última acción: 9/9 steps ✅ + review vanta-review atendido (R1/R2/nits) + commit c682c008 (14 files, sin push) + plan/task files sincronizados
Resultado: ✅
Próxima acción: ninguna — lead: push + nextest workspace/CI; deuda Gate C a Backlog lead
Contrato: verificacion: python -m pytest vantadb-python/tests/test_subclients.py -q (22 passed) + rg stuttered vantadb-python/vantadb_py/__init__.py → 0 ✅; evidencia: [{claim: subclients cortos + plano sin *_memory, evidencia: sonda hasattr + commit c682c008, confianza: alta}, {claim: sin regresiones, evidencia: test_sdk 75 passed + stub_drift 7 passed + fmt/clippy exit 0 + review vanta-review, confianza: alta}]; artefactos: [commit c682c008 (14 files), docs/tasks/AST-012.md]; invariantes: flat get/delete nodo-u128 intactos, shared-names misma firma, sin aliases; deuda: integrations/docs viejos → lead (precedente AST-008); queda_pendiente: push + nextest workspace via lead/CI
Próxima tarea si completa: ninguna (cierra excepción)
=== END RECITATION ===

=== RECITATION 1 ===
Campaign ID: af650ef2-83ac-46e5-9842-33a7da271b36
Objetivo activo: AST-012 subclientes sin apellido
Estado: completed
Última acción: RESUME OK COMPLETO 9/9 commit c682c008 tras fallo de modelo
Resultado: ✅
Próxima acción: campaña 1/1 completa
Contrato: subclients 22 passed + rg *_memory 0
Próxima tarea si completa: ninguna
=== END RECITATION ===
