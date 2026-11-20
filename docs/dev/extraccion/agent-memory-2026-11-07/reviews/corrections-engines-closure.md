---
title: "Reconfirmación independiente de correcciones de motores y RAG"
kind: research
status: proposed
description: "Cierre selectivo documental de ocho reworks, con ledger por eje, contraste de fuentes y residuales explícitos"
tags: [research, memory, engineering]
---

# Reconfirmación independiente de correcciones de motores y RAG

## Alcance y trazabilidad

Revisor independiente de la autoría correctiva. **No es aceptación final ni
certificación runtime.** Se leen íntegros los ocho informes y sus diffs; el
contraste de fuentes es dirigido a mecanismos, cambios y hallazgos previos,
no una auditoría exhaustiva de los repositorios.
Normativa: [contrato](../EXPANSION-CONTRACT.md),
[referencia Vanta](../VANTA-REFERENCE.md), [método](../METHOD.md) y
[notas de benchmarks](../BENCHMARK-NOTES.md).
Antecedentes: [engines](engines.md) y
[engines-rag-remaining](engines-rag-remaining.md). Faiss queda fuera.

Se comprobó que el rework ya existe antes de organizar esta reconfirmación.
Base real: `%TEMP%\vantadb-fix-engines-base\`; recibo del autor:
`%TEMP%\vantadb-fix-engines.md`; diff:
`%TEMP%\vantadb-fix-engines.diff`, SHA-256
`34b0df6c1eae79e05c9199b35f3ea2ba5c7c9934a074c6c845ecb50aa6ee215c`.
La reconstrucción en memoria con `difflib.unified_diff` base→documentos actuales
coincide exactamente con ese diff. Los ocho hashes de base coinciden con el
recibo. Es comprobación documental, no medición de un motor.

Motor Vanta fijado a `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; Studio
a `0ad38c542b39a589035b0e17fdedb4c09dca24b9`. HEAD de ambos y de los
ocho clones comprobado, sin fetch/pull. Toda ruta Studio inmutable se resuelve
en `feature-freez/desktop/`, no en los archivos AM de `desktop/`.
Los rangos de informe son los de esta lectura; los rangos de código se
vinculan al pin indicado por proyecto.

**Estados:** cerrado = corrección documental contrastada; residual = límite
acotado que no invalida el cierre indicado; no resuelto = defecto documental
que aún impide cerrar ese hallazgo. «Sustentado» en el ledger significa
evidencia suficiente en este alcance estático, nunca funcionalidad certificada.
No se impone runtime a todas las propuestas ni se aprueban sus contratos.

## Índice

- [Qdrant](#qdrant)
- [Redis](#redis)
- [TiDB](#tidb)
- [SurrealDB](#surrealdb)
- [LlamaIndex](#llamaindex)
- [GraphRAG](#graphrag)
- [RAGFlow](#ragflow)
- [Quivr](#quivr)
- [Cierre FMEA, matriz y UI](#cierre-fmea-matriz-y-ui)
- [Cierre y límites](#cierre-y-límites)

## Qdrant

**Recibo:** [qdrant.md](../qdrant.md), leído completo, 1–682;
diff real 1–107. Pin `222b2e44ab1c3f17e8246396ead6558e45041df7`.
**Veredicto propuesto: cerrar Q-R1 y Q-R2 documentalmente.**
No requiere reproducir el caso para cerrar la omisión narrativa; sí conservar
pendiente su alcanzabilidad antes de atribuir un defecto operativo.

| Hallazgo previo | Estado | Cambio y contraste independiente |
|---|---|---|
| Q-R1, pérdida de estado sin timeout | Cerrado; residual de alcanzabilidad | Informe 131–146, 226–229, 295–298, 677–679. `await_update_result` descarta `InternalUpdateResult` en rama sin timeout y fabrica Completed; con timeout preserva `op_num/status`. Worker deferred envía WaitTimeout dentro de `Ok`, no sólo como error del canal. |
| Q-R2, aceptación sin representación en DTO | Cerrado | Informe 32–55, 218–221, 277–298, 603. Campo `acceptance: Accepted/Rejected/Unknown` separado de commit/durabilidad/proyecciones; aceptación conocida con commit desconocido, primario synced con ANN pendiente y timeout tras dispatch explícitos. `outcome` no reintroduce escalera excluyente. |

Fuente discriminante releída:
[shard_ops.rs:165–218](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/shard_ops.rs#L165-L218)
y
[update_worker.rs:181–244](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/update_workers/update_worker.rs#L181-L244).
Condiciones observadas: `wait_for_deferred && prevent_unoptimized`, sender y
fallo de espera. No se trazaron todos los callers/configuraciones para demostrar
la conjunción en ejecución. No se infiere pérdida durable ni bug Vanta.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §1/1.2; WAL, reserva, callback, fence y worker separados; asimetría ahora presente. |
| 1 Recomendaciones | Sustentado | §1/4; explicación y contratos, no sustitución ANN ni clúster. |
| 2 Fichas Rust | Sustentado | A–E/precisiones, fuentes, once campos, FMEA/rollback/aceptación; DTO B explícitamente propuesto. |
| 3 Incidencias | Sustentado con límites heredados | §3 #9575/#10851/#9869; revisión anterior leyó páginas y reporte; rework no altera su estado. No se recertifican discusión completa, PoC ni merge/release. |
| 4 Prerrequisitos | Sustentado | V1–V7/§4; A/C independientes de B; WriteReceipt y rebuild existentes reconocidos. |
| 5 Rechazos | Sustentado | §5; no Raft, defaults, sidecar durable o Edge por semejanza nominal. |
| 6 Método/licencia | Sustentado | §6/11; Apache de componentes separado de transitivas y frontend. |
| 7 Benchmarks | Sustentado | §7; población total vs 1 000 offsets y división entera corroboradas en `conditional_search.rs:30–115`; Q14 plain no equivale a top-10. No resultados propios. |
| 8 Matriz | Sustentado | §8, aceptación independiente añadida; no niega híbrido, filtros, certificados, delete ni Studio. |
| 9 Negativos Vanta | Sustentado/acotado | §9 mantiene explain ejecutor y reportes parciales, no resurrección/corrupción Vanta. |
| 10 UI | Sustentado en alcance declarado | §10 acredita servido web, no fuente frontend externa ni app nativa; Studio es destino existente, no UI nueva certificada. |

Contraste Vanta adicional:
[memory.rs:1223–1301](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1301):
`put_certified` construye atestación tras `put`; no acredita por sí solo C13.
Commit por chunk no es rollback batch-wide. El rework conserva estas
distinciones en matriz, FMEA y cierre. Residuales: backend/dominio durable,
frontend separado y propagación futura del contrato; ninguno es una nueva
exigencia de fsync de índices reconstruibles.

## Redis

**Recibo:** [redis.md](../redis.md), leído completo, 1–574; diff 108–124.
Pin `5b69ba2ce91fea8e9fc70ebac8ce341b37e75533`.
**Veredicto propuesto: cerrar el hallazgo Studio; conservar límites de UI
externa, no convertirlos en certificación ni bloqueo runtime universal.**

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| P2 identidad inmutable Studio | Cerrado | Informe 520–531 reemplaza `desktop/...` por `feature-freez/desktop/...`, SHA completo y rangos. Releídos cuerpos `git show`: package 1–56 y tauri.conf 1–38 contienen React/Vite/Tauri, ventana y CSP. No se usa JobChip/jobs ni bytes AM como prueba del pin. |

Fuentes Studio:
[package.json:1–56](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/package.json#L1-L56),
[tauri.conf.json:1–38](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src-tauri/tauri.conf.json#L1-L38).
Evidencia de shell/producto y configuración, no de integración de receipts
ni seguridad de todas las conexiones.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0; entrada, PEL, efecto y fsync no confundidos; claim no excluye ejecución externa. |
| 1 Recomendaciones | Sustentado | §1; delta condicionado al inventario, sin eviction del canon ni segundo broker. |
| 2 Fichas Rust | Sustentado | 2.1–2.5: once campos, ownership, FMEA, migración y rollback; resolución de efecto no duplica C13. |
| 3 Incidencias | Sustentado con límites heredados | §3 #13956/#5177, mecanismo/versión y ausencia de fix acreditado. No se reconsulta aquí su estado ni se equipara issue abierto a bug Vanta. |
| 4 Prerrequisitos | Sustentado | §4 por candidata; RMW de checkpoint no es bloqueo universal. |
| 5 Rechazos | Sustentado | §5; no servidor/clúster, ACK=exactly-once ni C→Rust como escape legal. |
| 6 Método/licencia | Sustentado | §6/11; triple licencia raíz separada de deps, IDMP/RDB/replay no auditados. |
| 7 Benchmarks | Sustentado | §7, releído `redis-benchmark.c:445–488`: primer read excluye parseo, errores abortan; población no es QA y adaptación clave→namespace/key delimita semántica de ACK. |
| 8 Matriz | Sustentado | §8; scheduler, TTL, índices, cuotas/gobernador y recall existentes; cola local no declarada durable. |
| 9 Negativos Vanta | Sustentado/acotado | §9; handler→complete separado y race condicionada a caller, no duplicación/crash reproducidos. |
| 10 UI | Corregido; cobertura externa parcial declarada | §10; Studio pin reproducible. RedisInsight package/documentación no acreditan implementación browser/secretos; esa laguna sigue explícita. |

Contraste adicional:
[t_stream.c:4248–4305](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/t_stream.c#L4248-L4305)
retira NACK, no demuestra efecto externo;
[replication.c:4950–4989](https://github.com/redis/redis/blob/5b69ba2ce91fea8e9fc70ebac8ce341b37e75533/src/replication.c#L4950-L4989)
comprueba offsets fsynced para WAITAOF;
[local_backend.rs:235–290](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/local_backend.rs#L235-L290)
ya compara owner bajo mutex. La propuesta añade identidad/revisión del efecto
donde proceda, no «fencing desde cero». FMEA y matriz preservan el límite.
Residual documental explícito: RedisInsight sólo leído a nivel package y
documentación, no fuente funcional de su browser. No se solicita código nuevo.

## TiDB

**Recibo:** [tidb.md](../tidb.md), leído completo, 1–556; diff 125–169.
Pin `41efd866c3801a637ab2d6712b86d8b2cd77ac5b`.
**Veredicto propuesto: cerrar los dos P2 previos documentalmente.**

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| Cadena KV atribuida al driver incorrecto | Cerrado | Informe 56–74 y TD 547: Set/Delete delegan `KVTxn.Set/Delete`; Get llama `GetMemBuffer().Get`, fallback a snapshot sólo con `ErrNotExist`, valor vacío exitoso→`kv.ErrNotExist`. No conserva `GetLocal` ni fallback `KVTxn.Get` ficticios. |
| Studio mal anclado | Cerrado | Informe 503–513 usa cuerpos/rangos `feature-freez/desktop` del pin; cotejo común Redis. No atribuye archivos movidos/modificados al commit. |

Relectura directa del
[driver:195–218](https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/store/driver/txn/txn_driver.go#L195-L218)
confirma ramas y conversión de errores. La delegación no demuestra implementación
del buffering cliente, fsync, consenso ni cancelación remota.
No hay `doc.go` en los dos paquetes discriminantes consultados (txn/memory).

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0/TD; traza del driver corregida, commit/rollback incierto separados. |
| 1 Recomendaciones | Sustentado | §1; recursos y cancelación locales, no SQL/TiKV/Raft. |
| 2 Fichas Rust | Sustentado | A/B/precisiones/C incluyen ownership, errores, pasos, migración efímera/aditiva, FMEA y aceptación; RAII propuesto, no API existente. |
| 3 Incidencias | Sustentado con límites heredados | §3 #72234; registro/unbind distinto de uso correcto; no PR mergeada ni bug Vanta inferidos. Estado no reconsultado aquí. |
| 4 Prerrequisitos | Sustentado | §4; A/B/C independientes y condicionados; bytes no son presupuesto contextual. |
| 5 Rechazos | Sustentado | §5; no panic FFI, cuotas heredadas ni plataforma distribuida. |
| 6 Método/licencia | Sustentado | §6/11; Apache y componentes externos/enterprise separados; no perfil build/test para este análisis read-only. |
| 7 Benchmarks | Sustentado | §7 y archivo completo de 42 líneas releído: errores MemTotal/MemUsed descartados; Consume cuenta 256 MiB sin asignarlos, RunParallel/b.N no equivalen a RSS/QA. |
| 8 Matriz | Sustentado | §8; gobernador, cuotas configuradas, híbrido y scheduler reconocidos; C13 propuesto. |
| 9 Negativos Vanta | Sustentado/acotado | §9; AtomicU64 depende de balance externo, underflow sólo condicionado; CancellationRegistry existente no se niega. |
| 10 UI | Corregido; frontend externo no auditado | §10; Dashboard documentado como web en PD, HTTP/trace local real; Studio pin válido, no ejecución ni importación de dashboard. |

Benchmark:
[memory/bench_test.go:1–42](https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/bench_test.go#L1-L42).
Se conserva adaptación de scope/reserva al gobernador existente y separación de
coste contable frente a buffers reales, sin runner/umbrales nuevos.
Residual: frontend Dashboard y camino remoto de TiKV no inspeccionados; no son
prueba de ausencia de UI ni condición universal para A/B/C.

## SurrealDB

**Recibo:** [surrealdb.md](../surrealdb.md), leído completo, 1–698;
diff 170–407. Pin `9577ac5f0caa60a39d5a334e9352aed0e232f9e6`.
**Veredicto propuesto: cerrar documentalmente los cuatro hallazgos previos.**
La restitución no se acepta por tener enlaces: se contrastaron cuerpos de
builder/transacción, benchmark, seams Vanta y flujo Surrealist.

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| P1 trazabilidad/cierre truncado | Cerrado | Informe 193–211, 638–698 restituye §11, inventario SCFG–STACTIONS, seams y V1/V2 al baseline real. A/B/C/D tienen dependencias individuales 490–495 y matriz 585–598 incorpora C/D/Studio. |
| P1 incidencias anunciadas sin contenido | Cerrado; residual de fix/release | 442–478 documenta #7552/#7523, mecanismo, revisión/configuración, estado y aplicabilidad. El comentario de fix no se convierte en prueba histórica de PR/release ni en defecto Vanta. |
| P2 benchmark operativo/adaptación | Cerrado | 528–581; runner completo contrastado: abortos, población `samples.len()`, mediana superior, store/caché compartidos, preparación excluida y mapping real `Embedded::get`. No se delega la correspondencia al método. |
| P2 UI sólo imagen | Cerrado en flujo inspeccionado | 610–636, 693–698: Surrealist externo al SHA `65b7b95109177deb486cc6254ed203fce257fddf`; QueryView→editor→executeUserQuery→frames/resultados/error/finally contrastados, no sólo package/README. Studio ActivityPanel al pin correcto. |

**Contraste de cuerpos, no presencia de referencias:**

- [builder.rs:4049–4123](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/index/builder.rs#L4049-L4123)
  comprueba abort/done, registra prueba por generación, renueva ownership y
  reinicia cursor si no hay prueba. No demuestra todas las intercalaciones.
- [tx.rs:3384–3507](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L3384-L3507),
  [:7856–7873](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L7856-L7873)
  y [:12734–12750](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/tx.rs#L12734-L12750):
  error primario preservado, outcome incierto, wakeups postcommit y callbacks
  fuera del mutex. Wakeup no equivale a ejecución ni fsync.
- Vanta [admin.rs:16–35](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/admin.rs#L16-L35),
  [impl_rebuild.rs:16–95](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_rebuild.rs#L16-L95),
  [migration.rs:18–85](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/migration.rs#L18-L85)
  y [get:1611–1625](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1611-L1625):
  rebuild/migrador ya existen; lookup devuelve `Option<MemoryRecord>`, no SQL.
- Surrealist [QueryView:125–199](https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/screens/surrealist/pages/Connection/views/query/QueryView/index.tsx#L125-L199),
  [editor:43–78](https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/editor/query.tsx#L43-L78)
  y [connection:331–550](https://github.com/surrealdb/surrealist/blob/65b7b95109177deb486cc6254ed203fce257fddf/src/screens/surrealist/pages/Connection/connection/connection.tsx#L331-L550)
  releídos por raw público. Error por sentencia y falta de respuesta son
  resultados explícitos; `finally` limpia activo, no deshace escrituras.

**Consulta pública de reconfirmación:** el informe consigna #7552 cerrado; su
[comentario 6283251799](https://github.com/surrealdb/surrealdb/issues/7552#issuecomment-6283251799)
afirma fix en main y test de concordancia, sin identificar SHA/PR.
[#7523](https://github.com/surrealdb/surrealdb/issues/7523), cuerpo completo por
REST, está abierto/actualizado 2026-11-05: TiKV y estado heredado, no receta
fiable para entrar en `cleaning` en 3.2.4; cambiar campo y repetir su valor
discriminan caminos distintos. No se consultan otros issues citados como fixes.
Se leyó completo el comentario por REST; la reconsulta de detalle de #7552
devolvió después **403**, por lo que no se recertifica aquí su estado completo
ni se declara lectura completa del PoC o reproducción.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0/2/refs; commit incierto, postcommit, generación y snapshot separados. |
| 1 Recomendaciones | Sustentado | §1; diagnóstico primero, barrera condicionada, B independiente. |
| 2 Fichas Rust | Sustentado | A–D con once campos, seams y FMEA; C puede declarar desconocido sin implementar A, D no obliga nuevo outbox. |
| 3 Incidencias | Sustentado/acotado | §3 ahora contiene dos reportes concretos; #7523 y comentario contrastados, reconsulta #7552 parcial por 403; fix/release y PoC pendientes. |
| 4 Prerrequisitos | Sustentado | §4 tabla individual por candidata, no bloqueo global por UI/multinodo/C13. |
| 5 Rechazos | Sustentado | §5; no SurrealQL/plataforma, leases distribuidos ni BSL como Apache vigente. |
| 6 Método/licencia | Sustentado | §6/11 sincronizados con consultas reales y límites; BSL motor distinta de MIT cliente, transitivas pendientes. |
| 7 Benchmarks | Sustentado | `point_read.rs:1–152` releído completo; muestras/error/cache y colapso de variantes SQL al get SDK explicitados. |
| 8 Matriz | Sustentado | §8 cubre A/B/C/D/Studio; no niega rebuild, migraciones ni híbrido. |
| 9 Negativos Vanta | Sustentado/acotado | §9 preguntas de frontera/compatibilidad; sin pérdida concurrente o ausencia global inventadas. |
| 10 UI | Sustentado en flujo fijado | §10; Surrealist con código y Studio `ActivityPanel.tsx:1–90` releído con `git show`; auditEvents/filtros/error, no jobs del worktree AM. |

Residuales: causalidad histórica exacta y release del fix #7552, recovery TiKV
del estado antiguo #7523, todos los callers/Drop/backends y seguridad/transitivas
UI. §11 los conserva; no impiden cerrar las omisiones documentales corregidas.

## LlamaIndex

**Recibo:** [llama-index.md](../llama-index.md), leído completo, 1–944;
diff 408–565. Pin `444fdf0684707de62ebb447d658dba137dd9cb99`.
**Veredicto propuesto: cerrar los tres hallazgos documentales previos.**
Los negativos estáticos del adapter siguen siendo negativos del producto
observado, no defectos que este rework documental pretenda reparar.

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| P1 L20–L23/V13–V17 sin definiciones/matriz incompleta | Cerrado | Informe 165–257, 555–602, 811–813, 874–879, 922–930. Fuentes reales navegables; matriz incluye fidelidad del adapter y spill/cursor existentes. Releídos jerarquía/envelope, archivo SQL, adapter, tests, SpillSink y offload RMW. |
| P2 receipt de etapa sin C13 | Cerrado | 474–505: D es observación por etapa/receptor; varias observaciones no son commit conjunto. Referencia C13 propuesto conserva aceptación/commit/durabilidad/proyecciones independientes; Applied de transform no prueba commit, PendingProjection puede coexistir con primario durable y otro receptor fallido. |
| P2 Studio incompatible con SHA | Cerrado | 849–860, 921 usa `feature-freez/desktop/.../MemoryLens.tsx:1–90`; cuerpo del pin releído. Imports/escenas/error y comentario de navegación, no bytes AM ni JobChip/jobs. |

**Fuentes discriminantes comprobadas:**
[hierarchical.py:160–205](https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/node_parser/relational/hierarchical.py#L160-L205)
devuelve todos los niveles planos;
[utils.py:40–98](https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/vector_stores/utils.py#L40-L98)
serializa `_node_content`/relaciones, no autoriza;
[memory.py:763–829](https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/memory/memory.py#L763-L829)
archiva antes de fan-out;
[sql.py:372–406](https://github.com/run-llama/llama_index/blob/444fdf0684707de62ebb447d658dba137dd9cb99/llama-index-core/llama_index/core/storage/chat_store/sql.py#L372-L406)
confirma ARCHIVED, no aceptación de todos los bloques.

Vanta:
[vectorstore.py:322–406](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/llamaindex/vantadb_llamaindex/vectorstore.py#L322-L406)
corrobora returns tempranos y postfiltro que modifica sólo nodes;
[:460–471](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/llamaindex/vantadb_llamaindex/vectorstore.py#L460-L471)
no usa filters. Tests 148–166/221–241 leídos, no ejecutados.
[engine.rs:62–79](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L62-L79),
[spill.rs:224–261](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/spill.rs#L224-L261)
y [state_manager.rs:78–96](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/offload/state_manager.rs#L78-L96)
corroboran sink `()`, errores absorbidos, contador de nuevos y cursor
load-modify-save. F mejora evidencia, no crea almacenamiento/cursor ni fsync.

**Independencia propuesta explicada:** D necesita identidad estable y backend
consultable, no implementar A y C completas (497–502). Puede reutilizar IDs
actuales; C13 sólo aplica a afirmaciones de mutación, no a transformaciones
puras. F observa copia recuperable por ejecución, no obliga A/B/C ni receptores
LLM (584–591). Su migración inicial es reporte efímero opcional, sin ledger nuevo.
Se conserva FMEA de dedup, fallo, unknown y retención, no garantía perpetua.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0; hash anticipado, vector/docstore/persist, padres y archivo/fan-out separados. |
| 1 Recomendaciones | Sustentado | §1 prioriza fidelidad del adapter existente y progreso, no framework obligatorio. |
| 2 Fichas Rust | Sustentado | A–F, once campos y precisiones; referencias/receipt D corregidos, F no offloader nuevo. |
| 3 Incidencias | Sustentado con límites heredados | §3 #22638 hash poisoning y #23768 requisito preview; estados no reconsultados aquí, fixes no certificados. |
| 4 Prerrequisitos | Sustentado | §4 por candidata; single-writer/at-least-once reconocido, no scheduler distribuido. |
| 5 Rechazos | Sustentado | §5; no delete-before-put atómico, UUID reimportable, `str(dict)` canónico ni denylist=ACL. |
| 6 Método/licencia | Sustentado | §6/11 distingue histórico y fuentes nuevas, MIT raíz no licencia transitiva. |
| 7 Benchmarks | Sustentado con frontera externa | §7 especifica BEIR delegado/no auditado, last-doc-score, población/errores y HitRate/MRR/vacíos; no QA ni cifras propias. |
| 8 Matriz | Sustentado | §8 incluye D/F/fidelidad; híbrido, gobernanza, checkpoints y Studio existentes. |
| 9 Negativos Vanta | Sustentado/acotado | §0.8/9; desalineación condicional estática y filters ignorados no implican ausencia de filtros del motor. |
| 10 UI | Corregido en alcance declarado | §10; MemoryLens al pin, Cloud/Parse externo no inspeccionado ni desktop inventado. |

Residual: fidelidad end-to-end del adapter, relaciones/ACL al hidratar,
garantías backend y evaluador BEIR externo. No se hacen pasar por fallos
resueltos mediante documentación ni impiden cerrar los enlaces/contratos reparados.

## GraphRAG

**Recibo:** [graphrag.md](../graphrag.md), leído completo, 1–598;
diff 566–642. Pin `34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857`.
**Veredicto propuesto: cerrar ambos P2 documentales previos.**

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| G16 sin resolución | Cerrado | Informe 63–70, 79–86, 354, 585–598 añade enlace/rango y distingue `except Exception` de cancelación/compensación. Fuente releída copia previous/delta y emite errores sin rollback de workflows anteriores. |
| Adaptación G14 caché ausente | Cerrado | 383–399 identifica `Embedded::get`, mapping namespace/key y envelope, preparación fuera/gets-decode dentro, completitud, batching no equivalente y CacheLayer interno rechazado como API del benchmark. No delega toda la adaptación a METHOD/global/DRIFT. |

**Fuentes releídas:**
[run_pipeline.py:54–90](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/run/run_pipeline.py#L54-L90),
[:117–157](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/run/run_pipeline.py#L117-L157),
[test_cache_benchmark.py:1–98](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/tests/unit/cache/test_cache_benchmark.py#L1-L98)
y [json_cache.py:20–65](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag-cache/graphrag_cache/json_cache.py#L20-L65).
G14 prellena `text/items/metadata/index`, JsonCache lo envuelve en `result`;
el adapter propuesto debe devolver el valor lógico, no confundir envelope con
hit de retrieval. El runner comprueba **claves**, no igualdad completa de valores;
verificar también valores es aceptación propuesta del mapping, no una
comprobación upstream ya ejecutada.

Vanta [get:1611–1625](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1611-L1625)
y [CacheLayer:17–73](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/storage/engine/cache.rs#L17-L73)
son seams reales contrastados: SDK materializa MemoryRecord; caché interna
usa u128/UnifiedNode/tombstones y es `pub(crate)`. No abrir esa API ni fabricar
paridad get_many para pasar el benchmark. Son lecturas KV adaptadas, no caché
pura ni latencia de respuesta GraphRAG.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0; ingesta/local/global/DRIFT/incremental y publicación parcial separados. |
| 1 Recomendaciones | Sustentado | §1/cierre; B/C primero, A derivado opcional, D/E alternativas con coste explícito. |
| 2 Fichas Rust | Sustentado | A–F con contratos, pasos, migración, rollback, FMEA y aceptación; no segundo grafo/scheduler/assembler. |
| 3 Incidencias | Sustentado con límites heredados | §3 #2590/#2540/#2614; fuente JSON corroborada; estado público no reconsultado aquí, sin fix ni MSRC certificados. |
| 4 Prerrequisitos | Sustentado | §4; ownership, revisión, autorización de derivados y coste por mecanismo. |
| 5 Rechazos | Sustentado | §5; título no revisión, score LLM no confianza, Parquet+vector no commit conjunto. |
| 6 Método/licencia | Sustentado | §6/cierre resuelve G16 y rangos; MIT no exonera corpus/modelos/transitivas. |
| 7 Benchmarks | Sustentado | §7/G14 completo; 1 000 entradas, siete rondas calientes, GC fuera, mediana, asserts/error y adaptación específica; sin runner QA acreditado ni cifras propias. |
| 8 Matriz | Sustentado/acotado | §8/cierre; primitivas existentes frente a comunidades/proyecciones no acreditadas, no ausencia global. |
| 9 Negativos Vanta | Sustentado/acotado | §9; preguntas de linaje/entrega/ownership, no defectos Vanta por analogía. |
| 10 UI | Sustentado en flujo leído | Streamlit `home_page.py:195–237` releído: run_all_searches→citas/error; no desktop o aislamiento backend certificados. Studio reconocido mediante referencia, sin fingir lectura propia de ese informe. |

Fuente UI:
[home_page.py:195–237](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/unified-search-app/app/home_page.py#L195-L237).
Residuales: publicación completa/erasure, authz de derivados y equivalencia
semántica/QA no demostradas. No cambian el cierre selectivo de G16/G14.

## RAGFlow

**Recibo:** [ragflow.md](../ragflow.md), leído completo, 1–500;
diff 643–726. Pin `3c6d2255af06164f0818d0959af10de0afd8b8fa`.
**Veredicto propuesto: cerrar ambos P2 y el seguimiento documental de PR.**

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| P2 Studio fuera del snapshot | Cerrado | Informe 472–481: RetrievalLens en `feature-freez/desktop`, pin correcto, cuerpo releído con git show; no AM/jobs atribuidos al commit. |
| P2 completar C y vocabulario E | Cerrado | C 195–228: efímera por turno/generación/principal, sin tabla/migración, owner/cancelación, rollback a probe autorizado, ProbeFailed ≠ Absent y revalidación. E 259–299: C13 propuesto, dimensiones independientes, owner de trabajo distinto del caller, UNKNOWN postdispatch, persistencia aditiva/legado desconocido y rollback sin revertir commit. |
| PR real localizada, seguimiento no bloqueante | Cerrado documental; fix no acreditado | 315–323 conserva #21187 abierta/no merged y head. Reconsulta REST independiente confirma exactamente ese estado; no se acredita inclusión en el pin, release ni ejecución de sus tests. |

**Consulta oficial independiente:**
[`GET /repos/infiniflow/ragflow/pulls/21187`](https://api.github.com/repos/infiniflow/ragflow/pulls/21187),
cuerpo y metadatos leídos, 2026-11-07:
[`#21187`](https://github.com/infiniflow/ragflow/pull/21187) `state=open`,
`merged=false`, `merged_at=null`, head
`223a26696ed9e082e0da17bab4c1bcee90833ca9`.
El autor describe harness aislado y tests, no una corrida nuestra ni merge.
La PR distingue enumeración completa destructiva de sync best-effort.

**Contraste independiente de fuente:**
[sitemap.go:181–226](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/syncer/connector/sitemap.go#L181-L226),
[:265–305](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/syncer/connector/sitemap.go#L265-L305),
[compiled_knowledge.go:106–151](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/agentic_rag/compiled_knowledge.go#L106-L151)
y [doc_state.go:29–97](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/internal/ingestion/service/doc_state.go#L29-L97).
Se confirma hijo fallido→warning/nil→inventario parcial; claim probe devuelve
false ante fallo; contadores/metadata son best-effort. C/E propuestos corrigen
la interpretación documental, no esos cuerpos upstream.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0 Go actual; append/notify/claim/ACK y proyección/consulta separados; ninguna notificación acredita entrega. |
| 1 Recomendaciones | Sustentado | §1 prioriza fencing y prune completo, después entrega/readiness/observación. |
| 2 Fichas Rust | Sustentado | A–E; C/E completados con once campos, FMEA/aceptación/migración/ownership/rollback; tipos propuestos. |
| 3 Incidencias | Sustentado/acotado | §3 #21186/#22082/#20134; #21187 reconsultada no merged; no fix/release de otros issues acreditado. |
| 4 Prerrequisitos | Sustentado | §4 por candidata; A fence real, B postassembly, C revisión, D sólo reconciliador destructivo, E owner. |
| 5 Rechazos | Sustentado | §5; no stack obligatorio, false=error, porcentajes inventados o prune parcial. |
| 6 Método/licencia | Sustentado | §6/cierre limita writers SQL, enterprise, modelos y APIs; Apache raíz no auditoría legal transitiva. |
| 7 Benchmarks | Sustentado | §7 HTTP versus wrapper Go, nearest-rank, successes/failures y totales distintos, reset/credenciales/efectos/costes; no QA ni cifras propias. |
| 8 Matriz | Sustentado/acotado | §8 reconoce híbrido, provenance, wiki/scheduler y Studio; no infiere publicación cercada de un nombre de API. C/E se detallan en fichas y ledger, no capacidades existentes certificadas. |
| 9 Negativos Vanta | Sustentado/acotado | §9 no convierte reporte agregado en ausencia global ni interleaving hipotético en bug. |
| 10 UI | Corregido en flujo estático | §10/CompileProgressBoard 1–61 releído: error o inflight/backlog, sin porcentaje/cancel inventados; Studio RetrievalLens 1–85 al pin, no runtime. |

UI:
[CompileProgressBoard:1–61](https://github.com/infiniflow/ragflow/blob/3c6d2255af06164f0818d0959af10de0afd8b8fa/web/src/pages/dataset/compilation/compile-progress-board.tsx#L1-L61)
y [RetrievalLens:1–85](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L1-L85).
Residual: implementación efectiva de fences en todos los writers, entrega final,
purga integral y PR/release posterior. Cerrar modal no ACKea cancelación del worker.

## Quivr

**Recibo:** [quivr.md](../quivr.md), leído completo, 1–764;
diff 727–1028. Pin `bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f`.
**Veredicto propuesto: cerrar los tres hallazgos documentales previos,
con residual explícito de integración autorizada.**

| Hallazgo previo | Estado | Cambio y contraste |
|---|---|---|
| P1 informe truncado tras §6 | Cerrado | Informe 530–764 restituye evaluación/matriz/negativos/frontend/cierre; cadena de runner, datos, límites, fallos, coste, efectos y correspondencia Vanta específicos, no simple cita a METHOD. §11 funcional está en «Cobertura y cierre» 718–741, aunque no lleve número. |
| P1 autorización en assembler equivocado | Cerrado como corrección contractual; integración residual | 361–392, 650–663: caller/host resuelve y autoriza antes del assembler, usa recall gobernado y revalida lector/revisión al hidratar/acceder. Firma pura contrastada; no se certifica que toda ruta actual autentique autoridad del host. |
| P2 cancelación y receipt de fuente | Cerrado | 216–235 referencia C13 propuesto, rechaza escalera accepted→ready y limita «no aceptación» a rechazo/cancelación confirmados antes de admisión. Postdispatch sin evidencia = Unknown; lookup conserva autoridad/idempotency key y no revierte commit conocido. |

**Autorización, evidencia y propuesta separadas:**
[engine.rs:252–285](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L252-L285)
no recibe principal/scope;
[auto_recall.rs:133–160](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L160)
y [:390–460](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L390-L460)
reciben policy explícita o wrapper allow-all. El seam gobernado existe; **no
autentica un principal por recibir `InjectionPolicy`**. El actor autorizado
requerido por E es el caller/resolver del host que posee autoridad autenticada
y construye esa política, no el assembler ni el contenido recuperado.
No se ha localizado en estos rangos la cadena completa principal autenticado →
policy → resolver → hidratación. La búsqueda exacta en `vanta-memory/src` sólo
localizó definición/reexport del helper: no equivale a inexistencia de callers
en otros crates. Se cierra la atribución falsa, no se acepta E integrada.

**Commit/outcome contrastado:**
[submissions.go:215–292](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/submissions.go#L215-L292)
fija organización/idempotency key, bloquea identidad, escribe retirada/receipt/
evento y hace `tx.Commit`. Error observado por caller no prueba que esa frontera
no ocurriera. La corrección A representa la incertidumbre sin afirmar rollback.

**Runner contrastado, no ejecutado:**
[run.py:244–284](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L244-L284),
[:325–491](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L325-L491),
[:581–625](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L581-L625),
[:763–788](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/run.py#L763-L788),
[scoring.py:17–52](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/scoring.py#L17-L52),
[trec.py:141–177](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/trec.py#L141-L177)
y [public_sets.py:40–66](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/scripts/eval/public_sets.py#L40-L66).
Registro JSON leído/contrastado: tamaños, splits y semillas de la tabla coinciden;
son objetivos, no población ejecutada. Queries sin positivo se excluyen y listan;
malformados abortan; fallo operativo de search produce ranking vacío/cero y
contador; BudgetExceeded detiene, no convierte no intentadas en ceros.
Latencias sólo de éxitos, calidad sobre queries qrels emitidas; corte deep MIRACL
hasta 150, scored limit 10 con deep/reranker, no asumir 50. Registro/retrieval no
son exactitud QA, memoria longitudinal ni certificación de retiro/retractación.

| Eje | Estado actualizado | Sustento y límite |
|---|---|---|
| 0 Identidad/arquitectura | Sustentado | §0 Go actual, SQL canónico, serving externo, monitoring y retirada; no Python histórico por popularidad. |
| 1 Recomendaciones | Sustentado | §1; A/B antes de C/D/E/F, callbacks sólo con caso de producto. |
| 2 Fichas Rust | Sustentado como propuestas | A–F/once campos; E caller autorizado y resolución viva, A outcome desconocido corregidos; autorización integrada no certificada. |
| 3 Incidencias | Sustentado con límites heredados | §3 #3697 histórico Python frente a #3684 requisito cerrado not planned; estados no reconsultados aquí, sin CVE/fix atribuidos al Go actual. |
| 4 Prerrequisitos | Sustentado | §4 reconoce Wiki run_id/rechazo tardío, L1 parcial y checkpoint; gates por candidata. |
| 5 Rechazos | Sustentado | §5; no stack/negocio/LLM obligatorio, cuota de almacenamiento ≠ budget contextual ≠ plan comercial. |
| 6 Método/licencia | Sustentado | §6/cierre distingue interfaz/SQL/runtime; MIT no licencias transitivas/modelos ni despliegue certificado. |
| 7 Benchmarks | Sustentado | §7 restituido y runner/score/loader contrastados: denominadores/error/invalid/reset/coste/adaptación concretos; no evaluador nuevo ni QA inventada. |
| 8 Matriz | Sustentado | §8 A–F, gobernanza/compaction/cuotas/checkpoints/Studio existentes; «no acreditado» no ausencia global. |
| 9 Negativos Vanta | Sustentado/acotado | §9; puro no autoriza, L1 por pasos, checkpoint at-least-once, no bugs heredados por analogía. |
| 10 UI | Sustentado en flujos fijados | §10 restituido: FeedPage/cliente, AddText/receipt y facade concretos; Studio pin correcto; demo web no desktop ni memoria del agente. |

**UI releída:**
[FeedPage:245–316](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/components/feed/FeedPage.tsx#L245-L316),
[search.ts:31–126](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/lib/search.ts#L31-L126),
[AddText:48–110](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/src/components/AddText.tsx#L48-L110),
[facade:268–304](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/server.mjs#L268-L304)
y [:737–760](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/quivr-search/server.mjs#L737-L760).
Fallback sin filtro conserva `within` efectivo, abort HTTP no revierte efectos,
poll distingue searchable/procesamiento y timeout de espera conserva «continúa».
Facade restringe corpus e inyecta Bearer; password vacío permite acceso a la
demo según ese helper, no identidad multitenant certificada. Ningún flujo fue
abierto en navegador. Residuales: authz completa, coherencia SQL/proyección,
erasure y garantías de plugins/receptor; no bloquean este cierre documental.

## Cierre FMEA, matriz y UI

El ledger anterior tiene **eje 0 y ejes 1–10 para cada proyecto**. Esta tabla
cruza mecanismos/riesgos con matriz y superficie de inspección; no añade
severity scores, umbrales, pruebas, tareas o requisitos universales.

| Proyecto | FMEA/contrato cuya omisión se cierra | Coherencia con matriz actual | UI y residual de cierre |
|---|---|---|---|
| Qdrant | Aceptación no implica commit; feedback deferred no puede fabricarse Completed; outcome incierto no rollback | §8 añade aceptación independiente y conserva WriteReceipt/proyecciones existentes | §10 acredita servicio web, frontend externo no auditado. Alcanzabilidad de conjunción deferred pendiente; no repro ni pérdida certificada. |
| Redis | Entrada/PEL/ACK/claim distintos de efecto externo y de AOF/WAIT/WAITAOF; reclamar tras idle no cerca al receptor | §8 conserva scheduler/cuotas/TTL, backend local no convertido en cola durable | Studio pin corregido; RedisInsight package/docs no flujo funcional ni secretos auditados. Cierre del enlace, no de todo el eje externo. |
| TiDB | Estimación/reserva RAII no RSS; cancelación por identidad y callback con owner, no rollback remoto | §8 conserva gobernador/cuotas/registro MCP y C13 propuesto | Studio shell reproducible, Dashboard documentado como externo. Cierra trazado KV y pin; operadores remotos/reset/cancelación total siguen fuera. |
| SurrealDB | Frontera/generación de rebuild; Opens no compatibilidad total; commit incierto y followup separados | §8 incorpora C/D/Studio y §4 gates individuales; B independiente, C diagnóstico aun con A desconocida | Surrealist flujo editor→ejecución→frames/error y Studio Activity; no interacción/authz ni repair automático certificados. |
| LlamaIndex | Receipt por etapa ≠ mutación total; hash anticipado, padres/revisión y spill fallido no «copia durable» | §8 ahora incluye fidelidad del adapter y F/spill/cursor existentes | MemoryLens al pin; Cloud externo no inspeccionado. No se certifican fidelidad/ACL del adapter ni recuperación por todos los receptores. |
| GraphRAG | Error de workflow no compensación; candidatos ≠ entregados; cache envelope ≠ memoria primaria | §8/cierre conserva grafo/assembly y vistas opcionales, no ausencia global de comunidades | Streamlit con ejecución/citas/error reales. No certificar grounding/erasure/aislamiento de cuatro columnas ni QA desde G14. |
| RAGFlow | C probe fallido ≠ ausencia; E contador ≠ primario, espera ≠ trabajo; prune parcial no autoriza bajas | §8 mantiene primitivas; fichas C/E y ledger explican readiness/observación, sin nueva cola/tabla | CompileProgressBoard sin porcentaje/cancel ficticios; RetrievalLens al pin. #21187 no merged, no fix/release acreditado. |
| Quivr | SourceReceipt complementa C13; postdispatch Unknown; E autoriza en host/resolver y revalida hidratación | §8 restituida A–F, incluye gobernanza/compaction/cuotas; no adjudica permisos al assembler | Demo feed/AddText/facade más Studio al pin; no desktop Quivr ni autenticación multitenant desde contraseña compartida. Cadena completa de autoridad residual. |

**Vocabulario existente frente a propuesto:** Vanta ya implementa
[`WriteReceipt`:678–703](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L678-L703)
y [`put_certified`:1261–1277](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1261-L1277):
atestación puntual con `recorded`, binding de contenido y referencia de cadena,
construida después de `put`. Esos cuerpos se releen, no se infiere de su nombre
un recibo de commit multialmacén o firma de webhook. Las extensiones de receipt
de las fichas y **MutationReceipt/C13 son propuestas** y no sustituyen receipts,
attributions o checkpoints existentes. El puntero «§4» de `quivr.md:668` no
localiza ese WriteReceipt: los dos enlaces anteriores fijan aquí el contraste
correcto; queda como precisión de navegación, no falsedad del mecanismo.

## Cierre y límites

**Decisión propuesta:** cerrar selectivamente las correcciones documentales
identificadas en las tablas, no declarar los ocho proyectos «aceptados».
No queda un hallazgo previo de redacción/trazabilidad marcado **no resuelto**
en este alcance; sí permanecen **residuales** explícitos. En particular,
RedisInsight/Dashboard no pasan a UI auditada; deferred Qdrant no pasa a
reproducción; el helper gobernado Quivr no pasa a autenticación end-to-end;
#7552 cerrado/comentario de fix y #21187 abierta no pasan a release verificada.

Se preservan memoria híbrida, scopes/gobernanza, provenance, sesiones,
supersession/dedup, context assembly/compaction con presupuesto compartido,
personas/escenas/skills, wiki, scheduler/checkpoints, cuotas/gobernador y Studio
del baseline. Ninguna muestra negativa se usa para negar esas capacidades.
El hash de un receipt, la reserva de bytes, el budget del modelo y una cuota
comercial no son garantías intercambiables.

**Límites de evidencia:** fuentes de código fijadas prueban cuerpos leídos,
no ejecución; documento upstream describe contrato, no certifica todos sus
callers; issue mutable prueba reporte/estado observado, no causalidad/fix;
comentario o harness reportado por un autor no es experimento nuestro.
Las consultas fallidas/rate limits de TiDB/SurrealDB y búsquedas sin resultados
se conservan en sus informes y en esta reconfirmación: no hay censo exhaustivo
de issues/PRs. No-localización tampoco es prueba de inexistencia.
No se certifican rendimiento, seguridad global, calidad QA, crash safety,
recuperación física, licencia transitiva o funcionamiento interactivo.
Los runners se leen y adaptan conceptualmente; ni su existencia ni sus números
publicados se presentan como corrida propia. No se crean prompts, rúbricas,
graders, fórmulas, métricas, parámetros, thresholds o infraestructura.

**Comprobación final exclusivamente documental:** se releyó el entregable y
se reconstruyó en memoria el diff unificado de las ocho bases contra los ocho
informes actuales. Sigue coincidiendo exactamente con el artefacto de **1 028
líneas**, mismo SHA-256; los ocho hashes de base coinciden con el recibo.
Comandos usados: `Get-Content`/`Select-String`/`Get-FileHash`, `git show`,
`git grep`, `git ls-tree`, `rev-parse`, `status --short`, `check-ignore -v`,
lecturas REST/raw oficiales y Python stdlib para comparar texto en memoria.
No se ejecutaron validadores de proyecto, tests, builds, benchmarks, aplicaciones,
instalaciones, servicios ni start/stop de runtime. Sin Git de escritura,
staging, commit, push, checkout, fetch o pull. Sin delegación del análisis.

**Única escritura de esta reconfirmación:** este archivo. Los cambios de otros
informes/contratos visibles en el workspace son ajenos a esta escritura; no se
revierten ni se incluyen. El entregable existe en disco pero está ignorado por
`.gitignore:268` (`docs/dev/research/`), comprobado con `git check-ignore -v`;
no se cambió esa regla ni se hizo `add -f`. No se usó repositorio/forge de
Meilisearch.

**Siguiente decisión reservada al responsable:** aceptar o rechazar por
candidata los contratos propuestos y determinar qué residuales importan a su
caso, sin convertir las futuras validaciones manuales específicas en un gate
runtime universal. Si después se aprueban specs, dividir por fuente/publicación,
progreso/receipts, contexto/evidencia y anexos benchmark/UI conservando este
dossier/TOC y enlaces; no se crean esos archivos o tareas aquí.
