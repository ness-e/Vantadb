# FIND-72 — CLI + pins + Chroma en benches py

> **Plan:** `docs/plans/2026-09-15-find-correcciones.md` (Task 27, Wave8)
> **Campaign ID:** 6ab26f3f-cf16-4416-9255-c18cca0bcaf0
> **Estado:** ⬜ PENDING → IN PROGRESS (DISCOVERY completo, sin task file previo — creado según pipeline-full.md)
> **Appetite / Branch / Commit:** 1d / develop / `fix: FIND-72`
> **Ruta:** vanta-worker · **Esfuerzo:** 🟡 · **Prioridad:** 🟢
> **SDP:** `campaign_discover_skills_v2` phase=BUILD keywords=[bench-cli, argparse, requirements-pins, winerror32] → 8 skills (ver §5)

## 1. TAREA — objetivo + contrato exacto + acceptance criteria

**Objetivo:** `benchmarks/batch_vs_sequential_bench.py` no tiene argparse y su `__main__` corre ambos benches → `--help` ejecuta todo (peor que lo reportado). `benchmarks/requirements.txt` sin pins (solo `vantadb-py>=0.5.0`). `benchmarks/competitive_bench.py` mezcla `rmtree` con/sin `ignore_errors` → WinError32 en Windows.

**Contrato (plan file Task 27):** `--help` no ejecuta + pins + WinError32 documentado/fix + `py_compile` 0.

**Acceptance criteria:**
1. `python benchmarks/batch_vs_sequential_bench.py --help` imprime ayuda y sale 0 sin correr benches.
2. `benchmarks/requirements.txt` con floors pineados en las 7 deps directas (+ nota pymilvus/milvus-lite ya pineada en comentarios).
3. Todos los `shutil.rmtree` de ambos benches con `ignore_errors=True` + nota WinError32 (Windows file lock) en cada archivo.
4. `python -m py_compile` sobre ambos archivos sale 0.
5. Defaults del CLI nuevo = valores hardcodeados actuales exactos (pre-mortem: no romper uso documentado/CI).

## 2. ARCHIVOS — clave (:línea verificada en DISCOVERY) + relacionados + prohibidos

**Clave (nombres reales verificados en disco):**
- `benchmarks/batch_vs_sequential_bench.py:31` (`run_bench(db_path="./benchmarks/batch_bench_db", num_vectors=5000, dim=128, batch_size=100, top_k=10)`), `:96-102` (`run_batch_requests_bench(db_path="./benchmarks/batch_requests_bench_db", num_records=2000, dim=128, batch_size=10, top_k=10)`), `:199-207` (`__main__` corre ambos sin argparse), rmtree sin `ignore_errors` en `:33,88,107,195`.
- `benchmarks/requirements.txt` — `:12` `vantadb-py>=0.5.0` (único pin), `:16-22` 7 deps sin versión (`numpy, h5py, lancedb, chromadb, qdrant-client, psutil, tabulate`), `:28-29` comentarios `pymilvus`/`milvus-lite` sin pin.
- `benchmarks/competitive_bench.py` — rmtree SIN `ignore_errors` en `:239` (bench_vantadb), `:361` (bench_lancedb), `:452` (bench_chromadb), `:559` (bench_qdrant — HALLAZGO DISCOVERY: el plan citaba 3, el código tiene 4; el fix `replaceAll` cubrió los 4, verificado por grep post-fix); CON `ignore_errors=True` en `:343,434,532,616,720,989`. `--help` ya OK (argparse en `main():878-900`, `__main__:1072-1073` solo llama `main()`). Nota WinError32 existente solo para Milvus (`:641-642`); falta nota general.

**Relacionados (leer, NO cambiar salvo contrato):**
- `benchmarks/vantadb_local_bench.py:241-257` — patrón argparse a seguir (mismo estilo de flags/descripciones).
- CI que invoque estos benches: grep `batch_vs_sequential|competitive_bench` en `.github/workflows/` → **0 callers del .py**: solo `heavy-certification-50.yml:99,109` corre el test Rust `competitive_bench` (`cargo test --release --test competitive_bench`, distinto artefacto) y `ci-rust-10.yml:604` skipea `sift1m_competitive_benchmark` (Rust). `perf-bench-40.yml` solo usa `vantadb_local_bench.py`. Conclusión: ningún workflow consume flags del batch .py → defaults nuevos solo deben igualar los hardcodeados actuales.
- `docs/operations/BENCHMARKS.md:971,974` — ejemplos de invocación de `competitive_bench.py` (flags `--dataset/--size/--queries/--engines/--batch-size/--json-output/--output/--yes` NO se tocan).

**Prohibidos (WIP ajeno / fuera de scope — NO tocar):**
`.opencode/` (submodule, solo lectura), `Justfile`, `completions/_vanta-cli*`, `desktop/src-tauri/Cargo.lock`, `.github/workflows/ocr-delegate.yml`, `dev-tools/ocr-review.ps1`, `reparacion.bat`, `docs/pipeline-state.json`, plan file (solo recitation orquestador), stash@{0..14}, archivos FIND-74 (`examples/`, QUICKSTART, README raíz — 5e428aea), FIND-86 (`vanta-memory/*` — 29ec9f02), `docs/Backlog.md` (race paralelo → orquestador vía progreso), `docs/avance/` (orquestador).

## 3. DEPENDENCIAS

Wave8 (Wave0-7 DONE 24/30 + FIND-74 ✅ 5e428aea + FIND-86 ✅ 29ec9f02). Sin bloqueantes. FIND-72 es la última de Wave8. Previa: FIND-86 ✅. Next: Wave9 (FIND-92/93 + FIND-94 sin wave — NO absorber salvo HALLAZGO explícito con evidencia).

## 4. REFERENCIAS + Spec mini + Regla 11

Ninguna técnica nueva (stdlib `argparse`/`shutil`, patrón copiado de `vantadb_local_bench.py:241-257` + `competitive_bench.py:878-900` del mismo repo). Paso 0c leído: `clean-code-clean-architecture.md` completo (Apéndice V normativo: benches = Frameworks/Drivers → Humble Objects, cero lógica de negocio; severidades `/cleanCA`), 4 páginas Notion leídas (Problema/Propuesta/Nuevas features/Plan de accion) — filtro VantaDB: sin mapeo a bench CLI (solo Propuesta Anexo B cita BENCHMARKS.md como fuente canónica de números; esta tarea no cambia números).

**Spec mini — flags NUEVOS de `batch_vs_sequential_bench.py` (defaults = hardcodeados actuales, sin cambio de comportamiento):**

| Flag | Default antes (hardcodeado) | Default después (argparse) | Notas |
|---|---|---|---|
| `--db-path` | `"./benchmarks/batch_bench_db"` (run_bench) | mismo | run_bench |
| `--num-vectors` | `5000` | `5000` | run_bench |
| `--dim` | `128` | `128` | ambos benches |
| `--batch-size` | `100` (bench1) / `10` (bench2) | `100` / `--requests-batch-size 10` | nombres distintos por bench para no colisionar |
| `--top-k` | `10` | `10` | ambos benches |
| `--requests-db-path` | `"./benchmarks/batch_requests_bench_db"` | mismo | run_batch_requests_bench |
| `--num-records` | `2000` | `2000` | run_batch_requests_bench |
| `--skip-batch-requests` | n/a (siempre corría) | `False` | opt-in para correr solo bench1; default preserva corrida completa |

**Regla 11:** N/A justificado — sin claims de performance nuevos; no se documentan tiempos, solo se preservan defaults. Si se citara un número, fuente = `docs/operations/BENCHMARKS.md`.

**Símbolos públicos nuevos:** ninguno en SDK (flags CLI de script bench ≠ API pública). Sin tabla Spec de símbolos.

## 5. SKILLS (SDP Paso 0b + base worker)

`campaign_discover_skills_v2` (phase=BUILD, keywords bench-cli/argparse/requirements-pins/winerror32) → 8 candidatas. Cargadas (5) + descartes justificados (3):
- `incremental-implementation` — slices verticales por archivo (Step 1 CLI, Step 2 pins+rmtree) ✅ cargada
- `test-driven-development` — Prove-It: `--help` debe salir 0 sin correr nada (RED = correr `--help` antes del fix y ver que ejecuta benches) ✅ cargada
- `context-engineering` — context pack del slice (rules → plan → source + patrón vecino) ✅ cargada
- `source-driven-development` — pins: stdlib argparse/shutil no requieren red; floors de terceros marcados [cita NO VERIFICADA — sin red] ✅ cargada
- `api-and-interface-design` — diseño de flags CLI consistentes con benches vecinos (naming `--*-*`, defaults aditivos) ✅ cargada
- `systematic-debugging` — bug inline: root cause `__main__` sin guard + rmtree inconsistente ✅ cargada
- ~~`doubt-driven-development`~~ — descartada: cambio mecánico de bajo riesgo, sin trust boundary ni irreversibilidad
- ~~`frontend-ui-engineering`~~ — descartada: sin `web/` tocado
- `SKILLS_CARGADAS:` incremental-implementation, test-driven-development, context-engineering, source-driven-development, api-and-interface-design, systematic-debugging (+ base campaign-executor/progreso/ponytail).

## 6. HERRAMIENTAS + MCP

**Comandos exactos (Python del repo: `python` 3.14.7 con `vantadb-py` 0.5.0 instalado):**
- RED/contrato: `python benchmarks/batch_vs_sequential_bench.py --help` (debe salir 0 sin correr nada) + `python benchmarks/competitive_bench.py --help`
- `python -m py_compile benchmarks/batch_vs_sequential_bench.py benchmarks/competitive_bench.py`
- `git diff --check` + `git status --short`
- `rg "batch_vs_sequential|competitive_bench" .github/` (ya corrido en DISCOVERY → 0 callers .py)

**MCP:** `campaign_detect_task_type` ✅ (unknown → base-only), `campaign_discover_skills_v2` ✅ (§5), `campaign_verify_cmd` (bug exit -1 conocido → fallback bash directa y anotarlo en RESULTADO), `campaign_update_task_state` (in-progress/completed + recitation), `codegraph_explore` N/A (scripts py de benchmarks fuera del índice Rust; lectura directa + grep usados en su lugar). Sin red (pins no verificables online → [cita NO VERIFICADA] + deuda TSYS-13 en recitation).

## 7. INVESTIGACIÓN CÓDIGO — blast radius (DISCOVERY)

- `__main__ (:199-207)` → corre `run_bench()` + `run_batch_requests_bench()` incondicionalmente; cualquier invocación con flags (`--help` incluido) ejecuta ambos benches. Implicación: añadir `argparse` con `parse_args()` hace que `--help`/`--version` salgan antes de tocar código de bench (comportamiento estándar).
- Defaults usados por CI: ninguno — 0 workflows invocan estos `.py` (evidencia §2). Riesgo argparse-con-defaults-distintos → mitigado fijando defaults = hardcodeados actuales (tabla §4).
- `rmtree` mixto: `competitive_bench.py` 3 sitios sin `ignore_errors` (`:239,361,559`) vs 6 con (`:343,434,532,616,720,989`); `batch_vs_sequential_bench.py` 4/4 sin (`:33,88,107,195`). En Windows, handles abiertos (RocksDB/Fjall lock, Milvus lite server) dejan el dir bloqueado → `WinError 32`. Fix: `ignore_errors=True` en los 7 sitios + nota. Riesgo: `ignore_errors` puede ocultar fallos reales de limpieza → aceptable en benches (dirs temporales regenerables; precedente: 6 sitios ya lo usan).
- `requirements.txt`: 7 deps sin floor (`numpy, h5py, lancedb, chromadb, qdrant-client, psutil, tabulate`). Instalados localmente: numpy 2.5.2, psutil 7.2.2, tabulate 0.10.0, vantadb-py 0.5.0; NO instalados: h5py, lancedb, chromadb, qdrant-client → sus floors son [cita NO VERIFICADA — sin red].

## 8. INVESTIGACIÓN PROBLEMA — causa raíz (systematic-debugging Phase 1)

- Síntoma 1: bench corre con `--help`. Peor que lo reportado: ni siquiera hay `argparse`; `__main__` llama directo a ambas funciones. Root cause: falta de CLI parsing (script crecido desde prototipo sin frontera CLI).
- Síntoma 2: deps sin pin → installs no reproducibles (un `pip install -r` hoy puede traer majors incompatibles con APIs usadas: `PersistentClient`, `create_index(metric,num_partitions,num_sub_vectors)`, `query_points`, `put_batch_raw`).
- Síntoma 3: crash WinError32 por lock de archivos en Windows. Root cause: `shutil.rmtree` sin `ignore_errors` en paths de setup/teardown (los 6 sitios con `ignore_errors` ya asumen esta realidad; los 7 sin él son la inconsistencia).

## 9. INVESTIGACIÓN INTERNET

No se espera (stdlib `argparse`/`shutil` + patrón vecino en-repo). Sin red en runner → floors de `h5py/lancedb/chromadb/qdrant-client` marcados `[cita NO VERIFICADA — sin red]` con deuda TSYS-13 (ratificar versiones con red en Wave9+). Regla de validación AGENTS.md: al haber duda se declara, no se inventa.

## 10. VALIDACIÓN + CIERRE

- Verify contrato (§1, 4 comandos) + `git diff --check`.
- OCR delegation: `pwsh dev-tools/ocr-review.ps1` (Critical/High = bloquea, Medium → FIND-*).
- `/cleanCA` sobre `benchmarks/*.py` tocados antes de cerrar (norma ritual AGENTS.md — audita, solo informa; fixes solo los del contrato).
- DoD 3 niveles (contrato ✅ + task file sync + recitation) + reviewer distinto P2-01 (vanta-review, solo notas) + Gates D/V/C registrados en RESULTADO.
- Commit conventional `fix: FIND-72 — ...` (solo archivos propios: 2 `.py` + `requirements.txt` + este task file). Push vía vanta-lead. Backlog→avance NO tocar.

## Impacto mapeado (Regla 0 — MUST antes de la primera edición)

- **Archivos leídos completos:** `benchmarks/batch_vs_sequential_bench.py` (207 líneas), `benchmarks/requirements.txt` (30 líneas), `benchmarks/competitive_bench.py` (1073 líneas, lectura completa), `benchmarks/vantadb_local_bench.py:1-80,200-257` (patrón argparse + estilo), `.github/workflows/` (grep callers), `docs/operations/BENCHMARKS.md` (grep invocaciones).
- **Referencias hacia dentro (qué importa cada archivo):** stdlib (`argparse/time/random/os/shutil/math`, +`gc/json/platform/statistics/tempfile/urllib` en competitive) + `vantadb_py` (instalado 0.5.0) + terceros solo en competitive (`numpy/h5py/lancedb/chromadb/psutil/tabulate`, opcionales `qdrant_client/pymilvus` con guards).
- **Referencias entrantes (quién los usa):** 0 workflows usan los `.py` de este contrato (evidencia §2); docs citan solo `competitive_bench.py` con flags existentes (no se tocan); ningún `.py`/`.rs` importa estos benches (scripts standalone).
- **Veredicto:** impacto 🟢 bajo y localizado — 3 archivos, sin callers en CI, sin imports cruzados, sin hot path Rust, sin API pública SDK. Reversible por `git checkout` de 3 paths.

## Steps atómicos (~100 líneas c/u)

- [x] **Step 1 — CLI argparse en `batch_vs_sequential_bench.py`** (ACT ✅: `import argparse` + flags §4 con defaults exactos + rmtree ×4 `ignore_errors=True` + nota WinError32 en docstring. VERIFY ✅: `--help` sale 0 sin correr benches — RED previo confirmó bug: `--help` imprimía `Initializing Database...` y corría `run_bench()`; py_compile 0. DeprecationWarning `import vantadb_py` pre-existente → NOT TOUCHING.)
- [x] **Step 2 — pins + `competitive_bench.py` rmtree + cierre** (ACT ✅: floors en 7 deps + `pymilvus>=2.5` en comentario; rmtree ×3 `ignore_errors=True` + nota WinError32 en docstring §3. VERIFY: `py_compile` 0 + `git diff --check` limpio + `campaign_verify_cmd` bug exit -1 vacío reproducido → fallback bash directa (riesgo conocido del plan). `competitive_bench.py --help` sale 1 por guard pre-existente de deps ausentes (`h5py/lancedb/chromadb`, sin red para instalar) — idéntico pre/post cambio (verificado contra `git show HEAD:`), guard/argparse no tocados por el diff; con deps instaladas el argparse existente imprime ayuda sin correr benches.)

## Gate D (question-gates.md) — evaluado tras DISCOVERY, antes del task file

**No disparado** — blast radius 3 archivos sin hot path (motivo: sin callers CI ni API pública).
## Context Save Point

- DISCOVERY 2026-09-16: plan Task 27 + código leído + CI grepeado + Notion 4/4 + skills 6/8 + task file creado. Pendiente: Step 1 (CLI argparse batch) → Step 2 (pins + rmtree competitive + verify + commit).
- Deuda TSYS-13: ratificar con red los floors h5py/lancedb/chromadb/qdrant-client.
- CIERRE 2026-09-16: Steps 1-2 ✅ + review vanta-review **approve** (0 bloqueantes; HALLAZGO review: eran 4 rmtree bare, no 3 — fix los cubrió) + OCR delegation sin Critical/High (rule pack aplicado al diff, 0 findings) + `/cleanCA` self-audit: 0 🔴, 🟡 pre-existente no introducida (run_bench >20 líneas, import deprecated `vantadb_py` → NOT TOUCHING). Commit `fix:` solo 4 archivos propios. Push vía vanta-lead. Backlog→avance NO tocado (orquestador).
