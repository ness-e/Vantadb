# DEVEX-DEMO: Demo app (Rust + Python)

## Metadata
- **Plan file:** P8 Post-Launch & Enterprise
- **Fuente:** `docs/Backlog.md:199`
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Tipo:** Rust + Python (Mixto)
- **Turns estimados:** 15-25
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | Developers/Users que prueban VantaDB |
| Callees | `vantadb` crate + Python SDK (`vantadb_py`) |
| Implicaciones | Solo nuevos archivos en `examples/demo/` o similar. Sin cambios en core. |

## Contrato
"Demo app funcional: un script `demo.py` o `examples/demo/main.rs` que inserta documentos, hace búsqueda híbrida, y demuestra persistencia. README explica cómo correrlo."

## Pasos
1. Decidir ubicación (ej: `examples/demo/`) y stack (Rust standalone o Python)
2. Implementar demo: insertar documentos → search → hybrid → persist
3. README con instrucciones
4. Verificar: `cargo run --example demo` o `python demo.py` funciona
