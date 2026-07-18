---
name: vantadb-certify
description: >
  Pre-push certification gate for VantaDB. Runs ALL verification layers
  sequentially: Rust compile/lint/test → Python SDK → Web → TypeScript SDK
  → docs → audit + skill-based code review. Must pass 100% before pushing.
---

# VantaDB Certification Gate

Certificación completa pre-push. El pipeline entero (CI) debe pasar localmente.

## Layers

### Layer 0: CodeGraph Impact Analysis

```
codegraph_explore "affected modules by $(git diff --name-only HEAD)"
```

Identifica qué tests se ven afectados por los cambios staged vía el knowledge graph del workspace.
Si detecta archivos sensibles (workflows, configs, unsafe), escala la revisión.

### Layer 1: Rust — Compilación + Lints + Tests

Corresponde al workflow CI Rust (`.github/workflows/ci-rust-10.yml`).

| Check | Comando | Falla si |
|-------|---------|----------|
| Format | `cargo fmt --all -- --check` | Cualquier diff |
| Compile | `cargo check --workspace --tests -j 2` | Cualquier error |
| Clippy | `cargo clippy --workspace --tests -j 2 -- -D warnings` | Cualquier warning |
| Audit | `cargo audit --ignore RUSTSEC-2026-0176 --ignore RUSTSEC-2026-0177` | Advisory activo |
| Deny | `cargo deny check` | Licencia/bans/sources violados |
| Tests | `cargo nextest run --profile audit --workspace --build-jobs 2` | Cualquier test fallido |
| Unused deps | `cargo machete` | Dependencias no usadas |
| Cli check | `pwsh dev-tools/scripts/check_cli.ps1` | CLI no compila |

Si el diff contiene `unsafe` → además:

| Check | Comando |
|-------|---------|
| Miri | `cargo +nightly miri test` (si nightly disponible) |

### Layer 2: Python SDK

| Check | Comando |
|-------|---------|
| Build + test | `pwsh dev-tools/setup_venv.ps1 && pwsh dev-tools/scripts/validate_python_sdk.ps1` |
| Integration tests | Cargar `doubt-driven-development` y verificar tests de adapters tocados |

### Layer 3: Web Frontend

Corresponde a `.github/workflows/ci-web-11.yml`.

```
cd web
npm ci --ignore-scripts
npm run lint          # 0 errors, 0 warnings
npx tsc --noEmit      # 0 errors
npm run build         # build exitoso
cd ..
```

### Layer 4: TypeScript SDK (si cambió)

```
cd vantadb-ts
npm ci --ignore-scripts
npx tsc --noEmit
npm test
cd ..
```

### Layer 5: Documentation

```
pwsh scripts/validate-docs-coverage.ps1
```

### Layer 6: GitHub Actions YAML (si cambió)

Si el diff toca `.github/workflows/*.yml`:
- `actionlint` (si está instalado) — valida sintaxis YAML + workflow
- Verificar que `act` dry-run no da error (si instalado)

### Layer 7: Code Review (skills)

Después de que todas las layers mecánicas pasan:

**7a. CI/CD Parity Check** (nuevo vector)

Para cada archivo tocado en el diff:

| Origen | Qué verificar contra CI |
|--------|------------------------|
| `Cargo.toml` | Nueva dep → existe `cargo add` o `apt-get install` en `.github/workflows/*.yml` |
| `package.json` | Nueva dep → existe `npm ci` o `npm install` en los workflows |
| `pyproject.toml` / `setup.py` | Nueva dep → existe `pip install` en los workflows |
| `.env` / `secrets` | Toda env var nueva está inyectada en los workflows o documentada como secret |
| `Cargo.toml` version | Version bump coincide con release workflow (release-wheels) |

Si el diff omite actualizar un workflow que debería cambiar → FAIL.

**7b. Auditoría cognitiva con skills:**

| Skill | Propósito |
|-------|-----------|
| `code-review-and-quality` | Revisión multi-eje: correctitud, seguridad, performance, mantenibilidad |
| `doubt-driven-development` | Verificación adversarial en contexto fresco (stakes altos) |
| `code-simplification` | ¿Código más simple de lo que quedó? |
| `security-and-hardening` | Input validation, trust boundaries, data exposure |
| `deprecation-and-migration` | Si se removió algo público |

Cargar cada skill con `skill <nombre>` en orden. Cada skill puede vetar el push registrando la objeción. El reporte final incluye todos los vetos, pero la certificación continúa (el dev decide qué vetos atender antes del push real).

### Layer 8: Commit Readiness

```
[ ] git status limpio (sin cambios sin stage)
[ ] git diff --cached revisado (sin secrets, sin debug code)
[ ] Commit message sigue Conventional Commits
[ ] Branche name sigue convención (fix/ feat/ chore/)
[ ] Referencia cruzada a issue/backlog ID
```

## Failure handling

Si una layer mecánica (L1-L5) falla → **detenerse**, reportar la falla y no continuar. No tiene sentido revisar código que no compila.

Si una layer cognitiva (L7) encuentra issues → **registrar todos los hallazgos** en el reporte pero continuar con las siguientes sub-layers. El reporte final consolida TODO: layers pasadas, fallos, y vetos de skills en un solo documento.

## Pre-push hook (instalación local)

El hook es la **primera línea de defensa** — si falla, OpenCode ni se invoca.

```powershell
# .git/hooks/pre-push.ps1 — SIPP
Write-Host "[SIPP] Barrera determinista..."
cargo check --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo nextest run --profile audit --workspace --build-jobs 2
if ($LASTEXITCODE -ne 0) { exit 1 }
Write-Host "[SIPP] Barrera superada."
```

## Referencias CI

| Workflow | Archivo |
|----------|---------|
| Rust CI | `.github/workflows/ci-rust-10.yml` |
| Web CI | `.github/workflows/ci-web-11.yml` |
| Docs Gate | `.github/workflows/gate-docs-21.yml` |
| Security | `.github/workflows/sec-codeql-30.yml` |
| Heavy Cert | `.github/workflows/heavy-certification-50.yml` |
| Release | `.github/workflows/release-wheels-60.yml` |
