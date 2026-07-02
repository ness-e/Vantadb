# Content Strategy — Estrategia de Contenido para Diseño

> El contenido es diseño. Cada palabra en una interfaz es una decisión de UX. Este documento establece cómo planificar, crear y gobernar contenido alineado con la estrategia de marca.

---

## Tabla de Contenidos

1. [Principios de Content Strategy](#1-principios-de-content-strategy)
2. [Auditoría de Contenido](#2-auditoría-de-contenido)
3. [Modelo de Contenido](#3-modelo-de-contenido)
4. [Voz y Tono por Canal](#4-voz-y-tono-por-canal)
5. [UX Writing System](#5-ux-writing-system)
6. [Content Governance](#6-content-governance)
7. [Integración con el Orquestador](#7-integración-con-el-orquestador)

---

## 1. Principios de Content Strategy

1. **El contenido no es relleno.** Cada bloque de texto debe responder a una necesidad del usuario o a un objetivo de negocio.
2. **Menos es más.** Eliminar la mitad de las palabras. Luego eliminar la mitad de lo que queda (Krug).
3. **Consistencia sobre creatividad.** Un tono consistente genera más confianza que uno "creativo" pero errático.
4. **El contenido es un producto.** Se versiona, se prueba, se itera. No es un "entregable de una sola vez".
5. **Accesibilidad lingüística.** Lectura nivel B1 (escuela secundaria). Sin jerga técnica innecesaria. Sin metáforas que no crucen culturas.

### Pirámide de contenido

```
                ╱╲
               ╱  ╲           Estrategia
              ╱────╲          (qué decir y por qué)
             ╱      ╲
            ╱────────╲        Modelo de contenido
           ╱          ╲       (estructura, tipos, taxonomía)
          ╱────────────╲
         ╱              ╲     Creación y curación
        ╱────────────────╲    (escribir, revisar, aprobar)
       ╱                  ╲
      ╱────────────────────╲  Distribución y governance
     ╱                      ╲ (canales, ciclos, responsables)
```

---

## 2. Auditoría de Contenido

### Inventario de contenido

| Propiedad | Descripción |
|:----------|:------------|
| **URL / Ruta** | Dónde vive el contenido |
| **Tipo** | Copy, microcopy, help, legal, blog, social |
| **Formato** | Texto, imagen, video, audio, interactive |
| **Audiencia** | Segmento de usuario objetivo |
| **Propósito** | Informar, persuadir, educar, convertir |
| **Estado** | Activo, desactualizado, duplicado, huérfano |
| **Dueño** | Responsable de mantenerlo |

### Matriz de calidad

| Dimensión | Escala 1-5 | Criterio |
|:----------|:----------:|:---------|
| Precisión | 1-5 | ¿La información es correcta y actual? |
| Consistencia | 1-5 | ¿Sigue la voz y tono de marca? |
| Claridad | 1-5 | ¿Se entiende en una lectura rápida? |
| Accesibilidad | 1-5 | ¿Cumple WCAG? ¿Lectura B1? |
| SEO | 1-5 | ¿Está optimizado para búsqueda? |
| Conversión | 1-5 | ¿Empuja al usuario a la acción deseada? |

---

## 3. Modelo de Contenido

### Tipos de contenido típicos

```yaml
page:
  name: string
  slug: string
  seo:
    title: string (max 60 chars)
    description: string (max 160 chars)
    og_image: url
  sections:
    - type: hero
      headline: string
      subheadline: string
      cta: string
    - type: feature_grid
      items:
        - icon: string
          title: string (max 40 chars)
          description: string (max 120 chars)
    - type: pricing
      plans:
        - name: string
          price: string
          features: string[]
          cta: string
```

### Taxonomía de contenido

```
/ ── landing page
├── product/
│   ├── features/
│   ├── pricing/
│   └── integrations/
├── resources/
│   ├── blog/
│   ├── docs/
│   └── case-studies/
├── company/
│   ├── about/
│   ├── press/
│   └── careers/
└── legal/
    ├── privacy/
    ├── terms/
    └── cookies/
```

---

## 4. Voz y Tono por Canal

| Canal | Voz | Tono | Longitud |
|:------|:----|:-----|:---------|
| **Web (landing)** | Confiable, directa | Aspiracional | Títulos ≤8 palabras. Párrafos ≤3 líneas |
| **App UI** | Útil, calmada | Neutral-informativo | Microcopy ≤15 palabras |
| **Email** | Personal, cercana | Conversacional | Asunto ≤50 chars. Body ≤150 palabras |
| **Social** | Auténtica, ágil | Canal-dependiente | X: ≤280 chars. LinkedIn: ≤1000 chars |
| **Docs** | Clara, precisa | Técnica-informativa | Oraciones ≤25 palabras |
| **Legal** | Transparente | Clara, sin legalese | Oraciones ≤30 palabras. Explicaciones en lenguaje ciudadano |

### Tabla de prohibiciones

| ❌ Prohibido | ✅ Alternativa |
|:-------------|:---------------|
| "Haz clic aquí" | "Crear cuenta" / "Empezar" |
| "Bienvenido a..." | Eliminar. El usuario ya está aquí |
| "Nuestro equipo..." | "El equipo de [marca]" o "Somos..." |
| Jerga técnica | Lenguaje del usuario |
| Adverbios (fácilmente, simplemente) | Demostrar con el diseño |
| Exclamaciones múltiples (!!) | Una o ninguna |

---

## 5. UX Writing System

### Estructura de microcopy

| Componente | Estado | Copy | Notas |
|:-----------|:-------|:-----|:------|
| **CTA** | Default | "Crear cuenta" | Verbo + objeto. Sin puntuación |
| | Hover | — | Sin cambio de copy |
| | Disabled | — | Tooltip: "Completa el formulario primero" |
| | Loading | "Creando cuenta..." | Verbo en gerundio + objeto |
| | Success | "Cuenta creada" | Participio + objeto |
| | Error | "Error al crear cuenta" | "Error al [acción]" + solución |
| **Input** | Label | "Correo electrónico" | Sustantivo. Sin puntuación |
| | Placeholder | "tu@correo.com" | Ejemplo real, no instrucción |
| | Error | "Ingresa un correo válido" | Qué pasó + qué hacer |
| | Success | ✓ | Icono + sin texto |
| **Empty state** | Sin datos | "Aún no hay proyectos" | Explicación + CTA |
| | Sin resultados | "No encontramos resultados para '[query]'" | Query entrecomillada + sugerencia |
| **404** | Error | "Página no encontrada" | Sin humor forzado. CTA a homepage |

---

## 6. Content Governance

### RACI de contenido

| Actividad | Content Strategist | Designer | Developer | PM | Legal |
|:----------|:------------------:|:--------:|:---------:|:--:|:-----:|
| Crear copy | R | C | I | A | I |
| Revisar tono | A | R | I | C | — |
| Implementar | I | C | R | C | — |
| Aprobar lanzamiento | C | C | I | A | R |
| Auditar calidad | R | I | — | A | — |

### Ciclo de vida del contenido

```
Crear → Revisar → Aprobar → Publicar → Monitorear → Iterar → (Archivar)
```

### Frecuencia de actualización

| Tipo | Frecuencia | Responsable |
|:-----|:-----------|:------------|
| Copy de UI | Por release | Product designer |
| Landing page | Trimestral | Content strategist |
| Blog | Semanal | Marketing |
| Docs | Por cambio de feature | Technical writer |
| Legal | Anual o por cambio regulatorio | Legal team |

---

## 7. Integración con el Orquestador

**Trigger words:** "content strategy", "content audit", "modelo de contenido", "UX writing", "microcopy", "voz y tono", "estrategia de contenido", "copy", "redacción", "contenido"

**Skills relacionadas:**
- `strategy/verbal-identity.md` — Voz y tono de marca (la base de la content strategy)
- `writing-guidelines` — Auditoría de prosa y compliance
- `designer-toolkit/ux-writing` — Microcopy system
- `strategy/brand-platform.md` — Alinear contenido con plataforma de marca
- `ai-seo` — Optimización de contenido para AI search

**Flujo:** `strategy/brand-platform.md` → `strategy/verbal-identity.md` → `strategy/content-strategy.md` → `writing-guidelines` (auditar) → `ai-seo` (optimizar)

