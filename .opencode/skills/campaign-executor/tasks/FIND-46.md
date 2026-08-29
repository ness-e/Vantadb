# FIND-46: Doc drift semver-checks — Documentar cargo semver-checks en pre-release gate

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W0-2)
- **Creado:** 2026-08-29T20:00
- **last-synced:** 2026-08-29T20:15
- **Estado:** ✅ COMPLETED
- **Prioridad:** 🟡 Media
- **Esfuerzo:** 🟢 1h
- **Appetite:** max 1h
- **Origen:** codegraph-20260827 Fase 11 — `cargo semver-checks` no documentado pre-release

## Contrato (verbatim del plan file, líneas 142-145)

`cargo semver-checks --help 2>&1 | Measure-Object | Select-Object Count` >= 1 (cargo-semver-checks instalado)
**OR**
`Select-String en docs/operations/RELEASE.md patrón "semver-checks" Count >= 1`

### Estado del contrato (verificado 2026-08-29)
- `cargo semver-checks --help 2>&1 | Measure-Object | Select-Object Count` → **92** ✅ (primera parte del contrato pasa)
- `docs/operations/RELEASE.md` → **NO EXISTE** (segunda parte del contrato no aplica por path)
- `docs/operations/` menciona `semver-checks` → **0 matches** (drift confirmado)

### Decisión de implementación
El gate existe en `.github/workflows/ci-rust-10.yml:88-118` (job `semver-checks`) desde RELEASE-01 pero NO está documentado como proceso pre-release en `docs/operations/`. FIND-46 corrige el drift docs↔CI agregando la sección correspondiente en `ci-cd-guide.md` (workflow ya catalogado allí) y una referencia cruzada en `CI_POLICY.md`. No creo `RELEASE.md` (over-engineering — el proceso pre-release vive en CI_POLICY + release-plz.toml; el gate mecánico vive en CI; un RELEASE.md separado duplicaría sin agregar info).

## Blast Radius

### Archivos a tocar (lista cerrada)
| Path | Acción | Razón |
|------|--------|-------|
| `docs/operations/ci-cd-guide.md` | ADD § "Semver-checks gate (public API)" (~30 líneas) | Drift codegraph-20260827 Fase 11: doc drift entre CI y operations docs |
| `docs/operations/CI_POLICY.md` | ADD semver-checks row a job table §1 (~3 líneas) | Consistencia con ci-cd-guide.md y evitar doc drift duplicado |
| `docs/operations/master-index.md` | UPDATE last_reviewed + sin agregar nueva fila (ci-cd-guide.md ya está) | Higiene |

### Referencias entrantes (¿quién cita estos archivos?)
- `docs/operations/ci-cd-guide.md`:
  - `docs/operations/CI_POLICY.md:21-22` ("documented below" + "Local Verification Scripts")
  - `docs/operations/master-index.md:50` (catalogado)
  - `docs/operations/AGENT_INSTRUCTIONS.md` (general)
- `docs/operations/CI_POLICY.md`:
  - `.opencode/AGENTS.md` (CI/Hooks integration table — referencia)
  - `docs/operations/ci-cd-guide.md` (workflow inventory)
  - `docs/operations/master-index.md:6` (catalogado)
- `docs/operations/master-index.md`:
  - `docs/operations/*` (todas las filas lo referencian)

### Referencias salientes (¿a qué nuevos archivos se referenciará?)
- `docs/operations/ci-cd-guide.md` → `.github/workflows/ci-rust-10.yml:88-118` (job a documentar)
- `docs/operations/CI_POLICY.md` → idem + ci-cd-guide.md (cross-ref)

### Veredicto de impacto (Regla 0)
**Bajo.** 3 archivos de docs (no código, no API pública, no tests, no schema). No rompe compiladores, no rompe contratos, no afecta runtime. Solo resuelve drift de documentación. Sin gates de seguridad/concurrencia/performance aplicables.

## SDP (Paso 0b)
- `campaign_discover_skills` para `files="docs/operations/"` + `keywords=["semver-checks","release","api"]` + `phase="BUILD"` →
- Cargadas: `documentation-and-adrs` (ya cargada), `codebase-memory` (no requerida para docs-only task sin blast radius Rust)
- Registro: **SDP: documentation-and-adrs** (base-only — docs-only task no requiere skills de implementación)

## Tools
- bash (Measure-Object, Select-String para contrato)
- read / grep (contexto docs/operations existentes)
- edit (3 archivos docs)
- git (commit al cierre — staged para vanta-lead)

## Steps

### Step 1: Verificar contrato + mapear docs pre-existentes
- **Archivos:** `docs/operations/ci-cd-guide.md`, `docs/operations/CI_POLICY.md`, `docs/operations/master-index.md`, `.github/workflows/ci-rust-10.yml`
- **Acción:** Ya hecho en discovery (ver líneas 80-129 de ci-rust-10.yml + grep en docs/operations).
- **Verify:** `cargo semver-checks --help 2>&1 | Measure-Object | Select-Object Count` ≥ 1 ✅ (92)
- **Estado:** ✅ DONE

### Step 2: Documentar gate semver-checks en ci-cd-guide.md
- **Archivos:** `docs/operations/ci-cd-guide.md`
- **Acción:** Agregada fila en job table de ci-rust-10.yml + subsección "Semver-checks gate (public API)" con: scope (solo `vantadb`), baseline (crates.io latest), qué bloquea (breaking change public API), install local, referencia al job CI.
- **Verify:** `Select-String -Path "docs/operations/ci-cd-guide.md" -Pattern "semver-checks" | Measure-Object Count` → 5 ✅
- **Estado:** ✅ DONE

### Step 3: Cross-reference en CI_POLICY.md job table §1
- **Archivos:** `docs/operations/CI_POLICY.md`
- **Acción:** Agregada fila `| semver-checks | Public API Semver (RELEASE-01) — ... |` en la tabla de jobs Fast Gate (§1) con cross-ref a ci-cd-guide.md.
- **Verify:** `Select-String -Path "docs/operations/CI_POLICY.md" -Pattern "semver-checks" | Measure-Object Count` → 1 ✅
- **Estado:** ✅ DONE

### Step 4: Update last_reviewed en master-index.md (higiene)
- **Archivos:** `docs/operations/master-index.md`
- **Acción:** Cambiado `last_reviewed: 2026-08-10` → `last_reviewed: 2026-08-29`.
- **Verify:** `Select-String -Path "docs/operations/master-index.md" -Pattern "2026-08-29" | Measure-Object Count` → 1 ✅
- **Estado:** ✅ DONE

### Step 5: Verify full del contrato
- **Archivos:** N/A (verify only)
- **Acción:** Ejecutados ambos paths del OR del contrato.
- **Verify:**
  - `cargo semver-checks --help 2>&1 | Measure-Object | Select-Object Count` → **92** ✅ (path 1)
  - `grep "semver-checks" docs/operations/` → **6 matches** ✅ (path 2 alternativo: ci-cd-guide 5 + CI_POLICY 1)
- **Estado:** ✅ DONE

### Step 6: Stage cambios + handoff a vanta-lead
- **Archivos:** 3 archivos modificados
- **Acción:** `git add docs/operations/CI_POLICY.md docs/operations/ci-cd-guide.md docs/operations/master-index.md` ejecutado. NO commit — vanta-docs no hace commit (regla leaf).
- **Verify:** `git diff --cached --stat` → 3 files, +15/-1 ✅
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (task W0-2 aislada, docs-only)

## Pre-mortem (del plan + propio)
- **Fallo 1:** cargo semver-checks requiere install — ya verificado (`--help` retorna 92 líneas), `taiki-e/install-action` lo instala en CI. Documentar install local en sección.
- **Fallo 2:** workflow semver-checks puede ser CI-only, no local → documentar ambos paths (CI automático + local manual con `cargo install cargo-semver-checks`).

## Notas
- vanta-docs es leaf node (task: deny) — no invoca otros agentes. Commit queda staged para vanta-lead.
- Idioma: inglés (Regla 3 — docs técnicas en inglés).
- Conventional commit sugerido: `docs: FIND-46 — Documentar cargo semver-checks en pre-release gate`

## Context Save Point
- **Fecha:** 2026-08-29T20:00
- **Branch:** develop
- **Estado git pre:** working tree limpio (a verificar con `git status --short` antes de Step 6)
- **Decisiones:**
  1. NO crear `docs/operations/RELEASE.md` (over-engineering — info ya vive en CI_POLICY + ci-cd-guide)
  2. Documentar en `ci-cd-guide.md` (donde está catalogado el workflow) + cross-ref en CI_POLICY (donde está la tabla de jobs)
  3. NO tocar `.github/workflows/ci-rust-10.yml` (gate ya existe y funciona — solo doc drift)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** W0-3 PROV-08 (READMEs providers) — secuencia del plan W0