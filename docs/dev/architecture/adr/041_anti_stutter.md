---
title: "ADR-041: anti-stutter total (eliminar prefijo Vanta redundante)"
type: adr
status: proposed
tags: [vantadb, architecture, adr, naming, breaking]
created: 2026-09-11
last_reviewed: 2026-09-11
---

# ADR-041: Anti-stutter total (eliminar prefijo Vanta redundante)

> **Forcing function (Regla 5):** la IA aportó solo evidencia (inventarios `rg` + blast radius CodeGraph, ver § Evidencia). El Contexto, la Decisión y las Consecuencias los articula y firma el autor humano abajo. Sin firma, este ADR sigue `proposed` y AST-002 no arranca.

## Status

Proposed — pendiente firma humana (`Firmado por:` al pie).

## Date

2026-09-11

## Context

<!-- HUMANO: problema o trade-off que motiva la decisión, con tus palabras -->

Cada símbolo público repite el nombre de su contenedor: `vantadb::VantaConfig`, `vantadb::VantaMemoryRecord`, `import { VantaDB } from "vantadb"`. El prefijo `Vanta` no agrega información — el contenedor (crate, paquete, módulo) ya dice "Vanta". El costo es ergonomía: nombres largos, imports ruidosos, y divergencia entre bindings (cada SDK resolvió el stutter a su manera o no lo resolvió).

La alternativa es conservar el prefijo como branding ("todo lo nuestro empieza con Vanta"). El tradeoff real: branding explícito en cada símbolo vs ergonomía y convención del ecosistema (ninguna crate seria repite su nombre en cada tipo: `serde::Value`, no `serde::SerdeValue`).

Mapa congelado de la decisión: `scripts/anti_stutter_map.json` (38 símbolos Rust + 10 TS + 3 d.ts + 8 clases Python + 4 métodos Python + 5 métodos Rust + 6 exclusiones + 4 decisiones abiertas).

## Decision

<!-- HUMANO: qué se eligió y por qué sobre las alternativas -->

Eliminar el prefijo `Vanta` redundante siguiendo la regla universal — ninguna entidad repite su contenedor (`vantadb::VantaConfig` → `vantadb::Config`) — con estrategia de aliases deprecated por binding para no romper consumidores:

- **Rust:** `pub type Viejo = Nuevo` + `#[deprecated]` en re-exports (`src/lib.rs`, `src/sdk/mod.rs`, `src/sdk/types.rs`); el alias se quita en el major siguiente.
- **Python:** alias en `vantadb_py/__init__.py` + `.pyi` con `DeprecationWarning` (patrón PY-03 existente).
- **TypeScript:** `export type Viejo = Nuevo` + `@deprecated`; nunca `Error` pelado (colisiona con el global) → `DbError`.
- **Release:** rename público = bump major vía release-plz (`semver_check=true`); canary TestPyPI / `npm --tag next` 24–48h antes del publish final (lo ejecuta AST-007).

Exclusiones deliberadas: `VantaHeader` (formato on-disk, compat binaria), códigos `VANTADB_*` (wire entre bindings), funciones libres `memory_*` (sin contenedor que repitan).

## Consequences

<!-- HUMANO: costos, riesgos y deuda asumida -->

- **Pros:** ergonomía (`Config`, `MemoryRecord`, `Client`); paridad real entre Rust/TS/Python/WASM gobernada por un solo mapa; el major limpia años de stutter de una vez en vez de goteo eterno.
- **Cons / costos:** breaking change — todos los consumidores migran (mitigado por aliases deprecated + codemod único desde el mapa + docs sync en el mismo PR, AST-006); riesgo de fusión incorrecta `VantaSearchHit`/`VantaMemorySearchHit` si el wire difiere (OD-3, owner AST-002); colisión `size` en AST-005 si un trait ya lo define.
- **Deuda asumida:** aliases deprecated viven hasta el major siguiente (cleanup obligatorio ≤2 releases); 4 decisiones abiertas (OD-1…OD-4) con owner y gate asignados — si OD-1 u OD-2 siguen abiertas al cierre de AST-004/AST-003, el plan se pausa (stop conditions del plan).
- **Rollback:** revert del Release PR + re-publish patch con aliases restaurados (plan §2b; detalle en AST-007).

## Evidencia (aporte IA — no es la decisión)

- `rg "pub (struct|enum) Vanta" src/` → 35 hits, 39 símbolos únicos (38 renameables + `VantaHeader` excluido); refs: `src/config.rs:248`, `src/error.rs:122`, `src/sdk/types/record.rs:13,24,35,79`, `src/storage/vfile.rs:110`.
- `rg "export (interface|type|class) Vanta" vantadb-ts/src/` → 10 hits (`types.ts` 7 + `errors.ts` 2 + `vantadb.ts:134 `VantaDB``).
- `rg "class Vanta|def (search_memory|get_memory|list_memory|delete_memory)" vantadb-python/` → 8 clases PyO3 + 4 métodos `AsyncVantaDB` (`__init__.py:144-240`).
- Blast radius CodeGraph: `VantaSearchHit` 6 callers; re-exports vía `src/sdk/mod.rs:15`, `src/sdk/types.rs:11,19`; tests `tests/query_result_basic.rs`.
- Referencias: `.opencode/rules/api-contract.md` (R-1: todo claim apunta a símbolo real), `docs/api/BINDINGS_NAMESPACES.md` (naming hazard `get`/`delete`), `release-plz.toml` (`semver_check=true`), `CONSTRAINTS.md` (floor: sin supresiones/stubs/secrets).

---

**Firmado por:** _pendiente — el autor humano firma aquí (nombre + fecha) al aprobar este ADR._
