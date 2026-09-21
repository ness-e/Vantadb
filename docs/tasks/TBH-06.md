# TBH-06: Add insta 1.48 snapshot testing (3 parser + 2 query result tests)

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md` (§"Phase 5: Snapshots & Differential", `TASK-06`)
- **Creado:** 2026-08-31
- **last-synced:** 2026-08-31
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Sub-agente:** vanta-worker

## Verificación previa (Regla 0 — Impacto mapeado)

### Archivos candidatos inspeccionados
| Path | Existe | Estado |
|---|---|---|
| `tests/logic/parser.rs` | ✅ Sí (90 líneas, 2 tests: `dql_parser_certification`, `dml_*` dentro de `harness.execute`) | OK para migrar — usa `parse_query`/`parse_statement` y devuelve AST con `Debug` deriveado |
| `tests/<query_result*>.rs` | ❌ NO existen | Spec documentada, marcada 🟡 INCOMPLETO — sin inventar paths |
| `Cargo.toml` workspace root | ✅ Sí (679 líneas) | Sin `[workspace.dependencies]` — dev-deps van directo en `[dev-dependencies]` (líneas 170-189). Formato a respetar: `tokio = { version = "1", features = [...] }` |

### BLAST RADIUS — qué se ve afectado al añadir `insta`
| Cambio | Impacto | Riesgo |
|---|---|---|
| `Cargo.toml` línea 170-189 (`[dev-dependencies]` añade `insta = "1.48"`) | Solo build de tests — no toca runtime | Bajo |
| Nuevo `use insta;` en `tests/logic/parser.rs` | Solo el archivo | Bajo |
| 3 `.snap` files nuevos junto a `tests/logic/parser.rs` | Build artifacts para `insta::assert_snapshot!` | Bajo |
| `Cargo.lock` (auto) | Una entrada nueva | Bajo |

### Decisiones
- **NO `[workspace.dependencies]`**: el workspace NO usa esa convención (todos los dev-deps van directo). El contrato del task dice "chequeá el formato existente" → respetar formato actual.
- **NO agrego `Display` al AST**: `Debug` ya está deriveado en `Statement`, `ParsedQuery`, `Condition`, `RelOp`, etc. — `insta::assert_snapshot!` funciona con cualquier `fmt::Debug`.
- **NO migro tests existentes** (`dql_parser_certification`, `dml_*`): las assertions específicas son útiles para regresiones precisas. Los snapshots **agregan** cobertura sin reemplazar (Ponytail reflex: borraría valor si reemplazo). En su lugar, **agrego 3 nuevos tests** al final del archivo que snapshot-an el `Debug` output del AST.
- **NO invento paths** para los tests de query results (la auditoría mencionó `tests/.../query_result*.rs` pero no existen).

### Hallazgo crítico
- El parser AST tiene `#[derive(Debug)]` → snapshots funcionan out-of-the-box.
- `parse_query` devuelve `ParsedQuery` con campos `from_entity`, `traversal`, `where_clause`, `temperature` — todos `Debug`.
- `parse_statement` devuelve `Statement` (enum con `Insert`, `Update`, `Delete`, `Relate`, `Select`, `Query`, `InsertMessage`) — todos `Debug`.

## Steps

### Step 1: Edit `Cargo.toml` ✅
- Insertada línea `insta = "1.48"` al final del bloque `[dev-dependencies]` (línea 189).
- Formato consistente con dev-deps vecinas (`serde_json = "1.0"`, `serial_test = "3.2"`).

### Step 2: Migrar 3 tests del parser ✅
- Añadidos 3 nuevos `#[test]` al final de `tests/logic/parser.rs` que usan `insta::assert_snapshot!`:
  - `dql_query_ast_snapshot` — snapshot del `Debug` output de `parse_query` sobre la query multi-cláusula
  - `dml_insert_ast_snapshot` — snapshot del `Debug` output de `parse_statement` sobre INSERT con vector posicional
  - `dml_relate_ast_snapshot` — snapshot del `Debug` output de `parse_statement` sobre RELATE con WEIGHT
- Cada test genera un `.snap` file en `tests/logic/snapshots/` al primer run con `INSTA_UPDATE=auto` (o el workflow lo acepta con `cargo insta review`).

### Step 3: Crear query_result tests ✅
- Creados 2 nuevos archivos de test con `insta::assert_debug_snapshot!`:
  - `tests/query_result_basic.rs` — 7 tests: search requests (basic, vector-only, text-only), search hits (basic, simple), list pages (empty, with records)
  - `tests/query_result_advanced.rs` — 13 tests: search profiles (hybrid, keyword, vector), exclude_superseded, sparse vectors, full complex request, hits with explanation, superseded chain, multi-page list, last page, VantaQueryResult variants (Read, Write, StaleContext)
- Añadidas entradas `[[test]]` en `Cargo.toml` para `query_result_basic` y `query_result_advanced`
- Snapshots generados y aceptados con `cargo insta accept` (20 archivos `.snap` en `tests/snapshots/`)

## Acceptance criteria — Verificación

| Check | Comando | Esperado | Obtenido |
|---|---|---|---|
| `insta` añadido | `Select-String -Path Cargo.toml -Pattern "^insta"` en `[dev-dependencies]` | ≥1 match | ✅ línea 190 |
| `cargo check` verde | `cargo check -p vantadb --tests` | exit 0 | ✅ |
| 3 snapshot tests en parser | `git grep "insta::assert_snapshot" tests/logic/parser.rs` | ≥3 | ✅ 3 |
| `.snap` files (parser) | `git status --short tests/logic/snapshots/` | ≥3 nuevos | ✅ 3 nuevos tracked |
| 2 query_result test files | `ls tests/query_result_*.rs` | 2 archivos | ✅ 2 |
| 7 snapshot tests basic | `cargo test -p vantadb --test query_result_basic` | 7 passed | ✅ 7 passed |
| 13 snapshot tests advanced | `cargo test -p vantadb --test query_result_advanced` | 13 passed | ✅ 13 passed |
| `.snap` files (query_result) | `ls tests/snapshots/query_result_*.snap` | 20 archivos | ✅ 20 |
| `cargo fmt --check` | sobre el repo | OK | ✅ |
| `cargo clippy -p vantadb --tests` | sin warnings nuevos | OK | ✅ (solo warning pre-existente en config.rs:1767) |

## Próximos pasos (para cerrar 🟡 INCOMPLETO)
- ✅ **Completado**: Creados `tests/query_result_basic.rs` y `tests/query_result_advanced.rs` con 20 snapshot tests totales.
- ✅ **Completado**: Añadidas entradas `[[test]]` en `Cargo.toml`.
- ✅ **Completado**: Snapshots generados y aceptados con `cargo insta accept`.
- ✅ **Task cerrado** con `campaign_update_task_state("completed")`.

## Notas
- **Por qué no usar `INSTA_UPDATE=no` en CI**: el primer run necesita generar los `.snap`. El flag canónico es `INSTA_UPDATE=new` (default en `cargo insta test`) o el workflow corre `cargo insta review --workspace`.
- **Ponytail reflex**: NO reemplacé assertions existentes — snapshots complementan, no substituyen. Las assertions específicas son más rápidas para regresiones puntuales; los snapshots detectan cambios estructurales no anticipados.
- **NO `cargo check --workspace`**: workspace tiene bug pre-existente `FIND-MCP-001` (`vantadb-mcp/tests/context_tests.rs:70`). Todos los `cargo check` aquí usan `-p vantadb`.
- **NO `cargo nextest`**: los tests snapshot usan `insta` que requiere `cargo test` o `cargo insta test` (cargo-nextest tiene soporte pero requiere setup adicional). El alcance de esta tarea es añadir la dep + 3 tests — la integración con nextest es follow-up.

## Context Save Point
- **Fecha:** 2026-09-01
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:**
  - 3 tests nuevos en parser (NO reemplazar existentes)
  - 7 tests nuevos en query_result_basic (search requests, hits, list pages)
  - 13 tests nuevos en query_result_advanced (profiles, exclude_superseded, sparse, complex, VantaQueryResult variants)
  - `insta = "1.48"` en `[dev-dependencies]` directo (NO `[workspace.dependencies]` — no existe)
  - Snapshots aceptados con `cargo insta accept` (20 archivos `.snap` en `tests/snapshots/`)
  - Tests corren con `cargo test -p vantadb --test query_result_basic --test query_result_advanced`
- **Próxima tarea:** TBH-XX (sigiente en plan)