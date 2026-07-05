---
title: Security Guide
type: operations
status: active
tags: [security, operations]
last_reviewed: 2026-07-04
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
- Token authentication is required for all endpoints except `/health`

### RBAC (Role-Based Access Control)

Three built-in roles:

| Role | Permissions |
|------|-------------|
| `admin` | `Admin` (full access) |
| `writer` | `Read` + `Write` |
| `reader` | `Read` only |

Roles are mapped to tokens via the `token_role_map` in `RbacConfig`. When a token matches, the mapped role's permissions are enforced per HTTP method — `POST`/`PUT`/`PATCH`/`DELETE` require `Write`, others require `Read`.

### Auth Rate Limiting

Authentication failures are rate-limited per IP address:

| Setting | Default |
|---------|---------|
| Max failed attempts | 5 |
| Time window | 60 seconds |

After exceeding the limit, the IP receives `429 Too Many Requests` and must wait for the window to elapse. Successful authentication resets the failure count.

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
| `0` (default) | Rate limiting disabled |
| `> 0` | Burst-aware token bucket limiter at N requests/minute |

## Deployment Security Best Practices

1. **Set `VANTADB_API_KEY` in production** — never run with authentication disabled on public networks
2. **Enable TLS** — configure `VANTADB_TLS_CERT` and `VANTADB_TLS_KEY` for encrypted transport
3. **Configure rate limiting** — set `VANTADB_RATE_LIMIT_RPM` appropriate to your expected traffic
4. **Use read-only mode for query-only instances** — set `read_only: true` in the config
5. **Validate export paths** — always use absolute paths or paths relative to a known safe directory
6. **Run with minimum necessary permissions** — the database process should not run as root
7. **Keep Rust and dependencies updated** — security patches are delivered via `cargo update`

## Reporting a Vulnerability

See [`.github/SECURITY.md`](../../.github/SECURITY.md) for the full disclosure policy.

- **Email:** security@vantadb.dev
- **GitHub:** https://github.com/ness-e/Vantadb/security/advisories

Do **not** report security vulnerabilities through public GitHub issues, Discord, or Twitter.
