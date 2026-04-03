# Fase 23: Soberanía Cognitiva & Gobernanza (Shadow Kernel)

## Meta
Implementar un sistema de auditoría proactiva que proteja la integridad semántica de la base de datos. En ConnectomeDB, las mutaciones no son simples escrituras en disco; son "decisiones cognitivas" que deben ser validadas contra el conocimiento preexistente.

## Componentes de Gobernanza (`src/governance/`)

### 1. El Abogado del Diablo (`DevilsAdvocate`)
Este módulo actúa como un filtro crítico durante las operaciones de `INSERT` y `UPDATE`.
- **Detección de Contradicciones**: Si se intenta insertar un nodo con una similitud vectorial muy alta (>0.95) a uno ya existente, pero con valores relacionales o etiquetas contradictorias, el sistema marca una alerta.
- **Evaluación de Trust**: Compara el `Trust Score` del "incumbente" (nodo existente) contra el "propuesto". Si el propuesto tiene un score significativamente menor, la mutación puede ser rechazada.

### 2. El Árbitro de Confianza (`TrustArbiter`)
Resuelve los conflictos identificados por el Abogado del Diablo.
- **ResolutionResult**:
    - `Accept`: La mutación es segura.
    - `Reject(reason)`: Se bloquea la escritura para preservar la integridad (Sovereignty Rejected).
    - `Shadow(id)`: La escritura se permite pero se marca para revisión manual o se desvía a una capa de almacenamiento de baja confianza.

## El Shadow Kernel (Núcleo en la Sombra)

### Borrados Atómicos y Lápidas (Tombstones)
ConnectomeDB no utiliza borrados físicos inmediatos (`hard-delete`). En su lugar, implementa un sistema de **Lápidas Auditables**:
- Al borrar un nodo, se mueve al **Shadow Archive**.
- Se deja una "lápida" (tombstone) con metadatos sobre por qué y quién realizó el borrado.
- Esto permite la prevención de pérdida semántica y auditorías post-mortem.

### Garbage Collection (GC) Asíncrono (`src/gc.rs`)
Un worker en segundo plano se encarga de la purga física de datos basados en políticas de retención (TTL) y el estado de las lápidas, liberando espacio en RocksDB sin comprometer la latencia de las queries activas.

## Axiomas de Seguridad (v0.4.0)
- **Topological Consistency**: No se permiten relaciones hacia nodos ya "difuntos" (tombstoned).
- **Life Insurance**: Checkpoints automáticos basados en hard-links de RocksDB para recuperación instantánea ante fallos catastróficos.
