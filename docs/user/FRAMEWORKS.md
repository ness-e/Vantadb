---
title: "VantaDB for agent frameworks — same memory code in dev and prod"
kind: howto
status: active
description: "Persistent memory adapters for LangGraph, CrewAI, DSPy, Haystack, LlamaIndex and more — same code in dev and prod, namespaces for multi-user isolation, migration guides from Mem0/Zep/Letta"
tags: [vantadb, frameworks]
---

# VantaDB for agent frameworks — same memory code in dev and prod

The frameworks you build on assume their memory disappears at the process
boundary. LangGraph hands you `InMemorySaver` for development and points
production at `PostgresSaver` — a database you now operate. CrewAI's native
memory settles on Chroma + SQLite without per-user isolation. Same shape of
problem: the memory your agents rely on is either ephemeral in dev or an
operational burden in prod.

VantaDB's answer is one embedded engine — checkpoints and hybrid search
in-process, no server — with adapters for the frameworks you already use.
**The application code does not change between dev and prod; only the
checkpointer (or store) changes.** Namespaces isolate threads, users and
projects without extra services.

## What plugs in where

| Framework | Package | Adapter surface | Adapter docs |
|-----------|---------|-----------------|--------------|
| LangGraph | `vantadb-langchain` | `VantaDBCheckpointer` (per-thread checkpoints), `VantaDBStore` (long-term store), `VantaDBVectorStore` | [integrations/langchain](../../integrations/langchain/README.md) |
| CrewAI | `vantadb-crewai` | `VantaDBMemoryBackend` (memory storage), `VantaDBTool` (memory tool) | [integrations/crewai](../../integrations/crewai/README.md) |
| DSPy | `vantadb-dspy` | `VantaDBRetriever` | [integrations/dspy](../../integrations/dspy/README.md) |
| Haystack | `vantadb-haystack` | `VantaDBDocumentStore` | [integrations/haystack](../../integrations/haystack/README.md) |
| LlamaIndex | `vantadb-llamaindex` | `VantaDBVectorStore` | [integrations/llamaindex](../../integrations/llamaindex/README.md) |
| Mem0 | `vantadb-mem0` | `VantaDBVectorStore` | [integrations/mem0](../../integrations/mem0/README.md) |
| Letta | `vantadb-letta` | `VantaDBVectorStore` | [integrations/letta](../../integrations/letta/README.md) |
| OpenAI SDK | `vantadb-openai` | `VantaDBOpenAI` (vector store) | [providers/openai](../../providers/openai/README.md) |
| Ollama | `vantadb-ollama` | `VantaDBOllama` (embedding vector store) | [providers/ollama](../../providers/ollama/README.md) |

Seven of these are standalone Python packages with their own test suites (the
OpenAI SDK and Ollama entries ship as the Rust providers — see their READMEs);
pins, fixtures and test commands are catalogued in
[integrations/README.md](../../integrations/README.md).

## Dev → prod: the checkpointer swap

```python
# dev — LangGraph's reference saver, state dies with the process
from langgraph.checkpoint.memory import InMemorySaver
graph = builder.compile(checkpointer=InMemorySaver())

# prod — same graph; embedded memory that survives restarts
from vantadb_langchain import VantaDBCheckpointer
graph = builder.compile(checkpointer=VantaDBCheckpointer(db_path="./agent_data"))
```

That swap is the whole change. The claim is CI-verified, not asserted: a
dedicated demo runs **five fresh processes** — same graph, `InMemorySaver` in
dev vs `VantaDBCheckpointer` in prod — and fails if the business code differs
between the two, if dev unexpectedly keeps state, if prod fails to resume
across processes, or if two threads in one database leak into each other.

- Run it: `python examples/langgraph_dev_to_prod/demo.py` — see
  [examples/langgraph_dev_to_prod](../../examples/langgraph_dev_to_prod/README.md)
- CI: [ci-frameworks-demo.yml](../../.github/workflows/ci-frameworks-demo.yml) (PR-blocking)

## Multi-user isolation

One process, one database file — and per-user separation without extra
services:

- The checkpointer keys every checkpoint by `(thread_id, checkpoint_ns,
  checkpoint_id)` and mirrors `thread_id` into record metadata, so thread
  queries never cross users.
- `VantaDBStore` maps namespace tuples (`("users", "alice", "prefs")`) to
  `/`-joined VantaDB namespaces.
- Vector stores take a `namespace="..."` argument per corpus or tenant.

The isolation claim is asserted mechanically in the demo above: a second
thread reading the same prod database sees nothing from the first.

## Migrating from Mem0 / Zep / Letta

The core crate ships importers that convert existing exports into the
[VantaDB memory interchange format](../api/MEMORY_INTERCHANGE_FORMAT.md) —
the converter reads the file only: no API keys, no calls to the source
service. Each guide lists exactly which fields survive the trip and which do
not:

- [Migrating from Mem0](tutorials/migrating-from-mem0.md)
- [Migrating from Zep](tutorials/migrating-from-zep.md)
- [Migrating from Letta](tutorials/migrating-from-letta.md)

## Install

**Alpha — not on PyPI yet.** The seven adapter names return 404 on the PyPI
JSON API (re-verified 2026-10-06; re-check with `curl -s -o /dev/null -w "%{http_code}"
https://pypi.org/pypi/vantadb-langchain/json`). They publish with the first
`adapters-v*` tag via
[release-adapters.yml](../../.github/workflows/release-adapters.yml) (the
`vantadb-openai`/`vantadb-ollama` names belong to the Rust providers —
`providers-v*` lane). Until then, install from a repo checkout:

```bash
python -m pip install ./integrations/langchain
```

Two pin notes for a source install:

- `vantadb-py>=0.6.1,<0.9.0` (bumped 2026-10-06 — the old
  `>=0.5.0,<0.7.0` excluded the live core and admitted 0.5.0, which lacks
  the `Client` API);
- `langchain-core>=0.3,<1` predates langchain-core 1.x, which LangGraph 1.x
  requires — resolving pins normally conflicts with a current LangGraph.

If you are on the current SDK and LangGraph majors, install the adapter with
`--no-deps` and bring the SDK + framework dependencies yourself (this is
exactly what the demo CI does):

```bash
python -m pip install vantadb-py
python -m pip install "langgraph>=1,<2"
python -m pip install --no-deps ./integrations/langchain
```

After the first adapters release, the install becomes the plain PyPI command:

```bash
pip install vantadb-langchain
```

## Adoption metric (installs per week)

Once published, the adoption metric for this track is **downloads per week per
adapter**, read from [pypistats](https://pypistats.org/):

- API: `https://pypistats.org/api/packages/vantadb-langchain/recent` returns
  `{"data": {"last_day": N, "last_week": N, "last_month": N}, ...}`
- CLI: `pip install pypistats && pypistats recent vantadb-langchain`

Baseline today: **0 for the adapters** — they are not published (404), so
there is nothing to measure here. Note what the 0 does not cover: the core
distribution `vantadb-py` *is* live on PyPI and has its own pypistats series —
read it separately for the engine's numbers, never as a stand-in for adapter
adoption. This page reports a number only when it is real; no estimates, no
projections.

## Honest limits

- **Not on PyPI yet (Alpha).** See install above; the switch to PyPI is a
  tracked follow-up after the first `adapters-v*` release.
- **Pin lag.** The `langchain-core` range still predates langchain-core 1.x
  / LangGraph 1.x (details in install); the core pin was refreshed
  2026-10-06.
- **Pre-1.0.** The adapters follow the core's versioning policy; expect API
  changes between minor releases ([VERSIONING.md](../api/VERSIONING.md)).
- **What the demo covers, and what it does not.** It verifies checkpoint
  persistence and thread isolation for LangGraph. It runs no LLM calls
  (deterministic by design) and says nothing about retrieval quality or model
  behavior.
- **Per-adapter reality.** Each adapter has its own README and test suite;
  read those before relying on a specific framework integration.

## Learn more

- [integrations/README.md](../../integrations/README.md) — the adapters, pins, test commands
- [examples/langgraph_dev_to_prod](../../examples/langgraph_dev_to_prod/README.md) — the dev→prod demo
- [COMPARISON.md](COMPARISON.md) — positioning against other stores (no competitor figures)
- [MEMORY_INTERCHANGE_FORMAT.md](../api/MEMORY_INTERCHANGE_FORMAT.md) — the import/export contract
