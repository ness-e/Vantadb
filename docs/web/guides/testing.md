# Testing en el Frontend Web de VantaDB

## 1. Estado Actual

El frontend web opera con **cero testing infrastructure**. No existe un solo archivo de test (ni `*.test.ts`, `*.test.tsx`, `*.spec.ts`), no hay directorios `__tests__/`, no hay dependencias de testing en `package.json`, y no hay script `test` en los scripts del proyecto.

Las únicas protecciones existentes son parciales:
- TypeScript tiene `noImplicitAny: false` e `ignoreBuildErrors: true` — compila aunque haya errores de tipos.
- ESLint corre con `eslint-config-next` pero **34 reglas están desactivadas**, reduciendo severamente su efectividad.
- Existe una auditoría manual (`docs/web/QA.md`) con 24 issues de calidad y un checklist de pre-merge (`docs/web/standards/review-checklist.md`) de 22 items.

El pipeline CI/CD existe (`ci-web-11.yml`) pero solo corre lint + typecheck + build — no hay tests automatizados. No hay gate de tests antes de deploy.

Esto significa que cualquier refactor, cambio de dependencias, o modificación de lógica compartida puede romper funcionalidad sin que nadie lo sepa hasta que alguien navega manualmente a la página afectada.

---

## 2. Qué Testear y Prioridades

Las prioridades están ordenadas por impacto/riesgo. No tiene sentido escribir tests de snapshot para todo antes de tener cobertura en lógica pura.

### P0 — Unit Tests para Lógica Pura (recuperación inmediata)

Son los tests más baratos de escribir y los que más valor dan: lógica sin DOM, sin rendering, puros datos de entrada/salida.

| Archivo | Qué testear |
|---------|-------------|
| `src/lib/vanta-data.ts` (1109 líneas, 25 exports) | Que cada función exportada retorne la shape correcta. Que arrays tengan los elementos esperados. Que valores por defecto sean correctos. |
| `src/lib/dictionaries.ts` (2993 líneas) | Que los objetos ES y EN tengan exactamente las mismas keys (detectar missing translations). Que valores anidados sean del tipo esperado. |
| `src/app/opengraph-image.tsx` | Que el componente `ImageResponse` se genere sin errores para distintos parámetros. |

**Ejemplo concreto para dictionaries.ts:**
```ts
import { describe, it, expect } from 'vitest';
import { getDictionary } from '@/lib/dictionaries';

const es = getDictionary('es');
const en = getDictionary('en');

it('ES y EN tienen las mismas keys', () => {
  const esKeys = Object.keys(es).sort();
  const enKeys = Object.keys(en).sort();
  expect(esKeys).toEqual(enKeys);
});
```

### P1 — Component Tests (impacto medio)

Una vez que la lógica pura está cubierta, pasar a componentes. El helper `tt()` se usa en casi todos los componentes — hay que mockear el dictionary para aislar los tests.

| Componente | Qué testear |
|-------------|-------------|
| Cualquiera que use `tt()` | Render con i18n mock. Verificar que strings se rendericen. |
| `mark-classic.tsx` | Render de benchmark data. Estados vacío/error. |
| `wal-simulator.tsx` | Interacción del usuario: click, drag, cambio de inputs. |
| `code-playground.tsx` | Syntax highlighting, copia al portapapeles. |
| `SiteShell`, `PageTransition`, `SiteNavbar` | Layout y navegación: rutas activas, links correctos, transiciones. |

### P2 — E2E Tests con Playwright (integridad del sistema)

Playwright ya está disponible como MCP server en el proyecto. Los tests E2E cubren flujos completos que unit tests no pueden.

| Test | Qué verifica |
|------|-------------|
| Navegación completa | Todas las rutas públicas (`/`, `/es`, `/en/benchmarks`, etc.) cargan sin errores 500 ni JS exceptions. |
| i18n toggle | Navegar de ES a EN y viceversa, verificar que los textos cambien. |
| Responsive | Las mismas rutas en viewport mobile (390x844) y desktop (1440x900). Sin overflow horizontal. |
| 404 | Navegar a `/ruta-inexistente`, verificar que se muestre la página de error. |

### P3 — Visual Regression (pulido)

Capturar screenshots de páginas clave y comparar contra un baseline para detectar cambios visuales no intencionales.

| Página | Notas |
|--------|-------|
| Home (`/`) | Hero, features, footer |
| Benchmarks (`/en/benchmarks`) | Tabla, gráficos |
| Docs | Una página de documentación con código |

---

## 3. Herramientas Recomendadas

| Herramienta | Propósito | Por qué |
|-------------|-----------|---------|
| **Vitest** | Unit + component tests | Nativo ESM, compatible con Next.js 16 y React 19, mismo ecosistema que Vite, rápido (es 10-20x más rápido que Jest en proyectos grandes). |
| **@testing-library/react** | Component tests | Testing Library > Enzyme: fuerza tests centrados en comportamiento, no en implementación. |
| **@testing-library/jest-dom** | Matchers DOM | `toBeInTheDocument()`, `toHaveTextContent()`, `toHaveAttribute()` — matchers legibles y declarativos. |
| **Playwright** | E2E + Visual regression | Ya disponible como MCP server. Es la herramienta estándar para E2E moderno. Soporta mobile, network intercept, y screenshot diff out of the box. |
| **MSW** | API mocking | No es prioritario ahora porque el frontend no tiene data fetching propio. Agregar cuando se introduzca. |

### No recomiendo

- **Jest**: Vitest es superior en velocidad y compatibilidad ESM.
- **Cypress**: Playwright es más rápido, más barato en CI, y ya está en el stack.
- **Storybook + Chromatic**: Sobredimensionado para el estado actual. Agregar cuando haya un design system formal.
- **Enzyme**: Obsoleto, no compatible con React 18+.

---

## 4. Configuración Sugerida

### Dependencias

```bash
npm install -D vitest @vitejs/plugin-react @testing-library/react @testing-library/jest-dom @playwright/test
```

### Scripts en package.json

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest run",
    "test:coverage": "vitest run --coverage",
    "test:e2e": "playwright test",
    "test:visual": "playwright test --config=playwright.visual.config.ts",
    "test:all": "vitest run && playwright test"
  }
}
```

### Vitest Config (`vitest.config.ts`)

El config debe extender el de Next.js (o ser compatible). Usar `@vitejs/plugin-react` y configurar alias `@/`.

```ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

### Setup File (`src/test/setup.ts`)

```ts
import '@testing-library/jest-dom/vitest';
```

### Playwright Config (`playwright.config.ts`)

```ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  retries: 1,
  use: {
    baseURL: 'http://localhost:3000',
    viewport: { width: 1440, height: 900 },
  },
  projects: [
    { name: 'desktop', use: { viewport: { width: 1440, height: 900 } } },
    { name: 'mobile', use: { viewport: { width: 390, height: 844 } } },
  ],
});
```

---

## 5. Roadmap

### Fase 0 — Infraestructura (1-2 días)

1. Instalar `vitest`, `@testing-library/react`, `@testing-library/jest-dom`
2. Crear `vitest.config.ts` y `src/test/setup.ts`
3. Agregar scripts al `package.json`
4. Verificar que `vitest run` pase (sin tests aún, debe reportar 0 tests)

### Fase 1 — P0: Unit Tests (2-3 días)

1. Test de `dictionaries.ts`: mismos keys ES↔EN, tipos correctos
2. Test de `vanta-data.ts`: shape de exports principales
3. Test de `opengraph-image.tsx`: render sin error

**Resultado**: ~30 tests, cobertura de lógica pura ~70%.

### Fase 2 — P1: Component Tests (3-5 días)

1. Mock de `tt()` como setup global
2. Tests de SiteShell, SiteNavbar, PageTransition
3. Tests de mark-classic, wal-simulator, code-playground
4. Test de página 404

**Resultado**: ~50 tests, cobertura de componentes críticos.

### Fase 3 — P2: E2E Tests (2-3 días)

1. Instalar Playwright, crear `playwright.config.ts`
2. Test de navegación completa (todas las rutas)
3. Test de i18n toggle
4. Test responsive (mobile + desktop)
5. Test de 404

**Resultado**: ~10 tests E2E, cubriendo todos los flujos críticos.

### Fase 4 — P3: Visual Regression (opcional, 1-2 días)

1. Configurar `playwright.visual.config.ts`
2. Generar baseline screenshots
3. Agregar a CI (cuando exista)

**Resultado**: detección automática de cambios visuales no intencionales.

### Fase 5 — Gates Automáticos (CI/CD exists — `ci-web-11.yml`)

Una vez implementados los tests:

1. Agregar `vitest run` al workflow `ci-web-11.yml` como gate pre-merge
2. Agregar `playwright test` como gate pre-deploy
3. `test:coverage` como reporte semanal (sin hard gate — la cobertura es guía, no meta)

> El workflow `ci-web-11.yml` ya existe y corre lint + tsc + build. Solo falta agregar los tests.

---

## Notas Finales

- **No buscar cobertura del 100%.** Priorizar lógica pura y flujos críticos. Un 40% de cobertura en las áreas correctas vale más que 90% en utils triviales.
- **Los tests E2E son lentos.** Ejecutarlos solo antes de deploy, no en cada commit.
- **Los visual regression tests son frágiles.** Usarlos con criterio: solo páginas estables, no componentes en desarrollo activo.
- **Mantener los tests en `src/` junto al código** (co-location), no en un directorio `tests/` separado. Esto mantiene la visibilidad y facilita la refactorización.
- **No mockear lo que no se necesita.** Si un componente es puramente visual y no tiene lógica, no necesita test unitario — el E2E lo cubre.
- **Actualizar esta guía** a medida que se adopten las herramientas. Lo que funciona para un equipo pequeño puede necesitar ajustes cuando el proyecto crezca.