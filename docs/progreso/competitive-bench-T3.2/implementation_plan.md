# Plan de Implementación: T3.2 — Benchmark Competitivo Oficial (GloVe & SIFT)

## Contexto

La infraestructura del benchmark competitivo ya está implementada en `benchmarks/competitive_bench.py`. Sin embargo, actualmente `docs/BENCHMARKS.md` solo contiene mediciones basadas en datos sintéticos.
Para cerrar formalmente la tarea **T3.2** del Plan Maestro, debemos:
1. Ejecutar las mediciones competitivas usando los datasets estándar de la industria: `glove-100-angular` (100 dimensiones, métrica coseno) y `sift-128-euclidean` (128 dimensiones, métrica euclideana) a una escala controlada de 10,000 registros para garantizar la equidad y reproducibilidad rápida.
2. Actualizar el documento `docs/BENCHMARKS.md` con las tablas de resultados oficiales.
3. Marcar la tarea T3.2 como completada en `VantaDB_Plan_Maestro_Unificado.md`.

De acuerdo con las reglas globales, los comandos de ejecución de benchmark serán proporcionados para que el usuario los corra manualmente y nos comparta los resultados en la consola.

## Cambios Propuestos

---

### Documentación de Rendimiento

#### [MODIFY] [BENCHMARKS.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/BENCHMARKS.md)
* **Objetivo:** Reemplazar o complementar la sección 7 ("Competitive Benchmark vs LanceDB & Chroma") con los resultados reales obtenidos en los datasets estándar `glove-100-angular` y `sift-128-euclidean`.

---

### Actualizaciones de Control

#### [MODIFY] [VantaDB_Plan_Maestro_Unificado.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/VantaDB_Plan_Maestro_Unificado.md)
* Cambiar el estado de `T3.2 — Benchmark competitivo vs LanceDB y Chroma` a `✅ COMPLETADA` y marcar todas sus subtareas (ST3.2.1, ST3.2.2, ST3.2.3) como completadas.

## Plan de Verificación y Ejecución

### Ejecución Manual (Usuario)
El usuario ejecutará los siguientes comandos en su entorno de Python configurado (con las librerías `numpy`, `h5py`, `lancedb`, `chromadb`, `psutil`, `tabulate` y el SDK `vantadb_py` instalado):

1. **Benchmark de GloVe (Coseno):**
   ```powershell
   python benchmarks/competitive_bench.py --dataset glove-100-angular --size 10000 --queries 100
   ```

2. **Benchmark de SIFT (Euclideana):**
   ```powershell
   python benchmarks/competitive_bench.py --dataset sift-128-euclidean --size 10000 --queries 100
   ```

El usuario proporcionará el output impreso en consola para que podamos registrar las métricas exactas en la documentación y cerrar la tarea.
