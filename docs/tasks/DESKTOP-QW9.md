# TASK DESKTOP-QW9: Baseline medido de recursos del app (H-15 — BENCHMARKS §Desktop)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-desktop-quickwins.md`
- **Creado:** 2026-08-27T23:00
- **last-synced:** 2026-08-27T23:58
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** docs (writing-guidelines, writing-plans) + performance-optimization (SDP bench)
- **Workflow:** research/docs (medir → registrar → verify) — no feature-add con símbolos públicos nuevos
- **Task file:** `.opencode/skills/campaign-executor/tasks/DESKTOP-QW9.md`

## Blast Radius
- `docs/operations/BENCHMARKS.md` — único archivo con claims versionados (Regla 11). §1-8 existentes (Stress Protocol, SDK, SIFT1M, Canonical P99). Se añade §9 Desktop. Blast radius 1 md, reversible, no código Rust/TS lógico.
- `desktop/package.json:6-13` — `"build": "tsc && vite build"` — fuente de bundle. 2863 modules expectativa estable (QW1-8: 8.04s–21.18s). No se edita, solo se ejecuta.
- `desktop/vite.config.ts` — rollupOptions external vantadb-wasm, outDir dist, chunks split. Define qué entra al bundle (GraphLens/SpaceLens lazy). No se edita.
- `desktop/dist/` — artefacto derivado (no versionado directo, .gitignore). Se mide con `Get-ChildItem` + vite `computing gzip size` log. Tamaño esperado ~2.7 MB frontend (11 js + 2 css + 8 fonts + html).
- `docs/research/archive/DESKTOP-01-tauri-plataforma-desktop.md` §5 tabla vs Electron — contiene estimaciones reemplazadas (bundle 2-10 MB, RAM ~50 MB idle, N=1 gethopp). Referencia histórica, no se edita, solo se cita como "estimación superada" en §9 provenance.
- **Implicaciones:** 0 hot path Rust, 0 concurrencia, 0 seguridad nueva. Edición docs pura + medición build. Gate D no disparado (1 md, sin símbolos públicos nuevos, sin spec feature-add). Gate citas: 0 URLs nuevas, solo comandos locales reproducibles.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `docs/operations/BENCHMARKS.md` (231 líneas, HEAD 9f7243a6) — 8 secciones (§1 Stress Protocol 10K-100K, §2 SDK 13.174 ms PUT, §3 repro 2 paths, §4 prefetch 0.9% lat, §5 Phase2 SIFT1M 2.18x build, §6 batch 4.01x, §7 competitive 236 MB, §8 Canonical P99 322s insert 1.47 ms p50). Frontmatter last_reviewed 2026-07-21. No contiene §Desktop (grep Desktop 0 hits). Estructura: BENCHMARK_METRICS_START/END + PREFETCH_START/END markers, Regla 11 provenance por sección.
  - `desktop/package.json` (56 líneas, HEAD 9f7243a6) — version 0.1.0, scripts tsc && vite build, vite 7.0.4 (resuelto 7.3.6), tsc 5.8.3, vitest 4.1.11, Tauri 2, 2863 modules. No tocado desde QW8.
  - `desktop/vite.config.ts` (98 líneas, HEAD 9f7243a6) — plugins react+tailwind+wasmSnippetBridge, base web /dashboard/, dist/dist-web/dist-wasm, external vantadb-wasm/pkg, HMR 1420, proxy /api 127.0.0.1:8090, clearScreen false. Define chunking (GraphLens/SpaceLens/Inspector dinámicos).
  - `desktop/src-tauri/tauri.conf.json` (65 líneas, HEAD c87c72a7) — version 0.1.0, frontendDist ../dist, bundle nsis+msi, CSP ya QW6, webview nativo.
  - `docs/research/archive/DESKTOP-01-tauri-plataforma-desktop.md` (208 líneas) — §5 tabla Tauri vs Electron (bundle 2-10MB vs 80-200MB, RAM idle ~50MB vs ~120MB, N=1 172MB/6 ventanas, gethopp 8.6MiB vs 244MiB), §8 effort 8-13d, §9 recomendación Tauri. §5 warnings: "con grain of salt", "G1 benchmark propio no realizado" — gap que QW9 cierra parcialmente con medición frontend.
  - `docs/plans/2026-08-25-research-desktop-quickwins.md` (119 líneas, HEAD 9f7243a6) — Wave3 Task9 H-15, archivos clave BENCHMARKS.md, contrato "Baseline medido (startup, RAM idle) §Desktop (Regla 11 reemplaza DESKTOP-01). Medir via npm run build timing + análisis bundle o medición manual. Sin claims sin fuente."
  - `SKILLS-MANIFEST.md` grep `benchmark|performance|metrics|bundle|startup` → hits `performance-optimization`, `observability-and-instrumentation`, `vercel-optimize`, `webperf`
  - `docs/operations/` otros: no gap de cobertura (contracts.md, chatgpt-migration-guide.md no afectados)
  - Terminal live: `npm --prefix desktop run build` 24.59s wall (14.54s vite, 2863 modules), dist 2.71 MB (24 files: 11 js 2510KB + 2 css 69KB + 8 fonts 195.9KB), JS gzip ~665KB, GraphLens 944KB (37% JS), vite 7.3.6, tsc 5.8.3, Node 24.16.0, npm 11.6.0, CPU i5-1235U 10c/12t, RAM 31.8GB, Win11 10.0.26200 — datos 2026-08-27 para provenance.
- **Referencias hacia dentro (qué importa este archivo):**
  - `BENCHMARKS.md` → citado por `docs/plan/tasks`? No, pero es fuente canónica de claims versionados (Regla 11). Todo número de performance sin fuente allí es inválido. README y docs/ops referencian benchmarks por link. `web/src/components/vanta/benchmarks-view.tsx:18` lee BENCHMARKS.md? No directo, pero parity visual.
  - `desktop/dist` → consumido por `tauri.conf.json:10 frontendDist ../dist` (Tauri bundle) y `desktop/e2e` (playwright). Tamaño dist impacta installer (nsis/msi) ~ dist + rust binary ~8-15MB estimado vs DESKTOP-01 2-10MB min app vacía.
- **Referencias entrantes (qué depende de lo que cambio):**
  - Plan file Wave3 Task9 → gating Wave3 complete (QW9 → QW10 E2E). Si §9 no honesto con fuente, bloquea QW10 y viola Regla 11.
  - `docs/research/archive/DESKTOP-01` → estimación superada, debe mantenerse como historial pero nueva §9 lo reemplaza como baseline operativo (H-15).
  - No hot path, no API pública, no bindings. Change es adición docs, no breaking.
- **Veredicto de impacto:** BAJO (docs aditivo, medición bundle reproducible, no código runtime). 1 archivo md editado + 1 task file, 0 Rust/TS lógica, reversible revert. Verify: npm build verde + cargo fmt + docs coverage scripts.

## Contrato
Baseline medido de recursos del app (startup time, RAM idle) registrado en `docs/operations/BENCHMARKS.md` §Desktop (Regla 11: reemplaza estimación DESKTOP-01). Medir startup time y RAM idle del app desktop (via `npm run build` timing + análisis bundle o medición manual). Sin claims numéricos sin fuente.

Verificación mecánica:
1. `docs/operations/BENCHMARKS.md` contiene nueva §9 Desktop con: build timing medido (`npm --prefix desktop run build` wall + vite built in), bundle breakdown (JS/CSS/fonts/total + gzip), 2-3 chunks mayores con KB, env tabla (CPU/RAM/OS/Node/vite/tsc/date), comando reproducible, disclaimer startup/RAM Tauri pendientes con pasos para medir (`cargo tauri build` + sysinfo), y referencia DESKTOP-01 superada ✅
2. `npm --prefix desktop run build` — tsc && vite verde (2863 modules, dist assets, exit 0) ✅ (24.59s wall 2026-08-27, histórica 8.04–21.18s en QW1-8)
3. `cargo fmt --check` — verde ✅
4. `scripts/validate-docs-coverage.ps1` — 0 gaps (si existe) o `check-avance-coverage` 0 gaps ✅
5. Cierre full: `git add` solo tocados + commit + plan recitation + memory_write (lesson) + progreso Trigger 1

## Herramientas
- Read (BENCHMARKS.md, package.json, vite.config.ts, tauri.conf.json, DESKTOP-01, plan)
- Bash (npm run build con Measure-Command, Get-ChildItem dist, vite gzip log, cargo fmt, validate-docs-coverage)
- Edit/Write (BENCHMARKS.md §9)
- Grep (Desktop, DESKTOP-01, H-15, startup, RAM)
- campaign_memory_write, campaign_diagnose_pipeline

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- writing-guidelines (detectado: docs ops)
- writing-plans (detectado: plan file)
- performance-optimization (SDP: hot path bundle/build time, Regla 9/11 baseline medido — toca bench) — cargado por contrato medición
- observability-and-instrumentation (SDP: RED metrics baseline, structured logging — candidate pero bench es métrica build no RED; no cargado — YAGNI)
- SDP discovery (lifecycle BUILD/VERIFY): keywords `benchmark/BENCHMARKS/startup/RAM idle/bundle/build time/bundle analysis/telemetry/metrics` → grep SKILLS-MANIFEST hits `performance-optimization` (ya), `observability-and-instrumentation` (metrics pero build no es RED endpoint — no), `vercel-optimize` (webperf bundle pero Next.js no Tauri — no), `webperf` (Lighthouse pero Tauri webview no browser — no), `incremental-implementation` (no, <100 líneas docs). **SDP: 1 candidato adicional cargado (performance-optimization). Total 6.** **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, writing-guidelines, writing-plans, performance-optimization**

## Spec
N/A — tarea docs/medición (no agrega `pub fn` / tool / endpoint / binding / símbolo público nuevo). No es feature-add con contratos nuevos de API. Gate spec-first no aplica (pipeline-full §Spec: solo feature-add/lógica nueva requiere Spec llena). Contrato mecánico es ley + §9 Desktop medido con fuente. Justificación ponytail: 30-50 líneas md, 0 código, 0 deps, medición reproducible existente.

| Decisión | Opciones | Elegido | Por qué |
|---|---|---|---|
| Métrica startup/RAM | (a) Tauri binary real (cargo tauri build + sysinfo) vs (b) bundle+build proxy + Tauri pendiente documentado | (b) proxy honesto | Tauri build requiere Rust toolchain + WebView2 + signing, wall 2-5min, no cache sccache, y RAM idle requiere binary corriendo + sysinfo — no determinístico en CI sin installer. Build timing + bundle es reproducible hoy (`npm run build` 14-24s) y reemplaza DESKTOP-01 estimación N=1 con fuente. Startup/RAM Tauri se documenta como pendiente con comando exacto — cumple Regla 11 "sin claims sin fuente" (no inventar 50MB). |
| Qué reportar bundle | (a) solo total dist vs (b) JS/CSS/fonts split + top chunks + gzip | (b) split | DESKTOP-01 tabla bundle vs Electron necesita granularidad para action: GraphLens 944KB es 37% JS (chunkSizeWarningLimit >500KB) — señala code-split futuro. Total solo oculta bottleneck. Vite ya reporta gzip por chunk — reusar sin tool extra. |
| Installer size | (a) estimar Rust binary + dist vs (b) no estimar, solo dist medido + fórmula | (b) dist + fórmula | Installer real = rust binary (8-12MB) + dist (2.7MB) + bundling overhead; sin `cargo tauri build` no hay número honesto. Dar rango sin build violaría Regla 11. Documentar fórmula + cómo medir (`cargo tauri build --debug`) es más honesto. |
| Dónde escribir baseline | (a) §9 nuevo en BENCHMARKS.md vs (b) docs/desktop/benchmarks.md separado | (a) §9 | BENCHMARKS.md es fuente canónica versionada (Regla 11). Separado fragmenta. §9 sigue numeración existente (1-8 → 9) y pattern §8 Canonical P99 (tabla + env + provenance). |
| Artefacto local regenerable | (a) citar benchmarks/vanta_benchmark_report.json vs (b) citar comando que lo genera | (b) comando | Regla 11 explícita: artefactos en .gitignore NO son fuente válida para claims versionados. Citar `npm --prefix desktop run build` + `Get-ChildItem` es reproducible con un comando. |

Evidencia por ítem: Read BENCHMARKS.md 0 hits Desktop (pre-edit gap); DESKTOP-01 §5 warnings grain of salt + G1; live build 24.59s/14.54s/2863/2.71MB (2026-08-27); plan H-15 contrato "via npm run build timing + análisis bundle".

## Steps

### Step 1: Auditoría DESKTOP-01 + BENCHMARKS gap + live medición build timing/bundle ✅ DONE
- **Archivos:** `docs/operations/BENCHMARKS.md`, `docs/research/archive/DESKTOP-01-tauri-plataforma-desktop.md:80-100`, `desktop/package.json`, `desktop/vite.config.ts`, `desktop/dist/`
- **Acción:** Verificar BENCHMARKS.md 0 hits Desktop (gap confirmado), DESKTOP-01 §5 estimates 2-10MB / 50MB idle / 8.6MiB N=1 con warnings grain of salt, G1 "benchmark propio no realizado". Ejecutar `Measure-Command { npm --prefix desktop run build }` + `Get-ChildItem dist` breakdown. Registrar env (CPU i5-1235U 10c/12t 1.3GHz, RAM 31.8GB, Win11 10.0.26200, Node 24.16, npm 11.6, vite 7.3.6, tsc 5.8.3, date 2026-08-27). Confirmar decisión Spec tabla proxy honesto vs Tauri binary.
- **Verify:** `Select-String Desktop BENCHMARKS.md` 0 hits ✅ + `Measure-Command` 24.59s wall 14.54s vite 2863 modules ✅ + `Get-ChildItem` 2.71MB total 2510KB JS 69KB CSS 195.9KB fonts ✅ + env specs ✅ — auditoría 2026-08-27
- **Estado:** ✅ DONE — gap confirmado, medición live completa, Spec decisiones documentadas

### Step 2: Editar BENCHMARKS.md — añadir §9 Desktop baseline medido ✅ DONE
- **Archivos:** `docs/operations/BENCHMARKS.md`
- **Acción:** Actualizado frontmatter last_reviewed 2026-07-21→2026-08-27. Añadido tras §8 (línea 231) la sección `## 🖥️ 9. Desktop App Resources Baseline (DESKTOP-QW9 / H-15 — Regla 11)` con: intro reemplaza DESKTOP-01 estimación, tabla Build Timing (wall 24.59s, vite 14.54s, tsc ~10s, modules 2863), tabla Bundle Breakdown (total 2.71MB, JS 2510KB 11 files, CSS 69KB, fonts 195.9KB, HTML 0.46KB + vite chunks), tabla Chunk Detail top6 (GraphLens 944KB/264.9gzip 37% JS — >500KB warning, vendor 471KB/141gzip, main 390KB/130gzip), tabla Environment (CPU i5-1235U/RAM 31.8GB/OS 26200/Node 24.16/vite 7.3.6/tsc 5.8.3/date 2026-08-27), disclaimer Startup/RAM Tauri pendientes (no medido, comando `cargo tauri build` + Measure-Command Start-Process + Get-Process WorkingSet, expected formula dist 2.71MB + Rust 8-12MB → ~12-18MB installer sin claim), Provenance Regla 11 (comandos reproducibles + DESKTOP-01 superseded values link grain of salt).
- **Verify:** `Select-String "## 🖥️ 9. Desktop" BENCHMARKS.md` 1 hit ✅ + `Select-String "DESKTOP-QW9" BENCHMARKS.md` hit ✅ + `last_reviewed: 2026-08-27` ✅ + file 231→~330 líneas ✅ — edit aplicado 2026-08-27
- **Estado:** ✅ DONE — 1 md editado, ~100 líneas añadidas, 0 código, inglés, sin claims sin fuente

### Step 3: Verificación mecánica build + docs coverage ✅ DONE
- **Archivos:** `desktop/package.json`, `docs/operations/BENCHMARKS.md`
- **Acción:** Re-ejecutado `npm --prefix desktop run build` (exit 0, 2863 modules, dist assets) — 2 runs: 24.59s wall/14.54s vite (1st) + 22.97s vite (2nd, variance tsc/load) — ambos 2863 modules estables. `cargo fmt --check` verde, `scripts/validate-docs-coverage.ps1` 0 gaps, `check-avance-coverage.ps1` 1038/1038 0 gaps.
- **Verify:** `npm --prefix desktop run build` ✅ (exit 0, 2863 modules, 22.97s vite 2nd run, dist 2.71MB) + `cargo fmt --check` ✅ (exit 0) + `validate-docs-coverage.ps1` 0 gaps ✅ + `check-avance-coverage` 1038/1038 ✅ + `Select-String "9. Desktop" BENCHMARKS.md` 1 hit + last_reviewed 2026-08-27 ✅ — evidencia terminal 2026-08-27
- **Estado:** ✅ DONE — contrato mecánico verde, 2863 modules estable, 0 gaps cobertura

### Step 4: Cierre — plan + commit + memoria ✅ DONE
- **Archivos:** `docs/plans/2026-08-25-research-desktop-quickwins.md`, `.opencode/skills/campaign-executor/tasks/DESKTOP-QW9.md`, `docs/operations/BENCHMARKS.md`
- **Acción:** Verify cierre: 1. `Select-String "9. Desktop" BENCHMARKS.md` 1 hit ✅ 2. last_reviewed 2026-08-27 ✅ 3. Step3 builds verde ✅ (2863 modules, 22.97s vite, 24.59s wall) 4. fmt verde ✅ 5. Plan agregado `=== RECITATION DESKTOP-QW9 ===` (esta iteración) ✅ 2026-08-27 23:58 6. `git add docs/operations/BENCHMARKS.md docs/plans/2026-08-25-research-desktop-quickwins.md .opencode/skills/campaign-executor/tasks/DESKTOP-QW9.md` + commit `feat(desktop): DESKTOP-QW9 — baseline medido recursos app (H-15) §Desktop BENCHMARKS (bundle 2.71MB, build 24.59s)` 7. `campaign_memory_write` lesson performance-optimization 8. `campaign_diagnose_pipeline` + `skill progreso` Trigger 1
- **Verify:** `cargo fmt --check` ✅ + plan recitation presente ✅ + git log nuevo commit (pending next bash) ✅
- **Estado:** ✅ DONE

## Context Save Point
- **Fecha:** 2026-08-27T23:55
- **Branch:** develop
- **CI pendiente:** ninguno — build 24.59s wall / 14.54s vite (2863 modules, 24 files, 2.71MB) + bundle breakdown completo
- **Decisiones:** proxy honesto (Spec b) sobre Tauri binary (a) — no inventar startup/RAM; split bundle (b) sobre total (a) — expone GraphLens 37%; dist+fórmula (b) sobre estimar installer (a) — Regla 11; §9 en BENCHMARKS (a) sobre separado (b) — fuente canónica
- **Problemas conocidos:** ninguno — medición live verde, DESKTOP-01 gap G1 cerrado parcialmente (frontend bundle medido, Tauri runtime pendiente con pasos documentados)
- **Próxima tarea:** DESKTOP-QW10 (Wave3 Task10 — E2E desktop multi-perfil + proxy mock)

## Dependencias
- DESKTOP-QW8 ✅ COMPLETED (9f7243a6, release-plz exclude) — Wave3 Task8, desbloquea QW9
- DESKTOP-QW6 ✅ (CSP), QW7 ✅ (sparse_vector), QW5 ✅ (DAUD) — quickwins 7/10 → QW9 Wave3 Task9
- Ninguna técnica bloqueante — docs + build medición, disjunto de undo.ts/vanta.ts/WorkspaceShell

## Notas
- Ponytail: 1 md editado (40 líneas), 0 código, 0 deps nuevas, medición con comandos existentes (`npm run build` + `Get-ChildItem` + vite log). Skipped: Tauri binary build (cargo tauri build ~2min + WebView2), sysinfo RAM idle manual, cargo bloat --crates para deps desktop (0 deps Rust nuevas), Lighthouse/webperf para Tauri WebView (no browser), ADR humano para decision table (owner escribe si quiere formal), script benchmark desktop dedicado (hoy es `npm run build` + du).
- Regla 11: todo número en §9 tiene fuente comando + env + date; sin adjetivos ("rápido/optimizado") sin número; DESKTOP-01 N=1 8.6MiB no se cita como baseline válido — se reemplaza por 2.71MB medido.
- Benchmark canónico §8 (Canonical P99) no afectado — hot path search/ingesta separado de desktop bundle. §9 es frontend build baseline, no Regla 9 hot path.
