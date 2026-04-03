# Arquitectura Cognitiva: Puntuación Temporal e Integridad por "Pinning"

## 1. Objetivo
Evitar reglas mágicas de caché y usar propiedades biológicas (Frecuencia y Recencia) para dictar qué nodos viven en RAM y cuáles soportan la recolección de basura (Garbage Collection).

## 2. Puntuación Heurística LFU/LRU

Se incorporan dos campos al `UnifiedNode` / `Neuron`:
- `hits: u32`: Frecuencia de acceso.
- `last_accessed: u64`: UNIX Timestamp en milisegundos de recencia.

### Reglas de Decaimiento Cognitivo:
- Todo query `SELECT` o cruce de grafos (`RELATE`) que atraviesa el nodo incrementa su `hits` en `+1`.
- Actualiza su `last_accessed` a `SystemTime::now()` instantáneamente.
- **Batched Decay:** Cuando el Garbage Collector se active, en lugar de borrar la memoria, decrece el score: `hits = hits / 2` (Decaimiento a la mitad - "olvido bayesiano").

## 3. Pinning Absoluto (El Clavo de Hierro)

El framework incorpora la bandera `NodeFlags::PINNED` nativa.

```rust
// en src/storage.rs o src/node.rs
impl UnifiedNode {
    pub fn pin(&mut self) {
        self.flags.insert(NodeFlags::PINNED);
    }
}
```

### Reglas del Pinned:
1. Una `Neuron` Pinned **JAMÁS** será expulsada de RAM.
2. Una `Neuron` Pinned **JAMÁS** sufrirá el olvido bayesiano (su `hits` no disminuye en la fase de GC).
3. Una `Neuron` Pinned bloquea cualquier eliminación dura (trigger pasivo de `Panic State` si se fuerza un `DELETE`). Se usa para Core Contextos o Prompt Bases de LLM.
