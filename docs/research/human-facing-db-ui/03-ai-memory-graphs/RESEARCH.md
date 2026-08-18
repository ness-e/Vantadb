# Visualización de memoria de agentes de IA y knowledge graphs

Cómo los sistemas de memoria para agentes de IA y los knowledge graphs hacen VISIBLE y ADMINISTRABLE su datos internos a humanos. Investigación de escritorio con URLs reales (2024-2026). Contexto objetivo: VantaDB (base de datos embebida de memoria persistente para agentes: registros con texto + metadata + embeddings, híbrido BM25+HNSW+RRF, capa de grafo con IQL, TTL, audit log, import/export).

---

## Resumen ejecutivo

El estado del arte en "human-facing" para memoria de agentes se divide en cuatro familias:

1. **Dashboard de memorias (cards/listas)** — Mem0 (plataforma y servidor self-hosted), ChatGPT "Memory summary" + "Saved memories", Reflect. El mínimo común: listas paginadas de hechos con **búsqueda semántica, filtros por usuario/agente/etiqueta, edición y borrado manual**. La mayoría son listas planas de cards; casi ninguno ofrece timeline.

2. **Inspector de agente / white-box** — Letta ADE / desktop app, LangGraph Studio. Ventana al **contexto completo** que ve el agente: bloques de memoria editables, archivo de contexto, estados entre pasos, "time-travel" sobre checkpoints. La edición manual es directa (incluso el propio agente se edita a sí mismo vía tool calls).

3. **Knowledge graph temporal** — Zep (Graphiti), Cognee, Microsoft GraphRAG, LightRAG, Neo4j Bloom. Nodos = entidades, aristas = hechos/relaciones **con validez temporal** (valid_at / invalid_at / episodes). La visualización estrella es el **grafo force-directed con clustering de comunidades** (colores), tamaño de nodo por centralidad, búsqueda + highlight y "community isolator" (zoom a subgrafo). Ningún producto comercial de memoria de agentes tiene un grafo 2D bien resuelto de serie; Zep tiene dashboard con visualización de grafo y un repo de referencia (D3), Cognee/LightRAG sí lo traen en su WebUI.

4. **Observabilidad de memoria (traces)** — LangSmith, Braintrust, Laminar, OpenTelemetry GenAI. El patrón consolidado es: **span por operación de memoria** (`memory.retrieve` / `memory.store`) que registra query, candidatos devueltos, scores de relevancia y qué se escribió, con metadata de frescura. Es la única forma real de responder "¿por qué recuperó X?" y "¿por qué NO recuerda Y?".

**Hallazgos clave para VantaDB:**

- **Explicabilidad de retrieval = la feature más demandada.** Mem0 expone `explain=True` en search (desglosa señal semántica + keyword + entidad). ChatGPT muestra "sources" con un icono de libro y una explicación de *por qué* se usó cada memoria. Zep mantiene *provenance* (cada hecho apunta a los episodios/chat turns que lo crearon) y re-rank con "relevance, recency, provenance quality". LangSmith/Braintrust instrumentan scores en el span. VantaDB ya tiene los datos (scores BM25, HNSW, RRF, audit log) pero no una vista que los componga.
- **El grafo solo sirve si se puede navegar sin llegar al "hairball"** (bola de pelos): los sistemas que funcionan (GraphRAG Workbench, LightRAG, InfraNodus) combinan layout force-directed + clustering por color + filtros por tipo de nodo + búsqueda con highlight + drill-down por comunidad + detalle de nodo/arista en panel lateral.
- **Edición manual y gestión del ciclo de vida son el diferenciador de producto** (no el almacenamiento): borrar/editar/desactivar memorias, TTL y "decay" (Mem0), olvido/consolidación (Cognee memify, Graphiti conflict detection, ChatGPT auto-update), y export/portabilidad (Reflect, Letta MemFS con git).
- **Anti-patrón dominante:** memoria como caja negra sin UI (Mem0 OSS: "no built-in way to see what agents remember") — los equipos terminan construyendo dashboards ad-hoc contra el vector store.

---

## Comparativa

| Sistema | Cómo muestra la memoria | Explicabilidad retrieval | Grafo? | Edición manual | Rating |
|---|---|---|---|---|---|
| **Mem0 (Platform/OSS dashboard)** | Cards en dashboard: Memory Browser con búsqueda semántica, filtros por user/agent/run, `threshold`, `top_k`; logs de requests | Sí: `explain=True` desglosa scores (semántico+keyword+entidad); confianza por memoria | Sí (Platform): Graph Memory, entidades→nodos, `relations` en search; sin visor 2D | Sí: editar/borrar por ID, cascade-delete por entidad, expiration_date, memory decay | 7/10 |
| **Letta (ADE / desktop)** | "Memory palace": bloques de memoria (core memory) y archival memory editables; Context Window Viewer; desktop app con **graph view de MemFS** (archivos de memoria con refs entre sí, clic→editar) | "White-box": ves el contexto/prompts exactos que entran al LLM en cada paso | Parcial: graph view de referencias entre archivos de memoria (no KG semántico) | Alta: edición directa de bloques, import/export .af, el agente se auto-edita | 8/10 |
| **Zep (Graphiti)** | Dashboard con graph visualization, debug logs, API logs, analytics, "Observations"; Context Lake | Alta: provenance por episodios (cada hecho → chat turn que lo creó); rerank por relevance/recency/provenance; search devuelve entities+facts+episodes | Sí: Context Graph temporal (un grafo por usuario), grafo navegable + repo de referencia D3 | Parcial: dashboard para ver; edición vía API | 8/10 |
| **LangGraph/LangSmith** | Studio: grafo de ejecución (nodos/edges), estado en cada nodo, time-travel por checkpoints; LangSmith: traces con runs anidados, timeline, dashboards | Alta: traces capturan prompts, tool calls, state changes, scores; "LangSmith agent memory" da visibilidad de access/performance/quality | Sí (grafo de ejecución del agente, no KG de datos); Store cross-thread inspeccionable | Sí: editar estado entre pasos, re-run desde checkpoint, TTL en langgraph.json | 8/10 |
| **OpenAI ChatGPT Memory** | "Memory summary" auto-generado (con "última actualización hace 2h") + "Saved memories" (lista editable) | **Fuente de referencia**: icono de libro bajo la respuesta → memoria usada → explicación de *por qué* → corregir desde el menú ••• | No | Sí: borrar individual, borrar todo, on/off, refrescar resumen; modelo auto-actualiza | 7/10 |
| **Gemini Memory** | No hay API de memoria; gestión a nivel UI (settings) sin panel dedicado documentado | No | No | Sí (a nivel de settings) | 3/10 |
| **Microsoft GraphRAG** | Sin UI oficial; **Visualization Guide** para Gephi (OpenORD + ForceAtlas2); comunidad: GraphRAG Workbench (3D WebGL force-directed, comunidades por color, centralidad, filters, chat) | Parcial: comunidad + reports; modos local/global; no expone scores | Sí: KG + comunidades jerárquicas (Leiden), community reports | No (index/reindex); Workbench: archivos, snapshots/archivos de grafo | 7/10 |
| **Neo4j Bloom** | "Perspectives" (categorías, estilos, saved searches), force-based layout, expand, reveal relationships, shortest path, scene actions, export | N/A (es un visor de DB, no de memoria) | Sí: gold standard de interacción con grafos | Sí (a nivel de datos vía Cypher/scene actions) | 8/10 |
| **LlamaIndex KnowledgeGraphIndex** | Sin UI; `index.get_networkx_graph()` → pyvis `Network` → HTML interactivo | No | Sí (NetworkX/pyvis) | No | 3/10 |
| **LightRAG (WebUI/Server)** | WebUI con tabs Documents/Knowledge Graph/Retrieval; Knowledge Graph Viewer (Sigma.js/Graphology), gravity layouts, node query, subgraph filtering | Parcial: 5 modos de retrieval (naive/local/global/hybrid/mix); no scores | Sí: visor de KG WebGL con búsqueda y filtros | Sí: CRUD de entidades y relaciones (create/edit/delete), delete por doc | 7/10 |
| **Cognee** | UI con página "Mindmap": grafo nodo-enlace interactivo por brain, status pill (Pending/Processing/Ready/Failed/Empty), búsqueda con paths resaltados; `visualize_graph()` HTML | Parcial: auto-routing recall; highlights de rutas de relación | Sí: KG + vector, temporal_cognify | Sí: remember/recall/forget/improve; memify consolida y borra nodos obsoletos | 7/10 |
| **Obsidian (graph view) + InfraNodus** | Graph view nativo (notas→nodos, backlinks→aristas); InfraNodus añade métricas de network science, community detection, gap analysis, insights IA | N/A | Sí: fuerza + clustering + analytics | Sí (son tus notas) | 7/10 |
| **Khoj** | Solo búsqueda semántica (web/Obsidian/Emacs), sin vista de memoria | No | No | No | 4/10 |
| **Rewind / Limitless** | Timeline "scrollable" de tu pantalla, búsqueda por palabra/clave/tiempo, clips + transcripciones + resúmenes de reunión | Parcial: muestra el clip fuente | No | Borrado/anonimización; app sunseat tras adquisición (dec 2025) | 6/10 |
| **Reflect** | Dashboard de memorias: título + tags + provenance + búsqueda; "recent memory" con timestamps; trash | Sí: cada memoria citable con título/tags; agente muestra "Recalled <memoria>" | No | Sí: editar en dashboard, trash, export/portable | 6/10 |
| **Cognee-adjacent: mem0-dashboard (comunidad)** | Memory Browser, Agent Tabs, Activity Feed, Query Explorer (NL), Qdrant telemetry, charts, auto-refresh | Parcial: NL query, no scores | No | No (read-only) | 5/10 |

---

## Análisis detallado por sistema

### 1. Mem0 — memoria de agentes con dashboard (Platform + self-hosted)

**Dónde:** https://mem0.ai , https://app.mem0.ai , https://docs.mem0.ai , OSS en https://github.com/mem0ai/mem0

**Dashboard Platform** (vía docs y blogs):
- **Memory Dashboard**: "Web UI for viewing, editing, and managing stored memories with search and filtering capabilities". Uso explícito: "Auditing what personal information agents have stored and managing GDPR deletion requests" (https://aitoolsatlas.ai/tools/mem0-platform/tutorial).
- **Memory Analytics**: "Insights into memory usage patterns, retention types, and the impact of memory on conversation quality" — usarlo para optimizar extracción.
- **Memory Decay** (toggle por proyecto en Settings → Instructions): cada memoria guarda "cuándo fue recuperada por última vez"; en search ese historial escala el score de relevancia (mínimo 0.3×, nunca se borra ni se hunde del todo). Es **recency sin TTL duro** (https://mem0.ai/blog/introducing-memory-decay-in-mem0).
- **Expiración**: `expiration_date` en add; las expiradas quedan ocultas de `search`/`get_all` salvo `show_expired` (https://docs.mem0.ai/core-concepts/memory-operations/add).

**Dashboard self-hosted** (DeepWiki del repo, https://deepwiki.com/mem0ai/mem0/12.3-server-dashboard):
- Next.js (port 3000) + FastAPI (8888). Módulos: **Request Logs** (audit log en vivo: method, path, status, latency; retención configurable REQUEST_LOG_RETENTION_DAYS), **Memory Browser** (search/get_all con `top_k`, `threshold` y `explain`), **Entity Management** (lista user_id/agent_id/run_id con **cascade-deletion** de memorias asociadas), **API Key Management**, **Configuration UI** (hot-reload de LLM/embedder sin reiniciar).

**Explicabilidad**: OSS `search(..., explain=True)` desglosa el score (similitud semántica + señales keyword/entidad) para "tuning retrieval quality o debugging por qué una memoria quedó donde quedó" (https://docs.mem0.ai/core-concepts/memory-operations/search). Platform fusiona señales keyword+entity+temporal.

**Graph Memory (Platform)**: las entidades extraídas (personas, lugares, organizaciones, conceptos) se vuelven nodos del grafo; `search` con `enable_graph=True` devuelve además un array `relations` ("Alex → learned → matplotlib") (https://www.datacamp.com/tutorial/mem0-tutorial). No hay visor 2D del grafo documentado en el dashboard.

**Lo mejor**: el combo explicación + gestión de ciclo de vida (decay, expiración, cascade delete, audit log) sobre CRUD simple. **Lo peor**: OSS "Inspect via CLI, logs, or custom UI" — sin dashboard fuera del server pago; la comunidad tuvo que hacer `mem0-dashboard` (read-only sobre Qdrant: Memory Browser, Agent Tabs, Activity Feed, Query Explorer, telemetría Qdrant, auto-refresh; https://github.com/dominiquedutra/mem0-dashboard) precisamente porque "self-hosted mem0 has no built-in way to see what your AI agents actually remember".

---

### 2. Letta (antes MemGPT) — UI de agente y memoria "white-box"

**Dónde:** https://www.letta.com , https://docs.letta.com , ADE legacy https://docs.letta.com/v1-sdk/ade , desktop https://docs.letta.com/platform/desktop-app

**Concepto**: memoria como **bloques etiquetados** (core memory blocks) que el propio modelo edita vía tool calls + **archival memory** fuera de contexto y buscable (hierarchy: main context / recall / archival). Contexto efectivamente ilimitado gestionado por el agente (MemGPT OS paper; https://sureprompts.com/blog/letta-memgpt-walkthrough).

**ADE (Agent Development Environment)**: tres paneles — Simulator (chat/sistema), Config (LLM, system instructions, tools, data sources), **Agent State Visualization**: *Context Window Viewer* (exactamente lo que procesa el agente), *Core Memory Blocks* (ver y **editar** la memoria persistente), *Archival Memory* (monitorear y buscar el store fuera de contexto).

**Desktop app / nuevo ADE (chat.letta.com)**: página **Memory** con **graph view** que muestra cómo los archivos de memoria se referencian entre sí; seleccionar un nodo permite ver, editar y guardar su contenido (https://docs.letta.com/platform/desktop-app). La memoria ahora son archivos versionados con **MemFS**, "git-tracked" (https://www.letta.com/agent).

**"Memory palace"**: UI para inspección y edición manual del 100% de la memoria (https://headofagents.ai/letta). **White-box memory**: "los prompts y memorias exactos que se pasan al LLM en cada paso de razonamiento son transparentes al desarrollador" (https://www.felicis.com/blog/letta).

**Lo mejor**: transparencia total + edición manual directa + portabilidad de memoria entre providers + ADE conectable a servers self-hosted vía REST (sin subir datos). **Lo peor**: complejidad de runtime (Postgres + server), el ADE legacy ya está deprecated, y el gráfo no es un KG semántico sino de referencias entre archivos.

---

### 3. Zep / Graphiti — memoria de largo plazo con grafo temporal

**Dónde:** https://www.getzep.com , https://help.getzep.com , Graphiti https://github.com/getzep/graphiti , paper https://arxiv.org/abs/2501.13956 , blog https://blog.getzep.com

**Modelo de datos**: **temporal knowledge graph** — los *hechos* son aristas entre entidades (nodos) con metadatos temporales `valid_at`, `invalid_at`, `expired_at` (soft delete) y `episodes` (referencias a los episodios que lo crearon/modificaron). Estados: Active / Historical / Expired (https://deepwiki.com/getzep/zep/2.1-temporal-knowledge-graph). Esto es exactamente el modelo "arista dirigida con peso + acumuladores" de VantaDB elevado a producto.

**Provenance / por qué**: cada hecho mantiene enlaces a los episodios origen (chat turn, documento, API). "Audit trails: *Why does the system think X is true?* → *Because of Episode Y*" — la base de una vista de explicabilidad. El blog "How Zep tracks provenance in agent memory" (https://blog.getzep.com/how-zep-tracks-provenance-in-agent-memory/) explica que sin lineage un hecho derivado "matches no source word-for-word" y nada lo conecta a su origen; Zep graba esa genealogía para debugging, retrieval source-scoped y compliance.

**Dashboard / UI**: según el README de Graphiti, Zep ofrece "Dashboard with graph visualization, debug logs, API logs". El blog "Evaluation and Control: Evaluation Framework, zepctl CLI, and Dashboard Overhaul" (https://blog.getzep.com/evaluation-and-control-evaluation-framework-zepctl-cli-and-dashboard-overhaul/) describe el dashboard rediseñado con analytics. "Observations: Patterns and Insights from the Context Graph" (https://blog.getzep.com/observations-patterns-and-insights-from-the-context-graph/). Batch API "with a progress dashboard". Además publican **zep-graph-visualization**: repo de referencia Next.js 15 + React 19 + **D3.js** para explorar el grafo que Zep aprende de interacciones (https://deepwiki.com/getzep/zep-graph-visualization).

**Retrieval**: híbrido entity search (vector) + fact retrieval (traversal) + **temporal filtering** + **reranking** por relevance/recency/provenance quality. Resultado en forma: `{entities[], facts[], episodes[]}` — pensado para poblar el contexto y, de paso, para una UI de explicación ("estos 3 hechos vienen de estos 2 episodios").

**Lo mejor**: temporalidad + provenance como ciudadanos de primera clase, retrieval sub-200ms, SOTA en benchmarks de memoria (LoCoMo/LongMemEval). **Lo peor**: la visualización de grafo no es parte central del producto comercial (repo de referencia), la edición manual es vía API, y hay poca documentación pública del dashboard.

---

### 4. LangGraph / LangSmith — state, memory y evals

**Dónde:** https://docs.langchain.com/oss/python/langgraph , https://smith.langchain.com , Studio https://docs.langchain.com/oss/python/langgraph/studio , memory page https://info.langchain.com/agent-memory

**Studio** (browser, v2 2026): renderiza el grafo de ejecución (nodos = cajas, edges = flechas, edges condicionales con su lógica). Durante la ejecución: **nodo activo resaltado, nodos completados marcados, camino recorrido trazado** → "cuál rama se tomó y cuáles nodos se saltaron". Inspección del estado en cada nodo, **time-travel debugging** (rewind a un checkpoint_id, editar estado, re-ejecutar) y hot-reload (https://www.autolearningagents.com/langgraph/langgraph-studio.php).

**Memoria en LangGraph**: checkpointer (short-term, por thread) + **Store** (cross-thread, long-term; namespace tipo directorios, key tipo archivo; búsqueda semántica sobre el store). TTL configurable para checkpoints y cross-thread memories en `langgraph.json`. El Store es inspeccionable programáticamente (https://langchain-ai.github.io/langgraph/how-tos/cross-thread-persistence/).

**LangSmith**: traces con runs anidados (graph → node → LLM/tool), timeline visual, metadata/tags por request, dashboards (latencia, errores, tokens), evals sobre dataset, y una página dedicada **"AI Agent Memory Management"** que promete "full visibility into memory access, performance, and quality" (https://info.langchain.com/agent-memory).

**Lo mejor**: traceabilidad end-to-end (qué memoria se leyó, cuándo, con qué score) + time-travel para reproducir "por qué recuperó X". **Lo peor**: no tiene vista de KG de datos (el grafo es de ejecución), la edición de memoria a mano es rudimentaria, y requiere el stack LangChain/LangSmith.

---

### 5. LlamaIndex / OpenAI Assistants / Gemini Memory — UI de memoria

**LlamaIndex**: sin UI de memoria. El patrón de visualización de su KnowledgeGraphIndex es programático: `index.get_networkx_graph()` → `pyvis.network.Network.from_nx(g)` → HTML interactivo (https://llamaindexxx.readthedocs.io/en/latest/examples/index_structs/knowledge_graph/KnowledgeGraphDemo.html). LlamaIndex además integra Zep como vector store (https://llamaindexxx.readthedocs.io/en/latest/examples/vector_stores/ZepIndexDemo.html).

**OpenAI Assistants API**: **no expone memoria** — "The API currently does not offer a memory function" (community.openai.com, 2024). Los desarrolladores deben construir su propia capa (function calling + tool createMemory). Es la razón de existir de todo el ecosistema Mem0/Zep/LangGraph.

**ChatGPT Memory** (el reference de UX para humanos):
- **Memory summary** auto-generado y auto-actualizado (muestra "última actualización hace 2 horas"), accesible en Settings → Personalization → Memory. No muestra todo lo que el modelo sabe — la síntesis es más amplia.
- **Sources**: icono de libro bajo cada respuesta → lista de fuentes usadas (custom instructions, chats pasados, archivos, **memories**) → tocar una memoria abre una **explicación de por qué se usó para personalizar** esa respuesta → menú ••• para **corregir/editar**. Es el mejor ejemplo de "explicabilidad de retrieval en producción".
- **Saved memories** (legacy): lista editable; borrar individual / borrar todo / on-off / Temporary Chat sin memoria. El modelo **auto-gestiona** (actualiza, combina, elimina). Los logs de borrado se retienen ~30 días por seguridad.
- Lección del rediseño: el sistema de saved memories previo "became stale and relied on users to manually manage updates", y las memorias podían contradecirse ("training for a marathon" vs "sprained my ankle") — por eso migraron a síntesis continua + auto-consolidación (https://help.openai.com/articles/8590148-memory-faq, https://openai.com/index/memory-and-new-controls-for-chatgpt).

**Gemini**: sin API de memoria ("Google AI Studio: Memory is managed at the UI level, not programmatically" — https://aimemory.pro/blog/gemini-memory-for-developers). Gestión a nivel de settings del app; sin panel dedicado con documentación pública.

---

### 6. Microsoft GraphRAG — visualización de comunidades

**Dónde:** https://microsoft.github.io/graphrag , Visualization Guide https://microsoft.github.io/graphrag/visualization_guide/ , blog https://www.microsoft.com/en-us/research/project/graphrag/

**Official (sin UI)**: tras indexar (con `graphml snapshots` habilitado en settings.yaml), el **Visualization Guide** lleva el grafo a **Gephi**: layout **OpenORD** (Liquid y Expansion en 50, resto en 0) seguido de **ForceAtlas2** (Scaling 15, **Dissuade Hubs** ON, LinLog OFF, **Prevent Overlap** ON) y labels de texto. Comunidades jerárquicas vía **Leiden** y **community reports** (resúmenes IA por comunidad). Global search hace map-reduce sobre reports de comunidades (detección dinámica de comunidades relevantes descartando subcomunidades irrelevantes — https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection).

**Comunidad (los UIs reales)**:
- **GraphRAG Workbench** (707★, Next.js + React Three Fiber + shadcn): **3D WebGL force-directed**, **comunidades con color-coded boundaries**, **node sizing por centralidad**, filtros por entity type / community level / relationship weight, search + highlight (Cmd+K), **Community Isolator** (aísla una comunidad + sus hijos jerárquicos), Inspector panel (detalle de entidad y sus relaciones), chat con modos local/global/drift/basic, PDF drag-drop, archivos/snapshots del grafo, logs de indexación en tiempo real (https://github.com/ChristopherLyon/graphrag-workbench).
- **GraphRAG-Local-UI** (800★, Gradio): 2D/3D Plotly, file management, settings UI, exploración de outputs/artefactos, logging (https://github.com/severian42/GraphRAG-Local-UI).

**Lo mejor**: el pipeline de comunidades (jerárquicas + reports IA) es el enfoque más escalable para grafos grandes; los workbenches de comunidad demuestran que el drill-down por comunidad es imprescindible. **Lo peor**: GraphRAG oficial no trae UI ni explicabilidad de scores; los UIs son de terceros y frágiles.

---

### 7. Neo4j (Bloom / NVL) — el gold standard de interacción con grafos

**Dónde:** https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/bloom-overview/ , blog https://neo4j.com/blog/developer/scoobygraph-3 , NVL https://neo4j.com/docs/nvl/current/ , python https://github.com/neo4j/python-graph-visualization

**Bloom**: "perspectives" que guardan categorías (nodos), relaciones, estilos por regla, **saved searches** y **scene actions**. Interacción: **force-based layout** (physics; los usuarios piden que siga corriendo para optimizar — community.neo4j.com/t/61920), clic para mover nodos, right-click → **Expand** (agregar vecinos de un tipo de relación), **Reveal relationships** (relaciones directas entre selección), **Shortest Path** entre dos nodos, contexto con leyenda de categorías, export PNG/CSV/share. Es el estándar de referencia para *exploración incremental* (partir de poco y expandir) en vez de renderizar todo.

**NVL / neo4j-viz**: biblioteca de visualización (sizing, colores, captions, pinning, tooltips, zoom/pan, layouts) usable en Jupyter/Streamlit — el patrón "biblioteca embebible" que un producto embebido puede copiar (https://github.com/neo4j/python-graph-visualization).

**Lo mejor**: interacción incremental (expand/shortest path), estilos persistentes, rendimiento WebGL. **Lo peor**: curva de aprendizaje, es un visor de DB (no entiende "memoria" ni scores), y sin clustering/community views de serie para grafos grandes.

---

### 8. Obsidian / Khoj / Rewind / Reflect — memoria personal

**Obsidian** (https://obsidian.md): graph view nativo — notas = nodos, `[[wikilinks]]`/backlinks = aristas, force-directed, filtros por tags/carpetas. **InfraNodus plugin** (https://infranodus.com/use-case/visualize-knowledge-graphs-pkm): añade network science — **community detection**, **betweenness centrality**, **diversity score**, **structural gap analysis** (clusters desconectados = "blind spots"), y **generación IA de preguntas/ideas para puentear esos gaps**; filtros por tags/resultados de búsqueda; nodo → navega a la nota. Otros plugins: Knowledge Brain (DAG de pensamientos + chat IA), Neural Composer (LightRAG local + vista 2D/3D del grafo), Related Notes (similaridad semántica en vivo) (https://www.obsidianstats.com/tags/knowledge-graph).

**Khoj** (https://khoj.ai , https://docs.khoj.dev): búsqueda semántica sobre tu second brain (web, Obsidian, Emacs, CLI), agentes con herramientas, **sin vista de grafo ni panel de memoria** (https://docs.khoj.dev/features/search/).

**Rewind / Limitless** (https://www.rewind.ai): grabación local de pantalla+audio, búsqueda por palabra/clave/tiempo, **timeline desplazable visualmente** ("scroll back through your day visually"), clips + transcripciones + resúmenes de reunión + "Ask Rewind". La UX de memoria aquí es **time-based** (línea de tiempo de tu actividad) más que cards/hechos. Rebrandeado a Limitless (abr 2024) y adquirido (dic 2025); la app original está sunseating (https://aigearbase.com/tool/rewind-ai).

**Reflect** (https://reflectmemory.com): dashboard de memorias con **título + tags + provenance + búsqueda**, lista "Recent Memory" con timestamps, trash, edición en dashboard, **portable entre Claude/ChatGPT/Cursor/Grok** vía connector MCP. Cada memoria es "structured for retrieval" (no un dump de logs). En el chat, el agente cita "Recalled: <memoria>". Patrón: **memoria como archivos/documents editables con metadatos**, dashboard para humanos + MCP para agentes.

---

### 9. Memory browsers open source / adjuntos

- **hermes-memory-ui** (46★, plugin de dashboard para Hermes Agent): pestaña "Memory" con vistas para **MEMORY.md/USER.md**, session search con filtros por fuente (CLI/Telegram/Discord/web/API), y **vistas por proveedor** (Mem0, Honcho, Mnemosyne, ByteRover, Hindsight): cards de peers, representaciones, conclusions, holographic facts con `min_trust` slider, tags, categorías; snapshot combinado por endpoint (https://github.com/xraysight/hermes-memory-ui). Buen ejemplo de **UI multi-proveedor read-only sobre la memoria de un agente**.
- **mem0-dashboard** (ver §1): dashboard read-only sobre Qdrant con agent tabs, activity feed, query explorer NL, telemetría.
- **mem0-analytics** (PyPI): traza cada interacción de memoria (latencia, eficiencia), dashboard in-terminal o PostHog (https://pypi.org/project/mem0-analytics/).
- **LightRAG WebUI** (ver §7 del análisis; Sigma.js/Graphology WebGL, MiniSearch local, Zustand; graph CRUD; https://deepwiki.com/HKUDS/LightRAG/7.2-knowledge-graph-viewer).
- **zep-graph-visualization** (D3, referencia, ver §3).
- **Cognee UI** (ver §10).

---

### 10. Cognee / otros memory frameworks con UI

**Cognee** (https://github.com/topoteretes/cognee , https://docs.cognee.ai): API `remember/recall/forget/improve` con arquitectura three-store (relacional + vector + grafo). **UI**: página **"Mindmap"** (knowledge graph view) — visor de nodos/aristas interactivo embebido desde `/v1/visualize`, selector de **brain**, **status pill** de procesamiento (Pending/Processing/Ready/Failed/Empty), drag/zoom/hover, Refresh; "Large graphs can take a while — waits up to 90s". `cognee.visualize_graph()` genera HTML interactivo. El UI también muestra **search results con highlighted relationship paths** y la UI "exposes search results with highlighted relationship paths, making debugging and knowledge exploration trivial" (https://docs.cognee.ai/cognee-cloud/ui/knowledge-graph , https://addrom.com/cognee-.../ ). `memify()` consolida: **elimina nodos obsoletos** y **fortalece conexiones frecuentes** — el patrón de consolidación/olvido (https://www.cognee.ai/blog/tutorials/beyond-recall-building-persistent-memory-in-ai-agents-with-cognee).

**Graphiti** (Zep OSS, 29.8k★): conflict detection + fact invalidation + temporal reasoning; sin UI (se espera visualizar vía Neo4j) (https://github.com/getzep/graphiti).

---

## Patrones de memory observability

("Ver qué recuerda el agente y depurar por qué no recuerda / por qué recupera lo que recupera")

1. **Span de memoria en el trace (read + write)**. OTel GenAI define `memory.retrieve` y `memory.store` como span kinds. En READ: registrar la query del agente, los candidatos devueltos y **los relevance scores**. En WRITE: qué se guardó, dónde (namespace/scope) y qué lo disparó. Fallos típicos que esto revela: *stale reads* (memoria correcta pero desactualizada) y *wrong-entity retrieval* (memoria de otro user/sesión) (https://www.braintrust.dev/articles/agent-observability-tracing-tool-calls-memory; https://laminar.sh/article/agent-observability; https://huggingface.co/blog/royswastik/evaluating-agentic-ai-systems-part-3-observability).

2. **Explain flag en search** (Mem0 `explain=True`): desglosar el score final en sus componentes (semántico, keyword/BM25, entidad/grafo, temporal/recency) para que el desarrollador entienda *qué señal ganó* (https://docs.mem0.ai/core-concepts/memory-operations/search).

3. **Provenance por episodio** (Zep/Graphiti): cada hecho devuelto trae los episodios que lo crearon/modificaron. "¿Por qué piensa el sistema que X es verdad? → porque el episodio Y" (https://deepwiki.com/getzep/zep/2.1-temporal-knowledge-graph; https://blog.getzep.com/how-zep-tracks-provenance-in-agent-memory/). En mem0 la fuente es el chat turn; en VantaDB el audit log ya es ese mecanismo pero no está expuesto como UI.

4. **Sources por respuesta** (ChatGPT): bajo cada respuesta, lista de qué memorias/custom-instructions/archivos se usaron, con un clic a "por qué" y opción de corregir. Es la explicabilidad *orientada a UX del usuario final* (https://help.openai.com/articles/8590148-memory-faq).

5. **Traces del pipeline completo** (LangSmith): run → nodo → LLM/tool, con metadata (user_id, session_id), timestamps, latencia, tokens, estado antes/después; búsqueda/filtros sobre traces (https://machinelearningplus.com/gen-ai/langgraph-observability-debugging-langsmith-tracing). Complementa a los logs: monitoring ve síntomas, observability expone causas.

6. **Métricas de salud de memoria** (mem0-analytics, Zep dashboard, LangSmith): latencia de retrieval, tasa de hits/fallos por query, crecimiento de memoria por user, freshness (stale count), y evals de recall/usage (LangSmith memory evals: "does the agent recall stored info, and does it correctly use it" — https://focused.io/lab/persistent-agent-memory-in-langgraph).

7. **Frescura / staleness explícita**: los embeddings no codifican recencia; hay que exponerlo (metadata de freshness, memory decay, TTL) para que el usuario vea *qué hay de viejo* (https://redis.io/blog/ai-agent-memory-vs-retrieval/).

## Patrones de visualización de grafos de conocimiento

1. **Force-directed layout como base** (d3-force / ForceAtlas2 / physics engines), con dos ajustes repetidos: **Dissuade Hubs** y **Prevent Overlap** (GraphRAG guide) — el layout solo ya no basta; se combina con clustering.

2. **Clustering por color de comunidades** (Leiden/community detection) como forma de reducir el "hairball": GraphRAG Workbench (color-coded boundaries), InfraNodus (community detection), LightRAG. El color por *tipo de entidad* (person/org/concept/method) es la alternativa barata sin algoritmos (LightRAG explorer).

3. **Nodo codificado por importancia**: tamaño ∝ degree/centrality (GraphRAG Workbench, LightRAG explorer), grosor de arista ∝ peso de relación/edge weight. Da jerarquía visual inmediata.

4. **Búsqueda + highlight** como puerta de entrada (Cmd/Ctrl+K en Workbench, `/` en LightRAG explorer, search real-time): buscar antes que navegar.

5. **Drill-down incremental en vez de render total**: Neo4j Bloom *expand* / *reveal relationships* / *shortest path*; GraphRAG *Community Isolator* (aísla comunidad + hijos jerárquicos); filtros por tipo de entidad, nivel de comunidad y peso. Empezar pequeño y expandir.

6. **Panel de detalle lateral (inspector)**: al hacer clic en un nodo, mostrar descripción, conexiones y metadata — no intentar meter todo en el lienzo (GraphRAG Workbench Inspector, LightRAG detail panel, Cognee).

7. **Jerarquía de comunidades + reports IA** (GraphRAG): en lugar de 100k nodos, mostrar niveles (Sector → System → Subsystem → Component → Element) con resúmenes por comunidad y drill-down. El único enfoque que escala a corpus grandes.

8. **Temporalidad visual**: en grafos temporales (Graphiti), la dimensión tiempo se muestra como *timeline del estado de un hecho* ("prefers coffee" valid_at → invalid_at, luego "prefers tea") o filtraje "¿qué sabíamos en T?" — la UI de grafos necesita un control de tiempo si el grafo es temporal (https://deepwiki.com/getzep/zep/2.1-temporal-knowledge-graph).

9. **Bibliotecas de referencia**: D3.js/force-graph (react-force-graph), Sigma.js+Graphology (WebGL, usado por LightRAG), PyVis/NetworkX (Python rápido), Gephi (offline), NVL (neo4j-viz embebible). El stack "react-force-graph + clustering + inspector" es replicable en ~1 archivo.

## Anti-patrones

- **La bola de pelos**: renderizar miles de nodos sin clustering/filtros/búsqueda. Todo grafo sin drill-down muere en el hairball (el motivo por el que Neo4j Bloom expande incrementalmente).
- **Memoria como caja negra sin UI** (Mem0 OSS): "no built-in way to see what your agents actually remember" — los equipos terminan hackeando dashboards contra el vector store, o debuggean a ciegas (hermes-memory-ui y mem0-dashboard nacieron exactamente de esto).
- **Scores sin explicación**: mostrar un score crudo (0.73) sin decir qué señal lo produjo ni qué fuente lo respalda. La explicación necesita *componentes* y *fuentes*, no un número.
- **Recuperación con stale facts silenciosos**: los embeddings no codifican recencia; sin freshness/TTL visible el agente razona perfecto sobre contexto roto (caso documentado: agente cerró 40 tickets con respuestas malas porque el índice no se refrescó — https://redis.io/blog/ai-agent-memory-vs-retrieval/).
- **Gestión 100% manual del usuario** (lección ChatGPT): las saved memories manuales "became stale", se contradecían y el usuario era el mantenedor. La consolidación/auto-actualización debe ser del sistema, con el humano solo corrigiendo (https://help.openai.com/articles/8590148-memory-faq).
- **Memoria no aislada / leakage**: sin scoping por user/agent, la UI y el retrieval mezclan contextos (wrong-entity retrieval, memory leakage — Braintrust).
- **Grafos decorativos sin significado**: visualizar el grafo sin tie a retrieval (qué subgrafo alimentó esta respuesta) convierte la vista en wallpaper. La vista debe poder responder "¿por qué esto?".
- **UI de solo lectura sin corrección**: un dashboard que muestra pero no deja borrar/corregir/esconder frustra al dev que detecta una memoria falsa (los buenos: ChatGPT •••, Mem0 dashboard, Reflect trash).

## Lecciones aplicables a VantaDB (lista concreta)

1. **Vista "Memories" por namespace**: tabla/cards de registros con key, payload, metadata expandible, score/labels (si vector hay), `created_at/updated_at/version`, estado TTL (activo/expirando/expirado) y nodo_id → enlace al grafo. Búsqueda híbrida en vivo (BM25+HNSW+RRF) con filtros por namespace/metadata — equivalente al Memory Browser de Mem0.

2. **`explain` en el motor de búsqueda ya**: VantaDB tiene BM25, HNSW y RRF nativamente → expone el desglose del score (score BM25, score HNSW, contribución RRF) por resultado. Es la feature de diferenciación más barata de implementar y la más pedida en el mercado (Mem0 `explain=True`).

3. **Ruta de observabilidad de retrieval**: cada consulta → registros devueltos + scores + (si aplica) ruta de grafo usada. Reutilizar el **audit log JSONL** existente para alimentar una vista "recent retrievals" con query, hits y por qué. = patrón de span `memory.retrieve`.

4. **Inspector del grafo (IQL)**: visor force-directed con (a) color por tipo de nodo, (b) tamaño por grado/peso acumulado, (c) **expand incremental** (empezar en 1-2 nodos y expandir vecinos), (d) búsqueda por nombre/key, (e) panel de detalle con payload + aristas entrantes/salientes + pesos, (f) ejecutar IQL y resaltar el subgrafo resultado. Stack: react-force-graph o Sigma.js (WebGL) — uno solo.

5. **Vista de ciclo de vida**: mostrar qué expirará (TTL), qué se escribió/borró/modificó recientemente (audit log), y stats por namespace (count, tamaño, crecimiento). Mem0 decay + Cognee memify sugieren ofrecer *consolidación asistida* (marcar duplicados/superados) más que solo TTL duro.

6. **Edición manual first-class**: crear/editar payload y metadata, borrar con confirmación, "esconder" (soft delete / expires) sin destruir, y export/import de un namespace (VantaDB ya tiene import/export — envolverlo en una UI).

7. **Explain "por qué" con fuentes**: al hacer search, cada resultado enlaza a su historial del audit log (quién lo escribió, cuándo, con qué trigger). Es la provenance de Zep sin infraestructura nueva.

8. **No construir el grafo sin navegación**: si se muestra el grafo, siempre con búsqueda, filtros por tipo, y drill-down; nunca el render completo de un namespace grande.

9. **Portabilidad / apertura**: exportar una vista como JSON/Markdown/CSV y URLs deep-link (namespace + key + query) para pegar en PRs/issues — barato y muy valorado en el ecosistema embebido.

10. **Empezar por lo más barato y de mayor ROI**: (1) tabla de registros con search + explain de scores, (2) vista recent-retrievals del audit log, (3) inspector de grafo force-directed con expand. Los tres reutilizan el modelo de datos existente sin cambios de schema.

---

## Referencias

**Mem0**
- https://mem0.ai/ · https://app.mem0.ai/
- https://docs.mem0.ai/core-concepts/memory-operations/add
- https://docs.mem0.ai/core-concepts/memory-operations/search
- https://docs.mem0.ai/core-concepts/how-it-works
- https://docs.mem0.ai/open-source/overview
- https://github.com/mem0ai/mem0 (dashboard: https://github.com/mem0ai/mem0/tree/main/server/dashboard)
- https://deepwiki.com/mem0ai/mem0/12.3-server-dashboard
- https://mem0.ai/blog/introducing-memory-decay-in-mem0
- https://aitoolsatlas.ai/tools/mem0-platform/tutorial
- https://www.datacamp.com/tutorial/mem0-tutorial
- https://github.com/dominiquedutra/mem0-dashboard
- https://pypi.org/project/mem0-analytics/

**Letta / MemGPT**
- https://www.letta.com/ · https://docs.letta.com/
- https://docs.letta.com/v1-sdk/ade (ADE legacy)
- https://docs.letta.com/platform/desktop-app
- https://www.letta.com/agent (MemFS)
- https://github.com/jorson-chen/letta-llm-mem
- https://sureprompts.com/blog/letta-memgpt-walkthrough
- https://headofagents.ai/letta
- https://www.felicis.com/blog/letta

**Zep / Graphiti**
- https://www.getzep.com/ · https://help.getzep.com/
- https://github.com/getzep/graphiti
- https://arxiv.org/abs/2501.13956 (paper Zep)
- https://blog.getzep.com/ (provenance: https://blog.getzep.com/how-zep-tracks-provenance-in-agent-memory/ · dashboard: https://blog.getzep.com/evaluation-and-control-evaluation-framework-zepctl-cli-and-dashboard-overhaul/ · observations: https://blog.getzep.com/observations-patterns-and-insights-from-the-context-graph/ · visual guide: https://blog.getzep.com/a-visual-guide-to-knowledge-graphs-for-ai-agents/)
- https://deepwiki.com/getzep/zep/2.1-temporal-knowledge-graph
- https://deepwiki.com/getzep/zep-graph-visualization
- https://www.getzep.com/product/agent-memory/

**LangGraph / LangSmith**
- https://docs.langchain.com/oss/python/langgraph
- https://docs.langchain.com/oss/python/langgraph/studio
- https://docs.langchain.com/oss/python/langgraph/observability
- https://smith.langchain.com/ · https://info.langchain.com/agent-memory
- https://machinelearningplus.com/gen-ai/langgraph-observability-debugging-langsmith-tracing
- https://langchain-ai.github.io/langgraph/how-tos/cross-thread-persistence/
- https://www.autolearningagents.com/langgraph/langgraph-studio.php
- https://focused.io/lab/persistent-agent-memory-in-langgraph

**OpenAI / Gemini**
- https://help.openai.com/articles/8590148-memory-faq
- https://openai.com/index/memory-and-new-controls-for-chatgpt
- https://community.openai.com/t/memory-in-assistants-and-chat-completion-apis/1041911
- https://community.openai.com/t/how-do-i-enable-or-disable-memory-in-api/703964
- https://aimemory.pro/blog/gemini-memory-for-developers

**GraphRAG**
- https://microsoft.github.io/graphrag/
- https://microsoft.github.io/graphrag/visualization_guide/
- https://www.microsoft.com/en-us/research/project/graphrag/
- https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection
- https://github.com/ChristopherLyon/graphrag-workbench
- https://github.com/severian42/GraphRAG-Local-UI
- https://www.blog.brightcoding.dev/2026/06/30/graphrag-workbench-the-revolutionary-3d-knowledge-graph-tool

**Neo4j**
- https://neo4j.com/docs/bloom-user-guide/current/bloom-visual-tour/bloom-overview/
- https://neo4j.com/blog/developer/scoobygraph-3
- https://community.neo4j.com/t/how-to-make-force-based-layout-continue-running-on-large-graph/61920
- https://neo4j.com/docs/nvl/current/
- https://github.com/neo4j/python-graph-visualization
- https://dev.to/lyonwj/graph-data-visualization-with-graphql-react-force-graph-18pk

**LlamaIndex / LightRAG / Cognee**
- https://llamaindexxx.readthedocs.io/en/latest/examples/index_structs/knowledge_graph/KnowledgeGraphDemo.html
- https://github.com/HKUDS/LightRAG
- https://deepwiki.com/HKUDS/LightRAG/7.2-knowledge-graph-viewer
- https://github.com/ianhandy/lightrag-explorer
- https://github.com/topoteretes/cognee
- https://docs.cognee.ai/cognee-cloud/ui/knowledge-graph
- https://docs.cognee.ai/how-to-guides/cognee-ui
- https://www.cognee.ai/blog/tutorials/beyond-recall-building-persistent-memory-in-ai-agents-with-cognee

**Memoria personal / memory browsers**
- https://obsidian.md/ · https://infranodus.com/use-case/visualize-knowledge-graphs-pkm · https://www.obsidianstats.com/tags/knowledge-graph
- https://khoj.ai/ · https://docs.khoj.dev/features/search/ · https://github.com/khoj-ai/khoj
- https://www.rewind.ai/ · https://aigearbase.com/tool/rewind-ai · https://aitoolscoop.com/tool/rewind-ai/
- https://reflectmemory.com/
- https://github.com/xraysight/hermes-memory-ui
- https://github.com/getzep/zep-graph-visualization

**Observabilidad / benchmarks**
- https://www.braintrust.dev/articles/agent-observability-tracing-tool-calls-memory
- https://laminar.sh/article/agent-observability
- https://huggingface.co/blog/royswastik/evaluating-agentic-ai-systems-part-3-observability
- https://arize.com/ai-agents/agent-observability/
- https://devrev.ai/blog/ai-agent-observability
- https://zylos.ai/en/research/2026-04-04-ai-agent-observability-tracing-debugging
- https://redis.io/blog/ai-agent-memory-vs-retrieval/
- https://arxiv.org/html/2601.01280v2 (Does Memory Need Graphs? — comparativa de arquitecturas de memoria con grafos)