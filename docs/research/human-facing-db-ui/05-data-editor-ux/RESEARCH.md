# UX/UI para edición y administración de datos

Investigación de campo (2024–2026) sobre cómo editar y administrar datos estructurados y
semiestructurados por humanos, aplicada a la UI de administración desktop de VantaDB
(Tauri v2 + React + Vite). Dominio de datos: registros con `payload` (texto libre, a veces
largo), `metadata` arbitraria key-value sin schema, vector denso (potencialmente miles de
dimensiones), TTL, timestamps y versionado.

Fecha de investigación: 2026-08-18. Todas las URLs verificadas o estables al momento de
redactar. Cifras de bundle/estrellas son aproximadas (fechadas 2026).

---

## Resumen ejecutivo

- **El grid no es el editor.** Para registros con payload largo + metadata arbitraria +
  vector, el patrón ganador es **master-detail split view** (tabla navegable a la izquierda,
  panel inspector a la derecha), igual que MongoDB Compass, DBeaver, TablePlus y Supabase.
  Editar "en la celda" es correcto solo para campos cortos y tipados (TTL, version, tags);
  el contenido pesado se edita en el inspector.
- **Librería de tabla recomendada: TanStack Table v9** (headless, MIT, tree-shakeable,
  ~16 kB gzip) + **TanStack Virtual** para ventaneo. Permite renderizar celdas a medida
  (preview de metadata como JSON, sparkline del vector, countdown de TTL) sin pelear con la
  opinión de un grid monolítico, y encaja con el design system shadcn ya usado en el repo.
  **Alternativa sólida: react-data-grid (Comcast)** si se quiere a11y de grid + edición +
  copy/paste "de fábrica" sin construirlos.
- **Metadata sin schema:** editor key-value por filas (estilo Compass): `key + type select
  + value`, con tipo **inferido del valor actual** y sobreescribible, más fallback a editor
  JSON (CodeMirror 6) para valores anidados/compuestos. No usar `react-json-view`
  (abandonado) ni `json-editor.org` (modo mantenimiento).
- **Editores de texto:** **CodeMirror 6** para payload (markdown), metadata compleja (JSON)
  y vector (JSON). Monaco es superior en features pero es un monolito de 2-4 MB; en una app
  desktop embebida pesa más de lo que aporta salvo que se quiera el "feel VS Code".
- **Vector:** nunca renderizar miles de inputs. Vista colapsada por defecto: dimensión +
  preview de N valores + stats (min/max/mean/L2) + bar-chart (sparkline) + "Copiar JSON".
  Expansión bajo demanda, con edición por chunks o pegado de JSON completo.
- **TTL:** columna con countdown en la tabla; edición con datetime + presets relativos;
  estados expirado/por expirar visualmente distintos.
- **Filtros:** react-querybuilder (MIT) si se necesitan grupos AND/OR compuestos con
  export/import; o una barra de filtros custom si basta con operadores Eq/Neq/Gt/Gte/Lt/Lte
  por campo (más ligera).
- **A11y:** implementar el patrón **ARIA grid** (role=grid, keyboard nav con flechas,
  F2/Enter para editar, Home/End/PageUp/Down, Ctrl+Space selección).
- **Anti-patrón n.º 1 detectado en el código actual:** "Load more" que agranda el limit
  (DataExplorer.tsx). Reemplazar por virtualización/infinite scroll con claves de fila
  estables. Ver `desktop/src/components/DataExplorer.tsx:6-9`.

---

## Comparativa de librerías de tablas

| Librería | Versión (2026) | Virtualización | Edición inline | Filtros | Size/eco | Licencia | Rating / actividad |
|---|---|---|---|---|---|---|---|
| **TanStack Table** | v9 | No incluida — se combina con TanStack Virtual | No (headless: tú construyes la celda editable) | No (headless: pipe de filtrado/sorting propio, client-side row-model) | ~16 kB gz core, tree-shakeable, 22M descargas/sem @org | MIT | 28k★, muy activo, v9 reescrito (TanStack Store + alien-signals, suscripciones finas) |
| **AG Grid** | v36.1 | Sí (rows+cols, nativa) | Sí, completa (celdas, batch, clic/doble-clic, formularios) | Sí (filtros column + quick filter; *advanced filters* = Enterprise) | ~150–200 kB gz Community (módulos), bundle pesado; wrapper React imperativo | MIT (Community) / comercial (Enterprise) | Muy activo, maduro, estándar en grids enterprise |
| **MUI X Data Grid** | v9.11 | Virtualización en **Pro** (comercial) | Sí (Community: row/cell edit, cell/edit modes) | Sí (Community: filter model básico; *advanced filter panel* = Pro) | ~170 kB gz aprox. (DataGrid), exige estética MUI | MIT (Community) / Pro+Premium comerciales | Muy activo; v9 añade undo/redo, list view, server-side data |
| **Glide Data Grid** | v6.x | Canvas, millones de filas, scroll nativo | Sí (integrada) | No (solo search incluida; filtros/sorting a implementar) | ~100 kB gz aprox. | MIT | 5.3k★, usado por Glide Data Editor; renderers solo en Canvas |
| **react-data-grid** | v7.x (repo Comcast) | Sí (rows+cols, solo visible en DOM) | Sí (editor de celda, onRowsChange) | No (sorting/grouping sí; filtros a implementar) | ~30–40 kB gz, sin deps, tree-shakeable | MIT | 7.7k★, activo (ahora bajo Comcast), React 19.2+, keyboard a11y, copy/paste, RTL, TreeDataGrid |

### Lectura de la comparativa

- **TanStack Table v9** es la opción "sin opiniones": motor de estado (sorting, filtering,
  faceting, grouping, pagination, expansión de filas, selección, pinning, resizing) sin
  markup. Perfecto para un grid que en VantaDB es **navegador + vista previa** (el editing
  pesado vive en el inspector). Coste: hay que construir header menu, filtros y edición.
- **react-data-grid** es el punto dulce si no se quiere construir la a11y de grid ni la
  edición de celdas: ya trae rol `grid`, teclado (flechas, Enter/F2, copy/paste), columnas
  congeladas, row selection y `onRowsChange`. Menos flexible que TanStack para celdas
  complejas, pero suficiente para columnas como TTL/version/metadata-preview.
- **AG Grid** es el más completo "batteries included", pero sus features clave para este
  dominio —master-detail, advanced filters, row grouping, server-side row model— son
  **Enterprise (pago)**. Su API imperativa (refs) choca con el estilo React idiomático y el
  bundle es el más pesado. Descartar para una app OSS desktop.
- **MUI X** trae lo esencial en MIT (edición, sorting, filtrado simple) pero la
  **virtualización está en Pro (pago)** — deal-breaker para listas largas en una app de
  base de datos — y arrastra la estética/tema MUI. Descartar salvo que el proyecto ya use MUI.
- **Glide** gana en rendimiento extremo (canvas) pero los custom renderers deben pintarse en
  canvas y la accesibilidad/ARIA es más débil (los propios autores piden bug reports de
  a11y). Para celdas ricas (preview JSON, sparklines) conviene DOM. Descartar como grid
  principal; reservable si algún día se necesitan 1M+ filas reales.

**Decisión:** base = **TanStack Table v9 + TanStack Virtual** (control total del render,
a11y grid según APG, MIT, bundle mínimo), con inspector master-detail como editor.
Si la construcción de keyboard-nav + copy/paste resulta cara, cambiar el plano de la tabla
a **react-data-grid** (mismo licencia y filosofía ligera).

---

## Comparativa de editores JSON

| Editor | Licencia | Tamaño/eco | Modo visual (árbol) | Edición | ¿Proyecto activo? | Uso recomendado |
|---|---|---|---|---|---|---|
| **Monaco Editor** | MIT | 2–4 MB JS; difícil de tree-shakear; ESM moderno disponible | No (solo texto con syntax highlight) | Texto completo, intellisense, folding, minimap, find/replace | Muy activo (MS) | Payload/JSON grande donde se quiera "feel VS Code". Overkill para snippets |
| **CodeMirror 6** | MIT | Modular; editor JSON/markdown ~150–250 kB gz según paquetes | No (texto + highlight) | Texto; `@codemirror/lang-json`, markdown, linters, autocomplete | Muy activo | **Recomendado**: ligero, tree-shakeable, fácil de embeber en Vite/Tauri |
| **react-json-view (RJV)** | MIT | ~100 kB gz aprox. | Sí (árbol colapsable, copy, theming) | onEdit/onAdd/onDelete (solo primitivos) | **ABANDONADO** (README lo declara; sugiere `@microlink/react-json-view`) | No usar; sustituir por react-json-view-lite o editor propio |
| **react-json-view-lite** | MIT | ~10 kB gz | Sí (árbol ligero) | Solo lectura (copy por nodo) | Activo | **Recomendado** para mostrar metadata/vector colapsados de solo lectura |
| **JSON Editor (json-editor.org)** | MIT | ~50–100 kB + dependencias opcionales (Ace, SimpleMDE…) | Sí (formularios generados desde JSON Schema) | Form-driven, arrays/objetos anidados, validación | **Modo mantenimiento** (activo migrado a **Jedison**) | No iniciar proyectos nuevos; útil como referencia conceptual de form-schema |
| **Jedison** | MIT | Sucesor de json-editor | Sí | Idem | Activo (sucesor oficial) | Alternativa si se quisiera form-from-schema en el futuro |
| **jq / JSONPath** | CC0/MIT | CLI / expresiones | No | Consultas | Activo | No son editores: útiles para poder de expresiones en filtros o "puedo ver este path" |

### Lectura de la comparativa

- Para un editor **desktop embebido**, CodeMirror 6 es la elección racional: modular,
  ESM, sin loader propio (Monaco exige gestionar workers/languages), arranque rápido.
  Se usa `@uiw/react-codemirror` como wrapper React y paquetes `@codemirror/lang-json` /
  `@codemirror/lang-markdown`.
- Monaco solo se justifica si la app quiere replicar la experiencia de edición de VS Code
  (intellisense de JSON Schema, múltiples cursors masivos). En Tauri, el bundle extra no
  es crítico en disco pero sí en memoria de arranque de cada ventana.
- **Nunca** usar `react-json-view` (abandonado). Para árboles de solo lectura, usar
  `react-json-view-lite` (10 kB) o construir un árbol propio sobre el design system.
- El **editor clave-valor por filas** (ver sección siguiente) es preferible a un árbol JSON
  genérico para metadata: expresa tipos de VantaValue con controles nativos (date picker,
  checkbox, number input) y evita errores de sintaxis.

---

## Patrones master-detail e inspección de registros

### Cómo lo hacen las herramientas de referencia

- **MongoDB Compass**: lista de documentos (JSON collapsible) + panel de detalle; toggle
  "Document / JSON" por documento; edición por filas key-value con type dropdown y botones
  add/remove field. Referencia directa para VantaDB por la analogía metadata↔documento.
- **DBeaver / TablePlus**: tabla principal con rows; al seleccionar una fila, **panel
  inferior o lateral** muestra la fila completa en formulario column-by-column; editor de
  texto/SQL para valores grandes; separadores arrastrables.
- **Supabase Dashboard (Table Editor)**: grid tipo Airtable con row-inspector lateral
  (formulario de columnas) + soporte JSONB con mini-editor JSON.
- **PocketBase admin**: tabla con CRUD sencillo y formulario generado por schema en un
  drawer/modal — útil como contraste "minimal".
- **Airtable / Notion databases**: filas expandibles, campos tipados, edición inline de
  primitivos, "expand record" para texto largo/multimedia.

### Patrón óptimo para VantaDB

```
┌──────────────┬──────────────────────────────┬──────────────────────────────┐
│ Sidebar      │  Data table (master)         │  Inspector (detail)          │
│ - namespaces │  id | key | payload (preview)│  tabs:                        │
│ - métricas   │  | TTL ⏳ | v | vec dims      │  1. General (payload)        │
│ - búsqueda   │  | score (search mode)       │  2. Metadata (KV editor)     │
│              │  ▶ fila seleccionada         │  3. Vector                   │
│              │                              │  4. Historial (versiones)    │
└──────────────┴──────────────────────────────┴──────────────────────────────┘
```

1. **Tabla = navegador.** Columnas fijas: `key` (mono), `namespace` (tag), `payload`
   (2-3 líneas clamp + tooltip full), `TTL` (countdown), `version`, `vec` (dims). En modo
   búsqueda añadir `score` (barra proporcional). Las columnas de metadata NO se renderizan
   en el grid por defecto: son infinitas y heterogéneas; se exploran en el inspector o con
   el query builder.
2. **Inspector = editor.** Al seleccionar una fila (click / flecha arriba-abajo), la fila
   se abre a la derecha en un **panel resizable** (split view). Tabs:
   - **General**: `namespace`, `key` (editable si el motor permite rename), `payload`
     textarea grande con **toggle preview/editar markdown**, timestamps de solo lectura.
   - **Metadata**: editor key-value por filas (sección siguiente).
   - **Vector**: vista colapsada + expansión (sección de vectores).
   - **Historial**: lista de versiones con diff y restore.
3. **Edición inline mínima en la tabla:** solo campos cortos y tipados (TTL, tags, version)
   con confirmación implícita al perder foco o con `Enter`. Todo lo demás se edita en el
   inspector. Regla: *una celda, un input nativo* (checkbox/date/number); nunca un textarea
   creciendo dentro de la tabla.
4. **Botones de acción** en el inspector: Guardar (Cmd+S), Duplicar, Eliminar, Ver como
   JSON (abre CodeMirror read-only del registro completo).
5. **Doble-click** en fila = abrir el registro en una **vista de solo lectura ampliada**
   (modal o tercera columna) para payload muy largo — alternativa a expandir filas en el
   grid.

---

## Edición de metadata arbitraria (key-value sin schema)

La metadata es un `Map<string, VantaValue>` sin schema fijo (string/int/float/bool/
datetime/list/null). El patrón de referencia es el **editor de documentos de Compass**:
una tabla de filas `clave → tipo → valor` con acciones `+ field` / remove.

### Diseño del editor KV

```
┌───────────┬─────────┬──────────────────────────────┬──┐
│ key       │ type    │ value                        │  │
├───────────┼─────────┼──────────────────────────────┼──┤
│ priority  │ number  │ [ 42 ]                       │ ×│
│ archived  │ boolean │ [✓]                          │ ×│
│ due       │ datetime│ [ 2026-08-20 12:00 ]         │ ×│
│ tags      │ list    │ [a] [b] [c]  +add            │ ×│
│ meta      │ object  │ { "k": 1, … }  (JSON editor) │ ×│
└───────────┴─────────┴──────────────────────────────┴──┘
[ + Add field ]                    (nombre nuevo con validación)
```

- **Tipo inferido del valor actual** (string→"string", true→"boolean", 42.0→"float",
  lista→"list", ausencia de valor→"null"), mostrado en un `<select>` para que el usuario
  pueda corregir la inferencia (p. ej. convertir un string numérico a int). El cambio de
  tipo **convierte** el valor (parse int/float, bool toggle, datetime picker).
- **Lista**: chips con add/remove (patrón Airtable tags). **Objeto/compuesto**: colapso a
  una línea y apertura de un editor JSON embebido (CodeMirror con `@codemirror/lang-json`)
  con validación de sintaxis en vivo; sin línea de sintaxis roja, no se puede guardar.
- **Null**: checkbox "unset" que marca el campo como null explícito o lo elimina.
- **Duplicados de key**: primera fila gana; duplicados se marcan en rojo y bloquean el
  save (o se desnormalizan a lista). Validar claves vacías y reservadas (namespace, key,
  payload, vector).
- **Rendimiento:** capar render a N filas con paginación si la metadata crece; ordenar por
  clave con opción "frecuentemente editadas primero" (las claves más comunes de los
  registros del namespace, calculadas por el core).

### Alternativas evaluadas y por qué no

- `react-json-view` (abandonado), `json-editor.org` (mantenimiento) → descartados.
- Editor JSON puro (CodeMirror) como único medio → rápido de construir pero peor UX para
  primitivos y sin controles tipados; se usa solo como fallback para objetos anidados.
- Formulario por schema → no hay schema; la inferencia de tipo sustituye a "generar form".

---

## Representación de vectores para humanos

Contexto: `Vec` de floats, posiblemente miles de dimensiones. Objetivo: verificar que el
vector existe, ver sus estadísticas, copiarlo y (rara vez) editarlo, sin colapsar la UI.

### Vista por defecto (colapsada) — el 99% de los casos

```
vec [1536 dims]  [▮▮▮▮▮▯▯▯▮▮▮▯▯▮▮▯▮▮▯▮▯▮▮▮▯▯▮▮▯▮▯▮▯▮▮▯▯▮▮▮▯▮▯▮▮▯▯▮▮▮▯▮▯▮▮▯]
   min -0.412 · max 0.398 · mean 0.021 · L2 1.000 · [0.021, -0.087, 0.334, …]
   [Copiar JSON] [Expandir y editar] [Download .npy/.json]
```

- **Miniature bar chart (sparkline)** de las primeras N muestras o de bins agrupados
  (p. ej. 200 bins promediados si dims ≫ 200) — coste O(1) de render.
- **Stats** (min/max/mean/L2) + preview de los primeros 3-5 valores.
- **Copiar JSON** directo al portapapeles (formato `[...]` completo).

### Vista expandida (bajo demanda)

- Tabla paginada/virtualizada `idx → value` (con TanStack Virtual o chunking de 100) para
  inspección/edición puntual de un valor.
- **Editar = pegar JSON completo** en un CodeMirror (validación: misma longitud, floats
  válidos; indicador "1536 valores · válido") — el flujo real de "reemplazo de embedding".
  Edición valor-a-valor individual es raro y costoso; no renderizar miles de inputs.
- **Generación opcional:** botón "recalcular embedding" que llama al pipeline de
  embeddings existente (fuera de alcance de esta investigación, solo se reseña).

### Señales visuales

- Registro **sin vector**: badge gris `no vector` (los registros de texto puro sin
  embedding son válidos en VantaDB).
- Cambio de dimensión entre versiones: advertir en Historial ("dim 1536 → 768") porque
  rompe búsqueda híbrida.

---

## TTL, versionado e historial (UX)

### TTL / expiración

- **En la tabla:** columna dedicada que muestra countdown legible ("expira en 3d 4h",
  "expira en 12 min") con color de alarma cuando < 24 h; estado **expirado** con tachado +
  badge (el registro aún puede listarse hasta que el core lo purgue).
- **Edición (inspector):** input datetime absoluto + **presets relativos** (`1h`, `1d`,
  `7d`, `30d`, `nunca`). Cambiar TTL no versiona contenido (es metadata de sistema) —
  indicarlo en UI si el core distingue TTL del body.
- **Limpiar TTL** ("nunca expira") es una acción que debe **confirmarse** (el registro ya
  no será purgado automáticamente). Mostrar consecuencia explícita.
- No usar countdown ticking por segundo en toda la tabla (cuesta caro); tick cada 30-60 s
  o solo al enfocar/virtualizar filas visibles.

### Versionado / historial

- **Version actual** visible como badge en inspector y columna en tabla (mono).
- **Historial (tab):** lista cronológica (versión, timestamp, "creado por" si existe),
  con **diff** de payload y metadata (renderizar diff de texto/JSON, no solo timestamps) y
  acción **"restaurar esta versión"** (crea una nueva versión copiando el contenido — nunca
  reescribir la historia) con confirmación.
- Regla UX: el historial es **read-only por defecto**; editar contenido crea una **nueva**
  versión. Evitar "viajar en el tiempo" en el mismo editor (confuso); usar drawer separado
  con vista side-by-side.

### CRUD y persistencia

- **Commit explícito vs autosave:** en una DB local con WAL, recomendar **guardado
  explícito** (botón "Guardar" + `Cmd+S`) con **indicador de suciedad** ("● cambios sin
  guardar") para payload/metadata/vector. Ediciones de primitivos en celdas (TTL) pueden
  autoguardar con debounce y **optimistic update** (aplicar al instante, rollback + toast
  si falla). Evitar escribir el WAL en cada keystroke.
- **Undo/redo:** pila en memoria para el editor KV y payload mientras el registro está
  abierto (no persiste entre sesiones). Suficiente; no implementar undo de operaciones
  remotas salvo que el core lo soporte.
- **Bulk edit:** selección múltiple de filas (checkbox + Shift+Space) → barra de acciones:
  "Editar TTL…", "Añadir campo metadata…", "Eliminar (n)".
- **Eliminación destructiva:** confirmación que muestre **cantidad** y consecuencia
  ("Se eliminarán 12 registros de ns 'chat'… y su historial"). Sin soft-delete (VantaDB no
  lo soporta): la única red de seguridad es la confirmación explícita. No confirmar con
  dialogo genérico: confirmar mostrando qué se va a borrar.
- **Duplicate/Clone:** crear registro copiando payload+metadata (sin vector o con vector)
  — la forma más usada de "crear variantes".

---

## Query builders visuales y filtros compuestos

### Herramienta: react-querybuilder

MIT, minimal-opinionated, con grupos AND/OR, operadores `= != < > <= >= contains
beginsWith endsWith isNull isNotNull in notIn between notBetween`, i18n/RTL y export/import
a SQL, MongoDB, CEL, JsonLogic, JSONata, Prisma, Elasticsearch, Cypher, SPARQL, Gremlin.
Compat oficial con MUI (útil si se adopta).

- **Campos de filtro para VantaDB:** `namespace`, `key`, `payload` (texto: contains/
  startsWith), `version` (=, !=, >, <), `created_at`/`updated_at` (between, >), `ttl`
  (estado: expirado/vigente/nunca), y **claves de metadata dinámicas** (descubiertas de los
  registros existentes del namespace — faceted: mostrar las N claves más frecuentes con
  su tipo inferido para elegir operador correcto).
- **Traducción a la query del motor:** el query builder devuelve JSON; un adaptador lo
  convierte al formato nativo de VantaDB (o a IQL si la UI lo expone). Persistir el JSON
  del filtro en `localStorage`/settings para reutilización ("filtros guardados").
- **Rendimiento:** los filtros deben evaluarse en el **core** (via comando Tauri), no
  filtrar 100k filas en el front.

### Alternativa ligera (si no hace falta AND/OR anidado)

Una **fila de filtros**: `[campo] [operador] [valor] [+ AND/OR]` con select tipado por
campo. Menos librería, menos bundle; suficiente si el 90% de los usuarios filtra con 1-2
condiciones. react-querybuilder se justifica cuando se quieren grupos anidados, guardado
de consultas y export a SQL/Mongo. **Recomendación:** empezar con la barra custom simple;
promover a react-querybuilder si llegan casos anidados reales (ponytail).

---

## Anti-patrones

1. **Editar vectores inline en celdas del grid.** Miles de inputs destruyen el scroll y la
   legibilidad. Vector = resumen + expansión bajo demanda.
2. **Cargar 100k+ filas en el DOM** (tabla HTML plana sin virtualización). El
   `DataExplorer` actual usa "Load more" (agrandar limit) — funcional pero rompe el scroll
   nativo y no hay offset real. Virtualizar o pedir offset/cursor al core.
3. **Autosave en cada keystroke** de payload/metadata → amplificación de escritura en WAL,
   versionado ruidoso (cada tecla crea versión). Commit explícito + dirty-state.
4. **Modal dentro de modal** para ver el detalle de un registro. Master-detail con panel
   fijo evita el stack de modales.
5. **Editar metadata solo como JSON crudo.** Sin estructura ni controles tipados, errores
   de sintaxis y de tipo constantes. KV editor + fallback JSON.
6. **Sin indicador de cambios sin guardar** (dirty state) → el usuario cierra y pierde
   edits silenciosamente.
7. **Confirmaciones de borrado genéricas** sin mostrar cantidad/consecuencia. La
   confirmación debe nombrar el impacto (n registros, historial incluido).
8. **TTL invisible en la lista.** El usuario no sabe qué está por expirar; columna con
   countdown + estados de alarma.
9. **Usar librerías abandonadas** (`react-json-view`) o en mantenimiento (`json-editor`)
   para la pieza central de la UI.
10. **Grid como único editor.** El grid navega; el inspector edita. Forzar edición de
    texto largo en celdas produce filas gigantes y desalineación.
11. **Infinite scroll sin estabilidad de claves** — re-renders de filas enteras al
    hacer scroll. Usar `rowKeyGetter`/key estable (namespace:key) y memoización de filas.
12. **Filtrar en el frontend** datasets grandes. Delegar filtrado al core.

---

## Stack y layout recomendados para la app desktop VantaDB

Estado actual: `desktop/package.json` solo tiene `react@19`, `react-dom@19`,
`@tauri-apps/api@2`, Vite 7, TS 5.8. `DataExplorer` es una tabla HTML plana sin edición.
Cero deuda de librerías — ventaja para empezar limpio.

### Stack

| Capa | Recomendación | Alternativa | Justificación |
|---|---|---|---|
| Grid | **TanStack Table v9 + TanStack Virtual** | react-data-grid (Comcast) | Headless/MIT/tree-shakeable; celdas a medida (TTL countdown, sparkline vector, preview metadata) sin fricción; encaja con design system shadcn. react-data-grid si se quiere a11y+copy/paste de fábrica |
| Texto/JSON | **CodeMirror 6** (`@uiw/react-codemirror`, `lang-json`, `lang-markdown`) | Monaco | Ligero, ESM, sin workers; payload markdown + metadata/vector JSON |
| Árbol JSON read-only | **react-json-view-lite** | custom tree | 10 kB, copy por nodo |
| KV metadata editor | **Custom component** (estilo Compass) | — | Infiere tipo de VantaValue, controles nativos, validación de duplicados |
| Query/filtros | Barra custom simple → escalar a **react-querybuilder** | — | Empezar mínimo; añadir AND/OR anidado si hace falta |
| Split/resizable | `react-resizable-panels` (MIT) o CSS grid + handles propios | — | Master-detail responsive |
| Persistencia de layout/filtros | `localStorage` vía settings Tauri | — | Columnas visibles, anchos, filtros guardados |
| Estado de datos | TanStack Query o hooks locales con refetch | — | Cache + invalidación tras mutaciones del inspector |

### Layout

1. **Shell de 3 paneles** (resizable): sidebar (namespaces, métricas, IngestForm/ExportPanel
   accesibles) → grid master → inspector detail. En ventanas estrechas, el inspector se
   vuelve drawer.
2. **Grid**: columnas `key | payload(preview clamp) | TTL(countdown) | v(version) | vec(dims)`
   (+ `score` en búsqueda). Fila seleccionada destacada; doble-click abre vista ampliada.
3. **Inspector por tabs**: General (payload markdown preview/edit) · Metadata (KV) ·
   Vector (colapsado+stats+copy, expandible) · Historial (versiones + diff + restore).
4. **Acciones globales**: Guardar (Cmd+S), Nuevo registro, Duplicar, Eliminar (confirmación
   con conteo). Barra de filtros sobre el grid.
5. **A11y**: patrón ARIA grid del APG (role=grid, keyboard: flechas, Home/End/Ctrl+Home/
   Ctrl+End/PageUp/Down, Enter/F2 para editar, Ctrl+Space/Shift+Space selección, aria-
   rowcount/colindex si hay ventaneo). Estados de carga con skeleton; empty state con CTA
   "Crear primer registro"; i18n con los diccionarios ya presentes en el repo (`web/src/lib/dictionaries.ts`).

### Próximos pasos sugeridos (por prioridad)

1. Master-detail: grid virtualizado + inspector de solo lectura (payload, metadata como
   árbol, vector colapsado, TTL, versión). Valor inmediato.
2. Editor KV metadata + commit explícito con dirty-state + Cmd+S.
3. Editor payload markdown (CodeMirror + preview toggle).
4. Filtros (barra simple → query builder) con evaluación en core.
5. Historial con diff + restore; bulk edit con confirmaciones destructivas.

---

## Referencias (URLs)

### Librerías de tablas
- TanStack Table (v9) — https://tanstack.com/table/latest
- TanStack Virtual — https://tanstack.com/virtual/latest
- AG Grid React Data Grid — https://www.ag-grid.com/react-data-grid/
- AG Grid Community vs Enterprise — https://www.ag-grid.com/react-data-grid/community-vs-enterprise/
- AG Grid Master/Detail (Enterprise) — https://www.ag-grid.com/react-data-grid/master-detail/
- MUI X Data Grid (v9) — https://mui.com/x/react-data-grid/
- MUI X Licensing (Community/Pro/Premium) — https://mui.com/x/introduction/licensing/
- MUI X Data Grid Virtualization (Pro) — https://mui.com/x/react-data-grid/virtualization/
- Glide Data Grid — https://github.com/glideapps/glide-data-grid
- react-data-grid (Comcast) — https://github.com/Comcast/react-data-grid
- react-data-grid website (features/a11y) — https://comcast.github.io/react-data-grid/

### Editores JSON y texto
- Monaco Editor — https://microsoft.github.io/monaco-editor/
- CodeMirror 6 — https://codemirror.net/
- @uiw/react-codemirror — https://github.com/uiwjs/react-codemirror
- react-json-view (⚠️ abandonado, recomienda @microlink) — https://github.com/mac-s-g/react-json-view
- @microlink/react-json-view (sucesor sugerido) — https://github.com/microlinkhq/react-json-view
- react-json-view-lite — https://github.com/zuplo/react-json-view-lite
- JSON Editor (json-editor.org, mantenimiento → Jedison) — https://github.com/json-editor/json-editor
- Jedison (sucesor) — https://github.com/germanbisurgi/jedison
- jq manual — https://jqlang.github.io/jq/
- JSONPath RFC 9535 — https://datatracker.ietf.org/doc/html/rfc9535

### Query builders y filtros
- react-querybuilder — https://react-querybuilder.js.org/
- react-querybuilder GitHub — https://github.com/react-querybuilder/react-querybuilder

### Herramientas de referencia (master-detail / CRUD)
- MongoDB Compass docs — https://www.mongodb.com/docs/compass/current/
- DBeaver — https://dbeaver.io/
- TablePlus — https://tableplus.com/
- Supabase Table Editor — https://supabase.com/docs/guides/database/tables
- PocketBase — https://pocketbase.io/docs/
- Retool — https://retool.com/
- Airtable — https://airtable.com/
- Notion databases — https://www.notion.so/help/guides/database

### Accesibilidad
- WAI-ARIA APG — Grid pattern — https://www.w3.org/WAI/ARIA/apg/patterns/grid/
- WAI-ARIA APG — Treegrid pattern — https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/
- MDN ARIA grid role — https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/grid_role

### Dominio VantaDB
- VantaDB — https://vantadb.com/ (sitio oficial del proyecto, repo local)
- Manual de operación — `.opencode/VANTADB-OPERATING-MANUAL.md` (repo)
- Componentes actuales — `desktop/src/components/DataExplorer.tsx` (patrón "Load more",
  pendiente de migrar a virtualización) y `desktop/package.json` (React 19, Tauri 2, sin
  librerías de UI todavía)