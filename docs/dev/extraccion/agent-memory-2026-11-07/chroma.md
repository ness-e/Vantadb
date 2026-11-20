---
title: "Informe estático — chroma-core/chroma"
kind: research
status: proposed
description: "Lectura de clientes, log, materialización, filtros, compactación e integridad de Chroma y propuestas contractuales para VantaDB"
tags: [research, vector, chroma]
---

# Informe estático — chroma-core/chroma

> **Identidad congelada:** `chroma-core/chroma`, clon `C:\Users\Eros\chroma`,
> SHA `1a3ac2c7c623adbd03ed2508dc27b98cdf26016d`; captura de la adición:
> 2026-11-07, [manifiesto](clone-additions-manifest.json). Apache-2.0 en
> `LICENSE`. Excepción expresa al filtro original de estrellas.
> **Referencia comparativa:** VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`,
> no los commits históricos ni el worktree concurrente.
> **Método:** inspección estática y lectura pública; ninguna ejecución.
> Los nombres Rust de las fichas son propuestas, no tipos existentes.

## Índice

1. [Arquitectura y mapa](#1-arquitectura-y-mapa)
2. [Recomendaciones](#2-recomendaciones)
3. [Fichas candidatas](#3-fichas-candidatas)
4. [Incidencias y riesgos](#4-incidencias-y-riesgos)
5. [Prerrequisitos Vanta](#5-prerrequisitos-vanta)
6. [Que no trasladar](#6-que-no-trasladar)
7. [Advertencias metodológicas](#7-advertencias-metodológicas)
8. [Benchmarks](#8-benchmarks)
9. [Matriz](#9-matriz)
10. [Negativos Vanta](#10-negativos-vanta) · [Escritorio/UI](#11-escritorio--ui)
11. [Fuentes y cierre de autoría](#12-fuentes-cobertura-y-cierre-de-autoría)

## 1. Arquitectura y mapa

### 1.1 Correcciones respecto a la ficha anterior

La convivencia Python/Rust es un hecho, pero «core Rust en migración» no basta
para describir el default: `chromadb/config.py:120` selecciona
`chromadb.api.rust.RustBindingsAPI`. `PersistentClient` configura persistencia;
`RustClient` selecciona explícitamente bindings; `HttpClient` usa el transporte
Python `FastAPI`, que no determina por sí solo el backend del servidor.
**No todo Rust escribe en wal3**: el binding local configura `LogConfig::Sqlite`
en `rust/python_bindings/src/bindings.rs:446-518`.
Tampoco hay evidencia de HNSW local «sobre parquet»: el camino leído usa
archivos nativos HNSW, mapa pickle y SQLite. Arrow/blockfiles corresponden a
otros segmentos. Finalmente, el filtro distribuido leído no es simplemente
post-ANN: prepara máscaras antes de buscar. Se corrigen esas generalizaciones.

### 1.2 Stack, modelos y fronteras

| Superficie | Implementación observada | Frontera y almacenamiento |
|---|---|---|
| Python local | `chromadb/__init__.py:201-260`; `config.py:120`; bindings PyO3 | Proceso llamador → Rust; directorio persistente o memoria efímera |
| Python HTTP/Cloud | `__init__.py:262-315,375-433` | Red, headers/TLS/tenant/database; Cloud es servicio separado, no motor local |
| Servidor Rust | `rust/frontend/src/lib.rs:47-125`, `server.rs`, `impls/service_based_frontend.rs` | Axum/Tokio; executor local o distribuido y sysdb/log configurables |
| Segmentos locales | `rust/segment/src/local_hnsw.rs`, `sqlite_metadata.rs` | SQLite para metadata/log; HNSW + `index_metadata.pickle` para vectores |
| Workers distribuidos | `rust/worker/src/execution/{operators,orchestration}` | Record/metadata blockfiles, HNSW/SPANN; lectura de log más proyección compactada |
| Log | `rust/log/src/lib.rs:24-45`, `sqlite_log.rs`; `rust/wal3/README.md:1-35` | SQLite local frente a servicio gRPC; wal3 es log sobre objetos, no WAL universal |
| Sysdb/storage | `rust/sysdb`, `rust/storage`, `rust/blockstore` | Catálogo/ubicación de segmentos y objetos; no es la misma transacción que el log |
| SDK e inspección | `chromadb/api`, `clients`, `rust/chroma`, `rust/cli` | CRUD/query/search; TUI Ratatui real, además de CLI de integridad |

El workspace (`Cargo.toml:1-90`) declara Tokio, sqlx, Axum, tonic, Arrow,
Tantivy y hnswlib desde un fork con revisión declarada. No se inspeccionaron
las licencias transitivas de ese fork ni de cada modelo; Apache-2.0 del
repositorio no las sustituye. Python, bindings y crates propios se estudian
bajo la licencia raíz; servicios Cloud y pesos/modelos necesitan análisis propio.
Reescribir una idea en Rust no elimina obligaciones de una obra derivada.

La jerarquía es tenant → database → collection UUID → segmentos/shards.
Un registro tiene ID de usuario, embedding, documento, metadata y operación;
el log añade posición y el segmento asigna offset ID interno. No intercambiar
posición del log, offset del índice y versión de colección. `OperationRecord`
es una mutación de datos, no un checkpoint de ejecución de agente.
Los vectores pueden venir del usuario. El proveedor ONNX de Python declara
`all-MiniLM-L6-v2` y caché/descarga en
`chromadb/utils/embedding_functions/onnx_mini_lm_l6_v2.py:34-44`;
no significa que todos los clientes usen ese modelo. No hay LLM obligatorio
para los flujos de almacenamiento estudiados.

```text
Python Collection / cliente Rust / HTTP
    → validación tenant + collection + dimensiones + operación
    → Frontend → Log seleccionado
        local: SQLite commit → backfill/compactador local → purge
        distribuido: servicio de log → materialización → writers
                     → commit/flush → registro de generación en sysdb
    consulta → colección/segmentos + log pendiente
             → Filter → KnnLog || KnnHnsw → merge → proyección
    diagnóstico offline → SQLite + archivos + mapa ID + watermark
```

### 1.3 Tres recorridos trazados

**Ingesta.** `Frontend::add`, `service_based_frontend.rs:2179-2278`, valida
database/dimensión, construye `Operation::Add` y reintenta únicamente
`PushLogsError::Backoff`. `retryable_push_logs:1863-1899` valida primero el
índice persistente si el executor es local; después delega a `log_client`.
Esto evita añadir más log sobre un índice ya ilegible en ese camino, no
acredita una transacción log+índice. En `SqliteLog::push_logs`,
`sqlite_log.rs:330-414`, serializa registros, abre una transacción, inserta
batches y hace `tx.commit()` **antes** de pedir backfill y purge.
Un fallo del backfill puede llegar al caller después de persistir el log:
error/timeout no equivale a «nada escrito». La cancelación antes del commit y
después del commit requiere estados diferentes; no se siguieron todas las
garantías de drop de los futures del transporte.

**Consulta distribuida HNSW.** `Frontend::query:3031-3115` valida dimensiones,
obtiene colección/segmentos para tenant y comprueba la indexación del predicado
si `enable_schema`; construye `Scan`, `Filter` y `KnnBatch`. La cota de log
depende de `enable_log_scouting`, no es una promesa universal de snapshot.
`KnnFilterOrchestrator`, `knn_filter.rs:157-190`, prepara log/filtro/reader.
`Filter::run`, `filter.rs:715-875`, materializa cambios y produce máscaras
distintas para log y compactado; excluye del compactado IDs tocados por log.
`knn_hnsw.rs:38-70` pasa allow/disallow al índice; `knn_log.rs:61-128`
materializa y omite deletes antes de distancias. `KnnOrchestrator`,
`knn.rs:166-235`, lanza y mezcla ambas ramas usando token de cancelación.
Los errores de fetch, filtro, reader y materialización conservan categorías
(`knn_filter.rs:47-143`); vacío no sustituye automáticamente error.
No se certificó cómo cada cliente propaga cancelación hasta operaciones nativas.

**Update/delete y derivados.** `update:2280-2383` y `upsert:2385-2488`
construyen operaciones diferentes; no comparten semántica de creación.
`retryable_delete:2492-2699` resuelve un filtro en IDs o toma IDs explícitos;
publica `Operation::Delete` y devuelve cantidad preparada. Selección y append
son fases separadas: no inferir delete serializable por predicado.
`delete:2701-2745` invalida caché ante NotFound y reintenta Backoff.
`materialize_logs`, `segment/src/types.rs:764-1020`, reduce operaciones
ordenadas: add repetido se omite, delete de nuevo registro lo retira del
batch, delete existente produce `DeleteExisting`, update ausente/deleted se
omite, add/upsert tras delete puede sobrescribir. No son reglas de dedup
semántico entre recuerdos. La consulta debe ocultar el viejo compactado antes
de que la compactación lo elimine físicamente. En local, `persist`,
`local_hnsw.rs:1353-1417`, sincroniza archivos nativos antes de reemplazar
mapa/checkpoint; `sync_dir` documenta excepción Windows. En distribuido,
`compact.rs:710-728,919-940,951-1096` separa aplicar/flush y registrar.
No se equipara delete lógico con borrado seguro de objetos, backups o Cloud.

## 2. Recomendaciones

1. **P0 contractual — A:** separar aceptación durable, aplicación y checkpoint
   persistente. La evidencia más útil es SQLite commit antes de backfill, no
   el número «3» de wal3; Vanta ya tiene formatos WAL y `SyncMode`.
2. **P0 integridad — E:** extender auditoría/rebuild existentes con relación
   entre archivos, mapa de IDs y watermark; diagnóstico read-only antes de reparar.
3. **P1 corrección de derivados — B/D:** explicitar replay de delete/update,
   publicación y retención de generaciones sin resucitar datos antiguos.
4. **P1 planner — C:** exponer el dominio efectivo de filtros y revisiones;
   no añadir otro motor de filtros ni asumir post-filter en Chroma.
5. **P2 compatibilidad — F:** declarar diferencias de backend/transporte y
   ownership de índice vivo. No imponer la infraestructura distribuida al modo embebido.

## 3. Fichas candidatas

Todas las fuentes C1–C12 del §12 fijan el mismo SHA completo de Chroma.
Los seams V1–V6 fijan el snapshot Vanta, no el código mutable de la sesión.
Las firmas siguientes son contratos conceptuales **no implementados**; nombres
de revisión/token deben reutilizar tipos existentes si ya expresan el invariante.

### A — Recibo de mutación con dominios de durabilidad y aplicación

- **Problema agente:** tras guardar una corrección, el agente recibe error y la
  vuelve a insertar; no sabe si falló antes o después del commit.
- **Upstream mínimo:** C2, `SqliteLog::push_logs:330-414`: transacción de
  embeddings_queue, commit y después backfill/purge. C3 distingue
  `applied_seq_id` de checkpoint. wal3 C8 aporta offsets/fragments/cursors;
  su nombre no prueba soporte de migración «v2→v3».
- **Delta/seam:** V1 ya tiene append/batch/sync, formatos y política de sync;
  V2 ya persiste derivados. Extender el recibo del camino SDK de escritura y
  health existente, no crear otro WAL ni usar el checkpoint de tareas V6.
- **Alternativas:** documentar solo (menor coste, no consultable); extender
  resultado SDK (preferida si el inventario no basta); ledger paralelo
  (rechazado por doble verdad). Adaptador Chroma no es requisito.
- **Interfaz propuesta:** `mutation_status(namespace, mutation_id, revision)
  -> Result<MutationStatus, StatusError>`, con dominios `Accepted`,
  `PrimaryDurable`, `Applied`, `Checkpointed`, `Unknown`. Input incluye
  backend/instancia; output incluye watermark observado y capacidades.
  Invariantes: aplicado no implica fsync; WAL position no es revisión MVCC;
  `Unsupported`, I/O y generación obsoleta no son «pendiente» ni éxito.
- **Pasos:** inventariar retornos existentes; marcar punto de commit por
  backend; enlazar aplicación de derivados; propagar estado a SDK y diagnóstico.
  Ninguna espera mantiene locks que necesite el productor del estado.
- **Persistencia/migración/rollback:** empezar como proyección diagnóstica
  del estado real; si se necesita conservar ID, metadato versionado V2 en el
  mismo commit que cubre. Registros anteriores = desconocido, no durable
  inventado. Desactivar la exposición no altera formato WAL; no bajar
  watermarks ni borrar log para hacer rollback de una respuesta.
- **Dependencias:** identidad de mutación, definición de sync efectiva,
  recuperación y publicación D. B aporta evidencia de aplicación.
- **FMEA:** cancelación tras commit → retry duplicado; detectar fase/ID y
  devolver indeterminado recuperable. Fallo de índice tras append → falso
  rechazo; detectar diferencia durable/applied y conservar log. Reopen con
  ID de otra instancia → falsa observación; scope por instancia+namespace.
- **Aceptación futura:** favorable: escritura y reopen conservan el mismo
  estado demostrado; adversos: error poscommit, cancelación y backend sin
  persistencia no mienten. Observar posiciones/fases, no documentos.
- **No-goals:** garantía distribuida, formato wal3, migración de una versión
  de WAL que no se ha leído, exactly-once universal o nuevo scheduler.

### B — Materialización ordenada y delete que invalida la versión compactada

- **Problema agente:** un dato borrado/corregido sigue en HNSW antiguo o
  reaparece después de compactar/reiniciar.
- **Upstream mínimo:** C4 `materialize_logs:764-1020`, estados y merges por
  ID; C5 `Filter::run:715-875` excluye del compactado IDs modificados por log;
  C6 `Knn::run` omite `DeleteExisting`. Es un contrato de replay, no dedup LLM.
- **Delta/seam:** V2 ya tiene `Tombstones`, `Versions`, índices payload/text/
  sparse; V5 ya tiene dedup/supersession. Revisar actualización coordinada de
  derivados y sus auditorías, no introducir un lifecycle competidor.
- **Alternativas:** actualizar índices síncronamente (menor lag, mayor coste
  del write); overlay de cambios (solo si el backend ya lo necesita);
  reconstrucción completa (operación existente, no hot path). No imponer la
  semántica «update ausente = ignorar» a la API Vanta.
- **Interfaz propuesta:** `apply_derived_change(change: VersionedChange,
  expected: DerivedRevision) -> Result<AppliedDelta, ApplyError>`.
  Identidad `(namespace,key,source_revision,operation_id)`; salida expresa
  ramas aplicadas y tombstone. Errores `Stale`, `MissingSource`, `Corrupt`,
  `Unsupported`; ausencia voluntaria no se confunde con fallo de lectura.
  Solo la revisión vigente puede producir un derivado consultable.
- **Pasos:** trazar insert/update/delete de cada backend; formalizar casos
  add→delete, delete→upsert y update de ausente según contrato propio;
  comprobar máscara de tombstones y aplicar delta con revisión.
- **Persistencia/migración/rollback:** aprovechar tombstones/versiones V2;
  derivado antiguo sin revisión debe auditarse/reconstruirse, no asumirse
  fresco. Mantener generación anterior mientras un lector la referencia;
  rollback de índice no puede retroceder tombstones de la fuente.
- **Dependencias:** A identifica mutación; D publica generación; E verifica
  cobertura. Mutaciones de un mismo ID requieren orden explícito y recursos
  compartidos con ownership claro.
- **FMEA:** mezcla revisión nueva/embedding viejo → recuerdo incorrecto;
  detectar revisión de cada rama y bloquear/publicar coherentemente.
  Cancelación entre texto y vector → parcialidad; detectar deltas incompletos,
  reanudar idempotentemente. Purge temprano → recovery imposible; exigir
  checkpoint de consumidores, no solo «tarea acabó».
- **Aceptación futura:** normal: update cambia todas las ramas prometidas;
  adversos: delete durante query/rebuild, replay duplicado y reapertura no
  resucitan la revisión eliminada. Contadores de deltas, sin IDs sensibles.
- **No-goals:** borrado criptográfico, GC de backups remotos, cambio de
  política de memoria o copia de estados/algoritmo upstream al producto.

### C — Plan de filtros y cobertura de log/índice declarado

- **Problema agente:** un scope restrictivo reduce resultados y el cliente
  cree que no hay memoria cuando el filtro se aplicó después de truncar ANN.
- **Upstream mínimo:** C5 devuelve máscaras separadas y C6 las pasa como
  allow/disallow a HNSW; C1 `query:3031-3115` puede rechazar un predicado sin
  índice habilitado. No demuestra autorización por el mero tenant/filter.
- **Delta/seam:** V3 `vector_memory_search` ya selecciona prefilter/scan/
  máscara y V4 fusiona lexical+dense+sparse. Exponer razón/cobertura efectiva,
  no rehacer filtros, HNSW o RRF.
- **Alternativas:** logging interno (no sirve a SDK); reporte opcional de
  plan (mínimo); gramática nueva (rechazada). El adapter conserva significado,
  no traduce `$contains` arbitrariamente a una operación no equivalente.
- **Interfaz propuesta:** `explain_filter(namespace, query, filters, revision)
  -> Result<FilterPlanReport, PlanError>`; salida por rama: `PreIndex`,
  `ExactScan`, `PostCandidate` o `Unsupported`, revisión y límite aplicado.
  Invariante: explicar lo ejecutado, no una estimación presentada como hecho.
  Un filtro de seguridad no puede degradarse a opcional.
- **Pasos:** reutilizar decisión del planner; adjuntar reporte opt-in;
  distinguir cardinalidad candidata/devuelta sin exponer payload.
- **Persistencia/migración/rollback:** reporte efímero y versionado en API;
  no migrar datos ni índices. Desactivarlo mantiene resultados/errores; si
  hay rebuild requerido, informar por mecanismo D/E, no lanzarlo al consultar.
- **Dependencias:** revisión B/D, capabilities F y autorización existente.
  Cancelar la consulta cancela también el reporte; no lanzar segunda query
  con datos o revisión potencialmente distintos.
- **FMEA:** reporte stale → falsa explicación; detectar generación del plan.
  filtro incompatible omitido → filtración; error tipado. Conteos globales
  entre tenants → canal lateral; limitar reporte al scope autorizado.
- **Aceptación futura:** filtros vacíos/selectivos y deletes pendientes
  concuerdan con plan; caso adverso unsupported produce error explícito.
  Paridad de resultados con reporte desactivado, sin nuevo scoring.
- **No-goals:** ACL nueva, parámetros de ranking, gramática Chroma completa
  ni promesa de recall basada en el nombre «prefilter».

### D — Compactación preparada, publicación y retención separadas

- **Problema agente:** una tarea de compactación figura completada, pero
  lectores observan vector nuevo y metadata vieja o pierden un archivo activo.
- **Upstream mínimo:** C7 `run_regular_compaction_workflow:710-728`,
  `run_register:919-940`, `run_compaction:951-1096`; C7b
  `CollectionFlushInfo::try_from:59-77` enlaza offset, versión y archivos.
  Aplicar/flush no es publicar catálogo. Los tokens de tarea se propagan;
  no se acreditó atomicidad de todo un workflow multicolección.
- **Delta/seam:** publicador real localizado en el pin Vanta:
  `src/sdk/api/admin.rs::Embedded::rebuild_index:19-32` coordina vector,
  derivados, texto, sparse y scalar **secuencialmente**, propagando `?` en
  cada fase; `src/sdk/serialization/impl_rebuild.rs:16-58,155-180` escribe
  el batch de derivados y después estado/health. Esa operación existente es
  el seam de publicación a extender por fases — no crear otro publicador.
  `V2` particiones/`InternalMetadata` solo enumera almacenamiento (sin CAS,
  pinning ni swap). Manifiesto versionado, CAS y lease de lectores quedan
  como **alternativas condicionadas**, no como mínimo implícito. Esta
  lectura no prueba por sí sola carrera ni atomicidad insuficiente: faltan
  locks del caller e índices concretos. Mapear publicación real antes de
  añadir envoltorios. Compactar contexto de agente y compactar índice son procesos
  diferentes aunque ambos se llamen compaction.
- **Alternativas:** rebuild offline existente (mínimo operacional); swap de
  generación preparado (si concurrencia lo exige); puerto distribuido de
  workers/sysdb (rechazado). Extender lifecycle existente es preferible.
- **Interfaz propuesta:** `prepare_derived_revision(source_revision, cancel)
  -> Result<PreparedDerived, PrepareError>` y
  `publish(prepared, expected_current) -> Result<PublishedRevision, PublishError>`.
  `PreparedDerived` posee temporales y no es clonable como permiso de
  publicación; errores conflict/cancel/I/O no avanzan el catálogo.
- **Pasos:** inventariar locks y manifiesto vigentes; mapear cambios sobre las fases de `rebuild_index` (vector, derivados, texto, sparse, scalar) y los publicadores de cada rama, con sus estados y locks; preparar fuera del
  lector; validar todas las ramas y fuente; publicar con comparación de
  revisión; retirar generación vieja solo después de liberar lectores (cobertura desconocida documentada, no escondida).
- **Persistencia/migración/rollback:** manifiesto/generación versionados solo
  si Vanta no tiene equivalente; conservar anterior y limpieza reanudable.
  Migrar reconstruyendo desde fuente verificable, nunca inventando offsets.
  Revertir puntero requiere validar deletes actuales; no revertir el primario.
- **Dependencias:** A/B; E inspecciona el preparado; F evita doble writer.
  Usar scheduler actual para límites, sin nuevas colas ni tuning propuesto.
- **FMEA:** crash tras flush antes del publish → huérfano; detectar manifiesto
  no referenciado y recolectar después. Cancelación durante publish → estado
  indeterminado; consultar generación durable. Recolectar con lector vivo →
  fallo; referencias/pinning y GC diferido. No guardar texto en trazas.
- **Aceptación futura:** favorable: lectores viejos/nuevos tienen generaciones
  válidas; adversos: CAS fallido, disco lleno y cancelación no publican
  preparado incompleto. Reopen identifica huérfanos sin borrarlos ciegamente.
- **No-goals:** servicio sysdb, formato blockfile, refactor del scheduler o
  garantía de transacción entre todos los objetos remotos.

### E — Auditoría relacional y reparación offline sobre copia

- **Problema agente:** la colección contiene fuente pero un checkpoint
  inconsistente deja recuerdos inaccesibles; una reparación automática puede
  destruir evidencia de recovery.
- **Upstream mínimo:** C9 `HnswIntegrityCheckArgs:25-63`, `Report:83-125`,
  códigos de salida `138-162`: distingue findings de check fallido.
  C10 `repair:64-124` trabaja en staging/copia, rechaza salida existente y
  enlaces/archivos especiales. C3 `persist:1353-1417` ordena sync, mapa y
  directorio; no afirmar misma garantía de directorio en Windows.
  **Dominio exacto leído** (`hnsw_config_repair.rs::repair_copy:146-329`,
  especialmente 204-207,241-260,264-314): repara **únicamente
  `ef_construction` fuera de rango**; no reconstruye edges ni embeddings.
  Precondiciones: un único HNSW local persistido, mapa y dimensión válidos.
  Rechazos explícitos: SQLite adelantado al checkpoint, formato distinto de
  v1, recuperación pendiente y pérdida parcial del índice. Si M excede el
  límite exige **rebuild**, no lo repara. C9b
  `hnsw_integrity_check.rs::inspect_segment:493-660` e
  `inspect_hnsw_metadata:662+`: stub vacío sin watermark no equivale a
  checkpoint corrupto; fast-forward no certifica integridad del log
  retenido.
- **Delta/seam:** V6 `cmd_rebuild_index` y V3 `audit_text_index` ya existen;
  ampliar relación fuente↔vector↔ID↔watermark y la separación check/repair,
  no crear una CLI paralela. No se ha probado que falten auditorías ANN.
- **Alternativas:** verificar conteos (barato, insuficiente para revisiones);
  ampliar deep audit existente (preferida); reparación in-place online
  (rechazada salvo contrato de exclusión propio futuro).
- **Interfaz propuesta:** `audit_generation(scope, revision)
  -> Result<GenerationAudit, AuditError>`; `plan_repair(audit, expected_revision)
  -> Result<RepairPlan, RepairError>`. `apply_offline_copy(plan, target)`
  requiere destino nuevo y autorización explícita; salida es artefacto
  verificado, no promoción automática. `CannotRead` nunca es `Clean`.
- **Pasos:** inventariar audit/rebuild; capturar metadata sin payload; validar
  formatos y revisiones; emitir plan; reparar copia; verificar y dejar al
  operador la decisión de promoción.
- **Persistencia/migración/rollback:** fuente intacta y copia completa con
  sidecars del backend; formato nuevo explícito, no mutación oportunista al
  consultar. Rollback = conservar origen y retirar copia, no borrar archivos
  de la instancia viva. Reindexar desde fuente solo si está disponible.
- **Dependencias:** A/D para watermarks y publicación; rutas seguras y
  exclusión de escritores. En cada fase cancelación deja original intacto.
- **FMEA:** snapshot online incoherente → falso corrupto; detectar revisión
  cambiante y abortar. Symlink hacia otro tenant → copia/borrado fuera de
  scope; resolución confinada y rechazo. Mapa leído no coincide con watermark
  → fast-forward destructivo; detectar mismatch y exigir replay verificable.
- **Aceptación futura:** check limpio/findings/error separados; copy repair
  repetido no sobrescribe; disco lleno/cancelación no toca origen.
  Adversos del dominio leído: `ef_construction` inválido se repara en copia;
  SQLite adelantado, formato !=v1, recovery pendiente o pérdida parcial del
  índice son rechazo explícito; M fuera de límite devuelve rebuild
  requerido; stub vacío sin watermark no se reporta como corrupción;
  fast-forward no se acepta como prueba del log retenido. Pruebas
  manuales futuras en cada SO soportado; reportes sin documentos por defecto.
- **No-goals:** ejecutar aquí herramientas, reparación universal, copiar
  pickle upstream ni afirmar seguridad de crash sin interrupción real.

### F — Contrato de backend y ownership de índice vivo

- **Problema agente:** misma firma Python funciona local y falla en servidor,
  o expulsión de caché crea dos escritores del mismo índice nativo.
- **Upstream mínimo:** C11 clientes/default; C2 rechaza sparse local;
  C12 `LocalSegmentManager:40-47,173-258` serializa misses y conserva identidad
  débil; C3 `ensure_usable:777-788` invalida handles fallidos/deleted.
  Evicción no equivale a eliminación lógica; cerrar FDs no elimina el índice.
- **Delta/seam:** bindings oficiales Vanta y `StorageBackend` V2 ya declaran
  capabilities opcionales. Alinear transporte, errores y ownership con esas
  capacidades; no asumir que un runtime Rust homogéneo elimina diferencias.
- **Alternativas:** documentación por SDK (necesaria, insuficiente si diverge);
  adapter de capabilities existentes (preferida); nueva caché global (rechazada
  mientras el inventario no demuestre necesidad).
- **Interfaz propuesta:** `describe_backend(handle) -> BackendContract`
  con persistencia, filtros soportados, sync y formato; opcional
  `acquire_index(index_id, expected_revision) -> Result<IndexLease, LeaseError>`.
  Lease RAII posee referencia a la identidad nativa; deleted/failed impiden
  mutación y un miss concurrente no fabrica otro writer.
- **Pasos:** revisar capacidades por binding/backend; propagar rechazo sin
  fallback silencioso; verificar identidad de handles bajo misses; adaptar
  Solo donde el ownership actual no exprese la garantía.
- **Persistencia/migración/rollback:** capabilities versionadas sin cambiar
  registros; lease solo en memoria. Formatos previos pasan por auditoría E
  antes de ser marcados usables; rollback de adapter preserva archivos y no
  libera recursos aún referenciados.
- **Dependencias:** A para semántica de errores, D/E para ciclo de vida.
  La cancelación de acquire libera espera; drop de un cliente no puede cerrar
  unilateralmente un recurso que otro lease utiliza.
- **FMEA:** caché expulsa pero query aún usa handle → doble writer; detectar
  identidad/número de propietarios, compartir lease. Backend no soporta
  predicado → resultado vacío engañoso; error Unsupported. Transporte corta
  respuesta tras commit → no retry ciego; consultar A.
- **Aceptación futura:** dos misses/reload concurrentes reutilizan identidad;
  adversos: eviction/delete/error nativo invalidan o conservan handles según
  revisión sin data race. Capabilities iguales entre SDK y servidor efectivo.
- **No-goals:** portar caché Chroma, añadir un SDK nuevo, prometer compatibilidad
  binaria Python/Rust ni certificar Cloud con evidencia del core local.

## 4. Incidencias y riesgos

### Incidencia pública efectivamente leída

[#7946, «Nothing found on disk after in-process LRU cache eviction»](https://github.com/chroma-core/chroma/issues/7946):
[cuerpo API](https://api.github.com/repos/chroma-core/chroma/issues/7946)
leído íntegramente; estado **open**, creado `2026-10-22T21:16:08Z`,
actualizado `2026-11-07T00:22:43Z` según esa respuesta. No se identifica
un commit completo afectado: el autor declara chromadb **1.5.9**, Python 3.11,
Linux x86_64 emulado con QEMU. Fallo tras expulsión, archivos nativos presentes
en observaciones previas; no se midió todo el estado al instante del error.
Una pausa con tres éxitos y una repetición de 50 éxitos no prueban corrección
ni permiten inferir una tasa estable de fallo.

Se leyeron también los
[comentarios](https://api.github.com/repos/chroma-core/chroma/issues/7946/comments).
El [comentario de 2026-10-31](https://github.com/chroma-core/chroma/issues/7946#issuecomment-6540462503)
reporta reproducción determinista en 1.5.9 con más colecciones que caché,
índices pequeños sin pickle/checkpoint persistido y límite de FDs bajo.
El [de 2026-11-07](https://github.com/chroma-core/chroma/issues/7946#issuecomment-6699506165)
reporta colisiones de shards de caché y diferencias con reset en 1.5.5.
Son informes de terceros, algunos explícitamente asistidos por IA; ni su
interpretación causal ni sus mediciones fueron reproducidas aquí.
Comentarios anteriores distinguen mapa ausente/vacío de archivos nativos
presentes y documentan reproducciones negativas. No se leyó una PR merged
que permita declarar arreglada la incidencia.

**Relación con el pin:** C12 conserva identidad débil de índice vivo; C3
separa replay/applied/checkpoint y comprueba fingerprints. Eso hace incorrecto
trasladar automáticamente el defecto de 1.5.9 al SHA estudiado. Validación
discriminante futura: expulsión/reapertura de índice pequeño frente a
checkpoint completo, con y sin reset, en entorno nativo y emulado; observar
mapa/watermark/identidad sin publicar IDs ni contenido. Aplicabilidad Vanta:
auditar ownership y checkpoint equivalentes (E/F), no elevar RLIMIT o bajar
umbrales como parche trasladado.

### Seguridad y errores por frontera

- Cliente de red → frontend: tenant/database y filtro no sustituyen
  autenticación/autorización. No se auditó el despliegue Cloud ni todos los
  middlewares, TLS o defaults de autenticación.
- Documentos/URI/modelo → proceso de embedding: contenido no confiable y
  descargas deben pasar política propia; no se acreditó SSRF en un camino
  concreto ni se ejecutaron fetches del producto.
- Operador → CLI de reparación: acceso a directorios es autoridad sobre datos;
  C10 rechaza symlinks/especiales y destino existente en el camino leído.
  Eso no certifica ausencia de toda carrera filesystem.
- Delete → derivados/log/backup: ocultación lógica, purga y reclamación física
  son garantías distintas. Ningún modelo debe autorizar el borrado.

## 5. Prerrequisitos Vanta

| Candidato | Prerrequisito concreto | No bloquea |
|---|---|---|
| A | Identificar recibo, SyncMode y punto de commit por backend V1/V2 | No requiere cambiar planner ni LLM |
| B | Revisión/tombstone propio y orden de deltas V2/V5 | No requiere nuevas políticas de dedup |
| C | Reutilizar decisión efectiva V3 y autorización existente | No requiere compactador nuevo |
| D | Inventario de ownership/publicación y scheduler V6 | No requiere copiar sysdb/objetos |
| E | Auditorías existentes, copia consistente y formatos soportados | No requiere ingesta asíncrona |
| F | Capabilities por backend/binding y ciclo de vida de handles | No requiere homogeneizar Cloud/local |

Los contratos transversales de durabilidad, integridad y embedding citados
por la ficha previa siguen siendo antecedentes útiles, no prerrequisitos
universales con estado acreditado. La selección de embedding debe seguir
opcional y declarar dimensión/modelo si se persisten vectores nuevos.

## 6. Que no trasladar

No trasladar el core Python, wal3 sobre objetos, control plane de workers,
arquitectura dual ni Cloud como runtime obligatorio. El coste oculto de wal3
es coordinación mediante operaciones condicionales de objetos y retención de
cursors/fragments; no es un sustituto directo del WAL local de Vanta.
No bajar `sync_threshold` para tapar un problema de ownership, ni confundir
`cleanup` con borrado seguro. No copiar silencios de add/update ausente sin
contrastar semántica SDK propia. No hacer obligatorio el embedder ONNX.
No proponer filtros/HNSW/BM25/RRF, cuotas, scheduler, sesiones, provenance o
checkpoints como si Vanta no los tuviera.

## 7. Advertencias metodológicas

La ampliación sí leyó los rangos C1–C12: sustituye la cobertura solo por árbol
de la ficha anterior sin reclamar exhaustividad. Los caminos Python legacy,
Rust local, worker distribuido y Cloud no se certifican entre sí.
`ReadLevel::IndexAndWal` en un microbenchmark con log vacío no mide frescura
postcommit. Que el código llame fsync no es una prueba de pérdida de energía.
Un hash/fingerprint de archivos tampoco prueba consistencia con cada registro.
Clon superficial, sin fetch/pull; no se recalcularon estrellas ni metadata.
No se instalaron dependencias, ejecutaron procesos ni modificaron evaluadores.

## 8. Benchmarks

### Runner realmente leído: consulta HNSW de workers

C13 `rust/worker/benches/query.rs:1-269` usa Criterion; llama
`bench_query → bench_run → bench_routine`, y `knn_filter → KnnOrchestrator`.
C14 `rust/benchmark/src/benchmark.rs:6-27` usa runtime Tokio multithread y
`iter_batched(..., BatchSize::SmallInput)`: setup se separa de rutina medida.
`load.rs:12-62` construye segmentos SIFT1M por lotes de 10 000, compactando
datos antes de medir; `empty_fetch_log` entrega un log in-memory sin registros.

- **Datos/unidad:** C15 `datasets/sift.rs:20-117,148-169` descarga/cachea
  base `.fvecs`, queries `.fvecs` y ground truth `.ivecs`; declara 1 000 000
  vectores, 10 000 queries, dimensión 128, k=100. URLs usan `resolve/main`,
  no hash de dataset acreditado. No hay sesiones, extracción, answerer ni juez.
- **Población efectiva:** `query.rs:144-237` baraja con `thread_rng()` sin
  semilla declarada y toma **cuatro queries** por rutina; tres variantes de
  filtro trivial/siempre verdadero/siempre falso. No mide las 10 000 por
  iteración ni representa filtros selectivos reales.
- **Métrica/denominador:** `bench_routine:122-141` ejecuta filtro y batch de
  consultas con `.buffered(32)`. Conserva `_expected` y `_results` sin
  comprobarlos y deja `TODO: verify recall`. La duración corresponde a
  rutina/batch Criterion, no a recall@100 ni a tiempo por pregunta LLM.
  La concurrencia máxima declarada es 32, pero el batch de cuatro la limita.
  No hay denominador de calidad calculado ni tratamiento de abstenciones.
- **Fallos/defaults:** `expect` aborta ante fallo; no emite registro por query
  fallida ni una tasa de éxito comparable a un runner de memoria. No se leyó
  configuración externa de Criterion: no se inventan warmup, muestras o
  percentiles. Setup carga segmentos/dispatcher; cachés pueden quedar calientes.
- **Efectos/costes:** red para dataset, caché local, archivos de segmentos,
  CPU/RAM e hilos Tokio. Sin llamadas pagadas de LLM en el camino leído.
  La limpieza completa de temporales/caché no se siguió; no ejecutar en datos
  compartidos. Arrancar el binario de benchmark está fuera de esta tarea.
- **Adaptación concreta, no diseño:** el adapter Vanta tendría que mapear
  vector/dimensión/métrica, IDs ground-truth, filtro y k hacia búsqueda V3;
  devolver IDs y errores sin cambiar el protocolo ni atribuir al runner un
  recall que no calcula. La comparación de ANN sigue
  [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), y
  [BENCHMARK-NOTES](BENCHMARK-NOTES.md) impide confundirla con QA/memoria.
  La parte de orquestadores/segmentos no es portable al SDK embebido.

Se localizaron además benches de filtro/get/limit/SPANN, blockstore y distancia;
no se auditaron sus métricas. No se deduce ausencia global de benchmarks de
calidad. No hay resultados propios ni validación de claims de rendimiento.

## 9. Matriz

Estado de **VantaDB en el snapshot fijado**, no tabla de capacidades Chroma.

| Capacidad | VantaDB | Evidencia y decisión incremental |
|---|---|---|
| WAL versionado/SyncMode | Existe | V1; A expone fases, no crea formato WAL |
| HNSW, BM25, sparse y RRF | Existe | V3/V4/V2; Chroma no justifica reimplementarlos |
| Planner/prefilter/máscara | Existe | V3; C solo explica dominio efectivo |
| Índices payload/text/sparse y tombstones | Existe | V2; B contrasta revisión/invalidación |
| Auditoría/rebuild | Existe | V3/V6; E amplía relación entre artefactos si falta |
| Recibo durable/applied/checkpoint por backend | No acreditado completo | V1 no basta para probar API de fases; A condicionado al inventario |
| Publicación/retención de generación coherente | Parcial en esta lectura | V2/V6 prueban superficies, no todo lifecycle; D no afirma bug |
| Cuotas y budgets | Existe | Referencia Vanta, config y recall; no confundir bytes/tokens |
| Sesiones, provenance, dedup/supersession | Existe | V5 y referencia; Chroma CRUD no las reemplaza |
| Scheduler/checkpoints de tareas | Existe | V6/referencia; no equivalen a watermark de índice |
| Studio | Existe en otro worktree | Referencia Vanta; extender diagnóstico, no inventar desktop |
| Coordinación de objetos wal3 | No aplicable al mínimo local | C8; no adoptar runtime distribuido |

## 10. Negativos Vanta

**Hechos propios:** V3 ya contiene decisión prefilter/scan/máscara;
V1 ya tiene append con política de sync y `sync()` separado; V4 fusiona ramas
existentes. Se descartan los falsos negativos «sin filtros», «sin WAL
versionado», «sin auditoría» y «sin UI».
**Hipótesis, no defectos demostrados:** recibo incompleto frente a una
proyección fallida, watermark que no cubra todas las ramas y reclamación de
generación referenciada. Requieren seguir callers y validación adversa antes
de abrir tareas. La coincidencia con #7946 no demuestra que Vanta comparta
su caché, formato pickle o disparador por RLIMIT.
**Negativo sustentado del evaluador propio, separado:** las notas fijadas
revalidan que `evals/eval_locomo.py::main:811-868` permite sintético/fallback
a fake fuera de self-test. No es prueba contra la durabilidad del motor ni
invalida todas las mediciones históricas; impide usar esa salida sin más como
certificación equivalente de memoria o ANN.

## 11. Escritorio / UI

La frase anterior «sin UI propia» era demasiado amplia. **Hay TUI real**:
C16 `CollectionBrowser::run:63-90` usa Ratatui/Crossterm, estados Main,
Search, SearchResult y Expand. `EventsHandler::load_records:100-151` pide
count y `collection.search` con documento/metadata; inicia lotes de 100 y
paginación por offset. `submit_search:144-190` construye filtros por IDs,
documento y metadata, lanza tarea Tokio y devuelve acciones por canal.
Error de count puede producir acción de error y count 0; eso no convierte
el dataset en vacío certificado. No se inspeccionó todo el manejo de tareas
al salir: `spawn` y canal unbounded no acreditan cancelación estructurada.

Es interfaz **terminal local**, no aplicación desktop nativa ni web.
`CloudClient` prueba una superficie SaaS, no que su consola web esté incluida
en este checkout. No se inspeccionaron todas las sample apps ni fuente de
consola externa; no inferir ausencia global. No se arrancó ninguna UI.
Para Studio existente, la oportunidad concreta es presentar fases A/D,
hallazgos E y dominio C con revisión, errores y controles de autorización;
no copiar una TUI ni exponer repair destructivo desde una pantalla de lectura.

## 12. Fuentes, cobertura y cierre de autoría

### Registro pin de fuentes leídas

Todos los enlaces C apuntan a
`1a3ac2c7c623adbd03ed2508dc27b98cdf26016d`. Los rangos expresan lectura
acotada; no implican auditoría del archivo completo.

| Ref. | Fuente y rango/símbolo utilizado |
|---|---|
| C1 | [service_based_frontend.rs](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/frontend/src/impls/service_based_frontend.rs#L1863-L1899), `retryable_push_logs`, add/update/upsert/delete/query en 2179-2745 y 3031-3115 |
| C2 | [sqlite_log.rs](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/log/src/sqlite_log.rs#L330-L414), `push_logs`; `purge_logs:505-527`; [lib.rs:24-45](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/log/src/lib.rs#L24-L45) selecciona backend |
| C3 | [local_hnsw.rs](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/segment/src/local_hnsw.rs#L715-L788), applied/checkpoint/replay/usable; `persist:1353-1417` |
| C4 | [types.rs:764-1020](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/segment/src/types.rs#L764-L1020), `materialize_logs`; estructuras borrowed/hydrated 254-536 |
| C5 | [filter.rs:715-875](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/operators/filter.rs#L715-L875), máscaras y materialización; `FilterOutput:39-65` |
| C6 | [knn_hnsw.rs:38-70](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/operators/knn_hnsw.rs#L38-L70), [knn_log.rs:61-128](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/operators/knn_log.rs#L61-L128), [knn.rs:166-235](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/orchestration/knn.rs#L166-L235) |
| C7/C7b | [compact.rs:710-728,919-1096](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/orchestration/compact.rs#L919-L1096); [register_orchestrator.rs:59-77,191-235](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/src/execution/orchestration/register_orchestrator.rs#L59-L77) |
| C8 | [wal3/README.md:1-105](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/wal3/README.md#L1-L105), diseño documental; API/LogPosition en `src/lib.rs`, no auditoría del writer completo |
| C9 | [hnsw_integrity_check.rs:25-179](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/hnsw_integrity_check.rs#L25-L179), args/report/exit |
| C10 | [hnsw_config_repair.rs:1-160](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/hnsw_config_repair.rs#L1-L160), `repair`, copy y migración sobre staging |
| C11 | [__init__.py:201-315,375-433](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/chromadb/__init__.py#L201-L315), [config.py:120](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/chromadb/config.py#L120), [bindings.rs:446-518](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/python_bindings/src/bindings.rs#L446-L518) |
| C12 | [local_segment_manager.rs:40-105,173-258](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/segment/src/local_segment_manager.rs#L173-L258), caché/identidad/cleanup |
| C13/C14 | [worker/benches/query.rs:1-265](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/benches/query.rs#L122-L265); [benchmark.rs:6-27](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/benchmark/src/benchmark.rs#L6-L27) |
| C15 | [sift.rs:20-169](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/benchmark/src/datasets/sift.rs#L20-L169); [load.rs:12-62](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/worker/benches/load.rs#L12-L62) |
| C16 | [collection_browser/mod.rs:29-91](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/tui/collection_browser/mod.rs#L29-L91); [events.rs:71-190](https://github.com/chroma-core/chroma/blob/1a3ac2c7c623adbd03ed2508dc27b98cdf26016d/rust/cli/src/tui/collection_browser/events.rs#L71-L190) |

### Seams Vanta al snapshot fijado

| Ref. | Evidencia, no propuesta de API ya existente |
|---|---|
| V1 | [wal.rs:850-907,1023-1028](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L850-L907), append/sync y formatos; lectura contrastada mediante `git show` para no usar HEAD concurrente |
| V2 | [backend.rs:34-57](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/backend.rs#L34-L57), particiones de fuente/derivados/versiones; capabilities opcionales en cabecera 7-18 |
| V3 | [vector.rs:129-248](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L129-L248); [audit.rs:7-36](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/audit.rs#L7-L36) |
| V4 | [hybrid.rs:13-60](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/hybrid.rs#L13-L60), presupuesto y fusión RRF |
| V5 | [Referencia de escritura/recall L1](VANTA-REFERENCE.md#capacidades-y-limites-discriminantes), `l1_writer::persist_planned`, `apply_dedup_batch`, `l1_dedup` y `InjectionPolicy`; lectura de referencia, no auditoría completa nueva |
| V6 | [index.rs:13-67](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L13-L67), rebuild; scheduler/checkpoints/Studio según [referencia](VANTA-REFERENCE.md) |

### Cobertura y lagunas

Leídos: contratos locales, fuentes C por rangos, licencia raíz/workspace,
proveedor ONNX por cabecera/config y V1–V4/V6 por lectura de snapshot;
V5 y Studio mediante referencia fijada. No leídos exhaustivamente: core
Python legacy, auth de cada deployment, writer/GC completo wal3, todas las
implementaciones SPANN, consola Cloud, clientes JS, modelos/licencias
transitivas, sample apps y resto de benches. Sin pruebas de pérdida de
energía, rendimiento, calidad ni equivalencia binaria de motores.

Router público usado tras cargar skills: DuckDuckGo para
`site:github.com/chroma-core/chroma/issues delete persist compaction`, sin
resultados; metasearch/Bing para `Chroma chroma github issue delete disk space`,
también vacío; fallback `websearch.fetchWebContent` a API pública de búsqueda
e issue/comentarios, y lectura JSON pública para evitar truncación de salida.
Una primera impresión Python falló por cp1252; se repitió con UTF-8.
No se modificó metadata congelada. Las fechas/estados de #7946 proceden de
API, no del buscador. No se leyó un merge/fix y no se afirma que exista.

Si se autorizan specs activas, separar A (recibos), B/D (derivados),
C/F (contratos de consulta/backend) y E (operación offline); este informe
supera 300 líneas por evidencia autónoma, no se crean specs en esta entrega.
Secuencia de dependencias: inventario → A/B → D; E puede empezar por auditoría
sin D nuevo; C/F son incrementales y no bloquean todos los demás.
Validación manual posterior: cancelación poscommit, replay de delete,
lectores durante publish/eviction, copia offline, paridad de filtro y errores
por backend. No se proponen aquí comandos de ejecución ni umbrales nuevos.

### Recibo durable de autoría — Chroma

- Archivo exclusivo completado: `chroma.md`; autoría del segundo agente,
  **no revisión independiente ni aceptación final**.
- Cambios: se preservan log/proyección, filtros e integridad como hallazgos
  útiles; se corrigen WAL universal, parquet local, filtro post-ANN genérico,
  default Rust, supuesta falta de benchmark y ausencia amplia de UI.
- Mecanismos: A recibos; B materialización/delete; C dominio de filtros;
  D publicación/retención; E auditoría/copia; F capacidades/ownership.
  Cada ficha incluye interfaz Rust propuesta, seam, pasos, migración,
  rollback, dependencias, FMEA y aceptación favorable/adversa pendiente.
- Fuentes públicas leídas: #7946 y sus comentarios en API GitHub; fuentes de
  código C1–C16 locales fijadas y Vanta V1–V6 con límites indicados.
- Lagunas: Cloud/auth/GC completo/licencias transitivas y validación dinámica.
  Estado documental: entregado por autor, sujeto a revisión del responsable.
- Operaciones: lectura local/git show/red pública y `apply_patch`; sin
  apps/tests/builds/benchmarks/install/servicios, sin git write ni commits.
  Copia de este recibo: `%TEMP%\vantadb-author-chroma.md`.
- Comprobación documental acotada a los dos informes: anclas propias y rutas
  de fuentes fijadas presentes; no validador global. `chroma.md` está
  **ignorado/no trackeado** por `.gitignore:268` (`docs/dev/research/`).
  El responsable debe decidir su inclusión durable en git; no se hizo staging
  forzado ni se modificó `.gitignore`.
