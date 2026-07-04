---
title: "Building AI Agent Memory with VantaDB"
status: draft
tags: [vantadb, tutorial, guide, ai-agents, memory]
last_reviewed: 2026-07-03
aliases: []
---

# Building AI Agent Memory with VantaDB

VantaDB gives AI agents **persistent memory** — the ability to store, recall, and search across past conversations, decisions, and context. Unlike stateless LLM calls, an agent backed by VantaDB remembers what happened last session, retrieves relevant past exchanges by meaning, and can even build a knowledge graph over time.

In this tutorial you'll build a REPL agent that:

- Stores every message as a **node** with metadata (role, timestamp, session_id)
- Searches past conversations by **semantic similarity**
- Filters by metadata (session_id, date range)
- Runs **hybrid search** (vector + BM25 keyword) together

## Prerequisites

```bash
pip install vantadb openai
```

Set your `OPENAI_API_KEY` environment variable (or swap in any OpenAI-compatible provider).

## 1. Connect and define the schema

```python
import vantadb
import time
import uuid
from datetime import datetime, timedelta

db = vantadb.connect("agent-memory.db")

# VantaDB is schemaless, but we define a helper to create consistent nodes.
def create_message(session_id: str, role: str, content: str):
    return {
        "type": "message",
        "content": content,
        "role": role,
        "session_id": session_id,
        "timestamp": datetime.utcnow().isoformat(),
        "embedding_field": "content",  # tells VantaDB which field to embed
    }

# Create a collection (VantaDB calls them "spaces" — isolated namespaces).
space = db.space("chat_history")
```

**Key concept:** Every node carries both **data** (content, role) and **metadata** (session_id, timestamp). VantaDB uses the `embedding_field` hint to know which text to vectorize automatically.

## 2. Store messages

```python
session_id = str(uuid.uuid4())

messages = [
    ("user", "How do I deploy a FastAPI app on Railway?"),
    ("assistant", "You need a Dockerfile, a requirements.txt, and a `start` command in railway.toml."),
    ("user", "Can I use SQLite with it?"),
    ("assistant", "Yes, but Railway's filesystem is ephemeral — use PostgreSQL via the Railway dashboard instead."),
]

for role, content in messages:
    node = create_message(session_id, role, content)
    space.put(node)

print(f"Stored {len(messages)} messages in session {session_id[:8]}...")
```

`space.put()` inserts or upserts a node. VantaDB automatically generates an embedding for the `content` field when it detects `embedding_field`.

## 3. Search by semantic similarity

```python
query = "What should I use instead of SQLite on Railway?"
results = space.similar_to(query, top_k=5)

print("=== Semantic Search ===")
for r in results:
    print(f"  [{r.role}] ({r.score:.3f}) {r.content[:80]}")
```

Expected output — the top result is the assistant message about PostgreSQL:

```
=== Semantic Search ===
  [assistant] (0.89) Yes, but Railway's filesystem is ephemeral...
  [user] (0.72) Can I use SQLite with it?
  ...
```

## 4. Filter by metadata (session_id, date range)

```python
# Filter to a specific session
results = space.similar_to(
    "deployment advice",
    top_k=10,
    filter={"session_id": session_id},
)

# Filter by date range
last_hour = (datetime.utcnow() - timedelta(hours=1)).isoformat()
results = space.similar_to(
    "database options",
    filter={"timestamp": {"$gte": last_hour}},
)

# Combine filters
results = space.similar_to(
    "deployment",
    filter={
        "session_id": session_id,
        "role": "assistant",
        "timestamp": {"$gte": last_hour},
    },
)
```

**Metadata filtering** narrows the search space before vector comparison — this is faster and more accurate than post-filtering.

## 5. Hybrid search (vector + BM25)

Sometimes you need exact keyword matches alongside semantic ones:

```python
results = space.search(
    "ephemeral filesystem PostgreSQL",
    mode="hybrid",       # combines vector and BM25 scores
    alpha=0.5,           # balance: 0 = pure BM25, 1 = pure vector
    top_k=5,
)

print("=== Hybrid Search (alpha=0.5) ===")
for r in results:
    print(f"  [{r.role}] ({r.score:.3f}) {r.content[:80]}")
```

**`alpha=0.3`** → more keyword-heavy (good for code snippets, exact names).  
**`alpha=0.7`** → more semantic (good for paraphrased questions).

## 6. Full REPL agent with memory

Putting it all together — a REPL that remembers past conversations:

```python
import vantadb
import uuid
from datetime import datetime

db = vantadb.connect("agent-memory.db")
space = db.space("chat_history")
session_id = str(uuid.uuid4())

print(f"Agent session: {session_id[:8]}")
print("Type 'exit' to quit. Type 'recall <query>' to search memory.\n")

while True:
    user_input = input("You: ").strip()
    if user_input.lower() == "exit":
        break

    if user_input.lower().startswith("recall "):
        query = user_input[7:]
        results = space.similar_to(
            query,
            top_k=3,
            filter={"role": {"$in": ["user", "assistant"]}},
        )
        print("\n--- Relevant memories ---")
        for r in results:
            ts = r.timestamp[:19] if hasattr(r, "timestamp") else "?"
            print(f"  [{r.role}] ({ts}) {r.content[:100]}")
        print("-------------------------\n")
        continue

    # 1. Retrieve relevant context
    context_chunks = space.similar_to(user_input, top_k=2)
    context = "\n".join(
        f"{c.role}: {c.content[:200]}"
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
    space.put({
        "type": "message",
        "content": user_input,
        "role": "user",
        "session_id": session_id,
        "timestamp": datetime.utcnow().isoformat(),
        "embedding_field": "content",
    })
    space.put({
        "type": "message",
        "content": reply,
        "role": "assistant",
        "session_id": session_id,
        "timestamp": datetime.utcnow().isoformat(),
        "embedding_field": "content",
    })

    print(f"Agent: {reply}\n")
```

## How it works

```
User input ──▶ VantaDB semantic search ──▶ relevant past messages
                                      │
                                      ▼
                              Prompt (context + query)
                                      │
                                      ▼
                              LLM generates reply
                                      │
                                      ▼
                              Store user + assistant msg
                                      │
                                      ▼
                              Loop ──────────────────┐
```

## Next steps

- Add **graph edges** between related messages to build an agentic knowledge graph
- Use **MCP protocol** to expose agent memory to any MCP-compatible LLM host
- Run the same code in the browser via **WASM runtime**

---

**Key takeaway:** VantaDB turns "stateless LLM calls" into "stateful agents" with ~30 lines of Python. No separate vector database, no embedding pipeline to manage — just `put()` and `similar_to()`.
