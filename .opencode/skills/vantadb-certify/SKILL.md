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
codegraph affected --stdin < git diff --name-only HEAD
```

Identifica qué tests se ven afectados por los cambios staged.
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

Ejecutar cada skill secuencialmente. Cada una puede vetar el push.

### Layer 8: Commit Readiness

```
[ ] git status limpio (sin cambios sin stage)
[ ] git diff --cached revisado (sin secrets, sin debug code)
[ ] Commit message sigue Conventional Commits
[ ] Branche name sigue convención (fix/ feat/ chore/)
[ ] Referencia cruzada a issue/backlog ID
```

## Early exit

Si cualquier layer falla → el comando se detiene ahí.
No seguir con la siguiente layer hasta que la falla esté resuelta.

## Pre-push hook (instalación local)

El hook es la **primera línea de defensa** — si falla, OpenCode ni se invoca.

```bash
#!/bin/bash
# .git/hooks/pre-push — SIPP
echo "[SIPP] Barrera determinista..."
cargo check --workspace --all-targets || exit 1
cargo clippy --workspace --all-targets -- -D warnings || exit 1
cargo test --workspace || exit 1
mypy . --strict 2>/dev/null || true    # si existe
pytest -q 2>/dev/null || true          # si existe
npx tsc --noEmit 2>/dev/null || true   # si existe
echo "[SIPP] Barrera superada."
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
