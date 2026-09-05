# Task MEM-40 — Recall scope híbrido (session|agent|team, default agent)

Plan: `docs/plans/2026-08-21-vanta-context-engine.md` · Task 2 · Estado: ✅ COMPLETED

## Impacto mapeado (Regla 0)

**Archivos leídos completos (codegraph verbatim / Read):**
- `vanta-memory/src/core/hooks/auto_recall.rs` (482L) — `perform_auto_recall`:160 usa `read_session_records` (session-only); `RecallConfig`:84; `AutoRecallParams`:231 (`isolation: Option<ProfileIsolation>`); `search_keyword`:250 (keyword-overlap LLM-free); `RecallError`:114 non_exhaustive.
- `vanta-memory/src/core/record/l1_reader.rs` (208L) — `l1_namespace`:21 = `l1/<sanitized>`; `read_session_records`:26 pagina vía `db.list` y deserializa `MemoryRecord`; `MemoryRecord` ya lleva `team_id`/`agent_id: Option<String>`.
- `src/sdk/search/multi.rs` (89L, SOLO LECTURA) — `search_multi`:20 merge por score desc + truncate global top_k; sin covering tests previos.
- `src/sdk/serialization/vector_types.rs` — `VantaMemorySearchRequest` (text_query Option → BM25 sin vectores OK), `VantaMemorySearchHit{record,score}`.
- `vanta-memory/src/core/profile/profile_sync.rs` — `ProfileIsolation{team_id,agent_id}` Default=("default","default").
- `vanta-memory/tests/recall.rs` (344L) — helpers `db()` (InMemory), `record()`, `put_l1()`.
- `src/sdk/api.rs` `list_namespaces`:570 — scan del partition NamespaceIndex (O(#namespaces)).

**Referencias entrantes:** `RecallConfig` 6 callers + tests recall/e2e_flow (campo nuevo con Default → constructores con `..default()` intactos); `read_session_records` 13 callers (firma intacta, extrajo cuerpo a `read_namespace_records`); `perform_auto_recall` tests e2e_flow+recall (own-session records SIEMPRE visibles → tests existentes verdes sin tocarlos).

**Veredicto:** cambio aditivo acotado a `vanta-memory` (4 archivos). Core `vantadb` intacto. Riesgo fuga cross-agente mitigado por test (d) gate.

## Diseño implementado (D22)

- `RecallScope { Session, Agent, Team }`, serde snake_case, **default Agent**, campo `scope` en `RecallConfig`.
- Pool = registros de la sesión actual (SIEMPRE, sin filtro — retrocompat legacy) + si scope≠Session, registros de otros namespaces `l1/*` cuyo `agent_id` (Agent) o `team_id` (Team) matchee `params.isolation`. Records sin metadata en otras sesiones: NO visibles cross-sesión (fallback pre-mortem #2).
- Enumeración: `db.list_namespaces()` → prefijo `l1/` → `read_namespace_records` (nueva fn en l1_reader.rs, extracción del cuerpo de `read_session_records`). Post-filtro por metadata del record deserializado.
- Desviación documentada vs nota del plan: el ranking sigue siendo `search_keyword` (keyword-overlap LLM-free determinístico; records L1 no llevan vector). `search_multi` cubierto por test dedicado (e) — primer covering test del contrato.
- **Pre-mortem perf medido (probe desechable, no commiteado):** setup 500 sesiones × 1 record = 1.10s; recall scope=Agent = **22ms** → stop condition del plan (índice sesiones-por-agente) NO aplica. Techo documentado con comentario ponytail en `read_scoped_records`.

## Steps

- ✅ S1: `RecallScope` enum + campo en `RecallConfig` (auto_recall.rs)
- ✅ S2: `read_namespace_records` en l1_reader.rs + delegación de `read_session_records`
- ✅ S3: pool híbrido en `perform_auto_recall` (`read_scoped_records`) + re-export `RecallScope` en hooks/mod.rs
- ✅ S4: tests D19 (a)-(e) en tests/recall.rs — 14/14 pass
- ✅ S5: verify mecánico + probe perf + cierre

## Verificación

| Comando | Resultado |
|---|---|
| `cargo check -p vanta-memory` | ✅ exit 0 |
| `cargo nextest run -p vanta-memory` | ✅ 378/378 (373 previos + 5 nuevos D19) |
| `cargo fmt --check` | ✅ exit 0 |
| `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` | ✅ exit 0 |

## Notas / deuda

- **Colateral pre-existente (NO de esta tarea):** `cargo clippy -p vanta-memory --all-targets -- -D warnings` SIN `--no-deps` falla por 7 `unused_unsafe` pre-existentes en `vantadb` core `src/storage/vfile_mmap.rs:73,112` (commiteados en HEAD, fuera del dominio worker — propiedad Arch/Engine). Con `--no-deps` el gate pasa limpio. Delegar fix a vanta-arch/vanta-engine.
- Fix colateral WIP: task files stale MEM-08b/MEM-12 (COMPLETED en plan P27 archivado) movidos a `tasks/complete/` + campo Estado corregido — desbloqueó el gate one-task-at-a-time del server MCP.
- Sin commit (regla del orquestador). Sin deps nuevas. Sin unwrap/expect en código nuevo.
