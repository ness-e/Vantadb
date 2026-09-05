# WASM-PLAYGROUND-001 (legacy: WEB-001)

## Metadata
- **Plan file:** docs/plans/2026-08-04-launch-web-campaign.md
- **Creado:** 2026-08-04
- **Estado:** ✅ CERRADO/LEGACY (2026-08-04 - playground WASM experimental; movido fuera del namespace WEB-0xx de Fase 3; ver BACKLOG_HISTORY)

## Blast Radius
El playground pasó de simulador a ejecución WASM real del core de VantaDB en el browser.
Callees: `vantadb-wasm/pkg` (build `no-modules`), `initSync(bytes)` → clases globales (`VantaDB`).
Callers: `code-playground.tsx` → `playground/page.tsx` → `/playground` route.

## Contrato
"grep 'wasm' en code-playground.tsx → import real de @vantadb/wasm o path local funcionando; npm run build pasa"

## Herramientas
- wasm-pack 0.15.0, target `wasm32-unknown-unknown`
- Playwright (runtime verification)
- npm build (web/)

## Decisiones clave
1. **Rebuild `wasm-pack build --target no-modules`** (no `--target web`).
   El glue `web` emite `import * as wasm from "./*.wasm"` que NO funciona en browser sin bundler
   (drager/wasm-pack#1432, ESM-integration spec).
2. `no-modules` → classic script global `wasm_bindgen.initSync(bytes)` → sin bundler.
   Fuente: https://rustwasm.github.io/wasm-bindgen/reference/no-modules.html
3. Copiamos `vantadb-wasm/pkg` (gitignored) a `web/public/vanta-wasm/` (versionado) para servir el .wasm.
4. Loader en `code-playground.tsx`: carga script + fetch del .wasm + `initSync`, caché por promesa singleton.

## Steps
### Step 1: Rebuild wasm no-modules
- **Acción:** `wasm-pack build --target no-modules` → pkg emite `initSync` (verified: línea 1371)
- **Estado:** ✅ CERRADO/LEGACY (2026-08-04 - playground WASM experimental; movido fuera del namespace WEB-0xx de Fase 3; ver BACKLOG_HISTORY)

### Step 2: Copiar pkg a web/public/vanta-wasm
- **Acción:** copiar vantadb-wasm/pkg/* a web/public/vanta-wasm/
- **Estado:** ✅ CERRADO/LEGACY (2026-08-04 - playground WASM experimental; movido fuera del namespace WEB-0xx de Fase 3; ver BACKLOG_HISTORY)

### Step 3: Integrar loader real en code-playground.tsx
- **Acción:** reemplazar simulador con initSync real (VantaDB class global)
- **Estado:** ✅ CERRADO/LEGACY (2026-08-04 - playground WASM experimental; movido fuera del namespace WEB-0xx de Fase 3; ver BACKLOG_HISTORY)

### Step 4: Verify build + runtime
- **Verify:** `npm run build` → 35/35 páginas ✅ (incluye /playground)
- **Runtime (Playwright):** pendiente re-verificación post-commit
- **Estado:** ✅ CERRADO/LEGACY (2026-08-04 - playground WASM experimental; movido fuera del namespace WEB-0xx de Fase 3; ver BACKLOG_HISTORY)

## Notas
- `vantadb-wasm/pkg/` está en .gitignore — NO se versiona; `web/public/vanta-wasm/` SÍ (copia).
- El build no-modules estandarizado; el runtime browser se verificó con http 200 en los 3 assets.

## Context Save Point
- **Fecha:** 2026-08-04
- **Branch:** develop
- **CI pendiente:** web build ✅
- **Decisiones:** no-modules sobre web por issue wasm-pack#1432 (namespace import requiere bundler)
- **Problemas conocidos:** runtime subject a initSync correcto post-commit
- **Próxima tarea:** MKT-15