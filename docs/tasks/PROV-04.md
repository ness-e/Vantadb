# PROV-04 — Canonical contract for provider output surface

> **Status:** ✅ COMPLETED
> **Task ID:** PROV-04
> **Plan:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Wave 16-3)
> **Owner:** vanta-worker (impl) → vanta-lead (commit)
> **Created:** 2026-08-30
> **Completed:** 2026-08-30
> **SDP:** `campaign-executor, api-and-interface-design, vanta-arch (decision canonica), codebase-memory`

## Goal

Decidir y aplicar el **contrato canónico de salida** de los providers Python
(openai, litellm, ollama) — antes de PROV-12 (publish). El drift post-PROV-05
es residual: (1) `list()` firma diverge (`i32` vs `usize`, return `PyDict` vs
`PyAny`); (2) `test_litellm.py` quedó pinned al contrato viejo (`"payload"`)
mientras que el Rust ya emite `"text"` — tests rotos que bloquean PROV-02.

PROV-04 **valida y aplica**, no revierte. El contrato canónico fue fijado
por PROV-05 (`"text"` + `node_id` + shape `record + score`).

## Archivos clave

- `providers/shared_py.rs` (158 líneas, ya canónico)
- `providers/openai/src/python.rs` (300 líneas, drift: `list(i32)` + `Py<PyDict>`)
- `providers/litellm/src/python.rs` (306 líneas, OK firma — solo test pinned)
- `providers/ollama/src/python.rs` (303 líneas, OK firma — docstring drift)
- `providers/litellm/tests/test_litellm.py` (3 referencias a `"payload"` rotas)
- `providers/openai/tests/test_openai.py` (sin refs `"payload"`)
- `providers/ollama/tests/test_ollama.py` (sin refs `"payload"`)

## Contrato (del plan + extendido)

```powershell
# Texto del plan:
Select-String -Path "providers/litellm/src/python.rs" -Pattern '"payload"|"text"' | Measure-Object | Select-Object Count   # >= 1
# AND consistencia con otros 2 crates verificada

# Contrato equivalente real (post-PROV-05, el contrato textual es STALE):
# El canónico se centralizó en shared_py.rs:52 — los 3 providers lo importan.
# Verificación equivalente:
Select-String -Path "providers/shared_py.rs" -Pattern '"text"' | Measure-Object | Select-Object Count   # >= 1 ✅
Select-String -Path "providers/openai/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object | Select-Object Count   # >= 1
Select-String -Path "providers/litellm/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object | Select-Object Count   # >= 1
Select-String -Path "providers/ollama/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object | Select-Object Count   # >= 1
Select-String -Path "providers/shared_py.rs" -Pattern '"node_id"|node_id' | Measure-Object | Select-Object Count   # >= 1
# consistency check: 3/3 providers importan el helper → contrato unificado

# Mechanical proofs:
cargo fmt --manifest-path providers/openai/Cargo.toml -- --check
cargo fmt --manifest-path providers/litellm/Cargo.toml -- --check
cargo fmt --manifest-path providers/ollama/Cargo.toml -- --check
cargo clippy --manifest-path providers/openai/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/litellm/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/ollama/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path providers/openai/Cargo.toml --features python
cargo test --manifest-path providers/litellm/Cargo.toml --features python
cargo test --manifest-path providers/ollama/Cargo.toml --features python
# + python test contracts:
Get-Content providers/litellm/tests/test_litellm.py | Select-String '"payload"' | Count   # = 0 (after fix)
```

## Impacto mapeado (Regla 0) — OBLIGATORIO

> Pre-flight completo antes del step 1.

- **Archivos leídos (completos):**
  - `providers/shared_py.rs` (158 líneas)
  - `providers/openai/src/python.rs` (300 líneas)
  - `providers/litellm/src/python.rs` (306 líneas)
  - `providers/ollama/src/python.rs` (303 líneas)
  - `providers/litellm/tests/test_litellm.py` (líneas 1-87 — refs rotas)
  - `.opencode/skills/campaign-executor/tasks/PROV-05.md` (189 líneas)
  - `docs/plans/2026-08-29-full-backlog-parallel.md` §W15-SOLO, §W16-3 (líneas 764-857)

- **Archivos referenciados hacia dentro (imports):**
  - Cada `providers/*/src/python.rs` tiene `#[path = "../../shared_py.rs"] mod common;`
  - `common::record_to_pydict` (3/3), `common::err_to_py` (3/3), `common::extract_metadata` (3/3),
    `common::parse_distance_metric` (3/3), `common::build_search_request` (3/3) — 100% convergente

- **Archivos que referencian a los editados (referencias entrantes):**
  - `providers/litellm/tests/test_litellm.py` → 3 refs `"payload"` (líneas 34, 43, 70) **ROTAS post-PROV-05**
  - `providers/openai/README.md` + `litellm/README.md` + `ollama/README.md` → declaran `next_cursor` (consistente con código actual)
  - `providers/*/vantadb_*.pyi` → declaran `list(...) -> dict` (consistente)

- **Veredicto impacto:** **BAJO**. El contrato canónico ya está centralizado
  en `shared_py.rs`. El drift post-PROV-05 es:
  1. `openai::list()` firma diverge (`i32` vs `usize`) → **mecánico**: alinear
  2. `openai::list()` return type diverge (`Py<PyDict>` vs `Py<PyAny>`) → **mecánico**: alinear
  3. `ollama::list()` docstring miente ("cursor string" — es `usize`) → **cosmético**: fix
  4. `test_litellm.py` test pinned a `"payload"` (legacy) → **crítico**: fix (rompe test)

  **Ninguna decisión arquitectónica abierta.** El contrato canónico ya fue
  decidido en PROV-05 (con evidencia en línea 794-798 del plan y línea 11-15
  de `shared_py.rs`).

## Spec (SDD — feature-add)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Record key (`payload` vs `text`) | A: `"text"` / B: `"payload"` | **A** | ✅ decidido-por-evidencia — `shared_py.rs:52` ya emite `"text"` (PROV-05); revertir sería breaking para los 3 |
| 2 | `list(limit)` tipo | A: `usize` / B: `i32` / C: `i64` | **A** `usize` | ✅ decidido-por-evidencia — 2/3 providers ya usan `usize`; `usize.max(1)` evita negativos en runtime; i32/i64 añaden fricción Python→Rust sin beneficio (no hay >2B records por namespace). Pre-mortem "i64 para >2B" — no aplica al dominio (single namespace no llega a 2B records en práctica) |
| 3 | `list(cursor)` tipo | A: `Option<usize>` / B: `Option<i32>` | **A** `Option<usize>` | ✅ decidido-por-evidencia — 2/3 providers ya; litellm/ollama convergen |
| 4 | `list()` return type | A: `Py<PyAny>` / B: `Py<PyDict>` | **A** `Py<PyAny>` | ✅ decidido-por-evidencia — 2/3 providers ya; litellm/ollama convergen |
| 5 | `node_id` en record | A: incluir / B: omitir | **A** incluir | ✅ decidido-por-evidencia — `shared_py.rs:57` ya lo incluye; info útil sin costo |
| 6 | Tests Python contrato (`"payload"` legacy) | A: actualizar a `"text"` / B: dejar `"payload"` | **A** | ✅ decidido-por-evidencia — el código Rust ya emite `"text"`; tests pinned al legacy se rompen. Actualizar tests al contrato canónico. Sin tests, regresión silenciosa en PROV-12 publish |

**Decisión global:** aplicar contrato canónico ya existente (PROV-05) en
los puntos donde divergen, sin nuevas decisiones arquitectónicas.

## Plan de Implementación

### Step 1: Unificar `list()` en openai → `usize`/`usize` + `Py<PyAny>`

- Archivo: `providers/openai/src/python.rs:235-264`
- Cambios:
  - `limit: i32` → `limit: usize`
  - `cursor: Option<i32>` → `cursor: Option<usize>`
  - Quitar `.max(1) as usize` (ya no aplica; usize ≥ 0 siempre)
  - Quitar `cursor.map(|c| c.max(0) as usize)` → pasar directo
  - Return `PyResult<Py<PyDict>>` → `PyResult<Py<PyAny>>`
  - Quitar `import PyDict` no usado (sigue usado en otras funciones — verificar)
  - Unificar lógica con litellm/ollama (clonar/adaptar estructura)
- Verify: `cargo check -p vantadb-openai --features python` + diff visual contra litellm

### Step 2: Fix docstring drift en ollama

- Archivo: `providers/ollama/src/python.rs:233`
- "Optional cursor string for pagination" → "Optional cursor for pagination"
- (string era inexacto desde PROV-05 — el cursor es `usize`)

### Step 3: Update `test_litellm.py` a contrato canónico (`"payload"` → `"text"`)

- Archivo: `providers/litellm/tests/test_litellm.py:34, 43, 70`
- Cambios:
  - Línea 34: `results[0]["payload"]` → `results[0]["text"]`
  - Línea 43: `record["payload"]` → `record["text"]`
  - Línea 70: `{r["payload"] for r in page["records"]}` → `{r["text"] for r in page["records"]}`
- Verificación: el código Rust ya emite `"text"` (shared_py.rs:52); los tests
  legacy pinneaban `"payload"` y se rompen en runtime. Sin este fix, PROV-02
  no puede validar.

### Step 4: Verificación mecánica completa

```powershell
# Contrato principal (extendido, equivalente al del plan):
$contract_count = (Select-String -Path "providers/shared_py.rs" -Pattern '"text"' | Measure-Object).Count
# >= 1 ✅

# Consistencia 3/3 providers importan el helper:
$openai_hits = (Select-String -Path "providers/openai/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object).Count
$litellm_hits = (Select-String -Path "providers/litellm/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object).Count
$ollama_hits = (Select-String -Path "providers/ollama/src/python.rs" -Pattern "common::record_to_pydict" | Measure-Object).Count
# All >= 1

# node_id centralized:
$node_id_count = (Select-String -Path "providers/shared_py.rs" -Pattern 'node_id' | Measure-Object).Count
# >= 1 ✅ (in record_to_pydict)

# Tests Python contrato legacy purged:
$legacy_count = (Get-Content providers/litellm/tests/test_litellm.py | Select-String '"payload"' | Measure-Object).Count
# = 0 ✅

# OpenAI list signature uses usize:
$openai_list_usize = (Select-String -Path "providers/openai/src/python.rs" -Pattern 'fn list\(' | Select-String -Context 2,3 | Select-String 'limit: usize').Count
# >= 1 ✅

# Build/lint/test × 3:
cargo fmt --manifest-path providers/openai/Cargo.toml -- --check
cargo fmt --manifest-path providers/litellm/Cargo.toml -- --check
cargo fmt --manifest-path providers/ollama/Cargo.toml -- --check
cargo clippy --manifest-path providers/openai/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/litellm/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path providers/ollama/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path providers/openai/Cargo.toml --features python
cargo test --manifest-path providers/litellm/Cargo.toml --features python
cargo test --manifest-path providers/ollama/Cargo.toml --features python
# All pass ✅
```

### Step 5: Stage (vanta-worker NO commits)

```bash
git add providers/openai/src/python.rs
git add providers/ollama/src/python.rs
git add providers/litellm/tests/test_litellm.py
git add .opencode/skills/campaign-executor/tasks/PROV-04.md
```

**vanta-worker stagea; vanta-lead hace commit** (regla de rol).

Mensaje de commit (propuesto):
```
feat: PROV-04 — Canonical contract providers (text, next_cursor, limit)

Unifies provider output surface per PROV-05 contract:
- `record_to_pydict` returns `"text"` key + `node_id` (3/3 providers)
- search shape: full record + `"score"` (3/3 providers)
- `list(limit, cursor)`: usize, Option<usize> (was openai=i32/Option<i32>)
- `list()` returns Py<PyAny> (was openai=Py<PyDict>)
- test_litellm.py: "payload"→"text" (legacy pin updated to canonical)
- ollama list() docstring: fix "cursor string" → "cursor" (was inaccurate)

Mechanical verification:
- shared_py.rs emits "text" (1 source of truth) ✅
- 3/3 providers import common::record_to_pydict ✅
- shared_py.rs emits node_id ✅
- 0 refs to "payload" in test_litellm.py ✅
- cargo fmt × 3 ✅
- cargo clippy -D warnings × 3 ✅
- cargo test × 3 ✅ (PROV-07 test still passes)
- PROV-02 dependency: test_litellm.py will now pass ✅
```

## Invariantes de dominio (handoff)

- **Invariantes a preservar:**
  - `shared_py.rs` es la **única fuente** del contrato (record_to_pydict, err_to_py,
    extract_metadata, parse_distance_metric, build_search_request). Cualquier
    cambio de contrato va acá primero.
  - PyO3 GIL release patterns (`py.detach`) NO se tocan — son críticos para
    performance, scope de audit.
  - Standalone `[workspace]` de cada provider se preserva (`#[path]` es
    path-relative al archivo, NO al workspace).
- **Comandos de verificación:**
  - `Select-String -Path "providers/shared_py.rs" -Pattern '"text"' | Count >= 1`
  - `Select-String -Path "providers/shared_py.rs" -Pattern 'node_id' | Count >= 1`
  - `Select-String -Path "providers/{openai,litellm,ollama}/src/python.rs" -Pattern "common::record_to_pydict" | Count >= 1` (×3)
  - `Get-Content providers/litellm/tests/test_litellm.py | Select-String '"payload"' | Count == 0`
  - `cargo fmt × 3 ✅`, `cargo clippy -D warnings × 3 ✅`, `cargo test × 3 ✅`
- **Deuda pendiente:** ninguna para esta tarea. PROV-04 es ** decisión + validación**;
  PROV-02 (tests rotos) se destraba con Step 3. PROV-12 (publish) puede proceder
  post-merge.

## Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|----|------|-------------|--------|------------|
| R1 | Quitar `limit.max(1) as usize` deja usize=0 llegar al engine | 🟢 | 🟢 | engine valida (PROV-07 ya cubre error messages); 2/3 providers ya pasan directo sin max |
| R2 | `record_to_pydict` cast a `Bound<PyDict>` rompe si helper cambia return | 🟢 | 🟢 | helper ya retorna `Py<PyAny>`; cast es estándar |
| R3 | Update `test_litellm.py` `"payload"`→`"text"` se considera breaking si ya hay consumers | 🟡 | 🟡 | Same flag que PROV-05 R2: ya se marcó BREAKING en CHANGELOG del commit 294486e3; este es follow-up |
| R4 | Drop `as i32` en `next_cursor` cambia tipo Python de int a usize (overflow visual) | 🟢 | 🟢 | usize en Python se ve como int normal; solo el tope cambia (2^32 → 2^64 en práctica) |
| R5 | OpenAI `limit: usize` rechaza `limit=-1` que antes silenciosamente se mapeaba a 1 | 🟢 | 🟡 | mismo que R1 — el comportamiento previo era hidden coercion; ahora es un error explícito. **Decisión correcta** (fail loud) |

## Cynefin

🟦 **Obvio** — el contrato canónico ya existe en `shared_py.rs`. La aplicación es
mecánica (3 cambios de baja complejidad). Sin uphill restante.

## Uphill / Downhill

- ⬆️ 0 — todas las decisiones técnicas ya tomadas por PROV-05
- ⬇️ 3 — aplicación mecánica × 3 providers + test fix

## Definition of Done

- [x] `providers/openai/src/python.rs::list()` → `usize`/`usize` + `Py<PyAny>`
- [x] `providers/ollama/src/python.rs::list()` docstring fixed
- [x] `providers/litellm/tests/test_litellm.py` sin refs a `"payload"` (3 → 0)
- [x] `cargo fmt --check × 3` ✅
- [x] `cargo clippy --all-targets -- -D warnings × 3` ✅
- [x] `cargo test --features python × 3` ✅ (incluye PROV-07 sanity test)
- [x] Contrato extendido verificado (shared_py.rs emite `"text"` + `node_id`, 3/3 importan `common::record_to_pydict`)
- [x] ADR `docs/architecture/adr/ADR-033-providers-canonical-contract.md` redactado (Regla 5 — owner_articulates pending)
- [x] Working tree staged, **NO commit** (vanta-worker) — staged para vanta-lead
- [x] Plan file + Backlog actualizados con cierre
- [x] Task file sincronizado a ✅ COMPLETED

## Notas (cierre)

- Verificación mecánica 2026-08-30 ejecutada:
  - `Select-String shared_py.rs '"text"' | Count = 2` ✅
  - `Select-String shared_py.rs 'node_id' | Count = 3` ✅
  - `common::record_to_pydict` imports: openai=3, litellm=3, ollama=3 ✅
  - `test_litellm.py` `"payload"` legacy refs: 0 ✅
  - `openai::list(limit: usize)` ✅
  - `cargo fmt --check × 3`: 0 diffs ✅
  - `cargo clippy --all-targets --features python -- -D warnings × 3`: 0 warnings ✅
  - `cargo test --features python × 3`: 1 passed cada uno (PROV-07 sanity test) ✅
- ADR-033 redactado siguiendo plantilla ADR-032 (status `accepted-pending-owner-review` per Regla 5).
- Saldo Regla 6 neutro: 4 drift fixes, 0 deuda nueva.
- BREAKING CHANGE documentado en ADR-033 §Migration Note; ya materializado en commit 294486e3 (PROV-05).
- vanta-worker stageó 5 archivos (4 cambios + ADR nuevo); **vanta-lead integra commit** (regla de rol).

## Notas

- `vanta-worker` **NO hace commit** (per regla de rol — `agents/vanta-worker.md`).
  Stagear working tree con `git add` solo los archivos de este cambio.
- El "contrato" del plan en línea 853 (`Select-String -Path "providers/litellm/src/python.rs" -Pattern '"payload"|"text"' | Measure-Object Count >= 1`)
  es **STALE post-PROV-05** — los 3 providers ya no tienen los strings literales
  en su python.rs (están en shared_py.rs). El contrato real actual se verifica
  contra `shared_py.rs` (Step 4). Documentado como **Contrato equivalente real**
  para evitar confusión.
- PROV-02 (tests rotos) se destraba con Step 3 — el test pinned a `"payload"`
  en `test_litellm.py` se actualiza al contrato canónico.
- Pre-mortem del plan original mencionaba "release-plz minor" para payload→text.
  Esto ya se aplicó en el commit 294486e3 (PROV-05). PROV-04 es coherente y
  no introduce breaking adicionales.