# Plan de Implementación: Fase SCALE-01c — Benchmark Comparativo Pre/Post Scaling

Este plan detalla el diseño y la ejecución del benchmark comparativo para cuantificar de forma determinista y empírica la optimización de latencia en la búsqueda HNSW MMap introducida por el prefetching predictivo (`PrefetchVirtualMemory`/`madvise`).

## User Review Required

> [!IMPORTANT]
> Para habilitar la comparación directa de rendimiento de forma justa bajo el mismo entorno de ejecución, introduciremos una variable de entorno de control interna: `VANTA_DISABLE_PREFETCH=1`. Esto nos permite desactivar dinámicamente el prefetch en el bucle caliente de búsqueda HNSW sin alterar la firma pública de las APIs de Rust ni el SDK de Python, facilitando un benchmark A/B riguroso.

## Proposed Changes

### ⚙️ Engine Instrumentation

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
*   Declarar un mecanismo estático ultra-rápido basado en `AtomicBool` para verificar si el prefetch predictivo ha sido desactivado a través de la variable de entorno `VANTA_DISABLE_PREFETCH`.
*   Interpolar este chequeo en el hot-path del HNSW (`search_layer`) antes de emitir las sugerencias del prefetch del kernel.

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static PREFETCH_ENABLED: AtomicBool = AtomicBool::new(true);
static PREFETCH_INIT: AtomicBool = AtomicBool::new(false);

#[inline(always)]
fn should_prefetch() -> bool {
    if !PREFETCH_INIT.load(Ordering::Relaxed) {
        let enabled = std::env::var("VANTA_DISABLE_PREFETCH").is_err();
        PREFETCH_ENABLED.store(enabled, Ordering::Relaxed);
        PREFETCH_INIT.store(true, Ordering::Relaxed);
        enabled
    } else {
        PREFETCH_ENABLED.load(Ordering::Relaxed)
    }
}
```

---

### 📊 Benchmark Automation Suite

#### [NEW] [prefetch_comparison.py](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/benchmarks/prefetch_comparison.py)
*   Crear un script de Python dedicado para automatizar el test A/B de prefetching:
    1.  Generar un dataset de vectores sintéticos de tamaño medio/alto (e.g. 50K vectores de 384 dimensiones) para asegurar un tamaño de index mapeado en disco de ~80 MB.
    2.  Ingresar y persistir el dataset en VantaDB mediante `VantaFile` (modo archivo persistente).
    3.  Forzar un `db.flush()` y cerrar el motor para congelar el estado físico en disco.
    4.  **Test A (Sin Prefetch)**:
        *   Establecer la variable de entorno `VANTA_DISABLE_PREFETCH=1`.
        *   Abrir la base de datos de manera limpia.
        *   Ejecutar `1,000` consultas de búsqueda semántica de forma secuencial y cronometrar las latencias de búsqueda (p50, p95, p99, y media).
        *   Cerrar la base de datos.
    5.  **Test B (Con Prefetch)**:
        *   Remover o limpiar la variable de entorno `VANTA_DISABLE_PREFETCH`.
        *   Abrir la base de datos de manera limpia.
        *   Ejecutar el mismo set de `1,000` consultas secuenciales y cronometrar las latencias de búsqueda (p50, p95, p99, y media).
        *   Cerrar la base de datos.
    6.  Generar una tabla comparativa en formato Markdown comparando las latencias y calculando el porcentaje de reducción en p99.
    7.  Actualizar la sección de rendimiento o guardar el reporte en `docs/BENCHMARKS.md`.

---

## Verification Plan

### Automated Tests
*   Ejecutar el benchmark manualmente para verificar la instrumentación y capturar la tabla de datos inicial:
    ```powershell
    # Compilar los Python bindings actualizados
    maturin develop --manifest-path vantadb-python/Cargo.toml --release
    
    # Ejecutar el script comparativo
    python benchmarks/prefetch_comparison.py --size 50000 --dim 384 --queries 1000
    ```
*   Verificar que no se introducen regresiones de compilación:
    ```powershell
    cargo check --all-targets
    ```
