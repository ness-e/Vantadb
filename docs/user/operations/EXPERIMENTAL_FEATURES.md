---
title: Experimental Features and Product Boundary
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-09-24
aliases: []
---

# Experimental Features and Product Boundary

This document classifies the current v0.7.0 repository surface. It is the operational reference for
what is production-facing, optional, experimental, or deferred.

> **Revisión 2026-09-24:** superficie re-verificada contra la rama `develop` (v0.7.0) tras la investigación integral. 4 claims falsos corregidos (IQL, MCP, consolidación, PyPI, tokenizer); añadidas las categorías **Labs** (proxy/desktop/web). Regeneración completa + gate CI de frontera: filas DEF-02/DEF-03 (Backlog P55). La versión 0.1.x de este documento quedó histórica.

## Production-Facing MVP

The product boundary (v0.7.0) is an embedded local-first persistent memory engine:

| Area | Status |
| --- | --- |
| Embedded Rust SDK and CLI | Production-facing |
| Memory `put/get/delete/list/search` | Production-facing |
| WAL-backed recovery | Production-facing |
| Namespaces and scalar metadata filters | Production-facing |
| Derived namespace and metadata indexes | Production-facing |
| HNSW vector retrieval | Production-facing |
| BM25 lexical retrieval | Production-facing |
| Hybrid Retrieval v1 with deterministic RRF | Production-facing |
| Basic phrase filtering | Production-facing |
| Manual rebuild and structural audit flows | Production-facing |
| JSONL export/import | Production-facing |
| Source-installed Python SDK | Production-facing |

## Optional

| Area | Status | Notes |
| --- | --- | --- |
| Local `vanta-server` binary | Optional wrapper around the embedded core | Local dev / network exposure |
| Local ONNX embeddings (`embed-local` feature, `LocalOnnxProvider`) | **Optional local-first** | Offline, `ort`+`tokenizers`, 9 models (8 ≤3 GB + Qwen3 exception), default `multilingual-e5-small` 384d — see `docs/api/EMBEDDINGS.md`, `embeddings/manifest.json`, `docs/user/tutorials/05-embedding-integrations.md` |

The server and `embed-local` are optional wrappers around the same embedded core. `embed-local` is **not Experimental** — it is a supported offline path (BYO-vector remains default).

## Experimental or Not MVP

These surfaces may exist in the repository, but they are not stable product claims for v0.1.x:

| Area | Boundary |
| --- | --- |
| IQL/LISP/DQL parser, evaluator, and executor paths | **IQL shipped** (2026-09-24): `POST /api/v2/query` + SDK/CLI/MCP exponen SELECT/INSERT/RELATE con JOIN (sin agregaciones). Lo archivado (2024-06-10) fue la evaluación LISP runtime (borrow checker/GIL); fuzz target legado preservado en [`FUZZING.md`](FUZZING.md). Estabilización de sintaxis: API-06 (P51) |
| MCP API | **Shipped** — ~87 tools + resources + prompts (v0.7.0). Estabilización de nombres/schemas: API-04 (P51); enforce de perfil + default `agent`: WIRE-02 (P56) |
| Remote LLM/Ollama integration (`remote-inference` feature, `OllamaProvider`/`OpenAIProvider`) | External optional integration, not core dependency — alternative to `embed-local` |
| Governance and maintenance semantics | Framework runtime legado archivado (2024-06-10). Gobernanza vigente: supersede/TTL/version_history (SDK) + programa MGR (P49) → v0.7 manual → v1.0 automática. Utilidades extraídas viven en `src/utils/` (cableado del Bloom al write-path pendiente — FUT-09/VER-07) |
| Graph traversal beyond stored local edges | Experimental, not a graph database claim |
| Docker/Ollama examples | Experimental development examples |
| **vanta-proxy (LLM gateway)** | **Labs** — pipeline de 16 etapas (redact/failover/cache/cost). Auth de `/snapshot`: API-05 (P51); loop de memoria + cost output: WIRE-01 (P56) |
| **Vanta Studio (desktop Tauri)** | **Labs** — 12 lentes / 3 transportes; testing frontend fuera de CI (pendiente de conectar) |
| **Web console** | **Labs** — sitio separado en [`ness-e/Vantadb-web`](https://github.com/ness-e/Vantadb-web) (docs en `docs/user/web/` de ese repo) |
| **vanta-memory (L0→L3)** | **Shipped parcial** — pipeline completo; scheduler sin host ubicuo (`bootstrap.rs` → `conversation_trigger: None`); dreams dry-run/promote real: VER-07 (P52) |

## Extracted Utilities (Production-Ready)

The following utilities were extracted from experimental governance and are now part of the core API:

| Utility | Purpose | API Location |
| --- | --- | --- |
| DuplicatePreventionFilter | Bloom filter for duplicate prevention in multi-writer scenarios (⚠️ sin callers en el write-path hoy — cableado pendiente, FUT-09/VER-07) | `vantadb::utils::DuplicatePreventionFilter` |
| OriginCollisionTracker | Collision tracking and friction metrics for multi-agent coordination | `vantadb::utils::OriginCollisionTracker` |
| compute_confidence_friction | Functional friction metric computation for conflict analysis | `vantadb::utils::compute_confidence_friction` |

These utilities are stateless, well-tested, and suitable for production use in multi-writer and multi-agent scenarios.

Visible in-tree examples and docs must carry the same boundary:

- `examples/docker/docker-compose.ollama.yml`
- `examples/docker/Dockerfile`
- `examples/docker/start.sh`
- `examples/python/langchain_rag.py`

## Deferred

The following are explicitly outside the v0.1.x MVP:

| Area | Boundary |
| --- | --- |
| Agent metacognition and automatic memory consolidation | **Shipped parcial (2026-09-24)** — dreams L0→L3 + `dream_consolidate`/`promote` en MCP (v0.6.x); dry-run + promote real: VER-07 (P52); consumo automático depende del scheduler con host (WIRE-01) |
| Plugins and marketplace | Deferred |
| RBAC, true multi-tenancy, quotas, and enterprise audit | Deferred |
| HA, replication, clustering, and cloud managed service | Deferred |
| Production PyPI publication and signed installers | **Shipped parcial** — `vantadb-py` (PyPI) y `vantadb` (npm) publicados con attestations OIDC; firmas de binarios (cosign/minisign) pendientes — P0 |
| Advanced ranking, snippets, highlighting, Unicode folding, stopwords, stemming | Parcial: **tokenizer avanzado (stemming + stopwords + ASCII folding) es default** desde 0.5.x; snippets/highlighting siguen diferidos |
| SQL, general OLTP, warehouse, and time-series workloads | Deferred |

---

### Cross-References

- [FUZZING.md](FUZZING.md) — Fuzzing strategy for legacy LISP parser (archived) and core deserialization paths
- [BENCHMARKS.md](BENCHMARKS.md) — Published performance benchmarks for production-facing features
