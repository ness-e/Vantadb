---
type: glosario-entry
status: stable
tags: [concurrencia, aislamiento, transacciones]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Multi-Version Concurrency Control, MVCC]
description: "Método de control de concurrencia donde cada transacción ve un snapshot consistente de la base de datos, permitiendo que lectores y escritores operen simultáneamente sin bloquearse"
---

# MVCC — Multi-Version Concurrency Control

## Definición

**MVCC** es un método de control de concurrencia donde **cada transacción ve un snapshot consistente** de la base de datos, permitiendo que lectores y escritores operen simultáneamente sin bloquearse mutuamente.

## Cómo Funciona

### Concepto Básico

```
Transacción A (lectura):
- Ve snapshot del tiempo T1
- No ve cambios de transacciones posteriores

Transacción B (escritura):
- Escribe nueva versión de datos
- No bloquea a Transacción A

Resultado:
- Lectores no bloquean escritores
- Escritores no bloquean lectores
- Cada transacción ve vista consistente
```

## Uso en VantaDB

MVCC es implementado por el backend [Fjall](Fjall.md) para transacciones concurrentes.

## Véase También

- [Fjall](Fjall.md) — Backend con MVCC nativo
- [Transaccional](Transaccional.md) — Propiedad que MVCC habilita
- [RwLock](RwLock.md) — Alternativa más simple

---

*MVCC permite concurrencia sin bloqueos globales.*

