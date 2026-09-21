# MOD-15: Nits agrupados del server (middleware.rs re-export, feature sysinfo, main.rs raw engine, ServerState para tests)

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-desktop-ux-core.md
- **Fuente:** backlog `MOD-15` + `docs/reviews/modulos/vantadb-server.md` §8.4-8.6 + P3
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust (HTTP server hygiene — `.opencode/rules/server-mcp.md`)
- **Turns estimados:** 6
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED (implementación worker + review vanta-review; commit del lead)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-server` es crate thin: `src/lib.rs` declara `pub mod middleware; pub mod server;`; tests `tests/{server,e2e,benchmarks,mcp_integration}.rs` importan `vantadb_server::server::{app, ServerState}`; `.opencode/rules/server-mcp.md` lista `middleware.rs` en scope |
| Callees | `vantadb::cli_server` (source de todos los re-exports), `vantadb-mcp` (binario MCP), tests/common/mod.rs (harness core incluido via `#[path]`) |
| Implicaciones | Ningún contrato público del server cambia: `middleware.rs` NO tiene consumidores (`vantadb_server::middleware::` = 0 matches en todo el workspace); feature `sysinfo` vacía nunca habilita nada; main.rs solo recibe comentario; helper `build_server_state` ya existe (refactor de 3 call-sites idénticos, test-only). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-server/src/middleware.rs` (1L), `vantadb-server/src/server.rs` (4L), `vantadb-server/src/lib.rs` (9L), `vantadb-server/src/main.rs` (148L), `vantadb-server/Cargo.toml` (52L), `vantadb-server/tests/helpers/mod.rs` (37L), `vantadb-server/tests/server.rs` (640L — construcciones ServerState :152/:371/:416/:519/:613), `vantadb-server/tests/e2e.rs` (:232), `vantadb-server/tests/benchmarks.rs` (:281), `.opencode/rules/server-mcp.md`, `docs/reviews/modulos/vantadb-server.md` §8.4-8.6+P3.
- **Archivos referenciados hacia dentro (imports/includes):** `lib.rs:7` → `pub mod middleware;`; `middleware.rs:1` → `vantadb::cli_server::{auth_middleware, AuthIdentity, AuthState}` (mismos items que `server.rs:1-4` re-exporta); `tests/server.rs:4` y `tests/mcp_integration.rs:4` → `#[path = "../../tests/common/mod.rs"] mod common;` (harness core con `#[cfg(feature = "sysinfo")]` — referencia la feature del crate compilado).
- **Archivos que referencian a los editados (referencias entrantes):** grep workspace `middleware` → docs/reviews (hallazgo fuente, histórico), task files MEM-05/DESKTOP-01b (históricos), `.opencode/rules/server-mcp.md:3` (scope — se actualiza). **Cero consumidores de `vantadb_server::middleware::` en código.** grep `sysinfo` en `vantadb-server/` → solo Cargo.toml (feature :39 + dev-dep :24); el consumo real es el cfg del harness core bajo el crate de tests. `ServerState` → construido en 8 sitios de tests.
- **Veredicto impacto:** **bajo** — eliminar `middleware.rs` no rompe nada (re-export muerto, server.rs ya expone los mismos items); quitar feature sysinfo vacía no cambia builds (nunca habilitada); comentario en main.rs no altera comportamiento; refactor de tests es test-only con semántica idéntica (helper añade `ensure_indexes_current` — aditivo, no cambia el server). NO se toca comportamiento del server (timeouts/rate-limit/auth intactos).

## Contrato
`cargo check -p vantadb-server` pasa + `cargo test -p vantadb-server` verde + `cargo fmt --check` + `cargo clippy -p vantadb-server --all-targets -- -D warnings` (0 warnings); nits resueltos o documentados.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) ningún cambio de comportamiento del server (auth, rate-limit, timeouts, rutas 1:1); (2) `vantadb_server::server::{app, ServerState, ...}` re-exports intactos (única puerta pública del crate); (3) clippy `-D warnings` sin lints nuevos (incl. `unexpected_cfgs` del harness core bajo este crate); (4) tests de RBAC/breaker/TLS siguen construyendo `ServerState` con sus campos custom (no forzar al helper genérico).
- **Comandos de verificación:** `cargo check -p vantadb-server` ✅ esperado · `cargo test -p vantadb-server` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅
- **Deuda pendiente:** ninguna (los 4 nits se resuelven o documentan inline).

## Recitation (canónico — estructura única)

- `activeGoal`: MOD-15 — resolver 4 nits de higiene del server: (1) middleware.rs re-export redundante, (2) feature sysinfo vacía, (3) main.rs abre engine raw sin comentario, (4) constructor ServerState para tests.
- `lastAction`: DISCOVERY completo — leídos todos los archivos clave + review source §8.4-8.6; veredictos: nit1 → eliminar (0 consumidores); nit2 → eliminar feature+dev-dep + `#[allow(unexpected_cfgs)]` en 2 includes del harness; nit3 → comentario; nit4 → helper ya existe (documentar + refactor 3 sitios idénticos).
- `result`: `PARTIAL` (task file creado, steps pendientes)
- `nextAction`: Step 1 — eliminar `middleware.rs` + `pub mod middleware;` en lib.rs + actualizar scope en server-mcp.md; verify `cargo check -p vantadb-server`.
- `contract`:
  - `verificacion`: `cargo check -p vantadb-server` + `cargo test -p vantadb-server` + `cargo fmt --check` + `cargo clippy -p vantadb-server --all-targets -- -D warnings` (pendiente de ejecutar)
  - `evidencia`: claim: `vantadb_server::middleware::` tiene 0 consumidores — evidencia: grep `middleware` workspace-wide (solo docs/task históricos + server-mcp.md) — confianza: alta; claim: feature `sysinfo = []` nunca habilitada y sin `cfg(feature="sysinfo")` en src — evidencia: grep sysinfo en vantadb-server (solo Cargo.toml) — confianza: alta; claim: helper `build_server_state` ya existe — evidencia: tests/helpers/mod.rs:9-37 — confianza: alta
  - `artefactos`: `.opencode/skills/campaign-executor/tasks/MOD-15.md`
  - `invariantes`: ver "Invariantes de dominio"
  - `deuda`: ninguna
  - `queda_pendiente`: lead verifica mecánico y commitea (sub-agente NO commitea); review P2-01 por agente distinto antes de COMPLETED.
- `nextTask`: la que asigne el lead (MOD-15 es Task 5 del plan; siguientes: FIND-17 / TIR-08).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — cambio neto NEGATIVO (elimina archivo muerto, feature vacía y dev-dep sin uso; solo añade comentario + doc en helper).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: `cargo check -p vantadb-server` + `cargo test -p vantadb-server` + fmt + clippy -D warnings |
| **Commit** | Lead commitea (sub-agente no commitea — regla plan Wave 0) |
| **Release** | No aplica (publish=false, crate experimental) — justificado en Notas |

## Herramientas necesarias
- cargo check/clippy/fmt (terminal), grep/read (CodeGraph sync deshabilitado — lock de otro proceso)

**Skills cargadas (SDP):** `incremental-implementation` (4 ediciones pequeñas por nit, verificar cada una) · `source-driven-development` (validar consumo real antes de eliminar — hecho con grep, no suposición) · `security-and-hardening` (FASE SECURITY — ver Notas) · base: campaign-executor/progreso/ponytail via MCP. SDP sin candidatos adicionales.

## Investigation Notes
- Review source: `docs/reviews/modulos/vantadb-server.md` §8.4 (`middleware.rs` duplica server.rs — "verificar antes de borrar": verificado, 0 consumidores), §8.5 (feature sysinfo — "verificar consumo o eliminar": consumida solo como cfg del harness core bajo este crate), §8.6 (main.rs raw engine — "comentario lo dejaria explicito"), P3 (constructor ServerState — helper ya existe desde MOD-12).
- Plan file Notas: CodeGraph auto-sync deshabilitado → lectura directa (hecho).

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — Evaluado: N/A sustantivo. Los cambios NO tocan trust boundaries: `middleware.rs` es shim muerto (eliminar no cambia el middleware servido — `auth_middleware` sigue expuesto via `server.rs`); feature/dev-dep removidas son build-time; comentario y refactor test-only. `security-and-hardening` cargada, checklist aplicada: no input de usuario, no auth/sesiones, no deps nuevas (solo remoción dev-dep), no FFI, no red. Justificación: higiene sin cambio de comportamiento.
- [x] **PERFORMANCE** — N/A: no hot paths (crate thin, re-exports, tests). Justificado.

## Steps

### Step 1: Nit 1 — eliminar middleware.rs re-export redundante
- **Archivos:** `vantadb-server/src/middleware.rs` (delete), `vantadb-server/src/lib.rs` (quitar `pub mod middleware;`), `.opencode/rules/server-mcp.md` (scope: quitar `middleware.rs`)
- **Acción:** borrar el archivo; `lib.rs` queda solo con `pub mod server;`; actualizar scope del rules file. `vantadb_server::server` ya re-exporta los mismos items → los callers importan directo de `server`.
- **Verify:** `cargo check -p vantadb-server` ✅ + grep `middleware` en `vantadb-server/` = 0 matches en código ✅ (solo `auth_middleware` item, correcto)
- **Estado:** ✅

### Step 2: Nit 2 — feature sysinfo vacía
- **Archivos:** `vantadb-server/Cargo.toml` (quitar feature `sysinfo = []` :39 y dev-dep `sysinfo = "0.38"` :24), `vantadb-server/tests/server.rs` + `vantadb-server/tests/mcp_integration.rs` (añadir `#[allow(unexpected_cfgs)]` al `mod common;` — el harness core usa `cfg(feature = "sysinfo")` que ya no es feature declarada de este crate)
- **Acción:** feature no habilita nada y no tiene uso en src → eliminar. El harness compartido (tests/common/mod.rs) referencia la feature por cfg → el allow en el include evita `unexpected_cfgs` (warn → clippy -D warnings fallaría).
- **Verify:** `cargo check -p vantadb-server --tests` ✅ + `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅ 0 warnings
- **Estado:** ✅

### Step 3: Nit 3 — comentario en main.rs (engine raw)
- **Archivos:** `vantadb-server/src/main.rs`
- **Acción:** comentario 2-3 líneas sobre `StorageEngine::open_with_config` en rama MCP: embedded-first — el binario es boundary thin; `vantadb_mcp::run_stdio_server` corre ensure_indexes_current internamente (único consumidor del handle raw). Solo comentario, NO refactor.
- **Verify:** `cargo check -p vantadb-server` ✅
- **Estado:** ✅

### Step 4: Nit 4 — documentar helper ServerState + refactor 3 sitios idénticos
- **Archivos:** `vantadb-server/tests/helpers/mod.rs` (doc comment en `build_server_state`), `vantadb-server/tests/server.rs` (:519 TLS, :613 certification), `vantadb-server/tests/benchmarks.rs` (:281)
- **Acción:** doc comment: constructor canónico para tests (parámetros: path relativo, api_key, concurrency; corre `ensure_indexes_current` como producción MOD-12; variantes RBAC/breaker/reopen construyen manual — campos custom). Reemplazar las 3 construcciones manuales idénticas por `helpers::build_server_state(...)` (misma semántica + ensure aditivo). NO tocar: RBAC (:152), breaker (:371/:416), e2e reopen (:232) — semántica distinta.
- **Verify:** `cargo test -p vantadb-server` ✅ 42/42 (3 main + 5 bench + 2 cli_args + 12 e2e + 1 mcp + 19 server)
- **Estado:** ✅

### Step 5: Verify full
- **Archivos:** — (solo comandos)
- **Acción:** `cargo fmt --check` ✅ · `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅ 0 warnings · `cargo test -p vantadb-server` ✅ 42/42 · `cargo check -p vantadb-server` ✅
- **Verify:** todos exit 0 ✅
- **Estado:** ✅

### Step 6: Review P2-01 + cierre
- **Archivos:** task file
- **Acción:** delegar review a agente distinto (`vanta-review`) → ✅ approve; mejora opcional aplicada (main.rs "only consumer needing index reconciliation"); task file actualizado. NO commit (lead — verifica mecánico y commitea por tarea).
- **Verify:** veredicto approve registrado ✅
- **Estado:** ✅

## Dependencias
- MOD-12 (creó helper `build_server_state` + ensure) — ya ✅, base del nit 4

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sesión `ses_fc5ce6973ffeMghM6uF9B7DaqL`)
- **Enfoque:** ✅ approve — eliminación middleware.rs no rompe nada (0 consumidores, server.rs re-exporta superset); `#[allow(unexpected_cfgs)]` fix correcto (alternativas evaluadas: check-cfg declarativo = over-engineering, mantener feature vacía = peor); refactor a `build_server_state` semántica idéntica diff por diff (única diferencia `ensure_indexes_current` aditivo/idempotente); invariantes intactos; comentario main.rs factual (run_stdio_server corre ensure bajo `!read_only`).
- **Cómo se probó:** el revisor ejecutó `cargo clippy -p vantadb-server --all-targets -- -D warnings` ✅ 0 warnings, `cargo test -p vantadb-server` ✅ 42/42, `cargo fmt --check` ✅ (no auto-reporte).
- **Checklist anti-hábitos tóxicos:** ✅ sin hallazgos.
- **Veredicto:** ✅ approve (+ 1 mejora opcional aplicada: redacción "ONLY consumer" → "only consumer needing index reconciliation; trailing flush() is durability-only" en main.rs).

## Notas
- Nit 4 parcialmente ya-resuelto: helper `build_server_state` existe desde MOD-12 — se documentó y se amplió su uso a los 3 sitios idénticos; NO se duplicó.
- Nit 2: decisión empírica confirmada — `#[allow(unexpected_cfgs)]` en los 2 includes del harness (server.rs + mcp_integration.rs) cubre el cfg interno; clippy -D warnings 0.
- Release level DoD no aplica: `vantadb-server` publish=false, crate experimental (CI_POLICY.md).
- Commit + `skill progreso` los ejecuta el lead (plan: "Sub-agentes NO commitean; el lead verifica mecánico y commitea por tarea").
- WIP colateral resuelto: AGT-04 (otro batch, plan inexistente) cerrado como FAILED en su task file para desbloquear one-task-at-a-time — reversible.