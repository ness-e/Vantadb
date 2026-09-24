# FIND-81 — higiene `vantadb-server/` (artefactos junto al crate)

- **Estado:** ⏳ IN PROGRESS → ✅ al cerrar
- **Plan:** `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 18, Wave5)
- **Branch/Commit:** develop / `chore:` (solo propios)
- **Appetite:** 2h · 🟢 · 🟢 Baja · Ruta vanta-docs
- **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design (discover_skills_v2 BUILD, keywords repo-hygiene/gitignore/leftover-artifacts/server-docs; lifecycle-noise incluido, solo se usan executor+source-driven+docs). Skill cargada: documentation-and-adrs.

## Contrato

`vanta_certification.json` movido/borrado con justificación + `vantadb_data/` en gitignore (+ limpieza local documentada) + mini README 5 líneas en `vantadb-server/README.md` (nuevo).

## DISCOVERY — resto vs runtime (pre-mortem del plan)

**Veredicto: ambos son RESTOS. Borrado local seguro. Gate V no dispara.**

### `vantadb-server/vanta_certification.json` (26.297 bytes) → RESTO

- Contenido: array de reportes VantaHarness (`API LAYER (MCP PROTOCOL): ...`, timestamps 2026-07-02 y 2026-07-15). Reporte generado, no fuente.
- Único escritor: `tests/common/mod.rs:227-228` (`REPORT_FILE`, `VANTA_CERT_REPORT`) + `:359-367` abre path **relativo al CWD** en modo append. Ningún lector en código (grep `vanta_certification` = solo writer + `.gitignore:29` + `collect_code.ps1:60` exclusión + docs).
- Las suites `vantadb-server/tests/server.rs:604` (`api_server_certification`) y `mcp_integration.rs:20` (`mcp_protocol_certification`) corren ese harness → el archivo nace de correr tests con CWD=`vantadb-server/`.
- Estado git: untracked (`git ls-files` vacío) + ignored (`.gitignore:29` `vanta_certification.json`, basename-match → `git check-ignore -v` confirma).

### `vantadb-server/vantadb_data/` (335.603.104 bytes ≈ 320 MiB) → RESTO

- Layout StorageEngine real (`data/`, `keyspaces/`, `0.jnl` 67 MB, `.vanta.lock`, `.vanta.schema`, `lock`, `version`) pero **stale: todos los mtimes 2026-08-14 01:57** (1 mes, cero escrituras desde entonces).
- Origen: `vantadb-server/src/main.rs:52,60` → `Config::from_env()` → `src/config.rs:902` (`VANTADB_STORAGE_PATH`, fallback `"vantadb_data"` relativo al CWD). Correr el binario/tests con CWD=`vantadb-server/` crea exactamente este dir.
- Proceso vivo verificado: `vantadb-server.exe --mcp` PID 14704 (hijo de `vanta-cli server --mcp --db C:/Users/Eros/.vantadb`, arrancado hoy 10:57) → storage explícito en `C:/Users/Eros/.vantadb`, **NO** este dir. `netstat` sin sockets propios (stdio MCP). No se toca ese proceso (infra del entorno).
- Cero literales `vantadb-server/vantadb_data` en código (grep); tests del crate usan `tempfile::TempDir` (`tests/helpers/mod.rs:25`, `e2e.rs:209`).
- Estado git: untracked + ignored (`.gitignore:26` `/vantadb_data/` + `:137` `vantadb_data*/`; `git check-ignore -v` confirma `:137`).

### `.gitignore` → YA CUBRE (sin cambios)

`vanta_certification.json` (:29) y `vantadb_data*/` (:137) + `/vantadb_data/` (:26) ya ignoran ambas rutas (verificado con `git check-ignore -v`). Añadir líneas duplicadas sería slop → **no se edita `.gitignore`** (ponytail); el contrato se cumple por verificación, documentada aquí.

### Colateral fuera de scope (no tocado)

`vantadb-server/.vanta_profile` (168 bytes, machine profile) — misma clase de residuo, pero fuera del contrato; también ignored (`:69`) + untracked. Se deja intacto.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-server/src/main.rs` (143L), `tests/common/mod.rs:200-400` (harness writer), `.gitignore` (líneas 24-32, 67-69, 135-137), `vantadb-server/` dir listing, `vantadb-server/src/` listing.
- **Referencias hacia dentro:** main.rs → `vantadb::config::Config::from_env` + `vantadb_mcp::run_stdio_server_auto`; harness → `OpenOptions` CWD-relativo.
- **Referencias entrantes:** ninguna hacia los artefactos (writer-only / default-path; cero readers, cero literales del path).
- **Veredicto:** blast radius = 0 archivos de código. Solo filesystem local (2 borrados untracked) + 1 README nuevo. Sin API pública, sin símbolos nuevos. Gate D no dispara.

## Steps

- [x] Step 1 (DISCOVERY): resto-vs-runtime con evidencia + pre-mortem proceso vivo → RESTO ambos, Gate V no dispara.
- [x] Step 2 (ACT): borrados `vanta_certification.json` + `vantadb_data/` local (Test-Path False/False, PID 14704 intacto); `vantadb-server/README.md` 5 líneas English.
- [x] Step 3 (VERIFY+CLOSE): `git status` sin esas rutas + `git diff --check` limpio + `cargo check -p vantadb-server` Finished 0 warnings + commit `chore:` (README + task file; borrados untracked no entran al commit, `.gitignore` intacto por cobertura previa) + recitation. Backlog/push al orquestador.

## Gates

- P: no (contrato mecánico del plan, sin ambigüedad). D: no (blast radius 0 archivos código, sin API pública). V: no (resto probado triple: writer-only + mtimes 14/8 + proceso vivo con --db a otro path). C: sí al cerrar (git status fuera de blast radius: `.opencode/`, `completions/`, `desktop/...lock`, `docs/pipeline-state.json` WIP ajeno intacto).

## Verify

- `Test-Path vantadb-server/vanta_certification.json` → False; `vantadb_data` → False; `README.md` → True.
- `git status --short` no lista esas rutas; `git diff --check` limpio.
- `cargo check -p vantadb-server` 0 warnings (prueba de que nada referenciaba los artefactos).

## Save Point

Borrados son locales (untracked+ignored): recuperables vía tests (`cargo test -p vantadb-server` regenera el json) y re-ejecución (data). README + task file en commit `chore:`.
