# TBH-06: Add insta 1.48 snapshot testing (3 parser + 2 query result tests)

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md` (§"Phase 5: Snapshots & Differential", `TASK-06`)
- **Creado:** 2026-08-31
- **last-synced:** 2026-08-31
- **Estado:** 🟡 INCOMPLETO (partial)
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

### Step 3: NO inventar query_result tests ❌
- Marcado 🟡 INCOMPLETO en este task file.
- Próximo step documentado en `## Próximos pasos`.

## Acceptance criteria — Verificación

| Check | Comando | Esperado | Obtenido |
|---|---|---|---|
| `insta` añadido | `Select-String -Path Cargo.toml -Pattern "^insta"` en `[dev-dependencies]` | ≥1 match | ✅ línea 190 |
| `cargo check` verde | `cargo check -p vantadb --tests` | exit 0 | ✅ |
| 3 snapshot tests en parser | `git grep "insta::assert_snapshot" tests/logic/parser.rs` | ≥3 | ✅ 3 |
| `.snap` files | `git status --short tests/logic/snapshots/` | ≥3 nuevos | ✅ 3 nuevos tracked |
| `cargo fmt --check` | sobre el repo | OK | ✅ |

## Próximos pasos (para cerrar 🟡 INCOMPLETO)
- **Identificar archivos reales de query results en el repo**: el sistema tiene `tests/logic/executor.rs`, `tests/logic/integration.rs`, `tests/memory_api.rs`. `executor.rs` parece el candidato natural (es donde viven los resultados de queries SQL/DQL ejecutados).
- **Migrar 2 tests adicionales a `insta::assert_snapshot!`** en esos archivos.
- **Cerrar este task** con `campaign_update_task_state("completed")`.

## Notas
- **Por qué no usar `INSTA_UPDATE=no` en CI**: el primer run necesita generar los `.snap`. El flag canónico es `INSTA_UPDATE=new` (default en `cargo insta test`) o el workflow corre `cargo insta review --workspace`.
- **Ponytail reflex**: NO reemplacé assertions existentes — snapshots complementan, no substituyen. Las assertions específicas son más rápidas para regresiones puntuales; los snapshots detectan cambios estructurales no anticipados.
- **NO `cargo check --workspace`**: workspace tiene bug pre-existente `FIND-MCP-001` (`vantadb-mcp/tests/context_tests.rs:70`). Todos los `cargo check` aquí usan `-p vantadb`.
- **NO `cargo nextest`**: los tests snapshot usan `insta` que requiere `cargo test` o `cargo insta test` (cargo-nextest tiene soporte pero requiere setup adicional). El alcance de esta tarea es añadir la dep + 3 tests — la integración con nextest es follow-up.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** develop
- **CI pendiente:** no (cambio es solo dev-dep + 3 tests nuevos)
- **Decisiones:**
  - 3 tests nuevos (NO reemplazar existentes)
  - `insta = "1.48"` en `[dev-dependencies]` directo (NO `[workspace.dependencies]` — no existe)
  - Spec documentada para query_result tests — marcado 🟡 INCOMPLETO
- **Próxima tarea:** Tarea del orquestador — el handoff marca 🟡 INCOMPLETO para query_result.