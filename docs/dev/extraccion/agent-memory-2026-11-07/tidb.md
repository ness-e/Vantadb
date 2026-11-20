---
title: "TiDB: atribución de recursos y cancelación con causa"
kind: research
status: proposed
description: "Análisis estático de mecanismos locales de TiDB aplicables a un motor embebido sin trasladar su arquitectura distribuida"
tags: [research, memory, engineering]
---

# TiDB: atribución de recursos y cancelación con causa

## Índice

- [0. Identidad, mapa y flujos reales](#0-identidad-mapa-y-flujos-reales)
- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura)
- [2. Evidencia y extracciones mínimas independientes](#2-evidencia-y-extracciones-mínimas-independientes)
- [3. Riesgos y defectos, separados de hipótesis](#3-riesgos-y-defectos-separados-de-hipótesis)
- [4. Prerrequisitos Vanta](#4-prerrequisitos-vanta)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Benchmark upstream leído, no ejecutado](#7-benchmark-upstream-leído-no-ejecutado)
- [8. Matriz frente al estado existente](#8-matriz-frente-al-estado-existente)
- [9. Carencias sugeridas, no bugs inventados](#9-carencias-sugeridas-no-bugs-inventados)
- [10. UI y escritorio](#10-ui-y-escritorio)
- [11. Cobertura y cierre](#11-cobertura-y-cierre)

## 0. Identidad, mapa y flujos reales

Ampliación documental de 2026-11-07. Se preservan A/B y el microbenchmark de
la ficha parcial; se corrige su referencia Vanta histórica y se agregan flujos,
incidencia pública leída y lifecycle de callbacks. Clon congelado y HEAD
releído: `41efd866c3801a637ab2d6712b86d8b2cd77ac5b`; sin fetch/checkout.
Contrato: [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](VANTA-REFERENCE.md), [METHOD](METHOD.md),
[BENCHMARK-NOTES](BENCHMARK-NOTES.md), leídos y no modificados.

| Mapa/stack | Implementación o evidencia | Límite |
|---|---|---|
| Proceso y API | `cmd/tidb-server`, `pkg/server`, `pkg/session`; Go [TG] | Servidor SQL/MySQL, no biblioteca Rust embebida |
| Parse/plan/ejecución | `pkg/parser`, planner/compiler y executor [T1][T2] | No se auditaron todos los operadores ni grammar |
| Persistencia | Driver `pkg/store/driver/txn/txn_driver.go` [TD] | Delega en cliente TiKV; TiKV/Raft y disco remoto no se inspeccionaron |
| Estado de sesión | `session.go::CommitTxn/RollbackTxn` [TC] | Transacción distinta de sesión conversacional o checkpoint de agente |
| Recursos | `pkg/util/memory/tracker.go` [T3][T4] | Bytes cooperativos y acciones; no RSS exacto |
| Cancelación | `pkg/util/sqlkiller/sqlkiller.go` [T5][T6] | Señal cooperativa, no preemption del proceso |
| Observabilidad | HTTP status/metrics/trace/pprof [T8] | No toda esa superficie es TiDB Dashboard |
| Dependencias | `go.mod`, TiKV client y protocolos externos | Enterprise no inicializado según manifiesto; no le atribuir capacidades |

### Modelo y fronteras de almacenamiento

La API SQL trabaja con filas/tablas y planes. En el tramo driver leído, las
mutaciones son claves/valores bytes, con buffer transaccional y lectura que
puede consultar snapshot: no texto de recuerdos, embeddings ni provenance
de agente automáticamente. `Tracker` contiene padre, consumo, máximo y acciones;
`SQLKiller` contiene señal, evento y callback de fin. Son estado de ejecución,
no memorias durables. LLM, extracción, answerer y juez no intervienen aquí.

1. **Write/update/delete:** el driver `tikvTxn.Set/Delete` delega directamente a
   `KVTxn.Set/Delete` [TD:195-198,213-215]. Ese rango no implementa ni demuestra el
   buffering interno del cliente: no inferir commit o flush SQL del nombre.
   Update de bytes tampoco implementa supersession con provenance de memoria.
2. **Read:** `tikvTxn.Get` llama `GetMemBuffer().Get`; sólo ante `ErrNotExist`
   usa `GetSnapshot().Get`. Otros errores se retornan; tras lectura
   exitosa `IsValueEmpty` se traduce a `kv.ErrNotExist` [TD:200-210].
   No hay llamada `MemBuffer.GetLocal` ni fallback `KVTxn.Get` en ese driver.
   El isolation level de cada sesión,
   stale reads y visibilidad entre transacciones requieren su manager. No
   certificar serializabilidad global por esa función.
3. **Commit:** `session.CommitTxn` llama `doCommitWithRetry`, agrega detalles y
   actualiza `LastCommitTS` sólo bajo sus condiciones [TC]. `doCommit` comprueba
   read-only/placement/opciones y llega a `commitTxnWithTemporaryData` [TPRE].
   Driver `tikvTxn.Commit` delega a `KVTxn.Commit(ctx)` y convierte errores [TD].
   Esta lectura no demuestra orden físico de fsync/replicación del cliente.
4. **Rollback:** `session.RollbackTxn` solicita `txn.Rollback`, invalida y limpia
   estado de sesión [TC]. No prueba que un timeout de COMMIT implique rollback
   remoto: resultados inciertos conservan esa incertidumbre.
5. **Presión/fallo:** `Consume` propaga a ancestros, consulta kill y puede llamar
   acción/panic [T4]. Señal de kill y liberación real son hechos distintos;
   error del cliente, error SQL y cierre del result set no equivalen a borrar
   datos ya confirmados.
6. **Ownership:** la sesión coordina transacción/callbacks; buffers de resultados
   pueden vivir más que un operador. En Rust el guard contable debe seguir al
   recurso, no morir al retornar la consulta si el SDK conserva el resultado.

**Receipt canónico:** para cualquier escritura Vanta se consume el
[C13 MutationReceipt](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén),
no `CompletionState` como reemplazo. `operation_id`, `primary_commit`,
`primary_durability: Option<DurablePosition>`, `projections`,
`visibility_generation`, `outcome` separan conocimiento de commit/durabilidad/
readiness. `accepted`/`applied`/`durable` no son un orden total; primario durable
puede preceder al índice visible. Timeout o cancelación no prueban rollback.

## 1. Recomendación y arquitectura

**Dictamen:** estudiar atribución de memoria y terminación de operaciones,
no migrar consenso, TiKV/TiFlash, SQL completo ni despliegue de clúster.
Clone: `C:\Users\Eros\tidb`.
SHA congelado: `41efd866c3801a637ab2d6712b86d8b2cd77ac5b`.
Snapshot: **40 715 estrellas, Apache-2.0**, del [inventario](github-candidates.json).
Preparación y submódulo enterprise omitido: [manifiesto](clone-manifest.json).
No se actualizó el checkout. Esta ampliación sí consultó fuentes públicas (§3/§10).

TiDB es una base SQL distribuida; el código principal aquí es Go.
El README describe separación de cómputo/almacenamiento y coordinación con
TiKV/TiFlash [T0]. Esa descripción no implica que su consenso esté implementado
por el pequeño subsistema local analizado.
`session.Execute` parsea, limita su API a una sentencia y llama ExecuteStmt [T1].
Otro tramo construye `executor.Compiler` y transforma AST en plan físico [T2].
No se trazó toda la ejecución SQL ni cada petición distribuida.

```text
SQL/API de sesión → Parse → ExecuteStmt → Compiler → plan físico
operadores/allocations declaradas → tracker hijo → ancestros → límite/acción
causa de cancelación → señal + evento → HandleSignal → error tipado
HTTP de diagnóstico → status/metrics/trace/pprof
```

Tracker es contabilidad cooperativa, no captura automática de cada byte del
allocator. SQLKiller representa causa y notificación; no prueba que todo loop
de operador consulte inmediatamente la señal.
Son mecanismos locales separables del almacenamiento distribuido.
El valor para Vanta es explicar presión y finalización de consultas/ingestas,
no adquirir escalabilidad distribuida por copiar unas estructuras.

**Vanta:** referencia revalidada `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`
en `develop`, no las bases históricas de la ficha inicial.
Se conserva el [inventario discriminante](VANTA-REFERENCE.md) y sus capacidades:
ANN/BM25/sparse/RRF, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints/ingestors/auditoría y Studio.
Se releen gobernador, seams y checkpoint acotados; no se audita todo el motor
ni se convierten hallazgos antiguos en actuales.

## 2. Evidencia y extracciones mínimas independientes

| Ref. | Archivo y rango leído | Evidencia |
|---|---|---|
| [T0]/[TG] | `README.md:16-59`; `go.mod:1-35` | Arquitectura declarada, módulo y stack |
| T1 | `pkg/session/session.go:1947-1968` | Parse y despacho público |
| T2 | `pkg/session/session.go:2664-2692` | Compilación y grupo de recursos |
| [T3S]/[T3] | `pkg/util/memory/tracker.go:78-110,333-384` | Jerarquía, attach/detach y estado |
| T4 | `pkg/util/memory/tracker.go:450-609` | Reversal, consumo y acciones de límites |
| T5 | `pkg/util/sqlkiller/sqlkiller.go:30-202` | Causas, evento, CAS y callback protegido |
| T6 | `pkg/util/sqlkiller/sqlkiller.go:204-268` | Observación, conexión y principio de reset |
| T7 | `pkg/util/memory/bench_test.go:1-42` | Tres microbenchmarks Go completos |
| [TE]/[TS]/[T8] | `pkg/server/http_status.go:77-95,283-288,402-425` | Diagnóstico y trace viewer |
| [TL] | `LICENSE:1-30` y cabeceras de T3-T7 | Apache-2.0 |

Las llamadas concretas están enlazadas al SHA, no a master mutable.
La tabla no acredita todos los allocators, ejecutores, stores o políticas.

### Candidata A — recibo de recursos por operación bajo gobernador existente

- **Problema:** un límite global no explica qué consulta/ingesta conservó o
  liberó recursos, y un contador manual puede quedar desbalanceado en errores.
- **Unidad mínima:** scope de contabilidad + reserva con propietario y liberación
  de una sola vez; T3/T4 permiten contrastar jerarquía y ciclo de vida.
- **Seam Vanta:** `MemoryGovernor` existente, con used_bytes y allocated/freed
  [V1], y resultados de ejecución materializados [V2].
- **Contrato hipotético:** `OperationBudgetRef { operation_id, parent_scope }`,
  `TrackedReservation` no clonable y `ResourceReceipt { estimated, peak, scope }`.
- **Acción 1:** inventariar contabilidad actual y reservas equivalentes antes
  de introducir otro tipo de gobernador.
- **Acción 2:** distinguir bytes reservados, capacidad retenida y uso estimado;
  no presentar una suma cooperativa como RSS exacto del proceso.
- **Acción 3:** representar propiedad mediante RAII Rust y transferencia explícita
  cuando un buffer sobrevive a la consulta o entra en una caché.
- **Acción 4:** integrar el receipt con diagnósticos actuales sin cambiar
  ranking, límites o política de eviction.
- **Dependencias:** propietario de buffers, convenciones de tamaños, scope
  estable, sincronización actual y errores del gobernador.
- **FMEA — double free contable:** copiar un guard libera dos veces; tipo
  no clonable y transferencia única, no llamadas manuales dispersas.
- **FMEA — liberación prematura:** el resultado sigue vivo pero su scope termina;
  la reserva debe seguir al almacenamiento superviviente.
- **FMEA — doble cargo:** padre y caché contabilizan el mismo buffer compartido;
  declarar identidad y propietario de capacidad.
- **FMEA — exceso antes de reserva:** contabilizar después de una gran asignación
  ya no es admisión preventiva; no prometer límite duro si sólo es medición.
- **Aceptación propuesta:** alta/baja equilibradas para recursos rastreados,
  incluso al devolver error; parent y leaf no duplican totales.
- **Aceptación de reporte:** estimación se etiqueta como tal, y desactivar
  diagnóstico no altera resultados funcionales ni permisos.
- **Independencia:** útil con operaciones no cancelables, sin candidata B.
- **No objetivos:** portar arbitrador global, finalizadores Go o nuevos umbrales.

### Candidata B — cancelación por operación con causa estable

- **Problema:** distinguir timeout, cancelación explícita y presión de recursos
  sin cancelar una operación nueva que reutilizó el mismo canal/sesión.
- **Unidad mínima:** señal monotónica por identidad de operación y conversión
  de causa a resultado; tomar T5/T6 como contraste.
- **Seam Vanta:** executor [V2], gobernador [V1] y worker por sesión [V3];
  no requiere añadir otro scheduler.
- **Contrato hipotético:** `OperationCancellation { operation_id, cause }`
  y `CompletionState { completed, cancelled, write_outcome_unknown }`.
- **Acción 1:** localizar tokens/contextos y causas de error actuales.
- **Acción 2:** definir quién puede solicitar cancelación y qué operación posee
  la señal; una sesión reutilizable no es identidad suficiente.
- **Acción 3:** mapear causas a errores/reportes sin perder el primer motivo
  válido; reusar primitivas async/sync ya adoptadas por Vanta.
- **Acción 4:** especificar observación en fronteras existentes de trabajo y
  limpieza sin agregar polling/configuración nueva durante esta investigación.
- **Dependencias:** identidad, ciclo de vida, ownership de tareas/callbacks y
  semántica de commit del backend.
- **FMEA — señal tardía:** cancela la siguiente consulta de la sesión; ligar
  notificación a identidad/generación, no a reset global sin propiedad.
- **FMEA — callback superviviente:** cierre de recurso nuevo por callback viejo;
  desregistrar bajo la misma frontera de ownership que su ejecución.
- **FMEA — cancelación tras escritura:** el llamador interpreta rollback;
  separar cancelación observada de resultado durable conocido/desconocido.
- **FMEA — loop no cooperativo:** señal emitida pero trabajo continúa; no
  prometer preemption ni liberación instantánea.
- **Aceptación propuesta:** una señal de operación finalizada no afecta su
  sucesora; causa reportada y cleanup corresponden a la operación correcta.
- **Aceptación de escritura:** no anunciar rollback sin evidencia del backend;
  preservar idempotencia/checkpoints existentes al reintentar.
- **Independencia:** funciona por cancelación del usuario sin tracker A.
- **No objetivos:** kill de procesos, MySQL wire, contexto Go o cuotas de clúster.

### Precisiones contractuales de A/B

**A:** entrada scope/operación y reserva de capacidad estimada; salida guard
no clonable y receipt de bytes estimados/peak/retención, nunca `MutationReceipt`.
Errores tipados propuestos: límite, overflow, scope cerrado, transferencia
inválida; no `panic` a través de FFI. Concurrencia: jerarquía acíclica, reparent
serializado y total sin doble cargo para `Arc` compartidos. No mantener mutex de
contabilidad al ejecutar callback arbitrario. Cancelación libera sólo lo que
ya no tiene propietario; resultados/caché conservan su cargo. Persistencia:
contadores efímeros, no migración on-disk salvo reporte opcional versionado.
Rollback de instrumentación no altera límites existentes ni payloads.

**B:** entrada identidad no reutilizable, autorización y causa; salida solicitud
aceptada/ya finalizada, separada del resultado efectivo. Primera causa y causalidad
deben fijarse antes de publicar evento, sin transformar reset en nueva identidad.
Errores propuestos: operación desconocida, owner/epoch obsoleto, no autorizado,
storage y outcome incierto. Pasos posteriores al inventario: instrumentar una
ruta existente, propagar por bindings, comprobar cleanup y luego ampliar cobertura.
Estado cancelación efímero; si se persiste resultado, schema aditivo compatible
y conservación de desconocido tras reinicio. Alternativa preferida: extender
`CancellationRegistry` ya presente en MCP si cubre identidad y transporte;
no convertir ese registro en scheduler global ni asumir que cubre SDK embebido.
Desactivar reporte no revierte escritura ni reejecuta automáticamente.

**Aceptación manual adicional:** normal/error/cancelación mantienen balances de
recursos vivos; dos operaciones de una misma sesión no comparten señal; error
primario y error de cleanup no se sobrescriben; receipt de mutación conserva
conocimiento parcial tras timeout. Diagnóstico sin SQL/payload/secretos por defecto.
Abiertos: ownership de resultados compartidos, alcance de instrumentación y
qué rutas son cooperativamente cancelables; no fijar cuotas/polling nuevos aquí.

### Candidata C — registro de acciones ligado a vida del propietario

- **Caso agente:** consultas/ingestas reabren operadores durante una sesión larga;
  el registro de callbacks de presión no debe retener buffers de cada intento
  ni llamar un closure de una operación terminada.
- **Evidencia:** `reArrangeFallback:301-312` conserva cadenas por prioridad;
  `UnbindActionFromHardLimit:280-298` ya permite retirar una acción [TCHAIN].
  `TopNExec.OpenSelf:109-161` registra acción con spillHelper [TTOP];
  issue [#72234][TI] reporta retención por reaperturas. No afirmar que upstream
  carezca de API de unbind ni que el reporte esté reproducido aquí.
- **Delta Vanta:** `MemoryGovernor` sólo acredita contadores/watermarks [V1];
  A instrumenta bytes y C instrumenta ciclo de vida de callbacks, no son DTOs
  duplicados. Aplicable **sólo** si se añaden/usarán acciones por operación;
  no se demostró cadena equivalente defectuosa en Vanta.
- **Contrato Rust propuesto:** `register_action(operation_id, owner_epoch,
  action_ref) -> Result<ActionRegistration, RegistrationError>`; guard no clonable,
  `Drop` desregistra identidad exacta. Registry conserva weak refs o referencias
  de vida acotada; ejecución toma propietario vigente sin sostener lock global.
- **Invariantes/errores:** un guard no elimina registro de otra generación;
  `ClosedScope`, `StaleOwner`, `DuplicateRegistration`, `Unauthorized` diferenciados;
  callback no causa pánico que atraviese FFI. Cancelar desregistra sin fingir
  que los buffers aún retenidos fueron liberados.
- **Pasos:** inventariar callbacks actuales; especificar relación registro→owner;
  integrar con gobernador y finalización existente; cubrir apertura fallida,
  reapertura y cierre concurrente; exponer sólo número de registros vivos.
- **Persistencia/deps:** registro efímero; ninguna migración de datos. A permite
  atribuir retención pero C no exige B. Desactivación drena callbacks y conserva
  recursos con propietarios reales; no guardar closures en disco.
- **Alternativas:** no registrar callbacks si basta señal existente; registro
  estable por operador; unbind explícito probado; rechazar lista global de
  closures fuertes con vida de sesión si retienen memoria por intento.
- **FMEA:** callback captura `Arc` de buffer → contador bajo y RSS alto →
  contrastar recursos vivos con registros → weak/guard; cierre compite con
  callback → use-after-close lógico → validar epoch y tomar owner vigente;
  reentrada bajo mutex → deadlock → ejecutar fuera de lock.
- **Aceptación manual futura:** repetir open/close deja sólo acciones de owners
  vivos; callbacks viejos no actúan sobre nueva operación; error de apertura no
  deja registro; cancelar mientras se ejecuta no libera recurso todavía usado.
  No reportar RSS exacto desde un contador cooperativo.
- **Mínimo/no objetivos:** una familia de callbacks, sin portar spill SQL ni
  arbitrador Go. Abierto: callback concurrente ya iniciado se drena o finaliza
  cooperativamente según contrato del owner, nunca se mata el proceso.

## 3. Riesgos y defectos, separados de hipótesis

**Estático T4:** consumo puede consultar SQLKiller y hacer `panic(err)`.
Las acciones de límite se ejecutan bajo su mutex; la política depende de los
implementadores de ActionOnExceed, no leídos integralmente.
No trasladar panic a una biblioteca embebida con FFI como respuesta por defecto.
Es una diferencia de contrato, no un bug de TiDB probado.

**Estático T3:** attach transfiere consumo al padre y detach altera acciones de
raíz de sesión. La seguridad del ciclo depende del propietario y de coordinación
externa. No se demostró una carrera de attach/detach upstream.
**Hipótesis de adaptación:** reutilizar scopes/callbacks con operaciones
superpuestas puede imputar consumo o cancelación al trabajo equivocado.

T5 serializa escritores de señal con estado del evento y utiliza CAS para la
primera causa; T6 usa lectura atómica y protege descripción cuando corresponde.
La lectura de reset se detuvo en 268: no se certifica su implementación completa.
La presencia de un enum de causas no acredita conversión íntegra en todos los
drivers remotos, y el propio comentario exige mantener esa conversión.

Fronteras: usuario/host → cancelación de operación; operadores → contabilidad;
diagnóstico → información de consultas. No reproducciones ni CVEs atribuidas.
Nada de lo anterior constituye hallazgo vigente de Vanta.

### Incidencia pública efectivamente leída

[#72234 — TopN as inner child of an Apply join retains…][TI],
**abierta**, creada 2026-10-23, actualizada 2026-10-30; estado consultado
2026-11-07 por REST GitHub. Se leyó cuerpo completo y
[primer comentario](https://github.com/pingcap/tidb/issues/72234#issuecomment-6372472845).
El autor reporta reproducción en v8.5.8 y producción v26.3: reaperturas TopN bajo
Apply retienen scaffolding mediante cadena de acciones; memoria del heap crece
mientras contador de consulta sigue bajo. Sus cifras son **reporte**, no medición
propia ni descripción universal de master.

El comentario propone `SetFinished` y omitir acciones terminadas al insertar;
no se leyó una PR de corrección que permita darlo por resuelto. En el SHA
congelado, [TTOP] y [TCHAIN] acreditan registro/fallback y unbind disponible,
no certifican todos los caminos Close ni cada retención reportada. Separar
existencia de la primitiva de desregistro de uso correcto por cada operador.

**Discriminante futuro:** mismo operador reabierto dentro de una sesión frente a
operadores independientes que se destruyen; inspeccionar reachability de callback
y memoria viva, no sólo contador. Para Vanta el riesgo es un closure nuevo que
retenga owner/buffer, **no** la importación del bug TopN a un motor sin ese Apply.
Los workarounds SQL/concurrencia del autor no se recomiendan a Vanta.

**Búsqueda y límites:** DuckDuckGo `websearch.search` con
`site:github.com/pingcap/tidb/issues memory tracker cancel` dio cero;
metasearch Bing/Yahoo sobre memory/tracker dio cero. REST search permitió #72234
y lectura de issue/comentario. Una búsqueda adicional `repo:pingcap/tidb
is:issue "SQLKiller"` recibió rate-limit: no se inventa otra incidencia ni se
presenta un censo de seguridad. El código de reset completo y conversiones de
todos los errores remotos siguen fuera de cobertura.

## 4. Prerrequisitos Vanta

1. Revalidar estado actual de gobernador, tokens, receipts y reportes.
2. Diferenciar presupuesto de contexto en tokens de memoria operativa en bytes.
3. Identificar recursos compartidos con cachés y duración de resultados SDK.
4. Mantener locks/checkpoints existentes y no duplicar worker/scheduler.
5. Revisar errores a través de bindings antes de ampliar causas públicas.
6. Acordar garantías de cancelación frente a commit/WAL por backend.
7. Mantener diagnóstico por namespace y sin payloads/credenciales en logs.

C sólo requiere definir lifecycle de acciones cuando existan; no bloquear A/B
por una cadena que no se acreditó en Vanta. A requiere transferencia de ownership,
B relación cancelación/commit, C desregistro exacto. Los tres pueden aprovechar
servicios existentes sin importar SQL. Una señal de cancelación no autoriza
consultar datos ni inspeccionar otra operación/tenant.

## 5. Qué no trasladar

No portar Raft, 2PC, TiKV/TiFlash, PD o el plano de control de TiDB.
No introducir servidor Go o MySQL para añadir dos contratos locales.
No adoptar panic/kill global como política del SDK embebido.
No trasladar watermarks, cuotas ni algoritmos de arbitraje sin necesidad.
No confundir contabilidad declarada con medición física de memoria.
No requerir una consulta SQL para observar el estado de una operación local.

## 6. Método, licencia y cobertura

Análisis estático: archivos/rangos T0-T8, driver/commit/acciones añadidos y seams
Vanta al snapshot. Hubo red de lectura pública (§3/§10), no ejecución remota.
No instalación, scripts upstream, builds, tests, benchmarks ni servicios.
No commits ni push; clones no modificados.
Apache-2.0 del inventario concuerda con LICENSE y cabeceras inspeccionadas.
Conservar licencia/avisos y revisar patentes/NOTICE/material concreto antes de
copiar; no se certifica cada dependencia del amplio `go.mod`.

Cobertura omitida: consenso, protocolos, optimizer completo, storage remoto,
acciones de spill, resource manager completo, SQL security, callbacks reales
y convergencia entre nodos. No se leyó todo TiDB.
El manifiesto omite `pkg/extension/enterprise`; no atribuirle comportamiento.
Historial shallow, ramas/tags y LFS no materializados según preparación.
Los enlaces V1-V3 se fijan a `97c9623...`; gobernador/checkpoint se releyeron,
sin certificar concurrencia completa. Licencia principal Apache-2.0 no cubre
automáticamente enterprise omitido, TiKV/TiFlash, Dashboard y todo el grafo Go.
BSL/source-available no sería OSI por aparecer como dependencia; revisar cada
componente antes de copiar. Reimplementación Rust no elimina derivación legal.

## 7. Benchmark upstream leído, no ejecutado

**T7** contiene tres runners `testing.B` de Go:

| Runner existente | Entrada/trabajo | Qué representa |
|---|---|---|
| `BenchmarkMemTotal` | Repetir `MemTotal()` | Coste de consulta de memoria total |
| `BenchmarkMemUsed` | Repetir `MemUsed()` | Coste de consulta de memoria usada |
| `BenchmarkConsume` | RunParallel; hijo adjunto a padre; Consume(256<<20) | Contabilidad concurrente del tracker |

El último incrementa contadores en **256 MiB por iteración**, no asigna un
buffer de 256 MiB. Confundirlo con throughput real del allocator es incorrecto.
Padre e hijos se crean sin límite (`-1`); no mide rechazo por presupuesto ni
eficacia de eviction. Los hijos no se desmontan dentro de cada iteración.
No hay dataset documental, embedding, LLM ni juez.
Métrica temporal corresponde a harness Go; el archivo no produce por sí solo
recall, p99 de API, RSS de consulta o coste de un clúster.

Dependencias: toolchain y paquete memory con su grafo de imports/sistema;
MemTotal/MemUsed dependen del entorno. No se inspeccionó su implementación
para certificar uniformidad Windows/Linux ni se compiló el paquete.
El runner no invoca TiKV/TiFlash; no se presupone arranque de servicios.
Tampoco se afirma que su suite completa carezca de inicialización adicional.
Resultados publicados de SQL/HTAP no se atribuyen a estos microbenchmarks.
Se localizaron otros `bench_test.go` de executor/sort/chunk, no leídos como
runners: no se les atribuyen datasets o métricas.
Comparación futura: [protocolo](METHOD.md#protocolo-de-comparacion-de-benchmarks).
Sin diseñar benchmarks, thresholds, sampling ni configuración nueva.

**Defaults y denominadores no inventados:** `testing.B` gobierna `b.N`,
duración adaptativa y paralelismo efectivo; este archivo no fija `GOMAXPROCS`,
semilla, caché ni CLI del invocador. La unidad del reporte Go es operación del
loop; el archivo no calcula percentiles ni filas SQL/s. `MemTotal/MemUsed`
descartan sus errores: una medición futura debe registrar si había error para no
atribuir rapidez a un camino fallido. `BenchmarkConsume` no balancea consumo por
iteración, de modo que tampoco prueba steady state de reservas con RAII.
No hay preguntas vacías, abstenciones ni denominador de calidad aplicables;
errores del SO no se convierten en recall cero.

**Efectos:** sólo se leyeron los tres entry points; no se auditó inicialización
de toda la suite o cada syscall. No prometer runner libre de efectos globales
por no ver un servidor en estas 42 líneas. Descargas/toolchain pertenecen a
una eventual ejecución manual autorizada, no a esta autoría.

**Adaptación concreta:** A mapearía operación/hijo/padre a scope y reserva del
gobernador existente; comparar coste contable separado de reservar/liberar buffers
reales. B/C requieren casos funcionales de cancelación/vida, no atribuir sus
garantías a `BenchmarkConsume`. Reusar pista motor de METHOD y evaluadores
existentes para retrieval/memoria/QA. No traducir `testing.B` a un harness nuevo,
ni incluir cluster SQL/TiKV para medir una primitiva local. Modelo, tokenizer,
embedding, split/holdout y juez son N/A en estos microbenchmarks, no datos omitidos
que haya que inventar. Claims HTAP quedan separados de esta población.

## 8. Matriz frente al estado existente

| Capacidad | Estado Vanta | Delta condicionado |
|---|---|---|
| Gobernador y contabilidad de memoria | Existe en base V1 | A amplía atribución, no crea gobernador |
| Ejecución/materialización de consultas | Existe V2 | Revisar ownership de buffers |
| Worker/scheduler/checkpoints | Existe V3/propietario | B se integra, no duplica infraestructura |
| ANN/BM25/sparse/RRF | Existe | No migrar planner SQL para obtener retrieval |
| Provenance/sesiones/dedup/supersession | Existe | Conservar identidad al reintentar |
| Recibo jerárquico de bytes por operación | No acreditado aquí | Inventario antes de A |
| Cancelación tipada uniforme de todos los hosts | Parcial: seams, no trazado actual | Verificar contratos antes de B |
| Registro owner-aware de callbacks por operación | No acreditado | C sólo si se requiere registrar acciones; no bug certificado |
| Cuotas namespace | Existe como configuración | `config.rs` según referencia; enforcement completo no auditado |
| C13 receipt de mutación universal | No acreditado | Diseño propuesto, no sustituir con estado de cancelación |
| Consenso y HTAP distribuido como requisito local | No aplicable | Excluido expresamente |

## 9. Carencias sugeridas, no bugs inventados

Comprobar si el operador puede atribuir presión a una operación y conocer
por qué terminó es más útil que trasladar infraestructura distribuida.
No se demuestra que Vanta carezca de límites, cancelación o trazas.
Una mejora de receipts y ownership puede ser suficiente.
Prioridad condicionada a integración actual y coste de instrumentación.
No se abren tareas ni se promete menor latencia por semejanza arquitectónica.

**Hallazgo propio sustentado:** en `src/memory_governor.rs:56-68` [V1],
`allocated` y `freed` son `fetch_add/fetch_sub` de `AtomicU64`, no guards RAII.
Ese seam exige balance externo y no rechaza por sí mismo liberar más de lo
contabilizado; underflow sería riesgo si un caller viola la disciplina, no una
fuga/RSS incorrecto ya reproducido. A añade propiedad/atribución sin reemplazar
watermarks. No inferir falta de cuotas desde la ausencia de scope en ese struct.

**Otro hecho acotado:** `utils/checkpoint.rs:112-117` [VCP] hace RMW separado.
No usarlo para afirmar pérdida de toda escritura ni acoplarlo a A/B/C si esas
unidades no mutan ese checkpoint. Caller locks y un interleaving discriminante
siguen pendientes. MCP ya contiene `CancellationRegistry` en `server.rs:298`
(símbolo localizado con CodeGraph); su presencia descarta «no existe cancelación»,
no certifica propagación uniforme a cada SDK. No se inspeccionó todo el registro.

## 10. UI y escritorio

**Código local T8:** HTTP status/metrics, ruta `/web/trace`, servido de
estáticos de una dependencia y endpoints pprof. Es evidencia de diagnóstico,
no de una aplicación de escritorio ni de todo TiDB Dashboard.
No se inspeccionó un frontend Dashboard completo o producto cloud.
No se atribuye su disponibilidad al submódulo enterprise omitido.
**Producto externo documentado:** [TiDB Dashboard Introduction](https://docs.pingcap.com/tidb/stable/dashboard-intro/),
leído 2026-11-07, lo define como Web UI integrado en **PD**, también desplegable
como Pod separado en condiciones descritas allí. Flujo real documentado:
overview→SQL Statements/Slow Queries→detalle de ejecución; Resource Manager y
profiles/logs son superficies operacionales, no memoria conversacional.
No presentarlo como desktop ni como frontend incluido en `pkg/server`.
Intento REST de resolver SHA de `pingcap/tidb-dashboard` recibió rate-limit;
no se leyó frontend externo ni su licencia material en esta pasada.
La implementación acreditada aquí sigue siendo servidor HTTP/trace [T8];
el resto es documentación oficial fechada, no código fijado a SHA.

Studio Tauri/React ya existe en `develop-desktop`; A/B serían datos por
operación en sus inspectores, no una propuesta de otro dashboard.
Cuerpos del SHA `0ad38c542b39a589035b0e17fdedb4c09dca24b9` releídos con
`git show`:
[`feature-freez/desktop/package.json:1-56`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/package.json#L1-L56)
y [`feature-freez/desktop/src-tauri/tauri.conf.json:1-38`](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src-tauri/tauri.conf.json#L1-L38).
No se atribuyen al pin los bytes actuales movidos/modificados de `desktop/...`.
Extender inspector de recursos/cancelación con estimado/peak/cause y alcance,
sin SQL/payload por defecto ni botón kill global. Es propuesta de datos para
Studio, no implementación UI aprobada ni importación de Dashboard.
Runtime de esta investigación: **ninguno**; no iniciar/reiniciar clúster.

## 11. Cobertura y cierre

Diez ejes desarrollados más identidad/TOC; no se declaran lectura exhaustiva,
paridad ni adopción aprobada. Se preservan A/B y benchmark originales. A/B/C son
independientes salvo compartir identidad y ownership; A facilita atribuir bytes
de C pero no le añade autorización. C13 se usa sólo para mutaciones durables.
Cobertura: session/driver, tracker/killer, TopN registration, benchmark de
contabilidad, issue completo/comentario y docs Dashboard. No leídos integralmente:
Close de todos los operadores, red TiKV, recovery/fsync remoto, planner, seguridad
SQL, enterprise, todos los allocators y frontend Dashboard.

Pendientes manuales: contratos de bindings y cancelación, balances reales de
reservas, callbacks concurrentes y comparabilidad de medición; no se ejecutó
ninguno. Revisión adversarial independiente queda para el responsable.
**Split plan (>300 líneas):** al convertir en specs aprobadas, separar recursos
(A/C), cancelación (B), evidencia SQL/persistencia y anexo de benchmark/UI;
mantener dossier como fuente. No crear archivos nuevos ahora.

[T0]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/README.md#L16-L59
[TG]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/go.mod#L1-L35
[T1]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/session/session.go#L1947-L1968
[T2]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/session/session.go#L2664-L2692
[T3]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/tracker.go#L333-L384
[T3S]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/tracker.go#L78-L110
[T4]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/tracker.go#L450-L609
[T5]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/sqlkiller/sqlkiller.go#L30-L202
[T6]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/sqlkiller/sqlkiller.go#L204-L268
[T7]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/bench_test.go#L1-L42
[T8]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/server/http_status.go#L402-L425
[TE]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/server/http_status.go#L77-L95
[TS]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/server/http_status.go#L283-L288
[TL]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/LICENSE#L1-L30
[TD]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/store/driver/txn/txn_driver.go#L115-L218
[TC]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/session/session.go#L993-L1060
[TPRE]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/session/session.go#L540-L601
[TCHAIN]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/util/memory/tracker.go#L265-L312
[TTOP]: https://github.com/pingcap/tidb/blob/41efd866c3801a637ab2d6712b86d8b2cd77ac5b/pkg/executor/sortexec/topn.go#L109-L161
[TI]: https://github.com/pingcap/tidb/issues/72234
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/memory_governor.rs#L1-L133
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/executor.rs#L1-L65
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/mod.rs#L1-L16
[VCP]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L112-L117
