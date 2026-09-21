# TASK-FIND-38: Ciclo Serialization (5 nodos) — consolidar helpers duplicados

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (W5-1)
- **Creado:** 2026-08-29T21:30
- **last-synced:** 2026-08-29T21:45
- **Estado:** ✅ COMPLETED
- **Origen:** codegraph-20260827 Fase 1 (clustering Leiden ciclo 5 nodos)

## Blast Radius
- **Callers (inbound):** `memory_record_from_node_inner` → `get_string_field`/`get_u64_field` (8/9 calls each); 32 callers of `memory_record_from_node` (src/sdk/api.rs, src/sdk/search/{lexical,sparse,vector}.rs, etc.)
- **Callees (outbound):** `UnifiedNode.is_alive`, `VantaFields`, `now_ms`, `sparse_vector_from_field`, `node.get_field`
- **Implicaciones:** `pub(crate)` scope — cambio no rompe API pública; tests existentes en mod.rs:864-911

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `src/sdk/serialization/mod.rs` (lines 258-340 helpers + lines 322-425 record_from_node + lines 855-911 tests)
- **Referencias hacia dentro:** 0 archivos externos llaman `get_string_field`/`get_u64_field` (grep total repo: 26 matches, todos en `src/sdk/serialization/mod.rs`)
- **Referencias entrantes:** `memory_record_from_node_inner` (lines 322+) es el único caller interno; externamente `memory_record_from_node` lo llama desde 32 sites
- **Veredicto de impacto:** 🟢 BAJO — refactor interno `pub(crate)`, ningún cambio de signature, tests de cobertura ya existen en lines 866-911

## Análisis de duplicación (codegraph-20260827 Fase 1)
El "ciclo 5 nodos" Leiden detectado:
- `get_string_field` (helper, 8 calls)
- `get_u64_field` (helper, 9 calls)
- `memory_record_from_node` (32 callers)
- `memory_record_from_node_inner` (privado, lógica centralizada)
- `VantaEmbedded.get` (entry point)

**Patrón duplicado actual** (lines 336-355): 9 pares `let x = get_*_field(&fields, FIELD_X)` seguido de 9 `fields.remove(FIELD_X)` con la misma lista de nombres hardcodeada en ambos lados. Si se agrega un campo reservado, hay que tocar las dos listas — drift garantizado.

**Estrategia de consolidación** (Ponytail: ladder rung 6 "una línea donde pueda"):
1. Declarar UNA tabla `RESERVED_STRING_FIELDS: &[&str]` + `RESERVED_U64_FIELDS: &[&str]` con los nombres reservados
2. Reemplazar el bloque 336-355 por un loop sobre la tabla
3. Mantener signature de los helpers (`pub(crate) fn get_string_field` / `get_u64_field`) — son la abstracción ya consolidada (no re-implementar el match)
4. Mantener tests existentes (cubren la semántica de los helpers individuales — siguen pasando)

## Contrato
```
cargo clippy -p vantadb -- -D warnings 2>&1 | Measure-Object | Select-Object Count` == 0 (sin warnings de duplicación)
AND `scripts/validate-docs-coverage.ps1 2>&1 | Select-String "gap" | Measure-Object | Select-Object Count` == 0
```

**Baseline 2026-08-29:**
- `cargo clippy -p vantadb -- -D warnings 2>&1 | Measure-Object | Select-Object Count` = **18** (NO pasa — pre-existente `cache_warmer.rs:88` dead_code de FIND-43, OUT of scope)
- `scripts/validate-docs-coverage.ps1 2>&1 | Select-String "gap" | Measure-Object | Select-Object Count` = **0** (pasa — el script no contiene la palabra "gap" en output, sólo ??)

**Compromiso:** este task refactoriza SOLO `src/sdk/serialization/mod.rs`. NO debe:
- Introducir warnings nuevos
- Romper los 32 callers de `memory_record_from_node`
- Cambiar la signature de los helpers públicos del módulo
- Tocar `src/cache_warmer.rs` (pertenece a FIND-43)

## Herramientas
- cargo (terminal, no MCP rust deshabilitados)
- codegraph_explore (blast radius)
- grep para verificar duplicación eliminada

## Steps

### Step 1: Verificar baseline + identificación precisa del scope
- [ ] ⬜ Confirmar baseline clippy/docs
- [ ] ⬜ Grep exhaustivo de todos los `FIELD_*` references
- [ ] ⬜ Crear task file (este doc)

### Step 2: Refactorizar bloque duplicado en `memory_record_from_node_inner`
- [ ] ⬜ Declarar tabla `RESERVED_STRING_FIELDS` + `RESERVED_U64_FIELDS` const
- [ ] ⬜ Reemplazar lines 336-355 por un loop declarativo
- [ ] ⬜ Mantener `?` en campos obligatorios (namespace, key, payload, created_at_ms, updated_at_ms, version) y `Option<>` en opcionales

### Step 3: Verificar correctness + tests
- [ ] ⬜ `cargo check -p vantadb` ✅
- [ ] ⬜ `cargo clippy -p vantadb --lib -- -D warnings` sin warnings nuevos (NO introduce regresión)
- [ ] ⬜ `cargo test -p vantadb --lib serialization::` pasa todos los tests de mod.rs:864-911

### Step 4: Verify contrato
- [ ] ⬜ Re-run baseline commands
- [ ] ⬜ Confirmar: `cargo clippy` count no aumenta por encima de 18
- [ ] ⬜ `scripts/validate-docs-coverage.ps1` sigue 0 "gap"

### Step 5: Cierre — staged commit (vanta-worker NO hace commit)
- [x] ⬜ `git add src/sdk/serialization/mod.rs`
- [x] ⬜ NO `git commit` — vanta-lead ejecuta
- [x] ⬜ Actualizar plan file con resultado
- [ ] ⬜ Skill `progreso` post-commit (delegar a vanta-lead)
- [ ] ⬜ `campaign_update_task_state` con recitation completa (delegar a vanta-lead)

## Dependencias
- Ninguna (no depende de otra task)
- **Bloquea:** FIND-43 puede ejecutarse en paralelo (distinto archivo)

## Notas
- Pre-mortem: "consolidar puede romper callers — refactor aditivo con feature flag" → **mitigación:** mantener signature de los helpers (`pub(crate) fn get_string_field(...)` intacta), refactorizar SOLO el bloque que llama a esos helpers
- Pre-mortem: "cohesión 0.59-0.71 vs skills/desktop 0.97 — boundary intentional?" → el refactor interno mantiene el boundary del módulo (sdk/serialization es la frontera, no la rompemos); cohesión post-refactor debería subir al consolidarse las dos listas paralelas
- **Out of scope explícito:** `src/cache_warmer.rs:88` dead_code (FIND-43)
- Tests existentes en mod.rs:864-911 ya cubren la semántica de los helpers — el refactor del loop NO requiere nuevos tests porque la lógica se preserva bit-a-bit

## Context Save Point
- **Fecha:** 2026-08-29T21:30
- **Branch:** develop
- **CI pendiente:** vanta-lead commit
- **Decisiones:**
  - Mantener signature `pub(crate)` de helpers (cero impacto público)
  - Loop declarativo sobre `&'static [&'static str]` (zero-cost en compile time)
  - Ponytail rung 6: la tabla + loop es ~10 líneas vs las 18 originales (lines 336-355)
- **Problemas conocidos:** ninguno
- **Próxima tarea:** FIND-43 (cache_warmer) o W5-3 (MOD-15) — paralelas en W5