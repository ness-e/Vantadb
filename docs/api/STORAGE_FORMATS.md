---
title: "Storage formats — on-disk versions and the encrypted variants"
kind: reference
status: active
description: "The on-disk format contract of VantaDB: the format versions of the WAL, the VantaFile segments, the HNSW index image and the encryption registry, the encrypted variants behind the `encryption` feature, and the constants that name them."
tags: [vantadb, api, storage, formats, encryption, versioning]
created: "2026-10-21"
---

# Storage formats — on-disk versions and the encrypted variants

> The **on-disk** contract of VantaDB: which version each artefact writes, how
> a reader decides whether to decrypt, and the public constants that name it.
> The design is [`ADR-0057`](../dev/architecture/adr/ADR-0057-encryption-at-rest.md);
> the strategy is
> [`STORAGE_VERSIONING.md`](../dev/architecture/STORAGE_VERSIONING.md).

## 1. The two variants of every format

Every data format has a **plaintext** variant and an **encrypted** variant.
They differ **only** in the `format_version` of the 16-byte cleartext header
(`src/binary_header.rs`) — the header is the **mode marker**: a reader decides
from the bytes on disk, never from the configuration.

| Artefact | Plaintext | Encrypted | Max readable (with `encryption`) |
|---|---|---|---|
| WAL (`data/vanta.wal`, `data/vanta.shard{N}.wal`) | `3` | `4` | `WAL_MAX_READABLE_VERSION` |
| VantaFile segments (`data/vstore_L{0..3}.vanta`) | `2` | `3` | `VFILE_MAX_READABLE_VERSION` |
| HNSW index image (`data/vector_index.bin`) | `8` | `9` | `VECTOR_INDEX_MAX_READABLE_VERSION` |
| backend key-value store | (no header of its own) | (no header of its own) | — the **registry** is the marker |
| Encryption registry (`data/encryption.registry`) | — | `1` (its own format) | — |

Without the `encryption` feature, the max readable version is the plaintext
one: a build without the feature **refuses** an encrypted artefact with an
explicit error (`EncryptionKeyRequired`, or `IncompatibleFormat` with the
hint *"the HNSW image is encrypted (format v9 or newer) but this build has no
'encryption' feature"*), **never** with a corruption error.

## 2. The public constants

```rust
// src/wal.rs
pub const WAL_FORMAT_VERSION: u16 = 3;
pub const WAL_ENCRYPTED_FORMAT_VERSION: u16 = 4;
pub const WAL_MAX_READABLE_VERSION: u16 = /* 4 with `encryption`, else 3 */;

// src/storage/vfile.rs
pub const VFILE_VERSION: u16 = 2;
pub const VFILE_ENCRYPTED_VERSION: u16 = 3;
pub const VFILE_MAX_READABLE_VERSION: u16 = /* 3 with `encryption`, else 2 */;

// src/index/graph/types.rs
pub const VECTOR_INDEX_VERSION: u16 = 8;
pub const VECTOR_INDEX_ENCRYPTED_VERSION: u16 = 9;
pub const VECTOR_INDEX_MAX_READABLE_VERSION: u16 = /* 9 with `encryption`, else 8 */;
```

The `*_MAX_READABLE_VERSION` constants are the ones the readers use
(`Header::validate_compat`): they accept every version up to the maximum, so
a build with the `encryption` feature reads the plaintext artefacts of an
older database **unchanged**.

The `encryption` feature is **not** in `default`: a crate built without it
does not even contain the encrypted code path, and the `*_MAX_READABLE_VERSION`
constants collapse to the plaintext version.

## 3. The encrypted envelope of each artefact

| Artefact | Envelope | Associated data (AAD) |
|---|---|---|
| WAL | `[u32 ct_len][u32 crc32c(ct)][12B nonce][ct ‖ 16B tag][32B prev_hash][32B record_hash]` | `artefact ‖ version ‖ epoch ‖ frame ordinal ‖ plaintext_len` |
| VantaFile | `[u32 blob_len][12B nonce][ct ‖ 16B tag]` | `artefact ‖ version ‖ epoch ‖ node offset ‖ plaintext_len` |
| HNSW image | `[12B nonce][ct ‖ 16B tag]` | `artefact ‖ version ‖ epoch ‖ 0 ‖ plaintext_len ‖ relative path` |
| backend KV | `[12B nonce][ct ‖ 16B tag]` | `artefact ‖ version ‖ epoch ‖ 0 ‖ plaintext_len ‖ partition ‖ 0x00 ‖ key` |

* The cipher is **AES-256-GCM**; the nonce is `epoch (u64 BE) ‖ counter
  (u32 BE)`, the epoch being random and persisted **before** the first
  encryption under it.
* The AAD is **authenticated, not encrypted**: it holds no secret, it binds a
  ciphertext to its position (a frame moved to another ordinal, a node moved
  to another offset, an image moved to another path, or a value moved to
  another key **fails to authenticate**).
* The WAL's CRC32C and its hash chain cover the **ciphertext**, so
  `vanta-cli verify` keeps working **without the key**.

## 4. The encryption registry

`<storage_root>/data/encryption.registry` (magic `VANTAREG`, format `1`)
holds, per artefact: the artefact id, the `key_version`, the **wrapped** DEK
(`[12B nonce][ct ‖ 16B tag]` under the KEK), the DEK's random 8-byte id, the
current nonce epoch, plus the public `db_salt` and a wrapped key-check
sentinel.

* The **master key is never persisted**: the file holds only wrapped DEKs.
* The registry is written atomically (tmp → `rename` → fsync of the
  directory) and it lives inside `data_dir` on purpose — the snapshot
  machinery copies that directory, and the registry is needed **before** the
  backend can be opened (the backend's own values are encrypted).
* The registry is the **mode record** of the database: an artefact without an
  entry is plaintext, and it is never migrated implicitly.
* **Losing the registry loses the database** — the DEKs live only there. See
  the [key runbook](../user/operations/security/KEY_RUNBOOK.md).

## 5. Compatibility

| Situation | Behaviour |
|---|---|
| A plaintext database, any build | Read and written **byte-identically**, forever |
| A plaintext database + a key configured | Plaintext, with a warning at open — **no implicit migration** |
| An encrypted database + the right key | Encrypted |
| An encrypted database + no key | `Error::EncryptionKeyRequired` — refuse, never "corrupt" |
| An encrypted database + the wrong key | `Error::EncryptionKeyRejected` ("wrong key or tampered") |
| An encrypted artefact in a build without the feature | Explicit refusal with the version in the message |
| A rotated master key | The DEKs are re-wrapped, not re-derived: **no ciphertext is invalidated** |
