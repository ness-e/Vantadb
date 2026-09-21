# FIND-11: Rutas alternativas sin pulir — desktop README + lazy-load wasm + npm naming

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-desktop-ux-core.md
- **Creado:** 2026-08-25T12:00
- **last-synced:** 2026-08-25T12:00
- **Estado:** ✅ COMPLETED (sync 2026-08-25 — steps todos ✅; stale cleanup por FIND-23)
- **Tipo:** documentation (vanta-docs) — NO código
- **Esfuerzo:** 🟢

## Blast Radius
| Callers | Callees | Implicaciones |
|---|---|---|
| `desktop/README.md` (nuevo) — sin callers | refs a `docs/desktop/*`, ADR-026/027/028, package.json scripts, src-tauri | File nuevo, cero riesgo |
| `vantadb-ts/README.md` — referenciado por `docs/api/BINDINGS_NAMESPACES.md:7` (anchor `#domain-sub-clients`), `vantadb-ts/package.json` files (npm), progreso skill (tabla Trigger 1.A) | refs a `docs/QUICKSTART.md`, `docs/api/*`, `../vantadb-node` | Adición de secciones — anchors existentes intactos |

## Contrato
- `desktop/README.md` creado (instalación + desarrollo) — nota instalador público pendiente
- Nota lazy-load bundle .wasm 1.3MB en `vantadb-ts/README.md` (cómo se carga + recomendación SSR/hooks)
- Aclaración `vantadb-node` vs `vantadb` en npm (qué es cada paquete)
- `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps, exit 0

## Herramientas
- Read/Write/Edit directos (CodeGraph auto-sync deshabilitado — plan file nota)
- pwsh para verificación

## Skills (SDP)
- Base: campaign-executor, progreso, ponytail, source-driven-development (de campaign_load_skills)
- Extra: documentation-and-adrs (README structure, no duplicar ADR), writing-guidelines (voz/tono docs técnicas EN)
- SDP: sin candidatas adicionales — tarea docs pura, sin diseño/SEO/video

## Steps

### Step 1: DISCOVERY — verificar estado real desktop + wasm + npm
- **Archivos:** `desktop/package.json`, `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/src/lib.rs`, `desktop/src-tauri/src/connections/mod.rs`, `desktop/vite.config.ts`, `desktop/DESIGN_DECISIONS.md`, `vantadb-ts/README.md`, `vantadb-ts/src/native.ts`, `vantadb-ts/src/vantadb.ts`, `docs/desktop/README.md`, `docs/architecture/adr/ADR-030-brand-identity-naming-convention.md`, `scripts/validate-docs-coverage.ps1`
- **Acción:** leídos completos (arriba). Hechos verificados: scripts reales (dev/build/build:wasm/preview/test/tauri), Tauri v2 (devUrl :1420, beforeDevCommand npm run dev, bundles nsis/msi), transporte triple (native embedded fjall / HTTP server / WASM-OPFS), lazy-load real (vite.config externaliza glue wasm en modos no-wasm → import nunca ejecuta; `--mode wasm` usa vite-plugin-wasm fetch+instantiate; `vantadb-node` se carga lazy vía `import("vantadb-node")` native.ts:113), bundle .wasm = 1,364,555 bytes (1.30 MB) en `vantadb-wasm/pkg/vantadb_wasm_bg.wasm`, npm `vantadb` publicado 0.5.0 / `vantadb-node` 404 nunca publicado (ADR-030:10, verificado FIND-17).
- **Verify:** `Test-Path desktop/README.md` = False (no existe aún); `Test-Path vantadb-ts/README.md` = True
- **Estado:** ✅ COMPLETED

### Step 2: Crear `desktop/README.md`
- **Archivos:** `desktop/README.md` (nuevo)
- **Acción:** Vanta Studio — consola human-facing desktop. Qué es, requisitos (Node ≥20, Rust ≥1.94.1, prereqs Tauri v2), instalación dev (`cd desktop && npm install`), ejecución (`npm run tauri dev` nativo / `npm run dev` HTTP / `npm run build:wasm` standalone), tabla transporte (native embebida / HTTP / WASM-OPFS), scripts package.json, advertencia instalador público pendiente (bundles nsis/msi configurados, sin canal público), links a `docs/desktop/README.md`/ARCHITECTURE/GUIDE + ADRs. Inglés.
- **Verify:** `Test-Path desktop/README.md` = True; grep "tauri dev" y "public installer" y "install" presentes
- **Estado:** ✅ COMPLETED

### Step 3: Nota lazy-load wasm + aclaración npm en `vantadb-ts/README.md`
- **Archivos:** `vantadb-ts/README.md` (edición aditiva)
- **Acción:** sección "WASM bundle & lazy loading": .wasm 1.3MB (`vantadb-wasm/pkg/vantadb_wasm_bg.wasm`), cómo se carga (glue wasm-bindgen estático vía `vantadb-wasm`; bundlers requieren vite-plugin-wasm; desktop/web code-split externaliza el glue → nunca se descarga en modos Tauri/HTTP; `--mode wasm` lo bundlea fetch+instantiate), recomendación SSR/hooks (no instanciar engine en server; crear client en cliente/useEffect; en Node usar `NativeVantaDB` lazy). Sección "vantadb vs vantadb-node (npm)": `vantadb` = SDK TS WASM publicado 0.5.0 (browser/Node/Bun/Deno, ESM-only); `vantadb-node` = bindings nativos napi-rs, NO publicado en npm (404), FS real fjall/WAL, async, carga lazy. Referenciar ADR-030.
- **Verify:** grep "1.3" y "lazy" y "vantadb-node" en README = matches; anchors existentes intactos
- **Estado:** ✅ COMPLETED

### Step 4: Verificación del contrato
- **Acción:** `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps exit 0; revisión final (EN, sin claims numéricos inventados, links relativos correctos)
- **Verify:** exit 0 + output "0 gaps"
- **Estado:** ✅ COMPLETED

## Dependencias
- FIND-17 (ADR-030 convención nombres — ya COMMITTED): base de la aclaración npm
- UX-A11Y (Task 1, mismo plan): toca desktop/ pero NO desktop/README.md — sin conflicto

## Notas
- NO COMMIT — el lead verifica mecánico y commitea por tarea (regla del plan).
- CodeGraph auto-sync deshabilitado → todo por lectura directa (plan file nota).
- `docs/desktop/README.md` YA existe y cubre instalación/transportes a nivel docs/ — el entregable `desktop/README.md` es la raíz de la app (nuevo), con el detalle dev/scripts/instalador; no duplicar el contenido de docs/ sino enlazarlo.
- Regla 11 (AI Guardian): el tamaño del bundle es un número medido (1,364,555 bytes verificado con Get-ChildItem) — citar el path, no adjetivos.

## Context Save Point
- **Fecha:** 2026-08-25T12:30
- **Branch:** develop
- **CI pendiente:** sí (lead: verify + commit)
- **Decisiones:** lazy-load nota en vantadb-ts/README.md (no docs/) porque es el README npm-facing; npm clarification misma ubicación (ADR-030 como fuente); desktop/README.md enlaza docs/desktop/ sin duplicar
- **Problemas conocidos:** WIP guard del server bloqueó el claim in-progress (TIR-08 corría en paralelo) — trabajo registrado en task file igual; resuelto al cierre vía update completed exitoso
- **Próxima tarea:** Lead: verify + commit FIND-11 (solo desktop/README.md + vantadb-ts/README.md + FIND-11.md); luego Wave 2 (DAUD-limpi / E2E+visual)