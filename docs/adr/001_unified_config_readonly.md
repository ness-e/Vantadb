# ADR 001: Arquitectura de Configuración Unificada y Barrera de Solo Lectura

## Estado

Estado: Aprobado

## Contexto

En las versiones MVP de VantaDB, las opciones de configuración estaban fragmentadas en múltiples interfaces mutables (por ejemplo, `VantaOpenOptions`), lo que permitía modificar parámetros estructurales del motor en caliente. Esta mutabilidad inconsistente causaba problemas de consistencia interna y riesgos de corrupción.
Además, las operaciones de escritura (como inserciones, actualizaciones y borrados) avanzaban profundamente en la canalización del motor de almacenamiento antes de validar el estado operativo de la base de datos (por ejemplo, si el motor fue abierto en modo `read_only`), incurriendo en costos innecesarios de CPU, asignación de memoria y contención de cerrojos (locks) antes de fallar.

## Decisión

1. **Consolidación Estructural:** Unificar todas las opciones de inicialización en una única estructura inmutable y con tipado fuerte llamada `VantaConfig`. Todos los puntos de entrada del SDK y del servidor consumen esta configuración consolidada en el momento de la apertura del motor (`open_with_config`).

2. **Barrera de Entrada Temprana (Fail-Fast):** Implementar la función de protección `guard_write_allowed(&self.config)` en la API pública de almacenamiento. Esta rutina evalúa inmediatamente el flag de protección de escritura:

   ```rust
   pub fn guard_write_allowed(config: &VantaConfig) -> Result<()> {
       if config.read_only {
           return Err(VantaError::ReadOnlyViolation);
       }
       Ok(())
   }
   ```

3. **Inyección en Mutadores:** Invocar esta barrera como la primera línea de ejecución en todos los métodos que alteren el estado lógico o físico del motor: `insert`, `put`, `delete`, `add_edge` y `flush`.

## Consecuencias

### Beneficios

* **Aislamiento Predictivo:** El motor aborta operaciones no autorizadas en tiempo $O(1)$ sin reservar locks de lectura/escritura ni abrir transacciones en los motores subyacentes (RocksDB, Fjall), previniendo la degradación de recursos.
* **Seguridad de Hilo (Thread Safety):** Al ser `VantaConfig` completamente inmutable tras la inicialización, se elimina cualquier posibilidad de data races o inconsistencias por reconfiguración en caliente en entornos altamente concurrentes.
* **Simplificación de la API:** Se remueven APIs redundantes y métodos deprecados, mejorando la mantenibilidad para clientes que integran VantaDB como motor embebido.

### Deuda Técnica / Costos

* **Re-inicialización Mandatoria:** Si un nodo requiere cambiar de estado operativo (de réplica de solo lectura a primario de lectura/escritura), la instancia del motor debe cerrarse y abrirse explícitamente con una nueva estructura `VantaConfig`. Esto se considera un comportamiento deseable en arquitecturas de sistemas distribuidos industriales.
