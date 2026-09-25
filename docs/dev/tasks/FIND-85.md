# FIND-85 — matriz wheels + firma `put_batch_raw`

> Plan: `docs/dev/plans/2026-09-15-find-correcciones.md` (Task 23, Wave7) · Appetite 1d · 🟡 · Ruta vanta-lead (CI) + worker
> Estado: ⬜ PENDING → ⏳ IN PROGRESS (2026-09-15)

## Contrato

Matriz 3.12/3.14 añadidas en `release-wheels-60.yml` (o classifiers recortados en pyproject)
+ firmas `put_batch_raw` comparadas 3 vías (rs/pyi/async) + pytest verde.
`probe_lock_db/` solo limpieza local documentada, NO parte del contrato CI.

## Tipo + SDP

- `campaign_detect_task_type` → **python** (Python SDK), checks: `python -m pytest vantadb-python/tests/ -v`
- `campaign_discover_skills_v2` phase=BUILD keywords=[wheels-matrix, classifiers-sync, put-batch-signature, maturin-ci] → 8 skills
- `SDP: campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, frontend-ui-engineering, api-and-interface-design`
  (cargadas: source-driven-development, test-driven-development, api-and-interface-design, incremental-implementation, doubt-driven-development, context-engineering; campaign-executor+progreso base auto; frontend-ui-engineering N/A — sin UI)

## Gate D — no dispara

Blast radius = 1 archivo editado (`vantadb-python/pyproject.toml`, metadata, −2 líneas + comentario).
Sin hot path, sin símbolos públicos nuevos, sin cambio de comportamiento.
Contrato con salida explícita ("recorte honesto es salida válida"). Sin `question`.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `.github/workflows/release-wheels-60.yml` (310L),
  `vantadb-python/pyproject.toml` (65L), `vantadb-python/src/lib.rs:800-972` (`put_batch_raw`),
  `vantadb_py/vantadb_py.pyi:193-211` + `:362-370`, `vantadb_py/__init__.pyi:189-198`,
  `vantadb_py/__init__.py:361-391`, `tests/test_stub_drift.py` (335L completo),
  `vantadb-python/Cargo.toml` (27L), `.gitignore:134`, `vantadb-python/.gitignore:7`.
- **Hacia dentro (qué toco):** solo `pyproject.toml` classifiers. Workflow NO tocado
  (matriz es OS/arch ×4, no hay matriz python-version que ampliar; build usa un solo
  intérprete 3.11 + verify 3.13).
- **Entrantes (quién depende):** maturin lee classifiers (metadata wheel);
  `test_stub_drift.py` guarda firmas (no toca classifiers); `verify_published_wheel.py`
  corre en CI post-publish. Nada importa los classifiers en runtime.
- **Veredicto:** impacto metadata-only. Reversible (`git revert`). Sin riesgo funcional.

## Hallazgo DISCOVERY (decisión de fix)

- `Cargo.toml:15` → `pyo3 abi3-py311`: el wheel es **stable-ABI** (`cp311-abi3`),
  un solo artefacto por plataforma válido en ≥3.11. Ampliar matriz python-version
  duplicaría tiempo CI con beneficio cero → **recorte honesto** (salida válida del contrato).
- `pyproject.toml:10` `requires-python = ">=3.11"` se mantiene (pip permite 3.12/3.14,
  el wheel abi3 funciona); solo los classifiers pasan a listar lo que CI verifica: 3.11 + 3.13.
- Firmas 3 vías (evidencia, ver Step 2): rs `lib.rs:810`
  `(vectors, keys, payloads=None, metadatas=None, namespaces=None, ttls=None)` ==
  sync `.pyi` Client `:203-211` == MemoryClient `:362-370` (mismo orden/nombres/defaults).
  Async (`__init__.py:361` + `.pyi:189`) usa cola keyword-only — diseño intencional
  con cola kwonly compartida con `AsyncClient.put_batch` (`:340`), stub↔impl cubierto por
  `test_async_wrapper_stub_matches_real_asyncclient`. **Sin drift real → documentar, no tocar.**
- `probe_lock_db/`: existe local (335MB), ignorado (`vantadb-python/.gitignore:7` +
  raíz `:134` + maturin `exclude` `:52`) → `rmdir` local + documentar. No commit.

## Steps

### Step 1 — recorte classifiers + comentario abi3 ✅ DONE

Editar `vantadb-python/pyproject.toml:23-26`: quitar `3.12`/`3.14`, añadir comentario
(abi3-py311 → un wheel por plataforma; CI verifica 3.11 build + 3.13 install).
Verify: `python -c tomllib.load` + `git diff --check`.
Evidencia: `tomllib` parsea OK, classifiers Python = `3, 3.11, 3.13`;
`git diff --check` exit 0 (warnings LF solo de WIP ajeno pre-existente).

### Step 2 — 3 vías + probe + pytest + commit ✅ DONE

1. Check mecánico 3 vías: `inspect.signature(nativo.put_batch_raw)` vs AST de ambos
   `.pyi` (sync) + async wrapper vs su stub (veredicto esperado: sin drift).
2. `rmdir vantadb-python/probe_lock_db` local (documentar; ignorado, no commit).
3. `pytest test_stub_drift.py + test_sdk.py -k put_batch_raw + test_perf_15_16.py` verde.
4. Commit `ci: FIND-85 — ...` solo con `pyproject.toml` + este task file.
   Backlog→avance NO tocar (race paralelo, orquestador). Push vía vanta-lead.

## Prohibidos (NO TOCAR)

`.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4,
`Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `reparacion.bat`,
FIND-84 (`integrations/`), FIND-80 (`fuzz/`, `fuzz-40.yml`, `docs/dev/workflow/fuzz-40.md`),
`docs/pipeline-state.json`, `Cargo.toml` workspace (solo lectura).
WIP ajeno visto en `git status` (`.opencode`, `Justfile`, `completions/`, tauri lock,
`pipeline-state.json`, `ocr-*`, `reparacion.bat`) → commit solo archivos propios.

## Context Save Point

- DISCOVERY completo 2026-09-15: nativo importable (py3.14.7), 7 tests stub_drift coleccionados.
- Si se interrumpe tras Step 1: reanudar en Step 2 (diff en worktree intacto).

## Evidencia Step 2 (2026-09-15)

- 3 vías mecánico: nativo `inspect.signature` =
  `(vectors, keys, payloads, metadatas, namespaces, ttls)` REQ={vectors,keys} ==
  `Client` stub == `MemoryClient` stub (MATCH True ×2); async stub
  `(vectors, keys, *, payloads, metadatas, namespaces, ttls)` == async real
  (diseño keyword-only intencional, consistente con `AsyncClient.put_batch`).
  **Veredicto: sin drift real → documentado, 0 líneas de firma tocadas.**
- `probe_lock_db/`: 335MB local eliminado (`Test-Path` False post-rmdir;
  `git status` limpio — ignorado, no commiteado).
- pytest: `test_stub_drift.py` 7/7 ✅ + `test_sdk.py -k put_batch` /
  `test_perf_15_16.py` 14/14 ✅ (6.03s).
- Doubt: cross-model omitido (contexto no-interactivo, se anuncia).
