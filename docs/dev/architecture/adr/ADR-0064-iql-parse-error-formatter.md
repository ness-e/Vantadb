---
title: "ADR-0064: One IQL parse-error formatter in the core - parse_iql and 1-based character positions"
kind: adr
status: accepted
description: "IQL parse errors get one public formatter in the core (parse_iql -> Result<Statement>); line/col are 1-based characters, so all three bindings inherit one position contract"
tags: [vantadb, architecture, adr, api, iql, bindings, errors]
created: "2026-11-25"
owner: vanta-arch
---

# ADR-0064: One IQL parse-error formatter in the core - `parse_iql` and 1-based character positions

## Status

Accepted. This record documents a decision **already implemented** in
`FIND-722` (commit `c8d9e95aa`) and closes the public-surface obligation that
the implementation created: `pub fn vantadb::executor::parse_iql` is the only
real addition to the crate's public API in that change. It supersedes nothing
and re-opens nothing: the options below are recorded so the rejected ones are
not re-proposed, not so the choice can be re-made. Implementation evidence lives
in [docs/dev/tasks/FIND-722.md](../../tasks/FIND-722.md).

## Context and problem statement

`Client.parse()` exists in the Python, WASM and TS bindings since `FIND-156`
(commit `20c164970`). Its error was useless for diagnosis: all three call
`parse_statement` and formatted the failure as
`Error::InvalidInput(format!("IQL parse error: {e}"))`, where `{e}` is the
`Debug` of `nom::Err` - combinator internals with **no position**:

```
IQL parse error: Error(Error { input: "", code: Eof })
```

The execution path already did the right thing. `Executor::execute_hybrid`
(`src/executor.rs:357`) composed two **private** helpers -
`iql_error_position` (`:90`) and `iql_parse_error_message` (`:120`) - into
`Error::IqlParse { msg, line, col }` (`src/error.rs:214-223`), whose `Display`
is `IQL parse error at line {line}, col {col}: {msg}`. The composed entry point
is `parse_iql` (`src/executor.rs:217`).

So the problem was not a missing capability but an **ownership** question: where
does the composition live, given that three FFI bindings and one core path all
need the same error? Copying the two helpers into each binding produces four
copies of one order of operations (`api-contract` R-8: one fact, one owner).

Two defects were measured while answering it, and both are contract, not
cosmetics:

1. **Mixed bases.** `line` was 1-based (`.max(1)`) while `col` was the 0-based
   token index. `FROM Person p extra_stuff` reported column 14 with
   `extra_stuff` at 15; `FROM Doc LIMIT abc` reported 9 for a `LIMIT` at 10.
   A published contract that is "1-based" in one field and "0-based" in the
   other is not a contract.
2. **Bytes vs characters.** Counting bytes makes the position wrong for any
   non-ASCII text before the token: with `"José"` in the input, bytes say 36
   where characters say 35. Users read characters.

## Considered options

### O1 - one composing `pub fn` in the core (chosen)

`pub fn vantadb::executor::parse_iql(&str) -> Result<Statement>` composes the two
private helpers and returns the finished `Error::IqlParse`. `execute_hybrid`
delegates to it; the three bindings call it instead of `parse_statement` +
`format!`.

- Pros: one entry point, so the error cannot be built without a position; the
  two paths are byte-identical by construction; exposes no `nom` type
  (`Statement` is already public in `src/query.rs`).
- Cons: +1 item of permanent public surface for a capability only the bindings
  and the core need - no external Rust user calls it.

### O2 - promote the two helpers to `pub`, compose in each caller

- Pros: smallest visibility change; each caller decides its own message.
- Cons: the composition order becomes a convention, not a type. Forgetting one
  of the two calls silently returns an error with no position, and there are
  four callers today and two more bindings coming (`vantadb-node`,
  `vantadb-sdk`). This is the exact failure mode the change exists to remove.

### O3 - `pub fn describe_parse_error(&str) -> (String, usize, usize)`

Every caller builds `Error::IqlParse` itself.

- Pros: still one place that knows where the position comes from; no `Result`
  coupling.
- Cons: moves the *construction* of the public error into four callers, which
  is where the mixed-base bug actually lived. It also invites each binding to
  format its own message, breaking the "same `Display` on every channel" rule
  that the Python and TS tests now assert.

### O4 - keep the status quo and document the gap

Bindings keep formatting `nom`'s `Debug`; the limitation is documented in
`docs/api/IQL.md`.

- Pros: zero surface change, zero code.
- Cons: the gap was already documented (`IQL.md:599-605`,
  `TS_SDK.md:567`) and a documented gap is still a gap. A user with a
  1,000-line IQL migration script gets `Eof` with no offset, on the surface the
  product tells them to use to validate that script.

### O5 - `#[doc(hidden)] pub fn parse_iql`

Same code as O1, hidden from the docs.

- Pros: satisfies the bindings without adding a documented API item.
- Cons: worst of both worlds. The symbol is still public - it is still a semver
  commitment forever, still in the `cargo public-api` golden - but the reader of
  `docs/api/` cannot find the one function that produces the error they are
  holding. Hiding reachable API is exactly what
  `api-and-interface-design` calls a Hyrum-surface smell.

## Decision outcome

**O1.** One public formatter, `vantadb::executor::parse_iql`, returning
`Result<Statement>`.

### D1 - the entry point returns the finished `Error`, not its parts

The single function makes "an IQL parse error without a position" unrepresentable
for callers. `Statement` was already public; no `nom` type crosses the boundary,
so the parser implementation stays private.

### D2 - `execute_hybrid` delegates to it

The execution path and the bindings must produce the **same** error byte for
byte; a test asserts it (`parse_iql_matches_execute_hybrid_iql_error`). Keeping
two formatters and hoping they stay in sync is how the divergence in the first
place happened.

### D3 - `line` and `col` are 1-based, counted in **characters**

`line 1` is the first line; `col 1` is the first character. The count is
`input[line_start..consumed].chars().count() + 1`. Base 0 was rejected because
`Display` is already published in `docs/api/IQL.md` and `openapi.yaml`, and bytes
were rejected because a user counts characters, not UTF-8 bytes.

### D4 - position is computed on the text **after** `trim_start()`

`execute_hybrid` already passes the trimmed text. Parity with `query()` is the
goal, so `parse()` reports the same numbers for the same statement, including
leading whitespace being invisible to both.

### D5 - soft errors report (1,1), and this is documented

`parse_statement` (`src/parser/grammar.rs:889-898`) resolves seven alternatives
with `alt`, which propagates only the **last** alternative's error
(`parse_query`, at offset 0). So `INSERT INTO` - a known keyword with an
incomplete statement - collapses to 1:1. `Failure` errors (`Verify`/`Fail`/`Eof`)
keep their position. Recovering the best error would be a parser redesign and is
out of scope; the limitation is documented in `docs/api/IQL.md`.

### D6 - the error class and message keep their shape

`Error::IqlParse` keeps mapping to `ValidationError` /
`VANTADB_VALIDATION_ERROR`. Python adds `line`/`col` as **attributes** of the
exception; TS puts them in `DbError.details`; the message string is unchanged.
Additive fields on an existing error, so no consumer that matched on class or
message breaks.

### D7 - the surface cost is one item, and it is additive

`cargo public-api` golden delta is exactly one line:

```diff
+pub fn vantadb::executor::parse_iql(&str) -> vantadb::error::Result<vantadb::query::Statement>
```

A new public function is a **minor** change (`feat:`), which is what
`FIND-722` shipped. Removing it later would be a **major** change; it is
therefore documented in `docs/api/IQL.md` rather than hidden (see O5).

### D8 - `[no-adr]` was evaluated and rejected

The `ADR Gate` (`ci-rust.yml`, job `adr-gate`) is path-based: a change under
`src/`, `vantadb-python/`, `vantadb-ts/`, `vantadb-server/` or `vantadb-wasm/`
needs at least one ADR file in the PR, or a `[no-adr]` marker in the PR
title/body with a justification. `[no-adr]` is the correct escape for a change
that touches one of those paths **without** taking a contract decision (a
refactor, a rename, a test-only edit). This change does take one - it publishes
a position contract that did not exist, fixes a mixed-base bug in a published
contract, and adds a permanent public symbol - so the marker would have been a
bypass, not a decision. It is also a weak bypass by construction: the check is a
plain `contains()` on the title and body (`ci-rust.yml:273-274`) with no
justification validation, and the acknowledgement step only emits
`::warning::` (`:286`). The ADR is the honest branch.

## Positive consequences

- One formatter, one order of operations, one `Display`. The four callers
  cannot drift; a fifth binding gets the position for free.
- Position is available on the surface users actually validate migrations
  against (`parse()`), not only on the execution path.
- The mixed-base defect is gone: `col` is 1-based everywhere and counts
  characters. Pinned by an assert with multibyte input (`"José"` -> 35, not 36)
  and by asserts for four previously off-by-one statements.
- The public boundary stays narrow: `Statement`, not `nom`.

## Negative consequences

- **One more permanent public item.** `parse_iql` is public for the bindings'
  convenience, not because a Rust consumer asked for it. It is now a semver
  commitment (removal = major), documented in `docs/api/IQL.md`.
- **`parse()` and `query()` diverge on LISP-style input.** Parenthesised LISP
  input is accepted by `query()` and rejected by `parse()` with a different
  error path; documented in `IQL.md` §Error Handling and pinned by a binding
  assert. Two entry points with one written difference is better than two
  undocumented behaviours.
- **Soft errors lose the position** (D5). `INSERT INTO` reporting 1:1 is
  correct-by-current-parser, not useful-by-intent; fixing it is a parser change
  with its own ADR.
- **The docs gate over-reports this change.** The static extractor in
  `scripts/docs/check-api-docs.mjs` records `impl llm::EmbedCache` and
  `impl llm::WarmLocal` as surface items in the same changeset, so the gate says
  `+3` where the compiled surface is `+1`. Both types are **private**
  (`src/llm.rs:240,267` declare `struct`, not `pub struct`), they appear in no
  `cargo public-api` line, and nothing outside `src/llm.rs` can name them. The
  extractor's `impl` rule does not test visibility. The defect is returned for
  registration as a follow-up Backlog row (see §Surface bookkeeping); it is not
  a reason to doc-hide real API, and it is not evidence that the warm ONNX state
  is contractual.

## Surface bookkeeping

Measured, not assumed (2026-11-25, `develop`):

| Check | Command | Result |
|---|---|---|
| Compiled surface delta | `git diff 06485fe8e..HEAD -- tests/api/public-api.txt` | **+1 line**, `parse_iql` only |
| Golden matches code | `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only --locked` | PASS |
| Static gate view | `node scripts/docs/check-api-docs.mjs --changed 06485fe8e..HEAD` | `+3` (`parse_iql` + two phantom `impl` lines of private types) |
| Nobody names the internals | `rg -n 'EmbedCache\|WarmLocal' --glob '*.rs' -l` | `src/llm.rs` only |

The two `impl` lines are a **gate defect, not a visibility leak**: there is no
`pub` to demote. Scope, measured over the whole crate at this commit: **46 of the
176** `impl` entries the extractor reports belong to types that are neither
declared `pub` anywhere nor publicly re-exported (`EmbedCache`, `WarmLocal`,
`EnvGuard`, the `MockScan`/`FakePorts` test doubles, …), so the over-report is
structural, not local to this change. Fixing the extractor to require a `pub`
type is a change to a shared CI script owned by the CI lane
(`scripts/docs/**`, rows `FIND-713`/`FIND-264`), so it is a separate row — the
row text is returned with `FIND-733` and is **not yet in the Backlog**; this
record does not claim an ID it does not have.

## Verification

- `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only --locked` - the golden is byte-identical to the code.
- `cargo nextest run --profile audit -p vantadb -E 'test(iql_error) or test(parse_error)' --build-jobs 2` - the position contract, including the multibyte and multiline cases (`FIND-722` §Verificación).
- Binding tests: `vantadb-python/tests/test_find156_ast_json.py`,
  `vantadb-ts/src/__tests__/find156.test.ts` - `line`/`col` arrive as data, not
  parsed from the message.
- Gate: `node scripts/docs/check-api-docs.mjs --changed <base>..HEAD` stays
  green because `docs/` and `llms.txt` move with the surface.

## References

- [FIND-722 task record](../../tasks/FIND-722.md) - the implementation, its
  measured defects and its P2-01 review (one High, four Medium, three Low, all
  closed).
- [ADR-0062](./ADR-0062-ast-json-numeric-fidelity.md) - the sibling decision on
  the same surface (`parse()`'s AST JSON), taken in the same release window.
- `docs/api/IQL.md` §Error Handling - the published position contract,
  including the `(1,1)` soft-error case and the `parse()`/`query()` asymmetry.
- `docs/api/ERROR_HANDLING.md`, `docs/api/PYTHON_SDK.md`, `docs/api/TS_SDK.md`,
  `docs/api/BINDINGS_NAMESPACES.md` - the per-binding shape of `line`/`col`.
- `.opencode/rules/api-contract.md` (R-8: one owner per fact) and
  `.opencode/rules/core-engine.md`.
- `ci-rust.yml` job `adr-gate` - the mechanical rule this record satisfies.
