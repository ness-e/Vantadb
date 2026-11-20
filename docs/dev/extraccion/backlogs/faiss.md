---
title: "Backlog de extracción — Faiss"
kind: research
status: active
description: "Candidatas del informe Faiss: generación de caches vectoriales, semántica de score, identidad estable en mutaciones, admisión de representaciones y diagnóstico de terminación"
tags: [extraccion, backlog, faiss]
---

# Backlog de extracción — Faiss

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/faiss.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-FAI-01 | 🔴 | Generación de caches vectoriales ligada a revisión de fuente | `src/index/search/alternate.rs:16-95` | 🟡 2-5d | 🔴 P0 | ⬜ Candidata | Qué: cada derivado lazy (IVF/SCANN) lleva source_epoch/métrica/dimensión/representación y una construcción para una época vieja no puede publicarse como actual; `StaleGeneration` es error explícito. Por qué: `validate_node` actualiza el vector de un ID sin cambiar `nodes.len()` y la condición de rebuild observa solo cardinalidad (negativo estático §9 del informe); una imagen íntegra puede conservar cache vieja. Acción mínima: inventariar mutaciones de vector/bitset/remove/bulk, invalidar al mutar con el orden de locks existente, construir sobre snapshot coherente y publicar solo si la revisión coincide; cubrir SCANN sin mezclar codecs. Trazabilidad: informe §Ficha A · evidencia `faiss/IndexIVF.cpp:1543-1588`, `faiss/IndexIVF.cpp:1592-1626` | X-FAI-04 (frescura persistida), C14 | Autoridad de revisión de `CPIndex`, orden de locks y snapshot consistente de nodos |
| X-FAI-02 | 🟡 | Semántica de score y método efectivo declarados en la explicación | `src/sdk/search/vector.rs:67-112` | 🟢 1d | 🟠 P1 | ⬜ Candidata | Qué: la explicación de búsqueda adjunta métrica, orden, transformación de distancia, representación y método efectivo; un campo desconocido se comunica como desconocido. Por qué: L2² etiquetada como L2 o IP llamada cosine sin normalización falsean umbrales y contratos; un codec con pérdida no vuelve «exact» exhaustivo respecto al vector original. Acción mínima: inventariar endpoints/bindings, contrastar exact/ANN y representación, y ampliar el DTO de `SearchExplanation` de forma compatible sin otro ranking ni conversión ficticia. Trazabilidad: informe §Ficha B · evidencia `faiss/IndexFlat.cpp:26-59` | X-FAI-05 (interpreta su terminación), C7 | DTO de explicación versionable y autoridad del método; campos opcionales sin reindexación |
| X-FAI-03 | 🟠 | Identidad estable y capacidades de mutación declaradas por índice | `src/index/mod.rs:49-61` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: cada índice declara su soporte de mutación y el resultado de mantenimiento conserva identidad canónica/revisión; unsupported es error tipado y posición interna/offset nunca sale como ID persistente. Por qué: componer un mapa posicional con un índice que compacta reasigna hits a identidades ajenas (issues #4655/#4656/#4535); verificar `ntotal` no valida el mapping. Acción mínima: auditar remove/update/bulk/rebuild y sus adapters, preservar el mapping original en compactación y exigir guard de ID/revisión antes de mutar. Trazabilidad: informe §Ficha C · evidencia `faiss/IndexFlatCodes.cpp:61-80`, `faiss/IndexIDMap.cpp:192-225,275-295`, `faiss/IndexIVF.cpp:1385-1410` | X-FAI-01 (para caches), C1 | IDs/revisión canónicos u128 `(namespace,key)`, operaciones realmente soportadas por el backend y respuesta ante resultado parcial |
| X-FAI-04 | 🟡 | Admisión y compatibilidad de representación antes de publicar | `src/index/serialize/bytes.rs:265-348` | 🟡 2-5d | 🔵 P3 | ⬜ Candidata | Qué: la apertura de una imagen valida dimensión, métrica, codec/versión e identidad de parámetros de entrenamiento cuando existe, separando corrupción, clave ausente e incompatibilidad como categorías. Por qué: igual tamaño binario no implica igual entrenamiento y decodificar con otro codebook produce scores erróneos silenciosos. Acción mínima: inventario de codecs reales y campos ya persistidos, validar tamaños/overflow antes de reservas y comprobar relaciones grafo→storage en `src/vector/quantization.rs`; no duplicar el header `VNDX` existente. Trazabilidad: informe §Ficha D · evidencia `faiss/impl/ProductQuantizer.cpp:58-78`, `faiss/impl/index_read.cpp:1190-1260`, `faiss/index_io.h:18-81` | X-FAI-01 (generación persistida), C15 | Límites y versiones del lector existente, fuente canónica reconstruible; candidato condicional según informe |
| X-FAI-05 | 🟡 | Diagnóstico de filtro, presupuesto y razón de terminación | `src/sdk/search/vector.rs:153-280` | 🟢 1d | 🟡 P2 | ⬜ Candidata | Qué: el diagnóstico declara `exhausted: Unknown/CandidateSet/Budget` y método efectivo, distinguiendo universo pequeño, filtro, presupuesto agotado y ANN incompleto; nunca se infiere exhaustividad de `len<k`. Por qué: pocos hits puede significar presupuesto o candidatos insuficientes, no inexistencia semántica; intentar completar k ilimitadamente o relajar el filtro de seguridad es inaceptable. Acción mínima: seguir señales existentes de perfil, propagar presupuesto realmente usado, distinguir cero hits de no ejecutado y conectar Studio read-only. Trazabilidad: informe §Ficha E · evidencia `faiss/IndexIVF.cpp:529-615`, `benchs/bench_hnsw.py:18-134` | X-FAI-02, C7 | Señal real de terminación y recheck exacto por namespace ya existente; diagnóstico sin cambiar la búsqueda primero |

## Fichas completas

> Las fichas expanden la tabla resumen con el contenido del informe
> `docs/dev/extraccion/agent-memory-2026-11-07/faiss.md` (Fichas A–E de §2,
> precisiones §2.1 y negativos §9). Todos los contratos son **hipotéticos** —
> propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de
> código upstream. Las referencias `archivo:línea` son del clon
> `C:\Users\Eros\faiss` en SHA `277b3c76b7a1507915eb8f5010dcb367cbc95494`; las
> rutas `src/` de VantaDB fijan el snapshot `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.

### X-FAI-01 — Generación de caches vectoriales ligada a revisión de fuente

- **Problema:** una memoria actualiza el vector manteniendo ID y cardinalidad; un derivado lazy puede seguir representando el contenido anterior. Negativo estático propio (informe §9): `search_ivf`/`search_scann` reconstruyen al cambiar `nodes.len()`, pero `validate_node` (`src/index/graph/core.rs:321-336`) actualiza el vector/bitset de un ID existente sin cambiar esa longitud; una imagen íntegra puede conservar cache vieja (`serialize/bytes.rs:224-239`, reapertura `:611-626`). Es condición de obsolescencia dentro de `CPIndex`, no incidente reproducido ni corrupción de disco.
- **Contrato hipotético (tipos propuestos):** `DerivedGeneration { source_epoch, metric, dimension, representation }`; adquirir una generación produce una vista inmutable válida para una revisión. Una construcción para una época vieja no puede publicarse como actual; error explícito `StaleGeneration` si no puede usarse/reconstruirse — nunca éxito con índice equivocado.
- **Fuente de evidencia:** `faiss/IndexIVF.cpp:1543-1588` (`train`/`train_q1`: entrenar centroides y codificador son trabajos diferentes; `is_trained` solo tras entrenar) y `faiss/IndexIVF.cpp:1592-1626` (`check_compatible_for_merge`: tipo, dimensión, listas, tamaño de código, compatibilidad de coarse quantizers; la fusión con direct maps se rechaza — igual tamaño binario no implica igual entrenamiento).
- **Seam VantaDB:** `src/index/search/alternate.rs:16-95` (`CPIndex::search_ivf`/`search_scann`), actualización `src/index/graph/core.rs:321-374`, serialización `src/index/serialize/bytes.rs:224-239`. Tres fronteras del mismo derivado, no tres features.
- **Acciones:**
  1. Inventariar mutaciones de vector, bitset, remove y bulk.
  2. Fijar la autoridad de revisión de `CPIndex`.
  3. Invalidar al mutar con el orden de locks existente.
  4. Construir sobre snapshot coherente de nodos.
  5. Publicar solo si la revisión coincide; descartar la construcción si difiere.
  6. Impedir serializar una cache vieja como vigente; cubrir SCANN bajo el mismo principio sin mezclar sus codecs.
- **Dependencias:** autoridad de revisión de `CPIndex`, orden de locks y snapshot consistente de nodos; no depende de checkpoints de agente, LLM ni ledger externo. X-FAI-04 para frescura persistida.
- **FMEA:** update entre snapshot y publicación → cache obsoleta → detectar epoch distinto y descartar; lock tomado en orden inverso → deadlock → revisión del orden y construcción fuera de publicación; rebuild fallido → no destruir la generación anterior válida para lectores ya iniciados; delete+add con misma longitud → nunca usar cardinalidad como revisión. Adicional §2.1: equiparar población fuente e indexada puede convertir exclusiones por representación en «no hay recuerdos» → contadores agregados de elegibles/excluidos y motivo.
- **Criterios de aceptación propuestos:** mismo ID con vector nuevo, bitset nuevo, remove+add con cardinalidad constante, serializar/reabrir después de mutar y dos rebuilds concurrentes no publican estado viejo. Aceptación manual §9: construir cache, actualizar mismo ID a vector distinto sin variar población y consultar por la ruta efectiva, discriminando vector previo vs actualizado y población realmente indexada.
- **Criterio negativo:** cancelar un rebuild descarta solo el derivado no publicado — no revierte escrituras canónicas; un error al construir no valida la cache vieja para una consulta nueva.
- **Independencia:** inicialmente cache descartable sin cambiar registros canónicos; desactivar cache conserva lectura canónica. Si se persiste epoch, requiere versión de formato; un downgrade debe poder ignorar el derivado, no interpretar bytes nuevos.
- **No objetivos:** codec nuevo, autotuning ni reentrenar todo en cada query (coste oculto inaceptable); un adapter externo no ve todas las escrituras y no basta.

### X-FAI-02 — Semántica de score y método efectivo declarados en la explicación

- **Problema:** un agente o UI puede tratar distancia como probabilidad, o «exact» sobre un codec con pérdida como exactitud respecto al vector original; L2² etiquetada como L2 o IP llamada cosine sin normalización falsean umbrales y contratos.
- **Contrato hipotético (tipos propuestos):** `ScoreSemantics { metric, ordering, distance_transform, representation, effective_method }` adjunto a la explicación existente — no otro ranking ni conversión ficticia. Un campo desconocido se comunica como desconocido, no como conversión supuesta ni score cero.
- **Fuente de evidencia:** `faiss/IndexFlat.cpp:26-59` (Flat separa L2 cuadrada e IP como métricas distintas).
- **Seam VantaDB:** `src/sdk/search/vector.rs:67-112` (`exact_scan` ya transforma Euclidean a distancia negativa con raíz) y `SearchExplanation` existente; `src/index/search/alternate.rs:107-145` (IVF/SCANN metric-bound ya rechazan override incompatible — no reemplazar ese error por conversión ficticia).
- **Acciones:**
  1. Inventariar endpoints y bindings que exponen scores.
  2. Contrastar exact/ANN y representación por ruta.
  3. Ampliar el DTO de `SearchExplanation` de forma compatible.
  4. Mapear HTTP/native/WASM y Studio.
  5. Documentar la diferencia entre distancia de motor y RRF.
- **Dependencias:** DTO de explicación versionable y autoridad del método; campos opcionales sin reindexación. No depende de X-FAI-01 salvo para mostrar generación.
- **FMEA:** L2² etiquetada L2 → umbrales mal interpretados → comparar con caso geométrico conocido; IP llamada cosine sin normalización → contrato falseado → describir la métrica real; diagnóstico que altera ranking → comparación de paridad.
- **Criterios de aceptación propuestos:** consultas degeneradas, empty, empates y errores de dimensión mantienen el contrato; explicación y búsqueda ordinaria conservan IDs y orden. Observabilidad: contador por método/transformación, sin query cruda.
- **Criterio negativo:** la semántica desconocida se reporta como desconocida — nunca se inventa una conversión ni un score cero; la explicación no inicia trabajo ANN adicional ni altera el ranking.
- **Independencia:** campos opcionales: lectores viejos ignoran la extensión; apagar la explicación conserva la búsqueda. Alternativa mínima: solo documentación SDK si la semántica es única.
- **No objetivos:** calibración, pesos, thresholds, promesa de mejora ni corregir «a ojo» los scores del motor para imitar Faiss.

### X-FAI-03 — Identidad estable y capacidades de mutación declaradas por índice

- **Problema:** compactar posiciones no debe desplazar la identidad de recuerdos supervivientes; componer un mapa posicional con un índice que compacta reasigna hits a identidades ajenas — y verificar `ntotal` no valida el mapping.
- **Contrato hipotético (tipos propuestos):** descriptor de capacidad por índice (`MutationSupport`) y resultado del mantenimiento con identidad/revisión canónica; `unsupported` es error tipado antes de mutar, nunca operación ignorada. Posición interna/offset nunca sale como ID persistente. Operación + ID canónico + revisión esperada cuando el backend la ofrezca → capacidad/resultado de mantenimiento; estado incierto si el backend no garantiza completar todo.
- **Fuente de evidencia:** `faiss/IndexFlatCodes.cpp:61-80` (`remove_ids` compacta los códigos restantes: posición no es identidad estable), `faiss/IndexIDMap.cpp:192-225,275-295` (tabla externa también compactada; traducción de etiquetas con comprobación de rango), `faiss/IndexIVF.cpp:1385-1410` (`update_vectors` con mapa hash: remove → comprobación de cantidad → add; si no encontró todas las entradas, falla **después** del remove — API de mantenimiento, no reemplazo atómico). Issues públicas [#4656](https://github.com/facebookresearch/faiss/issues/4656) (mapping desincronizado tras delete con IndexIDMap2), [#4655](https://github.com/facebookresearch/faiss/issues/4655) (assertion `j == ntotal` al segundo delete) y [#4535](https://github.com/facebookresearch/faiss/issues/4535) (assertion tras varios borrados pese a cardinalidad correcta).
- **Seam VantaDB:** `src/index/mod.rs:49-61` (`VecIndex::add` devuelve error de capacidad; IVF construido es read-only en ese contrato). VantaDB ya usa `u128` y `(namespace,key)`: no introducir IDMap ni reducir IDs a `i64`.
- **Acciones:**
  1. Auditar remove/update/bulk/rebuild y sus adapters.
  2. Extender el port existente si falta información de capacidad.
  3. Preservar el mapping original en compactación.
  4. Exigir guard de ID/revisión antes de mutar.
  5. Publicar el índice tras validar referencias.
- **Dependencias:** IDs/revisión canónicos u128 `(namespace,key)`, operaciones realmente soportadas por el backend y respuesta ante resultado parcial; para caches, X-FAI-01. Una fachada externa no puede prometer atomicidad que el backend no ofrece.
- **FMEA:** retry de delete alcanza identidad reciclada → pérdida ajena → guard id/revisión; mezcla de mapa posicional y estable → hits reasignados → comparar IDs antes/después; partial update → old/new mezclados → resultado incierto y reconciliación, no éxito.
- **Criterios de aceptación propuestos:** borrar primer/medio/último, repetir borrado, actualizar superviviente, reabrir y reconstruir preservan identidad. Observabilidad: contadores de rechazadas/obsoletas/reconciliadas sin IDs sensibles por defecto. Discriminación futura del informe: comparar IDs canónicos supervivientes antes/después de borrar — no aceptar solo ausencia de crash o cardinalidad correcta.
- **Criterio negativo:** `unsupported` se decide antes de mutar; un resultado parcial es «incierto + reconciliación», nunca éxito; perder el waiter no es undo.
- **Independencia:** sin migración de IDs; cambios de layout solo con rebuild desde registros y publicación compatible. Rollback = generación anterior + replay canónico según garantías reales — no «deshacer» deletes ya confirmados.
- **No objetivos:** imponer el delete+add de Faiss al store de memoria ni prometer compare-and-swap donde el port actual no lo ofrece.

### X-FAI-04 — Admisión y compatibilidad de representación antes de publicar

- **Problema:** una imagen o código de dimensión/tamaño compatible puede tener entrenamiento incompatible: igual tamaño binario no implica igual entrenamiento y decodificar con otro codebook produce scores erróneos silenciosos.
- **Contrato hipotético (tipos propuestos):** `EncodingCompatibility` calculado a partir del formato vigente: dimensión, métrica, codec/versión y, solo si existe entrenamiento, identidad de parámetros. Fallo antes de publicar/asignar estructuras incompatibles; corrupción, clave ausente e incompatibilidad permanecen categorías distintas. Ninguna credencial/ACL se deduce del fingerprint.
- **Fuente de evidencia:** `faiss/impl/ProductQuantizer.cpp:58-78` (`set_derived_values`: M positivo, dimensión divisible, límite de bits, multiplicación protegida contra overflow), `faiss/impl/index_read.cpp:1190-1260` (lector HNSW valida niveles, offsets, vecinos y entry point — es falso que no tenga validación), `faiss/index_io.h:18-81` (ownership/flags de lectura: `SKIP_STORAGE` no produce un índice consultable), `faiss/IndexIVF.cpp:1594-1626` (`check_compatible_for_merge` + `merge_from` con ownership explícito).
- **Seam VantaDB:** `src/index/serialize/bytes.rs:265-348` (el lector ya valida header `VNDX`, versión, imagen cifrada y lecturas truncadas — extender invariantes faltantes, no duplicar el header) y `src/vector/quantization.rs` (relaciones grafo→storage).
- **Acciones:**
  1. Inventario de codecs reales y campos ya persistidos.
  2. Validar tamaños/overflow antes de reservas.
  3. Comprobar relaciones grafo→storage.
  4. Separar corrupción de falta de clave y de incompatibilidad.
  5. Revisión legal si se reutiliza material.
- **Dependencias:** X-FAI-01 para generación persistida; límites y versiones del lector existente; fuente canónica reconstruible. Candidato condicional según el informe (solo añadir validaciones no presentes).
- **FMEA:** metadatos mentirosos → OOM o score erróneo → límites del lector y rechazo; checksum correcto no prueba autenticidad → mantener frontera de confianza; mmap de fichero reemplazado → lifetime ownership, no punteros escapados.
- **Criterios de aceptación propuestos:** truncado, longitud imposible, dimensión/codec incompatible y clave ausente distinguidos como categorías; reopen correcto conserva resultados y significado. Observabilidad: conteos por error/versiones, sin bytes del índice.
- **Criterio negativo:** nunca decodificar silenciosamente con otro codebook; cancelar la apertura antes de publicar libera recursos temporales — no borra el primario.
- **Independencia:** sin nuevo campo si basta validar; si hace falta descriptor, versionado explícito y reconstrucción desde originales. Mantener la imagen anterior hasta validar la apertura; no eliminar datos fuente por fallo de índice.
- **No objetivos:** certificar todos los formatos Faiss ni nuevos esquemas de cifrado.

### X-FAI-05 — Diagnóstico de filtro, presupuesto y razón de terminación

- **Problema:** pocos hits puede significar universo pequeño, filtro, presupuesto agotado o ANN incompleto — nunca inexistencia semántica por defecto; intentar completar k ilimitadamente o relajar el filtro de seguridad es inaceptable.
- **Contrato hipotético (tipos propuestos):** `SearchCompletion { requested, returned, exhausted: Unknown|CandidateSet|Budget, effective_method }`; «exhausted» debe acreditarse por el algoritmo, no inferirse de `len<k`. El filtro de seguridad no puede relajarse para llenar k; un presupuesto sin instrumentación se reporta `Unknown`, no mentira operativa.
- **Fuente de evidencia:** `faiss/IndexIVF.cpp:529-615` (`search_preassigned`: exige entrenamiento, `k>0`, acota `nprobe` por `nlist`, toma `max_codes` y selector con restricciones de composición — el presupuesto delimita el trabajo, no garantiza llenar k; el selector comprueba IDs candidatos, **no permisos**), `benchs/bench_hnsw.py:18-134` (`missing_rate` cuenta etiquetas `-1`; conserva huecos, no los transforma en resultado malo).
- **Seam VantaDB:** `src/sdk/search/vector.rs:153-280` (ya selecciona Pre/In/PostFilter, adapta búsqueda y hace recheck exacto de namespace/filtros después del bitset, `:252-280`) — añadir solo razón de terminación a la explicación/perfil existente; Studio read-only para visualización.
- **Acciones:**
  1. Seguir las señales existentes de perfil.
  2. Conservar autorización/namespace y filtro exacto.
  3. Propagar el presupuesto realmente usado.
  4. Distinguir cero hits de no ejecutado.
  5. Conectar Studio en modo read-only con conteos y razones — nunca filtros privados.
- **Dependencias:** señal real de terminación y recheck exacto por namespace ya existente; depende de X-FAI-02 para interpretar la terminación; no de un nuevo autotuner.
- **FMEA:** aproximación Bloom tratada como autorización → fuga → recheck exacto obligatorio; «budget» sin instrumentación → mentira operativa → `Unknown`; intentar completar k ilimitadamente → DoS → mantener el presupuesto existente.
- **Criterios de aceptación propuestos:** filtros vacíos, namespace sin datos, selectividad alta, presupuesto agotado y k mayor que población; no hits ajenos al scope.
- **Criterio negativo:** la exhaustividad nunca se deduce de `len<k` ni de resultados truncados; el reporte no prolonga la búsqueda ni relaja filtros al cancelar.
- **Independencia:** diagnóstico sin cambiar la búsqueda primero; campos opcionales sin reindexación y rollback que ignora el diagnóstico. Ampliar cursor/resume solo si existe necesidad demostrada.
- **No objetivos:** cambiar ANN por principio, ni exponer `SearchCompletion` como promesa de recall.
