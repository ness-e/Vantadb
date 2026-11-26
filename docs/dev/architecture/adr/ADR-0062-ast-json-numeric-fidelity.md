---
title: "ADR-0062: Numeric fidelity of the AST JSON — u128 ids travel as decimal strings"
kind: adr
status: accepted
description: "The IQL AST JSON emits its six u128 id fields as decimal strings; i64 Int literals stay numbers, limit pinned by test. Scoped to the token channel while parse() is still unreleased"
tags: [vantadb, architecture, adr, api, iql, json, precision, bindings]
created: "2026-11-24"
owner: vanta-arch
---

# ADR-0062: Numeric fidelity of the AST JSON — `u128` ids travel as decimal strings

## Status

Accepted. Design decision for FIND-723 (plan §6.b). This record decides the
shape; the serde change, the doc updates and the binding edits are a follow-up
implementation task. It supersedes nothing.

## Context and problem statement

`Statement` and its variants derive `serde::Serialize` — only `Serialize`, no
`Deserialize` (`src/query.rs:17,36,49,60,67,80`). That projection is public in
two senses:

- it is documented as the canonical AST JSON (`docs/api/IQL.md` §AST JSON) and
  pinned by `src/parser/mod.rs::tests::test_ast_json_projection_shape`
  (lines 1559–1576);
- it is exposed to users by `parse()` in `vantadb-python`, `vantadb-wasm` and
  `vantadb-ts` (FIND-156, commit `20c164970`).

The projection emits **plain JSON numbers** for two families of values:

| Family | Rust type | Where in the AST |
|---|---|---|
| Node / thread ids | `u128` | `InsertStatement.node_id`, `UpdateStatement.node_id`, `DeleteStatement.node_id`, `RelateStatement.source_id`, `RelateStatement.target_id`, `InsertMessageStatement.thread_id` — six fields, `src/query.rs:39,52,63,70,72,87` |
| Integer literals | `i64` (`FieldValue::Int`) | `Condition::Relational(String, RelOp, FieldValue)` (`src/query.rs:148`) and the `fields` maps of `Insert`/`Update` |

JSON has no integer type — only a number grammar — and RFC 8259 §6 bounds the
*interoperable* range to `[-(2^53)+1, (2^53)-1]`. Above that, decoders that
implement numbers as IEEE-754 binary64 are silently lossy.

That is exactly what happens on the JS side: `vantadb-wasm/src/lib.rs:1865-1874`
serializes with `serde_json::to_string` and then decodes with
`js_sys::JSON::parse`, so **every** number lands as a double. Measured in the
FIND-156 review (Critical-1): `2^53 + 1` decodes as `2^53`, and
`DELETE NODE#123456789012345678901234` comes back as `1.2345678901234569e+23`.
Python is unaffected (`json.loads` keeps arbitrary precision, documented at
`docs/api/PYTHON_SDK.md:960-962`).

**What is guarded today, exactly.** The `Int` half of the limit is documented in
four places and pinned by a test that will fail when it is fixed
(`vantadb-ts/src/__tests__/find156.test.ts:103-115`,
`pins the documented >2^53 precision limitation` — it asserts only an `Int`
literal: `9007199254740993` → `9007199254740992`). The **id half is pinned
nowhere**: `rg -n 'node_id|thread_id|source_id' vantadb-ts/src/__tests__/find156.test.ts`
returns **zero hits**, and the Python AST test only asserts that the
`InsertMessage` key exists (`vantadb-python/tests/test_find156_ast_json.py:88-91`).
So the id corruption has been measured and written down, but no test would catch
it moving — which is how it will stay caught only by luck.

The problem is not cosmetic. A node id is an **opaque token** that must be
bit-exact to be usable at all: a truncated id does not throw, it addresses the
wrong node or no node. And `vantadb-ts/src/vantadb.ts:160-168` already **throws**
on an unsafe-integer id at the *input* boundary — the output surface should not
silently accept what the input surface rejects.

There is also a standing rule this surface violates:
`docs/api/VERSIONING.md:54-55` — *"Identifiers: `u128` crosses every JSON/FFI
wire as a decimal string (no exceptions)"*. The AST is the exception today.

### Why now and not at 1.0

`parse()` **has never been released**. `git tag --contains 20c164970` is empty;
the newest tag is `v0.9.0` (2026-10-20) and `[workspace.package] version` is
still `0.9.0` (`Cargo.toml:819-820`). The next minor — 0.10.0, days away — is
the first release that will ever contain this surface. There are zero external
consumers to migrate, nothing to deprecate, and nothing in the tree breaks: no
test pins an id numerically. Deferring to 1.0 would freeze the shape
permanently — Hyrum's Law operates on *observable* behaviour, and a published
numeric id channel is observable.

### Prior art (external, verified)

- **ProtoJSON** emits `int64`/`uint64` as **decimal strings**
  ([protobuf.dev/programming-guides/json](https://protobuf.dev/programming-guides/json/)):
  "the json.org spec does not speak to the intended precision limits of numbers.
  Many implementations … interpret all numbers as binary64 … and are silently
  lossy if a number is an integer larger than 2\*\*53".
- **RFC 8259 §6**: "numbers that are integers and are in the range
  [-(2\*\*53)+1, (2\*\*53)-1] are interoperable in the sense that
  implementations will agree exactly on their numeric values"
  ([rfc-editor.org/rfc/rfc8259.html](https://www.rfc-editor.org/rfc/rfc8259.html)).
- **ECMAScript / MDN**: `Number.MAX_SAFE_INTEGER === 9007199254740991` (2\*\*53−1)
  ([MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER)).
- **Discord**: "Because Snowflake IDs are up to 64 bits in size (e.g. a uint64),
  they are always returned as strings in the HTTP API to prevent integer
  overflows in some languages"
  ([docs.discord.com/developers/reference](https://docs.discord.com/developers/reference)).
  Twitter/X solved it the same way (`id_str` next to numeric `id`).

The counter-pattern: `serde_json`'s `arbitrary_precision` feature is a
**producer-side** facility. The loss here happens in `js_sys::JSON::parse` on the
consumer side, so enabling it changes nothing observable.

## Considered options

### O1 — `u128` as decimal string, `i64` stays a JSON number (chosen)

- **Pros:** the lossy token channel closes; zero new public symbols — the
  attribute reuses `u128_serde`, and the identical path is already used in-tree
  (`src/graphrag/pipeline.rs:26,35,37`), so it is proven to compile; restores the
  `VERSIONING.md:54-55` rule; matches ProtoJSON/Discord; the surface is
  unreleased, so no migration cost.
- **Cons:** the AST stops being uniform — ids are strings, literals are numbers
  — and `Int` above 2^53 still loses precision in JS (addressed by R2, not by
  O1).

### O2 — keep numbers, document the limit (status quo)

- **Pros:** zero work; the `Int` half is already documented and pinned.
- **Cons:** a documented known-wrong answer on a surface with no users gets
  copied: the first consumer reads the doc, keeps large ids as strings, and the
  next integration does not. The 0.x window makes that trade gratuitous — the
  fix costs six attributes and no migration.
- **Rejected.**

### O3 — everything as string, uniformly (ids **and** `i64` `Int`)

- **Pros:** one rule, no exceptions; strictly ProtoJSON-like.
- **Cons:** integer literals are carried by `crate::node::FieldValue`, which
  derives `Serialize + Deserialize` (`src/node/field.rs:6`) and is the field map
  of `UnifiedNode` — the type `src/storage/` persists (postcard in
  `src/storage/`, `src/wal.rs`; `src/storage/engine/insert.rs:8,1189-1192`). So
  "make `Int` a string" is not an AST edit: it is an on-disk-format question
  (ADR-0060 territory) plus a simultaneous shape change of every `QueryResult`
  metadata payload in four bindings. Scoping it to the AST alone needs an
  AST-specific value type, i.e. a new `pub` type and a golden diff.
- **Rejected for this ADR:** right direction, wrong blast radius — see R2.

### O4 — emit both (`node_id` string + `node_id_numeric` deprecated)

- **Pros:** no consumer breaks.
- **Cons:** buys nothing when there is no consumer to keep compatible; two
  representations of one fact invite drift; and `*_numeric` would carry the
  known-wrong value, so reading the deprecated field silently corrupts.
- **Rejected:** the Twitter pattern pays off only once consumers already break.

### O5 — change the id type so it fits (`u64`/`i64`)

- **Pros:** removes the >2^53 problem from the AST.
- **Cons:** breaks the engine model — node identity is `u128`
  (`parse_u128_id`, accumulators, thread store all take `u128`) — and does not
  solve it: `u64::MAX` ≈ 1.8e19 is far past 2^53, so JS still loses digits. It
  shrinks the window instead of closing it.
- **Rejected.**

### O6 — let consumers opt into a bigint-aware parser (`json-bigint`, …)

- **Pros:** no shape change; works today.
- **Cons:** pushes a correctness invariant of *our* contract onto every consumer
  and every transitive tool (log shippers, proxies, `structuredClone`). The wire
  must be safe to read with the platform's stock parser.
- **Rejected** as the primary mechanism; it remains a legitimate consumer-side
  workaround for `Int` literals.

### O7 — add a raw-text accessor (`parse_ast_json() -> str`) and skip serde

- **Pros:** the only **zero-cost** exactness fix for `Int` in JS: the caller
  receives the core's JSON *text* and decodes it with whatever it wants
  (`BigInt`, `json-bigint`). Bypasses `FieldValue` entirely, so no on-disk
  question, no shape change, no golden diff. ≈5 lines per binding.
- **Cons:** it does not fix the *object* channel — the default `parse()` keeps
  returning numbers — so it makes exactness a per-call choice instead of a
  property of the wire, and it adds a third method to keep in parity across three
  bindings.
- **Rejected for this ADR:** it is an *additive API decision* for a separate
  ADR, and it leaves the id channel broken if used alone. **Adopted as the
  mitigation for R2**, in preference to waiting on the `FieldValue` format
  decision — the two are independent, and saying otherwise would overstate the
  coupling.

### O8 — a newtype id (`#[serde(transparent)] pub struct NodeId(u128)`)

- **Pros:** the structural answer; one type change covers all six fields, so no
  per-field attribute can be forgotten (see R1), and it documents the unit at the
  type level.
- **Cons:** it changes public field types, so `tests/api/public-api.txt` diffs
  (`query::*Statement::node_id: u128` → `: NodeId`), every `Statement`-shaped
  pattern match in-tree follows, and it adds `Display`/`From`/`Deref` surface to
  maintain. It also does not touch `FieldValue::Int`, so it solves nothing that
  O1 does not.
- **Rejected:** pays a public-API diff for a guarantee that a 6-field test
  provides for free. Recorded because "the type should have been a newtype"
  is the objection a future reader will raise.

## Decision outcome

Chosen: **O1**, three rules.

### D1 — the six `u128` id fields serialize as decimal strings

Applied as a field attribute reusing the existing helper:

```rust
#[serde(serialize_with = "crate::sdk::types::u128_serde::serialize")]
pub node_id: u128,
```

`u128_serde::serialize` (`src/sdk/types.rs:42-47`) emits
`serializer.serialize_str(&val.to_string())`; the module is `pub(crate)`
(`src/sdk/mod.rs:13`) behind an unconditional `pub mod sdk;` (`src/lib.rs:140`),
so `src/query.rs` resolves it with no feature gate and no new `pub` symbol.

Shape: `{"Insert": {"node_id": "123456789012345678901234", "node_type": …}}`.
Always a string — never conditionally a number.

**Six hand-applied attributes need a guard, not a convention.** The
implementation MUST add one test that parses one statement per id-bearing
variant and asserts `as_str()` for each field: `Insert` (`node_id`), `Update`
(`node_id`), `Delete` (`node_id`), `Relate` (`source_id` **and** `target_id`),
`InsertMessage` (`thread_id`). ~20 lines. Without it the seventh `u128` field
regresses silently — which is exactly how this bug was born.

### D2 — `i64` `Int` literals stay JSON numbers, limit pinned by test

Three reasons, in order of weight:

1. **Blast radius.** `FieldValue` is shared with the persisted node model and
   with `QueryResult` metadata (see O3). Changing it is an on-disk-format
   decision.
2. **The two families need different guarantees.** An id must be **bit-exact to
   be usable** — it is passed back through `delete()`/`relate()` and wrong digits
   mean the wrong node. A literal only needs to be **legible**: the caller wrote
   it and still has it; the AST is an echo, not a handle. (Note `Statement` has
   no `Deserialize`, so an AST is never fed back into the engine — the risk is
   the token channel, not a round-trip.)
3. **Scope discipline.** Widening this to the value model would couple an API
   decision to a storage decision — the coupling ADR-0060 exists to prevent.

The existing `2^53` test keeps asserting exactly what it asserts today (an `Int`
literal). It is **not** split: it does not cover ids and never did, so the id
coverage in D1 is **new** coverage, not a migration of that test.

### D3 — ship it in 0.10.0 (before `parse()` is ever published)

`parse()` is unreleased (§Context). The change is free today, breaks nothing in
the tree, and lands in the last cheap window. Under 0.x semver it is a minor
bump; no migration note is needed because no released version exposes the
surface. If the release slips, the decision does not.

### Migration surface (ordered by the implementation task)

| # | Artefact | Today | After | Consequence if missed |
|---|---|---|---|---|
| 1 | `src/query.rs` (6 fields) | `node_id: u128` → number | `#[serde(serialize_with = …u128_serde::serialize)]` → decimal string | nothing changes anywhere; the whole ADR is inert |
| 2 | `src/parser/mod.rs::tests::test_ast_json_projection_shape` (1559–1576) | asserts `Relational[2]["Int"] == 28` (unaffected); no id case | **add** the per-variant id test of D1 | the new shape is unpinned; the 7th `u128` field regresses silently. The other three `to_value(&stmt)` sites in the module (1719, 2025, 2031) assert no ids → they do not break |
| 3 | `tests/api/public-api.txt` (golden) | `pub vantadb::query::*Statement::node_id: u128` — 6 lines (5128, 5148, 5167, 5278, 5279, 5370) | **no diff** — an attribute is not a signature | if it diffs, a `pub` symbol leaked (e.g. `u128_serde` made `pub`): back it out |
| 4 | `docs/api/IQL.md` §AST JSON (512–559) | no id example; `:523` claims "`Int` literals serialize as `{"Int": 28}` (i64, exact)" | add the id rule + an `Insert`/`Delete` example; qualify `:523` (exact in Rust/Python, lossy through JS above 2^53) | the doc contradicts the code **and** keeps an unqualified falsehood |
| 5 | `vantadb-python/tests/test_find156_ast_json.py:88-91` | `TO THREAD#1` → asserts only that the key exists | assert `ast["InsertMessage"]["thread_id"] == "1"` — the output type changes for Python too | Python's AST pin does not move with the shape |
| 6 | `docs/api/PYTHON_SDK.md:960-962` | already says the projection is **exact** on this platform | **no retraction needed** (it was right); ensure no example shows a numeric id | — (verify-only row) |
| 7 | `vantadb-wasm/src/lib.rs:1860-1862` + `src/vantadb_wasm.d.ts:144-146` + rebuild `pkg/` | docstring and `.d.ts` say ids **and** `Int` >2^53 lose precision | narrow both to `Int` only | the published `.d.ts` keeps a false caveat |
| 8 | `vantadb-ts/src/__tests__/find156.test.ts` | pins the `Int` limitation (103–115); **no id test** | **add** an id assertion (`typeof node_id === "string"` + exact decimal); leave the `Int` assertion untouched | the fixed half stays untested — in the one file that already documents the limit |
| 9 | `vantadb-ts/src/types.ts:48-54` | caveat covers ids **and** `Int` | narrow to `Int` | TS type doc lies about ids |
| 10 | `docs/api/TS_SDK.md:551-555` | "`u128` node ids and `Int` literals above `2^53` lose precision" | narrow to `Int` | published doc keeps a false caveat |
| 11 | `docs/api/BINDINGS_NAMESPACES.md:48` | rule "`node_id`/`id` as decimal strings (u128 > 2^53)" | **no edit** — the rule stays true and the AST now obeys it | — (verify-only row) |

`vantadb-node` is **not** in the table: it does not expose `parse()`
(FIND-156 §Incógnitas 6), so there is nothing to change — and that parity gap
stays open.

### Positive consequences

- `parse()` becomes a faithful transport for the one family that must be
  bit-exact. An id read from the AST can be fed back to `delete`/`relate` on
  every platform, and the output surface stops accepting what the input surface
  (`vantadb.ts:160-168`) rejects.
- One convention across the product: `u128` ids are decimal strings on every
  JSON/FFI wire, as `docs/api/VERSIONING.md:54-55` already requires.
- No new public symbol, no new type, no feature coupling: `public-api.txt` stays
  byte-identical and the One-Version rule holds.
- The decision is recorded while it is still free, so it cannot be re-litigated
  as "breaking" later.

### Negative consequences

- **R1 — six attributes, no structural guard.** `u128` is not a type here, so
  forgetting the attribute on a seventh field is possible; the compiler will not
  help. Mitigation: the per-variant id test in D1 is mandatory, and
  `src/graphrag/pipeline.rs:26` shows the in-tree precedent to copy.
- **R2 — `Int` literals above 2^53 are still lossy through JS** (residual, by
  design). The fix is **not** blocked on the `FieldValue` on-disk decision: the
  cheap escape hatch is O7, a raw-text accessor (`parse_ast_json() -> str`),
  which is an additive API decision deferred to its own ADR. Until then the
  limit is documented and pinned for the object channel. This is debt this ADR
  creates knowingly.
- **R3 — asymmetry with `Value::Int` in results.** `QueryResult` metadata keeps
  numeric `Int`. Two answers to "can an integer be a string on the wire?"
  coexist until R2 is paid.
- **R4 — a decimal string is not self-validating.** Consumers must not
  `parseInt`/`Number(...)` an id; keeping the string is the point, and nothing in
  JSON enforces it. This is a documented obligation (`AstJson` docstring), the
  same one ProtoJSON places on its users.
- **R5 — the wider 0.10.0 blast radius.** Three docs, three bindings and two
  tests must move with the six attributes; a partial landing leaves the `.d.ts`
  or `IQL.md` contradicting the code — the exact bug class FIND-156 closed.

## Verification

This ADR is design-only. The design checks below are falsifiable today; the
implementation checks belong to the follow-up task and are listed so the
decision can be falsified later.

**Design (runnable now, from the repo root):**

```bash
A=docs/dev/architecture/adr/ADR-0062-ast-json-numeric-fidelity.md
rg -c '^## Context and problem statement$' "$A"   # 1
rg -c '^## Considered options$'              "$A"   # 1
rg -c '^## Decision outcome$'                "$A"   # 1
rg -c '^### Positive consequences$'          "$A"   # 1
rg -c '^### Negative consequences$'          "$A"   # 1
rg -c '^### O[0-9]+ '                        "$A"   # 8 — not a single-option ADR
node scripts/docs/check-docs.mjs && node scripts/docs/check-links.mjs
```

`check-docs` can be red for a reason that has nothing to do with this ADR: it
gates the whole tree, so any **untracked** doc missing frontmatter fails it. The
gating entry is printed by name in the report — if it is not this ADR and not a
file in the migration table, it belongs to another lane.

**Implementation (belongs to the follow-up task):**

- `cargo nextest run --profile audit -p vantadb -E 'test(ast_json)'` — the
  projection test asserts the six string ids and the untouched numeric `Int`.
- `cargo public-api` vs `tests/api/public-api.txt` — must be byte-equal.
- `npx vitest run src/__tests__/find156.test.ts` — the `Int` pin unchanged, the
  new id assertion exact.
- `python -m pytest vantadb-python/tests/test_find156_ast_json.py` — Python stays
  **exact** and its output type **changes** (ids become strings): the pin proves
  the shape moved, it does not prove behaviour is unchanged.

## References

- `.opencode/rules/api-contract.md` — R-1 (docs must name real symbols), R-8
  (bindings are glue; the shape comes from the core).
- `.opencode/rules/js-ecosystem.md` — WASM/TS binding lifecycle.
- `docs/api/IQL.md` §AST JSON — the documented surface this ADR changes.
- `docs/api/VERSIONING.md:54-55` — the identifier rule this ADR restores.
- `docs/dev/tasks/FIND-723.md` — the analysis, option table and migration
  checklist this ADR summarises.
- `docs/dev/tasks/FIND-156.md` §Incógnitas 7 — the measurement that opened this.
- [RFC 8259 §6](https://www.rfc-editor.org/rfc/rfc8259.html) ·
  [ProtoJSON](https://protobuf.dev/programming-guides/json/) ·
  [MDN `Number.MAX_SAFE_INTEGER`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER) ·
  [Discord API reference](https://docs.discord.com/developers/reference)
  (all fetched 2026-11-24).
