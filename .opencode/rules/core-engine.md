# Core Engine — Reglas

> **Scope:** núcleo del motor `src/` — `node.rs`, `engine.rs`, facade `storage/engine/mod.rs`, `config.rs`, `error.rs`, `planner/executor/query` (flujo IQL)
> **No tocar aquí:** durabilidad/WAL (`durability.md`), índices (`indexes.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-011 (feature-gating audit), auditoría código 2026-08-04

## Reglas

### R-1: Features experimentales con feature-gating `experimental-*` — sin dependencias en default

- **Must:** todo módulo o capacidad experimental (ej. `Governor::ingest`, storage partition no predeterminado) compilarse detrás de un feature flag `experimental-*` en `Cargo.toml`.
- **Must not:** habilitar módulos experimentales en `default = [...]` del workspace ni en builds release de producción; tampoco dejar sin gate capacidades marcadas experimental en specs (INV-011: `Governor` fue diseñado experimental y quedó sin flag).
- **Por qué:** un módulo experimental sin gate entra en builds de producción por accidente, expone API inestable y contradice el contrato `spec_declares_*` que la suite de tests exige (INV-011).

### R-2: Funciones internas sin callers documentadas como tal — no exportar al SDK

- **Must:** marcar (comentario + `#[doc(hidden)]` o `pub(crate)`) funciones internas que no tienen callers externos verificados (ej. `run_loop` en INV-003) para que no aparezcan en la API pública.
- **Must not:** exponer en el SDK público funciones que solo se invocan internamente, aumentando la superficie de semver.
- **Por qué:** cada símbolo público es un contrato semver; lo que no se usa fuera del crate no debe prometerse (INV-003 §matices).

### R-3: Prohibición de unwrap/expect fuera de tests

- **Must:** propagar errores con `?` y el alias `crate::error::Result<T>` en todo código no-test.
- **Must not:** usar `.unwrap()`, `.expect()`, `.unwrap_or_else(|_| ...)` que enmascare un error real en rutas no-test. Única excepción sancionada: los helpers `RwLockExt::lock_rwlock*`/`MutexExt` de `src/sync_ext.rs` (un lock envenenado es invariante irrecuperable del proceso).
- **Por qué:** 1904 unwraps en 62.7K LOC (1 por cada ~33 líneas); un panic en hot path (search/ingest) tumba el proceso entero en vez de retornar `VantaError` al binding. La auditoría confirmó pánicos en capa baja (`src/storage/engine/ops.rs:1761`).

### R-4: Todo `unsafe` requiere `// SAFETY:` + invariante

- **Must:** cada bloque `unsafe` lleva `// SAFETY:` que documente el invariante (lifetime, alineación, alias) y nombre la propiedad que lo garantiza.
- **Must not:** introducir `unsafe` nuevo sin pago de deuda equivalente (AGENTS.md Regla 6: saldo neto cero o negativo) ni sin test de Miri cuando el bloque es FFI/transmute.
- **Por qué:** 86 usos de `unsafe` en 16 archivos de `src/` (`index/distance.rs` 23, `storage/vfile.rs` 22 concentran 52); la Regla 4 de AGENTS.md lo exige pero ninguna regla de área lo codificaba.

### R-5: Convención de configuración por env vars — prefijo único

- **Must:** prefijo único `VANTADB_*` para todas las env vars; lectura vía helper `parse_env_or::<T>`/`env::var(...).ok()`; valor no reconocido → `warn!` + default documentado, nunca panic.
- **Must not:** leer la misma var con `unwrap_or_else` a mano en cada sitio ni duplicar prefijos (hoy conviven `VANTA_DB` en `src/cli.rs:15`/`cli_handlers/server.rs:244` con `VANTADB_STORAGE_PATH` en `src/config.rs:408` — mismo concepto, dos nombres).
- **Por qué:** el operador no puede predecir el nombre de la var; la auditoría DOC3 confirmó la inconsistencia de naming entre capas (F13).

<!-- Referencias cruzadas: → ver durability.md, api-contract.md, indexes.md, concurrency-async.md -->
