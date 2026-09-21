# IMPL-112-S2: runners ollama/openai de ingesta (slice mecánico 2 de FIND-112)

## Metadata

- **Plan file:** `docs/plans/2026-09-18-cierre-mvp.md` (Task 7, Wave2 primera en secuencia)
- **Fuente:** spec cerrada `docs/tasks/FIND-112.md` (P2-01 approve; § `llm-driver` + G2 + tests 7-8 nombrados) + slice S1 `1dd9019c` (`docs/tasks/IMPL-112-S1.md`)
- **Esfuerzo:** 🟡 1d (appetite 2d) · **Prioridad:** 🟡 Media
- **Tipo:** test-add sobre feature existente (0 símbolos `pub` nuevos — las variantes ya viven en S1; ver Hallazgo DISCOVERY) → Spec = FIND-112 S1-S6 citadas, sin tabla nueva
- **Branch:** develop · **Commit:** `feat: IMPL-112-S2 — ...` (NO PUSH, staging selectivo solo paths de esta tarea)
- **Creado:** 2026-09-18 · **last-synced:** 2026-09-18
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 (gate de entrada CUMPLIDO — veredicto S1 "SIN FRICCIÓN"; veredicto propio idéntico abajo)
- **Pendientes (downhill):** 3 steps (test 7 ollama / test 8 openai / verify full + commit)
- **NextTask:** FIND-117 (la ejecuta el orquestador, no esta ejecución)

## Tarea

**Objetivo:** completar la matriz owner local+ollama/openai (FIND-112 §(a)): G2 observable en ambas variantes — sin servidor/key degrada P4 con el mismo assert que G1 (`sources_skipped` == nº fuentes, `state` consultable, nunca hard error); con mock/runner canned extract+merge escriben — + tests 7-8 verdes + G3 + G0.

**Contrato:** G2 (sin servidor/key → degrada P4 mismo assert G1; con mock → extract+merge escriben) + tests 7-8 (`ingest_ollama_down_degrades`, `ingest_openai_no_key_degrades`) verdes + G3 (suites `ingest`+`wiki` verdes, clippy 0, fmt, coverage 0 gaps; inputSchema del tool SIN cambios) + G0 (status limpio + secrets-grep limpio, secrets solo env).

**AC (verificables por comando):**

- `cargo test -p vanta-memory -j 2 ingest` verde (regresión S1: tests 1-4,9-10 intactos)
- `cargo test -p vantadb-mcp -j 2 --test wiki_ingest_runner` verde (tests 5-6 S1 + 7-8 nuevos + schema intacto)
- `cargo test -p vantadb-mcp -j 2 wiki` verde (toda la familia wiki, sin regresión)
- `cargo clippy --workspace --all-targets --all-features -j 2 -- -D warnings` 0 warnings
- `cargo fmt --check` limpio
- `pwsh scripts/validate-docs-coverage.ps1` 0 gaps
- `rg -n "sk-|api_key\s*=\s*\"[^\"]+" --glob '*.toml' --glob '*.md'` vacío salvo placeholders (G0 secrets; en este slice: ningún secreto nuevo en ningún archivo)

## Archivos

**Clave (a crear/editar):**

- `vantadb-mcp/tests/wiki_ingest_runner.rs` (APPEND tests 7-8, ~100L: `ingest_ollama_down_degrades` + `ingest_openai_no_key_degrades`, cada uno con mitad degrada-P4 + mitad mock-escribe vía `ScriptedRunner` + `pipeline_config()` de la variante)
- `vanta-memory/src/ingest/runner_config.rs` (LECTURA + posible micro-ajuste SOLO si un test revela gap; por defecto 0 cambios — variantes ya implementadas en S1: `IngestRunnerProvider::Ollama/OpenAi` `:33-44`, `ConcreteRunner::Ollama/OpenAi/None` `:90-96`, `build_runner` `:259-270`, `standalone_cfg` `:274-284`, `apply_env` defaults por provider `:197-220`)
- `vantadb-mcp/src/wiki.rs` (LECTURA — wiring por llamada ya hecho en S1 `:280-296`; 0 cambios salvo gap)

**Relacionados (lectura, callers/callees de codegraph_explore):**

- `docs/tasks/FIND-112.md` (spec: §(a) tabla variante×transporte/config/secret/test `:151-163`, §(b) TOML+env `:165-207`, §(c) lifecycle dueño por llamada `:209-240`, §(d) gates G2/G3 + tests 7-8 `:242-264`, §(e) degradado declarado `:265-275`, decisiones S1-S6 `:279-288`)
- `docs/tasks/IMPL-112-S1.md` (base + veredicto fricción `:206` + S7 desviación documentada `:127`)
- `vanta-memory/src/ingest/worker.rs:187-235` (serial + P4: `None`/`NotConfigured` → `sources_skipped` `:201-204`; CUALQUIER `LlmError` → warn+skip `:206-211` — por eso ollama-down degrada igual sin `llm-driver`; serial, NO tocar)
- `vanta-memory/src/ingest/worker.rs:311-327` (`extract_chunk` con `runner: Option<&R>` — firma consumida por tests)
- `vanta-memory/src/adapters/standalone/llm_runner.rs:104-118` (sin `llm-driver` → `NotConfigured`; con `llm-driver` → `run_http` con `Transport`/`Timeout`/`Http` — ambos degradan en worker; referencia, NO editar)
- `vanta-memory/src/ingest/mod.rs` (`IngestConfig` + `clamp_llm_concurrency` — reuso, NO editar)
- `vantadb-mcp/tests/wiki_async_ingest.rs` (patrón `ScriptedRunner` + `poll_until` a reusar en espíritu)
- `vanta-memory/tests/ingest.rs` (contrato P4 existente — regresión G3)
- `src/llm.rs` (fábrica EMB `get_embedding_provider` como referencia de precedencia env; NO editar)

**Prohibidos:** secrets en TOML/fixtures/docs/código (G0 — keys solo env, mocks con puerto cerrado `127.0.0.1:9`, nunca key real ni siquiera placeholder `sk-...` en archivos nuevos) · cambiar inputSchema del tool (S6) · `Box<dyn>` (S1 exige enum) · persistencia nueva · `reparacion.bat` · `.opencode` (incluye `.opencode/skills/` de FIND-119) · `Justfile` · `ocr-*` · `completions/*` · `desktop/src-tauri/Cargo.lock` · stash@{0} GOV-C4 · `docs/Backlog.md` · plan file (solo recitation al cierre) · `C:/Users/Eros/.vantadb*` (datos vivos) · `examples/` (FIND-116 ✅) · `SPEC.md` (edit del lead sin commitear) · WIP ajeno en `git status` (staging selectivo: SOLO `vantadb-mcp/tests/wiki_ingest_runner.rs` + este task file + micro-ajuste si lo hubiera).

## Dependencias

- **Wave2 primera en secuencia.** Wave0 ✅ ×3 (FIND-98 STOP-lock, FIND-110-spec, FIND-113-spec). Wave1 ✅ ×3 (IMPL-112-S1 `1dd9019c`, FIND-116 `08b7ac8d`, FIND-119 `83ff5ce0`).
- **Gate de entrada CUMPLIDO:** S1 veredicto "SIN FRICCIÓN" (`IMPL-112-S1.md:206` — `start_ingest<R>` absorbió `ConcreteRunner` sin tocar worker; `StandaloneLlmRunner` ya degrada sin `llm-driver`; construcción = mapping puro) → GO directo, sin Gate V.
- **Si se encontrara fricción real que S1 no vio** (p.ej. `Transport` no degradara, o el enum exigiera rediseño) → STOP + Gate V (re-triage, no forzar).
- **Después (orquestador, no esta ejecución):** FIND-117 + SHOW-05 (disjuntos, secuencial por rate-limit).
- **Stop:** appetite >2d → ship una variante + nota (no aplica: 0 código prod esperado, solo tests).

## Referencias

- **Rules (leídas completas antes de codificar):** `.opencode/rules/durability.md` (scope storage — esta tarea NO lo toca; leída para no pisarlo) · `.opencode/rules/server-mcp.md` (R-1 schema/tools sync — inputSchema intacto; R-2 N/A — hilo sync existente, sin Tokio nuevo; R-3 N/A — sin métricas nuevas).
- **Refs:** `.opencode/references/architecture.md` (workspace: `vantadb-mcp` glue + `vanta-memory` core) · `definition-of-done.md` (3 niveles + capa determinista clippy/fmt/nextest) · `clean-code-clean-architecture.md` Ap. V (lógica en core, bindings humble; naming sin stuttering — sin símbolos nuevos, N/A activo) · `dev-tools.md` + `test-suite.md` (comandos verify: `cargo nextest`, `cargo test --doc`, clippy/fmt).
- **Commands:** `pipeline.md` (ejecución vía `pipeline-full.md`) · `audit.md` (verify L9/post-tarea).
- **SPEC.md raíz:** § Alcance cierre-mvp (IMPL-112→spec FIND-112).
- **Notion:** no mapeó (dim memoria sin correspondencia con runner de ingesta wiki; mismo motivo registrado en FIND-112 § Referencias y S1 § Referencias) — omitido con motivo, cero citas.
- **Tabla Spec:** N/A nueva (la trae FIND-112 § Spec (a)-(e) + decisiones S1-S6; se cita, no se re-deriva).

## Skills

- **SDP real (pipeline-full Paso 0b, `campaign_discover_skills_v2` phase BUILD + contractKeywords [ingest, runner, ollama, openai, degrade, secrets, mock], ≤8):** base `campaign-executor`, `source-driven-development` + lifecycle `incremental-implementation`, `test-driven-development`, `context-engineering`, `doubt-driven-development`, `api-and-interface-design` (+ `security-and-hardening` por tipo MCP/security-sensitive, `systematic-debugging` por plan).
- **Descartada con motivo:** `frontend-ui-engineering` (el lifecycle la sugiere por defecto; esta tarea no toca `web/` — mismo descarte que S1).
- `SKILLS_CARGADAS:` test-driven-development, incremental-implementation, context-engineering, source-driven-development, security-and-hardening, doubt-driven-development, api-and-interface-design, codebase-memory, systematic-debugging (+ base campaign-executor/progreso/ponytail).

**Cuándo aplica c/u:**

- `test-driven-development` — RED→GREEN tests 7-8 (núcleo del slice: tests primero, fallan sin existir, pasan con la implementación que ya vive en S1)
- `security-and-hardening` — secrets G0 (key solo env, nunca TOML/fixtures; mock puerto cerrado, nunca red real ni key real)
- `systematic-debugging` — si un verify falla (root cause antes de reintentar; umbral 2-fallas-mismo-error → Gate V)
- `incremental-implementation` — 1 test por step (~50L cada uno), repo siempre verde entre steps
- `context-engineering` — context pack por slice (spec FIND-112 + S1 como base + ejemplo test 5-6 como patrón)
- `source-driven-development` — todo claim → `file:línea` real (cero APIs inventadas)
- `doubt-driven-development` — veredicto fricción propia al cierre (decide si S2 confirma el GO de S1)
- `api-and-interface-design` — contract-first cfg + Hyrum/schema intacto (S6 fijado en test existente)
- `codebase-memory` — blast radius (ya corrido en DISCOVERY)

## Herramientas + MCP

- `codegraph_explore` PRIMERO ✅ (blast radius: `build_ingest_runner` 2 callers en `wiki.rs`; `ConcreteRunner`/`IngestRunnerCfg` intra-módulo; `start_ingest` 3 callers en `lib.rs`; tests S1 existentes)
- `cargo test -p vanta-memory -j 2 ingest` + `cargo test -p vantadb-mcp -j 2 wiki` + clippy/fmt + `validate-docs-coverage.ps1` + mock puerto cerrado determinista (`127.0.0.1:9`, nunca timing) + `campaign_verify_cmd` (BUG exit -1 conocido → bash directa + mención en RESULTADO). Cargo SIEMPRE `-j 2`.
- Internet SOLO si ambigüedad (por defecto N/A — spec + S1 alcanzan; sin red → `[cita NO VERIFICADA]` + deuda TSYS-13).

## Investigación código (DISCOVERY — evidencia)

1. **HALLAZGO (código contradice la letra del plan, se sigue al código):** el plan pide "extender: variantes" en `runner_config.rs`, pero S1 YA implementó las 3 variantes completas — `IngestRunnerProvider::{Local,Ollama,OpenAi}` (`runner_config.rs:33-44`), `ConcreteRunner::{Ollama(OpenAi),None}` (`:90-96`), `build_runner` con rama openai-sin-key → `None` (`:259-270`), `standalone_cfg` compartido (`:274-284`), `apply_env` con defaults por provider (`:197-220`), y el wiring por llamada en `wiki.rs:280-296`. S2 es por tanto **tests + verificación G2**, 0 código prod esperado (micro-ajuste solo si un test revela gap real).
2. **Por qué el degrade es feature-agnóstico (base del G2 "mismo assert G1"):** `worker.rs:199-211` trata CUALQUIER `LlmError` igual (warn + skip, nunca hard error). Sin `llm-driver` el `Ollama`/`OpenAi` inner devuelve `NotConfigured` (`llm_runner.rs:104-118`); con `llm-driver` y puerto cerrado devuelve `Transport` — ambos caen en el mismo `sources_skipped`. El test aserta el OBSERVABLE (`skipped`, `ready`), nunca la variante del error → verde con y sin la feature. `vantadb-mcp` depende de `vanta-memory` SIN features (verificado en `vantadb-mcp/Cargo.toml`), luego sus tests corren LLM-free: el assert portable es el degrade, no `Transport`.
3. **Facade intacta:** `start_ingest<R: LlmRunner + Send + Sync + 'static>` (`wiki.rs:420-481`) absorbe `ConcreteRunner` por el `impl LlmRunner` por delegación (`runner_config.rs:98-105`); `ingest_status` por `run_id` (`wiki.rs:485-505`) no distingue variantes. `wiki_tool_definitions` `wiki_ingest` inputSchema (`wiki.rs:145-153`) INTOCABLE (S6, fijado por test existente `ingest_tool_input_schema_unchanged`).
4. **Patrón de test a seguir:** `wiki_ingest_runner.rs` tests 5-6 (S1): fixture markdown en Temp ×N → nivel worker (`worker::run` retorna report → assert `sources_*`) + nivel facade (`start_ingest` + `poll_ready` → assert `state`/`namespace`/`slug`). Tests 7-8 replican el patrón con cfg de variante + puerto cerrado / sin key; la mitad mock-escribe usa `ScriptedRunner` + `pipeline_config()` de la variante (prueba que extract+merge escriben bajo cfg ollama/openai sin necesidad de HTTP real).
5. **Lints:** workspace `unwrap_used`/`expect_used` = deny en prod; el test file tiene `#![allow]` propio en cabecera (precedente S1 `:2`) — los tests nuevos heredan el allow, sin `allow` nuevo.

## Investigación problema (tradeoffs — decididos por FIND-112, no re-derivar)

- **Degrada-P4 observable en ambas variantes:** mismo assert G1 (`sources_skipped == nº fuentes`, `IngestReport` OK, `state=ready` consultable) porque el worker no ramifica por variante (R-8: el worker no decide, solo degrada). Sin servidor (ollama, puerto cerrado) y sin key (openai, env ausente) son los dos caminos P4 de la tabla §(a).
- **Mocks deterministas:** puerto cerrado `127.0.0.1:9` (discard, connection-refused inmediato, sin timing ni flake) + `timeout_secs` explícito corto (clamp MIN 5) por higiene + `ScriptedRunner` FIFO con bloques `file_block()`-válidos (patrón S1/test 6). NUNCA key real, NUNCA red externa, NUNCA `sleep` fijo.
- **Secrets-solo-env:** el test openai aserta `openai_api_key == None` sin env (construcción pura, sin mutar `std::env` global — cfg inyectada vía `from_toml_str` + `apply_env(|_| None)`); test 4 S1 ya fija que TOML `api_key` se ignora. G0 grep al cierre.

## Investigación internet

No usada — sin ambigüedad (diseño = reutilizar S1 + runners existentes verificados en código; `127.0.0.1:9` es convención discard, no requiere cita). Sin deuda TSYS-13.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/lib.rs` (3 callers de `start_ingest` vía `handle_tools_call`, firma intacta); tools `wiki_ingest`/`wiki_ingest_status` (JSON-RPC, schema INTACTO) |
| Callees | `vanta-memory::ingest::{worker, runner_config::{IngestRunnerCfg,ConcreteRunner,build_ingest_runner}}`, `core::abstractions::{LlmRunner,LlmError}`, `adapters::standalone::{StandaloneLlmRunner,LlmConfig}`, `WikiStore` |
| Escritura real | 1 archivo: `vantadb-mcp/tests/wiki_ingest_runner.rs` (APPEND ~100L); 0 archivos prod (salvo gap revelado por test) |
| Implicaciones | contrato público: inputSchema SIN cambios; comportamiento default idéntico; sin performance/memoria/serialización nueva; sin migración; tests S1 (1-6,9-10) como regresión G3 |
| Riesgo | bajo (solo tests + wiring ya probado por S1) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/ingest/runner_config.rs` (420L) · `vantadb-mcp/src/wiki.rs` (677L) · `vantadb-mcp/tests/wiki_ingest_runner.rs` (231L) · `vanta-memory/src/adapters/standalone/llm_runner.rs` (220L) · `vanta-memory/src/ingest/worker.rs` (`:187-246` + `:311-327`) · `vanta-memory/src/ingest/mod.rs` (237L) · spec `docs/tasks/FIND-112.md` (completa) · S1 `docs/tasks/IMPL-112-S1.md` (completo) · rules `durability.md`+`server-mcp.md` · refs `definition-of-done.md` + `architecture.md` + `clean-code-clean-architecture.md` (cabecera + Ap. V por referencia S1)
- **Referenciados hacia dentro:** tests 7-8 → `ingest::runner_config::{IngestRunnerCfg,ConcreteRunner,build_ingest_runner}`, `ingest::{worker,IngestConfig}`, `core::abstractions::{LlmError,LlmRunner,LlmRunParams}`, `start_ingest/ingest_status` facade
- **Referencias entrantes:** ninguna nueva (tests son hojas; 0 símbolos `pub` nuevos)
- **Veredicto impacto:** MÍNIMO — append a 1 test file + 0 prod; worker/core/facade/schema intactos.

## Contrato

`cargo test -p vanta-memory -j 2 ingest` ✅ + `cargo test -p vantadb-mcp -j 2 wiki` ✅ (tests 7-8 verdes, 1-6,9-10 sin regresión) + clippy 0 + fmt + coverage 0 gaps + secrets-grep limpio + inputSchema `wiki_ingest` byte-idéntico + degrada-P4 en ambas variantes con mismo assert G1 + mock-escribe bajo cfg de variante.

## Spec (feature-add sin símbolos nuevos — contenido = FIND-112 S1-S6 citadas + delta S2)

| # | Decisión | Estado en S2 |
|---|----------|--------------|
| S1 | Enum `ConcreteRunner` por delegación, NO `Box<dyn>` | ✅ heredado-verificado (S1 `:90-105`; S2 no lo toca) |
| S2 | TOML mínima solo no-secrets + env con precedencia (`VANTADB_INGEST_*` > TOML > `VANTADB_LLM_*` > defaults) | ✅ heredado-verificado (S1 `:129-237`; tests 7-8 la ejercitan por variante) |
| S3 | Runner por llamada, sin global | ✅ heredado-verificado (`wiki.rs:280-296`; tests 7-8 usan el mismo camino) |
| S4 | `NoLlm` vive (variante `ConcreteRunner::None` + struct facade) | ✅ heredado-verificado (openai-sin-key → `None`, test 8 lo fija) |
| S5 | Etapas, local primero (S1) → ollama/openai (este slice, tests 7-8, G2) | ✅ este slice |
| S6 | `wiki_ingest` inputSchema SIN cambios | ✅ fijado por test S1 existente (regresión G3) |
| S7 | Sin knobs `McpConfig` (fuente = env+TOML vía `storage.data_dir`) | ✅ heredado (S1 § Spec S7; S2 no lo reabre) |

Precedencia y validación en frontera: ver S1 § Spec (desconocido/clamps/TOML ilegible → `warn!` + default seguro, nunca `unwrap`).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** inputSchema `wiki_ingest`/`wiki_ingest_status` byte-idéntico · secrets solo env (grep G0; ningún secreto nuevo en ningún archivo) · `NoLlm` exportado y funcional · default ≡ P4 actual (`sources_skipped`, nunca hard error) · `STRUCTURAL_FILES` + `locked` intactos · WIP ajeno intocable (staging selectivo)
- **Comandos de verificación:** `cargo test -p vanta-memory -j 2 ingest` · `cargo test -p vantadb-mcp -j 2 wiki` · `cargo clippy --workspace --all-targets --all-features -j 2 -- -D warnings` · `cargo fmt --check` · `pwsh scripts/validate-docs-coverage.ps1`
- **Deuda pendiente:** P2-01 `vanta-review` lo hace el ORQUESTADOR (esta ejecución no auto-revisa) · veredicto fricción propia al cierre (confirma o revoca el GO de S1)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** Sin deuda nueva. 0 código prod (solo tests). `127.0.0.1:9` como puerto-cerrado canónico queda documentado en el test (upgrade path: si algún entorno bindea discard, cambiar a puerto efímero cerrado vía `TcpListener` bind+drop — nota `ponytail:` en el test). Sin dependencias nuevas, sin flags nuevos.

## Definition of Done (3 niveles)

| Nivel | Gate |
|-------|------|
| Task | G0+G2+G3 + tests 7-8 + task file + recitation |
| Commit | atómico `feat:`, staging selectivo (2 paths: test file + task file; + micro-ajuste solo si hubo gap), verify mecánico previo (nunca auto-reporte) |
| Release | N/A (tras flag natural = default P4; shippable = default intacto + opt-in por config) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — `security-and-hardening` cargada. Trust boundary: TOML operada + env (igual que S1; sin superficie nueva). Checklist: (1) `api_key` NUNCA de TOML (test 4 S1 lo fija; test 8 reafirma `None` sin env) ✅; (2) key vive solo en stack del thread de ingesta (S3 heredado) ✅; (3) G0 secrets-grep en verify (cero `sk-` nuevo; único hit = canary S1 pre-existente) ✅; (4) sin `cargo audit` (0 dependencias nuevas) ✅. Sin auth/input-usuario nuevo. Cerrado Step 3.
- [x] **PERFORMANCE** — N/A con motivo: worker serial intacto, 0 código prod (sin `dashmap`/`parking_lot`/Tokio nuevo → Regla 8 no dispara); tests Medium (localhost, Temp DB, puerto cerrado) en ~2s c/u. Sin baseline (nada que medir). Cerrado Step 3.

## Steps

### Step 1: Test 7 — `ingest_ollama_down_degrades` (TDD RED→GREEN)
- **Archivos:** `vantadb-mcp/tests/wiki_ingest_runner.rs` (APPEND ~50L)
- **Acción:** RED: test ausente (falla por no existir). GREEN: cfg `provider="ollama"` vía `from_toml_str` + `apply_env(|_| None)` (defaults heredados) con `base_url` override a `http://127.0.0.1:9` + `timeout_secs` corto → `build_ingest_runner` → `Some(Ollama)` → `run` → `Err` (cualquier `LlmError`, feature-agnóstico) → `worker::run` sobre fixture md ×2 → `sources_skipped==2`, `processed` vacío → facade `start_ingest::<ConcreteRunner>` con el runner construido → `poll_ready` → `ready` + consultable. Mitad mock: mismo cfg `pipeline_config()` + `ScriptedRunner` canned `file_block` → `sources_processed>0` + página legible (extract+merge escriben bajo cfg ollama).
- **Verify:** `cargo test -p vantadb-mcp -j 2 --test wiki_ingest_runner ingest_ollama_down_degrades`
- **Estado:** ✅ COMPLETED (verde 1.78s: `Some(Ollama)` + `Err` degradable + skipped==2 + facade ready + mock escribe 2 páginas bajo cfg ollama)

### Step 2: Test 8 — `ingest_openai_no_key_degrades` (TDD RED→GREEN)
- **Archivos:** `vantadb-mcp/tests/wiki_ingest_runner.rs` (APPEND ~50L)
- **Acción:** RED: test ausente. GREEN: cfg `provider="openai"` vía `from_toml_str` + `apply_env(|_| None)` (SIN key) → assert `openai_api_key == None` (secrets-solo-env) + `build_ingest_runner` → `Some(None)` (deferred `NotConfigured`, patrón B2b) → `worker::run` sobre fixture md ×2 → `sources_skipped==2` → facade → `ready` + consultable. Mitad mock: `pipeline_config()` + `ScriptedRunner` canned → escribe + legible (extract+merge bajo cfg openai).
- **Verify:** `cargo test -p vantadb-mcp -j 2 --test wiki_ingest_runner ingest_openai_no_key_degrades`
- **Estado:** ✅ COMPLETED (verde 1.25s: `None` sin key + `NotConfigured` + skipped==2 + facade ready + mock escribe 2 páginas bajo cfg openai)

### Step 3: Verify full + commit + cierre
- **Archivos:** ninguno (verificación + git)
- **Acción:** G0 (`git status` solo paths de la tarea + secrets-grep) + G2 (tests 7-8 verdes) + G3 (clippy workspace + fmt + coverage ps1 + suites `ingest`+`wiki` completas) + SECURITY/PERFORMANCE checklists + OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea) + `campaign_update_task_state(completed)` + commit `feat:` staging selectivo + `skill progreso` + RESULTADO §7 con veredicto fricción propia explícito
- **Verify:** `cargo test -p vanta-memory -j 2 ingest` + `cargo test -p vantadb-mcp -j 2 wiki` + `cargo clippy --workspace --all-targets --all-features -j 2 -- -D warnings` + `cargo fmt --check` + `pwsh scripts/validate-docs-coverage.ps1`
- **Estado:** ✅ COMPLETED (wiki_ingest_runner 5/5 + wiki_async_ingest 3/3 + memory ingest 15/15 + lib-ingest 19/19; clippy 0; fmt (1 autofix formato); coverage 0 gaps; secrets limpio; OCR preview sin Critical/High)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** orquestador asigna (`vanta-review` batch al cierre por área)
- **Enfoque:** ¿G2 observable en ambas variantes con asserts portables (feature-agnósticos)? ¿secrets-solo-env intacto? ¿0 código prod justificado por el Hallazgo (variantes ya en S1)?
- **Cómo se probó:** tests 7-8 + regresión 1-6,9-10 + G2/G3/G0 (evidencia en recitation, nunca auto-reporte)
- **Checklist anti-hábitos tóxicos:** a verificar por el revisor
- **Veredicto:** pendiente (lo registra el orquestador)

## Notas

- Gate D: NO disparado (0 símbolos `pub` nuevos; blast radius = 1 test file append; contrato cerrado por spec P2-01; Hallazgo documentado arriba).
- Gate V: no disparado en discovery (cero verify mecánico fallido; gate de entrada S1 CUMPLIDO). Umbral 2-fallas-mismo-error → question. Sin tool `question` en esta sesión: los gates se evalúan por evidencia y se registran aquí.
- `campaign_verify_cmd` bug exit -1 conocido → bash directa + mención en RESULTADO.
- `frontend-ui-engineering` descartada (sin `web/`).
- Context Save Point: DISCOVERY completo + task file creado; próximo = Step 1 (test 7 ollama).

## Recitation (canónica — cierre 2026-09-18)

- activeGoal: IMPL-112-S2 runners ollama/openai de ingesta
- lastAction: 3/3 steps ✅ + verify full G0/G2/G3 + commit (ver COMMIT_HASH en RESULTADO)
- result: OK
- nextAction: orquestador — P2-01 vanta-review (enfoque: G2 portable feature-agnóstico + secrets-solo-env + 0-código-prod justificado) + skill progreso + recitation plan
- contract: G0 (status: solo 2 paths tarea + secrets-grep sin `sk-` nuevo) + G2 (ollama puerto cerrado → skipped==2 + ready; openai sin key → `None` + skipped==2 + ready; mock escribe 2 páginas bajo cada cfg) + G3 (5/5 runner + 3/3 async + 15/15 ingest + 19/19 lib-ingest; clippy 0; fmt; coverage 0 gaps; inputSchema intacto por regresión); evidencia: comandos Steps 1-3 en este file + `git show --stat HEAD`; artefactos: 2 paths (test file + task file); invariantes: § Invariantes (schema/secretos/NoLlm/P4/STRUCTURAL/WIP); deuda: P2-01 pendiente (orquestador), ninguna técnica; queda_pendiente: ninguna (FIND-117 la ejecuta el orquestador)
- **Veredicto fricción propia (Gate V S2): SIN FRICCIÓN** — los tests 7-8 pasaron al primer run contra la implementación S1 con 0 archivos prod modificados; el genérico `start_ingest<R>` + el skip universal de `worker.rs:199-211` absorbieron ambas variantes; único ajuste = `cargo fmt` (formato, no diseño). Confirma el GO de S1: la matriz local+ollama/openai está completa.
- nextTask: FIND-117
