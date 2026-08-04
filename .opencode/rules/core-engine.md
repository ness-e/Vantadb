# Core Engine — Reglas

> **Scope:** núcleo del motor `src/` — `node.rs`, `engine.rs`, facade `storage/engine/mod.rs`, `config.rs`, `error.rs`, `planner/executor/query` (flujo IQL)
> **No tocar aquí:** durabilidad/WAL (`durability.md`), índices (`indexes.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-011 (feature-gating audit)

## Reglas

### R-1: Features experimentales con feature-gating `experimental-*` — sin dependencias en default

- **Must:** todo módulo o capacidad experimental (ej. `Governor::ingest`, storage partition no predeterminado) compilarse detrás de un feature flag `experimental-*` en `Cargo.toml`.
- **Must not:** habilitar módulos experimentales en `default = [...]` del workspace ni en builds release de producción; tampoco dejar sin gate capacidades marcadas experimental en specs (INV-011: `Governor` fue diseñado experimental y quedó sin flag).
- **Por qué:** un módulo experimental sin gate entra en builds de producción por accidente, expone API inestable y contradice el contrato `spec_declares_*` que la suite de tests exige (INV-011).

### R-2: Funciones internas sin callers documentadas como tal — no exportar al SDK

- **Must:** marcar (comentario + `#[doc(hidden)]` o `pub(crate)`) funciones internas que no tienen callers externos verificados (ej. `run_loop` en INV-003) para que no aparezcan en la API pública.
- **Must not:** exponer en el SDK público funciones que solo se invocan internamente, aumentando la superficie de semver.
- **Por qué:** cada símbolo público es un contrato semver; lo que no se usa fuera del crate no debe prometerse (INV-003 §matices).

<!-- Referencias cruzadas: → ver durability.md, api-contract.md, indexes.md -->
