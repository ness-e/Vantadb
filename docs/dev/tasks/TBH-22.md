# TBH-22 — release-binaries-63.yml: add `push tags v*` trigger

## Contexto (CI infra hardening — Phase 1 closeout)

Workflow `release-binaries-63.yml` solo dispara con `release: types: [published]`.
Si alguien pushea un tag `vX.Y.Z` sin crear un GitHub Release primero, NO se
construyen los binarios. Gap identificado en auditoría multi-agente
2026-08-30 (Prioridad BAJA — 1-line fix trivial).

## Contrato

- Agregar `push: tags: ['v*']` al bloque `on:` (mismo patrón que `release-sbom-64.yml`).
- **NO eliminar** el trigger existente `release: types: [published]`.
- NO tocar jobs, steps, secrets, matrix, concurrency, permissions.

## Archivos clave

- `.github/workflows/release-binaries-63.yml` (modificar bloque `on:`)
- `.github/workflows/release-sbom-64.yml` (referencia del patrón `tags: ["v*"]`)
- `.github/workflows/release-wheels-60.yml` (referencia, usa `v*.*.*` — distinto)
- `.github/workflows/release-npm-61.yml` (referencia, usa `v*.*.*` — distinto)

## Impacto mapeado (Regla 0)

- Archivo leído completo: `release-binaries-63.yml` (148 líneas)
- Referencias salientes: ninguna — es un workflow de CI
- Referencias entrantes: ningún doc/skills/workflows lo invocan
- Veredicto: cambio aislado, blast radius = 0 archivos aguas abajo

## Plan

1. Edit `on:` block — añadir `push: tags: ['v*']` (YAML usa single quotes,
   consistente con `release-wheels-60.yml:19` y `release-npm-61.yml:17`)
2. Verify: YAML parse, pattern match `tags:.*v\*`, pattern match `release:.*published`
3. Commit `ci(TBH-22):` + update plan

## Spec

N/A (infra-only YAML, sin API pública, sin lógica, sin spec de feature).

## Steps

- [x] **S1 (ACT):** Edit `on:` block — add `push: tags: ['v*']` between
      `workflow_dispatch:` and `release:` (orden lógico: manual → tag → release)
- [x] **S2 (VERIFY):** `python -c "import yaml; yaml.safe_load(...)"`
- [x] **S3 (VERIFY):** `Select-String` matches both patterns
- [x] **S4 (VERIFY):** `actionlint` si está disponible
- [x] **S5 (COMMIT):** `ci(TBH-22):` conventional commit
