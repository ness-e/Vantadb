# WIRE-07 — Refactors de frontera: crate `vantadb-ffi-core` (OpGate×3) + desacople `server→cli` + bookkeeping trait-split

> **Fase:** F2 · **Plan:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 21) · **Branch:** develop
> **Ruta:** vanta-arch · **Appetite:** max 1sem · **Estado:** ⏳ IN PROGRESS
> **Commit esperado (LEAD — 1 commit por slice, para bisectar):**
> `refactor(ffi): vantadb-ffi-core (OpGate) + server↔cli decouple (WIRE-07)`

## Contrato (verbatim — ley)

"crate `vantadb-ffi-core` consumido por node/py/wasm (`rg 'struct OpGate'` = 1 en el workspace) Y `server` sin
feature `cli` (grafo sin arrastre; `cargo check` por-crate verde: vantadb, vantadb-node, vantadb-python,
vantadb-wasm, vantadb-server) Y `BOUNDARIES.md` §3/§5 actualizados con evidencia F3X `13f0f729` Y suites verdes sin
cambio de comportamiento"

## SDP (Paso 0b)

SDP: `campaign-executor` · `progreso` · `security-and-hardening` (pinned: trust boundary) ·
`source-driven-development` · `incremental-implementation` · `test-driven-development` · `context-engineering` ·
`doubt-driven-development` (BUILD lifecycle) + esperados del seed: `code-simplification`, `rust-write-tests`,
`api-and-interface-design`.

## Verificación de realidad (ajustada vs seed)

- ✅ **OpGate triplicado literal** (bloques de ~80 líneas con la misma semántica drain):
  `vantadb-node/src/lib.rs:661-735` · `vantadb-python/src/lib.rs:96-173` (`#[derive(Clone)]`) ·
  `vantadb-wasm/src/lib.rs:435-514` (con `#[cfg(target_arch = "wasm32")]` en `drain`, Condvar::wait panics en
  wasm32-unknown-unknown). Divergencia real: sólo el mapeo de rechazo (`napi::Error` / `PyRuntimeError` /
  `JsValue`) — el resto es byte-idéntico.
- ✅ **Clamps duplicados**: `clamp_top_k` literal en node (`:34-42`, warn vía `eprintln!`) y python (`:41-48`,
  warn vía `tracing::warn!`); wasm usa `top_k.min(MAX_K)` inline ×5 (`:1304,1335,1369,1466,1504`) sin warning.
  `MAX_K`/`MAX_VEC_DIM`/`MAX_F32_VEC_LEN`/`MAX_BATCH_SIZE` YA viven en core (`src/config.rs:41-57`, re-export
  `src/lib.rs:172`) — el clamps NO debe re-definir constantes (single source of truth WSM-09): la hoja recibe
  `max` por parámetro.
- ✅ **`vantadb-node` NO es workspace member** (standalone, `[workspace]` propio en `vantadb-node/Cargo.toml:13`
  por el crash MSVC de cdylib; comparte `target/`). Es el mismo patrón que su dep actual `vantadb = { path = ".." }`:
  una path-dep a un miembro del workspace raíz funciona sin cambios.
- ✅ **trait-split YA EJECUTADO** (commit `13f0f729` "feat!: trait-split storage-index con hoja neutral + 6 firmas
  a traits (F3X, ADR-042)": `src/index_port.rs` 224L + `src/index/port_impl.rs` 424L + ADR-042 accepted).
  Verificado HOY: `storage → index` sólo queda en shims/fixtures `#[cfg(test)]` (`archive.rs:205-211` fn
  test-only, `archive.rs:470` mod tests, `index/core.rs:156` dentro de un `#[test]`); `index → storage` sólo usa
  la capa `vfile_mmap`/`vfile` movida a la hoja + `search/tests.rs` (test). **La pata del seed es bookkeeping**:
  `BOUNDARIES.md` §3 (L117-160) y §5 ítem 5 (L205-206) siguen listando el ciclo como deuda abierta.
- ✅ **Acople `server→cli` vigente**: `Cargo.toml:177-185` (`server = ["cli", …]`). Cadena real del arrastre al
  binario HTTP: `vantadb-server/Cargo.toml:10` (`features = ["cli","server"]`) + `vantadb-mcp/Cargo.toml:10`
  (idem) → unificación de features → clap/clap_complete/indicatif/anyhow entran al build del server. Razón dura
  del acople en código: `src/server/bootstrap.rs` usa `crate::console::*` y `pub mod console` está
  `#[cfg(feature = "cli")]` (`src/lib.rs:85`); `src/console.rs:16` importa `indicatif` (sólo lo usan
  `create_progress_bar`/`create_spinner`, `console.rs:291,307`, llamados sólo desde `src/cli_handlers/**`).
  `src/server/**` no usa clap ni indicatif directamente; `src/cli_handlers/**` (cli) ya compila con `server` off
  (error runtime en `cli_handlers/server.rs:273`).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `Cargo.toml` (raíz, 792L) · `src/lib.rs` (220L) · `src/index_port.rs` (224L) ·
`docs/dev/architecture/BOUNDARIES.md` (262L) · `docs/dev/architecture/adr/ADR-042-f3x-pub-signatures-major.md` ·
`vantadb-node/Cargo.toml` · `vantadb-python/Cargo.toml` · `vantadb-wasm/Cargo.toml` · `vantadb-server/Cargo.toml` ·
`vantadb-mcp/Cargo.toml` · `vanta-memory/Cargo.toml` · `vanta-proxy/Cargo.toml` · `vantadb-server/src/lib.rs` ·
`vantadb-server/src/main.rs` · `src/console.rs` (1-214/312; resto = format helpers sin deps nuevas) ·
`src/rules`: `.opencode/rules/{js-ecosystem,python-bindings,api-contract,core-engine}.md`.

**Binding lib.rs (2.2k-2.6k L):** lectura de cabeceras + bloques OpGate/clamps + **barrido exhaustivo por símbolo**
(`OpGate|OpState|OpGuard|enter(|op_gate|clamp_top_k|Arc|Condvar|Mutex|PoisonError` — todas las ocurrencias
listadas; los ~40 call-sites `let _g = enter(&self.op_gate)?` no cambian). Greps de apoyo: `vantadb::` paths en
`vantadb-mcp/src` (0 cli-gated) y `vantadb-server/src` (sólo `cli_server`+`config`).

**Referencias hacia dentro (quién depende de lo que toco):**

| Símbolo | Referencias entrantes | Efecto |
|---|---|---|
| `OpGate`/`OpGuard` (nuevo crate) | node `struct VantaDB.op_gate` + 40 `enter()` + `drain()` (close) · py `struct Client.op_gate` + ~40 `enter()` + `op_gate.clone()` (close, GIL detach) + `drain()` · wasm `Client.op_gate` + ~45 `enter()` + `op_gate.drain()` | Reemplazo 1:1; `#[derive(Clone)]` obligatorio (py `close` clona) |
| `clamp_top_k` (nuevo crate) | node `:579,876` · py `:1173,1262,1356,1700,1739,2281,2412-2413` | Los wrappers locales conservan logging idéntico (eprintln / tracing) |
| `crate::console` | `src/server/bootstrap.rs` (22 sitios) · `src/server/telemetry.rs:78` · `src/cli_handlers/**` | Se re-gatea a `any(cli, server)`; los helpers indicatif quedan `#[cfg(feature = "cli")]` |
| feature `server` | `vantadb-server:10` · `vantadb-mcp:10` · `vanta-memory` (`http-server = ["vantadb/server"]`) · `tests/{request_id,server_auth_rotation,rbac_namespace}` (`required-features=["server"]`) | `server` deja de implicar `cli`: sólo se AGREGA `dep:console`; los que necesitan cli lo piden explícito (`vanta-cli` sigue por `--features cli`) |
| `vantadb::cli_server::*` (re-export REVIEW-10) | `vantadb-server/src/server.rs:1` · `vantadb-server/tests/{helpers,server}.rs` · MCP `server.rs` (usa `vantadb::server::*` directo) | NO se toca (`src/server/mod.rs:16,34` ya lo preserva byte-for-byte) — semver-checks lo cubre |

**Blast radius:** 1 crate nuevo (2 archivos) + 3 bindings (imports + borrado de los 3 bloques) + 4 features/Cargo
(raíz, server, mcp) + 2 cfgs de gating (`console.rs`, `lib.rs`) + 1 doc (BOUNDARIES.md). **0 cambios de
comportamiento** en el build por defecto (cli sigue on en default features); el contenedor `vantadb --features
server` sin cli pasa a compilar (capacidad nueva) sin clap/indicatif/anyhow.

**Veredicto:** verde — refactor de frontera aditivo con preservación de re-exports; riesgo acotado a feature graph
(mecánico y verificable con `cargo tree`/`cargo check` por-crate).

## Steps atómicos

| # | Step | Archivos | Verificación | Estado |
|---|------|----------|--------------|--------|
| 1 | **Slice A — crate `vantadb-ffi-core`**: leaf std-only con `OpGate`/`OpGuard` (+ `#[derive(Clone)]`, cfg wasm en `drain`) y `clamp_top_k(requested, max) -> (usize, bool)` (política ERR-022, sin constantes propias) + unit tests | `vantadb-ffi-core/{Cargo.toml,src/lib.rs}` (nuevo) + `Cargo.toml` raíz (`members`) | `cargo check -p vantadb-ffi-core` ✅ + `nextest -p vantadb-ffi-core` 3/3 ✅ + `cargo tree` = 0 deps ✅ | ✅ |
| 2 | **Slice B — migrar node**: `vantadb-ffi-core` dep + import; borrar bloque local; wrapper `clamp_top_k` conserva `eprintln!` | `vantadb-node/{Cargo.toml,src/lib.rs}` | `cargo check` (cwd `vantadb-node`) ✅ 0 warnings propios (sólo pre-existentes de `vantadb` en esa feature-config) | ✅ |
| 3 | **Slice C — migrar python**: idem; wrapper conserva `tracing::warn!`; `Client::close` (GIL detach) intacto | `vantadb-python/{Cargo.toml,src/lib.rs}` | `cargo check -p vantadb_py` ✅ (0 warnings propios) | ✅ |
| 4 | **Slice D — migrar wasm**: idem; el cfg wasm32 vive ahora en la hoja | `vantadb-wasm/{Cargo.toml,src/lib.rs}` | `cargo check -p vantadb-wasm` ✅ host + `--target wasm32-unknown-unknown` ✅ (cfg wasm de `drain` compila) | ✅ |
| 5 | **Slice E — desacople server→cli**: `server = ["dep:console", …]` (sin "cli"); gate `any(cli, server)` de `console` + helpers indicatif `#[cfg(cli)]`; **`vantadb-server`/`vantadb-mcp` → `default-features = false` + set default sin `cli`** (hallazgo: sin esto la unificación re-enciende `cli` vía default features); **gate `wal_sharded` salvage `#[cfg(any(feature = "cli", test))]`** (el build server-only dejaba 10 dead-code warnings de la superficie CLI-only) | `Cargo.toml`, `src/lib.rs`, `src/console.rs`, `vantadb-server/Cargo.toml`, `vantadb-mcp/Cargo.toml`, `src/wal_sharded.rs` | `cargo check -p vantadb --no-default-features --features fjall,server` ✅ (0 warnings propios) + `-p vantadb-server --all-targets` ✅ + `-p vantadb-mcp --all-targets` ✅ + `-p vantadb --all-targets` (default) ✅ + **unit-graph: features vantadb = [advanced-tokenizer,arrow,fjall,fs2,memmap2,rayon,roaring,server,sysinfo] (sin cli/default), PLAN-CLAP-UNITS=0, PLAN-INDICATIF-UNITS=0** ✅ | ✅ |
| 6 | **Slice F — bookkeeping + contrato**: BOUNDARIES §3 (ciclo roto con evidencia F3X) y §5 ítem 5 (cerrado); verify del contrato completo + semver-checks + suites | `docs/dev/architecture/BOUNDARIES.md` | `rg 'struct OpGate'` (rust) = 1 ✅ + checks por-crate ✅ + docs-coverage ✅ + suites 2299/2300 (único rojo: `public_api` por drift de superficie AJENO sin commitear — WIRE-05 staged +2237 L en `src/entity`/`src/sdk` + WIRE-06 `InsertBatchConfig`; mi diff agrega/borra **0 items `pub`**) + semver ✅ (triado) | ✅ |

## Verificación (evidencia)

| Check | Comando | Resultado |
|---|---|---|
| OpGate único (rust) | `rg -n 'struct OpGate' -t rust -g '!target'` | ✅ 1 → `vantadb-ffi-core/src/lib.rs:73` |
| ffi-core tests | `cargo nextest run -p vantadb-ffi-core` | ✅ 3/3 (clamp, reject-post-drain, drain blocks+wakes) |
| ffi-core leaf (0 deps) | `cargo tree -p vantadb-ffi-core -e normal` | ✅ solo el crate |
| check node | `cd vantadb-node; cargo check` | ✅ 0 warnings propios |
| check python | `cargo check -p vantadb_py` | ✅ 0 warnings propios |
| check wasm (host + wasm32) | `cargo check -p vantadb-wasm` · `--target wasm32-unknown-unknown` | ✅ · ✅ (cfg wasm32 de `drain`) |
| server sin cli | `cargo check -p vantadb --no-default-features --features fjall,server` | ✅ |
| core default (cli on) | `cargo check -p vantadb --all-targets` | ✅ |
| server/mcp all-targets | `cargo check -p vantadb-server --all-targets` · `-p vantadb-mcp --all-targets` | ✅ · ✅ |
| grafo sin arrastre | `cargo +nightly build -p vantadb-server --unit-graph -Z unstable-options` | ✅ `vantadb` unit features `[advanced-tokenizer,arrow,fjall,fs2,memmap2,rayon,roaring,server,sysinfo]` (sin `cli`/`default`); `PLAN-CLAP-UNITS=0 PLAN-INDICATIF-UNITS=0` (mismo resultado para `-p vantadb-mcp`) |
| fmt (scoped) | `cargo fmt --all -- --check` | ✅ ninguno de mis archivos en el diff; los únicos `Diff in` son WIP ajeno (`benches/ingestion_concurrent.rs`, `src/ingestion.rs` = WIRE-06; antes `src/entity/linking.rs` = WIRE-05) |
| clippy ffi-core | `cargo clippy -p vantadb-ffi-core --all-targets -- -D warnings` | ✅ |
| clippy core gate-feats | `cargo clippy -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring -- -D warnings` | ⛔ bloqueado por WIP ajeno: único error = `has_active_transaction` never used (`src/storage/engine/mod.rs:601`, modificado por otra sesión en vuelo) |
| clippy core server-only | `cargo clippy -p vantadb --no-default-features --features fjall,server -- -D warnings` | ✅ tras el gate de `wal_sharded` (WIRE-07): de 11 errores (10 salvage + 1 ajeno) a 0 propios — los fns salvage ahora `#[cfg(any(feature = "cli", test))]` (único consumidor `cli_handlers/wal.rs`; `test` mantiene los unit tests) |
| audit / deny | `cargo audit` · `cargo deny check` | ✅ exit 0 · ✅ advisories/bans/licenses/sources ok |
| docs-coverage | `pwsh scripts/validate-docs-coverage.ps1` | ✅ 0 gaps (exit 0) |
| suites core | `cargo nextest run --profile audit -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring --build-jobs 1 -E <fastGateFilter>` (CARGO_TARGET_DIR aislado, FIND-177) | ✅ **2300 run: 2299 passed, 1 failed, 5 skipped** (151s) — el único rojo es `public_api_snapshot_matches_committed_file`: drift de superficie **ajeno** (la API actual tiene +390 L vs snapshot: WIRE-05 staged `src/entity/linking.rs` +754 / `src/sdk/search/fusion.rs` +420 / `src/sdk/**` +2237 total; WIRE-06 `InsertBatchConfig`; snapshot actualizado por última vez en `54261e96`=WIRE-04). Mi diff aporta **0 items `pub` agregados y 0 removidos** en `vantadb` (verificado con `git diff -U0 -- src/lib.rs src/console.rs src/wal_sharded.rs \| rg 'pub '` = vacío) ⇒ neutralidad de superficie probada; el snapshot lo deben regenerar WIRE-05/06 (dueños del cambio de superficie) |
| suite python | `vantadb-python/.venv\Scripts\python.exe -m pytest tests -q` (desde `vantadb-python/`; extensión **FRESCA post-F1**: `cargo build -p vantadb_py` → copia a `vantadb_py/vantadb_py.pyd` — el nombre que Python importa, NO `vantadb_native.pyd`; verificado `rg -a "ffi[-_]core" vantadb_py.pyd` = 1) | ✅ **159 passed, 4 deselected** (pyd fresco 2026-09-28 02:55; incluye `test_close_concurrency.py` MOD-17 = op-gate/drain bajo GIL) |
| semver | `cargo semver-checks -p vantadb` (CARGO_TARGET_DIR aislado) | ✅ corrido: 196 checks — 189 pass / **7 fail** (6 pre-existentes CLI + **1 nuevo `feature server no longer enables cli`** — intencional, triado y documentado en `docs/api/COMPATIBILITY.md` §Pre-release deltas). Ningún finding de items removidos/cambiados ⇒ all-features surface intacto |
| extra: server-only all-targets | `cargo check -p vantadb --no-default-features --features fjall,fs2,memmap2,server --all-targets` | ✅ (incluye los tests `required-features=["server"]`) |
| extra: cli opt-in | `cargo check -p vantadb-server --features cli` | ✅ (el escape hatch al comportamiento previo compila) |
| extra: memory http-server | `cargo check -p vanta-memory --features http-server` | ✅ (consumidor de `vantadb/server` sin cli) |
| BOUNDARIES | §3 flip a CLOSED + §5 ítem 5 ✅ con `13f0f729` + ADR-042 | ✅ |

## Context Save Point

- **Última acción (2026-09-28):** **6/6 steps ✅**. Slices A–F completos: crate `vantadb-ffi-core` (std-only) +
  migración node/py/wasm + desacople `server→cli` (root feature + `console` re-gate + `default-features = false`
  en server/mcp + gate `wal_sharded`) + bookkeeping BOUNDARIES + triage semver en COMPATIBILITY. Evidencia completa
  en §Verificación. Sin commits (LEAD commitea 1 por slice; ver §Deuda).
- **Pendiente (LEAD):** commits por slice + review P2-01 adversarial + regenerar snapshot public-api cuando
  WIRE-05/06 cierren su superficie + flip del plan (`Task 21 Estado`).
- **Decisiones de diseño:** (a) la hoja NO importa `vantadb` (std-only, perfil `index_port.rs`) → `clamp_top_k`
  recibe `max` por parámetro y las constantes siguen en `src/config.rs` (WSM-09); (b) el mapeo de errores queda
  por-binding **a propósito**: cada transporte tiene canal propio (napi Status+String / PyErr con jerarquía /
  `js_sys::Error` con `.code`) y compartirlo forzaría deps napi/js-sys/pyo3 en la hoja (la contaminación que el
  pre-mortem F1 prohíbe) — se documenta en el crate; (c) `enter()` wrapper se queda en cada binding (3 líneas,
  error local); (d) desacople por re-gate de `console` (`any(cli, server)`) + `dep:console` en `server`, NO por
  mover código; (e) **hallazgo clave**: quitar `cli` del feature `server` en el root no alcanza — los dependents
  (`vantadb-server`, `vantadb-mcp`) seguían pidiendo `default` (que incluye `cli`) y la unificación lo re-enciende
  → `default-features = false` + set default sin `cli` en ambos (mismo set efectivo menos cli); la prueba
  definitiva es el unit-graph de nightly (cargo tree sobre-aproxima con la unión del workspace); (f) el gate de
  `wal_sharded` (`any(cli, test)`) es consecuencia de habilitar el build server-only: sin él, 10 dead-code
  warnings de la superficie CLI-only en la config nueva.

## Notas

- La cadena de arrastre real incluye `vantadb-mcp` (dep de `vantadb-server`): sin quitarle `cli` la unificación de
  features lo reintroduce — el desacople es de los dos Cargo, no sólo del root.
- `tui = [..., "cli"]` intacto. `vantadb-server` conserva su feature `cli = ["vantadb/cli"]` como opt-in.
- **cargo tree ≠ build plan:** `cargo tree -p vantadb-server` muestra `clap` porque unifica los defaults de TODOS
  los workspace members (incluye el root `vantadb` con `cli`); el build `-p` real NO. Evidencia definitiva:
  `cargo +nightly build -p vantadb-server --unit-graph -Z unstable-options` (script:
  `target/session-wire07/unit-graph-features.ps1`).
- **WIP ajeno en el árbol (coordinación):** WIRE-05 (`src/entity/**`) y WIRE-06 (`benches/ingestion_concurrent.rs`,
  `src/ingestion.rs`) editan el mismo working tree en paralelo; el `cargo fmt --all --check` global falla por sus
  archivos (no míos) y los warnings extra del core (12 vs 10) son de `src/entity/linking.rs` (WIRE-05).
- FIND-177 (bins lockeados por MCP vivas): usar `CARGO_TARGET_DIR=target/session-wire07` si `-p vantadb-server`
  falla por lock. `CARGO_BUILD_JOBS=2`.
- **semver-checks:** baseline all-features = build pesado en Windows; el primer intento quedó trabado en un lock
  (`target/semver-checks` compartido con sesiones concurrentes) y el retry corre aislado en
  `target/session-wire07/semver`. El CI lo corre main-only (`ci-rust.yml:92-133`); HARD-01 documentó exit 100
  esperado hasta 0.8.0. Evidencia complementaria de preservación de superficie (default features):
  `public-api.txt` snapshot test.

## Review (§Review — P2-01 risk-tiered, HARD-02)

**Tier mecánico: ADVERSARIAL.** El diff toca `src/wal_sharded.rs` (glob `src/wal*.rs`)
y `docs/api/COMPATIBILITY.md` (glob `docs/api/**`) → regla: un diff mixto con AL MENOS un
path adversarial = review adversarial por agente distinto (`vanta-review`/`vanta-audit`).
**NO self-review** (instrucción de la tarea: lo ejecuta el LEAD). Sin veredicto registrado la
tarea NO se marca COMPLETED (gate HARD-07: `review` payload en `campaign_update_task_state`).

Puntos que el reviewer debe atacar (ordenados por riesgo):
1. **Semántica OpGate preservada**: comparar el bloque movido vs. los 3 originales (`git show HEAD:vantadb-node/src/lib.rs` etc.) — byte-equivalencia de `try_enter`/`drain`/`Drop`; el cfg wasm32 de `drain` debe seguir exacto.
2. **`#[derive(Clone)]` obligatorio** en la hoja (python `Client::close` clona el gate, `py.detach(move || gate.drain())`).
3. **Desacople real sin cambio de comportamiento del build default**: unit-graph evidencia `cli` fuera del plan de `-p vantadb-server`; el build con `cli` (default) debe seguir idéntico (incl. `create_progress_bar`/`create_spinner` presentes — snapshot public-api los cubre).
4. **`default-features = false` en server/mcp**: verificar que la lista explícita = default menos `cli` (hoy `[advanced-tokenizer, arrow, fjall, fs2, memmap2, rayon, roaring, server, sysinfo]`) y que `-p vantadb-server --features cli` sigue compilando (opt-in).
5. **Gate `wal_sharded`**: confirmar que `cli_handlers/wal.rs` es el único consumidor y que `test` no rompe los unit tests de salvage.
6. **Semver**: `feature no longer enables cli` documentado en COMPATIBILITY §Pre-release deltas (intencional, 0.8.0).

**Bloqueadores externos conocidos (WIP ajeno concurrente):** `cargo fmt --all --check` y
`cargo clippy -p vantadb <gate feats> -D warnings` fallan por archivos de WIRE-05/WIRE-06
(`src/entity/linking.rs` antes; hoy `benches/ingestion_concurrent.rs`, `src/ingestion.rs`,
`src/storage/engine/mod.rs:601` `has_active_transaction`). Ninguno toca mis archivos.

## Pre-mortem / Stop conditions

- F1 crate FFI con deps pesadas → leaf std-only (`index_port.rs` perfil) + `cargo tree -p vantadb-ffi-core` = 0
  deps. ✅ mitigado por diseño (verificado).
- F2 surface `cli_server` roto → re-exports preservados (`src/server/mod.rs:16,34`) + `cargo semver-checks`. ✅
  verificado: 0 findings sobre items removidos/cambiados; el único finding propio es el intencional
  `feature server no longer enables cli` (documentado en COMPATIBILITY §Pre-release deltas).
- F3 regresión silenciosa → 1 commit por slice (bisectable) + suites por crate. ✅ suites 2299/2300 + py 159/159 +
  ffi-core 3/3; slices separados para el LEAD.
- **Stop:** si `server→cli` no se desacopla sin mover código compartido → DEFER parcial documentado (ffi-core
  primero; FIND para server→cli). *Verificado en discovery: SÍ se desacopla sin mover código (re-gate de
  `console` + `dep:console`), así que NO aplica.* Rabbit hole: mudanzas cosméticas de módulos (BND-08) → NO.

## Deuda / handoff al LEAD

1. **Commits por slice** (bisectables): A) `vantadb-ffi-core/**` + `Cargo.toml` (members) + `Cargo.lock` raíz ·
   B) `vantadb-node/{Cargo.toml,src/lib.rs,Cargo.lock}` · C) `vantadb-python/{Cargo.toml,src/lib.rs}` ·
   D) `vantadb-wasm/{Cargo.toml,src/lib.rs}` · E) `Cargo.toml` (feature server) + `src/lib.rs` + `src/console.rs` +
   `src/wal_sharded.rs` + `vantadb-server/Cargo.toml` + `vantadb-mcp/Cargo.toml` · F) `BOUNDARIES.md` +
   `docs/api/COMPATIBILITY.md` + `docs/dev/tasks/WIRE-07.md`. (Sugerido: A+F primero, o F en el commit de cierre.)
2. **Review P2-01 ADVERSARIAL** (vanta-review/vanta-audit, contexto fresco): puntos en §Review.
3. **`public_api` snapshot**: regenerar (`VANTADB_PUBLIC_API_UPDATE=1`) SOLO cuando WIRE-05/06 hayan cerrado su
   superficie — hoy el rojo es de ellos (evidencia en §Verificación).
4. **Suites JS no corridas** (decisión documentada): vitest de `vantadb-node` exige `napi build` (regenera
   `index.cjs`/`index.js`/`index.d.ts` trackeados) y las suites wasm/ts necesitan wasm-pack/browser; el movimiento
   de OpGate quedó cubierto por `cargo check` de los 3 bindings + 3 unit tests de la hoja + la suite Python
   (MOD-17 = el caso más hostil: drain bajo GIL con hilos). Si el LEAD quiere la suite node, correrla post-commit
   en un árbol limpio.
5. `perf-bench.yml` (WIP ajeno) y `docs/dev/plans/2026-09-26-master-roadmap.md` (F2a) intactos por mí.
6. **FIND candidato (fila NO creada — evita colisión de numeración con WIRE-05/06 activos):** wasm clampa
   `top_k.min(MAX_K)` inline ×5 **sin warning** — divergencia con ERR-022 ("silent truncation stays observable")
   que node/py sí cumplen (warn). No se unificó a `ffi_core::clamp_top_k` en este refactor para no cambiar el
   comportamiento observable del binding (contrato: "sin cambio de comportamiento"); candidato a unificar
   agregando aviso por `console_warn` en una tarea futura de bindings (WIRE-08/API-xx).
