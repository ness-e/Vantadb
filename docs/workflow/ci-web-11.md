# `ci-web-11.yml` — CI: Web — Build & Lint (Next.js)

## ¿Qué hace?

Pipeline de integración continua para el frontend web (Next.js 16 + React 19 + shadcn/ui + Tailwind v4) del sitio de VantaDB.

## ¿Cómo lo hace?

Un solo job `build` con los siguientes pasos secuenciales:

1. `npm ci` — instala dependencias exactas desde `package-lock.json`
2. `npm run lint` — ESLint (34 reglas — muy permisivo)
3. `npx tsc --noEmit` — type-checking de TypeScript
4. `npm run build` — `next build` con standalone output

> **Nota:** No hay tests unitarios ni E2E en `web/`. La nueva web es una Next.js App Router SPA (todo `"use client"`) sin infraestructura de tests aún.

## ¿Qué verifica?

- No hay errores de linting
- TypeScript compila sin errores
- El build de producción (`next build`) es exitoso

## Funcionalidad final

Asegurar que la web del proyecto compila y no tiene errores de tipo antes de integrarse a `main`.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en `web/**`
- **Pull Request** a `main` con cambios en `web/**`
- **Workflow dispatch** manual
