---
title: "Quivr 2: aceptación durable, proyecciones y observaciones corregibles"
kind: research
status: proposed
description: "Informe autónomo del engine Go congelado: identidad, withdrawal, generaciones, monitoring, demo web y evaluación, contrastados con VantaDB."
tags: [research, memory, engineering]
---

# Quivr 2: aceptación durable, proyecciones y observaciones corregibles

## Índice

- [0. Identidad y arquitectura](#0-identidad-y-arquitectura)
- [1. Recomendaciones](#1-recomendaciones)
- [2. Fichas Rust propuestas](#2-fichas-rust-propuestas)
- [3. Issues públicos y seguridad](#3-issues-públicos-y-seguridad)
- [4. Contratos previos Vanta](#4-contratos-previos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método y límites](#6-método-y-límites)
- [7. Evaluación y benchmarks](#7-evaluación-y-benchmarks)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Hallazgos negativos sustentados](#9-hallazgos-negativos-sustentados)
- [10. Frontend real y Studio](#10-frontend-real-y-studio)
- [Cobertura y cierre](#cobertura-y-cierre)

## 0. Identidad y arquitectura

Repositorio `The-Vibe-Company/quivr`, clon `C:\Users\Eros\quivr`;
SHA **`bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f`**.
[Manifiesto](clone-manifest.json): observado 2026-11-07T06:56:26.7580156Z,
shallow, sin otras ramas/tags/historia ni LFS descargado. Sin fetch/reclone.
Licencia raíz [MIT:1-21](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/LICENSE#L1-L21);
no certifica licencias de plugins, datasets ni dependencias transitivas.
Fuente anterior leída con `git show HEAD:<ruta>`: se preservan retiro cercado y
observación sensible a correcciones; se amplían contratos y frontend.

**Identidad decisiva:** este árbol es un engine Go de contenido continuo,
búsqueda y monitoring, no el antiguo «brain Python». La popularidad y los
issues históricos no permiten atribuirle el mismo backend.
[go.mod:1-35](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/go.mod#L1-L35):
Go 1.27.3, pgx/PostgreSQL, Temporal, OpenTelemetry y MCP.
[README:5-54](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/README.md#L5-L54)
presenta engine sin UI de producto propia. `quivr-search/` sí contiene demo web
real y facade; negar toda UI por esa frase sería otro error.

### Mapa y superficies

| Área | Responsabilidad |
|---|---|
| `cmd/`, `internal/online/`, `contracts/` | API/CLI y contratos públicos; búsqueda lexical/semantic/hybrid |
| `internal/content/` | Source, Command, Receipt, Record, Version, Manifest/Parts y withdrawal |
| `internal/processing/` | Normalización, segmentación, embeddings y publicación por generación |
| `internal/monitoring/` | Intents, leases, evaluador, Match, notice y delivery/outbox |
| `migrations/`, adapters de persistencia | PostgreSQL canónico y journal; no se auditó cada transacción SQL |
| `plugins/` | Extensiones HTTP y modelos; no una política universal del motor |
| `quivr-search/` | Facade Node y React/TypeScript; búsqueda, feed, alertas, fuentes y admin |
| `scripts/eval/` | Runners Python de retrieval, corpus/qrels, métricas y reportes |
| `docs-site/`, `sdks/`, `client/` | Documentación/clientes, no producto SaaS inferido |

HTTP/collector → aceptación con idempotency key → PostgreSQL/objetos canónicos →
trabajo async/Temporal → plugins de normalización/segmentación/embedding →
proyección de búsqueda → evidencia de pasaje/Version.
Journal → intents de monitoring → evaluación fijada → commit de Match/notice/
outbox → polling o entrega webhook. No es memoria de ejecución del agente.

### Modelos: identidad no es hash del texto

[content.go:117-219](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L117-L219)
separa:

- `Command`: key, Source, source_revision, source_position, texto/manifest,
  extensions y provenance. ConnectorInstanceID no viene de bytes del cliente.
- `Record`: identidad estable de fuente y CurrentVersionID, Withdrawn.
- `Version`: Manifest, provenance, disponibilidad, diagnóstico y AcceptedAt.
- `Receipt`: aceptación, procesamiento, disponibilidad y diagnósticos; puede
  ocultar RecordID/VersionID si el caller no puede leer contenido.
- `Withdrawal`: comando absorbente con key/Source; no publica Version.
- `ResolvedRelation`: vista viva del destino; unavailable engloba ausente/no
  listo/retirado/no autorizado sin revelar existencia/IDs.

Revisión y posición ordenan evolución; volver a texto anterior no debería
confundirse con replay solo porque coincida el hash. El evaluador y el índice
son derivados: su disponibilidad no define que la escritura haya sido aceptada.

### Traza de escritura e indexación

1. [Service.Accept:331-419](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L331-L419)
   exige acción y corpus autorizado, borra attribution del cliente y valida
   provenance/blobs/manifests. Un blob verificado no autoriza corpus ajeno.
2. `Submissions.Accept` conserva idempotencia por entrada; `SubmitBatch`
   no convierte todos los elementos en una transacción de dominio única
   ([submission.go:8-35](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/submission.go#L8-L35)).
3. Receipt aceptado y Version pendiente no implican searchable. Procesamiento
   llena las contribuciones por plugins; diagnóstico distingue causas/retry.
4. [RunServing:12-101](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/processing/serving_projection.go#L12-L101)
   carga job queued, comprueba elegibilidad y generación/espacios servidos,
   obtiene Version y ejecuta `FillIndependent`.
5. Publica segmentación y embeddings y registra cobertura. Son llamadas
   separadas: este rango no prueba atomicidad distribuida SQL+índice.
   Publicación requerida tiene slots independientes de evaluación opcional.

### Consulta, corrección, retirada y error

[online/search.go:14-46](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/online/search.go#L14-L46)
y cliente de demo §10 muestran consulta con corpus/modo/perfil/filtros,
salida con Record/Version/pasaje. No se acreditó answerer QA obligatorio,
presupuesto de prompt ni compactación de conversaciones.

Actualizar produce revisión/versionado y nueva elegibilidad. [Withdraw:422-455](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L422-L455)
cerca identidad incluso antes de materialización; ingesta ordinaria posterior
de esa identidad es conflicto terminal. Withdrawal no equivale a purga física
de historia, objetos o datos que ya recibió un consumidor.

[EvaluationStore:104-150](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/engine.go#L104-L150)
define fanout con checkpoint, leases, retry y commits distintos:
`CommitMatch`, `CommitNoMatch`, `CommitWithdrawal`.
El contrato exige revalidar elegibilidad y registrar Match/delivery/notice/event/
outbox más finalización del intent en la misma transacción.
No-match tras corrección retracta positivo previo; withdrawal emite notice
específico. Error de proveedor queda pending con backoff, no negativo.

Cancelación de una llamada no revoca automáticamente trabajo ya aceptado.
[match_committer:22-100](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/match_committer.go#L22-L100)
agrupa positivos ya validados por organización; rechazo antes de encolar si
contexto cerrado, pero petición encolada recibe outcome durable incluso durante
shutdown administrado. No se prometen esos plazos a Vanta sin necesidad real.

### Persistencia contrastada: qué confirma realmente el adapter

La lectura adicional del adapter PostgreSQL permite ir más allá de las
interfaces de dominio, sin atribuir atomicidad a todo el sistema:

- [SubmissionStore.acceptAttempt:60-202](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/submissions.go#L60-L202)
  deriva `recordID` de organización/corpus/namespace/record_key y `receiptID`
  de organización/familia/key. Canonicaliza el comando y compara el request
  previo: misma key con bytes distintos devuelve conflicto, no otro recibo.
  Inserción de identidad, orden aceptado, reserva de revisión, recibo y trabajo
  se realizan dentro del intento transaccional. La vía rápida de primera
  revisión también hace `tx.Commit` antes de devolver pending.
- `digestSlot`, [26-48](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/submissions.go#L26-L48),
  reserva nuevo slot al volver a bytes antiguos si estos ya no son la versión
  deseada y la posición entrante debe prevalecer. Deduplicar contenido no debe
  suprimir una corrección temporal legítima.
- [withdrawAttempt:205-292](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/submissions.go#L205-L292)
  inserta identidad aunque aún no exista, bloquea la fila, marca withdrawn,
  escribe tombstone y receipt resolved; emite `record.withdrawn` solo en la
  primera transición y después hace commit. No espera a eliminar los vectores
  para reconocer la retirada canónica.
- [EvaluationStore.commitAttempt:442-460](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/evaluation.go#L442-L460)
  abre transacción, toma locks de journal/recursos y lee el intent `FOR UPDATE`.
  [CommitNoMatch:403-419](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/evaluation.go#L403-L419)
  tiene vía rápida para una fuente nunca positiva; la corrección de un positivo
  pasa por `commitNoMatch`. [595-682](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/evaluation.go#L595-L682)
  distingue notice `no_longer_matches` de `withdrawn`, deriva identidad estable
  de notice/delivery y evita volver a insertar la misma notificación.

Esto confirma fronteras SQL concretas, no recuperación certificada tras crash.
Objetos, proyección externa, ejecución de plugins y recepción HTTP no comparten
esa transacción. Un éxito canónico no significa que todas las copias físicas
hayan desaparecido ni que un consumidor externo haya visto la corrección.

### Recuperación, filtros y evidencia: no basta con un score

[retrieval.Service.Search:223-355](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/retrieval/retrieval.go#L223-L355)
requiere acción de búsqueda, fija router/plugin concreto durante la llamada y
aplica el deadline del perfil al camino completo. Defaults: hybrid, perfil
default, límite 10. Rechaza corpora repetidos/fuera de scope y peticiones
inválidas; autoriza antes de resolver las generaciones.
Los filtros de metadata se resuelven contra los campos de cada generación:
un corpus sin los campos pedidos se declara en `ExcludedCorpora`; si falta la
proyección requerida devuelve error, no un resultado vacío engañoso.
El modo semántico no mezcla espacios servidos incompatibles. Estas son
condiciones de consulta, no una prueba de equivalencia de rankings entre plugins.

`PrepareIndex`, [209-218](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/retrieval/retrieval.go#L209-L218),
explicita que se puede escribir el índice externo antes de la publicación
canónica: los lectores todavía deben exigir elegibilidad canónica.
Por tanto, «hay vector» y «puedo servir la evidencia» son estados distintos.

[Segment/SourceRange/SourceSlices:15-115](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/searchable.go#L15-L115)
anclan fragmentos a Parts inmutables con coordenadas en **code points Unicode**,
no bytes Rust ni unidades UTF-16 del navegador. Valida orden, pertenencia,
límites y no solapamiento de rangos; conserva aparte texto lexical y
`ModelInputSHA256`. Texto enviado al embedding no es necesariamente la cita.
`versionFromStored`, [725-774](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L725-L774),
lee manifiesto/texto, comprueba su coherencia y resuelve relaciones en vivo.
Fallo del resolver es error de infraestructura; no se degrada a «sin relación».

## 1. Recomendaciones

| Orden | Mecanismo | Valor incremental condicionado |
|---|---|---|
| Primero | Receipt revisionado + withdrawal cercado | Distinguir aceptar/servir y evitar revive de fuente |
| Después | Publicación por generación elegible | No competir con evaluación ni servir derivados viejos |
| Después | Observaciones retractables | Corregir alertas, no solo almacenar primer match |
| Condicional | Delivery idempotente firmado | Solo si producto requiere callbacks externos |
| Condicional | Resolución viva y redactada de relaciones | Citas históricas sin filtrar existencia ni ignorar retirada |
| Después del contrato de evidencia | Coordenadas de fuente separadas del texto del modelo | Explicar por qué una cita no coincide con el input enriquecido, sin reimplementar chunking |

Vanta ya tiene provenance, dedup, supersession, sesiones, filtros, híbrido,
wiki, scheduler, checkpoints, context engine y Studio. No son cinco
justificaciones para construir otro motor o otra base de memorias.

## 2. Fichas Rust propuestas

Contratos de diseño, **no tipos existentes ni decisiones aprobadas**.
No se fijan reglas comerciales, pesos, prompts ni thresholds de evaluación.

### A. Receipt y fence de identidad de fuente

**Caso/unidad:** conector reenvía revisión o termina alta después de baja.
Unidad upstream: `Accept/Withdraw` y Receipt/Record/Version, SHA/rangos §0.
**Delta/seam real:** `vanta-memory/src/core/record/lifecycle.rs::mark_contradiction`,
`l1_writer.rs::{plan_write,persist_planned}` y `src/wiki/store.rs::WikiStore`.
Supersession de memoria no acredita por sí solo retiro absorbente de fuente.
**Contrato:** `SourceReceipt { source_id, revision, accepted_state,
projection_state, authorized_refs }`; `accept_revision`, `retire_source`,
`inspect_receipt`. Errores `Conflict|Forbidden|Unavailable`; scope del servidor.
Inputs separan idempotency key de revisión; receipt no revela IDs a no lectores.
Es vista de dominio que referencia por operación al
[C13 MutationReceipt propuesto](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén),
no su sustituto. Aceptación, commit, durabilidad y readiness son dimensiones
independientes: fuente aceptada con commit desconocido y primario durable con
proyección pendiente son representables, no una escalera `accepted→ready`.
**Pasos:** inventariar IDs/tombstones actuales; definir si retiro es reversible;
reusar primario; enlazar intent/checkpoint y derivados; devolver aceptación
antes de proyección solo si contrato lo explicita; replays recuperan recibo.
**Fallos/concurrencia/commit:** cerca identidad aunque no exista materializada;
alta y baja compiten en la misma frontera durable. Timeout tras aceptación
requiere lookup/replay, no una nueva identidad. Sólo rechazo/cancelación
**confirmados antes de admisión** acreditan que no se aceptó. Tras dispatch,
aunque el caller aún no haya observado commit, timeout/cancelación deja
outcome **Unknown** salvo evidencia del servidor; no significa «sin efectos».
Consultar operación/idempotency key con la autoridad original; si existe
commit confirmado no degradarlo a rollback por terminar la espera.
**Deps/relaciones:** autorización, transacción/CAS real, provenance y B;
comparte principio con RAGFlow publicación, no dos implementaciones de fence.
**Alternativas:** extender tombstone actual; adapter de conector; rechazar
historia ilimitada y otra base canónica. Migración aditiva con legado explícito.
**FMEA:** baja antes de alta ignorada → fuente revive; detectar fence; replay
con payload distinto → conflicto; receipt sin ACL → fuga, redactar referencias.
**Aceptación futura:** orden alta/baja invertido no revive; retry estable;
texto vuelve al anterior pero revisión nueva se conserva; no lector no ve IDs.
**No objetivos/abierto:** Temporal obligatorio, purga legal por simple tombstone;
decidir política de recreación y retención antes de activar.

### B. Proyección fijada por generación y prioridad de serving

**Caso/unidad:** cambio de plugin/modelo mientras job viejo sigue en curso;
evaluación opcional no debe ocupar todo trabajo necesario para servir.
**Evidencia:** `ServingProjectionStore/RunServing`, `serving_projection.go:12-104`.
Elegibilidad, GenerationID y espacio servido son controles distintos.
**Delta/seam:** WikiStore `begin_processing/complete/put_page`, scheduler
`vanta-memory/src/services/scheduler.rs::run_pass`, task checkpoints.
No se afirma carencia de generación de índice en todo Vanta.
**Contrato:** `ProjectionJob { source_revision, generation, owner_revision,
purpose: Serving|Evaluation }`; `publish_if_eligible` → `Covered|Skipped|Failed`.
Error de dependencia no se traduce automáticamente en rechazo definitivo.
**Pasos:** registrar revisión de pipeline/modelo en job existente; producir fuera
de lock; validar generación actual al commit; conmutar lectura solo con cobertura;
separar admisión si existe competencia; rollback mantiene generación anterior.
**Fallos/cancelación/concurrencia:** cancelación deja job no publicado;
stale generation salta publicación; fallo entre pasajes/embeddings exige
reconciliación y barrera de visibilidad, no promesa de atomicidad multi-store.
**Deps/relaciones:** A, metadata de encoder y estado de índices; C consume
fuentes vigentes, no jobs de evaluación. No añadir SQL como requisito.
**Alternativas:** extender generación existente; reconstrucción offline;
descartar prioridad diferenciada si no hay workloads en competencia.
**FMEA:** modelo viejo publica en espacio nuevo → distancias incompatibles;
detectar owner/generación; evitar revelar cobertura antes de completar derivados.
**Aceptación futura:** cambiar encoder durante job no mezcla espacios; error
parcial no marca covered; rollback no borra primario; backlog distingue propósito.
**No objetivos:** planificador distribuido nuevo o copiar timeouts upstream;
falta validar transacción/publicación real de cada seam antes de tarea.

### C. Observación persistente sensible a correcciones

**Caso/unidad:** alerta positiva de fuente R deja de ser verdadera en R+1.
**Evidencia:** outcomes y `EvaluationStore.CommitNoMatch/CommitWithdrawal`,
`engine.go:16-55,104-150`, snapshot enlazado en §0.
**Delta/seam:** scheduler/checkpoints y provenance actuales, ActivityPanel;
no reemplazar recall ni llamar checkpoint a una memoria factual.
**Contrato:** `Observation { rule_revision, source_revision, evidence_refs,
outcome: Match|NoMatch|Pending|Withdrawn|Superseded|Error }`;
`evaluate`, `commit_if_current`, `retract`, `list_changes`.
Evaluador opcional, identidad incluye revisión de regla y proveedor.
**Pasos:** comprobar watchers actuales; definir fuente de cambios; fijar
regla/modelo al comenzar; evaluar fuera del commit; revalidar permiso/actualidad;
commit de decisión+notice+checkpoint; delivery desacoplado y desactivable.
**Fallos/concurrencia/commit:** error/timeout reintentable no es NoMatch;
regla editada durante inferencia invalida resultado; lease evita commit viejo.
Un retry no genera dos retractaciones; cancelar antes de commit deja pending.
**Deps/relaciones:** A/B; D consume notices, no modifica decisiones.
**Alternativas:** consulta bajo demanda sin subscripción; adapter opt-in;
no construir monitoring persistente sin caso de producto aprobado.
**FMEA:** perder notice → alerta obsoleta; detectar secuencia/checkpoint;
regla vieja sigue activa → falso positivo, comparar revisión al commit.
**Aceptación futura:** corrección retracta; retirada produce motivo diferente;
error queda visible y no negativo; replay y dos evaluadores no duplican evento.
Observabilidad por IDs/código, sin prompt/texto sensible por defecto.
**No objetivos/abierto:** trading/negocio/billing ni SLA; acordar si se conserva
evidencia histórica tras revocación y cómo se redacta.

### D. Entrega durable con admisión separada del transporte

**Caso agente:** un workflow consume avisos de corrección por webhook y el
receptor procesa el POST, pero la conexión se corta antes de confirmar.
**Unidad upstream:** [DeliveryStore/Deliverer.Step:31-108,224-265](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/delivery.go#L31-L108)
y [Step](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/delivery.go#L224-L265):
claim con lease → revalidar admisión y persistir intento → enviar bytes
inmutables → registrar outcome. Un intento interrumpido después de admisión
queda sin outcome, recuperable como unknown y susceptible de repetirse.
Eso es entrega al menos una vez, **no exactly-once del efecto receptor**.

**Delta/seam real:** `TaskCheckpointManager::{begin,advance,complete,fail}`
en `vanta-memory/src/utils/task_checkpoint.rs:129-188` y scheduler
`vanta-memory/src/services/scheduler.rs::run_pass:79`, más el adaptador externo
que realmente necesite callback. Son puntos de integración, no prueba de que
Vanta ya tenga este outbox ni motivo para añadirlo al motor de búsqueda.
**Contrato Rust propuesto:** `DeliveryIntent { event_id, destination_ref,
payload_digest, source_revision }`, `AdmittedAttempt { attempt_id, lease,
immutable_payload_ref }`; `admit`, `record_outcome`, `retry_due`.
Outcomes `Acknowledged|Retryable|Permanent|Unknown`; errores de store no
equivalen a fallo del receptor. Destino y secreto resueltos en servidor.

**Pasos:** 1) confirmar necesidad de callbacks; 2) hacer evento+intent durables
en el propietario existente o introducir reconciliación explícita si no hay
transacción común; 3) admitir bajo revisión/lease vigente; 4) ejecutar I/O fuera
del lock; 5) guardar outcome y reintento; 6) ofrecer inspección por IDs.
Desactivar detiene nuevas admisiones, no revierte efectos ya recibidos.
Migración solo para nuevos eventos; no reenviar todo el histórico al activarla.
**Alternativas/deps:** polling de cambios es preferible si basta; C origina
notices, D solo entrega; no acoplar aceptación A a disponibilidad del webhook.
No transportar prompts o documentos completos por defecto.

**Seguridad observada:** [NewWebhookClient:137-157](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/delivery.go#L137-L157)
rechaza red privada en dial salvo opt-in, sin proxy/redirect/cookies;
[Sign:15-37](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/monitoring/webhook.go#L15-L37)
firma id+timestamp+raw body con HMAC-SHA256. Receptor debe validar firma,
frescura y deduplicación; la firma sola no impide replay.
**FMEA:** timeout ambiguo → repetir efecto → detectar mismo event_id en receptor;
DNS rebinding → SSRF → comprobación al conectar; revocación entre claim y envío
→ admisión debe revalidar configuración; secreto en error → logs redactados.
**Aceptación manual futura:** ACK perdido conserva identidad; dos workers no
admiten el mismo lease vigente; destinatario deshabilitado no inicia intento;
private/redirect rechazados; shutdown posterior a admisión permanece auditable.
**No objetivos/abierto:** proveedor universal de colas, SLA de upstream, ventanas
o backoff copiados sin necesidad; resolver ownership/retención antes de tarea.

### E. Relación inmutable, resolución viva y referencias redactadas

**Caso/unidad:** memoria histórica cita otro documento que hoy fue retirado o
dejó de estar autorizado. Conservar la relación original sin entregar un enlace
aparentemente vigente. Unidad: `Service.resolveRelations` (§0) y
[RecordStore.Resolve:146-170](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/records.go#L146-L170).
El adapter consulta identidad estable con scope, versión actual y elegibilidad;
ausente/no listo/retirado/quarantined/inaccesible no exponen target IDs.
La **relación fuente** se conserva: no se promete ocultar un nombre que el
propio documento autorizado ya contenga.

**Delta/seam:** `src/wiki/store.rs::WikiStore::{get_page,list_pages}` y
provenance de L1. El caller debe resolver/autorizar **antes** de entregar texto a
[`assemble_with_recall:252-325`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L325):
es ensamblador puro, recibe strings/mensajes/presupuesto, **no principal/scope**
y no autoriza. El seam concreto de recall gobernado es
[`perform_auto_recall_governed:402-408`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L402-L408)
con [`InjectionPolicy:115-142`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L115-L142).
El host fija esa política desde autoridad autenticada; su filtrado por prefijos
no prueba authz universal de cada transporte. Revalidar permisos y revisión al
hidratar el destino. No añadir otro assembler, ACL ficticia ni knowledge graph.
**Contrato propuesto:** `RelationRef { source_revision, target_identity }`;
`resolve_for_reader(scope, refs)` → `Available { authorized_handle, revision }`
o `Unavailable`; `ResolverUnavailable` separado del negativo de dominio.
Política pública de redacción no informa si fue Forbidden o NotFound.
**Pasos:** inventariar relaciones actuales; conservar refs originales;
resolver tras autorización usando estado actual; no cachear disponibilidad
entre lectores; invalidar por revisión/retirada; UI conserva cita histórica con
estado. Migración lazy para páginas antiguas sin convertir hashes en identidad.
El resolver/caller posee scope y fallos; el assembler no hace IO ni adopta
autoridad desde contenido. Un error al resolver no se rellena con texto antiguo.
**Deps/alternativas:** A define retirada, B vigencia; primera alternativa
enriquecer provenance existente; rechazar grafo paralelo o joins externos
sin coherencia. Cancelar lectura no muta relación ni confirma entrega.
**Commit/concurrencia:** no hay commit de contenido al resolver; cada resolución
es una observación. Revocación posterior exige revalidar en el acceso al handle,
no confiar indefinidamente en el resultado anterior.
**FMEA:** cache compartida por ID → fuga entre scopes → clave por lector/revisión;
caída DB mapeada a unavailable → falso juicio negativo → error operativo;
link a versión vieja presentado actual → cita incorrecta → mostrar revisión.
**Aceptación futura:** retiro y cambio de ACL ocultan IDs del destino; error
del store no parece negativa; cita fuente sigue inmutable; dos scopes reciben
vistas distintas sin filtrar causa. Logs por código, no nombres privados.
**No objetivos:** promesa de acceso histórico tras revocar permisos, purga
retroactiva de lo ya enviado a un modelo ni política legal de retención.

### F. Anclas de fuente y derivación del input del modelo

**Caso/unidad:** agente recupera un fragmento cuyo embedding usó título y
normalización, pero necesita citar literalmente la fuente. Evidencia
`Segment/SourceSlices/SegmentDerivation`, SHA y rangos §0; `CheckManifest`,
[495-569](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/content/content.go#L495-L569),
valida Parts únicas, padres existentes/acíclicos y targets completos.
Esto evita que enriquecer retrieval reescriba el documento canónico.

**Delta/seam:** provenance de `l1_writer.rs::{plan_write,persist_planned}`,
`WikiStore::put_page`, y `IntegratedContext`; Vanta ya tiene provenance y
tokens/budgets, no se propone inventarlos de nuevo.
**Contrato Rust propuesto:** `SourceAnchor { source_id, revision, part_key,
coordinate_unit, start, end, source_digest }`; `DerivationRef { recipe,
model_revision, input_digest, anchors }`; `resolve_anchor` → cita exacta o
`StaleRevision|InvalidRange|Forbidden|Unavailable`.
La unidad se declara y se convierte explícitamente entre Rust/JS.
**Pasos:** revisar metadata existente; añadir campos opcionales versionados;
resolver sobre snapshot fuente; validar Unicode, orden y rangos antes de
persistir derivación; indexar texto enriquecido aparte; exponer anchors al
inspector. Regenerar solo derivados sin borrar fuente; rollback ignora metadata
nueva, no inventa offsets para registros legacy.
**Deps/relaciones:** A fija revisión, B generación, E disponibilidad y C/D
pueden transportar handles; no almacenar dos veces el input íntegro si basta
hash+receta. Alternativa: solo referencia de documento cuando no hay rangos.
**Fallos/commit:** producir fuera de lock; publicar derivación solo contra
revisión vigente. Cancelar no deja anchor válido a un producto no publicado;
si se conserva historia, resolver explícitamente la revisión histórica.
**FMEA:** offset UTF-8 tratado como code points → cita equivocada → unidad
serializada y validación con caracteres multibyte; plugin inventa rangos →
fuente manipulada → resolver contra Part, no aceptar texto del plugin;
input sensible en logs → usar hashes/referencias y control de acceso.
**Aceptación futura:** acentos/CJK/emoji y Parts múltiples resuelven exactamente;
texto lexical distinto no altera cita; revisión nueva invalida anchor antiguo
salvo lectura histórica autorizada; ausencia legacy no fabrica evidencia.
**No objetivos:** OCR universal, port de todos los normalizadores ni tokenización
exacta de cualquier modelo; revisar licencia por plugin/modelo antes de adopción.

## 3. Issues públicos y seguridad

Fuentes públicas leídas en este cierre, consulta 2026-11-07. Búsqueda keyless
`websearch.search` y contraste `metasearch.compare_engines` (DDG/Bing) no
localizaron resultados pertinentes; fallback a búsqueda de issues por API
GitHub y lectura HTML completa de los dos issues siguientes. Las llamadas
directas posteriores de API devolvieron **403**; no se reinterpretaron como
ausencia de issues y el HTML público permitió confirmar estados/actividad.

| Fuente leída | Evidencia, alcance y decisión |
|---|---|
| [#3697](https://github.com/The-Vibe-Company/quivr/issues/3697), abierto; creado 2026-07-03 | Reporta IDOR de history/delete/inyección de mensajes por `chat_id` sin comprobar dueño en el backend Python, imagen `stangirard/quivr-backend-prebuilt:latest`. La actividad visible incluye aviso stale 2026-10-01; no acredita fix ni versión resuelta |
| [#3684](https://github.com/The-Vibe-Company/quivr/issues/3684), creado 2026-06-04; cerrado **not planned** 2026-09-16 por bot | Propuesta de ffmpeg/Whisper, muestreo visual y citas temporales. Es requerimiento histórico, no pipeline probado, ni cierre que demuestre implementación |

**Mecanismo #3697:** autenticación del bearer no ata un objeto al sujeto;
operaciones filtradas solo por chat_id permiten actuar sobre un ID ajeno.
El reporte cita una ruta Python y `latest`, no el SHA Go congelado. No se
inspeccionó esa imagen ni se reprodujo la PoC. Tampoco se valida su comentario
sobre UUIDs «adivinables»: el problema sigue existiendo si el ID llega por
un enlace/log, sin necesidad de enumerar UUIDs aleatorios.
No se abrió el advisory privado enlazado; no se declara CVE ni alcance adicional.

**Contraste con el árbol actual:** `Search` comprueba scope; `storedVersion`
verifica Corpus; `RecordStore.Resolve` redacta destinos; la facade
`server.mjs::authorizeRecord` [413-421](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/server.mjs#L413-L421)
comprueba corpus antes de leer Version. Son controles concretos, **no auditoría
integral de autorización** ni prueba de que el issue histórico esté arreglado
en todas las imágenes. Validación futura discriminante: mismo ID válido leído
desde dos scopes/organizaciones en cada operación; no basta probar ID ausente.
En Vanta aplicar la misma discriminación a los transports autorizados sin
registrar una vulnerabilidad por analogía.

**Requisito #3684:** Parts y source ranges presentes hoy no prueban implementación
de Whisper, vídeo, timestamps ni un answerer multimodal. La ficha F propone
contrato de ancla, no incorporar ese feature histórico por estar cerrado.

**Otras fronteras:** plugins HTTP procesan contenido tainted y cruzan red;
webhooks tienen secretos y riesgo SSRF incluso con guard si se habilita acceso
privado. Retirada lógica, archivo de corpus, pausa de conector y borrado de
alerta no son sinónimos de purga física. No se leyeron todos los advisories,
endpoints, validadores de plugin ni SQL de aislamiento.

## 4. Contratos previos Vanta

Referencia fija **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`** y
[VANTA-REFERENCE](VANTA-REFERENCE.md); la rama en movimiento no sustituye esa base.

| Evidencia propia | Consecuencia y candidatos |
|---|---|
| [WikiStore:107-135](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L107-L135) | delete lee y prepara batch; comentario explicita callers single-process y ausencia de transacción entre claves. Antes de A/B multiwriter, determinar quién serializa |
| [begin_processing/complete:162-197](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L162-L197) | Ya hay run_id y rechazo de paquete tardío al completar; no vender B como primer control de actualidad |
| [put_page:283-318](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L283-L318) | Página gestionada locked; introducir edición humana requiere contrato separado, no desactivar ownership del compilador |
| [l1_writer:289-350](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L350) | Persistencia por pasos y efectos posteriores no acreditan commit común de receipt+derivados; A no puede prometerlo sin seam transaccional probado |
| [TaskCheckpointManager:129-188](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L129-L188) | C/D amplían trabajo durable existente, no crean scheduler/checkpoint paralelo |
| [assemble_with_recall:252-325](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L325) | E/F deben respetar presupuesto compartido e InjectionPolicy; disponibilidad no otorga permiso |

Gates **por candidato**, no bloqueo universal: A necesita política de recreación
y retención; B revisión de encoder/índice y frontera publish; C consumidor de
cambios y retractación aprobados; D necesidad de webhook y autoridad de destino;
E semántica de redacción/caché por lector; F unidad de coordenadas y versión
fuente. Si falta un gate, mantener la propuesta inactiva sin paralizar retrieval.

## 5. Qué no trasladar

- No convertir el motor Rust embebido en despliegue PostgreSQL+Temporal+Node.
  Extraer invariantes; revisar si el scheduler local satisface la misma necesidad.
- No introducir límites de planes, billing ni reglas de negocio al core:
  cuotas de almacenamiento, presupuesto de contexto y política comercial son
  dominios distintos. Vanta ya declara cuotas por namespace.
- No hacer LLM obligatorio para aceptación, tombstone o consulta lexical.
  Un clasificador opcional no convierte timeout en decisión negativa.
- No traducir todos los plugins, semántica de ranking ni parámetros upstream.
  Licencia MIT raíz no cubre por sí sola pesos, corpus, fuentes ni assets.
- No confundir demo UI con SaaS o cliente desktop; no importar su clave
  compartida como diseño de identidad de Studio.
- No copiar «retirar fuente» como borrado físico, ni outbox como garantía
  exactamente una vez. Polling sin efectos es una alternativa de menor coste.
- No reutilizar código o material derivado sin revisión legal por componente;
  reescribir en Rust no borra obligaciones de autoría/licencias.

## 6. Método y límites

Análisis estático dirigido, no auditoría exhaustiva ni revisión independiente.
Se conserva la ampliación A/B/C que ya estaba en disco; D/E/F y los ejes
faltantes se completan sin reiniciar el informe. Código fijado por SHA,
issues fechados por lectura pública; ninguno fue reproducido.
Separar declaración de interfaz, implementación SQL leída y comportamiento
dinámico pendiente: no son tres niveles intercambiables de evidencia.

El nombre histórico «Second Brain» no acredita memoria conversacional ni
checkpoint de agente en este engine. La demo sí posee estado local de lectura/
borradores e historial de versiones observadas, que no es memoria factual
persistente del modelo. Métricas de búsqueda no son exactitud QA o éxito de
agente. No se instalaron dependencias ni ejecutaron tests, apps, frontend,
builds, servicios, scripts de medición o limpieza.

## 7. Evaluación y benchmarks

### Runner y datos realmente inspeccionados

[`scripts/eval/run.py:1-51`][QEVAL] es evaluación de **retrieval por documento**,
no QA ni éxito de agente. Cadena: `main/resolve_sets` → `public_sets.prepare`
o archivo privado → `trec.load` → `measure_set` → `ingest` → search por
modo/perfil → `scoring.score` → report.json/report.md. No hay answerer/juez
obligatorio. Hay perfiles deep/reranker opcionales y costes de proveedor.
La existencia del runner no acredita cifras propias ni promoción aprobada.

[`trec.py:141-177`][QTREC] lee corpus JSONL `{_id,title?,text}`, queries JSONL
`{_id,text}` y qrels TSV (también `qrels/test.tsv`). Duplicados, IDs desconocidos
o juicios sobre documentos no incluidos son error. Excluye preguntas sin
juicio positivo y las lista en `dropped_queries`; no son abstenciones medidas.
URL privada requiere SHA-256; descarga/cache se verifican en `fetch:40-67`.
No se descargaron datasets en esta investigación.

Registro [`public_sets.json`][QSETS] y muestreador
[`public_sets.py:40-66,124-145`][QSAMPLE]:

| Familia / split | Muestra declarada queries/documentos / semilla | Condición |
|---|---|---|
| MIRACL-fr dev | 343 / 5 000 / 775 | Pool de pasajes/negativos, no Wikipedia completa |
| MLDR-fr test | 200 / 600 / 775 | Consultas sintéticas; una relevante por query según registro |
| SciFact test | 300 / 2 000 / 775 | Claims→abstracts; no evaluación de respuesta factual |
| XPQA-fr, MKQA-fr | 200 / 2 000 / 991 cada una | test y train respectivamente; no llamarlos holdout universal |
| WebFAQ-fr test | 200 / 2 500 / 991 | Recuperación de FAQ, no chat longitudinal |
| FiQA, ArguAna test | 200 / 2 000 / 991 cada una | Restricciones/licencia por dataset, no heredadas de MIT raíz |
| TREC-COVID test | 5 / 10 000 / 991 | Muestra muy pequeña de preguntas, no benchmark completo |
| SciDocs test | 200 / 6 000 / 991 | Unidad documento |
| Alloprof, BSARD, NFCorpus test | 200 / 2 000, 2 000, 3 600 / 991 | `restricted`, opt-in diagnóstico, fuera de suite default |

Son tamaños objetivo del registro, no conteos ejecutados. El muestreador exige
positivo y que quepan todos los positivos bajo `max_source_bytes` del perfil de
ingesta; conserva documentos juzgados, añade hard negatives y distractores con
semilla, y registra juicios descartados por ausencia/tamaño. Selección usa gold:
no es un corpus ciego para ajustar ranking. Cuando deep está habilitado,
`measure_set:371-375` restringe MIRACL a las primeras hasta 150 queries ordenadas;
comparar ese corte, no mezclarlo con toda la muestra declarada.

### Defaults, métricas, denominadores y fallos

En [QEVAL]: modos lexical/semantic/hybrid; baseline hybrid/default, límite 50
deduplicado por Record; límite 10 se cronometra en segunda pasada. Si hay deep
o reranking el pase scored usa 10 (`429-448`), no suponer 50 para todo perfil.
Tres primeras queries ordenadas calientan los caminos no-deep; la rama deep
no hace ese warmup en el bucle medido. El seed del muestreo no fija proveedor,
embedding, tokenizer ni orden aleatorio de la medición. Se registran perfiles
servidos/versiones/refusados; los valores efectivos de un servicio remoto son
desconocidos hasta inspeccionar su configuración. No inventar modelo universal.
CLI `644-659`: ingest-timeout 5 400 s, stall 600 s, cache y salida `.scratch/eval`.

[`scoring.py:17-52`][QSCORE] calcula por cada query de **qrels emitidos**:
nDCG@10 con ganancia lineal y descuento log2(rank+1), normalizado por ideal de
hasta diez juicios; Recall@10 divide relevantes recuperados por todos los
juicios con grade>0; MRR@10 usa inverso del primer rank relevante.
Sin hits puntúa cero; no se elimina del promedio. IDs duplicados en ranking
conservan la última posición en ese helper; `search` deduplica por Record antes.
El promedio divide por número de queries evaluadas, no por hits. Una entrada
qrels vacía no produce por sí sola un promedio válido. La comparación pareada
usa intersección de queries, t-test bilateral y alpha upstream 0.05 sin
corrección múltiple: se describe, no se adopta como umbral Vanta.

`search:325-347` hace un intento medido; excepción operativa devuelve ranking
vacío y error, por tanto cero en calidad y contador de fallo.
`BudgetExceeded` se propaga: queries no intentadas **no** son ceros ni corrida
completa. `Client._call:98-124` sólo reintenta HTTP 429/503 con
`retryable=true`, backoff acotado, normalmente cinco intentos; search fija uno.
Ingesta rechazada/atascada aborta con diagnóstico de receipts (`244-284`);
no se retiran silenciosamente documentos para mejorar scores.

Latencias de `350-360,456-491`: percentiles sobre **búsquedas exitosas**,
fallos aparte; p50/p95 nearest-rank
[`measure_metrics.py:9-14`][QPERCENT]. Sin muestras retorna None.
`client_ms` incluye llamada/respuesta/parse; usage elapsed y fases
query_encoding/index_query/hydration/plugin_rounds son tiempos del servidor
cuando existen, no ceros imputados. Aceptación/searchable-with-vectors se mide
separado de búsqueda; preparar datos/modelos/stack y scoring no entra en
latencia retrieval. No hay exactitud QA ni medición de presupuesto contextual.

### Efectos, coste y adaptación Vanta sin nuevo harness

`start_stack:581-625` prepara tokenizer/modelos, compila Go, arranca dependencias
compose, migra y levanta procesos; requiere Linux x86_64. `--api-url` usa
instalación existente y `QUIVR_EVAL_API_KEY`, **también escribe**: crea corpus
por run y acepta registros. No es una lectura inocua. `finally:763-788` guarda
reportes, captura y destruye stacks propios, y retira worktree comparativo.
No se observó borrado del corpus en servicio remoto: prever limpieza autorizada
de datos propios, no reset compartido. Cache de descargas/modelos y artefactos
persisten según sus rutas; preparación en frío y warmup no son equivalentes.
Deep exige credenciales/budget y puede generar llamadas pagadas; sin uso
reportado, coste desconocido no es cero. No se ejecuta el runner aquí.

Adaptación futura del runner existente: loader TREC intacto, `doc_id` estable a
`MemoryInput(namespace de ensayo,key,payload title+text)` mediante
[`Embedded::put_batch`][VPUT]; búsqueda a
[`Embedded::search`/ruta híbrida existente][VSEARCH], y vuelta de namespace/key
a doc_id antes del score. No convertir cada chunk en documento adicional:
deduplicar al mismo Record/documento y conservar el presupuesto/corte real.
Reutilizar filtros y política de inyección existentes; igual embedding,
normalización, corpus/split, backend y perfil para variantes. Separar pista
lexical de semantic si no se puede fijar encoder; no traducir deep a BM25
fingiendo equivalencia. A/B aportarían aceptación/generación observables,
no un nuevo scheduler, evaluador, prompt, fórmula o threshold.
Retirada A, retractación C y refs E requieren sus casos funcionales, no quedan
certificadas por nDCG. [METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md)
complementan esta correspondencia. No resultados propios ni comandos lanzados.

## 8. Matriz de capacidades

| Mecanismo | Estado Vanta al `97c9623...` / evidencia | Decisión incremental |
|---|---|---|
| Híbrido BM25/HNSW/sparse/RRF y filtros | Existe [VSEARCH] | No reemplazar motor ni copiar ranking Quivr |
| Identidad/provenance/dedup/supersession | Existe, L1 [VL1] | A separa identidad de fuente y retiro absorbente, no otro canon |
| Retiro absorbente antes de materialización | No acreditado en seams Wiki/L1 leídos [VWIKI][VL1] | A condicionada a política y transacción/CAS; no bug probado |
| Publicación por revisión/run | Parcial: Wiki begin/complete/fail [VWIKI] | B sólo refuerza cobertura/generación entre derivados si falta |
| Checkpoint/scheduler | Existe [VTASK][VWORK] | A–D reutilizan ejecución; no otro WAL/Temporal |
| Retractación de observaciones ya entregadas | No acreditada para producto C | Requiere consumidor/semántica aprobados, no confundir supersession factual |
| Webhook firmado y entrega durable de observaciones | No acreditado en esta muestra | D opcional; polling es alternativa; SHA-256 de WriteReceipt no es firma |
| Gobernanza de recall | Existe [VGOV] | E autoriza en caller y revalida hidratación; assembler puro no concede acceso |
| Assembly/compaction y reporte | Existe [VASSEMBLY] | Conservar contratos/budgets; no segundo assembler |
| Referencia histórica con resolución viva redactada | Parcial: provenance/Wiki | E distingue Unavailable de error sin enumerar destino |
| Fragmento/coordenadas sobre revisión | Parcial: payload/provenance | F conserva unidad/revisión; no presentar offsets viejos como evidencia actual |
| Cuotas por namespace/checkpoints | Existe según [referencia](VANTA-REFERENCE.md) | No confundir cuota de almacenamiento, budget de modelo y plan comercial |
| Studio | Existe [VSTUDIO][VRETRIEVAL] | Inspección sobre producto actual; propuestas no implementadas |
| PostgreSQL+Temporal+Node como requisito | No aplicable | Arquitectura upstream no es dependencia del motor embebido |

## 9. Hallazgos negativos sustentados

**Hecho acotado:** [VASSEMBLY] recibe mensajes, strings de recall, estimador y
configuración; no principal/scope. Es diseño puro, no bug ni authz faltante en
todo Vanta. Seguridad de E depende del caller [VGOV] y del resolver autorizado,
no de pasar texto al assembler. No atribuirle ACL por tener «recall» en el nombre.

**Hecho acotado:** `apply_dedup_batch` [VL1] separa deletes y puts y advierte
fallo parcial; eso no demuestra atomicidad global ni corrupción ejecutada.
`TaskCheckpointManager` [VTASK] expresa reanudación at-least-once, no exactly-once
del efecto externo. `WriteReceipt` de `put_certified` (§4) contiene hash SHA-256,
no firma como el HMAC del webhook Quivr. C13 sigue **propuesto**.

**No acreditado, no inexistente:** fence absorbente de fuente, invalidación de
todas sus proyecciones, retractación de notice ya consumido y resolver E en
todas las rutas/bindings. Inventariar callers/revisiones antes de abrir tareas.
**Hipótesis discriminantes:** publish tardío tras baja → comprobar generación
en B; cancelación tras dispatch → lookup de A, no retry con otra identidad;
destino revocado entre selección e hidratación → revalidar E; fallo de entrega
→ distinguir efecto confirmado, pendiente y Unknown en D.
**Falsos positivos descartados:** ausencia de híbrido, dedup, governance,
cuotas, scheduler, checkpoints, compaction o UI. No traslado de #3697/#3684 al
engine Go actual ni a Vanta por analogía. No claim de seguridad/rendimiento.

## 10. Frontend real y Studio

La demo `quivr-search` es React/TypeScript/Vite con facade Node:
[`package.json:1-29`][QUIPKG], no cliente desktop acreditado ni el SaaS antiguo.
[`App.tsx:42-55`][QUIAPP] carga vistas de alertas, fuentes, admin, explorer y
AddText; el feed viene con la página. Eso no demuestra todos los flujos vivos.

Flujo de búsqueda [`FeedPage.tsx:245-316`][QUIFEED]: debounce → lexical/hybrid,
corpora/sources/profile → search → dedup por Record → excerpt/revisión/estado
degradado/error. Si source_filter_unavailable reintenta sin ese filtro y conserva
`within` efectivo: la UI debe mostrar el alcance real, no llamar al resultado
filtrado. Cleanup aborta espera HTTP; **no acredita rollback de efectos remotos**.
[`lib/search.ts:31-126`][QUICLIENT] combina señal con timeout, clasifica
APIError/retryable, envía body de búsqueda, recupera versión y consulta receipt.
No realiza por sí misma autorización de corpus.

Facade [`server.mjs:268-304,737-760`][QUIFACADE] inyecta Bearer del servidor,
rechaza redirects upstream y limita cuerpo; comprueba sesión de demo y corpora
permitidos antes de proxy search. No exportar la clave al navegador ni convertir
la contraseña compartida en identidad multitenant de Vanta. No se certificó
toda política CSRF/SSRF/replay ni despliegue público.
[`AddText.tsx:48-110`][QUIADD] conserva idempotency key en borrador, hace accept,
poll de receipt hasta searchable/bloqueado y muestra progreso; aceptación no
equivale a publicación. `sessionStorage` del borrador no es memoria del agente.

Destino: Studio existente, cuerpos `git show` del SHA
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`,
[`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:1-90`][VSTUDIO]
y [`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:1-85`][VRETRIEVAL].
Hay auditEvents/filtros y búsqueda/score/contexto; proponer A/B/D como observación
de primario y derivados, C retractación visible, E cita histórica con estado,
F revisión/coordenadas. No crear app, dar permiso desde UI ni marcar «sin efectos»
porque se cerró un modal. Las rutas actuales `desktop/...` están movidas y
modificadas; no atribuir esos bytes al pin. JobChip/jobs no figuran en ese árbol
y no sustentan esta propuesta. No se abrió navegador ni se inició runtime.

## Cobertura y cierre

Se conservan las seis fichas A–F y los issues históricos con sus límites; se
restituyen §7–10 y se corrigen autorización y cancelación. No autoaceptación:
requiere revisión independiente del responsable.
**Leído:** rangos enlazados de dominio/SQL, serving projection, retrieval,
monitoring/webhook; runner run/scoring/trec/public_sets y percentiles; cliente,
feed, AddText, facade/entrada React; seams Vanta y cuerpos Studio indicados.
**No leído completo:** todos los plugins/SQL/migrations, modelos/tokenizers
externos, datasets descargados, campaña/Modal/OSS bakeoff, billing, authz de
todos los transports, replay/SSRF total ni transitivas/licencias de assets.
No tests, apps, builds, benchmarks, instalaciones, servicios o git writes.

Relaciones: B necesita identidad/vigencia A para publicación; C consume cambio
vigente y necesita contrato de retractación; D sólo si hay webhook real;
E necesita retirada/resolución por lector sin exigir todo C/D; F puede empezar
con evidencia de versión existente. C13 sólo describe mutación total y no
obliga transacciones distribuidas ni fsync de derivados reconstruibles.
Validación futura manual: replay/conflicto, baja antes de alta, publish tardío,
cancelación postdispatch, no-match tras positivo, proveedor fallido, destino
revocado y coordenadas sobre revisión antigua. Ningún caso ejecutado.
**División futura:** specs de fuente/publicación A/B, monitoring/entrega C/D,
resolución/evidencia E/F y anexo runner/UI; mantener dossier y enlaces.
No crear esos documentos, harnesses ni tareas activas sin aprobación.

[QEVAL]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L1-L788
[QTREC]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/trec.py#L40-L177
[QSETS]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/public_sets.json#L1-L523
[QSAMPLE]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/public_sets.py#L40-L145
[QSCORE]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/scoring.py#L17-L52
[QPERCENT]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/measure_metrics.py#L9-L14
[QUIPKG]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/package.json#L1-L29
[QUIAPP]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/App.tsx#L42-L55
[QUIFEED]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/components/feed/FeedPage.tsx#L245-L316
[QUICLIENT]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/lib/search.ts#L31-L126
[QUIFACADE]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/server.mjs#L268-L760
[QUIADD]: https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/components/AddText.tsx#L48-L110
[VPUT]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1303-L1340
[VSEARCH]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/mod.rs#L80-L142
[VL1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L446
[VWIKI]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/store.rs#L107-L199
[VTASK]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L129-L188
[VWORK]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L265-L390
[VGOV]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L115-L142
[VASSEMBLY]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L325
[VSTUDIO]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L1-L90
[VRETRIEVAL]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L85
