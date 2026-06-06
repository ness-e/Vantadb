# Reliability Gate: Test de Estrés RSS Extendido (30 Minutos)

Este documento detalla el procedimiento operativo para ejecutar el test de estrés manual de 30 minutos requerido para certificar la estabilidad de la memoria física (RSS) de VantaDB bajo cargas sostenidas (Criterio de Aceptación ST2.2.2).

## Propósito del Test

El test rápido de CI valida la ausencia de fugas catastróficas en caliente. Sin embargo, para certificar formalmente que el asignador de memoria global `mimalloc` mitiga la fragmentación del heap a largo plazo bajo el ciclo de vida del motor de almacenamiento, se requiere una ejecución continua durante **30 minutos**.

## Requisitos Previos

- Compilación del motor con la feature `custom-allocator` habilitada para activar `mimalloc`:
  ```bash
  cargo build --release --features custom-allocator
  ```

## Script de Prueba Manual

El script ejecuta un bucle continuo de inserciones y lecturas sobre `VantaDB` a través del SDK Python o de la utilidad CLI para evaluar el comportamiento del heap del proceso.

### Código del Test (Python)

Guarda este código en un script (ej. `tests/stress_rss_30m.py`):

```python
import time
import os
import psutil
import vantadb_py as vanta

def run_stress_test(duration_minutes=30):
    print(f"Iniciando test de estrés RSS por {duration_minutes} minutos...")
    
    # Inicializar DB en directorio temporal
    db_path = "./temp_stress_db"
    if not os.path.exists(db_path):
        os.makedirs(db_path)
        
    db = vanta.VantaDB(db_path)
    
    # RSS Inicial
    process = psutil.Process(os.getpid())
    rss_initial = process.memory_info().rss
    print(f"RSS Inicial: {rss_initial / 1024 / 1024:.2f} MB")
    
    start_time = time.time()
    end_time = start_time + (duration_minutes * 60)
    
    count = 0
    while time.time() < end_time:
        # Insertar lote de 1,000 vectores de 128 dims
        for i in range(1000):
            vector = [1.0] * 128
            db.insert(count, f"content_{count}", vector)
            count += 1
            
        # flush periódico para persistir y actualizar telemetría
        db.flush()
        
        # Consultar métricas de perfil hardware en caliente
        profile = db.hardware_profile()
        current_rss = profile["process_rss_bytes"]
        
        elapsed = (time.time() - start_time) / 60
        print(f"[{elapsed:.2f} min] RSS actual: {current_rss / 1024 / 1024:.2f} MB | Nodos insertados: {count}")
        
        time.sleep(1) # pausa breve para regular flujo
        
    # RSS Final
    rss_final = process.memory_info().rss
    drift = (rss_final / rss_initial) - 1.0
    
    print("\n" + "="*40)
    print("TEST FINALIZADO")
    print(f"RSS Inicial: {rss_initial / 1024 / 1024:.2f} MB")
    print(f"RSS Final: {rss_final / 1024 / 1024:.2f} MB")
    print(f"Drift Residual de RSS: {drift * 100:.2f}%")
    print("="*40)
    
    # Criterio de Aceptación ST2.2.2: drift < 10%
    if drift < 0.10:
        print("✅ Certificación Exitosa: Crecimiento residual de RSS inferior al 10%.")
    else:
        print("❌ Certificación Fallida: Fuga o fragmentación excesiva detectada.")

if __name__ == "__main__":
    run_stress_test(30)
```

## Criterios de Aceptación (Gating)

1. **Drift de RSS < 10%** entre el minuto 5 (cuando la memoria se estabiliza tras el cold-start) y el minuto 30.
2. **Estabilidad de fragmentación:** La memoria lógica de HNSW (`hnsw_logical_bytes`) y el RSS física (`process_rss_bytes`) deben mostrar una correlación lineal y estable.
