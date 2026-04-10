# Fase 35: mmap Neural Index (Survival Mode)

> **Estado:** 🔲 PENDIENTE  
> **Versión Objetivo:** v0.5.0  
> **Prerequisito:** Fase 34

---

## Concepto

Completar el pendiente de la Fase 24: permitir que el HNSW Neural Index opere sobre Memory-Mapped Files en lugar de RAM pura, habilitando búsquedas vectoriales en máquinas con recursos severos (< 8GB RAM).

## Objetivo

Eliminar la barrera de entrada de RAM para búsquedas vectoriales en dispositivos edge, IoT y laptops de desarrollo básicas.

## Componentes Propuestos

### 1. MMap Backend para HNSW (src/index.rs)
```rust
pub enum IndexBackend {
    InMemory(Vec<(u64, Vec<f32>)>),   // Actual
    MMapFile(memmap2::Mmap),           // Nuevo
}
```

### 2. Serialización del Índice a Disco
- Al cerrar el engine: serializar el HNSW a `data/neural_index.bin`.
- Al re-abrir: mmap del archivo evitando reconstrucción completa.
- Fallback: si el archivo no existe/está corrupto → rebuild clásico.

### 3. Activación Automática
- `SurvivalProfile` (RAM < 16GB): activar mmap automáticamente.
- `PerformanceProfile`: in-memory por defecto.
- Flag override: `CONNECTOME_INDEX_MMAP=true/false`.

### 4. Dependencia
```toml
[dependencies]
memmap2 = "0.9"
```

## Archivos a Crear/Modificar
- `src/index.rs` — IndexBackend enum + mmap logic
- `src/storage.rs` — serialización en shutdown
- `src/hardware/mod.rs` — activación por perfil
- `Cargo.toml` — dependencia memmap2
- `tests/mmap_index.rs`

## Métricas de Aceptación
- [ ] HNSW funciona sobre mmap en Survival Mode.
- [ ] Cold start omite rebuild si `neural_index.bin` existe.
- [ ] Fallback limpio si archivo corrupto.
- [ ] Test verde: `tests/mmap_index.rs`.
