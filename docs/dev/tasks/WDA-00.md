# Task WDA-00 — F0 Baseline medible (auditoría diseño web)

**Estado:** ✅ COMPLETED (2026-08-24, ejecutada inline por vanta-lead tras 3 sub-agentes SARL-fallidos)
**Modo:** SOLO MEDICIÓN · sin cambios en web/src/**

## Contrato verificable — RESULTADO
1. ✅ `npm run build` exit 0 — 35 rutas estáticas + /api, /blog/[slug], /case-studies/[slug] dinámicas
2. ✅ 30 screenshots (15 rutas × 1440×900 + 390×844) en `web/audit-baseline/*.png`
3. ✅ Lighthouse desktop JSON+HTML en `web/audit-baseline/lighthouse/`:
   - **home:** perf=96 a11y=95 bp=100 seo=100
   - **about-team:** perf=95 a11y=96 bp=100 seo=100
4. ✅ Violaciones a11y (axe vía Lighthouse) en `web/audit-baseline/a11y/violations-lighthouse.txt`:
   - home: color-contrast ×19 · heading-order ×1 · label-content-name-mismatch ×1
   - about-team: color-contrast ×5 · label-content-name-mismatch ×1

## Baseline adicional (DISCOVERY)
- next ^16.1.1 · react ^19 · output standalone (`npm start` = node .next/standalone/server.js)
- `npm run lint` = **limpio hoy** (0 errores/warnings con reglas actuales desactivadas — baseline antes de reactivar reglas en WDA-05)
- Server standalone corriendo en :3000 (dejar vivo para fases siguientes)
- chrome-launcher requiere `CHROME_PATH=C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe` + `--chrome-flags="--headless"` (el `--headless=new` falla con Edge)
- playwright-cli 0.1.14 OK para screenshots

## Artefactos
- `web/audit-baseline/` (PNGs + lighthouse + a11y) — **local only**, en `.gitignore` (binarios pesados); métricas durables viven en este task file

## Context Save Point
- **Fecha:** 2026-08-24
- **Branch:** develop
- **CI pendiente:** no aplica (sin push)
- **Decisiones:** artefactos de medición fuera de git (ponytail); métricas aquí. A11y scan via Lighthouse/axe en vez de accesslint (reuso de lo ya medido).
- **Problemas conocidos:** sub-agentes mueren silenciosos en tareas largas (build+screenshots) — dividir futuras tareas largas o hacer inline.
- **Próxima tarea:** WDA-01 — F1 Diseño
