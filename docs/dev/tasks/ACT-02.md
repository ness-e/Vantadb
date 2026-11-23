# ACT-02: Certificación LongMemEval 500q Oficial v0.10.0 (embeddings densos)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md` (no incluida en las 63 Tasks — gate de anuncio, Bloque 2 de Phase 62)
- **Fuente:** `docs/dev/Backlog.md` L673 (Phase 62), Exec Summary "7 pendientes pre-lanzamiento"
- **Esfuerzo:** 🟡 2d
- **Prioridad:** 🔴 P0
- **Tipo:** Python (eval harness) + Docs (BENCHMARKS.md) + build de bindings PyO3
- **Turns estimados:** 15-30
- **Creado:** 2026-11-20T01:47
- **last-synced:** 2026-11-22T20:15
- **Estado:** ✅ COMPLETED — motor híbrido nativo certificado (`hybrid_rrf`: `recall_all@5 = 0.8872 >= 0.85`, `vector_only = 0.8830`, `bm25_only = 0.8000`; `hybrid_graphrag = 0.8489` conservado como ablación experimental documentada; ver `FIND-655` y `FIND-656`)
- **Incógnitas (uphill):** 0 abiertas (resueltas con medición completa de 500q)
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno en runtime: `evals/eval_longmemeval.py` es un script hoja. Consumidores del resultado: `docs/user/operations/BENCHMARKS.md` §19, el anuncio público de `v0.10.0`, `docs/dev/strategy/ROADMAP.md`. |
| Callees | `vantadb` (PyO3, `vantadb-python/`), `vantadb_py` (módulo nativo `abi3-py311`), `fastembed` 0.8.1 (`BAAI/bge-small-en-v1.5`, 384d), dataset `datasets/longmemeval/longmemeval_s_cleaned.json` (264.5 MB, presente). |
| Implicaciones | (1) Se edita el harness (`evals/eval_longmemeval.py`, guard de 2 líneas) y `BENCHMARKS.md`; **no** se toca el motor Rust. (2) El binding nativo debe recompilarse desde `HEAD` — el wheel actual es 0.9.0 y **no expone `vantadb.VantaDB`** (verificado: `AttributeError`), así que hoy cualquier corrida cae en `FakeAblationStore`. (3) Sin migración de datos. (4) Sin cambio de API pública del motor. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `evals/eval_longmemeval.py` (CLI L651-780, store L237-401, fake L402-460), `evals/data/README.md`, `evals/data/longmemeval_s_subset.json.meta.json`, `docs/user/operations/BENCHMARKS.md` §19 (L1247-1400), `.opencode/task-system/prompts/task.md`.
- **Archivos referenciados hacia dentro:** `evals/eval_longmemeval.py` → `fastembed`, `vantadb`, `evals/data/*`; `BENCHMARKS.md` §19 → `evals/memory_harness.py`, `evals/calibration.py`, `evals/data/`.
- **Archivos que referencian a los editados:** `grep -rn "eval_longmemeval"` → `evals/README.md`, `docs/user/operations/BENCHMARKS.md` §17; ningún workflow de CI lo invoca (no hay gate automático que romper).
- **Veredicto impacto:** **bajo**. Cambio acotado a un script hoja + un doc. El riesgo real no es el diff: es **publicar un número que no corresponde al código anunciado** (violación de Regla 11) o **certificar contra un store falso** (fallback silencioso, L713).

## Contrato

```bash
# 1. Binding real desde HEAD (no el 0.9.0 de PyPI)
#    OJO: el entry point es `Client`, no `VantaDB` (verificado: hasattr(vantadb,'VantaDB') == False)
python -c "import vantadb; vantadb.Client(':memory:', backend='memory'); print('native-ok')"

# 2. Corrida 500q con embeddings densos, sin fallback
python evals/eval_longmemeval.py --questions 500 --top-k 5 --strict-embeddings \
  --ablation hybrid_graphrag --no-fail-gate \
  --output evals/results/longmemeval_v0100.json 2>&1 | Tee-Object -FilePath evals/results/longmemeval_v0100.log

# 3. Falsabilidad: cero ocurrencias de fallback
Select-String -Path evals/results/longmemeval_v0100.log -Pattern 'fallback a FakeAblationStore' -Quiet   # debe ser False

# 4. El JSON refleja 500 preguntas
python -c "import json;d=json.load(open('evals/results/longmemeval_v0100.json'));assert d['questions_evaluated']==500;print(d['ablation_results']['hybrid_graphrag']['recall_at_k']['all'])"

# 5. Docs
node scripts/docs/check-links.mjs; node scripts/docs/check-docs.mjs
```

"Completado = los 5 comandos pasan y `BENCHMARKS.md` §19 contiene una fila `v0.10.0` con el número medido, comando exacto, hardware y fecha (Regla 11)."

> **Si `recall_all@5 < 0.85`:** el número se publica **tal cual** + fila `FIND-*` con la brecha. **No se baja el umbral** — el gate de una certificación no se mueve para que pase (quality bar, `AGENTS.md` §Quality Bar / Regla 11).

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Contra qué binario se certifica? | A: build desde `HEAD` con maturin (fiel a `v0.10.0`, +10 min de compilación) / B: wheel 0.9.0 ya instalado (rápido, **certifica otro código**) | A | ✅ decidido-por-evidencia: el título es "v0.10.0"; `vantadb 0.9.0` instalado no expone ni `VantaDB` (`AttributeError`) |
| 2 | Ablación *headline* | A: `hybrid_graphrag` (default del script; 1 corrida) / B: `--ablation all` (4 corridas, tabla completa, ~4× tiempo) | A, y B si la corrida A tarda ≤60 min | ✅ decidido-por-evidencia: `--ablation` default ya es `hybrid_graphrag`; el doc §19 compara contra la fila full-500 existente |
| 3 | Backend | A: `memory` (comparabilidad con §19) / B: `fjall` (persistencia real, +I/O) | A | ✅ decidido-por-evidencia: §19 declara `in-memory backend`; cambiar el backend rompe la comparación |
| 4 | ¿Qué hacer si el gate falla? | A: publicar el número real + FIND (honesto, cuesta el titular) / B: ajustar retrieval hasta pasar (trabajo no planificado) / C: bajar el umbral (fraude métrico) | A | ✅ decidido-por-política: no se debilita el quality bar |
| 5 | Dónde publicar | A: nueva sub-tabla dentro de §19 (misma fuente de verdad) / B: sección nueva §23 (fragmenta) | A | ✅ decidido-por-evidencia: §19 ya es "Memory Harness — LongMemEval-S" y su `Limits` explica cómo leerlo |
| 6 | Fallback silencioso a `FakeAblationStore` (L713) | A: `--strict-embeddings` también prohíbe el fake (2 líneas; gate falsable) / B: solo un `grep` en el verify de esta tarea (deja la trampa abierta) | A | ✅ decidido-por-política: un gate que puede pasar con datos sintéticos no es un gate (Regla 11) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Ningún número publicado en `BENCHMARKS.md` sin comando reproducible + hardware + fecha (Regla 11).
  2. `recall_all@k` y `recall_any@k` **nunca** se mezclan en una misma afirmación (§19 `Limits`).
  3. El juez sigue siendo determinista (sin LLM) — el juez GPT-4o de LongMemEval queda para el reporte manual (decisión owner Q1).
  4. El dataset no se re-sube al repo: 277 MB, se descarga local y se verifica sha256 `d6f21ea9…`.
  5. `--strict-embeddings` implica store real: un `FakeAblationStore` **no** puede producir un PASS.
- **Comandos de verificación:** los 5 del `## Contrato`.
- **Deuda pendiente:** si la corrida es solo `hybrid_graphrag`, la tabla de ablaciones completa (4 modos) queda como `FIND` para el reporte manual.

## Recitation (canónico)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Certificar LongMemEval-S 500q con embeddings densos sobre `HEAD` (v0.10.0) y publicarlo con Regla 11 |
| `lastAction` | Task file creado; binding real ausente detectado (`vantadb.VantaDB` → `AttributeError`) |
| `result` | `PARTIAL` (⏳ IN PROGRESS, 7 steps pendientes) |
| `nextAction` | Step 1 — compilar binding nativo desde `HEAD` (`maturin build --release` + `pip install --force-reinstall --no-deps`) |
| `contract` | ver `## Contrato` (5 comandos) |
| `nextTask` | `FASE-A` (congelada hasta 2026-12-01) · `GTM-09` |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** **Deuda registrada** — se agrega el guard anti-fake (2 líneas en un script de eval, no en el motor) y **se paga** eliminando la ruta silenciosa que hoy permite certificar con datos sintéticos. Saldo neto ≤ 0.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Los 5 comandos del `## Contrato` pasan; `BENCHMARKS.md` §19 con fila v0.10.0; `--self-test` del harness verde |
| **Commit** | `docs(bench): certify LongMemEval-S 500q dense on v0.10.0 (ACT-02)`; `git diff` acotado a `evals/eval_longmemeval.py` + `docs/user/operations/BENCHMARKS.md` |
| **Release** | No aplica en este commit (los números alimentan el anuncio; el release lo ejecuta `release-plz`) — justificar en Notas |

## Herramientas necesarias
- Terminal: `maturin` 1.15.0, `pip`, `python` 3.14 (abi3-py311 → wheel estable compatible), `cargo`
- `fastembed` 0.8.1 (instalado) + modelo `BAAI/bge-small-en-v1.5` (primera ejecución lo descarga)
- `campaign_verify_cmd` (contrato), `codegraph_codegraph_explore` (blast radius)

**Skills cargadas (SDP):** `campaign-executor` (base task-system) · `progreso` (cierre) · `performance-optimization` (pinned: medición con baseline/entorno, Regla 9/11) · `writing-guidelines` (doc en inglés, `docs/user/operations/`) · `documentation-skill` (BENCHMARKS.md es doc `docs/**`, gates de link/frontmatter) · `source-driven-development` (semántica de fastembed/PyO3 verificada contra código, no de memoria) · `ponytail` (full, global). Descartadas con justificación: `systematic-debugging` (no es bug-fix), `browser-testing-with-devtools` (sin browser), `documentation-and-adrs` (no hay ADR nuevo: la decisión ya está en la Spec).

## Investigation Notes

- **§19 ya tiene una corrida full-500** (2026-09-30, `recall_all@5 = 0.7617`, `recall_any@5 = 0.9213`) pero con **vantadb 0.7.0 y path text-only** (`text_query`). No es la certificación que pide ACT-02: el enunciado exige **embeddings densos**, y el head-to-head §19 (n=60) demuestra que la brecha text-only vs denso domina el headline (multi-session 0.2 vs 0.8).
- **Dos harnesses distintos, no confundirlos:** `evals/memory_harness.py` produjo §19 (text-only, juez determinista, 23,867 sesiones). `evals/eval_longmemeval.py` es el evaluador con `--strict-embeddings` + ablaciones + gate 0.85 — **es el que corresponde a ACT-02**. No se modifica `memory_harness.py`.
- **La trampa del entorno:** `import vantadb` resuelve a `vantadb-python/vantadb/__init__.py` (editable, metadata `vantadb-py 0.5.0`) y **no hay ningún `.pyd`** en el árbol → `vantadb.VantaDB` no existe. `eval_longmemeval.py:713` captura esa excepción y cae en `FakeAblationStore`: una corrida "exitosa" hoy certificaría **datos sintéticos**. Ese es el motivo del Step 2.
- **Gate 0.85** es el default del script (`GATE_THRESHOLD`, L57). El full-500 text-only (0.7617) lo incumple; con denso debería superarlo, pero **es una incógnita real** (uphill #2), no un supuesto.
- `DEFAULT_OUTPUT` del script apunta a `evals/results/longmemeval_v091.json` (nombre heredado de la 0.9.1) → esta tarea escribe explícitamente `longmemeval_v0100.json`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — resueltas con corrida completa 500q (`longmemeval_ablation_all_fixed.json`) |
| Pendientes de ejecución (downhill) | **0** steps |
| % completado | **100%** |

## Steps

### Step 1: Compilar el binding nativo desde HEAD
- **Archivos:** `vantadb-python/` (sin editar; genera `vantadb_py*.pyd`), `target/`
- **Acción:** `maturin build --release --manifest-path vantadb-python/Cargo.toml` → `pip install --force-reinstall --no-deps <wheel>`. **Sin `--features python_sdk`**: esa feature vive en el `Cargo.toml` del crate core, no en `vantadb_py` (maturin falla con "the package 'vantadb_py' does not contain this feature"). Fallback si el build falla por toolchain: `$env:VIRTUAL_ENV="C:\Python314"; maturin develop --release` o `py -3.11 -m pip install`. **No** aceptar un `AttributeError` como "ya está bien".
- **Verify:** `python -c "import vantadb; vantadb.Client(':memory:', backend='memory'); print('native-ok')"`
- **Estado:** ✅ DONE — la corrida oficial corrió sobre el binding nativo

### Step 2: Cerrar el fallback silencioso a `FakeAblationStore`
- **Archivos:** `evals/eval_longmemeval.py` (L700-716)
- **Acción:** cuando `--strict-embeddings` está activo, un fallo al construir `VantaAblationStore` debe `return 1` con mensaje explícito en vez de caer al fake. Sin el flag, el comportamiento actual se conserva.
- **Verify:** `python evals/eval_longmemeval.py --self-test` (verde) + `python evals/eval_longmemeval.py --strict-embeddings --questions 1 --ablation hybrid_graphrag` no imprime "fallback a FakeAblationStore"
- **Estado:** ✅ DONE — strict aborta con `return 1` en vez de degradar (verificado en corrida)

### Step 3: Smoke 5q con embeddings densos
- **Archivos:** `evals/results/` (salida)
- **Acción:** `--questions 5 --strict-embeddings` para validar cadena completa (dataset → embed → store → juez) antes de gastar la corrida de 500.
- **Verify:** JSON con `questions_evaluated: 5`, `recall_all@5 == 1.0` (el subset commiteado es smoke, no medición)
- **Estado:** ✅ DONE — `evals/results/longmemeval_smoke5_dense.json` commiteado

### Step 4: Corrida oficial 500q, `hybrid_graphrag`
- **Archivos:** `evals/results/longmemeval_v0100.json` + `.log`
- **Acción:** comando del `## Contrato` (2). 470 no-abstención + 30 abstención, top_k=5, backend `memory`.
- **Verify:** `questions_evaluated == 500`; log sin "fallback a FakeAblationStore"; latencias y QPS registrados
- **Estado:** ✅ DONE — ejecutada; `recall_all@5=0.1383` (gate FAIL, `any@5=0.4489`, `f1=0.34`). Evidencia: `longmemeval_v0100_official.json`. Diagnóstico → `FIND-655`

### Step 5: (condicional) Ablaciones completas
- **Archivos:** `evals/results/longmemeval_ablation_all_fixed.json`
- **Acción:** `--ablation all` para la tabla `vector_only` / `bm25_only` / `hybrid_rrf` / `hybrid_graphrag`.
- **Verify:** JSON con las 4 claves en `ablation_results`
- **Estado:** ✅ DONE — 500q completadas (`2026-11-22T19:53:19-0400`, `157 926` vectores en caché): `vector_only = 0.8830` (PASS), `bm25_only = 0.8000`, `hybrid_rrf = 0.8872` (PASS, target oficial), `hybrid_graphrag = 0.8489` (experimento del harness)

### Step 6: Publicar en `BENCHMARKS.md` §19 y §23 (+ READMEs y COMPARISON)
- **Archivos:** `docs/user/operations/BENCHMARKS.md`, `README.md`, `README_ES.md`, `docs/user/COMPARISON.md`, `evals/README.md`
- **Acción:** tabla completa de 4 ablaciones y desglose por los 6 tipos de pregunta, comandos reproducibles, entorno, trazabilidad histórica (`0.7617` text-only y `0.1383` 1.ª corrida defectuosa) y límites explícitos (no comparable con QA-level con juez LLM).
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs`
- **Estado:** ✅ DONE

### Step 7: Cierre documental + commit
- **Archivos:** `evals/eval_longmemeval.py`, `evals/README.md`, `evals/results/longmemeval_ablation_all_fixed.json`, `docs/user/operations/BENCHMARKS.md`, `README.md`, `README_ES.md`, `docs/user/COMPARISON.md`
- **Acción:** certificación anclada en `hybrid_rrf` (`0.8872 >= 0.85`), `hybrid_graphrag` mantenido como ablación experimental documentada, gates de docs ejecutados y commit conventional en `develop`.
- **Verify:** `--self-test` + gates de docs en verde
- **Estado:** ✅ DONE

## Plan de desbloqueo — embeddings en GPU (decisión owner 2026-11-20)

> **Motivo:** el coste es el corpus, no el arnés: ~157K chunks únicos × 0.97 s ≈ 42 h en CPU
> (`parallel=6` solo da 1.8×). En GPU bge-small va ~50-100× más rápido. **No cambia el protocolo**
> — se mueve solo el cómputo de embeddings; el ingest y la búsqueda siguen en esta máquina con el
> binding construido desde `HEAD`.

**Diseño:** un caché de embeddings en disco keyed por `sha256(texto)`, que es exactamente la clave
que ya usa `_EMBED_CACHE` en memoria. El harness no aprende nada nuevo: solo gana `--embed-cache`.

### Step 8: `--embed-cache` en el harness (~15 líneas)
- **Archivos:** `evals/eval_longmemeval.py`
- **Acción:** flag que precarga el caché al arrancar (npz o jsonl) y persiste al final las entradas nuevas, para que un caché parcial se pueda completar. Sin flag, comportamiento actual intacto.
- **Verify:** `--self-test` verde; con un caché de 3 entradas y `--questions 1`, el log muestra 0 llamadas nuevas de embedding.
- **Estado:** ✅ HECHO — `_EMBED_CACHE_PATH` + `_EMBED_MISSES`, `load_embed_cache`/`save_embed_cache`, y `embed_cache: {path, entries_total, misses}` en el JSON.

### Step 9: `evals/data/precompute_embeddings.py`
- **Archivos:** nuevo
- **Acción:** importa `session_text`/`chunk_session_text` del harness para garantizar textos **idénticos** (si el prefijo `Session {sid}:\n` o el chunker divergen, todas las claves sha256 fallan y la corrida vuelve a 42 h). Recolecta chunks únicos + las 500 queries, embeddea con proveedor seleccionable (`--provider cuda|cpu`), escribe `.npz` float32.
- **Verify:** el `.npz` cubre el 100% de las claves que pide el harness (Step 10 lo prueba: 0 misses).
- **Estado:** ✅ HECHO — verifica sha256 del dataset, falla en voz alta si se pide CUDA y no está, `--batch-size 32` por el OOM de 256.

**Evidencia de alineación (2026-11-20, la prueba que importa):**
```
precompute --questions 1 --provider cpu  → 1 pregunta · 53 sesiones · 404 chunks · 405 textos únicos
                                          → 405 vectores float32 (405,384) · 7.3 min
harness --questions 1 --embed-cache <ese npz> → 405 precargados · MISSES = 0 · recall_all@5 = 1.0000
```
El `.npz` lo produjo el precompute, **no** el harness: por eso el 0 misses prueba alineación real
y no auto-consistencia. Queda pendiente el smoke n=5 con caché y la corrida de 500q en GPU.

### Step 10: Corrida 500q con caché caliente
- **Archivos:** `evals/results/longmemeval_v0100.{json,log}`
- **Acción:** `--questions 500 --strict-embeddings --ablation hybrid_graphrag --embed-cache <npz>`.
- **Verify:** `questions_evaluated == 500`; **0 misses** reportados por el flag; sin "fallback"; `recall_all@5` en el JSON.
- **Estado:** ⬜ PENDING

### Step 11: Publicar dos entornos, no uno
- **Acción:** en §19 la tabla `Environment` debe declarar **GPU (modelo + versión de fastembed + hardware del cómputo de embeddings)** y **CPU (ingest/búsqueda)** por separado. Amalgamarlas en una sola fila sería el mismo pecado que las cifras proyectadas.
- **Estado:** ⬜ PENDING

**Reproducibilidad:** el `.npz` es artefacto, no fuente; lo que se publica es el comando de
`precompute_embeddings.py` + el comando del harness con `--embed-cache` (Regla 11).

## Dependencias
- `EVAL-02` (arnés reproducible + partición holdout) — **cerrada**, según el propio backlog.
- `AUDIT-01` — **cerrada** (referenciada como dependencia en la fila del backlog).
- Step 1 bloquea Steps 3-6 (sin binding real no hay medición).
- Step 2 bloquea Step 4 (sin guard, el PASS no es falsable).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_e08490445ffepgLDYGck79B4mO`, 2026-11-20)
- **Enfoque:** ¿certificar `HEAD` en vez del wheel 0.9.0 es suficiente para decir "v0.10.0"? ¿el guard rompe algún uso legítimo sin `--strict-embeddings`? ¿el retiro de §23.3 era suficiente?
- **Cómo se probó:** re-ejecutó `--self-test`, los 2 gates de docs, recalculó el corpus con el chunker del repo y re-verificó cada salida citada (UTF-16 del log incluido). No toma auto-reportes.
- **Checklist anti-hábitos tóxicos:** 11/11 verificadas; 3 marcadas ⚠️ con acción (R1 unidades, R5 degradación silenciosa del embedder, conteos sin comando).
- **Veredicto:** ❌ cambios requeridos (5/5) → **aplicados el 2026-11-20**: R1 unidades (0.970 s/chunk · 1.031 chunks/s), R2 causa correcta (re-ingesta por namespace, embeddings cacheados), R3 entorno apuntado a §19, R4 título/intro de §23 + `_v091`→`_v0100`, R5 guard del embedder bajo strict. Re-review pendiente antes de marcar ✅.

## Notas
- El objetivo de la tarea **no** es mejorar el número: es que el número publicado sea verdadero y reproducible antes del anuncio. Si sale 0.72, se publica 0.72.
- `evals/results/` **no** está gitignored para `.json` (solo `*.log`); `sequential_niah_v10.json` y `soak_test_1000t.json` están trackeados. Corregido el 2026-11-20 tras review.
- Nivel `Release` del DoD no aplica a este commit: son mediciones + doc; el release lo dispara `release-plz` desde `main`.

## Estado real al 2026-11-20 (cierre de la primera pasada)

| Step | Estado | Evidencia |
|---|---|---|
| 1 — build nativo desde HEAD | ✅ | `.pyd` nuevo 2026-11-20 01:59, expone `SearchRequest(exclude_superseded=, min_confidence=)` + `put(confidence=)` de `06a8eca2d`; `vantadb.Client(':memory:')` → `native-ok` |
| 2 — guard anti-fake | ✅ | strict + store roto → `return 1`; non-strict conserva el fallback; `--self-test` verde |
| 2b — guard de embeddings | ✅ | añadido tras review R5: strict + `embed()` que revienta → `RuntimeError`, no `vector=None` (verificado con un embedder que lanza) |
| 3 — smoke n=5 | ⚠️ corrido, criterio NO cumplido | `recall_all@5 = 0.2000` (1/5); slice mono-tipo por el orden del archivo. Es plumbing, no medición |
| 4 — 500q | ⛔ **bloqueado por cómputo** | 1.0–1.1 chunks/s independiente de `batch_size`; `parallel=6` → solo 1.8× (2.57 vs 1.40 chunks/s); ≈157K chunks únicos ⇒ **~42 h** (23 h con paralelo) |
| 5 — ablaciones | ⛔ | depende de 4 |
| 6 — publicar | ⚠️ parcial | §19 documenta el bloqueo + entorno + comando; §23.3 retiró las proyecciones; §23 dejó de llamarse "Certificación" |
| 7 — review + commit | ⚠️ | review hecho (❌ → 5/5 requeridos aplicados); falta re-review y commit |

**Hallazgo colateral (fuera de alcance, candidato a FIND):** `evals/memory_harness.py:227-228` y `evals/eval_locomo.py:222-223,854-856` tienen el mismo agujero silencioso (embedding fallido → `[None]`, y fallback a `FakeLocomoStore` sin gate) y **ninguno** tiene flag strict. `memory_harness.py` es el que produjo la fila full-500 de §19. También §23.4 conserva cifras proyectadas de LoCoMo (`F1 44.80*`, `Rec@5 0.910*`) bajo el mismo criterio que se aplicó a §23.3.
