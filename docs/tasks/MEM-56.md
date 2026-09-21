# Task 10 — MEM-56: Hook Langfuse/OTLP sobre ReportHook

## Estado: ✅ COMPLETED

## Contrato (del plan P33)
"tests D19 con collector mockeado: turno → spans emitidos; disabled default; fallo de red nunca bloquea proxy (P4)"

## Steps
- ✅ S1: Config `[report]` TOML (`langfuse_endpoint`, `langfuse_auth_header`) en config.rs — off by default (`ReportConfig::enabled()`)
- ✅ S2: `vanta-proxy/src/langfuse.rs` — OTLP-JSON manual (serde_json) + worker thread con reqwest blocking + `langfuse_hook() -> Option<ReportHook>`
- ✅ S3: Wiring en `server.rs::from_engine` (registro condicional) + 4 tests D19
- ✅ S4: Verify mecánico completo

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-proxy/src/report.rs` (158L verbatim) — `TurnReport`, `ReportHook`, `Reporter::add_hook/emit`
- `vanta-proxy/src/config.rs` (125L) — patrón `#[serde(default)]` por sección
- `vanta-proxy/src/server.rs` (`AppState::from_engine` verbatim) — wiring único del Reporter
- `docs/plans/2026-08-22-vanta-ultima-milla.md:153-161` — Task 10 spec
- `vanta-proxy/Cargo.toml` — reqwest 0.12 presente; se agregó feature `blocking`

**Referencias entrantes:** `add_hook` (solo tests), `Reporter` (solo server.rs::from_engine), `emit` (server.rs::process + desktop lib.rs — desktop crea su propio Reporter sin config, cero impacto).

**Veredicto:** BAJO — cambio aditivo; TOMLs existentes sin `[report]` siguen parseando.

## Decisión de implementación (ponytail, documentada)
**OTLP-JSON manual sin SDK.** OTLP/HTTP acepta JSON (`resourceSpans[]` shape fijo); `serde_json::json!` + reqwest POST cubre el contrato completo sin opentelemetry-sdk (~30 crates transitivas).
**P4:** hook = `mpsc::Sender::send` (unbounded); worker thread dedicado hace POST blocking timeout 5s; fallo → `tracing::warn!`.

## Verificación (S4)
- `cargo check -p vanta-proxy --all-targets` ✅
- `cargo test -p vanta-proxy` ✅ 89 passed (65→69 unit con 4 langfuse nuevos + 20 integration)
- `cargo fmt --check` ✅
- `cargo clippy -p vanta-proxy --all-targets --no-deps -- -D warnings` ✅ (7 warnings pre-existentes en core `vantadb`, fuera del crate)

## Tests D19 (langfuse.rs)
1. `otlp_payload_has_one_span_with_turn_attributes` — shape OTLP válido, attrs del turno
2. `turn_is_exported_as_otlp_span_to_mock_collector` — collector HTTP mockeado (TcpListener) recibe el span
3. `disabled_when_no_endpoint_configured` — default off, hook None
4. `network_failure_returns_err_without_blocking` — endpoint caído → Err rápido + emit nunca bloquea
