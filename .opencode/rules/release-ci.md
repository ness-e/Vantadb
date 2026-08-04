# Release & CI — Reglas

> **Scope:** release-plz (`release-plz.toml`), versionado semver, conventional commits, changelog (`cliff.toml`, `docs/CHANGELOG.md`), GitHub Actions (`.github/workflows/`), deny.toml, `cargo semver-checks`, publish (crates.io / PyPI / npm), `integrations/` (adapters Python)
> **No tocar aquí:** API pública (`api-contract.md`), bindings específicos (`python-bindings.md`, `js-ecosystem.md`, `server-mcp.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-017, INV-004, Regla 7 (AGENTS.md), auditoría docs/operations + Dockerfile 2026-08-04

## Reglas

### 1 — Builds de producción: habilitar allocator custom por plataforma

- **Must:** El workflow `release-binaries-63.yml` compila los binarios distribuibles (`vanta-cli`, `vantadb-server`) de producción con `--features custom-allocator` en Windows (`mimalloc`) y con `--features jemalloc` en Linux/macOS.
- **Must not:** Publicar builds release de binarios con el allocator del sistema por defecto (glibc malloc / MSVC CRT) en plataformas donde `custom-allocator` o `jemalloc` están disponibles.
- **Must not:** Habilitar `jemalloc` en Windows — `tikv-jemallocator` solo compila en `cfg(any(target_os = "linux", target_os = "macos"))`.
- **Por qué:** VantaDB maneja índices HNSW, grafos en RAM y serialización f32 frecuente; el allocator del sistema causa fragmentación de heap (RSS drift) y contención multihilo. mimalloc/jemalloc reducen ambos. La integración `#[global_allocator]` está en `src/bin/vanta-cli.rs:12-21` y `vantadb-server/src/main.rs:6-18`. Ver INV-004.

### 2 — CI: sccache habilitado vía rust-setup, sin pasos por workflow

- **Must:** mantener la integración de sccache en `.github/actions/rust-setup/action.yml` (mozilla-actions/sccache-action@v0.0.11, sccache v0.16.0, env `SCCACHE_GHA_ENABLED=true` + `RUSTC_WRAPPER=sccache` a `$GITHUB_ENV`) como punto único — beneficia todos los jobs que la usan.
- **Must not:** volver a `cargo install cargo-nextest --locked` en jobs de CI (usar `taiki-e/install-action` — fix del bottleneck de Windows en `ci-rust-10.yml:136-139`) ni duplicar steps de sccache por workflow.
- **Must not:** afirmar sccache en `.opencode/AGENTS.md` u otra doc sin que la implementación exista realmente (drift documental INV-017 §7).
- **Por qué:** sccache + install-action son los dos cambios de mayor ROI sobre el CI ya cacheado (INV-017: fix nextest ~30% del job Windows; sccache 0-15%); el drift previo costó tiempo de debugging.

### 3 — Dockerfile: MSRV del build ≥ `rust-version` del workspace, COPYs a rutas existentes

- **Must:** `ARG RUST_VERSION` en `Dockerfile` ser ≥ `rust-version` del root `Cargo.toml` (hoy 1.94.1); cada `COPY <crate>/Cargo.toml` apuntar a un directorio que exista en el repo.
- **Must not:** dejar `ARG RUST_VERSION=1.94.0` cuando el workspace exige 1.94.1, ni `COPY vantadb-mem0/...` etc. cuando los crates se movieron a `integrations/` (los 8 COPY de integraciones en el Dockerfile apuntan a dirs inexistentes → build roto).
- **Por qué:** la auditoría DOC1 confirmó el bug vivo: `Dockerfile:4` fija 1.94.0 < MSRV 1.94.1 y `Dockerfile:32-39` referencia crates movidos a `integrations/`; el build docker falla.

### 4 — Version sync: docs de API y packaging usan la versión del workspace

- **Must:** `docs/api/openapi.yaml`, `docs/api/*.md`, `web/package.json` y workflows de release declarar la misma versión que `[workspace.package] version` (hoy 0.5.0).
- **Must not:** hardcodear versiones divergentes en el mismo docset (openapi 0.4.0, MCP.md 0.1.5, HTTP_API.md 0.0.4) ni afirmar "web versión 0.2.0" cuando el workspace es 0.5.0.
- **Por qué:** tres versiones distintas en docs de API + una en el web; la auditoría encontró el drift en `openapi.yaml:4`, `MCP.md:310` y `web/AGENTS.md`.

### 5 — No `continue-on-error` sin CATEGORY tag (AGENTS.md Regla 2)

- **Must:** toda instancia de `continue-on-error: true` en `.github/workflows/` llevar un comentario `# CATEGORY:` explícito (EXPERIMENTAL / BEST-EFFORT / NON-CRITICAL / INFORMATIONAL según `docs/operations/CI_POLICY.md`).
- **Must not:** añadir `continue-on-error` nuevo sin justificación + CATEGORY tag ni sin issue `flaky` asociado.
- **Por qué:** hoy hay 8 instancias en 4 workflows (heavy-bench-nightly-51, ci-rust-10 ×4, release-adapters-62, release-wheels-60); la Regla 2 de AGENTS.md lo prohíbe salvo exenciones etiquetadas.

<!-- Referencias cruzadas: → ver api-contract.md, python-bindings.md, core-engine.md -->
