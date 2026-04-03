# Axiomas de Seguridad y Panic State

## 1. Objetivo
Convertir ConnectomeDB en una base de datos blindada que no acepte mutaciones contradictorias sobre nodos y vectores, protegiendo la integridad transaccional por encima del uptime.

## 2. Los Axiomas de Hierro

Son aserciones puras (asserts estructurados) que se ejecutan automáticamente en el Storage Engine durante un Commit WAL profundo:

### Axioma 1: Consistencia Topológica (No Huérfanos)
- Todo `Edge` (Synapse) en ConnectomeDB debe tener un `target_id` que EXISTA físicamente en la base de datos al momento de la escritura.
- Si el nodo destino es borrado, todos sus Edges entrantes deben ser limpiados automáticamente en cascada (Cascade Delete Fuerte).

### Axioma 2: No-Contradicción
- Un vector embbedding de una misma `Neuron` no puede ser diferente a la metadata codificada en la dimensión transaccional si el LLM emite una alerta geométrica incongruente (Se diferirá a fase posterior, pero las reglas base existen preparadas).

## 3. The Panic State Mode

Cuando la base de datos detecta una inconsistencia crítica (Ej: Un nodo falló al serializarse luego del WAL, o un axioma fue violado).

```rust
// Pseudo lógica recomendada en src/engine.rs / panic_state.rs
pub fn trigger_panic_state(&self, reason: &str) -> ! {
    // 1. Detener entrada de todas las mutaciones IQL.
    // 2. Hacer fsync(WAL) de todos los buffers vivos.
    // 3. Imprimir el Trace Completo de Corrupción.
    // 4. std::process::exit(1) / panic para salvaguardar el estado físico.
}
```

**Motivo:** Una caída controlada (Fail-stop proxy) es miles de veces preferible a seguir escribiendo basura en el subsuelo binario corrompiendo el archivo `.db`.
