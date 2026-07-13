---
title: Documentación Técnica — Herramientas y Estrategia
type: research
status: draft
date: 2026-07-10
tags: [documentation, mdbook, starlight, vitepress, docusaurus, research]
---

# Documentación Técnica: Investigación de Herramientas

## Contexto

VantaDB tiene un vault Obsidian extenso en `docs/` (34 entradas, wikilinks `[[Link]]`).
Core: Rust con bindings a Python, TypeScript, WASM.
Necesidad: documentación interna (dev) + documentación pública para usuarios.

---

## 1. Herramientas Evaluadas

### Generadores de Site Estático (SSG) para Documentación

Basado en comparativa 2026 ([fuente](https://www.youngju.dev/blog/2026-05-14-static-site-generators-2026-hugo-eleventy-astro-mkdocs-docusaurus-mintlify-starlight-comparison-deep-dive.en)):

| Herramienta | Lenguaje | Tipo | Search | i18n | Pricing | Veredicto para VantaDB |
|---|---|---|---|---|---|---|
| **mdBook** | Rust | Docs | Plugin | Plugin | OSS | ✅ **Ideal para docs internos/dev** |
| **Starlight** (Astro) | JS/TS | Docs | Pagefind built-in | Built-in | OSS | ✅ **Ideal para docs públicos** |
| **Docusaurus** | JS/TS | Docs | Algolia | Built-in | OSS | Pesa, mucha config. No recomendado |
| **VitePress** | JS/TS | Docs | Built-in | Partial | OSS | Solo ecosistema Vue |
| **Mintlify** | SaaS (MDX) | Docs | AI built-in | Built-in | $150-550/mes | No para OSS self-hosted |
| **MkDocs** (Material) | Python | Docs | Built-in | Plugin | OSS | No encaja (core es Rust) |
| **Hugo** | Go | General | External | Built-in | OSS | Muy rápido, pero no es docs-first |
| **Astro** | JS/TS | General | Pagefind | Built-in | OSS | Content-first, versátil |

### Detalle de los principales candidatos

#### mdBook (`cargo install mdbook`)
- Cero nuevas dependencias de runtime (Rust, como VantaDB)
- Lee Markdown directamente — el vault Obsidian actual migra casi 1:1
- Integración con rustdoc (la API reference de Rust ya genera docs)
- Buscador (plugin), temas, output HTML estático
- `mdbook serve` para dev, `mdbook build` para producción
- Desventaja: wikilinks `[[Link]]` de Obsidian no son compatibles — toca convertirlos

#### Starlight (Astro) (`npm create astro -- --template starlight`)
- Documentación pública pulida: search (Pagefind), i18n, sidebar, light/dark mode
- MDX + Astro components — permite widgets interactivos en la doc
- llms.txt support (importante para AI search visibility)
- Momentum fuerte en 2026, adoptado por proyectos nuevos de OSS
- Desventaja: requiere Node.js, build más lento que mdBook

---

## 2. Skills de OpenCode Relacionadas

Skills ya instaladas y relevantes para documentación:

| Skill | Propósito |
|---|---|
| `documentation-and-adrs` | ADRs, API docs, feature docs. Documentar el *why* |
| `doc-kami-parchment` | Documentos estilo editorial cálido (PDF/HTML) |
| `docx` | Documentos Word con tracked changes, comentarios |
| `design-md` | DESIGN.md — dirección visual y tokens |
| `ai-seo` | llms.txt, AI search visibility, structured content |
| `writing-guidelines` | Revisión de estilo y tono en docs/prosa |
| `context-engineering` | Configuración de contexto y reglas para agentes |

No existe skill para migrar vault Obsidian → mdBook/Starlight. Habría que crearla si se requiere.

---

## 3. Proyectos GitHub de Referencia

- **mdBook**: [`rust-lang/mdBook`](https://github.com/rust-lang/mdBook) — el estándar Rust, creado por el equipo del Rust Book
- **Starlight**: [`withastro/starlight`](https://github.com/withastro/starlight) — tema oficial de Astro para docs
- **Pagefind**: [`cloudcannon/pagefind`](https://github.com/cloudcannon/pagefind) — búsqueda offline static
- **Comparative 2026**: Artículo de Youngju Kim comparando 11 SSGs (mayo 2026)

---

## 4. Recomendación

### Estrategia en dos capas:

1. **mdBook** para docs internos/dev — apunta al directorio `docs/` existente.
   - Migración casi directa del vault Obsidian
   - Integración con `cargo doc` (rustdoc)
   - Un comando: `mdbook serve`
   - Trabajo principal: convertir wikilinks `[[Link]]` → markdown links estándar

2. **Starlight** para sitio público (vantadb.dev) — cuando se justifique.
   - Documentación pulida para usuarios
   - AI search, i18n, llms.txt
   - Requiere Node.js como build dependency

### Prioridad (ponytail)

Empezar solo con mdBook. Mínimo esfuerzo, máximo impacto.
Starlight se agrega solo cuando haya necesidad demostrable de un site público separado.

---

## 5. Próximos Pasos

- [ ] Configurar `book.toml` en la raíz del proyecto
- [ ] Convertir wikilinks Obsidian a markdown links estándar
- [ ] Integrar rustdoc (`cargo doc`) con mdBook
- [ ] Evaluar si Starlight es necesario o si mdBook solo cubre todas las necesidades
