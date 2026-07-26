# CLI-01: CLI polish — handlers no conectados al binary

## Metadata
- **Plan file:** P8 Post-Launch & Enterprise
- **Fuente:** `docs/Backlog.md:196`
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 20-30
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `src/bin/vanta-cli.rs` (binary entry point) |
| Callees | `src/cli_handlers.rs`, `src/cli.rs`, `src/storage.rs` |
| Implicaciones | CLI commands de backup/restore/doctor/stats/inspect no disponibles. `vanta-cli help` no los muestra. |

## Contrato
"`cargo build -p vantadb --features cli && ./target/debug/vanta-cli --help` muestra backup, restore, doctor, stats, inspect. Al menos 3 comandos funcionan end-to-end."

## Pasos
1. Investigar qué handlers existen en `cli_handlers.rs` vs qué commands están registrados en el binary
2. Conectar handlers faltantes al CLI dispatcher
3. Agregar tests CLI
4. `cargo check -p vantadb --features cli && cargo clippy -p vantadb --features cli -- -D warnings`
5. `cargo nextest run --features cli`
