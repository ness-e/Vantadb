---
title: "ADR-033: Contrato canónico de salida unificado para providers Python (openai, litellm, ollama)"
type: adr
status: accepted-pending-owner-review
tags: [vantadb, architecture, adr, providers, python, contract, pyi, pyo3, prov-04]
created: 2026-08-30
last_reviewed: 2026-08-30
related: [ADR-005-error-handling, ADR-016-adapter-tiers, DRV-018-shared-py-canonicalization, PROV-05-shared-helpers, PROV-12-publish]
owner_articulates: pending
---

> **⚠️ DRAFT — requiere articulación del owner (Regla 5, AGENTS.md)**
>
> El cuerpo de este ADR (Contexto, Decisión, Consecuencias, Alternativas,
> Riesgos) fue redactado por IA durante la implementación PROV-04 para
> registrar evidencia técnica. **El trade-off central — `usize` vs `i32` vs
> `i64` para `limit`/`cursor`/`next_cursor` y la decisión de incluir
> `node_id` en todos los records (era litellm-only) — debe ser articulado
> por el owner humano en sus propias palabras** para que el ADR cumpla su
> función de memoria de decisión. La IA aporta los datos y la evidencia
> mecánica; el humano decide si acepta el trade-off tal cual, lo ajusta, o
> lo reemplaza por una alternativa.
>
> Hasta que el owner articule: status `accepted-pending-owner-review`,
> `owner_articulates: pending`. Las refs internas y el código son válidos
> independientemente (PROV-05 ya emitió el contrato en `shared_py.rs` y los
> 3 providers lo importan).

# ADR-033: Contrato canónico de salida unificado para providers Python

## Context

Tres providers Python (`vantadb-openai`, `vantadb-litellm`, `vantadb-ollama`)
son el surface público para consumidores que integran VantaDB como memoria
persistente + retrieval en pipelines de AI. Antes de `PROV-05` los 3 crates
eran **standalones** (cada uno con su `Cargo.toml` y `pymodule` propio) y
duplicaban ~370 LOC de helpers PyO3 (`record_to_pydict`, `err_to_py`,
`extract_metadata`, `parse_distance_metric`, `build_search_request`).

Esa duplicación produjo **drift de contrato** que PROV-04 cierra:

| Drift | openai | litellm | ollama | Detectado en |
|-------|--------|---------|--------|--------------|
| **Record key (`payload` vs `text`)** | `"text"` | `"payload"` ❌ | `"text"` | grep `record_to_pydict` post-PyO3 wiring |
| **`node_id` en record** | ❌ omitido | ✅ incluido (legacy) | ❌ omitido | grep `record_to_pydict` |
| **`list(limit)` firma** | `i32` ❌ | `usize` | `usize` | grep `fn list` |
| **`list(cursor)` firma** | `Option<i32>` ❌ | `Option<usize>` | `Option<usize>` | grep `fn list` |
| **`list()` return type** | `Py<PyDict>` ❌ | `Py<PyAny>` | `Py<PyAny>` | grep return type |
| **`next_cursor` cast en JSON** | `as i32` ❌ | none | none | grep `next_cursor` |
| **`limit.max(1) as usize`** | sí (hidden coercion) ❌ | no | no | grep `max(1)` |
| **search shape** | record + score ✅ | record + score ✅ | `{id, text, score}` ❌ | grep `search` return |
| **`test_litellm.py` pinned to `"payload"`** | n/a | **3 refs legacy** ❌ | n/a | pytest runtime |

La consecuencia era triple:

1. **Tests rotos**: `providers/litellm/tests/test_litellm.py` assertaba
   `record["payload"]` mientras el código Rust ya emitía `"text"` post-PROV-05
   → PROV-02 (regression test suite) bloqueado por 3 asserts en runtime.
2. **API pública divergente**: el consumidor que importe `vantadb_litellm`
   y `vantadb_openai` debe manejar dos shapes distintos para la misma
   operación. **Hyrum's Law** (skill `api-and-interface-design`): todo
   observable behavior se convierte en contrato de facto — drift here
   significa que YA hay consumidores que dependen de `"payload"` solo en
   litellm.
3. **Hidden coercions**: `limit.max(1) as usize` y `next_cursor.map(|c| c as i32)`
   silenciaban input inválido (negativos) — fail-silent en lugar de fail-loud.

El contrato canónico ya fue **fijado por PROV-05** (commit `294486e3`) al
centralizar los helpers en `providers/shared_py.rs:52,57`. El doc comment
del módulo declara explícitamente:

> Surface decisions (PROV-05 canonical):
> - `record_to_pydict` payload key: `"text"` (was drift openai="text"/litellm="payload")
> - `record_to_pydict` extras: includes `node_id` (was litellm-only)
> - search shape: full record + `"score"` (was ollama-minimal {id, text, score})
>
> PROV-04 (canonical contract unification) may revise these — tracked separately.

PROV-04 **aplica** el contrato en los puntos donde divergen
(`openai::list()`, `ollama::list()` docstring, `test_litellm.py` legacy
pin). No revierte PROV-05.

## Invariantes

1. **Una única fuente de verdad del contrato**: `providers/shared_py.rs`
   expone los 5 helpers como `pub(super) fn` y los 3 providers los importan
   vía `#[path = "../../shared_py.rs"] mod common;`. Cualquier cambio de
   contrato va primero a `shared_py.rs`.
2. **Backwards compat dentro de lo posible**: si un usuario ya consumía
   `result["payload"]` desde `vantadb-litellm` antes de PROV-05, ese
   path se rompe con PROV-04. Marcar como **BREAKING CHANGE** en
   `CHANGELOG.md` (release-plz minor bump) — migration note: `"payload"`
   → `"text"`. La decisión ya está materializada en el commit 294486e3;
   PROV-04 la hace consistente en los 3 crates.
3. **`next_cursor` Python type**: `usize` se serializa como `int` Python
   estándar (`PyO3` lo convierte vía `IntoPy<PyInt>`). El límite superior
   pasa de `i32::MAX ≈ 2.1 × 10^9` a `usize::MAX ≈ 1.8 × 10^19` — sin
   impacto práctico (un namespace no llega a 2B records).
4. **`limit = 0`**: ahora retorna lista vacía explícita (`limit` es
   `usize`, sin `.max(1)` que silenciara `0`). El core ya validaba
   `limit == 0 → return empty` (api.rs:613-621) — fail loud consistente
   con el resto del SDK.
5. **`limit < 0` (Python)**: ahora se rechaza con `PyValueError` (el
   binding PyO3 falla la conversión `i64 → usize`). Antes openai hacía
   `.max(1)` y silenciaba el negativo → resultado incoherente (`limit=1`
   cuando el caller pidió `-1`). **Decisión correcta**: fail loud.

## Decision

PROV-04 adopta el contrato canónico ya existente en `shared_py.rs` y lo
aplica en los 3 puntos donde divergen. Las 6 decisiones del contrato:

### D1 — Record key: `"text"` (revalidado)

- **Elegido**: `"text"` (canónico, definido en `shared_py.rs:52`)
- **Rechazado**: `"payload"` (legacy litellm pre-PROV-05)
- **Por qué**: `record_to_pydict` es la única vía de serializar
  `VantaMemoryRecord` → Python. Centralizar el nombre de la key en
  `shared_py.rs` elimina drift entre los 3 providers. Revertir a
  `"payload"` reintroduce el problema.

### D2 — `list(limit)`: `usize`

- **Elegido**: `usize`
- **Rechazado**: `i32` (legacy openai), `i64`
- **Por qué**:
  - `i32` rechaza `limit > 2^31` con error; `usize` permite hasta 2^64.
    En VantaDB un namespace no llega a 2B records en práctica (single-host,
    HNSW + memory pressure), pero el costo de aceptar `usize` es cero
    (PyO3 convierte a `int` Python nativo, sin overhead observable).
  - `i64` añade fricción Python↔Rust sin beneficio (limita a 2^63 vs 2^64).
  - `usize` ya es la convención del SDK Rust core (`VantaMemoryListOptions.limit`).
- **Pre-mortem "i64 para >2B"**: rechazado. `usize` soporta 2^64 records
  por namespace (~10^19). Single-namespace no llega a 2B records en
  práctica; `i64` solo añade fricción.

### D3 — `list(cursor)`: `Option<usize>`

- **Elegido**: `Option<usize>`
- **Rechazado**: `Option<i32>` (legacy openai)
- **Por qué**: misma lógica que D2. litellm/ollama ya convergen.

### D4 — `list()` return type: `Py<PyAny>`

- **Elegido**: `Py<PyAny>`
- **Rechazado**: `Py<PyDict>` (legacy openai)
- **Por qué**: el dict interno es `PyDict`, pero devolver `Py<PyAny>`
  permite flexibilidad (futuro: subclassing, wrapping sin copiar). El
  type erasure en el binding PyO3 es 0-cost. litellm/ollama ya convergen.

### D5 — `node_id` en record: **incluido**

- **Elegido**: incluir `node_id` (`u128`, serializado como `int` Python)
- **Rechazado**: omitir (legacy openai/ollama)
- **Por qué**:
  - litellm ya lo incluía (legacy), openai/ollama no
  - `node_id` permite al consumidor correlacionar records con la API
    core (cuando expongamos `get_by_node_id`) — info útil sin costo
  - Costo: 8 bytes en el dict (`PyLong` para `u128`) — despreciable
- **Compatibilidad**: agregado aditivamente — ningún consumer existente
  se rompe por agregar un key nuevo al dict.

### D6 — Test contract `"payload"` → `"text"`

- **Elegido**: actualizar `providers/litellm/tests/test_litellm.py` (3 refs)
- **Rechazado**: dejar `"payload"` legacy (rompe runtime post-PROV-05)
- **Por qué**: el código Rust ya emite `"text"`; tests pinned al legacy
  fallan en `pytest`. Sin este fix PROV-02 (regression suite) queda
  bloqueado.
- **Compatibilidad**: marcado como **BREAKING CHANGE** en CHANGELOG con
  migration note 1-liner: `r["payload"]` → `r["text"]`.

## Alternatives Considered

### A1 — Mantener `"payload"` como legacy alias

Re-emitir `"payload"` y `"text"` (dual keys) para backward compat.

- **Pro**: no rompe consumers litellm existentes
- **Contra**: viola **One-Version Rule** (skill `api-and-interface-design`):
  "diamond dependency problems arise when different consumers need
  different versions of the same thing". Y mantiene el drift latente que
  PROV-05 ya pagó para eliminar.
- **Rechazado**.

### A2 — Revertir PROV-05: `"text"` → `"payload"` global

Unificar al contrato viejo (litellm legacy).

- **Pro**: openai/ollama ya eran `"text"`; el cambio sería openai/ollama
  → `"payload"` (más invasivo pero coherente con PROV-04 como reverter).
- **Contra**: viola Hyrum's Law — los consumers openai/ollama que ya
  consuman `"text"` (shipped en versiones previas) se rompen. La doc
  publicitada en PROV-08 (READMEs) usa `"text"`.
- **Rechazado**.

### A3 — `limit: i64` para headroom >2B

Pre-mortem mencionaba `i64` para namespaces >2B records.

- **Pro**: explícito el tipo "entero grande"
- **Contra**: fricción Python↔Rust innecesaria (`usize` ya es 8 bytes en
  64-bit, sin diferencia observable). `i64` añade un type cast extra.
- **Rechazado**.

### A4 — Remover `node_id` para reducir dict size

Pre-mortem mencionaba "node_id solo en litellm → si lo removemos,
litellm consumers pierden info".

- **Pro**: dict más pequeño (8 bytes menos)
- **Contra**: rompe consumers litellm existentes. Hyrum's Law.
- **Rechazado**.

## Consequences

### Positivas

1. **API pública consistente**: 3 providers, mismo surface, mismo test
   contract. Consumers pueden portar código entre providers sin cambiar
   `record["text"]` o `result["records"]`.
2. **Type-safe Python**: `usize` se serializa como `int` Python nativo;
   `i32`/`i64` tienen los mismos bits pero el binding PyO3 rechaza
   negativos en `usize` → fail loud (mejor que `.max(1)` hidden coercion).
3. **Tests verdes**: `test_litellm.py` ya no pinned al contrato legacy;
   PROV-02 (regression suite) puede validar.
4. **PROV-12 (publish) destrabado**: el contrato canónico es ahora
   estable; `cargo publish --tag` puede proceder post-merge.
5. **Saldo Regla 6 (deuda) neutro**: 4 fixes de drift (openai firma +
   ollama docstring + test legacy pin + 2 cast removidos) sin abstracción
   nueva.

### Negativas / Riesgos

1. **BREAKING CHANGE para consumers litellm pre-PROV-05**: `r["payload"]`
   ahora es `KeyError` (no emite más esa key). Mitigación:
   - CHANGELOG release-plz minor bump (release-plz autodetecta `feat:` +
     `BREAKING CHANGE:` → minor)
   - migration note 1-liner: `r["payload"]` → `r["text"]`
   - El cambio ya está materializado en commit 294486e3 (PROV-05) —
     PROV-04 es **coherente** (sin nuevos breakings adicionales).
2. **Drift latente si nuevos providers se agregan sin usar `shared_py`**:
   si en el futuro alguien agrega `vantadb-anthropic` o similar sin
   importar `#[path = "../../shared_py.rs"] mod common;`, el contrato
   vuelve a divergir. Mitigación: ADR-016 (adapter tiers) + review
   checklist pre-merge requiere `import common::record_to_pydict`.
3. **Hidden coercion removida (`limit.max(1)`)**: callers openai que
   pasaban `limit=-1` ahora reciben `PyValueError` explícito en lugar
   de un resultado silencioso. **Decisión correcta** (fail loud, mismo
   patrón que el resto del SDK).

### Deuda técnica (Regla 6)

- **Saldo neto: neutral**.
- **Quita**: 2 cast `as i32`/`as usize` (deuda) + 2 hidden coercions
  `.max(1)`/`.max(0)` (deuda) en `openai::list` + 1 docstring inexacto
  (deuda) en `ollama::list` + 3 asserts legacy pinned (deuda) en
  `test_litellm.py`.
- **Agrega**: 0 deuda nueva. Refactor aditivo sin abstracción nueva.

## Migration Note (CHANGELOG)

```markdown
### BREAKING CHANGES — PROV-04 (Canonical contract unification)

- All 3 Python providers (`vantadb-openai`, `vantadb-litellm`,
  `vantadb-ollama`) now return records with the `"text"` key
  (previously `vantadb-litellm` used `"payload"`).
- Migration: `record["payload"]` → `record["text"]`.
- `list(limit, cursor)` signature unified to `(int, int | None)`
  (was `(int, int | None)` on litellm/ollama, `(int, int | None)`
  accepting negative on openai). `limit < 0` now raises `ValueError`
  (previously openai silently clamped to 1).
- All records now include `node_id` field (`int`, was openai/ollama missing).
- BREAKING applies only to consumers of `vantadb-litellm` who pinned
  to the legacy `"payload"` key. Already released versions of
  `vantadb-openai` and `vantadb-ollama` are compatible.
```

## References

- `providers/shared_py.rs:48-65` — `record_to_pydict` (canónico)
- `providers/shared_py.rs:11-16` — doc comment declarando contrato PROV-05
- `providers/openai/src/python.rs:233-270` — `list()` unificado
- `providers/ollama/src/python.rs:233` — docstring fix
- `providers/litellm/tests/test_litellm.py:34,43,70` — `"payload"` → `"text"`
- `commit 294486e3` — PROV-05 (shared_py.rs canonicalization)
- `docs/plans/2026-08-29-full-backlog-parallel.md` §W16-3 (PROV-04)
- `docs/research/2026-08-25-research-providers-quickwins.md` (origen del hallazgo)
- `docs/architecture/adr/ADR-016-adapter-tiers.md` (política adapters)
- `docs/architecture/adr/ADR-005-error-handling.md` (taxonomía errores)
- Skill `api-and-interface-design` (Hyrum's Law, One-Version Rule)