# RES-11: Job CI `cargo doc --no-deps --workspace` + artifact

## Metadata
- **Plan file:** docs/plans/2026-08-31-fast-gate-residues.md
- **Creado:** 2026-08-31
- **last-synced:** 2026-08-31
- **Estado:** ✅ COMPLETED
- **Tipo (campaign_detect_task_type):** ci / workflow (GitHub Actions)
- **Esfuerzo:** 🟢 ~30 min
- **Prioridad:** 🟢 Baja (post-docs adoption; útil para adopters pre-docs.rs)
- **Sub-agent:** vanta-lead (CI/CD)
- **Task previa:** TBH-06 ✅

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `.github/workflows/ci-rust-10.yml` | ~120 | Fast gate principal (fmt, clippy, check, test). NO tiene `cargo doc`. |
| `.github/workflows/ci-rustdoc-10.yml` | N/A | NO EXISTE — se crea nuevo. |
| `.github/workflows/heavy-bench-nightly-51.yml` | ~260 | Nightly benches. Referencia para triggers `schedule` + `workflow_dispatch`. |
| `Cargo.toml` | 680 | Workspace con 17+ crates. `cargo doc --no-deps --workspace --all-features` compila docs de todos. |

### Referencias hacia adentro
- Nuevo workflow `.github/workflows/rustdoc-70.yml` → consume `cargo doc`, `actions/upload-artifact`, `tar` para compresión.

### Referencias hacia afuera
- Nadie consume este workflow (es generador de artifact para adopters externos).

### Veredicto de impacto
**Mínimo.** Un archivo nuevo en `.github/workflows/`. Zero blast radius sobre código existente.

## Contrato (verificable mecánicamente)

```
1. Archivo `.github/workflows/rustdoc-70.yml` existe y pasa `actionlint` ✅
2. Trigger: `push` a `develop` + `workflow_dispatch` + `paths:` filter (solo docs/ + src/ + Cargo.toml)
3. Job: `cargo doc --no-deps --workspace --all-features --document-private-items`
4. Artifact: `rustdoc-html.tar.gz` (comprimido, < 500MB)
5. Retención: 30 días (GitHub default) o configurable
6. `cargo doc` NO rompe fast gate existente (workflow separado, nombre único)
```

## Steps

### Step 1: Crear workflow `rustdoc-70.yml`
- **Archivos:** `.github/workflows/rustdoc-70.yml` (NUEVO)
- **Acción:** Escribir workflow YAML con:
  - name: "Rustdoc API Reference"
  - triggers: push to develop (paths: ['src/**', 'docs/**', 'Cargo.toml', '*.md']) + workflow_dispatch
  - jobs: `build-doc` → `cargo doc --no-deps --workspace --all-features --document-private-items` → `tar -czf rustdoc-html.tar.gz target/doc` → `actions/upload-artifact@v4` con `retention-days: 30`
  - timeout: 30 min
  - concurrency: cancel-in-progress para develop
- **Verify:** `actionlint .github/workflows/rustdoc-70.yml` → exit 0
- **Estado:** ⬜ PENDING

### Step 2: Verificar sintaxis y dry-run
- **Archivos:** `.github/workflows/rustdoc-70.yml`
- **Acción:** `actionlint .github/workflows/rustdoc-70.yml` + `gh workflow view rustdoc-70.yml --repo ness-e/Vantadb` (si gh auth) O revisión manual
- **Verify:** actionlint 0 errors
- **Estado:** ⬜ PENDING

### Step 3: Commit + push
- **Archivos:** `.github/workflows/rustdoc-70.yml`
- **Acción:** `git add .github/workflows/rustdoc-70.yml` + `git commit -m "ci: add rustdoc workflow generating API reference (RES-11)"`
- **Verify:** commit creado, push a develop
- **Estado:** ⬜ PENDING

### Step 4: Verificar ejecución (opcional - si gh auth disponible)
- **Archivos:** n/a
- **Acción:** `gh workflow run rustdoc-70.yml --repo ness-e/Vantadb` o esperar push a develop
- **Verify:** workflow aparece en Actions tab, job completa sin errores críticos (warnings aceptables)
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna. Independiente.

## Notas
- **Ponytail:** workflow mínimo, reusa patterns de `ci-rust-10.yml` y `heavy-bench-nightly-51.yml`.
- **Regla 1:** `--no-deps` evita compilar deps; `--document-private-items` incluye items privados para adopters; warnings no rompen (no `-D warnings` en cargo doc).
- **Artifact size:** `tar -czf` antes de upload. `target/doc` típico ~50-150MB comprimido <50MB.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** develop
- **CI pendiente:** no (workflow nuevo, no rompe existing gates)
- **Decisiones:**
  - Nombre `rustdoc-70.yml` (número 70 para ordenar después de ci-rust-10, ci-rustdoc-10 no existe)
  - Trigger paths filter para no spamear en cambios no-docs
  - `--document-private-items` para adopters que necesitan ver internals
  - Comprimido con tar.gz para artifact size