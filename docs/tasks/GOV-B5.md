# Task: GOV-B5 — HTTP_API.md completo

**Plan:** docs/plans/2026-08-22-doc-governance-plan.md (Task 13)
**Archivos clave:** `docs/api/HTTP_API.md` (reescritura); `docs/api/openapi.yaml` (READ-ONLY, spec formal de GOV-B4: 35 paths); `src/cli_server.rs` (read-only)
**Estado inicial:** ⬜ PENDING

## Contrato

1. HTTP_API.md cubre TODOS los endpoints del yaml agrupados por dominio (System/Records/Search/Graph/Maintenance/Threads/Experimental), cada uno con método, path, request y response example real derivado del yaml.
2. 0 ejemplos con sintaxis LISP (ejemplo muerto :108 eliminado).
3. Regla en cabecera: "openapi.yaml es la spec formal; este doc es la guía narrativa".
4. curl REAL probado contra server local para ≥5 endpoints representativos (health, query, records put/get, search, list) — DB SIEMPRE en $env:TEMP; transcripciones reales en este task record.
5. Endpoints x-experimental agrupados al final con banner.

## Steps

- ✅ S1: Discovery + Regla 0 (leer yaml completo ✅, HTTP_API.md ✅)
- ✅ S2: Levantar `vanta-cli server --db $env:TEMP\gov-b5` y probar curl real (≥5 endpoints)
- ✅ S3: Reescribir docs/api/HTTP_API.md (35 paths agrupados por dominio + experimental al final)
- ✅ S4: Verify markdownlint exit 0 + cierre (plan file recitation)

## Impacto mapeado (Regla 0)

- **Leídos completos:** `docs/api/openapi.yaml` (1959L), `docs/api/HTTP_API.md` (191L).
- **Referencias entrantes a HTTP_API.md:** docs maestros/enlaces pendientes grep en S3 (solo se reescribe contenido, el path no cambia → sin rotura estructural).
- **Referencias salientes:** ninguna (doc standalone).
- **openapi.yaml:** READ-ONLY — regenerado en GOV-B4, es fuente de verdad de schemas/ejemplos.
- **Veredicto:** reescritura segura de un único .md; sin impacto en código ni specs.

## Transcripción curl (S2) — server: `vanta-cli server --http --port 18099 --db %TEMP%\gov-b5-db`

Rebuild requerido: el binario default no trae feature `server` → `cargo build --bin vanta-cli --features server`.

```text
=== GET /health ===
{"success":true,"data":"OK"}

=== POST /api/v2/records ===
req: {"namespace":"agent/main","key":"note-1","payload":"VantaDB is a vector-native knowledge graph","metadata":{"topic":{"String":"intro"}},"vector":[0.1,0.2,0.3,0.4],"ttl_ms":null}
res: {"namespace":"agent/main","key":"note-1","payload":"...","metadata":{"topic":{"String":"intro"}},"created_at_ms":1787432535128,"updated_at_ms":1787432535128,"version":1,"node_id":"258969631918792983342456585065684593810","vector":[0.1,0.2,0.3,0.4],"sparse_vector":null,"expires_at_ms":null,"superseded_by":null,"superseded_at_ms":null}

=== GET /api/v2/records/agent%2Fmain/note-1 ===
res: (mismo cuerpo que PUT)

=== GET /api/v2/list?namespace=agent/main ===
res: {"records":[<record note-1>],"next_cursor":null}

=== POST /api/v2/search (fresh DB) ===
req: {"namespace":"agent/main","query_vector":[],"filters":{},"text_query":"vector-native","top_k":10,"distance_metric":"Cosine","explain":false}
res ANTES de rebuild: {"error":"text_index not found: bm25","success":false}
POST /api/v2/maintenance/rebuild-index → {"scanned_nodes":1,"indexed_vectors":1,"skipped_tombstones":0,"duration_ms":0,...,"success":true}
res DESPUÉS: {"records":[{"record":{<record note-1>},"score":0.57536423,"explanation":null}],"next_cursor":null}

=== POST /api/v2/query ===
INSERT NODE#7 TYPE note {title: "hello"} VECTOR [0.5, 0.5]
→ {"success":true,"data":"Mutated 1 nodes: Node 7 inserted.","node_id":7}
FROM note
→ {"success":true,"data":"Read 1 nodes.","nodes":[{"id":7,"semantic_cluster":0,"relational":{"title":{"String":"hello"},"type":{"String":"note"}},"hits":1,"confidence_score":0.5}]}
(lowercase "from agent/main" y "insert <id> as <type> fields k=v" FALLAN con parse error — keywords UPPERCASE obligatorias; gramática del yaml description difiere de la real → deuda/ticket)

=== POST /api/v2/graph/bfs ===
req: {"roots":["7"]} → 400 invalid number | {"roots":[7]} → 400 missing field `max_depth`
req OK: {"roots":[7],"max_depth":2} → [7]
(graph/degree {"roots":[7],"max_depth":1} → {"7":[0,0]})
(yaml GraphTraversalBody dice roots:string + solo required — drift real → deuda)

=== POST /api/v2/threads === {"title":"demo thread"} → {"thread_id":"310279622029206533993990662647183162021"}
=== GET /api/v2/threads?limit=10 → [{"thread_id":"...","title":"demo thread","messages":[],"created_at":1787432771,"updated_at":1787432771,"metadata":{}}]
=== POST /api/v2/threads/{id} {"role":"user","content":"hello thread"} → {"sent":true}
=== GET /api/v2/threads/{id} → {...,"messages":[{"role":"user","content":"hello thread","timestamp":1787432790463,"metadata":{}},...]}

=== POST /api/v2/snapshots/demo-snap → {"name":"demo-snap","path":"C:\\Users\\Eros\\AppData\\Local\\Temp\\gov-b5-db\\data\\snapshots\\demo-snap"}
=== GET /api/v2/snapshots → ["demo-snap"]

=== GET /api/v2/metrics → {"metrics":{"startup_ms":772,...,"hnsw_nodes_count":3,...},"namespaces":{"agent/main":{"count":1,"expiring_soon":0,"expired":0}}}
=== POST /api/v2/maintenance/purge → {"purged":0}
=== GET /api/v2/autocomplete?prefix=FR → ["FROM"]
=== DELETE /api/v2/records/agent%2Fmain/note-1 → {"deleted":true} ; GET luego → {"error":"record not found: note-1","success":false}
=== POST /api/v2/records/batch (2 records ns=demo) → array de 2 records upserted con node_id asignado

Metadata: valores tipados como tagged enum ({String|Int|Float|Bool|DateTime|List*|Null}) — un valor plano "intro" da 400 unknown variant.
```

## Context Save Point

- Yaml: 35 paths. Experimental (x-experimental): `/dashboard`, `/dashboard/{path}`, `/conversation/add`, `/skill/listing`.
- Auth: Bearer si api_key configurada; dev mode sin key pasa directo.
