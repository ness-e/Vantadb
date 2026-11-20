---
title: "Informe estático — pgvector/pgvector"
kind: research
status: proposed
description: "Flujos C HNSW/IVFFlat, MVCC, TAP y cinco contratos de adopción incremental contrastados con el snapshot VantaDB"
tags: [research, vector, pgvector]
---

# Informe estático — pgvector/pgvector

> **Dictamen:** estudiar contratos de búsqueda filtrada, dominio vectorial y mantenimiento; **no importar otro motor ni atribuir a pgvector una consistencia eventual que no demuestra su código**. Cinco fichas separan C7, C15, C9, C16 y la lección condicionada C13. Ninguna es implementación aprobada.
>
> **Snapshot upstream P:** clon `C:\Users\Eros\pgvector`, commit **`724588a5504b0e9c462e3126563370e8d50a01eb`**, rama `master`, shallow. Captura congelada del 2026-11-07 en [manifest de adiciones](clone-additions-manifest.json), inclusión expresa por debajo del filtro original de estrellas. `Makefile:1-9` y `vector.control:1-4` declaran extensión `vector` **0.8.7**; no significa que se haya instalado.
>
> **Snapshot comparativo V:** **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**. Las rutas Vanta de este informe se leyeron con `git show V:ruta`; no se extrapola el estado de cambios concurrentes. CodeGraph se usó como navegación, no como sustituto de ese pin. Studio se distingue expresamente como lectura local complementaria, no contenido acreditado de V.
>
> **Licencia:** [LICENSE en P][PG-LIC] = **PostgreSQL License**, con copyrights PostgreSQL Global Development Group y Regents of UC; exige conservar aviso y permiso y contiene exención de garantías. **`NOASSERTION` se preserva en el snapshot JSON: es la licencia no detectada por la API, no “sin licencia”**. PostgreSQL/PGXS, bibliotecas cliente y herramientas de terceros son componentes separados; no se heredan sus licencias por citar el README ni reescribir en Rust elimina una posible derivación.

## Índice

[Arquitectura/modelos/API](#1-arquitectura-modelos-y-api) · [Tres flujos C](#2-tres-flujos-c-y-sus-fronteras) · [Prioridades](#3-recomendaciones-priorizadas) · [Cinco fichas](#4-fichas-de-extracción-y-contratos-rust-propuestos) · [Incidencias](#5-incidencias-públicas-revalidadas) · [Prerrequisitos](#6-prerrequisitos-y-relaciones) · [Rechazos](#7-qué-no-trasladar) · [Advertencias](#8-advertencias-y-confianza) · [Benchmarks/TAP](#9-benchmarks-regresión-y-medición-operativa) · [Matriz](#10-matriz-pgvector-vs-vantadb) · [Negativos](#11-negativos-vanta-sustentados-e-hipótesis) · [UI](#12-ui-extensión-clientes-externos-y-studio) · [Fuentes/cobertura/cierre](#13-fuentes-cobertura-y-cierre-de-autoría)

## 1. Arquitectura, modelos y API

### 1.1 Unidad de producto y stack

pgvector es una **extensión C cargada dentro del backend PostgreSQL**. No tiene servidor HTTP, proceso de embeddings, memoria conversacional ni scheduler propios en las rutas estudiadas. PostgreSQL aporta tablas, transacciones, snapshots, roles/RLS, buffers, WAL, planner y workers. `src/vector.c::_PG_init:59-65` registra los subsistemas; [`hnswhandler:468-528`][PG-HAM]/[`ivfflathandler:390-450`][PG-IAM] conectan callbacks de access method, no RPC. El usuario proporciona vectores mediante SQL o un cliente Postgres; los modelos de embeddings/LLM y sus costes son externos y opcionales.

| Área | Implementación observada y responsabilidad |
|---|---|
| Tipos y kernels | `src/{vector,halfvec,sparsevec,bitvec}.c`, headers y `halfutils`/`bitutils`; entrada/salida, casts, distancias, agregados y despachos de CPU. |
| HNSW | `hnsw.c` configura AM/GUC; `hnswbuild.c` construye; `hnswinsert.c` mantiene enlaces; `hnswscan.c` emite heap TIDs; `hnswutils.c` valida/recorre; `hnswvacuum.c` retira y repara. |
| IVFFlat | `ivfbuild.c` muestrea, calcula centros y asigna listas; `ivfkmeans.c` clustering; `ivfinsert.c`, `ivfscan.c`, `ivfvacuum.c` mantienen páginas/listas. |
| SQL/versionado | `sql/vector.sql` declara tipos, operadores y opclasses; scripts `sql/vector--X--Y.sql` migran catálogos. `vector.control` es metadata de extensión; no esquema por colección Vanta. |
| Build/operación | `Makefile` usa `pg_config`/PGXS, compila un módulo `vector`; `Makefile.win` es otra vía. `-march=native` es default del Makefile salvo excepciones: importa a portabilidad, no demuestra rendimiento. |
| Verificación | `test/sql` + `test/expected` = regresión SQL; **`test/t/*.pl` = TAP**, con clusters efímeros y pgbench en varios casos. `.github/workflows/build.yml` los conecta. No reducir `test/` a SQL. |

### 1.2 Modelos, dimensiones y distancias

El primario es una **fila de usuario**; la clave de negocio pertenece a su tabla. El índice se vincula mediante `ItemPointerData` (bloque/offset del heap), no mediante ID de conversación. Headers [vector][PG-TYPES], [halfvec][PG-HALF], [sparsevec][PG-SPARSE] y [HNSW tuples][PG-HTUP] permiten distinguir:

- `Vector`: varlena, dimensión `int16`, reservado, `float x[]`; máximo del **tipo** 16.000 dimensiones. `HalfVector` tiene estructura homóloga con escalares `half`, máximo de tipo 16.000. F16 es precisión/representación, no sinónimo de un cuantizador aprendido.
- `SparseVector`: dimensión `int32`, `nnz`, índices ordenados base cero y valores f32; máximo de tipo 10⁹ dimensiones y 16.000 no ceros. **Sparsity no es cuantización.** HNSW limita además a **1.000 no ceros**, no a 1.000 dimensiones.
- `bit(n)` es tipo bit PostgreSQL con operadores de distancia de pgvector. Puede contener una firma binaria o datos binarios originales: no asumir que todo `bit` procede de cuantizar un float.
- HNSW `HnswElementTupleData` contiene `level`, `deleted`, `version`, varios heap TIDs, `neighbortid` y valor. `HnswNeighborTupleData` contiene versión y TIDs de vecinos. La versión física de tupla sirve al recorrido/reutilización, **no es la revisión semántica de una memoria**.
- IVFFlat tiene metapágina de dimensiones/listas, centros y páginas de entradas con `IndexTuple.t_tid`; insertar asigna al centro más cercano, no vuelve a entrenar todos los centros.

| Superficie | Capacidad en P; fuente |
|---|---|
| Operadores | `<->` L2; `<#>` producto interno **negado** para orden ascendente; `<=>` distancia coseno; `<+>` L1; bit `<~>` Hamming y `<%>` Jaccard. No comparar estos números como si todos fueran similitud mayor-mejor. |
| HNSW opclasses | vector/halfvec/sparsevec: L2, IP, coseno, L1; bit: Hamming/Jaccard. `sql/vector.sql:427-446,843-866,901-911,1194-1217`. |
| IVF opclasses | vector/halfvec: L2, IP, coseno; bit: Hamming. No acreditar IVF sparse, L1 o Jaccard desde la existencia de una función de distancia. `sql/vector.sql:406-425,819-841,894-899`. |
| Límite ANN | vector 2.000; halfvec 4.000; bit 64.000 dimensiones; HNSW sparse ≤1.000 nnz, hasta la dimensión del tipo. `HnswGetTypeInfo`/supports, `hnswutils.c:1638-1702`, y README de IVF. No confundir aceptación SQL con indexabilidad. |
| Validaciones | `vector.c::CheckDims/CheckExpectedDim/CheckDim/CheckElement:71-125`; halfvec homólogos; `sparsevec.c` valida orden/nnz/índices/valores. HNSW exige dimensión fija y `ef_construction >= 2*m`, rechaza `varbit`: `InitBuildState:680-712`. |

**API real:** `CREATE EXTENSION vector`; columnas `vector(d)`; `INSERT`/`COPY`, `UPDATE`/`DELETE`; `SELECT ... ORDER BY embedding <-> query LIMIT k`; `CREATE INDEX ... USING hnsw|ivfflat (..._ops)`; opciones de índice y GUC `SET [LOCAL]`; `VACUUM`/`REINDEX`. SQL y protocolo Postgres son la interfaz, no un SDK de agente. NULL no se indexa y valores de norma cero se omiten en caminos que requieren normalización (`HnswFormIndexValue:423-443`, `ivfinsert.c:89-106`); una fila visible puede no ser candidata ANN.

Defaults distintos: **HNSW `m=16`, `ef_construction=64`, `ef_search=40`** (`hnsw.h:65-73`); IVF **`lists=100`, `probes=1`** (`ivfflat.h:53-56`). Iterativo está **off** por defecto; HNSW ofrece strict/relaxed, IVF solo relaxed. `hnsw.max_scan_tuples=20000`, `scan_mem_multiplier=1`; `ivfflat.max_probes=32768`, acotado por listas y al menos probes. Son defaults upstream, no recomendaciones de configuración para Vanta.

## 2. Tres flujos C y sus fronteras

### 2.1 Escritura: incremental y construcción masiva no son el mismo camino

**Frontera PostgreSQL:** se crea la versión del heap antes de sus entradas de índice; un lector MVCC no acepta una fila sin confirmar. Los cambios de páginas del AM y el commit de la transacción **no son el mismo evento**. La [documentación de locking PostgreSQL][PG-LOCK] acredita el protocolo externo; este estudio no auditó todo el executor de PostgreSQL.

**HNSW incremental**, [hnswinsert.c][PG-HI]:

1. `hnswinsert:836-859` omite NULL, crea contexto temporal y llama `HnswInsertTuple:817-829`; `HnswFormIndexValue` detoasta, valida y normaliza.
2. `HnswInsertTupleOnDisk` obtiene parámetros/entry point, usa `HNSW_UPDATE_LOCK`, escala a exclusividad cuando puede cambiar el entry point; `HnswFindElementNeighbors` → `UpdateGraphOnDisk` (`:740-808`). Valores duplicados pueden compartir elemento y añadir heap TID (`AddDuplicateOnDisk`, `:649-735`).
3. `AddElementOnDisk:149-413` busca insert page/FSM, bloquea buffers en exclusivo, registra imágenes con `GenericXLogStart/RegisterBuffer`, sobrescribe slots reutilizados o añade tuplas y enlaces. Error de `PageAddItem`/`PageIndexTupleOverwrite` produce `ERROR`, no éxito vacío.
4. **Trazado de “Skip WAL for now”:** `HnswFreeOffset:49-119` lee una página vecina para comprobar espacio; su caller `AddElementOnDisk:243-269` ya abrió estado y registra ese vecino si es distinto. Las escrituras ocurren después, `:370-385`, y el estado llega a **`GenericXLogFinish:397-405`**, antes de liberar buffers `:406-408`. Es evidencia de WAL del camino incremental examinado, **no un hueco de durabilidad**. `GenericXLogFinish` emite WAL/aplica buffers; no equivale a COMMIT/fsync de transacción. Para relaciones unlogged no emite registro, según [Generic WAL][PG-WAL].

**IVF incremental**, [ivfinsert.c][PG-II]: `ivfflatinsert:190-216` crea contexto y omite NULL → `InsertTuple:73-184` normaliza/comprueba dimensiones → `FindInsertPage:20-67` compara **todos los centros** bajo locks compartidos → página de la lista bajo exclusivo → `index_form_tuple` enlaza heap TID → append de página con imagen completa si hace falta → `IvfflatCommitBuffer` y eventual `IvfflatUpdateList`. Un fallo de página es `ERROR`; el centro existente se conserva, por lo que drift de datos exige evaluar rebuild, no asumir entrenamiento incremental.

**Bulk build:** HNSW `BuildIndex` → `InitBuildState` → `BuildGraph`; workers pueden compartir grafo/allocator. `InsertTuple:479-545` usa `flushLock`/`allocatorLock`; al superar `maintenance_work_mem` emite NOTICE, hace `FlushPages` y continúa en disco con `building=true`. El camino omite WAL incremental, pero `BuildIndex:1153-1154` registra el rango de nuevas páginas cuando corresponde: no extrapolar “sin WAL” desde `building`. IVF `BuildIndex:1035-1051` → `ComputeCenters:429-474` (objetivo max(50×listas,10.000), limitado por capacidad estimada), muestreo → `IvfflatKmeans` → páginas de listas → `AssignTuples`/sort/load. NOTICE si muestras < listas. Ownership de memoria/contextos y workers es del build, no del cliente SQL; cancelación usa interrupciones PostgreSQL y cleanup del host. No se ha probado cancelación/crash aquí.

### 2.2 Consulta: candidatos ANN, filtro SQL y visibilidad

**HNSW**, [hnswscan.c][PG-HS]: `hnswbeginscan:136-167` reserva estado/contexto y presupuesto `work_mem * scan_mem_multiplier * 1024 + 256`; `hnswrescan:175-190` resetea visitados, descartados, contador y distancia previa. `hnswgettuple:197-337` exige ORDER BY y snapshot MVCC; otro snapshot dispara `ERROR`. Con `HNSW_SCAN_LOCK` compartido, `GetScanItems:26-60` desciende capas con ef=1 y busca capa cero con `ef_search`. Retorna **heap TID**, no una fila autorizada.

El executor comprueba heap/snapshot y aplica predicados SQL. Si pide más TIDs, `ResumeScanItems:67-91` reutiliza descartados/visitados en lotes `ef_search`; **no es una inserción**. Al alcanzar `max_scan_tuples` o memoria no expande más, pero puede devolver descartados restantes (`:265-272`): es límite aproximado de exploración, no garantía de contar exactamente N filas devueltas. Strict descarta candidatos que romperían monotonía (`:320-325`); relaxed puede mejorar cobertura con desorden entre lotes. Ninguno prueba top-k exacto global.

**IVF**, [ivfscan.c][PG-IS]: `ivfflatbeginscan:256-332` lee listas/dimensiones y limita probes/maxProbes; `GetScanLists:48-118` selecciona listas cercanas con heap; `GetScanItems:124-187` lee sus entradas y ordena `(distancia,TID)` mediante tuplesort con `work_mem`. `ivfflatgettuple:364-416` exige ORDER BY/MVCC; agotado un lote, obtiene las siguientes listas hasta maxProbes. Cada lote está ordenado, no necesariamente la unión: por eso solo relaxed. `ivfflatendscan:423-433` termina sort/temporales y libera contexto.

**Consecuencia para memoria:** el filtro selectivo puede dejar menos de k resultados; no equivale a que no existan recuerdos. README propone índices normales de filtro, índices parciales/particiones y escaneo iterativo según caso. ANN compartido puede sufrir interferencia de recall entre tenants; esa interferencia no demuestra fuga de filas. **Postfilter no autoriza**: permisos/RLS del host y verificación de namespace/ACL en Vanta siguen siendo fronteras independientes.

### 2.3 UPDATE/DELETE y VACUUM: MVCC no es retención física

`UPDATE` crea versiones del heap y mantiene los índices según decisiones del host; no existe aquí una API autónoma “editar memoria in-place”. `DELETE` cambia visibilidad transaccional; **VACUUM no es el commit ni el interruptor ready de la fila nueva**. Una entrada ANN que todavía referencia una versión muerta puede gastar candidatos, espacio y trabajo hasta mantenimiento, sin volver visible esa versión a un snapshot que no deba verla.

**HNSW**, [hnswvacuum.c][PG-HV], `hnswbulkdelete:889-913` ejecuta **cinco** pasos, no solo RemoveHeapTids:

1. `RemoveHeapTids:37-174` consulta el callback del host, conserva TIDs vivos, acumula contadores y conjunto `deleting`; no reañade elementos ya `deleted`. Conserva highest/fallback point.
2. `RepairGraph`/`RepairGraphElement` retiran referencias muertas y reparan entry point/enlaces bajo `HNSW_UPDATE_LOCK`, con escalado de lock (`:351-503`).
3. `RepairDeleting:548-612` espera barreras UPDATE/SCAN y limpia vecinos de elementos a retirar; un scan iterativo reanudado todavía puede guardar candidatos anteriores.
4. `ConfirmRepaired:618-695` comprueba referencias; emite **`ERROR "hnsw graph not repaired"`** si queda una hacia `deleting`. No se afirma que ese error se haya reproducido.
5. `MarkDeleted:701-840` espera otro lote de scan, usa cleanup/exclusive locks, pone `deleted=1`, limpia valor/vecinos, incrementa versión física, finaliza WAL y actualiza insert page/FSM para reutilización. Versión se cicla entre 1 y 15; no usarla como revisión de negocio ni como prueba de borrado criptográfico.

**IVF**, [ivfvacuum.c][PG-IV]: `ivfflatbulkdelete:18-144` recorre listas/entradas, llama callback del host, usa `LockBufferForCleanup` y `PageIndexMultiDelete` sobre offsets muertos, finaliza/aborta estado WAL y actualiza insertPage con espacio. No reentrena centros ni compacta por ello todo el archivo. Ambos `vacuumcleanup` reportan páginas y respetan `analyze_only`; estadísticas físicas no certifican olvido en backups/WAL.

Los scans HNSW comprueban versiones al leer vecinos de candidatos reanudados; barreras y versiones coordinan reutilización, no sustituyen MVCC. `vacuum_delay_point`/interrupciones permiten cooperación con PostgreSQL. Fuente externa [locking][PG-LOCK] explica por qué estos caminos rechazan snapshots no MVCC al no conservar pins durante todo el salto índice→heap. Ninguna de estas garantías se trasplanta automáticamente a Rust.

## 3. Recomendaciones priorizadas

| Orden | Decisión y valor incremental | Condición para aprobar |
|---|---|---|
| 1 — C7 | Declarar estrategia efectiva de filtro, cobertura y fin de exploración; extensión iterativa acotada cuando corresponda. Vanta ya tiene pre/in/postfilter y exact fallback. | No cambiar scoring/RRF ni convertir límite en recall garantizado; conservar ACL fuera del filtro. |
| 2 — C15 | Dominio/precisión, dimensión, operador y capabilities por colección lógica; cuantización queda en su eje propio. | Reusar índices/cuantizadores existentes; admitir “unsupported”, no anunciar F16/bit/IVF-sparse por analogía. |
| 3 — C9 | Exponer fases, recursos y resultados parciales del ciclo incremental/bulk/rebuild. | Envolver rebuild secuencial real; no diseñar un pipeline paralelo nuevo. |
| 4 — C16 | Diagnóstico de borrado lógico, retención ANN, reparación y reclamación física. | Extender auditoría/certificados actuales y Studio; no implementar VACUUM PostgreSQL ni otro WAL. |
| 5 — C13 condicionado | Hacer explícita evidencia de revisión aplicada/durable en proyecciones **Vanta**, si la semántica existente no alcanza. | `WriteReceipt` existe; `MutationReceipt` es **propuesto**. pgvector transaccional no prueba una brecha ready/visible. |

## 4. Fichas de extracción y contratos Rust propuestos

Los nombres siguientes son **diseños**, no símbolos actuales ni IDs nuevos de backlog. Las referencias C7/C15/C9/C16/C13 son relaciones con candidatos transversales existentes. Cada ficha tiene once campos; las validaciones son manuales futuras y no introducen harnesses, prompts, fórmulas o umbrales de evaluación.

### 4.1 C7 — Filtro efectivo y exploración iterativa con cobertura observable

1. **Problema de agente.** Una consulta de memoria pide k hechos del namespace/equipo con un predicado selectivo. Devuelve algunos, pero menos de k; el agente interpreta “no hay más” cuando el ANN solo consumió un presupuesto de candidatos. El caso discriminante es **resultado parcial no vacío**, no solo cero resultados.

2. **Upstream/unidad mínima.** [hnswscan.c en P:26-91,197-337][PG-HS], `GetScanItems`, `ResumeScanItems`, `hnswgettuple`: mantiene visitados/descartados y vuelve a explorar capa cero al pedir el executor más TIDs; termina por candidatos, coste o consumo del caller. [ivfscan.c en P:48-187,256-332,364-416][PG-IS] conserva listas y ordena lotes de probes. No transportar la firma C ni su supuesto de heap: lo extraíble es **continuación acotada + motivo de parada**, no otro ranking.

3. **Delta Vanta comprobado.** [V-vector:67-127,158-223,240-312][V-VEC] ya tiene `exact_scan`, selección `PreFilter/InFilter/PostFilter`, máscara Bloom, ef adaptativo y filtro exacto final. Fallback a exacto ocurre en la rama **`hits.is_empty()`**, no automáticamente cuando `0 < hits.len() < top_k`. No es “Vanta carece de filtros”. [V-index:36-61][V-IDX] devuelve `Vec` sin cursor/visited del backend; exponer continuidad requiere capability explícita. [V-explain:12-90][V-EXP] ya produce explicación/fusión; se amplía, no se crea otro explain.

4. **Alternativas/decisión.** Mantener ejecución actual y solo reportar cobertura es el mínimo elegido. Exact scan prefiltrado ya existe y se conserva cuando el planner lo elige. Iterar ANN es extensión opcional, únicamente si el backend conserva estado verificable; repetir búsquedas con k creciente sin deduplicación/snapshot no es equivalente. Rechazar portar filtro SQL o imponer postfilter por defecto: degradaría caminos Vanta existentes y no aporta autorización.

5. **Contrato Rust propuesto.** `fn search_filtered_report(&self, request: MemorySearchRequest, limits: ScanLimits) -> Result<FilteredSearchReport, FilteredScanError>`. Inputs: namespace/filtros/opciones actuales y límites explícitos por operación, sin cambiar sus defaults en este estudio. Output: `hits`, `effective_filter: ExactPre|BloomInThenExact|ExactPost`, `strategy`, `examined_candidates: Option<u64>`, `returned`, `stop: FilledRequested|CandidateExhausted|WorkLimit|MemoryLimit|Cancelled|BackendNoContinuation`, `coverage: ExactEligibleSet|Approximate|Unknown`, `index_generation: Option<u64>`. `ScanLimits` declara unidad y si el límite es duro o cooperativo; no se etiqueta un contador inexistente como cero.
   Errores propuestos: `InvalidFilter`, `UnsupportedContinuation`, `GenerationChanged`, `BackendFailure`, `InvalidLimits`; agotamiento normal retorna reporte parcial, no error genérico “sin memoria”. Invariantes: todo hit pasa namespace y predicado exacto; dedup por identidad/revisión; `FilledRequested` significa cardinalidad, **no recall 1**. Exactitud solo cuando se agotó un conjunto elegible completo y estable. `Cancelled` no certifica ausencia.

6. **Seam real/ownership.** `src/sdk/search/vector.rs::vector_memory_search` y `exact_scan`; `src/sdk/search/explain.rs::explain_memory_search`; `src/index/mod.rs::VecIndex::search`. El adaptador de reporte vive en esa frontera SDK; continuación, si se añade, debe pertenecer al índice existente y ser opcional por backend. Estado de scan propiedad de la llamada, libera buffers/visitados por RAII; no guardar un cursor con referencias a una generación ya reemplazada. No callbacks de UI bajo guard de índice.

7. **Pasos/persistencia/migración/rollback.** Primero representar la estrategia y los datos realmente disponibles en explain; después distinguir parcial/no match y motivo de parada en salida opt-in; solo después implementar continuación en backends capaces, con el mismo filtro exacto. Reporte efímero: no necesita WAL ni persistir candidatos/query. Si se serializa una respuesta, campos nuevos opcionales/versionados; lectores anteriores conservan `hits`. Desactivar la opción devuelve ejecución legacy; no reindexa ni cambia pesos. Rebuild concurrente invalida token de generación o retorna `GenerationChanged`; no continúa sobre memoria vieja.

8. **Dependencias/relaciones.** C7 se apoya en filtros/planner/budgets actuales; C15 sirve para declarar qué backend puede continuar; C9 invalida generación durante rebuild; C16 visualiza el motivo de parada. C3 solo si se altera compilación del filtro; no bloquear esta mejora por un AST universal ni por C13.

9. **FMEA concreto.** Predicado selectivo → sobreviven pocos de los candidatos ANN → recuerdos omitidos → detección `returned<requested`, estrategia aproximada y stop acotado → reportar parcial y ofrecer exacto/continuación según capacidad. Bloom falso positivo → pasa poda pero no filtro exacto → fila de otro scope si se omite el recheck → detección por contraste de namespace/filtro antes de emitir → mantener `matches_memory_filters`. Rebuild entre lotes → reutilización de estado ajeno → duplicados/lecturas inválidas → generación discordante → abortar continuación. Filtro imposible → exploración sin llenar k → coste sin límite → contador/stop por límite y cancelación cooperativa.

10. **Aceptación/observabilidad futura.** Caso normal: prefilter actual devuelve sus mismos hits y se rotula exacto solo sobre elegibles inspeccionados. Caso parcial no vacío: no se presenta como conjunto exhaustivo. Adversos: filtro imposible, cambio de generación, backend sin continuación, cancelación y error de storage conservan causas diferenciadas. Verificar que namespace/Bloom falso positivo nunca salen como hits y que budgets no cambian por introducir reporte. Observar estrategia, candidatos **si existen**, tiempo y motivo de parada, sin texto, vector, valores del filtro ni IDs sensibles por defecto. No se fija un umbral nuevo de recall o latencia.

11. **Mínimo/no objetivos/abierto.** Mínimo: reporte efectivo con parcialidad y capability, sin prometer continuación inexistente. No objetivos: nuevo ranking, retuning automático, portar GUC/SQL, ACL basada en postfilter ni garantía de completar k cuando hay menos elegibles. Abierto: qué backends pueden conservar cursor y qué contador de trabajo es comparable; debe resolverse por implementación, no inferirse de HNSW nominal.

### 4.2 C15 — Dominio y precisión por colección, independientes de cuantización

1. **Problema de agente.** Una aplicación mezcla embeddings densos y señales sparse; al cambiar modelo/dimensión o convertir a firma binaria, el agente desconoce si la colección acepta ese input, qué distancia usa o si una consulta hizo conversión con pérdida. Tener un campo “quantization” no explica esa compatibilidad.

2. **Upstream/unidad mínima.** [Tipos en P][PG-TYPES], `Vector`/`HalfVector`/`SparseVector`; [hnswutils.c:1606-1702][PG-HSUP], `HnswCheckDim`, `SparsevecCheckValue`, supports de half/bit/sparse; [opclasses SQL en P][PG-SQL]. El índice selecciona capacidades por **tipo + opclass** y rechaza dimensiones/nnz incompatibles. Half, sparse y bit no constituyen tres técnicas equivalentes de cuantización; expresión `binary_quantize` es una operación distinta.

3. **Delta Vanta comprobado.** [V-quant:15-90][V-QUANT] ya ofrece representación binaria/nibbles; [V-maint:604-645][V-MAINT] contiene `run_quantization_maintenance` f32↔SQ8. El SDK ya tiene vector denso y sparse, y `validate_vector_dimension` [V-ser:190-203][V-SER] rechaza dimensión incompatible con el índice. **No proponer corregir de nuevo ese mismatch**. El delta es un contrato consultable por colección lógica, con capacidades/operadores y precisión separadas del codec físico; no se acreditó tal contrato completo en las superficies leídas.

4. **Alternativas/decisión.** Extender metadata/capabilities SDK existentes es preferible a nuevos índices. Mantener validación global de dimensión es correcto para un índice compartido; no inventar que namespace implica índice independiente. F16 nativo, ANN sparse o multi-dimensión por namespace se rechazan del mínimo si el backend no los soporta. Adaptar input mediante cast explícito es opción del caller, nunca conversión silenciosa.

5. **Contrato Rust propuesto.** `fn vector_capabilities(&self, collection: &CollectionRef) -> Result<VectorCapabilities, VectorSpecError>` y `fn declare_vector_spec(&self, collection: &CollectionRef, expected_revision: Option<u64>, spec: VectorSpec) -> Result<VectorSpecRevision, VectorSpecError>`. `CollectionRef { namespace, collection_key }` es **identidad lógica propuesta**, no afirmar un registro de colección ANN actual. `VectorSpec { domain: Dense|Sparse|Binary, scalar_precision: F32|F16|Bit, dims: u32, max_nnz: Option<u32>, metric, index_backend, quantization: ExistingCodecPolicy, schema_version }`. `ExistingCodecPolicy` referencia la política disponible, no introduce codecs.
   Output capabilities: combinaciones aceptadas para ingest/query/index, límites de dimensión/nnz y si existe conversión explícita, continuación/rebuild. Errores: `UnsupportedCombination`, `DimensionMismatch`, `NnzLimit`, `InvalidValue`, `SpecConflict`, `SharedIndexConflict`, `MigrationRequired`. Invariantes: spec se valida **antes de escribir**; dimensión lógica ≠ nnz ≠ bits almacenados; `Sparse+F32` no se llama cuantizado; métricas llevan orientación de score; no declarar F16 o Jaccard soportados por Vanta solo porque existen en pgvector.

6. **Seam real/ownership.** `src/sdk/serialization/mod.rs::validate_vector_dimension`, `src/sdk/api/memory.rs::put_one` (guard/validación antes de `engine.insert`, `:1029-1045,1130`); `src/sdk/search/vector.rs::vector_memory_search`; `src/sdk/search/sparse.rs` como frontera sparse, sin certificar aquí todo su backend. `src/sdk/api/admin.rs::capabilities:138-145` es capability actual de proceso, no por colección. Metadata persistente puede extender `BackendPartition::InternalMetadata` mediante `src/storage/engine/partition.rs::put_to_partition/write_backend_batch`, conservando formatos actuales. `src/vector/quantization.rs` y governor son consumidores de política, no registro de colecciones.

7. **Pasos/persistencia/migración/rollback.** Inventariar capacidades realmente soportadas por cada ruta; añadir respuesta read-only; declarar spec versionada solo para combinaciones actuales; validar put/batch/query de igual modo; persistir revisión de spec atómicamente en el backend actual. Para legado, “inferred/unknown” hasta verificar dimensión/codec del índice, no reescribir datos al abrir. Cambio incompatible requiere colección/índice nuevo **solo si ya hay aislamiento físico disponible**, o mantenimiento offline del índice compartido con C9; rechazar mientras no exista ruta segura. Rollback restaura spec compatible anterior únicamente tras comprobar datos; desactivar declaración no habilita datos que la validación antigua rechaza.

8. **Dependencias/relaciones.** C15 usa el registro de metadata y codecs Vanta existentes. C7 consume capabilities para continuación; C9 ejecuta la migración/rebuild necesario; C16 presenta specs inferidas/desconocidas. No depende de LLM, nuevas librerías SIMD ni entrenamiento de cuantizadores. El mapeo colección lógica↔índice compartido es prerrequisito **de habilitar heterogeneidad**, no de la consulta read-only.

9. **FMEA concreto.** Cambio de embedding con misma dimensión → validación dimensional pasa pero espacio semántico cambia → vecinos sin significado → detectar revisión/identidad de modelo declarada por el caller, no deducirla de floats → exigir migración explícita y no afirmar detección automática. Confundir nnz con dimensión → rechazar sparse válido o aceptar coste inesperado → comprobar ambos límites → validadores separados. Spec persistida antes de rebuild fallido → contrato anuncia datos no preparados → revisión activa no cambia hasta mantenimiento verificado; estado pendiente/unknown. Namespace con dims distintas sobre grafo compartido → conflicto global → rechazar `SharedIndexConflict`, no modificar kernel para truncar.

10. **Aceptación/observabilidad futura.** Declaración compatible se conserva al reabrir y consulta devuelve misma revisión. Normal: dense f32 y sparse conservan sus rutas/score actuales. Adversos: mezcla de métricas/precisiones no soportada, dimensión diferente, nnz excesivo, spec obsoleta y caída durante migración producen error antes de mutación o conservan spec activa anterior. Observar dominio, dimensión, codec/capability y razón de rechazo; no vectores ni nombre de modelo privado por defecto. La prueba debe incluir put/put_batch/query para no crear contratos divergentes.

11. **Mínimo/no objetivos/abierto.** Mínimo: consulta de capabilities y declaración de lo ya soportado, con distinción explícita dominio/precisión/codec. No objetivos: nuevos tipos físicos, IVF sparse, nuevos cuantizadores, particionado multi-dimensional o autoelección de modelo. Abierto: identidad estable de colección y política de compatibilidad del embedding-space; no resolverlo inventando un `CollectionConfig` ya implementado.

### 4.3 C9 — Ciclo incremental/bulk/rebuild con progreso y recursos declarados

1. **Problema de agente.** Importación inicial y mantenimiento bloquean una base de memoria; un único “success” no explica si ANN terminó pero texto/sparse fallaron, qué recursos fueron usados o qué fases quedan pendientes. La aplicación no debe iniciar rebuilds solapados por creer que un timeout equivale a rollback.

2. **Upstream/unidad mínima.** [hnswbuild.c en P:479-545,680-737,1075-1177][PG-HB], `InsertTuple`, `BuildGraph`, `BuildIndex`: in-memory→on-disk con NOTICE, workers y progreso del host. [ivfbuild.c en P:429-474,966-1073][PG-IB], `ComputeCenters`, `AssignTuples`, `CreateEntryPages`: centros→asignación→sort/load. Es extraíble la **declaración de fases, recursos y transiciones**; el diseño de DSM/workers y sus presupuestos PostgreSQL no son portables.

3. **Delta Vanta comprobado.** [V-admin:19-32][V-ADMIN] ejecuta secuencialmente vector→derived→text→sparse→scalar con `?`; ya existe el ciclo de rebuild. [V-rebuild:16-58,74-85,155-179][V-REBUILD] materializa operaciones en `Vec`, hace `write_backend_batch` y después actualiza estado; hay métricas y schema version. [V-maint:463-509][V-MAINT] ya reconstruye y publica HNSW. Además, [V-insert:30-36][V-INSERT] ya decide Incremental/Rebuild/Auto en `batch_should_insert_hnsw`. Delta: reporte **por fase** y límites de recursos honestos; no vender como inexistentes bulk, rebuild o instrumentación.

4. **Alternativas/decisión.** Envolver el flujo secuencial existente es la decisión mínima. Añadir workers o reconstrucciones simultáneas se rechaza: compiten por locks/RSS, no hay evidencia de mejora y cambia el modelo de fallo. Un job/scheduler nuevo duplica mecanismos existentes. Un contador de fases sin estado de error sería insuficiente; un presupuesto “duro” que la etapa no puede obedecer sería una promesa falsa.

5. **Contrato Rust propuesto.** `fn rebuild_index_reported(&self, declaration: MaintenanceResources) -> Result<MaintenanceRunReport, MaintenanceRunError>`. Inputs: límites declarados (`memory_bytes`, `workers`, alcance solicitado, carácter `Hard|Advisory|Unknown`), sin parámetros nuevos de ranking. Output: `run_id`, alcance **real**, fases ordenadas con `Started|Completed|Failed|NotStarted`, contadores disponibles, duración, recurso observado `Option`, advertencias y `last_completed_phase`. Error estructurado incluye reporte parcial: `ReadOnly`, `LockTimeout`, `Storage`, `UnsupportedResourceLimit`, `CancelledBetweenPhases`.
   Invariantes: fases no se ejecutan en paralelo; `Completed` requiere resultado de etapa y publicación correspondiente; `success` global solo si todas terminaron. Una declaración dura no soportada se rechaza antes de comenzar; memoria no medida es `None`, no cero. Cancelación mínima solo **entre fases**, no interrumpe inseguramente la escritura de una fase.

6. **Seam real/locks/callers.** `Embedded::rebuild_index` y `src/cli_handlers/index.rs::cmd_rebuild_index:14-68` son entrada/reporting; `IndexRebuildReport` en `src/sdk/types/search.rs:32-47` ya contiene scanned/indexed/tombstones/duración. `StorageEngine::rebuild_vector_index` primero hace **`flush()` antes de adquirir `insert_lock`**, porque no es reentrante; luego `fresh_box`, vector_store[0].read, persist/sync y `self.hnsw.store(Arc::new(rebuilt))`. `acquire_insert_lock` [V-lock:684-696][V-LOCK] mapea contención a `Timeout`; wasm usa try_lock. No envolver todo `rebuild_index` con ese mismo lock ni llamar `flush_pending_hnsw` dentro de una guard ya tomada. `write_backend_batch` [V-part:20-23][V-PART] delega atomicidad al backend: no vuelve atómicas todas las fases.

7. **Pasos/persistencia/migración/rollback.** Primero reportar límites actuales y fases; propagar errores con último estado; publicar progreso fuera de locks; declarar cancelación entre fases; solo si se necesita reanudación, guardar estado versionado de ejecución en metadata administrativa existente. Al reabrir, una fase `Started` sin cierre se marca `Interrupted/Unknown`, nunca completada por inferencia. Reejecutar mediante rebuild actual, no reconstruir otra tubería. No hay rollback global acreditado: si texto falla después de ANN, conservar evidencia de ANN aplicado y reparar fases faltantes. Desactivar el wrapper conserva API/reportes anteriores. El alcance de `rebuild_vector_index` leído es L0; no prometer cobertura L1+ por el comentario general “all indexes”.

8. **Dependencias/relaciones.** C9 usa admin, informes/metrics y scheduler existentes si se agenda; no depende universalmente de C13. C15 valida compatibilidad antes de un build; C16 consume progreso/retención; C7 debe invalidar continuaciones ante cambio de generación. Certificar cobertura multi-segmento o memoria dura exige lectura/implementación adicional, no se declara resuelta en este informe.

9. **FMEA concreto.** Texto falla tras swap ANN → `?` retorna error sin revertir ANN → cliente reintenta todo e incrementa coste → reporte parcial por fases → reanudar mediante ruta administrativa existente, con exclusión de trabajos solapados. Callback de progreso bajo `insert_lock` que vuelve al SDK → reentrancia/deadlock → traza de lock/timeout → emitir eventos/copiar contadores tras liberar guard. `Vec<ops>` grande → presión RSS/OOM → memoria real no acotada por wrapper → declarar advisory/unknown o rechazar hard limit; no prometer streaming hasta modificar la fase concreta. Crash durante publicación → estado administrativo no demuestra archivo válido → inspección/rebuild y `Interrupted`, no `Completed`.

10. **Aceptación/observabilidad futura.** Normal: mismo orden/resultado que rebuild actual, con contadores de cada fase. Adversos: read-only, timeout de lock, error de una etapa tardía, cierre/reapertura con etapa iniciada y cancelación entre etapas no borran evidencia de las completadas. Validar no doble adquisición del lock y no API “cancelled” mientras una fase sigue escribiendo. Métricas: fase, duración, contadores/tombstones, recurso observado/desconocido y causa; sin payload ni rutas privadas exportadas por defecto. No se fija speedup ni nuevo presupuesto.

11. **Mínimo/no objetivos/abierto.** Mínimo: wrapper/reporte de fases y recursos honestos, sin concurrencia nueva. No objetivos: PostgreSQL DSM, entrenamiento IVF nuevo, scheduler paralelo, snapshot atómico multiíndice, rollback de fases publicadas ni mejorar P99 sin medir. Abierto: alcance multi-segmento efectivo y persistencia de run_id, sujetos a revisión de callers/recovery antes de prometer reanudación.

### 4.4 C16 — Auditoría de mantenimiento, deleted y retención física

1. **Problema de agente.** Tras borrar un recuerdo sensible, la aplicación necesita distinguir “ya no recuperable en esta API”, “tombstone aplicado”, “ANN reparado” y “bytes reclamados”. Un contador de filas o un éxito lógico no acredita olvido físico en WAL, versiones, backups o exportaciones. El operador necesita evidencia administrativa, no acceder al contenido borrado para inspeccionarlo.

2. **Upstream/unidad mínima.** [hnswvacuum.c en P:37-174,548-612,618-840,889-934][PG-HV], `RemoveHeapTids`, `RepairDeleting`, `ConfirmRepaired`, `MarkDeleted`: retirar TIDs, reparar conectividad, esperar scans y reutilizar slots son pasos distintos. [ivfvacuum.c en P:18-163][PG-IV] retira entradas bajo cleanup lock y recupera espacio de página sin cambiar centros. No copiar estructuras de página; extraer inventario de superficies y resultado de cada fase de mantenimiento.

3. **Delta Vanta comprobado.** [V-cli:118-205][V-CLI] ya tiene auditoría textual (`audit_text_index[_deep]`), no auditoría genérica inexistente. [V-memory:1686-1755][V-MEM] tiene `delete_inner`, `delete_certified` y certificado con límites declarados; [V-delete:19-65,128-155][V-DEL] tiene WAL tombstone, vector tombstone, backend delete y mantenimiento de índices. [V-maint:513-565][V-MAINT] compacta layout L0 y [V-ops:130-155][V-OPS] retira versiones por cutoff. El delta es **unir diagnóstico de mantenimiento/retención con alcance y evidencia** sin rebautizar todo como “purga completa”.

4. **Alternativas/decisión.** Extender reportes/certificados y su vista C16; no crear otro log de auditoría ni otro GC. “VACUUM Rust” como subsistema paralelo se rechaza. Un botón único “borrar físicamente todo” también: medios físicos/archivos externos no están controlados por esta función. Mantener CLI textual como entrada válida y Studio como presentación adicional, no dependencia del motor.

5. **Contrato Rust propuesto.** `fn audit_index_retention(&self, scope: MaintenanceScope) -> Result<IndexRetentionReport, RetentionAuditError>`. `MaintenanceScope` identifica namespace/colección solicitada y segmentos que pueden inspeccionarse. Output por superficie: `logical_visibility`, `ann_membership`, `tombstones`, `reclaim_status`, `physical_bytes: Option<u64>`, `verified_scope`, `observed_generation`, `evidence: Observed|NotAssessed|Unknown`, `last_maintenance`, `warnings`. Los campos son resumen administrativo; identificadores individuales solo opt-in autorizado.
   Errores: `InvalidScope`, `ReadDenied` en adapters que autorizan, `UnsupportedSurface`, `ReadFailure`, `GenerationChanged`; dato no observado no se convierte en “cero pendientes”. Invariantes: read-only, no desencadena flush/compactación; borrado lógico no implica reclamación física; generación cambiante se declara, no produce certificado de snapshot. `reclaim_status=Completed` solo para superficie/alcance verificados; nunca para backups externos.

6. **Seam real/ownership.** `src/cli_handlers/index.rs::cmd_audit_index` y `src/sdk/serialization/impl_rebuild.rs` para auditorías; `src/sdk/api/memory.rs::delete_certified`, `src/attestation.rs` para evidencia ya existente; `StorageEngine::apply_delete_inner`, `compact_layout_bfs`, `gc_mvcc_versions` para observación limitada de sus superficies, no ejecución desde la lectura. UI complementaria: `desktop/src/components/indices/IndicesLens.tsx::IndicesLens`, `desktop/src/components/SopPanel.tsx`, puente `vanta.ts::metrics` (§12). Copiar contadores bajo sus guards existentes y soltarlos antes de I/O/UI; no mantener lock de purge mientras se pide compactación.

7. **Pasos/persistencia/migración/rollback.** Inventariar qué certificados y métricas ya prueban cada superficie; añadir vista read-only con valores `NotAssessed`; conectar estados C9 cuando existan; enriquecer con inspecciones reales por segmento. Guardar solo resumen de ejecución/schema si se requiere historial, utilizando metadata/audit existentes; no persistir copias de payload borrado. Certificado anterior conserva formato e integridad; ampliación requiere versión, no editarlo retrospectivamente. Desactivación elimina la vista adicional, no restaura recuerdos; fallos se reauditan/reparan por rutas actuales con permiso separado. No hay migración de WAL ni rollback de una eliminación física.

8. **Dependencias/relaciones.** C16 es consumidor de C9 y de los certificados de purga actuales; puede empezar con sus datos sin C13. C15 explica representación/segmento inspeccionado. C7 distingue invisibilidad lógica de falta de cobertura. Borrado físico no depende de que un LLM confirme nada; permisos del adapter son prerrequisito para exposición, no nueva política de autorización del motor.

9. **FMEA concreto.** Tombstone escrito pero ANN pendiente → filas muertas consumen candidatos/retención → confusión con recall o cumplimiento de borrado → comparar superficie primaria/ANN y estado de mantenimiento → separar diagnóstico y reparar mediante flujo existente. Snapshot vivo cambia durante audit → cifras mutuamente incompatibles → certificado falso de exhaustividad → generación/corte no acreditado → `Unknown/GenerationChanged`. Operador interpreta compactación L0 como todos los segmentos → remanencia no reportada → alcance por segmento y `NotAssessed` fuera de él. Exportar IDs/rutas en diagnóstico → fuga administrativa → acceso restringido y agregados por defecto.

10. **Aceptación/observabilidad futura.** Normal: delete seguido de auditoría separa resultado lógico y retención; certificado y UI dicen el mismo alcance. Adversos: índice no cargado, datos L1+, error de lectura, mantenimiento concurrente y ausencia de métricas no se muestran como “saludable/0”. Verificar que una consulta read-only no compacta ni altera WAL. Trazar run_id/fase/superficie, contadores y advertencias sin texto/vector; observación de reclamación física requiere medir esa superficie, no restar contadores de memoria.

11. **Mínimo/no objetivos/abierto.** Mínimo: tabla de evidencias existentes con `NotAssessed` explícito y navegación a reparación administrativa actual. No objetivos: nuevo WAL/GC, promesa de secure erase, deshacer delete, indexador paralelo o certificados de terceros. Abierto: cómo obtener conteos coherentes entre segmentos sin bloquear escritores; hasta entonces declarar muestreo temporal/no snapshot, no una garantía global.

### 4.5 C13 condicionado — Evidencia de aplicación de proyecciones en Vanta

1. **Problema de agente.** Un cliente Vanta escribe y luego consulta por texto/vector; necesita saber qué significa el éxito del primario si una proyección quedó pendiente o falló. **La hipótesis procede del pipeline Vanta leído, no de VACUUM de pgvector**. pgvector no demuestra aquí una fila committed que deba esperar “ready” por vacuum para ser visible.

2. **Upstream/unidad mínima.** [hnswinsert.c en P:243-269,370-408][PG-HI] y [hnswvacuum.c:889-934][PG-HV] enseñan a distinguir cambios físicos, WAL de página, mantenimiento y visibilidad del host. No hay en esos símbolos `ProjectionStatus`, recibo de mutación ni cola eventual del SDK. La lección transferible es **no confundir fronteras de evidencia**, no trasplantar un contrato que upstream no tiene.

3. **Delta Vanta comprobado.** [V-memory:1261-1277][V-MEM] implementa `put_certified -> (MemoryRecord, WriteReceipt)`; [V-attest:678-703][V-ATTEST] contiene versión, status `recorded`, superficies store/wal, binding e integridad. **`MutationReceipt` no existe acreditado: es propuesta C13**. `put_one` toma `purge_lock`, hace `engine.insert` y después `replace_derived_indexes` (`:1029,1130,1180`); `delete_inner` también separa motor/derived (`:1700-1711`). [V-ops:163-195,201-240][V-OPS] tiene cola HNSW y drain; [V-maint:35-92][V-MAINT] ya coordina drain→snapshot→checkpoint. Delta condicionado: evidencia por proyección/revisión cuando el recibo actual no responde esa pregunta; no otro registro de durabilidad.

4. **Alternativas/decisión.** Mantener `WriteReceipt` intacto y añadir un reporte opt-in asociado es preferible a cambiar el significado de `recorded`. Hacer flush de cada índice por cada put se rechaza por coste y cambio semántico; introducir una cola/replay paralela también. Si después de revisar todos los callers el contrato actual ya satisface el caso, rechazar C13 para ese camino. La opción mínima reporta **Unknown** donde no existe prueba por revisión, no inventa watermarks desde tamaño de cola.

5. **Contrato Rust propuesto.** `fn put_with_projection_evidence(&self, input: MemoryInput) -> Result<MutationReceipt, MutationEvidenceError>`, donde el **nuevo** `MutationReceipt { record_revision, write_receipt: Option<WriteReceipt>, primary_outcome: Recorded|Indeterminate, projections: Vec<ProjectionEvidence> }` no sustituye el recibo actual con binding de contenido e integridad
SHA-256, **sin firma digital**: `src/attestation.rs:92-99,726-738,920-954`
no acredita firma ni identidad de firmante y no detecta a un actor capaz de
recalcular hashes; tampoco acredita durabilidad de crash automática. `ProjectionEvidence { projection, source_id, source_revision, generation: Option<u64>, application: ObservedApplied|PendingObserved|Unknown|Failed, persistence: CheckpointEvidence|NotAssessed }`.
   Errores propuestos: `RejectedBeforeWrite`, `EvidenceReadFailure`, `MutationOutcomeIndeterminate { operation_id }`; tras un fallo tardío no declarar “no se escribió”. Inputs/outputs se ligan al `(namespace,key,version)` del registro; `ObservedApplied` requiere observar **esa revisión**, no solo ID presente o cola vacía. `CheckpointEvidence` debe referenciar una frontera existente verificable; sin mapeo operación↔checkpoint se usa `NotAssessed`. Una observación aplicada en RAM no significa fsync ni igualdad con revisión posterior.

6. **Seam real/locks.** `src/sdk/api/memory.rs::put_one/put_certified/delete_inner`, `src/attestation.rs::build_write_receipt`, `src/storage/engine/ops.rs::try_push_pending_hnsw/drain_hnsw_batch_locked`, `src/storage/engine/maintenance.rs::flush`, `src/wal.rs::WalRecord/WalWriter::append`. El caller que ya tiene `insert_lock` llama la variante `_locked`; no reentra por `flush_pending_hnsw`. `purge_lock` serializa mutaciones SDK; no asumir que abarque todos los callers de motor/bindings. **`src/wal.rs` no es `vanta-memory/src/utils/checkpoint.rs`**; el RMW de un checkpoint de tareas no acredita ninguna de estas fronteras.

7. **Pasos/persistencia/migración/rollback.** Primero mapear éxito/error de cada fase actual y qué revisión se puede demostrar; añadir reporte opt-in; asociarlo a recibo de escritura sin modificar su integridad; registrar solo evidencia administrativa mínima si C13 lo exige. Estados pendientes persistentes, si se aprueban, se versionan en metadata derivada vinculada a revisión y se reparan mediante replay/rebuild **existentes**, no un segundo WAL. Tras restart toda prueba solo en RAM pasa a `Unknown` hasta validar generación/checkpoint. Rollback desactiva reporte, preserva primario, receipts anteriores y rutas de recovery; no elimina datos para “compensar” un error de proyección.

8. **Dependencias/relaciones.** Depende de definición transversal C13 y de trazar las rutas de mutación/recovery relevantes; no es prerrequisito universal del resto. C9 aporta eventos de rebuild; C16 visualiza evidencias; C15 define identidad de proyección/capability. Una revisión no probada no puede usarse para autorizar lectura ni sustituir C7. No se propone un subsistema nuevo de entrega “exactly once”.

9. **FMEA concreto.** Drain consume batch con `mem::take` y `add_node?` falla a mitad → restantes no tienen prueba aplicada por esa llamada → recibo basado solo en cola vacía sería falso → registrar resultado por fase/revisión o `Unknown`, recuperación/rebuild existente; no afirmar pérdida durable sin trazar recovery. Upsert de misma clave después del recibo → ID sigue presente, versión distinta → falso `Applied` → comparación estricta de versión. SDK devuelve error tras motor→derived → retry ciego puede repetir efectos → outcome indeterminado + consulta por identidad/revisión antes de reintentar. Crash post-primario → evidencia volátil desaparece → no inferir pendiente/aplicada desde vacío → revalidación.

10. **Aceptación/observabilidad futura.** Normal: escritura certificada mantiene significado/integridad de `WriteReceipt`; reporte solo acredita revisión observada. Adversos: fallo después de `engine.insert`, error parcial de drain, actualización/borrado posterior y restart diferencian rejected/recorded/indeterminate. Validar que un recibo viejo falla o queda histórico, no se “actualiza” silenciosamente. Observabilidad: proyección, estado, razón y generación opcional; no payload/vector ni contenido de WAL. Validación de crash queda pendiente manual, no realizada aquí.

11. **Mínimo/no objetivos/abierto.** Mínimo condicionado: reporte opt-in de fronteras comprobables y `Unknown`; no modificar durabilidad ni añadir replay. No objetivos: supuesta espera de vacuum para ready en pgvector, nuevo WAL, fsync por proyección, 2PC multiíndice, lectura linealizable universal o exactly-once. Abierto: identificación durable de revisión aplicada en cada proyección y compatibilidad de bindings; si no puede demostrarse, se mantiene Unknown o se descarta el candidato, no se rellena con contadores globales.

## 5. Incidencias públicas revalidadas

Se releyeron **cuerpos y metadata JSON públicos**, no se reutilizaron etiquetas del borrador. Estado y timestamps siguientes son los observados en respuestas de `api.github.com/repos/pgvector/pgvector/issues/{n}` en esta ampliación; son fuentes vivas, distintas del código P. No se afirma un fix commit ni merge de PR sin evidencia.

| Issue/título verificado | Fechas/estado acreditados | Cuerpo, mecanismo y alcance | Transferencia y validación discriminante pendiente |
|---|---|---|---|
| [#259 — Understanding HNSW + filtering](https://github.com/pgvector/pgvector/issues/259) | Creado 2023-09-11; cerrado/actualizado 2024-10-30; `closed`, 87 comentarios en respuesta. | Pregunta cómo combinar índice en `category`, `WHERE category=$1`, ANN y `LIMIT 10`; propone poda previa/particiones. Es discusión de diseño, no demostración de fuga ni reporte reproducido de corrupción. El cuerpo no fija versión. P acredita post-scan/iteración en `hnswscan.c`, no pushdown universal de cualquier predicado SQL. | C7: contrastar mismo predicado bajo plan exacto/ANN y reportar candidato/stop; no transformar la discusión en “todos los filtros fallan actualmente”. Cierre no acredita solución completa. No se leyeron los 87 comentarios. |
| [#54 — Hosted Providers](https://github.com/pgvector/pgvector/issues/54) | Creado 2023-01-28; cerrado 2025-04-14; actualizado 2025-04-14; `closed`, locked. | El cuerpo pide agregar proveedores que soporten pgvector; lista inicial Supabase. **No es un bug de dependencia/host ni una lista de compatibilidad auditada hoy.** No revisión afectada ni fix aplicables. | No trasladar al motor embebido. Si se compara un servicio hospedado, verificar versión/opciones/privilegios efectivos de ese servicio; no inferirlos del cierre. Se descarta como evidencia de defecto. |
| [#461 — Increase max vectors dimension limit for index](https://github.com/pgvector/pgvector/issues/461) | Creado 2024-02-16; cerrado 2025-02-25; actualizado 2025-03-26; `closed`, locked. | Pregunta por constantes HNSW/IVF=2000. Primer [comentario leído](https://github.com/pgvector/pgvector/issues/461#issuecomment-1948641449), 2024-02-16, vincula límite a tamaño de página por defecto y floats de 4 bytes; es explicación de contribuidor, no auditoría de todos los formatos. Se leyeron los tres primeros comentarios, no el hilo entero. P conserva límites por tipo/support y halfvec=4000; no se infiere que cerrar eliminara el límite de vector f32. | C15: diferenciar máximo del tipo y del índice/codec; Vanta ya valida dimensión de su índice. No duplicar límites 2000 ni declarar bug común a motores con almacenamiento diferente. Ensayo futuro distingue formato aceptado de indexable y error antes de escritura. |
| [#678 — Iterative index scans](https://github.com/pgvector/pgvector/issues/678) | Creado 2024-09-22; cerrado 2024-10-30; actualizado 2024-10-30; `closed`. | Cuerpo presenta **ramas de trabajo** hnsw-streaming/ivfflat-streaming, nombres históricos `hnsw.streaming`/`ef_stream`; explica descartados en capa cero, lotes ef/probes y problema de cuándo parar con filtro de distancia y solo nueve elegibles para LIMIT 10. Es propuesta entonces, no PR merge acreditada. P sí contiene implementación `iterative_scan`, distinta de esos nombres. | C7: fin por recursos y parcialidad explícitos; C9: no confundir scan con build. Los primeros comentarios leídos discuten que nkeys era cero, no prueban fix del planner actual. Discriminante: filtro imposible/insuficiente debe terminar sin prometer top-k completo. |

**Protocolo y límites de acceso:** se cargó `coordinated-web-search`, se listaron herramientas/servidores. Búsqueda keyless `site:github.com/pgvector/pgvector/issues filtering iterative scan 259 678` devolvió cero resultados y Bing error HTTP 301; se escaló a metasearch GitHub (`pgvector iterative scan filtering`), que devolvió repositorios externos de benchmarks, **no evidencia de estos issues**. El fetch público directo de los cuatro JSON sí funcionó. La petición a comentarios de #678 con `per_page=100` fue truncada por el extractor: solo se usa el fragmento inicial leído y no se atribuye cierre/fix a comentarios no recibidos. #461 `comments?per_page=3` se leyó completo. Fuentes oficiales de Generic WAL/locking corroboran las fronteras técnicas; no hace falta convertir popularidad de un issue en causa probada.

**Estado de fixes:** esta sección acredita discusión histórica y capacidades presentes en P. No acredita fecha/commit exactos de incorporación ni que toda variante del problema esté resuelta. La implementación P y TAP son evidencia de presencia/intención, no resultado de ejecución ni demostración de ausencia de regresiones. No se investigaron CVEs de todo PostgreSQL/proveedores: no es certificación de seguridad.

## 6. Prerrequisitos y relaciones

| Candidato | Reutilización previa y condición concreta | Qué no bloquea |
|---|---|---|
| C7 | Mantener `matches_memory_filters`, exact prefilter, Bloom recheck y budgets existentes. Para continuar, añadir solo capability/cursor realmente soportado por `VecIndex`/backend y generación estable. | El reporte de estrategia no requiere C13, nuevos rankings ni reindex. |
| C15 | Resolver colección lógica↔índice compartido; validar operadores y dims desde capacidades reales. Heterogeneidad incompatible necesita aislamiento/migración, no relajar `validate_vector_dimension`. | Consulta de capacidades read-only y representación legacy unknown no exigen nuevos codecs. |
| C9 | Preservar orden secuencial de `Embedded::rebuild_index`, flush antes de insert_lock y errores por fase. Revisar caller/recovery para promesas multi-segmento. | No requiere paralelismo, job manager nuevo ni activar C13. |
| C16 | Reusar auditoría textual, purge certificates y métricas; declarar alcance/superficies no inspeccionadas; autorización en adapter/UI antes de exponer detalles. | Diagnóstico no necesita ejecutar compactación/rebuild ni reescribir WAL. |
| C13 | Aprobar semántica de evidencia por revisión y trazar todos los caminos participantes antes de prometer Applied/durable. `WriteReceipt` se conserva. | No es fundamento obligatorio de C7/C15/C9/C16 ni una deducción sobre pgvector. |

Relación operativa: **C15 describe capacidad → C7 la consume; C9 cambia/publica índices → invalida continuidad y alimenta C16; C13, si se aprueba, observa revisiones, no dirige el motor**. Los presupuestos de memoria/contexto de agentes, scheduler/checkpoints de tarea y cuotas por namespace ya existen [en la referencia](VANTA-REFERENCE.md); no sustituirlos por las GUC PostgreSQL.

## 7. Qué no trasladar

- **PostgreSQL/PGXS como runtime embebido alternativo.** AM callbacks, reloptions, Datum/varlena, buffer manager, heap TIDs y GenericXLog requieren el host. Portarlos introduciría un segundo modelo de almacenamiento/transacción sin caso de agente que lo justifique.
- **Ranking o formatos nuevos por analogía.** HNSW/IVF/Flat, cuantización, sparse, BM25/RRF ya están en Vanta. C7 no cambia ranking; C15 no convierte sparse en cuantización ni promete soporte de cada opclass SQL.
- **Workers/build paralelo como receta.** Upstream comparte DSM y locks bajo planificación PostgreSQL; Vanta tiene rebuild secuencial y locks específicos. Más workers no es mejora acreditada ni entra en C9.
- **VACUUM como señal de ready o como secure erase.** MVCC del heap y retención ANN son cosas distintas; limpieza de página tampoco prueba eliminación de WAL/backups. No crear otro WAL para imitar mantenimiento.
- **Prescripción SQL universal.** COPY/CREATE INDEX tras carga/particiones/índices parciales/SET LOCAL son opciones válidas del host; no todos los workloads requieren bulk diferido, y Vanta no necesita una gramática SQL para explicar filtros.
- **Issue #54 como carencia.** Es inventario de proveedores; no oportunidad extraíble de memoria. Tampoco se propone adapter Postgres nuevo sin un caso de interoperabilidad aprobado.
- **Cambios de benchmark o evaluación.** No se crean runners, prompts, scoring, thresholds, datasets, configs ni gates; §9 describe lo existente y el contrato de adaptación, no una implementación de harness.

## 8. Advertencias y confianza

1. **Hecho estático ≠ ejecución.** Las cinco pasadas de vacuum y los tests de concurrencia existen; no se ejecutaron. Un comentario TODO no acredita el fallo actual de todos los caminos. El clon shallow limita historia y rastreo de fixes.
2. **Separar contrato host/extensión.** Se leyó documentación oficial PostgreSQL de Generic WAL e index locking; no todo el código de COMMIT/executor/recovery del host. Una emisión `GenericXLogFinish` no es por sí sola confirmación durable del cliente; influyen logged/unlogged y configuración del servidor.
3. **Seguridad de filtros.** `WHERE`/scope no sustituyen authz; roles/RLS viven en PostgreSQL, y adapters/InjectionPolicy en Vanta tienen su propio alcance. Un ANN compartido altera recall/coste entre tenants aunque el recheck evite filas ajenas; eso no acredita aislamiento de canales laterales.
4. **Entradas hostiles/costes.** Dimensiones, nnz, no finitos y buffers se validan en rutas citadas, pero no se auditó toda superficie SQL/binaria/TOAST ni fuzzing. Budgets iterativos son recursos, no límite del tamaño del payload ni defensa completa de DoS.
5. **Diagnóstico/retención.** EXPLAIN puede mostrar filtros y `ANALYZE` ejecuta consultas; NOTICE/logs y certificados pueden contener identificadores. Reducir/redactar exposición y restringir acceso antes de mostrarlos en Studio; no copiar contenido sensible a telemetría.
6. **No ausencia global.** “No acreditado” se limita a las fuentes concretas. La frase anterior “sin brechas de índice” era demasiado absoluta: aquí se documenta un límite contractual y de alcance en rutas específicas, no una auditoría global Vanta.
7. **Mantenimiento del tooling no autorizado.** Se cargó skill codebase-memory para navegación CodeGraph; no se reindexó, regeneró ni cambió metadata/grafos. Las consultas en disco al pin prevalecen frente al grafo mutable. No se modificó runtime, código, contratos, índices documentales o JSON congelado.

## 9. Benchmarks, regresión y medición operativa

### 9.1 Entry points reales y efectos de ejecución

**Corrección sustancial del borrador:** no era válido afirmar “Makefile build-only” ni concluir ausencia de evaluación por mirar el árbol. [Makefile:1-12,51-62][PG-MAKE] descubre `test/sql/*.sql` y configura `REGRESS`; `prove_installcheck` elimina **`tmp_check`** y ejecuta `prove ... test/t/*.pl` con `TESTDIR`, `PGPORT` y `PG_REGRESS`. La ruta es **`test/t`**, no `test/tap`. La búsqueda literal inicial de `test/tap` falló; se corrigió leyendo los Perl. [CI build:24-41,54-76][PG-CI] conecta SQL/TAP en sus jobs; es configuración escrita, no un resultado observado.

Los comandos documentados upstream son `make installcheck` y `make prove_installcheck`, o `PROVE_TESTS=test/t/012_hnsw_vector_build_recall.pl` para uno; **solo se transcriben para revisión manual posterior, no se ejecutaron ni se solicita ejecutarlos durante esta investigación**. Requieren extensión compilada/instalada, servidor/herramientas PostgreSQL, módulos Perl `PostgreSQL::Test::*` y dependencias de la infraestructura TAP (CI instala IPC::Run). Los tests llaman `init/start`, crean bases/tablas/índices, escriben archivos/WAL, hacen DROP/TRUNCATE/VACUUM y algunos levantan réplicas/pgbench. No son comandos read-only; no ejecutarlos en una base compartida ni inferir aislamiento por su nombre.

| Familia leída en P | Entrypoint, datos/defaults y cadena | Métrica, denominador y casos de fallo | Alcance/efectos y límite |
|---|---|---|---|
| HNSW build recall | [`012_hnsw_vector_build_recall.pl:7-129`][PG-T012]: cluster→10.000 vectores 3D, componentes `random()*random()`→20 consultas Perl rand→gold exacto antes del índice→build serial y paralelo→`test_recall`. Opclasses L2/IP/coseno/L1; opciones HNSW por defecto. | `correct/total`, donde total incrementa **por cada ID gold emitido**; pertenencia de gold en set actual. Plan `EXPLAIN ANALYZE` con seqscan off debe contener Index Scan. Mínimo **0,97 IP; 0,99 resto** es aserción del test, no resultado. `psql` comprueba ret=0/DEBUG de workers. | Se comprueba recall tras build serial/paralelo en memoria; para build paralelo de 4MB en disco se comprueban retorno/workers/NOTICE, **no se vuelve a llamar test_recall después**. No afirmar cobertura de calidad de esa variante desde el NOTICE. |
| HNSW insert recall | [`013:7-110`][PG-T013]: índice vacío→pgbench **10 clientes × 1000 transacciones**, una fila por transacción→gold exacto con indexscan off→misma familia de consultas/opclasses. | Denominador por IDs gold; mínimos 0,97 IP/0,99 otros. Además retorno pgbench=0, stdout “actually processed”, stderr vacío. No contador de exactitud por inserción. | Carga nominal 10.000 inserts si todos se completan; la aserción de proceso no es medición de p99. DROP/TRUNCATE entre opclasses. Inserción concurrente, no test de continuación iterativa. |
| IVF build recall | [`003:7-145`][PG-T003]: 100.000 vectores 3D uniformes y 20 queries rand; gold por todos los IDs con distancia ≤ frontera del top-20; build serial/paralelo, listas default=100. `test_recall(probes,...)` fuerza Index Scan por nombre idx. | Numerador cuenta **IDs actuales** pertenecientes a set gold; denominador **20 × nº consultas**, no tamaño gold expandido por empates. Con probes 1/10 se exige 0,71/0,95 salvo IP, omitido con TODO por distribución uniforme; con probes=100, 1,00 salvo coseno=0,9925. | Valores son thresholds **condicionales existentes**; no generalizarlos a embeddings semánticos o a IVF Vanta. El planner está forzado para comprobar AM aunque documentación diga que probes=listas puede evitar el índice en selección natural. |
| IVF insert recall | [`004:7-111`][PG-T004]: índice creado antes de datos→10 clientes×1000 transacciones, **10 filas por transacción**, gold exacto después; consultas 3D/L2/IP/coseno. | A diferencia de 003, recorre IDs gold y suma total por gold. Probes 1/10: 0,71/0,95 salvo IP omitido; probes=100 exige 0,9925 en la sección leída. No asumir denominador idéntico entre todos los tests IVF. | Carga nominal 100.000 filas, distinto del número de operaciones. Construir IVF sin datos aquí sirve a prueba de inserts/errores, no recomienda entrenar producción sin muestra. |
| Iterativo filtrado HNSW | [`044:7-118`][PG-T044]: 50.000 vectores 3D uniformes, 20 queries, `LIMIT=20`, filtro `i % c = 0`, c=50/500; build `maintenance_work_mem=128MB`; L2/coseno, ef_search=40 y strict/relaxed. Gold con indexscan off y frontera top-20. | Cuenta actual∈gold; total suma **20 por consulta**. Exige Index Scan usando idx y recall≥0,99 para ambas selectividades/modos. Vacío suma cero aciertos pero mantiene denominador; resultados parciales penalizan. | Selectividades nominales 1/50 y 1/500, no RLS ni ACL real. No mide costo por ampliar exploración ni latencia percentil. No demuestra top-k exacto en toda dimensión. |
| Iterativo filtrado IVF | [`042:7-125`][PG-T042]: 100.000 vectores 3D, 20 queries, LIMIT20, c=100/1000, L2/coseno, relaxed; probes especificado. | Actual∈gold / (20×queries). c=100: probes1≥0,57, probes10≥0,98; c=1000: probes1≥0,80 L2 o ≥0,88 coseno. Aserción de plan idx. | Dependencia de distribución/probes explícita; no una cifra única “recall IVF”. No convertir sus thresholds en criterios nuevos para Vanta. |
| Límites iterativos | [`043:7-59`][PG-T043] y [`041:7-54`][PG-T041]: 100.000 vectores, filtro i%10000=0, LIMIT11 (solo diez elegibles). HNSW usa relaxed, max_scan=100000 y multiplier2; IVF probes10. | Primero count=10; luego promedio de count en 20 queries para topes HNSW 30k/50k/70k o IVF max_probes30/50/70, con banda **esperado±2** ya escrita. No recall contra gold en esos bloques. | Prueba comportamiento de caps/exhaustion, no timeout ni p99. max_scan es aproximado; tampoco prueba aislamiento multi-tenant. |
| VACUUM y recall | [`014:7-97`][PG-T014]: 10.000 vectores, autovacuum off, HNSW m4/ef_construction8, DELETE i>2500, 20 queries y gold exacto sobre vivos. | Denominador IDs gold. Antes de vacuum: mínimos 0,18 con ef20 y 0,93 con ef100; después: 0,95 con ef20. Esas aserciones describen **ese grafo/dataset**, no cifras medidas aquí. | Discrimina consumo de candidatos muertos de visibilidad incorrecta del heap. TODO de concurrent inserts en este test no significa ausencia global: 047 aborda concurrencia separadamente. |
| Sparse, calidad y reutilización | [`028:7-129`][PG-T028] replica build recall HNSW con sparsevec(3), 10k filas y consultas con **los tres componentes presentes**; [`038:7-42`][PG-T038] usa sparsevec(100000), tres ciclos de 100 inserts con índices únicos aleatorios y hasta 1000 intentos de nnz→DELETE pares→VACUUM. | 028 tiene denominador gold/mínimos equivalentes a 012. En 038, `safe_psql` y `is(1,1)` acreditan intención de no fallar, **no recall ni reclamación de bytes**. | 028 no representa sparsity alta aunque el tipo sea sparse. 038 sí varía tamaño/reutilización de tupla; no mide búsqueda. No atribuir a ambos la misma métrica. |
| WAL/replay | [`010:10-98`][PG-T010]: primario/backup/réplica streaming, dim32, 1000 filas iniciales; compara top10 y diez ciclos DELETE→VACUUM→100 inserts por ciclo. | Espera `current_wal_lsn <= replay_lsn`; timeout hace `die`; compara strings de resultados de primario y réplica con `is`. No ground truth de calidad ni tasa de crash recovery. | Inicia **dos** nodos, backup y WAL; igualdad tras catch-up no certifica todos los cortes de crash, ni durabilidad del cliente con cualquier setting. 001 IVF se inventarió, no se atribuye lectura detallada. |
| Carreras de mantenimiento | [`046:7-38`][PG-T046]: mil filas luego DELETE total, 5 clientes×1000 tx, scripts SELECT strict:VACUUM con pesos1000:1. [`047:7-39`][PG-T047]: INSERT/DELETE/SELECT/VACUUM pesos500/500/20/1, 5×1000 tx. [`048:7-39`][PG-T048]: IVF lists10, pesos500/500/500/1, 5×1500 tx. | Retorno pgbench0, stderr vacío y texto “actually processed”; la población real por operación depende de pesos, **no cinco mil de cada una**. No assertions de recall, fairness o ausencia de deadlock para toda carga. | Son pruebas de no-error bajo mezcla concurrente (047 señala “graph not repaired”), no benchmark publicado de throughput. Cambian datos y vacuum; no ejecutadas. |
| Memoria baja build | [`045:7-29`][PG-T045]: mil vectores 3D, mantenimiento3073kB, un worker solicitado y DEBUG. | Ret=0 y NOTICE “after 0 tuples”/workers; no score de calidad ni máximo de RSS medido. | Verifica intención del fallback a disco, no que 3073kB sea RAM total del proceso ni receta Vanta. |

### 9.2 Denominadores, aleatoriedad y fallos: qué prueban estos TAP

- **No son evaluación de agente.** Unidad = vector/consulta SQL/operación de almacenamiento. No hay sesiones de conversación, answerer, juez LLM, tokenizer ni proveedor de embeddings en los runners leídos. Coste de llamadas pagadas no aplica a esos tests; generar embeddings reales queda fuera de ellos.
- **Preparación y gold:** las consultas se generan en Perl con `rand`; datos por `random()` SQL. No se observó seed fija de esos generadores en estos scripts. El gold se obtiene mediante exact scan antes de crear índice o deshabilitando indexscan después. Es oráculo sobre el mismo corpus, no split train/test de memoria semántica. IVF entrena centros con datos del corpus: no es filtración de respuestas de un juez.
- **Empates:** 003/042/044 expanden el gold con `<= MAX(distance)` pero mantienen denominador k por query. En 042/044 el subquery top tiene filtro y el SELECT exterior de IDs por distancia no lo repite; es la implementación leída, relevante al analizar empates, no una fórmula a copiar ciegamente. La salida ANN sigue filtrada. 012/013/014 usan IDs del top-k y denominador gold efectivo. No intercambiar esos agregados sin declararlo.
- **Errores y vacíos:** `safe_psql` falla el test ante error SQL en vez de producir puntuación de abstención; `psql` verifica retorno cuando interesa capturar NOTICE. Aserciones de plan son parte del resultado: si no usa índice esperado no hay un score comparable de ANN. En fórmulas de denominador fijo, un vacío mantiene k intentados; en denominador gold, este dataset pretende gold no vacío, sin política general de abstención/zero denominator. No hay agregador que trate preguntas incompletas de un agente.
- **Concurrencia/cache/warmup:** clientes y transacciones arriba son parámetros de pruebas existentes. El EXPLAIN ANALYZE previo **ejecuta** la primera consulta y puede calentar páginas; los tests no separan warmup/cold/warm ni publican p50/p95/p99. No se conoce cache efectiva o CPU/RAM del resultado porque no se ejecutó. Default del servidor no explícito se deja desconocido.
- **Claims:** README afirma trade-offs HNSW/IVF, efectos de ef/probes y maintenance_work_mem. TAP codifica mínimos esperados y casos; no se registraron valores observados propios ni se convierte una aserción 0,99 en claim “pgvector logra 99% en nuestro corpus”.

### 9.3 Regresión SQL, instrumentación y guías de operación

[SQL hnsw_vector:1-114][PG-REG] crea una tabla de cuatro entradas (incluye cero y NULL), construye índice, inserta otra, consulta L2/IP/coseno/L1, hace TRUNCATE/DROP, prueba unlogged, strict/relaxed y opciones inválidas. [Expected:190-217][PG-EXPECTED] espera errores para `iterative_scan=on`, caps fuera de rango y vector(2001) indexado; es **salida golden de regresión**, no transcripción de esta sesión. `pg_regress` compara resultados/error esperado, no calcula QPS ni recall estadístico. No confundir un ERROR esperado con una prueba fallida.

Hay instrumentación de tiempo explícita, que el borrador omitía:

| Entrada/configuración existente | Qué mide y qué excluye |
|---|---|
| [README:1339-1355][PG-DEV] documenta compilación con `-DHNSW_BENCH -DIVFFLAT_BENCH`, y variantes MEMORY/KMEANS_DEBUG. | Requiere limpiar/compilar/instalar; **no se ejecutó**. Cambia instrumentación y logging, por tanto no es neutro para timings. No es un runner autónomo ni un dataset versionado. |
| `HnswBench`, [hnsw.h:100-113][PG-HHEADER], invocado en `hnswbulkdelete:896-909`. | `INSTR_TIME` alrededor de cada una de las cinco pasadas; INFO en milisegundos. Tiempo de esa llamada, incluido trabajo bloqueante interno; no commit cliente completo, cola de servidor, p99 ni todo VACUUM de PostgreSQL. Si la llamada falla, no garantiza emisión de duración final. |
| `IvfflatBench`, [ivfflat.h:69-82][PG-IHEADER]; `ivfbuild.c:458,470,1015-1021`, `ivfscan.c:397-408`. | Duraciones sample/kmeans/assign/sort/load y GetScanLists/GetScanItems, separadas; no todas las operaciones insert/update del sistema. Con `IVFFLAT_BENCH`, `ivfflatbuild:1063-1064` fija seed interna42 del build; eso **no fija** random SQL/Perl de TAP. |
| [README:712-748,762-772][PG-OPS] propone EXPLAIN(ANALYZE,BUFFERS), comparar ANN con exact scan mediante SET LOCAL y reindex antes de vacuum. | EXPLAIN informa plan/tiempo/buffers de la consulta y ejecuta al usar ANALYZE; no incluye ingestión/embedding. REINDEX/VACUUM escriben, consumen recursos y necesitan privilegios. Son guías operativas sin tabla competitiva reproducida aquí. |

### 9.4 Adaptación a Vanta sin diseñar otro harness

Aplican [METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md): mantener separados motor, retrieval, memoria, respuesta y éxito de agente. La adaptación conceptual mínima es reemplazar **solo el backend** de preparación/consulta por `Embedded::put_batch`, `MemorySearchRequest` y mantenimiento actual, conservando IDs, dimensión, métrica, filtro, k, población y definición de gold del runner que se seleccione. No mezclar L2 distancia creciente de PostgreSQL con similitud descendente Vanta sin conversión explícita; no equiparar sparse_dot con opclasses sparse L2/coseno.

Las dependencias PostgreSQL::Test/SQL planner/réplica no son portables a un motor embebido. El equivalente de plan sería la explicación actual ampliada por C7, **no** la regex `Index Scan`; HNSW/IVF/Flat deben rotular su ruta efectiva. Runners de no-error no se convierten en tests de calidad; el resultado desconocido/operación fallida no se borra del registro. Para futuras mediciones se usaría el harness Vanta **ya existente**, con lectura previa de su contrato, no thresholds copiados de estos datos 3D ni cambios de prompts/configs. No se autoriza ni implementa aquí ese adapter ni una comparación competitiva.

## 10. Matriz pgvector vs VantaDB

Estado V = **existe**, **parcial**, **no acreditado en muestra** o **no aplicable**. Todas las fuentes V de esta tabla son el pin completo declarado al inicio; no resultados runtime.

| Capacidad | pgvector P / fuente real | Estado V y fuente real | Delta y decisión |
|---|---|---|---|
| ANN y rutas alternativas | HNSW/IVF callbacks y exact scan del host; [PG-HS]/[PG-IS]. | **Existe:** `VecIndex` en [V-IDX], routing Flat/IVF/HNSW en [V-VEC:225-249][V-VEC]; [referencia V](VANTA-REFERENCE.md). | No sustituir motor ni implementar ANN desde cero. |
| Filtros exactos/poda | SQL executor después del ANN; índices normales/particiones según host. | **Existe:** prefilter/exact_scan, máscara Bloom y postcheck namespace/filtro [V-VEC:118-223,264-278][V-VEC]. | C7 extiende reporte; no alegar ausencia de filtros ni autorización por filtro. |
| Continuación ANN con stop/cobertura | `ResumeScanItems`/probes y caps [PG-HS]/[PG-IS]. | **Parcial:** budgets/adaptive ef/fallback existen; `VecIndex::search` retorna Vec [V-IDX:40-47][V-IDX] y [V-VEC:289-300][V-VEC] hace fallback solo con cero hits. | C7: parcialidad observable; continuidad backend no acreditada en esa firma. |
| Explain/diagnóstico | SQL EXPLAIN y estadísticas/NOTICE del host [PG-OPS]. | **Existe:** `explain_memory_search` y métricas [V-EXP], [V-ADMIN:148-161][V-ADMIN]. | Añadir dominio/stop/cobertura, no nuevo explain global. |
| Dense/sparse | Tipos SQL diferentes, límites/opclasses [PG-TYPES]/[PG-SQL]. | **Existe:** dense en [V-VEC], campo sparse/ramas en [V-EXP:28,70-83][V-EXP]; no se certifica toda paridad de operadores. | C15 declara combinación soportada; sparse no es quantization. |
| F16/bit/operadores por colección | halfvec/bit + opclasses; indexabilidad distinta de tipo [PG-HSUP]. | **No acreditado en muestra** como contrato completo por colección; `capabilities()` actual es de proceso [V-ADMIN:138-145][V-ADMIN]. | C15 mínimo = exponer lo real/rechazar unsupported, no prometer implementación física nueva. |
| Cuantización | `binary_quantize`/cast como expresiones; halfvec es precisión, sparse dominio. | **Existe:** `rabitq_quantize`/`turbo_quant_quantize` [V-QUANT], governor SQ8 [V-MAINT:604-645][V-MAINT]. | Reusar mecanismos; mantener ejes dominio/precisión/codec separados. |
| Validar dimensión | Tipo/typmod y supports de índice [PG-TYPES]/[PG-HSUP]. | **Existe:** `validate_vector_dimension` [V-SER:190-203][V-SER] y validación pre-write en `put_one`. | No reabrir defecto histórico como carencia actual; C15 amplía declaración por colección. |
| Incremental vs bulk | `hnswinsert`/`ivfflatinsert` vs `BuildIndex` [PG-HI]/[PG-II]/[PG-HB]/[PG-IB]. | **Existe:** `put_batch` contrato por chunks [V-MEM:1280-1301][V-MEM]; `batch_should_insert_hnsw` [V-INSERT:30-36][V-INSERT] decide Incremental/Rebuild/Auto. | C9 observa ciclo/recursos, no inventar otra vía bulk ni atomicidad batch-wide. |
| Rebuild multiíndice | CREATE/REINDEX bajo host; HNSW/IVF distintos builds. | **Existe**, con **alcance parcial a auditar**: `Embedded::rebuild_index` secuencial [V-ADMIN:19-32][V-ADMIN], ANN reconstruido de vector_store[0] [V-MAINT:481-490][V-MAINT]. | C9 preserva orden y reporta fases; no pipeline paralelo ni claim global L1+ sin prueba. |
| Progreso/recursos build | NOTICE/memoria, workers y pgstat progress [PG-HB]/[PG-IB]. | **Parcial:** informes finales/métricas [V-CLI:59-68][V-CLI], [V-REPORT:32-47][V-REPORT]; derived/text acumulan ops [V-REBUILD:19-46,87-165][V-REBUILD]. | C9: fases, recursos observados y límites honestos; no afirmar falta de toda observabilidad. |
| WAL/checkpoint | GenericXLog + commit/snapshot PostgreSQL [PG-WAL]/[PG-LOCK]. | **Existe:** `WalRecord`, `WalWriter::append` [V-WAL]; `flush` drain/snapshot/checkpoint [V-MAINT:35-92][V-MAINT]. | No nuevo WAL; distinto de checkpoint de tareas/memoria. No paridad ACID deducida por nombre. |
| Receipt de escritura/proyección | No contrato de receipt autónomo en AM leído; vacuum no indica ready. | **Existe:** `WriteReceipt` [V-ATTEST], `put_certified` [V-MEM:1261-1277][V-MEM]; **no acreditado** `MutationReceipt` como API actual. | Solo C13 condicionado y propuesto; no inferir estado eventual upstream. |
| Borrado/retención/auditoría | Cinco pasadas HNSW, cleanup IVF [PG-HV]/[PG-IV]. | **Existe:** tombstones/delete [V-DEL], `delete_certified` [V-MEM], auditoría textual [V-CLI]; **parcial** vista integral de alcance/retención. | C16 une evidencias existentes, no secure erase universal ni VACUUM nuevo. |
| Memoria de agente/sesiones/budgets | **No aplicable al AM**: filas/metadata de usuario, políticas de aplicación. | **Existe** según [VANTA-REFERENCE](VANTA-REFERENCE.md): RecallConfig/InjectionPolicy, sesiones, dedup, provenance y presupuestos. | No importarlos desde pgvector ni tratar ausencia upstream como defecto. |
| UI | Extensión; clientes externos y monitorización [PG-OPS]. | **Existe:** Studio fuente local en §12; separado del pin V y sin ejecución. | Extender C16 en superficies reales y rotular métricas globales/desconocidas. |

## 11. Negativos Vanta sustentados e hipótesis

**Observaciones locales acotadas, no ausencia global:**

- `vector_memory_search` retorna hits parciales no vacíos sin activar su rama exact fallback (`:289`); su retorno `Vec<MemorySearchHit>` no lleva motivo de parada/cobertura. Es límite de **esa ruta/contrato**, no prueba de recall insuficiente en toda consulta. C7 lo hace explicable; demostrar mejora exige medición propia.
- `Embedded::rebuild_index` aplica fases secuenciales con `?` y no una transacción envolvente multiíndice. Si falla una fase tardía, el reporte global no devuelve las anteriores por esa firma. Es mecanismo verificable, no un crash reproducido. No confundir atomicidad `backend.write_batch` de una fase con atomicidad de todas.
- En `rebuild_derived_indexes_with_report`, escritura de lote (`:46`) y estado (`:49-58`) son llamadas separadas; text también (`:165,168`). Un corte entre ambas es **hipótesis de estado desalineado** que exige seguir recovery/readers, no evidencia de pérdida de datos certificada. C9/C16 deben representar interrupción/unknown.
- El rebuild ANN examinado toma `vector_store[0]`, con comentario de alcance L0; la cadena admin no demuestra por sí sola que cubra todos los L1+. Se requiere rastrear archivo auxiliar, merge y callers antes de llamar a esto bug de cobertura global; mientras tanto no prometer “reconstrucción de todos los segmentos”.
- Capabilities actual de proceso no especifica matriz completa por colección. Buscar solo esa API no demuestra que ninguna otra superficie lo haga; C15 queda como delta contractual a validar antes de implementación.

**Riesgos condicionados:** presión por materializar `Vec<ops>` en rebuild (depende de volumen); evidencia por revisión después de drain parcial (depende de error/recovery); consistencia de cifras durante audit concurrente (depende de snapshot/guards). No son resultados de stress ni vulnerabilidades confirmadas.

**Falsos positivos descartados:** Vanta ya tiene índices, pre/in/postfilter, ef adaptativo, fallback exacto, validación dimensional, cuantización/governor, bulk modes, rebuild, auditoría textual, write receipts y purge certificates. `MutationReceipt` no se rebautiza como existente. No usar `Skip WAL for now` para afirmar falta de WAL; ni `deleted`/vacuum para afirmar lectura de heap invisible o que autorización dependa de postfilter.

## 12. UI: extensión, clientes externos y Studio

### 12.1 Producto upstream y frontera de cliente

El producto examinado es una extensión: [README Languages:775-805][PG-CLIENTS] declara uso desde cualquier cliente Postgres y enlaza bibliotecas externas; [Monitoring:762-772][PG-OPS] remite a pg_stat_statements/PgHero y contraste exacto/ANN. No hay UI propia acreditada por los flujos C/SQL/Makefile/README inspeccionados. **Eso no es una búsqueda exhaustiva de cada cliente o desktop del ecosistema**: no se inspeccionó el código ni licencia de PgHero, pgAdmin, bibliotecas Python/Rust u otros clientes.

Flujo concreto del operador: cliente SQL con permisos→`CREATE INDEX`/consulta de `pg_stat_progress_create_index`→fase/tuplas/bloques reportados; exceso de memoria de grafo produce NOTICE; `EXPLAIN (ANALYZE,BUFFERS)` ejecuta y muestra plan; VACUUM/REINDEX cambia almacenamiento. No se inventa un “desktop pgvector”. Esas herramientas pueden mostrar SQL/filtros/identificadores, requieren protección de acceso; las GUC no son una política de seguridad.

### 12.2 Superficies reales de Vanta Studio, con límites de revisión

Se leyeron archivos físicos en `C:\Users\Eros\VantaDB Proyect\develop-desktop\desktop`, **sin ejecutar frontend/Tauri**. La referencia común consigna HEAD observado `0ad38c542`, pero en esta pasada no se acreditó que los archivos locales citados correspondan a ese objeto: la consulta de árbol para `desktop/web` en ese pin no devolvió rutas. Por tanto, se citan como **lectura local complementaria**, no se fabrican permalinks ni paridad con snapshot V.

| Fuente local leída | Flujo/comportamiento comprobado | Oportunidad concreta, sin inventar soporte actual |
|---|---|---|
| `src/components/indices/IndicesLens.tsx:1-35,67-112` | `IndicesLens` consume `useMetricsPoll`, health y namespaceStats; fallback a `list({limit:500})` para conteos. Comentarios declaran que dims/live WAL/LSM no expuestos se muestran “—”. | C15/C16 pueden añadir capability/dimensión y alcance cuando motor lo exponga. Etiquetar fallback de 500 como vista acotada, no censo global. |
| `src/hooks/useMetricsPoll.ts:1-66` | Store compartido, un intervalo4s, histórico máximo12, evita llamadas superpuestas mediante inFlight; mantiene error separado de snapshots. | Reusar poller; no nuevo servicio/timer por tarjeta. Reportar stale/error y stop de C7/C9 sin transformar falta de datos en cero. |
| `src/vanta.ts::metrics:540-568` | Define subset de OperationalMetrics: RSS, replay, duraciones ANN/derived/text, counters BM25/HNSW; llama `transport.call("vanta_metrics")`. | Extender contrato opt-in/versionado de métricas/reportes; NOTICE sería evento/advertencia estructurada, no parsear logs C. |
| `src-tauri/src/commands/metrics.rs::vanta_metrics:17-32` | Usa `Embedded::test_empty(Config::default()).operational_metrics()`; comentarios explican contadores **globales de proceso**, no de conexión, sin abrir DB por poll. | No rotular esos contadores como estado de colección/DB activa. C9/C16 necesitan identidad/alcance explícitos o indicador global; no un “ready” por base derivado de globals. |
| `src/components/SopPanel.tsx:143-175` | Reindex es **solo lectura**: muestra últimos tiempos ANN/derived/text y dice que no hay comando de disparo por ese puente. | Mantener diagnóstico primero. No afirmar que exista botón rebuild aunque SDK/CLI sí lo tengan. Un futuro trigger necesita autorización, confirmación y estado C9; queda fuera de este mínimo. |

Propuesta de presentación C16: en la lente Índices mostrar estrategia/stop de la última consulta solo si el reporte existe; fases completadas/fallidas/interrumpidas con alcance y recurso unknown; por superficie de borrado, lógico/ANN/reclamación/NotAssessed. Mantener NOTICE de presión de memoria como advertencia operativa y EXPLAIN como diagnóstico solicitado, no ejecutar ANALYZE implícitamente desde una tarjeta. El motor sigue usable sin UI ni LLM. No se certifica comportamiento visual, accesibilidad, transport parity, HTTP/WASM, cancelación real o permisos sin revisión/ejecución posterior.

## 13. Fuentes, cobertura y cierre de autoría

### 13.1 Inventario y calidad de evidencia

| Evidencia | Cobertura efectivamente utilizada | Límite deliberado |
|---|---|---|
| Identidad/licencia/build | Manifest congelado, LICENSE completo, vector.control, Makefile y secciones dev/operación/cliente del README P. | No fetch/pull, recálculo de metadata, instalación ni validación de proveedores externos. |
| Modelo/tipos/opclasses | `vector.h:11-24`, `halfvec.h:60-73`, `sparsevec.h:11-40`; validadores y supports citados; secciones opclasses de `sql/vector.sql`. | No auditoría completa de parseo binario, casts, SIMD o kernels bit/half. |
| HNSW C | Rutas y rangos desarrollados en §2: insert/free slot/caller/Finish; scan inicial/reanudado/orden/caps; build transiciones/WAL final; cinco pasadas de vacuum. | No verificación formal del grafo, de todos los interleavings, de wraparound de versión o de cada instrucción en hnswutils. |
| IVF C | Insert/selección de centro/páginas/WAL; scan listas/sort/probes; sample/centros y build; bulkdelete/cleanup. | `ivfkmeans.c` no se auditó íntegramente ni se derivó convergencia matemática; no certificar el algoritmo por el nombre de función. |
| Evaluación | Cuerpos TAP citados en §9, SQL/expected HNSW, macros/timers y llamadas, runner Makefile/CI. | No todos los 48 TAP, SQL regression o plataformas; bit/half y tests de costo se inventariaron por nombres pero no se les atribuye resultado o protocolo detallado. |
| Fuentes públicas | JSON cuerpo/metadata de #259/#54/#461/#678; tres comentarios #461 y fragmento inicial #678; documentación oficial PostgreSQL Generic WAL/locking. | No comentarios completos #259/#54/#678 ni revisión exhaustiva de PRs/eventos/historia. Closed no equivale a fix. No estudio completo de CVEs. |
| Vanta pin V | SDK vector/explain/admin/memory; serialización rebuild/validación; VecIndex; storage maintenance/ops/delete/partition/lock y bulk helper; attestation/quantization/config/CLI, rangos citados. | No certificar todos los bindings, recovery, isolation, callers directos o todas las rutas legacy/core; hechos de otras áreas remiten a VANTA-REFERENCE, no se presentan como relectura íntegra propia. |
| Studio local | IndicesLens, SopPanel, useMetricsPoll, vanta.ts metrics y comando Rust metrics, rangos §12. | Archivos físicos complementarios sin pin acreditado; no UI ejecutada, HTTP/WASM o permisos probados. |

**Fuentes inmutables upstream:** todos los enlaces de código `PG-*` siguientes fijan **P=`724588a5504b0e9c462e3126563370e8d50a01eb`**; rangos/funciones específicos se indican junto a cada afirmación. Los enlaces V son navegación local a rutas comprobadas mediante **`git show 97c9623c8ef5a70fdb51aff215bfbb3b05b15612:ruta`**; su contenido navegable puede avanzar después del pin. Las páginas PostgreSQL “current” y APIs de issues son fuentes vivas consultadas el **2026-11-07**, no sustituyen los snapshots.

### 13.2 Correcciones respecto del informe anterior

Se conservan identidad, licencia real/NOASSERTION, papel del AM, utilidad de declarar filtros/dominio/recursos y límites de lectura. Se corrigen: defaults m/ef, sparse≠quantization, falta de flujos IVF, falta de cinco pasos vacuum, trazado incompleto de “Skip WAL for now”, confusión visibilidad/MVCC-retención, atribución excesiva de receipts a pgvector, `MutationReceipt` presentado como existente, seams genéricos y omisión de TAP/instrumentación. #54 pasa de “dependencia del host” a su contenido real de inventario; #678 distingue nombres de la propuesta histórica de los presentes en P. C15 y C9 se separan para no mezclar spec vectorial con ciclo de build.

### 13.3 Validaciones pendientes y división posterior en specs

Este documento supera una ficha breve porque conserva trazas, alternativas y límites. **No crea specs activas:** si se aprueban candidatos, separar en cinco specs C7/C15/C9/C16/C13, cada una copiando su contrato/aceptación y enlazando la evidencia P/V, con revisión independiente. Mantener aquí arquitectura, incidencias, benchmarks y matriz como estudio; no mover contratos comunes ni modificar backlog para simular aceptación.

Antes de implementación quedan por resolver:

1. C7: capacidad real de continuación por backend y semántica del token de generación, sin cambiar ranking ni filtros de seguridad.
2. C15: identidad de colección lógica, límites de índice compartido y compatibilidad de embedding-space/operador/codec.
3. C9: alcance L0/L1+ y estrategia de error/reapertura de fases; recursos duros solo si se pueden cumplir; no introducir paralelismo.
4. C16: evidencia coherente por superficie/segmento y autoridad de lectura; los bytes no observados quedan desconocidos.
5. C13: aprobación condicionada, tracing completo de callers/recovery y prueba por revisión antes de declarar Applied/durable. No confundir `WriteReceipt` vigente con `MutationReceipt` propuesto.

**Cierre propio:** autoría documental completada con diez ejes, cinco fichas de once campos y fuentes/lagunas explícitas; no implica aceptación del responsable, revisión adversarial independiente ni autorización de implementar. No se ejecutaron apps, tests, builds, benchmarks, installs, servicios, validadores globales o cambios git. El recibo temporal registra esta entrega, **no se usa como evidencia de aceptación**.

[PG-LIC]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/LICENSE
[PG-TYPES]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/vector.h#L11-L24
[PG-HALF]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/halfvec.h#L60-L73
[PG-SPARSE]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/sparsevec.h#L11-L40
[PG-HAM]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnsw.c#L468-L528
[PG-IAM]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfflat.c#L390-L450
[PG-HTUP]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnsw.h#L371-L391
[PG-SQL]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/sql/vector.sql
[PG-HI]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnswinsert.c
[PG-II]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfinsert.c#L20-L216
[PG-HS]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnswscan.c#L26-L337
[PG-IS]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfscan.c#L48-L433
[PG-HV]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnswvacuum.c
[PG-IV]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfvacuum.c#L18-L163
[PG-HB]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnswbuild.c
[PG-IB]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfbuild.c
[PG-HSUP]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnswutils.c#L1606-L1702
[PG-HHEADER]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/hnsw.h#L65-L113
[PG-IHEADER]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/src/ivfflat.h#L53-L82
[PG-MAKE]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/Makefile#L1-L62
[PG-CI]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/.github/workflows/build.yml#L24-L76
[PG-T012]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/012_hnsw_vector_build_recall.pl#L7-L129
[PG-T013]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/013_hnsw_vector_insert_recall.pl#L7-L110
[PG-T003]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/003_ivfflat_vector_build_recall.pl#L7-L145
[PG-T004]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/004_ivfflat_vector_insert_recall.pl#L7-L111
[PG-T044]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/044_hnsw_iterative_scan_recall.pl#L7-L118
[PG-T042]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/042_ivfflat_iterative_scan_recall.pl#L7-L125
[PG-T043]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/043_hnsw_iterative_scan.pl#L7-L59
[PG-T041]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/041_ivfflat_iterative_scan.pl#L7-L54
[PG-T014]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/014_hnsw_vector_vacuum_recall.pl#L7-L97
[PG-T028]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/028_hnsw_sparsevec_build_recall.pl#L7-L129
[PG-T038]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/038_hnsw_sparsevec_vacuum_insert.pl#L7-L42
[PG-T010]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/010_hnsw_wal.pl#L10-L98
[PG-T046]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/046_hnsw_vacuum_scan.pl#L7-L38
[PG-T047]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/047_hnsw_vacuum_insert.pl#L7-L39
[PG-T048]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/048_ivfflat_vacuum_insert.pl#L7-L39
[PG-T045]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/t/045_hnsw_low_memory_build.pl#L7-L29
[PG-REG]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/sql/hnsw_vector.sql#L1-L114
[PG-EXPECTED]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/test/expected/hnsw_vector.out#L190-L217
[PG-DEV]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/README.md#L1320-L1361
[PG-OPS]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/README.md#L710-L772
[PG-CLIENTS]: https://github.com/pgvector/pgvector/blob/724588a5504b0e9c462e3126563370e8d50a01eb/README.md#L775-L805
[PG-LOCK]: https://www.postgresql.org/docs/current/index-locking.html
[PG-WAL]: https://www.postgresql.org/docs/current/generic-wal.html
[V-VEC]: ../../../../src/sdk/search/vector.rs
[V-IDX]: ../../../../src/index/mod.rs
[V-EXP]: ../../../../src/sdk/search/explain.rs
[V-QUANT]: ../../../../src/vector/quantization.rs
[V-MAINT]: ../../../../src/storage/engine/maintenance.rs
[V-SER]: ../../../../src/sdk/serialization/mod.rs
[V-ADMIN]: ../../../../src/sdk/api/admin.rs
[V-REBUILD]: ../../../../src/sdk/serialization/impl_rebuild.rs
[V-LOCK]: ../../../../src/storage/engine/mod.rs
[V-PART]: ../../../../src/storage/engine/partition.rs
[V-CLI]: ../../../../src/cli_handlers/index.rs
[V-MEM]: ../../../../src/sdk/api/memory.rs
[V-DEL]: ../../../../src/storage/engine/delete.rs
[V-OPS]: ../../../../src/storage/engine/ops.rs
[V-ATTEST]: ../../../../src/attestation.rs
[V-WAL]: ../../../../src/wal.rs
[V-REPORT]: ../../../../src/sdk/types/search.rs
[V-INSERT]: ../../../../src/storage/engine/insert.rs
