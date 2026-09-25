# FIND-100 — graceful ante onnxruntime incompatible

> **Plan:** `docs/dev/plans/2026-09-17-mvp-memoria-agentes.md` (Wave0, sin dependencias)
> **Ruta:** vanta-worker · **Branch:** develop · **Commit:** `fix: FIND-100 — ...`
> **Appetite:** 1d · **Esfuerzo:** 🟡 · **Prioridad:** 🔴 Alta
> **Estado:** ⬜ PENDING → ⏳ IN PROGRESS (DISCOVERY completo 2026-09-17)

## 1. TAREA — objetivo + contrato + acceptance criteria

**Objetivo:** el init ORT en `src/llm.rs:308` (`let _ = ort::init().commit()`, sin pre-chequeo)
aborta el proceso cuando el dylib ONNX Runtime es incompatible o inexistente. Lo "automático"
(MVP memoria en agentes) exige degradación graceful a embeddings dummy deterministas.

**Contrato (ley):**
- (a) con dylib incompatible el proceso NO aborta: exit controlado + `fallback:true` visible
      (campo estructurado `fallback = true` en `tracing::warn!`, grep-able);
- (b) suite `llm` verde: `cargo test -p vantadb --features embed-local --lib llm -j 2`;
- (c) `cargo clippy -p vantadb --all-targets --features embed-local -- -D warnings` con 0 warnings.

**Fix acotado:** solo `try_load_session` en `src/llm.rs` (+ tests en el mismo archivo).
Sin símbolos públicos nuevos → sin tabla Spec (plan lo permite expresamente).

## 2. ARCHIVOS

**Clave (con :línea):**
- `src/llm.rs:306-308` — `try_load_session`: `ort::init().commit()` con `let _`, sin pre-chequeo.
  El `commit()` en sí retorna `bool` (inofensivo); el abort real está aguas abajo:
  `Session::builder()...commit_from_file()` (`:323-324,335-336,344-345`) dispara
  `ort::api()` → `setup_api()` → `load_dynamic::init(&path).expect(...)` (pánico).
- `src/config.rs:935-940` — `local_model_path` (`VANTADB_LOCAL_MODEL`, default
  `embeddings/models/multilingual-e5-small/onnx`). Solo contexto, NO se edita.

**Relacionados (callers/callees — codegraph_explore + CBM):**
- Callers de `get_embedding_provider` (`src/llm.rs:65,95,105` según features):
  `src/physical_plan/vector.rs:56,149` (fallback local by design), `src/executor.rs`
  (auto_embed_insert / auto_embed_message). Verificado vía codegraph_explore.
- Callees: `LocalOnnxProvider::new` → `from_llm_cfg` (`:198`) → `try_load_session` (`:205`)
  → `Session::builder/commit_from_file`; `embed_as` (`:408`) → `run_onnx` (`:421`) → dummy.
- Fábrica ya es graceful ante modelo ausente (`unwrap_or_else(new_dummy)` en `:73-76,80-83,108-110`);
  el hueco es SOLO el pánico ORT cuando el modelo SÍ existe pero el dylib no sirve.

**Prohibidos (WIP ajeno — NO tocar):**
`reparacion.bat`, `.opencode` (submodule, solo lectura), `Justfile`, `ocr-delegate.yml`,
`ocr-review.ps1`, `completions/*`, `desktop/src-tauri/Cargo.lock`, budgets huérfanos 2026-09-02,
stash@{0} GOV-C4. Tampoco `FIND-107` (`vanta-memory/`, `vantadb-mcp/`) ni `SHOW-04` (`examples/`)
— archivos disjuntos Wave0.

## Impacto mapeado (Regla 0) — MUST antes del primer edit

- **Leídos completos:** `src/llm.rs:1-123` (imports, trait, factory, struct),
  `:180-353` (`from_llm_cfg`, `try_load_session`, `deterministic_embed`, `run_onnx`),
  `:905-1079` (tests embed-local); `src/config.rs:935-940`;
  ort 2.0.0-rc.13 `src/lib.rs:90-175` (`load_dynamic::init`, `LoadError::{Dlopen,MissingApi,BadVersion}`),
  `src/lib.rs:200-260` (`setup_api` con 3× `.expect`), `src/environment.rs:658-660`
  (`commit(self) -> bool`), `init_from` (`:718`, retorna `Result<_, LoadDynamicError>`);
  `Cargo.toml:51` (`ort 2.0.0-rc.13 load-dynamic`), `:689-690` (`unwrap_used/expect_used = deny` a nivel workspace).
- **Referencias hacia dentro (lo que el cambio usa):** `ort::init_from` (público, graceful),
  `ort::init().commit()`, `std::panic::catch_unwind`, `tracing::warn!` (fully-qualified,
  `tracing` es dep directa no-opcional), `std::path::PathBuf`, `std::env::var`.
- **Referencias entrantes (quién usa lo cambiado):** `from_llm_cfg` (único caller de
  `try_load_session`); firma `-> Option<Session>` SE MANTIENE → 0 callers afectados.
  `load_session_inner` (nueva, privada) solo llamada desde `try_load_session`.
- **Veredicto:** impacto mínimo (1 archivo, 1 fn privada reestructurada + 1 helper privado +
  tests). Sin cambio de firma pública, sin hot path (init una vez por proceso),
  sin concurrencia nueva (parking_lot intacto), sin deuda nueva. Gate D: NO dispara
  (blast radius 1 archivo, sin API pública, contrato mecánico).

## 3. DEPENDENCIAS

- **Wave:** Wave0 — sin dependencias, paralela con FIND-107 + SHOW-04 (archivos disjuntos).
- **Bloqueantes:** ninguna.
- **Previa:** ninguna (primera del plan). **NextTask tras cierre:** FIND-103 (Wave1, la ejecuta el orquestador, no este agente).

## 4. REFERENCIAS

- **Rules (lectura completa antes de codificar):** `.opencode/rules/core-engine.md`
  (R-3: `?` + `Result`, excepción locks; R-5: env vars `VANTADB_*` con warn+default, nunca panic).
- **Refs:** `definition-of-done.md` (DoD 3 niveles) · `clean-code-clean-architecture.md` Ap. V
  (V.2: `?` sobre unwrap; V.4: unwrap en prod = 🔴 bloqueante) · `dev-tools.md` + `test-suite.md`
  (comandos verify) · `skills-engineering.md` (SDP).
- **Commands:** `pipeline.md` (ejecución) · `audit.md` (verify L9/post-tarea).
- **Contexto:** SPEC.md raíz (8/8 decisiones, 0 abiertas); `docs/dev/tasks/EMB-10.md:148-171`
  (crash BadVersion ort 1.x era, solución env-only ORT 1.30) como evidencia previa, NO como diseño
  (el ort actual es 2.0.0-rc.13, API 17 — ver §8).
- **Sin símbolo público nuevo** → sin tabla Spec.

## 5. SKILLS (SDP v2 — pipeline-full Paso 0b)

`campaign_discover_skills_v2(archivosClave="src/llm.rs:308, src/config.rs:935-940",
phase="BUILD", contractKeywords=[ort, onnxruntime, graceful, fallback, abort, embedding], maxSkills=8)`
→ 8 skills score 1.0. Cargadas vía `skill`: base ya en sesión + sugeridas del plan.

| Skill | Cuándo aplica (1 línea) |
|---|---|
| systematic-debugging | Root-cause del abort: repro real antes de cualquier fix (Fase 1 ley de hierro) |
| test-driven-development | RED (test incompatible-dylib debe fallar) → GREEN (graceful) → REFACTOR |
| incremental-implementation | 1 slice vertical (~100 líneas: helper + wrapper + tests), repo siempre verde |
| context-engineering | Context Pack por slice: rules → spec → source del slice + ejemplo patrón |
| source-driven-development | API `ort` verificada contra fuente (registry local, NO memoria del modelo) |
| doubt-driven-development | El "graceful" actual es falso (`let _` ignora bool, no el pánico interno) |
| codebase-memory | Blast radius init→factory→fallback (codegraph + CBM ya ejecutados) |
| api-and-interface-design | NO aplica código (sin API nueva); solo como guardia de no-superficie |

SDP: systematic-debugging, test-driven-development, incremental-implementation,
context-engineering, source-driven-development, doubt-driven-development, codebase-memory
(+ base campaign-executor/progreso/ponytail-full ya activas; api-and-interface-design y
frontend-ui-engineering descartadas por scoring ciego a `phase=BUILD` — sin `web/`, sin API nueva).

## 6. HERRAMIENTAS + MCP

- `codegraph_explore` primero (blast radius init→factory→fallback) ✅ ejecutado.
- `codebase-memory-mcp`: `check_index_coverage(src/llm.rs, src/config.rs)` ✅ (`no_recorded_issue`);
  `trace_path(get_embedding_provider)` → ambiguo (3 cfgs; se usó codegraph en su lugar);
  `detect_changes` en cierre si hay diff amplio.
- `cargo test -p vantadb --features embed-local --lib llm -j 2` (contrato b; `-j 2` siempre, OOM Windows).
- `cargo clippy -p vantadb --all-targets --features embed-local -- -D warnings` (contrato c).
- `campaign_verify_cmd`: BUG CONOCIDO exit -1 → bash directa + mención en RESULTADO.
- agent-search/metasearchmcp: NO necesarios (API `ort` verificada en registry local).
- `pwsh dev-tools/ocr-review.ps1` en cierre (Critical/High bloquean; Medium → FIND-*).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — DISCOVERY)

`get_embedding_provider` (3 variantes cfg) → `LocalOnnxProvider::new` → `from_llm_cfg` →
`try_load_session` (PÁNICO AQUÍ si `model.onnx` existe + dylib inservible) /
`try_load_tokenizer` (seguro, `ok()` + `exists()`).
Callers aguas arriba: `physical_plan/vector.rs:56,149`, `executor.rs` (auto-embed).
Implicaciones: el pánico cruza `unwrap_or_else(new_dummy)` (no atrapa pánicos) → tumba
hilo main (CLI abort) o task Tokio (HTTP 500). Riesgo del fix: bajo — `try_load_session`
corre 1 vez por construcción de provider, fuera de hot path; `catch_unwind` + pre-chequeo
no tocan `run_onnx`/inferencia.

## 8. INVESTIGACIÓN PROBLEMA (hipótesis / causa raíz)

**Evidencia previa:** `docs/dev/tasks/EMB-10.md:148-171` — crash ort 1.x era:
`lib.rs:234 expect → BadVersion{1.17.1}` (System32) → mutex poisoned → abort.
**Estado actual verificado (registry local, NO memoria):**
- `ort::init().commit()` retorna `bool` (`environment.rs:658`), NO carga el dylib — el
  `let _` de `llm.rs:308` es inofensivo por sí mismo (falso graceful: ignora un bool,
  no un error).
- La carga real es perezosa en `setup_api()` (`lib.rs:~224-260`): `load_dynamic::init(&path)`
  + `.expect("Failed to load ONNX Runtime dylib")` + `.expect("OrtGetApiBase...")` +
  `.expect("Failed to initialize ORT API")` → **pánico ante Dlopen/MissingApi/BadVersion**.
- Vía pública graceful EXISTE: `ort::init_from(path) -> Result<_, LoadDynamicError>`
  (`environment.rs:718`; `LoadError` con `Display` descriptivo).
- Entorno actual: System32 trae `onnxruntime.dll` 1.17 inbox; `ort-sys` 2.0.0-rc.13 exige
  `ORT_API_VERSION = 17` → 1.17 PASA el gate (`Equal`); dylib con minor <17 o ausente/rota
  → pánico → abort en main. `ORT_DYLIB_PATH` hoy unset.
**Hipótesis (root cause):** `try_load_session` invoca API ORT paniqueante (`Session::builder`)
sin pre-chequeo ni `catch_unwind`; el `Result` de `commit_from_file` nunca llega porque el
pánico ocurre antes, dentro de `setup_api`.
**Fix:** (1) pre-chequeo con `ort::init_from(path)` (retorna `Err` en vez de paniquear) →
`None` + `warn!(fallback = true)`; (2) `catch_unwind` alrededor de la construcción de
sesión como defensa en profundidad (poison de `OnceLock`, pánicos internos de builder).
Sin empaquetar dylib (YAGNI: medir tamaño queda fuera — pre-mortem (2) se responde con
"no se empaqueta nada").

## 9. INVESTIGACIÓN INTERNET

N/A — código local + registry local suficientes. API `ort` verificada contra fuente
(`~/.cargo/registry/.../ort-2.0.0-rc.13`), no hizo falta webfetch. Sin citas URL →
GATE CITAS TSYS-13 no aplica.

## 10. VALIDACIÓN + CIERRE

- Verify contrato: (a) test dylib-incompatible → `Ok(dummy)` sin pánico + `fallback=true`
  en log; (b) suite `llm` verde; (c) clippy 0 warnings.
- Verify full: `cargo fmt --check` + clippy + nextest llm + docs-coverage si aplica.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1`): Critical/High = bloquea, Medium → FIND-*.
- DoD 3 niveles (task/commit/release) + commit conventional `fix: FIND-100 — ...`. NO PUSH.
- Reviewer distinto P2-01: LO HACE EL ORQUESTADOR (no este agente).
- Gates D/V/C: D evaluado (no dispara, §Impacto); V/C activos durante ACT (2 fallas
  mismo-error → STOP + reportar, sin `question` tool disponible en este harness).

## Steps atómicos

- [x] **Step 0 — DISCOVERY:** tipo (bug-fix/rust-core) + SDP + blast radius + task file. Verify: este archivo existe con §§1-10 + Regla 0.
- [x] **Step 1 — RED:** suite `llm` pre-fix ABORTABA (`STATUS_STACK_BUFFER_OVERRUN`):
      `local_embed_multilingual` → `BadVersion{1.17.1}` (`ort/lib.rs:234 expect`) → test FAILED
      → `Mutex poisoned` (mutex_std.rs:15) en main → `panic in a function that cannot unwind` → abort.
      Tests f100 añadidos (fallaban pre-fix: helper inexistente + pánico ORT). Verify: `find100-baseline.txt`, `find100-red.txt`.
- [x] **Step 2 — GREEN:** `resolve_ort_dylib_path()` + `ensure_ort_ready()` (2 capas:
      `init_from` graceful + probe `ort::api()` bajo `catch_unwind` fuera de `G_ENV`) +
      `try_load_session` = ensure + `catch_unwind(load_session_inner)`; `warn!(fallback=true)`.
      Hallazgo profundo (backtrace): el abort post-test venía de `release_env_on_exit`
      (`environment.rs:84`) sobre `G_ENV` envenenado por un pánico bajo su guardia — la capa 2
      (probe fuera de `G_ENV`) lo elimina; exit code 0 verificado en serie y paralelo.
      Sin símbolos públicos nuevos; guard `cfg(target_arch="wasm32")` (gate FIND-58).
      Verify: 15/15 `llm` EXIT=0 (serie+paralelo) + clippy 0 + fmt + `cargo check` default.
- [x] **Step 3 — CIERRE:** fmt + clippy + nextest llm + ocr-review + commit (no push) + lessons + progreso.

## Evidencia mecánica (CIERRE)

- `cargo test -p vantadb --features embed-local --lib llm -j 2` → 15/15 ok, EXIT=0
  (serie `--test-threads=1` y paralelo; pre-fix: abort `0xc0000409`).
- `cargo clippy -p vantadb --all-targets --features embed-local -j 2 -- -D warnings` → EXIT=0.
- `cargo fmt --check -p vantadb` → EXIT=0. `cargo check -p vantadb -j 2` (default) → EXIT=0.
- `campaign_verify_cmd` NO usado (bug conocido exit -1) → bash directa (mencionado en RESULTADO).
- OCR `dev-tools/ocr-review.ps1`: ver abajo (advisory).
- Diff: solo `src/llm.rs` (+~110/-~10) + este task file. Sin deuda nueva (Regla 6: 0 unwrap/expect
  añadidos; los `expect` del test viven bajo el `#![allow]` preexistente del módulo + patrón vecino).

## Context Save Point

DISCOVERY completo 2026-09-17 (código + registry ort verificados; internet N/A).
Siguiente: Step 1 RED. Comando exacto:
`cargo test -p vantadb --features embed-local --lib llm -j 2 -- --test-threads=1`
Riesgo conocido: globals ORT por proceso → RED solo determinista en aislamiento;
documentado en el test. `campaign_verify_cmd` con bug exit -1 → bash directa.

## RESUME 2026-09-17 (re-verificación + commit, sin cambios de código)

- Re-verificado en vivo: `cargo test -p vantadb --features embed-local --lib llm -j 2`
  → 15/15 ok EXIT=0; `cargo clippy -p vantadb --all-targets --features embed-local -j 2
  -- -D warnings` → EXIT=0; `cargo fmt --check -p vantadb` → EXIT=0.
- Commit solo `src/llm.rs` + este task file (WIP ajeno intacto en worktree). NO PUSH.
- OCR delegation + review P2-01: los hace el orquestador (fuera de este resume).
