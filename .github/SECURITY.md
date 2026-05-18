# Security Policy

## Supported Versions

| Version  | Supported          |
| -------- | ------------------ |
| 0.1.x    | :white_check_mark: |
| Pre-v0.1 | :x:                |

## Reporting a Vulnerability

VantaDB does not currently publish a dedicated security mailbox.

If you discover a security vulnerability in VantaDB (including but not limited to memory safety
violations, unsafe deserialization, index corruption, or unauthorized access through the HTTP API),
report it through one of these channels:

1. **Preferred private channel:** If private reporting is enabled on this repository, use
   [GitHub Security Advisories](https://github.com/DevpNess/Vantadb/security/advisories/new).
2. **Non-sensitive reports:** Open a GitHub Issue and apply or request the `security` label.
3. **If private reporting is unavailable:** Open a minimal public issue asking for a private
   coordination path, but do not include exploit details, secrets, or weaponized proof-of-concept
   material.

> Do not disclose sensitive vulnerability details in a public issue. Use the private advisory flow
> when available, and reserve labeled public issues for low-risk or already-public hardening items.

## Scope

The following are in scope for security reports:

- Memory safety violations in Rust `unsafe` blocks
- PyO3 serialization boundary escapes
- HTTP API authentication or authorization bypasses
- Storage engine corruption vectors
- Denial of service through crafted queries

## Disclosure Policy

We follow coordinated disclosure. Once a fix is released, we will credit the reporter (unless
anonymity is requested) in the release notes.
