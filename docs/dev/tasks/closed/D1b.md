--
status: completed
title: "D1b â€” Partir `vector_search` (~133) y `run_pipeline` (~132) en helpers SLAP â‰¤20"
kind: task
---

# D1b â€” Partir `vector_search` (~133) y `run_pipeline` (~132) en helpers SLAP â‰¤20

## 1. Descubrimiento (auto-detect tipo â†’ codegraph blast radius â†’ web si ambigÃ¼edad â†’ baseline `/cleanCA <scope>`)
- **Tipo:** refactor-split lÃ³gica/frontera (vanta-worker). Norma: clean-code-clean-architecture Â§2.2/F1 (SLAP â‰¤20/funciÃ³n convenciÃ³n operativa) + Regla 9 (bench before/after hot path).
- **Scope exacto:** `src/engine.rs:338` (`vector_search`) + `src/storage/engine/maintenance.rs:1086` (`run_pipeline`). PROHIBIDO tocar `get.rs`/`insert.rs` (D1a paralelo) ni `maintenance:267 lock_held` (D2).
- **Blast radius:** callers de `vector_search` (SDK/API/benches canonical_p99 HNSW en memoria); callees internos (distancia, filtros, top-k). `run_pipeline`: callers scheduler/mantenimiento; callees fases (compact, gc, checkpoints, mÃ©tricas). Verificar con `codegraph_explore`.
- **Baseline:** conteo funciones >20 lÃ­neas antes/despuÃ©s en ambos archivos + `cargo nextest` mÃ³dulos + `clippy -D warnings` + `fmt`. Bench `canonical_p99` antes/despuÃ©s si el entorno lo permite (si tantivy rompe build release como en H2, documentar e igualar con tests).
- **Gate D:** blast radius <10 archivos, sin sÃ­mbolos pÃºblicos nuevos â†’ GO sin `question`.

## 2. Contrato (quÃ© cambia / quÃ© NO cambia / archivos exactos / comandos de verify)
- **Cambia:** partir `vector_search` (~133 lÃ­n) y `run_pipeline` (~132 lÃ­n) en helpers de un solo nivel (validateâ†’fetchâ†’transformâ†’persistâ†’metrics / fases pipeline), cada helper â‰¤20 lÃ­neas, nombres intention-revealing (Â§2.1).
- **NO cambia:** semÃ¡ntica, orden de efectos, firmas pÃºblicas, wire/API; PROHIBIDO optimizar (solo partir); PROHIBIDO `get.rs`/`insert.rs` y `maintenance:267 lock_held`.
- **Archivos exactos:** `src/engine.rs`, `src/storage/engine/maintenance.rs` (+ tests vecinos existentes, + 1 test por helper con lÃ³gica).
- **Verify contrato (bash, sin campaign MCP):**
  - `cargo fmt --check`
  - `cargo clippy -p vantadb --all-targets -- -D warnings` (o `--workspace` si aplica)
  - `cargo nextest run --profile audit -p vantadb <filtro engine|maintenance|storage::engine>`
  - conteo >20 antes/despuÃ©s: `pwsh -c` o `rg` + script conteo
  - `cargo bench --bench canonical_p99` antes/despuÃ©s si compila (si no, documentar causa + igualar con tests)
  - cierre `/cleanCA <scope>` F1 mejora medible

## 3. Steps atÃ³micos (â˜ uno por slice: implementar â†’ test â†’ verificar â†’ commit; si falla: `git reset --hard HEAD` del slice)
### Step 1: Baseline + conteo + bench-before
- **Archivos:** lectura `src/engine.rs:338`, `src/storage/engine/maintenance.rs:1086`, bench `benches/canonical_p99.rs`
- **AcciÃ³n:** medir funciones >20, correr nextest mÃ³dulos en verde, intentar bench-before (o documentar bloqueo tantivy/H2)
- **Verify:** outputs guardados para comparar
- **Estado:** â˜ PENDING

### Step 2: Slice vector_search â†’ helpers SLAP â‰¤20
- **Archivos:** `src/engine.rs`
- **AcciÃ³n:** extraer helpers un nivel (ej: resolve_params â†’ gather_candidates â†’ score_rank â†’ truncate/map), â‰¤20 lÃ­n c/u, sin cambio semÃ¡ntico
- **Verify:** `cargo nextest run --profile audit -p vantadb engine` + clippy + fmt
- **Estado:** â˜ PENDING

### Step 3: Slice run_pipeline â†’ helpers SLAP â‰¤20
- **Archivos:** `src/storage/engine/maintenance.rs`
- **AcciÃ³n:** extraer helpers por fase (ej: plan_steps â†’ run_phase â†’ record_metrics/handle_err), â‰¤20 lÃ­n c/u, sin cambio semÃ¡ntico
- **Verify:** `cargo nextest run --profile audit -p vantadb maintenance` + clippy + fmt
- **Estado:** â˜ PENDING

### Step 4: Tests (existentes verdes + 1 por helper con lÃ³gica) + bench-after + conteo
- **Archivos:** tests vecinos + nuevos tests helpers
- **AcciÃ³n:** agregar 1 test por helper con lÃ³gica (TDD: REDâ†’GREEN si lÃ³gica nueva de particiÃ³n lo requiere; si pura extracciÃ³n, test de equivalencia/contrato), correr bench-after neutralidad Regla 9
- **Verify:** nextest + clippy + fmt + conteo >20 despuÃ©s + bench delta documentado
- **Estado:** â˜ PENDING

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO siempre (âœ…/ðŸŸ¡/âŒ, STEPS_OK, PROXIMO_STEP, COMMIT_HASH ninguno â€” no commitear, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS â‰¥10). Nunca vacÃ­o.
- `/cleanCA src/engine.rs src/storage/engine/maintenance.rs` F1 mejora medible (conteo >20 antes/despuÃ©s).
- Recitation: objetivo D1b, estado, Ãºltima acciÃ³n, resultado, prÃ³xima acciÃ³n, contrato+resultado, invariantes (semÃ¡ntica intacta, D1a/D2 no tocados), comandos verify, deuda, prÃ³xima tarea.
