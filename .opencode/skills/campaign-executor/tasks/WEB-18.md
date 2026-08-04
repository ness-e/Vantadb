# WEB-18: Alinear pricing web con GO_TO_MARKET

## Metadata
- **Plan file:** docs/plans/2026-08-04-launch-web-campaign.md
- **Creado:** 2026-08-04
- **Estado:** ✅ COMPLETED

## Blast Radius
Conflicto: web vendía plan "Team $49/mo PER DEV SEAT" (SLA 48h, soporte privado).
GO_TO_MARKET §195 dicta Phase 1 "Open Source Pure, Revenue $0"; §243 pricing Cloud marcado "(Future)".

## Decisión (Opción b — default del plan)
Alinear sitio a GTM actual (Phase 1 open source, revenue $0). El tier pagado Team $49 no corresponde a la estrategia vigente. **Se eliminó** del sitio; quedan Community ($0) + Enterprise (Custom/SLA on-prem).

## Contrato (cumplido)
- grep '49' en vanta-data.ts → 0 matches; GO_TO_MARKET solo $499 Business (consistente)
- npm run build pasa (2 planes en grid lg:grid-cols-2)

## Archivos tocados
- web/src/components/vanta/vanta-data.ts (PRICING_PLANS: 3→2, Team eliminado, Enterprise features "Everything in Community")
- web/src/app/pricing/page.tsx (grid-cols-3 → grid-cols-2)
- web/src/app/pricing/layout.tsx (metadata: 3 planes→2, sin Team)
- web/src/lib/dictionaries.ts (plan.1 → Enterprise, CTA Contact Sales, plan.2.* muertas eliminadas, ES+EN)

## Notas
- Enterprise perdió `highlight` → ya no hay badge "Most popular" (consistente: nada es "popular" en fase open source).
- Si el team decide monetizar más tarde: re-agregar tier pagado con CTA real, NO $49 especulativo.

## Context Save Point
- **Fecha:** 2026-08-04
- **Branch:** develop
- **Commit:** f90b4ec8
- **CI pendiente:** web build ✅
- **Decisiones:** (b) alinear a GTM — eliminar Team $49, no agregarlo a GTM, porque contradice Phase 1 (revenue $0)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** campaña completada