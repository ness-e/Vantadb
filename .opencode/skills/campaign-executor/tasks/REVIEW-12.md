# TASK-REVIEW-12: Split api.rs (~2610L) por dominio (memory/search/namespaces/admin/graph)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (W24-2, line 1212)
- **Fuente:** review-full-20260822 H06-ARCH-002
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (refactor aditivo)
- **Turns estimados:** 5 steps
- **Creado:** 2026-08-30T21:00
- **last-synced:** 2026-08-30T22:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0
- **Campaign ID:** full-20260829-parallel

## Blast Radius (mapeado, Regla 0)

| Dirección | Módulos |
|-----------|---------|
| Callers (internos) | `src/lib.rs:164` re-exporta `BulkImportReport` desde `sdk::api`; `src/cli_server.rs:2362` comentario |
| Callers (externos) | `vanta-proxy/src/memory_tools.rs:360`, desktop/TS bindings usan `VantaEmbedded` directamente (NO `api::*`) |
| Callees | `crate::backend`, `crate::error`, `crate::executor`, `crate::node`, `crate::config`, `crate::metrics`, `crate::storage::engine`, `super::builder::VantaEmbedded`, `super::serialization::*`, `super::types::*` |
| Implicaciones | Ningún break público: solo `BulkImportReport` era público en `sdk::api`, preservado |

- **Veredicto impacto:** BAJO

## Contrato

```
(Get-ChildItem src/sdk/*.rs | Measure-Object).Count >= 4
AND
cargo check -p vantadb --all-targets == 0
```

## Resultado verificado (2026-08-30)

- `(Get-ChildItem src/sdk/*.rs | Measure-Object).Count` = **8** (>= 4) ✅
- `(Get-ChildItem src/sdk/api/*.rs | Measure-Object).Count` = **5** (memory, graph, admin, search, namespaces) ✅
- `cargo check -p vantadb --all-targets` = **0 errores** ✅
- `rustfmt --check` en los 6 archivos del scope = **pass** ✅
- `cargo clippy -p vantadb --lib -- -D warnings` = **0 errores** ✅
- Pre-existente (fuera de scope): clippy `assertions_on_constants` en `src/config.rs:1767` (commit WSM-04) y `mod auth_tests` roto en `src/server/routing.rs:4982` (deuda de otra tarea sin commitear). NO introducidos por REVIEW-12.

## Spec (SDD — refactor)

| # | Decisión | Resuelto |
|---|----------|----------|
| 1 | Estrategia: sub-módulo `sdk::api::` vs archivos planos | ✅ Decidido por evidencia — A) `sdk/api/{memory,...}.rs` preserva `pub use api::BulkImportReport` |
| 2 | Tests inline | ✅ Mantenidos en `api.rs` con `super::*` |
| 3 | Helpers privados | ✅ Movidos a `memory.rs` con `pub(super)` para reuso cross-module |
| 4 | Re-exports | ✅ `pub use memory::BulkImportReport;` preserva el path |

## Invariantes de dominio (MUST)

- **Invariantes preservadas:**
  - Firmas de `VantaEmbedded::*` idénticas (97 funciones, 5 archivos de impl blocks distribuidos)
  - `BulkImportReport` re-exportado desde `sdk::api` y `lib.rs`
  - Multi-impl blocks de Rust resuelven a través de archivos en `src/sdk/api/`
  - `mod tests` en `api.rs` usa `super::*` que resuelve correctamente vía `pub mod`
- **Comandos de verificación:** ver arriba
- **Deuda pendiente:** clippy pre-existente en `src/config.rs:1767` y módulo roto en `src/server/routing.rs` (REVIEW-10/SRV-*)

## Recitation (canónico — RESULTADO final)

```
activeGoal: TASK-REVIEW-12: Split api.rs por dominio
lastAction: 5/5 steps completed, contract verified, commit ac128bcb created
result: OK
nextAction: ninguna para REVIEW-12; orquestador continúa Wave W24 (REVIEW-10, GOV-TK4)
contract:
  verificacion: cargo check -p vantadb --all-targets == 0 (verificado 2026-08-30 22:30)
  evidencia:
    - claim: 5 archivos sdk/api/* (memory, graph, admin, search, namespaces)
      evidencia: Get-ChildItem src/sdk/api/*.rs | wc
      confianza: alta
    - claim: sin break publico, firmas identicas
      evidencia: grep "pub fn" en cada modulo coincide con api.rs original
      confianza: alta
    - claim: pre-mortem checks mitigados
      evidencia: tests inline compilan, dominios no solapan, BulkImportReport preservado
      confianza: alta
  artefactos:
    - commit: ac128bcb97aa640274ece102718aee9a71ee0bde
    - archivos: src/sdk/api.rs (675L), src/sdk/api/{admin,graph,memory,namespaces,search}.rs
  invariantes:
    - VantaEmbedded::* firmas identicas
    - BulkImportReport preservado en sdk::api via re-export
  deuda:
    - clippy::assertions_on_constants pre-existente en src/config.rs:1767 (WSM-04, ajeno)
    - src/server/routing.rs:4982 mod auth_tests roto (REVIEW-10/SRV-*, ajeno)
  queda_pendiente:
    - ninguna para REVIEW-12
nextTask: REVIEW-10 (cli_server.rs split) — paralelo en W24, no encadenado
```

## Deuda técnica (Regla 6)

**Saldo:** 0 (refactor aditivo sin nuevos `unsafe`, sin nuevos clones en hot paths)

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | Contrato pasa ✅ (count≥4 + cargo check 0) |
| Commit | atomic, conventional `refactor:`, sin auto-reporte ✅ (ac128bcb) |
| Release | (no release) |

## Steps

### Step 1: Crear `src/sdk/api/memory.rs` con impl block memory ✅
### Step 2: Crear `src/sdk/api/graph.rs` con impl block graph ✅
### Step 3: Crear `src/sdk/api/admin.rs` con impl block admin ✅
### Step 4: Crear `src/sdk/api/search.rs` y `src/sdk/api/namespaces.rs` ✅
### Step 5: api.rs → barrel + verify mecánico + commit ✅

## Pre-mortem verification

| Fallo anticipado | Mitigación | Estado |
|-----------------|------------|--------|
| 1. SDK public surface | Solo `BulkImportReport` público, preservado vía `pub use memory::BulkImportReport` | ✅ |
| 2. Tests usan api.rs | `mod tests` mantenido en `api.rs` con `super::*` | ✅ |
| 3. Dominios solapados | Sin cross-imports entre modulos, cada uno agrupa funciones homogeneas | ✅ |

## Notas

- SECURITY/PERFORMANCE: NO APLICAN (refactor puro)
- Commit hash: `ac128bcb97aa640274ece102718aee9a71ee0bde`
- Mensaje: `refactor: REVIEW-12 — Split api.rs por dominio (memory/search/namespaces/admin/graph)`
- Diff: 6 archivos, scope acotado al barrel + sub-módulos

## Context Save Point

- **Fecha:** 2026-08-30T22:30
- **Branch:** develop
- **CI pendiente:** sí (verify completo pre-push, fuera del scope del task)
- **Decisiones:** sub-módulo `api/` para preservar path; tests consolidados en api.rs
- **Problemas conocidos:** clippy pre-existente y mod server roto (ajenos)
- **Próxima tarea:** REVIEW-10 (cli_server.rs split) — paralelo en W24