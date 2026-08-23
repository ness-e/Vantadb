# CAPA 2 — ESTRUCTURA Y USABILIDAD

---

## 3. `ux-heuristics` — Principios de Usabilidad

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Marco cognitivo basado en las 10 heurísticas de Jakob Nielsen y las leyes de Steve Krug (_Don't Make Me Think_). Incluye checklist evaluable de 0 a 10 y el framework de severidad (cosmético/menor/mayor/catástrofe). |
| **¿Para qué es?** | Reducir la carga cognitiva del usuario y hacer la navegación autodescriptiva. |
| **¿Para qué se usa?** | Evaluar flujos de usuario contra las 10 heurísticas: visibilidad del estado, correspondencia con el mundo real, control y libertad, consistencia, prevención de errores, reconocimiento sobre recuerdo, flexibilidad, diseño estético minimalista, recuperación de errores y ayuda. Incluye el **Trunk Test** de Krug (identidad del sitio, sección actual, opciones de navegación y búsqueda evidentes al instante). |
| **¿Cómo se usa?** | Evaluando cada heurística de 0 a 4 en severidad. Ejecutando el Trunk Test en cada página. Aplicando la regla de "eliminar la mitad de las palabras, y luego eliminar la mitad de lo que queda". |
| **¿Cómo debería usarse?** | Como filtro obligatorio antes de pasar a la fase visual. Si una página no pasa el Trunk Test, no se estiliza — se reestructura. |
| **¿Cuándo debería usarse?** | **Fase 1** — Al estructurar wireframes, menús de navegación, textos y flujos conversacionales. |
| **Dependencias** | Skill de proyecto (`.agent/skills/ux-heuristics/`). No requiere instalación — es marco cognitivo basado en las 10 heurísticas de Nielsen y el Trunk Test de Krug. |
| **Requerimientos** | Ninguno. Solo acceso al diseño o prototipo a evaluar. |

## 4. `frontend-design` — Estructuración Limpia de Componentes

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Pautas de calidad frontend destinadas a producir código HTML5 semántico y CSS limpio, con composiciones modulares y asimétricas que eviten la monotonía típica de IA. |
| **¿Para qué es?** | Evitar maquetas genéricas. Desarrollar estructuras que desafíen la rigidez geométrica (layouts asimétricos equilibrados, uso intencional de whitespace, composiciones de peso visual desbalanceado con intención). |
| **¿Para qué se usa?** | Configurar grids CSS, flexbox avanzado, evitar el anidamiento excesivo de contenedores (divitis), asegurar que la estructura HTML refleje la jerarquía semántica del contenido. |
| **¿Cómo se usa?** | Evaluando la estructura propuesta contra su checklist interno: ¿es semántica? ¿evita nesting innecesario? ¿el layout tiene tensión visual o es plano? ¿los espacios negativos son intencionales? |
| **¿Cómo debería usarse?** | Diseñando layouts con bento-grid, composiciones de 60/40 o 70/30, hero sections con whitespace dramático, y evitando la cuadrícula perfecta de 3 columnas iguales. |
| **¿Cuándo debería usarse?** | **Fase 1** — Durante la escritura inicial de la estructura HTML y estilos base de cualquier componente. |
| **Dependencias** | Skill de proyecto (`.agent/skills/frontend-design/`). No requiere instalación — son pautas de calidad frontend. |
| **Requerimientos** | Ninguno. Aplica a cualquier proyecto HTML/CSS/JS. |

## 5. `ux-strategy` — Estrategia y Arquitectura de Producto

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 10 sub-skills estratégicas: análisis competitivo, principios de diseño, brief de diseño, arquitectura de información, estrategia de contenido, mapeo de experiencia, definición de métricas (HEART framework), visión north-star, framework de oportunidades (RICE, Kano, Impact-Effort), service blueprints y alineación de stakeholders. |
| **¿Para qué es?** | Dar dirección estratégica al producto antes de diseñar píxeles. |
| **¿Para qué se usa?** | Definir la estructura de información del producto (sitemap, taxonomía, modelo de contenido), evaluar competidores, establecer principios de diseño que resuelvan debates, definir métricas de éxito UX, y crear service blueprints que mapeen todo el sistema de entrega. |
| **¿Cómo se usa?** | Invocando la sub-skill relevante: `information-architecture` para IA, `competitive-analysis` para benchmarks, `design-principles` para principios, `metrics-definition` para KPIs, `service-blueprint` para mapeo sistémico. |
| **¿Cómo debería usarse?** | Como fase de discovery antes de la implementación. Un análisis competitivo identifica oportunidades; los principios de diseño resuelven debates futuros. |
| **¿Cuándo debería usarse?** | **Pre-Fase 1** — Antes de iniciar cualquier trabajo de diseño significativo. |
| **Workflows disponibles** | `/ux-strategy:benchmark`, `/ux-strategy:frame-problem`, `/ux-strategy:strategize` |
| **Dependencias** | Skill de proyecto (`.agent/skills/ux-strategy/`). No requiere instalación — es conocimiento estratégico de producto. |
| **Requerimientos** | Ninguno. Solo acceso a información del negocio/competencia. |
