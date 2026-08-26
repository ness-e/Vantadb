# INV-{{MODULO}}-PROD — Investigación profunda: producto {{TIPO}} y evaluación integral de `{{MODULO}}`

> **ACTIVE INSTRUCTION — cargada por `/research <módulo>`** para módulos de tipo
> **producto/superficie** (web, desktop). Variante de `research-module.md`: la
> unidad de análisis es la EXPERIENCIA y el PRODUCTO, no una API pública.
> Read-only: ningún fix; hallazgos → informe + decisiones (Fase D del comando).

## Objetivo
Producir un informe de investigación profunda, detallada y amplia que evalúe
`{{MODULO}}` contra (a) las necesidades reales de sus usuarios objetivo
({{USUARIOS}}) y (b) productos comparables de referencia — para determinar con
evidencia: **qué tiene, qué le falta, qué es mejorable y qué es optimizable**.

## Alcance
- **Objeto:** `{{MODULO}}` completo — páginas/rutas, componentes, flujos de uso,
  design system, i18n, performance, empaquetado (si aplica), tests.
- **Frontera:** NO auditar el engine/core; el producto se evalúa como superficie.
- Modo **read-only**. Evidencia visual permitida y recomendada: capturas con
  `playwright-cli` (open/screenshot/console) contra dev server local.

## 1. Usuarios objetivo y sus flujos críticos
{{USUARIOS}}
Para cada flujo crítico documentar: paso a paso esperado, dónde se rompe hoy,
fricciones (copy confuso, estados vacíos/carga sin diseño, errores sin acción),
y evidencia real (issues propios, auditorías previas del repo, patrones de los
productos competidores).

## 2. Estándares del ecosistema
{{ESTANDARES}}

## 3. Productos de referencia — análisis mínimo obligatorio
{{COMPETIDORES}}
Por cada uno analizar: first impression (hero/messaging), flujos core (los 2-3
más importantes del producto), onboarding/quickstart, estados de error/vacío,
performance percibida (**con fuente reproducible o marcado "claim sin
evidencia"** — Regla 11), y qué copiarían abiertamente.

## 4. Estado actual de `{{MODULO}}` (interna, con evidencia file:line)
- Inventario: rutas/pantallas, componentes principales, design system en uso.
- Flujos implementados y su calidad (tests E2E/unit que los cubren, gaps).
- Auditorías previas del repo aplicables (WDA para web, DAUD/P34-P37 para desktop).
- Deudas conocidas registradas en Backlog (referenciarlas, no redescubrirlas).
- Performance/i18n/a11y medidos existentes o ausencia de ellos.

## 5. Framework de evaluación de producto (score 0-10, con evidencia)
First impression & messaging · flujos core completos sin fricción · accesibilidad
(WCAG 2.2 AA) · performance (Core Web Vitals / latencia de interacción) · i18n ·
consistencia de design system · robustez (errores, vacíos, carga, offline si
aplica) · seguridad (CSP/secrets/validación) · testabilidad (E2E existente) ·
**diferenciación** (¿qué tiene {{MODULO}} que {{COMPETIDOR_PRINCIPAL}} no?).

## Entregables → `docs/reviews/research-{{MODULO}}-prod-<YYYYMMDD>.md`
1. Matriz productos de referencia × dimensiones (incluyendo `{{MODULO}}`).
2. Gap analysis priorizado: falta (P0/P1/P2) · mejorable · optimizable.
3. Quick wins (<1 día) vs apuestas estratégicas (>1 semana).
4. **Apéndice obligatorio H-NN:** todo hallazgo con ID, categoría sugerida
   (APLICAR/MEJORAR/AGREGAR/OPTIMIZAR/ESTRATEGIA/DESCARTAR), severidad, esfuerzo,
   `file:line` o URL/pantalla. Alimenta la Fase D del comando.
5. Recomendaciones → filas FIND-\*/UX-\*/DESKTOP-\* según decisión (Fase D).
6. Claims de performance solo con medición reproducible citada (Regla 11).

## Método y fuentes
- Internet: sitios/products de competidores (analizarlos EN VIVO con
  playwright-cli: open + snapshot + screenshot), changelogs, guías de estilo del
  ecosistema (Next.js/Tailwind/Tauri docs oficiales).
- Interno: codegraph_explore de {{MODULO}}, tests, auditorías previas
  (`docs/reviews/*wda*`, `*daud*`, `*desktop*`), Backlog histórico,
  `campaign_memory_read(lessons|decisions)`.
- Skill Discovery (SDP): `frontend-design`/`frontend-ui-engineering`,
  `platform-design` (HIG/Material/WCAG), `visual-review`/`playwright-cli`,
  `coordinated-web-search`.
- RESULTADO final con SKILLS_CARGADAS y GATES_EVALUADOS.
