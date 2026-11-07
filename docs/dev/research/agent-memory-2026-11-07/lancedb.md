---
title: "LanceDB: versiones, derivados y ejecución de retrieval frente a VantaDB"
kind: research
status: proposed
description: "Inspección estática de LanceDB fijado: arquitectura, escrituras, consultas, incidencias y siete mecanismos candidatos para extender VantaDB"
tags: [research, memory, engineering]
---

# LanceDB: versiones, derivados y ejecución de retrieval frente a VantaDB

## Índice

- [0. Identidad, arquitectura y flujos](#0-identidad-arquitectura-y-flujos)
- [1. Recomendaciones priorizadas](#1-recomendaciones-priorizadas)
- [2. Fichas de mecanismos candidatos](#2-fichas-de-mecanismos-candidatos)
- [3. Incidencias oficiales y seguridad](#3-incidencias-oficiales-y-seguridad)
- [4. Contratos previos de VantaDB](#4-contratos-previos-de-vantadb)
- [5. Rechazos, alternativas y costes](#5-rechazos-alternativas-y-costes)
- [6. Método y límites específicos](#6-método-y-límites-específicos)
- [7. Benchmarks inspeccionados](#7-benchmarks-inspeccionados)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Negativos sustentados y falsos positivos](#9-negativos-sustentados-y-falsos-positivos)
- [10. UI y VantaDB Studio](#10-ui-y-vantadb-studio)
- [11. Fuentes, cobertura y cierre](#11-fuentes-cobertura-y-cierre)

## 0. Identidad, arquitectura y flujos

### 0.1 Corte reproducible e incorporación expresa

| Campo | Evidencia y alcance |
|---|---|
| Proyecto | `lancedb/lancedb`, biblioteca/base de datos de retrieval; no framework de memoria conversacional |
| Checkout | `C:\Users\Eros\lancedb`, rama `main`, SHA `bff93ff314eee34f625d05bd92b73d95867fe787`; `git rev-parse HEAD` coincide con la instrucción del propietario |
| Captura separada | [github-additions.json](github-additions.json), 2026-11-07T17:16:49.828443Z: 11.857 estrellas, no archivado, Apache-2.0 |
| Selección | Excepción expresa del propietario bajo 30.000 estrellas; **no** se presenta como elegible del filtro original ni modifica [github-candidates.json](github-candidates.json) |
| Clon | [clone-additions-manifest.json](clone-additions-manifest.json): shallow, una rama, sin tags, sin inicialización de submódulos ni smudge LFS; metadata separada de los 50 originales |
| VantaDB | Base revalidada `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; `c561158253bb88192590fa406c61e0a27dd7df8d` es antecedente histórico, no base actual |
| Consulta pública | 2026-11-07; issues leídos mediante API oficial vía `websearch.fetchWebContent`, después de fallos de descubrimiento/MCP (§3) |
| Ejecución | Ninguna app, test, build, benchmark, instalación, script de proyecto o servicio; ningún resultado propio de performance |

**Licencia:** `LICENSE:1-202` declara Apache-2.0; los archivos Rust y runners muestreados contienen SPDX Apache-2.0 [LA]. El workspace declara la misma licencia [LB]. Python/PyO3, TS/napi-rs y Java son superficies distintas: la licencia raíz no sustituye inventario de dependencias, datos ni código remoto. Se estudian contratos; no se propone port source-to-source ni incorporar Lance/DataFusion al motor de VantaDB.

Comprobación por distribución: Python declara archivo `LICENSE` y classifier
Apache en `python/pyproject.toml:17,31` [LX]; Node declara Apache-2.0 en
`nodejs/package.json:35` [LY]; Java declara Apache-2.0 en
`java/pom.xml:21-26` [LZ]. Son declaraciones de esos componentes, no una
resolución de licencias transitivas. El servicio TypeSafe, sus modelos y el
corpus GooAQ mencionado por el runner no quedan relicenciados por esos avisos.

### 0.2 Mapa y stack realmente observado

```text
Python sync/async ─ PyO3 ─┐
TypeScript ─ napi-rs ────┼─ Table / BaseTable (Rust, builders)
Java bindings ───────────┘       ├─ NativeTable (proceso local)
                                │    ├─ Arrow RecordBatch / streams
                                │    ├─ planes DataFusion
                                │    ├─ DatasetConsistencyWrapper
                                │    └─ Lance Dataset: datos, índices, manifests
                                └─ RemoteTable / feature remote → servicio externo
```

| Área | Rutas y unidad leída | Responsabilidad / límite |
|---|---|---|
| Composición | `Cargo.toml:1-110` [LB] | Rust edition 2024/MSRV 1.95; Arrow 59.2.0, DataFusion 55, Tokio, object_store; Lance `15.0.0-beta.13` desde Git |
| API | `rust/lancedb/src/table.rs:940-998,1742-1747,2490-2501` [LC] | Versiones, checkout, restore, branches; builder merge |
| Estado de lectura | `table/dataset.rs:14-145,198-270` [LD] | Dataset compartido por `Arc`, estado bajo `Mutex`, pin, política de consistencia |
| Escritura | `table.rs:3821-3928`; `table/{merge,update,delete}.rs` [LE][LF][LG][LH] | Planificación y delegación al Dataset, recibos de versión y filas |
| Consulta | `query.rs:1517-1695`; `table/query.rs:334-425` [LI][LJ] | Dos ramas híbridas, proyección, filtros y stream de batches |
| Fusión | `rerankers/rrf.rs:19-149` [LK] | Unión por row ID, suma recíproca por rango, score de relevancia |
| Derivados | `table/freshness.rs:4-21,47-110` [LL] | Definición, firmas de entrada, excepciones de filas y sidecars inmutables |
| Bindings/docs | `python/`, `nodejs/`, `java/`, `docs/` | Mapa del checkout; no revisión exhaustiva de semántica entre lenguajes |
| Microbenchmarks | `python/benchmarks/bench_{typesafe,streaming_dataloader}.py` [LM][LN] | Reranking externo y lectura de filas, respectivamente |

**Frontera importante:** el checkout no contiene el código completo de la dependencia Lance ni del servicio Cloud. Que `execute()` devuelva un nuevo Dataset identifica la frontera API de commit observada; no prueba por sí solo orden de fsync, atomicidad multiobjeto o recuperación tras crash de Lance. El despliegue nativo puede abrir datos locales; las dependencias habilitan object stores. El backend remoto es otro contrato operativo, no el mismo proceso trasladado a HTTP.

### 0.3 Modelo de datos, identidad y capacidades

La unidad de entrada es un esquema Arrow y uno o más `RecordBatch`; no un turno de chat. Un vector se expresa como columna de lista de floats de tamaño fijo; `nearest_to` distingue falta de columna, múltiples columnas y datos no finitos en su contrato [LO]. Texto y metadatos son columnas, no una memoria que el motor deba interpretar mediante LLM. El llamador puede suministrar vectores; no hay obligación de usar un generador LLM.

Hay cuatro identidades que no se deben confundir:

1. **Tabla/namespace y branch:** coordenada lógica del dataset.
2. **Versión:** revisión seleccionada por un handle; `checkout(version)` documenta una vista de solo lectura.
3. **Row ID interno:** requerido por el RRF inspeccionado para unir hits; no equivale automáticamente a la clave de documento de una aplicación.
4. **Clave de merge:** columnas `on` declaradas por el llamador; la API no acredita por su mera presencia unicidad de negocio.

Los resultados distinguen `_distance` vectorial, score textual y `_relevance_score` fusionado [LI][LK]. La documentación de `refine_factor` explica distancia aproximada frente a la refinada con vectores sin comprimir [LO]; no interpretar toda distancia devuelta como exacta. Los filtros SQL/expresión y `Select::{Columns,Dynamic,Expr,All}` llegan al scanner [LJ]. Filtro de negocio y proyección son planificación, **no autorización**.

Features verificadas en la muestra: append/overwrite, update por expresión, delete por predicado, merge condicionado, checkout/versionado/restore, consultas vectoriales y FTS combinadas, RRF y firmas de frescura de columnas calculadas. La presencia de RTE/LSM y branches se identifica en [LC][LF], pero su semántica completa queda fuera: no atribuir la garantía del camino estándar al camino RTE.

### 0.4 Flujo concreto A: ingesta de documentos con vectores explícitos

Escenario: aplicación ya fragmentó un manual y entrega `doc_id`, texto, revisión y vector en batches Arrow.

1. `Table::add(...).execute()` llega a `NativeTable::add`; comprueba mutabilidad, rechazo de vista y restricciones de columnas calculadas [LE].
2. Lee esquema y Dataset; la planificación puede estimar particiones a partir del primer batch y de CPU. Convierte append/overwrite a `WriteParams`, construye plan y `InsertExec`.
3. Ejecuta particiones en tareas; drena los streams y propaga errores de cada batch o `JoinError`. El ownership es `Arc` para ejecución compartida; el reader/plan contiene datos de entrada.
4. Tras completar los handles, consulta `manifest().version`, actualiza la frescura del handle cuando procede y devuelve `AddResult { version }` [LE]. `bump_freshness` es no-op para tablas sin namespace [LP], no una invalidación universal de todas las cachés del producto.
5. Texto/vector quedan bajo el formato y commit de Lance, no en un docstore Python paralelo en este camino. La formación de embeddings fuera del proceso no está cubierta por ese commit.

**Error y cancelación:** fallo de schema/columnas se comunica antes de ejecutar esa etapa; fallo de una tarea no es un recibo exitoso. No se auditó `InsertExec` completo ni el commit subyacente: **no** afirmar all-or-nothing de todas las particiones, ni que cancelar el future revierta archivos o commits ya realizados. Un retry que repita append puede duplicar filas; la aplicación necesita identidad/reconciliación, no solo repetir por timeout. Índice materializado y dato durable son estados distintos.

### 0.5 Flujo concreto B: consulta híbrida con filtro y proyección

Escenario: recuperar un fragmento autorizado para responder una pregunta, sin cargar inicialmente columnas de blobs.

1. `VectorQuery::execute_hybrid` pasa por `wal_fusion::with_pk_fallback`, cuyo fallback completo no se auditó [LI]. En el camino estándar `run_hybrid` clona consulta FTS y vectorial, activa row IDs y retira FTS de la rama vectorial.
2. Ejecuta ambas ramas con `try_join!`; drena sus streams con otro `try_join!`; concatena batches por esquema. Por tanto, **esta fusión materializa candidatos**, aunque exponga un stream de salida.
3. El scanner aplica opciones de ANN, rango de distancia, prefilter, proyección, filtro y FTS [LJ]. Una proyección estrecha reduce las columnas solicitadas; no acredita que ningún operador intermedio lea otras.
4. El RRF de [LK] requiere row IDs en ambas entradas, acumula `1/(k+rank)` con rangos desde 1, une resultados y ordena por `_relevance_score`. `k=60` es default upstream observado, **no recomendación nueva**.
5. `DatasetConsistencyWrapper` diferencia lazy (`None`), strong (intervalo cero) y eventual (TTL, refresh de fondo). `new_query_snapshot` crea estado independiente fijado a la revisión seleccionada y conserva origen de pin [LD].

**Fronteras:** crear un snapshot no demuestra que todas las ramas de todos los builders lo utilicen. En el tramo híbrido leído no se certifica aislamiento multi-request end-to-end. Un error en una rama propaga error; no existe aquí evidencia de un fallback universal a la otra. El assembler del agente y el presupuesto del modelo no forman parte de esta consulta. Un score RRF es orden relativo, no probabilidad de que la respuesta sea correcta.

### 0.6 Flujo concreto C: modificación, borrado y lectura histórica

Escenario: sustituir fragmentos de la revisión de un manual y retirar los eliminados.

1. `merge_insert(on)` configura acciones matched/not-matched y opcional timeout. El camino estándar obtiene Dataset, construye `LanceMergeInsertBuilder`, ejecuta y recoge Dataset nuevo + estadísticas [LF].
2. Publica ese Dataset en el wrapper después de `future.await?`; devuelve versión y contadores. El camino LSM se despacha antes y es una frontera distinta, no auditada integralmente.
3. `execute_update` canonicaliza filtro, exige mutable, rechaza escribir columnas computadas, construye operación y solo después de éxito actualiza el wrapper [LG].
4. `execute_delete` exige mutable, canonicaliza SQL o acepta expresión, llama a delete y devuelve versión/filas; después publica el Dataset nuevo [LH].
5. `checkout` cambia la revisión visible del handle; `restore` usa la revisión seleccionada para escribir una nueva versión y volver a latest [LC][LQ]. Leer pasado, restaurar pasado y borrar físicamente versiones no son lo mismo.
6. Para derivados, [LL] describe firmas de entradas por fragmento, filas dirty y un sidecar inmutable `_computed/` referenciado por metadatos. Una modificación del origen no debe confundirse con un valor no nulo todavía vigente.

**Error real de contrato a investigar:** #4844 documenta merge sobre handle fijado que escribe latest (§3); `NativeTable::merge_insert` en el SHA leído no llama al guard de mutabilidad que sí está en add/update/delete. No prometer inmutabilidad por la documentación sola. **Timeout** en el wrapper no demuestra ausencia de commit: debe reconciliarse revisión/identidad antes del retry. **Borrado lógico** no acredita purga de backups/versiones/cachés; las firmas de frescura no son certificados de eliminación de datos.

## 1. Recomendaciones priorizadas

| Orden | Recomendación condicional | Motivo específico | Qué no implica |
|---|---|---|---|
| 1 | Recibo de escritura/reemplazo y estado de resultado desconocido (ficha C) | Merge devuelve revisión y estadísticas, mientras timeout no resuelve qué se persistió | No adoptar el algoritmo de merge ni prometer una transacción Vanta ya inexistente |
| 2 | Identidad de derivado con definición y revisión de origen (B) | Sidecars distinguen «presente» de «fresco» | No instalar Lance como segundo almacén de memoria |
| 3 | Revisión de lectura explícita para operaciones compuestas (A) | Pins y consistencia del handle son contratos distintos | No pedir MVCC/branches globales antes de justificar necesidad |
| 4 | Proyección por etapas con consumo acotado (D) | El híbrido puede materializar candidatos aunque salga como stream | No afirmar mejora de RSS/latencia sin medir |
| 5 | Explicación de plan y resultado efectivo (E) | Prefilter, refinamiento y score cambian lo que un diagnóstico significa | No recrear explain/RRF: Vanta ya los tiene |

Vanta dispone de híbrido lexical+dense+sparse con RRF [V1], explain con rangos y reporte [V2], chunks con provenance y claves deterministas [V3], assembler [V4], scheduler/jobs [V5] y Studio [S1][S2]. La prioridad es **extender estos seams**, no adoptar una base vectorial porque tenga las mismas primitivas.

**Ampliación conservadora:** se mantienen A–E y los dos runners originales.
La lectura adicional distingue dos unidades antes no trazadas: espera de
cobertura de índice (F) y finalización cercada de reparación parcial (G).
F es diagnóstico sobre los índices existentes; G solo se justifica si se
aprueba reparación por partes. No son requisitos para adoptar A/D/E.

### Traza complementaria: índice, refresh y commit del derivado

`NativeTable::create_index`, `table.rs:3931-3943` [LT], prepara y construye;
su variante async prepara **antes** de `tokio::spawn` y devuelve `Job`.
El tramo no demuestra que aceptar un job signifique índice disponible.
`wait_for_index`, `index/waiter.rs:16-89` [LU], lista índices y consulta
estadísticas hasta observar `num_unindexed_rows==0` para los nombres pedidos.
El polling usa un segundo y rechaza timeout superior a dos horas: defaults
upstream, no parámetros nuevos propuestos. El propio comentario reconoce que
escrituras continuas pueden impedir estabilizar; timeout no prueba corrupción.
No se inspeccionó el constructor ANN dentro de Lance.

`refresh_column`→`execute_refresh_column` exige feature experimental,
mutabilidad y ausencia de RTE en `table/refresh.rs:287-333` [LS].
Liga expresión y campos de entrada al Dataset leído, calcula staleness y
recorre fragmentos vivos. Un fragmento dirty se recomputa; otro se sondea para
no reescribir si no obtiene valores. Luego `write_columns` produce reemplazos.
`Dataset::commit(Operation::DataReplacement, Some(source_version), ...)`
publica los valores (`381-441`). **Después** intenta registrar frescura
(`444-467`): fallo de stamp se registra como warning y no falla el refresh ya
committed; faltará evidencia de frescura y se recomputará conservadoramente.
No describir valores y stamp como un único commit indivisible.

La cabecera `refresh.rs:10-18` describe dependencias en orden topológico y
un commit por columna; un append concurrente puede quedar para otro pass.
Esto acota el contrato documental, no acredita haber recorrido todos los
callers de dependencias o Functions/Enterprise. Cancelar antes del commit
puede dejar objetos preparados; cancelar después no revierte valores.

Separadamente, `freshness::{begin_row_repair,finish_row_repair}` [LV][LW]
valida fragmentos/firmas y marca dirty antes de publicar. El token contiene
basis, digest de intención y filas seleccionadas; finalización comprueba
pertenencia, revisión opcional, identidad de intención e inputs actuales.
`verified_version=None` **omite** fence de versión según contrato, no equivale
a CAS. Un resultado omitido o saltado no debe presentarse como reparación
completa. No se atribuye este camino a todo refresh de expresión.

**Camino remoto revalidado:** `RemoteTable::submit_create_index`,
`remote/table.rs:772-805` [LAA], extrae job ID de la respuesta. Con
`wait_timeout` y job ID espera **esa submission** mediante
`index_job(...).wait()` dentro de `tokio::time::timeout`; propaga error del
job. Solo si el servidor no devuelve ID recurre a `wait_for_index` por nombre.
Sin timeout no espera. Por tanto, el defecto histórico de #4402 no se puede
atribuir indiscriminadamente a este SHA: hay una corrección cliente concreta
y un fallback legacy aún menos fuerte (§3.3). Aceptación, ejecución de job,
cobertura de índice y visibilidad de lectura siguen siendo estados distintos.

## 2. Fichas de mecanismos candidatos

Todas las interfaces de esta sección son **hipotéticas**, propuestas para una spec posterior, no símbolos existentes ni tareas aprobadas. No son traducciones de código. No fijan presupuestos, umbrales de rendimiento ni nuevas configuraciones de evaluación.

### A. Coordenada de lectura estable para inspección compuesta

- **Problema/escenario:** inspector recupera IDs y después hidrata texto; una actualización entre ambos pasos puede mostrar el score de una revisión junto al contenido de otra.
- **Fuente/unidad mínima:** `DatasetConsistencyWrapper::{new_query_snapshot,ensure_mutable}`, `dataset.rs:101-128,215-226` [LD]. Extraer el principio de un handle de lectura ligado a una revisión, no su implementación con Lance.
- **Delta y seam real:** `Embedded::hybrid_search` y `explain_memory_search` [V1][V2] ya producen retrieval y explicación; añadir coordenada verificable para consumidores de varios pasos si el contrato existente no la comunica. El pin no reemplaza validación de permisos.
- **Alternativas:** primero extender la respuesta existente; si no hay snapshot disponible, validar versión al hidratar y devolver conflicto. Rechazar guardar una copia de toda la DB por consulta o inventar branch por sesión. Un adapter Studio solo puede detectar mezcla, no fabricar aislamiento del motor.
- **Interfaz propuesta:** `ReadCoordinate { scope, logical_revision }`; `hydrate_at(coordinate, ids) -> Result<HydratedBatch, ReadConflict>`. Errores diferenciados: revisión no retenida, cambio concurrente, acceso denegado, IO; no usar «no existe» para todos.
- **Invariantes:** cada hit identifica namespace/key y revisión; revisión inaccesible no se sustituye silenciosamente por latest; solo lectura no puede mutar por ningún método. Identidad estable del documento no cambia al cambiar revisión.
- **Pasos:** (1) inventariar revisiones/snapshots actuales del SDK; (2) definir alcance por consulta frente a namespace; (3) propagar coordenada por explain y detalle; (4) comprobar revisión en hidratación; (5) tratar conflictos explícitamente en el consumidor.
- **Persistencia/migración:** metadata de respuesta primero, sin migrar payloads. Si se necesita retención durable, requiere spec del motor y compatibilidad de lectores; no inferirla del contrato de `ReadCoordinate`.
- **Rollback/dependencias:** desactivar lectura compuesta y volver a una lectura única actual; no «revertir» datos. Depende de identidad/revisión accesible y autorización del host; independiente de B/C para un diagnóstico read-only.
- **FMEA:** cambio entre ID e hidratación → contenido/score mezclados → detectar mismatch de revisión → repetir toda la lectura o devolver conflicto; pin que admite merge → cambio inesperado → validar todas las mutaciones contra modo read-only; snapshot expirado → error tipado, no fallback a latest.
- **Aceptación:** consulta+detalle conserva revisión o falla de forma visible; actualización concurrente no mezcla; borrado/revocación impide mostrar contenido no autorizado; dos scopes con la misma key no se confunden. Casos de cancelación liberan el handle.
- **Observabilidad/mínimo:** contadores de conflictos y revisiones no retenidas, duración del handle, sin query/texto ni IDs sensibles en labels. Mínimo: lectura compuesta detectablemente coherente; no objetivos: time travel de usuario, branches ni promesa de snapshot global.

### B. Frescura de derivados ligada a la definición y al origen

- **Problema/escenario:** embedding/resumen almacenado sigue presente tras editar el documento o cambiar extractor. Un cache hit por «hay valor» reutiliza una proyección obsoleta.
- **Fuente/unidad mínima:** `table/freshness.rs:4-21,51-110` [LL], firmas de entrada + definición + estado dirty. No trasladar direcciones físicas de fragmentos Lance a keys Vanta.
- **Delta/seam:** chunks/provenance de `Ingestor::ingest` [V3], merge de ingesta y jobs de `run_pass` [V5][V6], auditoría textual existente [V7]. Evaluar reportar frescura por artefacto en el sistema existente; no un segundo scheduler.
- **Alternativas:** recomputar siempre es simple pero cuesta embeddings/LLM; invalidar por fuente completa evita complejidad de filas; granularidad por chunk solo si su identidad es estable. Adapter externo sirve si es único dueño de la derivación.
- **Interfaz propuesta:** `DerivedStamp { source_key, source_revision, processor_identity, output_digest }`; `inspect_freshness(stamp, current) -> Freshness::{Current,Stale,Unknown}`. `Unknown` no significa fresco ni prueba corrupción.
- **Invariantes:** publicación del derivado ligada al origen que leyó; cambio de modelo/config semántica invalida; metadatos operativos no invalidan sin razón semántica; ningún stamp permite saltar authz; contenido borrado no se regenera desde una caché sin autorización.
- **Pasos:** (1) inventariar stamps/checkpoints actuales; (2) separar identidad semántica y operativa; (3) incluir stamp opcional en metadatos existentes; (4) calcular estado en auditoría; (5) encolar recomputación idempotente; (6) publicar solo si origen sigue en revisión esperada.
- **Persistencia/migración:** registros antiguos quedan `Unknown`, no se etiquetan retrospectivamente `Current`. Reindexación selectiva después de inventario; retener artefacto anterior como no vigente hasta reemplazo, según política existente.
- **Rollback/dependencias:** desactivar recomputación automática y conservar lectura explícita del estado; no restaurar un derivado viejo como vigente. Depende de revisión de origen, identidad de procesador y publicación condicionada de C si se automatiza escritura; auditoría read-only no está bloqueada por C.
- **FMEA:** origen cambia durante job → publicación stale → comparar revisión al commit y reencolar; modelo usa alias mutable → falso hit → registrar identidad resuelta si existe; política de dirty incompleta → falso limpio → detección por auditoría cruzada y recomputación conservadora.
- **Aceptación:** cambio de texto/config invalida; no-op semántico no recalcula por metadata de ejecución; job interrumpido no publica stamp exitoso; borrado concurrente impide resurrección; fallo del procesador conserva error y origen.
- **Observabilidad/mínimo:** motivos de invalidación, conteos current/stale/unknown, coste/tiempo por etapa sin texto. Mínimo por fuente o chunk ya existente; fuera: columnas computadas SQL, sidecars físicos de Lance y nueva jerarquía cognitiva.

### C. Recibo de reemplazo y reconciliación tras resultado desconocido

- **Problema/escenario:** agente reingiere una revisión después de timeout; necesita saber si reemplazó, falló antes de escribir o quedó parcial. Repetir un append no equivale a idempotencia.
- **Fuente/unidad mínima:** `MergeResult`/publicación de Dataset en `table/merge.rs:24-42,420-444` [LF], `UpdateResult` [LG] y `DeleteResult` [LH]. Lo adoptable es recibo con revisión y resultado, **no** source-porting del merge.
- **Delta/seam real:** `l1_writer::write_l1_batch` borra targets antes de `put_batch` en la base revalidada [V8]; `ingest::merge` conserva decisiones y errores por entrada [V6]. Extender contrato propio de reemplazo, no ejecutar semántica documental SQL dentro de dedup LLM.
- **Alternativas:** reparación auditada después de fallo; preparar artefacto nuevo y cambiar referencia; transacción del motor si acredita atomicidad requerida. No elegir una alternativa por analogía con el commit de Lance.
- **Interfaz propuesta:** `ReplaceIntent { operation_id, scope, expected_revision, replacement_keys }`; `ReplaceReceipt { operation_id, committed_revision: Option<Revision>, outcome: Applied|Rejected|Partial|Unknown, counts }`; errores con fase y posibilidad de retry. Los tipos y variantes son diseño, no API actual.
- **Invariantes:** misma operación no aplica dos veces efectos de negocio; receipt de éxito solo tras frontera durable acreditada; conteos diferencian insert/update/delete de operaciones internas; `Unknown` no se convierte en «cero escrituras».
- **Pasos:** (1) inventariar límites actuales de `put_batch` y delete; (2) definir frontera de reemplazo sin borrar evidencia antes de tener reemplazo durable; (3) persistir intento/revisión donde ya vive estado de ingesta; (4) publicar receipt después del commit; (5) reconciliar por identidad en retry; (6) reparar proyecciones postcommit mediante jobs existentes.
- **Persistencia/migración:** esquema versionado y opcional para intentos nuevos; registros anteriores siguen legibles. Si requiere modificar atomicidad del motor, es trabajo separado y condición de esa ruta, no migración aprobada aquí.
- **Rollback/dependencias:** detener nuevos reemplazos y mantener lectura/reconciliación de intentos pendientes; compensación explícita por revisión, jamás reejecutar delete a ciegas. Depende de operación estable, revisión/CAS o alternativa acreditada; no depende de ANN, B ni UI.
- **FMEA:** fallo tras delete y antes de put → pérdida parcial → detectar fase/targets y comparación canónico↔receipt → evitar delete-first o compensar con evidencia; respuesta perdida después de commit → retry duplicado → operation_id durable; crash antes de receipt → resultado desconocido → reconciliar datos, no marcar éxito por timeout.
- **Aceptación:** fallo antes/durante/después de publicación produce estados distinguibles; retry no duplica ni borra otra revisión; target concurrentemente actualizado genera conflicto; embeddings fallidos no eliminan la fuente; replay de logs no produce efectos externos dobles.
- **Observabilidad/mínimo:** conteos por outcome/fase, pendientes sin resolver, duración de reconciliación y errores sin payload. Mínimo: recibo honesto del contrato presente; no afirmar atomicidad hasta acreditarla, no mezclar checkpoint de ejecución con memoria factual.

### D. Proyección por etapas y consumo acotado del retrieval

- **Problema/escenario:** inspector solicita pocos hits, pero recupera todos los blobs/vectores o materializa ambas ramas sin comunicarlo. El formato stream no constituye backpressure end-to-end.
- **Fuente/unidad mínima:** `run_hybrid` drena y concatena candidatos [LI]; scanner proyecta columnas y aplica límite por batch [LJ]. La unidad es un plan de selección y posterior hidratación, no un motor Arrow nuevo.
- **Delta/seam:** `hybrid_search` ya separa presupuesto de candidatos de top-k [V1]; `explain_memory_search` ya comunica ranking [V2]. Evaluar proyección acotada en SDK/bridge y reutilizar el resultado para abrir detalle en Studio [S1].
- **Alternativas:** conservar objetos completos para pocos hits; introducir solo proyección de respuesta en adapter; proyección física en motor si medición justifica complejidad. Recortar JSON después de cargar todo no equivale a reducir IO.
- **Interfaz propuesta:** `SearchProjection { fields, include_vector, include_payload }` y `ProjectedHit { identity, revision, score, selected_fields }`; `hydrate_at` de A solo si hay segundo paso. Errores: campo no permitido, revisión cambiada, límite/cancelación, IO.
- **Invariantes:** campos omitidos no se representan como valor vacío existente; orden/score/identidad no cambian por proyección; autorización se aplica antes de devolver incluso metadata; memoria intermedia no se supone acotada por tamaño del batch final.
- **Pasos:** (1) medir dónde se hidrata hoy, sin ejecutar durante esta investigación; (2) inventariar selección ya existente; (3) ampliar opciones/serialización compatibles; (4) separar candidatos y detalle; (5) propagar cancelación al productor y liberar buffers; (6) informar materialización cuando ocurra.
- **Migración/rollback:** opción ausente conserva comportamiento; sin cambio de datos ni reindexación para proyección de respuesta. Volver a lectura única si hidratación coherente no está soportada; nunca devolver campos prohibidos para «hacer fallback».
- **Dependencias:** A para multi-request coherente, no para proyección en una sola respuesta; soporte de transporte para cancelar y semántica de campos opcionales. No requiere B/C.
- **FMEA:** campo omitido se interpreta como borrado → re-put pierde datos → distinguir no solicitado de ausente; stream cancelado retiene buffers → detectar tareas vivas/RSS → liberar al cancelar; hidratación N+1 → latencia/coste → batch por identidad y revisión.
- **Aceptación/observabilidad:** misma consulta con distinta proyección conserva ranking; vector no solicitado no se serializa; cancelación cierra recursos; permisos/revisión se revalidan al detalle. Medir bytes/filas/batches por fase y cancelaciones; no prometer ahorro hasta medición. Mínimo: opción explícita, sin paginación física nueva ni librería Arrow en core.

### E. Explicación del plan efectivo y de la procedencia del score

- **Problema/escenario:** un operador interpreta distancia aproximada como exacta o atribuye al motor un reranking que solo aplicó el cliente.
- **Fuente/unidad mínima:** scanner y opciones efectivas [LJ], semántica de refinamiento [LO], unión RRF por row ID [LK]. Extraer diferencia plan solicitado/ejecutado y score de etapa, no otra fórmula de ranking.
- **Delta/seam:** `SearchExplanation` y `HybridFusionReport` existen [V2]; Studio ya recibe `ExplanationHit` [S1]. Ampliar solamente datos que una ruta efectiva no comunique, conservando los tipos existentes.
- **Alternativas:** documentación de defaults basta para algunos callers; reporte por consulta para rutas variables; rechazar un segundo explain que ejecute otra consulta y se presente como traza de la primera.
- **Interfaz propuesta:** extensión opcional `ExecutionProvenance { route, revision, stages, score_kind, degraded_reason }` del reporte actual. Errores por «explicación no disponible» distintos de búsqueda fallida.
- **Invariantes:** score conserva semántica/unidad, no se normaliza para aparentar probabilidad; traza y hits pertenecen a una ejecución; degradación no se oculta; no generar explicaciones con LLM.
- **Pasos:** (1) inventariar reporte actual por backend; (2) obtener opciones resueltas donde ya se elige ruta; (3) adjuntar procedencia a esa ejecución; (4) serializar opcionalmente; (5) mostrar no disponible en Studio si transporte no soporta detalle.
- **Persistencia/migración/rollback:** metadatos diagnósticos opt-in, no payload durable ni reindexación; desconocidos tolerados por lectores anteriores. Desactivar reporte ampliado no cambia ranking. Depende de ruta efectiva y A para una revisión común si se compone con detalle.
- **FMEA:** explain recalcula con otra revisión → relato falso → correlación por ejecución; scores incompatibles sumados sin etiqueta → diagnóstico engañoso → tipos de etapa y conservación de originales; alta cardinalidad en métricas → coste/fuga → labels de categoría, detalles sujetos a permisos.
- **Aceptación:** vector-only/keyword/hybrid/sparse y top-k cero describen su ruta real; fallo de etapa no aparece como cero hits exitoso; no hay cambios de orden al activar reporte. Observabilidad: disponibilidad de explain por backend, fallos de reporte y rutas, sin texto. No objetivo: nuevos pesos, nprobes ni thresholds Vanta.

### F. Disponibilidad del índice referida a una revisión objetivo

- **Caso:** operador acaba de importar un manual y abre el inspector; «dato
  guardado», «job aceptado» e «índice alcanzó esa escritura» son preguntas
  diferentes. Mostrar solo que el índice existe induce decisiones erróneas.
- **Evidencia/unidad:** [LT] y `wait_for_index` [LU:29-68]; distinguir
  existencia, cobertura y timeout sin copiar un bucle de polling global.
- **Delta/seam:** `cmd_rebuild_index` en [V7:14-68] ya informa vectores,
  tombstones y duración; `explain_memory_search` [V2] ya informa ruta.
  Propuesta condicionada: comunicar cobertura respecto a la escritura que
  interesa, no añadir índices ni otro job manager.
- **Contrato Rust propuesto:** `IndexObservation { scope, index_identity,
  target_revision, observed_revision, state }`; `state` distingue pendiente,
  cubierto, no acreditable y error. `observe_index(target)` es read-only.
  Si el backend no expone una revisión comparable debe decirlo, no inferirla
  del conteo de filas. No usar versión L1 como offset físico del índice.
- **Pasos:** inventariar receipts reales; extender reporte existente con
  campos opcionales; definir comparación por backend; propagar target al
  inspector; separar espera cancelable de creación/rebuild.
- **Commit/concurrencia:** target se congela antes de esperar; nuevas
  escrituras no deben desplazarlo silenciosamente. Sin watermark verificable,
  informar observación puntual, no garantía causal. Timeout conserva outcome
  y permite volver a consultar sin reejecutar ingesta.
- **Identidad de ejecución:** [LAA] y #4402 justifican asociar la observación
  al receipt/job de reconstrucción cuando exista. Índice anterior con el
  mismo nombre no satisface la nueva intención; `job_done` tampoco acredita
  por sí solo la cobertura de escrituras posteriores. Extender el reporte
  existente con identidad de operación/generación, sin otra cola ni waiter.
- **Persistencia/deps:** primera entrega solo diagnóstico, sin migración ni
  retención de snapshots; depende de revisión observable y permisos de
  mantenimiento. Reusa scheduler [V5] si hace falta espera de tarea.
- **Alternativas/rollback:** conservar reporte puntual actual si no hay
  delta; lectura fallback explícita por backend frente a reconstrucción
  indiscriminada. Desactivar la espera no borra datos ni cancela commits.
- **FMEA:** polling «latest» con tráfico continuo → inanición → target fijo;
  índice recreado con mismo nombre → cobertura falsa → identidad de
  generación; timeout→retry de importación → duplicados → separar comandos.
- **Aceptación manual futura:** índices ausente/parcial/listo y generación
  reemplazada se distinguen; nuevas escrituras no cambian target; cancelar
  espera libera recursos sin prometer rollback; sin watermark no aparece
  «cubierto». Solo contadores de estados, sin texto/vectores.
- **Caso adverso adicional:** reemplazar índice homónimo mientras el antiguo
  sigue listo; observar job failed/cancelled y servidor sin job ID. No
  transformar un error de construcción en timeout genérico ni un timeout
  local en cancelación remota confirmada.
- **Mínimo/no objetivos:** reporte causal o explícitamente no acreditable;
  no SLO inventado, segundo scheduler, ni promesa de ANN más rápido.

### G. Intención de reparación parcial con finalización cercada

- **Caso:** un job recalcula embeddings por partes; mientras tanto cambia la
  fuente o corre otro reparador. El worker antiguo no puede marcar limpio el
  artefacto nuevo ni los chunks que nunca procesó.
- **Evidencia:** `begin_row_repair:1451-1564` [LV] y
  `finish_row_repair:1633-1724` [LW]. Unidad: token de intención y conjunto
  seleccionado, distinto del stamp de lectura B o del recibo general C.
- **Delta/seam:** scheduler `run_pass` [V5], ingesta [V6], publicación L1 [V8]
  y checkpoint [V9]. Existen trabajos y progreso; no está acreditado en esta
  muestra que su completion cercado identifique cada derivado seleccionado.
- **Contrato propuesto:** `RepairIntent { scope, source_revision,
  processor_identity, intent_id, selected_artifacts }`;
  `finish_repair(intent, applied, skipped) -> Result<RepairReceipt,
  RepairConflict>`. Los conjuntos son disjuntos y subconjuntos de selected;
  skipped queda pendiente o fallido explícito, nunca éxito sintético.
- **Pasos:** primero auditoría B; después registrar intención en la
  persistencia existente; calcular fuera de locks; verificar revisión al
  publicar cada resultado; finalizar solo evidencias publicadas; reconciliar
  después de interrupción mediante IDs estables.
- **Commit/ownership:** un solo dueño o mecanismo condicional por artefacto;
  el UUID evita ABA de intenciones iguales, pero no sustituye atomicidad de
  comparación+publicación. No implementar CAS con RMW de [V9].
  Si [V8] permite fallo parcial, receipt conserva aplicado/desconocido.
- **Migración/deps:** campos opcionales; legado sin token se audita como
  desconocido. Depende de B y garantía C solamente para publicación activa;
  no requiere trasladar sidecars, Roaring físico o formato Lance.
- **Alternativas:** recomputar toda la fuente con único escritor es más
  simple; usarla hasta justificar reparación parcial. Adapter con ownership
  único puede resolver el caso sin extender el motor. Rollback: detener
  publicación y conservar dirty; no borrar el origen ni declarar clean.
- **FMEA:** worker tardío → limpia revisión incorrecta → mismatch de token;
  subset forjado → oculta trabajo incompleto → validar pertenencia;
  crash tras escribir → retry duplicado → reconciliar por artefacto/revisión;
  revocación/borrado → resurrección → revalidar permisos y vigencia al publicar.
- **Aceptación manual futura:** dos workers, fuente cambiada, token obsoleto,
  cancelación, resultado saltado y duplicado; ninguno limpia trabajo ajeno.
  Fallo de stamp no oculta dato committed (traza de refresh en §1); señal de reparación
  pendiente sin contenido sensible. No se ejecutaron estos casos.
- **No objetivos:** transacción distribuida, traductor de expresiones,
  ejecución de funciones ajenas ni nuevo motor de artefactos.

## 3. Incidencias oficiales y seguridad

### 3.1 Dossier leído: #4844, mutación de handle version-pinned

**Fuente oficial:** [issue #4844](https://github.com/lancedb/lancedb/issues/4844), título «Local: merge_insert on a version-pinned handle writes to the branch's latest (skips ensure_mutable)»; **abierto** observado 2026-11-07, creado 2026-11-05. Se leyó el cuerpo mediante [API oficial](https://api.github.com/repos/lancedb/lancedb/issues/4844) incluido en la respuesta de búsqueda; [comentarios](https://api.github.com/repos/lancedb/lancedb/issues/4844/comments) devuelve lista vacía. No se acredita PR de corrección.

- **Síntoma reportado:** en 0.42.0b3 local Python sync/async y TS beta.3, merge sobre revisión fijada crea nueva versión en latest; add en el mismo handle se rechaza. El reporte diferencia remoto, que sí rechaza.
- **Mecanismo acreditado por lectura local:** `NativeTable::merge_insert`, `table.rs:3997-4010` [LR], comprueba vista y columnas calculadas, pero no llama `dataset.ensure_mutable`. `execute_merge_insert` estándar [LF] tampoco lo llama en el tramo leído; add/update/delete sí [LE][LG][LH]. La documentación de checkout exige read-only [LC].
- **Límite:** se confirma la asimetría estática, no se ejecutó el repro ni se inspeccionó el rebase interno de Lance. No inferir desde ello que cualquier merge o modo RTE escriba siempre latest; el reporte y la muestra delimitan caminos.
- **Aplicabilidad a Vanta:** lección de enforcement transversal: no basta poner read-only en una UI o endpoint si una mutación alternativa lo omite. No se atribuye este bug a Vanta; sus handles y escrituras son distintos.
- **Validación discriminante pendiente:** misma tabla/revisión, intentar cada mutación por SDK y transporte, contrastar versión antes/después y latest, con control de handle no fijado. Rechazo debe ocurrir antes del primer efecto persistente. Solo en instancia desechable y tras autorización.

### 3.2 Dossier leído: #4397, mantenimiento remoto y paridad de SDK

[Issue #4397](https://github.com/lancedb/lancedb/issues/4397), «Remote
optimize/compact_files/cleanup_old_versions behave inconsistently across
SDKs»: **open**, creado/actualizado 2026-10-03T01:40:48Z según la respuesta
[API de búsqueda](https://api.github.com/search/issues?q=repo:lancedb/lancedb+is:issue+wait_for_index&per_page=2)
efectivamente leída el 2026-11-07. Se leyó su cuerpo completo, no todos los
resultados ni una implementación del servidor.

- **Alcance reportado:** Python sync/async, TS 0.40.0 beta.12; servidor
  `release-2026-10-02.e25b0a6437`, Linux aarch64. Reporta sync remoto con
  warnings/no-op frente a `NotSupported` async/TS, y filas nuevas no
  incorporadas al índice vectorial remoto que hacen expirar la espera.
- **Mecanismo reportado:** política de mantenimiento del servidor distinta
  del camino local; el waiter exige cero filas no indexadas. El supuesto
  threshold de mantenimiento procede del reporte, no de código servidor leído.
- **Contraste actual acotado:** [LAB:3688-3743] ya admite compactación remota
  vía SQL job y espera su finalización; rechaza acciones de pruning/index y
  opciones personalizadas no soportadas. Devuelve métricas de compactación
  **placeholder**, no medición real. [LAC:1082-1105] ya delega `optimize`;
  `cleanup_old_versions`/`compact_files` conservan warning/no-op
  (`1057-1080`). No afirmar «todo optimize sigue siendo no-op» ni «issue
  resuelto por completo». La docstring Python dice que no espera el job,
  mientras el tramo Rust sí llama `.wait()`: discrepancia documental visible,
  pendiente de trazar la acción exacta elegida por el binding async.
- **Lección Vanta:** capacidad no soportada, comando aceptado, efecto
  committed y estadísticas reales necesitan salidas diferenciadas. No tratar
  ceros placeholder como «ningún trabajo realizado». Se aplica a C/E/F,
  no prueba un defecto Vanta ni obliga a importar mantenimiento SQL.
- **Aceptación futura:** contrastar capacidades y resultados del mismo
  comando por SDK/backend; distinguir no-op advertido, NotSupported,
  submission y completion, sin exigir paridad de algoritmos físicos.

### 3.3 Dossier leído: #4402 y corrección cliente acreditada

[Issue #4402](https://github.com/lancedb/lancedb/issues/4402), «Remote
create_index hides background build failures; wait_timeout waits by name»:
página pública leída completa el 2026-11-07, muestra **Open**, abierto el
2026-10-03. No se deduce hora exacta de esa página.

- **Reporte histórico:** mismas versiones beta.12/servidor de §3.2;
  configuración inválida falla en background; reemplazar índice homónimo
  retorna pronto porque el índice viejo ya tiene cero filas pendientes.
  El cliente consulta por nombre, no por ejecución; los errores de build
  pueden quedar solo en logs de servidor.
- **Fix efectivamente confirmado:** la página de
  [PR #4428](https://github.com/lancedb/lancedb/pull/4428), también leída,
  acredita **Merged**, 2026-10-22, commit abreviado `4bdc8c3`. No se infiere
  merge del comentario de un participante. [LAA:781-802] en el SHA congelado
  confirma espera por job ID, límite de timeout y fallback por nombre.
- **Límite de solución:** PR declara corregir espera cliente, no validación
  síncrona servidor; mantiene fire-and-forget sin timeout y fallback para
  servidores sin ID. El issue continúa abierto en la fuente leída. Los tests
  mocked-HTTP y su éxito son afirmaciones del autor de PR, no ejecuciones
  propias ni calificación live Cloud/Python/Node.
- **Aplicabilidad:** F debe identificar la operación observada; E no puede
  presentar el índice anterior como plan del reemplazo; C conserva outcome
  desconocido ante timeout. Un job remoto cancelado y un future local
  abandonado no equivalen.
- **Validación discriminante futura:** índice anterior listo, nuevo build
  pendiente/fallido/cancelado; respuesta con/sin job ID; observar que solo el
  build solicitado satisface completion cuando ese ID está disponible.

### 3.4 Fronteras de seguridad y fallos de acceso

La fuente de datos/metadata y los textos a embeber son no confiables. Un predicado textual no concede permisos sobre namespace; el scope debe provenir del principal autenticado. La existencia de `remote`/HTTP/credenciales no demuestra enforcement del servicio Cloud: esa implementación no fue auditada.

Los wrappers contienen rutas de IO, futures y cache de Dataset: expiración de caché no prueba borrado de versiones; un pin puede retener una revisión accesible. Un índice de retrieval puede devolver texto hostil: solo contenido, nunca autorización para ejecutar tools, cambiar rutas o borrar datos. Las propuestas de observabilidad evitan registrar texto, vectores, tokens o secretos por defecto.

**Registro de herramientas, 2026-11-07:**

| Acceso | Resultado y efecto sobre evidencia |
|---|---|
| `websearch.search`, DuckDuckGo: `site:github.com/lancedb/lancedb/issues merge_insert delete` | Cero resultados, sin error declarado; no prueba ausencia de incidencias |
| `github-mcp-server` `list_tools` | Catálogo disponible; el responsable principal había comunicado éxito previo de `get_me` como `ness-e`; eso no garantiza este acceso |
| `github-mcp-server.search_issues`, query `repo:lancedb/lancedb is:issue merge_insert` | «Failed to connect to MCP server»; no se usó como fuente |
| `metasearch.compare_engines`, DuckDuckGo/Bing: `LanceDB merge_insert version pinned issue 4844` | Ambos sin resultados; no certificación multi-engine |
| `websearch.fetchWebContent`, búsqueda API `search/issues?q=repo:lancedb/lancedb+is:issue+merge_insert&per_page=3` | Lectura positiva de #4844; respuesta global truncada: no se declara leída toda la muestra |
| API de comentarios de #4844 | Lista vacía leída; no PR atribuida por conjetura |

Ampliación de acceso: la consulta API `wait_for_index&per_page=2` permitió
leer #4397; GET individual de #4402 devolvió **API rate limit exceeded**.
Fallback positivo `websearch.fetchWebContent` sobre las páginas públicas de
#4402 y PR #4428, sin credenciales. `metasearch.compare_engines` para
`LanceDB 4397 4402 wait_for_index` devolvió cero resultados en DuckDuckGo/Bing:
no se presenta como corroboración multibuscador. Los cuerpos/estados citados
proceden de GitHub, no de snippets ni de esos resultados vacíos.

No se inició servicio ni frontend. Las tres incidencias leídas no constituyen
auditoría de todas las incidencias de persistencia, seguridad o Cloud.

## 4. Contratos previos de VantaDB

Se releyeron seams en `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no solo la narrativa histórica de OpenViking.

1. **Para C y publicación automática de B:** `l1_writer.rs:430-446` [V8] conserva delete-before-put y comentario de fallo parcial. El batch mejora agrupación, no acredita reemplazo indivisible. Resolver frontera propia de reemplazo o limitar receipt a estado parcial/desconocido; no bloquear D/E read-only por este motivo.
2. **Para contabilidad durable compartida:** `CheckpointManager::mutate`, `checkpoint.rs:111-116` [V9], hace read→modify→write; la key sigue global. La carrera es hipótesis, no fallo reproducido. No usar ese patrón como CAS para publicar stamps sin evaluar ownership/concurrencia.
3. **Para A:** inventariar revisión/snapshot ya accesible en SDK y backend concreto. No proponer «añadir versiones» porque el informe de Lance las tenga.
4. **Para D/E:** Vanta ya tiene presupuesto de candidatos y explain; conservar sparse/provenance al proyectar o hacer re-put. No cambiar por defecto recall legacy al core.
5. **Jobs:** scheduler `run_pass` [V5] ya despacha/reclama tareas; estado de cola/leases es process-local según ese contrato. No atribuirle durabilidad de cola distribuida.
6. **Autorización y contenido:** cada adapter y lector de detalle conserva la frontera existente. Ni un LLM ni una revisión histórica pueden autorizar acceso.

## 5. Rechazos, alternativas y costes

| No trasladar | Alternativa sostenible / coste oculto |
|---|---|
| Lance/Arrow/DataFusion como almacenamiento paralelo de memoria | Extender estructuras Vanta; añadir ese stack implica formato, recursos, recuperación y mantenimiento duplicados |
| Branches/versiones por cada sesión de agente | Separar identidad/revisión de fuente y checkpoint; retención de versiones genera coste de GC y riesgo de privacidad |
| RRF nuevo o pesos copiados | Vanta ya tiene fusión y perfiles; justificar solo nueva información de procedencia |
| `try_collect` de ambas ramas como promesa de streaming acotado | Declarar materialización; evaluar proyección y consumo por etapas |
| «Timeout = no se escribió» | Outcome desconocido y reconciliación por operación/revisión |
| Row ID físico como ID de negocio | Namespace/key/fuente estables y mapeo explícito |
| Portar funciones por Apache sin revisión | Ideas sí; fragmentos derivados exigen avisos/licencias y revisión de dependencia concreta |
| Extrapolar Cloud desde OSS | Separar capacidades/errores/latencia de red y feature `remote` |

## 6. Método y límites específicos

Se aplican [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md); este dossier responde sus ejes sin delegar su evidencia.

- Una dependencia Git beta fijada no es el mismo artefacto que un wheel publicado. No extrapolar el SHA a todas las versiones.
- Versionado de Dataset no demuestra una transacción conjunta con embeddings externos, objetos de aplicación o ledger de agente.
- Código de snapshot y mutabilidad expresa intención y mecanismo parcial; #4844 demuestra por qué hay que revisar todos los caminos.
- Las firmas por fragmento son específicas de Lance; en Vanta el origen puede ser una key y revisión lógica, no un fragmento físico.
- Los runners de reranking y streaming miden unidades distintas. Ninguno certifica memoria multisesión, calidad de respuesta o éxito de agente.
- No hay cifras propias ni promesas de reducción de coste/latencia/RSS. Licencias de datasets/modelos/proveedores siguen pendientes.

## 7. Benchmarks inspeccionados

### 7.1 `bench_typesafe.py`: candidatos congelados y reranker externo

**Fuente** [LM], runner leído `python/benchmarks/bench_typesafe.py:1-245`; entry point argparse→`run`→`score_row`→`summarize`. No hace búsqueda real en la DB durante la fase de scoring: recibe candidatos preconstruidos.

| Eje | Lo que calcula/configura el código |
|---|---|
| Esquema/unidad | Lista JSON por query: `query`, mapa `documents` ID→texto, listas ordenadas `vector`/`fts`, `answer`. Una fila es pregunta con candidatos, no sesión |
| Preparación | Hash SHA256 de bytes del archivo candidato; no genera dataset/split/embeddings en este runner. Procedencia de pools y separación dev/holdout no acreditadas |
| Pipeline | Construye tabla Arrow de textos, posición y distancia cero; llama `TypeSafeReranker.rerank_vector`. Es reranking de pools; no ANN, answerer ni juez independiente |
| Métrica | Para métodos vector/fts/hybrid y k=5/10: toma primeros `4*k` IDs de cada rama, une en hybrid sin duplicados, ordena por score, suma si **algún texto** del top-k iguala exactamente `answer`; porcentaje `100*hits/len(rows)` [LM:129-164] |
| Denominador | Todas las filas de entrada en una ejecución que termina. Archivo vacío se rechaza; no define abstenciones ni gold múltiple. Coincidencia de texto no acredita identidad de documento |
| Defaults existentes | Modelo `jev-1.13.0`, concurrencia 32; compara batch size 1 y 40 y alterna cuál va primero. No se proponen estos defaults para Vanta |
| Caché | Identidad incluye hash de candidatos, modelo, concurrencia, batch, formato, instrucciones/criterios, SDK, versión lancedb y hash del módulo. Registro por query valida IDs, scores, conteo esperado y latencia finita |
| Errores | Discrepancia de requests/IDs/score genera `ValueError`; llamadas no manejadas abortan. No hay informe completo de intentadas/fallidas si aborta |
| Latencia/coste | `perf_counter` alrededor de rerank; mediana/p95 sobre `record["seconds"]`, incluidos registros recuperados de caché con **latencia histórica**, no latencia actual de cache hit |
| Request count | Cuenta `system_one` del SDK, excluye retries internos según etiqueta del resultado; no tokens, dinero ni coste de producir candidatos |
| Efectos/deps | TypeSafe SDK/credencial/red potencialmente pagada, PyArrow/numpy/lancedb; crea directorios y JSON de caché/output mediante temporal+replace. No ejecutado |

**Límites que importan:** el runner contiene instrucciones/criterios upstream: se describe su existencia sin crear ni cambiar prompts. Si el texto gold no está en pools, el reranker no puede recuperarlo; no es comparación de motores. El modelo resuelto se guarda; un alias mutable sigue requiriendo pin/registro del servicio.

Relectura de denominadores [LM:80-165,168-245]: las preguntas se recorren
secuencialmente; `max_concurrency` pertenece al reranker, no significa 32
preguntas simultáneas. `queries=len(rows)`, pero `candidate_pairs` cuenta
todos los documentos de cada fila; requests esperados excluyen documentos
`None` y usan `ceil(non_null/batch_size)`. La mediana/p95 resume tiempo
de reranking del pool completo por pregunta, no latencia por candidato ni
top-k. No hay warmup explícito del modelo, control de seed/tokenizer ni
medición de tokens. Batch 1/40 cambia formato del request, registrado en
identidad: no aislar el efecto como si solo cambiara transporte.
El docstring identifica candidatos de investigación GooAQ; origen, licencia,
split y generación del archivo externo siguen sin verificarse.

**Mapping Vanta bajo protocolo del responsable:** obtener candidatos del retrieval real actual [V1], preservar IDs y origen vector/lexical/sparse, emitir el esquema congelado solo por adapter externo. Verificar unidad documento/chunk y no introducir `answer` en el retrieval. Mantener el evaluador upstream intacto y registrar por separado coste de generación de pools, reranking y errores. No usar hit de texto como `recall_all` de LongMemEval ni como QA accuracy. Una eventual medición read-only de E no exige implementar C.

### 7.2 `bench_streaming_dataloader.py`: throughput de lectura, no retrieval

**Fuente** [LN], `python/benchmarks/bench_streaming_dataloader.py:1-135`; `main`→`make_table`→`bench_chunk`→`StreamingDataset`.

- **Datos:** tabla local temporal `bench`, `mode="overwrite"`, `id:int32` y `payload:large_binary` repetido; no documentos semánticos, embeddings ni ground truth. Default: 24 splits, `NUM_ROWS=24*2048`, 4096 bytes de payload, 100 pasos; exige divisibilidad.
- **Muestreo/defaults:** recorre read_batch_size 1/4/16/64/256/1024/4096/16384. `shuffle_seed=42`. Warmup consume 24 filas por tamaño antes de una nueva instancia medida. No acredita cold-cache de OS entre variantes.
- **Métricas/denominadores:** drena hasta `steps*NUM_SPLITS` filas o fin; `rows/s = drained/elapsed`, `ms/step = elapsed/STEPS*1000`. Si drena menos, ms/step conserva denominador solicitado, no recalculado por pasos completos.
- **Qué incluye:** iteración real del dataloader y fetch durante ese intervalo; no tiempo de crear tabla, no embedding, ingesta remota, respuesta ni judge. `elapsed==0` produce infinito en rows/s.
- **Errores:** assert/config y excepciones abortan; no agregación de ensayos fallidos ni percentiles. Comparación de warmups no es distribución de latencia de consultas ANN.
- **Efectos/deps:** PyArrow, lancedb y formato nativo instalados; escribe y elimina directorio temporal. Payloads repetidos tienen representatividad limitada para compresión/cache/IO de un corpus real.

**Mapping Vanta:** adapter de lectura de batches/IDs estables para pista de motor/producto, con revisión común si se usa D; no copiar `StreamingDataset` ni sus knobs como configuración nueva. Preparar datos equivalentes y declarar coste de adaptación fuera del tiempo medido; conservar errores y cardinalidad efectivamente drenada. No hay ejecución ni comando de arranque solicitado en esta fase.

## 8. Matriz de capacidades

| Capacidad de la muestra LanceDB | Estado Vanta revalidado | Fuente y decisión |
|---|---|---|
| Dense + lexical + RRF | **Existe**; también sparse opcional | [V1][V2]; no duplicar |
| Parámetros/explicación de ranking | **Existe**, paridad por backend parcial | [V2][S1]; E condicionado a delta concreto |
| Chunks con provenance/idempotencia por key | **Existe** | [V3]; no proponer chunker básico |
| Presupuesto y compactación | **Existe** en dominio de contexto | [V4]; no es feature de la tabla LanceDB |
| Jobs/scheduler/retry de memoria | **Existe**; cola process-local | [V5]; B se integra ahí |
| Snapshot compuesto expuesto como A | **No acreditado** en las rutas inspeccionadas | Inventariar revisión del SDK antes de una tarea |
| Frescura de derivado con definición+origen | **Parcial/no acreditada equivalencia** | Provenance sí [V3], stamp B no certificado |
| Reemplazo L1 indivisible | **Parcial**: contrato visible de fallo parcial | [V8]; C no puede prometer más |
| Auditoría de índice textual | **Existe** | [V7]; extender, no sustituir |
| Branches de Dataset/SQL column store | **No aplicable** como requisito de agente | Diferente modelo operativo |
| Inspector y actividad | **Existe** en Studio | [S1][S2]; adiciones incrementales |
| Completion de índice vinculada a generación/operación | **No acreditado** en seams leídos; rebuild/auditoría sí existen | [V7][LAA]; F primero inventaría receipts existentes |
| Reparación parcial con intención cercada | **No acreditado**; jobs/progreso sí existen | [V5][V6][V8]; G condicionado a necesidad y ownership, no ausencia global |
| Estadísticas de mantenimiento como efecto medido | **Parcial**, reporte de rebuild existente | [V7][LAB]; no transportar placeholders como medición |

## 9. Negativos sustentados y falsos positivos

**Hecho estático revalidado:** la ruta L1 batch mantiene delete-before-put [V8]. Consecuencia posible: fallo intermedio deja targets eliminados. No se ejecutó inyección de fallo, ni se demuestra desde esto un bug de WAL/crash recovery. La atomicidad del lote y la del reemplazo son contratos distintos.

**Patrón + hipótesis:** RMW del checkpoint global [V9] requiere intercalación determinista para confirmar lost update; no prueba que todos sus callers sean concurrentes. No usarlo como bloqueo universal de A/D/E.

**No acreditado:** snapshot compuesto expuesto y sello de derivado con definición. Falta de evidencia en esta muestra no equivale a ausencia de implementación en otro módulo o commit.

**Falsos positivos descartados:** «falta híbrido/RRF», «no hay chunks», «no hay explain/auditoría/jobs/Studio» contradicen [V1-V7][S1-S2]. Tampoco hay evidencia para «Lance es más durable/rápido» ni «Vanta carece de benchmarks»: se conserva la distinción del protocolo y Paso 23. #4844 no es un bug propio por analogía.

## 10. UI y VantaDB Studio

El producto inspeccionado ofrece SDKs y CLI/documentación; se recorrieron mapa raíz, bindings y rutas de docs. No se localizó **en esa muestra** una app desktop propia implementada. Los productos Cloud externos y material visual de `docs/web/` no acreditan frontend completo ni experiencia runtime; no se atribuye stack a una UI no inspeccionada.

Studio sí se leyó, solo lectura, en `C:\Users\Eros\VantaDB Proyect\develop-desktop`, commit `0ad38c542` (referencia abreviada observada; no se equipara a la base del motor): `desktop/src/vanta.ts:1-90` [S1] define conexión, ingesta, sparse, perfil de búsqueda y explicación; `MemoryLens.tsx:1-83` y `Timeline.tsx:1-70` [S2] muestran memoria persistida/actividad e inspector.

**Flujo incremental propuesto:** buscar → ver ruta/score efectivo E → abrir detalle con coordenada A → inspeccionar stamp B → consultar receipt C si hay modificación. No recrear conexiones, memoria, genlog, escenas ni skills. Para invalidación de caché de UI usar identidad conexión+scope+revisión, no solo la key. Acción destructiva necesita permisos/confirmación/revisión esperada; mostrar pasado no habilita restore. Runtime y diseño visual no evaluados.

## 11. Fuentes, cobertura y cierre

### 11.1 Inventario inmutable de código

Las referencias [LA-LR] fijan **solo los rangos efectivamente usados**, no lectura exhaustiva del archivo. Los paths `table/` y `query.rs` de esta sección están bajo `rust/lancedb/src/`.

| Ref | Archivo/rango | Cobertura |
|---|---|---|
| [LA] | `LICENSE:1-202` | Texto de licencia; no inventario transitivo |
| [LB] | `Cargo.toml:1-110` | Workspace/stack |
| [LC] | `table.rs:940-998` y `2490-2501` | API de versiones/checkout |
| [LD] | `table/dataset.rs:14-145,198-270` | Estado, consistencia, snapshot, mutable |
| [LE] | `table.rs:3821-3928` | Add y recibo |
| [LF] | `table/merge.rs:24-42,337-444` | Dispatch/ejecución/estadísticas; no LSM integral |
| [LG]/[LH] | `table/update.rs:87-130`; `table/delete.rs:12-61` | Actualización/borrado |
| [LI]/[LJ]/[LK] | `query.rs:1517-1695`; `table/query.rs:334-425`; `rerankers/rrf.rs:19-149` | Híbrido/scanner/fusión |
| [LL] | `table/freshness.rs:4-21,47-110` | Diseño/estado de frescura, no refresh completo |
| [LM]/[LN] | Runners completos 1-245 / 1-135 | §7; no ejecución |
| [LO]/[LP]/[LQ]/[LR] | `query.rs:1030-1056,1458-1514`; `table.rs:3172-3177,3778-3796,3997-4010` | Contratos/refresco/restore/merge |

Lectura adicional: [LS] `refresh.rs:1-24,287-467`, [LT]
`table.rs:3931-3943,4104-4115`, [LU] `index/waiter.rs:1-89`, [LV]
`freshness.rs:1451-1564`, [LW] `freshness.rs:1633-1724`. El rango
`1565-1600` se usó para comprobación de conjuntos; no se recorrió el archivo
entero. #4844 se volvió a leer directamente mediante API pública el
2026-11-07: **open**, actualizado 2026-11-05T01:02:03Z, sin cierre ni PR
acreditada. Se conserva el análisis anterior, no una reproducción nueva.

Se añadieron [LAA] `remote/table.rs:772-812`, [LAB]
`remote/table.rs:3676-3743`, [LAC] `remote/table.py:1057-1105` y declaraciones
de licencia [LX–LZ]. Esa muestra de cliente remoto sustituye la antigua
laguna total en ese tramo, no acredita una auditoría Cloud.

**Cobertura no leída:** implementación completa de Lance y object_store, fsync/manifest commit interno, RTE/LSM completo, servicio remoto y auth Cloud, optimización/GC/version pruning end-to-end, totalidad de bindings, UDF/embedding registries, cadena de dependencias/licencias, otras ramas/tags. Los fallos de búsqueda se conservan en §3, no se rellenan con hipótesis.

### 11.2 Dependencias, validaciones y división futura

Relaciones: A habilita hidratación compuesta D; B necesita identidad de origen y usa C para publicar con revisión esperada, pero auditoría B read-only puede adelantarse; E amplía explain sin requerir reemplazo. C exige resolver garantías propias de escritura si se pretende all-or-nothing. Coinciden con familias transversales de identidad/derivados/auditoría/pipeline del [modelo OpenViking](../../Backlog-OpenViking-VantaDB.md), no candidatos de backlog duplicados.

Pendiente manual: reproducción #4844 en versión aislada, fallos por fase de C, concurrencia A/B/checkpoint, comportamiento de cancelación D, consumo de memoria efectivo y equivalencia de datos/metricas de §7. No se ejecutó ninguno. Una futura aprobación se divide en specs de contrato de escritura, frescura de artefactos, lectura/proyección y diagnóstico Studio; no se crean specs ni IDs ahora.

**Cierre de autoría:** arquitectura, escritura/consulta/modificación e
índice/refresh trazados; siete fichas, tres incidencias oficiales, una PR
de merge confirmado y dos runners
descritos sin ejecución. F se puede abordar sin G; G se mantiene condicionado
a B/C. Por superar 300 líneas, una futura conversión en specs separaría
lectura compuesta A/D/E, derivados B/G, escritura C y disponibilidad F;
este informe permanece autónomo. Pendientes: revisión adversarial
independiente del responsable principal, validación runtime autorizada y
dependencias externas señaladas. No certifica rendimiento, seguridad global
ni completitud del ecosistema. No se ejecutaron validadores globales.

**Balance por eje, de autoría y no aceptación:** arquitectura y flujos,
recomendaciones, siete fichas Rust propuestas, prerrequisitos, rechazos,
advertencias, matriz y negativos quedan sustentados al nivel estático
delimitado. Incidencias: tres cuerpos leídos, sin reproducción; solo #4428
tiene merge acreditado. Benchmarks: dos runners leídos, sin dataset externo
ni resultado propio. UI: flujo Studio existente y propuesta incremental,
sin frontend Cloud ni evaluación visual. Las lagunas no se transforman en
ausencias globales ni en tareas aprobadas; no staging, commit o push.

[LA]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/LICENSE#L1-L202
[LB]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/Cargo.toml#L1-L110
[LC]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L940-L998
[LD]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/dataset.rs#L14-L270
[LE]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L3821-L3928
[LF]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/merge.rs#L337-L444
[LG]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/update.rs#L87-L130
[LH]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/delete.rs#L12-L61
[LI]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/query.rs#L1517-L1695
[LJ]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/query.rs#L334-L425
[LK]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/rerankers/rrf.rs#L19-L149
[LL]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/freshness.rs#L4-L110
[LM]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/python/benchmarks/bench_typesafe.py#L1-L245
[LN]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/python/benchmarks/bench_streaming_dataloader.py#L1-L135
[LO]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/query.rs#L1458-L1514
[LP]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L3172-L3177
[LQ]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L3778-L3796
[LR]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L3997-L4010
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/hybrid.rs#L7-L60
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/explain.rs#L9-L90
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L1-L89
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L93-L218
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L1-L118
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L128-L186
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L1-L115
[V8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L390-L456
[V9]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L69-L116
[S1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/src/vanta.ts#L1-L90
[S2]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/src/components/memory/MemoryLens.tsx#L1-L83
[LS]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/refresh.rs#L287-L467
[LT]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table.rs#L3931-L3943
[LU]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/index/waiter.rs#L1-L89
[LV]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/freshness.rs#L1451-L1564
[LW]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/table/freshness.rs#L1633-L1724
[LX]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/python/pyproject.toml#L14-L34
[LY]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/nodejs/package.json#L30-L38
[LZ]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/java/pom.xml#L21-L26
[LAA]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/remote/table.rs#L772-L812
[LAB]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/rust/lancedb/src/remote/table.rs#L3676-L3743
[LAC]: https://github.com/lancedb/lancedb/blob/bff93ff314eee34f625d05bd92b73d95867fe787/python/python/lancedb/remote/table.py#L1057-L1105
