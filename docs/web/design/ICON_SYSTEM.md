# Icon System — Swiss Monoline

> Fuente: `design/DiseñoNuevo.md §11` | Estilo: Planos Técnicos

---

## Estilo

| Propiedad | Valor |
|:---|:---|
| Trazo | `1.5px` constante |
| Relleno | Sin relleno — solo contornos |
| Geometría | 90° exclusivamente |
| Color resting | `--steel` |
| Color hover/active | `--amber` |
| Tamaño base | `20×20px` (escala por contexto) |

## Prohibiciones

- ❌ Lucide, FontAwesome, Material Icons genéricos
- ❌ Iconos con relleno sólido
- ❌ Multi-color en un solo icono
- ❌ Stroke-width variable
- ❌ Curvas bezier suaves (solo 90°)

## Permitidos

| Fuente | Uso | Nota |
|:---|:---|:---|
| **Phosphor Bold** | Compatible con monoline | Preferir variante Bold |
| **Radix UI Icons** | Compatible con monoline | Trazo consistente |
| **SVG monoline custom** | Diagramas técnicos, benchmarks | Ideal para planos |

## Iconos del Sistema

### Core Engine (DiseñoNuevo §9.4)

| Icono | Descripción SVG |
|:---|:---|
| Engranaje | Círculo con dientes 90° |
| Grafo nodos | Círculos + líneas ortogonales |
| Documento + lupa | Rectángulo + círculo con línea |
| Disco + check | Círculo segmentado + marca |
| Puente | Dos rectángulos conectados |
| Flecha bidireccional | Línea con puntas opuestas |

### Benchmarks (DiseñoNuevo §9.2)

| Icono | Descripción |
|:---|:---|
| `↓` (faster) | Flecha abajo en `--amber` |
| `↑` (slower) | Flecha arriba en `--danger` |
| Barra horizontal | SVG rect con `--amber` fill |

### Ecosystem (DiseñoNuevo §9.6)

| Categoría | Icono |
|:---|:---|
| Frameworks | Cuadrícula 2×2 |
| LLM Providers | Nube con nodo |
| Deployment | Contenedor rectangular |

### Technical Diagrams (DiseñoNuevo §9.5)

| Elemento | Especificación |
|:---|:---|
| Líneas de cota | Flechas horizontales/verticales con medida |
| Capas | Rectángulos apilados con borde 1px |
| Flechas ortogonales | Líneas con giros 90° |
| Nodos de red | Puntos `ø 4px` con líneas de conexión |

## Guidelines de Implementación

```tsx
// Componente base para iconos monoline
const SwissIcon = ({ paths, size = 20, color = "var(--steel)" }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 20 20"
    fill="none"
    stroke={color}
    strokeWidth={1.5}
    strokeLinecap="square"
    strokeLinejoin="miter"
  >
    {paths.map((d, i) => <path key={i} d={d} />)}
  </svg>
);
```

- `strokeLinecap: "square"` — esquinas rectas
- `strokeLinejoin: "miter"` — ángulos 90° nítidos
- Transición de color: `150ms var(--ease-swiss)`
