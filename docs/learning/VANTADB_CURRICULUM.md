# VantaDB → Medium Competence: A Sequenced Curriculum

**Learner:** motivated, 2–4 focused hours/day, wants AI as tutor/reviewer not substitute.
**Goal:** medium competence to work effectively in the VantaDB codebase.
**Horizon:** 6–12 months with AI assistance.
**Hard constraint:** Regla 10 — never merge AI code you can't explain line-by-line. Understanding is the deliverable, not the diff.

This curriculum is grounded in the real repo: docs paths, real source files, real glossary. Every phase ends with a deliverable that *proves understanding*, because with AI you can't fake output — you can only fake understanding, and the portfolio project is what exposes the gap.

---

## The Routing Rule (read this once, it governs everything)

**70% of effort → Rust core + Database internals + Vector search.** That triple IS VantaDB. Everything else (bindings, frontend, DevOps) is 30% — you touch it enough to be competent, not expert.

A medium engineer in this codebase is someone who can:
1. Read a Rust file and trace a mutation end-to-end (WAL → backend → indexes).
2. Explain *why* the write path has that exact order, and what breaks if you change it.
3. Add/tune a component (an index, a backend, a retrieval path) and defend it line-by-line.
4. Find the root cause of a crash or a corrupted-index bug without guessing.

Everything in the curriculum builds toward those four abilities. Anything that doesn't serve them gets deferred.

---

# Phase 0 — Orient & Build (Week 1)

**Goal:** get the thing running, see the whole shape, learn the repo's own glossary as your map.

**What to learn:**
- Follow `docs/QUICKSTART.md` end-to-end: build the CLI, exercise put/get/list, install the Python binding, run every search path (vector / BM25 / hybrid).
- Read `docs/architecture/ARCHITECTURE.md` fully — it is your mental model.
- Read `SKILLS-MANIFEST.md` and `.opencode/AGENTS.md` to know what tooling exists (there's already a `vantadb` skill and a glosario).
- Skim the 57-term `docs/glosario/` — treat it as a domain dictionary. Read deeply only the terms you hit while building.
- Read `docs/tutorials/index.md` — its 4-step path (agent memory → RAG → hybrid → embeddings) is your user-level tour.

**Why (priority):** You can't learn a codebase you've never run. Orientation is cheap and unblocks everything.

**Duration:** ~1 week (10–20 hours).

**Deliverable / portfolio project:** A runnable "small memory demo": a script that puts ~200 mixed records, searches all three ways (vector/BM25/hybrid), exports to JSONL, re-imports, and audits the index. Commit it under your own `experiments/` dir. You should be able to narrate the whole thing from memory in 5 minutes.

**AI integration:** Use AI as *map reader* — paste an architecture section and ask "where does this live in `src/`?" Have it find the actual files for each component in the Component Map (ARCHITECTURE.md lists them: `src/sdk/api.rs`, `src/storage/wal.rs`, `src/text_index.rs`, etc.). **Banned:** asking AI to write your demo script from scratch. You type every line; AI explains what each does.

**Done criteria (medium = …):**
- You can open the repo and point at: the SDK boundary, the storage engine, the WAL, the two indexes, and the backends — without looking.
- You can explain to a stranger why vector + BM25 + RRF is "Hybrid Retrieval v1" and what it is *not* yet.

---

# Phase 1 — Rust Core (Weeks 2–6)

**Goal:** read Rust fluently, write idiomatic Rust, understand the ownership model well enough to trace data flow through `UnifiedNode`.

**What to learn (in order):**
- Ownership, borrowing, lifetimes — but only to the depth the codebase needs. You're not becoming a Rust expert; you need to *read* VantaDB and not be misled by `&`, `&mut`, `Rc`/`Arc`, `Box`, `Vec`, `HashMap`.
- Traits: the codebase is trait-heavy — `StorageBackend`, `EmbeddingProvider`. Learn how to read a trait and its impls (`src/backends/fjall_backend.rs`).
- `Result`/`Option` + `thiserror` (VantaDB's main error crate). Trace error types in `src/error.rs`.
- `serde` + `postcard` serialization (how records become bytes on disk).
- Concurrency primitives you'll actually see: `dashmap`, `parking_lot::RwLock`, `arc-swap`, `rayon`, `lru`.
- Async/tokio: *only if/when you touch the server path* (`vanta-cli server`, `src/cli_server.rs`). The *core is synchronous* — don't deep-dive tokio yet. That's an explicit defer.
- Write a small Rust CLI or library of your own (not touching VantaDB) to practice: read stdin, parse args, do I/O, write a test.

**Why (priority):** Rust is the substrate. Everything else is layers on it. Without reading fluency you cannot satisfy Regla 10.

**Duration:** ~5 weeks (60–100 hours).

**Deliverable / portfolio project:** A standalone Rust tool (e.g. a tiny `watch_dog` that watches a directory and writes a summary) — *your own code, typed by you*. Then the real test: **read** `src/node.rs` (the `UnifiedNode` struct) and write a 1-page explanation of what each field is for and where it's used, citing file:line. This is the first "explain the codebase" deliverable.

**AI integration:**
- **Tutor:** when a borrow error confuses you, paste it and ask "explain this specific lifetime error in the context of this function." Ask for mini-exercises, not the answer.
- **Debugger:** when your own tool won't compile, ask AI to explain the *error*, then fix it yourself. If you can't fix it after understanding, *then* ask for the fix — and read it line-by-line before committing (Regla 10).
- **Banned:** "write this function for me" on any VantaDB file. On *your own* scratch tool, asking for a full function is fine *as long as you then delete it and rewrite it from your understanding*.

**Done criteria (medium = …):**
- You can read any file in `src/` and follow ownership/data flow without getting lost.
- You can explain a trait and its impls, and can name where `StorageBackend` is implemented and why the abstraction exists.
- Your own Rust tool has tests and compiles cleanly.
- You can defend, line-by-line, your explanation of `UnifiedNode`.

---

# Phase 2 — Database Internals (Weeks 6–12)

**Goal:** understand durability and storage from the inside — the WAL, LSM/backend, crash recovery, integrity. **This is the heart of VantaDB's correctness.**

**What to learn (in order):**
- The **write path** by heart: `WAL append → fsync → backend.put → index update → ACK` (ARCHITECTURE.md). Learn *why* this order and what violates it.
- **WAL**: binary layout, record structure, `postcard` payload, per-record CRC32C, sharding (`wal_sharded.rs`), compaction, `checkpoint_seq`.
- **Crash recovery**: the sort-based multi-shard replay, idempotency, skip-via-`checkpoint_seq`. Read `DURABILITY_GUARANTEES.md` completely — it's your spec. *Then* read `src/wal.rs` and `src/wal_sharded.rs` and trace it.
- **LSM-trees**: what Fjall is, why LSM, merge/SST compaction, MVCC, `PersistMode::SyncAll`. Read `docs/glosario/lsm-tree.md`, `fjall.md`, `mvcc.md`.
- **Fjall vs RocksDB**: the `StorageBackend` trait abstracting both. Read `src/backends/fjall_backend.rs` and the redundancy/dedup logic.
- **mmap + SIMD + CRC32C**: memory-mapped vector file (`vfile.rs`), `msync`/RCU rename, integrity checks.
- **Canonical vs Derived** mental model: canonical data (source of truth) vs rebuildable indexes (HNSW, BM25, payload indexes). This design principle explains *everything* about recovery.

**Why (priority):** Medium competence means you can be trusted with data. If you understand the durability path, you understand how the whole product guarantees "no corruption after crash."

**Duration:** ~6 weeks (72–120 hours). This is the single most important technical phase.

**Deliverable / portfolio project:** Two-part.
1. **Write** a tiny "mini-WAL" from scratch (append records with CRC, replay after simulated truncation/corruption) — maybe 200 lines. This is the classic rite of passage and it *forces* the understanding no amount of reading gives.
2. **Read** `tests/storage/crash_injection.rs` and `wal_resilience.rs`. Then write a written report: "what failure modes does VantaDB handle, what does it NOT, and how would I test one of them myself." Refute/confirm with actual code.

**AI integration:**
- **Tutor:** "explain LSM compaction using Fjall's real schema" / "walk me through this crash-recovery test line by line." Ask for Socratic questions.
- **Reviewer (highest value phase for this):** after you write your mini-WAL, have AI review it *for correctness under crash* (fsync ordering, partial-record truncation, CRC check) — not for style. This is where AI review is a genuine second set of eyes, and where Regla 10 bites hardest.
- **Troubleshooter:** reproduce a corruption by deleting a shard mid-write and see what opens; ask AI only to explain the recovery behavior you observe.

**Done criteria (medium = …):**
- You can draw the write path and crash-recovery replay on a whiteboard, including the `checkpoint_seq` skip logic.
- You can explain idempotency and *prove* why replaying a record twice is safe.
- You can distinguish guarantee vs non-guarantee from `DURABILITY_GUARANTEES.md` (e.g. why `SyncMode::Periodic` can lose writes but `Always` can't) without opening the doc.
- Your mini-WAL survives a simulated crash and you can whiteboard why.

---

# Phase 3 — Vector Search (Weeks 12–18)

**Goal:** understand the three retrieval arms — HNSW, BM25, and their RRF fusion — well enough to tune, debug, and extend them.

**What to learn (in order):**
- **Embeddings first (conceptually):** what a vector is, cosine similarity, dimensions, why embedding models map semantics. Read `docs/glosario/vectors.md`, `vector-similarity.md`.
- **HNSW:** the algorithm (multi-layer graph, `M`, `ef_construction`, `ef_search`), how recall/latency trade off, persistence via mmap, the known concurrency issue (AUD-03: rebuild vs lookup). Read the `docs/glosario/hnsw.md` fully — it even has the SIMD and cache-friendly optimizations.
- **BM25:** the inverted index, tokenizer (`lowercase-ascii-alnum`, `docs/architecture/TEXT_INDEX_DESIGN.md`), `k1=1.2`, `b=0.75`, IDF formula, phrase matching over token positions.
- **RRF fusion** (`docs/glosario/rrf.md`): why it ignores raw scores and fuses ranks, `k=60`, the candidate-budget logic from TEXT_INDEX_DESIGN.md.
- **Recall / ANN metrics:** read `docs/glosario/recall.md`, `ann.md`. Learn to *measure*, not just run.

**Why (priority):** Vector search is VantaDB's product identity. Medium competence = you can explain why a recall number is what it is and how to move it.

**Duration:** ~6 weeks (72–120 hours).

**Deliverable / portfolio project:** "Hybrid search tuning lab": take a small real corpus (your notes, docs, anything with 200–1,000 items), embed it, and build a notebook/script that:
1. Runs vector-only, BM25-only, and hybrid (RRF) searches on the same queries.
2. Measures recall@k against a hand-labeled "ideal" set.
3. Varies `ef`, `k`, and candidate budgets; plots recall vs latency.
4. Identifies a case where hybrid *beats both single modes* and one where it *doesn't*, and explains why (whiteboard the RRF math for it).

This is the strongest portfolio piece of the whole curriculum because it demonstrates *judgment*, not just tool use.

**AI integration:**
- **Tutor:** "explain HNSW layer selection with the actual `HnswIndex` struct" and "why does higher M raise memory but improve recall."
- **Debugger:** when a search returns garbage, use AI to brainstorm hypotheses (dimension mismatch, index not rebuilt, mmap stale) but **investigate yourself** before confirming.
- **Banned:** letting AI write the whole tuning lab. You write the measurement loop; AI can suggest *what metrics to measure*.

**Done criteria (medium = …):**
- You can explain HNSW search top-to-bottom and the AUD-03 concurrency risk from memory.
- You can compute an RRF score by hand for two rankings.
- You can articulate the exact trade-off between `SyncMode`/`ef`/`k` and why they're separate knobs.
- Your tuning lab shows a real per-query recall/latency curve and you defend every line.

---

# Phase 4 — ML/AI Infrastructure (Weeks 18–22)

**Goal:** understand the embedding and RAG story — especially local ONNX inference — since it's VantaDB's differentiator (local-first, no external LLM required).

**What to learn (in order):**
- **Embedding providers:** the `EmbeddingProvider` trait, BYO-vector model, OpenAI/Ollama/LiteLLM, and the *local-first* path via `ort` (ONNX Runtime) + `tokenizers`. Read `docs/api/EMBEDDINGS.md` and `docs/tutorials/05-embedding-integrations.md`, `embed-local` in QUICKSTART.md.
- **ONNX local inference:** what ONNX is, why local, the `multilingual-e5-small` 384d model, `EmbeddingProvider::embed_batch`, the one-model-per-namespace rule.
- **RAG:** retrieval-augmented generation patterns (chunking, embedding, retrieval, generation). Read `docs/glosario/rag.md`, `graphrag.md` (skim — graph RAG is beyond medium), `docs/tutorials/02-local-rag-pipeline.md`.
- **MCP (skim):** how VantaDB exposes an agent protocol. Good to know, low priority to master.

**Why (priority):** *Moderate* — this is where the "AI agent memory" and RAG story lives, so you need functional competence, but you don't need to build a model.

**Duration:** ~4 weeks (48–80 hours).

**Deliverable / portfolio project:** A fully **offline** RAG pipeline: download the local ONNX model, embed a document set, store in VantaDB, and run hybrid retrieval where the "generation" step uses a local model (or a mocked one). Ship a `README` explaining the data flow at each layer, and be able to hot-swap the embedding provider and explain what changed.

**AI integration:**
- **Tutor:** "explain the difference between the ONNX path and a remote Ollama provider in the `EmbeddingProvider` trait."
- **Integration-guide / implementer-behind-your-hands:** ONNX + tokenizers setup has fiddly system deps — it is legitimate to let AI generate the *build* glue (e.g. exact `cargo`/`ort` config) because that's configuration, not understanding. But you author the RAG *logic*.

**Done criteria (medium = …):**
- You can draw the pipeline: documents → chunk → embed → store → search → generate, and point each to a file.
- You can explain BYO-vector (what the user supplies vs what VantaDB does) and the `embed-local` path.
- Your offline RAG demo runs with zero network.

---

# Phase 5 — Bindings (Weeks 22–26) [30% bucket]

**Goal:** competent, not expert, in the multi-language surface: PyO3 (main), WASM, napi-rs.

**What to learn (in order):**
- **PyO3** (highest priority of this bucket): how `vantadb-python` wraps the Rust `VantaEmbedded` SDK boundary — reads/writes Python objects to Rust types, GIL implications. Read `docs/glosario/pyo3.md`, `gil.md`, `python-sdk.md`, `ffi.md`.
- **WASM** (skim): how core compiles to `wasm32-wasip1`, what's stubbed (`memmap2`→Vec shim, `rayon`→sequential, `sysinfo`→stub). You don't need to build for WASM; you need to know the strategy.
- **napi-rs** (awareness only): it's listed but for TS binding — read enough to know what it is, don't learn to build it.

**Why (priority):** Medium competence *in VantaDB* means the Python binding is where most users live, so you should be able to trace "Python call → PyO3 → Rust core." WASM/napi are YAGNI for medium unless you'll specifically work on the TS SDK.

**Duration:** ~4 weeks (48–80 hours) — or defer WASM/napi entirely if your work is Python/Rust-centered.

**Deliverable / portfolio project:** Add **one** small PyO3-exposed function to a scratch binding (not necessarily merged): e.g. a wrapper that calls into the core SDK and returns a typed result. Simpler acceptance: write a Python test that exercises the full surface (put/get/search/export/import/audit) and document the Rust↔Python type mapping for each.

**AI integration:**
- **Tutor:** "explain why this Python list becomes a `Vec<f32>` across the PyO3 boundary, and where the GIL is held."
- **Reviewer:** have AI review your PyO3 wrapper for GIL/panic-correctness (pyo3 has sharp edges — `panics` crossing the boundary abort the process). This is a legit safety review.

**Done criteria (medium = …):**
- You can trace a `search_memory()` call from Python through PyO3 to the Rust SDK boundary and back, naming the types at each hop.
- You can explain why the SDK boundary exists (to keep consumers off `StorageEngine`/`Executor`/direct HNSW locks).
- You can state what WASM stubs are in place by design (bonus, not required).

---

# Phase 6 — Web Frontend (Weeks 26–28) [lowest priority]

**Goal:** enough to navigate and make small changes; nowhere near a frontend expert.

**What to learn (skim):** the Next.js/React/Tailwind structure of `web/`, how it calls the backend (HTTP server wrapper), basic components.

**Why (priority):** Lowest. The core value is the engine, not the UI. Only invest here if you'll actually touch `web/`.

**Duration:** ~2 weeks (or fully skip). **This phase is the #1 candidate for YAGNI-out.**

**Deliverable:** Skip, OR a one-page component that renders hybrid search results. Trivial.

**AI integration:** Frontend is where AI as *co-writer* is most legitimate — the barrier to value is low and the stakes are low. But if your goal is DB medium competence, don't spend your scarce hours here.

**Done criteria (medium = …):** you can find and edit a component and understand how it hits the API. That's it. If that isn't part of your job, defer forever.

---

# Phase 7 — DevOps/CI (Weeks 28–30, ongoing) [30% bucket]

**Goal:** not an SRE, but understand how the repo ships — because that's how you get your portfolio merged and how tests protect correctness.

**What to learn:**
- **GitHub Actions** workflows (build, test, Python Wheels, releases). Read `docs/glosario/ci-cd.md`.
- **Releases & benchmarks:** how releases are cut (tag-gated, TestPyPI→PyPI), Sigstore signing, `docs/glosario/sigstore.md`, `slsa.md`, `benchmarks.md`.
- **Testing culture:** the repo has crash-injection, chaos/failpoints, WAL-resilience tests. Learn to *run* them locally and read them (you already did in Phase 2).

**Why (priority):** Cold, but *how tests gate your portfolio* matters for shipping and for Regla 10.

**Duration:** ~2 weeks focused, then ongoing (every merge is practice).

**Deliverable:** Your portfolio project from a previous phase passes CI and you can trigger a manual build; you can explain the wheel-publish path. Ideally: one merged PR (even a doc or test fix you can defend line-by-line).

**AI integration:**
- **Debugger/automator:** CI failures are ideal for AI + you: AI interprets the confusing CI log, you find the root cause in your code.
- **Banned:** letting AI author a release or mutate workflows blindly — those touch governance.

**Done criteria (medium = …):** you can read a workflow file and state what each job gates; you can diagnose a failed local `cargo test` from the log; one of your changes is in CI green.

---

# What to Skip / Defer (YAGNI for the curriculum itself)

Ponytail applied to learning: don't learn what you only need when you need it. Defer until it's actually on your plate:

- **tokio/async in depth** — the core is synchronous. Only learn it when you touch `vanta-cli server`.
- **napi-rs / TS SDK / WASM building** — unless you own that SDK. Know the *strategy*, not the code.
- **Graph RAG / multi-hop graph traversal** — beyond medium; the product boundary says graph is not primary yet.
- **IQL/LISP/DQL, MCP deep, enterprise features, cloud, plugins** — explicitly outside current MVP boundary (QUICKSTART).
- **Full web frontend mastery** — Phase 6 is the first thing to drop under time pressure.
- **Deep Rust expertise** (traits beyond reading, macros, unsafe, async runtime internals) — you need *reading* fluency, not *authoring* fluency, to hit medium.
- **RocksDB backend internals** — you need to understand the *trait* and the trade-off, not the C++ LSM. That's Fjall's job in production.

Rule of thumb: if you can't tie a topic to a file you'd plausibly touch in the next 90 days, it's deferred.

---

# The AI Integration Playbook (per phase, summarized)

| Phase | AI role that's LEGITIMATE | AI role that VIOLATES your learning |
|-------|---------------------------|-------------------------------------|
| 0 Orient | Map reader: "where does this live" | Writing your demo script from scratch |
| 1 Rust | Tutor on errors; Socratic mini-exercises | Writing VantaDB functions for you |
| 2 Internals | Reviewer of your mini-WAL for crash-correctness | Explaining durability instead of you reading the doc |
| 3 Vector | Tutor on HNSW/RRF internals; metric suggestions | Authoring the whole tuning lab |
| 4 ML/AI | Builder of build-glue (ONNX config) | Writing your RAG logic |
| 5 Bindings | GIL/panic-safety reviewer | Writing binding logic unread |
| 6 Frontend | Full co-writer (low stakes, low value) | Spending scarce hours here at all |
| 7 CI | CI-log interpreter | Authoring/mutating workflows |

**Universal rules:**
1. **Explain-before-you-merge** (Regla 10): any AI code you keep, you can line-by-line explain, or it doesn't merge.
2. **Ask in the tutor voice first:** "explain", "why", "walk me through" — not "do it for me." Escalate to "do it" only after you can't, then read it fully.
3. **Prove with your own hands:** every phase has a manual deliverable. If AI could have done the deliverable, the deliverable isn't proving *you*.
4. **Use AI to compress, not bypass:** compressing docs into study maps and flashcards is great use. Having AI "teach you the summary" of a chapter you never read is not learning.

---

# Time-Budget Snapshot

| Phase | Focus | ~Weeks | ~Hours @2.5h/day |
|-------|-------|--------|------------------|
| 0 Orient | Core 70% | 1 | 15 |
| 1 Rust Core | Core 70% | 5 | 90 |
| 2 DB Internals | Core 70% | 6 | 110 |
| 3 Vector Search | Core 70% | 6 | 110 |
| 4 ML/AI Infra | 30% bucket* | 4 | 60 |
| 5 Bindings | 30% bucket | 4 | 60 |
| 6 Web Frontend | 30% bucket (drop first) | 2 | 30 |
| 7 DevOps/CI | 30% bucket, ongoing | 2+ | 30 |
| **Total** | | **~30** | **~505** |

\* ML/AI sits half-core-half-bucket because local inference is a differentiator; treat it as high-value 30%.

At 2–4h/day this lands inside 6–12 months with room for the ongoing Phase 7 work and re-explanation time (which AI compresses).

---

## One-paragraph summary

Build first, Learn Rust to *read* the core, master the durability/storage path (that's where medium competence lives), then the retrieval trio (HNSW/BM25/RRF), then the AI-embedding story — and treat bindings, frontend, and DevOps as lightweight competence you earn by shipping one defended change. Every phase ends in a portfolio piece you authored, and AI's only job is to be your tutor, crash-correctness reviewer, and log interpreter — never your substitute. Regla 10 isn't a rule to obey; it's the definition of the competence you're building.
