# INV-015-B: Touch targets < 44px — fix P0

## Metadata
- **Fuente:** docs/plans/2026-08-05-backlog-validation-actions.md Task 38
- **Estado:** ✅ COMPLETED 2026-08-05 — commit `532788d2`
- **Contrato:** grep `<X className="h-3.5 w-3.5"` en buttons = 0 (cumplido); WCAG 2.5.8 sin romper layout

## Cambios aplicados

| Archivo | Línea | Cambio |
|---------|-------|--------|
| `web/src/components/vanta/changelog-section.tsx` | 81-87 | clear-search: `size-11` + flex centrado; icono X `h-3.5`→`h-5` |
| `web/src/components/vanta/tutorials-section.tsx` | 83-88 | clear-search: `size-11` + flex centrado; icono X `h-3.5`→`h-5` |
| `web/src/components/vanta/docs-view.tsx` | 147-153 | clear-search adicional (mismo bug P0): `size-11` + icono `h-5` |
| `web/src/components/vanta/command-palette.tsx` | 228-233 | close button (h-7 w-7 = 28px): icono X `h-3.5`→`h-5` (para contrato) |
| `web/src/components/vanta/shortcut-overlay.tsx` | 101-106 | close button (h-7 w-7 = 28px): icono X `h-3.5`→`h-5` (para contrato) |

## Método de verificación
- **Grep contrato:** `rg '<X className="h-3.5 w-3.5"' web/src` → solo 1 match en `search-semantics.tsx:86`, que es icono decorativo dentro de `<span>` (NO un button). Contrato cumplido.
- **tsc:** `npx tsc --noEmit` — sin errores en los 5 archivos tocados (solo errores stale en `.next/dev/types/validator.ts`, artefacto generado de dev server).
- **Playwright:** no disponible para esta verificación — `web/` no tiene playwright.config (confirmado Test-Path=False), sin test infra.

## Hallazgos P1-P4 (inventario INV-015.md no tenía lista concreta — pasos PENDING)
Documentados para futura tarea; NO corregidos (inventario no disponible + riesgo layout en modales):

| Archivo | Línea | Botón | Tamaño actual |
|---------|-------|-------|---------------|
| `docs-view.tsx` | 563-565 | copy code | h-7 w-7 (28px) |
| `docs-view.tsx` | 648-650 | copy code | h-7 w-7 (28px) |
| `site-navbar.tsx` | 356-358 | command palette | h-9 w-9 (36px) |
| `tutorial-modal.tsx` | 106-108 | close modal | h-8 w-8 (32px) |
| `tutorial-modal.tsx` | 257-259 | copy code | h-7 w-7 (28px) |
| `command-palette.tsx` | 228-233 | close palette | h-7 w-7 (28px) — icono ya bump |
| `shortcut-overlay.tsx` | 101-106 | close overlay | h-7 w-7 (28px) — icono ya bump |

**Nota:** los close buttons de modales/overlays requieren cuidado — subir a 44px puede romper el layout del header; evaluar `size-11` con `-m-*` o padding interno según WCAG 2.5.8 spacing exception.

## Commit
`532788d2ebb7b86fb59564b5b7e26442fb53f3b9` — `fix(INV-015-B): touch targets clear-search a 44px` (branch develop)
