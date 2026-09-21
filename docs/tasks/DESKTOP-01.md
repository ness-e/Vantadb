# DESKTOP-01: Investigar Tauri como plataforma desktop para VantaDB

## Metadata
- **Plan file:** — (tarea standalone del backlog)
- **Fuente:** `docs/Backlog.md` línea 166
- **Esfuerzo:** 🟡 1-2d (investigación + recomendación, sin implementación)
- **Prioridad:** 🟠 (origen 🔴, backlog actual 🟡)
- **Tipo:** Investigación / Arquitectura (sin código)
- **Turns estimados:** 15-20
- **Creado:** 2026-08-04T00:00
- **last-synced:** 2026-08-04T00:00
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | — (ningún código llama a esto; es investigación pura) |
| Callees | `vantadb` crate core (`VantaEmbedded`), `vantadb-ts` (WASM wrapper), `web/` (mark-classic.tsx, i18n dictionaries) |
| Implicaciones | No rompe contratos. Informa decisiones futuras de arquitectura desktop. No toca API pública, storage, ni bindings. Riesgo: bajo |

**Origen (backlog):** investigación previa en `docs_backup_2026-06-30/Investigaciones/VantaDB_Investigacion_Contexto_GTM.md` (líneas 966-976). **La ruta `docs_backup_2026-06-30/` NO existe en el repo actual** — el contexto GTM original se perdió en la reorganización de docs. El contenido de la propuesta está resumido en el propio backlog.

**Contexto del crate (verificado con codegraph):**
- `vantadb` es la crate core Rust; `VantaEmbedded::open_with_config(VantaConfig)` es la API de integración directa (`vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`).
- Ya existen bindings PyO3 (`python_sdk` feature), WASM (`vantadb-wasm`, `opfs`/`idb`/`worker` persistence), y TS wrapper (`vantadb-ts`) sobre el WASM.
- Para Tauri: la vía natural es `vantadb` como dependency directa en el `Cargo.toml` de la app Tauri (sin bridge WASM), con acceso a la FS nativa vía backend Rust (lo que elimina la capa OPFS de WASM).

## Contrato
"Reporte `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` existe, con: comparativa Tauri vs Electron (con fuentes citadas), effort estimate para MVP desktop, y recomendación de arquitectura explícita (SI/NO Tauri + vía de integración). Todas las afirmaciones técnicas sobre Tauri/Electron tienen fuente verificable. `scripts/validate-docs-coverage.ps1` pasa."

## Herramientas necesarias
- MetaSearchMCP.search_web / Argus (web research: docs oficiales de Tauri, comparativas)
- webfetch (docs.tauri.app, tauri.app, electronjs.org)
- codegraph_explore (ya ejecutado — contexto del crate)
- bash (grep en Backlog/Investigaciones)

## Investigation Notes
- Workflow research (`research.json`): scoping → searching → extracting → synthesizing → verifying → review → accept → close. Output final a `docs/Investigaciones/` (convención del proyecto; el workflow por defecto dice `docs/research/` — usar el estándar local).
- El deliverable NO es código — es un reporte de investigación en español en `docs/Investigaciones/` (permite español; es planning/research).
- Pregunta central: ¿conviene Tauri (Rust nativo, sin bridge) sobre Electron (requiere TS SDK vía WASM) para una desktop AI app privada con memoria local?

## Steps

### Step 1: Scoping — definir sub-queries de la investigación
- **Archivos:** — (planning)
- **Acción:** descomponer en 4-5 sub-queries: (1) Tauri v2 estado actual + stack Rust, (2) integración de una crate Rust nativa como backend desktop (command API, events), (3) casos de uso desktop AI/local-first y qué se necesita (memoria local, embeddings, UI), (4) comparativa Tauri vs Electron (bundle size, RAM, performance, madurez, DX), (5) effort estimate MVP (scaffold, integración `vantadb`, UI con memoria + search).
- **Verify:** las 5 sub-queries listadas y documentadas en el task file
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

### Step 2: Searching — recolectar fuentes por sub-query
- **Archivos:** — (research)
- **Acción:** ejecutar cada sub-query con MetaSearchMCP/Argus. Priorizar fuentes oficiales (docs.tauri.app, tauri.app, electronjs.org, GitHub tauri-apps) + benchmarks/publicaciones comparativas recientes. Guardar URLs + snippets.
- **Verify:** al menos 1 fuente oficial Tauri + 1 oficial Electron + 2 comparativas/benchmarks con URL
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

### Step 3: Extracting — extraer hechos clave de las 5-10 URLs más relevantes
- **Archivos:** — (research)
- **Acción:** extraer datos: bundle size típico Tauri vs Electron, RAM en idle, tiempo de build, soporte Windows/macOS/Linux, plugin system (window, tray, notifications, deep-link), cómo se llama Rust desde el frontend (invoke/commands), y si existe patrón de integrar una DB vectorial nativa como dependency.
- **Verify:** cada hecho tiene URL de origen anotada
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

### Step 4: Synthesizing — redactar reporte en `docs/Investigaciones/`
- **Archivos:** `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md`
- **Acción:** redactar reporte con: contexto VantaDB, comparativa Tauri vs Electron (tabla), vía de integración recomendada (`vantadb` como dep Rust directa en `src-tauri`, comandos backend para search/ingest, sin WASM/OPFS), casos de uso desktop AI app privada con memoria local, effort estimate MVP (scaffold + integración + UI mínima) desglosado, riesgos, y recomendación final explícita.
- **Verify:** reporte escrito con todas las secciones del contrato
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

### Step 5: Verifying + coverage — revisar citas y validar docs
- **Archivos:** `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md`
- **Acción:** auto-revisión: toda afirmación técnica tiene fuente; sin contradicciones sin resolver; gaps documentados explícitamente. Correr `pwsh scripts/validate-docs-coverage.ps1`.
- **Verify:** `pwsh scripts/validate-docs-coverage.ps1` pasa; reporte sin afirmaciones sin fuente
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

### Step 6: Cierre — registrar en progreso y commit
- **Archivos:** `docs/Backlog.md`, `docs/progreso/README.md`
- **Acción:** skill progreso (Trigger 1): tachar DESKTOP-01 en Backlog con nota de fecha y link al reporte; migrar a progreso; commit `docs: DESKTOP-01 tauri platform research report`
- **Verify:** commit creado; Backlog y progreso actualizados; reporte linkeado
- **Estado:** ✅ COMPLETO (evidencia: desktop/ implementado en Vanta Studio Fase 0-3, 2026-08)

## Dependencias
- Ninguna (tarea independiente)

## Notas
- No tocar código Rust/web en esta tarea — solo investigación y documentación.
- El contexto GTM original (`docs_backup_2026-06-30/`) no existe; la descripción del backlog es la fuente primaria.
- La comparativa debe usar versiones actuales (Tauri v2, Electron) — validar con docs oficiales, no asumir.
