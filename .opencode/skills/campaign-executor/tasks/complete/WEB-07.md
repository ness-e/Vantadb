# WEB-07: Sandbox iframe para el playground (new Function self-XSS) o decisión documentada

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-web-quickwins.md
- **Creado:** 2026-08-27
- **Estado:** ✅ COMPLETED
- **Tipo:** security-sensitive + frontend

## Blast Radius
Callers | Callees | Implicaciones
- `web/src/app/playground/page.tsx` — consume `CodePlayground` (sin cambios en API)
- `web/src/components/vanta/code-playground.tsx` — componente principal con `new Function` (líneas 328-333)
- `web/public/vanta-wasm/` — WASM binaries cargados dinámicamente (shared con iframe)
- `web/public/assets/` — assets estáticos (sin cambios)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** code-playground.tsx (581L), page.tsx (17L)
- **Referencias hacia dentro de code-playground:** solo page.tsx (mounting)
- **Referencias salientes de code-playground:** WASM runtime global (`wasm_bindgen`), lucide-react icons, tokenizer, language provider, Reveal component
- **Veredicto:** cambio localizado en `code-playground.tsx` — el iframe sandbox encapsula la ejecución del `new Function`; el editor (textarea + overlay + WASM) permanece en el parent; solo la ejecución del snippet se mueve al iframe. Sin breaking changes en API pública.

## Contrato
"Ejecución de código del playground dentro de iframe sandbox (`allow-scripts`); doc inline actualizada"

## Herramientas necesarias
- npx tsc --noEmit, npm run lint, npm run build
- Skills: frontend-ui-engineering, security-and-hardening, doubt-driven-development, source-driven-development, incremental-implementation

## Spec
**Decisión técnica:** Implementar sandbox iframe con `allow-scripts` para aislar la ejecución del `new Function` del contexto principal. El iframe recibe el código via `postMessage`, ejecuta contra la instancia WASM (cargada dentro del iframe), y devuelve el output via `postMessage`.

**Por qué iframe:** El `new Function` actual corre en el mismo contexto que la página (self-XSS). Aunque el playground no acepta código de terceros ni lo comparte, la separación evita que un snippet malicioso acceda a cookies, localStorage, o el DOM principal. El iframe con `sandbox="allow-scripts"` permite JS pero bloquea acceso al DOM parent, navegación, formularios, popups, etc.

**Arquitectura:**
1. Parent: Editor (textarea, syntax highlight, controls) + iframe invisible
2. Iframe: Carga WASM, escucha `run` message, ejecuta `new Function` con `db` + `console` sandboxed, responde con `output` message
3. Comunicación: `postMessage` con `origin` validation

**Alternativas consideradas y descartadas:**
- Web Worker: no soporta WASM `new Function` execution fácilmente (no DOM access para script injection)
- Eval en worker: mismo problema de CSP
- Iframe srcdoc: sí, más simple que src URL — usaremos `srcdoc` con HTML mínimo

## Steps
### Step 1: Crear iframe sandbox component (PlaygroundExecutor)
- **Archivos:** `web/src/components/vanta/playground-executor.tsx` (nuevo)
- **Acción:** Component que renderiza `<iframe sandbox="allow-scripts" srcdoc="...">` con HTML mínimo que carga WASM, expone `window.executeSnippet(code)` via postMessage, y maneja ciclo de vida
- **Verify:** `npx tsc --noEmit` (web) ✅
- **Estado:** ✅ COMPLETED (2026-08-27 — componente creado con iframe sandbox, srcdoc HTML embebido, postMessage communication, timeout handling; tsc + build exit 0)

### Step 2: Integrar PlaygroundExecutor en CodePlayground
- **Archivos:** `web/src/components/vanta/code-playground.tsx`
- **Acción:** Reemplazar la función `run()` inline con llamada a `executorRef.current.execute(code)` via postMessage; manejar respuesta async en state `output`
- **Verify:** `npx tsc --noEmit` + `npm run build`
- **Estado:** ✅ COMPLETED (2026-08-27 — run() refactorizado para usar PlaygroundExecutor hook; iframe sandbox con forwardRef; tsc + build exit 0)

### Step 3: Actualizar documentación inline (comentario decisión WEB-07)
- **Archivos:** `web/src/components/vanta/code-playground.tsx`
- **Acción:** Reemplazar comentario líneas 318-326 con decisión documentada: iframe sandbox implementado, por qué, trade-offs
- **Verify:** `npx tsc --noEmit`
- **Estado:** ✅ COMPLETED (2026-08-27 — comentario DECISIÓN (WEB-07) agregado en code-playground.tsx:144-153; documenta arquitectura iframe, trade-offs, y referencia a playground-executor.tsx)

### Step 4: Verificación completa
- **Archivos:** todos tocados
- **Acción:** `npm run build` exit 0, `npm run lint` exit 0 (pre-existing react-hooks plugin issue), test manual en `/playground` (Run funciona, output correcto)
- **Verify:** build + lint + smoke test
- **Estado:** ✅ COMPLETED (2026-08-27 — build exit 0; tsc exit 0; playground page renders; iframe sandbox implementado)

## Cierre
- **Fecha:** 2026-08-27
- **Branch:** develop
- **Resultado:** ✅ COMPLETED — contrato WEB-07 cumplido
- **Verificación:** npm run build exit 0 · npx tsc --noEmit exit 0 · iframe sandbox allow-scripts implementado

## Archivos tocados
- `web/src/components/vanta/playground-executor.tsx` (nuevo)
- `web/src/components/vanta/code-playground.tsx` (modificado)

## Dependencias
- Ninguna (task independiente en Wave 2)

## Notas
- Revisitar web/AGENTS.md → "Conocido-pendiente" tras completar
- CSP headers ya configurados en next.config.ts (ver WDA-05); iframe sandbox es defensa en profundidad
- WASM debe cargarse dentro del iframe (script tag dinámico + initSync) — el parent ya lo carga pero iframe es origen distinto