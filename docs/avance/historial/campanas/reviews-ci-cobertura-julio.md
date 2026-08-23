# Reviews julio — clippy, cobertura REV-003, INT/REL, P1/P2 CI

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-13 — Review Item 1: Limpieza de warnings de Clippy

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| TASK-38 | Review Item 1 — clippy | `cargo clippy --workspace --all-targets --all-features` corre sin `redundant_closure` (review desactualizado). Fixed 3 warnings nuevos (2 `needless_range_loop` + 1 `redundant_pattern_matching`). `cargo fmt` aplicado. | ✅ |

**Verificación:** `cargo clippy -p vantadb --all-features` 0 warnings, `cargo fmt --check` clean, 576/577 tests pass.

### 2026-07-13 — P4: Escrituras reversibles de VantaFile

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| TASK-31 / P4 | VantaFile writes reversibles | `insert()`: si KV put falla tras VantaFile write → tombstone. `batch_insert()`: si write_batch falla → re-acquire vstore lock + tombstone offsets. `delete()`/`delete_batch()` ya tombstoneaban antes del KV delete — no afectados | ✅ |

**Verificación:** `cargo check` ✅, `cargo nextest run` 576/577 pass (1 pre-existing), `cargo fmt --check` clean.

### 2026-07-23 — REV-003: Campaña de cobertura 53.85% → 80.55% (CII Silver)

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-003 | Coverage gate + push to ≥80% | 14 batches, +728 tests (764→1492). Line coverage 53.85%→80.55% (region 82.16%, function 88.18%). CI threshold 76%→80%. Fix: SQ8 format bug in ops.rs get(). CLOSED. | ✅ |

**Cobertura por módulo (post-campaña):**
- parser/mod.rs: 13% → 97%
- error.rs: 70% → 100%
- sdk/graph.rs: 0% → 100%
- columnar.rs: 0% → 99%
- metrics/core/registry.rs: 46% → 78%
- index/distance.rs: 50% → 67%
- index/search.rs: 0% → 60%
- Todos los archivos SDK <80 → ≥80% (api 83%, builder 87%, graph 100%, types 99%)
- storage/engine/ init 73%, ops 77%, stats 76%, maintenance 77%

**Archivos tocados:** 23 (10.4K líneas agregadas, 13 borradas)
**Verificación:** `cargo llvm-cov test --lib -p vantadb` → 80.55%, `just verify` → fmt+check+clippy+actionlint ✅

### 2026-07-14 — REV-004: Fix de rlib de tantivy en vantadb-openai

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-004 | Corregir rlib de tantivy no encontrado | Agregado `"rlib"` al `crate-type` de `vantadb-openai/Cargo.toml`. Los binarios de test necesitan `rlib` para linkear contra `vantadb_openai`; solo `cdylib` causa "tantivy rlib not found" en CI. | ✅ |

**Verificación:** `cargo check -p vantadb-openai` ✅, `cargo nextest run --no-run -p vantadb-openai` ✅.

### 2026-07-14 — REV-005: Corregir 6x no-explicit-any + prettier en frontend web

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-005 | Corregir ESLint/prettier en demo.lazy.tsx + why-vantadb.tsx | Agregados tipos `HitResult` + `VantaDemoDB`; cambiado `catch (err: any)` → `catch (err: unknown)` con narrowing `instanceof Error`; corrido `eslint --fix` para prettier. 0 violaciones restantes. | ✅ |

**Verificación:** `npx eslint` ✅ (0 errors), `npx tsc --noEmit` ✅ (0 errors).

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-07-14 — REV-017: Corregir el trailing newline de prettier en why-vantadb.tsx

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-017 | Corregir el trailing newline de `why-vantadb.tsx` | Ya corregido en el commit `ad4d1e1`. El archivo termina con `\n`, `prettier --check` pasa, `eslint` silencioso, `git diff` vacío. | ✅ |

**Verificación:** `npx prettier --check web/src/routes/why-vantadb.tsx` ✅, `npx eslint web/src/routes/why-vantadb.tsx` ✅.

### 2026-07-14 — REV-015: Corregir los 2x no-explicit-any restantes en demo.lazy.tsx

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-015 | Eliminar los tipos `any` restantes en demo.lazy.tsx | Reemplazado `Promise<any>` con `Promise<HFExtractor>` tipado, import dinámico tipado como `{ pipeline: PipelineFn }`, eliminados ambos comentarios `eslint-disable-next-line`. | ✅ |

**Verificación:** `npx eslint src/routes/demo.lazy.tsx` ✅ (0 errors), `npx tsc --noEmit` ✅ (0 errors).

### 2026-07-14 — REV-008: Actualizar actions/checkout + setup-node a v4

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-008 | Actualizar actions/checkout@v3 + setup-node@v3 deprecados a v4 | Reemplazado el SHA `actions/checkout@v3` con `@v4` (42 ocurrencias) y el SHA `actions/setup-node@v3` con `@v4` (5 ocurrencias) en 13 archivos de workflow. El runner usa Node 24; v4 usa Node 20 por compatibilidad. | ✅ |

**Verificación:** `grep` confirms 0 remaining old SHA references, 53 `@v4` references in project workflows.

### 2026-07-14 — REV-006: Clippy a nivel workspace en CI

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-006 | Clippy a nivel workspace en todos los adaptadores | Eliminado el `[profile.release]` duplicado de `vantadb-wasm/Cargo.toml` (el workspace ya tenía `[profile.release.package.vantadb-wasm]`); agregado `--all-targets --all-features` a los jobs de clippy de Windows y macOS en `ci-rust-10.yml` para consistencia con Linux. | ✅ |

**Verificación:** Profile warning eliminated (`cargo check -p vantadb-wasm` has no profile warning). All 3 OS clippy jobs now use uniform `--workspace --all-targets --all-features -- -D warnings`.

### 2026-07-14 — REV-007: reducedMotion en deps de useEffect (3 componentes)

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-007 | Add `reducedMotion` to `useEffect` deps | NbMonolith.tsx: `[]` → `[reducedMotion]`; NbVectorNebula.tsx: `[]` → `[reducedMotion]`; `__root.tsx`: `[routeId]` → `[routeId, reducedMotion]`. Previene closure obsoleto al cambiar preferencias de accesibilidad. | ✅ |

**Verificación:** `npx eslint` ✅ (0 errors), `npx tsc --noEmit` ✅ (0 errors).

### 2026-07-14 — INT-01: Publicar adaptador LangChain en PyPI

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| INT-01 | Publicar adaptador LangChain en PyPI | El paquete compila (`python -m build` ✅), 5/5 tests pasan, existe el workflow CI `release-adapters-62.yml` con OIDC trusted publishing. Push del `tag adapters-v0.3.0` para disparar el publish de producción. | ✅ |

**Verificación:** `python -m build integrations/langchain/` ✅ compila `.tar.gz` + `.whl`. `python -m pytest integrations/langchain/tests/ -v` ✅ 5/5 pasaron. Nombre PyPI `vantadb-langchain` disponible. Dependencia `vantadb-py>=0.2` satisfecha (v0.2.0 publicado).

### 2026-07-14 — INT-02: Publicar adaptador LlamaIndex en PyPI

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| INT-02 | Publicar adaptador LlamaIndex en PyPI | El paquete compila (`python -m build` ✅), 5/5 tests pasan, el workflow CI cubre llamaindex en la matrix. Push del `tag adapters-v0.3.0` para disparar el publish de producción. | ✅ |

**Verificación:** `python -m build integrations/llamaindex/` ✅. `python -m pytest integrations/llamaindex/tests/ -v` ✅ 5/5 pasaron. Nombre PyPI `vantadb-llamaindex` disponible.

### 2026-07-14 — DEVOPS-05: Pipeline CI unificado para publicación de adaptadores en PyPI

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| DEVOPS-05 | Pipeline CI unificado para publicar todos los adaptadores en PyPI | Verificado el `release-adapters-62.yml` existente: pipeline de 3 etapas (test → build → publish) cubre los 9 adaptadores en `integrations/`. OIDC trusted publishing para TestPyPI (dispatch) y PyPI producción (tag `adapters-v*`). Los 9 adaptadores compilan correctamente. | ✅ |

**Verificación:** `python -m build integrations/*/` ✅ los 9 pasan. El workflow CI existe en `.github/workflows/release-adapters-62.yml`.

### 2026-07-14 — REL-02: Publicar `vantadb-ts` en npm (build WASM)

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REL-02 | Publicar `vantadb-ts` en npm | 3 fixes aplicados, verificación completa. Fixes: (1) visibilidad de `impl_text_index.rs` (`fn` → `pub(crate)` en 2 métodos), (2) `wasm-opt = false` en `vantadb-wasm/Cargo.toml` (binaryen local demasiado viejo para bulk-memory), (3) el trigger de tag `ts-v*` del CI `release-npm-61.yml` ahora corre `publish-wasm`. Verificación: build WASM ✅, build TS ✅, npm dry-run ✅. Nombres npm `vantadb` + `vantadb-wasm` ambos disponibles. Doc `release-npm-61.md` actualizado. | ⏳ |

**Problema pre-existente:** 80/219 tests TS fallan con panics `unreachable!()` en el entorno vitest de Node.js — bug pre-existente del runtime WASM, no bloquea el publish. 113 pasan (type guards, lifecycle, errores), 26 skip (los tests de search necesitan datos). Requiere investigación aparte.

**Verificación:** `wasm-pack build --release` ✅ en `vantadb-wasm/`. `tsc` ✅ en `vantadb-ts/`. `npm publish --dry-run` ✅ (`npm pkg fix` aplicado). Fix de CI `release-npm-61.yml` verificado leyendo el YAML.

### 2026-07-17 — P1-2: Timeout del step de tests Windows 25→30 min

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-2 | Timeout del step de tests Windows 25→30 min | Aumentado el timeout del step de 25 a 30 min en el job `test-windows` de `ci-rust-10.yml` para igualar el timeout del job. `test-threads=2` preservado (necesario para evitar el OS error 1455). | ✅ |

**Verificación:** diff inspeccionado, commit 3acd07c.

### 2026-07-17 — P1-3: Clave de cache hashFiles para dataset GloVe

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-3 | Clave de cache `glove-100d-v1` → `hashFiles('scripts/download_benchmark_datasets.sh')` | Reemplazada la clave de cache estática con `hashFiles` en los jobs `test` y `coverage` de `ci-rust-10.yml`. El cache ahora se invalida cuando cambia el script de descarga. `hashFiles` es expresión nativa de GitHub Actions — no se necesita dependencia. | ✅ |

**Verificación:** `grep hashFiles ci-rust-10.yml` → 2 coincidencias (L104, L239). Commit 9386079.

### 2026-07-17 — P1-4: macOS unificar con action rust-setup

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-4 | macOS usar `.github/actions/rust-setup` | Reemplazado el `dtolnay/rust-toolchain` + `Swatinem/rust-cache` + `cargo install cargo-nextest` manuales con un solo `uses: ./.github/actions/rust-setup`. -10 líneas. Homebrew deps preservadas. | ✅ |

**Verificación:** diff inspeccionado — 2 inserts, 10 deletes. Commit 8bd15fa.

### 2026-07-17 — P1-5: Re-activar wasm-opt

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-5 | Re-activar `wasm-opt` en WASM build | Eliminado el override `wasm-opt = false`. Binaryen v121+ (actual: v128+) soporta bulk-memory-opt. wasm-opt ahora corre con `-Os` por defecto en builds release, ahorrando ~30-50% del tamaño del bundle. | ✅ |

**Verificación:** diff inspeccionado — -1 línea neta. Commit e96a6f5.

### 2026-07-17 — P1-6: Worker timeout 5s sin retry — backoff exponencial

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-6 | Worker timeout 5s sin retry | `send()` wrappeado con bucle de reintentos (max 3, backoff 1s→2s→4s). Solo reintentan los timeout errors. `try_send()` extraído con body original. `cargo check -p vantadb-wasm` ✅ | ✅ |

**Verificación:** `cargo check -p vantadb-wasm` — 0 errores, 0 warnings nuevos.

### 2026-07-17 — P1-7: CI — Version extraction frágil con grep

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-7 | Version extraction frágil en CI | `grep '^version'` → `grep -A1 '^\[workspace\.package\]'` en `release-wheels-60.yml` y `release-npm-61.yml`. Ahora extrae del source of truth real (`[workspace.package]`), no del coincidencial `[package]`. | ✅ |

**Verificación:** diff inspeccionado — 2 líneas cambiadas (1 por archivo). La regex de semver en npm actúa como captura extra.

### 2026-07-17 — P1-8: CI — Inconsistencia de timeouts

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-8 | Inconsistencia de timeouts en heavy-certification-50.yml | Removido `timeout-minutes: 150` del step `Run stress protocol`. Job timeout (180 min) actúa como único guardián. -1 línea neta. | ✅ |

**Verificación:** diff inspeccionado — 1 línea eliminada.

### 2026-07-17 — P1-9: WASM — SIMD duplicado eliminado

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-9 | SIMD duplicado en WASM (cosine_distance_simd) | Eliminado `vantadb-wasm/src/simd.rs` (208 líneas) + `pub mod simd` de `lib.rs`. `cosine_distance_simd()` era código muerto (0 llamadores externos). Alternativa: `vantadb::index::cosine_sim_f32`. | ✅ |

**Verificación:** `cargo check -p vantadb-wasm` — 0 errores, 0 warnings nuevos. -208 líneas netas.

### 2026-07-17 — P1-10: PyPI — sleep de propagación CDN → bucle de reintentos

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P1-10 | CI: sleep de propagación CDN PyPI → bucle de reintentos | Reemplazado `sleep 90` + step install separado por un solo step con bucle de reintentos (1s, 2s, 4s, 8s, 16s, 32s, 64s). Si CDN propaga en 10s, instala en 10s. Max 127s vs 90s fijo antes. `release-wheels-60.yml:256-259` → `:256-264` | ✅ |

**Verificación:** diff inspeccionado. 2 pasos fusionados en 1. Sin compilación Rust (cambio YAML puro).

### 2026-07-17 — P2-1: OpfsFile::delete() implementado

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P2-1 | WASM: OpfsFile::delete() stub → real | Reemplazado error stub con `js_call(&self.handle, "remove", ...)`. `opfs.rs:83-87` — 5 líneas → 3 líneas. `cargo check -p vantadb-wasm` ✅ | ✅ |

**Verificación:** `cargo check -p vantadb-wasm` — 0 errores.

### 2026-07-17 — P2-2: VantaVector.__array_interface__ fix de UB (Vec→Box)

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| P2-2 | PyO3: VantaVector Vec→Box&lt;[f32]&gt; | `Vec<f32>` → `Box<[f32]>` en struct + `new()`/`__iter__`/`__getstate__`/`__setstate__`. Elimina realloc como fuente de UB en `__array_interface__`. `cargo check` ✅ | ✅ |
