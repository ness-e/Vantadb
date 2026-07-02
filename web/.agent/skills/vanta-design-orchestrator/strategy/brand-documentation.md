# Brand Documentation — Sistema de Documentación de Marca

> Documentar la marca no es un gasto administrativo — es el activo que multiplica la consistencia, protege la identidad, y permite escalar sin perder esencia.
> Este documento cubre los 6 artefactos documentales que toda marca profesional debe tener: Discover & Diagnosis, Brand Book, MIC, Manual Digital, Decision Log, y Brand Kit.

---

## 1. Discovery & Diagnosis

Documento previo que define el problema antes de diseñar. La fase más saltada y más crítica.

### Estructura

| Sección | Contenido | Herramientas |
|:--------|:----------|:-------------|
| **Internal Audit** | Estado actual de la marca: activos existentes, consistencia visual/verbal/legal, percepción interna | `visual-critique`, `strategy/brand-operations.md` (stress testing), entrevistas internas |
| **External Audit** | Percepción del mercado, competencia directa/indirecta, benchmark de identidad | `ux-strategy/competitive-analysis`, web scraping de referentes |
| **Business Context** | BMC, propuesta de valor, mercado target, diferenciación competitiva | `strategy/business-model-design.md` |
| **Stakeholder Alignment** | Expectativas, restricciones, presupuesto, timeline, RACI | `design-ops`, `strategy/brand-operations.md` (RACI) |
| **Brand Platform Draft** | Propósito, visión, misión, valores, territorio, personalidad | `strategy/brand-platform.md` |
| **Risk Assessment** | Riesgos legales (trademark conflicts), riesgos de naming, riesgos de percepción | `strategy/legal-protection.md` |
| **Scope Definition** | Qué se entrega, qué NO se entrega, fases y milestones | `design-md`, `lean-design` |

### Output
- **Diagnosis Document** (10-20 páginas, PDF/Notion)
- **Approved Brand Platform** (1 página, firmado por stakeholders)
- **RACI Matrix + SLA** (quién decide qué)
- **Brief de Diseño** para la fase de ejecución

### Trigger words
"diagnóstico de marca", "brand audit", "brand diagnosis", "discovery phase", "brand assessment", "evaluación de marca", "situación actual"

---

## 2. Brand Book

Documento aspiracional que captura el alma de la marca. Combina storytelling estratégico con guías de aplicación. Formato ideal: **30-50 páginas**, PDF interactivo + web.

### Estructura recomendada

| Sección | Páginas | Contenido clave |
|:--------|:--------|:----------------|
| **1. Brand Soul** | 4-6 | Propósito (why), visión, misión, valores, promesa de marca, personalidad (arquetipo + rasgos) |
| **2. Brand Story** | 2-4 | Origen, hito fundacional, narrativa de evolución, manifiesto |
| **3. Audience** | 2-3 | Arquetipos de audiencia, jobs-to-be-done, mapa de empatía (solo insights, no datos crudos) |
| **4. Competitive Landscape** | 1-2 | Posicionamiento diferencial, territorio de marca ocupado vs disponible |
| **5. Verbal Identity** | 4-6 | Voz (5 dimensiones), matriz de tono (7+ contextos), frases que decimos/no decimos, brand lexicon |
| **6. Visual Identity** | 8-12 | Logo system (versiones, clear space, tamaños), paleta de color con psicología, tipografía voz/funcional, fotografía/ilustración, iconografía, motion principles |
| **7. Applications** | 6-10 | Aplicaciones por canal: web, redes, impreso, empaque, video, eventos, email, PPT. Cada una con ejemplo real + do's/don'ts |
| **8. Governance** | 2-3 | Brand steward, proceso de aprobación, excepciones, contacto, changelog |

### Reglas de calidad
- **Mostrar, no decir**: Cada regla abstracta debe ir acompañada de ejemplo visual ✅/❌
- **20-35 páginas** es el sweet spot de usabilidad. >80 páginas = nadie lo lee
- **Formato vivo**: No PDF congelado. Versión web + PDF para descarga. Actualizar cada 12-24 meses
- **Brand Book + Brand Guide son complementarios**: El Brand Book inspira (estrategia + storytelling), la Brand Guide ejecuta (reglas técnicas)

### Trigger words
"brand book", "manual de marca", "brand bible", "brand manual", "guía de marca", "identidad de marca", "libro de marca"

---

## 3. MIC — Manual de Identidad Visual (Brand Guide)

Documento técnico que responde CÓMO usar los activos visuales. Es la especificación de implementación para diseñadores, desarrolladores, agencias y partners.

### Estructura técnica (15-25 páginas)

| Sección | Páginas | Contenido técnico |
|:--------|:--------|:------------------|
| **1. Logo System** | 4-6 | Versiones (full color, 1-bit black, 1-bit white, grayscale, inverso), proporciones, clear space mínimo (1x/2x), tamaño mínimo impreso/digital, qué NO hacer (8+ ejemplos), variantes por fondo |
| **2. Color System** | 3-4 | Paleta primaria/secundaria/semántica (success, error, warning, info) con valores: **HEX, RGB, CMYK, Pantone, OKLCH, HSL**. WCAG contrast ratios 4.5:1/3:1. Usos prohibidos. Variante dark mode |
| **3. Typography** | 3-4 | Tipografía de voz vs funcional, jerarquía (H1-H6 + body + caption + overline + micro). Font stack web, font-face declarations, tamaños mínimos. Variable font axes si aplica |
| **4. Photography & Illustration** | 2-3 | Estilo fotográfico (iluminación, composición, sujeto), tratamiento de color, do's/don'ts. Estilo de ilustración (line weight, color treatment, complexity level) |
| **5. Iconography** | 1-2 | Grid system, stroke weight, corner radius, sizes, semantics. Sistema de iconos vs ilustraciones |
| **6. Motion Principles** | 2-3 | Easing curves (cubic-bezier values), duraciones (micro: 150-200ms, UI: 200-300ms, page: 300-500ms), stagger patterns, prefers-reduced-motion fallback |
| **7. Grid & Spacing** | 2-3 | Grid system (columnas, gutter, margin), spacing scale (4-8-12-16-24-32-48-64-96), layout patterns, responsive breakpoints |
| **8. Applications** | 4-6 | Stationery (tarjetas, hojas, sobres), digital (website, email, social templates, app UI), physical (packaging, signage, merch). Cada uno con specs precisas |

### Diferencias clave con Brand Book

| Dimensión | Brand Book (inspira) | MIC / Brand Guide (ejecuta) |
|:----------|:---------------------|:----------------------------|
| **Audiencia** | Todos (ejecutivos, empleados, partners) | Diseñadores, desarrolladores, agencias |
| **Propósito** | ¿Por qué existe la marca? | ¿Cómo se aplica la marca? |
| **Contenido** | Estrategia + storytelling + reglas | Reglas técnicas + specs precisas |
| **Formato** | PDF editorial, web inspiracional | PDF técnico, web searchable, Figma library |
| **Actualización** | 12-24 meses (cambios estratégicos) | Continuo (nuevos templates, correcciones) |
| **Tono** | Narrativo, aspiracional | Instructivo, preciso, sin ambigüedad |
| **Cantidad** | 30-50 páginas | 15-25 páginas |

### Automatización recomendada
- Generar `strategy/sensory-identity.md` → convertir a secciones 2 y 3 del MIC
- Extraer motion tokens del design system → sección 6
- Usar screenshots de aplicaciones reales para sección 8
- Mantener en Figma + web + PDF, no solo PDF

### Trigger words
"manual de identidad visual", "MIC", "brand guidelines", "brand guide", "style guide", "guía de estilo", "brand standards", "normas de marca", "visual identity manual"

---

## 4. Manual de Marca Digital

Extensión del MIC enfocada exclusivamente en canales digitales. Especialmente importante porque la mayoría de interacciones son digitales y tienen requerimientos únicos.

### Estructura (10-15 páginas)

| Sección | Contenido |
|:--------|:----------|
| **1. Web & App** | Responsive logo behavior, favicon system (16×16 a 512×512), OG image specs (1200×630), social cards por plataforma, loading states branded |
| **2. Email** | Email signature (HTML + texto plano), newsletter templates (dark mode ready), transactional email specs, preview text optimization, responsive email constraints |
| **3. Social Media** | Perfiles (avatar, cover, bio specs por plataforma), templates por tipo de post, stories/reels frame specs, video specs (9:16, 16:9, 1:1), hashtag strategy alineada con voz de marca |
| **4. Video & Motion** | Intro/outro branded (duración: 3-5s), lower thirds, captions style, motion graphics templates, YouTube/Social video thumbnails |
| **5. Ads** | Display ad specs (300×250, 728×90, 320×50, etc.), social ad templates (Facebook, Instagram, LinkedIn, TikTok), dark mode considerations para ads |
| **6. Docs & Presentations** | PPT/Google Slides master, Doc/Google Docs templates, Notion template, PDF branded reports, data visualization style (charts, graphs) |
| **7. Digital Governance** | Proceso de publicación, review antes de publicar, versión "dark" de cada activo, assets descargables (en Brand Kit) |

### Reglas clave
- **Dark mode nativo**: Cada activo debe tener variante light + dark. No asumir fondo blanco
- **Responsive logo**: Sistema de logos que se adapta (horizontal → stacked → icon-only) según viewport
- **Accesibilidad digital**: WCAG AA mínimo en todos los activos, incluso imágenes decorativas con alt text
- **Consistencia cross-platform**: Los colores se ven diferente en cada pantalla. Proporcionar valores exactos para cada medio

### Trigger words
"manual de marca digital", "digital brand manual", "brand guidelines web", "guía de marca digital", "brand online guidelines", "activos digitales de marca"

---

## 5. Decision Log

Registro cronológico de decisiones de diseño con su fundamento estratégico. Previene la deriva de marca y responde "¿por qué se eligió esto?" cuando aparecen dudas.

### Formato

```markdown
## D-2026-001: Color primario
- **Fecha**: 2026-03-15
- **Decisión**: Azul profundo #0A1B3F → #1B3A6B
- **Razón**: Mejor contraste WCAG (3.1:1 → 4.7:1 con texto blanco) sin perder sofisticación
- **Alternativas consideradas**: #0D2B4E (muy frío), #1A2A4A (bajo contraste con modo oscuro)
- **Impacto**: Paleta completa recalibrada (11 tokens afectados)
- **Aprobado por**: Lead Design + Brand Steward
- **Próximo paso**: Actualizar MIC sección 2, regenerar Brand Kit
```

### ¿Cuándo registrar?

| Evento | Prioridad |
|:-------|:----------|
| Cambio en paleta de color | Alta |
| Cambio de logo o variante | Alta |
| Nueva tipografía o peso | Alta |
| Nueva directriz de tono de voz | Media |
| Nueva aplicación de marca imprevista | Media |
| Ajuste menor de spacing/grid | Baja |
| Nueva foto de stock o tratamiento | Baja (solo notificar) |

### Trigger words
"decision log", "registro de decisiones", "design decisions", "brand decisions", "por qué elegimos", "historial de decisiones", "design rationale"

---

## 6. Brand Kit Digital

Repositorio central de activos descargables. No es un documento — es una **carpeta pública** que contiene todo lo que partners, agencias, medios y empleados necesitan.

### Estructura de carpeta

```
brand-kit/
├── logo/
│   ├── full-color/
│   │   ├── logo-horizontal.svg
│   │   ├── logo-horizontal.png (1x, 2x)
│   │   ├── logo-stacked.svg
│   │   ├── logo-icon.svg
│   │   └── logo-icon.png (1x, 2x)
│   ├── mono-black/
│   ├── mono-white/
│   ├── grayscale/
│   └── favicon/
│       ├── favicon.ico (16×16, 32×32)
│       ├── apple-touch-icon.png (180×180)
│       ├── icon-192.png
│       └── icon-512.png
├── color/
│   ├── palette.pdf (swatches con valores)
│   ├── palette.ase (Adobe Swatch Exchange)
│   ├── palette.clr (macOS)
│   └── palette.json (tokens)
├── typography/
│   ├── fonts/ (WOFF2 + licencias)
│   └── type-scale.pdf
├── imagery/
│   ├── photography/ (hero seleccionadas)
│   ├── illustrations/ (SVG + PNG)
│   └── icons/ (SVG por set)
├── templates/
│   ├── social/ (PSD, AI, Figma, Canva)
│   ├── email/ (HTML + texto)
│   ├── presentation/ (PPT + Google Slides)
│   ├── document/ (DOC + Google Docs)
│   └── video/ (intro, lower thirds, outro)
├── brand-assets/
│   ├── texture/ (SVG patterns, grains)
│   ├── sound/ (brand audio, sonic logo)
│   └── motion/ (Lottie, Rive, JSON animaciones)
└── BRAND-KIT-README.md (instrucciones de uso, versión, contacto)
```

### Plataformas recomendadas
- **Gratuito**: Google Drive / Dropbox con estructura clara
- **Premium**: Brandfolder / Frontify / Bynder (con aprobación y tracking)
- **Developer-friendly**: Repo GitHub + CDN (jsdelivr, unpkg)
- **Website embed**: Página pública de descargas en el propio sitio

### Reglas
- Cada activo debe tener un naming consistente: `marca-tipo-contexto-variante.formato` (ej: `vantadb-logo-horizontal-full-color.svg`)
- No incluir assets en uso (borradores, WIP). Brand Kit = solo finales aprobados
- Versionar el Brand Kit con semver, vinculado al changelog de marca
- Incluir README con instrucciones, licencia de uso, y contacto del brand steward

### Trigger words
"brand kit", "kit de marca", "brand assets", "descargar logo", "assets de marca", "logotipo descargable", "recursos de marca", "media kit"

---

## Integración con el Orquestador

**Trigger words:** "documentación de marca", "brand documentation", "brand book", "MIC", "manual de identidad", "brand guidelines", "brand kit", "decision log", "descubrimiento de marca", "brand audit", "brand diagnosis", "digital brand manual", "activos digitales de marca", "brand bible"

**Flujo documental completo:**

```
Fase 0: Discovery & Diagnosis
  → strategy/brand-documentation.md §1 (Discovery & Diagnosis)
  → strategy/business-model-design.md (BMC + VPC)
  → strategy/brand-platform.md (Brand Platform draft)
  → strategy/legal-protection.md (Risk assessment)
  → Aprobación de stakeholders

Fase 0.5: Estrategia de Marca
  → strategy/brand-platform.md (Arquetipos, naming)
  → strategy/verbal-identity.md (Voz + tono)
  → strategy/decision-hierarchy.md (Activación)
  → strategy/sensory-identity.md (Psicología de color)

Fase 1: Brand Book + MIC
  → brandkit (Brand book board visual)
  → strategy/sensory-identity.md → secciones 2-3 del MIC
  → strategy/verbal-identity.md → sección 5 del Brand Book
  → canvas-design (Assets visuales finales)
  → strategy/brand-documentation.md §2 (Brand Book) + §3 (MIC)

Fase 2: Manual Digital + Brand Kit
  → strategy/brand-documentation.md §4 (Manual Digital) + §6 (Brand Kit)
  → Design system tokens → assets digitales
  → Brand Kit estructurado y publicado

Fase 3: Governance
  → strategy/brand-documentation.md §5 (Decision Log)
  → strategy/brand-operations.md (RACI, Changelog, Stress Testing)
  → Publicación en web de marca
```

**Relación con otros documentos:**

| Desde | Lee esto |
|:------|:---------|
| `SKILL.md` §1.6 (Routing) | `strategy/brand-documentation.md` §2-6 para rutas de documentación |
| `strategy/brand-platform.md` | Se usa como input para §1 (Diagnosis) y §2 (Brand Book) |
| `strategy/sensory-identity.md` | Se convierte en §3 (MIC) secciones 2-3 |
| `strategy/verbal-identity.md` | Se convierte en §2 (Brand Book) sección 5 |
| `strategy/brand-operations.md` | Se convierte en §5 (Decision Log) y §3 (Governance) |
| `strategy/legal-protection.md` | Se usa como input para §1 (Risk Assessment) |
| `strategy/metrics-framework.md` | Mide efectividad de la documentación (consistencia, adherence rate) |
