# Cleanup List — Archivos a eliminar

> Auditoría completa de archivos basura, duplicados y configs obsoletas en el monorepo.
> Basado en exploración del 2026-07-02.

---

## 🔴 Alta Prioridad (basura segura, gitignored)

| Archivo/Directorio | Tamaño | Gitignored | Razón |
|--------------------|--------|------------|-------|
| `ghost_out/` | ~128 MB | ✅ | Base de datos runtime de tests |
| `_test_async/` | ~67 MB | ✅ | Base runtime de tests |
| `_test_import/` | ~67 MB | ✅ | Base runtime de tests |
| `_test_listfmt/` | ~67 MB | ✅ | Base runtime de tests |
| `test/` | ~128 MB | ✅ | Base runtime de tests |
| `vector_index.bin` | 253 B | ✅ | Binario stale |
| `job_failpoint_injections.log` | 0 B | ✅ | Log vacío |
| `job_memory_concurrency.log` | 0 B | ✅ | Log vacío |
| `job_other_heavy.log` | 0 B | ✅ | Log vacío |
| `job_storage_persistence.log` | 0 B | ✅ | Log vacío |

## 🟡 Media Prioridad

| Archivo/Directorio | Tamaño | Gitignored | Acción |
|--------------------|--------|------------|--------|
| `db/` | ~64 MB | ✅ | Base runtime, borrar si no hay sesión activa |
| `quickstart_db/` | ~64 MB | ✅ | Base runtime quickstart |
| `vantadb_data/` | ~64 MB | ✅ | Base runtime server |
| `job_log.txt` | 328 KB | ✅ | Log de job CI, stale (2026-06-20) |
| `vanta_certification.json` (root) | 361 KB | ✅ | Duplicado — copia canónica en dev-tools/reports/ |
| `vanta_certification.json` (vantadb-server/) | 69 KB | ❌ | No gitignored — borrar o mover |
| `vanta_benchmark_report.json` (root) | 463 B | ✅ | Duplicado — copia más reciente en benchmarks/ |
| `vanta_benchmark_report.json` (benchmarks/) | 783 B | ❌ | Mantener como canónico |
| `web/.playwright-mcp/` | ~26 archivos | ❓ | Historial de sesiones MCP, borrable |
| `web/.tanstack/tmp/` | Temp | ✅ | Temp de codegen |

## 🟢 Baja Prioridad (housekeeping)

| Archivo/Directorio | Gitignored | Acción |
|--------------------|------------|--------|
| `vantadb-python/__pycache__/` | ✅ | Clean opcional |
| `vantadb-python/.pytest_cache/` | ✅ | Clean opcional |
| `vantadb-python/.venv/` | ✅ | Clean opcional |
| `scripts/__pycache__/` | ✅ | Clean opcional |
| `packages/*/.pytest_cache/` | ✅ | Clean opcional |
| `packages/*/*.egg-info/` | ✅ | Clean opcional |

## ✅ Archivos a CONSERVAR (no basura)

| Archivo | Razón |
|---------|-------|
| `Cargo.toml`, `Cargo.lock` | Core del proyecto |
| `src/`, `tests/` | Código fuente |
| `benches/`, `benchmarks/` | Benchmarks activos |
| `docs/` | Documentación del producto |
| `.github/workflows/` | CI/CD pipelines (6 activos) |
| `vantadb-python/`, `vantadb-ts/`, `vantadb-wasm/`, `vantadb-mcp/`, `vantadb-server/` | Crates activos |
| `web/` | Frontend web (recién integrado) |
| `dist/` | Wheel Python compilado |
| `.vanta_profile` (all copies) | Hardware profiles intencionales |
| `README.md`, `CONTRIBUTING.md`, `LICENSE` | Proyecto |
| `deny.toml`, `cliff.toml`, `rust-toolchain.toml` | Config |
| `completions/` | Shell completions (generados por build.rs) |

## 📊 Sumario de Impacto

| Categoría | Archivos | Tamaño Recuperable |
|-----------|----------|-------------------|
| 🔴 Alta | ~10 items | ~457 MB |
| 🟡 Media | ~10 items | ~193 MB + configs |
| 🟢 Baja | ~5 items | Variable |
| **Total** | **~25 items** | **~650 MB** |
