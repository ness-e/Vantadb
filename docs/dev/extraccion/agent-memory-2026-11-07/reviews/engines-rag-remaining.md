---
title: "Revisión independiente de motores y RAG pendientes"
kind: research
status: proposed
description: "Recibos adversariales de Redis, TiDB, SurrealDB, LlamaIndex, GraphRAG, RAGFlow y Quivr con evidencias y correcciones pendientes"
tags: [research, memory, engineering]
---

# Revisión independiente de motores y RAG pendientes

## Alcance y criterio

Revisión independiente de siete informes, no autoría ni aceptación final.
No repite Qdrant/Faiss de [engines](engines.md) ni LanceDB de
[rag-ingestion](rag-ingestion.md). Contratos leídos:
[EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md), [VANTA-REFERENCE](../VANTA-REFERENCE.md),
[METHOD](../METHOD.md), [BENCHMARK-NOTES](../BENCHMARK-NOTES.md),
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md) y brief temporal del responsable.
Se cargaron las skills documentation-skill y coordinated-web-search y se
descubrieron servidores/herramientas MCP. No se delegó ni se ejecutaron proyectos.

La fila 0 evalúa arquitectura/identidad y las filas 1–10 los ejes del contrato,
sin puntuación agregada. «Parcial» no significa defecto de runtime; identifica
evidencia insuficiente o corrección documental concreta. P1: impide confiar en
una conclusión/contrato central; P2: precisión o cobertura localizada.
Toda aceptación de propuestas sigue reservada al responsable principal.

## Índice

- [Redis](#redis)
- [TiDB](#tidb)
- [SurrealDB](#surrealdb)
- [LlamaIndex](#llamaindex)
- [GraphRAG](#graphrag)
- [RAGFlow](#ragflow)
- [Quivr](#quivr)
- [Límites de la revisión](#límites-de-la-revisión)

## Redis

**Recibo guardado:** lectura íntegra de [redis.md](../redis.md), líneas 1–570;
SHA upstream `5b69ba2ce91fea8e9fc70ebac8ce341b37e75533`, HEAD del clon contrastado.
**Veredicto propuesto: cambios requeridos**, localizados en identidad Studio;
sin hallazgo bloqueante adicional en los mecanismos contrastados.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | sustentado | §0 distingue XADD/entrega/PEL/efecto, claim tras recorte y AOF/WAIT/WAITAOF; no inventa atomicidad externa ni ANN inspeccionado |
| 1 Recomendaciones | sustentado | §1 condiciona prioridad a inventario; conserva memoria canónica y scheduler existentes |
| 2 Fichas Rust | sustentado | §2.1–2.5 incluyen problema, unidad, delta, alternativas, contrato/errores/invariantes, seams, pasos/persistencia/rollback, dependencias, FMEA, aceptación y no objetivos; no otro WAL/broker |
| 3 Incidencias | sustentado | §3: #13956 y #5177 cuerpos/comentarios releídos; ambas abiertas; min-idle vs ejecución externa y corrupción histórica no se diagnostican como bugs Vanta |
| 4 Prerrequisitos | sustentado | §4 restringe C13 a afirmaciones durables y separa checkpoint global de cada candidata |
| 5 Exclusiones | sustentado | §5 rechaza servidor/cluster, eviction del canon y copia C→Rust como escape legal |
| 6 Método | sustentado | §6 y §11 delimitan módulos/IDMP/replay no auditados; licencia triple raíz contrastada, dependencias no certificadas |
| 7 Benchmarks | sustentado | §7: defaults/semilla, primer read, HDR, prefijos excluidos, errores/efectos, sin warmup explícito; pista motor, no QA |
| 8 Matriz | sustentado | §8 no niega TTL/híbrido/recall/Studio ni convierte backend local en almacenamiento durable |
| 9 Negativos | sustentado | §9 formula riesgos condicionados a callers/handlers, no carreras reproducidas |
| 10 UI | parcial | §10 separa Redis Insight externo/documental de fuente package.json; atribución Studio incorrecta al SHA |

### Hallazgos Redis

- **P2 — identidad inmutable de Studio incorrecta.** `redis.md:520–527`
  atribuye `desktop/package.json` y `desktop/src-tauri/tauri.conf.json` a
  `0ad38c542b39a589035b0e17fdedb4c09dca24b9`. `git ls-tree` del worktree
  separado sólo devuelve `feature-freez/desktop/...`; `git show` de
  `feature-freez/desktop/package.json:1–48` acredita React/Vite/Tauri.
  **Mecanismo/consecuencia:** mezcla rutas movidas/modificadas del worktree con
  bytes inmutables; el enlace/atribución no reproduce la observación.
  **Corrección:** citar rutas `feature-freez/desktop/...` del SHA y rangos
  efectivamente releídos, o declarar por separado lectura dirty; no negar Studio.

### Evidencias y límites Redis

- Clon `C:\Users\Eros\redis`: `t_stream.c::xackCommand:4248–4305`,
  `xautoclaimCommand:5220–5308`; ACK elimina NACK, claim comprueba payload y
  minidle. `aof.c:1580–1650,2094–2129` distingue write/fsync y tail reparable;
  `replication.c:4916–4989` distingue ACK de procesamiento/fsync.
  `db.c:3137–3185` separa expired/trimmed/replica; `LICENSE.txt:1–8` tri-licencia.
- `redis-benchmark.c:445–554,974–1010,1751–1799`: muestras por respuesta usando
  latencia del primer evento, prefijos fuera del contador; configuración y
  estado del servidor no quedan fijados por el cliente. No se midió nada.
- Fuentes públicas releídas por MCP/GET:
  [#13956](https://github.com/redis/redis/issues/13956),
  [comentario](https://github.com/redis/redis/issues/13956#issuecomment-2816596033),
  [#5177](https://github.com/redis/redis/issues/5177) y sus comentarios completos.
  Lectura del 2026-11-07: no PR/fix acreditado por esos hilos.
  [RedisInsight package.json](https://github.com/RedisInsight/RedisInsight/blob/43bd288ed30dc6d2355723158dad5d7ecf4a280e/package.json#L1-L55)
  confirma SSPL/scripts desktop; no prueba flujos CRUD ni seguridad de secretos.
- Vanta al `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`:
  `pipeline_worker.rs:265–390` ejecuta handler antes de complete;
  `local_backend.rs:235–290` ya verifica owner bajo mutex. El delta de epoch/
  resolución del efecto no es fencing desde cero. C13 sigue propuesto.
  No se certificaron durabilidad de cola, replay completo, módulos ni UI runtime.

## TiDB

**Recibo guardado:** [tidb.md](../tidb.md) leído completo, 1–550.
HEAD del clon `41efd866c3801a637ab2d6712b86d8b2cd77ac5b` contrastado.
**Veredicto propuesto: cambios requeridos**, de precisión de trazado e identidad;
la selección de mecanismos locales no exige adoptar una plataforma distribuida.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | parcial | §0 separa buffer/commit/rollback y lecturas; los nombres/cadena de `Get/Set/Delete` no corresponden al driver citado |
| 1 Recomendaciones | sustentado | §1 distingue SQL/TiKV de tracker/killer locales y del motor Rust |
| 2 Fichas Rust | sustentado | A/B, precisiones y C cubren once campos: RAII/bytes estimados, señal por identidad, guard de callback; seams gobernador/executor/registro MCP, migración efímera/aditiva y FMEA |
| 3 Incidencias | sustentado | §3 #72234 cuerpo/comentario releídos, abierto; reproducción reportada v8.5.8/v26.3, sin fix certificado ni diagnóstico Vanta |
| 4 Prerrequisitos | sustentado | §4 A/B/C independientes; ownership/commit por candidata, sin checkpoint como bloqueo universal |
| 5 Exclusiones | sustentado | §5 excluye TiKV/Raft/SQL, panic FFI y cuotas heredadas |
| 6 Método | sustentado | §6 delimita enterprise, drivers remotos y árbol legal; no presenta trackers como RSS |
| 7 Benchmarks | sustentado | §7 `bench_test.go:1–42` completo: b.N/RunParallel, 256 MiB contables no asignados; errores MemTotal/MemUsed descartados y efectos de suite no certificados |
| 8 Matriz | sustentado | §8 reconoce gobernador/cuotas/híbrido/scheduler y C13 propuesto |
| 9 Negativos | sustentado | §9 AtomicU64 requiere balance externo; underflow condicionado, cancelación MCP existente, no fallo reproducido |
| 10 UI | parcial | Dashboard es web/PD documentado, no desktop; Studio usa rutas ajenas al SHA atribuido |

### Hallazgos TiDB

- **P2 — cadena de acceso KV narrada con métodos distintos a la fuente.**
  `tidb.md:56–63` atribuye `MemBuffer.GetLocal`→`KVTxn.Get`, y Set/Delete al
  MemBuffer. En [txn_driver.go:195–215](https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/store/driver/txn/txn_driver.go#L195-L215),
  `Get` usa `GetMemBuffer().Get`, fallback `GetSnapshot().Get` sólo ante
  `ErrNotExist`; `Set/Delete` delegan directamente en `KVTxn.Set/Delete`.
  **Consecuencia:** se confunde frontera del driver con implementación del cliente,
  y se omiten la condición de fallback y el tratamiento `IsValueEmpty`.
  **Corrección:** describir exactamente esa rama; atribuir buffering del cliente
  sólo a código adicional efectivamente leído. No cambia el límite reconocido
  por el informe: el driver no acredita fsync/aislamiento global.
- **P2 — Studio mal anclado.** `tidb.md:500–506` atribuye `desktop/...` al
  `0ad38c542b39a589035b0e17fdedb4c09dca24b9`. Mismo contraste `ls-tree/show`
  del recibo Redis: corregir a `feature-freez/desktop/...` o declarar worktree.
  No es ausencia de UI ni motivo para crear otro dashboard.

### Evidencias y límites TiDB

- `C:\Users\Eros\tidb`: `memory/bench_test.go:1–42`,
  `tracker.go:265–312,450–609`: unbind existe, cadenas por prioridad,
  acción bajo mutex y panic por kill; `sqlkiller.go:100–202` CAS de primera
  señal, FinishFunc protegido. No se verificó reset entero ni cada operador.
- `txn_driver.go:115–218` y `session.go:993–1060` separan delegación commit,
  LastCommitTS condicionado y rollback/cleanup. No lectura de TiKV ni
  certificación de cancelación remota/consenso.
- [#72234](https://github.com/pingcap/tidb/issues/72234) y
  [comentario causal](https://github.com/pingcap/tidb/issues/72234#issuecomment-6372472845)
  leídos por REST, estado abierto observado 2026-11-07. La propuesta
  SetFinished/skip no se trató como PR mergeada.
  [Dashboard Introduction](https://docs.pingcap.com/tidb/stable/dashboard-intro/)
  leído por GET confirma Web UI incorporada en PD/Pod opcional; no frontend
  externo inspeccionado ni licencia de Dashboard acreditada.
- Vanta `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`,
  `memory_governor.rs:38–85`: estimación con fetch_add/sub y watermarks
  ya presentes. Ni scope nuevo ni callback se justifican como corrección
  de un bug Vanta demostrado. No medidas RSS, throughput o crash safety.

## SurrealDB

**Recibo guardado:** [surrealdb.md](../surrealdb.md) leído íntegro, 1–552;
HEAD upstream `9577ac5f0caa60a39d5a334e9352aed0e232f9e6` contrastado.
**Veredicto propuesto: cambios requeridos.** La ampliación contiene mecanismos
útiles, pero su cierre anuncia evidencia que no está entregada en este archivo.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | parcial | §0 traza tres flujos y commit incierto/readiness; muchas referencias nuevas no tienen definición recuperable |
| 1 Recomendaciones | sustentado | §1 prioriza diagnóstico/fronteras locales; explícitamente no plataforma distribuida |
| 2 Fichas Rust | parcial | A/B/C/D describen once campos, alternativas/errores/FMEA; seams nuevos VADMIN/VREBUILD/VAUDIT/VL1 sin enlaces y V1/V2 históricos impiden acreditar integración actual |
| 3 Incidencias | no acreditado | §3 sólo hechos estáticos/hipótesis; no URL, número, estado ni revisión de incidencia pública pese a anuncio inicial |
| 4 Prerrequisitos | parcial | §4 delimita inventario/licencia, pero carece de mapa individual A/B/C/D comparable a las precisiones contractuales |
| 5 Exclusiones | sustentado | §5 no traslada leases/heartbeats ni BSL como permiso Apache vigente |
| 6 Método | parcial | §6 contradice consultas/lecturas anunciadas en §0/§1; V1/V2 siguen c561158; faltan inventario actualizado y cierre §11 |
| 7 Benchmarks | parcial | §7 runner completo y dimensiones correctas; faltan errores/denominador/reset y adaptación Vanta concreta |
| 8 Matriz | parcial | §8 sólo A/B: omite C/D como capacidades específicas; V1/V2 enlazados a snapshot histórico |
| 9 Negativos | sustentado | §9 pregunta por frontera/compatibilidad sin inventar bugs concurrentes de Vanta |
| 10 UI | no acreditado | §10 únicamente imagen README y producto no inspeccionado; contradice anuncio de fuente UI externa y no acredita implementación/flujo Surrealist |

### Hallazgos SurrealDB

- **P1 — ampliación sin trazabilidad completa y cierre no entregado.**
  `surrealdb.md:28–31,34–44,280–407,455–462,541–552` usa, entre otros,
  `[STCOMMIT]`, `[STBOUND]`, `[SSTATE]`, `[SHEALTH]`, `[VADMIN]`,
  `[VREBUILD]`, `[VAUDIT]`, `[VL1]` sin definiciones; termina en V1/V2 al
  `c561158253bb88192590fa406c61e0a27dd7df8d`, no al baseline declarado.
  TOC anuncia §11 inexistente. **Mecanismo/consecuencia:** referencias textuales
  no permiten reproducir los nuevos claims ni distinguir inventario histórico
  de seams revalidados; el mapa de cobertura queda artificialmente cerrado.
  **Corrección:** restituir definiciones SHA/rango y §11 con leído/no leído,
  dependencias y split plan; fijar seams reales al `97c9623...` o etiquetarlos
  históricos sin llamarlos revalidados. No requiere cambiar código Vanta.
- **P1 — incidencias anunciadas pero no documentadas.**
  `surrealdb.md:28–31,153–154,409–433,455–461` promete consultas/incidencias y
  a la vez dice «sin red». No hay issue concreto en §3.
  **Consecuencia:** no puede comprobarse estado/fix/revisión afectada; hipótesis
  propias sustituyen un eje obligatorio. **Corrección:** añadir incidencias
  efectivamente leídas con URL/estado/fecha/revisión/PR o acceso fallido y
  laguna expresa; retirar el claim de cierre. No inventar incidente ni reproducirlo.
- **P2 — benchmark sin tratamiento operativo ni adaptación específica.**
  `surrealdb.md:474–504` describe bien mean/median/warmup, pero
  [point_read.rs:90–138](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/benches/point_read.rs#L90-L138)
  aborta por unwrap/assert, usa `samples.len()` y mediana superior, y crea
  store por esquema, compartido por cuatro casos. No hay retry ni reset por
  petición. **Corrección:** documentar esa población/efectos, caché heredada y
  qué seam SDK Vanta mapearía point-get y shape de resultado; enlace al método
  común no sustituye adaptación. No crear runner/configuración.
- **P2 — UI no verificada.** `surrealdb.md:530–539` acredita sólo imagen.
  Restituir fuente concreta de Surrealist si se pretende estudiar flujo real;
  en otro caso declarar eje incompleto y no «fuente externa añadida».
  Studio ya existe, pero no convierte la imagen en evidencia upstream.

### Evidencias y límites SurrealDB

- Clon: `engine-local/benches/point_read.rs:1–152` leído completo;
  `core/src/kvs/write_barrier.rs:1–60` distingue contrato local/multinodo;
  `datastore/src/triggers.rs:1476–1538,1572–1625` cuenta writers por epoch
  antes del snapshot y coordina avance. No adopción de fencing distribuido.
- `datastore/src/tx.rs:3284–3379,3384–3480,7856–7873` confirma error primario,
  deltas/changefeed precommit y CommitOutcomeUnknown; no prueba fsync.
  `core/src/kvs/rollback.rs:165–261` separa veredicto por ledger de hallazgos
  posteriores y lee múltiples transacciones, no snapshot único.
  [LICENSE](https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/LICENSE#L1-L68)
  leído: BSL/Database Service/2030; sin dictamen legal de dependencias.
- Vanta `ls-tree` localiza `src/sdk/api/admin.rs` y
  `src/sdk/serialization/impl_rebuild.rs`; una primera lectura intentada de
  `src/sdk/admin.rs` falló y no se trata como evidencia. No se certificó la
  frontera concurrente de rebuild ni todo el pipeline citado sin referencias.
- Sin issue específico que contrastar en el informe, ni consulta remota
  Surrealist presentada como realizada en esta revisión. No se atribuyó
  un bug upstream a Vanta ni se ejecutó reproducción.

## LlamaIndex

**Recibo guardado:** [llama-index.md](../llama-index.md), lectura completa 1–916;
HEAD del clon `444fdf0684707de62ebb447d658dba137dd9cb99` contrastado.
**Veredicto propuesto: cambios requeridos.** Los defectos del adapter que
identifica el autor son hechos estáticos válidos; falta hacer reproducible
parte sustancial de la ampliación con referencias y revisión bien enlazadas.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | sustentado | §0 distingue hash anticipado, caché, add/vector, docstore, persist separado; consulta/hidratación y update/delete con cancelación parcial |
| 1 Recomendaciones | sustentado | §1 prioriza fidelidad del adapter ya existente; no obliga Python/LLM ni otro índice |
| 2 Fichas Rust | parcial | A–F contienen once campos y seams reales; referencias esenciales L20–L23/V13–V17 no definidas; D omite relación explícita con C13 |
| 3 Incidencias | sustentado | #22638 y #23768 cuerpos releídos, abiertos; hash poisoning frente a requisito preview, no fix ni bug Vanta por analogía |
| 4 Prerrequisitos | sustentado | §4 separa identidad/jerarquía/exportación/progreso; no gate universal ni nuevo scheduler |
| 5 Exclusiones | sustentado | §5 rechaza delete-before-put como atomicidad, str(dict) estable y UUID como identidad reimportable |
| 6 Método | parcial | Cobertura delimitada pero inventario nuevo no resuelve todas sus referencias; lectura no equivale a tests ejecutados |
| 7 Benchmarks | sustentado | BEIR delegado/dataset/caché/errores/last-doc-score y HitRate/MRR con vacíos/denominadores exactos; sin equiparar IR, QA y memoria |
| 8 Matriz | parcial | Reconoce híbrido, governance, checkpoint y Studio; no añade F/spill existente ni fidelidad del adapter como filas específicas |
| 9 Negativos | sustentado | §0.8 acredita desalineación condicional nodes/ids/similarities y filtros no aplicados en rutas tempranas; §9 separa otros límites de bugs |
| 10 UI | parcial | LlamaCloud/Parse externo no inventado como desktop; enlace MemoryLens apunta fuera de la ruta del commit |

### Hallazgos LlamaIndex

- **P1 — evidencia añadida no enlazada.** `llama-index.md:165–257,545–592,882–916`
  cita `[L20]`–`[L23]` y `[V13]`–`[V17]` sin definiciones.
  **Consecuencia:** jerarquía, serialización, archivo SQL, adapter ya existente
  y delta de spill son centrales a B/D/F y no tienen cadena SHA navegable.
  **Corrección:** añadir enlaces completos/rangos de las fuentes realmente
  leídas; ampliar inventario §11 y matriz para F/adapter. El código comprobado
  sí existe: no se corrige inventando una ausencia o integración desde cero.
- **P2 — receipt de etapa no vinculado al receipt canónico.**
  `llama-index.md:463–501` propone `IngestStageReceipt` con outcomes
  `Applied/Failed/Unknown/PendingProjection`. Son válidos como observaciones
  de una etapa, pero no bastan para describir simultáneamente commit durable
  primario y proyección pendiente. **Corrección:** aclarar que referencia/
  complementa C13 propuesto cuando describe mutación; conservar sus dimensiones
  independientes, sin convertir esos outcomes en cadena ni exigir fsync derivado.
- **P2 — enlace Studio incompatible con el SHA.**
  `llama-index.md:837–845,902`: MemoryLens existe en
  `feature-freez/desktop/src/components/memory/MemoryLens.tsx` al
  `0ad38c542b39a589035b0e17fdedb4c09dca24b9`, no `desktop/...`.
  `git show` 1–90 confirma imports/escenas y tratamiento de error; corregir
  URL/ruta, sin atribuir los bytes actuales movidos/modificados al SHA.

### Evidencias y límites LlamaIndex

- Clon: `ingestion/pipeline.py:451–507` confirma hash escrito antes del
  transform y delete previo; `auto_merging_retriever.py:66–125,166–194`
  confirma hidratación docstore/media/log INFO/loop sin guard de ciclos.
  `memory.py:763–829` y `chat_store/sql.py:372–406` confirman archive commit
  previo al gather de bloques; archive no implica aceptación por receptor.
- `evaluation/benchmarks/beir.py:1–109` completo y
  `retrieval/metrics.py:16–158`: BEIR descarga/borrado de caché fallida,
  resultados por doc_id con último hit; fórmulas BEIR externas no auditadas.
  HitRate granular cuenta duplicados y MRR granular divide por hits recuperados.
- Vanta `97c9623...`, `integrations/llamaindex/vantadb_llamaindex/vectorstore.py`
  `118–169,322–406,460–471`: put_batch/delete existen, postfiltro cambia sólo
  nodes; MMR/híbrido cliente retornan antes y get_nodes no aplica filters.
  No se ejecutó el caso; no implica ausencia de filtros del motor.
  `context_engine/spill.rs:224–261` confirma contadores y error absorbido:
  F mejora evidencia, no crea spill ni transforma API success en fsync.
- [#22638](https://github.com/run-llama/llama_index/issues/22638) y
  [#23768](https://github.com/run-llama/llama_index/issues/23768), cuerpos
  completos por GET, estado abierto observado 2026-11-07. #22638 dio 504
  por REST y MCP; variante del mismo endpoint con query permitió leerlo.
  No comentarios/fix histórico certificados por esta revisión.
  Cloud/BEIR externo, todos los plugins y runtime permanecen fuera de alcance.

## GraphRAG

**Recibo guardado:** [graphrag.md](../graphrag.md) íntegro, 1–574;
HEAD `34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857` contrastado.
**Veredicto propuesto: cambios requeridos**, puntuales de trazabilidad/adaptación.
No se encontró contradicción bloqueante en los mecanismos estáticos contrastados.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | sustentado | §0/§2 distingue ingesta, consulta local/global/DRIFT e incremental; write de tabla/callback no implica generación atómica |
| 1 Recomendaciones | sustentado | §1 y cierre priorizan B/C antes de derivados/orquestación; grafo/híbrido existentes |
| 2 Fichas Rust | sustentado | A–F más precisiones cubren once campos, identidad/revisión, alternativas, migración/rollback, FMEA y aceptación; fuentes/LLM fuera del core |
| 3 Incidencias | sustentado | #2590/#2540/#2614 cuerpos releídos y código concordante; abiertas, sin fix certificado ni repro propio |
| 4 Prerrequisitos | sustentado | §4 exige dependencias/ownership por mecanismo y autorización de derivados; no runtime como bloqueo universal |
| 5 Exclusiones | sustentado | §5 no traslada título como revisión, scores LLM como confianza, plataforma Python ni atomicidad implícita |
| 6 Método | parcial | G16 mencionado en §0/cierre no tiene enlace/entrada en inventario; resto fija SHA y límites |
| 7 Benchmarks | parcial | Caché: runner/defaults/warmup/mediana/denominador/efectos correctos; adaptación descrita para global/DRIFT, no para get_many Vanta |
| 8 Matriz | sustentado | §8 distingue primitiva existente de comunidad/proyección no acreditada; no inferencia de ausencia global |
| 9 Negativos | sustentado | §9 sólo preguntas concretas de revisión/entrega/ownership, no bug Vanta atribuido |
| 10 UI | sustentado | Fuente Streamlit real y flujo main→run_all_searches→citas; no desktop ni SaaS supuesto; Studio reconocido sin fingir lectura |

### Hallazgos GraphRAG

- **P2 — falta resolver G16.** `graphrag.md:63–69,81–85,499–500`
  remite a G16 sin entrada en §6/cierre. El código efectivamente contrastado es
  [run_pipeline.py:54–157](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/run/run_pipeline.py#L54-L157):
  copia previous/delta y captura excepción sin rollback.
  **Consecuencia:** la evidencia central de efectos parciales queda no navegable.
  **Corrección:** añadir enlace/rango G16 y clasificar cancelación aparte de
  `except Exception`; no convertir esa captura en garantía de cancelación.
- **P2 — adaptación del microbenchmark de caché ausente.**
  `graphrag.md:367–382` describe correctamente el runner pero pasa a medir
  global/DRIFT. `test_cache_benchmark.py:35–98` opera sobre Cache/get_many,
  no recall del agente. **Corrección:** identificar adapter/seam Vanta concreto
  y mapping key→valor/envelope, misma completitud y preparación excluida; si
  no existe equivalente útil, rechazar justificadamente esa adaptación.
  No autorar harness/umbral ni usar el nombre GraphRAG como dataset QA.

### Evidencias y límites GraphRAG

- `index/update/incremental_index.py:29–78`,
  `workflows/load_update_documents.py:30–62`,
  `update/communities.py:14–84`: títulos, consumo sólo new_inputs y remapeo de
  comunidad/padre, no miembros. `graphrag_cache/json_cache.py:20–65`:
  JSON válido no objeto falla en get; objeto sin result devuelve None.
- `global_search/search.py:142–214,306–365` confirma gather/map/reduce,
  categorías de uso y NO_DATA_ANSWER condicionado; `drift_search/search.py:225–305`
  conserva acciones por query_state y límites configurados; calcula
  `t_elapsed` antes del reduce opcional, así que `completion_time` no acredita
  latencia end-to-end de respuesta reducida. `local_search/
  mixed_context.py:91–162` concatena historial y mapea entidades antes del render.
  No todos los builders/streaming ni proveedores auditados.
- `tests/unit/cache/test_cache_benchmark.py:1–98` completo: 1 000 entradas,
  siete rondas, asserts antes/después, GC fuera de tiempo y ratio SQLite.
  Sin latencia QA, reset semántico, retry ni resultado medido propio.
  `unified-search-app/app/home_page.py:195–237` acredita llamada y presentación
  de citas/error, no aislamiento backend o entailment.
- GET de [#2590](https://github.com/microsoft/graphrag/issues/2590),
  [#2540](https://github.com/microsoft/graphrag/issues/2540) y
  [#2614](https://github.com/microsoft/graphrag/issues/2614), cuerpos completos,
  estados abiertos observados 2026-11-07. No se validó afirmación MSRC,
  forks, cifras ni fixes de otros issues relacionados; comentarios no re-auditados.
- Vanta `97c9623...`: `ontology/mod.rs:147–185` y
  `context_engine/engine.rs:252–325` ya ofrecen provenance/assembly gobernable.
  Son seams, no prueba de comunidades existentes ni de su ausencia global.

## RAGFlow

**Recibo guardado:** [ragflow.md](../ragflow.md) completo, 1–470;
HEAD `3c6d2255af06164f0818d0959af10de0afd8b8fa` contrastado.
**Veredicto propuesto: cambios requeridos**, de trazabilidad Studio y
contrato de observaciones; el informe identifica correctamente el snapshot Go.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | sustentado | §0 distingue ingesta/claim/ACK, consulta compiledRun y update/retirada; primario, notificación y proyección separados |
| 1 Recomendaciones | sustentado | §1 delta publication/prune/entrega/producto/diagnóstico; no plataforma SQL/NATS obligatoria |
| 2 Fichas Rust | parcial | A–E describen unidad/delta/seam/contrato/pasos/deps/FMEA/aceptación/no objetivos; C debe explicitar persistencia/migración/rollback y E referencia a C13 |
| 3 Incidencias | sustentado | #21186/#22082/#20134 cuerpos completos releídos; abiertos, revisiones/configuración específicas, no bug Vanta por analogía |
| 4 Prerrequisitos | sustentado | §4 no niega scheduler/wiki/checkpoint/assembly; cada requisito ligado a candidata |
| 5 Exclusiones | sustentado | §5 rechaza pruning parcial, gate false=error y stack completo |
| 6 Método | sustentado | §6 diferencia interfaces de SQL, claims de ejecución y límites de módulos/enterprise |
| 7 Benchmarks | sustentado | §7 separa HTTP y wrapper Go; métricas nearest-rank, success/failure, preparación/reset/servicios/coste y adaptación, no QA |
| 8 Matriz | sustentado | §8 reconoce índices/tenancy/dedup/Studio, no acredita fence desde nombre de WikiStore |
| 9 Negativos | sustentado | §9 no convierte agregado contextual en ausencia global ni L1 secuencial en corrupción |
| 10 UI | parcial | CompileProgressBoard real y sin cancel imaginario; RetrievalLens enlazado a ruta que no pertenece al SHA |

### Hallazgos RAGFlow

- **P2 — Studio enlazado fuera del snapshot.** `ragflow.md:444–451`
  usa `0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/...`.
  La ruta inmutable es `feature-freez/desktop/src/components/lens/retrieval/
  RetrievalLens.tsx`, según VANTA-REFERENCE y árbol Git contrastado.
  **Consecuencia/corrección:** no reproducible como está; corregir ruta y
  releer rangos, o separar lectura del worktree AM, nunca atribuirle esos bytes.
- **P2 — completar garantías de C y vocabulario de E.**
  `ragflow.md:195–219,250–275` define readiness cacheada por turno y
  `IngestObservation`, pero no aclara migración/rollback de C ni enlace con
  C13 para mutaciones de E. **Corrección:** indicar si C es estrictamente
  efímera y cómo desactivarla sin fail-open; E debe consumir/referenciar
  MutationReceipt propuesto cuando informe commit/durabilidad/proyección.
  `primary_state` por sí solo no acredita fsync ni readiness.
- **Seguimiento no bloqueante — PR real localizada, no merge.**
  `ragflow.md:291–295` deja PR sin verificar. El cuerpo de #21186 enlaza
  [#21187](https://github.com/infiniflow/ragflow/pull/21187), consultada por
  REST: abierta, `merged=false`, head
  `223a26696ed9e082e0da17bab4c1bcee90833ca9` el 2026-11-07.
  Añadir ese estado y no inferir merge por `merge_commit_sha` de prueba.

### Evidencias y límites RAGFlow

- `sitemap.go:181–226,265–305`: fetch hijo falla con warning/nil y OpenPrune
  acepta supervivientes; fetch de descubrimiento PDF también puede saltarse.
  `compiled_evidence.go:115–203` separa estado por run, probe y shown con mutex:
  marcado previo a entrega real no certifica inclusión final.
- `test/benchmark/metrics.py:1–78` y `retrieval.py:14–54` completos:
  nearest-rank, qps de intentos/success_qps, error HTTP/transporte/JSON/API;
  hasta cuerpo excluye parse JSON de la latencia. No se inspeccionaron todos
  los transportes chat ni se recalcularon defaults del wrapper por ejecución.
- `web/.../compile-progress-board.tsx:1–61` confirma estados/error,
  inflight/backlog y log; no botón cancel ni porcentaje estable.
  Sin navegador, producto SaaS ni licencia enterprise acreditados.
- GET de [#21186](https://github.com/infiniflow/ragflow/issues/21186),
  [#22082](https://github.com/infiniflow/ragflow/issues/22082),
  [#20134](https://github.com/infiniflow/ragflow/issues/20134), cuerpos completos,
  abiertos observados 2026-11-07. #21186 usa dobles, no KB viva; #22082 es
  reporte OceanBase/SeekDB, no fallo universal. No fix certificado de éste.
  Persistencia SQL de todos los claimers/writers y auth end-to-end no auditadas.

## Quivr

**Recibo guardado:** [quivr.md](../quivr.md) íntegro, 1–509;
HEAD `bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f` contrastado.
**Veredicto propuesto: cambios requeridos.** El archivo termina en §6:
no contiene los ejes 7–10 ni el cierre anunciados por su TOC.

| Eje | Estado | Sustento o laguna concreta |
|---|---|---|
| 0 Arquitectura | sustentado | §0 distingue aceptación SQL, proyección externa, consulta y withdrawal; engine Go actual, no backend Python histórico |
| 1 Recomendaciones | sustentado | §1 prioriza receipt/fence, generaciones y retractación; webhooks sólo con caso de producto |
| 2 Fichas Rust | parcial | A–F desarrollan once campos y seams existentes, alternativas/FMEA/aceptación; E atribuye scope al assembler puro; A requiere aclarar cancelación precommit/C13 |
| 3 Incidencias | sustentado | #3697 abierto Python/latest y #3684 closed/not_planned releídos; histórico no equivale a bug/fix actual Go |
| 4 Prerrequisitos | sustentado | §4 reconoce run_id/rechazo tardío, ownership wiki y checkpoint ya existentes, gates por candidata |
| 5 Exclusiones | sustentado | §5 no obliga PostgreSQL/Temporal/Node, LLM o nueva base ni confunde retirada con purga |
| 6 Método | parcial | §6 delimita evidencia estática, pero carece de inventario final leído/no leído y cierre anunciado |
| 7 Benchmarks | no acreditado | No existe §7; `scripts/eval/` en mapa no acredita runner, datos/defaults/denominadores/errores/reset/costes/adaptación |
| 8 Matriz | no acreditado | No existe §8; referencias a capacidades en prosa no sustituyen matriz por mecanismo y evidencia |
| 9 Negativos | parcial | §4 aporta hechos acotados, pero falta §9 que separe negativos/hipótesis/falsos positivos y revalidaciones |
| 10 UI | no acreditado | No existe §10; enlaces a facade y mapa quivr-search no trazan implementación frontend ni Studio |

### Hallazgos Quivr

- **P1 — informe incompleto en disco.** `quivr.md:20–24,495–509`
  anuncia evaluación, matriz, negativos, frontend y cierre que no están.
  **Consecuencia:** no se puede aceptar el desarrollo de diez ejes ni comprobar
  métricas/UI/dependencias desde este informe autónomo.
  **Corrección:** restituir §7–10 y cierre con fuentes efectivamente leídas:
  entrypoints/cadena de runners, datasets/defaults/métricas/denominadores,
  errores/reintentos/reset/efectos/coste y adaptación; matriz por candidata;
  implementación de demo web/facade/Studio y límites. No inventar desktop
  por el nombre Quivr, ni llamar QA a retrieval ni crear evaluador nuevo.
- **P1 — autorización atribuida al componente equivocado.**
  `quivr.md:351–353` afirma «assemble_with_recall aplica scope antes de
  inclusión». Vanta `context_engine/engine.rs:252–325` recibe mensajes,
  strings de recall y presupuesto; no recibe principal/scope ni ejecuta
  comprobación de permisos. **Mecanismo/consecuencia:** un caller podría
  asumir que pasar texto al ensamblador es suficiente y exponer relaciones
  denegadas. **Corrección:** exigir autorización y resolución por lector en
  host/recall gobernado antes de pasar texto, revalidar al hidratar destino y
  dejar el ensamblador puro. No añadir ACL ficticia ni un segundo assembler.
- **P2 — cancelación y receipt de fuente necesitan frontera precisa.**
  `quivr.md:216–225` propone SourceReceipt y afirma «Cancelar antes de commit
  no acepta». En [submissions.go:215–292](https://github.com/The-Vibe-Company/quivr/blob/bb1b19ecf4094dbd1e309e521ce8b5fed4fb539f/internal/adapters/postgres/submissions.go#L215-L292)
  hay transacción y commit, pero timeout del caller no acredita que el servidor
  no haya aplicado el commit. **Corrección:** distinguir cancelación confirmada
  antes de admisión/commit de interrupción con outcome desconocido; lookup por
  operación/idempotency key. SourceReceipt de dominio complementa C13 propuesto,
  no sustituye primary_commit/durability/projections por una cadena automática.

### Evidencias y límites Quivr

- Clon `submissions.go:205–292`: withdraw transaccional, identidad estable por
  organización/corpus, requestDigest y conflicto de replay, tombstone incluso
  sin materialización y evento sólo en primera transición.
  Commit SQL no acredita recepción webhook ni purga física externa.
- `processing/serving_projection.go:12–101` leído completo: elegibilidad/
  generación/espacios, FillIndependent, Publish, PublishEmbeddings, cobertura y
  complete separados. No atomicidad universal ni dependencia obligatoria LLM.
- `monitoring/webhook.go:15–37` confirma HMAC de id.timestamp.rawbody:
  firma de webhook distinta del hash SHA-256 del WriteReceipt Vanta, que no es
  firma. No se certificó toda protección SSRF/replay ni el resto de plugins.
- `retrieval/retrieval.go:223–279` confirma scope.Require/Contains antes de
  selección, defaults hybrid/default/10 y deadline del perfil. Esa autorización
  upstream no convierte el assembler de Vanta en una frontera de permisos.
- [#3697](https://github.com/The-Vibe-Company/quivr/issues/3697) cuerpo completo,
  abierto; [#3684](https://github.com/The-Vibe-Company/quivr/issues/3684)
  cuerpo completo, closed/not_planned, cerrado 2026-09-16.
  GET público 2026-11-07; no imagen latest, advisory privado ni PoC ejecutados.
  No se deduce fix Go de ese cierre ni enumerabilidad de UUIDv4.
- No se contrastó runner o frontend ausente del dossier como si ya estuviera
  revisado; su restablecimiento exige revisión focal posterior. Seams Vanta
  citados en §4 son integración propuesta, no nueva base/scheduler autorizado.

## Límites de la revisión

Siete recibos guardados, ninguno constituye aceptación final. Se leyeron
completos los siete informes; las fuentes se contrastaron de manera dirigida,
con límites individuales anteriores. No se certifican runtime, seguridad,
durabilidad/crash, rendimiento, calidad QA ni cobertura integral de repositorios.
No se editaron informes ajenos, contratos, índices, código, backlog o JSON;
no apps/tests/builds/benchmarks/install/servicios ni git de escritura.
El archivo de revisión queda en disco pero no aparece en `git diff/status`:
`.gitignore:268` ignora `docs/dev/research/`. No se alteró ignore ni se forzó
staging; corresponde al responsable resolver la incorporación si la autoriza.
La ruta inexistente de CONSTRAINTS.md raíz y el intento fallido de
`src/sdk/admin.rs` se registran como accesos fallidos, no contenido leído.

**División futura:** si se convierten en seguimiento activo, separar correcciones
por proyecto y anexos de evidencia, manteniendo este recibo/TOC como referencia.
No se crean ahora esos documentos, rúbricas, umbrales o tareas activas.
