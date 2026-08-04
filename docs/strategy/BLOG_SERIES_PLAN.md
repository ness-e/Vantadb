---
title: Blog Series Completion Plan
type: strategy
status: active
tags: [vantadb, marketing, blog, content, launch, hn, seo]
last_reviewed: 2026-08-02
aliases: [BLOG_SERIES_PLAN, Blog Plan, Blog Calendar, Blog Series]
---

# Blog Series Completion Plan

> **Domain:** Marketing & Content
> **Purpose:** Inventory the existing blog series, review draft quality, define audience and keywords, and propose a publication calendar (Show HN + blog cadence) to take the series from 3 drafts to a complete, launched set.
> **Scope:** Planning only. No new content is written and no code is changed by this document. Tracking task: INV-006.

---

## Status Summary

The blog series is **4 of 5 posts complete** as drafts in `docs/blog/`, but the production site exposes **4 posts** (one of them, `introducing-vantadb`, has no source draft in `docs/blog/`). Every live post has metadata drift between the web manifest and its markdown draft. See [Section 1](#1-content-inventory).

| Surface | Posts | Notes |
|---------|-------|-------|
| Web (`BLOG_POSTS` in `web/src/components/vanta/vanta-data.ts`) | 4 | Full inline content, published dates Jan–Feb 2025, author `ness-e` |
| Drafts (`docs/blog/`) | 5 | Real content (not stubs), all dated 2026-06-06, author `VantaDB Team` |
| Backlog / MKT-05 claim | 4/5 | Audit `backlog-validation-2026-07-28` corrected the count to 3 drafts; MKT-05 added the 5th (benchmarks) draft on 2026-08-04 |

---

## 1. Content Inventory

### 1.1 Live posts (web)

Defined in `web/src/components/vanta/vanta-data.ts` (line 833, `BLOG_POSTS`). All four posts carry full content inline; the site renders `/blog` listing and `/blog/[slug]` detail pages.

| # | Slug | Title (web) | Date (web) | Author | Read time | Tag |
|---|------|-------------|-----------|--------|-----------|-----|
| 1 | `introducing-vantadb` | Introducing VantaDB | 2025-01-15 | ness-e | 6 min | Announcement |
| 2 | `how-hybrid-search-works` | How Hybrid Search Actually Works | 2025-01-22 | ness-e | 9 min | Engineering |
| 3 | `sqlite-for-ai-agents` | SQLite for AI Agents: The Missing Memory Layer | 2025-02-05 | ness-e | 7 min | Architecture |
| 4 | `why-i-built-vantadb-local-memory-engine` | Why I Built VantaDB: A Local Memory Engine | 2025-02-15 | ness-e | 5 min | Story |

### 1.2 Drafts (docs/blog)

| # | File | Title (draft) | Date | Author | Size | Lines |
|---|------|---------------|------|--------|------|-------|
| 1 | `docs/blog/how_hybrid_search_works.md` | How Hybrid Search Works: BM25 + HNSW + RRF in Practice | 2026-06-06 | VantaDB Team | 8,540 B | 144 |
| 2 | `docs/blog/sqlite_for_ai_agents.md` | SQLite for AI Agents: Benchmarks and Architecture Decisions | 2026-06-06 | VantaDB Team | 7,339 B | 105 |
| 3 | `docs/blog/why_i_built.md` | Why I Built a Local Memory Engine for AI Agents in Rust | 2026-06-06 | VantaDB Team | 8,477 B | 92 |
| 4 | `docs/blog/introducing_vantadb.md` | Introducing VantaDB | 2026-06-06 | VantaDB Team | 5,960 B | 37 |
| 5 | `docs/blog/benchmarks_vs_lancedb_chroma.md` | VantaDB vs LanceDB vs ChromaDB: Real Numbers from an Embedded Engine | 2026-06-06 | VantaDB Team | 12,586 B | 130 |

> **M1 status update (2026-08-04):** `docs/blog/introducing_vantadb.md` now exists as an editable source draft, resolving the original M1 gap — every live post has a `.md` source.

### 1.3 Planned (backlog / GTM)

The original backlog planned 3 articles: *Why I Built a Local Memory Engine in Rust*, *How Hybrid Search Works*, *SQLite for AI Agents*. All three exist as drafts. `GO_TO_MARKET.md` additionally lists topics that are **not** yet drafted (GraphRAG, WAL/durability lessons, Ollama + VantaDB tutorial, Claude Code memory). The benchmarks post (VantaDB vs LanceDB vs ChromaDB) was planned in [Section 4.3](#43-missing-posts-and-proposed-order) and is now drafted (MKT-05, 2026-08-04).

### 1.4 Mismatch summary

| # | Gap | Severity | Detail |
|---|-----|----------|--------|
| M1 | **`introducing-vantadb` has no source draft** | High | The post is live on the web (4th entry, with full content) but has no `.md` in `docs/blog/`. It is the only post without an editable source. The 2026-07-28 audit already flagged that MKT-05's "4/5" inflated the count. |
| M2 | **Author drift** | Medium | Web lists author `ness-e`; all drafts list `VantaDB Team`. One voice should win (recommendation: `ness-e` for the story post, `VantaDB Team` for engineering posts). |
| M3 | **Date drift** | Medium | Web dates are Jan–Feb 2025; drafts are all 2026-06-06 (draft-creation date, not publish date). Dates will need reconciliation before any post is published. |
| M4 | **Title drift** | Medium | Same topic, different titles: web `How Hybrid Search Actually Works` vs draft `How Hybrid Search Works: BM25 + HNSW + RRF in Practice`; web `SQLite for AI Agents: The Missing Memory Layer` vs draft `...: Benchmarks and Architecture Decisions`; web `Why I Built VantaDB: A Local Memory Engine` vs draft `Why I Built a Local Memory Engine for AI Agents in Rust`. |
| M5 | **Slug convention drift** | Low | Draft files use underscores (`how_hybrid_search_works.md`); web slugs use hyphens (`how-hybrid-search-works`). No automation maps one to the other. |
| M6 | **Version drift in copy** | High | `why_i_built.md` states "currently at version `0.1.4`"; `SHOW_HN_PREP.md` states `0.4.0`; `COMPANY_INFO` in `vanta-data.ts` states `v0.1.2`; `GO_TO_MARKET.md` states `v0.1.4`. The correct current version must be confirmed and standardized before any post ships. |

---

## 2. Draft Review & Quality Assessment

### 2.1 Per-post assessment

| File | Length | Structure | Content quality | CTA | Verdict |
|------|--------|-----------|-----------------|-----|---------|
| `how_hybrid_search_works.md` | 144 lines — solid deep-dive | 5 numbered sections + conclusion; LaTeX formulas, ASCII operator tree, Rust snippet | Strong: real architecture (BM25-in-LSM, SIMD HNSW, Volcano CBO, RRF) with `$<10\%$` selectivity heuristics | Weak: single GitHub link in conclusion ("visit GitHub") | **Ready to publish** after CTA + metadata fix |
| `sqlite_for_ai_agents.md` | 105 lines — tight | 4 sections + conclusion; ASCII write-path diagram, benchmark numbers | Strong: concrete benchmarks (59% fewer major page faults, 750→1,195 QPS, 4.01x batch speedup, 2.43ms latency) | Weak: single GitHub link | **Ready to publish** after CTA + metadata fix |
| `why_i_built.md` | 92 lines — narrative | Motivation + 4 constraints + landscape + architecture + conclusion | Strong: first-person motivation, competitor critique (cloud vector DBs, in-memory indexers, sqlite-vss) | Best of the three: explicit `pip install vantadb-py` call-to-action | **Ready to publish** after version fix (M6) + metadata |

### 2.2 Frontmatter / metadata gaps

All three drafts share a consistent frontmatter (`title`, `date`, `author`, `tags`, `description`) but are **missing** fields the web manifest has:

- `slug` — needed to map `docs/blog/*.md` to web routes deterministically (fixes M5).
- `readTime` — web shows 5–9 min per post.
- `tag` / `tagColor` — web uses Announcement/Engineering/Architecture/Story.
- `published` (boolean or `draft: false`) — the series needs an explicit published/draft flag so drafts are not mistaken for live content.
- `canonical` URL — optional, but recommended once `vantadb.dev/blog/<slug>` exists.

### 2.3 CTA analysis

- **Good:** `why_i_built.md` ends with an action (`run pip install vantadb-py`).
- **Weak:** `how_hybrid_search_works.md` and `sqlite_for_ai_agents.md` end with a bare repo link.
- **Recommendation (applies to all posts):** close with a 2–3 sentence CTA block — primary CTA (try it: `pip install vantadb-py`), secondary CTA (join Discord / star the repo), plus a related post link. Keep the CTA consistent across the series.

### 2.4 Blocking issues before any publication

1. **Version number (M6):** confirm the current release and update all posts + `SHOW_HN_PREP.md` + `COMPANY_INFO` to one number.
2. **Date + author reconciliation (M2/M3):** pick final dates and author attribution per post; align web manifest with the draft source.
3. **`introducing-vantadb` source (M1):** create the missing `docs/blog/introducing_vantadb.md` from the web content so the series has a single editable source of truth.
4. **`why_i_built.md` heading/title:** the draft title ("Why I Built a Local Memory Engine...") reads more "team" than "I" — decide first-person vs team voice for the series.

---

## 3. Target Audience & Keyword Research

### 3.1 Audience segments

Derived from `GO_TO_MARKET.md` verticals and the existing posts' framing:

| Segment | Profile | What they search for | Post fit |
|---------|---------|----------------------|----------|
| **AI agent builders** | LangGraph/CrewAI/Pydantic AI devs needing persistent, crash-safe cyclic memory | "agent memory", "persistent memory for AI agents", "local LLM memory" | `sqlite_for_ai_agents.md`, `why_i_built.md` |
| **RAG pipeline devs** | Builders doing retrieval-augmented generation, often local-first | "hybrid search", "BM25 + HNSW", "RRF", "RAG without cloud", "embedding database" | `how_hybrid_search_works.md`, `sqlite_for_ai_agents.md` |
| **Rust / embedded DB enthusiasts** | Engineers evaluating or building embedded storage in Rust | "Rust vector database", "embedded database Rust", "HNSW Rust", "WASM vector search" | `how_hybrid_search_works.md`, `why_i_built.md` |
| **Local LLM stack users** | Ollama/AnythingLLM users wanting zero-server memory | "Ollama memory", "local agent memory", "offline RAG" | `why_i_built.md` (future tutorial) |
| **AI-IDE tooling devs** | Claude Code/Cursor/OpenCode users losing context between sessions | "Claude Code memory", "MCP memory server", "project memory" | Future MCP post |

### 3.2 Keyword clusters

Validated via web search (2026-08-02; hybrid-search/RAG guides from 2026 confirm the cluster is active, and npm/crates competition confirms the "agent memory" space is hot):

| Cluster | Keywords / phrases | Intent | Priority |
|---------|--------------------|--------|----------|
| **Core hybrid retrieval** | hybrid search, BM25 + HNSW, reciprocal rank fusion (RRF), vector + keyword fusion | Transactional (how-to) | High |
| **Embedded vector DB** | embedded vector database, local vector database, vector database no server | Informational → transactional | High |
| **Local-first AI memory** | local-first AI, agent memory, persistent memory for AI agents, long-term memory LLM | Informational | High |
| **Comparative** | SQLite for AI agents, SQLite vs LSM tree, vector database comparison, Pinecone vs local | Commercial investigation | Medium |
| **Rust/embedded** | Rust vector database, embedded database Rust, HNSW memory mapping, mmap graph layout | Informational | Medium |
| **Edge/offline** | offline RAG, RAG without cloud, edge AI memory, Ollama memory | Informational | Medium |

**Differentiation angle to keep repeating:** zero-dependency (`pip install vantadb-py`, no C++ toolchain), single-engine hybrid retrieval with a real query planner, and WAL crash-safety — versus cloud vector DBs, sqlite-vss/vec, and in-memory indexers. Competitors observed in the same keyword space: Orama, ruvector, retriv, kura, enquire-mcp.

**Caveat:** search-volume numbers were not pulled (no keyword planner access in this run). Before spending effort on titles, validate the top 5 long-tail phrases with an actual volume tool; the clusters above are based on intent + 2026 content landscape, not volume data.

### 3.3 Post → keyword mapping (proposed)

| Post | Primary keywords | Secondary |
|------|------------------|-----------|
| How Hybrid Search Works | hybrid search, BM25 + HNSW, RRF | RAG pipeline, vector fusion |
| SQLite for AI Agents | SQLite for AI agents, LSM tree, embedded database | agent memory, write throughput |
| Why I Built | local-first AI, agent memory, Rust embedded database | local LLM memory, offline RAG |
| Introducing VantaDB (missing source) | embedded vector database, local-first, zero-dependency | vector database comparison |

---

## 4. Publication Calendar & Editorial Proposal

### 4.1 Show HN launch

The Show HN draft already exists and is **not duplicated here**: see [SHOW_HN_PREP.md](SHOW_HN_PREP.md) (status `active`, last reviewed 2026-07-27) for the full post draft and the 10-item defensive Q&A matrix.

Sequence recommendation:

1. **Before Show HN (blocking):** resolve M6 (version number) and confirm the launch version + wheels are published on PyPI/crates.io/npm. The Show HN draft references features (SIMD, BFS layout, RRF, GIL release) that all three blog drafts corroborate — good consistency.
2. **Launch day:** Show HN post goes live; `introducing-vantadb` goes live on the blog the same day (it is the "why this exists" anchor that the HN thread can link to).
3. **48h after launch:** publish `why_i_built.md` (story post) to capitalize on HN attention; it is the most shareable, personal post.
4. **Defensive follow-up:** use the Q&A matrix from `SHOW_HN_PREP.md` to seed blog replies and/or a follow-up comment.

### 4.2 Blog cadence

Target from `GO_TO_MARKET.md`: **2 posts/month**, with 6/12/24 posts at 3/6/12 months. That cadence fits an indie/1-dev team and keeps the series alive without exhausting the topic backlog.

Suggested rhythm:

- **Week 1 of month:** technical/architecture post.
- **Week 3 of month:** tutorial or case study (integration, benchmark, MCP).
- One of the two monthly posts should map to a keyword cluster from [Section 3.2](#32-keyword-clusters).

### 4.3 Missing posts and proposed order

| Order | Post | Source | Status | Proposed window | Aligned with |
|-------|------|--------|--------|-----------------|--------------|
| 1 | Introducing VantaDB | web inline content only | **Needs `docs/blog/introducing_vantadb.md`** | Launch day (with Show HN) | Show HN launch |
| 2 | Why I Built a Local Memory Engine in Rust | `docs/blog/why_i_built.md` | Ready (fix M6 first) | Launch +48h | HN attention window |
| 3 | How Hybrid Search Works: BM25 + HNSW + RRF | `docs/blog/how_hybrid_search_works.md` | Ready (CTA fix) | Launch +1 week | Post-launch |
| 4 | SQLite for AI Agents: Benchmarks and Architecture Decisions | `docs/blog/sqlite_for_ai_agents.md` | Ready (CTA fix) | Launch +3 weeks | Post-launch |
| 5 | GraphRAG with VantaDB — reducing tokens 40–60% | not drafted | Plan | Month 2 | GTM agentic-frameworks vertical |
| 6 | Local agent memory with Ollama + VantaDB | not drafted | Plan | Month 2 | GTM local-LLM vertical |
| 7 | VantaDB as persistent memory for Claude Code (MCP) | not drafted | Plan | Month 3 | GTM AI-IDE vertical |
| 8 | WAL & durability: lessons from chaos testing | not drafted | Plan | Month 3 | Release with durability feature |
| 9 | VantaDB vs LanceDB vs ChromaDB (benchmarks) | `docs/blog/benchmarks_vs_lancedb_chroma.md` | **Drafted (MKT-05, 2026-08-04)** — real run of `benchmarks/competitive_bench.py` (glove-100-angular 10K, median-of-3, chunked ingest) | Drafted; publish with benchmark data | Release with benchmark data |

### 4.4 Release alignment

- **Every minor release** (e.g., 0.4.x → 0.5.0) should ship with at least one blog post that highlights the release's headline feature (WAL durability, new storage backend, MCP server, etc.).
- **Do not publish version-specific copy** (like the current `0.1.4` in `why_i_built.md`) until the version is fixed once in one place; consider phrasing that survives version bumps ("the current release", "v0.4+").
- Tie tutorial posts to integration releases (LangChain/LlamaIndex adapters on PyPI, MCP server stabilization) so each integration launch has a discoverable post.

### 4.5 Content pipeline (process recommendation)

To prevent recurrence of M1–M6:

1. **Single source of truth:** `docs/blog/` markdown is canonical; the web manifest is generated/mirrored from it. Create the missing `introducing_vantadb` draft to close M1.
2. **Frontmatter contract:** every post must include `slug`, `title`, `date`, `author`, `readTime`, `tag`, `description`, `draft` (true/false). Add this contract to the writing checklist.
3. **Publish gate:** before marking a post `draft: false`, run: version check (M6), CTA present, keywords mapped ([Section 3.3](#33-post--keyword-mapping-proposed)), `docs/blog` and web manifest in sync.
4. **Monthly cadence review:** at the start of each month, pick the two posts from [Section 4.3](#43-missing-posts-and-proposed-order) and assign them.

---

## See Also

- [SHOW_HN_PREP.md](SHOW_HN_PREP.md) — Show HN draft + defensive Q&A (the launch artifact)
- [GO_TO_MARKET.md](GO_TO_MARKET.md) — distribution strategy, verticals, content marketing targets
- [ROADMAP.md](ROADMAP.md) — technical timeline for release alignment
- `docs/blog/` — the three drafts reviewed in [Section 2](#2-draft-review--quality-assessment)
- Backlog task INV-006 — tracking task for this plan
