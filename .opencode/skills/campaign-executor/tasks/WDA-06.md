# Task WDA-06 — F6 Escritura / i18n residual

**Estado:** ✅ COMPLETED (2026-08-24, inline por vanta-lead — proveedor de sub-agentes caído)
**Nota:** FAQ bilingüe YA existía en dictionaries (EN desde línea ~1796) — el claim del plan "FAQ 100% español" estaba stale.

## Contrato — RESULTADO
1. ✅ site-navbar: `{ t }` → `{ t, tt }`; toast "- coming soon" → `· ${tt("common.comingSoon")}`; 3 aria-labels EN → tt("a11y.mainNav/toggleMenu/mobileNav") (el em-dash de la línea original requirió [char]8212 para el replace)
2. ✅ code-playground: import useLanguage + `// press Run to execute` → tt("playground.pressRun"). El filename `playground.js` se conserva (es un nombre de archivo, no prosa — YAGNI)
3. ✅ easter-egg: alt + mensaje → tt("easterEgg.alt/found") — `found` reutiliza clave pre-existente
4. ✅ Dictionaries: 6 claves nuevas ES+EN simétricas (common.comingSoon, a11y.mainNav/toggleMenu/mobileNav, playground.pressRun, easterEgg.alt); easterEgg.found ya existía → sin duplicar (TS1117 atrapado y resuelto)
5. ✅ Microcopy restante ya estaba vía tt() (WDA-04 lo adelantó)
6. ✅ lint exit 0 (0 errores) · build exit 0 · tsc --noEmit exit 0

## Context Save Point
- **Fecha:** 2026-08-24 · **Branch:** develop
- **Próxima tarea:** WDA-07 — F7 Diseño comercial
