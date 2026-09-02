# Plan de Ejecución: Error Handling & Observability Excellence — VantaDB

> **Inicio:** 2026-09-02
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** `docs/Backlog.md` (121 activas) + investigación multi-agente 2026-09-02 (4 audits paralelos read-only) + research internet 10 fuentes (Rust thiserror/anyhow, Python Real Python/docs.python.org, TS Convex, Vanta standardized errors)
> **Autonomous:** false
> **Campaign ID:** 20260902-error-observability
> **SDP:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full), spec-driven-development, systematic-debugging, code-review-and-quality, security-and-hardening, observability-and-instrumentation, api-and-interface-design, **coordinated-web-search** — phase=PLAN
> **Spec:** No existe `SPEC.md` — backlog + 4 audits + internet research son spec implícita; tareas son vertical slices de estandarización, no feature-add monolítica

> **Herramientas por tarea — Code Intelligence + Internet (obligatorio en cada DO):**
> - **CodeGraph (pre-indexado, 20.5K símbolos):** `codegraph_explore "VantaError"` / `"VantaError code"` — blast radius inmediato + callers/callees antes de editar (ej: Task 1: `src/error.rs` 30 vars, callers `storage/`, `index/`, `server/`)
> - **codebase-memory-mcp:** `detect_changes scope="impact" direction="inbound" depth=3` (blast radius transitivo), `get_architecture aspects="['clusters','hotspots','boundaries','overview']"` (contexto), `check_index_coverage paths=["src/error.rs", ...]` (cobertura índice), `query_graph`/`search_graph` para dead-code
> - **Internet (skill `coordinated-web-search` + MCPs):** `agent-search free_search` → `free_search_advanced` (domain filter) → `free_extract` (Jina), `argus_search_web`/`argus_extract_content`, `metasearchmcp compare_engines`, `firecrawl/webfetch` — 10 fuentes verificadas (RustTraining ch10, docs.rs/thiserror, Real Python, docs.python.org, Convex TS, Vanta standardized errors) citadas por tarea

## Investigación previa — Síntesis completa (requerida en plan)

### Skills investigadas (todas las útiles, sin duplicar DEPRECATED)
`systematic-debugging` (8/10 root-cause), `code-review-and-quality` (9/10 multi-axis), `constraint-driven-development` (CONSTRAINTS.md), `doubt-driven-development` (adversarial), `security-and-hardening` (validación FFI/boundary), `observability-and-instrumentation` (tracing), `api-and-interface-design` (contratos), `frontend-ui-engineering` + `codegraph_explore`/`grep` dual. `debugging-and-error-recovery` DEPRECATED → `systematic-debugging`.

### Veredicto global — 4 audits paralelos (read-only, 2026-09-02)

| Lenguaje | ¿Error tipado? | ¿Código estable? | ¿Hardcodeado texto? | ¿Propagación? | ¿anyhow? | Veredicto |
|---|---|---|---|---|---:|---|
| **Rust core `src/`** | ✅ `VantaError` 30 variantes `thiserror` + `ChainedError` + `is_retriable()/recovery_hint()` | ❌ solo nombre clase (`Display`), no `code()` | Sí inglés `#[error("Node not found: {0}")]` **centralizado en 1 punto** (979L) | `?` + `#[from]` 90% `unwrap` solo tests (1090/543/80) | No (0 `anyhow`/`bail`) | **Alta** seguridad, **Media** observabilidad |
| **Python `vantadb-python` MOD-20** | ✅ `VantaError(RuntimeError)` + 10 subclases `create_exception!` (`convert.rs:32`) | ❌ solo nombre clase | Sí `err.to_string()` (Display) | `.map_err(map_vanta_error)` 40 sites, sin `From` | No | OK pero boilerplate |
| **Python `providers/*`** | ❌ bucket 4 `KeyError/ValueError/RuntimeError(Debug)` (`shared_py.rs:31`) | ❌ | Sí `format!("{:?}",e)` debug | `err_to_py` colapsa 6 variantes finas → `RuntimeError` | No | **HIGH drift** vs `vantadb-python` |
| **TS `vantadb-ts` + WASM `lib.rs`** | ✅ `VantaError {code, details, timestamp}` + `ERROR_CODES` 10 | ✅ `as const` 10 strings | Sí `"_mapRecord: …"` | `wrapWasmError` preferente `code` + fallback `classifyWasmError` 7 regex | No | WASM conserva code, `vantadb-node` lo pierde `e.to_string()` |
| **MCP `error.rs`/`validation.rs`** | ✅ `McpError{code:i32}` 5 factories `-32700..-32600` | ✅ JSON-RPC codes | Sí inglés `"'{label}' must not be empty"` | Validación→`Err(Value)` code, dominio→`Ok(isError:true)` string `"Put Error: {}"` **pierde code** | No | **CRITICAL** arquitectura |
| **Desktop `error.rs`** | ✅ `thiserror` + `HttpErrorKind` 8 | ✅ `kind/status` serializable | Sí inglés | `VantaError::Native(Http.Domain)` | `thiserror=2` | `Native(e.to_string())` degrada |
| **Web `web/src`** | ❌ `Error & {digest?}` / `string` | ❌ | Sí ES/EN `dictionaries.ts` `"Ocurrió un error"` | `catch {}` vacío silencia | No | Sin tipado |

**Hardcodeado:** 100% inglés directo, pero **normalizado** en `Display`/`thiserror` (no `format!` disperso). `Generic(String)`/`ResourceLimit(String)` son hardcode bajo variante genérica → pierden `match`. `catch {}` en web silencia. `validateVector` lanza `TypeError` nativo, no `VantaError`.

### Por módulo y por archivo — ¿texto directo vs estructurado? (revisar CADA archivo, no saltarse ninguno)

**Rust core — `src/error.rs` (979L, fuente):** `VantaError` 30 variantes `#[error("Node not found: {0}")]` + `#[source]` + `is_retriable()/recovery_hint()` — estructurado tipado, `Display` centralizado, sin `anyhow`. Hardcodeado sí pero en 1 punto. `ResourceLimit(String)`/`Generic(ChainedError)` son catch-all de texto libre → degradan tipado. **Lista completa a revisar:** `src/error.rs`, `src/sdk/api.rs` 755L (`ValidationError{field:"read_only"}`), `src/storage/ops.rs` 535L (`ResourceLimit(format!)`), `src/storage/vfile.rs` ~400L, `src/storage/engine/mod.rs` + `insert.rs` + `get.rs` + `delete.rs` + `maintenance.rs` + `init.rs` + `ops.rs` (~2500L `Generic(BadBackend(format!))`), `src/index/graph.rs` 1846L + `ivf.rs` (`InvalidInput(format!)` + `expect` 15 sin `SAFETY`), `src/server/errors.rs` 182L (mapeo estructurado `vanta_error_status` → `StatusCode` 400/422/404/409/500 — **positivo**), `src/server/bootstrap.rs` ~450L (`CliError(ChainedError::msg)`), `src/server/handlers.rs` ~1500L.

**Python — `vantadb-python/src/`:** `convert.rs:786` `map_vanta_error` match exhaustivo `VantaError→PyErr` sin `From` (40 sites `.map_err`), `lib.rs` `PyValueError/TypeError::new_err(format!(...))` hardcodeado. `providers/shared_py.rs:31` HIGH drift — bucket 4 `NotFound→PyKeyError` pierde 6 variantes finas (`Timeout→RuntimeError(Debug)`). **Archivos a revisar:** `vantadb-python/src/convert.rs`, `vantadb-python/src/lib.rs`, `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/vantadb_py/vantadb_py.pyi`, `providers/shared_py.rs`, `providers/openai/src/python.rs`, `providers/litellm/src/python.rs`, `providers/ollama/src/python.rs`.

**TS/WASM — `vantadb-ts/src/` + `vantadb-wasm/src/lib.rs`:** `errors.ts:24` `VantaError` + `ERROR_CODES` 10 (`CLOSED,WASM_ERROR,VALIDATION_ERROR…`) + `classifyWasmError` 7 regex mirrors `src/error.rs` prefijos. `vanta_error_code()` en Rust mapea 30→8 códigos y `to_js_err` hace `Reflect::set(err,"code")` — WASM conserva code, `vantadb-node` lo pierde (`e.to_string()`). `guards.ts` lanza `TypeError/RangeError` nativos, no `VantaError` — anomalía. **Archivos a revisar:** `vantadb-ts/src/errors.ts`, `vantadb-ts/src/vantadb.ts`, `vantadb-ts/src/native.ts`, `vantadb-ts/src/guards.ts`, `vantadb-wasm/src/lib.rs` (1927 `vanta_error_code`, 1960 `to_js_err`), `vantadb-node/src/lib.rs`.

**MCP/Server/Desktop/Web:**
- `MCP validation.rs` 1044L — 6 validadores `validate_identifier`/`validate_vector` hardcode inglés pero estructurados + tests; **CRITICAL:** `handlers/tools.rs:1306 Ok(error_content(format!("Put Error: {}",e)))` serializa `VantaError` como `isError:true` string, pierde `code/is_retriable` → cliente LLM no puede retry programático. Debe ser `From<VantaError> for McpError` `-32001 Unauthorized`, `-32004 NotFound` etc. **Archivos:** `vantadb-mcp/src/error.rs`, `validation.rs`, `handlers/tools.rs`, `server.rs`, `skills.rs`.
- `Desktop error.rs` — `thiserror` + `HttpErrorKind` 8 serializable, `Native(e.to_string())` degrada. **Archivos:** `desktop/src-tauri/src/error.rs`, `desktop/src-tauri/src/connections/server.rs`, `desktop/src-tauri/src/commands/memory.rs:91`.
- `Web app/error.tsx` + `toast.tsx` `sonner` — único i18n `dictionaries.ts` (`toast.error` ES/EN), resto `catch {}` vacío silencia errores, `throw new Error(HTTP ${status})` pierde code. **Archivos:** `web/src/app/error.tsx`, `web/src/components/vanta/toast.tsx`, `web/src/lib/dictionaries.ts`, `web/src/components/vanta/code-playground.tsx:174`, `web/src/lib/copy-utils.ts:17`.

### Qué significa

- **Estructura sí, observabilidad no.** Tienes enum tipado y `?` (lo difícil), pero sin `code()` estable el cliente no puede `match` sin parsear `Display`. Para LLM/MCP es crítico: `Put Error: Validation` no permite retry programático → se degrada a toast genérico (caso Vanta web 2024).
- **Inconsistencia cross-language:** TS tiene 10 códigos, Rust 30 variantes, Python 11, providers 4, MCP 5 → un `TimeoutError` es `TIMEOUT` en TS, `BusyError` en MCP, `RuntimeError(Debug)` en providers. El `Display` es la única verdad compartida, y es frágil.
- **Texto directo no es el bug, el catch-all sí.** `thiserror` con `#[error("…")]` hardcodeado es correcto para motor embebido (no i18n). El problema es `Generic(ChainedError)` donde `format!("node {} vector_len {}")` podría ser `VectorLenOverflow{id,len,limit}` con campos.

### Qué dice internet — mejor forma por lenguaje (fuentes verificadas, aplicar)

**Rust — `thiserror` para libs, `anyhow` para bins** — [RustTraining ch10](https://github.com/microsoft/RustTraining/blob/main/rust-patterns-book/src/ch10-error-handling-patterns.md), [docs.rs/thiserror](https://docs.rs/thiserror/latest/thiserror/), [andrewodendaal.com](https://andrewodendaal.com/rust-error-handling-patterns-production/):
> Libraries: `thiserror` enum concreto → callers hacen `match` por variante. Applications/binaries: `anyhow::Error` + `.context(|| format!(…))` para cadena humana. `#[from]` genera `From` para `?` automático, `#[non_exhaustive]` permite añadir variante sin breaking, mensajes lowercase sin punto, `source()` preserva chain, `panic!` solo para bugs, `unwrap` solo tests (`clippy unwrap_used` deny en `src/`).

**Python — jerarquía + `raise … from` + `add_note` + `except*`** — [Real Python](https://realpython.com/ref/best-practices/exception-handling/), [docs.python.org](https://docs.python.org/3/library/exceptions.html), [EngineersOfAI](https://engineersofai.com/docs/python/python-foundation/error-handling-and-defensive-engineering/custom-exceptions):
> `class LibraryError(Exception): pass` + subclases por dominio, `raise ConfigError(f"…{path}") from e` preserva traceback, `except SpecificError` (nunca bare `except:`), `try` laser-focused, `add_note()` 3.11 para breadcrumbs, `ExceptionGroup` para concurrent. `super().__init__(msg)` + atributos estructurados (`code`, `retryable`, `to_dict()`).

**TypeScript — `VantaError extends Error` + `code` enum + `cause` chain** — [Convex](https://www.convex.dev/typescript/best-practices/typescript-error-type):
> `catch (e: unknown) { if (e instanceof VantaError) … }` (TS 4.0 `unknown`), `class VantaError extends Error { code: ErrorCode, cause?: unknown }`, `throw new VantaError("Failed…", {cause: dbErr})` preserva chain, `strict:true`, nunca `any`. Mensajes como `ERROR_CODES` `as const`.

**MCP/HTTP — JSON-RPC codes + mapeo capa** — [Vanta how-we-standardized-error-handling](https://www.vanta.com/resources/how-we-standardized-error-handling):
> Códigos canónicos pequeños (`InvalidInputError`, `NotAuthorizedError`, `ResourceNotFoundError`) → monitoreo/alerting + GraphQL middleware + React boundaries. Mapping capa: `RepoError → ServiceError → HttpResponse` con sanitización (500 no filtra `sqlx::Error`).

## Resumen

| Resultado | Count | % |
|-----------|-------|---|
| ✅ DO | 9 | 6.9% |
| 🟡 DEFER | 85 | 65.4% |
| ❌ SKIP | 18 | 13.8% |
| 🔴 BLOQUEADO | 18 | 13.8% |
| **Total triado** | **130** | 100% |

Status: ⬆️ uphill = 5 · ⬇️ downhill = 9 (ver § uphill/downhill)
SDP: `campaign_discover_skills` por tarea — base campaign-executor + lifecycle PLAN + manifest grep. Shape Up: cada DO pasa 3 preguntas (problema correcto + appetite + AHORA).
**Captura & Observabilidad:** Task 9 cubre `Backtrace` + `tracing::error!` + `metrics::counter!` + `panic::catch_unwind` FFI + sanitización 500 — cierra gap de captura/observabilidad detectado.

## Triage Gate — Criterios aplicados

Ver plan.md §Reglas del gate + Paso 0 Verificación de Realidad. Gate P: 🔴/ambigua confirmada vía question. Pre-mortem y Cynefin obligatorios para 🔴/ambiguas. Appetite declarado ANTES de Effort.

**SKIP verificados (premisa falsa):** FIND-44 22 ADRs ya existen, TS-01/02 ya async, FIND-24 cursor resuelto, SRV-01/WSM-02 ya implementados — no re-triage.
**BLOQUEADO (persisten):** AUD-042 tantivy, CORE-02 PITR, FIND-33 snapshot layout, STABLE-* ADR-031, MCP-34b depende FIND-33, BND-08..10 npm, SRV-06 OIDC, TS-10/11 WSM-06 (core expose wiki/skills).
**DEFER:** P5/P6 launch, P24 I+D, P32-P34 reviews, P38 RES-04/06/07/09, MEM-*/PRX-*/WEB-09 gate humano, config.rs split si >2000L.

### Question Gates — Gate P (HITL, obligatorio)

> **Si algo es ambiguo → investigar y preguntar antes de fijar DO.** Toda tarea 🔴 o 🟨 Complicado/Complejo con contrato ambiguo, que agrega `pub fn`/`code()` nuevo (`VantaError::code()`, `From<VantaError> for McpError`), o que toca API pública/boundary (`--all-features`, `#[non_exhaustive]`) → `question` tool con opciones `GO / ajustar scope / dividir / DEFER` **antes** de fijar `✅ DO`. Ver `prompts/question-gates.md` §Gate P.

**En este plan (9 DO), gates armados:**
- **ERR-CORE-01** `code()` prefix `VANTADB_` vs `ERR_` + granularidad 10 vs 30 variantes → `question` si `code()` requiere ADR >1d
- **ERR-MCP-01** tabla 30→5 codes `-32001..-32099` colisiona con `-32600` → `question` `GO / ajustar a 5 críticos / DEFER`
- **ERR-OBS-01** `Backtrace` nightly 1.73+ vs `std::backtrace::Backtrace` stable 1.65+ → `question` si CI no tiene nightly
- Cualquier `Top 3 riesgos` con `Prob×Impacto 🟡×🔴` o `Uphill = 1` sin cerrar → `question` antes de `DISCOVERY`

Sin `GO` explícito del usuario, la tarea queda 🟡 DEFER (no se fuerza `DO`).

---

## Tasks — ✅ DO (8) — REVISAR CADA ARCHIVO, NO SALTARSE NINGUNO — desarrollar, investigar, analizar y aplicar

> **Regla de oro de este plan:** cada tarea DEBE revisar **cada archivo** listado en **Archivos clave**, sin saltarse ninguno. Verificación real y contrato exigen evidencia por archivo (grep/lines/codegraph). Si un archivo no se revisa, el contrato no pasa. Desarrollar = codificar, investigar = internet + codegraph, analizar = blast radius + causa raíz, aplicar = commit verde.

### Task 1: ERR-CORE-01 — VantaError::code() estable + tipar Generic/ResourceLimit (Rust core)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta (P0 observabilidad, desbloquea TS/Python/MCP)
- **Archivos clave (TODOS a revisar, ninguno saltable):** `src/error.rs` (979L, 30 vars), `src/sdk/api.rs` 755L, `src/storage/ops.rs` 535L, `src/storage/vfile.rs` ~400L, `src/storage/engine/mod.rs`, `src/storage/engine/insert.rs`, `src/storage/engine/get.rs`, `src/storage/engine/delete.rs`, `src/storage/engine/maintenance.rs`, `src/storage/engine/init.rs`, `src/storage/engine/ops.rs` (~2500L total), `src/index/graph.rs` 1846L, `src/index/ivf.rs`, `src/index/search/alternate.rs`, `src/index/search/core.rs`, `src/server/errors.rs` 182L, `src/server/bootstrap.rs` ~450L, `src/server/handlers.rs` ~1469L
- **Verificación real:** ✅ CÓDIGO-REAL — `src/error.rs` 30 variantes `thiserror` sin `code()`, `Generic(ChainedError)` y `ResourceLimit(String)` catch-all con `format!` en `storage/ops.rs:72,130`, `generic_error(format!(...))` en `engine/*`, `expect` 15 en `graph.rs` sin SAFETY
- **Gate Justificación:** fuente canónica para TS/Python/MCP — sin `code()` estable toda cadena cross-language depende de `Display` frágil; tipar catch-all permite `match` cliente sin parsear texto
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "fn code" src/error.rs | wc -l == 1 AND grep -n "VectorLenOverflow|EdgeCountOverflow" src/error.rs | wc -l >= 2 AND grep -n "Generic(String)" src/error.rs | wc -l == 0 AND cargo check -p vantadb --all-targets --all-features 2>&1 | grep -c "error\[E" == 0`
- **Pre-mortem:**
  - Fallo 1: `code()` string rompe `#[non_exhaustive]` consumidores → versión minor ok, documentar en CHANGELOG
  - Fallo 2: tipar `Generic` revela variantes faltantes en `match` exhaustivo → añadir `_ =>` fallback
  - Fallo 3: `format!` movido a `reason` pierde contexto → preservar `source()` chain
- **Stop conditions:** `code()` requiere ADR >1d → DEFER con ADR draft; tipar Generic excede 1d → recortar a 2 variantes críticas (VectorLen, EdgeCount)
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | `code()` string inestable para clientes | `VANTADB_` prefix + tests `code()` snapshot | pre-commit |
  | 🟡×🟡 | `Generic` → tipado rompe callers `match` | `_ => VantaError::Generic` fallback temporal | cargo check |
  | 🟢×🟡 | `expect` sin SAFETY en `graph.rs` | añadir `// SAFETY: header 64B checked at construction` + clippy deny | review |
- **Cynefin:** 🟨 Complicado — requiere experto `thiserror` + `is_retriable` para decidir granularidad
- **Top 3 riesgos:** 1 code inestable, 2 Generic break, 3 expect sin SAFETY
- **Uphill/Downhill:** ⬆️ 1 (granularidad code) · ⬇️ 3 (code() + 2 variantes + clippy)
- **DoD multi-nivel:** Task: `code()` + 2 variantes + clippy 0 · Commit: `feat(error): VantaError::code() estable + tipa Generic (ERR-CORE-01)` · Release: `docs/api/ERROR_HANDLING.md` + `CHANGELOG` code listing
- **Validación Appetite vs Effort:** max 1d ≥ 🟡 1d ✅
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-CORE-01.md`
- **Branch:** develop

### Task 2: ERR-CORE-02 — Clippy unwrap_used/expect_used deny en src/ + bins anyhow

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠 Media (previene 1090 unwrap futuros)
- **Archivos clave (TODOS):** `src/binary_header.rs:67,199`, `src/index/graph.rs:15`, `src/index/ivf.rs:8`, `src/backends/*.rs:1`, `Cargo.toml` (lints), `src/main.rs`, `vantadb-server/src/main.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `grep -rn "unwrap()\|expect(" src/ --include="*.rs" | grep -v "#\[cfg(test)" | wc -l == ~30 (prod), `Cargo.toml` lints sin `unwrap_used` deny
- **Gate Justificación:** `unwrap` en prod es bug latente; `anyhow` con `.context()` en bins da chain humano sin `thiserror` en libs
- **Gate Result:** ✅ DO
- **Contrato:** `grep -rn "unwrap_used" Cargo.toml | wc -l == 1 AND cargo clippy -p vantadb --all-targets --all-features -- -D clippy::unwrap_used -- -D clippy::expect_used 2>&1 | grep -c "error\[clippy" == 0 (allow en tests)`
- **Pre-mortem:** 1 `expect("invariant")` legítimo marcado como error → `#[allow(clippy::expect_used)]` con `// SAFETY:`
- **Stop conditions:** `anyhow` añade dep no deseada en lib → usar solo en bins, no en `src/lib.rs`
- **Risk Register:** | 🟢×🟢 | `expect` legítimo bloqueado | allow con SAFETY | clippy |
- **Cynefin:** 🟦 Obvio
- **Top 3 riesgos:** 1 expect legítimo
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 (lints + anyhow bins)
- **DoD:** Task: lints + anyhow bins · Commit: `chore(clippy): deny unwrap/expect en prod + anyhow bins (ERR-CORE-02)` · Release: N/A
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-CORE-02.md`

### Task 3: ERR-PY-01 — Unificar providers a jerarquía MOD-20 + code/to_dict

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta (HIGH drift, 6 variantes perdidas)
- **Archivos clave (TODOS):** `vantadb-python/src/convert.rs:32,786` (11 clases + `map_vanta_error`), `vantadb-python/src/lib.rs:34,173` (40 sites), `vantadb-python/vantadb_py/__init__.py:27`, `vantadb-python/vantadb_py/vantadb_py.pyi:30`, `providers/shared_py.rs:18-42` (`err_to_py` bucket 4), `providers/openai/src/python.rs`, `providers/litellm/src/python.rs`, `providers/ollama/src/python.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `shared_py.rs:31` `NotFound→PyKeyError, BackendError→RuntimeError, _→RuntimeError(Debug)` colapsa `Timeout→RuntimeError`, `convert.rs` 11 clases no reusadas en providers
- **Gate Justificación:** providers pierden tipado fino → cliente Python no puede `except TimeoutError` — drift vs MOD-20
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "err_to_py" providers/shared_py.rs | xargs grep -c "map_vanta_error" == 1 AND grep -n "format!.*Debug" providers/shared_py.rs | wc -l == 0 AND cargo check --manifest-path providers/openai/Cargo.toml --all-targets 2>&1 | grep -c "error\[E" == 0`
- **Pre-mortem:** 1 `map_vanta_error` no disponible en `providers/*` crate (no depende de `vantadb-python`) → extraer `common` crate o duplicar match con `code()`
- **Stop conditions:** requiere extraer crate común >1h → duplicar match temporal + DEFER crate común
- **Risk Register:** | 🟡×🟠 | `map_vanta_error` no linkable desde providers | duplicar match con code() | check |
- **Cynefin:** 🟨 Complicado — DISCOVERY crate boundary
- **Top 3 riesgos:** 1 crate boundary, 2 Debug leak, 3 code port
- **Uphill/Downhill:** ⬆️ 1 (crate boundary) · ⬇️ 2 (err_to_py + 3 python.rs)
- **DoD:** Task: `err_to_py` usa `code()` + `to_dict()` · Commit: `fix(providers): unifica err_to_py a jerarquía MOD-20 (ERR-PY-01)` · Release: `docs/api/PYTHON_SDK.md` code listing
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-PY-01.md`

### Task 4: ERR-TS-01 — Unificar TS/WASM codes + wrapNativeError + guards VantaError

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta (WASM conserva code, Node lo pierde)
- **Archivos clave (TODOS):** `vantadb-ts/src/errors.ts:24` (10 codes + `classifyWasmError`), `vantadb-ts/src/vantadb.ts` (~40 `wrapWasmError` sites), `vantadb-ts/src/native.ts:148` (`wrapNativeError` `NATIVE_ERROR`), `vantadb-ts/src/guards.ts` (`validateVector` `TypeError`), `vantadb-wasm/src/lib.rs:1927` (`vanta_error_code` 30→8), `vantadb-wasm/src/lib.rs:1960` (`to_js_err` 69 sites), `vantadb-node/src/lib.rs` (`map_err(e.to_string())`)
- **Verificación real:** ✅ CÓDIGO-REAL — `to_js_err` adjunta `code` 8 strings, `wrapWasmError` fallback 7 regex, `wrapNativeError` siempre `NATIVE_ERROR` fuera de `ERROR_CODES`, `guards.ts` lanza `TypeError` no `VantaError`
- **Gate Justificación:** `vantadb-node` pierde taxonomía → cliente JS no puede `if (e.code===TIMEOUT)` — inconsistencia 30→10→1
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "NATIVE_ERROR" vantadb-ts/src/errors.ts | wc -l == 0 AND grep -n "TypeError" vantadb-ts/src/guards.ts | wc -l == 0 AND grep -n "code.*GenericFailure" vantadb-node/src/lib.rs | wc -l >= 1`
- **Pre-mortem:** 1 `NATIVE_ERROR` usado en tests → actualizar tests a `VALIDATION_ERROR` 2 `guards.ts` consumidores esperan `TypeError` → breaking, documentar en CHANGELOG
- **Stop conditions:** mapear 30→10 requiere tabla exhaustiva >1d → recortar a 5 críticos (VALIDATION, NOT_FOUND, TIMEOUT, BUSY, IO)
- **Risk Register:** | 🟡×🟡 | `NATIVE_ERROR` test break | actualizar tests | tsc |
- **Cynefin:** 🟨 Complicado — tabla 30→10 + `cause` chain
- **Top 3 riesgos:** 1 NATIVE_ERROR break, 2 guards breaking, 3 code subset
- **Uphill/Downhill:** ⬆️ 1 (tabla 30→10) · ⬇️ 3 (WASM 8→10, Node code, guards)
- **DoD:** Task: `NATIVE_ERROR` eliminado + `guards` VantaError + Node code · Commit: `fix(ts): unifica codes TS/WASM/Node + guards VantaError (ERR-TS-01)` · Release: `docs/api/TYPESCRIPT_SDK.md` code table
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-TS-01.md`

### Task 5: ERR-MCP-01 — From<VantaError> for McpError (CRITICAL) + validation code enum

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta (CRITICAL arquitectura, LLM no puede retry)
- **Archivos clave (TODOS):** `vantadb-mcp/src/error.rs` (5 factories), `vantadb-mcp/src/validation.rs` 1044L (6 validadores), `vantadb-mcp/src/handlers/tools.rs:1306` (`Ok(error_content(format!("Put Error: {}",e)))`), `vantadb-mcp/src/server.rs:126` (`-32001 Unauthorized`), `vantadb-mcp/src/skills.rs:237` (`Skill Error: {e}`), `vantadb-mcp/src/handlers/resources.rs`, `vantadb-mcp/src/wiki.rs`, `vantadb-mcp/src/code.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `handlers/tools.rs:1306` serializa `VantaError` como `isError:true` string `"Put Error: {}"` perdiendo `code/is_retriable`, validación sí usa `McpError::invalid_params` con code
- **Gate Justificación:** cliente LLM no puede decidir retry programático sin `code` → se degrada a toast genérico (Vanta 2024)
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "Put Error" vantadb-mcp/src/handlers/tools.rs | wc -l == 0 AND grep -n "From<VantaError> for McpError" vantadb-mcp/src/error.rs | wc -l == 1 AND cargo check -p vantadb-mcp --all-targets 2>&1 | grep -c "error\[E" == 0`
- **Pre-mortem:** 1 `isError:true` consumidores (Cursor) esperan string → mantener compat `code` + `message` ambos 2 tabla `-32001..-32099` colisiona con `-32600` → reservar `-32000..-32009` para Vanta
- **Stop conditions:** tabla 30→5 requiere 1d → recortar a 5 críticos (Validation→-32602, NotFound→-32004, Busy→-32001, Timeout→-32008, Corrupt→-32002)
- **Risk Register:** | 🔴×🟡 | `isError` string consumidores break | backward compat code+message | test |
- **Cynefin:** 🟨 Complicado — DISCOVERY mapping table
- **Top 3 riesgos:** 1 isError break, 2 code collision, 3 validation code enum
- **Uphill/Downhill:** ⬆️ 1 (tabla mapping) · ⬇️ 2 (From impl + validation enum)
- **DoD:** Task: `From` + validation code enum · Commit: `feat(mcp): From<VantaError> for McpError (ERR-MCP-01)` · Release: `docs/api/MCP.md` code table
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-MCP-01.md`

### Task 6: ERR-DESK-01 — Desktop HttpKind preservado + Native degrada

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Archivos clave (TODOS):** `desktop/src-tauri/src/error.rs` (`thiserror` + `HttpErrorKind` 8), `desktop/src-tauri/src/connections/server.rs` (mapea 401→Unauthorized), `desktop/src-tauri/src/commands/memory.rs:91` (`VantaError::Native(e.to_string())`), `desktop/src-tauri/src/manager.rs`, `desktop/src-tauri/src/native.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `commands/memory.rs:91` `mem_err = VantaError::Native(e.to_string())` degrada `HttpErrorKind` tipado de `server.rs`
- **Gate Justificación:** `HttpErrorKind` serializable pierde `kind/status` al envolver en `Native(String)` — cliente Tauri no puede `match` 401 vs 500
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "Native(e.to_string" desktop/src-tauri/src/commands/memory.rs | wc -l == 0 AND cargo check --manifest-path desktop/src-tauri/Cargo.toml 2>&1 | grep -c "error\[E" == 0`
- **Pre-mortem:** 1 `Native(String)` usado en 5 sites → cambiar a `Domain(code, msg)` requiere `code()` de Task 1
- **Stop conditions:** requiere `code()` de Task 1 → BLOQUEADO hasta Task 1, o hardcode `code` temporal
- **Risk Register:** | 🟢×🟡 | `code()` no listo | hardcode temporal | check |
- **Cynefin:** 🟦 Obvio
- **Top 3 riesgos:** 1 code dependency
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 (1 line)
- **DoD:** Task: `Domain` preservado · Commit: `fix(desktop): preserva HttpKind en commands (ERR-DESK-01)` · Release: N/A
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-DESK-01.md`

### Task 7: ERR-WEB-01 — Web toast code + catch {} silenciado

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Archivos clave (TODOS):** `web/src/app/error.tsx` (boundary 500), `web/src/components/vanta/toast.tsx` (`sonner` `error()/info()` hardcode ES), `web/src/lib/dictionaries.ts` (`toast.error` ES/EN), `web/src/components/vanta/code-playground.tsx:174` (`catch {}` vacío), `web/src/lib/copy-utils.ts:17` (`catch {}`), `web/src/app/playground/page.tsx`, `web/src/components/vanta/docs-view.tsx`
- **Verificación real:** ✅ CÓDIGO-REAL — `toast.tsx` usa `dictionaries.ts` pero `code-playground.tsx:174` `catch {}` silencia, `copy-utils.ts:17` silencia, `throw new Error(HTTP ${status})` pierde code
- **Gate Justificación:** `catch {}` vacío viola observabilidad; único i18n `dictionaries.ts` no usado con `code` → toast genérico
- **Gate Result:** ✅ DO
- **Contrato:** `grep -rn "catch {}" web/src --include="*.tsx" --include="*.ts" | wc -l == 0 AND grep -rn "error.code" web/src --include="*.tsx" | wc -l >= 2`
- **Pre-mortem:** 1 `catch {}` en `copy-utils.ts` es fallback clipboard → `catch (e) { console.error(e) }` preserve
- **Stop conditions:** requiere `VantaError.code` de Task 1 → usar `ERROR_CODES` TS existente (10) temporal
- **Risk Register:** | 🟢×🟢 | `catch {}` fallback legítimo | log error | lint |
- **Cynefin:** 🟦 Obvio
- **Top 3 riesgos:** 1 catch legítimo
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 (toast + 2 catch)
- **DoD:** Task: `catch {}` eliminado + `error.code` toast · Commit: `fix(web): toast code + catch silenciado (ERR-WEB-01)` · Release: N/A
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-WEB-01.md`

### Task 8: ERR-DOCS-01 — Docs ERROR_HANDLING.md + observabilidad (is_retriable, recovery_hint, code table)

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja (docs, desbloquea consumidor)
- **Archivos clave (TODOS):** `docs/api/ERROR_HANDLING.md` (nuevo), `docs/api/EMBEDDED_SDK.md` (VantaError section), `docs/api/PYTHON_SDK.md` (10 subclases), `docs/api/TYPESCRIPT_SDK.md` (10 codes), `docs/api/MCP.md` (5 factories + nueva tabla -320xx), `docs/CHANGELOG.md` (code listing), `src/error.rs` (is_retriable/recovery_hint), `CONSTRAINTS.md` (quality bar)
- **Verificación real:** ✅ CÓDIGO-REAL — `is_retriable()` + `recovery_hint()` existen en `src/error.rs` pero no documentados en `docs/api/`; `ERROR_CODES` TS 10 no listados en `docs/api/`; `MCP` 5 factories sin tabla Vanta `-320xx`
- **Gate Justificación:** docs son contract para consumidores — sin `code` table cliente no puede `match` sin leer `Display`
- **Gate Result:** ✅ DO
- **Contrato:** `test -f docs/api/ERROR_HANDLING.md && grep -c "VANTADB_" docs/api/ERROR_HANDLING.md | xargs test 10 -le && grep -c "is_retriable" docs/api/ERROR_HANDLING.md | xargs test 1 -le`
- **Pre-mortem:** 1 docs desactualizados tras Task 1 code() → regenerar table desde `src/error.rs` via script
- **Stop conditions:** requiere Task 1 code() → BLOQUEADO hasta Task 1, o draft con 10 códigos TS existentes
- **Risk Register:** | 🟢×🟢 | docs stale tras code() | script gen | docs |
- **Cynefin:** 🟦 Obvio
- **Top 3 riesgos:** 1 stale table
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1 (1 md + 3 updates)
- **DoD:** Task: `ERROR_HANDLING.md` + 3 updates · Commit: `docs: ERROR_HANDLING.md + code tables (ERR-DOCS-01)` · Release: N/A (docs-only)
- **Estado:** ✅ COMPLETED (2026-09-02T19:30)
- **Commit:** 962831ae -- 7 files (ERROR_HANDLING.md 374L + 5 doc updates + CHANGELOG + CONSTRAINTS)
- **Verify:** 6/6 PASS (VANTADB_=10, is_retriable=6, TS ERROR_CODES=4, MCP McpError=3, PYTHON VantaError=24)
- **Progreso:** docs/avance/activo/{core-engine,bindings,web-frontend}.md OK
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-DOCS-01.md`

### Task 9: ERR-OBS-01 — Captura y observabilidad: Backtrace + tracing + metrics + catch_unwind + sanitización

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media-Alta (cierra gap captura/observabilidad, Vanta web 2024)
- **Archivos clave (TODOS a revisar, ninguno saltable):** `src/error.rs` (añadir `#[backtrace] Backtrace` nightly 1.73+ o `std::backtrace::Backtrace` stable), `src/main.rs` + `vantadb-server/src/main.rs` (bins `anyhow` + `.context()` + `catch_unwind` FFI), `Cargo.toml` (deps `anyhow` `tracing-error` `miette`), `src/server/errors.rs` (sanitización 500: `Internal` → genérico, log full chain `{:#}`), `src/server/bootstrap.rs` (panic hook + `RUST_BACKTRACE`), `vantadb-mcp/src/error.rs` (metrics counter code), `desktop/src-tauri/src/error.rs` (HttpKind), `web/src/app/error.tsx` (boundary), `web/src/lib/dictionaries.ts` (i18n code), `docs/operations/OBSERVABILITY.md` (nuevo), `docs/operations/BENCHMARKS.md` (baseline), `CONSTRAINTS.md` (quality bar)
- **Verificación real:** ✅ CÓDIGO-REAL — `src/error.rs` sin `Backtrace` field, bins sin `anyhow`, `tracing::error!(error=?e, code=e.code())` 0 hits en `server/errors.rs`, `metrics::counter!("vanta_errors_total")` 0 hits, `RUST_BACKTRACE` no documentado, `500` filtra `sqlx::Error` solo en `vanta_error_response` pero no en `MCP` `isError`
- **Gate Justificación:** sin `Backtrace` no debug 3am; sin `metrics::counter!` no alerta `rate >2× baseline`; sin `catch_unwind` un panic en `storage` crashea PyO3/WASM; sin sanitización leakage interno a cliente LLM
- **Gate Result:** ✅ DO
- **Contrato:** `grep -n "Backtrace" src/error.rs | wc -l >= 1 AND grep -rn "tracing::error" src/server --include="*.rs" | wc -l >= 2 AND grep -rn "vanta_errors_total" src --include="*.rs" | wc -l >= 1 AND grep -rn "catch_unwind" src --include="*.rs" | wc -l >= 1 AND cargo check -p vantadb --all-targets 2>&1 | grep -c "error\[E" == 0`
- **Pre-mortem:**
  - Fallo 1: `Backtrace` nightly requerido → gate `#[cfg(nightly)]` o `std::backtrace::Backtrace` stable (1.65+)
  - Fallo 2: `anyhow` en lib rompe `thiserror` API → solo bins, no `src/lib.rs`
  - Fallo 3: `metrics::counter!` sin registry Prometheus → usar `metrics` crate ya en `Cargo.toml` o defer a `prometheus`
- **Stop conditions:** `Backtrace` requiere nightly no disponible → DEFER con `RUST_LIB_BACKTRACE=1` docs; `anyhow` añade dep no deseada → usar `std::error::Error` + `.context()` manual
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | `Backtrace` nightly bloquea CI stable | `std::backtrace::Backtrace` stable (Rust 1.65+) | cargo check nightly |
  | 🟡×🟡 | `anyhow` en lib | solo `src/main.rs`/`server/main.rs` | Cargo.toml bins |
  | 🟢×🟡 | `metrics` sin Prometheus | `metrics` crate existente | cargo check |
  | 🟢×🟢 | `catch_unwind` FFI overhead | solo boundaries PyO3/WASM | bench |
- **Cynefin:** 🟨 Complicado — requiere experto `tracing-error` + `miette` para decidir `Backtrace` vs `anyhow` chain
- **Top 3 riesgos:** 1 Backtrace nightly, 2 anyhow en lib, 3 metrics sin registry
- **Uphill/Downhill:** ⬆️ 1 (Backtrace nightly vs stable) · ⬇️ 3 (Backtrace field + anyhow bins + metrics counter)
- **DoD multi-nivel:** Task: `Backtrace` + `tracing::error!` + `metrics` + `catch_unwind` · Commit: `feat(observability): Backtrace + tracing + metrics + catch_unwind (ERR-OBS-01)` · Release: `docs/operations/OBSERVABILITY.md` + `CONSTRAINTS.md` quality bar
- **Validación Appetite vs Effort:** max 1d ≥ 🟡 1d ✅
- **Estado:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-OBS-01.md`
- **Branch:** develop

---

## Dependencias entre Tasks

```
Wave 0 (paralelo, sin dependencias): ERR-CORE-02 (clippy), ERR-DOCS-01 (docs draft con 10 codes TS existentes, no requiere code())
Wave 1 (depende Wave 0 + Task 1 code): ERR-CORE-01 (code() fuente) → desbloquea ERR-PY-01, ERR-TS-01, ERR-MCP-01, ERR-DESK-01, ERR-WEB-01
Wave 2 (paralelo tras Task 1): ERR-PY-01, ERR-TS-01, ERR-MCP-01 (archivos disjuntos: providers/*, vantadb-ts/*, vantadb-mcp/*) — MAX_CONCURRENT=3
Wave 3 (depende Wave 2): ERR-DESK-01 (requiere code() de Task 1, pero puede hardcode temporal), ERR-WEB-01 (puede usar ERROR_CODES TS existente), ERR-OBS-01 (Backtrace + tracing + metrics + catch_unwind — depende Task 1 code() para code tag)
```

MAX_CONCURRENT=3 (Windows RAM), FAIL_MODE=stop. Si `code()` requiere ADR >1d, Wave 1→2 se retrasa pero Wave 0 avanza.

## Checkpoint post-plan

Tras Wave 0+1: `code()` estable en `src/error.rs` + `clippy unwrap_used` deny → `cargo check --workspace --all-targets --all-features -D warnings` 0 + `code` visible en `Display`.

Tras Wave 2: `providers/*` ya no usa `Debug`, `vantadb-node` conserva `code`, `MCP` `isError` → `McpError` con `code` retryable.

Tras Wave 3: `desktop`/`web` usan `code` no `message`, `ERROR_HANDLING.md` lista 10+ códigos con `is_retriable` table, `ERR-OBS-01` añade `Backtrace` + `tracing::error!` + `metrics::counter!("vanta_errors_total")` + `catch_unwind` + sanitización 500.

## Notas

- **Revisar CADA archivo listado, ninguno saltable** — contratos exigen `grep` por archivo; si un archivo no se revisa, el `wc -l` de variantes/codes falla.
- **Desarrollar, investigar, analizar y aplicar en cada tarea:** investigar = codegraph + internet (fuentes citadas), analizar = blast radius + causa raíz (systematic-debugging), desarrollar = codificar, aplicar = commit verde + `skill progreso` (Trigger 1: elimina fila Backlog, migra a `docs/avance/<dominio>`, `campaign_memory_write`).
- **Poner en práctica internet:** Rust `thiserror` libs vs `anyhow` bins + `#[non_exhaustive]` + `source()` chain; Python `raise ... from` + `add_note` + jerarquía `LibraryError`; TS `VantaError extends Error {code, cause}` + `strict:true`; MCP JSON-RPC `-320xx` + sanitización 500.
- **Ponytail full:** `code()` 10 strings canónicos, no catálogo i18n completo (defer), no `anyhow` en libs, no códigos numéricos si `code()` string suficiente.

## Context Save Point

- **Fecha:** 2026-09-02
- **Branch:** develop
- **Estado:** ⏳ EN PROGRESO (9 PENDING)
- **Próxima tarea:** Task 2 ERR-CORE-02 (clippy quick win, desbloquea verde)
- **Decisiones:** `code()` 10 strings canónicos `VANTADB_` prefix + `Generic` → 2 variantes críticas primero + `clippy deny` solo en `src/` allow tests + `Backtrace` stable + `tracing`/`metrics`/`catch_unwind`

SDP: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail, spec-driven-development, systematic-debugging, code-review-and-quality, security-and-hardening, observability-and-instrumentation, api-and-interface-design, coordinated-web-search
**Code Intelligence por tarea:** `codegraph_explore` + `codebase-memory-mcp: detect_changes` + `get_architecture` + `check_index_coverage` — en cada DO antes de editar
**Internet por tarea:** `coordinated-web-search` → `agent-search free_search` → `argus_search_web` → `metasearchmcp compare_engines` → `webfetch/Jina` (10 fuentes citadas)
