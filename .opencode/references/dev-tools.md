# Dev Tools (Instalados)

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar cuando necesites comandos `just`, aliases git, tooling instalado o detalles de release. Si editas, actualiza también el puntero en AGENTS.md.

Herramientas de desarrollo instaladas globalmente para optimizar el workflow de un solo dev.

## Cargo Tools

| Herramienta | Instalada | Comando | Propósito |
|-------------|-----------|---------|-----------|
| **cargo-watch** | ✅ | `cargo watch -x check` | Feedback loop sub-second. Re-ejecuta comandos en cada cambio de archivo |
| **cargo-machete** | ✅ | `cargo machete` | Detecta dependencias no usadas |
| **cargo-bloat** | ✅ | `cargo bloat --crates` | Analiza qué engorda el binario release |
| **cargo-outdated** | ✅ | `cargo outdated` | Lista dependencias desactualizadas |
| **cargo-nextest** | ✅ | `cargo nextest run` | Test runner ~3× más rápido que cargo test |
| **cargo-deny** | ✅ | `cargo deny check` | Auditoría de licencias + advisory + bans |
| **cargo-audit** | ✅ | `cargo audit` | Security advisory checker |
| **release-plz** | ✅ | `release-plz release` | Automatiza bump de versiones, changelog, y publish |
| **git-cliff** | ✅ | `git-cliff -o CHANGELOG.md` | Generador de changelog desde conventional commits |

## Justfile

El **Justfile** en la raíz del proyecto es el reemplazo moderno de Makefile. Instalación: `cargo install just`

Comandos principales:

```bash
just check            # cargo check --workspace (feedback rápido)
just test             # cargo nextest run --profile audit
just verify           # fmt + clippy + test + deny (pre-flight completo)
just verify-quick     # dev-tools/verify_changed.ps1 (30s, CodeGraph-optimized)
just watch            # cargo watch -x check -x 'nextest run' (loop infinito)
just fmt-fix          # cargo fmt (aplica formato)
just machete          # cargo machete (deps no usadas)
just size             # cargo bloat --crates (tamaño binario)
just outdated         # cargo outdated (deps stale)
just audit            # cargo audit (seguridad)
just changelog        # git-cliff -o docs/CHANGELOG.md
just ci               # fmt + clippy + test + deny + audit (mismo orden que CI)
just certify          # nocturnal_suite.ps1 (certificación pesada local)
just release          # cargo build --release
just run-cli          # cargo run --features cli
just run-server       # cargo run --features server --bin vantadb-server
```

## Git Aliases

Configurados globalmente en `~/.gitconfig`:

| Alias | Comando real |
|-------|-------------|
| `git lg` | `log --oneline --graph --all --decorate` |
| `git st` | `status -sb` |
| `git ci` | `commit` |
| `git co` | `checkout` |
| `git br` | `branch` |
| `git rb` | `rebase -i` |
| `git up` | `push -u origin HEAD` |
| `git fixup` | `commit --fixup` |
| `git amend` | `commit --amend --no-edit` |
| `git undo` | `reset --soft HEAD~1` |
| `git unstage` | `reset HEAD --` |

## VS Code Setup

Archivos en `.vscode/`:

| Archivo | Propósito |
|---------|-----------|
| `extensions.json` | Recomienda rust-analyzer, CodeLLDB, crates, Even Better TOML, GitLens, cSpell, markdownlint, ShellCheck |
| `settings.json` | Config: rust-analyzer con clippy + features del proyecto, formatOnSave, exclude patrones |
| `tasks.json` | 10 tareas: check, clippy, nextest, fmt, deny, verify, build release, run cli/server |
| `mcp.json` | cargo-mcp + rust-analyzer-mcp para GitHub Copilot Chat |

## Dependabot

Configurado en `.github/dependabot.yml` para 4 ecosistemas:

| Ecosistema | Schedule | Límite PR |
|------------|----------|-----------|
| **Cargo** | Weekly (lunes) | 10 PRs |
| **npm (web/)** | Weekly (lunes) | 5 PRs |
| **GitHub Actions** | Weekly (lunes) | Ilimitado |
| **Docker** | Weekly (lunes) | Ilimitado |

Las PRs se agrupan por tipo (patch, minor) para reducir ruido.

## release-plz

Configurado en `release-plz.toml`. Automatiza:

1. Análisis de conventional commits desde el último tag
2. Bump semántico de versiones (feat → minor, fix → patch, breaking → major)
3. Actualización de `docs/CHANGELOG.md`
4. Creación de tag `v{{ version }}` en git
5. Publicación a crates.io (en orden de dependencias del workspace)

Uso: `release-plz release` (desde la rama main, después de mergear)

## CI: sccache

Integrado en `.github/actions/rust-setup/action.yml` mediante `mozilla-actions/sccache-action@v0.0.11` (sccache `v0.16.0`), con env `SCCACHE_GHA_ENABLED=true` + `RUSTC_WRAPPER=sccache` escritas a `$GITHUB_ENV` (las composite actions no soportan `env` a nivel `runs`). Usa el backend de GHA cache automáticamente (sin infra adicional) y complementa a `Swatinem/rust-cache`: acelera rebuilds reutilizando objetos compilados entre jobs/runs.

## Flujo diario recomendado

```bash
# Desarrollo iterativo
just watch-check                    # terminal 1: feedback instantáneo

# Antes de commit
just verify                         # fmt + clippy + test + deny

# Commit
git add -p && git ci -m "feat: ..."
git up

# Release (cuando toca)
release-plz release                 # bump + changelog + tag + publish
```
