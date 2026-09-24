# Plan de Ejecución: Cierre total PR #182 — 2026-09-19

> **Campaign ID:** 8ae06b20-0178-41bc-8261-1634feb71271
> **Inicio:** 2026-09-19
> **Estado:** ✅ COMPLETADO (Wave A 3/3 + Wave B 3/3, 2026-09-19)
> **Fuente:** PR #182 checks post-push `cb954abc` (5 fails: ADR-Gate script, Windows flaky `audit.rs:134`, CodeQL init, Semver ~20, Vercel) + Codex review P1/P1/P2 + Backlog FIND-128/129/130/131/132/133

## Resumen
5 fails → 0 (o rojos declarados con dueño). Codex P1/P1/P2 fixeados. Sin duplicar trabajo: archivos disjuntos por wave.

## Waves (archivos disjuntos → paralelizable)
- **Wave A (paralelo ×3):** FIND-133 (src core semver) · CODEX-130/131/132 (mcp/server/memory) · FIND-128 (workflows ADR-gate + triggers)
- **Wave B (tras A):** FIND-129 (merges dependabot, necesita CI verde de A) + WIN-flaky `audit.rs:134` (necesita src estable tras 133) + cierre Publicación (P2-01 EXE-03-prep + progreso + archive)

## Tasks DO
- [x] FIND-133 · Alta · semver hacerlo pasar (triage intencional 0.6.0 + ADR Regla 5) · Wave A · `src/cli.rs`, `src/query.rs`, `src/sdk/types/`, `src/metrics/core/snapshot.rs`, `src/storage/engine/mod.rs`, `src/node/`, `src/planner.rs`, `src/server/middleware.rs`, `src/agentic/thread.rs`, `src/wal_shipping.rs`, `src/graph.rs`, `.github/workflows/ci-rust-10.yml` (solo lectura scope) · Commit `fix: FIND-133 — ...`
- [x] CODEX · Alta/Media · P1 recall-L1 (`vantadb-mcp/src/handlers/tools.rs:1723-1731`) + P1 compose-auth (`vantadb-server/docker-compose.yml:18-24`) + P2 L0-mismo-ms (`vanta-memory/src/services/conversation_hook.rs:64-68`) · Wave A · Commit `fix: CODEX-130/131/132 — ...`
- [x] FIND-128 · Media · fix script ADR-Gate (`Invalid format ADR-015` + `output` command) + matriz triggers push-vs-PR (duplicados) + propuesta dedup mínima · Wave A · `.github/workflows/` solo · Commit `ci: FIND-128 — ...`
- [x] FIND-129+WIN+CIERRE · Media · Wave B (no lanzar hasta A verde) · merges `#180` primero + `audit.rs:134` flaky + P2-01 EXE-03-prep + progreso + archive

## Gates
- **Gate P:** owner ya aprobó "Plan cierre total" + "Hacerlo pasar" semver (2026-09-19).
- **Pre-mortem:** (1) 133 toca src amplio → solo triage/revert/`non_exhaustive`, cero refactors; (2) Codex toca 3 áreas → 1 agente, 3 slices; (3) workflows → cambio mínimo, no refactors CI.
- **Appetite / Branch / Commit:** 2d / develop / conventional con ID.

## Retrospectiva de cierre (Start/Stop/Continue + 1 acción medible)

- **Start:** Waves disjuntas + 3 workers en paralelo ×2 waves (0 colisiones; rebase limpio sobre 4 merges dependabot).
- **Stop:** `python -c` con pipes en pwsh (quoting roto 2×) → scripts en archivo siempre.
- **Continue:** verify lead por slice + P2-01 batch con revisor distinto + transcripción dictamen al task file.
- **Acción medible:** flaky Windows cazado por duplicado push-vs-PR (mismo SHA pass+fail) — mantener lectura de runs duplicados en cada triage.

=== RECITATION ===
Objetivo activo: PLAN cierre-total-182 — CERRADO
Estado: completed
Última acción: Wave A (133 triage+ADR-044 · CODEX 3 slices · 128 ADR-fix+matriz) + Wave B (129 4 merges+9 rebases · WIN-flaky fix · P2-01 approve) + progreso (Backlog −6, avance 4 dominios)
Resultado: ✅
Próxima acción: archive + nota meta.md (este script no; orquestador con git)
Contrato: 6/6 con commit + verify lead + P2-01 Wave A approve; semver rojo-diseño aceptado hasta 0.6.0; CodeQL/Vercel rojos declarados con dueño
Invariantes: WIP ajeno intacto; mirrors skills drift FIND-83 no tocado; sin push de este cierre (commits previos ya pusheados salvo este)
Comandos de verificación: actionlint exit 0 · audit lib 6 passed · avance 1038/1038 · docs-coverage: solo drift mirrors pre-existente
Deuda: bump 0.6.0 en main vía release-plz · #180 a main tras proof · Lote1b/2 tras CI verde · mismatch ruleset↔triggers · Vercel inspect owner-side
Próxima tarea si completa: ninguna (plan cerrado)
last-synced: 2026-09-19
=== END RECITATION ===

=== RECITATION FIND-129 ===
Campaign ID: 8ae06b20-0178-41bc-8261-1634feb71271
Objetivo activo: FIND-129 triage 20 PRs
Estado: completed
Última acción: subagent 4 merges remotos + 9 rebases + task file; lead rebase limpio + push develop
Resultado: ✅
Próxima acción: monitorear CI post-merge; Lote1b/2 tras verde
Contrato: Lote1a mergeado + resto veredicto + cero closes ciegos
Próxima tarea si completa: cierre
=== END RECITATION ===

=== RECITATION WIN-FLAKY-AUDIT ===
Campaign ID: 8ae06b20-0178-41bc-8261-1634feb71271
Objetivo activo: WIN-FLAKY-AUDIT fixture unica
Estado: completed
Última acción: subagent fix seq+thread + regression test, commit 44a37d1b; lead verify lib 6 passed + push
Resultado: ✅
Próxima acción: cierre
Contrato: causa + fix determinista + 3 runs verdes
Próxima tarea si completa: cierre
=== END RECITATION ===
