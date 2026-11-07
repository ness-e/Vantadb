---
title: "Faiss: referencia ANN, contratos de cuantización y límites de evaluación"
kind: research
status: proposed
description: "Estudio estático de una biblioteca de índices vectoriales sin confundirla con un producto de memoria"
tags: [research, memory, engineering]
---

# Faiss: referencia ANN, contratos de cuantización y límites de evaluación

## 1. Recomendación y arquitectura

**Dictamen:** referencia de algoritmos, representación y contratos, no sistema
de memoria ni sustituto automático del motor VantaDB.
Local `C:\Users\Eros\faiss`; HEAD con árbol verificado:
`277b3c76b7a1507915eb8f5010dcb367cbc95494`.
[Inventario](github-candidates.json): **41 261 estrellas, MIT**.
[Manifiesto](clone-manifest.json): límites del clon reproducible.

```text
bindings Python / C API → API C++ Index
  train → add → search / range_search
  Flat → distancias exactas sobre almacenamiento vectorial
  IVF → coarse quantizer → nprobe listas → scan/códigos
  HNSW → grafo → DistanceComputer del storage
  PQ/SQ y otras representaciones → códigos + parámetros entrenados
  index_io → serialización/lectura, mmap en tipos soportados
backends CPU OpenMP/SIMD/BLAS y GPU opcionales
benchs + contrib/datasets → referencias ANN, no sesiones de agentes
```

No se localizó en la API `Index` inspeccionada dominio de memoria, procedencia,
sesiones, RRF léxico ni WAL transaccional. Son responsabilidades del integrador,
no «defectos» de una biblioteca cuyo objetivo es búsqueda vectorial.

## 2. Fuentes reales y fichas mínimas

| Ref. | Archivo/rango con enlace inmutable | Evidencia |
|---|---|---|
| F1 | [`faiss/Index.h:87-171`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/Index.h#L87-L171) | Parámetros por consulta/selector, dimensión, métrica, is_trained y ciclo train/add |
| F2 | [`faiss/IndexFlat.cpp:26-59`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexFlat.cpp#L26-L59) | Flat separa inner product, L2 cuadrada y otras métricas; acepta selector |
| F3 | [`faiss/IndexIVF.cpp:529-571`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexIVF.cpp#L529-L571) | Comprueba entrenamiento/listas; nprobe/max_codes; combinaciones selector/store_pairs inválidas |
| F4 | [`faiss/IndexHNSW.cpp:419-475`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/IndexHNSW.cpp#L419-L475) | Storage requerido, efSearch por llamada y ejecución OpenMP por bloques |
| F5 | [`faiss/impl/ProductQuantizer.cpp:58-78`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/impl/ProductQuantizer.cpp#L58-L78) | M positivo, d divisible por M, code_size/ksub derivados |
| F6 | [`faiss/AutoTune.cpp:105-150`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/AutoTune.cpp#L105-L150) | OperatingPoints retiene puntos rendimiento/tiempo no dominados |
| F7 | [`faiss/index_io.h:20-81`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/index_io.h#L20-L81) | Ownership raw/unique_ptr; opciones IO no universales; mmap no es WAL |
| F8 | [`faiss/impl/index_read.cpp:1190-1226`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/faiss/impl/index_read.cpp#L1190-L1226) | Valida estructura HNSW: niveles y monotonicidad, entre otros checks |
| F9 | [`benchs/bench_hnsw.py:18-125`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/benchs/bench_hnsw.py#L18-L125) | Runner SIFT: train/database/query/groundtruth, tiempo medio, nearest-neighbor recall y missing |
| F10 | [`contrib/datasets.py:155-181`](https://github.com/facebookresearch/faiss/blob/277b3c76b7a1507915eb8f5010dcb367cbc95494/contrib/datasets.py#L155-L181) | DatasetSIFT1M y ficheros fvecs/ivecs separados |

### Ficha candidata A — descriptor verificable de representación vectorial

- **Objetivo:** impedir que «cuantizado» o «mmap» oculte dimensión, codec o
  parámetros incompatibles al persistir/reabrir.
- **Mecanismo mínimo / fuente:** invariantes de tamaño y estado entrenado
  F1/F5, más límites de lectura por tipo F7/F8; no importar ProductQuantizer entero.
- **Seam Vanta existente:** `src/vector/quantization.rs:12-84` ya contiene
  RaBitQ/packing escalar; `src/index/ivf.rs:25-57` ya tiene IVF Flat.
  No proponer «añadir cuantización/IVF» como si no existieran.
- **Tipos propuestos:** `VectorEncodingDescriptor { codec_version, dimension,
  metric, code_bytes, training_fingerprint }`, con fingerprint opcional
  sólo para codecs entrenados; preferir extender el formato vigente.
- **Acciones:** revisar metadatos actuales y formatos/versiones; identificar qué
  invariantes ya valida recovery; proponer sólo las que falten acreditadamente.
- **Dependencias:** versionado on-disk, migración, ownership mmap y métrica SDK.
- **FMEA:** reutilizar codebook ajeno devuelve scores incoherentes; padding se
  confunde con dimensiones reales; validar después de asignar permite OOM.
- **Aceptación funcional propuesta:** descriptor incompatible falla antes de
  publicar índice; reapertura conserva significado de métrica/dimensión;
  representación no entrenada no anuncia codebook inexistente.
- **No objetivos:** nuevo codec PQ, ajustes de bits, SIMD ni optimización.

### Ficha candidata B — semántica explícita de score y exactitud

- **Objetivo:** no confundir L2 con L2², similitud con distancia ni exactitud
  geométrica con calidad de memoria.
- **Mínimo mecanismo / fuente exacta:** distinción F2, estado de parámetros
  efectivos F3/F4 y soporte parcial IO F7.
- **Seam Vanta:** `src/index/distance`, `src/sdk/search/vector.rs:67-112`
  (scan exacto y conversión Euclidean), explicaciones de búsqueda y SDKs actuales.
- **Tipos propuestos:** `ScoreSemantics { metric, ordering, squared,
  representation }`; extender explicación actual, no inventar otro resultado.
- **Acciones:** inventariar contratos actuales por endpoint/binding, documentar
  score efectivo y ruta exact/approx sin alterar orden ni parámetros.
- **Dependencias:** datos originales disponibles para exactitud, normalización,
  validación de dimensiones y proyección de metadatos a cliente.
- **FMEA:** tratar inner product como cosine sin normalizar cambia significado;
  signo invertido de distancia ordena peor primero; «exact» sobre representación
  con pérdida no acredita exactitud respecto al vector original.
- **Aceptación funcional propuesta:** cliente puede interpretar orden/unidades;
  filtros y tombstones no desaparecen en modo exacto; explicación no modifica hits.
- **No objetivos:** nuevo benchmark/harness, umbrales de recall ni autotuning.

## 3. Riesgos, issues y frontera de seguridad

**Código:** F7 no ofrece el mismo mmap/flags para cualquier tipo de índice;
F3 rechaza algunas combinaciones. El adaptador debe declarar capacidades,
no traducir unsupported a éxito silencioso.
**Estático:** F8 demuestra validación estructural actual; sería falso afirmar
que toda lectura de índices carece de validaciones.
**Hipótesis:** archivo no confiable puede activar asignaciones grandes o rutas
no leídas; los checks de un tipo no certifican todos los deserializadores.
Frontera: fichero/importación no confiable → proceso C++/FFI; aquí no hay
configuración de auth de servidor que auditar. Predeterminar confianza en un
archivo local no equivale a sandbox seguro.
**Issues upstream:** no consultados; no se afirma vulnerabilidad vigente.

## 4. Prerrequisitos VantaDB

Revalidar baseline dirty y auditorías existentes de ANN, híbrido, WAL/backend.
Vanta ya tiene HNSW, Flat, IVF Flat, cuantización, filtros y RRF. Evidencia
actual adicional:
[`src/index/flat.rs:11-52`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/index/flat.rs#L11-L52),
[`src/index/ivf.rs:25-57`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/index/ivf.rs#L25-L57),
[`src/vector/quantization.rs:12-84`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/vector/quantization.rs#L12-L84).
La existencia del código no acredita integración/calidad de todas sus rutas.

[`src/index/auto_tune.rs:21-51`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/index/auto_tune.rs#L21-L51)
ya tiene adaptación de ef. F6 es referencia conceptual, **no autorización**
para cambiarla ni añadir una búsqueda automática de configuraciones.

## 5. No trasladar

No importar todos los índices ni GPU/BLAS/FFI para resolver tipos de score.
No usar un serializador de índices como storage backend, WAL o aislamiento de
sesiones. No copiar constantes `efSearch`/`nprobe` del runner a producción.
No atribuir a Faiss contratos de procedencia, TTL, dedup semántico o agente.

## 6. Método, licencia y omisiones

Sólo fuentes/rangos F1-F10; F5 se amplió con lectura de su train por localización,
sin auditoría del algoritmo completo. No builds, tests, scripts, instalaciones,
benchmarks ni ejecución del source. Clon shallow, sin historia/tags,
LFS/submódulos no materializados según manifiesto.
Omitidos: GPU, kernels completos, código de cada codec, C API/bindings completos,
concurrencia add/search y todos los formatos de serialización.

MIT congelado; licencia raíz y cabecera de F9 verificadas. Terceros constan en
`THIRD_PARTY_NOTICES`, no auditado íntegro; obligaciones por archivo/dependencia
pendientes antes de derivación. Portar C++ a Rust no elimina copyright/licencia.
No convertir observaciones históricas Vanta/OpenViking en fallos actuales.

## 7. Runner upstream y lectura correcta de métricas

F9 usa **SIFT1M**: d=128, train=100 000, base=1 000 000, queries=10 000 según F10.
Carga ficheros distintos `sift_learn`, `sift_base`, `sift_query`,
`sift_groundtruth`. La presencia del loader no garantiza que el dataset esté
en el clon; no se descargó. Faiss Python/NumPy y biblioteca nativa instalada
son dependencias; CPU/OpenMP/BLAS y threads afectan el resultado.

El runner recibe k y variantes; HNSW Flat no entrena, SQ e IVF sí.
Temporiza **un batch search** y divide por nq: reporta ms/query medio, no
p50/p95/p99 por consulta ni distribución de colas de un servidor.
`recall_at_1 = (I == gt[:, :1]).sum()/nq` cuenta si el vecino groundtruth
primero aparece entre los k devueltos; no es recall de todo el conjunto top-k.
`missing_rate` cuenta etiquetas -1. No mide sesiones, QA, aislamiento ni durable ACK.

Costes a registrar: RAM de vectores/grafo/códigos, entrenamiento y add,
threads, tiempo/almacenamiento de dataset; no costes LLM/juez.
Usa índices en memoria y no muestra cleanup de los ficheros externos del dataset.
No ejecutar sobre directorio compartido como si fuera desechable.
F6 ofrece una estructura Pareto, no resultados medidos en esta investigación.
Comparaciones/percentiles futuros pertenecen al
[protocolo del responsable](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se diseñó harness, scoring ni ajustes nuevos.

## 8. Matriz contra VantaDB

| Superficie | VantaDB | Brecha defendible |
|---|---|---|
| HNSW y scan/Flat con filtros | Existe | Revisar contrato de métrica/explicación, no reimplementar ANN |
| IVF Flat | Existe | Faiss tiene más variantes; no demuestra necesidad de portarlas |
| Cuantización escalar/binaria | Existe | Descriptor/versionado completo pendiente de acreditar |
| BM25 + dense/sparse + RRF | Existe | No equivalente en la API Index leída; no objetivo de Faiss |
| WAL/backend y auditorías de recovery | Existe | No aplicable como capacidad a exigir a Faiss |
| Auditorías ANN/híbrido/WAL/backend | Existentes; vigencia parcial | Revalidación dirty antes de propuesta implementable |
| Producto de memoria/sesiones/procedencia | Existe en Vanta; no objetivo de Faiss | No sustituible por recall ANN |
| GPU de Faiss | No aplicable a extracción mínima | No recomendado sin necesidad y evidencia propias |

## 9. Debilidades sugeridas por el contraste

Pregunta acotada: ¿todos los bindings comunican score/unidades y formato
efectivo igual, incluyendo exact scan frente a cuantización?
Pregunta de persistencia: ¿la metadata actual impide mezclar codecs/versiones?
No se respondió exhaustivamente: A/B son candidatos condicionados a ese inventario.
El runner enseña por qué recall ANN no certifica memoria ni latencia de cola;
no acredita que Faiss o Vanta sea superior.

## 10. UI e inspección

No se localizó frontend de administración o desktop nativo en las rutas
inspeccionadas. Bindings, scripts y notebooks de benchmarks son herramientas
de desarrollo, no un navegador de memoria con permisos/sesiones.
No se inspeccionaron UIs de terceros que integran Faiss.
Studio de Vanta ya tiene diagnóstico retrieval en `develop-desktop`; la
aportación sería mostrar semántica/representación existente, no crear una UI.
