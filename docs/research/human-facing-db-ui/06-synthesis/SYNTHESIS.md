# Síntesis — Representación humana de VantaDB: concepto "Vanta Studio"

> Documento integrador de los 5 reportes de `docs/research/human-facing-db-ui/01..05`,
> corregido por el análisis cognitivo de `07-cognitive-psychology/RESEARCH.md` (5 debilidades
> detectadas y corregidas: overview, workspace unificado, timeline+diff, undo/papelera/palette,
> barras de scores).
> Propone **un concepto de UI original** (no una tabla más) para que una persona pueda
> **ver, crear, editar y eliminar cualquier cosa** de VantaDB sin escribir código, y que
> además explote lo que hace única a esta base: ser **memoria persistente para agentes de IA**.

---

## 1. La intuición central

Toda la investigación converge en un mismo hueco:

> **Ninguna consola existente combina payload + metadata + vector + grafo + explicabilidad en un solo lugar.**
> Cada vector DB resuelve un pedazo (Weaviate el explorer, Compass el editor JSON, Neo4j el grafo,
> Mem0 la memoria, TensorFlow Projector el espacio vectorial). Nadie los une.

VantaDB no es una base de datos tradicional. Es la **memoria de un agente de IA**.
Por eso la UI no debe parecer un "admin tool de DBA" sino un **studio de memoria**:
una herramienta que contesta las tres preguntas que un desarrollador de agentes realmente tiene:

1. **¿Qué recuerda mi agente?** → ver todos los registros (memorias), su contenido, metadatos, vida.
2. **¿Por qué recuperó esto?** → explicabilidad de cada búsqueda (desglose BM25/HNSW/RRF, scores, fuentes).
3. **¿Cómo está conectado?** → grafo de conocimiento, vecindad semántica, espacio de embeddings.

**Nombre de trabajo del concepto: "Vanta Studio"** — el espacio de trabajo donde el humano
inspecciona, corrige y poda la memoria de su agente. Una sola metáfora, coherente con el
modelo mental del desarrollador (workspace tipo IDE).

---

## 2. Principios de diseño (derivados de los reportes 01–05 + correcciones cognitivas de 07)

| # | Principio | Fuente |
|---|-----------|--------|
| P1 | **Los registros son el núcleo.** Nada de "índices" ni "colecciones" como foco: la unidad de trabajo es la *memoria* (registro). | 01, 02 |
| P2 | **Master-detail siempre.** Grid para navegar, inspector para editar. Un registro = vista de detalle con tabs. | 02, 05 |
| P3 | **Overview first** (Shneiderman). El home es un resumen de la memoria (conteos, tipos, expiración, actividad), no la tabla desnuda. El cerebro hace scoping antes que probing. | 07 |
| P4 | **Workspace unificado, no pestañas aisladas** (anti split-attention). La tabla es el centro; grafo/espacio/retrieval son *lentes contextuales* del registro o namespace activo, no destinos aparte. | 07 |
| P5 | **Nada de SQL, nada de JSON obligatorio.** Query builder visual + formularios; JSON solo como formato de lujo. Para el desarrollador, JSON sigue disponible (CodeMirror) pero nunca es el camino obligatorio. | 01, 05 |
| P6 | **Commit explícito + diff.** Nunca auto-guardar. Editar → ver diff → Guardar/Revertir. | 02, 05 |
| P7 | **Explicabilidad first-class.** Todo resultado de búsqueda muestra *por qué* llegó, codificado con barras (posición/longitud), no solo números. | 03, 04, 07 |
| P8 | **Recuperación de errores** (Norman): undo global, papelera/soft-delete con restore, confirmación en destructivos. | 07 |
| P9 | **Teclado-first + command palette (Ctrl+K).** El usuario objetivo piensa en comandos; cada operación se reduce a segundos. | 07 |
| P10 | **Local-first, sin login.** La consola se sirve desde el propio proceso embebido (y WASM/OPFS en web). | 01 |
| P11 | **Grafos con navegación, nunca render completo.** Expand incremental + filtros + drill-down. | 03 |
| P12 | **El mapa vectorial es una herramienta de mantenimiento**, no solo inspección (seleccionar → borrar/editar/exportar). | 04 |
| P13 | **Todo cross-linkable.** De un resultado de búsqueda → sus vecinos semánticos → su subgrafo → su vector → su historial+diff. | 03, 04 |
| P14 | **Visualizar el ciclo de vida**: TTL con countdown (barra/ring), versionado con diff, timeline de actividad. | 02, 03, 07 |
| P15 | **Encoding redundante** (color + ícono + texto) para tipos `VantaValue`, TTL y estados — accesible a daltonismo. | 05, 07 |

---

## 3. Layout propuesto (workspace unificado, ventana principal del desktop)

```
┌──────────┬──────────────────────────────────────────────────────────────────────┐
│ SIDEBAR  │  TOPBAR: búsqueda híbrida global + namespace activo + Ctrl+K        │
│          ├──────────────────────────────────────────────────────────────────────┤
│ ● RESUMEN│   ┌────────────────────────────────────────────────────────────────┐ │
│ ● Namespaces   │  SUPERFICIE CENTRAL (contexto = namespace o registro activo) │ │
│   vanta-core   │  · HOME: cards de overview (conteos, tipos, expiración,      │ │
│   users   (1.2k)│   actividad)                                                 │ │
│   chats   (340)│  · MEMORIAS: grid virtualizada (la superficie permanente)     │ │
│ ▸ Timeline     │  · LENTES contextuales sobre lo seleccionado:                │ │
│ ▸ Actividad    │      RETRIEVAL (por qué recuperó) · GRAFO (IQL) ·            │ │
│ ▸ Índices/salud│      ESPACIO (embeddings) · OPERACIONES (import/export)      │ │
│ ▸ IQL console  │                                                               │ │
│                  └────────────────────────────────────────────────────────────┘ │
│                 ┌─┴───────────────────────────────────────────────────────────┐ │
│  INSPECTOR      │  Detalle del objeto seleccionado (context-aware)             │ │
│  (panel derecho)│  General | Metadata | Vector | Historial+Diff | Conexiones   │ │
│                 └─────────────────────────────────────────────────────────────┘ │
└──────────┴──────────────────────────────────────────────────────────────────────┘
```

- **Sidebar**: RESUMEN (home) + namespaces con conteos (primer nivel de organización) + Timeline + Actividad + Índices + consola IQL.
- **Topbar**: búsqueda híbrida global (texto + vector-picker + slider de pesos + filtros visuales por metadata), live desde cualquier lente; indicador Ctrl+K.
- **Superficie central**: HOME (overview) como entrada; MEMORIAS como tabla permanente; los lentes (Retrieval, Grafo, Espacio, Operaciones) actúan **sobre el namespace o registro seleccionado** — cambian de contexto, no de metáfora.
- **Inspector derecho**: master-detail permanente, context-aware (registro, nodo, arista, punto del mapa).
- **Command palette (Ctrl+K)**: abrir namespace, buscar key, ejecutar IQL, exportar, borrar, undo — descubrimiento incremental de toda la app.

---

## 4. Las superficies del studio

### Superficie 0 — HOME (overview, nueva — Fix 1)
Entrada cognitiva (Shneiderman: overview → zoom/filter → details). Máximo 6-8 cards/sparklines:
- Conteo por namespace + tendencia de crecimiento de memoria.
- Distribución de tipos de metadata (mini-histograma).
- Próximos a expirar (TTL) y expirados recientes.
- Actividad reciente (últimas escrituras/borrados del audit log).
Un vistazo contesta "¿qué estado tiene la memoria de mi agente?" sin abrir nada.

### Superficie 1 — MEMORIAS (grid de registros)
El centro permanente. Tabla virtualizada de `VantaMemoryRecord`:

| Columna | Representación |
|---------|----------------|
| key | monospace |
| payload | preview 1 línea + hover/fila expandida (markdown-ish) |
| metadata | chips tipados (color + ícono por tipo `VantaValue`: string/int/float/bool/datetime/list/null) |
| vector | badge (presencia + dimensión) |
| version | chip |
| updated_at | legible + relativa |
| TTL | barra/ring de countdown (activo / expirando / expirado) + estado textual |

**Inspector tabs por registro:**
- **General**: key, namespace, node_id (u128, monospace), timestamps, version, TTL (editable, countdown).
- **Metadata**: key-value editor tipo Compass con **tipo inferido de `VantaValue`** (agregar/quitar filas, selector de tipo).
- **Vector**: colapsado + stats (norma, min/max) + mini sparkline + botones copiar JSON / pegar.
- **Historial+Diff** (Fix 3): cada versión con su cambio resaltado (payload/metadata/vector) estilo git.
- **Conexiones**: vecinos del grafo + top-k semánticos del registro.

Modelos: Weaviate Explorer + Attu browse (grid), Compass (editor), TablePlus (sidebar detail), Redis Insight (TTL).

### Lente RETRIEVAL (¿por qué recuperó esto? — ahora con encoding correcto, Fix 5)
El diferenciador más barato y más pedido del mercado (Mem0 `explain=True`, Zep provenance).

- Barra de consulta: texto (BM25) + vector (pegar o *picker de registro*) + slider de pesos híbridos + top-k + umbral + **filtros visuales por metadata** (`VantaFilterOp`: Eq/Neq/Gt/Lt/Gte/Lte) sin escribir JSON.
- Resultados con **desglose de score como barras horizontales apiladas** (BM25 + HNSW + RRF; longitud = score, color solo secundario) — el ojo compara por longitud, no por dígitos (Cleveland–McGill).
- Cada resultado → botón "ver contexto": vecinos semánticos, subgrafo IQL, historial del audit log (provenance: quién lo escribió, cuándo, con qué trigger).
- Vista "recent retrievals": alimentada del audit log JSONL existente, con query + hits + motivo.

Modelos: Qdrant console, Redis VSIM FILTER, Zilliz hybrid tabs, LangSmith traces.

### Lente GRAFO (conocimiento + IQL)
Ninguna consola de vector DB tiene esto: es oportunidad única.

- Force-directed (react-force-graph o Sigma.js WebGL), **expand incremental** desde 1-2 nodos.
- Color por tipo de nodo, tamaño por grado/acumulador; panel de detalle con payload + aristas entrantes/salientes + pesos.
- **IQL console**: playground con autocompletado (estilo Kibana Dev Tools); el resultado resalta el subgrafo.
- Vista complementaria "Matriz": heatmap + dendrograma de similitud de los top-k de un query.

Modelos: Neo4j Bloom, Memgraph Lab, Obsidian local graph, react-querybuilder.

### Lente ESPACIO (embeddings)
- Scatterplot 2D con **regl-scatterplot** (WebGL, 1M+ puntos), **UMAP-js en web worker** (client-side, colecciones de 10k–100k viables), o PaCMAP portado al core WASM a futuro.
- Color por cluster (kmeans en JS) o por namespace/tier/metadata (con leyenda + ícono, encoding redundante); hover → payload preview; click → vecinos top-k con scores.
- **El mapa como herramienta de mantenimiento**: seleccionar puntos → borrar/editar/exportar/ajustar metadata.
- El core ya produce `explain` en `search_memory`; `NodeRecord` ya trae `tier`/`importance`.

Modelos: Apple Embedding Atlas, TensorFlow Projector, regl-scatterplot demos.
**Anti-patrón de la industria (documentado en 01):** nadie muestra proyección 2D en producción → hacerla *opcional*, siempre con filtros, nunca la primera vista.

### Lente OPERACIONES (salud + vida de la memoria)
- **Métricas**: hnsw_nodes_count, dimensiones, conteos por namespace, TTL próximos a expirar, tamaño de almacenamiento (LSM keyspaces/levels, WAL), 3-4 charts simples del propio motor (no hace falta Datadog).
- **Timeline** (Fix 3): línea de tiempo unificada de escrituras, actualizaciones, expiraciones y borrados por namespace (eventos del audit log agrupados en el tiempo).
- **Activity**: audit log filtrable por namespace/op/outcome (equivalente al Profiler/SlowLog de RedisInsight).
- **Import/export**: drag & drop `.vdbdump`/JSONL (patrón Qdrant snapshots) + exportar selección/namespace + **reporte legible** (markdown/HTML del estado) para compartir.
- **Papelera** (Fix 4): soft-delete con restore de lo borrado en la sesión (tombstones).
- **Consolidación asistida**: marcar duplicados/superados (decay de Mem0, memify de Cognee) con su diff visible — más humano que solo TTL duro.

---

## 5. Lo que hace esto "totalmente nuevo" (resumen del argumento)

1. **Un solo lugar para todo el modelo de datos**: payload + metadata tipada + vector + sparse + TTL + version + grafo + audit — ninguna consola existente lo junta.
2. **Memoria, no tablas**: la metáfora es "trabajar con la memoria de tu agente", no "administrar una base".
3. **Explicabilidad como feature nativa**, no como debug: el desglose BM25/HNSW/RRF es la propuesta de valor, legible por longitud (no por dígitos).
4. **El grafo como lente ciudadana** (aristas dirigidas + acumuladores + IQL) — imposible de copiar de otra consola porque ninguna lo tiene.
5. **De la búsqueda al mapa al grafo, todo cross-linked** en un solo clic, sin cambiar de pestaña.
6. **Diseñada para el cerebro humano**: overview antes del detalle, workspace unificado (sin split-attention), timeline + diff (historia de la memoria), undo + papelera + command palette (recuperación de errores y velocidad), encoding redundante accesible.

---

## 6. Stack recomendado (para el módulo desktop Tauri v2 + React + Vite)

| Necesidad | Elección | Por qué |
|-----------|----------|---------|
| Grid virtualizado + sort/filtro | **TanStack Table v9 + TanStack Virtual** | MIT, headless, el mejor fit React; AG Grid/MUI X con features clave en plan pago |
| Editor payload/markdown/JSON | **CodeMirror 6** | ligero, embebible, highlight + lint |
| Metadata key-value | **editor propio** (tipo inferido de `VantaValue`) | sin librería que lo haga bien para schema-less |
| Query builder filtros | **react-querybuilder** | composición visual de filtros AND/OR |
| Grafo | **react-force-graph** o **Sigma.js** (uno solo) | WebGL, expand incremental |
| Scatterplot embeddings | **regl-scatterplot** | WebGL, 1M+ puntos, hover/selection |
| Proyección client-side | **UMAP-js** (web worker) | sin servidor; PaCMAP→WASM como evolución |
| Vectores preview | **sparkline propio** (svg) | sin dependencia |
| Desglose de scores (barras) | **barras horizontales apiladas propias** (div/svg) | sin librería; el canal correcto es longitud |
| Undo / command palette | **zustand + reducer propio / cmdk** (palette) | undo por snapshot del estado; palette tipo VS Code |
| Deep links / portabilidad | URLs `vanta://ns/key?query=...` + export JSON/Markdown/CSV | barato, muy valorado en ecosistema embebido |

**Descarte deliberado (justificado en 05):** Glide Data Grid (renderers en canvas, a11y débil), react-json-view (abandonado), json-editor.org (mantenimiento), D3 force directamente (react-force-graph lo envuelve).

---

## 7. Plan de implementación por fases (mapeado al código actual)

El módulo desktop hoy (`desktop/src/`) tiene una sola página con paneles apilados
(MetricsGrid, KpiCards, ExportPanel, SopPanel, ConnectionPanel, IngestForm, SearchBar,
DataExplorer, ProcessPanel, ResultsList); `DataExplorer.tsx` usa "Load more" sin cursor
(anti-patrón #2 de 05) y no hay detalle de registro ni edición.

**Fase 0 — Fundamentos (esta iteración)**
- Reestructurar `App.tsx` al **workspace unificado** (sidebar/topbar/superficie central + inspector) — no más paneles apilados ni 5 pestañas hermanas.
- **HOME/overview** (Fix 1): 6-8 cards con conteos, tipos, expiración próxima, actividad reciente.
- `DataExplorer.tsx`: reemplazar "Load more" por paginación virtualizada; columnas completas.
- **Inspector de registro** (nuevo, el P0 de 02): General/Metadata/Vector/Historial, editor JSON con commit explícito (Guardar/Revertir). Ancla en `vanta.ts` (ya expone `get`/`put`/`delete`).
- Filtros compuestos en la búsqueda (react-querybuilder).
- **Undo + papelera** (Fix 4): snapshot del estado por sesión, soft-delete con restore.
- **Command palette** (Ctrl+K) con las acciones básicas.

**Fase 1 — Explicabilidad y tiempo (diferenciadores, P0 de 03)**
- Lente RETRIEVAL: desglose de score por **barras apiladas** (Fix 5).
- **Historial+Diff** entre versiones (Fix 3).
- Vista ACTIVITY + Timeline: audit log filtrable y agrupado en el tiempo.
- Deep links + export de vistas + **reporte legible** (markdown).
- A11y: encoding redundante (color + ícono) en chips y badges.

**Fase 2 — Grafo y espacio**
- Lente GRAFO: lista de aristas → force-directed con expand incremental + IQL console.
- Lente ESPACIO: scatterplot + UMAP-js en worker; selección → batch ops.
- Batch operations con confirmación + undo (borrar/exportar selección).

**Fase 3 — Web/embebido**
- Servir la misma consola desde el proceso embebido (`:puerto/dashboard` estilo Qdrant) y desde WASM/OPFS (IndexedDB). Local-first sin login. Reutilizar componentes React.

---

## 8. Gap a resolver en el core (recomendaciones para backlog)

La investigación detecta limitaciones actuales que conviene anotar:

1. **Cursor/paginación en listado**: `vanta_list` solo acepta namespace+limit → exponer offset/cursor real para virtualización sin "Load more". (Ver `desktop/src/vanta.ts`.)
2. **`explain` estructurado en la API**: el desglose BM25/HNSW/RRF debería salir como objeto tipado en las SDKs (Rust, Python, TS), no solo trace.
3. **Contar registros por namespace / stats de TTL**: exponer contadores para la sidebar, el HOME y la vista de operaciones.
4. **Exportar selección / query** (no solo namespace completo).
5. **IQL en la web/desktop**: confirmar que `query()` (IQL) está expuesto en la API WASM (sí está en `vantadb-wasm/src/lib.rs` vía `query`) y su ergonomía para autocompletado.
6. **Batch delete con filtro** desde la UI (existe `delete_by_filter`).
7. **Versiones históricas accesibles** para el diff: confirmar si el core retiene versiones anteriores o solo el `version` actual (si solo lo actual, el diff n vs n-1 necesita retener snapshots — decisión de diseño en el core).

---

## 9. Fuentes directas (resumen de los reportes)

| Reporte | Contenido | Top-3 takeaways |
|---------|-----------|-----------------|
| `01-vector-db-consoles/RESEARCH.md` | 12 consolas vectoriales | Nadie junta payload+vector; search híbrida como feature de UI; local-first sin login |
| `02-desktop-db-tools/RESEARCH.md` | 19 herramientas desktop | Master-detail + editor JSON con commit; filtros compuestos; 11 lecciones P0–P3 |
| `03-ai-memory-graphs/RESEARCH.md` | 10 sistemas de memoria/grafos | `explain` como feature; observabilidad de retrieval; force-directed con expand incremental |
| `04-embedding-visualization/RESEARCH.md` | técnicas de reducción + stack | UMAP-js client-side; regl-scatterplot; mapa como herramienta de mantenimiento; no 2D en producción sin filtros |
| `05-data-editor-ux/RESEARCH.md` | librerías de grid/JSON/forms | TanStack Table v9 + CodeMirror 6; KV editor con tipo inferido; commit explícito; anti-patrón "Load more" |
| `07-cognitive-psychology/RESEARCH.md` | ciencia cognitiva de la visualización | Overview first; workspace unificado (anti split-attention); timeline+diff; undo+paperela+palette; barras para scores (Cleveland–McGill) |

---

## 10. Conclusión

La representación humana óptima para VantaDB **no es una tabla ni un dashboard**: es un
**workspace de memoria** ("Vanta Studio") donde el payload, la metadata tipada, el vector,
el TTL y el grafo de cada registro se ven, se editan y se borran con commit explícito + undo,
y donde **cada resultado de búsqueda explica por qué llegó** con encoding legible (barras).
El diseño sigue el orden natural del cerebro humano: **overview → detalle → contexto**,
sin conmutación de pestañas, con la historia de la memoria (timeline + diff) siempre a la
vista. Todo local-first, sin login, sirviéndose desde el propio motor.

Esto es alcanzable por fases sobre el módulo desktop actual (Tauri v2 + React), empezando por
el workspace unificado + overview + inspector de registro (P0 de 02 y de 07), seguido de la
explicabilidad y el diff (P0 de 03), y es el diferenciador que ninguna consola de vector DB
tiene hoy.