---
title: "`gate-docs.yml` — GATE: Docs — Lint & Frontmatter"
kind: runbook
status: active
description: Quality gate para la documentación del proyecto. Verifica que los archivos Markdown en docs/ estén bien formateados y tengan frontmatter YAML válido con los campos requeridos
tags: [vantadb, ci, gate-docs]
---

# `gate-docs.yml` — GATE: Docs — Lint & Frontmatter

## ¿Qué hace?

Quality gate para la documentación del proyecto. Verifica que los archivos Markdown en `docs/` estén bien formateados y tengan frontmatter YAML válido con los campos requeridos.

## ¿Cómo lo hace?

5 jobs independientes:

1. **`lint-markdown`**: ejecuta `npx markdownlint-cli2 "docs/**/*.md"` — lintea todos los MD con reglas configurables de markdownlint
2. **`check-format`**: script bash que itera sobre todos los `*.md` en `docs/`, verifica que tengan frontmatter YAML (delimitado por `---`) y que contengan el campo `title:`
3. **`check-api-version`**: valida `docs/api/openapi.yaml` + `docs/api/MCP.md` contra `[workspace.package] version` (incluye paridad OpenAPI/router)
4. **`check-npm-versions`**: valida que `vantadb-ts/package.json` (estricto) y `vantadb-node/package.json` (excepción never-published) sigan la versión del workspace (FIND-230)
5. **`check-frontier`**: valida el product frontier (docs vs features/routes)

## ¿Qué tests usa?

No usa tests. Usa **markdownlint-cli2** y scripts propios (bash/Node).

## ¿Qué verifica?

- Formato Markdown correcto (indentación, tablas, listas, etc.)
- Todos los documentos tienen frontmatter YAML
- Todos los frontmatters tienen el campo `title` requerido
- Las versiones npm (`vantadb-ts` y `vantadb-node`) siguen la versión del workspace
- El product frontier es consistente con los routes/features

## Funcionalidad final

Mantener la documentación del proyecto consistente, bien formateada y con metadatos completos para generación de índices y navegación.

## ¿Cuándo se ejecuta?

- **Push** a `main`/`develop` con cambios en `docs/**`, `scripts/**`, `Cargo.toml`, `src/server/router.rs`, `src/server/routing.rs` o los `package.json` de `vantadb-ts`/`vantadb-node`
- **Pull Request** a `main`/`develop` con los mismos paths
- **Workflow dispatch** manual
