---
title: "VantaDB for AI IDEs — persistent project memory over MCP"
kind: howto
status: active
description: "One MCP server for persistent project memory in Claude Code, Cursor, OpenCode and Codex — what works today, how to install it, and what is not there yet"
tags: [vantadb, mcp]
---

# VantaDB for AI IDEs — persistent project memory over MCP

Your AI IDE forgets. A new chat starts blank, `CLAUDE.md` is a static snapshot, and the context you built yesterday is gone. VantaDB is a local-first memory engine that plugs into the IDEs you already use through a single **Model Context Protocol (MCP) server**: it stores notes, decisions and conversation turns, and hands them back through hybrid search ([BM25](glosario/bm25.md) + dense vectors fused with RRF) when a new session starts.

One integration, many clients. Claude Code, Cursor, VS Code, OpenCode, OpenClaw, Devin and Antigravity all speak MCP — per-editor setup lives in [EDITOR_INTEGRATIONS.md](operations/EDITOR_INTEGRATIONS.md); the server reference in [docs/api/MCP.md](../api/MCP.md).

## Install

### 1. Install the CLI

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex
```

The installer verifies the checksum and puts `vanta-cli` in `~/.vanta/bin` (Windows: `$HOME\.vanta\bin`).

> **Version note (honest):** `vanta-cli server --mcp` works with the released CLI (≥ 0.5). The **hooks launcher and the metric script** below use the `vanta-cli mcp-call` subcommand, which ships in the **0.8.0 train** — it is on `develop` today and the current latest release (v0.7.0) predates it. Until 0.8.0 ships, build from `develop` if you want hooks/metrics.

### 2. Point your IDE at VantaDB

Claude Code — project-level `.mcp.json`, or one-shot via the CLI:

```bash
claude mcp add vantadb -- vanta-cli server --mcp --db C:/Users/<you>/.vantadb
```

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--db", "C:/Users/<you>/.vantadb"]
    }
  }
}
```

Use an **absolute path**: MCP clients spawn the server without a shell, so `~` arrives literally and the open fails. Cursor, VS Code, OpenCode and the rest: [EDITOR_INTEGRATIONS.md](operations/EDITOR_INTEGRATIONS.md).

The default `agent` profile lists 38 tools (memory CRUD, search, recall, threads, scenes, wiki read); `VANTADB_MCP_PROFILE=full` lists all 80 for unrestricted clients ([MCP.md § Profiles](../api/MCP.md)).

### 3. Optional — automatic recall (hooks)

Deterministic hooks wire `memory_recall` into each session: recall on session start, recall per user message, save-state before compaction, auto-capture on stop. Templates exist for **Claude Code, Codex, Cursor and OpenCode** with pinned `template_version` + `docs_verified` dates, and the policy (what to recall, budget, verbatim rules) is [recall-policy.md](../../skills/vantadb-mcp/references/recall-policy.md):

- Hook map + per-client install: [hooks/README.md](../../skills/vantadb-mcp/assets/hooks/README.md)
- Offline schema test: `pwsh -NoProfile -File skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1`

## What works today (and how it is verified)

| Surface | Status | Evidence |
|---------|--------|----------|
| MCP server over stdio, tool profiles | ✅ | `vanta-cli server --mcp` — [MCP.md](../api/MCP.md) |
| Hooks for 4 clients | ✅ schema-tested offline | [hooks/README.md](../../skills/vantadb-mcp/assets/hooks/README.md) + `test-hooks.ps1` |
| Desktop memory viewer (Memory Lens) | ✅ in the desktop app | [desktop/README.md](../../desktop/README.md) |
| Cross-session recall demo | ✅ PR-blocking in CI | [demo_ai_ides.rs](../../vantadb-mcp/tests/demo_ai_ides.rs) · [ci-ai-ides-demo.yml](../../.github/workflows/ci-ai-ides-demo.yml) |
| North Star metric (put+search per 7 days) | ✅ proxy-side | [north_star_metric.py](../../scripts/north_star_metric.py) · [PROXY.md § North Star](../api/PROXY.md) |

The demo is the acceptance test of this track: session 1 stores a note (`memory_put` with embed-on-put), session 2 recalls it with a **synonym query that shares zero literal tokens** with the stored text (a keyword-only path could never hit it — the test asserts the zero-overlap property itself). Embeddings come from a deterministic in-process fake provider, declared as a test double: it proves the pipeline (embed → store → embed query → similarity ranking), not model quality. Offline, no tokens, no model files — a regression turns CI red.

Run it locally (offline, no tokens; expected: the test passes — session 2 recalls the session-1 note):

```bash
cargo nextest run -p vantadb-mcp --features remote-inference --test demo_ai_ides --build-jobs 2
```

Expected output (abridged):

```text
PASS [   1.415s] (1/1) vantadb-mcp::demo_ai_ides session_one_stores_session_two_recalls_by_synonym
Summary [   1.415s] 1 test run: 1 passed, 0 skipped
```

## Honest limits

- **No live repo-map / incremental repo indexing yet.** The structural MCP tools (`code_search`, `code_callers`, `code_impact`, …) answer on-demand graph queries, but a persistent "map of your repo" with a file watcher that re-indexes changes is **not shipped** — it is a declared research line on the roadmap. Do not expect `CLAUDE.md`-style project overviews generated automatically.
- **Hooks are schema-tested, not live-smoke-tested** against each client binary in CI; a client-side schema change is caught by `test-hooks.ps1`, but end-to-end behavior per client is validated by the templates' pinned docs, not by a live integration test.
- **The desktop viewer requires the desktop app** (Tauri build); there is no packaged release for every platform yet.
- **Pre-1.0.** Expect API changes between minor releases; the naming freeze and deprecation policy are in [VERSIONING.md](../api/VERSIONING.md).
- **This page is not a comparative benchmark.** For positioning against other stores see [COMPARISON.md](COMPARISON.md) — and note it deliberately publishes no competitor performance figures.

## Learn more

- [MCP.md — server reference, profiles, client configs](../api/MCP.md)
- [EDITOR_INTEGRATIONS.md — per-editor setup](operations/EDITOR_INTEGRATIONS.md)
- [hooks/README.md — automatic recall templates](../../skills/vantadb-mcp/assets/hooks/README.md)
- [recall-policy.md — what gets recalled, and when](../../skills/vantadb-mcp/references/recall-policy.md)
- [PROXY.md — North Star metric](../api/PROXY.md)
- [QUICKSTART.md — first steps with VantaDB](QUICKSTART.md)
