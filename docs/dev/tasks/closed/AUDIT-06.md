---
title: "AUDIT-06: Gobernabilidad .opencode, Posicionamiento STRAT-01 y Rieles Negocio VE"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-lead"
category: "governance-strategy"
tags: [governance, opencode, strat-01, readme, business, venezuela]
description: "Limpieza de .opencode/, sincronización de posicionamiento STRAT-01 ('SQLite for agents') en README y docs, y actualización de Backlog-negocio.md para rieles de pago desde Venezuela."
---

# AUDIT-06: Gobernabilidad .opencode, Posicionamiento STRAT-01 y Rieles Negocio VE

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Auditoría v0.9.0 / Directiva STRAT-01 y Estrategia Negocio VE
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴 Alta
- **Tipo:** Documentación, Configuración y Estrategia
- **Turns estimados:** 10-15
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 4

## Objetivo

Restaurar la higiene estricta del entorno de gobernabilidad `.opencode/`, alinear el posicionamiento público del producto conforme a la decisión `STRAT-01` y reconfigurar la estrategia de recaudación comercial en `Backlog-negocio.md` para operaciones radicadas en Venezuela (`VE`).
Específicamente:
1. Eliminar fallos de herramientas estáticas en `develop` causados por rutas a `desktop/` (VantaDB Studio), aclarando formalmente que `desktop/` reside en el worktree independiente `develop-desktop`.
2. Sanitizar `README.md` y `README_ES.md`: suprimir siglas internas crípticas (`ADR-0047`, `ADR-0030`, `FIND-105`, `BENCH-01`), reformular la sección "North Star metric" en términos de valor para el usuario y reparar enlaces que daban 404 por apuntar a rutas bajo `.gitignore`.
3. Sincronizar la documentación (`docs/index.md`, `docs/user/`, `docs/api/`, `docs/dev/`) con la identidad única de producto: **"The local-first cognitive memory engine for AI agents ('SQLite for agents')"**.
4. Actualizar `Backlog-negocio.md`: registrar la resolución de `STRAT-01` (Opción A), posponer explícitamente `BIZ-18` por decisión del propietario, y establecer la estrategia de rieles de pago para Venezuela priorizando Creem.io (USDC en Polygon), Binance Pay / USDT directo (`BIZ-10`) y Payoneer → Airtm (`BIZ-12`), descartando Polar.sh debido a bloqueos de Stripe Connect Express en VE hasta la constitución de una entidad legal extranjera (`BIZ-22`).

## Archivos en scope

- `.opencode/configs/sgconfig.yml`, `.opencode/configs/semgrep-vanta.yml`
- `.opencode/AGENTS.md`, `.opencode/SKILLS-MANIFEST.md`
- `.opencode/commands/{harness,cleanCA,research}.md`
- `.opencode/agents/{vanta-audit,vanta-worker,vanta-lead}.md`
- `.opencode/rules/{frontend-web,mcp-integration}.md`
- `README.md`, `README_ES.md`
- `docs/index.md`, `docs/user/guide/getting-started.md`, `docs/user/concepts/architecture.md`, `docs/api/mcp/index.md`, `docs/dev/OPERATING-MODEL.md`
- `docs/dev/Backlog.md`, `docs/dev/Backlog-negocio.md`

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | Desarrolladores externos, agentes OpenCode/Antigravity, evaluadores de producto |
| Callees | Configuración de linters, gates de documentación, pipeline de cobranza |
| Implicaciones | Los gates de linting pasan en `develop` sin requerir clonar `desktop/`; la propuesta de valor comercial y técnica es nítida; los rieles de pago son viables y libres de bloqueos de cumplimiento internacional. |

## Criterios de aceptación

1. `ast-grep` y `semgrep` ejecutan sin advertencias ni fallos por rutas no encontradas a `desktop/` en `develop`.
2. `SKILLS-MANIFEST.md` incluye la ficha canónica de `documentation-skill`.
3. `README.md` y `README_ES.md` no contienen siglas de auditoría interna ni enlaces rotos a `docs/dev/`.
4. El posicionamiento "SQLite for agents" se refleja consistentemente en el README y la documentación técnica.
5. `Backlog-negocio.md` refleja la resolución de `STRAT-01`, la postergación de `BIZ-18` y la ruta de pagos adaptada a Venezuela.

## Contrato

```bash
node scripts/docs/check-links.mjs
node scripts/docs/check-docs.mjs
git diff README.md README_ES.md (verificar ausencia de ADR-0047, FIND-105, BENCH-01)
```

## Steps

### Step 1: Higiene y desacople de `.opencode/`
- Ajustar `sgconfig.yml` y `semgrep-vanta.yml` para documentar que `desktop/` reside en `develop-desktop`.
- Actualizar `AGENTS.md`, comandos y agentes correspondientes.
- Añadir nota en `frontend-web.md` y crear/corregir `mcp-integration.md`.
- Registrar `documentation-skill` en `SKILLS-MANIFEST.md`.

### Step 2: Limpieza de README.md y README_ES.md
- Purgar menciones de siglas internas y reescribir bloque de métrica.
- Reparar enlaces a documentos públicos o eliminar enlaces hacia `docs/dev/` ignorados en git.
- Reforzar el tagline de memoria cognitiva local-first para agentes.

### Step 3: Alineación de documentación técnica
- Alinear `docs/user/guide/getting-started.md`, `docs/user/concepts/architecture.md`, `docs/api/mcp/index.md` y `docs/dev/OPERATING-MODEL.md` con `STRAT-01`, `McpProfile::Brain` y `forget()`.

### Step 4: Actualización de Backlogs
- Registrar tareas `AUDIT-01` a `AUDIT-06` en `docs/dev/Backlog.md`.
- Actualizar `docs/dev/Backlog-negocio.md` con las decisiones de `STRAT-01`, `BIZ-18` y rieles de pago para `VE`.

## Invariantes de dominio

- La documentación pública no debe exponer artefactos internos ni procesos de desarrollo privados.
- Los rieles de pago registrados deben cumplir con la legalidad vigente y minimizar riesgos cambiarios para el operador.

