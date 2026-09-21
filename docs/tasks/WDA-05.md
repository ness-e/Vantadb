# Task WDA-05 — F5 Performance

**Estado:** ✅ COMPLETED (2026-08-24, ejecutada inline por vanta-lead tras 3 sub-agentes sin progreso)
**Nota:** `turbopack.root` ya estaba configurado (REVIEW-18) — el plan estaba stale en ese punto.

## Contrato — RESULTADO
1. ✅ 13 deps zombies eliminadas de package.json (`@dnd-kit/*` ×3, `@hookform/resolvers`, `@reactuses/core`, `@tanstack/*` ×2, `date-fns`, `next-intl`, `uuid`, `z-ai-web-dev-sdk`, `zod`, `zustand`). **`react-markdown` CONSERVADA** — tiene import real en `docs/guides/page.tsx` (el plan la contaba mal). Lockfile regenerado.
2. ✅ `ui/` muerta eliminada: 46 wrappers shadcn → queda solo `sonner.tsx`. Cadena huérfana `hooks/use-toast.ts` → `ui/toast` ← `ui/toaster` también fuera.
3. ✅ Code-splitting: `command-palette` lazy vía `next/dynamic` ssr:false en site-shell (saca vanta-data 1136 líneas del bundle global).
4. ✅ ESLint reactivado: `no-unused-vars`=error, `no-explicit-any`=warn, `exhaustive-deps`=warn, `no-img-element`=warn. **58 errores → 0** (`npm run lint` exit 0, quedan 3 warnings `<img>` permitidos). Fixes: `{ t, tt }`→`{ tt }` en 32 archivos, ~16 imports muertos fuera, 3 componentes muertos borrados (navbar/ecosystem/metrics-bar — vs-table SE CONSERVA: lo usa /why-vantadb, el plan estaba mal), `language-provider` useCallback→useMemo inline fn (react-compiler rule), remotion excluido del lint (fuera de alcance).
5. ✅ `npm run build` exit 0 · Toaster refs = 0 · zombies en lockfile = 0

## Lecciones
- PowerShell `-replace` no acepta 3 args (fue el error que vació 35 archivos al sub-agente de WDA-02); usar `.Replace()` literal o edit tool para cambios quirúrgicos.
- `[slug]` en paths es wildcard de PowerShell → siempre `-LiteralPath`.
- Sub-agentes mueren silenciosos en tareas multi-step pesadas → dividir o inline.

## Context Save Point
- **Fecha:** 2026-08-24 · **Branch:** develop · **CI pendiente:** no
- **Próxima tarea:** WDA-06 — F6 Escritura
