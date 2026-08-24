---
title: Versioning & Stability Policy
type: api
status: active
tags: [vantadb, api, semver]
last_reviewed: 2026-08-23
aliases: []
---

# Versioning & Stability Policy

VantaDB follows [Semantic Versioning](https://semver.org/) with the standard
`0.x` pre-1.0 conventions. This document states exactly what consumers can rely on
before `1.0.0`.

## Pre-1.0 stability contract

Until `1.0.0`:

- **MINOR** releases (`0.5.0` → `0.6.0`) **may contain breaking changes**. This follows
  the semver `0.x` convention: the MINOR version acts as the compatibility boundary.
- **PATCH** releases (`0.6.0` → `0.6.1`) are always backward-compatible: bug fixes,
  performance improvements, and doc updates only. No API removals, no signature
  changes, no on-disk format changes.

Every breaking change is marked in `docs/CHANGELOG.md` via a `feat!:` commit or a
`BREAKING CHANGE:` footer, so consumers can scan for them mechanically.

## What counts as stable public API

The following surfaces are covered by this policy:

| Surface | Contract |
|---------|----------|
| Python SDK (`import vantadb`) | Documented methods and types in `docs/api/PYTHON_SDK.md` |
| TypeScript SDK (`vantadb-ts`) | Documented API in `docs/api/TS_SDK.md` |
| HTTP API | `/api/v2/*` routes as defined by `docs/api/openapi.yaml` |
| MCP tools | Tools listed in `docs/api/MCP.md` |
| On-disk format | Files written under the data directory (WAL, VantaFile segments) |

A change that breaks any of these requires a MINOR bump plus an explicit breaking
change note.

## What is NOT covered

- APIs explicitly marked **experimental** (see `docs/operations/EXPERIMENTAL_FEATURES.md`)
  — they may change or disappear in any release, including PATCH.
- Rust crate internals (non-public modules, private structs, internal traits).
- IQL behavior beyond what `docs/api/IQL.md` documents.

## Deprecation policy

- A deprecated API gets a deprecation notice in the changelog and stays functional
  for **at least one MINOR release** before removal.
- Removal then happens in a subsequent MINOR release with a `feat!:` /
  `BREAKING CHANGE:` marker.

## Release mechanics

Version bumps, tags, and changelog entries are produced automatically by
[release-plz](https://release-plz.github.io/) + [git-cliff](https://git-cliff.org/)
from Conventional Commits. See the *Release Workflow* section of
[`CONTRIBUTING.md`](../../CONTRIBUTING.md) for the full flow, and
[`docs/operations/UPGRADE.md`](../operations/UPGRADE.md) for consumer upgrade guidance.
