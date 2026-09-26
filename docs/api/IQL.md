---
title: VantaDB IQL Reference
type: api
status: active
tags: [vantadb, api, iql]
last_reviewed: 2026-09-25
aliases: []
---

# VantaDB IQL Reference

> IQL (Interactive Query Language) is VantaDB's query language for CRUD operations, graph traversal, vector search, and hybrid queries. It is parsed by the Nom-based parser at `src/parser/mod.rs`.

## Language Version

The parser implements **IQL version 1**. The version is exposed as `IQL_VERSION`
(`vantadb::IQL_VERSION`, re-exported at the crate root); version-gated syntax
documents its minimum version and can be feature-detected from Rust with
`vantadb::parser::iql_supports(min_version)`.

| Version | Grammar |
|---------|---------|
| 1 | `FROM`/`MATCH`/`SELECT` (+ `JOIN`, subqueries) · DML (`INSERT`, `UPDATE`, `DELETE`, `RELATE`, `INSERT MESSAGE`) · `PROFILE` (minimum: `IQL_VERSION_MIN_PROFILE`) · operators `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `~` |

A clause documented in this file is guaranteed to parse at the reported version.

## Statements

IQL supports seven statement types:

| Statement | Description |
|-----------|-------------|
| `FROM` / `MATCH` | Query nodes with optional traversal, filters, ranking |
| `SELECT` | Query with projections, `JOIN`s and scalar subqueries |
| `INSERT NODE#` | Create a new node |
| `UPDATE NODE#` | Modify existing node fields or vector |
| `DELETE NODE#` | Remove a node by ID |
| `RELATE NODE#` | Create a directed edge between two nodes |
| `INSERT MESSAGE` | Insert a message into a conversation thread |

**One read grammar, three spellings.** `FROM <entity>` and `MATCH <entity>` are
interchangeable, and `SELECT` is the same read surface with projections, `JOIN`
and subqueries. All three default the result alias to `target` and share the
same condition grammar.

## Lexical Rules

- **Keywords are case-sensitive UPPERCASE** (`FROM`, `WHERE`, `SIGUE`, …):
  lowercase spelling is a parse error, not an accepted alias. Lowercase
  grammar literals are the exceptions: `min`, `rrf_k`, `candidate_k`, and the
  `PROFILE` modes (`keyword` / `vector` / `hybrid`).
- **Only double quotes delimit strings** (`"hello"`; escapes `\"`, `\\`, `\n`,
  `\r`, `\t`). Single-quoted strings (`'hello'`) are **not supported** and fail
  with a parse error — use double quotes. (`RELATE` labels use the structural
  `--"<label>"-->` form.)
- **Numeric literals:** unquoted integers parse as `Int` (exact across the
  whole `i64` range — no precision loss above 2^53); decimal/exponent literals
  parse as `Float`; integer literals beyond `i64` fall back to `Float`
  (precision may be lost). Comparisons are **type-strict**: an `Int` literal
  matches only an `Int` field and a `Float` literal only a `Float` field —
  mixed comparisons (e.g. `x = 28` against a stored `Float(28.0)`, including
  numeric data written by pre-v1 IQL) evaluate to `false` without error, no
  coercion is applied. Write the literal in the stored type: `28` for `Int`,
  `28.0` for `Float`.

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
  PROFILE keyword|vector|hybrid [rrf_k <n>] [candidate_k <n>]
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
| `PROFILE keyword\|vector\|hybrid [rrf_k <n>] [candidate_k <n>]` | Search profile: fusion mode + RRF k + candidate budget (MEM-01). `rrf_k` / `candidate_k` default to the core constants when omitted. |

---

## Select (`SELECT`)

```
SELECT <field>, ... | * FROM <entity> [<alias>]
  [JOIN <entity> <alias> ON <left_field> = <right_field>] ...
  [WHERE <item> AND <item> ...]
  [WITH TEMPERATURE <float>]
```

| Clause | Description |
|--------|-------------|
| `SELECT <field>, ...` / `SELECT *` | Projection: named fields, or `*` for all fields (empty projection = no narrowing). |
| `FROM <entity> [<alias>]` | Same scan surface as `FROM`/`MATCH`; alias defaults to `target`. |
| `JOIN <entity> <alias> ON <left> = <right>` | Chained joins; `ON` fields are alias-qualified (`p.addr_id = a.id`). |
| `WHERE` | Mixes regular conditions and scalar subqueries: `<field> <op> (SELECT ...)`. |
| `WITH TEMPERATURE <float>` | Query temperature (0.0 = deterministic/exhaustive). |

Example:

```
SELECT name, age FROM Person p
  JOIN Address a ON p.addr_id = a.id
  WHERE a.city == "Caracas"
```

---

## Conditions

Conditions appear inside `WHERE` clauses, separated by `AND`.

### Relational Comparisons

| Operator | Meaning |
|----------|---------|
| `=` | Equals |
| `==` | Equals (alias of `=`) |
| `!=` | Not equals |
| `>` | Greater than |
| `>=` | Greater than or equal |
| `<` | Less than |
| `<=` | Less than or equal |

**Syntax:** `field <op> <value>`

Values are strings, integers, floats, `true`, `false`, or `null` (see
[Lexical Rules](#lexical-rules)).

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

## AST JSON

The core AST types (`Statement`, `Query`, `SelectStatement`, …) derive
`serde::Serialize` and expose serde's default representation — externally
tagged enums with `snake_case` fields, the same convention as the SDK
`QueryResult`:

- enums carry a variant tag: `{"Query": {...}}`, `{"Insert": {...}}`,
  `{"Select": {...}}`, …;
- tuple variants become arrays: `Condition::Relational(field, op, value)` →
  `{"Relational": ["edad", "Eq", {"Int": 28}]}`;
- `Int` literals serialize as `{"Int": 28}` (i64, exact), floats as
  `{"Float": 1.5}`.

Rust consumers serialize the `Statement` returned by
`vantadb::parser::parse_statement` with `serde_json`. The shape is pinned by
`src/parser/mod.rs::tests::test_ast_json_projection_shape`.

Example — `FROM Person p WHERE edad == 28 FETCH name`:

```json
{
  "Query": {
    "from_entity": "Person",
    "traversal": null,
    "target_alias": "p",
    "where_clause": [
      { "Relational": ["edad", "Eq", { "Int": 28 }] }
    ],
    "fetch": ["name"],
    "rank_by": null,
    "temperature": null,
    "owner_role": null,
    "search_profile": null
  }
}
```

## Error Handling

Parse errors return an IQL-specific error format:

```
IQL parse error at line <line>, col <col>: <message>
```

Execution errors during query processing return:

```
IQL error: <description>
```

## Operator Extensibility (C2S6)

Logical operators dispatch by name through `src/operator_registry.rs`
(`OperatorRegistry`: `OperatorCompiler` + `OperatorCostModel` traits).
`LogicalOperator::Dedup { field }` is the exemplar: it compiles, costs
(passthrough, same convention as `Sort`/`Project`), and executes
(`PhysicalDedup` Volcano wrapper in `src/physical_plan/dedup.rs`) with zero
`match` edits in `planner`/`executor` — the planner catch-all routes unknown
operators to the registry, and unregistered names fail as
`Schema("SCHEMA_UNKNOWN_OPERATOR: ...")` instead of being silently dropped.
`Dedup` has no IQL producer yet (test-constructed plans only). New operators:
add the enum variant + physical file + one `register` line.

## Related

- [`HTTP_API.md`](HTTP_API.md) — REST API endpoints that accept IQL queries
- [`src/parser/mod.rs`](../../src/parser/mod.rs) — IQL parser implementation (Nom-based)
- [`src/query.rs`](../../src/query.rs) — Query AST and logical plan types
- [`src/executor.rs`](../../src/executor.rs) — Hybrid IQL execution engine
