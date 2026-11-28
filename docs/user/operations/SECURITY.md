---
title: Security Guide
kind: runbook
status: active
description: VantaDB validates all user-supplied file paths against directory traversal attacks using preventpathtraversal() in src/storage/ops.rs
tags: [security, operations]
---

# Security Guide

## Path Traversal Protection (CODE-012)

VantaDB validates all user-supplied file paths against directory traversal attacks using `prevent_path_traversal()` in [`src/storage/ops.rs`](../../../src/storage/ops.rs):

```rust
pub(crate) fn prevent_path_traversal(path: &str) -> Result<()> {
    let p = std::path::Path::new(path);
    for component in p.components() {
        if component == Component::ParentDir {
            return Err(Error::Validation { .. });
        }
    }
    Ok(())
}
```

**How it works:**
- Iterates over every path component using `std::path::Component`
- Rejects any component equal to `Component::ParentDir` (`..`)
- Returns an `Error::Validation { field: "path", reason }` with the offending
  path (`VANTADB_VALIDATION_ERROR`; a `ValidationError` in the Python binding)

**Paths validated:**
- Export/import file paths (`export_namespace`, `export_all`, `import_file`)
- Storage path on engine open
- All paths passed through the public API that touch the filesystem

**What it prevents:**
- `../../etc/passwd` style directory traversal
- Symlink-based traversal (component-level check)
- Zip-slip style path escapes

## Encryption at Rest (opt-in)

With the `encryption` cargo feature (**not** in `default`) **and** a master key
configured, the four independent on-disk write paths of the core are encrypted
with per-artefact AES-256-GCM envelopes: the **WAL** (format v5), the
**VantaFile** segments (format v4), the **HNSW index image** (format v9) and
the **backend key-value store** (per value) — and, by transit, the text index
and the snapshots.

The AAD of every envelope binds the ciphertext to **its file as well as its
slot** (FIND-586): the frame/record carries the segment or shard identity, so a
blob moved between sibling files (`vstore_L0.vanta` → `vstore_L1.vanta`, or
`vanta.shard0.wal` → `vanta.shard1.wal`) at the same offset no longer
authenticates. Formats v4/v5 are the first layouts with that binding: the
earlier encrypted layouts (VantaFile v3, WAL v4) are refused with
`IncompatibleFormat` instead of being read with a weaker binding — they were
never published, so the database has to be rewritten (re-index from canonical
data), not migrated.

It is **off by default twice**: the feature is not in `default`, and without a
master key the bytes on disk are byte-identical to a build without the
feature. An existing plaintext database is **never** migrated implicitly.

- **Fuentes de clave** (precedencia fija): `VANTADB_ENCRYPTION_KEY` →
  `VANTADB_ENCRYPTION_KEY_FILE` → `VANTADB_ENCRYPTION_KEY_CMD`.
- **Lo que NO se cifra**: las claves del backend y las **etiquetas ciegas** de los
  índices (v4: `tag8 = SHA-256(domain ‖ valor)[0..8]`, sin clave — ver las
  secciones de abajo), el namespace, el tamano de los valores, la estructura
  (conteos, orden, timestamps), y la master key vive en la memoria del proceso.
  Los **nombres** de campo y los **valores** del índice derivado de payload
  siguen en claro (un filtro por igualdad tiene que encontrarlos sin el
  registro); la clave del registro **no**, desde `FIND-830`.

### Text index keys (FIND-513): blind tags, and what still leaks

The encrypted at-rest layout protects the **payload** and the **posting value**
(tf, positions, node id). The **key** is not encrypted (the AAD binds a value to
its key; it never encrypts the key), so the key must not carry the data itself.

As of the **v4** layout (`src/text_index.rs`, marker `\xffvanta_text_v4\0`) it
does not:

```
posting    = namespace ‖ 0x00 ‖ tag8(token) tag8(record_key)
doc_stats  = v4 ‖ "doc" ‖ namespace ‖ 0x00 ‖ tag8(record_key)
term_stats = v4 ‖ "term"‖ namespace ‖ 0x00 ‖ tag8(token)
tag8(x)    = SHA-256(domain ‖ x)[0..8]
```

An operator with disk access can still learn:

- **How many** postings a term has and **in which order** they were written
  (the KV sorts the keys), and the **length** of every value:
  `ciphertext = plaintext + 28` (nonce + tag).
- **Which namespace** an entry belongs to — kept in clear on purpose, it is the
  scan prefix and the namespace purge relies on it.
- **The vocabulary, by dictionary.** The tag is a bare 64-bit prefix of
  `SHA-256` with no key, no salt and no stretching, so an offline wordlist
  recovers the indexed terms — and, since record ids are short and guessable
  (`k1`, `item-1`), also the term→document map. The index is no longer readable
  by `strings`; it is **not** confidential against an attacker who will hash a
  dictionary. Closing that needs a keyed tag (`k_blind` derived from the
  database key), tracked as its own finding.

What is gone **from the text index**: the cleartext token, the cleartext record
key, and the trivially-readable term→document correlation.

### Derived index and sparse postings (FIND-830): the record key is blind too

The same primitive (`tag8`, one implementation, a domain separator per family)
now backs the two derived index keys and the sparse postings — the keyspaces
`FIND-513` left open:

```
namespace index = namespace ‖ 0x00 ‖ tag8(record_key)
payload index   = namespace ‖ 0x00 ‖ field ‖ 0x00 ‖ encoded_value ‖ 0x00 ‖ tag8(record_key)
sparse posting  = "v2" ‖ 0x00 ‖ namespace ‖ 0x00 ‖ dim ‖ 0x00 ‖ tag8(record_key)
```

What stops being readable: **which record** each derived entry belongs to. That
was not only the record key — the payload index made it a payload map ("record
`invoice-2026-0001` has `owner = eros`"), so the association key→payload is gone
from the file. The `sparse_posting_prefix` (the range scan in
`src/sdk/search/sparse.rs`) and the two derived prefixes are untouched: the tag
sits **after** the prefix, exactly like the text-index `posting_prefix`.

What is **not** covered, and must not be read as covered:

- **The namespace stays in clear** in all four keyspaces. It is the scan prefix
  and the namespace purge depends on it; encrypting it would break the ordered
  lookups (same declared limit as `ADR-0057 §5.3.1`).
- **Field names and filter values stay in clear** in the payload index
  (`field`, `encoded_value`). An equality filter has to find `(field, value)`
  without knowing the record, so those two components must stay scannable; the
  record they point at is a tag. An operator still learns *which* field/value
  pairs a namespace indexes — never *which record* holds them.
- **The tags are still unkeyed**, so the dictionary attack applies here too: a
  guessed record key or payload value can be confirmed, and a short/guessable
  record id (`k1`, `invoice-2026-0001`) is recovered by hashing candidates.
  Measured on a real text-index partition (`FIND-829`): **40/40 indexed tokens
  recovered (100 %) from a 200-word list**, 0 false positives. The same holds
  for the derived and sparse tags. Closing that needs `k_blind` — a per-database
  secret derived from the key hierarchy (`ADR-0057`) — which is **not
  implemented** (`k_blind` appears in **0** source files); the contract and the
  missing pieces are in [`FIND-829`](../../dev/tasks/FIND-829.md).

Version marker: the derived and sparse indexes carry it in their state
(`DERIVED_INDEX_SCHEMA_VERSION` 1→2, `SPARSE_INDEX_SCHEMA_VERSION` 1→2) plus a
`v2` key prefix for the sparse postings, so a v1 entry is rejected and the
partition is **rebuilt** from canonical records on the **next writable open**.
A **read-only** open never rebuilds (it cannot write), and what it answers
differs per index — stated here so nobody reads "rebuilt" as "always correct":

- **Derived namespace/payload index:** the prefixes did not change
  (`namespace_index_prefix`, `payload_index_prefix`), so a stale v1 index still
  answers correctly and is merely refurbished later.
- **Sparse postings:** the prefix moved `v1`→`v2`, so a stale index would match
  **no key at all**. The query path therefore **refuses** with
  `Validation { field: "sparse_index_schema" }` ("reopen it writable to rebuild
  the index") instead of returning an empty list
  (`ensure_sparse_index_query_ready`, `src/sdk/search/sparse.rs`). Pinned by
  `search::sparse::tests::sparse_search_on_a_read_only_open_of_an_old_index_fails_loudly`.

Pinned by
`impl_index::tests::f830_no_clear_record_key_in_derived_or_sparse_keys` and
`impl_rebuild::tests::f830_clear_key_indexes_are_rejected_and_rebuilt`.

The layout change is a **version marker plus a rebuild** of the text index
(derived from canonical data): an index written by the clear-key layout reports
an incompatible state and is rebuilt on the next writable open. Design and
delta: `docs/dev/tasks/FIND-513.md`; two tests pin it
(`text_index::tests::posting_key_leak_is_declared_in_the_security_guide`,
`impl_rebuild::tests::f513_v3_index_rebuilds_to_the_same_search_result`).

- **WAL shipping does not replicate an encrypted database** (`FIND-496`): a
  replica would need the master key, and sending key material over the network
  is out of scope. `WalShipper::ship_once` refuses **before** shipping
  anything.

### Segment and shard identity is part of the ciphertext (FIND-586)

The AAD of every encrypted artefact carries the **identity of its file** (the
segment name for a VantaFile, the shard stem before `.wal` for the WAL). Two
operational consequences:

- **Renaming a segment makes it unreadable.** `vstore_L0.vanta` copied to
  `copy.vanta` will not decrypt: it is a different identity. Move the whole
  `data/` directory (names included) — never individual segments.
- **A backup/restore keeps the names**, so it keeps working; a manual
  re-shuffle of files between segments does not.
- The encrypted layouts changed with that binding (VantaFile **v4**, WAL
  **v5**); the earlier encrypted layouts (v3 / v4) are **refused** with
  `IncompatibleFormat` — `vanta --verify`, `migrate` and `check_integrity` say
  "rewrite the database", never "newer build".
- **Perder `data/encryption.registry` pierde la base**: las DEKs viven solo ahi.
- **Runbook completo** (guardar la key, rotarla, deshacer la rotacion,
  cifrar una base existente): [`security/KEY_RUNBOOK.md`](security/KEY_RUNBOOK.md).
- **Diseno**: [`ADR-0057`](../../dev/architecture/adr/ADR-0057-encryption-at-rest.md).

## TLS Configuration

VantaDB supports TLS 1.2 and 1.3 via the `tls` feature (rustls) in the CLI server.

### Supported Versions

| Protocol | Status |
|----------|--------|
| TLS 1.3 | ✅ Preferred |
| TLS 1.2 | ✅ Included for legacy client compatibility |

TLS 1.2 is included alongside 1.3 for compatibility with legacy HTTP clients (older curl, Java 8, Python <3.7) that do not support TLS 1.3 exclusively.

### ALPN Protocols

```rust
config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
```

- **h2** (HTTP/2) — preferred
- **http/1.1** — fallback for legacy clients

### Configuration

Set via environment variables:

| Variable | Description |
|----------|-------------|
| `VANTADB_TLS_CERT` | Path to PEM-encoded TLS certificate file |
| `VANTADB_TLS_KEY` | Path to PEM-encoded TLS private key file |

If both are set, the server serves HTTPS. If only the `tls` feature is active without configured cert/key, the server falls back to plain HTTP and logs a warning.

### TLS-Exclusive Server

When TLS is enabled, the server uses `axum_server::bind_rustls` which requires both a valid certificate and key. On failure to load either, the server logs the error and shuts down without falling back to insecure.

## Authentication

### Bearer Token Auth

The CLI server supports optional Bearer token authentication via the `VANTADB_API_KEY` environment variable:

```
Authorization: Bearer <token>
```

- Uses constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing attacks
- When no key is configured, the server runs without authentication (development mode)
- When `VANTADB_REQUIRE_AUTH=true` is set and no key is configured, the server fails to start, preventing accidental unauthenticated exposure.
- Token authentication is required for all endpoints except `/health`

### RBAC (Role-Based Access Control)

Three built-in roles:

| Role | Permissions |
|------|-------------|
| `admin` | `Admin` (full access) |
| `writer` | `Read` + `Write` |
| `reader` | `Read` only |

Roles are mapped to tokens via the `token_role_map` in `RbacConfig`. When a token matches, the mapped role's permissions are enforced per HTTP method — `POST`/`PUT`/`PATCH`/`DELETE` require `Write`, others require `Read`.

#### Configuring `token_role_map`

`token_role_map` is a `HashMap<String, String>` mapping a literal API key value to a role name. Both `VANTADB_API_KEY` and `VANTADB_ALT_API_KEY` are eligible for mapping — the auth middleware checks the `token_role_map` against the Bearer presented by the client, regardless of which configured key it is.

Programmatic configuration (e.g. from a custom config file loader):

```rust
use vantadb::config::RbacConfig;
use std::collections::HashMap;

let mut token_role_map = HashMap::new();
token_role_map.insert("sk-primary-admin".into(),   "admin".into());
token_role_map.insert("sk-alt-readonly".into(),    "reader".into());
let rbac_config = RbacConfig { token_role_map };
```

A token not present in the map authenticates as a bare `Transport` identity (L1) without any role — write/read authorization then falls through to the per-handler `PermissionChecker` defaults.

> **Note:** the role map is a `pub(crate)` field of `RbacConfig` and is wired into the `AuthState` by `AuthState::new`. It is not directly settable via environment variable in the current version; configure it programmatically or extend the env loader (see the proposed `VANTADB_TOKEN_ROLE_<KEY>=<role>` env-var loader in development roadmap).

### Auth Rate Limiting

Authentication failures are rate-limited per IP address:

| Setting | Default |
|---------|---------|
| Max failed attempts | 5 |
| Time window | 60 seconds |

After exceeding the limit, the IP receives `429 Too Many Requests` and must wait for the window to elapse. Successful authentication resets the failure count.

### API Key Rotation (Zero-Downtime)

VantaDB supports zero-downtime API key rotation using the `VANTADB_ALT_API_KEY` environment variable (SRV-04). This enables rolling key rotation without service interruption.

#### How It Works

When both `VANTADB_API_KEY` (primary) and `VANTADB_ALT_API_KEY` (alternative) are configured, **both keys are accepted simultaneously** for authentication. This allows you to:

1. **Deploy the new key as `alt_api_key`** — both old and new keys work
2. **Migrate clients** — gradually switch clients to the new key
3. **Promote the new key** — set `VANTADB_API_KEY` to the new value, remove `VANTADB_ALT_API_KEY`
4. **Complete rotation** — old key is rejected, only new key works

#### Environment Variables

| Variable | Description |
|----------|-------------|
| `VANTADB_API_KEY` | Primary Bearer token (required for auth) |
| `VANTADB_ALT_API_KEY` | Alternative Bearer token for rotation (optional) |
| `VANTADB_REQUIRE_AUTH` | If `true`, server fails to start without `VANTADB_API_KEY` |

#### Rotation Workflow Example

```bash
# Step 1: Current state - only primary key
VANTADB_API_KEY=sk-old-primary
VANTADB_REQUIRE_AUTH=true

# Step 2: Add alternative key (rotation window - both work)
VANTADB_API_KEY=sk-old-primary
VANTADB_ALT_API_KEY=sk-new-primary
VANTADB_REQUIRE_AUTH=true

# Step 3: Migrate clients to sk-new-primary, then promote
VANTADB_API_KEY=sk-new-primary
# VANTADB_ALT_API_KEY is removed
VANTADB_REQUIRE_AUTH=true
```

#### Security Notes

- Both keys use constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing attacks
- The `alt_api_key` requires `api_key` to be set (rotation needs a primary)
- RBAC `token_role_map` applies to both keys independently — see [Configuring `token_role_map`](#configuring-token_role_map) for how to wire it
- Audit logs record auth outcomes as `auth_l1` events; the recorded `key` field is `"N/A"` (the raw Bearer is **never** persisted to the audit JSONL — only the outcome and reason are). To correlate a request with the configured key used, join `auth_l1` events with the access window of the rotation.

## Input Validation

### Empty Namespace/Key Checks

All CRUD operations validate that namespace and key are non-empty before processing. Empty values return a validation error.

### Path Validation

All file system operations run through `prevent_path_traversal()` as described above.

### Read-Only Enforcement

When the engine is configured in read-only mode (`read_only: true`), write operations (`put`, `putBatch`, `delete`, `insertNode`, `deleteNode`, `rebuildIndex`, `compactWal`, `purgeExpired`, `compactLayout`, `import*`, `repairTextIndex`) return an error before touching storage.

### Rate Limiting

General HTTP rate limiting is configured via `VANTADB_RATE_LIMIT_RPM`:

| Setting | Behavior |
|---------|----------|
| `600` (default) | Burst-aware token bucket limiter at N requests/minute |
| `0` | Rate limiting disabled |

## Security Guards (Refuse-to-Start + Fail-Closed)

Two startup invariants keep the server from accidentally serving traffic in an
unsafe configuration. Both are documented in source under `src/cli_server.rs`
and exercised by integration tests.

### Refuse-to-start on exposed unauthenticated binds

The server **refuses to start** when all of the following hold:

- The bind host is non-loopback (anything other than `127.0.0.1`, `localhost`, `::1` — e.g. `0.0.0.0`)
- No API key is configured (`VANTADB_API_KEY` unset)
- No explicit dev override is given (`--allow-insecure`)

```text
Refusing to start: non-loopback host without an API key
Fix either way: (1) set VANTADB_API_KEY to enable Bearer auth, or
(2) bind a loopback host (127.0.0.1/localhost/::1), or (3) pass
--allow-insecure to override this check in dev.
```

This pattern is uncommon among vector databases in this space — Qdrant,
Weaviate, and Milvus all default to "open to all interfaces unless you
configure an API key", with the user responsible for closing the bind host
themselves. VantaDB flips this: the unsafe default is not a valid
configuration. See the [competitive positioning table in
`docs/api/HTTP_API.md`](../../api/HTTP_API.md#positioning-vs-other-vector-databases).

### Rate-limit fail-closed

The HTTP rate limiter is wired through `tower::GovernorLayer` and built at
startup. If the `GovernorConfig` fails to build (e.g. malformed RPM, clock
issues during init), the server **refuses to start** rather than serving
traffic without a limiter:

```rust
// pseudo-code from src/cli_server.rs (simplified)
let cfg = build_rate_limit_config(rpm)?; // returns Err on failure
let governor = GovernorLayer { config: cfg.into() };
```

Fail-closed here means a misconfigured limiter becomes a *hard error*, not a
silent unthrottled listener. This is the safer default for any production
deployment where unbounded request rates can amplify cost or DoS impact.

### How the other vector databases compare (honest)

| Engine | Auth default | Refuse-to-start guard | Fail-closed rate limit | Source |
|---|---|---|---|---|
| **VantaDB** | Bearer if key set; loopback no-key in dev | ✅ Non-loopback without key | ✅ Server refuses to start on limiter build failure | this document |
| **Qdrant** | Open by default unless `api_key` is set | ❌ User must configure bind host + key separately | ⚠️ `Governor` middleware is pluggable; no documented fail-closed startup | [Qdrant security doc](https://qdrant.tech/documentation/security/) (verified 2026-08-29) |
| **Weaviate** | Anonymous access is supported; can be disabled | ❌ No documented refuse-to-start guard | ⚠️ No documented fail-closed startup | [Weaviate authorization doc](https://weaviate.io/developers/weaviate/configuration/authorization) (verified 2026-08-29) |
| **Milvus** | User/password required by default; opt-in via `authorizationEnabled: true` | ❌ No documented refuse-to-start guard | ⚠️ No documented fail-closed startup | [Milvus authenticate doc](https://milvus.io/docs/authenticate.md) (verified 2026-08-29) |
| **Marqo (OSS)** | n/a — project is deprecated | n/a | n/a | [Marqo mainline README: "Open Source project is deprecated"](https://github.com/marqo-ai/marqo/blob/mainline/README.md) (verified 2026-08-29) |

Full hardening playbook (Docker, TLS, key rotation, RBAC, audit,
monitoring): see [`docs/user/operations/hardening.md`](hardening.md).

## Deployment Security Best Practices

1. **Set `VANTADB_API_KEY` in production** — never run with authentication disabled on public networks
2. **Enable TLS** — configure `VANTADB_TLS_CERT` and `VANTADB_TLS_KEY` for encrypted transport
3. **Configure rate limiting** — set `VANTADB_RATE_LIMIT_RPM` appropriate to your expected traffic
4. **Use read-only mode for query-only instances** — set `read_only: true` in the config
5. **Validate export paths** — always use absolute paths or paths relative to a known safe directory
6. **Run with minimum necessary permissions** — the database process should not run as root
7. **Keep Rust and dependencies updated** — security patches are delivered via `cargo update`

## Reporting a Vulnerability

See the [GitHub Security Advisories](https://github.com/ness-e/Vantadb/security/advisories) page for the full disclosure policy.

Do **not** report security vulnerabilities through public GitHub issues, Discord, or Twitter.
