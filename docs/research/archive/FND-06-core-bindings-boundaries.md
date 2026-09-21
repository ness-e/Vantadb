# FND-06: Boundaries core↔bindings — lógica de negocio no filtrada a capas de interfaz

**Estado:** ✅ Resuelto (regla R-8 + reporte + hallazgos señalados; fixes estructurales diferidos a spec)
**Fecha:** 2026-08-16
**Prioridad:** 🔴 (P20a)
**Fuente:** docs/Backlog.md:488
**Alcance:** edges `vantadb-server/`, `vantadb-python/`, `vantadb-wasm/`, `vantadb-ts/`, `vantadb-node/`, `integrations/` → internos del core (`src/`)
**Archivos tocados:** `.opencode/rules/api-contract.md` (R-8), `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`, `integrations/llamaindex/vantadb_llamaindex/vectorstore.py`, `integrations/langchain/vantadb_langchain/vectorstore.py`, este reporte

---

## 1. Metodología

1. `codegraph_explore` sobre edges binding→core (llamadas, instanciaciones, referencias).
2. `grep` dirigido por patrón en cada familia de binding: validaciones re-implementadas, cálculos de distancia/similitud, decisiones de búsqueda (métrica, fusion, top-k, dedup), workarounds de errores del core.
3. Lectura de los guards del core (`src/sdk/search/mod.rs`, `src/sdk/api.rs`) para comparar el contrato real del core contra el comportamiento de cada binding.

## 2. Clasificación

| Clase | Definición | Hallazgos |
|---|---|---|
| 🔴 **business-logic-duplicated** | Cálculo/decisión de negocio reimplementado en la capa de interfaz | H1, H2, H3 |
| 🟡 **boundary-violation** | Binding workaroundea o suprime un contrato deliberado del core | H1 (suprime ERR-028) |
| ✅ **glue-legítimo** | Serialización, mapping, errores, GIL/async, guards FFI | G1–G5 |

## 3. Hallazgos

### 🔴 H1 — Decisión de búsqueda en binding: fallback silencioso de métrica (zero-norm) + drift cross-binding

- **Sitio:** `vantadb-ts/src/vantadb.ts:333-353` — `_buildSearchRequest` detecta query vector zero-norm (`every(v => v === 0)`) y **cambia silenciosamente la métrica a Euclidean**.
- **Contrato del core:** `src/sdk/search/mod.rs:106-120` (ERR-028) **rechaza** zero-norm bajo cosine con `InvalidInput` — decisión deliberada desde AUDREP-55 (`src/index/search/tests.rs:281-328`). El error sugiere explícitamente "use a non-zero vector **or the euclidean distance metric**"; el binding WASM automatizó esa sugerencia.
- **Drift real:** `vantadb-ts/src/native.ts:250-260` **no** tiene el fallback → mismo input (`query_vector: [0,0,0]`, metric default cosine): WASM devuelve matches, native devuelve error del core. Python (`vantadb-python/src/lib.rs:954-988`) tampoco workaroundea → error (consistente con core). Tests que dependen del workaround: `vantadb-ts/src/__tests__/hardening.test.ts:204`.
- **Clasificación:** 🔴 business-logic-duplicated + 🟡 boundary-violation (suprime un error deliberado del core en una plataforma, creando semántica divergente).
- **Fix aplicado:** TODO(core) en ambos bindings TS documentando la decisión + el drift (R-8).
- **Fix diferido (requiere spec):** mover la decisión al core (zero-norm cosine → error, o fallback automático a Euclidean). Cambia comportamiento público → spec + migración de tests de core.

### 🟡 H2 — Cálculo de distancia coseno reimplementado en adapters

- **Sitios:** `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:280-286` (`_cosine_sim`), `integrations/langchain/vantadb_langchain/vectorstore.py:241-248` (`_cosine_sim`).
- **Core:** `src/index/distance/` (metrics.rs, mapper.rs) — única fuente de verdad de distancia.
- **Uso:** solo para diversidad MMR entre candidatos ya fetcheados (modo opt-in del framework).
- **Clasificación:** 🔴 business-logic-duplicated (cálculo duplicado), con matiz: el algoritmo MMR es feature del adapter (excepción R-8).
- **Fix aplicado:** TODO(core) en ambos.
- **Fix diferido:** exponer helper de similitud desde el core (API nueva → spec) o aceptar como glue documentado.

### 🟡 H3 — Mapping score→similitud y fusión/dedup client-side duplicados

- **Sitios:** `1.0 - hit.score / 2.0` en `integrations/llamaindex/.../vectorstore.py:183,188,237,340` y `integrations/langchain/.../vectorstore.py:213` (+`_cosine_relevance_score_fn` en `:563-566`, test `:252-256`); RRF fusion + dedup en `integrations/llamaindex/.../vectorstore.py:148-204` (`_hybrid_search`).
- **Riesgo:** asume semántica de score del core (cosine distance ∈ [0,2]) hardcodeada en 2 adapters; si el core cambia la escala, los adapters rompen silenciosamente.
- **Nota:** el core ya tiene hybrid fusion server-side (planner `hybrid_candidate_budget`); el adapter lo reimplementa solo en modo opt-in de framework.
- **Clasificación:** 🔴 business-logic-duplicated (fórmula de negocio duplicada sin fuente única).
- **Fix aplicado:** TODO(core) en ambas ocurrencias representativas.
- **Fix diferido:** documentar semántica oficial de scores en `docs/api/` + helper de conversión en core (spec).

## 4. Glue legítimo verificado (✅ — sin acción)

| ID | Sitio | Por qué es glue |
|---|---|---|
| G1 | `vantadb-python/vantadb_py/__init__.py` (AsyncVantaDB, SearchRequest) | Wrappers asyncio puros + dataclass espejo de kwargs; cero lógica de negocio |
| G2 | `vantadb-server/src/server.rs:1-4` | Re-export puro de `vantadb::cli_server`; `main.rs` parseo de flags CLI |
| G3 | `vantadb-wasm/src/lib.rs` (search, search_vector, search_hit_to_js) | Serialización JS, guards de longitud FFI (MAX_F32_VEC_LEN/MAX_BATCH_SIZE), clamp `top_k.min(MAX_K)`, mapping string→enum |
| G4 | `vantadb-python/src/lib.rs` (open_vantadb, map_vanta_error, py.detach) | Construcción de config, traducción de errores, release de GIL |
| G5 | `vantadb-node/src/lib.rs:380-389` | Parseo de top_k + mapping metric string→enum |

**Nota menor (DRY, no fix):** el clamp `MAX_K = 1_000` está duplicado como constante en `vantadb-python/src/lib.rs:43`, `vantadb-wasm/src/lib.rs:43` (y análogo en node/TS). Es guard FFI legítimo; el riesgo es drift si cambia el límite. No se unifica sin spec (afecta contrato de input de bindings).

## 5. Regla normativa (R-8, `.opencode/rules/api-contract.md`)

Añadida la sección **R-8: Lógica de negocio en el core — bindings son glue + memoria**: must (negocio en `src/`, bindings = glue), must-not (reimplementar distancias, decisiones de búsqueda, workaround de errores deliberados, fórmulas duplicadas), por-qué (evidencia H1–H3), excepciones documentadas (features de framework en adapters con condiciones), y criterio de fix (mover al core solo si no cambia API; si no → TODO + reporte).

## 6. Verificación

- `cargo check -p vantadb -p vantadb-wasm -p vantadb-python` → ✅ (ver RESULTADO de la tarea; pyo3 dependiente del toolchain)
- Cambios: comments + regla + reporte — cero comportamiento alterado (sin refactor de bindings; sin cambios en core).

## 7. Deuda diferida (requiere spec antes de mover lógica al core)

1. **H1:** decidir en core si zero-norm cosine → error (hoy) o fallback automático; alinear `vantadb.ts` y `native.ts`.
2. **H3:** documentar semántica oficial de scores en `docs/api/` y evaluar helper de score→similarity en core.
3. **DRY:** unificar límite `MAX_K` si se toca el contrato de input de bindings.