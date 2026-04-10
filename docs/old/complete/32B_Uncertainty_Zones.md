# Fase 32: Uncertainty Zones (Superposición Lógica)

> **Estado:** 🔲 PENDIENTE  
> **Versión Objetivo:** v0.5.0  
> **Prerequisito:** Fase 31 (Hybrid Quantization & Reactive Invalidation) ✅

---

## Concepto

Cuando el `DevilsAdvocate` detecta una contradicción entre nodos, ahora tiene un desencadenante mecánico primario: el **Pánico Axiomático de Cuantización** introducido en la Fase 31. Si la inferencia de re-ranking (Turbo 3-bit o Binary 1-bit) choca con los Axiomas de Hierro tras recuperar la fidelidad FP32 (L3), se asume ruido de compresión. En lugar de rechazar el dato o paralizar el sistema, el motor crea un `QuantumNeuron` que mantiene ambos candidatos en **superposición** hasta que un agente externo o un deadline temporal colapse el estado.

## Objetivo

Permir que ConnectomeDB maneje la incertidumbre como ciudadano de primera clase, mitigando las colisiones causadas por el muro de memoria (cuantización). Transforma las alucinaciones del hardware edge en objetos "superpuesto" que esperan resolución.

## Componentes Propuestos

### 1. `QuantumNeuron` (src/node.rs)
```rust
pub struct QuantumNeuron {
    pub id: u64,
    pub candidates: Vec<UnifiedNode>,
    pub collapse_deadline_ms: u64,
    pub created_at: u64,
}
```

### 2. Integración con `DevilsAdvocate` (src/governance/mod.rs)
- Nuevo veredicto: `TrustVerdict::Superposition(QuantumNeuron)`.
- En lugar de `Reject`, crear `QuantumNeuron` con ambos candidatos contradictorios.

### 3. Colapso Temporal (src/governance/sleep_worker.rs)
- El `SleepWorker` supervisa `QuantumNeuron` con deadlines vencidos.
- Al vencer: colapsa al candidato con mayor `TrustScore`.
- El perdedor se mueve a `shadow_kernel` como tombstone auditable.

### 4. Acceso desde IQL
- `FROM QuantumZone#ID` → retorna ambos candidatos con sus scores.
- `COLLAPSE QuantumZone#ID FAVOR candidate_index` → colapso manual.

## Archivos a Crear/Modificar
- `src/node.rs` — struct QuantumNeuron
- `src/governance/mod.rs` — TrustVerdict::Superposition
- `src/governance/sleep_worker.rs` — colapso temporal
- `src/executor.rs` — comandos COLLAPSE
- `tests/uncertainty_zones.rs`

## Métricas de Aceptación
- [ ] QuantumNeuron persiste y se recupera de RocksDB.
- [ ] DevilsAdvocate crea superposición en lugar de rechazar.
- [ ] SleepWorker colapsa automáticamente al vencer deadline.
- [ ] IQL permite inspección y colapso manual.
- [ ] Test verde: `tests/uncertainty_zones.rs`.
