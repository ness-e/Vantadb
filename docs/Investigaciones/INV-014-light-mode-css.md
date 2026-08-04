# INV-014: Light Mode (CSS muerto) — Auditoría

> **Estado:** ✅ COMPLETADA 2026-08-03 · **Fuente:** docs/Backlog.md INV-014 · **Tipo:** Web Frontend (CSS/theme) — auditoría + propuesta, sin implementación

## Resumen Ejecutivo

**La premisa del backlog estaba invertida: NO existe CSS light muerto — el sitio es LIGHT-ONLY por diseño.**

La paleta (manga/linocut: cream `#FBF9F5` / ink `#000000` / neon `#FF5500`) es inherentemente light. El problema real es la **plomería DARK inerte**: infraestructura de dark mode que nunca se monta.

## Evidencia

### 1. CSS: solo tokens light

| Check | Resultado |
|---|---|
| Bloque `.dark` en `globals.css` | **No existe** |
| Variantes `light:` | **Cero** |
| `@media (prefers-color-scheme: light)` | **Cero** (único media query: `prefers-reduced-motion`) |
| Duplicación `:root` vs `.dark` | No aplica — `.dark` no existe |
| `@theme inline` | Define **solo tokens light** (cream/ink/neon) |
| Utilities (`.press`, `.paper-bg`, `.scroll-manga`…) | Hardcodean hex light |

### 2. Wiring de next-themes: 100% inerte

| Componente | Estado |
|---|---|
| `theme-provider.tsx` | Configurado bien (`attribute="class"`, `defaultTheme="light"`, `enableSystem={false}`) pero **NUNCA se monta** — `layout.tsx` envuelve solo en `LanguageProvider`. Cero imports del provider fuera de su propio archivo. |
| `theme-toggle.tsx` | Funcional (useTheme + mounted guard + aria) pero su **único consumer es `navbar.tsx:396` — código muerto** (reemplazado por `site-navbar.tsx`, que no tiene toggle; grep confirma que `navbar.tsx` no se importa en ningún lado). |
| `next-themes` dep | Importado únicamente por los 2 componentes huérfanos. |

**Conclusión del wiring:** aun si se montara el provider, togglear `.dark` produciría **cero cambio visual** — no hay CSS que lo escuche.

## Recomendación

1. **Eliminar la plomería DARK inerte** (YAGNI): `theme-provider.tsx` + `theme-toggle.tsx` + dep `next-themes` de `package.json`.
2. **NO reactivar dark mode**: requeriría bloque `.dark` con nueva paleta + sobreescribir decenas de utilities hardcodeadas a light — contradice la estética manga/linocut. Dark mode = feature aparte si se quiere.
3. **Corregir nota stale en DOS archivos**: `web/AGENTS.md` ("class-based theme switching (light default)") **y la tabla Stack decisions de `.opencode/AGENTS.md` raíz** — ambos afirman un next-themes que no está montado (verificado 2026-08-04; la versión original del doc solo citaba `web/AGENTS.md`).

## Notas

- Coincide con el patrón dead-code ya documentado en `web/AUDIT.md`.
- Matiz (verificado 2026-08-04): `web/src/components/ui/chart.tsx:9` (recharts) referencia el selector `.dark` (`THEMES.dark = ".dark"`), pero no hay ninguna variable `.dark` en el stylesheet — efecto visual insignificante en Tailwind v4. "Cero cambios visuales" se precisa a "cero cambios con paleta dark definida".
- Solo auditoría + propuesta (alcance del backlog). Cero cambios de código.

<!-- Changed 2026-08-04: alcance de la nota stale ampliado a 2 AGENTS.md (web/ + raíz); matiz chart.tsx. -->
