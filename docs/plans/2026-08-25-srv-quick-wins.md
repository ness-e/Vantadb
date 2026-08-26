# Plan: SRV Quick Wins — INV-vantadb-server-01 (2026-08-25)

> Origen: `/research vantadb-server` Fase D — quick wins aprobados por el owner.
> Ejecutar con `/pipeline run docs/plans/2026-08-25-srv-quick-wins.md`.
> Las tareas estructurales (SRV-04..08) quedan en Backlog P40, fuera de este plan.

## Wave 1 — Gates rotos / higiene (bloqueantes)

| ID | Tarea | Contrato verificable | Archivos |
|---|---|---|---|
| AUD-043 | Fix clippy `unused variable: ns` en closure `options_for` → renombrar `_ns` | `cargo clippy -p vantadb --deny warnings` pasa sin ese warning | `src/cli_server.rs:1302` |
| MOD-15 | Nits agrupados: eliminar `middleware.rs` re-export redundante (verificar consumidores antes), resolver feature `sysinfo=[]` vacía, comentario ensure-indexes en `main.rs` MCP path, constructor `ServerState::new` para tests | `just verify-quick` pasa; grep sin referencias a `vantadb_server::middleware` | `vantadb-server/src/middleware.rs`, `vantadb-server/Cargo.toml`, `vantadb-server/src/main.rs` |

## Wave 2 — Quick wins server (independientes, paralelizables)

| ID | Tarea | Contrato verificable | Archivos |
|---|---|---|---|
| SRV-01 | Rotación/retención audit log JSONL: rotación por tamaño configurable (`audit_max_bytes`, default p.ej. 10MB) con rename `.1/.2` + cap de archivos; test de rotación y de query post-rotación | Test nuevo en `tests/server.rs` verde: escribir > límite → archivo rota; `GET /api/v2/audit` sigue sirviendo el activo | `src/audit.rs`, `src/cli_server.rs` |
| SRV-02 | Tracing-id por request: leer primer match de `x-request-id`/`x-tracing-id`/`traceparent` (≤256 chars, truncar) en middleware de métricas, incluirlo en `AuditEvent` y en span tracing | Test e2e: request con `x-request-id: abc` → evento audit lo contiene | `src/cli_server.rs:860-908`, `src/audit.rs` |
| SRV-03 | Verificar docs instalación apuntan a GitHub Release binaries (crate publish=false); corregir README/QUICKSTART si dicen `cargo install vantadb-server` o crates.io | Grep repo sin instrucciones de instalación vía crates.io para el server | README, `docs/QUICKSTART.md`, `docs/api/HTTP_API.md` |

## Reglas

- Conventional commits por tarea (`fix:`/`chore:`/`docs:`).
- Nada de esto toca API pública del SDK → sin semver-checks obligatorio.
- Regla 6 (deuda neta): SRV-01/02 eliminan deuda existente (audit infinito, falta correlación) — saldo negativo.
