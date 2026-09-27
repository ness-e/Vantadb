---
title: Experimental Features and Product Boundary
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-09-27
aliases: []
---

# Experimental Features and Product Boundary

This document classifies the current v0.7.0 repository surface. It is the operational reference for
what is production-facing, optional, experimental, or deferred. Every row carries verification
evidence (Cargo feature, `file:line`, or command). JavaScript/WASM bindings (`vantadb-ts`,
`vantadb-node`, `vantadb-wasm`) and the provider packages are separate distribution surfaces
with their own docs (see `docs/api/TS_SDK.md`, `docs/api/NODE_SDK.md`, `docs/api/WASM_API.md`);
the consolidated scope inventory (core-promise vs labs) lands with the DEF-07 Scope Budget.

> **Revision 2026-09-27 (DEF-02):** full row-by-row re-verification against the codebase at workspace version 0.7.0 (`Cargo.toml:781`); stale version references removed, MCP tool count corrected to 85, labs categories (core-promise vs labs) made explicit, dead example paths and links fixed. Supersedes the 2026-09-24 pass (false IQL, MCP, consolidation, PyPI, and tokenizer claims corrected; Labs rows introduced). The historical 0.1-era revision of this document is obsolete.

## Production-Facing MVP

The product boundary (v0.7.0) is an embedded local-first persistent memory engine:

| Area | Status | Evidence |
| --- | --- | --- |
| Embedded Rust SDK and CLI | Production-facing | `feature:cli` (`Cargo.toml:157`) · `src/sdk/` · `src/cli.rs` |
| Memory `put/get/delete/list/search` | Production-facing | `src/sdk/api/memory.rs` · `src/sdk/api/namespaces.rs` (`list`) · `src/sdk/search/mod.rs` (`search`) |
| WAL-backed recovery | Production-facing | `src/wal.rs`, `src/wal_sharded.rs` · `tests/storage/wal_resilience.rs` |
| Namespaces and scalar metadata filters | Production-facing | `src/sdk/api/namespaces.rs` · `src/sdk/types/record.rs` (`MemoryFilterItem`) |
| Derived namespace and metadata indexes | Production-facing | `src/sdk/api/admin.rs` (`rebuild_index`) · `src/sdk/serialization/impl_rebuild.rs` |
| HNSW vector retrieval | Production-facing | `src/index/` (HNSW build/search) · `src/index/graph/` |
| BM25 lexical retrieval | Production-facing | `src/text_index.rs` · `src/sdk/search/lexical.rs` |
| Hybrid Retrieval v1 with deterministic RRF | Production-facing | `src/sdk/search/fusion.rs` (`fuse_rrf`, `RRF_K`) |
| Basic phrase filtering | Production-facing | `src/sdk/search/phrase.rs` · `src/text_index.rs` (phrase groups) |
| Manual rebuild and structural audit flows | Production-facing | `src/sdk/api/admin.rs` (`rebuild_index`) · `src/cli.rs` (`audit-index`, `doctor`) |
| JSONL export/import | Production-facing | `src/cli.rs` (`export`/`import`) · `src/cli_handlers/data.rs` (jsonl) |
| Source-installed Python SDK | Production-facing | `vantadb-python/` · [QUICKSTART](../QUICKSTART.md) |

## Optional

| Area | Status | Notes | Evidence |
| --- | --- | --- | --- |
| Local `vantadb-server` binary | Optional wrapper around the embedded core | Local dev / network exposure | `feature:server` (`Cargo.toml:177`) · `vantadb-server/` |
| Local ONNX embeddings (`embed-local` feature, `LocalOnnxProvider`) | **Optional local-first** | Offline, `ort`+`tokenizers`, 9 models (8 ≤3 GB + Qwen3 exception), default `multilingual-e5-small` 384d — see `docs/api/EMBEDDINGS.md`, `embeddings/manifest.json`, `docs/user/tutorials/05-embedding-integrations.md` | `feature:embed-local` (`Cargo.toml:160`) · `src/llm.rs` (`LocalOnnxProvider`) |

The server and `embed-local` are optional wrappers around the same embedded core. `embed-local` is **not Experimental** — it is a supported offline path (BYO-vector remains default).

## Experimental or Not MVP

These surfaces may exist in the repository, but they are not stable product claims for v0.7.0:

| Area | Boundary | Evidence |
| --- | --- | --- |
| IQL/LISP/DQL parser, evaluator, and executor paths | **IQL shipped** (2026-09-24): `POST /api/v2/query` + SDK/CLI/MCP expose SELECT/INSERT/RELATE with JOIN (no aggregations). What was archived (2024-06-10, legacy batch codename, not a release date) is the LISP runtime evaluator (borrow checker/GIL); the legacy fuzz target is preserved in [`FUZZING.md`](../../dev/operations/FUZZING.md). Syntax stabilization: API-06 (P51) | `src/parser/` · `src/server/router.rs` (`/api/v2/query`) · `src/cli.rs` (`query`) |
| MCP API | **Shipped** — 85 tools + resources + prompts (v0.7.0). Naming/schema stabilization: API-04 (P51); profile enforcement + default `agent` profile: WIRE-02 (P56) | `vantadb-mcp/src/handlers/tools.rs:25` ("85 tools total, 47 base + 38 extend") · `docs/api/MCP.md:536` |
| Remote LLM/Ollama integration (`remote-inference` feature, `OllamaProvider`/`OpenAIProvider`) | External optional integration, not core dependency — alternative to `embed-local` | `feature:remote-inference` (`Cargo.toml:159`) · `src/llm.rs` |
| Governance and maintenance semantics | Legacy framework runtime archived (2024-06-10, batch codename). Current governance: supersede/TTL/version_history (SDK) + MGR program (P49) → v0.7 manual → v1.0 automatic. Extracted utilities live in `src/utils/` (Bloom filter wiring into the write path pending — FUT-09/VER-07) | `src/sdk/api/memory.rs` (`supersede`, `versions`, `purge_expired`) · `src/utils/` |
| Graph traversal beyond stored local edges | Experimental, not a graph database claim | `src/sdk/graph.rs` · `src/graphrag/` · MCP `graph_*` tools |
| Docker/Ollama examples | Experimental development examples | `docs/dev/archive/docker/` (archived) · `vantadb-server/docker-compose.yml` |
| **vanta-proxy (LLM gateway)** | **Category: labs** — multi-stage request pipeline (auth → rate-limit → session → mem-commands → inject → redact → context trim → cache/translate → forward + failover → capture/writeback; stages numbered inline in `vanta-proxy/src/server.rs`). `/snapshot` auth: ✅ API-05 (P51) — every route requires `x-vanta-user-key` (`vanta-proxy/src/auth.rs:19`; desktop: FIND-155); memory loop + cost output: WIRE-01 (P56) | `vanta-proxy/` · `docs/api/PROXY.md` |
| **Vanta Studio (Tauri desktop)** | **Category: labs** — 12 workspace surfaces / 3 transport modes; Desktop CI builds installers + runs `src-tauri` tests on Windows/macOS/Linux — frontend unit/E2E tests (vitest/Playwright) still outside CI | `desktop/src/components/layout/WorkspaceShell.tsx` (`Surface`) · `desktop/src/transport.ts` · `.github/workflows/desktop.yml` |
| **Web console** | **Category: labs** — separate site in [`ness-e/Vantadb-web`](https://github.com/ness-e/Vantadb-web) (user docs live in its docs/user/web/ tree) | external repo — no surface in this tree |
| **vanta-memory (L0→L3)** | **Category: core-promise** (provisional, pending the DEF-07 Scope Budget) — **Partially shipped**: full pipeline; scheduler without a ubiquitous host (`src/server/bootstrap.rs:332` → `conversation_trigger: None`; trigger trait implemented in `vanta-memory/src/services/conversation_hook.rs`); dreams dry-run/promote real: VER-07 (P52) | `vanta-memory/` · `src/server/bootstrap.rs:332` |

## Extracted Utilities (Production-Ready)

The following utilities were extracted from experimental governance and are now part of the core API:

| Utility | Purpose | API Location | Source |
| --- | --- | --- | --- |
| DuplicatePreventionFilter | Bloom filter for duplicate prevention in multi-writer scenarios (⚠️ no callers in the write path today — wiring pending, FUT-09/VER-07) | `vantadb::utils::DuplicatePreventionFilter` | `src/utils/duplicate_prevention.rs:30` |
| OriginCollisionTracker | Collision tracking and friction metrics for multi-agent coordination | `vantadb::utils::OriginCollisionTracker` | `src/utils/confidence_metrics.rs:25` |
| compute_confidence_friction | Functional friction metric computation for conflict analysis | `vantadb::utils::compute_confidence_friction` | `src/utils/confidence_metrics.rs:106` |

These utilities are stateless, unit-tested, and suitable for production use in multi-writer and multi-agent scenarios.

Visible in-tree examples and docs must carry the same boundary:

- `examples/python/langchain_ollama_rag.py` — real LangChain + Ollama integration (degrades to offline mocks when Ollama is absent)
- `docs/dev/archive/docker/` — archived local Ollama demo (`docker-compose.yml`, `Dockerfile`; no published images)
- `vantadb-server/docker-compose.yml` · `vantadb-server/docker-compose.prod.yml` — optional server wrapper deployment

## Deferred

The following are explicitly outside the v0.7.0 MVP:

| Area | Boundary | Evidence |
| --- | --- | --- |
| Agent metacognition and automatic memory consolidation | **Partially shipped (2026-09-24)** — dreams L0→L3 + `dream_consolidate`/`promote` in MCP (v0.6.x); dry-run + real promote: VER-07 (P52); automatic consumption depends on the host scheduler (WIRE-01) | `vanta-memory/src/core/dream/` · MCP `dream_*` tools (`vantadb-mcp/src/handlers/tools.rs`) |
| Plugins and marketplace | Deferred | No surface in tree (`rg -li "plugin|marketplace" src` → 0 hits) |
| RBAC, true multi-tenancy, quotas, and enterprise audit | Deferred — a basic `pub(crate)` token→role map already ships with the server (programmatic only, not env-configurable yet — FIND-49); true multi-tenancy, quotas, and enterprise audit remain deferred | `src/rbac.rs` · `src/server/state.rs` (`rbac`) |
| HA, replication, clustering, and cloud managed service | Deferred — no HA/clustering/managed-service surface; an opt-in experimental WAL-shipping primitive exists (not in default features) | `feature:wal-shipping` (`src/wal_shipping.rs`) |
| Production PyPI publication and signed installers | **Partially shipped** — `vantadb-py` (PyPI) and `vantadb` (npm) published with OIDC attestations; binary signatures (cosign/minisign) pending — P0 | `.github/workflows/release-wheels.yml` (`publish-pypi`) · `.github/workflows/release-npm-61.yml` |
| Advanced ranking, snippets, highlighting, Unicode folding, stopwords, stemming | Partial: **advanced tokenizer (stemming + stopwords + ASCII folding) is default** since 0.5.x; snippets/highlighting remain deferred | `feature:advanced-tokenizer` (default, `Cargo.toml:150`) · `src/text_index.rs` |
| SQL, general OLTP, warehouse, and time-series workloads | Deferred | No surface in tree (`rg -li "warehouse|time.?series" src` → 0 hits) |

---

### Cross-References

- [FUZZING.md](../../dev/operations/FUZZING.md) — fuzzing strategy: legacy LISP parser (archived) and core deserialization paths
- [BENCHMARKS.md](BENCHMARKS.md) — published performance benchmarks for production-facing features
