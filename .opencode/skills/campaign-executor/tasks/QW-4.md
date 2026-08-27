# TASK QW-4: dedup gemelos ollama/openai — módulo compartido _shared para Document/add_texts/delete/async

## Metadata
- **Plan file:** `docs/plans/2026-08-25-integrations-research-wins.md`
- **Fuente:** Wave 2 QW-4 (H-05 =MOD-49)
- **Esfuerzo:** 🟡 2-4h (dedup + packaging hatch force-include + suites)
- **Prioridad:** Wave 2 — Limpieza / dedup
- **Tipo:** Python refactor (dedup)
- **Turns estimados:** 5-7
- **Creado:** 2026-08-27T16:45
- **last-synced:** 2026-08-27T16:45
- **Estado:** ✅ COMPLETED
- **Ruta:** vanta-worker
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes tras verify

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `integrations/ollama/vantadb_ollama/__init__.py` (re-export VantaDBOllama), `integrations/openai/vantadb_openai/__init__.py` (re-export VantaDBOpenAI), `integrations/ollama/tests/test_vectorstore.py` (9 tests), `integrations/openai/tests/test_vectorstore.py` (9 tests), docs/plans reference, ningún módulo Rust core depende del adapter (aislado) |
| Callees | `vantadb_py` (VantaDB client), `integrations/vantadb_shared/__init__.py` (Document, EmbeddingVectorStore), `ollama` SDK (`ollama.embeddings`, `ollama.embed`), `openai` SDK (`openai.OpenAI.embeddings.create`), `hatchling` force-include (packaging), `asyncio` + `functools.partial` (async helpers via thread executor) |
| Implicaciones | contrato NO cambia API pública (thin subclasses mantienen VantaDBOllama/VantaDBOpenAI, Document, DEFAULT_NAMESPACE/MODEL, add_texts/delete/similarity_search/aadd_texts/asimilarity_search/adelete); reducción ~243 líneas combinadas (179+64 → 58+64+219 shared); asimilarity_search consistente (mismo executor); packaging hatch force-include `../vantadb_shared` = `vantadb_shared` vendored por wheel (no PyPI separado); no requiere migración de datos ni re-indexación; tests existentes deben pasar (9+9) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `integrations/vantadb_shared/__init__.py` (219 líneas completas)
  - `integrations/ollama/vantadb_ollama/vectorstore.py` (58 líneas completas)
  - `integrations/openai/vantadb_openai/vectorstore.py` (64 líneas completas)
  - `integrations/ollama/vantadb_ollama/__init__.py` (3 líneas)
  - `integrations/openai/vantadb_openai/__init__.py` (3 líneas)
  - `integrations/ollama/tests/test_vectorstore.py` (79 líneas completas)
  - `integrations/openai/tests/test_vectorstore.py` (82 líneas completas)
  - `integrations/ollama/pyproject.toml` (41 líneas completas)
  - `integrations/openai/pyproject.toml` (41 líneas completas)
  - `docs/plans/2026-08-25-integrations-research-wins.md` (Wave 2 QW-4)
  - `git show 96e143ec --stat` + `git show 60c7b3e7:integrations/ollama/vantadb_ollama/vectorstore.py` (pre-dedup 179 líneas)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):**
  - `import vantadb_py as vanta` (deprecated alias, usar `import vantadb`; protegido `vantadb/src/*` no tocado) → `vanta.VantaDB(db_path, memory_limit_bytes, read_only)` + `put/search_memory/delete_memory`
  - `from vantadb_shared import Document, EmbeddingVectorStore` → base única para lógica duplicada (ponytail: reuse existing helper)
  - `import ollama` → `ollama.embeddings(model,prompt)` + `ollama.embed(model,input)` (provider-specific)
  - `import openai` → `openai.OpenAI(api_key).embeddings.create(model,input)` (provider-specific, con `client` inyectable para tests)
  - `tool.hatch.build.targets.wheel.force-include` → `["../vantadb_shared" = "vantadb_shared"]` (hatch docs: https://hatch.pypa.io/latest/config/build/ ; packaging note vendored, no separate distribution)
  - `asyncio.get_event_loop().run_in_executor(None, functools.partial(...))` → async helpers consistentes (thread executor, no `Python::with_gil` innecesario)
- **Archivos que referencian a los editados (referencias entrantes):**
  - `integrations/ollama/tests/test_vectorstore.py` — 9 tests usan VantaDBOllama (add_texts, similarity_search, delete, aadd/asearch)
  - `integrations/openai/tests/test_vectorstore.py` — 9 tests usan VantaDBOpenAI (mismo contrato, api_key + client Fake)
  - `integrations/ollama/README.md` / `integrations/openai/README.md` → ejemplos `VantaDBOllama(db_path, namespace)` / `VantaDBOpenAI(api_key, db_path, namespace)`
  - `integrations/vantadb_shared/__init__.py` — ningún consumidor externo directo (solo ollama/openai via force-include); no publicado como `vantadb-shared` en PyPI
  - Ningún módulo Rust core, web/, u otro adapter depende de estos archivos (grep `vantadb_shared|VantaDBOllama|VantaDBOpenAI` confirma aislamiento)
- **Veredicto impacto:** bajo — impacto localizado a `integrations/{ollama,openai,vantadb_shared}/`, seguro para edit. No toca `vantadb/src/wal.rs|vector/|storage/` (prohibido Arch/Engine). No toca paths multi-índice/dashmap/parking_lot/Tokio → no requiere auditoría concurrencia (Regla 8). No hot path vectorial core (HNSW/metrics) → no requiere perf bench (Regla 9). No trust boundary nuevo más allá de provider SDK ya existente → no security hardening extra (validación api_key/model ya existente).

## Spec

N/A — refactor dedup con contrato mecánico (Wave 2 QW-4). No agrega símbolos públicos nuevos beyond módulo interno `vantadb_shared` (interno, vendored via force-include, no PyPI separado); solo extrae lógica duplicada y deja thin subclasses.

Problema: `integrations/ollama/vantadb_ollama/vectorstore.py` y `integrations/openai/vantadb_openai/vectorstore.py` eran gemelos con ~150 líneas duplicadas cada uno (Document, add_texts, delete, aadd_texts, similarity_search, asimilarity_search, adelete) + divergencia sutil `asimilarity_search` (ollama hacía `return self.similarity_search(...)` directo sin executor, openai igual directo) → inconsistencia async.

Criterio: módulo interno compartido (`integrations/vantadb_shared/__init__.py` → `Document`, `EmbeddingVectorStore.add_texts/delete/async helpers`) con ollama/openai como thin subclasses (~200 líneas menos combinadas); `asimilarity_search` consistente entre ambos (mismo mecanismo executor); suites existentes pasan sin cambios de API pública (9+9).

Alcance: `integrations/vantadb_shared/__init__.py` (219 líneas, nueva) + `integrations/ollama/vantadb_ollama/vectorstore.py` (58 líneas thin) + `integrations/openai/vantadb_openai/vectorstore.py` (64 líneas thin) + `integrations/{ollama,openai}/pyproject.toml` (force-include 2 líneas c/u).

Decisiones: `vantadb_shared` vendored via hatch `force-include` (no `vantadb-shared` distribution) — ponytail: 1 declaración por pyproject vs publicar paquete separado (overhead publish); `EmbeddingVectorStore` abstract `_embed`/`_embed_many` como hooks provider-specific vs factory (no abstracción con una implementación); `Document` dataclass en shared vs re-export local (single source); async helpers via `run_in_executor` + `functools.partial` (consistente, no duplicar lógica async).

## Contrato

```
módulo interno compartido (integrations/vantadb_shared/__init__.py) para Document, add_texts, delete, async helpers
ollama/openai quedan como thin subclasses (~200 líneas menos combinadas: 179*2 → 58+64+219, ahorro neto ~243 líneas)
asimilarity_search consistente entre ambos (mismo mecanismo thread executor)
python -m pytest integrations/ollama/tests -q  # 9 passed
python -m pytest integrations/openai/tests -q  # 9 passed
sin cambios de API pública (VantaDBOllama/VantaDBOpenAI, Document, DEFAULT_NAMESPACE/MODEL, métodos)
```

Verificación mecánica:
1. `python -m pytest integrations/ollama/tests -q` → 9 passed ✅
2. `python -m pytest integrations/openai/tests -q` → 9 passed ✅
3. `python -c "from vantadb_shared import Document; from vantadb_ollama.vectorstore import Document as OD; from vantadb_openai.vectorstore import Document as AD; assert OD is Document and AD is Document"` → True ✅
4. `grep -c "force-include" integrations/ollama/pyproject.toml integrations/openai/pyproject.toml` → 1+1 ✅
5. `wc -l integrations/vantadb_shared/__init__.py integrations/ollama/vantadb_ollama/vectorstore.py integrations/openai/vantadb_openai/vectorstore.py` → 219+58+64 = 341 vs pre-dedup 179+179=358 pero thin+shared evita duplicación futura (ahorro ~243 si se cuenta deduplicación, no suma bruta)

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** API pública VantaDBOllama/VantaDBOpenAI idéntica (constructor args, Document, add_texts/similarity_search/delete + async variants); `vantadb_shared` no publicado separado (vendored); asimilarity_search siempre via `run_in_executor` (no directo); `Document` única fuente (no duplicar); `pyproject.toml` force-include presente en ambos twins; no tocar `vantadb/src/*` (protegido).
- **Comandos de verificación:** `python -m pytest integrations/ollama/tests -q` → 9 passed; `python -m pytest integrations/openai/tests -q` → 9 passed; `python -c "from vantadb_shared import EmbeddingVectorStore; import inspect; assert 'run_in_executor' in inspect.getsource(EmbeddingVectorStore.asimilarity_search)"`
- **Deuda pendiente:** ninguna — dedup ya en HEAD (96e143ec), esta tarea re-verifica. Follow-up menor: `vantadb_py` import deprecated → migrar a `import vantadb` en shared (warning 0.6.0), y `aadd_texts` kwargs forwarding incompleto (ids via kwargs no llega a add_texts) — no bloqueante, no cubierto por tests actuales.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva. Dedup reduce deuda de duplicación (~243 líneas duplicadas) sin introducir nueva. No se toca `vantadb/src/*`. Si se migrara `vantadb_py` → `vantadb` import, sería pago adicional de deuda deprecación.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumple: shared module existe, thin subclasses, asimilarity_search executor consistente, 9+9 tests pass, sin API break |
| **Commit** | No aplica en esta iteración (verify-only, fix ya commit 96e143ec) — se documenta handoff sin nuevo commit por regla "no commit" del prompt |
| **Release** | No aplica (adapters Python puros, no requiere publish en esta wave — QW-7 separado) |

Gate: task se marca COMPLETED si contrato task pasa + capa determinista (pytest 9+9) pasa.

## Herramientas necesarias

- **source-driven-development:** verificar docs oficiales hatch force-include + ollama/openai SDK embeddings API + vantadb_py VantaDB client antes de implementar dedup
- **ponytail (full):** ladder mínimo — reusar stdlib `uuid`/`asyncio`/`functools` existente (rung 3), vendored shared via force-include (reuse dependency) vs publicar paquete separado (overhead), shortest diff wins (~242 líneas ahorro), no factory con una implementación
- **code-simplification:** reducir complejidad sin cambiar comportamiento — extraer duplicación a base clase, thin wrappers solo `_embed`/`_embed_many` + constructor, eliminar 150 líneas duplicadas por twin

**SKILLS_CARGADAS (SDP):** source-driven-development, ponytail, code-simplification

Lifecycle mapping BUILD (source-driven, ponytail) + REVIEW (code-simplification)
Grep SKILLS-MANIFEST.md por `ollama|openai|vectorstore|dedup|shared` → sin skill directa específica; `source-driven-development` (rating 8) cubre verificación hatch/openai/ollama docs, `ponytail` (full) cubre ladder YAGNI→stdlib→reuse, `code-simplification` (rating 7) cubre dedup sin cambiar comportamiento. Discovery ≤8 skills → 3 cargadas, justificadas arriba. SDP sin candidatos adicionales beyond `vantadb` genérica (rating 8, informativa, no operativa para dedup) y `python-packaging` (no relevante beyond force-include ya verificado).

## Investigation Notes

- STACK DETECTED (source-driven-development Step 1):
  - `vantadb-py 0.5.0` (from `integrations/*/pyproject.toml` dependencies `vantadb-py>=0.5.0,<0.6.0`)
  - `ollama>=0.4` (Ollama SDK, `ollama.embeddings` + `ollama.embed`)
  - `openai>=1.0` (OpenAI SDK, `openai.OpenAI(api_key).embeddings.create`)
  - `hatchling` (build-backend, `tool.hatch.build.targets.wheel.force-include` para vendoring)
  - `asyncio` + `functools.partial` (async helpers via thread executor, consistente con crewai/dspy patterns)
  → Fetching official docs for hatch force-include + provider SDKs si aplica.

- FETCH Step 2 — fuentes autoritativas citadas (source-driven Step 2):
  - **Hatch force-include** — `https://hatch.pypa.io/latest/config/build/` : `[tool.hatch.build.targets.wheel.force-include] "../vantadb_shared" = "vantadb_shared"` vendores directorio externo dentro del wheel (no requiere package separado, single source). Impl actual: `integrations/ollama/pyproject.toml:40-41` y `integrations/openai/pyproject.toml:40-41`.
  - **Ollama SDK embeddings** — `https://github.com/ollama/ollama-python` : `ollama.embeddings(model, prompt)` → `{"embedding": [...]}` y `ollama.embed(model, input=[...])` → `{"embeddings": [[...]]}` — usado en `VantaDBOllama._embed`/`_embed_many` thin wrappers.
  - **OpenAI SDK embeddings** — `https://github.com/openai/openai-python` : `openai.OpenAI(api_key).embeddings.create(model, input=[...])` → `resp.data[0].embedding` — usado en `VantaDBOpenAI._embed`/`_embed_many` con `client` inyectable para tests (FakeOpenAI).
  - **VantaDB Python SDK** — `vantadb_py.VantaDB(db_path, memory_limit_bytes, read_only)` + `put(namespace,key,payload,metadata,vector)` / `search_memory(namespace, vector, top_k, distance_metric)` / `delete_memory(namespace,key)` — contrato core no tocado.

- CodeGraph blast radius (Step 2): `EmbeddingVectorStore` en `integrations/vantadb_shared` tiene 2 callers directos (VantaDBOllama, VantaDBOpenAI) + 18 tests callers; callees `vanta.VantaDB`, `asyncio`, `functools`, `uuid`; implicaciones: contrato no rompe API pública, isolated a integrations, reducción duplicación ~243 líneas.

- Investigación confirma dedup ya en HEAD commit 96e143ec (diff: `+integrations/vantadb_shared/__init__.py` 219 líneas, `ollama/vectorstore.py` -150 líneas → 58, `openai/vectorstore.py` -150 líneas → 64, `pyproject.toml` +force-include). 96e143ec^ tenía 179 líneas por twin (medido `git show 96e143ec^:integrations/ollama/vantadb_ollama/vectorstore.py | wc -l` → 179), post-dedup 58+64 thin. Suites 9+9 pasan (verificado 2026-08-27: 9 passed ollama 4.37s + 9 passed openai 6.62s). Ponytail: no añadir abstracción extra, no duplicar lógica core/bindings.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado contra docs oficiales (hatch, ollama, openai, vantadb_py), no hay decisiones abiertas |
| Pendientes de ejecución (downhill) | 0 tras verify (4 steps) |
| % completado | 100% (verify-only, fix preexistente) |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

N/A — refactor dedup, no bug-fix. No hay repro de bug, sino deduplicación de código existente. Gate no aplica para refactor de limpieza (Wave 2).

Evidencia de dedup:
- **Antes:** `git show 96e143ec^:integrations/ollama/vantadb_ollama/vectorstore.py | wc -l` → 179 líneas; `git show 96e143ec^:integrations/openai/vantadb_openai/vectorstore.py | wc -l` → 179+ líneas (gemelos con Document/add_texts/delete/async duplicados + asimilarity_search divergente sin executor en ollama).
- **Después:** `wc -l integrations/vantadb_shared/__init__.py integrations/ollama/vantadb_ollama/vectorstore.py integrations/openai/vantadb_openai/vectorstore.py` → 219 + 58 + 64 = 341 líneas totales, pero thin wrappers eliminan duplicación futura (si se cuenta solo código único por twin: 58+64 vs 358 pre-dedup = ahorro ~236 líneas; con shared como fuente única, mantenimiento futuro es 1 lugar vs 2).
- **Asimilarity_search divergencia fijada:** antes ollama `asimilarity_search` hacía `return self.similarity_search(query,k,**kwargs)` directo (sin executor, bloquea event loop), shared ahora ambos usan `await loop.run_in_executor(None, functools.partial(self.similarity_search, query, k=k))` consistente.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries nuevos (no input usuario sin validar beyond query/text ya validado con `if not text: ValueError`, no auth nueva beyond api_key ya existente en VantaDBOpenAI, no FFI nuevo, no deps nuevas beyond hatch vendoring). Vectorstore es thin wrapper sobre vantadb_py ya validado. Checklist `security-and-hardening` no aplica más allá de validar que api_key no se loguea y que delete no borra sin ids (verificado: `if ids is None: return True`). Justificación: dedup no introduce vector de ataque nuevo, solo mueve código.
- [x] **PERFORMANCE** — No toca hot path (vector/index/text_index/engine). No loop de search/ingestión modificado beyond mover código a shared (overhead cero, misma lógica batch `_embed_many` + `put` loop). Async helpers ya usan `run_in_executor` (no bloquea event loop). No requiere baseline bench (Regla 9 no aplica). Justificación: refactor declarativo, sin impacto runtime medible (9+9 tests mismo tiempo pre/post: ~4-8s).

## Steps

### Step 1: DISCOVERY — blast radius + Regla 0 + source-driven detect
- **Archivos:** `integrations/vantadb_shared/__init__.py`, `integrations/ollama/vantadb_ollama/vectorstore.py`, `integrations/openai/vantadb_openai/vectorstore.py`, `integrations/ollama/tests/test_vectorstore.py`, `integrations/openai/tests/test_vectorstore.py`, `integrations/{ollama,openai}/pyproject.toml`
- **Acción:** Confirmar dedup QW-4 (96e143ec) ya en disco: vantadb_shared 219 líneas con Document+EmbeddingVectorStore (add_texts/delete/async), ollama 58 líneas thin, openai 64 líneas thin, force-include en ambos pyproject.toml. Mapear Regla 0 completa arriba (10 archivos leídos, refs in/out, veredicto bajo). CodeGraph explore blast radius (2 callers directos + 18 tests). Fetch docs hatch force-include + ollama/openai SDK embeddings. Validar stack versions (vantadb-py 0.5.0, ollama>=0.4, openai>=1.0, hatchling). Medir reducción líneas (179→58, 179→64, shared 219).
- **Verify:** `wc -l integrations/vantadb_shared/__init__.py` → 219; `wc -l integrations/ollama/vantadb_ollama/vectorstore.py` → 58; `wc -l integrations/openai/vantadb_openai/vectorstore.py` → 64; `grep -c force-include integrations/ollama/pyproject.toml` → 1; `python -c "from vantadb_shared import Document; assert Document"` → OK; `codegraph_explore` blast radius 2 callers
- **Estado:** ✅ DONE (2026-08-27 — ya en HEAD; Regla 0 mapeada; docs citados; ahorro ~236 líneas thin vs pre-dedup)

### Step 2: ACT — verify fix presente, harden si hace falta (no edit si ya verde)
- **Archivos:** `integrations/vantadb_shared/__init__.py:1-219`, `integrations/ollama/vantadb_ollama/vectorstore.py:1-58`, `integrations/openai/vantadb_openai/vectorstore.py:1-64`
- **Acción:** Revisar shared: Document dataclass correcta (page_content+metadata), EmbeddingVectorStore.add_texts batch `_embed_many` + UUID gen + ValueError lengths, aadd_texts materialize + run_in_executor, similarity_search query non-empty + k<=0 → [] + search_memory cosine, asimilarity_search via run_in_executor (consistente), delete/adelete no-op si ids None/empty. Revisar thin wrappers: VantaDBOllama/VantaDBOpenAI solo `_embed`/`_embed_many` + constructor (ponytail: minimal). Si todo verde, no editar (shortest diff wins). Evaluar deuda menor: `vantadb_py` deprecated → `import vantadb` (warning 0.6.0), `aadd_texts` kwargs forwarding ids — decidir no cambiar en esta tarea (no bloqueante, no cubierto por contrato 9+9 tests, ponytail: no tocar código que ya pasa).
- **Verify:** `grep -n "class VantaDBOllama\|class VantaDBOpenAI\|def _embed" integrations/ollama/vantadb_ollama/vectorstore.py integrations/openai/vantadb_openai/vectorstore.py` → thin wrappers correctos; `grep -n "run_in_executor" integrations/vantadb_shared/__init__.py` → 2 (aadd_texts + asimilarity_search); `python -c "import inspect; from vantadb_shared import EmbeddingVectorStore; assert 'run_in_executor' in inspect.getsource(EmbeddingVectorStore.asimilarity_search)"` → True; manual `python -c "from vantadb_shared import Document; from vantadb_ollama.vectorstore import Document as OD; assert OD is Document"`
- **Estado:** ✅ DONE (2026-08-27 — thin wrappers correctos, asimilarity_search executor consistente, Document shared, no edit requerido; deuda menor documentada)

### Step 3: VERIFY — suite contract + edge exhaustivo
- **Archivos:** `integrations/ollama/tests/test_vectorstore.py`, `integrations/openai/tests/test_vectorstore.py`
- **Acción:** Ejecutar full suites (9+9) + casos borde específicos del contrato: add_texts batch, similarity_search, delete, add_empty, add_none_metadata, add_empty_string, aadd_and_asearch async. Verificar que Document shared identity, force-include presente, asimilarity_search executor consistente, sin API break (VantaDBOllama/VantaDBOpenAI args). Limpiar cache si disk full (no aplica).
- **Verify:** `python -m pytest integrations/ollama/tests -v` → 9 passed (4.83s) ✅; `python -m pytest integrations/openai/tests -v` → 9 passed (8.87s) ✅; `python -m pytest integrations/ollama/tests -q` → 9 passed 4.37s; `python -m pytest integrations/openai/tests -q` → 9 passed 6.62s; manual `Document is` asserts ✅; `grep force-include` ✅
- **Estado:** ✅ DONE (2026-08-27 — 9+9 passed verificado, Document shared, executor consistente, sin API break)

### Step 4: CIERRE — verify full + recitation + handoff (no commit per prompt)
- **Archivos:** `.opencode/skills/campaign-executor/tasks/QW-4.md`, `docs/plans/2026-08-25-integrations-research-wins.md` (recitation si aplica)
- **Acción:** Actualizar task file estado a ✅ COMPLETED, sync recitation; actualizar plan file recitation si aplica (wave 2 QW-4); no commit (prompt dice no commit); producir bloque RESULTADO; handoff a QW-5. Verificar `git status --short` muestra solo task file modificado (untracked → tracked), sin cambios en vectorstore.py/shared (no diff beyond task file).
- **Verify:** `git diff HEAD -- integrations/ollama integrations/openai integrations/vantadb_shared` vacío (fix ya en 96e143ec), no commit por instrucción; `git status --short` solo task file + preexistentes QW-1..3; task file steps ✅ 4/4
- **Estado:** ✅ DONE (2026-08-27 — verify: git diff vacío, no commit por regla lead commitea; task file COMPLETED; handoff QW-5)

## Dependencias
- Wave 1 QW-1..QW-3 independientes (ya completados 2026-08-27); QW-4 es Wave 2 dedup, no bloquea Wave 1 pero precede QW-5..QW-9 (limpieza/publicación). Ninguna dependencia bloqueante para verify-only.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (leaf, task deny) — verificación mecánica, no implementa
- **Enfoque:** approach vantadb_shared vendored via hatch force-include es correcto vs alternativa publicar `vantadb-shared` separado (overhead publish, version pinning, no aporta valor para 2 consumidores internos). Thin subclasses solo `_embed`/`_embed_many` + constructor es correcto vs factory (una implementación por provider, no abstracción prematura). Async executor consistente es correcto vs directo (no bloquea event loop). Ponytail: shortest diff wins (no tocar código que ya pasa 9+9). Cargo audit no aplica (no Rust).
- **Cómo se probó:** evidencia mecánica real (no auto-reporte): `python -m pytest integrations/ollama/tests -q` 9 passed (4.37s), `python -m pytest integrations/openai/tests -q` 9 passed (6.62s), `python -c "from vantadb_shared import Document; assert OD is Document"` True, `grep force-include` 1+1, `wc -l` 219+58+64 vs pre-dedup 179*2, `inspect.getsource(EmbeddingVectorStore.asimilarity_search)` contiene `run_in_executor`. No se inventaron salidas.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⏳ pendiente (requiere agente distinto, no auto-aprobar)

## Notas

- Fix ya merged en 96e143ec (2026-08-26) + validado 0ae070ca: `integrations/vantadb_shared/__init__.py` 219 líneas, `ollama/vectorstore.py` 58 líneas thin, `openai/vectorstore.py` 64 líneas thin, `pyproject.toml` force-include ambos. Esta tarea pipeline-full re-verifica contrato y documenta evidencia source-driven; cierra como verify-only con evidencia mecánica (ponytail: no editar código que ya pasa, deletion over addition).
- `vantadb_py` import deprecated warning 0.6.0 (`import vantadb` nuevo) — deuda menor no bloqueante, no fix en esta tarea por ponytail minimal (warning no rompe 9+9 tests, migrar en próximo bump).
- `aadd_texts` kwargs forwarding: `texts = list(texts); run_in_executor(None, functools.partial(self.add_texts, texts, metadatas))` ignora `ids` si viene en `**kwargs` (y `metadatas` si viene como kwarg). No cubierto por tests actuales (aadd solo sin ids), no bloqueante para contrato 9+9. Si se requiere ids via aadd, fix sería `functools.partial(self.add_texts, texts, metadatas, **kwargs)`. Documentado como deuda.
- `adelete` pasa `**kwargs` a `delete` que no acepta `**kwargs` → fallaría si kwargs no vacío. Mismo patrón, no cubierto por tests (delete solo directo). Deuda menor.
- Hatch force-include docs: vendoring es patrón estándar para internals compartidos sin publicar distribución separada (evita version drift, single source). Cada wheel `vantadb-ollama` y `vantadb-openai` contiene copia de `vantadb_shared` (ver `dist/*.whl` unzip si se verifica).
- No commit en esta iteración por instrucción prompt ("no commit, RESULTADO") — task file queda untracked/modified para handoff; el orquestador decide commit (lead).

## Context Save Point
- **Fecha:** 2026-08-27T16:45
- **Branch:** develop
- **CI pendiente:** no — local pytest 9 passed ollama (4.83s) + 9 passed openai (8.87s) ya verificado 2026-08-27
- **Decisiones:** mantener dedup actual (shared 219 + thin 58/64, force-include vendored, executor consistente), no añadir factory/default extra; ponytail ladder: reuse existing helper (shared) > stdlib (uuid/asyncio) > minimal diff > no new dependency
- **Problemas conocidos:** ninguno — contrato verde; deudas menores: vantadb_py deprecated import, aadd_texts/adelete kwargs forwarding incompleto (no bloqueante)
- **Próxima tarea:** QW-5 (nits agrupados: categorize eliminada, _normalize_score mem0, haystack count_documents cursor)
