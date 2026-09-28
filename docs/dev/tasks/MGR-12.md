# MGR-12: Jerarquía asserted/derived + scores + derivación (D6/AM1/AM7 — research, cero implementación)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` Task 24 (Fase F3) · consumido por SCH-01/SCH-02/SCH-04/SCH-05 (P53, migración única 0.8.0)
- **Fuente:** Backlog `P49:846` · Decisión owner 2026-09-14: migración única junto con MGR-10/13 (`Backlog:934`) · pre-requisito duro de SCH-01 (`Backlog:938`)
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** research/design (doc-only, cero código)
- **Turns estimados:** 12-18
- **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ EN PROGRESO — steps ✅ para el scope del sub-agente; commit + review P2-01 + `completed` = LEAD
- **Incógnitas (uphill):** 1 (taxonomía operacional sin jueces → resuelta, doc §2) · **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `docs/dev/research/mgr-12-confianza.md` (nuevo — único deliverable) + `docs/dev/tasks/MGR-12.md` |
| Callees | Solo lectura de código: `src/sdk/types/record.rs`, `vantadb-mcp/src/axioms.rs`, `src/node/unified.rs`, `src/config.rs`, `src/eviction.rs`, `src/executor.rs`, `src/utils/confidence_metrics.rs`, `src/sdk/serialization/mod.rs`, `src/sdk/version_history.rs`, `src/sdk/api/memory.rs`, `src/sdk/api/graph.rs`, `src/llm.rs` — sin edición |
| Implicaciones | Cero cambios de código productivo. Riesgos: scores decorativos (sin semántica), doble fuente record↔nodo divergente, scope creep a jueces LLM. El impacto real es contractual: los campos definidos aquí viajan a SCH-02 (breaking v0.7.0) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos o rangos citados):** `src/sdk/types/record.rs:1-553` · `vantadb-mcp/src/axioms.rs:1-56` · `src/utils/confidence_metrics.rs:1-166` · `src/sdk/version_history.rs:1-160` · `src/node/unified.rs:1-140` · `src/eviction.rs:1-120` · `src/executor.rs:190-269,455-524` · `src/sdk/api/graph.rs:195-264` · `src/sdk/api/memory.rs:60-180` · `src/sdk/serialization/mod.rs:25-54,85-159,380-528` · `src/node/disk.rs:1-50` · `src/config.rs:270-309` · `src/schema.rs:1-60` · `src/llm.rs:928-952` · `vantadb-mcp/src/handlers/tools.rs:1700-1759,1980-2039`
- **Referencias hacia dentro:** plan Task 24; `Backlog:846,934,938`; `docs/dev/strategy/ROADMAP-v0.7.md:43`
- **Referencias entrantes:** SCH-01 (consolida dims 5-6), SCH-02 (campos), SCH-04 (scores), SCH-05 (trust-aware/abstención), MGR-18 (post-1.0, `Backlog:867`), MGR-13 (co-batch — clase de confianza mínima)
- **Veredicto impacto:** nulo en código (doc-only). Contractual alto: el modelo §3 del research-doc es input normativo de SCH-02

## Contrato
"research-doc cerrado con modelo de confianza asserted/derived (scores por registro + reglas de derivación + `last_validated` + calibración básica), tradeoffs y mapeo al `confidence_score` de nodo, listo para SCH-01."

## Spec (SDD — Phase 1b)
Cero implementación, cero símbolos. No es feature-add. Excepción canónica (`question-gates.md` §"Contenido válido de `## Spec`"): tarea 100% docs sin código → "sin decisiones técnicas en código" + archivos tocados: `docs/dev/research/mgr-12-confianza.md` (nuevo) + `docs/dev/tasks/MGR-12.md`. Las decisiones de diseño del modelo viven en el research-doc (§3 modelo, §7 tradeoffs) y las 4 decisiones abiertas al owner en §9 (opciones + default recomendado).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** (1) `MemoryRecord`/`MemoryInput` son contrato público — todo campo nuevo es aditivo con `#[serde(default)]` (One-Version Rule); (2) semántica canónica de confianza en el record; el nodo es proyección de un único punto de mapeo (anti-divergencia); (3) cero jueces LLM / calibración empírica en este slice (VER-08/v1.0, `Backlog:934`); (4) el `confidence_score` de nodo hoy es 0.5 constante para todo record — el mapeo debe preservar el comportamiento observable existente (Hyrum).
- **Comandos de verificación:** `pwsh scripts/validate-docs-coverage.ps1` + revisión mecánica de citas (HEAD 200, GATE CITAS) + lectura del contrato §1-§10 del doc.
- **Deuda pendiente:** ninguna al abrir.

## Recitation
```
=== RECITATION ===
Objetivo activo: MGR-12 — research-doc confianza (dim 6)
Estado: in-progress (desde: PENDING)
Última acción: discovery codebase + research externo (18 URLs HEAD-verificadas) + research-doc redactado
Resultado: OK (scope sub-agente)
Próxima acción: LEAD — commit docs + review P2-01 + preguntas owner §9 + campaign_update(completed)
Contrato: ver ## Contrato
Invariantes: campos aditivos; canónico en record; sin jueces; nodo 0.5 compat
Deuda: ninguna
Próxima tarea si completa: SCH-01 (plan único + ADR, Task 26)
last-synced: 2026-09-28
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** sin deuda (research docs-only; no se toca código).

## Definition of Done (3 niveles)
- **Task:** contrato + Cierre MGR (research-doc + preguntas owner + plan de implementación)
- **Commit:** `docs:` + MGR-12 (solo docs/) — lo ejecuta el LEAD (sub-agente sin permiso de commit)
- **Release:** N/A (research sin cambio funcional)

## Herramientas necesarias
- `webfetch`/`websearch` (citas verificadas con HEAD/fetch), codegraph/grep/read (evidencia código-real), `scripts/validate-docs-coverage.ps1`
- **Skills cargadas (SDP v3, phase DEFINE):** campaign-executor + progreso (base auto vía MCP) + source-driven-development + security-and-hardening + spec-driven-development + interview-me + idea-refine + documentation-and-adrs + writing-guidelines + api-and-interface-design + coordinated-web-search (router de búsqueda web)

## Investigation Notes
- **Gap re-verificado 2026-09-28 (HEAD):** `MemoryRecord` sin score/clase/`last_validated` (`record.rs:102-138`); `MemoryInput` sin vía de declaración (`:59-78`); `rg last_validated` repo = 0 hits; `asserted` = 0 hits de dominio (solo comentarios de tests). `vantadb-mcp/src/axioms.rs:22-30` = 4 Iron Axioms hardcodeadas (JSON), el axioma #2 "Confidence Constraint" menciona historical Confidence Score pero no implementa clase por registro.
- **Nodo con confianza desconectada:** `UnifiedNode.confidence_score: f32` (`unified.rs:41-42`, default 0.5 `:92`, persistida en disk header `disk.rs:18-19` offset 40); `memory_record_to_node_owned` (`serialization/mod.rs:433-495`) NO la setea (todo record queda 0.5); `record_from_node` (`:305`, construcción `:416-430`) NO la lee. Única vía pública que la setea: `restore_graph_nodes` (`graph.rs:231`) con `NodeRecord.confidence_score` (`graph_types.rs:69`). Consumidores existentes: eviction (`config.rs:285`, fórmula `eviction.rs:38-45`, default 2.0 `:63-68`), filtro executor `<0.4` solo `SemanticSummary` (`executor.rs:219-242`), prompt LLM (`llm.rs:940`), MCP `get_node_neighbors` (`tools.rs:2011`), server state (`state.rs:74,84`). `StaleContext` (`executor.rs:33`) declarado sin productor (solo consumidores).
- **Escape hatch:** metadata libre (`validate_metadata` solo bloquea prefijo `__vanta_`, `serialization/mod.rs:12,136-150`) — nada lee claves de confianza.
- **Multi-agente:** `OriginCollisionTracker` (`confidence_metrics.rs:8-94`, export `lib.rs:203`) — EMA por `_owner_role` + slash + friction; no cableado a records. `_owner_role` en relational se usa para RBAC pruning (`executor.rs:499-511`).
- **Citas externas verificadas (2026-09-28, HEAD 200 o fetch):** Guo ECE arXiv:1706.04599 · P(IK) arXiv:2205.14334 · Kadavath arXiv:2207.05221 · Xiong arXiv:2306.13063 · selective classification arXiv:1705.08500 · conformal arXiv:2107.07511 · TruthfulQA arXiv:2109.07958 · FACTS Grounding (deepmind.google/blog/…) · FaithJudge arXiv:2505.04847 · PROV-O w3.org/TR/prov-o · GraphDB asserted/inferred (graphdb.ontotext.com/documentation/11.0/reasoning.html) · Jena inference (jena.apache.org/documentation/inference/) · Generative Agents arXiv:2304.03442 · Zep/Graphiti arXiv:2501.13956 · Mem0 arXiv:2504.19413 · Self-RAG arXiv:2310.11511 · CRAG arXiv:2401.15884. MetaSearchMCP wedgeado (todos los providers timeout) → keyless webfetch/websearch.
- **Destraba:** SCH-01 → SCH-02 (campos + backfill) → SCH-04 (scores) / SCH-05 (trust-aware). Deferrals citados: grounding/jueces = v1.0 (`Backlog:934`); calibración empírica = VER-08 (F5, dep cross-fase).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — taxonomía operacional cerrada (doc §2, sin jueces) |
| Pendientes | 4 steps (1-4) |
| % completado | 100% scope sub-agente (cierre formal = LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A: doc-only, sin input externo ni cambio de superficie. El doc alimenta el trust-aware de SCH-05 (consumidor), no lo implementa.
- [x] **PERFORMANCE** — N/A: sin cambio de código. La calibración/medición de scores queda en VER-08 (Regla 9/11 no aplican a un doc).

## Steps
### Step 1: Discovery codebase (gap + infra base)
- **Archivos:** los listados en Blast Radius (lectura)
- **Acción:** re-verificar gap (record sin confianza) + mapear infra de nodo (eviction/executor/mapping) con rg/read
- **Verify:** cada claim con `archivo:línea` en Investigation Notes
- **Estado:** ✅ DONE (2026-09-28)

### Step 2: Research externo + verificación de citas (GATE CITAS)
- **Archivos:** refs externas (calibración, provenance, truthfulness, abstention, memoria)
- **Acción:** recolectar y verificar mecánicamente cada URL (HEAD 200 / fetch)
- **Verify:** 0 citas sin resolver; las no verificadas se marcan o descartan
- **Estado:** ✅ DONE (18 URLs verificadas 2026-09-28; 1 descartada: blog FACTS con URL vieja → URL canónica verificada)

### Step 3: Redactar research-doc (taxonomía + modelo + mapeo + calibración + tradeoffs + 0.8.0 vs v1.0)
- **Archivos:** `docs/dev/research/mgr-12-confianza.md` (nuevo)
- **Acción:** doc §1-§10 según contrato; mitigar pre-mortem F1/F2/F3 del plan (taxonomía operacional, exclusión explícita de jueces, mapeo canónico)
- **Verify:** contrato cubierto sección a sección; `validate-docs-coverage.ps1`
- **Estado:** ✅ DONE

### Step 4: Cierre MGR (preguntas owner + plan de implementación + task sync)
- **Archivos:** research-doc §9-§10, task file, recitation
- **Acción:** 4 preguntas owner con opciones + default (ruteo: LEAD, sub-agente sin permiso `question`); plan de implementación para SCH-01/02/04/05/VER-08; sync task file + recitation
- **Verify:** preguntas ruteadas + plan con archivos/comandos concretos
- **Estado:** ✅ DONE (scope sub-agente) — commit + review P2-01 + `completed` = LEAD

## Dependencias
- Ninguna. Co-batch F3 con MGR-10/MGR-13 (archivos disjuntos — no tocados). Destraba SCH-01 (pre-requisito duro `Backlog:938`).

## Review (GATE P2-01)
- **Revisor:** `vanta-review` fresco (sesión `ses_f169edc6affe2nAvjZppBCV2es`; ≠ autor `ses_f16afedcafferxMld0J0BAe1Ud`) — **✅ APPROVE** (2026-09-28). 0 Required; 4 Optional (semántica `Derived+Some` → SCH-01 · `D_a` vs 0.5 → SCH-01 · punto de escritura residual → SCH-01 · predicado import v2 → SCH-02) + 5 Nits (4 aplicados). 30+ refs re-verificadas; 5/18 fuentes spot-checked.

## Notas
- WIP ajeno PROHIBIDO: `docs/dev/research/mgr-10-*.md` y `mgr-13-*.md` (co-batch) — no tocar.
- Plan file (`2026-09-26-master-roadmap.md`) y `docs/dev/Backlog.md` NO editados (co-batch comparte el archivo; cierre/status = LEAD).
- Hallazgo lateral NO tocado: `StaleContext` sin productor (`executor.rs:33`) — candidato a FIND para SCH-05 (es la señal natural de abstención/rehydration); documentado en el research-doc §6, sin fila FIND nueva (evitar edición de Backlog compartido en co-batch).
