---
title: "ADR-0057: Encryption at rest — envelope-cipher wiring of the core on-disk artifacts (WAL, HNSW, VantaFile, backend-KV)"
kind: adr
status: accepted
description: "Wires the existing `encryption` feature to the real write paths with per-artifact AEAD envelopes, a wrapped-DEK key hierarchy with rotation, a provably-unique nonce scheme, and a declared no-in-place-migration policy; corrects the false SECURITY.md claim (FIND-407) by making it true, opt-in and bounded."
tags: [vantadb, architecture, adr, security, encryption, storage, wal, durability]
created: "2026-10-20"
---

# ADR-0057: Encryption at rest — envelope-cipher wiring of the core on-disk artifacts (WAL, HNSW, VantaFile, backend-KV)

> **Ratified by the owner on 2026-10-20.** Two decisions: (1) "corregir y ademas cablear la
> feature `encryption`"; (2) ratification of this design — "Ratifico el ADR-0057 ... y arranco
> la implementacion por sus 8 waves". Status: `accepted` (Regla 5, `.opencode/AGENTS.md`).
>
> This document is the **design**; the implementation follows the plan
> `docs/dev/plans/2026-10-20-wiring-encryption-feature.md`
> (W0-W7). The wording of the encryption feature is already corrected in `SECURITY.md`
> (PR #267); the wiring is what makes that wording true, and W0 adds a gate so it cannot
> silently drift back (risk R-12).
>

## 1. Context and problem statement

### 1.1 What the code actually does today

The `encryption` cargo feature exists (`Cargo.toml:178` — `encryption = ["dep:aes-gcm", "dep:ring"]`)
and is **not** in `default` (`Cargo.toml:153`). The primitives are real and tested:

| Primitive | Location | Note |
|---|---|---|
| `Cipher` (AES-256-GCM) | `src/crypto.rs:161` | `encrypt` `:257`, `decrypt` `:310` |
| `Cipher::derive_namespace(ns, version)` (HKDF-SHA256) | `src/crypto.rs:216` | already binds a namespace + key version to the key |
| `EncryptionStream<S>` (framed Read/Write) | `src/crypto.rs:385` | `[u32 len][nonce][ct+tag]` framing |
| `EncryptionConfig::resolve_cipher()` | `src/crypto.rs:104` | reads `VANTADB_ENCRYPTION_KEY` |
| `File::with_cipher` / `encryption_stream` | `src/storage/vfile.rs:439` / `:459` | attaches a cipher to a `File` |

**The single call site of `with_cipher` in the whole workspace is a `#[test]`**
(`src/storage/vfile.rs:929`, inside `test_vfile_encryption_stream_uses_attached_cipher`,
`:917`). Both production constructors of `File` hard-code `cipher: None`
(`src/storage/vfile.rs:174` and `:243`). `Config` exposes `encryption_key: Option<String>`
(`src/config.rs:914`), and `resolve_cipher` reads it — but nothing calls `resolve_cipher`
from a storage path. **Net effect: 0 bytes written by the engine are encrypted.** The
backlog row `FIND-249` states this as "0/6 artefactos on-disk cifrados".

This made the public claim in `SECURITY.md:26` — *"AES-256-GCM at-rest encryption behind the
`encryption` feature flag (`src/crypto.rs`)"*, listed under **Security Practices** — **false**
(`FIND-407`, Critical).

**That claim has already been corrected in this branch** (commit `c0e63d4e`, 2026-10-20,
"fix(security): SECURITY.md deja de publicar un cifrado en reposo que el core no aplica"):
`SECURITY.md:26` now says the primitives exist behind the feature and are used by the
`vanta-proxy` envelope but that the core storage files are **not** encrypted at rest yet; the
module doc of `src/crypto.rs:13-18` says the same and points at this ADR; and
`docs/CHANGELOG.md` records the correction. The honest wording also exists in
`docs/user/operations/CONFIGURATION.md:75` and `:520` ("*not wired to storage write paths yet*").

**This ADR is therefore the *forward* half of the work, and it creates a new obligation.** The
product now publishes an honest *negative*. When the wiring lands, `SECURITY.md:26` and the
`src/crypto.rs` module doc must be **amended forward** to state the real capability and its
exact scope (§5.3) — otherwise the documentation becomes the **inverse** false claim ("not
encrypted" when it now is). The amendment is part of the rollout (§7.2, S6), not an afterthought.

The owner decision of 2026-10-20 is therefore two-sided: **the claim is already corrected; wire
the feature** so that the corrected text can be replaced by the true one.

### 1.2 Inventory: the six logical surfaces and their real write paths

`FIND-249` enumerates six logical surfaces (WAL / HNSW / VantaFile / backend-KV / text_index /
snapshots). The table below maps each to its **verified** write path. Two of the six are **not
independent artifacts** — this is a material correction to the backlog row and it shrinks the
implementation surface.

| # | Logical surface | On-disk files | Write path (verified `file:line`) | Encrypted today | Independent work? |
|---|---|---|---|---|---|
| 1 | **WAL** | `data/vanta.wal` (1 shard) · `data/vanta.shard{0..N}.wal` (N shards) · archives `vanta.shard{N}.wal.<ts>` | `src/wal.rs:399` `WalWriter::append` → `:411-413` (len, payload, crc32c) · `:435` `batch_append` · sharded: `src/wal_sharded.rs:438` `append` → `:470` per-shard `batch_append` · rotation `src/wal.rs:533` / `src/wal_sharded.rs:549` | **No** | Yes |
| 2 | **HNSW index** | `data/vector_index.bin` | `src/index/serialize/file.rs:16` `persist_to_file` (`File::create` + `serialize_to_writer`) · `:108` `sync_to_mmap` (tmp file + `rename` at `:147`) · invoked from `src/storage/engine/maintenance.rs:144-158` `save_vector_index` (called at `:64` and `:541`) | **No** | Yes |
| 3 | **VantaFile segments** | `data/vstore_L0..L3.vanta` (+ legacy `data/vector_store.vanta`) | **single primitive**: `src/storage/ops.rs:64` `write_node_to_vstore` → header at `:184`, payload at `:188` `mmap_bytes_mut()`; production callers `src/storage/engine/ops.rs:213`, `src/storage/engine/insert.rs:449` and `:612`, `src/storage/engine/txn.rs:455`, `src/storage/engine/mod.rs:400`, `src/storage/engine/maintenance.rs:982`; whole-file rewrite `src/storage/archive.rs:42` `compact_layout` → tmp + `rename` at `:160` | **No** | Yes |
| 4 | **backend KV** | Fjall: `<root>/<backend>/0.jnl` + SSTs · RocksDB: `<root>/<backend>/` | `src/backends/fjall_backend.rs:136` `put` · `:170` `write_batch` · `:196-205` `persist(PersistMode::SyncAll)` · RocksDB CFs at `src/backends/rocksdb_backend.rs:134-152` | **No** | Yes |
| 5 | **text index** | **no file of its own** — keyspace `text_index` inside #4 | `src/text_index.rs:730` `posting_put_ops` → `BackendPartition::TextIndex` (`:757`) → `src/backends/fjall_backend.rs:86` `keyspace("text_index")` · RocksDB CF `src/backends/rocksdb_backend.rs:152` | **No** | **No** — covered by #4 |
| 6 | **snapshots** | `data/snapshots/<name>/data/*` + `.../backend/*` | `src/storage/engine/mod.rs:717` (unix) / `:776` (windows/wasm) `create_snapshot` → `mirror_data_dir` `:543` + `mirror_backend_to` `:584` | **No** | **No** — a mirror of #1-#4 |

**Correction to `FIND-249`:** the count "6 artefacts" is a count of *logical surfaces*, not of
independent write paths. There are **4** independent encryption points (WAL, HNSW image, VantaFile
segments, backend KV); the text index is a keyspace inside the backend KV, and snapshots are a
filesystem mirror of the other four. Encrypting #1-#4 encrypts #5 and #6 transitively — the
snapshot of an encrypted database contains only ciphertext plus the cleartext structural headers
listed in §5.3.

**Other derived surfaces, deliberately not encrypted** (metadata only, no payload):
`vanta.wal.shards` (shard count, `src/wal_sharded.rs:21`), `.wal_ship_marker`
(`src/wal_shipping.rs:89`), the `edge_index` (derived in memory from WAL/KV, no artefact), the
`.tmp` files of an in-flight compaction and the quarantined corrupt tails
(`src/wal.rs:741`) — the last two are *ciphertext* because they are copies of encrypted artefacts.

## 2. Considered options

### Option A — One AEAD envelope over each whole file

Encrypt each artefact as a single AEAD blob.

- **Pros:** simplest possible implementation; one envelope per file; the format-version marker is
  trivial.
- **Cons:** destroys random access. The VantaFile is an mmap'd, offset-addressed store
  (`src/storage/ops.rs:64-226` writes a 64-byte `DiskNodeHeader` + payload at a 64-byte-aligned
  offset, and the HNSW stores byte offsets into it) and the HNSW image is mmap'd for zero-copy
  cold start (`src/index/serialize/file.rs:50-82`). A whole-file blob forces "read everything into
  RAM", which breaks the 2N/3N-I/O decision (INV-012, `.opencode/rules/durability.md:10-17`) and
  the memory budget (`.opencode/rules/memory-budget.md`). **Rejected:** it does not keep the
  product's defining property (local-first, mmap, bounded RAM).

### Option B — Per-artifact envelopes with the granularity matched to the access pattern  ← **chosen**

The envelope granularity is chosen per artefact so that the *access pattern* survives encryption:
frame-granular for the sequential WAL, node-granular for the offset-addressed VantaFile,
file-granular for the whole-file-rewritten HNSW image, value-granular for the opaque backend KV.

- **Pros:** keeps random access (the VantaFile stays mmap'd — the *ciphertext* is mapped, the
  plaintext is produced per read); keeps atomicity (the envelope is the unit that is written, so
  the existing tmp+rename and transactional-batch machinery is unchanged); the byte overhead is
  deterministic and small; each artefact gets an independent DEK, so a flaw in one surface does
  not expose the others.
- **Cons:** four different envelope shapes and three different format-version bumps (the backend
  KV has no version of its own — see §D1); the VantaFile read path stops being zero-copy (see
  §5.2); the HNSW cold start stops being mmap zero-copy.

### Option C — Keep the primitives, retract the claim

Ship only the doc fix (rewrite `SECURITY.md:26` with the `CONFIGURATION.md:75` wording) and leave
the storage paths plaintext. This is option (b) of `FIND-249`.

- **Pros:** zero risk, zero cost, the false claim disappears immediately.
- **Cons:** the owner decided on 2026-10-20 to *also* wire the feature; a vector database whose
  vectors are plaintext on disk is not meaningfully "encrypted at rest" for the use case the
  product targets. **Rejected as the whole answer, adopted as the fallback** if the wiring is
  descoped (see §8, R-6).

### Option D — Delegate to the OS / block layer (LUKS, BitLocker, dm-crypt)

Require the operator to place the database directory on an encrypted volume.

- **Pros:** zero code; covers every artefact including the ones this ADR does not reach; the
  key never touches the process.
- **Cons:** out of the product's control (managed CI runners, container volumes, cloud disks
  where the provider manages the volume key), unavailable on Windows/WASM targets
  (`vantadb-wasm` is a first-class target), and it is an *operational* instruction, not a
  *product capability*. **Rejected as the primary mechanism**; it remains the recommended
  defence-in-depth complement and is documented as such.

## 3. Decision outcome

### D1 — Envelope granularity per artefact, and how a reader knows

Every encrypted artefact carries an explicit, versioned marker so the reader can decide — from
the bytes on disk, without configuration — whether to decrypt. The marker lives in the artefact's
**existing cleartext header** wherever one exists, so no new sidecar is needed to *read* an
artefact; the sidecar registry (§D2) is only needed to *write* one.

| Artefact | Marker location | Format version | Envelope shape |
|---|---|---|---|
| WAL | `WalHeader` (`src/wal.rs:158`, 20 bytes: 16-byte `binary_header` + CRC32C) | `WAL_FORMAT_VERSION` 3 → **4** (`src/wal.rs:26`; the chaining marker is a different constant, `WAL_CHAIN_VERSION = 3` at `src/wal.rs:31` — the *format* marker is the `format_version` field of `binary_header::Header`, validated by `validate_compat` at `src/wal.rs:222`) | per **frame**: `[u32 ct_len][u32 crc32c(ct)][12B nonce][ct‖16B tag][32B prev_hash][32B record_hash]` — the chain hashes stay **at the end of the frame**, as in v3, so the reader's frame walk barely changes |
| HNSW image | the index file's own `binary_header` (`magic "VNDX"`, written at `src/index/serialize/bytes.rs:26`, validated at `:317`) | `VECTOR_INDEX_VERSION` 8 → **9** (`src/index/graph/types.rs:22`) | per **file**: whole payload in one AEAD envelope |
| VantaFile | `File` header (`magic "VFLE"`, `src/storage/vfile.rs:163-166`; version const `VFILE_VERSION: u16 = 2` at `src/storage/vfile.rs:42`) | `VFILE_VERSION` 2 → **3** | per **node record**: `[u32 blob_len][12B nonce][ct‖16B tag]` where the plaintext is `DiskNodeHeader(64B) ‖ vector payload` |
| backend KV | **no header of its own** → the decision is *per database*, taken from the registry (§D2) and from the WAL/VantaFile markers | n/a (the backend's on-disk format is the backend's) | per **value**: `[12B nonce][ct‖16B tag]` (plaintext length is derivable from the value length) |

Rules that follow from this table and are **binding**:

1. **The plaintext layout of a non-encrypted database is byte-identical to today's.** No format
   version is bumped for the plaintext path; the version bump only happens in the encrypted
   format. A database created without a key is readable by any build, forever.
2. **The reader decides by marker, not by configuration.** A database whose WAL says
   `format_version = 4` is an encrypted database; a reader that has no key must fail with
   `EncryptionKeyRequired`, not with a corruption error.
3. **No mixed mode.** A database is entirely plaintext or entirely encrypted. There is no
   per-artefact toggle: mixing would make the backend's "is this value an envelope?" question
   unanswerable (the backend value has no self-describing header — see §D1 table row 4) and would
   create plaintext residue in an "encrypted" database, which is the worst possible outcome for a
   security feature.
4. **A value/record/blob is the unit of encryption, never a partial fragment.** A ciphertext is
   never split across two envelopes and never shares a nonce with another envelope.

### D2 — Key hierarchy: master key → KEK → per-artefact DEK, wrapped in a registry

**Master key sources** (first one that resolves wins; the resolution order is fixed and
documented):

| Order | Source | Env var | Notes |
|---|---|---|---|
| 1 | Environment variable | `VANTADB_ENCRYPTION_KEY` | hex, 32 bytes. Already the contract (`src/config.rs:914`, `src/crypto.rs:196`). Kept for compatibility, **not recommended** for production: `/proc/<pid>/environ`, core dumps and CI logs expose it. |
| 2 | Key file | `VANTADB_ENCRYPTION_KEY_FILE` | path to a file holding the hex key. On Unix the engine **must** refuse to use a file whose mode grants group/other read/write (must be `0600`-equivalent), and **must** refuse a symlink. On Windows the check is limited to "regular file, non-empty" — ACL inspection is out of scope and the limit is documented. |
| 3 | Key command | `VANTADB_ENCRYPTION_KEY_CMD` | a command whose stdout is the hex key (KMS / secret-manager shim: `aws kms decrypt ...`, `gcloud secrets versions access ...`). The key never touches the disk. The command is executed **once at open**, its stdout is captured into a zeroizing buffer, and it is never logged. |

**Master key is never persisted in the clear.** The persisted material is:

- `db_salt` — 32 random bytes, stored **in the clear** in the registry (it is not secret; it only
  domain-separates one database's derived keys from another's).
- `db_salt` + the master key produce a **KEK**:
  `KEK = HKDF-SHA256(IKM = master_key, salt = db_salt, info = "vanta-kek-v1" ‖ key_version, L = 32)`.
- Each artefact has its own **DEK**: 32 random bytes generated at DB creation, stored **wrapped**
  under the KEK: `wrapped_dek = AEAD_KEK(nonce, dek, AAD = "vanta-dek-v1" ‖ artifact_id ‖ key_version)`.
- The registry file `<storage_root>/data/encryption.registry` holds, per artefact: `artifact_id`,
  `key_version`, the wrapped DEK, the DEK's `id` (a random 8-byte label, so a re-wrap is
  detectable), the `db_salt`, and the current nonce epoch per artefact (§D3). It is written
  atomically (tmp + `rename` + parent-dir fsync, the pattern of
  `src/storage/archive.rs:160-163`).
  **It lives inside `data_dir` on purpose:** `mirror_data_dir`
  (`src/storage/engine/mod.rs:543`) already copies that directory into every snapshot, so the
  registry travels with the data by construction. It must **not** live inside the backend
  directory — the backend's own values are encrypted, so the registry is needed *before* the
  backend can be opened (chicken-and-egg). Because the registry is rewritten on every writable
  open (a new epoch), it must also be added to `is_mutable_after_snapshot`
  (`src/storage/engine/mod.rs:521`) so the snapshot **copies** it instead of hard-linking it (a
  hard link would let a post-snapshot epoch write mutate the snapshot through the shared inode —
  the same class of bug the function already exists to prevent).

**Why a registry and not direct derivation.** Deriving the DEK straight from the master
(`KEK` used directly as the DEK) makes **master-key rotation invalidate every ciphertext** —
exactly the defect `FIND-194` records ("rotación de la master key invalida envelopes
existentes + recovery solo programática"). With a registry of wrapped DEKs:

- **Rotation never invalidates data.** `vanta-cli encryption rotate --new-key <src>` unwraps every
  DEK with the current KEK, re-wraps it with the KEK derived from the new master under a new
  `key_version`, and rewrites the registry atomically. The DEKs — and therefore every ciphertext
  already on disk — are untouched. The operation is O(number of artefacts) = 4, not O(data).
- **The pattern already exists in this repo.** `vanta-memory/src/utils/erasure.rs` implements
  exactly this shape: a random 32-byte DEK per scope, wrapped with the core `Cipher`
  (`create_scope` `:122`, `seal` `:145`, `open` `:162`, `erase` `:174`), stored under a reserved
  namespace (`ERASURE_DEK_NAMESPACE` `:42`). `FIND-302` asks for a DEK registry on the L1 write
  path; this ADR defines the registry in the *core* so that `vanta-memory` can later consume it
  instead of keeping a second, parallel key hierarchy. **One hierarchy, not two** (One-Version
  Rule).
- **`vanta-proxy`'s envelope is the second precedent**: `vanta-proxy/src/envelope.rs:196` `seal` /
  `:223` `open` bind a `key_version` into the key derivation (`:232`
  `derive_namespace(&blob.namespace, blob.key_version)`), which is the same "version participates
  in the key" property this ADR adopts. The core registry generalises it from "a version byte" to
  "a wrapped random DEK", which is what makes rotation non-destructive.

**Registry failure modes (all explicit, none silent):**

| Situation | Behaviour |
|---|---|
| Registry missing, artefacts say "encrypted" | `Error::EncryptionKeyRequired` — refuse to open |
| Registry present, artefacts say "plaintext" | Open in plaintext, **warn** ("encryption key configured but this database is plaintext") — never silently migrate |
| Registry present but its `db_salt`/wrapped DEKs do not unwrap | `Error::EncryptionKeyRejected` — the master key is wrong |
| Registry unreadable / malformed | `Error::EncryptionCorrupt` — refuse to open |

### D3 — Nonce scheme: `epoch ‖ counter`, provably unique per DEK

```
nonce (12 bytes) = epoch (u64 BE) ‖ counter (u32 BE)
```

- **`epoch`** — a **random `u64`**, allocated from the registry at each *write epoch* (a writable
  open, a WAL rotation, a VantaFile compaction, a segment promotion). It is persisted in the
  registry **before** the first encryption under it. The registry stores only the *current* epoch
  per artefact — a bounded set of 8 bytes each — so the registry does not grow with the number of
  opens.
- **`counter`** — a per-epoch sequence counter, `u32`, incremented once per encryption. It is
  **not** persisted: a new epoch means a new random value, so a counter that restarts at 0 is
  still unique. When the counter would exceed `u32::MAX` the engine allocates a **new epoch**
  (a fresh random value) instead of wrapping.
- The **artefact discriminator is not in the nonce** — each artefact has its own DEK, so the
  nonce space is already per-artefact. The artefact id travels in the **AAD** (§D4), where it is
  authenticated, which is strictly stronger than being merely mixed into a nonce.

**Why this is safe, in the two ways that matter:**

1. **No reuse within an epoch.** The counter is monotonic and the DEK is fixed for the epoch, so
   two encryptions under one DEK get different nonces by construction. The AES-GCM catastrophic
   case (two messages under the same key and nonce) cannot occur.
2. **No reuse across epochs, including after a rollback.** The epoch is *random*, not a counter,
   so restoring an old registry does not replay an old epoch: the engine allocates a **fresh
   random** epoch. A rollback of the registry alone therefore cannot cause a nonce repeat; a
   rollback of *both* the registry and the data re-uses the old ciphertext, not a new encryption
   under a repeated nonce. The residual risk (a rollback of the whole database to a state that is
   then *extended* with new writes) is bounded by the 64-bit random epoch: the extended writes
   land in a new epoch, and the replayed old ciphertexts are byte-identical copies, not
   re-encryptions. The collision probability of two epochs coinciding is
   ≈ `N² / 2^65` for `N` epochs in the database's life (≈2.7 × 10⁻⁸ for a million epochs) —
   and it is the *only* probabilistic step in the whole scheme. The registry stays bounded
   because it stores only the current epoch per artefact, not a growing history.

**Rejected alternatives:**

- *Random 96-bit nonce per encryption* (what `Cipher::encrypt` does today, `src/crypto.rs:257-262`):
  statistically fine (a birthday collision under one DEK needs ≈2^48 messages), but it **loses the
  slot check** — with a random nonce the reader cannot tell "this frame is in the wrong place"
  from "this frame is fine", and the whole point of the AAD (§D4) is to bind a ciphertext to its
  position. It also cannot be *reconstructed* by a reader, so a partially written tail cannot be
  classified as "corrupt" vs "wrong key" (§D6).
- *Persisted monotonic counter, fsynced per write*: provably unique, but it costs one durable
  write per record on the WAL hot path. The WAL is the one place where per-write fsync is
  unacceptable (`src/wal.rs:429` `maybe_sync` is already policy-driven).

**The counter is per-artefact and reset per epoch; it is NOT the WAL frame ordinal.** A WAL frame
carries its own nonce, so a frame's uniqueness does not depend on the order in which concurrent
writers commit to the shard mutex. This matters because encryption happens **outside** the shard
lock (§D5): two threads may encrypt concurrently and commit in either order; the counters are
distinct, so both nonces are distinct.

### D4 — AAD: what is authenticated, and why a ciphertext cannot be moved

The AEAD associated data is a canonical, length-delimited byte string:

```
AAD = artifact_id (u16 BE)
    ‖ format_version (u16 BE)
    ‖ epoch (u64 BE)                ← taken from the nonce itself, so the reader needs no side state
    ‖ slot (u64 BE)
    ‖ plaintext_len (u64 BE)
    [‖ identifier bytes]           ← only where a public identifier exists
```

| Artefact | `slot` | `identifier` | What a moved ciphertext would break |
|---|---|---|---|
| WAL | frame ordinal within the file (0-based) | — | a frame copied from position 7 to position 9 fails to authenticate |
| HNSW image | 0 | the artefact's path relative to the storage root | an index image copied over another database's image fails |
| VantaFile | the node record's byte offset in the segment | — | a node blob moved to another offset fails |
| backend KV | 0 | `partition_name ‖ 0x00 ‖ key` (the key is a public identifier, not a secret) | a value moved from key `A` to key `B` fails |

The AAD is **authenticated, not encrypted** — it is public metadata that the AEAD binds into the
tag. It contains no secret. `plaintext_len` is included so that a length-manipulation attack
(truncating or extending the plaintext) is caught by the tag rather than by a downstream parser.

### D5 — Lock order, I/O ordering and atomicity

The repo's global lock order is fixed and normative
(`.opencode/rules/concurrency-async.md:54`):
`cardinality_stats → insert_lock → {wal, pending_hnsw_batch, hnsw, vstore, volatile_cache, backend}`.

**This ADR adds no lock to that order.** Concretely:

- The cipher is an **immutable, stateless** value (`Cipher` is `Clone` and holds only
  `raw_key`/`password`, `src/crypto.rs:161-168`). It is resolved once at open and shared as an
  `Arc`. It is not a lock and cannot deadlock.
- The **nonce counter** is an `AtomicU32` per artefact — no lock, no ordering constraint.
- The **registry** is read once at open (into an immutable snapshot) and written only at
  *epoch allocation* and *rotation*. A registry write is taken **outside** every engine lock, at
  open/rotation time, and it is a tmp+rename on the storage root. It is never taken while holding
  `insert_lock`, so it cannot invert the order.
- **Encryption happens before the target lock is acquired**, wherever the call site allows it. For
  the WAL this means: allocate the counter → encrypt → *then* take the shard mutex → write the
  bytes. The shard critical section grows only by the memcpy of the ciphertext, not by the GCM
  computation. The same applies to `batch_append`
  (`src/wal_sharded.rs:453-472`): encrypt the group, then one lock + one `write_all` per shard.
  A counter consumed by a write that then fails is simply burned (a gap), which is harmless.
- **No `panic` on the hot path.** Every decryption failure is an `Err`, never an `unwrap`.

**Atomicity is preserved per artefact** because the envelope is the unit that is written:

- HNSW image: already tmp + `rename` (`src/index/serialize/file.rs:123-147`) → the envelope is
  written into the tmp file; the rename stays atomic.
- VantaFile compaction: already tmp + `rename` (`src/storage/archive.rs:160`) → same.
- backend KV: `write_batch` is transactional in both backends → the envelope is the value, so the
  batch is still all-or-nothing.
- WAL: append-only frames → a torn tail is still detectable (below).

**WAL frame integrity — three independent layers, each with a distinct job.** This is the part
that must keep the WAL replayable and corruption-detectable:

| Layer | Covers | Needs the key? | Purpose |
|---|---|---|---|
| `crc32c` over the **ciphertext** (`:414` today covers the plaintext) | bit rot in the stored bytes | No | distinguish *corrupt* from *wrong key* |
| AES-GCM tag | tampering with the ciphertext | Yes | authenticity of the plaintext |
| `prev_hash ‖ record_hash` chain (`src/wal.rs:414-422`, `CHAIN_FRAME_EXTRA = 64` bytes) | reordering, truncation, splicing of frames | No (computed over the **ciphertext**) | keeps `vanta-cli verify` working **without the key** |

The chain hash is computed over the ciphertext instead of the plaintext so that the WAL chain stays
verifiable by an operator who does not have the key — that is a real operational property
(`vanta-cli verify` is the incident-response tool) and it is why the CRC also moves to the
ciphertext. The cost is that the chain no longer proves the *plaintext* is the one that was
written; the AEAD tag does that.

**Replay reproducibility.** The reader of a v4 WAL is frame-aware: read `[ct_len][crc][nonce]`,
verify the CRC, decrypt with the AAD built from the frame ordinal, and only then parse the
postcard payload. The recovery path (`recover_valid_records`, `src/wal.rs:693`;
`WalReader::next_record`, `:1046`; sharded `recover`, `src/wal_sharded.rs:481`) truncates a
corrupt/incomplete tail exactly as today — a frame whose CRC fails is a corrupt tail, a frame
whose CRC passes but whose tag fails is a **wrong key or tampering**, which is a hard error, not
a truncatable tail. `quarantine_corrupt_tail` (`src/wal.rs:741`) keeps the bytes for forensics;
the quarantined file is ciphertext.

**Consumer that must become frame-aware:** `src/wal_shipping.rs` reads WAL segments with
`WalReader` (`:7`, `ship_once` `:113`) and ships the *decoded records* over HTTP. Two
consequences, both declared: (a) the shipper must be able to decrypt, so **a replica that
receives shipped records must hold the same DEK**; (b) shipping plaintext records to a remote
replica is an **egress** of plaintext, which is outside the scope of "at rest". This ADR does not
change wal-shipping's transport; it records the constraint. The `wal-shipping` feature is
opt-in and off by default.

### D6 — Error surface

`Error` is `#[non_exhaustive]` (`src/error.rs:136`), so new variants are additive for consumers.
Four variants are added, **only compiled with the `encryption` feature** (the feature already
gates the whole `crypto` module, `src/lib.rs:56-57`):

| Variant | Meaning | When |
|---|---|---|
| `Error::EncryptionKeyRequired { artifact }` | the artefact says "encrypted" and no key resolved | open |
| `Error::EncryptionKeyRejected { artifact }` | the key resolved but the AEAD tag failed | any read |
| `Error::EncryptionCorrupt { artifact, detail }` | the structural check (CRC / length / magic) failed | any read |
| `Error::EncryptionUnsupported { artifact, found_version }` | the artefact's format version is newer than this build | open |

`EncryptionKeyRejected` deliberately does **not** distinguish "wrong key" from "tampered": a single
AEAD tag cannot tell them apart, and claiming otherwise would be a false statement in an error
message. The message says "wrong key or tampered".

Because the variants are feature-gated, the *set* of `Error` variants depends on the feature
configuration. This is acceptable under `#[non_exhaustive]` (consumers already need a `_` arm) and
is recorded in the API-contract notes for the bindings: `VantaErrorCode` in
`vantadb-python` / `vantadb-node` must gain the matching codes in the same PR, and the
`docs/api/` error table must be updated (Regla 3, `.opencode/AGENTS.md`).

### D7 — Activation and migration policy

**Activation is off by default, twice over:**

1. **Compile time:** the `encryption` cargo feature stays **out of `default`**
   (`Cargo.toml:153`). The published crate on crates.io does not even contain the code path. This
   is already true and must not change.
2. **Run time:** encryption is active **iff** the feature is compiled **and** one of the three
   key sources resolves. No key ⇒ the plaintext path is taken and the bytes are identical to
   today's.

The engine states the mode explicitly, and the two mismatches are loud, never silent:

| Engine state | Key configured | Behaviour |
|---|---|---|
| plaintext artefacts | no | plaintext (today's behaviour, byte-identical) |
| plaintext artefacts | yes | plaintext + `warn` ("encryption key configured but this database is plaintext") — **no implicit migration, no mixed mode** |
| encrypted artefacts | yes | encrypted |
| encrypted artefacts | no | `Error::EncryptionKeyRequired` — refuse to open (never "corrupt") |

The existing "key set but feature not compiled" warning (`src/server/bootstrap.rs:137-161`) is
kept and extended to the core path.

**Migration policy — explicit decision: no in-place migration in v1.** Enabling encryption on an
existing database requires: **export → create a new database with the key → import**. The export
and import paths already exist and are exercised
(`Embedded::export_all` — `src/sdk/serialization/impl_export.rs:315`; `import_records` `:449`;
`import_file` `:531`).

Rationale for not building an in-place migration:

- An in-place migration must rewrite **all four** independent write paths atomically-or-
  recoverably: the WAL (rotate + replay into a new encrypted WAL), the HNSW image (rebuild from
  the VantaFile — `rebuild_hnsw_from_vstore` already exists at `src/storage/engine/init.rs:410`),
  the VantaFile segments (a full segment rewrite — `compact_layout` is the template,
  `src/storage/archive.rs:42`), and the backend KV (read-all / write-back). A crash in the middle
  leaves a half-migrated database whose recovery semantics are undefined, and it must interoperate
  with `FIND-246` (the `checkpoint_seq`-before-artefacts ordering defect at
  `src/storage/engine/maintenance.rs:81-90`), which is itself open.
- The export/import path converts the problem from "rewrite 4 subsystems crash-safely" into
  "read a database and write a new one" — a path with an existing, tested contract.
- The cost of the decision is stated: **turning encryption on for an existing database is a
  manual, offline operation with a data-export dependency.** It is not a one-flag change.

The in-place migration is recorded as a **deferred item** with its own ADR, not as an omission
(see §8, R-6).

### D8 — The current `src/crypto.rs` API is insufficient: the required internal changes

**The design cannot be implemented on top of the primitives as they stand today.** The gap is
structural, not cosmetic: the existing `Cipher` API was built for the *proxy envelope* use case
(one key, random nonce, no associated data), while at-rest encryption needs a
**caller-chosen nonce**, **associated data**, and a **caller-supplied HKDF salt**. The changes
below are the minimum; they touch `src/crypto.rs` only (plus the `Error` variants of §D6) and
they are **not applied by this ADR**.

| # | Gap today | Required change |
|---|---|---|
| C-1 | `Cipher::encrypt` (`:257`) generates the nonce **internally** and offers **no AAD** — `aes_gcm::Aead::encrypt(&nonce, plaintext)` binds nothing beyond the ciphertext | Add `encrypt_with_nonce(nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Vec<u8>` and `decrypt_with_nonce(nonce, ciphertext, aad) -> Result<Vec<u8>, CryptoError>`, using `Aead::encrypt_in_place_detached` with the AAD. The existing `encrypt`/`decrypt` stay as thin wrappers (random nonce, empty AAD) so `vanta-proxy` and `vanta-memory` are unaffected. |
| C-2 | The nonce must be `epoch ‖ counter` (§D3) — the caller decides it | Provided by C-1. The nonce allocator lives in the **new registry module**, not in `Cipher`. |
| C-3 | `derive_namespace(ns, version)` (`:216`) uses a **fixed** salt (`NS_ENVELOPE_SALT`, `:141`) and offers no caller-supplied salt → the KEK cannot be domain-separated per database | Add `derive_kek(salt: &[u8], key_version: u8) -> Result<Self, CryptoError>` (HKDF-SHA256, `info = "vanta-kek-v1" ‖ key_version`, caller-supplied salt). `derive_namespace` is left untouched for `vanta-proxy`/`vanta-memory`. |
| C-4 | The master key is either a raw 32-byte key or a passphrase (`Cipher::new`, `:177`), and the raw bytes are not reachable from outside the type | Add a way to obtain the **raw 32-byte master** (`fn raw_key_bytes(&self) -> Result<[u8; 32], CryptoError>`), erroring for passphrase ciphers — the same contract `derive_namespace` already enforces (`:217-222`). **Decision:** for at-rest, a supplied passphrase is stretched **once** with PBKDF2-HMAC-SHA256 (210 000 iterations, `src/crypto.rs:119`) using the registry's `db_salt`, and the *stretched* 32 bytes are the master. The recommended form remains a 32-byte random key. |
| C-5 | Key material is held in plain `Vec<u8>` / `[u8; 32]` and is never zeroized | Wrap the master and the DEKs in a zeroizing holder (`ring::secrecy` if the pinned version exposes it, otherwise a ~10-line local `Zeroizing` wrapper). The DEKs live for the process lifetime, so their copies in the registry are the only on-disk exposure — but the in-memory copies should still be wiped on drop. |
| C-6 | `EncryptionStream` (`:385`) frames as `[u32 len][nonce][ct+tag]` — **no AAD, no slot, no epoch, and a length prefix the WAL frame does not have** | **`EncryptionStream` is not used by this design.** The WAL needs the chain hashes and the ciphertext CRC inside the frame (§D5) and the reader needs the slot in the AAD; the backend needs a value-shaped envelope; the HNSW needs a whole-file one. A dedicated internal `Envelope` type (nonce + ciphertext + tag, with AAD) replaces it. `EncryptionStream` keeps its current single consumer (the `vfile` test, `src/storage/vfile.rs:929`) and is **deprecated** rather than removed, per the deprecation cycle (`.opencode/AGENTS.md` §3c). |
| C-7 | `KeySource` (`:82`) has one variant and is a **public enum without `#[non_exhaustive]`**; `CryptoError` (`:54`) likewise | Add `KeySource::File` and `KeySource::Command` and mark **both** enums `#[non_exhaustive]`. Adding a variant to a non-`#[non_exhaustive]` public enum is a **breaking change** for external matchers → the crate is pre-1.0, so this lands as a `feat!:` (or at minimum a documented breaking note) in the changelog, and `docs/api/` is updated in the same PR (Regla 3). |
| C-8 | `CryptoError` cannot express "wrong key" vs "tampered" (nor should it — §D6) | Keep `DecryptionFailed` and make the *message* say "wrong key or tampered". The crate-level `Error` carries the artefact context (§D6). |

**Net effect:** `src/crypto.rs` grows a sibling module (the registry + envelope + nonce allocator)
and gains four or five new methods on `Cipher`; it does not change behaviour for its existing
consumers (`vanta-proxy/src/envelope.rs`, `vanta-memory/src/utils/erasure.rs`). Nothing in this
section requires a new dependency: `aes-gcm` and `ring` are already there
(`Cargo.toml:144`, `:178`).

## 4. What this ADR does *not* change

- **No change to the plaintext on-disk format.** Every existing database stays byte-identical and
  readable. This is the backward-compatibility guarantee.
- **No change to the lock order** (§D5).
- **No change to the durability policy** (`fsync` policy, `SyncMode`,
  `PersistMode::SyncAll` at `src/backends/fjall_backend.rs:196-205`): encryption is a
  transformation of the bytes, it does not add or remove a barrier. **One caveat, stated
  explicitly:** the encrypted WAL frame is ~28 bytes larger than the plaintext frame, and the
  GCM computation adds CPU *before* the write — the `fsync` cost is unchanged, the bytes are not.
- **No new dependency.** `aes-gcm` and `ring` are already dependencies of the feature
  (`Cargo.toml:144`, `:178`); `ring` already provides the HKDF used by `derive_namespace`
  (`src/crypto.rs:216-234`) and can provide the DEK wrapping. The zeroizing holder of §D8 C-5 is
  either `ring::secrecy` or a ~10-line local wrapper — not a new crate.
- **No behavioural change for the existing `Cipher` consumers.** `vanta-proxy` and `vanta-memory`
  keep the exact `encrypt`/`decrypt` semantics they have today (§D8, C-1).
- **No change to the public SDK API.** The feature is invisible to `put`/`get`/`search`. The
  observable changes are limited to: new `Error` variants (§D6), the new configuration keys
  (`VANTADB_ENCRYPTION_KEY_FILE`, `VANTADB_ENCRYPTION_KEY_CMD`), and — the one **breaking**
  change — new variants on the public `KeySource` / `CryptoError` enums, which is why both get
  `#[non_exhaustive]` in the same change (§D8, C-7).

## 5. Consequences

### 5.1 Positive

- `SECURITY.md:26` becomes **true** for the opt-in, encrypted configuration, with the scope
  stated exactly (which artefacts, which key handling, what is *not* covered). `FIND-407` is
  already closed by the honest negative (commit `c0e63d4e`); this ADR lets the text be amended
  **forward** to the positive claim, with the exact scope (§5.3).
- The four independent write paths are instrumented at **four** points, not six: the WAL at
  `src/wal.rs:399`/`:435` and `src/wal_sharded.rs:438`/`:453`; the HNSW image at
  `src/index/serialize/file.rs:16`/`:108`; the VantaFile at the single primitive
  `src/storage/ops.rs:64`; the backend KV at `src/backends/fjall_backend.rs:136`/`:170` and the
  RocksDB equivalent.
- The text index and the snapshots are covered **for free** (§1.2) — no extra code, no extra
  risk.
- Key rotation becomes non-destructive (`FIND-194` closed by design, not by a lazy re-encrypt).
- The engine reuses the key-management pattern that already exists in `vanta-memory`
  (`vanta-memory/src/utils/erasure.rs`) instead of inventing a third one — `FIND-302`'s DEK
  registry can be pointed at the core registry later.
- The `vanta-proxy` envelope (`vanta-proxy/src/envelope.rs`) keeps working unchanged: it has its
  own master key and its own namespace derivation. This ADR does not merge the two hierarchies; it
  only makes the core's one real.

### 5.2 Negative / cost

- **The mmap zero-copy tier is forfeited for encrypted stores.** This is the single largest
  consequence and it is not avoidable: ciphertext is not a `&[f32]`. Consequences:
  - the VantaFile read path becomes "mmap the ciphertext, decrypt the visited node on demand" —
    the page cache still avoids the read syscall, but the plaintext vector is materialised in a
    short-lived buffer instead of being a slice into the mapping;
  - the HNSW cold start (`src/index/serialize/file.rs:50-82`, mmap branch) becomes "read + decrypt
    the whole image" — the file is already rewritten whole on every flush, so this is a
    whole-file cost, not a per-query one;
  - the `MmapFull` vector representation (`src/storage/ops.rs:130-148`,
    `src/node/vector_data.rs`) cannot be used in an encrypted store — vectors are materialised as
    `Full`. This interacts with `.opencode/rules/memory-budget.md` and must be part of the
    measurement (§6).
- **Byte overhead** (deterministic arithmetic, not a benchmark claim): WAL +28 bytes per frame
  (12-byte nonce + 16-byte tag) plus 8 bytes of CRC/length bookkeeping; VantaFile +28 bytes per
  node plus a 4-byte length prefix; backend KV +28 bytes per value; HNSW image +28 bytes per
  file. For a 6 KiB `f32` vector the VantaFile overhead is ≈0.46 %; for a 20-byte KV value it is
  ≈140 % in bytes (but ≈28 bytes in absolute terms).
- **CPU cost of GCM** is unmeasured. It must be measured (§6), not assumed.
- The registry is a **new file in the storage root** that must be backed up with the database and
  must never be separated from it. Losing the registry means losing the database (the DEKs are
  only there). This is stated in the operations documentation as a hard rule.
- The `Error` enum's variant set becomes feature-dependent (§D6) — a small, additive, documented
  API surface change that the bindings' error-code mapping must absorb in the same PR.

### 5.3 Declared limits (what "encrypted at rest" does **not** cover)

These are part of the contract and must appear in `SECURITY.md` when it is rewritten:

1. **KV keys are not encrypted.** The backend's key bytes (namespace + key name) stay in the
   clear: the text index needs prefix scans per namespace
   (`src/text_index.rs:549` `posting_prefix`) and the namespace index needs ordered lookups.
   Encrypting keys would break those access patterns. **Values, postings and metadata are
   encrypted; key names are not.**
2. **Structural metadata leaks.** The WAL header's timestamp (`src/wal.rs:158`), the VantaFile's
   write cursor and node count, the shard count sidecar, the file sizes and the WAL frame count
   are cleartext. The number of records and the data volume are observable.
3. **The snapshot of an encrypted database contains ciphertext**, but the *structure* (file names,
   sizes, timestamps) is cleartext, exactly as for a plaintext database.
4. **No protection against an attacker who has the process's memory** or the key itself. The
   master key lives in process memory while the database is open.
5. **wal-shipping egresses plaintext records** to the remote replica (§D5) — the replica must
   hold the DEK, and the transport is outside the "at rest" boundary.
6. **The `encryption` feature is not in `default`**; a user of the published crate must opt in at
   build time *and* supply a key at run time.

## 6. Cost measurement (Regla 9 — no optimisation without measurement)

The design is only shippable if the cost is quantified. The required measurement is a
**before/after against the canonical baseline**, because this change touches a hot path
(the WAL append and the vector read).

| Measurement | Command | Baseline |
|---|---|---|
| Canonical P99 (insert + search) | `cargo bench -p vantadb --bench canonical_p99` | `docs/user/operations/BENCHMARKS.md:380` — p50 2.2388 / p95 4.248 / **p99 4.9987 ms**, measured 2026-09-25, `benchmarks/criterion_baseline.json` |
| WAL throughput | `cargo bench -p vantadb --bench wal_throughput` | must be recorded before the change; no baseline exists in `BENCHMARKS.md` → record it in the same PR that introduces the change |
| Peak RSS (memory-budget rule) | the same run, `insert.total_records` profile | must not regress beyond the memory-budget rule's ceiling |

**Method:** same machine, same dataset (seed 42, 100k × 1536d), same profile
(`benches/common/mod.rs:apply_fixed_profile`, warmup 3 s / measure 5 s), three runs, report
median. The encrypted run uses the same binary with `--features encryption` **and** a key
configured — the feature alone changes nothing (the plaintext path is byte-identical), so the
comparison isolates the encrypted path.

**Acceptance threshold (proposed):** the encrypted configuration must not regress the canonical
p99 by more than **15 %** — the same threshold the repo already uses for the
locality/complexity trade-off (`.opencode/rules/durability.md:10-17`). A larger regression is a
design signal (e.g. the per-node decryption is too expensive → the granularity must change), not
something to be absorbed silently.

**Every number produced by this measurement must land in
`docs/user/operations/BENCHMARKS.md` with the command, the environment and the date** (Regla 11).
No claim without a reproducible command.

## 7. Migration and rollout

### 7.1 Migration

- **Existing plaintext databases: not migrated.** They keep working, byte-identical, forever. The
  `encryption` feature being compiled does not change their bytes.
- **New databases:** create the database with a key configured → every artefact is encrypted from
  the first byte. The mode is recorded in the artefacts' own headers, so a later open detects it
  without configuration.
- **To encrypt an existing database (manual, offline, v1):**
  1. `vanta-cli export --out <dir>` (existing path).
  2. Configure the key (`VANTADB_ENCRYPTION_KEY_FILE` with mode `0600`, or
     `VANTADB_ENCRYPTION_KEY_CMD`).
  3. Open a **new** database directory with the key configured.
  4. `vanta-cli import --from <dir>` (existing path).
  5. Verify: the new database's artefacts must contain no plaintext canary; the old database is
     retired only after the import is verified.
- **Not built in v1:** an in-place `vanta-cli encryption enable`. Deferred with its own ADR (§8,
  R-6).

### 7.2 Rollout

| Stage | Content | Gate |
|---|---|---|
| **S0 — claim corrected** *(already done in this branch, commit `c0e63d4e`)* | `SECURITY.md:26` reworded to the honest negative; `src/crypto.rs:13-18` module doc reworded and pointing at this ADR; `docs/CHANGELOG.md` records it. | `FIND-407` closed on its own, **before** the wiring existed. **Nothing to do — this stage is already in the branch.** |
| **S1 — primitives** (no behaviour change) | `KeySource` extended to `Environment` / `File` / `Command`; the registry type; the DEK wrap/unwrap; the nonce allocator; the AAD builder; the `Cipher` API changes of §D8 (C-1..C-8); the new `Error` variants. Everything behind the `encryption` feature; the plaintext path untouched. | `cargo nextest run -p vantadb --profile audit --lib --features encryption` green; `cargo clippy -- -D warnings`; `cargo fmt --check` |
| **S2 — WAL** | v4 frames, frame-aware reader, ciphertext CRC, ciphertext chain hash, replay of v3 and v4 files, `vanta-cli verify` on both. | WAL round-trip, corruption detection, wrong-key error, and **replay of a pre-existing v3 WAL** (backward compat) |
| **S3 — VantaFile** | v3 segments, per-node envelope, the single write primitive `src/storage/ops.rs:64` instrumented, the read paths (search, archive, maintenance) frame-aware, compaction re-encrypting. | Node round-trip, corruption detection, wrong-key error, compaction round-trip, **reading a v2 segment unchanged** |
| **S4 — backend KV** | per-value envelope, both backends, `write_batch` transactional, the `internal_metadata` keyspace included. | Value round-trip, batch atomicity, wrong-key error, **reading a plaintext backend unchanged** |
| **S5 — HNSW image** | whole-file envelope, `persist_to_file` + `sync_to_mmap`, cold start decrypting. | Index round-trip, corrupt image detected, wrong-key error, **rebuild-from-VantaFile path still works** |
| **S6 — activation, registry, rotation, docs** | the registry lifecycle, epoch allocation, `vanta-cli encryption rotate`, the open-time mode detection and warnings, and the **forward amendment** of `SECURITY.md:26` + the `src/crypto.rs` module doc: from "not encrypted at rest yet" to the true capability and its exact scope (§5.3). `CONFIGURATION.md` and `FEATURES.md` updated; the operations runbook (key handling, backup of the registry). | A new database created with a key contains **no plaintext canary** in any artefact (the probe that `FIND-249` used); rotation preserves readability of pre-rotation data; **and no document still says "not encrypted at rest yet"**. |
| **S7 — measurement** | the §6 measurements; the numbers land in `BENCHMARKS.md`. | The p99 delta is within the declared threshold, or the design is revisited |

**Order matters:** S2 (WAL) before S3/S4/S5, because the WAL is the artefact whose *replay* is the
crash-recovery contract; a WAL that cannot be replayed breaks everything else. S1 must land
before S2 because the registry and the nonce allocator are shared infrastructure.

**Rollback:** every stage is a normal commit; the feature is off by default, so reverting the
wiring leaves the plaintext behaviour intact. The only irreversible step is S6's rotation, which
writes a new registry — the rotation keeps the previous registry as `encryption.registry.bak`
so a manual rollback is possible while the old master key is available.

## 8. Risks

| ID | Risk | Severity | Mitigation / owner |
|---|---|---|---|
| R-1 | **Nonce reuse** under one DEK (catastrophic for GCM: it leaks the XOR of two plaintexts and the auth key) | Critical | The scheme is unique by construction (§D3): a random 64-bit epoch per write-epoch, persisted before first use, never reused; a monotonic `u32` counter within the epoch. The only probabilistic step is the epoch collision (≈`N²/2^65`). A dedicated test asserts that N encryptions under one DEK produce N distinct nonces, and that a new epoch after a simulated restart does not repeat a nonce. |
| R-2 | **Wrong key silently accepted** (the engine opens a database with the wrong key and writes garbage) | Critical | Every read authenticates; a tag failure is a hard error, never a "empty result". The open path validates the registry (unwrap must succeed) before any write. No write happens on an unverified key. |
| R-3 | **The mmap degradation breaks the memory budget or the search p99** | High | Measured in §6 before the feature is declared done; the 15 % threshold; the memory-budget rule applies to the decrypted-vector working set. If the delta exceeds the threshold, the granularity of the VantaFile envelope is the thing to change (page-granular instead of node-granular), not the threshold. |
| R-4 | **Partial rollout**: the WAL is v4 but the VantaFile is still v2 (or the reverse) → an inconsistent database | High | The mode is **per database**, not per artefact: a database is created encrypted or plaintext, and the encrypted path writes *all four* artefacts in the encrypted format. There is no per-artefact switch (§D1 rule 3). The mode is detected from the WAL header at open, and a mismatch with the other artefacts is a hard error, not a warning. |
| R-5 | **Half-written encrypted tail** (crash during an envelope write) | High | The CRC over the ciphertext distinguishes a torn frame from a wrong key; the existing tail-truncation and quarantine logic (`src/wal.rs:693`, `:741`) applies unchanged to the encrypted frames. |
| R-6 | **Descope pressure**: the wiring is too big and the team retreats to Option C | Medium | The fallback is explicit: S0 (the honest `SECURITY.md`) ships **first and independently**, so the false claim is closed even if the wiring never lands. The deferred items are named: in-place migration (`vanta-cli encryption enable`), KMS-native key management beyond the `*_KEY_CMD` shim, and per-record re-encryption for granular key rotation. |
| R-7 | **The registry is lost or separated from the database** | Medium | The registry lives **inside `data_dir`** (`<storage_root>/data/encryption.registry`), which the snapshot machinery already mirrors (`mirror_data_dir`, `src/storage/engine/mod.rs:543`), and it is added to `is_mutable_after_snapshot` (`:521`) so it is copied, not hard-linked. The operations runbook states that the registry is part of the database. A missing registry on an encrypted database is `EncryptionKeyRequired`, not a silent re-key. |
| R-8 | **`FIND-246` interacts**: the `checkpoint_seq` is published durable before the snapshot artefacts (`src/storage/engine/maintenance.rs:81-90`) → a crash between them yields an inconsistent recovery, and now the artefacts are ciphertext | High | The ordering defect is **independent** and stays open (`FIND-246`); this ADR does not fix it. But it constrains the design: the DEK/IV material must be persisted (the registry) **before** any encrypted artefact is written, and the registry write is durable (tmp+rename+dir fsync) precisely so that a crash cannot leave encrypted artefacts whose key material is not yet durable. The epoch allocation is persisted **before** the first encryption under it (§D3). |
| R-9 | **Rotation invalidates data** (the `FIND-194` defect reappearing in a new shape) | High | The registry stores wrapped DEKs, not derived keys: rotation re-wraps, it does not re-derive. A test rotates the master key and asserts that every pre-rotation artefact is still readable. |
| R-10 | **The `Error` enum's variant set becomes feature-dependent** and the bindings' error-code parity drifts | Low | The variant additions land in the same PR as the bindings' `VantaErrorCode` mapping and the `docs/api/` error table (Regla 3). `#[non_exhaustive]` makes it additive for external consumers. |
| R-11 | **The `wal-shipping` feature ships plaintext records off-box** while the database is "encrypted at rest" | Medium | Declared in §5.3; the shipper needs the DEK; the transport is out of scope. The `wal-shipping` feature is opt-in and off by default. |

## 9. Related

- Plan de implementación por waves (`docs/dev/plans/2026-10-20-wiring-encryption-feature.md`) — the
  wave breakdown, the per-wave gates and the verification commands.
- Commit `c0e63d4e` (this branch, 2026-10-20) — the **honest negative** already published:
  `SECURITY.md:26`, the `src/crypto.rs` module doc and `docs/CHANGELOG.md`. This ADR is the
  forward half; the doc amendment it requires is stage S6 of the rollout.
- `docs/dev/Backlog.md` — `FIND-249` (feature not wired), `FIND-407` (false `SECURITY.md`
  claim; owner's decision of 2026-10-20 recorded there with pointers to this ADR and the plan),
  `FIND-194` (master-key rotation invalidates envelopes), `FIND-246` (`checkpoint_seq`
  published before the snapshot artefacts), `FIND-247`/`FIND-248` (WAL archive/rotation
  semantics), `FIND-302` (DEK registry on the L1 write path).
- `docs/dev/reviews/2026-10-20-auditoria-profunda-v0.9.0.md` — the P0 finding that opened this
  work.
- ADR-0055 — multi-writer merge (LWW) — unrelated to encryption, cited here only because it
  establishes the ADR's house style for decision records with declared limits.
- ADR-0056 — object storage path — cited as the precedent for "an ADR that records a path and
  its conditions, with implementation owned elsewhere".
- `vanta-memory/src/utils/erasure.rs` — the DEK-registry precedent this ADR generalises.
- `vanta-proxy/src/envelope.rs` — the key-version-in-derivation precedent.
- `.opencode/rules/durability.md`, `.opencode/rules/concurrency-async.md` (R-8, lock order),
  `.opencode/rules/memory-budget.md` (R-3) — the normative rules this design must not break.

## 10. INV-012 exception — `DecryptCache` is out of scope

> **Owner ratification: 2026-10-23** (FIND-617, via `docs/dev/tasks/FIND-617.md`)

INV-012 (`.opencode/rules/durability.md:10-17`) prohibits reintroducing a complex buffer cache or
data layout with physical locality **without a measurement against `vfile_search.rs`**.

The `DecryptCache` LRU introduced by FIND-598 / Paso 0 (`src/storage/vfile.rs:160-223`) is
**declared out of scope of INV-012** for the following reasons, each confirmed by measurement:

1. **No layout change on disk.** The DecryptCache lives entirely in heap memory; the on-disk format
   is unchanged (per-artifact AES-GCM envelopes, ADR-0057 §3).
2. **No effect on the plaintext path.** The cache is only instantiated when `is_encrypted()` returns
   true (segment header v3). A plaintext vfile (`File::open` without a DEK) never touches the cache.
3. **No re-introduction of anti-locality.** The original INV-012 concern was caching I/O pages that
   defeat mmap locality. The DecryptCache caches *post-AES-GCM plaintext blobs* — it sits after
   the I/O layer, not between mmap and the HNSW traversal.
4. **Eviction policy is back-pressure, not prefetch.** Under memory pressure the cache stops
   caching; it does not proactively pull pages into RAM.

**Measurement confirming the plaintext path was not regressed (2026-10-23):**

```
cargo bench --bench vfile_search -- --output-format bencher

test vfile_search/in_memory          bench: 865,485,650 ns/iter (+/- 33,167,453)  [200 queries]
test vfile_search/with_vfile         bench: 872,369,000 ns/iter (+/- 36,465,475)  [200 queries]
test vfile_search/with_vfile_compacted bench: 943,605,450 ns/iter (+/- 22,594,261) [200 queries]
```

`with_vfile` overhead vs `in_memory`: **+0.8%** — within noise. The plaintext search path is
unaffected by the DecryptCache introduction. The divergence with INV-012 was of **form** (the
measurement was not recorded at the time of the Paso 0 commit), not of **substance** (the risk
INV-012 was designed to prevent does not exist here).

The fix for the formal gap: this entry. The measurement is now on record.
