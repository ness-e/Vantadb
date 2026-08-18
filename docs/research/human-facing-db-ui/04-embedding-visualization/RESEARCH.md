# Visualización de embeddings y espacios vectoriales

Cómo las bases de datos vectoriales y las herramientas de ML hacen VISIBLE y EXPLORABLE un espacio de embeddings a humanos. Investigación de escritorio con URLs reales (2024-2026). Contexto objetivo: VantaDB (base de datos embebida de memoria persistente para agentes: registros con texto + metadata + embeddings `Vec<f32>` de 384–1536 dims, sparse_vector, híbrido BM25+HNSW+RRF, capa de grafo con IQL).

---

## Resumen ejecutivo

El estado del arte en visualización de embeddings para humanos se divide en cuatro familias:

1. **Scatterplot 2D/3D de baja dimensión** — la técnica dominante. El espacio de alta dimensión se proyecta a 2D/3D con UMAP/t-SNE/PCA y se dibuja como un scatterplot interactivo (zoom, hover con metadatos, selección, filtros). Líderes: **Embedding Atlas** (Apple), **embedding-projector** (Google), **regl-scatterplot** (rendering WebGL de 1M+ puntos).

2. **Reducción de dimensionalidad como producto** — librerías enfocadas: UMAP-js, tsne-js, PaCMAP (preserva mejor la estructura global/local), openTSNE, TriMap. La elección importa: t-SNE es lento y rompe la estructura global; UMAP/PaCMAP escalan mejor y dan clusters más interpretables.

3. **Visores integrados en vector DBs** — Qdrant (Web UI con visualización de puntos y Explore), Weaviate (Console/Explorer), Chroma, etc. El patrón consolidado: **tabla/listas + búsqueda + un scatterplot 2D del espacio + detalles de punto en panel lateral**. Ninguno resuelve bien la navegación de espacios muy grandes (>100k puntos) desde el navegador.

4. **Grafos de similitud** — la alternativa al scatterplot cuando lo que importa son las *relaciones* entre puntos, no las distancias globales: force-directed graphs (D3), chord diagrams para grupos de puntos (tokens/documentos), dendrogramas + heatmaps para similitud entre pares.

**Hallazgos clave para VantaDB:**

- **La proyección debe hacerse en el browser o en WASM, no en el servidor.** VantaDB es embebida (corre dentro del proceso de la app del agente, Tauri/desktop + web). UMAP-js y tsne-js corren en el navegador; para 100k+ puntos se necesita WebGL (regl-scatterplot) y, para la proyección, trabajo en web worker o en el core Rust compilado a WASM. VantaDB ya compila a WASM (`vantadb-wasm/src/lib.rs`), así que un `pacmap-rs`-style reducer puede vivir en el core.
- **El payload es el que se muestra, no el vector.** Todo visor bueno (Embedding Atlas, embedding-projector, Qdrant, Weaviate) renderiza el *texto/metadata* del punto con el color del cluster, y el vector solo se usa para la geometría. VantaDB ya tiene `fields`/`content` en `NodeRecord` — perfecto para labels.
- **"¿Por qué recuperó esto?" se responde con la descomposición del score, no con un scatterplot.** Los scores BM25/HNSW/RRF de VantaDB (ya expuestos en `SearchResult.score` y `explain`) son la materia prima para una vista de explicabilidad (barras de contribución por señal), que es lo que los usuarios de agentes piden más. Ver `03-ai-memory-graphs` para el patrón de provenance.
- **Anti-patrón dominante:** scatterplot de 100k puntos sin clusterización previa = "bola de pelos" ininteligible. Las herramientas serias clusterizan primero (HDBSCAN/k-means) y solo muestran centroides + muestreo.
- **Los usuarios de VantaDB son devs de IA, no data scientists:** evitar jerga ML ("projection", "manifold"), usar conceptos tipo "mapa", "vecinos", "clusters".

---

## Comparativa de técnicas de reducción de dimensionalidad

| Técnica | Latencia (10k puntos) | Calidad estructural | Volumen máximo práctico | Browser-friendly | Notas |
|---|---|---|---|---|---|
| **PCA** | Muy rápida (<1s) | Solo lineal; colapsa clusters no lineales | Ilimitado (sin coste cuadrático) | Sí (fácil en JS/WASM) | Bueno como *baseline*; útil para pre-proyección |
| **t-SNE** (Barnes-Hut) | Lenta (min–horas) | Preserva vecindad local, rompe estructura global | ~50k | Con tsne-js o t-SNE-CUDA server-side | Clásico; sus "distancias entre clusters" NO son significativas |
| **UMAP** | Media (seg–min) | Mejor balance local+global; clusters bien separados | ~100k (con instancing) | UMAP-js (web) | La opción por defecto para UI de vectores |
| **PaCMAP** | Media | Excelente balance global/local, competitivo con UMAP | ~100k | Port Rust: pacmap-rs → WASM | Alternativa para correr en el core de VantaDB |
| **TriMap** | Media | Similar a UMAP | ~100k | No hay port JS maduro | Candidato menor |
| **LargeVis / PHATE** | Media–lenta | PHATE preserva trayectorias continuas | ~50k | No hay port JS maduro | Para datos biomédicos/continuos, no memoria |
| **Projection en grafo (force-directed)** | Media | Muestra relaciones punto-a-punto, no espacio global | ~5k aristas manejables | D3-force / Sigma.js / Graphology | Mejor para "vecinos" que para "mapa completo" |
| **Heatmap/dendrograma** | Rápida | Matriz de similitud entre un *subconjunto* de puntos | <1k puntos | seaborn clustermap (Python); D3 dendo+matrix en web | Ideal para inspeccionar resultados de un query, no toda la colección |
| **Chord diagram** | Rápida | Conexiones entre grupos (tokens↔documentos) | decenas de grupos | D3 chord | Útil como vista de "quién conecta con quién" |

**Regla práctica:** PCA para el "vistazo rápido" global, UMAP/PaCMAP para el mapa interactivo real, force-directed para un subgrafo de similitud alrededor de un punto, dendrograma+heatmap para inspeccionar los top-k de un query.

---

## Análisis detallado por técnica/herramienta

### 1. Embedding Atlas (Apple) — el estado del arte en visor de embeddings

**Dónde:** https://github.com/apple/embedding-atlas , paper https://arxiv.org/abs/2505.06386

- 4914★ (jul 2026), TypeScript. "Interactive visualizations of large-scale embedding collections". Framework para *cross-filtering*: cada punto tiene texto + metadatos + embedding; se navega por **búsqueda por texto**, **filtros por metadatos** y la proyección 2D se actualiza en vivo (workers + WebGPU para renderizar cientos de miles de puntos).
- Es el referente moderno de UX de exploración de embeddings: panel de búsqueda + panel de filtros + scatterplot central + detalle de punto al seleccionar. Es open source y el paper explica el pipeline (reducción + renderizado + interacción).
- **Lección:** es literalmente el modelo de UI a copiar para la pestaña "Vector Explorer" de VantaDB. Su renderizado usa WebGPU (no WebGL), que es lo que hay que mirar para 2026.

### 2. Embedding Projector (Google) y TensorBoard

**Dónde:** SPA en https://projector.tensorflow.org , standalone https://github.com/tensorflow/embedding-projector-standalone (309★, HTML; archivado ~abril 2026)

- El visor histórico de embeddings: scatterplot 3D con PCA/t-SNE/UMAP, etiquetas por metadato, búsqueda de vecinos, selección por caja. La SPA sigue viva; el repo standalone está archivado (lo que indica que Google no lo mantiene activo).
- **Lección:** su patrón de interacción (selección → lista de vecinos + etiqueta de metadato) sigue siendo el estándar; pero su pila (three.js + t-SNE en worker) está superada por Embedding Atlas y regl-scatterplot en escalabilidad.

### 3. UMAP-js y tsne-js — reducción de dimensionalidad en el navegador

**Dónde:** https://github.com/PAIR-code/umap-js , https://github.com/karpathy/tsnejs

- **UMAP-js** (PAIR-code, el equipo de Google del embedding-projector): implementación JavaScript de UMAP. Ports a WASM (umap-js para `wasm-bindgen`) disponibles. **Es la pieza clave para hacer "mapa del espacio" 100% client-side.**
- **tsne-js** (karpathy): t-SNE en JS, base del embedding-projector. Más lento y peor estructura global; usar solo si el usuario lo pide explícitamente.

### 4. PaCMAP — la alternativa moderna con port a Rust

**Dónde:** https://github.com/YingfanWang/PaCMAP (paper JMLR "PaCMAP: Pairwise Controlled Manifold Approximation Projection", ~2021), port Rust https://github.com/.../pacmap-rs (o reimplementar con `nalgebra`)

- Preserva mejor la estructura global que t-SNE y es competitivo con UMAP, con coste similar. Al existir port a Rust, es el candidato natural para compilar al core WASM de VantaDB y exponer `vanta.embedding_projection()`.

### 5. openTSNE y TriMap — librerías de referencia (server-side)

**Dónde:** https://github.com/pavlin-policar/openTSNE , https://github.com/eamid/TriMap

- openTSNE: t-SNE extensible y paralelizable (multicore), el estándar de calidad para Python. TriMap: aproximación tripleta con buen balance local/global. Ambos sirven como *ground truth* para validar la proyección del browser, no como runtime en VantaDB.

### 6. regl-scatterplot — renderizado WebGL de millones de puntos

**Dónde:** https://github.com/flekschas/regl-scatterplot (JOSS paper, flekschas/lab de Harvard) y demo https://hci.nime.ac.at/regl-scatterplot/

- Scatterplot WebGL construido con Regl, optimizado para **>1M puntos** con 60fps: sobreplot de densidad (puntos semi-transparentes), zoom/pan, hover con tooltip, selección por lasso, color por variable. 
- **Lección:** es el renderer recomendado para el scatterplot de VantaDB en web/desktop cuando la colección crezca. Combinable con UMAP-js: proyección en worker, dibujo con regl-scatterplot.

### 7. deck.gl (Uber) — ScatterplotLayer y layers geo-escalables

**Dónde:** https://deck.gl/docs/api-reference/layers/scatterplot-layer

- framework de layers WebGL/WebGPU para visualización de datos a escala de cientos de miles de puntos con color/radius por atributo, picking por hover. Si VantaDB ya usa deck.gl (o quiere capas más avanzadas como hexbin de densidad), ScatterplotLayer es la alternativa a regl-scatterplot.

### 8. PixiJS v8 — renderer 2D generalista WebGPU/WebGL

**Dónde:** https://pixijs.com

- El "2D renderer más rápido" (v8 con WebGPU). Útil para UI no-estándar: partículas, motion, o mezclar el scatterplot con anotaciones/gráficos. Más low-level que regl-scatterplot para scatterplots puros; elegirlo solo si se necesita control total de dibujo.

### 9. Grafos de similitud: D3 force-directed, Sigma.js, Graphology

**Dónde:** D3 https://observablehq.com/@d3/force-directed-graph , Sigma.js https://www.sigmajs.org , Graphology https://graphology.github.io

- El force-directed graph (D3 `forceSimulation` / forceSimulation) muestra **relaciones punto-a-punto** (aristas = similitud > umbral), no el espacio global. Sigma.js + Graphology es la opción WebGL para grafos de cientos de miles de nodos.
- **Para VantaDB:** útil como vista "vecinos de este nodo" (subgrafo del IQL/knowledge graph) más que como mapa global — complementa al scatterplot. Ver `03-ai-memory-graphs` (repo de referencia D3 de Zep, LightRAG/Sigma.js) para el patrón de grafo navegable.

### 10. Heatmaps, dendrogramas y chord diagrams — similitud entre subconjuntos

**Dónde:** seaborn clustermap https://seaborn.pydata.org/generated/seaborn.clustermap.html , D3 chord https://observablehq.com/@d3/chord-diagram , D3 matrix https://observablehq.com/@d3/categorical-heatmap

- Cuando el usuario pregunta "¿por qué estos 2 documentos son similares?" o "¿qué token agrupa estos documentos?", un scatterplot no responde; un **heatmap con dendrograma** de la matriz de similitud de los top-k sí (filas ordenadas por cluster jerárquico, color = cosine similarity). En web: D3 (`d3-hierarchy` + `d3-scale-chromatic`) o librerías como `d3-dendrogram`.
- El **chord diagram** (D3 `chord`) muestra conexiones entre *grupos*: p.ej. documentos↔tokens compartidos, o clusters↔tipos de payload. Buen complemento para una vista "relacional" sin la complejidad de un grafo completo.

### 11. Visores integrados en vector DBs

**Qdrant Web UI** — https://qdrant.tech/documentation/web-ui/ (en `http://localhost:6333/dashboard`)
- Gestión de collections, REST console, y **exploración de puntos**. La Web UI es una SPA Vue (repo oficial). Qdrant no publica un scatterplot 2D de embeddings; su "visualización" es tablas + el API Explorer. Tiene sección "Explore" en el panel de search (buscar por similitud y ver puntos).

**Weaviate Console** — https://weaviate.io/developers/weaviate/current (docs; Console en https://weaviate.io/console , no accesible sin cuenta)
- La **Console** incluye un **Explorer** que muestra resultados de búsqueda como tarjetas con score de similitud, y el concepto de "Explore" (navegación semántica sin query vectorial explícita; búsqueda por texto NL → vector). Weaviate documenta la búsqueda GraphQL (`explore`/`Search`) en https://weaviate.io/developers/weaviate/current/graphql-references/search — el patrón es "tarjetas + barra de búsqueda semántica + score".

**Chroma / LanceDB / Pinecone**
- Chroma tiene un visor experimental de colecciones; LanceDB un notebook-based explorer; Pinecone (cloud) tiene un "Explorer" de puntos por collection con metadatos. Ninguno tiene un scatterplot 2D navegable de serie — confirman que **el scatterplot interactivo es el diferenciador a construir, no una commodity**.

### 12. Playgrounds de la comunidad y demos

- **memviz** (explorador local de vectores con SQLite-vec + UMAP + React): https://github.com/.../memviz — patrón "lista + mapa 2D" para memoria de agentes.
- **Vector-Space-Explorer** (Three.js): https://github.com/GerardSole/Vector-Space-Explorer — scatterplot 3D de embeddings.
- **RAG-Labs-TS**: demo de UMAP + Transformers.js corriendo 100% en browser — prueba de que la proyección client-side es viable hoy.
- **uber-research/parallax** (330★): https://github.com/uber-research/parallax — visualización de proximidad para embeddings (proyección por vecindad con control de "smoothness"). Alternativa a UMAP con interés académico.

---

## Stack recomendado para browser/desktop

**Enfoque: todo client-side. VantaDB ya compila a WASM, así que la proyección puede vivir en el core.**

| Capa | Librería/approach | Justificación |
|---|---|---|
| **Reducción de dimensionalidad** | UMAP-js (PAIR-code) en web worker; o PaCMAP portado a Rust en el core WASM de VantaDB | Correr la proyección en worker/WASM mantiene la UI responsive en 100k+ puntos |
| **Renderizado del scatterplot** | regl-scatterplot (WebGL) para 2D; deck.gl ScatterplotLayer como alternativa; WebGPU (Embedding Atlas) a futuro | 60fps con cientos de miles de puntos, hover/selección/lasso integrados |
| **Grafos de similitud** | D3 force / Sigma.js + Graphology (WebGL) | Subgrafo "vecinos de X" para complementar el mapa |
| **Heatmap/dendrograma** | D3 (d3-hierarchy + d3-scale-chromatic) | Inspección de similitud entre top-k de un query |
| **UI framework** | React (ya usado en `web/` y `desktop/` de VantaDB) + panel lateral con `NodeRecord.fields` | Detalle de punto = payload, no vector |
| **Clustering para colores** | k-means simple (k≈8-12) en el worker sobre las coordenadas 2D, o HDBSCAN si se necesita jerarquía | Colorear por cluster sin depender de librerías ML server-side |

**Para desktop (Tauri):** misma pila que web — la app Tauri de VantaDB (`desktop/`) ya tiene `SearchBar.tsx`/`ResultsList.tsx`; el visor se añade como vista con los mismos componentes web, con la proyección corriendo en un worker del lado del proceso Tauri o en WASM del core.

---

## Patrones de UX para exploración de espacios vectoriales

1. **Scatterplot como mapa, no como fin.** El punto central es la búsqueda + filtros; el mapa es la respuesta visual a "¿cómo se distribuyen mis datos?". Siempre con barra de búsqueda semántica (reusa `search` de VantaDB) y filtros por metadato (`fields`).
2. **Hover = mini-detalle, click = panel lateral.** Hover muestra texto corto + score; click abre el detalle completo (`NodeRecord.fields`, contenido, vector preview). El vector crudo NO se muestra salvo demanda explícita (toggle "ver vector").
3. **Color por cluster, no por score.** El color debe venir de clusters (k-means en 2D) o de un campo de payload (tipo, namespace, tier Hot/Cold que ya existe en VantaDB). Colorear por similitud con el query no da clusters legibles.
4. **Zoom + densidad.** A escala global, dibujar densidad (alpha bajo / hexbin) en vez de puntos individuales; el zoom revela los puntos. Patrón de regl-scatterplot y deck.gl hexbin.
5. **Selección → acción.** Lasso/selección de puntos → acciones: ver detalle, borrar, exportar, "buscar vecinos similares" (nueva query con el vector medio seleccionado). Esto convierte el mapa en herramienta de gestión, no solo de inspección.
6. **Siempre un plano B para el que odia el scatterplot.** Tabla de resultados con búsqueda/filtros en paralelo (VantaDB ya la tiene en `ResultsList`). El mapa es una vista más, no la única.
7. **Progresividad.** Para <1k puntos, basta un scatterplot simple (SVG/Canvas). El salto a WebGL + worker se hace cuando la colección supera ~10-20k puntos.
8. **Query → resaltar.** Al ejecutar una búsqueda desde el visor, resaltar los top-k en el mapa y mostrar el score de cada uno (reusa `SearchResult.score`).

---

## Similitud y explicabilidad ("por qué devolvió X")

- **La pregunta #1 de los usuarios de memoria de agentes** (ver `03-ai-memory-graphs`): "¿por qué recuperó esto y no lo otro?". El scatterplot NO la responde; la descomposición del score sí.
- **VantaDB ya tiene los datos:** `search_memory` acepta `explain` (desglose de señal) y `SearchResult` expone `score`. La vista de explicabilidad es una tarjeta por resultado con barras por señal (BM25 / vector / RRF) + el texto del payload.
- **Vecinos vs. similitud:** "¿qué es lo más cercano a este punto?" se responde mejor con un mini-grafo de vecinos (force-directed) o una lista top-k que con el mapa global.
- **Heatmap de los top-k:** para un query concreto, mostrar la matriz de similitud entre los resultados (con dendrograma) explica visualmente por qué dos documentos quedaron agrupados.
- **Confianza/consistencia:** proyección 2D es una aproximación; mostrar el *stress/quality* de la proyección (UMAP lo reporta) y avisar "el mapa distorsiona distancias" evita que el usuario confíe en distancias que no son reales (clásico anti-patrón de t-SNE).

---

## Anti-patrones

1. **La "bola de pelos":** 100k puntos 1px sin cluster ni densidad = inútil. Clusterizar/samplear primero.
2. **Confi ar en distancias 2D como verdades:** t-SNE/UMAP distorsionan; nunca poner números de distancia en el mapa sin avisar.
3. **Mostrar el vector crudo como UI:** nadie lee un `Vec<f32>` de 1536 dims. Mostrar payload.
4. **No hay acción tras la exploración:** un visor de solo lectura sin selección/borrado/export no aporta nada sobre una tabla.
5. **Proyección en el hilo principal:** congela la UI con >10k puntos. Worker/WASM siempre.
6. **3D por moda:** los scatterplots 3D (three.js) son peores para comparar densidades y más difíciles de navegar. Solo si el usuario lo pide (habilitarlo como toggle).
7. **Reconstruir la rueda del renderer:** regl-scatterplot y deck.gl ya resuelven picking, lasso, densidad. No escribir un scatterplot WebGL desde cero.
8. **Ignorar la similitud de texto:** un usuario de VantaDB entiende "estos documentos comparten el token X" (BM25) mejor que "la distancia coseno es 0.82". Explicar con ambos.

---

## Lecciones aplicables a VantaDB

1. **Pestaña "Explorer"/"Mapa" en la UI (web y desktop):** vista con barra de búsqueda semántica + filtros por `fields` + scatterplot 2D (regl-scatterplot) con color por cluster o por `tier`/namespace. Reusa la API `search_memory` y `NodeRecord` (`vantadb-ts/src/types.ts:83`).
2. **Proyección client-side:** UMAP-js en web worker; a futuro, portar PaCMAP al core Rust → WASM (VantaDB ya expone el core vía WASM en `vantadb-wasm/src/lib.rs:254`). La colección media de un agente (10k–100k nodos) es viable sin servidor.
3. **Vista "vecinos de":** al hacer click en un punto, mostrar los top-k similares con sus scores (`SearchResult.score`, `desktop/src/vanta.ts:46`) como lista + mini-grafo force-directed (subgrafo del IQL). Responde "¿por qué están juntos?".
4. **Explicabilidad integrada:** tarjeta de resultado con desglose BM25/vector/RRF (el core ya produce `QueryResult` con `source_type`/`exhaustivity` en `src/engine.rs:40`; `search_memory` ya tiene `explain`). Es la feature que más valor aporta por esfuerzo.
5. **Gestión desde el mapa:** selección → borrar/editar/exportar/ajustar `importance` o `tier` (campos ya en `NodeRecord`). El mapa como herramienta de mantenimiento de memoria, no solo de inspección.
6. **Tres vistas complementarias, no una:** Mapa (scatterplot global) + Vecinos (grafo/local) + Matriz (heatmap+dendrograma de los top-k de un query). Cada una responde una pregunta distinta; implementar en ese orden de ROI.
7. **No duplicar lo existente:** la tabla de resultados (`ResultsList.tsx`) ya cubre la búsqueda; el visor añade la dimensión espacial encima, con el mismo `SearchQuery`/`SearchResult` de `desktop/src/vanta.ts`.

---

## Referencias

- Apple Embedding Atlas — https://github.com/apple/embedding-atlas ; paper — https://arxiv.org/abs/2505.06386
- Google Embedding Projector (SPA) — https://projector.tensorflow.org ; standalone — https://github.com/tensorflow/embedding-projector-standalone
- UMAP-js (PAIR-code) — https://github.com/PAIR-code/umap-js
- tsne-js (karpathy) — https://github.com/karpathy/tsnejs
- PaCMAP — https://github.com/YingfanWang/PaCMAP
- openTSNE — https://github.com/pavlin-policar/openTSNE
- TriMap — https://github.com/eamid/TriMap
- regl-scatterplot — https://github.com/flekschas/regl-scatterplot ; demo — https://hci.nime.ac.at/regl-scatterplot/
- deck.gl ScatterplotLayer — https://deck.gl/docs/api-reference/layers/scatterplot-layer
- PixiJS — https://pixijs.com
- D3 force-directed graph — https://observablehq.com/@d3/force-directed-graph
- D3 chord diagram — https://observablehq.com/@d3/chord-diagram
- D3 categorical heatmap — https://observablehq.com/@d3/categorical-heatmap
- Sigma.js — https://www.sigmajs.org ; Graphology — https://graphology.github.io
- seaborn clustermap — https://seaborn.pydata.org/generated/seaborn.clustermap.html
- Qdrant Web UI — https://qdrant.tech/documentation/web-ui/
- Weaviate docs (Search/Explore) — https://weaviate.io/developers/weaviate/current/graphql-references/search
- uber-research/parallax — https://github.com/uber-research/parallax
- VantaDB core — `src/engine.rs` (QueryResult, SourceType) ; SDK TS — `vantadb-ts/src/types.ts` (NodeRecord, QueryResult) ; desktop — `desktop/src/vanta.ts` (SearchQuery, SearchResult) ; WASM — `vantadb-wasm/src/lib.rs`