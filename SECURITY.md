# Security Policy

## Supported Versions

ConnectomeDB is currently in early active development. Security updates are guaranteed for the latest minor and patch versions.

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| 0.2.x   | :white_check_mark: |
| < 0.2.0 | :x:                |

## Reporting a Vulnerability

Security is a top priority for ConnectomeDB, especially considering its role in local data persistence for AI agents.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them to **security@connectomedb.dev** or use the GitHub Security Advisory feature in this repository. You should receive a response within 48 hours.

If the issue is confirmed, we will release a patch as soon as possible, depending on complexity, and we will credit you in the release notes.

### Scope
We are particularly sensitive to vulnerabilities targeting:
- Arbitrary code execution via IQL deserialization.
- Path traversal exploits when accessing the internal RocksDB storage layer.
- Out-of-bounds memory accesses in the `UnifiedNode` zero-copy parser.
