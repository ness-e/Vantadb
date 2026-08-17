---
title: "Decisiones — WONTFIX"
type: decisions
status: active
tags: [vantadb, avance, wontfix, decisiones, yagni]
last_reviewed: 2026-08-07
aliases: []
---

# Decisiones — WONTFIX

> Registro de decisiones de no implementar (WONTFIX) y diferimientos (DEFER). Cada entrada documenta por qué, con criterio de re-apertura. Regla: YAGNI sobre especulación; embedded-first sobre server-heavy.

## WONTFIX formal (con ADR)

### COMP-019: Binary protocol (gRPC) — WONTFIX ✅ (2026-08-02)
- **Decisión:** WONTFIX. gRPC contradice el posicionamiento **embedded-first** de VantaDB.
- rkyv (serialización binaria zero-copy) ya cubre la serialización interna en storage/WAL — el 80% del valor técnico.
- Sin demanda de usuario ni dependencias → YAGNI.
- Micro-ADR: `docs/architecture/adr/COMP-019-binary-protocol-wontfix.md`
- **Criterio de re-apertura:** caso de uso de servidor remoto con transferencia masiva de vectores, o issue de usuario.

### DEVOPS-15: Optimizar default features Cargo.toml — WONTFIX ✅
- **Análisis:** Reducir de 7 a 3 features (`cli`, `memmap2`, `fs2`, `sysinfo`) rompe la experiencia "it just works".
- **Decisión:** WONTFIX. Las 7 features mantienen UX completa.
- Re-abierto tras detectar discrepancia en `Cargo.toml:89`, luego WONTFIX confirmado.

### AUDIT-02: Sparse hot-path micro-opt (gate de medición) — WONTFIX ✅ (2026-08-06)
- **Decisión:** WONTFIX. Mediciones en `docs/Investigaciones/AUDIT-02-2026-08-06.md`.
- Sparse hot-path no superó el gate de medición.

### T2: prefetch_mmap_vector — WONTFIX ✅
- `prefetch_mmap_vector` ya implementa `madvise(MADV_WILLNEED)` / `PrefetchVirtualMemory`. Prefetch ya activo.

### T3: Node reordering (BFS compact_layout) — WONTFIX ❌
- Investigado y descartado. Benchmark con `compact_layout` (BFS reorder) mostró solo ~9% de mejora (2,440→2,221 ms). Search sigue greedy distance-guided path, no BFS order. Overhead es de function calls y bounds checks, no page misses. <20% threshold → cerrado como WONTFIX.
- **INV-012** re-confirmó: ~7.0% incluso con LSM multi-level. NO re-abrir.

## WONTFIX operativos (bitacora)

| Item | Razón |
|------|-------|
| **routeTree.gen.ts @ts-nocheck** (640L sin typecheck) | Auto-generado por TanStack Router. No se edita manualmente. |
| **CSP en Rust HTTP server** | JSON API puro (3 rutas: /query, /health, /metrics). No sirve HTML. CSP no aplica. |
| **Adapters namespace fijo** | Namespace configurable como parámetro opcional — no blocking. |
| **SQL implementation** | 6-12 meses, diluye identidad, sin user demand. (Ver R1) |
| **SOC2 / HIPAA cert** | Tomaría meses, sin negocio actual. Remover claims falsos de web. |
| **VantaDB Cloud** | Product-market fit no validado. Remover de web hasta tener MVP. |

## DEFER (diferidos con razón, re-apertura condicional)

### P0 backlog — release blockers diferidos
| Item | Razón |
|------|-------|
| `DEVOPS-10` — Windows code signing (SmartScreen) | DEFERIDO (ponytail). SHA256 + .zip dan integridad básica. Agregar Azure Trusted Signing cuando el release público lo requiera. Step YAML preparado. |

### Optimizaciones prematuras (bitacora R6 → VantaDB_ANALISIS_COMPLETO Sección 3.1)
- **Async transcript I/O** — no es hot path
- **FilterBitset overhead** — no es bottleneck
- **Visual regression tests** (Percy/Chromatic) — sin recursos
- **WAL shipping replication** — sin mercado
- **PITR via archival WAL** — enterprise sin demanda
- **SOC2 prep** — 3-5d irreal, toma meses
- **HIPAA assessment** — sin negocio healthcare
- **Multi-tenant isolation** — no hasta Cloud
- **All VantaDB Cloud items** — product-market fit no validado
- **Async ingestion pipeline** — ya existe via channel

### CI/CD deferidos (P10)
- NIGHTLY benchmarks, self-hosted runners, matrix OS completo, coverage window auto, benchmark CI failure auto-window — catalogados P10 sin adoptar.

### Docker multi-arch
- Docker build multi-arch diferido (single-arch suficiente para release actual).

## Decisiones técnicas registradas

| Decisión | Detalle |
|----------|---------|
| DuplicatePrevention hash interno sigue XxHash64 | Interfaz pública u128 (CODE-067); el hash interno de bloom se mantiene XxHash64 deliberadamente. |
| L3 archive tier diferido | COMP-026: L0+L1 implementados; L3 archive deferido (ponytail). |
| HNSW flat threshold 10K | `VANTADB_FLAT_THRESHOLD` default 10000 — brute-force bajo el umbral por diseño (VFY-004). |
| Server→CLI coupling | `server = ["cli",...]` acopla server→cli intencionalmente (INV-011); YAGNI separar hoy. |
| Security: NON-CRITICAL advisories | 7 categorías explícitas (# CATEGORY:) alineadas con CI_POLICY y Regla 2 — no requieren gates duros. |
| CSP 'unsafe-eval' prod | Removido en frontend; WONTFIX en Rust server (JSON API puro). |
| DEVOPS-10 code signing | SHA256 + .zip; Azure Trusted Signing cuando release público lo requiera. |
| Fjall default vs RocksDB opt-in | ADR-020 (consolidación retroactiva, FND-21): Fjall backend por defecto, RocksDB feature opt-in; evidencia `Cargo.toml:97`, `config.rs:582-598`, `init.rs:269-289`. |
| Zero-copy Arrow en bindings | ADR-021 (nuevo, FND-21): buffers Arrow sin copia como dirección; bindings Python/Node aún sin Arrow. **FND-04 diferido 2026-08-16** con señal de reapertura (`docs/Investigaciones/FND-04-arrow-zero-copy.md`). |
| WAL async/batch | ADR-022 (consolidación, FND-21): batch-append por shard + roadmap async; relaciona DRV-014/DRV-015. |
| Backend compaction tuning | ADR-023 (FND-08, 2026-08-16): compactación fjall/rocksdb diferida como marginal tras bench de lectura; regla en `.opencode/rules/durability.md`. |
| Grafos default-on vs opt-in | ADR-024 (FND-23, 2026-08-16): **default-on hasta señal de telemetría** (`vanta_graph_ops_total`); no decidir por intuición; complementa FND-03. |

### FND-23: Decidir grafos default-on vs opt-in con telemetría real — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ ADR-024: motor de grafos **default-on hasta señal de telemetría** (métrica `vanta_graph_ops_total`) — no decidir por intuición; complementa FND-03. Commit `bde23fd3`.
- **Ids:** `FND-23`
