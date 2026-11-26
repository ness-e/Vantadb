---
title: "Fase 2 — Arquitectura, riesgos y deuda técnica"
kind: report
description: "Auditoría estática por áreas con FMEA cualitativo, evidencia de código y comprobaciones pendientes."
---

# Fase 2 — Arquitectura, riesgos y deuda técnica

[HECHO] Objeto auditado: worktree en HEAD `08c6dc603c009edf8d9b76cee14e7006a8a89ad7`, con las exclusiones de [fase 1](fase1-estadisticas.md). Hubo cuatro revisiones estáticas por áreas —core, FFI, MCP/ecosistema e infraestructura— y contraste del lead mediante CodeGraph y lectura de fuente actual. **No se ejecutaron tests, builds, fault injection, benchmarks ni servidores.**

[HECHO] El estado CBM señala cambios de metadata en las rutas críticas consultadas; se usó fuente del disco para contrastarlas. Los mensajes «no covering tests found» del grafo no se interpretan como prueba de ausencia de tests. La cobertura de esta auditoría es dirigida por riesgo, no una demostración exhaustiva de seguridad o corrección.

## Criterio de evidencia y FMEA

[HECHO] `[HECHO]` describe código/configuración observado; `[INFERENCIA]` describe la consecuencia condicionada que se deduce; `[ESPECULACIÓN]` se reserva para hipótesis no resueltas. La existencia de una ventana de fallo no equivale a haber reproducido pérdida de datos.

[INFERENCIA] Prioridad: **P0 validar antes de asegurar durabilidad**, P1 antes de ampliar uso sensible/servicio, P2 consolidación de contratos/operación, P3 mantenimiento. FMEA cualitativo: modo de fallo, desencadenante, efecto, control y prueba discriminante. No se calcula RPN: no hay frecuencias observadas para puntuar ocurrencia ni tasas medidas de detección.

## 1. Arquitectura y rutas críticas

### Mapa de componentes

| Área | [HECHO] Implementación observada | [INFERENCIA] Implicación |
|---|---|---|
| SDK | [Embedded](../../../../src/sdk/builder.rs#L15), tipos y API en [src/sdk](../../../../src/sdk/) | La API embebida organiza dominios; no todos se exponen por FFI |
| Storage | [StorageEngine](../../../../src/storage/engine/mod.rs), backends [Fjall/RocksDB/memoria](../../../../src/backends/) | Combinar WAL, KV, VantaFile e índices exige un protocolo común de recuperación |
| Índices | [index](../../../../src/index/), [index_port](../../../../src/index_port.rs), [text_index](../../../../src/text_index.rs), [scalar_index](../../../../src/scalar_index.rs) | Abstracción de índice y leaf neutral reducen algunos ciclos; las estructuras secundarias requieren sincronización |
| Retrieval | [search](../../../../src/sdk/search/mod.rs#L73), [fusion](../../../../src/sdk/search/fusion.rs), [search_profile](../../../../src/search_profile.rs#L20) | ANN y lexical se fusionan; score mayor-es-mejor no es distancia ANN |
| Grafo | [graph](../../../../src/graph.rs), [GraphRAG](../../../../src/graphrag/pipeline.rs), [gds](../../../../src/gds.rs) | Hay almacenamiento/traversals y recuperación relacional, no solo vectores |
| Consulta | [parser](../../../../src/parser/mod.rs#L20), [planner](../../../../src/planner.rs), [physical_plan](../../../../src/physical_plan/), [executor](../../../../src/executor.rs) | IQL posee planificación y operadores propios; su superficie no debe deducirse de SQL genérico |
| Memoria | [governor](../../../../src/governor.rs), [memory_governor](../../../../src/memory_governor.rs), [eviction](../../../../src/eviction.rs), [gc](../../../../src/gc.rs) | Límites lógicos y RSS deben distinguirse; mmap no equivale a memoria gratis |
| Cognitiva | [vanta-memory](../../../../vanta-memory/src/lib.rs) | Pipeline L0–L3 separado del engine y dependiente del host; superficie Python mínima |
| Transporte | [FFI leaf](../../../../vantadb-ffi-core/src/lib.rs), bindings, [HTTP](../../../../src/server/router.rs), [MCP](../../../../vantadb-mcp/src/server.rs) | Contratos, lifecycle y garantías difieren según runtime |

[HECHO] El core declara feature gates y reexportaciones en [src/lib.rs:63–241](../../../../src/lib.rs#L63-L241). [INFERENCIA] Que un módulo exista en disco no significa que se compile por defecto, que sea público o que esté disponible en WASM.

### Escritura y recuperación

[HECHO] La escritura toma `insert_lock`, añade WAL, aplica a KV/vstore/índices y drena HNSW bajo la política de lock correspondiente: [insert.rs](../../../../src/storage/engine/insert.rs), [LockPolicy](../../../../src/storage/engine/mod.rs), [commit_transaction:309–382](../../../../src/storage/engine/txn.rs#L309-L382). El mantenimiento usa el mismo dominio de coordinación para evitar checkpoints adelantados a operaciones pendientes: [maintenance.rs](../../../../src/storage/engine/maintenance.rs).

[HECHO] `SyncMode::Always` sincroniza, `Never` delega en page cache salvo sync explícito y `Periodic` aplica umbral: [wal.rs:999–1020](../../../../src/wal.rs#L999-L1020). [INFERENCIA] No se puede prometer el mismo RPO para toda configuración porque todas se llamen «persistentes».

[HECHO] Al abrir, se lee `checkpoint_seq` y se reconcilia el número de shards: [init.rs:562–576](../../../../src/storage/engine/init.rs#L562-L576). La rama sharded lee cada fichero activo mediante `next_record`, reconstruye secuencia global y verifica recuentos entre shards: [init.rs:618–661](../../../../src/storage/engine/init.rs#L618-L661).

[HECHO] La recuperación marca como descartables batches sin Commit correspondiente y batches abortados: [init.rs:663–712](../../../../src/storage/engine/init.rs#L663-L712). [INFERENCIA] Esto controla el replay WAL; por sí solo no demuestra rollback de cambios que ya hayan llegado a otros soportes persistentes.

### Controles positivos del core

1. [HECHO] Lock de escritura compartido y variantes `AssumeHeld` evitan readquirir un lock no reentrante en rutas ya protegidas: [txn.rs:309–357](../../../../src/storage/engine/txn.rs#L309-L357), [maintenance.rs:239–250](../../../../src/storage/engine/maintenance.rs#L239-L250).
2. [HECHO] La recuperación multi-shard detecta secuencias truncadas en lugar de aceptar silenciosamente cualquier prefijo: [init.rs:614–659](../../../../src/storage/engine/init.rs#L614-L659).
3. [HECHO] La rotación hace flush/sync del WAL y sincroniza directorio después del rename: [wal.rs:1086–1120](../../../../src/wal.rs#L1086-L1120). [INFERENCIA] Es un control de persistencia del fichero, no prueba de replay de todos los segmentos.
4. [HECHO] CRC/versionado, formatos de almacenamiento y rutas de cifrado están representados en [wal.rs](../../../../src/wal.rs), [binary_header.rs](../../../../src/binary_header.rs), [crypto](../../../../src/crypto/) y [contrato de formatos](../../../api/STORAGE_FORMATS.md).
5. [HECHO] Hay controles de presupuesto, backpressure y métricas en [Config](../../../../src/config.rs#L719-L789), [governor](../../../../src/governor.rs) y [métricas](../../../../src/metrics/). [INFERENCIA] Son mecanismos valiosos, pero sus umbrales efectivos y costes bajo presión necesitan carga real.

## 2. FMEA del core

### P0 de validación: segmentos WAL rotados y coordenadas de replay

[HECHO] `try_auto_rotate` renombra el WAL a `<nombre>.<millis>` y abre un WAL nuevo en la ruta original: [wal.rs:1086–1120](../../../../src/wal.rs#L1086-L1120). `batch_append` llama a `maybe_sync` y después a esa rotación: [wal.rs:988–996](../../../../src/wal.rs#L988-L996). Hay enumeración explícita de rotados en [wal_sharded.rs:850–873](../../../../src/wal_sharded.rs#L850-L873).

[HECHO] En el bucle de recuperación inspeccionado se abren las rutas activas por shard y se calculan posiciones a partir de `checkpoint_seq`; ese bucle no recorre los archivos rotados: [init.rs:618–651](../../../../src/storage/engine/init.rs#L618-L651).

[INFERENCIA] **Escenario de riesgo, no pérdida reproducida:** al cruzar el tamaño de segmento antes de que todos los soportes e índices cubiertos estén en un checkpoint durable coherente, un crash podría dejar operaciones necesarias solo en segmentos que ese replay no lee. También debe comprobarse la coherencia entre checkpoint acumulado y posiciones locales tras reiniciar segmentos.

[HECHO] Control existente: WAL sincronizado antes del rename, backends/segmentos pueden conservar parte o todo el estado, y existen rutas de reconstrucción. [INFERENCIA] Estos controles impiden afirmar «cada rotación pierde datos»; no eliminan por inspección la necesidad de demostrar el invariante.

[INFERENCIA] Prueba discriminante para una tarea autorizada: umbral pequeño, uno y varios shards, insert/update/delete con secuencia de referencia, crash después de rotación y antes de checkpoint; reabrir y comparar KV, vstore, HNSW, BM25 y deletes. Separar caso `Always` de `Never`; verificar que lo reconocido como durable reaparece o produce error explícito, nunca omisión silenciosa.

[INFERENCIA] Mitigación de diseño a decidir después del reproducer: replay ordenado de segmentos con coordenadas persistidas o checkpoint/retención coordinados antes de retirar el segmento activo del replay. **No se implementa aquí** ni se recomienda borrar archivos rotados.

### P1: Commit durable posterior a aplicar los stores

[HECHO] `commit_transaction` construye Begin+ops+Prepare, escribe phase-1 bajo lock, luego modifica stores con stamps MVCC y tombstones; drena HNSW y añade Commit al final: [txn.rs:296–382](../../../../src/storage/engine/txn.rs#L296-L382). En fallo de aplicación escribe Abort, pero esta ruta no muestra una operación compensatoria completa sobre cada soporte.

[INFERENCIA] **Escenario condicionado:** crash o error después de aplicar parte de la transacción y antes de Commit durable podría producir estado físico no committed que el filtro del WAL descarta. El riesgo necesita contrastar MVCC, visibilidad y reconstrucción; el mero orden observado **no basta** para afirmar que toda lectura pública verá datos no committed.

[INFERENCIA] Prueba discriminante: failpoints después de cada put/stamp/tombstone y antes/después de Commit, tanto con overwrites/deletes de registros preexistentes como con inserts nuevos; reabrir y comprobar atomicidad lógica y física, además de reads con snapshot. El caso de rollback en memoria sin crash no cubre este escenario.

[INFERENCIA] No cambiar de forma simplista a «Commit antes de apply»: podría trasladar el problema a reintentos, errores de aplicación y visibilidad. Requiere una decisión explícita de recovery/redo/undo/MVCC y tests que demuestren el contrato.

### Otros riesgos dirigidos

| Prioridad y modo | [HECHO] Evidencia | [INFERENCIA] Efecto condicionado / control / comprobación |
|---|---|---|
| P2: layout mixto | [detect_shard_count:34–37](../../../../src/wal_sharded.rs#L34-L37) devuelve 1 si existe WAL base, antes de explorar shards | Con base residual más shards válidos podría elegirse layout incorrecto. No se demostró que una apertura normal cree esa coexistencia; probar fixture mixto, rechazo explícito y migración |
| P2: metadatos checkpoint | Lectura postcard con fallback cero en [init.rs:565–568](../../../../src/storage/engine/init.rs#L565-L568); se detectaron rutas de escritura postcard/raw little-endian en mantenimiento | Revisar uniformidad antes de atribuir fallo de recovery; un fallback a cero puede provocar replay adicional. Hace falta demostrar qué escritor/lector se combina y con qué valor |
| P3: prelectura de delete | [delete.rs:38–60](../../../../src/storage/engine/delete.rs#L38-L60) captura nodo antes de tomar insert_lock; estadísticas se actualizan después | Un overwrite concurrente puede dejar una imagen anterior para índices/contadores. El delete principal sí se serializa; no se etiqueta como pérdida principal de KV sin interleaving reproducido |
| P2: memoria y cuotas | [Config:748–788](../../../../src/config.rs#L748-L788), [memory governor](../../../../src/memory_governor.rs), [vector governor](../../../../src/vector/governor.rs) | RSS, caches, mmap y memoria de otros runtimes no son una única magnitud. Medir picos y rechazos, especialmente batch/rebuild/import; cuota de bulk import tiene exención declarada |
| P2: retrieval bajo filtros | [hybrid](../../../../src/sdk/search/hybrid.rs), [fusion](../../../../src/sdk/search/fusion.rs), [page](../../../../src/sdk/search/page.rs) | Filtros, TTL, no finitos, abstención y top-k deben compartir contrato. No se encontró evidencia suficiente aquí para declarar un bug concreto de ranking ni ventaja frente a competidores |
| P2: DSL/operadores | [parser](../../../../src/parser/), [physical_plan](../../../../src/physical_plan/), [executor](../../../../src/executor.rs) | Revisar presupuestos, cancelación y límites en joins/aggregates con datos adversariales; no extrapolar capacidades de SQL a IQL |

[HECHO] La revisión dirigida no produjo perfiles CPU/RSS ni resultados de recall actuales. [INFERENCIA] La priorización de rendimiento debe empezar por un workload reproducible, no optimizar un lock o índice por su nombre.

## 3. FFI y contratos entre runtimes

### Fortalezas y diferencias reales

[HECHO] `vantadb-ffi-core` concentra `OpGate`, `OpGuard` y `clamp_top_k` sin dependencias normales: [lib.rs:72–154](../../../../vantadb-ffi-core/src/lib.rs#L72-L154), [Cargo.toml](../../../../vantadb-ffi-core/Cargo.toml). Su contrato exige no bloquear con locks externos y explica los límites WASM y GIL.

[HECHO] Python desprende trabajo del GIL; Node usa snapshots de handles y `spawn_blocking`; WASM combina engine síncrono con persistencia asíncrona, y TS adapta errores/tipos: [Python](../../../../vantadb-python/src/lib.rs), [Node](../../../../vantadb-node/src/lib.rs), [WASM](../../../../vantadb-wasm/src/lib.rs), [TS](../../../../vantadb-ts/src/vantadb.ts).

[HECHO] IDs `u128` usan strings decimales/BigInt en JS donde corresponde; no se sustituyen por Number para datos por encima de 2^53. La matriz y el escenario de paridad declaran excepciones: [BINDINGS_NAMESPACES](../../../api/BINDINGS_NAMESPACES.md), [scenario.json](../../../../tests/parity/scenario.json), [comparador](../../../../dev-tools/parity-compare.mjs).

| Superficie | [HECHO] Modelo de ejecución | [HECHO] Alcance/límite |
|---|---|---|
| Python | Engine síncrono con detachment; wrapper async con threads | Mayor superficie de dominio y hooks cognitivos mínimos |
| Node nativo | Métodos Promise y trabajo bloqueante apartado | No miembro del workspace; divergencias IQL/ANN declaradas |
| WASM | Operaciones engine síncronas, persistencia OPFS/IDB asíncrona | Sin drain bloqueante en wasm32-unknown-unknown |
| TS | Clase actual `Client`; wrapper WASM y alternativa native | No es core Rust ni capa cognitiva completa |
| HTTP/MCP | Páginas/envelopes para búsqueda | Pueden transportar señal de abstinencia no presente en arrays de bindings |

### FMEA de frontera

| Prioridad | [HECHO] Evidencia | [INFERENCIA] Riesgo y acción |
|---|---|---|
| P1/P2: cierre sin código canónico en gate | [WASM enter:491–494](../../../../vantadb-wasm/src/lib.rs#L491-L494) arroja string; [Node enter:1404–1407](../../../../vantadb-node/src/lib.rs#L1404-L1407) error genérico | El límite compartido no asegura por sí solo `VANTADB_CLOSED`; probar cierre/operación en vuelo y llamada FFI directa, no solo precheck del wrapper |
| P2: fallback de errores TS | [errors.ts:112–159](../../../../vantadb-ts/src/errors.ts#L112-L159) no clasifica «database is closing» como CLOSED y mapea «serialization error» a CORRUPT | Si esos mensajes llegan sin code, el fallback es WASM_ERROR/CORRUPT. No afirmar corrupción real del backend por fallo de serialización |
| P2: Node serialization | [serde_map_err:1398–1400](../../../../vantadb-node/src/lib.rs#L1398-L1400) no preserva enum de error del core | Separar invalid input/serialize/transport/corruption con códigos; mantener tests para mensajes legacy |
| P1/P2: WASM close | [FFI contrato:44–58](../../../../vantadb-ffi-core/src/lib.rs#L44-L58), [WASM persistence](../../../api/WASM_PERSISTENCE.md) | Drain activa barrera pero no espera sincrónicamente las tareas del event loop. Precisar close/flush y probar async en vuelo; no prometer semántica idéntica a nativo |
| P2: señal top-k | [clamp_top_k](../../../../vantadb-ffi-core/src/lib.rs), [binding Python](../../../../vantadb-python/src/lib.rs) usa tracing para aviso | Logging depende de subscriber; no equivale a warning Python garantizado. Probar canal documentado por runtime |
| P2: paridad parcial | [scenario](../../../../tests/parity/scenario.json), [job parity de CI Fleet](../../../../.github/workflows/ci-fleet.yml#L191-L286) | Un escenario común protege operaciones cubiertas, no equivalencia completa de API, errores, crash o lifecycle |

[HECHO] El gate TS puede rechazar operaciones cerradas antes de cruzar la frontera. [INFERENCIA] Por eso se distingue el hecho «error FFI no estructurado» del claim más fuerte «toda llamada pública tras close devuelve código incorrecto», que no queda demostrado.

[ESPECULACIÓN] Bloqueo de `gate.drain()` en un executor Node y efectos de finalizers merecen runtime testing; no se presentó un deadlock reproducido ni se afirma específicamente bloqueo del hilo libuv sin traza.

[INFERENCIA] Recomendación: matriz de lifecycle/errores junto a paridad de datos, con casos exactos: close idempotente, operación nueva tras cierre, operación en vuelo, serialización inválida, top-k fuera de rango, IDs >2^53 y diferencias declaradas. No modificar aquí bindings ni snapshots canónicos.

## 4. MCP y adaptadores de ecosistema

### Controles positivos

1. [HECHO] JSON-RPC conserva la distinción de notificación/ID y serializa escrituras stdout; tareas en vuelo se drenan: [server.rs:489–538](../../../../vantadb-mcp/src/server.rs#L489-L538), [protocol.rs](../../../../vantadb-mcp/src/protocol.rs).
2. [HECHO] Concurrencia y timeout están en el dispatcher directo; el permiso se mantiene dentro del trabajo bloqueante: [server.rs:559–627](../../../../vantadb-mcp/src/server.rs#L559-L627). [INFERENCIA] Evita liberar capacidad mientras el trabajo continúa, aunque no resuelve cancelación.
3. [HECHO] Validación de identificadores/argumentos y error envelopes tienen implementaciones separadas: [validation.rs](../../../../vantadb-mcp/src/validation.rs), [error.rs](../../../../vantadb-mcp/src/error.rs), [handlers/tools.rs](../../../../vantadb-mcp/src/handlers/tools.rs).
4. [HECHO] La gobernanza de inyección usa presupuesto/taint y auditoría de metadata, no volcado indiscriminado del contenido: [governance.rs](../../../../vantadb-mcp/src/governance.rs), [config.rs:192–218](../../../../vantadb-mcp/src/config.rs#L192-L218).
5. [HECHO] Fallback multi-instancia implementa writer guard/discovery y listener loopback: [proxy.rs](../../../../vantadb-mcp/src/proxy.rs), [server.rs:171–252](../../../../vantadb-mcp/src/server.rs#L171-L252). [INFERENCIA] Mantiene la exclusión de escritor sin requerir abrir el mismo motor varias veces.

### FMEA y resolución de una hipótesis demasiado amplia

[HECHO] La segunda instancia construye `McpConfig` solo con `max_concurrency` y defaults: [server.rs:864–870](../../../../vantadb-mcp/src/server.rs#L864-L870). `from_storage` sí lee perfil, byte budget, namespaces, taint y auditoría: [config.rs:177–218](../../../../vantadb-mcp/src/config.rs#L177-L218). `tools/list` se responde localmente en modo proxy: [server.rs:657–679](../../../../vantadb-mcp/src/server.rs#L657-L679).

[INFERENCIA] Hay **riesgo de deriva de superficie/configuración local en proxy**, especialmente perfil anunciado y comportamiento temporal. Sin embargo, no se concluye que se anule toda gobernanza: el **writer** vuelve a construir config mediante `from_storage` y pasa esa config al handler remoto: [server.rs:135–156](../../../../vantadb-mcp/src/server.rs#L135-L156). Esto es un control discriminante que limita la hipótesis inicial de «proxy ignora ACL/taint».

| Prioridad / modo | [HECHO] Evidencia | [INFERENCIA] Efecto y prueba |
|---|---|---|
| P1/P2: config anunciada en proxy | Defaults de segunda instancia frente a env de writer | Comparar tools/list y tool calls con perfiles/budget/taint diferentes por proceso; un listado no es una barrera de autorización |
| P1/P2: timeout/cancelación | [Directo:593–603](../../../../vantadb-mcp/src/server.rs#L593-L603) usa request_timeout; [proxy:672–695](../../../../vantadb-mcp/src/server.rs#L672-L695) delega al cliente HTTP | Tiempo de espera no homogéneo y trabajo bloqueante no cancelable. Probar pool saturado, desconexión y fin del trabajo, no solo respuesta timeout |
| P2: listener más amplio | [server.rs:237–248](../../../../vantadb-mcp/src/server.rs#L237-L248) fusiona router HTTP con proxy | No es únicamente stdio ni un endpoint mínimo. Revisar modelo de confianza local y autenticación; loopback no es exposición remota demostrada |
| P2: auth configurable | [server.rs:118–133](../../../../vantadb-mcp/src/server.rs#L118-L133) verifica Bearer si existe clave y compara strings | No elevar a bypass remoto sin escenario; definir amenaza entre procesos/usuarios locales, permisos de discovery y comparación de secretos |
| P2: perfil inválido | [config.rs:178–181](../../../../vantadb-mcp/src/config.rs#L178-L181) parse inválido cae a default | Typo puede ampliar/reducir herramientas anunciadas sin error. Probar valores inválidos y exigir fallo o warning inequívoco en cambio posterior |
| P2: discovery/versionado | [proxy.rs](../../../../vantadb-mcp/src/proxy.rs), `ServerInfo`, `try_proxy`, `cleanup_stale` | Version mismatch y carreras de cleanup requieren pruebas entre procesos; PID vivo/puerto abierto no prueban compatibilidad ni identidad suficiente por sí solos |
| P2: adapter async | [LangChain vectorstore.py](../../../../integrations/langchain/vantadb_langchain/vectorstore.py), `adelete` | Async que llama delete síncrono puede bloquear event loop; probar con heartbeat y trabajo lento controlado |
| P2: batch de adapter | `add_texts` del mismo adaptador y [LlamaIndex](../../../../integrations/llamaindex/vantadb_llamaindex/vectorstore.py) | Varios puts individuales no prometen atomicidad; documentar fallo parcial y usar bulk/txn solo con contrato adecuado |

[HECHO] El MCP `vantadb` configurado en este entorno no permitió listar herramientas, también en el reintento final. [INFERENCIA] No se certifica dogfooding ni se interpreta el fallo como ausencia del servidor o de sus datos.

## 5. CI/CD, releases, docs y evals

### Fortalezas verificables de diseño

1. [HECHO] Workflows de publicación usan acciones fijadas, permisos acotados y rutas OIDC/Trusted Publishing: [release](../../../../.github/workflows/release.yml), [wheels](../../../../.github/workflows/release-wheels.yml), [npm WASM](../../../../.github/workflows/release-npm-wasm.yml). [INFERENCIA] Reduce superficie de supply chain; no sustituye revisar secrets/scopes en un release real.
2. [HECHO] [release-plz.toml](../../../../release-plz.toml) y gates de API/versiones mantienen política semver y superficies: [public_api](../../../../tests/api/public_api.rs), [OpenAPI parity](../../../../tests/api/openapi_yaml_parity.rs). [INFERENCIA] Es una base sólida para evitar drift si los jobs son realmente requeridos y cubren cada artefacto.
3. [HECHO] [docs-gates](../../../../.github/workflows/docs-gates.yml) y [scripts/docs](../../../../scripts/docs/) verifican frontmatter, links, índices y contratos relacionados. [INFERENCIA] La regeneración automatizada evita un inventario editorial manual que envejece.
4. [HECHO] Eval tooling incluye procedencia de datasets, aislamiento de namespaces y scoring determinista: [memory_harness.py](../../../../evals/memory_harness.py), [eval_longmemeval.py](../../../../evals/eval_longmemeval.py), [calibration.py](../../../../evals/calibration.py). [INFERENCIA] Facilita reproducibilidad, pero no demuestra calibración externa ni generalización.

### Riesgos y deuda de operación

| Prioridad | [HECHO] Evidencia localizada | [INFERENCIA] Consecuencia y recomendación |
|---|---|---|
| P1: calidad de eval no certificada por CI observado | [evals](../../../../evals/), [.github/workflows](../../../../.github/workflows/), gates revisados no mostraron ejecución del umbral recall_all@5≥0,85 | No promover cifra a garantía de release; localizar job/artefacto requerido o declarar manual. No afirmar ausencia absoluta de CI por búsqueda incompleta |
| P1/P2: calibración y muestreo | [calibration.py](../../../../evals/calibration.py), [eval_longmemeval.py](../../../../evals/eval_longmemeval.py), selección de subsets | Ajustar abstención sobre datos evaluados y usar primeros N puede sesgar resultados. Proponer holdout/estratos antes de cambiar thresholds; no escribir harness o parámetros nuevos aquí |
| P2: docs locales no viajan al clone | AGENTS referencia `docs/dev/workflow/`; [.gitignore](../../../../.gitignore) los excluye | El documento existe en disco, pero no en checkout limpio. No confundir ignorado con inexistente; planificar durable source sin modificar ignore aquí |
| P2: enforcement de missing_docs | [Cargo lints](../../../../Cargo.toml), [ffi-core](../../../../vantadb-ffi-core/src/lib.rs), [CI Rust](../../../../.github/workflows/ci-rust.yml) | Distinguir warn de deny y qué clippy se ejecuta por crate; no afirmar enforcement global uniforme |
| P2: sanitizers no bloqueantes | [nightly](../../../../.github/workflows/nightly.yml), [platform-ops](../../../../.github/workflows/platform-ops.yml) | TSan/continuaciones toleradas no certifican ausencia de races; conservar diagnóstico y decidir explícitamente cuándo debe bloquear |
| P2: verificación condicional local | [verify_changed.ps1](../../../../dev-tools/verify_changed.ps1) | Rutas que saltan scripts ausentes pueden dar falsa confianza; reportar skip como degraded, no PASS |
| P2: required checks y excepciones ADR | [ci-gate](../../../../.github/workflows/ci-gate.yml), [pre-push](../../../../.githooks/pre-push) | Listas mantenidas a mano y bypass dependiente del revisor pueden derivar; contrastar con protección remota antes de certificar un merge |
| P2: publicación y fórmula | [release-verify](../../../../.github/workflows/release-verify.yml), [Formula](../../../../Formula/) | Actualización manual de fórmulas/distribución exige comprobación separada; metadata de registry no valida paquete/plataforma |
| P2: superficie documental stale | [SURFACE_MANIFEST](../../../api/SURFACE_MANIFEST.md) frente a [npm Node](https://registry.npmjs.org/vantadb-node), 2026-11-06 | Node ya tiene 0.9.0 publicado; el manifiesto aún dice planeado/no publicado. Reconciliar generador y authority en tarea posterior |

[HECHO] No se verificaron en remoto branch protections, secretos, ejecución de Actions ni resultados de release de todas las plataformas. [INFERENCIA] Las fortalezas anteriores describen controles presentes en archivos, no una certificación de que el sistema de entrega esté completamente verde.

## 6. Priorización sostenible y dependencias en cascada

| Orden | [INFERENCIA] Acción | Dependencia / criterio de cierre |
|---|---|---|
| 1 | Reproducir o descartar rotación/recovery y ventana apply-before-Commit | Oracle independiente y crash checkpoints con SyncMode conocido; evidencia antes de cambiar protocolo |
| 2 | Fijar contrato de lifecycle/errores por binding | Casos gate/close/async y serialización; mantener compatibilidad de wrappers y matriz |
| 3 | Unificar semántica de config/timeout writer-proxy | Preservar gobernanza efectiva del writer; probar segunda instancia y saturación |
| 4 | Hacer visibles calidad de eval y checks degradados | Holdout definido, evidencia requerida en release; no bajar el quality bar para pasar |
| 5 | Corregir autoridad documental/publicación y rutas de clone limpio | Manifiesto actual vs registry, generación reproducible, docs normativas distribuidas |

[INFERENCIA] Cambiar WAL afecta init/checkpoint/shipping/redaction/cifrado y formatos; cambiar FFI afecta errores, tipos y clientes publicados; cambiar MCP afecta discovery/auth/governance y recursos; tocar eval thresholds afecta comparativas y marketing. Ninguno admite un parche aislado sin evaluar esa cascada.

[HECHO] No se aplicó ninguna mitigación al código. No se crearon tests ni se alteraron thresholds, workflows, `.opencode/`, manifiestos o políticas de licencia en esta misión.

## 7. Qué validar después y qué no afirmar aún

[INFERENCIA] Las suites existentes son un primer control de regresión, **no** sustituyen los nuevos reproducer de los escenarios exactos:

```powershell
cargo nextest run -p vantadb --test durability_recovery
cargo nextest run -p vantadb --test public_api
```

[HECHO] No se ejecutaron esos comandos. Los correrá el usuario si procede; pueden requerir toolchain/entorno y compilación. Los tests nuevos de crash/rotación/FFI deben prepararse en una tarea de código separada, con autorización.

[HECHO] Permanecen no verificados: atomicidad bajo los crash points señalados; recuperación de todos los soportes tras rotación; paridad completa de lifecycle; rendimiento/recall/RSS recientes; seguridad adversarial completa; dogfooding MCP; cobertura exhaustiva de workflows y branch protection.

[INFERENCIA] Veredicto: arquitectura y maquinaria de contratos sustanciales, con riesgos de durabilidad que justifican prioridad máxima de comprobación. **No equivale a «datos perdidos confirmados», «inseguro en toda configuración» ni «listo para SLA»**.

[HECHO] Navegación: [reporte maestro](REPORT.md) · [estadísticas](fase1-estadisticas.md) · [adopción](fase3-adopcion.md) · [finanzas](fase4-financiero.md) · [wiki](../../../wiki/index.md).
