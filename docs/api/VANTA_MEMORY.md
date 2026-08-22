# Vanta Memory Engine — API Reference (`vanta-memory`)

> **Estado:** ✅ documentación canónica del crate (cierra la cita de ADR-029 §Nota mecánica).
> **Nota:** `scripts/validate-docs-coverage.ps1` hoy NO escanea `vanta-memory`; esta página es
> la referencia manual. Las superficies F1-F3 (search profile, entity_*, skills) viven en
> `EMBEDDED_SDK.md`.

Crate LLM-driven para memoria de agentes: captura L0, extracción/dedup L1, escenas L2,
persona L3, recall con scope, context engine (compresión), offload y generación wiki.
**Principio rector:** el LLM es opcional (P4) — todo flujo degrada sin perder datos cuando
el runner falla o no está configurado.

## Arquitectura por capas

| Capa | Módulo | Qué hace |
|---|---|---|
| L0 | `core::conversation::l0_recorder`, `core::hooks::auto_capture` | Captura idempotente de turnos (cursor `l0_cursor/<session>`) |
| L1 | `core::record::{l1_extractor,l1_dedup,l1_reader,l1_writer}` | Extracción 1-call LLM JSON con parse reparado; dedup 2 fases store/update/merge/skip |
| L2 | `core::scene::{scene_index,scene_format,scene_extractor,scene_tools}` | Escenas con META {created,updated,summary,heat}, strategy UPDATE>MERGE>CREATE, soft-delete, tools sandboxed |
| L3 | `core::persona::{persona_generator,persona_trigger}` | Persona first/incremental con triggers P1-P4 y escape XML |
| Recall | `core::hooks::auto_recall`, `core::memory_prompt::*`, `core::profile::profile_sync` | Prepend/append + 3 modos (`RecallScope::Session\|Agent\|Team`, default Agent) |
| Context | `context_engine::{engine,compressor,mmd,mmd_injector,token_estimator,types}` | Compresión LLM-free mild/aggressive/emergency + MMD persistente + budget coordinator |
| Offload | `offload::{state_manager,storage,reclaimer,hooks::after_tool_call}` | Cursor `lastOffloadedToolCallId`, entradas por tool_call_id, GC por retention |
| Ingest | `ingest::{worker,merge,prompts,callback}` | Ingest wiki serial (fallo por página no bloquea), progreso canal interno + polling run_id |
| Skills | `core::skill::skill_extractor` + `conversation_add` | Extracción desde transcript con marcadores anti role-capture; sink idempotente doble cursor+content-hash |
| Orquestación | `services::pipeline_worker`, `utils::{pipeline_manager,stateful_pipeline_manager,managed_timer,checkpoint}` | Timers/locks estado local, trait `Clock` inyectable (FakeClock determinista), worker L0→L1→L2→L3 |
| Gateway | `gateway::knowledge_handlers` | Handlers tipados scene_read/list/query para exposición MCP/server |

## Contratos clave

### Trait `LlmRunner` (host-neutral, sync)
```rust
pub trait LlmRunner {
    fn run(&self, params: LlmRunParams) -> Result<String, LlmError>;
    // complete_json<T>: helper genérico — NO dyn-compatible; usar <R: LlmRunner>
}
```
Los extractores/generadores son funciones genéricas `<R: LlmRunner>`; el fallo del runner
degrada a `success: false` / skip documentado — jamás bloquea ni corrompe estado previo.

### Context engine
```rust
assemble(messages, budget, estimator, protected_prefix, cfg) -> AssembleOutput
assemble_with_recall(...)  // coordinator único: assemble → inject_mmd → recall, un solo budget
```
- Ratio < 0.5 → skip sin tocar mensajes.
- Mild cascade (MIN=10/INITIAL=7/FLOOR=1) → aggressive one-shot (fingerprint boundary
  `role + primeros 200 chars`, idempotente) → emergency prefix-aware (~2000 chars).
- Los pares tool_call/tool_result son unidades atómicas: nunca se parten.
- Mensajes ≤ cursor `lastOffloadedToolCallId` (MEM-20) van en `protected_prefix`.
- `inject_mmd` agrega `<current_task_context>` tras el prefijo System con dedup fingerprint.

### Recall
```rust
RecallConfig { recall_scope: RecallScope, .. }   // Session | Agent | Team — default Agent
perform_auto_recall(db, params) -> RecallResult { prepend_context, append_system_context, .. }
```
Prepend = memories dinámicas per-turn; Append = persona/escenas estables (prompt-cache friendly).
Embedding/Hybrid degradan a keyword-overlap hasta que el core exponga embeddings (D37).

### Wiki ingest (F7)
```rust
worker::run(store, sources_root, runner_opt, cfg) -> IngestResult
worker::run_with_progress(..., Option<&ProgressTracker>)   // throttle 500ms
progress_tracker.wiki_status(run_id) -> Option<IngestProgress>
```
Serial por página; fallo de página no bloquea las siguientes; STRUCTURAL_FILES protegidos;
`ensure_sources` fuerza frontmatter. Fallback P4 sin runner: páginas nuevas verbatim,
merges requeridos se registran como skipped.

## Namespaces (sanitizados `[A-Za-z0-9._/-]` ≤128B)

| Namespace | Contenido |
|---|---|
| `l0/<session>` · `l0_cursor/<session>` | turnos crudos · cursor de captura |
| `l1/<session>` | memories extraídas (+ dedup state) |
| `scene/<session>` · `mmd/<session>/{active,history}` | escenas · memoria de tarea |
| `persona/<session>` · `profile/{scope}` | persona · perfil sincronizado |
| `offload/<session>` · `offload_state/<session>` | entradas offload · cursor |
| `pipeline_checkpoint` | contadores del orquestador |
| `genlog/<session>` | provenance de generaciones (best-effort, cap 100) |
| skills_extract/<scope> | seed/import CLI |

## CLI

- `vanta-seed <seed.json> [--db <path>]` — import inicial de skills/persona, idempotente por
  content-hash.

## Deudas conocidas (upgrade paths documentados)

1. **Keyword-overlap sin embeddings** — recall/dedup/query heurísticos; upgrade a vector
   search cuando el core exponga API de embeddings (afecta cross-idioma/paráfrasis).
2. **TokenEstimator chars/3** (D21) — subestima CJK/código; factor configurable; calibrar
   contra benchmarks reales si el drift >15%.
3. **Scoring de compresión heurístico** — sustituye los scores del L1 de TDAM; consumir
   scores reales es upgrade futuro.
4. **Context engine ↔ pipeline worker** — el engine es library-API (consumido por tests e2e);
   el wiring productivo al worker de MEM-16 está pendiente de decisión (ver backlog).
5. **Fetcher HTTPS/git** — diferido (D30/D36); implementar con SSRF blocklist NO desactivable
   cuando haya fuentes remotas.

Ver ADR-029 para el racional completo de las decisiones D21-D23.
