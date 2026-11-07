---
title: "Faiss: arquitectura, identidad vectorial, filtros y reapertura frente a VantaDB"
kind: research
status: proposed
description: "Investigación estática profunda de Faiss con flujos, incidencias públicas, runners ANN y candidatos de integración sin sustituir el motor"
tags: [research, memory, engineering]
---

# Faiss: arquitectura, identidad vectorial, filtros y reapertura frente a VantaDB

## Índice

- [0. Identidad, arquitectura y flujos](#0-identidad-arquitectura-y-flujos)
- [1. Recomendaciones priorizadas](#1-recomendaciones-priorizadas)
- [2. Fichas de adaptación](#2-fichas-de-adaptación)
- [3. Incidencias públicas y seguridad](#3-incidencias-públicas-y-seguridad)
- [4. Prerrequisitos específicos](#4-prerrequisitos-específicos)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Advertencias metodológicas](#6-advertencias-metodológicas)
- [7. Benchmarks leídos](#7-benchmarks-leídos)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Negativos revalidados](#9-negativos-revalidados)
- [10. Escritorio y administración](#10-escritorio-y-administración)
- [Fuentes, cobertura y cierre](#fuentes-cobertura-y-cierre)

## 0. Identidad, arquitectura y flujos

**Dictamen:** Faiss es una biblioteca de índices y cómputo vectorial, no una memoria de agentes, una base transaccional ni una aplicación administrativa. El valor incremental reside en contratos de representación, identidad, compatibilidad de composiciones y explicación del trabajo efectivo. No hay evidencia comparativa ejecutada que justifique sustituir HNSW, IVF, Flat o cuantización de VantaDB.

| Dato | Evidencia y alcance |
|---|---|
| Repositorio/local | `facebookresearch/faiss`, `C:\Users\Eros\faiss` |
| Snapshot conservado | `277b3c76b7a1507915eb8f5010dcb367cbc95494`, `main`; SHA local releído el 2026-11-07 |
| Captura original | 41.261 estrellas; metadata 2026-11-07T06:56:17.3058811Z en [manifiesto](clone-manifest.json); no actualizada |
| Clon | Shallow según manifiesto; historia/tags y materialización LFS/submódulos no certificados por esta lectura |
| Licencia | [LICENSE:1-21](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/LICENSE#L1-L21), MIT, leído; no equivale a inventario legal de todos los componentes GPU/dependencias |
| Comparación propia | VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, `develop`, limpio al inicio; no se usa c561158 como prueba actual |
| Ejecución | Ninguna app, test, benchmark, instalación, build o servicio; sólo lecturas y documentos |

Se leyeron [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [METHOD](METHOD.md), [BENCHMARK-NOTES](BENCHMARK-NOTES.md) y el [estudio OpenViking](../../Backlog-OpenViking-VantaDB.md). Sus exigencias se aplican aquí; sus defectos históricos no se importan.

### 0.1 Mapa y modelo

```text
faiss/Index.h (API C++) ← bindings Python / c_api
  IndexFlat                    códigos contiguos; búsqueda exhaustiva
  IndexIVF                     coarse quantizer → listas (id, código)
    IndexIVFFlat / IVFPQ        almacenamiento original / residual codificado
  IndexHNSW                    grafo + storage + DistanceComputer
  IndexIDMap / IndexIDMap2      identidad externa ↔ posición; mapa inverso
  impl/ProductQuantizer        subespacios, centroides, códigos entrenados
  impl/index_{read,write}       imagen de índice; ownership y flags por tipo
benchs/ + contrib/datasets     evaluación de motor; no answerer ni juez LLM
```

La API conserva `d`, `ntotal`, `metric_type`, `is_trained`; `idx_t` identifica resultados, pero **su significado depende del índice**. En Flat la posición es implícita; IVF almacena identificadores en listas; IDMap añade otra traducción. `SearchParameters` permite selector y parámetros especializados por llamada: [Index.h:69-171](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/Index.h#L69-L171). No existe en ese contrato una sesión conversacional, revisión de documento o autorización de tenant.

Stack: C++/CMake, bindings Python, kernels CPU/OpenMP/SIMD/BLAS y backends GPU opcionales. Este informe inspecciona CPU, no certifica equivalencia GPU. El integrador posee los buffers y la sincronización de escrituras; que una operación termine no crea una transacción de aplicación. `own_fields`, `own_invlists` y los lectores raw/`unique_ptr` son fronteras de ownership, no ACL.

### 0.2 Flujo real A — entrenar, añadir, consultar con filtro

1. Construir IVF exige dimensión compatible con el coarse quantizer. `is_trained` refleja si éste contiene `nlist` centroides: [IndexIVF.cpp:288-309](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L288-L309).
2. `train` llama `train_q1`, submuestrea opcionalmente y entrena el encoder con residuales o vectores directos. Sólo después establece `is_trained=true`: [1543-1588](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L1543-L1588). Entrenar centroides y codificador son trabajos diferentes; un Flat no necesita ese ciclo.
3. `add_with_ids` asigna cada vector al coarse quantizer y delega `add_core`; el código precodificado incluye número de lista, y su ID puede venir del caller: [314-337](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L314-L337).
4. `search_preassigned` exige entrenamiento, listas y `k>0`; acota `nprobe` por `nlist`, toma `max_codes` y selector. Rechaza selector junto a `store_pairs`, iteradores con determinadas opciones y modos paralelos incompatibles: [529-615](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L529-L615).
5. El selector comprueba IDs candidatos, **no permisos**. Un rango sólo utiliza el atajo de IDs ordenados cuando corresponde; afirmar `assume_sorted` sin preservar el orden rompe la premisa del optimizador.
6. El presupuesto de códigos y las listas visitadas delimitan el trabajo; no garantizan recuperar todos los vecinos permitidos ni llenar k. `ensure_topk_full` ajusta el presupuesto efectivo, no convierte ANN en búsqueda exacta.

No hay frontera de commit durable en esa secuencia. Un entrenamiento o add fallido no tiene en la API observada un rollback multioperación documentado; el caller debe conservar un índice publicable anterior o reconstruir. Cancelación/interrupción y excepciones OpenMP requieren revisar el backend concreto, no asumir transacciones por usar C++.

En este SHA, `search_preassigned` captura excepciones dentro de regiones
OpenMP, marca `interrupt` y relanza fuera de la región; incluye
`InterruptCallback::check`:
[IndexIVF.cpp:789-887](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L789-L887).
No se deben consumir buffers parcialmente escritos como resultados completos
si la llamada falla. Esto acredita contención en esa consulta CPU, no
cancelación uniforme de train/add/GPU ni rollback de actualizaciones.

### 0.3 Flujo real B — actualizar o borrar sin confundir identidad y posición

`IndexFlatCodes::remove_ids` compacta los códigos restantes hacia posiciones menores: [IndexFlatCodes.cpp:61-80](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexFlatCodes.cpp#L61-L80). Por tanto, posición no es identidad estable. IDMap conserva una tabla externa y también la compacta; al buscar traduce etiquetas con comprobación de rango: [IndexIDMap.cpp:192-225](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIDMap.cpp#L192-L225), [275-295](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIDMap.cpp#L275-L295).

IVF elimina por `DirectMap` y reduce `ntotal`. Con mapa hash, `update_vectors` hace remove→comprobación de cantidad→add con los mismos IDs; si no encontró todas las entradas, falla después del remove: [IndexIVF.cpp:1385-1410](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L1385-L1410). Es una API de mantenimiento, **no reemplazo atómico de memoria**. El camino Array es diferente; no se extrapola este orden a todos los índices. Tampoco se promete que HNSW admita las mismas operaciones que IVF.

El error práctico de envolver IVF con un mapa pensado para posiciones compactadas está reportado públicamente en §3. Una capa extra que “unifica IDs” puede empeorar el contrato si desconoce la estabilidad del índice subyacente.

### 0.4 Flujo real C — guardar, reabrir y combinar

Los lectores devuelven objeto owning (`read_index_up`) o raw cuyo borrado compete al caller. Los flags de mmap/read-only/skip no son universales: [index_io.h:18-81](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/index_io.h#L18-L81). `SKIP_STORAGE` no produce automáticamente un índice consultable; omitir tablas PQ puede impedir operaciones necesarias para construir el grafo.

El lector HNSW valida niveles, offsets, vecinos y entry point; se inspeccionaron particularmente [index_read.cpp:1190-1260](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/impl/index_read.cpp#L1190-L1260). Es falso que no tenga validación. Esa validación tampoco acredita límites completos de asignación antes de leer cualquier tipo.

Al fusionar IVF se comprueban tipo, dimensión, listas, tamaño de código y compatibilidad de coarse quantizers; el chequeo costoso compara centroides y la fusión con direct maps se rechaza: [IndexIVF.cpp:1592-1626](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L1592-L1626). Igual tamaño binario no implica igual entrenamiento. Ninguno de estos métodos sustituye fsync, WAL, publicación atómica o recuperación de VantaDB.

## 1. Recomendaciones priorizadas

| Orden | Decisión | Valor específico |
|---|---|---|
| Primero | Investigar revisión de caches derivadas y coherencia al guardar/reabrir | Hay un negativo estático propio acotado (§9); no es una propuesta de ANN nuevo |
| Primero | Documentar identidad, score y capacidades de cada método | Los issues de composición muestran consecuencias observables, no sólo limpieza de tipos |
| Después | Extender diagnóstico de filtro y terminación | Distinguir “no existe vecino” de “se agotó presupuesto” sin prometer recall |
| Condicional | Robustecer admisión de imágenes y compatibilidad de derivados entrenados | Sólo añadir validaciones no presentes; Vanta ya tiene header/versiones y límites de lectura |
| No aprobado | PQ nuevo, GPU, autotuning o reemplazo de algoritmo | Coste de mantenimiento/FFI y mejora de memoria no demostrados |

## 2. Fichas de adaptación

Las letras son referencias locales, **no IDs de backlog**. Todos los nombres Rust de contratos siguientes son hipotéticos. Se extienden seams reales; no se crea un registro, scheduler o arnés paralelo.

### A. Generación de caches vectoriales derivadas

- **Problema:** una memoria actualiza el vector manteniendo ID y cardinalidad; un derivado lazy puede seguir representando el contenido anterior. La unidad upstream útil es separar entrenamiento/almacenamiento y comprobar compatibilidad, no copiar `IndexIVF`.
- **Delta real:** Vanta tiene IVF y SCANN. `search_ivf` reconstruye al cambiar `nodes.len()`; `validate_node` actualiza el vector de un ID existente sin cambiar esa longitud (§9). Extender invalidación, no “añadir IVF”.
- **Contrato propuesto:** `DerivedGeneration { source_epoch, metric, dimension, representation }`; adquirir una generación produce vista inmutable válida para una revisión. Un resultado de construcción para una época vieja no puede publicarse como actual. Error explícito `StaleGeneration` si no puede usarse/reconstruirse, nunca éxito con índice equivocado.
- **Seam:** `CPIndex::search_ivf/search_scann` en `src/index/search/alternate.rs:16-95`, actualización `src/index/graph/core.rs:321-374`, serialización `src/index/serialize/bytes.rs:224-239`. Son tres fronteras del mismo derivado, no tres features.
- **Alternativas:** invalidar bajo lock y reconstruir lazy (mínimo); epoch monotónico si las mutaciones lo necesitan; adapter externo no ve todas las escrituras y no basta. Reentrenar todo en cada query es coste oculto inaceptable.
- **Pasos:** inventariar mutaciones de vector, bitset, remove y bulk; fijar autoridad de revisión; invalidar al mutar con el orden de locks existente; construir sobre snapshot coherente; publicar sólo si revisión coincide; impedir serializar cache vieja como vigente; cubrir SCANN bajo el mismo principio sin mezclar sus codecs.
- **Persistencia/migración/rollback:** inicialmente cache descartable, sin cambiar registros canónicos. Leer versiones antiguas descartando derivado no verificable y reconstruir. Si se persiste epoch, requiere versión de formato; un downgrade debe poder ignorar derivado, no interpretar bytes nuevos. Desactivar cache conserva lectura canónica y no revierte memorias escritas.
- **Dependencias:** ownership de snapshot y jerarquía de locks; no depende de checkpoints de agente, LLM ni ledger OpenViking.
- **FMEA:** update entre snapshot/publicación→cache obsoleta→detectar epoch distinto y descartar; lock tomado en orden inverso→deadlock→revisión del orden y construcción fuera de publicación; rebuild fallido→no destruir generación anterior válida para lectores ya iniciados; delete+add con misma longitud→no usar cardinalidad como revisión.
- **Aceptación:** mismo ID/vector nuevo, bitset nuevo, remove+add cardinalidad constante, serializar/reabrir después de mutar y dos rebuilds concurrentes no publican estado viejo. Validación dinámica pendiente; no se ha reproducido corrupción de disco.
- **Observabilidad/mínimo:** epoch fuente/cache, motivo de invalidación, construcción descartada, duración y error agregado; nunca vectores ni payloads. Fuera de alcance: codec nuevo y autotuning.

### B. Semántica de score y ruta efectiva

- **Problema:** un agente o UI trata distancia como probabilidad, o “exact” sobre un codec con pérdida como exactitud respecto al vector original. Faiss Flat separa L2 cuadrada/IP: [IndexFlat.cpp:26-59](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexFlat.cpp#L26-L59).
- **Delta/seam:** Vanta ya transforma Euclidean a distancia negativa con raíz en `exact_scan` (`src/sdk/search/vector.rs:67-112`) y tiene `SearchExplanation`. Proponer `ScoreSemantics { metric, ordering, distance_transform, representation, effective_method }` en explicación existente, no otro ranking.
- **Alternativas:** sólo documentación SDK si la semántica es única; metadato opcional para rutas distintas; prohibido corregir “a ojo” los scores del motor para imitar Faiss.
- **Invariantes/errores:** explicación y búsqueda ordinaria conservan IDs y orden; campo desconocido se comunica como desconocido. IVF/SCANN metric-bound ya rechazan override incompatible (`alternate.rs:107-145`); no reemplazar ese error por conversión ficticia.
- **Pasos:** inventariar endpoints/bindings; contrastar exact/ANN y representación; ampliar DTO compatible; mapear HTTP/native/WASM y Studio; documentar diferencia entre distancia de motor y RRF.
- **Migración/rollback/dependencias:** campos opcionales sin reindexación; lectores viejos ignoran extensión, apagar explicación conserva búsqueda. Depende de DTO estable y autoridad del método; no de A salvo mostrar generación.
- **FMEA:** L2² etiquetada L2→umbrales mal interpretados→comparar con caso geométrico conocido; IP llamada cosine sin normalización→contrato falseado→describir métrica real; diagnóstico altera ranking→comparación de paridad.
- **Aceptación/observabilidad:** consultas degeneradas, empty, empates y errores de dimensión mantienen contrato; contador por método/transformación, sin query cruda. Mínimo: trazabilidad, no calibración, pesos, thresholds ni promesa de mejora.

### C. Identidad estable y capacidades de mutación

- **Problema:** compactar posiciones no debe desplazar la identidad de recuerdos supervivientes. Upstream: Flat compacta, IVF usa IDs, IDMap traduce; composición incorrecta produce §3.
- **Delta/seam:** Vanta ya usa `u128` y `(namespace,key)`; `VecIndex::add` devuelve error de capacidad (`src/index/mod.rs:49-61`), IVF construido es read-only en ese contrato. No introducir IDMap ni reducir IDs a `i64`.
- **Contrato propuesto:** descriptor de capacidad por índice (`MutationSupport`) y resultado del mantenimiento con identidad/revisión canónica; unsupported es error tipado, nunca operación ignorada. Posición interna/offset no sale como ID persistente.
- **Pasos/alternativas:** extender port existente si falta información; auditar remove/update/bulk/rebuild y sus adapters; preservar mapping original en compactación; publicar índice tras validar referencias. Una fachada externa no puede prometer atomicidad que el backend no ofrece.
- **Persistencia/dependencias:** sin migración de IDs; cambios de layout sólo con rebuild desde registros y publicación compatible. Rollback = generación anterior + replay canónico según garantías reales; no “deshacer” deletes ya confirmados. Depende de durabilidad y, para caches, A.
- **FMEA:** retry de delete alcanza identidad reciclada→pérdida ajena→guard id/revisión; mezcla de mapa posicional y estable→hits reasignados→comparar IDs antes/después; partial update→old/new mezclados→resultado incierto y reconciliación, no éxito.
- **Aceptación/observabilidad/mínimo:** borrar primer/medio/último, repetir borrado, actualizar superviviente, reabrir y reconstruir preservan identidad. Contadores de rechazadas/obsoletas/reconciliadas sin IDs sensibles por defecto. No objetivo: imponer delete+add de Faiss al store de memoria.

### D. Admisión y compatibilidad de representación

- **Problema:** imagen o código de dimensión/tamaño compatible puede tener entrenamiento incompatible. `ProductQuantizer::set_derived_values` exige M positivo, dimensión divisible, limita bits y utiliza multiplicación protegida: [ProductQuantizer.cpp:58-78](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/impl/ProductQuantizer.cpp#L58-L78).
- **Delta/seam:** Vanta tiene cuantización y formatos, no hay que añadir PQ. El lector ya valida `VNDX`, versión, imagen cifrada y lecturas truncadas: `src/index/serialize/bytes.rs:265-348`. Extender sólo invariantes faltantes en import/reopen y `src/vector/quantization.rs`, no duplicar header.
- **Contrato propuesto:** `EncodingCompatibility` calculado a partir del formato vigente; dimensión, métrica, codec/version y, sólo si existe entrenamiento, identidad de parámetros. Fallo antes de publicar/asignar estructuras incompatibles; ninguna credencial/ACL se deduce del fingerprint.
- **Pasos:** inventario de codecs reales; identificar campos ya persistidos; validar tamaños/overflow antes de reservas; comprobar relaciones grafo→storage; separar corrupción de falta de clave; revisión legal si se reutiliza material.
- **Migración/rollback:** sin nuevo campo si basta validar; si hace falta descriptor, versionado explícito y reconstrucción desde originales. No reutilizar código comprimido con codebook distinto. Mantener imagen anterior hasta validar apertura; no eliminar datos fuente por fallo de índice.
- **Dependencias/FMEA:** A para generación persistida; metadatos mentirosos→OOM o score erróneo→límites del lector y rechazo; checksum correcto no prueba autenticidad→mantener frontera de confianza; mmap de fichero reemplazado→lifetime ownership, no punteros escapados.
- **Aceptación/observabilidad:** truncado, longitud imposible, dimensión/codec incompatible y clave ausente distinguidos; reopen correcto conserva resultados y significado. Conteos por error/versiones; sin bytes del índice. No objetivo: certificar todos los formatos Faiss o nuevos esquemas de cifrado.

### E. Diagnóstico de filtro, presupuesto y terminación

- **Problema:** pocos hits puede significar universo pequeño, filtro, presupuesto o ANN incompleto. Upstream separa `nprobe/max_codes/selector`, y el runner contabiliza `-1`; no equivale a inexistencia semántica.
- **Delta/seam:** Vanta ya selecciona Pre/In/PostFilter, adapta búsqueda y valida filtros exactos después del bitset (`src/sdk/search/vector.rs:153-280`). Añadir sólo razón de terminación a explicación/perfil existente.
- **Contrato hipotético:** `SearchCompletion { requested, returned, exhausted: Unknown|CandidateSet|Budget, effective_method }`; “exhausted” debe acreditarse por el algoritmo, no inferirse de `len<k`. El filtro de seguridad no puede relajarse para llenar k.
- **Pasos:** seguir señales existentes de perfil; conservar autorización/namespace y filtro exacto; propagar presupuesto realmente usado; distinguir cero hits de no ejecutado; conectar Studio read-only.
- **Alternativas/migración:** diagnóstico sin cambio de búsqueda primero; ampliar cursor/resume sólo si existe necesidad demostrada. Campos opcionales, sin reindexación, rollback ignora diagnóstico. Depende de B; no de un nuevo autotuner.
- **FMEA:** aproximación Bloom tratada como autorización→fuga→recheck exacto obligatorio; “budget” sin instrumentación→mentira operativa→Unknown; intentar completar k ilimitadamente→DoS→mantener presupuesto existente.
- **Aceptación/observabilidad:** filtros vacíos, namespace sin datos, selectividad alta, presupuesto agotado y k mayor que población; no hits ajenos. Emitir conteos y razones, nunca filtros privados. Mínimo: explicar límites, no cambiar ANN por principio.

### 2.1 Precisiones de contratos, ownership y cancelación A–E

Los contratos anteriores son **diseño propuesto, no APIs implementadas**. Para
que una futura spec no confunda un descriptor con una operación:

| Ficha | Entrada → salida propuesta | Error y frontera de publicación | Cancelación y ownership |
|---|---|---|---|
| A | Snapshot de nodos, revisión fuente, métrica/dimensión/representación → vista de derivado con generación y población cubierta | Revisión distinta al publicar → descartar construcción; error al construir no valida cache vieja para una consulta nueva | El builder posee sus buffers; lectores retienen su vista. Cancelar un rebuild descarta sólo el derivado no publicado, no revierte escrituras canónicas |
| B | Método efectivo y score ya producido por el motor → descriptor de semántica adjunto a explicación | Semántica desconocida → dato desconocido, no conversión supuesta ni score cero | No inicia trabajo ANN adicional ni mantiene buffers de consulta después de terminar; cancelar explicación no confirma cancelación de búsqueda |
| C | Operación, ID canónico y revisión esperada cuando el backend la ofrezca → capacidad/resultado de mantenimiento | Unsupported antes de mutar; estado incierto si el backend no garantiza completar todo | El dueño del índice excluye escrituras incompatibles; perder el waiter no es undo. No prometer compare-and-swap donde el port actual no lo ofrece |
| D | Imagen autorizada, límites del lector y descriptor esperado → índice validado listo para publicar | Corrupción, clave ausente e incompatibilidad permanecen categorías distintas | El lector posee imagen/mmap mientras viva la vista. Cancelar apertura antes de publicar libera recursos temporales, no borra el primario |
| E | Señales efectivas de búsqueda, cantidades y presupuesto existente → motivo de terminación conocido o Unknown | No deducir agotamiento exacto desde resultados truncados; propagar error operativo original | El reporte no prolonga búsqueda ni relaja filtros al cancelar; no retener payload o selector prestado más allá de su lifetime |

**Unidad upstream de compatibilidad:** `IndexIVF::check_compatible_for_merge`
comprueba dimensión, listas, tamaño de código, tipo concreto y ausencia de
DirectMap; con comprobación costosa habilitada compara centroides. Después
`merge_from` mueve listas y deja `other.ntotal=0`:
[IndexIVF.cpp:1594-1626](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L1594-L1626).
Es evidencia para D y para exigir ownership explícito en C, **no** una
implementación Faiss de epochs de A. Tampoco demuestra equivalencia de todos
los codebooks derivados de clases hijas desde ese bloque de base.

**Población no es sólo cardinalidad:** en Vanta, `IvfIndex::build_with_dimension`
descarta vectores vacíos y dimensiones incompatibles
([ivf.rs:104-150](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/ivf.rs#L104-L150));
SCANN lazy añade sólo `VectorRepresentations::Full` y registra rechazos de add
([alternate.rs:66-83](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/search/alternate.rs#L66-L83)).
El contador `nodes.len()` no acredita que todos sean vecinos elegibles.
A/E deben distinguir población fuente, elegible y efectivamente indexada.
Esto describe ramas leídas, no acredita que una configuración productiva
mezcle representaciones ni que se pierdan recuerdos por esa vía.

**FMEA adicional acotado:** equiparar población fuente e indexada puede
convertir exclusiones por representación en “no hay recuerdos”. Detección
propuesta: contadores agregados de elegibles/excluidos y motivo, sin IDs;
mitigación: admitir sólo capacidades compatibles o indicar parcialidad,
nunca descodificar silenciosamente con otro codebook. La aceptación manual
debe cubrir esas ramas además del update de cardinalidad constante de §9.

## 3. Incidencias públicas y seguridad

Consultas públicas realizadas el **2026-11-07 UTC**, sin ejecutar reproducciones.
Se leyeron cuerpo, estado y comentarios por la API de GitHub; el estado es
el observado ese día, no una garantía para revisiones futuras.

| Incidencia leída | Estado y fechas comprobadas | Reporte, explicación y límite |
|---|---|---|
| [#4656 — Mapping gets out of sync when deleting an ID from IVF + PQ with an IndexIDMap2 wrapper](https://github.com/facebookresearch/faiss/issues/4656) | Closed/completed; creada 2025-11-03, cerrada 2025-11-06 06:20:41Z; comentario posterior 2025-11-07 | Reporta Faiss 1.11.0/C++17/Raspberry Pi4: tras eliminar ID 1, los mismos scores aparecen asociados a IDs 2/12 en vez de 1/11. El autor elimina el wrapper IDMap2 y comunica desaparición del problema; un contributor confirma la interpretación y remite a #4535. No se leyó un PR que acredite reparación de todas las composiciones. |
| [#4655 — assertion `j == index->ntotal` al borrar IVF+PQ con IndexIDMap2](https://github.com/facebookresearch/faiss/issues/4655) | Closed/completed; creada 2025-11-03, cerrada 2025-11-06 06:21:04Z | Misma versión y composición: primer delete parece funcionar y el segundo aborta. El comentario de cierre también prescribe quitar el wrapper redundante, no anuncia transacción ni corrección general de remove. Es un segundo síntoma del mismo mecanismo, no dos oportunidades independientes. |

Se releyeron ambos cuerpos y sus comentarios mediante `websearch.fetchWebContent`
sobre los endpoints públicos `/repos/facebookresearch/faiss/issues/{4655,4656}`
y `/comments`. Precisión sobre la tabla: el comentario que confirma la
interpretación de #4656 figura como **CONTRIBUTOR** en la API; no acredita
por sí solo un rol de mantenedor.

Se amplió la remisión a
[#4535 — Removing from IndexIVFFlat with a IndexIDMap2 wrapper fails](https://github.com/facebookresearch/faiss/issues/4535):
**cuerpo efectivamente leído**, Python 3.12.5 / faiss-cpu 1.11.0.post1,
dimensión 384 y wrapper sobre IVFFlat; el autor reporta assertion después de
varios borrados pese a pasar comprobaciones de cardinalidad. Refuerza C:
verificar `ntotal` no valida el mapping. No aporta otra feature ni prueba
actual de todos los wrappers. **Estado, fechas y comentarios no acreditados
en esta lectura**: el endpoint API devolvió `API rate limit exceeded`;
fallback a la página GitHub con extracción de contenido permitió leer el
cuerpo, no certificar su resolución. No se afirma merge ni reparación.

**Mecanismo contrastado con el SHA:** Flat compacta posiciones (§0.3);
`IndexIDMapTemplate::remove_ids` compacta su tabla
([IndexIDMap.cpp:275-295](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIDMap.cpp#L275-L295)).
IVF ya mantiene identificadores. La etiqueta “todo Index tiene remove_ids”
no demuestra que un wrapper posicional sea válido para cada tipo.
El SHA incorpora defensas de rango al traducir resultados; devolver `-1`
ante un offset inválido evita un acceso concreto, pero no demuestra identidad
correcta de todos los resultados que siguen dentro de rango.

**Aplicabilidad:** C pide contratos de identidad/capacidad en Vanta, no
presume que Vanta use IndexIDMap ni reproduzca ese fallo. La discriminación
manual futura es comparar IDs canónicos supervivientes antes/después de borrar,
no aceptar únicamente ausencia de crash o cardinalidad correcta. La incidencia
no invalida las cifras ANN sin mutaciones de Faiss ni prueba corrupción del WAL.

La [FAQ oficial, sección concurrencia](https://github.com/facebookresearch/faiss/wiki/FAQ#can-searches-and-additions-be-performed-concurrently-from-several-threads)
se leyó también por su [Markdown público](https://raw.githubusercontent.com/wiki/facebookresearch/faiss/FAQ.md):
búsquedas CPU concurrentes admitidas; search/add y add/add requieren
sincronización del caller; no trasladar esa afirmación a GPU. Es documentación
viva consultada el 2026-11-07, **no código del SHA congelado**.
También explica ownership del coarse quantizer (`own_fields`) e IDs int64.
Un binding Rust no puede declarar `Send/Sync` por similitud de firmas:
debe encapsular lifetime de buffers y excluir mutación durante lectores.

### Fronteras de confianza

- **Tenant y filtros:** `IDSelector` determina candidatos, no autoriza. El wrapper
  de aplicación debe aplicar namespace/ACL también al recuperar payload y
  explicar resultados; nunca rellenar k con vecinos fuera del scope.
- **Imágenes:** leer una imagen es cruzar una frontera binaria. Las validaciones
  HNSW leídas en `index_read.cpp` descartan algunos estados corruptos; no son
  una auditoría de todos los codecs, reservas y offsets. D propone rechazo
  previo a publicación; no afirmar aquí explotación remota reproducida.
- **Contenido hostil/LLM:** Faiss no interpreta instrucciones ni ejecuta prompts.
  Que un vector represente texto hostil no lo vuelve confiable; la política de
  inyección de Vanta sigue actuando después del retrieval.
- **Rutas, red y secretos:** no se encontró servidor HTTP en la API de biblioteca
  inspeccionada. El riesgo de abrir un path o cargar una imagen arbitraria
  pertenece al integrador; no atribuir SSRF a `Index::search`.
- **Borrado:** eliminar entrada no certifica borrado de fichero anterior, copia
  GPU, checkpoint de aplicación o respaldo. No usar “removed=1” como evidencia
  de borrado legal integral.

## 4. Prerrequisitos específicos

Las fichas no están bloqueadas por una migración universal:

| Candidata | Contrato previo que se debe comprobar | Qué puede avanzar sin él |
|---|---|---|
| A | Autoridad de revisión de `CPIndex`, orden de locks y snapshot consistente de nodos | Documentar invalidación y caso adverso; no publicar una cache nueva |
| B | Una semántica por score y DTO de explicación versionable | Documentación de diferencias L2/L2²/IP y rutas ya existentes |
| C | ID/revisión canónicos, operaciones realmente soportadas y respuesta ante resultado parcial | Describir capacidad; no cambiar IDs persistidos |
| D | Límites y versiones del lector existente; fuente canónica reconstruible | Rechazar combinaciones incompatibles ya detectables, sin inventar un formato |
| E | Señal real de terminación y filtros exactos por namespace | Exponer `Unknown`, no prometer exhaustividad |

En Vanta `vector_memory_search` ya hace recheck de namespace y filtros después
del bitset y resolución de nodos
([vector.rs:252-280, snapshot de referencia](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L252-L280)).
La representación comprimida, memoria canónica y grafo no tienen por ello una
transacción demostrada en todos los callers. A necesita trazar esa publicación,
no fusionar este estudio con el helper RMW de checkpoints de agente.

## 5. Qué no trasladar

1. **Un motor alrededor de Faiss:** añade FFI, memoria nativa, compilación
   C++/BLAS y quizá GPU sin resolver durabilidad, namespace ni gobernanza.
   Alternativa preferida: extender contratos del motor Rust existente.
2. **Delete→add como upsert transaccional:** la ruta hash de `update_vectors`
   puede fallar después de borrar. Vanta necesita conservar su propia frontera
   de commit; una llamada C++ exitosa no la define.
3. **Autotuning en consultas certificadoras:** el runner busca parámetros usando
   ground truth. Eso puede servir como exploración de desarrollo, no como
   evidencia holdout independiente sin la separación de METHOD.
4. **Nuevos HNSW/Flat/IVF/PQ por catálogo:** ya existen índices y cuantización.
   Codec, GPU, OPQ, additive/residual quantizers y kernels especializados sólo
   serían candidatos después de un perfil real y revisión de sus fuentes, no
   por estar disponibles en `index_factory`.
5. **Dedup de vector como dedup de recuerdos:** igualdad geométrica no implica
   mismo autor, fecha, procedencia o permiso. Vanta ya tiene dedup/supersession.
6. **Distancia como confianza factual:** aproximación, compresión, ranking y
   evidencia para responder son capas diferentes.

## 6. Advertencias metodológicas

Los resultados dependen de representación, métrica, entrenamiento y modo batch.
La FAQ describe problemas de precisión al expandir L2 con normas/dot products;
“Flat exacto” significa exhaustividad del recorrido sobre su representación,
no aritmética real infinita ni empate ordenado por una clave canónica.
No se ha medido el efecto de esos errores en embeddings de Vanta.

El snapshot incluye variantes recientes y composiciones opcionales; ni nombre
de clase ni versión del README acredita que todos los bindings empaquetados
expongan la misma API. El clon shallow tampoco fecha la introducción del código.
Los tres cuerpos de issues son reportes de la familia 1.11.0, no reproducciones del SHA estudiado; sólo #4655/#4656 tienen estado y comentarios comprobados aquí.
Las unidades agente/sesión/turno no existen en los runners ANN siguientes.

## 7. Benchmarks leídos

### 7.1 `bench_hnsw.py`: inclusión del primer vecino en un batch

Fuente leída:
[runner:18-134](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_hnsw.py#L18-L134)
y [DatasetSIFT1M](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/contrib/datasets.py#L147-L190).
`k` es argumento posicional obligatorio; sin lista de trabajos prueba varias
familias, no sólo HNSW. El dataset declara d=128, train=100.000,
base=1.000.000 y queries=10.000; carga ground truth desde datos externos.
Los archivos no se descargaron ni se certificó su presencia.

- `evaluate` llama una vez a `index.search(xq,k)` y divide tiempo de batch por
  `nq`. No calcula p95 ni latencia de una petición individual.
- `recall_at_1 = (I == gt[:,:1]).sum()/nq`: comprueba si aparece el vecino
  exacto número uno **en cualquiera de las k posiciones**. No es intersección
  de los k vecinos gold ni el `recall_all` de sesiones de LongMemEval.
- `missing_rate` divide etiquetas `-1` por `k*nq`. Conserva huecos; una excepción
  no se transforma en un resultado malo de calidad: interrumpe el script.
- HNSWFlat usa M=32, efConstruction=40 y recorre efSearch 16/32/64/128/256 con
  cola bounded/unbounded; IVF baseline usa 16.384 listas y varios nprobe.
  Son defaults/valores **upstream**, no propuestas para Vanta.
- El ajuste a un hilo está comentado. Concurrencia efectiva depende del runtime
  OpenMP; no atribuir un número de cores. No hay warmup separado en `evaluate`,
  answerer, juez, tokenizer ni coste por API de embeddings.
- Entrenamiento/add/carga quedan fuera de la ventana de search; sí consumen
  RAM/CPU antes de ella. No se observa reset de un servicio compartido.

**Corrección de alcance del dataset:** el entry point
[bench_hnsw.py:9-12](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_hnsw.py#L9-L12)
intenta primero `faiss.contrib.datasets_fb`, y sólo ante `ImportError` utiliza
el dataset OSS descrito arriba. Las cardinalidades y rutas SIFT acreditan
**esa implementación OSS**, no cualquier entorno interno donde importe la
primera. Los barridos reutilizan índice, matrices y proceso; sin warmup
separado ni limpieza de caches, las mediciones sucesivas no son arranques
fríos independientes. No se modifican runner ni protocolo de comparación.

### 7.2 `bench_all_ivf`: denominadores y selección de parámetros

[eval_setting:65-98](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_all_ivf/bench_all_ivf.py#L65-L98)
repite batches hasta superar `min_time`, informa tiempo/(nq*nrun), y contabiliza
distancias `indexIVF_stats.ndis/nrun`. Sólo la última matriz I alimenta calidad.
Con `--inter`, usa `eval_intersection(gt[:,:k],I[:,:k])/(nq*k)`;
sin él, informa inclusión del primer gold en prefijos 1/10/100, con denominador
`nq`. No mezclar estas dos curvas bajo la etiqueta única Recall@k.

[CLI:423-594](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_all_ivf/bench_all_ivf.py#L423-L594)
declara deep1M, HNSW32, k=100, parámetros `autotune`, 500 experimentos,
min_test_duration=3s, nthreads/searchthreads=-1 y `todo=check_files`.
Los paths de índices son opcionales y el estado de ficheros decide trabajo;
reusar una imagen previa no mide coste de construcción desde cero.
[run_train:164-183](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_all_ivf/bench_all_ivf.py#L164-L183)
deduce maxtrain y puede entrenar sobre base si el dataset no tiene train;
comprueba finitud, no independencia semántica de las sesiones de un agente.

[datasets_oss.py:9-23](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_all_ivf/datasets_oss.py#L9-L23)
fuerza una ruta `/checkpoint/matthijs/simsearch/` y `float32` contiguo.
El script prefiere `datasets_fb` si existe: registrar implementación realmente
importada es necesario para no atribuir al OSS datos del entorno interno.
[Autotune:355-396](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_all_ivf/bench_all_ivf.py#L355-L396)
entrega ground truth al criterio y reevalúa operating points escogidos.
No se observó un holdout independiente en ese bloque.

No hay abstenciones de QA, preguntas descartadas por un juez ni coste LLM;
los errores de carga/search no forman una población QA alternativa. Datos,
semillas efectivas de todos los algoritmos, BLAS, afinidad y caches deben
registrarse antes de reproducir; no se los infiere de una cifra publicada.
No se ejecuta ningún comando del runner en esta investigación.

### 7.3 Adaptación a Vanta sin un harness nuevo

Aplicar [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md): éste es el carril **motor ANN**.
Un adapter sobre bindings existentes traduciría matrices f32, métrica y ordinal
de dataset a ID canónico de Vanta, devolviendo distancias/IDs y errores sin
recalibrar el gold. La conversión de L2² a la semántica Vanta debe ser explícita;
calidad se compara por identidad, no igualando números de score.

Entrenamiento sólo donde el backend lo requiere; no atribuir ese coste a Flat.
Separar importación/indexación/search/reapertura; preservar filtros, cardinalidad
y recursos comparables. Los parámetros seleccionados con gold pertenecen a
desarrollo según METHOD. Este dossier no fija nuevas muestras, thresholds,
prompts, rúbricas, pesos ni configuraciones. Un wrapper de `faiss.index_factory`
no es portable directamente al SDK de memoria, y una curva ANN no acredita
mejor retención, respuesta o éxito de agente.

## 8. Matriz frente a VantaDB

Estados se refieren al snapshot **97c9623c8ef5a70fdb51aff215bfbb3b05b15612**.
“No acreditado” limita esta inspección; no significa ausencia en todo el repo.

| Capacidad | Faiss acreditado | VantaDB | Decisión y evidencia |
|---|---|---|---|
| HNSW, Flat, IVF | Existe, familias con contratos distintos | Existe | No reimplementar; `src/index/{hnsw,flat,ivf}.rs`, `graph/` y `search/alternate.rs` |
| Cuantización | Existe; PQ/SQ entrenados o derivados según tipo | Existe | `src/vector/quantization.rs`, SCANN SQ8; comparar representación antes de B/D |
| IDs de aplicación | Parcial: int64, IDMap o IDs nativos según familia | Existe | u128 y `(namespace,key)`; C sólo aclara capacidades |
| Filtros de candidatos | Existe: selector con restricciones por composición | Existe | Pre/In/PostFilter y recheck exacto en `src/sdk/search/vector.rs:153-280`; E no crea filtros |
| Invalidación de cache tras update de igual cardinalidad | N/A como garantía transaccional de biblioteca | Parcial | Negativo estático acotado en §9; investigar A |
| Persistencia de imagen | Existe; no commit de aplicación | Existe | `src/index/serialize/bytes.rs`; D extiende validación, no elimina WAL |
| Semántica de métrica/método | Existe, pero depende del índice | Existe; diagnóstico transversal parcial | `search_with_method`, `SearchExplanation`; B sólo información faltante |
| Autorización multitenant | N/A en `Index` | Existe en superficies de Vanta; cobertura global no acreditada | No sustituir `InjectionPolicy`/namespace por selector |
| BM25/RRF, provenance, dedup y contexto | N/A en la biblioteca ANN inspeccionada | Existe | Mantener capa de memoria; Faiss no la reemplaza |
| Diagnóstico de presupuesto agotado | Parámetros/stats existentes, no respuesta de agente | No acreditado end-to-end para todas las rutas | E condicional; nunca inferir de `len<k` |
| UI de administración | N/A para esta biblioteca | Existe, Studio separado | Extender RetrievalLens/ActivityPanel, §10 |
| ACID entre recuerdo, índice y sesión | N/A como contrato Faiss | No acreditado por esta lectura | No deducir ni ausencia ni certificación desde un índice serializable |

## 9. Negativos revalidados

### 9.1 Invalidación basada en cardinalidad: hecho estático, no incidente reproducido

En [search_ivf/search_scann:16-95](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/search/alternate.rs#L16-L95)
la condición de reconstrucción observa presencia del cache y `nodes.len()`
frente a `*_built_at_node_count`. En
[validate_node:321-336](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/graph/core.rs#L321-L336)
un ID existente recibe nuevo bitset, vector, offset y normas sin cambiar la
longitud. Los callers `add`/`add_with_level` retornan después de ese caso.
La búsqueda de referencias de ambos contadores en `src/index` al mismo SHA
localizó inicializaciones, rutas lazy, serialización y tests; no un incremento
por esa actualización. Esto acredita una **condición de obsolescencia dentro
de CPIndex** si se reutiliza el objeto después de modificarlo.

No demuestra que todos los SDK lleguen al mismo objeto ni que el mecanismo de
publicación del motor nunca reemplace el índice. La aceptación pendiente es
construir cache, actualizar mismo ID a vector distinto sin variar población y
consultar por la ruta efectiva; después trazar el caller de memoria concreto.
Un resultado distinto de HNSW por aproximación no basta: se debe discriminar
vector previo vs vector actualizado y población realmente indexada.

### 9.2 Guardar una cache no acredita su frescura

[serialize/bytes.rs:224-239](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/serialize/bytes.rs#L224-L239)
escribe el IVF presente; la reapertura
[611-626](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/serialize/bytes.rs#L611-L626)
inicializa el contador con la población leída. Una imagen íntegra puede por
tanto conservar una cache vieja **si se llega al writer con ella vieja**.
Es una consecuencia condicional de 9.1, no una corrupción de disco observada
ni prueba de fallo del recovery WAL. SCANN reaparece como `None`: no atribuirle
la misma persistencia de IVF.

### Falsos positivos descartados y pendientes

- Hay header, versión, rechazo de imagen cifrada sin contexto y detección de
  lecturas insuficientes. “Vanta no valida imágenes” sería falso.
- Hay métricas alternativas y rechazo explícito de overrides incompatibles
  en IVF/SCANN; no prometer convertirlos silenciosamente para imitar Faiss.
- Hay filtros exactos después del bitset; no se ha demostrado fuga multitenant.
- HNSW, IVF, Flat y cuantización ya existen. El negativo es frescura/contrato,
  no falta de algoritmos ni calidad semántica inferior medida.
- Concurrencia de reconstrucción, crash entre registros y proyecciones,
  overflow en todos los lectores y efectos de embedders externos siguen sin
  reproducción. No promover hipótesis a bug desde la FAQ de otra biblioteca.

## 10. Escritorio y administración

La superficie estudiada es API C++/Python/C, tutoriales y scripts. No se
identificó un desktop de administración propio en esa muestra; no se infiere
ausencia de toda aplicación externa que use Faiss. Los notebooks/demos no
prueban una UI de ciclo de vida de recuerdos ni control de permisos.

Se releyó el worktree **develop-desktop**, pero se corrigió una atribución
incorrecta conservada de la ampliación anterior: `git show` confirma que los
dos componentes siguientes **no existen en el commit**
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`. Su estado local es `AM`
(añadidos al índice y modificados en disco). Por tanto no se ofrecen enlaces
GitHub a ese SHA como prueba; esto es evidencia **de disco concurrente, no
del snapshot Vanta congelado ni de un release**:

- `C:\Users\Eros\VantaDB Proyect\develop-desktop\desktop\src\components\lens\retrieval\RetrievalLens.tsx`,
  líneas 10-36 y 138-164: importa search/get/audit y filtros existentes;
  `runSearch` envía query, embedding, top_k, explain y search_profile al servidor.
  Líneas 121-129 filtran resultados por metadata/umbral en cliente: no
  confundir ese filtro visual con autorización ni exhaustividad server-side.
  Se conserva el orden del servidor, no se recalcula ANN.
- `C:\Users\Eros\VantaDB Proyect\develop-desktop\desktop\src\components\activity\ActivityPanel.tsx`,
  líneas 54-83: `fetchPage` envía filtros y cursor a `auditEvents`, concatena
  páginas y distingue audit no configurado de error de lectura.
  No acredita generación IVF, estado de tarea durable ni `SearchCompletion`.

Pendiente fijar esos cambios en una revisión de escritorio por su propietario;
este autor no modifica ni stagea dicho worktree. La existencia de Studio sí
forma parte de [VANTA-REFERENCE](VANTA-REFERENCE.md), pero no permite inventar
permalinks para componentes posteriores.

B/E añadirían metadatos al diagnóstico existente, A podría mostrar generación
y motivo de rebuild sin exponer vectores, y D mostraría errores de apertura
por categoría. Ninguna ficha autoriza pantalla nueva, controles para cambiar
presupuestos ni lanzar reconstrucciones desde UI sin permisos. Runtime visual
y paridad de transports permanecen pendientes; no se abrió ni arrancó Studio.

## Fuentes, cobertura y cierre

**Fuentes de código leídas:** API y ownership `Index.h`, `index_io.h`;
`IndexFlat.cpp`, `IndexFlatCodes.cpp`, `IndexIDMap.cpp`;
rutas de train/add/search/remove/update/merge en `IndexIVF.cpp`;
validaciones seleccionadas en `ProductQuantizer.cpp` e `impl/index_read.cpp`;
`bench_hnsw.py`, `bench_all_ivf/bench_all_ivf.py`,
`bench_all_ivf/datasets_oss.py`, contrato y SIFT de `contrib/datasets.py`.
Los enlaces de las secciones fijan el mismo SHA y rangos, no `main`.
Las lecturas Vanta de §9 se hicieron con `git show` al snapshot, no con una
revisión dirty atribuida al commit.

**Fuentes públicas complementarias efectivamente leídas:** #4656 y comentarios,
#4655 y comentario de cierre, FAQ oficial (apartados de concurrencia, ownership,
IDs, precisión y reproducción); se añadió el cuerpo de #4535 por fallback
HTML, conservando laguna de estado/fechas/comentarios y el rate limit API (§3).
No hay score propio ni reproducción de esos tickets. La lectura directa de
fuente es la comprobación primaria; no se validó por varios motores una
afirmación distinta de las que contienen esas fuentes.

**Cobertura por eje:** arquitectura/flujos, recomendaciones, cinco fichas,
incidencias, prerrequisitos, rechazos, método, dos familias de runners, matriz,
negativos y Studio incluidos. No se auditaron kernels GPU/cuVS, todas las
variantes de `index_factory`, todos los lectores binarios, todas las rutas de
cancelación ni todas las dependencias legales. Más clases no justifican
automáticamente más propuestas.

**Dependencias:** A precede a prometer frescura persistida en D; B permite
interpretar E; C conserva identidad para cualquier compactación/rebuild.
No exigen un LLM, checkpoint de agente o scheduler paralelo.

**Partición futura:** si se aprueban specs, separar identidad/mutación,
frescura/publicación de derivados, diagnóstico y compatibilidad de imágenes.
Mantener este dossier como evidencia; no se crean esos archivos ahora.
TOC permite navegar el informe de más de 300 líneas sin fingir que sea una spec.

**Cierre de autoría de esta pasada:** se conservaron cinco fichas, los dos
runners y los negativos de cache; se precisaron I/O, publicación/cancelación,
población elegible, import preferente de dataset, #4535 y procedencia dirty de
Studio. Los dos últimos puntos corrigen límites de evidencia, no implementan
features. La cobertura de incidencias sigue parcial para #4535 y la evidencia
de esos componentes UI aún no tiene SHA publicable.

**Pendiente manual:** casos adversos A–E, crash/reopen scoped, paridad SDK/UI y
benchmarks sólo tras autorización, usando METHOD sin otro protocolo.
La revisión adversarial independiente y la validación documental por lote
corresponden al responsable; no se declaran realizadas. No staging, commit,
push, inventarios, servicios ni ejecución de código en este cierre documental.
