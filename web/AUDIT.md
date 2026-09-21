# Auditoría — VantaDB Web

> ## ⚠️ ADDENDUM 2026-08-24 — La mayoría de los issues de abajo fueron RESUELTOS
> Auditoría completa de 7 dimensiones ejecutada (WDA-00..07, commits `fbc404b5..53785dfd`).
> **Reporte canónico:** `docs/reviews/web-design-audit-2026-08-24.md` · Detalles por tarea: `.opencode/skills/campaign-executor/tasks/WDA-0*.md`
>
> Resuelto: claims falsos (recall/p50/snippets/versión/changelog/blog) · dominio muerto → SITE_URL centralizada · sitemap +7 rutas · not-found.tsx · tt() ×36 → lib · dark-mode huérfano eliminado (R-FE-4: light-only) · leak animejs + timers sin cleanup · 13 deps zombies · ui/ muerta (46 archivos) · doble Toaster · contrastes a11y ×24 · touch targets · literales sin i18n · hero value prop + CTA primario · TCO "Team plan" fantasma · funnel /demo honesto · JSON-LD logo.
> Pendiente (ver reporte): densidad de efectos en home, pricing mid-tier (decisión producto), assets faltantes en public/assets, re-medir Lighthouse post (EPERM ambiental).

**Fecha original:** 2026-07-26  
**Proyecto:** `vantadb-web` (Next.js 16 + Tailwind v4 + shadcn/ui)  
**Stack real:** Next.js 16 App Router · TypeScript · Tailwind v4 · framer-motion 12 · animejs 4 (cleanup fixed WDA-01)

---

## 🟥 Críticos (4)

### 1. AGENTS.md desactualizado

**Archivo:** `AGENTS.md`  
**Severidad:** Crítico  
**Descripción:** El documento describe una SPA con 3 vistas manejadas por `useState<View>` y "sin ruteo de Next.js". La realidad es que el proyecto usa el App Router completo con 28 rutas reales, `router.push()`, layouts, metadata dinámica, sitemap y robots.

**Evidencia:**

| Afirmación en AGENTS.md | Realidad actual |
|---|---|
| "3 views (home/benchmarks/docs) managed via useState<View>" | 28+ rutas en `src/app/`, cada una con su `page.tsx` |
| "no Next.js routing" | `next/navigation` (useRouter, usePathname) usado en 12+ archivos |
| "Single page at /" | 35 rutas generadas en build (`/`, `/benchmarks`, `/pricing`, `/blog/[slug]`, etc.) |

**Impacto:** Cualquier desarrollador nuevo se guiará por documentación incorrecta.

---

### 2. `<html lang="es">` hardcodeado

**Archivo:** `src/app/layout.tsx:85`  
**Severidad:** Crítico  
**Descripción:** El atributo `lang` del `<html>` está fijo a `"es"` sin importar el idioma seleccionado por el usuario en `LanguageProvider`. Afecta SEO (Google interpreta todo el sitio como español) y accesibilidad (lectores de pantalla usan el acento incorrecto cuando el usuario cambia a inglés).

```tsx
// Actual — siempre español
<html lang="es">

// Debería ser dinámico
<html lang={currentLang}>
```

---

### 3. Modo oscuro no funcional

**Archivos:** `globals.css`, `tailwind.config.ts`, `theme-provider.tsx`, `theme-toggle.tsx`  
**Severidad:** Crítico  
**Descripción:** `next-themes` y `ThemeProvider` están instalados y configurados, pero `globals.css` **no define variables CSS para `:root.dark` o `@media (prefers-color-scheme: dark)`**. El `ThemeToggle` renderiza un botón de cambio, pero al activarlo no hay colores oscuros definidos — la UI se rompe visualmente.

Además, el `ThemeToggle` solo existe en `navbar.tsx` (componente muerto). El navbar activo (`site-navbar.tsx`) ni siquiera lo importa.

---

### 4. Catch-all 404 sin `notFound()`

**Archivo:** `src/app/[...slug]/page.tsx`  
**Severidad:** Crítico  
**Descripción:** La página 404 usa una catch-all route (`[...slug]`) en vez de `next/navigation` `notFound()` con el archivo canónico `not-found.tsx`. Esto causa que las rutas inválidas rendericen sin el `<SiteShell>` (sin navbar, sin footer, sin command palette), rompiendo la experiencia de navegación.

Además, Next.js genera la ruta `/[...slug]` como dinámica (ƒ) en vez de estática, lo que impide el pre-renderizado.

---

## 🟧 Altos (6)

### 5. Código muerto — 4 componentes completos

| Archivo | Líneas | Estado |
|---|---|---|
| `src/components/vanta/navbar.tsx` | 546 | Reemplazado por `site-navbar.tsx` |
| `src/components/vanta/hero-mark-interactive.tsx` | 312 | Reemplazado por `mark-classic.tsx` |
| `src/components/vanta/metrics-bar.tsx` | 131 | Eliminado del HomeView |
| `src/components/vanta/ecosystem.tsx` | 168 | Eliminado del HomeView |
| **Total código muerto** | **1,157 líneas** | |

Ninguno de estos archivos es importado por ningún otro archivo en `src/`.

---

### 6. 17 dependencias zombies

Instaladas en `package.json` pero con cero imports en cualquier archivo `.ts`/`.tsx`:

| Paquete | Categoría | Tamaño estimado |
|---|---|---|
| `@dnd-kit/core` | Drag & drop | ~80KB |
| `@dnd-kit/sortable` | Drag & drop | ~30KB |
| `@dnd-kit/utilities` | Drag & drop | ~20KB |
| `@hookform/resolvers` | Validación forms | ~15KB |
| `@mdxeditor/editor` | Editor MDX | ~500KB |
| `@reactuses/core` | Hooks utilities | ~50KB |
| `@tanstack/react-query` | Data fetching | ~100KB |
| `@tanstack/react-table` | Tablas | ~120KB |
| `date-fns` | Fechas | ~300KB |
| `next-auth` | Autenticación | ~200KB |
| `next-intl` | i18n | ~100KB |
| `react-markdown` | Markdown | ~50KB |
| `react-syntax-highlighter` | Syntax highlighting | ~800KB |
| `sharp` | Imágenes | ~30MB (nativo) |
| `uuid` | UUID | ~10KB |
| `z-ai-web-dev-sdk` | SDK AI | ~50KB |
| `zod` | Schemas | ~50KB |
| `zustand` | Estado global | ~10KB |

**Ahorro potencial:** ~150MB+ en `node_modules` y ~30s en install.

---

### 7. Bloque muerto en Hero

**Archivo:** `src/components/vanta/hero.tsx:166-195`  
**Severidad:** Alto  
**Descripción:** Un bloque de 30 líneas envuelto en `{false && (...)}` que contiene la lógica para mostrar el gato mascota. La variable de estado `heroVariant` se inicializa como `"mark"` y nunca cambia (el setter nunca se llama). El código compila pero nunca se ejecuta.

```tsx
// Este bloque NUNCA se renderiza
{false && heroVariant === "mascot" && (
  <div className="...">
    <Image ... alt="VantaDB cat mascot" />
  </div>
)}
```

---

### 8. `tailwind.config.ts` inerte

**Archivo:** `tailwind.config.ts`  
**Severidad:** Alto  
**Descripción:** Este archivo está escrito en sintaxis de **Tailwind CSS v3** usando `hsl(var(--...))`. El proyecto usa **Tailwind CSS v4** donde la configuración de tema se declara en `globals.css` con la directiva `@theme inline {}`. El archivo `tailwind.config.ts` no tiene ningún efecto en el build.

Los colores reales están definidos en `globals.css:53-87` como variables CSS nativas con valores hex (`#FBF9F5`, `#000000`), no HSL. El `tailwind.config.ts` referencia `hsl(var(--background))` que mapea a `var(--background)` que a su vez es `#FBF9F5` — funciona porque el navegador interpreta `#FBF9F5` como color aunque esté dentro de `hsl()`, pero es engañoso.

---

### 9. Helper `tt()` triplicado

**Archivos:** `hero.tsx:20-23`, `cta-final.tsx:14-17`, `footer.tsx:14-17`  
**Severidad:** Alto  
**Descripción:** La función de fallback para traducciones está definida idénticamente en tres archivos separados:

```tsx
const tt = (key: string, fallback: string) => {
  const val = t(key);
  return val === key ? fallback : val;
};
```

**Solución:** Extraer a `src/lib/utils.ts` o `src/lib/i18n-utils.ts`.

---

### 10. Componente `SfxLabel` triplicado

**Archivos:** `hero.tsx:278-301`, `mark-classic.tsx:251-273`, `hero-mark-interactive.tsx:289-312`  
**Severidad:** Alto  
**Descripción:** El componente decorativo `SfxLabel` (para etiquetas estilo manga como "1.2ms", "RRF", "WAL") está definido con código idéntico en tres archivos:

```tsx
const SfxLabel = ({ children, className, ... }: { ... }) => (
  <span className={`font-mono text-[0.55rem] ...`} {...}>{children}</span>
);
```

---

## 🟨 Medios (8)

### 11. Hardcoded English en toasts

**Archivos:** `site-navbar.tsx:247`, `navbar.tsx:261`  
**Línea:** `toast.info(\`${t(item.labelKey)} — coming soon\`)`  
**Descripción:** El texto `"— coming soon"` está hardcodeado en inglés y no pasa por i18n. Cuando el sitio está en español, los toasts muestran "X — coming soon".

---

### 12. Hardcoded English en mark interactivo

**Archivo:** `mark-classic.tsx:244`  
**Descripción:** Las etiquetas de interacción `"◆ blink"` y `"◆ click me · move mouse"` están hardcodeadas en inglés.

---

### 13. Hardcoded Spanish en aria-labels

| Archivo | Línea | Texto |
|---|---|---|
| `src/components/vanta/back-to-top.tsx` | 43 | `"Volver arriba"` |
| `src/components/vanta/theme-toggle.tsx` | 22-24 | `"Activar tema claro/oscuro"` |
| `src/app/layout.tsx` | 91 | `"Saltar al contenido"` |
| `src/components/vanta/command-palette.tsx` | 224, 231, 332 | `"Buscar"`, `"Buscar páginas..."`, `"Toolbar"` |

Estos textos no se traducen cuando el usuario cambia el idioma.

---

### 14. Etiquetas decorativas sin `aria-hidden`

**Archivo:** `mark-classic.tsx:223-238`  
**Descripción:** Las etiquetas `SfxLabel` ("1.2ms", "RRF", "WAL · CRC32C", "ZERO NET", "IN-PROCESS") son puramente decorativas pero no tienen `aria-hidden="true"`. Los lectores de pantalla las leerán sin contexto, creando ruido.

---

### 15. Círculos interactivos sin nombre accesible

**Archivo:** `mark-classic.tsx:140-152`  
**Descripción:** Los círculos invisibles que definen las zonas interactivas de los nodos del grafo no tienen `aria-label` ni `role`. Un usuario de teclado no puede interactuar con ellos.

```tsx
{GRAPH_NODES.map((node, i) => (
  <circle
    key={node.id}
    ref={(el) => { nodeRefs.current[i] = el; }}
    cx={node.cx} cy={node.cy} r={18}
    fill="transparent" className="cursor-pointer"
    // Falta: aria-label, role, tabIndex
  />
))}
```

---

### 16. Eventos sintéticos frágiles

**Archivos:** `site-shell.tsx`, `site-navbar.tsx`  
**Descripción:** Para abrir la paleta de comandos y el overlay de atajos, los componentes disparan eventos `KeyboardEvent` sintéticos:

```tsx
// site-navbar.tsx:92-97
const openCommandPalette = () => {
  window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", metaKey: true }));
};
```

Esto crea un acoplamiento frágil entre componentes. Si `CommandPalette` cambia su atajo, el resto del sistema se rompe silenciosamente.

---

### 17. Build warnings

- **`turbopack.root` no configurado** — Next.js detecta múltiples lockfiles y elige `C:\Users\Eros\package-lock.json` como raíz en vez del directorio del proyecto. Causa: tener `package-lock.json` tanto en `~` como en el proyecto.
- **`metadataBase` no definido** — Las imágenes de Open Graph usarán `http://localhost:3000` como URL base.

---

### 18. Animaciones anime.js sin cleanup explícito

**Archivo:** `mark-classic.tsx:76-93`  
**Descripción:** El `useEffect` inicia animaciones de bucle (`shimmer`, `rotateRing`) que continúan incluso después de que el componente se desmonta. El cleanup comentado dice: *"Cleanup handled by anime.js on unmount"*, pero anime.js no limpia automáticamente al desmontar — solo si se llama `.kill()` o `.pause()` explícitamente.

**Riesgo:** Memory leaks y animaciones fantasma si el componente se monta/desmonta repetidamente.

---

## 🟩 Bajos (6)

### 19. `useState` innecesario en mark/index.tsx

**Archivo:** `src/components/vanta/mark/index.tsx:18`  
**Descripción:** El estado `activeVariant` se inicializa como `"classic"` pero nunca cambia. Podría ser una constante.

```tsx
// Actual
const [activeVariant] = useState<MarkVariantName>(variant ?? "classic");

// Podría ser
const activeVariant = variant ?? "classic";
```

---

### 20. Espaciado excesivo en globals.css

**Archivo:** `globals.css:389-461, 464-517, 556-557`  
**Descripción:** ~50 líneas en blanco consecutivas. Sin impacto funcional, pero desorden visual.

---

### 21. Comentarios que referencian dead code

**Archivo:** `src/components/vanta/vanta-data.ts`  
**Líneas:** 22, 40  
**Descripción:**

- Línea 22: `"Core metrics (hero, metrics-bar, cta-final, trust-section, benchmarks)"` — `metrics-bar` es dead code.
- Línea 40: `"navbar"` — referencia al navbar viejo, reemplazado por `site-navbar`.

---

### 22. Comentario desactualizado en dictionaries.ts

**Archivo:** `src/lib/dictionaries.ts:2`  
**Descripción:** El comentario dice `"300+ keys"` pero hay aproximadamente 880+ keys reales.

---

### 23. `press-neon` sin hover definido

**Archivo:** `globals.css:138-151`  
**Descripción:** La clase `press-neon` no tiene un estado `:hover` definido (solo `:active` y el estado base). Se usa en `navbar.tsx` (dead code) pero `site-navbar.tsx` ni siquiera la usa.

---

### 24. Marquee items duplicados

**Archivos:** `navbar.tsx` y `site-navbar.tsx`  
**Descripción:** El array de ítems del marquee (barra superior animada) está copiado y pegado en ambos archivos. Si se actualiza uno, el otro queda desincronizado.

---

## Resumen

### Por severidad

| Severidad | Cantidad |
|-----------|----------|
| 🟥 Críticos | 4 |
| 🟧 Altos | 6 |
| 🟨 Medios | 8 |
| 🟩 Bajos | 6 |
| **Total** | **24** |

### Por categoría

| Categoría | Cantidad |
|-----------|----------|
| Código muerto / redundante | 8 |
| i18n / traducciones | 5 |
| Accesibilidad | 4 |
| Configuración / build | 4 |
| Documentación desactualizada | 2 |
| Animaciones / performance | 1 |

### Archivos más problemáticos

| Archivo | Issues | Líneas |
|---|---|---|
| `globals.css` | 3, 8, 20, 23 | 817 |
| `mark-classic.tsx` | 12, 14, 15, 18 | 274 |
| `AGENTS.md` | 1 | - |
| `layout.tsx` | 2, 4, 13 | 111 |
| `site-navbar.tsx` | 11, 16 | 460 |
| `hero.tsx` | 7, 9, 10 | 317 |
| `vanta-data.ts` | 21 | 1,109 |
| `dictionaries.ts` | 22 | 2,830 |

---

*Documento generado automáticamente por auditoría de código.*
