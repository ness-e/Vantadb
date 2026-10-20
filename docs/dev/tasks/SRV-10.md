---
title: Task SRV-10 — Cifrado server honesto + CI-08/FIND-321
kind: task
status: in_progress
wave: 0
---

# SRV-10 + CI-08 / FIND-321 — Task file (plan Task 11)

> Idioma: español (planning). Docs técnicas que se toquen van en inglés.

## Objetivo

(a) **SRV-10**: cifrado en reposo del server HTTP — el core tiene feature
`encryption`; la capa `vantadb-server` debía exponerla
(`docs/api/HTTP_API.md` lo promete como roadmap).
(b) **CI-08**: consolidar workflows 40→≤18 en estructura
CI/Certificación/Release (cada eliminado migra sus jobs o documenta su muerte);
antes reconciliar inventario **FIND-321** (`CI_POLICY.md` declaraba 28, hay 40).
Siguiente: EGO-01 (Wave 0, sin dependencias bloqueantes).

## SDP

`SDP: codebase-memory (blast radius server↔core) · progreso (lectura Backlog/avance) · ponytail full (slice mínimo verificable) + server-mcp/release-ci rules como referencias obligatorias`
`SKILLS_CARGADAS: codebase-memory, progreso, ponytail`

## Discovery (2026-10-15, branch `develop`)

### SRV-10 — YA RESUELTO, no reimplementar

- Commits en historia: `ca3f00da feat(server): SRV-10 — cifrado en reposo
  honesto: aviso key-sin-feature, feature opt-in encryption, docs sin claim
  (re-scope b; wiring=FIND-249)` + `8f758e24 docs(avance): SRV-10 registrado`.
- Evidencia en árbol:
  - `src/server/bootstrap.rs:127-159` — `encryption_key_ignored_notice()` /
    `warn_if_encryption_key_ignored()` (`#[cfg(not(feature = "encryption"))]`,
    aviso key-sin-feature).
  - `vantadb-server/Cargo.toml:66-70` — feature `encryption =
    ["vantadb/encryption"]` (opt-in, comentario SRV-10/FIND-249/ENC-01).
  - `src/lib.rs:55-56` — primitivas AES-256-GCM tras feature, wiring pendiente
    (FIND-249, dueño del wiring real a write paths).
  - `docs/api/HTTP_API.md:772-776` — "No encryption at rest", honesto, sin claim.
- Veredicto: alcance (a) = **verificado pre-existente**. E2e+security de
  `-p vantadb-server` en verde corresponde al cierre original (`ca3f00da`);
  no se re-corrió (budget ~10 calls, builds ≥600s fuera de presupuesto —
  deuda explícita para el orquestador si quiere re-verificar).

### FIND-321 — reconciliado en este turno

- `docs/dev/operations/CI_POLICY.md:16` declaraba 28 → **40 verificados**
  (`Get-ChildItem .github/workflows/*.yml` = 40, lista completa en evidencia).
- `docs/dev/workflow/CI_POLICY.md` **NO EXISTE** (solo el de operations) —
  el prompt la listaba; no crear por disciplina de alcance.
- `docs/dev/workflow/TRIGGERS.md` cubre **40/40** ficheros (chequeo mecánico
  nombre-por-nombre contra `on:`-owner). Consistente con los `on:` ✅.
- Reglas duras workflows (muestra `ci-ai-ides-demo.yml:22-25`): `push:
  branches: ["main"]` + `paths`, sin `continue-on-error` ✅ (patrón Regla 1).

### CI-08 — planificado, NO ejecutado (requiere Gate D + budget propio)

- Estructura objetivo: CI / Certificación / Release con jobs múltiples.
- Candidatos naturales a fusión (sin tocar publish): 5 demos
  (`ci-ai-ides-demo`, `ci-frameworks-demo`, `icp02-privacy-demo`,
  `injection-governance-demo`, `wal-verify-demo`) → 1 `demo-acceptance.yml`
  con 5 jobs; 4 informational
  (`arch-metrics-informational`, `bench-canonical-p99-informational`,
  `lurkr-informational`, + `ocr-*`?) → `nightly.yml` jobs.
- **PROHIBICIÓN PERMANENTE respetada**: `release.yml`,
  `release-npm-61.yml`, `release-npm-node.yml` INTACTOS (OIDC Trusted
  Publishing atado al filename); tag-namespaces de `PUBLISH.md` intactos.
- Reglas duras por job migrado: `timeout-minutes`, SHAs pineados,
  `permissions` mínimas, `continue-on-error` solo con `CATEGORY`.

## Impacto mapeado (Regla 0)

- **Leídos completos**: `.opencode/rules/server-mcp.md`,
  `.opencode/rules/release-ci.md`, `RULES.md` (§Excepción filenames + Regla 1),
  Backlog §SRV-10/§CI-08/§FIND-321, `TRIGGERS.md` (matriz), `CI_POLICY.md`
  (operations, §inventario), `HTTP_API.md:760-790`, `server.rs` (re-export
  `cli_server`), `vantadb-server/Cargo.toml:55-80`.
- **Referencias hacia dentro**: server → `vantadb/encryption` (feature),
  `src/server/bootstrap.rs` → `Config.encryption_key`; CI_POLICY →
  `TRIGGERS.md`/`README.md` (workflow).
- **Referencias entrantes**: workflows ← `PUBLISH.md`, `RUNBOOK.md`;
  `encryption` ← `src/storage/engine/init.rs` (ENC-01).
- **Veredicto**: 1 archivo tocado (`CI_POLICY.md`, 1 línea, docs-only).
  Sin paths multi-índice/Tokio/`dashmap` tocados → Regla 8 no dispara.
  `vantadb-python/**` sucio en árbol = WIP ajeno, NO tocado. Sin stash,
  sin commit, sin push.

## Steps

- [x] **Step 1 — DISCOVERY + FIND-321** (este turno): blast radius
      (server↔core encryption + matriz TRIGGERS), inventario 40 verificado,
      TRIGGERS 40/40, 1 línea CI_POLICY.md. ✅
- [x] **Step 2 — CI-08 slice 1** (este turno): 5 demos →
      `demo-acceptance.yml` (jobs 1:1 mismos IDs/steps/SHAs/timeouts,
      triggers = unión de paths, TRIGGERS.md actualizado). ✅
- [x] **Step 3 — CI-08 slice 2** (este turno): 4 informational → jobs
      `nightly.yml` (heavy gated `!= PR`, info gated `PR/dispatch`,
      `issues:write` job-level en bench jobs, CATEGORY tags añadidos en
      arch-metrics, TRIGGERS.md actualizado). ✅
      40 → 32 ficheros. `actionlint`: ausente en runner → validación por
      `py-yaml` + inspección + `git diff --check` (ver validación abajo).
- [x] **Step 4 — CI-08 slice 3** (este turno): resto 32→18
      con tabla de migración (ejecutada 1:1 + dedup `ci-gate` ×3→1).
      4 nuevos: `docs-gates.yml` (17 jobs), `ci-fleet.yml` (5),
      `certification.yml` (21), `platform-ops.yml` (8). Jobs verbatim
      + gates `if: event_name` que preservan triggers origen + fixes
      requeridos (bench perms write/issues job-level; opencode concurrency
      FIND-146 a nivel job; fuzz `max_seconds` required→default).
      `TRIGGERS.md` + `README.md` reescritos a 18. ✅ (SIN commit/SIN push —
      review+commit = orquestador)
- [ ] **Step 5 — Re-verificar SRV-10** (opcional orquestador):
      `cargo nextest -p vantadb-server` e2e+security en verde.

## Context Save Point (2026-10-15, Steps 2-4 ejecutados, SIN commit/SIN push)

- Rama `develop`; WIP ajeno intacto (`vantadb-python/**`, `src/cli_handlers/*`,
  `tests/*`, `docs/user/QUICKSTART.md`, `Justfile`, `completions/*`,
  `docs/dev/operations/CI_POLICY.md`, plan pre-mercado) — no tocado.
- Este turno crea (untracked, sin stagear): `docs-gates.yml`, `ci-fleet.yml`,
  `certification.yml`, `platform-ops.yml`, `demo-acceptance.yml` (Step 2);
  modifica (unstaged): `nightly.yml`, `TRIGGERS.md`, `README.md`, `SRV-10.md`.
  27 ficheros origen eliminados del worktree (9 Steps 2-3 + 18 Step 4,
  deletions UNSTAGED — `git diff --cached` vacío; el `git rm`/stage lo hace
  el review). Helper `dev-tools/ci08-step4.py`: scaffolding del turno —
  ELIMINADO al cierre (migración documentada acá; re-ejecutable vía git).
- `actionlint`: **NO-VERIFICADO** (binario ausente en runner — verificado
  `Get-Command` vacío también en Step 4).
- Deuda: re-run tests `-p vantadb-server` si se exige contrato fresco;
  gate mecánico de conteo (= 18, FIND-321 ideal); branch protection
  (checks renombrados — ver BLOQUEO, lista ampliada Step 4).
- Deltas de comportamiento asumidos (documentados en headers de cada nuevo
  workflow): (1) jobs schedule-gated corren en TODOS los crons del archivo
  (GitHub no distingue cron) — resto queda skipped; (2) `certification`
  PR sin `branches` (fuzz verbatim, RULES §7 follow-up); (3) codeql push/PR
  acotado a paths desktop (docs-only ya no lo dispara); (4) cancel-in-progress
  estandar salvo opencode (FIND-146 preservado); (5) `max_seconds` dispatch
  required→default '60'; (6) fuzz no corre en push, chaos no en schedule
  (nunca lo hacían — gates explícitos).
- NOTICED BUT NOT TOUCHING: `lurkr` header cita promoción a gate
  obligatorio el 2026-10-08 (fecha pasada) — decisión owner, no de este turno;
  `ci-rust.yml:635` + 3 `release-*` con `continue-on-error` sin tag CATEGORY
  en ≤2 líneas (fuera del set Step 4 — no verificado si el tag está más
  arriba); `README.md` fila `ci-web.yml` stale eliminada en rewrite (archivo
  borrado en `82317140`); conteo real de jobs nuevos = 51 (53 origen − 2
  dedup `ci-gate`).

## Tabla de migración Step 4 (32→18; release/* INTACTOS, 9 ficheros)

| Origen (se elimina) | Destino | Jobs migran |
|---|---|---|
| `gate-docs.yml`, `gate-docs-links.yml`, `gate-docs-secrets.yml`, `gate-api-docs.yml`, `gate-doc-examples.yml`, `ci-rustdoc.yml` | `docs-gates.yml` (nuevo, 6→1) | 1:1; preservar dual-PR `develop` + schedules (docs-links Mon 07:23) |
| `ci-examples.yml`, `ci-bindings-parity.yml`, `adapters-compat.yml`, `providers-ci.yml` | `ci-fleet.yml` (nuevo, 4→1) | 1:1; preservar schedule adapters-compat (Sun 03:00) |
| `heavy-certification.yml`, `heavy-bench-nightly.yml`, `chaos.yml`, `fuzz.yml` | `certification.yml` (nuevo, 4→1) | 1:1; preservar schedules (bench 02:00, cert Sun 03:00, fuzz Mon 06:00) + PR paths fuzz |
| `desktop.yml`, `sec-codeql.yml`, `opencode.yml`, `ocr-nightly.yml` | `platform-ops.yml` (nuevo, 4→1) | 1:1; preservar `issue_comment` (opencode), schedule ocr 04:00, branches-only codeql |
| — (se mantienen) | `ci-rust.yml`, `ci-gate.yml`, `demo-acceptance.yml`, `nightly.yml`, `perf-bench.yml` (excepción develop-push) + 9 release/* | — |

## Validación (Step 4 — VERIFY de este turno)

- `actionlint`: ausente en runner (`Get-Command` vacío) → NO-VERIFICADO,
  sustituido por parser `py-yaml` (4/4 nuevos parsean OK) + inspección
  rigurosa + `git diff --check` limpio.
- Checker propio (`dev-tools/ci08-step4.py`, eliminado post-ejecución):
  **PASS** — todo job con `runs-on` tiene `timeout-minutes` (jobs `uses:`
  reusable exentos); todo `continue-on-error: true` con `# CATEGORY:` en
  ±6 líneas; 0 colisiones de job IDs salvo `ci-gate` ×3 (verificado
  idéntico, dedup a 1).
- Triggers verificados contra headers `on:` reales de los 18 orígenes
  (dump completo en historial del turno): schedules preservados (docs-links
  lun 07:23, adapters dom 03:00, bench 02:00 diario, cert dom 03:00, fuzz
  lun 06:00, ocr 04:00 diario, codeql dom 00:00), `issue_comment` /
  `review_comment` (opencode), branches-only codeql → acotado (delta #3).
- Conteo: **18** (`Get-ChildItem` post-rm). `release*.yml` (9): intactos —
  mismos filenames, sin cambios de triggers (verificado en status: solo
  los 18 orígenes figuran ` D`, 0 `M` en release/*). tag-namespaces y
  publish paths: no tocados (PROHIBICIÓN PERMANENTE ✅).
- `git diff --cached` vacío (nada stageado — el stage es del review).
  WIP ajeno intacto. TRIGGERS.md 18/18 + README.md reescrito a 18.
- Steps 2-3 (contexto heredado): su validación vive abajo sin cambios.

## Validación (turno continuación — Steps 2-3 VERIFY completo, heredado)

- `actionlint`: ausente en runner → NO-VERIFICADO, sustituido por inspección
  rigurosa + parser (`py-yaml` OK, node v26.8.1).
- Checker mecánico propio (timeouts §2, CATEGORY §6, SHA-pins §3, secrets §4,
  permissions §4): **PASS** en `demo-acceptance.yml` + `nightly.yml`.
  Hallazgos corregidos en el turno: (1) `arch-metrics` sin `timeout-minutes`
  (heredado del original — añadido `timeout-minutes: 30`); (2) 22
  `continue-on-error` bare + línea lurkr sin literal `CATEGORY` → tag
  `# CATEGORY: INFORMATIONAL` añadido (cero cambio de comportamiento);
  (3) bug del checker (`on:` parsea como `True` en YAML 1.1 — solo test).
- Triggers verificados contra `on:` reales: demo `push/PR/dispatch`
  (17 paths unión, self-ref OK, 5 jobs IDs originales); nightly
  `schedule/PR/dispatch` (7 paths, self-ref OK; 8 heavy con `if != PR`,
  5 info con `if PR/dispatch`, `notify-failure` sin info en `needs`).
- `git diff --check`: limpio. Conteo: **40 → 32**. `release*.yml`: intactos
  (status confirma, sin renames ni trigger changes). WIP ajeno intacto.
- TRIGGERS.md + README.md consistentes con cada `on:` (matriz y tablas).
- Stale noting (NO tocado, Step 4): comentario `ocr-nightly.yml:16`
  ("mirror ocr-delegate.yml"); sección demos nunca existió en README
  (Step 4 reescribe inventario al llegar a ≤18).

## BLOQUEO review (orquestador, antes del commit)

1. Branch protection: checks renombrados — actualizar required checks
   (acción ADMIN pre-merge, solo registrada): `5 demos/*` → `demo-acceptance/*`,
   `4 informational/*` → `nightly/*` (Steps 2-3) + Step 4: `gate-docs/*`,
   `gate-docs-links/*`, `gate-docs-secrets/*`, `gate-api-docs/*`,
   `gate-doc-examples/*`, `ci-rustdoc/*` → `docs-gates/*`;
   `ci-examples/*`, `ci-bindings-parity/*`, `adapters-compat/*`,
   `providers-ci/*` → `ci-fleet/*`; `heavy-certification/*`,
   `heavy-bench-nightly/*`, `chaos/*`, `fuzz/*` → `certification/*`;
   `desktop/*`, `sec-codeql/*`, `opencode/*`, `ocr-nightly/*` →
   `platform-ops/*`.
2. Stage + commit: `git add` SOLO del set CI-08 (4 nuevos + 18 deletes +
   `nightly.yml` + `demo-acceptance.yml` + `TRIGGERS.md` + `README.md` +
   `SRV-10.md`) — NO stagear `vantadb-python/**` (WIP ajeno) ni nada más;
   `git diff --check` + conteo (= 18) + `gh run list` en la primera PR
   `develop → main` (1 run por workflow cuando paths matchean).

## Evidencia

- Workflows (40): adapters-compat, arch-metrics-informational,
  bench-canonical-p99-informational, chaos, ci-ai-ides-demo,
  ci-bindings-parity, ci-examples, ci-frameworks-demo, ci-gate, ci-rust,
  ci-rustdoc, desktop, fuzz, gate-api-docs, gate-doc-examples,
  gate-docs-links, gate-docs-secrets, gate-docs, heavy-bench-nightly,
  heavy-certification, icp02-privacy-demo, injection-governance-demo,
  lurkr-informational, nightly, ocr-delegate, ocr-nightly, opencode,
  perf-bench, providers-ci, release-adapters, release-binaries,
  release-npm-61, release-npm-node, release-providers, release-sbom,
  release-verify, release-wheels, release, sec-codeql, wal-verify-demo.

## Review (Gate P2-01 — Step 4)
- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-15, re-ejecución con `yaml.safe_load` + `Select-String`).
- **Veredicto:** 🔴 CHANGES-REQUIRED inicial → ✅ APPROVE tras 7 fixes aplicados por orquestador (mismo turno): `certification.yml:701,725` if truncado por `#` inline → comentario arriba + condición limpia; `platform-ops.yml:281-285` `if: (|)` opencode → expresión completa desde `HEAD:opencode.yml`; `demo-acceptance.yml:128` pin maturin 37-hex → 40-hex origen; 4 `if:` con `${{ }}` anidados → forma simple (inputs `include_sift/include_competitive/enable_critcmp` verificados definidos en `workflow_dispatch`, opt-in intencional). Re-verificado: patrones ausentes + YAML 18/18 + diff --check limpio.
- **Step 5** (re-verificar SRV-10, opcional): cubierto — código server sin cambios en este task (verificado pre-existente por worker1); no aplica.
- **Estado:** ✅ COMPLETED (Steps 2-4 + review + fixes).
