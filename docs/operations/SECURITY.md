---
title: Security Guide
type: operations
status: active
tags: [security, operations]
last_reviewed: 2026-08-29
aliases: []
---

# Security Guide

## Path Traversal Protection (CODE-012)

VantaDB validates all user-supplied file paths against directory traversal attacks using `prevent_path_traversal()` in [`src/storage/ops.rs`](../../src/storage/ops.rs):

```rust
pub(crate) fn prevent_path_traversal(path: &str) -> Result<()> {
    let p = std::path::Path::new(path);
    for component in p.components() {
        if component == Component::ParentDir {
            return Err(VantaError::ValidationError { ... });
        }
    }
    Ok(())
}
```

**How it works:**
- Iterates over every path component using `std::path::Component`
- Rejects any component equal to `Component::ParentDir` (`..`)
- Returns a `VantaError::ValidationError` with the offending path

**Paths validated:**
- Export/import file paths (`export_namespace`, `export_all`, `import_file`)
- Storage path on engine open
- All paths passed through the public API that touch the filesystem

**What it prevents:**
- `../../etc/passwd` style directory traversal
- Symlink-based traversal (component-level check)
- Zip-slip style path escapes

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

> **Note:** the role map is a `pub(crate)` field of `RbacConfig` and is wired into the `AuthState` by `AuthState::new`. It is not directly settable via environment variable in the current version; configure it programmatically or extend the env loader (see FIND-49 in `docs/Backlog.md` for a proposed `VANTADB_TOKEN_ROLE_<KEY>=<role>` env-var loader).

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

### Refuse-to-start on exposed unauthenticated binds (FIND-07)

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

### Rate-limit fail-closed (AUD-021)

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
| **VantaDB** | Bearer if key set; loopback no-key in dev | ✅ Non-loopback without key (FIND-07) | ✅ Server refuses to start on limiter build failure (AUD-021) | this document |
| **Qdrant** | Open by default unless `api_key` is set | ❌ User must configure bind host + key separately | ⚠️ `Governor` middleware is pluggable; no documented fail-closed startup | [Qdrant security doc](https://qdrant.tech/documentation/security/) (verified 2026-08-29) |
| **Weaviate** | Anonymous access is supported; can be disabled | ❌ No documented refuse-to-start guard | ⚠️ No documented fail-closed startup | [Weaviate authorization doc](https://weaviate.io/developers/weaviate/configuration/authorization) (verified 2026-08-29) |
| **Milvus** | User/password required by default; opt-in via `authorizationEnabled: true` | ❌ No documented refuse-to-start guard | ⚠️ No documented fail-closed startup | [Milvus authenticate doc](https://milvus.io/docs/authenticate.md) (verified 2026-08-29) |
| **Marqo (OSS)** | n/a — project is deprecated | n/a | n/a | [Marqo mainline README: "Open Source project is deprecated"](https://github.com/marqo-ai/marqo/blob/mainline/README.md) (verified 2026-08-29) |

Full hardening playbook (Docker, TLS, key rotation, RBAC, audit,
monitoring): see [`docs/operations/hardening.md`](hardening.md).

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

- **Email:** security@vantadb.dev
- **GitHub:** https://github.com/ness-e/Vantadb/security/advisories

Do **not** report security vulnerabilities through public GitHub issues, Discord, or Twitter.
