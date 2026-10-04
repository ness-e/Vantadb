---
title: "TASK DUR-03: H-023 — put sobre key expirada-sin-purgar (Node ID collision)"
kind: task
description: "Fix: put/put_batch/put_record_exact sobre una key expirada-sin-purgar purga la entrada física (purge-on-write) y escribe fresco — fin del NodeIdCollision; regresión expirado→put→ok + race con sweeper cubiertos"
---

# TASK DUR-03: H-023 — `put` sobre key expirada-sin-purgar muere con "Node ID collision"

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 10, Wave F0)
- **Fuente:** `docs/dev/Backlog.md` (fila DUR-03, fuente H-023) + plan Task 10
- **Esfuerzo:** 🟡 1-2d | **Appetite:** 2d
- **Prioridad:** 🟠
- **Tipo:** Rust core (write path de `src/sdk/api/memory.rs` + lógica de expiración/purge)
- **Turns estimados:** 15-25
- **Creado:** 2026-10-04T07:36Z | **last-synced:** 2026-10-04T10:20Z
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 — resueltas en Discovery (causa raíz localizada por código + semántica decidida con evidencia; ver §Decisión y §Fase 1)
- **Pendientes (downhill):** 1 step (Steps 1-4 ✅; Step 5: re-review ronda 2 + commit)
- **Campaign ID:** master-plan-0.9.0-20261004

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Embedded::put`/`put_batch`/`put_record_exact` son la superficie de escritura del SDK: los consumen TODOS los bindings (`vantadb-python`, `vantadb-node`, `vantadb-wasm`, `vantadb-ts` vía wasm) y el server HTTP (`vantadb-server`) — el cambio es de comportamiento interno del write path, sin firma nueva |
| Callees | `engine.get` (lectura física), `record_from_node` (`src/sdk/serialization/mod.rs:331` — lazy TTL), `memory_record_from_node_include_expired` (mismo archivo, `pub(crate)`), `engine.delete` (`src/storage/engine/delete.rs:19`), `version_history::purge_key` (`src/sdk/version_history.rs:342`), `replace_derived_indexes` (`src/sdk/serialization/impl_index.rs:154` — derived + text + sparse) |
| Implicaciones | **Contrato público:** `put` sobre expirado deja de fallar (bugfix); keys vivas SIN cambio (mismo upsert vN+1); `get`/`delete`/`list`/search sobre expirado SIN cambio (lazy TTL). **Concurrencia:** `Embedded` gana `purge_lock: Arc<Mutex<()>>` (`builder.rs`) que serializa `purge_expired` vs purge-on-write (hallazgo del test de race: el doble decremento de text df hacía fallar al sweeper). **Performance:** el branch nuevo solo corre cuando hay nodo físico expirado (caso frío — el sweeper normalmente purga antes); path caliente intacto. **Durabilidad:** purge-on-write usa los mismos primitivos que `purge_expired`/`delete_inner` (WAL tombstone + backend delete + index ops). **Migración de datos:** ninguna. **Tests existentes:** `delete_expired_ttl_record` (`tests/edge_cases.rs:450`) sigue válido (delete de expirado = false); suite TTL intacta |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, WIP de otros workers (DX-01/WSM-15 editan `vantadb-wasm/` — no tocar wasm; `vantadb-ts/repro-dx01.mjs` ajeno).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o secciones funcionales completas):**
  - `src/sdk/api/memory.rs` — `put_one` (304-418), `put`/`put_batch`/`put_batch_inner` (448-670), `get` (702-717), `delete_inner` (784-810), `put_record_exact` (889-939+), `purge_expired` (1177-1455), `effective_ttl_ms` (175-177)
  - `src/sdk/serialization/mod.rs` — `memory_node_id` (77-83, xxhash3-128 determinístico de `ns\0key`), `record_from_node` (331-333), `memory_record_from_node_include_expired` (344-346), `memory_record_from_node_inner` (348-442, lazy TTL en 433-441)
  - `src/sdk/serialization/impl_index.rs` — `derived_delete_ops` (135-152), `replace_derived_indexes` (154-239), `adjust_derived_index_state_after_replace` (242-279, saturating)
  - `src/sdk/version_history.rs` — `purge_key` (342-356), `write_snapshot`/`versions`/`get_version` (contexto)
  - `src/storage/engine/delete.rs` (completo, 317L) — `delete` (19-60, WAL + HNSW + backend + shred), `apply_delete_stats`, `purge_permanent`
  - `src/storage/engine/insert.rs` — `insert` (155-225), `apply_insert_stats` (234-284: limpia scalar/edges del nodo previo)
  - `src/error.rs` — variante `NodeIdCollision` (207-208), mapeo HTTP (400), test display (633-636)
  - `tests/edge_cases.rs` — sección 14 `delete_expired_ttl_record` (448-468) + imports (1-13)
  - `src/sdk/api.rs` — tests TTL/WIRE-04 (202-370)
  - `.opencode/rules/durability.md`, `.opencode/rules/core-engine.md`, `.opencode/rules/api-contract.md`, `.opencode/references/definition-of-done.md`, `.opencode/references/clean-code-clean-architecture.md` (Apéndice V)
- **Archivos referenciados hacia dentro (imports/dependencias):** `memory.rs` usa `super::super::version_history`, `crate::storage::ops::{deserialize_node_payload, NodeMetadata}`, `crate::text_index::*`, `crate::backend::{BackendPartition, BackendWriteOp}`, `crate::sdk::serialization::{record_from_node, memory_node_id, ...}` — todos ya importados en el archivo (el helper nuevo no agrega imports salvo `memory_record_from_node_include_expired` si no está en el prelude local).
- **Referencias entrantes (grep `put_one|NodeIdCollision|memory_node_id`):** `src/server/errors.rs:50` (mapea `NodeIdCollision` → 400 — sigue existiendo para colisiones reales de hash/foreign node), `src/cli_handlers/db.rs:28` (re-export de `memory_node_id`), tests de bindings que ejercitan put. `purge_expired`: 2 callers en `vantadb-mcp/src/handlers/tools.rs` + server + bindings (no cambian).
- **Veredicto impacto:** **LOCALIZADO, write path únicamente.** Sin cambios de firma pública; sin cambios de formato on-disk; sin migración. El único cambio de comportamiento es el contratado: `put` sobre expirado-sin-purgar pasa de `Err(NodeIdCollision)` a purge-on-write + insert fresco. Riesgo principal: regresión del path caliente (mitigado: branch frío + suite completa) y race con el sweeper (mitigado: test concurrente + primitivos idempotentes).

## Decisión (semántica — documentada antes de codear, pre-mortem #1)

**Elegida: purge-on-write** (la opción que el contrato del plan admite explícitamente como alternativa de upsert).

| # | Opción | Veredicto | Evidencia |
|---|--------|-----------|-----------|
| A | **Purge-on-write**: expirado = lógicamente ausente → purga física del registro expirado (mismos primitivos que `purge_expired`) y escritura fresca (v1, `created_at` nuevo) | ✅ **ELEGIDA** | Es exactamente el workaround documentado (`purge_expired()` → `put`) elevado a comportamiento del write path; semántica ya establecida en el repo: `get` oculta expirados (lazy TTL `serialization/mod.rs:433-441`), `delete` de expirado devuelve `false` (`tests/edge_cases.rs:450-468`), `list`/search excluyen. Redis (referencia de KV store moderno): el timeout expira ⇒ "the key will automatically be deleted"; `SET` "overwrites... regardless of its type. Any previous time to live... is discarded" — un write nunca falla por estado previo |
| B | **Upsert**: conservar el nodo físico, tratar el expirado como versión previa (v2, `created_at` preservado) | ❌ descartada | Resucita historia de un registro lógicamente muerto (`versions()` mostraría v1 de un registro invisible) y difiere del estado observable del workaround aprobado (purge → put ⇒ v1). Además no limpia residuo físico (HNSW/shred/versiones) sin trabajo extra equivalente |

**Rationale largo (por qué A y no B):** el registro expirado es invisible para todas las lecturas; un `put` posterior crea un registro nuevo. La opción A hace que el estado tras `put` sea **idéntico** al estado tras `purge_expired(); put()` — verificable en el test (v1, payload nuevo, sin snapshot v1 duplicado, sin postings viejos). La opción B mantiene dos v1 en `versions()` o exige renumerar, y deja al HNSW/vector-store dependiendo de la semántica de reemplazo same-id del engine para un caso que A evita por completo. Se elige A por consistencia observable y por reutilizar primitivos ya probados (`engine.delete` + `replace_derived_indexes(Some(expired), None)` + `purge_key` — el mismo trío de `delete_inner`).

**Pre-mortem (3 riesgos del plan) — respuesta:**
1. *Cambio de semántica no documentado* → decisión explícita acá + commit `fix(engine):` (release-plz lo levanta como bugfix público en el changelog del release; Regla 7: no tocar CHANGELOG a mano).
2. *Race con el TTL sweeper* → **el test concurrente la encontró de verdad** (no era benigna): con el purge-on-write del writer y el sweeper concurrentes, el segundo decremento de text df en `purge_expired` fallaba con `Validation { field: "stats", reason: "text index df would go negative" }`. Fix (ronda 1): `purge_lock` (Mutex) serializa `purge_expired` y `purge_expired_record`, que re-verifica bajo el lock. Fix (ronda 2, review P2-01): `purge_lock` pasa a **RwLock** — los purge paths toman el guard de escritura y los upserts de registros vivos toman el guard de lectura sobre `resolve → insert → replace` (`put`/`put_record_exact`), cerrando la variante "put resuelve vivo, el deadline vence y un purger limpia la generación en medio" (el re-check post-insert no puede detectarla porque el nodo ya fue sobrescrito). Tests de race (arranques 0/1/2 ms: a veces vivo, a veces expirado) verde + stress 20×. Residuales → FIND-245.
3. *Blast radius al write path (hot)* → branch nuevo solo alcanzable con nodo físico expirado (caso frío); el guard de lectura se toma en upserts de registros existentes (sin contención entre sí; solo espera a un purge en vuelo). Suite `-p vantadb` 2549/2549 verde (shared target) + rerun limpio en target aislado.

**Stop condition del plan ("semántica ambigua → Gate D"):** NO dispara — el plan delega la elección ("upsert o purge-on-write (decisión documentada)") y la evidencia del repo resuelve la ambigüedad sin símbolos públicos nuevos.

## Contrato

"`put` sobre una key expirada-sin-purgar hace purge-on-write → `Ok` (exit 0); test de regresión `expirado → put → ok` verde (y variante `put_batch`); semántica para keys vivas sin cambio (suite existente de TTL/upsert verde); `cargo nextest run --profile audit -p vantadb` verde + fmt/clippy sin warnings nuevos."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `get`/`list`/search sobre un registro expirado-sin-purgar siguen tratándolo como ausente (lazy TTL sin cambio).
  2. `delete` sobre expirado sigue devolviendo `false` (test `delete_expired_ttl_record` intacto).
  3. Upsert de key viva sin cambio: mismo `node_id`, `version` incrementa, `created_at_ms` preservado.
  4. El error `NodeIdCollision` sigue existiendo para colisiones reales (nodo físico de OTRO namespace/key en el mismo id, o nodo no-memory) — no se elimina la variante.
  5. `purge_expired` y el sweeper sin cambio de contrato.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb --test edge_cases` (nuevos tests) + `cargo nextest run --profile audit -p vantadb` (suite del crate) + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings`.
- **Deuda pendiente:** FIND-245 (ventana TOCTOU residual del sweeper — pre-existente; la variante principal re-put-vs-sweeper quedó cerrada por `purge_lock` + re-check). `put_record_exact` se corrige con el mismo helper pero no tiene test de integración propio (es `pub(crate)`, cubierto por el path de import/export existente).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (el helper reutiliza primitivos existentes; el fix elimina un bug de datos visible para usuarios). Sin moneda de pago requerida.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable + tests de regresión (RED→GREEN probado) + fmt/clippy/nextest del área |
| **Commit** | Commit atómico conventional `fix(engine):` + `git diff` limpio + verificación mecánica |
| **Release** | Bugfix público → release-plz lo levanta del commit (CHANGELOG no se toca a mano — Regla 7). `dev-tools/verify.ps1` completo queda para el batch de cierre del plan (push diferido por instrucción del owner) |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius put → id → collision; usado) + `codebase-memory-mcp_check_index_coverage` (cobertura verificada: 6 paths `no_recorded_issue`)
- `cargo nextest run --profile audit -p vantadb --test edge_cases` (loop TDD) / `-p vantadb` (suite del crate) / `--workspace` solo en cierre
- `campaign_verify_cmd` (verify mecánico por step)
- `pwsh dev-tools/ocr-review.ps1` (OCR delegation al cierre)

**Skills cargadas (SDP v3):** `systematic-debugging` (bug — Iron Law; causa raíz antes del fix) · `rust-write-tests` (regresión + naming + concurrency) · `test-driven-development` (RED→GREEN, Prove-It) · `incremental-implementation` (slices) · `context-engineering` (jerarquía de contexto) · `source-driven-development` (semántica Redis vs docs oficiales) · `doubt-driven-development` (base Rust core) · `deprecation-and-migration` (pinned storage/schema — aplicado: sin migración de datos, semántica de expiración) · `documentation-skill` (task file docs/**) · base auto (campaign-executor/progreso/ponytail). SDP: 9 cargadas de 10 (frontend-ui-engineering excluida por no aplicar a Rust core).

## Investigation Notes

- **Causa raíz (código, confirmada por lectura):** `memory_node_id(ns, key)` es un hash determinístico (`serialization/mod.rs:77-83`). `put_one` hace `engine.get(node_id)` → devuelve el nodo **físico**; luego `record_from_node(&node)` aplica **lazy TTL** y devuelve `None` para un registro expirado (`serialization/mod.rs:433-441`). El match en `memory.rs:312-325` conflaciona `None` con colisión:
  ```rust
  Some(node) => match record_from_node(&node) {
      Some(record) if record.namespace == input.namespace && record.key == input.key => Some(record),
      _ => return Err(Error::NodeIdCollision(memory_node_id(&input.namespace, &input.key))),
  },
  ```
  Mismo patrón en `put_batch_inner` (`memory.rs:519-536`) y `put_record_exact` (`memory.rs:920-932`). El workaround documentado (`purge_expired()` antes del seed) funciona porque elimina el nodo físico. Mismo id en 3 corridas = determinismo del hash, consistente con H-023.
- **Semántica de expiración del repo:** "TTL is enforced lazily — expired records are treated as non-existent" (`tests/edge_cases.rs:465-467`). `purge_expired` materializa candidatos por scalar index (`expires_at_ms <= now`) y los borra físicamente con derived/text/sparse/versiones.
- **Primitivos reutilizables para el purge-on-write:** el trío de `delete_inner` (`memory.rs:789-799`): `engine.delete(node_id, ...)` + `replace_derived_indexes(&engine, Some(&existing), None)` + `version_history::purge_key`. `replace_derived_indexes` cubre derived + text postings/stats + sparse (`impl_index.rs:160-237`). `engine.delete` purga KV + HNSW + shred (`delete.rs:19-60`).
- **Web (Redis, docs oficiales verificadas 2026-10-04):** [SET](https://redis.io/docs/latest/commands/set/) — "If `key` already holds a value, it is overwritten, regardless of its type. Any previous time to live associated with the key is discarded on successful `SET` operation."; [EXPIRE](https://redis.io/docs/latest/commands/expire/) — "After the timeout has expired, the key will automatically be deleted" + "A key is passively expired when a client tries to access it and the key is timed out." → un write sobre una key expirada es un write sobre una key ausente; nunca error. (SQLite/LevelDB no tienen TTL nativo — la comparación aplicable es Redis; conceptualmente todos tratan el write como overwrite sin fallo.)
- **Race con sweeper — análisis:** `purge_expired` materializa `to_delete` (lee bytes actuales y re-chequea `now > expires`) y luego borra por id. Ventana: si un re-put purge+inserta entre la lectura y el delete del sweeper, el sweeper puede borrar el registro fresco. Es una ventana TOCTOU **pre-existente** del sweeper (también alcanzable hoy con `delete`+`put` manuales concurrentes) y no la introduce el fix; los counters usan `saturating_*` (sin underflow). Se documenta + test concurrente que garantiza ausencia de errores/corrupción y recuperabilidad.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — (a) causa raíz localizada por código; (b) semántica decidida (purge-on-write, §Decisión); (c) alcance de paths afectados mapeado (3 call sites, helper compartido) |
| Pendientes de ejecución (downhill) | 5 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica como superficie nueva: no hay input nuevo de usuario ni cambios de auth/FFI; el cambio endurece un path existente (un write legítimo ya no muere). Sin dependencias nuevas. El purge-on-write borra datos expirados que el usuario ya no puede leer — no expone nada.
- [x] **PERFORMANCE** — evaluada: el branch nuevo solo se ejecuta con nodo físico expirado (caso frío); el path caliente (key viva / key ausente) no cambia (mismo `engine.get` + un match más). No hay hot path nuevo; no se requiere bench (Regla 9 no dispara: no es una optimización). Impacto esperado: nulo en el caso común; el caso expirado agrega un delete + batch de índices (coste análogo al workaround que ya hacía el usuario).

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** `db.put(ns, key, ttl_ms=1)` → `sleep(5ms)` (registro expirado, sin purgar) → `db.put(ns, key, nuevo payload)` → **`Err(NodeIdCollision)`** en HEAD. Determinístico (hash xxhash3-128 fijo; el sleep solo cruza el deadline). Test RED: `put_after_ttl_expiry_succeeds_as_fresh_insert` + `put_batch_after_ttl_expiry_succeeds` (`tests/edge_cases.rs`, sección 14b).
- **Hipótesis:** `put_one`/`put_batch_inner`/`put_record_exact` conflacionan "nodo físico presente pero lógicamente ausente (lazy TTL)" con "id ocupado por otro registro" — `record_from_node` devuelve `None` y el match cae en la rama `_ => NodeIdCollision` (`memory.rs:317-322`, `531-536`, `927-929`). Fijar: detectar expirado vía `memory_record_from_node_include_expired` + ns/key match, purgar (trío de `delete_inner`) y continuar como insert fresco.
- **1 variable controlada:** el branch `record_from_node == None` con nodo físico memory-record del mismo ns/key → purge-on-write; ningún otro comportamiento cambia.
- **Test RED:** verificado como FALLO antes del fix (Step 1, evidencia transcripta en el step).

## Steps

### Step 1 — Tests de regresión (RED)

- **Archivos:** `tests/edge_cases.rs`
- **Acción:** agregar sección "14b. Re-put after expired TTL (DUR-03 / H-023)": (a) `put_after_ttl_expiry_succeeds_as_fresh_insert` (v1 sin TTL + v2 con TTL 1ms → expirado → get None → re-put Ok v1 payload nuevo → get nuevo → `versions()` solo v1 fresco); (b) `put_batch_after_ttl_expiry_succeeds`. Correr ANTES del fix y verificar que fallan con `NodeIdCollision` (RED).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test edge_cases --ignore-default-filter -E "test(~ttl_expiry)"` → FALLA (NodeIdCollision) en HEAD
- **Evidencia:** ✅ RED verificado en HEAD: `put_after_ttl_expiry_succeeds_as_fresh_insert` → `NodeIdCollision(337403689315191484007250301346772122544)`; `put_batch_after_ttl_expiry_succeeds` → `NodeIdCollision(283395976416762123563240748723681314549)`. (Nota: `edge_cases` está en el `default-filter` de nextest → requiere `--ignore-default-filter` para correr; la primera corrida sin el flag dio "0 tests".)
- **Estado:** ✅ COMPLETED

### Step 2 — Fix: purge-on-write en el write path (GREEN)

- **Archivos:** `src/sdk/api/memory.rs`, `src/sdk/builder.rs`
- **Acción:** helper compartido `resolve_existing_for_write` (vivo match ns/key = `Some` + **guard de lectura**; expirado del mismo ns/key = purge-on-write bajo **guard de escritura** + re-check; otro caso = `NodeIdCollision`) aplicado en `put_one`, `put_batch_inner` y `put_record_exact`; `purge_expired_record` = trío de `delete_inner` (engine.delete + `replace_derived_indexes(Some(expired), None)` + `purge_key`) + re-check + rechazo con transacción activa (`engine.txn.has_active()`: el delete se bufferea y el cleanup de índices no). `purge_lock: Arc<RwLock<()>>` en `Embedded` (3 constructores): purge paths toman write; upserts de registros vivos toman read sobre `resolve→insert→replace` (ronda 2 del review).
- **Verify:** `cargo nextest run --profile audit -p vantadb --test edge_cases --ignore-default-filter -E "test(~ttl_expiry)"` → GREEN; `cargo check -p vantadb`
- **Evidencia:** ✅ 2/2 verdes; `cargo check -p vantadb` sin warnings. `resolve_existing_for_write` + `purge_expired_record` en `memory.rs:302-455`; `purge_lock` en `builder.rs:27-39` + 3 constructores.
- **Estado:** ✅ COMPLETED

### Step 3 — Test concurrente (race con el sweeper)

- **Archivos:** `tests/edge_cases.rs`
- **Acción:** `put_after_expiry_racing_ttl_sweeper_stays_consistent` (24 rondas con arranque del writer a 0/1/2 ms del seed → a veces resuelve vivo, a veces expirado; writer vs `purge_expired()` concurrentes; ambos Ok y el registro fresco SIEMPRE presente, sin fallback) + `put_rejects_foreign_node_at_deterministic_id` (nodo no-memory en el id determinístico → `NodeIdCollision`). **Hallazgo ronda 1:** la race NO era benigna (`text index df would go negative`) → `purge_lock` + re-check. **Hallazgo ronda 2 (review P2-01):** la ventana "resolve vivo → deadline vence → purger limpia" exigía el guard de lectura (RwLock) sobre el upsert.
- **Verify:** `-E "test(~racing_ttl)"` → verde; stress `--stress-count 20` → 20/20
- **Evidencia:** ✅ 4/4 tests nuevos verdes (ttl_expiry×2 + racing + foreign); stress race 20/20 (59.7s, target aislado `target/dur03`). Antes del lock: `Err(Validation { field: "stats", reason: "text index df would go negative" })` en el sweeper (transcripto).
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full scoped + OCR delegation

- **Archivos:** —
- **Acción:** `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + `cargo nextest run --profile audit -p vantadb` (suite del crate); OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) aplicando cada Rule Group a los archivos propios; gates docs del task file (`check-links`/`check-docs`).
- **Verify:** todos exit 0 / sin Critical/High
- **Evidencia:** ✅ `cargo fmt --check -p vantadb` exit 0 · `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 · `cargo nextest run --profile audit -p vantadb` → **2549/2549 passed** (shared target) + **rerun limpio en `CARGO_TARGET_DIR=target/dur03`** (367s) verde vía `campaign_verify_cmd` · `edge_cases` completo → 28 run, 27 passed, 1 failed (único: `all_zeros_vector_put_and_list`, **pre-existente verificado en HEAD vía stash** → FIND-244) · TTL excluidos: `memory_api` (namespace_stats + canonical) 2/2, `fuzz_proptest` ttl 1/1, `quarantine_containment` expired 2/2 · docs: `check-links` 0, `check-docs` 0 (GATING all clear), `gen-index --check` 0 · OCR spec: `src/sdk/api/memory.rs` + `src/sdk/builder.rs` (Rule Group 2 — Rust core + api-contract) y `tests/edge_cases.rs` (Rule Group 3 — Rust genérico) revisados contra sus checklists: **0 Critical / 0 High** (sin unwrap/expect nuevos en prod, sin unsafe, locks sin `.await`, re-check + guard cierran los check-then-act, sin API pública nueva).
- **Nota de entorno:** el target compartido sufrió fingerprints stale dos veces por builds concurrentes de otros workers (binario `edge_cases` con 24 tests vs 28 del source) → resuelto forzando rebuild y verificando con `CARGO_TARGET_DIR` aislado. La verificación final del contrato corre en target aislado.
- **Estado:** ✅ COMPLETED

### Step 5 — Review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/DUR-03.md`
- **Acción:** review adversarial por agente distinto (tier **Adversarial**: `src/sdk/**` — fork a `vanta-review`); registrar veredicto en §Review; commit **LOCAL** `fix(engine): DUR-03 — ...` solo con archivos propios (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Estado:** ⬜ PENDING

## Dependencias

- F0 — sin dependencias (el plan declara "F0 — sin dependencias").
- nextTask: DUR-01.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Tier:** **Adversarial** (paths del diff: `src/sdk/**` matchea la tabla HARD-02 → review adversarial obligatorio por `vanta-review`).
- **Revisor:** ✅ `vanta-review` (contexto fresco — sesión `ses_efa031798ffesZHUdsGIpjy4k1`). Ronda 1 (2026-10-04): **❌ changes-required** (1 High, 2 Medium, 6 Low/Optional). Ronda 2 (delta, misma sesión): **✅ approve**.
- **Enfoque:** ¿el approach purge-on-write es correcto? ¿la race con el sweeper queda cerrada? ¿el cleanup de índices es completo? ¿se preserva la semántica de keys vivas?
- **Reconciliación ronda 1 (clasificación doubt-driven):**
  - **F1 (High) — válido + accionable → CORREGIDO (variante correcta):** el fix literal sugerido (re-resolver post-insert) no funciona — tras `insert` el nodo ya es el nuevo y el re-resolve leería nuestra propia generación; implementado **guard de lectura RwLock** sobre `resolve→insert→replace` en `put`/`put_record_exact` + write guard en purge paths. Test de race extendido (arranques vivos y expirados) + stress 20×.
  - **F2 (Medium) — válido + accionable → CORREGIDO:** `purge_expired_record` devuelve el estado re-chequeado: vivo refrescado → `Some(record)` (upsert real, sin sobrescribir con v1); foreign → `NodeIdCollision`; ausente → `None`.
  - **F3 (Medium) — válido + accionable → CORREGIDO:** con transacción activa (`engine.txn.has_active()`) el purge-on-write no corre (el delete se bufferea y el cleanup sería inmediato → abort dejaría stats ya borradas) y se mantiene el error pre-DUR-03.
  - **F4 (Low) — válido → CORREGIDO (comentarios) + FIND-251:** los doc comments afirmaban "equivalente exacto" al workaround cuando el purge-on-write limpia MÁS (sparse) que `purge_expired`; comentarios corregidos; gap sparse del sweeper → FIND-251.
  - **F5 (Low) — trade-off válido (documentado):** un put que falla validación después de resolver expirado ya purgó el registro — pero el registro era lógicamente ausente (estado observable idéntico: `get`=None antes y después); la pérdida es la de la historia física del workaround aprobado. Aceptado.
  - **F6 (Low) — contrato mal leído + trade-off:** "poisoning" no aplica (parking_lot no envenena — el reviewer asumió `std::sync`); el lock por-handle de `from_engine` es la misma propiedad que `supersede_lock` (precedente del repo) → residual en FIND-245.
  - **F7 (Low) — válido + accionable → CORREGIDO:** test `put_rejects_foreign_node_at_deterministic_id` cubre la rama collision (antes solo code-review).
  - **F8 (Low) — válido → CORREGIDO:** fallback muerto eliminado del test de race (ahora afirma presencia directa); arranques alternados 0/1/2 ms; filtros del task file corregidos.
  - **F9 (Optional) — trade-off documentado:** el sweeper retiene el write guard durante todo el sweep; un put contra una key expirada espera. Aceptado (sweeps periódicos, usualmente vacíos; backlog grande = stall acotado); chunking futuro si molesta.
  - **F10 (Low) — válido → FIND-250 (decisión owner/ADR-046):** TTL vs cuarentena sticky (un registro en cuarentena puede expirar y purgarse; re-put crea uno sin cuarentena).
- **Cómo se probó (evidencia real, no auto-reporte):** 4 tests nuevos verdes (RED transcripto antes del fix con `NodeIdCollision`); stress race 20/20 (59.7s); `edge_cases` 28 run / 27 passed (único fallo = FIND-244, pre-existente verificado en HEAD con stash); suite `-p vantadb` 2549/2549 (shared) + rerun aislado `target/dur03` verde; fmt/clippy exit 0; docs gates exit 0; OCR Rule Groups 2/3 → 0 Critical/High. El revisor de ronda 1 re-ejecutó por su cuenta: tests 3/3 + stress 10/10 + suite 2549/2549 + fmt/clippy + inspección de API pública.
- **Checklist anti-hábitos tóxicos:** sin comandos inventados (todo transcripto); RED verificado como fallo real; fallo pre-existente (FIND-244) investigado en HEAD con stash y NO atribuido al diff; staleness del target compartido diagnosticada y aislada; review delegado a agente distinto (nunca auto-review).
- **Veredicto:** **✅ APPROVE** — ronda 1 ❌ changes-required → corregido (F1-F4, F7, F8) / documentado como trade-off (F5, F6, F9) / FIND (F10 → FIND-250; F4-sparse → FIND-251). **Ronda 2 (delta): ✅ approve** — el revisor re-ejecutó focused 4/4, suite 2549/2549, `memory_api` 9/9, `quarantine_containment` 27/27, `importers` 19/19, `memory_export_import` 10/10, fmt/clippy. Residuales de ronda 2: (a) writer-vs-writer same-key → **FIND-252**; (b) `put_batch` suelta el guard antes del insert → FIND-245 residual (a); (c) refusal por transacción activa en `purge_expired` (`Ok(0)`), (d) present-but-unparseable → `NodeIdCollision` en `purge_expired_record`, (e) nota de excepción del guard en el call site de `put_batch_inner` — **(c)(d)(e) aplicados inline**.

## Notas

- **Hallazgos colaterales (filas en `docs/dev/Backlog.md`):** **FIND-244** — `all_zeros_vector_put_and_list` contradice el filtrado `usable_vector` (rojo determinístico, verificado en HEAD sin los cambios DUR-03 vía stash; rota en silencio por el default-filter). **FIND-245** — residual TOCTOU del sweeper para paths sin generation guard (delete_inner/batch/import, multi-handle, delete sin CAS). **FIND-250** — TTL vs cuarentena sticky (ADR-046): decisión de owner pendiente. **FIND-251** — `purge_expired` no limpia sparse (el purge-on-write de DUR-03 sí).
- **Nota de entorno:** se mataron 2 procesos `vanta-cli` colgados (debug, 2h+) que bloqueaban `target/debug/vanta-cli.exe` (clase del incidente `:8080`); el target compartido produjo fingerprints stale en 2 corridas por builds concurrentes de otros workers → verificación final con `CARGO_TARGET_DIR=target/dur03`.
- `put_record_exact` es `pub(crate)` (transporte de import) — el helper se aplica por consistencia; su rama collision queda cubierta por el test de foreign-node (que ejercita `put`).
- Release: el commit `fix(engine):` entra al changelog del próximo release vía release-plz (Regla 7 — no se edita CHANGELOG a mano).
- WIP ajeno en el árbol (no stage): `opencode.jsonc`, plan file, `FIND-233.md`, `DX-01.md`, `WSM-15.md`, `DUR-01/02.md`, `vantadb-wasm/**`, `vantadb-ts/**`, `docs/index.md`, `llms.txt`.

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ⬜ (se completa al cierre)
```
