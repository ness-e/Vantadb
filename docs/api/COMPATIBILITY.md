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
| Rust API snapshot | `cargo test -p vantadb --test public_api` | `vantadb` public API, default features, simplified (blanket impls omitted) | CI job `public-api-snapshot`; golden file [`tests/api/public-api.txt`](../../tests/api/public-api.txt), regenerated only via `VANTADB_PUBLIC_API_UPDATE=1` |
| Tool presence check | `cargo semver-checks --version` | The semver job fails if the tool is missing (no silent skip) | Same job, step before the lint |
| Docs coverage | `pwsh scripts/validate-docs-coverage.ps1` | SDK methods, config fields, error variants, CLI commands, Python methods, MCP tools | Fast CI + local pre-commit |
| HTTP ↔ OpenAPI parity | `cargo test --test openapi_yaml_parity` | HTTP API surface (#6) | `cargo nextest` (ci-rust) |
| Cross-binding wire | `cargo test --test sdk_serialization` | `u128`-as-string, score semantics across bindings | `cargo nextest` (ci-rust) |
| Python boundary | `cargo test --test python_sdk_boundary` | Python SDK surface (#2) | `cargo nextest` (ci-rust) |

> **Reading the semver gate on `develop`:** `check-release` compares the tree
> against the latest crates.io release (0.7.0). While accepted breaks for the
> next MINOR sit unreleased on `develop`, it exits `100` with the delta list
> below — that is the decision signal at the develop→main PR. On the release
> commit (version bumped, e.g. 0.8.0) the same command exits `0`, because 0.x
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

## Pre-release deltas (vs published 0.7.0)

Accepted breaking changes merged to `develop` **after** the `v0.7.0` release
cut (2026-09-25), to ship in the next MINOR (`0.8.0`, under 0.x rules). Detected
by `cargo semver-checks check-release` (exit 100; 9 deny-level lint families).
Four are **CLI-only (#9)**: the three `Commands`-shape lints plus the
`cli_handlers::cmd_*` arity wave. Three are **core-only (#1)**: the `FormatKind`
discriminant shift, the `Embedded::import_*` arity, and the Cargo-feature
decouple (`server` no longer enables `cli` — a feature-graph change, not a Rust
item signature). The remaining two mix surfaces: `constructible_struct_adds_field`
(CLI `Cli` + core wire structs/`Config`) and `enum_variant_added` (`Commands` +
`FormatKind`):

| lint (`cargo-semver-checks`) | What changed | Commit |
|------------------------------|--------------|--------|
| `constructible_struct_adds_field` | `Cli.json` (`src/cli.rs:38`); `Config` +5 fields (`config.rs:783-821`): `insert_batch`, `memory_default_ttl_ms`, `ttl_sweep_interval_ms`, `quarantine_review_default_days`, `confidence_threshold`; v2 fields on the wire structs (`MemoryRecord` ×10, `MemoryInput` ×5, `MemoryExportLine` ×10, `MemorySearchRequest` ×8, `MemoryListOptions` ×4, `Query`/`SelectStatement` `as_of_ms`, `ImportReport`/`BulkImportReport` `quarantined`) | `f6c395ef` · WIRE-08 · SCH-02..07 |
| `enum_no_repr_variant_discriminant_changed` | `FormatKind::Schema` discriminant `3 -> 4` (`src/migration.rs:30`) — `Records` inserted before it | SCH-02 (`7af34366`) |
| `enum_struct_variant_changed_kind` | `Commands::Stats` changed variant kind (`src/cli.rs:192`) | `f6c395ef` |
| `enum_struct_variant_field_added` | `query_flag` / `limit` added to `Search` / `SearchMulti` / `SearchAll` / `SimilarToKey`; `attest` / `out` added to `Delete` | `f6c395ef` · VER-02 (`ccc51d09`) |
| `enum_struct_variant_field_missing` | `json` / `top_k` removed or renamed on several `Commands` variants | `f6c395ef` |
| `enum_variant_added` | `Commands::McpCall` (`src/cli.rs:380`), `Commands::Verify` (`:201`) and `Commands::Certificate` (`:261`) added; `FormatKind::Records` (`src/migration.rs:28`) | `8e55e853` · SCH-02 · VER-01/02 |
| `feature_no_longer_enables_feature` | **`feature server` no longer enables `cli`** (WIRE-07 feature decouple): the HTTP binary (`vantadb-server`/`vantadb-mcp`, now `default-features = false`) stops dragging `clap`/`clap_complete`/`indicatif`/`console`/`anyhow`. Downstreams that relied on the implication restore it with `features = ["server", "cli"]`; `vantadb-server` keeps its own opt-in `cli = ["vantadb/cli"]` | WIRE-07 |
| `function_parameter_count_changed` | 26 `pub` `cli_handlers::cmd_*` gained one parameter (path/POSIX + global `--json` plumbing) | `f6c395ef` |
| `method_parameter_count_changed` | `Embedded::import_records` / `import_file` gain `quarantine: bool` (`src/sdk/serialization/impl_export.rs`) | SCH-05 (`83d65518`) |

- **Disposition:** accepted under the 0.x MINOR policy (introduced by `feat!:` /
  `fix:` commits on `develop`); **not** excluded and no lint levels weakened —
  the gate keeps its teeth. The consumer-facing write-up ships with `0.8.0` in
  [`UPGRADE.md`](../user/operations/UPGRADE.md).
- **Reproduce:** `cargo semver-checks check-release --baseline-rev v0.7.0`
  (rustdoc builds; `cargo-semver-checks` 0.49.0). Raw evidence: 196 checks —
  187 pass / 9 fail / 57 skip, exit 100 (measured 2026-10-01 at `ae97538e`;
  cold `target/semver-checks` cache: 3303s total — 1897s current + 1379s
  baseline rustdoc builds, `CARGO_BUILD_JOBS=2`; warm runs ≈150s, 2026-09-27).
  The 9-family / 187-9-57 counts match the SCH-07 run of 2026-09-29; the two
  families added over the WIRE-07 run (2026-09-28, 7 families) are the
  `FormatKind` discriminant and the `Embedded::import_*` arity. Isolate with
  `CARGO_TARGET_DIR` when concurrent sessions share the `target/semver-checks`
  cache.

## 1.0 Readiness — exit criteria

`1.0.0` is declared only when **all** of the following are verifiable:

- [ ] `cargo semver-checks check-release` exits `0` at the `1.0.0` release commit (the 0.9→1.0 bump licenses the major in one step).
- [ ] `cargo test -p vantadb --test public_api` is green with `tests/api/public-api.txt` updated deliberately for 1.0 (the pre-release deltas section above is empty).
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
