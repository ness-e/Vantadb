---
title: "TASK FIND-236: Job ASan — excluir sift1m_competitive_benchmark (guard release-only en debug)"
kind: task
description: "El test release-only panics en el job debug+ASan por diseño (competitive_bench.rs:64) — ruido pre-existente que impedía leer el rojo como 'leak real'"
---

# TASK FIND-236: Job ASan — excluir sift1m_competitive_benchmark

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 7, F0)
- **Fuente:** `docs/dev/Backlog.md` (FIND-236; origen FIND-226 post-push)
- **Esfuerzo:** 🟢 1h | **Appetite:** max 1h | **Prioridad:** 🟡
- **Tipo:** CI (workflow)
- **Creado:** 2026-10-04T07:05Z | **last-synced:** 2026-10-04T07:10Z
- **Estado:** ⏳ IN PROGRESS (review P2-01 pendiente)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | CI (job `sanitizer-asan`, best-effort/continue-on-error) |
| Callees | `cargo +nightly test` (debug + `-Zsanitizer=address`) sobre `-p vantadb` |
| Implicaciones | Solo la selección de tests del job; sin cambio de código ni de producto. El rojo del job pasa a significar "leak real" |

## Impacto mapeado (Regla 0)

- **Evidencia:** run `37102066714` (post-push 2026-10-03): el job quedó rojo SOLO por el panic `"competitive_bench must run with --release"` en `tests/certification/competitive_bench.rs:64`; idéntico en el run pre-fix `111086294644` → **pre-existente, no causado por FIND-226** (leaks = 0 desde el fix).
- **Causa:** el job corre en debug (ASan) por diseño; el test tiene un guard que exige release.

## Contrato

1. El job ASan no ejecuta `sift1m_competitive_benchmark` (skip explícito) → su rojo significa solo leaks reales: ✅ (`--skip` agregado al comando del step)
2. `actionlint` exit 0: ✅
3. Verificación final en el próximo run de `ci-rust` (post-push — **push diferido al final del plan por instrucción del owner**): `gh run list --workflow=ci-rust.yml --limit 1` → `gh run view --job <asan-job-id> --log | Select-String "sift1m|LeakSanitizer|SUMMARY"` → esperado: sin panic del guard; leaks (si hay) simbolizados.

## Steps

- [x] **Step 1 — Evidencia:** runs `37102066714` / `111086294644` (mismo panic del guard; pre-existente) — ✅
- [x] **Step 2 — Fix:** `--skip sift1m_competitive_benchmark` + comentario `FIND-236` en el workflow — ✅
- [x] **Step 3 — actionlint exit 0** — ✅
- [ ] **Step 4 — Review P2-01 (vanta-review fresco)** — ⏳ en curso
- [ ] **Step 5 (post-push) — verificar el job en el próximo run** — ⬜ diferido (comando arriba)

## Review (P2-01)

| Campo | Valor |
|-------|-------|
| Reviewer | (pendiente — vanta-review fresco) |
| reviewer_context | (pendiente) |
| Verdict | (pendiente) |

## DoD (3 niveles)

- **task:** contrato 1-2 ✅ (3 diferido por push)
- **commit:** `ci:` + actionlint · release: n/a

## Deuda técnica (Regla 6)

Ninguna. Nota: si en el próximo run apareciera OTRO test debug-incompatible, se documenta como FIND derivado (stop condition del plan).
