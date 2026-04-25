# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| Pre-v0.1| :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in VantaDB (including but not limited to memory safety violations, unsafe deserialization, index corruption, or unauthorized access through the HTTP API), please report it responsibly:

1. **Preferred:** Use [GitHub Security Advisories](https://github.com/DevpNess/VantaDB/security/advisories/new) to open a private report directly on this repository.
2. **Alternative:** Email the maintainer at **devpness@proton.me** with:
   - A description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment

> **Please do not open a public Issue for security vulnerabilities.** We will acknowledge your report within 48 hours and aim to provide a fix or mitigation within 7 days for critical issues.

## Scope

The following are in scope for security reports:
- Memory safety violations in Rust `unsafe` blocks
- PyO3 serialization boundary escapes
- HTTP API authentication or authorization bypasses
- Storage engine corruption vectors
- Denial of service through crafted queries

## Disclosure Policy

We follow coordinated disclosure. Once a fix is released, we will credit the reporter (unless anonymity is requested) in the release notes.
