# VantaDB × CrewAI

CrewAI Tool adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

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

- `VantaDBTool(name, description, db_path, namespace)` — CrewAI-compatible RAG tool

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
