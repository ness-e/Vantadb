# Plan de Ejecución: Vanta Studio — Fase 0 (consola human-facing desktop)

> **Campaign ID:** 7f1c9a4e-8b2d-4c3e-9a6f-2d5b8e1a9c40
> **Inicio:** 2026-08-18
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/research/human-facing-db-ui/06-synthesis/SYNTHESIS.md` (concepto "Vanta Studio" + Fase 0 §7) + decisiones del usuario 2026-08-18 (ver Decisiones).
> **Modo:** secuencial — prototipo visual primero, luego implementación React.

## Decisiones del usuario (2026-08-18)

| # | Decisión | Valor |
|---|----------|-------|
| D1 | Alcance | **Fase 0 completa** (workspace + HOME + grid + inspector + filtros + undo/papelera + Ctrl+K) |
| D2 | Diff de versiones | Crear tarea de backlog: `VantaMemoryRecord` debe retener versiones anteriores (planif/investigación/análisis/impl). Propuesta Historial+Diff **en espera** hasta completarla. |
| D3 | Distribución | **Solo desktop** (web/embebido → Fase 3) |
| D4 | Dirección visual | **Manga Tradicional & Grabado Linocut (Neo-brutalista)** — tokens de `web/`: cream `#FBF9F5`, ink `#000`, neon `#FF5500`, paper `#F2EDE2`, smoke `#1A1A1A`; borde `4px black` + sombra `6px_6px_0_0_#000`; efectos press/glitch/halftone/ink-corner; fuentes Anton (display) + Space Mono (tech) + Geist (body). |
| D5 | Grafo | **Renderer three.js propio (react-three-fiber)** — control total de shaders (toon+outline) y perf, física en worker. |
| D6 | Estilos desktop | **Tailwind v4 + tokens de la web** (replicar utilities manga/linocut) |
| D7 | Tema | **Ambos con toggle, default claro** (cream). ⚠️ La web NO implementa dark mode → la paleta dark (Vanta Black `#0a0a0a`, fg cream, neon preservado) se **diseña propia** en VS-01, con override de utilities hardcodeadas en `#000` (`.press`, `.halftone`, `.scroll-manga`, `.ink-divider`). |
| D8 | Prototipo | **Fase 0 core en HTML** (HOME + MEMORIAS + Inspector), validado con Playwright |
| D9 | Mascota | **MARK** (personaje SVG del hero de la web) — crear **variante desktop** (misma geometría: ring negro sin relleno + esfera neon + 2 ojos barra verticales + grafo interactivo), comportamiento de "asistente de datos". NO la mascota gato. |

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 1 prototipo + 2 fundamentos + 9 Fase 0 + 7 VS-CORE | Historial+Diff (espera VS-CORE-07) + hallazgos 02/05 (favoritos, Copy-as, import CSV/JSON, a11y chips) | Fases 1–3 | — |

## Orden de ejecución

1. **Prototipo visual HTML** (validación con Playwright) — fija el diseño antes de codificar.
2. **Fundamentos visuales** en desktop: Tailwind v4 + tokens + tema toggle (+ paleta dark diseñada propia) + MARK variante.
3. **Fase 0 React**: workspace → **VS-10 (bridge put/update)** + **VS-11 (DTO enriquecido)** → HOME → grid → inspector (con CodeMirror) → filtros → undo/papelera → Ctrl+K.
4. **Gaps del core** (VS-CORE-01..07): VS-CORE-01 y VS-CORE-02 son **bloqueantes de VS-05/VS-04** y se ejecutan en paralelo desde el inicio; VS-CORE-03/06 se re-escopean (ya existen en core); el resto se difiere.

## Archivos protegidos (NO tocar por sub-agentes)

- `docs/Backlog.md` — migración la hace el lead
- `web/src/components/vanta/mark/` — solo lectura (referencia para variante desktop)
- `src/sdk/` (tipos públicos) — cambios solo vía task file con contrato

---

## Wave 0 — Prototipo visual (HTML, validado con Playwright)

### Task 1: VS-00 — Prototipo HTML de Fase 0 core (HOME + MEMORIAS + Inspector)
- **Archivos clave:** `desktop/prototype/index.html` (nuevo, autónomo con Tailwind CDN o CSS inline), `docs/research/human-facing-db-ui/06-synthesis/SYNTHESIS.md`
- **Gate Justificación:** D8; el diseño se fija ANTES de codificar React (el estilo manga/linocut necesita validación visual temprana); validación con Playwright (1440×900).
- **Contrato:** 3 pantallas navegables en 1 HTML: HOME (6-8 cards overview: conteos por namespace, tipos metadata, TTL próximos, actividad reciente), MEMORIAS (grid con key/payload/metadata chips tipados/vector badge/version/updated/TTL countdown), Inspector (General/Metadata KV con tipo inferido/Vector stats + botones copiar). Tokens exactos de la web. MARK variante desktop presente (empty state + idle).
- **Estado:** ✅ HECHO (2026-08-18) — `desktop/prototype/index.html` construido con Tailwind v4 browser CDN + `@theme inline`/`:root`/utilities VERBATIM de `web/src/app/globals.css` + Google Fonts (Anton/Space Mono/Geist); MARK portado sin Anime.js (rAF lerp, CSS keyframes, SMIL glow ring, 10 nodos/18 edges/4 SfxLabels/tag IN-PROCESS); validado con Playwright 1440×900: HOME (6 cards) ✓, MEMORIAS (18 cards) ✓, Inspector (General/KV/Vector/Commit explícito) ✓, toggle dark (bg #0a0a0a) ✓, estilos exactos (bg #FBF9F5, borde 4px #000, sombra 6px 6px 0 0 #000) ✓, sin overflow ✓. Screenshots en `desktop/prototype/vs00-*.png`.

## Wave 1 — Fundamentos visuales desktop

### Task 2: VS-01 — Tailwind v4 + tokens manga/linocut + tema toggle
- **Archivos clave:** `desktop/package.json` (deps: tailwindcss v4, @tailwindcss/vite), `desktop/src/index.css` (replicar `@theme inline` + `:root` COMPLETOS de `web/src/app/globals.css` — no solo 5 colores: incluye muted `#ECE6D8`, muted-foreground `#3A3A3A`, primary/secondary/accent/destructive, border/input/ring `#000`, sidebar, chart-1..5 `#000 #FF5500 #4A4A4A #1A1A1A #C9C0AC`, `--radius: 0.125rem`, motion tokens `90/200/300ms` + ease; **`ink-corner` NO existe en globals.css → NO replicar, definir como utility propia si se necesita**), utilities reales: `.press`/`.press-lg`/`.press-neon`/`.glow-neon`/`.glitch-hover`/`.halftone`/`.halftone-fade`/`.speed-lines`/`.speed-lines-radial`/`.grid-tech`/`.text-stencil`/`.text-outline-neon`/`.scanlines`/`.scroll-manga`/`.glow-box-neon`/`.neon-underline`/`.btn-neon-glow`/`.ink-divider`/`.vanta-slider`/`.stagger-children` + keyframes `animate-*`; **a11y**: `:focus-visible { outline: 3px solid #FF5500 }` + `prefers-reduced-motion`; `--destructive: #FF5500` (neon, no rojo)), `desktop/src/main.tsx`, `desktop/index.html` (fuentes Anton + Space Mono + Geist locales, Tauri offline).
- **Dark mode (D7):** diseñar paleta dark propia (la web NO tiene tokens dark): bg Vanta Black `#0a0a0a`, fg cream `#FBF9F5`, neon preservado, surface `#1A1A1A`; `@custom-variant dark` en Tailwind v4; **override de utilities hardcodeadas en `#000`** (`.press` shadow, `.halftone` dots, `.scroll-manga`, `.ink-divider`) para que inviertan en dark.
- **Gate Justificación:** D4/D6/D7; sin esto ninguna tarea de Fase 0 tiene estética.
- **Contrato:** `npm run build` (tsc + vite) verde; token `--color-neon: #FF5500` y utility `.press` presentes; toggle claro/oscuro funcional (clase `.dark`), default claro; fuentes cargadas localmente; foco visible neon + reduced-motion presentes.
- **Estado:** ✅ HECHO (2026-08-18) — `desktop/src/index.css` (900+ líneas: Tailwind v4, `@custom-variant dark`, `@theme inline`/`:root` verbatim de globals.css, dark D7 propia con `--muted-foreground:#C9C0AC` porque `#3A3A3A` falla AA sobre `#0a0a0a`, utilities manga con overrides `.dark`, 12 keyframes, a11y `:focus-visible` neon + `prefers-reduced-motion` + `.skip-link`, 8 `@font-face` → `desktop/public/fonts/*.woff2`), `vite.config.ts` (+plugin tailwindcss v4.3.3), `main.tsx` (init tema pre-mount, `localStorage "vanta-theme"`, default claro), `App.tsx` (toggle `.press`), `index.html` (title "Vanta Studio"). Commit `fa6c1427`. Build verde (44 modules, CSS 36.78 kB/gzip 8.33 kB); contrato verificado en dist.

### Task 3: VS-02 — MARK variante desktop (asistente de datos)
- **Archivos clave:** `desktop/src/components/mark/Mark.tsx` (nuevo; port de `web/src/components/vanta/mark/mark-classic.tsx` + `use-mark-interaction.ts`), `desktop/src/components/mark/mark-studio.tsx` (variante: estados idle/loading/empty/error según estado de la app)
- **Gate Justificación:** D9; MARK es la mascota del producto; la variante desktop le da vida al workspace (empty states, loading, toasts).
- **Especificación (verificada contra la web):** 2 SVGs viewBox `0 0 100 100`: (a) grafo bg `preserveAspectRatio="none"` — 10 nodos (x,y,r de `mark-classic.tsx:19-30`) + 18 edges (:32-36), edge `stroke=currentColor text-black/25 width 0.25 (0.4 hover) strokeDasharray="1 1.2" opacity 0.5 (0.95 hover)`, nodo fill `currentColor black/35` (hover `#FF5500`), hit-target transparent r=5; (b) MARK svg 78%×78%, drop-shadow `0 0 28px rgba(255,85,0,.22)`: ring `r=42 fill=none stroke=#000 width=3.5` + **glow ring SMIL** `r 42→46→42 / opacity .3→0→.3, 3.5s loop` + esfera `r=22 fill #FF5500` en `50+sphereOffset` (scale .94 si annoyed) + ojos rects `x=43/57+pupilOffset.x-2 width 4 rx=2 y=45+pupilOffset.y height=squintHeight (3–10)`. Etiquetas: 4 SfxLabels ("1.2ms" neon TL, "RRF" ink TR, "WAL · CRC32C" ink BL, "ZERO NET" neon BR) + tag "IN-PROCESS" -4deg neon + hint chip "◆ click me · move mouse"/"◆ blink".
- **Interacción sin Anime.js (verificado viable):** squint es React puro (port directo); follow → rAF lerp `current += (target-current)*(1-exp(-dt/τ))`, τ≈60ms ojos / 130ms esfera; blink → CSS transition/WAAPI sobre height/y (cierre 60ms inQuad → hold 50ms → apertura 120ms outQuad, ciclo left→right→both); pulse nodos → keyframes CSS `transform-box: fill-box; transform: scale()`. **Mantener `prefers-reduced-motion`** (la web lo tiene). Se pierde solo el ease elástico (outElastic) del pulse ambiental — declarado.
- **Contrato:** mismas proporciones SVG y comportamiento; interactivo (ojos siguen mouse, blink en click, squint); variante con estados data-driven (idle/loading/empty/error); reduced-motion respetado.
- **Estado:** ✅ HECHO (2026-08-18) — `desktop/src/components/mark/` (4 archivos: `use-mark-interaction.ts`, `Mark.tsx`, `mark-studio.tsx`, `mark.css`) commit `2573d8a5`. Port sin Anime.js: rAF lerp exp (τ 60/130ms), squint React puro, blink WAAPI (cierre 60ms inQuad → hold 50ms → apertura 120ms outQuad, ciclo left→right→both, setAttribute final), pulse nodos CSS keyframes `transform-box: fill-box`, SMIL glow condicional a reduced-motion; variante `MarkStudio` con estados idle/loading/empty/error; CSS plano namespaced `.vmark-*` (VS-01 Tailwind aún pendiente). `npm run build` verde (3×).

## Wave 2 — Fase 0: estructura + superficies (React)

### Task 4: VS-03 — Workspace unificado (reestructurar App.tsx)
- **Archivos clave:** `desktop/src/App.tsx` (reestructurar: quitar paneles apilados MetricsGrid/KpiCards/SopPanel/ProcessPanel sueltos), `desktop/src/components/layout/WorkspaceShell.tsx` (nuevo: Sidebar + Topbar + Superficie central + Inspector)
- **Gate Justificación:** P4 anti split-attention; el layout es la base de todo.
- **Contrato:** Sidebar (RESUMEN + namespaces con conteos + Timeline/Actividad/Índices/IQL), Topbar (búsqueda global + namespace activo + Ctrl+K), superficie central con contexto (namespace o registro), Inspector derecho (master-detail). Los paneles legacy se reubican como lentes/superficies, no se borra funcionalidad.
- **Estado:** ⏳ PENDIENTE

### Task 5: VS-10 — Bridge Tauri: comando put/update (nuevo, crítico)
- **Archivos clave:** `desktop/src-tauri/src/commands/data.rs` (nuevo comando `vanta_put`), `desktop/src/vanta.ts`
- **Gate Justificación:** hallazgo CRITICO del revisor: NO existe `put` en `vanta.ts` ni comando `vanta_put` en Tauri → VS-06 no puede Guardar ni editar TTL. Bloqueante de VS-06.
- **Contrato:** comando Tauri `vanta_put(namespace, key, payload, metadata, expires_at_ms?)` mapeando a `VantaEmbedded.put`; expuesto en `vanta.ts` con tipos. Verificado con `cargo check` + `npm run build`.
- **Estado:** ⏳ PENDIENTE

### Task 6: VS-11 — Bridge Tauri: enriquecer DTO del registro (nuevo, crítico)
- **Archivos clave:** `desktop/src-tauri/src/connections/types.rs` (DTO `MemoryRecord`), `src/sdk/types.rs:175-201` (fuente), `desktop/src/vanta.ts`
- **Gate Justificación:** hallazgo CRITICO del revisor: el DTO del bridge NO tiene `version`, `node_id`, `updated_at_ms`, `expires_at_ms`, `vector` → VS-05 (columnas version/updated_at/TTL) y VS-06 (timestamps/TTL/vector) son imposibles. Bloqueante de VS-05/VS-06.
- **Contrato:** `MemoryRecord` enriquecido con todos los campos de `VantaMemoryRecord` + mapeo completo; `vanta.ts` tipado. Verificado con `cargo check` + `npm run build`.
- **Estado:** ⏳ PENDIENTE

### Task 7: VS-04 — HOME/overview (Fix 1)
- **Archivos clave:** `desktop/src/components/home/HomeOverview.tsx` (nuevo), datos de contadores (depende de **VS-CORE-02**; fallback = list+count local)
- **Gate Justificación:** P3 overview first (Shneiderman).
- **Contrato:** 6-8 cards: conteo por namespace + tendencia (de VS-CORE-02), distribución de tipos metadata (mini-histograma), próximos a expirar (TTL) + expirados recientes, **actividad reciente = registros actualizados recientemente (updated_at desc)** — decisión de usuario: audit log real llega con ACTIVITY/Timeline en Fase 1. Nada abre si no se requiere; encoding redundante (color+ícono+texto).
- **Estado:** ⏳ PENDIENTE

### Task 8: VS-05 — MEMORIAS: grid virtualizado (reemplazar "Load more")
- **Archivos clave:** `desktop/src/components/DataExplorer.tsx` (reescribir), deps: `@tanstack/react-table`, `@tanstack/react-virtual`; requiere VS-11 (DTO enriquecido) y VS-CORE-01 (cursor en bridge)
- **Gate Justificación:** P2/P1; el grid es el centro permanente; "Load more" es anti-patrón (reporte 05).
- **Contrato:** TanStack Table v9 + TanStack Virtual; paginación por cursor (usa VS-CORE-01); columnas: key (mono), payload (preview 1 línea), metadata (chips tipados), vector (badge dim), version (chip), updated_at (legible+relativa), TTL (barra countdown). Sort/filtro por columna.
- **Estado:** ⏳ PENDIENTE

### Task 9: VS-06 — Inspector de registro (master-detail + commit explícito + CodeMirror)
- **Archivos clave:** `desktop/src/components/inspector/Inspector.tsx`, `inspector/GeneralTab.tsx`, `inspector/MetadataTab.tsx`, `inspector/VectorTab.tsx`, `inspector/PayloadTab.tsx` (nuevos), dep `@uiw/react-codemirror` + `@codemirror/lang-json` + `@codemirror/lang-markdown`; ancla en VS-10 (put) + VS-11 (DTO)
- **Gate Justificación:** P2/P5/P6; el P0 de 02 y 05. **Decisión de usuario:** edición de payload en Fase 0 (CodeMirror 6).
- **Contrato:** Tabs General (key/ns/node_id mono/timestamps/version/TTL editable con countdown), **Payload (preview markdown ↔ editar JSON con CodeMirror 6, lint)** — decisión de usuario, Metadata (KV editor con tipo inferido de `VantaValue`: string/int/float/bool/datetime/list/null; agregar/quitar filas), Vector (colapsado + stats norma/min/max + sparkline + copiar/pegar JSON). **Nunca auto-guardar**: Editar → ver diff → Guardar (vía VS-10 put) / Revertir (commit explícito P6).
- **Estado:** ⏳ PENDIENTE

### Task 10: VS-07 — Filtros compuestos en búsqueda
- **Archivos clave:** `desktop/src/components/search/FiltersBuilder.tsx` (nuevo), dep `react-querybuilder`
- **Gate Justificación:** P5; filtros visuales por metadata (VantaFilterOp: Eq/Neq/Gt/Lt/Gte/Lte) sin escribir JSON.
- **Contrato:** query builder visual AND/OR sobre metadata tipada; se serializa a `VantaMemoryFilter`; compatible con la búsqueda híbrida global de la Topbar.
- **Estado:** ⏳ PENDIENTE

### Task 11: VS-08 — Undo + papelera (Fix 4)
- **Archivos clave:** `desktop/src/store/undo.ts` (nuevo), deps: `zustand`
- **Gate Justificación:** P8 recuperación de errores (Norman).
- **Contrato:** undo por snapshot del estado de la sesión (Ctrl+Z), soft-delete con papelera (tombstones) y restore, confirmación en destructivos (eliminar/sobrescribir).
- **Estado:** ⏳ PENDIENTE

### Task 12: VS-09 — Command palette (Ctrl+K)
- **Archivos clave:** `desktop/src/components/palette/CommandPalette.tsx` (nuevo), dep `cmdk`
- **Gate Justificación:** P9 teclado-first.
- **Contrato:** acciones básicas (abrir namespace, buscar key, exportar, borrar, undo, toggle tema, abrir lentes); disparo global Ctrl+K; atajos visibles. **IQL movido a Fase 2** (el bridge no expone `vanta_query` en Fase 0).
- **Estado:** ⏳ PENDIENTE

## Wave 3 — Gaps del core (backlog VS-CORE; en paralelo)

### Task 13: VS-CORE-01 — Cursor/paginación en el bridge desktop (re-scopeado)
- **Archivos clave:** `desktop/src-tauri/src/commands/data.rs:65` (comando `vanta_list`), `desktop/src/vanta.ts:159`; el core YA tiene cursor (`list` con `options.cursor` + `next_cursor` en `src/sdk/api.rs:545` y en Python/TS/WASM) — gap §8.1 real solo para Desktop
- **Gate Justificación:** sin cursor no hay virtualización real (VS-05). Bloqueante de VS-05.
- **Contrato:** exponer `cursor`/`next_cursor` en el comando Tauri `vanta_list` + `vanta.ts` (aditivo, compat backward).
- **Estado:** ⏳ PENDIENTE

### Task 14: VS-CORE-02 — Contadores por namespace + stats TTL
- **Archivos clave:** `src/sdk/api.rs` (`count` en :1278), `src/metrics/core/snapshot.rs:41` (50 campos, no 72)
- **Gate Justificación:** sidebar + HOME (VS-04) los necesitan; gap §8.3. Bloqueante de VS-04.
- **Contrato:** método que devuelva `{namespace: {count, expiring_soon, expired}}` reutilizando `count`/scan; sin N llamadas paginadas.
- **Estado:** ✅ HECHO (2026-08-18) — commit `822f7742`. Tipos `VantaNamespaceStats`/`Map`/`DEFAULT_EXPIRING_SOON_WINDOW_MS` en `types.rs` + re-exports; método `namespace_stats(Option<u64>)` en `api.rs` tras `count` (:1303) — UNA pasada de `scan_nodes()` con `memory_record_from_node_include_expired` (clasificación `expired <= now`, `expiring_soon <= now+window`); helper `memory_record_from_node_include_expired`/`inner` aditivo en `serialization/mod.rs` (original intacto); 4 unit tests + 1 integración (`memory_api.rs`); docs `EMBEDDED_SDK.md`. Verify: fmt ✅, clippy -D warnings ✅, namespace_stats 13/13 ✅, audit workspace 1929/1930 (1 fallo PRE-EXISTENTE ajeno `storage::engine::maintenance::test_consolidate_node_with_binary_vector`, reproducido en HEAD limpio — escalar a vanta-arch/vanta-engine).

### Task 15: VS-CORE-03 — `explain` estructurado: ya existe → consumir (re-scopeado)
- **Archivos clave:** `src/sdk/types.rs:418` (`VantaSearchExplanation` con `fusion_report` text/vector + `rrf_k`), `types.rs:429` (`VantaSearchExplanationHit`: score, snippet, matched_tokens, `bm25_terms`, `rrf_text_rank`/`rrf_vector_rank`), expuesto en Rust `explain_memory_search`, Python, TS `explainSearch`
- **Gate Justificación:** gap §8.2 ya RESUELTO en core; NO implementar nuevo. Fase 1 (lente RETRIEVAL) consume lo existente.
- **Contrato:** Fase 1 consume `explain_memory_search`; opcional (si la lente lo requiere): añadir vector score crudo al explanation (hoy solo rank).
- **Estado:** ⏳ PENDIENTE (Fase 1) — re-scopeado de "crear" a "consumir"

### Task 16: VS-CORE-04 — Exportar selección/query (no solo namespace)
- **Archivos clave:** `src/sdk/serialization/impl_export.rs` (`export_namespace:121`, `export_all:151`, internamente `records_for_namespace(ns, filters):76` ya acepta filtros)
- **Gate Justificación:** P12/P13 (mapa como herramienta de mantenimiento); gap §8.4.
- **Contrato:** añadir `filter: Option<VantaMemoryFilter>` a `export_namespace` (aditivo) → export de subconjunto a JSONL/`.vdbdump`.
- **Estado:** ⏳ PENDIENTE (Fase 2)

### Task 17: VS-CORE-05 — Batch delete con filtro desde UI
- **Archivos clave:** `desktop/src/vanta.ts` (existe `delete_by_filter` en `src/sdk/api.rs:1210`; NO está en Python/WASM/TS/`vanta.ts` — solo Rust+CLI), gap §8.6
- **Contrato:** exponer en WASM → `vanta.ts` + comando Tauri `vanta_delete_by_filter(namespace, filter)`; confirmación + undo integrado.
- **Estado:** ⏳ PENDIENTE (Fase 2)

### Task 18: VS-CORE-06 — IQL en desktop: exponer en bridge + autocompletado (re-scopeado)
- **Archivos clave:** nativo `src/sdk/api.rs:1110` (`query`), WASM `vantadb-wasm/src/lib.rs:1210`, TS `vantadb.ts:680` — ya expuesto; `src/parser.rs::parse_statement` (nom) es crate-interno
- **Gate Justificación:** gap §8.5 YA resuelto en core/bindings; falta exponer en bridge Tauri + shim de autocompletado para la consola IQL (lente GRAFO, Fase 2).
- **Contrato:** comando Tauri `vanta_query` (envuelve `query` nativo); autocomplete vía shim core-side (prefix sobre `parse_statement`) que exponga lista de tokens a la UI.
- **Estado:** ⏳ PENDIENTE (Fase 2) — re-scopeado de "confirmar" a "exponer+autocomplete"

### Task 19: VS-CORE-07 — Retención de versiones históricas en `VantaMemoryRecord` (D2)
- **Archivos clave:** `src/sdk/types.rs:175` (`VantaMemoryRecord`), `src/sdk/api.rs:291-295` (put bump in-place `version+1`), diseño de almacenamiento (KV extra `versions/{ns}\0{key}\0{ver}` en Fjall = 1 write extra por put)
- **Gate Justificación:** D2 explícito del usuario: **planificación → investigación → análisis → implementación**. Destraba Historial+Diff (Fix 3, Fase 1) — que queda **EN ESPERA** hasta completarse.
- **Contrato:** análisis de decisión en el core (retener snapshots n-1 vs solo version actual; coste de almacenamiento/compacción; API de acceso) + propuesta; implementación tras aprobación.
- **Estado:** ⏳ PENDIENTE

---

## Relación con P27 (Vanta Memory Engine)

Integración **por contratos, no por ejecución** — campañas independientes. Nada de esto bloquea la Fase 0; la integración real se toca cuando vanta-memory F4/F5 exista (2ª iteración). Punto de unión principal ya decidido en vanta-memory D10: *"Hook síncrono local + estado en store (MEM-28); Vanta Studio lee el estado"*.

| # | Contrato | Lado Studio (P26) | Lado Memory (P27) | Estado |
|---|----------|-------------------|-------------------|--------|
| 1 | `explain_memory_search` (VS-CORE-03, ya existe en core) | Lente RETRIEVAL (Fase 1) muestra por qué | Recall (F4) usa el mismo search | Un contrato, dos consumidores — ya resuelto en core |
| 2 | Nodos escena + META `{created,updated,summary,heat}` | Grafo/IQL (Fase 2) + Inspector renderizan escenas/skills/entities | F4 añade nodo escena al grafo core (L2, MEM-12) | Inspector KV genérico ya los cubre — sin código ahora |
| 3 | Audit log JSONL compartido | ACTIVITY + Timeline (Fase 1) | Telemetría por capa (MEM-34): eventos L1/L2/L3/offload | Memory escribe en el MISMO audit log que Studio lee — disciplina, no código |
| 4 | DTO estado (MEM-28) | Studio lee estado vía bridge Tauri | State store (pending→ready, run_id) | Mismo patrón que VS-11 (DTO enriquecido); definir cuando exista F7 |

**Punto de diseño compartido (no bloqueante):** VS-CORE-07 (retención de versiones) lo necesitan ambos — Studio para Historial+Diff, memory para offload/skills versionadas. Acordar el diseño una sola vez cuando VS-CORE-07 se ejecute (task file con cláusula de doble consumidor). Ver también: la lente RETRIEVAL (Fase 1) podrá consumir el search profile que MEM-01 parametrice (mismas estructuras que `explain`).

---

## DEFER (fuera de este plan)

| Item | Cuándo | Estado |
|------|--------|--------|
| Lente RETRIEVAL (barras apiladas BM25/HNSW/RRF) | Fase 1 | Consume VS-CORE-03 (ya existe en core) |
| Historial+Diff entre versiones (Fix 3) | Fase 1 | **En espera hasta VS-CORE-07** (D2) |
| ACTIVITY + Timeline (audit log) | Fase 1 | — |
| Deep links `vanta://` + export vistas + reporte markdown | Fase 1 | — |
| **Favoritos/historial de búsqueda** (02 P1, 05 filtros) | Fase 1 | Añadido tras auditoría — barato (localStorage), complementa Ctrl+K |
| **Copy-as** (02 P3: copiar registro/query/key en JSON/JSONL/markdown) | Fase 1 | Añadido tras auditoría |
| **A11y pass Fase 1** (encoding redundante chips/badges en MEMORIAS/Inspector) | Fase 1 | Añadido tras auditoría |
| Lente GRAFO (R3F propio, D5) + IQL console | Fase 2 | Espera VS-CORE-06; trade-off R3F vs react-force-graph/Sigma documentado (D5, decisión de usuario) |
| **Import CSV/JSON pegado** (02 P2) sobre IngestForm legacy | Fase 2 | Añadido tras auditoría |
| Lente ESPACIO (regl-scatterplot + UMAP-js worker) | Fase 2 | — |
| Batch ops con confirmación + undo | Fase 2 | — |
| Fase 3 web/embebido | Fase 3 | D3: solo desktop por ahora |

=== RECITATION ===