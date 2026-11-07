---
title: "Flowise: composición de recuperación y configuración observable"
kind: research
status: proposed
description: "Análisis estático de ejecución, colas, estado, memoria, índices derivados y canvas de Flowise frente a los seams existentes de VantaDB"
tags: [research, memory, engineering]
---

# Flowise: composición de recuperación y configuración observable

## Índice

- [0. Identidad, arquitectura y tres trazas](#0-identidad-arquitectura-y-tres-trazas)
- [1. Arquitectura, stack, flujo y recomendación](#1-arquitectura-stack-flujo-y-recomendación)
- [2. Mecanismos mínimos extraíbles y seams Vanta](#2-mecanismos-mínimos-extraíbles-y-seams-vanta)
- [3. Riesgos estáticos frente a hipótesis](#3-riesgos-estáticos-frente-a-hipótesis)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. No trasladar](#5-no-trasladar)
- [6. Límites, licencia, método y ledger](#6-límites-licencia-método-y-ledger)
- [7. Benchmarks/runners realmente leídos](#7-benchmarksrunners-realmente-leídos)
- [8. Matriz capacidades Vanta](#8-matriz-capacidades-vanta)
- [9. Deuda sugerida sin falsos bugs](#9-deuda-sugerida-sin-falsos-bugs)
- [10. Interfaz/escritorio e ideas Studio](#10-interfazescritorio-e-ideas-studio)
- [11. Dependencias, cobertura y separación futura](#11-dependencias-cobertura-y-separación-futura)

## 0. Identidad, arquitectura y tres trazas

Esta ampliación conserva S1–S10 y corrige su alcance, no repite la investigación
ya acreditada. Referencia Vanta única para conclusiones nuevas:
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; c561158 y el HEAD dirty anterior
son antecedentes, no estado actual. Se aplican
[EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [VANTA-REFERENCE](VANTA-REFERENCE.md),
[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md).
Clon leído: `C:\Users\Eros\Flowise`, SHA
`9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`, captura del 2026-11-07.
No se recalcula metadata congelada ni se certifica runtime.

### 0.1 Mapa, modelos y superficies

```text
packages/ui (React/MUI/Redux + ReactFlow) → JSON nodes/edges
  Express routes/controllers → ChatFlow.flowData (TypeORM)
  prediction → utilBuildChatflow → executeFlow
    modo directo / BullMQ→Redis→PredictionQueue.processJob
      AGENTFLOW → executeAgentFlow → executeNode → componente.run
        runtime.state / chatHistory / form / webhook
        Execution en DB + SSE/pubsub + ChatMessage al finalizar
  documentstore → chunks DB → embeddings/vector provider + record manager
    UpsertHistory y estado de store en DB; commits remotos separados
packages/components → nodos LangChain/modelos/tools/memory/retrievers
packages/agentflow → editor embebible; no implica runtime embebido
```

| Modelo / frontera | Persistencia e identidad observadas |
|---|---|
| `ChatFlow` | UUID, `flowData:text`, tipo CHATFLOW/AGENTFLOW/MULTIAGENT/ASSISTANT, workspace, API/analytics config; definición, no ejecución |
| `Execution` | UUID, `agentflowId`, `workspaceId`, `sessionId`, `state`, `executionData:text`, acción/fechas; snapshots de pasos, no memoria semántica |
| `ChatMessage` | UUID, flow/chat/session, rol/contenido, executionId opcional, sourceDocuments/usedTools/artifacts; historial diferente del checkpoint |
| `IAgentFlowRuntime` usado por runner | Estado mutable, historial, formulario/webhook; puede restaurar state anterior si Start lo pide |
| `DocumentStore` / `DocumentStoreFileChunk` | Config de loader/embedding/vector/record manager y chunks locales; no transacción distribuida con proveedor |
| Job BullMQ / eventos SSE | Trabajo y entrega de eventos: sus IDs no son revisión del documento ni prueba de lectura por el cliente |

Fuentes nuevas F1–F8 al final de §6 fijan archivo, símbolo y SHA. El servidor
usa TypeORM, BullMQ/Redis y carga dinámica de nodos; credenciales/modelos y
vector stores son dependencias de cada flow, no requisitos de toda primitiva.
`routes/predictions/index.ts:10-15` recibe POST con archivos/rate limiter;
`routes/documentstore/index.ts:8-79` expone upsert/refresh, chunks, configuración,
insert/query/delete vectorial. RBAC importa código enterprise: leer la ruta no
certifica todos los permisos ni habilita copiar su implementación comercial.

### 0.2 Traza 1 — definición, solicitud, cola y escritura de ejecución

1. Canvas v2 `handleSaveFlow` limpia status/selección de nodos, separa referencia
   de credencial y serializa `reactFlowInstance.toObject()`; crea tipo AGENTFLOW
   o actualiza `flowData` (F5:219-254). Guardar no ejecuta ni valida sus efectos.
2. `utilBuildChatflow` carga flow por ID, deriva chatId de entrada/session/UUID,
   valida clave para llamada externa, resuelve workspace→organización y cuota
   (F1:990-1082). No inferir autorización universal desde un helper.
3. Modo QUEUE: `addJob(omit(executeData,...))`, luego
   `job.waitUntilFinished(queueEvents)`; **no es una API que retorna listo al
   encolar**. En directo registra AbortController y llama `executeFlow`.
   El worker reinyecta DataSource/cache/telemetría/nodos/SSE Redis y crea signal
   (F2:65-107). Estos objetos no se transfieren como ownership serializable.
4. `executeFlow` deriva AGENTFLOW a `executeAgentFlow` (F1:481-505). Éste parsea
   nodes/edges, valida Start/historial y frontera webhook/schedule, construye
   graph/reversedGraph y cola local de nodos (F3:1575-1660).
5. Bucle `nodeExecutionQueue.shift()`→`executeNode`→procesar salidas/dependencias.
   Abort se comprueba al entrar; STOPPED representa espera humana, no fracaso;
   ERROR/TERMINATED persisten detalle y propagan error. FINISHED guarda
   Execution y emite SSE **antes** de guardar los dos ChatMessage finales
   (F3:2020-2235, 2363-2409). No hay commit único de tools, DB, Redis y mensaje.

**Errores/fronteras:** `BaseQueue.addJob` pasa un nombre `jobId` a `queue.add`,
no la opción `{jobId}`: no acreditar idempotencia sólo por la variable.
`removeOnFail=true` por defecto y concurrencia de worker por env/default
100000 (F2/BaseQueue:5-56); son defaults de código, no medición de paralelismo.
Fallo DB/SSE después de herramienta aplicada puede devolver error con efecto
externo ya realizado. Retirar AbortController no deshace ese efecto.
Un timeout/desconexión del caller no es rollback ni ausencia del trabajo.

### 0.3 Traza 2 — recuperación, memoria y state

1. `BufferMemoryExtended.getChatMessages` usa sessionId efectivo + chatflowid,
   orden ASC, añade prepend y convierte a mensajes; sin ID retorna `[]`.
   `addChatMessages`/`clearChatMessages` son no-op deliberados: el servidor es
   dueño de esas escrituras (F4:102-146). No llamar “memoria sólo en RAM” al nodo.
2. Agentflow v2 inicia runtime vacío y busca ejecuciones por sessionId,
   agentflowId y workspace; si `startPersistState===true` restaura el último
   `data.state` encontrado (F3:1664-1715). Eso contradice una afirmación absoluta
   de state siempre efímero; no convierte state en perfil durable autorizado.
3. Retriever S1 resuelve store→config persistida→nodo vectorial→`invoke` con
   signal; texto/JSON→plantillas→nuevo state→SSE si es último nodo.
   Varias fuentes sobrescriben `docs`; error de una no tiene outcome parcial
   tipado. S3 intenta reconciliar hits por contenido, no revisión estable.

**Concurrencia y confianza:** restaurar “última ejecución” no demuestra CAS,
aislamiento por turno ni serialización de dos llamadas simultáneas a sesión.
Cancelar es cooperativo; el proveedor debe observar signal. Contenido/metadata,
history suministrado y `$flow.state` son datos, nunca autoridad para tools/ACL.
La muestra BufferMemory no prueba controles globales multi-workspace ni su
ausencia; el flow y la autorización del caller siguen siendo fronteras.

### 0.4 Traza 3 — actualización/indexación y borrado con derivados

`insertIntoVectorStore` guarda config→UPSERTING→worker; éste carga chunks
locales, construye embedding/record manager/vector store, invoca
`vectorStoreMethods.upsert`, escribe UpsertHistory, envía telemetría y finalmente
UPSERTED (F6:1270-1444). Un error de historial/telemetría puede llegar **después**
del upsert remoto; no se observa compensación universal. Cambiar config y
reindexar son acciones distintas. Refresh/upsert usa cola opcional y espera
job; `executeDocStoreUpsert` recorre items, no lote atómico (F6:1995-2077).

`deleteDocumentStore` elimina chunks→archivos (error capturado)→historial→store
(F6:319-354). La proyección remota tiene ruta **separada**:
`deleteVectorStoreFromStore` exige config embedding/vector/record manager y
sólo llama delete si el proveedor lo expone, con IDs resueltos por record
manager (F6:399-470). No equiparar borrar catálogo con borrado vectorial.
Tampoco ese método acredita purga de backups o visibilidad inmediata remota.
`removeAllChatMessages` borra feedback/uploads/mensajes; la variante por IDs
borra Execution vinculadas antes de mensajes (F7:110-180). Son fronteras
diferentes del state en curso; borrar historial no cancela un worker.

## 1. Arquitectura, stack, flujo y recomendación

**ARCHIVADO según el snapshot del 2026-11-07: `archived=true`.**
No se oculta ni se infiere mantenimiento activo a partir de estrellas o código.
**Dictamen:** referencia histórica de composición/diagnóstico, no dependencia
estratégica del core. Estudiar identidad de resultados y precedencia de config,
sin portar flows visuales, marketplace o el monolito Node.

- Repositorio: `FlowiseAI/Flowise`; clon `C:\Users\Eros\Flowise`.
- SHA congelado: `9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`.
- Snapshot: **55 562 estrellas**, **archivado**, licencia API `NOASSERTION`.
- Fuente: [candidatos](github-candidates.json) y [clone-manifest.json](clone-manifest.json).
- LICENSE distingue contenido Apache 2.0, enterprise/comercial y terceros.
- `NOASSERTION` no autoriza copiar ni significa ausencia de licencia.

Monorepo Node/TypeScript con componentes LangChain, servidor Express/TypeORM y
UI React/MUI/Redux; paquetes y scripts de Turbo/pnpm constan en manifiestos.
Los nodos construyen proveedores a partir de configuraciones persistidas,
con imports dinámicos y credenciales que resuelve el entorno de aplicación.
No se auditaron todos los proveedores, agentes ni mecanismos enterprise.

Flujo leído: Retriever Agentflow → lista de document stores → cargar config
embedding/vector store → construir retriever → `invoke` con cancelación →
texto o JSON metadata → estado de ejecución/SSE.
La consulta diagnóstica del servidor usa un flujo similar y trata de reconciliar
los resultados con chunks persistidos por store y contenido.
RRF es otro nodo: genera variantes LLM, busca por variante y fusiona; no se
demuestra que esté activo en el flujo Agentflow inspeccionado.

Vanta ya tiene BM25/HNSW/RRF/sparse, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints, ingestors/auditoría y Studio.
Studio Tauri/React vive en `develop-desktop`; no se propone otra aplicación.
El contraste nuevo usa el snapshot de §0 y lecturas discriminantes de §8;
no se reaudita ni se persigue el HEAD concurrente.

**Prioridades propuestas, no aprobadas:** P0 conservar identidad/outcome por
fuente (A) y recibos honestos de derivados (D); evitan falso éxito/pérdida de
procedencia. P1 separar checkpoint/state/memoria en adaptadores (C) y hacer
observable precedencia/emisor (B). P2 presentar esos datos en Studio existente.
Ninguna prioridad depende de adoptar Flowise, Redis, LLM o su runtime.

## 2. Mecanismos mínimos extraíbles y seams Vanta

### A. Resultado compuesto con identidad y outcome por fuente

**Problema:** componer varias fuentes exige conservar qué devolvió cada una,
su identidad estable y sus errores; concatenar texto no lo garantiza.
Unidad mínima: envelope de resultados inspirada en fronteras S1/S2/S3,
**no copiar el merge observado**, que tiene limitaciones estáticas.
S1 permite seleccionar varios stores, pero asigna `docs = ...` en cada vuelta.
S3 intenta resolver ID por contenido y genera UUID si no encuentra chunk.
S4 usa `relevancy_score` para distinguir resultados; score no es identidad.

**Seam Vanta:** retrieval/RRF existente → provenance → assembler/reporte;
ingestors o conexiones externas como adaptadores, no nuevo agregador en core.
**DTO hipotético, no API existente:** `SourceRetrievalBatch { scope,
source_id, source_revision, config_revision, status, hits }`;
`EvidenceHit { evidence_id, revision, rank, score_domain, source_ref }`.
Operaciones propuestas: `retrieve_scoped`, `record_source_outcome`,
`merge_by_identity`, `select_existing_rrf`; sin LLM obligatorio.

**Acciones:** comprobar composición actual antes de agregar tipos; preservar
resultados por fuente hasta ensamblado; distinguir error, vacío y cancelación;
mantener desempates por identidad sin descartar documentos de igual score.
El DTO externo se traduce a provenance ya existente, no a metadatos arbitrarios
que el motor tenga que interpretar como política.
**Dependencias:** IDs/revisiones, autorización por fuente, cancelación y
presupuesto compartido existentes; revisión del adaptador si usa red/modelos.
**FMEA:** asignación por iteración → fuentes anteriores desaparecen; empate
deduplicado por score → pérdida de documentos; match por contenido → identidad
ambigua; fallo parcial → falso éxito total. Mitigar con batches por fuente,
clave revisionada, outcome tipado y política explícita de parcialidad.
**Aceptación pendiente manual:** dos fuentes conservan sus hits; scores iguales
no colapsan IDs distintos; mismo contenido con distinta procedencia se representa;
cancelar no se informa como éxito vacío; no mezclar scopes.
**No objetivo:** implementar RRF de nuevo o copiar generación multi-query.

**Contrato y seam concretos (complemento de A):** input propuesto
`retrieve_scoped(scope, query, source_revisions, cancel) -> Result<Vec<SourceRetrievalBatch>,
RetrievalAdapterError>`. Error tipado: Unauthorized, UnsupportedRevision,
SourceFailed, Canceled; éxito vacío requiere fuente ejecutada. El adaptador
posee buffers de cada fuente y devuelve hits owned; no presta referencias a
una petición que puede expirar. `scope` deriva de autorización, no de texto.
Usar `RecallConfig`/`InjectionPolicy` en
`vanta-memory/src/core/hooks/auto_recall.rs`, `assemble_with_recall` en
`context_engine/engine.rs` y `src/sdk/search/fusion.rs::fuse_rrf_many`;
no presentar `MemoryScope` como tipo existente.

**Secuencia/migración:** (1) mapear IDs externos a namespace/key y revisión
de provenance existentes; (2) resolver por fuente sin perder outcome; (3) usar
fusión vigente y ensamblador; (4) exponer sólo metadatos autorizados en reporte.
Alternativa mínima: ampliar reporte existente sin DTO público; adaptador si
hay frontera de proveedor; rechazar agregador nuevo. Versionar envelope externo;
registros legacy sin revisión quedan Unknown, no inventar UUID estable.
No exige reindexar contenido; desactivar el envelope conserva retrieval actual.
Depende de D sólo cuando se promete revisión visible; B no bloquea consulta.
FMEA adicional: detectar cardinalidad/identidades por fuente antes/después del
merge; cancelación posterior al retorno no debe invalidar ownership de hits.

### B. Configuración efectiva de observabilidad con dueño único

**Problema:** entorno y UI pueden activar trazadores duplicados o hacer que una
preferencia aparente no coincida con el proveedor realmente usado.
Unidad mínima: resolución de precedencia y ownership de emisión de S5.
El resolver cachea configuración de entorno durante la vida del proceso.
UI gana cuando su proveedor tiene `status:true`; en otros casos el entorno
habilitado puede aportar entrada. Un UI desactivado no es veto universal.
Al adoptar LangSmith, elimina flags del auto-tracer para evitar doble emisión.
Eso describe una decisión upstream; no se recomienda mutar env global de Vanta.

**Seam Vanta:** configuración/diagnóstico y auditoría existentes → actividad y
conexiones Studio; exportador externo opt-in fuera del motor.
**DTO hipotético:** `EffectiveObservationConfig { revision, enabled,
origin: User|Environment|Default, emitter_owner, destination_ref, redaction }`.
Nunca incluir token, URL con secreto ni texto recuperado por defecto.
Operaciones propuestas: `resolve`, `inspect_redacted`, `emit_once`,
`report_reload_requirement`; no construir un servicio nuevo.

**Acciones:** cotejar precedencia actual; hacer visible origen y cuándo entra
en vigor un cambio; elegir un dueño de eventos y no un segundo auto-tracer;
mantener valores secretos en el dueño existente de credenciales.
**Dependencias:** configuración versionada, lifecycle, redacción y trazas locales;
conectividad solo si el usuario aprueba exportación externa.
**FMEA:** dos emisores → costes/trazas duplicadas; caché eterna → config obsoleta;
disable de UI sobreescrito por env → expectativa de privacidad rota;
error exportador → bloqueo del retrieval. Mitigar con política explícita,
estado efectivo inspeccionable y fallo de exportación separado.
**Aceptación pendiente:** un evento tiene un dueño; origen efectivo es visible;
desactivación tiene semántica documentada; inspección no revela credenciales;
retrieval local funciona sin exportadores ni red.
**No objetivo:** adoptar LangSmith/analytics o reiniciar servicios en esta tarea.

**Contrato/secuencia (complemento de B):**
`resolve(user, environment_snapshot, defaults) -> Result<EffectiveObservationConfig,
ObservationConfigError>`; `inspect_redacted(revision)` devuelve origen y necesidad
de reload, nunca credencial. Inputs owned/inmutables por revisión; emisor
con lifetime de conexión fuera del motor, cierre idempotente y cancelación
independiente de recall. Reutilizar diagnóstico/auditoría existentes; seam
visible Studio: `ActivityPanel` (§10), no mutar env desde una consulta.
Pasos: inventariar precedence actual→resolver puro→redactar→conectar emisor
único opt-in→mostrar revisión efectiva. Alternativas: sólo documentar precedencia
si ya es verificable; adapter de exportación si hay red; rechazar auto-tracer
global. Migración aditiva, no reindex; legacy muestra origen Unknown hasta
resolución. Desactivar exportador no cambia persistencia ni resultados.
FMEA/detección: revisión configurada ≠ efectiva exige diagnóstico de reload;
contar dueño/evento, no guardar payload para detectar duplicados. Ningún fallo
de exportación revierte una escritura D ni vuelve terminal un checkpoint C.

### C. Identidad y ownership separados de ejecución, state y memoria

**Caso agente:** reanudar una ingestión interrumpida sin interpretar checkpoint
como recuerdo, sin volver a ejecutar herramienta ya aplicada y sin reutilizar
por accidente la sesión de otro workspace. Unidad observada: F3 restaura state,
valida STOPPED/resume (`1776-1868`) y persiste Execution; F4 delega escritura de
mensajes al servidor. No portar graph runner, nodos, condiciones ni LangGraph.

**Delta Vanta y seam:** ya existen sesiones/provenance, generación de memoria,
context assembly y `TaskCheckpointManager::{begin,load,advance,complete,fail}`.
En snapshot fijado, `utils/task_checkpoint.rs:112-246` documenta un escritor
por task_id y RMW; `l1_writer.rs::plan_write/persist_planned/log_and_mark`
separa recuerdo de efectos secundarios. Delta: vínculo explícito de esos
recursos en adaptadores/actividad, no scheduler ni tabla de memoria nuevos.

**Interfaz Rust propuesta, no implementada:**
`RunMemoryBinding { run_id, task_id, session_key, session_id, scope_ref,
definition_revision, checkpoint_revision: Option<u64> }`;
`inspect_binding(&AuthorizedScope, &RunId) -> Result<RunMemoryBinding, BindingError>`;
`resume_existing_task(binding, expected_revision, cancel) -> Result<ResumeDisposition,
BindingError>`. Disposition: Resumable, RequiresReconciliation, Terminal;
errores: ScopeDenied, RevisionConflict, MissingCheckpoint, UnsupportedLegacy.
`scope_ref` es referencia a autorización vigente, no bypass guardado en JSON.
Tipos y revisión esperada son diseño, no promesa de CAS hoy en Vanta.

**Invariantes/ownership:** el adaptador posee run binding; TaskCheckpointManager
sigue dueño de checkpoint; sesiones/L1 son dueños del recuerdo. Mantener
escritor único por task; no sostener lock durante red/modelo. Una pausa humana
no marca Failed, y cancelar await no elimina checkpoint ni revierte efectos.
No reanudar automáticamente desde una cadena de error que contenga “Aborted”.
La revisión de definición evita ejecutar un paso de un flow distinto tras editarlo.

**Pasos, alternativas y migración:** (1) inventariar IDs y callers existentes;
(2) asociar checkpoint/sesión en adapter, con definición revisionada;
(3) rechazar reanudación ambigua y reconciliar D; (4) mostrar referencias sin
contenido en Activity; (5) habilitar sólo transport autorizado. Preferir
extender el checkpoint/reporte actual; si no hay flujo reanudable, sólo
inspección. Rechazar copiar snapshots completos de Flowise a memoria semántica.
Registro legacy sin revisión no es resumible por inferencia; migración lazy
del vínculo, sin reindex de recuerdos. Desactivación vuelve al camino actual,
no borra checkpoints ni habilita repetir side effects.

**Dependencias:** aislamiento/IDs actuales; D cuando hay mutación externa;
A para evidencias de consulta; B sólo diagnóstico. FMEA:

| Disparador → mecanismo → efecto | Detección → mitigación propuesta |
|---|---|
| Dos resumes → RMW pierde avance → doble herramienta | conflicto de revisión/owner → escritor único; CAS sólo con soporte acreditado |
| Editar definición → mismo run/otro grafo → paso erróneo | revision mismatch → reconciliar, no replay ciego |
| Reset de sesión → checkpoint confundido con recuerdo | IDs/tipos separados → no borrar uno al limpiar otro |
| Timeout después de efecto → retry → duplicado | D incierto → consultar/reconciliar antes de repetir |

**Aceptación manual futura:** mismo task no se inicia dos veces; vínculo de
otra scope se deniega; legacy sin definición no se reanuda silenciosamente;
STOPPED/resumible se distingue de Failed; borrar conversación no afirma
cancelación ni purga de efectos. Observabilidad por IDs/estados, sin prompts.
No objetivos: ejecución visual, exactly-once distribuido, perfil de usuario nuevo.

### D. Recibo de mutación y conciliación de proyección externa

**Caso agente:** subir un documento y consultarlo, o retirarlo y comprobar
qué derivados siguen visibles. F6 muestra upsert remoto anterior a historial/
telemetría/status; delete del catálogo y delete vectorial son rutas distintas.
Unidad extraíble: contrato de frontera, no proveedor vectorial ni transacción
distribuida. Mismo candidato transversal que Meilisearch A; no dos ledgers.

**Seam/delta:** `l1_writer.rs::persist_planned`, `apply_dedup_batch` y
`log_and_mark`, SDK Embedded/backend/WAL y checkpoints ya existen. Añadir
evidencia al resultado del ingestor/adaptador sólo donde no la exponga; no
reemplazar commit WAL por status UPSERTED ni hacer de un recibo un WAL nuevo.
Provenance conserva la relación fuente→derivado; scheduler actual concilia.

**Forma canónica propuesta en ambos informes, no API existente:**

```rust
// Esquema conceptual; no implementación ni contrato aprobado.
struct MutationReceipt {
    operation_id: String, scope_ref: String, source_revision: String,
    accepted: Observation<Acceptance>,
    applied: Observation<AppliedRevision>,
    durable: Observation<DurabilityEvidence>,
    visibility: Vec<ProjectionObservation>,
}
// Observation<T>: Unknown | Pending | Confirmed(T) | Failed(ErrorRef).
// ProjectionObservation: projection_id + target_revision + Observation<ReadEvidence>.
```

Accepted: responsable reconoce operación; Applied: cambio primario acreditado;
Durable: evidencia y política concreta de persistencia (no bool “fsync” inferido);
Visibility: observación por proyección/revisión/scope. Una dimensión Unknown
no se rellena desde otra. Guardar recibo durable no vuelve durable al dato remoto.
`inspect_mutation(scope, operation_id)` y
`reconcile_projection(receipt, projection_id, cancel)` son operaciones propuestas;
Unauthorized/UnsupportedEvidence/RevisionConflict/Transport son errores
separados del outcome ya observado. Timeout devuelve resultado incierto/recibo
conocido: **timeout no es rollback** ni Failed de una mutación aplicada.

**Pasos/migración:** inventariar garantías por camino→mapear evidencia real
a dimensiones→correlacionar operation_id estable antes de red→persistir en
checkpoint/actividad existentes→consultar proveedor por ID/revisión, nunca
reinsertar ciegamente→exponer inspección. Alternativa inicial sin persistencia
nueva: envelope de respuesta con Unknown; adaptar proveedor si soporta consulta;
rechazar prometer visibilidad si sólo retorna ack. Legacy se conserva como
Unknown, sin upgrade ficticio; no reindex salvo cambio de embedding/chunking
decidido aparte. Deshabilitar conciliación no borra documentos ni recibos.

**Ownership/concurrencia:** ingestor posee operación; proveedor posee commit
remoto; snapshot/revisión de config acompaña job. Una respuesta tardía de
revisión vieja no pisa nueva; borrar exige autorización actual e identidad
de derivados. Cancelación deja observaciones acreditadas y marca sólo espera
interrumpida. Depende de C para reanudar, de provenance para identificar;
A puede consumir visibility sin depender de UI/telemetría B.

| FMEA: disparador → mecanismo → consecuencia | Detección / mitigación |
|---|---|
| Error telemetría tras upsert → error global → retry duplica | comparar receipt/proveedor → no degradar Applied a no aplicado |
| Delete catálogo → vector remoto permanece → recuerdo reaparece | visibilidad por derivado → conciliación explícita, no “borrado total” |
| Worker viejo usa config nueva → revisión mezclada | config_revision/target_revision → rechazar o declarar incertidumbre |
| Ack sin read-after-write → falso listo | Visibility Pending/Unknown → consulta autorizada, no deducir desde Accepted |

**Aceptación manual futura:** ack sin aplicación no es ready; indexación
demorada conserva primario aplicado/durable según evidencia; falla secundaria
no borra commit; timeout tras commit conserva operation_id; delete sin método
remoto se declara unsupported, no purga exitosa; ninguna inspección revela
contenido/credenciales. No objetivos: atomicidad global, scheduler nuevo,
barrier de lectura inventado ni port del runtime de Flowise.

## 3. Riesgos estáticos frente a hipótesis

- **Código:** S1:153-176 sobrescribe `docs` por store. Para dos stores exitosos,
  el valor final conserva la última asignación, no la unión. No ejecutado.
- **Código:** S4:83-90 usa score en `seenScores`, no ID documental.
  Dos documentos distintos con igual score no se conservan ambos en esa función;
  inferencia lógica del bucle, no medición de recall en una aplicación real.
- **Código:** `RRFRetriever.ts:112` elige default de `c` usando `topK`, no `c`.
  Configuraciones no predeterminadas requieren comprobación antes de copiar.
- **Código:** S3 reconcilia contenido de resultados con chunks; fallback UUID
  no acredita continuidad de identidad ni revisión de fuente.
- **Hipótesis:** contenido igual en varios chunks puede producir atribución
  ambigua; requiere fixtures del store y consulta completa para demostrarlo.
- **Código:** S2/S3 resuelven por ID; no se revisó la cadena completa de
  controladores/ACL. No se afirma fuga cross-workspace ni vulnerabilidad.
- **Código:** S5 cachea env y borra flags de auto-tracing; un cambio de env
  no actualiza el valor cacheado. El comentario exige reinicio para recargar.
- **Hipótesis de seguridad:** texto con metadata → estado/plantillas de agente;
  no debería conceder privilegios. No se auditaron todas las herramientas.

Las incidencias públicas leídas se detallan en §11; la afirmación inicial
«no revisados» queda sustituida. No se inventan CVEs ni explotación atribuida.
ARCHIVADO eleva riesgo de continuidad/mantenimiento, pero no prueba inseguridad
de una ruta concreta. No se recomienda adoptar dependencia de un fork sin evaluar.

## 4. Prerrequisitos Vanta

Revalidar la implementación actual antes de aprobar tareas; RRF ya existe.
Conservar scope y provenance al mezclar conectores, sin merge por score.
Definir resultado parcial y cancelación en el reporte actual.
Precisar quién fija configuración efectiva y quién emite/exporta diagnósticos.
Comprobar políticas existentes de redacción/retención y cierre de conexiones.
Separar revisión documental, revisión de embedding y configuración de búsqueda.
Revisión legal por archivo: enterprise y avisos explícitos tienen condiciones
distintas; reescribir en Rust no elimina obligaciones si se deriva expresión.

## 5. No trasladar

No portar marketplace, canvas, cloud, billing/enterprise ni monolito Node al core.
No copiar esta implementación RRF ni justificar duplicar la de Vanta.
No usar score como identidad, ni contenido como sustituto de fuente revisionada.
No elevar `UPSERTED` o `SYNC` a garantía transaccional universal.
No mutar variables de entorno globales para coordinar librerías del motor.
No activar inferencia LLM para fusionar listas que Vanta ya puede fusionar.
No arrastrar evaluadores/jueces del producto como criterio de aceptación Vanta.
No recomendar dependencia estratégica ignorando el archivo del repositorio.

## 6. Límites, licencia, método y ledger

Lectura estática acotada del snapshot **archivado**; no auditoría completa de
Flowise, proveedores, flujos enterprise, auth, colas o integraciones.
Sin scripts, instalaciones, apps/tests/builds/benchmarks/servicios,
commits o push. La ampliación §11 añade consultas públicas autorizadas;
la condición offline y enlaces no navegados del borrador es histórica.
Clon superficial sin historia/otras ramas/tags; omisiones LFS del manifiesto.
LICENSE: enterprise y archivos con aviso explícito son comerciales; resto
Apache 2.0 salvo terceros. No sustituir ese reparto por etiqueta genérica MIT.

| ID | Evidencia inmutable y rangos leídos | Uso |
|---|---|---|
| S1 | [Retriever.ts:138-218](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/agentflow/Retriever/Retriever.ts#L138-L218) | Bucle de stores, estado y SSE |
| S2 | [DocStoreVector.ts:40-173](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/vectorstores/DocumentStoreVS/DocStoreVector.ts#L40-L173) | Config persistida e imports |
| S3 | [documentstore/index.ts:1489-1576](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/documentstore/index.ts#L1489-L1576) | Consulta, timing y reconciliación |
| S4 | [ReciprocalRankFusion.ts:1-96](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/retrievers/RRFRetriever/ReciprocalRankFusion.ts#L1-L96) y [RRFRetriever.ts:103-137](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/retrievers/RRFRetriever/RRFRetriever.ts#L103-L137) | Multi-query y fusión |
| S5 | [tracingEnv.ts:1-111](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/src/tracingEnv.ts#L1-L111) | Caché, precedencia y emisión |
| S6 | [DocumentStore.ts:1-43](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/database/entities/DocumentStore.ts#L1-L43) | Estado y configuraciones persistidas |
| S7 | [EvaluationRunner.ts:1-235](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/evaluation/EvaluationRunner.ts#L1-L235) | Runner real de aplicación |
| S8 | [evaluations/index.ts:160-196](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L160-L196) y [250-310](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L250-L310) | Persistencia y evaluadores |
| S9 | [DocumentStoreStatus.jsx:1-100](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/views/docstore/DocumentStoreStatus.jsx#L1-L100) y [EvaluationResultSideDrawer.jsx:1-115](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/views/evaluations/EvaluationResultSideDrawer.jsx#L1-L115) | Estado y detalle UI |
| S10 | [LICENSE.md:1-32](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/LICENSE.md#L1-L32) y [package.json:1-85](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/package.json#L1-L85) | Licencia mixta y paquetes |

Lectura adicional: `RRFRetriever.ts:1-102`, `packages/server/package.json:1-38`,
`packages/ui/package.json:1-36`; inventario de UI/evaluaciones/retrievers.
No inspeccionados todos los evaluadores simples/LLM, workers de predicción,
checkpointers, memory nodes, controladores de permisos ni frontend completo.
El código de evaluación no acredita resultados reproducidos.

## 7. Benchmarks/runners realmente leídos

S7 ejecuta evaluación por filas de dataset `{input, output, sequenceNo}` y
lista de chatflow IDs; llama `/api/v1/prediction/{id}` con request/evaluation ID.
Puede reutilizar `sessionId` configurado upstream: el runner no garantiza
aislamiento de memoria entre filas bajo cualquier configuración.
Recoge salida real, error, intervalo de cliente (HTTP más posprocesado/
awaits de coste) y métricas de tokens/proveedor/modelo;
estima coste con catálogo y métricas anidadas de Agentflow cuando existen.
S8 agrega pass/fail/error sólo de evaluadores simples efectivos y guarda
resultados LLM aparte; sin simples passPcnt=-1 aunque haya LLM. Su
averageLatency suma latencias fila×flow y divide por filas, no por requests
(fórmulas/poblaciones precisadas en §11.5).
No se inspeccionó su implementación/prompt completo: no se certifica el juez.
Persiste EvaluationRun con entrada, esperado, salidas, errores y métricas.
Dependencias: Flowise operativo, DB, dataset/chatflows configurados,
credenciales y proveedores que cada flow utilice; axios/Node y modelo si aplica.
Efectos: ejecuta flows reales, escribe evaluaciones y potencialmente historial,
estado/herramientas; puede gastar tokens/dinero y consultar servicios externos.
No se acredita rollback universal ni sandbox de efectos de los flows.
Latencia end-to-end no aísla índice, y coste estimado depende del catálogo.
Pass/fail configurable de aplicación no prueba recall ni memoria longitudinal.
No runner de microbenchmark RRF acreditado en esta lectura acotada.
No se creó harness, prompt/juez, threshold, configuración o sampling.
No ejecutado; comparación futura bajo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz capacidades Vanta

| Capacidad | Estado baseline | Diferencial a revalidar |
|---|---|---|
| BM25/HNSW/RRF/sparse | Existe | No copiar la fusión Flowise |
| Provenance/sesiones/supersession/dedup | Existe | Batches externos deben conservar identidad |
| Assembler/compaction | Existe | Presupuesto global, no texto concatenado sin recibo |
| Scheduler/checkpoints, ingestors/auditoría | Existe | No duplicar jobs/estado de aplicación |
| Studio conexiones/memoria/actividad/retrieval | Existe | Integración outcome por fuente: parcial por verificar |
| Precedencia observable única de trazadores | No acreditado | Revisar contrato actual antes de actuar |
| Plataforma visual/enterprise en core | No aplicable | Adaptadores y UI opt-in |

## 9. Deuda sugerida sin falsos bugs

¿Toda fusión conserva IDs y revisión por fuente?, ¿se ven resultados parciales?,
¿desactivar un exportador tiene una precedencia única?, ¿config efectiva es
inspeccionable sin secretos? No son bugs demostrados de Vanta.
Los fallos lógicos observados en Flowise son advertencias para no copiar
algoritmos por reputación; no invalidan ni reabren RRF existente de Vanta.
No se crean tareas ni se atribuyen ausencias a HEAD sin lectura autorizada.

## 10. Interfaz/escritorio e ideas Studio

**Código verificado:** UI React/MUI de estado documental distingue STALE,
EMPTY, SYNCING, UPSERTING, SYNC, UPSERTED y NEW; drawer de evaluación muestra
entrada, esperado y salidas por flow. Colores/status no certifican durabilidad.
Consulta diagnóstica S3 devuelve docs/timing para inspección, no calidad medida.
No se localizó shell Electron/Tauri propio en inventario acotado; frontend web
no equivale a escritorio nativo. No se ejecutó UI ni auditoría de accesibilidad.
Para Studio: origen/revisión por resultado, estado parcial por fuente y
config efectiva redactada en actividad/RetrievalLens; no una consola nueva.
Conservar Tauri/React y no importar canvas/marketplace a la distribución embebida.

## 11. Dependencias, cobertura y separación futura

### 11.1 Recursos, APIs y tenancy: comprobación de las trazas F1–F8

Continuación de autoría **2026-11-07**, no revisión independiente. Se preservan
S1–S10 y A–D y se completan sus fuentes. Upstream pin
`9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb`, Vanta base fija
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no actualizar clones/manifiestos.

| Recurso | Identidad/owner y operación real | Frontera que no debe confundirse |
|---|---|---|
| Canvas v2 | ReactFlow `Canvas.handleSaveFlow` serializa nodes/edges/flowData, limpia status/selección, mueve referencia de credencial [F5] | Guardar definición no ejecuta ni aprueba herramientas |
| ChatFlow | UUID, workspaceId, tipo y configuración persistida [F8] | ID de flow no es revisión inmutable ni identidad de usuario |
| Prediction | `/api/v1/prediction/:id`, multipart/rate limiter y validación externa de clave [F1/F8] | Clave del flow no identifica necesariamente cada usuario final |
| Cola | BullMQ job/Redis events + worker inyecta servicios [F2] | Job name no demuestra dedup ni ID de operación durable |
| Execution | UUID, flow/workspace/session, executionData/state [F3/F8] | Snapshot de runtime no es perfil de hechos ni replay exactly-once |
| ChatMessage | UUID, flow/chat/session, output/tools/sourceDocuments/execution opcional [F8] | Historial distinto de state y vector store |
| DocumentStore | Config/chunks/upsert history local y vector provider externo [F6/S6] | UPSERTED no prueba visibilidad/durabilidad de todas proyecciones |
| Memoria conversacional | `BufferMemoryExtended.getChatMessages` por session+flow, ASC [F4] | add/clear delegados al servidor, no una lista sólo en RAM |

**Tenancy concreta:** `utilBuildChatflow:990-1095` carga flow por ID, obtiene
chatId de `incomingInput.chatId ?? overrideConfig.sessionId ?? UUID`; valida
API key para externo y deriva workspace→organization→subscription/product,
comprueba cuota de predicciones y construye executeData. Esto no equivale a
autorizar user final a todo recuerdo: sessionId/control de texto son datos,
no actor autenticado. `executeAgentFlow:1664-1715` busca últimas ejecuciones
por **sessionId+agentflowId+workspaceId** y sólo restaura state si
`startPersistState===true`. Se comprueba ese aislamiento del query, no toda
cadena de auth ni aislamiento de cada proveedor remoto.

`deleteVectorStoreFromStore:399-470` sí filtra DocumentStore por
`id+workspaceId`, exige configs embedding/vector/record manager y llama delete
únicamente si método existe. Ausencia de método no equivale a purga confirmada.
`routes/documentstore/index.ts` añade checkPermission/checkAnyPermission a
operaciones específicas; el middleware de upsert/refresh tiene otra cadena.
Son referencias a RBAC **enterprise comercial**, no permiso para copiarlo ni
evidencia de que todas las rutas públicas tengan idéntica autorización.
No declarar fuga por ver un helper sin workspace si outer flow lo resuelve;
tampoco deducir seguridad global por encontrar un checkPermission.

**Handoff/resume y cancelación:** la pausa humana recupera Execution previa,
admite STOPPED o ERROR inmediatamente posterior a STOPPED, valida startNodeId
existente y actualiza INPROGRESS [F3:1776-1868]. No hay CAS de todo grafo
acreditado en ese bloque; definición editada puede requerir revisión explícita
del host. Dos continuaciones/copias son riesgo condicionado a exclusión externa.
El bucle comprueba signal abort, clasifica error por substring `"Aborted"`
como TERMINATED y el resto ERROR, persiste datos/estado y emite SSE
[F3:2041-2042,2164-2235]. No trasladar esa clasificación textual a un contrato
de cancelación Rust. Pausa STOPPED no es memoria aprendida ni fallo terminal.
Tool ya aplicada, DB y Redis tienen commits independientes: reinicio debe
reconciliar, no repetir side effect ni afirmar rollback porque desconectó SSE.

**Stack/licencia:** servidor Express/TypeORM y BullMQ/Redis en modo queue,
nodos TypeScript/LangChain/modelos/proveedores opcionales; UI React/MUI/Redux/
ReactFlow, pnpm/Turbo. SDKs/modelos/credenciales dependen de cada nodo, no hay
un modelo/embedding default universal de Flowise. `LICENSE.md:1-8` excluye
enterprise y archivos con aviso explícito (ejemplo IdentityManager) del
contenido Apache-2.0, y conserva licencias de terceros. Estudiar fronteras no
autoriza traducir expresión comercial a Rust. Cloud mencionado en issues es
un despliegue del reportante, no código SaaS completo inspeccionado ni desktop.

### 11.2 Complemento contractual A–D y WriteReceipt

Las firmas de A–D permanecen **PROPUESTAS**, nunca API de Vanta ya implementada.
Se añaden firmas Rust sobre tipos conceptuales del adapter, no otro contrato
activo ni nuevos tipos de motor:

```rust
fn retrieve_scoped(req: AuthorizedRetrieval, sources: &[SourceRevision],
    cancel: &CancelToken) -> Result<Vec<SourceRetrievalBatch>, RetrievalAdapterError>;
fn resolve_observation(user: &UserObservationChoice, env: &EnvironmentSnapshot,
    defaults: &ObservationDefaults) -> Result<EffectiveObservationConfig, ObservationConfigError>;
fn inspect_binding(scope: &AuthorizedScope, run: &RunId)
    -> Result<RunMemoryBinding, BindingError>;
fn resume_existing_task(binding: RunMemoryBinding, expected: Revision,
    cancel: &CancelToken) -> Result<ResumeDisposition, BindingError>;
fn inspect_mutation(scope: &AuthorizedScope, operation: &OperationId)
    -> Result<MutationReceipt, MutationInspectionError>;
fn reconcile_projection(receipt: &MutationReceipt, projection: &ProjectionId,
    cancel: &CancelToken) -> Result<ProjectionObservation, ReconciliationError>;
```

Los inputs conservan actor autorizado, identidad/revisión/config y op_id
estable; outputs son owned para sobrevivir al request. Error de transporte
no transforma Applied en no aplicado. `RunMemoryBinding.checkpoint_revision`
es propuesta, no afirmación de CAS implementado en TaskCheckpointManager.

**Corrección indispensable:** Vanta **ya tiene `WriteReceipt`** en FV5,
versión/contenido/store/WAL y límites declarados. Integridad por SHA-256 y
relectura, **no firma digital**, no garantía remota ni prueba de todos los
derivados. `MutationReceipt` D es **candidato transversal C13 PROPUESTO**:
extender respuesta del adapter sólo donde falta evidencia por dimensión;
no sustituir recibo existente ni crear otro WAL/ledger/worker.

| Ficha | Mínimo verificable / relación / decisión abierta |
|---|---|
| A resultados por fuente | Envelope hasta assembler existente; no nuevo RRF. D sólo si promete revisión visible, B no bloquea recall. Abierto: política del host para fuentes parciales, no elegir scoring nuevo |
| B observación efectiva | Resolver/reporte redactado por revisión antes de exporter; puede existir sin A/C/D ni red. Abierto: si cambio admite reload o requiere reconectar, no reiniciar servicios aquí |
| C binding | Referencias run/task/session/definición, no snapshot de todo runtime. Usa D para efectos inciertos; A para evidencia y B para diagnóstico. Abierto: coordinación del único writer ya exigida por checkpoint |
| D recibo | Unknown por dimensión hasta evidencia real, consulta proveedor por ID antes de retry. Depende de provenance/auth y checkpoint, no canvas. Abierto: qué proveedores soportan inspección de revisión/read-after-write |

**Aceptación futura normal/adversa, complementaria a §2:** A con fuente vacía
y otra fallida conserva outcomes distintos; score igual no colapsa documentos;
cancelación preconsulta no fabrica negativo. B con env/UI en conflicto declara
origen y owner efectivo; secreto no aparece en inspección; error de exporter
no aborta búsqueda local. C reanuda mismo task/definición autorizada, rechaza
definición vieja/otro tenant y no trata borrar chat como cancelación. D distingue
ack, primario aplicado, durabilidad acreditada y proyección visible; timeout
después de commit conserva op_id y no duplica. No hay pruebas ejecutadas.

Persistencia/migración/rollback conservan secuencias §2: metadata aditiva en
adapter/checkpoint/actividad existente, legacy Unknown/no resumible si falta
revisión; sin reindexar salvo cambio real de chunking/modelo. Quitar adapter
o exporter no borra hechos y no deshace efecto remoto. No crear un segundo
scheduler para conciliar; reutilizar infraestructura actual una vez aprobada.
Métricas de diagnóstico propuestas: IDs autorizados, revisión, etapa,
conflicto/partial/cancelled/unknown y tiempos por frontera; **sin payload**,
prompts, valores de variables ni credenciales por defecto.

### 11.3 E — Binding autorizado por recurso/transporte

1. **Problema agente:** misma API visual/API externa elige session distinta,
   o confunde API key de flow con identidad del usuario que puede recordar.
2. **Unidad/fuente:** `utilBuildChatflow` F1:990-1095,
   `BufferMemoryExtended.getChatMessages` F4:102-146 y routes F8:8-79;
   no se copia RBAC comercial ni routing Express.
3. **Delta:** negociación de capacidades y mapping de identidad en adapter,
   no IAM ni sesiones nuevas. Completa C, que enlaza ejecución, no autoriza.
4. **Alternativas:** DB por tenant reduce shared-state pero multiplica gestión;
   filtro por session sin actor no basta. Preferir política externa existente.
5. **Rust PROPUESTO:** `authorize_memory_binding(actor: &AuthenticatedActor,
   resource: MemoryResourceRef, transport: TransportCapability,
   action: MemoryAction) -> Result<AuthorizedMemoryBinding, BindingAuthorizationError>`.
   Salida scope/session/recurso autorizado/versión política; errores
   MissingIdentity/ScopeDenied/UnsupportedAction/LegacyUnmapped.
   sessionId del JSON o del LLM nunca crea permiso.
6. **Seam REAL:** `auto_recall.rs::InjectionPolicy/RecallConfig`, tenancy
   `l1_writer.rs::default_tenancy/plan_write`, bindings y adapters LangChain
   existentes FV1/FV4; no inventar `MemoryScope` implementado.
7. **Pasos:** inventariar actor/recurso/transporte→resolver política→validar
   acción antes de recall/mutación→mapear namespace/sesión→conservar decisión
   en reporte. Legado sin owner se rechaza o resuelve explícitamente, nunca
   migrar a público. Desactivar transporte revoca acceso sin borrar primario.
8. **Dependencias:** política/auth del host y scopes existentes; C consume
   binding, A/D comparten decisión; sin modelo y sin dependencia de queue.
9. **FMEA:** UI y API usan distinta session→pérdida aparente→mostrar mapping
   redactado; mismo sessionId dos tenants→cruce si falta actor→autorización
   previa; clear no soportado→silencio/no-op→UnsupportedAction. Detectar
   conteos/denegaciones/IDs seguros, no contenidos de cada tenant.
10. **Aceptación futura:** UI/API mapeadas deliberadamente a misma sesión ven
    mismo historial; sesiones distintas no se unen por texto; actor revocado
    no borra ni consulta; cancelación postcommit no reautoriza ni revierte.
11. **Mínimo/no-goals/abierto:** mapping/capacidades en adapter; no IAM, runtime
    ni scheduler. Abierto: actor efectivo del API consumer y compartición
    explícita entre agentes, no una política inferida del canvas.

### 11.4 Incidencias públicas y diferencia histórico/pin

Leídas el **2026-11-07** por HTML GitHub, después de consultas DDG/Bing sin
resultados útiles y fallback al buscador de issues exitoso:

| Incidencia y estado observado | Evidencia y causalidad | Decisión aplicable |
|---|---|---|
| [#4968](https://github.com/FlowiseAI/Flowise/issues/4968), v2 Agent Memory API Prediction - Session ID not Persistent; abierta 2025-07-29, Closed 2025-07-31 | Autor inicialmente reporta olvido en API; luego reconoce configuración y muestra `overrideConfig.sessionId` que funciona. No versión/SHA exactos del entorno | **Falso positivo descartado como bug**. F1/F3 en pin usan overrideConfig.sessionId. E muestra mapping, no «arreglar persistencia» ni atribuir PR inexistente |
| [#4433](https://github.com/FlowiseAI/Flowise/issues/4433), State Variable Set by LLM Node Is Lost in Subsequent Messages; abierta 2025-05-15, Closed observado | Reportante Cloud; comentario mantenedor 2025-05-21 describe state runtime y sugiere variables. Otros distinguen downstream del mismo run frente a turno posterior; versión/SHA no declarados | Pin F3 **sí** restaura state si Start persist=true. No usar frase histórica «siempre efímero» como hecho actual. Relación #4512 visible, merge/diff no inspeccionado; no atribuir fix/version publicada |

De #4968 se leyó cuerpo e hilo visible completo. En #4433 se leyó cuerpo y
tramo de conversación hasta límite de extracción (17.000 caracteres); no se
presenta como lectura de todos los adjuntos/exportaciones ni de toda PR.
La distinción de períodos se acredita además por F3, no por inferencia de
cierre de issue. No se afirma fallo actual de persistencia ni vulnerabilidad.
La página pública confirma archivo del repo el 2026-08-13; metadata congelada
no se recalcula ni se presenta como mantenimiento vigente.

### 11.5 Runner de evaluación: población, defaults, errores y efectos

La lectura adicional de S7/S8 separa **evaluación de producto/QA** de
benchmark del motor y de pytest/unit tests:

- Entry point `EvaluationRunner.runEvaluations:86-119` valida chatflowId JSON
  como array y dataset.rows; crea una fila por input/expectedOutput/sequenceNo.
  Itera flows y luego filas secuencialmente en `evaluateChatflow:121-235`.
  No hay split/seed/warmup universal, modelo fijo ni tokenizer del runner;
  esos componentes pertenecen a cada flow. No inventar valores por defecto.
- Entrada efectiva a `/api/v1/prediction/{id}` incluye question, evaluation
  ID, cabecera/request ID y API key si existe; si data.sessionId está presente
  reutiliza **la misma** session en filas. No reset genérico de chat/state/
  memoria entre ellas. No es automáticamente evaluación aislada por pregunta.
- `actualOutput` elige text, luego json, luego serialización de response.
  Captura error por fila y agrega runData con status/error/latency aun fallando;
  no confundir salida vacía por error con abstención legítima.
- `EvaluationRunner.ts:135-180` inicia `performance.now()` antes del POST y
  termina **después** de procesar métricas Agentflow y de awaits
  `getCostMetrics`. Mide intervalo **del cliente**, no sólo HTTP, en **ms**;
  `toFixed(2)` produce string. Incluye ese posprocesado/coste además de red/
  API; no separa retrieval/LLM/cola/herramientas y no da p50/p95.
  Token metrics y coste estimado dependen de usage y catálogo del modelo;
  coste por 1000 tokens no es factura verificada ni coste total de infraestructura.
- Servicio `evaluations/index.ts:196-202` suma en `totalTime` **todas** las
  latencias emitidas por cada fila×flow. `321-325` calcula
  **averageLatency = totalTime / result.rows.length**, y `totalRuns` es
  `result.rows.length`: **suma media por fila**, no media por request.
  Añadir flows cambia la magnitud sin demostrar regresión del motor; errores
  de prediction con runData/latency también participan. Fallo global antes
  del retorno puede impedir el agregado; no suponer una latencia inexistente.
- Servicio `evaluations/index.ts:174-325` recorre resultados, acumula métricas,
  persiste `EvaluationRun` con input/expected/actual/errors y llama evaluadores
  simples/LLM según configuración existente. `270-315` alimenta
  passCount/failCount/errorCount **sólo** desde `runAdditionalEvaluators`/
  `additionalConfig.simpleEvaluators`. La rama LLM guarda su `resultArray`
  en **otro campo**, `newRun.llmEvaluators`, sin incrementar esos contadores.
  Se conserva **pass/(pass+fail+error)*100**, o **-1 sin casos simples**,
  incluso si existen resultados LLM. Población: **fila×flow×evaluador simple
  efectivamente emitido**, no pregunta única ni todos los jueces.
  `EvaluatorRunner.ts:13-51` produce Error por cada evaluador cuando el
  prediction falló; eso no equivale a una única pregunta fallida.
  No convertir passPcnt en recall@k, accuracy QA homogénea o score LLM.
- No se leyó implementación completa de cada evaluador/prompt ni todos los
  modelos de juez. Expected output se entrega al evaluador, no por ello se
  prueba fuga al retrieval; flujo completo necesita inspección discriminante.
  Tampoco se acredita que métricas de errores equivalgan a todas intentadas
  si fallo global sucede antes de devolver runEvaluations.

**Efectos y adaptación:** ejecuta flows reales, posibles tools/pagos/red,
escribe DB/logs/memoria/artefactos; borrar EvaluationRun no deshace efectos.
Dependencias Node/axios, Flowise/DB, dataset, flow, proveedores y credenciales;
queue puede añadir Redis/worker. No comandos de arranque, ejecución ni limpieza
en esta tarea. Adaptar sólo adapter de memoria/binding A/C/E con la misma
interfaz del flow y criterios existentes; usar evals Vanta vigentes bajo
METHOD/BENCHMARK-NOTES para pista retrieval y no portar evaluador de producto
al motor. No nuevos harnesses/prompts/rúbricas/scoring/umbrales/configs.
No se ejecutó ningún benchmark ni se compara un claim con medición propia.
F-1/F-2 son precisiones del método leído: conservar fórmulas, criterios,
poblaciones y fallos; no otra normalización, denominador, scoring ni harness.
Aceptación futura del adapter: preservar identidad fila/flow y distinguir
prediction fallido, Error simple, resultado LLM y fallo global; UI debe
explicar unidades/población sin presentar -1 como cero ni promedio por request.
La existencia de mediciones previas de Vanta Paso23 no se niega; su alcance
tampoco se amplía a certificación semántica o crash safety.

### 11.6 UI local real y oportunidad en Studio existente

Además de status/drawer S9, se leyó Canvas F5 y `packages/ui/src/api/chatflows.js:
3-55` (FU1): crear/actualizar flow llama POST/PUT `/chatflows`, consultar
definición GET, borrar DELETE. Canvas serializa **definición**, no persistencia
del runtime. ReactFlow dibuja nodos/edges y botones guardan antes de predecir;
no hay evidencia de que UI haga ACID un pipeline remoto. Editor Agentflow v2
no es un shell desktop. Cloud de #4433 es despliegue externo reportado;
el código UI local no prueba paridad con todo SaaS ni licencia de producto
alojado. No se ejecutó UI ni se evaluó interacción visual.

Vanta Studio: pin separado `0ad38c542b39a589035b0e17fdedb4c09dca24b9`,
cuerpos releídos con git show y rutas/rangos del commit, no del worktree:

- [RetrievalLens:1-130](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L130):
  filtros existentes, scores y callback onOpenRecord.
- [ActivityPanel:1-65](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L1-L65):
  audit paginado por namespace/op/outcome y estado no configurado.
- [MemoryLens:57-412](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412):
  ScenesPanel/PersonaPanel/SkillsPanel/GenlogPanel, no snapshot del runtime externo.

Oportunidad: mostrar por fuente parcial/vacía/fallida y source/config revision
en RetrievalLens; resultado mutación Applied/Unknown/visibilidad y binding
run/task/session en actividad/inspector. Política efectiva B redactada en
conexión/actividad, no token ni URL con secreto. No otro dashboard/canvas;
no tratar generation log como traza exacta de entrega de contexto al modelo.

### 11.7 Matriz upstream/Vanta con fuentes y decisiones

| Capacidad | Upstream estado/fuente | Vanta 97c9623 estado/fuente | Delta/decisión |
|---|---|---|---|
| Captura/conversación/state | Existen separados F3/F4/F8 | Existen sesiones/checkpoints/provenance FV2/FV4 | C referencia owners, no memoria=replay |
| Recall multi-store | Parcial S1/S3, sobreescritura por iteración | Híbrido/fusión ya existe FV3 | A outcome por fuente, no portar RRF defectuoso |
| Config de trazas | Existe S5 con caché/precedencia | Auditoría/diagnóstico existen, estado externo no acreditado | B resolución efectiva/redactada |
| Derivados y delete | Separados F6/F7, no commit distribuido | Provenance/supersession existentes FV4 | D concilia, no nuevo WAL |
| Scope/tenancy | Existe por recurso F1/F3/F6/F8; no auth universal probada | InjectionPolicy/tenancy existentes FV4 | E mapping de transport, no IAM nuevo |
| WriteReceipt | Status/upsert history no equivalen S6/F6 | **Existe**, FV5, SHA-256 sin firma | C13 sólo propuesta complementaria |
| Budget/compaction/quotas | Config por nodo/flow, no único contrato de motor | Existe, FV3/FV4 y VANTA-REFERENCE | Reutilizar presupuestos y quotas, no copiar límites de cola |
| Scheduler/checkpoints | Colas/runtime/Execution F2/F3 | Existe TaskCheckpointManager/scheduler, FV4 | Referenciar, no portar ejecutores |
| LangChain adapters | Nodos/integraciones Flowise | Existen store/checkpointer FV1/FV2 | Extender adapter antes de duplicarlo |
| UI | Web local F5/FU1/S9; Cloud externo | Studio Tauri/React existente | Ampliar lentes, no nueva aplicación |

**Negativos Vanta — alcance estricto:** no se acredita nuevo bug. Preguntas
abiertas: si todo adapter informa partial/outcome/config efectiva y si cada
binding expone revisión/owner. `TaskCheckpointManager` documenta un escritor;
su RMW no prueba una carrera efectiva sin caller/reproducción. Nunca atribuir
loss de state de issue 2025 al pin actual ni el error de configuración #4968
a Vanta. Se excluyen falsos positivos de ausencia de híbrido, dedup, budgets,
provenance, quotas, checkpoints y Studio. RecallConfig permite ruta core
configurable y presupuesto por caracteres, no tokens exactos; protegido del
assembler puede exceder presupuesto. No prometer equivalencia de todos caminos.

### 11.8 Ledger nuevo y cierre propio

F1–F8 completan las referencias abreviadas de §0/§2; todos al SHA Flowise fijado:

- [F1, utilBuildChatflow:990-1110](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/utils/buildChatflow.ts#L990-L1110), executeFlow `481-505`.
- [F2, PredictionQueue.processJob:65-107](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/queue/PredictionQueue.ts#L65-L107) y [BaseQueue.addJob/createWorker:5-105](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/queue/BaseQueue.ts#L5-L105).
- [F3, executeAgentFlow:1575-1715](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/utils/buildAgentflow.ts#L1575-L1715), [resume:1776-1868](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/utils/buildAgentflow.ts#L1776-L1868), [execute/error/status:2020-2235](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/utils/buildAgentflow.ts#L2020-L2235), mensajes finales `2363-2409`.
- [F4, BufferMemoryExtended:86-146](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/components/nodes/memory/BufferMemory/BufferMemory.ts#L86-L146).
- [F5, Canvas.handleSaveFlow:219-254](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/views/agentflowsv2/Canvas.jsx#L219-L254), ReactFlow `709-740`.
- [F6, DocumentStore delete catálogo:319-354](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/documentstore/index.ts#L319-L354), [delete vector:399-470](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/documentstore/index.ts#L399-L470), [insert/upsert:1270-1444](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/documentstore/index.ts#L1270-L1444), queue/upsert `1995-2077`.
- [F7, removeAllChatMessages/removeChatMessagesByMessageIds:110-180](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/chat-messages/index.ts#L110-L180).
- [F8, rutas documentstore:8-79](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/routes/documentstore/index.ts#L8-L79), [predictions:10-15](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/routes/predictions/index.ts#L10-L15); entidades [ChatFlow:5-75](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/database/entities/ChatFlow.ts#L5-L75), [Execution:5-47](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/database/entities/Execution.ts#L5-L47), [ChatMessage:6-73](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/database/entities/ChatMessage.ts#L6-L73).
- [FU1, API UI chatflows:3-55](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/ui/src/api/chatflows.js#L3-L55).
- [S8 ampliada, agregación:174-325](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/index.ts#L174-L325).
- [EvaluatorRunner.runAdditionalEvaluators:13-51](https://github.com/FlowiseAI/Flowise/blob/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb/packages/server/src/services/evaluations/EvaluatorRunner.ts#L13-L51): población simple, no resultados LLM.

Seams Vanta fijados, no nombres inventados:
[FV1, EmbeddingVectorStore:31-163](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/vantadb_shared/__init__.py#L31-L163),
[FV2, VantaDBCheckpointer:232-296](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L232-L296),
[FV3, assemble_with_recall:252-326](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L326)
y `src/sdk/search/fusion.rs::fuse_rrf_many`;
[FV4, persist_planned/log_and_mark:289-330](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L330),
`core/hooks/auto_recall.rs::InjectionPolicy/RecallConfig:133-335`,
`utils/task_checkpoint.rs::TaskCheckpointManager:112-246`;
[FV5, WriteReceipt/límites:678-745](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L745).

**Cobertura leída:** S1–S10 preservadas; F1/F2/F3/F4/F5/F6/F7/F8 y FU1
profundizan rutas críticas anteriores, no auditoría completa del monorepo.
Lectura nueva discriminante incluye BaseQueue entero hasta createWorker,
prediction tenant/queue, persistState/resume/error, BufferMemory, vector delete,
upsert-before-history/telemetry, Canvas/API y denominador de evaluaciones.
**No leído:** todas las funciones de buildAgentflow, todos los nodos,
seguridad enterprise integral, cada driver/record manager, middleware global,
LLM evaluators/prompts completos y adjuntos de issues. Fuente inaccesible/
truncada queda declarada, no sustituida por título como prueba.

**Cierre propio:** A–D conservadas/completadas y E agregada con once campos;
diez ejes documentados bajo límites estáticos. Se mantienen abiertos decisión
de auth por consumidor, atomicidad real de cada backend y validación dinámica
normal/adversa. No autoaceptación ni revisión independiente. Sólo lectura de
grafo Vanta; sin índices/código/contratos activos/backlog, apps/tests/builds/
benchmarks/instalaciones/servicios ni git writes. Runtime: nada iniciar/parar.
Por >300 líneas, una fase aprobada separaría specs de adapter/receipts,
binding/autorización y reporte de observación; aquí no se crean esos archivos.
