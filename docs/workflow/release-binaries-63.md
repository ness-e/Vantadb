# `release-binaries-63.yml` — RELEASE: Binaries — Build & Upload

## ¿Qué hace?

Construye los binarios compilados de VantaDB (vanta-cli, vantadb-server, vantadb-mcp) para 5 targets/platforms diferentes y los sube como assets al GitHub Release.

## ¿Cómo lo hace?

Un job `build` con matrix de 5 targets:

| Target | OS | Arquitectura |
|--------|----|-------------|
| `x86_64-unknown-linux-gnu` | ubuntu | x86_64 |
| `x86_64-apple-darwin` | macos | x86_64 |
| `aarch64-apple-darwin` | macos | ARM64 (Apple Silicon) |
| `x86_64-pc-windows-msvc` | windows | x86_64 |
| `aarch64-unknown-linux-gnu` | ubuntu | ARM64 (Linux) |

Para el target `aarch64-unknown-linux-gnu` instala el cross-compiler `gcc-aarch64-linux-gnu`.

Build de 3 binarios por target:
- `vanta-cli`
- `vantadb-server`
- `vantadb-mcp`

Empaqueta en `.tar.gz` (Unix) o `.zip` (Windows) y sube al Release de GitHub con `gh release upload`.

## ¿Qué tests usa?

No ejecuta tests. Solo compila en release y empaqueta.

## ¿Qué verifica?

- Los 3 binarios compilan correctamente para cada target
- Los artefactos se empaquetan y suben al Release

## Funcionalidad final

Distribuir binarios precompilados de VantaDB para Linux (x86_64 + ARM64), macOS (Intel + Apple Silicon) y Windows (x86_64) en cada release de GitHub.

## ¿Cuándo se ejecuta?

- **Cuando se publica un Release** en GitHub (`release: [published]`)
