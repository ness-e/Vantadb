# Persistent Text Index Design

Date: 2026-05-04

## Purpose

This design defines the internal persistent text-index contract needed before
BM25/RRF. It is not a public hybrid-search claim and does not change
`search_memory(text_query=...)`, which still returns the explicit deferred
error.

## Tokenization

- Tokenizer: `lowercase-ascii-alnum`.
- Split on non-ASCII-alphanumeric characters.
- Lowercase ASCII tokens.
- Preserve numeric tokens.
- Do not apply stemming, stop words, language-specific normalization, or scoring in this stage.

## Internal Key Shape

Posting keys use:

`namespace + NUL + token + NUL + key`

The canonical memory record remains addressed by `namespace + key`; the text index only maps textual terms back to canonical records.

Posting values store the canonical `node_id` as little-endian bytes, matching
the existing derived namespace/payload index convention.

## Storage

- `text_index` is a dedicated backend partition/keyspace/column family.
- `InternalMetadata` stores a separate text-index state marker.
- The marker includes schema version, tokenizer name, key format, canonical
  record count, and posting count.

## Mutation and Rebuild Model

The text index is derived from canonical memory records, same as ANN and
namespace/payload indexes.

- `put`/update deletes previous payload postings and writes current payload
  postings in the same backend batch used for derived namespace/payload index
  maintenance.
- `delete` removes postings for the previous canonical record.
- Rebuild scans canonical records, tokenizes `payload`, deletes existing text
  postings, writes fresh postings, and updates the text-index state marker.
- `open` validates the marker and counts. Writable opens rebuild from canonical
  records when the marker is missing, corrupt, incompatible, or count-stale.
- Import/export does not serialize text-index internals; imported records keep
  the ability to reconstruct the text index from canonical payloads.

Repeated tokens within one payload create one posting per `namespace/token/key`.

## Deferred Work

- BM25 scoring.
- RRF fusion.
- Query planner.
- Debug ranking output.
- Multi-language tokenization.
- Public text-search API.

## Current Repo State

`src/text_index.rs` contains the tokenizer, posting-key shape, unique-token
deduplication, and backend write helpers. The SDK wires those helpers into
derived-index mutation, rebuild, repair-on-open, import, and operational
metrics. It is intentionally not connected to public text search yet.
