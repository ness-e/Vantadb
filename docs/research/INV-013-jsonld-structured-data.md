# INV-013: JSON-LD Structured Data — Auditoría

> **Estado:** ✅ COMPLETADA 2026-08-03 · **Fuente:** docs/Backlog.md INV-013 · **Tipo:** Web Frontend (SEO) — auditoría + propuesta, sin implementación

## Resumen Ejecutivo

**Hallazgo: JSON-LD AUSENTE en el sitio web.** No existe ningún `<script type="application/ld+json">` ni campo `jsonLd`. Consecuencia: sin rich snippets / rich results en Google.

## Evidencia

| Archivo | Estado |
|---|---|
| `web/src/app/layout.tsx` | Metadata rico exportado: `title`, `description`, `keywords` (12 keys), `authors` (ness-e), `icons`, `manifest`, `openGraph`, `twitter`, `viewport`. **Cero** JSON-LD. |
| `web/src/app/page.tsx` | 13 líneas, `"use client"`. No exporta metadata ni structured data. Solo renderiza `HomeView`. |

## Veredicto sobre Next.js Metadata API

**Next.js 16 Metadata API NO genera JSON-LD.** Validado contra docs oficiales (nextjs.org/docs/app/building-your-application/optimizing/metadata, v16.3.0):

- La API solo emite tags `<head>` automáticos: title, description, OG, Twitter, charset, viewport.
- No existe campo `jsonLd` en el tipo `Metadata`.
- El JSON-LD debe marcarse manualmente con `<script type="application/ld+json">` dentro del JSX de un Server Component (o `jsonLd` en `generateMetadata` no existe — solo render directo).

## Propuesta: schema.org/SoftwareApplication

```json
{
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  "name": "VantaDB",
  "description": "Local-first, embedded Rust database engine for AI agents and local RAG. Persistent memory, crash-safe WAL (CRC32C), and native hybrid search (BM25 + HNSW via RRF) — zero network, in-process, 1.2ms latency.",
  "applicationCategory": "DatabaseApplication",
  "applicationSubCategory": "Vector Database",
  "operatingSystem": "Windows, macOS, Linux",
  "softwareVersion": "0.2.0",
  "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
  "author": { "@type": "Person", "name": "ness-e" },
  "keywords": ["vector database", "Rust", "embedded database", "local-first", "HNSW", "BM25", "RRF", "hybrid search", "RAG"]
}
```

## Validación recomendada

- Google Rich Results Test — https://search.google.com/test/rich-results
- validator.schema.org — https://validator.schema.org

## Siguiente paso (si se implementa)

1. Convertir `layout.tsx` a Server Component (hoy es client) o inyectar el script vía `metadata` + un Server Component `Head`.
2. Emitir el JSON-LD en el `<head>` raíz (aplica a todas las páginas del sitio).
3. Validar con Rich Results Test.

## Notas

- Solo auditoría + propuesta (alcance del backlog). Cero cambios de código.
- El JSON-LD propuesto es una sola wallet (un solo bloque) para el sitio completo.
- **Corrección de trazabilidad (2026-08-04):** `docs/progreso/README.md:1323` etiqueta esta investigación como "superseded por WEB-13". Esa nota es **FALSA** — WEB-13 era sobre `web/src/routes/` (páginas Next.js Pages Router) que ya no existe tras la migración a App Router; el JSON-LD NUNCA se implementó. Se corrige el registro de progreso; la propuesta de arriba sigue siendo el plan de acción vigente (ver tarea en Backlog).
