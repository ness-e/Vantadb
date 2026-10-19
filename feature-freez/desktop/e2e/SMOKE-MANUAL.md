# Smoke manual — GraphLens (GRAFO) y SpaceLens (ESPACIO)

> H-07 (DESKTOP-QW10) — contrato: `graph/space quedan smoke visual manual documentado; npx playwright test verde`.
> Estos lentes son WebGL/canvas + datos vivos (HNSW/UMAP) no-deterministas en CI, por eso su verificación es visual
> guiada, no E2E automática. Este doc es la checklist para el owner / reviewer antes de cerrar el plan quickwins.

## Prereq

```ps1
cargo build -p vantadb                # opcional si ya hay target/debug/vanta-cli.exe
npm --prefix desktop run build        # tsc && vite, 2863 modules, dist/
npx --prefix desktop playwright test  # 4 specs (flujo-critico, daud01-temas, multi-perfil, proxy-dashboard) verde
# Para smoke manual, levanta la app real (Tauri) o el web build embebido:
cargo build -p vanta-cli              # produce target/debug/vanta-cli.exe
# En un terminal: target/debug/vanta-cli.exe server --http -p 8090 -d (New-TemporaryFile) --dashboard-dir desktop/dist-web
#   (o `cargo tauri dev` para Tauri WebView)
```

## GRAFO (IQL) — `desktop/src/components/graph/GraphLens.tsx`

Dataset: `vanta-cli server` con al menos 10 registros con graph edges (o `IQL` insert `EDGE` si aplica). Seed vía `POST /api/v2/records/batch`.

Pasos:

1. Abrí la superficie **IQL** (sidebar **⌘ IQL** o palette `Ctrl+K → iql`).
2. Verificá que el canvas 3D carga: nodos naranja toon, aristas líneas, sin error `WebGL no soportado`.
3. Toolbar:
   - `⛶ fit` centra el grafo.
   - `↺ reset` vuelve al seed (hubs del namespace).
   - `labels` toggle muestra/oculta labels (solo top-20 por degree).
   - `⌨ iql` colapsa/expande la consola inferior (220px).
4. Interacción:
   - Click en un nodo → expande vecinos (≤50) con cap 500 + fade de nodos viejos.
   - Arrastrar = orbitar, scroll = zoom, click vacío = deseleccionar.
   - `hover` muestra tooltip del nodo (label / id).
5. Consola IQL (cuando expandida):
   - Escribí `SELECT * FROM default` (o el namespace de seed) → `Ctrl+Enter` ejecuta.
   - El resultado `Read` resalta nodos en el canvas (badge `● N resaltados`).
   - Un query inválido muestra error en el notice (no crash).
6. Accesibilidad:
   - Debajo del canvas hay una lista `sr-only` de nodos (lector de pantalla).

Esperado: sin `pageerror`/`console.error` (salvo `vanta_graph_* unsupported` en backends sin grafo — degradado ok),
canvas con `role="img"` y `aria-label` describiendo nodos/aristas.

## ESPACIO — `desktop/src/components/space/SpaceLens.tsx`

Dataset: al menos 20 registros con `vector` / `embedding` (requiere HNSW). Seed con `vector: number[]` de dim 8+.

Pasos:

1. Abrí la superficie **ESPACIO** (sidebar `✳ ESPACIO` o palette `Ctrl+K → espacio`).
2. Toolbar:
   - Select `todos los namespaces` / filtro por namespace.
   - Botón `⤒ proyectar` dispara UMAP-js en worker (seed fijo = reproducible, aviso `Proyección lista: N puntos`).
3. Canvas:
   - Puntos coloreados por namespace (palette categórica neón, 10 colores).
   - Hover = tooltip con `namespace/key` + preview del payload (140 chars).
   - Click en un punto = abre en Inspector (mismo patrón que RETRIEVAL).
   - `SHIFT+arrastrar` = lasso → `SelectionBar` (barra batch ops) con contador.
4. SelectionBar (solo con selección >0):
   - `Exportar` → descarga `vanta-selection-<stamp>.jsonl` importable 1:1 (ver `export-jsonl.ts`).
   - `Eliminar` → confirmación 2 pasos inline → `softDeleteBatch` → papelera + `Ctrl+Z` restaura el lote.
   - `Limpiar` → deselecciona.
5. Nota: `UMAP-js distorsiona distancias — solo agrupa por vecindad` (banner bajo toolbar) debe estar visible.
6. Error handling:
   - Sin backend / sin embeddings → mensaje `sin proyección — click en ⤒ proyectar` o `WebGL no soportado`.
   - `sr-only` lista los primeros 200 puntos para lector de pantalla.

Esperado: `regl-scatterplot` `isSupported` true, resize observa el contenedor, `pointOver/pointOut/select/lassoStart/lassoEnd` sin leaks,
`plot.destroy()` al desmontar la surface.

## Cierre

- Al terminar el smoke, capturá `npx playwright test --reporter=list` + 2 screenshots manuales (opcional `e2e/screenshots/smoke-graph.png|smoke-space.png`)
  y adjuntalos al PR / recitation de DESKTOP-QW10.
- Si alguno falla, abrir fila `FIND-*` en Backlog vía `prompts/findings.md` (no anotar solo en plan).

## Referencias

- `desktop/e2e/multi-perfil.spec.ts` + `proxy-dashboard.spec.ts` — E2E auto de H-07 (mock upstream).
- `desktop/src/components/graph/useGraphData.ts` — cap MAX_NODES 500, expand BFS limit 50.
- `desktop/src/components/space/useProjection.ts` + `projection.worker.ts` — UMAP seed fijo.
