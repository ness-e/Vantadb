---
type: glossary-entry
status: stable
tags: [concurrencia, aislamiento, transacciones]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Multi-Version Concurrency Control, MVCC]
description: "Concurrency control method where each transaction sees a consistent snapshot of the database, allowing readers and writers to operate simultaneously without blocking"
---
#MVCC—Multi-Version Concurrency Control

##Definition

**MVCC** is a method of concurrency control where **each transaction sees a consistent snapshot** of the database, allowing readers and writers to operate simultaneously without blocking each other.

## How It Works

### Basic Concept

```
Transacción A (lectura):
- Ve snapshot del tiempo T1
- No ve cambios de transacciones posteriores

Transaction B (write):
- Write new version of data
- Does not block Transaction A

Result:
- Readers do not block writers
- Writers do not block readers
- Every transaction sees consistent view
```

## Usage in VantaDB

MVCC is implemented by the [[fjall]] backend for concurrent transactions.

## See Also

- [[fjall]] — Backend with native MVCC
- [[transactional]] — Property that MVCC enables
- [[rwlock]] — Simpler alternative

---

*MVCC allows concurrency without global locks.*

