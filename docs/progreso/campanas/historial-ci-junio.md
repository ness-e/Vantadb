# Historial junio — heavy certification, CI batch, jemalloc

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Historial de Tareas Completadas

### [2026-06-22] Fix de Fallas de Heavy Certification Workflow

**Objetivo:** Corregir los 4 tests que causaban fallas en la pipeline `VantaDB Heavy Certification` de GitHub Actions.
- **Checklist:**
  - [x] Fix `test_stale_lock_recovery` en `tests/file_locking_stress.rs` (asserción incorrecta sobre el contenido del archivo de lock)
  - [x] Cambiar `BackendKind::InMemory` → `BackendKind::Fjall` en 3 tests de `tests/storage/wal_resilience.rs`
  - [x] Eliminar `wal_write_failure_returns_error` de `tests/edge_cases.rs` (test roto en Unix)
  - [x] Añadir `test_wal_write_failure_simulated` con failpoints en `tests/storage/wal_resilience.rs`
  - [x] Añadir step `bash scripts/download_benchmark_datasets.sh` en `.github/workflows/heavy_certification.yml`
  - [x] Validación local: `edge_cases` (24/24 ✅), `test_stale_lock_recovery` (✅)

**Archivos modificados:**
- `tests/file_locking_stress.rs` — Fix de asserción stale del lock
- `tests/storage/wal_resilience.rs` — 3x InMemory→Fjall + nuevo test de failpoint
- `tests/edge_cases.rs` — Eliminado test roto de permisos Unix
- `.github/workflows/heavy_certification.yml` — Añadido step de descarga de datasets

### [2026-06-22] Fixes Batch de CI/CD + Locking de StorageEngine (TSK-134/135/138/140/126/128/129)

**Objetivo:** Limpiar los workflows de CI/CD y hacer robusto el sistema de locking del StorageEngine.

**Checklist CI/CD:**
- [x] TSK-134: Swap validado en `release.yml` — lógica correcta, sin cambios necesarios
- [x] TSK-135: `python_wheels.yml` — `dtolnay/rust-toolchain@master` → `@stable`
- [x] TSK-138: Eliminado checkout duplicado en `rust-setup/action.yml`
- [x] TSK-140: Eliminado job ARM64 muerto (`if: false`) en `python_wheels.yml` (-69 líneas)
- [x] TSK-141: Eliminado `librocksdb-dev` de `rust-setup/action.yml` (sesión anterior)

**Checklist Locking de StorageEngine:**
- [x] TSK-126: `impl Drop for StorageEngine` — liberar lock `fs2` explícitamente al destruir
- [x] TSK-128: timeout de `insert_lock` configurable vía `VANTADB_INSERT_LOCK_TIMEOUT_MS` (default 2000ms)
- [x] TSK-129: timeout de `.vanta.lock` configurable vía `VANTADB_FILE_LOCK_TIMEOUT_MS` (default 1000ms)

**Archivos modificados:**
- `src/config.rs` — +2 campos de struct (`insert_lock_timeout_ms`, `file_lock_timeout_ms`) + impl Default
- `src/storage.rs` — +impl Drop, 5× `lock()` → `try_lock_for()`, `refresh_index()` → `Result<()>`
- `.github/workflows/python_wheels.yml` — -69 líneas (job ARM64 muerto), toolchain stable
- `.github/actions/rust-setup/action.yml` — -checkout duplicado

### [2026-06-22] Instrumentación jemalloc + Swap CI/CD (TSK-130/137)

**Objetivo:** Instrumentar estadísticas detalladas de drift de memoria heap (jemalloc stats) y añadir espacio de swap para Windows/macOS en CI/CD.

**Checklist Jemalloc (TSK-130):**
- [x] Añadir dependencias Unix-only `tikv-jemallocator` y `tikv-jemalloc-ctl`.
- [x] Configurar `global_allocator` condicionalmente en CLI y Server.
- [x] Recolectar estadísticas (`allocated`, `active`, `metadata`, `resident`, `mapped`, `retained` bytes) y exponerlas a Prometheus y snapshots.
- [x] Soportar mapeos de estas métricas en Python y testing de serialización.

**Checklist Swap CI/CD (TSK-137):**
- [x] Configurar pagefile (8-16GB) para Windows en `release.yml` y `python_wheels.yml`.
- [x] Liberar espacio eliminando cache en macOS para permitir paging dinámico en `release.yml` y `python_wheels.yml`.

**Archivos modificados:**
- `Cargo.toml` — dependencias condicionales Unix para jemalloc
- `vantadb-server/Cargo.toml` — feature `jemalloc` y dependencias Unix
- `src/bin/vanta-cli.rs` — global allocator condicional
- `vantadb-server/src/main.rs` — global allocator condicional
- `src/metrics.rs` — gauges jemalloc, actualización de snapshot
- `src/sdk.rs` — campos jemalloc en VantaOperationalMetrics
- `vantadb-python/src/lib.rs` — mapeo en el Python SDK
- `tests/sdk_serialization.rs` — test de serialización de métricas
- `.github/workflows/release.yml` — pagefile/swap en CI/CD Windows/macOS
- `.github/workflows/python_wheels.yml` — pagefile/swap en CI/CD Windows/macOS
