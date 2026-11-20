---
title: "Backlog de extracción — LanceDB"
kind: research
status: active
description: "Siete candidatas del informe LanceDB: coordenada de lectura, frescura de derivados, recibo de reemplazo, proyección por etapas, explain efectivo, disponibilidad de índice y reparación cercada"
tags: [extraccion, backlog, lancedb]
---

# Backlog de extracción — LanceDB

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/lancedb.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-LAN-01 | 🟡 | Coordenada de lectura estable para inspección compuesta | `src/sdk/search/hybrid.rs`, `src/sdk/search/explain.rs` | 🟡 | 🟠 P1 | ⬜ Candidata | `ReadCoordinate {scope, revisión}` con hidratación verificada: cada hit identifica namespace/key y revisión · Una actualización entre selección e hidratación no debe mostrar score de una revisión junto a contenido de otra; revisión inaccesible es conflicto tipado, no latest silencioso · Acción mínima: inventariar revisiones/snapshots del SDK, propagar coordenada por explain y validarla al hidratar · **Trazabilidad:** `informe §2 ficha A · evidencia rust/lancedb/src/table/dataset.rs:101-128,215-226` | X-LAN-04 la usa para multi-request; X-LAN-05 compone con detalle; converge con C4 de `CONSOLIDATED.md` | Revisión/snapshot accesible en SDK (inventariar); autorización del host; independiente de X-LAN-02/03 para diagnóstico read-only |
| X-LAN-02 | 🟠 | Frescura de derivado ligada a definición y revisión de origen | `src/wiki/ingestors.rs`, `vanta-memory/src/ingest/merge.rs`, `services/scheduler.rs::run_pass`, `src/cli_handlers/index.rs` | 🟠 | 🟠 P1 | ⬜ Candidata | `DerivedStamp {source_key, source_revision, processor_identity, output_digest}` con estado Current/Stale/Unknown · "Hay valor" no es "está fresco": edición de documento o cambio de extractor invalida la proyección · Acción mínima: stamp opcional en metadatos existentes, cálculo de estado en auditoría y recomputación idempotente que publica solo con origen en revisión esperada · **Trazabilidad:** `informe §2 ficha B · evidencia rust/lancedb/src/table/freshness.rs:4-21,47-110` | X-LAN-03 para publicación automática; X-LAN-07 la usa como auditoría previa; converge con C14 de `CONSOLIDATED.md` | Revisión de origen e identidad de procesador; auditoría read-only no bloqueada por X-LAN-03 |
| X-LAN-03 | 🔴 | Recibo de reemplazo con outcome y reconciliación por operación | `vanta-memory/src/core/record/l1_writer.rs:430-446` (delete-before-put), `ingest/merge.rs` | 🟠 | 🔴 P0 | ⬜ Candidata | `ReplaceIntent`/`ReplaceReceipt` con operation_id, expected_revision y outcome Applied/Rejected/Partial/Unknown · Tras timeout no se sabe qué se persistió: repetir append no es idempotencia y un fallo entre delete y put deja pérdida parcial · Acción mínima: inventariar frontera actual de put/delete, persistir intento/revisión y reconciliar por identidad en retry · **Trazabilidad:** `informe §2 ficha C · evidencia rust/lancedb/src/table/merge.rs:24-42,420-444, table/update.rs:87-130, table/delete.rs:12-61` | X-LAN-02 y X-LAN-07 publican por ella; converge con C1/C13 de `CONSOLIDATED.md` | Resolver garantías propias de reemplazo L1 (informe §4.1) o limitar receipt a parcial/desconocido |
| X-LAN-04 | 🟡 | Proyección por etapas y consumo acotado del retrieval | `src/sdk/search/hybrid.rs`, `src/sdk/search/explain.rs` | 🟡 | 🟡 P2 | ⬜ Candidata | `SearchProjection` explícita: campos omitidos no se representan como vacíos, vector no solicitado no se serializa y la materialización de candidatos se informa · Un stream de salida no constituye backpressure end-to-end · Acción mínima: inventariar dónde se hidrata hoy, ampliar opciones de respuesta y propagar cancelación liberando buffers · **Trazabilidad:** `informe §2 ficha D · evidencia rust/lancedb/src/query.rs:1517-1695, table/query.rs:334-425` | X-LAN-01 para hidratación coherente; X-LAN-05 informa materialización; converge con C7 de `CONSOLIDATED.md` | Soporte de transporte para cancelar; no requiere X-LAN-02/03 |
| X-LAN-05 | 🟡 | Procedencia de ejecución en el explain existente | `src/sdk/search/explain.rs` (`SearchExplanation`, `HybridFusionReport`), Studio `ExplanationHit` | 🟡 | 🟡 P2 | ⬜ Candidata | `ExecutionProvenance {route, revisión, stages, score_kind, degraded_reason}` como extensión opcional del reporte actual · Distancia aproximada no debe interpretarse como exacta ni el rerank del cliente atribuirse al motor · Acción mínima: obtener opciones resueltas donde ya se elige ruta y adjuntar procedencia a esa misma ejecución · **Trazabilidad:** `informe §2 ficha E · evidencia rust/lancedb/src/table/query.rs:334-425, query.rs:1458-1514, rerankers/rrf.rs:19-149` | X-LAN-01 comparte revisión; X-LAN-04/X-LAN-06 aportan rutas; converge con C7/C16 de `CONSOLIDATED.md` | Ruta efectiva inventariada por backend; metadata diagnóstica opt-in sin payload durable |
| X-LAN-06 | 🟡 | Disponibilidad de índice referida a revisión objetivo | `src/cli_handlers/index.rs` (`cmd_rebuild_index`), `src/sdk/search/explain.rs` | 🟡 | 🟡 P2 | ⬜ Candidata | `IndexObservation {index_identity, target_revision, observed_revision, state}` con estados pendiente/cubierto/no acreditable/error · "Índice existe", "job aceptado" e "índice cubre esa escritura" son preguntas distintas; sin watermark verificable se informa observación puntual · Acción mínima: extender el reporte existente con campos opcionales e identidad de operación/generación · **Trazabilidad:** `informe §2 ficha F · evidencia rust/lancedb/src/table.rs:3931-3943, index/waiter.rs:29-68, remote/table.rs:772-805 · issue lancedb#4402` | X-LAN-05 informa ruta; converge con C9/C16 de `CONSOLIDATED.md` | Revisión observable por backend y receipts de rebuild; permisos de mantenimiento |
| X-LAN-07 | 🟠 | Reparación parcial con intención y finalización cercada | `vanta-memory/src/services/scheduler.rs::run_pass`, `ingest/merge.rs`, `l1_writer.rs`, `utils/checkpoint.rs` | 🟠 | 🔵 P3 | ⬜ Candidata | `RepairIntent {scope, source_revision, processor, intent_id, selected_artifacts}` y `finish_repair` con conjuntos disjuntos applied/skipped · Un worker tardío no debe marcar limpio el artefacto nuevo ni los chunks que nunca procesó; skipped queda pendiente, nunca éxito sintético · Acción mínima: registrar intención en persistencia existente, verificar revisión al publicar y reconciliar por IDs estables tras interrupción · **Trazabilidad:** `informe §2 ficha G · evidencia rust/lancedb/src/table/freshness.rs:1451-1564,1633-1724` | X-LAN-02 (auditoría previa), X-LAN-03 (publicación); converge con C9/C14 de `CONSOLIDATED.md` | X-LAN-02 primero; X-LAN-03 solo para publicación activa; ownership único o mecanismo condicional por artefacto |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/lancedb.md` (§2 fichas A–G). Todas las interfaces son **hipotéticas** — propuestas para una spec posterior, no símbolos existentes ni traducciones de código; no fijan presupuestos ni umbrales. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\lancedb` (rama `main`) en SHA `bff93ff314eee34f625d05bd92b73d95867fe787` (rutas relativas al repo LanceDB); las rutas VantaDB citan el snapshot del informe y requieren revalidación contra HEAD. Prioridad del informe §1: 1=C, 2=B, 3=A, 4=D, 5=E; F es diagnóstico condicionado y G solo si se aprueba reparación parcial.

### X-LAN-01 — Coordenada de lectura estable para inspección compuesta

- **Problema:** un inspector recupera IDs y después hidrata texto; una actualización entre ambos pasos puede mostrar el score de una revisión junto al contenido de otra. El pin no reemplaza validación de permisos.
- **Contrato hipotético (tipos propuestos):** `ReadCoordinate { scope, logical_revision }`; `hydrate_at(coordinate, ids) -> Result<HydratedBatch, ReadConflict>`. Errores diferenciados: revisión no retenida, cambio concurrente, acceso denegado, IO — no usar «no existe» para todos.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table/dataset.rs:101-128,215-226` (`DatasetConsistencyWrapper::{new_query_snapshot, ensure_mutable}`) [LD]. Extraer el principio de un handle de lectura ligado a revisión, no su implementación con Lance.
- **Seam VantaDB:** `src/sdk/search/hybrid.rs` (`Embedded::hybrid_search`) y `src/sdk/search/explain.rs` (`explain_memory_search`) [V1][V2] ya producen retrieval y explicación; añadir coordenada verificable para consumidores de varios pasos si el contrato existente no la comunica.
- **Acciones:**
  1. Inventariar revisiones/snapshots actuales del SDK.
  2. Definir alcance por consulta frente a namespace.
  3. Propagar la coordenada por explain y detalle.
  4. Comprobar la revisión en hidratación.
  5. Tratar conflictos explícitamente en el consumidor; metadata de respuesta primero, sin migrar payloads (retención durable requeriría spec del motor aparte).
- **Dependencias:** revisión/snapshot accesible en SDK (inventariar); autorización del host; independiente de X-LAN-02/X-LAN-03 para diagnóstico read-only. X-LAN-04 la usa para multi-request; X-LAN-05 compone con detalle; converge con C4 de `CONSOLIDATED.md`. Alternativas: extender la respuesta existente primero; si no hay snapshot, validar versión al hidratar y devolver conflicto; rechazado guardar copia de toda la DB ni inventar branch por sesión.
- **FMEA:** cambio entre ID e hidratación → contenido/score mezclados (detectar mismatch de revisión → repetir toda la lectura o devolver conflicto); pin que admite merge → cambio inesperado (validar todas las mutaciones contra modo read-only); snapshot expirado → error tipado, no fallback a latest.
- **Criterios de aceptación propuestos:** consulta+detalle conserva revisión o falla de forma visible; actualización concurrente no mezcla; borrado/revocación impide mostrar contenido no autorizado; dos scopes con la misma key no se confunden; cancelación libera el handle. Observabilidad: contadores de conflictos y revisiones no retenidas, duración del handle — sin query/texto ni IDs sensibles en labels.
- **Criterio negativo:** una revisión inaccesible nunca se sustituye silenciosamente por latest; cada hit identifica namespace/key y revisión; la lectura no puede mutar por ningún método; la identidad estable del documento no cambia al cambiar revisión.
- **Independencia:** sí para diagnóstico read-only — no depende de X-LAN-02/X-LAN-03; rollback desactiva la lectura compuesta y vuelve a lectura única actual, sin «revertir» datos.
- **No objetivos:** time travel de usuario, branches ni promesa de snapshot global; un adapter Studio solo puede detectar mezcla, no fabricar aislamiento del motor.

### X-LAN-02 — Frescura de derivado ligada a definición y revisión de origen

- **Problema:** un embedding/resumen almacenado sigue presente tras editar el documento o cambiar el extractor; un cache hit por «hay valor» reutiliza una proyección obsoleta — «hay valor» no es «está fresco».
- **Contrato hipotético (tipos propuestos):** `DerivedStamp { source_key, source_revision, processor_identity, output_digest }`; `inspect_freshness(stamp, current) -> Freshness::{Current, Stale, Unknown}`. `Unknown` no significa fresco ni prueba corrupción.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table/freshness.rs:4-21,47-110` (firmas de entrada + definición + estado dirty) [LL]. No trasladar direcciones físicas de fragmentos Lance a keys Vanta.
- **Seam VantaDB:** `src/wiki/ingestors.rs` (chunks/provenance de `Ingestor::ingest` [V3]), `vanta-memory/src/ingest/merge.rs` [V6], `vanta-memory/src/services/scheduler.rs::run_pass` [V5] y auditoría textual existente en `src/cli_handlers/index.rs` [V7] — reportar frescura por artefacto en el sistema existente, no un segundo scheduler.
- **Acciones:**
  1. Inventariar stamps/checkpoints actuales.
  2. Separar identidad semántica y operativa.
  3. Incluir stamp opcional en metadatos existentes.
  4. Calcular estado en auditoría.
  5. Encolar recomputación idempotente.
  6. Publicar solo si el origen sigue en revisión esperada.
- **Dependencias:** revisión de origen e identidad de procesador; X-LAN-03 solo para publicación automática — la auditoría read-only no está bloqueada por ella. X-LAN-07 la usa como auditoría previa; converge con C14 de `CONSOLIDATED.md`. Alternativas: recomputar siempre (simple, costoso); invalidar por fuente completa; adapter externo único dueño de la derivación.
- **FMEA:** origen cambia durante el job → publicación stale (comparar revisión al commit y reencolar); modelo usa alias mutable → falso hit (registrar identidad resuelta si existe); política de dirty incompleta → falso limpio (auditoría cruzada y recomputación conservadora).
- **Criterios de aceptación propuestos:** cambio de texto/config invalida; no-op semántico no recalcula por metadata de ejecución; job interrumpido no publica stamp exitoso; borrado concurrente impide resurrección; fallo del procesador conserva error y origen. Observabilidad: motivos de invalidación, conteos current/stale/unknown, coste/tiempo por etapa — sin texto.
- **Criterio negativo:** la publicación del derivado queda ligada al origen que leyó; ningún stamp permite saltar authz; contenido borrado no se regenera desde una caché sin autorización; metadatos operativos no invalidan sin razón semántica.
- **Independencia:** parcial — la auditoría/inspección es independiente de X-LAN-03; la recomputación que publica escrituras la requiere. Migración: registros antiguos quedan `Unknown`, nunca etiquetados retrospectivamente `Current`.
- **No objetivos:** instalar Lance como segundo almacén de memoria; columnas computadas SQL; sidecars físicos de Lance; nueva jerarquía cognitiva.

### X-LAN-03 — Recibo de reemplazo con outcome y reconciliación por operación

- **Problema:** un agente reingiere una revisión tras timeout y necesita saber si reemplazó, falló antes de escribir o quedó parcial; repetir un append no equivale a idempotencia y un fallo entre delete y put deja pérdida parcial.
- **Contrato hipotético (tipos propuestos):** `ReplaceIntent { operation_id, scope, expected_revision, replacement_keys }`; `ReplaceReceipt { operation_id, committed_revision: Option<Revision>, outcome: Applied | Rejected | Partial | Unknown, counts }`; errores con fase y posibilidad de retry. Los tipos y variantes son diseño, no API actual.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table/merge.rs:24-42,420-444` (`MergeResult` y publicación de Dataset) [LF], `rust/lancedb/src/table/update.rs:87-130` (`UpdateResult`) [LG], `rust/lancedb/src/table/delete.rs:12-61` (`DeleteResult`) [LH]. Lo adoptable es el recibo con revisión y resultado, no el port del merge.
- **Seam VantaDB:** `vanta-memory/src/core/record/l1_writer.rs:430-446` (`write_l1_batch` borra targets antes de `put_batch` en la base revalidada [V8]) y `vanta-memory/src/ingest/merge.rs` (decisiones y errores por entrada [V6]); extender el contrato propio de reemplazo, no ejecutar semántica documental SQL dentro de dedup LLM.
- **Acciones:**
  1. Inventariar los límites actuales de `put_batch` y delete.
  2. Definir la frontera de reemplazo sin borrar evidencia antes de tener reemplazo durable.
  3. Persistir intento/revisión donde ya vive el estado de ingesta.
  4. Publicar el receipt después del commit.
  5. Reconciliar por identidad en retry.
  6. Reparar proyecciones post-commit mediante jobs existentes.
- **Dependencias:** resolver las garantías propias de reemplazo L1 (informe §4.1) o limitar el receipt a `Partial`/`Unknown`; operación estable, revisión/CAS o alternativa acreditada; no depende de ANN, X-LAN-02 ni UI. X-LAN-02 y X-LAN-07 publican por ella; converge con C1/C13 de `CONSOLIDATED.md`. Alternativas: reparación auditada tras fallo; preparar artefacto nuevo y cambiar referencia; transacción del motor si acredita la atomicidad requerida.
- **FMEA:** fallo tras delete y antes de put → pérdida parcial (detectar fase/targets y comparación canónico↔receipt → evitar delete-first o compensar con evidencia); respuesta perdida tras commit → retry duplicado (operation_id durable); crash antes del receipt → resultado desconocido (reconciliar datos, no marcar éxito por timeout).
- **Criterios de aceptación propuestos:** fallo antes/durante/después de publicación produce estados distinguibles; retry no duplica ni borra otra revisión; target concurrentemente actualizado genera conflicto; embeddings fallidos no eliminan la fuente; replay de logs no produce efectos externos dobles. Observabilidad: conteos por outcome/fase, pendientes sin resolver, duración de reconciliación — sin payload.
- **Criterio negativo:** la misma operación no aplica dos veces efectos de negocio; receipt de éxito solo tras frontera durable acreditada; `Unknown` nunca se convierte en «cero escrituras»; conteos diferencian insert/update/delete de operaciones internas.
- **Independencia:** sí como recibo honesto del contrato presente (mínimo declarado) — no afirmar atomicidad hasta acreditarla, no mezclar checkpoint de ejecución con memoria factual; rollback detiene nuevos reemplazos y mantiene lectura/reconciliación de intentos pendientes.
- **No objetivos:** compensación por reejecución ciega de delete (prohibido — compensación explícita por revisión); modificar la atomicidad del motor sin spec separada.

### X-LAN-04 — Proyección por etapas y consumo acotado del retrieval

- **Problema:** un inspector solicita pocos hits pero se recuperan todos los blobs/vectores o se materializan ambas ramas sin comunicarlo; un stream de salida no constituye backpressure end-to-end.
- **Contrato hipotético (tipos propuestos):** `SearchProjection { fields, include_vector, include_payload }` y `ProjectedHit { identity, revision, score, selected_fields }`; `hydrate_at` de X-LAN-01 solo si hay segundo paso. Errores: campo no permitido, revisión cambiada, límite/cancelación, IO.
- **Fuente de evidencia (clone):** `rust/lancedb/src/query.rs:1517-1695` (`run_hybrid` drena y concatena candidatos) [LI]; `rust/lancedb/src/table/query.rs:334-425` (scanner proyecta columnas y aplica límite por batch) [LJ]. La unidad es un plan de selección y posterior hidratación, no un motor Arrow nuevo.
- **Seam VantaDB:** `src/sdk/search/hybrid.rs` (ya separa presupuesto de candidatos de top-k [V1]) y `src/sdk/search/explain.rs` (ya comunica ranking [V2]); proyección acotada en SDK/bridge y reutilizar el resultado para abrir detalle en Studio [S1].
- **Acciones:**
  1. Medir dónde se hidrata hoy (sin ejecutar durante esta investigación).
  2. Inventariar la selección ya existente.
  3. Ampliar opciones/serialización compatibles.
  4. Separar candidatos y detalle.
  5. Propagar cancelación al productor y liberar buffers.
  6. Informar la materialización cuando ocurra.
- **Dependencias:** X-LAN-01 para multi-request coherente — no para proyección en una sola respuesta; soporte de transporte para cancelar; no requiere X-LAN-02/X-LAN-03. Converge con C7 de `CONSOLIDATED.md`. Alternativas: proyección de respuesta en adapter; proyección física en motor solo si la medición justifica la complejidad; recortar JSON después de cargar todo no equivale a reducir IO.
- **FMEA:** campo omitido interpretado como borrado → re-put pierde datos (distinguir no solicitado de ausente); stream cancelado retiene buffers (detectar tareas vivas/RSS → liberar al cancelar); hidratación N+1 → latencia/coste (batch por identidad y revisión).
- **Criterios de aceptación propuestos:** misma consulta con distinta proyección conserva ranking; vector no solicitado no se serializa; cancelación cierra recursos; permisos/revisión se revalidan al detalle. Medir bytes/filas/batches por fase y cancelaciones; no prometer ahorro hasta medición.
- **Criterio negativo:** los campos omitidos nunca se representan como valor vacío existente; orden/score/identidad no cambian por proyección; la autorización se aplica antes de devolver incluso metadata; nunca devolver campos prohibidos para «hacer fallback».
- **Independencia:** sí para proyección en una sola respuesta — opción ausente conserva comportamiento, sin cambio de datos ni reindexación; X-LAN-01 solo se requiere si se compone lectura multi-request.
- **No objetivos:** paginación física nueva ni librería Arrow en el core; afirmar mejora de RSS/latencia sin medir.

### X-LAN-05 — Procedencia de ejecución en el explain existente

- **Problema:** un operador interpreta distancia aproximada como exacta o atribuye al motor un reranking que solo aplicó el cliente; plan solicitado y ejecutado son cosas distintas.
- **Contrato hipotético (tipos propuestos):** extensión opcional `ExecutionProvenance { route, revision, stages, score_kind, degraded_reason }` del reporte actual. Errores de «explicación no disponible» distintos de búsqueda fallida.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table/query.rs:334-425` (scanner y opciones efectivas) [LJ]; `rust/lancedb/src/query.rs:1458-1514` (semántica de refinamiento) [LO]; `rust/lancedb/src/rerankers/rrf.rs:19-149` (unión RRF por row ID) [LK]. Extraer la diferencia plan solicitado/ejecutado y score de etapa, no otra fórmula de ranking.
- **Seam VantaDB:** `src/sdk/search/explain.rs` (`SearchExplanation` y `HybridFusionReport` existentes [V2]); Studio ya recibe `ExplanationHit` [S1]. Ampliar solo datos que una ruta efectiva no comunique, conservando los tipos existentes.
- **Acciones:**
  1. Inventariar el reporte actual por backend.
  2. Obtener las opciones resueltas donde ya se elige ruta.
  3. Adjuntar la procedencia a esa misma ejecución.
  4. Serializar opcionalmente.
  5. Mostrar «no disponible» en Studio si el transporte no soporta detalle.
- **Dependencias:** ruta efectiva inventariada por backend; X-LAN-01 para revisión común si se compone con detalle; metadata diagnóstica opt-in sin payload durable. Converge con C7/C16 de `CONSOLIDATED.md`. Alternativas: documentación de defaults basta para algunos callers; rechazado un segundo explain que ejecute otra consulta y se presente como traza de la primera.
- **FMEA:** explain que recalcula con otra revisión → relato falso (correlación por ejecución); scores incompatibles sumados sin etiqueta → diagnóstico engañoso (tipos de etapa y conservación de originales); alta cardinalidad en métricas → coste/fuga (labels de categoría, detalles sujetos a permisos).
- **Criterios de aceptación propuestos:** vector-only/keyword/hybrid/sparse y top-k cero describen su ruta real; fallo de etapa no aparece como cero hits exitoso; no hay cambios de orden al activar el reporte. Observabilidad: disponibilidad de explain por backend, fallos de reporte y rutas — sin texto.
- **Criterio negativo:** el score conserva semántica/unidad — nunca se normaliza para aparentar probabilidad; traza y hits pertenecen a una misma ejecución; la degradación no se oculta; no generar explicaciones con LLM.
- **Independencia:** sí — extensión opcional del reporte existente; desactivar el reporte ampliado no cambia ranking; desconocidos tolerados por lectores anteriores.
- **No objetivos:** nuevos pesos, nprobes ni thresholds Vanta; recrear explain/RRF que ya existen.

### X-LAN-06 — Disponibilidad de índice referida a revisión objetivo

- **Problema:** «dato guardado», «job aceptado» e «índice alcanzó esa escritura» son preguntas diferentes; mostrar solo que el índice existe induce decisiones erróneas.
- **Contrato hipotético (tipos propuestos):** `IndexObservation { scope, index_identity, target_revision, observed_revision, state }`; `state` distingue pendiente, cubierto, no acreditable y error. `observe_index(target)` es read-only; si el backend no expone una revisión comparable debe decirlo, no inferirla del conteo de filas. No usar versión L1 como offset físico del índice.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table.rs:3931-3943` (`NativeTable::create_index` prepara y construye; async prepara antes de `tokio::spawn` y devuelve `Job`) [LT]; `rust/lancedb/src/index/waiter.rs:29-68` (`wait_for_index`: polling hasta `num_unindexed_rows==0`) [LU]; `rust/lancedb/src/remote/table.rs:772-805` (`submit_create_index` extrae job ID y espera esa submission) [LAA]; issue [lancedb#4402](https://github.com/lancedb/lancedb/issues/4402) (job ID esperado — hay corrección cliente concreta en este SHA, no atribuir el defecto histórico indiscriminadamente).
- **Seam VantaDB:** `src/cli_handlers/index.rs` (`cmd_rebuild_index` ya informa vectores, tombstones y duración [V7:14-68]) y `src/sdk/search/explain.rs` [V2]; reusar scheduler [V5] si hace falta espera de tarea — comunicar cobertura respecto a la escritura que interesa, no añadir índices ni otro job manager.
- **Acciones:**
  1. Inventariar los receipts reales de creación/rebuild.
  2. Extender el reporte existente con campos opcionales e identidad de operación/generación.
  3. Definir la comparación por backend.
  4. Propagar el target al inspector.
  5. Separar espera cancelable de creación/rebuild.
- **Dependencias:** revisión observable por backend y receipts de rebuild; permisos de mantenimiento. X-LAN-05 informa ruta; converge con C9/C16 de `CONSOLIDATED.md`. Alternativas: conservar el reporte puntual actual si no hay delta; lectura fallback explícita por backend frente a reconstrucción indiscriminada.
- **FMEA:** polling «latest» con tráfico continuo → inanición (target fijo); índice recreado con el mismo nombre → cobertura falsa (identidad de generación); timeout → retry de importación → duplicados (separar comandos; un timeout local no es una cancelación remota confirmada ni un error de construcción un timeout genérico).
- **Criterios de aceptación propuestos:** índices ausente/parcial/listo y generación reemplazada se distinguen; nuevas escrituras no cambian el target; cancelar la espera libera recursos sin prometer rollback; sin watermark verificable no aparece «cubierto». Solo contadores de estados, sin texto/vectores.
- **Criterio negativo:** un índice anterior con el mismo nombre no satisface la nueva intención; `job_done` no acredita por sí solo la cobertura de escrituras posteriores; sin watermark verificable se informa observación puntual, nunca garantía causal.
- **Independencia:** sí — primera entrega solo diagnóstico, sin migración ni retención de snapshots; propuesta condicionada del informe, no requisito para adoptar X-LAN-01/04/05.
- **No objetivos:** SLO inventado, segundo scheduler ni promesa de ANN más rápido; no copiar un bucle de polling global.

### X-LAN-07 — Reparación parcial con intención y finalización cercada

- **Problema:** un job recalcula embeddings por partes mientras cambia la fuente o corre otro reparador; el worker antiguo no puede marcar limpio el artefacto nuevo ni los chunks que nunca procesó.
- **Contrato hipotético (tipos propuestos):** `RepairIntent { scope, source_revision, processor_identity, intent_id, selected_artifacts }`; `finish_repair(intent, applied, skipped) -> Result<RepairReceipt, RepairConflict>`. Los conjuntos son disjuntos y subconjuntos de `selected`; `skipped` queda pendiente o fallido explícito, nunca éxito sintético.
- **Fuente de evidencia (clone):** `rust/lancedb/src/table/freshness.rs:1451-1564` (`begin_row_repair`) [LV] y `:1633-1724` (`finish_row_repair`: valida pertenencia, revisión opcional, identidad de intención e inputs actuales; `verified_version=None` omite el fence de versión según contrato — no equivale a CAS) [LW]. Unidad distinta del stamp de X-LAN-02 y del recibo general de X-LAN-03.
- **Seam VantaDB:** `vanta-memory/src/services/scheduler.rs::run_pass` [V5], `ingest/merge.rs` [V6], `l1_writer.rs` [V8] y `utils/checkpoint.rs` [V9] — existen trabajos y progreso; no está acreditado en la muestra que su completion cercado identifique cada derivado seleccionado.
- **Acciones:**
  1. Primero la auditoría de X-LAN-02.
  2. Registrar la intención en la persistencia existente.
  3. Calcular fuera de locks.
  4. Verificar la revisión al publicar cada resultado.
  5. Finalizar solo evidencias publicadas.
  6. Reconciliar tras interrupción mediante IDs estables.
- **Dependencias:** X-LAN-02 (auditoría previa) y X-LAN-03 solo para publicación activa; ownership único o mecanismo condicional por artefacto — el UUID evita ABA de intenciones iguales pero no sustituye la atomicidad de comparación+publicación (no implementar CAS con RMW de [V9]). Converge con C9/C14 de `CONSOLIDATED.md`. Alternativas: recomputar toda la fuente con único escritor (más simple, preferida hasta justificar reparación parcial); adapter con ownership único sin extender el motor.
- **FMEA:** worker tardío → limpia revisión incorrecta (mismatch de token); subset forjado → oculta trabajo incompleto (validar pertenencia); crash tras escribir → retry duplicado (reconciliar por artefacto/revisión); revocación/borrado → resurrección (revalidar permisos y vigencia al publicar).
- **Criterios de aceptación propuestos:** dos workers, fuente cambiada, token obsoleto, cancelación, resultado saltado y duplicado — ninguno limpia trabajo ajeno; fallo de stamp no oculta dato committed (traza de refresh del informe §1); señal de reparación pendiente sin contenido sensible. Casos no ejecutados.
- **Criterio negativo:** un resultado omitido o saltado nunca se presenta como reparación completa; un worker no marca limpio un artefacto que no procesó ni una revisión distinta de la suya.
- **Independencia:** condicionada — el informe la justifica solo si se aprueba reparación por partes; depende de X-LAN-02 primero y de X-LAN-03 para publicación; rollback detiene publicación y conserva dirty sin borrar el origen ni declarar clean.
- **No objetivos:** transacción distribuida, traductor de expresiones, ejecución de funciones ajenas ni nuevo motor de artefactos; no trasladar sidecars, Roaring físico ni formato Lance.
