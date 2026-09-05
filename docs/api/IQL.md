---
title: VantaDB IQL Reference
type: api
status: active
tags: [vantadb, api, iql]
last_reviewed: 2026-07-21
aliases: []
---

# VantaDB IQL Reference

> IQL (Interactive Query Language) is VantaDB's query language for CRUD operations, graph traversal, vector search, and hybrid queries. It is parsed by the Nom-based parser at `src/parser/mod.rs`.

## Statements

IQL supports six statement types:

| Statement | Description |
|-----------|-------------|
| `FROM` / `MATCH` | Query nodes with optional traversal, filters, ranking |
| `INSERT NODE#` | Create a new node |
| `UPDATE NODE#` | Modify existing node fields or vector |
| `DELETE NODE#` | Remove a node by ID |
| `RELATE NODE#` | Create a directed edge between two nodes |
| `INSERT MESSAGE` | Insert a message into a conversation thread |

---

## Query (`FROM` / `MATCH`)

### Syntax

```
FROM <entity> [SIGUE <min>..<max> "<label>" [TYPE <type>] [AS <alias>]] [<alias>]
  WHERE <condition> AND <condition> ...
  FETCH <field1>, <field2> ...
  RANK BY <field> [DESC]
  WITH TEMPERATURE <float>
  ROLE "<role>"
```

### Components

| Clause | Description |
|--------|-------------|
| `FROM <entity>` / `MATCH <entity>` | Entity type to search. `FROM` and `MATCH` are interchangeable. |
| `SIGUE <min>..<max> "<label>"` | Graph traversal: follow edges with the given label, between `min` and `max` hops. |
| `TYPE <type>` | Optional target type filter for traversal. |
| `AS <alias>` | Alias for traversed nodes. |
| `<alias>` | Target alias for result nodes (defaults to `"target"`). |
| `WHERE <cond> AND <cond>...` | Filter conditions (see [Conditions](#conditions)). |
| `FETCH <field1>, <field2>` | Projection: return only these fields. |
| `RANK BY <field> [DESC]` | Sort results by a field. |
| `WITH TEMPERATURE <float>` | Query temperature (0.0 = deterministic/exhaustive). |
| `ROLE "<role>"` | RBAC owner role filter. |

---

## Conditions

Conditions appear inside `WHERE` clauses, separated by `AND`.

### Relational Comparisons

| Operator | Meaning |
|----------|---------|
| `=` | Equals |
| `!=` | Not equals |
| `>` | Greater than |
| `>=` | Greater than or equal |
| `<` | Less than |
| `<=` | Less than or equal |

**Syntax:** `field <op> <value>`

Values are typically double-quoted strings. The parser also supports unquoted integers, floats, `true`, `false`, and `null`.

### Vector Similarity

```
<field> ~ "<text_query>", min = <score>
```

Performs semantic vector search. The `~` operator triggers embedding-based similarity matching with a minimum score threshold.

---

## Data Manipulation

### INSERT

```
INSERT NODE#<id> TYPE <type> { <field>: <value>, ... } [VECTOR [x, y, z, ...]]
```

Creates a new node with the given type, fields, and optional embedding vector.

### UPDATE

```
UPDATE NODE#<id> SET <field> = <value>, ...
UPDATE NODE#<id> SET VECTOR [x, y, z, ...]
```

Updates fields or the embedding vector of an existing node.

### DELETE

```
DELETE NODE#<id>
```

Removes a node by its numeric ID.

---

## Graph Operations

### RELATE

```
RELATE NODE#<src> --"<label>"--> NODE#<dst> [WEIGHT <n>]
```

Creates a directed edge from source to target with the given label and optional weight.

### INSERT MESSAGE

```
INSERT MESSAGE <SYSTEM|USER|ASSISTANT> "<content>" TO THREAD#<id>
```

Inserts a message into a conversation thread. Roles: `SYSTEM`, `USER`, `ASSISTANT`.

---

## Examples

### Basic query

```
FROM person WHERE name = "Alice"
```

### Query with graph traversal

```
FROM person SIGUE 1..3 "knows" TYPE place AS places p
  WHERE p.age > "25" AND p.bio ~ "engineer", min = 0.7
  FETCH name, bio
  RANK BY name
```

### Insert a node

```
INSERT NODE#42 TYPE person { name: "Bob", age: "30" } VECTOR [0.1, 0.2, 0.3]
```

### Update fields

```
UPDATE NODE#42 SET name = "Robert"
```

### Delete a node

```
DELETE NODE#42
```

### Relate two nodes

```
RELATE NODE#42 --"knows"--> NODE#7 WEIGHT 0.95
```

---

## Hybrid Search

IQL supports hybrid search combining BM25 lexical search with HNSW vector search. The `POST /api/v2/query` endpoint accepts IQL strings directly:

```bash
curl -X POST http://127.0.0.1:8080/api/v2/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <api-key>" \
  -d '{"query": "FROM memory WHERE text ~ \"neural network\", min = 0.75 FETCH text, score RANK BY score DESC"}'
```

Route selection (text-only, vector-only, hybrid) is automatic based on the request payload (see [`HTTP_API.md`](HTTP_API.md)).

## Error Handling

Parse errors return an IQL-specific error format:

```
IQL parse error at line <line>, col <col>: <message>
```

Execution errors during query processing return:

```
IQL error: <description>
```

## Related

- [`HTTP_API.md`](HTTP_API.md) — REST API endpoints that accept IQL queries
- [`src/parser/mod.rs`](../../src/parser/mod.rs) — IQL parser implementation (Nom-based)
- [`src/query.rs`](../../src/query.rs) — Query AST and logical plan types
- [`src/executor.rs`](../../src/executor.rs) — Hybrid IQL execution engine
