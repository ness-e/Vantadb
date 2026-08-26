# INV-providers-01 — Investigación profunda: adapters de inference

> **Fecha:** 2026-08-25 · **Comando:** `/research providers` · **Modo:** read-only
> **Registro:** `.opencode/references/research-modules.md` fila `providers`
> **Antecedente:** `docs/reviews/modulos/providers.md` (review 2026-08-23, score 5.0)

---

## Resumen ejecutivo

`providers/` son 3 crates PyO3 independientes (`vantadb-openai`, `vantadb-ollama`,
`vantadb-litellm`) que combinan generación de embeddings vía el SDK Python del
proveedor (duck-typing por reflexión) con persistencia/búsqueda sobre
`VantaEmbedded`. **El estado empeoró desde el review 2026-08-23 (5.0 → ~4.0):**
el crate openai ya no compila contra el core actual, las tareas MOD-41..45 que
derivaban los hallazgos previos **desaparecieron del Backlog sin registro en
docs/avance**, y los tests/stubs rotos siguen exactamente igual.

## 1. Usuarios objetivo

Devs Python/AI que conectan embeddings al engine. Flujo esperado:
`pip install vantadb-openai` → `VantaDBOpenAI(path, key)` → embed/store/search <5min.
**Hoy el flujo es imposible:** no hay wheel (404 PyPI verificado 2026-08-23), y el
constructor requiere además `pip install openai|ollama|litellm` (requisito no
documentado en ningún README).

## 2. Estándares del ecosistema (verificado contra docs oficiales)

- **LiteLLM `embedding(model=, input=)`** soporta batch, `api_key`, `timeout`
  por llamada y async `aembedding()` — [docs.litellm.ai/docs/embedding/supported_embedding](https://docs.litellm.ai/docs/embedding/supported_embedding). Nuestro uso del kwargs coincide salvo `timeout` (H-07).
- **Ollama Python `Client(host=...)`** pasa kwargs extra a `httpx.Client`
  (timeout válido) y `client.embed(model=, input=[list])` es batch nativo;
  existe `AsyncClient` — [github.com/ollama/ollama-python](https://github.com/ollama/ollama-python).
- **OpenAI SDK v1**: `client.embeddings.create(model=, input=)` — nuestro uso coincide.
- Convención del ecosistema: wheels publicados, stubs tipados, batch embedding,
  IDs deterministas con upsert (chroma `add/update(ids=...)`). Nosotros: nada de eso.

## 3. Competidores — matriz mínima

| | VantaDB providers | LiteLLM (uso directo) | fastembed | chromadb (EF) |
|---|---|---|---|---|
| Arquitectura | PyO3 cdylib ×3, engine embebido | Gateway Python puro | ONNX local, sin server | DB+embeddings integrados |
| Batch embed | ✅ (lista) | ✅ | ✅ | ✅ |
| Custom ID / upsert | ❌ keys autogeneradas | n/a | n/a | ✅ |
| Distribución | ❌ sin wheels | ✅ PyPI | ✅ PyPI | ✅ PyPI |
| Stubs tipados | ❌ stale | ✅ | ✅ | ✅ |
| Licencia | Apache-2.0 | MIT | Apache-2.0 | Apache-2.0 |

Claims de performance de competidores: no citados aquí (Regla 11 — ninguno fue
necesario para las decisiones; los nuestros: **cero números medidos**, ausencia registrada).

**Diferenciación honesta:** el único valor propio es *persistencia vectorial
embebida local* acoplada al embed. LiteLLM solo transporta; nosotros almacenamos.
Sin wheels ese valor es inaccesible para usuarios reales.

## 4. Estado interno (evidencia file:line)

- API surface por clase: `embed/get/list/search/store/delete/list_namespaces` (7 métodos).
- GIL release correcto en operaciones motor (`py.detach`) — patrón consistente.
- Mapeo de errores FFI completo y correcto (`err_to_py` ×3 duplicado).
- Exclusión del workspace documentada (Cargo.toml:638-640, causa MSVC linker).
- CI compila los 3 crates (`ci-rust-10.yml:428-433`) pero **ningún workflow corre
  sus tests Python** (grep 0 resultados).
- Historial: review 2026-08-23 derivó MOD-41..45 → **hoy inexistentes en Backlog
  y docs/avance** (grep 0 resultados en ambos árboles).

## 5. Score por dimensión

| Dimensión | Score | Justificación breve |
|---|---|---|
| DX onboarding | 3 | Sin install path; stubs mienten; requisito pip oculto |
| Completitud funcional | 6 | 7 métodos sólidos; faltan custom keys/batch store |
| Performance/overhead | 6 | GIL release correcto; cero benchmarks (Regla 9/11) |
| Robustez | 5 | Errores tipados ✓; unwrap frágil litellm:131; defaults silenciosos |
| Seguridad | 7 | Sin unsafe; keys por parámetro (no logging visto) |
| Docs & ejemplos | 3 | READMEs de 5 líneas desactualizados |
| Observabilidad | 3 | Sin tracing/logging de errores de red |
| Testabilidad | 2 | Tests rotos, sin CI, sin mocks de embed() |
| Paridad inter-módulos | 2 | 3 contratos distintos + trait Rust `src/llm.rs` separado |
| Diferenciación vs LiteLLM | 5 | Valor único = storage embebido; inaccesible sin wheels |
| **GLOBAL** | **4.0 / 10** | Empeoró vs 5.0 del 2026-08-23 (regresión de compilación) |

## Quick wins (<1 día) vs apuestas estratégicas

- **Quick wins:** H-01 (1 línea), H-03 (stubs), H-07 (pasar timeout), H-09 (validar inputs), H-10 (READMEs).
- **Estratégicos:** H-04 (decisión distribución PyPI vs experimental), H-06 (crate común), H-08 (custom keys), H-14 (unificación Rust/Python provider surfaces).

---

## Apéndice de hallazgos H-NN (entrada Fase D)

| ID | Categoría | Severidad | Esfuerzo | Ubicación | Hallazgo |
|----|-----------|-----------|----------|-----------|----------|
| H-01 | APLICAR | Critical | 🟢 | `providers/openai/src/python.rs:296-302`; `src/sdk/types.rs:214-232` | **vantadb-openai NO compila**: `VantaMemoryListOptions` construido sin campo `exclude_superseded` (añadido por ADR-028) ni `..Default::default()` → E0063. El CI check existe (ci-rust-10.yml:431) → rojo o no corrió desde 28a1788d. Fix: añadir `exclude_superseded: false` (1 línea) |
| H-02 | APLICAR | Critical | 🟡 | `providers/*/tests/test_*.py` | Tests rotos (vigentes desde review 2026-08-23 P1/P2): litellm+openai llaman `search(emb, top_k)` sin `namespace` obligatorio; ollama usa `vanta.VantaDB().create_namespace()` que no existe en vantadb_py |
| H-03 | APLICAR | High | 🟢 | `providers/*/vantdab_*.pyi` (×3) | Stubs .pyi stale: firman `search(emb, top_k)` sin namespace/text_query/filters/distance_metric/top_k default; omiten get/list/delete/list_namespaces, params model/timeout/base_url |
| H-04 | ESTRATEGIA | High | 🔴 | `providers/*/Cargo.toml` (`publish = false`) | Sin camino de distribución: sin pyproject.toml/maturin, sin wheels, 404 PyPI. Decidir: publicar a PyPI (requiere CI release multiplataforma) vs declarar experimental-interno en docs |
| H-05 | MEJORAR | Medium | 🟡 | `litellm/python.rs` vs `ollama|openai/python.rs` | API inconsistente entre crates: records devuelven `payload` (litellm) vs `text` (ollama/openai); cursor key `next_cursor` vs `cursor`(string) vs `next_cursor`(i32); limit `usize` vs `i32`; litellm expone `node_id`, otros no |
| H-06 | MEJORAR | Medium | 🟡 | los 3 `python.rs` | ~85% duplicación (~500 líneas): `record_to_pydict`, `err_to_py`, extracción metadata, loop store idénticos copia-pega. Es la causa directa de H-05. Extraer helpers compartidos |
| H-07 | MEJORAR | Low | 🟢 | `litellm/src/python.rs:73-74,90` | `timeout` aceptado pero muerto (`#[allow(dead_code)]`) — nunca se pasa a `litellm.embedding()`, que SÍ soporta param `timeout` (doc oficial). Ollama/openai sí lo pasan al cliente |
| H-08 | AGREGAR | Medium | 🟡 | los 3 `store()` | Keys autogeneradas por nanosegundo — sin custom key ni upsert determinista (estándar del ecosistema: chroma `add(ids=...)`) |
| H-09 | MEJORAR | Low | 🟢 | los 3 `search()`/`store()` | `distance_metric` inválido cae silencioso a cosine; metadata con tipos no soportados (None/list/dict) se descarta sin warning |
| H-10 | MEJORAR | Low | 🟢 | los 3 `README.md` | READMEs de 5 líneas: "Methods: embed, search, store" cuando hay 7; sin quickstart completo, sin requisito `pip install openai/ollama/litellm` |
| H-11 | MEJORAR | Low | 🟢 | tests + `.github/workflows/` | Tests sin `pytest.importorskip` para SDKs Python; sin test mockeado de `embed()`; **ningún workflow corre estos tests** (solo cargo check) |
| H-12 | OPTIMIZAR | Low | 🟢 | los 3 `embed()` | Sin streaming/batching configurable; ollama ofrece `AsyncClient`, litellm `aembedding()` — no aprovechados. Solo relevante con volúmenes grandes |
| H-13 | APLICAR | High | 🟢 | `docs/Backlog.md`, `docs/avance/` | **Pérdida de trazabilidad**: MOD-41..45 derivados al Backlog el 2026-08-23 (providers.md §Trazabilidad) no existen hoy ni en Backlog ni en avance/historial — viola el invariant progreso. Re-registrar o archivar explícitamente |
| H-14 | ESTRATEGIA | Medium | 🔴 | `src/llm.rs` (trait `EmbeddingProvider`) vs `providers/` | Dos superficies de providers sin contrato común: el trait Rust (feature `remote-inference`, factory env-var, solo openai/ollama) y las pyclasses Python (openai/ollama/litellm). Documentar relación o unificar decisión de arquitectura (ADR) |

**Conteo:** 14 hallazgos — aplicar 4 · mejorar 6 · agregar 1 · optimizar 1 · estrategia 2 · descartar 0
