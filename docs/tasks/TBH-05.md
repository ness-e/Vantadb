# TBH-05: Untrack benchmark artifacts (data_comp_bench, data_bench_db)

## Metadata
- **Plan file:** docs/plans/2026-08-30-testing-bench-harden.md
- **Creado:** 2026-08-30T21:30
- **last-synced:** 2026-08-30T21:35
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Sub-agente:** vanta-worker

## Blast Radius
| Path | Antes | Después | Acción |
|---|---|---|---|
| `benchmarks/data_comp_bench/` | 42 archivos tracked | 0 tracked, working tree intacto | `git rm -r --cached` + gitignore |
| `benchmarks/data_bench_db/` | 0 tracked (existe solo en working tree, unstaged) | gitignored (defense-in-depth) | solo gitignore |
| `.gitignore` | 261 líneas | 263 líneas (+2) | append `benchmarks/data_comp_bench/` + `benchmarks/data_bench_db/` |

## Contrato (verificable mecánicamente)
| Check | Comando | Esperado | Obtenido |
|---|---|---|---|
| Ignored data_comp_bench | `git check-ignore -v benchmarks/data_comp_bench/.vanta.lock` | exit 0, línea 36 | ✅ exit=True, `.gitignore:36` |
| Ignored data_bench_db | `git check-ignore -v benchmarks/data_bench_db/anything` | exit 0 | ✅ exit=True (líneas 36 y 143) |
| 0 tracked en ambos | `git ls-files benchmarks/data_comp_bench/ benchmarks/data_bench_db/ \| measure` | 0 | ✅ 0 |
| 2 entradas gitignore | `Select-String .gitignore "data_comp_bench\|data_bench_db"` | 2 líneas | ✅ líneas 36-37 |
| Working tree intacto | `Test-Path benchmarks/data_comp_bench`, `Test-Path benchmarks/data_bench_db` | True / True | ✅ True / True |

## Herramientas
- `git ls-files` (descubrir archivos tracked)
- `git rm -r --cached` (untrack sin borrar working tree)
- `git check-ignore -v` (verificar ignore)
- `Test-Path` (PowerShell — verificar que working tree files siguen ahí)

## Steps
### Step 1: Discovery ✅
- Confirma 42 archivos tracked en `data_comp_bench/`, 0 en `data_bench_db/`.
- `data_bench_db` existe en working tree pero nunca fue commiteado (untracked).
- `vector_index.bin` ya estaba parcialmente gitignored (línea 177) pero solo ese archivo;
  el directorio entero ahora queda cubierto por la línea 36.

### Step 2: Edit `.gitignore` ✅
- Insertadas 2 líneas en el bloque "Local database instances and logs"
  (entre `/test/` y `job_log.txt`, líneas 36-37).
- Formato consistente con patrones vecinos (`high_density_bench_db/`, `benches_db/`).

### Step 3: `git rm -r --cached benchmarks/data_comp_bench` ✅
- 42 archivos unstaged del index (preserved en working tree).
- `git rm -r --cached benchmarks/data_bench_db` falló: no había nada tracked
  (correcto — el path nunca fue commiteado).

### Step 4: `git add .gitignore` + commit ✅
- `git add .gitignore` staged la nueva entrada.
- Commit message: `chore(TBH-05): untrack benchmark artifacts (data_comp_bench, data_bench_db) — gitignored`
- NO se agregaron archivos de `benchmarks/` (los deletions ya están staged por `git rm`).

## Dependencies
- TBH-01, TBH-02, TBH-03 ✅ (precedentes — hygiene del sprint de testing bench)

## Notas
- **Por qué 2 entradas en gitignore aunque data_bench_db no estaba tracked:**
  defense-in-depth. El directorio existe en working tree (unstaged); sin la entrada,
  `git add benchmarks/` lo trackearía accidentalmente. Cobertura explícita > implícita.
- **Línea 143 (`*db/`) ya cubre `data_bench_db` indirectamente** (exit=True por línea 143),
  pero la entrada explícita es más legible para futuros devs haciendo `git check-ignore`.
- **No se tocó `benchmarks/README.md`:** Ponytail reflex — el gap era "versionado por error",
  no "documentación faltante". Los scripts Python ya están bien documentados ahí.
- **No se corrió `cargo check --workspace`:** workspace tiene bug pre-existente
  `FIND-MCP-001` en `vantadb-mcp/tests/context_tests.rs:70`. Esta tarea es solo git ops,
  no toca código Rust.

## Context Save Point
- **Fecha:** 2026-08-30T21:35
- **Branch:** develop
- **CI pendiente:** no (cambio es solo gitignore + deletions staged)
- **Decisiones:**
  - Inserción en bloque "Local database instances and logs" (consistencia con `high_density_bench_db/`, `benches_db/`).
  - NO `rm -rf` los working tree files — preservados como untracked pero gitignored.
  - NO se eliminó la línea 177 `vector_index.bin` — sigue siendo válida
    (cubre vector_index.bin en cualquier path, redundancia defensiva).
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** TBH-10 (verificar estado en plan; otras waves paralelas pueden haber avanzado).