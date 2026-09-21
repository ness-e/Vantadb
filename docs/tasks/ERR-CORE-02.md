# TASK-ID: ERR-CORE-02 — Clippy unwrap_used/expect_used deny en prod + anyhow bins

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md`
- **Creado:** 2026-09-02
- **last-synced:** 2026-09-02 (reanudación, sesión worker)
- **Estado:** ✅ COMPLETED (5/5 steps + Step 2b colateral workspace sweep)

## Blast Radius
- `Cargo.toml` (workspace.lints.clippy) — afecta a todos los crates del workspace
- `src/bin/vanta-cli.rs` — único bin principal (no existe `src/main.rs`)
- `vantadb-server/src/main.rs` — bin server (idempotente)
- `src/binary_header.rs:67` — 1 `expect` legítimo en prod (guard previo garantiza `bytes[8..16]`)
- `src/cli_handlers/fmt.rs:17`, `src/cli_handlers/data.rs:66` — 2 `expect` en CLI (literal template) — producción
- `src/cli_handlers/export_md.rs:302,305` — 2 `expect` en export_md (frontmatter) — producción
- `src/crypto.rs:203,213,221,272,278,503` — 6 `expect` en crypto (infallible por RustCrypto) — producción
- `src/index/serialize/bytes.rs:17` — 1 `expect` (Vec::write cannot fail) — producción

## Contrato
```bash
grep -n "unwrap_used\|expect_used" Cargo.toml | wc -l >= 1
cargo clippy -p vantadb --all-targets --all-features -- -D clippy::unwrap_used -- -D clippy::expect_used 2>&1 | grep -c "error\[clippy" == 0
grep -n "anyhow::Result\|anyhow!" src/bin/vanta-cli.rs | wc -l >= 1
grep -n "anyhow::Result\|anyhow!" vantadb-server/src/main.rs | wc -l >= 1
cargo fmt --all -- --check; echo $?  # exit 0
```

## Herramientas
- `cargo clippy`, `cargo fmt --check`
- `codegraph_explore "VantaError"` (no necesario aquí — solo lints)
- `grep` (enfoque narrow — lints + anyhow)

## Steps

### Step 1: Añadir `[workspace.lints.clippy] unwrap_used = "deny"` y `expect_used = "deny"` en `Cargo.toml`
- **Archivos:** `Cargo.toml` (líneas 646-659)
- **Acción:** añadir 2 líneas en `[workspace.lints.clippy]`
- **Verify:** `grep -c "unwrap_used\|expect_used" Cargo.toml`
- **Estado:** ✅ DONE — ya en HEAD (`73f49e6f`, líneas 667-668). `rg -n unwrap_used Cargo.toml` → 2 hits.

### Step 2: Marcar expect legítimos en prod con `#[allow(clippy::expect_used)]` + rationale
- **Archivos:** ver Blast Radius original + corrección por discovery-ejecución:
  - `src/binary_header.rs` (invariantes u64 try_into / clock) — `#![allow]` + comentario ponytail ✅
  - `src/cli_handlers/fmt.rs:17`, `src/cli_handlers/data.rs:66` (template literals) ✅
  - `src/crypto.rs` (RustCrypto infallible) ✅
  - `src/index/serialize/bytes.rs` (Vec::write) ✅
  - `src/cli_handlers/export_md.rs:302,305` — VEREDICTO: dentro de `#[cfg(test)]` → cubierto por `#![cfg_attr(test, allow(...))]` de `src/lib.rs`, sin marca extra.
  - `src/index/graph.rs` / `src/index/ivf.rs` — VEREDICTO: los 15+8 `expect` visibles son todos de módulos `#[cfg(test)]` (líneas >1200 en graph.rs; `expect("test vectors...")`) → cubiertos por cfg_attr de lib.rs.
- **Acción:** `#[allow(clippy::expect_used)]` en función o scope de línea
- **Verify:** `cargo clippy -p vantadb --all-targets --all-features -- -D warnings`
- **Estado:** ✅ DONE — exit 0.

### Step 2b (colateral descubierto en ejecución): workspace lints deny → sweep de allows
- **Hallazgo:** la premisa "tests excluidos por default" era FALSA — `clippy::unwrap_used/expect_used` aplica también a código `#[cfg(test)]` en targets `--all-targets`.
- **Acción:** sweep de 2 líneas `#![allow(clippy::expect_used, clippy::unwrap_used)]` en ~170 archivos (tests/, benches/, examples/, benchmarks/graphrag_bench.rs) + `#![cfg_attr(test, allow(...))]` en lib.rs de vantadb, vanta-memory, vanta-proxy, vantadb-wasm, vantadb-mcp + allows puntuales con rationale en prod: vanta-memory (mock.rs lock-poison, token_estimator peek, merge last_mut), vantadb-mcp/src/server.rs (ServerInfo Serialize infallible).
- **Estado:** ✅ DONE.

### Step 3: Convertir bins a `anyhow::Result` con `.context()`
- **Archivos:** `src/bin/vanta-cli.rs` (no existe `src/main.rs`), `vantadb-server/src/main.rs`
- **Acción ejecutada:**
  - root `Cargo.toml`: `anyhow = { version = "1", optional = true }` gated tras feature `cli` (vanta-cli required-features=["cli"]) → la lib no lo jala si no es cli; nunca usado en `src/lib.rs` (regla).
  - `vantadb-server/Cargo.toml`: `anyhow = "1"`.
  - `vanta-cli.rs`: `fn main() -> anyhow::Result<()> { run().context("vanta-cli: command failed") }` + `fn run() -> anyhow::Result<()>` (cuerpo intacto); allow del bin removido (sin unwrap/expect en body).
  - `vantadb-server/src/main.rs`: `async fn main() -> anyhow::Result<()>` con `.context("failed to start MCP server")` / `.context("server error")`; exit code 1 preservado; exit(2) de args invalidos intacto.
- **Verify:** `rg -c "anyhow::Result|anyhow!"` → vanta-cli=2, server=1 ✅
- **Estado:** ✅ DONE.

### Step 4: `cargo fmt --all -- --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Verify:** FMT=0, WS_EXIT=0 (verificado 2026-09-02, sesión de reanudación)
- **Estado:** ✅ DONE.

### Step 5: Commit atómico
- **Mensaje:** `chore(clippy): deny unwrap/expect en prod + anyhow bins (ERR-CORE-02)`
- **Estado:** ✅ DONE — ver hash en plan file (commit de codigo) + docs commit posterior.

## Dependencias
- Ninguna (Wave 0)

## Notas
- Tests `#[cfg(test)]` ya están excluidos por default en clippy::unwrap_used/expect_used
- Pre-mortem cubierto: anyhow solo en bins, allow con SAFETY para invariantes
- No tocar `src/lib.rs` (mantener thiserror puro)