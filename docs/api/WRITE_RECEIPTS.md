---
title: Write receipts (VER-10) — attestation of writes
kind: reference
description: "Verifiable write receipts emitted by put_certified (Rust SDK): content binding (sha256), VER-01 chain reference, canonical surfaces and declared limits; point-in-time verification via schema, integrity hash and live re-scan"
---

# Write receipts (VER-10) — attestation of writes

> **Status:** ✅ VER-10 (2026-10-05) — implemented in `src/attestation.rs` (receipt schema v1) and `src/sdk/api/memory.rs` (`put_certified`, `verify_write_receipt`). The delete-side [purge certificates](./CERTIFIED_DELETE.md) (VER-02) are unchanged.

A **write receipt** is a JSON document emitted by a certified write. It binds the written record's content with an sha256, references the [WAL hash-chain](./WAL_INTEGRITY.md) (VER-01) and carries declared limits — the same *honest by construction* contract as the purge certificates. It is **opt-in**: plain `put` pays nothing.

## Where it is emitted

| Surface | How |
|---|---|
| Rust SDK | `Embedded::put_certified(input) -> Result<(MemoryRecord, WriteReceipt)>` |
| CLI / MCP | follow-up (tracked in the [Backlog](../dev/Backlog.md) as FIND-308) |

## What a receipt contains

| Field | Meaning |
|---|---|
| `schema_version` | Receipt schema (`1`, VER-10) |
| `timestamp` | ISO 8601 UTC — the single inherently variable field |
| `namespace`, `key`, `node_id` | The written record (node id as decimal string) |
| `version` | Record version at write time |
| `status` | `recorded` — the only status a write receipt carries |
| `surfaces[]` | Fixed-order inventory: `{surface, action, residues, evidence}` |
| `content` | Content binding: `{algorithm: "sha256", sha256}` over the canonical record projection |
| `chain` | VER-01 reference: `{scheme: "sha256-prev-hash", format_version, chained, status: "referenced", verify_command}` |
| `out_of_scope[]` | Declared limits — always present |
| `integrity` | `{algorithm: "sha256", sha256}` over the canonical body with this field emptied |

### Surfaces

| Surface | Action | Evidence at emission |
|---|---|---|
| `store` | `written` | The write path applied the node (WAL frame + KV + HNSW); verification re-reads partition `Default` and recomputes the content binding |
| `wal` | `frame-recorded` | The write's WAL frame is appended before store I/O — the append-only log keeps the trace **by design** |

### Content binding

`content.sha256` covers the canonical projection of the record: `namespace`, `key`, `node_id`, `version`, `payload`, `metadata`, the dense and sparse vectors and the TTL (`expires_at_ms`). Deterministic across processes: `metadata` and `sparse_vector` are sorted maps and the projection field order is fixed.

**Only the projection fields are covered** — every other record field is out of scope, including the validity window (`valid_at_ms`/`invalid_at_ms`), lineage (`derived_from`), provenance class (`confidence_class`), `last_validated_at_ms`, system timestamps (`created_at_ms`/`updated_at_ms`), quarantine state and derived index representations. A later operation that only touches out-of-scope state (e.g. a confidence reinforcement) does not invalidate the receipt. The boundary is declared in `out_of_scope`, never silent.

**Finite floats only**: the projection serializes floats through JSON, where `NaN`/`±Inf` collapse to `null` (non-injective). Certified writes reject a non-finite float in `metadata`/`vector`/`sparse_vector` **before** committing (`Error::Validation`, nothing persisted), and verification fails safe on a live record that carries one — the binding is injective over everything it actually binds.

## Verifying a receipt

```
Embedded::verify_write_receipt(receipt_json) -> Result<WriteReceiptVerification>
```

The verifier (read-only, against the live database handle):

1. Validates the receipt: the schema version, the `sha256` over the canonical body (an edited/corrupted receipt fails) and the structural contract — known `status`, non-empty declared limits and every canonical surface present exactly once (a claimless receipt is rejected even if its integrity hash was recomputed; verification is not claim-driven).
2. Re-reads the record and recomputes its content binding — the record must still be present with the same namespace/key, version and content, and carry finite floats (a non-finite value fails safe).
3. **Fails by design** when the record is gone (deleted or expired since emission) or changed (a re-put bumps the version) — a write receipt is a **point-in-time attestation**, not a history walker.

A successful verification echoes `integrity: "ok"` and `content: "match"` with the attested `version`.

## Durability (cross-process verification)

Same caveat class as the purge certificates: the receipt reflects the issuing process's live state. Call `flush()`/`close()` after `put_certified` when the receipt will be verified from **another process** — otherwise the verifier may see a stale state and fail safe (never silently pass).

## Scope and limits (declared, not silent)

Every receipt lists its limits; the summary:

- **Chain reference, not chain re-verification** — `vanta-cli verify` is the chain authority; crash durability before WAL sync/segment rotation is the WAL's domain.
- **Point-in-time** — verification fails after an update or delete, by design.
- **Content scope** — only the projection fields are covered; validity window, lineage, provenance class, `last_validated_at_ms`, system timestamps, quarantine state and derived index representations are not (see above).
- **Finite floats only** — non-finite floats are rejected before a certified write commits and fail safe at verification (the JSON binding would be ambiguous).
- **No digital signature** — integrity is an sha256 self-hash plus the live re-scan; an actor who recomputes the hashes is not detected (same class as VER-02; cryptographic signing is a `vanta-audit` decision).
- **Not bound to a database instance** — verification matches `namespace`/`key` against whichever database is opened.
- **Expired records read as absent** — the read path treats a passed deadline as gone; verification reports it as not present.
- **Audit logs** — write events (namespace/key) survive in the audit JSONL until retention drops them.
- **The WAL frame is not cited by hash** — `append` does not expose its per-frame `record_hash` to the write path today (upgrade path tracked in the [Backlog](../dev/Backlog.md) as FIND-307); the chain reference plus the content binding are the equivalent evidence.

## Example

```json
{
  "schema_version": 1,
  "timestamp": "2026-10-05T12:00:00.000Z",
  "namespace": "persona",
  "key": "alice",
  "node_id": "85620034616681451318833750766317756800",
  "version": 1,
  "status": "recorded",
  "surfaces": [
    {"surface": "store", "action": "written", "residues": 0, "evidence": "the write path applied the node (WAL frame + KV + HNSW); verification re-reads partition Default and recomputes the content binding"},
    {"surface": "wal", "action": "frame-recorded", "residues": 0, "evidence": "the write's WAL frame is appended before store I/O (append-only history keeps the trace by design); `vanta-cli verify` is the chain authority"}
  ],
  "content": {"algorithm": "sha256", "sha256": "…64 hex chars…"},
  "chain": {"scheme": "sha256-prev-hash", "format_version": 3, "chained": true, "status": "referenced", "verify_command": "vanta-cli verify"},
  "out_of_scope": ["…declared limits…"],
  "integrity": {"algorithm": "sha256", "sha256": "…64 hex chars…"}
}
```

```rust
use vantadb::sdk::{Embedded, MemoryInput};

// Certified write: the receipt is built from the persisted record.
let (_record, receipt) = db.put_certified(MemoryInput::new(
    "persona",
    "alice",
    "prefers dark mode",
))?;
std::fs::write("write-receipt.json", serde_json::to_string_pretty(&receipt)?)?;

// Later (same or another process — flush/close first for cross-process):
let json = std::fs::read_to_string("write-receipt.json")?;
let verification = db.verify_write_receipt(&json)?;
assert_eq!(verification.content, "match");
```

## Determinism

For the same database state and inputs, every claim in the receipt is byte-identical: surfaces are emitted in fixed order, evidence strings are static and the content projection is sorted. Only `timestamp` (and the sha256 derived from it) varies.
