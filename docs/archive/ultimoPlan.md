# Finalización de la Fase 32 (Hard-Urgency / NMI)

Tras el análisis de la base de código actual (`thalamic_gate.rs`, `sleep_worker.rs`, y `uncertainty.rs`), se ha validado que casi todos los componentes de la arquitectura descrita en **32_Hard_Urgency_NMI.md** ya existen.

- **Bloom Filter In-House:** `ThalamicGate` ya prescinde de librerías externas.
- **Estadísticas Atómicas y Decaimiento:** El `UncertaintyBuffer` lleva un conteo atómico y `SleepWorker` utiliza un ratio del `70%` de anomalías para acelerar colapsos (`_shrinks_deadline`).
- **Lógica de Colapso:** `force_collapse_nmi()` está diseñada y compilando.

## Missing Link (Falta)

Para completar definitivamente la Fase 32, falta **el activador (trigger)** del NMI. El `ResourceGovernor` (`src/governor.rs`) debe detectar cuándo el uso de memoria (cuota Penumbra/'allocations') excede el umbral crítico (>90%) y disparar el NMI. Actualmente el gobernador rechaza queries en el límite blando (OOM), pero no se ha enlazado con la función `force_collapse_nmi()` del buffer.

## User Review Required

> [!IMPORTANT]  
> Diseño de interconexión propuesto: Para evitar bloqueos cíclicos de referencia, propongo que `ResourceGovernor::request_allocation` reciba un callback opcional de emergencia (o se haga la llamada al NMI directamente en el entorno de ejecución del storage antes de requerir la memoria, u obteniendo una referencia atenuada al `UncertaintyBuffer`). ¿Prefieres pasar el `Arc<StorageEngine>` al `ResourceGovernor` al inyectarlo, o manejamos el activador directamente en la capa de inserción del `StorageEngine/Executor` (ej: interceptando el uso)?

## Proposed Changes

---

### `src/governor.rs`

#### [MODIFY] governor.rs
- Modificar `request_allocation_with_fallback` (nuevo método) o adaptar `request_allocation` para que devuelva un nuevo valor: `Ok(bool)` (donde `true` indica "presión > 90%, por favor llama al NMI"). Esto aísla el gobernador del storage evadiendo dependencias cruzadas y delegando el colapso a quien tiene acceso al buffer (el ejecutor).

### `src/executor.rs`

#### [MODIFY] executor.rs
- En las rutinas intensivas (o al añadir colisiones cuánticas), luego de la verificación del `ResourceGovernor`, si este reporta presión del > 90%, el Executor hará la invocación directa a `self.storage.uncertainty_buffer.force_collapse_nmi()`.

### `docDev/32_Hard_Urgency_NMI.md`

#### [MODIFY] 32_Hard_Urgency_NMI.md
- Cambiar Estado: `🔲 PENDIENTE` -> `✅ COMPLETADO`.

## Open Questions

Ninguna adicional por ahora. Al usar un patrón de *return flag* mantenemos desacoplado `ResourceGovernor` de `UncertaintyBuffer`, preservando el principio de "cero dependencias cruzadas innecesarias" del framework.

## Verification Plan

### Automated Tests
- Validar mediante el test suite existente `cargo test` para evitar daños de regresión en las interacciones de colapso normal.
- Implementar un pequeño test unitario en `tests/cognitive_sovereignty.rs` donde causemos colisiones superando el 90% (artificialmente) para validar el disparo de `force_collapse_nmi()`.
