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

- **AES-256-GCM encryption at rest** behind the `encryption` feature flag (`src/crypto.rs`) — **opt-in twice**: the cargo feature is **not** in `default`, and without a master key the bytes on disk are byte-identical to a build without the feature. With the feature compiled **and** a master key configured, the four independent on-disk write paths of the core are encrypted with per-artefact AEAD envelopes: the **WAL** (format v4, per frame), the **VantaFile** segments (format v3, per node), the **HNSW** index image (format v9, per file) and the **backend key-value store** (per value) — and, by transit, the **text index** (a keyspace of the backend) and the **snapshots** (a mirror of the four). The key hierarchy is `master → KEK → per-artefact DEK`, wrapped in `data/encryption.registry`; the master key is never persisted, and `vanta-cli encryption rotate` re-wraps the DEKs without invalidating a single ciphertext (`ADR-0057`, `FIND-249`, `FIND-194`).
- **What "encrypted at rest" does NOT cover** (the honest limits — see [the key runbook](docs/user/operations/security/KEY_RUNBOOK.md) and `ADR-0057` §5.3):
  - The **backend's keys stay in cleartext** (namespace + key name): the text index needs prefix scans per namespace and the namespace index needs ordered lookups. The **values, the postings and the metadata are encrypted**; the key names are not.
  - The **text index's tokens stay in cleartext**: they are part of the key, not of the value. The posting key holds `namespace ‖ field ‖ indexed string ‖ key`, so a **string field's value** (a metadata value, for instance) is indexed **verbatim** in the key and is therefore readable on disk.
  - The **encryption registry's key material is wrapped, not secret**: the file `data/encryption.registry` holds the wrapped DEKs, the public database salt and the current nonce epoch. The **master key is never in it**.
  - The **size of the values leaks** (the envelope is `plaintext + 28 bytes`: a 12-byte nonce and a 16-byte tag), and so does the **existence, the number and the order of the values of a range** — the structural metadata (file sizes, record counts, timestamps, the shard count) is cleartext for encrypted databases too.
  - The **master key lives in the process memory** while the database is open: this protects the data at rest, **not** against an attacker who can read the process's memory (or who has the key).
  - **Losing `data/encryption.registry` loses the database**: the DEKs only live there. Back it up with the data, never separately from it.
  - There is **no in-place migration**: an existing plaintext database is never migrated implicitly (a key configured on a plaintext database is ignored, with a warning). To encrypt it, export it and import it into a new database created with the key configured.
  - **wal-shipping egresses plaintext records** to the remote replica: the replica must hold the same DEK, and the transport is outside the "at rest" boundary.
- **Constant-time token comparison** (`subtle::ConstantTimeEq`) for API key auth — no timing side channels.
- **Path traversal prevention** — all filesystem paths validated via `prevent_path_traversal()`.
- **Read-only mode** — write operations fail before touching storage when `read_only: true`.
- **Per-IP rate limiting** on auth failures (5 attempts / 60s window).
- **TLS 1.2+** via rustls on the HTTP server (feature `tls`).
- **RBAC** — admin/writer/reader roles enforced per HTTP method.

See [`docs/user/operations/SECURITY.md`](docs/user/operations/SECURITY.md) for the full security guide.
