---
title: "LM Studio + VantaDB — Persistent Memory for Your Local GUI Stack"
kind: tutorial
status: active
description: "Pattern tutorial: apps consuming LM Studio's OpenAI-compatible local server + VantaDB as persistent, searchable memory (rescued from the 2026-06 GTM research, updated to 0.7.x API)."
tags: [vantadb, tutorial, lm-studio, local-llm, memory]
---

# LM Studio + VantaDB — Persistent Memory for Your Local GUI Stack

> Rescued from the 2026-06 GTM research (OLD docs, 2026-09-30) and updated to the current API (0.7.x).

LM Studio is the local-LLM "GUI": model browser, chat, and an **OpenAI-compatible API server** at `http://localhost:1234/v1`. It persists models — **not governed conversation memory**: every chat starts from scratch, and the GUI's saved files have no indexing or semantic search.

The integration point is **the app that consumes LM Studio's API**, not LM Studio itself:

## Pattern

```python
from openai import OpenAI
from vantadb import Client

# 1) LM Studio as the local LLM (OpenAI-compatible endpoint)
llm = OpenAI(base_url="http://localhost:1234/v1", api_key="not-needed")

# 2) VantaDB as persistent memory (embedded, local, no server)
db = Client("./personal-ai-memory")

# 3) Store a fact (namespace + key; same key overwrites)
db.put(
    "prefs",
    "editor",
    "The user prefers tabs over spaces.",
    metadata={"source": "user"},
)

# 4) Recall relevant memory for the prompt
#    (text_query = BM25 pool; pass a vector for the semantic pool — see 05-embedding-integrations)
hits = db.search("prefs", text_query="code formatting", top_k=3)
context = "\n".join(h.payload for h in hits)

resp = llm.chat.completions.create(
    model="local-model",
    messages=[
        {"role": "system", "content": f"Relevant memory:\n{context}"},
        {"role": "user", "content": "How should I format this file?"},
    ],
)
print(resp.choices[0].message.content)
```

## Notes

- **Embeddings:** bring your own vectors, or use `embed-local` (offline ONNX) — see [Embedding Integrations](05-embedding-integrations.md).
- **Namespaces** per user/project; **TTL** for session-scoped facts; `supersede` when a fact changes.
- **MCP clients** (Claude/Cursor/OpenCode) don't need this glue at all — see `docs/user/AI_IDES.md`.
- Ollama follows the same pattern (identical OpenAI-compatible surface) — see [Local RAG Pipeline](02-local-rag-pipeline.md).
