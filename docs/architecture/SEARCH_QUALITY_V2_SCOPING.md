# Search Quality v2 — Scoping (INV-025)

> **Status:** Scoping approved — implementation deferred to INV-009-B and follow-ups.
> **Date:** 2026-08-05
> **Scope:** Design/documentation only. No code changes in this task.
> **Source task:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 44 (INV-025).
> **Precedes:** Task 45 (INV-009-B — `Condition::TextMatch` / phrase queries).

## 1. Purpose

Define the public API surface for snippet generation + highlighting in v0.6,
separate stable outputs from debug-only internals, state explicit non-goals,
and establish the validation corpus that INV-009-B (phrase queries) must keep
green. This document is the contract Task 45 (INV-009-B) implements against.

## 2. Current state (verified 2026-08-05)

### 2.1 Internal machinery — `src/sdk/search/snippet.rs`

All primitive functions are **`pub(crate)`** (or private) — none is exported:

| Symbol | Visibility | Notes |
|---|---|---|
| `fold_char` / `fold_str` | `pub(crate)` | Unicode→ASCII folding, case-insensitive |
| `generate_snippet_with_highlighting` | `pub(crate)` | Core; ~120-char window around first token match, `...` ellipses |
| `debug_snippet` | `pub(crate)` | Plain-text variant (no highlighting) |
| `highlight_terms` | private | Wraps every term occurrence in `<strong>` |

Highlighting is **term-level only**: it iterates `query_plan.terms`
(`BTreeSet<String>`) and matches each term independently. There is **no phrase
awareness** today — a query like `"neural network"` highlights `neural` and
`network` anywhere in the payload, not the contiguous phrase. This is the gap
INV-009-B closes.

### 2.2 Public exposure — already shipped

Contrary to the stale backlog premise ("no public API decision"), a stable
entry point already exists and is surfaced in **three** bindings:

- **Rust SDK:** `pub fn generate_snippet(&self, payload: &str, text_query: &str, with_highlighting: bool) -> Option<String>` — `src/sdk/search/mod.rs:814-823`.
- **TypeScript:** `db.generateSnippet(payload, query, withHighlighting)` — `vantadb-ts/src/vantadb.ts:882-906` (+ integration test).
- **Python:** `generate_snippet(payload, text_query, with_highlighting)` — `vantadb-python/src/lib.rs:1613-1627` (delegates to engine).
- **WASM:** exported in `pkg/vantadb_wasm.d.ts:90-92`.

### 2.3 What is NOT exposed today

- **CLI:** no subcommand exposes snippet generation (grep of `src/cli/` = 0 hits).
- **HTTP API:** no endpoint returns snippets (only explain-mode `snippet` field in SDK types).
- **Explain mode** (`VantaSearchExplanationHit.snippet`, `src/sdk/types.rs:434-435`) populates a plain-text snippet via `debug_snippet` (`src/sdk/search/debug.rs:72`).

## 3. Desired public outputs (v0.6)

### 3.1 Public (stable, documented)

| Output | Surface | Rationale |
|---|---|---|
| `generate_snippet(payload, text_query, with_highlighting) -> Option<String>` | Rust SDK + TS + Python + WASM (exists) | Primary user-facing primitive; already shipped, must become covered/verified, not re-architected |
| Phrase-aware highlighting (multi-word `<strong>` for contiguous matches) | Same 4 surfaces, same signature | Delivered by INV-009-B (`highlight_phrases`) — **no new signature** |
| Documentation of behavior: window ~120 chars, first-token anchoring, `...` ellipses, accent/case-insensitive folding | `docs/api/*` | Contract for bindings parity |

### 3.2 Debug-only (NOT public, stay `pub(crate)`/private)

| Symbol | Reason to keep internal |
|---|---|
| `debug_snippet` | Explain-mode/planning diagnostics only; no user value |
| `fold_char` / `fold_str` | Implementation detail; any change would be a breaking leak |
| `highlight_terms` | Internal helper; `generate_snippet_with_highlighting` is the only sanctioned entry |
| `VantaSearchExplanationHit.snippet` | Already public in types; content stays plain-text (debug flavor) — do **not** start emitting `<strong>` into explain payloads |

### 3.3 Explicitly OUT of v0.6

- No new CLI command for snippets.
- No snippet field added to `VantaMemorySearchHit` (non-explain mode). If UX later needs snippets on hit rows, that is a separate design (index-time snippet storage) — out of scope.
- No HTML escaping / sanitization of highlighted output beyond the current `<strong>` convention. Callers receive raw `<strong>` markup as today.

## 4. Non-goals (explicit)

1. **No hybrid-search parity claims.** Snippet quality is defined for lexical
   (text) queries only. No assertion that snippets match dense-vector ranking.
2. **No tokenizer rewrite.** `src/tokenizer.rs` / `src/text_index.rs`
   tokenization stays as-is. INV-009-B reuses existing literal tokenization
   ("sin stopwords") — it must not fork a new tokenizer.
3. **No new snippet-scoring algorithm.** Keep the current first-match
   anchoring heuristic. Better snippet selection (e.g., best cluster of terms)
   is a future work item, not part of v0.6.
4. **No extraction of HTML/markdown structure** from payloads. Payload is
   treated as plain text.
5. **No performance work** (pre-computed snippet index, caching) in this
   scope.

## 5. Validation corpus (small, deterministic)

Corpus of 6 payloads + assertions, extracted from existing unit tests in
`snippet.rs` so the bar is enforceable without new tooling. INV-009-B must keep
these green and add phrase cases.

| # | Payload | Query | Assertion (with_highlighting=true) |
|---|---|---|---|
| C1 | `"hello world"` | `hello` | `<strong>hello</strong> world` (short payload, no truncation) |
| C2 | `"The quick brown fox jumps over the lazy dog. "` ×5 | `fox` | contains `fox`; length < payload; `...` ellipses on both sides |
| C3 | `"quick brown fox"` | `quick fox` | `<strong>quick</strong> brown <strong>fox</strong>` (multi-term) |
| C4 | `"El café molido y la crème brûlée son exquisitos"` | `cafe creme` | `<strong>café</strong>` and `<strong>crème</strong>` (accent folding) |
| C5 | `"hello world this is a long payload..."` | `zzzzz` | snippet still returned (non-match → prefix fallback) |
| C6 | `"hello world"` | `""` | `None` (empty query) |

**Phrase cases (added by INV-009-B):**

| # | Payload | Query | Assertion |
|---|---|---|---|
| P1 | `"neural network training"` | `"neural network"` | `<strong>neural network</strong> training` (contiguous, single wrap) |
| P2 | `"neural nets and network topology"` | `"neural network"` | no phrase match → **no** `<strong>network</strong>` on second word (phrase ≠ union of terms) |

## 6. Formal dependency — INV-009-B (Task 45)

**INV-009-B implements:** `Condition::TextMatch(field, query)` in the graph IQL
parser (`src/query.rs:121-126` — enum currently has only `Relational` and
`VectorSim`; `TextMatch` does not exist yet).

**Contract INV-009-B must honor (from this doc):**

1. Phrase matching must go through snippet highlighting as **contiguous
   phrase** highlighting (`highlight_phrases`), not term-union highlighting —
   see corpus P1/P2.
2. Reuse `generate_snippet_with_highlighting` signature — do not add a new
   public entry point.
3. Keep the existing 6 corpus cases green; add P1/P2 to the same unit test
   module.
4. `highlight_phrases` (planned) may be a **private** helper; the public API
   surface remains unchanged.
5. No tokenizer changes (Non-goal #2) — phrase tokenization uses literal
   matching against the existing token stream.

**Blocker semantics:** this doc is a precondition, not a code dependency. If
the phrase design (P1/P2) conflicts with the current term-anchored window
logic, INV-009-B adjusts the window logic inside `snippet.rs` but must not
change the public signature.

## 7. Decisions (registered)

> Format: lightweight ADR-style records. Filed in this doc (scoping precedes
> implementation; a standalone ADR is not warranted until v0.6 ships).

- **D-1 (2026-08-05):** The existing 4-surface `generate_snippet` is the
  sanctioned public API for v0.6. No redesign, no new signature.
- **D-2 (2026-08-05):** Snippet highlighting for phrase queries is
  **phrase-contiguous** (`<strong>neural network</strong>`), never
  term-union. Rationale: avoids false-positive highlights that erode
  user trust in quoted queries.
- **D-3 (2026-08-05):** Debug internals (`debug_snippet`, folding helpers,
  term-level `highlight_terms`) remain non-public. Explain-mode snippet stays
  plain text.
- **D-4 (2026-08-05):** No CLI snippet command in v0.6. CLI surface is
  deferred until a concrete consumer (e.g., interactive `search` output with
  context) exists.
- **D-5 (2026-08-05):** Validation corpus = §5 (6 existing + 2 new phrase
  cases), enforced as unit tests in `snippet.rs`. No external dataset needed.

## 8. Future work (explicitly out of v0.6)

- Best-cluster snippet selection (beyond first-match anchoring).
- Snippet field on non-explain `VantaMemorySearchHit` (index-time snippet storage).
- HTML-safe rendering / escaping strategy for `<strong>` output.
