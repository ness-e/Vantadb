# Task 35: INV-005-A - error.tsx boundary + drop @mdxeditor/editor

## Metadata
- **Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` (Task 35)
- **ID:** INV-005-A
- **Estado:** COMPLETED 2026-08-05

## Acción ejecutada
1. **`web/src/app/error.tsx` creado** — App Router error boundary (Next.js 16 pattern):
   - `"use client"` + props `error: Error & { digest?: string }` y `reset: () => void`
   - `useEffect` → `console.error(error)` (logging estándar del patrón Next)
   - UI siguiendo el design system del sitio (cream `#FBF9F5` / ink `#000` / neon `#FF5500`, `border-4 border-black`, `shadow-[6px_6px_0_0_#000]`, fonts `font-display`/`font-tech`, idioma es como el resto del site)
   - Botón "Reintentar" (`reset`) + link "Volver al inicio" (`/`), digest mostrado en font-tech
2. **`@mdxeditor/editor` eliminado** vía `npm uninstall @mdxeditor/editor` (actualizó package.json + package-lock.json juntos; 150 paquetes removidos, incluido `@mdxeditor/gurx`)
   - Verificado: 0 imports en `web/src`, 0 referencias en package.json/lock
   - Otras deps del bloque MDX NO tocadas (react-markdown, react-syntax-highlighter siguen en uso)

## Verificación (contrato)
- `npm run build` en web/ → ✅ PASSA (Next.js 16.2.12, 35/35 rutas estáticas generadas, compiló en 7.3s)
- `rg "@mdxeditor" web/src` = 0 ✅
- `rg "mdxeditor" web/package.json web/package-lock.json web/src` = 0 ✅

## Notas
- Primer intento de build falló con "Another next build process is already running": lock stale (`.next/lock`) de un build previo que ya había salido. Se verificó que ningún proceso next build activo existía (solo `next start -p 3002` viejos del 4/8, ajenos) y se reintentó; lock ya había desaparecido.
- Warning de Next "inferred workspace root" (múltiples lockfiles: hay un `C:\Users\Eros\package-lock.json` en el home) — pre-existente, no bloquea el build. `ignoreBuildErrors: true` en next.config (TS no bloquea build).
- Staging selectivo: solo `web/package.json`, `web/package-lock.json`, `web/src/app/error.tsx`. El working tree tiene cambios de otras tareas del pipeline (plan file, Backlog.md, etc.) que NO se tocaron ni se commitearon.
- 6 vulnerabilidades npm reportadas post-uninstall (3 moderate, 3 high) — pre-existentes del resto del árbol de deps, fuera de scope de esta task.
