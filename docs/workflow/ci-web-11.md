# `ci-web-11.yml` — CI: Web — Build & Test

## ¿Qué hace?

Pipeline de integración continua para el frontend web (React + Vite + Tailwind + TypeScript) del sitio de VantaDB.

## ¿Cómo lo hace?

Un solo job `build` con los siguientes pasos secuenciales:

1. `npm ci` — instala dependencias exactas desde `package-lock.json`
2. `npm run lint` — ESLint
3. `npx tsc --noEmit` — type-checking de TypeScript
4. `npm run build` — build de producción con Vite
5. `npx vitest run` — tests unitarios con Vitest
6. `npx playwright install --with-deps chromium` — instala navegador para E2E
7. `npx playwright test` — tests end-to-end con Playwright

## ¿Qué tests usa?

- **Vitest**: tests unitarios definidos en `web/`
- **Playwright**: tests E2E en `web/`

## ¿Qué verifica?

- No hay errores de linting
- TypeScript compila sin errores
- El build de producción es exitoso
- Tests unitarios pasan
- Tests end-to-end en Chromium pasan

## Funcionalidad final

Asegurar que la web/documentación del proyecto es funcional, no tiene errores de tipo, y pasa todas las pruebas antes de integrarse a `main`.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en `web/**`
- **Pull Request** a `main` con cambios en `web/**`
- **Workflow dispatch** manual
