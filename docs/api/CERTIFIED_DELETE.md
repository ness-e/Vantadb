---
title: Certified delete - purge certificates (VER-02)
kind: reference
description: "Per-surface purge certificate emitted by delete (CLI/MCP/Rust): residue inventory for store, JSON-shredded metadata, indexes and WAL tombstone; integrity hash, VER-01 chain reference and documented limits (logical vs physical purge, backups, exports)"
---

# Certified delete - purge certificates (VER-02)

> **Status:** ✅ VER-02 (2026-09-29) — implemented in `src/attestation.rs` (certificate schema v1), `src/sdk/api/memory.rs` (`delete_certified`) and the `vanta-cli delete --attest` / `vanta-cli certificate verify` commands. Certificate design under review by `vanta-audit` (P2-01).

A **purge certificate** is a JSON document emitted after a record delete. It inventories every purge surface with its evidence, states what remained (if anything), and carries an integrity hash plus a reference to the [WAL hash-chain](./WAL_INTEGRITY.md) (VER-01). It is *honest by construction*: what is not covered is listed, never silent.

## Where it is emitted

| Surface | How |
|---|---|
| CLI | `vanta-cli delete --namespace <ns> --key <k> --attest [--out <file>]` (add `--json` for `{deleted, certificate}`; `--out` writes the raw certificate file from the CLI itself — preferred on Windows, where shell redirection can mangle non-ASCII) |
| MCP | `memory_delete` with `"attest": true` → response gains `certificate` |
| Rust SDK | `Embedded::delete_certified(namespace, key) -> Result<PurgeCertificate>` |

All three go through the same core builder — no duplicated logic.

## What a certificate contains

| Field | Meaning |
|---|---|
| `schema_version` | Certificate schema (`1`, VER-02) |
| `timestamp` | ISO 8601 UTC — the single inherently variable field |
| `namespace`, `key`, `node_id` | The deleted record (node id as decimal string) |
| `reason` | Audit reason of the delete (`memory delete`) |
| `status` | `purged` (clean) · `residues` (store gone, something remained) · `not_found` (nothing to delete; keyless surfaces still scanned) |
| `surfaces[]` | Fixed-order inventory: `{surface, action, residues, evidence}` |
| `chain` | VER-01 reference: `{scheme: "sha256-prev-hash", format_version, chained, status: "referenced", verify_command}` |
| `out_of_scope[]` | Declared limits — always present |
| `integrity` | `{algorithm: "sha256", sha256: <hex>}` over the canonical body with this field emptied |

### Surfaces

| Surface | Delete action | Evidence at emission |
|---|---|---|
| `store` | `deleted` | Point-read of backend partition `Default` is absent |
| `shred` | `deleted` | `InternalMetadata` key `shred::<node_id>` absent — **JSON Shredding** column store (metadata fields), not secure-delete |
| `vector_index` | `deleted` | HNSW graph has no entry for the node |
| `vector_store` | `tombstoned` (or `none-needed`) | Vector header `FLAG_TOMBSTONE` set at delete; **physical bytes remain until segment compaction** (declared limit) |
| `derived_index` | `deleted` | `NamespaceIndex` + `PayloadIndex` keys rebuilt from the deleted record are absent |
| `text_index` | `deleted` | Posting keys for the record terms are absent |
| `sparse_index` | `deleted` (or `none-needed`) | Sparse posting keys rebuilt from the deleted record are absent |
| `version_history` | `deleted` | `Versions` partition prefix scan for the key is empty |
| `wal` | `tombstone-recorded` | `WalRecord::Delete` appended before store I/O — the append-only log keeps the trace **by design** |

When the key was not found (`status: not_found`), keyless surfaces (`store`, `shred`, `vector_index`, `version_history`) are still scanned; record-dependent surfaces are marked `not-assessed` — **never reported as clean**.

## Verifying a certificate

```
vanta-cli certificate verify --file certificate.json [--json]
```

The file may be a raw certificate or the `delete --attest --json` envelope (`{"deleted": …, "certificate": {…}}`). The verifier (offline, read-only):

1. Validates the certificate **schema** first: a known `status`, non-empty declared limits and **every canonical surface present exactly once** — a claimless or partial certificate is rejected even if its integrity hash was recomputed (verification is not claim-driven).
2. Recomputes the `sha256` over the canonical body — an edited/corrupted certificate fails.
3. Re-scans the surfaces that are re-checkable **without the deleted record**: `store`, `shred`, `vector_index`, `version_history` and the exact `NamespaceIndex` key. A surface the certificate claimed clean (`residues: 0`) that now holds an entry fails; live residues on the re-checked surfaces also fail the verdict unless the certificate is a `not_found` report.
4. Exit code: `0` valid; non-zero invalid/tampered/residues present.

**Integrity is not authenticity.** The hash detects edits and corruption; it does not prove *who* emitted the certificate — there is no engine key/signature. Cryptographic signing (external anchor) is a follow-up decision owned by `vanta-audit`, tracking the same upgrade path noted in [WAL integrity](./WAL_INTEGRITY.md). Record-derived surfaces (`PayloadIndex`/text/sparse key sets cannot be rebuilt after deletion) are covered by the integrity hash only — a documented limit.

Check output states `residues_now` in both human and JSON modes.

## Durability (cross-process verification)

`certificate verify` opens the database **read-only**, and a read-only open does not replay the WAL. The certificate reflects the issuing process's live state, so **cross-process verification requires the purge to be persisted before that process exits**:

- **CLI** `delete --attest [--out]` closes (flushes) the database before returning — the `--out` file verifies immediately afterwards.
- **Rust SDK** `Embedded::delete_certified` is flush-agnostic: call `flush()`/`close()` after it when the certificate will be verified from another process, otherwise the verifier may see a stale HNSW/tombstone state and report residues that the in-memory delete already removed.
- **MCP** `memory_delete {attest:true}` runs inside the long-lived server process: the certificate is valid against that process; external verification is meaningful after a flush (`flush` tool) or clean shutdown.

Without the flush, verification fails **safe** (reports residues), never silently passes.

## Scope and limits (declared, not silent)

The certificate covers the **live database directory**: logical entries are removed, and vector bytes are tombstoned. It does **not** claim secure erase. Every certificate lists:

- physical media: bytes may survive in storage segments, page cache, filesystem journals or SSD remap — not secure erase;
- backups and snapshots stored outside the live database directory;
- archived WAL segments (shipping archive) — the append-only log keeps history by design;
- exports — copies outside the store perimeter (the export/import sandbox, WIRE-09, governs them);
- **audit logs** — delete events (namespace/key) survive in the audit JSONL, including rotated archives, until retention drops them;
- parametric unlearning — embedding-model weights are never modified.

Two verification semantics worth knowing:

- **`not_found` reports do not remediate legacy orphans.** Scanning a key that no longer exists reports the actual state of the keyless surfaces and marks record-derived ones `not_assessed`; a shredded metadata row left behind by a pre-VER-02 delete (when the delete path did not purge it) shows up as `shred.residues: 1`. The decision is to **report it, not purge it in place** — deleting a `shred::` row for a key whose node is absent could destroy data the caller did not ask to remove. Such reports verify as `not_found` with `residues_now` visible in the output.
- **The certificate is not bound to a database instance.** It carries no per-database id/key; verification matches `namespace`/`key`/`node_id` against whichever database is opened, so pair the certificate with the database path/backup identity in your own records.

Deleting a record purges the JSON-shredded metadata row, HNSW entry, derived/payload/text/sparse index entries and version history through the same core delete path used by `delete`, `delete --attest`, `delete-by-filter`, the TTL sweeper and `purge_expired`.

## Determinism

For the same database state and inputs, every claim in the certificate is byte-identical: surfaces are emitted in fixed order, evidence strings are static and no map iteration leaks into the JSON. Only `timestamp` (and the `sha256` derived from it) varies.

## Example

```json
{
  "schema_version": 1,
  "timestamp": "2026-09-29T10:15:00Z",
  "namespace": "persona",
  "key": "alice",
  "node_id": "85620034616681451318833750766317756800",
  "reason": "memory delete",
  "status": "purged",
  "surfaces": [
    {"surface": "store", "action": "deleted", "residues": 0, "evidence": "point-read of backend partition Default is absent after the delete"},
    {"surface": "shred", "action": "deleted", "residues": 0, "evidence": "InternalMetadata key shred::<node_id> absent (JSON Shredding column store)"},
    {"surface": "wal", "action": "tombstone-recorded", "residues": 0, "evidence": "WalRecord::Delete appended before store I/O (append-only history keeps the trace by design)"}
  ],
  "chain": {"scheme": "sha256-prev-hash", "format_version": 3, "chained": true, "status": "referenced", "verify_command": "vanta-cli verify"},
  "out_of_scope": ["physical media: ... secure erase", "backups and snapshots ...", "archived WAL segments ...", "exports ...", "audit logs ...", "no parametric unlearning ..."],
  "integrity": {"algorithm": "sha256", "sha256": "…64 hex chars…"}
}
```

```bash
# Certified delete + verify, end to end:
vanta-cli delete --db ./db --namespace persona --key alice --attest --out certificate.json
vanta-cli certificate verify --db ./db --file certificate.json   # exit 0 = valid
```
