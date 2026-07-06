---
title: "Hello, VantaDB World"
description: "Introducing VantaDB — the database that thinks with you."
pubDate: 2026-07-05
author: "VantaDB Team"
tags: ["announcement", "product"]
---

## Welcome to the future of data

VantaDB is a multi-engine database that combines vector, relational, and graph querying in a single binary.

No orchestrators. No sidecars. No microservices tax.

### Why VantaDB?

Most databases force you to choose a data model upfront. Vector or relational? Graph or document? With VantaDB, you get all three — and you can query across them in a single statement.

```sql
SELECT * FROM hybrid_search(
  query: 'find products similar to X',
  vector_index: 'product_embeddings',
  filter: 'price < 100 AND in_stock = true',
  graph_traverse: 'category_hierarchy'
);
```

### One Binary

```bash
curl -fsSL https://vantadb.com/install | sh
vantadb start
```

That's it. No Docker required. No external dependencies.

### Three Query Engines

- **Vector Engine** — HNSW-based ANN search with hybrid scoring
- **Relational Engine** — Full SQL with ACID transactions
- **Graph Engine** — Native property graph with Cypher-compatible queries

### Zero Ops

Built-in replication, automatic failover, and self-healing storage. VantaDB handles operations so you can focus on building.

Stay tuned for more updates.
