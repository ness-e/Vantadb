# FIND-112: spec + diseño del runner real de ingesta (SIN código)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-17-seguimiento-mvp.md` (Task 4, Wave1 primera en secuencia)
- **Fuente:** plan seguimiento-mvp Task 4 + FIND-107 §7 (P4 `sources skipped`)
- **Esfuerzo:** 🟡 1d (appetite 1d)
- **Prioridad:** 🟡 Media
- **Tipo:** spec-first / research (cero código; `campaign_detect_task_type` N/A — no hay `Archivos clave` a mutar, solo lectura)
- **Branch:** develop · **Commit:** `docs: FIND-112 — spec runner real de ingesta` (NO PUSH, staging solo este file)
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (SPEC — entregable = este file; implementación = IMPL-112, plan subsiguiente)
- **SDP:** spec-driven-development · documentation-and-adrs · doubt-driven-development · api-and-interface-design · source-driven-development · security-and-hardening (6; `interview-me`/`idea-refine` del lifecycle DEFINE descartadas: decisiones owner ya CERRADAS vía `question` 2026-09-17 — re-preguntar sería ruido, no señal; skill fantasma ` existe en ` del manifest ignorada — score 0.50 sin justificación real)

## Tarea

**Objetivo:** escribir la spec + diseño del runner real de ingesta wiki, SIN
tocar código. Hoy `start_ingest::<NoLlm>` está fijo (`vantadb-mcp/src/wiki.rs:274`,
runner `None`, `IngestConfig::default()`) = toda fuente cae en `sources_skipped`
(P4, `vanta-memory/src/ingest/worker.rs:201-221`); el pipeline es serial por
página (`vanta-memory/src/ingest/mod.rs:6-12`). La spec convierte ese uphill en
un slice mecánico para IMPL-112.

**Contrato (plan):** `docs/dev/tasks/FIND-112.md` spec con 5 puntos ((a) trait del
runner + qué provee cada variante de la matriz, (b) schema config TOML + tabla
env `VANTADB_*`, (c) lifecycle del runner en MCP — dueño explícito, (d) gates
de aceptación + tests a escribir en implementación, (e) qué sigue degradado con
`NoLlm` y por qué) + P2-01 sobre la spec; **CERO archivos de código tocados**
(solo el task file).

**Decisiones owner CERRADAS vía `question` 2026-09-17 (NO re-derivar, NO re-preguntar):**
(1) alcance = spec sin código; (2) matriz local+ollama/openai reusando
proveedores EMB; (3) TOML mínima + env, secrets solo env nunca disco;
(4) superficie `wiki_ingest` + pipeline general.

**AC de esta ejecución:**

- (a) spec con los 5 puntos completos + tabla variante×(transporte/config/secret/test) obligatoria.
- (b) dueño explícito del runner en MCP (sin dueño la spec NO cierra).
- (c) cero archivos de código tocados (solo este task file; `git status` limpio salvo este untracked→commiteado).
- (d) P2-01 lo hace el orquestador SOBRE LA SPEC (no esta ejecución).

**Prohibido en este slice:** todo `src/`, `vantadb-mcp/src/`, `vanta-memory/src/`,
`skills/`, `examples/`, `scripts/`, `reparacion.bat`, `.opencode`, `Justfile`,
`ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4,
`docs/dev/Backlog.md` (decisiones ya registradas por el lead), plan file (solo
recitation), `C:/Users/Eros/.vantadb*`, WIP ajeno en `git status`.

## Archivos

**Creación (único archivo tocado):**

- `docs/dev/tasks/FIND-112.md` (este file — el entregable ES la spec).

**Lectura (discovery, verificación CÓDIGO-REAL — no mutados):**

- `vantadb-mcp/src/wiki.rs:134-166` — schema input `wiki_ingest`/`wiki_ingest_status` (namespace/slug/root/run_id).
- `vantadb-mcp/src/wiki.rs:265-288` — handler `wiki_ingest`: valida `root` es dir, llama `start_ingest::<NoLlm>` fijo con `None` + `IngestConfig::default()` (el punto fijo a reemplazar).
- `vantadb-mcp/src/wiki.rs:365-391` — `IngestRun` + `ingest_runs()` registry + `NoLlm` (devuelve `Err(NotConfigured)` siempre).
- `vantadb-mcp/src/wiki.rs:399-460` — `start_ingest<R: LlmRunner + Send + Sync + 'static>` genérico: `begin` sync → registra tracker → `std::thread::spawn(worker::execute)`. El genérico YA existe: falta solo la selección del runner, no el genérico.
- `vantadb-mcp/src/wiki.rs:180-185` — `handle_wiki_tool(..., _config: &McpConfig)`: el config se IGNORA hoy (prefijo `_`); la spec lo convierte en la fuente del runner.
- `vanta-memory/src/ingest/worker.rs:32-62,95-104,187-235,311-327` — `run`/`run_with_progress`/`execute` serial; `extract_chunk` con `runner: Option<&R>` → `None`/`NotConfigured` = `sources_skipped` (P4, nunca hard error); merge serial vía `commit`.
- `vanta-memory/src/ingest/mod.rs:56-96` — `IngestConfig { global_llm_concurrency (clamp 1..=20, default 5), chunk_target_chars, chunk_overlap_chars }`.
- `vanta-memory/src/core/abstractions/llm_runner.rs:19-34,57-115` — `LlmRunParams { prompt, system_prompt, task_id, timeout, max_tokens, ... }`, `LlmError { NotConfigured, Timeout, Transport, Http, InvalidJson, Other }`, trait `LlmRunner { run + complete_json default }`. Implementaciones existentes: `StandaloneLlmRunner` (OpenAI-compat `/chat/completions`, gated `llm-driver`, sin feature = `NotConfigured`), `OpenClawLlmRunner` (delega a host), `MockLlmRunner` (tests, feature `mock`).
- `src/llm.rs:31-53,57-111,685-724` — `EmbeddingProvider { embed, embed_query, embed_batch }` + fábrica `get_embedding_provider()` sobre `VANTADB_EMBEDDING_PROVIDER` (ollama|openai|local); `OllamaProvider` lee `VANTADB_LLM_URL` (default `http://localhost:11434`) + `VANTADB_LLM_MODEL` (default `all-minilm`); `OpenAIProvider` exige `VANTADB_OPENAI_API_KEY` (error diferido al `embed`, nunca panic en construcción).
- `src/config.rs:197-214,295-309,871-957` — `LlmCfg` + defaults + lectura env vía `parse_env_or`/`env::var().ok()` (R-5 core-engine: prefijo único `VANTADB_*`, valor no reconocido → warn + default, nunca panic). NO existe TOML de config del core: env es la única fuente; el único precedente TOML mínima es `vanta-proxy.toml` (`[server]+[upstream]`, escrita por `setup-embeddings.ps1:444-464`, secrets nunca a disco `:16-18,125-131`).
- `setup-embeddings.ps1` — matriz EMB de referencia (provider local/ollama/openai, probe Ollama `:117-123`, presencia maskeada de key `:125-131`, ORT nativo, prueba viva put→get→search).

## Dependencias

- **Wave1 primera en secuencia.** FIND-109 ✅ (`9c881ac5`), FIND-110 ✅ re-DEFER (`1b3b3409`), FIND-111 ✅ SHIP (`234f0627+e61a15a3`).
- **Después:** FIND-113 (orquestador, dueño scheduler — espejo de esta spec) + FIND-101.
- **Trigger futuro:** IMPL-112 (plan subsiguiente): (1) runner local primero (cero secrets, cierra degradado P4), (2) Ollama/OpenAI si el trait los absorbe sin fricción. Si la spec muestra que ni el caso local tiene consumidor → la spec queda como decisión registrada y la implementación espera demanda (cierre válido, ponytail).
- **Stop (aplicado):** no hubo desacuerdo en dueño (decisión owner cerrada) → Gate V no disparado; spec cerró en 1d → DEFER no necesario.

## Referencias

- **Rules (leídas):** `.opencode/rules/core-engine.md` (R-1 feature-gating `experimental-*`; R-3 `?` + `Result`, sin unwrap; R-5 prefijo único `VANTADB_*` + `parse_env_or`) + `.opencode/rules/api-contract.md` (R-1 claim→símbolo real; R-3 no exponer en MCP lo que el core rechaza por defecto; R-8 lógica en core, bindings glue + memoria).
- **Refs:** `definition-of-done.md` (nivel task/commit: AC + DoD standing; shippable N/A — spec docs-only) · `architecture.md` · `skills-engineering.md` (SDP).
- **Commands:** `pipeline.md` (vía `pipeline-full.md`).
- **SPEC.md raíz:** N/A — no existe SPEC.md raíz versionado; la spec vive en este task file por contrato del plan.
- **Notion (fetch, filtro VantaDB memoria/core):** no mapeó — dim memoria sin correspondencia con runner de ingesta wiki; omitido con motivo (cero citas Notion en esta spec).
- **Internet:** no usado — sin ambigüedad de diseño (matriz = reutilizar EMB existente, verificado en código; patrones Mem0/Letta innecesarios). Sin deuda TSYS-13.

## Skills

- **Cargadas (SDP Paso 0b, `campaign_discover_skills_v2` phase DEFINE):** `spec-driven-development` (núcleo: Specify→Plan→Tasks, decisiones owner como input validado) · `documentation-and-adrs` (spec como documento vivo + registro de decisiones) · `doubt-driven-development` (dueño difuso: el punto que decide si la spec cierra) · `api-and-interface-design` (schema config contract-first, validación en frontera) · `source-driven-development` (todo claim → `file:línea` real) · `security-and-hardening` (secrets solo env, base type MCP server).
- **Descartadas con motivo:** `interview-me`/`idea-refine` (lifecycle DEFINE las sugiere, pero decisiones ya cerradas por owner — re-preguntar = ruido).

## Herramientas + MCP

- `codegraph_explore "wiki start_ingest NoLlm ingest worker pipeline LlmRunner"` — blast radius: `start_ingest` genérico ya existe; `LlmRunner` con 3 implementaciones + N mocks de test; `PipelineWorker` (scheduler, FIND-113) como caller aguas arriba.
- `Read` directo (ver § Archivos — paths exactos con líneas).
- `campaign_verify_cmd`: bug exit -1 conocido → N/A con motivo (cero código: no hay `cargo`/clippy/fmt/nextest que correr; sí `git diff --check` al cierre).
- OCR delegation: N/A con motivo (cero código; solo toca este task file, untracked → sin diff de código que delegar).

## Investigación código (DISCOVERY — evidencia)

1. **Facade fija:** `handle_wiki_tool` ignora `McpConfig` (`_config`, `wiki.rs:180-185`) y llama `start_ingest::<NoLlm>(..., None, IngestConfig::default())` (`wiki.rs:274-281`). Punto único de cambio para IMPL-112.
2. **Genérico ya existe:** `start_ingest<R: LlmRunner + Send + Sync + 'static>(storage, namespace, slug, root, runner: Option<R>, config: IngestConfig)` (`wiki.rs:399-409`) hace `begin` sync → registra tracker → `thread::spawn(worker::execute)`. IMPL-112 NO crea el genérico: solo construye el `R` concreto desde config y lo pasa.
3. **Pipeline serial + P4 honesto:** `execute` itera fuentes serial (`worker.rs:187-235`); sin runner o con `NotConfigured` → `sources_skipped.push` (`:214`), nunca error duro; `commit` merge serial (`:249-301`). `IngestConfig` = 3 campos con clamp/default (`mod.rs:58-96`).
4. **Fábrica EMB reutilizable:** `get_embedding_provider()` (`src/llm.rs:65-101`, 3 variantes por feature-set) + `LlmCfg` env (`config.rs:919-957`): `VANTADB_EMBEDDING_PROVIDER` (default `ollama`), `VANTADB_LLM_URL`, `VANTADB_LLM_MODEL`, `VANTADB_LLM_SUMMARIZE_MODEL`, `VANTADB_LOCAL_MODEL`, `VANTADB_OPENAI_API_KEY` (solo presencia, nunca log en claro `:942-944`), `VANTADB_OPENAI_MODEL`. `OpenAIProvider` sin key = error diferido al `embed` (`llm.rs:836`), no panic en construcción (patrón B2b a replicar en el runner chat).
5. **Runners chat existentes:** `StandaloneLlmRunner { LlmConfig { base_url, api_key, model, max_tokens, timeout } }` (`adapters/standalone/llm_runner.rs:23-64`); sin `llm-driver` compila pero `run` = `NotConfigured` (`:104-118`) — el mismo degradado P4 que necesita la matriz.

## Investigación problema (tradeoffs decididos)

- **Matriz ×3 sin explosión:** la tabla obligatoria (§ Spec (a)) fija por variante transporte/config/secret/test; el trait común (`LlmRunner::run`) absorbe las 3 sin branching en el worker (R-8 api-contract: el worker no decide por variante, solo degrada ante `LlmError`). Riesgo pre-mortem (1) cerrado por tabla.
- **Lifecycle-dueño (el punto que decide si la spec cierra):** dueño explícito = MCP (`ingest_runs()` registry en `vantadb-mcp/src/wiki.rs:377-380`), ver § Spec (c). Riesgo pre-mortem (2) cerrado por dueño + regla de no-cierre-sin-dueño.
- **Qué-queda-degradado declarado:** § Spec (e) — `NoLlm` sigue siendo el default honesto (cero secrets, cero red) y el contrato P4 no cambia.

---

## Spec (el entregable — 5 puntos)

### (a) Trait del runner + qué provee cada variante de la matriz

**Trait (sin cambios — reutilizar, NO redefinir):**

```rust
// vanta-memory/src/core/abstractions/llm_runner.rs:91-115 — YA EXISTE
pub trait LlmRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError>;
    fn complete_json<T: DeserializeOwned>(&self, params: &LlmRunParams) -> Result<T, LlmError> { /* default */ }
}
```

IMPL-112 NO toca el trait. El worker consume `Option<&R>` y degrada ante
`LlmError::NotConfigured` (P4) o cualquier otro `LlmError` (warn + skip,
`worker.rs:206-211`). Decisión `doubt-driven`: ¿nuevo trait `IngestRunner`?
RECHAZADO — `LlmRunner::run` puro-texto ya cubre extract (`worker.rs:320-326`)
y merge (`merge::commit`); un trait nuevo duplicaría el contrato sin consumidor
distinto (R-8: no duplicar lógica; YAGNI).

**Constructor requerido (único código nuevo del slice 1 de IMPL-112):**

```rust
// Propuesto: vantadb-mcp/src/wiki.rs (o módulo nuevo `ingest_runner.rs` si >100L)
fn build_ingest_runner(cfg: &IngestRunnerCfg) -> Option<ConcreteRunner>;
```

donde `ConcreteRunner` es un enum (NO `Box<dyn>` — el `start_ingest<R>` genérico
monomorfiza por `R`; un enum con `impl LlmRunner` por delegación mantiene el
genérico sin object-safety ni `Send+Sync+'static` adicionales):

```rust
enum ConcreteRunner { Local(LocalChatRunner), Ollama(StandaloneLlmRunner), OpenAi(StandaloneLlmRunner), None }
impl LlmRunner for ConcreteRunner { /* delega run; None → Err(NotConfigured) */ }
```

**Tabla obligatoria variante×(transporte/config/secret/test):**

| Variante | Transporte | Config (de § (b)) | Secret | Test en IMPL-112 |
|----------|-----------|-------------------|--------|------------------|
| `local` | ONNX/embed-local en proceso (chat local vía `LlmCfg.local_model_path`; si el modelo chat no existe → `NotConfigured`, igual que `LocalOnnxProvider::new_dummy` en `llm.rs:70-77`) | `provider="local"` + `model_path` | ninguno (cero secrets — cierra degradado P4) | `ingest_local_no_model_degrades` (sin modelo → `sources_skipped`, `IngestReport` OK) + `ingest_local_with_model_writes_pages` (con fixture modelo o fake `LlmRunner`, ver slice 1) |
| `ollama` | HTTP `POST {base_url}/api/chat` (o `/chat/completions` OpenAI-compat si el servidor lo expone) vía `StandaloneLlmRunner` + feature `llm-driver` | `provider="ollama"` + `base_url` (default `VANTADB_LLM_URL`) + `model` (default `VANTADB_LLM_MODEL`) | ninguno (localhost sin auth; si el servidor exige key futura → env, nunca TOML) | `ingest_ollama_down_degrades` (servidor caído → `Transport` → skip, nunca hard error; patrón `executor.rs:899-927` auto-embed graceful) + `#[ignore] ingest_ollama_live` (requiere `ollama serve`, solo CI heavy/manual) |
| `openai` | HTTPS `POST {base_url}/chat/completions` vía `StandaloneLlmRunner` + feature `llm-driver` | `provider="openai"` + `model` (default `VANTADB_OPENAI_MODEL`-análogo chat) + `base_url` overrideable | `VANTADB_OPENAI_API_KEY` SOLO env (ausente → `NotConfigured` diferido al `run`, patrón B2b `llm.rs:836`; prohibido leer key desde TOML) | `ingest_openai_no_key_degrades` (sin env → `sources_skipped`, exit OK) + `ingest_openai_mock_http` (mock `LlmRunner` con respuesta canned → páginas escritas; NUNCA key real en tests/fixtures) |

**Regla de feature-gating (R-1 core-engine):** `ollama`/`openai` reales exigen
feature `llm-driver` (como `StandaloneLlmRunner` hoy); sin la feature el enum
construye la variante pero su `run` = `NotConfigured` (compila en default,
degrada en runtime — R-3 api-contract: nunca exponer tool que falla siempre
SIN el degradado documentado; el degradado P4 ES el contrato).

### (b) Schema config TOML + tabla env `VANTADB_*`

**TOML mínima (precedente `vanta-proxy.toml` — solo no-secrets; secrets NUNCA a disco):**

```toml
# <db-path>/vanta-ingest.toml — config mínima de ingesta (IMPL-112 la lee si existe; si no, todo defaults = comportamiento actual NoLlm)
[ingest]
provider = "local"          # "local" | "ollama" | "openai" (default "local" → ver nota)
model = ""                  # override; "" = default por provider (ver tabla env)
base_url = ""               # override; "" = default por provider
max_tokens = 0              # 0 = default del runner (120s timeout fijo hoy → parametrizable en slice 2)
timeout_secs = 0            # 0 = 120 (default StandaloneLlmRunner)

[ingest.pipeline]
global_llm_concurrency = 5  # clamp 1..=20 (IngestConfig existente)
chunk_target_chars = 12000  # default core chunker
chunk_overlap_chars = 400   # default core chunker
```

**Nota default `provider`:** default `"local"` (cero red, cero secrets) con
fallback honesto: si `local` no tiene modelo disponible → `NotConfigured` →
P4 `sources_skipped` (comportamiento actual preservado bit a bit). Ponytail:
no hay auto-detección de Ollama corriendo (probe = red en el path de config;
el error `Transport` en runtime ya informa).

**Tabla env `VANTADB_*` (R-5 core-engine: prefijo único, `parse_env_or`, warn+default nunca panic; env GANA a TOML — operador > archivo):**

| Env | Default | Usada por variante | Notas |
|-----|---------|-------------------|-------|
| `VANTADB_INGEST_PROVIDER` | `local` | las 3 | `local`\|`ollama`\|`openai`; valor no reconocido → `warn!` + `local` (R-5) |
| `VANTADB_INGEST_MODEL` | `""` → default por provider | las 3 | `ollama`→`VANTADB_LLM_MODEL`; `openai`→`VANTADB_INGEST_OPENAI_MODEL` o `gpt-4o-mini`; `local`→`VANTADB_LOCAL_MODEL` |
| `VANTADB_INGEST_BASE_URL` | `""` → default por provider | ollama/openai | `ollama`→`VANTADB_LLM_URL`; `openai`→`https://api.openai.com/v1` |
| `VANTADB_INGEST_CONFIG` | `<db>/vanta-ingest.toml` | las 3 | path del TOML (inyectable en tests vía Temp) |
| `VANTADB_OPENAI_API_KEY` | unset | openai | SOLO env, nunca TOML (audita `vanta-review`: grep `api_key` en TOML = fail). Ausente → `NotConfigured` diferido, nunca panic en construcción |
| `VANTADB_INGEST_MAX_TOKENS` | `0` (= default runner) | las 3 | clamp 1..=8000 cuando >0 |
| `VANTADB_INGEST_TIMEOUT_SECS` | `120` | las 3 | clamp 5..=600 |
| `VANTADB_LLM_URL` / `VANTADB_LLM_MODEL` / `VANTADB_LOCAL_MODEL` / `VANTADB_OPENAI_MODEL` | (existentes, `config.rs:919-957`) | reused | NO duplicar: `IngestRunnerCfg::from_env` delega en `LlmCfg` y solo añade el prefijo `VANTADB_INGEST_*` como override específico |

**Orden de precedencia (contract-first):** `VANTADB_INGEST_*` env > TOML >
`VANTADB_LLM_*` heredadas > defaults del código. Validación en frontera
(api-and-interface-design §3): `IngestRunnerCfg::validate()` en construcción
(provider desconocido/clamps/ruta TOML ilegible → `warn!` + default seguro,
nunca `unwrap`).

### (c) Lifecycle del runner en MCP — dueño explícito

**Dueño: el proceso MCP (`vantadb-mcp`), registry `ingest_runs()` (`wiki.rs:377-380`).**
Sin dueño la spec NO cierra (pre-mortem 2) — aquí está:

1. **Construcción (por llamada `wiki_ingest`, NO global):** `handle_wiki_tool`
   deja de ignorar `McpConfig` y construye `IngestRunnerCfg::from_env_toml()`
   → `build_ingest_runner(&cfg)` → `Option<ConcreteRunner>` movido al
   `thread::spawn` de `start_ingest` (`wiki.rs:440-456`). Por llamada (no
   `OnceLock` global) porque: (a) env puede cambiar entre llamadas; (b) el
   runner es barato de construir (struct con Strings, sin conexión persistente
   — `reqwest::blocking::Client` se crea por llamada, como `run_http` hoy);
   (c) evita estado global con secrets (el `api_key` vive solo en el stack del
   thread de ingesta, nunca en un static).
2. **Vida:** el runner vive lo que el thread de ingesta (`move` al closure);
   al terminar `execute`, se dropea con el thread (la key OpenAI nunca queda
   en memoria global). El `IngestRun { tracker, namespace, slug }` sigue en el
   registry para `ingest_status` (contrato D19 intacto).
3. **Multi-proceso (stdio stateless):** cada proceso MCP tiene su registry
   (igual que FIND-110: queue por proceso). `wiki_ingest_status` con `run_id`
   de otro proceso → `Unknown run_id` (mensaje actual, honesto; no se promete
   cross-process en este slice — FIND-113 decide el backend del scheduler, no
   esta spec).
4. **Pipeline general (superficie (4) owner):** el mismo `build_ingest_runner`
   sirve a futuros consumidores (`skill_extract` FIND-111, L1/L2/L3) — el
   enum + `IngestRunnerCfg` viven en `vanta-memory` (core, R-8: lógica en
   core) y `vantadb-mcp` solo mapea `McpConfig → IngestRunnerCfg` (glue).
   `NoLlm` NO se borra: queda como variante `ConcreteRunner::None` explícita
   y como runner de tests.
5. **NO-dueños declarados:** `WikiStore` (core, stateless por llamada) no
   posee el runner; el scheduler FIND-113 no lo posee (cuando exista,
   pedirá un runner al mismo constructor con su propio config).

### (d) Gates de aceptación + tests a escribir en implementación

**Gates de IMPL-112 (slice mecánico — orden obligatorio, local primero):**

- **G0 (contrato):** `git status` muestra solo archivos de IMPL-112; secrets grep-limpios (`rg -n "sk-|api_key\s*=\s*\"[^\"]+" --glob '*.toml' --glob '*.md'` vacío salvo placeholders `...`).
- **G1 (slice 1 — local):** `wiki_ingest` con `VANTADB_INGEST_PROVIDER=local` sin modelo → `sources_skipped` == nº fuentes, `state=ready|failed` consultable por `run_id`, exit OK. Con fake runner → páginas escritas + `sources_processed` > 0.
- **G2 (slice 2 — ollama/openai tras `llm-driver`):** sin servidor/key → degrada P4 (mismo assert que G1); con mock HTTP/runner canned → extract+merge escriben.
- **G3 (regression):** suite `ingest` + `wiki` existente verde; `cargo clippy -- -D warnings` 0; `cargo fmt --check`; `validate-docs-coverage.ps1` 0 gaps (fila MCP.md si cambia el schema del tool — NO cambia en slice 1: `wiki_ingest` mantiene su inputSchema; el config es server-side).
- **G4 (review):** P2-01 `vanta-review` sobre el diff + `vanta-chaos` N/A con motivo (ingesta serial single-thread, sin `dashmap`/`parking_lot`/Tokio nuevo — Regla 8 no dispara; si slice 2 añade concurrencia → dispara).

**Tests a escribir (nombres exactos, ubicación propuesta `vantadb-mcp/tests/wiki_ingest_runner.rs` + `vanta-memory` unit):**

1. `ingest_runner_cfg_defaults_to_local_p4` — sin env ni TOML → provider `local`, `build` → `None`/`NotConfigured`, `run` → `Err(NotConfigured)`.
2. `ingest_runner_cfg_env_beats_toml` — TOML Temp `provider="openai"` + env `VANTADB_INGEST_PROVIDER=ollama` → `ollama`.
3. `ingest_runner_cfg_unknown_provider_warns_and_falls_back` — `provider="watson"` → `local` (no panic).
4. `ingest_runner_cfg_never_reads_key_from_toml` — TOML con `api_key="sk-..."` → ignorada (o error); runner openai sin env → `NotConfigured`.
5. `ingest_local_no_model_degrades` (slice 1) — `wiki_ingest` sobre fixture markdown en Temp sin modelo → `sources_skipped.len() == n_fuentes`, `IngestReport` OK, `wiki_ingest_status` consultable.
6. `ingest_local_canned_runner_writes_pages` (slice 1) — `MockLlmRunner`/fake con bloques `parse_file_blocks`-válidos → `sources_processed > 0`, páginas legibles vía `wiki_read`.
7. `ingest_ollama_down_degrades` (slice 2) — `base_url` a puerto cerrado → `Transport` → skip, nunca hard error.
8. `ingest_openai_no_key_degrades` (slice 2) — env ausente → `NotConfigured` → P4.
9. `ingest_concrete_runner_is_send_sync_static` — assert de tipos (`fn assert_send_sync<T: Send + Sync>()`) para el enum (el `start_ingest<R>` genérico lo exige).
10. `ingest_chunk_config_clamps` — `global_llm_concurrency=99` → 20; `0` → 5 (reuso `clamp_llm_concurrency`, test de wiring no de lógica duplicada).

### (e) Qué sigue degradado con `NoLlm` y por qué

Con `provider=local` sin modelo, o cuando cualquier variante devuelve
`LlmError` (cualquier variante de `LlmError`, no solo `NotConfigured` —
`worker.rs:206-211` trata `Transport`/`Timeout`/`Http` igual: warn + skip):

- **Extract:** `extract_chunk` → `Err` → fuente a `sources_skipped`; `extracted` vacío para esa fuente. Por qué: sin LLM no hay segmentación semántica en candidatos `CandidatePage` (el prompt de extracción `prompts::extraction_*` exige modelo); inventar candidatos sin modelo = alucinación persistida (peor que skip explícito).
- **Merge:** `commit` sin candidatos por fuente → nada que fusionar para ella; páginas pre-existentes intactas (merge serial nunca borra — `STRUCTURAL_FILES` + `locked` frontmatter). Por qué: el merge LLM decide conflicto/merge/skip; sin modelo la única fusión segura es ninguna.
- **Estado final:** la build COMPLETA (`processing → ready`, report con `sources_skipped` explícito, cero skip silencioso) — el degradado es observable vía `wiki_ingest_status` + `IngestReport`, no un fallo. Por qué: P4 es contrato, no bug (tool description `wiki.rs:134` lo declara: "Without an LLM configured the build completes with sources skipped").
- **Lo que NO degrada (siempre funciona):** scan/chunk (`chunk_text` puro), state machine (`begin_processing/complete/fail`), progress tracker, `wiki_read/list/search/graph` sobre páginas ya existentes.
- **Cambio de comportamiento introducido por IMPL-112:** ninguno por defecto — default `local` sin modelo ≡ `NoLlm` actual bit a bit (G1 lo testea). La mejora es opt-in por config.

---

## Spec — decisiones registradas (-speaking como ADR ligera)

| # | Decisión | Alternativa rechazada | Por qué |
|---|----------|----------------------|---------|
| S1 | Reutilizar `LlmRunner`, enum `ConcreteRunner` por delegación | Nuevo trait `IngestRunner` / `Box<dyn LlmRunner>` | YAGNI: ningún consumidor necesita otro método; el genérico `start_ingest<R>` monomorfiza mejor con enum (sin object-safety) |
| S2 | TOML mínima solo no-secrets + env con precedencia | Todo-env (status quo) / todo-TOML | TOML da `[ingest.pipeline]` versionable; env da secrets + override operador (precedente proxy + setup-embeddings) |
| S3 | Runner por llamada, sin global | `OnceLock<Runner>` global | Secrets en static = riesgo; construcción barata; env mutable entre llamadas |
| S4 | `NoLlm` vive (variante `None` explícita) | Borrar `NoLlm` al shippear | Default cero-red/cero-secret debe seguir testeable; es el runner de tests P4 |
| S5 | IMPL por etapas, local primero | Big-bang ×3 | Local cierra P4 sin secrets; si el trait fricciona en ollama/openai, G1 ya shipeó valor |
| S6 | `wiki_ingest` inputSchema SIN cambios (slice 1) | Añadir `provider` al tool input | Config server-side (operador), no per-call del LLM (Hyrum: cada campo del schema es contrato para siempre) |

## Validación + cierre (esta ejecución)

- [x] Spec con 5 puntos completos ((a) trait+matriz+tabla obligatoria · (b) TOML+env · (c) lifecycle+dueño explícito · (d) gates+10 tests nombrados · (e) degradado declarado)
- [x] Dueño explícito (§ (c): proceso MCP, registry `ingest_runs()`, construcción por llamada)
- [x] Cero código: `git status` solo este file (verificado al cierre con `git diff --check` + `git status --short`)
- [x] `git diff --check` limpio
- [x] OCR delegation: N/A justificado (cero código; solo este task file untracked)
- [x] DoD nivel task/commit: AC cumplidos + commit `docs:` + sin deuda + secreto ninguno (placeholders `sk-...` solo como patrón grep, nunca key real)
- [ ] P2-01: lo hace el ORQUESTADOR sobre esta spec (pendiente, `NextTask` amici: FIND-113 tras review)
- [ ] Gates D/V/C: D no disparado (spec-first sin blast radius de escritura; decisiones owner cerradas) · V no disparado (cero verify mecánico aplicable; `git diff --check` como verify) · C cumplido en RESULTADO
- [ ] `skill progreso`: la ejecuta el orquestador al cerrar (esta ejecución no muta Backlog/plan/avance — prohibido por contrato)

## Handoff (para el orquestador)

1. P2-01 `vanta-review` SOBRE ESTA SPEC (veredicto approve/changes-required con evidencia).
2. Si approve → `campaign_update_task_state(FIND-112, completed)` + recitation + commit ya hecho por esta ejecución (ver COMMIT_HASH en RESULTADO) + `skill progreso`.
3. NextTask: FIND-113 (orquestador/dueño scheduler — espejo de § (c)).
4. IMPL-112 vive en el plan subsiguiente (trigger: spec cerrada + P2-01 approve). Slices: (1) local (`IngestRunnerCfg` + enum + G1 + tests 1-6,9-10), (2) ollama/openai (`llm-driver` + G2 + tests 7-8).
