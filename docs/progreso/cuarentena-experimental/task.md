# Fase CUARENTENA-01: Aislamiento de Código Experimental

## CUARENTENA-01a: Inicializar subcrates experimentales
- `[x]` Auditar dependencias internas de `src/eval/`, `src/parser/lisp.rs`, `src/governance/`, `src/governor.rs`
- `[x]` Crear `packages/experimental-lisp/Cargo.toml` y `src/lib.rs`
- `[x]` Mover código LISP (eval + parser/lisp.rs + vm.rs) al subcrate
- `[x]` Crear `packages/experimental-governance/Cargo.toml` y `src/lib.rs`
- `[x]` Mover código de gobernanza al subcrate
- `[x]` Registrar ambos subcrates en `[workspace.members]`

## CUARENTENA-01b: Depurar features y dependencias del core
- `[x]` Remover features `experimental`, `eval`, `parser`, `governance`, `executor`, `graph`, `mcp` del `Cargo.toml` raíz
- `[x]` Limpiar `src/lib.rs`: remover módulos condicionales experimentales
- `[x]` Limpiar `src/executor.rs`: remover imports de LISP y gobernanza, sustituir por errores controlados
- `[x]` Limpiar `src/storage.rs`: depurar features del core (inactivar gobernanza por defecto para el core estable)
- `[x]` Limpiar `src/parser/mod.rs`: remover `pub mod lisp`
- `[x]` Eliminar archivos originales del core (`src/eval/`, `src/parser/lisp.rs`, `src/governance/`)

## CUARENTENA-01c: Estabilizar tests del core
- `[x]` Auditar tests que requieren feature `experimental` o `governance`
- `[x]` Ajustar o mover tests afectados
- `[x]` Solicitar al usuario que ejecute `cargo check --lib` y `cargo test`
