---
title: Bitacora — Development Log
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-13
aliases: []
---

# bitacora — Development Log

## Julio 2026

### Semana 1 (2026-07-01) — Documentation Audit, Rust Examples & Certification

- **AUD-06 — Documentation audit:** Reviewed all docs for consistency, fixed mixed-language content, expanded coverage.
- **AUD-07 — FAQ created:** `docs/FAQ.md` with common questions.
- **AUD-08 — Rust examples shipped:** 4 compilable examples in `examples/rust/` (basic, hybrid, graphrag, concurrent).
- **AUD-09 — Memory Telemetry doc fixed:** title/content unified to English.
- **AUD-10 — Master Index verified:** all internal links confirmed valid.
- **AUD-11 — Performance optimizations:** HNSW lock timeout defaults, reduced allocation hot paths in planner.
- **AUD-12 — Dead code removal:** unused imports, deprecated `VantaOpenOptions`, commented-out hardware refs.
- **AUD-13 — Config hardening:** `VantaConfig` defaults validated, env-var overrides for lock timeouts.
- **AUD-14 — Documentation improvements:** unified terminology across CONFIGURATION, DURABILITY_GUARANTEES, SDK.
- **AUD-15 — Test expansion:** WAL resilience, zero-vector, TTL-expired edge cases.
- **AUD-16 — Doc consolidation:** redundant Spanish MPTS sections archived, stale wikilinks removed.
- **AUD-17 — CI hardening:** failpoint chaos tests + `cargo audit` step.
- **AUD-18 — Changelog update:** backfilled v0.1.3–v0.1.5 entries.

---

### Semana 2 (2026-07-09..11) — Full Codebase Audit (Jul-09 & Jul-11)

**Scores:** 7.3/10 (Jul-09) → 7.8/10 (Jul-11, ↑0.5)
**Source:** `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` (755L, ~80 hallazgos)

**3 rounds of AI-agent audits, 5 skills, ~45 commits between audits.**

**Key blocks resolved (verificado contra código):**
- P0: SAFETY docs en todo `unsafe { }`, `lru 0.12.5` → 0.13, `llms.txt` creado
- P1: Docker profile path, `wasm-opt = true`, CI workflows validados
- Source chaining (`ChainedError`), forced-auth (`--require-auth`), `AtomicU128` entry_point
- `cli_handlers/` fragmentado en 12 submódulos, `index/core.rs` en 5 archivos
- WASM `wee_alloc` removido, `idb_bridge.js` inlineado vía `#[wasm_bindgen(inline_js)]`
- `#[must_use]` en `VantaError`, dead_code ~6 métodos limpiados
- `engine.lazy.tsx` 412L→142L (4 subcomponentes), `pricing.lazy.tsx` 348L→139L
- `NbNav.tsx` 298L→224L (`useFocusTrap` + `NavDrawer`)
- CSP `'unsafe-eval'` removido en prod; WONTFIX en Rust server (JSON API puro, no sirve HTML)
- LangChain + LlamaIndex adapters: verificados, ambos existen con tests

---

### Semana 3 (2026-07-13) — Consolidación Research + Reviews → Bitácora

**Qué se hizo:** Carga completa de `docs/research/` (9 archivos) y `docs/reviews/` (12 archivos). Extracción de ~150 items. Verificación cruzada contra el codebase real usando `codegraph_explore`, `grep`, y lectura directa para determinar estado (resuelto vs pendiente). Skills usadas: `code-review-and-quality`, `doubt-driven-development`, `writing-plans`, `ponytail (full)`.

**Items verificados como resueltos** (marcados pendientes en audits previos, ahora confirmados):

| Item | Código verificado | Estado |
|------|-------------------|--------|
| **VERSION sync** (VAL-13/UNI-01): 9 crates hardcodeaban v0.1.5 | `src/metadata.rs:13` usa `env!("CARGO_PKG_VERSION")`. `src/cli.rs:11` también. Sin hardcoded const. | ✅ Resuelto |
| **U4 — TOCTOU en ops.rs** (delete/delete_batch sin insert_lock) | `src/storage/engine/ops.rs:616-624` (delete) y `700-708` (delete_batch): ambos adquieren `insert_lock.try_lock_for()` antes de tocar HNSW. | ✅ Resuelto |
| **WASM tests vacíos** (AP-26): `wasm_tests.rs` reportado como 0 tests | `vantadb-wasm/tests/wasm_tests.rs`: 751 líneas, 30+ tests (OPFS, CRUD, vector search, error handling, batch, pagination, concurrent). | ✅ Resuelto |
| **OG tags faltantes** (AW-01): Sin og:title/og:description | `web/src/lib/seo.ts:28-29` exporta función que genera OG tags. Cada ruta los incluye en `<head>` (56 matches de `og:title` + `og:description`). | ✅ Resuelto |
| **Graph N+1** (AP-04): BFS/DFS con storage.get() uno por uno | `src/graph.rs:25,74,93,125`: código documentado con "to avoid N+1 storage lookups" — usa `get_many()` batch loading. | ✅ Resuelto |
| **Backend N+1** (AP-05/06): PhysicalScan/vector search N+1 | `StorageBackend::get_many()` tiene default sequential, pero `RocksDbBackend.get_many()` (rocksdb_backend.rs:182-221) usa `db.multi_get()` true batch. FjallBackend usa default sequential — parcial. | ⚠️ Parcial (RocksDB OK, Fjall N+1) |

---

## Consolidación de Items Pendientes

Todos los items extraídos de `docs/research/` (9 archivos) y `docs/reviews/` (12 archivos) que implican modificar el proyecto, verificados contra código real. Organizados por dominio. Cada item incluye: qué es, por qué importa, qué investigar antes de implementar, archivos fuente de referencia, y estado.

---

### 🔴 CORE ENGINE — Integridad, Performance, Arquitectura

#### P1: HNSW insert_lock bottleneck
- **Qué:** `StorageEngine.insert_lock` es `parking_lot::Mutex<()>` único que serializa TODAS las mutaciones HNSW (insert, delete, delete_batch). Busca y delete también lo adquieren ahora (verificado en `ops.rs`), pero sigue siendo un bottleneck de writer único.
- **Por qué:** Bajo carga de inserción continua, throughput está limitado a ~1 core. Server DBs usan particionamiento HNSW.
- **Solución:** (1) Rayon micro-batching (UNI-17) — pending batch de hasta 64 ops, flush bajo `insert_lock` único; (2) HNSW particionado por namespace (futuro).
- **Archivos:** `src/storage/engine/mod.rs:148`, `src/storage/engine/ops.rs`
- **Fuente:** FULL_CODEBASE_AUDIT P7, UNI-17
- **Esfuerzo estimado:** 1-2 semanas
- **Estado:** ✅ Resuelto 2026-07-13. Implementado pending batch buffer (64 ops) en `ops.rs`: `PendingHnswOp`, `flush_pending_hnsw()`, `try_push_pending_hnsw()`. `insert()` push → pending batch → flush bajo lock único (64x menos adquisiciones en alta concurrencia). `batch_insert()` ya óptimo (1 lock para N nodos) — no migrado. `delete()`/`delete_batch()` mantienen sync. `flush()` drena pending batch. Maintenance paths ya batch bajo single lock — no migrados. Bench existente en `benches/bench_concurrent.rs` (mixed read-write). Commits: `141e628`, `3a52180`.

#### P2: WAL Mutex contention
- **Qué:** `Mutex<Option<WalWriter>>` en WAL tradicional serializa writes. ShardedWal (`wal_sharded.rs`) reduce contención pero cada shard tiene su propio WalWriter. El init usa ShardedWal (`src/storage/wal.rs`), pero hay paths legacy que usan WalWriter directo.
- **Por qué:** Bajo write-heavy throughput, WAL puede ser bottleneck de I/O.
- **Investigar:** (1) Confirmar que ShardedWal se usa en todos los paths de escritura; (2) Verificar si hay backends/configs que usen WalWriter directo; (3) Medir contención real con `#[instrument]` o tracing.
- **Archivos:** `src/wal.rs:172`, `src/storage/wal.rs`, `src/wal_sharded.rs`
- **Fuente:** AP-08 (analisis_proyecto.md), FULL_CODEBASE_AUDIT P5
- **Esfuerzo:** 2 días
- **Estado:** ✅ Resuelto 2026-07-13. ShardedWal ya usado en TODOS los paths de escritura (ops.rs: insert, batch_insert, delete, delete_batch; engine.rs InMemoryEngine; maintenance.rs). WalWriter directo solo en tests — legítimo. Se removió `#[allow(dead_code)]` stale, se fixeó `rotate_all()` para preservar buffer_size/flush_threshold configurados. Commit `fc28768`.

#### P3: ACID Transaction Layer — Phase 1 (WAL Transaction Records)
- **Qué:** No hay Begin/Commit/Abort en WAL. Cada write se "commitea" implícitamente. Si `write_node_to_vstore` éxito pero `write_batch` (KV) falla, orphan vector queda en VantaFile sin rollback. Similar para HNSW.
- **Por qué:** Integridad de datos. Hoy no hay coordinación entre VantaFile, KV backend y HNSW — cada paso puede fallar independientemente.
- **Investigar:** (1) Implementar variantes `Begin`/`Commit`/`Abort` en `WalRecord`; (2) Recovery debe descartar writes entre Begin y Abort no cerrados; (3) VantaFile necesita operación reversible (o lazy cleanup).
- **Archivo recomendado:** `docs/research/ACID_TRANSACTIONS.md` (análisis completo con 3 enfoques evaluados)
- **Archivos código:** `src/wal.rs`, `src/storage/engine/ops.rs`, `src/storage/vfile.rs`
- **Fuente:** ACD-01..10
- **Esfuerzo:** ~2 semanas (Phase 1)
- **Estado:** ✅ Resuelto 2026-07-13. Phase 1: `Begin(u64)`, `Commit(u64)`, `Abort(u64)` agregados a `WalRecord` en `src/wal.rs:57-61`. Engine expone `begin_transaction()`, `commit_transaction()`, `abort_transaction()` en `ops.rs`. Recovery (`init.rs`) usa skip_mask de dos pasadas: descarta writes de transacciones abortadas/no cerradas. Compila, fmt limpio, 576/577 tests pasan (pre-existing `deserialize_absurd_node_count`). VantaFile rollback deferred a Phase 2 (P4).

#### P4: VantaFile writes no reversibles
- **Qué:** Una vez escritos bytes en VantaFile, no hay "un-write". Solo se puede marcar tombstone. Si el KV write posterior falla, el vector queda huérfano.
- **Por qué:** Arquitectura actual no soporta rollback. Afecta ACID.
- **Investigar:** (1) Approach A: lazy cleanup al inicio de próxima escritura (detectar orphans recorriendo VantaFile); (2) Approach B: buffered writes con VantaFile append-only y GC periódico; (3) Approach C: journal posicional reversible. Ver ACD_TRANSACTIONS.md para análisis detallado.
- **Fuente:** ACD-06, ACD-10
- **Esfuerzo:** 1-3 días (según enfoque)
- **Estado:** ✅ Resuelto 2026-07-13. `insert()` y `batch_insert()` en `ops.rs` ahora tombstones el entry de VantaFile si el KV write falla, previniendo vectores huérfanos. `delete()` y `delete_batch()` ya tombstoneaban antes del KV delete, por lo que no tenían el problema. `cargo check` + `cargo nextest` 576/577 pass (1 pre-existing) + `cargo fmt --check` limpio.

#### P5: Fragmentar archivos monolíticos del core
- **Qué:** 6 archivos >700 líneas sin fragmentar, dificultan mantenimiento y review.
- **Contexto:**
  | Archivo | Líneas | Propuesta de división |
  |---------|--------|----------------------|
  | `src/sdk/serialization.rs` | 1827 | `sdk/serialization/{records, formats, io, tests}` |
  | `src/metrics/core.rs` | 1604 | `metrics/{core, histogram, gauge, registry}` |
  | `src/config.rs` | 1184 | `config/{vantaconfig, env, cli, builder}` |
  | `src/cli_server.rs` | 831 | `server/{routes, middleware, tls}` |
  | `src/wal.rs` | 747 | `wal/{writer, reader, record}` |
  | `src/text_index.rs` | 736 | `text_index/{tokenizer, index, search}` |
- **Investigar:** (1) Verificar que no hay dependencias circulares entre las divisiones propuestas; (2) Asegurar que visibilidad `pub(crate)` se mantiene; (3) Ejecutar tests completos post-división.
- **Fuente:** FULL_CODEBASE_AUDIT A1-A6
- **Esfuerzo:** ~5 días total

#### P6: Duplicate code patterns
- **Qué:** `append_to_vstore` / `write_node_to_vstore` (~40 líneas casi idénticas). `if let Some(ref mut wal) = *self.wal.lock() { wal.append(...) }` repetido en insert/update/delete. `read_only` check 5 veces en `sdk/api.rs`. `init_telemetry` ~160 líneas de if/else repetitivo.
- **Investigar:** Extraer a funciones compartidas o macros; verificar que no hay side effects diferentes entre las instancias duplicadas.
- **Archivos:** `src/storage/ops.rs`, `src/sdk/api.rs`, `src/lib.rs`
- **Fuente:** AP-10/11/12/13 (analisis_proyecto.md)
- **Esfuerzo:** 2 días

#### P7: Error hierarchy gaps
- **Qué:** 4 variantes String remanentes (`IqlError`, `CliError`, `SearchError`, `RuntimeError`) sin proper error types. `IqlParseError` sin tipo `Spanned`. 4 `unwrap()` en producción en `wal_archiver.rs:78,81,120,183`.
- **Investigar:** (1) Migrar String variants a struct variants con source chaining; (2) Usar `Spanned` para IQL parse errors; (3) Reemplazar `unwrap()` con `?` o `context()`.
- **Archivo:** `src/error.rs:217-230,160`, `src/wal_archiver.rs`
- **Fuente:** FULL_CODEBASE_AUDIT E2, E3, E7
- **Esfuerzo:** 1 día

#### P8: release_mmap_vector pública sin doc SAFETY
- **Qué:** `fn release_mmap_vector()` en `src/index/graph.rs:65` tiene `#[allow(clippy::missing_safety_doc)]` — es `unsafe fn` sin documentación de precondiciones.
- **Investigar:** Agregar `# Safety` doc describiendo bajo qué condiciones es seguro llamarla.
- **Archivo:** `src/index/graph.rs:65`
- **Fuente:** FULL_CODEBASE_AUDIT U3
- **Esfuerzo:** 30 min

#### P9: Magic numbers
- **Qué:** `1024` capacity, `64` byte alignment, `0x8` tombstone flag, `0.80` RSS threshold hardcodeados.
- **Investigar:** Mover a constantes con nombre (`const TOMBSTONE_FLAG: u8 = 0x8;`) en módulos apropiados.
- **Fuente:** AP-14 (analisis_proyecto.md)
- **Esfuerzo:** 1 día

#### P10: Mixed Spanish/English code comments
- **Qué:** Comentarios en español en `storage.rs`, `wal.rs`, `text_index.rs`.
- **Investigar:** Unificar a inglés; o decidir política de lenguaje y aplicarla consistentemente.
- **Fuente:** AP-15 (analisis_proyecto.md)
- **Esfuerzo:** 1 día
- **Estado:** ✅ Completado 2026-07-13. 11 comentarios traducidos (8 en `src/wal.rs`, 3 en `src/bin/lock_helper.rs`).

#### P11: No `#![warn(missing_docs)]` en ningún crate
- **Qué:** Ningún crate del workspace tiene `#![warn(missing_docs)]`.
- **Investigar:** Agregar y documentar APIs públicas faltantes.
- **Fuente:** AP-16 (analisis_proyecto.md)
- **Esfuerzo:** 2-3 días

#### P12: `/metrics` endpoint público sin auth
- **Qué:** `src/cli_server.rs` expone `/metrics` sin autenticación. El resto del server requiere API key.
- **Investigar:** Agregar autenticación opcional configurable para `/metrics`.
- **Fuente:** AP-19 (analisis_proyecto.md)
- **Esfuerzo:** 1 hora

#### P13: Flat index threshold (small dataset optimization)
- **Qué:** Para datasets <10K vectors, brute-force search es 10-100x más rápido que HNSW (evita overhead del grafo).
- **Investigar:** (1) Implementar threshold automático basado en cardinalidad; (2) Benchmark para determinar el threshold óptimo en hardware target; (3) Opción de configuración para override manual.
- **Fuente:** UNI-06 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** 2 días

#### P14: PQ + Scalar Quantization
- **Qué:** Product Quantization 4-128x compression para datasets que no caben en RAM. Scalar Quantization SQ8 (f32→i8) ya tiene gobernador implementado.
- **Investigar:** (1) Implementar PQ como backend opcional en `src/vector/`; (2) Evaluar librerías (Faiss PQ, etc.); (3) Definir API de activación (automática por threshold + manual).
- **Fuente:** UNI-13 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** ~2 semanas

#### P15: Enterprise crate stubs
- **Qué:** `src/enterprise/encryption.rs` (26L), `audit.rs` (52L), `replication.rs` (48L), `license.rs` (24L) son stubs con `todo!()`.
- **Investigar:** (1) Evaluar si encryption en reposo debe integrarse con AES-256-GCM existente en `crypto.rs`; (2) Audit trail necesita WAL-based event sourcing; (3) Replication requiere server mode multi-instancia; (4) License module necesita scheme de licenciamiento.
- **Fuente:** UNI-29 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** 3 días (completar stubs funcionales)

#### P16: Security gaps
- **Qué:** No encryption at rest (solo stub), WAL en texto plano, security fuzzing solo parser (no API/IPC), security audit tests solo 1 file.
- **Investigar:** (1) Crypto.rs tiene AES-256-GCM — integrar con VantaFile y WAL; (2) Usar `crypto.rs` para WAL record encryption; (3) Agregar fuzz targets para HTTP API, IQL parser; (4) Expandir security tests (auth bypass, input validation, path traversal).
- **Fuente:** UNI-34/35/36/38 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** ~1 semana

#### P17: ACORN filtered search
- **Qué:** Gamma-augmented graph structure para filtered recall 10%→96% (Alotaibi et al., 2024).
- **Contexto:** VantaDB hoy no tiene filtered search optimizado — filtra post-search. ACORN integra filtros en la estructura del grafo.
- **Investigar:** (1) Evaluar implementación vs. pre-filtering + HNSW; (2) Paper: "ACORN: Accelerating Filtered Vector Search via Graph Structure"; (3) Compatibilidad con HNSW existente.
- **Fuente:** UNI-15 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** 2-4 semanas (research phase)

---

### 🟠 WEB FRONTEND — Credibilidad, UX, Performance

#### W1: Benchmarks web falsificados
- **Qué:** La web (vantadb.dev/product/benchmarks) muestra VantaDB 50x más rápido que ChromaDB. La realidad documentada en `docs/BENCHMARKS.md` muestra ChromaDB 40x más rápido. **La relación es inversa.**
- **Por qué:** Riesgo legal y de credibilidad. Si un competitor verifica, es reputación dañada inmediatamente.
- **Investigar:** (1) Leer `docs/research/AUDITORIA_COMPLETA_VantaDB_WEB.md` líneas 424-430; (2) Verificar números actuales en `web/src/routes/product/benchmarks.tsx`; (3) Decidir: corregir números, o remover la sección completa si no hay benchmarks reproducibles.
- **Fuente:** AW-47 (AUDITORIA_COMPLETA_VantaDB_WEB.md)
- **Esfuerzo:** 1 día

#### W2: API docs web rotos
- **Qué:** Las signatures en la web (vantadb.dev/docs-api) muestran parámetros incorrectos: falta `namespace` y `payload`, estructura de retorno es `results[0].score` en vez del real `results["records"][0]["score"]`, `create_collection()` no existe como método.
- **Por qué:** Los ejemplos fallan inmediatamente al copiarlos. Bloquea adopción.
- **Investigar:** (1) Leer AW-46 (AUDITORIA_COMPLETA_VantaDB_WEB.md líneas 413-419); (2) Cross-reference contra `vantadb-python/vantadb/__init__.pyi` y `src/sdk/api.rs` para obtener APIs reales; (3) Reescribir docs-api para reflejar API real.
- **Fuente:** AW-46
- **Esfuerzo:** 2 días

#### W3: Claims falsos en landing page
- **Qué:** La web afirma "SQL support" (no existe), "auto-embeddings" (VantaDB no genera embeddings), "sub-millisecond latency" (benchmark real Python: 179ms).
- **Por qué:** Engineering puede evaluar el producto y encontrar discrepancias inmediatas. Credibilidad destruida.
- **Investigar:** (1) Buscar todos los claims en `web/src/routes/` que no corresponden a funcionalidad real; (2) Decidir: feature roadmap vs. current capability; (3) Si es roadmap, marcarlo explícitamente como tal.
- **Fuente:** AW-42/43/45
- **Esfuerzo:** 1 día

#### W4: Cloud tiers / SOC2 / HIPAA sin infraestructura
- **Qué:** "Deploy Now" → formulario de contacto (bait-and-switch). SOC2/HIPAA mencionados sin implementación. Pricing: Self-Hosted "Unlimited vectors" vs Cloud Pro "1M vectors" — contradictorio.
- **Por qué:** Riesgo legal (SOC2/HIPAA claims falsos). UX engañosa (Deploy Now no deploya).
- **Investigar:** (1) Remover claims de SOC2/HIPAA hasta certificación real; (2) Reemplazar "Deploy Now" con "Request Early Access" o similar; (3) Ajustar pricing page para reflejar realidad.
- **Fuente:** AW-49/50/52/53
- **Esfuerzo:** 1 día

#### W5: OG image branding wrong
- **Qué:** OG image usa `#ff6a00` en vez de brand amber `#ff5500`, `#08080c` en vez de `#0a0a0a`. Logo sin dark variant (invisible en OLED footer).
- **Investigar:** Verificar colores en `web/public/og-image.svg` (o .png) y corregir.
- **Fuente:** AW-36/37
- **Esfuerzo:** 1 día

#### W6: Security headers missing in Vercel
- **Qué:** No HSTS, no X-Content-Type-Options, no HTTP→HTTPS redirect configurados en `web/vercel.json`.
- **Investigar:** Agregar headers en `vercel.json`; verificar con securityheaders.com.
- **Fuente:** AW-56/57/58
- **Esfuerzo:** 1 hora

#### W7: Contenido no-responsivo
- **Qué:** About pages (company.tsx, community.tsx, contact.tsx) tienen 0 media queries — grids no colapsan en mobile. 6+ breakpoints diferentes en 27 archivos. Sistema mobile-last (max-width) en vez de mobile-first (min-width).
- **Investigar:** (1) Definir breakpoint system centralizado en Tailwind config; (2) Migrar a mobile-first; (3) Agregar media queries a about pages.
- **Fuente:** AW-31/32/33
- **Esfuerzo:** 1-2 días

#### W8: Diseño system gaps
- **Qué:** `--white: #000000` (nombre confuso), `--amber` es naranja (#ff5500), no hay spacing/z-index scales. Pricing hardcodea `#ff3b30` en vez de `var(--danger)`. SwissHero.tsx hardcodea `"#ff5500"` en vez de `var(--amber)`.
- **Investigar:** Renombrar tokens inconsistentes; centralizar colores en CSS custom properties; reemplazar hardcodes.
- **Fuente:** AW-07/08/09/10/38/39/40
- **Esfuerzo:** 2 días

#### W9: SEO gaps
- **Qué:** Twitter cards sin site/creator, 3 routes fuera del sitemap (/docs-api, /security, /product/benchmarks), JSON-LD incomplete (falta url/image/softwareVersion), blog sin canonical en /$slug, no schema.org/BlogPosting, blog sin CTAs de conversión.
- **Investigar:** (1) Agregar missing meta tags en `lib/seo.ts`; (2) Actualizar `sitemap.xml`; (3) Expandir JSON-LD; (4) Agregar canonical URLs.
- **Fuente:** AW-02/03/04/05/23/24
- **Esfuerzo:** 1 día

#### W10: UX/navigation issues
- **Qué:** Nav drawer diferente de desktop (agrega link "Docs" que desktop ya tiene separado). 13 rutas invisibles desde nav. /docs y /docs-api duplican contenido. Sin breadcrumbs.
- **Investigar:** (1) Unified nav config (drawer y desktop comparten items); (2) Agregar rutas faltantes al nav; (3) Consolidar /docs + /docs-api.
- **Fuente:** AW-17/18/19/20
- **Esfuerzo:** 2 días

#### W11: CSS duplication & inline styles
- **Qué:** 3 sistemas de estilo compitiendo (CSS classes + inline styles + `<style>` JSX). Botones definidos en buttons.css Y swiss-hero.css. Footer en footer.css Y utilities.css Y utilities.css. ~80% de componentes con `style={{}}` inline.
- **Investigar:** (1) Decidir strategy: Tailwind-only? CSS modules-only?; (2) Migrar inline styles a clases; (3) Consolidar CSS duplicado.
- **Fuente:** AP-21, AW-28
- **Esfuerzo:** ~1 semana

#### W12: React performance — 0 memoization
- **Qué:** 0 `React.memo`, 0 `useMemo`, 0 `useCallback` en ~50 componentes. Componentes rerenderizan en cada navegación.
- **Investigar:** (1) Identificar componentes pesados (Three.js hero, Nav con ~22 rutas, benchmark tables); (2) Agregar memoization estratégica (no blanket — YAGNI).
- **Fuente:** AP-24 (analisis_proyecto.md)
- **Esfuerzo:** 2 días

#### W13: Animation libraries bundling
- **Qué:** 3 librerías de animación: GSAP 3.15 (legacy residual) + Motion 12.42 + AnimeJS 4.5 ≈ ~155KB extra.
- **Investigar:** (1) Verificar si GSAP se usa en algún componente activo; (2) Motion ya es el reemplazo post-migración; (3) AnimeJS tiene usos únicos? Si no, dropear.
- **Fuente:** AP-20
- **Esfuerzo:** 1 día

#### W14: Direct DOM mutation (React anti-pattern)
- **Qué:** Componentes mutan `element.style` directamente en handlers `onMouseEnter`/`onMouseLeave`.
- **Investigar:** Migrar a estado React (`useState` + `style` prop, o CSS classes condicionales).
- **Fuente:** AP-25
- **Esfuerzo:** 1 día

#### W15: Three.js hero issues
- **Qué:** Sin error boundary (WebGL fail → blank space), mouse tracking activo en mobile, wireframe position fija causa overflow en mobile, sin `prefers-reduced-motion` check (solo SwissHero verifica).
- **Investigar:** (1) Error boundary con fallback visual; (2) Detectar mobile por touch support; (3) Responsive wireframe position; (4) Respetar reduced motion en todos los componentes animados.
- **Fuente:** AW-12/13/14/15
- **Esfuerzo:** 1 día

#### W16: Blog factual errors
- **Qué:** "License: MIT" en blog post (actual: Apache 2.0). GitHub link apunta a `ness-e/VantaDB` no a `vantadb/vantadb`.
- **Investigar:** Corregir en `web/src/routes/blog/introducing-vantadb.md`.
- **Fuente:** AP-34 (analisis_proyecto.md)
- **Esfuerzo:** 30 min
- **Estado:** ✅ Ya correcto. License ya dice "Apache 2.0", GitHub link coincide con remote real `ness-e/Vantadb`.

#### W17: Touch targets <44px (Apple HIG)
- **Qué:** Hamburger menu 36px, nav-cta ~32px, close button 36px. Mínimo HIG es 44px.
- **Investigar:** Agregar padding para alcanzar 44px target.
- **Fuente:** AW-34
- **Esfuerzo:** 30 min
- **Estado:** ✅ Completado 2026-07-13. Hamburger 40→44px, drawer close 36→44px, modal close 2.25rem→2.75rem en `nb-nav.css` y `nb-components.css`.

#### W18: Misc web cleanup
- **Qué:** 404 page usa Tailwind classes inexistentes (`rounded-md`, `bg-primary` — Tailwind v4 cambió naming). Discord/X links son `"#"` placeholders. Emails no clickeables (sin `mailto:`). Blog sin CTAs de conversión. Sin `pendingComponent` en lazy routes. No analytics. TanStack Router over-engineering (~27 rutas para SPA principalmente estática). FaQ solo 4 preguntas. Sin dark mode.
- **Fuente:** AW-41/54/55/59/60/61, AP-22, AW-11, AW-24
- **Esfuerzo:** 3-4 días acumulado

---

### 🟡 BINDINGS — WASM, Python, TS, MCP, Adapters

#### B1: WASM IndexedDB fallback
- **Qué:** OPFS usado directamente sin fallback. Firefox private mode y Safari <15.2 no soportan OPFS — usuarios obtienen error críptico.
- **Investigar:** (1) Detectar OPFS disponibilidad; (2) Si no disponible, fallback a IndexedDB via `idb.rs` (ya existe bridge para IDB); (3) Agregar test en `wasm_tests.rs`.
- **Archivo:** `vantadb-wasm/src/opfs.rs`
- **Fuente:** UNI-04 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** 3 días

#### B2: WASM code-splitting
- **Qué:** WASM bundle único sin code-splitting. Plan existe en `docs/plans/2026-07-10-wasm-code-splitting.md`.
- **Investigar:** Leer plan existente; evaluar división: core vs. search features vs. maintenance.
- **Fuente:** P10 (FULL_CODEBASE_AUDIT)
- **Esfuerzo:** ~10h

#### B3: WASM bundle size — serde_json y tracing-wasm
- **Qué:** `serde_json` ~200KB extra en WASM bundle (debería ser feature flag opcional). `tracing-wasm` ~50KB extra (feature flag opcional).
- **Investigar:** Agregar feature flags en `vantadb-wasm/Cargo.toml` para excluir dependencias pesadas en builds minimal.
- **Fuente:** WA6/WA7 (FULL_CODEBASE_AUDIT)
- **Esfuerzo:** 2 días

#### B4: WASM multi-tab coordination
- **Qué:** `BroadcastChannel` detectado en `idb.rs` como dead code (`has_broadcast_channel()` sin callers). No hay coordinación cross-tab para acceso concurrente.
- **Investigar:** (1) Web Locks API para coordinación; (2) BroadcastChannel para notificación de cambios; (3) Worker dedicado para serializar acceso.
- **Fuente:** WA4, UNI-22
- **Esfuerzo:** 3 días

#### B5: WASM en main thread (no Worker)
- **Qué:** WASM corre en main thread bloqueando UI durante operaciones pesadas (index rebuild, compact, export).
- **Investigar:** Migrar a Web Worker con `comlink` o `wasm-bindgen` worker support.
- **Fuente:** UNI-39
- **Esfuerzo:** 3 días

#### B6: WASM NPM package not published
- **Qué:** `vantadb-wasm` no está publicado en NPM. Solo disponible como build local.
- **Investigar:** Configurar NPM publish en CI (`release-npm-61.yml`) — ya existe el workflow pero verificar que publique correctamente.
- **Fuente:** UNI-40
- **Esfuerzo:** 1 día

#### B7: WASM en Vercel demo
- **Qué:** No hay demo interactiva WASM publicada. Propuesta: Transformers.js + OPFS interactive showcase en landing page.
- **Investigar:** Build + deploy a Vercel; Integrar Transformers.js para embedding on-device; Mostrar benchmark en vivo.
- **Fuente:** ANA-17
- **Esfuerzo:** 2 días

#### B8: save() serializa O(n) completo
- **Qué:** WASM `save()` serializa estado completo sin dirty tracking. Cada autosave es dump completo de todos los nodos.
- **Investigar:** Implementar dirty flag tracking: solo serializar nodos modificados desde último save.
- **Archivo:** `vantadb-wasm/src/`
- **Fuente:** WA3
- **Esfuerzo:** 2 días

#### B9: Python — AsyncVantaDB sin límite de concurrencia
- **Qué:** AsyncVantaDB (`vantadb-python`) no tiene límite de concurrencia — thread pool saturation posible.
- **Investigar:** Agregar `Semaphore` o `max_concurrency` parameter.
- **Fuente:** PY1
- **Esfuerzo:** 1 día

#### B10: MCP — session cache tier
- **Qué:** MCP server solo tiene almacenamiento permanente (VantaDB). Le falta tier de session cache (volátil, rápido) estilo Cognee (Redis).
- **Por qué:** Agent memory necesita acceso rápido a contextos de sesión reciente sin hits a disco.
- **Investigar:** (1) Leer `docs/research/COGNEE_EVALUATION.md` (análisis completo); (2) Implementar `SessionCache` trait con backend Redis opcional y fallback HashMap in-memory; (3) Integrar con MCP tools.
- **Fuente:** COG-01/02 (COGNEE_EVALUATION.md)
- **Esfuerzo:** 3 días

#### B11: MCP — Claude Code plugin
- **Qué:** VantaDB MCP podría tener plugin dedicado para Claude Code con 6 lifecycle hooks (SessionStart, UserPromptSubmit, PostToolUse, Stop, PreCompact, SessionEnd).
- **Investigar:** (1) Crear `.claude-plugin/` structure; (2) Implementar hooks; (3) Session memory context injection middleware.
- **Fuente:** COG-11/12/13
- **Esfuerzo:** ~1 semana

#### B12: MCP — search_memory fallback silencioso
- **Qué:** `search_memory` usa Cosine como default cuando distance_metric no se especifica, sin advertir al caller.
- **Investigar:** Cambiar a `None` → error explícito "distance_metric required", o loggear warning.
- **Fuente:** MC1
- **Esfuerzo:** 30 min

#### B13: MCP — agent-scoped features
- **Qué:** Background task management, LLM usage tracking, abandoned session detection, agent lesson extraction.
- **Investigar:** Secuencia recomendada: session tracking → usage tracking → lesson extraction.
- **Fuente:** COG-06/07/08/09/10
- **Esfuerzo:** ~1 semana

#### B14: MCP — get_node_neighbors inconsistente
- **Qué:** Usa `storage.get()` directo en vez de pasar por `StorageEngine`.
- **Investigar:** Alinear con el patrón usado por otras tools MCP.
- **Fuente:** MC2
- **Esfuerzo:** 1 hora

#### B15: MCP — schema:// resource duplica metrics://
- **Qué:** `schema://` resource devuelve información similar a `metrics://` pero redundante.
- **Investigar:** Evaluar si consolidar o eliminar.
- **Fuente:** MC3
- **Esfuerzo:** 1 hora

#### B16: TS SDK hardening
- **Qué:** TS SDK tiene ~18 tests (debería 50+), faltan type stubs detallados, JSDoc incompleto.
- **Investigar:** (1) Expandir test suite; (2) Agregar type stubs para todas las APIs; (3) JSDoc completo.
- **Fuente:** ANA-24
- **Esfuerzo:** 3 días

#### B17: Integration crate boilerplate
- **Qué:** 9 integration crates (`vantadb-langchain`, `vantadb-crewai`, etc.) comparten patrón idéntico: `#[cfg(feature = "python")]` + `src/python.rs` ~200-300L cada uno.
- **Investigar:** Extraer a macro procedural o crate compartido `vantadb-integration-core`.
- **Fuente:** UNI-28
- **Esfuerzo:** 2 días

#### B18: Homebrew SHA256 placeholders
- **Qué:** `Formula/vantadb.rb:13` tiene 4 SHA256 `0000...` placeholders — fórmula no verificable.
- **Investigar:** Generar SHA256 reales de los artifacts release y actualizar formula.
- **Fuente:** S7
- **Esfuerzo:** 1 hora

---

### 🔵 CI/CD & DOCKER

#### C1: 5 workflows duplican Rust setup
- **Qué:** `fuzz-40`, `release-npm-61`, `release-wheels-60`, `release-adapters-62`, `release-binaries-63` tienen inline Rust setup duplicado. No usan composite action `./.github/actions/rust-setup`.
- **Investigar:** Extraer Rust setup a composite action y referenciarlo.
- **Fuente:** C1
- **Esfuerzo:** 1 día

#### C2: Fuzzing corpus sin merge cross-branch
- **Qué:** Fuzzing corpus se persiste vía cache por branch. No se propaga a otras branches ni se archivean crashes.
- **Investigar:** Agregar step en `fuzz-40.yml` para upload de corpus como artifact + merge script.
- **Fuente:** C3
- **Esfuerzo:** 1 día

#### C3: perf-bench markdown falla silenciosamente
- **Qué:** `perf-bench-40.yml` produce tabla con `*N/D*` cuando `update_markdown.py` falla. Falla no es fatal, pero datos corruptos.
- **Investigar:** Hacer que el script falle explícitamente (exit non-zero) si no puede parsear resultados.
- **Fuente:** C4
- **Esfuerzo:** 1 día

#### C4: Docker improvements
- **Qué:** `cargo-watch` reinstalado en cada `docker-compose.dev.yml up`. Skeleton build usa `echo "" > lib.rs` en vez de `fn main() {}`.
- **Investigar:** (1) Optimizar layer caching para cargo-watch; (2) Skeleton build correcto como `fn main() {}`.
- **Fuente:** DK3/DK4
- **Esfuerzo:** 1 día

#### C5: Sanitizer CI jobs (ASan + TSan)
- **Qué:** No hay ASan/TSan jobs en CI Rust. Previene detección de memory corruption y data races.
- **Investigar:** Agregar jobs en `ci-rust-10.yml` con `-Z sanitizer=address,thread`.
- **Fuente:** UNI-08, ANA-20
- **Esfuerzo:** 1 día

#### C6: Code coverage CI
- **Qué:** No hay code coverage upload en CI.
- **Investigar:** Usar `cargo-llvm-cov` + upload a Codecov/Coveralls.
- **Fuente:** UNI-09
- **Esfuerzo:** 1 día

#### C7: Pinned action SHAs sin Dependabot
- **Qué:** Actions pinned por SHA (correcto) pero sin Dependabot configurado para updates automáticos.
- **Investigar:** Agregar Dependabot config para actions (grupo semanal).
- **Fuente:** C6
- **Esfuerzo:** 1 hora
- **Estado:** ✅ Ya existe. `.github/dependabot.yml` tiene 4 ecosistemas: cargo, npm, github-actions, docker.

#### C8: release-binaries-63 usa toolchain setup distinto
- **Qué:** Usa `actions-rust-lang/setup-rust-toolchain` en vez del estándar `dtolnay/rust-toolchain` del resto de workflows.
- **Investigar:** Unificar a `dtolnay/rust-toolchain`.
- **Fuente:** C1-var
- **Esfuerzo:** 1 hora

---

### 🟢 DOCUMENTACIÓN

#### D1: mdBook adoption
- **Qué:** Se recomienda adoptar mdBook para documentación interna/dev, reemplazando la estructura plana actual de `docs/`.
- **Investigar:** (1) Leer `docs/research/DOCS_TOOLS_RESEARCH.md` análisis completo; (2) Convertir Obsidian wikilinks `[[Link]]` a markdown estándar `[Link](path.md)`; (3) Crear `book.toml` y `SUMMARY.md`; (4) Integrar `cargo doc`.
- **Fuente:** DOC-01/03/04/05/06
- **Esfuerzo:** ~1 semana

#### D2: Version inconsistencies (12 items)
- **Qué:** Múltiples docs con version desactualizada: `master-index.md` dice v0.3.0 (Cargo.toml: 0.2.0), `ADVANCED_TOKENIZER.md` snippet Cargo v0.1, `PYTHON_SDK.md` dice 3.11+ vs README badge 3.8+, `FAQ.md` v0.1.5 vs actual, `llms.txt` menciona "v0.4.0→v0.6.0" (actual v0.2.0), etc.
- **Investigar:** (1) Decidir canonical source of truth para version; (2) Usar `env!("CARGO_PKG_VERSION")` en docs generados automágicamente; (3) Corregir docs estáticos manualmente.
- **Fuente:** DC1-DC12 (FULL_CODEBASE_AUDIT), AP-35
- **Esfuerzo:** 1 día

#### D3: llms.txt en repo root
- **Qué:** `llms.txt` está en `web/public/` — debería estar en raíz del repo para que LLMs (Claude Code, Codex, etc.) lo encuentren. Crear también `llms-full.txt`.
- **Investigar:** (1) Mover+copiar a raíz; (2) Actualizar contenido con flat index, IDB fallback, auto-tune, nuevos adapters.
- **Fuente:** VAL-14, ANA-18, DC5
- **Esfuerzo:** 2 horas

#### D4: Arch-diagrams, OpenAPI spec, CLI reference
- **Qué:** Solo ASCII art en ARCHITECTURE.md. Sin OpenAPI/Swagger spec. CLI reference solo en README (insuficiente). HTTP_API.md (149L) muy breve.
- **Investigar:** (1) Diagramas formales (Mermaid o similar); (2) OpenAPI spec autogenerada del server; (3) CLI reference page dedicada.
- **Fuente:** AP-38/39/40
- **Esfuerzo:** 3 días

#### D5: ADRs insuficientes
- **Qué:** Solo 3 ADRs para todo el proyecto. Faltan: Fjall vs RocksDB criteria, HNSW params (M=16, ef_construction=200), RRF constant (k=60), PyO3 binding architecture, WASM strategy, governance model.
- **Investigar:** (1) Revisar `docs/archived-decisions/` y `docs/architecture/` para decisiones existentes; (2) Escribir ADRs faltantes.
- **Fuente:** AP-36
- **Esfuerzo:** 1 día

#### D6: Docs split (archivos >500L)
- **Qué:** `docs/progreso/README.md` (1529L), `docs/DESIGN_RULES.md` (709L), `docs/Backlog.md` (674L) sin fragmentar.
- **Investigar:** Fragmentar por sección.
- **Fuente:** AUD-06
- **Esfuerzo:** 1 día

#### D7: Spanish docs cleanup
- **Qué:** `docs/glosario/wal.md` mixed language, `docs/web/investigacion.md` español, `docs/DESIGN_RULES.md` español, `docs/archive/REPORTE_INVESTIGACION_Y_DECISIONES.md` español.
- **Investigar:** Decidir policy (all English o bilingüe aceptado); Unificar.
- **Fuente:** AUD-04/05
- **Esfuerzo:** 1 día

#### D8: Frontmatter estandarizado
- **Qué:** Varios archivos sin YAML frontmatter (`web/investigacion.md`, `references/troubleshooting.md`, `reviews/FINAL-REVIEW.md`, `DESIGN_RULES.md`).
- **Investigar:** Agregar frontmatter: title, type, status, tags, last_reviewed.
- **Fuente:** AUD-08
- **Esfuerzo:** 1 día

#### D9: Docs content gaps
- **Qué:** FAQ solo 4 preguntas. Sin deployment guide. Sin migration guides verificados (FROM_LANCEDB.md, FROM_CHROMADB.md existen pero contenido no verificado). Sin rustdoc expuesto en docs/. Sin integrations index page.
- **Investigar:** Expandir cada gap según prioridad.
- **Fuente:** AW-61, AUD-13, VAL-15, UNI-32, AUD-11
- **Esfuerzo:** 2-3 días

#### D10: Broken links y assets
- **Qué:** README referencia `docs/assets/demo_terminal.png` — verificar si existe. `docs/web/README.md` referencia `brand/BRAND_PLATFORM.md` (no existe). `master-index.md` referencia `web/brand/` (no existe). SECURITY.md, SUPPORT.md, CODE_OF_CONDUCT.md referenciados en README son 404 (`.github/` directory no existe).
- **Investigar:** (1) Verificar assets; (2) Corregir o remover broken refs; (3) Crear `.github/` directory con community files.
- **Fuente:** AP-33, AUD-15/18, AUD-02
- **Esfuerzo:** 1 día

---

### 🟣 TESTING & QUALITY

#### T1: Cross-browser WASM testing
- **Qué:** `wasm_tests.rs` existe con 30+ tests pero requieren `wasm-pack test --chrome`. No se ejecutan en CI. No hay tests para Firefox/Safari.
- **Investigar:** (1) Agregar job de wasm-pack test en CI; (2) Configurar múltiples browsers.
- **Fuente:** UNI-46 (VantaDB_RESEARCH_UNIFIED.md)
- **Esfuerzo:** 1 día

#### T2: Security testing — fuzzing + audit
- **Qué:** Fuzzing solo para parser IQL (4 targets existentes). No hay fuzzing para: API HTTP, inputs JSON, WASM bindings. Security audit tests: solo 1 file.
- **Investigar:** (1) Agregar fuzz targets para HTTP server endpoints; (2) Para WASM deserialization; (3) Expandir security tests.
- **Fuente:** UNI-34/38
- **Esfuerzo:** 2 días

#### T3: Snapshot testing + regression tests
- **Qué:** No hay snapshot testing para HNSW recall certification, export/import format, serialization format. No hay regression test suite para bugs ya fixed.
- **Investigar:** (1) Snapshot tests con `insta` crate para serialization format; (2) Regression test suite.
- **Fuente:** AP-30/31
- **Esfuerzo:** 2 días

#### T4: Frontend tests — 0
- **Qué:** No hay Vitest, RTL, o Playwright tests para componentes frontend. El stack las tiene configuradas pero sin tests.
- **Investigar:** (1) Agregar tests para componentes core (Nav, Hero, Pricing, Benchmark); (2) Playwright E2E para flow landing→docs→pricing.
- **Fuente:** AP-27
- **Esfuerzo:** 2-3 días

#### T5: Load/stress tests en TS/Python
- **Qué:** Solo Rust tiene stress tests. Python y TS SDKs no tienen load tests.
- **Investigar:** Agregar stress tests with concurrent operations.
- **Fuente:** AP-32
- **Esfuerzo:** 1 día

#### T6: Security audit tests expand
- **Qué:** 1 file de security audit tests. Faltan: IQL injection, auth bypass, path traversal validation, input validation en todos los bindings.
- **Investigar:** (1) Agregar tests para cada vector de ataque identificado; (2) Verificar que el rate limiting funciona.
- **Fuente:** UNI-34
- **Esfuerzo:** 2 días

#### T7: Cargo test-threads=2 global
- **Qué:** `.cargo/config.toml` tiene `test-threads = 2` global — aplica a Linux/macOS también, no solo Windows (donde era necesario por OOM).
- **Investigar:** Mover a per-platform en nextest config.
- **Fuente:** AP-43
- **Esfuerzo:** 30 min

---

### 🟤 RESEARCH ITEMS (requieren decisión/investigación antes de implementar)

#### R1: SQL — NO implementar
- **Recomendación definitiva de `docs/research/SQL_ANALYSIS.md`:** NO agregar SQL. Costaría 6-12 meses, diluye identidad ("SQLite for AI Agents" es tagline fuerte), expectations dilemma (SQL parcial genera quejas, SQL completo compite con SQLite 100M tests). No hay user demand — piden Python docs, LangChain, durability, WASM.
- **Acción:** Cambiar claim "SQL support" en web a "Programmatic API — no SQL needed".
- **Re-evaluar:** Q2-Q3 2027 si hay demanda real.

#### R2: Signed releases — Phase 1 (GitHub Attestations + GPG)
- **Qué:** Implementar GitHub Attestations (SLSA L2) para todos los artifacts release. Agregar SHA256SUMS.txt + GPG-sign. Phase 2: Windows signing cert (~$300-500/yr). Phase 3: macOS notarization ($99/yr).
- **Fuente:** `docs/research/SIGNED_RELEASES.md`
- **Esfuerzo:** 2 días (Phase 1)

#### R3: ACID — Análisis completo en ACID_TRANSACTIONS.md
- **Conclusión:** Approach B (Custom Transaction Layer sobre WAL existente) es el recomendado. Approach A (Fjall-only) rechazado (no cubre vector store/HNSW). Approach C (SQLite journal) rechazado (WAL existente es mejor base).
- **No implementar:** SQL-style multi-statement user transactions. VantaDB necesita single-batch atomicity + multi-key CAS.
- **Leer:** `docs/research/ACID_TRANSACTIONS.md` para análisis completo de 14 páginas.

#### R4: Cognee evaluation — MCP enhancements roadmap
- **Qué:** Evaluación completa de Cognee patterns contra VantaDB MCP. Recomendaciones priorizadas:
  1. **Session cache tier** (COG-01, ~3d) — Redis/FS volatile cache
  2. **Claude Code plugin** (COG-11, ~1w) — 6 lifecycle hooks
  3. **Session lifecycle tracking** (COG-03, ~2d) — SessionRecord
  4. **Agent trace persistence** (COG-04, ~2d) — tool calls as VantaDB entries
  5. **Session→permanent sync** (COG-05, ~3d) — importance scoring pipeline
- **Leer:** `docs/research/COGNEE_EVALUATION.md` para análisis completo.

#### R5: Docs tools — recomienda mdBook, difiere Starlight
- **Conclusión:** Adoptar mdBook para dev docs (Rust tool, markdown from Obsidian migra 1:1, integra rustdoc). Starlight para public docs site solo cuando haya demanda demostrada.
- **Leer:** `docs/research/DOCS_TOOLS_RESEARCH.md`

#### R6: Backlog items para eliminar
- **Fuente:** `docs/research/VantaDB_ANALISIS_COMPLETO.md` Sección 3.1 — 15 items para eliminar del backlog:
  - Semantic Kernel adapter (bajo uso en mercado)
  - Object pool PyDict (PERF-16 ya resuelve)
  - Cosine→Euclidean mapping (prematuro)
  - Prefetching HNSW (micro-optimization sin evidencia)
  - Norm caching (prematuro)
  - Async transcript I/O (no es hot path)
  - FilterBitset overhead (no es bottleneck)
  - Visual regression tests (Percy/Chromatic sin recursos)
  - WAL shipping replication (sin mercado)
  - PITR via archival WAL (enterprise sin demanda)
  - SOC2 prep (3-5d irreal, toma meses)
  - HIPAA assessment (sin negocio healthcare)
  - Multi-tenant isolation (no hasta Cloud)
  - All VantaDB Cloud items (product-market fit no validado)
  - Async ingestion pipeline (ya existe via channel)

---

### ⚪ SKILLS ECOSYSTEM (no-code items para maintenance)

#### S1: Consolidar skills duplicadas
- **Qué:** Skills ecosystem tiene ~40% waste (~80 skills a remover de ~190 únicas).
- **Items:** `minimalist-skill` = `minimalist-ui`, `redesign-skill` = `redesign-existing-projects`, `stitch-skill` = `stitch-design-taste`, `soft-skill` = `high-end-visual-design`, `threejs` local = `threejs-*` global suite, `prisma` basic = `prisma-expert`, `browser-use` = `agent-browser` + Playwright MCP, `gpt-taste` = `impeccable` + `design-taste-frontend`.
- **Eliminar:** Venice.ai suite (5 stubs), Fal.ai stub suite (10 de 14), `imagen` (5th image gen), `design-taste-frontend-v1` (migrar a v2).
- **Referencia:** `docs/reviews/FINAL-REVIEW.md` (Core 50 lista recomendada).

#### S2: Empty skill directories
- **Qué:** 9 directories en `.claude/skills/` sin `SKILL.md`: cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer.
- **Investigar:** Poblar o limpiar.

---

## WONTFIX

Items evaluados y decididos como no resolver:

| Item | Razón |
|------|-------|
| **routeTree.gen.ts @ts-nocheck** (640L sin typecheck) | Auto-generado por TanStack Router. No se edita manualmente. |
| **CSP en Rust HTTP server** | JSON API puro (3 rutas: /query, /health, /metrics). No sirve HTML. CSP no aplica. |
| **Adapters namespace fijo** | Namespace configurable como parámetro opcional — no blocking. |
| **SQL implementation** | 6-12 meses, diluye identidad, sin user demand. Ver R1. |
| **SOC2 / HIPAA cert** | Tomaría meses, sin negocio actual. Remover claims falsos de web. |
| **VantaDB Cloud** | Product-market fit no validado. Remover de web hasta tener MVP. |

---

---

## Julio 13 — Consolidación Doc + Verificación Cross-Code

**Qué se hizo:** 4 sub-agentes lanzados en paralelo para verificar cada claim del backlog, research docs, reviews, y planes contra el código real.

**Verificaciones:**
- ~150 claims de estado en backlog verificados contra `codegraph_explore`, `grep`, y lectura directa
- 9 research docs evaluados (3 ✅ vigentes, 3 ⚠️ parciales, 3 📝 pura investigación)
- 13 review docs evaluados (5 raw agent output, 3 superseded, 1 consolidado)
- 12 plan files evaluados (4 archivados por completados/abandonados)

**Archivados (17 archivos → `docs/archive/`):**
- 5 agent reports (raw data, consolidados en FINAL-REVIEW)
- 3 superseded audits (EXECUTIVE, WEB, Jul-09)
- 1 web-audit-report
- 4 cold research docs (SQL_ANALYSIS, COGNEE, DOCS_TOOLS, DOCS_AUDIT)
- 4 plan files (fragmentar-index, findings 3.1-3.6, webV2-astro)

**Backlog reescrito:** De 176 items a 48 open items verificados. Se agregaron 12 nuevos items (VFY-001→012) descubiertos durante la verificación cross-code.

**Master-index reparado:** Paths rotos corregidos (operations/ → strategy/ para SHOW_HN_PREP, operations/ → archive/ para EXECUTIVE_TECHNICAL_AUDIT, blog wikilinks rotos eliminados).

## Archivos Fuente de Referencia

| Dominio | Archivos |
|---------|----------|
| **Audit Técnico** | `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md` (755L, ~80 findings) |
| **Full Review Jul 13** | `docs/reviews/2026-07-13-full-review.md` |
| **Análisis de Proyecto** | `docs/reviews/analisis_proyecto.md` (~50 findings) |
| **Backlog Activo** | `docs/Backlog.md` (48 items verificados) |
| **ACID Transactions** | `docs/research/ACID_TRANSACTIONS.md` |
| **Signed Releases** | `docs/research/SIGNED_RELEASES.md` |
| **Research Unificado** | `docs/research/VantaDB_RESEARCH_UNIFIED.md` |
| **Research Validado** | `docs/research/VantaDB_RESEARCH_VALIDADO.md` |
| **Análisis Completo** | `docs/research/VantaDB_ANALISIS_COMPLETO.md` |
| **FINAL-REVIEW (skills)** | `docs/reviews/FINAL-REVIEW.md` |
| **Archivo General** | `docs/archive/` (17 documentos históricos) |
