---
title: "Versioning & Stability Policy"
kind: reference
status: active
description: VantaDB follows Semantic Versioning with the standard
tags: [vantadb, api, semver]
---

# Versioning & Stability Policy

VantaDB follows [Semantic Versioning](https://semver.org/) with the standard
`0.x` pre-1.0 conventions. This document states exactly what consumers can rely on
before `1.0.0` — and enumerates the public API surfaces covered by the policy.

## Pre-1.0 stability contract

Until `1.0.0`:

- **MINOR** releases (`0.5.0` → `0.6.0`) **may contain breaking changes**. This follows
  the semver `0.x` convention: the MINOR version acts as the compatibility boundary.
- **PATCH** releases (`0.6.0` → `0.6.1`) are always backward-compatible: bug fixes,
  performance improvements, and doc updates only. No API removals, no signature
  changes, no on-disk format changes.

Every breaking change is marked in `docs/CHANGELOG.md` via a `feat!:` commit or a
`BREAKING CHANGE:` footer, so consumers can scan for them mechanically.

## The 11 public API surfaces

The surface inventory is frozen by the 2026-09 API standardization campaign
(inventory: `docs/dev/tasks/API-STD-01.md`; binding decisions:
`API-STD-15.md`). A change that breaks any surface below requires a MINOR bump
plus an explicit breaking-change note (`feat!:` / `BREAKING CHANGE:`).

| # | Surface | Entry point | Normative contract |
|---|---------|-------------|--------------------|
| 1 | Rust core SDK | `vantadb` crate — `Embedded` (`src/sdk/`) | [`EMBEDDED_SDK.md`](EMBEDDED_SDK.md) |
| 2 | Python SDK | `vantadb-python` (PyO3, `import vantadb`) | [`PYTHON_SDK.md`](PYTHON_SDK.md) |
| 3 | TypeScript SDK | `vantadb-ts` (`vantadb.ts`, `native.ts`) | [`TS_SDK.md`](TS_SDK.md) |
| 4 | Node.js SDK (NAPI) | `vantadb-node` | [`NODE_SDK.md`](NODE_SDK.md) |
| 5 | WebAssembly SDK | `vantadb-wasm` + OPFS/IDB persistence | [`WASM_API.md`](WASM_API.md) · [`WASM_PERSISTENCE.md`](WASM_PERSISTENCE.md) · [`WASM_STANDALONE.md`](WASM_STANDALONE.md) |
| 6 | HTTP API + OpenAPI | `vantadb-server` — `/api/v2/*` | [`HTTP_API.md`](HTTP_API.md) + [`openapi.yaml`](openapi.yaml) |
| 7 | MCP server | `vanta-cli server --mcp` (stdio) | [`MCP.md`](MCP.md) |
| 8 | IQL | `IQL_VERSION = 4` grammar (`src/parser/`) | [`IQL.md`](IQL.md) |
| 9 | CLI | `vanta-cli` | [`CONFIGURATION.md` §4 Embedded CLI](../user/operations/CONFIGURATION.md) + built-in `--help` |
| 10 | LLM proxy | `vanta-proxy` | [`PROXY.md`](PROXY.md) |
| 11 | vanta-memory (Rust crate) | `vanta-memory` — L0–L3 pipeline, core-only | [`VANTA_MEMORY.md`](VANTA_MEMORY.md) |

Cross-surface wire conventions pinned by the same campaign (each surface's
reference doc is authoritative for its own wire):

- **Search semantics:** `score` (higher is better) for memory/hybrid search;
  `distance` (lower is better) only for raw ANN vectors — [`scores.md`](scores.md).
- **Identifiers:** `u128` crosses every JSON/FFI wire as a decimal string (no
  precision loss above 2^53) — [`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md).
- **Errors:** one envelope shape (`code` + `message` + `context`) across bindings
  and MCP; clients match on the stable `VANTADB_*` codes — [`ERROR_HANDLING.md`](ERROR_HANDLING.md).
- **Casing:** camelCase on JSON/MCP frontiers, kebab-case for CLI commands and
  tool names, native interior (snake Rust/Python, camel TS) — [`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md).

Storage compatibility (WAL, VantaFile segments, index formats) is a separate
contract outside this surface list: see
[`STORAGE_VERSIONING.md`](../dev/architecture/STORAGE_VERSIONING.md).

## Enforcement

The contracts above are checked mechanically, not by convention:

- `scripts/validate-docs-coverage.ps1` — documented SDK methods, config fields,
  CLI commands, Python methods, and MCP tools must all
  appear in their `docs/api/` reference (0 gaps required).
- `cargo test --test openapi_yaml_parity` — `openapi.yaml` stays in lockstep with
  the Axum router. (`HTTP_API.md` mirrors the YAML by docs-sync convention, not
  by this test.)
- `cargo test --test sdk_serialization` — pins the cross-binding wire (`u128`
  strings, score semantics) across Python/TS/Node/WASM.
- `cargo semver-checks check-release` — lints the `vantadb` crate's Rust public
  API against the latest crates.io release (CI job `semver-checks` + release-plz
  `semver_check`). Accepted pre-release deltas are tracked in
  [`COMPATIBILITY.md`](COMPATIBILITY.md).
- `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` — golden snapshot of the simplified
  public API ([`tests/api/public-api.txt`](../../tests/api/public-api.txt)); any
  surface change must update the snapshot deliberately.
- Per-surface enforcement matrix + 1.0 exit criteria:
  [`COMPATIBILITY.md`](COMPATIBILITY.md).

## What is NOT covered

- APIs explicitly marked **experimental** (see `docs/user/operations/EXPERIMENTAL_FEATURES.md`)
  — they may change or disappear in any release, including PATCH.
- Rust crate internals (non-public modules, private structs, internal traits) —
  including `vanta-memory` modules beyond its documented crate API.
- IQL behavior beyond what `docs/api/IQL.md` documents.
- Adapters, integrations, `desktop/`, and `web/` — they consume the surfaces
  above but are not themselves covered.

## Deprecation policy

- A deprecated API gets a deprecation notice in the changelog and stays functional
  for **at least one MINOR release** before removal.
- Removal then happens in a subsequent MINOR release with a `feat!:` /
  `BREAKING CHANGE:` marker.
- Concrete deprecations are registered in [`DEPRECATIONS.md`](DEPRECATIONS.md)
  (entry shape + removal window per instance).
- The 9 artifact names (`vantadb`, `vantadb-py`, `vantadb-node`,
  `vantadb-ts`, `vantadb-server`, `vantadb-mcp`, `vanta-cli`, `vanta-proxy`,
  `vanta-memory`) are frozen through `1.0.0` by the naming freeze
  [ADR-0045](../dev/architecture/adr/ADR-0045-naming-freeze.md) — renaming one is
  never silent: it ships an alias with a mandatory removal date, registered in
  `DEPRECATIONS.md` (after `1.0.0`, a breaking name change requires a MAJOR).

## Release mechanics

Version bumps, tags, and changelog entries are produced automatically by
[release-plz](https://release-plz.github.io/) + [git-cliff](https://git-cliff.org/)
from Conventional Commits. See the *Release Workflow* section of
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for the full flow, and
[`docs/user/operations/UPGRADE.md`](../user/operations/UPGRADE.md) for consumer upgrade guidance.
