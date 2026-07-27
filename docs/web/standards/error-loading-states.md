# Error, Loading y Páginas de Error — VantaDB Web Frontend

## Estado Actual

La app web no implementa ninguno de los mecanismos que Next.js 16 App Router provee para manejar errores, estados de carga, o 404s canónicos. Todo está por construir.

| Mecanismo | Estado | Impacto |
|-----------|--------|---------|
| `error.tsx` | ❌ No existe en ninguna ruta | Errores burbujean al error overlay de Next.js (dev) o pantalla blanca (prod) |
| `loading.tsx` | ❌ No existe en ninguna ruta | Sin Suspense boundaries ni feedback visual de carga |
| `not-found.tsx` | ❌ No existe | No hay página de 404 canónica |
| `ErrorBoundary` (React) | ❌ No se usa | Sin recovery de errores en componentes anidados |
| `React.Suspense` | ❌ No se usa | Sin lazy loading ni streaming |
| `try/catch` en pages | ❌ No hay | El único `try/catch` está en `opengraph-image.tsx` (lectura PNG de mascota) |
| `[...slug]` catch-all | ✅ Existe | 404 visual pero fuera de SiteShell |
| Inline 404 en `blog/[slug]` | ✅ Existe | Renderiza "Post Not Found" si el slug no existe |
| Inline 404 en `case-studies/[slug]` | ✅ Existe | Renderiza "Case Study Not Found" si el slug no existe |

## Catch-all `[...slug]`

Ubicación: `web/src/app/[...slug]/page.tsx`

Es un componente `"use client"` que captura todas las rutas que no coinciden con ninguna ruta definida. Renderiza:

- Badge neon `404` con etiqueta "No route matched the requested path"
- Cuerpo "Nothing to see here" con texto explicativo i18n
- 5 sugerencias como grid de tarjetas (Home, Docs, Benchmarks, Pricing, Blog)
- Botones "Back to Home" y "Go back"
- Metadata `robots: noindex, nofollow`, canonical `/404`

### Problemas conocidos

1. **Fuera de SiteShell** — el catch-all renderiza contenido standalone, sin navbar, footer, CommandPalette, ShortcutOverlay, ni easter egg tipográfico. El usuario en una página 404 no puede navegar excepto por los botones y links sugeridos. Bisagra con el layout raíz: SiteShell envuelve rutas conocidas, `[...slug]]` se renderiza fuera de él. Para integrarlo, SiteShell debería ser un componente que cada página decida usar, no un wrapper automático del layout.

2. **Ruta dinámica** — Next.js trata `[...slug]` como dinámica, impidiendo pre-render estático. La metadata (robots, canonical) se define inline en el componente vía `useEffect` o directamente en JSX; no hay un `generateMetadata` exportado.

3. **Sin `notFound()` de `next/navigation`** — el catch-all no llama `notFound()`, por lo que Next.js nunca activa el mecanismo de `not-found.tsx` (que tampoco existe). Esto es intencional dado que no hay `not-found.tsx`, pero bloquea la migración futura.

## Inline 404s en páginas dinámicas

`blog/[slug]/page.tsx` y `case-studies/[slug]/page.tsx` siguen el mismo patrón: buscan el slug en el array de datos estáticos (`BLOG_POSTS` / `CASE_STUDIES`) y si no existe, renderizan inline un mensaje de "not found" con un botón para volver al listado.

Ninguna usa `notFound()` de `next/navigation`. Ventaja: evitan un page boundary y mantienen el layout intacto. Desventaja: no hay un componente reutilizable; cada ruta duplica la misma estructura de fallback.

## Patrón recomendado para nuevas rutas

### error.tsx

```tsx
"use client";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="animate-rise">
      {/* contenido del error — ver roadmap en Roadmap > Error components */}
    </div>
  );
}
```

- Crear en `web/src/app/<ruta>/error.tsx` por cada segmento de ruta que lo requiera
- El componente recibe `error` y `reset()`. `reset()` intenta re-renderizar el segmento
- No necesita `useEffect` para loguear el error — eso va en un servicio aparte (aún no existe)

### loading.tsx

```tsx
export default function Loading() {
  return (
    <div className="animate-rise">
      {/* skeleton o spinner — ver Roadmap > Loading components */}
    </div>
  );
}
```

- Crear en `web/src/app/<ruta>/loading.tsx`
- Next.js lo envuelve automáticamente en un Suspense boundary
- Para skeletons parciales, usar `<Suspense fallback={...}>` directamente en el componente padre

### not-found.tsx

```tsx
import Link from "next/link";

export default function NotFound() {
  return (
    <div className="animate-rise">
      {/* 404 canónico con navbar y footer — ver Roadmap > Error components */}
    </div>
  );
}
```

- Crear en `web/src/app/not-found.tsx` (404 global)
- Opcionalmente en `web/src/app/<ruta>/not-found.tsx` para 404s específicos de segmento
- Llamar `notFound()` de `next/navigation` desde las páginas para activarlo

### Migración del catch-all actual

Para reemplazar `[...slug]` por `not-found.tsx`:

1. Crear `web/src/app/not-found.tsx` con el mismo contenido visual de `[...slug]/page.tsx`
2. Eliminar `web/src/app/[...slug]/page.tsx`
3. Exportar `generateMetadata` en `not-found.tsx` con `robots: { index: false, follow: false }` y canonical `/404`
4. Verificar que ninguna ruta válida dependa del catch-all (no debería, dado que todas las rutas están explícitamente definidas)

## i18n de errores

Keys disponibles en `dictionaries.ts` para usar en componentes de error:

```ts
// Errores genéricos
"common.error":        "Error"
"common.tryAgain":     "Try again"
"common.loading":      "Loading..."

// 404
"error.notFound":      "Page not found"
"error.notFoundDesc":  "The page you're looking for doesn't exist or has been moved."
"error.backHome":      "Go back home"

// 500
"error.serverError":       "Server error"
"error.serverErrorDesc":   "Something went wrong. Please try again."

// Página 404 actual (catch-all)
"notFoundPage.*":   8 keys (title, subtitle, tag)
"notFound.*":       6 keys (headerNote, bodyTitle, bodyText, backToHome, goBack, suggestedRoutes, open, note)
```

Al crear componentes de error nuevos, usar las keys de `error.*` para consistencia. Las keys `notFoundPage.*` y `notFound.*` están asociadas al catch-all actual y deberían migrarse a usar `error.*` cuando se cree `not-found.tsx`.

## Checklist de implementación

- [ ] Crear `web/src/app/not-found.tsx` con el contenido visual de `[...slug]/page.tsx` pero dentro de SiteShell (navbar + footer)
- [ ] Migrar metadata del catch-all a `generateMetadata` en `not-found.tsx`
- [ ] Eliminar `web/src/app/[...slug]/page.tsx` tras validar que `not-found.tsx` lo reemplaza correctamente
- [ ] Crear `web/src/app/error.tsx` (error global del root segment)
- [ ] Crear `web/src/app/loading.tsx` (loading global)
- [ ] Extraer el patrón inline 404 de `blog/[slug]/page.tsx` a un componente `<InlineNotFound backHref="/blog" backLabel={...} />`
- [ ] Extraer el mismo patrón de `case-studies/[slug]/page.tsx` — ambos son funcionalmente idénticos
- [ ] Usar `notFound()` de `next/navigation` en `blog/[slug]` y `case-studies/[slug]` en vez de inline return
- [ ] Agregar `error.tsx` y `loading.tsx` a rutas que eventualmente tengan data fetching asíncrono
- [ ] No implementar diseño de componentes de error hasta que el roadmap lo especifique (este documento es solo el andamiaje de archivos)

## Roadmap (no implementado aquí)

- Diseño de componentes visuales: `<ErrorPage>`, `<LoadingSkeleton>`, `<NotFoundPage>`
- Servicio de logging de errores (actualmente no existe)
- ErrorBoundary class component para wraps manuales
- Soporte de error recovery con `reset()` en páginas transientes
