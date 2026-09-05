# TASK-ID: ERR-OBS-01 - Captura y observabilidad: Backtrace + tracing estructurado + docs

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 9, Wave 3)
- **Creado:** 2026-09-02 (sesión vanta-worker)
- **last-synced:** 2026-09-02
- **Estado:** ✅ COMPLETED (2026-09-02, contrato 7/7)
- **Esfuerzo:** 🟡 1d apetite → real ~½d · **Prioridad:** 🟠 Media-Alta

## Scope ejecutado (4 pilares de captura + docs)

1. **Backtrace (stable, sin nightly):** rustc 1.95.0 verificado → `std::backtrace::Backtrace` es estable desde 1.65 → el gate nightly del plan ("Fallo 1") NO aplica. Campo `backtrace: Option<Backtrace>` en `ChainedError`, capturado en los DOS constructores (`msg`/`with_source`) vía helper `capture_backtrace()` (Disabled→None). Expuesto en `Debug` (derive) + `backtrace()`/`backtrace_str()`. **Nunca en `Display`** (contrato cross-language). `// ponytail:` documenta techo (captura en constructores; upgrade path: OnceCell diferido si algún constructor entra en hot path). Los 40+ call-sites usan los constructores (grep `ChainedError {` = 0 literales fuera del módulo) → cero blast radius en callers.
2. **tracing estructurado:** `log_vanta_error` en `src/server/errors.rs` con campos estables `error.code` / `error.retriable` / `error.hint`, nivel por clase (4xx=WARN, 5xx=ERROR) vía helper puro `error_log_level` (testeado). Cableado en `query_error_response` y `vanta_error_response`. `panic_error_response` ya logueaba (intacto, recibe `&dyn Display` no `VantaError`). `event!` con level dinámico no compila (E0435, callsite const) → dos brazos con set de campos idéntico.
3. **metrics counter — NO cableado (por instrucción):** crate externo `metrics` NO es dependencia del crate raíz (verificado `rg metrics Cargo.toml` = solo nombres de test-target + módulo in-tree `src/metrics/`). Siguiendo la regla "no agregar deps sin justificar" → TODO documentado en `OBSERVABILITY.md` §4 + **FIND-53** registrado en Backlog. Interim: tasas derivables de logs §3.
4. **catch_unwind FFI — verificado, todo cubierto (respuesta correcta = documentar, no inventar trabajo):**
   - Python: `pyo3-0.29.0/src/impl_/trampoline.rs:301` → `std::panic::catch_unwind` + `PanicTrap` en CADA `#[pyfunction]`/`#[pymethods]` → `pyo3_runtime.PanicException` (`src/panic.rs`, derivada de BaseException). Evidencia local del registry (fuente fidedigna, versión exacta).
   - WASM: `vantadb-wasm/src/lib.rs:1901` `console_error_panic_hook::set_once()` → mensaje real al console; el panic devenía trampa wasm = `RuntimeError` catchable en JS. Sin wrappers extra.
   - Server: tasks en `tokio::spawn` → `JoinError` (panic capturado por tokio) → `panic_error_response` (`handlers.rs:129,192,678`) ya sanitiza (AUDREP-32).
   - Bins: anyhow `main()` con `.context()` imprime cadena completa (ERR-CORE-02, pre-existente).
5. **Docs:** `docs/operations/OBSERVABILITY.md` NUEVO (cadena de error, `RUST_LIB_BACKTRACE=1`, niveles, TODO métricas, alerta `rate(5xx) > 2× baseline`, evidencia panics, verificación). Nota de gate en `CI_POLICY.md` (jobs table). Principio #6 backtrace en `docs/api/ERROR_HANDLING.md`. FIND-53 + FIND-54 en Backlog.

## Archivos clave — evidencia de revisión (regla de oro del plan)

| Archivo | Revisado | Resultado |
|---|---|---|
| `src/error.rs` | ✅ completo (1269L) | +Backtrace en ChainedError + helpers + 2 tests |
| `src/server/errors.rs` | ✅ completo (220→276L) | +log_vanta_error/error_log_level + 1 test |
| `src/bin/vanta-cli.rs` | ✅ | anyhow ya (ERR-CORE-02) — sin cambio |
| `vantadb-server/src/main.rs` | ✅ | anyhow ya; telemetry en `cli_server::init_telemetry` — sin cambio |
| `Cargo.toml` | ✅ | métricas: sin `metrics` externo → FIND-53; sin edición |
| `vantadb-python/src/` | ✅ (grep) | trampoline PyO3 cubre panics → evidencia §6 |
| `vantadb-wasm/src/lib.rs` | ✅ (grep) | panic hook + trap JS → evidencia §6 |
| `docs/operations/OBSERVABILITY.md` | ✅ NUEVO | — |
| `docs/operations/CI_POLICY.md` | ✅ | nota post-jobs |
| `docs/api/ERROR_HANDLING.md` | ✅ | principio #6 |

NO tocados (ajenos a esta tarea / otros agentes en paralelo): `desktop/`, `web/`, `vantadb-mcp/`, `src/server/bootstrap.rs` (fuera de mi lista de archivos; panic hook de subscriber queda como candidate si se quiere ERROR-level en panics de tasks fuera de tokio).

## Verificación (contrato mecánico — todo con evidencia de run)

| Ítem | Comando | Resultado |
|---|---|---|
| 1 | `rg -c "Backtrace" src/error.rs` | 11 ≥ 1 ✅ |
| 2 | `cargo test -p vantadb --lib error::tests` | 80 passed; 0 failed ✅ (+ `chained_error_backtrace_follows_capture_status` corre Both branches: `RUST_LIB_BACKTRACE=1`→Some, =0→None ✅) |
| 3 | `rg -c "error.code" src/server/errors.rs` | 3 ≥ 1 ✅ |
| 4 | `cargo check -p vantadb --all-targets` | exit 0 ✅ |
| 5 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 ✅ |
| 6 | `cargo fmt --all -- --check` | exit 0 ✅ |
| 7 | `test -f docs/operations/OBSERVABILITY.md` | True ✅ |
| suite | `cargo test -p vantadb --lib` | 1983/1983 ✅ (run 1 tuvo 3 flakes de tests de disco en paralelo; re-runs verdes 2/2) |
| server | `cargo test -p vantadb --lib --features server server::errors` | 4/4 ✅ |

## Hallazgos ejecución
- **FIND-54 (pre-existente, no introducida aquí):** `server::router::tests::cors_layer_none_when_empty` falla determinístico en HEAD (`HeaderValue::from_str("")` es válido en http 1.5.0 → `Some` ≠ `is_none()` asserted); el test nunca corrió en CI porque el perfil audit no habilita `server`. Registrado con repro y fix trivial (15m).
- **Premisas del plan resueltas:** Fallo 1 (nightly) falso — stable desde 1.65, toolchain 1.95. Stop condition no aplica.
- **NOTICIED BUT NOT TOUCHING:** sanitización del 500 en `query_error_response` (`"Execution Error: {}"` filtra Display interno en variantes 5xx como `BackendError`) — el plan la menciona pero NO está en el contrato de 7 cláusulas de esta tarea ni en los 4 pilares; change de response body requiere su propia validación cross-binding (consumidores TS regex-mirroran prefijos). → candidate FIND para wave 4 si el orquestador lo aprueba.
- **Tracing de tests:** el assert de nivel por status queda cubierto por test del helper puro (state, no interaction). Forzar asserts de output de tracing requiere capricornio `tracing-subscriber` test-buffer — YAGNI por ahora.

## Cierre
- **Commit:** `feat(observability): Backtrace + tracing estructurado + docs OBSERVABILITY (ERR-OBS-01)`
- **Files del commit:** `src/error.rs`, `src/server/errors.rs`, `docs/operations/OBSERVABILITY.md`, `docs/operations/CI_POLICY.md`, `docs/api/ERROR_HANDLING.md`, `docs/Backlog.md` (FIND-53/54)
- **Colateral NO stageado deliberadamente:** `completions/*`, `.opencode` (regla de tarea), `desktop/*` (ERR-DESK-01 en paralelo).
