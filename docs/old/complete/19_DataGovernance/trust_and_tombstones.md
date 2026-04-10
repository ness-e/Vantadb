# Gobernanza: Trust Score Estático y Tumbas Auditables

## 1. Objetivo
Brindar al ecosistema del motor semántico un índice de "Veracidad" inmutable frente a múltiples agentes de Inteligencia Artificial que operan en un mismo grafo, y no dejar rastros fantasma de operaciones nulas (Graveyard Accountability).

## 2. Campo Estático `Trust Score`

Toda `Neuron` recibe el score:
- `trust_score: f32`
- Valor por defecto: `0.5` (Confianza media / Incertidumbre).
- Rango soportado: `0.0` (Totalmente Falso/Repudiado) a `1.0` (Axioma Absoluto).

### Integración en Consultas IQL:
El motor debe permitir filtrar o penalizar matemáticamente basándose en esto:
`SELECT * FROM Cortex WHERE trust_score > 0.8`

## 3. Tombstones Auditables (Bajas)

Si la operación de Borrado (`DELETE`) ocurre pero no activamos el Shadow Archive por políticas de espacio (Storage Limit), debemos usar *Tombstones*.

Un Tombstone es un struct minimalista residente en disco:
```rust
pub struct AuditableTombstone {
    pub id: u64,
    pub timestamp_deleted: u64,
    pub reason: String, // Ejemplo: "Agent 'X' invoked logical TTL cleanup"
    pub original_node_hash: u64, // Referencia rápida
}
```

- Cuando NodeD muere, su slot primario se libera, pero en el índice de metadatos guardamos un `AuditableTombstone`.
- Si otro nodo busca el `ID` muerto forzando un join por error, el sistema no devuelve `NotFound` genérico, devuelve un `TombstoneHit(Reason)`, deteniendo de inmediato ciclos LLM ruidosos.
