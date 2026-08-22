# Serie AUD-0xx — hardening y auditoría 2026-08-13

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-08-13 — AUD-031: Panic-hardening engine embebido (unwrap/expect alcanzables) ✅

**Fuente:** Backlog `AUD-031` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`)

**Objetivo:** reemplazar unwrap/expect alcanzables desde la API pública del SDK por propagación de error (`Result`/`?`) — un panic en `VantaEmbedded` mata el proceso host (Python/WASM/TS).

**Resuelto por (vanta-worker):**
- **Alcance:** solo código no-test alcanzable por usuario. `src/parser/mod.rs` no-test = 0 unwraps (151 matches todos en `#[cfg(test)]`, módulo ≥ línea 550). `src/storage/engine/ops.rs` no-test = exactamente 5 unwraps, todos `active.iter().next().unwrap()` en los sitios 642/949/1004/1483/1837.
- **Conversión 5/5:** `insert`/`get`/`delete` (funciones `Result`) → `active.iter().next().copied().ok_or_else(|| VantaError::generic_error("active transaction set corrupted: len()==1 but no txn id"))?`; helpers `existing_for_batch`/`existing_for_batch_many` (sin `Result`) → `if let Some(&txn_id)` anidado con comentario de decisión (branch imposible: `parking_lot::Mutex<HashSet<u64>>` poison-free, `len()==1` ⇒ `next()` es `Some`; degradación segura a cache/backend).
- **Decisión de diseño:** los unwraps estaban protegidos por invariante local, pero son alcanzables por API pública y el costo de conversión es trivial → defensa en profundidad (espíritu del finding: panic mata el host). No se tocaron los 1381−5 restantes (tests/benches/paths internos ya hardened — `ops.rs:1761` bounds-guards + SAFETY, INV-024).
- **Verify:** `cargo check -p vantadb` ✅; `cargo nextest run --profile audit -p vantadb --build-jobs 2` → **1885 passed** ✅; `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` ✅; `cargo fmt --check` ✅; `rg "\.unwrap\(\)|\.expect\(" src/storage/engine/ops.rs` → 0 matches.
- **Review P2-01:** dictamen vanta-review — 2 bloqueantes de cierre corregidos (commit + registro REVIEW) y 2 mejoras de documentación aplicadas (`HashSet` no `BTreeSet`; comentarios de defensa en helpers). Approve post-fix.

**Commit:** `c7185d25` — fix: propagate active-txn corruption as error instead of panic (AUD-031)

**Ids:** `AUD-031`

### 2026-08-13 — AUD-023: Validar dims de sparse vector en decode (P2-7) ✅

**Fuente:** Backlog `AUD-023` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`, finding P2-7)

**Objetivo:** `sparse_vector_from_field` hacía `pair[0] as u32` sin validar — NaN/negativo/out-of-range saturaban silencioso a 0/u32::MAX y dims no-enteras truncaban, corrompiendo el sparse vector decodificado en vez de devolver `None`.

**Resuelto por (vanta-worker):**
- **Validación en decode:** `!dim.is_finite() || dim < 0.0 || dim > u32::MAX as f64 || dim.fract() != 0.0` → retorna `None` (payload rechazado, mismo camino corrupto que odd-length). `u32::MAX as f64` es exacto (2^32−1 < 2^53). Dims no-enteras incluidas: `1.5 as u32 → 1` es el mismo bug class de pérdida silenciosa.
- **Warning actualizado:** `memory_record_from_node` loguea "malformed ListFloat pairs" en vez de "odd ListFloat length" (ya no describe todos los None).
- **Weights f32 no validados** — fuera del contrato P2-7 (solo dims).
- **Test de rechazo:** `test_sparse_read_corrupt_listfloat_invalid_dims_return_none` — NaN, +inf, negativa, >u32::MAX, no-entera → `None`. TDD: RED (fallaba con `Some(SparseVector({0: 0.5}))`) → GREEN.
- **Verify:** `cargo check -p vantadb` ✅; `cargo fmt --check` ✅; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅; `cargo nextest run --profile audit --workspace --build-jobs 2` → **1913 passed** ✅; `scripts/validate-docs-coverage.ps1` → 0 gaps ✅.

**Commit:** `(AUD-023)` — fix: validate sparse vector dims on decode (AUD-023)

**Ids:** `AUD-023`

### 2026-08-13 — AUD-024: Eliminar heap clones por op en drain_hnsw_batch_locked ✅

**Fuente:** Backlog `AUD-024` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`)

**Objetivo:** `drain_hnsw_batch_locked` clonaba bitset+vector por op (2 heap clones/insert) — iterar por valor tras `mem::take` para mover cada op en vez de clonarla.

**Resuelto por (vanta-worker):**
- **Refactor de ownership:** `for op in ops` (consume la Vec ya tomada del mutex vía `mem::take`) en vez de `for op in &ops`; `hnsw.add(op.id, op.bitset, op.vector, op.storage_offset)` sin `.clone()`. `HnswGraph::add` ya toma ambos por valor (src/index/graph.rs:596) → 0 clones de heap por insert en el drain.
- **Alcance:** también `try_push_pending_hnsw` (drain opportunista, ruta más caliente) — mismo anti-pattern, mismo root cause, mismo archivo.
- **Perf (FASE PERFORMANCE):** `cargo bench --bench bench_concurrent` (10k inserts secuenciales → path completo engine): **178.11s → 137.95s (-22.5%, -40.2s)**.
- **Verify:** `cargo check -p vantadb` ✅; `cargo fmt --check` ✅; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅; `cargo nextest run --profile audit --workspace --build-jobs 2` → **1913 passed** ✅; `scripts/validate-docs-coverage.ps1` → 0 gaps ✅. `rg "for op in &ops|bitset.clone|vector.clone"` en los drains → 0.
- **Sin cambio de comportamiento:** drain sigue vaciando el batch completo; tests de flush existentes (`test_flush_pending_hnsw_*`) cubren el path.

**Commit:** `e4c2ff8e` — perf: avoid per-op heap clones in drain_hnsw_batch_locked (AUD-024)

**Ids:** `AUD-024`

### 2026-08-13 — AUD-039: LRU eviction O(1) con crate `lru` en python bindings ✅

**Fuente:** Backlog `AUD-039` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`, finding P2-3)

**Objetivo:** `py_dict_to_metadata` cacheaba metadata con un LRU hand-rolled cuya evicción era O(n) (`min_by_key` scan sobre capacity 64) — swap a `lru::LruCache` (O(1), hash + lista doblemente enlazada).

**Resuelto por (vanta-worker):**
- **Reemplazo:** struct `LruCache` custom (convert.rs:26-70, 49 líneas) → `lru::LruCache<String, BTreeMap<String, VantaValue>>`; `const CACHE_CAPACITY: NonZeroUsize = 64` (match + `unreachable!()` sin args por E0015); call sites `cache.get(&key).cloned()` y `let _ = cache.put(...)` (`Option` es `#[must_use]`).
- **Deps:** `lru = "0.16"` agregada a `vantadb-python/Cargo.toml` — ya era dep directa del core (cli_server.rs) y estaba resuelta en el lockfile (0.16.4); sin crate nuevo ni bump. NO usar la 0.12.5 transitiva de tantivy.
- **Perf (FASE PERFORMANCE):** evicción O(1) documentada (lru) vs O(64) scan previo. Microbench venv: ~78-80 ops/s en thrash y hits — el cuello de botella es el engine (WAL+indexación), no la cache; sin regresión funcional (thresholds de `test_sustained_*` pasan).
- **Colateral:** `test_load.py` usaba `search(vector=[0.0]*dim)` → el core rechaza zero-norm cosine queries desde ERR-028 (b8058a26, pre-existente) — fix de test a query vector non-zero.
- **Verify:** `cargo check -p vantadb_py` ✅; fmt ✅; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅; `cargo nextest run --profile audit --workspace --build-jobs 2` → **1913 passed** ✅; pytest bindings → **85 passed** ✅; `scripts/validate-docs-coverage.ps1` → 0 gaps ✅.
- **Sin cambio de comportamiento:** cache thread-local privada, capacidad 64 preservada; `py_dict_to_metadata` (7 callers en lib.rs) sin firma modificada.

**Commit:** `af905c65` — perf: swap LRU eviction to O(1) lru crate in python bindings (AUD-039)

**Ids:** `AUD-039`

### 2026-08-13 — AUD-022: Pin SHA sccache-action (supply-chain CI) ✅

**Fuente:** Backlog `AUD-022` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`)

**Objetivo:** `mozilla-actions/sccache-action@v0.0.11` era la única acción externa del workspace sin pin SHA (ref mutable por tag) — higiene supply-chain CI (OpenSSF Scorecard).

**Resuelto por (vanta-lead):**
- **Pin SHA verificado:** `.github/actions/rust-setup/action.yml:73` → `mozilla-actions/sccache-action@fd02668681acd5f960e1372061bee5e3e987195c # v0.0.11`. SHA obtenido vía GitHub API (refs/tags/v0.0.11 → tag object) el 2026-08-13 — no confiar en memoria del modelo.
- **Convención de anotación:** `# v0.0.11` tras el SHA, alineado a AUD-028 (74 pins existentes con anotación).
- **Verify:** YAML parse OK (`yaml.safe_load` UTF-8); sin otros cambios en el action.

**Commit:** `(AUD-022)` — ci: pin sccache-action to verified SHA (AUD-022)

**Ids:** `AUD-022`

### 2026-08-13 — AUD-030: Gate de regresión bench en PRs + baseline auto-commiteado ✅

**Fuente:** Backlog `AUD-030` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`)

**Objetivo:** `heavy-bench-nightly-51.yml` solo corría en schedule/dispatch → el gate de regresión nunca validaba PRs; y el baseline `benchmarks/criterion_baseline.json` nunca se actualizaba (el modo `update-baseline` de `bench_regression.py` no tenía caller) → podía quedar stale.

**Resuelto por (vanta-lead):**
- **Trigger `pull_request`:** agregado con `paths` filter (benches/**, benchmarks/**, scripts/bench_regression.py, Cargo.toml) — solo PRs que tocan el sistema de bench disparan el gate; el resto no paga 2hrs de bench.
- **Auto-commit baseline:** step "Update and commit baseline (nightly only)" en el job `analyze` — corre `update-baseline` + commit/push del baseline. `if: github.event_name == 'schedule' && steps.check_regression.outputs.has_regression != 'True'` → PRs jamás mutan el baseline del repo y un run con regresión nunca se hornea como baseline. `permissions.contents: write` (antes read).
- **Verify:** YAML parse OK UTF-8; grep confirma `steps.check_regression.outputs.has_regression` coincide con el `id: check_regression` existente (línea 159).

**Commit:** `(AUD-030)` — ci: run bench regression gate on PRs + auto-commit baseline (AUD-030)

**Ids:** `AUD-030`

### 2026-08-13 — AUD-028: Anotar 78 SHA pins con versión (# vX.Y.Z) en GitHub Actions ✅

**Fuente:** Backlog `AUD-028` (derivado del audit full 2026-08-12, `docs/reviews/audit-full-20260812-231204.md`)

**Objetivo:** los pins de acciones externas por SHA (higiene supply chain / OpenSSF Scorecard) eran ilegibles sin el tag semver — agregar `# vX.Y.Z` a cada `uses: repo@sha` sin anotar en `.github/**`.

**Resuelto por (vanta-lead):**
- **Inventario real:** 146 usos `uses:` totales en `.github/**/*.yml`; 68 ya anotados + **78 sin anotar** (el audit decía 74; el re-scan post-edición detectó 4 `pypa/gh-action-pypi-publish` que el primer map de edición había omitido → 78). 15 repos upstream, 12 archivos.
- **Resolución de versiones contra tags reales (NO memoria del modelo):** `git ls-remote --tags` para tags semver exactos (upload-artifact→v4.6.2, setup-python→v5.6.0, download-artifact→v4.3.0, cache→v4.3.0, github-script→v7.1.0, gh-release→v3.0.2, pypi-publish→v1.14.0, maturin→v1.51.0, wasm-pack→v0.4.0, configure-pagefile→v1.5, setup-rust-toolchain→v1, install-action@43aecc8d→v2.83.2) y `git clone --filter=blob:none` + `git describe --tags` para commits intermedios (rust-cache@7e35be21→v2.9.1, install-action@25f25a6e→v2.83.4, attest-build-provenance→v4.1.1 — este último es tag object que peels al commit). dtolnay/rust-toolchain: único tag del repo es `v1` (commits 2026 post-tag) → `# v1`.
- **Aplicación:** script PowerShell aditivo (`uses: repo@SHA` → `uses: repo@SHA # vX.Y.Z`), preserva indentado. **Los SHAs quedan intactos** — solo comentario.
- **Verify:** grep de pins sin anotar = **0**; actionlint 10/10 workflows OK (action.yml es composite action → no aplica, esperado); YAML parse 23/23 OK (excl `.github/workflows-dl/` que no es código del repo). Diff verificado 100% aditivo.
- **Review P2-01 (vanta-audit):** ✅ approve — 16/16 correspondencias SHA→versión verificadas contra `ls-remote`/`describe` independientes, 0 mismatches de SHA, 0 pins restantes.

**Commit:** `8e9f5eb1` — ci: annotate pinned actions with version tags (AUD-028)

**Ids:** `AUD-028`
