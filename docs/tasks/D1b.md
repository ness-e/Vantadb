# D1b — Partir `vector_search` (~133) y `run_pipeline` (~132) en helpers SLAP ≤20

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- **Tipo:** refactor-split lógica/frontera (vanta-worker). Norma: clean-code-clean-architecture §2.2/F1 (SLAP ≤20/función convención operativa) + Regla 9 (bench before/after hot path).
- **Scope exacto:** `src/engine.rs:338` (`vector_search`) + `src/storage/engine/maintenance.rs:1086` (`run_pipeline`). PROHIBIDO tocar `get.rs`/`insert.rs` (D1a paralelo) ni `maintenance:267 lock_held` (D2).
- **Blast radius:** callers de `vector_search` (SDK/API/benches canonical_p99 HNSW en memoria); callees internos (distancia, filtros, top-k). `run_pipeline`: callers scheduler/mantenimiento; callees fases (compact, gc, checkpoints, métricas). Verificar con `codegraph_explore`.
- **Baseline:** conteo funciones >20 líneas antes/después en ambos archivos + `cargo nextest` módulos + `clippy -D warnings` + `fmt`. Bench `canonical_p99` antes/después si el entorno lo permite (si tantivy rompe build release como en H2, documentar e igualar con tests).
- **Gate D:** blast radius <10 archivos, sin símbolos públicos nuevos → GO sin `question`.

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- **Cambia:** partir `vector_search` (~133 lín) y `run_pipeline` (~132 lín) en helpers de un solo nivel (validate→fetch→transform→persist→metrics / fases pipeline), cada helper ≤20 líneas, nombres intention-revealing (§2.1).
- **NO cambia:** semántica, orden de efectos, firmas públicas, wire/API; PROHIBIDO optimizar (solo partir); PROHIBIDO `get.rs`/`insert.rs` y `maintenance:267 lock_held`.
- **Archivos exactos:** `src/engine.rs`, `src/storage/engine/maintenance.rs` (+ tests vecinos existentes, + 1 test por helper con lógica).
- **Verify contrato (bash, sin campaign MCP):**
  - `cargo fmt --check`
  - `cargo clippy -p vantadb --all-targets -- -D warnings` (o `--workspace` si aplica)
  - `cargo nextest run --profile audit -p vantadb <filtro engine|maintenance|storage::engine>`
  - conteo >20 antes/después: `pwsh -c` o `rg` + script conteo
  - `cargo bench --bench canonical_p99` antes/después si compila (si no, documentar causa + igualar con tests)
  - cierre `/cleanCA <scope>` F1 mejora medible

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
### Step 1: Baseline + conteo + bench-before
- **Archivos:** lectura `src/engine.rs:338`, `src/storage/engine/maintenance.rs:1086`, bench `benches/canonical_p99.rs`
- **Acción:** medir funciones >20, correr nextest módulos en verde, intentar bench-before (o documentar bloqueo tantivy/H2)
- **Verify:** outputs guardados para comparar
- **Estado:** ☐ PENDING

### Step 2: Slice vector_search → helpers SLAP ≤20
- **Archivos:** `src/engine.rs`
- **Acción:** extraer helpers un nivel (ej: resolve_params → gather_candidates → score_rank → truncate/map), ≤20 lín c/u, sin cambio semántico
- **Verify:** `cargo nextest run --profile audit -p vantadb engine` + clippy + fmt
- **Estado:** ☐ PENDING

### Step 3: Slice run_pipeline → helpers SLAP ≤20
- **Archivos:** `src/storage/engine/maintenance.rs`
- **Acción:** extraer helpers por fase (ej: plan_steps → run_phase → record_metrics/handle_err), ≤20 lín c/u, sin cambio semántico
- **Verify:** `cargo nextest run --profile audit -p vantadb maintenance` + clippy + fmt
- **Estado:** ☐ PENDING

### Step 4: Tests (existentes verdes + 1 por helper con lógica) + bench-after + conteo
- **Archivos:** tests vecinos + nuevos tests helpers
- **Acción:** agregar 1 test por helper con lógica (TDD: RED→GREEN si lógica nueva de partición lo requiere; si pura extracción, test de equivalencia/contrato), correr bench-after neutralidad Regla 9
- **Verify:** nextest + clippy + fmt + conteo >20 después + bench delta documentado
- **Estado:** ☐ PENDING

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO siempre (✅/🟡/❌, STEPS_OK, PROXIMO_STEP, COMMIT_HASH ninguno — no commitear, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS ≥10). Nunca vacío.
- `/cleanCA src/engine.rs src/storage/engine/maintenance.rs` F1 mejora medible (conteo >20 antes/después).
- Recitation: objetivo D1b, estado, última acción, resultado, próxima acción, contrato+resultado, invariantes (semántica intacta, D1a/D2 no tocados), comandos verify, deuda, próxima tarea.
