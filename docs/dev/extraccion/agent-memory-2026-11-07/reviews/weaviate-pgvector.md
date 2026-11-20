---
title: "Revisión independiente — Weaviate y pgvector"
kind: research
status: proposed
description: "Recibo de revisión estática de dos informes, con cobertura por eje, discrepancias contrastadas y límites de evidencia"
tags: [research, review, vector]
---

# Revisión independiente — Weaviate y pgvector

## Índice

- [Alcance y recibo](#alcance-y-recibo)
- [Weaviate — recibo persistente terminado](#weaviate--recibo-persistente-terminado)
- [pgvector — recibo persistente terminado](#pgvector--recibo-persistente-terminado)
- [Cierre de revisión](#cierre-de-revisión)

## Alcance y recibo

Revisión independiente de [Weaviate](../weaviate.md) y
[pgvector](../pgvector.md), con fecha de consulta pública 2026-11-07.
No soy autor de esos informes. No hay aceptación final: los dictámenes
propuestos corresponden exclusivamente a esta revisión documental.

Contratos leídos: [EXPANSION-CONTRACT](../EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../VANTA-REFERENCE.md), [METHOD](../METHOD.md),
[BENCHMARK-NOTES](../BENCHMARK-NOTES.md) y
[REVIEW-COVERAGE](../REVIEW-COVERAGE.md), además del brief temporal asignado.
Skills: catálogo consultado; cargadas `documentation-skill` y
`coordinated-web-search`. No delegación ni agentes adicionales.

Identidades comprobadas mediante `git rev-parse HEAD`, sin fetch:

| Referencia | SHA completo |
|---|---|
| Weaviate | `85bd085e6fd6bcf2c6269c880dc49181c86bbb92` |
| pgvector | `724588a5504b0e9c462e3126563370e8d50a01eb` |
| VantaDB | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` |
| Studio, worktree separado | `0ad38c542b39a589035b0e17fdedb4c09dca24b9` |

La única escritura de esta revisión es este archivo. No apps, tests, builds,
benchmarks, instalaciones, servicios, modificaciones del runtime, índices,
contratos, JSON, backlog, staging, commits ni push.

## Weaviate — recibo persistente terminado

**Lectura:** informe completo, líneas 1-1012, incluidos seis candidatos,
benchmarks y referencias. **Veredicto propuesto: cambios requeridos**, por
la representación de errores de C13 descrita abajo; no por longitud ni por
ausencia de resultados de ejecución.

### Ledger 0 y diez ejes

| Eje | Estado | Sección del informe y contraste |
|---|---|---|
| 0. Identidad, arquitectura y flujos | parcial | §§1-2: pin/licencia por subárbol, fuente→inverted→vector/cola, search y delete contrastados. Orden flush/Wait correcto. Falta distinguir negativa inicial y sustitución de error raíz en el relato del batch: W-1 |
| 1. Recomendaciones | sustentado | §3: C13/C7/C4-C14/C16 y C15/C9 condicionados; no nueva WAL ni sharding obligatorio |
| 2. Fichas Rust | parcial | §4: seis fichas con los once campos, alternativas, FMEA, pasos y rollback. Seams existentes contrastados; tipos nuevos señalados propuestos. Salida/errores C13 incompletamente conectados: W-1 |
| 3. Incidencias | sustentado | §5: bodies #3948/#3949/#4262/#9274, comentarios #4262/#9274 y PR9240 contrastados; closed no equivale a corrección universal |
| 4. Prerrequisitos Vanta | sustentado | §6: batch/WriteReceipt, WAL, rebuild, cuotas/scheduler ya existentes; dependencias por candidato, no bloqueo universal |
| 5. Rechazos y costes | sustentado | §7: no copiar RAFT, logs paralelos, HNSW entero, proxy o SaaS; coste de archivos/handles/backup identificado |
| 6. Advertencias | sustentado | §8: cancelación≠rollback, Flush≠Sync, timestamps≠MVCC, filtros≠auth, pin/licencias y límites explícitos |
| 7. Benchmarks | parcial | §9: SIFT/BM25/Go leídos; tiempos, fórmulas, denominadores y efectos destructivos correctamente separados. Cardinalidad efectiva SIFT omitida: W-2 |
| 8. Matriz comparativa | sustentado | §10: filtros pre/in/post y exact scan existen en Vanta; cuantización≠paridad PQ/BQ/RQ; memoria/Studio no negados |
| 9. Negativos Vanta | sustentado | §11: Result global del batch y predicados schema/conteos son hechos acotados; auditoría deep/WAL/rebuild reconocidos; no atribuye #4262 a Vanta |
| 10. Escritorio/UI | sustentado | §12: demo web/proxy reales, Explorer externo y Studio separado. Bind comodín comprobado estáticamente; no explotación ni escritorio nativo inventados |

### W-1 — Media: C13 no conecta el error por etapa con su salida y omite pérdidas de causa en el upstream

- **Informe:** `weaviate.md:98-137,227-249,261-269`.
  Se dice que `flushWALs` «agrega» fallos y se promete conservar un error
  terminal con recibos de lo aplicado. Sin embargo, el DTO propuesto
  `MutationReceipt` no contiene ni referencia `ItemFailure`, ni se define cómo
  un fallo posterior a admisión devuelve el informe sin caer en el
  `BatchAdmissionError` de la firma.
- **Fuente:** en el pin Weaviate,
  [PutObjectBatch:33-51](https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_write_batch_objects.go#L33-L51)
  replica read-only/cuota a **todos** los ordinales antes de escribir.
  [setErrorAtIndex/checkContext/flushWALs:463-519](https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_write_batch_objects.go#L463-L519)
  asigna, no acumula: flush puede **sobrescribir** una causa previa; el
  `err` del range sombrea al del contexto y se envuelve la causa preexistente,
  mientras los ordinales nil se omiten. El informe ya detecta esa última
  omisión, pero no explica la preservación/pérdida de causa.
  [Vanta put_batch:1289-1303,1497-1529](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1289-L1303)
  distingue rechazo previo sin efectos de fallos tras commits por chunk;
  no proporciona ya un DTO de progreso.
- **Consecuencia:** una futura tarea puede repetir el problema que C13 intenta
  resolver: `Unknown/Failed/Rejected` sin causa recuperable o pérdida del
  progreso al convertir un fallo tardío en `Err`. No se afirma un nuevo bug
  reproducido de Vanta ni se exige resolver el código upstream.
- **Corrección verificable:** completar la traza de negativa inicial y
  sobrescritura, y especificar documentalmente dónde viajan causa raíz,
  fallos secundarios por etapa y error terminal con recibos. Reservar el
  error de admisión a cero efectos, o explicar otro portador equivalente.
  Incluir el caso ya previsto de cancelación/fallo de flush tras un ítem
  fallido, sin inventar un harness ni una segunda WAL.

### W-2 — Baja: el límite SIFT no es la cardinalidad efectivamente leída

- **Informe:** `weaviate.md:672-686`.
- **Fuente:** [readSiftFloat:100-128](https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark/benchmark_sift.go#L100-L128)
  empieza con `i=0`, hace append y solo después corta con `i >= maxObjects`.
  Con archivo suficiente lee `maxObjects+1` vectores. Las llamadas
  [140-141,158-167](https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark/benchmark_sift.go#L137-L167)
  usan el mismo helper para base y queries, luego parten por división entera.
- **Consecuencia:** para los defaults y datos suficientes, 100001 objetos y
  1001 queries leídas, no exactamente 100000/1000. Con varios batches puede
  además descartarse el resto, riesgo que el informe sí menciona. No son
  resultados observados de una corrida.
- **Corrección verificable:** diferenciar argumento nominal, longitud leída
  y cantidad enviada; condicionar el +1 a que no se alcance EOF antes.
  No cambiar el runner ni diseñar nuevos parámetros.

### Evidencia contrastada y límites Weaviate

- **Persistencia/consulta:** `shard_write_batch_objects.go:33-225,298-537`,
  `shard_write_put.go:325-415,503-551`, `vector_index_queue.go:147-235`,
  `shard_read.go:711-925,1034-1045`, `shard_write_delete.go:33-223`;
  `lsmkv/memtable.go:304-343,739-777`, `commitlogger.go:401-452`,
  `bucket.go:2529-2540`. Commitlog previo al árbol no implica fsync;
  async Insert no implica ANN listo; tenant/allow-list no prueban autorización.
- **Lifecycle/reindex:** `shard_lazyloader.go:150-245`,
  `backup.go:416-491`, `inverted_reindex_record.go:28-119`,
  `inverted_reindex_record_store.go:542-566,731-761`,
  `inverted_reindex_reconcile.go:155-245`, `lsmkv/store.go:813-925`,
  `bucket_recover_from_wal.go:255-409`. No nueva inferencia de atomicidad
  multiíndice ni swap de directorios en vivo.
- **Representaciones:** configs flat `194-230`, dynamic `24-115`,
  `helpers/helpers.go:25-43`; fuente canónica de objetos separada de
  representación comprimida. La corrección de bucket por target coincide
  con el helper del pin; no certifica todos los modos de compresión.
- **Runners:** `test/benchmark/benchmark.go:39-155,203-261`,
  `benchmark_sift.go:33-245`; BM25 `cmd/root.go:37-72`,
  `cmd/import.go:87-175`, `cmd/query.go:37-232`,
  `lib/scores.go:20-71`, `lib/durations.go:20-58`;
  HNSW `benchmark_test.go:35-185`, `index_test.go:226-265`;
  WALReplay `bucket_recover_chunked_benchmark_test.go:31-220`.
  BM25 puntúa solo queries con gold/ID; latencia usa completadas. No QA,
  ni throughput concurrente inferido de inversa de la media.
- **Vanta:** `src/sdk/api/memory.rs:1223-1323,1478-1529`,
  `serialization/impl_index.rs:16-90,195-242`,
  `impl_rebuild.rs:16-120`, `search/vector.rs:129-235`,
  `src/wal.rs:1024-1028,1828-1874`, `src/config.rs:750-782`,
  `src/cli_handlers/index.rs:118-156`, `backup.rs:25-131`.
  `WriteReceipt`/`put_certified` existentes no son `MutationReceipt` propuesto;
  atestación puntual tampoco implica durabilidad/readiness heredada.
- **UI:** demo `app.js:1-130`, `start.py:65-151`; Studio
  `vanta.ts:566-579`, `connections/native.rs:657-672`,
  `commands/data.rs:147-159`. La nueva vista C16 necesita DTO/bridge,
  no se trata como ya disponible.
- **Públicas leídas:** API GitHub bodies
  [3948](https://github.com/weaviate/weaviate/issues/3948),
  [3949](https://github.com/weaviate/weaviate/issues/3949),
  [4262](https://github.com/weaviate/weaviate/issues/4262),
  [9274](https://github.com/weaviate/weaviate/issues/9274);
  comentarios completos devueltos por API para 4262/9274;
  metadata PR [9240](https://github.com/weaviate/weaviate/pull/9240) y
  fragmento de diff del helper. Merge acreditado:
  `278683c8a2df92437d62b19cee0f04e8a45ae232`, 2025-10-02.
  [Explorer](https://docs.weaviate.io/cloud/tools/explorer-tool) leído mediante
  `websearch.fetchWebContent`, respuesta no truncada en esta revisión.
- **Límites:** no todos los comentarios 3948/3949, adjuntos, todos los archivos
  de PR9240, providers, `wl/`, protocolos de cluster, ni cada enlace de
  inventario fueron auditados. No son nuevas lagunas bloqueantes: las
  conclusiones históricas y de fix están acotadas al mecanismo contrastado.
  Sin ejecución ni afirmación de paridad de runtime, seguridad o calidad.

## pgvector — recibo persistente terminado

**Lectura:** informe completo, líneas 1-495 de la entrega encontrada
(el cierre narrativo está en 432; las referencias continúan hasta 495).
Cinco fichas de once campos revisadas. **Veredicto propuesto: cambios
requeridos**, por la atribución de firma al recibo Vanta, no por los
comentarios públicos parciales ni por falta de ejecución.

### Ledger 0 y diez ejes

| Eje | Estado | Sección del informe y contraste |
|---|---|---|
| 0. Identidad, arquitectura y flujos | sustentado | §§1-2: extensión 0.8.7/PGXS, tipos/opclasses y límites separados; insert/scan/vacuum HNSW e IVF trazados. FreeOffset→caller→Finish no demuestra ausencia de WAL ni equivale a COMMIT |
| 1. Recomendaciones | sustentado | §3: C7/C15/C9/C16 y C13 condicionado; no motor paralelo ni dependencia universal |
| 2. Fichas Rust | parcial | §4: cinco fichas con once campos, seams, ownership/locks, alternativas, FMEA, pasos y aceptación. C13 no inventa MutationReceipt existente, pero llama firmado al WriteReceipt real: P-1 |
| 3. Incidencias | sustentado | §5: #259 filtrado, #54 proveedores, #461 límites y #678 propuesta iterativa descritos con alcance histórico y límites de comentarios; contraste público independiente indicado abajo |
| 4. Prerrequisitos Vanta | sustentado | §6: preserve filtros/recheck, índice compartido, rebuild secuencial y receipts/certificados. Dependencias condicionadas y sin nueva WAL |
| 5. Rechazos y costes | sustentado | §7: no PostgreSQL embebido alternativo, pipeline de workers, VACUUM Rust ni nuevos índices por analogía |
| 6. Advertencias | sustentado | §8: límites del host, MVCC, no ejecución, snapshots, DoS y privacidad separados; no certificar toda seguridad |
| 7. Benchmarks | sustentado | §9: TAP en test/t, denominadores distintos 003/004/012/042/044, condiciones por operador/probes y tests de no-error separados de calidad; no resultados propios ni nuevos umbrales |
| 8. Matriz comparativa | sustentado | §10: pre/in/postfilter, fallback exacto, cuantización, rebuild/receipts existentes. Precisión/dominio/codec no equiparados |
| 9. Negativos Vanta | sustentado | §11: fallback solo con cero hits y fases con `?` contrastados; alcance L0 explícito. El corte batch→estado se conserva como hipótesis condicionada, no corrupción reproducida |
| 10. Escritorio/UI | sustentado | §12: clientes Postgres externos no implican desktop propio; métricas globales y panel reindex read-only contrastados en Studio; fuente local declarada complementaria |

### P-1 — Media: el WriteReceipt actual no tiene firma digital

- **Informe:** `pgvector.md:227`, ficha C13: «no sustituye el recibo
  **firmado** actual». La distinción de `MutationReceipt` propuesto frente
  a `WriteReceipt` existente es correcta; el calificativo firmado no.
- **Fuente Vanta fijada:** [CertificateIntegrity:92-99](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L92-L99)
  contiene algoritmo/hash SHA-256, no firma ni identidad de firmante.
  [write_declared_limits:726-738](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L726-L738)
  declara expresamente **“no digital signature”** y que un actor capaz de
  recalcular hashes no es detectado. También excluye la garantía automática
  de crash durability antes de sync y la vinculación a una instancia de DB.
  [build_write_receipt:920-954](https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L920-L954)
  construye content binding, referencia de cadena y self-integrity.
- **Consecuencia:** atribuye autenticidad criptográfica frente a adversario
  donde solo se acredita integridad hash y relectura viva; podría contaminar
  la futura especificación C13 o criterios de confianza del consumidor.
  No demuestra una vulnerabilidad del producto: el código sí declara el límite.
- **Corrección verificable:** sustituir «firmado» por «con binding de
  contenido e integridad SHA-256» y preservar los límites del recibo. No
  añadir firma, otro registro ni heredar `Synced/Ready` por portar el receipt.
  El resto de la ficha ya separa aplicación RAM, revisión y checkpoint.

### Evidencia contrastada y límites pgvector

- **Código P:** `hnswinsert.c:49-119,149-290,350-413` confirma inspección
  previa de slots, registro de buffer vecino y Finish posterior a escritura;
  `hnswscan.c:26-91,136-337`, `ivfscan.c:256-433` distinguen TIDs,
  snapshots MVCC, continuación y caps aproximados. No se atribuye
  consistencia eventual ni filtro pushdown universal al AM.
- **IVF escritura/mantenimiento:** `ivfinsert.c:20-216`,
  `ivfvacuum.c:18-163`: centros existentes, append/WAL de páginas y
  reclamación de offsets no son reentrenamiento. HNSW
  `hnswvacuum.c:618-737,889-934` confirma cinco pasadas, ERROR de
  referencias no reparadas y barrera de scan antes de reutilizar.
- **Build/tipos:** `hnswbuild.c:479-545,680-712,1120-1177`,
  `ivfbuild.c:429-474,1000-1073`, `hnswutils.c:1606-1702`;
  diferencias entre WAL incremental/build, memoria y límite nnz/dimensión
  contrastadas. `Makefile:1-62`, `vector.control` y `LICENSE` completos.
- **TAP contrastados sin ejecutarlos:** 003/004/012/042/044 en los rangos
  citados por el informe; 013:55-110, 014:50-97, 010:10-98,
  041:7-54, 043:7-59, 038:7-42, 045:7-29, 046:7-38,
  047:7-39, 048:7-39. Los denominadores y condicionales sensibles sí
  coinciden; pgbench pesa operaciones, no garantiza igual número de cada una.
  Igualdad primario/réplica tras catch-up no es ground truth ANN.
  No se auditó entero cada TAP restante ni el coste real de sus runners.
- **Vanta V:** además de las fuentes de Weaviate,
  `search/vector.rs:240-312`, `api/admin.rs:19-40`,
  `serialization/mod.rs:190-203`, `api/memory.rs:1029-1045,1686-1755`,
  `storage/engine/maintenance.rs:35-92,463-565,604-645`,
  `ops.rs:163-240`, `attestation.rs:92-99,667-703,726-774,920-954`.
  Flush ocurre antes de tomar insert_lock en rebuild; L0 no se
  generaliza a todos los segmentos. Cola vacía no prueba aplicación de
  cada revisión si el drain salió antes por `?`.
- **Studio complementario:** `commands/metrics.rs:17-32`,
  `SopPanel.tsx:143-175`, `IndicesLens.tsx:67-112`,
  `useMetricsPoll.ts:1-66`. Confirma atomics globales,
  reindex sin trigger por ese panel, fallback limitado y poller compartido.
  No se verificó UI visual, permisos ni todas las conexiones.
- **Fuentes públicas:** bodies/metadata REST de
  [259](https://github.com/pgvector/pgvector/issues/259),
  [54](https://github.com/pgvector/pgvector/issues/54) y
  [461](https://github.com/pgvector/pgvector/issues/461) leídos y
  estados/fechas coincidentes. La siguiente petición (#678) y comentarios
  REST devolvieron **rate limit**; una variable PowerShell retenida volvió
  a imprimir #461 y **no** se considera respuesta #678.
  Fallback `websearch.fetchWebContent` a las páginas oficiales
  [678](https://github.com/pgvector/pgvector/issues/678) y
  [461](https://github.com/pgvector/pgvector/issues/461) permitió leer
  cuerpo y comentarios iniciales; en 461 hubo truncamiento explícito.
  En 678 confirma Closed, nombres streaming históricos, batches y discusión
  nkeys=0; no revalida por REST sus timestamps ni todo el historial de cierre.
  No es bloqueo: el informe no deduce merge/fix universal de esos comentarios.
- **Host:** [Generic WAL](https://www.postgresql.org/docs/current/generic-wal.html)
  e [index locking](https://www.postgresql.org/docs/current/index-locking.html)
  leídos mediante fetch oficial no truncado; explican copias de página,
  Finish, relaciones unlogged y seguridad del salto índice→heap con MVCC.
  No se auditó todo executor, COMMIT, recovery PostgreSQL ni proveedores.
- **Límite de veredicto:** no certifica paridad ACID, auth/RLS, retención física,
  calidad, latencia ni todas las rutas/bindings Vanta. No se exige completar
  todos los comentarios o nuevas lagunas universales para corregir P-1.

## Cierre de revisión

Ambos proyectos tienen recibo persistente, ledger 0+10, fuentes, límites y
dictamen propuesto. Hallazgos a resolver por autor/responsable: **W-1 y P-1
(media)**; **W-2 (baja)**. No se editaron los informes revisados ni se
aprobaron propuestas. La aceptación final y la relectura de sus correcciones
corresponden al responsable principal.

Operaciones realizadas: lecturas `Get-Content`/`Get-ChildItem`, consultas
read-only `git rev-parse`/`git grep`, descubrimiento MCP `list_servers` y
`list_tools(websearch)`, fetch público y `Invoke-RestMethod`, y parches
absolutos solo a este archivo. No comandos de ejecución de proyectos.
Incidencias de tooling: alias PowerShell `R` colisionó con Invoke-History
y se sustituyó por helper lector `ShowLines`; `rg` no estaba instalado y se
usó `git grep`; rate limit público registrado arriba. Ningún fallo se
presenta como lectura de evidencia exitosa.
