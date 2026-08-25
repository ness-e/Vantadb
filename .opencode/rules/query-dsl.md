# Query DSL (IQL) — Reglas

> **Scope:** `src/parser/` (DSL IQL estilo Lisp vía nom), `planner.rs`, `executor.rs`, `physical_plan.rs`, `query.rs`, búsqueda híbrida en `sdk/search/`
> **No tocar aquí:** API pública/serialización (`api-contract.md`), índices (`indexes.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** DRV-002, INV-009

## Reglas

### R-1: Frases entre comillas = adyacencia estricta (slop=0)

- **Must:** interpretar `"multi word"` como frase con tokens contiguos en orden exacto (slop=0), usando `phrase::text_positions_match_phrases` sobre `TextQueryPlan.phrases`.
- **Must not:** relajar la adyacencia de frases (proximity/slop>0) en el matching default; si se quiere proximidad relajada, extender `phrase.rs` con slop — nunca tocar storage.
- **Por qué:** INV-009 verificó que positions por documento + matching de orden/adyacencia ya están implementados y son el default correcto; equivale al default de `tantivy::query::PhraseQuery`.

### R-2: Tokenización de frases literal — sin stopwords ni stemming

- **Must:** construir los tokens de las frases de forma literal (sin stopword removal ni stemming) al ajustar `query_plan_with_config`.
- **Must not:** aplicar el pipeline de stemming/stopwords a las frases entre comillas — distorsiona el matching exacto.
- **Por qué:** la tokenización normal agresiva rompe frases literales (INV-009 §6 paso 3; el parser ya resuelve `Condition::TextMatch` en `src/parser/mod.rs` — mantener esta semántica literal al evolucionarlo).

### R-3: NO agregar tantivy — storage custom ya cubre el alcance

- **Must not:** agregar `tantivy` (o ~40 crates transitivas) al workspace para phrase matching.
- **Por qué:** `phrase.rs` ya implementa positions + matching con 12 tests sobre el storage existente; tantivy exigiría schema separado, índice duplicado y sincronización de escrituras — YAGNI (INV-009 §4).

### R-4: El parser delega frases al planner — no duplica gramática

- **Must:** extraer las frases de comillas en `parse_condition` y delegar a `query_plan`/`query_plan_with_config` (que ya produce `TextQueryPlan.phrases`) — implementado; toda condición nueva de texto sigue este camino.
- **Must not:** crear una segunda gramática de frases en el parser que compita con la del planner.
- **Por qué:** el parser nom ya resuelve `string_literal`; duplicar la extracción de frases crearía drift entre parse y ejecución (INV-009 §6 paso 1).

<!-- Referencias cruzadas: → ver core-engine.md, api-contract.md, indexes.md -->
