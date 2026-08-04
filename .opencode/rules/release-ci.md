# Release & CI — Reglas

> **Scope:** release-plz (`release-plz.toml`), versionado semver, conventional commits, changelog (`cliff.toml`, `docs/CHANGELOG.md`), GitHub Actions (`.github/workflows/`), deny.toml, `cargo semver-checks`, publish (crates.io / PyPI / npm), `integrations/` (adapters Python)
> **No tocar aquí:** API pública (`api-contract.md`), bindings específicos (`python-bindings.md`, `js-ecosystem.md`, `server-mcp.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-017, INV-004, Regla 7 (AGENTS.md)

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

<!-- Referencias cruzadas: → ver api-contract.md, python-bindings.md -->
