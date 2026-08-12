# VantaDB Competitive SDK Benchmark — Honest Results (PERF-03)

> **Fecha:** 2026-08-12 · **HW:** Windows 10 (build 26200), Intel 12-core, Python 3.11.9
> **Harness:** `benchmarks/competitive_bench.py` (modo **embedded local, sin docker**)
> **Estado:** ✅ VantaDB / LanceDB / ChromaDB / **Qdrant** medidos en este HW · Milvus-frugal (milvus-lite) no instalado (opcional)

## Por qué esta tabla es honesta
- Todos los números fueron **generados por una corrida real del harness** en este HW — no estimates ni claims de marketing.
- Todos los motores corren **en el mismo proceso/CPU/RAM** (modos embedded: VantaDB PyO3, LanceDB, ChromaDB y Qdrant `QdrantClient(path=)` in-process). Ninguno usa servidor/docker, así que la comparación es directa.
- Las celdas marcadas `N/A (Inc)` = índice incremental (no hay fase de build separada medible). Las marcadas `N/M` = no medido. No se afirma superioridad sobre lo no medido.
- **No se ocultan resultados donde VantaDB pierde.** En esta configuración VantaDB pierde en recall frente a Chroma y Qdrant; se publica tal cual.

## Metodología
- Comando: `python benchmarks/competitive_bench.py --dataset synthetic --size 2000 --queries 50 --engines vanta,lance,chroma,qdrant`
- `--batch-size 999` para VantaDB (evita doble rebuild documentado en el header del harness).
- 3 iteraciones por motor; se reporta **mediana** (D4). Warmup 10 queries (D3). Ground truth = brute-force numpy (D2). Métrica euclidean, top-k 10.
- P50/P99 en ms por query; QPS = queries / tiempo total; Recall@10 vs ground truth exacto.

## Resultados medidos (synthetic 2K / 50q / top-10, euclidean)

| Engine | Ingest QPS | Index (ms) | Query QPS | p50 (ms) | p99 (ms) | Recall@10 | Peak RSS (MB) |
|--------|-----------|------------|-----------|---------|---------|-----------|--------------|
| **VantaDB** | 520.3 | 1695.7 | **635.6** | 1.510 | 2.655 | 59.20% | 236.1 |
| LanceDB | 50,086.9 | 679.3 | 126.2 | 7.379 | 17.853 | 27.00% | 233.8 |
| ChromaDB | 1,511.7 | N/A (Inc) | 398.8 | 2.242 | 5.769 | 97.60% | 257.1 |
| Qdrant | 129.5 | N/A (Inc) | 490.9 | 1.855 | 4.377 | **100.00%** | 253.9 |

## Lectura honesta (no sesgada a favor de VantaDB)
- **Query throughput:** VantaDB (635.6 QPS) > Qdrant (490.9) > Chroma (398.8) > Lance (126.2). VantaDB gana aquí.
- **Recall@10 (euclidean synthetic):** Qdrant 100% ≈ Chroma 97.6% ≫ VantaDB 59.2% > Lance 27%. **VantaDB NO es superior en recall en esta configuración.** Esto es un número real del default `ef`; subir `ef` lo mejora a costa de QPS (ver `COMPETITIVE_ANALYSIS.md` §7 sweep). No afirmar superioridad de recall.
- **Ingest:** LanceDB domina (50K QPS, batch), luego Chroma; VantaDB y Qdrant son más lentos en ingest (Vanta chunked rebuild, Qdrant upload secuencial).
- **LanceDB** tiene ingest rápido pero query lento y recall pobre (27%) con el índice por defecto en este dataset — se reporta sin maquillar.

## Caveats (para no malinterpretar)
- **ChromaDB:** solo 1 iteración válida. En Windows, runs 2/3 fallan con `WinError 32` (lock de archivo en cleanup entre corridas, issue conocido P5 de `COMPETITIVE_ANALYSIS.md`). Sus números son de la iteración 1; tratar como menos robustos.
- **Dataset sintético 2K es pequeño y no representativo.** Los números absolutos no se extrapolan. Para cifras representativas usar datasets reales: `--dataset glove-100-angular` o `--dataset sift-128-euclidean` (descarga ~1 GB ann-benchmarks).
- **Milvus-frugal** no medido: requiere `pip install milvus-lite` (comentado en `benchmarks/requirements.txt`); el harness lo salta limpio si está ausente. No se afirma nada sobre Milvus.

## Cobertura del harness (motores disponibles en este HW)
| Motor | Cliente | Disponible aquí | Medido | Cómo |
|-------|---------|----------------|--------|------|
| VantaDB | `vantadb_py` | ✅ | ✅ | PyO3 local |
| LanceDB | `lancedb` | ✅ | ✅ | in-process |
| ChromaDB | `chromadb` | ✅ | ✅ (1 iter válida) | in-process |
| Qdrant | `qdrant_client` | ✅ | ✅ | `QdrantClient(path=)` embedded, sin docker |
| Milvus | `milvus_lite` | ❌ no instalado | ❌ no medido | requiere `pip install milvus-lite` |

## Relación con el website (pendiente, fuera de alcance de este bench)
- `web/src/lib/vanta-data.ts` → `COMPETITIVE_TABLE` aún solo incluye LanceDB/ChromaDB. Para cumplir el claim competitivo del sitio contra Qdrant, esa tabla debe actualizarse con estos números (y eventualmente Milvus). El esquema `competitive-benchmark.json` (contrato INV-007-B) **no debe romperse**; este bench escribe a `competitive_sdk_bench.json` (archivo separado) para no pisar el contrato web.

## Para reproducir / cerrar al 100%
1. Qdrant ya medido. Para Milvus: `pip install milvus-lite`, luego
   `python benchmarks/competitive_bench.py --engines milvus --dataset synthetic --size 2000 --queries 50 --json-output docs/benchmarks/competitive_sdk_bench_milvus.json --output benchmarks/_n.md --yes`
2. Para cifras representativas (no sintéticas): `--dataset glove-100-angular --size 10000 --queries 100`.
