# DiseñoNuevo.md — Swiss High-Contrast Minimal (Neon Precision) para VantaDB

> Documento maestro de especificación visual, interactiva y técnica para el rediseño completo del sitio web de VantaDB.
> Fecha: 2026-06-23 | Versión: 2.0

---

## 0. Manifiesto de Diseño

**"La precisión es el lujo. El espacio vacío es la confianza. El neón es la señal, no la decoración."**

VantaDB es un motor de base de datos embebido escrito en Rust. Su interfaz web debe comunicar **velocidad mecánica**, **rigor ingenieril** y **transparencia técnica**. No vendemos promesas; mostramos datos, arquitectura y rendimiento real.

El estilo **Swiss High-Contrast Minimal (Neon Precision)** fusiona la disciplina de la Escuela Suiza de Diseño (Müller-Brockmann, Hofmann, Ruder) con la estética de interfaces de telemetría de alta precisión.

---

## 1. Principios Fundamentales del Swiss Design Aplicados

### 1.1 Del Swiss Clásico (1950-1970)
| Principio | Implementación Web |
|:---|:---|
| **Sistemas de rejilla (Grids)** | CSS Grid de 12 columnas con líneas visibles de 1px. Gap de 1px sólido `oklch(15% 0.008 265)` |
| **Tipografía sans-serif** | Space Grotesk (display), Outfit (body), JetBrains Mono (datos/código) |
| **Asimetría** | Títulos que arrancan en columna 3/12 o 5/12. Bloques de contenido desplazados del centro |
| **Jerarquía visual estricta** | Contraste de peso 100 (Hairline) vs 900 (Black) en la misma pantalla. Tamaños de `0.72rem` a `7.5rem` |
| **Alineación a la izquierda (Bandera)** | `text-align: left` por defecto. Nunca `justify`. Nunca centrado excepto en CTAs aislados |
| **Espacio negativo** | Macro-spacing de `96px`–`160px` entre secciones. Columnas enteras vacías intencionalmente |
| **Color funcional** | El naranja Safety Orange `#ff5500` solo para: estados activos, datos críticos, CTAs principales |
| **Sin decoración** | Cero ornamentos, cero bordes redondeados innecesarios, cero sombras difusas |
| **Enfoque universal** | Copys técnicos directos en inglés. Sin metáforas vacías ni buzzwords de marketing |
| **Geometría matemática** | Proporciones basadas en múltiplos de 8px. Ángulos de 90° y 45° exclusivamente |

### 1.2 De la Variante Neon Precision
| Principio | Implementación Web |
|:---|:---|
| **Contraste OLED absoluto** | Fondo warm paper `#f9f8f6` con bloques de negro puro `#000000`. Contraste >15:1 |
| **Acentos de neón** | Safety Orange `#ff5500` como único color cromático. Glow de radio corto, alta intensidad |
| **Grosor tipográfico extremo** | Display 700 + Label 600 (ALL CAPS 0.72rem). Contraste máximo de escala |
| **Líneas de rejilla visibles** | Bordes de 1px `var(--border)` dibujados explícitamente como elementos de diseño |
| **Microinteracciones precisas** | Duraciones 100ms-250ms. Easing lineal/cortante. Sin bounce ni elastic |
| **Densidad de información** | Paneles tipo telemetría con datos organizados en grid compacto |
| **Cero texturas** | Superficies planas digitales. Sin gradientes difusos, sin ruido excesivo, sin imitación de materiales |

---

## 2. Paleta de Color Definitiva

### 2.1 Colores Primarios
```
--background:       #f9f8f6          /* Warm paper (lienzo principal) */
--foreground:       #000000          /* Negro absoluto (texto principal) */
--amber:            #ff5500          /* Safety Orange (ÚNICO acento cromático) */
```

### 2.2 Superficies
```
--surface:          #ffffff          /* Tarjetas resting */
--surface-raised:   oklch(92% 0.003 85) /* Tarjetas hover */
--surface-glass:    rgba(249,248,246,0.85) /* Nav flotante + blur */
--deep-space:       oklch(96% 0.003 85) /* Fondo alternativo */
--void:             oklch(94% 0.004 85) /* Terminales y bloques de código */
```

### 2.3 Bloques Invertidos (Secciones oscuras)
```
--block-dark-bg:    #0a0a0a          /* Fondo negro OLED para secciones invertidas */
--block-dark-text:  #f0f0f0          /* Texto claro sobre fondo oscuro */
--block-dark-muted: #808080          /* Texto secundario sobre fondo oscuro */
--block-dark-border: rgba(255,255,255,0.08) /* Bordes sutiles en modo oscuro */
```

### 2.4 Estados y Señales
```
--amber-light:      #ff3300          /* Hover en elementos naranja */
--amber-soft:       #ff7733          /* Sub-indicadores activos */
--amber-dim:        rgba(255,85,0,0.08) /* Fondo sutil de elementos activos */
--success:          #00aa30          /* Validaciones */
--warn:             #dd9900          /* Advertencias */
--danger:           #cc1100          /* Errores */
```

### 2.5 Texto y Bordes
```
--muted:            oklch(40% 0.01 80)  /* Texto secundario */
--steel:            oklch(35% 0.01 240) /* Etiquetas y metadatos */
--border:           oklch(15% 0.008 265) /* Líneas divisorias de 1px */
--border-strong:    #000000             /* Bordes de acción principal */
--subtle:           oklch(88% 0.004 85) /* Guías de grilla secundarias */
```

### 2.6 Regla del 95/5
- **95%** de la pantalla: monocromática (negro, blancos, grises)
- **5%** máximo: naranja Safety Orange para señales activas
- El neón NO se usa para texto de lectura, solo para: hover/focus, datos en tiempo real, CTAs

---

## 3. Tipografía

### 3.1 Familias
| Rol | Familia | Uso |
|:---|:---|:---|
| **Display** | Space Grotesk | Títulos masivos, números gigantes, hero |
| **Body** | Outfit | Párrafos, descripciones, navegación |
| **Mono/Label** | JetBrains Mono | Código, datos tabulares, etiquetas ALL CAPS |

### 3.2 Escala Tipográfica
| Token | Tamaño | Peso | Letter-spacing | Line-height |
|:---|:---|:---|:---|:---|
| `--text-hero` | `clamp(3.8rem, 8vw, 7.5rem)` | 700 | `-0.05em` | 0.95 |
| `--text-display` | `clamp(2.2rem, 5vw, 4rem)` | 700 | `-0.04em` | 1.05 |
| `--text-title` | `clamp(1.3rem, 2.2vw, 1.7rem)` | 600 | `-0.02em` | 1.2 |
| `--text-body` | `1.05rem` | 400 | `-0.01em` | 1.65 |
| `--text-label` | `0.72rem` | 600 | `0.14em` | 1.2 (ALL CAPS) |
| `--text-code` | `0.88rem` | 400 | `normal` | 1.5 |

### 3.3 Reglas Tipográficas Estrictas
- **Nunca** usar text-align: center en bloques de contenido (solo en CTAs aislados)
- **Nunca** justificar texto
- Etiquetas de datos en `font-variant-numeric: tabular-nums` para estabilidad numérica
- Textos largos en `--muted` (#666), NUNCA en naranja
- Títulos masivos pueden usar `mix-blend-mode: difference` para efecto de inversión

---

## 4. Sistema de Grid

### 4.1 Grid Principal
```css
.swiss-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 1px;
  max-width: 1200px;
  margin: 0 auto;
}
```

### 4.2 Líneas Visibles
Las líneas de la cuadrícula son elementos de diseño, no guías invisibles:
```css
.grid-line-v { 
  width: 1px; 
  background: var(--border); 
  position: absolute; 
}
.grid-line-h { 
  height: 1px; 
  background: var(--subtle); 
}
```

### 4.3 Asimetría Intencional
- Hero: título en columnas 1-8, espacio vacío en 9-12
- Secciones de datos: grid asimétrico `2fr 1fr` o `1fr 2fr`
- Labels/indices en columna estrecha izquierda, contenido en columna ancha derecha

---

## 5. Bordes, Elevación y Profundidad

### 5.1 Sin Sombras
- `box-shadow: none` en todo el sistema
- La profundidad se logra exclusivamente con:
  - Cambio de color de fondo (resting → hover)
  - Cambio de borde de `--border` a `--border-strong`
  - Contraste de secciones claras/oscuras

### 5.2 Border Radius
```
--radius-sm: 0px   /* Botones, inputs, tarjetas */
--radius-md: 2px   /* Mínimo suavizado */
--radius-lg: 4px   /* Terminales */
--radius-xl: 6px   /* Máximo del sistema */
```
**Regla:** Nunca `border-radius > 6px`. Los botones son rectangulares.

---

## 6. Motion & Animación

### 6.1 Principios
- **Rápido y mecánico**: 100ms–250ms máximo
- **Easing cortante**: `cubic-bezier(0.25, 1, 0.5, 1)` o `cubic-bezier(0.2, 0.8, 0.2, 1)`
- **Sin bounce, sin elastic, sin spring suave**
- **Cortes editoriales**: Transiciones secas y directas entre estados

### 6.2 Scroll Animations (GSAP + ScrollTrigger)
- Revelado por máscara tipográfica: `clip-path` o `overflow: hidden` con `translateY`
- Expansión desde líneas de cuadrícula: elementos que crecen desde bordes de 1px
- Contadores numéricos que avanzan rápidamente al entrar en viewport
- Líneas de grid que se "dibujan" con `stroke-dashoffset` animado

### 6.3 Microinteracciones
- Hover en botones: transición de fondo en 150ms, sin transform
- Hover en tarjetas: borde cambia de `--border` a `--border-strong` en 100ms
- Links: underline que se expande de izquierda a derecha en 200ms
- Labels `[01]`, `[OK]` que cambian de `--steel` a `--amber` al hover del padre

### 6.4 Respeto a `prefers-reduced-motion`
- Si `reduce`: todas las animaciones se desactivan, elementos visibles inmediatamente
- GSAP `matchMedia` para manejar este caso

---

## 7. Componentes del Sistema

### 7.1 Navegación (Nav)
- Barra fija superior, fondo `--surface-glass` con `backdrop-filter: blur(12px)`
- Altura: 64px
- Borde inferior: `1px solid var(--border)`
- Logo izquierda, links centro, CTA derecha
- Links en `--text-label` (ALL CAPS, 0.72rem, 0.14em tracking)
- Hover: color cambia de `--steel` a `--foreground` en 100ms
- Mobile: hamburger → panel lateral con fondo `--surface-overlay`

### 7.2 Footer
- Fondo: `#0a0a0a` (bloque invertido negro OLED)
- Grid de 4 columnas con links en `--block-dark-muted`
- Hover: links cambian a `#ffffff`
- Líneas divisorias de 1px `rgba(255,255,255,0.08)`
- Copyright en `--text-label` con `--block-dark-muted`

### 7.3 Botones
- **Primary**: fondo `--amber`, texto `#ffffff`, border-radius `0px`, padding `10px 24px`
  - Hover: fondo `#000000`, texto `#ffffff`
- **Ghost**: fondo transparente, borde `1px solid var(--border)`, texto `--foreground`
  - Hover: fondo `--border`, texto `#ffffff`
- **Link**: texto `--amber`, sin fondo, underline animado de izquierda a derecha

### 7.4 Tarjetas / Bloques
- Fondo `--surface`, borde `1px solid var(--border)`
- Padding `24px`
- Hover: borde cambia a `--border-strong`
- Index label en esquina superior izquierda: `[01]` en `--text-label`
- Sin sombras, sin border-radius > 4px

### 7.5 Terminal / Code Block
- Fondo `--void`
- Borde `1px solid var(--border)`
- Header con 3 dots falsos en `--subtle` y título en `--text-label`
- Código en JetBrains Mono
- Syntax highlighting mínimo: keywords en `--foreground`, strings en `--amber`, comments en `--muted`

---

## 8. Diseño del Index (Landing Page)

### 8.1 HERO — "Typographic Grid Hero"
**Concepto:** Hero sin estadísticas. Tipografía masiva asimétrica con grid de hairlines visible.

**Layout:**
- Fondo: bloques alternados de `--background` y `#0a0a0a`
- Grid de 12 columnas con líneas verticales visibles de 1px
- Título "VantaDB" en `--text-hero` (7.5rem), peso 700, alineado a la izquierda columnas 1-8
- Subtítulo técnico debajo: "Embedded cognitive memory for AI agents" en `--text-title`, peso 400
- Label superior: `[RUST-NATIVE] [IN-PROCESS] [ZERO-SERVERS]` en `--text-label` con `--amber`
- CTA: dos botones rectangulares — "pip install vantadb" (primary) + "Read Docs" (ghost)
- Columnas 9-12: vacías o con un elemento 3D wireframe minimalista (cubo/monolito de Three.js)

**Interacción:**
- Las líneas verticales del grid se dibujan con `stroke-dashoffset` de arriba a abajo al cargar
- El título se revela por máscara tipográfica desde abajo con `clip-path`
- Labels parpadean una vez en naranja al cargar y luego quedan estáticos
- Objeto 3D (si se incluye) rota ortogonalmente con el mouse (no suave, sino en saltos de 15°)

**Lo que NO tiene:**
- Sin estadísticas de downloads, stars, etc.
- Sin partículas flotantes
- Sin gradientes difusos
- Sin animación de typewriter

### 8.2 COMPARATIVA — "Swiss Benchmark Grid"
**Concepto:** Grilla Bento asimétrica que compara VantaDB vs arquitecturas cliente-servidor.

**Layout:**
- Grid Bento de tamaños variados (2x2, 1x1, 3x1) con bordes de 1px
- Columna izquierda estrecha: label vertical `[VANTADB]` vs `[TRADITIONAL]`
- Cada celda contiene UNA métrica:
  - Número gigante (Space Grotesk 700, tamaño display)
  - Unidad pequeña debajo (JetBrains Mono label)
  - Indicador de diferencia: flecha `↓` en `--amber` para "mejor" o `↑` en `--danger` para "peor"

**Métricas a mostrar:**
| Métrica | VantaDB | Traditional | Indicador |
|:---|:---|:---|:---|
| Query Latency (p99) | 0.8ms | 12ms | ↓ 15x faster |
| Memory Overhead | 2MB | 180MB | ↓ 90x less |
| Setup Time | 1 line | 45 min | ↓ Instant |
| Dependencies | 0 | 12+ | ↓ Zero |
| Crash Recovery | WAL | Manual | Automatic |
| Search Type | Hybrid (BM25+HNSW) | Single | Full-spectrum |

**Interacción:**
- Al hacer scroll, las celdas se expanden desde las líneas de la cuadrícula (no fade-in)
- Los números hacen count-up rápido (200ms) al entrar en viewport
- Hover en celda: borde cambia a `--amber`, label index `[01]` se ilumina

### 8.3 ESTADÍSTICAS — ELIMINADA
Se eliminan completamente: downloads, github stars, uptime, license, p99 query latency, max vector dims, bm25 recall, external dependencies.

### 8.4 QUICKSTART — "Precision Terminal"
**Concepto:** Terminal + pasos animados 01-04, rediseñados al estilo Swiss.

**Layout:**
- Grid 2 columnas: izquierda (4col) = lista de pasos, derecha (8col) = terminal
- Pasos numerados `[01]` `[02]` `[03]` `[04]` en `--text-label`
- Paso activo: número en `--amber`, borde izquierdo de 2px `--amber`
- Terminal: fondo `--void`, borde `1px solid var(--border)`, sin sombras

**Interacción:**
- Animación de escritura de código en la terminal (typewriter a velocidad rápida)
- Al completar un paso, el siguiente se activa con transición de 150ms
- Labels de paso cambian de `--steel` a `--foreground` al activarse
- Output del código aparece instantáneamente (no fade) con borde izquierdo `--amber`

### 8.5 CORE ENGINE — "Exploded Architecture"
**Concepto:** Las ventajas del core engine reveladas con scroll, estilo diagrama de patente.

**Layout:**
- Sección con fondo invertido `#0a0a0a` (bloque oscuro OLED)
- Grid de 3 columnas con bloques de features
- Cada bloque: icono monoline (contorno de 1px naranja), título en blanco, descripción en `--block-dark-muted`
- Centro: diagrama de arquitectura simplificado con líneas ortogonales y labels técnicos

**Features a mostrar:**
1. Rust Core → velocidad y seguridad de memoria
2. HNSW Index → búsqueda vectorial de alta dimensión
3. BM25 Engine → búsqueda textual full-spectrum
4. WAL Durability → crash-safe, zero data loss
5. PyO3 Bridge → Python bindings nativos
6. Zero-Copy Serde → serialización sin overhead

**Interacción (GSAP ScrollTrigger):**
- Pin de la sección durante el scroll
- Cada feature se revela secuencialmente al scrollear
- Líneas de conexión se dibujan con `stroke-dashoffset` animado
- Labels técnicos aparecen con clip-path desde la izquierda
- Al completar todas las features, la sección se des-pinea

### 8.6 ARCHITECTURE — "Blueprint Cross-Section"
**Concepto:** Rediseño completo como diagrama de corte transversal tipo plano industrial.

**Layout:**
- Diagrama SVG de capas apiladas (Python App → PyO3 Bridge → Rust Core → Storage)
- Cada capa: rectángulo con borde de 1px, relleno semi-transparente
- Labels con líneas de cota (flechas de medición) y coordenadas
- Tipografía monoespaciada para todas las etiquetas técnicas

**Interacción:**
- Al hacer scroll, las capas se separan verticalmente (exploded view)
- Las flechas de flujo de datos se animan con `stroke-dasharray`
- Hover en capa: se resalta con borde `--amber` y las demás se atenúan a `opacity: 0.3`

### 8.7 ECOSYSTEM — "Integration Matrix"
**Concepto:** Grid limpio de integraciones con iconos monoline.

**Layout:**
- Grid de 4x3 con bordes de 1px
- Cada celda: icono SVG monoline (contorno de 1px, sin relleno) + nombre en `--text-label`
- Categorías: `[FRAMEWORKS]` `[VECTOR-STORES]` `[LLM-PROVIDERS]` `[DEPLOYMENT]`

**Interacción:**
- Hover en celda: icono cambia de `--steel` a `--amber`, fondo a `--amber-dim`
- Al hacer scroll, las celdas aparecen en stagger desde las líneas del grid

### 8.8 USE CASES — "Case Study Cards"
**Concepto:** Tarjetas horizontales con diseño editorial.

**Layout:**
- Stack vertical de tarjetas horizontales (full-width)
- Cada tarjeta: grid `3fr 9fr` → índice numérico grande + contenido
- Número del caso en tamaño display `--text-display` en `--subtle`
- Título del caso + descripción + label de industria

**Interacción:**
- Hover: el número del caso cambia de `--subtle` a `--amber`
- Borde superior de 1px `--border` separa cada tarjeta
- Click expande para mostrar más detalle (optional)

### 8.9 ÚLTIMA SECCIÓN — "CTA Monolith"
**Opciones propuestas:**

**Opción A: "The Monolith"**
- Bloque negro OLED full-width con un solo texto centrado masivo:
  `"pip install vantadb"` en `--text-hero`, peso 700, color `#ffffff`
- Debajo: `"Zero servers. One line. Infinite context."` en `--text-body`, color `--block-dark-muted`
- Un botón primary centrado: "Get Started" en naranja

**Opción B: "Swiss Grid CTA"**
- Grid 2 columnas: izquierda = estadística impactante ("0.8ms p99 latency"), derecha = CTA + descripción breve
- Fondo `--deep-space` con bordes visibles

**Opción C: "Terminal Echo"**
- Terminal completa que muestra el output de `pip install vantadb` + `import vantadb` + resultado exitoso
- Debajo: link a docs y GitHub

---

## 9. Diseño de Subpáginas

### 9.1 Patrón Común de Subpágina
Todas las subpáginas siguen este esqueleto:
1. **Hero compacto**: título masivo asimétrico + breadcrumb label + descripción
2. **Secciones de contenido**: alternando fondo claro/oscuro con grid visible
3. **Diagrams/Illustrations**: SVG monoline con etiquetas técnicas
4. **CTA bottom**: bloque negro con comando de terminal

### 9.2 /engine
- Hero: "The Rust Core" con diagrama de capas del motor
- Secciones: HNSW, BM25, WAL, PyO3, cada una con benchmark real
- Scroll animations: GSAP ScrollTrigger con pin para revelar capas
- Fondo alternado: warm paper → OLED black → warm paper

### 9.3 /architecture
- Hero: "Architecture" con diagrama de corte transversal
- Sección principal: SVG interactivo de la arquitectura completa
- Secciones secundarias: detalle de cada capa con código ejemplo

### 9.4 /integrations
- Grid matrix de integraciones con iconos monoline
- Categorías con labels `[CATEGORY]`
- Cada integración: card con código de ejemplo en terminal

### 9.5 /use-cases
- Lista editorial de casos de uso con tarjetas horizontales
- Cada caso: número display + descripción + stack técnico

### 9.6 /cost
- Comparativa de costos: grid Bento con VantaDB vs alternativas
- Números gigantes para precios/savings
- Gráfico de barras minimalista (SVG)

### 9.7 /latency
- Benchmarks de latencia: gráficos de barras horizontales
- Tabla de métricas p50/p95/p99 en grid monoespaciado
- Comparativa con competidores

### 9.8 /storage
- Diagramas de almacenamiento WAL + HNSW + BM25
- Métricas de compresión y overhead
- Explicación visual del crash recovery

### 9.9 /config
- Referencia de configuración en formato de terminal
- Cada opción: nombre + tipo + default + descripción
- Estilo de documentación técnica

### 9.10 /maint
- Guía de mantenimiento con pasos numerados
- Diagramas de flujo de operaciones

### 9.11 /solutions/ai-agents, /solutions/local-rag, /solutions/ai-ide-tooling
- Hero específico por solución
- Diagrama de arquitectura de la solución
- Features grid + código ejemplo + CTA

### 9.12 /docs
- Layout de documentación: sidebar izquierda + contenido derecho
- Sidebar con navegación de secciones
- Código con syntax highlighting mínimo

### 9.13 /pricing
- Grid de planes con borde de 1px
- Plan destacado: borde `--amber`
- Números de precio en tamaño display
- Feature comparison table con checks/crosses

### 9.14 /about
- Información de empresa con diseño editorial
- Grid de equipo/valores
- Timeline de hitos

### 9.15 /about/roadmap → ELIMINADA
- Archivo `roadmap.tsx` será eliminado
- Links al roadmap serán removidos de la navegación

### 9.16 /blog
- Lista de posts con diseño editorial
- Tarjetas con fecha + categoría + título
- Grid de 2 columnas en desktop

### 9.17 /about/community
- Links a comunidad y recursos
- Grid de canales (Discord, GitHub, etc.)

### 9.18 /about/contact
- Formulario minimalista con campos rectangulares
- Labels en `--text-label`
- Botón submit en naranja

---

## 10. Iconografía — "Planos Técnicos"

### 10.1 Estilo
- **Monoline**: grosor de trazo constante (1.5px)
- **Sin relleno**: solo contornos
- **Color**: `--steel` en reposo, `--amber` en hover/activo
- **Geometría**: ángulos de 90° y 45° exclusivamente
- **Labels**: texto diminuto monoespaciado junto a las ilustraciones

### 10.2 Diagramas
- Flechas ortogonales (nunca curvas)
- Líneas de cota con medidas
- Coordenadas y labels técnicos en las esquinas
- Estilo de patente industrial / manual de arquitectura de software

---

## 11. Elemento 3D — "The Monolith" (Opcional)

### 11.1 Concepto
Un cubo o monolito wireframe minimalista que representa la base de datos embebida.

### 11.2 Especificaciones
- **Estilo**: wireframe con líneas de 1px en `--border` o `--amber`
- **Material**: sin relleno sólido, solo bordes (MeshBasicMaterial con wireframe: true)
- **Interacción**: rotación ortogonal vinculada al mouse (saltos de 15°, no suave)
- **Scroll**: al avanzar, el cubo se separa en capas (exploded view) revelando la arquitectura interna
- **Rendimiento**: geometría mínima (<1000 polígonos), sin post-processing pesado

---

## 12. Contraste Invertido por Secciones

### 12.1 Patrón de alternancia
```
[Warm Paper] Hero
[OLED Black] Core Engine
[Warm Paper] Architecture
[OLED Black] CTA Final / Footer
```

### 12.2 Reglas de inversión
- En secciones oscuras: texto en `#f0f0f0`, muted en `#808080`
- Bordes: `rgba(255,255,255,0.08)`
- El naranja mantiene su valor `#ff5500` en ambos modos
- Los botones primary mantienen su apariencia naranja
- Los botones ghost invierten: borde blanco, texto blanco

---

## 13. Anti-Slop Checklist

Cada componente DEBE pasar esta verificación antes de ser aceptado:

- [ ] ¿`border-radius` ≤ 6px? (Nunca > 6px)
- [ ] ¿Sin sombras difusas? (`box-shadow: none`)
- [ ] ¿Sin gradientes decorativos?
- [ ] ¿Naranja usado SOLO para señales activas/CTAs?
- [ ] ¿Texto alineado a la izquierda? (Nunca centrado excepto CTAs aislados)
- [ ] ¿Tipografía del sistema? (Space Grotesk / Outfit / JetBrains Mono)
- [ ] ¿Bordes de 1px presentes?
- [ ] ¿Animaciones ≤ 250ms?
- [ ] ¿Sin ilustraciones 3D de plástico brillante?
- [ ] ¿Sin copys genéricos de marketing?
- [ ] ¿Espaciado macro entre secciones (≥ 96px)?
- [ ] ¿Grid asimétrico?
- [ ] ¿`prefers-reduced-motion` respetado?
- [ ] ¿`font-variant-numeric: tabular-nums` en datos numéricos?

---

## 14. Stack Tecnológico

| Capa | Tecnología |
|:---|:---|
| Framework | React 19 + TypeScript 5 |
| Routing | TanStack Router (file-based) |
| Bundler | Vite 8 + Rolldown |
| CSS | Tailwind CSS v4 + CSS custom properties |
| Animaciones | GSAP + ScrollTrigger |
| 3D (opcional) | Three.js (wireframe solo) |
| Fonts | Google Fonts: Space Grotesk, Outfit, JetBrains Mono |
| Hosting | Vercel SPA |

---

## 15. Skills y Herramientas a Utilizar

| Skill | Uso |
|:---|:---|
| `vanta-design-orchestrator` | Validación de identidad visual y routing de skills |
| `design-taste-frontend` | Anti-slop frontend, evitar estética genérica |
| `high-end-visual-design` | Fonts, spacing, shadows, card structures |
| `emil-design-eng` | Micro-interacciones y polish de UI |
| `emilkowalski-motion` | Motion design con restraint |
| `industrial-brutalist-ui` | Grids rígidos, escala tipográfica extrema |
| `minimalist-ui` | Composición editorial limpia |
| `gpt-taste` | GSAP ScrollTriggers, bento grids, editorial typography |
| `awesome-claude-design` | Design systems y anti-slop |
| `frontend-design` | Componentes production-grade |
| `color-expert` | Paleta OKLCH, contraste WCAG |
| `plan-design-review` | Gate de calidad antes de merge |
| `design-review` | Auditoría visual con before/after |
| `impeccable-design-polish` | Polish final antes de ship |
| `threejs` | Wireframe 3D del monolito (si se incluye) |
