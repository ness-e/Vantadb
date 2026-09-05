# FIND-58: Gate wasm32 crudo rojo tras 175790a9 — `wasm_js` unificado en crate equivocado

## Metadata
- **Plan file:** — (ejecución directa desde Backlog L221, orden del orquestador 2026-09-03; Gate D N/A: el brief delega el diagnóstico a DISCOVERY con comando exacto y autoriza fix mínimo según causa — alias vs feature-gate — sin reescribir el sistema de features)
- **Creado:** 2026-09-03
- **last-synced:** 2026-09-03
- **Estado:** ✅ COMPLETO (2026-09-03, vanta-worker — Steps 1-3 DONE abajo)
- **SDP:** campaign-executor (pipeline) + ponytail full (activo); base del agente: incremental-implementation (1 slice) + context-engineering (context pack §3a) + source-driven-development (backends.rs 0.3/0.4 leídos como fuente); test-driven-development N/A con rationale (0 lógica Rust: el cambio es declarativo Cargo.toml — la verificación es el gate mecánico mismo, precede TDD de código); systematic-debugging N/A (causa raíz ya aislada por evidencia, no hay bug de lógica); `js-ecosystem.md` leída (R-1..R-4: el fix no toca persistencia IDB/OPFS, ni `pkg/`, ni lifecycle OpGate); `CONSTRAINTS.md` leída (floor: 0 `#[allow]`, 0 stubs; types/fmt gates aplican al scope).
- **Sub-agente:** vanta-worker
- **Área:** `Cargo.toml` raíz (feature `wasm` + `[target.cfg(target_arch="wasm32")'.dependencies]` + `[package.metadata.cargo-machete]` ignored) y `Cargo.lock` (churn mecánico del fix — stagear con rationale). NADA más. Prohibido: `src/ingestion.rs` (FIND-57), `src/cli.rs`/`backup.rs` (GOV-TK1), `vantadb-server/` (FIND-56), `src/sdk/search/debug_ops.rs` (colateral release-only — NOT TOUCHING, se documenta), `completions/*`, `.opencode/`, `stash@{0}`.

## Blast Radius (Discovery con evidencia mecánica)

- **Gate que falla (reproducido 2026-09-03):** `cargo check -p vantadb --target wasm32-unknown-unknown --no-default-features --features wasm` → 2 errores `getrandom` lib: (1) `0.3.4/src/backends.rs:40` "The `wasm_js` backend requires the `wasm_js` feature"; (2) `0.4.3/src/backends.rs:176` "wasm32/64-unknown-unknown are not supported by default; you may need to enable the `wasm_js` crate feature".
- **QUÉ getrandom falla:** DOS copias, ninguna es 0.2 — `cargo tree -i` en el grafo del gate: `getrandom 0.3.4` vía `ahash 0.8.12` (dep directa no-opcional) + vía `rand 0.9.5 → rand_core 0.9.5` (dep directa no-opcional); `getrandom 0.4.3` vía `twox-hash 2.1.2 → rand 0.10.2` (dep directa no-opcional). `getrandom 0.2.17` existe en `Cargo.lock` pero `cargo tree -p vantadb` (wasm y host) NO lo contiene — solo aparece en `--workspace --target all` vía `vantadb-server` dev-deps (`rcgen/ring/rustls`) → fuera del grafo, fuera de scope, NO es el culpable.
- **POR QUÉ el cfg `wasm_js` no lo cubre (fuente: `backends.rs` de ambas versiones, leídos del registry):**
  - 0.3.4 SÍ tiene brazo `#[cfg(getrandom_backend = "wasm_js")]` PRIMERO — el cfg de `.cargo/config.toml` lo selecciona, pero ese brazo exige ADEMÁS `#[cfg(feature = "wasm_js")]` (glue JS) → cfg presente + feature ausente = error #1. cfg solo no basta, tal como dice la fila.
  - 0.4.3 NO tiene NINGÚN brazo `getrandom_backend = "wasm_js"` (eliminado en 0.4) — el cfg se ignora; el path wasm es `cfg(all(target_family="wasm", ...unknown/none))` + `feature = "wasm_js"` → feature ausente = error #2.
  - En el grafo `-p vantadb` (`cargo tree -e features -i`): ambas copias solo tienen `default`/`std`/`sys_rng` — `wasm_js` en 0.
- **POR QUÉ 175790a9 solo desbloqueó wasm-pack:** el commit añadió `getrandom 0.4/wasm_js` directo + alias `getrandom_03 0.3/wasm_js` SOLO a `vantadb-wasm/Cargo.toml`. `cargo build -p vantadb-wasm` (wasm-pack) unifica features porque ese crate está en el grafo; `cargo check -p vantadb` solo NO incluye `vantadb-wasm` en su grafo → los alias no existen para este gate → sigue rojo. El fix unificó en el crate equivocado.
- **Fix mínimo (sin reescribir features):** replicar el patrón alias en el crate RAÍZ, gateado por la feature `wasm` (que hoy es `[]` vacía): 2 deps opcionales solo-`cfg(target_arch="wasm32")` (`getrandom_03`→0.3+wasm_js, `getrandom_04`→0.4+wasm_js) + `wasm = ["dep:getrandom_03", "dep:getrandom_04"]`. En host o sin `wasm` son inertes (no cambian resolución nativa). `cargo-machete` las vería "unused" (nunca hay `use` — son solo unificación) → `ignored` igual que `vantadb-wasm` ya hace.
- **NO es:** bump de versiones (pins de terceros), `patch` de getrandom, cambio de `default-features` de `rand/ahash/twox-hash`, ni rewrite del sistema de features.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `Cargo.toml` raíz (features L103-143, deps `rand 0.9` L41 / `ahash 0.8.12` L100 / `twox-hash 2.1.2` L87 — las 3 no-opcionales), `vantadb-wasm/Cargo.toml` (patrón alias L30-44), `.cargo/config.toml` (cfg backend L9-10), `docs/Backlog.md` L221 (fila), `getrandom-0.3.4/src/backends.rs` L1-80 + `getrandom-0.4.3/src/backends.rs` L1-200 (fuente del mecanismo), `Cargo.lock` (getrandom ×3: 0.2.17/0.3.4/0.4.3), `.opencode/rules/js-ecosystem.md`, `CONSTRAINTS.md`.
- **Referencias hacia dentro:** feature `wasm` hoy `[]` — 0 consumidores en código (`rg` feature wasm en `src/` = solo gates existentes, el cambio no añade `cfg(feature="wasm")` nuevo); las 2 deps nuevas solo existen bajo `cfg(target_arch="wasm32")` + `wasm` activa.
- **Referencias entrantes:** `vantadb-wasm` depende de `vantadb/default-features=false,features=["wasm"]` → al activar `wasm` ahora también unifica `wasm_js` en el grafo del crate raíz — coherente, no conflictivo (mismo feature, misma versión).
- **Veredicto:** blast radius = 1 bloque `[target...]` (4 líneas) + 1 línea `wasm = [...]` + 1 sección machete (2 líneas) + churn `Cargo.lock` mecánico. Rollback-friendly: revert de 1 commit restaura resolución previa. Riesgo: error de sintaxis `dep:` o nombre inválido rompería resolución del workspace → verificado por el propio contrato (`--all-targets` host + workspace sin regresión). Fuera de scope declarado: todo lo de Metadata.

## Contrato

`cargo check -p vantadb --target wasm32-unknown-unknown --no-default-features --features wasm` exit 0 Y `cargo check -p vantadb --all-targets` (default, host) exit 0 Y `cargo check --workspace --all-targets` sin regresión vs HEAD (si ya estaba rojo por `vanta-memory` pre-existente, documentarlo con evidencia, NO arreglarlo) Y `cargo clippy -p vantadb --all-targets` (scope tocado: manifest-only → clippy N/A, se corre como sanity) 0 Y `cargo fmt --check` 0 (0 `.rs` tocados → trivialmente 0).

Commit: `fix(wasm): gate wasm32 crudo verde tras 175790a9 parcial (FIND-58)` — `fix:` → release-plz patch. Stagear SOLO: `Cargo.toml`, `Cargo.lock` (churn del fix, con rationale), task file, `docs/Backlog.md` (fila eliminada), `docs/avance/**` (entrada), decisions (vía memory). NUNCA: `completions/*`, `.opencode/`, stash.

## Herramientas

- read, bash (cargo check/tree, rg, Test-Path), edit (Cargo.toml, Backlog, avance), git (status/diff/add scoped + commit por orden del brief)

## Steps

### Step 1: Fix raíz — alias wasm_js para 0.3 + 0.4 gateados por `wasm`
- **Archivos:** `Cargo.toml` (raíz)
- **Acción:** (1) `wasm = []` → `wasm = ["dep:getrandom_03", "dep:getrandom_04"]`; (2) añadir bloque `[target.'cfg(target_arch = "wasm32")'.dependencies]` con `getrandom_03 = { package = "getrandom", version = "0.3", features = ["wasm_js"], optional = true }` y `getrandom_04 = { package = "getrandom", version = "0.4", features = ["wasm_js"], optional = true }`; (3) añadir `[package.metadata.cargo-machete] ignored = ["getrandom_03", "getrandom_04"]` (paridad con `vantadb-wasm`: aliases de unificación sin `use`).
- **Verify:** gate wasm crudo exit 0 (comando exacto del contrato)
- **Estado:** ✅ DONE (2026-09-03: exit 0 en 9.31s tras el edit; `Cargo.lock` +2 líneas `getrandom 0.3.4/0.4.3` bajo deps de `vantadb`)

### Step 2: Sin regresión host + workspace
- **Archivos:** — (solo ejecución)
- **Acción:** `cargo check -p vantadb --all-targets` (default, host) exit 0; `cargo check --workspace --all-targets` y comparar con HEAD (`git stash` PROHIBIDO — comparar por inspección: si falla, `cargo check` del mismo comando en worktree limpio es imposible sin stash; en su lugar: si el error es en `vanta-memory` u otro miembro no tocado por el diff, con `rg getrandom` = 0 en ese error, se documenta como pre-existente).
- **Verify:** cláusulas 2-3 del contrato
- **Estado:** ✅ DONE (2026-09-03: host `-p vantadb --all-targets` 0; `--workspace --all-targets` 0 final — primera corrida tras el edit dio rojo transitorio `vantadb-wasm` E0433/`vantadb` test gc por fingerprints stale del cambio de manifest, no reproducible en 3 corridas siguientes ni tras `cargo clean -p vantadb -p vantadb-wasm` + rebuild (26.45s); `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0 — logro 175790a9 preservado; `getrandom 0.2` confirmado fuera del grafo `-p vantadb`)

### Step 3: Cierre
- **Archivos:** `docs/Backlog.md` (fila FIND-58 eliminada), `docs/avance/activo/bindings.md` u `operaciones.md` (entrada según tabla progreso — wasm/bindings → bindings.md; si no existe, operaciones.md), memoria `decisions` (causa raíz: qué getrandom/cfg fallaba y por qué), commit scoped
- **Verify:** `cargo fmt --check` 0 + `git status --short` (solo archivos del blast radius) + commit hash
- **Estado:** ✅ DONE (2026-09-03: fmt 0; clippy `-p vantadb --all-targets -D warnings` 0; `cargo machete` 0 — el `ignored` era necesario; fila FIND-58 eliminada de Backlog — solo mención histórica en el contador L17; entrada en `docs/avance/activo/bindings.md`; decisions vía memory; commit scoped por orden explícita del brief)

## Dependencias

- 175790a9 + 580ef0e2 (parcial previo: cfg + alias en crate equivocado) — landed, es el punto de partida. FIND-57/FIND-56/GOV-TK1: paralelos declarados, archivos prohibidos intactos. FIND-59/FUT-12: sin relación.

## Notas

- NOTICED BUT NOT TOUCHING: `src/sdk/search/debug_ops.rs` 4 warnings unused-import solo en release (colateral de la fila) — fuera de ruta (NO tocar `src/`), queda para su dueño; `getrandom 0.2.17` solo en grafo `vantadb-server` dev-deps — fuera de ruta.
- `ponytail:` no aplica como comentario (0 código Rust — cambio declarativo TOML); la simplificación ES no tocar `src/` ni reescribir features.
- Si el fix requiriera toolchain nightly o rebuild 30min+ → documentar y cerrar como hallazgo con workaround (no ocurre: es resolución de features, check incremental).

## Context Save Point
- **Fecha:** 2026-09-03
- **Branch:** develop (sin cambiar)
- **CI pendiente:** no (gates locales del contrato)
- **Decisiones:** alias-en-raíz + `wasm`-gated (no incondicional wasm32) para no cambiar builds wasm32 sin feature `wasm`; nombres `getrandom_03/_04` espejan `vantadb-wasm` (`_03` ya existe allí para 0.3; `_04` evita colisión con el nombre `getrandom` directo que la raíz no tiene)
- **Problemas conocidos:** ninguno bloqueante
- **Próxima tarea:** Step 1
