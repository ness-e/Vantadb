# Consolas y paneles de administración de Vector DBs

Investigación de fuentes públicas (docs oficiales, repositorios y páginas de producto) recopilada entre 2024-2026. El objetivo es alimentar el diseño de una consola de administración humana para VantaDB: cómo las bases de datos vectoriales existentes le muestran sus datos a un humano (layout, visualización de vectores, editores de consulta, filtros, gestión de índices, monitoreo), qué funciona y qué no.

**Contexto de VantaDB que guía la lectura:** motor embebido de memoria persistente para agentes de IA con `VantaMemoryRecord` (namespace + key único + payload de texto + metadata arbitraria + timestamps + version + vector denso opcional + sparse vector + TTL), búsqueda híbrida BM25 + HNSW + fusión RRF, capa de grafo (aristas dirigidas, acumuladores, IQL), audit log JSONL, almacenamiento LSM. El usuario objetivo es un **desarrollador de IA / agente**, no un DBA.

---

## Resumen ejecutivo

La industria de las vector DBs está dividida en dos mundos claros respecto a las consolas humanas:

1. **DBs "puristas de embeddings"** (Pinecone, Qdrant, Chroma, LanceDB, Milvus, Weaviate): la consola gira alrededor de **colecciones/índices**, y el vector es tratado como un campo binario opaco. Ninguna ofrece una *visualización* real de los embeddings (proyección 2D/3D, scatter, etc.) en la consola de producción; los vectores se muestran como números, JSON o "n/a" (Pinecone directamente **no muestra registros** en su consola). La excepción didáctica: los **vector sets de Redis**, cuyos docs literalmente dibujan vectores 2D sobre un plano cartesiano para explicar `VSIM`.
2. **DBs que integran búsqueda y operacional** (MongoDB Atlas, Elastic/Kibana, RedisInsight, Supabase/pgvector): el vector es **una columna más** dentro de un explorador de datos clásico (tabla/JSON/documento). No hay "vista de embeddings", pero sí un explorador de registros completo y herramientas de consulta potentes.

**Hallazgos transversales:**
- Casi todas las consolas priorizan **búsqueda por similitud como acción de primera clase** (una caja "pega tu vector / texto" en la vista principal) y dan al vector el rol de *input*, no de *objeto a inspeccionar*.
- El **payload/metadata es la parte que el humano realmente lee** (JSON formateado con colores, filtrable). Es lo que separa una consola útil de una inútil.
- El **monitoreo** (métricas, health, latencia, índice) es la segunda función más común: Pinecone, Zilliz, Qdrant Cloud, Weaviate, RedisInsight (Profiler/SlowLog) y Mongo Atlas lo tienen.
- Los **editores de consulta** vienen en dos sabores: REST/JSON playground (Qdrant Console, Pinecone playground) y lenguaje de consulta estructurado (GraphiQL en Weaviate, YQL en Vespa, `$vectorSearch` pipeline en Mongo).
- **Gap de mercado:** nadie (salvo Attu y el Explorer de Weaviate, parcialmente) ofrece un *explorador visual de datos de búsqueda vectorial* para no-programadores. VantaDB tiene oportunidad de diferenciarse sin copiar a nadie.
- Varias DBs no tienen GUI oficial alguna (Chroma, LanceDB, Vespa, Marqo, Vald, txtai): el "admin console" se resuelve con CLI + SDKs + dashboards de Grafana. Esto confirma que una GUI es opcional pero que, donde existe, se convierte en parte del pitch comercial (Attu, RedisInsight).

---

## Comparativa

| Herramienta | Layout | Cómo muestra vectores | Editor de consultas | Filtros | Índices | Rating (1-10) |
|---|---|---|---|---|---|---|
| **Pinecone Console** | Sidebar: Proyectos → Database > Indexes; tabs por índice (Metrics, Data?, API Keys) | No muestra registros ni vectores en consola; solo métricas/estado | Playground de API REST (obsoleto en favor de SDKs) | Solo server-side via API (namespaces, metadata filters) | Vista por índice, estado (Initializing/Ready), dimensiones, métricas | **5** |
| **Weaviate Cloud Console** | Clusters → Collections; herramientas dedicadas (Explorer, Query, Collections) | Explorer: browse/inspección visual de objetos sin código; vector como campo | **Query tool** (GraphiQL) + CLI; gRPC/REST | Filtros por propiedad en Explorer (equals, contains, ranges) | Schema editor visual, status del cluster, TTL por collection | **8** |
| **Qdrant Web UI / Cloud** | SPA local (`localhost:6333/dashboard`); tabs Console / Collections | Vectores como JSON crudo dentro del punto; sin visualización | **Console**: playground REST (search, upsert, update, delete) con JSON | Payload filters JSON en el playground | Collections UI (crear, configurar, snapshots); config de HNSW/quantización vía API | **7** |
| **Milvus / Zilliz Cloud / Attu** | Attu (web+desktop): sidebar de collections, clusters, monitoreo, backups | Attu browse de colecciones; search con input de vector; sin proyección | Attu: form de búsqueda + query; Zilliz Cloud: search tabs (vector, filtrada, híbrida, full-text, query) | Filtros de scalar fields en búsqueda (Zilliz docs: "Filtered Search", "Grouping Search") | Crear/editar índice por campo (manage-indexes); refresh de colecciones externas; métricas y alerts | **8** |
| **Chroma** | Sin GUI oficial | — | Solo Python/JS SDKs + CLI | Metadata filtering via API | API-only (HNSW automático) | **3** |
| **LanceDB** | Sin GUI oficial | — | Solo SDKs (Python/TS/Rust) | SQL y filtros via SDK | API-only | **3** |
| **Redis / RedisInsight** | Desktop/Electron: Browser (tree de keys), Workbench, Profiler, SlowLog | **Vector sets**: CRUD de elementos, editar vector, correr VSIM; docs dibujan vectores 2D en plano cartesiano | **Workbench** (Monaco Editor): CLI con autocompletado, visualización de resultados, raw mode | Browser con filtros de key/type; VSIM con `FILTER` por atributos JSON | Visualizaciones de índices RediSearch (`FT.INFO`), autocompletado de comandos de search/query | **7** |
| **pgvector / Supabase** | Supabase Studio: Database > Tables, Editor, SQL editor | `vector` como columna; valores mostrados como array numérico en la tabla | **SQL editor** + rpc() para funciones de similitud (PostgREST no soporta operadores `<->`/`<=>` directamente) | SQL WHERE sobre columnas; para vector search hay que escribir función + llamarla por RPC | Enable extensión con toggle; índices HNSW/IVFFlat vía SQL | **5** |
| **MongoDB Atlas Vector Search** | Atlas UI: clusters, Database Deployments, índice de búsqueda; Compass (GUI desktop) | Vectores como campos `number[]` dentro del documento; sin visualización dedicada | Aggregation pipeline con stage `$vectorSearch` (ANN/ENN) en Atlas UI o Compass; auto-embedding single-click (Voyage AI) | Query API filtering (`$eq`, `$gte`) combinado con `$vectorSearch`; hybrid con fusión RRF | Crear índices Atlas Search con campos vectoriales (dim, similarity); quantización scalar/binary; native rerank `$rerank` | **8** |
| **Vespa** | Sin consola GUI; config-as-code (services.xml) | — | YQL + `vespa query` CLI; pyVespa; tensors en JSON | Filtros en YQL + ranking expressions | Reindex, metrics API, memory visualizer (perf) | **3** |
| **Elastic / Kibana** | Kibana (Discover, Dashboards, Dev Tools); Elastic Cloud | `dense_vector` como campo; kNN via API | **Dev Tools Console** (query DSL) + Kibana Discover | kNN con metadata filters inline; hybrid BM25F + vector | Índices y mappings UI; ESRE; AI Playground | **6** |
| **Typesense** | Typesense Cloud dashboard | Collections + documentos JSON; typesearch API | API Keys UI, collections, autocomplete/synonyms config | Filtering UI en console; filtros server-side | Cluster status, configuración de typo-tolerance/synonyms | **6** |

---

## Análisis detallado por herramienta

### Pinecone Console

- URL consola: `https://app.pinecone.io/organizations/-/projects` (projects → index)
- Documentación: `https://docs.pinecone.io/llms.txt` (índice completo), `https://docs.pinecone.io/guides/production/monitoring.md`
- **Qué hace:** monitorizar índices (estado, dimensión, métricas), namespaces, API keys, configuración de serverless (pods/storage), y exportar métricas a **Prometheus o Datadog**.
- **Layout:** jerárquico "Organization → Project → Index"; cada índice tiene tabs con **Metrics tab** (Database > Indexes > [index]).
- **Cómo muestra vectores:** **no los muestra**. No hay explorador de puntos/registros en la consola (gap notable: un usuario no puede ver "qué hay dentro" de un índice desde la web; toca usar SDKs o scripts).
- **Editor de consultas:** playground REST histórico; el énfasis actual está en SDKs y el agente `assistant` (chat de soporte con contexto del proyecto).
- **Lección:** la UI de operación (estado + métricas) puede existir **sin** UI de datos. Pinecone elige deliberadamente no construir exploradores de datos porque su target son equipos que interactúan por API. Para VantaDB (memoria de agentes), el explorador de datos es **el** producto visible.

### Weaviate Cloud Console

- Explorer tool: `https://docs.weaviate.io/cloud/tools/explorer-tool` y `https://weaviate.io/product/explorer.md`
- Query tool: `https://docs.weaviate.io/cloud/tools/query-tool`
- Collections tool: `https://docs.weaviate.io/cloud/tools/collections-tool`
- Índice de docs: `https://weaviate.io/llms.txt`
- **Explorer:** herramienta visual para **buscar e inspeccionar objetos sin escribir código** — eliges una colección, ves sus propiedades (incluido el vector como campo), filtras, y navegas resultados. Pensado para desarrollo/troubleshooting, no para producción.
- **Query tool:** editor **GraphiQL** (Weaviate fue originalmente GraphQL-first). Hoy Weaviate también ofrece gRPC y una "Query Agent" para RAG (búsqueda en lenguaje natural).
- **Collections tool:** editor visual de schema (colecciones, propiedades, tipos), configuración de TTL, e **import wizard** (CSV/PDF).
- **Cloud:** consola de clusters (estado, métricas), multi-tenancy, backups. `console.weaviate.cloud`.
- **Ecosistema comunidad:** `weaviate-db-extension` (npm) — GUI open source sobre Weaviate: explorar schemas, correr GraphQL, herramientas RAG.
- **Lección:** separar las *herramientas* de la *consola de operación* (Explorer/Query/Collections como pestañas) es un patrón claro y escalable. El Explorer es exactamente lo que VantaDB quiere: **ver qué hay dentro sin escribir una query**.

### Qdrant Web UI / Cloud

- Web UI: `https://qdrant.tech/documentation/web-ui/`
- Cloud: `https://qdrant.tech/documentation/cloud-quickstart/`, consola en `https://cloud.qdrant.io`
- **Web UI** (SPA React): servida por el propio binary en `http://localhost:6333/dashboard`. Pestañas principales: **Console** (playground REST: busca, upsert, update, delete, con autenticación si está configurada), **Collections** (listar, crear, configurar, y **subir/restaurar snapshots**), y un **tutorial interactivo** embebido (`#/tutorial`). Para cloud: Cluster URL + `:6333/dashboard`.
- **Cómo muestra datos:** los puntos (point id + vector + payload) como **JSON crudo**. El payload (metadata) se muestra legible; el vector como array de floats. Sin visualización de embeddings.
- **Docs de modelo de datos** (relevantes para VantaDB): Points, Vectors (dense y sparse), Payload, Collections, Storage, **Indexing** (HNSW, payload index), **Quantization**, Multitenancy, Bulk Upload — en `https://qdrant.tech/documentation/manage-data/`.
- **Lección:** el playground REST que **ejecuta contra el server real** (con auth real) es la forma más barata de dar un "query editor" con valor inmediato. El snapshot upload desde la UI (drag & drop) es un feature de ops muy apreciado.

### Milvus / Zilliz Cloud / Attu

- **Attu** (la GUI open source de Milvus): `https://github.com/zilliztech/attu`; página oficial `https://zilliz.com/attu`. Distribución: web app (Docker/K8s) y desktop (macOS/Linux/Windows).
  - Features del README (v3): conectar a múltiples clusters, **browse de colecciones**, **correr vector searches**, gestionar **backups**, **monitorear salud**, y chat con un **agente AI** de soporte. Es la GUI de vector DB más completa del mercado open source.
- **Zilliz Cloud console** (`https://cloud.zilliz.com`, docs en `https://docs.zilliz.com/`): documentos que describen la UI → Collection, Schema & Data Fields, Insert & Delete, **Indexes** (`/docs/manage-indexes`), Search (`/docs/search-query-get`; tabs de *Single Vector Search*, *Filtered Search*, *Grouping Search*, *Hybrid Search*, *Full Text Search*, *Query/Get*), Import & Export, Data Lake Search.
  - Operaciones: create/scale clusters (`/docs/create-cluster`, `/docs/manage-cluster`), **métricas y alertas** con charts (`/docs/view-cluster-metric-charts`), backups programados (`/docs/create-backup`, `/docs/schedule-automatic-backups`), snapshots, organizaciones/proyectos/usuarios, **recycle bin**, **auditing logs** y **access logs** (`/docs/auditing`), billing/cost.
  - **BM25 function** y embedding functions integradas (generan el vector en ingesta — el "Integrated Embedding" model en consola).
- **Milvus docs** (referencia Attu): `https://milvus.io/docs/quickstart_with_attu.md` — "visualizar estado del cluster, gestionar metadata, hacer data queries".
- **Lección:** Attu demuestra que el *mismo* motor puede tener dos consolas: una de **auto-servicio cloud** (Zilliz) orientada a operaciones y una **desktop/self-hosted** (Attu) orientada a desarrollo. Y que los **filters visuales** (single/filtered/grouping/hybrid/full-text/query) son el estándar que los usuarios esperan en una pestaña de búsqueda.

### Chroma

- Docs: `https://docs.trychroma.com/docs/overview/introduction`; producto serverless: `https://trychroma.com/products/chromadb` (vector + full-text + regex + metadata search).
- **Sin GUI oficial.** Toda interacción humana es vía Python/JS SDK o CLI. Chroma "Get" / "Where" (metadata filtering) / "Query" son conceptos de API, no de UI.
- **Lección:** la ausencia de consola es una oportunidad (muchos usuarios de Chroma son exactamente los usuarios de VantaDB: desarrolladores de RAG que quieren ver qué hay en su colección sin abrir un notebook).

### LanceDB

- Sitio: `https://www.lancedb.com/`; repo: `https://github.com/lancedb/lancedb`; docs: `https://docs.lancedb.com/`
- **Sin GUI oficial.** Multimodal lakehouse (embeddings + tabular data en formato columnar Lance). Python/TS/Rust SDKs; integraciones con pandas/duckdb.
- **Lección:** cuando el vector coexiste con datos tabulares, la "consola" natural es un **explorador de tablas/SQL** (DuckDB), no una vista de vectores. Refuerza: los humanos piensan en filas y columnas.

### Redis / RedisInsight (vector sets + RediSearch)

- RedisInsight: `https://redis.io/insight/`; repo (Electron + Monaco + NodeJS): `https://github.com/redis/RedisInsight`
- **Vistas:** Browser (tree de keys), **Workbench** (CLI avanzada con autocompletado, raw mode, visualizaciones), **Profiler** (comandos en tiempo real), **SlowLog**, análisis.
- **Soporte vectorial (README oficial):**
  - **CRUD de vector sets** — crear/manejar vector sets, añadir elementos, **correr vector similarity search** (VSIM).
  - **Vector search** — crear y gestionar índices de búsqueda (RediSearch/Redis Stack) y **consultar los datos indexados**, con **visualizaciones de índices y resultados**.
  - Autocompletado de comandos para search/query, JSON y time series.
- **Vector sets en Redis (tipo nativo, Redis 8.0+):** `https://redis.io/docs/latest/develop/data-types/vector-sets/` — similares a sorted sets pero con vectores en vez de score. Comandos: `VADD`, `VCARD`, `VDIM`, `VEMB`, `VGETATTR`/`VSETATTR` (atributos JSON), `VRANGE`, `VRANDMEMBER`, `VREM`, `VLINKS` (vecinos por capa del grafo HNSW), `VSIM` (búsqueda por similitud con `FILTER` de atributos, ej. `".year > 1950"`). Los docs **visualizan vectores 2D en un plano cartesiano** para explicar la búsqueda.
- **Lección:** RedisInsight es el ejemplo de *GUI desktop* que aporta valor sin necesidad de cloud: profiler, slowlog y visualizaciones de índices son exactamente las "vistas de operaciones" que VantaDB quiere para su audit log y métricas (hnsw_nodes_count, etc.). La idea de "atributos JSON asociados al vector + filtro por atributo" mapea 1:1 a la metadata `VantaValue` de VantaDB.

### pgvector / Supabase

- Guía oficial: `https://supabase.com/docs/guides/ai/vector-columns`; extensión `https://github.com/pgvector/pgvector`
- **Modelo:** tipo `vector(n)` como columna normal; operadores `<->` (euclídea), `<#>` (producto interno negativo), `<=>` (coseno).
- **Supabase Studio:** se habilita la extensión "vector" desde Database > Extensions (un toggle en el dashboard). Las tablas se ven en el **Table Editor**; el vector se muestra como array numérico.
- **Limitación de UI relevante:** **PostgREST no soporta los operadores de similitud**, así que la búsqueda por vector requiere escribir una función SQL (`match_documents(...)`) y llamarla por `rpc()` desde el cliente. Es decir: en Supabase, la búsqueda vectorial **no es una feature de la consola**, es SQL que el usuario escribe.
- **Lección:** cuando ya tienes SQL, el query builder es tu consola. Pero el costo es que el desarrollador no ve el vector como primera clase; es un "truco de extensión". Para VantaDB, que la búsqueda híbrida sea **UI nativa** es diferenciador.

### MongoDB Atlas Vector Search

- Producto: `https://www.mongodb.com/products/platform/atlas-vector-search`; docs: `https://www.mongodb.com/docs/atlas/atlas-vector-search/tutorials/vector-search-quick-start/` y nuevo root `https://www.mongodb.com/docs/vector-search/`
- **Modelo:** stage de agregación **`$vectorSearch`** (ANN o **ENN** exacto), ejecutable dentro del pipeline de MongoDB con **filtros del Query API** (`$eq`, `$gte`, etc.), **hybrid search** con fusión nativa (incluye RRF) de lexical + vector, **native reranking** (`$rerank` con modelos Voyage AI), y **Automated Embedding** (genera y sincroniza embeddings con un "single click" en la consola usando Voyage AI). Límite 4096 dimensiones, quantización scalar/binary.
- **Consola Atlas:** se crean **índices Atlas Search** con campos vectoriales (nombre, dimensión, función de similitud) desde la UI; los documentos (con su embedding dentro) se ven en el **Data Explorer**; las queries con `$vectorSearch` se corren desde la UI o **MongoDB Compass** (`https://www.mongodb.com/products/tools/compass`).
- **Lección:** Mongo no muestra "el espacio vectorial"; muestra **el documento completo** (payload + vector juntos). Su apuesta es "no muevas tus datos: búscalos donde viven". Para VantaDB esto valida mostrar el **record completo** (payload + metadata + vector) en un solo lugar, no separar "datos" y "vectores".

### Vespa

- Docs: `https://docs.vespa.ai/`; nearest-neighbor: `/en/querying/nearest-neighbor-search.html`; ANN HNSW: `/en/querying/approximate-nn-hnsw.html`; CLI: `/en/clients/vespa-cli.html`
- **Sin consola GUI.** Config-as-code (services.xml, schemas, ranking profiles), búsqueda por **YQL** y APIs REST, **Vespa CLI** (`vespa query`, `vespa visit`, `vespa feed`, `vespa document`), métricas vía `/metrics/v1` y Prometheus, "memory visualizer" para performance.
- **Lección:** es posible operar un motor de búsqueda serio sin ninguna GUI. Pero Vespa es de nivel plataforma (ingenieros de búsqueda), no desarrolladores de agentes. VantaDB (embedded, para AI devs) sí necesita una GUI ligera.

### Elastic / Kibana

- Vector search: `https://www.elastic.co/what-is/vector-search`; vector DB: `https://www.elastic.co/elasticsearch/vector-database`; kNN: `https://www.elastic.co/guide/en/elasticsearch/reference/current/knn-search.html`; Kibana: `https://www.elastic.co/kibana`
- **Modelo:** campo `dense_vector` indexado con HNSW (ANN); kNN con filtros inline; **hybrid scoring** BM25F + vector; ESRE (Elasticsearch Relevance Engine) y **AI Playground** (`/demo-gallery/ai-playground`) como GUI de prueba de RAG.
- **Kibana** es la GUI generalista: Discover (exploración de documentos), Dashboards, **Dev Tools Console** (query DSL con autocompletado). No hay una vista dedicada a "vectores" (se ven como campos numéricos dentro del documento).
- **Lección:** el Dev Tools Console con autocompletado y resultados JSON formateados es el patrón de "editor de consultas" más difundido de la industria — barato de construir (un editor + un endpoint) y muy querido por devs.

### Typesense

- Sitio: `https://typesense.org/`; repo: `https://github.com/typesense/typesense`; cloud: `https://cloud.typesense.org/login`; docs: `https://typesense.org/docs/overview/what-is-typesense.html`
- **Typesense Cloud** ofrece un dashboard web (clusters, colecciones, API keys, configuración de búsqueda: typo-tolerance, synonyms, facetas). Enfocado en search UX en vez de "vectores"; admite embeddings/vector search vía API.
- Nota: el detalle fino del dashboard de Typesense Cloud no fue verificado en detalle en esta sesión; se cita la URL de login y el dominio de docs.

---

## Patrones de UI recurrentes

Extraídos de todas las herramientas analizadas:

1. **Jerarquía anidada proyecto → cluster → índice/colección** (Pinecone, Zilliz, Weaviate, Qdrant Cloud, Atlas). El usuario siempre navega en tres niveles; cada nivel tiene su "home" de estado.
2. **Búsqueda por similitud como acción de primera clase**: una caja de input (vector pegado o texto) en la vista principal del índice/colección, con selector de top-k y métrica (Qdrant Console, Attu, Zilliz search tabs, Weaviate Explorer, RedisInsight VSIM).
3. **Payload/metadata legible**: JSON formateado con colores y colapsable como estándar para mostrar el contenido del registro (Qdrant, Attu, MongoDB Data Explorer, Kibana Discover). El vector, en cambio, es un array numérico opaco o está oculto.
4. **Explorador de registros sin código** (Weaviate Explorer, Attu browse, Mongo Data Explorer, Supabase Table Editor, Kibana Discover): paginación, búsqueda/filtro básico, click para detalle.
5. **Playground de API / editor de consultas embebido** (Qdrant Console, Kibana Dev Tools, RedisInsight Workbench, Zilliz console, Atlas aggregation builder): el usuario pega JSON/query y corre contra el server real, con resultados formateados.
6. **Monitoreo con charts** (Pinecone Metrics, Zilliz metric charts + alerts, Weaviate cluster status, RedisInsight Profiler/SlowLog, Atlas): latencia, QPS, dimensiones, estado de índice, uso de recursos.
7. **Backups / snapshots desde la UI** (Qdrant upload snapshot, Zilliz scheduled backups, Attu backups).
8. **Tutorial / quickstart embebido** (Qdrant `#/tutorial`, RedisInsight interactive tutorials).
9. **Audit logs y logs de acceso** (Zilliz Auditing/Access Logs) — necesario para cualquier consola multi-usuario.
10. **Import/export de datos** (Zilliz Import & Export, Weaviate CSV/PDF wizard, Qdrant bulk upload).

## Anti-patrones y críticas honestas

1. **El vector como ciudadano de segunda clase**: casi todas tratan el embedding como "un campo más" o lo esconden. Nadie deja *ver* qué significa la similitud (sin proyección 2D, sin "¿por qué matchea esto?"). Crítica directa a Pinecone, Qdrant, Atlas, Elastic.
2. **Pinecone no tiene explorador de datos**: el usuario no puede ver qué hay en su índice desde la web. Resulta frustrante en debugging ("¿se insertó mi punto?"). Es el contra-ejemplo de "operar sin tocar datos".
3. **Consolas "REST-only"** (Qdrant Console, Pinecone): pegar JSON crudo sin validación es potente pero hostil; sin autocompletado ni guards, el error se descubre al correr. Kibana Dev Tools lo hace bien; Qdrant no.
4. **Falsas GUI / dashboards de humo**: páginas que solo muestran estado del cluster y esconden las operaciones de datos bajo SDKs (Chroma, LanceDB, Vespa — directamente sin GUI). Deja al dev sin diagnostico visual.
5. **Lock-in del layout "índice/vista"**: cuando el único mental model es "colección de vectores", el payload/metadata (la parte que el humano lee) queda relegado a un drawer colapsable. Weaviate y Attu lo resuelven con tabs explícitos.
6. **Búsqueda vectorial como feature SQL escondida** (Supabase/pgvector): la similitud no es UI, es una función SQL que el usuario escribe. Confirma que "vector search sin UI" es un coste, no una virtud.
7. **Consolas cloud-first que excluyen localhost** (Pinecone, Zilliz, Weaviate Cloud): si tu producto es embedded/local (VantaDB), la consola debe vivir *en el puerto local* (como Qdrant `:6333/dashboard` o RedisInsight desktop), no requerir cuenta.
8. **Sobrecarga de ops en detrimento de datos**: Zilliz/Atlas muestran billing, orgs, roles, regiones, backup policies antes que *qué hay dentro de la colección*. Para un desarrollador de IA, lo segundo es lo primero.

## Lecciones aplicables a VantaDB

1. **Explorador de registros como núcleo (obligatorio)**: tabla paginada de `VantaMemoryRecord` con columnas key/namespace/updated_at/version/vector(bool)/payload preview; click para detalle con payload formateado (tipo `VantaValue` coloreado por tipo: string/int/float/bool/datetime/list/null), vector como array expandible, y `node_id`/timestamps visibles. Copiar de Weaviate Explorer + Attu browse.
2. **Search híbrida como feature de UI nativa (diferenciador)**: una pestaña "Buscar" con (a) texto para BM25, (b) vector pegado o *picker de un registro existente* para semantic, (c) combinación híbrida con slider de pesos, (d) top-k y umbral de score, (e) **filtros visuales por metadata** (`VantaValue` por tipo) sin escribir JSON — inspirado en Zilliz "Filtered/Grouping/Hybrid Search" tabs y Redis VSIM FILTER.
3. **Namespaces como primer selector**: dado que VantaDB usa namespaces como aislamiento, la sidebar izquierda debe listar namespaces (con conteo de registros y métricas) como primer nivel — antes que "índice". Es el equivalente a Project/Cluster/Collection de las otras consolas, pero a escala local.
4. **Audit log y métricas como vistas dedicadas**: tabs de *Activity* (audit log JSONL, filtrable por namespace/operación/resultado — el equivalente al Profiler/SlowLog de RedisInsight) y *Stats* (hnsw_nodes_count, dimensiones, TTL próximos a expirar, conteos por namespace) con charts simples. No hace falta Datadog; con 3-4 charts servidos desde el propio motor alcanza.
5. **Editor de consultas doble**: un **IQL console** (playground del Query Language de la capa de grafo, estilo Kibana Dev Tools / Qdrant Console con autocompletado) + los formularios visuales de la lección 2. El console se vende solo para power-users; los forms para el resto.
6. **Grafos visibles**: la capa de grafo (aristas, acumuladores, IQL) es única de VantaDB — una vista "Graph" con nodos/aristas clickeables (o una lista de aristas del nodo) sería funcionalidad que ninguna otra consola de vector DB tiene. No hay precedente que copiar; empezar por lista/table de aristas dirigidas con peso y acumuladores.
7. **Local-first sin login**: la consola debe servirse como Qdrant (`:6333/dashboard`) desde el propio proceso VantaDB (y desde WASM/OPFS en el navegador), con cero registro. La ruta cloud/login queda fuera de alcance.
8. **Import/export en la UI**: drag & drop de `.vdbdump`/JSONL (patrón Qdrant snapshots), porque el ciclo "respaldar/restaurar memoria de un agente" es operación frecuente y muy valorada.
9. **TTL visible**: dado que VantaDB soporta `expires_at_ms`, la consola debería mostrar registros próximos a expirar y permitir ajustar el TTL desde la UI (nadie más lo hace — es un mini-diferenciador de "memoria de agente").
10. **No visualizar los embeddings como proyección (evitar scope creep)**: ninguna consola madura lo hace en producción; es caro y dudoso en >2D. Mejor: mostrar el vector como datos (preview, stats de norma/dimensiones) y **explicar similitud con scores y co-occurrence**, no con scatter plots.
11. **Formularios de inserción/edición**: la operación *put* debe tener su form (key, namespace, payload, metadata por tipo, vector opcional con botón "generar con modelo local" — mapea al Integrated Embedding de Zilliz/Atlas auto-embedding). El humano quiere *ver* el dato que va a insertar.
12. **Pagination + virtualización desde el día uno**: las tablas de payload/texto crecen; planificar lazy-load desde la primera iteración.

## Referencias

Todas las URLs fueron verificadas/consultadas durante esta investigación (agosto 2026).

**Pinecone**
- https://app.pinecone.io/organizations/-/projects
- https://docs.pinecone.io/llms.txt
- https://docs.pinecone.io/guides/production/monitoring.md

**Weaviate**
- https://weaviate.io/llms.txt
- https://docs.weaviate.io/cloud/tools/explorer-tool
- https://weaviate.io/product/explorer.md
- https://docs.weaviate.io/cloud/tools/query-tool
- https://docs.weaviate.io/cloud/tools/collections-tool
- https://docs.weaviate.io/cloud/manage-clusters/status

**Qdrant**
- https://qdrant.tech/documentation/web-ui/
- https://qdrant.tech/documentation/cloud-quickstart/
- https://cloud.qdrant.io
- https://api.qdrant.tech/api-reference
- https://qdrant.tech/documentation/manage-data/

**Milvus / Zilliz / Attu**
- https://github.com/zilliztech/attu
- https://zilliz.com/attu
- https://milvus.io/docs/quickstart_with_attu.md
- https://docs.zilliz.com/ (subpáginas: /docs/collection, /docs/manage-indexes, /docs/search-query-get, /docs/view-cluster-metric-charts, /docs/create-backup, /docs/manage-snapshots, /docs/auditing, /docs/access-logs)
- https://cloud.zilliz.com/login

**Chroma**
- https://docs.trychroma.com/docs/overview/introduction
- https://trychroma.com/products/chromadb

**LanceDB**
- https://www.lancedb.com/
- https://github.com/lancedb/lancedb
- https://docs.lancedb.com/

**Redis / RedisInsight**
- https://redis.io/insight/
- https://github.com/redis/RedisInsight
- https://redis.io/docs/latest/develop/data-types/vector-sets/
- https://redis.io/commands/vsim

**pgvector / Supabase**
- https://supabase.com/docs/guides/ai/vector-columns
- https://github.com/pgvector/pgvector
- https://supabase.com/dashboard/project/_/database/tables

**MongoDB Atlas Vector Search**
- https://www.mongodb.com/products/platform/atlas-vector-search
- https://www.mongodb.com/docs/atlas/atlas-vector-search/tutorials/vector-search-quick-start/
- https://www.mongodb.com/docs/vector-search/
- https://www.mongodb.com/products/tools/compass

**Vespa**
- https://docs.vespa.ai/
- https://docs.vespa.ai/en/querying/nearest-neighbor-search.html
- https://docs.vespa.ai/en/querying/approximate-nn-hnsw.html
- https://docs.vespa.ai/en/clients/vespa-cli.html

**Elastic / Kibana**
- https://www.elastic.co/what-is/vector-search
- https://www.elastic.co/elasticsearch/vector-database
- https://www.elastic.co/guide/en/elasticsearch/reference/current/knn-search.html
- https://www.elastic.co/kibana
- https://www.elastic.co/demo-gallery/ai-playground

**Typesense**
- https://typesense.org/
- https://github.com/typesense/typesense
- https://cloud.typesense.org/login
- https://typesense.org/docs/overview/what-is-typesense.html
