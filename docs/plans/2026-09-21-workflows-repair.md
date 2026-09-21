# Plan de Ejecución: Reparación integral de workflows (28 archivos) — 2026-09-21

> **Campaign ID:** (asigna `campaign_get_next_task` al arrancar)
> **Inicio:** 2026-09-21
> **Estado:** ⬜ PENDIENTE (plan listo, sin iniciar)
> **Fuente:** auditoría 2026-09-21 (3× `vanta-review` en paralelo, 28/28 archivos leídos)
> + validación internet (duplicados push/PR, SHA-pinning 2026, sccache oficial)

## Decisiones del owner (vinculantes)

1. Alcance: reparar/optimizar, **cero refactors** de CI (sin migrar a merge-queue ni reescribir pipelines).
2. Publishing: cambios con cuidado extra (rollback `git revert` por commit + re-run del workflow).
3. Reglas durables van a `docs/workflow/RULES.md` (trackeado). NO a `.opencode/` (local-only desde C-01)
   ni duplicadas en `AGENTS.md` raíz (su cabecera lo prohíbe; solo 1 fila puntero en su tabla).
4. Renombres de archivos al FINAL (tras todos los edits) para no colisionar.

## Resumen

13 tasks FIND-134..146 en 6 waves. Puerta: todo mergeado a `main` (#202 ✅ `bcc92693`).
-Principio: 1 archivo = 1 dueño por wave (sin ediciones concurrentes al mismo file).

## Wave 0 — Seguridad y costo directo (paralelo ×3, archivos disjuntos)

- [ ] FIND-134 · 🟡 · `ci-rust-10.yml` hardening: quitar `develop` de `push.branches` (dedup),
  `needs: [fmt, clippy]` como fast-fail antes de jobs 30-60min, política por cada
  `continue-on-error` (quitar o tag CATEGORY justificado), condición `wasm-test` válida en PR.
  Contrato: 1 push → 1 run; fmt rojo frena lo pesado; `actionlint` 0. Commit `ci: FIND-134 — ...`
- [ ] FIND-135 · 🟢 · `timeout-minutes` a jobs sin límite: `desktop.yml` ×3, `ci-gate.yml`,
  `ocr-delegate.yml`, `ocr-nightly.yml` (3 jobs), `opencode.yml` (valores = duración real + margen).
  Contrato: ningún job sin timeout; `actionlint` 0. Commit `ci: FIND-135 — ...`
- [ ] FIND-136 · 🟡 · Cobertura + caché: `providers-ci.yml` (`branches: [main, develop]`,
  toolchain SHA = resto), `chaos-45.yml` (+paths `vantadb-*/**`, `vanta-memory/**`, +concurrency),
  `ci-examples-12.yml` (`cache: pip`), `adapters-compat.yml` (caché Rust/pip, quitar `|| true`).
  Contrato: cambio en providers dispara clippy/tests; sin `|| true`; `actionlint` 0. Commit `ci: FIND-136 — ...`

## Wave 1 — Docs builds y gates (paralelo ×3, disjuntos; tras Wave 0)

- [ ] FIND-137 · 🟢 · Unificar rustdocs: `ci-rustdoc.yml` vs `rustdoc-70.yml` → uno solo
  (mismo toolchain/flags/artifact/trigger); actualizar badges del README si citan el eliminado.
  Contrato: 1 workflow rustdoc, 0 refs rotas. Commit `ci: FIND-137 — ...`
- [ ] FIND-138 · 🟢 · Diagnosticar flake "Generate API reference 0s" (2 muertes sin log) y fix
  (retention/cancel-concurrency o causa real). Contrato: 3 runs verdes seguidos o causa declarada.
  Commit `ci: FIND-138 — ...` (o solo task file si es infra externa + deuda escrita)
- [ ] FIND-139 · 🟢 · `ci-gate.yml` (timeout, concurrency, `*)` ya no traga pending/not-found,
  REQUIRED incluye semver/ADR) + `gate-docs-21.yml` (PR→develop también). Contrato: gate rojo
  cuando un check falta; `actionlint` 0. Commit `ci: FIND-139 — ...`

## Wave 2 — Publishing con cuidado extra (paralelo ×3, disjuntos; lead supervisa)

- [ ] FIND-140 · 🔴 · Publishing hardening (1 task, steps atómicos por archivo, NADA sin re-run):
  `release.yml` push solo `main` + timeout/env/concurrency en `release-plz-release`;
  `release-npm-61.yml` + `release-npm-node.yml` triggers separados tags-vs-push (+PR trigger a node);
  `release-adapters-62.yml` gate `version-exists` en prod; `release-binaries-63.yml` alinear
  trigger con condición de upload; namespaces de tags documentados. Contrato: publish solo donde
  debe; cada cambio con re-run verde. Commits `ci: FIND-140 — ...` (uno por archivo si hace falta)
- [ ] FIND-141 · 🟡 · Schedules: desolapar `heavy-cert-50` vs `heavy-bench-nightly-51` (03:00),
  `[skip ci]` o guard en auto-push de baseline, `retention-days` a corpus fuzz.
  Contrato: 0 solapes; `actionlint` 0. Commit `ci: FIND-141 — ...`
- [ ] FIND-146 · 🟢 · Pins: `arch-metrics-informational.yml` (`checkout@v4`→SHA), OCR (`@v4`→SHA,
  `npm install -g latest`→versión fija), guard doble-run en `opencode.yml`.
  Contrato: 0 tags móviles en workflows; `actionlint` 0. Commit `ci: FIND-146 — ...`

## Wave 3 — Renombres (SOLO tras Waves 0-2 verdes; 1 task, último edit de archivos)

- [ ] FIND-142 · 🟡 · Quitar numeración (`ci-rust-10.yml`→`ci-rust.yml`, etc.) vía `git mv`
  + badges del README en el mismo commit. Contrato: `actionlint` 0 + checks requeridos resuelven
  con nuevos nombres + badges 200. Commit `ci: FIND-142 — ...`

## Wave 4 — Documentación (tras código estable; paralelo ×2)

- [ ] FIND-143 · 🟢 · `docs/workflow/` (vanta-docs): inventario 28 + matriz triggers + flujo de
  publish por registro + runbook (re-run, approve environments, [no-adr]) + FAQ duplicados.
  Contrato: lint/frontmatter/coverage verdes; 0 links rotos. Commit `docs: FIND-143 — ...`
- [ ] FIND-145 · 🟢 · Verificar CodeQL verde post-#202 en `main`; si sigue rojo, fix o deuda escrita.
  Contrato: check verde o causa declarada con dueño. Commit `ci: FIND-145 — ...` (o task file)

## Wave 5 — Reglas durables (tras FIND-143)

- [ ] FIND-144 · 🟢 · `docs/workflow/RULES.md` (triggers, timeouts, pins, permissions, publish,
  anti-patrones con ejemplo bueno/malo) + refresh `CI_POLICY.md` (26→28, triggers reales) +
  1 fila puntero en tabla de `AGENTS.md` raíz. Contrato: lint verdes; regla verificable por cada
  hallazgo Alta. Commit `docs: FIND-144 — ...`

## Cierre (orquestador)

P2-01 batch (revisor distinto) → `skill progreso` (Backlog −13, avance `ci-cd.md`) →
retrospectiva + archive + nota `meta.md`. WIP ajeno intacto siempre; `cargo -j 2`;
staging selectivo; NO PUSH sin verify (pushea solo vanta-lead).

## Gates

- **Gate P:** owner aprobó auditoría→plan (2026-09-21) + publishing con cuidado extra.
- **Stop honesto:** si un fix exige rediseño (merge-queue, reescribir release) → DEFER con
  diagnóstico, no refactors. Si un publish se comporta distinto a lo documentado → STOP + owner.
- **Appetite / Branch / Commit:** 3d / develop / conventional con ID (`ci:`/`docs:`).

## Deuda conocida (no bloquea)

- Rustdoc-flake puede ser infra externa (FIND-138 lo dirá).
- Benchmarks pesados informativos fuera del critical path (ya acordado C-06).
- #187 (release v0.6.1) vive su propio ciclo; este plan no la toca.

## Fuentes

- Auditoría: 3 reportes `vanta-review` 2026-09-21 (CI-core 10 files, release 9+release-plz.toml, gates-heavy 9+CI_POLICY).
- Internet: duplicados push/PR ([discusión](https://github.com/orgs/community/discussions/57827),
  [SO](https://stackoverflow.com/questions/74441152/how-to-avoid-the-duplicated-job-in-github-action-like-build-image),
  [skip-duplicate-actions](https://github.com/marketplace/actions/skip-duplicate-actions));
  SHA-pinning 2026 ([1](https://www.romainlespinasse.dev/posts/github-actions-commit-sha-pinning/),
  [2](https://dsotn.com/articles/pin-github-actions-sha/));
  sccache ([README oficial](https://github.com/mozilla-actions/sccache-action)).
  Capas keyless/Argus/Firecrawl caídas o sin proveedores → validado vía MetaSearch+webfetch.

=== RECITATION ===
Objetivo activo: PLAN workflows-repair — plan creado
Estado: plan (13 DO en 6 waves + cierre)
Última acción: plan creado desde auditoría 28/28 + validación internet + decisiones owner
Resultado: ✅
Próxima acción: `/pipeline run docs/plans/2026-09-21-workflows-repair.md` (Wave 0: FIND-134/135/136)
Contrato: plan file existe con waves disjuntas + contratos + gates; task files bajo demanda
Próxima tarea: FIND-134 + FIND-135 + FIND-136 (paralelo ×3)
last-synced: 2026-09-21
=== END RECITATION ===
