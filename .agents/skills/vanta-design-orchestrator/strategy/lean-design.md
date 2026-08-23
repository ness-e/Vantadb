# Lean Design — Crear, Medir, Aprender

> No esperes a tener el diseño "perfecto". Lanza una versión funcional, mide cómo responde el mercado, y pivota según datos reales.
> El diseño perfecto es el que sobrevive al contacto con el mercado.

---

## 1. Producto Mínimo Viable (PMV/MVP)

### ¿Qué es un MVP de diseño?
No es un producto feo o incompleto. Es la **versión más pequeña que puede validar una hipótesis de valor**.

### Criterios del MVP
- **Prueba una hipótesis:** Cada MVP responde una pregunta específica
- **Entrega valor core:** Resuelve el JTBD principal, sin features secundarias
- **Es usable:** Pasa el Trunk Test y cumple WCAG AA mínimo
- **Es medible:** Tiene métricas definidas para evaluar éxito/fracaso

### Qué incluir (y qué NO) en un MVP
| Incluir | No incluir |
|:--------|:-----------|
| Core flow completo (happy path) | Edge cases y error paths completos |
| Diseño limpio y funcional | Animaciones y micro-interacciones |
| Tokens base del design system | Sistema de temas completo (dark/light) |
| Branding esencial (logo, paleta) | Brand guidelines completos |
| Responsive básico (1-2 breakpoints) | Adaptación a todos los dispositivos |
| Accesibilidad AA core | Accesibilidad AAA |
| Analytics básicos | Dashboards complejos de analytics |

---

## 2. Ciclo Build-Measure-Learn

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  BUILD   │────→│  MEASURE │────→│  LEARN   │
│ (Diseño) │     │  (Data)  │     │ (Insight)│
└──────────┘     └──────────┘     └──────────┘
      ↑                                │
      └────────────────────────────────┘
               (Pivot o Persevere)
```

### Build
- Diseñar la hipótesis más riesgosa primero
- Prototipar en la fidelidad correcta para la pregunta
- Tiempo: 1-2 semanas por iteración

### Measure
- **Métrica de validación:** Definida antes de construir (ej: 30% de registro en semana 1)
- **Método:** A/B test, cohort analysis, usability test, entrevistas
- **Tiempo:** 1-4 semanas (depende del volumen de tráfico)

### Learn
- **Persevere:** La métrica superó el umbral → invertir más
- **Pivot:** La métrica no alcanzó el umbral → cambiar enfoque
- **Kill:** La métrica está lejos y no hay señales de mejora → cancelar

---

## 3. Pivotaje

### Tipos de Pivot (según Ries)
| Tipo | Señal de mercado | Cambio de diseño |
|:-----|:-----------------|:-----------------|
| **Zoom-in Pivot** | Una feature tiene más tracción que el producto completo | Rediseñar alrededor de esa feature |
| **Zoom-out Pivot** | El producto es una feature de algo más grande | Expandir el alcance del diseño |
| **Customer Segment Pivot** | El producto resuelve el problema pero para otro segmento | Rebranding + reenfocar UX |
| **Customer Need Pivot** | El segmento correcto tiene un problema diferente | Rediseñar propuesta de valor |
| **Platform Pivot** | De app a plataforma (o viceversa) | Arquitectura de producto diferente |
| **Business Architecture Pivot** | Cambio de B2B a B2C (o viceversa) | Tono, UX, pricing rediseñados |
| **Value Capture Pivot** | Cambio de monetización | Diseño de pricing page, upsell flows |
| **Engine of Growth Pivot** | Cambio de viral a paid a sticky | UX enfocado en sharing, ads, o engagement |
| **Channel Pivot** | Cambio de canal de distribución | Rediseño para el nuevo canal |
| **Technology Pivot** | Nueva tecnología más eficiente | Refactor de diseño aprovechando nuevas capacidades |

### Regla de pivotaje para diseño
> Cuando pivoteas, no descartes el trabajo anterior — recicla los componentes, tokens y aprendizajes. El diseño system debe ser lo suficientemente modular para absorber un pivot sin reescribirse completo.

---

## 4. Validación Antes de Perfección

### Pirámide de Validación
```
                  ┌─────────────┐
                  │   ¿Lo usa?  │  ← Analytics, cohorts, retention
                 ┌┴─────────────┴┐
                 │  ¿Lo entiende?│  ← Usability test, Trunk Test
                ┌┴───────────────┴┐
                │ ¿Lo necesita?   │  ← Entrevistas, surveys, JTBD
               ┌┴─────────────────┴┐
               │ ¿Lo quiere pagar?  │  ← Pricing page, pre-orders
```

### Regla
No pasar al siguiente nivel de fidelidad hasta que el nivel actual esté validado.
- **No diseñar UI** hasta que la propuesta de valor esté validada (entrevistas)
- **No diseñar interacciones** hasta que la estructura pase el Trunk Test
- **No diseñar animaciones** hasta que el layout esté validado (A/B test)
- **No producir assets finales** hasta que las métricas de engagement sean positivas

---

## Integración con el Orquestador

**Trigger words:** "lean design", "MVP", "producto mínimo viable", "minimum viable product", "build measure learn", "pivot", "pivotaje", "validación", "hypothesis testing", "validar antes de perfeccionar"

**Flujo:** `strategy/business-model-design.md` → `strategy/lean-design.md` → `prototyping-testing` → Fase 1-4 (iterativo)
