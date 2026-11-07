---
title: "SurrealDB: fronteras de rebuild y diagnóstico de compatibilidad"
kind: research
status: proposed
description: "Lectura estática de barreras locales, rollback-check y benchmark embebido con cautela explícita de licencia BSL"
tags: [research, memory, engineering]
---

# SurrealDB: fronteras de rebuild y diagnóstico de compatibilidad

## Índice

- [0. Snapshot, arquitectura, modelos y flujos](#0-snapshot-arquitectura-modelos-y-flujos)
- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura)
- [2. Evidencia y extracciones mínimas independientes](#2-evidencia-y-extracciones-mínimas-independientes)
- [3. Riesgos, defectos e hipótesis](#3-riesgos-defectos-e-hipótesis)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Benchmark upstream realmente leído](#7-benchmark-upstream-realmente-leído)
- [8. Matriz frente a Vanta](#8-matriz-frente-a-vanta)
- [9. Carencias sugeridas, no bugs certificados](#9-carencias-sugeridas-no-bugs-certificados)
- [10. UI, escritorio e inspección](#10-ui-escritorio-e-inspección)
- [11. Cierre de autoría, cobertura y lagunas](#11-cierre-de-autoría-cobertura-y-lagunas)

## 0. Snapshot, arquitectura, modelos y flujos

Se conservan las candidatas A/B, la lectura de licencia y el runner de la
ampliación parcial. Este cierre añade transacciones, publicación/estado de
derivados, efectos postcommit, incidencias públicas y fuente de UI externa.
Fecha de consulta pública: **2026-11-07**; no equivale a la fecha del código.
No se actualizan clon, inventarios ni metadata congelada.

| Superficie | Fuente leída al SHA `9577ac5f0caa60a39d5a334e9352aed0e232f9e6` | Responsabilidad y límite |
|---|---|---|
| API embebida | `surrealdb/engine-local/src/lib.rs:1-45` [S1] | SDK → engine-api → Datastore; interfaz interna sin estabilidad patch |
| Configuración embebida | `engine-local/src/lib.rs:65-155`; `Cargo.toml:15-110` [SCFG][SDEP] | Tokio, canales async, capacidades, backends `kv-mem`/`kv-surrealkv` y opciones explícitas |
| Entrada de consulta | `core/src/kvs/ds.rs:12039-12060` [SEXEC] | `execute/process` construyen `QueryRequest` y llaman `run`; no se audita toda VM/planner |
| Transactor | `datastore/src/tx.rs` [STGET][STSET][STDEL][STCOMMIT] | Claves tipadas/bytes y write set; cancelación/commit diferentes del cierre del cliente |
| Escritores y barrera | `datastore/src/triggers.rs` [S3][S4], `core/src/kvs/write_barrier.rs` [S2] | Epochs locales antes del snapshot; coordinación multinodo explícitamente excluida de adaptación |
| Índices online | `core/src/kvs/index/mod.rs:1-32`, `builder.rs:4049-4123` [SINDEX][SBUILD] | Admisión, scan, replay, generación y prueba de barrera |
| Estado persistido | `datastore/src/values/index_build.rs:20-205` [SSTATE] | Generación/owner/fase, cursor y base de construcción versionados |
| Lectura de estado | `datastore/src/index_state.rs:1-180` [SHEALTH] | Estado reportado y backlog materializable, no profundidad de cola de notificaciones |
| Compatibilidad | `core/src/kvs/rollback.rs:1-261` [S5][S6] | Reporte read-only sobre ledger, catálogo y generaciones; no downgrade |
| Evaluación | `engine-local/benches/point_read.rs:1-152` [S7] | Microbenchmark de petición serial in-memory, no calidad de memoria |

Los paths de la tabla son relativos a `C:\Users\Eros\surrealdb\surrealdb`,
salvo cuando empiezan por `surrealdb/`. Motor/core y código SDK del mismo árbol
no se convierten en MIT por ser Rust: véase BSL en §6. Dependencias como Tokio,
canales, SurrealKV o un backend remoto requieren su propio análisis legal.
Esta lectura no incorpora TiKV/Raft ni un servicio de base de datos al producto.

### 0.1 Modelo, identidad y confianza

La frontera de consulta acepta texto o AST, sesión y variables, y devuelve
`Result<Vec<QueryResult>, TypesError>` [SEXEC]. El runner además comprueba
`QueryResult.result`: éxito de transporte no acredita éxito de cada sentencia.
Una sesión con namespace/database no es una sesión conversacional ni autoriza
por sí misma todas las operaciones. `Session::owner` del benchmark es privilegio
de preparación, no modelo recomendado para un agente hostil.

El almacenamiento leído es KV con claves tipadas, decoders versionados y
registros identificados por tabla/clave. Los índices son derivados mantenidos
desde escrituras. `BuildGeneration` identifica intento de construcción;
`BuildTicket` ordena admisión por transacción y `BuildTicketMutationSeq`
distingue cambios del mismo ticket [SSTATE]. Ninguno sustituye a una revisión
semántica de fuente Vanta, identidad de tenant, autorización o provenance.
Los DTOs de índices describen ejecución/mantenimiento, no recuerdos.
Embeddings, extracción LLM, answerer y juez no participan en estos tramos.

### 0.2 Escritura, lectura, update/delete y commit trazados

1. Host/SDK selecciona backend y configuración; `engine-local` expone el
   Datastore [S1]. En el tramo `LocalConfig::datastore_config` leído se parte de
   `ConfigMap::empty`, se agregan opciones y se trata la allowlist de carpetas
   separadamente [SCFG]. No inferir que cada ruta servidor usa esa política.
2. Entrada pública `execute/process` → `run(QueryRequest...)` [SEXEC].
   No se trazó cada instrucción SQL hasta su operador: el siguiente tramo
   inspeccionado es la API transaccional, no una cadena end-to-end certificada.
3. `Transaction::set_key` reserva escritura, codifica clave/valor con nivel y
   llama `write_encoded`; `set` usa bytes [STSET]. Update de clave y creación
   no son automáticamente supersession de un recuerdo.
4. `get_key` llama `get_encoded` y hace `transpose` del decode [STGET]:
   error de almacenamiento/decodificación no es `None`. `getu_key` rechaza una
   transacción read-only porque allí no habría validación al commit de la
   lectura que registra conflicto. No afirmar aislamiento universal desde ello.
5. `del_key/del` reservan escritura y pasan a `delete_encoded` [STDEL].
   `del_reclaimed` distingue soft-delete de expunge para un prefijo retirado:
   borrado lógico, retiro de catálogo y recuperación física no son equivalentes.
6. `commit` → `commit_body`: comprueba poison, almacena deltas de índice y
   changefeed **antes** del commit subyacente; fallos previos llaman cancel y
   conservan el error original [STCOMMIT]. Después arma guards, admite commit
   y llama `commit_store_bounded`. Esto no acredita fsync de todos los backends.
7. `cancel_body` limpia buffers, pide cancel al transactor, termina inscripción,
   ejecuta rollback/release y descarta acciones postcommit [STCANCEL]. Error de
   cancel mantiene incertidumbre: no se documenta como descarte confirmado.
8. Tras commit exitoso, libera reservas, descarta rollback actions, ejecuta
   commit actions y notifica async/index-compaction/graph-fold/reclaim
   [STPOST]. Notificación es activación de trabajo, **no** prueba de finalización.

`commit_store_bounded:7856-7870` [STBOUND] sólo aplica su timeout en stores
compartidos; para sole-writer llama directamente `commit_store`. Un timeout
compartido registra commit abandonado y devuelve `CommitOutcomeUnknown`.
No trasladar sus constantes de lease/timeouts a Vanta embedded.
`end_live_write` usa swap idempotente; `park_live_write` permite esperar un build
sin esperarse a sí mismo, con reinscripción al soltar su guard [STEND].
El comentario local de barrera [S2] cuenta antes de adquirir snapshot y hasta
retorno de commit/cancel; no se recorrió cada constructor/Drop de transacción.

### 0.3 Derivados: generación, reanudación y visibilidad

`index/mod.rs` [SINDEX] separa estado `!bs`, tickets `!bt`, cambios `!bg`,
punteros `!bp` y reservas `!br`. Escritores admitidos en una generación
encolan cambios; Closing impide nuevas admisiones mientras resuelve los previos.
El planner debe usar Online, no simplemente una definición presente.
En Error se sigue encolando salvo restricciones de UNIQUE descritas en el módulo;
no deducir que todo fallo mantiene siempre disponibles todas las escrituras.

`wait_for_older_writes` [SBUILD] inicia barrera, observa abort, comprueba `done`,
registra prueba de generación y renueva ownership mientras espera.
`rescan_without_barrier_proof` reinicia cursor/scan cuando takeover encuentra
progreso sin prueba correspondiente. Un cursor persistido no acredita por sí
solo una cobertura completa. `IndexBuildState.initial_cursor` se persiste con
su batch; su `serde(skip)` advierte que un roundtrip serde pierde ese campo
[SSTATE]. No copiar este formato; sí conservar la relación progreso↔trabajo.

`index_has_unmaterialised_updates` [SHEALTH] pregunta existencia de pendientes
reales por tipo de índice: una notificación por índice no cuenta sus registros,
y una cola de triggers vacía no prueba backlog vacío. El comentario describe
lecturas que combinan estado materializado y deltas; su corrección completa y
coste no se verificaron recorriendo todos los lectores ANN/fulltext.

Para Vanta sólo se consume el contrato hipotético
[C13 MutationReceipt](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén):
`operation_id`, `primary_commit`,
`primary_durability: Option<DurablePosition>`, `projections`,
`visibility_generation`, `outcome`. **Dimensiones independientes**:
`accepted`/`applied`/`durable` no son orden total ni estados excluyentes.
Primario durable puede preceder a readiness; no exigir fsync individual de cada
derivado. Epoch de writer, cursor de rebuild y backlog no son DurablePosition.
Timeout/cancelación no prueban rollback ni autorizan una reinserción ciega.

## 1. Recomendación y arquitectura

**Dictamen:** estudiar contratos de publicación de índices y preflight de
compatibilidad; no importar SurrealQL completo, arquitectura de servidor ni
código BSL como si tuviera licencia permisiva.
Clone: `C:\Users\Eros\surrealdb`.
SHA congelado: `9577ac5f0caa60a39d5a334e9352aed0e232f9e6`.
Snapshot: **33 289 estrellas, `NOASSERTION`**, del [inventario](github-candidates.json).
`NOASSERTION` es el resultado del inventario, no permiso ni ausencia de licencia.
La licencia raíz leída es **Business Source License 1.1**, detallada en §6.
Preparación: [manifiesto](clone-manifest.json); clon sin actualizar. El cierre
sí consulta fuentes públicas concretas (§3/§10), sin recalcular el inventario.

El README declara motor multimodelo Rust, embebido, servidor y WASM [S0].
Hay evidencia de separación concreta: `engine-local` abre un Datastore y
sirve al SDK por `engine-api`; el core entra al seleccionar backend [S1].
Su interfaz entre crates se declara interna y sin garantía de estabilidad,
incluso en patches. No tratarla como API pública reutilizable estable.

```text
SDK Rust → engine-api → engine-local → Datastore → backend elegido
escritura → registro de epoch antes de snapshot → commit/cancel → fin de registro
rebuild → definición/generación → espera de escrituras anteriores → lectura
rollback_check(target) → ledger + formatos/catálogo → reporte, sin modificar
```

El flujo de barrera está documentado en [S2] y parcialmente implementado en
los rangos LiveWrites [S3]-[S4]. El preflight tiene código de veredicto y
recorrido [S6]. No se trazaron todas las consultas, escrituras ni backends.
Aunque el proyecto soporta escenarios distribuidos, el mecanismo interesante
para Vanta es la frontera **local** entre writers y estado derivado.

**Vanta:** referencia fijada `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`;
se descartan las revisiones históricas como prueba del estado actual.
Se conserva la [referencia discriminante](VANTA-REFERENCE.md) y las capacidades
acreditadas: ANN/BM25/sparse/RRF, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints/ingestors/auditoría y Studio.
Se releen particiones, migración y rebuild/reportes sobre ese snapshot mediante
`git show`, sin checkout ni checks globales. No se presenta un hallazgo histórico
como bug vigente ni se audita todo Vanta.

| Orden cualitativo | Decisión | Dependencia específica |
|---|---|---|
| Primero, read-only | C: explicar generación consultable y pendiente real | Aprovechar auditorías/reportes existentes antes de modificar algoritmos |
| Primero si falta cobertura | A: frontera local de writers vs snapshot | Inventariar sincronización actual; no presuponer pérdida de escrituras |
| Independiente de A | B: diagnóstico por formato/operación | Reusar MigrationEngine; sin migración o downgrade implícito |
| Tras trazar una ruta real | D: distinguir efectos postcommit y wakeups | C13 y control de trabajo existente; no instalar otro scheduler/outbox sin necesidad |

## 2. Evidencia y extracciones mínimas independientes

| Ref. | Archivo y rango leído | Observación |
|---|---|---|
| S0 | `README.md:68-82` | Arquitectura declarada e imagen Surrealist |
| S1 | `surrealdb/engine-local/src/lib.rs:1-45` | Frontera del SDK embebido y estabilidad |
| S2 | `surrealdb/core/src/kvs/write_barrier.rs:1-60` | Contrato local y distribuido documentado |
| S3 | `surrealdb/datastore/src/triggers.rs:1476-1538` | open/close por epoch y begin_commit |
| S4 | `surrealdb/datastore/src/triggers.rs:1572-1625` | avance coordinado y consulta de finalización |
| S5 | `surrealdb/core/src/kvs/rollback.rs:1-165` | Fuentes leídas y tipos de hallazgo |
| S6 | `surrealdb/core/src/kvs/rollback.rs:165-261` | Veredicto y recorrido read-only |
| S7 | `surrealdb/engine-local/benches/point_read.rs:1-152` | Runner propio leído completo |
| S8 | `surrealdb/engine-local/Cargo.toml:118-134` | Registro bench sin harness |
| SL | `LICENSE:1-68` | BSL, restricción Database Service y Change Date |
| SCFG/SDEP | `engine-local/src/lib.rs:65-155`; `engine-local/Cargo.toml:15-110` | Configuración explícita y features/dependencias |
| SEXEC | `core/src/kvs/ds.rs:12039-12060` | execute/process → run |
| STGET/STSET/STDEL | `datastore/src/tx.rs:3527-3579,8202-8251,4046-4087` | Lectura/decode, reserva y mutación |
| STCOMMIT/STCANCEL/STPOST | `tx.rs:3284-3478,3017-3052,3479-3507` | Error primario, cancel y efectos postcommit |
| STBOUND/STEND/STACTIONS | `tx.rs:7856-7873,9331-9360,12734-12750` | Outcome incierto, ownership y callbacks fuera de lock |
| SINDEX/SBUILD/SSTATE/SHEALTH | `index/mod.rs:1-32`; `builder.rs:4049-4123`; `values/index_build.rs:20-205`; `index_state.rs:1-180` | Generación, barrera y pendiente real |
| V1/V2/VADMIN/VREBUILD/VAUDIT/VL1/VWORK | Enlaces al final, todos al `97c9623...` | Backend, formatos y rutas existentes, no ausencia global |

El contrato documental S2 y su implementación parcial S3/S4 están separados;
no se infiere verificación completa del protocolo distribuido.

### Candidata A — frontera local de escritores durante rebuild

- **Problema:** un índice reconstruido no debe omitir escrituras iniciadas bajo
  una generación anterior que aún pueden confirmar tras comenzar el escaneo.
- **Unidad mínima:** inscripción de writer antes de snapshot, identidad de epoch
  y espera sobre escritores previos; S3/S4 acreditan ese núcleo conceptual.
- **Seam Vanta:** particiones derivadas y `InternalMetadata` del backend [V1],
  migración/formato de índice ya existente [V2], más auditoría del propietario.
- **Contrato hipotético:** `WriteEpochGuard { generation, operation_id }`
  y `RebuildBoundary { source_generation, prior_writers_settled }`.
- **Acción 1:** inventariar fences/generaciones y publicación actual de rebuild;
  no crear otro coordinador si el motor ya expresa la misma frontera.
- **Acción 2:** identificar la toma real de snapshot y el último punto en que
  una escritura antigua puede modificar el estado fuente.
- **Acción 3:** especificar inscripción y cierre ligados a esa frontera;
  cancelación de un future no equivale a commit/cancel del almacenamiento.
- **Acción 4:** exponer motivo de espera/estado en auditoría existente antes
  de alterar el algoritmo de construcción de índices.
- **Dependencias:** orden de snapshot/registro, resultado de commit/cancel,
  autoridad para publicar generación y coordinación con GC ya existente.
- **FMEA — inscripción tardía:** writer invisible al fence confirma después;
  la inscripción debe preceder al snapshot que determina la generación.
- **FMEA — cierre temprano:** Drop del llamador libera guard mientras una
  escritura en background sigue activa; el propietario real conserva el guard.
- **FMEA — espera sin fin:** writer abandonado mantiene rebuild bloqueado;
  preservar diagnóstico y estado, no declarar índice sano por timeout.
- **FMEA — bloqueo global innecesario:** afecta namespaces no relacionados;
  ámbito del contrato debe coincidir con el estado derivado mantenido.
- **Aceptación propuesta:** publicación cubre cada escritura confirmada de la
  frontera elegida o mantiene la generación anterior; no hay intervalo omitido.
- **Aceptación operativa:** espera y cancelación no anuncian progreso durable
  que el backend no confirmó; error deja estado de salud interpretable.
- **Independencia:** aplicable sin inspección de downgrade B.
- **No objetivos:** claves heartbeat, leases multinodo, Raft ni copiar S3/S4.
- **Licencia:** extracción conceptual; cualquier reutilización material queda
  detenida hasta revisión legal específica de BSL y del archivo.

### Candidata B — reporte read-only de compatibilidad por formato

- **Problema:** saber que una base abre no garantiza que todos sus índices,
  registros o operaciones puedan ser leídos/mantenidos por un binario anterior.
- **Unidad mínima:** separar veredicto de apertura, formatos afectados,
  objetos ilegibles/desconocidos y restricciones de escritura.
- **Fuente:** S5/S6; el veredicto usa ledger, mientras generaciones/entries
  detallan rechazos posteriores sin cambiar automáticamente dicho veredicto.
- **Seam Vanta:** `FormatKind` ya enumera File, VectorIndex, WAL, Records y
  Schema; el orden de migración de records precede al schema bump [V2].
- **Contrato hipotético:** `CompatibilityReport { target, open_mode,
  format_findings, unknown_count, unreadable_count, observation_scope }`.
- **Acción 1:** inventariar reports/preflights actuales y versiones por formato.
- **Acción 2:** declarar qué compatibilidad puede inferirse de cabeceras y cuál
  exige recorrer estructuras; un campo desconocido no significa compatible.
- **Acción 3:** preservar categorías de daño, formato futuro y no inspeccionado.
- **Acción 4:** integrar diagnóstico read-only con migración existente,
  sin ejecutar reparación ni rebajar versión como efecto de inspección.
- **Dependencias:** decoders versionados, conocimiento de formatos publicados,
  acceso autorizado y frontera de observación consistente.
- **FMEA — falso seguro:** sólo se valida apertura y un índice sigue ilegible;
  detallar alcance y operaciones restringidas.
- **FMEA — desconocido tratado como vacío:** omite material incompatible;
  unknown debe permanecer visible y no autorizar escrituras.
- **FMEA — informe mezclado:** lecturas sucesivas ven generaciones distintas;
  declarar snapshot/alcance o indicar que el diagnóstico no es atómico global.
- **Aceptación propuesta:** informe no modifica bytes, no repara ni promete
  downgrade; distingue «abre», «sólo lectura» y «no acreditado» donde proceda.
- **Aceptación de seguridad:** reportes no filtran nombres/payloads de namespaces
  que el operador no puede inspeccionar.
- **Independencia:** útil sin rebuild online ni candidata A.
- **No objetivos:** nuevo formato de disco, un migrador automático o copiar ledger.
- **Licencia:** mismo límite conceptual; no código derivado autorizado aquí.

### Precisiones de integración y contrato de A/B

**A — seam revalidado:** `Embedded::rebuild_index` [VADMIN] encadena reconstrucción
vectorial, derivados, texto, sparse y escalar; `rebuild_derived_indexes_with_report`
[VREBUILD] recorre particiones y nodos, escribe un batch y después su marcador.
`rebuild_text_index_with_report` hace lo propio con postings/estadísticas.
No es ausencia de rebuild ni prueba de fallo concurrente: estos métodos aislados
no acreditan todas las exclusiones/callers del motor. El delta es describir la
frontera cubierta y, sólo si falta, reforzarla en esas rutas.

Entrada hipotética A: scope autorizado, generación objetivo y operación;
salida `WriteEpochGuard` exclusivo/transferible al propietario real y
`RebuildBoundary`; errores tipados propuestos: epoch superado, scope no autorizado,
writer no resuelto, cierre del backend y almacenamiento. No liberar el guard
al cancelar un wrapper si el commit aún puede aplicar. La barrera se observa
fuera de locks de índice y no cambia por sí sola C13 a durable/ready.
Alternativas: ventana exclusiva ya existente, snapshot backend válido, o
publicación por generación si está disponible; no imponer rebuild online a un
motor cuyo contrato operativo ya exige exclusión.
Migración mínima: diagnóstico aditivo; sólo si se persiste generación, versionar
metadata existente y mantener desconocido el pasado sin prueba.
Desactivar el diagnóstico no cancela un commit ni publica un índice incompleto.
Métricas por fase/motivo, nunca payload o claves privadas.

**B — seam revalidado:** `FormatKind`, `MigrationPlan`, `MigrationEngine` y
`RecordsBackfillReport` [V2] ya distinguen versiones y dry-run.
Entrada propuesta: scope autorizado, versiones objetivo por formato y modo de
inspección; salida `CompatibilityReport` con alcance, restricciones por objeto
y categorías no inspeccionado/ilegible/desconocido. Errores: objetivo no soportado,
acceso denegado, decoder/IO y observación invalidada por cambio de fuente.
Alternativas: ampliar dry-run/reporte actual o limitarse a cabeceras marcando
desconocido el contenido; rechazar un «downgrade seguro» inferido de semver.
Reporte efímero por defecto, sin migración; serialización opcional versionada
no sustituye formatos del motor. Rollback de la presentación no modifica datos.
B no depende de A; sólo requiere declarar si la observación fue snapshot único.
Aceptación adversa: fallo de lectura nunca se vuelve «compatible», ni siquiera
cuando el target abre la base. Error y unknown permanecen visibles sin nombres
de namespaces no autorizados.

### Candidata C — diagnóstico de publicación y backlog por proyección

- **Caso agente:** una captura ya confirmada aún no participa en búsqueda;
  el operador debe distinguir índice no publicado, generación vieja, deltas
  pendientes y error de inspección antes de repetir ingesta o reconstruir.
- **Fuente/unidad mínima:** fases y cursor de [SSTATE], reporteo/backlog de
  [SHEALTH], prueba de generación y rescan de [SBUILD]. No copiar `!bs`/tickets.
- **Delta Vanta:** hay reportes rebuild, marcadores InternalMetadata y auditoría
  textual superficial/profunda [V1][VREBUILD][VAUDIT]. Ampliar el significado de
  esos reportes, no inventar ANN/BM25/RRF o health desde cero.
- **Contrato Rust propuesto:** `inspect_projection(scope, expected_generation)
  -> Result<ProjectionObservation, ProjectionInspectError>`, con generación
  observada, alcance de fuente, readiness, existencia conocida/desconocida de
  pendientes y motivo de error. Se adjunta a `projections`/`visibility_generation`
  de C13 cuando corresponda; **no** reemplaza `MutationReceipt`.
- **Invariantes:** readiness tiene evidencia de la generación descrita;
  contador de wakeups no equivale a registros; `NotInspected` no es cero;
  primario durable no retrocede a «no guardado» por índice pendiente.
- **Pasos:** (1) inventariar reportes/marcadores presentes; (2) anotar qué prueba
  cada campo; (3) resolver bajo un alcance consistente o declararlo no atómico;
  (4) ampliar auditoría/SDK; (5) consumir read-only desde Studio.
- **Alternativas:** documentar el reporte existente si suficiente; flag de
  existencia del pendiente cuando contar requiera escaneo completo; no introducir
  sondeos costosos de todo el índice para dibujar una barra de progreso.
- **Ownership/concurrencia/cancelación:** el inspector no toma propiedad de
  builders ni repara; comparar generación observada antes de ofrecer una acción;
  cancelar inspección no modifica el índice ni completa tareas.
- **Persistencia/deps:** marcadores y reportes actuales; observación read-only
  sin nuevo layout. A aporta prueba de frontera, pero C puede reportar
  «cobertura no acreditada» antes de implementar A. B es independiente.
  Desactivar panel/reporte no elimina marcadores ni evidencia de error.
- **FMEA:** empty trigger queue → falso ready → contrastar pendiente real por
  proyección → indicar desconocido si no se inspeccionó; fase/cursor de distintas
  generaciones → falso progreso → sellar alcance/generación; reparación desde
  diagnóstico → pérdida → separar operaciones y autorización.
- **Aceptación manual futura:** con proyección retrasada el receipt conserva
  primary confirmado; error de lectura no muestra backlog cero; takeover/cambio
  de generación no hereda porcentajes del anterior. Sin contenido en labels;
  un indicador de existencia se rotula como tal, no como medida de profundidad.
- **Mínimo/no objetivos:** una proyección y auditoría existente; no compactor,
  nuevo scheduler, promises de latencia, ranking ni inspección forense global.
  Abierto: qué datos permiten los backends sin escaneo adicional.

### Candidata D — efectos postcommit diferenciados de la notificación

- **Caso agente:** tras persistir un turno se despierta extracción/indexación;
  fallar o perder ese wakeup no debe convertir un commit confirmado en rollback
  ni permitir que una tarea observe datos aún provisionales.
- **Fuente:** `commit_body` almacena deltas/changefeed antes de commit [STCOMMIT],
  luego ejecuta acciones y notificaciones [STPOST]; `run_commit_actions`
  toma la lista fuera del lock y ejecuta cada acción [STACTIONS].
  `cancel_body` descarta efectos provisionales [STCANCEL].
- **Unidad/delta:** mapear hechos de efecto a la ruta de escritura y scheduler
  **existentes**. Vanta `plan_write`/`persist_planned`/`apply_dedup_batch` [VL1]
  separa planificación, put y efectos; esa separación no acredita una
  transacción única para todo el batch. No proponer atómicamente L1+broker+LLM.
- **Contrato Rust propuesto:** `observe_followup(operation_id, source_revision,
  effect_kind) -> Result<FollowupObservation, FollowupError>` devuelve identidad
  del trabajo existente, fuente requerida, observación pendiente/aplicado/
  desconocido y referencia al C13 del primario. Un wakeup es sólo un evento
  operativo; no DurablePosition ni prueba de ejecución.
- **Pasos:** (1) trazar un put→enqueue→handler real; (2) clasificar qué trabajo
  ya es persistido y qué sólo es notificado; (3) correlacionar operación/revisión;
  (4) ampliar resultado/diagnóstico sin alterar retries; (5) sólo con una laguna
  demostrada, evaluar registro durable en el mismo dominio que el primario.
- **Alternativas:** scheduler/job ya persistido, reescaneo reconstruible,
  invalidación síncrona existente o simple documentación. Outbox nuevo sólo si
  ninguno conserva la obligación necesaria; esta autoría no lo implementa.
- **Ownership/errores:** store conoce commit; worker posee intento; cliente sólo
  espera. Errores propuestos: revisión superada, tarea desconocida, storage y
  acceso denegado; error secundario de notificación no oculta resultado primario.
- **Concurrencia/cancelación:** worker revalida revisión antes de publicar efecto;
  cancelar espera no deshace commit; no invocar callbacks externos bajo mutex.
  Outcome incierto se resuelve por operación, no reejecutando efecto no idempotente.
- **Persistencia/deps:** C13 y tarea actual; metadata opcional compatible y
  estado legado desconocido. Rollback de instrumentación no borra la fuente ni
  obligaciones persistidas. Relación con Redis: settlement/intento allí,
  orden commit→activación aquí; no dos subsistemas de cola.
- **FMEA:** wakeup antes de commit → lectura provisional → notificar después;
  crash commit→wakeup → trabajo omitido → verificar job persistido/reconciliación;
  callback falla tras commit → reingesta duplicada → preservar receipt primario;
  worker antiguo sobre revisión nueva → derivado stale → comprobar revisión.
- **Aceptación manual futura:** no se activa derivado de escritura cancelada;
  commit confirmado con fallo de activación sigue confirmado y muestra followup
  pendiente/desconocido; la misma tarea repetida no duplica un efecto de revisión.
  Métricas de fase/resultado sin texto ni IDs sensibles como labels.
- **No objetivos/abierto:** exactly-once LLM, atomicidad global, nuevo changefeed,
  runtime SurrealDB, copiar handlers BSL o garantizar fsync con un `notify_one`.
  Pendiente definir qué obligaciones requieren supervivencia a crash.

## 3. Riesgos, defectos e hipótesis

**Estático S3:** open cuenta antes de entregar epoch; close reduce el contador;
begin_commit se cuenta antes de leer fences. **S4:** avance bloquea shards
en orden y comprueba epochs anteriores. No se certifica todo el lifecycle
de Transaction ni todos los puntos que invocan estas funciones.
Su corrección depende de invariantes externos no recorridos.

**S2 documental:** el caso multinodo depende de heartbeats, marcas de versión,
leases y acuerdo de relojes dentro del margen descrito.
Es una condición del protocolo, no fallo observado; no se traslada a Vanta.
**Hipótesis de adaptación:** un guard liberado al cancelar el wrapper mientras
persiste el commit produciría una frontera falsa.

**Estático S6:** rollback_check abre lecturas separadas para raíz/namespace/DB;
no acredita un snapshot único de toda la base bajo escrituras concurrentes.
No es defecto demostrado: el resultado debe interpretarse según ese alcance.
Veredicto `Opens` no equivale a compatibilidad de todas las generaciones.
S5 cuenta claves/valores ilegibles sin inventar un nombre fiable para ellos.

Fronteras de confianza: formato persistido → decoder; operador → namespaces;
writer → generación publicada; código upstream → producto Vanta/licencia.
No CVEs atribuidas ni reproducciones. Las incidencias públicas del cierre
se distinguen debajo de los hechos estáticos y las hipótesis de adaptación.
No se declara que Vanta tenga pérdidas de escrituras o downgrade defectuoso.

### Incidencias públicas efectivamente leídas

Consulta **2026-11-07**, cuerpos oficiales por REST; sin ejecutar los repros.
Son reportes de versiones/configuraciones concretas, no bugs certificados del
pin ni de Vanta.

- [#7552 — concurrent writes during DEFINE/REBUILD INDEX](https://github.com/surrealdb/surrealdb/issues/7552):
  **cerrado**, 2026-10-20. Reporta entradas stale y algunas ausentes tras
  `ready`, en 3.2.4 (`93ab219`), 3.3.0 y nightly `df34ba7`, con writers que
  cambian valores indexados; contrasta queries con índice y `WITH NOINDEX`.
  Los resultados/tablas son del autor, no mediciones nuestras. Su análisis de
  tickets/scan/replay no acredita por sí mismo causa única. El
  [comentario del mantenedor](https://github.com/surrealdb/surrealdb/issues/7552#issuecomment-6283251799)
  afirma fix en main y test de concordancia; no identifica aquí PR/SHA/release
  verificado. No se deduce corrección sólo del cierre. El pin sí tiene espera
  de writers y prueba de generación/rescan [SBUILD], evidencia diferente del
  reporte histórico: no basta para certificar todas sus intercalaciones.
  Transferencia A/C: exigir concordancia de la generación publicada con la
  fuente; aceptación futura con escrituras antes/durante/después del rebuild,
  no copiar SQL ni umbrales del repro.
- [#7523 — index stranded in building: cleaning](https://github.com/surrealdb/surrealdb/issues/7523):
  **abierto**, observado actualizado 2026-11-05. TiKV, estado heredado de
  3.1.6 y presente en 3.2.4; describe WriteConflict → error → cleaning,
  `REBUILD` que vence y `REMOVE` que queda esperando. El autor no ofrece
  receta fiable para **entrar** de nuevo al estado en 3.2.4: sí describe
  operaciones sobre la instancia ya afectada. Cambiar valor indexado frente
  a escribir el mismo valor discrimina el camino; reinicio/upgrade reportados
  no equivalen a reparar generaciones antiguas. No PR/fix acreditado aquí.
  El pin separa fases y estado/backlog [SSTATE][SHEALTH], pero no se auditó
  la recuperación TiKV ni se homologa `cleaning` a todas sus fases actuales.
  Transferencia B/C: distinguir prevención de nueva inconsistencia y diagnóstico
  de estado previo; nunca recomendar borrar/rebuild destructivo automáticamente.

Acceso: DuckDuckGo no devolvió resultados para la búsqueda focal; metasearch
localizó #7552 (algunos providers 403/timeout). Se validaron las páginas/API
oficiales, no los snippets. Cuerpo de #7552 y comentario leídos; no se auditaron
otros issues/PRs citados como si fueran fixes confirmados.

## 4. Prerrequisitos Vanta

1. Revalidar fences/rebuild y reports actuales sobre el HEAD concurrente.
2. Conservar auditorías ANN/texto/WAL/backend y sus estados de salud.
3. Definir propietario real de escrituras cuando sobreviven al llamador.
4. Inventariar versiones de formatos sin confundirlas con versión de paquete.
5. Preservar invariantes de migración ya presentes; un preflight no debe reparar.
6. Revisar datos disponibles para Studio y bindings antes de ampliar contratos.
7. Revisar licencia antes de cualquier reutilización material o dependencia.

| Candidata | Gate concreto / relación | No bloquear por |
|---|---|---|
| A | Frontera de snapshot/callers de rebuild [VADMIN][VREBUILD], dueño que conserva guard hasta resolver escritura; C13 sólo para describir mutaciones | B, UI o protocolo multinodo |
| B | Versiones/decoders y modo read-only de MigrationEngine [V2]; declarar alcance si no hay snapshot común | A o scheduler nuevo |
| C | Semántica de marcadores/reportes [VREBUILD][VAUDIT], acceso autorizado; si A no existe reportar cobertura desconocida | Implementar A antes de mostrar diagnóstico honesto |
| D | Una ruta real `persist_planned`/worker [VL1][VWORK], identidad/revisión y obligación persistida o reconstruible; referencia C13 propuesto | B/C completos o nuevo WAL/outbox |

## 5. Qué no trasladar

No portar el lenguaje SurrealQL completo ni catálogo/VM/multimodelo integral.
No importar servidor, auth de usuario final o scripting al motor por analogía.
No copiar heartbeats/fencing distribuido para resolver rebuild de un proceso.
No asumir estabilidad de `engine-local` por estar escrito en Rust.
No confundir publicación de un índice con durability del almacenamiento fuente.
No considerar BSL «Apache futura ya vigente» ni sustituir asesoramiento legal
por una reescritura nominal.

## 6. Método, licencia y cobertura

Lectura local de cuerpos fijados con `git show`, consultas públicas focales
(§3/§10) y redacción; sin scripts upstream, instalación, build,
tests, apps/servicios, benchmarks ni git de escritura.
Rangos S0-S8/SL más SCFG–STACTIONS y seams Vanta al `97c9623...` definidos
al cierre; no se presentan V1/V2 históricos como revalidados.
Omitidos: transacciones completas, totalidad de index builders, HNSW/DiskANN, planner,
SurrealQL, auth/RBAC, RPC, WASM, SDKs restantes y recuperación física.
Historial superficial, otras ramas/tags y LFS según manifiesto.

**Licencia material:** SL dice BSL 1.1, Licensed Work SurrealDB 3.0,
Additional Use Grant que excluye uso como «Database Service» según su definición,
Change Date **2030-01-01**, Change License Apache-2.0.
El propio texto declara que BSL no es licencia Open Source.
No se resuelve aquí cómo aplica cada cláusula a Vanta, distribución o servicios.
La conversión futura tiene condiciones por versión; no autoriza hoy una copia
permisiva. Código derivado y redistribución requieren revisión específica.
La investigación de mecanismos no aprueba una dependencia ni una transposición
de código. Reimplementar una idea no elimina automáticamente riesgo de derivación.

## 7. Benchmark upstream realmente leído

**S7:** `surrealdb/engine-local/benches/point_read.rs`, registrado con
`harness=false` [S8]. Es un programa de medición propio, **no Criterion**.
Usa Tokio current_thread, Datastore memory, mantenimiento desactivado y sesión
owner en namespace/DB de bench; no arranca servidor en ese código.
Dataset sintético: 1 000 registros person con name, age y tags.
Dos esquemas: schemaless/schemafull; cuatro formas de SELECT por ID, llave
variable, literal variable y literal fijo.

Por defecto: 2 000 warmups y 20 000 solicitudes medidas por caso;
`BENCH_ITERS`, `BENCH_WARMUP` y `BENCH_STATEMENT_CACHE` son controles existentes.
No se propone modificar sus valores ni introducir configuración.
IDs variables usan `(i*7919)%1000`; literal fijo lee person:1.
Cronometra execute y extracción de resultado; informa **media y mediana en μs**.
Comprueba una fila en la primera iteración de cada caso, no una auditoría de
correctitud completa por solicitud.

Dependencias: toolchain Rust y crates core/types/cnf/Tokio, backend memory
disponible y features de compilación apropiadas; nada se compiló para probarlo.
No dataset externo, embeddings, generación ni juez.
Statement cache, ruta compilada y esquema forman parte del trabajo medido.
No mide ANN/recall, persistencia en disco, caída de proceso, throughput paralelo,
cliente de red, p95/p99 o permisos de usuario no-owner.
No atribuir resultados de benchmarks de SurrealKV o ANN a este runner.

Se localizaron `core/benches/index_ann.rs`, `engine-local/benches/scan.rs` y
otros, pero no se leyeron como runners: sin atribución de métricas.
No hay resultado medido propio ni superioridad frente a Vanta.
**Población/fallos:** `case:90-132` aborta con `unwrap` ante error de execute
o sentencia, y `assert` comprueba una fila sólo en la primera iteración
(normalmente warmup). No retry, score cero ni descarte de una solicitud fallida:
no se obtiene una corrida completa si aborta. Media usa `samples.len()`,
mediana `samples[len/2]` (superior para cardinal par); no percentil interpolado.
Un override de cero iteraciones no produce una muestra válida.
`main:135-152` crea store por esquema y lo comparte entre los cuatro casos:
no reset por petición/caso, cache calentada y preparación heredada. Escrituras
de 1 000 registros y construcción del esquema quedan fuera del tiempo; dentro
execute y extracción de resultado, fuera preparación de variables/texto.
No llamadas pagadas, descargas ni limpieza de una base externa en ese runner.

**Adaptación concreta Vanta:** reutilizar
[`Embedded::get:1611-1625`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1611-L1625),
con `(namespace aislado, key person/<n>)` → `Option<MemoryRecord>` y payload
name/age/tags verificable. Prellenar con SDK fuera del tiempo; `None`, error
o contenido distinto no es lectura exitosa. El punto-get SDK no ejecuta SQL:
las cuatro formas de SELECT se colapsan al mismo lookup y no pueden servir
para comparar parsers/statement cache/schemafull. Conservar secuencia de IDs,
warmup/muestras y shape lógico; reportar explícitamente serialización/adaptación
incluida y backend/caches. Sin ANN, LLM, juez o transacción nueva.
Es una adaptación futura al tooling de medición existente, no un harness
autorizado ahora. [Protocolo común](METHOD.md#protocolo-de-comparacion-de-benchmarks)
complementa, no sustituye esta correspondencia.
Sin nuevos prompts/jueces/evals, thresholds, sampling ni configuraciones.

## 8. Matriz frente a Vanta

| Capacidad | Estado Vanta | Delta defendible |
|---|---|---|
| ANN/BM25/sparse/RRF | Existe | No importar otro motor multimodelo |
| Índices derivados y marcadores de salud | Existe [V1][VREBUILD] al snapshot fijado | Revisar explicación de frontera rebuild |
| Migración por formato | Existe V2 | B amplía diagnóstico, no crea migrador |
| Provenance/sesiones/supersession/dedup | Existe | Conservar referencias durante rebuild |
| Assembler/compaction/checkpoints/scheduler | Existe | No reemplazar con runtime SurrealDB |
| Barrera local registrada antes de snapshot en toda ruta | No acreditado aquí | A condicionada a inventario actual |
| Preflight uniforme de downgrade por formato | No acreditado aquí | B sólo si falta alcance equivalente |
| Integración end-to-end de los contratos candidatos | Parcial: seams, no trazado actual | Revalidación previa |
| Diagnóstico de generación y pendiente real | Parcial: rebuild/auditoría [VADMIN][VAUDIT] | C amplía evidencia, no barra desde número de wakeups |
| Efectos después del primario | Parcial: L1/worker [VL1][VWORK] | D correlaciona obligación y C13; no commit multialmacén acreditado |
| UI operacional | Existe Studio [VSTUDIO] | A–D amplían inspectores existentes; no copiar Surrealist |
| SurrealQL/consenso completo como requisito | No aplicable | Excluido |

## 9. Carencias sugeridas, no bugs certificados

Dos preguntas acotadas: ¿la auditoría explica qué escritores cubre un rebuild
y qué generaciones se publicaron?, ¿un reporte distingue apertura de lectura/
mantenimiento efectivo por formato?
No se prueba ausencia de barreras ni incompatibilidad de datos en Vanta.
La mejora mínima puede consistir en ampliar reportes actuales.
La licencia hace preferible estudiar invariantes y no copiar implementación.
Toda tarea futura exige revalidar estado concurrente y decisión del propietario.

## 10. UI, escritorio e inspección

README incluye imagen **Surrealist** [S0], insuficiente por sí sola.
Se inspeccionó además código del repo **externo** `surrealdb/surrealist`,
SHA `65b7b95109177deb486cc6254ed203fce257fddf`, consulta 2026-11-07,
sin clonar/actualizar inventario. [SUIPKG] acredita Vite, React/UI y scripts
Tauri; [SUILIC] es MIT del cliente, distinta de BSL del motor y no licencia
transitiva universal. No se acredita distribución firmada ni runtime desktop.

[SUIVIEW] traza QueryView: intent `run-query` → `executeEditorQuery`,
editor activo/selección → ResultPane. [SUIEXEC] selecciona texto y delega;
[SUICONN] acumula frames/resultados por sentencia, distingue error y libera
estado activo en finally. Pestañas/configuración local no son memoria del agente.
Se estudia separar sentencia enviada, resultado y error; no un botón que
prometa rollback de escritura al cerrar una vista. QueryPane usa CodeMirror
y lectura/escritura de consulta en [estrategia config/fichero][SUISTRATEGY],
no sólo imagen.
No se auditó auth/secrets, todos los adapters, cloud o telemetría end-to-end.

Vanta Studio ya existe: [VSTUDIO] fija
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:1-90`
al `0ad38c542b39a589035b0e17fdedb4c09dca24b9`, auditEvents/filtros/errores.
A/C mostrarían frontera/generación y pendiente; B restricciones por formato;
D followup pendiente separado del primario. Acciones de repair autorizadas
fuera del inspector read-only. Los bytes actuales movidos/modificados de
`desktop/...` y JobChip/jobs **no** son evidencia de ese SHA.
Runtime necesario: **ninguno**; no iniciar/reiniciar servicios.

## 11. Cierre de autoría, cobertura y lagunas

Se conservaron A/B y sus contratos; se completaron fuentes de flujos C/D,
incidencias, prerequisitos individuales, tratamiento operativo del point-read,
matriz y UI real. La autoría no declara aceptación ni cierra su propia revisión.
**Leído:** rangos S0–S8/SL, configuración/consulta, get/set/delete,
cancel/commit/postcommit, estado/barrera/rescan y acciones; Vanta backend,
migración, rebuild/auditoría/L1/worker; Surrealist y Studio según enlaces.
**No leído completo:** protocolo distribuido, todos los backends/Drop/callers,
recovery físico, planner/RBAC, todo frontend/transitivas legales ni cada PR.
No pruebas de crash, rendimiento o calidad ni comandos ejecutados de evaluación.

Dependencias A→evidencia de frontera, B independiente, C puede declarar
desconocido sin A, D usa C13 propuesto y trabajo actual; ninguna obliga otro
WAL/scheduler. Validación manual futura: intercalaciones rebuild, pérdida de
observación del commit, decoder ilegible, backlog no inspeccionado y cancelación
postdispatch; con datos propios/desechables y autorización previa.
**División futura:** si se aprueban specs, separar barrera/publicación A/C,
compatibilidad B, followups D y anexo de fuentes/bench/UI, conservando enlaces
al dossier. No se crean esas tareas, harnesses o archivos ahora.

[S0]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/README.md#L68-L82
[S1]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/src/lib.rs#L1-L45
[S2]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/write_barrier.rs#L1-L60
[S3]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/triggers.rs#L1476-L1538
[S4]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/triggers.rs#L1572-L1625
[S5]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/rollback.rs#L1-L165
[S6]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/rollback.rs#L165-L261
[S7]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/benches/point_read.rs#L1-L152
[S8]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/Cargo.toml#L118-L134
[SL]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/LICENSE#L1-L68
[SCFG]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/src/lib.rs#L65-L155
[SDEP]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/Cargo.toml#L15-L110
[SEXEC]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/ds.rs#L12039-L12060
[STGET]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L3527-L3579
[STSET]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L8202-L8251
[STDEL]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L4046-L4087
[STCOMMIT]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L3284-L3478
[STCANCEL]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L3017-L3052
[STPOST]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L3479-L3507
[STBOUND]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L7856-L7873
[STEND]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L9331-L9360
[STACTIONS]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L12734-L12750
[SINDEX]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/index/mod.rs#L1-L32
[SBUILD]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/index/builder.rs#L4049-L4123
[SSTATE]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/values/index_build.rs#L20-L205
[SHEALTH]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/index_state.rs#L1-L180
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/backend.rs#L20-L100
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/migration.rs#L18-L126
[VADMIN]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/admin.rs#L16-L35
[VREBUILD]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_rebuild.rs#L16-L180
[VAUDIT]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L118-L245
[VL1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L160-L446
[VWORK]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L265-L390
[VSTUDIO]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L1-L90
[SUIPKG]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/package.json#L1-L100
[SUILIC]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/LICENSE#L1-L7
[SUIVIEW]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/screens/surrealist/pages/Connection/views/query/QueryView/index.tsx#L125-L199
[SUIEXEC]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/editor/query.tsx#L43-L78
[SUICONN]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/screens/surrealist/pages/Connection/connection/connection.tsx#L331-L550
[SUISTRATEGY]: https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/screens/surrealist/pages/Connection/views/query/QueryView/strategy.tsx#L13-L80
