# Sonic & Kinetic Identity — Marca que se Escucha y se Mueve

> La identidad ya no es solo visual. En 2026, las marcas se construyen también con sonido y movimiento.
> **Sonic branding** para ser reconocido donde no hay pantalla. **Kinetic typography** para que el texto cuente una historia antes de leerse.

---

## PARTE 1: SONIC BRANDING (Identidad Sonora)

### 1.1 Concepto

Un **sonic logo** (logo sonoro) es una pieza de sonido corta y distintiva, normalmente de 2 a 5 notas y 1 a 5 segundos de duración, que identifica a una marca auditivamente. Funciona como el logo visual pero para el oído.

**Diferencia clave:**
| Concepto | Qué es | Ciclo de vida |
|:---------|:-------|:--------------|
| **Jingle** | Música publicitaria con letra cantada para una campaña | Temporal |
| **Sonic Logo** | Sonido abstracto y corto que identifica la marca | Permanente |

### 1.2 Touchpoints Sonoros

El sonic branding opera donde el visual no llega:
- Asistentes de voz (Alexa, Siri, Google)
- Podcasts y radio
- Sonidos de app (notificaciones, UI sounds)
- Terminales de pago
- Música en espera telefónica
- Eventos presenciales
- Publicidad en audio (Spotify, YouTube)
- Dispositivos IoT

### 1.3 Arquitectura de un Sistema Sonoro (2026 Trend)

Las marcas líderes ya no crean un solo logo sonoro estático — construyen **sistemas sónicos adaptativos**:

| Componente | Descripción | Ejemplo |
|:-----------|:------------|:--------|
| **Core Mnemonic** | 2-5 notas que son la "firma" irreducible | Netflix "ta-dum" |
| **Brand Track** | Versión extendida para publicidad | 15-30 segundos |
| **UI Sounds** | Micro-sonidos para interfaces | Click, confirmación, error |
| **Retail/Ambient** | Música de ambiente en espacios físicos | Tempo, instrumentación consistente |
| **Sonic Guidelines** | Reglas de tempo, instrumentación, tono y voz | Documentación del sistema |

### 1.4 Tendencias Sonic Branding 2026

| Tendencia | Descripción | Aplicación |
|:----------|:------------|:-----------|
| **Adaptive Sound Identity** | Sistema que se ajusta a contexto, plataforma y audiencia sin perder reconocibilidad | Modo activo, modo relax, modo enfoque |
| **Spatial Audio** | Audio inmersivo 3D para experiencias de marca | Eventos, retail, apps |
| **Audio Attribution** | Tracking de qué activos sonoros generan conversión | Medir recall por sonic logo |
| **B2B Sonic Branding** | Marcas enterprise adoptando identidad sonora | SAP, IBM, Microsoft |
| **Human Touch + AI** | IA acelera producción, músicos humanos aportan emoción y autenticidad | Composición híbrida |
| **Sonic Sustainability** | Sonidos que comunican valores ecológicos | Materiales reciclados en grabación |

### 1.5 Proceso de Creación

```
1. Estrategia → ¿Quién es la marca? ¿Qué promete? ¿Dónde se escuchará?
2. Audit → ¿Cómo suena la competencia? ¿Qué espacios sonoros están vacíos?
3. Composición → Encontrar el sonido de la marca (2-5 notas, campo o composición)
4. Prototipado → Testear en touchpoints reales (app, radio, evento, hold music)
5. Sistema → Entregar no solo el sting, sino su familia: brand track, UI sounds, guidelines
```

### 1.6 Medición

| Métrica | Cómo se mide |
|:--------|:-------------|
| **Recognition** | ¿La gente identifica la marca solo por el sonido? |
| **Attribution** | ¿El sonic logo hace que la publicidad funcione mejor? |
| **Consistency** | ¿El sistema suena coherente en todos los touchpoints? |

---

## PARTE 2: KINETIC TYPOGRAPHY (Tipografía en Movimiento)

### 2.1 Concepto

Tipografía cinética es texto que se mueve, anima y transforma en la interfaz. En 2026 ha pasado de ser un efecto de nicho a una herramienta mainstream de diseño web gracias a:

| Avance técnico | Impacto |
|:---------------|:--------|
| **CSS Scroll-Driven Animations API** | Animaciones vinculadas al scroll sin JavaScript |
| **Variable Fonts** | Un solo archivo anima weight, width, slant continuamente |
| **GSAP + ScrollTrigger** | Control preciso de animaciones scroll-driven |
| **Webflow / Framer / Spline** | No-code tools que hacen accesible la tipografía animada |

### 2.2 Patrones Comunes (2026)

| Patrón | Descripción | Implementación |
|:-------|:------------|:---------------|
| **Scroll Reveal** | Texto aparece letra por letra al scrollear | CSS Scroll-Driven Animations o GSAP |
| **Weight Morphing** | La fuente cambia de peso dinámicamente (light → bold) | Variable fonts + animación |
| **Split Text** | Palabras o caracteres individuales animan en secuencia | GSAP SplitText, Lottie |
| **Hover Kinetic** | Heading cambia weight/width al hacer hover | CSS + variable fonts |
| **Typewriter** | Texto se escribe solo en tiempo real | JS, GSAP |
| **Stagger Grid** | Palabras en grid aparecen desde el centro | Anime.js stagger |
| **Morphing Headline** | Frases se transforman unas en otras | GSAP, Framer Motion |

### 2.3 Técnico: Lo que Animar

```
✅ GPU-accelerated (siempre preferir):
   transform: translateX/Y, scale, rotate
   opacity
   clip-path

❌ Evitar (causa reflow/repaint):
   width, height, margin, padding
   color, background-color
   font-size, line-height
```

### 2.4 Accesibilidad

| Regla | Implementación |
|:------|:---------------|
| Respetar `prefers-reduced-motion` | `@media (prefers-reduced-motion: reduce)` → fade simple o estático |
| No auto-play infinito | El usuario debe poder pausar |
| Duración controlada | 200-400ms para micro-interacciones |
| Fallback legible | El texto debe ser legible sin animación |

### 2.5 Reglas de Uso

1. **Una intención por animación.** No animar todo — elegir qué texto merece movimiento
2. **La jerarquía primero.** Si la página funciona sin animación, está lista para añadirla
3. **Performance siempre.** Testear en dispositivo mid-range antes de producción
4. **Variable fonts = una request HTTP.** Preferir variable fonts sobre múltiples archivos
5. **Duración corta.** 200-400ms para efectos sutiles, 500-1000ms para storytelling

---

## Integración con el Orquestador

**Trigger words:** "sonic branding", "logo sonoro", "brand sound", "identidad sonora", "audio branding", "kinetic typography", "tipografía cinética", "motion typography", "animated text", "variable font animation", "scroll-driven typography"

**Flujo:** `strategy/sonic-kinetic-identity.md` → `strategy/brand-platform.md` (alinear con territorio) → CAPA 4 (motion) si kinetic typography, o producción de audio si sonic branding
