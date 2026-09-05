# WEB-08: Specs Playwright E2E del flujo crítico landing→docs→playground (1 spec mínimo, patrón desktop/e2e)

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-web-quickwins.md
- **Creado:** 2026-08-27
- **Estado:** ✅ COMPLETED
- **Tipo:** frontend + testing

## Blast Radius
Callers | Callees | Implicaciones
- `web/e2e/flujo-critico.spec.ts` — spec E2E principal
- `web/playwright.config.ts` — configuración Playwright con webServer
- `web/public/playground-executor.html` — iframe executor para WASM sandbox
- `web/src/components/vanta/playground-executor.tsx` — componente React que monta el iframe
- `web/src/components/vanta/code-playground.tsx` — usa el executor via hook

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** flujo-critico.spec.ts (47L), playwright.config.ts (36L), playground-executor.html (100L), playground-executor.tsx (275L), code-playground.tsx (405L)
- **Referencias hacia dentro:** solo code-playground.tsx importa playground-executor.tsx
- **Referencias salientes:** WASM binaries en `/vanta-wasm/`, Next.js dev server
- **Veredicto:** cambios aditivos en web/ — nueva infraestructura de testing E2E + fix del iframe executor para que WASM cargue correctamente en sandbox. Sin breaking changes.

## Contrato
"Spec corre verde local (`npx playwright test`); registrado en CI o documentado comando local"

## Herramientas necesarias
- npx playwright test, npm run build, npm run lint
- Skills: frontend-ui-engineering, browser-testing-with-devtools

## Steps
### Step 1: Verificar spec E2E existente y configuración
- **Archivos:** `web/e2e/flujo-critico.spec.ts`, `web/playwright.config.ts`
- **Acción:** Revisar spec existente que cubre landing → /docs#quickstart → /playground con patrón desktop/e2e (asserts por roles/labels visibles)
- **Verify:** `npx playwright test` (falló inicialmente por timeout en WASM)
- **Estado:** ✅ COMPLETED (spec y config existían, pero WASM no cargaba en iframe)

### Step 2: Fix iframe executor para carga WASM en sandbox
- **Archivos:** `web/public/playground-executor.html` (nuevo), `web/src/components/vanta/playground-executor.tsx`
- **Acción:** El iframe sandbox (`allow-scripts allow-same-origin`) no cargaba `window.wasm_bindgen` correctamente. Solución: fetch del módulo WASM JS como texto, eval con `new Function` capturando el return value del IIFE, luego `initSync` con binario fetchado.
- **Verify:** `npx playwright test` pasa (3-4s)
- **Estado:** ✅ COMPLETED (2026-08-27 — WASM carga y ejecuta en iframe sandbox)

### Step 3: Limpieza y verificación completa
- **Archivos:** `web/e2e/flujo-critico.spec.ts` (limpieza debug logs), `web/public/playground-executor.html` (remover console.log debug)
- **Acción:** Eliminar logs de debug, mantener solo asserts funcionales
- **Verify:** `npx playwright test` ✅, `npm run build` ✅, `npm run lint` (pre-existing react-hooks plugin issue)
- **Estado:** ✅ COMPLETED (2026-08-27)

## Cierre
- **Fecha:** 2026-08-27
- **Branch:** develop
- **Resultado:** ✅ COMPLETED — contrato WEB-08 cumplido
- **Verificación:** npx playwright test ✅ (3.8s) · npm run build exit 0

## Archivos tocados
- `web/e2e/flujo-critico.spec.ts` (modificado - limpieza)
- `web/playwright.config.ts` (existente, sin cambios)
- `web/public/playground-executor.html` (nuevo)
- `web/src/components/vanta/playground-executor.tsx` (existente, sin cambios funcionales)
- `web/src/components/vanta/code-playground.tsx` (existente, sin cambios)

## Dependencias
- WEB-07 (iframe sandbox implementado) — ✅ COMPLETED
- Task independiente en Wave 2

## Notas
- El comando local documentado: `cd web && npx playwright test`
- Patrón desktop/e2e: webServer arranca `npm run dev`, asserts por roles/labels visibles sin implementation details
- El fix del iframe resuelve la carga de WASM en contexto sandboxed (srcdoc → archivo HTML separado + fetch+evaluation)