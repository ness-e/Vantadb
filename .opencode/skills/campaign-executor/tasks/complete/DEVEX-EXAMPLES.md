# DEVEX-EXAMPLES: Rust examples en examples/rust/

## Metadata
- **Plan file:** P8 Post-Launch & Enterprise
- **Fuente:** `docs/Backlog.md:200`
- **Esfuerzo:** 🟢 4-6h
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 8-12
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | Developers leyendo examples (no production code) |
| Callees | `vantadb` crate API pública: `VantaEmbedded`, `VantaConfig`, `put`, `search`, etc. |
| Implicaciones | Solo nuevos archivos. Sin impacto en código existente. Los examples sirven como documentación viva del API. |

## Contrato
"`cargo build --example basic` compila. `cargo build --example hybrid` compila. Al menos 3 examples funcionales."

## Pasos
1. Identificar qué examples ya existen en `docs/examples/` vs target `examples/rust/`
2. Crear `examples/rust/` directory
3. Implementar basic example (insert + search)
4. Implementar hybrid example (dense + sparse)
5. Implementar graphrag example
6. `cargo check --examples` pasa
