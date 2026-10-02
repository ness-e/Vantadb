---
title: "Privacy & local LLMs — what VantaDB guarantees (and what it doesn't)"
kind: howto
status: active
description: "The exact privacy guarantees of the local-first stack — redaction-on-write, encrypted originals, certified delete, injection governance — with commands and a covered / not covered table"
tags: [vantadb, privacy, proxy]
---

# Privacy & local LLMs — what VantaDB guarantees (and what it doesn't)

This page is for the local-LLM stack (Ollama / AnythingLLM-style) where the
memory must stay in your process: what VantaDB's proxy and store **do**
guarantee once you opt in, the exact commands that prove it, and the limits we
declare instead of hiding. Every guarantee below is implemented and tested; the
demo runs them end to end in CI.

> **Almost everything here is opt-in.** Redaction and envelopes ship disabled
> (`[redact]` / `[envelope]`), and the injection ACL and audit are opt-in. One
> behavior is already active by default: the `<vanta-memory>` injection budget
> (`[injection] max_tokens = 2000`; set `0` to disable injection). Defaults and
> the full config surface live in the [proxy reference](../api/PROXY.md).

## The four guarantees

### 1. Redaction-on-write — PII never lands in cleartext

When `[redact] enabled = true`, a captured turn is scanned **before it is
persisted** and the masked text is what reaches both storage destinations
(`proxy-turns` + the search-facing `l1/{session}` record). The audit payload
records the kind labels found — never the values.

```toml
[redact]
enabled = true
mode = "mask"            # mask | block | log
# patterns = ["ACME-\\d{4}"]   # optional extra regexes (kinds: custom)
```

- **What is detected (built-ins):** `email`, `aws_key` (`AKIA…`), `aws_secret`
  (40-char run near an `aws_secret` marker), `token` (`sk-`, `ghp_`, `xoxb-`, …).
- **Modes on write:** `mask` and `block` both mask (a completed turn is never
  dropped); `log` observes only — same semantics as the wire.
- **Declared cap:** turns above 2 MiB (`redact.max_scan_bytes`) fail open on the
  wire and persist **unscanned** with a warning (no envelope). Shorten capture
  or raise the cap if your traffic exceeds it.

### 2. Encrypted originals — per-namespace AEAD (optional)

With `[envelope] enabled = true` and a 32-byte `VANTADB_ENCRYPTION_KEY`
(64 hex chars), the **pre-redaction original** of a turn survives — and only —
inside an AES-256-GCM envelope whose key is derived per namespace
(`HKDF-SHA256`, salt + namespace + key version). Namespace and version are part
of the key: a re-labelled or tampered blob does not authenticate.

```toml
[envelope]
enabled = true           # requires [redact] enabled + the key
key_version = 1          # bump to rotate: old envelopes stay readable
```

- **Rotation (v1):** bump `key_version` — new writes seal under the new derived
  key. Rotating the master key itself requires re-encrypting existing envelopes
  (not automated yet — declared limit).
- **Disarmed = explicit:** enabled without a usable key → startup warning,
  envelopes skipped, the original is **not** written — never a silent fallback
  to cleartext. The effective mode is visible in `GET /snapshot` under
  `"envelope"` (`redacted_only` · `active` · `disarmed`).

### 3. Certified delete — forget with a per-surface receipt

```bash
vanta-cli delete --db ./db --namespace persona --key alice --attest --out certificate.json
vanta-cli certificate verify --db ./db --file certificate.json   # exit 0 = valid
```

The certificate inventories every purge surface (`store`, JSON-shredded
metadata, HNSW, vector tombstone, derived/text/sparse indexes, version history,
WAL tombstone), carries a `sha256` integrity hash and references the
[WAL hash-chain](../api/WAL_INTEGRITY.md) (`vanta-cli verify`). Verification
re-scans the re-checkable surfaces and fails (non-zero) on tampering or
residues. Scope and limits are always listed in
[Certified delete](../api/CERTIFIED_DELETE.md) — including the one people
actually ask about:

- **Not covered:** secure erase of physical media, backups/snapshots outside the
  live database directory, archived WAL segments, exports, audit logs (namespace
  + key survive until retention), and **no parametric unlearning** — embedding
  model weights are never touched.

### 4. Injection governance — budget, ACL, audit

The `<vanta-memory>` block that feeds memory into prompts is budgeted
(`[injection] max_tokens`, `0` disables), optionally ACL'd by namespace prefix
(`namespace_allow_prefixes`, empty = allow all), and auditable:

```toml
[injection]
max_tokens = 2000
namespace_allow_prefixes = ["l1/", "persona/", "scene/"]
audit_log_path = "injection-audit.jsonl"
```

Every injected memory and every ACL denial leaves one **metadata-only** JSONL
event (`namespace`, `key`, `score`, `budget`, `truncated`, `acl` — no content):

```bash
jq -c 'select(.op=="injection" and .outcome=="denied")' injection-audit.jsonl
```

The MCP server exposes the same governance through
`VANTADB_MCP_INJECT_NAMESPACES` / `VANTADB_MCP_AUDIT_LOG` — see
[MCP injection governance](../api/MCP.md#injection-governance-ver-04).

## Prove it: the PII audit + the end-to-end demo

The versioned audit scans **raw bytes** of any path (file or directory) with
the same built-in detectors as `[redact]` — binary-safe: non-UTF-8 files
(journals, column data) are scanned byte-wise, never skipped — so store
records, derived indexes, WAL and column stores are covered in one sweep, plus
any extra file (e.g. an export). Output is value-free (`path`, `kind`,
`offset`):

```bash
cargo run -p vanta-proxy --bin vanta-pii-audit -- --json ./db ./export-v2.jsonl
```

| Exit | Meaning |
|------|---------|
| `0` | every byte of every file was scanned and nothing matched |
| `1` | findings and/or `unscanned` files (oversize, symlink, unreadable) — **not audited is not clean** |
| `2` | usage error |

Run it against a **closed** store: on Windows an open engine byte-range-locks
its lock files, which would surface as `unscanned` (honest, but noisy).

The full chain — capture with synthetic PII → audit (0 cleartext, with a
planted-leak negative control) → certified forget (`delete --attest` +
`certificate verify`, exit 0) → injection-audit consult — is one command:

```powershell
pwsh -NoProfile -File scripts/demo-privacy-e2e.ps1
```

It runs in CI (`.github/workflows/icp02-privacy-demo.yml`) on every change to
the proxy or the delete paths, and locally on Windows or Linux (pwsh 7).

## Prove it: WAL verification and injection governance

Two more demos run the same way — offline, deterministic, PR-blocking: if the
guarantee breaks, CI goes red.

### WAL tamper detection (`vanta-cli verify`)

The [WAL hash-chain](../api/WAL_INTEGRITY.md) is verifiable offline with the
real CLI: a clean WAL exits 0; a WAL with a record **removed** (chain link
broken) or **rewritten with its CRC32C recomputed** — invisible to CRC alone —
exits non-zero with `status: tampered`. The demo also runs the certified-delete
roundtrip (`put` → `delete --attest` → `certificate verify`, exit 0 ×3):

```powershell
pwsh -NoProfile -File scripts/demo-verify-e2e.ps1
```

Expected output (abridged):

```text
== [2/4] Clean WAL: vanta-cli verify -> exit 0
   clean: exit 0, ok:true, statuses: verified
== [3/4] Tampered WALs: vanta-cli verify -> exit != 0 (negative control)
   tampered_removed: exit 1, ok:false, statuses: tampered (tamper detected)
   tampered_rewritten: exit 1, ok:false, statuses: tampered (tamper detected)
== [4/4] Certified delete: put -> delete --attest -> certificate verify
   put + delete --attest + certificate verify: exit 0 x3
```

CI: [wal-verify-demo.yml](../../.github/workflows/wal-verify-demo.yml).

### Injection governance (budget, ACL, audit)

One governed request proves the three knobs of §4: the block stays within
`[injection] max_tokens` (60 in the demo — a visible `…[truncated]` cut), a
namespace outside `namespace_allow_prefixes` never reaches the prompt, and
both decisions leave consultable, metadata-only audit rows:

```powershell
pwsh -NoProfile -File scripts/demo-governance-e2e.ps1
```

Expected output (abridged):

```text
== [2/3] Audit consult: op=injection rows
   audit: 2 injection rows (1 ok+truncated, 1 denied)
== [3/3] Negative control: audit must not contain the payload marker
   metadata-only: PERSONA-MARKER absent from the audit
```

The audit rows for that run (timestamp omitted; one per decision, no payload):

| namespace | outcome | reason |
|-----------|---------|--------|
| `persona/sess-1` | `ok` | `…;kind=persona;budget=60/60;truncated=true;acl=allow` |
| `scene/sess-1` | `denied` | `…;budget=60/60;acl=deny;…` |

CI: [injection-governance-demo.yml](../../.github/workflows/injection-governance-demo.yml).

## Covered / not covered

| Claim | Status |
|-------|--------|
| Captured PII is masked before persistence (store + search-facing record) | ✅ verified by tests + the audit above |
| The original survives only inside the per-namespace AEAD envelope | ✅ (with the key; `disarmed` never writes it) |
| Certified purge per record with a verified per-surface inventory | ✅ logical purge — see limits below |
| Injection events are auditable without content | ✅ metadata-only JSONL |
| "0 cleartext" provable on a real store | ✅ `vanta-pii-audit` (value-free, fail-closed) |
| Secure erase (media / SSD remap / journals) | ❌ not claimed — logical/physical-entry purge only |
| Backups, snapshots, exports outside the store perimeter | ❌ not touched (export/import sandbox governs) |
| Parametric unlearning (embedding model weights) | ❌ never claimed |
| Retroactive redaction of records written before `[redact]` was enabled | ❌ not retroactive — new writes only |
| Redaction accuracy on all possible PII shapes | ⚠️ built-ins are heuristics; extend with `patterns` / audit with `--pattern` |

## Related

- [vanta-proxy reference — redaction, envelope, injection](../api/PROXY.md)
- [Certified delete (VER-02)](../api/CERTIFIED_DELETE.md)
- [WAL integrity / hash-chain (VER-01)](../api/WAL_INTEGRITY.md)
- [MCP server — injection governance](../api/MCP.md)
- [Honest comparison & limits](COMPARISON.md)
