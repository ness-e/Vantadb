# Plan de Implementación: Fase SERV-01 (Desacoplamiento de Tokio y Red del Core)

Este plan detalla el desacoplamiento físico y lógico de dependencias asíncronas pesadas (`tokio`) y de red (`reqwest`) del core de producción de VantaDB. Se consolida el motor core (`vantadb`) como una base de datos embebida 100% síncrona en producción, reubicando runtime asíncronos y llamadas externas a entornos aislados o a la feature de inferencia `remote-inference` (antigua `llm`).

---

## User Review Required

> [!IMPORTANT]
> **Core 100% Síncrono en Producción:** Se remueve `tokio` por completo de la sección de `[dependencies]` del `Cargo.toml` raíz. A partir de ahora, el core de producción compilará libre de dependencias asíncronas pesadas, mejorando drásticamente el tamaño del binario y eliminando la deuda técnica de runtimes no deseados en enlazados embebidos (ej. FFI con Python o WebAssembly).
> 
> **Suite de Tests Intacta:** `tokio` se mantendrá bajo `[dev-dependencies]` con su conjunto completo de características (`rt-multi-thread`, `macros`, `time`, etc.). De este modo, los tests de integración de concurrencia, resiliencia y el pipeline asíncrono del servidor seguirán funcionando sin ninguna alteración.
> 
> **Feature `remote-inference` (Ex-`llm`):** La feature de comunicación con el puente de inferencia (Ollama) para embeddings y resúmenes semánticos (`src/llm.rs`) se renombra a `remote-inference` para mayor coherencia de diseño. Seguirá siendo opcional y dependerá de `reqwest`, aislando completamente las capacidades de red de VantaDB.

---

## Open Questions

> [!NOTE]
> No hay preguntas bloqueantes identificadas. El análisis de impacto demuestra que el core no requiere de un runtime asíncrono para ejecutar lecturas, escrituras, ni reconstrucciones de índices. El servidor (`vantadb-server`) y los tests de integración seguirán proveyendo su propio runtime Tokio sin generar fricción en el motor embebido local.

---

## FMEA (Análisis de Modos de Fallo y Mitigación)

A continuación se detalla la matriz FMEA para esta refactorización, evaluando el impacto estratégico e ingenieril de los cambios:

| Modo de Fallo Potencial (FMEA) | Gravedad | Causa Raíz | Estrategia de Mitigación / Diseño |
| :--- | :--- | :--- | :--- |
| **Falla en el build del servidor (`vantadb-server`)** | **Alta** | El servidor monta rutas asíncronas de Axum y un runtime de Tokio. Si el core no incluye Tokio, podría haber problemas si el servidor asume que el core re-exporta tipos de Tokio. | **Mitigación Pasiva:** El `Cargo.toml` de `vantadb-server` ya incluye de forma directa su propia dependencia de `tokio` con features `full`. No existe acoplamiento de tipos asíncronos en los contratos del core. |
| **Pánicos o fallas de compilación en tests de integración** | **Media** | Los tests de integración (`tests/`) validan concurrencia y recuperaciones asíncronas. Si se remueve Tokio, la suite fallaría al compilar. | **Mitigación Activa:** Reubicar `tokio` en `[dev-dependencies]` del core. De esta forma, el compilador solo inyecta Tokio al ejecutar `cargo test`, manteniendo los builds de producción (`cargo build --release`) libres del runtime. |
| **Inconsistencias en features del Workspace** | **Baja** | Módulos opcionales como `experimental-governance` dependen de `llm` para tareas en segundo plano. Si se renombra a `remote-inference`, se generaría un fallo en cascada de compilación. | **Mitigación de Dependencias:** Modificar el `Cargo.toml` de `experimental-governance` para mapear correctamente la feature hacia `vantadb/remote-inference` y actualizar los condicionales en el código. |

---

## Proposed Changes

### [Workspace Configuration]

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
*   Remover de `[dependencies]` la entrada de `tokio`:
    ```diff
-   tokio = { version = "1", features = ["sync", "rt"] }
    ```
*   Verificar que `[dev-dependencies]` mantenga Tokio completo:
    ```toml
    tokio = { version = "1", features = [
        "rt-multi-thread",
        "sync",
        "time",
        "fs",
        "macros",
        "io-util",
    ] }
    ```
*   Renombrar la feature `llm` a `remote-inference` en la sección `[features]`:
    ```diff
-   llm = ["reqwest"]
+   remote-inference = ["reqwest"]
    ```

---

### [Crate Core: vantadb]

#### [MODIFY] [src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/lib.rs)
*   Actualizar la declaración condicional de importación para usar la nueva feature:
    ```diff
-   #[cfg(feature = "llm")]
+   #[cfg(feature = "remote-inference")]
    pub mod llm;
    ```

#### [MODIFY] [src/executor.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs)
*   Reemplazar todos los condicionales de features para inyección de dependencias remotas:
    ```diff
-   #[cfg(feature = "llm")]
+   #[cfg(feature = "remote-inference")]
    ```
    y
    ```diff
-   #[cfg(not(feature = "llm"))]
+   #[cfg(not(feature = "remote-inference"))]
    ```

---

### [Crate: experimental-governance]

#### [MODIFY] [packages/experimental-governance/Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-governance/Cargo.toml)
*   Actualizar la feature local para depender de la nueva feature del core:
    ```diff
-   llm = ["vantadb/llm"]
+   remote-inference = ["vantadb/remote-inference"]
    ```

#### [MODIFY] [packages/experimental-governance/src/maintenance_worker.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/packages/experimental-governance/src/maintenance_worker.rs)
*   Reemplazar las directivas condicionales `#[cfg(feature = "llm")]` y `#[cfg(not(feature = "llm"))]` por `remote-inference`.

---

## Verification Plan

### Automated Tests (Solicitado al Usuario)

Para verificar que el desacoplamiento es óptimo y que no se ha introducido ninguna regresión, te solicitaré ejecutar los siguientes comandos y enviarme la salida:

1.  **Auditoría de Dependencias del Core (Compilación de Producción):**
    `cargo tree -p vantadb --no-default-features`
    *Deberá confirmar que no se incluye 'tokio' ni 'reqwest' en el árbol de dependencias del motor embebido.*

2.  **Verificación de Compilación General:**
    `cargo check --workspace --all-targets`
    *Garantiza que tanto el core, la CLI, el servidor asíncrono y los subcrates en cuarentena compilan sin advertencias.*

3.  **Ejecución de la Suite de Tests:**
    `cargo test --lib`
    *Asegura la estabilidad y consistencia de todos los tests locales en verde.*
