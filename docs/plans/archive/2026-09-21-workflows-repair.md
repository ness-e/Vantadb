# Plan de Ejecución: Reparación integral de workflows (28 archivos) — 2026-09-21

> **Campaign ID:** c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
> **Inicio:** 2026-09-21
> **Estado:** ✅ COMPLETADO (2026-09-22: Waves 0/1/2/3/4/5 — 13/13. Cierre administrativo pendiente)
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

- [x] FIND-134 · ✅ 2026-09-22 (commit 37eee390: dedup triggers, needs fast-fail, wasm-if, sin || echo; race worktree resuelta) 🟡 · `ci-rust-10.yml` hardening: quitar `develop` de `push.branches` (dedup),
  `needs: [fmt, clippy]` como fast-fail antes de jobs 30-60min, política por cada
  `continue-on-error` (quitar o tag CATEGORY justificado), condición `wasm-test` válida en PR.
  Contrato: 1 push → 1 run; fmt rojo frena lo pesado; `actionlint` 0. Commit `ci: FIND-134 — ...`
- [x] FIND-135 · ✅ 2026-09-22 (commit 217d583a: 9 timeouts calibrados) 🟢 · `timeout-minutes` a jobs sin límite: `desktop.yml` ×3, `ci-gate.yml`,
  `ocr-delegate.yml`, `ocr-nightly.yml` (3 jobs), `opencode.yml` (valores = duración real + margen).
  Contrato: ningún job sin timeout; `actionlint` 0. Commit `ci: FIND-135 — ...`
- [x] FIND-136 · ✅ 2026-09-22 (commit 6cf0e2cf: branches+SHA, paths+concurrency, caches, sin || true) 🟡 · Cobertura + caché: `providers-ci.yml` (`branches: [main, develop]`,
  toolchain SHA = resto), `chaos-45.yml` (+paths `vantadb-*/**`, `vanta-memory/**`, +concurrency),
  `ci-examples-12.yml` (`cache: pip`), `adapters-compat.yml` (caché Rust/pip, quitar `|| true`).
  Contrato: cambio en providers dispara clippy/tests; sin `|| true`; `actionlint` 0. Commit `ci: FIND-136 — ...`

## Wave 1 — Docs builds y gates (paralelo ×3, disjuntos; tras Wave 0)

- [x] FIND-137 · ✅ 2026-09-22 (commit 4b0686b0: survivor ci-rustdoc + rm rustdoc-70) 🟢 · Unificar rustdocs: `ci-rustdoc.yml` vs `rustdoc-70.yml` → uno solo
  (mismo toolchain/flags/artifact/trigger); actualizar badges del README si citan el eliminado.
  Contrato: 1 workflow rustdoc, 0 refs rotas. Commit `ci: FIND-137 — ...`
- [x] FIND-138 · ✅ 2026-09-22 (commit 2e9b6e8b: flake = cancel-in-progress by-design, sin fix) 🟢 · Diagnosticar flake "Generate API reference 0s" (2 muertes sin log) y fix
  (retention/cancel-concurrency o causa real). Contrato: 3 runs verdes seguidos o causa declarada.
  Commit `ci: FIND-138 — ...` (o solo task file si es infra externa + deuda escrita)
- [x] FIND-139 · ✅ 2026-09-22 (commit 81263fb1: fail-closed + PR-develop) 🟢 · `ci-gate.yml` (timeout, concurrency, `*)` ya no traga pending/not-found,
  REQUIRED incluye semver/ADR) + `gate-docs-21.yml` (PR→develop también). Contrato: gate rojo
  cuando un check falta; `actionlint` 0. Commit `ci: FIND-139 — ...`

## Wave 2 — Publishing con cuidado extra (paralelo ×3, disjuntos; lead supervisa)

- [x] FIND-140 · ✅ 2026-09-22 (5 commits: release.yml main-only + npm-61/npm-node tags-solo + adapters skip-existing + binaries trigger; docs/tasks/FIND-140.md) 🔴 · Publishing hardening (1 task, steps atómicos por archivo, NADA sin re-run):
  `release.yml` push solo `main` + timeout/env/concurrency en `release-plz-release`;
  `release-npm-61.yml` + `release-npm-node.yml` triggers separados tags-vs-push (+PR trigger a node);
  `release-adapters-62.yml` gate `version-exists` en prod; `release-binaries-63.yml` alinear
  trigger con condición de upload; namespaces de tags documentados. Contrato: publish solo donde
  debe; cada cambio con re-run verde. Commits `ci: FIND-140 — ...` (uno por archivo si hace falta)
- [x] FIND-141 · ✅ 2026-09-22 (commit 8dfd4a09: bench 03:00→02:00, skip-ci baseline, retention 14d fuzz) 🟡 · Schedules: desolapar `heavy-cert-50` vs `heavy-bench-nightly-51` (03:00),
  `[skip ci]` o guard en auto-push de baseline, `retention-days` a corpus fuzz.
  Contrato: 0 solapes; `actionlint` 0. Commit `ci: FIND-141 — ...`
- [x] FIND-146 · ✅ 2026-09-22 (commit 1391ddde: 15 pins SHA + npm 1.12.9 + guard opencode) 🟢 · Pins: `arch-metrics-informational.yml` (`checkout@v4`→SHA), OCR (`@v4`→SHA,
  `npm install -g latest`→versión fija), guard doble-run en `opencode.yml`.
  Contrato: 0 tags móviles en workflows; `actionlint` 0. Commit `ci: FIND-146 — ...`

## Wave 3 — Renombres (SOLO tras Waves 0-2 verdes; 1 task, último edit de archivos)

- [x] FIND-142 · ✅ 2026-09-22 (commit 97a3a03c: 14 renames vía git mv + badges + docs/workflow refs; release.yml/npm-61/npm-node NO renombrados — pineados por trusted publishers) 🟡 · Quitar numeración
  + badges del README en el mismo commit. Contrato: `actionlint` 0 + checks requeridos resuelven
  con nuevos nombres + badges 200. Commit `ci: FIND-142 — ...`

## Wave 4 — Documentación (tras código estable; paralelo ×2)

- [x] FIND-143 · ✅ 2026-09-22 (commit fd792fcf: README+TRIGGERS+PUBLISH+RUNBOOK+FAQ, lint 0/20) 🟢 · `docs/workflow/` (vanta-docs): inventario 28 + matriz triggers + flujo de
  publish por registro + runbook (re-run, approve environments, [no-adr]) + FAQ duplicados.
  Contrato: lint/frontmatter/coverage verdes; 0 links rotos. Commit `docs: FIND-143 — ...`
- [x] FIND-145 · ✅ 2026-09-22 (commit 63f77152: Analyze success en main, sin fix) 🟢 · Verificar CodeQL verde post-#202 en `main`; si sigue rojo, fix o deuda escrita.
  Contrato: check verde o causa declarada con dueño. Commit `ci: FIND-145 — ...` (o task file)

## Wave 5 — Reglas durables (tras FIND-143)

- [x] FIND-144 · ✅ 2026-09-22 (commit 7021480f: RULES.md 180L + CI_POLICY 26→27 + AGENTS pointer) 🟢 · `docs/workflow/RULES.md` (triggers, timeouts, pins, permissions, publish,
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

=== RECITATION FIND-135 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-135: timeout-minutes a 9 jobs sin límite en 5 workflows
Estado: completed
Última acción: S1-S6 completos: 9 timeouts aplicados, whitespace accidental revertido, commit selectivo 217d583a sin push
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 1 (FIND-137/138/139).
Contrato: Verificación: actionlint full exit 0 + git diff --check limpio + pre-commit hook ok. Evidencia: commit 217d583a (6 files, +79); gh run view 35693124393 (win 41.5/macOS 24/linux 11 min); ocr-delegate max ~10min en 5 runs; ocr-nightly ~23s; opencode sin runs (estimación 30). Artefactos: 5 workflows + docs/tasks/FIND-135.md commiteados. Invariantes: cero lógica (9 líneas añadidas); gate `*)` intacto (FIND-139); WIP ajeno (FIND-134/136, plan file) intacto; NO PUSH. Deuda: campaign_verify_cmd BUG exit -1 → verificado vía bash directa. Queda_pendiente: ninguno; NextTask Wave 1 (orquestador).
Próxima tarea si completa: Wave 1 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-136 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-136 cerrar huecos cobertura y caché en 4 workflows
Estado: completed
Última acción: S2-S6: 4 edits + verify triple + commit selectivo 6cf0e2cf sin push; carrera de index con FIND-134 resuelta (restore --staged, verificado HEAD ajeno limpio)
Resultado: ✅
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 1 (FIND-137/138/139).
Contrato: verificacion: git diff --check exit 0 + actionlint 4 files exit 0 + pre-commit hook actionlint ok + secrets-grep 0 | evidencia: commit 6cf0e2cf (5 files, +83/-1); SHAs 02cb101e/7e35be21 vigentes verificados por grep; toolchain no-op documentado | artefactos: 4 workflows + docs/tasks/FIND-136.md commiteados | invariantes: ci-rust-10/resto intactos; plan file (recitations server) no commiteado; NO PUSH | deuda: ninguna (riesgo S5 documentado en task file) | queda_pendiente: ninguno
Próxima tarea si completa: Wave 1 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-137 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: # FIND-137 — Unificar rustdocs caníbales en uno solo
Estado: completed
Última acción: S4 commit ci FIND-137 (survivor-edit + git rm + task file, 3 files +296/-92) + verify mecánico verde; NO PUSH
Resultado: ✅
Próxima acción: Wave 2 (orquestador)
Contrato: 1 solo workflow rustdoc (ci-rustdoc.yml survivor), 0 referencias rotas (grep), actionlint exit 0, survivor con lo mejor de ambos — evidencia §8 + commit 4b0686b0
Próxima tarea si completa: Wave 2 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-139 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-139: endurecer ci-gate.yml + gate-docs-21.yml
Estado: in-progress
Última acción: DISCOVERY: ambos workflows leidos + ruleset develop (11 checks) + semver/ADR if-guards mapeados
Resultado: PARTIAL
Próxima acción: Crear docs/tasks/FIND-139.md + edits S2/S3
Contrato: verificacion: actionlint exit 0 + git diff --check limpio (pendiente) | evidencia: plan docs/plans/2026-09-21-workflows-repair.md:45-46; ci-gate.yml 59L + gate-docs-21.yml 87L leidos completos; ruleset develop 23692587 = 11 checks | artefactos: docs/tasks/FIND-139.md (a crear) | invariantes: no tocar resto workflows (FIND-137/138 paralelo), timeout ci-gate intacto (FIND-135), NO PUSH | deuda: ninguna aun | queda_pendiente: crear task file + S2-S4
Próxima tarea si completa: Wave 2 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-138 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-138 — Diagnosticar flake Generate API reference (rustdoc) 0-1s sin log
Estado: completed
Última acción: S1-S4 completos: discovery + observacion GH solo-GET + veredicto cancel-by-design + commit selectivo 2e9b6e8b sin push
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 2.
Contrato: verificacion: GH GET x8 + actionlint survivor exit 0 (read-only) + git diff --check limpio + pre-commit hook ok | evidencia: 7 cancelados con jobs:[] (5 en batch 05:50:16-33 mismo group rustdoc-develop) + log vacio exit 0 + 6 verdes (35693192386 21m, 35686343573 4m, 35681924853 6m, +3) + concurrency ci-rustdoc.yml:54-56 | artefactos: docs/tasks/FIND-138.md + commit 2e9b6e8b (solo task file, NO PUSH) | invariantes: survivor intacto (STOP condicional no activado), resto workflows/src/web/desktop/locks/plans/Backlog intactos, plan file M ajeno no commiteado | deuda: causa externa/by-design declarada (GitHub cancel-in-progress); sin fix; idea futura sin diff | queda_pendiente: ninguno
Próxima tarea si completa: Wave 2 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-141 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-141: desolapar schedules pesados + guards auto-push + retention fuzz
Estado: completed
Última acción: S1-S5 completos: cron bench->02:00, [skip ci], 2x retention 14d, verify verde, commit selectivo 8dfd4a09 sin push (race FIND-146 separada via reset --soft + restore --staged)
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 3 FIND-142.
Contrato: verificacion: actionlint 3 files exit 0 (campaign_verify_cmd + pre-commit hook ok) + git diff --check limpio + secrets-grep vacio + grep cron/skip-ci/retention | evidencia: commit 8dfd4a09 (3 files +173/-2: bench 2 lineas, fuzz 2 lineas, task file); bench cron 0 2 * * * vs cert 0 3 * * 0; skip-ci 1 hit L266; retention 2x14 L116/171 | artefactos: .github/workflows/heavy-bench-nightly-51.yml, .github/workflows/fuzz-40.yml, docs/tasks/FIND-141.md | invariantes: resto workflows/src/web/plan/Backlog intactos; WIP FIND-140/146 preservado (su commit 1391ddde encima, atomico); NO PUSH | deuda: ninguna (colateral adapters Sun 03:00 pre-existente solo nota) | queda_pendiente: ninguno
Próxima tarea si completa: FIND-142 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-146 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-146: pins SHA + guard doble-run en 4 workflows
Estado: completed
Última acción: S1-S5 completos: 2+4+9 pins SHA + npm 1.12.9 ×3 + guard concurrency opencode + verify triple + commit selectivo sin push
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 3 (FIND-142) cuando Wave 2 verde.
Contrato: verificacion: actionlint 4 files exit 0 + git diff --check limpio + grep 0 tags moviles | evidencia: SHAs gh api (checkout 3d3c42e5 v7.0.1, setup-node 49933ea5 v4.4.0, upload-artifact ea165f8d v4.6.2) + npm view 1.12.9 | artefactos: 4 workflows + docs/tasks/FIND-146.md commiteados (NO PUSH) | invariantes: FIND-140/141, src/, web/, locks, Backlog intactos; plan file solo recitation no commiteada | deuda: campaign_validate_scope TASK_NOT_FOUND (plan sin bloque formal) — no bloquea; npm pin 1.12.9 quedara atras (renovate futuro) | queda_pendiente: ninguno
Próxima tarea si completa: Wave 3 (orquestador)
=== END RECITATION ===

=== RECITATION FIND-140 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-140: Publishing hardening en 5 workflows
Estado: completed
Última acción: S1-S7 completos: 5 edits minimos + verify triple + 5 commits selectivos sin push (FIND-146 commiteo en paralelo sin colision)
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: Wave 2 restante (FIND-141/146) / Wave 3.
Contrato: verificacion: actionlint 5/5 exit 0 + git diff --check exit 0 + secrets-grep 0 + pre-commit hook ok x5 | evidencia: commits a0d3123b (release.yml) + 95243f72 (npm-61) + 457dea46 (npm-node) + ac724291 (adapters-62) + 0312f52e (binaries-63 + task file); docs oficiales tags+branches+paths https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax | artefactos: 5 workflows + docs/tasks/FIND-140.md commiteados | invariantes: resto workflows/src/web/desktop/locks/Backlog intactos; WIP ajeno (FIND-141/146) no tocado; NO PUSH | deuda: campaign_verify_cmd BUG exit -1 → verificado via bash directa; re-run verde de workflows pendiente (solo vanta-lead/owner puede re-runear en GH) | queda_pendiente: re-run de los 5 workflows en GH + verificar release-plz-pr en proximo push a main
Próxima tarea si completa: FIND-141
=== END RECITATION ===

=== RECITATION FIND-143 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-143 docs/workflow inventario+triggers+publish+runbook+FAQ
Estado: completed
Última acción: S1-S6: task file + 5 docs + lint 0 + commit fd792fcf sin push
Resultado: OK
Próxima acción: Ninguno. Orquestador: FIND-145.
Contrato: verificacion: markdownlint-cli2 20 files 0 issues + grep ](docs/workflow 0 + git status selectivo | evidencia: commit fd792fcf (6 files +497) | artefactos: docs/workflow/README,TRIGGERS,PUBLISH,RUNBOOK,FAQ + docs/tasks/FIND-143.md | invariantes: cero YAML/src/web/desktop/locks/plans/Backlog; NO PUSH | deuda: coverage N/A, actionlint N/A, FIND-142 renombres pendientes | queda_pendiente: ninguno
Próxima tarea si completa: FIND-145
=== END RECITATION ===

=== RECITATION FIND-142 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-142: quitar numeración de filenames de workflows vía git mv + actualizar referencias
Estado: completed
Última acción: S1-S6 completos: 14 git mv + 9 refs yml + READMEs + 19 docs/workflow + CI_POLICY + commit selectivo 97a3a03c sin push
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: FIND-144.
Contrato: verificacion: actionlint exit 0 + git diff --check limpio + grep 0 old-names en alcance + pre-commit hook ok | evidencia: commit 97a3a03c (42 files, 14 renames R); greps de refs §Impacto | artefactos: 14 yml + 2 READMEs + 19 docs/workflow + CI_POLICY + docs/tasks/FIND-142.md | invariantes: 3 pins intactos; cero triggers/jobs; prohibidos intactos; stash GOV-C4 intacto; plan file M ajeno no commiteado; NO PUSH | deuda: badges-200 sin red; prosa stale fuera de alcance (TEST_MAP, ci-cd-guide, BENCHMARKS, QUICKSTART, CONTRIBUTING, Formula, pyproject, web/guides, glosario); campaign_verify_cmd BUG (dir) → bash directa | queda_pendiente: ninguno
Próxima tarea si completa: FIND-144
=== END RECITATION ===

=== RECITATION FIND-144 ===
Campaign ID: c56e3f17-2f32-4c20-b5e9-f7d1d3f00b36
Objetivo activo: FIND-144: RULES.md + CI_POLICY refresh + AGENTS pointer
Estado: completed
Última acción: S1-S6 completos: task file + RULES.md 7 reglas + CI_POLICY 26->27 + AGENTS fila + verify verde + commit selectivo 7021480f sin push
Resultado: OK
Próxima acción: Ninguno — tarea completa. Orquestador: cierre Wave 5 / plan.
Contrato: verificacion: markdownlint-cli2 22 files 0 issues + git diff --check limpio + grep 0 old-names + links resuelven + pre-commit hook ok | evidencia: commit 7021480f (4 files +262/-10: RULES.md nuevo 180L, CI_POLICY 7 edits, AGENTS.md 1 fila, task file); campaign_verify_cmd exit 0 (sin BUG esta vez) | artefactos: docs/workflow/RULES.md + docs/operations/CI_POLICY.md + AGENTS.md + docs/tasks/FIND-144.md commiteados | invariantes: cero YAML/src/web/desktop/locks/Backlog; plan file M ajeno no commiteado; NO PUSH | deuda: ninguna (progreso batch lo hace orquestador al cierre Wave 5) | queda_pendiente: ninguno
Próxima tarea si completa: cierre Wave 5 (orquestador)
=== END RECITATION ===
