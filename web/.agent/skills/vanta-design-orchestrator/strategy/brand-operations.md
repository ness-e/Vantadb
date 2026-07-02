# Brand Operations — Gestión Avanzada de Marca

> La marca no termina en el manual de identidad. Requiere operaciones continuas: reglas de convivencia, roles claros, versionado, validación técnica, estrategias de crecimiento y herramientas de posicionamiento.
> Esta skill cubre 9 conceptos operacionales que transforman la marca de un documento estático a un sistema vivo.

---

## 1. Reglas de Convivencia de Marca (Co-branding)

Cuando dos o más marcas aparecen juntas, se necesitan reglas claras de jerarquía, espaciado y proporción para evitar confusión y preservar el equity de cada marca.

### 1.1 Jerarquía en Co-branding

| Rol | Quién lidera | Proporción de logos | Posición |
|:----|:-------------|:--------------------|:---------|
| **Lead Partner** | Tu marca lidera | 60:40 (tu marca mayor) | Izquierda o superior |
| **Equal Partner** | Relación equilibrada | 50:50 (ópticamente igual) | Centrados, mismo tamaño visual |
| **Minor Partner** | El partner lidera | 80:20 (partner mayor) | Derecha o inferior, 20% más pequeño |

### 1.2 Reglas Generales

- **Clear space mínima:** Altura de una letra (ej: altura de la "A" del logo principal) alrededor del lockup completo
- **No lockups forzados:** No fusionar logos en uno solo. Mantener separación visual clara
- **Designación de partnership:** Incluir texto "En partnership con", "Powered by", "En colaboración con"
- **Tratamiento equitativo:** Ambas marcas deben sentirse respetadas visualmente

### 1.3 Decision Tree para Co-branding

```
¿Quién lidera la comunicación?
├── Tu marca → Usa tus guidelines. Tu logo lidera (izquierda/superior)
│   └── ¿Espacio limitado? → Hairline lockup con aprobación del brand team
└── Partner → Sigue las guidelines del partner
    └── ¿Puedes usar tu sistema visual completo?
        ├── Sí → Usa logo en posición secundaria
        └── No → Solo logo + nombre, sin sistema visual
```

### 1.4 Múltiples Partners

| Situación | Regla |
|:----------|:------|
| **Tri-brand (relación igual)** | Logos al mismo tamaño, misma jerarquía visual |
| **Tri-brand (relación desigual)** | Dos logos principales juntos, el tercero en área separada |
| **Sponsors (4+ logos)** | Logos independientes con igual peso visual, sin lockup |

---

## 2. Matriz de Responsabilidades RACI

Define quién hace qué en la gobernanza de marca. Evita bloqueos y ambigüedad operativa.

### 2.1 Estructura RACI

| Rol | Descripción |
|:----|:------------|
| **R — Responsible** (Ejecuta) | Crea el activo o toma la acción |
| **A — Accountable** (Aprueba) | Responde por el resultado. Solo 1 por tarea |
| **C — Consulted** (Opina) | Aporta input antes de la decisión |
| **I — Informed** (Informado) | Recibe notificación después de la decisión |

### 2.2 RACI Aplicado a Marca

| Actividad | Diseñador | Brand Manager | CMO | Legal | Dev Lead |
|:----------|:---------:|:-------------:|:---:|:-----:|:--------:|
| Creación de logo | R | A | C | I | - |
| Guías de marca | R | A | C | C | I |
| Aplicación web | R | C | I | - | A |
| Co-branding | C | R | A | C | - |
| Registro de marca | - | R | A | C | - |
| Changelog de marca | I | R | A | I | - |
| Stress test de identidad | R | A | I | - | C |
| Actualización de tokens | R | A | C | - | C |

### 2.3 SLA Recomendados

| Actividad | SLA Máximo |
|:----------|:-----------|
| Revisión de diseño (Brand Manager) | 2 días hábiles |
| Aprobación CMO | 5 días hábiles |
| Revisión legal | 7 días hábiles |
| Publicación de cambio menor | 1 día hábil |
| Publicación de cambio mayor | 1 semana |

---

## 3. Changelog de Marca

Registro histórico público de versiones y cambios en el sistema de identidad. Mantiene transparencia y control de activos.

### 3.1 Formato

```
## [2.1.0] — 2026-06-15

### Added
- Nueva variante dark mode del isotipo
- Guía de animación para logo en motion

### Changed
- Actualización de paleta OKLCH (color-action-primary: de 55% 0.20 265 a 55% 0.18 260)
- Minimum size de 20mm a 18mm

### Deprecated
- Versión light del logo con tagline (usar submark en su lugar)

### Removed
- Variante duotone (bajo uso, eliminar por simplicidad)
```

### 3.2 Reglas
- **Semver estricto:** Major (cambio rompedor), Minor (nueva variante), Patch (ajuste técnico)
- **Público:** Accesible desde la web de marca o design system
- **Trazable:** Cada cambio vincula al ticket/issue que lo originó
- **Frecuencia:** Mínimo trimestral. Más frecuente en fases iniciales

---

## 4. Pruebas de Estrés de Identidad (Logo Stress Testing)

Validar cómo responde el logotipo y la paleta de color en condiciones adversas **antes** de lanzar.

### 4.1 Batería de Tests

| Test | Qué detecta | Cómo ejecutarlo | Pase mínimo |
|:-----|:------------|:----------------|:------------|
| **Favicon Test** | Detalles finos, trazos delgados | Escalar a 16×16 píxeles. ¿Se reconoce? | Silueta identificable, sin detalles críticos perdidos |
| **Monochrome Test** | Dependencia del color | Convertir a 100% blanco y negro (sin grises) | Funciona sin color. La forma comunica la marca |
| **Thermal/Receipt Test** | Contraste extremo | Simular impresión térmica (1-bit, baja resolución) | Legible en condiciones de impresión más adversas |
| **Blur/Squint Test** | Jerarquía visual | Desenfocar la vista o entrecerrar ojos | El elemento principal se distingue primero |
| **Cluttered Background** | Contraste y legibilidad | Colocar el logo sobre 5 fondos diferentes (texturados, fotográficos, patrones) | El logo no se pierde en ningún fondo |
| **Extreme Sizing** | Escalabilidad | Ver en 16px, 32px, 150px, 1024px y 10m (simulado) | Funciona en cada tamaño sin perder identidad |
| **Dark Mode Test** | Variante oscura | Colocar sobre fondo negro (#000, #111, #222) | La variante dark existe y funciona |

### 4.2 Sistema de Logo Responsivo

No uses un solo logo para todo. Define variantes:

| Variante | Uso | Tamaño típico |
|:---------|:---|:--------------|
| **Primary Logo** | Wordmark + Icon + Tagline | Header web, print grande |
| **Secondary Logo** | Wordmark + Icon | Header web compacto |
| **Submark / Icon** | Solo símbolo | Avatar social, favicon |
| **Favicon** | Símbolo simplificado | 16×16 / 32×32 px |

### 4.3 Regla de Minimum Size
```
Print:   20mm mínimo (primary), 15mm (secondary)
Digital: 32px mínimo (primary), 24px (secondary)
Favicon: 16px (submark/favicon simplificado)
```

Debajo del minimum size, usar la siguiente variante más simple.

---

## 5. Clasificación Técnica de Logos

| Tipo | Descripción | Cuándo usarlo | Ejemplo |
|:-----|:------------|:--------------|:--------|
| **Logotipo** | Solo texto (tipografía estilizada) | Marca nueva, nombre fuerte, sin necesidad de símbolo | Google, Coca-Cola, Sony |
| **Isotipo** | Solo símbolo (sin texto) | Marca establecida con alto reconocimiento | Apple, Nike, Twitter/X |
| **Imagotipo** | Símbolo + texto separados | Máxima flexibilidad: pueden usarse juntos o separados | Adidas, Pepsi, Mastercard |
| **Isologo** | Símbolo + texto integrados (inseparables) | Cuando el símbolo no funciona sin el texto | Burger King, Starbucks, Puma |

**Recomendación:** Tener al menos 2 variantes (logotipo para aplicaciones formales, isotipo para contexts small/icon). La mayoría de las marcas necesitan los 4 tipos en su sistema.

---

## 6. Fórmula del Elevator Pitch

Estructura de 30 segundos basada en la jerarquía estratégica de marca:

> **"Para [PÚBLICO], [MARCA] es la [CATEGORÍA] que [BENEFICIO] porque [RAZÓN DE CREER]."**

### Ejemplo:
> "Para **startups tecnológicas que necesitan crecer rápido**, **CRO-AI** es la **plataforma de optimización de conversión** que **aumenta tus ingresos recurrentes en un 30% en 90 días** porque **nuestro motor de IA aprende de cada visitante y personaliza la experiencia en tiempo real**."

### Variantes por contexto:

| Contexto | Tono | Longitud |
|:---------|:-----|:---------|
| Pitch de ventas | Beneficio + ROI | 30s |
| Networking casual | Propósito + diferencial | 15s |
| Web/Homepage hero | Impacto + audiencia | 5s |
| Bio de redes | Quién + para quién | 1 línea |

---

## 7. Análisis de Clientes Perdidos (Churned)

Investigar por qué los clientes abandonaron y ajustar el sistema de diseño en consecuencia.

### 7.1 Metodología

```
1. Seleccionar cohort de churned (últimos 90 días)
2 Segmentar por:
   ├── Onboarding: ¿Llegaron al "aha moment"?
   ├── Feature usage: ¿Usaron las features core?
   ├── Support: ¿Tuvieron tickets abiertos?
   └── UX friction: ¿Dónde encontraron obstáculos?
3. Entrevistar 5-10 clientes churned
4. Identificar patrones de fallo
5. Mapear cada patrón a:
   ├── Fix de diseño (UI/UX)
   ├── Fix de contenido (copy, onboarding)
   └── Fix de producto (feature, pricing)
```

### 7.2 Preguntas Clave para Entrevistas de Churn

| Pregunta | Qué revela |
|:---------|:-----------|
| "¿Qué esperabas lograr cuando te registraste?" | Expectativa vs realidad |
| "¿En qué momento pensaste 'esto no es para mí'?" | Punto exacto de fricción |
| "¿Qué te impedía hacer [tarea core]?" | Barrera de UX |
| "Si pudieras cambiar una cosa del producto, ¿qué sería?" | Prioridad de fix |
| "¿Qué solución encontraste?" | Competidor o alternativa |

### 7.3 Output
- **Churn Report:** Documento con patrones encontrados
- **UX Fix List:** Issues priorizados por severidad × frecuencia
- **Onboarding Redesign:** Si el churn ocurre en días 1-7

---

## 8. Ingeniería como Marketing

Estrategia de construir herramientas técnicas gratuitas (calculadoras, checkers, generadores) que atraen clientes de forma orgánica y compuesta.

### 8.1 El Concepto

En lugar de escribir blog posts o pagar ads, construye algo **útil** que tu audiencia quiere usar. La herramienta hace el marketing por ti.

**Ejemplos clásicos:**
- HubSpot Website Grader (65K visitas/mes, lanzado 2008)
- Ahrefs Free SEO Tools
- CoSchedule Headline Analyzer

### 8.2 Criterios para una Herramienta Efectiva

| Criterio | Descripción | Ejemplo |
|:---------|:------------|:--------|
| **Baja fricción** | Sin registro, funciona inmediatamente | Calculadora de ROI |
| **Valor obvio** | Resuelve un problema real del target | Checker de accesibilidad |
| **Relacionada al producto** | Usarla lleva naturalmente a tu producto | Analyzer de landing page |
| **Compartible** | La gente quiere compartir los resultados | Grade de email subject |
| **SEO-friendly** | Responde a búsquedas tipo "free X calculator" | "ROI calculator for startups" |

### 8.3 Satelite Apps (2026)

Con vibe coding (Lovable, Bolt, v0, Cursor), construir herramientas ya no requiere un equipo de engineering. Cualquier equipo de marketing puede crear una en una tarde.

| Funnel Stage | Tradicional | Satelite App |
|:-------------|:------------|:-------------|
| Top of funnel | Blog posts, SEO | Free calculators, widgets |
| Middle of funnel | Gated PDFs, lead magnets | Interactive assessments, benchmarks |
| Bottom of funnel | Demos, case studies | Personalized ROI calculators |

### 8.4 Proceso
```
1. Keyword research → "how to calculate", "free tool for", "[category] calculator"
2. Validar demanda → 500+ búsquedas/mes
3. MVP → Función core, UI limpia, sin registro obligatorio
4. Landing → SEO-optimizada, share buttons, email capture opcional
5. Distribución → Tool roundups, comunidades, social sharing
6. Medición → Traffic, backlinks, leads, conversion a producto
```

---

## 9. Estrategia de Especialización Selectiva

Diseñar distintas ofertas para diferentes segmentos de mercado. El producto puede ser el mismo, pero el mensaje, canal y envase cambian.

### 9.1 Matriz de Especialización

| Segmento | Mensaje | Canal principal | Diseño/UX | Pricing |
|:---------|:--------|:----------------|:----------|:--------|
| **Enterprise** | ROI, seguridad, compliance | LinkedIn, events | Sobrio, data-dense, accesible | Anual, quote-based |
| **SMB** | Crecimiento, facilidad | Google, content | Colorido, guided, templates | Monthly, self-serve |
| **Startup** | Velocidad, precio | Product Hunt, X/Twitter | Minimalista, API-first | Freemium, usage-based |
| **Agencia** | White-label, escalabilidad | Partnerships, SEO | Modular, customizable | Revenue share |

### 9.2 Regla de Implementación
> El **core del producto** es el mismo. Lo que cambia es: pricing page, onboarding flow, ejemplos/case studies, testimonios, y canales de distribución.

### 9.3 Diferenciación Visual por Segmento
- **Landing pages** diferentes por segmento (mismo design system, diferente énfasis)
- **Onboarding** personalizado según el caso de uso del segmento
- **Dashboard** con features visibles según el plan

---

## Integración con el Orquestador

**Trigger words:** "co-branding", "convivencia de marca", "partner branding", "RACI", "matriz de responsabilidades", "changelog de marca", "brand changelog", "logo stress test", "pruebas de estrés", "logo test", "logotipo isotipo imagotipo isologo", "elevator pitch", "churned", "clientes perdidos", "ingeniería como marketing", "engineering as marketing", "free tools", "calculadora gratis", "satellite apps", "especialización selectiva", "segmentación de mercado"

**Flujo:** `strategy/brand-operations.md` → `design-ops` (operacionalización) → Fase 4 (QA y testing)
