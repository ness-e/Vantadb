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

<!-- Referencias cruzadas: → ver api-contract.md -->
