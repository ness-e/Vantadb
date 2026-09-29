---
title: "VER-07: Dreams — dry-run + diff report + `promote_dream_run` real"
kind: task
description: "dryrun:true (CLI/MCP) devuelve diff por registro {action: ADD|UPDATE|DELETE|NOOP, key, razón (merge|dedup|supersede|normalize)} con L1 byte-identical (test) Y promotedreamrun real aplica el plan a l1/<session> (ADD/UPDATE/DELETE/NOOP..."
---

---
title: "VER-07: Dreams — dry-run + diff report + `promote_dream_run` real"
kind: task
description: "dry-run con diff por registro (ADD/UPDATE/DELETE/NOOP + razón) y L1 byte-identical; promote real a l1/<session> idempotente con puerta de calidad; MCP dream_promote readOnlyHint false + docs al día."
---

# VER-07: Dreams — dry-run + diff report + `promote_dream_run` real

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 34, Fase F4, wave F4.1)
- **Fuente:** plan Task 34 + Backlog:929 (P52 2026-09-24)
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴
- **Tipo:** Rust (vanta-memory) + MCP (vantadb-mcp) + Docs (`docs/api/`)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-29T14:55
- **last-synced:** 2026-09-29T19:05
- **Estado:** ⏳ IN PROGRESS (steps 1-5 ✅ + batch review P2-01 ✅; commit = LEAD)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (commit/git = externo)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory/src/services/pipeline_worker.rs` (`run_dream` :545-565 — solo `consolidate_session`, no promote), `vantadb-mcp/src/dreams.rs` (`dream_promote` :254-261), `vanta-memory/tests/dreaming.rs`, `vantadb-mcp/tests/dream_tests.rs` |
| Callees | `vanta-memory/src/core/record/` (`read_session_records`, `l1_namespace`, `sanitize_key`, `mark_contradiction`), `vantadb::sdk::Embedded` (put/delete/get/list) |
| Implicaciones | `promote_dream_run` cambia firma (`Result<usize>` → `Result<PromotionPlan>`) — breaking interno de crate `publish=false`, no superficie semver. `DreamRun` gana campo aditivo `input_ids` (serde default, retro-compatible con run.json viejos). MCP `dream_promote` pasa de read-only a write (hints + descripción + meta-test). L1: **primera** mutación del módulo (solo vía promote explícito; en default dry-run no muta). Sin migración de datos; sin re-indexación. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/dream/mod.rs` (1016L), `vantadb-mcp/src/dreams.rs` (267L), `vanta-memory/tests/dreaming.rs` (323L), `vantadb-mcp/tests/dream_tests.rs` (383L), `vanta-memory/src/core/record/l1_reader.rs` (322L), `vanta-memory/src/core/record/l1_writer.rs` (386L), `vanta-memory/src/core/record/mod.rs` (46L), `vanta-memory/src/core/abstractions/types.rs` (:40-219, MemoryRecord completo).
- **Regiones leídas (archivos grandes):** `vantadb-mcp/src/handlers/tools.rs` (:23-76 comentarios de hints, :1160-1260 lista de perfiles, :3085-3105 dispatch; ~3110L total — cambios limitados a comentarios + lista destructiva + dispatch sin cambio), `vantadb-mcp/tests/mcp_tests.rs` (:4592-4731 meta-test de anotaciones), `vanta-memory/src/services/pipeline_worker.rs` (:520-565, :790-839), `.opencode/rules/{core-engine,api-contract,server-mcp}.md` (completos).
- **Archivos referenciados hacia dentro (imports):** `core::record::{read_session_records, l1_namespace, L1Error}`, `core::record::lifecycle::{mark_contradiction, ContradictionProvenance}`, `core::conversation::sanitize_component`, `vantadb::sdk::{Embedded, MemoryInput, MemoryMetadata}`.
- **Referencias entrantes (grep):** `promote_dream_run` ← `dreams.rs:22,261`, `dreaming.rs:25,289`; `DreamRun` ← `dream/mod.rs` (list/load/discard/write) + `docs`; `consolidate_session` ← `pipeline_worker.rs:28,562`, `dreams.rs:22,240`, tests; `list_dream_runs` ← `pipeline_manager.rs:16,665,702`.
- **Veredicto impacto:** **medio**. Nada fuera de `vanta-memory` + `vantadb-mcp` + docs citadas. `DreamRun` aditivo (serde default) → run.json existentes cargan; el promote real muta `l1/<session>` solo con `dry_run:false` explícito (default true). Riesgo principal: semántica de DELETE — acotada por `input_ids` (solo registros escaneados por el run) y gate de supersede.

## Contrato

> Verbatim del plan (Task 34), es ley:

"`dry_run:true` (CLI/MCP) devuelve diff por registro {action: ADD|UPDATE|DELETE|NOOP, key, razón (merge|dedup|supersede|normalize)} con L1 byte-identical (test) Y `promote_dream_run` real aplica el plan a `l1/<session>` (ADD/UPDATE/DELETE/NOOP; idempotente: re-promote → todo NOOP) con puerta de calidad sobre merges/supersedes Y MCP `dream_promote` deja de ser preview (readOnlyHint false + descripción/hints + meta-test de conteos) Y tools MCP + docs (`MCP.md`/`VANTA_MEMORY.md`) al día"

**Nota de alcance CLI:** `src/cli.rs` es región co-batch de VER-01 en esta wave → PROHIBIDO tocarlo. No existe comando CLI de dreams hoy. La superficie CLI queda servida por la API core `plan_promotion` (el CLI podrá cablearla cuando VER-01 libere su región). Documentado como deuda/FIND.

## Spec (SDD — símbolos públicos nuevos)

> Gate D evaluado: el contrato del plan ya prescribe la superficie externa (param `dry_run`, acciones ADD|UPDATE|DELETE|NOOP, idempotencia, flip de hints). Los símbolos nuevos viven en `vanta-memory` (crate `publish=false`, core-only por Gate P/HARD-04 — SIN bindings nuevos) y son consecuencia mecánica del contrato. Decisiones cerradas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Identidad de diff | A: por `id` de registro (clave de `l1/<session>`) + `namespace` explícito por op / B: por (session_key, id) compuesto | A | ✅ decidido-por-evidencia (plan pre-mortem F1: "diff por (namespace, key)"; `l1_reader.rs:9-10` key = id; `l1_writer.rs:326` key = `sanitize_key(id)`) |
| 2 | Semántica DELETE | A: solo registros **escaneados** por el run (`input_ids`) y ausentes de `consolidated` / B: todo registro L1 ausente del view (borra adiciones post-run) | A | ✅ decidido-por-evidencia (pre-mortem "NOOP como default seguro"; F2 idempotencia; adiciones post-run no deben morir por un run stale). Requiere campo aditivo `DreamRun.input_ids` (serde default → run.json viejos = sin deletes) |
| 3 | Aplicación de supersede | A: stamping en `consolidate_session` sobre la copia dream-side (run.json = estado objetivo) y promote aplica verbatim / B: promote deriva de `contradicted_ids` en el momento | A | ✅ decidido-por-evidencia (`resolve_contradictions` doc :317-321 declara "The caller persists the loser's updated superseded_by"; B rompe idempotencia: L1≠consolidated tras apply → re-promote no sería NOOP) |
| 4 | Puerto de calidad | A: gate mínimo fail-closed: (i) supersede target debe existir post-apply, (ii) duplicate keys rechazadas, (iii) DELETE solo a `input_ids` / B: esperar spec MGR-08 (cuarentena/derived) | A | ✅ decidido-por-evidencia (stop condition del plan: "entregar ... con las reglas existentes de consolidate_session y FIND para la puerta avanzada"; MGR-08 fuera del plan) |
| 5 | Forma de respuesta MCP | A: `{dry_run, mutated, counts{add,update,delete,noop}, ops[{action(MAYÚS),namespace,key,reason(snake)}]}` / B: solo counts | A | ✅ decidido-por-evidencia (contrato pide diff por registro; plan :901) |
| 6 | Default de seguridad | A: `dry_run=true` default (mutar exige `dry_run:false` explícito) / B: apply default | A | ✅ decidido-por-evidencia (risk register: "dry-run obligatorio por default + apply idempotente"; incremental Rule 4 safe defaults) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `consolidate_session`/`scan_session_records`/`plan_promotion` **nunca** mutan `l1/<session>` (solo `promote_dream_run` con apply lo hace).
  2. `promote_dream_run` es idempotente: re-promote con L1 = consolidated ⇒ plan 100% NOOP, cero escrituras.
  3. DELETE limitado a IDs escaneados por el run (`input_ids`); jamás borra adiciones post-run.
  4. Gate fail-closed: plan con supersede colgante o clave duplicada ⇒ `ConsolidationError::QualityGate`, cero mutación.
  5. Serialización `run.json` retro-compatible (campo aditivo `input_ids` con `#[serde(default)]`).
- **Comandos de verificación:** `cargo nextest run --profile audit -p vanta-memory --test dreaming` · `cargo nextest run --profile audit -p vantadb-mcp --test dream_tests` · `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` · `cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings` · `cargo fmt --check`.
- **Deuda pendiente:** puerta de calidad avanzada (MGR-08: cuarentena `derived_promotion` / calibración MEM-70→VER-08) y superficie CLI (región VER-01) → FIND/flow.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | `# VER-07: Dreams — dry-run + diff report + promote_dream_run real` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | VER-01 / VER-05 (wave F4.1) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — el cambio **elimina** la deuda del stub (`promote_dream_run` preview-only, invariante #4 "promote is a stub") y no introduce `unwrap`/`unsafe`/deps nuevas. `ponytail:` documents: DELETE sin tombstone (MGR-08), gate mínimo (FIND).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del plan (verbatim arriba) verificado por tests + fmt/clippy/nextest |
| **Commit** | Atómico, conventional (`feat(dreams):`), sin push (política: commit = LEAD; push = owner) |
| **Release** | N/A — sin cambio de versión/packaging; verify full del cierre lo corre el LEAD |

## Herramientas necesarias
- codegraph / grep (blast radius) · cargo nextest scoped `-p vanta-memory` y `-p vantadb-mcp --ignore-default-filter` · clippy `-D warnings` · fmt · `CARGO_BUILD_JOBS=2` · `node scripts/docs/*.mjs` (docs) · `pwsh dev-tools/ocr-review.ps1` (advisory, cierre LEAD)

**Skills cargadas (SDP):** `test-driven-development` (pinned; RED→GREEN del promote), `systematic-debugging` (pinned; retry ladder), `source-driven-development` (MCP hints/spec), `security-and-hardening` (mutación de storage = trust boundary), `incremental-implementation` (5 steps reversibles), `context-engineering` (aplicada vía §3a del agente; sin carga separada), `rust-write-tests` (calidad de tests del slice), `doubt-driven-development` (delegada al review fresco P2-01 del LEAD). SDP v3 base: campaign-executor/progreso auto-cargadas via MCP.

## Investigation Notes
- Web research: no requerida (sin APIs externas; contrato del plan + código local).
- Precedente dry-run revisado: `wal salvage --dry-run` (`src/cli.rs:443-447`) — patrón "preview sin mutar" ya usado en el repo; no aplicable directo (región VER-01 off-limits).
- Doc `dream/mod.rs:29-36` ("deliberately not wired into pipeline_worker yet") es **STALE** (verificado: `run_dream` :545 + `TaskKind::Dream` :810 ya existen) → se corrige en Step 2.
- Backlog:929 refiere `vantadb-mcp/src/handlers/dreams.rs` — path real **`vantadb-mcp/src/dreams.rs`** (re-baseline confirmado).
- `read_session_records` filtra `include_quarantined:false` (SCH-05) → registros en cuarentena quedan fuera del plan de promoción (ni UPDATE ni DELETE). Comportamiento seguro; interacción fina con cuarentena `derived_promotion` → MGR-08/FIND.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — diseño cerrado por contrato + evidencia (ver Spec) |
| Pendientes de ejecución (downhill) | 5 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — toca storage (L1) = trust boundary: promote aplica solo tras plan + gate fail-closed; default dry-run; DELETE acotado a `input_ids`; sin inputs de red. `security-and-hardening` checklist: input validado en frontera MCP (`validate_identifier`, `run_id_arg`), sin secrets, sin nuevos deps, errores como content (sin stack traces). ✅
- [x] **PERFORMANCE** — no toca hot path de búsqueda/indexación; diff es O(n) sobre sesión (n ≤ registros L1 de una sesión). No aplica `performance-optimization` (sin loops calientes persistentes). ✅

## Steps

### Step 1: RED — tests core del promote/dry-run (failing)
- **Archivos:** `vanta-memory/tests/dreaming.rs`
- **Acción:** reemplazar el test 6 del contrato viejo ("promote MUST NOT mutate") por la nueva suite RED: (a) dry-run determinista + L1 byte-identical; (b) promote aplica (supersede+normalize), idempotente (2º = todo NOOP), L1 estable entre corridas; (c) DELETE solo de escaneados + adición post-run intacta; (d) gate bloquea supersede colgante; (e) run viejo sin `input_ids` no borra nada; (f) `consolidate_session` stampea `superseded_by` en la copia dream-side (test 4 extendido).
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test dreaming` → RED confirmado (12 errores E0432/E0599/E0609/E0610 por símbolos inexistentes)
- **Estado:** ✅ DONE

### Step 2: GREEN — core `plan_promotion` + `promote_dream_run` real
- **Archivos:** `vanta-memory/src/core/dream/mod.rs`, `vanta-memory/src/core/record/l1_writer.rs` (`put_record` → `pub(crate)`), `vanta-memory/src/core/record/mod.rs` (re-export)
- **Acción:** tipos `PromotionAction/Reason/Op/Counts/Plan` + `plan_promotion` (diff puro + gate) + `promote_dream_run` (plan→apply idempotente) + `DreamRun.input_ids` + stamping supersede en `consolidate_session` + `ConsolidationError::QualityGate` + fix doc stale.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test dreaming` → **11/11 pass** (tras workaround delete+put por bug de cache del engine — ver Notas) + suite completa `-p vanta-memory` → **554/554 pass**
- **Estado:** ✅ DONE

### Step 3: MCP — `dream_promote` deja de ser preview (tests RED→GREEN)
- **Archivos:** `vantadb-mcp/tests/dream_tests.rs`, `vantadb-mcp/tests/mcp_tests.rs`, `vantadb-mcp/src/dreams.rs`, `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** handler con `dry_run` param (default true), respuesta `{dry_run,mutated,counts,ops}`; defs: descripción + `readOnlyHint:false` + `destructiveHint:true`; comentarios de hints + `destructive_set` del meta-test.
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp --ignore-default-filter --test dream_tests` → **12/12**; `--test mcp_tests` → **111/111**; suite completa `-p vantadb-mcp` → **250/250**
- **Estado:** ✅ DONE

### Step 4: Docs — MCP.md + VANTA_MEMORY.md al día
- **Archivos:** `docs/api/MCP.md`, `docs/api/VANTA_MEMORY.md`
- **Acción:** `dream_promote` real (dry_run + destructive 13 tools), párrafo `core::dream`, tabla de namespaces, refs de línea actualizadas.
- **Verify:** `check-links` ✅ (exit 0) · `check-docs` ✅ mi parte (único gating: VER-01.md, ajeno) · `gen-index --check` ✅ · `validate-docs-coverage` → MCP tools parity 47 ok ✅ (falla solo por comando `verify` CLI de VER-01)
- **Estado:** ✅ DONE — edits absorbidos en commit ajeno `1bf4df1b` (barrido docs concurrente)

### Step 5: Verify full del bloque
- **Archivos:** —
- **Acción/Verify:** `cargo fmt -p vanta-memory -p vantadb-mcp -- --check` ✅ · `cargo clippy -p vanta-memory -p vantadb-mcp --all-targets -- -D warnings` ✅ · suites completas vanta-memory (555) + vantadb-mcp (250) ✅ · spot `-p vantadb --test memory_api` 9/9 ✅ · OCR advisory (ver Notas).
- **Estado:** ✅ DONE

## Batch review P2-01 (opt-in aplicados — owner 2026-09-29)

> Review P2-01: **✅ APPROVE** (0 Critical/Required). Owner pidió aplicar opcionales baratos: Opt 1 + Opt 3 + Nit 1 + Nit 2. Opt 2/Nit 3 según instrucción.

- [x] **Opt 1 — vector fallback en UPDATE** (`promote_dream_run`): si la copia dream trae `vector: None`, el apply usa el vector del nodo L1 actual (mapa del snapshot de `build_promotion`) antes del delete+put; `build_promotion` ahora devuelve `(plan, run, l1)`. Evidencia RED→GREEN: test `dream_promote_update_falls_back_to_current_l1_vector` FALLA con fallback deshabilitado (`left: None, right: Some([0.1,0.2,0.3])`) y PASA con el fix; payload-diff sigue aplicándose.
- [x] **Opt 3 — consumidores stale**: `skills/vantadb-mcp/SKILL.md` + `references/api-reference.md` actualizados a la superficie real (`dry_run` default true, `{dry_run,mutated,counts,ops}`, destructive). **Espejado en `.opencode/skills/vantadb-mcp/`** (gate skills-mirror FIND-83: hash-SAME re-verificado).
- [x] **Nit 1 — refs de línea** `docs/api/VANTA_MEMORY.md:153`: `:946` (`consolidate_session`) · `:720` (`plan_promotion`) · `:869` (`promote_dream_run`).
- [x] **Nit 2 — legacy sin campo**: test `dream_promote_legacy_run_without_input_ids_never_deletes` ahora serializa el run a JSON, remueve `input_ids` del objeto y persiste el payload crudo (deserialización con `#[serde(default)]` verificada explícitamente).
- [x] **Nit 3 (no aplicar) — documentado en código**: `dreams.rs` — `dry_run` ausente o no-bool → default seguro `true` (nunca apply implícito).
- **Re-verify batch:** `-p vanta-memory --test dreaming` **12/12** · `-p vanta-memory` **555/555** · `-p vantadb-mcp --ignore-default-filter` **250/250** · fmt ✅ · clippy ✅ · check-links ✅ · check-docs "all clear" ✅ · validate-docs-coverage exit 0 (skills mirror 10 pares SAME) ✅.

## Dependencias
- Task 34 declara "Dep: — (aditivo; consume MEM-69/70)". MEM-69 (batch) ya wired; MEM-70 (harness) fuera. MGR-08 fuera del plan (stop condition). Sin bloqueos.

## Review (GATE — agente distinto, P2-01)

> Tier: mixto → paths `vanta-memory/**` + `vantadb-mcp/**` + `docs/api/**` → adversarial. Contrato verificado por cláusula, idempotencia real, gate fail-closed y dry-run sin mutación.
> OCR advisory: `pwsh dev-tools/ocr-review.ps1 -Format json` ejecutado — preview + delegación por Rule Groups (group 5: {vantadb-mcp/**, src/server/**}; grupo vanta-memory/dreams) generados como input del reviewer.

- **Revisor:** P2-01 fresco (`vanta-review`, contexto ≠ autor) — **✅ APPROVE** (0 Critical/Required), comunicado vía owner 2026-09-29.
- **Batch post-approve:** Opt 1 + Opt 3 + Nit 1 + Nit 2 aplicados y re-verificados (ver §Batch review P2-01); Nit 3 documentado en código; CLI deferral aceptado por LEAD (no se agrega CLI).
- **Harvest/commit:** LEAD (regla de wave). Nota Gate H: se tocó el espejo `.opencode/skills/vantadb-mcp/` (sync FIND-83) → `/harness` antes del commit del harness.
- **Veredicto:** ✅ approve (cerrado)

## Notas
- Gate D: evaluado y **no-bloqueante** — el contrato del plan prescribe la superficie; símbolos nuevos confinados a `vanta-memory` (publish=false, core-only Gate P); sin tools MCP nuevas. Decisiones registradas en §Spec con evidencia.
- Gate V: no disparado (0 fallas de verify con mismo error).
- **HALLAZGO (FIND candidato — dominio engine, no VER-07): coherencia de cache en overwrite de nodos Cold.**
  - Evidencia: durante este task, `promote` con put-overwrite quedaba invisible a TODAS las lecturas (get/list/get_many) tras el flow consolidate→plan; aislado con tests/probes: nodos de memory-records se insertan `NodeTier::Cold` (`src/node/unified.rs:86`; `memory_record_to_node_owned` no setea tier), `apply_insert` solo refresca la volatile cache para `Hot` (`src/storage/engine/insert.rs:669-676`), el prefetcher del read-path cachea incondicional (`prefetch_related`, `src/storage/engine/get.rs:436-440`), y el overwrite no invalida → lecturas stale hasta evicción/restart. `delete` sí evicta (`src/storage/engine/delete.rs:137`).
  - Workaround aplicado (consistente con `l1_writer` update/merge): UPDATE = `delete` + `put` en `promote_dream_run` (documentado en el código).
  - Acción sugerida al LEAD: crear fila `FIND-*` (Backlog) para fix de engine (`vanta-engine`): invalidar/refresh de cache en overwrite de nodos Cold o gate de tier en `prefetch_related`.
- FIND candidates (para el LEAD — Backlog.md es LEAD-only en esta wave):
  1. **Engine cache coherence** (arriba) — prioridad alta (afecta lecturas post-update en cualquier path put-overwrite).
  2. ~~skills/vantadb-mcp stale~~ → **resuelto en este batch** (Opt 3; skills/ + espejo `.opencode/` actualizados).
  3. Superficie CLI de dreams (`dry-run` CLI) — requiere región `src/cli.rs`; **deferral aceptado por el LEAD** (no se agrega CLI).
  4. Puerta de calidad avanzada MGR-08 (cuarentena `derived_promotion`, `quarantined_by=system:dream_promote`) — stop condition del plan.
- Notas de integración wave F4.1: los edits de docs de este task fueron absorbidos por un commit docs concurrente (`1bf4df1b`); el código queda en working tree para el commit del LEAD. `validate-docs-coverage` y `check-docs` ahora **verdes enteros** (los fallos ajenos de la primera pasada — comando CLI `verify` y `VER-01.md` — fueron resueltos por el co-batch).
- Nit 3 (documentado en código, no aplicado): `dry_run` ausente o no-bool en MCP → default `true` (plan only, nunca apply implícito).
- `ponytail:` promoción aplica el view como batch de upserts por key (idempotente); sin transacción global — techo conocido: crash mid-apply deja L1 parcial, re-promote converge (documentado en el módulo).
- Scope discipline: NO se toca `src/cli.rs`, `src/wal*.rs`, `src/audit.rs`, `src/sdk/serialization/**` (regiones co-batch). `skills/` + espejo `.opencode/skills/` tocados SOLO por Opt 3 (sync mirror, hash-SAME).
