# DESKTOP-40 — i18n real ES/EN (slice 1: infra + Settings)

> **Plan:** docs/dev/plans/2026-09-10-fixes.md (Task 5, Wave2)
> **Estado:** ✅ COMPLETED
> **Branch:** develop
> **Appetite:** max 3d | **Esfuerzo:** slice 1 ~0.5d (full UI = slice 2 DEFER)
> **Contrato:** slice 1: catálogo ES/EN + `tt()` + Settings cableado + `npm run build` 0 + tsc 0 (resto UI = slice 2 DEFER)
> **SDP:** frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering (doubt-driven + api-and-interface-design descartados: sin trust boundary ni API pública nueva)

## Spec

N/A — fix sobre código existente (criterio planes 2026-09-07/08/09: DO set 100% fixes, sin SPEC.md).
Decisiones por evidencia (Gate P del plan, owner aprobó 2026-09-10):

| # | Decisión | Evidencia |
|---|----------|-----------|
| 1 | Catálogo propio mínimo en `desktop/src/i18n/`, NO portar `web/src/lib/dictionaries.ts` | dictionaries web = 2823 líneas (~1.2k claves) + `language-provider.tsx` con `"use client"` Next; slice 1 solo necesita ~20 claves de Settings |
| 2 | `tt(lang, key, fallback)` función pura (sin provider/context) | Settings ya lee `connectionPrefs.get().lang`; provider React = abstracción prematura para 1 pantalla (ponytail) |
| 3 | Fuente de idioma = `connectionPrefs.lang` existente (store + sanitize + test ya existen) | `connections.ts:25,32,68`, `connections.test.ts:58-67` |
| 4 | Slice 1 = solo Settings.tsx; WorkspaceShell/resto UI = slice 2 DEFER | contrato del plan + stop condition appetite >3d |

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src/pages/Settings.tsx` (224L), `desktop/src/components/layout/WorkspaceShell.tsx` (1139+L, parcial — solo superficie ajustes + imports), `web/src/lib/language-provider.tsx` (85L), `web/src/lib/i18n-utils.ts` (15L), `desktop/src/store/connections.ts` (128L).
- **Referencias hacia dentro (qué importa lo que toco):** Settings importa `connectionPrefs`, `embedPrefs`; es importado por WorkspaceShell (`surface === "ajustes"`). `connectionPrefs.lang` solo se escribe en Settings (botones ES/EN) — ningún otro lector de `lang` en desktop/src (grep `lang` solo en store + test).
- **Referencias entrantes (quién depende):** WorkspaceShell → Settings (props intactas, no cambian). Ningún test existente cubre Settings (no hay Settings.test.*).
- **Veredicto:** impacto 🟢 mínimo — 2 archivos nuevos aditivos (`i18n/dictionaries.ts`, `i18n/index.ts`) + 1 test nuevo + edición localizada en Settings.tsx (solo strings → tt()). WorkspaceShell NO se toca en slice 1. Rollback = revert 1 commit.

## Gate D

No dispara: blast radius 3 archivos (2 nuevos), sin hot path, sin API pública nueva (tt local a desktop), contrato explícito aprobado por owner (Gate P 2026-09-10). ⬇️ downhill (patrón `createTt` existe en web).

## Steps

- [x] **Step 0 — DISCOVERY:** gap real (0 `tt(` en desktop/src), web 2823L no portable, lang store existe. Gate D no dispara.
- [x] **Step 1 — i18n infra + test RED→GREEN:** `desktop/src/i18n/dictionaries.ts` (26 claves Settings ES/EN) + `desktop/src/i18n/index.ts` (`tt`/`tp`) + `desktop/src/i18n/i18n.test.ts` (vitest). Verify: `npx vitest run src/i18n` → 4/4 ✅ (RED previo: módulo inexistente).
- [x] **Step 2 — Settings cableado:** strings ES → `tt(lang)`/`tp(lang)` con `lang = prefs.lang ?? "es"`; props/imports intactos (solo se quitó `DEFAULT_EMBED_MODEL` no usado). Verify: `npx tsc --noEmit` → exit 0 ✅.
- [x] **Step 3 — VERIFY contrato + commit + sync:** `npm run build` exit 0 (14.00s, warning chunk preexistente) + tsc 0 + vitest i18n 4/4 + commit solo archivos propios + sync plan + recitation.

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 0 | DISCOVERY: gap real, decisión catálogo mínimo, Gate D no dispara | ✅ | read/grep/bash |
| 1 | i18n infra + test RED→GREEN (dictionaries 26 claves, tt/tp, vitest 4/4) | ✅ | write/vitest |
| 2 | Settings cableado (strings → tt/tp, lang de prefs, quitó DEFAULT_EMBED_MODEL no usado) | ✅ tsc 0 | edit/bash |
| 3 | VERIFY contrato: vitest 4/4 + tsc 0 + build 14.00s + commit solo propios + sync | ✅ | bash/git |

## Notas

- WIP ajeno intacto: M opencode.jsonc + M .opencode + ?? Investigacion-plan.md — NO tocar ni stagear.
- Slice 2 DEFER: resto UI (WorkspaceShell nav, topbar, lenses) + posible provider.
