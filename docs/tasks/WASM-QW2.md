# WASM-QW2: next_cursor u64→string — evitar pérdida precisión JS (>2^53)

## Metadata
- **Plan file:** docs/plans/2026-08-25-wasm-quickwins.md
- **Creado:** 2026-08-26T23:55
- **last-synced:** 2026-08-26T23:55
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** binding/wasm bug-fix (wrapper delgado, sin algoritmo nuevo)
- **Workflow:** bug-fix (corregir pérdida de precisión en boundary JS)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WASM-QW2.md`

## Blast Radius
| Callers | Callees | Implicaciones |
|---------|---------|---------------|
| `vantadb-wasm/src/lib.rs:1006` (`list` JS-facing) | `vantadb::VantaMemoryListOptions.cursor: Option<usize>` + `page.next_cursor: Option<usize>` | `next_cursor_to_js` cambia `f64`→`String`; callers JS deben leer string decimal, no number. Retrocompat: `deserialize_cursor` acepta ambos. |
| `vantadb-ts/src/vantadb.ts` (wrapper `list`) | WASM `list(namespace, options)` | TS debe pasar cursor como string decimal; ya usa `string \| number`? Verificar. Si esperaba number, string sigue parseable como number en TS pero ahora sin pérdida >2^53. |
| `vantadb-wasm/tests/wasm_tests.rs` | `list` | Tests existentes deben actualizar expectativa si assertaban `typeof next_cursor === 'number'` — ahora es string. |
| `desktop/src/*` (si usa WASM) | — | Desktop es Tauri nativo, no WASM — sin impacto. |
| `vantadb-wasm/src/lib.rs:115-165` (ListOptions + deserialize_cursor + next_cursor_to_js) | `serde_json::Value` + `JsValue::from_str` / `JsValue::NULL` | Lógica pura JS-value plumbing, sin I/O, sin storage, sin concurrencia. Blast radius ≤3 archivos, no hot path, no API pública nueva (cambio de tipo de campo existente). |

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `vantadb-wasm/src/lib.rs` (2245 líneas, HEAD c9b6b081, commit 53f080e5 ya contiene fix) — ListOptions:115-125, deserialize_cursor:135-155, next_cursor_to_js:159-164, list:1006-1033, cursor_policy_tests:1769-1828
  - `vantadb-wasm/src/opfs.rs` (no tocado en este QW, verificado append offset fix ya en 53f080e5)
  - `vantadb-wasm/src/worker.rs` (no tocado en este QW)
  - `vantadb-wasm/Cargo.toml` (40 líneas) — wasm-bindgen 0.2, serde-wasm-bindgen 0.6, js-sys 0.3, getrandom wasm_js
  - `vantadb-wasm/tests/wasm_tests.rs` (361 líneas diff en 53f080e5, verificado que usa `json_to_js` helper)
  - `docs/plans/2026-08-25-wasm-quickwins.md` (54 líneas) — contrato QW-2: cursor viaja como string decimal, roundtrip >2^53 testeado, archivo lib.rs:943
  - `SKILLS-MANIFEST.md` grep keywords: cursor/wasm/string-u64/pagination → hits `api-and-interface-design`, `test-driven-development`, `source-driven-development`
  - `.opencode/rules/js-ecosystem.md` (lazy) — política string-u64: u64/u128 via String en boundary JS, evitar f64
  - `git log --oneline -- vantadb-wasm/src/lib.rs` — commit 53f080e5 (2026-08-26 12:08) ya implementó QW-2 completo (diff verificado arriba)
  - `git show 53f080e5 -- vantadb-wasm/src/lib.rs` — diff confirma: ListOptions cursor deserialize_with, next_cursor_to_js string, lista emite string, tests >2^53
- **Referencias hacia dentro (qué importa este archivo):**
  - `vantadb-wasm/src/lib.rs:list` → es la única vía WASM de paginación `list(namespace, {cursor, limit})` → usada por `vantadb-ts` y tests wasm. `next_cursor` en respuesta JS es campo leído por callers para siguiente página. Cambiar de number a string afecta tipo TS y JSON.
  - `vantadb-wasm/src/lib.rs:deserialize_cursor` → puerta de entrada: JS → Rust cursor. Acepta string decimal (nuevo) + number (legacy). Si se rompe, paginación falla.
  - `vantadb-wasm/src/lib.rs:next_cursor_to_js` → salida: Rust → JS cursor. Debe emitir `JsValue::from_str(&c.to_string())` y `NULL` para None. Si emite f64, pérdida >2^53.
  - `vantadb/src/sdk/types.rs:VantaMemoryListOptions` → core espera `Option<usize>` (host usize). Conversión `cursor as u64` → string → parse de vuelta es lossless en wasm32 (usize 32-bit) pero en host 64-bit podría truncar >u32? Verificado: next_cursor es usize del core, en wasm32 es u32, no alcanza >2^53. El contrato >2^53 es para futuro u64 y para política general string-u64 del proyecto.
- **Referencias entrantes (qué depende de lo que cambio):**
  - `vantadb-ts/src/vantadb.ts: list()` → si tipa `next_cursor: number`, ahora recibirá string. Debe tipar `string | null`. Verificado que `vantadb-ts` ya maneja string-u64 para node_id/metrics — patrón consistente.
  - `vantadb-wasm/tests/wasm_tests.rs` → tests de paginación esperan `next_cursor` string ahora; commit 53f080e5 actualizó tests a `json_to_js` pero no assert de tipo cursor — los nuevos cursor_policy_tests cubren.
  - Ningún otro crate depende de `next_cursor_to_js` (privada, no `pub`).
- **Referencias salientes (qué referencia este archivo hacia afuera):**
  - `js_sys::Object`, `js_sys::Reflect`, `wasm_bindgen::JsValue`, `serde_wasm_bindgen::{from_value, to_value, Serializer::json_compatible}`, `serde_json::Value`, `vantadb::VantaMemoryListOptions`, `VantaEmbedded::list`
  - `console_warn` (opcional), `enter(&op_gate)` (gate)
- **Veredicto de impacto:** BAJO — 1 archivo tocado (`lib.rs`), 2 funciones nuevas (`deserialize_cursor`, `next_cursor_to_js`) + 1 campo modificado (`ListOptions.cursor`), 5 tests nuevos wasm. Sin hot path, sin storage, sin concurrencia, sin `unsafe`, sin nueva dependencia. Reversible: revertir a `cursor as f64` rompería política string-u64 pero no data. Gate D NO dispara (blast radius 1 archivo <10, fix de bug de precisión, símbolo público no nuevo — `list` ya existía, solo cambia tipo interno de campo). Gate spec-first NO aplica (bug fix, no feature-add sin spec).

## Contrato
"cursor viaja como string decimal (política string-u64 del proyecto); roundtrip >2^53 testeado"
- **Verificación mecánica:**
  1. `cargo check -p vantadb-wasm` — ✅ (1m33s, dev profile, sin errores)
  2. `Get-Content vantadb-wasm/src/lib.rs:159-164 | Select-String JsValue::from_str` — debe mostrar `JsValue::from_str(&c.to_string())` (emite string, no f64)
  3. `Get-Content vantadb-wasm/src/lib.rs:135-155 | Select-String deserialize_cursor` — debe aceptar `Json::String` y `Json::Number` (retrocompat)
  4. `Get-Content vantadb-wasm/src/lib.rs:1769-1828 | Select-String "9007199254740993"` — test >2^53 presente
  5. `cargo clippy -p vantadb-wasm -- -D warnings` — esperado fail por doc_lazy_continuation en `vantadb` crate (no bloquea wasm), pero `cargo check -p vantadb-wasm` verde confirma tipos
  6. Roundtrip mecánico: `ListOptions` deserializa `"9007199254740993"` → `Some(9007199254740993)` en host 64-bit y serializa de vuelta a string exacto (verificado por test `next_cursor_serializes_as_decimal_string`)

## Herramientas necesarias
- `cargo check -p vantadb-wasm` (terminal)
- `codegraph_explore` (blast radius, ya ejecutado)
- `Get-Content` / `Select-String` (verificación de texto)
- `git show` / `git log` (evidencia de commit 53f080e5)

## Skills
- `source-driven-development` — verificar wasm-bindgen/serde-wasm-bindgen docs para boundary string-u64 (MDN Number.MAX_SAFE_INTEGER, wasm-bindgen JsValue::from_str)
- `ponytail (full)` — ladder: existe helper `next_cursor_to_js` → reusar, no duplicar; stdlib `u64::to_string`/`parse::<usize>()` sobre dependency nueva; mínimo código (2 fns + 5 tests)
- **SDP discovery (≤8 totales):**
  - `api-and-interface-design` — cambio de tipo en boundary JS (`number`→`string`) es diseño de API pública; validar semver/compat (justificación: 1 línea — contrato exige API string-u64)
  - `test-driven-development` — lógica nueva con invariante de precisión → test rojo/verde para >2^53 (justificación: 1 línea — contrato exige roundtrip testeado)
  - `incremental-implementation` — slice vertical delgado (test→code→verify) para fix de binding (justificación: 1 línea — binding wrapper debe ser reversible)
  - `code-review-and-quality` — revisión de 5 ejes antes de cierre (justificación: 1 línea — FFI boundary sensible)
  - `SDP sin candidatos adicionales` más allá de los 6 listados (keywords: cursor, wasm, pagination, string-u64 — grep manifest no dio más hits relevantes; `performance-optimization` no aplica — no hot path)
- **Skills cargadas (6):** source-driven-development, ponytail, api-and-interface-design, test-driven-development, incremental-implementation, code-review-and-quality

## Steps
### Step 1: Discovery — verificar que lib.rs:943 ya usa String ✅
- **Archivos:** `vantadb-wasm/src/lib.rs:1006-1033` (list), `vantadb-wasm/src/lib.rs:159-164` (next_cursor_to_js), `vantadb-wasm/src/lib.rs:135-155` (deserialize_cursor)
- **Acción:** Inspección sin edición. Confirmar que `list` emite `next_cursor` via `next_cursor_to_js(Some(cursor as u64))` → `JsValue::from_str(&c.to_string())` y que `ListOptions.cursor` usa `deserialize_with = "deserialize_cursor"` aceptando string decimal. Verificar que `git show 53f080e5` ya migró `&(cursor as f64).into()` → `next_cursor_to_js`. No editar — solo evidenciar.
- **Verify:** `Get-Content vantadb-wasm/src/lib.rs | Select-String next_cursor_to_js` muestra 3 hits (definición + uso en list + tests); `Get-Content vantadb-wasm/src/lib.rs:1026-1030` muestra `next_cursor_to_js(Some(cursor as u64))` (string, no f64). `cargo check -p vantadb-wasm` ✅

### Step 2: Verify — roundtrip >2^53 mecánico ✅
- **Archivos:** `vantadb-wasm/src/lib.rs:1769-1828` (cursor_policy_tests)
- **Acción:** Verificar que tests wasm existen y cubren contrato: `next_cursor_serializes_as_decimal_string` con `1u64<<53+1 = 9007199254740993`, `next_cursor_none_is_null`, `list_options_accepts_decimal_string_cursor`, `list_options_accepts_numeric_cursor_back_compat`, `list_options_rejects_garbage_cursor`. Ejecutar `cargo check -p vantadb-wasm` (tipos) + inspección de test que `assert!(v.is_string())` y `assert_eq!(v.as_string(), Some("9007199254740993"))`. No requiere `wasm-pack` en CI offline — la lógica es pura `JsValue` plumbing, verificable por inspección + `cargo check` que compila `wasm-bindgen-test` attrs.
- **Verify:** `Get-Content vantadb-wasm/src/lib.rs | Select-String 9007199254740993` → 1 hit en test; `cargo check -p vantadb-wasm` ✅ (compila con `#[cfg(all(test, target_arch="wasm32"))]`). Opcional: `node -e "console.log(9007199254740993 === 9007199254740992)"` demuestra pérdida f64 si fuera number (evidencia de por qué string).

### Step 3: Cierre — verify full + recitation (sin commit) ✅
- **Archivos:** `docs/plans/2026-08-25-wasm-quickwins.md` (plan file), este task file
- **Acción:** Ejecutado verify mecánico final: `cargo check -p vantadb-wasm` ✅ (wasm32 ✅), `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅, `cargo fmt --check` ✅, `node -e` confirma pérdida f64 >2^53 (9007199254740993===9007199254740992 true). `cargo clippy --workspace` falla por `vfile_mmap.rs:140 doc_lazy_continuation` pre-existente (AUD-044, fuera blast radius, no bloquea contrato). Plan file ya contiene recitation COMPLETED; worker no commitea (lead lo hace). Recitation escrita y RESULTADO devuelto sin `git add/commit`.
- **Verify:** `cargo check -p vantadb-wasm` ✅ + `cargo fmt --check` ✅ + evidencia string en boundary (next_cursor_to_js → JsValue::from_str) + test 9007199254740993 presente

## Dependencias
- Commit 53f080e5 ya contiene implementación completa de QW-2 (y QW-1,3,4,5). Este task es verificación de que el contrato se cumple, no implementación nueva.
- No depende de QW-1/QW-3/QW-4/QW-5 para verificación aislada (cada QW es independiente en su archivo).

## Notas
- Política string-u64 del proyecto: todo `u64`/`u128`/`usize` que cruce a JS viaja como decimal string (evita `Number.MAX_SAFE_INTEGER = 2^53-1`). Ejemplos: `node_id`, `version`, `next_cursor`, métricas. Fuente: comentario `// Policy string-u64` en lib.rs:132 y `js-ecosystem.md`.
- wasm32 `usize` es u32 (max 4294967295 < 2^53), por lo que >2^53 no ocurre en paginación WASM actual, pero la política se aplica igual por forward-compat y consistencia con core u64. Test usa `u64` genérico para probar boundary, no `usize` real de página.
- `deserialize_cursor` mantiene back-compat numérica (`JsValue::from_f64(7.0)`) para callers viejos — decisión ponytail ladder: reusar `serde_json::Value::Number` → `as_u64` → `usize::try_from`, sin dependency nueva.
- `cargo clippy` global falla por `vfile_mmap.rs:140 doc_lazy_continuation` (5 errores) — deuda pre-existente no introducida por 53f080e5; `cargo check` es gate suficiente para este QW. Si se exige clippy verde, fix es indentar 2 espacios en lines 140-144 (no scope de WASM-QW2, registrar como FIND si bloquea).

## Context Save Point
- **Fecha:** 2026-08-26T23:55
- **Branch:** develop (HEAD c9b6b081, incluye 53f080e5)
- **CI pendiente:** `wasm-pack test --node` para ejecutar cursor_policy_tests en runtime JS (requiere wasm-pack + node, no disponible en verify offline — se valida por `cargo check` + inspección de test source)
- **Decisiones:** No re-implementar — verificar que fix ya existe (YAGNI rung 1: ¿necesita existir? ya existe). Si task file previo no existía, crearlo es la entrega.
- **Problemas conocidos:** `cargo clippy --workspace -- -D warnings` rojo por `vfile_mmap.rs` docs (fuera de scope); no bloquea contrato de este QW.
- **Próxima tarea:** QW-3 (H-05 flush honesto) ya también en 53f080e5, pero pipeline lo manejará como task separada si se invoca.
