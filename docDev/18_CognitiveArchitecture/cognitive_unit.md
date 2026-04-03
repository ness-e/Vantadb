# Arquitectura Cognitiva: El Trait `CognitiveUnit` y División Neuronal (ST/LT)

## 1. Objetivo
Convertir el antiguo `UnifiedNode` en una verdadera entidad con comportamiento dual. Introducimos una división formal de memoria (Corto y Largo Plazo) usando el trait `CognitiveUnit` para manejar ciclos de vida diferenciados en RAM y en el Graph.

## 2. El Trait `CognitiveUnit`
Todas las estructuras fundamentales del plano semántico deben implementar este Trait:

```rust
pub trait CognitiveUnit {
    fn trust_score(&self) -> f32;
    fn hits(&self) -> u32;
    fn last_accessed(&self) -> u64; // UNIX Timestamp
    fn pin(&mut self);
    fn unpin(&mut self);
    fn is_pinned(&self) -> bool;
}
```

## 3. División Neuronal
A nivel de código (en `src/node.rs`), el alias genérico debe envolver la nueva lógica transitoria usando un enum de dos caras.

```rust
pub enum NeuronType {
    STNeuron, // Memoria de Corto Plazo (Transitoria, rápida, no garantizada ACID en disco)
    LTNeuron, // Memoria de Largo Plazo (Persistente, con conexiones sinápticas ACID)
}

// Extensión al actual UnifiedNode
pub struct UnifiedNode {
    pub id: u64,
    pub neuron_type: NeuronType,
    // ... campos existentes (labels, props, vectors, edges)
    // + nuevos campos cognitivos
}
```

## 4. Lazy Loading (Carga Perezosa)
- Las `LTNeuron` residirán en RocksDB por defecto.
- Al ser consultadas, se cargan a RAM y se marcan como "Calientes" (Aumenta `hits`).
- Si la RAM supera el 85%, el sistema puede "De-instanciar" las `LTNeuron` más frías de la memoria volátil utilizando un barrido estocástico, conservándolas intactas en el Storage engine.
