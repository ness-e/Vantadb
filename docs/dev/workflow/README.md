---
title: Workflows — Inventory (40 active)
kind: index
status: active
description: "Per-workflow detail pages live next to this index (ci-gate.md,"
tags: [vantadb, ci, workflows, inventory]
---

# Workflows — Inventory (40 active)

> **Count verified mechanically on 2026-10-04: 40 files** (`Get-ChildItem .github/workflows -Filter *.yml`; +1 `ci-bindings-parity.yml`, DIST-17; +1 `release-providers.yml`, PROV-12).
> The old "28" predates the recent waves (nightly/OCR/gates/demos) and is superseded; registry: `docs/dev/references/verified-numbers.md`.
> Pre-rename snapshot (FIND-142 pending): filenames keep the numeric suffix
> (`ci-rust.yml`, `release-wheels.yml`, …). After renames, this index
> must be updated. History: 28 in FIND-128; `rustdoc-70.yml` merged into
> `ci-rustdoc.yml` (FIND-137, commit `4b0686b0`); +1 `nightly.yml` (HARD-02, 2026-09-27).

Per-workflow detail pages live next to this index (`ci-gate.md`,
`release-wheels.md`, …). This file is the 1-line map; see
[TRIGGERS.md](./TRIGGERS.md) for the full trigger matrix and
[PUBLISH.md](./PUBLISH.md) for the release chain.

## CI core (5)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `ci-rust.yml` | Main Rust gate: fmt, clippy, nextest, ADR gate | push/PR (paths core), dispatch |
| `ci-rustdoc.yml` | Builds rustdoc + API reference (survivor) | push/PR (paths src/tests/Cargo), dispatch |
| `ci-web.yml` | Web workspace CI | push/PR (paths `web/**`), dispatch |
| `ci-examples.yml` | Examples + Python SDK smoke | push/PR (paths examples/src/python), dispatch |
| `ci-gate.yml` | Reusable fail-closed gate for heavy jobs | `workflow_call` only |

## Quality gates (3)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `gate-docs.yml` | Docs coverage + router drift gate | push/PR (paths docs/router/scripts), dispatch |
| `sec-codeql.yml` | CodeQL static analysis | push/PR `main`, weekly schedule, dispatch |
| `ci-bindings-parity.yml` | Cross-language conformance Py/Node/WASM (canonical hash/diff) | push/PR (paths bindings), dispatch |

## Providers / compat (2)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `providers-ci.yml` | `providers/**` clippy + tests | push/PR (paths providers), dispatch |
| `adapters-compat.yml` | Adapter compat matrix (informational) | weekly schedule, dispatch |

## Heavy / fuzz / perf (7)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `chaos.yml` | Chaos integrity (failpoints) | push/PR (paths core), dispatch |
| `fuzz.yml` | Cargo-fuzz gate + nightly corpus | schedule Mon 06:00, PR (paths src/fuzz), dispatch |
| `perf-bench.yml` | Python perf bench vs baseline | push (paths src/python/bench), dispatch |
| `heavy-certification.yml` | Heavy certification suite (2h) | schedule Sun 03:00, dispatch |
| `nightly.yml` | Nightly heavy-cert subset + coverage budget + release dry-run | schedule daily 05:00, dispatch |
| `heavy-bench-nightly.yml` | Nightly criterion benches | schedule daily 02:00, PR (paths benches), dispatch |
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

## Release (9)

| Workflow | Purpose (1 line) | Triggers (short) |
|----------|------------------|------------------|
| `release.yml` | release-plz: version, changelog, tags, crates.io | push `main` only |
| `release-wheels.yml` | Python wheels → PyPI/TestPyPI | tags `v*.*.*`, PR (paths src/python), dispatch |
| `release-npm-61.yml` | WASM + TS SDK → npm | tags `v*.*.*`, PR (paths wasm/ts), dispatch |
| `release-npm-node.yml` | Node binding → npm | tags `node-v*.*.*`, PR (paths node), dispatch |
| `release-adapters.yml` | 9 adapters → PyPI/TestPyPI | tags `adapters-v*.*.*`, dispatch |
| `release-providers.yml` | 3 Rust providers → PyPI/TestPyPI (maturin 3×4 matrix) | tags `providers-v*.*.*`, PR (paths providers), dispatch |
| `release-binaries.yml` | Binaries → GitHub Release | `release` published, dispatch (`release_tag` = backfill) |
| `release-sbom.yml` | CycloneDX SBOM artifacts | tags `v*`, dispatch |
| `release-verify.yml` | Post-release artifact verification (assets + registries + smoke) | schedule Mon 08:00, dispatch |

## See also

- [TRIGGERS.md](./TRIGGERS.md) — full trigger matrix
- [PUBLISH.md](./PUBLISH.md) — publish flow per registry
- [RUNBOOK.md](./RUNBOOK.md) — re-run and approvals
- [FAQ.md](./FAQ.md) — duplicates, cancel-in-progress, skipped vs required
