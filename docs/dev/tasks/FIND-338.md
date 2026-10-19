---
title: "FIND-338: Borrar bak + templates .md duplicados"
kind: task
status: active
description: "Higiene raíz: borrar bak huérfano y 2 templates duplicados"
tags: [higiene, github]
---

# FIND-338: Borrar bak + templates .md duplicados

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 3)
- **Creado:** 2026-10-15
- **last-synced:** 2026-10-15
- **Estado:** ✅ COMPLETED
- **Skills:** `progreso`, `ponytail` (full)
- **Decisión owner:** borrar (aprobado explícito 2026-10-15)

## Blast Radius (Regla 0)
- **Verificado:** bak trackeado y huérfano (difiere del `opencode.jsonc` vigente = backup pre-cleanup superseded; cero refs). `config.yml` NO lista los `.md` (solo contact links, 8L). Grep: solo plan + research/archive (acta fechada, excluida). `.yml` sucesores existen y válidos.
- **Excluidos:** research/archive.
- **Implicaciones:** GitHub muestra solo los Issue Forms `.yml`; 3 deletes puros, 181 deletions, 0 adiciones.

## Contrato
`Test-Path` ×3 = False + `*.md` vacío en ISSUE_TEMPLATE + status con solo deletes + `check-links.mjs` exit 0

## Herramientas
- `git rm`, `node scripts/docs/check-links.mjs`

## Steps
### Step 1: Borrar los 3 archivos
- **Archivos:** `opencode.jsonc.bak-cleanup-20260928`, `.github/ISSUE_TEMPLATE/bug_report.md`, `.github/ISSUE_TEMPLATE/feature_request.md`
- **Acción:** `git rm <los 3>`
- **Verify:** Test-Path False ×3 + `*.md` vacío + `.yml` intactos
- **Estado:** ✅ COMPLETED

### Step 2: Gates + review P2-01 + commit local
- **Archivos:** plan file (Estado + Iteraciones)
- **Acción:** check-links exit 0 + review `vanta-review` + commit local; sin push (Regla 7)
- **Verify:** contrato verde + commit existe + plan actualizado
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0).

## Deuda / follow-up
- Ninguna.

## Notas

## Context Save Point
- **Fecha:** 2026-10-15
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** borrar (owner); archive excluido a propósito
- **Problemas conocidos:** ninguno
- **Próxima tarea:** FIND-339 (plan Task 4)

## Review (Gate P2-01)
- **Revisor:** vanta-review (sesión ses_ebebbfc12ffegK9N3lMKoG3Yzh — contexto distinto)
- **Veredicto:** ✅ APPROVE — 3 deletes puros, `config.yml` sin refs, sucesores `.yml` válidos, check-links exit 0, red-team sin rupturas, sin fixes.
