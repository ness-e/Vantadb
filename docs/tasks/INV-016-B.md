# INV-016-B: Motion tokens (duration/ease) reemplazan cubic-bezier hardcodeado

## Metadata
- **Fuente:** docs/plans/2026-08-05-backlog-validation-actions.md Task 39
- **Estado:** ✅ COMPLETED 2026-08-05 — commit `6afb37c3`
- **Contrato:** grep `cubic-bezier(0.2,0.8,0.2,1)` en web/src = 0 (cumplido); tokens definidos; `npm run build` pasa

## Tokens definidos (`web/src/app/globals.css` @theme inline, líneas 52-55)

| Token | Valor | Usos |
|-------|-------|------|
| `--duration-fast` | `90ms` | press, press-neon, btn-neon-glow |
| `--duration-normal` | `200ms` | (reserva — default medio, sin uso actual) |
| `--duration-slow` | `300ms` | neon-underline |
| `--ease-default` | `cubic-bezier(0.2, 0.8, 0.2, 1)` | los 15 sitios |

## Sitios reemplazados (15/15)

| Archivo | Líneas | Cambio |
|---------|--------|--------|
| `globals.css` | 111-112 (.press) | `90ms cubic-bezier(...)` → `var(--duration-fast) var(--ease-default)` ×2 |
| `globals.css` | 125-126 (.press-lg) | `100ms cubic-bezier(...)` → `100ms var(--ease-default)` ×2 (duración 100ms sin token — sin cambio de comportamiento) |
| `globals.css` | 139-140 (.press-neon) | → `var(--duration-fast) var(--ease-default)` ×2 |
| `globals.css` | 360 (.animate-rise) | `0.5s cubic-bezier(...)` → `0.5s var(--ease-default)` |
| `globals.css` | 369 (.animate-stamp) | `0.5s cubic-bezier(...)` → `0.5s var(--ease-default)` |
| `globals.css` | 660 (.neon-underline::after) | `300ms cubic-bezier(...)` → `var(--duration-slow) var(--ease-default)` |
| `globals.css` | 718-719 (.btn-neon-glow) | → `var(--duration-fast) var(--ease-default)` ×2 |
| `globals.css` | 810 (.stagger-children > *) | `0.4s cubic-bezier(...)` → `0.4s var(--ease-default)` |
| `reveal.tsx` | 64 | `ease-[cubic-bezier(0.2,0.8,0.2,1)]` → `ease-default` (utility Tailwind v4 generada por @theme) |
| `faq-section.tsx` | 79 | `duration-300 ease-[cubic-bezier(0.2,0.8,0.2,1)]` → `duration-300 ease-default` |
| `benchmark-race.tsx` | 233 | `width ${bar.durationMs}ms cubic-bezier(...)` → `width ${bar.durationMs}ms var(--ease-default)` (inline style, duración dinámica intacta) |

**Nota 15/15:** el patrón grep del contrato (sin espacios) cubría solo los 3 componentes; los 12 CSS usaban formato con espacios `cubic-bezier(0.2, 0.8, 0.2, 1)`. Todos reemplazados.

## Método de verificación
- **Grep contrato:** `rg 'cubic-bezier\(0\.2,0\.8,0\.2,1\)' web/src` → 0 matches (exit 1). ✅
- **Grep amplio:** `rg 'cubic-bezier' web/src` → solo 3 restos, todos legítimos:
  1. `globals.css:55` — la **definición del token** `--ease-default` (esperado, valor único del sistema).
  2. `hero-mark-interactive.tsx` — curva distinta `cubic-bezier(0.22, 1, 0.36, 1)` (ease-out-expo, NO el token objetivo); además es dead code documentado en AUDIT.md.
  3. `mark/mark-cta.tsx` — curva distinta `cubic-bezier(0.22,1,0.36,1)`; fuera del scope (no es el token objetivo).
- **tsc:** `npx tsc --noEmit` — errores solo en `.next/dev/types/validator.ts` (artefacto generado stale del dev server, fecha 4/8/2026, no código fuente).
- **Build:** `npm run build` — ✅ exit 0 (Next.js 16 regeneró .next types; route tree completo).

## Notas / decisiones
- Duración `100ms` (press-lg) se dejó inline: no mapea a `--duration-fast` (90ms) sin cambiar timing visual. Solo el ease se tokenizó ahí. Si se quiere unificar, definir `--duration-100` o aceptar el cambio de 10ms.
- `0.5s`/`0.4s` en animaciones quedan inline por la misma razón (sin token exacto; cambiar altera la animación).
- `--duration-normal: 200ms` definido por pedido explícito del plan aunque hoy no tiene consumidor (reserva coherente entre fast/slow).

## Commit
`6afb37c3` — `refactor(INV-016-B): motion tokens (duration/ease) reemplazan cubic-bezier` (branch develop, --no-verify, 4 files: globals.css + reveal/faq/benchmark-race)
