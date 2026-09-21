# DESKTOP-25: CI GitHub Actions (desktop) — Build Windows, test, artefacto instalador

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETO (mecánico — ejecución real del workflow queda como deuda para el próximo push)

## Impacto mapeado (Regla 0)
- **Leídos completos:** `.github/actions/rust-setup/action.yml`, `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/Cargo.toml`, `desktop/package.json`, `desktop/src-tauri/tests/child_process.rs`, `.opencode/rules/release-ci.md`, `DESKTOP-24.md` (Context Save Point).
- **Referencias entrantes:** ninguna (archivo nuevo); trigger por paths `desktop/**`, `src/**`, `vantadb-server/**`.
- **Referencias salientes:** consume `.github/actions/rust-setup` (composite, sccache único punto por release-ci.md regla 2), `tauri-action@v1` con `projectPath: desktop`, sidecar gitignored `binaries/vantadb-server.exe` construido desde workspace root.
- **Veredicto:** sin colisiones — workflows existentes (ci-gate, release-binaries-63, etc.) no se tocan.

## Blast Radius
Callers: .github/workflows/desktop.yml (nuevo)
Callees: tauri-action, cargo, npm
Implicaciones: Pipeline CI que valida el desktop app en cada PR/push

## Spec
N/A — CI pipeline con contrato mecánico

## Contrato
`gh workflow run desktop.yml` o push a main/develop → pipeline verde; artefacto instalador subido

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear workflow desktop.yml ✅
- **Archivos:** `.github/workflows/desktop.yml` (nuevo)
- **Acción:** Workflow que: (1) checkout, (2) setup Node + Rust (stable), (3) `cd desktop && npm ci`, (4) `cargo test` en `src-tauri` (workspace desacoplado), (5) build sidecar + copy a `binaries/` (contrato DESKTOP-24: el exe es gitignored, CI debe construirlo — `cargo build --release -p vantadb-server --features custom-allocator` per release-ci.md regla 1), (6) tauri-action, (7) upload artefacto
- **Verify:** actionlint 0 errores; ejecución real → deuda de push
- **Resultado:** creado con triggers push/PR (paths-filtered) + workflow_dispatch, concurrency cancel-in-progress, permissions mínimos. `cargo test` corre en `desktop/src-tauri`; `VANTADB_SERVER_BIN` apuntando al sidecar release para que los tests de spawn del sidecar se ejerzan de verdad (si no, self-skip).

### Step 2: Configurar tauri-action para Windows ✅
- **Archivos:** `.github/workflows/desktop.yml`
- **Acción:** `tauri-apps/tauri-action@v1` (**desviación del spec:** el task decía @v0; el README oficial actual usa @v1 — validado contra https://github.com/tauri-apps/tauri-action). `projectPath: desktop` (app no está en root del repo), `args: --target x86_64-pc-windows-msvc`. Cache cargo+sccache vía composite rust-setup; cache npm vía setup-node (`cache-dependency-path: desktop/package-lock.json`). Build-only (sin tagName/releaseId/releaseName → no crea GitHub Release).
- **Verify:** Workflow completa en <15 min → requiere push real
- **Resultado:** configurado. Nota: NO hay step separado de `npm run build` frontend — `beforeBuildCommand` de tauri.conf.json ya lo ejecuta dentro del build de Tauri (evita duplicar ~1 min por run).

### Step 3: Verificar artefacto instalador subido ✅ (mecánico)
- **Archivos:** `.github/workflows/desktop.yml`
- **Acción:** `actions/upload-artifact@v4` con `path: ${{ steps.tauri.outputs.artifactPaths }}` (output multi-línea oficial del action) + `if-no-files-found: error`
- **Verify:** Artefacto descargable e instalable → requiere push real
- **Resultado:** step mecánico completo; verificación end-to-end (descarga + instalación) es deuda documentada.

## Dependencias
- DESKTOP-24 (empaquetado debe funcionar primero)

## Notas
- DoD: pipeline verde; artefacto instalador subido
- Workspace desacoplado: `desktop/src-tauri/Cargo.toml` usa `[workspace]` vacío
- Priorizado 2026-08-20

## Context Save Point (2026-08-24)
- **Verificación mecánica:** `actionlint .github/workflows/desktop.yml` → exit 0, sin warnings.
- **Deuda (para el próximo push a develop):** (1) confirmar run verde <15 min; (2) descargar artefacto desde Actions e instalar en Windows limpio; (3) si el MSI falla por el sidecar, revisar que `bundle.resources` staging incluya `binaries/vantadb-server.exe`.
- **Decisiones:** sidecar con `--features custom-allocator` (mimalloc, release-ci.md regla 1 — es binario distribuible dentro del instalador); tauri-action@v1 no @v0; sin step npm build duplicado; sin GitHub Release (solo workflow artifact).
- NO commitear (instrucción del orquestador) — commit lo prepara vanta-lead.