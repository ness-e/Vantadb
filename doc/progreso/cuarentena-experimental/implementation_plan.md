# Plan de Implementación: Fase CUARENTENA-01 (Aislamiento de Código Experimental)

Este plan detalla el desacoplamiento físico y lógico de los módulos experimentales de LISP (`src/eval/`, `src/parser/lisp.rs`) y Gobernanza (`src/governance/`, `src/governor.rs`) del core de VantaDB, trasladándolos a subcrates dedicados dentro del workspace de Cargo. Esto asegura que el core se estabilice para producción y se prepare para el nuevo planificador (Fase PLANNER-02), preservando a la vez el código experimental intacto para su futura mejora o reintegración.

---

## User Review Required

> [!IMPORTANT]
> **No se elimina código histórico:** Todo el código fuente experimental de LISP y Gobernanza se mantendrá intacto y compilable dentro del directorio `packages/`.
> 
> **Desactivación de Features del Core:** Se removerán las features `experimental` and `governance` del `Cargo.toml` raíz. Los campos condicionales en `StorageEngine` y `Executor` se deshabilitarán o aislarán para que no influyan en la ruta crítica del core.
> 
> **Frontera de Consulta LISP en el Core:** Si se intenta ejecutar una consulta que comience con `(` (sintaxis LISP) a través de la API del core, se retornará un error semántico limpio indicando que se requiere el uso de la extensión experimental.

---

## Open Questions

> [!NOTE]
> No hay preguntas abiertas bloqueantes. La modularización física mediante subcrates locales bajo `packages/` resuelve limpiamente el requerimiento sin alterar la semántica ni el histórico de control de versiones.

---

## Proposed Changes

### [Workspace Configuration]

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
*   Remover del `Cargo.toml` de la raíz las features: `experimental`, `eval`, `parser`, `governance`.
*   Agregar en la sección `[workspace.members]`: `"packages/experimental-lisp"` y `"packages/experimental-governance"`.
*   Depurar dependencias exclusivas de pruebas y módulos experimentales que ya no se usen en el core estable.

---

### [Crate Core: vantadb]

#### [MODIFY] [src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/lib.rs)
*   Remover los módulos declarados condicionalmente bajo las features eliminadas:
    ```diff
-   #[cfg(feature = "experimental")]
-   pub mod eval;
-   #[cfg(feature = "experimental")]
-   pub mod executor;
-   #[cfg(feature = "governance")]
-   pub mod governance;
-   #[cfg(feature = "experimental")]
-   pub mod graph;
-   #[cfg(feature = "experimental")]
-   pub mod parser;
    ```
*   Remover las re-exportaciones de tipos experimentales no soportados por el core estable en el bloque de `pub use`.

#### [MODIFY] [src/parser/mod.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/parser/mod.rs)
*   Remover la línea `pub mod lisp;`. El parser de LISP se compilará únicamente dentro del nuevo subcrate dedicado.

#### [MODIFY] [src/executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs)
*   Remover las importaciones directas de `crate::eval::LispSandbox` y `crate::parser::lisp::parse`.
*   Modificar el método `execute_hybrid` para que devuelva un error controlado en tiempo de ejecución ante sintaxis de LISP:
    ```rust
    let trimmed = query_string.trim_start();
    if trimmed.starts_with('(') {
        return Err(VantaError::Execution(
            "LISP queries require the experimental-lisp extension/crate.".to_string(),
        ));
    }
    ```
*   Remover o encapsular limpiamente la lógica de gobernanza dependiente de `admission_filter` o `conflict_resolver`.

#### [DELETE] [src/eval/mod.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/eval/mod.rs) (Físicamente movido a cuarentena)
#### [DELETE] [src/eval/vm.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/eval/vm.rs) (Físicamente movido a cuarentena)
#### [DELETE] [src/parser/lisp.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/parser/lisp.rs) (Físicamente movido a cuarentena)
#### [DELETE] [src/governance/](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/governance) (Directorio completo físicamente movido a cuarentena)

---

### [Crate: experimental-lisp]

#### [NEW] [packages/experimental-lisp/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-lisp/Cargo.toml)
*   Definir el nuevo subcrate local `experimental-lisp` con su manifiesto de Cargo.
*   Incluir dependencias requeridas para la compilación (como `nom`, `serde`, `rand`, y la referencia local al core `vantadb` si es necesaria para estructurar el AST en el futuro).

#### [NEW] [packages/experimental-lisp/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-lisp/src/lib.rs)
*   Alojar el código del parser de LISP (`lisp.rs`), la máquina virtual (`vm.rs`) y la lógica de ejecución del sandbox que antes residía en `src/eval/`.

---

### [Crate: experimental-governance]

#### [NEW] [packages/experimental-governance/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-governance/Cargo.toml)
*   Definir el nuevo subcrate local `experimental-governance` con su manifiesto de Cargo.
*   Declarar dependencias necesarias (como `serde`, `chrono`, etc.).

#### [NEW] [packages/experimental-governance/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-governance/src/lib.rs)
*   Reunir el código de gobernanza experimental, filtros de admisión, arbitraje de confianza, trabajadores de mantenimiento y consistencia.

---

## Verification Plan

### Automated Tests (Solicitado al Usuario)
Para validar que el desacoplamiento es exitoso, le pediremos al usuario que ejecute los siguientes comandos en su terminal y nos provea el resultado:

1.  **Validación del Core:**
    `cargo check --lib`
    *Debe compilar al 100% sin advertencias ni errores relacionados con LISP o Gobernanza.*

2.  **Ejecución de Pruebas Unitarias/Integración del Core:**
    `cargo test --lib`
    *Debe validar todas las invariants estables del motor síncrono.*

3.  **Validación de los Subcrates en Cuarentena:**
    `cargo check -p experimental-lisp`
    `cargo check -p experimental-governance`
    *Debe confirmar que el código aislado compila de manera independiente sin errores.*

### Manual Verification
*   Inspección visual de la estructura del árbol de directorios para verificar que el core `src/` no posea rastros de `eval/` o `governance/`.
