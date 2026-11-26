---
title: "Backlog de extracción — Zep Cloud / Konig"
kind: research
status: active
description: "Candidatas del dossier Zep Cloud/Konig: contexto cross-scope, templates, observations, gobernanza, trazas, skills verificados y evaluación"
tags: [extraccion, backlog, zep, konig]
---

# Backlog de extracción — Zep Cloud / Konig

> **Origen:** informe `docs/dev/extraccion/ZepCloud-Konig/ZEP-CLOUD-KONIG.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.
> **Evidencia:** docs-only de Zep Cloud/Konig; los claims comerciales no son implementación verificada.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-ZEP-01 | 🔴 | AutoContextRank cross-scope con ledger de selección | `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/context_engine/engine.rs`, `src/core/retrieval` | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: union tipada de artefactos, fan-out por scopes, autorización, rerank global, presupuesto, `selection_rank`, `include_results` y `truncated`. Por qué: hoy retrieval y ensamblado no constituyen un ranking global auditable sobre facts/entities/observations/scenes/reflections/skills. Acción mínima: contrato `ContextObject` + planner read-only + ledger backwards-compatible. **Trazabilidad:** informe §1.2–1.4; docs `graph.get_context`, `include_results`, `selection_rank`. | Enriquece C7/C8; relacionado X-GRT-04 | ContextObject union; autorización por scope; presupuesto único |
| X-ZEP-02 | 🟠 | ContextTemplateStore con DSL segura | `vanta-memory/src/context_engine`, API/SDK surface | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: templates reutilizables/versionados con variables permitidas, límites por scope, filtros declarativos y render ledger. Por qué: el ensamblado actual no expone templates nombrados reutilizables con trazabilidad por variable. Acción mínima: parser de whitelist + store versionado + render read-only. **Trazabilidad:** informe §1.5; docs `context.create_template`, `%{edges limit=N}`. | Consume X-ZEP-01; converge C7 | ContextObject union; ACL por scope |
| X-ZEP-03 | 🟠 | RelativeTimeResolver auditable | `vanta-memory/src/core/retrieval`, `context_engine` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: resolver expresiones relativas a ventanas `{start_ms,end_ms,timezone}` con confidence y disclosure. Por qué: los filtros temporales existen pero el caller debe calcularlos. Acción mínima: componente puro que devuelve ventana/no-window y metadatos; integración opcional en auto-context. **Trazabilidad:** informe §1.4; docs relative time en `graph.get_context`. | Consume filtros temporales existentes; enriquece X-GRT-01 | Timezone por scope/proyecto/default UTC; precedencia de filtro explícito |
| X-ZEP-04 | 🔴 | ObservationArtifact cross-entity derivado | `vanta-memory/src/core/record`, derivados/persona/scenes/reflections | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: artefacto derivado read-only para patrones duraderos cross-entity con evidencia, dedup/merge y supersession. Por qué: falta una capa entre fact granular y narrativa/persona/reflection. Acción mínima: schema derivado + `derived_from` + publicación condicionada; sin escritura directa. **Trazabilidad:** informe §3/§7; docs `observations`, `steering-observations`. | Enriquece C5/C6; alimenta X-ZEP-01 | Governance de derivados; provenance; ABAC/metadata si aplica |
| X-ZEP-05 | 🔴 | TraceStore y AttemptFamilyGrouper | `vanta-memory/src/core`, `src/sdk`, MCP/tools | 🔴 2-4sem | 🔴 P0 | ⬜ Candidata | Qué: registrar ejecuciones como Trace/TraceEvent con task family, attempt family, outcome, verification, retries, close/reopen e idempotency. Por qué: skills verificados requieren evidencia de ejecución, no solo transcripts. Acción mínima: append-only trace schema + close/commit + outcome verification. **Trazabilidad:** informe §3.1–3.5; docs `capture-traces`, `trace events`. | Base de X-ZEP-06/07/08; puede enriquecer C9/C10 | Definir alcance embedded vs host runtime; retention propia |
| X-ZEP-06 | 🔴 | Verified Skill Compiler desde trazas | `vanta-memory/src/core/skill/skill_extractor.rs`, SkillStore | 🔴 2-4sem | 🔴 P0 | ⬜ Candidata | Qué: compilar trazas exitosas en SkillCandidate con citas de TraceEvent por paso y diff contra versión previa. Por qué: extractor actual usa transcripts; el delta es outcome verificado + attempt families + evidencia por paso. Acción mínima: compiler read-only que propone candidate + evidence map. **Trazabilidad:** informe §3.1–3.6; docs Agent Skills compile/curate. | Enriquece C10; depende X-ZEP-05; produce input para X-ZEP-07 | TraceStore; task family; verification strength |
| X-ZEP-07 | 🔴 | SkillAdmissionPolicy configurable | SkillStore, approval/capture surfaces, MCP tools | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: policy de admission con `compilation_mode`, `approval_mode`, allowlist de familias, tools que requieren revisión, mínimo de attempt families, evals, provisional limit/expiry y `expected_revision`. Por qué: un Skill es procedimiento/instrucción; no debe servirse solo porque fue generado. Acción mínima: candidate state machine + policy config + human approval path. **Trazabilidad:** informe §3.3; docs `learning-and-admission`. | Enriquece C10; consume X-ZEP-06; alimenta serving | SkillStore versionado; approval gateway; eval hooks |
| X-ZEP-08 | 🟠 | SkillUseLedger + TrustUpdater | SkillStore, `memory_reinforce`, retrieval/context assembly | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: ledger append-only de skill selected/rendered/followed/outcome con verification strength y trust explicable. Por qué: `memory_reinforce` no equivale a feedback por versión/procedimiento. Acción mínima: eventos append-only + agregado read-only de trust. **Trazabilidad:** informe §3.5; docs retrieve skills/use/outcome. | Enriquece X-ZEP-01 ranking; depende X-ZEP-05/07 | Trace/outcome schema; anti self-amplification |
| X-ZEP-09 | 🔴 | EpisodeMetadataProjection | `vanta-memory/src/core/record`, importers, provenance/derived artifacts | 🟠 1-2sem | 🔴 P0 | ⬜ Candidata | Qué: contrato de allowlist/clasificación para proyectar metadata de episodios a artefactos derivados con lineage y auditoría. Por qué: ABAC por clase de datos, retention y content policies dependen de que el derivado conserve atributos autorizados. Acción mínima: projection map read-only + auditoría de pérdida. **Trazabilidad:** informe §2.3; docs `episode-metadata-projection`. | Habilita X-ZEP-10/11/12; enriquece C4/C5 | Provenance/derived links existentes; clasificación de metadata |
| X-ZEP-10 | 🔴 | ABACPolicyEngine con report_only | namespace ACL, retrieval, context assembly, export/purge | 🔴 2-4sem | 🟠 P1 | ⬜ Candidata | Qué: policy sets versionados `off/report_only/enforce`, allow/deny con deny precedence, selectors por acción/dato/grafo y attach a principals. Por qué: VantaDB tiene permisos, no un motor ABAC declarativo con shadow evaluation. Acción mínima: evaluator puro + decision receipt + modo report_only. **Trazabilidad:** informe §2.1–2.2; docs `policy-based-access-control`. | Depende X-ZEP-09 para data-class rules; protege X-ZEP-01/02 | Metadata projection; principal model; audit sink |
| X-ZEP-11 | 🟠 | ContentPolicyEngine de derivación | capture/ingest, extraction, derived artifact publication | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: políticas declarativas por grafo/scope que deciden drop/flag/quarantine para categorías de información derivada, con auditoría de artefactos descartados. Por qué: quarantine actual no equivale a reglas versionadas de derivación. Acción mínima: policy evaluator + evento `policy_decision` + dry-run. **Trazabilidad:** informe §2.4; docs `content-policies`. | Depende X-ZEP-09; protege X-ZEP-04/06 | Metadata projection; derived artifact lifecycle |
| X-ZEP-12 | 🟠 | RetentionPolicyPlanner | lifecycle, purge/TTL, snapshots/WAL, provenance | 🟠 1-2sem | 🟠 P1 | ⬜ Candidata | Qué: planner declarativo de retención por grafo/objeto/artefacto con dependency check y receipts. Por qué: TTL/purge no expresan políticas versionadas ni preservan dependencias de evidencia. Acción mínima: plan read-only → ejecución → receipt. **Trazabilidad:** informe §2.5; docs `context-retention`. | Depende X-ZEP-09/provenance; relacionado X-GRT-03 | Lifecycle existente; receipts; snapshot/WAL semantics |
| X-ZEP-13 | 🟠 | SmartQuery iterative retrieval | GraphRAG/IQL, retrieval planner, context engine | 🔴 2-4sem | 🟠 P1 | ⬜ Candidata | Qué: recuperación multi-hop con `fast/deep`, máximo de rondas/records, citas, `answer_status` y ledger por hop. Por qué: GraphRAG actual no expone planner iterativo con refusal/parcialidad. Acción mínima: planner controlado que llama retrieval autorizado y devuelve traza. **Trazabilidad:** informe §1.8; docs `graph.query`. | Consume X-ZEP-01/10; enriquece GraphRAG | Retrieval autorizado; budget duro; citation contract |
| X-ZEP-14 | 🟡 | EvalContextSufficiency | `evals/`, `benchmarks`, `docs/user/operations/BENCHMARKS.md` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: métrica primaria de suficiencia de contexto y diagnóstico por etapa source/extraction/retrieval/render/use. Por qué: benchmarks actuales no separan evidencia disponible de respuesta juzgada. Acción mínima: añadir métricas/report schema al harness existente. **Trazabilidad:** informe §5.3; docs Zep eval harness/evaluate. | Enriquece C7/C8 y benchmarks existentes | ContextObject/trace ledger deseable, no bloqueante |
| X-ZEP-15 | 🟢 | GraphDirectory/Clone para scopes lógicos | namespaces, export/import, benchmarks | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: registro de grafos lógicos/scopes con descriptor y clone para fixtures, testing y migración. Por qué: facilita pruebas de políticas, benchmarks y blue/green sin tocar producción. Acción mínima: descriptor + clone/export de namespace. **Trazabilidad:** informe §4.2/§7; docs Zep graphs/clone concept. | Apoya X-ZEP-10/11/14 | Namespace lifecycle; export/import |
| X-ZEP-16 | 🟢 | Trace imports externos | importers, TraceStore | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: importar trazas desde Braintrust/LangSmith u otros harnesses al schema Trace. Por qué: permite aprendizaje/eval sin capturar runtime propio. Acción mínima: adapter a `Trace` normalizado. **Trazabilidad:** informe §3; docs import-traces Braintrust/LangSmith. | Depende X-ZEP-05 | TraceStore; contrato de outcome/verification |
| X-ZEP-17 | 🟢 | Context MCP/UserGroup governance | MCP server, auth, principals | 🔴 2-4sem | 🟡 P2 | ⬜ Candidata | Qué: grupos de usuarios/IdP para Context MCP con policies adjuntas y default-deny. Por qué: Zep gobierna usuarios MCP como security subject distinto de API keys. Acción mínima: principal model + group attachment + policy evaluation. **Trazabilidad:** informe §2.1–2.2; docs `usergroup-access`, Context MCP auth. | Depende X-ZEP-10 | MCP auth principal model; PolicyEngine |

## Fichas completas

> Las fichas expanden la tabla resumen. Los contratos son **hipotéticos** y deben adaptarse a tipos existentes. Las rutas son seams orientativas; requieren validación contra HEAD.

### X-ZEP-01 — AutoContextRank cross-scope con ledger de selección

- **Problema:** VantaDB tiene múltiples superficies de contexto, pero no un ranking global auditable que decida entre facts, entities, observations, scenes, reflections, skills y summaries bajo un mismo presupuesto.
- **Contrato hipotético:** `enum ContextObjectKind { Fact, EntitySummary, Observation, Scene, Reflection, Skill, EpisodeRef, ThreadSummary }`; `struct ContextCandidate { id, kind, score_evidence, source_refs, policy_state }`; `struct ContextBlockResult { context: String, truncated: bool, results: Vec<SelectedContextObject> }`; `selection_rank` por item.
- **Acciones:** definir union tipada; fan-out por scopes; autorización antes de ranking; normalización de evidencia; packing por presupuesto; ledger de inclusión/descarte.
- **FMEA:** scores incommensurables; leakage entre scopes; recall-over-precision excesivo; selección no explicable. Mitigar con evidencia estructurada, policy pre-rank y ledger obligatorio.
- **Criterios propuestos:** una llamada produce bloque y resultados tipados; cada item declara scope/rank/fuente; descarte por budget/policy explícito; episodios pueden ser referencia pero no item primario si se adopta la semántica Zep.

### X-ZEP-02 — ContextTemplateStore con DSL segura

- **Problema:** la composición de contexto no es una configuración reutilizable, versionable y auditable.
- **Contrato hipotético:** `ContextTemplate { name, revision, slots: Vec<TemplateSlot> }`; `TemplateSlot { variable, scope, limit, filters }`; `render(template_uuid, query, budget)`.
- **Acciones:** whitelist de variables; parser seguro; store con CAS; render ledger; limits por scope; provider-agnostic output.
- **FMEA:** exfiltración por slot; ambigüedad de variables; injection; templates que ocultan descartes. Mitigar con ACL por scope y render report.
- **Criterios propuestos:** nombre único; update con `expected_revision`; render declara variables efectivas y truncamiento; ningún slot evalúa código.

### X-ZEP-03 — RelativeTimeResolver auditable

- **Problema:** queries como “ayer” requieren conversión a ventana; el caller no debería calcular timezone ni inferir silenciosamente.
- **Contrato hipotético:** `fn resolve_relative_time(query, principal_tz, project_tz, now_ms) -> TemporalResolution`, con `NoConstraint|Window{start,end,timezone,confidence}`.
- **Acciones:** resolver puro; umbral de confianza; precedencia de filtros explícitos; disclosure en ContextBlock; fallback normal.
- **FMEA:** timezone incorrecta; expresión ambigua; silent filtering. Mitigar con confidence, report y override.
- **Criterios propuestos:** si `confidence < threshold`, no filtra; si filtra, declara UTC interval y timezone; explicit datetime siempre gana.

### X-ZEP-04 — ObservationArtifact cross-entity derivado

- **Problema:** no hay artefacto derivado que represente patrones duraderos cross-entity sin convertirlos en facts ni en narrativa de alto nivel.
- **Contrato hipotético:** `Observation { id, kind, summary, derived_from, confidence, valid_window, superseded_by?, readonly: true }`.
- **Acciones:** schema; dedup/merge; publication condicionada; steering por tipo/instrucciones; supersession; inclusión opcional en AutoContextRank.
- **FMEA:** generalización falsa; fuga cross-entity; convertir inferencia en verdad. Mitigar con evidencia mínima, provenance, ABAC y etiqueta derived.
- **Criterios propuestos:** no editable directamente; cada claim cita soporte; superseded no se sirve como vigente; policy puede bloquear publicación.

### X-ZEP-05 — TraceStore y AttemptFamilyGrouper

- **Problema:** no existe un registro de ejecución suficientemente estructurado para aprender procedimientos verificados.
- **Contrato hipotético:** `Trace`, `TraceEvent`, `AttemptFamily`, `Outcome`, `Verification`, `TaskFamily`; append events, close/reopen, retry, idempotency key.
- **Acciones:** schema append-only; event ordering/sequence gaps; outcome verification; `learn_from`; retention separada; export/import futuro.
- **FMEA:** telemetría sensible; trazas incompletas; aprendizaje de datos no autorizados. Mitigar con `learn_from` opt-in, retention y autorización.
- **Criterios propuestos:** append idempotente; close calcula outcome; reintentos agrupados sin duplicar familia; eventos citables por SkillCompiler.

### X-ZEP-06 — Verified Skill Compiler desde trazas

- **Problema:** el extractor actual aprende desde transcripts, no necesariamente desde ejecuciones verificadas con outcome.
- **Contrato hipotético:** `SkillCandidate { task_family, use_when, do_not_use_when, procedure, tools, environments, evidence_map, base_revision }`.
- **Acciones:** eligibility por task/attempt family; compilar pasos; bind evidence por paso; diff contra versión; emitir proposal no servible.
- **FMEA:** role-capture; casualidad; unsafe workflow; evidencia insuficiente. Mitigar con mínimo de familias, verificación fuerte y rejection classes.
- **Criterios propuestos:** todo paso tiene cita; candidate nunca entra a serving; diff muestra cambios; `rejection_class` explicado.

### X-ZEP-07 — SkillAdmissionPolicy configurable

- **Problema:** un Skill compilado necesita admission antes de ser servible, especialmente si usa tools con side effects.
- **Contrato hipotético:** `SkillAdmissionPolicy { compilation_mode, approval_mode, auto_approval_task_families, approval_required_tools, min_attempt_families, required_evals, provisional_limit, provisional_expiry_days, expected_revision }`.
- **Acciones:** policy store; evaluator; approval queue; eval hooks; provisional expiry; CAS.
- **FMEA:** auto-admisión de skill peligroso; feedback corrupto; versión obsoleta aprobada. Mitigar con deny-by-default para tools críticas, evals y revision check.
- **Criterios propuestos:** proposal no searchable hasta admitted; dangerous tool implica human review; auto-approval solo familias permitidas; todas las decisiones auditadas.

### X-ZEP-08 — SkillUseLedger + TrustUpdater

- **Problema:** feedback de uso de un Skill debe diferenciarse de reinforcement genérico de memorias.
- **Contrato hipotético:** eventos `SkillSelected`, `SkillRendered`, `SkillFollowed`, `SkillOutcome`, `SkillRejected`, `SkillSuperseded`; `TrustSnapshot` agregado.
- **Acciones:** append-only ledger; separación self-report/verified; trust por task_family/environment/agent_version/tool; decay.
- **FMEA:** autoamplificación; feedback malicioso; mezclar éxito parcial con verificado. Mitigar con verification strength y pesos explícitos.
- **Criterios propuestos:** outcome enlaza versión exacta; trust es explicable; un evento no cambia versión; trust alimenta ranking sin alterar evidencia.

### X-ZEP-09 — EpisodeMetadataProjection

- **Problema:** metadata de fuentes no tiene un contrato formal de propagación a artefactos derivados.
- **Contrato hipotético:** `ProjectionRule { source_key, target_scopes, classification, transform, audit }`; `ProjectedMetadata { lineage, policy_tags }`.
- **Acciones:** allowlist; clasificación; projection map; auditoría de pérdida; filtros/policy consumption.
- **FMEA:** propagar secretos; perder atributos necesarios para ABAC; transformación no auditable.
- **Criterios propuestos:** solo allowlisted se propaga; lineage indica origen; pérdida genera warning/audit; consumers reciben tipos consistentes.

### X-ZEP-10 — ABACPolicyEngine con report_only

- **Problema:** permisos por namespace no equivalen a policy sets declarativos con shadow evaluation.
- **Contrato hipotético:** `PolicySet { revision, rules }`; `Rule { actions, effect, selectors }`; `mode=off|report_only|enforce`; `DecisionReceipt`.
- **Acciones:** evaluator puro; deny precedence; attach a principals; report-only; audit; selectors por acción/dato/grafo.
- **FMEA:** bypass por derivados sin metadata; falsos deny; última regla ambigua. Mitigar con projection, deny wins y receipts.
- **Criterios propuestos:** misma decisión en SDK/MCP/export; `report_only` no bloquea pero registra; deny explícito gana; policy revision visible.

### X-ZEP-11 — ContentPolicyEngine de derivación

- **Problema:** no existe una capa declarativa que decida qué información derivada puede persistir.
- **Contrato hipotético:** `ContentPolicy { revision, categories, rules, scope }`; `PolicyDecision { Allow, Drop, Flag, Quarantine }`.
- **Acciones:** policy evaluator; decision audit; dropped-artifact log; dry-run; human review.
- **FMEA:** censura silenciosa; pérdida de evidencia; replay inconsistente. Mitigar con revision, receipts y revisión.
- **Criterios propuestos:** drop produce artefacto de decisión; flag no equivale a delete; re-evaluation usa misma revisión salvo override explícito.

### X-ZEP-12 — RetentionPolicyPlanner

- **Problema:** TTL/purge no expresan retención por grafo/objeto ni dependencias de provenance.
- **Contrato hipotético:** `RetentionPolicy { selectors, idle_period, exclusions }`; `RetentionPlan { eligible, blocked_by_provenance, actions }`; `RetentionReceipt`.
- **Acciones:** planner read-only; dependency check; ejecución por etapas; receipt; coordinación con snapshots/WAL.
- **FMEA:** borrar fuente que sostiene derivado; confundir retención lógica con purga física; backups inconsistentes.
- **Criterios propuestos:** plan precede ejecución; dependencias bloquean/marcan; receipts distinguen logical delete, purge y pending backup expiry.

### X-ZEP-13 — SmartQuery iterative retrieval

- **Problema:** algunas queries requieren multi-hop: recuperar, aprender, reformular y volver a buscar.
- **Contrato hipotético:** `SmartQueryRequest { query, mode: fast|deep, max_rounds, max_records, generate_answer }`; `SmartQueryResponse { results, answer_status, citations, plan_trace }`.
- **Acciones:** planner acotado; cada hop usa retrieval autorizado; citations; `answered|partial|refused`; ledger interno.
- **FMEA:** policy bypass por hop; latencia explosiva; respuesta sin evidencia. Mitigar con auth por hop, presupuestos y refusal.
- **Criterios propuestos:** ningún hop escapa ACL; `partial/refused` conserva records; plan_trace lista cada búsqueda; max rounds/records obligatorios.

### X-ZEP-14 — EvalContextSufficiency

- **Problema:** una respuesta correcta puede ocultar retrieval deficiente y una incorrecta puede provenir del lector, no de memoria.
- **Contrato hipotético:** métricas `source_present`, `graph_extracted`, `retrieved`, `selected`, `rendered`, `used`, `answer_correct`.
- **Acciones:** extender harness existente; config snapshots; chunk-set reutilizable; report por categoría.
- **FMEA:** judge LLM inconsistente; overfit a preguntas; métricas no comparables. Mitigar con checks deterministas y report de configuración.
- **Criterios propuestos:** context sufficiency se reporta separado de answer score; cada fallo tiene categoría; runs comparables por config snapshot.

### X-ZEP-15 — GraphDirectory/Clone para scopes lógicos

- **Problema:** no hay una abstracción de grafo lógico clonable para fixtures, migraciones o pruebas de policies.
- **Contrato hipotético:** `GraphDescriptor { id, namespaces, ontology/config, metadata }`; `clone_graph(source, destination, mode)`.
- **Acciones:** descriptor; clone/export; cleanup; fixtures de benchmark.
- **FMEA:** clonar datos sensibles; divergencia de índices; clones persistentes no auditados.
- **Criterios propuestos:** clone respeta políticas; descriptor versiona configuración; clone tiene owner/scope y cleanup explícito.

### X-ZEP-16 — Trace imports externos

- **Problema:** trazas existentes en herramientas de observabilidad no alimentan el TraceStore.
- **Contrato hipotético:** `TraceImport { provider, external_ids, mapping, expected_revision }`; adapter a `Trace/TraceEvent`.
- **Acciones:** adapter Braintrust/LangSmith; normalización de outcome/verification; dedup por external ID.
- **FMEA:** mapping pierde outcome o tool calls; import duplica familias; datos sensibles entran con `learn_from=true`.
- **Criterios propuestos:** import idempotente; external ID preservado; `learn_from` conservador; errores de mapping reportados.

### X-ZEP-17 — Context MCP/UserGroup governance

- **Problema:** usuarios MCP no son lo mismo que API keys de agentes; Zep los trata como principals agrupados.
- **Contrato hipotético:** `UserGroup { members, policy_sets }`; `ContextMcpPrincipal { user, group, idp_claims }`.
- **Acciones:** principal model; group attach; default-deny; evaluación de policies; audit.
- **FMEA:** grupo demasiado amplio; IdP claims inseguros; MCP omite policy check.
- **Criterios propuestos:** todo MCP request tiene principal; user hereda solo policies del grupo; default-deny; denial registrado.

## Prioridad sugerida

```text
P0: X-ZEP-01, X-ZEP-04, X-ZEP-05, X-ZEP-06, X-ZEP-07, X-ZEP-09
P1: X-ZEP-02, X-ZEP-03, X-ZEP-08, X-ZEP-10, X-ZEP-11, X-ZEP-12, X-ZEP-13
P2: X-ZEP-14, X-ZEP-15, X-ZEP-16, X-ZEP-17
```

## No objetivos

- No portar código de Zep Cloud/Konig.
- No convertir claims de latencia/escalabilidad en requisitos.
- No implementar multi-tenant SaaS, multi-AZ WAL ni BYOC en el core embedded.
- No permitir que Skills compilados se sirvan sin admission.
- No usar benchmarks del vendor como prueba de superioridad sin reproducción propia.
