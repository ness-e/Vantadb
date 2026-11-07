---
title: "Letta: memoria versionada, publicación de contexto y límites del clon"
kind: research
status: proposed
description: "Clon landing de Letta y lectura complementaria fijada de Letta Code; contratos Rust sin usar el servidor V1 retirado"
tags: [research, memory, engineering]
---

# Letta: memoria versionada, publicación de contexto y límites del clon

## Índice

- [0. Identidad y arquitectura verificadas](#0-identidad-y-arquitectura-verificadas)
- [1. Recomendaciones](#1-recomendaciones)
- [2. Mecanismos y fichas Rust](#2-mecanismos-y-fichas-rust)
- [3. Incidencias públicas y seguridad](#3-incidencias-públicas-y-seguridad)
- [4. Contratos previos en VantaDB](#4-contratos-previos-en-vantadb)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks leídos y adaptación](#7-benchmarks-leídos-y-adaptación)
- [8. Matriz comparativa](#8-matriz-comparativa)
- [9. Negativos, hipótesis y falsos positivos Vanta](#9-negativos-hipótesis-y-falsos-positivos-vanta)
- [10. UI, escritorio y Studio](#10-ui-escritorio-y-studio)
- [11. Fuentes, cobertura y pendientes](#11-fuentes-cobertura-y-pendientes)

## 0. Identidad y arquitectura verificadas

### Corte y corrección de premisa

| Objeto | Evidencia congelada |
|---|---|
| Clon solicitado | `C:\Users\Eros\letta`, `letta-ai/letta`, `main`, `8816d3a8516893bfdc0fa14bfb1235226012e295` |
| Selección | Excepción expresa del owner: 25 377 estrellas en [manifest adicional](clone-additions-manifest.json); no supera el filtro original de 30k ni lo modifica |
| Qué contiene | `README.md`, `AGENTS.md`, políticas y Apache-2.0; **landing, no servidor implementado** [L0][L1] |
| Complemento remoto | `letta-ai/letta-code`, `9176d8c0b16f408e9240422a98379a423e2176c3`, observado por API el 2026-11-07; no nuevo clon/candidato ni cambio del manifest |
| Vanta | Snapshot contractual `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no se persigue HEAD concurrente |
| Método | [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [VANTA-REFERENCE](VANTA-REFERENCE.md), [METHOD](METHOD.md), [BENCHMARK-NOTES](BENCHMARK-NOTES.md), leídos sin modificarlos |

**Dictamen:** atribuir al clon las capacidades del antiguo MemGPT/servidor Python sería falso. Su README remite al harness actual Letta Code; `AGENTS.md` prohíbe usar V1/archive para comparaciones e integraciones actuales. **No se leyó archive, tags, paquetes Python retirados ni Docker V1.** La ampliación inspecciona únicamente fuente pública actual complementaria y la distingue del clon seleccionado. No acredita todo Letta Cloud.

Leyenda: **Código** = implementación estática fijada; **documentación** = declaración; **reporte** = incidencia upstream no reproducida; **propuesta** = contrato futuro, no API implementada. Fecha de consulta externa: 2026-11-07. No ejecución, instalaciones, servicios ni operaciones Git de escritura.

### Mapa, stack, superficies y licencias

| Ruta en Letta Code | Responsabilidad y lectura | Límite |
|---|---|---|
| `package.json` [L2] | `@letta-ai/letta-code` 0.34.44, TypeScript ESM, Bun 1.3.14; React 18.2, Ink 5.2.1, ws, MCP SDK 1.32.2, Letta Agent SDK 0.8.38, Letta client `^1.10.2`, pi-ai, node-pty | Dependencias no auditadas transitivamente |
| `src/agent/memory-filesystem.ts` [L3] | Scoping, checkout local/remoto, árbol y flags MemFS | No motor vectorial ni garantía SaaS |
| `src/agent/memory-git.ts` [L4] | Identidad de autor, commit de paths, auth no interactiva, sync/rebase y estado post-turno | Commit local, push y compilación son fronteras diferentes |
| `src/backend/local/system-prompt-compilation.ts` [L5] | Memoria desde archivos **committed**, revisión y hash del prompt | No prueba de entrega efectiva al proveedor |
| `src/memory-constraints.ts`, `src/agent/memory-format.ts` [L6][L7] | Layout v1/v2, índices `MEMORY.md`, límites y read-only | Política debe provenir de fuente confiable |
| `src/agent/memory-operation.ts`, `src/utils/file-lock.ts` [L8][L9] | Exclusión de checkout con identidad de proceso y release condicionado | Lock de filesystem, no lease distribuido universal |
| `src/agent/memory-conflict-repair.ts` [L10] | Firma de conflicto y claim de intento con token | No prueba que un LLM resuelva bien conflictos |
| `src/backend/local/local-store.ts` [L11] | Agentes, conversaciones, transcript, prompt compilado y persistencia local | No confundir checkpoint de ejecución con recuerdo |
| `src/backend/local/compaction.ts` [L12] | Planificación, summary, presión y fallos irreducibles | Estimación de tokens, no tokenización exacta acreditada |
| `src/cli/components`, scanner [L13][L14] | TUI e inspección de memoria | No código completo del cliente desktop/web |

Licencias: Apache-2.0 en el clon y en `letta-code/LICENSE` del complemento [L15]; no se deduce que los servicios comerciales o todos los binarios/clientes externos compartan licencia. Antes de adoptar material derivado revisar notices por archivo, dependencias y distribución. Traducir TS a Rust no elimina obligaciones.

### Modelos y arquitectura

```text
Agente (identidad persistente) ── varias conversaciones (historia de ejecución)
       │
       ├─ MemFS por agente / repositorios adjuntos
       │     archivos editados → validación → commit Git → sync opcional
       │                                   │ revisión committed
       └─ compilación de contexto ← core + índices/proyección
                         │
                 turno + tools + proveedor
                         └─ transcript / compactación / estado de conversación
```

`LocalCompiledSystemPrompt` lleva `content`, `coreMemory`, `compiledAt`, `rawSystemHash`, `memfsRevision` [L5:23-30]. No es un vector ni una memoria semántica. `LocalStore` mantiene mapas de conversaciones/mensajes y prompts compilados; `persistAgent`, `persistConversationState`, transcript y prompt usan escrituras separadas [L11:2745-2759,2824-2870,2964,3035,3126-3145]. La muestra **no prueba una transacción atómica conjunta ni recuperación tras corte eléctrico**.

`detectMemoryFormat` usa presencia de `MEMORY.md`: v2 considera core los `.md` raíz; v1, `system/`. La proyección v2 excluye skills y exige índices en directorios ascendentes [L7:6-40]. Es selección jerárquica por archivos, no ANN/recall-all. `getDefaultMemoryBlocks` carga `persona/human`, conserva flags read-only y cachea plantillas [L16:16-105]; no confundir estos bloques de compatibilidad con todo el flujo MemFS.

### Context blocks, recall y archivo: tres superficies, no tres motores

La documentación **actual** [D1][D2] describe entradas iniciales `{label,value}` que se convierten en Markdown, memoria inline y archivos que se leen a demanda. El pin complementario tiene dos layouts: `initialMemoryFilesFromCreateBody` crea agentes locales en **v2**, agrega el bloque raíz si falta, aplana `/` a `_`, valida paths y rechaza colisiones sin distinguir mayúsculas [L18:9-38,78-115]. Por tanto, `system/` en la guía SDK no debe presentarse como layout único: difiere de la inicialización local v2 leída. Es una divergencia documental localizada, no evidencia de fallo de persistencia.

| Concepto | Modelo/API acreditado | Semántica y límite |
|---|---|---|
| Bloque de contexto | Entrada inicial `label/value/description`; `memory_blocks` en body interno [L18]; `LocalCompiledSystemPrompt` [L5] | Texto fijado al prompt según layout, editable/versionable; no colección de embeddings |
| Recall de mensajes | `searchMessagesForBackend`, body con query, mode, agent/conversation, fechas, hidden y limit [L19][L20:18-27] | Local: transcripts JSONL; remoto: delegación HTTP. Buscar no modifica memoria ni confirma que el resultado fue entregado |
| Recall de conversaciones | `searchConversationsForBackend`, resultado `{embedded_text,conversation,rrf_score}` [L21:8-20,44-115] | Local busca términos en summaries de las conversaciones recientes listadas, no en todo el contenido; `rrf_score` local es fracción de términos, **no RRF** |
| Archivo de experiencias | Transcripts `conversation.json/manifest.json/messages.jsonl`, conservados fuera de la ventana activa [L20:423-544]; guía de recall [L22] | Archivo durable de conversación no equivale a un índice semántico de documentos externos |
| “Archival memory” como producto/API | No acreditada como colección separada en el harness actual inspeccionado | No importar antiguos endpoints, embeddings, CRUD de passages ni garantías del servidor retirado |

La consulta `guides/agents/memory/` redirigió a una página rotulada **V1 SDK (legacy)** [D4]. Se detectó la redirección y se excluyó como evidencia comparativa actual; **no se inspeccionó la rama `archive` ni se diseñó una integración V1**. `archive` como rama retirada y archivo conceptual de conversaciones no son lo mismo. La petición de cubrir archive no habilita atribuir capacidades retiradas al producto presente.

**API realmente observada:** [L23:1-19] implementa wrappers `POST /v1/messages/search`, `/v1/conversations/search` y `/_internal_search/cache-warm`. El prefijo de ruta `/v1` en el cliente actual no demuestra uso del servidor V1 retirado: es nomenclatura de transporte observada en el pin actual. No se auditó el servidor de esas rutas. Para integración nueva rige el Agent SDK/App Server [D3], no una llamada directa inventada a estos endpoints internos.

**Recall local trazado:** `searchMessagesForBackend` selecciona backend por capability y pasa el ID de conversación actual → `searchLocalTranscriptMessages` rechaza `vector/hybrid`, parsea términos/frases, filtra agente/conversación/hidden/fecha → recorre JSONL por filas y proyecta mensajes → ranking léxico → IDs para expansión [L19:19-44][L20:558-630]. No usa un LLM para indexar ni consulta embeddings en esa rama. `warmMessageSearchCacheForBackend` devuelve `local-backend-noop`, no un índice calentado [L19:47-59].

**Errores específicos:** JSON/config ilegible puede omitirse; transcript ilegible o malformado devuelve cero filas de esa conversación, mientras errores de proyección se propagan [L20:62-69,515-530]. Una fecha ausente/inválida atraviesa `dateInRange` [L20:328-346]. Son comportamientos de la muestra, no ACL ni validación estricta de tiempo. No existe una revisión snapshot conjunta acreditada entre todos los JSONL recorridos; mensajes añadidos durante la lectura requieren contrato del caller.

### Lifecycle de agent-state frente a memoria del motor

Según guía actual [D3], `createAgent` crea agente y conversación default; `createSession(agentId)` abre otra conversación; `resumeSession(agentId|conversationId)` reconecta; `send/stream` forman un turno. Persisten identidad, memoria e historia; conexiones MCP, callbacks de permisos, cwd y elección de máquina pertenecen a la sesión. `stateless:true` omite carga/cambio de memoria de esa sesión: **no vuelve efímeros agente ni conversación**. Es documentación, no certificación de crash recovery.

`abort` solicita cancelar el turno, pero se consume hasta resultado terminal; `close` libera recursos locales. Tras desconexión se reanuda con sesión nueva y se reconcilia historia: **no hay replay de eventos perdidos**, ni se debe reenviar a ciegas tras `send` exitoso. Un resultado `success` de turno no es un recibo de commit MemFS, sincronización ni borrado. En código, `LocalStore::deleteAgent` elimina registros/conversaciones, mientras el prompt compilado y MemFS son otras superficies [L11:482-520]. Vanta ofrece L0 y checkpoints de trabajo; no se equiparan a este runtime persistente ni se propone uno nuevo.

### Trazas concretas y fronteras de error

1. **Ingesta/escritura:** `resolveScopedMemoryDir` prioriza agente explícito/runtime antes de variables de entorno; `ensureLocalMemfsCheckout` inicializa repo local o clona/tira del remoto según capacidades [L3:108-138,280-305]. Edición ordinaria de archivo no implica commit. `commitMemoryWrite` normaliza pathspecs, prepara local/remoto, stage, commit y devuelve SHA; ante fallo del commit intenta unstage [L4:1180-1268]. El push ocurre por otra ruta.
2. **Recall/contexto:** `collectCommittedMemoryFiles` obtiene HEAD, lista `.md`, determina formato, lee `git show HEAD:path` y frontmatter. Otros nodos se representan sin cargar cuerpo en esa fase [L5:65-130]. Lectura desde HEAD evita publicar borradores, pero la secuencia de comandos debe analizarse bajo concurrencia antes de asumir snapshot único. No se midió recuperación semántica.
3. **Update:** nuevo commit cambia revisión; compilación guarda la revisión utilizada. `syncPendingMemoryCommitsAfterTurn` devuelve `conflict`, `dirty`, `skipped`, `clean`, `pushed` o `push_failed`; no auto-commitea archivos dirty. Non-fast-forward intenta pull/rebase, revalida y repush [L4:1833-1977]. Éxito de commit no significa sync remoto.
4. **Delete de memoria:** eliminar un archivo es una modificación que debe pasar validación y commit para cambiar el árbol publicado; protección read-only contempla creación, modificación **y eliminación** [L6:119-142]. Es una deducción del contrato de árbol/commit, no una llamada de borrado físico SaaS auditada.
5. **Delete de agente:** `LocalStore::deleteAgent` retira mapas y directorios de conversación y el JSON del agente [L11:482-522]. **No acredita** que elimine MemFS, repos adjuntos, Cloud, backups o copias entregadas al modelo. No inferir borrado transitivo por su nombre.
6. **Errores/cancelación:** `withMemoryOperation` libera en `finally`; lock comprueba AbortSignal al adquirir y no recupera un holder vivo solo por antigüedad [L8][L9]. Repair token evita que un worker viejo borre un claim nuevo [L10:212-234]. La cancelación de espera no equivale a cancelar un proveedor ya llamado.
7. **Compactación:** plan sliding preserva la cola con tool call pendiente; hay errores distintos de planificación/irreducible. El fallback de overflow completo conserva último turno y emite aviso de descarte, no inventa resumen [L12:43-205,738-820]. No es memoria de hechos ni checkpoint de tarea.

## 1. Recomendaciones

| Orden | Propuesta y decisión | Motivo específico |
|---|---|---|
| Primero | Publicación de contexto desde revisión aceptada, ficha A | Separar borrador, persistido, derivado y entregado |
| Primero | Validador de proyección y read-only, ficha B | Documento hostil no puede autoconcederse autoridad ni cuota |
| Después | Reparación por firma y claim, ficha C | No repetir indefinidamente el mismo conflicto ni publicar output viejo |
| Después | Compactación con progreso explícito, ficha D | Evitar bucles y tool results huérfanos, reutilizando assembler |
| Condicionado | Estado de sync en Studio, ficha E | Diagnosticar local-commit/remoto/compilado sin importar Git como base |
| Después | Recall con capacidad y cobertura efectivas, ficha F | No confundir local FTS, summaries y API remota bajo la misma etiqueta |
| Condicionado | Identificar ecos de búsquedas, ficha G | Evitar que un resultado recitado se use como fuente independiente |
| Rechazar | Adaptador V1, benchmarks V1 y segundo runtime | Fuente retirada y arquitectura redundante |

## 2. Mecanismos y fichas Rust

Todas las firmas siguientes son **propuestas no implementadas**; nombres descriptivos, no IDs oficiales. Se comparan con los seams Vanta fijados al final, no con el estado mutable de la sesión.

### A. Contexto publicado con revisión explícita

- **Problema agente:** una reflexión cambia preferencias mientras otra conversación prepara el siguiente turno; el modelo no debe recibir una mezcla de borradores y versiones.
- **Fuente/unidad:** recopilación committed + `memfsRevision/rawSystemHash` [L5:23-39,65-130]; separación commit/sync [L4:1230-1268,1833-1977].
- **Delta/alternativas:** Vanta ya tiene provenance, `IntegratedContext`, personas y assembler [V1][V2]. Añadir identidad del snapshot seleccionado; no sustituirlo por Git ni crear memoria paralela. Adapter Git optativo solo si un origen lo exige.
- **Contrato propuesto:** `assemble_published(request: AuthorizedContextRequest, revision: AcceptedRevision) -> Result<PublishedContext, PublishError>`; resultado `{source_refs, revisions, policy_revision, content_digest, compaction_report}`.
- **Invariantes/errores:** revisión fijada durante selección/render; revocación revalidada al entregar; `StaleRevision/Denied/SourceUnavailable/Cancelled` distintos de vacío. Digest no debe exponerse como prueba criptográfica de entrega.
- **Seam real:** `ContextEngine::assemble_with_recall`, `IntegratedContext`, `InjectionPolicy` [V1][V2]. Las revisiones deben provenir de registros existentes, no del nombre de sesión.
- **Plan:** 1) inventariar revisión fuente y contrato delivery; 2) propagar refs hasta assembler; 3) seleccionar contra revisión aceptada; 4) invalidar compilaciones al modificar/borrar; 5) exponer diagnóstico aditivo.
- **Persistencia/concurrencia/commit:** no mantener lock durante LLM; validar revisión al publicar. Guardar manifiesto mínimo solo si se aprueba, separado de payload; migración aditiva, entradas legacy = revisión desconocida, no inventada; sin reindexado para el DTO. Rollback vuelve al ensamblado anterior sin revivir borrados.
- **Dependencias/relaciones:** B define validación; E muestra el estado. Relación con ledger OpenViking y caché Supermemory: mismo contrato de revisión, no tres almacenes.
- **FMEA:** edición concurrente → mezcla → detectar mismatch y replanificar; caché vieja tras delete → fuga → invalidar/reautorizar; persistencia OK pero delivery falló → falso positivo → estados distintos.
- **Aceptación manual futura:** dos conversaciones, edición sin publicar, revisión nueva, revocación entre selección/entrega y fuente borrada; ninguna revisión inventada ni contenido en logs. Medir counts, revisión opaca y outcome.
- **Mínimo/no objetivos/abierto:** manifiesto de publicación; no Git obligatorio, sincronización Cloud ni exactly-once del proveedor. Resolver quién fija revisión aceptada.

### B. Validación de árbol/proyección antes de publicación

- **Problema agente:** un recuerdo masivo o un índice roto expulsa contexto útil; un agente edita una instrucción protegida para elevar su autoridad.
- **Fuente/unidad:** `validateMemoryTreeConstraints` recibe reader, trusted accepted tree y política; verifica índices ascendentes, archivos regulares y read-only [L6:21-41,46-155]; selección jerárquica [L7].
- **Delta/alternativas:** extender `ingest::merge::commit` y `is_structural`, que ya protegen archivos estructurales [V3]. Validador puro común para ingesta/UI; rechazar copiar hooks Git o aceptar una política enviada por el propio documento.
- **Contrato propuesto:** `validate_projection(before: AuthorizedTree, proposed: CandidateTree, policy: TrustedProjectionPolicy) -> Result<ProjectionPlan, ProjectionError>` con rutas canónicas, revisiones y violations tipadas.
- **Invariantes/errores:** política del host, no LLM; `ProtectedPath/MissingIndex/InvalidPath/BudgetExceeded/StaleBase`; límites en unidad declarada, no equivalencia chars=tokens; errores previos no deben quedar ocultos como nuevos éxitos.
- **Seam:** `normalize_wiki_path`, `is_structural`, `commit` [V3]; entrega posterior por [V1]. No endurecer cuota del motor a partir del tamaño de un prompt.
- **Plan:** 1) mapear políticas existentes; 2) validar árbol antes de merge; 3) registrar plan contra revisión base; 4) revalidar al persistir; 5) presentar errores en Studio. No cambiar formatos de memoria sin spec.
- **Persistencia/rollback:** política versionada aparte, validación sin escritura; reporte aditivo. Si cambia layout, doble lectura y regeneración de proyección con índices existentes; desactivar publicación nueva conserva último aceptado, sin borrar evidencia fuente.
- **Dependencias/relaciones:** A consume solo aprobado; no dependencia de C. Las rutas son autoridad de almacenamiento, no autorización por sí mismas.
- **FMEA:** path traversal/symlink → escape → canonicalización/control host; read-only definido en candidato → bypass → trusted policy; presupuesto parcial → falsa cabida → reporte de excedentes protegidos.
- **Aceptación manual futura:** índice padre faltante, borrado protegido, renombre, Unicode, archivo no regular, dos revisiones y documento que intenta cambiar política; detectar sin filtrar texto sensible en errores.
- **Mínimo/no objetivos/abierto:** validación determinista aditiva; no copiar límites numéricos Letta ni rediseñar wiki. Resolver tratamiento explícito de legado inválido.
- **Extensión específica leída:** inicialización [L18] añade una fase distinta de la validación de árbol: colisiones `a/b` frente a `a_b` y case-insensitive. Una spec B debe conservar identidad original y diagnosticar colisión antes de escribir; no trasladar el aplanado de labels a claves Vanta ni interpretar un label como path autorizado.

### C. Reparación limitada por firma, identidad y generación

- **Problema agente:** reparación automática repite el mismo conflicto o un worker tardío sobrescribe un recuerdo ya corregido.
- **Fuente/unidad:** firma de conflicto/estado inválido, `claimed/in_progress/attempted`, identidad PID+inicio y token [L10:119-234]; exclusión de checkout [L8][L9].
- **Delta/alternativas:** Vanta tiene scheduler, owner, reclaim y `TaskCheckpointManager` [V4][V5]; añadir identidad del problema/revisión a trabajos reparables, no otro scheduler. Preferir serialización existente a un lease distribuido sin necesidad.
- **Contrato propuesto:** `claim_repair(scope, source_revision, problem_digest, actor) -> Result<RepairClaim, RepairError>`; `publish_repair(claim_generation, expected_revision, candidate) -> RepairOutcome`.
- **Invariantes/errores:** un intento vigente por problema; `AlreadyAttempted/InProgress/StaleClaim/Conflict/Cancelled`; el LLM propone contenido, nunca autorización ni commit sin validación.
- **Seam/plan:** 1) inventariar `PipelineWorker::run_once/reclaim_stale`; 2) extender payload/checkpoint con firma; 3) ejecutar cálculo fuera de lock; 4) validar B y revisión al commit; 5) registrar resultado y habilitar reintento humano explícito.
- **Persistencia/concurrencia/rollback:** generación monotónica o escritor serial definido en spec; preservar checkpoint y estado de error. Migración aditiva, sin reindexar recuerdos intactos; deshabilitar reparación mantiene trabajos inspeccionables. Nunca copiar PID como identidad distribuida.
- **Dependencias/relaciones:** A/B para publicación; recibos/ownership Agentmemory describen la misma frontera transversal. No requiere adoptar Git para beneficiarse.
- **FMEA:** firma omite revisión → se suprime reparación válida → incluir revisión; holder pausado tratado como muerto → dos commits → fencing al publicar; token viejo libera nuevo → comparar generación.
- **Aceptación manual futura:** mismo conflicto dos veces, cambio de conflicto, reinicio del worker, cancelación, revisión editada durante LLM y resultado rechazado; outcomes/attempts sin prompt.
- **Mínimo/no objetivos/abierto:** reparación determinista con salida propuesta; no prometer resolver semántica ni fusionar tenants. Resolver reset humano de `AlreadyAttempted`.

### D. Compactación con progreso y unidades conversacionales

- **Problema agente:** resumen que no reduce contexto causa retry infinito; cortar a mitad de tool call deja resultados sin llamada o pierde la última petición.
- **Fuente/unidad:** `assertLocalCompactionProgress`, `planLocalSlidingWindowCompaction`, fallback completo y errores diferenciados [L12:43-205,738-820].
- **Delta/alternativas:** extender `CompactionReport`/assembler existentes [V1]. La ganancia buscada es estado irreducible y preservación de unidad, no otra implementación de summarizer ni nuevos prompts.
- **Contrato propuesto:** `plan_compaction(history_revision, budget, protected_units) -> Result<CompactionPlan, CompactionError>`; `apply(expected_revision, plan, summary) -> CompactionOutcome`, con `NoProgress/Irreducible/StaleHistory/Cancelled`.
- **Invariantes:** preservar relaciones call/result; no resumen fabricado ante fallo; estimación declarada; prefijo protegido puede exceder presupuesto según contrato Vanta, debe informarse y no ocultarse.
- **Seam/plan:** 1) mapear unidades actuales en `assemble_inner`; 2) identificar protección y progreso existente; 3) añadir outcomes solo donde falten; 4) revalidar historia al aplicar; 5) exponer reporte a host. Mantener proveedor optativo.
- **Persistencia/concurrencia/rollback:** plan puro, resumen fuera de lock, commit condicionado a revisión. Reporte aditivo; no purgar L0 para ahorrar contexto; no reindexado. Desactivar fallback nuevo conserva historial fuente.
- **Dependencias/relaciones:** A registra revisión entregada; distinto de C (reparar fuente) y de checkpoint de tarea.
- **FMEA:** token estimator subestima → overflow → reporte de residual y salida irreducible; resumen sin progreso → bucle → detección; edición concurrente → resumen obsoleto → mismatch y descarte.
- **Aceptación manual futura:** solo protegido, último tool pendiente, summary vacío, fallo proveedor, no progreso y revisión concurrente; counters/estimaciones sin mensajes.
- **Mínimo/no objetivos/abierto:** protección estructural y error honesto; no copiar el porcentaje upstream ni umbral nuevo, no garantizar exactitud semántica del resumen.

### E. Estado separado de guardado, sincronización y uso

- **Problema agente:** la UI dice “guardado” aunque el archivo siga dirty, el push falle o la conversación siga con contexto viejo.
- **Fuente/unidad:** outcomes post-turno [L4:1833-1977], revisión compilada [L5] y reporte #4831.
- **Delta/alternativas:** adaptar actividad/diagnóstico Studio existentes; vocabulario portable a orígenes no Git. No convertir sync remoto en requisito de almacenamiento embebido.
- **Contrato propuesto:** `source_status(auth, source_ref) -> Result<PublicationStatus, StatusError>` con `{source_revision, accepted_revision, projected_revision, delivered_revision?, sync_outcome?}`; `Unknown/Denied/Unavailable` explícitos.
- **Seam/plan:** 1) mapear `LayerTelemetry` y `load_assembled_context` [V4]; 2) sumar traducción por adapter; 3) publicar metadatos mínimos; 4) mostrar diferencias en ActivityPanel/MemoryLens; 5) asociar acciones autorizadas de reintento, no botones que ejecuten Git arbitrario.
- **Persistencia/concurrencia/rollback:** lectura revisionada; no derivar “entregado” de compilado. Estado aditivo, sin reindexado; retirar vista no altera recibos/tombstones. Refrescar si cambia revisión durante consulta.
- **Dependencias/relaciones:** A/C, y estados de procesamiento Supermemory; un único modelo de outcomes con traducciones, no DTO idéntico para mecanismos distintos.
- **FMEA:** merge de estados reduce fallo a clean → confianza falsa → mostrar cada dimensión; error contiene token Git → fuga → redacción y código tipado; acción reintento ajena → daño → reautorizar.
- **Aceptación manual futura:** dirty, commit local, conflicto, push fallido, proyectado viejo y delivery desconocido visibles por separado; no contenido ni secretos en telemetría.
- **Discriminación L-1:** fallo de autenticación del refresh padre y checkout
  ausente no son el mismo estado. Un fix de runner Git reportado no acredita
  sync universal; la UI conserva diagnóstico por fase y no aconseja recrear
  repos/credenciales para todo error. Caso adverso: checkout ausente frente a
  existente con fetch fallido, sin exponer tokens ni lanzar Git desde UI.
- **Mínimo/no objetivos/abierto:** estado y diagnóstico; no cliente desktop nuevo ni prueba de sync Cloud. Resolver retención de metadata.

### F. Contrato de recall que declara backend y cobertura

- **Caso agente:** pedir una decisión antigua y recibir “sin recuerdos” cuando solo se buscaron summaries recientes o un modo vectorial no estaba soportado.
- **Fuente SHA/unidad:** complemento `9176d8c0b16f408e9240422a98379a423e2176c3`, dispatch [L19:19-59], validación/filtrado [L20:558-630], búsqueda de summaries [L21:44-115]. Prompts [L22] son instrucciones, no implementación.
- **Delta/alternativas:** Vanta ya dispone de `RecallConfig::core_search`, scopes y motor híbrido [V2]. Extender resultado del caller con capacidad/ruta efectiva; no copiar BM25 ni usar un campo denominado RRF como score comparable. Adapter Letta solo si después se especifica SDK actual.
- **Contrato Rust propuesto, no existente:** `recall_with_coverage(auth, request, cancel) -> Result<CoveredRecall, RecallFailure>`; resultado `{refs, source_revisions, backend_kind, effective_mode, coverage, skipped_sources}`. `UnsupportedMode`, `UnreadableSource`, `Denied`, `Cancelled` y `UnknownRevision` no se convierten en vacío.
- **Invariantes/ownership/commit:** scope del principal, no variable de entorno como autoridad; resultado posee refs autorizadas, no lectores abiertos de archivos. No commit en consulta; versión fuente se revalida antes de entregar. Cancelación impide publicar un bundle incompleto como completo; partial se etiqueta sin revelar fuentes denegadas.
- **Seam/pasos:** 1) mapear `RecallResult/RecallConfig` en [V2]; 2) propagar ruta efectiva desde backend existente; 3) conservar errores de fuente por clase; 4) pasar cobertura a `assemble_with_recall` [V1]; 5) enseñar estado en E. No activar core_search universalmente por inferencia.
- **Persistencia/migración/rollback:** DTO aditivo, históricos Unknown; no reingesta ni reindexado. Desactivar diagnóstico conserva recall anterior, sin inventar retroactivamente cobertura.
- **Relaciones/dependencias:** A fija revisión de entrega, G marca ecos, E comunica degradación. ACL del transporte y cancelación del host previas; no depende de Git ni LLM.
- **FMEA:** modo solicitado no soportado → vacío aparente → detectar capability/UnsupportedMode; JSONL saltado → falsa exhaustividad → contadores de fuente fallida; summary-only → decisión ausente → coverage explícita y expansión autorizada.
- **Aceptación manual futura:** local FTS, modo no soportado, summary sin contenido, archivo corrupto, hidden, fecha inválida, revocación y cancelación; IDs/revisiones o Unknown honestos, logs sin query ni reasoning. No se ejecutó.
- **Mínimo/no objetivos/abierto:** metadata de ruta/cobertura, no universalizar un score ni crear archivo semántico. Resolver si el consumidor admite éxito parcial y cómo expresa un rango temporal desconocido.

### G. Provenance de eco en recall conversacional

- **Caso agente:** el resultado de una búsqueda anterior contiene la misma frase que el usuario pregunta ahora; recuperar esa recitación como evidencia independiente refuerza errores.
- **Fuente SHA/unidad:** [L20] mismo pin: `searchToolCallId/echoToolCallId:384-415`, asociación call/result `538-542`, pregunta activa `606-619` y `rankMatches:285-325`. Upstream atenúa estos ecos; no borra globalmente tool results.
- **Delta/alternativas:** Vanta ya conserva provenance y L0/L1 [V6][V7]; incorporar origen de recitación en selección, no dedup semántico adicional ni blacklist de mensajes de tools. Alternativa mínima: advertir eco sin modificar ranking hasta tener política aprobada.
- **Contrato Rust propuesto:** `classify_recall_origin(refs, authorized_turn_identity) -> OriginReport` con `Original/RecallEcho/CurrentQuery/Unknown`, refs/revisión y enlace call/result. Decisión de selección la conserva el caller; **no se importan pesos 0.1, boosts ni nueva fórmula**.
- **Invariantes/errores/ownership:** contenido idéntico no basta para clasificar eco; solo enlace de origen acreditado. `BrokenCallLink/StaleHistory/Denied/Cancelled` diferenciados. Reporte posee IDs opacos; no usa texto de reasoning en logs. Consulta sin escritura ni confirmación de memoria.
- **Seam/pasos:** 1) inventariar enlaces de tool calls en L0Recorder; 2) propagar referencias al record/recall existente [V6][V2]; 3) clasificar antes de assembler [V1]; 4) emitir motivos; 5) habilitar política solo tras spec. No añadir un parser de comandos shell como autoridad.
- **Migración/concurrencia/rollback:** metadata opcional, legacy Unknown; historia fijada por revisión o detección de cambio al entregar. No reindexado obligatorio; enriquecimiento solo si source refs disponibles. Desactivar política mantiene originales y diagnóstico; cancelación no deja clasificación persistente parcial.
- **Dependencias/relaciones:** F declara cobertura y A revisión; mismo problema transversal que dedup de presentación Supermemory, pero aquí importa **linaje**, no igualdad textual.
- **FMEA:** tool result legítimo clasificado eco → pérdida de evidencia → vínculo explícito y fallback Unknown; recap sin vínculo → circularidad → detectar provenance incompleta; pregunta activa vence a fuente histórica → falsa confirmación → distinguir CurrentQuery sin borrarla del historial.
- **Aceptación manual futura:** resultado recitado, herramienta no buscadora, misma frase de fuente independiente, resultado sin call, consulta explícita de conversación actual y append concurrente. Observar razones/counts, no contenido.
- **Mínimo/no objetivos/abierto:** clasificación explicable; no pesos, prompts nuevos ni garantía de eliminar toda circularidad. Resolver propagación de linaje cuando el host solo entrega texto.

## 3. Incidencias públicas y seguridad

Consulta 2026-11-07: DuckDuckGo vía `websearch.search` con `site:github.com/letta-ai/letta/issues memory block passage concurrency` no devolvió resultados; `metasearch.compare_engines` para `site:github.com/letta-ai/letta-code memory issues`, Bing/DDG, tampoco. Se continuó con API pública GitHub `search/issues` y lectura de los cuerpos. No se iniciaron navegadores ni servicios. Los estados siguientes son los observados, no garantías de vigencia futura.

| Incidencia leída | Estado observado, síntoma y causalidad | Código relacionado, alcance y discriminación futura |
|---|---|---|
| [#5091: Memory checkout clone-swap of large shared-memory repos races a ~3 min deadline and aborts $MEMORY_DIR commands](https://github.com/letta-ai/letta-code/issues/5091) | Abierta. Cuerpo reporta clone-swap/deadline; seguimiento 2026-10-19 matiza: `syncRepoMount` clona si falta mount y usa pull ff-only si existe; recuperación por fetch del cuerpo no cubre checkout ausente. Cifras y análisis son del reportante, no repro propia | [L4] y helper de checkout no auditado íntegro. Conservar diferencia ausente/existente y gates de acceso a paths; no elevar el diagnóstico inicial a causa universal. PR #5248 relacionado no prueba merge ni resolución completa |
| [#4831: Commit memory edits a turn leaves uncommitted (post-turn auto-commit)](https://github.com/letta-ai/letta-code/issues/4831) | Abierta, propuesta de producto: ediciones normales quedan dirty; menciona draft #4725 cerrado sin merge. Comentario leído de 2026-11-01 reporta reflexión `parent_dirty` visible solo en debug | [L4:1866-1875] efectivamente retorna dirty, no auto-commit. No se afirma que Vanta sufra pérdida por Git: Vanta no exige ese flujo. Validar que un ack de captura no se anuncie como contexto ya publicado. El comentario no prueba pérdida física |
| [#4832: Background conflict repair for attached shared-memory repositories](https://github.com/letta-ai/letta-code/issues/4832) | Abierta, solicitud de ampliar reparación a repos adjuntos; señala política más delicada al haber varios autores. No corrección confirmada | [L10] acredita repair de checkout con firma/token, no cobertura de todos los repos adjuntos. Evitar “Letta repara cualquier conflicto”; validar C por ámbito/autor |
| [#4249: Windows: Git Credential Manager pops up during memory sync push](https://github.com/letta-ai/letta-code/issues/4249) | Abierta. Reporte inicial Windows 11/0.31.11–0.31.12; comentario de `jibramnobhill` 2026-09-09 declara fix [#4277](https://github.com/letta-ai/letta-code/pull/4277) publicado en v0.31.14: refresh padre usaba runner Git plain sin token MemFS, proxy Desktop ni entorno no interactivo | El pin [L-worktree:286-293] usa `runMemoryGit` para fetch y pide token solo a origen MemFS. Comentario reporta regresión HTTP Windows y mutation control; no son tests propios ni prueba de abrir GCM real. Código y seguimiento acotan el defecto, no certifican toda auth/sync ni cierre universal |

**Fronteras de confianza:** scoping de filesystem no sustituye autenticación App Server; read-only depende del caller que suministra política confiable. Herramientas de archivo/shell y Git manejan contenido hostil: no dar a un recuerdo autoridad de sistema por estar en `system/`. La muestra de locks usa filesystem/proceso, no consenso entre máquinas. Los tokens de Git deben redactarse también en errores y procesos; se localizaron helpers dedicados [L4:126-178], no se auditó toda fuga posible. Ni commit ni `deleteAgent` garantizan borrado de historia Git/backup/cloud. No se declararon vulnerabilidades reproducidas.
Relectura pública correctiva 2026-11-07: #4249 y #5091 con seguimiento vía
`websearch.fetchWebContent`, respuestas completas; [L-worktree] raw al SHA
fijado. El runner local `runGit` (`memory-worktree.ts:29-58`) y el import
`runGit as runMemoryGit` no son intercambiables: el fetch de padre usa el
segundo. No se inspeccionó diff causal completo de #4277 ni artefacto v0.31.14.

## 4. Contratos previos en VantaDB

Prerrequisitos **por ficha**, no un bloqueo universal:

| Ficha | Qué existe en el snapshot | Verificación necesaria antes de una spec |
|---|---|---|
| A | `assemble_with_recall`, `IntegratedContext`, `InjectionPolicy` [V1][V2] | Cómo fijar revisión conjunta y distinguir selección de entrega real |
| B | Canonicalización y archivos estructurales protegidos en merge [V3] | Alcance de trusted policy en cada ingestor, autorización de publicación y legado |
| C | `PipelineWorker::with_owner/run_once/reclaim_stale`, dead letters; checkpoints [V4][V5] | Fencing efectivo en publicación del resultado, no solo reclamar trabajo |
| D | Compactación/reportes y prefijo protegido en assembler [V1] | Unidad call/result y no progreso en sus callers concretos |
| E | `LayerTelemetry`, contexto ensamblado cargable [V4] | Semántica de estados que transporta MCP/Studio; presencia de panel no acredita ese contrato |
| F | `RecallConfig`, `RecallResult`, `core_search` y scopes [V2] | Cobertura y modo efectivos por caller; fallo de fuente no debe simular exhaustividad |
| G | L0, provenance y enlaces de generación [V6][V7] | Disponibilidad real del linaje call/result; Unknown para clientes que solo entregan texto |

La ingesta Vanta ya protege archivos estructurales y asegura sources; su `commit` procesa páginas secuencialmente y puede retornar error tras escrituras previas [V3:239-288]. No asumir atomicidad de todo el lote. Las fichas no autorizan alterar esta política ni añadir scheduler antes de analizar el existente.

## 5. Qué no trasladar

- **V1/archive y sus adaptadores:** retirados explícitamente; no comparar Letta actual usando su servidor Python o Docker. No “reparar” un benchmark V1.
- **Git como base obligatoria:** útil para origen editable e historia, costoso como primario de registros embebidos, auth, rebase, locks y distribución. La unidad adoptable es revisión/publicación; Vanta ya tiene primario/WAL.
- **Auto-commit ciego al finalizar turno:** #4831 advierte que conversaciones concurrentes pueden incluir la edición ajena a medio hacer. Validar identidad de operación, no copiar un lifecycle hook.
- **Read-only definido por contenido:** el recuerdo no decide su propia política. Ni un prefijo ni una herramienta autorizan por sí solos.
- **Prompts, porcentajes y modelos de compactación:** no portables sin evaluación aprobada. Las cifras upstream descritas abajo no son nueva configuración Vanta.
- **UI completa/Cloud:** la fuente inspeccionada del harness no certifica clientes externos ni backend comercial; mantener proveedor LLM y red optativos.

## 6. Advertencias metodológicas

El clon main es una landing. Tener muchas estrellas, el nombre MemGPT o documentación histórica no prueba que conserve la implementación; este informe corrige esa premisa antes de recomendar. El complemento remoto se fijó por SHA, sin alterar inventarios ni inicializar otro checkout.

La lectura del compilador cubre adquisición de revisión/archivos y tipos, no toda generación final ni todos los puntos de recompilación del runtime. HEAD y `git show HEAD:path` son comandos separados: hay que estudiar el lock del caller para certificar snapshot consistente; no se eleva esa observación a carrera reproducida. Una función de validador puede aplicarse correctamente o ser omitida por un caller no leído. Los tests localizados no se ejecutaron ni se presentaron como aprobados.

La consulta de incidencias usa estado vivo independiente del SHA. Reproducciones del reportante, títulos de PR y flags de “closed” no prueban aplicación del fix al pin. No se auditó todo App Server, ACL multitenant, sandbox de herramientas, API web/desktop ni dependencias transitivas.

## 7. Benchmarks leídos y adaptación

### Runner actual localizado: latencia de CLI, no memoria QA

Fuente complementaria [L17], `scripts/latency-benchmark.ts:1-340`, leída estáticamente:

| Aspecto | Comportamiento real |
|---|---|
| Entrada y cadena | `main` parsea `--scenario/--iterations`, exige `LETTA_API_KEY`; `runBenchmark` hace spawn de `bun run dev` en headless, con `LETTA_DEBUG_TIMINGS=1` |
| Defaults | Una iteración; escenarios `fresh-agent`, `resume-agent`, `minimal-math`; todos usan preguntas aritméticas cortas escritas upstream y `--yolo --output-format json`; fresh añade `--new-agent` |
| Unidad/dataset/split | Una ejecución de CLI por escenario e iteración; tres entradas fijas, no corpus de sesiones con gold, no dev/holdout de calidad |
| Preparación y contaminación | Fresh crea agente; resume reutiliza el último. La muestra no resetea memoria ni define un warmup separado; reusar agente cambia condiciones |
| Métricas | `totalMs` desde antes del spawn hasta close; parser extrae milestones y duraciones HTTP de stderr. Suma duraciones API; imprime `totalMs - totalApiMs` como overhead |
| Denominador | Imprime cada resultado; no accuracy ni score de memoria, ni percentiles agregados de calidad. Conserva exitCode por ejecución |
| Fallos | Temporizador envía SIGTERM tras 120000 ms; close nulo se transforma en exit 1. No se acreditó manejo del evento spawn `error`; logs sin match dejan arrays vacíos, no latencia cero de memoria |
| Modelo/costes | No fija answerer/modelo, embeddings, tokenizer o juez en el script. Los toma del runtime/estado; coste no contabilizado. Credencial puede habilitar llamadas pagadas |
| Concurrencia | Bucles de iteraciones y escenarios secuenciales; HTTP interno puede solaparse, por eso suma API no es necesariamente porción disjunta de wall time |
| Efectos | Crea/modifica agentes y conversaciones, ejecuta herramientas en modo yolo y puede llamar red; no cleanup/borrado al finalizar acreditado en la lectura |

**Limitación discriminante:** si dos requests se solapan, restar su suma a wall time no mide CPU/overhead exclusivo y puede ser engañoso. No “arreglar” esa fórmula aquí. Fuente de defaults = upstream, no parámetros propuestos para Vanta. No se copian preguntas, prompts, rúbricas ni configs.

### Adaptación bajo el protocolo existente

- **Pista producto/latencia:** si se aprueba después, un adapter de host Vanta mapearía fases captura → persistencia → proyección → recall → ensamblado/entrega a eventos de tiempo ya definidos en [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks). Mantener outcomes de fallo, warm/cold explícitos y coste por fase. No equiparar resta de HTTP a latencia del motor.
- **Pistas memoria/retrieval/respuesta:** el runner leído no las evalúa. Usar los protocolos Vanta ya existentes para LoCoMo/LongMemEval, con source isolation y gold solo en evaluador. No inventar un score de Letta a partir de sus tres escenarios.
- **Integración con Letta actual:** una futura comparación de agente necesita Agent SDK y App Server actuales documentados por [L0]; no reutilizar adapter del servidor V1. Semántica y portabilidad del SDK no se certificaron aquí; requiere especificación y autorización posteriores.
- **Reset:** instancias desechables y consentimiento antes de cualquier eliminación; ninguna limpieza sobre identidad compartida. En esta tarea no se inicia, detiene, instala ni ejecuta nada.

Superficies buscadas: árbol remoto completo por rutas `eval/benchmark`; localizado el script de latencia y tests de compactación/memoria. No se leyó un harness actual de calidad LoCoMo/LongMemEval dentro de esta muestra. **No localizado no equivale a inexistente globalmente**. Los resultados publicados de V1, si aparecen en terceros, describen otro sistema y no se trasladan como métricas actuales.

## 8. Matriz comparativa

Estados relativos al contrato concreto, no puntuación de producto:

| Capacidad | Letta fuente observada | Vanta snapshot | Decisión |
|---|---|---|---|
| Memoria editable/provenance | Existe MemFS Git y modelos de bloque [L4][L16] | Existe L0/L1, provenance y personas [V6][V7] | Extender revisión publicada, no otro primario |
| Recall vectorial/híbrido | Parcial: local FTS implementado y rechaza vector/hybrid [L19][L20]; remoto delegado, motor no leído | Existe motor BM25/HNSW/RRF; ruta memoria configurable [V2] | F, no atribuir el backend remoto al local |
| Archivo/recall histórico | Existe búsqueda JSONL y summaries; colección archival separada no acreditada [L20][L21] | Existe L0/provenance [V6][V7] | No importar arquitectura V1; G solo linaje |
| Lifecycle de agente/sesión | Documentado en SDK actual [D3], LocalStore parcialmente leído | Checkpoints y sesiones existen, no equivalencia runtime [V5][V6] | Adapter de host, no nuevo motor de agentes |
| Contexto de revisión aceptada | Existe compilación committed [L5] | Parcial: assembly/report existentes [V1], snapshot conjunto no certificado | Ficha A |
| Read-only/proyección | Existe validador con árbol previo [L6] | Parcial para este contrato: protección estructural [V3] | Ficha B, reusar rutas |
| Reparación en background | Existe claim por firma [L10]; cobertura adjuntos discutida | Existe scheduler/checkpoint [V4][V5]; mismo problema-digest no acreditado | Ficha C condicional |
| Compactación | Existe, fallos irreducibles y plan [L12] | Existe [V1], prefijo protegido puede exceder budget | Ficha D solo delta comprobable |
| Sesiones/conversaciones | Existe LocalStore [L11] | Existe L0 y checkpoint de tarea [V5][V6] | No confundir ejecución con conocimiento |
| Borrado completo fuente/derivados | No acreditado globalmente; deleteAgent acotado | Parcial para este contraste; erasure existe [V8] | No deducir GDPR completo por nombre |
| Cloud sync | Cliente Git observado, servicio no auditado | N/A como requisito del motor embebido | Adapter solo si caso real |
| Refresh padre autenticado | `runMemoryGit` en el pin [L-worktree]; fix/versión reportados en #4249 aún open | No aplica Git como requisito del motor | E/C conservan errores por fase, no fallo universal ni resolución total |
| Desktop/web | Documento oficial y TUI real; cliente completo no leído | Studio existe en worktree separado | Extender Studio, no crearlo |

## 9. Negativos, hipótesis y falsos positivos Vanta

- **Hecho limitado:** `ingest::merge::commit` no declara un commit transaccional de todo el árbol; llama `write_page` por página [V3]. Un plan aprobado sobre árbol completo necesita definir concurrencia y publicación. No prueba pérdida de datos en el caller.
- **Hecho limitado:** L0 persiste mensajes y después cursor [V6:274-372]; ya tiene tie-break para mismo milisegundo y replay. No proponer “dedup inexistente”. Un recibo durable por evento de host es un contrato distinto.
- **Hipótesis por discriminar:** un contexto compuesto de varias fuentes podría mezclar revisiones o usarse tras revocación. Revisar callers de [V1][V2] antes de declarar bug.
- **Falso positivo descartado:** Vanta no carece de personas, compactación, scheduler, checkpoints ni Studio. No convertir un conflicto Git reportado en defecto del WAL Vanta.
- **Hallazgo de alcance, no CVE:** `utils/erasure.rs::create_scope` documenta serialización del host para exists-check+put [V8:111-139]. Aplica a creación de DEK de ese helper, no acredita fallo universal de transacciones ni autoriza cambiar erasure en esta investigación.

## 10. UI, escritorio y Studio

**Real leído:** `MemoryTabViewer` React/Ink muestra bloques y contadores; `memory-scanner.ts` recorre árbol y se declara compartido por TUI y generador web [L13][L14]. Ordena directorios, prioriza `system`, omite ocultos, navega archivos y devuelve marcador si falla lectura. `statSync` y recursión requieren considerar symlinks/rutas hostiles; no se auditó el confinamiento de todos sus callers.

**Documentación, no implementación leída:** [L0] enlaza chat web/móvil, canales, Agent SDK y Cloud; `AGENTS.md` remite a interfaces desktop/web actuales. No se descargó ni inspeccionó el cliente desktop completo y no se identifica como Electron/Tauri por inferencia. TUI no equivale a escritorio; web no equivale a binario nativo.

**Studio ya existe:** worktree `develop-desktop`, SHA documental
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`. Cuerpos releídos con `git show`:
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409,488-550`
consulta genlog/anchors y selecciona sesión;
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
pagina audit y separa error/no configurado;
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
pide explain y get/vecinos/audit. Esto acredita seams para E, **no** la nueva
proyección de publicación. `GraphScene` permanece solo localizado, sin cuerpo
contrastado aquí. No se atribuyen bytes dirty de `desktop/...` al pin ni se
acredita JobChip/jobs o runtime. Oportunidad E: mostrar fuente/aceptado/
proyectado/entregado y fallo por fase; preview compara revisión y reautoriza
al aplicar, no botón shell/Git ni permiso concedido por UI.
Fuentes Studio inmutables de esos cuerpos (relectura 2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

## 11. Fuentes, cobertura y pendientes

### Evidencia fijada

L0/L1 apuntan al **clon solicitado**; L2-L17 al **complemento actual remoto**, no al clon ni al servidor retirado. Los enlaces V fijan el snapshot contractual.

[L0]: https://github.com/letta-ai/letta/blob/8816d3a8516893bfdc0fa14bfb1235226012e295/README.md#L1-L38
[L1]: https://github.com/letta-ai/letta/blob/8816d3a8516893bfdc0fa14bfb1235226012e295/AGENTS.md
[L2]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/package.json
[L3]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-filesystem.ts#L108-L465
[L4]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-git.ts
[L5]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/local/system-prompt-compilation.ts#L1-L130
[L6]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/memory-constraints.ts#L1-L180
[L7]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-format.ts#L1-L40
[L8]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-operation.ts#L1-L58
[L9]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/utils/file-lock.ts#L1-L140
[L10]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-conflict-repair.ts#L119-L234
[L11]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/local/local-store.ts
[L12]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/local/compaction.ts#L43-L205
[L13]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/cli/components/MemoryTabViewer.tsx
[L14]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-scanner.ts#L1-L108
[L15]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/LICENSE
[L16]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory.ts#L16-L105
[L17]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/scripts/latency-benchmark.ts#L1-L340
[L18]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/local/initial-memory.ts#L1-L116
[L19]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/message-search.ts#L1-L59
[L20]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/local/transcript-search.ts#L1-L630
[L21]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/conversation-search.ts#L1-L115
[L22]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/prompts/recall_subagent_local.md#L1-L101
[L23]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/backend/api/search.ts#L1-L19
[L-worktree]: https://github.com/letta-ai/letta-code/blob/9176d8c0b16f408e9240422a98379a423e2176c3/src/agent/memory-worktree.ts#L286-L293
[D1]: https://docs.letta.com/agent-sdk/memory
[D2]: https://docs.letta.com/configuration/memory
[D3]: https://docs.letta.com/agent-sdk/sessions
[D4]: https://docs.letta.com/v1-sdk/concepts/stateful-agents
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L239-L288
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L56-L346
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L188
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/conversation/l0_recorder.rs#L274-L372
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs
[V8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/erasure.rs#L111-L180

### Cobertura y cierre

Lectura de cuerpos: checkout/scoping/árbol [L3:108-155,280-465], commit y sync [L4:1180-1268,1833-1977], compilador [L5:1-130], formato completo [L7], validador [L6:1-180], lock [L9:1-140], repair [L10:119-234], compactación [L12:43-205,718-867], scanner completo, defaults de bloques y runner. LocalStore: modelos/mapas, creación/listado/borrado y puntos de persistencia seleccionados, no archivo entero. MemFS/auth/helper restantes se localizaron por símbolos/imports; **localizar no cuenta como auditar cuerpo**.

Ampliación de cierre: leídos completos [L18][L19][L21][L22][L23]; [L20] lectura directa de DTOs, texto buscable, ranking, fechas, colección de filas, errores y entry point (`1-185,285-630`), no auditoría del parser `187-283` ni del lector JSONL importado. Guías [D1-D3] leídas completas como documentación viva (consulta 2026-11-07, sin SHA); [D4] solo para identificar/excluir redirección legacy. Se releyó cuerpo y comentario visible de #4831; se conservan las otras incidencias previamente documentadas sin afirmar nueva reproducción. GitHub Trees por `urllib` agotó rate limit 403; fallback a raw fijado funcionó. Metasearch `site:docs.letta.com "recall" "MemFS"` no devolvió resultados en Bing/DuckDuckGo; no se dedujo ausencia.

No leído: V1 deliberadamente; App Server completo, backend SaaS, clientes desktop/web completos, SDK transitivo, servidor Git remoto y enforcement de todas las rutas. No se ejecutaron los tests, herramientas Git de las trazas, CLI ni benchmark. La fuente complementaria evita un informe histórico falso, pero no convierte esta muestra en auditoría completa del nuevo producto.

Dependencias: B → A para publicación validada; A → E para visibilidad; C usa revisión/validación A/B; D se integra al assembler sin esperar Git/Cloud; F/G comparten refs, no un índice adicional. Cada ficha tiene aceptación manual futura, no especificación de harness. Pendientes externos: alcance SDK/App Server, cobertura end-to-end de auth/checkout tras los fixes reportados de #4249 y matices de #5091, políticas de reparación compartida y divergencia guía system/ frente a layout local v2.

**Recibo [L-1/L-2](reviews/memory-additions.md#letta):** seguimiento, runner
del pin y cuerpos Studio incorporados; propagados a E/matriz/UI y pendientes.
No aceptación ni certificación universal de una versión publicada.

**Plan de división si se aprueban specs:** separar publicación/proyección (A/B), repair/compactación (C/D), recall/linaje (F/G) y producto/medición (E/§7), manteniendo aquí identidad, límites y fuentes. Este archivo supera 300 líneas por cobertura de mecanismos, no autoriza crear esos documentos ahora. La autoría trata los diez ejes con las lagunas anteriores abiertas; no constituye revisión independiente ni aceptación final.
