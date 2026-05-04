# Persistent Text Index Design

Date: 2026-05-04

## Purpose

The text index is an internal, persistent, derived index over memory `payload`
fields. It now supports BM25 lexical retrieval for text-only `text_query`
requests while keeping canonical memory records as the source of truth.

This is not a hybrid-search claim. Requests that combine `text_query` and
`query_vector` still return an explicit RRF/planner deferred error.

## Tokenization

- Active tokenizer spec: `lowercase-ascii-alnum`, version `1`.
- Split on non-ASCII-alphanumeric characters.
- Lowercase ASCII tokens and preserve numeric tokens.
- No stemming, stop words, phrase positions, snippets, Unicode folding, or
  language-specific normalization are implemented in this phase.

The tokenizer is versioned in the text-index state marker so future tokenizer
changes can force a safe rebuild.

## Key And Value Shape

Posting keys keep the stable shape:

`namespace + NUL + token + NUL + key`

Posting values are compact serialized records containing:

- canonical `node_id`
- document term frequency `tf`

The text index also stores small derived stats under reserved internal v2
prefixes that cannot collide with validated namespaces:

- term stats by `namespace/token`: document frequency `df`
- document stats by `namespace/key`: `node_id` and document length
- namespace stats: document count and total document length

The index does not duplicate full payloads.

## Mutation And Rebuild Model

The text index is derived from canonical memory records, same as ANN and
namespace/payload indexes.

- `put`/update deletes stale postings and document stats, writes current
  postings with TF, updates DF, and updates namespace corpus stats in the same
  derived backend batch.
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

Whitespace-only `text_query` preserves the previous vector-search behavior.
Hybrid text+vector remains deferred until RRF/planner work is implemented and
covered.

## Consistency And Observability

The state marker tracks schema/tokenizer/key format plus counts for canonical
records, postings, document stats, term stats, and namespace stats. Writable
open rebuilds when those counts do not match the canonical source of truth.

A debug/internal structural audit can compare expected postings/stats from
canonical records against actual text-index entries. This catches incorrect
entries with matching counts and is used in certification tests.

Operational metrics remain diagnostic only:

- `text_index_rebuild_ms`
- `text_postings_written`
- `text_index_repairs`
- `text_lexical_queries`
- `text_lexical_query_ms`
- `text_candidates_scored`
- `text_consistency_audits`
- `text_consistency_audit_failures`

## Deferred Work

- RRF fusion for text+vector hybrid ranking.
- Planner behavior that combines vector, lexical, and structured retrieval.
- Phrase queries, positions, snippets, and debug ranking explanations.
- Unicode/ascii-folding/stemming/stopword tokenizer evolution.
- Competitive hybrid-search claims.
