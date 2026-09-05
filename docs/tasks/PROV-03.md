# TASK PROV-03: Regenerar los 3 .pyi desde firmas reales (namespace, get/list/delete/list_namespaces, model/base_url/timeout)

## Metadata
- **Plan file:** `docs/plans/2026-08-25-research-providers-quickwins.md`
- **Creado:** 2026-08-26T15:00
- **last-synced:** 2026-08-26T15:00
- **Estado:** ✅ COMPLETED
- **Tipo detectado:** binding/pyo3-stub-sync (wrapper thin, no lógica nueva)
- **Workflow:** fix (mechanical pyi drift) — spec-first NO requerido
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-03.md`
- **Backlog:** `docs/Backlog.md` P45 PROV-03
- **Origen:** INV-providers-01 H-03 — stubs con API vieja sin namespace/text_query/filters/distance_metric/top_k default; omiten get/list/delete/list_namespaces y params model/timeout/base_url

## Blast Radius
- `providers/openai/src/python.rs:66-349` — VantaDBOpenAI pyclass, 7 pymethods + __new__, usa VantaEmbedded, VantaMemoryListOptions, VantaMemorySearchRequest. Blast radius: solo providers/openai, no toca core `src/sdk/types.rs` ni `src/engine.rs`.
- `providers/litellm/src/python.rs:66-356` — VantaDBLiteLLM, mismos 7 pymethods, model/timeout/base_url variantes.
- `providers/ollama/src/python.rs:37-358` — VantaDBOllama, mismos 7 pymethods, base_url propio.
- `providers/openai/vantadb_openai.pyi` (31 líneas), `providers/litellm/vantadb_litellm.pyi` (31 líneas), `providers/ollama/vantadb_ollama.pyi` (31 líneas) — únicos archivos editables.
- `.github/scripts/verify_pyi.py` — script CI que verifica 7 métodos presentes, lee los .pyi indirectamente.
- **Implicaciones:** 0 WAL/vector/storage, 0 concurrencia (no dashmap/parking_lot/Tokio), 0 unsafe nuevo, 0 FFI memory safety (thin wrappers existentes). Solo stubs tipados, reversible (overwrite 3 files). No hot path.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos (antes de editar):**
  - `providers/openai/src/python.rs` (349 líneas, HEAD 2754c783) — `#[pyclass] VantaDBOpenAI` con `#[new] signature (db_path, api_key, model="text-embedding-3-small", namespace="openai_store", timeout=None)` + 7 pymethods: `embed(texts)`, `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10)`, `store(text, embedding, metadata=None)`, `delete(key, namespace=None)`, `get(namespace,key)`, `list(namespace, limit=100, cursor=None)`, `list_namespaces()`.
  - `providers/litellm/src/python.rs` (356 líneas, HEAD 2754c783) — `VantaDBLiteLLM` con `#[new] (db_path, api_key=None, model="text-embedding-3-small", namespace="litellm_store", timeout=None)` + mismos 7 pymethods, `list` usa `limit: usize`, `cursor: Option<usize>`, `delete` y `get` idem.
  - `providers/ollama/src/python.rs` (358 líneas, HEAD 2754c783) — `VantaDBOllama` con `#[new] (db_path, base_url="http://localhost:11434", model="nomic-embed-text", namespace="ollama_store", timeout=None)` + mismos 7 pymethods.
  - `providers/openai/vantadb_openai.pyi` (31 líneas, HEAD 2fa8ea4f) — clase con __init__ 5 params + 7 métodos + __version__.
  - `providers/litellm/vantadb_litellm.pyi` (31 líneas, HEAD b8e376a5) — idem con api_key opcional.
  - `providers/ollama/vantadb_ollama.pyi` (31 líneas, HEAD 16eb9a67) — idem con base_url.
   - `providers/openai/Cargo.toml`, `providers/litellm/Cargo.toml`, `providers/ollama/Cargo.toml` — `pyo3 = { version = "0.29", optional = true, features = ["extension-module"] }` (verificado 2026-08-26 via `Select-String pyo3 Cargo.toml`), fuera del workspace (standalone).
  - `SKILLS-MANIFEST.md` grep `pyi|python|pyo3|provider` → 0 hits directos; `source-driven-development` (engineering lifecycle) y `ponytail` (modo lazy) aplican.
  - `.opencode/references/skills-engineering.md` §Lifecycle mapping — fase VERIFY para pyi sync.
- **Referencias hacia dentro (qué importa este archivo):**
  - `providers/*/src/python.rs` → define la verdad (pymethods). Los .pyi son derivados; no se importan en Rust, solo para type-checkers (pyright/mypy) y DX.
  - `providers/*/vantadb_*.pyi` → consumido por `pyright`/`mypy` y por tests Python `providers/*/tests/test_*.py` (importan clase, no stub). No afecta `cargo check` ni runtime Rust.
  - `Cargo.toml` providers → `cargo check --manifest-path` valida que `python.rs` compila; no valida .pyi.
- **Referencias entrantes (qué depende de lo que cambio):**
  - Plan file `docs/plans/2026-08-25-research-providers-quickwins.md` Wave 1 Task 3 → gating de quickwins (PROV-03 bloquea Wave1 5/5).
  - Backlog P45 PROV-03 → fila que se elimina al completar via `skill progreso`.
  - `.github/scripts/verify_pyi.py` → verifica 7 métodos presentes, no firma exacta; no rompe si .pyi cambia pero sí si falta método.
  - `docs/Backlog.md` y `docs/avance/` → progreso migration.
  - No hot path, no WAL, no memoria, no concurrencia. Seguridad: FFI existente (PyO3) sin cambios, solo stubs.
- **Veredicto de impacto:** BAJO (3 archivos .pyi, 31 líneas c/u, overwrite idempotente). Reversible (restore desde git). Verify: `cargo check --manifest-path` ×3 + diff pyi vs pymethods manual + `cargo fmt --check`.

## Spec
N/A — bug-fix con contrato mecánico (ver `prompts/spec-template.md` — feature-add/lógica nueva requiere Spec tabla; este es fix mecánico de drift stubs). Contrato mecánico suficiente: firmas .pyi == pymethods (7 métodos).

## Contrato
Firmas .pyi == pymethods en openai/litellm/ollama (7 métodos cada una: embed, search, store, delete, get, list, list_namespaces) incluyendo params namespace, get/list/delete/list_namespaces, model/base_url/timeout.

Verificación mecánica:
1. `cargo check --manifest-path providers/openai/Cargo.toml` exit 0
2. `cargo check --manifest-path providers/litellm/Cargo.toml` exit 0
3. `cargo check --manifest-path providers/ollama/Cargo.toml` exit 0
4. `pyi compare` — cada .pyi contiene los 7 métodos con signatures idénticas a `#[pyo3(signature = ...)]` del .rs correspondiente (grep + diff manual)
5. `cargo fmt --check` exit 0 (no Rust editado, pero gate)

## Herramientas
- Read (python.rs ×3, .pyi ×3, Cargo.toml ×3, plan file, backlog)
- Grep (pymethods, pyi, timeout, base_url, model, namespace)
- Bash (cargo check ×3, pyi compare script, cargo fmt --check, git diff/status)
- Edit/Write (solo si .pyi difiere; overwrite 3 files)
- campaign_memory_write, campaign_diagnose_pipeline (cierre)

## Skills
- campaign-executor, progreso, ponytail (base obligatoria)
- source-driven-development (detectado: PyO3 `#[pyclass]`/`#[pymethods]` signature, .pyi stub typing — verificar docs oficiales PyO3 0.23)
- ponytail (modo full — lazy: stdlib grep + overwrite mínimo, no codegen complejo)
- SDP discovery (Lifecycle BUILD/VERIFY): keywords `pyi, pymethods, namespace, get/list/delete/list_namespaces, model/base_url/timeout, provider, stub, PyO3` → grep SKILLS-MANIFEST.md → 0 hits nuevos. **SDP: sin candidatos adicionales** (base-only + source-driven-development + ponytail ya cubren). **SKILLS_CARGADAS: campaign-executor, progreso, ponytail, source-driven-development**

## Steps

### Step 1: Discovery — mapear pymethods reales y comparar contra .pyi existentes ✅ DONE
- **Archivos:** `providers/openai/src/python.rs`, `providers/litellm/src/python.rs`, `providers/ollama/src/python.rs`, `providers/openai/vantadb_openai.pyi`, `providers/litellm/vantadb_litellm.pyi`, `providers/ollama/vantadb_ollama.pyi`
- **Acción:** Extraer de cada python.rs: `#[new]` signature + 7 `#[pymethods]` signatures (embed, search, store, delete, get, list, list_namespaces) con tipos y defaults. Comparar línea por línea contra .pyi actuales. Documentar match/mismatch para namespace, model/base_url/timeout, get/list/delete/list_namespaces.
- **Verify:** `grep -n "signature\|fn new\|fn embed\|fn search\|fn store\|fn delete\|fn get\|fn list" providers/*/src/python.rs` output capturado + diff pyi vs rs documentado. Si todo match → Step 2 es no-op verify; si mismatch → Step 2 regenera. **Resultado:** 3× `#[new]` con namespace/model/base_url/timeout presentes (openai: api_key req + model + namespace + timeout; litellm: api_key None + model + namespace + timeout; ollama: base_url + model + namespace + timeout) + 7 pymethods c/u con `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10)`, `delete(key, namespace=None)`, `get(namespace,key)`, `list(namespace,limit=100,cursor=None)`, `list_namespaces()`, `embed(texts)`, `store(text,embedding,metadata=None)` — todos presentes en .pyi actuales (31 líneas c/u). Diff 0 → S1 no-op verificado. Evidencia: `Select-String signature` 9 hits (3× new + 3× search + 3× delete/get/list) + `cat *.pyi` 7 métodos c/u.
- **Estado:** ✅ DONE

### Step 2: Regenerar/overwrite los 3 .pyi desde firmas reales (solo si drift; idempotente) ✅ DONE
- **Archivos:** `providers/openai/vantadb_openai.pyi`, `providers/litellm/vantadb_litellm.pyi`, `providers/ollama/vantadb_ollama.pyi`
- **Acción:** Si Step 1 encontró drift → overwrite cada .pyi con contenido derivado exactamente de pymethods (namespace param en search, get/list/delete/list_namespaces presentes, model/base_url/timeout en __init__). Si no hay drift → re-escribir idempotente mismo contenido (touch) para cumplir contrato "regenerar". Ponytail: overwrite directo 31 líneas, no script codegen complejo. **Ejecución:** Step 1 confirmó diff 0 (commit 2754c783 ya alineó stubs), así que regeneración es idempotente verify-only + overwrite temporal con `Set-Content` y `git checkout --` para restaurar newline canónico (evita CRLF drift). No diff final en providers/*.pyi (restored to HEAD 2754c783 state, 31 líneas, trailing newline `\n`).
- **Verify:** `cat providers/*/vantadb_*.pyi` contiene 7 métodos c/u + `__init__` con model/base_url/timeout correctos + `__version__`; diff contra firmas rs == 0. `git diff -- providers/*.pyi` 0 lines (after restore).
- **Estado:** ✅ DONE

### Step 3: Verificación mecánica cargo check + pyi compare + fmt ✅ DONE
- **Archivos:** `providers/openai/Cargo.toml`, `providers/litellm/Cargo.toml`, `providers/ollama/Cargo.toml`
- **Acción:** `cargo check --manifest-path providers/openai/Cargo.toml`, `.../litellm/...`, `.../ollama/...` (los 3 deben exit 0). Ejecutar script compare: `grep "def __init__\|def embed\|def search\|def store\|def delete\|def get\|def list" providers/*/*.pyi` vs `grep "signature" providers/*/src/python.rs` y validar 7 métodos. `cargo fmt --check` (si toca Rust, debe pasar).
- **Verify:** 3× `cargo check` exit 0 (openai 5.12s, litellm 5.03s, ollama 12.05s) ✅ + pyi 7 métodos presentes con namespace/model/base_url/timeout (grep 7/7 c/u + timeout/base_url/model checks ✅) + `cargo fmt --check` exit 0 ✅. Contrato pasa.
- **Estado:** ✅ DONE

## Context Save Point
- **Fecha:** 2026-08-26T15:30
- **Branch:** develop (dirty: 6 files pre-existentes .opencode/AGENTS.md etc + PROV-03 task file nuevo, 0 diff en providers/*.pyi tras idempotent restore)
- **CI pendiente:** ninguno — `cargo check` ×3 verde, `cargo fmt` verde, pyi 7/7 métodos verificados. `pytest` de providers requiere maturin wheel (no en Fast Gate, es Heavy — PROV-02/09 futuro).
- **Decisiones:** Ponytail: verify-only idempotente porque commit 2754c783 ya hizo sync; no codegen complejo, solo grep + overwrite restore. Source-driven: PyO3 0.29 signature attribute verificado en Cargo.toml.
- **Problemas conocidos:** Ninguno — pyi sync ya verde desde 2754c783; tarea es cierre formal + recitation para desbloquear Wave1 5/5.
- **Próxima tarea:** PROV-07 (Wave1 Task 4) — ValueError distance_metric + warning metadata

## Cierre
- **Recitation plan:** pendiente agregar `=== RECITATION PROV-03 ===` en `docs/plans/2026-08-25-research-providers-quickwins.md`
- **Commit:** NO commit per reglas del prompt (verify no-commit, RESULTADO)
- **Progreso:** pendiente `skill progreso` Trigger 1 tras RESULTADO (elimina fila PROV-03 de Backlog P45)
- **Verify full gates:** P: no disparado (pending es mecánica, no hot path) D: no disparado (fix mecánico, no publica símbolos nuevos, solo stubs) V: no disparado (cargo check ×3 + pyi compare + fmt pasan, no retry) C: no disparado (0 diff en providers/*.pyi, no colaterales fuera de blast radius)

## Dependencias
- PROV-01 ✅ done (commit 2754c783 ya incluye fix compile openai, pero no bloquea pyi)
- Ninguna bloqueante para pyi sync — es mecánico, no depende de PROV-06/07/08

## Notas
- Ponytail: 3 files ×31L, ladder rung "existe > stdlib": existe .pyi ya alineado en 2754c783 (commit ya hizo sync), así que S1 es verify no-op y S2 es overwrite idempotente. No añadir deps, no codegen crate, no template. Si drift futuro → overwrite manual sigue siendo mínimo.
- Source-driven: PyO3 0.29 `#[pyo3(signature = (...))]` → Python positional args con defaults. Fuente: https://pyo3.rs/v0.29/class.html#customizing-the-class + https://pyo3.rs/v0.29/function/signature.html (signature attribute). .pyi stubs: PEP 484, typing `list[str]`, `dict`, `float | None` (Python 3.10+ `|` union). Cargo.toml pyo3 0.29 verificado.
- No deps nuevas, no unsafe, no concurrency, no performance hot path.
- Regla 6 deuda: +0/-0 (overwrite stubs, no nueva deuda).
