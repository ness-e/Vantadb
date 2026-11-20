---
title: "Backlog de extracción — TiDB"
kind: research
status: active
description: "Tres candidatas del informe TiDB (recibo de recursos por operación, cancelación con causa y registro de acciones owner-aware) como filas del esquema canónico de 10 columnas"
tags: [extraccion, backlog, tidb]
---

# Backlog de extracción — TiDB

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/tidb.md`.
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|----|-----------|----------|---------------|----------|-----------|--------|-------------|------------|--------------|
| `X-TID-01` | 🟡 | Recibo de recursos por operación con guard de propiedad única | `src/memory_governor.rs:56-68` · `src/executor.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: scope de contabilidad por operación con reserva no clonable cuya propiedad se transfiere explícitamente a resultados/caché que la sobreviven; el receipt reporta estimado/pico/retención etiquetado como estimación, nunca RSS exacto. · Por qué: un límite global no explica qué operación conservó o liberó recursos, y los contadores `allocated`/`freed` atómicos exigen balance externo sin guard que lo haga a prueba de errores. · Acción mínima: inventariar la contabilidad actual, distinguir bytes reservados/capacidad retenida/uso estimado y representar propiedad con RAII y transferencia única. · Trazabilidad: informe §2/Candidata A · evidencia `pkg/util/memory/tracker.go:78-110`, `pkg/util/memory/tracker.go:333-384`, `pkg/util/memory/tracker.go:450-609` (clon `C:\Users\Eros\tidb`). | X-TID-02 · X-TID-03 | Propietario de buffers, convenciones de tamaños, scope estable y errores del gobernador; sin cambiar watermarks ni eviction |
| `X-TID-02` | 🟠 | Cancelación por operación con causa estable y outcome incierto | `src/executor.rs` · `vanta-memory/src/services/mod.rs` | 🟡 2-5d | 🟠 P1 | ⬜ Candidata | Qué: señal monotónica ligada a identidad de operación no reutilizable con causa tipada y estado de terminación que separa cancelación observada de resultado durable conocido/desconocido. · Por qué: distinguir timeout, cancelación explícita y presión evita cancelar la operación nueva que reutilizó el mismo canal/sesión; cancelar tras una escritura no autoriza a afirmar rollback. · Acción mínima: localizar tokens/contextos y causas actuales, ligar la notificación a identidad/generación y mapear causas a errores conservando el primer motivo válido. · Trazabilidad: informe §2/Candidata B · evidencia `pkg/util/sqlkiller/sqlkiller.go:30-202`, `pkg/util/sqlkiller/sqlkiller.go:204-268`. | X-TID-01 · X-TID-03 | Identidad de operación, ownership de tareas/callbacks y semántica de commit del backend; preferir extender `CancellationRegistry` de MCP si cubre transporte |
| `X-TID-03` | 🟡 | Registro de acciones desregistrable ligado a vida del propietario | `src/memory_governor.rs` | 🟡 2-5d | 🟡 P2 | ⬜ Candidata | Qué: registro de callbacks por operación con guard no clonable cuyo `Drop` desregistra la identidad exacta; la ejecución toma el propietario vigente sin sostener el lock global ni retener buffers de intentos anteriores. · Por qué: reaperturas de operadores en una sesión larga no deben retener scaffolding de intentos anteriores ni invocar closures de operaciones terminadas (mecanismo del issue pingcap/tidb#72234). · Acción mínima: inventariar callbacks actuales, especificar la relación registro→owner y cubrir apertura fallida, reapertura y cierre concurrente; desregistro exacto por identidad/generación. · Trazabilidad: informe §2/Candidata C · evidencia `pkg/util/memory/tracker.go:265-312`, `pkg/executor/sortexec/topn.go:109-161`, issue pingcap/tidb#72234. | X-TID-01 | Lifecycle de acciones definido sólo cuando existan registros por operación; aplicable sin X-TID-02; no exige cadena equivalente ya acreditada |

## Fichas completas

> Expansión de cada candidata a partir del informe `agent-memory-2026-11-07/tidb.md` (Candidatas A–C). Todos los contratos son **hipotéticos**: no describen APIs existentes de VantaDB. La evidencia `archivo:líneas` cita el clone upstream fijado en `41efd86` (`C:\Users\Eros\tidb`); las rutas VantaDB citan el snapshot del informe (`97c9623`) y requieren revalidación contra HEAD. TiDB es un servidor Go/TiKV: sus memory trackers y SQL killer son mecanismos de ejecución, no memoria de agente.

### X-TID-01 — Recibo de recursos por operación con guard de propiedad única

- **Problema:** un límite global no explica qué operación conservó o liberó recursos; contadores `allocated`/`freed` atómicos exigen balance externo sin un guard a prueba de errores.
- **Contrato hipotético:** `OperationBudgetRef {operation_id, parent_scope}`, `TrackedReservation` no clonable y `ResourceReceipt {estimated, peak, scope}`; entrada: scope/operación y reserva estimada; errores: límite, overflow, scope cerrado, transferencia inválida; nunca `panic` a través de FFI.
- **Fuente de evidencia (clone):** `pkg/util/memory/tracker.go:78-110` (jerarquía de trackers), `pkg/util/memory/tracker.go:333-384` (attach/detach y estado), `pkg/util/memory/tracker.go:450-609` (reversal, consumo y acciones de límites).
- **Seam VantaDB:** `src/memory_governor.rs:56-68` (`MemoryGovernor` con `used_bytes` y `allocated`/`freed`), `src/executor.rs` (resultados materializados).
- **Acciones:**
  1. Definir el guard de reserva no clonable ligado a `operation_id`/scope.
  2. Instrumentar las rutas de materialización existentes para adquirir/liberar por el guard.
  3. Exponer `ResourceReceipt` con estimado/pico/scope etiquetados.
  4. Cubrir error, doble free y transferencia; rollback: diagnóstico desactivable sin cambiar resultados.
- **Dependencias:** propietario de buffers, convenciones de tamaños, scope estable, sincronización actual y errores del gobernador; X-TID-02 y X-TID-03 se apoyan en la identidad de operación; sin cambiar watermarks ni eviction.
- **FMEA:** double free contable (tipo no clonable + transferencia única); liberación prematura (la reserva sigue al almacenamiento superviviente); doble cargo padre/caché (declarar identidad y propietario de capacidad); exceso antes de reserva (no prometer límite duro si sólo es medición).
- **Criterios de aceptación propuestos:** alta/baja equilibradas incluso al devolver error; padre y hoja no duplican totales; la estimación queda etiquetada como tal; desactivar el diagnóstico no altera resultados funcionales ni permisos.
- **Criterio negativo:** una suma cooperativa nunca se presenta como RSS exacto; ningún guard copiable puede liberar dos veces.
- **Independencia:** útil con operaciones no cancelables, sin X-TID-02.
- **No objetivos:** portar el arbitrador global, finalizadores Go ni nuevos umbrales.

### X-TID-02 — Cancelación por operación con causa estable y outcome incierto

- **Problema:** distinguir timeout, cancelación explícita y presión sin cancelar la operación nueva que reutilizó el mismo canal/sesión; cancelar tras una escritura no autoriza a afirmar rollback.
- **Contrato hipotético:** `OperationCancellation {operation_id, cause}` y `CompletionState {completed, cancelled, write_outcome_unknown}`; entrada: identidad no reutilizable, autorización y causa; salida: solicitud aceptada/ya finalizada separada del resultado efectivo; errores: operación desconocida, owner/epoch obsoleto, no autorizado, storage, outcome incierto.
- **Fuente de evidencia (clone):** `pkg/util/sqlkiller/sqlkiller.go:30-202` (causas, evento, CAS y callback protegido), `pkg/util/sqlkiller/sqlkiller.go:204-268` (observación, conexión y principio de reset).
- **Seam VantaDB:** `src/executor.rs`, `vanta-memory/src/services/mod.rs` (worker por sesión), `src/memory_governor.rs`; preferir extender el `CancellationRegistry` de MCP si cubre el transporte — no convertirlo en scheduler global ni asumir cobertura del SDK embebido.
- **Acciones:**
  1. Instrumentar una ruta existente con identidad de operación no reutilizable.
  2. Propagar la señal por los bindings existentes.
  3. Comprobar el cleanup en tests adversos.
  4. Ampliar cobertura a otras rutas. Rollback: desactivar la observación nueva sin tocar resultados.
- **Dependencias:** identidad de operación, ciclo de vida, ownership de tareas/callbacks y semántica de commit del backend; X-TID-01, X-TID-03.
- **FMEA:** señal tardía cancela la siguiente consulta (ligar a identidad/generación); callback superviviente cierra recurso nuevo (desregistrar bajo la misma frontera de ownership); cancelación tras escritura interpretada como rollback (separar cancelación observada de resultado durable conocido/desconocido); loop no cooperativo (no prometer preemption ni liberación instantánea).
- **Criterios de aceptación propuestos:** la señal de una operación finalizada no afecta a su sucesora; la causa reportada y el cleanup corresponden a la operación correcta; no anunciar rollback sin evidencia del backend; preservar idempotencia/checkpoints al reintentar. Adicional del informe: dos operaciones de una misma sesión no comparten señal; error primario y de cleanup no se sobrescriben; el receipt conserva conocimiento parcial tras timeout.
- **Criterio negativo:** «cancelado» nunca significa automáticamente «no escrito»; una señal tardía nunca cancela la operación sucesora.
- **Independencia:** funciona por cancelación del usuario sin X-TID-01.
- **No objetivos:** kill de procesos, MySQL wire, contexto Go ni cuotas de clúster.

### X-TID-03 — Registro de acciones desregistrable ligado a vida del propietario

- **Problema:** las reaperturas de operadores en una sesión larga no deben retener scaffolding de intentos anteriores ni invocar closures de operaciones terminadas (mecanismo del issue pingcap/tidb#72234).
- **Contrato hipotético:** `register_action(operation_id, owner_epoch, action_ref) -> Result<ActionRegistration, RegistrationError>`; guard no clonable cuyo `Drop` desregistra la identidad exacta; registry con weak refs o referencias de vida acotada; la ejecución toma el propietario vigente sin sostener el lock global. Errores `ClosedScope | StaleOwner | DuplicateRegistration | Unauthorized`; el callback nunca causa pánico que atraviese FFI.
- **Fuente de evidencia (clone):** `pkg/util/memory/tracker.go:265-312` (`reArrangeFallback`: cadenas por prioridad; `UnbindActionFromHardLimit` retira la acción), `pkg/executor/sortexec/topn.go:109-161` (`TopNExec.OpenSelf` registra la acción con spillHelper), issue pingcap/tidb#72234 (retención por reaperturas — reporte, no reproducción propia).
- **Seam VantaDB:** `src/memory_governor.rs` (`MemoryGovernor` sólo acredita contadores/watermarks; esta ficha instrumenta el ciclo de vida de callbacks, no un DTO duplicado de X-TID-01).
- **Acciones:**
  1. Inventariar callbacks actuales registrados por operación.
  2. Especificar la relación registro→owner con epoch.
  3. Integrar con el gobernador y la finalización existentes.
  4. Cubrir apertura fallida, reapertura y cierre concurrente.
  5. Exponer sólo el número de registros vivos, sin payloads.
- **Dependencias:** lifecycle de acciones definido sólo cuando existan registros por operación; aplicable sin X-TID-02; X-TID-01 permite atribuir la retención pero esta ficha no la exige.
- **FMEA:** callback captura `Arc` de buffer → contador bajo y RSS alto (contrastar recursos vivos con registros → weak/guard); cierre compite con callback → use-after-close lógico (validar epoch, tomar owner vigente); reentrada bajo mutex → deadlock (ejecutar fuera del lock).
- **Criterios de aceptación propuestos:** repetir open/close deja sólo acciones de owners vivos; callbacks viejos no actúan sobre la nueva operación; un error de apertura no deja registro; cancelar mientras se ejecuta no libera un recurso todavía usado. No reportar RSS exacto desde un contador cooperativo.
- **Criterio negativo:** un guard nunca desregistra otra generación; cancelar nunca finge que los buffers aún retenidos fueron liberados.
- **Independencia:** una familia de callbacks; aplicable sólo si se añaden/usarán acciones por operación — no se demostró una cadena equivalente defectuosa en Vanta.
- **No objetivos:** portar spill SQL ni el arbitrador Go; no lista global de closures fuertes con vida de sesión; no guardar closures en disco. Abierto: un callback concurrente ya iniciado se drena o finaliza cooperativamente según el contrato del owner, nunca se mata el proceso.
