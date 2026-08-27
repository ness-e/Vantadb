# TASK PROV-07: ValueError en distance_metric inválido; warning en metadata descartada (los 3 crates)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-providers-quickwins.md`
- **Fuente:** Wave 1 · Task 4 (Backlog PROV-07 — INV-providers-01 H-09)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** Baja-Media (quickwin mecánico — validación explícita)
- **Tipo:** Rust (PyO3 binding)
- **Turns estimados:** 3
- **Creado:** 2026-08-26T23:30
- **last-synced:** 2026-08-26T23:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes (verify-only — 3/3 steps ✅)

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `providers/openai/src/python.rs::VantaDBOpenAI::search`, `providers/litellm/src/python.rs::VantaDBLiteLLM::search`, `providers/ollama/src/python.rs::VantaDBOllama::search` (Python consumers `search(distance_metric=...)`), `providers/*/src/python.rs::store` (Python `store(metadata=...)`) |
| Callees | `pyo3::exceptions::PyValueError::new_err`, `pyo3::types::PyDict::import("warnings").call_method1("warn", ...)`, `vantadb::DistanceMetric::{Cosine,Euclidean}`, `VantaMemorySearchRequest`, `VantaMemoryInput::metadata` |
| Implicaciones | Contrato rompe **a propósito** en error-case: antes `distance_metric` inválido caía silencioso a `Cosine` (`_ => Cosine`); ahora levanta `ValueError` — comportamiento correcto y esperado por el contrato. No rompe callers válidos (cosine/euclidean/l2/None siguen idénticos). Warning en metadata es aditivo (antes silent drop, ahora `warnings.warn` + drop) — no cambia tipo de retorno. No afecta performance/memoria/serialización. Sin migración. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `providers/openai/src/python.rs` (349L), `providers/litellm/src/python.rs` (356L), `providers/ollama/src/python.rs` (358L), `src/node/vector_data.rs` (DistanceMetric enum), `providers/openai/Cargo.toml` / `providers/litellm/Cargo.toml` / `providers/ollama/Cargo.toml` (23L c/u), `docs/plans/2026-08-25-research-providers-quickwins.md`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `python.rs` → `pyo3::{prelude::*, exceptions::PyValueError, types::{PyDict,PyList,PyModule}}`, `vantadb::{config::VantaConfig, error::VantaError, sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryRecord, VantaMemorySearchRequest, VantaValue}}`, `std::collections::HashMap`, `std::time::{SystemTime,UNIX_EPOCH}`; `Cargo.toml` → `vantadb = { path="../.." }`, `pyo3 = 0.29`
- **Archivos que referencian a los editados (referencias entrantes):** `rg "distance_metric" providers/` → 3× python.rs search; `rg "dropped_keys|warnings.*warn" providers/` → 3× store; `rg "VantaDBOpenAI|VantaDBLiteLLM|VantaDBOllama" providers/` → tests `providers/*/tests/test_*.py` (10+ instanciaciones por crate); `rg "DistanceMetric" src/` → `src/node/vector_data.rs:11`
- **Veredicto impacto:** BAJO — fix mecánico de validación en 2 métodos por crate (search 8L + store 10L). Ya implementado en HEAD commit 2754c783. No mueve Cargo features/workspace. Riesgo: callers que pasaban typo silencioso ahora reciben ValueError (intencionado, documentado). Warning es best-effort (si `warnings` import falla, propaga PyErr — aceptable).

## Contrato
"Compila + caso de test manual documentado (distance_metric inválido → ValueError; metadata descartada → warning)" — `docs/plans/2026-08-25-research-providers-quickwins.md:19` — Wave 1 Task 4
- **Cita contrato plan:** `PROV-07 | ValueError en distance_metric inválido; warning en metadata descartada (los 3 crates) | Compila + caso de test manual documentado`
- **Verify mecánico:** `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 + `cargo check --manifest-path providers/litellm/Cargo.toml` exit 0 + `cargo check --manifest-path providers/ollama/Cargo.toml` exit 0 + grep `PyValueError.*invalid distance_metric` 3 matches + grep `warnings.*warn.*dropping metadata` 3 matches + casos manuales documentados en ## Steps

## Spec (SDD — feature-add check Phase 1b)
No es feature-add (no agrega símbolo público nuevo): `search(distance_metric: Option<String>)` y `store(metadata: Option<PyDict>)` ya existían en las 3 clases. El cambio solo **endurece** el comportamiento: search valida el string existente y levanta ValueError en rama `Some(other)`; store emite `warnings.warn` cuando descarta tipos no soportados. No agrega `pub fn`/endpoint/binding/método nuevo. Por tanto, `## Spec` no requerida (gate P/D no dispara). Justificación por evidencia: `git show 2754c783 --stat` muestra diff en los mismos métodos sin nuevas pymethods.

| # | Decisión | Opciones | Default recomendado | Resuelto |
|---|----------|----------|---------------------|----------|
| 1 | Mensaje ValueError | genérico vs específico con valor recibido | específico `invalid distance_metric '{other}': expected "cosine", "euclidean" or "l2"` | ✅ decidido-por-evidencia (commit 2754c783 ya usa mensaje específico, grep 3× idéntico) |
| 2 | Mecanismo warning | `eprintln!` / `log` / `warnings.warn` | `warnings.warn` (Python stdlib, capturable con `pytest.warns` / `warnings.catch_warnings`) | ✅ decidido-por-evidencia (proveedores ya usan `py.import("warnings")?.call_method1("warn", ...)` en 2754c783) |

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** `distance_metric` válido (`None`/`"cosine"`/`"euclidean"`/`"l2"`) → mapea a `DistanceMetric::Cosine/Euclidean` idéntico a antes; `store` con metadata `str/bool/int/float` → inserta sin warning; GIL release (`py.detach`) intacto en search/store; `exclude_superseded`/`search_profile` agregados en 2754c783 no se tocan.
- **Comandos de verificación:** `cargo check --manifest-path providers/openai/Cargo.toml` + `providers/litellm` + `providers/ollama` exit 0; `Select-String -Path providers/*/src/python.rs -Pattern "invalid distance_metric"` 3 matches; `Select-String -Path providers/*/src/python.rs -Pattern "dropping metadata"` 3 matches; casos manuales en Step 3
- **Deuda pendiente:** ninguna — fix mecánico cerrado sin deuda nueva (Regla 6 saldo 0)

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | PROV-07 — ValueError distance_metric + warning metadata en los 3 crates |
| `lastAction` | discovery + verify (grep ValueError 3 matches + grep warnings 3 matches + cargo check x3 exit 0 — implementación ya en HEAD 2754c783) |
| `result` | PARTIAL (steps verify-only, task file creado) |
| `nextAction` | Step 1 grep ValueError → Step 2 grep warnings → Step 3 cargo check x3 + casos manuales → close |
| `contract` | Contrato + Invariantes + evidencia (ver arriba) |
| `nextTask` | PROV-08 (según plan Wave 1) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** Sin deuda — fix reemplaza silent fallback por validación explícita y silent drop por warning (0 líneas netas vs deuda; reemplazo 1:1). No introduce deuda. Payoff: elimina drift silencioso H-09 (INV-providers-01). Si se quisiera extraer helper compartido (~500L duplicadas) sería PROV-05, no este task.

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | `cargo check` ×3 exit 0 + grep ValueError 3 matches + grep warnings 3 matches + casos manuales documentados (ver Steps) ✅ |
| **Commit** | No commit (instrucción: no commitear — lead commitea). Si hubiera diff, sería `fix(providers): PROV-07 validación distance_metric + warning metadata` con diff limpio |
| **Release** | No aplica (crates `publish=false` fuera workspace) — verify `publish=false` ok |

## Herramientas necesarias
- cargo check (`--manifest-path providers/*/Cargo.toml` ×3)
- Select-String / rg grep (verify patrones)
- codegraph_explore (blast radius — opcional, ya mapeado)
- campaign_verify_cmd (verificación mecánica)

**Skills cargadas (SDP):**
- `source-driven-development` — verificar PyO3 0.29 patterns (`PyValueError::new_err`, `py.import("warnings").call_method1("warn")`) contra docs oficiales — Lifecycle BUILD
- `ponytail` (full) — ladder: stdlib `warnings` + 1-liner match + `Vec<String>` dropped_keys, sin abstracción — base
- `systematic-debugging` — bug silent-fallback requiere Iron Law (root cause: `_ => Cosine` sin validación) — Lifecycle VERIFY
- `incremental-implementation` — slice vertical delgado (search 8L + store 10L) — Lifecycle BUILD
- `doubt-driven-development` — stakes producción (input validación → ValueError contract) — Lifecycle BUILD
- `code-review-and-quality` — gate pre-commit 5 ejes (correctitud, simplicidad, consistencia) — Lifecycle REVIEW
- `campaign-executor` — núcleo task system PLAN/ACT/VERIFY
- `progreso` — migración Backlog → docs/avance al cierre
- SDP: keywords grep `distance_metric|metadata|warning|ValueError|pyo3` en `SKILLS-MANIFEST.md` → sin candidatos adicionales más allá de los 8 listados (Essential 37 + Engineering Lifecycle 12 cubiertos) — `SDP: sin candidatos adicionales (keywords: distance_metric/metadata/warning/ValueError/pyo3)`

## Investigation Notes
- **STACK DETECTED:** `pyo3 0.29` (Cargo.toml en 3 crates), `vantadb 0.5.0`, `DistanceMetric` en `src/node/vector_data.rs:11` (Cosine default, Euclidean, SparseDot). `store` metadata usa `VantaValue::{String,Bool,Int,Float}`.
- **Source PyO3:** `pyo3::exceptions::PyValueError::new_err` — patrón oficial PyO3 para mapear a Python `ValueError` (docs.rs/pyo3 0.29 exceptions). `py.import("warnings")?.call_method1("warn", (format!(...),))?` — patrón estándar para emitir `UserWarning` capturable (CPython `warnings.warn`, PyO3 `PyModule::import` + `call_method1`). No requiere `unsafe`.
- **Evidencia código (HEAD 2754c783):** `providers/openai/src/python.rs:179-187` + `providers/litellm/src/python.rs:231-239` + `providers/ollama/src/python.rs:139-147` → `match distance_metric.as_deref() { None|Some("cosine")=>Cosine, Some("euclidean"|"l2")=>Euclidean, Some(other)=>Err(PyValueError::new_err(format!("invalid distance_metric '{other}'..."))) }`. Antes: `Some("euclidean"|"l2")=>Euclidean, _=>Cosine` (silent). `store` → `providers/openai/src/python.rs:244-270`, `providers/litellm/src/python.rs:295-321`, `providers/ollama/src/python.rs:205-231` → `dropped_keys: Vec<String>` + `if !dropped_keys.is_empty() { py.import("warnings")?.call_method1("warn", (format!("dropping metadata keys..."),))?; }`.
- **Git:** `git show 2754c783 --stat` confirma fix en 12 archivos (3 crates × python.rs + tests + pyi + README). `git diff HEAD -- providers/*/src/python.rs` vacío hoy — fix ya en HEAD.
- **Ponytail:** skipped: helper compartido `validate_distance_metric()` extraído a crate common (PROV-05), enum custom para métricas, feature gate — add when PROV-05 se ejecute. Hoy 3× copia idéntica 8L es más corto que abstracción prematura.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado (match con ValueError + warnings.warn), evidencia en HEAD |
| Pendientes de ejecución (downhill) | 3 — Step 1 grep ValueError, Step 2 grep warnings, Step 3 cargo check x3 + casos manuales |
| % completado | 60% (discovery done, task file creado, verify pendiente) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — evaluado: toca validación de input usuario (`distance_metric: Option<String>` desde Python) — hardening positivo (antes bypass silencioso, ahora ValueError explícito). No introduce trust boundary nuevo, no toca auth/sesiones, no agrega dependencias (`cargo deny` no necesario). Checklist `security-and-hardening` N/A — justificado: `PyValueError` es mapeo seguro, no hay `unwrap`/`expect` nuevo, `unsafe` no usado.
- [x] **PERFORMANCE** — evaluado: no toca hot path (search/store son wrappers PyO3 → `engine.search/put` que sí son hot, pero validación es O(1) match + warning O(k) sobre metadata keys k≤~10). Sin benchmark requerido (Regla 9 no aplica — no optimización, solo validación). Justificación: branch adicional constant-time, no loops sobre datos.

## Steps
### Step 1: Verificar ValueError en distance_metric inválido (los 3 crates) ✅
- **Archivos:** `providers/openai/src/python.rs:179-187`, `providers/litellm/src/python.rs:231-239`, `providers/ollama/src/python.rs:139-147`
- **Acción:** Confirmar `match distance_metric.as_deref()` con rama `Some(other) => Err(PyValueError::new_err(format!("invalid distance_metric '{other}'...")))` en los 3 archivos. Ejecutar `Select-String -Path providers/*/src/python.rs -Pattern "invalid distance_metric"` y verificar 3 matches (uno por crate) + `PyValueError` import.
- **Verify:** `Select-String -Path providers/openai/src/python.rs -Pattern "invalid distance_metric"` → 1 match L183 ✅; `providers/litellm` L235 ✅; `providers/ollama` L143 ✅; `Select-String -Pattern "PyValueError" providers/openai/src/python.rs` → presente L183 ✅; total 3 ValueError branches verificadas
- **Estado:** ✅ COMPLETED (2026-08-26T23:31 — sin edición, ya en 2754c783)

### Step 2: Verificar warning en metadata descartada (los 3 crates) ✅
- **Archivos:** `providers/openai/src/python.rs:244-270`, `providers/litellm/src/python.rs:295-321`, `providers/ollama/src/python.rs:205-231`
- **Acción:** Confirmar `let mut dropped_keys: Vec<String>` + `None => dropped_keys.push(key)` + `if !dropped_keys.is_empty() { py.import("warnings")?.call_method1("warn", (format!("dropping metadata keys with unsupported value types..."),))?; }` en los 3 archivos. Ejecutar `Select-String -Path providers/*/src/python.rs -Pattern "dropping metadata"` y verificar 3 matches.
- **Verify:** `Select-String -Path providers/openai/src/python.rs -Pattern "dropping metadata"` → 1 match L267 ✅; `providers/litellm` L318 ✅; `providers/ollama` L228 ✅; `Select-String -Pattern "dropped_keys" providers/openai/src/python.rs` → 3 matches (init/push/check) ✅; total 3 warning blocks verificados
- **Estado:** ✅ COMPLETED (2026-08-26T23:31 — sin edición, ya en 2754c783)

### Step 3: Verificar crate compila ×3 + casos de test manual documentados ✅
- **Archivos:** `providers/openai/Cargo.toml`, `providers/litellm/Cargo.toml`, `providers/ollama/Cargo.toml`, `providers/openai/src/python.rs`, `providers/litellm/src/python.rs`, `providers/ollama/src/python.rs`
- **Acción:** Ejecutar `cargo check --manifest-path providers/openai/Cargo.toml` + `providers/litellm` + `providers/ollama` y verificar exit 0. Documentar casos manuales (ver subsección).
- **Verify:** `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 (3.24s) ✅; `cargo check --manifest-path providers/litellm/Cargo.toml` exit 0 (6.97s) ✅; `cargo check --manifest-path providers/ollama/Cargo.toml` exit 0 (3.48s) ✅; `cargo fmt --check` sobre providers no requerido (fuera workspace) pero `rustfmt` ya pasó en 2754c783
- **Estado:** ✅ COMPLETED (2026-08-26T23:32)

#### Casos de test manual — PROV-07 (documentado, reproducible tras `maturin develop`)
> **Pre-requisito:** `pip install maturin && maturin develop --manifest-path providers/openai/Cargo.toml` (repetir para litellm/ollama) + `pip install openai litellm ollama` (mocks permitidos). Los casos usan `warnings.catch_warnings(record=True)` y `pytest.raises`.

**Caso 1 — distance_metric inválido → ValueError (los 3 crates, idéntico)**

```python
import tempfile, warnings
from vantadb_openai import VantaDBOpenAI  # análogo: vantadb_litellm.VantaDBLiteLLM, vantadb_ollama.VantaDBOllama

store = VantaDBOpenAI(tempfile.mkdtemp(), api_key="sk-fake")
# search requiere namespace + embedding; distance_metric es el 5º arg keyword-only
try:
    store.search(namespace="ns", query_embedding=[0.1,0.2,0.3], distance_metric="dot_product", top_k=5)
    assert False, "debe levantar ValueError"
except ValueError as e:
    assert "invalid distance_metric 'dot_product'" in str(e)
    assert "cosine" in str(e) and "euclidean" in str(e)
    print("PASS — ValueError con mensaje específico:", e)

# Variantes que también deben fallar:
for bad in ["", "COSINE", "Cosine", "euclid", "l2 ", " manhattan"]:
    try:
        store.search(namespace="ns", query_embedding=[0.1,0.2,0.3], distance_metric=bad)
        assert False, f"debe fallar para {bad!r}"
    except ValueError:
        pass
print("PASS — variantes inválidas todas levantan ValueError")

# Casos válidos (no deben levantar):
for good in [None, "cosine", "euclidean", "l2"]:
    # no levanta ValueError; puede devolver [] si namespace vacío, pero no error de validación
    store.search(namespace="ns", query_embedding=[0.1,0.2,0.3], distance_metric=good, top_k=1)
print("PASS — casos válidos no levantan")
```
- **Esperado:** `ValueError: invalid distance_metric 'dot_product': expected "cosine", "euclidean" or "l2"` (mensaje exacto de `python.rs:183-185`). Antes del fix devolvía resultados con Cosine silencioso — ahora falla rápido.
- **Evidencia código:** `providers/openai/src/python.rs:179-187` (idem litellm:231-239, ollama:139-147)

**Caso 2 — metadata descartada → warning (los 3 crates, idéntico)**

```python
import tempfile, warnings
from vantadb_openai import VantaDBOpenAI

store = VantaDBOpenAI(tempfile.mkdtemp(), api_key="sk-fake")
emb = [0.1]*8  # dim dummy; VantaDB acepta cualquier len (search lo valida contra índice)

with warnings.catch_warnings(record=True) as w:
    warnings.simplefilter("always")
    rid = store.store("hello", emb, metadata={
        "ok_str": "val",
        "ok_int": 42,
        "ok_float": 3.14,
        "ok_bool": True,
        "bad_list": [1,2,3],
        "bad_dict": {"a": 1},
        "bad_none": None,  # None tampoco es str/bool/int/float → descartado
    })
    assert len(w) == 1, f"esperaba 1 warning, hubo {len(w)}"
    msg = str(w[0].message)
    assert "dropping metadata keys" in msg
    assert "bad_list" in msg and "bad_dict" in msg and "bad_none" in msg
    assert "ok_str" not in msg  # claves válidas no aparecen
    print("PASS — warning emitido:", msg)
    print("PASS — rid:", rid)

# Verificar que el record guardado solo tiene claves válidas:
ns, key = rid.split(":", 1)
rec = store.get(namespace=ns, key=key)
assert rec is not None
assert rec["metadata"]["ok_str"] == "val"
assert rec["metadata"]["ok_int"] == 42
assert "bad_list" not in rec["metadata"]
print("PASS — metadata filtrada correctamente en storage")

# Caso sin warning (todo válido):
with warnings.catch_warnings(record=True) as w:
    warnings.simplefilter("always")
    store.store("hello2", emb, metadata={"a": "x", "b": 1})
    assert len(w) == 0, "no debe emitir warning si todo es válido"
print("PASS — sin warning cuando metadata es válida")
```
- **Esperado:** `UserWarning: dropping metadata keys with unsupported value types (expected str/bool/int/float): bad_list, bad_dict, bad_none` (formato `python.rs:267`). Warning es capturable con `warnings.catch_warnings` / `pytest.warns`.
- **Evidencia código:** `providers/openai/src/python.rs:244-270` (idem litellm:295-321, ollama:205-231)

**Nota:** ambos casos son idénticos en los 3 crates (código copiado 1:1, pendiente de unificar en PROV-05). El test manual de un crate valida el contrato de los tres.

## Dependencias
- Ninguna dentro del plan (Wave 1 Task 4 — independiente; comparte Wave 1 con PROV-01/03/06/08). No bloquea ni es bloqueado por PROV-06 (timeout wiring).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** doubt-driven-development (contexto fresco adversarial — validación de inputs, contrato público)
- **Enfoque:** ¿ValueError mensaje específico correcto? ¿No rompe callers válidos? ¿warnings.warn capturable y no paniquea si warnings module falla? ¿Dropped keys no pierde claves válidas?
- **Cómo se probó:** grep mecánico 3× ValueError + 3× warnings + cargo check x3 exit 0 + casos manuales documentados reproducibles (ver Step 3). Cita commit 2754c783 como evidencia de implementación.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (cargo check timings reales 3.24s/6.97s/3.48s)
  - [x] No saltarse clarificación (contrato mecánico claro: ValueError + warning)
  - [x] No declarar done sin verify mecánico (grep + cargo check)
  - [x] No ignorar fallos parciales (verifica los 3 crates, no solo 1)
  - [x] No single-search (grep + cargo check + code read + git show)
  - [x] No copiar sin citar (cita commit 2754c783 + líneas exactas)
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos pasos (3 steps cerrados)
  - [x] No degradar chequeo errores (PyResult + ? en warnings import)
  - [x] No gastar presupuesto infinito (verify-only, sin edición)
- **Veredicto:** ✅ approve (degraded — single-context, fix ya en HEAD; casos manuales documentados sin ejecución live por requerir maturin wheel, justificado)

## Notas
- Fix ya merged en 2754c783 (2026-08-26) con Wave 1 completo (PROV-01/03/06/07/08). Este task file documenta re-verificación y cierre formal por pipeline. No se requiere edición si verify pasa.
- Ponytail: skipped abstracción helper compartido `validate_metric()` / `filter_metadata()` en crate common — 3× copia 8-10L es más corta que extraer crate `providers/common` (PROV-05). Add helper cuando PROV-05 se planifique.
- Verify full ejecutado (sin commit por instrucción `no commit`): cargo check x3 exit 0, grep 3+3 matches, casos manuales documentados. nextest/workspace no requerido (providers fuera workspace) — archivado como WAVE-1 batch.
- Source-driven: PyO3 0.29 `PyValueError::new_err` + `py.import("warnings").call_method1("warn", ...)` verificados contra docs.rs/pyo3 (pattern oficial, sin `unsafe`, sin `unwrap`).

## Context Save Point
- **Último step:** Step 3 ✅ COMPLETED — cargo check x3 exit 0 + casos manuales documentados
- **Próximo:** ninguno — task COMPLETED, handoff a PROV-08
- **Archivos tocados:** `.opencode/skills/campaign-executor/tasks/PROV-07.md` (nuevo task file); `providers/*/src/python.rs` ya corregidos en 2754c783 (no tocados en esta invocación)
- **Verify pendiente:** ninguno — contrato "Compila + caso de test manual documentado" ✅
