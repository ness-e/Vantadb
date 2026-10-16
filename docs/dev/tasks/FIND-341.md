---
title: "FIND-341: Congelar desktop en feature-freez/"
kind: task
status: active
description: "Freeze desktop: move a feature-freez + quitar jobs CI + fix refs"
tags: [freeze, desktop, higiene]
---

# FIND-341: Congelar desktop en feature-freez/

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (fuera de las 12 tasks: orden directa owner 2026-10-16)
- **Creado:** 2026-10-16
- **last-synced:** 2026-10-16
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE + commit orquestador)
- **Skills:** `documentation-skill`, `progreso`, `ponytail` (full)
- **Decisión owner:** carpeta `feature-freez/` + mover `desktop/` + quitar sus 3 jobs de CI + quitar entrada dependabot

## Blast Radius (Regla 0)
- **Leído:** `desktop/` (198 tracked + junk ignorado node_modules/dist/logs), grep `desktop/` (60 matches: CI, Cargo exclude, machete, e2e, comments, docs), `.gitignore` (sin entradas desktop), `CONTRIBUTING.md:50-68` (menciona hooks, no desktop).
- **MUEVE:** `desktop/` → `feature-freez/desktop/` (filesystem move, junk ignorado viaja en disco sin commitear).
- **REFS MECÁNICOS:** `Cargo.toml` exclude, `ci-rust.yml` machete, `e2e-persistence.mjs` require, `rocksdb_backend.rs` comments, `release-plz.toml` comment, `audit.toml` + `osv-scanner.toml` comments, `opencode.jsonc` texto, `vantadb-wasm/README.md` tabla, `desktop/README.md` (banner freeze + `../docs`→`../../docs` + paths).
- **CI (decisión owner):** quitar jobs windows/macos/linux de `platform-ops.yml` + paths `desktop/**` + header; quitar entrada cargo desktop de `dependabot.yml`. TRIGGERS.md + workflow README actualizados.
- **DOCS VIVAS:** `docs/dev/desktop/*` + `docs/user/desktop/*` re-pathadas (código movido, docs quedan).
- **EXCLUIDOS (histórico):** research/*, avance/*, reviews/*, tasks/* ajenos, CHANGELOG, `.opencode/**`.

## Contrato
`Test-Path desktop` = False + `Test-Path feature-freez/desktop` = True + `git status` muestra renames + `check-links` exit 0 + yaml parse 18→17 workfl.. (verificar conteo) + `git diff --check` limpio

## Herramientas
- `git mv`/`Move-Item`, `edit`, `node scripts/docs/*.mjs`, `python yaml`

## Steps
### Step 1: Mover + junk ✅ (hecho: 198 renames, índice solo feature-freez)
### Step 2: Refs mecánicos ✅ (9 archivos)
### Step 3: README movido + CI (jobs fuera, dependabot fuera) + docs vivas ✅ + fixes review (R1-R3 + O1) ✅
### Step 4: Gates + review P2-01 + commit local (sin push) ✅ (links 0, docs baseline, yaml 18, diff limpio, APPROVE)

## Dependencias
- Ninguna (orden directa owner). EGO-03 corre en paralelo en `src/graphrag` (sin solape).

## Deuda / follow-up
- `feature-freez/desktop/node_modules` y `dist*/` viajan en disco (ignorados, regenerables).
- Unfreeze: procedimiento en `feature-freez/README.md`.

## Notas

## Context Save Point
- **Fecha:** 2026-10-16
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** nombre literal `feature-freez/` (owner); quitar jobs (no re-path); quitar entrada dependabot (no actualizar path)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** cerrar Step 4 + Wave DIST-19

## Review (Gate P2-01)
- **Revisor:** vanta-review, sesión distinta (veredicto 2026-10-16, re-ejecución mecánica).
- **Veredicto:** 🔴 CHANGES-REQUIRED inicial → ✅ APPROVE tras R1-R3 + O1 + staging selectivo (sin `desktop/` pelado vivo, jobs 8→5 exacto, pins/docs verificados, set sin prohibidos).
