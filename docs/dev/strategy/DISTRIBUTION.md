---
title: "VantaDB Distribution & Adoption Plan"
kind: concept
status: active
description: "Strategy combining Backlog TS-10 (distribution/adoption: playground +"
tags: [vantadb, distribution, npm, adoption, comparison]
---

# VantaDB Distribution & Adoption Plan

Strategy combining Backlog `TS-10` (distribution/adoption: playground +
docs-site + comparison) and `WSM-14` (npm adoption plan, H-21 strategy).
Strategy + executed first slice (2026-10-04): the web npm-install-path card
lives on local branch `ts10/npm-install-card` of `ness-e/Vantadb-web`
(commit `de5e25c`) — see §8. No code in this repo; nothing pushed.

> **Status gate:** announcement posts in §4 are **PREPARED, not published**.
> Publishing stays paused until Fase A (`EXE-03`) is executed owner-side **and
> the install SLO in §5 passes**.

---

## 1. Distribution channels

| Channel | Artifact | Install | Source of truth |
| :--- | :--- | :--- | :--- |
| **PyPI** | `vantadb-py` (`import vantadb`) | `pip install vantadb-py` | [README → Installation](../../../README.md#installation) · [PYTHON_RELEASE_POLICY](../../user/operations/PYTHON_RELEASE_POLICY.md) · TestPyPI first (`TEST_PYPI_API_TOKEN`), then PyPI — pending `PROV-12` |
| **npm (browser/Node)** | `vantadb` 0.8.0 (WASM, ESM-only, `engines: node>=22.19`) | `npm install vantadb` · browser via `esm.sh` (verified 2026-08-26); jsDelivr `+esm` does **not** work (Rollup limitation) | [`vantadb-ts/README.md`](../../../vantadb-ts/README.md) · [`vantadb-ts/package.json`](../../../vantadb-ts/package.json) |
| **npm (native Node)** | `vantadb-node` (napi-rs, async, real fjall/WAL persistence) | **Not yet published** (registry 404) — do not advertise as installable | [`vantadb-ts/README.md` §"vantadb vs vantadb-node"](../../../vantadb-ts/README.md#vantadb-vs-vantadb-node-npm) |
| **GitHub Releases** | `vanta-cli` / `vantadb-server` binaries + wheels | One-liner without clone: `install.sh` / `install.ps1` (sha256-verified, chains to setup wizard) | [QUICKSTART §0](../../user/QUICKSTART.md#0-install-without-cloning-no-cloner-no-rust-toolchain) · [README → Embedded CLI](../../../README.md#embedded-cli) |
| **From source** | Rust workspace + maturin develop | Contributors only | [QUICKSTART §1-§4](../../user/QUICKSTART.md#1-prerequisites) |

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
[BENCHMARKS.md](../../user/operations/BENCHMARKS.md) and [COMPARISON.md](../../user/COMPARISON.md)
(sqlite-vec / LanceDB / Qdrant / Chroma — Orama is covered here, not there).

### 2.1 What Orama verifiably is (re-verified 2026-10-04)

Source: `GET https://registry.npmjs.org/@orama/orama/latest` (v3.1.18,
Apache-2.0, 0 dependencies — re-checked 2026-10-04; weekly downloads
**re-measured 2026-10-04: 1,530,732** via
`GET https://api.npmjs.org/downloads/point/last-week/@orama/orama`; the
686,968 figure was the 2026-09-19 npm-page check — superseded, do not quote),
<https://docs.orama.com/docs/orama-js/search/hybrid-search> (updated
31/10/2025) and the package source (<https://github.com/oramasearch/orama>,
`packages/orama/src/`, fetched 2026-10-04):

- Full-text-first search engine (JS puro): `mode: 'fulltext' | 'vector' | 'hybrid'`,
  BM25, typo tolerance, facets, geosearch, filters, boosting, 30 languages.
- Hybrid fusion is a **weighted score sum** (default `text: 0.5` /
  `vector: 0.5`, configurable via `hybridWeights`) — not RRF
  (`methods/search-hybrid.ts`, `getQueryWeights`).
- Vector search is a **brute-force scan** (cosine over every stored vector,
  default similarity 0.8) — no ANN index (`trees/vector.ts`).
- Persistence is a **plugin** (`@orama/plugin-data-persistence`: `persist` /
  `restore` snapshots, JSON or binary; file persistence is server-only) over an
  in-memory core — not a native durable engine (docs updated 8/9/2025).
- Errors are `Error` objects carrying a `code` string (`errors.ts`,
  `createError`) — no details payload, no JSON wire shape.
- Embeddings via `@orama/plugin-embeddings`; client-side OpenAI via Secure Proxy;
  GenAI Answer Sessions (chat/RAG UX) since v3.0.0.
- Zero-install browser evaluation: `cdn.jsdelivr.net/npm/@orama/orama@latest/+esm`.
- The "21μs" figure in Orama's README is a trivial single-doc example without a
  dataset — not a citable benchmark (same ruling as research §53).

### 2.2 Size (sourced, dated)

| Library | Version | Gzipped | Source |
| :--- | :--- | :--- | :--- |
| `@orama/orama` | 3.1.18 | **23.8 KB** (75.2 KB min) | <https://bundlephobia.com/package/@orama/orama> API re-checked 2026-10-04 (`GET https://bundlephobia.com/api/size?package=@orama/orama@3.1.18` → 24,416 B gzip) |
| VantaDB WASM | **0.8.x** | **~739 KB gzip transfer** (1.77 MB raw; 745,152 B gzip wasm + 11.0 KB glue) | `vantadb-wasm/pkg/` via .NET GzipStream, measured 2026-10-04 from a fresh `wasm-pack build --release` (`vantadb-wasm/README.md` §1, WSM-14) |

Full table (MiniSearch 5.9 KB, Lunr 8.1 KB) and the 7-item feature-gap list live
in [`vantadb-wasm/README.md` §4](../../../vantadb-wasm/README.md#4-honest-comparison-vs-javascript-only-search-engines)
— read there, not repeated here. (Older 599 KB (2026-08-30, `vantadb-ts/README.md`)
and 671 KB (2026-09-15) transfer figures are superseded by the 2026-10-04
measurement; rebuild `pkg/` before quoting for release.)

### 2.3 Adoption (sourced, dated — re-measured 2026-10-04)

- Orama: **1,530,732 downloads/week · 5,310,295/month** (re-measured 2026-10-04,
  `GET https://api.npmjs.org/downloads/point/last-week/@orama/orama`; the 686,968
  figure was the 2026-09-19 npm-page check — superseded, do not quote).
- VantaDB npm (same source/date; week 2026-09-27→10-03, month 2026-09-04→10-03):
  - `vantadb-wasm` (browser bindings): **232/week · 764/month**.
  - `vantadb` (TypeScript SDK): **210/week · 548/month**.
  - Context: H-21 measured **187/month** for the binding over 2026-07-26→08-24
    (`docs/dev/reviews/archive/research-vantadb-wasm-20260825.md`) — the monthly
    figure has grown ~4× since; npm counts include CI/mirror traffic, so treat
    them as a trend signal, not a user count.

### 2.4 When to pick what (both directions honest)

- **Pick Orama** if you need full-text + RAG in-memory with no persistence,
  ≤1k docs, mature search-engine features (facets/geosearch/typo tolerance),
  plugin ecosystem, and a 23.8 KB footprint.
- **Pick VantaDB** if any of these matter: OPFS/IndexedDB durable persistence
  across reloads (atomic writes + CRC-32, Web Locks cross-tab, opt-in auto-save;
  snapshot-based, **not** a WAL — `src/storage/engine/init.rs:41-48`), HNSW k-NN
  past linear-scan scale, BM25+vector RRF fusion (k = 60) in one query, graph
  traversal (BFS/DFS/topo) + IQL beside search, TTL auto-expiry — at ~739 KB
  gzip transfer.
- **No performance winner is declared.** Nobody in this niche publishes a
  reproducible public JS/WASM benchmark (research §53; our JS/WASM path has
  zero published numbers, H-11). The qualitative matrix is in
  `docs/dev/reviews/archive/research-vantadb-ts-20260825.md` §3 (verified 2026-08-25).

---

## 3. Niche positioning — "browser AI agent memory"

One-line: **the only embedded JS SDK that gives browser AI agents durable
memory (OPFS + IndexedDB) with native hybrid RRF search and graph traversal** —
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
| 1 | Re-verify §2 numbers (bundlephobia Orama + rebuild `pkg/` + npm downloads both sides) | owner | PARTIAL — npm downloads both sides re-measured 2026-10-04 (§2.1/§2.3); bundle size + `pkg/` re-measure still pending |
| 2 | `PROV-12`: TestPyPI → PyPI green (`pip install` clean-env + smoke) | owner (Gate V: secrets) | blocked on Gate V |
| 3 | `EXE-03`: Fase A kit executed (5 humans, stranger-test) — pass criteria in §5 | owner-side | kit pending |
| 4 | Draft Show HN / launch post from §2-§3 (no new numbers, link BENCHMARKS + §4 wasm README) | owner | TODO — do not post |
| 5 | Draft npm README niche line ("browser AI agent memory") + esm.sh recipe | owner | TODO — do not publish |
| 6 | Post only after 1-3 green; monitor error rate / P95 per plan §2b thresholds · install SLO §5 · opt-in telemetry §6 | owner | paused |

Rollback for a bad announcement: correct the doc (`git revert` + re-publish
patch version note), never edit a published registry artifact in place.

---

## 5. Install SLO — Fase A gate (DEF-08)

> Gate that unpauses the announcement (§4 item 3). Instrument: the 5 tester
> fichas of `docs/dev/FASE-A.md` §1 — human, clock-based, zero new code.
> Anchor: SPEC Success #1 (`SPEC.md`:101) — a recall saved and retrieved by
> synonym in the user's agent **in <30 min, without compiling or cloning**.

**SLO — both must hold:**

| # | Metric | Target |
| :--- | :--- | :--- |
| 1 | Time to first recall (TTFR) | **≤ 30 min** for every passing ficha *(component of "passing"; the operative gate is row 2)* |
| 2 | Install success rate | **≥ 80 % — at least 4 of 5 fichas pass** |

**A ficha passes** when: install completed without owner help (ficha §1 Q1 =
"sí"), first recall succeeded **with the real model** (semantic recall per
`SPEC.md`:101 — a `fallback:true` run is not a passing recall; record the
provider state per §7), and `TTFR ≤ 30 min` (SLO 1). Success rate = passing
fichas / 5. *Scope note: this SLO is the install proxy; semantic quality of the
shipped default is covered by §7 + the FASE-A §2 checklist.*

**Timestamps (measured with a clock, never estimated — `FASE-A.md`:46):**

| Timestamp | Definition |
| :--- | :--- |
| `t0` — install start | the tester begins step 1 of the kit (`FASE-A.md` §3): first install command typed (`pip install vantadb-py` or the one-liner). |
| `t1` — install done | the install path exits 0 and the setup step reports the model verified. Ficha field *"Tiempo hasta instalación"* = `t1 − t0` (diagnostic; no target of its own). |
| `t2` — first recall | first successful retrieval: QUICKSTART §3 `get` prints the stored payload, or §5 prints the three `vector:/text:/hybrid:` hit lines. Ficha field *"Tiempo hasta primer resultado"* = **TTFR** = `t2 − t0`. |

**Measurement protocol:** run the FASE-A kit as written (`FASE-A.md` §3, 5
humans — 2-3 stranger-tests); record `t0`/`t1`/`t2` per ficha with a clock. No
other instrument: network telemetry (§6) is opt-in and is **never** the gate
source.

**GO / NO-GO:** the announcement (§4 item 3, `EXE-03`) resumes only when ≥4/5
fichas pass and the FASE-A §2 checklist stays green (`FASE-A.md`:55 — 0
críticos = no automatic NO-GO). On a failed run: fix the top blockers and
re-run with fresh testers — **the numbers are not relaxed post-hoc**. This is
the install gate; the product-level metric is the North Star (`SPEC.md`
§North Star — DEF-05).

## 6. Telemetry — opt-in, privacy-first (DEF-08 spec)

> **Status: spec only** (design fixed 2026-09-27; implementation is a separate
> follow-up slice — nothing transmits today). Pre-mortem F1: default-on is
> **prohibited**; opt-in only.

**Event set (closed — 3 events):**

| Event | Fires when | Fields (max) |
| :--- | :--- | :--- |
| `install_completed` | install path exits 0 and the setup step verifies the model | `version`, `os`, `arch`, `channel` (`one-liner` \| `pip` \| `npm`) |
| `first_recall` | first successful retrieval in a session | `latency_ms` (rounded), `mode` (`vector` \| `text` \| `hybrid`) |
| `fallback_used` | dummy fallback activates (`fallback:true`) | `reason` (`dylib` \| `model` \| `api`), `surface` (`cli` \| `mcp`) |

**Rules (must hold):**

1. **Default OFF.** A fresh install transmits nothing. Network send requires
   explicit, revocable opt-in consent; the only persisted state is the consent
   flag plus the local JSONL log (rule 2). Local logging is local-only from
   install; nothing leaves the machine until consent.
2. **Local first.** Events append to a local log (JSONL) before anything else;
   the log stays local by default and is deletable with one documented command.
3. **Anonymous.** No stable identifiers — no machine/install/account id. Each
   transmission carries a random per-run nonce, discarded after send, so events
   carry no *client-side* correlation identifiers across runs (receiver-side
   raw connection metadata is prohibited — rule 4).
4. **Never collected (deny list):** memory content/payloads, keys, namespaces,
   query text, prompts, file paths, hostnames/user names, environment
   variables, API keys/tokens, IP-derived location, raw connection metadata
   (IP/port/timing) at the receiver, stack traces.
5. **Never telemetry-only.** `fallback_used` implies a local user-visible
   notice (§7) — the design never allows silence (`SPEC.md`:74).
6. **Review gate.** Before implementation, this design passes the P2-01
   privacy review (telemetry = user data — DEF-08 §SECURITY). The
   implementation spec must additionally pin: consent prompt/revocation
   surface, the one-command local-log deletion, and receiver-side retention.

## 7. Visible fallback — message spec (DEF-08)

Contract: "fallback avisado (`fallback:true`), nunca silencioso" (`SPEC.md`:74).
Existing behavior (`src/llm.rs`:341-419, regression test
`f100_incompatible_dylib_never_panics` at `src/llm.rs`:1087):

- ORT dylib missing/incompatible, API unavailable or session panic → the
  provider keeps running with deterministic dummy embeddings and logs
  `tracing::warn!(fallback = true, ...)` (3 messages, e.g. "ONNX Runtime dylib
  unusable; using deterministic dummy embeddings").
- Model files missing → the factory still succeeds (`from_llm_cfg`,
  `src/llm.rs`:206) and `embed_as` falls back deterministically
  (`src/llm.rs`:526-531) **without any warning** — currently silent.

The gap: the dylib path was log-only and the model-missing path was silent. The
DEF-08 follow-up slice (2026-09-27) reports a user-visible notice covering
**both** triggers on every status surface: `vanta-cli status` (`embedding`
object with `fallback` + `notice`; present in `--json`), the MCP `capabilities`
tool (`embedding` object), and the `setup-embeddings.ps1` live test. The notice
text below is the one those surfaces emit.

**Message requirements** — at the wizard/live test and any surface that loads
local embeddings:

1. State fallback is active in machine-readable form (`fallback: true`) — never silent, for both triggers (dylib/API/panic and model-missing).
2. State the consequence: deterministic dummy embeddings; recall is **not semantic** until restored.
3. State the remedy: re-run the setup wizard (`pwsh setup-embeddings.ps1` — installs native ONNX Runtime ≥1.27 into a persistent store and sets `ORT_DYLIB_PATH` for that session), then verify offline with `python embeddings/verify.py --check` / `python embeddings/download.py --check`.

Draft wording (implementation may adjust while keeping 1-3):

```text
[vantadb] Local embeddings degraded: ONNX Runtime unusable or model missing — using deterministic dummy embeddings (fallback: true). Recall quality is not semantic. Fix: pwsh setup-embeddings.ps1, then verify: python embeddings/verify.py --check
```

User-facing documentation: `docs/user/QUICKSTART.md` §7 "If ONNX Runtime or the
model does not load" — symptoms, both triggers, remediation and verification.

---

## 8. Adoption execution sequence — web surfaces (TS-10 + TS-13, executed 2026-10-04)

> Slices 1 and 3 executed in the web repo (local branches); the rest is
> sequenced with an owner. Web work lives on its own repo/branch — local only,
> nothing pushed from here.

**Gap found (verified 2026-10-04):** the site never mentioned the npm/WASM
install path — `rg "npm install|npmjs" web/src` = **0 hits**; `/docs` §01
Installation covered pip / cargo / CLI binary only, while the playground runs
the real WASM engine. A visitor could execute the engine but not install it.

**Executed slice (web repo, local branch):**

- `web/src/components/vanta/docs-view.tsx` — new **"JavaScript · npm"** install
  card: `npm install vantadb` + raw-bindings note (`npm install vantadb-wasm`)
  + link to the npm registry entry. Branch `ts10/npm-install-card`, commit
  `de5e25c` (worktree `..\web-ts10`, local only — owner merges/cherry-picks).
- Verified: `npx tsc --noEmit` ✅ · `npm run lint` ✅ · `npm run build` ✅
  (Turbopack) · `npx playwright test` ✅ (2 passed) · visual check ✅.

**Executed slice 3 (TS-13, web repo, local branch):**

- **"VantaDB vs Orama"** comparison section on `/why-vantadb`
  (`web/src/components/vanta/orama-comparison.tsx` + page + `oramaCompare.*`
  i18n keys ES/EN): vector index, hybrid fusion, browser persistence, graph +
  IQL, errors, TTL, bundle size. Every Orama cell verified 2026-10-04 against
  the npm registry, docs.orama.com and the `oramasearch/orama` source; every
  VantaDB cell verified against the engine source (see §2.1 for the corrections
  this pass produced). Branch `ts13/orama-column`, commit `fad5bf3` (worktree
  `..\web-ts13`, local only — owner merges/cherry-picks; the owner's redesign
  replaces the page's old competitor table, this section is additive and
  Regla 11-clean by construction).
- Verified: `npx tsc --noEmit` ✅ · `npm run lint` ✅ · `npm run build` ✅
  (Turbopack) · `npx playwright test flujo-critico` ✅ (1 passed) · visual
  check ES + EN ✅.

**Sequence (next slices, in order):**

| # | Slice | Surface | Status / owner |
| :--- | :--- | :--- | :--- |
| 1 | JS/npm install card | docs-site (`/docs`) | ✅ executed — branch `ts10/npm-install-card` (`de5e25c`) |
| 2 | Playground CTA → install + docs (i18n keys) | playground | next slice |
| 3 | Orama comparison in "Why VantaDB" (content §2, cells re-verified 2026-10-04) | web | ✅ executed — branch `ts13/orama-column` (`fad5bf3`), local only; merge/cherry-pick = owner decision |
| 4 | Re-measure bundle (`pkg/` 0.8.0) before quoting §2.2 | web/benchmarks | ✅ measured 2026-10-04 (WSM-14, `vantadb-wasm/README.md` §1) — §2.2 updated; linking it from a web surface still optional before announcement |
| 5 | npm README niche line + demo link | npm package | ✅ executed (WSM-14, commit `0c3e465e`) |

**Metric (declared before promising — Regla 11):** primary = npm weekly
downloads of `vantadb-wasm` + `vantadb`; source
`https://api.npmjs.org/downloads/point/last-week/<pkg>` (public, reproducible,
no instrumentation). Baseline captured 2026-10-04 in §2.3. Site visits are
**not measurable today** — the web repo ships no analytics (verified
2026-10-04); instrumenting the site is an owner decision and is not promised
as a metric. Announcement gate stays §4/§5 (Fase A).
