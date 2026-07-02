# Sensory Identity — Psicología del Color, Forma y Percepción

> El diseño no solo se ve — se siente. El color influye en el **85% de las decisiones de compra**. Las formas comunican personalidad antes de leer una palabra.

---

## 1. Psicología del Color Avanzada

### Significados por Color

| Color | Psicología | Sectores típicos | Riesgo |
|:------|:-----------|:-----------------|:-------|
| **Azul** | Confianza, seguridad, tecnología, calma | Fintech, salud, enterprise, SaaS | Frío, corporativo genérico |
| **Rojo** | Urgencia, pasión, apetito, energía | Food, retail, entertainment, sports | Agresivo, ansiedad |
| **Verde** | Salud, naturaleza, finanzas, crecimiento | Salud, fintech, eco, orgánico | Asociación genérica "eco" |
| **Naranja** | Honestidad, accesibilidad, diversión | SaaS, educación, food delivery | Barato si se usa mal |
| **Amarillo** | Optimismo, claridad, juventud | Kids, food, alerts | Cansancio visual en grandes áreas |
| **Púrpura** | Creatividad, lujo, imaginación | Beauty, luxury, spirituality | Elitista si se sobreusa |
| **Rosa** | Empatía, calidez, feminidad | Beauty, health, dating | Encasillamiento de género |
| **Negro** | Poder, sofisticación, minimalismo | Luxury, tech, fashion | Frío, inaccesible |
| **Blanco** | Pureza, limpieza, simplicidad | Healthcare, minimalism, tech | Difícil de mantener en impresión |
| **Marrón/Ocre** | Estabilidad, natural, elegancia orgánica | Coffee, craft, outdoor, premium | Aburrido si se usa sin contraste |
| **Gris** | Neutralidad, profesionalismo, madurez | B2B, corporate, legal | Depresivo, sin personalidad |

### Técnico: Formatos de Color

Cada decisión de color debe documentarse en 4 formatos:

```css
/* Digital */
--brand-primary: oklch(55% 0.20 265);  /* Color space moderno */
--brand-primary-hsl: hsl(220, 100%, 50%); /* Fallback */
--brand-primary-hex: #0066FF;              /* Universal */

/* Impresión */
--brand-primary-cmyk: cmyk(100, 50, 0, 0);
--brand-primary-pantone: 286 C;
```

### Accesibilidad Cromática
- **Texto normal:** Contraste ≥ 4.5:1 (WCAG AA)
- **Texto grande (≥18px bold o ≥24px):** Contraste ≥ 3:1
- **Componentes UI:** Contraste ≥ 3:1 contra adyacentes
- **Modo oscuro:** No invertir. Rediseñar superficies con elevación por luminosidad. Usar off-white `#E0E0E0` no pure white.
- **Colorblind-safe:** No usar solo color para información crítica. Acompañar con iconos/patrones.

---

## 2. Formas, Estructura y Geometría

### Psicología de la Forma

| Forma | Comunicación | Uso ideal |
|:------|:-------------|:----------|
| **Círculo** | Comunidad, protección, infinito | Logos, avatares, CTAs suaves |
| **Cuadrado** | Estabilidad, confianza, orden | Cards, dashboards, iconos |
| **Triángulo** | Dinamismo, dirección, poder | Flechas, señales, gaming |
| **Espiral** | Crecimiento, evolución, creatividad | Marcas orgánicas, wellness |
| **Orgánica/Asimétrica** | Natural, humano, imperfecto | Marcas artisanales, creativas |

### Psicología de Formas Orgánicas (2026)

Como respuesta a la perfección artificial de la IA, las formas orgánicas están en auge:

| Técnica | Efecto psicológico | Aplicación |
|:--------|:-------------------|:-----------|
| **Bordes redondeados suaves** | Cercanía, seguridad, accesibilidad | Cards, CTAs, modales |
| **Curvas bezier irregulares** | Natural, hecho a mano, único | Logos, ilustraciones, secciones hero |
| **Asimetría intencional** | Dinamismo, movimiento, no genérico | Layouts, composiciones, bento grids |
| **Blobs orgánicos** | Fluidez, adaptabilidad, moderno | Fondos, backgrounds, transiciones |
| **Formas ameboides** | Creatividad, innovación, sorpresa | Secciones hero, elementos decorativos |
| **Imperfección controlada** | Autenticidad, confianza, humano | Texturas, ilustraciones, iconografía |

**Regla:** La forma orgánica debe sentirse **intencional**, no accidental. Si el ojo no percibe que hay una decisión de diseño detrás, parece un error.

### Retículas y Composición
- **Ángulos dinámicos:** 15° o 45° para expresar dinamismo
- **Zona de Exclusión (Área de Protección):** Mínimo la mitad del ancho del logo alrededor
- **Medidas Mínimas:**
  - Favicon: 16px
  - Logo digital: 32px mínimo
  - Logo impreso: 20mm mínimo
  - Body text: 16px mínimo

### Identidades Adaptativas (2026)
El diseño debe ser **elástico**: funcionar igual de bien en un avatar de 16px que en una valla publicitaria de 10m.

| Formato | Consideraciones |
|:--------|:----------------|
| **16px (favicon)** | Solo el símbolo, sin texto |
| **32px (app icon)** | Símbolo + 1 letra máximo |
| **150px (social avatar)** | Símbolo + texto corto |
| **300px (profile banner)** | Marca completa + tagline opcional |
| **1024px+ (web hero)** | Marca completa + espacio para breathing room |

### Clasificación Técnica de Logos

| Tipo | Descripción | Cuándo usarlo | Ejemplo |
|:-----|:------------|:--------------|:--------|
| **Logotipo** | Solo texto (tipografía estilizada) | Marca nueva, nombre fuerte, sin necesidad de símbolo | Google, Coca-Cola, Sony |
| **Isotipo** | Solo símbolo (sin texto) | Marca establecida con alto reconocimiento | Apple, Nike, Twitter/X |
| **Imagotipo** | Símbolo + texto separados | Máxima flexibilidad: pueden usarse juntos o separados | Adidas, Pepsi, Mastercard |
| **Isologo** | Símbolo + texto integrados (inseparables) | Cuando el símbolo no funciona sin el texto | Burger King, Starbucks, Puma |

**Recomendación:** Tener al menos 2 variantes (logotipo para aplicaciones formales, isotipo para contexts small/icon). La mayoría de las marcas necesitan los 4 tipos en su sistema de logo responsive.

---

## 3. Identidad Sensorial 360°

La marca no es solo visual. Debe orquestarse en todos los sentidos.

### Auditivo
- **Brand sound logo (mnemónico):** 2-5 segundos. Ej: el "bum" de Netflix, el "ta-dum" de THX
- **Voice brand:** Tono, ritmo, acento de la voz en anuncios/IVR
- **Music identity:** Género, tempo, instrumentación consistente

### Táctil
- **Textura de empaque/impresos:** Mate, brillante, rugoso, suave
- **Paper stock preferido:** Gramaje, acabado, sensación al tacto
- **Surface texture digital:** Grain, noise, paper texture en fondos

### Olfativo (brand scent)
- **Firma olfativa:** Fragancia única para espacios físicos o empaque
- **Asociación:** El olor debe ser coherente con el territorio de marca

---

## 4. Imperfectismo Curado (2026 Trend)

Como reacción a la perfección artificial de la IA, integrar imperfecciones intencionales:

| Técnica | Aplicación |
|:--------|:-----------|
| **Noise/grain overlay** | Fondos, imágenes hero, avatares |
| **Asimetría intencional** | Layouts, grids, márgenes |
| **Trazos manuales** | Ilustraciones, iconos, bordes |
| **Texturas orgánicas** | Paper, canvas, clay en backgrounds |
| **Tipografía variable** | Letter-spacing no uniforme, ligaduras |
| **Fotografía raw** | Sin filtros excesivos, luz natural |

### Regla
El imperfectismo debe sentirse **intencional y curado**, no descuidado. Cada desviación de la perfección debe tener un "por qué".

---

## Integración con el Orquestador

**Trigger words:** "psicología del color", "color psychology", "sensory branding", "identidad sensorial", "sensory identity", "forma", "shape", "área de protección", "exclusion zone", "adaptive identity", "imperfectismo", "brand scent", "sonido de marca"

**Flujo:** `strategy/brand-platform.md` → `strategy/sensory-identity.md` → `ui-ux-pro-max` (paleta) → `ui-design` (aplicación)
