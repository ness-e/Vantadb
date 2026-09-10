# DESKTOP-42 — Bundles macOS/Linux + CI matrix

> **Plan:** `docs/plans/2026-09-10-code.md` (Task 6, Wave1) · **Estado:** ⏳ IN PROGRESS
> **Ruta:** vanta-worker · **Appetite:** max 2d · **Tipo:** devops (CI/CD)
> **Contrato:** targets dmg+AppImage/deb en config + job CI por SO verde (o documentado) + build local del target primario
> **SDP:** ci-cd-and-automation, source-driven-development, incremental-implementation, context-engineering (+ base campaign-executor, progreso, doubt-driven-development)

## DISCOVERY (2026-09-10)

- `desktop/src-tauri/tauri.conf.json`: `bundle.targets` = `["nsis","msi"]` (solo Windows); `resources` = mapa fijo `binaries/vanta-cli.exe` (nombre Windows-only); `icon.icns` existe ✅ (98 KB); sin secciones `macOS`/`linux` (usan defaults).
- `.github/workflows/desktop.yml`: un solo job `windows` (windows-latest, tauri-action v1 pineado por SHA, sidecars `custom-allocator`, artifact NSIS+MSI). Sin jobs macos/linux.
- `desktop/src-tauri/src/connections/child_process.rs:40-43`: `CANONICAL_EXE` ya es per-OS (`vanta-cli.exe` vs `vanta-cli`) — el runtime resuelve el sidecar por plataforma; el config solo debe empaquetar el nombre correcto por SO.
- `binaries/` está en `.gitignore` (`src-tauri/.gitignore:10`) — solo contiene lo que CI/local construye; un glob `binaries/*` no empaqueta basura versionada.
- Docs oficiales verificadas: `bundle.targets` admite `["deb","rpm","appimage","nsis","msi","app","dmg"]` o `"all"`, default `"all"` (source: https://v2.tauri.app/reference/config/#bundleconfig — el default `all` prueba que el bundler filtra por SO host: un build Windows nunca intenta `.dmg`); `bundle.resources` acepta globs y en forma mapa "everything gets copied to the target directory directly" (misma fuente); platform-specific conf files existen pero NO se usan (una sola config + filtrado nativo es más simple); tauri-action matrix canónica + deps Ubuntu `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf` (source: https://github.com/tauri-apps/tauri-action README).
- `.opencode/rules/release-ci.md` Regla 1: binarios prod en Linux/macOS con `--features jemalloc` (root `Cargo.toml:117`, server `Cargo.toml:35`); Regla 2: sccache solo vía `./.github/actions/rust-setup` (NO duplicar); Regla 5: sin `continue-on-error` nuevo.
- **Gate D (question-gates):** NO dispara — blast radius 2 archivos config, sin símbolos públicos nuevos, sin hot path, sin API pública. Motivo: cambio config-only disjunto Wave1.
- **Pre-mortem del plan:** firma macOS sin cert → solo build, no notarize (tauri-action sin inputs de signing = unsigned build-only ✅, sin acción requerida); runners Linux sin deps → se documenta/instala step `apt-get` según README oficial.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src-tauri/tauri.conf.json` (65L), `.github/workflows/desktop.yml` (114L), `.opencode/rules/release-ci.md`, `.github/actions/rust-setup/action.yml`, `desktop/src-tauri/src/connections/child_process.rs` (1-120), `desktop/package.json`.
- **Referencias hacia dentro (qué toca el cambio):** `tauri.conf.json` → leído por `tauri-cli`/`tauri-action` en build; `desktop.yml` → solo CI (ningún código lo importa). `icon.icns` ya referenciado en `bundle.icon`.
- **Referencias entrantes (quién depende):** `child_process.rs` `locate_binary()` resuelve `vanta-cli[.exe]` por `cfg(windows)` — el glob por plataforma del config alimenta exactamente esos nombres; `desktop/e2e/` y tests src-tauri usan `VANTADB_CLI_BIN` (no tocan config).
- **Veredicto:** impacto confinado a empaquetado/CI. Windows local y job `windows` intactos (targets nsis/msi siguen incluidos; glob `binaries/vanta-cli*` casa `vanta-cli.exe` igual que antes + `vantadb-server.exe` que ya se construía en CI). Sin cambios Rust/TS. Riesgo residual: `tauri build` Windows local debe seguir verde (step 4 lo prueba).

## Steps

- [x] 1. `tauri.conf.json`: targets ×6 + metadata (category/descripciones) + resources por-glob multiplataforma
- [x] 2. `desktop.yml`: job `macos` (sidecars jemalloc, build-only sin firma, artifacts dmg+app)
- [x] 3. `desktop.yml`: job `linux` (deps webkit per README oficial, sidecars jemalloc, artifacts AppImage+deb)
- [x] 4. Verify mecánico: json/yaml lint + `tauri info` + build local target primario
- [ ] 5. Commit solo-propios en develop + recitation + RESULTADO

## Verify (2026-09-10)

- `python json.load(tauri.conf.json)` ✅ targets `["nsis","msi","dmg","app","appimage","deb"]`, resources globs `binaries/vanta-cli*` + `binaries/vantadb-server*`
- `python yaml.safe_load(desktop.yml)` ✅ jobs `[windows, macos, linux]`; `actionlint` ✅ exit 0
- `cargo fmt --all --check` ✅ (repo limpio, sin bloqueos ajenos esta vez)
- `cargo check --manifest-path desktop/src-tauri/Cargo.toml --offline` ✅ 1m05s
- `npm run tauri -- build --debug --bundles nsis` ✅ → `target/debug/bundle/nsis/vantadb-desktop_0.1.0_x64-setup.exe` (14.3 MB; tamaño consistente con app 32 MB + sidecars 23 MB comprimidos LZMA; sin warnings de resources en log `--verbose`)
- Release local NO corre en este runner: rustc 1.95.0 crashea (`STATUS_STACK_BUFFER_OVERRUN` / `Stack overflow`) compilando deps release (`serde_spanned`, `webview2-com`, `bitpacking`, `tauri-utils`) — 3 intentos (default, `cargo clean -p serde_spanned` + `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`); mismo crash documentado en MEM-69 (tests). Estrategia cambiada a `--debug`: compila y bund Guzla NSIS OK. Causa raíz = toolchain del runner, no el cambio (solo config JSON/YAML, sin código Rust). Release queda cubierto por CI (job `windows` con `--target x86_64-pc-windows-msvc`).
- Regla 5 release-ci: ningún `continue-on-error` nuevo; SHAs pineados reutilizados; `if-no-files-found: error` en los 3 jobs.
- `desktop/src-tauri/Cargo.lock` modificado en worktree: pre-existente + posible churn de mi build (descarga `serde_spanned v0.6.9`) → NO stagear (precedente 09-10, locks ajenos).
