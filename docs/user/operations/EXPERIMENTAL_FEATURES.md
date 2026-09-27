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
the consolidated scope inventory (core-promise vs labs) is defined in §Scope Budget below.

> **Revision 2026-09-27 (DEF-02):** full row-by-row re-verification against the codebase at workspace version 0.7.0 (`Cargo.toml:781`); stale version references removed, MCP tool count corrected to 85, labs categories (core-promise vs labs) made explicit, dead example paths and links fixed. Supersedes the 2026-09-24 pass (false IQL, MCP, consolidation, PyPI, and tokenizer claims corrected; Labs rows introduced). The historical 0.1-era revision of this document is obsolete.
>
> **Revision 2026-09-27 (DEF-07):** Scope Budget section added — the core-promise vs labs classification is finalized per surface (DEF-02's provisional labels resolved: `vanta-memory` stays core-promise; `vanta-proxy`, Vanta Studio and the web console stay labs) and the inversion rule (admission → promotion → freeze) is documented. `SPEC.md` §Frontera now references this section.

## Scope Budget — core-promise vs labs

Every surface spends one of two budgets. **core-promise** surfaces spend the budget of the product promise (`SPEC.md` Objective): an active agent goes from the repo to a successful memory recall with one command and zero configuration. **labs** surfaces are declared secondary: they ship, but they do not consume that budget. This is the consolidated scope inventory announced in the header; the sections below keep the area-level detail and shipping status — where a row there carries a category, it matches this table.

**Classification rule (North Star test).** A surface is core-promise only if it directly serves the product North Star — *active agents that successfully recall a memory within a 7-day window* (`SPEC.md` § North Star; `VISION.md`) — or one of the three ICP entry gates served by the single core (AI-IDEs via MCP · local-LLM/privacy · frameworks; DEF-01 hierarchy) — or instruments the promise's guardrails/evidence layer (harness/benchmarks, row 7). Every other surface is labs by default. Each row cites its evidence; no row is classified by preference.

| Surface | Category | Disposition | Evidence |
| --- | --- | --- | --- |
| Core engine + Rust SDK + CLI (incl. installers) | **core-promise** | Accelerate | The recall path itself: `src/sdk/api/memory.rs`, `src/wal.rs`; one-command installers per SPEC F1/F2 |
| Python SDK (`vantadb-py`) | **core-promise** | Accelerate | QUICKSTART path (Production-Facing MVP below); ICP-03 entry via PyPI (`VISION.md` §Update 2026-09-24) |
| MCP server (`vantadb-mcp`) | **core-promise** | Accelerate | ICP-01 entry gate (`VISION.md` §Update); North Star measured "in proxy/MCP" (DEF-05) |
| `vanta-memory` (L0→L3) | **core-promise** | Accelerate (partial — scheduler host pending, WIRE-01) | Memory lifecycle is the North Star substance (capture → recall → consolidate); SPEC F5; finalizes DEF-02's provisional label |
| Local embeddings (`embed-local`) | **core-promise** | Accelerate | The zero-config promise requires real local embeddings (SPEC F1/F6); Optional section below ("not Experimental") |
| Framework adapters & importers (`integrations/`) | **core-promise** | Accelerate | ICP-03 gate: adapters on PyPI + Mem0/Zep importers (`VISION.md` §Update; MKT-18f) |
| Verification harness & benchmarks (`benches/`, `benchmarks/`) | **core-promise** | Accelerate | Evidence layer of the promise: guardrails (p99 gate), Regla 11 public claims, memory-quality harness (VER-08/09) |
| `vanta-proxy` (LLM gateway) | **labs** | Freeze gateway features (rate-limit, cache/translate, failover, cost, virtual keys). Carve-out, maintenance only: memory loop (capture/inject) is the MVP auto-recall path and the North Star instrumentation (DEF-05); the redaction path feeds ICP-02 | DEF-02 "Category: labs"; research §3 (`docs/dev/research/product-definition-gap-2026-09-24.md`) |
| `vantadb-server` (HTTP wrapper) | **labs** | Freeze — optional wrapper (local dev / network exposure); maintenance only. Only core-adjacent role: prospective host of the `vanta-memory` scheduler (`src/server/bootstrap.rs:332`, WIRE-01) — promote that role via the inversion rule if it lands there | Optional section below; research §3 (JWT/rate-limit = overrun); no ICP gate requires it |
| Vanta Studio (desktop) | **labs** | Freeze the 12-surface GUI product. Promotion candidate: ICP-01's minimal "viewer" if F5 evidence requires it — viewer only, not the Studio | DEF-02 "Category: labs"; research §3 (GUI = overrun) |
| Web console | **labs** | Freeze — separate site ([`ness-e/Vantadb-web`](https://github.com/ness-e/Vantadb-web)); no surface in this tree | Experimental section below; DEF-02 "Category: labs" |
| WASM/TS/Node bindings (`vantadb-wasm`, `vantadb-ts`, `vantadb-node`) | **labs** | Freeze — published artifacts stay; no new feature budget. Promotion requires North Star evidence from JS runtimes or an ICP gate pulling them | research §3 (overrun); ICP-03 is Python/PyPI only (`VISION.md`); `vantadb-wasm` outside `default-members` (Cargo.toml, ADR-031) |
| LLM providers (`remote-inference`: Ollama/OpenAI/litellm) | **labs** | Freeze — external optional integration, alternative to `embed-local` | Experimental section below ("not core dependency") |

### Inversion rule (admission → promotion → freeze)

1. **Admission.** New surfaces (crate, binding, app, integration) enter as **labs by default**. "It might be useful" is not an admission criterion; a surface that cannot cite its evidence starts frozen. Every admitted surface must appear in this document — the CI frontier gate (`scripts/validate-frontier.ps1`) enforces the doc↔repo coupling for Cargo features and workspace members.
2. **Promotion.** A labs surface becomes core-promise only with evidence tied to the North Star or an ICP gate metric: sessions with successful recall in the 7-day window (proxy/MCP instrumentation), an ICP pilot that requires it, or the falsification of a core alternative. Promotion is recorded as a row change here. Demos, roadmap proximity, and symmetric effort do not count.
3. **Freeze.** Labs surfaces are frozen: security fixes and release-blocking regressions still apply, but no new product features. Frozen is the default state; thawing requires promotion.

**Mechanical consequences.** Core-promise surfaces carry the promise's quality bars (SPEC success criteria + North Star guardrails: p99 regression gate, Regla 11 claims, install SLO per DEF-08). Labs surfaces cannot block a core release — precedent: `vanta-proxy` and `vantadb-wasm` already sit outside `default-members`/Fast Gate (Cargo.toml CATEGORY comment, ADR-031). When labs and core-promise compete for the same budget (attention, CI time, release risk, doc surface), core-promise wins by default.

**Promotion watchlist (next triggers).** `vanta-proxy` gateway features: only with session telemetry showing gateway use drives successful recalls · Vanta Studio: ICP-01/F5 evidence that a minimal viewer is required · WASM/TS/Node: JS-runtime North Star evidence or a new JS gate · `vantadb-server`: the WIRE-01 scheduler-host decision or ICP-02 network-deployment evidence. **Review cadence:** re-checked every release together with the frontier gate, and on every new-surface proposal (rule 1).

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
| **vanta-memory (L0→L3)** | **Category: core-promise** (finalized by the DEF-07 Scope Budget) — **Partially shipped**: full pipeline; scheduler without a ubiquitous host (`src/server/bootstrap.rs:332` → `conversation_trigger: None`; trigger trait implemented in `vanta-memory/src/services/conversation_hook.rs`); dreams dry-run/promote real: VER-07 (P52) | `vanta-memory/` · `src/server/bootstrap.rs:332` |

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
