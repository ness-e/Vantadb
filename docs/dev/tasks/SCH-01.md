---
title: "SCH-01: Plan único + ADR de migración (dims 5-6; alcance 0.8.0 vs v1.0)"
kind: task
description: "ADR de migración aceptado (campos, semántica valid vs transaction, alcance 0.8.0 vs v1.0, plan de migración/backfill, compat export/import) + plan de implementación único con comandos + revisión owner registrada\""
---

# SCH-01: Plan único + ADR de migración (dims 5-6; alcance 0.8.0 vs v1.0)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` — Task 26 (F3) · **Origen:** `docs/dev/Backlog.md:938` (+ decisión owner de migración única `:934`)
- **Fuente del prompt:** sub-agente vanta-arch (orquestador pipeline) — ejecución directa de SCH-01
- **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴 · **Tipo:** análisis + ADR (doc-only, cero código)
- **Turns estimados:** 10-15 · **Creado:** 2026-09-28 · **last-synced:** 2026-09-28
- **Estado:** ⏳ IN PROGRESS (ADR-0046 + task file ✅; verify mecánico ✅; review P2-01 + commit/firma = LEAD)
- **Incógnitas (uphill):** 2 → resueltas (reconciliación de insumos + decisión edges SCH-09) · **Pendientes (downhill):** verify + cierre LEAD

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Alcance | `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` (nuevo), `docs/dev/tasks/SCH-01.md` (nuevo) — docs only |
| Callees | Solo lectura: 3 research-docs MGR-10/12/13 + task files; ADR-045/044/028; `docs/api/{VERSIONING,COMPATIBILITY,DEPRECATIONS}.md`; `release-plz.toml`; `docs/CHANGELOG.md`; código: `src/sdk/types/record.rs`, `src/sdk/serialization/mod.rs`, `src/sdk/version_history.rs`, `src/sdk/api/graph.rs`, `src/schema.rs`, `src/eviction.rs`, `src/llm.rs`, `src/config.rs`, `src/node/unified.rs` |
| Implicaciones | Cero cambios de código. El ADR fija el contrato que SCH-02..08 (y SCH-09) consumen verbatim; es pre-requisito duro de `record.rs` (Backlog:938). Riesgo de diseño: ambigüedad por insumos divergentes (mitigado: tabla de reconciliación §D8 del ADR) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos / secciones):** `mgr-10-bitemporalidad.md` (238L, completo), `mgr-12-confianza.md` (286L, completo), `mgr-13-cuarentena.md` (238L, completo); `docs/dev/_templates/adr.md`; `ADR-0045-naming-freeze.md` (144L, precedente de estructura/firma); `ADR-0044` (precedente breaking acumulado); `docs/api/VERSIONING.md` (119L), `COMPATIBILITY.md` (112L), `DEPRECATIONS.md` (44L); `release-plz.toml`; `CHANGELOG.md:1-30`; plan master Tasks 23-33 (L599-904) + gate F3 (:35) + carril owner (:981-994); Backlog :839-847 (MGR) y :934-946 (SCH); código: `record.rs:40-269`, `serialization/mod.rs:1-70,300-564`, `version_history.rs:60-149`, `graph.rs:195-269`, `schema.rs:1-40`, `eviction.rs:20-79`, `llm.rs:920-954`, `config.rs` (grep), `node/unified.rs` (grep)
- **Referencias hacia dentro:** plan Task 26 (:677-701); pre-req Backlog:938 (Cierre MGR de los 3 research-docs); 16 defaults ratificados por el owner (question 2026-09-28); convención ADR (template + precedente ADR-0045); rails HARD-01
- **Referencias entrantes:** SCH-02 (campos+backfill), SCH-03 (semántica AS OF), SCH-04 (scores), SCH-05 (cuarentena/abstención), SCH-06 (tests/determinismo/fechas de referencia), SCH-07 (superficies), SCH-08 (guía), SCH-09 (envelope v2 o FIND); VER-08 (calibración), MGR-18 (post-1.0)
- **Veredicto impacto:** bajo en código (docs-only; `src/` intacto), alto en contrato (fija la migración única del corte 0.8.0)

## Contrato
"ADR de migración aceptado (campos, semántica valid vs transaction, alcance 0.8.0 vs v1.0, plan de migración/backfill, compat export/import) + plan de implementación único con comandos + revisión owner registrada"

## Spec (SDD — Phase 1b)
Cero implementación, cero símbolos nuevos. El "spec" ES el ADR-0046: D1 (corte/alcance/edges), D2 (campos consolidados), D3 (semántica valid vs transaction), D4 (confianza + reglas V1-V5 + tests V2/V4/V5), D5 (cuarentena + deadline), D6 (record↔nodo), D7 (compat export/import), D8 (reconciliación de insumos) + planes de migración/implementación con comandos. No es feature-add: la ejecución del contrato es la revisión owner + review P2-01 (LEAD).

## Invariantes de dominio (handoff — MUST)
- **A preservar en SCH-02..08:** un solo breaking (nada de v3 — D1b); `#[serde(default)]` en TODO campo nuevo + normalización v1 en los 4 formatos (D7); backfill función pura sin reloj (idempotente/order-independent); `supersede()` mantiene `invalid_at == superseded_at` (D3-3); gates de inyección de cuarentena sin excepción (D5); invariante anti-divergencia record↔nodo (D6); los únicos deltas observables son los listados en D4c y se documentan en la guía UPGRADE (SCH-08).
- **No tocar:** `src/` en esta task (doc-only); otros task files del co-batch F3.
- **Comandos de verificación (esta task):** `npx markdownlint-cli2 <archivos>` + `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1`

## Owner sign-off (Regla 5) — REGISTRO (pre-mortem F3)
- **Base ratificada (2026-09-28):** los 16 defaults MGR-10/12/13 aprobados vía question ("Aprobar defaults y avanzar") — registrados en ADR-0046 §D0; no se re-abren.
- **Firma del ADR-0046:** ⬜ PENDIENTE — status `proposed`; el LEAD eleva la firma tras review P2-01 y la registra en `ADR-0046 §Owner sign-off` (slots `[OWNER]`: articulación, firma, riesgos aceptados).
- **Decisiones nuevas a firmar:** D4b (rechazo `Derived + Some(confidence)`), D4c (backfill `D_a = 1.0` uniforme + deltas), D6 (precedencia `restore_graph_nodes`), D8 (default de insert `valid_at`), D7 (import ≤2 / >2), D5d (deadline default 30d).
- **Este task file ES el registro del carril owner** (plan §Carril owner :981-994; pre-mortem F3 del bloque).

## Recitation
```
=== RECITATION ===
Objetivo activo: SCH-01 — Plan único + ADR de migración (dims 5-6; alcance 0.8.0 vs v1.0)
Estado: in-progress (desde: ⏳ EN PROGRESO del plan)
Última acción: DISCOVERY completo (3 research-docs + template + ADR-0045 + api docs + rails + código citado) + ADR-0046 redactado (propuesto; 6 pendientes P2-01 resueltos) + task file creado; verify docs ✅ (markdownlint 0 issues; validate-docs-coverage EXIT=0/0 gaps; 2026-09-28)
Resultado: PARTIAL
Próxima acción: LEAD — review P2-01 (tier Fast) + commit docs-only + elevar firma owner (proposed→accepted) + actualizar plan file
Contrato: ADR con campos/semántica/alcance/plan/compat + reconciliación D8 + owner sign-off registrado → ADR-0046 §§D1-D8 + §Migration + §Owner sign-off
Invariantes: doc-only; src/ intacto; un solo breaking (v2/0.8.0); 16 defaults ratificados no se re-abren; 6 pendientes resueltos explícitamente
Deuda: ninguna nueva (docs-only; saldo neto Regla 6 = 0)
Próxima tarea si completa: SCH-02 (tras firma/review de SCH-01 — no tocar record.rs antes)
last-synced: 2026-09-28
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)
**Saldo neto:** Sin deuda (análisis/ADR; cero código). La deuda que el ADR documenta (P27 crash-exactitud, cap de retención 32/key, calibración VER-08) es citada, no creada.

## Definition of Done (3 niveles)
- **Task:** contrato — ADR-0046 con campos consolidados (D2), semántica valid vs transaction (D3), alcance 0.8.0 vs v1.0 (D1d), plan de migración/backfill con comandos (§Plan de migración), compat export/import (D7), reconciliación de los 3 docs (D8), plan de implementación único (§Plan de implementación) + revisión owner registrada (§Owner sign-off) ✅
- **Commit:** `docs:` + SCH-01 (2 archivos) — **lo ejecuta el LEAD** (sub-agente sin commit ni self-review por instrucción)
- **Release:** N/A (la migración es el corte 0.8.0 = SCH-08)

## Herramientas necesarias
- Read/grep (insumos y código citado), markdownlint-cli2, `scripts/validate-docs-coverage.ps1`, campaign MCP (recitation)
- **Skills cargadas (SDP v3, DEFINE):** base `campaign-executor`+`progreso` (auto) · pin `deprecation-and-migration` (storage/schema — expand→backfill→bump) · `documentation-and-adrs` · `api-and-interface-design` · `spec-driven-development` · `writing-plans` · `writing-guidelines`

## Investigation Notes
- 3 Cierre MGR ✅ consumidos: MGR-10 (`b6614e1e`), MGR-12 (`50df4efd`), MGR-13 (`72f29720`); pre-req Backlog:938 satisfecho.
- 16 defaults ratificados por el owner (2026-09-28) consolidados como §D0 del ADR (no re-abrir).
- **6 pendientes P2-01 resueltos en el ADR:** (1) `Derived + Some(confidence)` → **rechazo en boundary** (D4b) + tests V2/V4/V5 enumerados; (2) `D_a` 0.5 vs 1.0 en backfill → **1.0 uniforme, delta aceptado pre-launch** citando consumidores (D4c: `eviction.rs:38-45` peso 2.0; `llm.rs:939-944`; `executor.rs:219-242`); (3) `restore_graph_nodes` (`graph.rs:231`) → **precedencia definida: dominio memory normaliza / restore transporte verbatim / last-writer físico + test** (D6); (4) `valid_at` insert → **`:= created_at_ms` salvo valor explícito** (D8); (5) import v2 → **aceptar ≤2 / rechazar >2** + boundaries de normalización v1 anotados para SCH-02 (D7); (6) deadline de cuarentena → **30d default, `keep`, nunca auto-promoción** (D5d).
- Riesgos del bloque mitigados: insumos divergentes → tabla D8; segundo breaking (edges SCH-09) → D1c (envelope v2 o FIND; nunca v3); alcance inflado → D1d estricta.
- Citas verificadas contra HEAD: `eviction.rs`, `llm.rs`, `graph.rs`, `record.rs`, `serialization/mod.rs`, `version_history.rs`, `schema.rs` leídos directo; el resto vía research-docs con `archivo:línea`.
- GATE CITAS: sin URLs nuevas en el ADR (las fuentes externas viven en los research-docs, ya verificadas — no se re-citan).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 — reconciliación (D8) + edges (D1c) + 6 pendientes resueltos |
| Pendientes | cierre LEAD únicamente (review/commit/firma) — verify docs ✅ |
| % completado | 95% (contenido ✅; cierre formal LEAD) |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — N/A código: doc-only. Las decisiones de confianza/cuarentena de diseño provienen de MGR-12/13 (threat model write-time ya cerrado); este ADR no crea superficie nueva.
- [x] **PERFORMANCE** — N/A: cero hot paths tocados. (Nota de diseño: D4c documenta el cambio de pesos en eviction como delta observable; recalibración = VER-08.)

## Steps
### Step 1: DISCOVERY (insumos + reconciliación + código citado)
- **Archivos:** 3 research-docs + template ADR + ADR-045/044 + api/{VERSIONING,COMPATIBILITY,DEPRECATIONS} + release-plz + CHANGELOG + plan/Backlog + código citado
- **Acción:** lectura completa; verify de deps (3 Cierres MGR ✅); identificación de divergencias campo×spec; verificación directa de citas de los 6 pendientes
- **Verify:** cada claim con `archivo:línea`; pre-req Backlog:938 ✅
- **Estado:** ✅ DONE (2026-09-28)

### Step 2: ADR-0046 (diseño consolidado + 6 pendientes)
- **Archivos:** `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md`
- **Acción:** D0 base ratificada; D1 corte/alcance/edges; D2 campos; D3 semántica; D4 confianza + tests V2/V4/V5; D5 cuarentena + deadline; D6 record↔nodo; D7 compat; D8 reconciliación; plan de migración (comandos); plan de implementación SCH-02..08; Owner sign-off con slots `[OWNER]`
- **Verify:** contrato cubierto por sección (mapeo en RESULTADO); 0 invenciones (citas `archivo:línea`)
- **Estado:** ✅ DONE (2026-09-28; status `proposed` — firma pendiente)

### Step 3: Task file + registro owner
- **Archivos:** `docs/dev/tasks/SCH-01.md`
- **Acción:** task file completo (Regla 0, contrato, invariantes, registro owner, steps)
- **Verify:** formato consistente con `MGR-13.md` (precedente)
- **Estado:** ✅ DONE (2026-09-28)

### Step 4: Verify mecánico docs
- **Archivos:** ambos nuevos
- **Acción:** `npx markdownlint-cli2` + `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1`
- **Verify:** lint 0 issues; coverage EXIT=0
- **Estado:** ✅ DONE (2026-09-28 — `npx markdownlint-cli2` → **0 issues / exit 0** en ambos archivos · `pwsh -NoProfile -File scripts/validate-docs-coverage.ps1` → **EXIT=0, 0 gaps** · `git status` = solo los 2 docs nuevos; `.github/workflows/perf-bench.yml` modificado es **WIP ajeno pre-existente**, no tocado)

### Step 5: Cierre (LEAD)
- **Acción:** review P2-01 (tier Fast, `docs/dev/**`) + commit `docs: SCH-01 — ...` + firma owner (`proposed→accepted`) + actualización del plan file
- **Estado:** ⬜ PENDING (LEAD — no self-review por instrucción)

## Dependencias
- **Consume:** MGR-10/12/13 (✅ cerrados) + 16 defaults ratificados + rails HARD-01.
- **Destraba:** SCH-02..08 (contrato verbatim) + SCH-09 (envelope v2/FIND). SIN firma del ADR no se toca `record.rs`.

## Review (GATE P2-01)
- **Revisor:** `vanta-review` fresco (sesión `ses_f166d8746ffeP5MvpNjZtbprjd`; ≠ autor `ses_f167eb2dcffe8Xxe9tsxqd1Phu`) — **✅ APPROVE** (2026-09-28). 0 Critical/Required; ~45 citas `archivo:línea` spot-checkeadas (todas resuelven) + tabla D8 verificada campo×campo + 2 claims externos (postcard 1.1.3). 2 Optional aplicados (nota SCH-02 acotada + boundary bulk declarado; `default_confidence` explícito) + 4 Nits (D5e en lista de firma — aplicado; dirección eviction matizada — aplicada; shell mix + registro — aceptados). Pre-req del owner sign-off satisfecho.

## Notas
- **No commit / no push / no self-review** (instrucción del orquestador; cierre en LEAD).
- 16 defaults del owner (2026-09-28) NO se re-abren — son §D0 del ADR.
- 6 pendientes P2-01 resueltos con decisión explícita (ver Investigation Notes + ADR §§ citadas).
- `src/` debe quedar intacto (`git diff --name-only` esperado: vacío fuera de docs).
- **WIP ajeno intacto:** `.github/workflows/perf-bench.yml` estaba modificado antes de esta ejecución (otra sesión); no tocado. `git status` de esta task = solo los 2 docs nuevos.
