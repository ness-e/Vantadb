# **Fase 31: Hybrid Quantization & Reactive Invalidation**

**Estado:** 🔲 PENDIENTE **Versión Objetivo:** v0.5.0 **Prerequisito:** Fase 30 (Memory Rehydration Protocol) ✅

## **1\. Concepto Arquitectónico**

Implementación de un sistema de cuantización de tres niveles (1-bit, 3-bit, FP32) diseñado para resolver el "Muro de Memoria" en hardware edge. El sistema intercala compresión extrema para búsquedas ultrarrápidas en RAM con un protocolo de corrección asíncrona (Reactive Invalidation) y un mecanismo de homeostasis (Cognitive Backpressure) que ajusta la fidelidad operativa basándose en la salud del hardware subyacente.

## **2\. Estratificación de Fidelidad Vectorial**

Se abandona el vector plano unificado a favor de una jerarquía de latencia y precisión. El UnifiedNode desacopla su representación vectorial.

### **2.1. Enum VectorRepresentations (src/node.rs)**

pub enum VectorRepresentations {  
    /// L1: Índice Rápido en RAM. Distancia de Hamming (XOR \+ POPCNT). Huella mínima.  
    Binary(Box\<\[u64\]\>),  
      
    /// L2: Re-ranking y Validación inicial. Mapeado a memoria desde disco.  
    Turbo(Box\<\[u8\]\>),  
      
    /// L3: Arqueología Semántica y Resolución de Pánico. Precisión absoluta en RocksDB.  
    Full(Vec\<f32\>),  
}

### **2.2. Transformada FWHT (src/vector/transform.rs)**

Para mitigar el error de redondeo de 1-bit y 3-bit, los vectores de entrada pasan por una Transformada Rápida de Walsh-Hadamard.

* **Fast Path:** Implementación SIMD bloque a bloque usando wide::f32x8 (AVX2/AVX-512).  
* **Fallback:** Implementación escalar iterativa para perfiles Survival (hardware antiguo).

## **3\. Gobernanza de Certeza y Backpressure Cognitivo**

El motor ajusta su rigor lógico basándose en los recursos físicos disponibles y el nivel de certeza exigido por el agente en NeuLISP.

### **3.1. Modos de Ejecución Lógica (src/eval/vm.rs)**

| Modo | Latencia Objetivo | Estrato Usado | Garantía | Comportamiento del Executor |
| :---- | :---- | :---- | :---- | :---- |
| **FAST** | \< 10ms | L1 (1-bit) | Probabilística | Búsqueda puramente heurística. Sin validación axiomática. |
| **BALANCED** | \< 50ms | L1 \+ L2 | Estadística | Default. Re-ranking 3-bit MMap. Emite invalidación asíncrona si L3 difiere. |
| **STRICT** | Variable | L1 \+ L2 \+ L3 | Determinista | Bloquea el hilo LISP hasta recuperar FP32 y validar contra Axiomas. |

### **3.2. Hardware Autodiscovery & Perfiles Dinámicos (src/hardware/mod.rs)**

Durante la instanciación, el motor ejecuta un micro-benchmark (I/O latency, Memory Bandwidth) para asignar un perfil operativo que gobierna el Backpressure:

* **Survival (\< 8GB RAM / HDD):** Umbral de backpressure de 50ms. MMap intensivo.  
* **Cognitive (16GB RAM / SSD):** Target principal. Umbral de 150ms.  
* **Enlightened (\> 32GB RAM / NVMe):** L1 y L2 en RAM. Backpressure desactivado.

### **3.3. Reflejo de Inhibición (Backpressure)**

Si la cola de I/O de disco para consultas STRICT supera el umbral del perfil detectado, el ResourceGovernor degrada automáticamente las nuevas consultas a BALANCED, devolviendo un aviso en el FidelityHeader de la respuesta para evitar el colapso sistémico.

## **4\. Protocolo de Invalidez Reactiva ("Aha\! Moment")**

Dado que el modo BALANCED permite inferencias basadas en L2 (3-bit) sin esperar a L3 (FP32), se requiere un mecanismo para deshacer paradojas lógicas post-hoc.

### **4.1. Suscripción de Verdad Eventual (src/governance/mod.rs)**

* **InvalidationDispatcher:** Un actor de Tokio que supervisa las validaciones en segundo plano del SleepWorker.  
* Si L3 (FP32) contradice la inferencia de L2 entregada previamente, emite un evento PREMISE\_INVALIDATED vía MCP o HTTP Webhooks.

### **4.2. Linaje Semántico y Epochs (src/node.rs)**

* Cada UnifiedNode incorpora un u32 epoch.  
* Si la validación L3 requiere mutar semánticamente el nodo, el epoch se incrementa. La versión anterior (alucinación) se traslada al shadow\_kernel con el flag NodeFlags::HALLUCINATION.

## **5\. Archivos a Intervenir**

1. **src/node.rs**: Actualizar UnifiedNode con VectorRepresentations, epoch y NodeFlags::HALLUCINATION.  
2. **src/vector/transform.rs**: Implementar FWHT (SIMD y Escalar).  
3. **src/hardware/mod.rs**: Implementar micro-benchmark de instanciación y perfiles.  
4. **src/executor.rs**: Implementar los modos FAST, BALANCED, STRICT y el límite temporal (Short-Circuit Lógico de MMap).  
5. **src/governance/invalidations.rs**: Crear el InvalidationDispatcher pub/sub.

## **6\. Criterios de Aceptación**

* \[ \] La compresión L1 ocupa \~96 bytes por nodo (768d). L2 reside exclusivamente en disco mapeado (MMap).  
* \[ \] El motor cambia de STRICT a BALANCED dinámicamente si se inyecta latencia artificial \> 150ms en el SSD.  
* \[ \] Consultas LISP que resulten en discrepancias axiomáticas asíncronas emiten un evento PREMISE\_INVALIDATED.  
* \[ \] Tests unitarios verdes para FWHT SIMD/Escalar.