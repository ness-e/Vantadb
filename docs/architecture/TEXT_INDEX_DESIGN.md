# Persistent Text Index Design

Date: 2026-05-04

## Purpose

The text index is an internal, persistent, derived index over memory `payload`
fields. It supports BM25 lexical retrieval and can participate in Hybrid
Retrieval v1 with vector results through RRF. Canonical memory records remain
the source of truth.

This is still a conservative hybrid implementation: simple planner, BM25,
current vector ranking, and RRF fusion. It is not a claim of competitive
hybrid-search parity.

## Tokenization

- Active tokenizer spec: `lowercase-ascii-alnum`, version `1`.
- Split on non-ASCII-alphanumeric characters.
- Lowercase ASCII tokens and preserve numeric tokens.
- No stemming, stop words, Unicode folding, or language-specific normalization
  are implemented in this phase.
- Phrase handling uses the same tokenizer and exact consecutive token
  positions; it is not a linguistic phrase parser.

The tokenizer is versioned in the text-index state marker so future tokenizer
changes can force a safe rebuild.

## Key And Value Shape

Posting keys keep the stable shape:

`namespace + NUL + token + NUL + key`

Posting values are compact serialized records containing:

- canonical `node_id`
- document term frequency `tf`
- token positions for basic phrase matching

The text index also stores small derived stats under reserved internal v3
prefixes that cannot collide with validated namespaces:

- term stats by `namespace/token`: document frequency `df`
- document stats by `namespace/key`: `node_id` and document length
- namespace stats: document count and total document length

The index does not duplicate full payloads.

## Mutation And Rebuild Model

The text index is derived from canonical memory records, same as ANN and
namespace/payload indexes.

- `put`/update deletes stale postings and document stats, writes current
  postings with TF and positions, updates DF, and updates namespace corpus
  stats in the same derived backend batch.
- `delete` removes postings/document stats and decrements term/namespace stats.
- `rebuild_index` rebuilds ANN, namespace/payload indexes, and then all text
  postings and BM25 stats from canonical records.
- JSONL export/import does not serialize text-index internals; import rebuilds
  the derived text index from imported canonical records.
- Writable `open` repairs missing, corrupt, incompatible, or count-stale text
  index state from canonical records.
- `read_only` open does not mutate or repair. Text-only search on an
  incompatible read-only index returns a clear operational error.

## Query Behavior

Text-only memory search (`text_query` non-empty and `query_vector` empty) uses
BM25 with `k1 = 1.2` and `b = 0.75`:

`idf = ln(1 + (N - df + 0.5) / (df + 0.5))`

The query path is namespace-scoped, uses persisted postings/stats, applies
existing metadata filters to lexical candidates, respects `top_k`, and sorts
by score descending with deterministic `key`/`node_id` tie breakers.

Phrase queries are enabled inside `text_query` with double quotes, for example
`"alpha beta"`. Phrase matching is implemented as an exact consecutive token
filter over persisted posting positions. Unquoted terms preserve the existing
BM25 behavior.

Whitespace-only `text_query` preserves the vector-search behavior.

Hybrid memory search (`text_query` non-empty and `query_vector` non-empty)
executes BM25 and vector retrieval independently, then fuses ranked candidates
with Reciprocal Rank Fusion. The internal RRF constant is `60.0`. Each side
uses a candidate budget of `max(top_k, min(max(top_k * 4, 32), 256))`, then the
final fused list is truncated to `top_k`.

RRF fuses by logical identity `namespace + key`; candidates that appear in only
one ranking still participate. Final ordering is fused score descending, then
stable `key`/`node_id` tie breakers. Raw BM25 and cosine scores are not blended.

Debug builds expose internal certification helpers for tests that report the
planned route, hybrid budget, text/vector candidate counts, fused candidate
count, top logical identities, snippets from canonical payloads, BM25 term
contributions, matched phrases, and RRF ranks. These helpers are not stable SDK
APIs.

The stable audit surface is narrower: `VantaEmbedded::audit_text_index` and
`vanta-cli audit-index` compare derived text-index entries against canonical
records, report drift, and do not repair.

## Consistency And Observability

The state marker tracks schema/tokenizer/key format plus counts for canonical
records, postings, document stats, term stats, and namespace stats. Writable
open rebuilds when those counts do not match the canonical source of truth.

A read-only structural audit compares expected postings/stats from canonical
records against actual text-index entries. This catches incorrect entries with
matching counts, works in read-only mode, and recommends explicit
`rebuild_index` repair when drift is detected.

Operational metrics remain diagnostic only:

- `text_index_rebuild_ms`
- `text_postings_written`
- `text_index_repairs`
- `text_lexical_queries`
- `text_lexical_query_ms`
- `text_candidates_scored`
- `text_consistency_audits`
- `text_consistency_audit_failures`
- `hybrid_query_ms`
- `hybrid_candidates_fused`
- `planner_hybrid_queries`
- `planner_text_only_queries`
- `planner_vector_only_queries`

## Deferred Work

- Public ranking explanations and rich snippet/highlighting APIs.
- Unicode/ascii-folding/stemming/stopword tokenizer evolution.
- Competitive hybrid-search claims.
