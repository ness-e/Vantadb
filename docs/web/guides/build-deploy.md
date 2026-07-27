# Build & Deploy — Frontend Web

Guía de compilación y despliegue del frontend web de VantaDB.

## 1. Visión general del build

`next build` (con `output: "standalone"`) produce:

```
.next/
  standalone/
    server.js        ← Entry point del servidor Node
    package.json     ← node_modules mínimos para producción
    .next/
      static/        ← Assets estáticos (JS, CSS, imágenes)
      server/        ← Server Components y rutas SSR
      standalone/    ← Trace de dependencias para standalone
  static/            ← Chunks de cliente (copia manual necesaria)
  build-manifest.json
  trace/
```

El output standalone contiene solo los archivos necesarios para correr la app.
Ideal para imágenes Docker ligeras o deploys directos con Node.

**IMPORTANTE**: Next.js standalone tiene un quirk conocido — `.next/static/` no se copia automáticamente a `.next/standalone/.next/static/`. El script `start` lo resuelve con una copia manual.

## 2. Scripts disponibles

| Script  | Comando                            | Propósito                                 |
| ------- | ---------------------------------- | ----------------------------------------- |
| `dev`   | `next dev -p 3000`                | Dev server con HMR en puerto 3000         |
| `build` | `next build`                      | Build standalone para producción           |
| `start` | `node .next/standalone/server.js` | Servir build standalone                    |
| `lint`  | `eslint .`                        | Lint con 34 reglas desactivadas           |

```bash
# Build
npm run build

# Servir localmente (después del build)
npm start
```

### `npm start` — detalle

El script `start` en `package.json` ejecuta:

```bash
node .next/standalone/server.js
```

Copiar `public/` y `.next/static/` al directorio standalone no es automático. El script asume que `public/` está en `../public` relativo a `standalone/` (verificado), pero `.next/static/` puede requerir copia explícita. Verificar después del build que los assets estáticos sirvan correctamente.

## 3. Configuraciones clave

### next.config.ts

```ts
const nextConfig: NextConfig = {
  output: "standalone",
  typescript: { ignoreBuildErrors: true },
  reactStrictMode: false,
};
```

| Opción                  | Valor   | Efecto                                                         |
| ----------------------- | ------- | -------------------------------------------------------------- |
| `output: "standalone"`  | —       | Empaqueta server + node_modules mínimos para deploy            |
| `ignoreBuildErrors`     | `true`  | Errores TypeScript no detienen el build                        |
| `reactStrictMode`       | `false` | No detecta side-effects ni doble-render en desarrollo          |

**Pendiente**: `metadataBase` no está configurado. Sin él, las Open Graph images resuelven a `localhost:3000` en producción. Agregar:

```ts
const nextConfig: NextConfig = {
  ...,
  metadataBase: new URL("https://vantadb.com"),
};
```

### PostCSS

```js
plugins: ["@tailwindcss/postcss"]
```

Solo Tailwind v4. No requiere `autoprefixer` — Tailwind v4 lo incluye internamente.

### tailwind.config.ts (inerte)

El archivo `tailwind.config.ts` existe en el proyecto pero Tailwind v4 **lo ignora**. La configuración real está en `globals.css` via `@theme inline {}`. No modificar tailwind.config.ts — no tiene efecto.

## 4. TypeScript strictness real

`tsconfig.json` tiene `strict: true` nominalmente, pero dos opciones lo debilitan:

- `noImplicitAny: false` — variables sin tipo explícito se infieren como `any` sin error
- `ignoreBuildErrors: true` en next.config — errores TS no bloquean el build

**Protección real**: baja. Combinación de tipos laxos + build que ignora errores significa que bugs de tipo llegan a producción sin advertencia.

Para hardening progresivo:
1. `noImplicitAny: true`
2. Sacar `ignoreBuildErrors` (bloquear build en error)
3. Habilitar `strictNullChecks` si no lo está — es parte de `strict: true`

## 5. Caddy y proxy

El archivo `Caddyfile` en la raíz del proyecto configura un proxy reverso:

```
:81 {
    reverse_proxy localhost:3000
}
```

Caddy escucha en puerto **81** y reenvía tráfico a `localhost:3000` (Next.js dev server).

**Estado actual**:
- Sin TLS (solo HTTP, puerto 81 no estándar)
- Sin headers de seguridad (CSP, HSTS, X-Frame-Options)
- Sin caching de activos estáticos
- Sin compresión explícita (Caddy la aplica por defecto)

Flujo típico de desarrollo:

```bash
# Terminal 1: Caddy
caddy run

# Terminal 2: Next.js dev server
npm run dev
```

`XTransformPort` query param permite redirigir a un puerto custom desde el frontend.

## 6. Docker

**No existe Dockerfile para el frontend web.**

El `Dockerfile` en la raíz del proyecto construye SOLO el server Rust (`vantadb-server`).
`docker-compose.yml` despliega solo el server Rust en puerto 8080.
El directorio `web/` está excluido via `.dockerignore`.

Para contenerizar el frontend hay que crear un `web/Dockerfile` separado. Ejemplo mínimo con standalone output:

```dockerfile
FROM node:22-alpine AS base
WORKDIR /app

FROM base AS deps
COPY package.json package-lock.json ./
RUN npm ci

FROM base AS builder
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:22-alpine AS runner
WORKDIR /app
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public
EXPOSE 3000
CMD ["node", "server.js"]
```

Nota: El multi-stage copia `.next/static` manualmente al runner para evitar el quirk de standalone.

## 7. Variables de entorno

**Estado actual**: el archivo `.env` del frontend está vacío (0 bytes). No hay `NEXT_PUBLIC_*` variables definidas.

El frontend web actualmente **no requiere** variables de entorno para funcionar. No hay:
- API URLs externas
- Tokens de servicio
- Feature flags
- `metadataBase` (pendiente, ver sección 3)

Si en el futuro se integran APIs externas o servicios, las variables deben seguir la convención Next.js:
- `NEXT_PUBLIC_*` → disponible en cliente y servidor
- Sin prefijo → solo disponible en Server Components y Route Handlers

## 8. CI/CD

**Estado actual:** Existe un pipeline de CI básico pero NO hay tests automatizados.

### Lo que SÍ existe

El workflow `.github/workflows/ci-web-11.yml` se ejecuta en cada push/PR a `main` tocando `web/**`. Corre:

1. `npm ci` — instalación limpia de dependencias
2. `npm run lint` — ESLint (con 34 reglas desactivadas, efectividad reducida)
3. `npx tsc --noEmit` — type checking (compensa `ignoreBuildErrors: true` en next.config)
4. `npm run build` — build de Next.js

El proyecto tiene **15 workflows** activos en total (Rust CI, Python, WASM, security, docs, etc.).

### Lo que NO existe

- ❌ Tests unitarios (ni Vitest, ni Jest)
- ❌ Tests E2E (Playwright no está configurado como test runner)
- ❌ Deploy automatizado (no hay Vercel ni Docker para el frontend)
- ❌ Coverage reports

### Pipeline ideal (futuro)

```yaml
# .github/workflows/deploy-web.yml (pendiente — ref: testing.md roadmap)
name: Deploy Web
on:
  push:
    branches: [main, develop]
    paths: ["web/**"]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - run: npm ci
      - run: npm run build
      - run: npm run lint
      - run: npm test          # ← pendiente: requiere vitest
      - run: npx playwright test  # ← pendiente: requiere setup
```

Ver `docs/web/guides/testing.md` para el roadmap de implementación.

## 9. Troubleshooting

### Issues conocidos de build

| #  | Problema                                              | Causa raíz                                      | Solución                                                          |
| -- | ----------------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------- |
| 1  | Errores TS no aparecen en build                       | `ignoreBuildErrors: true`                       | Temporal: `npx tsc --noEmit` para check manual. Permanente: sacar `ignoreBuildErrors`. |
| 2  | Side-effects no detectados en desarrollo              | `reactStrictMode: false`                        | Activar `reactStrictMode: true` temporalmente para debuggear.     |
| 3  | OG images resuelven a localhost                       | `metadataBase` no configurado                   | Agregar `metadataBase: new URL("https://vantadb.com")` en next.config. |
| 4  | Build produce errores raros o inconsistentes          | `next@^16.1.1` — Next 16 no es stable release   | Monitorear releases de Next 16 estable. Lock en versión si es estable. |
| 5  | Next.js elige lockfile incorrecto en monorepo         | `turbopack.root` no configurado                 | Agregar `experimental.turbopack.root` apuntando a raíz del monorepo. |
| 6  | Assets estáticos no sirven (404) después del build    | Quirk de standalone — `.next/static/` no se copia | Agregar `cp -r .next/static .next/standalone/.next/` después del build. |
| 7  | PWA shortcuts rotos                                   | Query params legacy (`?view=benchmarks`) vs rutas (`/benchmarks`) | Migrar shortcuts a rutas limpias.                                  |

### Comandos de diagnóstico

```bash
# Verificar que el build standalone funciona
npm run build && npm start

# TypeScript check manual (aunque build ignore errores)
npx tsc --noEmit

# Verificar puerto en uso
netstat -ano | findstr :3000
netstat -ano | findstr :81

# Verificar que Caddy está corriendo
curl http://localhost:81
```
