# Logo Usage — VantaDB Brand

> Estándar: Swiss High-Contrast Minimal (Neon Precision)
> Documento maestro: `design/DiseñoNuevo.md`

---

## Primary Logo

El logo VantaDB consiste en un **torus negro (anillo)** + **esfera naranja (core)**.
Representación 3D wireframe en el hero (ver `design/DiseñoNuevo.md §7`).

### Versiones

| Versión | Descripción | Uso |
|:---|:---|:---|
| **Full 3D Wireframe** | Torus `#0a0a0a` + esfera `#ff5500` | Hero del index (Three.js) |
| **Flat SVG Mark** | Logo simplificado 2D monocromático | Nav, Footer, favicon |
| **Wordmark** | "VantaDB" en Space Grotesk 700 | Alternativa cuando el mark no aplica |

### Colores

| Elemento | Light Mode | Dark Mode |
|:---|:---|:---|
| Torus (anillo) | `#000000` | `#f0f0f0` |
| Core (esfera) | `#ff5500` | `#ff5500` |
| Wordmark | `#000000` | `#ffffff` |

### Clear Space

- Mínimo: **24px** (3 unidades de grid) alrededor del logo en todos los lados
- Proporción mark/wordmark: espacio de **16px** entre mark y texto

### Tamaños Mínimos

| Contexto | Mark mínimo | Wordmark mínimo |
|:---|:---|:---|
| Nav | 24×24px | 16px font-size |
| Footer | 20×20px | 14px font-size |
| Favicon | 16×16px | — |
| Hero 3D | 200×200px (canvas) | — |

---

## Prohibiciones

- ❌ No recolorear el core naranja (siempre `#ff5500`)
- ❌ No aplicar sombras, gradientes o glow al logo
- ❌ No rotar el torus más de 15° en reposo (solo animación 3D)
- ❌ No usar en fondos con patrón o fotografía
- ❌ No combinar con otros logotipos en el mismo contenedor
- ❌ No border-radius en contenedores del logo
- ❌ No escalar desproporcionadamente (mantener relación de aspecto 1:1 para el mark)

---

## Aplicaciones

### Nav (DiseñoNuevo §8.1)

```
┌──────────────────────────────────────────────┐
│ [⚫ Mark] VantaDB    Engine  Architecture ... │
└──────────────────────────────────────────────┘
```

- Mark + wordmark inline, Space Grotesk 700
- Altura: 64px bar, logo centrado verticalmente
- Color: `--foreground` (#000000)

### Footer (DiseñoNuevo §8.5)

```
┌──────────────────────────────────────────────┐
│ [⚫ Mark] VantaDB              © 2026 ...    │
└──────────────────────────────────────────────┘
```

- Mark + wordmark en bottom bar
- Color: `--block-dark-text` (#f0f0f0)
- Copyright en `--block-dark-muted` (#808080)

### Favicon

- SVG monoline del mark (torus + core)
- 16×16 y 32×32
- Sin wordmark
- Color naranja `#ff5500` sobre fondo transparente

---

## Archivos Fuente

| Archivo | Formato | Ubicación |
|:---|:---|:---|
| Logo 3D wireframe | Three.js | `src/components/ThreeLogo.tsx` |
| Logo SVG | `.svg` | `src/assets/logo.svg` |
| Favicon | `.png` / `.ico` | Raíz del proyecto |
