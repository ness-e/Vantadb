---
title: WAL Integrity — tamper-evident hash-chain
kind: reference
description: "Hash-chain per WAL record (VER-01): detects altered, removed or reordered frames with exact position; documented limits (suffix truncation, consistent rewrite, archived segments) and on-demand detection via vanta-cli verify"
---

# WAL Integrity — tamper-evident hash-chain

> **Status:** ✅ VER-01 (2026-09-29) — implemented in `src/wal.rs` (framing v3), `src/wal_sharded.rs` and the `vanta-cli verify` command. Design reviewed by `vanta-audit` (P2-01).

Each WAL record stores `record_hash = SHA-256(prev_hash ‖ len ‖ payload ‖ crc)`; `prev_hash` is the previous frame's `record_hash` (genesis per file/shard). No re-hash of the whole file on append: the chain is incremental, +64 B per record.

## What the chain covers

1. **Content tampering with recomputed CRC** — alter payload + recompute `crc` → `record_hash` no longer matches → `tampered` at the record's exact offset. (CRC32C alone cannot see this; the chain can.)
2. **Extirpation (record removed mid-file)** — the successor's `prev_hash` points to a `record_hash` that is gone → `tampered` (chain link broken) at the successor's offset. Insertion and reordering of whole frames are detected the same way.
3. **Corruption** (CRC/deserialization/framing broken mid-file) → `corrupt` with offset — reported as corruption, not tampering (could be crash/bit-rot).
4. **Pre-chain prefix (v1/v2)** → reported `legacy` explicitly, never treated as tampering.

## What it does NOT cover (documented limits)

- **Suffix truncation at a clean record boundary**: removing the last N records is not distinguishable without an external anchor (no signature/key). Partial multi-shard mitigation: `verify_shard_counts` (ERR-011) flags short shards only when the round-robin count pattern breaks — a truncation that keeps counts coherent yields no signal. Real limit, not a guarantee.
- **Consistent rewrite — of the whole file *or of a suffix*** (attacker recomputes every link): indistinguishable without an external anchor/signature. Same upgrade path.
- **Header bytes** are not inside the chain (they carry their own CRC32C); a version flip forces rejection via `validate_compat`.
- **Archived segments not on disk**: there is no segment manifest → a wholly deleted segment is undetectable. Also: **a directory with no WAL files verifies green (`ok: true`, `shards: []`)** — the expected shard count is not attestable without a manifest.
- **Performance**: SHA-256 per frame; the ≤5 % write-cost gate holds on the durability (fsync) path; buffered short-record paths pay more (tracked as a FIND for `vanta-tuner`).

## Recovery invariance

Recovery does **not** validate the chain (keeps its CRC + scan-forward + quarantine semantics), and replaying a tampered record on open is by design. **Detection is on-demand, not automatic at open**: `vanta-cli verify` is the authoritative detector — offline, read-only, and it never opens the engine.

## `vanta-cli verify`

```
vanta-cli verify [--json]
```

- Walks every WAL shard, recomputes the chain and reports the **first** offending frame with its exact position.
- Exit code ≠ 0 when integrity fails; `--json` emits a machine-readable report, e.g.:

```json
{ "status": "tampered", "offset": 20, "record": 1, "reason": "chain link broken (record removed, inserted or reordered)" }
```

- Statuses: `verified` · `tampered` · `corrupt` · `legacy` (pre-chain prefix) · `incomplete_tail` (torn tail — not treated as tampering).

## Upgrade path

External anchor / signature of the chain head (attestation with an engine key) — deferred follow-up owned by `vanta-audit` (plan L941). Cross-segment chaining is out of scope (round-robin sharded WAL has no byte-global order).
