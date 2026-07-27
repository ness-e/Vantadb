# Gestión de Contenido Estático — Frontend Web

> **App:** Next.js 16 App Router (todo `"use client"`)  
> **Última actualización:** Julio 2026

---

## 1. Arquitectura del Contenido

VantaDB no tiene CMS, backend ni API calls. Todo el contenido estático vive en **un solo archivo**:

```
web/src/components/vanta/vanta-data.ts   ← 1109 líneas, 25 exports
```

Las traducciones residen en `dictionaries.ts` (ES/EN). El flujo es unidireccional:

```
vanta-data.ts → page.tsx importa → tt() helper con i18n → render
```

No hay GraphQL, no hay fetch, no hay `getStaticProps`. Es contenido en memoria, disponible desde el primer render del lado del cliente.

---

## 2. `vanta-data.ts` — Inventario Completo de Exports

### 2.1 Metadata de Producto

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `VANTA` | `object` | 4-14 | `name`, `tagline`, `repo`, `pypi`, `discord`, `license`, `rustVersion`, `pythonVersion` |
| `PRODUCT` | `object` | 21-78 | `metrics`, `versions`, `techStack`, `ecosystem`, `distribution`, `hardware` |
| `HERO_STATS` | `array[4]` | 83-88 | Headline stats del hero |
| `View` | `type` | 80 | `"home" \| "benchmarks" \| "docs"` (legacy) |

### 2.2 Features y Benchmarks

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `CORE_CAPABILITIES` | `array[6]` | 91-140 | Features con `icon`, `title`, `mechanism`, `detail`, `tag` |
| `BENCH01` | `object` | 143-175 | Benchmark rows con `metric`, `p50`, `p99`, `throughput` |
| `SIFT1M` | `object` | 178-236 | SIFT1M benchmark con 5 rows |

### 2.3 Contenido de Marquees y Páginas

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `QUICKSTART_PYTHON` | `string` | 239-267 | Código Python de ejemplo |
| `CLI_COMMANDS` | `array[6]` | 270-301 | Comandos CLI con `cmd`, `args`, `desc` |
| `SEARCH_SEMANTICS` | `array[4]` | 304-321 | Bullets de search semantics |
| `PRODUCT_BOUNDARY` | `array[4]` | 324-348 | Construido vs diferido |
| `DOC_LINKS` | `array[8]` | 351-360 | Links a documentación |

### 2.4 FAQ y Tutoriales

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `FAQ` | `array[8]` | 363-396 | Preguntas frecuentes (ES, las traducciones se sobreescriben vía i18n) |
| `TUTORIALS` | `array[4]` | 399-460 | Tutoriales con `steps` |

### 2.5 Changelog

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `CHANGELOG` | `array[3]` | 463-512 | Releases: 0.1.0, 0.1.1, 0.1.2 |

### 2.6 Pricing y Confianza

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `PRICING_PLANS` | `array[3]` | 519-579 | `Community` / `Team` / `Enterprise` |
| `SECURITY_PILLARS` | `array[6]` | 582-619 | 6 pilares de seguridad |
| `TCO_COMPARISON` | `array[4]` | 734-763 | VantaDB vs Pinecone vs Weaviate |
| `MAINTENANCE_PILLARS` | `array[4]` | 766-791 | 4 pilares de mantenimiento |
| `WHY_VANTADB` | `object` | 794-826 | `benefits[]` + `comparison[]` |

### 2.7 Contenido Editorial

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `BLOG_POSTS` | `array[4]` | 833-990 | Posts con slugs, `content` como `{type, text}[]` |
| `CASE_STUDIES` | `array[3]` | 993-1042 | 3 case studies con `metrics`, `challenge`, `solution` |

### 2.8 Compañía

| Export | Tipo | Líneas | Propósito |
|--------|------|--------|-----------|
| `TEAM_MEMBERS` | `array[4]` | 1045-1074 | 4 miembros con nombre, rol, bio |
| `COMPANY_INFO` | `object` | 1077-1108 | `mission`, `principles`, `stats` |

### 2.9 Convenciones del Archivo

- Todos los exports son `const` con `as const` para inferencia literal de tipos.
- Sin clases, sin instancias, sin factory functions.
- Sin imports externos — es puramente data.
- Los strings de contenido pueden contener HTML inline (ej: `<code>`, `<strong>`).

---

## 3. Shape de Datos Editoriales

### Blog Post
```ts
{
  slug: string;           // ej: "introducing-vantadb"
  title: string;          // NO traducible (brand decision)
  excerpt: string;        // SÍ traducible
  date: string;           // "2026-03-15"
  author: string;
  readTime: string;       // "5 min"
  tag: string;            // SÍ traducible
  tagColor: string;       // hex
  content: {              // SÍ traducible
    type: "p" | "h2";
    text: string;
  }[];
}
```

### Case Study
```ts
{
  slug: string;
  company: string;
  industry: string;
  title: string;          // NO traducible
  summary: string;        // SÍ traducible
  metrics: { value: string; label: string }[];
  challenge: string;      // SÍ traducible
  solution: string;       // SÍ traducible
  quote: string;          // SÍ traducible
  quoteAuthor: string;
}
```

---

## 4. Flujo de Datos: Data → Componente → Render

```
vanta-data.ts
    ↓ import { BLOG_POSTS } from "@/components/vanta/vanta-data"
    ↓
page.tsx (blog/[slug]/page.tsx)
    ↓ const idx = BLOG_POSTS.findIndex(p => p.slug === slug)
    ↓ const post = BLOG_POSTS[idx]
    ↓
tt(`blogPost.data.${idx}.title`, post.title)  ← traducción con fallback
    ↓
Componente <BlogPostContent> renderiza
```

El helper `tt(key, fallback)` busca en `dictionaries[currentLang]`. Si no encuentra la key, devuelve el `fallback` (el valor en español de `vanta-data.ts`). Esto significa que el español ES el contenido por defecto.

### `tt()` — el helper duplicado

```ts
// Presente en 36 archivos (cada page que necesita i18n)
function tt(key: string, fallback: string): string {
  return t(key) ?? fallback;
}
```

Donde `t()` viene del hook `useLanguage()` que lee `dictionaries[currentLang]`.

### Lookup de Slugs

- `blog/[slug]/page.tsx`: `BLOG_POSTS.findIndex(p => p.slug === slug)`.
- `case-studies/[slug]/page.tsx`: mismo patrón con `CASE_STUDIES`.
- Si el slug no existe: render **inline 404** (NO llama a `notFound()` de Next.js).
- **No hay `generateStaticParams`** — todo es client-side. Los slugs se resuelven en el navegador.

---

## 5. Cómo Agregar un Blog Post

### Paso 1: Crear el slug

Elegí un slug URL-friendly, ej: `"vantadb-v0-2-release"`.

### Paso 2: Agregar entry en `BLOG_POSTS`

En `vanta-data.ts`, entre los posts existentes (≈ línea 833):

```ts
{
  slug: "vantadb-v0-2-release",
  title: "VantaDB v0.2.0: Hybrid Search",
  excerpt: "Ahora con búsqueda híbrida vectorial + BM25...",
  date: "2026-07-20",
  author: "Eros",
  readTime: "4 min",
  tag: "release",
  tagColor: "#6366f1",
  content: [
    { type: "p", text: "Estamos emocionados de anunciar..." },
    { type: "h2", text: "¿Qué es Hybrid Search?" },
    { type: "p", text: "Combina búsqueda semántica y por keywords..." },
  ],
},
```

**IMPORTANTE:** El orden en `BLOG_POSTS` define el índice que se usa como key de traducción. Insertar en el medio recorre todos los índices siguientes. Siempre agregar **al final** del array.

### Paso 3: Agregar keys de traducción

En `dictionaries.ts` (ES y EN), agregar:

```ts
// ES (dict base, coincide con vanta-data.ts)
blogPost: {
  data: {
    // índices existentes...
    4: {
      excerpt: "Ahora con búsqueda híbrida vectorial + BM25",
      tag: "release",
      content: [
        { type: "p", text: "Estamos emocionados de anunciar..." },
        { type: "h2", text: "¿Qué es Hybrid Search?" },
        { type: "p", text: "Combina búsqueda semántica y por keywords..." },
      ],
    },
  },
},

// EN
blogPost: {
  data: {
    4: {
      excerpt: "Now with hybrid vector + BM25 search",
      tag: "release",
      content: [
        { type: "p", text: "We're excited to announce..." },
        { type: "h2", text: "What is Hybrid Search?" },
        { type: "p", text: "Combines semantic and keyword search..." },
      ],
    },
  },
},
```

**Nota:** `title` NO se traduce (es decisión de brand que los títulos se muestren igual en todos los idiomas).

### Paso 4: Verificar

- El slug se resuelve automáticamente vía `findIndex`.
- No hay que tocar el router ni el sitemap (aunque idealmente debería actualizarse — ver sección 9).

---

## 6. Cómo Agregar un Case Study

Mismo patrón que blog posts, pero con `CASE_STUDIES` y estructura de case study.

```ts
// En vanta-data.ts, agregar al final de CASE_STUDIES
{
  slug: "acme-corp",
  company: "Acme Corp",
  industry: "E-commerce",
  title: "100x Faster Product Search",
  summary: "Cómo Acme Corp redujo latencias...",
  metrics: [
    { value: "100x", label: "Speedup" },
    { value: "99.9%", label: "Uptime" },
  ],
  challenge: "El buscador anterior tardaba 2 segundos...",
  solution: "Migraron a VantaDB con índices IVF...",
  quote: "VantaDB transformó nuestra plataforma.",
  quoteAuthor: "Jane Doe, CTO Acme Corp",
},
```

Keys de traducción en `dictionaries.ts` → `caseStudy.data.{idx}.{field}`.

---

## 7. Cómo Agregar un Changelog Entry

```ts
// En vanta-data.ts, CHANGELOG ≈ línea 463
// AGREGAR AL PRINCIPIO del array (orden cronológico descendente)
{
  version: "0.2.0",
  date: "2026-07-20",
  title: "Hybrid Search & Performance Improvements",
  changes: [
    "Hybrid search (vector + BM25)",
    "30% faster indexing",
    "New Python client methods",
  ],
},
```

No hay traducción para changelog (es contenido técnico en inglés).

---

## 8. Cómo Agregar Items al FAQ

```ts
// En vanta-data.ts, FAQ ≈ línea 363
// AGREGAR AL FINAL
{
  q: "¿VantaDB soporta búsqueda híbrida?",
  a: "Sí, desde v0.2.0 combinamos búsqueda vectorial y BM25...",
},
```

Traducciones en `dictionaries.ts` → `faq.data.{idx}.q` y `faq.data.{idx}.a`.

---

## 9. i18n de Contenido — El Patrón `tt()` y el Coupling Frágil

### Cómo funciona

1. `vanta-data.ts` tiene contenido en español (es el idioma base).
2. `dictionaries.ts` tiene sobrescrituras por idioma.
3. En cada page, se usa `tt(key, fallback)`:
   - Busca `dictionaries[lang].blogPost.data[0].title`
   - Si no existe → devuelve `fallback` (el valor de `vanta-data.ts`)
   - Si existe → devuelve el valor traducido

```ts
// Uso típico
const t = useLanguage().t;
const post = BLOG_POSTS[idx];

<h1>{tt(`blogPost.data.${idx}.title`, post.title)}</h1>
<p>{tt(`blogPost.data.${idx}.excerpt`, post.excerpt)}</p>
```

### Coupling Frágil (⚠️ Importante)

El índice numérico del `findIndex` se usa como key de traducción. Esto significa:

- Si cambiás el orden de `BLOG_POSTS` o `CASE_STUDIES`, se rompen TODAS las traducciones de los elementos siguientes.
- Insertar un elemento al medio del array cambia los índices de todos los que le siguen.
- **Regla:** siempre agregar al FINAL del array. Si necesitás reordenar, actualizá todas las keys de traducción manualmente.

### Qué se traduce vs qué no

| Se traduce | NO se traduce |
|-----------|---------------|
| `excerpt`, `tag`, `content[]` (blog) | `title` (brand decision) |
| `summary`, `challenge`, `solution`, `quote` (case study) | `company`, `slug` |
| `q`, `a` (FAQ) | `date`, `author`, `readTime` |
| `label` en metrics | `value` en metrics |

---

## 10. Convenciones y Anti-Patrones

### Hacer

- Agregar contenido siempre al **final** de los arrays (preserva índices de i18n).
- Usar `as const` en nuevos exports (coincide con la convención del archivo).
- Poner HTML inline (`<code>`, `<strong>`) dentro de los strings de contenido cuando sea necesario para resaltar términos técnicos.
- Sincronizar ES y EN en el mismo commit (nunca dejar una language sin traducción).

### No Hacer

- **No** agregar nuevos imports a `vanta-data.ts` (debe ser puramente data, sin dependencias).
- **No** crear archivos de datos separados (todo el contenido estático centralizado en `vanta-data.ts`).
- **No** duplicar slugs entre blog posts, case studies, u otras colecciones (causa colisión silenciosa en el lookup).
- **No** cambiar el orden de los arrays sin actualizar las keys numéricas en `dictionaries.ts`.
- **No** agregar `generateStaticParams` a páginas de contenido (el routing es client-side por diseño).
- **No** hardcodear textos en componentes — siempre pasar por `tt()` con fallback.

### Deuda Técnica Conocida

- `tt()` helper duplicado en 36 archivos (cada page lo redefine). Refactorizar a un import compartido está en el backlog.
- El coupling por índice numérico entre `BLOG_POSTS` y `dictionaries` es frágil. Una refactorización a keys por slug (ej: `blogPost.data.[slug].excerpt`) eliminaría el problema pero requiere cambiar todas las pages.
- No hay `generateStaticParams`, lo que significa que los slugs no generan rutas estáticas. Esto es intencional (app 100% client-side), pero implica que no hay pre-renderizado SEO para blog posts y case studies.
