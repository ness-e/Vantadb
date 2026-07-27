# SEO y Metadata — Guía de Referencia

> **Applies to:** `web/` (Next.js 16 App Router)
> **See also:** `docs/web/reference/stack.md`, `docs/web/guides/routing.md`

---

## 1. Resumen del sistema SEO

El frontend de VantaDB usa el sistema de Metadata nativo de Next.js 16 App Router. Cada ruta exporta un objeto `Metadata` estático desde su `layout.tsx`, o una función `generateMetadata()` en layouts dinámicos. Next.js serializa estos objetos en `<meta>` tags, `<link>` tags, y `robots`/`sitemap` en build time o request time.

La infraestructura SEO se compone de:

- **Root layout** — metadata global, viewport, lang, favicon, manifest
- **29 sub-layouts** — metadata por ruta (patrón estático o dinámico)
- **`opengraph-image.tsx`** — OG image global (1200×630, runtime Node.js)
- **`sitemap.ts`** — 29 entradas, combinación de rutas estáticas y dinámicas
- **`robots.ts`** — directrices para crawlers

No existe `twitter-image.tsx` dedicado; Twitter hereda de la OG image via `twitter:card`.

---

## 2. Layout raíz

**Archivo:** `web/src/app/layout.tsx`

### Metadata export

```ts
export const metadata: Metadata = {
  title: "VantaDB — Embedded Rust Engine for Local-First Hybrid Retrieval",
  description:
    "VantaDB is a local-first, embedded Rust database engine for AI agents and local RAG...",
  keywords: ["VantaDB", "vector database", "Rust", ...],
  authors: [{ name: "ness-e" }],
  icons: {
    icon: "/assets/avatar_gato.png",
    apple: "/assets/avatar_gato.png",
  },
  manifest: "/manifest.json",
  openGraph: {
    title,
    description,
    url: "https://github.com/ness-e/Vantadb",
    siteName: "VantaDB",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title,
    description,
  },
};
```

### Viewport

```ts
export const viewport = {
  themeColor: "#FF5500",
  width: "device-width",
  initialScale: 1,
};
```

### HTML

```html
<html lang="es" suppressHydrationWarning>
```

### Problemas conocidos

| Issue | Impacto | Prioridad |
|-------|---------|-----------|
| `metadataBase` no está definido | OG images, URLs canónicas y Open Graph se resuelven contra `localhost:3000` en producción en vez de `https://vantadb.dev` | **Crítico** |
| `openGraph.url` apunta a `https://github.com/ness-e/Vantadb` | Crawlers de Facebook/X/LinkedIn interpretan que la URL canónica de la página es GitHub | **Crítico** |
| `lang="es"` hardcodeado | Afecta SEO multilingüe y accesibilidad (lectores de pantalla). No refleja el idioma real del contenido (parte del sitio está en inglés) | **Alto** |
| Sin `twitter-image.tsx` | Twitter usará la OG image genérica, sin control sobre el formato específico de `twitter:image` | **Medio** |
| Sin `verification` tags | Google Search Console y otros verificadores requieren meta tags específicos que no están presentes | **Bajo** |
| Solo favicon PNG (`avatar_gato.png`) | Sin SVG favicon para navegadores modernos que soportan `mask-icon` y tema oscuro | **Bajo** |

---

## 3. Layouts por ruta

### Patrón estático (27 layouts)

Todas las rutas estáticas siguen este patrón idéntico:

```ts
export const metadata: Metadata = {
  title: "Page · VantaDB",
  description: "...",
  openGraph: {
    title,
    description,
    url: "https://vantadb.dev/<path>",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/<path>",
  },
};
```

Layouts que usan este patrón:

| Ruta | Layout file |
|------|-------------|
| `/` | `layout.tsx` (root — patrón expandido) |
| `/architecture` | `architecture/layout.tsx` |
| `/benchmarks` | `benchmarks/layout.tsx` |
| `/changelog` | `changelog/layout.tsx` |
| `/config` | `config/layout.tsx` |
| `/cost` | `cost/layout.tsx` |
| `/demo` | `demo/layout.tsx` |
| `/docs` | `docs/layout.tsx` |
| `/docs-api` | `docs-api/layout.tsx` (redirect, `robots: noindex`) |
| `/engine` | `engine/layout.tsx` |
| `/integrations` | `integrations/layout.tsx` |
| `/latency` | `latency/layout.tsx` |
| `/maint` | `maint/layout.tsx` |
| `/playground` | `playground/layout.tsx` |
| `/pricing` | `pricing/layout.tsx` |
| `/security` | `security/layout.tsx` |
| `/showcase` | `showcase/layout.tsx` |
| `/storage` | `storage/layout.tsx` |
| `/use-cases` | `use-cases/layout.tsx` |
| `/why-vantadb` | `why-vantadb/layout.tsx` |
| `/about/company` | `about/company/layout.tsx` |
| `/about/community` | `about/community/layout.tsx` |
| `/about/contact` | `about/contact/layout.tsx` |
| `/about/team` | `about/team/layout.tsx` |
| `/blog` | `blog/layout.tsx` |
| `/case-studies` | `case-studies/layout.tsx` |
| `/solutions/ai-agents` | `solutions/ai-agents/layout.tsx` |
| `/solutions/ai-ide-tooling` | `solutions/ai-ide-tooling/layout.tsx` |
| `/solutions/local-rag` | `solutions/local-rag/layout.tsx` |
| `/*` (404) | `[...slug]/layout.tsx` (`robots: noindex`) |

### Patrón dinámico: blog

**Archivo:** `web/src/app/blog/[slug]/layout.tsx`

Usa `generateMetadata()` que busca en `BLOG_POSTS` (definido en `vanta-data.ts`):

```ts
export async function generateMetadata({ params }): Promise<Metadata> {
  const { slug } = await params;
  const post = BLOG_POSTS.find((p) => p.slug === slug);
  if (!post) return { title: "Not Found · VantaDB", robots: { index: false, follow: false } };
  return {
    title: `${post.title} · VantaDB Blog`,
    description: post.excerpt,
    openGraph: {
      title, description, url,
      siteName: "VantaDB",
      type: "article",
      publishedTime: post.date,
      authors: [post.author],
    },
    alternates: { canonical: url },
  };
}
```

- Incluye `publishedTime` y `authors` — correcto para Article schema
- Fallback a 404 con `noindex` si el slug no existe

### Patrón dinámico: case-studies

**Archivo:** `web/src/app/case-studies/[slug]/layout.tsx`

Mismo patrón pero con `CASE_STUDIES`:

```ts
export async function generateMetadata({ params }): Promise<Metadata> {
  const { slug } = await params;
  const cs = CASE_STUDIES.find((c) => c.slug === slug);
  if (!cs) return { title: "Not Found · VantaDB", robots: { index: false, follow: false } };
  return {
    title: `${cs.title} · VantaDB Case Study`,
    description: cs.summary,
    openGraph: {
      title, description, url,
      siteName: "VantaDB",
      type: "article",
    },
    alternates: { canonical: url },
  };
}
```

- **Carencia:** no incluye `publishedTime` — el tipo de datos `CASE_STUDIES` no expone fecha

---

## 4. OG Image

**Archivo:** `web/src/app/opengraph-image.tsx`

### Configuración

| Propiedad | Valor |
|-----------|-------|
| Runtime | `nodejs` (usa `fs.readFile`) |
| Tamaño | 1200×630 PNG |
| Content-Type | `image/png` |
| Alt | "VantaDB — Embedded Rust Engine for Local-First Hybrid Retrieval" |

### Layout visual

```
┌──────────────────────────────────────────────┐
│  ┌─────────────────────┐  ┌──────────────┐   │
│  │ [v0.1 · MVP] [Embedded·Rust] │  │              │   │
│  │                     │  │              │   │
│  │  VANTA              │  │   🐱 / img  │   │
│  │  DB (naranja)      │  │              │   │
│  │                     │  │              │   │
│  │  Tagline...         │  │              │   │
│  │                     │  │              │   │
│  │  ┌─────────────────┐│  │              │   │
│  │  │1.2ms│5,400│100%│0││  │              │   │
│  │  └─────────────────┘│  └──────────────┘   │
│  └─────────────────────┘                      │
└──────────────────────────────────────────────┘
```

### Problemas

- **Sin fuentes custom:** usa `fontFamily: "sans-serif"` en vez de `loadGoogleFont()` con Geist/Anton. El renderizado varía entre sistemas operativos.
- **Sin OG images dinámicas:** no existe `blog/[slug]/opengraph-image.tsx` ni `case-studies/[slug]/opengraph-image.tsx`. Cada artículo comparte la misma imagen genérica.
- Fallback a emoji 🐱 si no encuentra `public/assets/mascota_gato.png`.

---

## 5. Sitemap

**Archivo:** `web/src/app/sitemap.ts`

Genera `MetadataRoute.Sitemap` con `https://vantadb.dev` hardcodeada como base URL. Usa `new Date()` para `lastModified` — misma fecha para todas las entradas.

### Composición actual (29 entradas)

**22 rutas estáticas:**

`/`, `/benchmarks`, `/docs`, `/engine`, `/architecture`, `/playground`, `/changelog`, `/why-vantadb`, `/pricing`, `/security`, `/use-cases`, `/cost`, `/maint`, `/solutions/ai-agents`, `/solutions/local-rag`, `/solutions/ai-ide-tooling`, `/blog`, `/case-studies`, `/about/company`, `/about/team`, `/about/community`, `/about/contact`

**4 blog slugs hardcodeados:**
`introducing-vantadb`, `how-hybrid-search-works`, `sqlite-for-ai-agents`, `why-i-built-vantadb-local-memory-engine`

**3 case-study slugs hardcodeados:**
`agent-local-memory-ollama`, `rag-edge-device`, `ide-semantic-search`

### Rutas que faltan en el sitemap

Las siguientes rutas existen en el App Router (tienen su propio `layout.tsx`) pero no están incluidas en `sitemap.ts`:

| Ruta | Prioridad sugerida |
|------|-------------------|
| `/config` | 0.7 |
| `/demo` | 0.6 |
| `/storage` | 0.7 |
| `/integrations` | 0.7 |
| `/showcase` | 0.6 |
| `/latency` | 0.5 |

Nota: `/docs-api` es un redirect a `/docs` y está correctamente excluido con `noindex`.

### Problemas

- **Slugs hardcodeados:** no derivan de `BLOG_POSTS` ni `CASE_STUDIES` — si se agrega un nuevo post sin actualizar el sitemap, no será indexado hasta el próximo crawl de Google.
- **`lastModified` uniforme:** `new Date()` en cada build da la misma fecha a todas las rutas. Google usa `lastModified` como señal de frescura.
- **Sin `changeFrequency` granular:** la mayoría usa `"monthly"` — rutas como `/changelog` o `/blog` deberían ser `"weekly"`.

---

## 6. Robots

**Archivo:** `web/src/app/robots.ts`

```ts
export default function robots(): MetadataRoute.Robots {
  return {
    rules: [
      {
        userAgent: "*",
        allow: "/",
        disallow: ["/api/", "/_next/"],
      },
    ],
    sitemap: "https://vantadb.dev/sitemap.xml",
    host: "https://vantadb.dev",
  };
}
```

- Permite crawling completo excepto `/api/` y `/_next/`
- Apunta al sitemap
- Incluye `host` (útil para webmaster tools)

---

## 7. Cómo agregar metadata a una nueva ruta

### Ruta estática

Crear `ruta/layout.tsx` con el patrón estándar:

```ts
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Page Name · VantaDB",
  description: "Descripción única de 150-160 caracteres que incluye keywords principales.",
  openGraph: {
    title: "Page Name · VantaDB",
    description: "Descripción OG (puede ser más corta que la meta description).",
    url: "https://vantadb.dev/ruta",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/ruta",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
```

### Ruta dinámica (blog, case-studies)

```ts
import type { Metadata } from "next";
import { DATA } from "@/components/vanta/vanta-data";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const item = DATA.find((d) => d.slug === slug);
  if (!item) {
    return {
      title: "Not Found · VantaDB",
      robots: { index: false, follow: false },
    };
  }
  const url = `https://vantadb.dev/ruta/${item.slug}`;
  return {
    title: `${item.title} · VantaDB`,
    description: item.excerpt || item.summary,
    openGraph: {
      title: `${item.title} · VantaDB`,
      description: item.excerpt || item.summary,
      url,
      siteName: "VantaDB",
      type: "article",
      publishedTime: item.date,     // si existe
      authors: [item.author],       // si existe
    },
    alternates: { canonical: url },
  };
}
```

### Agregar la ruta al sitemap

En `sitemap.ts`:

```ts
// Si es estática
{ url: `${baseUrl}/ruta`, lastModified, changeFrequency: "monthly", priority: 0.7 },

// Si es dinámica: derivar los slugs de la fuente de datos en vez de hardcodearlos
const slugs = DATA.map((d) => d.slug);
const dynamicRoutes = slugs.map((slug) => ({
  url: `${baseUrl}/ruta/${slug}`,
  lastModified,
  changeFrequency: "yearly",
  priority: 0.6,
}));
```

---

## 8. Known issues y roadmap

### Críticos (arreglar antes de producción)

1. **`metadataBase` faltante** en `layout.tsx:33`
   ```ts
   export const metadata: Metadata = {
     metadataBase: new URL("https://vantadb.dev"),  // ← agregar
     title: "...",
   };
   ```
   Sin esto, las URL relativas en OG images y alternates se resuelven contra `localhost`.

2. **`openGraph.url` apunta a GitHub**
   Cambiar `url: "https://github.com/ness-e/Vantadb"` por `url: "https://vantadb.dev"`.

3. **`lang="es"` hardcodeado**
   Reemplazar por `<html lang={detectedLang}>` usando el `LanguageProvider` existente o la cabecera `Accept-Language`.

### Medios (siguiente iteración)

4. **OG images dinámicas por ruta**
   Implementar `blog/[slug]/opengraph-image.tsx` y `case-studies/[slug]/opengraph-image.tsx` con el título y excerpt de cada post.

5. **`twitter-image.tsx`**
   Agregar imagen dedicada para Twitter con dimensiones 1200×600 (aspect ratio 2:1).

6. **Fuentes custom en OG image**
   Usar `loadGoogleFont("Geist")` y `loadGoogleFont("Anton")` en `opengraph-image.tsx` para renderizado consistente cross-OS.

7. **Sitemap derivado de datos**
   Extraer slugs de `BLOG_POSTS` y `CASE_STUDIES` en vez de hardcodearlos. Agregar las 6 rutas faltantes.

8. **`publishedTime` en case-studies**
   Agregar campo `date` al tipo `CASE_STUDIES` y exponerlo en `openGraph.publishedTime`.

9. **JSON-LD (structured data)**
   Implementar `WebSite` schema en el root layout y `Article`/`BlogPosting` schema en layouts dinámicos. Usar `ld+json` inline.

10. **Hreflang tags**
    Si el contenido existe en ES y EN, agregar `<link rel="alternate" hreflang="en">` usando las rutas del `LanguageProvider`.

### Bajos (nice to have)

11. **Google Search Console verification tag**
    Agregar `<meta name="google-site-verification" content="...">` vía `metadata.other`.

12. **SVG favicon**
    Agregar `/favicon.svg` con `sizes="any"` y `type="image/svg+xml"`.

13. **`lastModified` granular en sitemap**
    Usar fechas reales de `git log --format=%cI` o fechas de `BLOG_POSTS`/`CASE_STUDIES` en vez de `new Date()`.

14. **PWA shortcuts sin query params legacy**
    Revisar `/manifest.json` y reemplazar `?view=` por rutas App Router reales.

> **Tracking:** estos issues están documentados en `docs/web/audit/SEO-AUDIT.md` con seguimiento individual.
