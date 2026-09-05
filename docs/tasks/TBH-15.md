# TBH-15 — Consolidar `scripts/audit-tokens.{sh,ps1}`

**Estado:** ✅ Completed
**Fecha:** 2026-08-30
**Esfuerzo real:** ~10min
**Branch:** develop
**Plan ref:** `docs/plans/2026-08-30-testing-bench-harden.md` Phase 2

## Resumen
Eliminar la variante bash del script de auditoría de tokens (duplicado de la variante PowerShell). Mantener el `.ps1` como único entrypoint.

## Discovery
Búsqueda global del nombre del archivo bash antes del delete:
- `docs/Backlog.md:767` — entrada de backlog describiendo TBH-15 (histórico)
- `docs/plans/2026-08-30-testing-bench-harden.md:138` — Phase 2 task spec
- `docs/plans/2026-08-30-testing-bench-harden.md:232` — lista de scope del plan
- el archivo bash líneas 4-5 — self-references (header del script)

`git grep "audit-tokens"` NO encontró referencias en `Justfile`, `.github/workflows/`, ni otros scripts. ✅ Limpio en código.

## Ejecución
1. Remover el archivo bash del repo
2. Actualizar `docs/Backlog.md:767` — marcar TBH-15 como ✅ Completado, descripción ajustada al estado post-consolidación
3. Actualizar `docs/plans/2026-08-30-testing-bench-harden.md:138` — reflejar "deleted (keep .ps1)"
4. Actualizar `docs/plans/2026-08-30-testing-bench-harden.md:232` — scope: solo `.ps1` queda

## Verificación
- `git grep` para el nombre del archivo bash → 0 matches ✅
- `git ls-files` del archivo bash → vacío ✅
- `git ls-files` del archivo PowerShell → 1 (preservado) ✅

## Contrato cumplido
- [x] Variante bash eliminada del index (no debe volver a aparecer)
- [x] Variante PowerShell preservada intacta
- [x] Docs actualizadas a estado post-consolidación (no mienten sobre el resultado)
- [x] Ponytail: 0 nueva lógica, 0 nuevos scripts, diff mínimo
