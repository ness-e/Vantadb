# TDAM — 04: Storage + Recall — Investigación profunda

> Fecha: 2026-08-18 · Agente: vanta-research · Fuente: `TencentCloud/TencentDB-Agent-Memory`, branch `feat/server_team`, v2.0.0-beta.1 (checkout local COMPLETO del commit `97f9465`, verificado contra `.git/logs/HEAD`). Ref = `MemoryCore/src/...` salvo indicación.

## 1. Resumen ejecutivo

TDAM separa el almacenamiento en dos abstracciones paralelas: **`IMemoryStore`** (L0/L1 → SQLite local o TCVDB remoto) y **`IStorageBackend`** (L2/L3 Markdown → filesystem local o COS). El recall (auto-recall hook) inyecta memorias L1 + persona L3 + navegación de escenas L2 en el contexto del agente con 3 estrategias: `keyword` (FTS5 BM25), `embedding` (coseno), `hybrid` (fusión RRF client-side sobre SQLite, o single-call nativa sobre TCVDB). **Hallazgo central para VantaDB: la fusión híbrida NO es nativa en el backend SQLite** — `getCapabilities().nativeHybridSearch: false`; el RRF vive en el caller (`auto-recall.ts`), con `candidateK = maxResults*3` y `RRF_K = 60`. El backend TCVDB sí hace `hybridSearch` nativo (dense + sparse + RRFRerank server-side).

## 2. Arquitectura y flujo

```
                    ┌─────────────── L0/L1 (estructurado) ───────────────┐
 config.storeBackend ─┤  sqlite (default): VectorStore (node:sqlite +     │
   │                   │    sqlite-vec vec0 + FTS5)  → vectors.db          │
   ▼                   │  tcvdb: TcvdbMemoryStore (dense server-side +    │
 createStoreBundle     │    sparse BM25 client-side, hybridSearch nativo) │
  │                    └──────────────────────────────────────────────────┘
  ▼
 StorePool (per-instanceId, LRU 100, shared BM25 encoder, grace-close 30s)
  │
  ├─► IMemoryStore: put/get/search/delete L1_records + L0_conversations (vec + meta + FTS)
  │
  └─► IStorageBackend: L2/L3 archivos Markdown (persona.md, scene-index, COS/local)

auto-recall (por turno):
  userText ─► sanitizeText ─► strategy?
    keyword    → buildFtsQuery (jieba) ─► searchL1Fts ─► bm25RankToScore
    embedding  → embeddingService.embed ─► searchL1Vector ─► score=1.0-distance
    hybrid     → [FTS5 + vec] en paralelo ─► RRF(k=60) client-side (SQLite)
              ─► o searchL1Hybrid nativo (TCVDB)
  ─► recall budget (maxCharsPerMemory/maxTotalRecallChars)
  ─► prependContext (memorias, dinámico) + appendSystemContext (persona+escenas+guía, cacheable)
```

Escritura L1: **dual-write** — JSONL `records/YYYY-MM-DD.jsonl` (fuente de verdad, shards diarios append-only) + SQLite (retrieval). Delete: vec en realtime, JSONL limpiado por memory-cleaner.

## 3. Lógica y algoritmos

- **RRF (Reciprocal Rank Fusion)** — `core/hooks/auto-recall.ts` `searchHybrid()`: `candidateK = maxResults * 3` (sobre-recupera para fusionar); score por item = `1 / (RRF_K + rank + 1)` con `RRF_K = 60`; si aparece en ambas listas se **suman** los scores; `sort desc` + `slice(0, maxResults)`. Existe también util reutilizable `core/store/search-utils.ts` (`rrfMerge`, RRF_K=60) usada por memory-search/conversation-search. Nota: el RRF del hook ignora el threshold (usa solo rank); el threshold (0.3) solo filtra en estrategias puras.
- **BM25 keyword** — `core/store/sqlite.ts:223-255` `buildFtsQuery()`: jieba `cutForSearch` (segmentación china, sub-palabras) + `ZH_STOP_WORDS` (~40 function words) → tokens OR-join en frases quoted (`"token" OR "token"`); fallback regex Unicode. `tokenizeForFts()` (278-291) indexa con los mismos tokens (match garantizado query↔index). FTS5 v2: columna `content` segmentada + `content_original UNINDEXED` (1087-1125). Score: `bm25(l1_fts) AS rank` (rank negativo en SQLite) → `bm25RankToScore()` = `relevance/(1+relevance)` (315-320). Fallback BM25 alternativo: `bm25-client.ts` (sidecar Python `bm25_server.py`, TF/IDF, degradación silenciosa) o `bm25-local.ts` (`@tencentdb-agent-memory/tcvdb-text`, jieba-wasm, idioma zh/en) — este último usado para sparse vectors en backend TCVDB.
- **Vector** — vec0 virtual tables con `distance_metric=cosine`; `searchL1Vector()` (1470-1580): over-retrieval `topK + ZERO_VEC_BUFFER(10)` (o `Math.max(topK*5, topK+10)` con isolation filter, sqlite.ts:1484) para saltar zero-vector legacy (distancia null/NaN); score = `1.0 - distance`. Filtrado isolation post-hoc `rowMatchesIsolation`.
- **Hybrid nativo** — `searchMemories()` en auto-recall: si `getCapabilities().nativeHybridSearch` → `searchL1Hybrid({query, topK})` single-call (TCVDB: dense server + sparse client + RRFRerank). SQLite: `nativeHybridSearch: false` (sqlite.ts:3462-3468) → fusion client-side. Nota: en TCVDB, si el sparse/BM25 no está disponible, `hybridSearch` degrada a BM25-only (tcvdb.ts:1019-1030).
- **Detección de reindex** — `embedding_meta` (552-557): cambio provider/model/dimensions → `dropVectorTables()` + `needsReindex` (565-618).
- **Tuning SQLite** — `PRAGMA busy_timeout=5000, journal_mode=WAL, cache_size=-65536, mmap_size=134217728, wal_autocheckpoint=1000` (456-468).
- **Seed** — `seed/input.ts`: validación 6 capas, Format A/B, `fillTimestamps()` contador global +100ms/msg (monotónico entre sesiones). `seed/seed-runtime.ts`: L0→L1→L2→L3; `waitForL1Idle` polling (estable 3 rondas); `captureStartTimestamp=0` (no filtra histórico); FIXME documentado: L2/L3 no se esperan (pipeline se destruye tras L1 idle).

## 4. Funcionalidades / APIs

- `IMemoryStore` (`core/store/types.ts`): backend-agnostic, capability-based (`getCapabilities()`), fault-tolerant (nunca lanza; devuelve []/false), sync-first. Métodos: `init`, `upsertL1/upsertL0`, `searchL1Vector/Fts/Hybrid`, `searchL0Vector/Fts`, `queryL1Records`, `deleteL1/L0`, `getCapabilities`, `isFtsAvailable`, `close`. Contrato extendido: `reindexAll`/`rebuildFtsIndex` (types.ts:635-638, sqlite.ts:1082), `deleteL1Expired`/`deleteL0Expired` (sqlite.ts:1696), `updateL0Embedding`/`supportsDeferredEmbedding` (types.ts:561,599); utilidades `core/store/embedding.ts` (653 líneas) y cliente TCVDB `core/store/tcvdb-client.ts`.
- `IsolationContext` (`core/store/isolation.ts`): `{teamId?, userId, agentId, sessionId, taskId?, sessionKey?}` obligatorio en writes; `DEFAULT_ISOLATION_ID="default"`; filtros de aislamiento en 6 ejes (team/user/agent/session/task/sessionKey, isolation.ts:159-171) aplicados post-query.
- `StorePool` (`core/store/store-pool.ts`): pool por instanceId; `mode: "sqlite"|"tcvdb"`; LRU `maxStores=100`; `configFingerprint` (url|database|apiKey) recrea store si cambia; **grace-close 30s** (CR-5: evict no cierra el store inmediatamente, protege in-flight); BM25 encoder compartido (evita OOM jieba); rutas: `default → dataDir/vectors.db`, otros → `dataDir/instances/{id}/vectors.db`; skill stores (TCVDB `{db}_skills`) con cache LRU propia.
- `IStorageBackend` (`core/storage/types.ts`, adapter.ts, factory.ts, local-backend.ts): `ScopedStorageBackend` con prefijo; **rechaza path traversal** (`..`, absolutos, NUL — CR-6, 2026-05-19) porque instanceId/sceneName/sessionKey entran en keys; factory: `"cos"` (dynamic import, requiere credentialProvider) vs `"local"` (default `./data/storage`).
- `l1-writer.ts` (v3): `MemoryType = persona|episodic|instruction|work_fact|work_task|work_method|work_artifact`; `priority: number 0-100` (−1 = instrucciones globales estrictas); `EpisodicMetadata{activity_start_time?, activity_end_time?}`; eliminó `keywords`.
- `l1-reader.ts`: `queryMemoryRecords` (SQLite preferido, composite index `(session_id, updated_time)`); fallback JSONL `readMemoryRecords`/`readAllMemoryRecords` (regex `\d{4}-\d{2}-\d{2}\.jsonl`).
- `abstractions/types.ts`: `IConfigSource` (fetchVdb/fetchCos), `IQuotaReporter` + `QuotaSnapshot` — edition-neutral.
- `utils/sanitize.ts`: `sanitizeText` (quita `<relevant-memories>`, `<user-persona>`, `<scene-navigation>`, metadatos gateway, timestamps, base64, directivas `[[reply_to]]`), `stripCodeBlocks`, `shouldCaptureL0` (permisivo) vs `shouldExtractL1` (estricto, con detección prompt-injection), `escapeXmlTags`, `sanitizeJsonForParse`.
- `offload-client/` (stateless, server-delegated): `registerOffloadClient` — 3 hooks de lifecycle (`offload-client/index.ts:46,51,60`), principal `after_tool_call` (ingest fire-and-forget) + Context Engine `memory-tencentdb` (compaction API); `checkHealth` GET `/v2/offload/health`, `ingest`/`ingestL15` POST `/v2/offload/ingest`, `compaction` POST `/v2/offload/compact` (devuelve null en fallo → caller conserva mensajes).

## 5. Código clave (fragmentos + ref)

```ts
// core/hooks/auto-recall.ts — searchHybrid()
const candidateK = maxResults * 3; // retrieve more for merging
const RRF_K = 60;
const rrfScore = 1 / (RRF_K + rank + 1); // por item, sumado si aparece en ambas listas
const sorted = [...mergedMap.entries()].sort((a,b) => b[1].rrfScore - a[1].rrfScore).slice(0, maxResults);
```

```ts
// core/store/sqlite.ts:1088-1107 — FTS5 v2 (BM25 indexado)
CREATE VIRTUAL TABLE IF NOT EXISTS l1_fts USING fts5(
  content, content_original UNINDEXED, record_id UNINDEXED, ...
  user_id UNINDEXED, agent_id UNINDEXED, ...)
// búsqueda: SELECT ..., bm25(l1_fts) AS rank FROM l1_fts WHERE l1_fts MATCH ?
```

```ts
// core/store/sqlite.ts:684-689 — vec0
CREATE VIRTUAL TABLE IF NOT EXISTS l1_vec USING vec0(
  record_id TEXT PRIMARY KEY, embedding float[N] distance_metric=cosine, updated_time TEXT DEFAULT '')
// upsert = INSERT meta ON CONFLICT DO UPDATE + DELETE vec + INSERT vec (sin ON CONFLICT en vec0)
```

```ts
// core/store/sqlite.ts:3462-3468
getCapabilities(): StoreCapabilities {
  return { vectorSearch: this.vecTablesReady, ftsSearch: this.ftsAvailable,
           nativeHybridSearch: false, sparseVectors: false };
}
```

## 6. Integración en VantaDB

VantaDB **ya es el motor** (HNSW + BM25 + RRF, WAL, columnar) — no copiar el stack SQLite/sqlite-vec ni el dual-write JSONL. Recomendación por capa:

- **Core puro (WASM-compatible, sin LLM):** implementar el **search profile por namespace** (`mode: keyword|vector|hybrid`, `rrf_k`, `candidate_k`) como feature de configuración del store — es LLM-free y directamente derivable de `auto-recall.ts` + `search-utils.ts`. Es la única pieza de este research que VantaDB no tiene como parámetro expuesto.
- **vanta-memory (capa LLM):** el *patrón* de recall — separar `prependContext` (memorias, dinámico) de `appendSystemContext` (persona/escenas/guía, cacheable) para no romper prompt caching — es portable y valioso. También `sanitizeText` (anti feedback-loop) y el recall budget con truncación por code-point (nunca cortar surrogate pairs).
- **MCP/server/integrations:** ignorar el split de 4 servicios (sidecar BM25 Python, bm25-client, TCVDB HTTP, offload server) — VantaDB no lo necesita.
- **Isolation** (team/user/agent/session con `DEFAULT_ISOLATION_ID`) es análogo al aislamiento por namespace de VantaDB; tomar el patrón de filtrado `rowMatchesIsolation` post-query solo si falta.

## 7. Riesgos / limitaciones / qué NO copiar

- **No copiar:** sqlite-vec + FTS5 dual-table + dual-write JSONL (VantaDB ya resuelve esto mejor con un solo engine); sidecar BM25 Python (dependencia de proceso externo); `ZERO_VEC_BUFFER` hack (síntoma de zero-vectors legacy); separación `IMemoryStore`/`IStorageBackend` (sobre-ingeniería para VantaDB: 2 backends × 2 capas = 4 combos, YAGNI).
- **Riesgos:** (1) RRF client-side ignora scores absolutos — umbral no aplica en hybrid (baja precisión con candidateK pequeño); (2) `searchL1Vector` sobre-recupera `topK*5` con filter → coste O(n) en vec0; (3) FTS5 con jieba-wasm = dependencia wasm pesada para chino (VantaDB no la necesita salvo target zh); (4) grace-close 30s es mitigación de diseño de pool, no feature; (5) `waitForL1Idle` con FIXME L2/L3 = seed incompleto conocido; (6) SQLite requiere Node 22+ (`node:sqlite`) — no portable a WASM.
- **Confianza:** alta (código leído completo de sqlite.ts, auto-recall.ts, store-pool.ts, seed, offload-client; mediana para tcvdb.ts — leído pero truncado en respuesta).

## RESULTADO
- Estado: ✅ COMPLETO
- Archivo: docs/research/tdam/04-storage-recall.md
- Hallazgo principal: el hybrid NO es nativo en SQLite — el RRF es client-side (`candidateK = maxResults*3`, `RRF_K=60`, score `1/(60+rank+1)` sumado); VantaDB solo necesita exponer search profile por namespace (mode/rrf_k/candidate_k) + patrón prepend/append para prompt caching.
- Ref clave: `core/hooks/auto-recall.ts` `searchHybrid()` + `core/store/sqlite.ts:3462-3468` (`nativeHybridSearch: false`)