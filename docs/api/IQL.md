---
title: VantaDB IQL Reference
kind: reference
status: active
description: The parser implements IQL version 3 (adds the AS OF valid-time clause and LIMIT/OFFSET pagination). The version is exposed as IQL_VERSION
tags: [vantadb, api, iql]
---

# VantaDB IQL Reference

> IQL (Interactive Query Language) is VantaDB's query language for CRUD operations, graph traversal, vector search, and hybrid queries. It is parsed by the Nom-based parser at `src/parser/mod.rs`.

## Language Version

The parser implements **IQL version 3**. The version is exposed as `IQL_VERSION`
(`vantadb::IQL_VERSION`, re-exported at the crate root); version-gated syntax
documents its minimum version and can be feature-detected from Rust with
`vantadb::parser::iql_supports(min_version)` — a clause is guaranteed to parse
when `iql_supports(<clause minimum>)` is true at the reported version.

| Version | Grammar |
|---------|---------|
| 1 | `FROM`/`MATCH`/`SELECT` (+ `JOIN`, subqueries) · DML (`INSERT`, `UPDATE`, `DELETE`, `RELATE`, `INSERT MESSAGE`) · `PROFILE` (minimum: `IQL_VERSION_MIN_PROFILE`) · operators `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `~` |
| 2 | Adds the optional `AS OF <unix-ms>` valid-time clause on top-level `FROM`/`MATCH`/`SELECT` (minimum: `IQL_VERSION_MIN_AS_OF`). Version 1 statements keep parsing unchanged |
| 3 | Adds the optional `LIMIT <n>` / `OFFSET <m>` pagination clauses on top-level `FROM`/`MATCH`/`SELECT` (minimum: `IQL_VERSION_MIN_PAGINATION`). Versions 1–2 statements keep parsing unchanged |

A clause documented in this file is guaranteed to parse at the reported version.
Wire consumers (MCP `query_iql`, HTTP `/api/v2/query`, bindings `query()`)
feature-detect by statement shape: `AS OF` and `LIMIT`/`OFFSET` are opt-in, so a
version-3 parser still accepts every version-1/2 statement — new syntax fails
loudly with a parse error on older parsers instead of being silently ignored.

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
FROM <entity> [SIGUE <min>..<max> "<label>" [TYPE <type>] [AS <alias>]] [<alias>] [AS OF <unix-ms>]
  WHERE <condition> AND <condition> ...
  FETCH <field1>, <field2> ...
  RANK BY <field> [DESC]
  WITH TEMPERATURE <float>
  ROLE "<role>"
  PROFILE keyword|vector|hybrid [rrf_k <n>] [candidate_k <n>]
  LIMIT <n> [OFFSET <m>]
  AS OF <unix-ms>
```

### Components

| Clause | Description |
|--------|-------------|
| `FROM <entity>` / `MATCH <entity>` | Entity type to search. `FROM` and `MATCH` are interchangeable. |
| `SIGUE <min>..<max> "<label>"` | Graph traversal: follow edges with the given label, between `min` and `max` hops. |
| `TYPE <type>` | Optional target type filter for traversal. |
| `AS <alias>` | Alias for traversed nodes. |
| `<alias>` | Target alias for result nodes (defaults to `"target"`). |
| `AS OF <unix-ms>` | Valid-time point (SCH-03, ADR-0046 §D3, minimum `IQL_VERSION_MIN_AS_OF` = 2): keep only records valid at that instant (`valid_at_ms <= T < invalid_at_ms`). Accepted after the table spec, before pagination, and at the end of the statement; repeating the clause is a parse error (`AS OF specified more than once`). Graph nodes without validity metadata are never excluded by it. |
| `WHERE <cond> AND <cond>...` | Filter conditions (see [Conditions](#conditions)). |
| `FETCH <field1>, <field2>` | Projection: return only these fields. |
| `RANK BY <field> [DESC]` | Sort results by a field (applied before `LIMIT`/`OFFSET`). |
| `WITH TEMPERATURE <float>` | Query temperature (0.0 = deterministic/exhaustive). |
| `ROLE "<role>"` | RBAC owner role filter. |
| `PROFILE keyword\|vector\|hybrid [rrf_k <n>] [candidate_k <n>]` | Search profile: fusion mode + RRF k + candidate budget (MEM-01). `rrf_k` / `candidate_k` default to the core constants when omitted. |
| `LIMIT <n>` | Result cap (minimum `IQL_VERSION_MIN_PAGINATION` = 3): keep at most `n` rows after filters/ranking. `LIMIT 0` returns an empty result. See [Pagination](#pagination-limit--offset). |
| `OFFSET <m>` | Skip the first `m` rows of the result (minimum `IQL_VERSION_MIN_PAGINATION` = 3). Must follow `LIMIT` when both are present — the reversed order (`OFFSET` then `LIMIT`) is a parse error. See [Pagination](#pagination-limit--offset). |

> **`AS OF` is valid time, not transaction time.** It answers "what did the
> record say was true at T" (the valid-time axis). Transaction-time travel is
> per key (`get_version`/`versions`, bounded retention) and is not available as
> a cross-key `AS OF` in this release (v1.0).

---

## Select (`SELECT`)

```
SELECT <field>, ... | * FROM <entity> [<alias>] [AS OF <unix-ms>]
  [JOIN <entity> <alias> ON <left_field> = <right_field>] ...
  [WHERE <item> AND <item> ...]
  [WITH TEMPERATURE <float>]
  [LIMIT <n> [OFFSET <m>]]
  [AS OF <unix-ms>]
```

| Clause | Description |
|--------|-------------|
| `SELECT <field>, ...` / `SELECT *` | Projection: named fields, or `*` for all fields (empty projection = no narrowing). |
| `FROM <entity> [<alias>]` | Same scan surface as `FROM`/`MATCH`; alias defaults to `target`. |
| `AS OF <unix-ms>` | Valid-time point (SCH-03): same semantics as in `FROM`/`MATCH`. A subquery containing `AS OF` is a parse error — the clause is top-level only (no silent scoping). |
| `JOIN <entity> <alias> ON <left> = <right>` | Chained joins; `ON` fields are alias-qualified (`p.addr_id = a.id`). |
| `WHERE` | Mixes regular conditions and scalar subqueries: `<field> <op> (SELECT ...)`. |
| `WITH TEMPERATURE <float>` | Query temperature (0.0 = deterministic/exhaustive). |
| `LIMIT <n> [OFFSET <m>]` | Pagination: same semantics as in `FROM`/`MATCH` (see [Pagination](#pagination-limit--offset)). |

Example:

```
SELECT name, age FROM Person p
  JOIN Address a ON p.addr_id = a.id
  WHERE a.city == "Caracas"
  LIMIT 20 OFFSET 40
```

---

## Pagination (`LIMIT` / `OFFSET`)

> Added in IQL version 3 (WIRE-12). Both clauses are optional; canonical order is
> `LIMIT <n>` then `OFFSET <m>` (SQL order). The reversed order or a duplicated
> pagination keyword is a parse error — never a silent drop.

```
FROM person WHERE age > 18 RANK BY age LIMIT 10 OFFSET 20
SELECT * FROM kb LIMIT 5
FROM person OFFSET 100
```

Semantics:

- `LIMIT <n>` caps the result at `n` rows **after** filters, traversal,
  ranking and — for hybrid searches — RRF fusion; `LIMIT 0` is a valid empty
  window. `OFFSET <m>` skips the first `m` rows of that result; combined they
  mean "skip `m`, take `n`".
- `OFFSET` without `LIMIT` skips and returns the rest; `OFFSET 0` is a no-op.
- Both are strict: `LIMIT`/`OFFSET` without a numeric count is a parse error
  (same rule as `AS OF`).
- **Vector searches:** the cap applies to the stream produced by the search
  operators. The vector candidate window of the IQL execution path is
  currently fixed at 5 (`PhysicalVectorSearch`), so a vector-only query yields
  at most 5 rows regardless of a larger `LIMIT` (tracked as FIND-313).
- **Interaction with `AS OF` and `ROLE`:** the valid-time filter and RBAC role
  pruning run after plan execution, so a `LIMIT` page can shrink below `n` when
  those filters drop rows — the short page is the end of the walk (same
  convention as the temporal page-completeness guarantee below).

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

## Valid-Time Queries (`AS OF`)

> Added in IQL version 2 (SCH-03, ADR-0046 §D3). The equivalent request params
> are `as_of_ms` / `valid_window` on `search`/`list` (see
> [`EMBEDDED_SDK.md`](EMBEDDED_SDK.md) and the SDK pages).

```sql
-- What did the record say was true at a given instant?
FROM mmd_s1_history AS OF 1788134400000 WHERE text ~ "deploy plan"

-- SELECT spelling; the clause may also close the statement
SELECT * FROM kb WHERE ts > 0 AS OF 1788134400000
```

Semantics (ADR-0046 §D3): the validity interval is **closed-open**
`[valid_at_ms, invalid_at_ms)` — the start instant matches, the end instant
does not; `invalid_at_ms = None` means open-ended. Records without validity
metadata (plain graph nodes) are never excluded by `AS OF`.

**Page-completeness guarantee with temporal filters:** a page that returns
fewer records than `limit`/`top_k` is the last page — clients stop when
`next_cursor` is absent. Temporal filters drop records during assembly, so a
short page may surface sooner than the raw data end; that short page is the
end of the walk for that filter (search grows its fetch window to fill
`top_k`; `list` returns the short page as final).

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
    "search_profile": null,
    "as_of_ms": null,
    "limit": null,
    "offset": null
  }
}
```

`LIMIT`/`OFFSET` serialize as `limit`/`offset` (`null` when absent), e.g.
`FROM Doc LIMIT 5 OFFSET 2` → `"limit": 5, "offset": 2`.

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

`LogicalOperator::Offset { skip }` (WIRE-12) is the second shipped extension —
same pattern (variant + `src/physical_plan/offset.rs` + one `register` line,
zero planner/executor match edits), and it has a real IQL producer: `OFFSET`
composes with the built-in `Limit` as skip-then-take (the emitted `Limit`
window is widened by the offset so the post-chain `Offset` trims the front).

## Related

- [`HTTP_API.md`](HTTP_API.md) — REST API endpoints that accept IQL queries
- [`src/parser/mod.rs`](../../src/parser/mod.rs) — IQL parser implementation (Nom-based)
- [`src/query.rs`](../../src/query.rs) — Query AST and logical plan types
- [`src/executor.rs`](../../src/executor.rs) — Hybrid IQL execution engine
