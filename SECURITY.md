# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.9.x   | ✅ Active development |
| 0.8.x   | ⚠️ Security patches only |
| 0.7.x   | ⚠️ Security patches only |
| < 0.7   | ❌ Not maintained   |

Patch releases are published for the latest minor version. Upgrade to the newest release to receive security fixes.

## Reporting a Vulnerability

**Do not** file a public GitHub issue for security vulnerabilities.

Report via either channel:

- **Email:** security@vantadb.dev
- **GitHub Advisories:** https://github.com/ness-e/Vantadb/security/advisories

We aim to acknowledge receipt within 48 hours and provide an initial assessment within 5 business days. Once a fix is ready, we publish a patch release and disclose the finding.

## Security Practices

- **AES-256-GCM primitives** behind the `encryption` feature flag (`src/crypto.rs`), used today by the `vanta-proxy` envelope — **the core storage files (WAL, HNSW, VantaFile, backend-KV, snapshots) are not encrypted at rest yet** (`ADR-0057`, `FIND-249`).
- **Constant-time token comparison** (`subtle::ConstantTimeEq`) for API key auth — no timing side channels.
- **Path traversal prevention** — all filesystem paths validated via `prevent_path_traversal()`.
- **Read-only mode** — write operations fail before touching storage when `read_only: true`.
- **Per-IP rate limiting** on auth failures (5 attempts / 60s window).
- **TLS 1.2+** via rustls on the HTTP server (feature `tls`).
- **RBAC** — admin/writer/reader roles enforced per HTTP method.

See [`docs/user/operations/SECURITY.md`](docs/user/operations/SECURITY.md) for the full security guide.
