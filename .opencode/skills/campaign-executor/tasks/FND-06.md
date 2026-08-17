# FND-06: Boundaries core↔bindings — lógica de negocio no filtrada a capas de interfaz

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ✅ COMPLETED (2026-08-16 — todos los steps; verificación cargo check ✅)

## Blast Radius
- **Callees:** core search SDK (`src/sdk/search/mod.rs`, `src/sdk/api.rs`), distancia (`src/index/distance/`), `VantaEmbedded` (bindings)
- **Callers:** `vantadb-ts/src/vantadb.ts` (WASM), `vantadb-ts/src/native.ts` (node/native), `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`, `integrations/{llamaindex,langchain}/.../vectorstore.py`
- **Tests que cubren el comportamiento:** `vantadb-ts/src/__tests__/hardening.test.ts:204` (zero-norm search), `integration.test.ts`, `src/index/search/tests.rs:281-328` (zero-norm cosine rejected), `integrations/langchain/tests/test_vectorstore.py:252-256` (score fn)
- **Implicaciones:** cambio de comentarios + regla normativa + reporte; NO toca comportamiento de bindings ni API pública. Cero riesgo de regresión.

## Impacto mapeado (Regla 0)
- **Archivos que se modifican:**
  - `.opencode/rules/api-contract.md` — nueva sección R-8 (regla must). Referencias entrantes: solo lazy-load desde AGENTS.md (tabla de reglas); añadir sección no rompe nada.
  - `vantadb-ts/src/vantadb.ts:333-353` — solo TODO comment sobre lógica de negocio (zero-norm fallback).
  - `vantadb-ts/src/native.ts:250-260` — solo TODO comment (drift vs vantadb.ts).
  - `integrations/llamaindex/vantadb_llamaindex/vectorstore.py` — solo TODO comments (cosine sim + score mapping + RRF).
  - `integrations/langchain/vantadb_langchain/vectorstore.py` — solo TODO comments.
  - `docs/Investigaciones/FND-06-core-bindings-boundaries.md` — reporte NUEVO (sin referencias previas).
  - `.opencode/skills/campaign-executor/tasks/FND-06.md` — task file NUEVO.
- **Referencias entrantes:** api-contract.md es lazy-loaded (no importado); los TS/PY solo ganan comments. Ninguna referencia entrante se rompe.
- **Referencias salientes:** los TODO apuntan a `src/sdk/search/mod.rs` (ERR-028) y `src/index/distance/` — sin cambios en core.
- **Impacto si se elimina:** N/A — no se elimina nada. Veredicto: cambio seguro, cero comportamiento alterado.
- **Dominio:** api-contract (reglas api-contract.md), bindings TS/PY — ninguna regla bloquea comments/regla normativa.
- **Git history:** working tree sin WIP propio (verificar con git status antes de cerrar).

## Contrato
- Regla must R-8 en `.opencode/rules/api-contract.md` (lógica de negocio = core; bindings = glue + memoria; excepciones documentadas)
- Reporte `docs/Investigaciones/FND-06-core-bindings-boundaries.md` con hallazgos clasificados (archivo:línea)
- `cargo check -p vantadb -p vantadb-wasm -p vantadb-python` pasa (o `-p vantadb -p vantadb-wasm` si pyo3 no disponible)

## Herramientas
- codegraph, grep, cargo

## Steps
### Step 1: DISCOVERY — mapear edges core↔bindings
- **Archivos:** codegraph_explore (edges), grep validación/distancia/dedup en bindings
- **Acción:** clasificar hallazgos: business-logic-duplicated / glue-legítimo / boundary-violation
- **Verify:** 3 hallazgos principales con archivo:línea (H1 zero-norm TS, H2 adapters cosine/score-mapping, + glue-legítimo confirmado)
- **Estado:** ✅ COMPLETED

### Step 2: Regla must R-8 en api-contract.md
- **Archivos:** `.opencode/rules/api-contract.md`
- **Acción:** añadir sección "R-8: Lógica de negocio en el core, bindings = glue + memoria" (must/must-not/por-qué + excepciones documentadas)
- **Verify:** sección presente, formato consistente con R-1..R-7
- **Estado:** ✅ COMPLETED

### Step 3: TODO comments en hallazgos seguros
- **Archivos:** `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`, `integrations/llamaindex/.../vectorstore.py`, `integrations/langchain/.../vectorstore.py`
- **Acción:** marcar cada hallazgo con TODO al core (ERR-028 / distance module); sin cambiar comportamiento
- **Verify:** grep TODO FND-06 → 7 hits (2 TS + 5 Python)
- **Estado:** ✅ COMPLETED

### Step 4: Reporte FND-06-core-bindings-boundaries.md
- **Archivos:** `docs/Investigaciones/FND-06-core-bindings-boundaries.md`
- **Acción:** reporte con hallazgos clasificados (archivo:línea), veredicto por hallazgo, fixes aplicados vs diferidos
- **Verify:** estructura del reporte completa
- **Estado:** ✅ COMPLETED

### Step 5: Verificación
- **Acción:** `cargo check -p vantadb -p vantadb-wasm` ✅ + `cargo check -p vantadb_py` ✅ (package real de vantadb-python; pyo3 disponible)
- **Verify:** exit 0 ambos
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 5, parallel-safe: toca solo api-contract.md + bindings TS/PY + reporte nuevo)