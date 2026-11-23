---
id: FIND-466
title: "Blindar filtros paths: en workflows de CI para evitar checks colgados"
status: completed
type: fix
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
absorbe: FIND-582 (M-13)
---

# FIND-466 — Blindar filtros `paths:` en workflows de CI

## Objetivo
Tres workflows tienen filtros `pull_request.paths` incompletos: PRs que solo tocan
ficheros de release dejan checks requeridos sin ejecutar → PR colgado (Pending forever).

## Causa raíz
GitHub Actions: workflow filtrado por `paths` que no matchea → skipped, sus checks
quedan "Pending" y bloquean merges con required checks (doc oficial).

## Impacto mapeado (Regla 0)
- Archivos leídos completos: `.github/workflows/ci-rust.yml` (sección on:), `docs-gates.yml` (on:), `ci-fleet.yml` (on:)
- Referencias hacia dentro: triggers de GitHub Actions; consumen `release-plz.toml`, `release-manifest.json`
- Referencias entrantes: branch protection / rulesets (required checks), release-plz PRs
- Veredicto: cambio acotado a 3 listas `paths` en 3 YAML. Sin código. Blast radius = qué PRs disparan checks.

## Estado verificado (baseline)
- `ci-rust.yml` PR paths: tiene `.github/workflows/ci-rust.yml` e `integrations/**`; FALTAN `release-plz.toml`, `release-manifest.json`
- `docs-gates.yml` PR paths: FALTA `release-manifest.json`
- `ci-fleet.yml` PR paths: FALTA `integrations/**` (ya tiene `release-plz.toml`)
- Contrato ANTES: AssertionError en ci-rust (FAIL esperado) ✔

## Steps
- [x] 1. `ci-rust.yml`: añadir `release-plz.toml` + `release-manifest.json` a push.paths y pull_request.paths
- [x] 2. `docs-gates.yml`: añadir `release-manifest.json` a push.paths y pull_request.paths
- [x] 3. `ci-fleet.yml`: añadir `integrations/**` a push.paths y pull_request.paths
- [x] 4. Validar YAML (PyYAML parsea los 3) + contrato POST → `OK`
- [x] 5. Commit local `ci(workflows): ...` (FIND-466) — sin push

## Contrato (verificación mecánica)
python -c "import yaml; g=lambda p:(lambda d: d.get('on', d.get(True)))(yaml.safe_load(open(p,encoding='utf-8'))); a=set(g('.github/workflows/ci-rust.yml')['pull_request']['paths']); assert {'release-plz.toml','release-manifest.json','.github/workflows/ci-rust.yml'} <= a, sorted(a); b=set(g('.github/workflows/docs-gates.yml')['pull_request']['paths']); assert 'release-manifest.json' in b, sorted(b); c=set(g('.github/workflows/ci-fleet.yml')['pull_request']['paths']); assert 'integrations/**' in c, sorted(c); print('OK')"

## Riesgos (pre-mortem)
- Glob amplio `.github/workflows/**` en ci-rust dispararía la matriz Rust al editar cualquier workflow → NO usar; acotar a los ficheros enumerados.
- Indentación YAML incorrecta invalida el trigger → validar parse.
- No modificar `on:` por YAML 1.1 (`True` key) — el contrato ya lo resuelve.

## SDP
SKILLS: ci-cd-and-automation, doubt-driven-development, shipping-and-launch (MCP campaign_* no disponible en este entorno — SDP manual por keywords del contrato)

## Review (P2-01)
- Tier: fast (paths `.github/workflows/**` = CI, no globs adversariales)
- Veredicto: approve (vanta-review, contexto fresco; 1 hallazgo required -> cliff.toml corregido inline y re-verificado)

## Context Save Point
Trabajo parcial = 3 YAML editados + este task file. Si se interrumpe: releer baseline
de este archivo, re-ejecutar contrato, retomar desde el primer step ⬜.
