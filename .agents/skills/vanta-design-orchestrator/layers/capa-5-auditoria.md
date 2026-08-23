# CAPA 5 — AUDITORÍA Y REFINAMIENTO

---

## 11. `impeccable` — Auditoría Visual e Iteración en Caliente

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Motor CLI con 23 comandos de refinamiento UX y 41 reglas contra la monotonía visual. Incluye el "slop test" (41 señales de código genérico de IA) y herramientas de auditoría en navegador en tiempo real. |
| **¿Para qué es?** | Auditar interfaces construidas, corregir contrastes, iterar en caliente sobre componentes y detectar "AI fingerprints" (patrones visuales que delatan generación automática). |
| **¿Para qué se usa?** | Refinar e implantar microdetalles antes del deploy: edge cases, estados de error, overflows de texto, delight moments, contrastes insuficientes, espaciado inconsistente. |
| **¿Cómo se usa?** | Comandos principales: `/impeccable craft <target>` (construir), `/impeccable shape <target>` (dar forma), `/impeccable audit <target>` (auditar), `/impeccable polish <target>` (pulir). Cada comando activa un conjunto específico de reglas. |
| **¿Cómo debería usarse?** | Ejecutando `audit` sobre cada sección construida → corrigiendo hallazgos → ejecutando `polish` para el refinamiento final. El slop-test es obligatorio antes de producción. |
| **¿Cuándo debería usarse?** | **Fase 4** — En la fase de maquetación media y finalización de componentes. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add pbakaus/impeccable -g`). El CLI `impeccable` se instala globalmente. Requiere Node 18+. |
| **Requerimientos** | Node 18+. El CLI corre desde la terminal. Para auditoría en navegador: Chrome/Edge. |

## 12. `web-design-guidelines` — Compliance de Interfaz Web

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill que descarga y aplica las Writing/Web Interface Guidelines de Vercel en tiempo real. Verifica código UI contra un conjunto de reglas de accesibilidad, performance y mejores prácticas. |
| **¿Para qué es?** | Asegurar que la interfaz web cumple con estándares de industria antes del deployment. |
| **¿Para qué se usa?** | Revisar UI code por: accesibilidad (ARIA, contraste, keyboard nav), rendimiento (LCP, CLS, FID), SEO (meta tags, heading hierarchy), y responsive behavior. |
| **¿Cómo se usa?** | Proporcionando archivos o patrones para revisión. El skill descarga las guidelines actualizadas desde el repo oficial de Vercel y aplica todas las reglas, reportando en formato `file:line`. |
| **¿Cómo debería usarse?** | Como gate de calidad final. Si hay violaciones críticas (accesibilidad, contraste), no se despliega. |
| **¿Cuándo debería usarse?** | **Fase 4** — Post-implementación, antes del deploy a producción. |
| **Dependencias** | Skill de proyecto (`.agent/skills/web-design-guidelines/`). Descarga guidelines de Vercel en tiempo real desde su repo oficial. Requiere conexión a internet. |
| **Requerimientos** | Conexión a internet para descargar guidelines. Los archivos a revisar deben ser accesibles localmente. |

## 13. `writing-guidelines` — Compliance de Prosa y Documentación

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill que descarga y aplica las Writing Guidelines de Vercel para revisar documentación y prosa del proyecto. |
| **¿Para qué es?** | Asegurar que la documentación, microcopy y contenido textual del producto siguen un estándar de calidad editorial. |
| **¿Para qué se usa?** | Revisar docs, READMEs, UI copy, y contenido editorial contra reglas de voz, tono, claridad y consistencia. |
| **¿Cómo se usa?** | Proporcionando archivos markdown o patrones de texto. El skill descarga las reglas desde el repo oficial y reporta hallazgos en formato `file:line`. |
| **¿Cómo debería usarse?** | Después de escribir cualquier documentación significativa o microcopy UI. |
| **¿Cuándo debería usarse?** | **Fase 4** — Revisión de contenido textual antes de publicación. |
| **Dependencias** | Skill de proyecto (`.agent/skills/writing-guidelines/`). Descarga guidelines de Vercel en tiempo real. Requiere conexión a internet. |
| **Requerimientos** | Conexión a internet. Archivos markdown o texto a revisar. |
