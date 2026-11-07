---
title: "Milvus: ciclo de segmentos, consistencia y preparación de ingesta"
kind: research
status: proposed
description: "Contraste estático de readiness e índices de Milvus con el motor embebido existente de VantaDB"
tags: [research, memory, engineering]
---

# Milvus: ciclo de segmentos, consistencia y preparación de ingesta

## 1. Recomendación y arquitectura

**Dictamen:** extraer vocabulario verificable de readiness y publicación,
no reproducir coordinadores y nodos distribuidos.
Local: `C:\Users\Eros\milvus`; HEAD con fuentes verificado:
`eba8be50f84776210d3946d83840dea18924846f`.
[Snapshot](github-candidates.json): **46 612 estrellas, Apache-2.0**.
Clon y omisiones: [manifiesto](clone-manifest.json).

```text
cliente → Proxy (Go, schema/IDs/timestamps)
  escritura → StreamingClient → StreamingNode/WAL
    RecoveryStorage → checkpoints/metadatos + binlogs/estadísticas
    DataCoord / index jobs → artefactos de índice
  consulta → QueryCoord / QueryNode delegator
    espera garantía temporal → fija segmentos legibles
    segmentos growing/sealed → segcore C++ / índices Knowhere
metadata/catalog: etcd; datos/índices: almacenamiento local u objetos
componentes adicionales: Rust/Tantivy; backend WAL según despliegue
```

La guía oficial del checkout distingue **persistencia de segmentos** de
**construcción de índices** y de consumo para consulta. «Insert terminó»,
«segmento sealed», «índice cargado» y «read-your-writes» no son sinónimos.
En esta revisión no se presupone que cada consulta espere lo mismo:
el camino de resultado parcial y las tablas externas tienen excepciones M2.

## 2. Lectura de código y fichas candidatas

| Ref. | Fuente exacta al commit local | Hecho estático |
|---|---|---|
| M1 | [`internal/proxy/util.go:886-907`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/util.go#L886-L907) | Traduce Strong/Bounded/Eventually a garantía temporal; Bounded usa graceful time |
| M2 | [`internal/querynodev2/delegator/delegator.go:603-646`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/delegator/delegator.go#L603-L646) | Espera tSafe o permite parcial; fija MVCC y pin de segmentos |
| M3 | [`internal/querynodev2/segments/segment.go:65-115`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/segments/segment.go#L65-L115) | Tipo growing/sealed, versión y estado IsLoaded por campo indexado |
| M4 | [`internal/querynodev2/segments/segment_loader.go:81-107`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/segments/segment_loader.go#L81-L107) | Reserva/liberación de recursos; cargar datos e índices no publica en SegmentManager |
| M5 | [`internal/datacoord/meta/segment_info.go:38-95`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/datacoord/meta/segment_info.go#L38-L95) | Separa estado privado con lock del almacén compartido; EnsureStats no muta lectores |
| M6 | [`internal/streamingnode/server/wal/interceptors/shard/policy/seal_policy.go:30-95`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/streamingnode/server/wal/interceptors/shard/policy/seal_policy.go#L30-L95) | Motivos tipados de seal: fence, capacidad, lifetime, idle, memoria |
| M7 | [`internal/proxy/dml/task_insert.go:30-51`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_insert.go#L30-L51) | Task mantiene schema/version/timestamp e inputs de idempotencia |
| M8 | [`tests/python_client/bulk_insert/test_bulk_insert_perf.py:21-164`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/tests/python_client/bulk_insert/test_bulk_insert_perf.py#L21-L164) | Runner pytest bulk, generador y upload MinIO, medias/success rate y monitores |

Documentación contrastada: guías
[`streaming-system.md:7-43`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/streaming-system.md#L7-L43),
[`recovery-storage.md:1-45`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/wal/recovery-storage.md#L1-L45)
y `wal/shard-management.md:1-33`; no contradicción observada en estos rangos.
RecoveryStorage se cita como **documentación**, no como verificación completa
del motor de replay. No se leyó toda la arquitectura WAL enlazada.

### Ficha candidata A — estado compuesto de preparación de ingesta

- **Objetivo:** comunicar si el contenido es durable, consultable y cubierto por
  cada índice, sin condensarlo en un ambiguo «ready».
- **Mecanismo mínimo / fuente exacta:** separación M3/M4, más snapshot de
  publicación M2; no necesita segmentos distribuidos.
- **Seam Vanta:** `src/backend.rs:34-55` (particiones derivadas),
  `src/sdk/search/hybrid.rs:33-55` y rutas de ingesta/diagnóstico existentes.
- **Tipos propuestos:** `IngestReadiness { source_version, durable_position,
  dense_generation, text_generation, sparse_generation }`; reutilizar
  estados actuales si ya expresan esos hechos.
- **Acciones:** inventariar estados reales; rastrear qué mutación avanza cada
  generación; documentar fallback y significado de missing por backend.
- **Dependencias:** identidad de batch/registro, contadores recuperables,
  publicación coherente y errores de reconstrucción visibles.
- **FMEA:** indicador actualizado antes del índice promete cobertura inexistente;
  un rebuild puede hacer retroceder estado; una generación global mezcla tenants.
- **Aceptación funcional propuesta:** readiness deriva de hechos publicados;
  consulta distingue no-indexado de no-encontrado; error/cancelación no marca listo.
- **No objetivos:** diseñar coordinador, réplica o un scheduler nuevo.

### Ficha candidata B — barrera local de lectura después de escritura

- **Objetivo:** expresar una expectativa concreta de visibilidad sin importar TSO.
- **Mecanismo mínimo / fuente:** token de mutación y espera cancelable por
  watermark local, inspirado en M1/M2; semántica exacta pendiente de decidir.
- **Seam Vanta:** receipt actual de escritura, `src/wal.rs:999-1028`,
  publicación de índices y consulta SDK; no confundir posición WAL con versión MVCC.
- **Tipos propuestos:** `ReadAfter { mutation_id }`,
  `VisibilityOutcome::{Observed, TimedOutUnknown, Unsupported}`.
- **Acciones:** comprobar si ya hay equivalente; distinguir read-from-primary,
  actualización de metadata y búsqueda ANN; especificar qué índices cubre el token.
- **Dependencias:** orden de operaciones durable/applied y política explícita
  para backend in-memory o índice ausente.
- **FMEA:** esperar sólo WAL no garantiza retrieval; timeout después de aplicación
  induce retry duplicado; locks retenidos durante espera bloquean publicación.
- **Aceptación funcional propuesta:** no éxito sin observar la mutación en las
  superficies prometidas; cancelación libera la espera; unsupported no es éxito.
- **No objetivos:** consistencia global, modo Bounded por reloj de pared ni MVCC completo.

### Ficha candidata C — publicación de índice como fase separada

- **Objetivo:** evitar que «artefacto cargado» signifique «visible a lectores».
- **Mecanismo mínimo / fuente:** contrato load-without-publish y reserva RAII de
  M4; lectura pin/version de M2 y estado privado/compartido de M5.
- **Seam Vanta:** HNSW publicado y backend, auditorías existentes de ANN/WAL,
  sin suponer que falta swap atómico.
- **Tipos propuestos:** `PreparedIndexGeneration` y `PublishedIndexGeneration`,
  como envoltorios internos sólo si el contrato actual no basta.
- **Acciones/dependencias:** mapear locks y lifecycle existentes; delimitar
  ownership de temporales y reclamación de generación vieja.
- **FMEA:** limpieza elimina archivos aún referenciados; fallo tras carga deja
  recursos reservados; publicar vector antes de payload crea snapshot mixto.
- **Aceptación funcional propuesta:** preparación fallida no es publicable;
  lectores anteriores conservan referencias válidas; recursos se liberan al abortar.
- **No objetivos:** pipeline de índices distribuido ni tuning de tamaños/colas.

## 3. Riesgos, issues y seguridad

**Código:** M2 permite no esperar el mismo tSafe cuando se habilita parcial.
No es un defecto; impide describir todas las consultas como fuertes por defecto.
**Hipótesis transferibles:** readiness parcial mal comunicado, referencias
durante compactación y pérdida de clasificación retryable en fronteras C++/Go.
No se siguió cada constructor de error: no se afirma un bug de propagación.
**Issues upstream:** no consultados. El TODO de migración documentado en
recovery-storage no equivale a un issue leído ni a fallo reproducido.

**Configuración estática:** [`configs/milvus.yaml:1391-1415`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/configs/milvus.yaml#L1391-L1415)
incluye `authorizationEnabled: false` y `defaultRootPassword: Milvus`.
Frontera: clientes de red → servicio; revisar auth/TLS y rotación real.
No demuestra exposición de una instancia; variables/deployments pueden cambiarlo.
Las credenciales MinIO del runner M8 son de fixture, no secretos descubiertos.

## 4. Prerrequisitos VantaDB

Antes de incorporar algo: revalidar worktree dirty, inventario de health/ingesta,
generaciones HNSW y auditorías ANN/híbrido/WAL/backend ya existentes.
[`src/sdk/search/vector.rs:157-200`](https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/sdk/search/vector.rs#L157-L200)
ya selecciona prefilter/scan/máscara; no justificar con «no hay planner».
WAL SyncMode y batch backend ya existen. La suite
`tests/durability_recovery.rs:30-80` usa drop/reopen, no acredita por sí sola
interrupción de energía. Contrastar publicación actual antes de pedir otra.

## 5. Qué no trasladar

No portar etcd/objeto/TSO/PChannels a un motor local por el vocabulario readiness.
No tomar reglas de seal como parámetros de Vanta ni usar flush como condición
universal de búsqueda. No llamar «sin índice» a todo segmento growing.
No convertir arquitectura distribuida en requisito para memoria de agentes.

## 6. Método, licencia y límites

Sólo estático: rangos M1-M8 y guías citadas. No se ejecutó código, tests,
benchmarks, scripts ni servicios. Shallow sin historia completa/tags; LFS y
submódulos sin materializar según manifiesto; Knowhere/segcore y backends WAL
externos no auditados. Omitidos: CDC, todas las políticas de recovery/GC,
errores cgo end-to-end, índices GPU y todos los modos de despliegue.

Apache-2.0 congelado; M5 contiene aviso Apache/LF AI. Revisar LICENSE/NOTICE,
subcomponentes y cabeceras de cada archivo antes de portar; no se hizo
inventario legal per-file completo y cambiar Go/C++ a Rust no exime.
Antecedentes Vanta/OpenViking son históricos, pendientes de revalidación actual.

## 7. Benchmarks upstream

M8 es un **runner pytest de bulk insert/perf**, no prueba ANN recall-QPS.
Genera JSON/NPY con número/dimensión/varchar parametrizados, los sube a MinIO
y lanza BulkInsertChecker en threads. Observa success rate, total, tiempo
medio y coste temporal de upload; no publica percentiles p50/p95/p99 en el
archivo leído ni calcula calidad de respuesta/memoria.

Infraestructura: Milvus real, pymilvus, pytest, MinIO, descubrimiento Kubernetes
y utilidades de chaos/deploy. Costes: datos locales, red de upload, objetos,
jobs de importación, CPU/RAM y servicios del clúster; no cuantificados.
`teardown_method` sólo espera/loguea threads; el éxito termina checkers.
No se acreditó limpieza completa de colecciones/objetos ni terminación si falla
antes del final. El bucle espera total de operaciones, no un deadline explícito.
Requiere entorno desechable; no usar credenciales/namespace compartido.
No se ejecutó ni se adoptaron sus umbrales; cualquier comparación sigue
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz de brechas

| Capacidad | VantaDB | Contraste defendible |
|---|---|---|
| ANN/híbrido BM25 + dense/sparse RRF | Existe | Readiness no equivale a añadir retrieval |
| Payload/text/sparse persistentes | Existe: backend.rs:34-55 | Revisar generación/estado por índice |
| WAL, SyncMode, backend batch | Existe | Mapear receipt → publicación → barrera |
| Auditorías ANN/híbrido/WAL/backend | Existentes históricas; vigencia parcial | Revalidar estado dirty, no duplicarlas como novedad |
| Barrera SDK por token de mutación | No acreditada aquí | Candidata B, no defecto afirmado |
| Estado compuesto de cobertura | Parcial: diagnóstico existente; detalle no acreditado | Candidata A condicionada al inventario |
| TSO/coordinadores distribuidos | No aplicable a extracción local mínima | No adoptar |

## 9. Debilidades sugeridas

Preguntar si las garantías publicadas distinguen persistencia, preparación
del índice y visibilidad de consulta. Si ya se distinguen, el resultado es
documentar/instrumentar el contrato, no añadir otro estado.
Las unidades A/B pueden servir a ingestors y SDK sin traer Milvus como runtime.
No se afirma superioridad, ausencia de capacidades Vanta ni crash safety medido.

## 10. UI e inspección

[`README.md:156`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/README.md#L156)
remite a **Attu externo** para administración y Birdwatcher/Prometheus/Grafana.
No se inspeccionó su fuente externa ni una aplicación desktop nativa de Milvus.
Una referencia comercial/ecosistema no prueba UI integrada en este repositorio.
Vanta Studio ya vive en `develop-desktop` con memoria/actividad/diagnóstico;
los estados propuestos serían insumos para esa UI existente.
