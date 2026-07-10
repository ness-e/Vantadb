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

## Development

```bash
pip install -e .
```
