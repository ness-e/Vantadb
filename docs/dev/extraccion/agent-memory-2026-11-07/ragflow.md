---
title: "RAGFlow: compilación revisionada, publicación y evidencia servida"
kind: research
status: proposed
description: "Estudio estático del snapshot Go de RAGFlow, sus fronteras de ingesta, compilación, consulta y sincronización, y deltas propuestos para VantaDB."
tags: [research, memory, engineering]
---

# RAGFlow: compilación revisionada, publicación y evidencia servida

## Índice

- [0. Identidad, arquitectura y trazas](#0-identidad-arquitectura-y-trazas)
- [1. Recomendaciones](#1-recomendaciones)
- [2. Fichas Rust propuestas](#2-fichas-rust-propuestas)
- [3. Incidencias públicas y seguridad](#3-incidencias-públicas-y-seguridad)
- [4. Contratos previos en VantaDB](#4-contratos-previos-en-vantadb)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks leídos](#7-benchmarks-leídos)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Hallazgos negativos y falsos positivos](#9-hallazgos-negativos-y-falsos-positivos)
- [10. UI real y Studio](#10-ui-real-y-studio)
- [Fuentes, cobertura y cierre](#fuentes-cobertura-y-cierre)

## 0. Identidad, arquitectura y trazas

Repositorio `infiniflow/ragflow`; clon congelado `C:\Users\Eros\ragflow`.
SHA **`3c6d2255af06164f0818d0959af10de0afd8b8fa`**, observado por el
[manifiesto](clone-manifest.json) el 2026-11-07T06:56:24.9769235Z.
No se hizo fetch, reclone ni actualización. Licencia raíz Apache-2.0
([texto](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/LICENSE#L1-L16)).
No extiende automáticamente esa licencia a modelos, bibliotecas nativas,
servicios de proveedores ni implementación enterprise ausente.

Se recuperó el antecedente mediante `git show HEAD:<ruta>`; se conservan sus
hallazgos de leases y evidencia compilada, pero se amplían sincronización,
errores de backend, runner y frontend. No se atribuye revisión a autores previos.
Referencia Vanta: **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**, no HEAD móvil;
[VANTA-REFERENCE](VANTA-REFERENCE.md) delimita lo ya implementado.

### Stack, despliegue y mapa

El [módulo](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/go.mod#L1-L65)
declara Go 1.27. No es correcto describir este snapshot como el antiguo backend
Python: el camino examinado es Go. Python sigue apareciendo en tooling/SDK y
benchmark; presencia de un runner no significa que sea el servidor productivo.

| Área | Responsabilidad y dependencia |
|---|---|
| `cmd/`, `internal/server/`, `internal/router/` | Bootstrap y superficies API/admin/ingestor/syncer; inventario, no auditoría de todas las rutas |
| `internal/ingestion/component/` | Parser/chunker/tokenizer/extractor y compilación documental |
| `internal/ingestion/knowledge_compile/` | Backlog, claim por dataset, merge, heartbeat y ACK |
| `internal/service/document/` | Reparse, disponibilidad y mutaciones sobre productos |
| `internal/engine/`, `internal/storage/` | Proyecciones de búsqueda y objetos; semántica dependiente del backend |
| `internal/agentic_rag/` | Selección de herramientas y evidencia compilada para el agente |
| `internal/syncer/connector/` | Enumeración externa y reconciliación de fuentes |
| `web/` | React/TypeScript/Vite; estado y vistas de compilación |
| `test/benchmark/`, `internal/cli/benchmark.go` | Medición HTTP y wrapper de comandos, respectivamente |

SQL conserva verdad de scheduling; NATS notifica datasets, no transporta el
único registro durable. El despliegue puede incluir almacenamiento de objetos,
Kvrocks y Elasticsearch/Infinity/OceanBase u otros motores: esta inspección
no demuestra equivalencia de capacidades entre ellos.
Parsing nativo, modelos OCR, embeddings y LLM son costes distintos; compilación
generativa no convierte cada escritura o consulta en una operación LLM obligatoria.

### Modelos y fronteras de ownership

- [Document](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/entity/document.go#L24-L88):
  ID de documento, dataset, progreso/mensaje, contadores y disponibilidad.
  Progreso de parser, ingesta y disponibilidad no son el mismo booleano.
- [BacklogEntry/Claimer](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/knowledge_compile/scheduler.go#L56-L153):
  evento por documento, variantes, límite congelado del batch y token de claim.
  Último evento por posición para un documento prevalece; no es un reloj global.
- [KnowledgeCompilerComponent.Invoke](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/component/knowledge_compiler/component.go#L63-L130):
  tenant/dataset desde globals y chunks de entrada; productos regresan como
  chunks. No hay justificación para crear otra base paralela en Vanta.
- [compiledRun](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/agentic_rag/compiled_evidence.go#L115-L203):
  tenant, datasets, probe y conjunto `shown` protegido con mutex por ejecución.
  Estado del turno, no memoria personal persistente ni checkpoint de tarea.

### Traza de éxito: ingesta → tarea → índice → consulta

1. Fuente sincronizada o documento cargado entra al pipeline de ingesta.
   `Invoke` compila por documento y devuelve productos tipados como chunks.
2. Se indexan productos y se publica evento de merge de dataset.
   [scheduler.go:36-166](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/knowledge_compile/scheduler.go#L36-L166)
   separa append durable, notificación y reclamación.
3. `processClaim` toma batch cerrado; heartbeat renueva token. Serialización por
   dataset permite datasets distintos en paralelo.
4. El contrato `WithDatasetLock` exige validar token y efectuar escritura
   destructiva bajo el mismo lock; comprobación previa suelta no basta.
5. [consumer.processClaim:220-310](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/knowledge_compile/consumer.go#L220-L310)
   ACKea solo tras éxito del batch. Eso no acredita transacción distribuida
   atómica entre SQL, objetos y motor de búsqueda.
6. Consulta agentica usa disponibilidad de productos para habilitar herramientas;
   tener árbol o timeline no implica que existan claims.
7. Evidencia compilada acompaña resultados ordinarios; `takeUnseen` omite texto
   ya servido y conserva handles. [compiled_serve.go:366-449](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/agentic_rag/compiled_serve.go#L366-L449)
   contiene sustitución/anexo del contenido enviado.

### Actualizar, retirar, cancelar y fallar

[knowledge_compile_status.go:32-118](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/service/document/knowledge_compile_status.go#L32-L118)
propaga enable/disable y busca derivados incluyendo no disponibles, por
tenant/dataset/doc. Borrado lógico, borrado físico y reconstrucción no se equiparan.
`CancelInflight` invalida token y vuelve pendiente el batch; rebuild se
serializa contra escrituras por dataset. El consumidor que pierde heartbeat
espera su goroutine y no ACKea; no promete interrupción inmediata del proveedor.

Si falla merge/reader/writer, el backlog queda reclamable; no se marca éxito.
[startDatasetKnowledgeCompile:423-453](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/service/ingestion_service.go#L423-L453)
puede desactivar consumidor por falta de MQ/provisioning: documento compilado
no significa dataset compilado. [doc_state.go:29-97](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/service/doc_state.go#L29-L97)
trata metadata/contadores best-effort; fallo de lectura aborta merge de metadata
para no perder claves. Un contador retrasado no prueba pérdida del primario.

## 1. Recomendaciones

| Orden | Delta propuesto | Motivo y condición |
|---|---|---|
| Primero | Publicación cercada de derivados | Reusar wiki/scheduler; impedir publicación de trabajo retirado |
| Primero | Enumeración completa antes de prune | Issue destructivo concreto; no confundir ingesta best-effort con snapshot completo |
| Después | Recibo de evidencia efectivamente servida | Dedup semántico ya existe; falta acreditar este contrato por turno |
| Después | Capacidades por producto y revisión | Evitar ofrecer herramienta sin evidencia; no convertir fallo de probe en ausencia |
| Después | Estado reconciliable de ingesta | Mostrar aceptación/proyección/diagnóstico por separado, no porcentajes inventados |

Prioridades de investigación, no IDs oficiales, tareas aprobadas ni promesas de
recall/coste. Los cinco mecanismos se relacionan pero tienen invariantes distintos.

## 2. Fichas Rust propuestas

Todos los nombres de tipos/operaciones de esta sección son **propuestos**.
No se implementó API, migración, modelo, prompt, peso ni evaluador.

### A. Publicación cercada de derivados

**Caso/unidad:** worker termina compilación de fuente R después de retirada o
rebuild. Unidad: claim + comprobación actual + publicación; no portar SQL/NATS.
**Evidencia:** `Claimer.WithDatasetLock/CancelInflight` y `processClaim`, rangos
SHA enlazados en §0. Heartbeat no sustituye fencing al escribir.
**Delta/seam real:** `src/wiki/store.rs::WikiStore::{begin_processing,complete,
put_page,delete}`, `src/wiki/state.rs::WikiState`, scheduler `run_pass` y
`TaskCheckpointManager` existentes; no duplicarlos.
**Contrato:** `DerivedPublication { source_id, source_revision, generation,
claim_token, artifact_refs }`; `publish_if_current` → `Published|Superseded|
Withdrawn`, error `LeaseLost|StoreError`. Scope resuelto antes de aceptar trabajo.
**Pasos:** inventariar revisionado actual; enlazar derivados a fuente/generación;
persistir intención en checkpoint existente; validar y publicar bajo una
frontera de exclusión; hacer ACK después; reconciliar restos no publicados.
**Fallos/concurrencia/commit:** cancelación antes de commit deja reclamable;
después de commit requiere recibo idempotente. Lease vencido no publica.
No mantener lock del motor durante llamada LLM; recalcular elegibilidad al volver.
**Persistencia/deps:** IDs y tombstones, transacción o CAS real del propietario,
rollback de proyección por generación; scheduler de Vanta, no broker obligatorio.
**Relaciones/alternativas:** comparte fence con Quivr, no el DTO de observaciones;
primera alternativa extender estado wiki; rechazar almacén canónico duplicado.
**FMEA:** retiro durante inferencia → escritor tardío → resurrección; detectar
revisión/token incompatibles y rechazar antes del cambio visible.
**Aceptación manual futura:** retiro antes/durante commit, lease perdido,
dos workers y replay producen un único estado vigente; logs solo IDs/estado.
**No objetivos/abierto:** exactly-once distribuido o port completo de compiler;
decidir retención de artefactos huérfanos con el propietario.

### B. Recibo de entrega de evidencia por turno

**Caso/unidad:** búsquedas sucesivas devuelven el mismo claim; evitar repetir
texto sin negar una revisión nueva. Unidad upstream: `compiledRun.takeUnseen`.
**Evidencia:** `compiled_evidence.go:115-203`, `compiled_serve.go:366-449`, SHA en §0.
**Delta/seam:** `vanta-memory/src/context_engine/engine.rs::{assemble_with_recall,
IntegratedContext}` y reporte existente. `shown` previo al truncado no prueba
entrega efectiva; mejorar contrato, no copiar el momento del marcado ciegamente.
**Contrato:** `EvidenceReceipt { run_id, source_id, revision, fragment_id,
selection_origin, inclusion }`; reservar → admitir → confirmar incluido.
Salida `Included|Duplicate|BudgetOmitted|Revoked`, error de referencia separable.
**Pasos:** comprobar reporte actual; resolver vigencia/ACL; dedup por identidad
revisionada; aplicar presupuesto compartido; confirmar solo contenido conservado;
conservar handle sin texto y cerrar estado al terminar turno.
**Fallos/concurrencia/commit:** reservas concurrentes necesitan ganador por
fragmento; cancelar libera reserva no confirmada. Retry usa run/fragment/revisión,
no un set global. Persistir recibo opcional no bloquea respuesta por defecto.
**Deps/relaciones:** provenance, InjectionPolicy, estimador; A define vigencia,
AnythingLLM aporta distinción pin/historial, no otra memoria persistente.
**Alternativas:** enriquecer `IntegratedContext`; adapter de auditoría; rechazar
un ledger global con texto sensible o duplicación de generation log.
**FMEA:** marcado antes de compactar → segunda consulta omite evidencia nunca
vista; detectar diferencia selección/inclusión y confirmar tras assembly.
**Aceptación futura:** repetición misma revisión mantiene handle sin duplicar;
revisión nueva reaparece; omisión por presupuesto no se cuenta como entrega;
cancelar un turno no afecta otro. Métricas por motivo sin payload por defecto.
**No objetivos/abierto:** exactitud universal de tokenizador o garantía de
lectura humana; decidir si recibo es efímero o retenido para auditoría.

### C. Gate de capacidades sobre productos concretos

**Caso/unidad:** dataset contiene árbol pero no claims; una herramienta de
claims carece de entradas aunque el banner diga «compilado».
**Evidencia:** [compiled_knowledge.go:27-151](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/agentic_rag/compiled_knowledge.go#L27-L151),
`datasetsHaveClaims` hace probe de existencia y retorna false si falla.
**Delta/seam:** wiki store y herramientas MCP existentes; incorporar detalle
de capacidad al diagnóstico del assembler/Studio, no registrar otro motor.
**Contrato:** `ProductReadiness { scope, generation, product_kind,
state: Ready|Absent|Building|ProbeFailed }`; consultar → autorizar → ofrecer.
Inputs no aceptan tenant proporcionado por el modelo como autorización.
**Pasos:** mapear variantes realmente consumidas; leer manifiesto revisionado;
cachear solo durante snapshot del turno; invalidar por publicación/retirada;
exponer `ProbeFailed` separado. Estado inicial desconocido no es listo.
**Fallos/concurrencia/commit:** probe cancelado no fija ausencia persistente;
revisión de readiness debe coincidir con lectura posterior; cierre del gate
durante consulta produce resultado revocado o retry, no evidencia antigua.
**Deps/alternativas:** A y validación de refs; extender respuesta de wiki antes
que otra tabla. Probe directo es alternativa simple cuando no hay manifiesto.
**Persistencia/migración/rollback:** C es estrictamente efímera por turno y
generación; no nueva tabla, formato de disco, migración ni reindexación.
El caller posee cache/probe y su cancelación; no compartir resultados entre
principales o revisiones. Desactivar la cache vuelve al probe autorizado
directo, nunca a habilitar capacidades sin evidencia (fail-open).
**FMEA:** cached false después de fallo → herramienta oculta indefinidamente;
detectar estado/error y limitar identidad al turno/revisión.
Readiness de otra revisión → herramienta sobre evidencia retirada → comparar
generación al hidratar; error de probe no autoriza ni revela existencia ajena.
**Aceptación futura:** árbol sin claims no anuncia claims; error de backend es
distinguible; tenant diferente no comparte readiness; no requiere LLM.
Cancelación, retirada concurrente y desactivar cache conservan el gate y no
persisten un falso Absent; métricas por estado/código sin corpus/payload.
**No objetivos:** scheduling adaptativo, catálogo de skills alternativo,
inventar capacidades enterprise ni conservar los defaults upstream.

### D. Prune condicionado a una enumeración completa

**Caso/unidad:** sitemap hijo devuelve 503 y documentos aún existentes quedan
fuera del retain set. Una enumeración parcial es útil para ingesta, peligrosa
para borrar. **Evidencia:** [OpenPrune:181-226](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/syncer/connector/sitemap.go#L181-L226)
y [listEntries:265-305](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/syncer/connector/sitemap.go#L265-L305):
error de fetch anidado se registra y retorna nil; el snapshot puede ser parcial.
**Delta/seam:** ingestors y `WikiStore::delete` existentes; no se acreditó que
Vanta tenga prune por sitemap ni que padezca este defecto.
**Contrato:** `SourceInventory { source_revision, seen_ids, completeness }`,
`completeness: Complete|Partial { diagnostics }`; `plan_prune` solo acepta
snapshot completo y revisión aún vigente; salida plan autorizado, no borrado LLM.
**Pasos:** identificar cualquier reconciliador actual; separar batch incremental
de cierre de inventario; conservar errores de subárbol; sellar snapshot; validar
revisión al aplicar tombstones; permitir vacío completo, rechazar vacío parcial.
**Fallos/cancelación/commit:** timeout o cancelación no cierra snapshot;
dos enumeradores no mezclan retain sets; commit de bajas ligado a una revisión
sellada. Cancelar aplicación conserva checkpoint y plan idempotente.
**Deps/relaciones:** A impide que derivadores revivan bajas; permisos del
conector, límites SSRF, journal/checkpoint. No depende de C o de un proveedor LLM.
**Alternativas:** no borrar automáticamente; confirmación manual del plan;
reimplementar contrato genérico, no copiar walker ni habilitar sync_deleted.
**FMEA:** subárbol inaccesible → falso eliminado → pérdida; detectar diagnóstico
de incompletitud y bloquear fase destructiva, no interpretar warning como éxito.
**Aceptación futura:** hijo 503 no elimina; hijo vacío válido sí propone bajas;
root 503 no aplica nada; replay del plan no duplica; reporte sin URLs sensibles.
**No objetivos:** crawler general ni mecanismo de retención ilimitada; falta
decidir qué fuente acredita inventario completo en cada ingestor real.

### E. Estado reconciliable de proyección y diagnóstico

**Caso/unidad:** tarea finaliza pero contadores/metadata fallan o merge no tiene
consumidor; UI no debe presentar todo como «listo».
**Evidencia:** `doc_stateUpdater.apply/mergeDocMetadata`, `doc_state.go:29-97`,
`startDatasetKnowledgeCompile:423-453` y `Document:24-88` enlazados en §0.
**Delta/seam:** `WikiStore::{request_ingest,begin_processing,complete,fail}`,
`TaskCheckpointManager`, ActivityPanel existente, no una segunda cola.
**Contrato:** `IngestObservation { run_id, source_revision, primary_state,
projection_state, counters_state, diagnostic_code }`; lectura de estado y
reconciliación idempotente. Error del contador no cambia éxito del primario.
Cuando describe mutaciones, referencia por `operation_id` al
[C13 MutationReceipt propuesto](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén),
sin reemplazarlo. `primary_state` no acredita commit, fsync ni readiness:
aceptación, commit primario, durabilidad y proyecciones siguen independientes.
Commit confirmado/durable con proyección pendiente y espera vencida conserva
ambos hechos; sin evidencia postdispatch el outcome es Unknown, no rollback.
**Pasos:** acordar dueño de cada estado; enlazar observación con checkpoint;
persistir transición crítica; emitir diagnóstico best-effort separado;
reconciliar proyección con revisión actual y permitir desactivar vista nueva.
**Fallos/concurrencia/commit:** actualizar con revisión esperada; un evento
retrasado no revierte estado terminal de otra generación; cancelar conserva
causa de espera interrumpida, no fabrica `Stopped` del trabajo: tras dispatch
el resultado de la mutación sigue Unknown salvo evidencia. No hacer RMW de
metadata sin exclusión acreditada. El worker/ingestor posee transición crítica
y followup; el caller sólo su espera y observación, nunca el commit ajeno.
**Deps/alternativas:** A y reporter actual; primera alternativa exponer campos
existentes. No elevar fallos de telemetría a pérdida de ingestión.
Persistencia aditiva al estado/checkpoint existente, versión explícita y campos
ausentes=desconocidos; sin nueva cola. Desactivar vista mantiene evidencia y
no revierte commits ni suprime obligaciones del owner de la proyección.
**FMEA:** contador stale → falso «vacío» → reingesta duplicada; detectar
inconsistencia primario/proyección y ofrecer reconciliación, no reset destructivo.
**Aceptación futura:** caída de actualización de contador conserva dato;
consumidor ausente visible; eventos fuera de orden no cambian generación;
commit primario durable con proyección pendiente conserva ambos hechos;
cancelación postdispatch no significa rechazo/rollback, y reabrir observación
no reingesta ni duplica trabajo; campo legado ausente permanece desconocido;
logs contienen códigos/IDs, no contenido del documento.
**No objetivos/abierto:** porcentajes inventados o certificado de durabilidad;
elegir qué transiciones ya cubren reportes antes de crear persistencia nueva.

## 3. Incidencias públicas y seguridad

Consulta pública **2026-11-07**, lectura del cuerpo y metadatos por API GitHub.
`websearch.search` y `metasearch.search_web` (DDG/Bing) devolvieron cero resultados
para consultas `site:github.com/infiniflow/ragflow/issues delete documents`;
fallback GET `https://api.github.com/search/issues?q=repo:infiniflow/ragflow+is:issue+delete+document`.
La ausencia de resultados del buscador no se trató como ausencia de incidencias.

| Fuente y estado observado | Reporte / causa y alcance |
|---|---|
| [#21186](https://github.com/infiniflow/ragflow/issues/21186), abierto; creado 2026-10-19 | «A failed child sitemap can cause pruning to delete existing documents». Reporta SHA `7eb673bfc3ab1cc22532fe23a1e9bb85c34e48bd`, `sync_deleted_files=true` — desactivado por defecto — y reproducción de frontera con dobles, no borrado en KB viva |
| [#22082](https://github.com/infiniflow/ragflow/issues/22082), abierto; creado 2026-11-06, actualizado ese día | OceanBase/SeekDB: `unknown chunk field: compile_kwd` impide crear tarea en reparse; imagen v1.0.0-rc3, SHA reportado `03c1246150febe6b204b3799dffb3afe52103fb8` |
| [#20134](https://github.com/infiniflow/ragflow/issues/20134), abierto; creado 2026-09-23 | Solicita operador ontológico, construcción incremental, visualización y RAG grounded. Requisito, no evidencia de implementación ni bug |

**Contraste #21186:** código congelado conserva fetch hijo con warning/nil,
y `OpenPrune` consume `listEntries`; concuerda con mecanismo reportado.
No se reejecutó reproducción ni se auditó cada caller de borrado.
La PR enlazada [#21187](https://github.com/infiniflow/ragflow/pull/21187)
se observó **abierta, `merged=false`**, head
`223a26696ed9e082e0da17bab4c1bcee90833ca9`, consulta pública 2026-11-07.
No pertenece al pin examinado ni acredita fix/release; `merge_commit_sha`
de prueba no demuestra merge. Validación futura discriminante: hijo falla frente a
hijo completo vacío, a igual configuración y sin tocar datos compartidos.

**Contraste #22082:** [schema.go:51-107](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/engine/oceanbase/schema.go#L51-L107)
no enumera los dos campos de compilación en `chunkColumns/chunkExtraColumns`;
el selector de estado sí los solicita. La UI
[buildDocumentIngestPayload:13-29](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/web/src/services/document-ingest-adapter.ts#L13-L29)
fuerza `delete:true` cuando `run===1`. Coincidencia estática específica,
no afirmación de que todos los backends fallen. DDL manual por sí solo no
resolvería una whitelist en código según el reporte. PR/fix no verificados.
Aplicación Vanta: comprobar simetría write/read/migration de metadata derivada,
no importar columnas de OceanBase ni atribuir este error a Rust.

**Fronteras de seguridad:** tenant/dataset filtran búsqueda pero no reemplazan
autorización; contenido compilado sigue siendo tainted. Proveedores y conectores
cruzan red/secretos/SSRF. Disponibilidad lógica no demuestra purga de objetos,
historial, cachés ni logs. No se auditó red completa, CVEs, sandbox de parsers ni
edición enterprise; no se declara explotación probada.

## 4. Contratos previos en VantaDB

Evidencia propia del snapshot Vanta:

- [engine.rs:197-235](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L197-L235):
  prefijo protegido puede dejar salida sobre presupuesto; ya hay reporte.
- [assemble_with_recall:252-325](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L325):
  un presupuesto compartido, inyecciones whole-or-skip, cursor protegido.
- [l1_writer.rs:289-350](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L350):
  `persist_planned` borra destinos y luego escribe; `log_and_mark` tiene efectos
  posteriores; comentarios explicitan modelo parcial, no batch atómico completo.
- `src/wiki/store.rs::WikiStore` ya tiene create/delete/request/begin/complete/
  fail/put_page; `services/scheduler.rs::run_pass` y task checkpoints ya existen.
  Nombres comprobados con `git show` del snapshot, no derivados de README.

Requisitos por candidato: A exige fence/commit real; B exige recibo post-assembly;
C necesita revisión de productos; D solo aplica si hay reconciliación destructiva;
E necesita ownership de estado. No convertir cada incertidumbre en bloqueo
universal ni proponer arreglar L1/checkpoint/WAL como si fueran el mismo problema.

## 5. Qué no trasladar

No SQL/NATS/MinIO/stack completo como requisitos del motor embebido.
No copiar algoritmos LLM de compilación como defaults, ni el comentario sobre
FRAMES y cero llamadas como resultado propio. No gate booleano que oculta errores.
No borrar faltantes de un inventario parcial. No adoptar metadata best-effort
para decisiones de acceso o eliminación. No elevar un pin/claim a instrucción.
No copiar UI enterprise ni presumir equivalencia del SaaS con el árbol público.

## 6. Advertencias metodológicas

Es análisis estático dirigido, no lectura exhaustiva de plataforma. Historia
shallow, tags/ramas y LFS no descargados según manifiesto; submódulos no
inicializados, no se afirma cobertura de contenido omitido.
Se usaron contratos, Vanta snapshot, fuente local y issues públicos; no se
ejecutó código upstream. Los enlaces SHA fijan código; issues son observaciones
fechadas mutables. Un comentario de diseño no prueba implementación SQL completa.
No se inició ni detuvo servidor; no hubo instalaciones/tests/builds/benchmarks.

## 7. Benchmarks leídos

### Familia HTTP Python

Entry [__main__.py:1-5](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/test/benchmark/__main__.py#L1-L5)
→ `cli.main` → `run_retrieval` → request HTTP.
[cli.py:484-579](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/test/benchmark/cli.py#L484-L579)
prepara dataset, opcionalmente carga/parsea documentos, y mide repeticiones de
una pregunta/payload; no es dataset QA anotado ni benchmark longitudinal.
[Defaults:35-110](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/test/benchmark/cli.py#L35-L110):
v1, 1 iteración, concurrencia 1, conexión 5 s, lectura 60 s, parse 120 s/poll 1 s.
Son defaults upstream leídos, no recomendaciones nuevas.

[retrieval.py:14-54](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/test/benchmark/retrieval.py#L14-L54)
mide POST `/retrieval` hasta cuerpo; conserva errores HTTP/transporte/JSON/API.
[metrics.py:1-78](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/test/benchmark/metrics.py#L1-L78)
define percentil nearest-rank con ceil, p50/p90/p95; no p99.
Promedio/percentiles retrieval incluyen solo samples sin error y con latencia;
`success+failure` es población de intentos, QPS total difiere de success_QPS.
Errores no se deben ocultar aunque se excluyan de la distribución de latencia.
ChatSample distingue t0/t1/t2 y TTFT/total; no se auditó todo el transporte chat.

No recall, nDCG, answer accuracy ni juez en esta familia leída. Modelos,
tokenizer, corpus y semillas de calidad no quedan fijados por este runner;
dependen de dataset/tenant/servicio. Preparación se hace antes del cronómetro
del bucle; no incluir su latencia implícitamente en retrieval.
Multiprocessing `spawn` añade overhead al total concurrente; no warmup explícito
en el bucle inspeccionado. El flag de respuesta puede exponer contenido.

### Familia CLI Go

[benchmark.go:26-215](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/cli/benchmark.go#L26-L215)
envuelve `ExecuteCommand(iterations,nestedCmd)`, defaults 1/1, rechaza concurrencia
menor que uno, copia mapa por worker y suma SuccessCount/FailureCount.
El wrapper concurrente mide wall-clock y retorna error si un worker falla;
no equivale al modelo de samples HTTP. No inferir `iterations*concurrency`
como denominador contado: esa línea está comentada; los counts provienen de
respuestas. Sin dataset/ground truth/juez propio ni coste monetario calculado.

**Efectos y adaptación:** requiere servicios y credenciales existentes;
bootstrap puede registrar usuario/token/model defaults, crear datasets/documentos
y llamar proveedores pagados. `--teardown` controla cleanup; no usar tenant común.
Comando upstream documentado, **no ejecutado**:
`PYTHONPATH=./test uv run -m benchmark [flags] retrieval [flags]`.
Adapter Vanta futuro mapearía dataset→namespace autorizado, IDs→refs revisionadas
y errores/latencia por request, sin cambiar métricas ni convertir este runner en
certificación de calidad. QA/memoria se contrastan únicamente bajo
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md); no se diseñó harness ni sampling nuevo.

## 8. Matriz de capacidades

Estados de Vanta en snapshot; «no acreditado» no significa inexistente.

| Capacidad | Estado Vanta | Evidencia y decisión |
|---|---|---|
| BM25/HNSW/RRF, filtros y cuantización | Existe | VANTA-REFERENCE, SDK/search e índices; no portar search stack |
| Provenance, dedup y supersession | Existe | l1_writer/lifecycle; A añade fence solo si falta |
| Assembler y presupuesto compartido | Existe | engine.rs; B no reemplaza compaction |
| Recibo por fragmento post-truncado | No acreditado | IntegratedContext observado es agregado; estudiar B |
| Wiki, ingestors, scheduler/checkpoints | Existe | WikiStore/run_pass/TaskCheckpointManager; extender |
| Lease+publicación atómica de derivados | No acreditado | No demostrable desde API wiki sola; A condicionado |
| Auditoría integral fuente→derivado | Parcial | Auditoría textual existe; cobertura de todos los derivados no acreditada |
| Prune con inventario certificado | No acreditado | D solo si reconciliador real lo necesita |
| Studio retrieval/memoria/actividad | Existe | §10, revisión desktop separada |
| Dependencia de NATS/SQL/enterprise | N/A | No condición del motor Rust |

## 9. Hallazgos negativos y falsos positivos

**Hecho local:** reporte contextual agregado no acredita qué fragmento sobrevivió;
delta B requiere revisar report_store/callers antes de declarar ausencia global.
**Hecho local:** L1 persistencia secuencial y efectos post-write no prometen
atomicidad de todos los derivados. No demuestra corrupción ni resurrección.
**Hipótesis:** un job wiki podría publicar tras retiro si no hay fence; no se
certificó ese camino y no se registra bug sin interleaving discriminante.
**Descartados:** «Vanta no tiene híbrido», «no tiene presupuesto», «no tiene
tenancy ni scheduler», «todo RAGFlow requiere Python»: contradicen evidencia.
Ningún issue upstream acredita por analogía un defecto Vanta.

## 10. UI real y Studio

Fuente inspeccionada:
[CompileProgressBoard:1-61](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/web/src/pages/dataset/compilation/compile-progress-board.tsx#L1-L61)
presenta estado Failed/error o inflight/backlog y log acotado.
Declara que no hay porcentaje estable ni cancel del scheduler en esa superficie:
no presentar un botón imaginario porque backend tenga `CancelInflight`.
`buildDocumentIngestPayload` demuestra reparse con borrado, no mera etiqueta UI.
El flujo leído es comando de parse → payload → estado de compilación/log;
no se abrió navegador ni se certifica UX end-to-end.
React/Vite web real; no se acreditó shell desktop propio en la muestra.
SaaS y overlays enterprise no se infieren de archivos vacíos/página comercial.

Studio ya existe en `C:\Users\Eros\VantaDB Proyect\develop-desktop`,
SHA **`0ad38c542b39a589035b0e17fdedb4c09dca24b9`** observado.
[feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:1-85](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L85)
usa search/get/auditEvents, filtros reutilizados y modo BM25/RRF/vector.
El cuerpo se releyó con `git show`; no se atribuyen al pin los bytes actuales
movidos/modificados de `desktop/...`, ni JobChip/jobs ausentes en ese árbol.
No se propone otra consola: mostrar generación/readiness/errores de proyección
en MemoryLens/ActivityPanel, y recibos de inclusión en RetrievalLens.
No marcar cancelado por cerrar modal; requiere ACK real del backend.
Ninguna UI puede conceder scope ni volver confiable un documento compilado.

## Fuentes, cobertura y cierre

**Leído:** rangos enlazados de compiler/scheduler/consumer/document state,
compiled knowledge/evidence/serve, sitemap, OceanBase schema, frontend adapter/
progress; módulo/licencia; runners HTTP/CLI y métricas; snapshot Vanta y
RetrievalLens. Se preserva evidencia válida del informe anterior.
**No leído íntegramente:** todos los writers SQL y motores, parsers/OCR nativos,
autenticación completa, SDK, billing, enterprise, clientes externos ni todas las
rutas UI. No se calculó coste ni se reprodujo issue.
**Dependencias:** A↔fence Quivr; D→A para evitar resurrección; B→vigencia A,
C→estado de productos; E reutiliza checkpoint, no bloquea todos los candidatos.
**División futura (>300 líneas):** separar specs de publicación/prune, entrega/
capabilities y observabilidad/UI, con este dossier como evidencia común;
no se crean archivos ni specs activas en esta tarea.
**Cierre:** diez ejes documentados; issues públicos examinados, benchmark leído
sin ejecución. Pendientes aceptación manual de cada ficha, seguimiento PR/fix,
auditoría de rutas no cubiertas y **revisión adversarial independiente**.
No se declara certificación del contrato ni aprobación de implementación.
