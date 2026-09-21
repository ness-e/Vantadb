# FIND-91: Fallout F3X — `tests/durability_recovery.rs:433` sobre `dyn IndexPort` (E0609)

## Metadata
- **Plan file:** (sin plan — creado desde validación proyecto 2026-09-15)
- **Fuente:** docs/Backlog.md fila FIND-91 · validación proyecto 2026-09-15 · causa `13f0f729` (F3X trait-split)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (test de durabilidad + trait `IndexPort`)
- **Turns estimados:** 3-5
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps (2/2 ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `tests/durability_recovery.rs` (Phase 2: reapertura tras snapshot fallido, gate `failpoints`) |
| Callees | `IndexPort::contains_node()` (`src/index_port.rs:158`), `IndexPort::node_count()` (línea siguiente, ya usa el trait) |
| Implicaciones | Cambio de una línea en un assert de test; semántica idéntica (`nodes.get(&id).is_some()` ≡ `contains_node(id)`). Sin cambio de comportamiento del engine. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `tests/durability_recovery.rs:383-442` (gate `failpoints`, fases 1-2), `src/index_port.rs:157-160` (`contains_node`, `node_count`)
- **Archivos referenciados hacia dentro:** `vantadb::node::*`, `fail::cfg` (failpoints), `StorageEngine::{insert, flush, get}`, `engine.hnsw.load()`
- **Archivos que referencian a los editados:** job heavy `storage-persistence` (`docs/workflow/heavy-certification-50.md`), perfil nextest `chaos`, `docs/operations/DURABILITY_GUARANTEES.md`, `TEST_MAP.md`
- **Veredicto impacto:** BAJO — un assert en un test; el resto del test (incluido `node_count()` en `:437`) ya compila contra el trait.

## Contrato

```
cargo check -p vantadb --test durability_recovery --all-features pasa (hoy falla E0609 en :433)
cargo clippy --workspace --all-targets --all-features --exclude vantadb-wasm --exclude vantadb-server --exclude vantadb-mcp -- -D warnings pasa (comando CI Windows; hoy falla en este test)
cargo nextest run --profile chaos --features failpoints -p vantadb --test durability_recovery pasa (test pesado ~218s históricos — ejecutar solo si el tiempo lo permite, si no dejar a Heavy)
```

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. El assert sigue verificando presencia del nodo en el índice tras replay (no debilitar a `node_count` global: el loop por-`i` debe seguir).
  2. No tocar la lógica de failpoints (`snapshot_serialize_fail` en `:414-416`) ni las fases del test.
- **Comandos de verificación:** los del Contrato.
- **Deuda pendiente:** ninguna.

## Recitation

```
=== RECITATION ===
Objetivo activo: FIND-91 — durability_recovery:433 sobre dyn IndexPort
Estado: COMPLETED
Última acción: Step1 edit :433 + Step2 verify completo + review vanta-review approve + commit fix:
Resultado: ✅
State: COMPLETED (desde: in-progress)
Próxima acción: ninguna (tarea cerrada)
Contrato: check --all-features ✅ (13.40s) + clippy CI Windows ✅ (1m22s) + fmt ✅ + cargo test failpoints 8/8 ✅ (30.71s); nextest --profile chaos corre 0 tests por diseño (default-filter chaos solo chaos_integrity_failpoints) — documentado, no requiere Heavy
Invariantes: assert por-nodo intacto; failpoints intactos (verificados por reviewer)
Comandos de verificación: ver Contrato (outputs reales arriba)
Deuda: ninguna
Próxima tarea si completa: FIND-64 (Wave0; FIND-90 independiente en paralelo)
last-synced: 2026-09-15
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — el fix elimina deuda (test roto por F3X).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable se cumple (al menos check + clippy; nextest chaos si tiempo) |
| **Commit** | Commit atómico, conventional commit (`fix:`), `git diff` limpio, verificación mecánica |
| **Release** | N/A (fix de test, sin cambio de API) |

## Herramientas necesarias

- cargo (check, clippy, nextest perfil chaos con `failpoints`)

**Skills cargadas (SDP):** `campaign_discover_skills_v2 archivosClave="tests/durability_recovery.rs src/index_port.rs" phase="BUILD" contractKeywords=[durability, IndexPort, contains_node, failpoints]` → 8 (campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, frontend-ui-engineering[no aplica], api-and-interface-design) + bug-fix: systematic-debugging + pre-commit: code-review-and-quality. Cargadas efectivas: systematic-debugging, test-driven-development (mínimo), code-review-and-quality. SDP: base+lifecycle, sin keyword-mapped (fix 1 línea a getter existente).
- **SKILLS_CARGADAS base:** campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full).

## Investigation Notes

- Causa raíz: F3X (`13f0f729`) cambió `StorageEngine.hnsw` a `ArcSwap<Box<dyn IndexPort>>`; el trait expone `contains_node(id)` (`index_port.rs:158`) pero no `nodes`. La línea siguiente (`:437` `hnsw.node_count()`) ya usa el trait — el fix es simétrico.
- El test vive bajo gate `failpoints` parcial (`:383`); el fallo E0609 aparece con `--all-features` (incluye `failpoints`). El perfil `audit` lo excluye por `default-filter`, por eso el Fast Gate audit pasó en verde y el rojo solo sale en clippy `--all-targets --all-features` y perfil `chaos`.
- Histórico: el test pasaba 7/7 antes de F3X (verificado en planes previos); es regresión de compilación, no flake.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 2 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica (assert de presencia en test de durabilidad; justificado).
- [x] **PERFORMANCE** — no aplica (test, no hot path; justificado).

## Steps

### Step 1: `durability_recovery.rs:433` — `nodes.get` → `contains_node`
- **Archivos:** `tests/durability_recovery.rs:427-435`
- **Acción:** reemplazar `hnsw.nodes.get(&(i as u128)).is_some()` por `hnsw.contains_node(i as u128)`, preservando el mensaje de assert.
- **Verify:** `cargo check -p vantadb --test durability_recovery --all-features`
- **Estado:** ✅ DONE (check ✅ 13.40s; Gate V semántica: NO dispara — `port_impl.rs:273-275` `contains_key` ≡ `get().is_some()`)

### Step 2: Verify completo
- **Archivos:** ninguno (verificación)
- **Acción:** clippy con el comando CI Windows + nextest perfil chaos del test (si el tiempo lo permite; documentar si se deja a Heavy).
- **Verify:** comandos del Contrato en verde.
- **Estado:** ✅ DONE — clippy CI Windows ✅ (1m22s, 0 warnings); `cargo fmt --check` ✅; `cargo test -p vantadb --features failpoints --test durability_recovery` 8/8 ✅ en 30.71s (incluye `test_checkpoint_not_advanced_on_snapshot_failure`). Nota: `cargo nextest run --profile chaos --features failpoints -p vantadb --test durability_recovery` corre 0 tests POR DISEÑO (default-filter del perfil chaos = solo `test(chaos_integrity_failpoints)`, `.config/nextest.toml:100-101`) — no es fallo; la ejecución real del binario vía libtest cubre el contrato sin necesidad de Heavy (30.71s ≪ 218s históricos).

## Dependencias
- Ninguna (independiente de FIND-90; pueden ejecutarse en paralelo).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review (sub-agente distinto al implementador, vía task tool — P2-01 ✅)
- **Enfoque:** ¿`contains_node` es semánticamente idéntico a `nodes.get(..).is_some()`? SÍ — `port_impl.rs:273-275` `contains_key`; `index_port.rs:157-158` probe canónica; alternativa `storage_offset_of().is_some()` descartada (acopla presencia a offset).
- **Cómo se probó:** check --all-features ✅ + clippy CI Windows ✅ + fmt ✅ + `cargo test --features failpoints --test durability_recovery` 8/8 ✅ (outputs reales en task, no auto-reporte).
- **Checklist anti-hábitos tóxicos:** 7/7 ✅ (assert no debilitado, semántica idéntica, solo test, sin allows/unwraps, failpoint off, sin abstracciones, sin import extra).
- **Veredicto:** ✅ approve

## Notas
- Lección de proceso (de la validación): el perfil `audit` excluye este binario por `default-filter`; el gate que lo cubre es clippy `--all-features` + perfil `chaos` + Heavy. No es deuda nueva del test.
