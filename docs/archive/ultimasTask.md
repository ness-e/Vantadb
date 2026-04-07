# Phase 31 Execution Tracker — v0.5.0

- [x] **Hito 1: Representación Vectorial Escalonada & FWHT**
  - [x] Reemplazar `VectorData` por `VectorRepresentations` (Binary, Turbo, Full, None).
  - [x] Añadir `epoch: u32` a `UnifiedNode`.
  - [x] Añadir `NodeFlags::HALLUCINATION`.
  - [x] Implementar FWHT (SIMD + scalar fallback) en `src/vector/transform.rs`.

- [x] **Hito 2: Algoritmos de Cuantización (RaBitQ y PolarQuant)**
  - [x] Implementar RaBitQ (1-bit cuantizador) en `src/vector/quantization.rs`.
  - [x] Implementar PolarQuant (3-bit custom packer) en `src/vector/quantization.rs`.
  - [x] Estructurar `calculate_similarity` paramétrico en `src/index.rs`.
  - [x] Crear `MmapIndexBackend` en `src/index.rs`.
  - [x] Migrar callers: `executor.rs`, `storage.rs`, `api/mcp.rs`.

- [x] **Hito 3: Autodiscovery Hardware**
  - [x] Implementar firma de entorno (`env_hash`) en `src/hardware/mod.rs`.
  - [x] Guardar estado estático en `.connectome_profile` (serde_json).
  - [x] Invalidación automática por cambio de hardware.

- [x] **Hito 4: Ejecución Cognitiva e Invalidaciones**
  - [x] Crear `InvalidationDispatcher` con bus MPSC en `src/governance/invalidations.rs`.
  - [x] Definir tipos de evento: `PremiseInvalidated`, `HallucinationPurged`, `EnvironmentDrift`.
  - [x] Conectar `SleepWorker` con `invalidation_tx` sender.
  - [x] Implementar Backpressure por perfil de hardware (Enterprise: 5000, Performance: 500, Survival: 50).
  - [x] Implementar purga reactiva de nodos HALLUCINATION con emisión de eventos.
  - [x] Crear `invalidation_listener` consumer task.
  - [x] Bootstrapear dispatcher en `connectome-server.rs`.
