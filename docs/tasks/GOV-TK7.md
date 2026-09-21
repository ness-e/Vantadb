# GOV-TK7 — put_batch metadatas solo-str (Wave 1, Task 5)

> Plan: `docs/plans/2026-09-04-durability-release-readiness.md` (Task 5, Wave 1) · Backlog P417/GOV-TK7 · Ruta: vanta-worker
> Commit (solo si contrato pasa): `fix(api): put_batch metadata coercion alineada (GOV-TK7)`
> Estado: ✅ COMPLETO (2026-09-05, commit pendiente — ver Context Save Point)

## Contrato (ley)

Doc y API coinciden (una dirección, documentada) + test de coercion verde + suite afectada verde + clippy/fmt limpios.

## DISCOVERY — evidencia (2026-09-05, verificado contra HEAD `7361d2f6`)

### El drift

| Lado | Evidencia |
|---|---|
| `put` acepta metadatas tipados | `vantadb-python/src/lib.rs:795-808`: `metadata: Option<&Bound<PyDict>>` + `py_dict_to_metadata(metadata)` → `BTreeMap<String, VantaValue>` vía `py_any_to_value` (str/int/float/bool/datetime/list/None). Tests: `test_sdk.py:240` (`done: False` bool), `:339-341` (`score: 10/50/5` int) ✅ |
| `put_batch_raw` acepta metadatas tipados | `lib.rs:595,626-632`: `metadatas: Option<Vec<Option<Py<PyAny>>>>` + cast `PyDict` + mismo `py_dict_to_metadata`. Mismo helper, misma semántica que `put` |
| `put_batch` SOLO str | `lib.rs:486`: `metadatas: Option<Vec<Option<HashMap<String, String>>>>` + loop `:555-563` que envuelve cada valor en `VantaValue::String`. Int/float/bool → `TypeError` de extracción PyO3 (fail-loud pero divergente) |
| Tutorial `put` usa ints nativos | `docs/tutorials/02-local-rag-pipeline.md:110`: `metadata={"source": ..., "chunk_index": i, "total_chunks": len(chunks)}` (ints) ✅ funciona |
| Tutorial `put_batch` fuerza `str()` | `02-local-rag-pipeline.md:125`: `metadatas=[{"source": name, "chunk_index": str(i), "total_chunks": str(len(chunks))} ...]` — workaround manual por el solo-str |
| Tutorial internamente inconsistente | `02-local-rag-pipeline.md:143-144`: `r.metadata['chunk_index']+1` exige **int** — con el batch str-coerced (`"3"+1`) lanza `TypeError`. El workaround de L125 rompe L144 |
| Test `put_batch` fosiliza el solo-str | `test_sdk.py:621,633`: `metadatas=[None, {"type": "greek"}, None, {"rank": "4"}]` — `rank` es `"4"` string, no `4` int |
| Stubs ya prometen `dict` genérico | `vantadb_py.pyi:198` + `__init__.pyi:160`: `metadatas: list[dict \| None]` — `dict` sin params = Any values. La firma Rust es MÁS estrecha que el stub publicado |

### Mini-decisión (question-gate, sin ronda con usuario)

| Opción | Qué implica | Qué rompe |
|---|---|---|
| **A) Alinear doc** (documentar solo-str) | Fijar `str()` en tutorial + nota "solo-str" en PYTHON_SDK.md + test que fija TypeError para ints | Perpetúa divergencia `put` vs `put_batch`; obliga a reescribir L144 (`int(...)`); deja cliff TypeError para ints/floats/bools; contradice stubs `list[dict\|None]`; 3 superficies a tocar igual |
| **B) Ampliar coercion** (put_batch usa `py_dict_to_metadata`) | Firma `metadatas: Option<Vec<Option<Py<PyAny>>>>` + cast+helper (copia patrón `put_batch_raw:626-632`) + quitar `str()` del tutorial + test coercion | Backwards-compatible: todo dict str-only válido antes sigue extrayendo (PyDict cast + `py_any_to_value` acepta str); solo se AÑADE aceptación. 1 archivo Rust + tutorial + test |

**Decisión: B (ampliar coercion).** Rompe menos: es aditiva (ningún caller válido anterior falla), elimina la divergencia `put`/`put_batch`/`put_batch_raw` (un solo helper `py_dict_to_metadata`), repara la inconsistencia interna del tutorial (L125 vs L144) en vez de extenderla, y alinea Rust con los stubs ya publicados. Precedente: `put_batch_raw` ya hizo exactamente este camino (PyAny + cast + helper).

## Impacto mapeado (Regla 0) — OBLIGATORIO antes del primer edit

- **Archivos leídos completos (regiones):** `vantadb-python/src/lib.rs:1-42` (imports), `:459-576` (`put_batch` + docstring), `:578-642` (`put_batch_raw` patrón a seguir), `:752-822` (`put` patrón), `src/convert.rs:59-120` (`py_any_to_value`), `:639-706` (`py_dict_to_metadata`); `docs/tutorials/02-local-rag-pipeline.md:95-146`; `vantadb-python/tests/test_sdk.py:613-700` (tests put_batch), `:1376-1385` (raw metadata); stubs `vantadb_py.pyi:193-211` (ya genéricos, sin cambio).
- **Referencias hacia dentro (qué consume lo que edito):** `put_batch` consumido por `integrations/llamaindex/.../vectorstore.py:134` (`put_batch(entries)` — firma legacy ya removida en PY-QW2; verificar que no sea caller real del kwargs actual), tests `test_sdk.py`, stub `.pyi`, tutorial. Cambio de tipo de param PyO3: callers Python con dicts str-only no se afectan (extracción genérica los acepta).
- **Referencias entrantes (de qué dependo):** `py_dict_to_metadata` + `py_any_to_value` (existentes, testeados por `put`), `VantaMemoryInput.metadata: BTreeMap<String, VantaValue>` (core, sin cambio), `check_lens` (sin cambio). Cero código productivo del core tocado (solo binding).
- **Veredicto:** BLAST RADIUS = 1 archivo Rust (`vantadb-python/src/lib.rs`, ~30L) + 1 tutorial (1L) + 1 test (nuevo, ~25L). 0 core (`wal`/`vector`/`storage` intactos), 0 símbolos públicos nuevos (misma fn, tipo de param más ancho — documentado como dirección B), 0 callers Rust afectados (frontera PyO3). Gate D (question-gates): mini-decisión B resuelta con evidencia arriba (sin ronda). Workflow: bug-fix TDD (Prove-It: test RED primero con ints → TypeError actual, luego GREEN).
- **Nota scope:** `HashMap` import (`lib.rs:12`) puede quedar sin uso tras el cambio — verificar y limpiar si clippy lo marca. Async wrapper `__init__.py:274-294` pasa `metadatas` opaco (sin cambio). Stubs ya genéricos (sin cambio).

## SDP

`campaign_discover_skills` MCP no disponible en este runtime → SDP manual. Cargadas/aplicadas: `test-driven-development` (§3c: RED→GREEN, Prove-It, stack `cargo`+`pytest`), `incremental-implementation` (§3b: slices verticales, 1 fix + 1 doc + 1 test), `context-engineering` (§3a: context pack en este file). `systematic-debugging` en standby si VERIFY falla. Code-intel: `codegraph_explore` (blast radius + fuente verbatim) — codebase-memory-mcp no disponible en runtime.

## Steps atómicos (~100L c/u, un step por turno, cada uno reversible)

- [x] **S1 — RED** ✅ (`test_put_batch_metadata_coercion` → `TypeError: 'int' object is not an instance of 'str' while processing 'metadatas'` — razón correcta)
- [x] **S2 — GREEN** ✅ (`put_batch`: `Option<Vec<Option<Py<PyAny>>>>` + cast `PyDict` + `py_dict_to_metadata`, patrón `put_batch_raw:626-632`; docstring; import `VantaValue` removido — `HashMap` se conserva, usado en `:2006/:2030`)
- [x] **S3 — DOC** ✅ (tutorial L125 sin `str()` — ints nativos, consistente L110+L144; nota coercion en `PYTHON_SDK.md put_batch()`)
- [x] **S4 — VERIFY+CIERRE** ✅ (ver Context Save Point)

## Context Save Point

- COMPLETO 2026-09-05 en branch `develop`. Contrato: doc↔API dirección B (ampliar coercion) + test coercion verde + suites verdes + fmt/clippy propios limpios.
- Verify: `pytest test_sdk.py` 75/75 (incl. nuevo `test_put_batch_metadata_coercion`) · `test_stub_drift.py+test_perf_15_16.py` 16/16 · `cargo fmt --check` 0 · `cargo check` 0 · `cargo clippy` crate `vantadb_py` 0 warnings (el `-D warnings` global falla SOLO por `dead_code apply_delete` en `vantadb` core — WIP ajeno de otra sesión en `src/storage/engine/`, fuera de blast radius).
- Rebuild binding: `maturin build` + wheel `.pyd` refrescado en `vantadb-python/vantadb_py/` (gitignored, no commiteado).
- NOTICED BUT NOT TOUCHING: `integrations/llamaindex/.../vectorstore.py:134` llama `put_batch(entries)` legacy (roto desde PY-QW2) — pre-existente, candidata FIND-* del orquestador; `M .opencode` submodule ajeno intacto.
