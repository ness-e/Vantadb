# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.9.x   | ✅ Supported |
| < 0.9   | ❌ Not maintained |

## Reporting a Vulnerability

Do not open public GitHub issues for security vulnerabilities.

Please report vulnerabilities through either channel:
- **GitHub Security Advisories:** https://github.com/ness-e/VantaDB/security/advisories
- **Email:** developererosnessy@gmail.com

Please include reproduction steps, affected components, and potential impact. We will review reports promptly and publish patches with appropriate security advisories.

## Security Architecture & Scope

- **AES-256-GCM encryption at rest** behind feature `encryption` with per-artifact envelopes (WAL frames, segments, index images, backend values).
- **WAL hash-chain tamper detection** to detect corruption or unauthorized modifications in the write-ahead log.
- **Constant-time token comparison** (`subtle::ConstantTimeEq`) for API key authentication to mitigate timing side channels.
- **Path traversal prevention** on all filesystem operations and database paths.
- **Role-based access control (RBAC) and TLS 1.2+** support on the optional HTTP server (`tls` feature).

For full security practices, threat model, and cryptographic boundaries, refer to [`docs/user/operations/SECURITY.md`](docs/user/operations/SECURITY.md) and [`docs/user/operations/security/KEY_RUNBOOK.md`](docs/user/operations/security/KEY_RUNBOOK.md).
