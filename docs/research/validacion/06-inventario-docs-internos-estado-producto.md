---
title: "06 — Inventario de docs/ internos y estado real del producto"
type: research
status: complete
date: 2026-08-25
author: "Agente F — analista producto/documentación"
method: "Glob completo docs/**/*.md + lectura de 16 docs clave + verificación ligera contra código (codegraph)"
scope_read_only: true
---

# Inventario de docs/ internos y evaluación del estado real del producto

**Fecha:** 2026-08-25 · **Fuente:** lectura directa de docs/ + contraste con código (`src/sdk/api.rs`, `src/graphrag/pipeline.rs`, `vantadb-python/src/lib.rs`, `desktop/src/transport.ts`).

---

## 1. Mapa de docs/

~570 archivos .md, 27 carpetas + 11 archivos raíz. El vault es también vault Obsidian (`.obsidian/` commiteado).

```
docs/
├── master-index.md        Índice maestro con regla "indexa en el mismo PR" (last_reviewed 2026-08-22)
├── README.md              Landing del vault (stale: cita progreso/ migrado)
├── QUICKSTART.md          Onboarding 5-min (Python wheel + npm, medido ~6s/~2s)
├── FAQ.md                 Preguntas frecuentes + ejemplos Rust
├── COMPARISON.md          Comparativa honesta vs sqlite-vec/LanceDB/Qdrant/Chroma
├── CHANGELOG.md           Keep-a-Changelog; [Unreleased] acumula ~700 líneas desde 0.5.0
├── Backlog.md             Catálogo único de tareas (118 abiertas, reconteo 2026-08-25)
├── backlog-futuro.md      Items diferidos
├── TEST_MAP.md            "Si cambias X, corre Y" + gates CI (fecha 2026-07-22, stale)
├── ci-cd-guide.md         Guía CI/CD · chaos-testing.md estrategia de caos
├── api/ (12)              Referencias: Embedded SDK, Python, TS/WASM, HTTP+OpenAPI, MCP (stub), IQL, GraphRAG, namespaces
├── architecture/ (53)     ARCHITECTURE.md, diseños (text index, WAL, tokenizer, storage versioning), FND-*, 10 ADRs
├── avance/ (68)           Registro vivo de progreso POR DOMINIO (core, bindings, web, ci-cd, ops, desktop, seguridad)
├── plans/ (56)            Planes de campaña (14 activos + 46 archivados)
├── research/ (81)         Investigaciones INV-/FND-/TIR-/competencia (incl. este archivo)
├── reviews/ (57)          Auditorías y certificaciones (/audit, unified-review)
├── reports/ (4)           DORA, northstar, pipeline evals (telemetría del PROCESO)
├── operations/ (34)       CONFIGURATION, BENCHMARKS, DURABILITY, SECURITY, performance, release policies, pilot program, GRAFANA, telemetría
├── workflow/ (15)         Espejo documental de .github/workflows (ci-rust, releases, fuzz, chaos…)
├── strategy/ (7)          ROADMAP (histórico), GO_TO_MARKET, VANTADB-PRO-FEATURES/DELIVERY, SHOW_HN_PREP, REDDIT_POSTS
├── vision/ (1)            VISION.md — posicionamiento, UVP, ICP, métricas
├── tutorials/ (8)         Learning path: agent memory, RAG, hybrid search, migraciones Chroma/LanceDB/Vectara, embeddings
├── blog/ (7)              Posts publicados (motivación, hybrid search, benchmarks, GraphRAG…)
├── book/ (75)             mdBook narrativo de internals (incluye build HTML commiteado)
├── glosario/ (57)         Glosario bilingüe término-por-archivo
├── benchmarks/ (5)        COMPETITIVE_ANALYSIS, SDK bench, IVF notes
├── desktop/ (3)           Guía/arquitectura del app Tauri
├── web/ (27)              Docs del frontend Next.js (guides, standards, QA)
├── discord/ (4)           Config comunidad (server activo, onboarding pendiente DISC-01)
├── graphrag/ (1), wasm/ (1), references/ (3), archive/ (6), _templates/ (4)
└── examples/               Código runnable citado por tutoriales
```

**Observaciones estructurales:** dos sistemas de progreso coexisten (progreso/ legacy citado por README vs avance/ canónico); build de mdBook commiteado; `.obsidian/` personal dentro del repo. La auditoría GOV del 21-08 ya detectó esto y hay campaña activa (30 tareas).

---

## 2. Síntesis de los 16 documentos leídos

1. **master-index.md** — Índice global con regla de mantenimiento estricta ("todo doc nuevo se indexa en el mismo PR"). Navegación completa por 20 secciones + lista de exclusiones deliberadas. Fue encontrado congelado en julio por la auditoría 21-08; hoy muestra `last_reviewed: 2026-08-22` (GOV lo revivió parcialmente).
2. **Backlog.md** — Fuente única de tareas: **118 items abiertas** (reconteo GOV-C7 25-08). Fases P0–P4, P7, P11–P15 cerradas (release 0.5.0, security UAF, coverage, WASM, desktop 17/17, Vanta Studio 4 fases ✅). Lo abierto es principalmente *features futuras*: P27 TDAM/Vanta Memory Engine (38), GOV gobernanza docs (30), P25 exposición MCP/HTTP (11), P38 research huérfanas (17), P6 launch campaign (11). **No está "atrasado" — está cargado de roadmap adelante.**
3. **CHANGELOG.md** — v0.5.0 publicada ~31-07; `[Unreleased]` acumula ~700 líneas: Vanta Studio completo, REST `/api/v2/*` (~29 paths), crate `vanta-memory`, supersession pública (ADR-028), fixes de seguridad. **Sin corte de release en ~3.5 semanas → los paquetes públicos no reflejan el producto actual.**
4. **COMPARISON.md** — Comparativa con reglas anti-fumo: features de competidores solo si verificables en sus repos, números propios solo de BENCHMARKS.md con comando de reproducción, cero cifras ajenas. Publica límites propios concretos (top_k ≤1000, ingesta HNSW single-threaded, cosine mejor métrica, ~6.5 GB RAM estimados por 1M×1536d). Madurez de honestidad sobresaliente.
5. **QUICKSTART.md** — Onboarding real y funcional: pip wheel → primera query híbrida con vectores toy offline; Ollama/OpenAI opcionales; TS vía npm/WASM; CLI export/audit. Mide TTFTQ propio: **~6 s Python, ~2 s TS**. Declara explícitamente el boundary del MVP (no cubre IQL, MCP, graph, cloud). Calidad alta.
6. **FAQ.md** — Respuestas correctas y honestas ("production-ready? → 0.5.0 bajo desarrollo activo"); Homebrew comentado porque no existe. Stale menor: usa `import vantadb_py` mientras QUICKSTART canonicaliza `import vantadb`.
7. **vision/VISION.md** — Posicionamiento "The SQLite for AI Agents"; UVP = fragmentación (vectores/docs/grafos) resuelta en un motor transaccional embebido. ICP primario: developer de agentes con LangChain/LlamaIndex que valora privacidad local. **Métricas revelan brecha brutal:** targets 6 meses = 1.000 stars / 10K descargas/mes / 500 Discord; actual = **3 stars / 347 descargas / ~50 miembros / 0 pilotos / $0 ARR**. Claims de impacto (GraphRAG −40–60% tokens, recall 15–20% mejor) tienen base de medición débil o única.
8. **strategy/ROADMAP.md v2.0** — Autodeclarado **histórico**: las fases Sem 1–16 ya se ejecutaron (real = 0.5.0, no 0.2.0). Riesgos vigentes: R4 MSVC linker overflow (bloquea build Windows workspace), R6 SQ8 no expuesto en hot path de búsqueda (benchmarks no competitivos a escala), R7 HNSW rebuild en cada startup (30–60 s @1M → mata serverless/edge).
9. **strategy/GO_TO_MARKET.md** — Canales activos: PyPI, crates.io 0.5.0, GitHub Releases, npm. Integraciones Tier 1 (LangChain, LlamaIndex, MCP) marcadas "implemented, pending PyPI + PR upstream". Cloud beta Fly.io: checkbox vacío, cero infra. Pricing definido (tiers web) sin producto cloud detrás.
10. **strategy/SHOW_HN_PREP.md** — Draft de Show HN listo con matriz defensiva de 10 críticas. Notas de higiene de claims recientes (23-08): "zero-dependency" corregido (croaring/rocksdb compilan C++), claim "recall >0.998 SIFT1M" **eliminado por no medible** (lo real: 0.9975@ef_400 en sintético 10K×128d). Post bloqueado implícitamente por MKT-04 ("corregir claims primero").
11. **strategy/VANTADB-PRO-FEATURES.md** — Open Core: gates existentes en el core (encryption, wal-shipping, prometheus, server, tls; pitr **removida** por dead code) catalogados como "candidatos Pro conceptuales". Repo privado `vantadb-pro` existe pero solo contiene licenciamiento; 6 features Pro (RBAC, multi-tenancy, replicación…) **sin código** (P23).
12. **avance/README.md** — Índice del árbol de progreso por dominio, migrado desde progreso/ (225 filas limpiadas). Convención ✅/⚠️/❌ verificada contra código. Las carpetas vivas (plans/, reviews/, research/) quedan fuera físicamente porque pipelines escriben ahí; se catalogan en `fuentes-vivas.md`.
13. **reports/northstar.md** — Telemetría del *sistema de tareas* (no del producto): 37 tareas completadas, 100% primer intento, 1 falso positivo, 0 regresiones. Indica disciplina de ejecución alta.
14. **TEST_MAP.md** — Mapa contribuidor "si cambias X corre Y": gates CI serios (clippy -D warnings, coverage ≥80% root ADR-018, Miri, ASan/TSan, cargo-deny, fuzz nightly). Data de 2026-07-22; la auditoría 21-08 encontró un filtro nextest inefectivo asociado (hallazgo 🔴 #1).
15. **reviews/auditoria-documentacion-2026-08-21.md** — Auditoría integral de 490 md: **doc↔código 9/10** (14/16 claims verificados con archivo:línea y commit hash; 0 falsos), **gobernanza 4–5/10** (índices congelados, Backlog contradictorio, CHANGELOG sin cortar, doble taxonomía). Salud global 6.5/10. Varias de sus quejas ya muestran corrección posterior (reconteo Backlog 25-08, master-index 22-08) → campaña GOV funcionando.
16. **README.md (docs)** — Landing del vault; envejeció mal: todavía apunta a `progreso/bitacora.md` (migrado a avance/) y omite carpetas nuevas.

---

## 3. Estado real: implementado vs planificado

Verificado contra código (codegraph: `VantaEmbedded`, `search_memory`, `GraphRagPipeline`, transports desktop, bindings):

| Feature | Docs dicen | Código | Veredicto |
|---|---|---|---|
| Motor embebido transaccional (put/get/list, namespaces, TTL, metadata, WAL+CRC32C) | EMBEDDED_SDK.md, DURABILITY_GUARANTEES | `src/sdk/api.rs`, `src/sdk/types.rs` | ✅ **Implementado** |
| Hybrid search BM25+HNSW+RRF (métodos, explain, profiles, clamp top_k) | API docs, blog deep-dive | `search_with_method`; binding Python `search_memory` (sync+async, GIL released) | ✅ **Implementado** |
| Grafo tipado: BFS/DFS, DAG check, PageRank, topological sort | GRAPH_RAG.md, EMBEDDED_SDK | `graph_is_dag`, pagerank… en core y bindings | ✅ **Implementado** |
| GraphRAG pipeline (seed→expand→rank→context_text) | GRAPH_RAG.md, blog benchmark | `src/graphrag/pipeline.rs` + builder; bench de indexación OK, **query bench PENDING por stack overflow** (changelog) | ✅ con asterisco |
| IQL query language | api/IQL.md | `engine.query()` + `/api/v2/query` | ✅ Implementado |
| Python SDK (sync + async, migrador ChromaDB) | PYTHON_SDK.md | `vantadb-python/`: clase sync + wrapper async + `migrate/chroma.py` | ✅ Implementado |
| TS/WASM SDK con persistencia OPFS/IndexedDB | TS_SDK, WASM_PERSISTENCE | `vantadb-ts`, `vantadb-wasm` (`connect_persistent`, `save_idb`) | ✅ Implementado |
| Node nativo (napi-rs) | changelog COMP-029 | `vantadb-node/src/lib.rs` | ✅ Implementado |
| Servidor HTTP REST `/api/v2` (~29 paths) + OpenAPI | HTTP_API.md, openapi.yaml | crate server + e2e metrics | ✅ Implementado |
| App desktop Tauri + consola Vanta Studio (Tauri/HTTP/WASM transports) | desktop/*, avance/desktop | `desktop/src-tauri/connections/`, `transport.ts` triple backend, UI | ✅ Implementado (deuda: smoke-test instalador en VM limpia) |
| MCP server | MCP.md = **stub**, SSOT en `skills/vantadb-mcp/` | crate `vantadb-mcp` con tests; experimental | ⚠️ Implementado experimental, doc pública insuficiente (P25: 11 tareas) |
| Adapters LangChain/LlamaIndex/Mem0/CrewAI/DSPy "first-class" | GTM: "implemented, pending PyPI" | Código en `integrations/`, **404 en PyPI** (MKT-18f) | ❌ **Documentado > publicado** |
| Auto-embedding nativo (`remote-inference`) | CONFIGURATION, QUICKSTART nota | Feature-flag Rust-only; **no expuesto en SDKs Python/TS** | ⚠️ Parcial |
| SQ8 cuantización en hot path de búsqueda | ROADMAP R6 | Solo tipo, no hot path (COMP-001, catálogo) | ❌ Planificado |
| HNSW persistido sin rebuild al startup | ROADMAP R7 | Rebuild 30–60 s @1M (COMP-002) | ❌ Planificado |
| Wheels ARM64 Linux + Homebrew útil | GTM | wheels solo x86_64; Formula SHA `0000…` placeholder (MKT-18h) | ❌ Pendiente |
| VantaDB Cloud (Fly.io beta) | GTM checkbox | Cero infra (CLD-01) | ❌ No existe |
| Features Pro (RBAC, multi-tenancy, replicación, audit) | PRO-FEATURES | Repo privado solo licencia; 6 features sin código (P23) | ❌ Futuro |
| TDAM / Vanta Memory Engine (personas, wiki, decay) | Backlog P27 MEM-01..38 | `vanta-memory` crate inicial existe; 38 tareas de diseño pendientes, "decisión de producto" 🔴 | 🟡 Diseño |
| PITR | Histórico | Feature flag removida por dead code (FIND-26) | ❌ Removida |

---

## 4. Valor actual del producto

**Qué resuelve HOY.** Un desarrollador de agentes IA que quiere memoria persistente local-first instala `pip install vantadb-py` (~6 s) y obtiene: almacenamiento transaccional embebido con durabilidad seria (WAL + fsync + CRC32C validado por chaos/failpoints), búsqueda vectorial HNSW, lexical BM25 y fusión RRF en una sola llamada, grafo con PageRank/traversals, GraphRAG que produce `context_text`, IQL, export JSONL + CLI de auditoría, y opcionalmente servidor HTTP, MCP experimental, y una consola desktop/browser (WASM+OPFS). Eso es real, testeado (coverage ≥80%, Miri, fuzz) y honestamente documentado.

**Para quién.** El ICP declarado (developer de agentes con LangChain/LlamaIndex, privacy-first) es correcto como *destino*, pero el usuario servible HOY es más estrecho: quien programa directo contra la API nativa (Python/TS/Rust) y no depende de integraciones de framework — porque esas no están publicadas.

**Propuesta de valor implícita actual.** "El SQLite de la memoria de agentes" en su fase MVP 0.5.0: motor excelente y verificado, distribución embrionaria. La ventaja diferencial real y defendible hoy es la combinación híbrida+grafo+durabilidad certificable en-proceso; nadie más la ofrece junta en embebido.

**Promesa vs realidad (huecos de credibilidad):**

| Promesa (docs) | Realidad |
|---|---|
| "First-class integrations" con 5 frameworks | Código existe; **ninguna en PyPI** — el ICP primario no puede usarlas |
| "Reduce tokens 40–60% vía GraphRAG" | Una medición interna (~50%), query-bench del pipeline pendiente por stack overflow |
| "Sub-millisecond latency" | p50 1.2–1.5 ms a 10K; 6.1 ms a 50K; ingesta single-threaded |
| "Time-to-first-query <2 min" | Quickstart mide ~6 s (install+query) pero VISION reporta "~3 min" — definiciones inconsistentes |
| "Ecosystem moat: comunidad" | 3 stars GitHub, 347 descargas/mes, ~50 Discord, 0 pilotos enterprise |
| Desktop/Studio completo en changelog | Solo en git; **los usuarios de PyPI/crates 0.5.0 no reciben nada de esto** |

La ingeniería va muy por delante del marketing y de la distribución. El riesgo dominante ya no es técnico: es que un producto superior muere invisible (Show HN redactado pero no lanzado; Reddit drafts nunca publicados; trademark no registrado).

---

## 5. Huecos de producto priorizados

1. **🔴 Distribución del ecosistema (MKT-18f/g/h):** publicar adapters en PyPI + npm + PRs upstream, wheels ARM64, Homebrew con SHA reales. Sin esto, toda la promesa "first-class integrations" es humo y el ICP no puede adoptar. Es el hueco #1 absoluto.
2. **🔴 Cadencia de release:** cortar 0.6.0 con semver-checks. ~700 líneas de Unreleased significan que TODO lo documentado reciente (REST v2, Studio, supersession, fixes de seguridad) es invisible para usuarios de paquetes.
3. **🟠 Superficie MCP como producto:** la API reference vive en `skills/vantadb-mcp/` (interna) y MCP.md es stub. MCP es EL canal de adopción para Cursor/Claude Code; falta doc pública por-IDE + estabilización (P25) + ejemplo end-to-end.
4. 🟠 **Onboarding hacia frameworks:** tutoriales buenos para API nativa y migradores (Chroma/LanceDB/Vectara), pero ningún quickstart "agente LangChain funcional desde PyPI" ni demo GUI enlazada desde landing para no-developers.
5. 🟡 **Prueba social verificable:** case studies archivados como "no verificados"; falta UN caso real (pilot CLD-04) y un benchmark a escala (SIFT1M) para sostener claims de rendimiento ante HN.
6. 🟡 **Historia de observabilidad usuario-final:** GRAFANA_SETUP/MEMORY_TELEMETRY son diseños internos; no hay guía "cómo monitorizo mi VantaDB en producción" orientada a usuario.
7. 🟡 **Higiene de índices:** README.md y TEST_MAP.md stale; doble sistema de progreso; `.obsidian/` y build de mdBook commiteados — ruido para contribuidores externos (la campaña GOV ya lo está atendiendo).
8. 🟡 **Decisión de producto TDAM (P27):** 38 tareas de mayor esfuerzo del backlog dependen de una decisión que sigue abierta; mientras esté abierta, compite por atención con launch (P6) que es lo que genera tracción.

---

## 6. Conclusiones

- **docs/ es enorme (≈570 md), bien intencionado y anómalo en honestidad**: COMPARISON y SHOW_HN_PREP eliminan claims no verificables; la auditoría 21-08 confirma paridad doc↔código 9/10. Esto es una fortaleza competitiva en sí misma.
- **El producto implementado supera con creces al producto percibido**: core + SDKs + desktop + REST v2 listos; lo que falta es el último kilómetro (publicar, cortar release, lanzar Show HN, MCP doc).
- **La tracción (3★/347 descargas) contradice la calidad**: la inversión marginal más rentable no es otra feature (ni TDAM, ni Pro) sino distribución: adapters en PyPI, release 0.6.0, Show HN con claims ya saneados.
- **Riesgos técnicos heredados vigentes** que condicionan claims futuros: SQ8 no expuesto (escala), HNSW rebuild al startup (edge/serverless), MSVC linker (build Windows full-workspace).
- La gobernanza documental estaba enferma (6.5/10) pero la campaña GOV muestra corrección activa y verificable (reconteos 22–25-08); mantener esa cadencia es barato y protege la credibilidad ganada.

<!-- ponytail: inventario basado en 16 lecturas profundas + glob completo; carpetas menores (book/, web/, discord/) caracterizadas por índices, no lectura exhaustiva -->
