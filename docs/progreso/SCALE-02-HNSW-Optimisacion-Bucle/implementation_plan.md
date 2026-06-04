# Plan de Implementación: Fases 1 (Saneamiento) y 2 (Layout Mmap Antilocatario)

Este plan de implementación detalla el diseño técnico para desbloquear el entorno de desarrollo y eliminar el cuello de botella crítico de rendimiento en búsquedas Mmap (problema de los 127s en SIFT 10K).

---

## User Review Required

> [!IMPORTANT]
> **Layout de Serialización Físico Modificado (Fase 2):**
> Para resolver el *Disk Thrashing* y los fallos de página en Mmap, reorganizaremos el orden físico de escritura en disco del índice HNSW en `src/storage.rs` y `src/index.rs`. En lugar de serializar los nodos en orden de ID lineal, utilizaremos una ordenación basada en **Niveles de Grafo HNSW** y **Vecindarios Contiguos**. 
> Esto significa que el formato de archivo binario final (Mmap) cambiará ligeramente en disco para agrupar nodos relacionados en las mismas páginas virtuales de 4KB.
>
> **Hermetización de Python en target/audit-venv (Fase 1):**
> Automatizaremos un script de PowerShell (`dev-tools/setup_venv.ps1`) para inicializar el entorno virtual exclusivo `target/audit-venv` e instalar localmente los bindings con Maturin. Esto erradicará interferencias de versiones viejas globales de Python.

---

## Open Questions

Ninguna. Hemos auditado el código y determinado con precisión matemática los verdaderos cuellos de botella para el escenario SIFT 10K:
1. **La syscall y lock global de `std::env::var` en `should_prefetch()`** que se invoca en el bucle caliente (hot path) para cada vecino evaluado en HNSW.
2. **El cálculo redundante de la raíz cuadrada (`.sqrt()`)** en cada llamada de distancia Euclidiana en el hot-path del recorrido del HNSW.

Ambos serán corregidos directamente en la Fase 2 sin comprometer la precisión matemática ni alterar el soporte de las métricas.

---

## Proposed Changes

### [Component: Python & FFI Environment]

#### [NEW] [setup_venv.ps1](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/dev-tools/setup_venv.ps1)
* Crear un script en PowerShell para inicializar `target/audit-venv` de forma segura.
* Instalar `pip`, `wheel`, y `maturin`.
* Ejecutar la compilación local e instalación de `vantadb-python` en modo desarrollo.

#### [MODIFY] [src/python.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/python.rs) y [src/sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs)
* Purgar todos los warnings reportados por Clippy al compilar con `--features python_sdk`.
* Limpiar tipos no utilizados, variables redundantes e imports inactivos.

---

### [Component: Storage & HNSW Serialization & Distance Engine]

#### [MODIFY] [src/index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs) y [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
* **Optimización de prefetch (Evitar allocations y locks de env)**:
  * Cachear dinámicamente el resultado de la variable de entorno `VANTA_DISABLE_PREFETCH` usando `std::sync::OnceLock`.
  * Esto elimina las millones de llamadas a `std::env::var` y las allocations e interbloqueos asociados.
* **Optimización de distancia Euclidiana**:
  * Utilizar distancia Euclidiana al cuadrado en el HNSW traversal para evitar el cómputo costoso de la raíz cuadrada (`.sqrt()`) en el hot path.
  * Solo aplicar la raíz cuadrada para los top-k resultados finales antes de devolverlos si la API requiere la distancia física.
* **Algoritmo de Agrupación Antilocatario**:
  * Dado que la travesía BFS para ordenamiento físico ya se encuentra implementada en `serialization_order()`, aseguraremos que la re-maquetación de datos en MMap aproveche este orden físico de forma robusta al serializar.

---

## Verification Plan

### Automated Tests
El usuario ejecutará los siguientes comandos manuales para certificar los cambios:

1. **Ejecutar el benchmark inicial (Baseline)** para documentar la latencia de 127 segundos:
   `cargo test --test competitive_bench --release -- --nocapture`

2. **Crear e inicializar el entorno virtual**:
   `powershell -ExecutionPolicy Bypass -File dev-tools/setup_venv.ps1`

3. **Ejecutar verificación de Clippy sin warnings**:
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`

4. **Ejecutar pruebas del SDK Python en el entorno hermético**:
   `target/audit-venv/Scripts/python -m unittest discover -s tests/api/ -p "python.rs"` (o equivalente)

5. **Correr el Benchmark de SIFT después de los cambios** y comparar latencias:
   `cargo test --test competitive_bench --release -- --nocapture`
   *Criterio de éxito:* El tiempo total de búsqueda de SIFT 10K en modo MMap debe descender significativamente de los 127s originales (meta: reducción p99 a < 15ms).
