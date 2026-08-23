# Metrics Framework — Diseño Orientado a Métricas de Negocio

> El diseño no es arte — es inversión. Todo píxel debe poder vincularse a una métrica de negocio.
> Esta skill traduce decisiones de diseño a lenguaje de CFO/CEO.

---

## 1. Brand Equity como Activo

La marca es un activo intangible que puede valer más que el negocio físico.

| Métrica | Qué mide | Cómo lo afecta el diseño |
|:--------|:---------|:-------------------------|
| **Brand Awareness** | % del mercado target que reconoce la marca | Consistencia visual cross-channel, memorabilidad del logo |
| **Brand Recall** | Capacidad de recordar la marca sin ayuda | Diferenciación visual, singularidad del sistema de diseño |
| **Brand Preference** | % que elige la marca sobre competidores | Calidad percibida a través del diseño premium |
| **Brand Loyalty** | Repetición de compra, NPS | UX consistente, delight moments, reducción de fricción |
| **Brand Premium** | % extra que el cliente paga por la marca | Percepción de calidad a través del diseño |
| **Share of Search** | % de búsquedas de marca vs competidores en la categoría | SEO de marca, presencia en AI overviews, visibilidad cross-platform |
| **AI Visibility** | Frecuencia con que LLMs (ChatGPT, Claude, Perplexity) citan la marca en respuestas | Contenido estructurado, autoridad temática, OKF (Open Knowledge Format) |

### Share of Search — Fórmula
```
Share of Search = (Búsquedas de tu marca) / (Búsquedas totales de la categoría) × 100
```
- Correlaciona directamente con market share (1% SoS ≈ 1% market share a 3-6 meses)
- Trend: SoS creciente = brand momentum
- Se mide con Google Trends, Semrush, Ahrefs

### AI Visibility — Checklist
- ¿Aparece la marca en ChatGPT/Claude/Perplexity para queries genéricas de la categoría?
- ¿Hay un `llms.txt` optimizado en el sitio?
- ¿El contenido usa structured data (Schema.org, OKF, knowledge bundles)?
- ¿La marca tiene citas en fuentes autoritativas que los LLMs indexan?

### El cálculo simple del valor de marca
```
Brand Equity = (Precio de tu producto - Precio del genérico) × Unidades vendidas
```

---

## 2. CAC (Costo de Adquisición de Clientes)

Un buen branding **reduce el CAC** porque genera confianza previa.

| Factor de diseño | Impacto en CAC | Cómo |
|:-----------------|:---------------|:-----|
| **Consistencia visual** | -15-25% | Menos fricción cognitiva = más conversión |
| **Trust signals** | -10-20% | Testimonios visibles, certificaciones, diseño pulido |
| **Onboarding UX** | -20-30% | Menos churn en día 1 = CAC amortizado más rápido |
| **Mobile optimization** | -10-40% | Donde está el tráfico, debe estar la conversión |

### Fórmula CAC ajustada por diseño
```
CAC = (Gastos de marketing + Ventas) / Nuevos clientes
CAC con diseño optimizado = CAC base × (1 - factor de reducción por diseño)
```

---

## 3. ROAS — Return on Ad Spend

Métrica que vincula directamente el diseño de campañas publicitarias con resultados financieros.

### Fórmula
```
ROAS = Ingresos generados por anuncios / Gasto en anuncios
```
- ROAS > 4:1 = saludable (DTC benchmark)
- ROAS < 2:1 = el diseño creativo o targeting necesita revisión
- Cada variante de diseño (A/B test) debe reportar ROAS separado

### Factores de diseño que mejoran ROAS
| Factor | Impacto típico |
|:-------|:--------------|
| CTA visible + action-oriented | +25-60% CTR → +20-40% ROAS |
| Color contrast en hero/ad | +15-30% attention → +10-20% ROAS |
| Mobile-first ad creative | +20-50% en mobile ROAS |
| Consistent brand recognition | -10-20% CAC → +15-30% ROAS |

### Trigger
"ROAS", "return on ad spend", "rendimiento publicitario", "ad performance", "creative ROAS"

---

## 4. Métricas de Supervivencia (CASH Metrics)

Para startups y proyectos en etapa temprana, estas métricas determinan si la empresa vive o muere. El diseño debe optimizarse para extender runway.

### 4.1 Burn Rate
```
Gross Burn = Gastos totales mensuales
Net Burn = Gastos totales - Ingresos mensuales
```
- **Gross Burn**: Cuánto dinero sale cada mes (renta, nómina, herramientas, marketing)
- **Net Burn**: Cuánto dinero se pierde cada mes (Gross Burn - Revenue)
- Diseño lean (MVP, no features innecesarias) reduce Gross Burn directo

### 4.2 Runway
```
Runway (meses) = Efectivo disponible / Net Burn mensual
```
- Runway < 6 meses: modo supervivencia. Diseño mínimo viable, postergar animaciones y branding completo
- Runway 6-12 meses: diseño estratégico pero eficiente. Priorizar features de retención
- Runway > 12 meses: diseño completo con branding, animaciones, sistema de diseño

### 4.3 MRR — Monthly Recurring Revenue
```
MRR = Número de clientes × Ingreso promedio por cliente/mes
```
- Diseño impacta MRR vía: mejora de conversión (más clientes) y mejora de percepción de valor (mayor precio)
- MRR Growth Rate = ((MRR este mes - MRR mes anterior) / MRR mes anterior) × 100

### 4.4 ARR — Annual Recurring Revenue
```
ARR = MRR × 12
```
- Estándar de reporte para inversores
- Startups SaaS con ARR < $1M: focus en encontrar product-market fit
- ARR $1M-10M: focus en crecimiento eficiente (CAC payback < 12 meses)
- ARR > $10M: focus en expansión (NRR > 110%)

### 4.5 NRR — Net Revenue Retention
```
NRR = (MRR inicio + Expansion - Contraction - Churn) / MRR inicio × 100
```
- NRR > 100% = la base de clientes existente crece por sí misma (expansiones > churn + contraction)
- NRR < 100% = necesitas vender a nuevos clientes solo para mantener revenue plano
- Diseño impacta NRR vía: mejor onboarding (menos churn temprano), mejor UX (menos contraction), upsell flows (más expansion)

### Trigger
"burn rate", "runway", "MRR", "ARR", "NRR", "cash metrics", "supervivencia financiera", "tasa de quema", "pista financiera", "ingresos recurrentes"

---

## 5. HEART Framework Extendido

El framework de Google con métricas de diseño añadidas:

| Dimensión | Métrica UX | Métrica de negocio vinculada |
|:----------|:-----------|:----------------------------|
| **H**appiness | SUS Score, NPS, CSAT | Churn rate, renovaciones |
| **E**ngagement | DAU/MAU, session duration, acciones/usuario | ARPU, stickiness |
| **A**doption | % de usuarios que usan feature nueva, time-to-first-value | Expansion revenue, feature adoption rate |
| **R**etention | Day 1/7/30 retention, churn rate | LTV, payback period |
| **T**ask Success | Task completion rate, time on task, error rate | Conversion funnel efficiency |

---

## 6. Reducción de Fricción — Métricas Directas

Cada punto de fricción eliminado por diseño mejora métricas específicas:

| Fricción eliminada | Métrica que mejora | Mejora típica |
|:-------------------|:-------------------|:--------------|
| Formularios largos → cortos | Completion rate | +20-40% |
| Navegación confusa → clara | Task success rate | +30-50% |
| Load time lento → rápido | Bounce rate | -15-30% |
| CTA invisible → visible | Click-through rate | +25-60% |
| Error message críptico → claro | Error recovery rate | +40-60% |
| Mobile broken → responsive | Mobile conversion | +30-80% |
| Login friction → social/sso | Signup completion | +20-45% |

---

## 7. OKRs de Diseño

Ejemplos de OKRs que conectan diseño con negocio:

**Objetivo:** Reducir fricción en el funnel de registro
- KR1: Incrementar task completion rate del registro de 62% a 85% (Q2)
- KR2: Reducir time-on-task del registro de 4:30 a 2:00 minutos
- KR3: Disminuir errores en formulario de 8 a 2 por sesión

**Objetivo:** Aumentar percepción de calidad de marca
- KR1: Mejorar SUS Score de 68 a 82
- KR2: Incrementar NPS de 32 a 50
- KR3: Reducir bounce rate en landing page de 55% a 35%

**Objetivo:** Mejorar salud financiera del producto
- KR1: Reducir Net Burn de $50K a $35K (lean design, eliminar features no validadas)
- KR2: Aumentar NRR de 92% a 105% (mejor onboarding + UX improvements)
- KR3: Extender runway de 8 a 14 meses (eficiencia operativa por diseño)
- KR4: Incrementar Share of Search de 5% a 12% (brand awareness + AI visibility)

---

## 8. Tabla Resumen — Todas las Métricas

| Métrica | Fórmula clave | Benchmark saludable | Dónde se usa | Trigger word |
|:--------|:--------------|:--------------------|:-------------|:-------------|
| Brand Awareness | % mercado target | >40% en mercado conocido | Board, marketing | "brand awareness" |
| Brand Recall | % que recuerda sin ayuda | >20% | Brand tracking | "brand recall" |
| Share of Search | Búsquedas marca / total categoría | >10% de la categoría | Growth, brand | "share of search" |
| AI Visibility | Frecuencia en respuestas LLM | Aparecer en top 3 resultados | SEO, AI | "AI visibility" |
| NPS | Promotores - Detractores | >50 (excelente) | CX, producto | "NPS" |
| CSAT | Satisfacción en escala 1-5 | >4.2 | Soporte, producto | "CSAT" |
| CAC | Gasto marketing / clientes nuevos | Payback < 12 meses | Finanzas, growth | "CAC" |
| LTV | Ingreso promedio × tiempo retención | LTV:CAC > 3:1 | Finanzas, producto | "LTV" |
| ROAS | Ingresos ads / gasto ads | >4:1 | Marketing | "ROAS" |
| Gross Burn | Gastos totales/mes | <50% de funding/mes | Finanzas, CEO | "burn rate" |
| Runway | Efectivo / Net Burn | >12 meses | Finanzas, CEO | "runway" |
| MRR | Clientes × ingreso promedio | Crecimiento MoM >10% | SaaS, finanzas | "MRR" |
| ARR | MRR × 12 | Depende del stage | Inversores, board | "ARR" |
| NRR | (MRR inicio + net expansion) / MRR inicio | >100% (ideal >110%) | SaaS, producto | "NRR" |
| Churn Rate | Clientes perdidos / total | <5% mensual (SaaS) | Producto, éxito cliente | "churn" |
| HEART | Happiness, Engagement, Adoption, Retention, Task | Varía por producto | UX, producto | "HEART" |
| Friction metrics | Completion, error, time-on-task | Mejora continua | UX, conversión | "fricción" |

---

## Integración con el Orquestador

**Trigger words:** "métricas de diseño", "design metrics", "CAC", "LTV", "brand equity", "ROI de diseño", "HEART framework", "fricción", "friction", "OKR de diseño", "KPI de diseño", "conversión", "retention", "NPS", "ROAS", "burn rate", "runway", "MRR", "ARR", "NRR", "share of search", "AI visibility", "churn", "métricas financieras", "KPIs de startup", "métricas de crecimiento"

**Flujo:** `strategy/business-model-design.md` → `strategy/metrics-framework.md` → `design-ops/design-impact-reporting` → Fase 4

**Relación con stage de empresa:**

| Stage | Métricas prioritarias | Enfoque de diseño |
|:------|:----------------------|:------------------|
| **Pre-seed / Seed** | Runway, Burn Rate, MRR, Churn | Lean design, MVP validation, zero wasted pixels |
| **Series A** | CAC, LTV, NRR, NPS | Brand consistency, onboarding optimization, retention UX |
| **Series B+** | Share of Search, AI Visibility, Brand Equity, ROAS | Design system, brand documentation, premium perception |
| **Enterprise** | NPS, CSAT, Adoption Rate, Task Success | Accessibility, scalability, governance, localization |
