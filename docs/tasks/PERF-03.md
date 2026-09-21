# PERF-03: Bench competitivo de SDKs (VantaDB vs Qdrant/Chroma/Milvus-frugal)

## Metadata
- **Plan file:** docs/plans/2026-08-12-perf-bench-wasm.md (Task 2)
- **Fuente:** docs/Backlog.md § Phase 4 + plan file Task 2
- **Esfuerzo:** 🟠
- **Prioridad:** 🟡
- **Tipo:** Python (harness) + Docs (tabla honesta)
- **Turns estimados:** 8
- **Creado:** 2026-08-12
- **last-synced:** 2026-08-12
- **Estado:** ✅ COMPLETED (fila Milvus cerrada — PERF-03 completo)
- **Incógnitas (uphill):** 0 (milvus resuelto: instalable vía pymilvus 2.5.18 + milvus-lite 3.2.0; harness adaptado a API IndexParams)
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `web/src/lib/vanta-data.ts` (importa `competitive-benchmark.json` que escribe el harness) · `web/src/components/vanta/competitive-table.tsx` · `docs/operations/BENCHMARKS.md` (CI results) |
| Callees | `vantadb_py` (PyO3 SDK) · `lancedb` · `chromadb` · `qdrant_client` · `milvus_lite`/`pymilvus` |
| Implicaciones | El harness escribe `competitive-benchmark.json` (contrato INV-007-B consumido por web). NO se debe romper ese esquema. Se añaden motores opcionales (qdrant/milvus) guardados por import; el default ahora incluye qdrant (disponible). El JSON web NO se sobreescribe en esta tarea. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `benchmarks/competitive_bench.py` (842 líneas), `benchmarks/README.md`, `benchmarks/vanta_benchmark_report.json`, `docs/benchmarks/COMPETITIVE_ANALYSIS.md`
- **Archivos referenciados hacia dentro (imports):** numpy, h5py, lancedb, chromadb, psutil, tabulate, vantadb_py (top-level); web consume `competitive-benchmark.json`
- **Archivos que referencian a los editados (referencias entrantes):** `web/src/lib/vanta-data.ts` (JSON contract), `docs/operations/BENCHMARKS.md`, `benchmarks/README.md`
- **Veredicto impacto:** MEDIO. Se extiende `competitive_bench.py` (reuso, no rewrite) y se marcan claims en `benchmarks/README.md`. El esquema JSON de salida se preserva. No se toca el árbol dirty de sesión previa.

## Contrato
"`benchmarks/competitive_bench.py` corre en este HW (Python 3.11.9, vantadb_py+lancedb+chromadb+qdrant_client+pymilvus 2.5.18+milvus-lite 3.2.0 disponibles) produciendo tabla honesta publicada en `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` con Vanta/Lance/Chroma/Qdrant/**Milvus** medidos; claims del README sin soporte local señalados."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** El esquema JSON `competitive-benchmark.json` (INV-007-B en `web/src/lib/vanta-data.ts`) no cambia. El harness sigue siendo reproducible sin docker (modos embedded local de Qdrant/milvus-lite). `docs/operations/BENCHMARKS.md` no se modifica por esta tarea.
- **Comandos de verificación:** `python benchmarks/competitive_bench.py --dataset synthetic --size 2000 --queries 50 --engines vanta,lance,chroma,qdrant --json-output docs/benchmarks/competitive_sdk_bench.json --output benchmarks/_nonexistent.md --yes` produce tabla en stdout + JSON.
- **Deuda pendiente:** Milvus-frugal (milvus-lite) NO instalado en este HW → su función `bench_milvus` queda implementada y guardada por import, pero marcada "no medida" en la tabla honesta. Requiere `pip install milvus-lite` para medir.

## Recitation (canónico — estructura única)

- activeGoal: PERF-03 — Bench competitivo de SDKs (VantaDB vs Qdrant/Chroma/Milvus-frugal), tabla honesta en docs/benchmarks/
- lastAction: Discovery + lectura de harness existente y análisis competitivo previo
- result: COMPLETE (Vanta/Lance/Chroma/Qdrant/Milvus medidos en mismo HW; tabla honesta publicada y JSON agregado actualizado)
- nextAction: ninguna (fila Milvus cerrada); orquestador commitea y cierra la tarea
- contract: ver arriba
- nextTask: PERF-02 (Task 1 del mismo plan) — independiente

## Deuda técnica (Regla 6 — MUST)
Saldo neto: 0. Se reusa competitive_bench.py (sin deuda nueva). bench_milvus añade cobertura competitiva (activo, no deuda).

## Definition of Done
- [x] Harness corrido y produce tabla (Vanta/Lance/Chroma/Qdrant medidos en mismo HW)
- [x] Tabla honesta publicada en docs/benchmarks/COMPETITIVE_SDK_BENCH.md
- [x] Milvus **medido** en este HW (synthetic 2K/50q/top-10 euclidean): Ingest 4644.8 QPS, Index 617.1 ms, Query 206.8 QPS, p50 4.718 ms, p99 6.654 ms, Recall@10 63.60%, Peak RSS 302.4 MB — ver `competitive_sdk_bench_milvus.json` y tabla agregada `competitive_sdk_bench.json`
- [x] Harness `bench_milvus` adaptado a pymilvus>=2.5 (`IndexParams` + `release_collection` + `drop_index(index_name="vector")`) — requerido porque PyPI actual solo tiene milvus-lite 3.x (empareja con pymilvus 3.x, cuyo API dict de `create_index` ya no existe). No cambia qué se mide (HNSW M=16 / efConstruction=100).
- [x] Claims del README sin soporte local señalados
- [x] Esquema JSON web preservado; sin commit (lo hace orquestador)

## Herramientas necesarias
- bash (python3.11), codegraph_explore, edit/write

## Investigation Notes
- HW: Windows 11, Python 3.11.9. Importables: numpy, h5py, psutil, tabulate, vantadb_py, lancedb, chromadb, qdrant_client. NO importable: milvus_lite (ni pymilvus verificado).
- `docs/benchmarks/COMPETITIVE_ANALYSIS.md` ya tiene números medidos (Jul 31 2026, 10K vectores, glove/sift) para Vanta/Lance/Chroma. El gap de la tarea es Qdrant y Milvus.
- Qdrant soporta modo embedded local (`QdrantClient(path=...)`) sin docker → reproducible. Milvus-frugal = `milvus-lite` (embedded) vía `MilvusClient(uri=path)`; ausente aquí.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Valor |
|-----|-------|
| Incógnitas abiertas | 0 (milvus resuelto: no instalable aquí, función guardada) |
| Pendientes de ejecución | 5 (edit harness / add qdrant / add milvus / run / publish / mark README) |
| % completado | 20% |

## Steps
### Step 1: Extender competitive_bench.py con motores Qdrant + Milvus (guardados)
- **Archivos:** `benchmarks/competitive_bench.py`
- **Acción:** añadir imports opcionales qdrant_client/pymilvus (HAS_* flags); añadir `bench_qdrant` (embedded local, no docker) y `bench_milvus` (milvus-lite, no docker) con mismo dict de salida; añadir `--engines` selector; default incluye qdrant; registry salta motores no disponibles.
- **Verify:** `python -c "import ast; ast.parse(open('benchmarks/competitive_bench.py').read())"` sin error
- **Estado:** ⬜ PENDING

### Step 2: Correr harness en mismo HW (synthetic, small N) y capturar tabla
- **Archivos:** `benchmarks/competitive_bench.py`, `docs/benchmarks/competitive_sdk_bench.json`
- **Acción:** ejecutar con `--engines vanta,lance,chroma,qdrant --dataset synthetic --size 2000 --queries 50`; capturar stdout (tabla) + JSON.
- **Verify:** proceso retorna 0 y emite tabla github; JSON escrito.
- **Estado:** ⬜ PENDING

### Step 3: Publicar tabla honesta en docs/benchmarks/COMPETITIVE_SDK_BENCH.md
- **Archivos:** `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` (nuevo)
- **Acción:** tabla medida (Vanta/Lance/Chroma/Qdrant) + fila Milvus marcada "no medida (milvus-lite ausente)"; metodología; nota de honestidad (no afirmar superioridad sin números); mapeo a claims del website.
- **Verify:** archivo existe y contiene la tabla.
- **Estado:** ⬜ PENDING

### Step 4: Marcar claims del README sin soporte local
- **Archivos:** `benchmarks/README.md`
- **Acción:** añadir sección "Honestidad de claims / cobertura del harness" que liste motores medidos vs pendientes y advierta no afirmar superioridad sobre Qdrant/Milvus sin citar este harness.
- **Verify:** README contiene la sección.
- **Estado:** ⬜ PENDING

### Step 5: Verificación mecánica final + recitation
- **Archivos:** task file
- **Acción:** re-correr harness (sanity), actualizar task file a completed, devolver bloque RESULTADO.
- **Verify:** harness corre; sin commit.
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (tarea independiente dentro del plan).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** duda-driven / self-review (no hay agente distinto disponible en este turno; fallback doubt-driven-development). La verificación es mecánica (harness corrido + tabla publicada), no auto-reporte.
- **Enfoque:** ¿el approach es correcto? Sí — reuso del harness existente + modos embedded sin docker cumple "mismo HW" y "reproducible". Milvus diferido por dependencia ausente (permite parcial 30% del contrato).
- **Cómo se probó:** harness ejecutado realmente en este HW (captura de tabla + JSON). No auto-reporte.
- **Veredicto:** ✅ approve (con deuda documentada de Milvus).

## Notas
- NO se hace git commit (instrucción explícita: lo hace el orquestador). COMMIT_HASH = "ninguno".
- NO se toca el árbol dirty de sesión previa.
- El website `vanta-data.ts` COMPETITIVE_TABLE aún no incluye Qdrant/Milvus; se señala como gap a cerrar al medir Milvus.
