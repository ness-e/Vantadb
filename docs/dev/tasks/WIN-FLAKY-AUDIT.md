# WIN-FLAKY-AUDIT — Test flaky solo-Windows `filters_by_namespace_op_and_outcome`

> **Plan:** `docs/dev/plans/2026-09-19-cierre-total.md` (Wave B) · **Tipo:** bug-fix (desktop) · **Estado:** IN PROGRESS

## Contrato
- **Objetivo:** causa raíz + fix determinista del flaky `commands::audit::tests::filters_by_namespace_op_and_outcome` (`desktop/src-tauri/src/commands/audit.rs:127`).
- **Evidencia CI:** run 35467540820 FAIL (90 passed, 1 failed, `assertion left==right failed: both docs events match` en `:134`) vs run 35467546717 mismo commit PASS.
- **AC:** causa raíz identificada (orden/fixtures/tiempo/semilla) + fix determinista (sin sleeps ciegos) + test verde 3× seguidas local (`cargo test -p vantadb-desktop audit`, workdir `desktop/src-tauri`, `-j 2`) + clippy/fmt del scope + 0 `unwrap/expect` nuevos en prod.
- **NextTask:** cierre (orquestador).

## SDP
`SDP: campaign-executor, frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design | keywords: flaky, windows, test, ordering, fixture, audit | cargadas en sesión: systematic-debugging, test-driven-development (+ base SDP registrada)`

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `desktop/src-tauri/src/commands/audit.rs` (224L, prod `read_audit_events:56-82` + tests `84-224`), `desktop/src-tauri/Cargo.toml` (crate `vantadb-desktop`, workspace propio aislado), `.opencode/rules/core-engine.md` (completo), `.opencode/references/definition-of-done.md`, `.opencode/references/test-suite.md`.
- **Codegraph:** `codegraph_explore "audit.rs read_audit_events write_fixture"` → 12 símbolos/4 files; `read_audit_events` 5 callers (1 prod `vanta_audit_events` + 4 tests), `write_fixture` 4 callers (todos en mismo módulo); blast radius = módulo audit únicamente. Frontend `ActivityPanel.tsx` consume el comando vía IPC (no toca lógica Rust).
- **Referencias hacia dentro (del módulo):** `crate::connections::types::{AuditEvent, AuditPage}`, `crate::error::VantaError`, `crate::AppState`; std `fs::read_to_string`; `serde_json::from_str`.
- **Referencias entrantes:** `vanta_audit_events` (IPC `#[tauri::command]`, llamado desde frontend); `read_audit_events` solo desde el comando + tests. Ningún otro crate depende de este módulo (workspace Tauri aislado).
- **Veredicto:** impacto LOCAL al módulo `desktop/src-tauri/src/commands/audit.rs` `#[cfg(test)]`. Prod (`read_audit_events`) no necesita cambio: filtros + `reverse()` son deterministas sobre un archivo dado. El flaky está en el harness, no en la lógica.

## Causa raíz (systematic-debugging Phase 1-3)
- **Hipótesis:** colisión de paths de fixture entre tests paralelos. `write_fixture` construye `temp_dir/vantadb-desktop-audit-<pid>-<nanos>/audit.jsonl` con `pid` + `SystemTime::now().as_nanos()`. En el mismo proceso (`pid` idéntico) y con granularidad gruesa del reloj en Windows (~15.6 ms), dos tests que arrancan en el mismo tick generan el **mismo dir** → `fs::write` del segundo **sobrescribe** el fixture del primero → el lector ve eventos ajenos → `assert_eq!(len, 2, "both docs events match")` falla según qué fixture ganó la carrera. Explica: solo-Windows, mismo commit PASS/FAIL, mensaje `both docs events match` (`:134`).
- **Descartadas:** orden de eventos docs (el `reverse()` es determinista y `tail_reads_newest_first` nunca falló); timestamp idéntico en datos (los `ev()` usan strings fijos distintos); filtro inestable (`is_none_or` + `==` puros, sin hash/tiempo/semilla).
- **Evidencia:** código `write_fixture:91-98` (pid+nanos sin contador/thread); 4 callers paralelos por default (`cargo test` corre tests del mismo binario en hilos); CI Windows fail 1/91 vs pass mismo commit.

## Steps
- [x] **S1 DISCOVERY:** leer módulo + blast radius + causa raíz + task file (este archivo). Verify: task file existe con Impacto Regla 0.
- [x] **S2 RED+GREEN:** fix `write_fixture` único por llamada (AtomicU64 seq + thread-id + pid + nanos) + regression test `fixture_paths_are_unique_across_threads`. Verify: ✅ `cargo test -p vantadb-desktop audit` 3× verde `-j 2` (12 passed: run1 threads=2, run2 threads=2, run3 threads=4; baseline pre-fix local 11 passed — RED canónico = CI run 35467540820 FAIL vs 35467546717 PASS mismo commit).
- [x] **S3 VERIFY+CIERRE:** clippy scope ✅ (`cargo clippy -p vantadb-desktop --all-targets -- -D warnings` exit 0; 10 warnings son del dep `vantadb` lib pre-existente fuera de scope) + fmt ✅ (`cargo fmt --check -p vantadb-desktop` limpio) + 0 `unwrap/expect` nuevos en prod (todo el diff vive en `#[cfg(test)]`) + commit selectivo (NO PUSH).

## Scope permitido / prohibido
- **Permitido:** `desktop/src-tauri/src/commands/audit.rs` (harness `#[cfg(test)]` únicamente; prod intocable salvo que RED lo exija).
- **Prohibidos (intocables):** resto de `src/` fuera del módulo audit, `vanta-memory/`, `vantadb-mcp/`, `vantadb-server/`, `.github/workflows/`, `web/`, `vantadb-ts/`, `desktop/` excepto el archivo clave, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock` (incl. `desktop/src-tauri/Cargo.lock` modificado ajeno), stash GOV-C4, `docs/dev/Backlog.md`, plan (solo recitation), `C:/Users/Eros/.vantadb*`. WIP en `git status` ajeno → commit selectivo solo del archivo clave.

## Refs
`.opencode/rules/core-engine.md` (R-3: 0 unwrap/expect nuevos en prod) · `definition-of-done.md` · `test-suite.md` (nextest en root; desktop usa `cargo test` por workspace propio) · Tabla Spec: N/A (fix test).
