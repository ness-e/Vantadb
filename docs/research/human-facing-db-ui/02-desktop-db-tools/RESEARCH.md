# Herramientas desktop de administración de bases de datos

> Investigación de patrones de UI en herramientas desktop/self-hosted de administración de bases de datos (2024–2026), orientada al diseño del módulo desktop de VantaDB (Tauri v2 + React + Vite).
>
> **Público objetivo de VantaDB**: desarrolladores de IA, no DBAs expertos. La UI debe ser explorable, con edición segura y feedback visible, sobre un modelo de datos de memoria persistente (namespaces, registros con payload JSON + metadata, vector embebido, TTL, versionado, grafos).

## Resumen ejecutivo

Tras analizar 15+ herramientas (DBAs relacionales, GUI NoSQL, navegadores de grafos, exploradores de almacenamiento y herramientas de memoria para IA), los patrones dominantes y transferibles son:

1. **Layout**: hay dos familias. (a) *IDE/árbol* — panel lateral de conexiones/colecciones + grid central de datos (DBeaver, TablePlus, Beekeeper, Compass). (b) *Foco en el dato* — la herramienta gira alrededor del documento/registro y su inspección (Compass document editor, Redis Insight, Fauxton, OpenMemory). VantaDB, por ser un *memory store* jerárquico plano (namespace → registro), se beneficia más de la familia (b): navegación por namespace + lista + panel de detalle del registro.
2. **Editor de valores**: el estándar es **edición inline en grid** con **confirmación explícita** (commit/revert), no auto-guardado. JSON se edita en editor dedicado con syntax highlighting (Compass, Fauxton, TablePlus Quick look). Los campos tipo vector/numéricos y fechas (TTL, timestamps) tienen tratamientos visuales propios (etiquetas, badges, monospace).
3. **Búsqueda/filtros**: barra de filtro con *query builder* visual o texto JSON/MQL (Compass), filtros compuestos columna+operador+valor (TablePlus), historial y **queries favoritas** (Compass). El patrón de "click en un valor del esquema → rellena el filtro" (Schema tab de Compass) es muy potente para descubrimiento.
4. **Grafos**: navegación por nodos con listado lateral + búsqueda + expandir relaciones on-demand (Obsidian graph view, Memgraph Lab, Surrealist explorer). El grafo no es el layout principal — es una *vista* a la que se entra cuando el usuario explora relaciones.
5. **Anti-patrones** (a evitar): grid monocromo sin estados de celda modificada (DBeaver legacy lo resolvió con color de fondo por estado), edición sin confirmación que escribe a disco al vuelo, JSON sin pretty-print ni highlight, tablas infinitas sin paginación/cursor visible, safe-mode ausente en herramientas de edición.

## Comparativa

| Herramienta | Tipo DB | Layout | Editor valores | Búsqueda/filtros | Grafos | Rating |
|---|---|---|---|---|---|---|
| DBeaver | SQL (multi) | IDE árbol + grid + editor SQL | Grid inline + entity editor; color de fondo por estado de celda | Filtros de grid, data filters, autocompletado | ER diagram | 9/10 |
| TablePlus | SQL + Redis | Native multi-tab + grid spreadsheet | Inline edit + row detail sidebar + Quick look (JSON pretty/BLOB) | Filtros columna+condición+valor, multi-filtro, ⌘P open anything | ER diagram | 9/10 |
| Beekeeper Studio | SQL (multi) | Tabbed, ligero | Grid con edición inline | SQL editor + filtros | — | 8/10 |
| MongoDB Compass | MongoDB | Navegador colecciones + Documents tab | Document editor (JSON view) + Insert modal | Query bar con operadores MQL, favoritos, historial, Schema tab → filtro | — | 9/10 |
| Redis Insight | Redis | Browser de keys por tipo | CRUD por tipo (hash/set/list/zset/stream), batch delete | Search por patrón, Redis Copilot | — | 8/10 |
| DB Browser for SQLite | SQLite | Árbol de tablas + tabs (Browse/Execute/Edit) | Edición de registros con vista de estructura | SQL query + filtros | — | 7/10 |
| SQLiteStudio | SQLite | Árbol + tabs | Edit grid | SQL | — | 7/10 |
| Neo4j Browser | Neo4j | Command bar + resultados | Resultados de query (table/graph/text) | Cypher | Grafo resultados | 7/10 |
| Neo4j Bloom | Neo4j | Perspectives visuales | — | Búsqueda low-code con estilo | Grafo visual (primer plano) | 8/10 |
| Memgraph Lab | Memgraph | Explorer de grafos | Node/edge props editables | Search, GraphChat (NL), Cypher | Grafo + schema visual + GSS | 8/10 |
| Surrealist/SurrealDB Studio | SurrealDB | Explorer + Designer + Query playground | Record edit + relación traversal | Query con autocomplete | Traversal de relaciones | 8/10 |
| CouchDB Fauxton | CouchDB | Web admin | JSON editor con syntax highlight, clone, delete, attachments | Find/Mango | — | 6/10 |
| Azure Storage Explorer | Blobs/tables/queues | Árbol izquierdo (containers) + detalles | Edit de entities/tables | Filtros por contenedor/cola | — | 8/10 |
| Datasette + CSV importer | SQLite web | Lista de tablas + vista row | Formularios vía plugin | SQL + facets | — | 7/10 |
| Obsidian Graph view | Markdown/PKM | Canvas | Note editor | Search + backlinks | Grafo global/local + inline local graph | 8/10 |
| OpenMemory (Mem0) | Memoria IA | Dashboard | Add/browse/delete memories | Search | — | 6/10 |
| Letta memory blocks | Memoria IA | Agent view + block editor | Edición de bloques de memoria | Search | — | 6/10 |
| Open WebUI | LLM + RAG | Admin dashboard | Documentos, embeddings, chats | Search vectorial | — | 7/10 |
| Mem0 Studio | Memoria IA | Memory Explorer | Lista/borrado de memorias | — | — | 5/10 |

## Análisis detallado por herramienta

### DBeaver (SQL multi-engine)
- Grid de resultados estilo hoja de cálculo; editor de entidades con **Save/Revert** explícito; el grid colorea el **fondo de la celda según su estado** (modificada, nueva, eliminada) para que el diff pendiente sea visible antes de commit.
- Exportación a Excel con descripciones de columnas.
- URLs: https://dbeaver.io/news/page/22 · https://www.devtoolreviews.com/reviews/tableplus-vs-dbeaver-vs-datagrip

### TablePlus (SQL multi-engine + Redis)
- Grid spreadsheet con **edición inline** (doble clic) + panel lateral de fila (toggle con `Space`) para ediciones largas o bulk.
- **Quick look** (clic medio / menú contextual): abre el valor en popup más grande — crítico para JSON en pretty format y BLOBs.
- Filtro compuesto: dropdown de columna + condición (`equal`, `contain`, `IS NULL`…) + campo de valor; múltiples filtros a la vez.
- **Commit explícito** (`⌘S` o botón) — no auto-guarda. Historial de cambios + 3 niveles de **safe mode**.
- `⌘P` **Open anything** (tabla, view, función…); multi-tabs por conexión con **color-coded labels** por entorno.
- Copy rows as: JSON, Markdown, CSV, SQL insert.
- URLs: https://docs.tableplus.com/getting-started · https://docs.tableplus.com/gui-tools/working-with-table/row · https://tableplus.com/blog/2018/05/11-tips-to-boost-productivity-with-tableplus.html · https://appmus.com/software/tableplus · https://toolsinfo.com/compare/beekeeper-studio-vs-tableplus · https://www.c-sharpcorner.com/article/essential-features-of-tableplus-the-best-gui-tool-to-manage-mysql-postgresql

### Beekeeper Studio
- Multi-tab ligero, SQL editor con autocompletado, export CSV/JSON, SSH tunneling. Menos maduro que TablePlus en editing pero limpio para el público developer.
- URL: https://toolsinfo.com/compare/beekeeper-studio-vs-tableplus

### MongoDB Compass
- **Query bar** (filtro MQL en JSON), opciones desplegables (Filter/Project/Sort/Skip/Limit), favoritos e historial (My Queries tab).
- **Schema tab**: muestreo de documentos con tipos de campo, cardinalidad y distribuciones; **click en un valor → rellena el filtro**. Patrón clave de descubrimiento.
- **Document editor** con vista JSON (`{}`) para insert/editar (los primeros Compass lo hicieron mal — ver anti-patrones); agregation pipeline builder con preview por etapa; index management; explain plan; monitoreo de performance; AI query generation (NL).
- URLs: https://www.mongodb.com/docs/compass/query/filter/ · https://www.mongodb.com/docs/compass/query/queries · https://oneuptime.com/blog/post/2026-03-31-mongodb-compass-gui/view · https://oneuptime.com/blog/post/2026-03-31-mongodb-build-queries-compass/view · https://mongodb-clone.onrender.com/www.mongodb.com/products/tools/compass.html

### Redis Insight
- **Browser** de keys agrupadas por tipo con CRUD completo y **batch operations** (delete múltiple); TTL visible y editable por key.
- Profiler de comandos, **slow log**, database analyzer; Workbench/CLI integrado; **Redis Copilot** (assistente NL).
- URLs: https://redis.io/insight/ · https://redis.io/docs/latest/develop/tools/insight

### DB Browser for SQLite / SQLiteStudio
- Árbol de tablas + pestañas Browse/Execute/Edit. Visual DB Browser: diseño de esquema visual, edición de registros; SQLiteStudio: portable, GPL. Sin grandes innovaciones de UI — valiosos como baseline de "DB tool simple".
- URL: https://appmus.com/vs/sqlite-database-browser-vs-sqlitestudio

### Neo4j Browser / Neo4j Bloom
- Browser: **command bar** (`:play`, Cypher) + resultados en vistas table/graph/text. Bloom: **Perspectives** basadas en estilo, búsqueda low-code, visualización en primer plano.
- Lección: en DBs de grafos la búsqueda/consulta es la puerta; la vista de grafo es el resultado, no el layout base.
- URL: https://neo4j.com/blog/developer/scoobygraph-3

### Memgraph Lab
- Explorer con visualización de nodos/edges, **graph schema**, layout algorithms, búsqueda, edición de propiedades, import CSV, monitoreo; **GraphChat** (NL→Cypher). Multi-tenancy.
- URL: https://memgraph.com/docs/memgraph-lab

### Surrealist / SurrealDB Studio
- **Explorer view**: browse tablas, editar records, **traverse graph relationships** del record. **Designer view**: esquema. Query playground con autocomplete.
- URLs: https://github.com/surrealdb/surrealist · https://github.com/Andrewknackstedt/Surrealist

### CouchDB Fauxton
- Admin web en `localhost:5984/_utils/`: **document editor JSON con syntax highlighting**, clone, delete, attachments; vistas de estructura.
- URLs: https://deepwiki.com/apache/couchdb-fauxton/5.1.2-document-editor · https://adhdecode.com/articles/couchdb/couchdb-fauxton-ui-admin

### Azure Storage Explorer
- **Árbol izquierdo** de containers/blobs/files/queues/tables/Data Lake/managed disks + panel de detalle; permisos, tiers, copia. Referencia de organización de múltiples "tipos de objeto" navegables en un solo panel.
- URLs: https://azure.microsoft.com/en-us/products/storage/storage-explorer/ · https://oneuptime.com/blog/post/2026-02-16-how-to-use-azure-storage-explorer-to-manage-blobs-files-queues-and-tables/view

### Datasette (+ plugin CSV importer)
- Web app de exploración SQLite con facets y row view; el plugin de import CSV añade UI de upload con live import. Referencia para la ingesta por arrastrar/soltar de datos semiestructurados.
- URL: https://github.com/next-LI/datasette-csv-importer

### Obsidian (Graph view + Inline Local Graph)
- **Global graph**, **local graph** (vecinos del nodo activo) y plugin **Inline Local Graph** (vis-network.js embebido). Search con backlinks, tags. El modelo "note ↔ nodo, link ↔ edge, backlink ↔ relación" es el análogo conceptual más cercano al grafo de memoria de VantaDB.
- URLs: https://obsidian.md/help/plugins/graph · https://community.obsidian.md/plugins/inline-local-graph · https://www.obsidianstats.com/tags/graph-view

### Herramientas de memoria IA (OpenMemory/Mem0 Studio, Letta, Open WebUI, Supermemory)
- OpenMemory (dashboard: add/browse/delete memories) + OpenMemory MCP; Mem0 Studio (Memory Explorer); Letta memory blocks (persona/human/custom/file, compartidos entre agentes); Open WebUI (documentos, embeddings, vector search, RBAC, export conversaciones); Supermemory (research técnico).
- Lección: las herramientas de memoria IA aún son **muy básicas** (listas planas + borrar). Es una oportunidad para que VantaDB supere el estado del arte con detalle de registro, metadata, TTL y grafo.
- URLs: https://github.com/fenmo-ai/mem0/blob/main/docs/openmemory/overview.mdx · https://mem0.ai/blog/introducing-openmemory-mcp · https://github.com/anshukr96/mem0-studio · https://deepwiki.com/letta-ai/letta/2.4-memory-block-management · https://gist.github.com/monotykamary/89226f685c17841d4d910c30b6b88442 · https://www.vps.org/docs/apps/open-webui?lang=yo · https://github.com/Lin-Guanguo/llm-memory-research/blob/main/supermemory.research.md

## Patrones de UI para data editing

1. **Grid con edición inline + commit explícito** (TablePlus, DBeaver, DataGrip): doble clic edita; la celda cambia de fondo (estado modificado); el usuario confirma o revierte. Nunca auto-guardar en edición de datos.
2. **Row detail sidebar** (TablePlus `Space`, Compass document editor): para registros con muchos campos o JSON grandes, un panel lateral/expansión que muestra el registro completo con tipos visibles.
3. **JSON pretty + syntax highlight + validación** (Compass JSON view, Fauxton, TablePlus Quick look): todo valor JSON se abre en editor dedicado, no en una celda de una línea. Aceptar pegar un JSON entero para insertar.
4. **Quick look / popup de valor** (TablePlus): clic medio o menú contextual abre el valor grande — esencial para payloads y metadata largos.
5. **Tratamiento por tipo de dato**: timestamps/TTL como etiquetas legibles (no epoch crudo); números y vectores en monospace; valores vacíos/ausentes diferenciados visualmente.
6. **Insert modal con JSON** (Compass "Add Data → Insert Document"): pegar JSON en un modal con preview validado es más rápido que construir campos uno a uno.
7. **Copy as…** (TablePlus): copiar fila como JSON/Markdown/CSV — muy útil para desarrolladores que pegan datos en código.
8. **Historial y favoritos de consultas** (Compass, TablePlus): la búsqueda usada a menudo se guarda, se nombra y se reutiliza.
9. **Batch operations** (Redis Insight): seleccionar múltiples registros y borrar/exportar en lote con confirmación.

## Patrones de UI para grafos

1. **Grafo como vista, no como layout base** (Memgraph Lab, Obsidian local graph, Surrealist): el usuario llega al grafo desde un registro/nodo o una búsqueda; el panel principal sigue siendo lista/detalle.
2. **Expandir relaciones on-demand**: click en un nodo → expande vecinos (local graph de Obsidian), en vez de pintar todo el grafo de una vez.
3. **Lista lateral de nodos/resultados + detalle del nodo**: doble panel (explorer de Memgraph Lab): nodos relacionados en lista, selección abre props editables.
4. **Schema/panorama visual** (Memgraph graph schema, Compass Schema tab): una vista agregada de qué tipos de nodos/edges existen y con qué frecuencia.
5. **Layout algorithms**: opciones de layout (force-directed, etc.) con búsqueda para centrar el nodo de interés.
6. **GraphChat / NL→query** (Memgraph): convertir lenguaje natural a consulta de grafo — alta fricción cubierta para DBs de grafo.

## Anti-patrones y críticas

1. **Edición que escribe al vuelo sin confirmación** — riesgo de corrupción de datos; todas las buenas herramientas usan commit explícito.
2. **Grid monocromo sin estado de celda** — DBeaver introdujo color de fondo por estado porque sin él no se ve qué cambió antes de guardar.
3. **JSON en celda de grid sin pretty-print** — ilegible; requiere Quick look o editor dedicado.
4. **Insertar JSON pegando campo a campo** — UX de Compass temprano, queja histórica real en Stack Overflow; hoy se resuelve con vista JSON.
5. **Paginación invisible/infinita** — "Load more" sin indicación de total o cursor frustra la navegación en datasets medianos (relevante para DataExplorer de VantaDB).
6. **Safe mode ausente** — borrar un namespace/registro sin protección; TablePlus ofrece 3 niveles de safe mode.
7. **Herramientas de memoria IA demasiado planas** — listas sin detalle, sin metadata, sin TTL, sin grafo; no son el modelo a copiar, son el *gap* a superar.

## Lecciones aplicables al módulo desktop de VantaDB

Estado actual del módulo (referencia): `desktop/src/App.tsx` es una sola página con paneles apilados (MetricsGrid, KpiCards, ExportPanel, SopPanel, ConnectionPanel, IngestForm, SearchBar, DataExplorer, ProcessPanel, ResultsList). `DataExplorer.tsx` lista registros con paginación "Load more" (limit 50→100…, sin cursor), `ResultsList.tsx` muestra id/score/text/namespace. No hay vista de detalle de registro, ni edición, ni metadata visible, ni vista de grafo.

Lista concreta, priorizada (P0 = core para la próxima iteración):

1. **[P0] Vista de detalle de registro** (doble panel). Click en una fila de DataExplorer/ResultsList abre un panel con: key, namespace, version, created_at/updated_at legibles, TTL con countdown, payload pretty-print, metadata tabla clave/valor tipada, badge de vector (presente/dimensión). Modelo: TablePlus row sidebar + Compass document editor. Aplica a `ResultsList.tsx`/`DataExplorer.tsx`.
2. **[P0] Editor JSON con confirmación** para payload y metadata: modal/panel con JSON pretty + highlight + validación + botones Guardar/Revertir (commit explícito). Modelo: Compass JSON view + Fauxton. No auto-guardar.
3. **[P0] Filtros compuestos en DataExplorer**: namespace (dropdown, ya existente), rango de fecha por `created_at_ms`/`updated_at_ms`, filtro por `expires_at_ms` (solo no expirados / expirando), texto en payload/metadata. Modelo: filtro columna+condición+valor de TablePlus. Remplazar el patrón "Load more" por paginación con indicador de total/cursor cuando el core lo soporte (hoy `vanta_list` solo acepta namespace+limit — ver `desktop/src/vanta.ts`).
4. **[P1] Estados visuales de celda + TTL badges**: colorear celdas modificadas en edición (DBeaver); badges de tipo de dato y de TTL (Redis Insight muestra TTL por key). Monospace para `node_id`, timestamps legibles.
5. **[P1] Batch operations** con confirmación: selección múltiple → borrar registros, exportar selección (JSONL). Modelo: Redis Insight batch delete; ExportPanel puede aceptar "selección actual".
6. **[P1] Historial y favoritos de búsqueda**: persistir las últimas búsquedas (query + top_k + filtros) y permitir guardar/nombrar favoritas. Modelo: Compass My Queries. Encaja sobre `SearchBar.tsx`.
7. **[P2] Vista de grafo como "vista" del nodo**: desde el detalle de un registro, ver nodos conectados (IQL graph layer) con panel lateral de vecinos + expandir on-demand. Modelo: Obsidian local graph + Memgraph Lab explorer. No reemplaza el layout principal.
8. **[P2] Import de CSV/JSON pegado**: ingestar registros pegando JSON o arrastrando CSV (datasette-csv-importer) hacia IngestForm, con preview validado.
9. **[P2] Safe mode** para operaciones destructivas (borrar namespace, limpiar todo): confirmación explícita con texto del objetivo. Modelo: TablePlus 3 niveles.
10. **[P3] Open anything (⌘P)** para saltar a un namespace/key rápido (TablePlus) — barato y muy developer-friendly.
11. **[P3] Copy as…** en resultados: copiar registro como JSON (ya hay texto en ResultsList; falta el copy de metadata/payload completo).

## Referencias

- https://dbeaver.io/news/page/22
- https://www.devtoolreviews.com/reviews/tableplus-vs-dbeaver-vs-datagrip
- https://docs.tableplus.com/getting-started
- https://docs.tableplus.com/gui-tools/working-with-table/row
- https://tableplus.com/blog/2018/05/11-tips-to-boost-productivity-with-tableplus.html
- https://appmus.com/software/tableplus
- https://toolsinfo.com/compare/beekeeper-studio-vs-tableplus
- https://www.c-sharpcorner.com/article/essential-features-of-tableplus-the-best-gui-tool-to-manage-mysql-postgresql
- https://www.mongodb.com/docs/compass/query/filter/
- https://www.mongodb.com/docs/compass/query/queries
- https://oneuptime.com/blog/post/2026-03-31-mongodb-compass-gui/view
- https://oneuptime.com/blog/post/2026-03-31-mongodb-build-queries-compass/view
- https://mongodb-clone.onrender.com/www.mongodb.com/products/tools/compass.html
- https://redis.io/insight/
- https://redis.io/docs/latest/develop/tools/insight
- https://appmus.com/vs/sqlite-database-browser-vs-sqlitestudio
- https://neo4j.com/blog/developer/scoobygraph-3
- https://memgraph.com/docs/memgraph-lab
- https://github.com/surrealdb/surrealist
- https://github.com/Andrewknackstedt/Surrealist
- https://deepwiki.com/apache/couchdb-fauxton/5.1.2-document-editor
- https://adhdecode.com/articles/couchdb/couchdb-fauxton-ui-admin
- https://azure.microsoft.com/en-us/products/storage/storage-explorer/
- https://oneuptime.com/blog/post/2026-02-16-how-to-use-azure-storage-explorer-to-manage-blobs-files-queues-and-tables/view
- https://github.com/next-LI/datasette-csv-importer
- https://obsidian.md/help/plugins/graph
- https://community.obsidian.md/plugins/inline-local-graph
- https://www.obsidianstats.com/tags/graph-view
- https://github.com/fenmo-ai/mem0/blob/main/docs/openmemory/overview.mdx
- https://mem0.ai/blog/introducing-openmemory-mcp
- https://github.com/anshukr96/mem0-studio
- https://deepwiki.com/letta-ai/letta/2.4-memory-block-management
- https://gist.github.com/monotykamary/89226f685c17841d4d910c30b6b88442
- https://www.vps.org/docs/apps/open-webui?lang=yo
- https://github.com/Lin-Guanguo/llm-memory-research/blob/main/supermemory.research.md