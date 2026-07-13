# `fuzz-40.yml` — FUZZ: LibFuzzer — Corpus + Regression

## ¿Qué hace?

Ejecuta fuzzing con cargo-fuzz/LibFuzzer sobre 4 objetivos del core de VantaDB para encontrar bugs de seguridad, crashes y comportamientos inesperados mediante entradas aleatorias.

## ¿Cómo lo hace?

2 jobs secuenciales:

1. **`build`**: compila todos los targets de fuzzing con `cargo fuzz build` (toolchain nightly)
2. **`fuzz`** (matrix con 4 targets): corre cada target con `cargo fuzz run <target> -- -max_total_time=<segundos>`

Targets de fuzzing:
- `fuzz_parser` — fuzzing del parser de queries
- `fuzz_node_deserialize` — fuzzing de deserialización de nodos
- `fuzz_wal` — fuzzing del Write-Ahead Log (WAL)
- `fuzz_archive` — fuzzing del archive/compaction

El corpus de cada target se cachea entre ejecuciones para guiding eficiente.

## ¿Qué tests usa?

No usa tests tradicionales. Usa **cargo-fuzz** (LibFuzzer) que genera inputs aleatorios y monitorea crashes.

## ¿Qué verifica?

Que ningún input malformado cause:
- Pánicos (panics)
- Desbordamientos de buffer
- Violaciones de memoria
- Cuelgues infinitos (timeouts)
- Comportamientos indefinidos

## Funcionalidad final

Detección temprana de vulnerabilidades y bugs de memoria en componentes críticos (parser, WAL, serialización) mediante fuzzing continuo con corpus persistente.

## ¿Cuándo se ejecuta?

- **Semanal** (cada lunes 06:00 UTC) vía `schedule`
- **Workflow dispatch** manual con parámetro configurable de segundos por target
