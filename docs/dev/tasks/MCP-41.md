# MCP-41 — Memoria conversacional auto-consolidada (DISCOVERY-first)

> **Plan:** `docs/dev/plans/2026-09-10-code.md` Task 17 · **Campaign:** 2c3d4e5f-6a7b-8c9d-0e1f-2a3b4c5d6e01
> **Ruta:** vanta-arch (diseño) → vanta-worker (slice) · **Estado:** ⏳ IN PROGRESS
> **Rama:** `develop` (verificado `git branch --show-current`)
> **SDP:** campaign-executor, spec-driven-development, interview-me, idea-refine,
> vanta-design-orchestrator, impeccable, source-driven-development,
> security-and-hardening + manual arch: documentation-and-adrs,
> api-and-interface-design, database-design, doubt-driven-development

## Spec (spec-first gate — feature-add sin spec no entra a ACT)

| # | Decisión | Opciones | Evidencia / default |
|---|----------|----------|---------------------|
| 1 | Qué es "auto-consolidación" aquí | (a) extract→consolidate→recall local-first sin LLM key **(Recomendado)** / (b) wrapper sobre LLM runner existente / (c) pipeline completo mem0/graphiti | (a): `extract_scenes_with_llm` exige `LlmRunner` (`scene_extractor.rs:353`); dream `consolidate_session` es LLM-free salvo runner opt-in (`dream/mod.rs:672-685`); `scene_query` keyword-only ya es local-first (`scenes.rs:9-10`). Brecha real = solo `extract`. |
| 2 | Dónde vive el slice | (a) `vanta-memory/src/core/scene/auto_consolidate.rs` nuevo + re-export en `mod.rs` **(Rec.)** / (b) módulo top-level `core/conversation/` / (c) dentro de `scene_extractor.rs` | (a): reutiliza `decide_strategy`/`apply_strategy`/`extract_scenes` + `normalize_scene_name` sin tocarlos; blast radius = 1 archivo nuevo + 1 línea mod. |
| 3 | Heurística de `extract_local` | (a) chunk por turnos con nombre por términos significativos ordenados **(Rec.)** / (b) un scene por turno / (c) ventana deslizante por tokens | (a): reutiliza `significant_terms` (`l1_reader.rs:99`, `pub(crate)`) ordenados (HashSet → sort, determinismo); fallback `turn-{n}`; sin merge_sources (local no juzga overlap — documentado). |
| 4 | Nuevo MCP tool `scene_consolidate` en este slice | (a) NO — diseño en ADR, follow-up **(Rec.)** / (b) sí ahora | (a): añadir tool 77 rompe conteos (`config.rs:11`, `tools.rs:25`, `mcp_tests.rs:4469/4577`) y perfiles; recall ya expuesto (`scene_read/list/query`). Ponytail: mínimo código. |
| 5 | recalls / consolidate posterior | reutilizar existente sin cambios | `scene_query` keyword + `consolidate_session` dream LLM-free ya cubren; tests existentes los fijan. |
| 6 | Errores | reutilizar `SceneExtractorError`, sin enum nuevo | Un solo formato (api-and-interface-design §2); `#[non_exhaustive]` ya presente. |

**Contrato del slice:** `auto_consolidate(db, session_key, turns)` determinista, sin LLM,
sobre el store existente (heat CREATE=1 / UPDATE+1 vía `extract_scenes`) + recall vía
`scene_list`/`scene_query` existentes + `cargo test -p vanta-memory` 0 failed + clippy 0.

**Stop condition del plan:** DISCOVERY sin diseño viable local-first → DEFER con diseño
documentado. **Veredicto DISCOVERY: diseño viable SÍ existe** (solo falta `extract`;
todo lo demás es LLM-free y testeado) → se implementa el slice, no se difiere.

## Impacto mapeado (Regla 0 — MUST antes de la primera edición)

**Archivos leídos completos:**
- `vanta-memory/src/core/scene/mod.rs` (39L) — re-exports; se añade 1 `pub mod` + re-export.
- `vanta-memory/src/core/scene/scene_index.rs` (343L) — `upsert_scene`/`list_scenes`/`get_scene`; NO se modifica.
- `vanta-memory/src/core/scene/scene_extractor.rs` (607L) — `SceneExtraction`, `decide_strategy`,
  `apply_strategy`, `extract_scenes`, `SceneExtractorError`; NO se modifica (solo se llama).
- `vanta-memory/src/core/scene/scene_tools.rs` (249L) — límites `MAX_*_BYTES`; NO se modifica.
- `vanta-memory/src/core/scene/scene_format.rs` (150L) — `SceneBlock::new`; NO se modifica.
- `vanta-memory/src/core/scene/filename_normalizer.rs` (198L) — `normalize_scene_name`; NO se modifica.
- `vanta-memory/src/core/dream/mod.rs` (1015L parcial) — `consolidate_session` LLM-free; NO se modifica.
- `vantadb-mcp/src/scenes.rs` (185L), `handlers/tools.rs` (dispatch + perfiles), `lib.rs` — NO se tocan
  (decisión Spec #4; conteos 76 tools intactos).
- `vanta-memory/src/core/record/l1_reader.rs:99` — `significant_terms` `pub(crate)`; NO se modifica.
- `vanta-memory/src/core/conversation/l0_recorder.rs` (`L0Message`) — solo referencia de forma; NO se modifica.

**Referencias hacia dentro (lo que el slice usa):**
`extract_scenes` ← `decide_strategy`+`apply_strategy` ← `execute_scene_tool`/`upsert_scene`/`soft_delete_scene`;
`normalize_scene_name`; `significant_terms`; `SceneExtraction`/`SceneExtractionResult`/`SceneExtractorError`.

**Referencias entrantes (quién depende de lo tocado):**
`scene/mod.rs` es importado por `pipeline_worker.rs:34`, `gateway/knowledge_handlers.rs`,
`auto_recall.rs:36`, tests (`generation_log.rs`, `scene_tests.rs` vía `upsert_scene`).
Añadir un `pub mod` + re-export no rompe ningún import existente (aditivo puro).

**Veredicto:** blast radius = 1 archivo NUEVO + 2 líneas en `mod.rs`. Sin ciclos nuevos
(`auto_consolidate` → `scene_extractor`/`scene_index`/`l1_reader`, todos ya acíclicos hacia abajo;
`cargo modules` no disponible en runner, verificado por inspección de `use`). Sin feature gates
tocados. API aditiva (`pub fn auto_consolidate`, `pub fn extract_local`, `pub struct LocalTurn`)
con `#[non_exhaustive]` donde aplique. **APTO para ACT.**

## Diseño (vanta-arch)

**Pipeline local-first:** `LocalTurn{role,content}[]` → `extract_local` (chunk+nombre heurístico,
truncado a `MAX_*_BYTES`, skips deterministas) → `extract_scenes` (UPDATE>MERGE>CREATE + heat,
ya testeado) → recall existente (`scene_list`/`scene_query` keyword, `perform_auto_recall`
keyword mode). Consolidación profunda L1 opcional vía `consolidate_session` (dream, LLM-free,
`dream/<s>/<run_id>`, originales intactos) — documentada en ADR-040, no cableada (fuera de appetite).

**Hyrum surface:** nombres `turn-{n}` o `term1-term2-...` son heurísticos y NO se garantizan
estables entre versiones (documentado); orden de `list` = heat desc (garantizado por `list_scenes`);
`SceneExtractionResult` shape reutilizado sin cambios.

**Seguimiento MCP:** `scene_consolidate(session_key, turns[])` diseñado en ADR-040 como
follow-up (validación en boundary MCP con `validate_identifier`/`validate_payload`, perfil Full,
conteos 76→77 + tests). No implementado en este slice.

## Steps

- [x] Step 0 — DISCOVERY + Spec + Regla 0 + task file (este archivo)
- [x] Step 1 — GREEN: `auto_consolidate.rs` (`LocalTurn`, `extract_local`, `auto_consolidate`) + re-export `mod.rs` (8 tests nuevos)
- [x] Step 2 — VERIFY mecánico: `cargo test -p vanta-memory --tests -j 2` 0 failed (336 lib + todas las suites) + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 + `cargo fmt -p vanta-memory --check` OK (1 warning propio corregido + 3 diffs fmt aplicados)
- [x] Step 3 — CLOSE parcial: ADR-040 escrito; task file verificado; `git add` solo-propio (4 paths);
  **commit BLOQUEADO por hook**: `cargo fmt --all --check` del pre-commit falla SOLO en
  `vanta-proxy/src/context.rs` + `server.rs` (WIP ajeno de PRX-13 en paralelo, sin formatear).
  Mis 4 paths están fmt-limpios (sin diffs propios en el hook; clippy ok, actionlint ok).
  Sin `--no-verify` (Regla 1). Staged preservado para RESUME cuando PRX-13 corra `cargo fmt`.

## Verificación (evidencia)

- `cargo test -p vanta-memory --lib core::scene::auto_consolidate -j 2` → 8 passed, 0 failed
- `cargo test -p vanta-memory --tests -j 2` → 0 failed en todos los targets (lib 336 + suites: dreaming 7, e2e 6, scene 9, scene_strategy 17, scene_tools 13, recall 14, …)
- `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` → 0 warnings
- `cargo fmt -p vanta-memory -- --check` → limpio tras `cargo fmt -p vanta-memory`
- Review 5 ejes (code-review-and-quality): Approve — aditivo, canónico, sin ciclos, boundary validado.
- `git status`: solo-propios stageados; WIP ajeno (`.opencode`, `opencode.jsonc`, `Investigacion-plan.md`, `vanta-proxy/*` de PRX-13 paralelo, `docs/dev/Backlog.md`, `docs/dev/avance/*`, `desktop/Cargo.lock`) intacto sin stagear; plan file edit inline sin stagear (untracked, precedente PRX-06).
