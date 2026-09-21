# INV-011: Core-Server Separation — auditoría

**Fecha:** 2026-08-03
**Estado:** ✅ COMPLETADA
**Fuente:** docs/Backlog.md línea 201
**Tipo:** Auditoría (sin implementación)

## Veredicto: separación YA limpia — sin cambios requeridos

El core embebido (`vantadb`) NO tiene dependencias no deseadas del modo servidor.

## Hallazgos

### 1. Features de Cargo.toml — server deps correctamente isoladas

| Feature | Deps | Veredicto |
|---------|------|-----------|
| `server` | `dep:tokio`, `dep:axum`, `dep:tower_governor`, `dep:tower-http` | ✅ Todas optional, detrás de feature |
| `tls` | `dep:axum-server`, `dep:rustls` | ✅ Optional |
| `opentelemetry` | `opentelemetry_sdk/rt-tokio`, `opentelemetry-otlp` (grpc-tonic) | ✅ Optional |
| `prometheus` | `dep:prometheus` | ✅ Optional |
| `async-ingestion` / `async-io` | `dep:tokio` | ✅ Optional |

### 2. default features NO incluyen server deps

```toml
default = ["cli", "arrow", "fjall", "roaring", "advanced-tokenizer", "memmap2", "fs2", "sysinfo"]
```

✅ Ninguna de axum/tower/tokio/rustls/opentelemetry en default.

### 3. Imports server-only en src/ están gated

- `src/cli_server.rs` (axum, tower_governor, tower-http, tokio) → gated por `#[cfg(feature = "server")]` en `lib.rs:72-73`
- `src/circuit_breaker.rs`, `src/connection_pool.rs` → `#[cfg(feature = "server")]` en `lib.rs:66-79`
- `src/cli_handlers/server.rs` → módulo gated por `cli`, handlers server con `#[cfg(feature = "server")]` internos (188, 207)
- **Ningún import axum/tower en módulos core no-gated** (engine, storage, sdk, wal, index)

### 4. Verificación mecánica

```bash
# Deps normales con -F cli (sin dev-deps): CERO axum/tower/hyper/tokio/tonic/rustls/opentelemetry
cargo tree --no-default-features -F cli -e normal -p vantadb | grep -E "axum|tower|tokio|..."   # → vacío
# Build cli-only compila limpio (exit 0, sin deps server):
cargo check -p vantadb --no-default-features -F cli   # ✅ EXIT 0
```

El único `tokio` que apareció en el tree completo era dev-dependency de `criterion` (bench) — correcto.

## Observación menor (no bloqueante)

`server = ["cli", ...]` acopla server→cli (el feature server requiere cli). Es intencional: `vantadb-server` y el subcomando `vanta-cli server` asumen CLI (config parsing, console). No es una deps no deseada del core; es una dependencia de features del wrapper server. Puede separarse en el futuro si existiera un consumidor server-only sin CLI, pero YAGNI hoy.

## Recomendación

**No cambiar nada.** La separación core vs server ya está limpia con feature gates. Proseguir normal.