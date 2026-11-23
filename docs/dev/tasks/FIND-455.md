---
id: FIND-455
title: Gate mecanico de ancestro origin/main en PRs hacia main
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
state: in-progress
type: ci-gate
---

# FIND-455 — Gate de ancestro origin/main en PRs hacia main

## Objetivo
Ningun job verifica que `origin/main` sea ancestro de `HEAD` cuando un PR apunta a
`main`. Sin ese gate, un `develop -> main` puede revertir hotfixes que solo viven
en `main` (higiene post-release: sincronizar `main -> develop`, .opencode/AGENTS.md).
Acceptance: job con `fetch-depth: 0` que ejecuta
`git merge-base --is-ancestor origin/main HEAD`, acotado a PRs cuya base es `main`.

## Contrato (verify mecanico)
```
python -c "import yaml; d=yaml.safe_load(open('.github/workflows/ci-rust.yml',encoding='utf-8')); hits=[(jn,j) for jn,j in d['jobs'].items() for s in j.get('steps',[]) if 'merge-base --is-ancestor' in (s.get('run') or '')]; assert hits, 'sin gate de ancestro'; ...; print('OK')"
```
Baseline ANTES: `AssertionError: sin gate de ancestro` (confirmado 2026-11-07).

## Causa raiz / premisa corregida
- `rg "merge-base --is-ancestor" ci-rust.yml` -> sin match: gate inexistente.
- `git rev-list --count origin/main ^HEAD` -> 0: NO hay deriva activa hoy.
  El gate es PREVENTIVO, no correctivo.

## Decisiones de diseno
- Job nuevo `main-ancestry-gate` (no acoplar a `secret-scan`, cuyo fetch-depth:0
  de la linea 83 es de otro job y no debe cargar mas responsabilidades).
- `if`: `github.event_name == 'pull_request' && github.base_ref == 'main' &&
  !startsWith(github.head_ref, 'release-plz-')`.
  - `github.base_ref` solo existe en eventos pull_request (docs.github.com
    contexts) -> hay que guardar por event_name.
  - Exencion `release-plz-*`: release-plz rebasa su PR en cada corrida y la rama
    solo toca version/CHANGELOG; coherente con el patron `^release-plz-[0-9]{4}-`
    de FIND-467 en release.yml (ramas `release-plz-<ISO timestamp>`).
- `fetch-depth: 0` obligatorio: con clone shallow `merge-base` falla siempre.
- `git fetch --no-tags origin main` explicito: el checkout de PR solo garantiza
  el ref del merge; asegura que `origin/main` exista antes de comparar.
- Exit codes (git-scm.com/docs/git-merge-base): 0 = ancestro, 1 = no ancestro,
  otro != 0 = error -> el step falla en ambos casos no-ancestro/error.

## Impacto mapeado (Regla 0)
- Archivos leidos: `.github/workflows/ci-rust.yml` (completo, 23 jobs),
  `docs/dev/workflow/RULES.md` (secciones CI), `release.yml` (patron FIND-467).
- Referencias entrantes: ningun otro archivo referencia el job nuevo.
- Blast radius: ALTO funcional (un gate mal configurado bloquea TODOS los PRs a
  main, incluido el release PR de v0.10.0) -> mitigado por exencion release-plz
  y scope a pull_request/base=main. BAJO en codigo (1 job ~30 lineas).
- No deshacer cambios de FIND-466 (paths:) ni FIND-392 (-p/--locked en
  test/test-windows/test-macos).

## Steps
- [x] DISCOVERY: baseline contrato (falla), premisa verificada, docs oficiales
- [x] Diseno del job `main-ancestry-gate`
- [x] ACT: insertar job en ci-rust.yml
- [x] VERIFY: contrato OK + YAML valido + demo exit 0/!=0
- [x] Review P2-01 (fresh reviewer, foco: no bloquear release-plz)
- [x] Commit local `ci(guard): ... (FIND-455)`

## SDP
SKILLS_CARGADAS: ci-cd-and-automation, git-workflow-and-versioning
(fijas del rol lead para CI/CD + branching; MCP campaign_* no disponible,
SDP manual segun instruccion del orquestador)

## Fuentes
- https://git-scm.com/docs/git-merge-base (exit codes --is-ancestor)
- https://docs.github.com/en/actions/reference/workflows-and-actions/contexts
  (github.base_ref solo en pull_request/pull_request_target)

## Post-review (P2-01, fresh context)
- CHANGES-REQUIRED ronda 1: el checkout default (refs/pull/N/merge) hacia el
  gate tautologico -> fix: `ref: ${{ github.event.pull_request.head.sha }}`.
- Gap de paths: del trigger documentado como scope aceptado (solo ci-rust.yml
  editable; un workflow dedicado sin paths: cerraria el caso docs/scripts).
- Re-review: APPROVE. Deuda: gate no corre en PRs a main sin cambios en paths:
  listados (riesgo bajo; develop->main siempre toca codigo).
