---
title: "FIND-190 — put-overwrite de nodo Cold invisible a get/list con payload viejo prefetcheado"
kind: task
---

# FIND-190: put-overwrite de nodo Cold invisible a get/list con payload viejo prefetcheado

## Metadata

- **Fuente:** `docs/dev/Backlog.md:372` (FIND-190, 🟠 Alta) · Origen: review VER-07 (engine-FIND)
- **Carril:** pre-release 0.8.0 · **Decisión owner:** 2026-10-01 (fix en este carril, no diferir)
- **Esfuerzo:** 🟢 (fix de correctitud, ~2 archivos) · **Tipo:** bug-fix
- **Branch:** develop · **Commit:** lo hace el LEAD (sub-agente NO commitea — mandato del carril)
- **Estado:** ✅ COMPLETO (steps 1-3 ✅ + verificación verde) — commit y review P2-01 = LEAD
- **Workaround documentado:** VER-07 — UPDATE = delete+put (deja de ser necesario con el fix)

## Contexto verificado del bug (mecanismo exacto)

Cadena de evidencia verificada contra el código actual:

1. **Tier default Cold** — `UnifiedNode::new` nace `NodeTier::Cold` (`src/node/unified.rs:86`).
2. **Prefetch cachea CUALQUIER tier** — `prefetch_related` (get.rs:435-440) inserta en
   `cache.volatile` con `Entry::Vacant` el nodo materializado, sin mirar el tier.
   Es el ÚNICO path que cachea nodos Cold (los writes solo cachean Hot).
3. **El overwrite solo refresca Hot** — `apply_insert` (insert.rs:669-704) hace
   `guard.insert` solo `if node.tier == Hot`; un overwrite Cold no toca el cache
   → la entry prefetcheada con el payload viejo queda.
4. **Lectura sirve el cache primero** — `get()` → `lookup_volatile` (get.rs:381) y
   `get_many()` → `try_write/read` (get.rs:466-499) devuelven la entry stale.
   El SDK `list` fetchea por `get_many` (`src/sdk/api/namespaces.rs:142`) → afectado.
5. **Precedente de fix ya existente:** el commit de txn invalida SIEMPRE antes de
   aplicar (`txn.rs:339`); delete también (`delete.rs:147`). Los paths non-txn
   (single + batch) son los únicos que no invalidan.

**Hole gemelo (batch):** `cache_batch_hot_nodes` (insert.rs:373-383) solo inserta Hot,
tampoco remueve Cold → `put_batch`/`batch_insert` reproduce el mismo bug.

## Decisión de fix

Vía elegida: **invalidar cache en overwrite Cold** (no gate de tier en `prefetch_related`).

- El gate de tier neutraría el prefetch para nodos Cold (el tier default) — el cache
  es donde el prefetch entrega valor; además el hole persistiría en el edge
  Hot-cacheado → overwrite Cold.
- La invalidación es el patrón ya establecido por txn/delete, cubre ambos write paths
  non-txn y restaura el invariante: **tras un write exitoso de X, el volatile cache
  no puede contener un payload más viejo de X**.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `insert()` single y `batch_insert_with_opts` (engine) · SDK `put`/`put_batch`/`insert_batch` · cualquier path que lea por `get`/`get_many` (search, list) |
| Callees | `cache.volatile` (CacheLayer) · `prefetch_related` (origen de entries Cold) |
| Implicaciones | Sin cambio de API pública ni formato on-disk. Solo coherencia del cache volátil. Costo: un `write().remove()` por overwrite Cold (el path de insert ya toma locks de I/O; despreciable vs WAL+KV+vstore) |

## Impacto mapeado (Regla 0)

> Gate cumplido antes de la primera edición.

- **Archivos leídos (completos/secciones relevantes):** `src/storage/engine/insert.rs`
  (paths insert/apply_insert/cache_batch_hot_nodes/batch) · `src/storage/engine/get.rs`
  (get/get_many/prefetch_related/materialize) · `src/storage/engine/cache.rs` ·
  `src/storage/engine/txn.rs` (commit + apply_insert_with_txn) · `src/storage/engine/delete.rs`
  (invalidación) · `src/storage/engine/tests/{engine,ops,mod}.rs` · `src/cache_warmer.rs` ·
  `src/sdk/api/namespaces.rs` (list → get_many) · `src/node/unified.rs`
- **Referencias hacia dentro:** `apply_insert` ← `insert()` (:216) · `cache_batch_hot_nodes` ←
  `commit_batch_locked` (:595) · `prefetch_related` ← `get()` (:389)
- **Referencias entrantes a los editados:** 87 callers de `insert` (benches/tests/SDK);
  tests existentes de overwrite (`tests/engine.rs:676,716,746`), cache (`tests/ops.rs:443-467,557-572`)
  y prefetch (`tests/engine.rs:788`) — todos deben seguir verdes
- **Veredicto impacto:** **bajo-medio** — 1 archivo de producción (`insert.rs`, 2 puntos de
  edición) + 1 archivo de tests. Path adversarial (src/storage/**) → review de agente
  distinto en el cierre del LEAD (P2-01)

## Contrato

> **Verbatim del encargo (orquestador, 2026-10-01):**

"un `put`-overwrite de un nodo Cold queda invisible a `get`/`list` si el prefetcher
cacheó el payload viejo. Fix: invalidar cache en overwrite Cold o gate de tier en
`prefetch_related`. Test primero (RED→GREEN): put → get para cachear → put-overwrite
Cold → get/list debe devolver el payload NUEVO. Fix mínimo. Verificación con
`CARGO_BUILD_JOBS=2`; test nuevo + `cargo check -p vantadb` + nextest del módulo.
NO COMMITEAR (lo hace el lead)."

## Invariantes de dominio (handoff — MUST)

1. **Coherencia cache volátil:** tras write exitoso de X (single/batch/txn/delete),
   ninguna entry de X en `cache.volatile` puede ser más vieja que el último write.
2. **Hot sigue refrescando:** el comportamiento Hot (`guard.insert` en apply_insert)
   NO cambia — los tests de cache Hot deben seguir verdes.
3. **Prefetch intacto:** `prefetch_related` sigue cacheando co-accesos de cualquier
   tier (el fix es en el write, no en el prefetch) — `test_get_prefetch_does_not_recurse_forever` verde.
4. **Sin cambios de API/on-disk:** 0 cambios en firmas públicas, 0 cambios de formato.
5. **Lock order:** `cache.volatile.write()` ya se toma en el path de insert (Hot) y en
   txn/delete; la invalidación Cold usa el mismo nivel (no introduce orden nuevo).

## Steps (TDD)

### Step 1: RED — tests de reproducción (single + batch) — ✅
- **Archivo:** `src/storage/engine/tests/engine.rs` (+94L; 2 tests junto a `test_get_prefetch_does_not_recurse_forever`)
- **RED (evidencia):** `CARGO_BUILD_JOBS=2 cargo nextest run --profile audit -p vantadb --lib -E 'test(cold_overwrite)'` → **FAIL**:
  `assertion left == right failed: get() must return the NEW payload after a Cold overwrite`
  `left: Some(String("old")) · right: Some(String("new"))`
  (`storage::engine::tests::engine::test_cold_overwrite_invalidates_prefetched_cache_entry`)

### Step 2: GREEN — fix mínimo — ✅
- **Archivo:** `src/storage/engine/insert.rs` (+14L)
- **Edit 1:** `apply_insert` — rama `else` (Cold): `self.cache.volatile.write().remove(&node.id)`
- **Edit 2:** `cache_batch_hot_nodes` — rama `else` (Cold): `guard.remove(&node.id)`
- **GREEN:** `cargo nextest run --profile audit -p vantadb --lib -E 'test(prefetched_cache_entry)'` → **2 passed, 0 failed**

- Step 3: VERIFY — suite + gates — ✅ (ver tabla abajo)

## Deuda técnica (Regla 6)

**Saldo neto: sin deuda nueva** — el workaround VER-07 (delete+put) **deja de ser necesario** para el caso cubierto (overwrite single/batch de Cold); su código en `dream promote` sigue vigente (correcto, solo cuesta un delete extra) — simplificación trackeada en FIND-223 (review P2-01, O2).
`insert_to_cf` (ops.rs:205) queda fuera de scope (API low-level, sin callers de producción
in-tree) — NOTICED BUT NOT TOUCHING (si aplica, FIND aparte).

## Verificación (completada)

| Comando | Resultado |
|---|---|
| RED: `cargo nextest run --profile audit -p vantadb --lib -E 'test(cold_overwrite)'` | ❌ 1 failed — bug reproducido (`old` vs `new`) |
| GREEN: `cargo nextest run --profile audit -p vantadb --lib -E 'test(prefetched_cache_entry)'` | ✅ 2 passed |
| `cargo nextest run --profile audit -p vantadb --lib -E 'test(storage::engine)'` | ✅ 390 passed |
| `cargo nextest run --profile audit -p vantadb --lib` (suite unit completa) | ✅ 2302 passed, 2 skipped, 0 failed (2 SLOW preexistentes) |
| `cargo check -p vantadb` | ✅ exit 0 |
| `cargo clippy -p vantadb --all-targets -- -D warnings` | ✅ exit 0 |
| `cargo fmt --check` | ✅ exit 0 |
| Integration (`--test basic_node --test mutations --test storage --test core_invariants`) | ⚠️ NO CORRIDOS — ver Bloqueos |

Todos con `CARGO_BUILD_JOBS=2`.

## Review (P2-01 — vanta-review, contexto fresco)

**Veredicto: ✅ APPROVE** (`06496a9a`) — sin Critical ni Required. Re-verificó contrato (2/2) + módulo storage::engine (390/390) y auditó cobertura de todos los paths de overwrite (insert/batch/txn/delete/replay/consolidate), orden de locks (sin inversión), eviction y calidad de tests (tier Cold genuino; RED airtight por inspección).

- 🟡 **O1** — Race residual de resurrección en `prefetch_related` (`get.rs:435-441`, pre-existente, multi-tier) → **FIND-222**.
- 🟡 **O2** — Claim "se elimina el workaround" impreciso + comentario stale en `vanta-memory/src/core/dream/mod.rs:903-909` → claim reformulado + **FIND-223**.
- 🟡 **O3** — Cierre documental (Review + Backlog + avance) ✅ este commit.
- ⚪ **O4** — Integration targets pendientes por entorno (`vanta-cli.exe` lockeado por MCP PID 28976) → correr en CI / antes del gate de release 0.8.0.

## Bloqueos de entorno (no de código)

1. **`vanta-cli.exe` bloqueado por el MCP server vivo** (PID 28976 `vanta-cli server --mcp --db C:/Users/Eros/.vantadb` + PID 26976 `vantadb-server --mcp`, desde 12:21). Cargo construye el bin al compilar integration tests (aunque no lo usen) → `failed to remove target/debug/vanta-cli.exe: Acceso denegado (os error 5)`.
   - Verificado que `cargo test --no-run -p vantadb --test basic_node` (sin nextest) falla igual → bloqueo de entorno, no de código.
   - **Acción sugerida (LEAD/CI):** correr los integration targets cuando el MCP server pueda reiniciarse, o en CI. Los unit tests (2302) cubren el código cambiado.
2. **Disco C: al 100%** al iniciar el build (os error 112). Se liberó espacio borrando scratch viejo de pytest (`%LOCALAPPDATA%\Temp\pytest-of-Eros\pytest-5`, `pytest-6` — runs del 29/9, 4.4 GB) → 23 GB libres. `pytest-7` (15 GB, 30/9) se dejó intacto. **Acción de entorno, no del repo.**

## Notas

- **`git status` al cierre:** mis archivos = `src/storage/engine/insert.rs`, `src/storage/engine/tests/engine.rs`, `docs/dev/tasks/FIND-190.md`. Además hay cambios de agentes concurrentes (`docs/index.md`, `llms.txt`, `opencode.jsonc`) — NO son de esta tarea; el LEAD decide si van en el mismo commit.
- **CRLF:** `insert.rs` queda `w/crlf` como otros 55 archivos del checkout (pre-existente; git normaliza a LF por `eol=lf`). Sin acción.
- **`insert_to_cf` (ops.rs:205):** mismo hole teórico (no invalida cache), pero sin callers de producción in-tree (solo tests) — NOTICED BUT NOT TOUCHING; si se expone, FIND aparte.
- **Workaround VER-07 (UPDATE = delete+put):** deja de ser necesario para el caso cubierto (overwrite single/batch de Cold con entry prefetcheada).
