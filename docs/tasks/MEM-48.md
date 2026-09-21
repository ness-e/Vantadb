# MEM-48 — Compresión consume scores L1 reales

**Plan:** `docs/plans/2026-08-22-vanta-final-cierre.md` · Task 6 · Estado inicial ⬜ PENDING
**Crate:** vanta-memory · **Ruta:** vanta-worker · Independiente

## Objetivo
El score de reemplazo de un mensaje en la cascada de compresión = max(priority de sus memories vinculadas) vía `source_message_ids`, con índice HashMap precomputado (anti O(n²)). Mensajes sin memories → heurístico actual como fallback (ToolResult=6>ToolCall=5>Assistant=4>User=2 + bonus edad).

## Impacto mapeado (Regla 0)
- **Leídos completos:** `vanta-memory/src/context_engine/compressor.rs` (157L), `engine.rs` (476L), `token_estimator.rs` (build_units), `context_engine/types.rs` (ChatMessage.id), `context_engine/mod.rs` (re-exports), `core/abstractions/types.rs` (MemoryRecord), `core/record/l1_reader.rs` (read_session_records), `services/pipeline_worker.rs` (único caller de assemble_with_recall).
- **Referencias hacia dentro:** `score_message` es pub-use en mod.rs; callers: `unit_score` (engine.rs:296-301). `assemble_with_recall`: 1 caller externo → `pipeline_worker.rs:448`.
- **Referencias entrantes nuevas:** `read_session_records` + `build_memory_scores` desde pipeline_worker (best-effort).
- **Veredicto:** cambio contenido a context_engine + 1 caller. Sin deps nuevas. Sin tocar core vantadb.

## Steps
- [x] S1 — compressor.rs: `MemoryScoreMap` (HashMap msg_id→priority máx) + `build_memory_scores(records)` + `score_message(msg, position, total, scores)` con override por id y fallback heurístico intacto. Mapeo prioridad→replaceability: replaceability = 10 − clamp(priority,0,100)/10 (prioridad ALTA ⇒ MENOS reemplazable; priority −1 strict ⇒ clamp 100 ⇒ 0, nunca comprimido).
- [x] S2 — engine.rs: thread `Option<&MemoryScoreMap>` por assemble → mild_cascade → unit_score; assemble_with_recall gana el parámetro. mod.rs exporta build_memory_scores/MemoryScoreMap.
- [x] S3 — pipeline_worker.rs: best-effort `read_session_records` → `build_memory_scores` → `Some(&map)` a assemble_with_recall (error de lectura → None + tracing warn, nunca rompe assembly).
- [x] S4 — Tests D19: (a) mensaje con memory priority 90 sobrevive la cascada antes que uno con priority 10 (`memory_priority_90_survives_before_10`, compressor.rs); (b) sin memories vinculadas → score heurístico byte-idéntico al actual (`fallback_heuristic_without_memory_scores`); (c) System vinculado sigue excluido — cubierto por el early-return `System → None` en score_message (previo al lookup del mapa) + aserto existente en `score_orders_roles_and_age`. ⚠️ Los ✅ de S1-S5 en la sesión anterior eran FALSOS: la sesión colgó antes de verificar.
- [x] S5 — Verify mecánico exit 0 (REAL, re-ejecutado esta sesión): cargo check --all-targets ✅ · nextest 472/472 ✅ · fmt --check ✅ · clippy -D warnings ✅.

## Recitación §3 (honesta — corrección post-sesión-rota)

**Estado previo:** sesión anterior marcó S1-S5 ✅ sin verify real. Estado real al retomar: 5 errores de compilación.

**Errores encontrados y causa raíz de cada uno:**
1. **E0603 `module types is private`** (compressor.rs:21): importaba `crate::core::abstractions::types::MemoryRecord` por el módulo PRIVADO en vez de la re-export pública `crate::core::abstractions::MemoryRecord` que usa todo el resto del crate. Fix: usar la re-export pública (root-cause: convención de imports violada).
2. **E0061 ×3** (compressor.rs tests :191/:192/:195): `score_message` llamado con 3 args (firma vieja) tras agregar `memory_scores: Option<&MemoryScoreMap>`. Fix: pasar `None`.
3. **E0061 ×1** (mmd_injector.rs:103): `assemble` llamado con 5 args (firma vieja de 5) tras agregar `memory_scores`. Fix: pasar `None`.
4. **Descubiertos en re-check**: otros 11 callers de test con firma vieja — `tests/context_engine.rs` (8× assemble + 2× assemble_with_recall) y `tests/e2e_flow.rs` (1× assemble_with_recall). Fix: `None` como arg final.
5. **cargo fmt**: engine.rs:14 orden de imports. Fix: `cargo fmt`.

**Lección:** marcar ✅ solo con comando de verify ejecutado y exit 0 en pantalla. `cargo check -p X` NO compila `tests/` ni integration targets — usar `--all-targets` SIEMPRE para detectar callers rotos en tests.

## Contrato de verificación
`cargo check -p vanta-memory` && `cargo nextest run -p vanta-memory` && `cargo fmt --check` && `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0.

## Context Save Point
- Decisión clave: el mapa guarda la priority RAW máxima (i32); la conversión a replaceability ocurre en lookup — monotónica decreciente, max(priority) ⇔ min(replaceability).
- Suite previa: 470/470 vanta-memory → 472/472 con los 2 tests D19 nuevos. Sin commits durante la tarea (regla del orquestador).
