# INV-integrations-01 — Investigación profunda: adapters de frameworks (`integrations/`)

**Fecha:** 2026-08-25 · **Comando:** `/research integrations` · **Plantilla:** `research-module.md`
**Método:** inventario interno (codegraph + verificación git/línea por línea del reporte previo `docs/reviews/modulos/integrations.md` del 2026-08-23) + investigación externa multi-fuente (docs oficiales de cada framework, PyPI, GitHub).
**Score global: 6.3 / 10**

---

## 1. Usuarios objetivo y su flujo diario

Devs Python de cada framework (langchain, llamaindex, dspy, haystack, crewai, letta, mem0, ollama, openai SDK) que quieren memoria persistente embebida para agentes.

Flujo esperado hoy:
```bash
pip install vantadb-langchain   # ❌ 404 en PyPI — el paso 1 falla
```
Workaround actual: `pip install git+https://...#subdirectory=integrations/langchain` — mata la adopción en cualquier contexto productivo. La metadata de los 9 `pyproject.toml` ya es publicable; es un gap de publicación, no de calidad (confirmado también en el review 2026-08-23 y vigente en Backlog como MKT-18f).

## 2. Estándares del ecosistema (verificado contra fuentes oficiales)

| Convención | Estado del ecosistema | VantaDB |
|---|---|---|
| Packaging de integraciones | **Paquete PyPI separado por framework** dominante (`llama-index-vector-stores-chroma` v0.5.5, `chroma-haystack`, partner-packages de LangChain). Excepción: mem0 monopackage | ✅ 9 directorios separados — alineado con la convención dominante… pero **ninguno publicado** |
| Distribución de stores Haystack 2.x | Doc oficial recomienda paquete standalone `<tech>-haystack` (docs.haystack.deepset.ai/docs/creating-custom-document-stores) | ✅ Patrón correcto |
| Memoria LangChain/LangGraph | Persistencia moderna vive en **LangGraph**: checkpointers + `BaseStore` KV con namespaces jerárquicos (reference.langchain.com/python/langgraph.store/base/BaseStore). `BaseMemory` clásico en camino de remoción desde 0.3.x *(media confianza — verificar release notes de langchain 1.x)* | ❌ Solo cubrimos `VectorStore` |
| LlamaIndex | `ChatMemoryBuffer` deprecado → clase `Memory`; vector stores siguen `BasePydanticVectorStore` (developers.llamaindex.ai → Memory guide) | ✅ Nuestro target sigue vigente |
| CrewAI v1.x | Clase `Memory` **unificada** que reemplaza short-term/long-term/entity/external (docs.crewai.com/concepts/memory) | ⚠️ Nuestro adapter es un `BaseTool`, no backend de esa Memory |

## 3. Competidores — matriz mínima

| Producto | Arquitectura | Integraciones | Licencia/actividad | Fuente |
|---|---|---|---|---|
| **mem0** (`mem0ai` 2.0.19) | Lib OSS de memoria, monopackage con todos los integrations adentro | Muchos frameworks, 1 solo pip install | Activa (PyPI verificado hoy); stars UNVERIFIED (GitHub API 403) | pypi.org/project/mem0ai |
| **zep** (`zep-python`) | **Cliente de servidor Zep** — ya no lib embebida | Plataforma | Competidor de plataforma, no de SDK local | github.com/getzep/zep-python |
| **cognee** | "AI memory platform" con knowledge graph self-hosted | Su propio runtime | licencia/stars UNVERIFIED esta pasada | github.com/topoteretes/cognee |
| Framework-native (langgraph BaseStore, llamaindex Memory, crewai Memory) | Memoria propia del framework | Cero instalación extra | El competidor real por-framework | docs oficiales c/u |

**Diferenciación vs {{COMPETIDOR_PRINCIPAL}} (mem0 como lib):** VantaDB es engine embebido Rust (WAL, HNSW+BM25 híbrido, TTL, graph) con un solo binario — mismo nicho "local-first" que mem0 pero con storage durable real. Zep requiere servidor; cognee trae su propio KG runtime. La propuesta de valor existe; **no es visible porque no hay nada instalable**.

Performance publicada de adapters: **claim sin evidencia** — no existen números medidos de overhead por-adapter (Regla 11).

## 4. Estado interno (evidencia file:line, verificada hoy)

- 9 adapters Python puros sobre `vantadb-py>=0.5.0,<0.6.0`, todos 0.5.0 hatchling, constructor consistente (`db_path`/`namespace`/`memory_limit_bytes`/`read_only`/`backend`) + embedding callable inyectable.
- Contratos: langchain `VectorStore` completo (~30 tests); haystack `DocumentStore` el más completo; dspy `Retrieve` correcto; llamaindex/mem0 correctos con bugs latentes; **crewai implementa `BaseTool`, no memoria**; **letta sin contrato alguno** (Letta es plataforma de agentes stateful con memoria propia — más competidor que host); ollama/openai son gemelos ~95% idénticos sin contrato de framework.
- **Los bugs del review 2026-08-23 siguen presentes:** `integrations/langchain/vantadb_langchain/vectorstore.py:470` (ids parciales) y `integrations/crewai/vantadb_crewai/vectorstore.py:164-176,217-218` (cursor str, from_dict string-embedding) verificados hoy. Cero commits en `integrations/` desde 2026-08-23.
- **⚠️ Hallazgo de proceso:** las filas **MOD-46..50 fueron removidas del Backlog activo sin completarse ni archivarse** (agregadas en commit c7b7e559 P32, ausentes hoy, ni en `backlog-history.md` ni en `docs/avance/`). Trabajo pendiente huérfano — viola el invariante de progreso skill.
- Historial relevante: MKT-18f activa (publicar adapters PyPI); review previo score 6.5.

## 5. Evaluación por dimensión (0-10)

| Dimensión | Score | Evidencia |
|---|---|---|
| DX onboarding | **4** | `pip install` 404 ×9; flujo exige install-from-git |
| Completitud funcional | **7** | Vector-store path sólido en 6/9; gaps LangGraph/crewai-Memory/ollama-openai |
| Performance/overhead | **6** | Sin números por-adapter (claim sin evidencia); thin wrapper = overhead bajo por diseño |
| Robustez | **6** | 4 bugs latentes confirmados aún presentes |
| Seguridad | **7** | Thin wrappers sin superficie propia de riesgo detectada |
| Docs & ejemplos | **6.5** | Docstrings buenos en langchain/haystack/mem0; READMEs por-adapter existen |
| Observabilidad | **5** | Sin métricas/logging propios del adapter |
| Testabilidad | **7.5** | Tests presentes en los 9; langchain ~30 |
| Paridad entre módulos VantaDB | **7** | Constructor y convenciones consistentes en los 9 |
| Diferenciación | **6** | Nicho local-first real vs zep-server/cognee-KG, pero invisible sin publicación |
| **Global** | **6.3** | |

## Gap analysis priorizado

**Falta (P0/P1):**
1. Publicación PyPI de los 9 paquetes (P0 — multiplica el valor de todo el directorio).
2. Re-trazar MOD-46..50 al Backlog (P0 proceso — trabajo huérfano).
3. Fix bugs crewai/langchain/llamaindex (P1, baratos, ya especificados).

**Mejorable:** dedup openai/ollama; matriz CI de compat contra versiones actuales de frameworks (pins `langchain-core>=0.3`, `llama-index-core>=0.12`, `dspy>=2.6` nunca validados contra releases actuales); decidir futuro de letta.

**Optimizable:** `count_documents()` haystack O(n); heurística `_normalize_score` mem0.

**Apuestas estratégicas (>1 semana):**
- Adapter **LangGraph** (checkpointer + `BaseStore`) — donde vive la persistencia del ecosistema langchain en 2026.
- Backend de la clase **Memory unificada de CrewAI** en vez de tool.
- Reposicionar ollama/openai: lo idiomático ahí es embeddings-provider/OpenAI-compatible API, no wrappers tipo-langchain duplicados.

## Quick wins (<1 día) vs estratégicas

- **Quick wins:** fixes MOD-46..48 · borrar `categorize()` muerta · decisión+ejecución de publicación PyPI.
- **Estratégicas:** LangGraph integration · crewai Memory backend · consolidación ollama/openai.

---

## APÉNDICE OBLIGATORIO — Inventario de hallazgos H-NN

| ID | Categoría | Severidad | Esfuerzo | Título | file:line |
|----|-----------|-----------|----------|--------|-----------|
| H-01 | APLICAR | 🔴 | 🟡 | Publicar los 9 adapters en PyPI (= MKT-18f, ampliada de 5→9 paquetes); convención ecosistema confirma paquetes separados | `integrations/*/pyproject.toml` |
| H-02 | APLICAR | 🔴 | 🟢 | Re-trazar MOD-46..50 al Backlog (filas huérfanas: removidas sin completar ni archivar) + fix bugs crewai from_dict string-embedding (:217-218) y cursor str→usize (:164-176) | `integrations/crewai/vantadb_crewai/vectorstore.py:164-176,217-218` |
| H-03 | APLICAR | 🟡 | 🟢 | langchain add_documents: ids parciales filtrados → ValueError engañoso (=MOD-47, presente hoy) | `integrations/langchain/vantadb_langchain/vectorstore.py:470` |
| H-04 | APLICAR | 🟡 | 🟢 | llamaindex: attrs privados sin `PrivateAttr` + import faltante (=MOD-48, presente hoy; `PrivateAttr` importado pero verificar uso) | `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:42-50` |
| H-05 | MEJORAR | 🟡 | 🟡 | Deduplicar gemelos ollama/openai ~95% idénticos, async inconsistente entre sí (=MOD-49) | `integrations/{ollama,openai}/vantadb_*/vectorstore.py` |
| H-06 | ESTRATEGIA | 🔴 | 🔴 | Adapter LangGraph (checkpointer + `BaseStore`) — la persistencia moderna del ecosistema langchain; hoy solo cubrimos VectorStore | `integrations/langchain/` (nuevo) |
| H-07 | ESTRATEGIA | 🟡 | 🟡 | CrewAI: migrar de `BaseTool` a backend de la clase Memory unificada (patrón actual del framework) | `integrations/crewai/vantadb_crewai/vectorstore.py` |
| H-08 | DESCARTAR* | 🟢 | 🟢 | letta sin contrato de framework y Letta es plataforma-competidora → documentar experimental o retirar el adapter (*decisión HITL) | `integrations/letta/` |
| H-09 | MEJORAR | 🟡 | 🟡 | Matriz CI de compatibilidad: validar pins mínimos contra versiones actuales de los 9 frameworks (nunca verificado; source-driven incompleto del review previo) | `.github/workflows/`, `integrations/*/pyproject.toml` |
| H-10 | MEJORAR | 🟢 | 🟢 | Nits agrupados (=MOD-50): borrar `categorize()` DEPRECATED (~65 líneas), heurística `_normalize_score` mem0 frágil, haystack `count_documents()` O(n) sobre hasta 1M records | `crewai/:229-293`, `mem0/:45-55`, `haystack/:370-382` |
| H-11 | AGREGAR | 🟢 | 🟢 | Posicionamiento diferencial en READMEs: local-first embebido vs zep-servidor / cognee-KG / memoria nativa del framework | `integrations/*/README.md` |

*Trazabilidad:* H-02..H-05, H-10 corresponden 1:1 a MOD-46..50 (reporte fuente: `docs/reviews/modulos/integrations.md`). H-01 = MKT-18f ampliada. H-06, H-07, H-09, H-11 nuevos (investigación externa 2026-08-25).

Fuentes externas citadas: docs.crewai.com/concepts/memory · reference.langchain.com/python/langgraph.store.base/BaseStore · docs.langchain.com/oss/python/langgraph/persistence · developers.llamaindex.ai (Memory guide) · docs.haystack.deepset.ai/docs/creating-custom-document-stores · pypi.org/project/mem0ai · github.com/getzep/zep-python · github.com/topoteretes/cognee. UNVERIFIED: stars exactas (GitHub API 403), estado de `langchain.memory` en 1.x (página de migración no accesible), licencia exacta cognee.
