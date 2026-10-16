---
title: Workflows — Inventory (19 active)
kind: index
status: active
description: "Per-workflow detail pages live next to this index (ci-gate.md,"
tags: [vantadb, ci, workflows, inventory]
---

# Workflows — Inventory (19 active)

> **Count verified mechanically on 2026-10-16: 19 files** (`Get-ChildItem .github/workflows -Filter *.yml`).
> Post CI-08 (SRV-10): 40 → 32 (Steps 2-3) → 18 (Step 4) → 19 (+`publish-mcp-registry.yml` FIND-344). Registry: `docs/dev/references/verified-numbers.md`.
> Pre-rename snapshot (FIND-142 pending): filenames keep the numeric suffix
> (`ci-rust.yml`, `release-wheels.yml`, …). After renames, this index
> must be updated. History: 28 in FIND-128; `rustdoc-70.yml` merged into
> `ci-rustdoc.yml` (FIND-137, commit `4b0686b0`); +1 `nightly.yml` (HARD-02, 2026-09-27).

Per-workflow detail pages live next to this index (`ci-gate.md`,
`release-wheels.md`, …). This file is the 1-line map; see
[TRIGGERS.md](./TRIGGERS.md) for the full trigger matrix and
[PUBLISH.md](./PUBLISH.md) for the release chain.

## CI (7)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `ci-rust.yml` | Main Rust gate: fmt, clippy, nextest, ADR gate | push/PR (paths core), dispatch |
| `ci-gate.yml` | Reusable fail-closed gate for heavy jobs | `workflow_call` only |
| `docs-gates.yml` | Docs gates ×6: lint, links, secrets, API surface, examples, rustdoc (17 jobs) | push/PR (paths docs/src), weekly Mon 07:23, dispatch |
| `ci-fleet.yml` | Fleet ×4: examples, bindings parity, adapters, providers (5 jobs) | push/PR, weekly Sun 03:00, dispatch |
| `demo-acceptance.yml` | Acceptance demos ×5 (F5/F6) | push/PR (union paths), dispatch |
| `nightly.yml` | Nightly heavy-cert subset + 5 informational jobs | schedule daily 05:00, PR (paths info-only), dispatch |
| `perf-bench.yml` | Python perf bench vs baseline | push (paths src/python/bench), dispatch |

## Certificación (1)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `certification.yml` | Heavy ×4: certification, benches, chaos, fuzz (21 jobs, ci-gate dedup) | push/PR (paths core/fuzz), 3 schedules, dispatch |

## Platform ops (1)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `platform-ops.yml` | CodeQL + opencode bot + OCR nightly (5 jobs; desktop builds removidos por freeze 2026-10-16) | push/PR (paths src/server), 2 schedules, comments, dispatch |

## Release (9)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `release.yml` | release-plz: version, changelog, tags, crates.io | push `main` only |
| `release-wheels.yml` | Python wheels → PyPI/TestPyPI | tags `v*.*.*`, PR (paths src/python), dispatch |
| `release-npm-61.yml` | WASM + TS SDK → npm | tags `v*.*.*`, PR (paths wasm/ts), dispatch |
| `release-npm-node.yml` | Node binding → npm | tags `node-v*.*.*`, PR (paths node), dispatch |
| `release-adapters.yml` | 7 adapters → PyPI/TestPyPI | tags `adapters-v*.*.*`, dispatch |
| `release-providers.yml` | 3 Rust providers → PyPI/TestPyPI (maturin 3×4 matrix) | tags `providers-v*.*.*`, PR (paths providers), dispatch |
| `release-binaries.yml` | Binaries → GitHub Release | `release` published, dispatch (`release_tag` = backfill) |
| `release-sbom.yml` | CycloneDX SBOM artifacts | tags `v*`, dispatch |
| `release-verify.yml` | Post-release artifact verification (assets + registries + smoke) | schedule Mon 08:00, dispatch |

## See also

- [TRIGGERS.md](./TRIGGERS.md) — full trigger matrix
- [PUBLISH.md](./PUBLISH.md) — publish flow per registry
- [RUNBOOK.md](./RUNBOOK.md) — re-run and approvals
- [FAQ.md](./FAQ.md) — duplicates, cancel-in-progress, skipped vs required
