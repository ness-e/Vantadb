---
title: Deprecations Registry
kind: reference
status: active
description: Concrete deprecations of the public API surfaces and their removal windows
tags: [vantadb, api, deprecation]
---

# Deprecations Registry

Concrete deprecations of the public API surfaces and their removal windows.
The policy itself lives in [`VERSIONING.md` § Deprecation policy](VERSIONING.md#deprecation-policy) —
this page registers instances, it does not restate the policy:

- A deprecated API stays functional for **at least one MINOR release**.
- Removal ships in a later MINOR with a `feat!:` / `BREAKING CHANGE:` marker.

## Active deprecations

### `crypto::EncryptionStream` → per-artefact AEAD envelope

- **Surface:** 1 — Rust core SDK
- **Deprecated in:** v0.9.1 (2026-10-20)
- **Removal target:** v0.10.0 — ≥ 1 MINOR after deprecation (earliest possible)
- **Replacement / migration:** `EncryptionStream` frames as
  `[u32 len][nonce][ct+tag]` — it expresses **no AAD, no slot and no epoch**,
  so a ciphertext moved to another position would still authenticate
  (ADR-0057 C-6). At-rest encryption uses
  `Cipher::encrypt_with_nonce` / `decrypt_with_nonce` with a caller-chosen
  nonce, plus `crypto::registry::AadBuilder` to bind the ciphertext to its
  slot. The type is kept (not removed) because `storage::vfile` still exposes
  it to its own test; it is retired with the VantaFile wave.
- **Changelog:** [`docs/CHANGELOG.md`](../CHANGELOG.md) § [0.9.1]
- **Code/doc marker:** `#[deprecated]` attribute in `src/crypto.rs`

### `VantaHeader` → `Header`

- **Surface:** 1 — Rust core SDK
- **Deprecated in:** v0.8.0 (pending release — release-plz sets the date)
- **Removal target:** v0.9.0 — ≥ 1 MINOR after deprecation (earliest possible)
- **Replacement / migration:** use `Header`; `VantaHeader` remains a
  `#[deprecated(since = "0.8.0")]` type alias, so existing code keeps
  compiling during the window
- **Changelog:** [`docs/CHANGELOG.md`](../CHANGELOG.md) § [0.8.0]
- **Code/doc marker:** `#[deprecated]` attribute in `src/binary_header.rs`

> Until v0.7.0 (updated 2026-09-27) no public API item was deprecated on any of
> the [11 surfaces](VERSIONING.md#the-11-public-api-surfaces).

## Entry shape (fill for every new deprecation)

### Template — `old_name` → `new_name`

- **Surface:** <number + name from the 11-surface table>
- **Deprecated in:** v0.X.0 (YYYY-MM-DD)
- **Removal target:** v0.Y.0 — ≥ 1 MINOR after deprecation (earliest possible)
- **Replacement / migration:** <what consumers should do instead>
- **Changelog:** [`docs/CHANGELOG.md`](../CHANGELOG.md) § [0.X.0]
- **Code/doc marker:** <deprecation attribute / doc note / release-note marker>

## Process

1. **Deprecate** — in the same PR that marks the item (attribute, doc note or
   reference-doc note): add the changelog entry, update the affected surface
   doc, and fill an entry above.
2. **Remove** — at or after the removal target, in a MINOR release with a
   `feat!:` / `BREAKING CHANGE:` marker; delete the entry above.
3. **Mechanical line** — the rails in [`COMPATIBILITY.md`](COMPATIBILITY.md)
   (semver-checks + public API snapshot) fail any removal that skips the
   marker/registers, so this registry cannot silently rot.
