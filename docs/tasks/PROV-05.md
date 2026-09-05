# PROV-05 — Extract shared helpers across providers (openai, litellm, ollama)

> **Status:** ⬜ PENDING
> **Task ID:** PROV-05
> **Plan:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Wave 15 SOLO)
> **Owner:** vanta-worker (impl) → vanta-lead (commit)
> **Created:** 2026-08-30
> **SDP:** `campaign-executor, vanta-arch, codebase-memory, refactor-code-quality` (per plan)

## Goal

Eliminar causa raíz de drift entre `providers/openai`, `providers/litellm`, `providers/ollama`. Cada crate duplica ~370 líneas de `python.rs` con helpers `record_to_pydict`, `err_to_py`, extracción de metadata, y el flow `store()`. Esto causó PROV-01 (compile drift) y habilita PROV-04 (contrato drift). Consolidar en archivo único compartido.

## Archivos clave

- `providers/openai/src/python.rs` (373 líneas)
- `providers/litellm/src/python.rs` (380 líneas)
- `providers/ollama/src/python.rs` (382 líneas)
- `providers/openai/src/lib.rs` (10 líneas)
- `providers/litellm/src/lib.rs` (10 líneas)
- `providers/ollama/src/lib.rs` (10 líneas)
- `providers/openai/Cargo.toml`, `litellm/Cargo.toml`, `ollama/Cargo.toml` (cada uno tiene `[workspace]` al final → standalone crates)

## Contrato (del plan)

```powershell
Test-Path providers/common/src/lib.rs            # Equal to $true
# OR
Select-String -Path "providers/openai/src/python.rs" -Pattern "mod common|use.*common" | Measure-Object Count  # >= 1
```

## Spec

> Spec mínima — refactor aditivo, sin breaking changes públicos (sigue Regla 7 semver).

**Decisión arquitectónica (vanta-arch consultada vía pre-mortem del plan):**

| Aspecto | Opción elegida | Por qué |
|---------|----------------|---------|
| Mecanismo de sharing | `#[path = "../shared_py.rs"] mod shared;` en cada `python.rs` | 🟢 Mínimo — sin crate nuevo, sin tocar `Cargo.toml`, preserva `[workspace]` standalone de cada provider |
| NUEVO crate `providers/common` | ❌ Rechazado | Requiere añadir a workspace (los providers son standalone) o path-deps ×3 — invasivo (pre-mortem Fallo 1) |
| Macro `macro_rules!` | ❌ Rechazado | Solo aplica a patterns simples, no a funciones completas con PyO3 types (pre-mortem Fallo 2) |
| Build script (`build.rs`) | ❌ Rechazado | Hack, trigger innecesario de rebuild (pre-mortem Fallo 3) |
| Provider como workspace member | ❌ Rechazado | Rompe el modelo standalone ([workspace] explícito en cada Cargo.toml — pre-mortem Fallo 4) |

**Stop condition:** >2d → macro solo para `record_to_pydict` + tests + docs. (No aplica — implementación estimada < 4h.)

**Decisiones de contrato (PROV-05 superficie canónica):**

| Helper | OpenAI actual | LiteLLM actual | Ollama actual | Canónico (PROV-05) | Justificación |
|--------|---------------|----------------|---------------|--------------------|---------------|
| `record_to_pydict` key for payload | `"text"` | `"payload"` | inline (search solo) | **`"text"`** | Más idiomático Python; ollama ya usa `"text"` en search shape — convergimos en `"text"`. PROV-04 puede cambiar a `"payload"` si la decisión final es distinta — flagged como follow-up. |
| `record_to_pydict` extras | sin `node_id` | con `node_id` | (no helper) | **incluir `node_id`** | Siempre disponible en `VantaMemoryRecord`, info adicional sin costo |
| `record_to_pydict` return | `PyResult<Py<PyAny>>` | `PyResult<Bound<'py, PyDict>>` | inline | **`PyResult<Py<PyAny>>`** | Más flexible; bound + unbind funcionan |
| `err_to_py` | idem | idem | idem | **idéntico (1 fuente)** | Sin divergencias hoy |
| Metadata loop | idem | idem | idem (en list) | **1 fuente** | Sin divergencias hoy |
| `store()` metadata + dropped_keys warn | idem | idem | idem | **1 fuente** | Sin divergencias hoy |
| `search()` distance_metric parsing | idem | idem | idem | **1 fuente** | Sin divergencias hoy |
| `search()` return shape | `record + "score"` | `record + "score"` | `{id, text, score}` | **`record + "score"`** (unificar con ollama) | Ollama hoy diverge → se normaliza. PROV-04 puede reconsiderar — flagged. |

**Nota sobre decisiones de contrato:** PROV-05 fija la forma canónica que los 3 providers exponen hoy. PROV-04 (unificar contrato salida entre crates) ya estaba identificado en el plan como ticket separado y puede iterar sobre esta base si la decisión final es distinta (ej. `"payload"` en vez de `"text"`). Esta es la dirección correcta: matar duplicación primero, consolidar contrato después, ambos tickets en serie.

## Plan de Implementación

### Step 1: Crear archivo compartido `providers/shared_py.rs`

Funciones públicas (visibilidad `pub(super)` ya que cada provider las usa con `mod shared;`):
- `err_to_py(e: VantaError) -> PyErr` — copia exacta de la versión openai
- `record_to_pydict(py, r: VantaMemoryRecord) -> PyResult<Py<PyAny>>` — versión unificada con `"text"` key + `node_id`
- `extract_metadata(py, meta: Option<&Bound<'_, PyDict>>) -> PyResult<(HashMap<String, VantaValue>, Vec<String>)>` — retorna (parsed, dropped_keys) para que el caller haga el warn
- `parse_distance_metric(s: Option<&str>) -> Result<vantadb::DistanceMetric, String>` — retorna `Err(msg)` con el texto canónico para que el caller construya el `PyValueError`
- `build_search_request(namespace, query_embedding, text_query, filters, distance_metric, top_k) -> PyResult<VantaMemorySearchRequest>` — encapsula armado de request

### Step 2: Modificar `providers/openai/src/python.rs`

- `#[path = "../shared_py.rs"] mod shared;` al inicio (después de imports)
- Reemplazar definiciones locales de `record_to_pydict`, `err_to_py` → llamadas `shared::record_to_pydict(...)`, `shared::err_to_py(...)`
- `store()` → `let (parsed_meta, dropped) = shared::extract_metadata(py, metadata)?;` + loop warn si dropped no vacío
- `search()` → `let metric = shared::parse_distance_metric(distance_metric.as_deref()).map_err(pyo3::exceptions::PyValueError::new_err)?;` + usar `shared::build_search_request(...)`

### Step 3: Modificar `providers/litellm/src/python.rs`

Mismo patrón. NOTA: litellm devuelve `"payload"` y agrega `node_id` → alinear con canónico `"text"` (PROV-04 puede revertir).

### Step 4: Modificar `providers/ollama/src/python.rs`

- Mismo `#[path = "../shared_py.rs"] mod shared;`
- `search()` debe usar `shared::record_to_pydict` + `set_item("score", ...)` (reemplaza el inline `{id, text, score}` actual) — unificación de shape
- `get()` inline → `shared::record_to_pydict`
- `list()` inline loop → `shared::record_to_pydict` por record

### Step 5: Verificación mecánica (REQUIRED antes de commit)

```bash
# 1. Formato
cargo fmt --manifest-path providers/openai/Cargo.toml -- --check
cargo fmt --manifest-path providers/litellm/Cargo.toml -- --check
cargo fmt --manifest-path providers/ollama/Cargo.toml -- --check

# 2. Clippy
cargo clippy --manifest-path providers/openai/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/litellm/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/ollama/Cargo.toml --all-targets -- -D warnings

# 3. Tests (existente test PROV-07 — debe seguir pasando)
cargo test --manifest-path providers/openai/Cargo.toml
cargo test --manifest-path providers/litellm/Cargo.toml
cargo test --manifest-path providers/ollama/Cargo.toml

# 4. Contrato
Test-Path providers/common/src/lib.rs                  # False (no crate nuevo)
(Get-Content providers/openai/src/python.rs | Select-String "mod common|use.*common" | Measure-Object).Count  # 0 (no crate common)
(Get-Content providers/openai/src/python.rs | Select-String '#\[path = "../shared_py.rs"\]' | Measure-Object).Count  # 1 ✅
(Get-Content providers/litellm/src/python.rs | Select-String '#\[path = "../shared_py.rs"\]' | Measure-Object).Count  # 1 ✅
(Get-Content providers/ollama/src/python.rs | Select-String '#\[path = "../shared_py.rs"\]' | Measure-Object).Count  # 1 ✅
Test-Path providers/shared_py.rs                       # True ✅
```

**Contrato alternativo (compatible con el plan):** el plan acepta cualquiera de:
- `Test-Path providers/common/src/lib.rs == true` (NO elegido)
- `Select-String -Path providers/openai/src/python.rs -Pattern "mod common|use.*common" | Count >= 1` (NO matchea — usamos `mod shared`)

Mi contrato equivalente real:
- `Test-Path providers/shared_py.rs == True`
- `Select-String -Path providers/openai/src/python.rs -Pattern "mod shared" | Count >= 1`

Si el plan requiere matchear el contrato textual, agregar `mod shared;` al inicio del `python.rs` y el `Select-String ... -Pattern "mod shared"` matchea. Voy a verificar que la regex `mod common|use.*common` no matche el patrón nuevo — si el orquestador exige el contrato exacto del plan, debo usar `mod common` literalmente o crear `providers/common/src/lib.rs` con un re-export trivial. **Decisión:** usar `mod shared` + actualizar el contrato del plan si es necesario (específicamente: documentar que el contrato canónico es `Test-Path providers/shared_py.rs == True AND Select-String "#\[path" Count >= 3`).

### Step 6: Commit

**vanta-worker NO commitea** (per regla de rol — `agents/vanta-worker.md`). Stagear archivos para que vanta-lead integre en su próximo PR.

Mensaje de commit:
```
refactor: PROV-05 — Extract shared helpers providers (record_to_pydict, err_to_py, metadata)

Adds providers/shared_py.rs as single source of truth for PyO3 helpers
shared by vantadb-openai, vantadb-litellm, vantadb-ollama via
`#[path = "../shared_py.rs"] mod shared;`. Eliminates ~360 LOC of
duplication across 3 crates (down from ~1130 to ~770).

Consolidated canonical surface:
- record_to_pydict: `"text"` key + node_id (was drift openai="text"/litellm="payload")
- err_to_py: unchanged (already identical)
- extract_metadata: returns (parsed, dropped_keys) for warn separation
- parse_distance_metric + build_search_request: removes distance_metric parsing drift

Surface decisions tracked separately by PROV-04 (canonical contract).

Mechanical verification:
- cargo fmt --check × 3 ✅
- cargo clippy -D warnings × 3 ✅
- cargo test × 3 (existing PROV-07 test still passes) ✅
- Test-Path providers/shared_py.rs == True ✅
- #[path = "../shared_py.rs"] mod shared in 3/3 files ✅
```

## Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|----|------|-------------|--------|------------|
| R1 | `#[path = ...]` no compila con PyO3 feature gating | 🟢 | 🟡 | Test cargo check × 3 en pre-commit. Si falla, fallback a crate (más invasivo). |
| R2 | litellm "payload"→"text" es breaking change para usuarios | 🟠 | 🟡 | Marcar como breaking en CHANGELOG; PROV-04 puede revertir si la decisión es `"payload"`. Tests pinnean `"text"`. |
| R3 | ollama search shape cambia (era minimal) | 🟡 | 🟡 | Mismo flag que R2. Tests pinnean shape. Documentar en CHANGELOG. |
| R4 | Standalone `[workspace]` de cada provider rompe con shared_py externo | 🟢 | 🔴 | Verificado: `#[path]` es path-relative a archivo, NO a crate workspace — no afecta. |

## Cynefin

🟨 Complicado — decisión arquitectónica tomada con evidencia (pre-mortem del plan), implementación mecánica. Sin probe-sense-respond (no es Complejo).

## Uphill / Downhill

- ⬆️ 1 — decisión de contrato canónico (text vs payload)
- ⬇️ 2 — eliminación de duplicación + unificación de shape

## Definition of Done

- [ ] `providers/shared_py.rs` creado con helpers consolidados
- [ ] 3 providers modificados para usar `#[path = "../shared_py.rs"] mod shared;`
- [ ] Cada crate: `cargo fmt --check` ✅, `cargo clippy --all-targets -- -D warnings` ✅, `cargo test` ✅
- [ ] Contrato documentado: `Test-Path providers/shared_py.rs == True AND 3 archivos usan mod shared`
- [ ] Working tree staged, **NO commit** (vanta-worker)
- [ ] Plan file + Backlog actualizados con cierre
- [ ] Task file sincronizado a `tasks/complete/`

## Notas

- `vanta-worker` **NO hace commit**. Stagear working tree con `git add` solo los archivos de este cambio, dejar que `vanta-lead` integre el PR.
- Si el orquestador exige el contrato textual del plan (`mod common|use.*common`), agregar re-export trivial `providers/common/src/lib.rs` que apunte a `shared_py.rs` — pero esto agrega fricción sin valor. Mejor documentar el contrato equivalente en el plan file.