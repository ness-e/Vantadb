# Review de Módulo — `providers/`

**Fecha:** 2026-08-23 · **Revisor:** ox-alpha (worker) · **Alcance:** `providers/openai`, `providers/ollama`, `providers/litellm`
**Método:** lectura completa del código fuente de los 3 crates + tests + `.pyi` + Cargo.toml + README; verificación contra la API real de `vantadb::sdk` y el workspace root. Verificación PyPI en vivo.

---

## Resumen

`providers/` contiene **3 crates Rust independientes con bindings PyO3** (no adapters Python): `vantadb-openai`, `vantadb-ollama` y `vantadb-litellm`. Cada uno expone **una única pyclass** (`VantaDBOpenAI` / `VantaDBOllama` / `VantaDBLiteLLM`) que combina dos cosas: (1) generación de embeddings delegando al SDK Python del proveedor (duck-typing vía `PyModule::import`), y (2) persistencia/búsqueda vectorial directa sobre `VantaEmbedded` del core. Son la capa "embedding-provider" nativa, complementaria a los 9 adapters puros-Python de `integrations/`.

## Arquitectura

```
Python user
   │  from vantadb_openai import VantaDBOpenAI
   ▼
pyclass (providers/*/src/python.rs)
   ├── embed() ──────────► SDK Python del proveedor (openai / ollama / litellm), vía reflexión PyO3
   └── store/search/get/list/delete/list_namespaces ──► vantadb::sdk::VantaEmbedded (core directo, sin pasar por vantadb-py)
```

- Cada crate declara `[workspace]` vacío → **excluido del workspace raíz por diseño**.
- Dependen del core vía `path = "../.."` con `default-features = false, features = ["fjall", "memmap2"]`.
- Feature-gate `python = ["pyo3"]` correcto; `crate-type = ["cdylib", "rlib"]`.
- Las operaciones de motor liberan el GIL (`py.detach`) — patrón correcto y comentado.

## Estado ACTUAL del issue CRIT-09 ("[workspace] providers")

**Resuelto por decisión documentada.** El `Cargo.toml` raíz dice explícitamente:

> *"NOTE: providers/openai, providers/ollama, providers/litellm are NOT workspace members. They use pyo3 + cdylib which triggers MSVC linker crash on Windows during workspace builds."*

Cada crate tiene su propio `Cargo.lock` y su `target/` local (verificado: `target/` está git-ignored; en git solo hay fuentes). No hay drift: los tres usan `pyo3 = "0.29"` igual que el core y apuntan al mismo path del core. La exclusión es coherente, no un accidente pendiente.

## Fortalezas

1. **GIL release correcto** en todas las operaciones de motor (`py.detach`) con comentario explicativo.
2. **Mapeo de errores FFI completo**: `NotFound→PyKeyError`, `BackendError→PyRuntimeError`, `InvalidInput/SchemaError/SerializationError→PyValueError`, fallback a `PyRuntimeError` con `Debug`.
3. **Docstrings estilo Sphinx** (Args/Returns) embebidas en Rust para las pyclasses.
4. **Sin `unsafe`**, sin red de red innecesaria; metadata dict→`VantaValue` tipado (str/bool/int/float).
5. Exclusión del workspace documentada con causa técnica (CRIT-09 cerrado correctamente).

## Hallazgos

| # | Severidad | Archivo(s) | Hallazgo |
|---|-----------|------------|----------|
| P1 | **Critical** | `providers/*/tests/test_*.py` (los 3) | **Tests rotos contra la firma actual**: llaman `store.search(embedding, top_k=5)` sin el `namespace` obligatorio que exige el Rust actual → `TypeError` seguro en `test_store_and_search` de openai, ollama y litellm. Los tests están desactualizados respecto a la API. |
| P2 | **Critical** | `providers/ollama/tests/test_ollama.py` | El fixture usa `vanta.VantaDB(path)` + `s.create_namespace("test_ollama")`. Verificado hoy: **`create_namespace` NO existe en vantadb-python** (grep sobre `src/*.rs` y `vantadb_py/*.py`: cero resultados) → `AttributeError`, suite entera roja. |
| P3 | **High** | `providers/*/vantdab_*.pyi` (los 3) | **Stubs `.pyi` desincronizados**: firman `search(query_embedding, top_k)` sin `namespace`/`text_query`/`filters`/`distance_metric`; omiten `get`, `list`, `delete`, `list_namespaces`; el `__init__` omite `timeout`. Cualquier IDE/type-checker miente al usuario. |
| P4 | **High** | `providers/*` (los 3) | **No empaquetables como wheels**: no existe `pyproject.toml` con build-backend maturin; además `Cargo.toml` tiene `publish = false`. Verificado en vivo: `vantadb-openai` da 404 en PyPI. Un dev no puede instalar estos providers salvo compilando a mano. |
| P5 | **Medium** | `openai/src/python.rs`, `ollama/src/python.rs` vs `litellm/src/python.rs` | **API inconsistente entre crates**: openai devuelve `"text"` en records, litellm devuelve `"payload"`; litellm expone `node_id`, los otros no; `list()` usa `"next_cursor"` en openai/litellm pero `"cursor"` en ollama; `limit` es `i32` en openai y `usize` en ollama/litellm. Misma operación, tres contratos distintos. |
| P6 | **Medium** | los 3 `python.rs` | **~85% de duplicación entre crates**: `record_to_pydict`, `err_to_py`, extracción de metadata y el loop de store son casi idénticos copia-pega. Debería existir un crate compartido `vantadb-providers-common` o macros. Costo real ya pagado: la inconsistencia P5 es consecuencia directa. |
| P7 | **Low** | `litellm/src/python.rs:131` | `self.embed_fn.as_ref().unwrap()` — viola la regla anti-unwrap. El invariante (set en constructor) lo hace seguro hoy, pero es frágil ante refactors. |
| P8 | **Low** | `README.md` de los 3 | READMEs mínimos y desactualizados: dicen "Methods: embed, search, store" cuando cada clase tiene 7-8 métodos. |
| P9 | **Low** | tests varios | `test_init` de openai requiere el paquete pip `openai` instalado (el constructor lo importa); no hay skip condicional (`pytest.importorskip`). En CI sin `openai` instalado, falla por import y no por lógica. |

### Ponytail-audit (solo complejidad)

- `shrink:` un solo crate `providers/common` eliminaría ~500 líneas duplicadas entre los 3. [providers/*/src/python.rs]
- `delete:` campo `timeout` guardado pero muerto (`#[allow(dead_code)]`) — se pasa al cliente del proveedor y se olvida. [openai, litellm]

## Incompletudes

- No hay forma oficial de instalar los providers (P4): sin maturin config, sin wheels publicados.
- Sin tests de integración reales contra OpenAI/Ollama (los tests actuales nunca llaman a `embed()` — razonable para CI offline, pero entonces falta un test mockeado de `embed()`).
- No verificado si algún workflow de CI compila estos crates (están fuera del workspace; `cargo check -p vantadb-openai` desde la raíz no los alcanza).

## Propuestas (priorizadas)

1. **P1+P2:** actualizar los 3 test files a la firma actual (`search(ns, emb, ...)`) y eliminar el uso de `create_namespace` en el fixture de ollama. Es el mínimo para volver a tener señal verde.
2. **P3:** regenerar los `.pyi` desde las firmas reales (manual, son 7 líneas cada uno).
3. **P4:** decidir destino: o se agregan `pyproject.toml` + maturin y se publica a PyPI, o se marca el directorio como experimental-interno en docs. Hoy es tierra de nadie.
4. **P6:** extraer `providers/common` (helpers PyO3 compartidos) — reduce duplicación y elimina P5 por construcción.
5. Unificar el contrato de salida (`text` vs `payload`, `cursor` vs `next_cursor`, tipos de `limit`) antes de publicar cualquier cosa: es API pública potencial.

## Score

**5.0 / 10**

Código Rust sólido y bien escrito (errores, GIL, docstrings), CRIT-09 correctamente resuelto y documentado. Pero: tests que no pueden pasar, stubs que mienten, tres APIs divergentes para la misma operación, y cero camino de distribución. El código está mejor que su ecosistema.

## No verificado

- **Tests de litellm más allá de la línea 80** (leí hasta `test_search_returns_score`; el resto del archivo no fue leído).
- Si existe algún job de CI que compile `providers/*` (requeriría leer `.github/workflows/`).
- Comportamiento runtime real de los builds (no ejecuté `cargo build` — fuera del alcance de esta review estática).

---

## Trazabilidad Backlog

Derivado a la fase **P32** de `docs/Backlog.md` (2026-08-23):

| Hallazgo | Tarea |
|---|---|
| P1/P2 — Tests rotos contra la firma actual (`search` sin namespace obligatorio; fixture usa `create_namespace` inexistente) | **MOD-41** |
| P4 — Indistribuibles: sin `pyproject.toml`/maturin, `publish = false`, 404 en PyPI | **MOD-42** |
| P6 — ~85% de duplicación entre los 3 crates (causa directa de la inconsistencia P5) | **MOD-43** |
| P3 — Stubs `.pyi` desincronizados: firman una API que ya no existe | **MOD-44** |
| P7–P9 — nits (unwrap frágil en litellm, READMEs desactualizados, sin `importorskip` para CI) | **MOD-45** |
