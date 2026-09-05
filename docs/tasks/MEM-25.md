# MEM-25: vanta-proxy crate + 3 protocolos wire verbatim

## Metadata
- **Plan file:** docs/plans/2026-08-21-vanta-proxy-knowledge.md (Task 4)
- **Creado:** 2026-08-21T00:00
- **last-synced:** 2026-08-21T00:00
- **Estado:** ✅ COMPLETED (cierre por lead: verify mecánico 7/7 wire + fmt/clippy exit 0; sub-agente completó código pero murió antes del cierre)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `Cargo.toml` raíz (workspace members/default-members/lints), `vantadb-server/Cargo.toml` (patrón crate hijo), TDAM `server.ts` (rutas :288-328), TDAM `config.ts` (:9-11 forwardTimeoutMs 600s).
- **Referencias hacia dentro:** ninguna — `vanta-proxy/` NO existe; ningún código depende de él.
- **Referencias entrantes (que este cambio toca):** `Cargo.toml` raíz `[workspace] members` (agregar "vanta-proxy", NO en default-members — patrón vanta-memory/vantadb-server).
- **Veredicto de impacto:** aditivo puro; riesgo único = romper build del workspace → mitigado con `cargo check --workspace` al cierre.
- **Deps:** axum 0.8 / tokio / serde / thiserror / tracing ya en lockfile. reqwest 0.12 (ya en workspace via vantadb/server dev-deps) como cliente upstream con feature `stream`. `toml` nuevo pero exigido por D31 (config TOML).

## Blast Radius
Callers: ninguno (crate binario nuevo). Callees: reqwest→upstream. Implicaciones: workspace members crece en 1; CI fast gate no lo compila (fuera default-members).

## Contrato
"`cargo check -p vanta-proxy` exit 0 · `cargo nextest run -p vanta-proxy` tests (a)-(g) green · `cargo fmt --check` exit 0 · `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings` exit 0 · `cargo check --workspace` exit 0"

Tests D19 (upstream mockeado axum):
(a) /v1/chat/completions forward verbatim body+headers; (b) /v1/messages ídem; (c) /v1/responses ídem (subset genérico); (d) upstream timeout → 504; (e) upstream caído → 502; (f) /health; (g) streaming SSE passthrough sin buffering (PRIORITARIO).

## Herramientas
- terminal cargo, codegraph, campaign MCP

## Steps
### Step 1: Scaffold crate + config TOML + health
- **Archivos:** `vanta-proxy/Cargo.toml`, `src/main.rs`, `src/config.rs`, `src/error.rs`, `src/server.rs`, `config.toml`, Cargo.toml raíz (members)
- **Acción:** crate binario standalone sin deps de vantadb/vanta-memory; config serde+toml (upstream url/apiKey, port, forward_timeout_secs)
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 2: Motor de forwarding (forward.rs)
- **Archivos:** `vanta-proxy/src/forward.rs`
- **Acción:** strip hop-by-hop headers, forward verbatim bytes, timeout configurable, mapeo errores (connect→502, timeout→504), respuesta streaming `Body::from_stream(bytes_stream)` sin buffering
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 3: Handlers 3 protocolos + rutas server.rs
- **Archivos:** `src/handlers/mod.rs`, `src/handlers/openai.rs`, `src/handlers/anthropic.rs`, `src/handlers/responses.rs`, `src/server.rs`
- **Acción:** rutas axum 0.8 `{agent}/{spaceId}/v1/chat/completions`, `{agent}/{spaceId}/v1/messages`, `/v1/responses`, GET `/health`
- **Verify:** `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 4: Tests D19 (a)-(g) con upstream mockeado
- **Archivos:** `vanta-proxy/tests/proxy_wire.rs`
- **Acción:** mock upstream = axum Router sobre TcpListener 127.0.0.1:0; proxy real bind efímero; cliente reqwest
- **Verify:** `cargo nextest run -p vanta-proxy`
- **Estado:** ✅ COMPLETED

### Step 5: Verify mecánico completo + cierre
- **Acción:** fmt/clippy/check workspace/nextest; task file ✅; recitation
- **Verify:** contrato completo
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 1). MEM-26 depende de esta.

## Notas
- axum 0.8: sintaxis params `{param}` (no `:param`).
- SSE passthrough prioritario: reqwest `.send()` devuelve al recibir headers; `bytes_stream()` → `Body::from_stream`. Cero buffering.
- Subset Responses API: SOLO POST genérico `/v1/responses` verbatim — sin adapters Codex/WorkBuddy (research §7). Recorte documentado acá.

## Context Save Point
- **Fecha:** 2026-08-21
- **Branch:** develop (sin commit — regla tarea)
- **Decisiones:** forward engine único compartido por los 3 handlers (verbatim = mismo código); reqwest con rustls-tls+stream
- **Próxima tarea:** Task 5 MEM-30 (ingest)
