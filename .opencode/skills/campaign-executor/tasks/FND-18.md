# FND-18: Time-to-first-query <5 min en SDKs Python/TS (Fase 0 pre-launch)

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-r2-r7-fnd.md (Task 7)
- **Fuente:** docs/Backlog.md P20d, prio 🔴, esfuerzo 🟡
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Docs (quickstart) + medición
- **Turns estimados:** 15
- **Creado:** 2026-08-16T08:50
- **last-synced:** 2026-08-16T08:50
- **Estado:** ⏳ IN PROGRESS (execución completa; lead commitea al cerrar wave)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `README.md` (raíz) → `docs/QUICKSTART.md`; `vantadb-python/pyproject.toml` readme = `vantadb-python/README.md` (visible en PyPI); `vantadb-ts/package.json` files incluye `vantadb-ts/README.md` (visible en npm) |
| Callees | API `vantadb_py` (`VantaDB.put`, `search_memory`, `VantaSearchHit`), `vantadb` TS (`VantaDB.put`, `search`, `VantaValue`) |
| Implicaciones | Contrato de docs: los ejemplos de quickstart deben ser ejecutables contra la API publicada 0.5.0. No cambia API pública, no afecta performance, no requiere migración |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-python/README.md` (87L), `vantadb-ts/README.md` (158L), `docs/QUICKSTART.md` (203L), `README.md` (403L), `vantadb-python/Cargo.toml`, `vantadb-python/pyproject.toml`, `vantadb-ts/package.json`, `vantadb-ts/src/vantadb.ts` (938L), `vantadb-ts/src/native.ts`, `vantadb-ts/src/types.ts`, `vantadb-wasm/pkg/package.json`, `vantadb-python/src/lib.rs` (put/search_memory, 700-820)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vantadb-python/README.md` y `vantadb-ts/README.md` no tienen imports; son auto-contenidos
- **Archivos que referencian a los editados (referencias entrantes):**
  - `README.md` → enlaza `docs/QUICKSTART.md` ("5-Minute Quickstart")
  - `vantadb-python/pyproject.toml:9` → `readme = "README.md"` (la página PyPI muestra el README del SDK)
  - `vantadb-ts/package.json:18-22` → `files: ["dist/", "README.md", "LICENSE"]` (npm muestra el README)
  - `docs/Backlog.md` menciona P20d (NO se toca — lead migra)
- **Veredicto impacto:** bajo — cambios solo en docs de quickstart; ninguna referencia a código; la API medida funciona con el shape correcto

## Contrato
"`git diff` sobre los 3-4 archivos docs; quickstart de ambos SDKs ejecutado localmente con métrica time-to-first-query <5 min registrada (python: install 5.52s + query 0.67s; ts: install 1.32s + query 0.30s); ningún archivo de código tocado"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** NO tocar código de SDKs (vantadb-python/src, vantadb-ts/src, vantadb-wasm/src); NO tocar `docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/AUD-024.md`, `vantadb-wasm/src/lib.rs`, `docs/plans/2026-08-16-wave-r2-r7-fnd.md`; NO git commit (lead commitea)
- **Comandos de verificación:** `python -m pip install vantadb-py && python quickstart_memory.py` (total <5 min); `npm install vantadb && node quickstart.mjs` (total <5 min)
- **Deuda pendiente:** gaps de API documentados sin implementar (docstrings `vantadb.ts:207,703` usan shape roto de metadata → FND-05; README python en español no se traduce — fuera de scope)

## Recitation (canónico — estructura única)
| Campo | Fuente |
|-------|--------|
| activeGoal | FND-18: Time-to-first-query <5 min en SDKs Python/TS |
| lastAction | Completar/verificar los 3 fixes de docs (worktree del intento abortado) + smoke test local `hit.key`/`hit.score` |
| result | OK — Steps 1-3 ✅; 0 patrones rotos residuales; ningún src/ tocado; smoke test py: 1 hit, `hit.key=fact_001` |
| nextAction | Lead commitea diff docs al cerrar wave; FND-18 listo para review |
| contract | verificacion: quickstart python y ts ejecutados con métrica <5 min; evidencia: mediciones locales (ver Notas); artefactos: diffs docs (git diff 3 docs); invariantes: no tocar código; deuda: gaps documentados para FND-05 |
| nextTask | FND-17 (siguiente en plan file Wave 2) |

## Deuda técnica (Regla 6 — MUST)
**Sin deuda** — los cambios son correcciones de docs (reducen deuda de docs rotas)

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| Task | Contrato pasa: quickstart ambos SDKs <5 min medido; docs corregidas; gaps documentados |
| Commit | Lead commitea (worker no commitea) |
| Release | No aplica (docs pre-launch; lead decide) |

## Herramientas necesarias
- Terminal (pip/npm/node/python), codegraph (ya usado)

## Investigation Notes
- Medición local 2026-08-16 (Windows, Python 3.11.9, Node 24.16.0, npm 11.6.0):
  - Python: `pip install vantadb-py` → 5.52s; script quickstart (open+put+search) → 0.67s; **total 6.2s**
  - TS: `npm install vantadb` → 1.32s; script quickstart (put+search) → 0.30s; **total 1.6s**
- Fricción mayor identificada: **docs rotas, no tiempo de instalación**:
  1. `vantadb-ts/README.md` y docstrings `vantadb.ts` usan `metadata: { lang: { type: "String", value: "en" } }` pero `VantaValue` es discriminated union `{ String: "en" }` (types.ts:1-10) → quickstart falla con `invalid length 2, expected 1`
  2. `vantadb-python/README.md` recomienda TestPyPI como install primario cuando PyPI real tiene 0.5.0; ejemplos usan `hit['key']` pero `VantaSearchHit` no es subscriptable (usa `.key`/`.score`)
  3. `docs/QUICKSTART.md` dice "Production PyPI is not yet available" (desactualizado) y `hit["record"]["key"]` falla; path primario es source build (>>5 min)
- Gaps de API (NO implementar — FND-05): docstrings `vantadb-ts/src/vantadb.ts:207,703` muestran shape roto de metadata; `search_memory(query_vector=[], text_query=...)` funciona (text-only ok)

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 0 (3/3 docs editados y verificados) |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [ ] **SECURITY** — NO aplica: cambios solo en docs markdown, sin trust boundaries, sin deps nuevas
- [ ] **PERFORMANCE** — NO aplica: no toca hot paths; la métrica es el deliverable (docs), el engine ya está medido en BENCHMARKS.md

## Steps

### Step 1: Corregir `vantadb-ts/README.md` (metadata shape del quickstart)
- **Archivos:** `vantadb-ts/README.md`
- **Acción:** reemplazar `metadata: { lang: { type: "String", value: "en" } }` por `metadata: { lang: { String: "en" } }` en Quick Start (línea 45)
- **Verify:** `node quickstart_fixed.mjs` (ya validado: put 0.006s, query 0.004s, 1 hit)
- **Estado:** ✅ COMPLETED (diff verificado en worktree; 0 residuales de shape roto)

### Step 2: Corregir `vantadb-python/README.md` (install PyPI + hit access)
- **Archivos:** `vantadb-python/README.md`
- **Acción:** (a) Instalación: recomendar `pip install vantadb-py` desde PyPI como primario, TestPyPI como alternativa; (b) Quickstart: `hit['key']` → `hit.key`, `hit['score']` → `hit.score`
- **Verify:** `python quickstart_memory.py` con acceso `hit.key` (ya validado; smoke test local repetido: 1 hit, `hit.key=fact_001`)
- **Estado:** ✅ COMPLETED (diff verificado en worktree)

### Step 3: Corregir `docs/QUICKSTART.md` (desactualizado + hit access + path primario + métrica)
- **Archivos:** `docs/QUICKSTART.md`
- **Acción:** (a) fix nota PyPI ("Production PyPI is not yet available" → disponible); (b) `hit["record"]["key"]` → `hit.key`; (c) path primario: wheel/PyPI primero, source como opción dev; (d) añadir sección métrica time-to-first-query medida
- **Verify:** `scripts/validate-docs-coverage.ps1` + leer diff
- **Estado:** ✅ COMPLETED (diff verificado en worktree; sección métrica añadida)

## Dependencias
- Ninguna (tarea independiente en Wave 1)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente (lead decide; cambios docs de bajo riesgo — fallback doubt-driven-development aplicado en discovery al verificar shapes contra código real)
- **Enfoque:** los fixes de docs se verificaron contra la API publicada (pip/npm), no contra memoria
- **Cómo se probó:** medición real local con paquetes publicados 0.5.0
- **Veredicto:** pendiente

## Notas
- El quickstart TS publicado fallaba con metadata roto; el shape correcto es discriminated union (`{ String: "en" }`), confirmado contra `vantadb-ts/src/types.ts:1-10` y guards test
- `VantaMemoryRecord` soporta subscript (`r['key']`) Y atributos (`r.key`); `VantaSearchHit` solo atributos (`hit.key`, `hit.score`) — confirmado en runtime
- `db.put` devuelve `VantaMemoryRecord` (root README lo captura correctamente)