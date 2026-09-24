---
title: VantaDB Distribution & Adoption Plan
type: operations
status: active
tags: [vantadb, distribution, npm, adoption, comparison]
last_reviewed: 2026-09-19
aliases: []
---

# VantaDB Distribution & Adoption Plan

Strategy combining Backlog `TS-10` (distribution/adoption: playground +
docs-site + comparison) and `WSM-14` (npm adoption plan, H-21 strategy).
Written strategy only — zero code, zero web changes.

> **Status gate:** announcement posts in §4 are **PREPARED, not published**.
> Publishing stays paused until Fase A (`EXE-03`) is executed owner-side.

---

## 1. Distribution channels

| Channel | Artifact | Install | Source of truth |
| :--- | :--- | :--- | :--- |
| **PyPI** | `vantadb-py` (`import vantadb`) | `pip install vantadb-py` | [README → Installation](../README.md#installation) · [PYTHON_RELEASE_POLICY](user/operations/PYTHON_RELEASE_POLICY.md) · TestPyPI first (`TEST_PYPI_API_TOKEN`), then PyPI — pending `PROV-12` |
| **npm (browser/Node)** | `vantadb` 0.5.0 (WASM, ESM-only, `engines: node>=22.19`) | `npm install vantadb` · browser via `esm.sh` (verified 2026-08-26); jsDelivr `+esm` does **not** work (Rollup limitation) | [`vantadb-ts/README.md`](../vantadb-ts/README.md) · [`vantadb-ts/package.json`](../vantadb-ts/package.json) |
| **npm (native Node)** | `vantadb-node` (napi-rs, async, real fjall/WAL persistence) | **Not yet published** (registry 404) — do not advertise as installable | [`vantadb-ts/README.md` §"vantadb vs vantadb-node"](../vantadb-ts/README.md#vantadb-vs-vantadb-node-npm) |
| **GitHub Releases** | `vanta-cli` / `vantadb-server` binaries + wheels | One-liner without clone: `install.sh` / `install.ps1` (sha256-verified, chains to setup wizard) | [QUICKSTART §0](user/QUICKSTART.md#0-install-without-cloning-no-cloner-no-rust-toolchain) · [README → Embedded CLI](../README.md#embedded-cli) |
| **From source** | Rust workspace + maturin develop | Contributors only | [QUICKSTART §1-§4](user/QUICKSTART.md#1-prerequisites) |

Docs-site and playground are adoption surfaces, not install channels: the
playground carries 6 clickable recipes (SHOW-02) and the RAG-over-PDFs demo
runs 100% local with verifiable citations (SHOW-03). `vantadb-ts/examples/`
(LangChain, LlamaIndex, Vercel AI SDK) stays referenced from README/QUICKSTART
(SHOW-05 decision) — not moved, not duplicated.

---

## 2. Honest comparison vs Orama (TS-13 content)

**Rule (Regla 11):** every number below names its source and measurement date.
A number without bench + command (or a verified registry/docs source) is
removed, not argued. Our own performance figures live in
[BENCHMARKS.md](user/operations/BENCHMARKS.md) and [COMPARISON.md](COMPARISON.md)
(sqlite-vec / LanceDB / Qdrant / Chroma — Orama is covered here, not there).

### 2.1 What Orama verifiably is (checked 2026-09-19)

Source: <https://www.npmjs.com/package/@orama/orama> (v3.1.18, Apache-2.0,
**686,968 weekly downloads** at check time) and
<https://docs.orama.com/docs/orama-js/search/hybrid-search> (resolves;
nav confirms Vector, Hybrid, BM25, Facets, Filters, Geosearch pages).

- Full-text-first search engine (JS puro): `mode: 'fulltext' | 'vector' | 'hybrid'`,
  BM25, typo tolerance, facets, geosearch, filters, boosting, 30 languages.
- Embeddings via `@orama/plugin-embeddings`; client-side OpenAI via Secure Proxy;
  GenAI Answer Sessions (chat/RAG UX) since v3.0.0.
- Persistence is a **plugin** (`@orama/plugin-data-persistence`, JSONL
  serialization) over an in-memory core — not a native durable engine.
- Zero-install browser evaluation: `cdn.jsdelivr.net/npm/@orama/orama@latest/+esm`.
- The "21μs" figure in Orama's README is a trivial single-doc example without a
  dataset — not a citable benchmark (same ruling as research §53).

### 2.2 Size (sourced, dated)

| Library | Version | Gzipped | Source |
| :--- | :--- | :--- | :--- |
| `@orama/orama` | 3.1.18 | **23.8 KB** (75.2 KB min) | <https://bundlephobia.com/package/@orama/orama>, measured 2026-08-30 |
| VantaDB WASM | 0.5.x | **~671 KB transfer** (1.58 MB raw) | `vantadb-wasm/pkg/` via .NET GzipStream, measured 2026-09-15 (`pkg/` built 2026-09-11) |

Full table (MiniSearch 5.9 KB, Lunr 8.1 KB) and the 7-item feature-gap list live
in [`vantadb-wasm/README.md` §4](../vantadb-wasm/README.md#4-honest-comparison-vs-javascript-only-search-engines)
— read there, not repeated here. (An older 599 KB transfer figure in
`vantadb-ts/README.md`, measured 2026-08-30, is superseded by the 2026-09-15
measurement; rebuild `pkg/` before quoting for release.)

### 2.3 Adoption (sourced, dated)

- Orama: 686,968 downloads/week verified 2026-09-19 (npm page above).
- VantaDB npm: **187 downloads/month** measured 2026-07-26→08-24 via
  api.npmjs.org (H-21,
  `docs/dev/reviews/archive/research-vantadb-wasm-20260825.md`). Current figure
  **TODO re-verify** (`GET https://api.npmjs.org/downloads/point/last-month/vantadb`)
  before any announcement quotes it — never quote H-21's number as current.

### 2.4 When to pick what (both directions honest)

- **Pick Orama** if you need full-text + RAG in-memory with no persistence,
  ≤1k docs, mature search-engine features (facets/geosearch/typo tolerance),
  plugin ecosystem, and a 23.8 KB footprint.
- **Pick VantaDB** if any of these matter: OPFS/IndexedDB durable persistence
  with WAL across reloads, HNSW k-NN past linear-scan scale, BM25+vector RRF
  fusion in one query, typed graph traversal (BFS/DFS/topo/DAG) beside search,
  TTL auto-expiry — at ~671 KB transfer.
- **No performance winner is declared.** Nobody in this niche publishes a
  reproducible public JS/WASM benchmark (research §53; our JS/WASM path has
  zero published numbers, H-11). The qualitative matrix is in
  `docs/dev/reviews/archive/research-vantadb-ts-20260825.md` §3 (verified 2026-08-25).

---

## 3. Niche positioning — "browser AI agent memory"

One-line: **the only embedded JS SDK that gives browser AI agents durable
memory (OPFS + WAL) with native hybrid RRF search and graph traversal** —
versus Orama's FTS-first in-memory engine (H-13,
research §55-59).

What we claim (each with evidence above or linked): durable browser
persistence · HNSW + BM25 + RRF native · graph + IQL · typed `VantaError`s ·
Apache-2.0 · offline-capable (`embed-local`, optional).

What we deliberately do **not** claim: faster than anyone (§2.4) · smaller
than anyone (§2.2 admits ~25-28× Orama) · production-hardened at scale
(pre-1.0, smaller ecosystem — say it) · published `vantadb-node` (still 404).

npm keywords (already in `vantadb-ts/package.json`):
`vector-database, memory, rag, embeddings, graph, wasm, ai, semantic-search`.

---

## 4. Announcement checklist (PREPARE only — publishing paused until Fase A)

| # | Item | Owner | Status |
| :--- | :--- | :--- | :--- |
| 1 | Re-verify §2 numbers (bundlephobia Orama + rebuild `pkg/` + npm downloads both sides) | owner | TODO |
| 2 | `PROV-12`: TestPyPI → PyPI green (`pip install` clean-env + smoke) | owner (Gate V: secrets) | blocked on Gate V |
| 3 | `EXE-03`: Fase A kit executed (5 humans, stranger-test) | owner-side | kit pending |
| 4 | Draft Show HN / launch post from §2-§3 (no new numbers, link BENCHMARKS + §4 wasm README) | owner | TODO — do not post |
| 5 | Draft npm README niche line ("browser AI agent memory") + esm.sh recipe | owner | TODO — do not publish |
| 6 | Post only after 1-3 green; monitor error rate / P95 per plan §2b thresholds | owner | paused |

Rollback for a bad announcement: correct the doc (`git revert` + re-publish
patch version note), never edit a published registry artifact in place.
