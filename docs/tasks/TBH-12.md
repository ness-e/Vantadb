# TBH-12 — data/README.md + datasets/README.md

**Status:** ⬜ PENDING → ✅ COMPLETO
**Phase:** 2 / docs
**Origin:** TBH-12 in plan `docs/plans/2026-08-30-testing-bench-harden.md`
**Sub-agent:** vanta-docs (Wave 2 parallel)
**Plan:** docs/plans/2026-08-30-testing-bench-harden.md (lines 118–122)

## Contrato

Crear 2 archivos README (`data/README.md`, `datasets/README.md`) que documenten QUÉ
datasets se esperan en cada directorio, DÓNDE van, licencia, tamaño y comando de
descarga. Formato consistente con `embeddings/README.md`.

## Archivos clave (discovery)

- `scripts/download_benchmark_datasets.sh` — descarga GloVe-100 → `data/benchmark/glove.6B.100d.txt`
- `scripts/download_benchmark_datasets.ps1` — PowerShell equivalente
- `dev-tools/scripts/download_sift.py` — descarga SIFT-1M tar.gz → `datasets/sift.tar.gz` + extrae
- `scripts/download_ground_truth.py` — descarga HDF5 ann-benchmarks → `data/benchmark/{sift-128,glove-100}/{train.f32,test.f32,test_neighbors.u64,meta.json}`
- `embeddings/README.md` — referencia de estilo (tabla markdown)
- `.gitignore` líneas 23 (`/data/`) y 75 (`datasets/`) — confirma ambos directorios gitignored

## Impacto mapeado (Regla 0)

- **Hacia dentro:** ninguno. Archivos nuevos en directorios gitignored (binarios no entran al repo).
- **Hacia afuera:** referencias opcionales — scripts NO referencian los READMEs (sólo crean
  archivos). Cross-link es opcional; NO modificar scripts en esta tarea (D5 / scope).
- **Veredicto:** Blast radius = 2 archivos nuevos (`data/README.md`, `datasets/README.md`) +
  1 task file. Riesgo: cero. No requiere review cruzada.

## Datasets verificados (info en scripts)

| Dataset (nombre lógica) | Fuente URL | Licencia | Tamaño aprox. | Script descarga | Destino |
|---|---|---|---|---|---|
| GloVe-100 (`glove.6B.100d.txt`) | https://nlp.stanford.edu/data/glove.6B.zip | Public Domain (PDDL 1.0, https://opendatacommons.org/licenses/pddl/1.0/) | 822 MB (zip completo), 100d txt ~350 MB | `scripts/download_benchmark_datasets.{sh,ps1}` | `data/benchmark/` |
| SIFT-1M (`sift.tar.gz` con `sift_base.fvecs`, `sift_query.fvecs`, `sift_groundtruth.ivecs`) | ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz | No comercial (research use, Jégou et al.) — confirmar antes de redistribuir | ~160 MB compressed, ~1 GB uncompressed | `dev-tools/scripts/download_sift.py` | `datasets/` |
| SIFT-128 Euclidean subset (HDF5) | http://ann-benchmarks.com/sift-128-euclidean.hdf5 | ann-benchmarks (MIT, https://github.com/erikbern/ann-benchmarks) | ~500 MB | `scripts/download_ground_truth.py --datasets sift-128` | `data/benchmark/sift-128/` |
| GloVe-100 Angular subset (HDF5) | http://ann-benchmarks.com/glove-100-angular.hdf5 | ann-benchmarks (MIT) | ~1 GB | `scripts/download_ground_truth.py --datasets glove-100` | `data/benchmark/glove-100/` |

**Nota deliberada:** el plan original mencionaba "GloVe-300" — los scripts NO descargan GloVe-300
(sólo extraen `glove.6B.100d.txt` del zip completo, pero el zip incluye 50d/100d/200d/300d).
Documento GloVe-100 (lo único que descargan los scripts reales) y menciono GloVe-300 como
opción opcional dentro del mismo zip (D5 / No inventes info).

## Spec (contenido válido — Decision Table)

| Decisión | Alternativa | Elegida | Evidencia |
|---|---|---|---|
| Formato READMEs | Tabla única vs 1 README detallado + symlink | 2 archivos simétricos con tabla + cross-link entre ellos | `embeddings/README.md` tiene tabla única — replicar patrón |
| Incluir GloVe-300 (300d)? | Sí (en tabla) / No (solo 100d que descargan scripts) | No — solo 100d | `download_benchmark_datasets.sh:17` extrae SOLO `glove.6B.100d.txt` |
| Incluir SHA256? | Por dataset / No | No — no están en scripts | Regla "No inventes info" — si los scripts no los incluyen, no inventar hashes |
| Cross-link entre data/ y datasets/ | Sí / No | Sí — ambos apuntan al otro | Tabla "Related docs" |
| Modificar scripts para que apunten a README? | Sí (agregar echo) / No | No — scope dice "Cross-link es opcional, NO modificar scripts" | task file línea 4 del contrato |

## Pasos

- [x] **STEP-1 (PLAN):** Crear task file + descubrimiento (este archivo)
- [x] **STEP-2 (ACT):** Escribir `data/README.md`
- [x] **STEP-3 (ACT):** Escribir `datasets/README.md`
- [x] **STEP-4 (VERIFY):** `Test-Path` ambos + `Select-String` confirma menciones GloVe/SIFT
- [x] **STEP-5 (ACT):** Commit `docs(TBH-12):` + update state completed