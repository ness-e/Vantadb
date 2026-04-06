# Fase 31: Hybrid Quantization & Reactive Invalidation

> **Estado:** 🔲 PENDIENTE
> **Versión Objetivo:** v0.5.0
> **Prerequisito:** Fase 30 (Memory Rehydration) ✅

## Concepto
Implementación de un sistema de cuantización de tres niveles (1-bit, 3-bit, FP32) con un protocolo de corrección de premisas asíncrono y gestión de backpressure cognitivo basado en la salud del hardware.
Esta arquitectura resuelve el "Muro de Memoria" en hardware edge, rotando características mediante FWHT y permitiendo que la compresión interactúe simbióticamente con el Devil's Advocate para proteger la pureza axiomática sin asfixiar los ciclos IO.

## Componentes Core

### 1. Sistema de Representación Vectorial (src/node.rs)
Desacoplamiento de la memoria de los nodos para manejar niveles de fidelidad:
```rust
pub enum VectorRepresentations {
    Binary(Box<[u64]>),  // L1: HNSW RAM (Hamming) - Rápido, RAM <100 bytes / vector
    Turbo(Box<[u8]>),    // L2: MMap Re-ranking (3-bit PolarQuant) - SSD local
    Full(Vec<f32>),      // L3: RocksDB Archaeology (FP32) - Alta latencia
}
```

### 2. Motor de Rotación FWHT (src/vector/transform.rs)
Transformada Rápida de Walsh-Hadamard para distribuir la varianza de los componentes vectoriales antes de la cuantización, mitigando el error de redondeo binario.
- **Fast Path:** Implementación SIMD usando `wide::f32x8`.
- **Fallback:** Escalar para hardware sin soporte AVX.

### 3. Protocolo de Invalidez Reactiva (`src/governance/`)
- **Event Dispatcher (`InvalidationDispatcher`):** Emite `PREMISE_INVALIDATED` cuando el nivel L3 (FP32) contradice una inferencia previa de baja fidelidad (L2).
- **Epoch Versioning:** Cada nodo tiene un `u32 epoch` que se incrementa en colapsos de incertidumbre, marcando las alucinaciones erróneas pasadas como `INVALID` en el `shadow_kernel`.

### 4. Modos de Certeza y Backpressure Cognitivo
Con el objetivo de garantizar reactividad biológica incluso bajo estrés físico del SSD:
- **STRICT:** Bloquea I/O hasta validación total L3. El sistema lo **degrada** automáticamente a BALANCED si la latencia supera el backpressure threshold de autoprotección.
- **BALANCED (Default):** Re-ranking 3-bit L2 inmediato en MMap. Si la lectura demora demasiado (`io_budget_ms`), el iterador activa un **"Threshold de Abandono"**, respondiendo con `TrustVerdict::LowConfidence` e iniciando validación asíncrona.
- **FAST:** Solo evalúa L1 (1-bit / XOR + POPCNT). Máxima velocidad de interfaz, cero validación axiomática, asumiendo riesgo.

### 5. Configuración Autodiscovery & Recalibración
Hardware detectado en la instanciación de red:
- **Survival:** < 8GB RAM. Uso agresivo de MMap, backpressure muy sensible (50ms).
- **Cognitive:** ~16GB RAM. Balanceado (150ms).
- **Enlightened:** > 32GB RAM. Desactiva degradación.
- **NeuLISP Comando:** `(RECALIBRATE-RESOURCES)` adaptará los budgets IO si se migra de hardware.

## Tareas de Implementación Inmediatas
1. Renombrar structs y dependencias (`VectorRepresentations`).
2. Implementar Transformada FWHT (`src/vector/transform.rs`).
3. Crear `mmap_backend` para el nivel de precisión intermedia (`Turbo`).
4. Integrar el `InvalidationDispatcher` en el `SleepWorker`.
