# SUMMARY — Índice de Documentos Estratégicos

> Referencia rápida de los 15 documentos de estrategia + reportes del Vanta Design Orchestrator.

---

## Documentos de Estrategia (`strategy/`)

| # | Documento | Propósito | Trigger Words |
|:-:|:----------|:----------|:--------------|
| 1 | `business-model-design.md` | BMC, Value Proposition Canvas, JTBD, diagnóstico estratégico | modelo de negocio, BMC, propuesta de valor, JTBD |
| 2 | `brand-platform.md` | Propósito, visión, misión, territorio, arquetipos, personalidad | plataforma de marca, territorio, arquetipos, brand platform |
| 3 | `decision-hierarchy.md` | Jerarquía Business > Brand > Marketing > Design + conflicto cross-level | jerarquía de decisiones, deseabilidad, factibilidad, viabilidad |
| 4 | `sensory-identity.md` | Psicología del color + specs técnicas OKLCH/HEX/CMYK/Pantone | psicología del color, identidad sensorial, OKLCH |
| 5 | `verbal-identity.md` | Voz, tono, jerarquía de contenido, naming, UX writing | voz de marca, tono, naming, UX writing, microcopy |
| 6 | `metrics-framework.md` | Brand equity, CAC, ROAS, CASH (Burn/Runway/MRR/ARR/NRR), HEART, OKRs | métricas, KPIs, ROAS, burn rate, HEART, OKRs |
| 7 | `lean-design.md` | Lean Canvas, MVP scope, hipótesis, validación | lean, MVP, hipótesis, validación, pivot |
| 8 | `legal-protection.md` | Naming clearance, clases Niza, contratos, IP assignment, GDPR | legal, naming clearance, clases Niza, IP, GDPR |
| 9 | `trends-2026.md` | Tendencias 2026: AI, motion, sostenibilidad, privacy-first, sonic | tendencias 2026, AI design, motion 2026 |
| 10 | `sonic-kinetic-identity.md` | Arquetipos sónicos, paleta de sonidos, timing, voiceline specs | identidad sonora, sonic branding, arquetipo sónico |
| 11 | `brand-operations.md` | RACI, co-branding rules, change management, gov. post-entrega | RACI, co-branding, governance, change management |
| 12 | `brand-documentation.md` | MIC, Brand Book, Brand Kit, Decision Log, Manual Digital | brand book, MIC, brand kit, decision log, manual de marca |
| 13 | `validation-sustainability.md` | Validación empírica, 5 porqués, triple balance, escalabilidad, productización | validación, 5 porqués, evidencia, sostenibilidad, producto diseño |
| 14 | `content-strategy.md` | Auditoría de contenido, modelo, UX writing system, governance, RACI | content strategy, auditoría contenido, UX writing, microcopy, copy |
| 15 | `accessibility-strategy.md` | WCAG 2.2 AA, checklist visual/componentes, herramientas, RACI | accesibilidad, a11y, WCAG, contraste, ARIA, inclusivo |

---

## Reportes de Auditoría (`reports/`)

| # | Documento | Propósito | Fecha |
|:-:|:----------|:----------|:------|
| 1 | `audit-vantadb-github-io.md` | Auditoría completa de vantadb.github.io — 8 dimensiones, severidad, plan de acción | 2026-06-19 |

---

## Infraestructura (`infrastructure/`)

| Documento | Propósito |
|:----------|:----------|
| `tool-registry.md` | Registro de herramientas externas y sistemas de diseño |
| `analytics-setup.md` | Configuración de GA4, Plausible, event plans, funnels |
| `developer-handoff.md` | Formato de spec, redlines, tokens JSON/CSS, assets |
| `design-critique-templates.md` | 7 plantillas para crítica de diseño estructurada |

---

## Flujo Recomendado por Tipo de Proyecto

| Proyecto | Ruta Estratégica |
|:---------|:-----------------|
| **Accesibilidad / a11y first** | 15 → 4 → 6 → 8 → tools §10 |
| **Content strategy setup** | 5 → 14 → 9 → writing-guidelines |
| **Brand desde cero** | 1 → 2 → 3 → 4 → 5 → 8 → 10 → 12 |
| **Startup / MVP** | 1 → 7 → 3 → 6 → 8 |
| **Rediseño** | 3 → 1 (diagnóstico) → 2 → 6 → 13 |
| **Documentación de marca** | 2 → 4 → 5 → 10 → 11 → 12 |
| **Validación de concepto** | 13 → 7 → 6 |
| **Auditoría de sitio web** | reportes/audit → impeccable → visual-critique → motion → a11y → ops |
| **Full product launch** | 1 → 2 → 3 → 7 → 4 → 5 → 10 → 6 → 8 → 12 → 13 |

---

## Convención de Nombres

| Archivo | Prefijo | Ejemplo |
|:--------|:--------|:--------|
| Estrategia de negocio | `strategy/` | `strategy/business-model-design.md` |
| Infraestructura | `infrastructure/` | `infrastructure/tool-registry.md` |
| Reportes de auditoría | `reports/` | `reports/audit-vantadb-github-io.md` |
| Analytics | `infrastructure/` | `infrastructure/analytics-setup.md` |
| Developer handoff | `infrastructure/` | `infrastructure/developer-handoff.md` |
| Configuraciones | `configs/` | `configs/project-presets.json` |

---

## Pendientes / Historial

| Fecha | Cambio |
|:------|:-------|
| 2026-06-19 | Creación inicial del índice |
| 2026-06-19 | Añadido reporte `audit-vantadb-github-io.md`, nuevo flujo de auditoría de sitio web |

