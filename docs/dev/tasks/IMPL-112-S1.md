# IMPL-112-S1: runner local de ingesta (slice mecánico 1 de FIND-112)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-18-cierre-mvp.md` (Task 4, Wave1 primera en secuencia)
- **Fuente:** spec cerrada `docs/dev/tasks/FIND-112.md` (P2-01 approve; trigger cumplido) + plan cierre-mvp Task 4
- **Esfuerzo:** 🟡 1d (appetite 2d) · **Prioridad:** 🟠 Media-Alta
- **Tipo:** feature-add (símbolos `pub` nuevos: `IngestRunnerCfg`, `ConcreteRunner`, `build_ingest_runner`) → Spec obligatoria (§ Spec abajo; contenido = decisiones S1-S6 de FIND-112 citadas, no re-derivadas)
- **Branch:** develop · **Commit:** `feat: IMPL-112-S1 — ...` (NO PUSH, staging selectivo)
- **Creado:** 2026-09-18 · **last-synced:** 2026-09-18
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (spec cerrada; Gate V de S2 = veredicto fricción al cierre)
- **Pendientes (downhill):** 4 steps (slice1 cfg+enum / slice2 wiring / slice3 tests G1 / slice4 verify+commit)
- **NextTask:** FIND-116 (orquestador)

## Tarea

**Objetivo:** construir el `R` local para el `start_ingest<R>` genérico ya existente (`vantadb-mcp/src/wiki.rs:399`): `IngestRunnerCfg` (TOML mínima solo no-secrets + env con precedencia) + enum `ConcreteRunner` por delegación (reusar `LlmRunner`, `StandaloneLlmRunner`, `clamp_llm_concurrency`; `NoLlm` vive), runner construido por llamada (sin global), `wiki_ingest` inputSchema SIN cambios.

**Contrato:** G0 (status limpio + secrets-grep limpio) + G1 (`VANTADB_INGEST_PROVIDER=local` sin modelo → `sources_skipped` == nº fuentes; con fake → páginas + `sources_processed` > 0; `wiki_ingest_status` consultable) + G3 (suites `ingest`+`wiki` verdes, clippy 0, fmt, coverage 0 gaps; inputSchema del tool SIN cambios) + tests 1-6,9-10 verdes.

**AC (verificables por comando):**
- `cargo test -p vanta-memory -j 2 ingest` verde (incluye tests 1-4,9-10 nuevos como unit tests del módulo)
- `cargo test -p vantadb-mcp -j 2 wiki` verde (incluye `wiki_ingest_runner.rs` nuevo con tests 5-6 + assert inputSchema intacto)
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0 warnings
- `cargo fmt --check` limpio
- `scripts/validate-docs-coverage.ps1` 0 gaps
- `rg -n "sk-|api_key\s*=\s*\"[^\"]+" --glob '*.toml'` vacío salvo placeholders (G0 secrets)

## Archivos

**Clave (a crear/editar):**
- `vanta-memory/src/ingest/runner_config.rs` (NUEVO, ~230L: `IngestRunnerCfg` + `IngestRunnerProvider` + `ConcreteRunner` + `build_ingest_runner` + unit tests 1-4,9-10)
- `vanta-memory/src/ingest/mod.rs` (+3L: `pub mod runner_config` + re-export)
- `vanta-memory/Cargo.toml` (+1L: `toml = "0.9"` — reuse locked dep de `vanta-proxy`, cero superficie nueva)
- `vantadb-mcp/src/wiki.rs` (arm `wiki_ingest` ~274-288: construir cfg+runner, pasar a `start_ingest::<ConcreteRunner>`; inputSchema 143-152 INTOCABLE)
- `vantadb-mcp/tests/wiki_ingest_runner.rs` (NUEVO: tests 5-6 + G1 status + inputSchema intacto)

**Relacionados (lectura, NO editar salvo que la spec lo pida):**
- `src/llm.rs` (fábrica `get_embedding_provider` como referencia de precedencia env; NO editar)
- `src/config.rs:919-957` (`LlmCfg` env `VANTADB_LLM_*` a reusar como fallback; NO editar)
- `vanta-memory/src/ingest/merge.rs` (`parse_file_blocks`, `commit` — firma consumida por tests)
- `vantadb-mcp/tests/wiki_async_ingest.rs` (patrón `ScriptedRunner` + `poll_until` a reusar en espíritu)
- `vanta-memory/tests/ingest.rs` (contrato P4 existente — regresión G3)

**Corrección al plan (evidencia, no re-derivación):** el plan cita `vanta-memory/src/wiki.rs:399-409 (punto wiring)` — ese archivo NO EXISTE (verificado: `vanta-memory/src/` no contiene `wiki.rs`). El punto real es `vantadb-mcp/src/wiki.rs:399-409` (`start_ingest<R>` genérico), coincidente con FIND-112 § Archivos (`vantadb-mcp/src/wiki.rs:399-460`). Se sigue spec+código.

**Prohibidos:** cambiar inputSchema del tool (S1/S6) · secrets en TOML/fixtures/docs (G0) · `Box<dyn>` (S1 exige enum) · persistencia nueva · `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation), `C:/Users/Eros/.vantadb*`, `examples/` (FIND-116), `.opencode/skills/` (FIND-119) · WIP ajeno (`SPEC.md`, `completions/*` en `git status` — staging selectivo).

## Dependencias

- **Wave1 primera en secuencia.** Wave0 ✅ ×3 (FIND-98 STOP-lock, FIND-110-spec, FIND-113-spec).
- **Después:** FIND-116 + FIND-119 (disjuntos) → Wave2 IMPL-112-S2 **condicional** a veredicto "S1 sin fricción" (Gate V si fricción).
- **Stop:** si el trait fricciona y exige rediseño → ship parcial documentado + S2 en riesgo (avisar, NO forzar).

## Referencias

- **Rules (leídas completas):** `.opencode/rules/durability.md` (scope storage — esta tarea NO lo toca; leída para no pisarlo) · `api-contract.md` R-8 (lógica en core `vanta-memory`, `vantadb-mcp` solo glue+mapping) + R-3 (default compila y degrada, nunca tool que falla siempre) + R-5 (inputSchema intacto → MCP.md intacto) · `server-mcp.md` (lectura completa; R-2 N/A — hilo sync existente, sin Tokio nuevo).
- **Refs:** `architecture.md` · `definition-of-done.md` (3 niveles) · `clean-code-clean-architecture.md` Ap. V (lógica en core/casos-uso, bindings humble; naming sin stuttering: `ingest::runner_config`, no `ingest::ingest_runner`) · `dev-tools.md` + `test-suite.md` (comandos verify).
- **Commands:** `pipeline.md` (ejecución) · `audit.md` (verify L9/post-tarea).
- **SPEC.md raíz:** § Alcance cierre-mvp (IMPL-112→FIND-112).
- **Notion:** no mapeó (dim memoria sin correspondencia con runner de ingesta wiki; mismo motivo registrado en FIND-112) — omitido con motivo, cero citas.
- **Tabla Spec:** N/A nueva (la trae FIND-112; se cita abajo).

## Skills

- **Cargadas (SDP Paso 0b, `campaign_discover_skills_v2` phase BUILD + base sesión):** `test-driven-development` (RED→GREEN tests 1-6,9-10) · `incremental-implementation` (3 slices verticales) · `context-engineering` (context pack por slice) · `source-driven-development` (todo claim → `file:línea`) · `security-and-hardening` (secrets G0) · `doubt-driven-development` (veredicto fricción S2) · `api-and-interface-design` (contract-first cfg + Hyrum/schema intacto) · `codebase-memory` (blast radius) · `systematic-debugging` (si verify falla).
- **Descartada con motivo:** `frontend-ui-engineering` (SDP lifecycle la sugiere por defecto; esta tarea no toca `web/`).
- `SKILLS_CARGADAS:` test-driven-development, incremental-implementation, context-engineering, source-driven-development, security-and-hardening, doubt-driven-development, api-and-interface-design, codebase-memory, systematic-debugging (+ base campaign-executor/progreso/ponytail).

## Herramientas + MCP

- `codegraph_explore` PRIMERO ✅ (blast radius: `start_ingest` 2 callers en `lib.rs`, `IngestConfig` 11 callers, `LlmRunner` impls Standalone/OpenClaw/Mock+Fixed/Scripted; tests existentes `wiki_async_ingest.rs`, `ingest.rs`)
- `cargo test -p vanta-memory -j 2 ingest` + `cargo test -p vantadb-mcp -j 2 wiki` + clippy/fmt + `validate-docs-coverage.ps1` + fixture markdown en Temp + `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención en RESULTADO). Cargo SIEMPRE `-j 2`.
- Internet SOLO si ambigüedad (no hubo: matriz = reutilizar EMB existente verificado en código; `toml 0.9` ya locked por `vanta-proxy`).

## Investigación código (DISCOVERY — evidencia)

1. **Facade fija (único punto de cambio):** `handle_wiki_tool` ignora `McpConfig` (`_config`, `wiki.rs:180-185`) y llama `start_ingest::<NoLlm>(..., None, IngestConfig::default())` (`wiki.rs:274-281`). El genérico YA existe (`wiki.rs:399-409`: `begin` sync → registra tracker → `thread::spawn(worker::execute)`); S1 solo construye el `R` concreto.
2. **Worker serial + P4 honesto:** `execute` itera fuentes serial (`worker.rs:187-235`); `None`/`NotConfigured` → `sources_skipped.push` (`:214`), nunca hard error; `extract_chunk` con `runner: Option<&R>` (`:312-327`); `commit` merge serial (`:249-301`). `IngestConfig` 3 campos + `clamp_llm_concurrency` 1..=20 default 5 (`mod.rs:58-96`) — a REUSAR, no duplicar (pre-mortem 2).
3. **Fábrica EMB reutilizable (patrón a seguir):** `get_embedding_provider()` (`src/llm.rs` — match sobre `Config::default().llm_cfg()` con fallback `LocalOnnxProvider::new_dummy`); `LlmCfg` env `VANTADB_LLM_URL`/`VANTADB_LLM_MODEL`/`VANTADB_LLM_SUMMARIZE_MODEL`/`VANTADB_LOCAL_MODEL`/`VANTADB_OPENAI_API_KEY` (solo presencia, nunca log en claro) — `IngestRunnerCfg` delega con override `VANTADB_INGEST_*`.
4. **Runners chat existentes:** `StandaloneLlmRunner { LlmConfig { base_url, api_key, model, max_tokens, timeout } }` (`adapters/standalone/llm_runner.rs:23-64`); sin `llm-driver` su `run` = `NotConfigured` (`:104-118`) — el mismo degradado P4 que necesita la matriz; construcción pura (sin I/O) → runner por llamada es barato (S3).
5. **`MockLlmRunner::fixed` (feature `mock`)** + `ScriptedRunner` en `tests/ingest.rs:42-79` y `wiki_async_ingest.rs:28-37` — patrón FIFO para test 6 (bloques `parse_file_blocks`-válidos vía `file_block()` helper idéntico).
6. **Config TOML dep:** `toml = "0.9"` ya locked (dependencia de `vanta-proxy` en `Cargo.lock`) → añadir a `vanta-memory` reusa versión, cero superficie supply-chain nueva (deny MIT/Apache-2.0 OK).
7. **Lints:** workspace `unwrap_used`/`expect_used` = deny en prod; tests con `#![allow]` propio (precedente `wiki_async_ingest.rs:2`, `ingest.rs:2`).

## Investigación problema (tradeoffs — decididos por FIND-112, no re-derivar)

- **Wiring = único código nuevo:** `build_ingest_runner(&cfg) -> Option<ConcreteRunner>` + enum por delegación (S1); el worker NO se toca (absorbe vía `LlmRunner::run` + degradado existente).
- **G1 observable:** default (sin env/TOML/modelo) → `None` → path idéntico al actual (`None` literal hoy) ≡ `NoLlm` bit a bit; con fake → `sources_processed > 0` + páginas legibles + `ingest_status` consultable.
- **McpConfig:** S1 NO añade knobs (teatro); fuente cfg = env+TOML server-side vía `storage.data_dir`. Desviación de la letra ("deja de ignorar McpConfig") documentada en § Spec S7 con motivo scope-discipline.

## Investigación internet

No usada — sin ambigüedad (diseño = reutilizar EMB/runners existentes verificados en código; `toml 0.9` con precedente interno `vanta-proxy`). Sin deuda TSYS-13.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/lib.rs` (2 callers de `start_ingest` vía `handle_tools_call`), tools `wiki_ingest`/`wiki_ingest_status` (JSON-RPC, schema INTACTO) |
| Callees | `vanta-memory::ingest::{worker, IngestConfig, clamp_llm_concurrency}`, `core::abstractions::{LlmRunner, LlmError}`, `adapters::standalone::{StandaloneLlmRunner, LlmConfig}`, `vantadb::wiki::WikiStore`, `toml 0.9` (locked) |
| Implicaciones | contrato público: inputSchema SIN cambios (R-5 docs intactos); comportamiento default idéntico (None path); sin performance/memoria/serialización nueva; sin migración; tests existentes como regresión G3 |
| Riesgo | bajo-medio (facade + módulo nuevo aislado; worker intacto) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/wiki.rs` (656L: tools 50-172, handler 180-300, facade 363-460) · `vanta-memory/src/ingest/worker.rs` (339L) · `vanta-memory/src/ingest/mod.rs` (236L) · `vanta-memory/src/core/abstractions/llm_runner.rs` (248L) · `vanta-memory/src/adapters/mock.rs` + `adapters/standalone/llm_runner.rs` (220L) · `vantadb-mcp/src/config.rs` (142L) · `vanta-memory/tests/ingest.rs` (598L) · `vantadb-mcp/tests/wiki_async_ingest.rs` (194L) · rules `durability.md`+`api-contract.md`+`server-mcp.md` · refs `definition-of-done.md` + `clean-code-clean-architecture.md` Ap. V
- **Referenciados hacia dentro:** `runner_config.rs` (nuevo) → `core::abstractions::{LlmRunner,LlmError,LlmRunParams}`, `ingest::{IngestConfig,clamp_llm_concurrency}`, `adapters::standalone::{StandaloneLlmRunner,LlmConfig}`, `toml`, `serde`; `wiki.rs` arm → `ingest::runner_config::{IngestRunnerCfg,ConcreteRunner}`
- **Referencias entrantes:** `start_ingest` ← `vantadb-mcp/src/lib.rs` (2 callers, firma intacta); `NoLlm` ← tests + arm actual (vive); `IngestConfig::default()` ← arm actual (se preserva como fallback)
- **Veredicto impacto:** BAJO — 1 módulo nuevo aislado + 1 arm de handler + 1 test file nuevo + 2 wiring de 1-3L (`mod.rs`, `Cargo.toml`); worker/core intactos; schema MCP intacto.

## Contrato

`cargo test -p vanta-memory -j 2 ingest` ✅ + `cargo test -p vantadb-mcp -j 2 wiki` ✅ (tests 1-6,9-10 verdes) + clippy 0 + fmt + coverage 0 gaps + secrets-grep limpio + inputSchema `wiki_ingest` byte-idéntico + default local-sin-modelo ≡ `NoLlm` (G1) + `wiki_ingest_status` consultable.

## Spec (Phase 1b feature-add — contenido = FIND-112 S1-S6 citadas + S7 local)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| S1 | Reutilizar `LlmRunner`, enum `ConcreteRunner` por delegación | Nuevo trait / `Box<dyn>` (object-safety + `Send+Sync` extra, sin consumidor distinto) | ✅ decidido-por-evidencia (FIND-112:283; genérico `start_ingest<R>` monomorfiza mejor con enum) |
| S2 | TOML mínima solo no-secrets + env con precedencia | Todo-env / todo-TOML | ✅ decidido-por-evidencia (FIND-112:284; `toml 0.9` ya locked vía `vanta-proxy`; `api_key` NUNCA leída de TOML — test 4) |
| S3 | Runner por llamada, sin global | `OnceLock` global (secrets en static + env congelado) | ✅ decidido-por-evidencia (FIND-112:285; construcción = struct con Strings, `reqwest::blocking::Client` por llamada como `run_http`) |
| S4 | `NoLlm` vive (variante `ConcreteRunner::None` explícita + struct en facade) | Borrar al shippear | ✅ decidido-por-evidencia (FIND-112:286; default cero-red testeable + runner de tests P4) |
| S5 | Etapas, local primero (este slice) | Big-bang ×3 | ✅ plan (este task = slice 1; ollama/openai = S2 con tests 7-8) |
| S6 | `wiki_ingest` inputSchema SIN cambios | Añadir `provider` al tool input (Hyrum: cada campo es contrato para siempre) | ✅ decidido-por-evidencia (FIND-112:288; config server-side operador, no per-call del LLM) |
| S7 | S1 no añade knobs a `McpConfig` (fuente = env+TOML vía `storage.data_dir`) | Mapeo teatro `McpConfig→cfg` sin campos | ✅ decidido-por-evidencia (local: `config.rs` no tiene knobs ingest; añadir struct vacía viola scope-discipline; la intención —cfg server-side en vez de `None` fijo— se cumple) |

Precedencia: `VANTADB_INGEST_*` env > TOML > `VANTADB_LLM_*` heredadas > defaults. Validación en frontera: desconocido/clamps/TOML ilegible → `warn!` + default seguro, nunca `unwrap`.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** inputSchema `wiki_ingest`/`wiki_ingest_status` byte-idéntico · secrets solo env (grep G0) · `NoLlm` exportado y funcional · default ≡ P4 actual (`sources_skipped`, nunca hard error) · `STRUCTURAL_FILES` + `locked` intactos · WIP ajeno intocable (staging selectivo)
- **Comandos de verificación:** `cargo test -p vanta-memory -j 2 ingest` · `cargo test -p vantadb-mcp -j 2 wiki` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo fmt --check` · `pwsh scripts/validate-docs-coverage.ps1`
- **Deuda pendiente:** P2-01 `vanta-review` lo hace el ORQUESTADOR (esta ejecución no auto-revisa) · S2 (tests 7-8, G2) condicional al veredicto fricción

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** Sin deuda nueva. `ConcreteRunner::Local` sin modelo → `NotConfigured` documentado (upgrade path = chat local real futuro, NO deuda silenciosa — es el contrato §(e) de la spec). `toml` dep ya locked (no superficie nueva).

## Definition of Done (3 niveles)

| Nivel | Gate |
|-------|------|
| Task | G0+G1+G3 + tests 1-6,9-10 + task file + recitation |
| Commit | atómico `feat:`, staging selectivo (5 paths), verify mecánico previo (nunca auto-reporte) |
| Release | N/A (feature tras flag natural = default P4; shippable = default intacto + opt-in por config) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — `security-and-hardening` cargada. Trust boundary: TOML operada + env. Hallazgos: (1) `api_key` NUNCA de TOML (test 4 lo fija); (2) key vive solo en stack del thread de ingesta (S3, sin global); (3) G0 secrets-grep en verify; (4) sin `cargo audit` (dependencia `toml` ya locked, sin bump). Sin auth/input-usuario nuevo.
- [x] **PERFORMANCE** — N/A con motivo: worker serial intacto (sin `dashmap`/`parking_lot`/Tokio nuevo → Regla 8 no dispara); construcción cfg es O(1) por llamada fuera del hot path. Sin baseline (nada que medir).

## Steps

### Step 1: Slice 1 — `IngestRunnerCfg` + `ConcreteRunner` en core (TDD RED→GREEN)
- **Archivos:** `vanta-memory/src/ingest/runner_config.rs` (NUEVO), `vanta-memory/src/ingest/mod.rs` (+3L), `vanta-memory/Cargo.toml` (+1L `toml`)
- **Acción:** RED: módulo con tests 1-4,9-10 + stubs → `cargo test` falla. GREEN: `IngestRunnerProvider` (local/ollama/openai) + `IngestRunnerCfg` (defaults/from_toml_str/apply_env/from_env_toml/validate/pipeline_config/build_runner) + `ConcreteRunner` enum (Local/None → NotConfigured; Ollama/OpenAi → StandaloneLlmRunner por delegación) + `build_ingest_runner`. Precedencia env>TOML>heredadas>defaults; clamps vía `clamp_llm_concurrency`; timeout 5..=600, max_tokens 1..=8000.
- **Verify:** `cargo test -p vanta-memory -j 2 ingest_runner` (unit) + `cargo clippy -p vanta-memory --all-targets -- -D warnings`
- **Estado:** ✅ COMPLETED (RED: 5 fallan en `todo!`; GREEN: 6/6; clippy 0; fmt)

### Step 2: Slice 2 — wiring `wiki_ingest` (constructor por llamada, schema intacto)
- **Archivos:** `vantadb-mcp/src/wiki.rs` (arm `wiki_ingest` ~274-288)
- **Acción:** construir `IngestRunnerCfg::from_env_toml(toml_path)` (path = `VANTADB_INGEST_CONFIG` o `storage.data_dir/vanta-ingest.toml`) → `build_ingest_runner` → `start_ingest::<ConcreteRunner>(..., runner, pipeline_cfg)`. Default sin config → `None` (path idéntico al actual). NO tocar `wiki_tool_definitions` ni `McpConfig`.
- **Verify:** `cargo check -p vantadb-mcp -j 2` + diff del inputSchema (grep `wiki_ingest` schema byte-idéntico)
- **Estado:** ✅ COMPLETED (check verde; schema intacto fijado en test S6)

### Step 3: Slice 3 — tests G1 (`wiki_ingest_runner.rs`: tests 5-6 + status + schema)
- **Archivos:** `vantadb-mcp/tests/wiki_ingest_runner.rs` (NUEVO)
- **Acción:** test 5 `ingest_local_no_model_degrades` (Temp md ×2 sin env → `sources_skipped==2`, `ready`, status consultable por run_id) + test 6 `ingest_local_canned_runner_writes_pages` (fake `LlmRunner` con `file_block` válido → `sources_processed>0` + `wiki_read` legible) + assert `tools/list` inputSchema `wiki_ingest` sin `provider` (S6 fijado en test). Patrón `poll_until` + `ScriptedRunner` de `wiki_async_ingest.rs`.
- **Verify:** `cargo test -p vantadb-mcp -j 2 --test wiki_ingest_runner`
- **Estado:** ✅ COMPLETED (3/3: degradado skipped==2 + canned escribe + schema)

### Step 4: Verify full + commit + cierre
- **Archivos:** ninguno (verificación + git)
- **Acción:** G0 (`git status` solo 5 paths + secrets-grep) + G3 (clippy workspace + fmt + coverage ps1) + full suites + `campaign_update_task_state(completed)` + commit `feat:` staging selectivo + `skill progreso` + RESULTADO §7 con veredicto fricción S2 explícito
- **Verify:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo fmt --check` + `cargo nextest run -p vanta-memory -p vantadb-mcp -j 2` (o `cargo test` scopes si nextest falla por env) + `pwsh scripts/validate-docs-coverage.ps1`
- **Estado:** ✅ COMPLETED (342 lib-mem + 15 ingest + 27 lib-mcp + 3+3+7+1 wiki; clippy 0; fmt; coverage 0 gaps; secrets limpio; OCR sin Critical/High)

## Dependencias

- FIND-112 ✅ (spec cerrada + P2-01 approve — trigger) · Wave0 ✅ ×3 · NextTask: FIND-116

## Review (GATE — agente distinto, P2-01)

- **Revisor:** orquestador asigna (`vanta-review` batch al cierre por área)
- **Enfoque:** ¿enum-por-delegación absorbe sin fricción? (veredicto Gate V de S2) ¿cfg server-side sin knobs McpConfig aceptable (S7)?
- **Cómo se probó:** tests 1-6,9-10 + G1 observable + clippy/fmt/coverage (evidencia en recitation, nunca auto-reporte)
- **Checklist anti-hábitos tóxicos:** a verificar por el revisor
- **Veredicto:** pendiente (lo registra el orquestador)

## Notas

- Gate D: NO disparado (blast radius 3 archivos escritura + 2 wiring-1L; contrato cerrado por spec P2-01; feature-add CON spec válida S1-S6+S7).
- Gate V: no disparado en discovery (cero verify mecánico fallido). Umbral 2-fallas-mismo-error → question.
- `campaign_verify_cmd` bug exit -1 conocido → bash directa + mención en RESULTADO.
- `frontend-ui-engineering` descartada (sin `web/`).

## Recitation (canónica)

- activeGoal: IMPL-112-S1 runner local de ingesta
- lastAction: 4/4 slices ✅ + verify full G0/G1/G3 + commit (ver COMMIT_HASH en RESULTADO)
- result: OK
- nextAction: orquestador — P2-01 vanta-review (enfoque: fricción S2 + S7) + skill progreso + recitation plan
- contract: G0 (status 7 paths tarea + secrets-grep vacío) + G1 (skipped==2 sin modelo; canned escribe + legible; status consultable) + G3 (342+15+27+3+3+7+1 verdes; clippy 0; fmt; coverage 0 gaps; inputSchema intacto fijado en test); evidencia: comandos Step 1-4 en este file + `git show --stat HEAD`; artefactos: 7 paths (5 código + lock-edge + task file); invariantes: § Invariantes (schema/secretos/NoLlm/P4/STRUCTURAL/WIP); deuda: P2-01 pendiente (orquestador) + S2 condicional; queda_pendiente: Gate V S2 según veredicto fricción (abajo)
- **Veredicto fricción S2 (Gate V): SIN FRICCIÓN** — el genérico `start_ingest<R>` absorbió `ConcreteRunner` y el fake `ScriptedRunner` sin tocar el worker; `StandaloneLlmRunner` ya degrada sin `llm-driver`; construcción = mapping puro de config. S2 puede arrancar directo (tests 7-8 + G2).
- nextTask: FIND-116
