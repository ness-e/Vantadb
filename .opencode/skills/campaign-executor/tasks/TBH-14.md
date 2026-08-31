# TBH-14 — Fix `cliff.toml` `conventional_commits = true` (1-line change)

> Plan: `docs/plans/2026-08-30-testing-bench-harden.md` — TASK-14 (Phase 2 — MED, Prioridad MEDIA per audit 2026-08-30)
> Owner: vanta-lead (release/CI orchestrator — git-cliff config is release infra)
> Predecessors ✅: TBH-01, TBH-02, TBH-03, TBH-04, TBH-05, TBH-13
> Strategy: 1-line flip in `[git]` section. NO reformat. NO comments.

## Contexto

VantaDB usa `git-cliff` (config: `cliff.toml`) para generar `docs/CHANGELOG.md` desde
conventional commits. La config define `commit_parsers` que parsean prefijos
`feat:`, `fix:`, `perf:`, `test:`, `ci:`, `docs:`, etc. — formato conventional.

PERO `cliff.toml:15` tiene `conventional_commits = false`. Esto **contradice** los
`commit_parsers` y puede causar que cliff:
1. No aplique el linkado automático de issues/PRs (feat=link, fix=close)
2. No normalice el commit message para extracción del scope
3. Genere agrupamiento incorrecto cuando el commit format varía

La auditoría multi-agente del 2026-08-30 lo identificó como **Prioridad MEDIA**
(gap audit CI/CD).

## SDP — Skills cargadas

- `git-workflow-and-versioning` — Conventional Commits + atomic commits (cargada inline)
- `campaign-executor` — task system, state machine, task file format (cargada inline)

SDP: `git-workflow-and-versioning, campaign-executor` (base + 1 lifecycle keyword: changelog/git-cliff no tiene skill dedicada en el manifest)

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `cliff.toml` (61 lines)
- `docs/CHANGELOG.md` (header spot-check — release-plz regenera, no es fuente)

**Estado actual de `cliff.toml`:**

| Línea | Contenido | Veredicto |
|---|---|---|
| 14 | `[git]` | Section header |
| 15 | `conventional_commits = false` | **CAMBIO: → true** |
| 16-55 | `commit_parsers = [...]` con entries `feat`, `fix`, `perf`, etc. | NO TOCAR — formato conventional |
| 57-61 | `exclude_paths` | NO TOCAR |

**Referencias hacia dentro (a `cliff.toml`):**
- `release-plz.toml` — release-plz invoca `git-cliff` con `--config cliff.toml` para regenerar `CHANGELOG.md`. Habilitar `conventional_commits = true` mejora la generación sin cambiar la interfaz.

**Referencias hacia fuera (lo que `cliff.toml` referencia):**
- `commit_parsers` configuran el grouping por tipo (feat/fix/perf/...). Ya son conventional — el flip los hace consistentes.

**Veredicto:** impacto local — 1 línea. Sin blast radius de código Rust/Python/TS. Sin cambio de API pública. Sin ADR necesario (Regla 5 — no es tradeoff arquitectónico, es fix de config).

## Spec (no aplica — es fix mecánico de CI infra)

Cambio de configuración, no lógica nueva. Contrato mecánico verificable vía
`Select-String` + TOML parse. Sin ADR (Regla 5).

## Steps atómicos

### Step 1 — Flip `conventional_commits` (ACT)

- Línea 15: `conventional_commits = false` → `conventional_commits = true`
- NO reformatear, NO comments, NO tocar `commit_parsers`.

Status: ⬜ PENDING

### Step 2 — Verify full (VERIFY)

Comandos contractuales:
1. `Select-String -Path cliff.toml -Pattern "^conventional_commits = (true|false)"` → match debe ser `true`
2. TOML parse: `python -c "import tomllib; tomllib.load(open('cliff.toml','rb'))"` (skip si Python <3.11 y no hay `tomli`)
3. `git cliff --unreleased --dry-run` → SKIP si `git-cliff` no está instalado localmente

Status: ⬜ PENDING

### Step 3 — Commit (CIERRE)

```bash
git add cliff.toml .opencode/skills/campaign-executor/tasks/TBH-14.md
git commit -m "fix(TBH-14): enable conventional_commits in cliff.toml (matches commit_parsers config)"
```

Conventional commit `fix(...)` (es un fix de inconsistencia, no un feat — la config
estaba mal seteada).

Status: ⬜ PENDING

## Reglas duras

- **Ponytail reflex:** 1-line change. NO comments nuevos. NO reformat.
- **NO TOCAR** `commit_parsers` (líneas 16-55), `changelog`, `git` (excepto línea 15), `exclude_paths`.
- **NO regenerar** `docs/CHANGELOG.md` — release-plz lo hace.
- **Conventional Commits:** `fix(TBH-14):` (es fix, no feat).

## Estado

- Step 1: ✅ Flip `conventional_commits` (línea 15: false → true)
- Step 2: ✅ Verify full (`Select-String` match = `true`; TOML parse OK; git-cliff skip — no instalado localmente)
- Step 3: ⬜ PENDING (commit en curso)

## Estado final

✅ COMPLETO — 1-line flip (false → true) en `cliff.toml:15`. Select-String confirma `conventional_commits = true`. TOML parse OK. No se regeneró `docs/CHANGELOG.md` (release-plz lo hace). No se tocaron otras secciones.

## CONTRACT (verificado ✅)

```powershell
Select-String -Path cliff.toml -Pattern "^conventional_commits = (true|false)"
# → cliff.toml:15:conventional_commits = true ✅

python -c "import tomllib; tomllib.load(open('cliff.toml','rb')); print('TOML OK')"
# → TOML OK ✅

git diff cliff.toml
# → 1 línea cambiada (línea 15), nada más ✅

git cliff --unreleased --dry-run
# → SKIP (git-cliff no instalado localmente; contract lo permite)
```