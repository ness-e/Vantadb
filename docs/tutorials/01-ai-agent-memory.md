---
title: "Building AI Agent Memory with VantaDB"
status: active
tags: [vantadb, tutorial, guide, ai-agents, memory]
last_reviewed: 2026-08-02
aliases: []
---

# Building AI Agent Memory with VantaDB

VantaDB gives AI agents **persistent memory** — the ability to store, recall, and search across past conversations, decisions, and context. Unlike stateless LLM calls, an agent backed by VantaDB remembers what happened last session, retrieves relevant past exchanges by meaning, and can even build a knowledge graph over time.

In this tutorial you'll build a REPL agent that:

- Stores every message as a **memory record** (payload + metadata) in a namespace
- Searches past conversations by **semantic similarity** with `search_memory()`
- Filters by metadata (`session_id`, `role`)
- Uses **hybrid search** (vector + BM25 keyword) via `text_query`

## Prerequisites

```bash
pip install vantadb-py openai
```

Set your `OPENAI_API_KEY` environment variable (or swap in any OpenAI-compatible provider).

## 1. Connect and define a helper

VantaDB is embedded — there is no server. Opening a database creates (or reopens) a directory on disk:

```python
from vantadb_py import VantaDB

db = VantaDB("agent-memory.db")
```

Records are stored under **namespaces** (the equivalent of a collection in other vector databases). Namespaces are created lazily on the first `put()`. Each record has a string `key`, a text `payload`, optional `metadata`, and an optional `vector`:

```python
import time
import uuid
from datetime import datetime

def embed(text: str) -> list[float]:
    """Embed text with OpenAI. Returns a 1536-dim vector."""
    import openai
    resp = openai.embeddings.create(
        model="text-embedding-3-small",
        input=text,
    )
    return resp.data[0].embedding

def store_message(db, session_id: str, role: str, content: str, seq: int):
    return db.put(
        "chat_history",                # namespace (≈ collection)
        f"{session_id}-{seq}",         # key
        content,                       # payload (what BM25 indexes)
        metadata={
            "role": role,
            "session_id": session_id,
            "timestamp": int(time.time()),
        },
        vector=embed(content),         # queryable embedding
    )
```

**Key concept:** every record carries both **data** (payload + metadata) and a **vector** you supply. VantaDB stores your payload for lexical (BM25) search and your vector for ANN search — you bring the embedding model, VantaDB brings the storage and the search.

## 2. Store messages

```python
# vanta-skip: requires OPENAI_API_KEY — embed() calls the OpenAI embeddings API
session_id = str(uuid.uuid4())

messages = [
    ("user", "How do I deploy a FastAPI app on Railway?"),
    ("assistant", "You need a Dockerfile, a requirements.txt, and a `start` command in railway.toml."),
    ("user", "Can I use SQLite with it?"),
    ("assistant", "Yes, but Railway's filesystem is ephemeral — use PostgreSQL via the Railway dashboard instead."),
]

records = []
for seq, (role, content) in enumerate(messages):
    records.append(store_message(db, session_id, role, content, seq))

print(f"Stored {len(records)} messages in session {session_id[:8]}...")
```

`put()` inserts or upserts a record: the same `key` overwrites. It returns a `VantaMemoryRecord` with `.key`, `.payload`, `.metadata`, `.node_id`, and timestamps — `node_id` is the numeric id used by the graph APIs.

## 3. Search by semantic similarity

Embed the query, then search:

```python
# vanta-skip: requires OPENAI_API_KEY — embed() calls the OpenAI embeddings API
query = "What should I use instead of SQLite on Railway?"
hits = db.search_memory(
    "chat_history",
    embed(query),
    top_k=5,
)

print("=== Semantic Search ===")
for h in hits:
    print(f"  [{h.metadata['role']}] ({h.score:.3f}) {h.payload[:80]}")
```

Expected output — the top result is the assistant message about PostgreSQL:

```
=== Semantic Search ===
  [assistant] (0.89) Yes, but Railway's filesystem is ephemeral...
  [user] (0.72) Can I use SQLite with it?
  ...
```

`search_memory()` returns `VantaSearchHit` objects exposing `.key`, `.payload`, `.metadata`, `.score`, and `.node_id`.

## 4. Filter by metadata

`search_memory()` accepts a `filters` dict. The Python SDK matches metadata values with **equality semantics**:

```python
# vanta-skip: requires OPENAI_API_KEY — embed() calls the OpenAI embeddings API
# Filter to a specific session
hits = db.search_memory(
    "chat_history",
    embed("deployment advice"),
    filters={"session_id": session_id},
    top_k=10,
)

# Combine filters
hits = db.search_memory(
    "chat_history",
    embed("deployment"),
    filters={"session_id": session_id, "role": "assistant"},
    top_k=10,
)
```

> **Range filters (e.g. `timestamp >= X`):** the Python SDK currently supports equality filters. The Rust SDK supports the full operator set (`$gt`, `$gte`, `$lt`, `$lte`, `$neq`). For Python, filter by equality first and apply range conditions client-side, or store a coarse bucket (e.g. an hour key) in metadata.

**Metadata filtering** narrows the search space before vector comparison — this is faster and more accurate than post-filtering.

## 5. Hybrid search (vector + BM25)

Sometimes you need exact keyword matches alongside semantic ones. Pass `text_query` and VantaDB fuses the BM25 lexical score with the vector score:

```python
# vanta-skip: requires OPENAI_API_KEY — embed() calls the OpenAI embeddings API
hits = db.search_memory(
    "chat_history",
    embed("ephemeral filesystem PostgreSQL"),
    text_query="ephemeral filesystem PostgreSQL",
    top_k=5,
)

print("=== Hybrid Search ===")
for h in hits:
    print(f"  [{h.metadata['role']}] ({h.score:.3f}) {h.payload[:80]}")
```

- **Vector-only:** omit `text_query` — best for paraphrased questions.
- **Hybrid:** add `text_query` — best for code snippets, product names, and exact terms.

## 6. Full REPL agent with memory

Putting it all together — a REPL that remembers past conversations:

```python
# vanta-skip: interactive REPL (input()) and requires OPENAI_API_KEY
import time
import uuid
from vantadb_py import VantaDB

db = VantaDB("agent-memory.db")
session_id = str(uuid.uuid4())
seq = 0

def embed(text: str) -> list[float]:
    import openai
    resp = openai.embeddings.create(model="text-embedding-3-small", input=text)
    return resp.data[0].embedding

def remember(role: str, content: str):
    global seq
    db.put(
        "chat_history",
        f"{session_id}-{seq}",
        content,
        metadata={"role": role, "session_id": session_id, "timestamp": int(time.time())},
        vector=embed(content),
    )
    seq += 1

print(f"Agent session: {session_id[:8]}")
print("Type 'exit' to quit. Type 'recall <query>' to search memory.\n")

while True:
    user_input = input("You: ").strip()
    if user_input.lower() == "exit":
        break

    if user_input.lower().startswith("recall "):
        query = user_input[7:]
        hits = db.search_memory(
            "chat_history",
            embed(query),
            filters={"session_id": session_id},
            top_k=3,
        )
        print("\n--- Relevant memories ---")
        for h in hits:
            print(f"  [{h.metadata['role']}] {h.payload[:100]}")
        print("-------------------------\n")
        continue

    # 1. Retrieve relevant context
    context_chunks = db.search_memory(
        "chat_history", embed(user_input), top_k=2
    )
    context = "\n".join(
        f"{c.metadata['role']}: {c.payload[:200]}"
        for c in context_chunks
    )

    # 2. Build prompt with memory context
    prompt = f"""Previous relevant context:
{context}

User: {user_input}
Assistant:"""

    # 3. Call your LLM (example with OpenAI)
    import openai
    response = openai.chat.completions.create(
        model="gpt-4o-mini",
        messages=[
            {"role": "system", "content": "You are a helpful assistant with memory of past conversations."},
            {"role": "user", "content": prompt},
        ],
    )
    reply = response.choices[0].message.content

    # 4. Store both sides
    remember("user", user_input)
    remember("assistant", reply)

    print(f"Agent: {reply}\n")
```

## How it works

```
User input ──▶ embed(query) ──▶ search_memory ──▶ relevant past messages
                                             │
                                             ▼
                                     Prompt (context + query)
                                             │
                                             ▼
                                     LLM generates reply
                                             │
                                             ▼
                             Store user + assistant msg (with vectors)
                                             │
                                             ▼
                                     Loop ──────────────────┐
```

## Next steps

- Add **graph edges** between related records with `add_edge()` (node IDs come from `record.node_id`) to build an agentic knowledge graph
- Use **MCP protocol** to expose agent memory to any MCP-compatible LLM host
- Run the same code in the browser via the **WASM runtime**

---

**Key takeaway:** VantaDB turns "stateless LLM calls" into "stateful agents" with ~40 lines of Python. No separate vector database, no server to run — just `put()` and `search_memory()`.
