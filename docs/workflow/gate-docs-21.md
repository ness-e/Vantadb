# `gate-docs-21.yml` — GATE: Docs — Lint & Frontmatter

## ¿Qué hace?

Quality gate para la documentación del proyecto. Verifica que los archivos Markdown en `docs/` estén bien formateados y tengan frontmatter YAML válido con los campos requeridos.

## ¿Cómo lo hace?

2 jobs independientes:

1. **`lint-markdown`**: ejecuta `npx markdownlint-cli2 "docs/**/*.md"` — lintea todos los MD con reglas configurables de markdownlint
2. **`check-format`**: script bash que itera sobre todos los `*.md` en `docs/`, verifica que tengan frontmatter YAML (delimitado por `---`) y que contengan el campo `title:`

## ¿Qué tests usa?

No usa tests. Usa **markdownlint-cli2** y un script bash propio.

## ¿Qué verifica?

- Formato Markdown correcto (indentación, tablas, listas, etc.)
- Todos los documentos tienen frontmatter YAML
- Todos los frontmatters tienen el campo `title` requerido

## Funcionalidad final

Mantener la documentación del proyecto consistente, bien formateada y con metadatos completos para generación de índices y navegación.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en `docs/**`
- **Pull Request** a `main` con cambios en `docs/**`
- **Workflow dispatch** manual
