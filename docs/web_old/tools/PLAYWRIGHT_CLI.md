---
title: "Playwright CLI — Visual Review Tool"
status: active
tags: [vantadb, web, tools]
last_reviewed: 2026-07-03
aliases: []
---

Integrado al flujo de diseño VantaDB para verificar
visualmente que los cambios se vean como se planeó.

## Setup

- CLI global: `playwright-cli` (v0.1.14, paquete `@playwright/cli`)
- Skills instaladas en `.claude/skills/playwright-cli/`
- Config: `.playwright/cli.config.json`
- Browser predeterminado: Chromium (detectó Chrome instalado)

## Comandos principales

```bash
# Abrir el sitio en desarrollo
playwright-cli open http://localhost:3000

# Tomar screenshot full-page
playwright-cli screenshot --filename=verificacion.png

# Snapshot para inspeccionar elementos
playwright-cli snapshot

# Abrir con viewport mobile (390x844)
playwright-cli open http://localhost:3000/pricing --headed
playwright-cli resize 390 844
playwright-cli screenshot --filename=pricing-mobile.png

# Abrir dashboard visual
playwright-cli show
```

## Workflow de revisión visual

1. `playwright-cli open http://localhost:3000` — navega al sitio
2. `playwright-cli screenshot --filename=hero-desktop.png` — captura hero
3. `playwright-cli snapshot` — lista elementos interactivos con refs
4. `playwright-cli click e<N>` — navega a otra página
5. `playwright-cli screenshot --filename=pagina-desktop.png`
6. `playwright-cli resize 390 844` — cambia a mobile
7. `playwright-cli screenshot --filename=pagina-mobile.png`
8. `playwright-cli close` — cierra sesión

## Script automatizado

```bash
# Todas las páginas clave en desktop + mobile
for page in \
  "/" "/pricing" "/about" "/changelog" "/engine" \
  "/architecture" "/docs" "/blog"; do
  playwright-cli goto "http://localhost:3000$page"
  playwright-cli screenshot --filename="$(echo $page | tr '/' '_')_desktop.png"
  playwright-cli resize 390 844
  playwright-cli screenshot --filename="$(echo $page | tr '/' '_')_mobile.png"
  playwright-cli resize 1440 900
done
```
