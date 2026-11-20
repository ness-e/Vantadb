---
title: "MemPalace: corpus literal, contratos de backend y limites de recall"
kind: research
status: proposed
description: "Estudio estatico del corpus verbatim, recuperacion, backends Python y Rust y runners de MemPalace frente a VantaDB"
tags: [research, memory, engineering]
---

# MemPalace: corpus literal, contratos de backend y limites de recall

Checkout verificado con `git rev-parse HEAD` y fuentes presentes:
`C:\Users\Eros\mempalace`, `MemPalace/mempalace`, rama `develop`,
SHA `f8e35362189bd0a5abf4c0df1dd5bf90ba059176`.
**60 446 estrellas**, MIT, no archivado: [snapshot](github-candidates.json);
sin reconsulta. Preparación/omisiones en [manifest](clone-manifest.json).
Captura de metadata 2026-11-07T06:56:03.7405919Z. Estudio estático bajo
[METHOD](METHOD.md), [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md), sin resultados propios.
Baseline Vanta **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**:
[referencia fijada](VANTA-REFERENCE.md). Las lecturas iniciales más antiguas
no certifican este snapshot ni el HEAD concurrente.

**Contenido:** [0. Mapa y modelo](#0-mapa-api-y-modelo-de-memoria) ·
[1. Arquitectura](#1-recomendaciones-para-vantadb-y-contexto-arquitectonico-del-proyecto) ·
[2. Extracción](#2-partes-extraibles-flujo-real-lectura-de-codigo-y-mapeo-a-rustvantadb) ·
[3. Riesgos](#3-defectos-riesgos-e-issues-transferibles-con-su-nivel-de-evidencia) ·
[4. Prerrequisitos](#4-prerrequisitos-en-vantadb-antes-de-incorporar-la-idea) ·
[5. No trasladar](#5-componentes-y-practicas-que-no-deben-trasladarse) ·
[6. Método](#6-advertencias-metodologicas) · [7. Benchmarks](#7-benchmarks-existentes-runner-entrada-metricas-dependencias-y-riesgos) ·
[8. Matriz](#8-matriz-existeparcialno-acreditadono-aplicable-frente-a-vantadb) ·
[9. Contraste](#9-debilidades-de-vantadb-sugeridas-por-el-contraste-no-bugs-inventados) ·
[10. Interfaz](#10-escritorio-interfaz-e-inspeccion-implementado-externo-o-no-localizado) ·
[11. Inventario ampliado, relaciones y división futura](#11-inventario-ampliado-relaciones-y-división-futura).

## 0. Mapa, API y modelo de memoria

El paquete leído es **3.12.0**, Python >=3.10 [M0]. `pyproject.toml`
declara Chroma `>=1.5.4,<2`, YAML, numpy, tokenizers, huggingface_hub y
python-dateutil; extras habilitan proveedores remotos y modelos específicos.
Esto no prueba resolución de un lock ni disponibilidad futura de modelos.
Licencia MIT raíz: el modelo y cada dependencia tienen obligaciones propias.

| Árbol | Función real y frontera |
|---|---|
| `mempalace/cli*`, `mcp_server/`, `mcp_proxy.py`, `mcp_light_server.py` | Entradas CLI y MCP declaradas; read/write/status/recuperación; no aplicación desktop |
| `convo_miner.py`, `miner.py`, `palace/writes.py` | Captura live o por archivo, clasificación, upsert y limpieza por fuente |
| `searcher/query.py` | Recuperación directa de drawers, boost auxiliar y enriquecimiento |
| `backends/base.py`, `chroma.py`, `sqlite_exact.py`, `rust_exact.py` | Contrato común, persistencia y selección de capacidades; otros adapters no auditados íntegros |
| `knowledge_graph.py` | SQLite independiente para entidades, triples y temporalidad; no mismo commit que colecciones vectoriales |
| `crates/mempalace-core`, binding PyO3 | Scan exacto nativo sobre snapshot SQLite; escritor sigue fuera del núcleo nativo |
| `benchmarks/longmemeval_bench.py` | Harness retrieval con variantes; no se identifica con producción por compartir nombre |
| `website/.vitepress/` | Documentación/landing Vue, no inspector de memoria acreditado |

### Identidades, derivados y superficies

| Unidad | Datos/identidad observados | No confundir con |
|---|---|---|
| Palacio | `PalaceRef { id, local_path, namespace }` [M1] | Namespace no soportado no debe ignorarse; path no es identidad de actor |
| Wing/room | Clasificación en metadata [M3][M5] | ACL o scope de autorización |
| Drawer | Texto, fuente, chunk index/total, fechas y receta de ID | Un hecho L1 siempre sintetizado |
| Drawer lógico chunked | Handle estable del contenido; filas `id_chunk_NNNNNN`, `parent_drawer_id` [M10] | Obligación de que exista una fila parent con embedding |
| Closet | Punteros/señales por fuente que mejoran ranking [M5] | Fuente literal ni condición obligatoria para llegar al drawer |
| AAAK | Derivado abreviado opt-in, invalidable por cambio del raw [M4] | Compresión reversible o transcripción byte a byte |
| Triple temporal | Entidades, predicado, intervalo, confidence, source_closet/file/drawer y adapter [M9] | Historial transaccional completo o checkpoint de ejecución |
| Índice Rust | Vectores e IDs de una colección y versión; hidratación SQL [M2] | Store escritor autónomo o ANN |

**API acreditada:** entradas `mempalace`, `mempalace-mcp`,
`mempalace-light-mcp` en [M0:53-56]; `search_memories` en [M5];
`tool_add_drawer`, `tool_get_drawer`, `tool_delete_drawer` [M10];
`KnowledgeGraph.add_triple`, `invalidate`, `supersede`, `query_entity`
y `import_graph_rows` [M9]. No se asume una API REST pública por existir
un proxy MCP. La autenticación remota y todos los transports quedan fuera.

**Memoria vs ejecución:** los drawers conservan evidencia y el grafo representa
relaciones con vigencia. Marcadores de importación/cache son control operacional,
no memoria de decisiones de herramientas, undo ni estado durable de un agente.
`created_at` de un triple no sustituye una ventana de conocimiento bitemporal;
invalidar un hecho no borra su evidencia ni sus proyecciones.

**Despliegue:** biblioteca/CLI/MCP local por defecto, backend seleccionable y
extensión nativa opcional. “Sin API key” no significa cero descarga, CPU,
almacenamiento o dependencia de modelos: [M0] comenta descarga lazy del modelo
multilingüe; el benchmark raw tiene otro default. No se instaló ni arrancó nada.

## 1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto

**Dictamen:** preservar evidencia literal y hacer explícitos los contratos de
backend son ideas útiles; la metáfora del palacio no justifica reemplazar la
memoria semántica existente. Vanta ya tiene L0, procedencia, sesiones, ingestors,
escenas, embeddings y contexto con presupuesto compartido.

| Componente | Stack y responsabilidad |
|---|---|
| CLI/MCP | Python >=3.10; ingest, consulta e integración de agentes |
| Palacio | Wings/personas/proyectos; rooms/agrupaciones; drawers/texto original |
| Índice auxiliar | Closets/punteros a drawers; señales de ranking, no fuente literal |
| Persistencia | Chroma por defecto; contrato `BaseBackend`/`BaseCollection`; alternativas SQLite, Rust exact y remotas |
| Grafo | SQLite separado; ampliación lee temporalidad, supersede e import preview [M9], no todo el módulo |
| Núcleo nativo | `crates/mempalace-core`, rusqlite y Rayon; PyO3 en crate de binding |
| Presentación | VitePress/Vue y landing; no confundir sitio con explorador del corpus |

El README anuncia **96,6% R@5 raw** y cero API; es un claim publicado de
retrieval, no exactitud QA ni medición de este estudio. «Verbatim» significa
evitar paráfrasis del corpus: la ruta live leída hace `.strip()`, de modo que
no promete preservación byte a byte de whitespace [M3].

**Prioridad específica:** primero A (identidad/capacidades) y C (publicación
recuperable) donde falte contrato; B (piso de evidencia) amplía recall actual.
D añade identidad lógica/auditoría, E solo si se requiere consulta temporal,
F solo si existe caso aprobado de migración/fusión. No convertir una metáfora
ni un porcentaje promocional en arquitectura obligatoria.

## 2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB

### Recorrido comprobado

1. **Write/retain:** `file_conversation_exchange` valida wing/room, asigna
   metadata canónica, fechas, agente, identidad de origen y `extract_mode`;
   `upsert` conserva texto del intercambio sin abstracción LLM [M3].
2. **Filing/mining:** `write_file_drawers` escribe nuevos lotes antes de eliminar
   stale drawers del mismo archivo/modo. Marca conjunto incompleto mediante
   `chunk_total`, purga stale y solo entonces finaliza metadata [M4].
3. **Extract/consolidate:** closets y clasificación organizan acceso, pero no
   sustituyen drawers. AAAK es abreviación experimental, lossy y opt-in según
   CLAUDE; no es el almacenamiento predeterminado. Rutas diary/LLM del benchmark
   son variantes distintas, no prueba de que toda captura sea extractiva.
4. **Read/recall:** `search_memories` consulta drawers directamente; closets
   añaden boost por archivo. Ausencia/fallo de closets degrada a drawer-only,
   no bloquea los originales. Hay pools vectoriales, ranking lexical y filtros;
   no toda combinación es idéntica al runner LongMemEval [M5].
5. **Rust real:** `VectorIndex::load_from_sqlite` abre solo lectura y selecciona
   colección explícita; `query`/`query_parallel` hacen búsqueda exacta y filtro
   wing. Python sigue siendo escritor y dueño del backend. `RustExactCollection`
   acelera casos soportados y cae a SQLite para filtros/includes incompatibles;
   no es un reemplazo completo de ingest ni un motor ANN [M2][M6].
6. **Reflect:** no se localizó un equivalente general a retain/recall/reflect
   de Hindsight en los cuerpos examinados. No convertir la clasificación o
   el dialecto en una supuesta reflexión autónoma.

**Identidad/aislamiento:** `PalaceRef.id` es obligatorio; `namespace` solo si el
backend anuncia capacidad. Si no la soporta debe rechazarla, no ignorarla.
`EmbedderIdentity` separa nombre/dimensión; compatibilidad del modelo importa
aun con igual dimensión [M1]. Wings/rooms son organización, no autorización.

**Borrado/retry:** [M4] muestra recuperación por re-mine con marcador incompleto,
no una transacción global entre todas las colecciones/archivos. Invalida AAAK
afectado y conserva resúmenes de drawers sin cambios. No demuestra borrado
de backups, grafo y todos los caches ni exactamente-una-vez para eventos live.

### Ficha A — contrato de capacidades e identidad del backend

- **Fuente/unidad:** `PalaceRef:153-189`, `EmbedderIdentity:193-204`,
  `validate_write_batch:462-474`, `BaseCollection:477-582` [M1].
  Problema: fallos silenciosos al intercambiar storage o embedders.
- **Seam Vanta:** [writer/EmbedFn](../../../../vanta-memory/src/core/record/l1_writer.rs)
  `51-110`; [recall_candidates](../../../../vanta-memory/src/core/record/l1_reader.rs)
  `321-380` ya admite vector opcional. No agregar otro store por comodidad.
- **Hipótesis Rust:** `EmbeddingIdentity { model, revision, dimension, metric }`
  y `BackendCapabilities`; la operación rechaza filtros/scopes no soportados.
  Son contratos candidatos, no tipos actuales acreditados.
- **Incremento/pasos:** inventariar validaciones actuales del SDK; unificar
  detección de capacidades; validar identidad al abrir; propagar errores
  tipados hasta adaptador. Dependencias: catálogo de modelos y formato persistido.
- **FMEA:** aceptar igual dimensión con distinto modelo mezcla espacios;
  fallback que quite scope puede filtrar datos; defaults de health no prueban salud.
- **Aceptación funcional:** modelo incompatible no modifica índice; backend
  incapaz de namespace devuelve error; batch desalineado se rechaza antes de
  escribir. **No objetivo:** portar Chroma ni copiar un ABC Python a Rust.

**Contrato propuesto completo A:** entrada `OpenEmbeddingSpace { scope,
expected_model_revision, dimension, metric, required_capabilities }`;
salida `ValidatedSpace { identity, generation, effective_capabilities }`.
`open_checked(...) -> Result<ValidatedSpace, SpaceError>` distingue
UnknownLegacyIdentity, ModelMismatch, DimensionMismatch, UnsupportedFilter,
UnsupportedScope y StorageFailure. Son nombres hipotéticos, no tipos del SDK.
El identificador del espacio no cambia por update de contenido; cambiar de
modelo requiere nueva generación, no reetiquetar vectores antiguos.

**Ownership/commit/concurrencia A:** SDK posee índice/metadata persistida;
`EmbedFn` solo calcula vectores y no declara por sí mismo identidad durable.
Validar antes del primer write; publicar identidad con la generación
correspondiente. `RustExactCollection::_query_native:141-199` [M2] captura
versión, carga snapshot, hidrata y vuelve a comprobarla: si cambia lanza
`_SnapshotChanged`, no devuelve un lote mezclado. El principio útil es
vincular vector y payload a una revisión; no copiar su cache como otro índice.
Fallback conserva filtros y verifica capabilities; cancelación no deja
identidad nueva con vectores viejos.

**Pasos/migración/alternativas A:** inventario de guards actuales → lectura
compatible de identidad → diagnóstico legacy Unknown → guard de escritura →
reembedding explícito en generación nueva cuando corresponda. Alternativas:
extender metadata actual del SDK (preferida), validación en adapter como
primer mínimo, o rechazar backend incapaz; no otro motor. Rollback desactiva
ruta nueva sin renombrar espacios ni eliminar generaciones previas retenidas.
Aceptación futura adicional: mismo tamaño/distinto modelo falla; escritura
durante hidratación entrega una revisión consistente o conflicto; no falso
“vacío”. Contadores mismatch/conflict/fallback, sin vectores ni texto en logs.

### Ficha B — índice abstracto que nunca excluye el corpus literal

- **Fuente/unidad:** `search_memories`, `searcher/query.py:315-369` [M5];
  `_closet_boosts:106-158` y `file_conversation_exchange:78-158` [M3].
  Problema: una abstracción incompleta oculta evidencia que sí existe.
- **Seam Vanta:** [L0Recorder](../../../../vanta-memory/src/core/conversation/l0_recorder.rs)
  `record_turn:274`, `read_messages:378`; escenas y
  [assemble_with_recall](../../../../vanta-memory/src/context_engine/engine.rs)
  `252-264` ya manejan contexto y recuperación.
- **Hipótesis Rust:** `EvidencePointer { source_id, span, revision }`,
  `CandidateOrigin::Raw | Derived`; consulta al corpus permitido más boost
  de derivados, siempre bajo el presupuesto actual.
- **Incremento/pasos:** mapear ruta raw ya existente; verificar permisos antes
  de ambos brazos; conservar puntero al texto; explicar boosts; integrar en
  ensamblador. Dependencias: retención L0, provenance, filtros y token estimator.
- **FMEA:** doble presupuesto y duplicación; ventanas que exponen otra sesión;
  índice derivado stale; preservar secretos literalmente sin política de borrado.
- **Aceptación funcional:** derivado vacío o fallido no bloquea evidencia raw
  autorizada; referencia resuelve a revisión correcta; borrado no deja previews
  recuperables. **No objetivo:** prohibir síntesis L1/persona ni prometer recall total.

**Contrato propuesto B:** `recall_with_raw_floor(scope, query, budget,
source_revision) -> Result<FloorRecall, RecallError>`, donde cada hit conserva
`EvidencePointer`, clase Raw/Derived y revisión. Errores separados:
RawUnavailable, UnsupportedScope, StaleReference y ProviderFailure.
“Piso” significa ejecutar la ruta raw autorizada aun sin abstracción útil,
no reservar un número nuevo de slots ni prometer que todo raw entre al top-k.
La consulta no eleva drawers por sobre el presupuesto del assembler.

**Ownership/commit B:** L0 es dueño del original, jobs de sus derivados;
recall es read-only y assembler posee el contexto entregado. Reautorización
al resolver el pointer; si cambió revisión se rechaza o reconsulta explícita,
no se mezcla texto nuevo con score viejo. Cancelación de un brazo se refleja
como fallo/omitido, no “éxito sin evidencia”. No añade commit de fuente.
Migración aditiva de metadata/punteros, históricos Unknown; no rellenar span
inventado. Rollback desactiva boost, mantiene ruta raw actual y provenance.
Alternativas: instrumentar primero recall actual o adapter de origen; no
replicar closets como nueva taxonomía. Depende de A solo al combinar espacios
distintos; es compatible con D para hidratar chunks y con C para revisiones.
FMEA adicional: boost de derivado viejo lleva a fuente nueva equivocada;
detección por revisión, excluir stale. Aceptación futura: raw y derived del
mismo source no duplican presupuesto; denegación en raw no se elude vía closet.

### Ficha C — reindexación recuperable por archivo, sin destruir primero

- **Fuente/unidad:** `write_file_drawers:150-227` y
  `StaleDrawerPurgeFailed:11-16`, `palace/writes.py` [M4].
  Problema: perder última evidencia válida durante reingest parcial.
- **Seam Vanta:** para páginas wiki, `ingest/worker.rs::ingest_body:244-301`
  [V-publish-wiki] liga `merge::commit` a `WikiStore::get_page/put_page`;
  `execute:109-151` marca complete/fail **después** de escrituras por página.
  Para el corpus/fuente de C, el puerto es `Embedded::put(MemoryInput)`
  [V-publish-record], no `CandidatePage` ni el recibo de wiki.
- **Hipótesis Rust:** `SourceImportGeneration { source, revision, stage }`,
  `ImportOutcome::PendingCleanup | Committed`; elegir marcador/commit adecuado
  al SDK existente, no imitar `chunk_total+1` literalmente.
- **Incremento/pasos:** revisar atomicidad real de ingest; registrar generación;
  preparar nuevos fragmentos; publicar/retirar stale; recuperar cleanup;
  invalidar derivados por referencias. Dependencias: IDs estables y checkpoints.
- **FMEA:** marcador se borra antes de cleanup; retry duplica; IDs chocan;
  conservar temporalmente dos versiones confunde recall.
- **Aceptación funcional:** interrupción entre lotes deja evidencia legible;
  siguiente pasada converge; purge fallido no aparece como importación completa;
  otro modo/fuente no se borra. **No objetivo:** certificar crash safety sin pruebas.

**Contrato propuesto C:** `plan_import(source_id, expected_revision,
chunks) -> ImportPlan`; `commit_import(plan) -> Result<ImportReceipt,
ImportError>`. Recibo separa Prepared, Published y CleanupPending.
RevisionConflict, CancelledBeforePublish, PartialCleanup y StorageFailure
no son intercambiables. ID de fuente estable; generación identifica revisión,
no timestamp del último retry. `CandidatePage`/`CommitReport` [V5] son páginas
`wiki/...` y paths/errores: no reciben generación de fuente ni publican L0/grafo.
Los contratos propuestos se conservan, pero su publicación por revisión es
integración **nueva no acreditada**, no extensión automática de ese DTO.

**Ownership/commit C:** ingestor posee identidad/revisión de fuente y prepara
registros; el SDK persiste cada registro mediante `put`, con errores/auditoría
[V-publish-record]. En conversación, `L0Recorder::record_turn` es otro dominio
específico, no importador universal. No se acredita una operación SDK que
publique fuente+chunks+relaciones como una generación indivisible.
La política de generación/cleanup siguiente es **propuesta** sobre esos
puertos y el progreso existente, no un nuevo WAL/scheduler. Propuesta:
antes de publicar, fallo conserva última generación válida; después, fallo
de cleanup no revierte fuente vigente ni dice “todo borrado”. Otro escritor
de la misma fuente exige revisión esperada, no lock global sobre todos los
ingestors. La docstring upstream exige `mine_lock(source_file)` [M4],
pero aquí no se acreditó lease durable multiproceso: no copiar como garantía.
Generación vieja no puede borrar chunks de la nueva por matching solo de path.

**Reuso y alternativas C:** `_split_unchanged`/`_upsert` de `writes.py:23-140`
reutilizan embeddings/documentos sin cambios antes del purge. Incorporar esa
economía únicamente con identidad de modelo A y digest de contenido; igualdad
de texto no demuestra igualdad de espacio. Preferir extendido del ingestor,
alternativa reingest completo por nueva generación, rechazar delete-first.
Fases: plan read-only → staging → publicación por revisión → cleanup
reanudable → invalidación de derivados. Migración marca legacy sin generación
como Unknown, nunca como completo por contar filas. Rollback pausa nuevos
imports y conserva recibos/cleanup, no borra el último raw.
Aceptación futura: cambio de modelo fuerza reembed; cleanup fallido aparece
pendiente; retry del mismo plan converge sin borrar fuente distinta. Métricas
por fase/reuso/conflicto, sin paths privados ni texto de chunks por defecto.

### Ficha D — handle lógico, hidratación y retirada de proyecciones

**Caso agente:** un agente guardó una salida larga en varios chunks y otra
herramienta conserva solo el ID devuelto. Leer, inspeccionar o borrar ese ID
debe referirse a la misma unidad lógica, sin exigir una fila parent ficticia.

- **Evidencia/unidad mínima:** `tool_add_drawer:489-625` devuelve handle lógico
  y chunks; `_logical_chunk_group`/`_chunk_group_record:99-139` agrupan y
  ordenan; `tool_get_drawer:1229-1251` hidrata; `_delete_resolved_drawer:697-712`
  distingue handle lógico y chunk físico [M10]. `drawers_removed:278-329`
  trata derivados **después** del commit del drawer [M11].
- **Delta/seam real:** provenance y lectura L0 ya existen; [V1] registra
  `source_message_ids`, [V5] tiene recibos de ingest. El delta es contrato
  común de unidad lógica/filas/proyecciones en ingest, auditoría
  `src/cli_handlers/index.rs` y detalle Studio. No otro índice ni catálogo
  paralelo de documentos.

```rust
// Propuesta; estos nombres no son APIs existentes.
enum EvidenceHandle { Source(String), PhysicalChunk(String) }
struct EvidenceView {
    source_id: String, revision: u64, chunk_ids: Vec<String>,
    completeness: Completeness,
}
fn resolve_evidence(scope: AuthorizedScope, id: EvidenceHandle)
    -> Result<EvidenceView, ResolveError>;
fn plan_withdrawal(view: EvidenceView) -> Result<WithdrawalPlan, ResolveError>;
```

**Contrato/invariantes:** resolver por ID y revisión bajo scope autorizado;
orden de chunks explícito; Source devuelve unidad lógica, PhysicalChunk
no implica autorización para retirar todo el source. Errores distinguen
NotAuthorized/NotFound sin revelar existencia ajena, MissingChunk,
RevisionConflict, UnsupportedHandle y StorageFailure. La view no prueba que
todos sus derivados estén borrados; el recibo separa retirada lógica,
eliminación del primario y cleanup pendiente de índices/resúmenes/previews.
Una fila parent ausente puede ser representación válida, no inconsistencia.

**Ownership/commit/concurrencia:** ingestor/SDK posee fuente y mapping;
jobs poseen cleanup. Plan lleva revisión esperada; si aparece un chunk
nuevo entre resolver y retirar se replantea, no se borra por prefijo textual.
Retirada lógica impide hidratación desde el índice viejo; fallar cleanup no
reautoriza lectura. Upstream atrapa fallos de purge AAAK y reporta errores de
revisiones [M11]; no prueba una transacción global ni eliminación del KG.

**Pasos y migración:** inventariar IDs/relaciones actuales → resolver
read-only → contadores fuente/chunks/filas/proyecciones por separado →
recibo de retirada → cleanup idempotente. Históricos sin mapping quedan
Unresolved, no se agrupan por adivinar sufijos. Reindex solo de proyecciones
afectadas; rollback oculta la view nueva pero conserva tombstones/recibos.
Alternativas: adapter de source para formatos antiguos; extender inspector
existente (preferida); rechazar borrar por rango/prefijo sin mapping.
Dependencias: C para revisiones; B usa resolución; E/F no son requisito.

**FMEA y aceptación manual futura:** falta chunk → texto incompleto → estado
MissingChunk, no concatenación exitosa; parentless correcto → falso positivo
de auditoría → contar unidades explícitas; delete parcial → preview sensible
→ retirada lógica y job reanudable; dos fuentes con basename idéntico →
cruce → ID canónico y scope antes de resolver. Casos: leer handle devuelve
orden correcto, retirar handle no toca otra fuente, retirar un chunk no
destruye otro, retry converge, fallo de proyección permanece visible.
Observabilidad: cantidades/estados/códigos, no previews ni paths privados.
**Exclusiones:** no prometer purga de backups/clientes, GDPR completo, ni
rediseñar IDs de todo Vanta por imitar sufijos upstream.

### Ficha E — relevo temporal preciso sin doble vigencia accidental

**Caso agente:** “el responsable era Ana; desde el día D es Luis”. Dos
operaciones separadas con fecha sin hora pueden dejar ambos como vigentes
en D o cerrar el anterior sin publicar sucesor.

- **Evidencia SHA/rango:** `KnowledgeGraph::_temporal_filter_sql:285-314`,
  `add_triple:464-582`, `invalidate:584-614`, `supersede:616-730` [M9].
  Fecha-only de inicio se expande al inicio del día, fin al final; consulta
  usa límite superior estricto. `supersede` convierte la frontera a un
  instante preciso compartido y cierra/abre dentro de `with conn`.
- **Delta Vanta:** writer ya materializa valid_at y lifecycle marca
  supersession [V1]; no se propone crear timestamps ni grafo SQL. Falta
  acreditar contrato unificado para fecha parcial, extremo y reemplazo en
  consulta histórica. Extender decisiones de `l1_dedup` y publicación del
  escritor, no insertar una segunda tabla de hechos.

```rust
// Contrato de dominio propuesto, no implementación.
struct FactHandover {
    old_id: String, expected_revision: u64, successor: FactCandidate,
    boundary: ExplicitInstant, source_event_id: String,
}
fn plan_handover(request: FactHandover) -> Result<HandoverPlan, TemporalError>;
```

**Invariantes/errores:** misma frontera `[start,end)` para ambos; no convertir
“fecha desconocida” en now; timestamp de conocimiento separado; fuente y
ocurrencia identificables. InvalidBoundary, UnknownTime, RevisionConflict,
UnsupportedAtomicPublish y StorageFailure son distintos. Upstream degrada
a add cuando no hay old abierto [M9]; **no** imponerlo como default Vanta:
el contrato propuesto exige decidir explícitamente esa política.

**Ownership/commit:** dominio planifica sin LLM; escritor/SDK publica
relevo bajo revisión. No cerrar viejo si no puede publicarse sucesor con
garantía acordada. El `with conn` upstream no acredita atomicidad de
Vanta `persist_planned` delete→put [V1:289-294]; esto es prerrequisito
solo del modo mutante. Dos relevos simultáneos con misma base entran en
conflicto, no crean dos actuales silenciosamente. Cancelación antes del
commit no cambia ninguno; después se informa committed, no rollback ficticio.

**Pasos/alternativas/dependencias:** acordar precisión/extremos → función
pura → diagnóstico sobre campos existentes → publicación segura → filtro
histórico/inspección si se aprueba. Alternativa: solo preservar dos
versiones con superseded_by actual; preferida antes de migración temporal
global. Legacy Unknown permanece Unknown; no backfill desde filed_at.
Reindex temporal solo si filtro lo necesita; rollback detiene nueva
semántica sin fusionar ocurrencias históricas. Relación con Graphiti:
relevo temporal y gate de recurrencia son complementarios, no candidatos
duplicados a implementar dos veces.

**FMEA/aceptación futura:** frontera ambigua → doble actual → precisión
explícita; replay → sucesor duplicado → source_event_id estable; disputa
multiobjeto → falsa contradicción → política de cardinalidad por predicado,
no todo distinto objeto es incompatible. Casos: instante anterior/exacto/
posterior, llegada tardía, end-only desconocido, conflicto y fallo de
publicación. Logs de acción/conflicto sin nombres/fechas personales.
**Exclusiones:** no decidir verdad automáticamente, ni afirmar bitemporalidad
completa o portar todo `KnowledgeGraph` por tener un método transaccional.

### Ficha F — preview de importación con conflictos explícitos

**Caso agente:** consolidar memorias de dos perfiles o importar una copia
histórica sin que un hecho importado reemplace silenciosamente el actual.
Es distinto del refresh de una sola fuente (C).

- **Evidencia:** `knowledge_graph_notice:56-74` advierte grafo legacy no leído;
  `open_graph:77-85` no crea archivo en lectura sin fuente;
  `import_graph_rows:94-193` [M12] abre source read-only, preview sobre copia
  en memoria, copia solo columnas declaradas, distingue present/conflicts/
  id_conflicts y hace commit solo con apply. No fusiona grafos por defecto.
- **Delta/seam:** `vanta-memory/src/ingest/merge.rs::CandidatePage`,
  `CommitReport` [V5] solo expresan merge de **páginas wiki**, no import de
  hechos. Para F, mapear entidades a registros `MemoryInput` publicados por
  `Embedded::put` [V-publish-record], resolver extremos al ID real y usar
  `Embedded::add_edge:113-151` [V-publish-edge] para relaciones compatibles.
  `add_edge` escribe extremos secuencialmente; no recibe revisión de plan ni
  metadata arbitraria por arista. Preview/validación/ownership de hechos son
  delta de aplicación no acreditado, no capacidad inferida de un DTO wiki.
  No portar SQLite backup ni otro migrador.

```rust
// Propuesta explícita; no es el CommitReport actual.
struct ImportPreview {
    source_revision: String, target_revision: u64, plan_digest: String,
    additions: Vec<ImportItem>, conflicts: Vec<ImportConflict>,
}
fn preview_import(input: ScopedImport) -> Result<ImportPreview, ImportError>;
fn apply_preview(plan: ImportPreview, decision: AuthorizedDecision)
    -> Result<ImportReceipt, ImportError>;
```

**Contrato:** preview no escribe fuentes, no crea namespace ausente ni
descarga embeddings. Mismo ID/distinto contenido es IdentityConflict;
misma afirmación compatible es Present; contradicción vigente es PolicyConflict
y no last-write-wins. No inferir autorización de pertenencia al mismo equipo.
Scope y decisiones aprobadas quedan asociados al plan; errores también
SourceChanged, TargetChanged, NotAuthorized y UnsupportedSchema.

**Ownership/commit/concurrencia:** ingestor posee plan serializable y SDK
publicación; aprobación humana no congela target. Apply revalida ambas
revisiones/digest; si cambian, preview nuevo. El preview en memoria upstream
no promete automáticamente que apply posterior use el mismo snapshot.
Cancelación previa no muta, después informa recibo y progreso; commit por
unidad existente, nunca afirmar lote atómico por nombre.

**Pasos/migración:** distinguir páginas wiki de registros/relaciones →
mapear IDs y fuentes por los puertos anteriores → clasificar conflictos read-only →
plan versionado → revisión/aprobación → aplicar/revalidar → reindex derivados
solo de registros incorporados. Rechazar esquemas no soportados; no adivinar
columnas ausentes como permiso o fecha cero. Rollback deshabilita apply,
no elimina memorias importadas sin una retirada autorizada posterior.
Alternativas: preview en adapter para formato externo, export/import sin
merge para primer mínimo, rechazo de fusión entre scopes no autorizados.
Depende de identidad/revisión A/C; E aporta clasificación temporal cuando
exista, no bloquea preview básico. Relacionado con D para resolver soportes.

**FMEA/aceptación futura:** colisión ID → overwrite → clasificación conflict;
target cambia tras preview → aprobación stale → TargetChanged; source cambia
mientras se lee → mezcla → revisión/digest; import de gold de eval → fuga
→ trazabilidad y exclusión de corpus de evaluación. Casos: mismo archivo
rechazado, preview no crea destino, duplicado compatible, intervalo distinto,
conflicto explícito y retry sin duplicado. Telemetría solo cantidades/estados.
Un fallo de `put` o de la segunda escritura de `add_edge` no se anuncia como
import completo: recibo por unidad y relación pendiente, sin borrar fuente o
memoria manual ajena. Aceptación C/F: un `CommitReport.written` de wiki nunca
se usa como prueba de fuente/grafo publicado; preview no invoca esos puertos.
No se acredita atomicidad ni CAS por estas firmas; no se implementa otro runtime.
**Exclusiones:** no crear ahora formato universal, política de resolución
automática ni nuevo harness de evaluación.

### Ledger de cobertura

| Fuente | Archivos/rangos de código inspeccionados | Cobertura |
|---|---|---|
| [M1] | `backends/base.py:153-207,462-582`, firmas opcionales | scopes, identidad y contrato |
| [M2] | `backends/rust_exact.py:1-199` | elegibilidad de nativo/fallback, snapshot e hidratación |
| [M3] | `convo_miner.py:78-172` | ruta live y metadata |
| [M4] | `palace/writes.py:1-140,150-228` | vector reuse y publicación recuperable |
| [M5] | `searcher/query.py:315-375`, búsqueda de funciones/filtros | piso raw y closets |
| [M6] | `crates/mempalace-core/src/lib.rs:105-215,335-405` | carga SQLite y exact scan paralelo |
| [M7] | `benchmarks/longmemeval_bench.py:53-80,146-243,3148-3208` y selección de modos/split | métricas y runner |
| [M8] | `website/.vitepress/theme/Landing.vue:1-29` | sitio Vue, no dashboard |
| [M9] | `knowledge_graph.py:285-314,464-730` | precisión, filtro temporal, invalidación y relevo; no todo el KG |
| [M10] | `mcp_server/tools_write.py:99-139,489-625,697-712,1229-1251` | handle lógico/chunks y resolución |
| [M11] | `palace/closets.py:278-329` | cleanup posterior al cambio de drawers |
| [M12] | `knowledge_graph.py:56-193` | notice, lectura sin crear archivo, preview e import explícito |
| [M13] | `longmemeval_bench.py:2826-2889,2907-3003,3129-3257,3265-3441` | denominadores, flags, split, errores y efectos |

También README, CLAUDE, licencia y pyproject. No inspeccionados íntegramente:
todos los backends remotos, MCP HTTP/auth, resto del grafo temporal, model eval, loaders
de cada formato, binding PyO3 y toda heurística de hybrid_v2/v3/v4.

## 3. Defectos, riesgos e issues transferibles, con su nivel de evidencia

- **Código:** identidad y namespace tienen contrato explícito; no equivale a
  prueba de conformance de cada adaptador. Un fallback sin filtro sería crítico.
- **Código:** ruta live usa `filed_at` al generar ID [M3:130-131]; sin estudiar
  la receta y llamador no se puede asegurar idempotencia del mismo evento reenviado.
- **Código:** benchmark raw rellena rankings con índices no retornados por
  Chroma [M7:235-239]. Si el corte supera los hits reales, cuenta orden artificial;
  distinguir retrieval de ese relleno antes de interpretar recall.
- **Hipótesis:** archivos viejos/derivados y caches nativos podrían tener ventanas
  de visibilidad diferentes. Requiere prueba de actualización concurrente.
- **Issues:** el intento inicial de GitHub MCP falló; la ampliación siguiente
  sí leyó una incidencia y una propuesta pública relacionada, sin deducir merge.

### Incidencia pública leída y contraste con el SHA

Consulta del cierre, **2026-11-07**:
[issue #2752](https://github.com/MemPalace/mempalace/issues/2752),
“Persistent store: chroma.sqlite3 embeddings rows lag committed writes
(parentless chunk rows); API unaffected”, Open en la página, abierto
2026-10-15; observaciones del reportante del 2026-10-14, MemPalace 3.10.0,
Linux/Python 3.12. Reporta filas chunks sin parent y presume retraso de
materialización pese a que get/list/search funcionan. **Hecho publicado:**
125 grupos sin parent y 3084 filas parent en su snapshot; no es medición propia.
**Causa alegada:** lag del store; no queda demostrada por ese conteo.

La [propuesta #2958](https://github.com/MemPalace/mempalace/pull/2958)
se leyó en el listado API público de issues/PR: Open, creada
2026-11-07T07:40:55Z, updated 08:04:33Z, `merged_at=null` en el objeto
`pull_request`. Propone diagnósticos separados de drawers/chunks/rows y
explica que parentless es representación intencional. Sus tests declarados
son relato del autor, no ejecutados aquí. No se incorpora esa implementación
propuesta a la cobertura del clon ni se declara resuelto el issue.

**Contraste estático:** `tool_add_drawer` [M10] escribe IDs de chunks y
devuelve handle lógico; `_logical_chunk_group` hidrata por `parent_drawer_id`.
El mecanismo de ausencia de fila parent sí está explicado por el SHA fijado.
No recomendar flush, persist adicional o parent artificial como “arreglo”.
Transferencia: D y auditoría Studio distinguen unidad lógica, fila y proyección;
la aceptación futura es que auditoría e hidratación concuerden con el contrato,
no que el número de embeddings sea igual al de recuerdos.

**Recibo de acceso:** búsqueda keyless `site:github.com/MemPalace/mempalace/issues
benchmark` sin resultados; fetch del listado REST permitió leer #2958.
GET `/issues/2752` devolvió `API rate limit exceeded`; fallback a la página
HTML oficial permitió leer cuerpo/estado. No se leyó historial completo de
comentarios/PR ni se infiere exhaustividad del listado limitado.

## 4. Prerrequisitos en VantaDB antes de incorporar la idea

Revalidar código actual y cambios concurrentes, especialmente storage/search.
Definir unidad raw recuperable, retención, borrado e identidad del modelo.
Confirmar qué contratos ya cumplen ingestors/checkpoints y evitar duplicarlos.
Todo scope debe aplicarse antes de retrieval/expansión y sobrevivir a fallbacks.
Embeddings y abstracción LLM deben seguir siendo opcionales.

## 5. Componentes y practicas que no deben trasladarse

No reemplazar escenas/personas por alas/habitaciones. No imponer verbatim como
único nivel: L0 preserva evidencia y L1 abstrae con procedencia. No importar
AAAK como compresión sin pérdida. No trasladar boosts o preguntas heurísticas
del benchmark a producción sin validación independiente. No crear un segundo
índice exacto Rust olvidando HNSW/BM25/RRF ya existentes en Vanta.

## 6. Advertencias metodologicas

Rama `develop` y clon superficial: no representa automáticamente una release.
Sin historial anterior/tags/submódulos/LFS descargado; sin punteros LFS en manifest.
MIT raíz, copyright MemPalace Contributors 2026; revisar notices y dependencias
de cada módulo/modelo. Rust/PyO3 no exime de licencia ni demuestra equivalencia.
README/CLAUDE describen intención; recuperación efectiva requiere ejecución.

## 7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos

`longmemeval_bench.py` construye por pregunta un corpus desde
`haystack_sessions`, IDs y fechas; raw/session concatena **solo turnos user**,
raw/turn crea documento por turno user [M7:163-241]. No es QA: no genera respuesta
ni usa juez en esa ruta. `recall_any` es indicador de al menos un gold session,
`recall_all` de todos; nDCG usa relevancia binaria [M7:53-80]. Un R@5 publicitado
no debe sustituir esas definiciones por «porcentaje de preguntas bien respondidas».

Otros modos incluyen habitaciones, hybrid, variantes con nombres/frases/fechas,
palace y diary; rerank LLM opcional Anthropic/Ollama. No mezclar raw local
sin llamadas LLM con diary/rerank de coste/modelo diferente. El script permite
split dev/held-out con archivo `lme_split_50_450.json`; su existencia no prueba
que todas las heurísticas históricas se ajustaran solo con dev.
Dependencias: Chroma, embeddings y posible descarga de modelos; LLM/clave
solo según modo. Costes reales de descarga/import/embed/rerank no medidos aquí.

`_fresh_collection` borra/recrea colección por pregunta y silencia error del
borrado; no ejecutarlo sobre storage compartido. No se acredita cierre/cleanup
completo ni clasificación exhaustiva de errores de todos los modos. El log de
resultados contiene pregunta, respuesta gold y previews: nunca reingerirlo
como memoria de test. La comparación Vanta se rige **solo** por
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks); no se diseñó otro runner.

### Cierre de lectura del runner: defaults, población y coste

Cadena [M13]: CLI `3265-3441` → `run_benchmark:2935+` → constructor por modo
→ `evaluate_retrieval` → listas por k → JSONL. Default **raw/session**,
limit=0, skip=0, rerank=false, embed-model=default (help: Chroma
all-MiniLM-L6-v2); no equiparar con el modelo multilingüe lazy del producto.
La opción upstream hybrid-weight=0.30 se registra, **no se prescribe a Vanta**;
el help menciona ajuste sobre 500 preguntas: no prueba un protocolo holdout
histórico limpio. Default de rerank es Anthropic y modelo
`claude-haiku-4-5-20251001`; nombres no garantizan disponibilidad.

Split se filtra **antes** de limit y skip; limit se aplica **antes** de skip,
por lo que “limit N, skip M” no significa N preguntas después de M.
`_load_or_create_split` usa seed=42/dev_size=50 si debe crear archivo; sin
`--dev-only`/`--held-out` se toma todo, aun pasando split-file. Este informe
no crea ni cambia split. Dev/holdout reales necesitan acreditar el JSON usado.

En `3129-3131` ranking vacío hace SKIP y **no añade fila ni cero a métricas**.
Las medias `3224-3231` dividen por listas de preguntas evaluadas, mientras
`elapsed / len(data)` usa seleccionadas, incluidas saltadas. Si todas quedan
vacías, las medias pueden dividir por cero. No publicar completitud ni coste
por completada a partir de esa salida. Session-level mapea IDs de turnos a
sesión; turn-correct considera relevantes todos los turnos de una sesión gold,
no evidencia exacta anotada por turno (`3148-3158`). Las unidades difieren.

El reranker promueve un candidato de un pool, no genera respuesta QA; timeout,
respuesta inválida o error de API pueden devolver ranking original
(`2826-2889`) sin indicador separado en la fila de resultado. Ollama apunta
por defecto a localhost pero acepta base-url y modelos cloud: “Ollama”
no prueba aislamiento local. Diary puede reutilizar cache de otro modelo;
precompute y cache deben declararse al interpretar tiempo, no son gratis.
No se midió warmup/cache fría ni coste total por fase; el total del loop no
es latencia pura de retrieval. Guardado de cache y JSONL añade escrituras.

**Adaptación limitada:** un adapter Vanta transformaría solo los
haystack_sessions/IDs/fechas a fuentes L0 autorizadas, conservaría mapping
session/chunk y devolvería IDs ordenados desde el recall existente. Gold y
logs quedan fuera de ingest. La pista QA y el juez no se inventan para este
runner; la pista motor tampoco se deduce de R@k. Debe conservarse el registro
de omitidas/errores según METHOD, sin editar aquí métricas, prompts o flags.

## 8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB

| Capacidad | Estado | Diferencia bajo estudio |
|---|---|---|
| Evidencia L0/sesiones/procedencia | Existe | piso raw autorizado frente a derivados |
| Escenas/context assembly | Existe | organización, no metáfora adicional |
| BM25/HNSW/RRF | Existe | no sustituir por exact scan nativo |
| Ingest/checkpoints/scheduler | Existe | Wiki worker liga closures a `WikiStore`; convergencia por fuente C no acreditada por ese flujo |
| Handles/procedencia de fuentes y recibos ingest | Parcial para C/D | `source_message_ids` [V1], `Embedded::put` [V-publish-record]; recibo wiki [V5] no prueba mapping lógico-chunk end-to-end |
| Precisión temporal y relevo indivisible | Parcial para E | timestamps/supersession existentes [V1]; no certificar publicación de ambos extremos |
| Preview read-only de hechos entre revisiones | No acreditado para F | `put`/`add_edge` [V-publish-record][V-publish-edge] son puertos por unidad, no preview ni commit multirrecurso; merge wiki es dominio distinto |
| Identidad persistida de modelo en toda ruta | No acreditado aquí | revalidar guards actuales |
| Conformance entre todos los backends externos | No aplicable al motor embebido | útil como principio de adaptadores |
| Recall perfecto / exactitud QA del README | No acreditado | claim upstream, no resultado Vanta |

## 9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados

Preguntas: ¿un derivado puede excluir silenciosamente raw?, ¿cambio de modelo
con igual dimensión se detecta?, ¿reingest fallido deja estado recuperable visible?,
¿borrado invalida previews y embeddings auxiliares? No se afirma fallo vigente:
METHOD reconoce capacidades ya implementadas y el worktree está sucio.
La deuda potencial es verificar composición e interoperabilidad, no sumar capas.

## 10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado

Se inspeccionó fuente Vue de landing [M8]: compone secciones de marketing y
documentación. No acredita un visor operativo de recuerdos. CLI/MCP son
interfaces programáticas; dashboard/desktop transaccional **no localizado en
esta muestra**, no ausencia absoluta. Studio Vanta ya existe en `develop-desktop`.

**Pendiente manual:** backend conformance, mismatch de modelos, scopes bajo
fallback, reingest interrumpido, borrado de derivados, validez de split y
costes/errores de cada runner. No se ejecutaron tests, builds, servicios,
benchmarks ni validadores documentales; tampoco commits.

### Flujo Studio propuesto, no aplicación MemPalace inventada

Extender `memorias`/`indices`/`actividad`/`retrieval` del Studio existente
(rutas acreditadas en [VANTA-REFERENCE](VANTA-REFERENCE.md), worktree
`develop-desktop`, snapshot `0ad38c542`), no la landing Vue:
seleccionar conexión+namespace+handle → backend resuelve revisión y chunks
autorizados D → detalle distingue raw/derivado y completo/incompleto → actividad
enlaza import C o preview F. El apply de F necesita backend y revisión esperada;
un botón no aporta atomicidad ni permisos. Unidades lógicas/filas se muestran
separadas según #2752; no presentar parentless válido como pérdida.

Error/timeout no se dibuja como cero recuerdos. Respuesta tardía al cambiar
scope debe descartarse; preview y textos cacheados se reautorizan al abrir.
Aceptación futura: navegación no mezcla namespaces, raw retirado no aparece
por preview, import cancelado no informa aplicado, e intervalo E muestra
Unknown donde no hay precisión. No se ejecutó frontend ni se aprobaron acciones
mutantes. Este flujo complementa las fichas, no añade otra plataforma UI.
Para C/F la UI distingue página wiki escrita, fuente persistida y relación
pendiente: el recibo wiki no acredita hecho aplicado. Selección/revisión del
plan permanecen del backend. Studio se apoya en los cuerpos releídos de
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409,488-550`,
`activity/ActivityPanel.tsx:52-89` y
`lens/retrieval/RetrievalLens.tsx:138-164,193-228` bajo ese mismo prefijo y
SHA completo `0ad38c542b39a589035b0e17fdedb4c09dca24b9`; no bytes dirty.
Corrección [MP-1](reviews/memory-code-remaining.md#mempalace) aplicada,
pendiente de closure independiente.
Fuentes Studio inmutables de esa lectura (2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L550),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

[V-publish-wiki]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L244-L301
[V-publish-record]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1240
[V-publish-edge]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/graph.rs#L113-L151

## 11. Inventario ampliado, relaciones y división futura

**Cobertura de los diez ejes:** contexto y prioridades §0–1; extracción/fallos/
Rust §2 A–F; incidencia pública §3; prerrequisitos §4 y cada ficha; rechazos
§5; método/licencia §6; runner/defaults/población §7; matriz §8; negativos
condicionados §9; interfaz real y seam Studio §10. Estado de autoría, no revisión
independiente ni aceptación final.

**Relaciones/no proliferación:** A valida espacio/modelo; B garantiza una
ruta raw elegible, no recall perfecto; C publica generación de una fuente;
D resuelve unidad lógica/cleanup; E releva hechos sin ambigüedad de extremos;
F consolida fuentes distintas con conflictos. C y F comparten revisiones pero
no tienen misma entrada ni decisión. E se consolida con temporalidad Graphiti,
no dos motores temporales. El guard de snapshot de A no sustituye
autorización de B/D. Las seis unidades son propuestas, no código implementado.

**Split plan (>300 líneas):** conservar dossier y ledger; solo tras aprobación
separar specs capacidades/embedding (A), recuperación/handles (B/D), import
por generación/preview (C/F) y temporalidad compartida (E). No se crean specs,
IDs ni backlog en este cierre.

**Lagunas restantes:** resto de backends y contratos remotos/auth; atomicidad
real por implementación; cierre de cache nativa bajo escritores múltiples;
extracción automática de KG, loaders y binding PyO3 íntegros; otros runners
de calidad y procedencia histórica del ajuste de heurísticas; purga de backups
y caches externos. La lectura nueva de supersede/import elimina su antigua
clasificación como totalmente no leído, no equivale a auditar todo el KG.
No se crean harnesses ni se ejecutan casos manuales.

**Cierre por archivo:** preservadas A–F; completadas referencias faltantes
M0/M9–M13/V1/V5, ledger, TOC/§11 y límites de población/errores de LongMemEval.
Fuentes públicas nuevas efectivamente leídas: #2752 HTML y cuerpo #2958 del
listado REST. Sin evidencia de merge ni resultados propios; la revisión
independiente permanece a cargo de la fase posterior.

[M0]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/pyproject.toml#L1-L100
[M1]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/backends/base.py#L153-L582
[M2]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/backends/rust_exact.py#L1-L199
[M3]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/convo_miner.py#L78-L172
[M4]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/palace/writes.py#L150-L227
[M5]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/searcher/query.py#L315-L375
[M6]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/crates/mempalace-core/src/lib.rs#L105-L405
[M7]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/benchmarks/longmemeval_bench.py#L146-L243
[M8]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/website/.vitepress/theme/Landing.vue#L1-L29
[M9]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/knowledge_graph.py#L285-L730
[M10]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/mcp_server/tools_write.py#L489-L712
[M11]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/palace/closets.py#L278-L329
[M12]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/knowledge_graph.py#L56-L193
[M13]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/benchmarks/longmemeval_bench.py#L2907-L3441
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L112-L620
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L16-L49
