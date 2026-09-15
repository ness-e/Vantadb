# FIND-73 — `key` en `.pyi` + `verify_pyi` con firmas (providers drift)

> **Plan:** `docs/plans/2026-09-15-find-correcciones.md` (Task 11, Wave3) · **Appetite:** 4h · 🟢 · 🟡
> **Ruta:** vanta-worker · **Estado:** ⏳ IN PROGRESS · **Campaign:** 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
> **Gate:** drift stub-vs-runtime (PROV-10) + gate que no gatea (hasattr).
> **SDP:** campaign-executor, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development, api-and-interface-design (+ progreso auto; `frontend-ui-engineering` devuelto por lifecycle pero N/A — sin `web/` en scope)

## Impacto mapeado (Regla 0)

- **Leídos completos:** `providers/openai/vantadb_openai.pyi`, `providers/litellm/vantadb_litellm.pyi`, `providers/ollama/vantadb_ollama.pyi` (57L c/u), `.github/scripts/verify_pyi.py` (22L), `providers/ollama/README.md` (43L), runtime `providers/*/src/python.rs` (secciones `store`/`embed_batch`/`search`/`__init__`), `.github/workflows/providers-ci.yml:50-110` (invocación del script).
- **Referencias hacia dentro (qué importa el cambio):** CI `providers-ci.yml:78` hace `exec(...read().replace('${PROVIDER}', matrix))` — el script DEBE mantener el placeholder literal `"${PROVIDER}"` para no romper ese replace (Nota 2 del workflow: sin escape bash lo vacía).
- **Referencias entrantes (quién consume):** type-checkers de consumidores Python (stubs), `test_stub_drift.py` (solo `vantadb-python`, no providers — fuera de scope), tests `providers/*/tests/test_*.py` (usan `store(text, emb, metadata)` posicional — compatible con `key` opcional añadido al final).
- **Veredicto:** blast radius = 4 archivos editables (3 `.pyi` + 1 script) + 1 README solo-lectura (verificación). Sin símbolos públicos nuevos (`key` ya existe en runtime desde PROV-10). Sin hot path. **Gate D: no disparado** (fix mecánico, contrato cerrado, 0 uphill).

## Diff stub-vs-runtime (DISCOVERY — pre-mortem del plan)

| Método | Runtime (los 3 `python.rs`) | Stub (los 3 `.pyi`) | Veredicto |
|---|---|---|---|
| `__init__` | openai `(db_path, api_key, model, namespace, timeout)`; litellm `api_key=None`; ollama `(db_path, base_url, model, namespace, timeout)` | idénticos en cada stub | ✅ OK |
| `embed` | `(texts)` | `(texts)` | ✅ OK |
| `embed_batch` | `(texts, batch_size=100)` | `(texts, batch_size=100)` | ✅ OK (ya documentado) |
| `search` | `(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10)` | idéntico | ✅ OK |
| **`store`** | **`(text, embedding, metadata=None, key=None)`** | **`(text, embedding, metadata=None)` — SIN `key`** | ❌ **drift (único)** |
| `delete`/`get`/`list`/`list_namespaces` | `(key, namespace=None)` / `(namespace, key)` / `(namespace, limit=100, cursor=None)` / `()` | idénticos | ✅ OK |

HALLAZGOS extra del discovery (entran al slice 2, sin scope-creep — mismo archivo):
- H1: `required_methods` no incluye `embed_batch` (el contrato lo pide documentado/gateado).
- H2: `PROVIDER = "${PROVIDER}"` sin fallback a env → corrido local siempre cae al branch ollama. CI lo sustituye vía `.replace()` — mantener compat.
- H3: solo `hasattr` → no caza el drift `key` (el bug que motiva la tarea).

## Steps atómicos

- [x] **Step 1 — `key` en 3 stubs:** añadir `key: str | None = None` a `store()` en `providers/openai/vantadb_openai.pyi:21-26`, `providers/litellm/vantadb_litellm.pyi:21-26`, `providers/ollama/vantadb_ollama.pyi:21-26`. (~6 líneas, mecánico)
- [x] **Step 2 — `verify_pyi.py` con firmas:** `inspect.signature` runtime vs `ast` del stub por método (nombres + defaults, incl. `__init__`), + `embed_batch` en required, + fallback `os.environ["PROVIDER"]` manteniendo placeholder CI, + SKIP documentado (exit 0) si el módulo no está buildeado. (~120 líneas, stdlib solo)
- [x] **Step 3 — Verify + README + cierre:** `py_compile` 4 archivos, self-check del comparador (detecta drift pre-fix / pasa post-fix, 9 firmas ×3 providers + negativo `__init__`), `git diff --check`, `rg "def store"`, README ollama verificado (quickstart sin `key` sigue válido — param opcional; sin versión en el doc → sin drift; Cargo 0.5.0 ×3 consistente) + fila `store` actualizada en los 3 READMEs (Regla 3 doc-sync). Commit `fix:` + recitation + RESULTADO.

## Prohibidos (plan)

`.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, `stash@{0}`, archivos FIND-87 (`vantadb-ts`), FIND-71 (`embeddings`), `Cargo.toml`, runtime `*.rs` (solo lectura).
