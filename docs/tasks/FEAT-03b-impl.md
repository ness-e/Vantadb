# FEAT-03b-impl — Core decay: implementación (supersession durable, ADR-028)

> Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 17, alcance D16 "b") · Estado: ✅ COMPLETO (commit `28a1788d`; verify del lead 2026-08-20)

## Verify REAL del lead (post-delegación, 2026-08-20)
- Primera delegación CANCELADA a mitad de camino → trabajo incompleto en worktree (NO daño). El lead completó 5 call sites (mcp x3, wasm x2) + cargo fmt; relanzó el sub-agente que terminó struct export + constructores de tests + tests del contrato.
- `cargo test -p vantadb` — **1803 passed, 0 failed** (incluye supersede/filtros/roundtrip/backward-compat)
- `cargo test -p vantadb --test memory_api --test sdk_serialization --test proptest_serialization_roundtrip` — 42 passed
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — EXIT=0
- `cargo fmt --all -- --check` — EXIT=0
- pytest: tests nuevos 2/2 aislados (supersede_smoke + missing/same-key errors); suite completa 23 fallos = TODOS "Memory pressure" (OOM conocido de esta máquina, no defectos)
- Commit: `feat: FEAT-03b — core decay como supersession durable (ADR-028)` — 35 archivos, 653 insertions

## Contexto (verify del lead, 2026-08-20)
- ADR-028 (accepted): supersession durable first-class, NO scoring/deleción automática. TTL duro ya existe; falta el marcador `superseded_by`/`superseded_at_ms` + `supersede()` + `exclude_superseded` en search/list.
- FEAT-03a (UI) commiteada — la lente CONSOLIDAR ya escribe `metadata.superseded_by` (vía put genérico); esta tarea core añade el campo first-class + filtro de lectura.
- UI de consolidación no depende de esto para su MVP, pero el campo core la hace filtrable/consistente entre clientes.

## Contrato (del plan + ADR-028 + contrato FEAT-03b — leer TODOS antes de tocar código)
1. **Campos core:** `superseded_by: Option<String>` + `superseded_at_ms: Option<u64>` en `VantaMemoryRecord`; `exclude_superseded: bool` en `VantaMemorySearchRequest` y `VantaMemoryListOptions` — aditivos `#[serde(default)]`, campos `FIELD_SUPERSEDED_BY`/`FIELD_SUPERSEDED_AT_MS` (`__vanta_` pattern), verificar `validate_metadata` rechaza `__vanta_` (si no, extender).
2. **API:** `supersede(namespace, old_key, new_key)` en `VantaEmbedded` — valida existencia de ambos, idempotencia (old ya superseded → error), reusa put/upsert (WAL + derived indexes consistentes), `ponytail:` comentario de ventana no-atómica (2 appends WAL, ACID Phase 0).
3. **Filtro lectura:** `exclude_superseded` en search (materialización de hits) y list — drop si `superseded_by.is_some()`, en ensamblaje final (sin cambio de índice).
4. **Python:** `supersede` + getters `superseded_by`/`superseded_at_ms` + `exclude_superseded` en search/list + wrappers async.
5. **CLI (opcional/cortable):** exponer campos nuevos en get/list si es barato.
6. **NO desktop/ changes** (FEAT-03a ya commiteada, disjunto). P2 `supersedes` en `VantaMemoryInput` cortable — cortar si complica put.

## Verificación (contrato)
- `cargo test` core (serialization roundtrip, backward compat, supersede(), search/list filter) — exit code real.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit code real.
- `cargo fmt --check` — exit code real.
- Python smoke: `pytest vantadb-python/tests/test_sdk.py` (o el comando que use el repo — verificar) — exit code real.
- Mecánico del lead post-delegación obligatorio. NO reportar PASS sin verlo.

## Contrato del plan (repetido para el RESULTADO)
- Supersession durable first-class en core + `supersede()` + filtro `exclude_superseded` — API surface EXACTA del contrato.
- Backward compatible (serde default), sin migración, sin worker background, sin cambio de scoring.
- Tests + clippy + fmt + Python smoke verdes.