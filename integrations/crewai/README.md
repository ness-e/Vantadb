# VantaDB × CrewAI

CrewAI Tool adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Published on PyPI at 0.9.0** (verified live 2026-10-20). The PyPI
> install pulls the published release; to run against the current tree use
> the repo-checkout path.

```bash
# From a repo checkout (current tree)
cd integrations/crewai && pip install .
```

### Install from PyPI

```bash
pip install vantadb-crewai
```

## Quickstart

```python
from crewai import Agent, Task, Crew
from vantadb_crewai import VantaDBTool

rag_tool = VantaDBTool(
    name="Memory Search",
    description="Search stored documents in VantaDB",
    db_path="./my_data",
    namespace="docs",
)

agent = Agent(
    role="Assistant",
    goal="Answer questions using stored knowledge",
    tools=[rag_tool],
)
```

## API

- `VantaDBTool(embedding=None, name=..., description=..., *, db_path=..., namespace=..., top_k=...)` — CrewAI-compatible RAG tool
- `VantaDBMemoryBackend(db_path, namespace)` — `StorageBackend` for unified
  `Memory` (CrewAI ≥1.14): `save/search/delete/update/get_record/list_records/`
  `get_scope_info/list_scopes/list_categories/count/reset` + async
  `asave/asearch/adelete`

```python
from crewai import Memory
from vantadb_crewai import VantaDBMemoryBackend

memory = Memory(storage=VantaDBMemoryBackend(db_path="./my_data"))
memory.remember("We decided to use PostgreSQL.", scope="/project/decisions")
print(memory.recall("What database did we choose?"))
```

System fields live under reserved `__mem_*` metadata keys; your metadata is
untouched. Records without embeddings are invisible to vector `search`
(same as LanceDB).

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  where CrewAI's native memory covers only short-term session recall.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```
