# Frontend Web — Reglas

> **Scope:** `web/` completo — Next.js 16 + React 19, `src/app/` (todas "use client"), `src/components/vanta/`, `src/components/ui/` (shadcn), `src/hooks/`, `src/lib/` (i18n ~880 claves ES/EN), `globals.css` (Tailwind v4 `@theme inline`)
> **No tocar aquí:** bindings/Rust (`python-bindings.md`, `js-ecosystem.md`, `server-mcp.md`), API pública (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-005, INV-013, INV-014, INV-015, INV-016

## Reglas

### R-FE-1: Manejo de errores con `error.tsx` nativo, no librerías externas

- **Must:** usar `src/app/error.tsx` (error boundary nativo de Next.js App Router) para el manejo de errores de página.
- **Must not:** instalar `react-error-boundary` ni otras librerías de error boundary; si aparece como dependencia transitiva, no importarla directamente.
- **Por qué:** `react-error-boundary` llegó solo como dep transitiva de `@lexical/react` vía `@mdxeditor/editor` (package-lock.json:2034); es peso muerto no usado. Next.js App Router ya provee `error.tsx`/`global-error.tsx` sin dependencias (INV-005).

### R-FE-2: Eliminar dependencias muertas detectadas en auditorías

- **Must:** remover del `package.json` toda dependencia sin imports directos verificados (ej. `@mdxeditor/editor` — 0 imports en `web/src/`, package.json:16) en el mismo cambio que la auditoría la detecta o en una tarea de cleanup dedicada.
- **Must not:** mantener dependencias transitivas huérfanas solo porque "algún día" se usen.
- **Por qué:** el bundle del web es estático por diseño; cada dep muerta infla `npm install` y el árbol de auditoría de dependencias (INV-005).

### R-FE-3: JSON-LD se emite manualmente en Server Component — Metadata API NO lo genera

- **Must:** emitir `<script type="application/ld+json">` explícitamente en el JSX de un Server Component (ej. en `layout.tsx` raíz) cuando se implemente structured data.
- **Must not:** asumir que `export const metadata` o `generateMetadata` de Next.js 16 genera JSON-LD — la Metadata API solo emite tags `<head>` (title, description, OG, Twitter); no existe campo `jsonLd`.
- **Por qué:** el sitio tiene cero `<script type="application/ld+json">` (INV-013); el JSON-LD debe marcarse a mano o no existe. La nota de `docs/progreso/README.md` que afirmaba JSON-LD implementado era falsa (WEB-13 fue sobre Pages Router, ya migrado).

### R-FE-4: Light-only por diseño — no reactivar dark mode

- **Must:** mantener el sitio en paleta light (cream `#FBF9F5`, ink `#000000`, neon `#FF5500`); `globals.css` define solo tokens light en `@theme inline` + `:root`.
- **Must not:** reintroducir dark mode, clases `.dark`, `prefers-color-scheme`, ni variables `--*dark*` — contradice la estética manga/linocut decidida.
- **Must not:** dejar componentes de plomería dark muertos (`theme-provider.tsx`, `theme-toggle.tsx`, dep `next-themes`) — eliminarlos cuando se limpie.
- **Por qué:** INV-014 verificó que NO existe CSS light muerto (premisa invertida): el sitio es light-only deliberado; `ThemeProvider`/`next-themes` son wiring inerte.

### R-FE-5: Touch targets accesibles — mínimo 44px en móvil

- **Must:** todo elemento interactivo (botones, icon-buttons, links) tener hit area ≥44×44px en viewports táctiles; mínimo absoluto 24px solo para controles inline secundarios.
- **Must not:** dejar icon-buttons de 14px (ni ningún target <24px) en el sitio.
- **Por qué:** auditoría INV-015 encontró ~24 componentes <44px (2 icon-buttons de 14px) que no cumplen WCAG 2.2 / buenas prácticas móviles.

### R-FE-6: Tokens de motion — sin easing/durations hardcodeados

- **Must:** definir duraciones y easings como tokens en `globals.css` (`@theme inline` — ej. `--duration-fast/normal/slow`, `--ease-default`) y referenciarlos desde componentes, CSS y JS de animación.
- **Must not:** hardcodear `cubic-bezier(0.2,0.8,0.2,1)` en más lugares (hoy está en ~15: `reveal.tsx`, `faq-section.tsx`, `page-transition.tsx`, `latency-comparator.tsx`, `benchmark-race.tsx` + ~10 utilities/keyframes de `globals.css`).
- **Por qué:** sin tokens centralizados, cambiar el lenguaje de motion exige editar ~15 sitios y el sistema se desincroniza (INV-016).

<!-- Referencias cruzadas: → ver release-ci.md, api-contract.md -->
