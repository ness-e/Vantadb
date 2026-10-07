---
title: "Compatibility Matrix & 1.0 Readiness"
kind: reference
status: active
description: "What each public API surface promises, how that promise is enforced"
tags: [vantadb, api, compatibility, semver]
---

# Compatibility Matrix & 1.0 Readiness

What each public API surface promises, how that promise is enforced
mechanically today, and the verifiable exit criteria for declaring `1.0.0`.
The stability policy itself is [`VERSIONING.md`](VERSIONING.md); concrete
deprecations are registered in [`DEPRECATIONS.md`](DEPRECATIONS.md).

## Breaking-change rails (added 2026-09, HARD-01)

| Rail | Command | Scope | Placement |
|------|---------|-------|-----------|
| Rust API lint | `cargo semver-checks check-release` | `vantadb` — the only crates.io-published crate. `vantadb-wasm` is npm-distributed and has no crates.io baseline, so it is skipped by the tool | [`.github/workflows/ci-rust.yml`](../../.github/workflows/ci-rust.yml) job `semver-checks` (main pushes + PRs targeting main) + release-plz [`semver_check = true`](../../release-plz.toml) |
| Rust API snapshot | `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` | `vantadb` public API, default features, simplified (blanket impls omitted) | CI job `public-api-snapshot`; golden file [`tests/api/public-api.txt`](../../tests/api/public-api.txt), regenerated only via `VANTADB_PUBLIC_API_UPDATE=1` |
| Tool presence check | `cargo semver-checks --version` | The semver job fails if the tool is missing (no silent skip) | Same job, step before the lint |
| Docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | SDK methods, config fields, error variants, CLI commands, Python methods, MCP tools | Fast CI + local pre-commit |
| HTTP ↔ OpenAPI parity | `cargo test --test openapi_yaml_parity` | HTTP API surface (#6) | `cargo nextest` (ci-rust) |
| Cross-binding wire | `cargo test --test sdk_serialization` | `u128`-as-string, score semantics across bindings | `cargo nextest` (ci-rust) |
| Python boundary | `cargo test --test python_sdk_boundary` | Python SDK surface (#2) | `cargo nextest` (ci-rust) |

> **Reading the semver gate on `develop`:** `check-release` compares the tree
> against the latest crates.io release (0.8.0). While accepted breaks for the
> next MINOR sit unreleased on `develop`, it exits `100` with the delta list
> below — that is the decision signal at the develop→main PR. On the release
> commit (version bumped, e.g. 0.9.0) the same command exits `0`, because 0.x
> MINOR bumps may contain breaking changes ([`VERSIONING.md`
> § Pre-1.0](VERSIONING.md#pre-10-stability-contract)).

## Surface × enforcement matrix

| # | Surface | Reference doc | Primary mechanical enforcement (today) |
|---|---------|---------------|------------------------------------------|
| 1 | Rust core SDK | [`EMBEDDED_SDK.md`](EMBEDDED_SDK.md) | `cargo semver-checks` + public API snapshot + validate-docs-coverage |
| 2 | Python SDK | [`PYTHON_SDK.md`](PYTHON_SDK.md) | `python_sdk_boundary` + `sdk_serialization` + validate-docs-coverage |
| 3 | TypeScript SDK | [`TS_SDK.md`](TS_SDK.md) | `npm test` (vitest) in the release-npm workflow |
| 4 | Node.js SDK | [`NODE_SDK.md`](NODE_SDK.md) | `npm test` in the release-npm-node workflow |
| 5 | WebAssembly SDK | [`WASM_API.md`](WASM_API.md) · [`WASM_PERSISTENCE.md`](WASM_PERSISTENCE.md) · [`WASM_STANDALONE.md`](WASM_STANDALONE.md) | wasm-pack browser tests (ci-rust job `wasm-test`, BEST-EFFORT) |
| 6 | HTTP API + OpenAPI | [`HTTP_API.md`](HTTP_API.md) + [`openapi.yaml`](openapi.yaml) | `openapi_yaml_parity` |
| 7 | MCP server | [`MCP.md`](MCP.md) | validate-docs-coverage (tool ↔ doc parity) |
| 8 | IQL | [`IQL.md`](IQL.md) | `openapi_yaml_parity` (documented-syntax subset) + `IQL_VERSION` pin |
| 9 | CLI | [`CONFIGURATION.md` §4](../user/operations/CONFIGURATION.md) | validate-docs-coverage (commands ↔ doc parity) + built-in `--help` |
| 10 | LLM proxy | [`PROXY.md`](PROXY.md) | review-only — see § Gaps |
| 11 | vanta-memory (Rust crate) | [`VANTA_MEMORY.md`](VANTA_MEMORY.md) | review-only — see § Gaps |

## Pre-release deltas (vs published 0.8.0)

Accepted breaking changes merged to `develop` **after** the `v0.8.0` release
cut (2026-10-03), to ship in the next MINOR (`0.9.0`, under 0.x rules). Detected
by `cargo semver-checks check-release` (exit 100; 3 deny-level lint families).
All three are **core-only (#1)** — the IQL pagination/aggregation wave
(`LIMIT`/`OFFSET`, `COUNT`/`SUM`/`GROUP BY`) and the portable-export manifest
fields:

| lint (`cargo-semver-checks`) | What changed | Commit |
|------------------------------|--------------|--------|
| `constructible_struct_adds_field` | `Query.limit` / `Query.offset` (`src/query.rs:122,126`); `ExportReport.sha256` / `ExportReport.manifest_path` (`src/sdk/types/record.rs:482,485`) | WIRE-12 (`22585e1b`) · MEMG-15 (`55b9a532`) |
| `enum_no_repr_variant_discriminant_changed` | `LogicalOperator::Dedup` `8 -> 10`, `Join` `9 -> 11`, `SubqueryFilter` `10 -> 12` (`src/query.rs:510,515,526`) — `Offset`/`Aggregate` inserted before them | WIRE-12 (`22585e1b`) · WIRE-13 (`753b9f78`) |
| `enum_variant_added` | `LogicalOperator::Offset` (`src/query.rs:490`), `LogicalOperator::Aggregate` (`:500`) — exhaustive enum | WIRE-12 (`22585e1b`) · WIRE-13 (`753b9f78`) |

- **Disposition:** accepted under the 0.x MINOR policy (introduced by `feat:` /
  `fix:` commits on `develop`); **not** excluded and no lint levels weakened —
  the gate keeps its teeth. The consumer-facing write-up ships with `0.9.0` in
  [`UPGRADE.md`](../user/operations/UPGRADE.md).
- **Reproduce:** `cargo semver-checks check-release` (baseline: latest crates.io
  release `0.8.0`; rustdoc builds; warm runs ≈150s). Raw evidence (CI, PR #242,
  2026-10-06): exit 100 — `3 major and 0 minor checks failed`; current rustdoc
  build `Finished [1398s]`. Exit `100` on `develop` is the decision signal; the
  same command exits `0` on the release commit once the version is bumped to
  `0.9.0` (0.x MINOR may contain breaking changes — [`VERSIONING.md`
  § Pre-1.0](VERSIONING.md#pre-10-stability-contract)). Isolate with
  `CARGO_TARGET_DIR` when concurrent sessions share the `target/semver-checks`
  cache.

## 1.0 Readiness — exit criteria

`1.0.0` is declared only when **all** of the following are verifiable:

- [ ] `cargo semver-checks check-release` exits `0` at the `1.0.0` release commit (the 0.9→1.0 bump licenses the major in one step).
- [ ] `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` is green with `tests/api/public-api.txt` updated deliberately for 1.0 (the pre-release deltas section above is empty).
- [ ] [`DEPRECATIONS.md`](DEPRECATIONS.md) holds no active entry younger than one MINOR (each is removed or past its ≥1-MINOR window).
- [ ] Every surface in the matrix above has ≥1 mechanical enforcement — zero `review-only` cells in the last column.
- [ ] Version coherence: `[workspace.package] version` == published artifacts (crates.io / PyPI / npm) == `docs/api/openapi.yaml` header (rule R-2).
- [ ] This file's pre-release deltas section is empty (no accepted break sits outside a release commit).

## Gaps (tracked, not silent)

Surfaces whose repo-level mechanical enforcement is incomplete today. Closing
these is 1.0 work:

- **#10 LLM proxy, #11 vanta-memory** — no mechanical gate yet (crates are not
  crates.io-published; release-plz does not cover them).
- **#5 WebAssembly** — browser tests are BEST-EFFORT (non-blocking); a hard gate
  would need wasm-pack promoted out of the experimental tier.
- **#8 IQL** — syntax parity is covered only for the subset `openapi.yaml`
  documents; `IQL.md` itself is not mechanically diffed.

## See also

- [`VERSIONING.md`](VERSIONING.md) — policy, 11 surfaces, release mechanics.
- [`DEPRECATIONS.md`](DEPRECATIONS.md) — deprecation registry.
- [`UPGRADE.md`](../user/operations/UPGRADE.md) — per-version migration guide.
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — release workflow (release-plz + git-cliff).
