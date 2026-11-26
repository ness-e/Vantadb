---
title: "Zep Cloud / Konig: contexto gobernado, retrieval, skills verificados y candidatos para VantaDB"
kind: research
status: proposed
description: "Estudio docs-only de Zep Cloud y Konig para extraer contratos, pipelines y funcionalidades candidatas no cubiertas por VantaDB"
tags: [research, zep, konig, memory, governance, skills]
---

# Zep Cloud / Konig: contexto gobernado, retrieval, skills verificados y candidatos para VantaDB

## Índice

- [0. Identidad, evidencia y límites](#0-identidad-evidencia-y-límites)
- [1. Retrieval y Context Block](#1-retrieval-y-context-block)
- [2. Governance y seguridad](#2-governance-y-seguridad)
- [3. Agent Skills y admission](#3-agent-skills-y-admission)
- [4. Konig](#4-konig)
- [5. Benchmarks, claims y límites](#5-benchmarks-claims-y-límites)
- [6. Estado real en VantaDB](#6-estado-real-en-vantadb)
- [7. Gaps investigables](#7-gaps-investigables)
- [8. Orden lógico y dependencias](#8-orden-lógico-y-dependencias)
- [9. Riesgos estratégicos](#9-riesgos-estratégicos)
- [10. Dictamen final](#10-dictamen-final)

## 0. Identidad, evidencia y límites

| Campo | Evidencia |
|---|---|
| Producto analizado | Zep Cloud / Context Lake / Context Graphs / Agent Skills |
| Data plane | Konig, descrito por Zep como servicio propietario de Context Graphs |
| Código del producto | Cerrado; no hay extracción de implementación |
| Evidencia primaria | `help.getzep.com/v4` y páginas públicas de producto |
| Evidencia complementaria | Repo público `getzep/zep` con ejemplos, `zep-ingest`, eval harness y benchmarks |
| Fecha de consulta | 2026-11-20 |
| Estado del análisis | Estático, docs-only; no se ejecutaron servicios, tests ni benchmarks |
| Relación previa | `getzep/graphiti` ya fue auditado en la campaña `agent-memory-2026-11-07` |

**Etiquetas de evidencia:**

- **Documentado:** contrato/API/límite publicado por Zep.
- **Vendor claim:** afirmación comercial no verificable desde código.
- **Inferencia:** traducción conceptual a VantaDB.
- **Candidata:** posible mecanismo; no está aprobado ni implementado.

**Dictamen:** Zep no aporta principalmente “otro vector DB”. Su valor es una **arquitectura operativa de contexto gobernado**: recuperación cross-scope, Context Block materializado, governance declarativa, observations derivadas, skills aprendidos desde trazas verificadas y admission control.

La extracción útil para VantaDB es de **contratos y pipelines**, no de código literal.

---

## 1. Retrieval y Context Block

### 1.1 Dos niveles de recuperación

Zep separa:

- **Búsqueda por scope:** `search_edges`, `search_nodes`, `search_episodes`, `search_observations`, `search_thread_summaries`.
- **Auto Search / Context Block:** una llamada de alto nivel que consulta varios scopes, rerankea globalmente y materializa un bloque listo para prompt.

### 1.2 Auto Search

La API documentada `graph.get_context` acepta:

| Parámetro | Función |
|---|---|
| `graph_uuid` | Grafo de usuario o Context Graph compartido |
| `query` | Texto de búsqueda |
| `max_characters` | Presupuesto del bloque; default `2500`, máximo `50000` |
| `include_results` | Devuelve también los artefactos seleccionados |
| `filters` | Filtros equivalentes a los search scopes |
| `template_uuid` | Template reutilizable para renderizar el bloque |

La respuesta contiene:

- `context`: string materializado;
- `truncated`: si el presupuesto limitó la salida;
- `results`: arrays tipados de `edges`, `nodes`, `episodes`, `observations`, `thread_summaries`;
- `selection_rank`: ranking global cross-scope asignado a cada item.

**Detalle documentado:** Auto Search no selecciona episodios como resultado primario; los episodios son evidencia fuente. La salida selecciona artefactos derivados o summaries.

### 1.3 Pipeline aprovechable

```text
query
  -> fan-out a scopes heterogéneos
  -> resultados tipados
  -> autorización/políticas
  -> rerank global cross-scope
  -> selección por presupuesto
  -> materialización en Context Block
  -> ledger opcional de items seleccionados
```

**Valor para VantaDB:** resolver la elección manual entre memorias, persona, scenes, reflections, skills, graph results y contexto ensamblado. El gap no es “tener búsqueda”; es tener un ranking global auditable sobre artefactos heterogéneos.

Esto solapa con las candidatas consolidadas C7/C8, pero Zep aporta un contrato concreto: `selection_rank`, `include_results`, `truncated` y Context Block materializado.

### 1.4 Relative-time retrieval

Zep interpreta expresiones como “ayer”, “la semana pasada” o “este mes” como posibles restricciones temporales:

1. Decide si la frase expresa una restricción temporal.
2. Si hay confianza suficiente, resuelve una ventana calendario.
3. Usa la timezone del usuario/grafo; si falta, la del proyecto; si falta, UTC.
4. Restringe automáticamente los facts.
5. Declara en el bloque el intervalo UTC y timezone usados.
6. Si la confianza no basta, ejecuta búsqueda normal.
7. Un filtro temporal explícito del caller tiene precedencia.

**Valor para VantaDB:** los filtros temporales existen (`as_of_ms`, ventanas de validez, supersession), pero el caller debe calcularlos. El gap es un **resolver de tiempo relativo auditable**, no el filtro temporal.

### 1.5 Context Templates

Zep permite templates reutilizables para controlar formato y contenido:

```text
# CUSTOMER PROFILE
%{user_summary}

# FACTS
%{edges limit=10}

# KEY ENTITIES
%{entities limit=5 types=[person,organization]}
```

Observaciones documentadas:

- `%{observations}` y `%{thread_summaries}` solo aceptan `limit`;
- esos scopes están limitados a 20 items por render;
- el nombre del template es único por proyecto;
- el template es configuración reutilizable, no prompt inline.

**Valor para VantaDB:** no se encontró evidencia de un store formal de templates de contexto con variables tipadas, límites por scope y UUIDs reutilizables.

Una implementación segura exigiría:

- whitelist de variables;
- límites por scope;
- filtros declarativos;
- sin evaluación arbitraria;
- respeto de ACL antes de materializar;
- ledger de qué variables produjeron qué contenido;
- templates versionados.

### 1.6 Advanced Context Block

Zep permite construir manualmente el bloque combinando facts con rangos `valid_at`/`invalid_at`, entity summaries, episodios, thread summaries, scopes, filtros estructurales, metadata y contenido propio.

Punto crítico documentado: los facts deben indicar si siguen vigentes o ya no son válidos. Esto evita que una preferencia histórica se trate como actual.

**Valor para VantaDB:** estandarizar un Context Renderer que siempre anote:

- vigencia temporal;
- fuente;
- estado de supersession;
- confianza;
- tipo de artefacto;
- razón de inclusión.

Encaja con C7/C8, pero Zep aporta un contrato concreto de presentación semántica.

### 1.7 Rerankers y filtros

Rerankers documentados:

| Reranker | Función |
|---|---|
| `RRF` | Fusiona ranking semántico + BM25 |
| `MMR` | Balance relevancia/diversidad con `mmr_lambda` |
| `cross_encoder` | Reevalúa query+result con modelo conjunto |
| `episode_mentions` | Prioriza resultados mencionados por episodios dados |
| `node_distance` | Prioriza proximidad estructural |
| `reranker_score` | Permite umbrales sobre score |

Filtros documentados:

- labels/tipos de nodo;
- tipos de edge;
- exclusiones;
- `connected_node_uuids`;
- `source_node_uuids`;
- `target_node_uuids`;
- `episode_uuids`;
- property filters con `eq`, `ne`, `gt`, `lt`, `gte`, `lte`, `in`, `is_null`, `is_not_null`;
- filtros datetime;
- BFS desde hasta cinco nodos/episodios.

**Gap para VantaDB:** no es “tener reranking”; ya existen RRF/MMR. Los deltas son filtros estructurales explícitos, reranking por evidencia/episodio, distancia de grafo y composición por artefacto.

### 1.8 Smart Query / `graph.query`

Zep describe `graph.query` como API experimental para recuperación multi-hop:

- el agente no sabe inicialmente qué buscar;
- Zep ejecuta varias rondas de búsqueda/traversal dentro de una llamada;
- `fast` limita a aproximadamente tres rondas;
- `deep` usa más capacidad/modelo de decisión;
- la respuesta puede tener hasta 300 records;
- `answer_status`: `answered`, `partial`, `refused`;
- `results` conserva records citados aunque la respuesta sea parcial o rechazada.

**Valor para VantaDB:** GraphRAG/IQL no equivalen a una capa iterativa con presupuesto de rondas, estado de respuesta y citas. Debe implementarse con autorización por hop, presupuestos duros, ledger interno y `refused` sin evidencia suficiente.

---

## 2. Governance y seguridad

### 2.1 Separación RBAC / ABAC

Zep separa:

- **RBAC:** humanos y administración de cuenta/proyecto.
- **ABAC:** agents, API keys y UserGroups; controla qué contexto puede leer/modificar cada principal.

La separación es correcta. VantaDB ya tiene ACL/RBAC y trust boundaries; el delta es un modelo de **policy sets reutilizables y versionados**.

### 2.2 Policy sets

Un policy set de Zep contiene reglas con:

- lista no vacía de acciones;
- `effect: allow|deny`;
- constraints opcionales;
- alcance por acción;
- alcance por clase de dato;
- alcance por grafo.

Modos documentados:

- `off`;
- `report_only`;
- `enforce`.

Cuando varias reglas coinciden, **deny gana**.

| Principal | Attach | Base posture |
|---|---|---|
| API key de agente | Policies en la key | `default_allow` o `default_deny` |
| UserGroup / Context MCP user | Policies en grupo | Default-deny |

Las reglas de clase de datos evalúan metadata de episodios proyectada sobre nodos, edges, communities y summaries.

**Valor para VantaDB:** motor ABAC con policy sets nombrados, versionado, shadow/report-only, deny precedence, selectors de grafo, reglas por clase de dato y attach a principals no humanos.

### 2.3 Episode metadata projection

Zep permite asociar metadata al episodio y proyectarla a nodos, edges, summaries, communities y artefactos derivados.

Habilita filtros, ABAC por clase de datos, auditoría de origen, retention policies y trazabilidad de respuesta.

**Gap:** definir un projection contract:

```text
episode.metadata.allowlist
  -> proyectar a artefactos derivados
  -> mantener lineage
  -> evaluar políticas
  -> auditar pérdida o transformación
```

No toda metadata debe propagarse automáticamente. Propagar campos sensibles puede crear fuga lateral.

### 2.4 Content policies

Zep permite definir categorías de información derivada que no deben entrar al Context Graph.

La idea no es solo sanitizar salida, sino gobernar derivación:

- qué tipo de fact puede persistirse;
- qué artefacto derivado se descarta;
- qué episodio queda flagged;
- qué evento de policy se audita.

**Valor para VantaDB:** existe quarantine, trust, capture approval y sanitización; no se encontró una capa declarativa de content derivation policies.

Una política de contenido puede volverse censura silenciosa si no tiene revision, decisión auditada, razón de descarte y revisión humana.

### 2.5 Retention policies

Zep separa retención de contexto dentro del grafo, retention por grafo/objeto, backups, write log, API/audit logs y archivos con retenciones diferentes.

**Gap para VantaDB:** un Retention Policy Planner declarativo:

```text
policy selector
  -> objetos elegibles
  -> dependencias/provenance
  -> plan de borrado
  -> ejecución
  -> receipt/audit
```

No debe mezclarse con autorización. Retention decide cuánto vive un artefacto; ABAC decide quién puede verlo mientras vive.

Riesgo crítico: borrar un episodio fuente invalida la trazabilidad de facts derivados. Deben preservarse referencias mínimas, marcar provenance perdido y coordinar snapshots/WAL/backups.

### 2.6 Memory security doctrine

La doctrina documentada:

> Memory is evidence for a decision. Memory is not an instruction, policy, credential, permission, or authorization to act.

Consecuencias:

- el Context Block no debe ir al system/developer message;
- debe entrar por el canal de datos no confiables del provider;
- la autorización de herramientas vive fuera del modelo;
- el contenido recuperado puede contener datos adversarios;
- conviene colocarlo después del historial y antes de la request reciente para preservar cache;
- deben incluirse los últimos mensajes crudos porque la ingesta puede ir retrasada.

**Valor para VantaDB:** formalizar un contrato de placement:

- `context_block.trust = untrusted_evidence`;
- `placement_hint = provider_specific_user/tool_result`;
- nunca `system`, `developer`, `policy`, `permission`;
- todo artifact con `source_refs`;
- el host conserva el ledger para auditar decisiones.

---

## 3. Agent Skills y admission

### 3.1 Ciclo completo

Zep Agent Skills no es “guardar instrucciones”. Es un ciclo:

```text
task run
  -> Trace
  -> verificación/outcome
  -> task family
  -> attempt families
  -> compilación de Skill candidate
  -> curación
  -> evaluación/admission
  -> versión SKILL.md
  -> serving por MCP/SDK
  -> registro de uso/outcome
  -> trust/rank actualizado
  -> nueva evidencia para compilación futura
```

Un **Trace** contiene objetivo, task family, eventos ordenados, resultado, outcome verificado, retries y metadata.

Un **Skill** contiene `use_when`, `do_not_use_when`, `procedure`, pasos con `instruction`, `decision`, `expect`, `tools`, `environments`, `agent_versions` y citas a eventos de trazas.

### 3.2 Relación con VantaDB

VantaDB ya tiene:

- extracción transcript → candidates con medidas anti role-capture;
- SkillStore versionado;
- versiones inmutables;
- optimistic locking;
- `memory_reinforce`;
- capture approval;
- dream consolidation/promoción controlada.

Referencias:

- `vanta-memory/src/core/skill/skill_extractor.rs`
- `docs/api/EMBEDDED_SDK.md`
- `docs/dev/extraccion/transversal/CONSOLIDATED.md`

Gap específico:

> No se encontró evidencia de un ciclo completo basado en trazas verificadas, attempt families, admission gates por herramienta, citas de evidencia por sección del skill y feedback de uso/outcome que actualice trust y ranking.

### 3.3 Configuración de admission

Campos documentados o equivalentes:

| Configuración | Función |
|---|---|
| `compilation_mode` | `scheduled`, `on_close`, `manual` |
| `approval` | `auto` o `manual` |
| `auto_approval_task_families` | Familias que pueden aprobarse automáticamente |
| `approval_required_tools` | Tools que obligan revisión humana |
| `approval_min_attempt_families` | Evidencia mínima por familia |
| `provisional_skill_limit` | Máximo de skills provisionales |
| `provisional_expiry_days` | Expiración si no se confirman |
| `admission_evaluation_requirements` | Evals obligatorias antes de admitir |
| `retention_days` | Cuánto conservar runs/traces |
| `expected_revision` | CAS/optimistic lock |

La compilación programada exige un mínimo de attempt families elegibles en una ventana. Solo trazas con `learn_from=true`, éxito y verificación fuerte deberían alimentar aprendizaje automático.

Cada learning run reporta `decision` (`produced`, `no_change`, `rejected`, `failed`), `stage` y `rejection_class`.

### 3.4 Arquitectura implementable

```text
TraceStore
  -> AttemptFamilyGrouper
  -> EligibilityFilter
  -> SkillCompiler
  -> EvidenceBinder
  -> AdmissionEvaluator
  -> ApprovalGateway
  -> VersionedSkillStore
  -> SkillUseLedger
  -> TrustUpdater
```

### 3.5 Skill evidence

Cada paso compilado debe citar los Trace events que lo respaldan. Esto evita que un Skill sea solo texto plausible.

Cada sección debería responder:

- qué trazas la generaron;
- qué outcome tenían;
- qué tool calls ocurrieron;
- qué evidencia apoya cada paso;
- qué versión anterior se modificó.

### 3.6 Skill use/outcome

Zep separa servir un Skill de registrar su uso:

- se busca/lee el skill aplicable;
- el agente decide si seguirlo;
- la app registra uso;
- luego registra outcome;
- ese outcome alimenta trust, ranking y futura compilación.

Ledger sugerido:

```text
skill_id
version
task_family
agent
use_context
followed/not_followed
outcome
verification_strength
```

Debe ser append-only y distinguir seleccionado, renderizado, seguido, verificado y auto-reportado.

### 3.7 Publish/share

Zep permite compartir Skills a otro agent dentro del mismo security domain; no comparte automáticamente.

Para VantaDB:

- `SkillPublication`;
- `SkillRelation`;
- source skill/version;
- destination agent/scope;
- policy de copia;
- lineage;
- permisos de lectura/modificación.

No debe haber propagación implícita cross-agent.

---

## 4. Konig

### 4.1 Claims de vendor

Zep describe Konig como data plane propietario para millones de grafos pequeños y sparse, con:

- bi-temporal edges;
- WAL append-only replicado;
- hot/warm/cold tiering;
- adjacency/CSR in-memory;
- vector + BM25 + graph traversal + pattern matching;
- provenance/ABAC/retention en el data layer;
- rehidratación desde object storage;
- latencias declaradas sub-200ms;
- escalabilidad de 1K a millones de grafos.

**Clasificación:** vendor claims. No son hechos verificables desde código.

### 4.2 Valor conceptual para VantaDB

#### Governance en el data layer

Políticas, provenance y retención deben evaluarse cerca del dato, no solo en wrappers API. `PolicyEngine` debería influir en index visibility, retrieval, materialización, export, purge y publication de derivados.

#### Unified Retrieval Plane

VantaDB ya tiene vector, BM25 y GraphRAG. El valor es un plan unificado donde cada candidato produzca evidencia comparable:

- score semántico;
- score lexical;
- soporte estructural;
- evidencia temporal;
- provenance;
- permisos;
- coste.

#### Per-graph locality

Millones de grafos pequeños equivalen conceptualmente a muchos namespaces/scopes aislados. Lo útil para VantaDB:

- estadísticas por namespace;
- índices compactos por grafo lógico;
- offload/rehydration por scope;
- políticas por grafo;
- clones/test graphs.

### 4.3 Qué no copiar

No se recomienda perseguir directamente:

- multi-AZ replicated WAL como feature core;
- hot/warm/cold object-storage tiering SaaS;
- millones de tenants como objetivo principal;
- claims de latencia como requisito;
- BYOK/BYOM/BYOC en el mismo sentido de Zep.

VantaDB es embedded/local-first. Si se implementa tiering, debe responder a necesidad local de memoria/disco, no a copia arquitectónica.

---

## 5. Benchmarks, claims y límites

### 5.1 Claims publicados

Zep publicita resultados como:

- LoCoMo aproximadamente 94–95%;
- LongMemEval aproximadamente 90%;
- retrieval p95 alrededor de 155–162ms;
- reducción de contexto con Auto Search;
- resultados en Tau3/AppWorld/EnterpriseOps-Gym/ALFWorld para Agent Skills.

### 5.2 Evaluación crítica

| Aspecto | Evaluación |
|---|---|
| Reproducibilidad | Parcial; harness público pero servicio cerrado |
| Independencia | Débil; resultados publicados por el vendor |
| Comparabilidad | Baja; depende de split, judge, context budget y modelo lector |
| Alcance | Buena para retrieval single-shot; limitada para comportamiento agentico real |
| Valor para VantaDB | Alto como metodología; bajo como prueba de superioridad |

### 5.3 Metodología aprovechable

1. **Context sufficiency como métrica primaria:** separar evidencia disponible de respuesta correcta.
2. **Decoupled user/document graphs:** comparar configuraciones sin reingerir.
3. **Config snapshots por run:** registrar configuración exacta de ingesta/retrieval.
4. **Chunk-set reutilizable:** misma entrada, distintas configuraciones.
5. **Diagnóstico por categoría:**
   - no está en el source;
   - está en source, pero no entró al graph;
   - está en graph, pero no fue recuperado;
   - fue recuperado, pero no renderizado;
   - fue renderizado, pero el modelo no lo usó.

VantaDB ya tiene harnesses; la brecha es metodológica.

---

## 6. Estado real en VantaDB

### 6.1 Ya implementado o sustancialmente cubierto

No conviene crear backlog nuevo para:

- hybrid retrieval BM25 + vector + RRF;
- filtros por namespace, metadata, temporalidad, confianza, quarantine;
- GraphRAG/traversals;
- context assembly con budgets;
- persona/scenes/reflections;
- trusted/tainted boundaries;
- capture approval;
- audit de inyección;
- certified delete/write receipts;
- Skill extraction;
- SkillStore versionado;
- outcome reinforcement;
- LongMemEval/LoCoMo;
- importador Zep/Graphiti.

Referencias:

- `docs/dev/extraccion/transversal/CONSOLIDATED.md`
- `docs/dev/extraccion/ZepCloud-Konig/ZEP-CLOUD-KONIG.md`
- `docs/dev/extraccion/graphiti/BACKLOG.md`
- `docs/api/MEMORY_INTERCHANGE_FORMAT.md`
- `docs/api/EMBEDDED_SDK.md`

### 6.2 Ya planeado o candidato

Zep debe enriquecer, no duplicar:

| Candidato VantaDB | Zep aporta |
|---|---|
| C4 context identity/revisions | Skill/template/policy revisions y CAS |
| C5 raw + versioned derivatives | Observations, facts y source projection |
| C6 conditional publication | Admission, skill visibility, policy-bound publication |
| C7 retrieval coverage/discard/budget | Auto Search cross-scope + `selection_rank` |
| C8 Selected/Rendered/Delivered | `include_results`, selected items, provenance |
| C9 ingest lifecycle observability | ingestion traces/debug runs |
| C10 approvable experiences | Skill admission/evaluation pipeline |
| X-GRT-01 temporal fact resolution | bi-temporal edges/fact intervals |
| X-GRT-02 dedup identity gating | dedup/merge de entities/facts/observations |
| X-GRT-03 source retraction | source traceability e invalidación |
| X-GRT-04 recall tracing | per-arm recall y query audit |
| X-GRT-05 Studio inspector | graph/visualizer y debug de retrieval |

---

## 7. Gaps investigables

Las siguientes candidatas son investigables, no backlog aprobado. Requieren revalidación formal contra HEAD.

### P0

- **X-ZEP-01 — AutoContextRank cross-scope:** union tipada de artefactos, fan-out, autorización, rerank global, `selection_rank`, `include_results`, `truncated`, presupuesto y ledger.
- **X-ZEP-02 — ContextTemplateStore:** templates versionados con DSL segura, límites y render ledger.
- **X-ZEP-03 — RelativeTimeResolver:** resolver tiempo relativo con confianza, timezone y disclosure.
- **X-ZEP-04 — ObservationArtifact:** artefacto derivado cross-entity, read-only, con evidencia, dedup/merge y supersession.
- **X-ZEP-05 — TraceStore y AttemptFamilyGrouper:** trazas de ejecución, eventos, outcomes, verificación y retries.
- **X-ZEP-06 — Verified Skill Compiler:** compilar trazas exitosas con evidencia por paso.
- **X-ZEP-07 — SkillAdmissionPolicy:** admission configurable antes de servir skills.
- **X-ZEP-08 — SkillUseLedger + TrustUpdater:** ledger de uso/outcome y trust explicable.

### P1

- **X-ZEP-09 — EpisodeMetadataProjection:** propagación controlada de metadata a derivados.
- **X-ZEP-10 — ABACPolicyEngine:** policy sets `off/report_only/enforce`, deny precedence y selectors.
- **X-ZEP-11 — ContentPolicyEngine:** gobierno de información derivada permitida.
- **X-ZEP-12 — RetentionPolicyPlanner:** retención declarativa con dependencia de provenance.
- **X-ZEP-13 — SmartQuery:** recuperación iterativa con rondas, citas y `answer_status`.
- **X-ZEP-14 — EvalContextSufficiency:** métrica de suficiencia y diagnóstico por etapa.

### P2

- **X-ZEP-15 — GraphDirectory/Clone:** grafos lógicos, clones y fixtures.
- **X-ZEP-16 — Trace imports:** importación desde Braintrust/LangSmith u otros harnesses.
- **X-ZEP-17 — Context MCP/UserGroup governance:** grupos/IdP para acceso MCP, fuera del core embedded.

---

## 8. Orden lógico y dependencias

Orden recomendado de investigación/especificación:

```text
1. ContextObject union + SourceTrace contract
2. AutoContextRank + selection ledger
3. ContextTemplateStore
4. EpisodeMetadataProjection
5. PolicyEngine report_only
6. ContentPolicyEngine + RetentionPolicyPlanner
7. TraceStore
8. Verified Skill Compiler
9. SkillAdmissionPolicy
10. SkillUseLedger/TrustUpdater
11. SmartQuery
12. EvalContextSufficiency avanzada
```

Dependencias:

- ABAC depende de metadata projection.
- Skills verificados dependen de TraceStore.
- Observations dependen de governance de artefactos derivados.
- Context templates dependen de union tipada de contexto.
- SmartQuery depende de retrieval autorizado y ledger.
- Retention depende de provenance y dependencias.

---

## 9. Riesgos estratégicos

| Riesgo | Problema | Mitigación |
|---|---|---|
| Copiar plataforma SaaS | Zep resuelve multi-tenant, IdP, billing, BYOC | Extraer contratos/pipelines, no topología cloud |
| Taxonomía ambigua | persona/scenes/reflections/skills/facts pueden solaparse con observations | Definir frontera semántica estricta |
| Auto-aprendizaje inseguro | Skill es más cercano a instrucción que a fact | Admission, tool gates, citations, expiry, rollback |
| Políticas que mienten | ABAC/content/retention fallan si metadata/provenance se pierde | `report_only`, decision receipts, completeness checks |
| Benchmarks engañosos | Claims vendor-run no son targets reproducibles | Adoptar metodología, no números |

Taxonomía propuesta:

- **Episode:** evidencia cruda.
- **Fact:** claim granular.
- **Entity:** actor/objeto con resumen.
- **Observation:** patrón derivado cross-entity.
- **Scene:** narrativa episódica.
- **Reflection/dream:** consolidación de alto nivel.
- **Skill:** procedimiento.
- **ContextBlock:** materialización temporal, no memoria.

---

## 10. Dictamen final

La prioridad real para VantaDB es:

1. **AutoContextRank + ledger de selección**
2. **TraceStore + Verified Skill Compiler**
3. **SkillAdmissionPolicy**
4. **EpisodeMetadataProjection + PolicyEngine report_only**
5. **ObservationArtifact**
6. **RetentionPolicyPlanner**
7. **ContextTemplateStore**
8. **SmartQuery**
9. **EvalContextSufficiency**

La investigación debería producir dos documentos separados si se continúa:

- `zep-cloud-platform.md`: dossier docs-only de producto.
- `konig.md`: dossier conceptual del data plane.

No se ejecutó implementación ni tests. Este documento es análisis estático de documentación pública y contraste con el estado/candidatos documentados en VantaDB.

## Apéndice — Incógnitas, lagunas de evidencia y falsos positivos descartados

> Consolidación transversal del patrón «Apéndice E» del estudio OpenViking.
> Esta sección no añade análisis nuevo: reúne lo que el propio informe ya
> declara como no acreditado o pendiente y los callejones ya descartados,
> para que una re-auditoría no repita trabajo. Las secciones originales
> citadas siguen siendo la fuente detallada.

### Incógnitas y lagunas de evidencia

- No se ejecutó implementación ni tests.

### Falsos positivos y descartes registrados

- No se registraron falsos positivos formales en el cierre de este informe.

### Limitaciones declaradas en este informe

- § 0. Identidad, evidencia y límites
- § 5. Benchmarks, claims y límites
