# Design Critique Templates — Plantillas de Revisión de Diseño

> Plantillas estructuradas para crítica de diseño en diferentes contextos. Cada plantilla sigue el principio: **feedback específico, fundamentado y accionable**.

---

## Tabla de Contenidos

1. [Principios de la Crítica de Diseño](#1-principios-de-la-crítica-de-diseño)
2. [Critique Rápida (15 min)](#2-critique-rápida-15-min)
3. [Critique Completa (60 min)](#3-critique-completa-60-min)
4. [Auditoría Heurística](#4-auditoría-heurística)
5. [Review de Accesibilidad](#5-review-de-accesibilidad)
6. [Review de Consistencia de Marca](#6-review-de-consistencia-de-marca)
7. [Review de Motion / Animación](#7-review-de-motion--animación)
8. [Review de Copy / UX Writing](#8-review-de-copy--ux-writing)
9. [Priorización y Seguimiento](#9-priorización-y-seguimiento)
10. [Integración con el Orquestador](#10-integración-con-el-orquestador)

---

## 1. Principios de la Crítica de Diseño

### Reglas de oro

1. **Crítica al diseño, no al diseñador.** Usar "este componente tiene..." no "tú hiciste mal..."
2. **Fundamentar cada punto.** "No funciona" no es crítica. "Esto no funciona porque viola la heurística de reconocimiento en vez de recuerdo" sí lo es.
3. **Separar problema de solución.** Primero identificar qué está mal. Después proponer cómo arreglarlo.
4. **Tres modos:** Praise (qué funciona), Polish (qué mejorar), Problem (qué está roto).
5. **Priorizar.** No todo es crítico. Usar la escala de severidad.

### Escala de severidad

| Nivel | Etiqueta | Significado | Acción |
|:-----:|:---------|:------------|:-------|
| 0 | ✅ Cosmético | No afecta usabilidad | Corregir si hay tiempo |
| 1 | 🔷 Menor | Fricción leve | Prioridad baja |
| 2 | 🟡 Mayor | Afecta tarea común | Corregir pronto |
| 3 | 🔴 Crítico | Bloquea tarea principal | Corregir antes de lanzar |
| 4 | ⚫ Catástrofe | Hace el producto inusable | Detener todo |

---

## 2. Critique Rápida (15 min)

Para revisiones diarias, feedback rápido entre pares, o cuando no hay tiempo para crítica completa.

```markdown
## Critique Rápida — [Componente/Página]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### ✅ Praise (2-3 cosas que funcionan)
1. [ ]
2. [ ]
3. [ ]

### 🔧 Polish (2-3 cosas que mejorar)
1. [ ] — Severidad: 🟡/🔷
   - Problema: [ ]
   - Sugerencia: [ ]
2. [ ] — Severidad: 🟡/🔷
   - Problema: [ ]
   - Sugerencia: [ ]

### ⚠️ Problems (urgentes)
1. [ ] — Severidad: 🔴/⚫
   - Problema: [ ]
   - Por qué es problema: [ ]
   - Sugerencia: [ ]
```

---

## 3. Critique Completa (60 min)

Para revisiones de milestone, diseño nuevo, o antes de handoff a desarrollo.

```markdown
# Critique Completa — [Proyecto/Página]

**Revisor(es):** [nombres]
**Fecha:** [YYYY-MM-DD]
**Versión del diseño:** [v1.2 / Figma link]

---

## 1. Resumen Ejecutivo

| Métrica | Score |
|:--------|:-----:|
| Usabilidad general | ★★★☆☆ |
| Consistencia visual | ★★★★☆ |
| Accesibilidad | ★★★☆☆ |
| Copy / Contenido | ★★★★☆ |
| Motion / Interacción | ★★★☆☆ |
| **Promedio** | **★★★☆☆** |

**Hallazgos críticos:** [0]
**Hallazgos mayores:** [3]
**Hallazgos menores:** [5]
**Cosméticos:** [2]

**¿Listo para producción?** Sí / No / Condicional
**Si condicional:** [requisito para aprobar]

---

## 2. Praise (Qué funciona)

### Excelente
| Elemento | Por qué |
|:---------|:--------|
| [ ] | [ ] |

### Bueno
| Elemento | Por qué |
|:---------|:--------|
| [ ] | [ ] |

---

## 3. Problemas por Categoría

### 🏗️ Estructura y Layout

| # | Elemento | Problema | Severidad | Sugerencia |
|:-:|:---------|:---------|:---------:|:-----------|
| 1 | [ ] | [ ] | 🔴/🟡/🔷 | [ ] |

### 🎨 Visual y Jerarquía

| # | Elemento | Problema | Severidad | Sugerencia |
|:-:|:---------|:---------|:---------:|:-----------|
| 1 | [ ] | [ ] | 🔴/🟡/🔷 | [ ] |

### ♿ Accesibilidad

| # | Elemento | Problema | WCAG | Severidad | Sugerencia |
|:-:|:---------|:---------|:----:|:---------:|:-----------|
| 1 | [ ] | [ ] | [SC 1.4.3] | 🔴/🟡 | [ ] |

### ✍️ Copy y Contenido

| # | Elemento | Problema | Severidad | Sugerencia |
|:-:|:---------|:---------|:---------:|:-----------|
| 1 | [ ] | [ ] | 🔴/🟡/🔷 | [ ] |

### 🎬 Motion e Interacción

| # | Elemento | Problema | Severidad | Sugerencia |
|:-:|:---------|:---------|:---------:|:-----------|
| 1 | [ ] | [ ] | 🔴/🟡/🔷 | [ ] |

---

## 4. Screenshots / Anotaciones

_[Adjuntar screenshots con anotaciones]_

| Screenshot | Problema |
|:-----------|:---------|
| [img] | [ ] |

---

## 5. Plan de Acción

| Prioridad | Item | Dueño | Deadline |
|:---------:|:-----|:-----|:--------:|
| 🔴 | [ ] | [ ] | [ ] |
| 🟡 | [ ] | [ ] | [ ] |
| 🔷 | [ ] | [ ] | [ ] |
```

---

## 4. Auditoría Heurística

Basada en las 10 heurísticas de Nielsen + Trunk Test de Krug.

```markdown
## Auditoría Heurística — [Componente/Página]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### Trunk Test (Krug)
| Pregunta | Sí/No | Notas |
|:---------|:-----:|:------|
| ¿Qué sitio es? (identidad) | [ ] | [ ] |
| ¿En qué página estoy? | [ ] | [ ] |
| ¿Cuáles son las opciones principales? | [ ] | [ ] |
| ¿Cómo busco / navego? | [ ] | [ ] |

### 10 Heurísticas de Nielsen

| # | Heurística | Score (0-4) | Evidencia | Recomendación |
|:-:|:-----------|:----------:|:----------|:--------------|
| 1 | Visibilidad del estado del sistema | [ ] | [ ] | [ ] |
| 2 | Relación sistema-mundo real | [ ] | [ ] | [ ] |
| 3 | Control y libertad del usuario | [ ] | [ ] | [ ] |
| 4 | Consistencia y estándares | [ ] | [ ] | [ ] |
| 5 | Prevención de errores | [ ] | [ ] | [ ] |
| 6 | Reconocimiento en vez de recuerdo | [ ] | [ ] | [ ] |
| 7 | Flexibilidad y eficiencia | [ ] | [ ] | [ ] |
| 8 | Estética y diseño minimalista | [ ] | [ ] | [ ] |
| 9 | Ayuda a reconocer y recuperarse | [ ] | [ ] | [ ] |
| 10 | Ayuda y documentación | [ ] | [ ] | [ ] |

### Score de severidad
| Score | Significado |
|:-----:|:------------|
| 0 | No es problema de usabilidad |
| 1 | Cosmético |
| 2 | Menor |
| 3 | Mayor |
| 4 | Catástrofe |
```

---

## 5. Review de Accesibilidad

```markdown
## Review de Accesibilidad — [Componente/Página]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### Perceptible

| Criterio | WCAG | Cumple | Evidencia |
|:---------|:----:|:------:|:----------|
| Alternativas textuales | 1.1.1 | ✅/❌ | [ ] |
| Subtítulos multimedia | 1.2.2 | ✅/❌ | [ ] |
| Contraste de color | 1.4.3 | ✅/❌ | [ ] |
| Redimensionar texto 200% | 1.4.4 | ✅/❌ | [ ] |
| Espaciado de texto | 1.4.12 | ✅/❌ | [ ] |

### Operable

| Criterio | WCAG | Cumple | Evidencia |
|:---------|:----:|:------:|:----------|
| Teclado navegable | 2.1.1 | ✅/❌ | [ ] |
| Orden de foco | 2.4.3 | ✅/❌ | [ ] |
| Skip link | 2.4.1 | ✅/❌ | [ ] |
| Touch target 44×44 | 2.5.5 | ✅/❌ | [ ] |

### Comprensible

| Criterio | WCAG | Cumple | Evidencia |
|:---------|:----:|:------:|:----------|
| Idioma de página | 3.1.1 | ✅/❌ | [ ] |
| Labels en inputs | 3.3.2 | ✅/❌ | [ ] |
| Sugerencias de error | 3.3.3 | ✅/❌ | [ ] |

### Robusto

| Criterio | WCAG | Cumple | Evidencia |
|:---------|:----:|:------:|:----------|
| ARIA roles válidos | 4.1.2 | ✅/❌ | [ ] |
| Nombre accesible | 4.1.2 | ✅/❌ | [ ] |
```

---

## 6. Review de Consistencia de Marca

```markdown
## Review de Marca — [Proyecto]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### Identidad Visual

| Elemento | Documentado | Implementado | Gap |
|:---------|:-----------:|:------------:|:---:|
| Colores | ✅/❌ | ✅/❌ | [ ] |
| Tipografía | ✅/❌ | ✅/❌ | [ ] |
| Espaciado | ✅/❌ | ✅/❌ | [ ] |
| Iconografía | ✅/❌ | ✅/❌ | [ ] |
| Fotografía/ilustración | ✅/❌ | ✅/❌ | [ ] |
| Motion | ✅/❌ | ✅/❌ | [ ] |

### Voz y Tono

| Dimensión | Documentado | Implementado | Gap |
|:----------|:-----------:|:------------:|:---:|
| Tono por canal | ✅/❌ | ✅/❌ | [ ] |
| Microcopy system | ✅/❌ | ✅/❌ | [ ] |
| Error messages | ✅/❌ | ✅/❌ | [ ] |
| CTA language | ✅/❌ | ✅/❌ | [ ] |

### Stress Tests

| Test | Resultado | Notas |
|:-----|:---------:|:------|
| Favicon 16×16 | ✅/❌ | [ ] |
| Monochrome (1-bit) | ✅/❌ | [ ] |
| Escala mínima | ✅/❌ | [ ] |
| Escala máxima | ✅/❌ | [ ] |
| Fondo blanco | ✅/❌ | [ ] |
| Fondo negro | ✅/❌ | [ ] |
| Co-branding | ✅/❌ | [ ] |
```

---

## 7. Review de Motion / Animación

```markdown
## Review de Motion — [Componente/Flujo]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### Checklist de Motion

| Criterio | Cumple | Notas |
|:---------|:------:|:------|
| Duración 150-300ms | ✅/❌ | [ ] |
| Máximo 500ms transiciones | ✅/❌ | [ ] |
| Easing físico (spring/cubic-bezier) | ✅/❌ | [ ] |
| Sin ease-in UI | ✅/❌ | [ ] |
| GPU-accelerated (transform + opacity) | ✅/❌ | [ ] |
| prefers-reduced-motion | ✅/❌ | [ ] |
| No animaciones decorativas sin propósito | ✅/❌ | [ ] |
| Stagger coherente | ✅/❌ | [ ] |
| Sin clipping por overflow | ✅/❌ | [ ] |

### Problemas detectados

| # | Elemento | Problema | Severidad |
|:-:|:---------|:---------|:---------:|
| 1 | [ ] | [ ] | 🔴/🟡/🔷 |
```

---

## 8. Review de Copy / UX Writing

```markdown
## Review de Copy — [Componente/Página]

**Revisor:** [nombre]
**Fecha:** [YYYY-MM-DD]

### Checklist de Copy

| Criterio | Cumple | Notas |
|:---------|:------:|:------|
| Sin "haz clic aquí" | ✅/❌ | [ ] |
| CTAs: verbo + objeto | ✅/❌ | [ ] |
| Sin jerga técnica innecesaria | ✅/❌ | [ ] |
| Lectura nivel B1 | ✅/❌ | [ ] |
| Estados cubiertos (empty, error, success, loading) | ✅/❌ | [ ] |
| Tono consistente con guía de voz | ✅/❌ | [ ] |
| Sin exclamaciones múltiples | ✅/❌ | [ ] |
| Títulos ≤8 palabras | ✅/❌ | [ ] |
| Párrafos ≤3 líneas | ✅/❌ | [ ] |

### Copy por estado

| Componente | Estado | Copy actual | Copy propuesto |
|:-----------|:-------|:------------|:---------------|
| [ ] | Default | [ ] | [ ] |
| [ ] | Error | [ ] | [ ] |
| [ ] | Empty | [ ] | [ ] |
| [ ] | Success | [ ] | [ ] |
```

---

## 9. Priorización y Seguimiento

### Matriz Impacto × Esfuerzo

```markdown
## Plan de Acción — [Proyecto]

| Item | Severidad | Impacto | Esfuerzo | Prioridad | Dueño | Deadline |
|:-----|:---------:|:-------:|:--------:|:---------:|:-----|:--------:|
| [ ] | 🔴 | Alto | Bajo | **P0** | [ ] | [ ] |
| [ ] | 🟡 | Alto | Bajo | **P1** | [ ] | [ ] |
| [ ] | 🟡 | Medio | Medio | **P2** | [ ] | [ ] |
| [ ] | 🔷 | Bajo | Bajo | **P3** | [ ] | [ ] |
```

### Seguimiento semanal

```markdown
## Seguimiento — Semana [XX]

| Item | Estado | Notas |
|:-----|:------:|:------|
| [ ] | ✅/🔄/❌ | [ ] |
| [ ] | ✅/🔄/❌ | [ ] |

**Próximos pasos:** [ ]
```

---

## 10. Integración con el Orquestador

**Trigger words:** "design critique", "review de diseño", "auditoría", "feedback de diseño", "crítica de diseño", "revisión por pares", "design review", "heuristic evaluation", "spot check"

**Skills relacionadas:**
- `visual-critique` — Crítica visual automatizada
- `impeccable` — Auditoría completa (51 reglas)
- `design-review` — Revisión profesional de diseños
- `ux-heuristics` — Evaluación heurística (Nielsen 10)
- `plan-design-review` — Plan de revisión de diseño
- `design-motion-principles` — Auditoría de motion
- `web-design-guidelines` — Compliance web
- `writing-guidelines` — Compliance de contenido
- `strategy/validation-sustainability.md` — Ciclo de validación

**Flujo:** `infrastructure/design-critique-templates.md` (elegir plantilla) → skill de auditoría correspondiente → `impeccable` (polish) → `plan-design-review` (reporte)

