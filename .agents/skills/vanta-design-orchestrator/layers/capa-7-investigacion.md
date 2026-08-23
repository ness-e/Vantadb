# CAPA 7 — INVESTIGACIÓN Y METODOLOGÍA

---

## 16. `design-research` — Investigación de Usuario

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 10 sub-skills de investigación: personas (Alan Cooper), empathy maps (Dave Gray), journey maps (Jim Kalbach), scripts de entrevista (Steve Portigal), tests de usabilidad, card sorting, diagramas de afinidad, JTBD (Christensen/Ulwick), diary studies, diseño de encuestas y repositorio de research. |
| **¿Para qué es?** | Fundamentar decisiones de diseño en evidencia real de usuario, no en suposiciones. |
| **¿Para qué se usa?** | Crear personas basadas en patrones conductuales. Mapear journeys con emociones, pain points y oportunidades. Diseñar scripts de entrevista con técnica de embudo. Planificar usability tests con 5-8 participantes. Sintetizar datos cualitativos en diagramas de afinidad. Mapear Jobs-to-Be-Done con dimensiones funcional, emocional y social. |
| **¿Cómo se usa?** | Invocando la sub-skill: `user-persona` para personas, `journey-map` para journeys, `interview-script` para entrevistas, `usability-test-plan` para tests, `affinity-diagram` para síntesis, `jobs-to-be-done` para JTBD. |
| **¿Cómo debería usarse?** | Personas se crean desde datos reales (entrevistas, analytics), no desde asunciones. Los insights del research repository se estructuran como statements atómicos con nivel de confianza (High/Medium/Low). |
| **¿Cuándo debería usarse?** | **Pre-Fase 1** — Antes de cualquier diseño. O durante validación post-diseño. |
| **Workflows disponibles** | `/design-research:discover`, `/design-research:interview`, `/design-research:synthesize`, `/design-research:test-plan` |
| **Dependencias** | Skill de proyecto (`.agent/skills/design-research/`). No requiere instalación — es conocimiento de investigación UX con 10 sub-skills. |
| **Requerimientos** | Ninguno. Opcional: herramientas de research (Dovetail, Condens, o simplemente una pizarra Miro/Figma). |

## 17. `prototyping-testing` — Validación de Diseño

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 8 sub-skills de validación: estrategia de prototipado (baja/media/alta fidelidad), tests de usabilidad, evaluación heurística (Nielsen 10), diseño de A/B tests (hipótesis/variantes/métricas/sample size), tests de accesibilidad (VoiceOver/NVDA/TalkBack), click tests, wireframe specs y user flow diagrams. |
| **¿Para qué es?** | Validar decisiones de diseño antes de invertir en implementación completa. |
| **¿Para qué se usa?** | Diseñar A/B tests rigurosos con hipótesis estructuradas ("If we [change], then [outcome] because [rationale]"). Planificar tests de accesibilidad en 4 capas (automated → manual → assistive tech → user testing). Crear user flows con happy path + error paths + exit points. |
| **¿Cómo se usa?** | Seleccionando la fidelidad correcta para la pregunta (paper para IA, clickable para interacción, coded para comportamiento real). Ejecutando evaluaciones heurísticas con severidad 0-4 por issue. |
| **¿Cómo debería usarse?** | Prototipando la asunción más riesgosa primero. Corriendo heuristic evaluation con 3-5 evaluadores independientes. |
| **¿Cuándo debería usarse?** | **Entre Fase 2 y Fase 3** — Después de definir visuals, antes de implementar interacciones complejas. |
| **Workflows disponibles** | `/prototyping-testing:evaluate`, `/prototyping-testing:experiment`, `/prototyping-testing:prototype-plan`, `/prototyping-testing:test-plan` |
| **Dependencias** | Skill de proyecto (`.agent/skills/prototyping-testing/`). No requiere instalación — es conocimiento de validación con 8 sub-skills. |
| **Requerimientos** | Ninguno. Opcional: herramientas de prototipado (Figma, ProtoPie, Framer) y testing (UserTesting, Maze, Lookback). |
