---
type: glosario-entry
status: stable
tags: [concepto, acid, durabilidad, consistencia]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Transaccional, ACID, Transactional]
description: "Propiedad de un sistema que garantiza que las operaciones sobre datos cumplan las propiedades ACID (Atomicidad, Consistencia, Aislamiento, Durabilidad)"
---

# Transaccional

## Definición

Un sistema **transaccional** garantiza que las operaciones sobre datos cumplan las propiedades **ACID** (Atomicidad, Consistencia, Aislamiento, Durabilidad), asegurando que las mutaciones sean confiables incluso ante fallos del sistema, crashes o concurrencia.

## Propiedades ACID

| Propiedad | Definición | Implementación en VantaDB |
|-----------|-----------|--------------------------|
| **Atomicidad** | Todo o nada: una transacción se completa entera o no se aplica | [WAL](WAL.md) + rollback automático |
| **Consistencia** | El sistema pasa de un estado válido a otro estado válido | Validación de constraints + índices derivados |
| **Aislamiento** | Transacciones concurrentes no interfieren entre sí | [MVCC](MVCC.md) en [Fjall](Fjall.md) + [RwLock](RwLock.md) |
| **Durabilidad** | Una transacción confirmada sobrevive a crashes | [WAL](WAL.md) con [fsync](fsync.md) + [CRC32C](CRC32C.md) |

## Por Qué Importa en VantaDB

VantaDB gestiona **memoria persistente para agentes de IA**. Si un agente guarda contexto importante (conversaciones, decisiones, conocimiento adquirido), **no puede perderse** por un crash o corte de energía.

### El Contrato Transaccional de VantaDB

```
Agente de IA                VantaDB                    Disco
    │                          │                          │
    │──── put(document) ──────▶│                          │
    │                          │── Write to WAL ─────────▶│
    │                          │── fsync() ──────────────▶│ [DURABLE]
    │                          │◀─ ACK ──────────────────│
    │◀──── Success ────────────│                          │
    │                          │                          │
    │                          │  [CRASH / Power Loss]    │
    │                          │                          │
    │                          │── Replay WAL ───────────▶│ [RECOVERED]
    │                          │                          │
```

### Regla Fundamental

> **Ninguna mutación se confirma al cliente hasta que el [WAL](WAL.md) esté sincronizado en disco físico mediante [fsync](fsync.md).**

Esto diferencia a VantaDB de sistemas que:
- Escriben en memoria y hacen flush periódico (riesgo de pérdida)
- Usan WAL pero sin fsync síncrono (riesgo en cortes de energía)
- No tienen WAL (sin garantías de durabilidad)

## Transaccionalidad Multimodelo

VantaDB es **transaccional a través de múltiples representaciones**:

```
Transacción Atómica
├── Documento canónico (fuente de verdad)
├── Embedding vectorial ([Vectores](Vectores.md))
├── Relaciones de [Grafo](Grafo.md) (aristas)
├── Metadatos tipados (payload)
└── Índices derivados ([HNSW](HNSW.md), [BM25](BM25.md))
```

Si actualizas un documento:
- ✅ El documento se actualiza
- ✅ Su embedding se regenera
- ✅ Sus relaciones de grafo se mantienen consistentes
- ✅ Los índices se reindexan
- ✅ Todo en una sola transacción atómica

**O todo sucede, o nada sucede.** No hay estados intermedios visibles.

## Niveles de Durabilidad Configurables

VantaDB soporta (o debería soportar) múltiples modos de sincronización:

| Modo | Descripción | Latencia | Riesgo |
|------|-------------|----------|--------|
| **SyncAlways** | fsync en cada write | Alta (~5-10ms) | Cero pérdida de datos |
| **SyncPeriodic** | fsync cada N ms | Baja (<1ms) | Pérdida de últimos N ms |
| **SyncNever** | Sin fsync (OS decide) | Mínima | Pérdida potencial alta |

> ⚠️ **Hallazgo de auditoría (AUD-01):** El snapshot no demuestra que VantaDB implemente fsync síncrono configurable. Esto es **bloqueante** para claims de durabilidad.

## Comparación con Alternativas

| Sistema | Transaccional | Atomicidad Multi-Modelo | Durabilidad Real |
|---------|--------------|------------------------|------------------|
| **VantaDB** | ✅ ACID completo | ✅ Doc + Vector + Grafo | ✅ WAL + fsync |
| **Pinecone** | Parcial | ❌ Solo vectores | ✅ Cloud-managed |
| **ChromaDB** | ⚠️ Básico | ⚠️ Doc + Vector | ⚠️ Dependiente de backend |
| **Qdrant** | ✅ ACID | ⚠️ Doc + Vector + Payload | ✅ WAL |
| **FAISS** | ❌ No transaccional | ❌ Solo índices | ❌ Sin persistencia propia |

## Anti-Patrón: "Transaccional Solo en Documentación"

Muchos sistemas claim ser transaccionales pero:
- No hacen fsync antes del ACK
- Pierden datos en crashes
- Reconstruyen índices desde estado inconsistente

VantaDB debe **demostrar** transaccionalidad mediante:
1. Tests de crash-injection ([Chaos Testing](Chaos Testing.md))
2. Verificación de checksum [CRC32C](CRC32C.md) en replay
3. Validación de consistencia post-recovery

## Véase También

- [WAL](WAL.md) — Mecanismo que habilita la durabilidad
- [fsync](fsync.md) — Garantía de persistencia física
- [CRC32C](CRC32C.md) — Integridad de registros
- [MVCC](MVCC.md) — Aislamiento de transacciones concurrentes
- [Fjall](Fjall.md) — Backend con soporte transaccional nativo

---

*Ser transaccional no es una feature opcional, es el contrato fundamental de una base de datos que gestiona memoria persistente.*

