# Examples — VantaDB

> 14 ejemplos + notebook, todos vigentes y corriendo en CI (`.github/workflows/ci-examples-12.yml`, 3 OS, sin `continue-on-error`).
> Los ejemplos de TypeScript viven en [`vantadb-ts/examples/`](../vantadb-ts/examples/) (fuera de este árbol).
> Ver también: [showcase en la web](https://vantadb.vercel.app/showcase) · [playground interactivo](https://vantadb.vercel.app/playground) · [QUICKSTART](../docs/user/QUICKSTART.md).

## Rust (`cargo run --example <nombre>`)

| Ejemplo | Qué demuestra | Comando CI |
|---|---|---|
| `rust/basic.rs` | CRUD vectorial mínimo | `cargo run --example basic` |
| `rust/hybrid.rs` | Búsqueda BM25 + HNSW con RRF | `cargo run --example hybrid` |
| `rust/graphrag.rs` | Nodos, aristas y BFS sobre el grafo | `cargo run --example graphrag` |
| `rust/concurrent.rs` | Lecturas/escrituras concurrentes (`Arc` + threads) | `cargo run --example concurrent` |

## Python (requiere `vantadb-py` instalado)

| Ejemplo | Qué demuestra |
|---|---|
| `python/agent_memory.py` | Memoria de agente básica (guardar/recordar) |
| `python/autogen_memory.py` | Memoria compartida multi-agente AutoGen |
| `python/crewai_memory.py` | Backend de memoria para CrewAI |
| `python/dspy_retriever.py` | Retrieval para programas DSPy |
| `python/haystack_documentstore.py` | VantaDB como DocumentStore de Haystack (RAG) |
| `python/langchain_ollama_rag.py` | RAG con LangChain + Ollama local |
| `python/langgraph_checkpoint.py` | Checkpoints persistentes para agentes LangGraph |
| `python/mem0_integration.py` | Integración estilo mem0 |
| `python/semantic_kernel_memory.py` | Memoria para Semantic Kernel |
| `demo/demo.py` | Demo end-to-end ([README](demo/README.md)) |
| `colab/vantadb_quickstart.ipynb` | Quickstart en Colab, paso a paso |

Todos degradan con gracia sin servicios externos (mocks/fallbacks); solo necesitan `vantadb_py`.

## TypeScript (en `vantadb-ts/examples/`)

| Ejemplo | Qué demuestra |
|---|---|
| `langchain-rag.mjs` (+`langchain/`) | RAG con LangChain en Node |
| `llamaindex-rag.mjs` (+`llamaindex/`) | RAG con LlamaIndex |
| `vercel-ai-memory.mjs` (+`vercel-ai/`) | Memoria con Vercel AI SDK |

## Empezar (2 minutos)

```bash
# Rust
cargo run --example basic

# Python (con venv + vantadb-py instalado)
python examples/python/agent_memory.py
```
