# DESKTOP-24: Empaquetado NSIS/MSI (Windows primero) — instalador que conecta nativo + server sin dev env

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ⏳ IN PROGRESS

## Impacto mapeado (Regla 0)
- **Leídos completos:** `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/build.rs`, `desktop/src-tauri/src/connections/child_process.rs`, `desktop/package.json`, `desktop/src-tauri/icons/*`.
- **Referencias entrantes:** `build.rs` → `tauri_build::build()` consume `tauri.conf.json`; `child_process.rs::locate_binary()` busca `vantadb-server.exe` junto al exe instalado; CI futuro DESKTOP-25 (`desktop.yml`) consumirá este config.
- **Referencias salientes:** bundler NSIS/MSI (targets), WebView2 bootstrapper, `resources` → instala sidecar en raíz del install dir.
- **Decisión sidecar (Step 2):** NO usar `externalBin` — Tauri lo instala con sufijo target-triple (`vantadb-server-x86_64-pc-windows-msvc.exe`), que no matchea `locate_binary()` (busca `vantadb-server.exe` plano) y exigiría tocar Rust para computar el triple en runtime. Se usa `bundle.resources` con rename a raíz del install dir: cero cambios en código Rust. Fuente: https://v2.tauri.app/develop/sidecar/
- **Bug hallazgo inline:** `child_process.rs:47` — env override escrito `VANTVADB_SERVER_BIN` (typo de `VANTADB_SERVER_BIN`); nunca matchea. Fix 1-char inline (dentro del blast radius).
- **Contrato real:** script npm es `tauri` → comando correcto: `cd desktop && npm run tauri build`.

## Blast Radius
Callers: tauri.conf.json, src-tauri/build.rs, GitHub Actions desktop.yml
Callees: Tauri bundler, NSIS/MSI tooling
Implicaciones: Genera instalador Windows que incluye binarios Rust + frontend + config por defecto

## Spec
N/A — feature de empaquetado con contrato mecánico

## Contrato
`cd desktop && npm run tauri:build` produce instalador `.msi` / `.exe` en `src-tauri/target/release/bundle/` que instala y ejecuta sin entorno de dev

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Configurar tauri.conf.json para bundling Windows ✅
- **Archivos:** `desktop/src-tauri/tauri.conf.json`
- **Acción:** Configurar `bundle.targets: ["nsis", "msi"]`, `identifier`, `icon`, `windows.webviewInstallMode: {type: "embedBootstrapper"}` para WebView2 offline
- **Verify:** `cd desktop && npm run tauri:build -- --target x86_64-pc-windows-msvc` (dry-run local)
- **Resultado:** targets `["nsis","msi"]` + `webviewInstallMode embedBootstrapper` configurados (`identifier` e `icon` ya existían). Verify real: `npm run tauri build` completo → ambos bundles generados.

### Step 2: Configurar build.rs para externalBin si aplica ✅
- **Archivos:** `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/src/connections/child_process.rs`, `desktop/src-tauri/.gitignore`
- **Acción:** Si `vantadb-server` o `vantadb-mcp` van como sidecars, declararlos en `externalBin`. Verificar que binarios core se incluyen
- **Verify:** `cargo check -p vantadb` (workspace root)
- **Resultado:** DECISIÓN: se usa `bundle.resources {"binaries/vantadb-server.exe": "."}` en vez de `externalBin` — externalBin instala con sufijo target-triple que rompe `locate_binary()` (busca nombre plano); resources instala `vantadb-server.exe` junto al exe sin tocar Rust. Sidecar copiado a `binaries/` (gitignored). Bug inline fixed: typo `VANTVADB_SERVER_BIN`→`VANTADB_SERVER_BIN`. `build.rs` sin cambios (no aplica). Verify: `cargo check` (src-tauri workspace) ✅.

### Step 3: Verificar instalador funcional (nativo + server) ⬜ PENDING
- **Archivos:** `desktop/src-tauri/tauri.conf.json`, `desktop/src/main.tsx` (entry point)
- **Acción:** Build completo → instalar en Windows limpio (VM o máquina test) → verificar que conecta a DB nativa embebida Y a `vanta-cli server` remoto sin entorno de dev
- **Verify:** Instalador produce app funcional
- **Estado parcial:** Build completo + bundling hecho (NSIS 9.9MB / MSI 13.4MB, sidecar 12.7MB incluido en staging). Falta: instalar en máquina Windows limpia y probar conexión nativa + server (requiere VM/máquina de test — tarea manual/humana).

## Context Save Point
- Instaladores generados: `desktop/src-tauri/target/release/bundle/nsis/vantadb-desktop_0.1.0_x64-setup.exe` y `.../msi/vantadb-desktop_0.1.0_x64_en-US.msi`.
- Para re-bundlear tras rebuild del core: `cargo build -p vantadb-server --release` (repo root) → copiar a `desktop/src-tauri/binaries/vantadb-server.exe` → `cd desktop && npm run tauri build`. CI (DESKTOP-25) debe automatizar ese copy.
- NO commitear `binaries/` (gitignored). Cambios versionables: tauri.conf.json, .gitignore, child_process.rs (typo fix), este task file.

## Dependencias
- DESKTOP-25 (CI) para automatizar en GitHub Actions

## Notas
- DoD: instalador produce una app que conecta nativo + server sin entorno de dev
- Windows primero (NSIS/MSI); macOS/Linux en tareas futuras
- Priorizado 2026-08-20