---
title: "Memory interchange format (JSONL v2)"
kind: reference
status: active
description: "The JSONL v2 contract shared by VantaDB export, import and the third-party importers (Mem0, Zep/Graphiti, Letta), with mapping tables and declared limits"
tags: [api, memory, import]
---

# Memory interchange format (JSONL v2)

> **Canonical reference.** One line per memory record; the same file format is
> written by `export_all`/`export_namespace`, consumed by `import_file`,
> `vanta-cli import` and the MCP `import` tool, and emitted by the
> [importers](#importers-mem0--zep--letta).

## Line format

`MemoryExportLine` serialized as JSON, one object per line
(`vantadb/src/sdk/types/record.rs`). `schema_version` is the only required
discriminator: the importer accepts `1..=2` (v1 lines are normalized with the
backfill defaults below) and rejects anything newer with
`unsupported memory export schema_version`.

| Field | Type | Notes |
|-------|------|-------|
| `schema_version` | u32 | `2` is what export writes and what the importers emit; `1` accepted (normalized). |
| `namespace` | string | `A-Z a-z 0-9 . _ / -`, ≤128 bytes. Scopes the key. |
| `key` | string | Unique per namespace, ≤512 bytes, no NUL. The deterministic node id is `xxh3_128(namespace, key)`. |
| `payload` | string | The memory text. Empty payloads are legal but discouraged. |
| `metadata` | object | `Fields = { string → Value }` with **externally tagged** values: `{"String":"x"}`, `{"Int":7}`, `{"Bool":true}`, `{"Float":1.5}`, `{"ListString":["a"]}`, `{"ListInt":[…]}`, `{"ListDateTime":[…]}`, `{"Null":null}`. Keys starting with `__vanta_` are reserved. |
| `vector` | `[f32]?` | Dense embedding, if any. Export files of the supported sources carry none. |
| `sparse_vector` | object? | Sparse embedding (ADR-0019). |
| `created_at_ms`, `updated_at_ms` | u64 | Unix milliseconds. `0` is the documented "unknown" sentinel. |
| `version` | u64 | Monotonic per key; `import` preserves the transported value. |
| `expires_at_ms` | u64? | TTL deadline. |
| `superseded_by`, `superseded_at_ms` | ? | Explicit supersession link (ADR-0028): the old record is soft-dead, recoverable. |
| `valid_at_ms`, `invalid_at_ms` | u64, u64? | **Valid time** window (ADR-0046 §D3): since when the claim holds, and until when. |
| `confidence_class` | enum | `"Asserted"` (direct claim) or `"Derived"` (computed from parents). |
| `confidence` | f32 | `[0,1]`. Default `1.0` (D_a); derived records discount their parents (`0.9`). |
| `last_validated_at_ms` | u64? | Last successful re-validation. |
| `derived_from` | `[string]` | Parent **keys** (same namespace). Non-empty iff `Derived`. |
| `quarantined_at_ms`, `quarantine_reason`, `quarantined_by`, `quarantine_review_due_ms` | ? | Quarantine state (MGR-13); import with `quarantine: true` marks records that arrive without state. |

## Semantics

### Temporal

- `created_at_ms`/`updated_at_ms` are **transaction time** (when VantaDB learned
  the record); `valid_at_ms`/`invalid_at_ms` are **valid time** (when the claim
  was true). Queries can ask for the state at a valid-time point (`as_of_ms`,
  SCH-03).
- v1 normalization on import: `valid_at = created_at`,
  `invalid_at = superseded_at` when superseded, `confidence_class = Asserted`,
  `confidence = 1.0`, `last_validated = None`, `derived_from = []`, no
  quarantine. Export always writes v2.
- `valid_at > invalid_at` is rejected. The transport deliberately tolerates
  `invalid_at != superseded_at` (the future retroactive setter may diverge
  them) and preserves both verbatim.

### Dedup and conflict

- A record is identified by `(namespace, key)`; the node id is a deterministic
  hash of both. Importing the same key again **updates** — it never duplicates.
- Conflicting content for the same key is last-write-wins on import; to keep
  history, write the new claim under a new key and link the old one with
  `superseded_by` (both records survive).
- The raw transport keeps arrival-order LWW for third-party files. Multi-writer
  convergence between writers (devices/agents) has a **declared policy** in the
  merge path: `Embedded::merge_record` resolves by explicit LWW over
  `(updated_at_ms, canonical content bytes)` with conflict detection -
  deterministic for any arrival order (ADR-0055).
- `Derived` records declare ≥1 `derived_from` parent key and a `confidence`
  bounded by the parents' (`min(parents) × 0.9` when computed by the engine).
  The transport validates shape (non-empty parents, class consistency,
  `[0,1]` range) — not parent existence.

### Import behavior

`import_file` reads JSONL line by line; empty lines are `skipped`, malformed
lines increment `errors` and do not abort the run. `ImportReport` returns
`inserted`, `updated`, `skipped`, `errors`, `quarantined`, `duration_ms`.
Import goes through the raw transport (`put_record_exact`), which re-validates
the confidence boundary and the validity window. `import_records` and
`import_file` are audited (`import_records`/`import_file` audit events).

### Limits of what is preservable

- **No transaction-time end.** Zep's `expired_at` (when the system learned the
  fact was no longer valid) has no field; only the valid-time window is kept.
- **Nested values.** Object/array metadata from third-party exports is stored
  as a JSON **string** (arrays of strings become `ListString` so they stay
  filterable).
- **No credentials, no network.** Importers read export files only; they never
  call the source system's API.
- **Vectors.** None of the supported export files carry embeddings; attach
  `vector` per line before importing if semantic search is required.

## Importers (Mem0 / Zep / Letta)

`vantadb::sdk::importers::{mem0, zep, letta}` — each exposes
`convert_str(&str)` / `convert_file(path)`. A conversion is pure: it returns
`Conversion { lines, stats }` where `stats` counts converted records, skipped
records (no textual payload) and **explicit discards** (source field →
occurrences). Shared normalization: timestamps accept RFC 3339, date-only
(UTC midnight) and epoch seconds/milliseconds/nanoseconds; a missing
`created_at` becomes the `0` sentinel, a missing `updated_at` becomes
`created_at`, an inverted pair keeps `created_at`, and `valid_at_ms` is
materialized to `created_at_ms` on conversion so the export roundtrip is
byte-stable. Then either:

```rust
// Import directly (validates through the canonical transport):
db.import_records(conv.into_records()?, /* quarantine */ false)?;

// Or write the interchange file and use the CLI / any binding:
conv.write_jsonl("vantadb-memory.jsonl")?;
// $ vanta-cli import --in vantadb-memory.jsonl
```

`write_jsonl` is a plain file helper — it writes exactly the path you give it
(no sandbox). The engine's own import/export paths apply
`Config::export_base_dir` / `..`-traversal protection
(`resolve_export_path`, WIRE-09) once the file is fed through
`import_file`/`vanta-cli import`.

Counters are **record-level**. Envelope-level transport metadata is not
imported and not counted; each source lists what it ignores at that level:
Mem0 wrapper keys of a `{results|data|memories}` envelope (`status`, `command`,
`count`, `scope`, `duration_ms`, …); Zep root keys other than
`user_id`/`graph_id`/`episodes`/`facts`/`edges` (artifact groups such as `nodes`/`communities`/`observations`/`*_summaries` are **counted** — see Zep table); Letta root `metadata` and
`created_at`. Droppable *data* at envelope level is counted (Zep `edges` when
`facts` is also present).

### Mem0

Accepts a JSON array, `{"results":[…]}` (OSS `get_all`) or `{"data":[…]}` (CLI
`--agent` envelope).

| Mem0 field | Destination |
|---|---|
| `memory` (alt: `text`, `content`, `data`) | `payload` (required; record skipped without it) |
| `id` (alt: `memory_id`, `uuid`) | `key`, `metadata.source_id`; without id → deterministic `mem0-<xxh3>` content key |
| `user_id`, `agent_id`, `run_id`, `app_id` | `metadata` (same names) + namespace scope `mem0/<scope>` |
| `created_at`, `updated_at` | `created_at_ms`, `updated_at_ms` (RFC 3339, date-only, epoch s/ms/ns) |
| `metadata` (object) | `metadata."mem0.metadata"` (JSON string) |
| `categories` | `metadata."mem0.categories"` (`ListString`) |
| `hash` | `metadata."mem0.hash"` |
| other fields | `metadata."mem0.<field>"` |
| `score`, `relevance`, `selection_rank` | **discarded** (retrieval-time) |

`confidence_class = Asserted`, whose confidence is `1.0`.

### Zep / Graphiti

Accepts `{"user_id"|"graph_id", "episodes":[…], "facts":[…]|"edges":[…]}` (bulk
list-method dumps). Namespace `zep/<scope>`.

| Zep field | Destination |
|---|---|
| episode `uuid`, `content`, `created_at` | `key`, `payload`, `created_at_ms` (asserted) |
| episode `source`, `source_description`, `role`, `role_type`, `thread_id`, `document_id`, `metadata` | `metadata."zep.*"` |
| fact `uuid`, `fact`, `created_at` | `key`, `payload`, `created_at_ms` |
| fact `valid_at`, `invalid_at` | `valid_at_ms`, `invalid_at_ms` |
| fact `episodes` | `derived_from` (parent keys) + `confidence_class = Derived` (`confidence = 0.9`); parents are filtered to the episodes actually converted (skipped/absent uuids are counted as `fact.episodes_unresolved`); a fact without resolvable parents → declared downgrade to `Asserted` (`fact.episodes_missing` when the source list was empty) |
| fact `name`, node uuids/names/labels, `attributes`, `scope`, `hyperedge_uuid` | `metadata."zep.*"` |
| `expired_at` | **discarded** (transaction time; no v2 field) |
| `score`, `relevance`, `selection_rank` | **discarded** |
| `edges` when `facts` is also present | **discarded** (the `facts` array wins) — counted as `edges` |
| root `nodes`, `communities`, `observations`, `*_summaries` | not imported (counted) |

### Letta (`.af`)

Accepts a Letta Agent File. Blocks → `letta/blocks` (one record per block, its
`letta.agents` metadata lists every agent referencing it); message history →
`letta/<agent>/messages`. Namespace components are sanitized to the namespace
charset.

| `.af` | Destination |
|---|---|
| `blocks[].value` | `payload`; key = block id (fallback: label, then content hash) |
| `blocks[].label`, `description`, `limit`, `read_only`, `metadata` | `metadata."letta.label" | "letta.description" | "letta.limit" | "letta.read_only" | "letta.block_metadata"` |
| `agents[].messages[].content` | `payload`: a string, or an array of content parts — text parts are joined (`\n`), non-text parts are counted (`message.content_parts`); empty or text-less arrays (tool-call messages, real `.af` shape `content: []`) are skipped |
| `agents[].messages[].id`, `created_at`, `role`, `type`, `model`, `in_context` | `key`, `created_at_ms`, `metadata."letta.*"` |
| agent `name` | namespace component + `metadata."letta.agent"`; key fallback = content hash |
| agent `system`, `llm_config`, `embedding_config`, tool rules, … | not imported (agent configuration, counted as `agent.*`) |
| `files`, `sources`, `tools`, `groups`, `mcp_servers` | not imported (counted) |
| block/message other fields | `metadata."letta.<field>"` |

Known upstream limits (declared, not silently dropped): `.af` does not carry
archival-memory passages nor data-source file contents, and exported secrets are
`null` by design.

## Determinism and idempotency

- Conversion output is deterministic: same input file → identical lines.
- Source ids become keys; id-less records get
  `xxh3_128(namespace, payload)` content keys. Re-importing the same export
  therefore **updates** records (`ImportReport.updated`) instead of duplicating
  them — the same content-hash idempotency used by the Markdown seed importer.

## See also

- [Migrating from Mem0](../user/tutorials/migrating-from-mem0.md)
- [Migrating from Zep / Graphiti](../user/tutorials/migrating-from-zep.md)
- [Migrating from Letta](../user/tutorials/migrating-from-letta.md)
- [Embedded SDK reference](./EMBEDDED_SDK.md#export--import)
