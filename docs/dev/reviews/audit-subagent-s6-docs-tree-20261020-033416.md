---
title: "S6 — Deep Audit: Árbol de Docs"
kind: review
status: active
description: "Informe crudo del subagente S6: auditoría del árbol `docs/**` (API reference, guides, operaciones, ADRs) contra el código real. Base de los hallazgos `S6-*` del ledger."
tags: [audit, reviews, docs, subagent]
---

# Informe del subagente — DOCS TREE DEEP AUDIT (read-only)

**Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
**Scope:** `docs/**` (api/, dev/, user/, specs/, reports/) + root docs (CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, SUPPORT.md, llms.txt, NOTICE, CITATION.cff). README.md excluded (audited elsewhere).
**Date:** 2026-10-20
**Method:** read-only. No repo file was created, edited, or deleted. No git write command was run. `gen-index.mjs --write` was NOT run. The repo-provided checkers were run after verifying they contain no write calls.

---

## 0. Inventory & tooling evidence

| Item | Value |
|---|---|
| Markdown files under `docs/` | **1330** total (1109 non-archived, 221 archived) |
| Breakdown | `docs/dev` 1161 · `docs/user` 131 · `docs/api` 30 · `docs/specs` 3 · `docs/reports` 2 · `docs/CHANGELOG.md` · `docs/index.md` · `docs/README.md` |
| `docs/dev/tasks/` | 634 `.md` files (93 in `complete/`) |
| `docs/dev/architecture/adr/` | 59 files (57 ADRs + `README.md` + `DECISIONS-NOT-TAKEN.md`) |
| Root `llms.txt` | 132 KB, 481 links → 475 unique targets |

### Repo-provided checkers (run read-only)

| Checker | Result |
|---|---|
| `node scripts/docs/check-links.mjs` | **EXIT 0** — 1329 files scanned, 2669 internal links checked, **0 broken markdown links**, 6 wikilinks (budget 20), 373 external links skipped |
| `node scripts/docs/check-docs.mjs` | **EXIT 0** — all gating checks pass; reporting-only: 382 unknown frontmatter keys, 15 duplicate-basename groups, 2 orphans (`docs/dev/tasks/C2T1.md`, `docs/dev/tasks/complete/REC-001.md`), 10 wikilink files |
| `node scripts/docs/check-doc-examples.mjs` | **EXIT 0** — 1109 files, 672 code blocks (323 python / 349 rust); 56 runnable python; **1 documented-but-nonexistent** (`vantadb.VantaDB`, budget 1) |

**Key takeaway:** every existing gate passes. The findings below are **all classes the gates do not cover** — they check *link existence*, *frontmatter shape*, and *python attribute names*. None of them checks whether a **named type exists**, a **default value is right**, a **version is current**, a **workflow file exists**, or **two docs agree with each other**. That is where every finding below lives.

---

## 1. Findings

### D-01 | P0 | security-policy / root-doc
- **Location:** `SECURITY.md:26`
- **Claim/Break:** `SECURITY.md:26` — "**AES-256-GCM at-rest encryption** behind the `encryption` feature flag (`src/crypto.rs`)." Listed under "## Security Practices" as an active capability of the product.
- **Reality:** `src/storage/vfile.rs:439` — `pub fn with_cipher(mut self, cipher: Cipher) -> Self` is **never called from production code**; its only call site is `src/storage/vfile.rs:929`, inside a `#[cfg(test)]` block. The method's own doc at `src/storage/vfile.rs:436-438` says the storage layer *"should use the cipher"* — aspirational, not wired. `src/crypto.rs:8` repeats the aspirational wording (*"AES-256-GCM at-rest encryption for VantaDB storage"*). The repo's own canonical config doc states the opposite explicitly: `docs/user/operations/CONFIGURATION.md:75` — *"**Not yet applied to core storage files** — WAL/HNSW/VantaFile/backend-KV/snapshots remain plaintext today (FIND-249); the current consumer is the `vanta-proxy` envelope."* `vantadb-server/Cargo.toml:16-27` does not enable the `encryption` feature, so the HTTP server never compiles it.
- **Why it matters:** A security reviewer (or a user choosing VantaDB for sensitive data) reads `SECURITY.md` and concludes the database is encrypted at rest. It is not. This is a false security guarantee in the file a security auditor reads first. It also **contradicts the 0.9.0 changelog entry it ships in**: `docs/CHANGELOG.md:39` — *"*(server) SRV-10 - cifrado en reposo honesto: aviso key-sin-feature, feature opt-in encryption, **docs sin claim** (re-scope b; wiring=FIND-249)"* — the release notes assert the docs no longer carry the at-rest claim. `SECURITY.md` still carries it.
- **Fix:** Rewrite `SECURITY.md:26` to match `CONFIGURATION.md:75`: e.g. *"AES-256-GCM primitives exist behind the `encryption` feature (`src/crypto.rs`) and are used by the `vanta-proxy` envelope; **core storage files are not yet encrypted at rest** (FIND-249)."* Also fix the `src/crypto.rs:8` module doc, which is the other half of the false claim.
- **Verify:** `rg "at-rest|at rest" SECURITY.md docs/user/operations/SECURITY.md` → no unconditional at-rest claim; `rg "with_cipher" src/` → only the test call site until FIND-249 lands.

### D-02 | P1 | api-reference / config
- **Location:** `docs/user/operations/CONFIGURATION.md:15`, `:17`, `:176-177`, `:185`, `:187`
- **Claim/Break:** The document's entire framing uses a public type that **does not exist**: `CONFIGURATION.md:15` — "## 1. VantaConfig Reference"; `:17` — "All configuration fields available in `VantaConfig` (Rust)"; `:176` — "`VantaConfig::default()` reads **all** environment variables"; `:185` — `use vantadb::VantaConfig;`; `:187` — `let cfg = VantaConfig::from_env()`. 21 occurrences in the file.
- **Reality:** `src/lib.rs:177` — `pub use config::{Config, MAX_BATCH_SIZE, MAX_F32_VEC_LEN, MAX_K, MAX_VEC_DIM};`. The public type is `vantadb::Config`. A repo-wide search for `VantaConfig` across `src/`, `vantadb-server/`, `vantadb-python/`, `vantadb-mcp/`, `vanta-memory/` returns **zero hits** — the symbol was removed in 0.6.0. The repo's own API docs say so: `docs/api/EMBEDDED_SDK.md:15` — "`Vanta*` aliases (`VantaEmbedded`, `VantaConfig`, .) were removed in 0.6.0"; and `docs/api/EMBEDDED_SDK.md:133`/`:191`/`:206` correctly use `Config::bulk_commit_interval` / `Config::confidence_threshold`.
- **Why it matters:** This is the **normative configuration reference** — `docs/api/VERSIONING.md:44` points the CLI surface contract at it. The Rust snippet does not compile, and every field lookup ("what is the field called for X?") sends the reader to a type that isn't in the crate. It is also the single largest doc-vs-code type drift in the tree: 79 doc files still say `VantaConfig`, but the *API* docs were updated and the *user/operations* docs were not. `scripts/docs/check-doc-examples.mjs` cannot catch it — its own output says `rust ... 304 fragment` of 349 Rust blocks are classified `fragment` and Rust attribute existence is explicitly out of its scope.
- **Fix:** Global rename `VantaConfig` → `Config` in `docs/user/operations/CONFIGURATION.md` (21 sites), `docs/user/operations/GRAFANA_SETUP.md:55`, `docs/user/operations/EDITOR_INTEGRATIONS.md:352`, `docs/user/operations/hardening.md:98`. Update the §1 heading to "## 1. `Config` Reference". The 75 remaining `dev/` hits are historical notes — leave them, but add a one-line "superseded" note where they are presented as current.
- **Verify:** `rg "VantaConfig" docs/api docs/user` → 0 hits; `cargo build` the snippet in `CONFIGURATION.md:184-195` → compiles.

### D-03 | P1 | version-pin / api-reference
- **Location:** `docs/api/MCP.md:676`
- **Claim/Break:** `| version | tracks [workspace.package].version (0.5.0 as of this doc) |`
- **Reality:** `Cargo.toml:4` (`[workspace.package]`) — `version = "0.9.0"`. The file the doc points at as the tracked source, `server.json:6`, literally says `"version": "0.9.0"`. The **same file** says `docs/api/MCP.md:8` — "Current MCP implementation version: 0.9.0".
- **Why it matters:** Self-contradictory within one file, and the value is wrong. Consumers reading the MCP registry descriptor table get a version four releases behind. This is the "version tables" class the audit brief calls out.
- **Fix:** `docs/api/MCP.md:676` → "(`0.9.0` as of this doc)".
- **Verify:** `rg "0\.5\.0" docs/api/MCP.md` → 0 hits.

### D-04 | P1 | version-pin / quickstart
- **Location:** `docs/user/QUICKSTART.md:5`, `:11`, `:54-55`, `:113`, `:143`, `:161`
- **Claim/Break:** `:11` — "This quickstart validates the current **v0.8.0** boundary from pre-built artifacts"; `:113` — `pip install vantadb-py==0.8.0`; `:143` — `pip install ./dist/vantadb_py-0.8.0-*.whl`; `:161` — "TestPyPI carries the same version (`0.8.0`)".
- **Reality:** `Cargo.toml:4` — `version = "0.9.0"`; `docs/CHANGELOG.md:18` — `## [0.9.0](...) - 2026-10-20`; `vantadb-ts/package.json:3` — `"version": "0.9.0"`; `vantadb-node/package.json:3` — `"version": "0.9.0"`.
- **Why it matters:** The wheel glob at `:143` is a **literal shell glob**, not a version range — `vantadb_py-0.8.0-*.whl` will not match the `vantadb_py-0.9.0-*.whl` that a 0.9.0 build produces. The documented command fails outright. The `==0.8.0` pin at `:113` installs a release two minors behind the crate the rest of the repo documents. "the current v0.8.0 boundary" is false: current is 0.9.0.
- **Fix:** Update all six sites to 0.9.0, or (better) make them version-agnostic: `pip install ./dist/vantadb_py-*.whl` and `pip install "vantadb-py>=0.9.0"`.
- **Verify:** `rg "0\.8\.0" docs/user/QUICKSTART.md` → 0 hits; glob resolves against a real `dist/` wheel.

### D-05 | P1 | benchmark-source / operations
- **Location:** `docs/user/operations/BENCHMARKS.md:20`, `:22`
- **Claim/Break:** `:20` — "These metrics are run under the heavy certification suite (**`heavy-certification.yml`**, job `stress-protocol`)"; `:22` — "same invocation as the `stress-protocol` job in `heavy-certification.yml`".
- **Reality:** `.github/workflows/` contains 20 workflow files; **`heavy-certification.yml` does not exist** (the only filesystem hit for that name is `docs/dev/workflow/heavy-certification-50.md`, a prose doc). The `stress-protocol` job lives in `.github/workflows/certification.yml:89-103`, whose `run:` line is byte-identical to the command the doc quotes: `cargo test --release --test stress_protocol -- --nocapture --test-threads=1`.
- **Why it matters:** The document names the exact file a reader must open to re-verify the numbers. It does not exist, so the provenance chain that Regla 11 requires is broken. The *command* is right, which is why this survived — only the file name is wrong. `docs/CHANGELOG.md` and `docs/dev/avance/historial/` still use the old name in places, so the error is likely inherited rather than invented.
- **Fix:** `:20` and `:22` → `certification.yml`.
- **Verify:** `rg "heavy-certification" docs/user/` → 0 hits; `rg "stress-protocol" .github/workflows/` → `certification.yml:89`.

### D-06 | P1 | benchmark-number / operations
- **Location:** `docs/user/operations/BENCHMARKS.md:28`
- **Claim/Break:** `| **Recall@10 (Block 1)** | 10K vectors, 128d, Cosine | **0.9560** | Certified |`
- **Reality:** `tests/certification/stress_protocol.rs` — the Block 1 harness is literally titled *"BLOCK 1 - GROUND TRUTH RECALL (**50K**/128D)"* and calls `gen_dataset(50_000, DIMS, SEED)` with `DIMS: usize = 128` and a unique `m=16, ef_s=250` config that cannot be shared with the 10K index. So Block 1 measures **50 000 vectors**, not 10 000. The three "Scaling Recall" rows at `:29-31` *are* correctly attributed (10K/50K/100K) because they come from Block 2.
- **Why it matters:** The certified recall number is attributed to a different dataset size than the one measured. A reader comparing "Recall@10 = 0.9560 at 10K" against a competitor's 10K figure is comparing different workloads. The "Certified" status marker makes the mis-attribution more consequential, not less.
- **Fix:** `:28` → "50K vectors, 128d, Cosine". Re-run the protocol and re-publish if the value itself came from a different run.
- **Verify:** `rg "BLOCK 1" tests/certification/stress_protocol.rs` → `50K/128D`.

### D-07 | P1 | env-var / api-reference
- **Location:** `docs/api/EMBEDDINGS.md:129-130`
- **Claim/Break:**
  ```
  VANTA_EMBEDDING_PROVIDER=local
  VANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx
  ```
- **Reality:** Neither variable is read anywhere in the repo. A search across `src/`, `vantadb-python/src/`, `providers/`, `vantadb-server/`, `scripts/` for `VANTA_EMBEDDING_PROVIDER|VANTA_LOCAL_MODEL` returns only two **deprecation comments** (`src/config.rs:296`, `src/config.rs:691`), never an `env::var` call. The live names are `VANTADB_EMBEDDING_PROVIDER` (`src/config.rs:1110`) and `VANTADB_LOCAL_MODEL` (`src/config.rs:1090`). The repo's own migration note is explicit — `docs/user/operations/CONFIGURATION.md:109-120` lists `VANTA_EMBEDDING_PROVIDER` and `VANTA_LOCAL_MODEL` in the table headed *"The following legacy env vars are **deprecated** and no longer read by VantaDB"*, and `:106-107` states *"legacy `VANTA_*` reads have been **removed** from source."* `docs/api/EMBEDDINGS.md:45` itself uses the **correct** `VANTADB_*` form.
- **Why it matters:** The two commands are in a copy-paste block; setting them has **zero effect** — the provider silently stays at the `ollama` default (`src/config.rs:1110`) and the model path stays at the hardcoded default. A user following this block gets a *silent* misconfiguration, not an error. The doc contradicts itself 84 lines earlier.
- **Fix:** `:129` → `VANTADB_EMBEDDING_PROVIDER=local`; `:130` → `VANTADB_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx`.
- **Verify:** `rg "VANTA_EMBEDDING_PROVIDER|VANTA_LOCAL_MODEL" docs/ src/` → only the deprecation table in CONFIGURATION.md.

### D-08 | P1 | counter / backlog
- **Location:** `docs/dev/Backlog.md:356`
- **Claim/Break:** FIND-98 — "**Binario `vanta-cli` instalado stale (79 vs 85 tools)**" and in the body "sirve 79 tools vs **85** del fuente (re-chequeado 2026-09-18: instalado 79; fuente 87÷85 tras API-04 dedup...)".
- **Reality:** `vantadb-mcp/tests/mcp_tests.rs` — `test_mcp_tool_profiles` asserts `full_tools.len() == 82` with the message *"Full profile should list 82 tools (88 defined - 6 WIRE-02 absorbed), got {}"*, and `agent_tools.len() == 39`. `vantadb-mcp/src/config.rs:27` — `/// Full profile (82 listed tools)`. An independent enumeration of every `"name":` in the nine tool-definition modules (`handlers/tools.rs` 49 + `skills.rs` 7 + `code.rs` 8 + `code_index.rs` 1 + `wiki.rs` 6 + `context.rs` 1 + `dreams.rs` 5 + `scenes.rs` 5 + `threads.rs` 6) yields **88 unique tool names**, matching the code's "88 defined".
- **Why it matters:** The backlog row is the *action item* for a tool-parity defect, and it is open (`?? Media`, "falta el cierre formal y el smoke del instalado"). A reader acting on it would compare against 85 and find a 3-tool discrepancy that does not exist, and would chase a phantom "API-04 dedup to 85". The correct numbers are 79 vs 82.
- **Fix:** `:356` → "(79 vs 82 tools)" and "fuente 82 (88 definidas - 6 absorbidas WIRE-02)".
- **Verify:** `cargo test -p vantadb-mcp test_mcp_tool_profiles` → 82/39/38/22.

### D-09 | P1 | counter / backlog-negocio
- **Location:** `docs/dev/Backlog-negocio.md:101`
- **Claim/Break:** DX-10 — "pipe verificado: **80 tools listadas - 86 definidas - 6 absorbidas** WIRE-02, `docs/api/MCP.md:198`".
- **Reality:** Same ground truth as D-08: 82 listed, 88 defined, 6 absorbed. `docs/api/MCP.md:198` — the cited line — says "**82** listed tools in 8 families (88 defined − 6 WIRE-02 absorbed...)" and its family table (`:200-209`) sums to exactly 82 and 88. So the backlog row cites `MCP.md:198` as its source and then states numbers that line contradicts (80/86 instead of 82/88).
- **Why it matters:** Same class as D-08 but worse: the row **cites the correct source line** and still misquotes it, which is strong evidence the backlog is maintained by hand and not re-derived. A reviewer who opens the cited line finds the answer immediately and has to reverse-engineer where "80/86" came from.
- **Fix:** → "82 tools listadas - 88 definidas - 6 absorbidas".
- **Verify:** The arithmetic must be self-consistent: `88 - 6 = 82`.

### D-10 | P1 | default-value / operations
- **Location:** `docs/user/operations/CONFIGURATION.md:37`
- **Claim/Break:** `| max_blocking_threads | usize | **16** | VANTADB_MAX_BLOCKING_THREADS | Max threads for blocking thread pool |`
- **Reality:** `src/config.rs:1050-1052` —
  ```rust
  let default_max_blocking = std::thread::available_parallelism()
      .map(|n| n.get() * 2)
      .unwrap_or(16);
  ```
  The default is **2 × logical CPU count**; `16` is only the fallback when `available_parallelism()` errors. `src/config.rs:1209` then applies it: `parse_env_or("VANTADB_MAX_BLOCKING_THREADS", default_max_blocking)`. The same helper exists at `src/config.rs:333-338` (`fn default_max_blocking_threads()`).
- **Why it matters:** On a 32-core CI runner the real default is 64, not 16 — a 4× difference in the blocking pool, which feeds `max_connections` (`src/config.rs:1214`, `default_max_blocking * 2`) and therefore the HTTP query-pool size. An operator sizing the server against "16" is sizing it against a number that only materialises on a machine where CPU detection failed.
- **Fix:** `:37` → Default: `2 × available_parallelism()` (fallback 16).
- **Verify:** `rg "available_parallelism" src/config.rs` → the two sites above.

### D-11 | P1 | default-value / operations
- **Location:** `docs/user/operations/CONFIGURATION.md:64`
- **Claim/Break:** `| wal_buffer_size | Option<usize> | **65536 (64KB)** | VANTADB_WAL_BUFFER_SIZE | Per-shard WAL buffer in bytes (None = OS default) |`
- **Reality:** `src/config.rs:1406-1413` — `parse_env_or::<u32>("VANTADB_WAL_BUFFER_SIZE", 0).try_into().ok().filter(|&n| n > 0)` → the default is `0` → filtered out → **`None`**. The second default impl agrees: `src/config.rs:344` — `wal_buffer_size: None`.
- **Why it matters:** The row is **self-contradictory**: the Default column says 64 KB, the Description column says "None = OS default". The truth is `None` (OS default) — there is no 64 KB default anywhere. A reader who believes 65536 is the default will conclude the WAL is pre-buffered when it is not, and will mis-tune `VANTADB_WAL_BUFFER_SIZE` accordingly. With 4 shards default (`wal_shards: 4`), the 256 KB-vs-0 buffer difference is a real I/O-behaviour delta.
- **Fix:** `:64` → Default: `None` (OS default).
- **Verify:** `rg "wal_buffer_size" src/config.rs` → `:344` and `:1406` both `None`.

### D-12 | P1 | version-pin / api-reference
- **Location:** `docs/api/MCP.md:21`
- **Claim/Break:** "- The VantaDB CLI (`vanta-cli` — **0.5.0**) installed and on PATH - see [Embedded CLI](../../../README.md#embedded-cli) for one-line installers."
- **Reality:** `Cargo.toml:4` — `version = "0.9.0"`. `docs/api/MCP.md:8` — "Current MCP implementation version: 0.9.0".
- **Why it matters:** A hard prerequisite pinned to a version four releases old, in the "Getting Started → Requirements" list of the MCP reference. A user on 0.5.0 following the doc gets an MCP server whose `tools/list` surface predates WIRE-02, API-04 and the `code_*`/`wiki_*`/`skill_*` families the rest of the document describes.
- **Fix:** → "(`vanta-cli` — **0.9.0**)" or "≥ 0.9.0".
- **Verify:** `rg "0\.5\.0" docs/api/` → 0 hits.

### D-13 | P1 | security-policy / root-doc
- **Location:** `SECURITY.md:3-11`
- **Claim/Break:**
  ```
  | Version | Supported          |
  | 0.5.x   | ? Active development |
  | 0.4.x   | ?? Security patches only |
  | < 0.4   | ? Not maintained   |
  ```
  plus `:13` — "Patch releases are published for the latest minor version."
- **Reality:** Current version is **0.9.0** (`Cargo.toml:4`; `docs/CHANGELOG.md:18` — `## [0.9.0] ... - 2026-10-20`). There have been four minor releases since 0.5.0 (0.6, 0.7, 0.8, 0.9), each documented in `docs/CHANGELOG.md`.
- **Why it matters:** This table is the **support contract for security fixes**. As written it says 0.5.x is the actively-developed line — so 0.6.x through 0.9.x are unclassified, and 0.4.x is claimed to receive security patches even though "Patch releases are published for the latest minor version" (0.9.x) contradicts it. A user on 0.9.0 cannot determine from this file whether they are supported; a user on 0.4.x is told they are.
- **Fix:** Rewrite the table for the 0.9.x line: `0.9.x` active, `0.8.x` security-patches-only, `< 0.8` not maintained — and state the policy in terms of "current minor" rather than a frozen list, so it does not rot again.
- **Verify:** Table minor lines match the current `Cargo.toml` version.

### D-14 | P1 | citation-metadata / root-doc
- **Location:** `CITATION.cff:9-10`
- **Claim/Break:** `version: 0.5.0` / `date-released: 2026-08-01`. The file's own instruction at `:3` — *"Al publicar una release nueva, actualizar `version` + `date-released`."*
- **Reality:** `Cargo.toml:4` — `version = "0.9.0"`; `docs/CHANGELOG.md:18` — `## [0.9.0](...) - 2026-10-20`.
- **Why it matters:** GitHub's "Cite this repository" button renders `CITATION.cff` verbatim. A user citing VantaDB for a 0.9.0-era result would cite version 0.5.0 — four releases and an entire year earlier. The file documents its own update procedure and it was not applied for four releases, which means the procedure is not enforced.
- **Fix:** `version: 0.9.0`, `date-released: 2026-10-20`. Consider adding `version: 0.9.0` to a CI check (the file already carries an explicit instruction that no tool enforces).
- **Verify:** `cffconvert --validate CITATION.cff` and compare `version` to `Cargo.toml`.

### D-15 | P1 | env-var coverage / operations
- **Location:** `docs/user/operations/CONFIGURATION.md:19-92` (the `VantaConfig` reference table) and `:122-129` (the "Environment Variables Outside `VantaConfig`" table)
- **Claim/Break:** The document is presented as the complete environment-variable reference: `:17` — "All configuration fields available in `VantaConfig` (Rust) **and via environment variables**"; `:122` — "These env vars are read at runtime outside `VantaConfig::from_env()`:". Six variables read by the code appear in **neither** table.
- **Reality:** Enumerated from the source:

  | Env var | Read at | Documented? |
  |---|---|---|
  | `VANTADB_QUIET` | `src/hardware/mod.rs:303` | ❌ |
  | `VANTADB_SILENT` | `src/hardware/mod.rs:304` | ❌ |
  | `VANTADB_PROFILE_PATH` | `src/hardware/mod.rs:99`, `:732` | ❌ |
  | `VANTADB_SCHEDULER_INTERVAL_MS` | `vantadb-server/src/scheduler.rs:70` | ❌ |
  | `VANTADB_INGEST_CONFIG` | `vantadb-server/src/scheduler.rs:93`, `vantadb-mcp/src/wiki.rs:389` | ❌ |
  | `VANTADB_SYNC_MODE` | `src/cli_handlers/config.rs:216` | ❌ (table row `:42` says Env Var "—") |

  The first three are also *written* by the binaries themselves (`src/bin/vanta-cli.rs:34` and `vantadb-server/src/main.rs:55` both `set_var("VANTADB_QUIET", "1")`).
- **Why it matters:** `VANTADB_INGEST_CONFIG` selects the ingest TOML that the scheduler and the MCP wiki ingester both read — an operator cannot discover it, cannot point it at a non-default path, and cannot audit which config the server is honouring. `VANTADB_SCHEDULER_INTERVAL_MS` controls the background pass cadence. `VANTADB_QUIET`/`VANTADB_SILENT`/`VANTADB_PROFILE_PATH` control the hardware banner and the performance profile — the same knobs `docs/user/operations/PERFORMANCE_TUNING.md` would need.
- **Fix:** Add a sixth row to the "Environment Variables Outside `VantaConfig`" table (`:124-129`) for `VANTADB_QUIET`, `VANTADB_SILENT`, `VANTADB_PROFILE_PATH`, `VANTADB_SCHEDULER_INTERVAL_MS`, `VANTADB_INGEST_CONFIG`; and correct `:42` (`sync_mode`) to `VANTADB_SYNC_MODE`.
- **Verify:** Diff the `env::var("...")` set in `src/` + `vantadb-server/` + `vantadb-mcp/` against the two tables — every hit must appear.

### D-16 | P2 | cli-flag / api-reference
- **Location:** `docs/api/EMBEDDINGS.md:60`
- **Claim/Break:** "Synthetic fallback is permitted only when `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` **or `--allow-dummy-embeddings` is supplied**."
- **Reality:** `--allow-dummy-embeddings` is **not a flag of any binary**. `src/cli.rs` declares the complete global flag set as `--db`/`-d`, `--verbose`/`-v`, `--memory-limit`, `--json` (`:15-38`) and the per-subcommand flags; there is no `allow-dummy-embeddings` anywhere in the file. A repo-wide search of `src/` finds the string only as the **env var** `VANTADB_ALLOW_DUMMY_EMBEDDINGS` (`src/config.rs:1117`, `src/error.rs:430`, `src/llm.rs:565`). The same non-existent flag is repeated in `docs/dev/tasks/FIND-381.md:33`.
- **Why it matters:** A user who follows the doc and passes `--allow-dummy-embeddings` gets an `unexpected argument` error from clap. The env-var half of the sentence is correct, which makes the flag half look equally authoritative.
- **Fix:** Delete the `--allow-dummy-embeddings` clause from `EMBEDDINGS.md:60` and `FIND-381.md:33`, or add the flag to `src/cli.rs` if it was intended to exist.
- **Verify:** `rg "allow.dummy.embeddings" src/cli.rs` → hit, or the doc clause is gone.

### D-17 | P2 | broken-code-example / user-blog
- **Location:** `docs/user/blog/ollama_vantadb_local_memory.md:45`
- **Claim/Break:** The snippet uses `vantadb.VantaDB`.
- **Reality:** `scripts/docs/check-doc-examples.mjs` output: *"DOCUMENTED BUT NOT EXISTENT (1, budget 1; 1 via import, 0 via attribute): docs/user/blog/ollama_vantadb_local_memory.md:45 vantadb.VantaDB -> no name 'VantaDB' on vantadb"*. The Python SDK exposes `Client` (the same checker's verified set lists `Client`), not `VantaDB`.
- **Why it matters:** It is the **only** entry in the repo's documented-but-nonexistent budget, and the budget was set to 1 — i.e. the gate was relaxed to accommodate this one defect rather than the defect being fixed. A blog post is exactly the surface an LLM or a new user will copy from. Note this is the *same* 0.6.0 rename that produced D-02, resurfacing on a different surface.
- **Fix:** `vantadb.VantaDB` → `vantadb.Client`; then drop `--max-missing=1` to `0` in `docs/dev/workflow/gate-doc-examples.md` so the budget cannot absorb a second one.
- **Verify:** `node scripts/docs/check-doc-examples.mjs --max-missing=0` → exit 0 with 0 findings.

### D-18 | P2 | ci-duration / contributing
- **Location:** `CONTRIBUTING.md:164-165`
- **Claim/Break:** "2. **Heavy Certification** (manual/scheduled): stress, HNSW validation, SIFT, chaos, WAL resilience - **up to 2h**, never in the Fast Gate."
- **Reality:** `.github/workflows/certification.yml:94` — the `stress-protocol` job sets `timeout-minutes: 180` (3 h). Other jobs in the same file (`multi-namespace-stress`) use `timeout-minutes: 30`. The repo's own `nightly.yml:4` comment also says "stress_protocol ~2h" — so the 2 h figure is a *rough expectation* the workflow no longer enforces.
- **Why it matters:** A contributor planning a release window budgets 2 h for Heavy Certification; the job's hard ceiling is 3 h, so a run that hits the ceiling is cut off at 3 h, not 2 h. Low blast radius, but it is exactly the "reproduce the timing" class of claim.
- **Fix:** → "up to 3h (job timeout 180 min in `certification.yml`)".
- **Verify:** `rg "timeout-minutes" .github/workflows/certification.yml`.

### D-19 | P2 | env-var / operations
- **Location:** `docs/user/operations/CONFIGURATION.md:42`
- **Claim/Break:** `| sync_mode | SyncMode | Periodic | **—** | wal sync: Always, Periodic, Never |` — the "—" asserts there is no env var for this field.
- **Reality:** `src/cli_handlers/config.rs:216` — the 5-layer configuration resolver reads it:
  ```rust
  resolve_item("sync_mode", "VANTADB_SYNC_MODE", "periodic", false, None, &project_map, &user_map);
  ```
  So `VANTADB_SYNC_MODE` is honoured by `vanta config show` / the 5-layer cascade documented at `CONFIGURATION.md:131-146`.
- **Why it matters:** The table and the 5-layer section of the *same document* disagree about whether the field is env-tunable. `SyncMode::Always` is the durability-critical setting (fsync on every write) — an operator looking for the switch cannot find it in the reference table.
- **Fix:** `:42` → Env Var: `VANTADB_SYNC_MODE` (5-layer resolver). Also reconcile with `:19-92`'s claim to list *all* env vars.
- **Verify:** `rg "VANTADB_SYNC_MODE" src/` → `src/cli_handlers/config.rs:216`.

### D-20 | P2 | adr-metadata / llms-txt
- **Location:** `docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md:5` (frontmatter `description`), and the derived line in `llms.txt` (ADR-0011 entry)
- **Claim/Break:** `description: "VantaDB exposes dense-only vector search (VantaMemorySearchRequest.queryvector: Vec<f32>)"`. `llms.txt` reuses each ADR's `description` as the link's prose summary, so the false claim is republished verbatim in the machine-readable index.
- **Reality:** Sparse vectors are native and shipped: `src/node/vector_data.rs:30` — `pub struct SparseVector(pub BTreeMap<u32, f32>);`; the field is threaded through the SDK at `src/sdk/api/memory.rs:734` and `:1048`; the ADR's own **Decision** section (`:30-38`) mandates exactly that representation, and `DistanceMetric::SparseDot`. The MCP surface carries `sparse_vector` in its schemas.
- **Why it matters:** The ADR title is *"Native sparse vectors"* and its Decision adds them — but the `description` still describes the **pre-decision** state, and it is the field `llms.txt` publishes. An agent reading `llms.txt` concludes VantaDB is dense-only and will not use the sparse path, or will conclude the ADR and the code disagree.
- **Fix:** Rewrite the frontmatter `description` to the *decision* ("Sparse vectors are native: `SparseVector(BTreeMap<u32,f32>)` with `DistanceMetric::SparseDot`"), then regenerate `llms.txt`. Consider an ADR-description lint that rejects past-tense "X does not exist" phrasing in an accepted ADR.
- **Verify:** `rg "dense-only" llms.txt docs/dev/architecture/adr/` → 0 hits.

### D-21 | P3 | adr-reference / architecture
- **Location:** `docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md:36`
- **Claim/Break:** "Representation is `SparseVector(BTreeMap<u32, f32>)` (dimension → value), defined in **`src/node.rs`** next to `DistanceMetric`."
- **Reality:** `src/node/vector_data.rs:30` — the type lives in `src/node/vector_data.rs`; `src/node.rs` no longer exists as a file (it is now the `src/node/` module directory: `src/node/vector_data.rs`, `src/node/...`).
- **Why it matters:** A reader following the pointer lands on a nonexistent path. The repo-wide path scan also flags `src/node.rs` as a missing file in 66 places across the docs tree — this ADR is one of them.
- **Fix:** → `src/node/vector_data.rs`.
- **Verify:** `rg "src/node\.rs" docs/dev/architecture/adr/` → 0 hits.

### D-22 | P3 | benchmark-internal-consistency / operations
- **Location:** `docs/user/operations/BENCHMARKS.md:34-36`
- **Claim/Break:** `p50 Latency (10K) = 1.2 ms`, `p50 Latency (50K) = 6.1 ms`, `Scaling Factor | Latency growth (10K → 50K) | **4.88x**`.
- **Reality:** 6.1 / 1.2 = **5.08×**. The published 4.88× is consistent with an unrounded 1.25 ms p50 (`6.1 / 4.88 = 1.25`), i.e. the factor was computed pre-rounding while the latencies were published post-rounding. The stress protocol's own threshold comment at `tests/certification/stress_protocol.rs` (Block 7) says *"Threshold: 8.0x … Theoretical HNSW: ~1.7x for 5x data. Practical observed: 2.6x-5.6x"* — so 4.88x is inside the accepted band, and the code's own observed range (2.6–5.6×) does not reconcile with the displayed 5.08× either.
- **Why it matters:** The scaling-factor row is the one number in the table a reader recomputes by hand, and it does not divide out. It is the headline "sub-linear" claim. Not a wrong measurement — a presentation defect that reads as an error.
- **Fix:** Publish the factor to the precision of its inputs (e.g. `~5.1x`) or publish the unrounded p50 alongside.
- **Verify:** 6.1 / 1.2 ≈ 5.1×.

### D-23 | P3 | task-status-vocabulary / dev-tasks
- **Location:** `docs/dev/tasks/` (634 files) — frontmatter `status:` field
- **Claim/Break:** The task tree carries a `status:` frontmatter field whose vocabulary is not single-valued. Distribution across all 634 files: `active` 309 · `completed` 235 · `pending` 66 · `in_progress` 11 · `in-progress` 6 · no status 7.
- **Reality:** `in_progress` (11 files) and `in-progress` (6 files) are two spellings of the same state. `docs/dev/Backlog.md` documents the row format as `ID | Description | Files | Effort | Priority | Status` (and `CONTRIBUTING.md:246` repeats it) with a single canonical status vocabulary, and `docs/dev/avance/COBERTURA.md` is generated from a script (`scripts/check-avance-coverage.ps1`, which exists) — so the tree is machine-processed.
- **Why it matters:** Any filter or rollup keyed on `status == "in_progress"` silently drops 6 files. With 309 of 634 tasks marked `active`, the tree also cannot be used to answer "what is actually in flight" without a manual pass — which is precisely the question the `progreso` skill exists to answer.
- **Fix:** Normalise the 6 `in-progress` files to `in_progress`, and add the vocabulary to `docs/_schema/frontmatter.schema.json` so the gate rejects a non-canonical value.
- **Verify:** `rg "^status: in-progress" docs/dev/tasks/` → 0 hits; schema enumerates the allowed set.

### D-24 | P3 | llms-txt-provenance / root-doc
- **Location:** `llms.txt:1-14` (header) and the whole file
- **Claim/Break:** `llms.txt:11-13` — "This index covers the **475 documentation pages**; it deliberately omits the **634 task files**… The full inventory of every file is in `docs/index.md` in the repository." The header reads as a generated artefact.
- **Reality:** **The counts are exactly right** — 475 unique `docs/**.md` targets (481 link occurrences), 634 files in `docs/dev/tasks/`, and 475 + 634 = 1109 = the exact non-archived file count reported by `check-links.mjs` ("1329 (1109 in scope)"). All 481 linked paths resolve on disk (0 missing). But **no generator exists**: a search of `scripts/`, `dev-tools/`, `package.json` scripts, and all workflow files finds no script that writes `llms.txt`; `llms.txt` is not in `.gitignore`; and `scripts/docs/gen-index.mjs` writes `docs/index.md`, not `llms.txt`.
- **Why it matters:** The content is correct, but it is hand-maintained (or generated by an out-of-tree tool) with no drift gate. The 3 points it omits — the 221 archived files, and the fact that it is 132 KB of hand-linked markdown — are exactly what an LLM indexer needs to know. Contrast `docs/index.md`, which *is* generated and *is* checked.
- **Fix:** Either commit a generator (`scripts/docs/gen-llms-txt.mjs`, modelled on `gen-index.mjs`) and add it to `docs-gates.yml`, or state in the header that the file is hand-maintained and how it is refreshed. The header should also say it omits the 221 archived files.
- **Verify:** A generator run reproduces the file byte-for-byte; `check-links.mjs` covers its internal links.

### D-25 | P3 | version-illustration / api-reference
- **Location:** `docs/api/VERSIONING.md:17-19`
- **Claim/Break:** "- **MINOR** releases (`0.5.0` → `0.6.0`) **may contain breaking changes**… - **PATCH** releases (`0.6.0` → `0.6.1`) are always backward-compatible".
- **Reality:** The policy text is correct, but the illustrative version pairs are four releases behind (`Cargo.toml:4` — `0.9.0`). Everything else in the file is current: `IQL_VERSION = 4` matches `src/parser/mod.rs:20` ✓; the 11-surface inventory ✓; the `§4 Embedded CLI` cross-reference resolves to `docs/user/operations/CONFIGURATION.md:370` ✓; the `Embedded` entry point matches `src/lib.rs:189` ✓.
- **Why it matters:** Minor. A reader could take `0.5.0 → 0.6.0` as the currently-supported boundary rather than as a semver illustration.
- **Fix:** Use `0.9.0 → 0.10.0` / `0.9.0 → 0.9.1`, or generic placeholders (`0.x.0 → 0.(x+1).0`).
- **Verify:** —

### D-26 | P3 | tool-drift-gate / cited-source
- **Location:** `skills/vantadb-mcp/scripts/test-mcp.py:26-28` and `:60-61` (the drift gate the docs cite as source of truth)
- **Claim/Break:** The script's docstring says *"TOOL PROFILES (VANTADB_MCP_PROFILE env var, default "full"): full (default): **85 tools** exacto; dev: **36 tools** exacto; memory: **20 tools** exacto"* and `EXPECTED_TOOLS = {"full": (85, 85), "dev": (36, 36), …}`. It self-describes as the "Drift gate (FIND-82): expected surface per tool profile. Source of truth: `vantadb-mcp/tests/mcp_tests.rs::test_mcp_tool_profiles`".
- **Reality:** The cited source of truth asserts different numbers: `vantadb-mcp/tests/mcp_tests.rs` — `full_tools.len() == 82` ("88 defined - 6 WIRE-02 absorbed"), `agent_tools.len() == 39`, and dev = 38 ("memory set (22) + dev-only tools (16) = 38. The original 35 cap was a Cursor budget heuristic, not a contract"). `vantadb-mcp/src/config.rs:22-33` — Agent (39) / Full (82) / Dev (38) / Memory (22).
- **Why it matters:** The script is **stale against the code it claims to mirror**, so the drift gate exits 1 against a correct binary (a false alarm) — and, worse, it would *pass* against a binary that regressed to 85/36/20. The documented counts in `docs/api/MCP.md` (82/39/38/22) are correct, so the doc and the gate disagree. This is a documentation-contract failure: `docs/api/MCP.md` and the changelog both treat this script as the parity reference.
- **Fix:** Update `EXPECTED_TOOLS` to `{"full": (82, 82), "agent": (39, 39), "dev": (38, 38), "memory": (22, 22)}` and the docstring counts; add `agent` to the profile table (it is missing entirely, and `agent` is the **default** profile per `vantadb-mcp/src/config.rs:23-25`).
- **Verify:** `python skills/vantadb-mcp/scripts/test-mcp.py target/debug/vantadb-server.exe` → exit 0; a `--skip` of one tool → exit 1.

---

## 2. Areas checked and found CORRECT (no finding)

Recording these so the coverage is auditable and so a future reader does not re-audit them.

| Claim | Doc | Verified against |
|---|---|---|
| MCP tool counts (82 listed / 88 defined / 39 agent / 38 dev / 22 memory), 8-family table summing to 82 and 88 | `docs/api/MCP.md:198-240, :276` | `vantadb-mcp/tests/mcp_tests.rs::test_mcp_tool_profiles`; `vantadb-mcp/src/config.rs:21-35`; independent enumeration of all 9 tool-definition modules = 88 unique names ✓ |
| OpenAPI paths (38) vs axum router (35 API routes + `/health` + `/metrics` + `/dashboard` + `/dashboard/{path}`) | `docs/api/openapi.yaml` | `src/server/router.rs:176-283` — every route present, no phantom path, no missing path ✓ |
| HTTP_API.md endpoint inventory (all 40 headers, incl. the 8 graph endpoints in the table at `:510-517`) | `docs/api/HTTP_API.md` | `src/server/router.rs` ✓ |
| `VantaConfig` field defaults: `rss_threshold` 0.80, eviction weights 1.0/2.0/3.0/1.0, `eviction_ratio` 0.20, `port` 8080, `host` 127.0.0.1, `storage_path` vantadb_data, `llm_url` http://localhost:11434, `llm_model` all-minilm, `llm_summarize_model` llama3, `local_model_path` …/multilingual-e5-small/onnx, `openai_model` text-embedding-3-small, `embedding_provider` ollama, `strict_embeddings` true, `wal_shards` 4, `flat_threshold` 10000, `sync_mode` Periodic, `pool_acquire_timeout_ms` 5000, `circuit_breaker_*` 5/30, `insert_lock_timeout_ms` 5000, `file_lock_timeout_ms` 1000, `rate_limit_rpm` 600, `ttl_sweep_interval_ms` 60000, `quarantine_review_default_days` 30, `audit_max_bytes` 10 MiB, `audit_max_files` 5, `version_history_limit` Some(32), `self_healing` true / `self_healing_quiet_secs` 300, `insert_batch` {false,32,1,1024}, `segment_optimizer` {true,15.0,3600,300} | `docs/user/operations/CONFIGURATION.md:19-92` | `src/config.rs:1048-1519` (`impl Default for Config`), `src/config.rs:337-357` (`StorageCfg`), `src/storage/engine/mod.rs:278` (`SegmentOptimizerConfig::default`) — **all match** except D-10, D-11, D-19 ✓ |
| Auth-failure rate limit "5 attempts / 60 s window" | `SECURITY.md:29` | `src/server/state.rs:288` — `AuthRateLimiter::new(5, 60)`; wired at `src/server/middleware.rs:86, 134, 298, 323` ✓ |
| Constant-time token comparison | `SECURITY.md:27` | `subtle::ConstantTimeEq` present ✓ |
| RBAC roles admin/reader/writer, "write covers delete in 0.9.0" | `SECURITY.md:30`, `CONFIGURATION.md:81` | `src/server/router.rs:133-135`; `src/server/middleware.rs:251` maps DELETE → `AccessMode::Write` ✓ |
| Per-IP rate limiting, `rate_limit_rpm` default 600 | `CONFIGURATION.md:48` | `src/config.rs:1266`; `src/server/router.rs:196-201` ✓ |
| `llms.txt` counts 475 pages / 634 task files, all 481 links resolve | `llms.txt:11-13` | 475 unique targets, 634 task files, 0 missing, 475+634=1109=in-scope count ✓ (but see D-24 on provenance) |
| `docs/api/index.md:15` "30 documents" | `docs/api/index.md:15` | exactly 30 `.md` in `docs/api/` ✓ |
| `IQL_VERSION = 4` | `docs/api/VERSIONING.md:41` | `src/parser/mod.rs:20` ✓ |
| 11 public API surfaces; `§4 Embedded CLI` cross-ref; `Embedded` type | `docs/api/VERSIONING.md:33-45` | `src/lib.rs:188` (`pub use sdk::{... Embedded ...}`); `docs/user/operations/CONFIGURATION.md:370` ✓ |
| CLI subcommands used across `docs/api` + `docs/user` (29 distinct: server, verify, import, doctor, rebuild-index, backup, restore, audit-index, status, count, export, delete, migrate, stats, search, namespace, mcp-call, certificate, repair-text-index, get, put, list, similar-to-key, completions, tui, …) | `docs/api/`, `docs/user/` | `src/cli.rs:47-397` (`enum Commands`), `:403-435` (`ConfigCommand`), `:437-455` (`NamespaceCommand`), `:458-470` (`SnapshotCommand`), `:472-500` (`MigrateCommand`), `:503-520` (`WalCommand`), `:523-535` (`CertificateCommand`) — **all 29 exist** ✓ |
| Global CLI flags `--db`/`-d`, `--verbose`/`-v`, `--memory-limit`, `--json`; per-command flags `--attest`, `--out`, `--filter`, `--namespace`, `--key`, `--payload`, `--vector`, `--metadata`, `--limit`, `--in`, `--query-vector`, `--top-k`, `--format`, `--dry-run`, `--force`, `--fix`, `--shell`, `--reveal-secrets`, `--global`, `--deep` | `src/cli.rs` | `src/cli.rs:15-38`, `:47-535` ✓ |
| `benchmarks/vantadb_benchmark_report.json` is gitignored and not a valid source | `docs/user/operations/BENCHMARKS.md:78-79` | `.gitignore:251` ✓ (the file exists on disk but is correctly declared machine-local) |
| Stress protocol reproduce command | `docs/user/operations/BENCHMARKS.md:22` | byte-identical to `.github/workflows/certification.yml:103` ✓ (only the workflow *filename* is wrong — D-05) |
| `[[test]] name = "stress_protocol"` registration + `rayon` in default features | `docs/user/operations/BENCHMARKS.md:22` | `Cargo.toml:478-480`; `Cargo.toml` `[features] default = [… "rayon"]` ✓ |
| `fuzz` targets `fuzz_parser`, `fuzz_node_deserialize`, `fuzz_wal`, `fuzz_archive` | `CONTRIBUTING.md:181-186` | `fuzz/Cargo.toml` `[[bin]]` × 4 ✓ (files live at `fuzz/fuzz_targets/*.rs`; the doc names targets, not paths) |
| `just verify` = fmt + clippy + test + deny; `just verify-quick` | `CONTRIBUTING.md:52-63` | `Justfile:52, :55` ✓ |
| `nextest` profiles `audit` and `experimental` | `CONTRIBUTING.md:39, :42` | `.config/nextest.toml` `[profile.audit]`, `[profile.experimental]` ✓ |
| All CONTRIBUTING file references (`rust-toolchain.toml`, `dev-tools/setup_venv.{ps1,sh}`, `vantadb-python/tests/test_sdk.py`, `deny.toml`, `.pre-commit-config.yaml`, `.githooks/pre-commit`, `.githooks/pre-push`, `cliff.toml`, `docs/api/VERSIONING.md`, `docs/dev/workflow/RULES.md`, `docs/dev/Backlog.md`, `.github/ISSUE_TEMPLATE/{config,feature_request}.yml`, `dev-tools/scripts/validate_python_sdk.{ps1,sh}`) | `CONTRIBUTING.md` | 20/20 exist on disk ✓ |
| Changelog generation: release-plz `changelog_update = true`, `changelog_path = "docs/CHANGELOG.md"`; Justfile `changelog: git-cliff -o docs/CHANGELOG.md` | `CONTRIBUTING.md:138, :147`; root `CHANGELOG.md` | `release-plz.toml:46, :50`; `Justfile` ✓ (both statements true — release-plz delegates to git-cliff) |
| Root `CHANGELOG.md` is a pointer stub; `docs/CHANGELOG.md` is the real changelog, latest `0.9.0 / 2026-10-20` | root `CHANGELOG.md` | `docs/CHANGELOG.md:18` ✓ — **the two files are consistent**; AGENTS.md points to `docs/CHANGELOG.md` |
| `docs/progreso/` was removed; its content lives under `docs/dev/avance/` | `docs/dev/avance/COBERTURA.md` | `Test-Path docs/progreso` → False ✓; `scripts/check-avance-coverage.ps1` exists ✓ |
| `docs/api/EMBEDDED_SDK.md` deprecation notice (`Vanta*` aliases removed in 0.6.0) and its use of `Config::` | `docs/api/EMBEDDED_SDK.md:15, :133, :191, :206` | `src/lib.rs:177` ✓ — **this doc is correct**; it is `docs/user/operations/CONFIGURATION.md` that drifted (D-02) |
| SUPPORT.md links (`docs/user/QUICKSTART.md`, `docs/user/FAQ.md`, `docs/api/`) | `SUPPORT.md:14-16` | all resolve ✓ |
| NOTICE references `LICENSE` and `deny.toml` | `NOTICE:11, :26` | `deny.toml` exists; `LICENSE` — **not checked** (see Coverage) |
| `docs/api/MCP.md` per-tool annotations, absorbed-name table (8 rows) | `docs/api/MCP.md:211-228` | `vantadb-mcp/src/handlers/tools.rs:1315` (`absorbed_canonical`) — 8 names match ✓ |

---

## 3. Coverage

### Read deeply, line-by-line, against source
`docs/user/operations/CONFIGURATION.md` (all 9 `##` sections + the 2 env-var tables + the Rust snippet) · `docs/user/operations/BENCHMARKS.md` (head 120 + all 26 section headers + the 4 benchmark tables) · `docs/api/MCP.md` (head 60, tool-family table, profile table, absorbed-names table, registry table) · `docs/api/EMBEDDINGS.md` (provider table + the embed-local command block) · `docs/api/HTTP_API.md` (all 40 endpoint headers + the graph table) · `docs/api/openapi.yaml` (all 38 paths) · `docs/user/QUICKSTART.md` (install + wheel + TestPyPI blocks) · `docs/api/VERSIONING.md` (full) · `CONTRIBUTING.md` (full, 315 lines) · `SECURITY.md` (full) · `SUPPORT.md` (full) · `NOTICE` (full) · `CITATION.cff` (full) · root `CHANGELOG.md` (full) · `docs/CHANGELOG.md` (head 60) · `llms.txt` (head 50 + all 481 links resolved) · `docs/dev/architecture/adr/ADR-0011-sparse-vectors-native.md` (head 45) · `docs/dev/avance/COBERTURA.md` (full).

### Automated sweep over the whole tree
- **1330** `.md` files under `docs/` walked; every backtick-quoted path-like token extracted (fence-aware) and resolved against both the doc's directory and the repo root → **980 unique missing tokens, 2929 hits**. After excluding `docs/dev/plans/archive/`, `docs/dev/research/archive/`, `docs/dev/avance/historial/` (frozen/archived trees by convention) and non-path tokens, the residual high-signal set is reported in D-21 and D-16; the rest are future-planned paths in backlog/plan documents, which are out of scope for a "does this file exist" claim.
- Every `--flag` token in `docs/` extracted and diffed against the canonical clap set derived from `src/cli.rs` → the only genuinely non-existent CLI flag surfaced is `--allow-dummy-embeddings` (D-16); all 29 `vanta-cli <subcommand>` usages in `docs/api` + `docs/user` exist.
- Every `VANTADB_*` / `VANTA_*` token in `docs/user/operations/CONFIGURATION.md` cross-checked against every `env::var("…")` / `env::var_os("…")` call in `src/`, `vantadb-server/`, `vantadb-mcp/`, `vanta-memory/`, `vanta-proxy/` → D-07, D-15, D-19.
- Every `N tools` / `N crates` / `N documents` counter in `docs/` extracted and checked → D-08, D-09; all others verified correct (see table above).
- Every `docs/dev/tasks/*.md` frontmatter `status` read (634 files) → D-23.
- All 59 ADR filenames enumerated; ADR-0001, 0004, 0011, 0012, 0020 read in detail → D-20, D-21.

### Sampled (not exhaustively verified)
- `docs/dev/Backlog.md` (1116 lines) — read the MCP-tool-count rows and the `VantaConfig` hits; ~90 task-ID rows and their status cells were **not** individually cross-checked against the code.
- `docs/dev/Backlog-negocio.md` — sampled the DX-10 row only.
- `docs/dev/tasks/` — **status-field distribution** computed over all 634; **no** task file's "completed" claim was individually re-verified against a diff. 235 files claim `status: completed`; none were audited for false completion.
- `docs/dev/avance/activo/*.md` (10 files, ~500 KB) — the `VantaConfig` hits were traced; the ~1200 other symbol/flag references were not verified.
- `docs/api/EMBEDDED_SDK.md` (56 KB) — the `Vanta*`/`Config::` and version-history sections read; the ~200 method signatures not all diffed against `src/sdk/`.
- `docs/api/PYTHON_SDK.md` (54 KB) and `docs/api/TS_SDK.md` (32 KB) — only the `VantaConfig` hits checked. The python *method* names in these are covered by `check-doc-examples.mjs`, which resolved 13 distinct attributes successfully but reported **78/78 attribute chains unresolved** (all classified `fragment` — `python/undefined` 151, `python/unparseable` 85, `python/skip-marker` 21) — so **the Python/TS SDK method surface was not effectively verified** by tooling and was not verified manually either.
- `docs/user/glosario/` (37 files), `docs/user/blog/`, `docs/user/tutorials/` — sampled; only D-17 surfaced.

### Skipped (with reason)
- `docs/dev/research/**` (archived competitive/ecosystem research) — historical third-party research notes; factual claims are about *other* projects (LanceDB, Chroma, Mem0) and are point-in-time by construction. Not verifiable against this repo.
- `docs/dev/plans/archive/**`, `docs/dev/avance/historial/**` (frozen snapshots + campaign logs) — by repo convention these are immutable history and are explicitly excluded from the "no information loss" guarantee (`docs/dev/avance/COBERTURA.md` §Convención). They contain the bulk of the 2929 missing-path hits; auditing them as if they were current documentation would produce false positives.
- `docs/dev/reviews/archive/**` — historical audit reports, snapshots of past states.
- `docs/specs/CONSTRAINTS.md`, `docs/specs/SPEC.md`, `docs/reports/*` — read for inventory only; not audited against code.
- `docs/user/operations/*` other than CONFIGURATION.md and BENCHMARKS.md (20 files, ~200 KB) — **not audited**. `PERFORMANCE_GUIDE.md`, `PERFORMANCE_TUNING.md`, `EXPERIMENTAL_FEATURES.md`, `GC_TTL.md`, `DURABILITY_GUARANTEES.md`, `SQLITE_MIGRATION_GUIDE.md`, `UPGRADE.md`, `DEPLOYMENT_GUIDE.md`, `DISASTER_RECOVERY_RUNBOOK.md` etc. are unverified and are the most likely remaining home of stale defaults/versions.
- `docs/api/*` other than MCP / HTTP_API / EMBEDDINGS / VERSIONING / EMBEDDED_SDK / openapi.yaml (23 files) — **not audited**, including `EMBEDDED_SDK.md` beyond the sections noted, `PROXY.md`, `VANTA_MEMORY.md`, `IQL.md`, `ERROR_HANDLING.md`, `BINDINGS_NAMESPACES.md`, `scores.md`, `WASM_*.md`, `TS_SDK.md`, `NODE_SDK.md`, `PYTHON_SDK.md`, `CERTIFIED_DELETE.md`, `WRITE_RECEIPTS.md`, `MEMORY_INTERCHANGE_FORMAT.md`.
- `NOTICE` → `LICENSE` file existence — not checked (the doc does not link it; `LICENSE` presence assumed).
- **No benchmark was re-run.** Every performance number in `BENCHMARKS.md` was checked for *internal consistency*, *presence of a reproduce command and environment block*, and *contradiction with the canonical source* — **not** for empirical correctness. `benchmarks/vanta_benchmark_report.json` exists on disk but is gitignored (`.gitignore:251`) and was deliberately **not** used as a source, per the audit brief. Numbers that are internally consistent and carry a reproduce command were marked NO VERIFICADO rather than reported as findings.

### NO VERIFICADO (claims that could not be checked without running code)
| Claim | Doc | Why unverified |
|---|---|---|
| Recall@10 = 0.9560, Scaling Recall 0.9980 / 1.0000 / 0.9980 | `BENCHMARKS.md:28-31` | Requires a 2 h+ `--release` run on a machine matching the doc's hardware profile. The *dataset mis-attribution* (D-06) was found statically; the values themselves are unverified. |
| Memory efficiency ~1172 bytes/vector; Graph Consistency Orphans 0 / Avg L0 64.0 | `BENCHMARKS.md:32-33` | Computed at runtime by `tests/certification/stress_protocol.rs` (`estimate_memory_bytes() / n`, `stats().avg_connections_l0`). The code asserts `orphan_count <= 1` — the published "0" is a plausible observed value, not a contract. |
| SDK p50/p95/p99 + throughputs (13.510 ms / 422 qps / 179 qps / 1 667 ops/s) | `BENCHMARKS.md:56-61` | All internally consistent (`1000/p50` reproduces each derived qps to <0.3%) and carry a full Regla-11 environment block + reproduce command. Empirically unverified. |
| Embedding-model table (9 models, sizes, licences) | `BENCHMARKS.md:198-221` | Model sizes/recall require the ONNX weights to be present; `embeddings/models/` presence not verified. |
| Section 4-13 benchmark tables (prefetch, LLM/HNSW optimisation, batch search, competitive vs LanceDB/Chroma, Node-vs-TS, consumption guard, memory budget, multi-consumer ingestion) | `BENCHMARKS.md:118-1400` | Each has a reproduce command; none was re-run. Cross-section comparisons were checked for internal contradictions only. |

---

## 4. Summary

| Severity | Count | IDs |
|---|---|---|
| **P0** | 1 | D-01 |
| **P1** | 13 | D-02 … D-15 |
| **P2** | 5 | D-16 … D-20 |
| **P3** | 6 | D-21 … D-26 |
| **Total** | **26** | |

**Theme.** Every finding is in a class the repo's own gates do not cover. The gates prove the docs are *structurally* sound (0 broken links in 2669, valid frontmatter, python attribute names resolve). They say nothing about whether a **named Rust type still exists** (D-02), whether a **default value matches the code** (D-10, D-11), whether a **cited CI workflow file exists** (D-05), whether a **version pin is current** (D-03, D-04, D-12, D-13, D-14), whether **two docs quoting the same source line agree** (D-08, D-09, D-20), or whether a **security guarantee in the security policy is actually implemented** (D-01).

**Two clusters deserve a process fix, not a one-line edit:**
1. **Version drift (D-03, D-04, D-12, D-13, D-14, D-26)** — six independent places carry a stale version. A single CI check comparing the versions quoted in `SECURITY.md`, `CITATION.cff`, `docs/user/QUICKSTART.md`, `docs/api/MCP.md` against `Cargo.toml`'s `[workspace.package].version` would have caught all six, and would prevent the next four releases from rotting the same way.
2. **Hand-derived counters (D-08, D-09, D-20, D-23, D-24)** — tool counts, ID coverage, ADR descriptions and `llms.txt` are all derived by hand from code that has a machine-readable source of truth (`vantadb-mcp/tests/mcp_tests.rs::test_mcp_tool_profiles` for the counts). `docs/api/MCP.md` is correct because it was regenerated; the Backlog rows are wrong because they were transcribed. Point the doc checkers at the same source of truth rather than at the prose.

**No file in the repository was created, modified, or deleted by this audit.**

*Post-audit `git status --porcelain` shows two entries — `M vanta-mcp-local.ps1` and `?? vantadb-node/vantadb-node-0.8.0.tgz`. Both are **pre-existing working-tree state** (a locally-modified helper script and an untracked npm tarball); neither file was opened for write by this audit, which only ever read from the repo and wrote to `%TEMP%\opencode\audit-v090\`. The audit ran no `git` write command, no `--write` script, and no formatter.*
