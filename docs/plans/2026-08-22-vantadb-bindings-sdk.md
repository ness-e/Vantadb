# Plan de Ejecución: VantaDB Bindings SDK — sub-clientes por dominio

> **Inicio:** 2026-08-22
> **Estado:** ✅ COMPLETADO (4/4 tareas — Bindings SDK)
> **Fuente:** meta-tarea MEM-36 (`.opencode/skills/campaign-executor/tasks/MEM-36.md`, spec completa) + decisión usuario 2026-08-21 (campaña separada)
> **Predecesores:** P27+P29+P30 (roadmap TDAM F1-F7 ✅ 42 tareas) · P31 cierre final (en curso)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 4 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

**Objetivo:** exponer **sub-clientes por dominio** (`db.memory.*`, `db.graph.*`, `db.conversation.*`, `db.wiki.*`) en `vantadb-ts` y `vantadb-python`, manteniendo **100% backward-compat** con la API plana.

## Decisiones fijadas (no re-debatir en DISCOVERY)

- **D42:** sub-clientes implementados **SOLO en capa TS/Python** como azúcar organizativa sobre los métodos planos existentes — **CERO cambios en WASM/Rust**. La fricción wasm-pack (VS-CORE-05) desaparece de esta campaña.
- **D43:** alcance v1 = agrupar métodos YA expuestos. Las capacidades del crate `vanta-memory` (pipeline L0-L3, context engine) NO están expuestas vía bindings hoy — exponerlas exige nuevo binding Rust y queda **deferido** con tarea documentada al final de esta campaña.
- **D44:** orden TS primero (superficie mayor), Python después. Tests de comportamiento espejados entre ambos.
- **D45:** versionado: minor bump al mergear (aditivo puro).
- **Principios:** backward-compat 100% verificada por suite existente · sin deps nuevas · errores tipados existentes.

## Superficie verificada (Paso 0, 2026-08-22)

| SDK | Estado actual |
|---|---|
| `vantadb-wasm/src/lib.rs` | **43 métodos públicos planos** (put, put_batch, get, delete, delete_by_filter, list, list_namespaces, search family, supersede, versions*, graph_bfs/dfs/topological/degree/is_dag, insert_node/delete_node/add_edge, query, import/export, metrics, capabilities...) |
| `vantadb-ts/src/vantadb.ts` | 1 clase plana que wrappea el pkg WASM (+ native.ts para Node) |
| `vantadb-python/src/lib.rs` | ~26 métodos planos (+ types.rs con getters vector/numpy PERF-31) |

*Nota: algunos métodos wasm (graph_*) pueden no estar re-expuestos aún en TS/Python — el mapa exacto método↔dominio se committea en SDKB-01.*

---

## Tasks

### Task 1: SDKB-01 — Mapa namespace ↔ método + diseño de sub-clientes
- **Appetite:** max ½d
- **Esfuerzo:** 🟢 | **Prioridad:** 🟠
- **Archivos clave:** `docs/api/BINDINGS_NAMESPACES.md` (crear — tabla canon método→dominio por SDK)
- **Verificación real:** ✅ CÓDIGO-REAL — superficie listada arriba (43 wasm / TS clase / ~26 python)
- **Gate Justificación:** fundación — el mapa es el contrato que SDKB-02/03 ejecutan
- **Gate Result:** ✅ DO
- **Contrato: verificacion: npm test (246 passed) ✅ · pytest vantadb-python/tests -q (105 passed, 4 skipped) ✅ · pwsh scripts/validate-docs-coverage.ps1 (0 gaps) ✅ | evidencia: [claim TS suite verde 246/246 → npm test exit 0, Test Files 7 passed — alta] [claim Python suite verde 105+4skip → pytest in 74.59s exit 0 — alta] [claim coverage 0 gaps → validator exit 0, vantadb-python 43 items ok — alta] [claim docs = superficie real → codegraph verbatim vantadb.ts:247-343 + lib.rs forward_to_db! memory15/graph10/system17/wiki1 — alta] | artefactos: vantadb-ts/README.md, docs/api/PYTHON_SDK.md, docs/api/BINDINGS_NAMESPACES.md, tasks/SDKB-04.md, scripts/validate-docs-coverage.ps1 ($pyInternals +4 getters PyO3 *_client renombrados via #[pyo3(name=...)] — no API Python visible) | invariantes: flat API sin cambios (suites intactas), cero cambios WASM/Rust (D42), no commitear ni tocar plan file / docs/reviews/* | deuda: ninguna | queda_pendiente: orquestador decide commit de los 5 archivos; plan Task 4 marcar ✅ COMPLETED
- **Pre-mortem:** métodos huérfanos (capabilities, import/export, metrics) → dominio `system`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟢×🟢 | método sin dominio claro | dominio system catch-all | diseño |
- **Cynefin:** 🟦 obvio
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/SDKB-01.md`
- **Notas:** Ruta: vanta-worker.

### Task 2: SDKB-02 — Sub-clientes TypeScript
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴
- **Archivos clave:** `vantadb-ts/src/vantadb.ts` (editar — getters por dominio), `vantadb-ts/src/__tests__/` (extender)
- **Verificación real:** ✅ CÓDIGO-REAL — clase plana existe (native.ts:174 patrón put); D42: cero cambios WASM
- **Gate Result:** ✅ DO
- **Contrato:** "`npm test` pasa (suite existente intacta = backward-compat); tests nuevos: `db.memory.put/get/search/supersede`, `db.graph.bfs/topological`, `db.wiki.*` según mapa SDKB-01 delegan al método plano idéntico (mismo resultado, misma firma)"
- **Pre-mortem:** (1) `this` binding roto en getters → arrow/bind explícito; (2) tipos duplicados → reusar types.ts
- **Stop conditions:** si un sub-cliente requiere lógica nueva (no solo delegación) → fuera de v1, documentar
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | this-binding roto en getters | bind explícito + test por sub-cliente | primer test |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/SDKB-02.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 1.

### Task 3: SDKB-03 — Sub-clientes Python
- **Appetite:** max 1d
- **Esfuerzo:** 🟡 | **Prioridad:** 🟠
- **Archivos clave:** `vantadb-python/src/lib.rs` (editar — propiedades/py-methods por dominio o módulo helper), `tests/` (extender)
- **Verificación real:** ✅ CÓDIGO-REAL — ~26 métodos planos verificados
- **Gate Result:** ✅ DO
- **Contrato:** "`pytest` pasa (suite existente intacta); tests nuevos espejo de SDKB-02: `db.memory.*`, `db.graph.*` delegan al método plano idéntico"
- **Pre-mortem:** PyO3 no soporta propiedades anidadas triviales → sub-cliente como objeto simple construido en `__init__` o funciones de módulo helper — elegir lo más simple (ponytail)
- **Stop conditions:** si PyO3 complica → exponer helper functions `vanta_memory_client(db)` en vez de atributos
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger/Due |
  |---|---|---|---|
  | 🟡×🟡 | PyO3 nested objects fricción | helper function en vez de property | DISCOVERY |
- **Cynefin:** 🟦 obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 3 steps
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/SDKB-03.md`
- **Notas:** Ruta: vanta-worker. DEPENDE de Task 1. Leer `.opencode/rules/python-bindings.md` antes.

### Task 4: SDKB-04 — Docs + gate backward-compat final
- **Appetite:** max ½d
- **Espec:** README de vantadb-ts + PYTHON_SDK.md secciones de sub-clientes con ejemplos; suite COMPLETA de ambos SDKs verde (backward-compat proof)
- **Gate Result:** ✅ DO
- **Contrato:** "`npm test` + `pytest` completos exit 0; READMEs actualizados; validate-docs-coverage 0 gaps si aplica"
- **Cynefin:** 🟦 obvio
- **Estado:** ✅ COMPLETED
- **Task file:** `.opencode/skills/campaign-executor/tasks/SDKB-04.md`
- **Notas:** Ruta: vanta-docs.

---

## DEFERIDO desde esta campaña

| Item | Motivo |
|---|---|
| Exponer pipeline vanta-memory vía WASM/bindings | requiere nuevo binding Rust (L0-L3 no está en wasm hoy) — campaña propia tras validar demanda |
| Regeneración wasm-pack | innecesaria con D42 (cero cambios WASM) |

---

=== RECITATION ===
Campaign ID: b0b20628-356b-4e65-9b02-178341fd30f7
Objetivo activo: SDKB-04: docs sub-clientes TS+Python + gate backward-compat final
Estado: pending ⏳
Última acción: SDKB-04 completo: sección Domain Sub-clients en vantadb-ts/README.md y PYTHON_SDK.md con ejemplos db.memory/graph/wiki/system + garantía flat-API-unchanged; cross-refs desde BINDINGS_NAMESPACES.md; suites verdes (npm 246/246, pytest 105+4skip); validate-docs-coverage 0 gaps. Sin commit por instrucción.
Resultado: OK
Próxima acción: Ninguna — campaña P32 completa (4/4). Orquestador decide commit.
Contrato: por tarea — suites existentes intactas (backward-compat) + tests nuevos de delegación
Próxima tarea si completa: ninguna (última tarea del plan)
=== END RECITATION ===
