# ERR-WEB-01 — Web toast por código de error + catch sin silenciar

- **Plan:** `docs/plans/2026-09-02-error-observability-excellence.md` · Task 7 (Wave 3)
- **Estado:** ✅ COMPLETED (2026-09-02)
- **Commit:** `10cc9671` `fix(web): toast por código de error + catch sin silenciar (ERR-WEB-01)`

## Pasos ejecutados

1. **DISCOVERY:** sweep `catch` en `web/src` → 4 bloques: `code-playground.tsx:174` (literal `catch {}`), `copy-utils.ts:17/37` (fallback clipboard), `opengraph-image.tsx:17` (asset opcional, ya justificado inline — NOT TOUCHING). Confirmado: la app **no importa `vantadb`** (WASM corre en iframe sandbox, postMessage solo pasa strings) → duck-type `error.code`, sin `instanceof VantaError`.
2. **toast.tsx:** helper exportado `toastError(error: unknown)` — duck-tipea `error.code: string`, mapea `errors.<CODE>` vía `dictionaries.ts` (idioma desde `document.documentElement.lang`, sync por LanguageProvider), fallback `toast.error`, message crudo solo en dev.
3. **dictionaries.ts:** +10 claves ES +10 EN `errors.VANTADB_*` (códigos canónicos de `vantadb-ts` ERROR_CODES, Wave 2 `2686bea2`), strings cortos accionables.
4. **catch no silenciados:**
   - `code-playground.tsx:174` → `catch (e) { console.error(...) }` + justificación (nudge en loop 10×500ms: toast spam; el fallo real se reporta en el output panel). En el catch exterior de `run()` → `toastError(err)` añadido.
   - `copy-utils.ts:17/37` → decisión DISCOVERY: `console.warn` + comentario explícito en ambos (degradación intencional: el caller tosea `false`, el error queda loggeado).
5. **HTTP throw sweep:** único `throw new Error(\`HTTP ${status}\`)` en `docs/guides/page.tsx:31` — ya se muestra en estado visible (no toast, code = status en el message) → sin cambio (scope discipline).

## Verificación (contrato)

| Check | Resultado |
|---|---|
| `rg -c "catch \{\}" web/src` | **0** ✅ |
| `rg -c "VANTADB_" web/src/lib/dictionaries.ts` | **22** (≥10) ✅ |
| `rg -n "error.code" web/src --glob "*.tsx"` | **4** (≥2) ✅ |
| `npm run build --prefix web` | exit 0, 36 rutas ✅ |
| `npm run lint --prefix web` | 0 errors (4 warnings pre-existentes, archivos no tocados) ✅ |
| `npx playwright test` (web/) | **2 passed** (WEB-08 guard + web09 screenshots) ✅ |
| web jest/vitest | no existe — N/A |

## Notas

- `opengraph-image.tsx:17` y `docs-view.tsx:640` inspeccionados y dejados: el primero ya tiene justificación inline (asset opcional documentado en web/AGENTS.md); el segundo es string duro de componente sin i18n (convención del archivo, fuera del contrato).
- Techo `ponytail:`: el mapeo code→i18n vive duplicado por diseño mínimo con la cadena de fallback del provider (`language-provider.tsx:63`); extraer un `translate` compartido cuando un 2º consumidor no-React lo necesite.
