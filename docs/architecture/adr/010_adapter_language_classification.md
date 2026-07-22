---
title: "ADR 010: Adapter Language Classification and Directory Structure"
type: adr
status: active
tags: [vantadb, architecture, adr, adapters, python, rust]
last_reviewed: 2026-07-22
aliases: []
---

# ADR 010: Adapter Language Classification and Directory Structure

## Context

VantaDB has 10 adapters for external AI frameworks and embedding providers. Each adapter
existed in **two parallel implementations**: a Rust PyO3 crate at the project root
(e.g. `vantadb-crewai/`) and a pure Python wrapper in `integrations/` (e.g.
`integrations/crewai/`). This duplication made the project harder to maintain.

Research revealed:

1. **Framework adapters** (LangChain, LlamaIndex, Haystack, CrewAI, DSPy, Mem0, Letta)
   all expose integration APIs as **Python abstract base classes and protocols**
   (e.g. `class VectorStore(ABC)`, `class DocumentStore`). These cannot be implemented
   from Rust PyO3 — they require actual Python code that inherits from the framework's
   base classes.

2. **Provider adapters** (OpenAI, Ollama, LiteLLM) are **REST API clients** that make
   HTTP calls to external services. Any language can implement them. Rust is correct here
   because it avoids the Python runtime dependency and integrates directly with VantaDB's
   native engine.

3. 7 of 9 Python wrappers were **functionally broken** — they used keyword substring
   matching instead of real vector search. Only `langchain` and `llamaindex` had proper
   `vantadb_py.search_memory()` calls. Fixing these is tracked separately.

## Decision

### General rule

- **Framework adapters** (implement a framework's protocol) → **Python**, live in `integrations/`
- **Provider adapters** (call an external REST API) → **Rust**, live in `providers/`

### Actions taken

| Directory | Change | Reason |
|-----------|--------|--------|
| `integrations/langchain/` | **Keep** — Python ✅ | LangChain's `VectorStore` ABC is Python |
| `integrations/llamaindex/` | **Keep** — Python ✅ | LlamaIndex's `BasePydanticVectorStore` is Python |
| `integrations/haystack/` | **Keep** — Python ✅ | Haystack's `DocumentStore` protocol is Python |
| `integrations/crewai/` | **Keep** — Python ✅ | CrewAI's `StorageBackend` is Python |
| `integrations/dspy/` | **Keep** — Python ✅ | DSPy's `Retrieve` module is Python |
| `integrations/letta/` | **Keep** — Python ✅ | Letta's archival memory API is Python |
| `integrations/mem0/` | **Keep** — Python ✅ | Mem0's `VectorStoreBase` registry is Python |
| `integrations/openai/` | **Keep** — Python ✅ | Published PyPI package (users expect it) |
| `integrations/ollama/` | **Keep** — Python ✅ | Published PyPI package (users expect it) |
| `vantadb-crewai/` (root) | **Deleted** | Wrong language — can't implement CrewAI's Python protocol |
| `vantadb-dspy/` (root) | **Deleted** | Wrong language |
| `vantadb-haystack/` (root) | **Deleted** | Wrong language |
| `vantadb-langchain/` (root) | **Deleted** | Wrong language |
| `vantadb-letta/` (root) | **Deleted** | Wrong language |
| `vantadb-llamaindex/` (root) | **Deleted** | Wrong language |
| `vantadb-mem0/` (root) | **Deleted** | Wrong language (420 LOC, most complex — still wrong language) |
| `vantadb-openai/` (root) | **Moved to** `providers/openai/` | REST API client — correct as Rust |
| `vantadb-ollama/` (root) | **Moved to** `providers/ollama/` | REST API client — correct as Rust |
| `vantadb-litellm/` (root) | **Moved to** `providers/litellm/` | REST API via LiteLLM Proxy — correct as Rust |

### Resulting directory structure

```
integrations/             ← Python wrappers (framework adapters)
├── langchain/            → pip install vantadb-langchain
├── llamaindex/           → pip install vantadb-llamaindex
├── haystack/             → pip install vantadb-haystack
├── crewai/               → pip install vantadb-crewai
├── dspy/                 → pip install vantadb-dspy
├── letta/                → pip install vantadb-letta
├── mem0/                 → pip install vantadb-mem0
├── openai/               → pip install vantadb-openai
└── ollama/               → pip install vantadb-ollama

providers/                ← Rust crates (native, HTTP clients)
├── openai/               → cargo package: vantadb-openai (publish = false)
├── ollama/               → cargo package: vantadb-ollama (publish = false)
└── litellm/              → cargo package: vantadb-litellm (publish = false)
```

## Status

Active.

## Consequences

**Positive:**
- Single implementation per adapter (no duplication)
- Language matches the integration target (Python for framework protocols, Rust for HTTP)
- Workspace Cargo.toml is smaller (removed 7 members)
- Clear convention: `integrations/` = Python, `providers/` = Rust

**Negative:**
- 7 Python wrappers need vector search fixes (separate effort)
- No Rust crate for LiteLLM standalone use (but LiteLLM Proxy covers this via HTTP)

**Migration:**
- Old `vantadb-{name}` root paths will 404 in git history after cleanup
- CI workflows unchanged because they target `integrations/` via matrix
