# INV-desktop-prod — Investigación de producto: Vanta Studio (desktop Tauri 2)

> **Fecha:** 2026-08-25 · **Módulo:** `desktop` · **Tipo:** App desktop Tauri 2 (GUI multi-conexión)
> **Usuarios objetivo:** Devs que gestionan múltiples conexiones VantaDB (embebida/server/proxy/MCP) desde una GUI local
> **Competidor principal:** MongoDB Compass · **Plantilla:** product (UX/a11y/perf, no API pública)
> **Modo:** read-only · Hallazgos → apéndice H-NN → Fase D del comando `/research`

---

## 1. Usuarios objetivo y flujos críticos

**Usuario:** dev AI/backend que corre agentes con VantaDB (embedded local, server HTTP remoto, proxy con upstream LLM, MCP spawn) y necesita inspeccionar/editar/debuggear memorias sin CLI.

| Flujo crítico | Estado hoy | Evidencia |
|---|---|---|
| Conectar a múltiples backends | ✅ Sólido: trait `VantaConnection`, perfiles persistentes (`store/connections.ts`), panel conexión | `desktop/src/components/ConnectionPanel.tsx`, DESKTOP-31 |
| Ingesta + edición CRUD | ✅ Grid con selección múltiple, Inspector con tabs (payload/metadata/vector/historial+diff), undo/papelera durable | `components/memory/MemoryLens.tsx`, `store/undo.ts`, DESKTOP-30/33 |
| Search híbrida + explicabilidad | ✅✅ Diferenciador: lente RETRIEVAL con barras BM25/HNSW/RRF vía `explain`, slider keyword/hybrid/vector paridad por construcción | `lens/retrieval/`, DESKTOP-35 |
| Exploración espacial/gráfica | ✅✅ UMAP scatterplot con lasso→batch ops, grafo R3F force-directed | `space/`, `graph/`, ESPACIO-01/02, GRAFO-02 |
| IQL interactivo | ✅ Consola CodeMirror + autocompletado core-side + highlight | `graph/IqlConsole.tsx`, VS-CORE-06 |
| Consolidar duplicados | ✅ Merge campo a campo + batch | `consolidate/`, DESKTOP-33 |
| Proxy dashboard | ⚠️ Implementado (DESKTOP-38) pero **sin validación manual con upstream LLM vivo** — riesgo de bugs latentes en el flujo completo | `components/proxy/ProxyDashboard.tsx` |
| Instalar/distribuir | ⚠️ NSIS/MSI construidos y sidecar incluido, pero **unsigned** y **sin smoke-test en VM limpia** (DESKTOP-24 deuda) | `src-tauri/tauri.conf.json:35-52` |

## 2. Estándares del ecosistema

- **Seguridad Tauri:** los docs oficiales establecen que Tauri restringe la CSP para reducir XSS y que configurarla en `security.csp` es parte del modelo defensivo — `csp: null` la desactiva explícitamente. Fuente: https://v2.tauri.app/security/csp/ · https://v2.tauri.app/security/
- **Distribución desktop:** firmar instaladores y auto-actualizar son expectativas base en GUIs de bases de datos comerciales (Compass/RedisInsight/TablePlus actualizan solos; todos firman binarios).
- **Multi-plataforma:** TablePlus, DBeaver, DataGrip, Compass y RedisInsight corren en Windows/macOS/Linux. Vanta Studio solo compila targets NSIS/MSI (`tauri.conf.json:37`).
- **A11y:** WCAG 2.2 AA aplica igual en WebView que en web; el repo ya lo trata como normativa (DESIGN_DECISIONS §2, tabla contraste).

## 3. Productos de referencia — matriz mínima

*Fuentes: páginas oficiales extraídas en vivo (Compass, RedisInsight). TablePlus/DBeaver/DataGrip/pgAdmin: conocimiento general del ecosistema, marcado como tal (los motores de búsqueda estuvieron degradados durante esta sesión — ver Método).*

| Dimensión | **Vanta Studio** | MongoDB Compass | RedisInsight | TablePlus/DBeaver/DataGrip | Qdrant Web UI / Zilliz console |
|---|---|---|---|---|---|
| Multi-conexión persistente | ✅ perfiles embedded/server/proxy/MCP | ✅ clusters/workspaces | ✅ múltiples DBs | ✅ (core de la categoría) | ❌ una consola por deploy |
| CRUD/batch + undo | ✅ papelera durable + Ctrl+Z batch | ✅ CRUD (undo limitado) | ✅ full CRUD/batch | ✅ | parcial |
| Explicabilidad de búsqueda | ✅✅ score breakdown BM25/HNSW/RRF (único en categoría vector-local) | Explain Plan para queries | profiling de comandos | explain plans SQL | métricas básicas |
| Visualización espacial | ✅✅ UMAP + lasso→batch ops | schema analysis (no embedding space) | ❌ | ❌ | ❌ |
| Grafo | ✅ R3F force-directed | ❌ | ❌ | ❌ | ❌ |
| Query console con autocompletado | ✅ IQL + CM6 | query bar + NL querying | Workbench Monaco + Copilot | SQL editor maduro | ❌ |
| Asistente IA | ❌ | ✅ NL→query (Atlas) | ✅ Copilot | ❌ | parcial |
| Temas light/dark AA | ✅ auditados (FIND-24, DAUD-01 E2E) | ✅ dark mode | ✅ | ✅ (DataGrip temas) | varía |
| Auto-update | ❌ | ✅ | ✅ | ✅ | n/a (web) |
| Firmado de binarios | ❌ unsigned | ✅ | ✅ | ✅ | n/a |
| Plataformas | Windows only (NSIS/MSI) | Win/mac/Linux | Win/mac/Linux/web/docker/K8s | Win/mac/Linux | web |
| Peso instalador | ✅ ~10MB (NSIS 9.9MB) — ventaja estructural vs Electron (~80-200MB) | ~100MB+ (Electron) | ~100MB+ | varía | n/a |

**Qué copiarían abiertamente:** auto-update silencioso (Compass/RedisInsight), onboarding embebido tipo tutorials de RedisInsight, NL→IQL estilo Copilot (apuesta posterior), instaladores multiplataforma.

## 4. Estado actual interno (evidencia)

- **Superficies (8):** HOME, MEMORIAS, BÚSQUEDA, GRAFO, ESPACIO, CONSOLIDAR, ÍNDICES, MEMORIA (vanta-memory), PAPELERA, PROXY, AJUSTES — shell único `WorkspaceShell` + paleta cmdk + tooltips + HelpPanel.
- **Design system:** manga/linocut tokens en `index.css` con tabla de contraste normativa (`DESIGN_DECISIONS.md` §2); LensShell compartido en las 6 lentes (UX-01).
- **Tests:** vitest 68/68, cargo src-tauri ~85, E2E Playwright **2 specs** (`e2e/daud01-temas.spec.ts`, `e2e/flujo-critico.spec.ts`) + selfcheck web contra binario real. Gaps E2E: cambio multi-perfil, proxy, graph/space.
- **Auditorías previas:** DAUD-01..11 (diseño post-fix) — commits `3c53d8b2`, `480935a7`, `b865c625`, `ae03cc7d` ya aplicaron D1-D11+FIND-23, aunque varias filas del Backlog siguen ⬜ (stale — H-13). UX-A11Y/UX-POLISH commiteados (`e24cc655`, `9cd202aa`).
- **Deudas conocidas referenciadas (no redescubiertas):** sparse_vector en rename (DESKTOP-32), F1/F2 sin handler (DESKTOP-34), i18n real (DESKTOP-31), smoke VM + firma (DESKTOP-24), proxy validación (DESKTOP-38).
- **Performance:** sin mediciones propias del app (startup, RAM idle). El dato "Tauri ~50MB vs Electron ~120MB+" viene del research doc DESKTOP-01, no de medición de Vanta Studio — Regla 11: tratarlo como estimación de plataforma, no como claim del producto.

## 5. Framework de evaluación (score 0-10)

| Dimensión | Score | Justificación |
|---|---|---|
| First impression & messaging | 7.5 | Splash + MARK + brand consistente; sin capturas runtime frescas en esta sesión (visual smoke DAUD-01 cubre temas) |
| Flujos core completos | 9 | CRUD→search→explicación→batch→undo sin salir del Studio; proxy sin validar end-to-end |
| Accesibilidad (WCAG 2.2 AA) | 8.5 | Contraste calibrado normativo, encoding redundante, keyboard grid nav, focus trap (commits UX-A11Y) |
| Performance | 6.0 | Sin baseline medido del app (startup/RAM); polling coordinado a 4s bien resuelto (DESKTOP-29) |
| i18n | 4.0 | Todo ES (unificado a mano), selector de idioma en AJUSTES sin backing real |
| Consistencia design system | 9 | Tokens canónicos documentados + LensShell + convención de iconos registrada |
| Robustez | 8 | Undo/papelera durable, estados vacíos honestos, errores tipados; rename ns pierde sparse_vector |
| Seguridad | 5.5 | Capabilities mínimas ✅, auth Bearer server ✅ — pero **CSP null** (contradice docs oficiales) e instaladores unsigned |
| Testabilidad | 7 | Vitest sólido + 2 E2E; cobertura E2E de superficies avanzadas vacía |
| **Diferenciación** | **9.5** | Única GUI local que combina multi-transporte VantaDB + explicabilidad RRF + espacio UMAP accionable + grafo + memoria de agente; Compass/RedisInsight no tienen nada equivalente para vector-local |

**Score global: 7.4/10**

---

## Gap analysis priorizado

**Falta (P0):** CSP, validación proxy end-to-end.
**Falta (P1):** firma/smoke instalador, auto-update, i18n real, E2E de superficies avanzadas.
**Falta (P2):** macOS/Linux, NL→IQL, onboarding embebido.
**Mejorable:** rename ns sparse_vector, F1/F2, palette desync, statusReport EN, filas Backlog stale.
**Optimizable:** baseline medido de recursos/startup (Regla 11), release-plz sync versión desktop.

**Quick wins (<1 día):** H-01, H-02, H-03, H-04, H-05, H-13, H-15.
**Apuestas estratégicas (>1 semana):** H-06 (i18n), H-08 (signing), H-09 (cross-platform), H-10 (updater).

## Apéndice de hallazgos H-NN (entrada Fase D)

| ID | Categoría | Severidad | Esfuerzo | Hallazgo | Evidencia |
|---|---|---|---|---|---|
| H-01 | APLICAR | 🔴 Alta | 🟢 <2h | `security.csp: null` desactiva la protección XSS que Tauri documenta como default recomendado. Definir CSP mínima (`default-src 'self'`; conectar a `http://127.0.0.1:*`/remoto según necesidad) | `desktop/src-tauri/tauri.conf.json:24-26` · https://v2.tauri.app/security/csp/ |
| H-02 | APLICAR | 🟢 Baja | 🟢 <1h | CommandPalette desincronizada con surfaces del shell (faltaba `memoria`, anotada en DESKTOP-38; verificar union completa tras AJUSTES/PROXY) | `components/palette/CommandPalette.tsx` · `docs/avance/activo/desktop.md` DESKTOP-38 |
| H-03 | APLICAR | 🟢 Baja | 🟢 <1h | Ayuda F1/F2 anunciada en tooltips pero sin handler keydown global | Deuda DESKTOP-34 · `layout/HelpPanel.tsx`, `WorkspaceShell.tsx` |
| H-04 | APLICAR | 🟠 Media | 🟡 2-4h | Rename de namespace copia registros sin `sparse_vector` — mutación silenciosa de datos en el caso híbrido | Deuda DESKTOP-32 · `vanta.ts` rename flow |
| H-05 | MEJORAR | 🟢 | 🟢 | `statusReport.ts` genera reporte markdown EN mientras toda la UI es ES | `components/export/statusReport.ts` |
| H-06 | AGREGAR | 🟡 Media | 🔴 >1sem | i18n real: selector idioma en AJUSTES sin framework ni catálogos (web ya tiene tt() ES/EN — inconsistencia entre superficies del mismo producto) | Deuda DESKTOP-31 · `pages/Settings.tsx` |
| H-07 | MEJORAR | 🟡 | 🟡 | E2E desktop cubre solo temas + flujo crítico; sin specs de cambio multi-perfil, proxy dashboard, graph/space lenses | `desktop/e2e/` (2 specs) |
| H-08 | ESTRATEGIA | 🟠 | 🔴 | Distribución: instaladores unsigned (SmartScreen avisa) + smoke-test en VM limpia nunca ejecutado (Step 3 DESKTOP-24). Smoke = APLICAR 🟢; firma de código requiere certificado (decisión negocio) | `docs/avance/activo/desktop.md` DESKTOP-24 |
| H-09 | ESTRATEGIA | 🟡 | 🔴 | Solo targets NSIS/MSI; toda la competencia es cross-platform. macOS (icns ya existe en bundle.icon) y Linux (AppImage/deb) requieren CI + testing | `src-tauri/tauri.conf.json:37-44` |
| H-10 | ESTRATEGIA | 🟡 | 🟡 | Sin auto-update (`tauri-plugin-updater` no configurado); expectativa base de la categoría | `tauri.conf.json` plugins (solo deep-link) |
| H-11 | MEJORAR | 🟢 | 🟡 | Versión desktop `0.1.0` hardcodeada en `package.json` + `tauri.conf.json`, fuera del versionado release-plz del workspace | `desktop/package.json:4` · `tauri.conf.json:4` |
| H-12 | APLICAR | 🟡 | 🟢 | Proxy Dashboard sin validación manual con upstream LLM vivo (TurnReports/sesiones/write-back/rate-limit solo testeados unitariamente) | Deuda DESKTOP-38 |
| H-13 | MEJORAR | 🟢 | 🟢 | Filas DAUD-01..09 en Backlog siguen ⬜ Pendiente pese a commits aplicados (stale — Trigger 4/1 de progreso); limpiar o actualizar | `docs/Backlog.md:522-530` vs git log `3c53d8b2`,`480935a7`,`b865c625` |
| H-14 | DECISIÓN | 🟢 | 🟢 | Semántica del botón FILTROS: hoy sigue al panel abierto, alternativa es reglas activas >0 — decisión de diseño del owner pendiente (DAUD-02) | `WorkspaceShell.tsx` topbar |
| H-15 | OPTIMIZAR | 🟢 | 🟡 | Sin baseline medido del app (startup time, RAM idle, tamaño heap WebView) — cualquier claim de performance actual es estimación de plataforma, no medición (Regla 11) | Ausencia en `docs/operations/BENCHMARKS.md` |

**Fortalezas a preservar (no tocar):** undo/papelera durable, lente RETRIEVAL, espacio UMAP con lasso→batch, peso de instalador ~10MB, tabla de contraste normativa en index.css.

## Método y fuentes

- **Interno:** codegraph_explore (shell/e2e/bridge), glob inventario `desktop/src/**`, `docs/avance/activo/desktop.md`, `docs/Backlog.md` (P37/DAUD), `package.json`, `tauri.conf.json`, `DESIGN_DECISIONS.md`, campaign_memory lessons, git log desktop.
- **Web (cascade):** agent-search keyless (duckduckgo/sogou bot-challenge) → Argus/yahoo OK para Tauri CSP (docs oficiales v2.tauri.app) → extracción directa Jina de mongodb.com/products/tools/compass y redis.io/insight. Búsquedas comparativas multi-engine fallidas (bing parse_error) → análisis de TablePlus/DBeaver/DataGrip marcado como conocimiento general del ecosistema, sin claims numéricos.
- **No ejecutado:** capturas playwright-cli contra `tauri dev` (levantar el app Tauri excede el alcance read-only de esta sesión; el E2E existente daud01-temas ya cubre evidencia visual de temas).
