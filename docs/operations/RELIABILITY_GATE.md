# VantaDB Reliability Gate & Certification Policy

Este documento consolida la política de certificación operativa, los umbrales de aceptación (gating) y los procedimientos prácticos para validar la resiliencia de VantaDB bajo estrés de memoria, inyección de fallos catastróficos y corrupción de datos.

---

## Sección 1: RSS Stability Gate (Confined Heap Memory)

Establece el procedimiento operativo para validar la estabilidad de la memoria física (RSS) de VantaDB bajo cargas sostenidas y certificar que el asignador global `mimalloc` mitiga la fragmentación del heap a largo plazo (Criterio de Aceptación ST2.2.2).

### 1.1 Propósito

El test rápido de CI valida la ausencia de fugas catastróficas en caliente. Sin embargo, para certificar formalmente la mitigación de la fragmentación a largo plazo bajo ciclos dinámicos del motor de almacenamiento, se requiere una ejecución continua durante **30 minutos**.

### 1.2 Requisitos Previos

Compilar el ejecutable o biblioteca con la feature `custom-allocator` habilitada para activar `mimalloc`:

```powershell
cargo build --release --features custom-allocator
```

### 1.3 Procedimiento de Ejecución Manual

El script ejecuta un bucle continuo de inserciones y lecturas masivas a través de la interfaz del SDK de Python, evaluando la deriva del Resident Set Size (RSS) a través del tiempo.

#### Código del Test (Guarda en `tests/stress_rss_30m.py`)

```python
import time
import os
import psutil
import vantadb_py as vanta

def run_stress_test(duration_minutes=30):
    print(f"Iniciando test de estrés RSS por {duration_minutes} minutos...")
    db_path = "./temp_stress_db"
    if not os.path.exists(db_path):
        os.makedirs(db_path)
        
    db = vanta.VantaDB(db_path)
    process = psutil.Process(os.getpid())
    rss_initial = process.memory_info().rss
    print(f"RSS Inicial: {rss_initial / 1024 / 1024:.2f} MB")
    
    start_time = time.time()
    end_time = start_time + (duration_minutes * 60)
    count = 0
    while time.time() < end_time:
        for i in range(1000):
            vector = [1.0] * 128
            db.insert(count, f"content_{count}", vector)
            count += 1
            
        db.flush()
        profile = db.hardware_profile()
        current_rss = profile["process_rss_bytes"]
        elapsed = (time.time() - start_time) / 60
        print(f"[{elapsed:.2f} min] RSS actual: {current_rss / 1024 / 1024:.2f} MB | Nodos: {count}")
        time.sleep(1)
        
    rss_final = process.memory_info().rss
    drift = (rss_final / rss_initial) - 1.0
    print("\n" + "="*40)
    print("TEST FINALIZADO")
    print(f"RSS Inicial: {rss_initial / 1024 / 1024:.2f} MB")
    print(f"RSS Final: {rss_final / 1024 / 1024:.2f} MB")
    print(f"Drift Residual de RSS: {drift * 100:.2f}%")
    print("="*40)
    
    if drift < 0.10:
        print("✅ Certificación Exitosa: Crecimiento residual de RSS inferior al 10%.")
    else:
        print("❌ Certificación Fallida: Fuga o fragmentación excesiva detectada.")

if __name__ == "__main__":
    run_stress_test(30)
```

### 1.4 Umbrales de Aceptación (Gating)

1. **Drift de RSS < 10%** medido entre la estabilización de memoria del minuto 5 (tras el warm-up) y el final en el minuto 30.
2. **Coherencia de Fragmentación**: La memoria HNSW lógica (`hnsw_logical_bytes`) y la memoria residente mapeada física (`mmap_resident_bytes`) deben reflejar estabilidad en RAM, sin crecimiento exponencial o divergente del RSS global del proceso.

---

## Sección 2: Chaos Integrity Gate (Fault-Injection and Recovery)

Garantiza que VantaDB es tolerante a fallos catastróficos inyectados de E/S de disco, memoria y serialización de índice, asegurando una auto-recuperación 100% libre de corrupción y sin pérdida de consistencia ácida.

### 2.1 Puntos de Caos Instrumentados (Failpoints)

VantaDB instrumenta inyecciones de error discretas controladas mediante la feature-flag `failpoints`:

| Nombre del Failpoint | Ubicación en Código | Comportamiento Simulado |
| :--- | :--- | :--- |
| `wal_append_fail` | `src/wal.rs` | Error al escribir registros en el Write-Ahead Log. |
| `storage_insert_fail` | `src/storage.rs` | Error catastrófico de E/S al insertar en las estructuras de almacenamiento. |
| `mmap_flush_fail` | `src/storage.rs` | Error al sincronizar con disco (`msync` / `flush`) el archivo mapeado en memoria. |
| `hnsw_serialize_fail` | `src/index.rs` | Error de E/S al serializar y persistir en disco el índice vectorial HNSW. |

### 2.2 Procedimientos de Verificación

#### A. Verificación Rápida de CI (Nextest)

Para ejecutar de forma rápida las pruebas de caos en entornos de CI o pre-push:

```powershell
cargo nextest run --profile chaos --features failpoints
```

#### B. Certificación de Loop de Caos Manual (Resiliencia Sostenida)

Para validar la ausencia de fugas, bloqueos mutuos o corrupción residual en ejecuciones repetitivas de inyección de errores, se corre el script de loop de caos:

```powershell
.\dev-tools\chaos_loop.ps1 -Iterations 1000 -Release
```

### 2.3 Umbrales de Aceptación (Gating)

1. **Ratio de Éxito: 100.00%** sobre 1,000 iteraciones (cero fallos no interceptados).
2. **Garantía de Auto-Recuperación**: Toda operación que retorne `Err` durante la inyección del fallo debe ejecutarse de forma exitosa (`Ok`) inmediatamente después de desactivar el failpoint.
3. **Consistencia Transaccional**: El estado del motor debe ser legible y correcto, recuperando todos los datos confirmados previo al fallo.

---

## Sección 3: Durabilidad WAL y Recuperación Fría (Durability and Cold-Start)

Valida que ante una detención forzada del proceso de la base de datos (crash abrupto), los datos confirmados en el WAL no se pierdan y el motor se reconstruya automáticamente a su último estado consistente conocido.

### 3.1 Suite de Validación

La resiliencia de durabilidad del WAL y el cold-start se verifica mediante las suites dedicadas:

- **`tests/storage/wal_resilience.rs`**: Certifica la integridad del parser del WAL, sumas de comprobación CRC32C, y la recuperación ante fragmentos truncados.
- **`tests/durability_recovery.rs`**: Certifica la reconstrucción fría del índice vectorial y relacional a partir de los registros del WAL tras fallos simulados del proceso.

### 3.2 Comandos de Ejecución

Para certificar de forma manual:

```powershell
cargo test --test wal_resilience --release
cargo test --test durability_recovery --release
```

### 3.3 Umbrales de Aceptación (Gating)

1. **Cero fugas de consistencia**: Todos los nodos insertados y confirmados en el WAL antes del apagado abrupto deben ser recuperables vía `get()` después de la reapertura fría (`StorageEngine::open`).
2. **Checksum Integrity**: Cualquier corrupción física introducida en el archivo de WAL debe ser detectada a nivel de página/registro mediante CRC32C, levantando errores de lectura o aplicando auto-reparación hasta el último punto consistente del log.
