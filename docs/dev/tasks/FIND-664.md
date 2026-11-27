---
title: "FIND-664: el CI de benchmarks no mide - paniquea tras continue-on-error o salta en verde"
kind: task
status: completed
description: "Cableada la provision de datasets con gate TBH-01 fail-loud, registrado glove_dimension_scale y corregida la ruta del artefacto de densidad. Los tests ya no pueden pasar sin su dataset."
id: FIND-664
type: bug
---

# FIND-664 — El CI de benchmarks no mide: paniquea o salta en verde

- **Prioridad:** 🟠 Alta · **Origen:** auditoría 59 benchmarks 2026-11-22 (#01,#02,#04,#11,#15,#28,#29,#31)
- **Creado:** 2026-11-22 · **Cerrado:** 2026-11-27
- **Estado:** ✅ RESUELTO

## Por qué es el primero del lote

Los puntos (1) y (2) de la fila son **`FIND-787`/`FIND-794` en otra capa**: un job
que se salta y se lee como verde. Mientras eso viva, **toda medición aguas abajo
es sospechosa** — incluidos los números de `FIND-668`, `FIND-669` y `FIND-670`.

## Diagnóstico (medido, no inferido)

| # | Rotura de la fila | Verificación mecánica | Resultado |
|---|---|---|---|
| 1 | Benches paniquean por falta de `data/benchmark/{glove-100,sift-128,beir-scifact}` | `benches/common/mod.rs:151-153` → `panic!("missing dataset {name} ...")`; el job `light-benchmarks` (`certification.yml`) y `bench-informational` (`nightly.yml`) **nunca ejecutaban** `download_ground_truth.py` | Confirmado |
| 2 | `sift-validation` / `competitive_bench` pasan en verde sin validar | `tests/certification/sift_validation.rs:18-21` y `competitive_bench.rs:172-176` → `println! + return` cuando falta el dataset, y el job no lo descargaba | Confirmado |
| 3 | `glove_dimension_scale.rs` sin `[[test]]` | `grep -c glove_dimension_scale Cargo.toml` = **0** | Confirmado (y peor: al registrarlo, **no compilaba**) |
| 4 | `certification.yml` escribe `metricas/benchmarks/nightly_density_results.txt` y sube `nightly_density_results.txt` | comparación literal de ambos `path:` | Confirmado |

### Hallazgo adicional del punto (1): el `continue-on-error` no era el único silencio

`analyze` copiaba con `cp light/nightly_*_results.txt .`. `upload-artifact` guarda
las rutas **relativas al *least common ancestor*** de los `path:` declarados; para
`metricas/benchmarks/nightly_*_results.txt` + `target/criterion/` ese LCA es la
raíz del repo, así que los archivos llegan como
`light/metricas/benchmarks/nightly_*_results.txt`. El `cp` no matcheaba nada y el
`|| true` lo tapaba: **el job `analyze` mergeaba cero resultados y el resumen
reportaba cero benchmarks**, en los dos jobs de bench, no solo en densidad.

## Contrato y lo que se cambió

1. **Provisión verificada con gate TBH-01 fail-loud.** `scripts/download_ground_truth.py`
   gana `--check` que **nombra el archivo que falta** y sale 1, y `main()` ahora sale 1
   si algún dataset falla (antes imprimía `FAILED:` y devolvía 0 — el `expect` del bench
   llegaba 40 minutos después). `h5py`/`requests` pasan a import perezoso: `--check`
   funciona sin la red ni el stack HDF5 instalados.
2. **Los tests no pueden saltar.** `tests/common/mod.rs` gana
   `require_datasets(paths, what, provision_with)`: si falta un archivo **paniquea**
   nombrando archivo, comando de provisión y la fila. La única salida es el opt-out
   local explícito `VANTA_ALLOW_MISSING_DATASETS=1` (CI nunca lo define).
   Lo usan `sift_validation`, `competitive_bench` y `glove_dimension_scale`.
3. **`glove_dimension_scale` registrado y arreglado.** `Cargo.toml` gana su `[[test]]`,
   `.config/nextest.toml` su exclusión del `default-filter` (misma forma
   `not (package(X) and binary(Y))` que los otros 55, `CI_POLICY.md:196-200`), y el
   test gana su lane en el job `benchmark-datasets`. Al compilarlo por primera vez
   aparecieron dos defectos de código muerto: el literal `HnswConfig` no tenía
   `auto_tune` (E0063) y `idx.add(...)` descartaba el `Result` (un insert rechazado
   habría medido el recall sobre un corpus más chico). Ambos corregidos.
4. **GloVe-300d.** `download_benchmark_datasets.sh` extraía solo 100d, así que el test
   de escala dimensional no tenía su segunda mitad. Ahora extrae y valida 100d + 300d,
   y el cache del job cubre ambas.
5. **Ruta del artefacto.** `high-density` sube `metricas/benchmarks/nightly_density_results.txt`
   (lo que el `tee` escribe) con `if-no-files-found: error`; `light-benchmarks` idem;
   `analyze` copia las rutas reales y **falla** si recuperó 0 archivos.
6. **SIFT provisionable.** `DATASETS["sift-1m"]` descendía de
   `dev-tools/scripts/download_sift.py`, un pull FTP de `corpus-texmex` que está muerto
   y que atrapaba su propia excepción con exit 0 — los dos jobs opt-in no podían
   aprovisionar **nunca**. Ahora `sift-1m` es `hdf5-fvecs`: baja el HDF5 de
   ann-benchmarks (la misma fuente que ya usan los demás datasets) y escribe
   `datasets/sift/sift_{base,query}.fvecs` + `sift_groundtruth.ivecs`, que es el layout
   que esos tests leen. `verify_datasets.sh` apuntaba además a
   `data/benchmark/glove-100-angular/`, un directorio que ningún dataset produce
   (el real es `glove-100-ann`).

## Medición

```
$ python scripts/download_ground_truth.py --datasets glove-100 beir-scifact sift-1m --check
All 3 datasets ready in ...\data\benchmark                       exit 0

$ # gate con un dataset ausente (BENCH_DIR redirigido a un path inexistente)
MISSING glove-100: data\benchmark\__absent__\train.f32, ...\test.f32,
                  ...\test_neighbors.u64, ...\meta.json
Provision with: python scripts/download_ground_truth.py --datasets glove-100
                                                                 exit 1

$ cargo test -p vantadb --test sift_validation    # con datasets en disco
SIFT1M Dataset Integrity: PASSED ✅
test result: ok. 1 passed; finished in 5.35s                      exit 0

$ # mismo test, datasets/sift movido a datasets/sift.hidden
thread 'validate_sift_dataset_integrity' panicked at tests\certification\..\common\mod.rs:49:5:
SIFT-1M dataset missing: datasets/sift/sift_base.fvecs, datasets/sift/sift_query.fvecs,
datasets/sift/sift_groundtruth.ivecs. This test measures SIFT-1M; without it the run
would validate nothing (TBH-01, FIND-664). Provision it with
`python scripts/download_ground_truth.py --datasets sift-1m`, or set
VANTA_ALLOW_MISSING_DATASETS=1 to skip locally only.
test result: FAILED. 0 passed; 1 failed                              exit 101

$ # mismo test, dataset ausente + VANTA_ALLOW_MISSING_DATASETS=1
SKIP (VANTA_ALLOW_MISSING_DATASETS=1): SIFT-1M absent — ...          exit 0

$ cargo check -p vantadb --test glove_dimension_scale --test sift_validation --test competitive_bench
    Finished `dev` profile [optimized] target(s) in 3.95s            exit 0
$ cargo clippy -p vantadb --test glove_dimension_scale --test sift_validation \
    --test competitive_bench -- -D warnings
    Finished `dev` profile [optimized] target(s) in 7.91s            exit 0

$ actionlint .github/workflows/{certification,nightly}.yml          exit 0  3.8 s
$ actionlint -shellcheck= .github/workflows/{certification,nightly}.yml  exit 0  0.6 s
$ ruff check scripts/download_ground_truth.py                       exit 0
$ ruff format --check scripts/download_ground_truth.py              exit 0
$ shellcheck scripts/{download_benchmark_datasets,verify_datasets}.sh  exit 0
```

**Tamaño en disco de la provisión** (medido, no estimado): `datasets/sift/` =
492.1 MiB `sift_base.fvecs` + 4.9 MiB `sift_query.fvecs` + 3.9 MiB
`sift_groundtruth.ivecs` = **500.9 MiB**, más ~500 MiB de HDF5 en `data/benchmark/cache/`.
`glove-100` = 40 MiB (100k × 100d) + 8 MiB de ground truth. `beir-scifact` < 5 MiB.

## Verificación del contrato

- **"El job debe fallar nombrando el dataset ausente"** → gate `--check` (paso
  bloqueante, sin `continue-on-error`) y guard dentro del test (exit 101). Ambos
  medidos arriba.
- **"El artefacto subido debe existir en la ruta que declara"** → `path:` del upload
  = la ruta que el `tee` escribe, `if-no-files-found: error`, y `analyze` falla con
  `::error::` si recupera 0 archivos.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `benches/common/mod.rs`, `benches/canonical_p99.rs` (cabecera +
  carga), `benches/hybrid_queries.rs`, `benches/param_sweep.rs`, `benches/tokenizer_bench.rs`,
  `scripts/download_ground_truth.py`, `scripts/download_benchmark_datasets.sh`,
  `scripts/verify_datasets.sh`, `tests/certification/{sift_validation,competitive_bench,glove_dimension_scale}.rs`,
  `tests/common/mod.rs`, `.config/nextest.toml`, `.github/workflows/certification.yml`,
  `.github/workflows/nightly.yml` (job `bench-informational`), `Cargo.toml` (sección `[[test]]`).
- **Referencias hacia dentro:** `load_standard_dataset` (3 benches), `DS_DATASET` de
  `param_sweep`, `verify_datasets.sh` (solo documentación de provisión local, ya no es gate),
  `download_ground_truth.py` (usado por `BENCHMARKS.md` y por los jobs de CI).
- **Referencias entrantes:** `docs/user/operations/BENCHMARKS.md` cita el comando de
  provisión; `CI_POLICY.md:196-200` documenta el `default-filter` como «cambia solo junto
  con `certification.yml`» — se respetó.
- **Veredicto:** cambio contenido en CI + tests + scripts de datos. **Ningún** path de
  `src/**`, `vanta-memory/**`, `vantadb-mcp/**` ni `vantadb-node/src/**` fue tocado.
- **Fuera de la zona declarada del lote (declarado al owner):** `Cargo.toml` (una
  entrada `[[test]]`), `.config/nextest.toml` (una línea del `default-filter`),
  `tests/common/mod.rs` + `tests/certification/*.rs` y `scripts/*`. Sin ellos la fila no
  se puede cerrar: el registro del test vive en `Cargo.toml`, la exclusión del fast lane
  en `.config/nextest.toml`, y el "no saltar" en los tests.

## Hallazgos colaterales (resueltos en el acto)

| # | Hallazgo | Estado |
|---|---|---|
| H1 | `analyze` mergeaba 0 archivos de resultado (LCA de `upload-artifact`) | ✅ corregido + gate que falla si recupera 0 |
| H2 | `glove_dimension_scale.rs` no compilaba (`HnswConfig.auto_tune`, `Result` descartado) | ✅ corregido |
| H3 | `download_ground_truth.py` se tragaba los fallos (exit 0) | ✅ corregido |
| H4 | `h5py`/`requests` importados en el tope rompían hasta `--check` | ✅ import perezoso |
| H5 | `verify_datasets.sh` apuntaba a `glove-100-angular/` (no existe) y a un script FTP muerto | ✅ corregido |
| H6 | `upload-artifact` de light-results con `if-no-files-found` por defecto (`warn`) | ✅ `error` |
| H7 | `download_benchmark_datasets.sh` no traía GloVe-300d | ✅ corregido |

## Deuda / fuera de zona

- `AGENTS.md` Regla 2 afirma «existen 8 instancias» de `continue-on-error: true`; el
  conteo real es 40 (39 tras quitar el job-level de `bench-informational`). Registrado
  como `FIND-825`.
- `dev-tools/scripts/download_sift.py` sigue en el árbol marcado DEPRECATED, atrapando
  su excepción y saliendo 0. Ya nadie lo invoca; borrarlo es de `dev-tools/**`.
  Registrado como `FIND-824`.

## Review (P2-01)

Tier: **Fast** (paths del diff: `.github/workflows/**`, `tests/**`, `scripts/**`,
`Cargo.toml`, `.config/**`, `docs/dev/**` — ninguno de la tabla Adversarial).
Verify mecánico: `dev-tools/verify_changed.ps1` + `actionlint` (3.8 s / 0.6 s) +
`ruff` + `shellcheck` verdes; veredicto: **approve**.
