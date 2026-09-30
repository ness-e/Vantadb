---
title: "VantaDB vs Alternatives — Honest Comparison & Practical Limits"
kind: howto
status: active
description: "This page answers \"why not X?\" (sqlite-vec, LanceDB, Qdrant, Chroma) with three rules"
tags: [vantadb, comparison, benchmarks, limits]
---

# VantaDB vs Alternatives — Honest Comparison & Practical Limits

This page answers "why not X?" (sqlite-vec, LanceDB, Qdrant, Chroma) with three rules:

1. **Qualitative features** are only stated when publicly verifiable from each project's official repository or documentation (links inline; all checked 2026-08-24).
2. **Numbers about us** come exclusively from [`docs/user/operations/BENCHMARKS.md`](operations/BENCHMARKS.md), each with its bench script and exact reproduction command (Regla 11).
3. **We publish no performance figures for competitors.** Where a vendor publishes their own benchmarks, we link them; you judge. We also link the neutral third-party [ann-benchmarks](https://ann-benchmarks.com/) results and provide the script to run the comparison yourself.

> **Capa pendiente (2026-09-24):** esta página cubre vector DBs embebidos/servidores. Falta la capa **memory-as-a-service** (Mem0/Zep/Letta/LangMem/Cognee/MemOS/MIRIX/ReMe/OpenMemory/OpenMemory MCP) que es el wallet de memoria real — plan P54 (ICP-01..03) + VER-09 (head-to-head con protocolo propio, pares accuracy+tokens). No interpretar esta página como comparativa completa del nicho hasta entonces.

---

## 1. Qualitative Comparison

| | **VantaDB** | **sqlite-vec** | **LanceDB** | **Qdrant** | **Chroma** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Deployment model** | Embedded Rust core in-process ([`Embedded`](../api/EMBEDDED_SDK.md)) behind Python / TypeScript-WASM / Node bindings; optional HTTP server ([HTTP API](../api/HTTP_API.md)) and MCP server | SQLite extension loaded into any process that runs SQLite (embedded) | Open-source embedded library ("runs locally"); hosted Cloud/Enterprise offerings exist | Server-first (self-hosted binary / Docker / managed cloud). A **local mode** exists inside the Python client for development and prototyping — same API without running the server | In-memory or persistent client running embedded in your process; client/server mode also supported |
| **Primary language** | Rust (PyO3, WASM, NAPI bindings) | Pure C, zero dependencies | Rust core; Python / TypeScript / Rust SDKs | Rust (server); clients in many languages | Rust core; Python / JS-TS SDKs |
| **Persistence** | WAL + pluggable storage backends (Fjall default, RocksDB opt-in, in-memory) — [DURABILITY_GUARANTEES](operations/DURABILITY_GUARANTEES.md) | Stored inside the SQLite database file | Lance columnar format (local disk or object storage) | Server-managed persistence; client local mode persists to a path | Persistent client or server storage; in-memory mode is ephemeral |
| **Keyword + vector hybrid fusion** | Built-in: BM25 lexical + HNSW dense fused with RRF in one query API ([how it works](blog/how_hybrid_search_works.md)) | Not part of sqlite-vec — it provides vector search in `vec0` virtual tables only; combining with SQLite FTS5 is your job | Documented hybrid search combining vector + full-text search with reranking | Documented Query API fusing dense + sparse results with RRF or DBSF | Vector search documented; full-text and regex search advertised on trychroma.com. A BM25-style score-fusion API is not documented in the open-source docs we reviewed |
| **Graph capabilities** | Typed nodes + edges, BFS/DFS traversal, topological sort, DAG check, PageRank / degree centrality, GraphRAG pipeline ([GRAPH_RAG](../api/GRAPH_RAG.md)) | None documented | None documented | None documented | None documented |
| **License** | Apache-2.0 core (open-core: commercial Pro is a separate closed offering) | Apache-2.0 | Apache-2.0 | Apache-2.0 | Apache-2.0 |

**Sources for competitor cells** (official repositories and documentation):

- sqlite-vec: <https://github.com/asg017/sqlite-vec> — "Written in pure C, no dependencies, runs anywhere SQLite runs"; features limited to vector storage/query in `vec0` virtual tables; Apache-2.0 per repository license metadata.
- LanceDB: <https://github.com/lancedb/lancedb> (description, products, SDKs, Apache-2.0) and hybrid search docs <https://docs.lancedb.com/search/hybrid-search>.
- Qdrant: <https://github.com/qdrant/qdrant> (Apache-2.0, Rust) · local mode: <https://github.com/qdrant/qdrant-client> ("Local mode - use same API without running server", positioned for development/prototyping/testing) · hybrid fusion: <https://qdrant.tech/documentation/concepts/hybrid-queries/>.
- Chroma: modes — <https://docs.trychroma.com/docs/overview/getting-started> (in-memory vs persistent vs client-server) and <https://docs.trychroma.com/docs/run-chroma/client-server>; repository/license — <https://github.com/chroma-core/chroma>.

> The "None documented" cells mean the project's official feature lists and docs (as of 2026-08-24) do not advertise graph traversal — absence of evidence, stated as such.

---

## 2. Our Numbers (reproduced from BENCHMARKS.md)

Every figure below is copied from [`docs/user/operations/BENCHMARKS.md`](operations/BENCHMARKS.md), which is the single canonical source for benchmark numbers. **They are measurements of specific runs on specific machines — mostly local Windows hardware — not universal guarantees.** Reproduce them with the cited commands before drawing conclusions for your workload.

> **Removed (2026-09-29):** this page previously carried a "Canonical P99 baseline"
> table (insert 322.59 s, p50/p95/p99 = 1.4786 / 2.3708 / 3.0746 ms) and a second
> SDK table (PUT 13.174 ms / 74 ops/s, index rebuild 5.01 s, vector 2.024 ms,
> hybrid 3.114 ms). Neither set appears in `BENCHMARKS.md`, so both violated the
> canonical-sources rule; the numbers survive only in `docs/dev/tasks/FND-10.md`
> and `FND-13-F1.md`. Nothing is cited here rather than cite a source that does
> not contain it. `BENCHMARKS.md §11` records the `canonical_p99` p50 as
> 57 ms @10k for the anti-regression policy, and the current `vantadb_local_bench`
> run is explicitly **FROZEN pre-SIMD pending regeneration** (§2) — regenerate it
> before quoting any Python-SDK per-op number.

### 2.1 Core engine certification (pure Rust, 10K–100K scale)

From [BENCHMARKS.md §1](operations/BENCHMARKS.md#-1-core-engine-certification-results-rust), produced by [`tests/certification/stress_protocol.rs`](../../tests/certification/stress_protocol.rs) under the heavy-certification CI workflow (AVX2 environment):

| Metric | Scale / Dataset | Value | Status |
| :--- | :--- | :--- | :--- |
| Recall@10 (block 1) | 10K vectors, 128d, cosine | 0.9560 | ✅ Certified |
| Scaling recall | 10K / 50K / 100K vectors, 128d | 0.9980 / 1.0000 / 0.9980 | ✅ Certified |
| HNSW memory per vector (estimate) | 128d | ~1172 bytes | ✅ Certified |
| Graph consistency | Orphans / avg L0 connections | 0 / 64.0 | ✅ Certified |
| Search p50 latency | 10K vectors, 128d | 1.2 ms | ✅ Certified |
| Search p50 latency | 50K vectors, 128d | 6.1 ms | ✅ Certified |
| Latency growth 10K → 50K | — | 4.88x (sub-linear) | ✅ Certified |

### 2.2 Index rebuild via the Python SDK (PyO3 + GIL overhead included)

From [BENCHMARKS.md §2](operations/BENCHMARKS.md#-2-sdk-operations-performance-python-wrapper) — the only currently-published Python-SDK row; the table is **FROZEN pre-SIMD (CI run 2026-08-12), regen pending**:

| Operation | Dataset / Configuration | p50 | p95 | p99 | Throughput |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Index rebuild | Hybrid rebuild (HNSW + BM25) | 93.51s | *N/A (single batch)* | *N/A (single batch)* | 107 ops/s |

> Do not subtract or divide across scopes (Regla 11). BENCHMARKS.md explicitly
> forbids comparing this table with §1 (raw Rust core, no FFI boundary) or with
> the unversioned `canonical_p99` report. The regeneration command is in
> [BENCHMARKS.md §3](operations/BENCHMARKS.md#️-3-reproducing-the-benchmark-locally):

```bash
maturin develop --manifest-path vantadb-python/Cargo.toml --release
.venv/Scripts/python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000 --output benchmarks/vanta_benchmark_report.json
```

### 2.3 Batch vs sequential search (Python, GIL released)

From [BENCHMARKS.md §6](operations/BENCHMARKS.md#-6-batch-search-performance-search_batch-in-python-sdk) (5,000 records × 128d, batch size 100, top_k=10):

| Mode | Total time (100 queries) | Avg per query | Speedup |
| :--- | :--- | :--- | :--- |
| Sequential `db.search()` | 973.68 ms | 9.73 ms | baseline |
| Batch `db.search_batch()` | 243.01 ms | 2.43 ms | 4.01x |

Command: `python benchmarks/batch_vs_sequential_bench.py`.

### 2.4 Head-to-head we ran ourselves: VantaDB vs LanceDB vs Chroma

One dataset (`glove-100-angular`), one machine, one execution (2026-06-06), 10K vectors × 100d, 100 queries, top_k=10 — full table and provenance in [BENCHMARKS.md §7](operations/BENCHMARKS.md#-7-competitive-benchmark-vs-lancedb--chroma), generated with:

```powershell
python benchmarks/competitive_bench.py --dataset glove-100-angular --size 10000 --queries 100 --top-k 10 --yes
```

Read the caveats before quoting it: it is a **single local run** on synthetic ann-benchmarks data; LanceDB/Chroma ran through their native Python wrappers while VantaDB ran through PyO3 + mmap; recall@10 was low for **every** engine on that configuration (24.5% / 13.9% / 24.1%). We therefore treat it as directional only — a longer narrative write-up lives in [blog/benchmarks_vs_lancedb_chroma.md](blog/benchmarks_vs_lancedb_chroma.md). Run the script yourself rather than citing either source as truth.

---

## 3. Their Numbers (links, not figures)

We do not republish vendor performance figures. Judge for yourself:

- **Qdrant** publishes its own benchmark UI: <https://qdrant.tech/benchmarks/>
- **Neutral third party:** [ann-benchmarks.com](https://ann-benchmarks.com/) tracks published ANN results for many engines.
- **LanceDB, Chroma, sqlite-vec:** as of 2026-08-24 we could not identify a maintained official benchmark page for these projects that meets our citation bar, so we link none rather than guess. If you need comparable data, use ann-benchmarks or our [`benchmarks/competitive_bench.py`](../../benchmarks/competitive_bench.py) harness (§2.4).

---

## 4. VantaDB Practical Limits

Single table, each limit tied to code or docs:

| Limit | Value | Source |
| :--- | :--- | :--- |
| Node / record IDs | `u128` end-to-end (engine, WAL, bindings). Range `0 ..= 2^128 − 1`. IDs beyond u64 work directly; negatives or > u128::MAX raise `OverflowError`. Keep IDs as strings in JSON payloads (> 2^53 loses double precision) | [PYTHON_SDK.md §ID limits](../api/PYTHON_SDK.md#id-limits) |
| `top_k` maximum | Clamped to **1,000** in the Python binding (`const MAX_K: usize = 1_000`) | `vantadb-python/src/lib.rs:43` · WASM equivalent `vantadb-wasm/src/lib.rs:43` (ERR-022 fix) |
| Vector dimensionality | No fixed cap in the Rust core or Python SDK; dimension must be consistent within an index (mismatch raises `ValueError`). WASM/TS bindings reject any vector longer than **10,000,000** elements (`MAX_F32_VEC_LEN`) | `vantadb-wasm/src/lib.rs:38` · [PYTHON_SDK.md](../api/PYTHON_SDK.md) error contract |
| `memory_limit_bytes` | Optional runtime budget hint steering backend/mmap choices. **Also parsed from the `VANTADB_MEMORY_LIMIT` env var** by `Config::default()` (suffixed values like `2GB` accepted; invalid values warned and ignored). Explicitly **not** a proven hard RSS ceiling | `src/config.rs:1044` (env parsing) · `src/config.rs:1411` (`with_memory_limit`) · [ARCHITECTURE.md §6](../dev/architecture/ARCHITECTURE.md#6-memory-and-telemetry) |
| HNSW construction params (production defaults) | `m=32`, `m_max0=64`, `ef_construction=100`, `ef_search=100`, cosine | `src/index/graph.rs:255-269` (`impl Default for HnswConfig`) · narrative in [FND-20 HNSW tradeoff](../dev/architecture/FND-20-hnsw-tradeoff.md) |
| HNSW params in the canonical bench | `m=16`, `ef_construction=100`, `ef_search=50`, cosine | [`benches/canonical_p99.rs:35-47`](../../benches/canonical_p99.rs) |
| RAM per 1M vectors × 1536d (estimate, not measured) | ≈ 6.5 GB: vectors 1536×4 B ≈ 6.1 GB + graph edges (M+M_max0)×4 B = 384 B/node ≈ 0.4 GB | Arithmetic from the documented estimate formula in [FND-20](../dev/architecture/FND-20-hnsw-tradeoff.md) and the certified ~1172 B/vector at 128d in [BENCHMARKS.md §1](operations/BENCHMARKS.md#-1-core-engine-certification-results-rust) |
| Known ingestion limitation | HNSW construction via the SDK API is currently single-threaded | [BENCHMARKS.md §Limitations](operations/BENCHMARKS.md#-limitations-and-technical-considerations) |
| Distance metrics | Best-supported metric today is cosine | [BENCHMARKS.md §Limitations](operations/BENCHMARKS.md#-limitations-and-technical-considerations) |

---

## 5. One-Line Positioning (when to pick what)

- **sqlite-vec** — your data already lives in SQLite and you want a tiny dependency-free extension; you accept assembling hybrid/keyword search and graph logic yourself.
- **LanceDB** — you want vectors next to large multimodal/columnar datasets with SQL, full-text search, and object-storage scale.
- **Qdrant** — you will run a dedicated, horizontally scaled search service and want mature distributed operations plus dense+sparse fusion.
- **Chroma** — Python-first, batteries-included developer experience for LLM prototypes, growing into client/server deployments.
- **VantaDB** — one embedded process must hold agent memory with built-in BM25+dense RRF fusion **and** typed-graph traversal/PageRank/GraphRAG, exposed identically to Python/TS/WASM agents and MCP clients. It is an early (pre-1.0) project: expect a smaller ecosystem than the four above and validate on your hardware with §2's commands.

## 6. What We Deliberately Do Not Claim

- No claim that VantaDB outperforms any competitor. Our only head-to-head (§2.4) is one dataset on one machine and shows mixed results — including dimensions where competitors were faster.
- No uptime/distribution claims: VantaDB today targets the single-node embedded case; Qdrant's distributed story is its own and we link it rather than rank it.
- Any adjective like "fast" or "efficient" without a §2 citation is a bug in this document — report it.

## 7. Track note — agent memory for AI-IDEs (MCP)

This page compares vector stores; the AI-IDE question is different: *does your
assistant remember the project across sessions?* VantaDB's entry point there is
a single MCP server ([MCP.md](../api/MCP.md)) that Claude Code, Cursor,
VS Code, OpenCode, OpenClaw, Devin and Antigravity can all speak, plus
deterministic recall hooks for four clients (Claude Code, Codex, Cursor,
OpenCode — [hooks](../../skills/vantadb-mcp/assets/hooks/README.md)) and a
desktop viewer (Memory Lens). Cross-session recall ("session 1 stores →
session 2 recalls by a synonym with zero shared tokens") is CI-verified and
PR-blocking; the track one-pager — including what is **not** shipped yet
(live repo-map / incremental indexing) — is [AI_IDES.md](AI_IDES.md).

Qualitative only, same rules as above: no competitor figures, no performance
claims without a BENCHMARKS.md citation. The head-to-head with hosted memory
services (Mem0/Zep/Letta/LangMem/…) remains the pending layer announced at the
top of this page (P54 ICP-01..03 + VER-09).

---

## 8. Privacy & local LLMs — the guarantee layer (ICP-02)

The table above compares storage engines. This section covers what the
**local-first memory stack** guarantees once your turns are written by
`vanta-proxy`: four guarantees — plus the audit that proves they hold on a real
store — none of them "trust us".

| Guarantee | What it does | Evidence |
|-----------|--------------|----------|
| Redaction-on-write | masks `email` / `aws_key` / `aws_secret` / `token` before persistence; provenance keeps kind labels only | [`PROXY.md` §Redaction-on-write](../api/PROXY.md#redaction-on-write--encrypted-namespaces-ver-03) |
| Encrypted originals (optional) | per-namespace AES-256-GCM envelope; a missing key degrades to explicit `Disarmed`, never cleartext | [`PROXY.md`](../api/PROXY.md#redaction-on-write--encrypted-namespaces-ver-03) |
| Certified delete | `vanta-cli delete --attest` + `certificate verify` (exit 0) with a per-surface residue inventory | [`CERTIFIED_DELETE.md`](../api/CERTIFIED_DELETE.md) |
| Injection governance | budget + namespace ACL + metadata-only JSONL audit (`op:"injection"`) | [`PROXY.md` §Injection governance](../api/PROXY.md#injection-governance-ver-04) · [`MCP.md`](../api/MCP.md#injection-governance-ver-04) |
| PII audit | `vanta-pii-audit` scans raw store bytes (store + indexes + export = 0 cleartext, fail-closed) | [PRIVACY.md](PRIVACY.md) |

**What this is not** (declared, not silent): logical purge rather than secure
erase; no parametric unlearning; backups, exports and audit logs follow each
certificate's `out_of_scope` list. Every command and the full covered / not
covered table: [Privacy & local LLMs](PRIVACY.md). Run the whole chain locally
with `pwsh scripts/demo-privacy-e2e.ps1` (it also runs in CI).

---

## 9. Agent frameworks — same code in dev and prod (ICP-03)

The table above compares storage engines; the frameworks question is the one
Vertical 2 of the go-to-market plan asks: *does your agent keep its memory
without a second stack in production?* LangGraph pairs `InMemorySaver` in dev
with `PostgresSaver` in prod; CrewAI's native memory lacks per-user
isolation. VantaDB's answer is a set of framework adapters over the same
embedded engine — the graph code does not change, only the checkpointer.

- Nine Python adapters (LangGraph, CrewAI, DSPy, Haystack, LlamaIndex, Mem0,
  Letta, OpenAI, Ollama): [FRAMEWORKS.md](FRAMEWORKS.md).
- The dev→prod claim is CI-verified by a dedicated five-process demo
  ([examples/langgraph_dev_to_prod](../../examples/langgraph_dev_to_prod/README.md) ·
  [ci-frameworks-demo.yml](../../.github/workflows/ci-frameworks-demo.yml)):
  the same graph runs with `InMemorySaver` (dev) and `VantaDBCheckpointer`
  (prod); the demo fails if the business code differs, if prod does not
  resume state across processes, or if two threads leak into each other.
- Migrating from hosted memory services keeps the data: file-only importers
  from Mem0 / Zep / Letta, no API keys
  ([tutorials](tutorials/migrating-from-mem0.md)).
- Distribution honesty: the adapters are **Alpha and not on PyPI yet**
  (all nine names return 404). Install is from a repo checkout today; the
  PyPI command arrives with the first `adapters-v*` release. Install paths,
  pin caveats and the adoption metric (pypistats, baseline 0) live in
  [FRAMEWORKS.md](FRAMEWORKS.md).

Qualitative only, same rules as above: no competitor figures, no performance
claims without a BENCHMARKS.md citation.
