---
title: "TiDB: atribución de recursos y cancelación con causa"
kind: research
status: proposed
description: "Análisis estático de mecanismos locales de TiDB aplicables a un motor embebido sin trasladar su arquitectura distribuida"
tags: [research, memory, engineering]
---

# TiDB: atribución de recursos y cancelación con causa

## 1. Recomendación y arquitectura

**Dictamen:** estudiar atribución de memoria y terminación de operaciones,
no migrar consenso, TiKV/TiFlash, SQL completo ni despliegue de clúster.
Clone: `C:\Users\Eros\tidb`.
SHA congelado: `41efd866c3801a637ab2d6712b86d8b2cd77ac5b`.
Snapshot: **40 715 estrellas, Apache-2.0**, del [inventario](github-candidates.json).
Preparación y submódulo enterprise omitido: [manifiesto](clone-manifest.json).
No hubo consultas de red ni actualización del checkout.

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

**Vanta:** base histórica `c561158253bb88192590fa406c61e0a27dd7df8d`;
HEAD concurrente comunicado por propietario `167435a9e`.
Se conserva el [baseline](METHOD.md#vantadb-como-referencia) y sus capacidades:
ANN/BM25/sparse/RRF, provenance/sesiones/supersession/dedup,
assembler/compaction, scheduler/checkpoints/ingestors/auditoría y Studio.
No se audita de nuevo el motor ni se convierten hallazgos antiguos en actuales.

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
diagnóstico → información de consultas. Sin issues, reproducciones ni CVEs.
Nada de lo anterior constituye hallazgo vigente de Vanta.

## 4. Prerrequisitos Vanta

1. Revalidar estado actual de gobernador, tokens, receipts y reportes.
2. Diferenciar presupuesto de contexto en tokens de memoria operativa en bytes.
3. Identificar recursos compartidos con cachés y duración de resultados SDK.
4. Mantener locks/checkpoints existentes y no duplicar worker/scheduler.
5. Revisar errores a través de bindings antes de ampliar causas públicas.
6. Acordar garantías de cancelación frente a commit/WAL por backend.
7. Mantener diagnóstico por namespace y sin payloads/credenciales en logs.

## 5. Qué no trasladar

No portar Raft, 2PC, TiKV/TiFlash, PD o el plano de control de TiDB.
No introducir servidor Go o MySQL para añadir dos contratos locales.
No adoptar panic/kill global como política del SDK embebido.
No trasladar watermarks, cuotas ni algoritmos de arbitraje sin necesidad.
No confundir contabilidad declarada con medición física de memoria.
No requerir una consulta SQL para observar el estado de una operación local.

## 6. Método, licencia y cobertura

Análisis estático offline: archivos/rangos T0-T8 y seams V1-V3.
No instalación, scripts upstream, builds, tests, benchmarks, servicios o red.
No commits ni push; clones no modificados.
Apache-2.0 del inventario concuerda con LICENSE y cabeceras inspeccionadas.
Conservar licencia/avisos y revisar patentes/NOTICE/material concreto antes de
copiar; no se certifica cada dependencia del amplio `go.mod`.

Cobertura omitida: consenso, protocolos, optimizer completo, storage remoto,
acciones de spill, resource manager completo, SQL security, callbacks reales
y convergencia entre nodos. No se leyó todo TiDB.
El manifiesto omite `pkg/extension/enterprise`; no atribuirle comportamiento.
Historial shallow, ramas/tags y LFS no materializados según preparación.
Los seams Vanta se leyeron en base histórica, no como auditoría de concurrencia.

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
| Consenso y HTAP distribuido como requisito local | No aplicable | Excluido expresamente |

## 9. Carencias sugeridas, no bugs inventados

Comprobar si el operador puede atribuir presión a una operación y conocer
por qué terminó es más útil que trasladar infraestructura distribuida.
No se demuestra que Vanta carezca de límites, cancelación o trazas.
Una mejora de receipts y ownership puede ser suficiente.
Prioridad condicionada a integración actual y coste de instrumentación.
No se abren tareas ni se promete menor latencia por semejanza arquitectónica.

## 10. UI y escritorio

**Código local T8:** HTTP status/metrics, ruta `/web/trace`, servido de
estáticos de una dependencia y endpoints pprof. Es evidencia de diagnóstico,
no de una aplicación de escritorio ni de todo TiDB Dashboard.
No se inspeccionó un frontend Dashboard completo o producto cloud.
No se atribuye su disponibilidad al submódulo enterprise omitido.
Studio Tauri/React ya existe en `develop-desktop`; A/B serían datos por
operación en sus inspectores, no una propuesta de otro dashboard.
Runtime de esta investigación: **ninguno**; no iniciar/reiniciar clúster.

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
[V1]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/memory_governor.rs#L1-L75
[V2]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/executor.rs#L1-L65
[V3]: https://github.com/ness-e/VantaDB/blob/c561158253bb88192590fa406c61e0a27dd7df8d/vanta-memory/src/services/mod.rs#L1-L17
