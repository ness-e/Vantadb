# MEM-08b: F4 Contratos L1 + trait LLMRunner host-neutral

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-memory.md (NO editar — lead)
- **Creado:** 2026-08-20T20:15
- **last-synced:** 2026-08-20T20:15
- **Estado:** ⏳ IN PROGRESS (delegado a vanta-worker)
- **Fuentes TDAM (clon `C:\Users\Eros\AppData\Local\Temp\opencode\tdam`):**
  - `MemoryCore/src/core/record/l1-writer.ts` — MemoryRecord/ExtractedMemory/DedupDecision (tipos reales; `core/abstractions/types.ts` NO contiene estos — contiene IConfigSource/Quota)
  - `MemoryCore/src/core/record/l1-dedup.ts` — DedupDecision + parse LLM (snake_case wire)
  - `MemoryCore/src/core/record/l1-extractor.ts` — L1ExtractionResult + SceneSegment
  - `MemoryCore/src/core/types.ts` — LLMRunner/LLMRunParams (contrato host-neutral)
  - `MemoryCore/src/adapters/standalone/llm-runner.ts` — StandaloneLLMRunner (OpenAI-compatible)
  - `MemoryCore/src/adapters/openclaw/llm-runner.ts` — OpenClawLLMRunner (wraps host runner)
  - `MemoryCore/src/offload/types.ts` — OffloadEntry/ToolPair/PluginState (cursor MEM-20)
- **Reportes:** `docs/research/tdam/01-core-pipeline.md` (§62-63 contratos L1), `02-scene-persona.md` (§33 SceneSegment, §34-37 strategy/heat, §41 triggers persona, §52-53 META/índice), `SYNTHESIS.md` (§2.4 qué NO copiar)

## Contrato
"`cargo check -p vanta-memory` pasa; tests dedicados de tipos/trait (D19)"
- `cargo check -p vanta-memory` (default) → exit 0
- `cargo check -p vanta-memory --no-default-features` y `--features mock` → exit 0
- `cargo nextest run -p vanta-memory` → all pass
- `cargo clippy -p vanta-memory --all-targets -- -D warnings` → exit 0
- `cargo fmt --check` → exit 0

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vanta-memory/Cargo.toml`, `vanta-memory/src/lib.rs`, `vanta-memory/src/{core,adapters,offload}/mod.rs`, `vanta-memory/tests/smoke.rs`, `docs/plans/2026-08-18-vanta-memory.md` (secciones plan/principios/tasks), `Cargo.toml` raíz (workspace.lints, members, workspace.package), `src/llm.rs` (grep: reqwest blocking pattern), TDAM refs listadas arriba.
- **Referencias hacia dentro (dependen de lo que creo):** nada aún — archivos nuevos; MEM-09..21 los consumirán (L1 extractor/dedup, scene, persona, cursor). `lib.rs` declara `pub mod core/adapters/offload` (solo mod.rs hoy — agrego submods).
- **Referencias hacia fuera (de lo que dependo):** `serde`/`serde_json`/`thiserror`/`tracing` ya en Cargo.toml (MEM-08a); `reqwest 0.12 blocking` patrón del workspace (Cargo.toml:50, src/llm.rs:15) — dep opcional SOLO bajo feature `llm-driver`. No toco `docs/plans/`.
- **Veredicto:** seguro — archivos nuevos dentro del crate experimental `vanta-memory`, fuera de default-members. Cero blast radius en core/bindings.

## Steps
- **Step 1 (contratos L1):** `src/core/abstractions/mod.rs` + `types.rs` — MemoryType, EpisodicMetadata(como Value), ExtractedMemory, MemoryRecord, DedupAction, DedupDecision, L1ExtractionResult, SceneSegment, SceneMeta, SceneIndexEntry, PersonaMode, PersonaTriggerPriority. Serde snake_case (wire LLM), doc comments EN. — ⬜
- **Step 2 (trait LLMRunner):** `src/core/abstractions/llm_runner.rs` — trait `LlmRunner` sync (D1) `run() -> Result<String, LlmError>` + default `complete_json<T>()`; `LlmRunParams`, `LlmError` (thiserror); `AsyncLlmRunner` (trait, `#[cfg(feature = "llm-driver")]`). Wire en `core/mod.rs` y `core/abstractions/mod.rs`. — ⬜
- **Step 3 (offload types):** `src/offload/types.rs` — OffloadEntry, ToolPair, PluginState (solo lo que MEM-20 cursor consume; MMD/L1.5 difieren a F5). — ⬜
- **Step 4 (adapters):** `src/adapters/standalone/mod.rs` + `llm_runner.rs` (LlmConfig + StandaloneLlmRunner: HTTP OpenAI-compatible bajo `llm-driver` vía reqwest blocking; `NotConfigured` sin feature — degradación LLM-free); `src/adapters/openclaw/mod.rs` + `llm_runner.rs` (trait `OpenClawHost` = port del host; `OpenClawLlmRunner` delega — sin dep de OpenClaw real); `src/adapters/mock.rs` (`MockLlmRunner` bajo `mock`). — ⬜
- **Step 5 (tests D19):** `tests/types.rs` (roundtrips serde) + `tests/llm_runner_contract.rs` (runner local fijo: run() + complete_json + error). — ⬜
- **Step 6 (verify):** cargo check default/no-default/mock + nextest + clippy + fmt. — ⬜
- **Step 7 (cierre):** git diff review, commit preparado para lead, RESULTADO. — ⬜

## Dependencias
- MEM-08a (scaffold) ✅ — Cargo.toml con features `llm-driver`/`mock`, 6 módulos esqueleto.

## Notas / decisiones de diseño
1. **`core/abstractions/types.ts` de TDAM NO contiene MemoryRecord** (es IConfigSource/Quota — deployment config, SKIP por SYNTHESIS §2.4/deuda TDAM). Los contratos reales viven en `l1-writer.ts`/`l1-dedup.ts`/`l1-extractor.ts` — de ahí los porto (plan file los citaba mal; documentado).
2. **Trait sync base (D1):** `LlmRunner::run` bloqueante = contrato host-neutral mínimo (idéntico shape a TDAM `LLMRunner.run`). `complete_json` default = la operación "JSON estructurado" que L1 extract/dedup, L2, L3 necesitan (strip fences + primer array/objeto + serde). Reparación completa = MEM-10 `json_utils.rs`.
3. **Async (D1):** `AsyncLlmRunner` gated `llm-driver` — solo trait, sin executor (tokio NO se agrega; server lo adapta vía spawn_blocking en MEM-16/35).
4. **Degradación LLM-free:** sin `llm-driver` el StandaloneLlmRunner devuelve `LlmError::NotConfigured` (nunca bloquea); callers degradan (store-all dedup heurística). Sin feature el crate NO trae reqwest.
5. **Scope offload:** SOLO OffloadEntry/ToolPair/PluginState (MEM-20 cursor + after-tool-call). MmdMetadata/MmdNode/TaskJudgment/L15Boundary = F5 (MEM-22/24) — no se inventan hoy.
6. **Meta/enums persona:** SceneMeta{created,updated,summary,heat} + SceneIndexEntry (MEM-12); PersonaMode{First,Incremental} + PersonaTriggerPriority{P1..P4,P2Recovery} (MEM-15) — citados en 02 §41, §52-53. Strategy UPDATE>MERGE>CREATE = MEM-14 (no hoy).