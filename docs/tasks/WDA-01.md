# WDA-01 — F1 Diseño

## Estado
✅ COMPLETO (2026-08-24) — sin commit (orquestador commitea)

## Impacto mapeado (Regla 0)

**Archivos leídos completos / relevantes:**
- `web/src/components/vanta/mark/mark-classic.tsx` (274L) — bucle animejs `loop:true` líneas 76-94 sin cleanup real
- `web/src/components/vanta/mark/use-mark-interaction.ts` (225L) — setTimeout :193/:205 sin cleanup
- `web/src/components/vanta/mark/mark-cta.tsx` (190L) — setTimeout :110 sin cleanup
- `web/src/components/vanta/latency-comparator.tsx` (584L) — setInterval :137 con ref + stopBenchmark pero SIN cleanup de unmount
- `web/src/lib/dictionaries.ts` (:558-563, :586-593 ES; :2045-2050, :2070-2080 EN)
- `web/src/components/vanta/site-navbar.tsx` (:354 comentario)

**Referencias hacia dentro:** grep `theme.light|theme.dark|toggleTheme|theme-provider|theme-toggle|next-themes|ThemeProvider|setTheme` en web/src = SOLO dictionaries.ts (6 matches, todos las claves a borrar). Cero usos en componentes. No hay theme-provider.tsx/theme-toggle.tsx ni dep next-themes activos.

**Referencias entrantes:** ninguna hacia las claves theme.*; los archivos mark/* se usan desde hero/cta sections (no cambian sus props/API).

**Veredicto:** cambios seguros, blast radius 6 archivos, todos dentro de web/. Sin API pública afectada.

## Spec

Decisión R-FE-4 (light-only). Borrar claves huérfanas, corregir leak de animación loop infinito animejs (el comentario "Cleanup handled by anime.js" era falso — anime.js NO auto-cleanup en unmount), gestionar timers con refs + clearTimeout/clearInterval en unmount, respetar prefers-reduced-motion en el loop ambiente.

## Steps

1. ✅ Limpiar claves huérfanas dictionaries.ts (ES+EN: theme.light/dark + shortcuts.toggleTheme ×2 idiomas) + comentario navbar (:354)
2. ✅ Fix mark-classic.tsx: ambientAnimsRef guarda instancias animejs, cleanup `.pause()` en unmount + skip si prefers-reduced-motion + comentario corregido
3. ✅ Cleanup timers: use-mark-interaction (blinkTimersRef ×2 clearTimeout en unmount), mark-cta (reactionTimerRef), latency-comparator (useEffect unmount clearInterval)
4. ✅ Verify: contrato greps = 0/0/loop con cleanup; `npm run build` exit 0

## Context Save Point

- Nota operativa: el build fallaba con EBUSY porque había un `node .next/standalone/server.js` (PID 22976) corriendo — se mató ese proceso para poder buildear. El dev server (next dev -p 3000) sigue vivo.
- Archivos tocados: los 6 de arriba. Sin cambios en web/remotion/ ni desktop/.

## Hallazgos diseño (para reporte del orquestador)

Ver bloque RESULTADO — consolidar desde ahí.
