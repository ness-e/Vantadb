# INV-vantadb-ts-01 — Investigación profunda: SDK TypeScript/WASM multi-runtime (`vantadb-ts`)

**Fecha:** 2026-08-25 · **Comando:** `/research vantadb-ts` · **Modo:** read-only
**Registro:** `.opencode/references/research-modules.md` fila `vantadb-ts`
**Fuentes:** codegraph (67 símbolos), lectura completa `vantadb-ts/src/*` + package.json + README, npm registry API (2026-08-25), GitHub API vía `gh`, README oficial Orama, reviews previas (`docs/reviews/modulos/vantadb-ts.md` 2026-08-22, score 7.0).

---

## 1. Usuarios objetivo y flujo diario

**Devs JavaScript en 4 runtimes** (Node ≥22.12 backend, Bun, Deno, browser frontend):

| Usuario | Descubrimiento | Flujo diario | Fricción específica |
|---|---|---|---|
| Node backend dev | npm search / blog posts | `npm i vantadb` → `VantaDB.create()` o `NativeVantaDB.connect(path)` | `require(esm)` solo Node ≥22.12; `engines` no declarado en el tarball publicado (H-04); `NativeVantaDB` roto hasta publicar `vantadb-node` (H-14) |
| Browser/frontend | CDN / docs site | bundler + plugin WASM | necesita `vite-plugin-wasm` (documentado); sin entrada CDN zero-install como Orama (H-10); binario ~1.3 MB sin estrategia lazy-load documentada para consumidores |
| Agente/AI app dev | ejemplos LangChain/LlamaIndex/Vercel AI | embeddings propios → `put`/`search` | semántica `search()` divergente entre SDKs (porting hazard documentado pero fuente de confusión) |

Evidencia externa de fricción del ecosistema: issues de DuckDB-WASM sobre tamaño de bundle y VSS ausente en WASM; vectra single-maintainer sin persistencia transaccional; Orama plugin-data-persistence existe porque su modelo es in-memory puro.

## 2. Estándares del ecosistema npm (2026)

- **Doble condición exports** (`import`+`require`) es el default; paquetes ESM-only apuntan ambas al mismo archivo cuando el grafo es síncrono (ya aplicado — FIND-10 ✅).
- **`engines` debe sobrevivir la publicación**: nuestro tarball lo pierde (H-04).
- **Prebuilds/platform packages** son el estándar napi-rs; irrelevante acá (WASM), pero el patrón "smoke-test del tarball publicado" sí aplica (H-08).
- Cambio reciente relevante: Node 22 LTS consolidó `require(esm)` estable — la decisión ESM-only sigue siendo correcta.

## 3. Competidores — matriz (datos verificados 2026-08-25)

| | **vantadb (nosotros)** | @orama/orama v2.0.6 | vectra v0.15.0 | wa-sqlite v1.0 + sqlite-vec v0.1.9 | @duckdb/duckdb-wasm v1.33-dev |
|---|---|---|---|---|---|
| Backend | Rust→WASM (HNSW+BM25+grafo+IQL) | JS puro (índices invertidos+vector) | JS puro file-backed | SQLite C→WASM + vec0 virtual table | DuckDB C++→WASM |
| Persistencia | ✅ OPFS/IndexedDB snapshots + WAL (browser); fjall real vía native backend | ⚠️ in-memory + plugin persistence (serializa a JSONL) | ⚠️ archivos JSON planos | ✅ SQLite file/OPFS | ⚠️ in-browser (sin ANN indexado) |
| Búsqueda híbrida | ✅ RRF vector+BM25 nativo | ✅ mode:'hybrid' | ❌ solo vector | ⚠️ FTS5+vec0 por separado (join manual) | ❌ sin ANN (VSS no compila a WASM) |
| Grafo/traversal | ✅ BFS/DFS/topo/DAG/filtrado + IQL | ❌ | ❌ | ❌ | ❌ (SQL recursivo manual) |
| Errores tipados | ✅ `VantaError{code}` estructural | parcial (códigos internos) | ❌ | ❌ (SQLite codes) | ❌ |
| Facets/geosearch/typo-tolerance | ❌ | ✅✅✅ | ❌ | FTS5 parcial | SQL |
| Instalación browser | npm + plugin bundler (~1.3MB wasm) | ✅ CDN ESM directo (`cdn.jsdelivr.net/.../+esm`) | Node-only | wasm multi-MB | wasm multi-dezena de MB |
| Licencia | Apache-2.0 | Apache-2.0 | MIT | MIT / MIT-OR-Apache | MIT |
| Actividad | repo privado, push diario | ★10,530 · push 2026-08-04 · 21 issues abiertos | single maintainer | ★1,406 (wa-sqlite) · activo | ★2,105 · 186 issues · versión -dev churn |
| Downloads/semana (npm) | **12** | scoped @orama/orama (no verificado el número exacto; unscoped legacy=270) | 35,103 | 16,151 / 1.86M (sqlite-vec dist) | 465,648 |

**Claims de performance:** ninguno de los competidores publica benchmarks reproducibles en su README (Orama muestra "21μs" de un ejemplo trivial sin dataset — *claim sin evidencia* per Regla 11). Nosotros tampoco publicamos números del path JS/WASM (H-11). Empate honesto: nadie tiene evidencia reproducible pública en este nicho.

### Diferenciación vs Orama (competidor principal)

**Nuestra ventaja real:** único SDK JS embebido que combina (a) persistencia durable con WAL en browser (OPFS/IDB), (b) búsqueda híbrida RRF nativa, (c) grafo con traversals + DSL (IQL), (d) errores tipados estructurales. Orama es FTS-first con vector añadido; no tiene grafo ni durabilidad nativa.

**Su ventaja real:** adopción (3 órdenes de magnitud), ecosistema de plugins (embeddings, secure-proxy, persistence, analytics), features de search-engine maduro (facets, geosearch, typo tolerance, boosting, 30 lenguajes), distribución CDN zero-install, GenAI Answer Session.

## 4. Estado actual interno (evidencia file:line)

- **API surface:** 38 métodos públicos flat + 4 sub-clientes (`memory/graph/wiki/system`) — verificado contra `docs/api/BINDINGS_NAMESPACES.md:85`.
- **Tests:** 261 casos en 9 archivos vitest (`src/__tests__/`), incl. hardening + integración contra WASM. Coverage vía c8.
- **Empaquetado:** tarball publicado 81 KB (glue TS) + dep `vantadb-wasm ^0.5.0` (el rewrite de `file:` funciona — verificado contra registry). ⚠️ `engines` se pierde en publicación (H-04).
- **Docs:** `vantadb-ts/README.md` (333 líneas, excelente), `docs/api/BINDINGS_NAMESPACES.md` (243 líneas), `examples/` (vercel-ai, langchain, llamaindex).
- **Historial:** REL-02 (publicado 0.5.0 ✅), FIND-10 (require(esm) ✅), PERF-08 (Float32Array zero-copy ✅), COMP-029 (native.ts wrapper ✅).
- **Performance:** sin benchmarks reproducibles del path JS/WASM (H-11).

## 5. Framework de evaluación

| Dimensión | Score | Justificación |
|---|---|---|
| DX onboarding | 8.5 | README ejemplar, quickstart medido 1.6s (FND-05), errores tipados |
| Completitud funcional | 6.5 | faltan `remove_edge`, `count`, versionado, batch search, sparse_vector, wiki vacío (H-09/H-12) |
| Performance/overhead | 6.0 | zero-copy logrado, pero cero datos publicados (H-11); sync API limita concurrencia en Node |
| Robustez | 6.0 | 3 bugs críticos de la review 2026-08-22 **siguen vivos** (H-01/H-02/H-03) |
| Seguridad | 8.0 | sandbox WASM, sin eval, errores estructurales; npm audit dev-only |
| Docs & ejemplos | 9.0 | mejor doc de los bindings; paridad cross-SDK documentada |
| Observabilidad | 7.5 | operationalMetrics/capabilities expuestos; tracing-wasm opcional |
| Testabilidad | 8.5 | 261 tests + coverage; sin gate CI (H-07) |
| Paridad otros módulos | 6.0 | batch search Python-only; caps divergentes wasm/node; IDs grafo number\|bigint vs strings |
| Diferenciación | 7.0 | nicho único (memoria+durable+híbrido+grafo) pero invisible (12 dl/sem, H-06) |

**Score global: 7.2/10** — el SDK mejor documentado del proyecto, con arquitectura sólida; penalizado por deuda crítica sin cerrar (MOD-22/23/24 nunca ejecutadas), huecos de API obvios e invisibilidad de mercado.

---

## Gap analysis priorizado

### Falta (P0)
1. Tipos de grafo ficticios + bug async `_native` + verificación semántica score/distance (H-01..H-03 — ya diagnosticados hace 3 días, sin tarea en Backlog).
2. Smoke-test de instalación del tarball publicado (H-08).

### Mejorable (P1)
3. Gate CI para los 261 tests (hoy Tier 3 sin gate — regresiones silenciosas).
4. `engines` en el tarball publicado + guía de migración Node<22.
5. Entrada CDN ESM para evaluación browser sin bundler.

### Optimizable / Estratégico (P2)
6. Benchmarks reproducibles del path JS/WASM.
7. Estrategia de distribución/adopción (playground, docs site, comparativas honestas).
8. Decisión de publicación de `vantadb-node` (desbloquea NativeVantaDB).

### Quick wins (<1 día)
H-02 (fix 3 líneas), H-04, H-05 (restaurar filas Backlog), H-07, H-08, H-10 (jsdelivr +esm ya funciona con paquetes ESM estándar — verificar y documentar).

### Apuestas estratégicas (>1 semana)
H-06, H-12 (wiki/threads/conversation via WASM), H-13 (posicionamiento), H-14.

---

## APÉNDICE OBLIGATORIO — Inventario de hallazgos (entrada Fase D)

| ID | Hallazgo | Categoría sugerida | Severidad | Esfuerzo | Evidencia |
|---|---|---|---|---|---|
| H-01 | Tipos de grafo ficticios: `GraphBfsResult{visited,levels,path}` no corresponde al wire format (`u128[]` plano); blind-cast `as GraphBfsResult`; consumers reciben `undefined` en `.visited` | APLICAR | 🔴 alta | 🟡 | `vantadb-ts/src/types.ts:208-212`, `vantadb-ts/src/vantadb.ts:1094` (=review #1/MOD-22, vivo hoy) |
| H-02 | `NativeVantaDB._native` captura solo throws síncronos; rechazos async escapan sin envolver en `VantaError` | APLICAR | 🟠 media | 🟢 | `vantadb-ts/src/native.ts:89-95` (=review #2/MOD-23, vivo hoy) |
| H-03 | Semántica `score`(similitud) vs `distance`(distancia) mapeada sin verificar contra core; docs ahora dicen "distance" pero el mapeo `h.score → hit.distance` no está verificado | APLICAR | 🟠 media | 🟡 | `vantadb-ts/src/types.ts:71-77`, `vantadb-ts/src/native.ts:304` (solapa RES-06) |
| H-04 | Campo `engines:{node:>=22.12}` existe en repo pero es `null` en el tarball publicado → `require(esm)` falla con error confuso en Node<22 sin declaración previa | MEJORAR | 🟡 baja | 🟢 | registry.npmjs.org/vantadb/latest `engines:null` vs `vantadb-ts/package.json:6-8` |
| H-05 | Trazabilidad rota: review 2026-08-23 afirma derivación de MOD-22/23/24 a Backlog fase P32 pero las filas NO existen → pérdida de datos en pipeline | APLICAR | 🟠 media | 🟢 | `rg MOD-2[234] docs/Backlog.md` → 0 hits; `docs/reviews/modulos/vantadb-ts.md:105-111` |
| H-06 | Adopción 12 downloads/semana vs 35K-465K de competidores; producto técnicamente superior en su nicho pero sin canal de distribución (playground/docs-site/comparativas) | ESTRATEGIA | 🟠 media-alta | 🔴 | npm api.npmjs.org/downloads 2026-08-25 |
| H-07 | Sin gate CI para los 261 tests del SDK (TEST_MAP: "no CI gate for TS SDK") — regresiones pueden mergear silenciosamente | MEJORAR | 🟠 media | 🟢 | `docs/operations/TEST_MAP.md:18,137` |
| H-08 | El rewrite `file:`→`^0.5.0` del publish funciona hoy pero no hay smoke-test automatizado de "instalar tarball y correr quickstart" — drift silencioso posible en próximo release | MEJORAR | 🟡 baja | 🟢 | registry deps `^0.5.0` verificado; sin test en CI |
| H-09 | Huecos API vs core y vs Python SDK: `remove_edge` (¡hay add sin remove!), `count`, `versions/supersede`, `sparse_vector` siempre None, `filter_ops`, batch search Python-only | AGREGAR | 🟠 media | 🟡 | review #5 §API coverage; `vantadb-wasm/src/lib.rs:871-879` (sparse None) |
| H-10 | Sin entrada CDN ESM documentada para evaluación browser zero-install (Orama ofrece `cdn.jsdelivr.net/+esm` directo); requiere bundler+plugin obligatorio | AGREGAR | 🟡 baja | 🟢 | README orama; `vantadb-ts/README.md:80-97` |
| H-11 | Cero benchmarks reproducibles del path JS/WASM (Regla 11: sin número citable no hay claim de performance posible en marketing ni comparativas) | OPTIMIZAR | 🟠 media | 🟡 | ausencia verificada; `benches/canonical_p99.rs` cubre solo core nativo |
| H-12 | Sub-clientes wiki/conversation/skills vacíos o inexistentes (features core-only) — paridad creciente con core no planificada | ESTRATEGIA | 🟡 baja | 🔴 | `vantadb-ts/src/vantadb.ts:311-319` |
| H-13 | Posicionamiento indocumentado: diferenciación única (durable+WAL browser, híbrido RRF, grafo+IQL, errores tipados) no articulada en README/site vs Orama FTS-first | ESTRATEGIA | 🟡 media | 🟢 | análisis §3 de este informe |
| H-14 | `NativeVantaDB` depende de `vantadb-node` (npm 404, no publicado) → ruta de persistencia real en Node rota para consumidores; examples no lo aclaran | ESTRATEGIA | 🟠 media | 🟡 | `vantadb-ts/README.md:117`, `vantadb-ts/src/native.ts:113` |

**Total: 14 hallazgos** (APLICAR 4 · MEJORAR 4 · AGREGAR 2 · OPTIMIZAR 1 · ESTRATEGIA 3 · DESCARTAR 0)

---

*Informe generado por `/research vantadb-ts`. Fuente de decisiones: Fase D del comando.*
