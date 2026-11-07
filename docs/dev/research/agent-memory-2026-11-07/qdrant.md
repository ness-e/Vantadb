---
title: "Qdrant: planificación filtrada, fusión y visibilidad persistente"
kind: research
status: proposed
description: "Lectura estática de Qdrant y unidades acotadas para contrastar con el retrieval existente de VantaDB"
tags: [research, memory, engineering]
---

# Qdrant: planificación filtrada, fusión y visibilidad persistente

## 1. Recomendación y arquitectura

**Dictamen:** estudiar el contrato de selección de ruta y de visibilidad, no
reemplazar el HNSW/híbrido de VantaDB ni importar un clúster.
Fuente local `C:\Users\Eros\qdrant`; HEAD verificado con árbol fuente:
`222b2e44ab1c3f17e8246396ead6558e45041df7`.
Snapshot autorizado: **35 458 estrellas, Apache-2.0**, según
[inventario congelado](github-candidates.json); preparación en
[manifiesto](clone-manifest.json). El SHA describe master observado, no un release certificado.

```text
REST src/actix / gRPC src/tonic — Rust, Tokio
  lib/storage — colecciones, autorización, consenso
    lib/collection — shards/réplicas, submit, workers
      lib/shard — segmentos, WAL, plan de prefetch/rescore
        lib/segment — payload/index/ID tracker/vector storage
          dense HNSW | sparse inverted | texto BM25
lib/edge — API embebida sobre shard/segment, no exige servidor/Raft
```

Lectura: consulta → hojas del plan → búsqueda por segmento → estimación de
cardinalidad → scan o grafo → rescore/fusión en su nivel adecuado → top-k.
Escritura observada: reserva de cola → WAL bajo lock → dispatch ordenado →
callback opcional. No equiparar callback de aplicación con barrera física.
Existe API Edge en este checkout: sería incorrecto describir todo Qdrant como
exclusivamente distribuido; tampoco se auditó Edge como sustituto embebido.

## 2. Evidencia y unidades extraíbles

**Código leído, enlaces al SHA local.** Rangos son la evidencia, no cobertura total.

| Ref. | Archivo y líneas | Observación |
|---|---|---|
| Q1 | [`lib/segment/src/index/hnsw_index/hnsw/read_view/dispatch.rs:40-190`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/index/hnsw_index/hnsw/read_view/dispatch.rs#L40-L190) | Exact desactiva cuantización; límites min/max deciden scan/grafo; zona incierta consulta estimación adicional |
| Q2 | [`lib/segment/src/index/struct_payload_index/read_view/filtering.rs:20-150`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/index/struct_payload_index/read_view/filtering.rs#L20-L150) | Estimaciones por campo/condición; IDs resueltos reutilizables; desconocido no se convierte en exacto |
| Q3 | [`lib/shard/src/query/planned_query.rs:13-93`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/shard/src/query/planned_query.rs#L13-L93) | Hojas de búsqueda/scroll/texto y rescore por shard o colección |
| Q4 | [`lib/segment/src/common/reciprocal_rank_fusion.rs:13-98`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/common/reciprocal_rank_fusion.rs#L13-L98) | Peso comprime posiciones; valida número de pesos; no desempata por ID |
| Q5 | [`lib/collection/src/shards/local_shard/shard_ops.rs:100-217`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/shard_ops.rs#L100-L217) | WAL y cola preservan orden; Completed/Acknowledged/WaitTimeout son estados distintos |
| Q6 | [`lib/collection/src/shards/local_shard/wal_ops.rs:21-74`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/wal_ops.rs#L21-L74) | Truncar pendientes detiene workers, cambia canal, drena y reinicia bajo fences |
| Q7 | [`lib/segment/benches/conditional_search.rs:20-180`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/benches/conditional_search.rs#L20-L180) | Criterion compara payload plano/estructurado y filter context |
| Q8 | [`src/actix/web_ui.rs:10-43`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/src/actix/web_ui.rs#L10-L43) | Servidor de estáticos; carpeta ausente impide habilitar dashboard |
| Q9 | [`lib/edge/src/lib.rs:1-54`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/edge/src/lib.rs#L1-L54) | Exporta EdgeShard, builders, lectura y variantes serverless feature-gated |

### Ficha candidata A — explicación de la decisión filtrada

- **Objetivo:** distinguir decisión basada en evidencia exacta de estimación incierta.
- **Mínimo mecanismo / fuente exacta:** transportar min/expected/max y motivo
  de ruta de Q1/Q2; no copiar umbral, muestreo ni conectividad del grafo.
- **Seam Vanta existente:** `src/cost_estimator` y
  [`src/sdk/search/vector.rs:157-200`](https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/sdk/search/vector.rs#L157-L200):
  ya hay prefilter, selectividad real, scan y máscara.
- **Tipos propuestos, no existentes acreditados:** `FilterEstimate { lower,
  expected, upper, origin }`, `RouteDecision { reason, route }`; ampliar una
  explicación existente antes que crear otro planner.
- **Acciones:** inventariar estimadores actuales; definir procedencia y ámbito
  namespace de los conteos; diseñar proyección diagnóstica sin cambiar ranking.
- **Dependencias:** índices payload sanos, tombstones/namespace, generación del
  índice y control de coste del diagnóstico.
- **FMEA:** conteo global con filtro por namespace induce plan erróneo; conteo
  obsoleto engaña al operador; explicación puede filtrar cardinalidad de otro tenant.
- **Aceptación funcional propuesta:** el motivo coincide con la rama tomada;
  unknown permanece unknown; el diagnóstico no cambia hits ni permisos.
- **No objetivos:** nuevos thresholds, tuning, prometer recall o QPS.

### Ficha candidata B — fronteras de aceptación, aplicación y persistencia

- **Objetivo:** que una confirmación de ingesta no prometa más de lo realizado.
- **Mínimo mecanismo / fuente exacta:** separar estados del receipt (Q5) y
  fencing de operaciones pendientes (Q6); no trasladar workers/Raft.
- **Seam Vanta existente:** [`src/wal.rs:999-1028`](https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/wal.rs#L999-L1028),
  `src/backend.rs`, APIs de inserción e índices derivados.
- **Tipos propuestos:** `WriteVisibility { accepted, applied_generation,
  durable_position }` con campos opcionales e información de backend;
  ausencia de posición durable no equivale a cero durable.
- **Acciones:** trazar ACK actual hasta WAL/backend/índices, documentar garantías
  por SyncMode, reutilizar receipts existentes si los hay y sólo luego proponer API.
- **Dependencias:** identidad estable de operación, propagación de errores de
  fsync, acuerdo de orden entre publicación vector/text/payload.
- **FMEA:** timeout tras commit produce retry duplicado; un flag applied antes
  de publicar todos los índices crea falsa read-after-write; truncar sin fence pierde trabajo.
- **Aceptación funcional propuesta:** no se anuncia durable sin barrera exitosa;
  resultado desconocido por timeout no se presenta como rollback.
- **No objetivos:** replicación, colas nuevas ni garantía de exactly-once.

### Ficha candidata C — contrato explícito de contribuciones híbridas

- **Objetivo:** hacer comparables explicaciones de dense/sparse/texto sin asumir
  que dos implementaciones llamadas RRF tienen semántica idéntica.
- **Mínimo / fuente:** tabla de rank/peso/contribución y nivel de fusión (Q3/Q4).
  Q4 usa `1 / ((position+1)/weight + k-1)`, no simplemente multiplicar
  `weight/(k+rank+1)`. Esa diferencia es contractual, no una recomendación de cambiar Vanta.
- **Seam:** [`src/sdk/search/fusion.rs:73-140`](https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/sdk/search/fusion.rs#L73-L140)
  y `HybridFusionReport`; híbrido de tres brazos ya existe.
- **Tipos propuestos:** extender reporte con `FusionContribution` y
  `FusionFormulaId`, no duplicar `HybridFusionReport`.
- **Acciones/dependencias:** revisar reporte actual y sparse, serialización SDK
  y desempate; explicar fórmula efectiva sin alterar scores.
- **FMEA:** ocultar truncamiento por brazo hace incompleta la explicación;
  serializar metadatos no autorizados expone memoria; pesos inválidos contaminan score.
- **Aceptación funcional propuesta:** suma reportada reproduce score base;
  empate conserva determinismo Vanta; campos sensibles siguen permisos.
- **No objetivos:** adoptar el default `k=2` de Qdrant ni sustituir el RRF actual.

## 3. Riesgos, defectos e issues transferibles

**Estático:** Q4 documenta que no desempata; resultados empatados no son una
fuente de orden total portable. No es por sí mismo un bug upstream.
**Hipótesis:** estimación obsoleta, timeout tras aplicación y publicación parcial
son modos de fallo a discriminar, no incidencias reproducidas.
**Issues upstream:** no consultados; no se adjudican CVEs, regresiones ni fixes.

**Seguridad/configuración:** [`config/config.yaml:389-446`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/config/config.yaml#L389-L446)
declara `host: 0.0.0.0` y claves comentadas. Es evidencia del fichero default,
no prueba del despliegue real ni auditoría de todos los middlewares.
La frontera crítica es cliente no confiable → API; no extrapolarla a Edge
sin red. Vanta ya tiene gate de autenticación fuera de loopback.

## 4. Prerrequisitos en VantaDB

Revalidar worktree dirty, receipts/diagnósticos actuales y auditorías ANN,
híbrido, WAL/backend antes de abrir tarea. El baseline
[METHOD](METHOD.md#vantadb-como-referencia) ya acredita HNSW/BM25/RRF y auditoría
textual. `src/backend.rs:34-55` declara payload/text/sparse como particiones.
`tests/durability_recovery.rs:30-80` comprueba reapertura después de drop:
su nombre «crash» no demuestra pérdida abrupta de proceso/energía.
Conservar validaciones existentes; no inventar una ausencia de WAL o persistencia.

## 5. Qué no trasladar

No importar sharding/consenso, estructura interna completa del payload index,
parámetros por defecto ni API Edge para resolver una explicación diagnóstica.
No confundir formato del WAL con garantía transaccional de todos los índices.
No portar fórmula RRF sin preservar contratos y comparabilidad histórica.

## 6. Método, licencia y cobertura

Sólo lectura estática; no builds, tests, servicios, benchmarks ni instalaciones.
Lectura de rangos Q1-Q9 y configuración citada, no auditoría integral.
Historial shallow, tags, LFS y submódulos no materializados según manifiesto.
Omitidos: consenso, GPU, optimizadores completos, IO/sparse kernels, RBAC
end-to-end, packaging de UI y fallos reales de almacenamiento.

Apache-2.0 del inventario y licencia raíz no certifican cada archivo/dependencia.
Revisar avisos y material derivado antes de reutilización; reescribir en Rust
no elimina obligaciones (el original ya es Rust).
Históricos OpenViking/ANN/WAL/backend de Vanta son antecedentes, pendientes de
revalidación sobre el estado concurrente; aquí no se declaran bugs actuales.

## 7. Benchmark upstream, no ejecutado

Q7 es un microbenchmark **Criterion** del payload, no un benchmark end-to-end ANN.
Entrada sintética: 100 000 puntos, seed 42, muestras de 1 000 IDs; filtros
must/match-any y rutas query-points/filter-context sobre plain/structured.
Mide iteraciones temporizadas y escribe tamaños medios de resultados/cardinalidad.
No produce por sí solo recall, coste cloud ni p50/p95/p99 de peticiones de servidor.
Necesita toolchain/dependencias Rust y almacenamiento temporal; profiling no
Windows condicionado por cfg. TempDir delimita limpieza normal, no cleanup
garantizado después de terminación abrupta; CPU/RAM/disco se deben registrar.
No hay embeddings, generación ni juez en ese runner.

`lib/segment/benches/hnsw_search_graph.rs` está inventariado, no leído como
segundo runner: no se le atribuyen métricas. Comparación futura sólo bajo
[protocolo del responsable](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se diseñaron harnesses, umbrales o ajustes. No hay superioridad medida.

## 8. Matriz frente al estado existente

| Capacidad | VantaDB | Delta defendible |
|---|---|---|
| ANN filtrado / scan selectivo | Existe: vector.rs:157-200 | Revisar explicación del grado de certeza, no añadir ANN |
| Dense + sparse + BM25/RRF | Existe: hybrid.rs:33-55 | Contrato de fórmula y contribuciones por brazo |
| Payload/text/sparse derivados | Existe: backend.rs:34-55 | Auditar conteos y generaciones, no asumir índice ausente |
| WAL y SyncMode | Existe: wal.rs:999-1028 | Verificar receipt aplicado/durable por backend |
| Auditorías ANN/híbrido/WAL/backend | Antecedentes existentes; alcance vigente parcial | Revalidación dirty obligatoria; no resetear hallazgos como nuevos |
| Fencing/receipt uniforme visible al SDK | No acreditado en esta lectura | Candidato B condicionado al inventario actual |
| Raft/sharding de Qdrant | No aplicable como requisito de memoria local | No propuesto |

## 9. Debilidades sugeridas, no bugs certificados

La cuestión útil no es «falta búsqueda híbrida», sino si un operador puede
reconstruir por qué se eligió una ruta y qué índices habían visto una escritura.
Auditar consistencia del vocabulario accepted/applied/durable y semántica de
pesos entre SDKs tiene más valor acotado que migrar el motor.
Prioridad condicional: B por honestidad contractual; A/C por inspección.
No se afirma que los contratos actuales fallen.

## 10. UI, escritorio e inspección

Código disponible de **servido** de dashboard: Q8; depende de carpeta estática.
README anuncia Web UI, pero esa declaración no demuestra que el frontend esté
incluido en este checkout. No se inspeccionó su repositorio separado ni se
localizó aplicación de escritorio nativa en el ámbito leído.
Vanta Studio ya dispone de diagnóstico retrieval en `develop-desktop`; A-C
serían datos para esa superficie, no la propuesta de crear otra UI.
