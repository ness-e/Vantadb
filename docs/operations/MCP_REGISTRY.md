---
title: "MCP Registry — server.json & ecosystem listings"
type: operations
status: active
tags: [vantadb, mcp, registry, ecosystem]
last_reviewed: 2026-08-29
aliases: [MCP_REGISTRY, registry-manifest]
---

# MCP Registry — `server.json` & ecosystem listings

This document covers VantaDB's presence in the **Model Context Protocol (MCP)
ecosystem**: the canonical `server.json` manifest at the repo root, the
submission state to the official registry, and the optional secondary
listings (glama.ai, smithery).

## What is `server.json`?

`server.json` is a **standardized, versioned manifest** that describes an MCP
server for the [Official MCP Registry](https://registry.modelcontextprotocol.io),
client discovery, and package management. It follows the
[generic server.json spec](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md)
and is required to publish at
[registry.modelcontextprotocol.io](https://registry.modelcontextprotocol.io).

The schema is versioned. Our `server.json` pins
`https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json`
(see `_meta.io.modelcontextprotocol.registry/publisher-provided.build_info.schema`).
When the official registry bumps the schema, regenerate `server.json` to keep
the validator happy.

## Current fields

| Field | Value | Why |
|-------|-------|-----|
| `name` | `io.github.ness-e/vantadb` | GitHub namespace — verifiable automatically (`ness-e` owns the GitHub org). |
| `title` | `VantaDB MCP` | Human-readable display name for clients. |
| `description` | "MCP server for VantaDB: durable local memory and hybrid vector retrieval (BM25 + HNSW) for AI agents." | Required (max 100 chars per schema). |
| `version` | `0.5.0` | Synced with `[workspace.package].version` in `Cargo.toml`. Update on every release. |
| `repository` | `https://github.com/ness-e/Vantadb` (source: `github`) | Required for GitHub namespace verification. |
| `websiteUrl` | docs/api/MCP.md on develop branch | Directs users to the canonical MCP documentation. |
| `packages` | **absent** | `vantadb-mcp` is `publish = false` and not on crates.io yet. See [Submission state](#submission-state). |
| `remotes` | **absent** | No hosted MCP service. `vantadb-mcp` is stdio-only. |
| `_meta.io.modelcontextprotocol.registry/publisher-provided` | publisher metadata | `build_info.timestamp` records the last `server.json` regenerate. `submission_state` is `pending` until the PR is merged at the registry. |

## Why no `packages` or `remotes`?

The registry requires **at least one** of `packages[]` or `remotes[]` —
**except** when a `websiteUrl` with custom installation instructions is
provided (see the spec's
["Server with Custom Installation Path" example](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md#server-with-custom-installation-path)).
VantaDB uses that escape hatch today because:

1. **`vantadb-mcp` is not published to crates.io** (`publish = false` in
   `vantadb-mcp/Cargo.toml`) — so a `cargo` package entry would be misleading.
2. **No Docker image / OCI artifact** is published for `vantadb-mcp` yet
   (planned as `ghcr.io/ness-e/vantadb-mcp`).
3. **No hosted MCP service** is available — the binary is stdio-only and
   runs on the user's machine.

Installation today:

```bash
cargo install --git https://github.com/ness-e/Vantadb vantadb-mcp
```

Or build from source (see [`docs/api/MCP.md`](../api/MCP.md)).

## Submission state

| State | Value |
|-------|-------|
| Target | https://github.com/modelcontextprotocol/registry (`io.github.ness-e/vantadb` namespace) |
| Tracking PR | **TBD** — open after `server.json` is merged to `develop` and release-plz ships `0.5.0`. |
| Submission script | `mcp-publisher` CLI (per the [publishing guide](https://github.com/modelcontextprotocol/registry/blob/main/modelcontextprotocol-io/quickstart.mdx)). Manual until automated. |
| Approval | **Manual** — registry maintainers review namespace ownership + package metadata. Expected turnaround: 1–7 days. |

Until the PR is approved, `_meta.io.modelcontextprotocol.registry/publisher-provided.build_info.submission_state`
stays at `pending`. Update to `submitted` / `approved` / `rejected` per
[registry PR lifecycle](https://github.com/modelcontextprotocol/registry/pulls?q=is%3Apr+label%3Asubmission).

> **Pre-mortem (from MCP-40 task):** if the registry rejects the submission
> (e.g., GitHub namespace cannot be verified, or schema validator is stricter
> than the generic spec), document the rejection reason here and update
> `server.json` accordingly. The release is **not blocked** by registry
> approval — the manifest is informational; the binary is the source of truth.

## Aggregators (glama.ai, smithery)

| Aggregator | Submission method | Status |
|------------|-------------------|--------|
| glama.ai/mcp | Auto-discovered from official registry + GitHub README | Passive — once the official registry approves, glama lists it. |
| smithery | Auto-discovered from official registry | Passive — same as glama. |

**No `glama.json` or `smithery.json` manifests are required.** Both scrapers
read from either the official registry or the GitHub repo's `server.json` /
`README.md`. If glama or smithery ever publish a stricter local manifest
format, the **first** task is to re-evaluate the spec — we will not maintain
parallel manifests in the meantime (Ponytail: don't duplicate state).

## Pre-mortem

| Failure mode | Mitigation |
|--------------|------------|
| Schema bump breaks `server.json` validation | `server.json` validator is part of the registry's PR CI. Bump `$schema` URL to latest date when a new version drops. Not a release blocker. |
| GitHub namespace verification fails | `ness-e` must remain an active GitHub org with admin push access. Confirmed 2026-08-29. |
| Registry maintainer changes requirements | Spec is followed literally. Read the [official-registry-requirements.md](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/official-registry-requirements.md) at submission time — it changes. |
| A future VantaDB Pro / Enterprise feature conflicts with registry `publish = false` policy | License is Apache-2.0 for the core. The MCP server is part of the core, so registry listing is OK. `vantadb-pro` is a separate artifact (see `docs/architecture/adr/007_open_core_split.md` if it exists, or the open-core rules). |

## How to update `server.json`

For every release:

1. Bump `version` in `server.json` to match the new
   `[workspace.package].version` (release-plz handles the workspace bump).
2. Update `_meta.io.modelcontextprotocol.registry/publisher-provided.build_info.timestamp`
   to the release date (ISO 8601, UTC).
3. If the schema version changes in the registry docs, update `$schema`.
4. If `name` ever changes (e.g., org rename), the registry's GitHub namespace
   authentication will need a re-verification PR.

## See also

- [`docs/api/MCP.md`](../api/MCP.md) — VantaDB MCP server reference (tools, config, profiles).
- [`docs/operations/EDITOR_INTEGRATIONS.md`](EDITOR_INTEGRATIONS.md) — How to wire VantaDB MCP into Cursor, VS Code, OpenCode, etc.
- [`docs/operations/CI_POLICY.md`](CI_POLICY.md) — CI/certification policy (note: `server.json` is **not** a CI gate today).
- [Official MCP registry](https://registry.modelcontextprotocol.io/) — Public listing.
- [MCP spec](https://modelcontextprotocol.io/) — Protocol definition.
