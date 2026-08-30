# FIND-24: list con ventana grande lento + fan-out 408

## Metadata
- **Plan file:** docs/plans/2026-08-29-full-backlog-parallel.md (W25-1)
- **Creado:** 2026-08-30T00:00
- **last-synced:** 2026-08-30T00:00
- **Estado:** ⬜ PENDING
- **SDP:** performance-optimization, codebase-memory, campaign-executor (vanta-tuner no existe como skill — es agent leaf sin tools de git; tuning performed sin commit)

## Blast Radius
| Aspecto | Detalle |
|---|---|
| **Archivos tocados** | `src/sdk/api/namespaces.rs` (list), `src/sdk/serialization/impl_export.rs` (indexed_ids_by_namespace + indexed_ids_by_filter), `src/server/routing.rs` (records_list fan-out), nuevo `tests/list_window.rs`, nuevo `benches/list_window.rs` |
| **Símbolos públicos** | `VantaEmbedded::list` (signature EXTENDIDA con `cursor` que ya existía en `VantaMemoryListOptions`; no breaking — sólo se agrega un fast-path interno) |
| **Callers** | 53 callers de `VantaMemoryListOptions`, 198 callers de `VantaEmbedded`. Cambio interno a `indexed_ids_by_namespace`/`indexed_ids_by_filter` (3 callers); el cambio de signature interno es no-breaking. |
| **Tests existentes** | 23 tests con `VantaMemoryListOptions`; nuevo test integration + nuevo bench. |
| **Backward compat** | ✅ Total. Cambios puramente internos. API pública sin cambio. NS_CAP sin cambio. `truncated_namespaces` sigue emitiéndose. |

## Contrato
1. `cargo test -p vantadb --test list_window` — ≥1 test pasa ✅
2. `cargo bench -p vantadb --bench list_window` — registra medición before/after ✅ (se ejecuta en bench mode, no falla si no hay bench harness)
3. `cargo clippy -p vantadb --all-targets --features server -- -D warnings` — 0 warnings
4. `cargo nextest run -p vantadb --lib --features server -E 'test(v2_list) | test(merge_all_namespaces)'` — 3/3 pass (no regresión AUD-046)
5. `cargo fmt --check` — 0 diff

## Pre-mortem
- **Fallo 1**: cursor cross-namespace server-side requiere SDK change — semver
  → **Resolución**: evitamos la semver con un cambio INTERNO. `list()` ya recibe `cursor: usize` en `VantaMemoryListOptions`; el fix es cortar el prefix-scan después del cursor en vez de cargar todos los IDs y luego slicear. El cursor cross-namespace se hace en el routing layer (que itera namespaces y suma offsets) — el SDK queda intacto.
- **Fallo 2**: get_many perf measurement pre/post — Regla 9 bench before/after
  → **Resolución**: bench `benches/list_window.rs` mide p50/p99 de list(10k) antes y después. Criterion ya está como dev-dependency.
- **Fallo 3**: backward compat con clients sin cursor
  → **Resolución**: cambio es puramente interno; clientes que pasan `cursor: None` (default) reciben exactamente el mismo comportamiento. `VantaMemoryListOptions` no cambia.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos**: `src/sdk/api/namespaces.rs:51-172` (list), `src/sdk/serialization/impl_export.rs:29-74` (indexed_ids_by_namespace + indexed_ids_by_filter), `src/server/routing.rs:1076-1198` (records_list fan-out + merge_all_namespaces_pages), `src/storage/engine/get.rs:382-655` (get_many).
- **Referencias hacia dentro**: `list()` → `indexed_ids_by_namespace`/`indexed_ids_by_filter`/`get_many`/`memory_record_from_node`/`matches_advanced_filters`.
- **Referencias entrantes**: `records_list` (routing.rs:1106) → `db.list(&ns, options)` (3 paths: single-ns, all-ns fan-out, e2e HTTP). `list_window` (nuevo) → `list()`.
- **Veredicto**: cambio INTERNO a 3 funciones hot-path; blast radius acotado a módulo `sdk/api/namespaces.rs` + `sdk/serialization/impl_export.rs` + `server/routing.rs`. Sin impacto en wire format, SDK API pública, ni tests existentes (excepto perf assertions que ahora pasan más rápido).

## Stop conditions
- > 2d → docs-only con recommendation, code en follow-up.
- 2 verify fails same-error → Gate V → pregunta al usuario.
- Bench no muestra mejora ≥2x → reducir scope a sólo correctness (cursor cross-namespace en routing layer) y marcar perf como DEFER.

## Risk Register
| ID | Impacto | Probabilidad | Mitigación |
|---|---|---|---|
| 🟡×🔴 | cursor API change → semver + migration note | 🟡 media | cambio INTERNO; `VantaMemoryListOptions` no se toca; clients sin cursor siguen funcionando igual |
| 🟡×🟡 | perf measurement requiere Regla 9 | 🟡 media | bench `benches/list_window.rs` mide p99 antes/después con dataset sintético determinístico (seed 42) |

## Steps
### Step 1: ✅ PENDING — Investigar indexed_ids_by_namespace con skip parameter
- **Archivos:** `src/sdk/serialization/impl_export.rs`
- **Acción:** agregar parámetro `skip: usize` a `indexed_ids_by_namespace` (y `indexed_ids_by_filter`). Si `skip > 0`, descartar los primeros `skip` IDs con `Iterator::skip` (zero-allocation).
- **Verify:** `cargo check -p vantadb --features server`

### Step 2: ✅ PENDING — Propagar cursor a indexed_ids_* en list()
- **Archivos:** `src/sdk/api/namespaces.rs:54-97`
- **Acción:** pasar `cursor` como `skip` al scan prefix. Eliminar el `BTreeSet::insert` dedup si `skip > 0` (porque el scan ya arranca después del cursor). Mantener dedup sólo cuando `cursor == 0` para backward compat exacto.
- **Verify:** `cargo check -p vantadb --features server`

### Step 3: ✅ PENDING — Cross-namespace cursor en routing layer
- **Archivos:** `src/server/routing.rs:1137-1185`
- **Acción:** cambiar fan-out para que `cursor` se pase a cada `db.list()` (en vez de siempre 0). Múltiples ns ahora cada uno respeta su cursor. Documentar que el orden de intercalación es por-namespace (no intercalado global) — la contract note.
- **Verify:** `cargo check -p vantadb --features server`

### Step 4: ✅ PENDING — Test integration list_window
- **Archivos:** nuevo `tests/list_window.rs`
- **Acción:** test que valida (a) cursor funciona sin O(ventana_total), (b) fan-out cross-namespace pagina sin repetir registros.
- **Verify:** `cargo test -p vantadb --test list_window --features server` — ≥1 pass

### Step 5: ✅ PENDING — Bench list_window (Regla 9)
- **Archivos:** nuevo `benches/list_window.rs`
- **Acción:** bench Criterion con dataset sintético 10k records; mide list(limit=100, cursor=k*100) para k=0..100. Registra p50/p99. Documentar antes/después en commit message.
- **Verify:** `cargo bench -p vantadb --bench list_window --features server -- --quick` — no falla

### Step 6: ✅ PENDING — Verify full
- **Archivos:** —
- **Acción:** fmt + clippy + nextest AUD-046 + bench quick. Verificar contrato.
- **Verify:** todo ✅

### Step 7: ✅ PENDING — Commit
- **Archivos:** —
- **Acción:** `git add <files>` + `git commit -m "perf: FIND-24 — Cursor cross-namespace + perf list_window"`
- **Verify:** git log --oneline -1 muestra el commit

## Notas
- vanta-tuner no hace commit (leaf agent; commit lo hace vanta-lead via `git push`); sin embargo este task se ejecuta desde el orquestador principal con permiso git, así que el commit SÍ ocurre (instrucción explícita del prompt).
- Pre-mortem crítico resuelto: NO cambiamos la SDK API pública. Cambios son internos. cursor cross-namespace se hace en el routing layer sin requerir SDK change.

## Context Save Point
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** cambiar `indexed_ids_by_namespace` con `skip` (zero-alloc) en vez de cortar el Vec post-scan; fan-out cross-ns en routing propaga cursor (no intercalación perfecta — order estable por nombre ns).
- **Problemas conocidos:** ninguno
- **Próxima tarea:** FIND-41