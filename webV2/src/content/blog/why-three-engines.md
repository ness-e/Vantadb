---
title: "Why Three Query Engines?"
description: "The philosophy behind VantaDB's multi-engine architecture."
pubDate: 2026-07-04
author: "VantaDB Team"
tags: ["engineering", "architecture"]
---

## One size doesn't fit all

For decades, the database industry has asked you to choose: relational or NoSQL? Vector or graph?

We think that's a false choice.

### The problem with polyglot persistence

The current solution is to run multiple databases: PostgreSQL for relational, Redis for cache, Elasticsearch for search, Neo4j for graphs. This means:

- **Operational complexity** — 4x the infrastructure to manage
- **Data fragmentation** — your data lives in silos
- **Query complexity** — application-level joins across systems
- **Consistency headaches** — distributed transactions across databases

### VantaDB's approach

One binary, three engines, unified query plane.

```sql
-- Hybrid query across all three engines
SELECT p.*, v.score, g.path
FROM products p
JOIN vector_search(
  model: 'text-embedding-3',
  query: 'wireless headphones',
  limit: 20
) v ON p.id = v.id
JOIN graph_traverse(
  start: p.category_id,
  relationship: 'subcategory_of',
  depth: 3
) g ON true
WHERE p.price < 200
ORDER BY v.score DESC;
```

### The engines

1. **Vector** — HNSW-based indexing for semantic search
2. **Relational** — Full SQL with MVCC and ACID guarantees
3. **Graph** — Property graph model with recursive traversals

### Why it works

Each engine is optimized for its workload, but they share the same storage layer and query planner. No data duplication. No sync lag. Just one database that adapts to your data.

Stay tuned for deep dives into each engine.
