---
title: "Workflows — Inventory (27 active)"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, inventory]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/TRIGGERS.md", "docs/workflow/PUBLISH.md", "docs/workflow/RUNBOOK.md", "docs/workflow/FAQ.md"]
---

# Workflows — Inventory (27 active)

> Pre-rename snapshot (FIND-142 pending): filenames keep the numeric suffix
> (`ci-rust-10.yml`, `release-wheels-60.yml`, …). After renames, this index
> must be updated. Count was 28 in FIND-128; now 27 — `rustdoc-70.yml` was
> merged into `ci-rustdoc.yml` (FIND-137, commit `4b0686b0`).

Per-workflow detail pages live next to this index (`ci-gate.md`,
`release-wheels-60.md`, …). This file is the 1-line map; see
[TRIGGERS.md](./TRIGGERS.md) for the full trigger matrix and
[PUBLISH.md](./PUBLISH.md) for the release chain.

## CI core (5)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `ci-rust-10.yml` | Main Rust gate: fmt, clippy, nextest, ADR gate | push/PR (paths core), dispatch |
| `ci-rustdoc.yml` | Builds rustdoc + API reference (survivor) | push/PR (paths src/tests/Cargo), dispatch |
| `ci-web-11.yml` | Web workspace CI | push/PR (paths `web/**`), dispatch |
| `ci-examples-12.yml` | Examples + Python SDK smoke | push/PR (paths examples/src/python), dispatch |
| `ci-gate.yml` | Reusable fail-closed gate for heavy jobs | `workflow_call` only |

## Quality gates (2)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `gate-docs-21.yml` | Docs coverage + router drift gate | push/PR (paths docs/router/scripts), dispatch |
| `sec-codeql-30.yml` | CodeQL static analysis | push/PR `main`, weekly schedule, dispatch |

## Providers / compat (2)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `providers-ci.yml` | `providers/**` clippy + tests | push/PR (paths providers), dispatch |
| `adapters-compat.yml` | Adapter compat matrix (informational) | weekly schedule, dispatch |

## Heavy / fuzz / perf (6)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `chaos-45.yml` | Chaos integrity (failpoints) | push/PR (paths core), dispatch |
| `fuzz-40.yml` | Cargo-fuzz gate + nightly corpus | schedule Mon 06:00, PR (paths src/fuzz), dispatch |
| `perf-bench-40.yml` | Python perf bench vs baseline | push (paths src/python/bench), dispatch |
| `heavy-certification-50.yml` | Heavy certification suite (2h) | schedule Sun 03:00, dispatch |
| `heavy-bench-nightly-51.yml` | Nightly criterion benches | schedule daily 02:00, PR (paths benches), dispatch |
| `bench-canonical-p99-informational.yml` | Canonical P99 bench (never blocks) | PR (paths index/storage), dispatch |

## Informational / bots (4)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `arch-metrics-informational.yml` | Architecture metrics (never blocks) | PR (paths src/Cargo), dispatch |
| `ocr-delegate.yml` | OCR delegation review (never blocks) | PR (all), dispatch |
| `ocr-nightly.yml` | Nightly OCR review | schedule daily 04:00, dispatch |
| `opencode.yml` | Opencode bot on comments | `issue_comment`, `review_comment` |

## Desktop (1)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `desktop.yml` | Tauri desktop builds | push/PR (paths desktop/server), dispatch |

## Release (7)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `release.yml` | release-plz: version, changelog, tags, crates.io | push `main` only |
| `release-wheels-60.yml` | Python wheels → PyPI/TestPyPI | tags `v*.*.*`, PR (paths src/python), dispatch |
| `release-npm-61.yml` | WASM + TS SDK → npm | tags `v*.*.*`, PR (paths wasm/ts), dispatch |
| `release-npm-node.yml` | Node binding → npm | tags `node-v*.*.*`, PR (paths node), dispatch |
| `release-adapters-62.yml` | 9 adapters → PyPI/TestPyPI | tags `adapters-v*.*.*`, dispatch |
| `release-binaries-63.yml` | Binaries + docker assets → GitHub Release | `release` published, dispatch |
| `release-sbom-64.yml` | CycloneDX SBOM artifacts | tags `v*`, dispatch |

## See also

- [TRIGGERS.md](./TRIGGERS.md) — full trigger matrix
- [PUBLISH.md](./PUBLISH.md) — publish flow per registry
- [RUNBOOK.md](./RUNBOOK.md) — re-run and approvals
- [FAQ.md](./FAQ.md) — duplicates, cancel-in-progress, skipped vs required
