# Analytics Setup — Instrumentación de Métricas para Diseño

> Conecta decisiones de diseño con datos reales. Este documento cubre qué medir, cómo medirlo y cómo presentarlo.

---

## Tabla de Contenidos

1. [Principios de Instrumentación](#1-principios-de-instrumentación)
2. [Google Analytics 4](#2-google-analytics-4)
3. [Plausible Analytics](#3-plausible-analytics)
4. [Planes de Event Tracking](#4-planes-de-event-tracking)
5. [Attribution y Funnels](#5-attribution-y-funnels)
6. [Dashboards y Reportes](#6-dashboards-y-reportes)
7. [Benchmarks contra Competencia](#7-benchmarks-contra-competencia)
8. [Integración con el Orquestador](#8-integración-con-el-orquestador)

---

## 1. Principios de Instrumentación

### Reglas de oro

1. **No medir por medir.** Cada evento responde a una pregunta de negocio o diseño.
2. **Eventos con propósito.** Todo evento debe tener un "por qué" y una acción asociada.
3. **Ciclo de vida del evento.** Definir: nombre, trigger, properties, límite, retención.
4. **Privacidad primero.** Sin PII en eventos. Consentimiento obligatorio (GDPR/CCPA).
5. **Una fuente de verdad.** GA4 o Plausible como sistema canónico, no duplicar.

### Preguntas que responde cada tipo de evento

| Tipo | Pregunta | Evento |
|:-----|:---------|:-------|
| Engagement | ¿Los usuarios interactúan? | `scroll_depth`, `video_play`, `cta_click` |
| Conversión | ¿Completan el flujo? | `signup_complete`, `purchase`, `form_submit` |
| Fricción | ¿Dónde abandonan? | `dropoff_step_2`, `error_validation` |
| Performance | ¿Es rápido? | `lcp`, `fid`, `cls`, `ttfb` |
| Diseño | ¿Funciona visualmente? | `hero_cta_click_rate`, `color_variant_test` |

---

## 2. Google Analytics 4

### Configuración base

```javascript
// gtag.js — snippet base
window.dataLayer = window.dataLayer || [];
function gtag() { dataLayer.push(arguments); }
gtag('js', new Date());
gtag('config', 'G-XXXXXXXXXX', {
  send_page_view: true,
  cookie_flags: 'samesite=none;secure'
});

// Eventos personalizados
gtag('event', 'design_interaction', {
  component: 'hero',
  variant: 'A',
  action: 'cta_click'
});
```

### Eventos recomendados para diseño

| Evento | Parámetros | Disparador |
|:-------|:-----------|:-----------|
| `design_view` | `component`, `variant`, `viewport` | Component entra en viewport |
| `design_interaction` | `component`, `action`, `time_to_interact` | Click/hover/scroll en componente |
| `design_abandon` | `flow`, `step`, `time_on_step` | Usuario sale antes de completar |
| `design_error` | `component`, `error_type`, `user_agent` | Error visual/UX |
| `design_conversion` | `flow`, `variant`, `value` | Conversión completa |

### Propiedades globales

```javascript
gtag('set', 'user_properties', {
  design_variant: 'hero_v2',
  theme: 'dark',
  ab_test_group: 'control'
});
```

---

## 3. Plausible Analytics

### Ventajas sobre GA4

- Sin cookies → GDPR compliant por defecto
- Dashboard simple y público
- Sin consent banner necesario (sin PII)
- Ideal para landing pages y sitios de marca

### Configuración

```html
<script defer data-domain="tudominio.com" src="https://plausible.io/js/script.js"></script>
```

### Eventos personalizados

```javascript
plausible('CTA Click', { props: { location: 'hero', variant: 'primary' }});
plausible('Form Start', { props: { form: 'signup', step: 1 }});
plausible('Design Variant', { props: { component: 'pricing', variant: 'B' }});
```

### Goals por defecto

| Goal | Evento | Valor |
|:-----|:-------|:------|
| Visitas páginas | `pageview` | URL |
| CTA clicks | `CTA Click` | Location + variant |
| Form completado | `Form Submit` | Form name |
| Scroll 50% / 100% | `Scroll` | Percentage |
| Tiempo en página | `Duration` | Seconds |

---

## 4. Planes de Event Tracking

### Estructura de plan

```yaml
event:
  name: hero_cta_click
  category: engagement
  trigger: click on hero CTA button
  properties:
    - name: variant
      type: string
      values: [primary, secondary, outline]
    - name: position
      type: string
      values: [hero_1, hero_2, hero_3]
    - name: session_id
      type: string
      source: auto-generated
  limits:
    - 1 event per 5 seconds (debounce)
    - max 100 events per session
  retention: 14 months
```

### Template de plan

```markdown
## Event: [nombre]

| Campo | Valor |
|:------|:------|
| Nombre | `[nombre_evento]` |
| Categoría | Engagement / Conversión / Fricción / Performance |
| Trigger | [acción de usuario que dispara el evento] |
| Frecuencia esperada | [por sesión / por día] |
| Propiedades | [lista con tipo, valores posibles] |
| Límites | [rate limiting, max por sesión] |
| Retención | [meses] |
```

---

## 5. Attribution y Funnels

### Modelos de atribución

| Modelo | Cuándo usar | Ejemplo |
|:-------|:------------|:--------|
| **First Click** | Campañas de awareness | ¿Qué canal trajo al usuario por primera vez? |
| **Last Click** | Conversiones directas | ¿Qué touchpoint final convirtió? |
| **Linear** | Customer journeys largos | SaaS con trial de 30 días |
| **Time Decay** | Contenido editorial | Artículo que educa antes de convertir |
| **Position Based** | Mix de canales | 40% first + 20% middle + 40% last |

### Funnel de diseño típico

```mermaid
graph LR
    A[Landing visit] --> B[Hero engagement]
    B --> C[Feature scroll]
    C --> D[CTA click]
    D --> E[Signup start]
    E --> F[Signup complete]
    F --> G[Onboarding]
    
    style A fill:#1a1a2e
    style G fill:#1e3a1e
```

### Dropoff tracking

| Paso | Visitantes | Dropoff | Causa probable |
|:-----|:----------:|:-------:|:---------------|
| Landing visit | 10,000 | — | — |
| Hero engagement | 6,500 | 35% | Hero no comunica valor |
| Feature scroll | 4,200 | 35% | Features no relevantes |
| CTA click | 2,100 | 50% | CTA no visible o no convincente |
| Signup start | 1,000 | 52% | Formulario muy largo |
| Signup complete | 400 | 60% | Error en validación o fricción |
| Onboarding | 300 | 25% | Onboarding confuso |

---

## 6. Dashboards y Reportes

### Dashboard mínimo (Plausible)

- Visitantes únicos / día
- Pageviews por página
- Bounce rate
- Tasa de conversión por goal
- Tiempo promedio en página
- Top referrers

### Dashboard avanzado (GA4 + Looker Studio)

- Funnel completo con dropoff por paso
- Segmentación por variante de diseño
- Mapas de calor (Hotjar / Microsoft Clarity)
- Correlación diseño ↔ conversión
- Benchmarks semanales

---

## 7. Benchmarks contra Competencia

| Métrica | Referencia | Tu sitio | Gap |
|:--------|:----------:|:--------:|:---:|
| Bounce rate | 40-55% | — | — |
| Tasa conversión landing | 2-5% | — | — |
| Tiempo en página | 2-4 min | — | — |
| Scroll depth | 50-70% | — | — |
| CTA click rate | 3-10% | — | — |

---

## 8. Integración con el Orquestador

**Trigger words:** "analytics", "GA4", "Plausible", "event tracking", "attribution", "funnel", "dashboard", "métricas", "conversión", "dropoff"

**Herramientas free/open-source recomendadas:**
- Plausible — auto-hosted, sin cookies
- Umami — auto-hosted, lightweight
- PostHog — producto analytics + session recording (free tier)
- Microsoft Clarity — heatmaps gratis
- Google Looker Studio — dashboards gratis

**Flujo:** `strategy/metrics-framework.md` → `infrastructure/analytics-setup.md` → `impeccable` (auditar) → `handoff` (reporte)

