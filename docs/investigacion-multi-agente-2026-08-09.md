---
title: "Investigación Multi-Agente — Estado y Publicación de VantaDB"
type: investigation-report
date: 2026-08-09
tags: [vantadb, investigation, multi-agent, release-readiness]
---

# Investigación Multi-Agente — VantaDB (2026-08-09)

## Resumen ejecutivo

VantaDB es un motor embebido de memoria persistente + búsqueda vectorial híbrida, local-first, escrito en Rust (Apache-2.0, v0.5.0 en repo). La infraestructura central está madura: SDK memory CRUD, retrieval híbrido con fusión RRF, HNSW, WAL con CRC32C, LSM multi-nivel, grafos + GraphRAG, snapshots, cifrado AES-256-GCM y bindings para Python/TS/WASM/MCP/HTTP ya existen y pasan su suite de tests. El problema no es "no está publicado": crates.io lleva v0.4.0 (32 descargas), PyPI lleva 0.1.5/+0.5.0 y el repo público tiene 1381 commits con 14 workflows — publicado pero con visibilidad ~cero (2 stars). La investigación identificó 4 blockers de corrección (ERR-022, ERR-016, ERR-021, UAF en `__array_interface__`) + 1 riesgo FFI que deben arreglarse antes de anunciar nada; el resto es polish de docs y benchmarks. Veredicto combinado: producción-ready 5/10 (audit) y confianza de test-suite 3/5 (chaos). Correcciones pequeñas (≤1-3 días) y la infra ya está lista para un release 0.5.0 coherente.

## 1. ¿Qué es VantaDB y cómo funciona?

- Motor embebido de memoria persistente + búsqueda vectorial híbrida local-first en Rust (Apache-2.0, v0.5.0).
- Flujo de query: `VantaEmbedded.put` → StorageEngine (WAL sharded CRC32C → Fjall/RocksDB/in-memory) → planner clasifica Hybrid/TextOnly/VectorOnly (BM25 lexical + HNSW vectorial + sparse opcional) → fusión RRF (K=60).
- API estable = `src/sdk/` (`VantaEmbedded`), **NO** el parser IQL (`src/parser/mod.rs:1-30`, "historical parser surface").
- Capas: core → bindings (PyO3, TS/Wiki, MCP stdio, server HTTP axum, node napi) → adaptadores IA (LangChain, llama_index, haystack, DSPy, CrewAI, Mem0, Letta, OpenAI, Ollama).

## 2. ¿Qué problema resuelve?

Memoria de largo plazo **local** para LLMs/agentes, sin servicios externos; durable (WAL), crash-safe, híbrido. Compite con Mem0/Letta/LanceDB/Chroma con enfoque embedded-first tipo SQLite para agentes. El valor diferencial no es la búsqueda vectorial (commodity) sino la **durabilidad crash-safe + almacenamiento híbrido + operabilidad local** en un solo binario embebible.

## 3. Inventario de funcionalidades

| Categoría | Implementación | Estado |
|---|---|---|
| SDK memory CRUD | `VantaEmbedded` (`src/sdk/`) | ✅ |
| Hybrid RRF 3-arm | BM25 + HNSW + RRF (K=60) | ✅ |
| HNSW + ACORN filtered search | `hnsw.rs`, `filtered_hnsw` | ✅ |
| WAL sharded + recovery CRC32C sync 3 modos | `wal.rs`, `crc32c` | ✅ |
| LSM multi-nivel L0-L3 + GC + tombstones | `lsm.rs` | ✅ |
| Graph dirigido + BFS/DFS + temporal + GDS/PageRank | `graph.rs` | ✅ |
| GraphRAG pipeline | `graphrag.rs` | ✅ |
| Snapshot/backup/restore + JSONL export/import | `snapshot.rs` | ✅ |
| Encryption AES-256-GCM | `encryption.rs` | ✅ |
| CLI completo | `cli/` | ✅ |
| Server HTTP RBAC+rate-limit+Auth | `server/` | ✅ |
| Bindings Python/TS/WASMCPMCP/node | `bindings/` | ⚠️ con bugs (UAF, paginación) |
| IVF-Flat, DiskANN, SCANN, SQ8 quant | `index/`, `quant.rs` | ⚠️ implementados pero sin exposición SDK |
| Índices derivados, prefetch mmap, SIMD | `index/`, `mmap.rs` | ✅ |

## 4. Módulos al 100% vs parciales/faltantes

**Al 100% (10):**
1. CRUD + export/import JSONL
2. Hybrid retrieval (RRF)
3. HNSW recall ≥ 0.90 (unit-gated)
4. WAL recovery + truncado/corrupción (3 modos, CRC32C)
5. LSM multi-nivel + GC + tombstones
6. Graph temporal + GDS/PageRank
7. GraphRAG pipeline
8. Snapshot/backup/restore
9. Encryption AES-256-GCM
10. CLI completo

**Parciales con evidencia (evidencia entre paréntesis):**

- PITR / WAL-shipping: módulos reales pero **HUÉRFANOS** — no llamados por StorageEngine/SDK, solo self-tests; feature `pitr = []` vacía.
- DiskANN: no hace disk I/O (doc interno: *"purely in-memory"*).
- Arrow columnar: solo exporta `id` + primera componente (A prop).
- IVFO/SCANN: implementados en `VecIndex` pero sin exposición SDK.
- TUI / MCP / server / WASM: **EXPERIMENTAL** en `Cargo.toml` (no bloquean CI).
- `src/integrations.rs`: stubs vacíos (`search_handler` return `[]`; `ollama_proxy` "Próximamente").

**No existe (❌):** cluster/replication/distributed (solo `wal_shipping` send-only, sin receive).

## 5. ¿Qué se puede hacer hoy?

| Ruta | Ejemplo |
|---|---|
| **Rust SDK** | `VantaEmbedded::open(path)?` → `put` / `search` híbrido con filtros |
| **pybrave** | `pip install vantadb-py` → `VDBMemory` para agentes |
| **TypeScript/WASM** | `import { MemoryClient } from "@vantadb/sdk"` |
| **MCP stdio** | `"vantadb-mcp"` → herramientas de memoria para agentes que hablan MCP |
| **HTTP server** | `--server` axum con RBAC + rate-limit en `localhost` |
| **CLI** | `vantadb memory add`, `vantadb search`, `vantadb export --jsonl` |

## 6. Publicación actual y visibilidad (estado en línea — investigación externa)

- **GitHub** `ness-e/Vantadb`: público, 1381 commits, 14 workflows, ⭐2 stars, 0 forks, 0 watchers.
- **crates.io** `vantadb`: publicada v0.4.0 (repo trae 0.5.0), 32 descargas, 0 dependen.
- **PyPI** `vantadb-py`: publicado 0.1.5 (+0.5.0).
- **npm/TS**: releases de TS SDK.
- **llms.txt**: existe pero con API Python **INVENTADA** (roto).
- **Conclusión importante:** la creencia *"no lo he publicado"* es incorrecta a medias — está publicado pero con **cero distribución/visibilidad** (2 stars, 32 descargas). El bloqueo real = blockers de corrección/seguridad + benchmarks incoherentes + marketing/docs sin terminar.

## 7. ¿Está justificada la creencia "le falta"? -- veredicto combinado de 6 sub-agentes

- Auditoría producción-ready: **5/10**.
- Confianza de test-suite (chaos): **3/5**.

Tabla de blockers verificados:

| ID | Tipo | Severidad |
|---|---|---|
| ERR-022 | `top_k` sin clamp en bindings → abort proceso | **Crítica** (DoS) |
| ERR-016 | parser + search consume WHERE/RANK como alias | **Crítica** (data loss silenciosa) |
| ERR-021 | MCP OOM (carga toda la tabla) | **Crítica** |
| UAF | use-after-free en `__array_interface__` PyO3 | **Crítica** (bloquea PyPI) |
| ERR-010 | ya fix en v0.4.0 | ✅ resuelto |
| ERR-035/036 | contención en hot path | ⚠️ media |

**Conclusión:** 4 blockers + 1 riesgo FFI (UAF) son **imprescindibles** antes de publicar; el resto (docs, marketing, Benchmarks) es polish.

## 8. Config / DX / UX / Rendimiento

### Configuración
- `config.rs` (1313 líneas): builder + env vars ✅
- **NO** existe `.vanta/config.toml` ⚠️
- Hot-reload JSON no documentado ⚠️
- `hardware_profiles` (auto-detection de recursos) excelente ✅

### DX de desarrollo
- Justfile cross-platform ✅
- CONTRIBUTING / CLA ✅
- **QUICKSTART desactualizado** (dice "v0.4.x"; real 0.5.0) ⚠️
- Wheel local stale: `dist/vantadb_py-0.1.5` ⚠️
- Badge PyPI 0.1.5 vs core 0.5.0 = **drift de versionado** ⚠️

### UX docs
- 8 docs API, 13 ADRs, 30+ ops docs, TEST_MAP ✅
- CHANGELOG muerto (última entrada 0.5.0; faltarán 25+ commits; sin `[Unreleased]` — ERR-050) ⚠️
- Mojibake en `vantadb-python/README.md` y `docs/DESIGN_RULES.md` ⚠️
- `llms.txt` inventado ⚠️
- `docs/README.md` con wikilinks Obsidian que no render en GitHub ⚠️
- Límite u64 (ERR-023) **no documentado** ⚠️

### Rendimiento
- Core OK: SIFT1M@100K → 3636 QPS p99 441µs ✅
- PERO `bench/hnsw_pure.rs` mide **brute-force** (`flat_threshold=10000`, ERR-019) ⚠️
- Doble verdad publicada: JSON local vs tabla CI (discord 2021×001×) ⚠️
- Competitivo del repo: 598 ingest QPS vs LanceDB 114K (**190×**); 24 query QPS vs Chroma 906 (* — path SDK 1-00× abajo) ⚠️
- `criterion_baseline.json` vacío ⚠️
- Hot paths con top-3 problemas ERR-036 (write-lock en `get()`), ERR-042 (`read_header` duplicada), ERR-048/047 (2x hashes, copias inline) ⚠️
- Prefetch mmap: micro-bench propio dice **NO aporta** (≤2% peor p99) → default OFF correcto ✅

### UX de usuario
- Quickstart de pip: 5 min ✅
- CLI para usuario final / contribuidor: **no** (exige toolchain Rust) ⚠️

## 9. ¿Qué le falta antes de publicar? (bloqueadores)

1. Fix **ERR-022** — clamp de `top_k` en los 3 bindings + hard cap en core.
2. Fix **ERR-016** — parser/planner: WHERE/RANK no deben consumirse como alias.
3. Fix **ERR-001-ERR-021** — MCP: paginación con `take(n)`, no cargar toda la tabla.
4. Fix **ERR-UAF** — `__array_interface__` PyO3 (use-after-free).
5. `cargo semver-checks` + `cargo deny audit` verde + `release-plz publish` v0.5.0 en crates.io y wheels PyPI **consistentes** (repositorio + versión).
6. Reparar benchmarks publicados: sello de máquina+commit, fix de `hnsw_pure.rs`, `criterion_baseline.json`.
7. Regenerar CHANGELOG (git-cliff), arreglar `llms.txt`, corregir mojibake, documentar límite de `u64` en tamaños/offsets.

## 10. Lo que falta para ser "completo" como producto competitivo — y backlog

### Roadmap (post-bloqueadores)

- **WAL async** — 10-100× ingest (hoy sync por lote).
- **PITR** y wal-shipping enforced (hoy huérfanos) en SDK.
- **Query planner** con optimizaciones reales (hoy router + heurística).
- **IVFO/SCANN/DiskANN** expuestos por SDK (hoy internos del `VecIndex`).
- **Extras post-launch**: desktop Tauri (consola admin), admin web (8/9 done), Enterprise encryption + RBAC ya en core.

### Backlog restante (~113 activas)

- P0-P4 cerradas.
- P5: 3 de docs.
- P6: 1 de launch (LEG-01 trademark humano).
- P8: BIZ-01b.
- P9: OLD-852 PGWire.
- P12: desktop + admin (ADMIN done 2026-08-08, queda DESKTOP).
- P14: REVIEW 3 god modules.
- P15: ERR 50 hallazgos (5 críticos).
- AUD-016..AUD-021 pendientes.

## Conclusiones y recomendaciones

1. **No esperes a "completar todo".** El punto de publicación es cuando se corrijan los 4-4 blockers — todos son fixes pequeños (≤1-3 días). Todo lo demás (roadmap de índices ID-, WAL async, wrappers poderosos) es post-launch iteration.

2. **La infra ya está madura.** 10 módulos core al 100%, suite de tests amplia, hook de CI, bindings principales — el caballo de batalla ya está hecho. El esfuerzo post-publicación se enfoca en marketing consistent y distribución (GitHub SEO, docs sin mojibake, llms.txt real, benchmark con métricas repetibles).

3. **El bloqueo real no es código sino visibilidad + narrativa.** El proyecto ya está en crates.io/PyPI/npm/GitHub — lo que falta es que las dos versiones publicadas coincidan (v0.4.0 crates vs 0.5.0 repo), que la documentación no mienta (llms.txt, wikilinks, QUICKSTART) y que los benchmarks cuenten la historia real (sustituir brute-force por HNSW real, sello de máquina y baseline).

4. **Cuida la coherencia de versionado.** El drift 0.1.5 (PyPI) vs 0.5.0 (core) en badges y wheels locales genera desconfianza de distribución. Único fuente de verdad del release-plz en CI, sin tags manuales.

5. **Métricas de éxito para el launch:** recuperar índice HNSW real en `bench/hnsw_pure.rs` (≠ brute), un benchmark publicado con sello de máquina+commit, y un experimento rápido de "pip install → memoria en 5 min" con wheel correcta.

## Fuentes

- `docs/Backlog.md` — estado del backlog, P0-P15, prio.
- `docs/CHANGELOG.md` — hist. de versiones (drift detectado, ERR-050).
- Código fuente Rust: `src/` (`sdk/`, `parser/mod.rs`, `storage/`, `index/`, `graph/`, `graphrag.rs`, `snapshot.rs`, `encryption.rs`, `cli/`, `integrations.rs`, `Cargo.toml`).
- `docs/planos/reviews/errors-found.md` — catálogo de errores (ERR-016, ERR-021, ERR-022, ERR-035, ERR-036, ERR-019, ERR-042, ERR-047, ERR-048, ERR-050, ERR-023…).
- crates.io: [vantadb v0.4.0](https://crates.io/crates/vantadb) — crates.ioóndome de descargas.
- PyPI: [vantadb-py](https://pypi.org/project/vantadb-py/) — dist 0.1.5/0.5.0.
- GitHub: `nesse/Vantadb` — commits, workflows, stars/forks/watchers.
- `docs/QUICKSTART.md`, `docs/README.md`, `docs/DESIGN_RULES.md` — drift de versionado, wikilinks, mojibake.
- `bench/hnsw_pure.rs` y `criterion_baseline.json` — problema de medición real vs brute-force.