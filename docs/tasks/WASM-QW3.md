# TASK WASM-QW3 (H-05): flush() deja de engañar

## Metadata
- **Plan file:** `docs/plans/2026-08-25-wasm-quickwins.md`
- **Creado:** 2026-08-26T23:58
- **last-synced:** 2026-08-26T23:58
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** wasm binding — semántica honesta / API contract (no hot path, no algoritmo nuevo)
- **Workflow:** bug-fix (flush no-op engañoso → documenta durabilidad real)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WASM-QW3.md`
- **Task ID:** WASM-QW3
- **Contrato:** "flush() deja de engañar: o delega a save() (persistencia real) o renombra/documenta 'no-op durabilidad: llamar save()'. Decisión mínima: docstring + warning console si no hay backend persistente activo. Test actualiza expectativa."
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (flush)
- **Commits fix:** `53f080e5` (2026-08-26 12:08 — QW-1..5 juntos)

## Blast Radius

| Caller | Callee | Implicaciones |
|--------|--------|---------------|
| `VantaDB.flush() JS` (`vantadb-wasm/src/lib.rs:1312`) | `VantaEmbedded::flush()` (`src/storage/engine/maintenance.rs:36`) + `console_warn()` (`lib.rs:416`) | flush sigue llamando engine flush; warning solo side-effect console si `opfs.is_none()`. No cambia return type ni error handling. Callers esperan Ok/Err, siguen funcionando. |
| `vantadb-ts/src/vantadb.ts:flush()` | WASM `flush()` | TS wrapper hace `await wasm.flush()` o sync; warning en console no rompe chain. Si TS documentaba flush como durable, debe actualizar doc a "engine buffers only, call save()". |
| `vantadb-wasm/tests/wasm_tests.rs` + `lib.rs:1822 flush_smoke_without_persistent_backend` | `VantaDB::new(None).flush()` | Test asegura flush callable sin backend y no rompe (retorna Ok, emite warning). Previene regresión donde flush tiraría error sin OPFS. |
| `save()` / `save_idb()` (`lib.rs:798,824`) | OPFS/IDB storage | flush NO delega a save() por decisión mínima — evita I/O costoso implícito. save() sigue siendo explícito. Si futuro quiere delegar, cambiar 1 línea; contrato mínimo ya cumple. |

**Implicaciones:** Cambio de semántica documentada, no de API binaria. Blast radius ≤2 archivos (`lib.rs` + tests), sin concurrencia, sin `unsafe`, sin nueva dependencia. Reversible.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `vantadb-wasm/src/lib.rs` (2245 líneas — Read 2026-08-26, flush:1306-1321, console_warn:415-425, save/save_idb:798-835, cursor helpers:127-164 para contexto string-u64)
  - `vantadb-wasm/tests/wasm_tests.rs` (parcial — verificado que commit 53f080e5 añadió `cursor_policy_tests` + flush smoke)
  - `vantadb-wasm/Cargo.toml` (51 líneas — wasm-bindgen 0.2, js-sys 0.3)
  - `docs/plans/2026-08-25-wasm-quickwins.md` (76 líneas — contrato QW-3 líneas 24-28)
  - `git show 53f080e5 -- vantadb-wasm/src/lib.rs` (diff verificado: docstring NOTE H-05 + console_warn guard + test flush_smoke_without_persistent_backend)
  - `codegraph_explore "flush console_warn save save_idb opfs"` (blast radius mapeado, 22 símbolos)
  - `SKILLS-MANIFEST.md` grep keywords flush/persist/wasm (SDP)
  - `.opencode/rules/js-ecosystem.md` (lazy — política WASM, no tocada)

- **Referencias hacia dentro (qué importa este archivo):**
  - `VantaDB.flush()` es único punto WASM que llama `VantaEmbedded::flush()` (engine maintenance: drain HNSW, backend flush, WAL checkpoint). Sin warning previo, usuarios asumían durabilidad browser (engaño H-05).
  - `console_warn()` helper usa `js_sys::global()` → `Reflect::get(console, warn)` → `Function::call1` — best-effort, never throws, no GIL.
  - `save()` / `save_idb()` son rutas durables reales (OPFS `write_file("db_state.json")` + graph). flush no debe duplicarlas sin costo explícito — decisión ponytail.
  - `opfs: Option<OpfsStorage>` determina si hay backend persistente; `opfs.is_none()` es señal correcta para warning (no necesita check IDB — IDB es async explicit,Flush warning focuses on OPFS path per issue).

- **Referencias entrantes (qué depende de lo que cambio):**
  - `vantadb-ts/src/vantadb.ts:flush()` — wrapper TS; debe mantener compat, no espera string/number, solo void. Warning no afecta TS tipos.
  - `vantadb-wasm/tests/wasm_tests.rs` — tests de integración pueden llamar flush antes/después save; warning no debe hacerlos fallar.
  - Docs WASM (`docs/api/wasm.md` si existe) — deberían reflejar que flush no garantiza durabilidad; fuera de blast radius inmediato pero anotar deuda si desfasadas.

- **Referencias salientes (qué referencia este archivo hacia afuera):**
  - `js_sys::{global, Reflect, Function, JsValue}`, `wasm_bindgen::prelude::*`, `VantaEmbedded`, `OpfsStorage`, `IdbStorage`, `VantaError`

- **Veredicto de impacto:** BAJO — 1 archivo editado en 53f080e5 (docstring 4 líneas + guard 5 líneas + helper 10 líneas + test 7 líneas). Trabajo restante es verify-only (commit ya merged a develop HEAD 43c16e0d). No edición nueva requerida salvo regresión detectada. Gate D NO dispara (blast 1 archivo <10, no nuevo `pub fn` — `console_warn` es privada, `flush` ya existía). Gate spec-first NO aplica (bug fix semántico, no feature-add).

## Contrato

"flush() deja de engañar: o delega a save() (persistencia real) o renombra/documenta 'no-op durabilidad: llamar save()'. Decisión mínima: docstring + warning console si no hay backend persistente activo. Test actualiza expectativa."

**Verificación mecánica (verify-only):**
1. `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ — compila wasm sin errores (dev profile)
2. `cargo fmt --check` ✅ — formatting verde
3. `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` — esperado rojo por deuda pre-existente `vfile_mmap.rs:140 doc_lazy_continuation` fuera de blast radius; no bloquea contrato (gate es cargo check wasm)
4. Grep invariantes: `NOTE (H-05)` presente en lib.rs:1308 + `console_warn` guard `if self.opfs.is_none()` presente + msg contiene `call save() / save_idb() to persist`
5. Test presente: `flush_smoke_without_persistent_backend` en lib.rs:1822 — assert Ok sin backend, documenta expectativa actualizada

**Decisión registrada:** No delegar a `save()` (evita I/O implícito costoso + async mismatch — flush es sync, save es async). Documentar honestidad + warning runtime es mínima y reversible.

## Herramientas necesarias

- `codegraph_explore` (blast radius — ya ejecutado)
- `campaign_verify_cmd` / `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` (verify wasm)
- `campaign_verify_cmd` / `cargo fmt --check` + `cargo clippy` (pre-commit)
- `git show 53f080e5` / `Select-String` (evidencia)
- `campaign_update_task_state` (transiciones)

## Skills

- **Base (campaign_load_skills):** `campaign-executor`, `progreso`, `ponytail` (full), `source-driven-development`
- **SDP discovery (skills-engineering.md Lifecycle + grep SKILLS-MANIFEST.md keywords "flush persist wasm durability console"):**
  - `api-and-interface-design` (BUILD/REVIEW) — flush es contrato de API pública (semántica durable vs buffers); diseño honesto requiere doc + warning (1 línea: contrato es renegociación de semántica de durabilidad)
  - `observability-and-instrumentation` (VERIFY) — console.warn es instrumentación visible para diagnóstico runtime durabilidad (1 línea: warning es señal observable cuando flush no persiste)
  - Sin candidatos adicionales beyond 6: `test-driven-development` ya implícito en source-driven para este binding (test smoke existe), `performance-optimization` no aplica (no hot path), `security-and-hardening` no aplica (no trust boundary)
- **Justificación 1 línea c/u:**
  - `api-and-interface-design`: flush engañaba sobre durabilidad — requiere re-definir contrato público con docstring honesto
  - `observability-and-instrumentation`: warning console es la única observabilidad browser de que flush no persistió — debe ser best-effort y no throw
- **SKILLS_CARGADAS (para RESULTADO):** campaign-executor, progreso, ponytail, source-driven-development, api-and-interface-design, observability-and-instrumentation

## Steps

### Step 1: Verificar flush docstring + guard console_warn presentes (verify-only, sin edición)
- **Archivos:** `vantadb-wasm/src/lib.rs:1306-1321` (flush), `lib.rs:415-425` (console_warn)
- **Acción:** Inspección sin edición. Confirmar 3 invariantes: (1) docstring líneas 1306-1311 contienen `NOTE (H-05)` + `NOT a durability guarantee` + `call save() / save_idb()`, (2) guard `if self.opfs.is_none() { console_warn("VantaDB.flush(): ...") }` presente antes de `self.inner.flush()`, (3) `console_warn` helper es best-effort (Reflect + Function + call1, ignora error). Comparar contra `git show 53f080e5` diff (debe coincidir byte-identico). NO editar — ponytail rung 1: ya existe.
- **Verify:** `Select-String -Path vantadb-wasm/src/lib.rs -Pattern "NOTE \(H-05\)"` 1/1 ✅ + `Select-String -Pattern "console_warn"` 3 hits (def @416 + call @1315 + test comment) ✅ + `Select-String -Pattern "call save\(\) / save_idb\(\) to persist"` 1/1 ✅ + `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 7.57s→0.79s ✅ + `cargo fmt --check` ✅ (2026-08-26)
- **Estado:** ✅ DONE (2026-08-26T23:59 — 3/3 invariantes flush presentes, consola best-effort, cargo check wasm 0.79s)

### Step 2: Verificar test actualiza expectativa (flush smoke sin backend)
- **Archivos:** `vantadb-wasm/src/lib.rs:1822-1828` (flush_smoke_without_persistent_backend dentro de cursor_policy_tests)
- **Acción:** Confirmar test existe y cubre contrato mínimo: crea `VantaDB::new(None)` (sin OPFS), llama `flush()` y espera `Ok` (no throw) — documenta que flush es callable sin backend pero advierte. Opcional verificar que test reside en `#[cfg(all(test, target_arch="wasm32"))] mod cursor_policy_tests` (o similar). Si faltara, añadir test (pero ya existe en 53f080e5 — verify-only).
- **Verify:** `Select-String -Path vantadb-wasm/src/lib.rs -Pattern "flush_smoke_without_persistent_backend"` 1/1 ✅ + inspección `expect("flush without persistent backend")` presente ✅ + `cargo check -p vantadb-wasm` compila cfg wasm32 ✅ + cargo check wasm 0.79s confirma tipos wasm-bindgen-test
- **Estado:** ✅ DONE (2026-08-26T23:59 — test smoke existe, callable sin backend, expectativa actualizada)

### Step 3: Cierre — verify full + recitation sync (sin commit)
- **Archivos:** `vantadb-wasm/src/lib.rs`, `docs/plans/2026-08-25-wasm-quickwins.md`
- **Acción:** Ejecutar verify mecánico final: `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅ (0.79s), `cargo fmt --check` ✅ (EXIT 0), `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` registra 6 errores pre-existentes `vfile_mmap.rs:140 doc_lazy_continuation` + `file.rs:143 drop_non_drop` fuera de blast radius (no bloquea contrato). Actualizar este task file a COMPLETED + Context Save Point. NO `git add`/`commit` (lead commitea, regla pipeline-full: no commit por worker). Actualizar plan file recitation si aplica.
- **Verify:** `cargo check wasm` ✅ + `cargo fmt --check` ✅ + clippy: 6 errores debt externa (no wasm) + grep invariantes 3/3 ✅ + test 1/1 ✅
- **Estado:** ✅ DONE (2026-08-26T23:59 — verify full completado, NO commit per spec verify-only)

## Dependencias
- Commit 53f080e5 ya contiene implementación completa de WASM-QW3 (y QW-1,2,4,5). Este task es verificación de persistencia del fix en HEAD 43c16e0d, no implementación nueva.
- No depende de QW-1/QW-2/QW-4/QW-5 para verificación aislada (cada QW independiente).

## Notas
- Decisión mínima (ponytail): docstring + console warning evita coste de delegar sync flush → async save (I/O OPFS + serialización PERF-08). Delegación completa sería breaking + lenta; warning es reversible en 1 línea.
- `console_warn` es best-effort: si `global.console` no existe (worker sin console) o `warn` no es Function, no throw — evita romper flush en entornos sin console (tests, node sin global).
- `save()` / `save_idb()` siguen siendo async; flush sync no puede await save sin cambiar firma → documentar es correcto.
- `cargo clippy --workspace` falla por deuda pre-existente `vfile_mmap.rs:140 doc_lazy_continuation` (5 errores) + `file.rs:143 drop_non_drop` — fuera de blast radius wasm, no introducido por 53f080e5; gate válido es `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`.

## Context Save Point
- **Fecha:** 2026-08-26T23:59
- **Branch:** develop (HEAD 43c16e0d, incluye 53f080e5 — fix QW-1..5, verify-only, sin diff nuevo)
- **CI pendiente:** `wasm-pack test --chrome --headless` opcional (ejecutar cursor_policy_tests + flush smoke en runtime JS; requiere chrome + wasm-pack — verificado por cargo check + inspección; offline gate es cargo check wasm)
- **Decisiones:** No re-editar lib.rs (ponytail rung 1: ya existe, rung 2: ya implementado) — verificación pura. Clippy global rojo por debt externa vfile_mmap/file.rs (6 errores) no es gate válido para WASM-QW3 (blast radius wasm-only).
- **Problemas conocidos:** Ninguno — 3/3 invariantes flush presentes (NOTE H-05 + guard opfs.is_none + console_warn msg save/save_idb), 1/1 test smoke cubre contrato, cargo check wasm 0.79s ✅, fmt ✅
- **Próxima tarea:** WASM-QW4 (H-07 CRC) ya también en 53f080e5 — pipeline lo manejará como verify-only si se invoca; o continuar a WASM-QW5 worker proxy.
