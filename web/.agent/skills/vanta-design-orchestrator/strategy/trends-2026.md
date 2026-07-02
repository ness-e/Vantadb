# Trends 2026 — Tendencias de Diseño para el Futuro Cercano

> Un orquestador de diseño debe estar alineado con las demandas estéticas y funcionales de 2026.
> Esta skill cubre las tendencias que definen el diseño contemporáneo y cómo aplicarlas con intención estratégica.

---

## 1. Identidades Adaptativas y Variables

El diseño ya no es estático. Una marca debe funcionar en 16px (favicon), 150px (avatar), 1024px (web) y pantallas plegables/AR.

### Principios de Diseño Adaptativo

| Principio | Aplicación |
|:----------|:-----------|
| **Modularidad** | Sistema de componentes que se reordenan según el viewport |
| **Jerarquía variable** | Elementos cambian de importancia según el contexto |
| **Símbolo + texto** | El símbolo funciona solo en sizes pequeños; texto aparece en sizes grandes |
| **Color Contextual** | Paleta que se adapta al fondo (modo oscuro/light, alto contraste) |

### Formatos Específicos
```
Favicon (16×16)    → Solo símbolo, 1 color, sin texto
App icon (1024×1024) → Símbolo + gradiente/textura
Social avatar (400×400) → Símbolo + 1-2 letras máximo
Hero web (1920×1080) → Marca completa + tagline
AR/VAR (variable)   → Versión 3D del símbolo
```

---

## 2. Ecodiseño Visual

La sostenibilidad visual no es solo un trend — es una responsabilidad. El diseño puede reducir el impacto ambiental.

### Paletas Eco-Eficientes

| Técnica | Ahorro estimado | Aplicación |
|:--------|:----------------|:-----------|
| **Dark mode por defecto** | -30-40% energía en OLED/AMOLED | Apps mobile, dashboards |
| **Colores desaturados** | Menos tinta en impresión | Folletos, empaque |
| **Fondos blancos/light** | Menos tinta que fondos oscuros | Impresión tradicional |
| **Evitar imágenes pesadas** | Menos ancho de banda = menos energía | Web, mobile |

### Pautas de Ecodiseño
- **Digital:** Optimizar assets (WebP, AVIF, lazy loading, compressión)
- **Impresión:** Preferir CMYK estándar sobre Pantone (menos placas), usar papeles reciclados
- **Packaging:** Reducir material, tintas vegetales, diseño monocolor
- **Dark mode:** No solo invertir — rediseñar con elevación por luminosidad

---

## 3. Imperfectismo Curado

Como reacción a la perfección artificial de la IA, las marcas buscan imperfección intencional.

### Técnicas de Imperfectismo

| Técnica | Cómo aplicarlo | Cuándo no usarlo |
|:--------|:---------------|:-----------------|
| **Grain/Noise overlay** | Capa sutil sobre fondos e imágenes | UI crítica (textos, botones) |
| **Asimetría** | Layouts con pesos visuales desiguales | Dashboards, data tables |
| **Texturas manuales** | Paper, tiza, acuarela en backgrounds | Productos digitales high-tech |
| **Tipografía imperfecta** | Letter-spacing variable, ligaduras | Body text, legal copy |
| **Fotografía raw** | Sin filtros, luz natural, sin retoque excesivo | Beauty, fashion (depende) |
| **Bordes irregulares** | Recortes no perfectos, formas orgánicas | Marcos, cards, secciones hero |

### Regla del Imperfectismo
La imperfección debe ser **intencional y consistente**. No es descuido — es decisión estética. Cada desviación de la perfección debe tener un propósito que refuerce el territorio de marca.

---

## 4. Prompt Engineering como Estrategia (No solo Ejecución)

El diseñador/orquestador debe usar IA como estratega, no solo como ejecutor.

### Niveles de Prompt
| Nivel | Descripción | Ejemplo |
|:------|:------------|:--------|
| **Táctico (N1)** | Pide un output específico | "Genera un logo azul para fintech" |
| **Estratégico (N2)** | Pide opciones basadas en contexto | "Sugiere 3 direcciones visuales para una fintech que quiere transmitir confianza y modernidad" |
| **Filosófico (N3)** | Pide que la IA proponga la estrategia | "Analiza el mercado fintech para jóvenes y proponme una filosofía de producto que los diferencie" |

### Framework de Prompt para Diseñadores
```
OBJETIVO:   [Qué necesito]
ESTILO:     [Referencia estética, designer, movimiento]
CONTEXTO:   [Mercado, audiencia, competencia, canal]
RESTRICCIONES: [Presupuesto, tech stack, deadline, skills del equipo]
FORMATO:    [Output esperado: texto, spec, código, imagen]
```

### Cerrando Brechas (Prompt Gap)

| Brecha | Problema | Solución | Ejemplo concreto |
|:-------|:---------|:---------|:-----------------|
| **Temporal** | La IA usa estilos de 2-3 temporadas atrás. "Moderno" para la IA suele ser 2022-2023 | Especificar año exacto + referencias contemporáneas. Incluir designers activos como referencia | ❌ "Diseño moderno" → ✅ "Estética UI 2026: bento grids, kinetic typography con variable fonts, glassmorphism sutil, dark mode con elevación por luminosidad" |
| **Cultural** | Diseño genérico global sin contexto local | Contextualizar mercado: país, comportamientos de usuario, referentes locales,禁忌 culturales | ❌ "Diseño limpio" → ✅ "Diseño para mercado japonés: mucho whitespace, tipografía vertical opcional, colores inspirados en temporadas (Wabi-sabi), evitar rojo brillante en CTAs (asociado a peligro en contexto cultural)" |
| **Abstracción** | Términos vagos como "intuitivo", "limpio", "profesional" que no son medibles | Traducir a comportamientos observables y métricas concretas. Definir el "cómo se mide" antes de pedir el "qué" | ❌ "Hazlo más intuitivo" → ✅ "El usuario debe completar el registro en <2 minutos, sin errores, sin consultar ayuda. La tasa de abandono debe ser <15%. El tiempo entre 'abre la app' y 'primer valor entregado' debe ser <60 segundos" |
| **Técnica** | La IA asume tecnología/stack incorrecto o desactualizado | Especificar stack técnico exacto, versión de librerías, constraints de rendimiento | ❌ "Animación suave" → ✅ "Animación con motion.dev v12, GPU-accelerated (transform + opacity), duración 200-300ms, spring easing, fallback prefers-reduced-motion a fade estático" |
| **Presupuestal** | La IA propone soluciones que no caben en el presupuesto o timeline | Incluir constraints de recursos: equipo, tiempo, presupuesto, habilidades disponibles | ❌ "Sistema de diseño completo" → ✅ "Design system mínimo: 20 componentes core, tokens base, 1 tema (light). 2 diseñadores, 3 semanas. Priorizar: colores, tipografía, botones, forms, navegación" |

---

## 5. Gobernanza de IA en Diseño

### Lo que la IA puede hacer
- Generar 50 variaciones de logo en segundos
- Sugerir paletas basadas en teoría del color
- Escribir microcopy para 20 estados de un componente
- Auditar contraste y accesibilidad automáticamente
- Generar código de componentes desde especificaciones

### Lo que SOLO el humano debe hacer
- Decidir la dirección estratégica de marca
- Curar entre opciones generadas (elegir la que tiene más valor emocional)
- Aprobar el output final antes de producción
- Garantizar que la marca mantenga su propósito y ADN
- Tomar decisiones basadas en contexto cultural y de mercado

### Regla de protección de marca con IA
> La IA genera — el humano **selecciona, refina y da significado**.
> Sin un curador humano, el diseño de IA es ruido visual sin alma.

---

## Integración con el Orquestador

**Trigger words:** "tendencias 2026", "trends 2026", "adaptive identity", "identidad adaptativa", "ecodiseño", "eco-design", "imperfectismo", "prompt engineering", "prompt strategy", "IA en diseño", "AI governance", "curador humano"

**Flujo:** `strategy/trends-2026.md` → `ui-ux-pro-max` (paleta) → `awesome-claude-design` (anti-slop check) → Fase 3
