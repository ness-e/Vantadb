# CAPA 8 — OPERACIONES Y HERRAMIENTAS DE EQUIPO

---

## 18. `design-ops` — Operaciones de Diseño

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 7 sub-skills operacionales: frameworks de crítica, auditoría de deuda de diseño, reporting de impacto, checklists de QA, proceso de revisión (4 gates), planificación de sprints de diseño (5 días), handoff specs (visual/interaction/content/assets/edge-cases), team workflow y estrategia de version control. |
| **¿Para qué es?** | Operacionalizar el diseño: desde cómo se critica hasta cómo se entrega a desarrollo y cómo se mide el impacto. |
| **¿Para qué se usa?** | Crear handoff specs completos (spacing exacto con tokens, estados de interacción, comportamiento responsive, edge cases, ARIA roles). Planificar design sprints de 5 días. Auditar deuda de diseño (visual, estructural, accesibilidad, documentación). Medir impacto con before/after y A/B test summaries. |
| **¿Cómo se usa?** | Invocando la sub-skill: `handoff-spec` para entrega a dev, `design-sprint-plan` para sprints, `design-debt-audit` para deuda, `design-impact-reporting` para métricas, `design-qa-checklist` para QA, `design-critique` para críticas estructuradas. |
| **¿Cómo debería usarse?** | Los handoffs siempre usan tokens (nunca hex/px raw). La deuda de diseño se cuantifica con: Severidad × Frecuencia / Esfuerzo. El QA checklist cubre: visual accuracy, layout, interaction, content, accessibility, cross-platform. |
| **¿Cuándo debería usarse?** | **Fase 4 y Post-producción** — Para entrega, QA y mejora continua. |
| **Workflows disponibles** | `/design-ops:handoff`, `/design-ops:plan-sprint`, `/design-ops:setup-workflow` |
| **Dependencias** | Skill de proyecto (`.agent/skills/design-ops/`). No requiere instalación — es conocimiento operacional con 7 sub-skills. |
| **Requerimientos** | Ninguno. Opcional: herramientas de project management (Linear, Jira, Notion) y handoff (Figma Dev Mode, Zeplin). |

## 19. `designer-toolkit` — Utilidades Esenciales del Diseñador

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 7 sub-skills utilitarias: case studies de portfolio, negociación de diseño (advocacy basada en evidencia), design rationale (conexión decisión→evidencia→trade-offs), adopción de design system, auditoría de tokens, presentaciones estructuradas y UX writing (microcopy, error messages, empty states, CTAs, onboarding copy). |
| **¿Para qué es?** | Dar al diseñador herramientas de comunicación y advocacy para defender decisiones de diseño con evidencia, documentar rationales y producir entregables profesionales. |
| **¿Para qué se usa?** | Escribir rationales que conecten decisiones a user needs ("Users are confused at step 3" > "this layout is unclear"). Diseñar microcopy efectivo (CTAs con verbo + outcome específico). Crear case studies con estructura Challenge → Process → Solution → Impact. Negociar scope con trade-offs explícitos. |
| **¿Cómo se usa?** | Invocando: `design-rationale` para justificaciones, `ux-writing` para copy, `case-study` para portfolio, `design-negotiation` para advocacy, `presentation-deck` para presentaciones. |
| **¿Cómo debería usarse?** | Toda decisión de diseño controversial debe acompañarse de un rationale escrito. Los error messages siguen formato: What happened → Why → What to do. Los CTAs empiezan con verbo y son específicos sobre el outcome. |
| **¿Cuándo debería usarse?** | **Cualquier fase** — Según la necesidad de documentación, comunicación o advocacy. |
| **Workflows disponibles** | `/designer-toolkit:build-presentation`, `/designer-toolkit:write-case-study`, `/designer-toolkit:write-rationale` |
| **Dependencias** | Skill de proyecto (`.agent/skills/designer-toolkit/`). No requiere instalación — es conocimiento utilitario con 7 sub-skills. |
| **Requerimientos** | Ninguno. Opcional: herramientas de presentación (slides.com, Google Slides, Keynote). |

## 20. `vanta-design-orchestrator` — Meta-Skill

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | El orquestador maestro que define el rol del agente, documenta las skills y establece el protocolo de uso combinado. |
| **¿Para qué es?** | Ser el punto de entrada único para cualquier tarea de diseño. |
| **¿Cuándo se lee?** | Automáticamente cuando se activa el rol de Lead Design Engineer (trigger words), o cuando se carga una capa condicional. |
| **Dependencias** | Este archivo. No requiere instalación — es el meta-skill orquestador. |
| **Requerimientos** | Los skills que orquesta deben estar instalados. Ver `Dependencias` de cada skill individual. |
