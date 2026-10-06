---
title: "TASK MEMG-05: Multi-escritor — LWW explícito + detección de conflicto (escenario federación v1)"
kind: task
description: "Estrategia multi-escritor declarada con ADR-0055 (LWW explícito + detección de conflicto en empate temporal) + implementación mínima opt-in en el escenario único (escritores que convergen vía merge/import) + tests de concurrencia deterministas + FIND del resto (transporte, bindings, CRDT)."
---

# TASK MEMG-05: Multi-escritor — LWW explícito + detección de conflicto (escenario federación v1)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 57, bloque F4 L1637-1663)
- **Fuente:** plan Task 57 + `docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md:412` ("memoria federada local-first con merge determinista — CRDT-lite sobre UpdateOperations") + `docs/api/MEMORY_INTERCHANGE_FORMAT.md:64` ("Conflicting content for the same key is last-write-wins on import" — sin política declarada ni detección) + `docs/dev/wasm/CRASH_MODEL.md:40` (2 pestañas OPFS = corrupción silenciosa; frontera WSM-15 = exclusión)
- **Esfuerzo:** 🔴 2-3sem | **Appetite:** max 1mes | **Stop (plan L1648):** 2-3sem sin contrato → estrategia declarada + implementación mínima en el escenario único + FIND del resto (réplicas/CRDT) → **este run entrega: ADR-0055 (escenario + estrategia + límites + upgrade) + `Embedded::merge_record` (LWW explícito + detección) + tests de concurrencia deterministas + FIND del transporte/bindings/CRDT**
- **Prioridad:** 🟠
- **Tipo:** Rust core SDK (`src/sdk/` + `src/lib.rs`) + tests de integración + docs (ADR + `docs/api/EMBEDDED_SDK.md` + `MEMORY_INTERCHANGE_FORMAT.md`) + FIND en Backlog
- **Turns estimados:** 10-14 (una sesión de sub-agente con corte declarado)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ EN PROGRESO — DISCOVERY ✅ (task file S0) · Steps S1-S6 pendientes
- **Incógnitas (uphill):** 2 → RESUELTAS en DISCOVERY:
  (a) **Escenario** → fijado: escritores múltiples (device/agente) cuyos registros convergen en un store vía la ruta de merge/import (federación); NO multi-proceso embebido (exclusión por lock `_lock_file`), NO multi-tab OPFS (WSM-15 = exclusión), NO wal-shipping (unidireccional, no multi-escritor — `wal_shipping.rs` replicación sin merge). Evidencia: `CRASH_MODEL.md:40`, `src/storage/engine/mod.rs:375` (`_lock_file`), `MEMORY_INTERCHANGE_FORMAT.md:60-66`.
  (b) **Estrategia** → fijada: LWW explícito (orden total determinista) + detección de conflicto en empate temporal; upgrade path declarado (vector-clock → CRDT-lite sobre UpdateOperations). Pre-mortem 2 lo pre-autoriza como mínimo viable.
- **Pendientes (downhill):** 6 steps (S0-S5 + cierre S6); S0 ✅ al crear este archivo.
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `57`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `merge_record` (nuevo) ← 0 callers productivos (superficie aditiva; consumidor futuro = transporte sync/import, FIND); `Embedded` ← constructores en `builder.rs` (3 sitios ganan `merge_lock`); re-exports ← `src/sdk/mod.rs` + `src/lib.rs` (aditivo). |
| Callees | `resolve_merge`/`content_bytes` (nuevos, `src/sdk/merge.rs`) ← `merge_record`; `resolve_existing_for_write` + `put_record_exact` (existentes, reuso — firma intacta); `parking_lot::Mutex` (ya presente); `postcard` (ya presente); `now_ms`/`validate_*`/`memory_node_id` (existentes). Cero deps nuevas, cero unsafe. |
| Implicaciones | **API pública aditiva (core SDK):** `MergeOutcome` (enum `#[non_exhaustive]`), `MergeResult` (struct), `Embedded::merge_record()`, re-export en `sdk`/crate root. **Write path intacto:** `put`/`put_batch`/`put_record_exact`/import/WAL/serialización on-disk SIN cambios (contrato: "wal-shipping/write path actuales intactos"). **Sin migración, sin cambios de wire, sin unsafe, sin tocar WAL.** Nuevo lock privado `merge_lock` (patrón `supersede_lock`, REVIEW-13) → Regla 8: auditoría de concurrencia mínima declarada (§Fases). |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `3c4a8935`, tree limpio salvo `opencode.jsonc` ajeno).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/sdk/api/memory.rs` (:260-271 `check_read_only`; :429-525 `resolve_existing_for_write{,_locked}`; :595-665 `put_one`; :750-830 `put`/`put_batch`; :1250-1341 `put_record_exact`; :1343-1420 `supersede` + `supersede_lock` REVIEW-13).
  - `src/sdk/builder.rs` (:20-40 struct `Embedded` + `supersede_lock`/`purge_lock`; :51-58, :109-122, :155-160 los 3 constructores).
  - `src/sdk/serialization/mod.rs` (:70 `now_ms`; :108-169 `validate_namespace`/`validate_key`/`validate_metadata`; :320-509 `record_from_node` + RESERVED_FIELDS; :511-560 `memory_record_to_node_owned`; :77 `memory_node_id`).
  - `src/sdk/types/record.rs` (:176-222 `MemoryInput`; :250-361 `MemoryRecord` + Default).
  - `src/sdk/mod.rs` (exports; `types` es pub(crate) → los tipos nuevos requieren re-export explícito) · `src/lib.rs` (:188-208 patrón de re-export crate-root).
  - `src/wal.rs` (:398-482 `append`/`batch_append` — 1-writer encadenado; SIN cambios) · `src/wal_shipping.rs` (shipping unidireccional — SIN cambios; frontera declarada).
  - `vanta-memory/src/core/profile/profile_sync.rs` (:1-90 — sync de persona por scope, no réplicas; SIN cambios).
  - Tests: `tests/memory_api.rs` (:1-70 patrón `Embedded::open` + `MemoryInput`), `tests/memory_export_import.rs`, `tests/property_durability.rs` (referencia del plan), `src/sdk/api.rs:1546` (test de import existente).
  - Docs: `docs/api/MEMORY_INTERCHANGE_FORMAT.md` (:45-92 — LWW-on-import sin merge declarado), `docs/api/TS_SDK.md:195`, `docs/dev/wasm/CRASH_MODEL.md:40`, `docs/dev/architecture/adr/` (ADR-0054 = formato/status reciente; README generado por `scripts/docs/gen-index.mjs`; `docs/dev/_templates/adr.md`).
  - Reglas: `.opencode/rules/api-contract.md` (R-6 non_exhaustive, R-8 lógica en core), `.opencode/rules/durability.md` (scope WAL — solo lectura, sin cambios).
- **Referencias hacia dentro (imports):** `merge.rs` → `crate::sdk::types::MemoryRecord` + `postcard`; `memory.rs` → `crate::sdk::merge::{resolve_merge, MergeDecision, MergeResult, MergeOutcome}`; `builder.rs` → `parking_lot::Mutex` (ya importado); `lib.rs`/`sdk/mod.rs` → re-export de los 2 tipos.
- **Referencias entrantes (verificadas en DISCOVERY):** `put_record_exact` = 7 callers (version_history, impl_export, search/tests, api.rs) — **firma intacta** (solo la invoco); `resolve_existing_for_write` = 2 callers existentes + 1 nuevo; `Embedded` = constructores de builder (los 3 + `from_engine`); ningún símbolo existente cambia de firma ni semántica (`rg "merge_record|MergeOutcome" src/` = 0 hits — no existe).
- **Archivos a crear/tocar (este run):** `src/sdk/merge.rs` (nuevo), `src/sdk/merge_tests.rs` (nuevo, sibling), `src/sdk/mod.rs`, `src/sdk/builder.rs`, `src/sdk/api/memory.rs`, `src/lib.rs`, `tests/memory_multi_writer.rs` (nuevo), `docs/dev/architecture/adr/ADR-0055-*.md` (nuevo), `docs/dev/architecture/adr/README.md` (regenerado por gen-index), `docs/api/EMBEDDED_SDK.md`, `docs/api/MEMORY_INTERCHANGE_FORMAT.md`, `docs/dev/Backlog.md` (fila FIND), este task file.
- **Veredicto impacto:** **MEDIO (superficie pública aditiva en core SDK + lock nuevo)** — cero cambios de comportamiento en paths existentes (merge es opt-in, método nuevo); cero cambios de serialización/WAL; el lock nuevo sigue el patrón REVIEW-13 ya auditado (`supersede_lock`). **Gate D evaluado (DISCOVERY): NO disparado** — el contrato F0 (Task 57, L1646) manda literalmente "una estrategia ... implementada y declarada (CRDT / vector-clock / LWW explícito — elección documentada con ADR) para el escenario fijado en DISCOVERY"; el pre-mortem (L1647-2) pre-autoriza "mínimo viable = LWW explícito + detección de conflicto"; el símbolo público nuevo es el vehículo que el propio contrato exige. Precedente: MEMG-09/MEMG-10 ("Gate D pre-respondido por el plan F0").

## Contrato

> Verbatim del plan (L1646-1648 — ley):

"una estrategia de resolución de conflictos multi-escritor implementada y declarada (CRDT / vector-clock / LWW explícito — elección documentada con ADR) para el escenario fijado en DISCOVERY (¿multi-device vía event-log/shipping? ¿multi-proceso?) + test de escrituras paralelas concurrentes sin pérdida silenciosa (resultado determinista: merge o winner declarado, nunca "el que escribe gana") + semántica documentada (qué gana, por qué y límites) + `wal-shipping`/write path actuales intactos (suite + failpoints verdes)."

- **Pre-mortem (plan L1647):** (1) escenario abierto → ADR de escenario en DISCOVERY, un solo escenario v1 → **S1/ADR-0055**; (2) CRDT completo excede appetite → mínimo viable LWW + detección, upgrade declarado → **S2/S3**; (3) tocar write path/WAL degrada durabilidad → cambios aditivos, chaos/failpoints verdes → **S5**; (4) solape WSM-15 → frontera: WSM-15 = exclusión (OPFS lock), MEMG-05 = resolución/merge → **ADR §Context**.
- **Stop (plan L1648):** 2-3sem sin contrato → estrategia declarada + implementación mínima en el escenario único fijado + FIND del resto (réplicas/CRDT).
- **Corte declarado de este run (stop package):** ADR-0055 (escenario + estrategia LWW + límites + upgrade) + `MergeOutcome`/`MergeResult`/`Embedded::merge_record` con política pura testeable + tests de concurrencia (paralelo + permutaciones) deterministas + suite/fmt/clippy scoped + failpoints verificados + FIND del transporte event-log/bindings/CRDT restante. NO entran: transporte de sync, bindings (Python/WASM/MCP/HTTP), CRDT/vector-clock completos, cambios de WAL/write path.

## Spec (SDD — decisiones por evidencia)

> **Gate spec-first:** `## Spec` completa (decisiones con alternativas + resolución por evidencia) — feature-add con lógica nueva.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Escenario v1 (uno solo) | A) **Multi-escritor entre escritores que convergen vía ruta de merge/import (device/agente federado)** — el escenario que HOY pierde datos silenciosamente: import = LWW por llegada sin merge ni detección (`MEMORY_INTERCHANGE_FORMAT.md:64`; `put_record_exact` sobrescribe) (pro: es la posición "memoria federada local-first" strategy:412; ya existen registros con timestamps de escritor en el import) / B) multi-proceso embebido (contra: `_lock_file` excluye el 2º proceso por diseño — `storage/engine/mod.rs:375`) / C) multi-tab OPFS (contra: frontera pre-mortem 4 — WSM-15 = exclusión; `CRASH_MODEL.md:40`) / D) shipping 1→réplica (contra: unidireccional, no hay conflicto que resolver — `wal_shipping.rs`) | ✅ **A** — decidido-por-evidencia: strategy:412 + MEMORY_INTERCHANGE_FORMAT.md:60-66 + pre-mortem 1/4 |
| 2 | Estrategia | A) **LWW explícito: orden total determinista `(updated_at_ms, contenido canónico)` + detección de conflicto en empate temporal** (pro: pre-mortem 2 pre-autoriza el mínimo; implementable/testeable en appetite; convergente sin metadatos de reloj por réplica) / B) vector-clock (contra: identidad de réplica + sesiones + GC de relojes = el appetite entero solo para el reloj) / C) CRDT completo (contra: excede 2-3sem — pre-mortem 2) | ✅ **A** — decidido-por-evidencia: pre-mortem 2 (L1647) + strategy:412 ("CRDT-lite ... merge determinista" como destino, LWW como piso) |
| 3 | Unidad de conflicto | A) **Registro completo `(namespace, key)`** — "contenido" = payload + metadata; libro mayor (version/created_at/timestamps) fuera del fingerprint (pro: unidad existente del store; import ya deduplica por (ns,key); simple) / B) por campo (UpdateOperations field-level) (contra: es el upgrade CRDT-lite declarado — no cabe) | ✅ **A** — decidido-por-evidencia: `MEMORY_INTERCHANGE_FORMAT.md:62` + scope |
| 4 | Ruta de aplicación | A) **Método nuevo opt-in `Embedded::merge_record()`** (pro: "write path actuales intactos" literal en contrato L1646; default seguro Rule 4 — `put`/import conservan semántica; testeable aislado) / B) modificar `put_record_exact`/import (contra: cambia semántica documentada del import + rompe tests existentes) | ✅ **A** — decidido-por-evidencia: contrato L1646 + `import_records_counts_invalid_raw_records_as_errors` (api.rs:1546) |
| 5 | Desempate en empate temporal (escrituras concurrentes en el mismo ms) | A) **Orden lexicográfico del contenido canónico (postcard de payload+metadata)** (pro: determinista y convergente sin campos nuevos; arbitrio declarado y honesto) / B) writer_id persistido (contra: campo nuevo en el wire + plomería de identidad en cada superficie — v2/FIND) / C) rechazar el empate (contra: deja el store sin decisión → no converge entre réplicas) | ✅ **A** — decidido-por-evidencia: "resultado determinista" del contrato + sin campos de wire (corte) |
| 6 | Atomicidad decidir+escribir | A) **`merge_lock` dedicado (patrón `supersede_lock`, REVIEW-13) cubriendo resolve→decide→insert** (pro: el `insert_lock` del engine solo serializa el insert individual, no el read-modify-write — comentario `builder.rs:22-27` + `memory.rs:1362-1366`; sin lock, dos merges leen el mismo `existing` y el perdedor puede persistir → NO determinista) / B) sin lock (contra: determinismo roto — hallazgo central del contrato) | ✅ **A** — decidido-por-evidencia: REVIEW-13 (precedente idéntico, lock equivalente) + Regla 8 |
| 7 | Superficie expuesta v1 | A) **Core SDK `Embedded::merge_record` + tipos en `vantadb::{sdk,}` (re-export)** (pro: R-8 "la lógica vive en el core"; el consumidor natural (sync/import) es core) / B) + MCP/HTTP/Python/WASM ahora (contra: sin consumidor declarado; appetite; el wire de bindings no viaja merge aún) | ✅ **A** — decidido-por-evidencia: api-contract R-8 + stop L1648 ("FIND del resto") |
| 8 | Documentación | A) **ADR-0055 (escenario+estrategia+límites+upgrade) + método en `docs/api/EMBEDDED_SDK.md` + nota cruzada en `MEMORY_INTERCHANGE_FORMAT.md` + FIND del transporte/bindings/CRDT** (pro: contrato exige ADR + "semántica documentada"; Regla 3: `pub fn` → docs/api en el mismo PR) / B) solo rustdoc (contra: contrato L1646 pide semántica documentada con ADR) | ✅ **A** — decidido-por-evidencia: contrato L1646 + Regla 3 + adr-gate CI |

## Invariantes de dominio (handoff — MUST)

1. **Write path existente intacto:** `put`/`put_batch`/`put_record_exact`/import/WAL/framing v3/serialización on-disk NO cambian de comportamiento ni de formato (contrato L1646). `merge_record` es aditivo y opt-in.
2. **Determinismo multi-escritor:** para un mismo conjunto de escrituras `{(updated_at_ms, contenido)}`, el registro final es idéntico (payload/metadata/`updated_at_ms`) para CUALQUIER orden de llegada/interleaving — nunca "el que escribe (llega) último gana".
3. **Nunca pérdida silenciosa:** todo `merge_record` devuelve un `MergeResult` no-silencioso (`Inserted|Updated|StaleRejected|AlreadyCurrent` + `conflict`); el perdedor de un empate temporal queda detectado (`conflict=true`), no descartado sin señal.
4. **Fronteras declaradas:** WSM-15 = exclusión (OPFS); MEMG-05 = resolución/merge. Multi-proceso = exclusión por lock (fuera). wal-shipping = replicación unidireccional (fuera). put-concurrente-vs-merge en la misma clave = fuera de cobertura v1 (una ruta por clave).
5. **Detección honesta:** conflicto = empate temporal + contenido distinto (única señal detectable sin vector-clock); actualizaciones con timestamps distintos = LWW declarado (no conflicto). Límite: skew de relojes de pared puede invertir orden causal real → upgrade vector-clock (ADR).
6. **Sin unsafe, sin deps nuevas, sin cambios de wire/on-disk, sin migración.** `unwrap`/`expect` prohibidos en producción (tests: `#![allow]` con justificación).
7. **No tocar WIP ajeno:** MEMG-04 (multi-tenant — otra área) releído fresco; paths disjuntos; pathspec SIEMPRE en el commit; `opencode.jsonc`/master plan/`docs/pipeline-state.json` PROHIBIDOS.
8. **Determinismo en tests:** mismos timestamps fijos, sin env mutation, sin sleeps como sincronización; `TempDir` para disco, `:memory:` para concurrencia.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR:** ≤0 — aditivo (2 tipos + 1 método + 1 lock privado + re-exports). Sin unsafe, sin deps, sin cambios de wire.
**Pago:** (1) cierra el riesgo de pérdida silenciosa del escenario federación (hoy `put_record_exact`/import sobrescribe por llegada — `MEMORY_INTERCHANGE_FORMAT.md:64`); (2) dota de política declarada + detección a un área con `CRDT`/`vector_clock`/`LWW` = 0 hits en código (verificación del plan).
**`ponytail:` notes:** (a) el desempate por bytes de contenido es un arbitrio declarado (no semántico) — el upgrade vector-clock lo reemplaza; (b) `merge_record` es de un solo registro (batch/merge por lotes = si un transporte lo pide); (c) el conflicto NO se materializa aparte (sin tabla de conflictos) — la señal es el `MergeResult`; persistir conflictos = roadmap.
**`NOTICED BUT NOT TOUCHING`:** `put_record_exact` valida `node_id` del caller (import lo exige); `merge_record` lo recomputa (identidad derivada de (ns,key)) — inconsistencia menor declarada, no se toca el path de import. Roles/tablas ajenas de MEMG-04 no se rozan.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: (a) ADR-0055 con escenario único + estrategia declarada + límites + upgrade path; (b) `MergeOutcome`/`MergeResult`/`Embedded::merge_record` implementados con política pura (`resolve_merge`) + unit tests (orden, empate, igualdad, stale); (c) test de concurrencia real (threads paralelos, mismo key) → resultado determinista + sin pérdida silenciosa (todo outcome no-silencioso; perdedor de empate → `conflict=true`); (d) test de permutaciones (mismo conjunto de escrituras en órdenes distintos → registro final idéntico); (e) semántica documentada (ADR + `docs/api/EMBEDDED_SDK.md` + nota en `MEMORY_INTERCHANGE_FORMAT.md`); (f) FIND del resto (transporte/bindings/CRDT) en Backlog; (g) write path/WAL intactos: suites scoped verdes + failpoints; (h) fmt/clippy scoped; (i) review P2-01 por agente distinto |
| **Commit** | Commit atómico conventional `feat(sync): MEMG-05 — ...` (DoD plan L1660) + pathspec solo de archivos propios + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | changelog (minor) — `feat:` → release-plz bump minor |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius hecho en DISCOVERY) · `codebase-memory-mcp_*` (architecture/coverage)
- `cargo nextest run -p vantadb --test memory_multi_writer` (foco) · `-p vantadb` (suite) · `cargo nextest run -p vantadb --features failpoints --test chaos_integrity` (si corre en este entorno; si no, failpoints scoped documentado)
- `campaign_verify_cmd` (fmt + clippy + nextest scoped/full + deny) · `pwsh dev-tools/ocr-review.ps1 -Format json` (cierre)
- Gates docs: `node scripts/docs/check-links.mjs && check-docs.mjs && gen-index.mjs --write` (ADR index) + `scripts/validate-docs-coverage.ps1`

**Skills cargadas (SDP v3, `campaign_discover_skills_v2` phase=BUILD):** `campaign-executor` · `progreso` (base auto-MCP) · `documentation-and-adrs` (pin policy) · `api-and-interface-design` (pin policy) · `deprecation-and-migration` (pin policy — evaluada: sin deprecaciones en este run) · `incremental-implementation` · `test-driven-development` · `rust-write-tests` · `documentation-skill` (docs del run). `doubt-driven-development` → cubierto por review P2-01 adversarial (`vanta-review`); `systematic-debugging` → disponible si RED/GREEN falla.

## Fases explícitas — SECURITY | PERFORMANCE | CONCURRENCIA (P2-07)

- [x] **SECURITY** — no dispara trust boundary nuevo: `merge_record` valida namespace/key/metadata con los validadores existentes; sin input de red/FFI nuevo; audit solo `namespace`/`key` (nunca payload). Sin deps nuevas.
- [x] **PERFORMANCE** — no dispara Regla 9: ruta de sync/import (no hot path); `content_bytes` (postcard) solo cuando hay `existing`; sin claims de performance (Regla 11).
- [x] **CONCURRENCIA (Regla 8)** — `merge_lock` nuevo con patrón idéntico a `supersede_lock` (REVIEW-13); orden de locks: `merge_lock` → (`purge_lock.read` interno de `put_record_exact`); el guard de `resolve_existing_for_write` se suelta ANTES de `put_record_exact` (temporario de statement) para no anidar reads del purge lock; evidencia: 2 tests paralelos con threads reales (distinct-ts y equal-ts, órdenes rotados por ronda) + chaos failpoints verde.

## Steps

### Step 0 — DISCOVERY + task file + coordinación

- **Archivos:** este task file + lectura de base (memory.rs/builder.rs/serialization/types/wal/wal_shipping/profile_sync/MEMORY_INTERCHANGE_FORMAT/CRASH_MODEL/ADR-0054) + MEMG-04 releído fresco + SDP v3.
- **Acción:** ✅ task file completo (Impacto Regla 0 + Spec + corte declarado) + escenario/estrategia fijados + coordinación MEMG-04 (paths disjuntos; sin commits ni worktree en vuelo al momento del check).
- **Verify:** ✅ task file existe; `## Spec` completa (tabla de 8 decisiones); Gate D evaluado (pre-respondido por plan F0).
- **Evidencia:** ✅ DISCOVERY arriba; `rg "merge_record|MergeOutcome" src/` = 0 hits; `git log --all --grep=MEMG-04` = vacío; HEAD `3c4a8935`.

### Step 1 — RED (TDD): tests del contrato (unit + integración)

- **Archivos:** `src/sdk/merge_tests.rs` (nuevo) + `tests/memory_multi_writer.rs` (nuevo) + `src/sdk/merge.rs` stub mínimo (solo tipos para compilar el RED) — o declaración en `mod.rs`.
- **Acción:** tests ANTES de implementar: unit (`resolve_merge`: incoming más nuevo gana; más viejo pierde; empate+distinto → desempate determinista por contenido (y la MISMA decisión con lados invertidos); empate+igual → `Unchanged`; sin existing → `Store`). Integración: (a) merge sin existente → `Inserted`; (b) secuencia A(t=100), B(t=200) → final B; (c) permutación invertida sobre store fresca → final B (mismo registro); (d) empate temporal con contenidos distintos → ganador por desempate + `conflict=true` + el perdedor `StaleRejected` con `conflict=true`; (e) contenido idéntico → `AlreadyCurrent` sin escritura; (f) **paralelo**: N threads merge sobre la misma clave → mismos asserts de (c) + exactamente un ganador final; (g) get() final = ganador declarado.
- **Verify:** RED confirmado — `cargo nextest run -p vantadb --test memory_multi_writer` falla por API ausente (razón correcta, E0599/E0432), no por test mal escrito.
- **Evidencia:** ✅ RED: integración `E0599: no method named 'merge_record' found for struct 'Embedded'` ×11 (12 errores, todos de la API ausente) · unit `cargo nextest run -p vantadb --lib -E 'test(resolve_merge) or test(content_bytes)'` → 9 FAIL con `panicked at src\sdk\merge.rs:73/86: not implemented: MEMG-05 S2 (RED scaffold)` (scaffold de contrato: tipos + firmas con `unimplemented!`, eliminado en S2).

### Step 2 — GREEN policy: `src/sdk/merge.rs` (política pura) + unit tests

- **Archivos:** `src/sdk/merge.rs` (nuevo), `src/sdk/merge_tests.rs`, `src/sdk/mod.rs` (mod + re-export), `src/lib.rs` (re-export crate-root).
- **Acción:** implementación mínima: `MergeOutcome` (`#[non_exhaustive]` R-6), `MergeResult`, `resolve_merge` + `content_bytes` (postcard de payload+metadata), rustdoc con semántica. Sin dependencias nuevas.
- **Verify:** `cargo nextest run -p vantadb -E 'test(merge)'` (unit) ✅.
- **Evidencia:** ✅ GREEN unit **16/16** (9 merge + filtro colateral; luego superseded por suite lib completa **2338/2338**). Impl: `content_bytes` (`merge.rs:71-76`, postcard de `(payload, metadata)`), `resolve_merge` (`merge.rs:83-115`, LWW total order + conflicto en empate + tie-break lexicográfico de bytes). `unimplemented!` eliminado.

### Step 3 — GREEN store: `Embedded::merge_record` + `merge_lock`

- **Archivos:** `src/sdk/builder.rs` (campo + 3 constructores), `src/sdk/api/memory.rs` (método).
- **Acción:** `merge_record` = check_read_only + validadores + `merge_lock.lock()` + `resolve_existing_for_write` (dropping guard) + `resolve_merge` + `put_record_exact` (reuso) con bookkeeping local declarado (created_at first-seen, version bump); audit `merge_record`; outcome no-silencioso. Node_id recomputado (identidad derivada).
- **Verify:** `cargo nextest run -p vantadb --test memory_multi_writer` ✅ (todos los tests de S1) + `cargo nextest run -p vantadb -E 'test(memory) or test(merge)'` sin regresión.
- **Evidencia:** ✅ GREEN integración **9/9** (incluye 2 tests paralelos con threads reales: distinct-ts converge a `writer-7`, equal-ts converge con `conflicts == N-1`; permutaciones secuenciales y cross-path `put`) · doctest `merge_record` **1/1** · `cargo clippy -p vantadb --lib -- -D warnings` **exit 0** · fmt de mis archivos exit 0. Impl: `merge_record`/`merge_record_inner` (`memory.rs:1343-1500`), `merge_lock` en `builder.rs` (campo + 3 constructores), re-exports en `sdk/mod.rs` + `lib.rs`.

### Step 4 — Docs: ADR-0055 + api docs + nota cruzada + FIND

- **Archivos:** `docs/dev/architecture/adr/ADR-0055-multi-writer-merge-lww.md` (nuevo), `docs/dev/architecture/adr/README.md` (gen-index), `docs/api/EMBEDDED_SDK.md`, `docs/api/MEMORY_INTERCHANGE_FORMAT.md`, `docs/dev/Backlog.md` (fila FIND-XXX), este task file (sync).
- **Acción:** ADR (escenario único + estrategia + límites + upgrade + frontera WSM-15/shipping); doc del método con semántica (qué gana, por qué, límites); nota cruzada en el doc de interchange (el merge declarado como ruta multi-writer); FIND del resto.
- **Verify:** gates docs (`check-links`/`check-docs`/`gen-index --write` + `validate-docs-coverage`).
- **Evidencia:** ✅ ADR-0055 (`docs/dev/architecture/adr/ADR-0055-multi-writer-merge-lww.md`) · `EMBEDDED_SDK.md` (fila + §Multi-writer merge + audit ops) · `MEMORY_INTERCHANGE_FORMAT.md` (bullet multi-writer) · `Backlog.md` FIND-303 · gen-index `--write` (adr/README.md + docs/index.md + llms.txt) · `check-links` **exit 0** (0 broken) · `check-docs`: mi description corregida (406→<400); queda SOLO la violación ajena de `MEMG-04.md` (445 chars — WIP ajeno, no se toca).

### Step 5 — Verify full + failpoints + OCR

- **Archivos:** n/a (verificación).
- **Acción:** `campaign_verify_cmd` (fmt + clippy + nextest scoped + deny); failpoints (chaos_integrity o scoped equivalente); stress del test de concurrencia (`--stress-count`); OCR delegation (`dev-tools/ocr-review.ps1 -Format json`).
- **Verify:** ✅ suite lib **2338/2338** · integración **9/9** · doctest **1/1** · memory_api+memory_export_import (`--profile audit --ignore-default-filter`) **19/19** · chaos failpoints (`--profile chaos --features failpoints --test chaos_integrity`) **1 passed** · clippy lib `-D warnings` **exit 0** · fmt mis archivos **exit 0**.
- **Evidencia:** ✅ logs en `target/tmp/mw_*.log`; OCR spec `target/tmp/ocr-memg05.json` — Rule Groups aplicados a mis archivos (src/sdk/merge.rs, memory.rs, builder.rs, mod.rs, lib.rs, tests): **0 Critical / 0 High** (sin unwrap/expect/panic/todo/unsafe en producción; `expect` solo en tests con allow header; locks documentados; clones mínimos justificados). Suites corridas bajo `dev-tools/heavy-test-lock.ps1` (regla owner 2026-10-05).

### Step 6 — Review P2-01 + commit + campaign completed + progreso

- **Archivos:** los del run (pathspec).
- **Acción:** review P2-01 por agente distinto (`vanta-review`, sesión `ses_ef187540effeyMqJDSOenbJFFw`) → verdict inicial **CHANGES REQUIRED** (R1 Critical + R2 Required + R3 proceso + R4-R6) → **fixes aplicados** → delta re-review **APPROVE** (nits N1-N4 corregidos) → commit local `feat(sync): MEMG-05 — ...` → campaign `completed` (taskId 57, payload review) → `skill progreso` (delegado al lead, patrón MEMG-10).
- **Verify:** ✅ review verdict registrado (approve); commit local pendiente de hash (se sincroniza en el commit `docs(task)` de cierre).
- **Evidencia:** ✅ re-review APPROVE (`ses_ef187540effeyMqJDSOenbJFFw`); fixes R1-R6 verificados con suites re-ejecutadas post-fix (lib **2339/2339**, integración **11/11**, doctest 1/1, memory regression **19/19**, clippy/fmt exit 0).

## RESULTADO (sección 7 — contrato de retorno)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 7/7 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: <pendiente — se sincroniza en el commit docs(task) de cierre>
ARCHIVOS: src/sdk/merge.rs (nuevo) · src/sdk/merge_tests.rs (nuevo) · src/sdk/api/memory.rs (hunks propios) · src/sdk/builder.rs · src/sdk/mod.rs · src/lib.rs · tests/memory_multi_writer.rs (nuevo) · docs/dev/architecture/adr/ADR-0055-multi-writer-merge-lww.md (nuevo) · docs/dev/architecture/adr/README.md · docs/api/EMBEDDED_SDK.md · docs/api/MEMORY_INTERCHANGE_FORMAT.md · docs/dev/Backlog.md (FIND-303) · docs/dev/tasks/MEMG-05.md (nuevo)
VERIFY_CONTRATO: pasa (RED→GREEN + ADR + tests concurrencia deterministas + suite/failpoints + docs gates scoped)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(pre-respondido por plan F0 — contrato L1646 delega la estrategia con ADR) D:no(pre-respondido por plan F0) V:no C:no | review P2-01 CHANGES REQUIRED→APPROVE (R1 fix convergencia)
SKILLS_CARGADAS: documentation-and-adrs · api-and-interface-design · deprecation-and-migration (pins SDP) · incremental-implementation · test-driven-development · rust-write-tests · documentation-skill + base campaign-executor · progreso · ponytail
```

**Review P2-01:** APPROVE (fresh, reviewer `vanta-review`, contexto `ses_ef187540effeyMqJDSOenbJFFw` ≠ autor `ses_ef1cddadbffez1wrhUSMdikx9B`; 2 rondas: CHANGES REQUIRED → delta APPROVE).

## Iteración post-review P2-01 (adversarial, reviewer distinto)

> Review inicial: **CHANGES REQUIRED** — R1 Critical real (convergencia rota con contenido idéntico + timestamps distintos: el reloj almacenado quedaba en el primer arribo y arrastraba ganadores posteriores). Findings resueltos antes del commit:

| Finding | Severidad | Resolución |
|---------|-----------|------------|
| R1 — contenido idéntico + ts distinto rompía la convergencia declarada (orden total no monótono) | 🔴 Critical | `resolve_merge`: contenido igual + reloj mayor → `Store { conflict: false }` (avanza la clave de orden = max del write set); menor/igual → `Unchanged`. Tests: unit dividido (newer→Store / older-equal→Unchanged) + 2 integración nuevos (reloj converge a max; cascada `{(c,100),(c,500),(c2,300)}` → `(c,500)` en 3 órdenes). ADR §Decisión 1 + límite 3 actualizados |
| R2 — alcance de campos no declarado (vector/validez/confidence fuera del fingerprint; entrante se escribe entero) | 🟠 Required | Declarado en ADR (§Decisión 2 + límite 7), module docs de `merge.rs`, `content_bytes`, y `EMBEDDED_SDK.md` (bullet Field scope; "byte-identical" → "identical under the fingerprint") |
| R3 — `memory.rs`/`Backlog.md` mezclan hunks de MEMG-04 (WIP ajeno en el mismo worktree) | 🟠 Medium (proceso) | **Staging quirúrgico por hunks**: blob staged = HEAD + solo mis cambios (`git hash-object` + `update-index --cacheinfo`), verificado sin marcadores ajenos (`check_namespace_quota`/`quota` = 0 en el diff staged) |
| R4 — `merge_lock` por handle, no por engine | 🟡 Low | Declarado como límite 8 del ADR (mismo alcance conocido de `supersede_lock` REVIEW-13) |
| R5 — `MergeResult` sin `#[non_exhaustive]` | ⚪ Nit | `#[non_exhaustive]` agregado |
| R6 — validación asimétrica del entrante inválido que pierde | ⚪ Nit | `validate_confidence_fields` + ventana validez ANTES de decidir (error sin importar outcome) |
| N1-N4 — nits documentales post-fix (numeración ADR, docs stale de "no-op") | ⚪ Nit | Corregidos en el mismo commit |

**Re-verify post-fixes:** lib **2339/2339** · integración **11/11** · doctest **1/1** · memory regression **19/19** · clippy `-D warnings` exit 0 · fmt exit 0. **Delta re-review: APPROVE (misma sesión).**

## Notas (coordinación + shared files)

- **MEMG-04** (misma fase F4, otra área): releído fresco en DISCOVERY (HEAD `3c4a8935`): sin task file ni commits. **Durante el run apareció su WIP en el MISMO worktree** (`src/config.rs`, `src/server/{middleware,router,state}.rs`, `src/cli_server_auth_tests.rs`, `src/server/cli_server_auth_tests.rs`, `tests/rbac_namespace.rs`, `vantadb-server/tests/server.rs`, `tests/quota_records.rs`, `docs/dev/tasks/MEMG-04.md`) y **también hunks dentro de archivos compartidos** (`src/sdk/api/memory.rs` → `check_namespace_quota` + call sites; `docs/dev/Backlog.md` → edición de FIND-301 + FIND-304). **Staging quirúrgico ejecutado**: blob staged = HEAD + solo mis hunks (`git hash-object` + `update-index --cacheinfo`, verificado sin marcadores ajenos) — nada suyo se tocó ni se stageó (review P2-01 R3). Nota: `docs/index.md`/`llms.txt` regenerados por `gen-index --write` incluyen entradas ajenas → **NO stageados** (consolidación del lead, patrón MEMG-10).
- **Heavy-test-lock (regla owner 2026-10-05):** todas las suites pesadas de este run (audit/chaos/regresión) corrieron con `dev-tools/heavy-test-lock.ps1 acquire/release`.
- **Incidente de entorno:** rustc `STATUS_STACK_BUFFER_OVERRUN` intermitente por disco casi lleno (`target/debug/incremental` = ~42 GB, C: 6.5 GB libres) → `dev-tools/target-cleanup.ps1 -Clean -Yes` (C: → 48 GB) + `CARGO_INCREMENTAL=0` para el resto del run. Sin impacto en el código.
- **PROHIBIDO tocar:** `opencode.jsonc` (dirty ajeno), master plan (`docs/dev/plans/2026-10-04-master-plan-0.9.0.md`), `docs/pipeline-state.json`.
- **Campaign server:** taskId `57` — cierre `completed` con payload review HARD-07.

**Context Save Point (si el run se interrumpe):** estado en §Steps + recitation; trabajo parcial = git diff del worktree; NO re-hacer steps ✅; slice aditivo y reanudable (S1 RED → S2 policy → S3 store → S4 docs → S5 verify → S6 cierre).
