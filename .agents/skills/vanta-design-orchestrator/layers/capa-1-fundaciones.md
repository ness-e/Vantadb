# CAPA 1 — FUNDACIONES Y TOKENS

---

## 1. `ui-ux-pro-max` — Motor de Estilos y Tokens

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Base de datos determinista con 50 estilos de diseño, 21 paletas de color, 50 parejas tipográficas, 20 tipos de gráficos y 9 stacks tecnológicos. Incluye scripts Python para búsqueda programática. |
| **¿Para qué es?** | Sentar las bases del sistema de diseño (tokens de color, tipografía, espaciado) de una aplicación o sección completa. |
| **¿Para qué se usa?** | Inicializar el look-and-feel global, emparejar tipografías adecuadas al nicho del proyecto y generar paletas HSL coherentes. |
| **¿Cómo se usa?** | `python skills/ui-ux-pro-max/scripts/search.py "<query>" --design-system`. Acepta queries como "cinematic dark database" o "minimal editorial SaaS". |
| **¿Cómo debería usarse?** | Con el flag `--persist` para generar automáticamente el archivo maestro `design-system/MASTER.md` que centraliza todos los tokens. |
| **¿Cuándo debería usarse?** | **Fase 1** — Al inicio de la conceptualización o rediseño estético general. Es la PRIMERA skill que se consulta en un nuevo proyecto. |
| **Dependencias** | Skill de proyecto (`.agent/skills/ui-ux-pro-max/`). Requiere Python 3.8+ para los scripts de búsqueda. Los scripts están en `skills/ui-ux-pro-max/scripts/search.py`. |
| **Requerimientos** | Python 3.8+. Opcional: `rich` (para output coloreado en terminal). |

## 2. `design-systems` — Arquitectura de Sistemas de Diseño

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 10 sub-skills que cubren: tokens de diseño, especificación de componentes, auditoría de accesibilidad (WCAG 2.2), sistema de temas (dark/light/high-contrast), sistema de movimiento (duración + easing tokens), convenciones de nombres, biblioteca de patrones, sistema de iconos, documentación y localización RTL/i18n. |
| **¿Para qué es?** | Construir, documentar y mantener un design system escalable desde sus fundaciones hasta su gobernanza. |
| **¿Para qué se usa?** | Definir tokens (`color-action-primary`, `spacing-md`), especificar componentes con estados completos (default/hover/focus/active/disabled/loading/error), crear sistemas de temas con override por capas, y establecer reglas de contribución y versionado semántico. |
| **¿Cómo se usa?** | Consultando la sub-skill relevante según la necesidad: `design-token` para tokens, `component-spec` para specs, `accessibility-audit` para WCAG, `theming-system` para temas, `motion-system` para animaciones, `naming-convention` para nombres, `icon-system` para iconos, `localization-design` para RTL/i18n. |
| **¿Cómo debería usarse?** | Definiendo primero tokens globales → luego alias semánticos → luego tokens de componente. Nunca referenciar valores raw en componentes. Usar CSS custom properties para temas runtime. |
| **¿Cuándo debería usarse?** | **Fase 1-2** — Después de definir el estilo global con `ui-ux-pro-max`, para formalizar y estructurar los tokens en un sistema versionable. |
| **Workflows disponibles** | `/design-systems:audit-system`, `/design-systems:create-component`, `/design-systems:tokenize` |
| **Dependencias** | Skill de proyecto (`.agent/skills/design-systems/`). No requiere instalación adicional — es conocimiento de arquitectura de design systems. |
| **Requerimientos** | Ninguno. Funciona sobre cualquier stack. Opcional: repo de tokens (CSS custom properties, JSON, o Figma). |
