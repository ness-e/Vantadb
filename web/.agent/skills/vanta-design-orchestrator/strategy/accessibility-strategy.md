# Accessibility Strategy — Estrategia de Accesibilidad en Diseño

> La accesibilidad no es una capa de compliance — es una decisión de diseño que define quién puede usar tu producto. Este documento establece estándares, procesos y herramientas para garantizar WCAG 2.2 AA mínimo en todo el ecosistema.

---

## Tabla de Contenidos

1. [Principios de Accesibilidad en Diseño](#1-principios-de-accesibilidad-en-diseño)
2. [Estándares Mínimos](#2-estándares-mínimos)
3. [Accesibilidad en el Ciclo de Diseño](#3-accesibilidad-en-el-ciclo-de-diseño)
4. [Checklist Visual](#4-checklist-visual)
5. [Checklist de Componentes](#5-checklist-de-componentes)
6. [Herramientas de Auditoría](#6-herramientas-de-auditoría)
7. [Documentación y Governance](#7-documentación-y-governance)
8. [Integración con el Orquestador](#8-integración-con-el-orquestador)

---

## 1. Principios de Accesibilidad en Diseño

1. **La accesibilidad es una feature.** No es un "nice to have" post-lanzamiento. Se planifica desde el discovery.
2. **Diseño para el borde, escala al centro.** Si funciona para un usuario con discapacidad visual, funciona para todos con luz solar directa.
3. **No todo es contraste.** La accesibilidad incluye navegación por teclado, soporte de lectores de pantalla, texto redimensionable, y más.
4. **Automático + Manual.** Las herramientas automatizadas detectan ~30% de los problemas. El resto requiere juicio humano.
5. **Cumplir no es suficiente.** WCAG AA es el piso, no el techo. Apuntar a AAA donde sea posible.

### Impacto de no hacer accesibilidad

- **Legal:** Demandas por incumplimiento (ADA, EAA, Ley 7600)
- **Mercado:** 15% de la población mundial tiene alguna discapacidad
- **SEO:** Sitios accesibles rankean mejor (Core Web Vitals + Lighthouse score)
- **UX:** Las mejoras de accesibilidad benefician a todos los usuarios

---

## 2. Estándares Mínimos

### WCAG 2.2 Nivel AA — Obligatorio

| Principio | Criterios clave | Nivel |
|:----------|:----------------|:-----:|
| **Perceptible** | Alternativas textuales (1.1.1), subtítulos (1.2.2), contraste 4.5:1 (1.4.3), redimensionar texto 200% (1.4.4), espaciado de texto (1.4.12) | AA |
| **Operable** | Teclado navegable (2.1.1), orden de foco (2.4.3), skip link (2.4.1), touch target 44×44 (2.5.5), focus visible (2.4.7) | AA |
| **Comprensible** | Idioma de página (3.1.1), labels en inputs (3.3.2), sugerencias de error (3.3.3), ayuda contextual (3.3.5) | AA |
| **Robusto** | ARIA roles válidos (4.1.2), nombre accesible (4.1.2), estado y propiedades (4.1.2) | AA |

### Valores de contraste Vanta

| Tipo | Ratio mínimo | Notas |
|:-----|:------------:|:------|
| Texto normal (<18px) | 4.5:1 | WCAG AA |
| Texto grande (≥18px bold o ≥24px) | 3:1 | WCAG AA |
| Componentes UI (bordes, inputs) | 3:1 | WCAG AA |
| Texto en hover/focus | Ídem default | Sin cambio de ratio |
| Texto sobre imágenes | 4.5:1 | Verificar en todas las variantes |
| Placeholder / disabled | No requiere | Pero informar al usuario |

---

## 3. Accesibilidad en el Ciclo de Diseño

```mermaid
graph LR
    A[Discovery] -->|Checklist a11y en brief| B[Wireframes]
    B -->|Navegación por teclado| C[Visual Design]
    C -->|Contraste + focus| D[Prototipo]
    D -->|Test con lector de pantalla| E[Handoff]
    E -->|Tokens + specs a11y| F[Desarrollo]
    F -->|QA automatizado + manual| G[Lanzamiento]
    G -->|Auditoría continua| A

    style A fill:#1a1a2e
    style G fill:#1e3a1e
```

### Discovery
- [ ] Incluir perfiles con discapacidad en personas/empathy maps
- [ ] Definir target WCAG (AA mínimo, AAA ideal)
- [ ] Identificar dependencias de terceros que puedan tener gaps de accesibilidad

### Wireframes
- [ ] Orden de tabulación lógico (top→bottom, left→right)
- [ ] Skip link al contenido principal
- [ ] Landmarks semánticos (`header`, `nav`, `main`, `footer`)

### Visual Design
- [ ] Contraste 4.5:1 texto, 3:1 UI
- [ ] Focus indicators visibles (2px outline + offset)
- [ ] No información transmitida solo por color
- [ ] Iconos con label textual

### Handoff
- [ ] Design tokens con valores de contraste
- [ ] Specs de foco, tab order, ARIA roles
- [ ] Estados de componente documentados (hover, focus, active, disabled, error, loading, empty)

---

## 4. Checklist Visual

### Color

| Item | Criterio | Herramienta |
|:-----|:---------|:------------|
| Contraste texto/body | ≥4.5:1 | axe DevTools, WebAIM Contrast Checker |
| Contraste texto grande | ≥3:1 | axe DevTools |
| Contraste UI components | ≥3:1 | axe DevTools |
| No solo color para información | Pasa con desaturación | Visión Simulator (Chrome) |
| Modo oscuro verificado | Mismos ratios | Evaluación manual |

### Tipografía

| Item | Criterio | Herramienta |
|:-----|:---------|:------------|
| Tamaño base | ≥16px | Inspección |
| Redimensionar 200% | Sin pérdida de contenido | Zoom browser 200% |
| Espaciado de texto | 0.16× font-size / 0.12× / 0.16× / 0.12× | Bookmarklet WCAG Text Spacing |
| Alineación justificada | No permitida | Evaluación manual |

### Interacción

| Item | Criterio | Herramienta |
|:-----|:---------|:------------|
| Touch target | ≥44×44px | Inspección |
| Focus visible | 2px outline + offset | Navegación por teclado |
| Skip link | Primer focusable | Tab al cargar página |
| Timeout con aviso | Advertencia 20s antes | Evaluación manual |

---

## 5. Checklist de Componentes

| Componente | Consideraciones a11y clave |
|:-----------|:---------------------------|
| **Botón** | `role="button"`, focus visible, disabled state con estilo distinto de solo color |
| **Input** | `<label>` asociado, `aria-describedby` para error, `aria-required` si obligatorio |
| **Modal** | `role="dialog"`, `aria-modal="true"`, focus trap, cerrar con Escape |
| **Dropdown** | `role="listbox"`, `aria-expanded`, navegación con flechas |
| **Tab** | `role="tablist"`, `aria-selected`, `aria-controls` |
| **Tooltip** | `role="tooltip"`, `aria-describedby`, persistente en hover y focus |
| **Card** | Si es clickable, `<button>` o `<a>` con `aria-label` descriptivo |
| **Tabla de datos** | `<th>` con `scope`, `<caption>`, `aria-sort` en headers ordenables |
| **Formulario** | Validación inline, error agrupado al inicio, `aria-live` para feedback |
| **Imagen** | `alt` descriptivo (o `alt=""` si decorativa) |
| **Icono** | `aria-hidden="true"` + `role="img"` + `aria-label` si es informativo |
| **Loader/spinner** | `role="status"`, `aria-live="polite"` |
| **Notificación toast** | `role="alert"`, `aria-live="assertive"` |

---

## 6. Herramientas de Auditoría

| Herramienta | Propósito | Tipo | Costo |
|:------------|:----------|:----|:------|
| **axe DevTools** | Auditoría automatizada WCAG | Browser extension | Gratis |
| **Lighthouse CI** | Score a11y + gates CI | CLI / CI | Gratis |
| **Pa11y** | CLI auditoría programática | CLI | Gratis |
| **WAVE** | Visualización de problemas | Browser extension | Gratis |
| **Colour Contrast Analyser** | Verificación contraste | Desktop app | Gratis |
| **NVDA / VoiceOver** | Lector de pantalla | Built-in (Win/Mac) | Gratis |
| **Accessibility Insights** | Guided assessment + fast pass | Desktop + web | Gratis |
| **HTML CodeSniffer** | Bookmarklet de auditoría | Bookmarklet | Gratis |

### Pipeline CI/CD

```yaml
# .github/workflows/a11y.yml
name: Accessibility Audit
on: [pull_request]
jobs:
  a11y:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npx pa11y-ci --sitemap https://staging.example.com/sitemap.xml
      - run: npx lighthouse-ci https://staging.example.com --score=accessibility=90
```

---

## 7. Documentación y Governance

### RACI de accesibilidad

| Actividad | Designer | Developer | QA | PM | Legal |
|:----------|:--------:|:---------:|:--:|:--:|:-----:|
| Definir target WCAG | R | I | I | A | C |
| Diseñar con a11y | R | C | I | A | — |
| Implementar a11y | C | R | I | A | — |
| Auditar automático | I | C | R | — | — |
| Auditar manual | C | I | R | — | — |
| Aprobar lanzamiento | C | C | I | A | R |

### Declaración de accesibilidad

Toda página debe incluir enlace a declaración de accesibilidad con:
- Nivel de conformidad (WCAG 2.2 AA)
- Fecha de evaluación
- Tecnologías compatibles (browsers, screen readers)
- Canal de contacto para reportar problemas
- Plan de mejora continua

---

## 8. Integración con el Orquestador

**Trigger words:** "accesibilidad", "a11y", "WCAG", "contraste", "screen reader", "lector de pantalla", "teclado navegable", "focus", "ARIA", "disabled", "inclusivo", "discapacidad", "universal design", "Lighthouse", "axe"

**Skills relacionadas:**
- `web-design-guidelines` — Compliance de guidelines web (incluye a11y)
- `ux-heuristics` — Evaluación heurística con criterios de accesibilidad
- `design-systems` — Tokens de accesibilidad en design system
- `ui-design` — Color system con contraste, tipografía
- `platform-design` — Diseño cross-platform con a11y
- `infrastructure/tool-registry.md` §10 — Herramientas de accesibilidad
- `infrastructure/design-critique-templates.md` §5 — Review de accesibilidad

**Flujo:** `strategy/accessibility-strategy.md` (estándares) → `ui-design` (aplicar) → `ux-heuristics` (evaluar) → `web-design-guidelines` (compliancia) → `infrastructure/tool-registry.md` §10 (auditar con herramientas)

