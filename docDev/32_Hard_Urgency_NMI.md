# Fase 32: Hard-Urgency / NMI (Mecanismo de Colapso Forzado)

> **Estado:** 🔲 PENDIENTE  
> **Versión Objetivo:** v0.5.0  
> **Prerequisito:** Fase 31B (ThalamicGate & Uncertainty Zones) ✅

---

## Concepto

En situaciones de alta carga cognitiva o escasez de recursos, ConnectomeDB no puede permitirse el lujo de mantener vectores en "Penumbra" especulativa esperando pasivamente a que venza su fecha de colapso. Esta fase introduce **Non-Maskable Interrupts (NMI)** para forzar decisiones subóptimas pero inmediatas, asegurando la supervivencia del motor en *Edge*. Además, optimiza el `ThalamicGate` deshaciéndose de estructuras con contención (`RWLock<HashSet>`) a favor de Filtros de Bloom manuales.

## Modificaciones Estructurales

### 1. Filtro de Bloom In-House (`ThalamicGate`)
La compuerta rechazada transiciona a un Bloom Filter minimalista "sin dependencias" escrito desde cero usando `Vec<u8>` y `DefaultHasher`.
*   **Target:** 10,000 IDs simultáneos.
*   **Falso Positivo:** < 0.01.
*   **k-hashes:** 3 semillas dinámicas de sal (Salts).
Esto reemplaza el cuello de botella previo limitando el overhead a operaciones atómicas o un solo lock ultra-rápido sobre la matriz de bytes.

### 2. Estadísticas Atómicas de Colapso (`CollapseStats`)
El `UncertaintyBuffer` rastreará atómicamente el destino de los `QuantumNeurons`:
*   `superposition_to_collapsed`
*   `superposition_to_decayed`

El motor de `SleepWorker` analizará el ratio $\frac{decayed}{total}$ al comienzo de cada ciclo REM. Si el ratio supera el **70%**, se asume que el flujo entrante es predominantemente "ruido" o falsos positivos. En consecuencia, el motor acortará preventivamente el `collapse_deadline_ms` de nuevas ingestas para liberar presión a priori.

### 3. NMI y Colapso Forzado Especulativo
Dentro de `UncertaintyBuffer`, se introduce el método de cortocircuito `force_collapse_nmi()`.
*   Activado por el `ResourceGovernor` frente a presión de RAM severa (>90% cuota).
*   Ignora cualquier validación de `TrustScore` restante.
*   **Comportamiento heurístico:** Integra ("acepta") el candidato que posea la Semántica Mayor (valencia alta) dictada probabilísticamente y purga todos los demás sin miramientos. Privilegia una reacción imperfecta frente al crash (OOM).

## Archivos Impactados
*   `src/governance/thalamic_gate.rs`: Refactorización hacia Filtro de Bloom manual.
*   `src/governance/uncertainty.rs`: Atómicos de `CollapseStats` y rutinas de NMI.
*   `src/governance/sleep_worker.rs`: Ingesta adaptativa guiada por ratios dinámicos.
