# MEM-42 — Reclaimer GC de artefactos offload

**Plan:** docs/plans/2026-08-21-vanta-context-engine.md · Task 8 · Wave 2
**Contrato:** tests D19: (a) entradas más viejas que retention_days se eliminan; (b) retention_days < 3 desactiva el reclaimer (paridad TDAM); (c) el cursor lastOffloadedToolCallId nunca apunta a entradas GC-eadas; (d) GC LLM-free e idempotente ante crash.
**Verificación:** `cargo check -p vanta-memory` + `cargo nextest run -p vanta-memory` + fmt/clippy.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vanta-memory/src/offload/state_manager.rs` (197L) — `OffloadStateManager`: load/save state, cursor `last_offloaded_tool_call_id` (sanitizado al guardar), ns `offload_state/<session>`, `OffloadError` (thiserror, `#[non_exhaustive]`).
- `vanta-memory/src/offload/storage.rs` (172L) — `OffloadStorage`: `append_entry`/`read_entries`/`has_entry`, entradas keyed por `sanitize_key(tool_call_id)` en ns `offload/<session>` (helper privado `entries_namespace`), payload = JSON de `OffloadEntry`.
- `vanta-memory/src/offload/types.rs` (144L) — `OffloadEntry { timestamp: String (ISO), tool_call_id, ... }`; NO se puede cambiar schema (stop condition del plan).
- `vanta-memory/src/offload/mod.rs` (20L) — declaración de módulos.
- `vanta-memory/Cargo.toml` — sin chrono/time; **deps nuevas prohibidas**.
- TDAM ref `MemoryCore/src/offload/reclaimer.ts` (:1-120) — retentionDays < 3 desactiva (:75-78), steps independientes try/catch, stats de retorno.
- SDK: `VantaEmbedded::delete(namespace, key) -> Result<bool>` existe (`src/sdk/api.rs:503`), idempotente (false si no existe).

**Referencias hacia dentro (entrantes):** ninguno aún — módulo nuevo. Callers futuros: API manual/orquestación (MEM-16 timer post-F5).

**Referencias hacia afuera (salientes):** `OffloadStateManager::last_offloaded_tool_call_id`, `OffloadStorage::read_entries`, `entries_namespace` (privada → se hará `pub(crate)`), `sanitize_key`, `db.get/delete`.

**Pre-mortem aplicado:**
1. GC borra data que L1 va a procesar → solo se GC-ean entradas ESTRICTAMENTE PRE-CURSOR (timestamp < timestamp de la entrada apuntada por el cursor). La entrada del cursor jamás se borra → test (c) trivialmente satisfecho y sin ambigüedad de orden.
2. mtime no existe en record store → se usa `entry.timestamp` (ISO heredado del tool result), parseado a epoch secs con days-from-civil (sin deps). Timestamp no parseable → skip conservador (nunca GC-ear lo que no se puede datar).

**Veredicto:** impacto BAJO — archivo nuevo + 1 cambio de visibilidad (`entries_namespace` pub(crate)) + 1 línea en mod.rs. Sin cambios de schema, sin deps nuevas, sin tocar core `vantadb`.

## Steps

### Step 1: reclaimer.rs — esqueleto + time helpers + gate de desactivación ✅ COMPLETADO
- [x] `OffloadReclaimer::new(db)`, `MIN_RETENTION_DAYS = 3`
- [x] `reclaim(session_id, retention_days)` (usa SystemTime::now) + `reclaim_as_of(..., now_secs)` inyectable para tests
- [x] `iso_to_epoch_secs` (days-from-civil, Hinnant) con tests propios — TZ marker obligatorio (naive → None, conservador)
- [x] Test (b): retention_days < 3 → no-op, deleted=0

### Step 2: lógica GC + tests D19 (a)(c)(d) ✅ COMPLETADO
- [x] Criterio: `epoch(ts) < epoch(ts_cursor)` AND `epoch(ts) < now - retention_days*86400`
- [x] Cursor ausente / entrada del cursor ausente / ts no parseable → no GC (conservador)
- [x] Test (a): viejas pre-cursor se eliminan, recientes no
- [x] Test (c): entrada del cursor sobrevive; post-cursor sobrevive aunque sea vieja
- [x] Test (d): segunda pasada → 0 deleted (idempotente); LLM-free (sin runner)
- [x] mod.rs: `pub mod reclaimer;`

### Step 3: Verify mecánico completo ✅ COMPLETADO
- [x] `cargo check -p vanta-memory` — ✅ (solo warnings pre-existentes de core `vantadb`)
- [x] `cargo nextest run -p vanta-memory` — ✅ 416/416
- [x] `cargo fmt --check` — ✅ / `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — ✅ exit 0

## Correcciones sobre el trabajo parcial heredado

1. `storage.rs`: `entries_namespace` → `pub(crate)` (reclaimer la necesita para delete-by-key).
2. `mod.rs`: declarado `pub mod reclaimer;`.
3. Tests: `OffloadStorage::new(&db)` → `db.clone()` (3 lugares; `new` toma por valor).
4. `iso_to_epoch_secs`: timestamps SIN TZ marker ahora se rechazan (`None`) — antes se aceptaban como UTC, contradiciendo su propio test; conservador = nunca GC-ear lo indatable.
5. Test `now`: `NOW_DAY_30 = days(30)` era día 30 desde 1970 pero las entradas son de 2026 → cutoff anterior a todo → nada se borraba. Reemplazado por `now()` = epoch real de `2026-08-31T00:00:00Z`.
6. Test `post_cursor_entries_survive_even_when_stale`: sembraba una entrada con timestamp ANTERIOR al cursor esperando supervivencia — contradice el criterio timestamp-based del diseño (no hay orden de inserción en el store). Corregido: post-cursor POR TIMESTAMP (Aug 20 > cursor Aug 5) sobrevive aunque pase el cutoff.

## Context Save Point

Trabajo completo y verificado. Sin commit (instrucción explícita: NO commitear).
Archivos tocados: `vanta-memory/src/offload/{reclaimer.rs (nuevo), storage.rs, mod.rs}`.
