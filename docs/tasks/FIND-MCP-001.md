# FIND-MCP-001: Fix `MemoryRecord { ... }` literal faltan `heat`/`superseded_by` en `vantadb-mcp/tests/context_tests.rs:70`

## Metadata
- **Plan file:** docs/plans/2026-08-31-fast-gate-residues.md
- **Creado:** 2026-08-31
- **last-synced:** 2026-08-31
- **Estado:** ✅ COMPLETED
- **Tipo (campaign_detect_task_type):** fix / test compile (completar struct literal)
- **Esfuerzo:** 🟢 ~5 min
- **Prioridad:** 🔴 Alta (bloquea `cargo check --workspace --tests`; pre-existente)
- **Sub-agent:** vanta-worker (delegated, pero fix ya aplicado en commit anterior)
- **Task previa:** AUD-043 ✅

## Impacto mapeado (Regla 0)

### Archivos leídos completos
| Archivo | Líneas | Notas |
|---------|--------|-------|
| `vantadb-mcp/tests/context_tests.rs` | ~150 | Test `seed_l1` en línea 86 crea `MemoryRecord { ... }` — **ya tiene `heat: 0, superseded_by: None`** (commit `43e0779e`, 2026-08-31). |
| `vantadb-python/src/types.rs:48` | — | `VantaPyMemoryRecord` tiene getters `heat` y `superseded_by:125`. El struct base `VantaMemoryRecord` (Python bindings) requiere estos campos. |

### Referencias hacia adentro (outbound references)
- `context_tests.rs:86-90` → usa `MemoryRecord` con todos los campos requeridos.

### Referencias hacia afuera (inbound)
- `vantadb-mcp/tests/context_tests.rs` → único consumidor de este literal en tests.

### Veredicto de impacto
**Cero cambios necesarios.** El fix ya fue aplicado en commit `43e0779e` (chore(cleanup): post-P48 residues). La línea 70 original del plan ahora es línea 86-90 con los campos agregados:
```rust
let options = VantaMemoryInput {
    ...
    heat: 0,
    superseded_by: None,
};
```

## Contrato (verificable mecánicamente)

```
1. `cargo check -p vantadb-mcp --test context_tests` — ✅ PASS (0.41s)
2. `cargo check -p vantadb-mcp --tests` — ❌ FAIL (otros errores pre-existentes en test_embed_texts.rs:78 `max_embed_batch_size` no existe en McpConfig — NO es FIND-MCP-001)
```

**Lectura correcta:** FIND-MCP-001 target específico (MemoryRecord literal en context_tests.rs) está ✅ RESUELTO. El contrato general `--tests` falla por issues separados en `test_embed_texts.rs` (scope nuevo → FIND-036 propuesto).

## Hallazgos colaterales (registrados como FIND-036)

`cargo check -p vantadb-mcp --tests` falla con 3 errores en `test_embed_texts.rs:78`:
- `max_embed_batch_size` no existe en `McpConfig` (campos disponibles: `max_concurrency`, `max_payload_length`, `max_key_length`, `max_namespace_length`, `max_vector_dim`, etc.)

**Acción:** agregar fila `FIND-036` a `docs/Backlog.md` para fix del struct `McpConfig` / test_embed_texts.rs. NO scope-creep a FIND-MCP-001.

## Steps

### Step 1: Verificar estado actual del literal
- **Archivos:** `vantadb-mcp/tests/context_tests.rs:86-90`
- **Acción:** Read tool → confirmar `heat: 0, superseded_by: None` presentes.
- **Verify:** `grep -n "heat\|superseded_by" vantadb-mcp/tests/context_tests.rs` → muestra líneas 89-90.
- **Estado:** ✅ COMPLETED (ya aplicado en commit 43e0779e).

### Step 2: Verificar compile del test específico
- **Archivos:** n/a (verify only)
- **Acción:** `cargo check -p vantadb-mcp --test context_tests` → exit 0.
- **Verify:** output "Finished `dev` profile [optimized] target(s) in 0.41s".
- **Estado:** ✅ COMPLETED.

### Step 3: Reportar hallazgo colateral FIND-036
- **Archivos:** `docs/Backlog.md`
- **Acción:** agregar fila FIND-036 para `test_embed_texts.rs` `max_embed_batch_size`.
- **Verify:** fila agregada con contract, prioridad 🟢.
- **Estado:** ✅ COMPLETED.

### Step 4: No commit (fix ya en commit anterior)
- **Archivos:** n/a
- **Acción:** FIND-MCP-001 no requiere edit — ya en `43e0779e`.
- **Verify:** `git status --short` no muestra cambios en vantadb-mcp/tests/.
- **Estado:** ✅ COMPLETED.

## Dependencias
- Ninguna. Independiente de otras tareas.

## Notas
- **Arqueológica:** igual que AUD-043 — el plan citaba línea 70 pero el código ya tenía el fix aplicado en commit anterior (`43e0779e`).
- **Scope acotado:** FIND-MCP-001 = SOLO el literal MemoryRecord en context_tests.rs. Otros errores en test_embed_texts.rs = FIND-036 (nueva fila).
- **Lección:** planes arqueológicos requieren verificar `cargo check` del target específico, no del workspace completo.

## Context Save Point
- **Fecha:** 2026-08-31
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:**
  - FIND-MCP-001 = arqueológica completada.
  - Errores en test_embed_texts.rs = FIND-036 (separado).
- **Problemas conocidos:** `cargo check -p vantadb-mcp --tests` sigue rojo por FIND-036.
- **Próxima tarea:** TBH-06 (insta snapshots completion) o FIND-036 si se prioriza.