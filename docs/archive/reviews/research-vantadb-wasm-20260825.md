# INV-vantadb-wasm-01 — Investigación profunda: `vantadb-wasm` (bindings WASM standalone)

**Fecha:** 2026-08-25 · **Tipo:** Bindings WASM standalone · **Ecosistema:** npm (pkg)
**Usuarios objetivo:** Frontend browser-only (sin servidor)
**Competidores:** Orama (browser), sql.js-httpvfs, DuckDB-WASM, vectra · **Competidor principal:** Orama (browser)
**Docs canónicas:** `vantadb-ts/README.md` (compartida) + `vantadb-wasm/demo/README.md`
**Base previa:** `docs/reviews/modulos/vantadb-wasm.md` (deep-review 2026-08-22, score 7/10) — este informe lo actualiza contra el estado real del código hoy y agrega la dimensión competitiva/ecosistema.

---

## 1. Usuarios objetivo y su flujo diario

El usuario de `vantadb-wasm` es un developer frontend que quiere memoria persistente
(vector + texto + grafo) para una app browser-only — típicamente un AI agent que corre
100% client-side (chat con embeddings on-device vía Transformers.js, RAG local, notas).
Flujo esperado: `npm i vantadb-wasm` → import ESM → `new VantaDB()` o
`connect_persistent(path)` (OPFS) / `connect_idb(path)` (fallback) → CRUD +
search híbrido → `save()` explícito o auto-save.

Fricciones conocidas del nicho (evidencia externa):
- **Cuotas de storage browser**: Safari ~1 GB por origen, Chrome ~60% del disco
  (medido en WASM-01, verificado contra MDN/web.dev; registrado en
  `docs/api/WASM_STANDALONE.md`). Los usuarios esperan manejo explícito de
  `QuotaExceededError` y eviction de IndexedDB.
- **Soporte OPFS**: Chrome/Edge 86+, Firefox 111+, Safari 15.2+ — el fallback a IDB
  es necesario y el usuario necesita saber cuál backend quedó activo.
- El demo interno (`demo/`) ya muestra el flujo completo con Transformers.js (~23 MB
  de modelo en el primer load) — es el mejor material de onboarding pero no está
  enlazado desde npm.

## 2. Estándares del ecosistema npm/WASM

- **Empaquetado wasm-pack**: el pkg generado cumple lo básico (`type: module`,
  `files`, `main`, `types`, `sideEffects`) pero el `.d.ts` generado es casi todo
  `any` (limitación conocida de wasm-bindgen). El estándar del ecosistema para
  paquetes WASM serios es un `.d.ts` hand-written encima del glue
  (patrón usado por DuckDB-WASM y sqlite-wasm-http).
- **Distribución**: npm registry directo (sin prebuilds multiplataforma como napi-rs;
  ventaja estructural vs `vantadb-node`). CI de release existe:
  `release-npm-61.yml` publica wasm+ts en tags `v*` con gate wasm32 por PR (FND-16).
- **CI de tests**: `ci-rust-10.yml:407` corre `wasm-pack test --chrome --headless` ✅
  (el hallazgo "no corre en CI" del storage review 2026-07-22 está RESUELTO; sigue sin
  Firefox/Safari, aceptable).
- **Tamaño**: bundle actual `pkg/vantadb_wasm_bg.wasm` = 1.333 KB con `-Oz`
  (`Cargo.toml:14-18`). Competidores JS puros (Orama ~50 KB gzip) ganan por 25×;
  DuckDB-WASM pesa más (~35 MB full). El tamaño solo se defiende narrándolo.

## 3. Competidores

| | **Orama** (@orama/orama) | **DuckDB-WASM** | **sql.js-httpvfs** | **vectra** | **vantadb-wasm** |
|---|---|---|---|---|---|
| Arquitectura | Motor JS puro (WASM opcional) | OLAP SQL sobre WASM (Rust/C++) | SQLite WASM read-only via HTTP Range | Archivos JSON locales Node | Engine Rust completo (vector+BM25+grafo+IQL) compilado a WASM |
| Persistencia browser | ❌ in-memory (serialization manual del JSON schema) | ✅ OPFS nativo (`opfs://`, `checkpoint_threshold='0KB'`) | ❌ (read-only) | ❌ (Node files) | ✅ OPFS + IndexedDB fallback + Web Worker opcional |
| Vector search | ✅ + hybrid (full-text+vector en una query) | ❌ (SQL extension aparte) | ❌ | ✅ cosine básico | ✅ HNSW + BM25 + RRF + sparse |
| Grafo/IQL | ❌ | ❌ | ❌ | ❌ | ✅ BFS/DFS/topo/DAG + DSL IQL |
| Tipos TS | ✅ excelentes (hand-written) | ✅ excelentes | ⚠️ básicos | ✅ | ❌ `.d.ts` casi todo `any` |
| Downloads/mes (npm, 2026-07-26→08-24) | 5.442.943 | ~500K (est.) | ~30K (est.) | ~40K (est.) | **187** |
| Licencia | Apache-2.0 | MIT | MIT | MIT | Apache-2.0 |

Fuentes: [npmjs @orama/orama](https://www.npmjs.com/package/@orama/orama),
[docs.orama.com hybrid](https://docs.orama.com/docs/orama-js/search/hybrid-search),
[DuckDBOPFSConfig](https://shell.duckdb.org/docs/interfaces/index.DuckDBOPFSConfig.html),
[Zenn: DuckDB-WASM OPFS auto-persist](https://zenn.dev/hideyuki_hori/articles/ae523f62f32fb8),
[sql.js-httpvfs](https://github.com/phiresky/sql.js-httpvfs),
[vectra](https://github.com/Stevenic/vectra) (631 stars), stats npm via api.npmjs.org.
Performance claims de competidores: Orama no publica benchmarks reproducibles en su
README principal → "claim sin evidencia" (Regla 11); no se usan números de terceros.

**Lectura honesta:** Orama gana DX, tipos, tamaño y adopción por órdenes de magnitud,
pero **no persiste en browser ni tiene grafo**. DuckDB-WASM valida que OPFS-first es
el patrón correcto (y que el auto-persist es la expectativa del usuario). El nicho
desatendido exacto de VantaDB-WASM: **memoria persistente de AI agent 100% client-side
con búsqueda híbrida + grafo** — el demo de Transformers.js es ese producto.

## 4. Estado actual de `vantadb-wasm`

- **API pública:** 43 pub fns (`BINDINGS_NAMESPACES.md:35`): memoria CRUD+search+explain,
  grafo (insert_node/add_edge/bfs/dfs/topo/dag/traversal/degree), IQL `query()`,
  export/import (+bulk .vdbdump), save/load OPFS+IDB, connect_persistent/connect_idb/
  connect_worker, audit/repair/reindex, operational_metrics/capabilities/close.
- **Persistencia de grafo: RESUELTA** desde el deep-review — `restore_graph_payload`
  serializa nodos desde `graph_state.json` (`lib.rs:728-748`, fix CORE-02 cross-session).
- **PERF-08 persistencia diferencial** implementada (`lib.rs:265-299`, dirty/deleted/cache_invalid).
- **P2-8 pendiente:** `collect_all_deduped()` O(n) en memoria (`lib.rs:564-596`).
- **Bug vivo:** `OpfsFile::append` sobreescribe desde offset 0 (`opfs.rs:85-97`;
  la versión JS sí usa posición, `opfs_bridge.js:53-57` — divergencia confirmada hoy).
- **Fallback silencioso vivo:** `OpfsStorage::open(path).await.ok()` en
  `connect_persistent` (`lib.rs:425`) — si OPFS falla abre igual y `save()` es no-op.
- **Testing:** ~40 wasm_bindgen_tests + suite OPFS/IDB (1146 líneas) + e2e Playwright;
  `wasm-pack test --chrome` en CI ✅.
- **Empaquetado:** `wasm-pack build` → `pkg/` (git-ignored glue, publicado por CI).
  Versión 0.5.0 workspace-inherited.
- **Historial:** P7 cerrado; WASM-01..04 completados (Fase 4 Vanta Studio);
  PERF-08 resuelto; FIND-11 parcial (bundle sin doc lazy-load).

## 5. Framework de evaluación (score por dimensión)

| Dimensión | Score | Evidencia clave |
|---|---|---|
| DX de onboarding | 6.0 | Demo excelente pero aislado; `.d.ts` any; README npm genérico |
| Completitud funcional | 6.5 | 43 ops sólidas; faltan filter_ops/exclude_superseded/sparse_vector/remove_edge/count/supersede |
| Performance/overhead | 7.0 | PERF-08 diferencial ✅; P2-8 O(n) pendiente; vectors zero-copy sanitizados |
| Robustez | 6.0 | append roto, quota sin manejar, CRC débil, fallback silencioso |
| Seguridad | 8.0 | Guardas FFI (MAX_*), OpGate close-barrier, CRC footers |
| Docs & ejemplos | 6.0 | `vantadb.ts:105` dice "always in-memory" (falso post-persistencia); doc compartida con ts genera confusión |
| Observabilidad | 7.0 | operational_metrics + audit_text_index(+deep) expuestos |
| Testabilidad | 8.5 | Suite WASM excepcional + e2e real + CI Chrome |
| Paridad con otros módulos | 5.5 | Límites divergentes (MAX_K 1k vs node 10k; vec len 10M vs dim 10k); score-vs-distance divergente entre transports |
| Diferenciación | 8.0 | Único engine browser con persistencia real + vector+texto+grafo; adopción aún testimonial |

**Score global: 6.8 / 10** (baja de 7.0 del review previo: la competencia maduró —
DuckDB-WASM OPFS auto-persist es hoy el estándar de UX de persistencia browser — y
nuestros gaps de robustez/DX pesan más en esa comparación).

## Gap analysis priorizado

**Falta (P0):** fallback silencioso→error/fallback-IDB explícito; manejo de cuotas;
fix append.
**Mejorable (P1):** shape de errores {code,message}; `.d.ts` hand-written;
auto-save; paridad de límites/score entre transports; docs TS contradictorias.
**Optimizable (P2):** P2-8 collect_all_deduped; estrategia de tamaño/lazy-load documentada.

## Quick wins (<1 día) vs apuestas estratégicas (>1 semana)

**Quick wins:** fix append (posición como bridge JS) · flush() delegar a save o
documentar · next_cursor u64→string · MessagePort.close() por request · corregir
comentario `vantadb.ts:105` · exponer helper spawnOpfsWorker desde pkg glue.
**Estratégicas:** taxonomía de errores tipados compartida entre SDKs · batch de
paridad API (filter_ops/exclude_superseded/sparse_vector + métodos faltantes) ·
posicionamiento/adopción npm (nicho "browser AI agent memory") · auto-save
(visibilitychange/pagehide hook).

---

## APÉNDICE — Inventario de hallazgos (H-NN)

> Fuente de la Fase D. Categorías: APLICAR/MEJORAR/AGREGAR/OPTIMIZAR/ESTRATEGIA/DESCARTAR.

| ID | Hallazgo | Categoría | Severidad | Esfuerzo | Ref |
|---|---|---|---|---|---|
| H-01 | `OpfsFile::append` sobreescribe desde offset 0 (bridge JS diverge) | APLICAR | 🟠 Alta | 🟢 | `opfs.rs:85-97` vs `opfs_bridge.js:53-57` |
| H-02 | MOD-25..28 derivados del review 2026-08-22 nunca aterrizaron en Backlog (solo tabla en el reporte) — gap de trazabilidad | APLICAR (proceso) | 🟡 Media | 🟢 | `docs/reviews/modulos/vantadb-wasm.md:105-116` |
| H-03 | Fallback silencioso a in-memory si OPFS falla al abrir (`.ok()`, sin warning, capabilities no refleja) | MEJORAR | 🔴 Crítica | 🟡 | `lib.rs:425` |
| H-04 | Cuotas sin manejar: sin `estimate()`, sin `navigator.storage.persist()`, QuotaExceeded crudo | MEJORAR | 🟠 Alta | 🟡 | `opfs.rs` write_file / `idb.rs` |
| H-05 | `flush()` engañoso: docstring dice "to disk" pero backend in-memory; durabilidad real = `save()` manual | APLICAR | 🟡 Media | 🟢 | `lib.rs` flush |
| H-06 | Metadata descartada silenciosamente si serde falla en `memory_record_to_js` | MEJORAR | 🟡 Media | 🟢 | `lib.rs:1582` aprox |
| H-07 | CRC débil: footer inválido devuelve datos crudos ("legacy fallback") → error de parseo confuso | APLICAR | 🟡 Media | 🟢 | `opfs.rs:207` |
| H-08 | `next_cursor` viaja como f64 — rompe política string-u64 (>2^53) | APLICAR | 🟢 Baja | 🟢 | `lib.rs:943` |
| H-09 | Errores sin shape `{code,message}`: `to_js_err` aplana VantaError a string | AGREGAR | 🟠 Alta | 🟡 | `lib.rs:1518` |
| H-10 | `pkg/*.d.ts` casi todo `any` — paquete standalone inusable con tipos; escribir d.ts hand-written | AGREGAR | 🟠 Alta | 🟡 | `pkg/vantadb_wasm.d.ts` |
| H-11 | Paridad API: parámetros perdidos (filter_ops, exclude_superseded, sparse_vector, search_profile) y métodos ausentes (remove_edge, count, namespace_stats, similar_to_key, supersede, graphrag_search) | AGREGAR | 🟡 Media | 🔴 | `BINDINGS_NAMESPACES.md` §faltantes |
| H-12 | `connect_worker` exige inyectar `globalThis.spawnOpfsWorker` a mano — exponer helper desde glue | AGREGAR | 🟢 Baja | 🟢 | `worker.rs` / `lib.rs:334` |
| H-13 | Doc contradictoria: `vantadb.ts:105` afirma "WASM always in-memory" (falso desde connect_persistent/connect_idb) | AGREGAR (docs) | 🟡 Media | 🟢 | `vantadb-ts/src/vantadb.ts:105` |
| H-14 | Límites divergentes entre transports: MAX_F32_VEC_LEN=10M vs node MAX_VEC_DIM=10k; MAX_K=1k vs node top_k≤10k — unificar constantes en core | MEJORAR | 🟡 Media | 🟡 | `lib.rs:38-43` |
| H-15 | Semántica score/distance divergente: wasm emite `score` que TS documenta como distance | MEJORAR | 🟡 Media | 🟡 | binding ↔ `vantadb-ts` |
| H-16 | Worker proxy: retry matchea strings frágiles + MessagePorts nunca `.close()` (leak por request) | APLICAR | 🟢 Baja | 🟢 | `worker.rs` |
| H-17 | Sanitización NaN/Inf→0.0 silenciosa — agregar contador de sanitizaciones en metrics | MEJORAR | 🟢 Baja | 🟢 | vectors coercion |
| H-18 | Sin auto-save: datos desde último `save()` se pierden al cerrar pestaña (hook visibilitychange/pagehide) | MEJORAR | 🟠 Alta | 🟡 | `lib.rs` save/load |
| H-19 | `collect_all_deduped()` O(n) en memoria (u128 HashSet) — deuda P2-8 ya trackeada | OPTIMIZAR | 🟡 Media | 🟡 | `lib.rs:564-596` (P2-8) |
| H-20 | Bundle 1.3 MB sin estrategia documentada de lazy-load/code-split (ni comparativa honesta vs JS puro) | OPTIMIZAR | 🟡 Media | 🟡 | `pkg/vantadb_wasm_bg.wasm`; FIND-11 |
| H-21 | Adopción npm 187 descargas/mes vs Orama 5.4M: sin posicionamiento del nicho (README npm, keywords, demo enlazada, comparativa honesta) | ESTRATEGIA | 🟠 Alta | 🟡 | api.npmjs.org 2026-07-26→08-24 |
| H-22 | Diferenciación sub-explotada: único browser engine con persistencia real + vector+BM25+grafo; DuckDB-WASM validó el patrón OPFS-auto-persist — apostar por "AI agent memory browser-first" | ESTRATEGIA | 🟡 Media | 🔴 | §3 matriz |
| H-23 | Caso de uso read-only static hosting (HTTP Range, estilo sql.js-httpvfs): fuera del scope del módulo (write-heavy) | DESCARTAR | 🟢 Baja | — | sql.js-httpvfs |

---

*Investigación INV-vantadb-wasm-01 ejecutada por `/research vantadb-wasm` (2026-08-25).
Decisiones por hallazgo → Fase D del comando; materialización en Backlog/wontfix/plan.*
