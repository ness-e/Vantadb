# Known Issues — Frontend Web

**Archivo:** `docs/web/reference/known-issues.md`
**Propósito:** Catálogo vivo de bugs, deuda técnica, y problemas conocidos del frontend web de VantaDB, extraídos de auditorías de código y análisis de agentes.
**Mantenimiento:** Este documento se actualiza cuando se descubre un nuevo issue o se resuelve uno existente. Cada issue debe incluir archivo:línea, impacto, y dirección de solución. No reemplaza a `QA.md` (que cubre scores cuantitativos) ni a `reference/deps-audit.md` (dependencias en detalle).

---

## 🔴 Críticos

Issues que afectan SEO, producción, o funcionalidad core. Deben resolverse antes del próximo deploy.

### 1. `<html lang="es">` hardcodeado
- **Archivo:** `layout.tsx:85`
- El sitio soporta ES/EN vía `LanguageProvider`, pero el atributo `lang` del HTML siempre es `"es"`.
- **Impacto SEO:** Google indexa todo como español.
- **Impacto a11y:** Lectores de pantalla usan acento español cuando el usuario está en EN.
- **Solución:** Dinámico según `lang` del `LanguageProvider`.

### 2. Dark mode no implementado (CSS faltante)
- `next-themes` instalado, `ThemeProvider` configurado, `ThemeToggle` funcional.
- **No hay selectores `.dark` en `globals.css`.** El toggle no tiene efecto visual.
- `enableSystem: false` (solo manual).
- **Solución:** Definir variables CSS para `.dark` o migrar a tema claro-only.

### 3. `metadataBase` no configurado
- El root layout NO define `metadataBase`.
- OG images resuelven a `http://localhost:3000` en desarrollo.
- En producción, las URLs de redes sociales pueden ser incorrectas.
- **Solución:** Agregar `metadataBase: new URL("https://vantadb.dev")` en `next.config.ts`.

### 4. `openGraph.url` apunta a GitHub
- En root layout: `url: "https://github.com/ness-e/Vantadb"`.
- Redes sociales enlazarán a GitHub, no al sitio web.
- **Solución:** Cambiar a `https://vantadb.dev`.

---

## 🟡 Altos

Deuda técnica significativa, código muerto, waste de mantenimiento y bundle.

### 5. Dead components (4 archivos, ~1,157 líneas)
- `navbar.tsx` (546) — reemplazado por `site-navbar.tsx`.
- `hero-mark-interactive.tsx` (312) — reemplazado por `mark/` directory.
- `metrics-bar.tsx` (131) — eliminado de `HomeView`.
- `ecosystem.tsx` (168) — eliminado de `HomeView`.
- Ninguno tiene imports activos en `src/`.
- **Acción:** Eliminar archivos.

### 6. `tt()` helper duplicado en 36 archivos
- Misma función de ~4 líneas definida en cada `page.tsx` y componente.
- **Acción:** Extraer a `src/lib/i18n-utils.ts` e importar.

### 7. Double-brace `{{}}` keys sin interpolación (8 keys)
- 4 keys ES + 4 keys EN en `dictionaries.ts`.
- Ej: `"No results for \"{{query}}\" with filter \"{{filter}}\""`.
- El sistema i18n (`t()`) no tiene motor de interpolación. Las variables nunca se reemplazan.

### 8. 17 zombie dependencies (~150MB+ waste)
- `@dnd-kit/*`, `@mdxeditor/editor`, `@tanstack/react-query`, `@tanstack/react-table`, `date-fns`, `next-auth`, `next-intl`, `react-syntax-highlighter`, `recharts`, `react-hook-form`, `zod`, `zustand`, `sharp`, `uuid`, `cmdk`, `@hookform/resolvers`, `@reactuses/core`.
- Cero imports en el código de la app.
- **Acción:** Revisar si alguna se usa indirectamente; si no, eliminar.

### 9. `tailwind.config.ts` inerte
- Archivo v3 en proyecto Tailwind v4.
- Tailwind v4 lo ignora completamente. La configuración real está en `globals.css` vía `@theme inline {}`.
- **Acción:** Eliminar o migrar a sintaxis v4 si aplica.

### 10. PWA shortcuts con query params legacy
- `manifest.json`: `"/?view=benchmarks"` y `"/?view=docs"`.
- Deberían ser `"/benchmarks"` y `"/docs"`.

---

## 🟡 Medios

Bugs funcionales y features incompletas que afectan la experiencia de usuario o la mantenibilidad.

### 11. anime.js sin cleanup en `mark-classic.tsx`
- Animaciones con `loop: true` en `useEffect` sin return de cleanup.
- **Impacto:** Memory leak potencial si el componente se desmonta.

### 12. Sin error/loading/not-found boundaries
- No existe `error.tsx`, `loading.tsx`, o `not-found.tsx` en ninguna ruta.
- `[...slug]/page.tsx` reemplaza a `not-found.tsx` pero renderiza sin `SiteShell`.

### 13. OG image usa system font
- `opengraph-image.tsx` usa `fontFamily: "sans-serif"` sin `loadGoogleFont()`.
- Renderizado inconsistente entre OS.

### 14. Sitemap slugs hardcodeados
- No derivados de `BLOG_POSTS` o `CASE_STUDIES`.
- Agregar un post requiere actualizar 2 lugares.

### 15. Sin structured data (JSON-LD)
- Sin schema.org de ningún tipo (`Article`, `FAQPage`, `SoftwareApplication`, etc.).

### 16. Sin hreflang tags
- Sitio bilingüe ES/EN sin tags de alternancia de idioma.

### 17. Blog "next post" siempre muestra el primero
- `blog/[slug]/page.tsx`: `BLOG_POSTS.findIndex(p => p.slug !== post.slug)` — bug lógico. Siempre encuentra el primer slug que no coincida.

### 18. Marquee items duplicados
- Los logos del marquee existen tanto en `navbar.tsx` (muerto) como en `site-navbar.tsx`.
- Se resolverá al eliminar `navbar.tsx` (issue #5).

### 19. Case studies sin fecha/author
- `CASE_STUDIES` data shape no incluye `date` ni `author`.
- `generateMetadata` no puede setear `publishedTime` o `authors`.

### 20. 404 catch-all sin SiteShell
- `[...slug]/page.tsx` renderiza fuera del `SiteShell` (sin navbar, footer, command palette).
- Porque el catch-all layout no incluye `SiteShell`.

---

## 🟢 Bajos

Cosméticos, comentarios desactualizados, limpieza menor.

### 21. Comentarios desactualizados en `vanta-data.ts`
- Mencionan `metrics-bar`, `navbar` — componentes ya eliminados.

### 22. `dictionaries.ts` dice "300+ keys" pero tiene ~880+
- Comentario del header desactualizado.

### 23. `press-neon` CSS sin estado `:hover` definido
- El botón neon de la sección Press no tiene estilo hover.

### 24. Whitespace excesivo en `globals.css`
- ~50 líneas en blanco consecutivas.

### 25. `activeVariant` useState en `mark/index.tsx` nunca cambia
- Podría ser `const`.

### 26. Icons PWA todos apuntan al mismo PNG
- Sin SVG, sin sizes dedicados.

### 27. Aria-labels hardcodeados en ES
- `skip-link`, `theme-toggle`, `command-palette` — labels siempre en español, incluso en modo EN.

---

## Cómo contribuir

Para agregar un nuevo issue:

1. Crear un entry con el formato:
   ```md
   ### N. Título del issue
   - **Archivo:** `ruta:línea`
   - Descripción del problema, contexto, y evidencia.
   - **Impacto:** qué afecta.
   - **Solución:** dirección de la corrección.
   ```
2. Clasificar por severidad (`🔴 Crítico`, `🟡 Alto`, `🟡 Medio`, `🟢 Bajo`).
3. Colocar en la sección correspondiente, manteniendo orden numérico.
4. Cuando un issue se resuelve, moverlo a una sección `✅ Resueltos` al final del documento con la fecha y referencia del PR/commit.
