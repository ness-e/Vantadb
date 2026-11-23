---
title: "AUDIT-01: Reparación de Calidad de Recuperación en LongMemEval"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-engine"
category: "retrieval"
tags: [retrieval, longmemeval, fastembed, rrf, chunking]
description: "Reparación de Calidad de Recuperación en LongMemEval (fastembed + chunking por turnos + pesos RRF por modalidad)."
---

# AUDIT-01: Reparación de Calidad de Recuperación en LongMemEval

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Auditoría v0.9.0 / `evals/eval_longmemeval.py`
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🔴 Alta
- **Tipo:** Python + Rust (evals/retrieval)
- **Turns estimados:** 15-20
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 4

## Objetivo

Restaurar la precisión y exhaustividad de recuperación semántica en el benchmark LongMemEval-S. Se debe sustituir la dependencia de embeddings remotos frágiles o proxies locales por inferencia embebida directa usando `fastembed` (modelo `BAAI/bge-small-en-v1.5`), migrar la estrategia de segmentación desde cortes ciegos por caracteres hacia chunking estructurado por turnos conversacionales (`user` / `assistant`), y habilitar calibración diferenciada de pesos RRF (`dense_weight` vs `sparse_weight`) en `vanta-memory`.

## Archivos en scope

- `evals/eval_longmemeval.py`: harness de evaluación LongMemEval-S.
- `evals/memory/**`: datasets, prompts y evaluador juez.
- `src/sdk/search/hybrid.rs`: lógica de fusión RRF y ponderación de modalidades.
- `src/sdk/api/memory.rs`: integración de recall y turnos.
- `vantadb-python/src/lib.rs`: bindings de búsqueda híbrida y scores.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | `evals/eval_longmemeval.py`, `vanta-memory` pipelines, Python SDK |
| Callees | `src/sdk/search/hybrid.rs`, `src/index/vector/`, `fastembed` |
| Implicaciones | La precisión de aguja en pajar (needle-in-a-haystack) sube sin regresión en latencia; contratos de scoring conservan rango [0.0, 1.0]. |

## Criterios de aceptación

1. Inferencia local con `fastembed` operativa en el arnés de `evals/` sin llamadas HTTP salientes.
2. Chunking estructurado respetando límites de turnos conversacionales (`turn_id`, `role`, `timestamp`) sin partir oraciones a mitad de palabra.
3. Fusión RRF con soporte para ponderaciones asimétricas de canal denso y léxico.
4. Recall@10 en LongMemEval-S superior a 0.88 en el dataset de control.

## Contrato

```bash
python -m pytest evals/test_retrieval_quality.py
python evals/eval_longmemeval.py --subset smoke --model fastembed --assert-recall 0.85
```

## Steps

### Step 1: Integrar `fastembed` en el harness de evaluación
- Configurar `fastembed` con `BAAI/bge-small-en-v1.5` en Python SDK / evals.
- Validar generación determinista de vectores y cacheo de embeddings.

### Step 2: Implementar segmentador conversacional por turnos
- Desarrollar `TurnBasedChunker` en `evals/` y adaptarlo para ingesta en `vanta-memory`.
- Preservar metadatos de sesión y autor en cada fragmento indexado.

### Step 3: Calibrar ponderación RRF por modalidad
- Ajustar `fuse_rrf` en `src/sdk/search/hybrid.rs` para recibir `dense_weight: f32` y `sparse_weight: f32`.
- Exportar los parámetros de peso hacia los bindings de Python y comandos de búsqueda.

### Step 4: Ejecución y verificación de benchmark
- Ejecutar LongMemEval-S y certificar resultados en reporte local.

## Invariantes de dominio

- La función de similitud vectorial de base no debe mutar las métricas de distancia (`Cosine`, `L2`).
- Las modificaciones no deben romper la compatibilidad de consultas léxicas BM25 puras.

