---
title: "SurrealDB: fronteras de rebuild y diagnóstico de compatibilidad"
kind: research
status: proposed
description: "Lectura estática de barreras locales, rollback-check y benchmark embebido con cautela explícita de licencia BSL"
tags: [research, memory, engineering]
---

# SurrealDB: fronteras de rebuild y diagnóstico de compatibilidad

## 1. Recomendación y arquitectura

**Dictamen:** estudiar contratos de publicación de índices y preflight de
compatibilidad; no importar SurrealQL completo, arquitectura de servidor ni
código BSL como si tuviera licencia permisiva.
Clone: `C:\Users\Eros\surrealdb`.
SHA congelado: `9577ac5f0caa60a39d5a334e9352aed0e232f9e6`.
Snapshot: **33 289 estrellas, `NOASSERTION`**, del [inventario](github-candidates.json).
`NOASSERTION` es el resultado del inventario, no permiso ni ausencia de licencia.
La licencia raíz leída es **Business Source License 1.1**, detallada en §6.
Preparación: [manifiesto](clone-manifest.json); no se consultó GitHub de nuevo.

El README declara motor multimodelo Rust, embebido, servidor y WASM [S0].
Hay evidencia de separación concreta: `engine-local` abre un Datastore y
sirve al SDK por `engine-api`; el core entra al seleccionar backend [S1].
Su interfaz entre crates se declara interna y sin garantía de estabilidad,
incluso en patches. No tratarla como API pública reutilizable estable.

```text
SDK Rust → engine-api → engine-local → Datastore → backend elegido
escritura → registro de epoch antes de snapshot → commit/cancel → fin de registro
rebuild → definición/generación → espera de escrituras anteriores → lectura
rollback_check(target) → ledger + formatos/catálogo → reporte, sin modificar
```

El flujo de barrera está documentado en [S2] y parcialmente implementado en
los rangos LiveWrites [S3]-[S4]. El preflight tiene código de veredicto y
recorrido [S6]. No se trazaron todas las consultas, escrituras ni backends.
Aunque el proyecto soporta escenarios distribuidos, el mecanismo interesante
para Vanta es la frontera **local** entre writers y estado derivado.

**Vanta:** base histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD concurrente comunicado `167435a9e`.
Se conserva el [baseline](METHOD.md#vantadb-como-referencia) y las capacidades
acreditadas: ANN/BM25/sparse/RRF, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints/ingestors/auditoría y Studio.
No se reaudita Vanta ni se presenta un hallazgo histórico como bug vigente.

## 2. Evidencia y extracciones mínimas independientes

| Ref. | Archivo y rango leído | Observación |
|---|---|---|
| S0 | `README.md:68-82` | Arquitectura declarada e imagen Surrealist |
| S1 | `surrealdb/engine-local/src/lib.rs:1-45` | Frontera del SDK embebido y estabilidad |
| S2 | `surrealdb/core/src/kvs/write_barrier.rs:1-60` | Contrato local y distribuido documentado |
| S3 | `surrealdb/datastore/src/triggers.rs:1476-1538` | open/close por epoch y begin_commit |
| S4 | `surrealdb/datastore/src/triggers.rs:1572-1625` | avance coordinado y consulta de finalización |
| S5 | `surrealdb/core/src/kvs/rollback.rs:1-165` | Fuentes leídas y tipos de hallazgo |
| S6 | `surrealdb/core/src/kvs/rollback.rs:165-261` | Veredicto y recorrido read-only |
| S7 | `surrealdb/engine-local/benches/point_read.rs:1-152` | Runner propio leído completo |
| S8 | `surrealdb/engine-local/Cargo.toml:118-134` | Registro bench sin harness |
| SL | `LICENSE:1-68` | BSL, restricción Database Service y Change Date |

El contrato documental S2 y su implementación parcial S3/S4 están separados;
no se infiere verificación completa del protocolo distribuido.

### Candidata A — frontera local de escritores durante rebuild

- **Problema:** un índice reconstruido no debe omitir escrituras iniciadas bajo
  una generación anterior que aún pueden confirmar tras comenzar el escaneo.
- **Unidad mínima:** inscripción de writer antes de snapshot, identidad de epoch
  y espera sobre escritores previos; S3/S4 acreditan ese núcleo conceptual.
- **Seam Vanta:** particiones derivadas y `InternalMetadata` del backend [V1],
  migración/formato de índice ya existente [V2], más auditoría del propietario.
- **Contrato hipotético:** `WriteEpochGuard { generation, operation_id }`
  y `RebuildBoundary { source_generation, prior_writers_settled }`.
- **Acción 1:** inventariar fences/generaciones y publicación actual de rebuild;
  no crear otro coordinador si el motor ya expresa la misma frontera.
- **Acción 2:** identificar la toma real de snapshot y el último punto en que
  una escritura antigua puede modificar el estado fuente.
- **Acción 3:** especificar inscripción y cierre ligados a esa frontera;
  cancelación de un future no equivale a commit/cancel del almacenamiento.
- **Acción 4:** exponer motivo de espera/estado en auditoría existente antes
  de alterar el algoritmo de construcción de índices.
- **Dependencias:** orden de snapshot/registro, resultado de commit/cancel,
  autoridad para publicar generación y coordinación con GC ya existente.
- **FMEA — inscripción tardía:** writer invisible al fence confirma después;
  la inscripción debe preceder al snapshot que determina la generación.
- **FMEA — cierre temprano:** Drop del llamador libera guard mientras una
  escritura en background sigue activa; el propietario real conserva el guard.
- **FMEA — espera sin fin:** writer abandonado mantiene rebuild bloqueado;
  preservar diagnóstico y estado, no declarar índice sano por timeout.
- **FMEA — bloqueo global innecesario:** afecta namespaces no relacionados;
  ámbito del contrato debe coincidir con el estado derivado mantenido.
- **Aceptación propuesta:** publicación cubre cada escritura confirmada de la
  frontera elegida o mantiene la generación anterior; no hay intervalo omitido.
- **Aceptación operativa:** espera y cancelación no anuncian progreso durable
  que el backend no confirmó; error deja estado de salud interpretable.
- **Independencia:** aplicable sin inspección de downgrade B.
- **No objetivos:** claves heartbeat, leases multinodo, Raft ni copiar S3/S4.
- **Licencia:** extracción conceptual; cualquier reutilización material queda
  detenida hasta revisión legal específica de BSL y del archivo.

### Candidata B — reporte read-only de compatibilidad por formato

- **Problema:** saber que una base abre no garantiza que todos sus índices,
  registros o operaciones puedan ser leídos/mantenidos por un binario anterior.
- **Unidad mínima:** separar veredicto de apertura, formatos afectados,
  objetos ilegibles/desconocidos y restricciones de escritura.
- **Fuente:** S5/S6; el veredicto usa ledger, mientras generaciones/entries
  detallan rechazos posteriores sin cambiar automáticamente dicho veredicto.
- **Seam Vanta:** `FormatKind` ya enumera File, VectorIndex, WAL, Records y
  Schema; el orden de migración de records precede al schema bump [V2].
- **Contrato hipotético:** `CompatibilityReport { target, open_mode,
  format_findings, unknown_count, unreadable_count, observation_scope }`.
- **Acción 1:** inventariar reports/preflights actuales y versiones por formato.
- **Acción 2:** declarar qué compatibilidad puede inferirse de cabeceras y cuál
  exige recorrer estructuras; un campo desconocido no significa compatible.
- **Acción 3:** preservar categorías de daño, formato futuro y no inspeccionado.
- **Acción 4:** integrar diagnóstico read-only con migración existente,
  sin ejecutar reparación ni rebajar versión como efecto de inspección.
- **Dependencias:** decoders versionados, conocimiento de formatos publicados,
  acceso autorizado y frontera de observación consistente.
- **FMEA — falso seguro:** sólo se valida apertura y un índice sigue ilegible;
  detallar alcance y operaciones restringidas.
- **FMEA — desconocido tratado como vacío:** omite material incompatible;
  unknown debe permanecer visible y no autorizar escrituras.
- **FMEA — informe mezclado:** lecturas sucesivas ven generaciones distintas;
  declarar snapshot/alcance o indicar que el diagnóstico no es atómico global.
- **Aceptación propuesta:** informe no modifica bytes, no repara ni promete
  downgrade; distingue «abre», «sólo lectura» y «no acreditado» donde proceda.
- **Aceptación de seguridad:** reportes no filtran nombres/payloads de namespaces
  que el operador no puede inspeccionar.
- **Independencia:** útil sin rebuild online ni candidata A.
- **No objetivos:** nuevo formato de disco, un migrador automático o copiar ledger.
- **Licencia:** mismo límite conceptual; no código derivado autorizado aquí.

## 3. Riesgos, defectos e hipótesis

**Estático S3:** open cuenta antes de entregar epoch; close reduce el contador;
begin_commit se cuenta antes de leer fences. **S4:** avance bloquea shards
en orden y comprueba epochs anteriores. No se certifica todo el lifecycle
de Transaction ni todos los puntos que invocan estas funciones.
Su corrección depende de invariantes externos no recorridos.

**S2 documental:** el caso multinodo depende de heartbeats, marcas de versión,
leases y acuerdo de relojes dentro del margen descrito.
Es una condición del protocolo, no fallo observado; no se traslada a Vanta.
**Hipótesis de adaptación:** un guard liberado al cancelar el wrapper mientras
persiste el commit produciría una frontera falsa.

**Estático S6:** rollback_check abre lecturas separadas para raíz/namespace/DB;
no acredita un snapshot único de toda la base bajo escrituras concurrentes.
No es defecto demostrado: el resultado debe interpretarse según ese alcance.
Veredicto `Opens` no equivale a compatibilidad de todas las generaciones.
S5 cuenta claves/valores ilegibles sin inventar un nombre fiable para ellos.

Fronteras de confianza: formato persistido → decoder; operador → namespaces;
writer → generación publicada; código upstream → producto Vanta/licencia.
No issues consultados, CVEs atribuidas ni reproducciones.
No se declara que Vanta tenga pérdidas de escrituras o downgrade defectuoso.

## 4. Prerrequisitos Vanta

1. Revalidar fences/rebuild y reports actuales sobre el HEAD concurrente.
2. Conservar auditorías ANN/texto/WAL/backend y sus estados de salud.
3. Definir propietario real de escrituras cuando sobreviven al llamador.
4. Inventariar versiones de formatos sin confundirlas con versión de paquete.
5. Preservar invariantes de migración ya presentes; un preflight no debe reparar.
6. Revisar datos disponibles para Studio y bindings antes de ampliar contratos.
7. Revisar licencia antes de cualquier reutilización material o dependencia.

## 5. Qué no trasladar

No portar el lenguaje SurrealQL completo ni catálogo/VM/multimodelo integral.
No importar servidor, auth de usuario final o scripting al motor por analogía.
No copiar heartbeats/fencing distribuido para resolver rebuild de un proceso.
No asumir estabilidad de `engine-local` por estar escrito en Rust.
No confundir publicación de un índice con durability del almacenamiento fuente.
No considerar BSL «Apache futura ya vigente» ni sustituir asesoramiento legal
por una reescritura nominal.

## 6. Método, licencia y cobertura

Sólo lectura offline y redacción; sin scripts upstream, instalación, build,
tests, apps/servicios, benchmarks, red ni commits/push.
Rangos S0-S8/SL y seams históricos V1/V2; no auditoría exhaustiva.
Omitidos: transacciones completas, index builders, HNSW/DiskANN, planner,
SurrealQL, auth/RBAC, RPC, WASM, SDKs restantes y recuperación física.
Historial superficial, otras ramas/tags y LFS según manifiesto.

**Licencia material:** SL dice BSL 1.1, Licensed Work SurrealDB 3.0,
Additional Use Grant que excluye uso como «Database Service» según su definición,
Change Date **2030-01-01**, Change License Apache-2.0.
El propio texto declara que BSL no es licencia Open Source.
No se resuelve aquí cómo aplica cada cláusula a Vanta, distribución o servicios.
La conversión futura tiene condiciones por versión; no autoriza hoy una copia
permisiva. Código derivado y redistribución requieren revisión específica.
La investigación de mecanismos no aprueba una dependencia ni una transposición
de código. Reimplementar una idea no elimina automáticamente riesgo de derivación.

## 7. Benchmark upstream realmente leído

**S7:** `surrealdb/engine-local/benches/point_read.rs`, registrado con
`harness=false` [S8]. Es un programa de medición propio, **no Criterion**.
Usa Tokio current_thread, Datastore memory, mantenimiento desactivado y sesión
owner en namespace/DB de bench; no arranca servidor en ese código.
Dataset sintético: 1 000 registros person con name, age y tags.
Dos esquemas: schemaless/schemafull; cuatro formas de SELECT por ID, llave
variable, literal variable y literal fijo.

Por defecto: 2 000 warmups y 20 000 solicitudes medidas por caso;
`BENCH_ITERS`, `BENCH_WARMUP` y `BENCH_STATEMENT_CACHE` son controles existentes.
No se propone modificar sus valores ni introducir configuración.
IDs variables usan `(i*7919)%1000`; literal fijo lee person:1.
Cronometra execute y extracción de resultado; informa **media y mediana en μs**.
Comprueba una fila en la primera iteración de cada caso, no una auditoría de
correctitud completa por solicitud.

Dependencias: toolchain Rust y crates core/types/cnf/Tokio, backend memory
disponible y features de compilación apropiadas; nada se compiló para probarlo.
No dataset externo, embeddings, generación ni juez.
Statement cache, ruta compilada y esquema forman parte del trabajo medido.
No mide ANN/recall, persistencia en disco, caída de proceso, throughput paralelo,
cliente de red, p95/p99 o permisos de usuario no-owner.
No atribuir resultados de benchmarks de SurrealKV o ANN a este runner.

Se localizaron `core/benches/index_ann.rs`, `engine-local/benches/scan.rs` y
otros, pero no se leyeron como runners: sin atribución de métricas.
No hay resultado medido propio ni superioridad frente a Vanta.
Comparación futura: [protocolo](METHOD.md#protocolo-de-comparacion-de-benchmarks).
Sin nuevos prompts/jueces/evals, thresholds, sampling ni configuraciones.

## 8. Matriz frente a Vanta

| Capacidad | Estado Vanta | Delta defendible |
|---|---|---|
| ANN/BM25/sparse/RRF | Existe | No importar otro motor multimodelo |
| Índices derivados y marcadores de salud | Existe en base V1 | Revisar explicación de frontera rebuild |
| Migración por formato | Existe V2 | B amplía diagnóstico, no crea migrador |
| Provenance/sesiones/supersession/dedup | Existe | Conservar referencias durante rebuild |
| Assembler/compaction/checkpoints/scheduler | Existe | No reemplazar con runtime SurrealDB |
| Barrera local registrada antes de snapshot en toda ruta | No acreditado aquí | A condicionada a inventario actual |
| Preflight uniforme de downgrade por formato | No acreditado aquí | B sólo si falta alcance equivalente |
| Integración end-to-end de los contratos candidatos | Parcial: seams, no trazado actual | Revalidación previa |
| SurrealQL/consenso completo como requisito | No aplicable | Excluido |

## 9. Carencias sugeridas, no bugs certificados

Dos preguntas acotadas: ¿la auditoría explica qué escritores cubre un rebuild
y qué generaciones se publicaron?, ¿un reporte distingue apertura de lectura/
mantenimiento efectivo por formato?
No se prueba ausencia de barreras ni incompatibilidad de datos en Vanta.
La mejora mínima puede consistir en ampliar reportes actuales.
La licencia hace preferible estudiar invariantes y no copiar implementación.
Toda tarea futura exige revalidar estado concurrente y decisión del propietario.

## 10. UI, escritorio e inspección

README incluye imagen identificada como **Surrealist** [S0].
Eso acredita una referencia de producto, no fuente frontend incluida.
No se inspeccionó repositorio externo de Surrealist ni su empaquetado desktop;
no se afirma aquí disponibilidad de ese código dentro de este checkout.
S1 acredita motor embebido Rust, no aplicación gráfica.
Vanta ya tiene Studio Tauri/React en `develop-desktop`: A/B aportarían estados
de rebuild y compatibilidad a sus inspectores, sin crear otra UI.
Runtime necesario: **ninguno**; no iniciar/reiniciar servicios.

[S0]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/README.md#L68-L82
[S1]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/src/lib.rs#L1-L45
[S2]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/write_barrier.rs#L1-L60
[S3]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/triggers.rs#L1476-L1538
[S4]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/datastore/src/triggers.rs#L1572-L1625
[S5]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/rollback.rs#L1-L165
[S6]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/core/src/kvs/rollback.rs#L165-L261
[S7]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/benches/point_read.rs#L1-L152
[S8]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/surrealdb/engine-local/Cargo.toml#L118-L134
[SL]: https://github.com/surrealdb/surrealdb/blob/9577ac5f0caa60a39d5a334e9352aed0e232f9e6/LICENSE#L1-L68
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/backend.rs#L1-L65
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/migration.rs#L1-L65
