---
type: glossary-entry
status: stable
tags: [vantadb, glosario, performance, hardware, optimización]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# SIMD (Single Instruction, Multiple Data)

## Definición

**SIMD** es una clase de instrucciones de CPU que permiten procesar múltiples datos con una sola instrucción, paralelizando operaciones vectoriales a nivel de hardware.

## Arquitecturas Soportadas

| Arquitectura | Conjunto de Instrucciones | Registros |
|--------------|---------------------------|-----------|
| **x86_64** | SSE, SSE2, AVX2, AVX-512 | XMM (128-bit), YMM (256-bit), ZMM (512-bit) |
| **ARM64** | NEON, SVE | Q (128-bit), Z (scalable) |
| **RISC-V** | V Extension | V (scalable) |

## Uso en VantaDB

### Cálculo de Distancia Coseno

```rust
use wide::f32x8;  // AVX2: 8 floats por ciclo

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = f32x8::ZERO;
    let mut norm_a = f32x8::ZERO;
    let mut norm_b = f32x8::ZERO;
    
    // Procesar 8 floats por iteración
    for i in (0..a.len()).step_by(8) {
        let va = f32x8::from_slice_unaligned(&a[i..]);
        let vb = f32x8::from_slice_unaligned(&b[i..]);
        
        dot += va * vb;       // 8 multiplicaciones en 1 ciclo
        norm_a += va * va;    // 8 multiplicaciones en 1 ciclo
        norm_b += vb * vb;    // 8 multiplicaciones en 1 ciclo
    }
    
    let dot = dot.reduce_add();
    let norm = (norm_a.reduce_add() * norm_b.reduce_add()).sqrt();
    dot / norm
}
```

### Comparación: Escalar vs SIMD

| Método | Operaciones/Ciclo | Speedup |
|--------|-------------------|---------|
| **Escalar** | 1 multiplicación | 1x (baseline) |
| **SSE (128-bit)** | 4 multiplicaciones | ~4x |
| **AVX2 (256-bit)** | 8 multiplicaciones | ~8x |
| **AVX-512 (512-bit)** | 16 multiplicaciones | ~16x |

## Impacto en Búsqueda Vectorial

### Latencia por Dimensión

| Dimensiones | Escalar | AVX2 | Speedup |
|-------------|---------|------|---------|
| 128 | 0.8 µs | 0.1 µs | 8x |
| 384 | 2.4 µs | 0.3 µs | 8x |
| 768 | 4.8 µs | 0.6 µs | 8x |
| 1536 | 9.6 µs | 1.2 µs | 8x |

### Throughput de Búsqueda (100K vectores, 128d)

| Configuración | Queries/segundo |
|---------------|-----------------|
| Sin SIMD | ~2,000 |
| Con AVX2 | ~16,000 |

## Detección en Runtime

```rust
use std::arch::is_x86_feature_detected;

fn compute_distance(a: &[f32], b: &[f32]) -> f32 {
    if is_x86_feature_detected!("avx2") {
        // Ruta AVX2
        unsafe { cosine_similarity_avx2(a, b) }
    } else if is_x86_feature_detected!("sse4.1") {
        // Ruta SSE4.1
        unsafe { cosine_similarity_sse(a, b) }
    } else {
        // Fallback escalar
        cosine_similarity_scalar(a, b)
    }
}
```

## Requisitos de Alineación

Para máximo rendimiento, los vectores deben estar alineados a 32 bytes (AVX2):

```rust
// Alineación óptima
#[repr(align(32))]
struct AlignedVector {
    data: Vec<f32>,
}

// Uso con mmap
let mmap = MmapOptions::new()
    .alignment(32)
    .map(&file)?;
```

## Véase También

- [HNSW](HNSW.md) — Índice vectorial que usa SIMD para distancias
- [Vector Similarity](Vector Similarity.md) — Métricas aceleradas por SIMD
- [Benchmarks](Benchmarks.md) — Métricas de performance con SIMD

---

*SIMD es fundamental para el rendimiento de VantaDB, permitiendo búsquedas vectoriales sub-milisegundo en datasets grandes.*

