# Fase 33: LTD Synaptic Depression (Edges)

> **Estado:** 🔲 PENDIENTE  
> **Versión Objetivo:** v0.5.0  
> **Prerequisito:** Fase 32

---

## Concepto

Implementar Long-Term Depression (LTD) biológica en las sinapsis del grafo. Los `Edge` que no son traversados decaen gradualmente en peso, emulando cómo el cerebro debilita conexiones neuronales no utilizadas.

## Objetivo

Mantener la integridad semántica del grafo eliminando automáticamente conexiones obsoletas, reduciendo el ruido en traversals y mejorando la calidad de las búsquedas de grafos.

## Componentes Propuestos

### 1. Campos Nuevos en `Edge` (src/node.rs)
```rust
pub struct Edge {
    pub target: u64,
    pub label: String,
    pub weight: f32,
    pub last_traversed_ms: u64,  // NUEVO
    pub traversal_count: u32,    // NUEVO
}
```

### 2. Tracking de Traversal (src/executor.rs)
- Cada vez que un `SIGUE` traversa un edge, incrementar `traversal_count` y actualizar `last_traversed_ms`.

### 3. Decaimiento Circadiano (src/governance/sleep_worker.rs)
- En fase REM: `edge.weight *= 0.95` para edges sin traversal en las últimas 24h.
- Si `edge.weight < 0.05` → remover edge y registrar tombstone auditable.

### 4. Protección de Edges Críticos
- Edges con `weight >= 0.9` y `traversal_count > 100` son inmunes al decaimiento (análogo al Amygdala Budget).

## Archivos a Crear/Modificar
- `src/node.rs` — campos nuevos en Edge
- `src/executor.rs` — tracking de traversal  
- `src/governance/sleep_worker.rs` — decaimiento REM
- `tests/synaptic_depression.rs`

## Métricas de Aceptación
- [ ] Edges no traversados decaen 5% por ciclo REM.
- [ ] Edges con weight < 0.05 se eliminan automáticamente.
- [ ] Edges de alta traversal están protegidos.
- [ ] Test verde: `tests/synaptic_depression.rs`.
