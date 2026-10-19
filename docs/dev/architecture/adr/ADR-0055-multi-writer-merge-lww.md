---
title: "ADR-0055: Multi-escritor — LWW explícito con detección de conflicto (escenario federación v1)"
kind: adr
status: accepted
description: "Una estrategia multi-escritor declarada: escritores que convergen vía la ruta de merge; LWW explícito sobre (updated_at_ms, contenido canónico) + conflicto detectado en empate temporal; merge_lock; upgrade vector-clock/CRDT-lite declarado."
tags: [vantadb, architecture, adr, memory, multi-writer, sync, lww]
created: "2026-10-05"
---

# ADR-0055: Multi-escritor — LWW explícito con detección de conflicto (escenario federación v1)

> **Decidido por delegación del owner (MEMG-05, plan 0.9.0 Task 57):** el contrato del plan manda
> "una estrategia de resolución de conflictos multi-escritor implementada y declarada (CRDT /
> vector-clock / LWW explícito — elección documentada con ADR) para el escenario fijado en
> DISCOVERY". Este ADR fija **el escenario, la estrategia y sus límites**.

## Context

La posición de producto es "memoria federada local-first con merge determinista — CRDT-lite sobre
UpdateOperations" (`docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md:412`;
Desktop + WASM + embedded = posición única). El estado verificado en HEAD (2026-10-05):

- `CRDT` / `vector_clock` / `LWW` = **0 hits en código Rust**. El comportamiento last-write-wins
  solo existe como dedup por ID de storage (evita duplicados, no resuelve conflictos entre
  escritores) y como evolución de campos.
- **WAL encadenado 1-writer** (`src/wal.rs:399-421`): un solo escritor por store; el framing v3
  encadena `prev_hash‖record_hash` por frame.
- **wal-shipping** (`src/wal_shipping.rs`): replicación primario→réplica **unidireccional**, sin
  merge — no es multi-escritor.
- **Import/interchange**: "Conflicting content for the same key is **last-write-wins on import**"
  (`docs/api/MEMORY_INTERCHANGE_FORMAT.md:64`) — el ganador lo decide el **orden de llegada**
  (quien importa último gana), sin política declarada ni detección. Es el escenario que HOY pierde
  datos silenciosamente cuando los registros de dos escritores (device/agente) convergen.
- **Multi-tab OPFS**: "last write wins with no merge", corrupción silenciosa
  (`docs/dev/wasm/CRASH_MODEL.md:40`). La mitigación por **exclusión** (Web Locks / OPFS lock) es
  WSM-15 (Task 9) — **frontera declarada**: WSM-15 = exclusión, MEMG-05 = resolución/merge.
- **Multi-proceso embebido**: excluido por diseño (lock de un solo escritor,
  `src/storage/engine/mod.rs` `_lock_file`).

**Escenario fijado (v1, uno solo):** escritores múltiples (devices/agentes) que producen versiones
de la misma `(namespace, key)` y cuyas escrituras **convergen en un store a través de la ruta de
merge** (federación / sync / intercambio). El transporte de sync (event-log entre dispositivos) NO
entra en v1; lo que se fija e implementa es la **resolución de conflictos** que cualquier
transporte necesitará (FIND del resto).

## Decision

**Estrategia: LWW explícito con detección de conflicto**, implementado como ruta opt-in
`Embedded::merge_record()` (el write path `put`/import/WAL queda intacto):

1. **Orden total determinista `(updated_at_ms, content-bytes)`.** El timestamp es el reloj
   del escritor (viaja en el registro, como en el import). El contenido = `payload + metadata`
   serializados de forma canónica (postcard; `BTreeMap` ordena las claves). En empate de tiempo,
   gana el contenido canónicamente mayor — **arbitrario pero convergente**: todas las réplicas
   computan el mismo ganador sin importar el orden de llegada. Un rewrite de contenido idéntico
   con reloj más nuevo **avanza la clave de orden** (`updated_at_ms` almacenado := max del write
   set) aunque no cambie el contenido: sin eso el reloj dependería del orden de llegada y
   arrastraría a los ganadores posteriores (review P2-01 R1).
2. **Fingerprint = `payload + metadata`.** El resto de campos (ventana de validez, TTL,
   confidence/clase, `derived_from`, vectores, `last_validated_at_ms`, cuarentena,
   `superseded_by`) no participa de la comparación de conflicto. En un merge que escribe, el
   registro entrante se persiste **como unidad completa** (sus campos reemplazan a los
   almacenados): un transporte debe llevar registros completos — un `vector = None` reemplaza
   al almacenado, igual que en el import.
3. **Detección de conflicto:** empate temporal + contenido distinto = conflicto detectado
   (`MergeResult::conflict`). Timestamps distintos = actualización LWW declarada (no conflicto).
   El transporte de sync NO necesita vector clock para que las réplicas converjan.
4. **Nunca silencioso:** cada merge devuelve `MergeResult` (`Inserted` | `Updated` |
   `StaleRejected` | `AlreadyCurrent` + `conflict` + `winner_updated_at_ms`). El perdedor de un
   empate queda señalado, no descartado en silencio.
5. **Atomicidad:** un `merge_lock` (patrón REVIEW-13, el mismo rationale del `supersede_lock`)
   serializa resolve→decide→write; sin él, dos merges concurrentes podrían decidir contra el
   mismo `existing` y persistir el perdedor (no-determinismo por scheduling).
6. **Upgrade path declarado:** vector clocks (identidad de réplica + reloj por registro, para
   detectar concurrencia causal real y no solo empates de reloj) → CRDT-lite field-level sobre
   update operations (la dirección de `strategy:412`).

## Alternatives Considered

### Vector clock ahora

- Pros: orden causal real; detección de concurrencia correcta.
- Cons: identidad/registro de réplicas, sesiones y GC de relojes por registro — el appetite
  completo (2-3sem) solo para el reloj, sin consumidor de sync aún.
- **Rechazado para v1; declarado como upgrade** (el `updated_at_ms` del registro ya es el punto
  de extensión natural).

### CRDT completo (registro con merge semántico, p.ej. campo a campo)

- Pros: convergencia sin coordinación ni relojes de pared.
- Cons: excede el appetite (pre-mortem del plan); requiere tipar las operaciones de update
  (`UpdateOperations`), que hoy no existen como API.
- **Rechazado para v1; declarado como dirección** (CRDT-lite sobre update operations).

### Aplicar la política dentro de `put_record_exact`/import

- Pros: cero superficie nueva.
- Cons: cambia la semántica documentada del interchange ("LWW on import" es el contrato del
  transporte crudo para archivos de terceros) y rompe tests existentes; el contrato del plan
  exige "write path actuales intactos".
- **Rechazado.** El merge es una ruta nueva y opt-in.

### Desempate por `writer_id` persistido (en vez de bytes de contenido)

- Pros: semánticamente más explicable ("gana el escritor de id mayor").
- Cons: campo nuevo en el wire/serialización de memoria + plomería de identidad en cada
  superficie (import, bindings) — fuera del corte.
- **Rechazado para v1.** El desempate por bytes de contenido converge sin identidad persistida.

## Consequences

- **Pros:** convergencia determinista y verificable (tests: permutaciones + paralelo) sin
  metadatos de reloj; detección de conflicto sin vector clock; cero cambios de wire/on-disk; cero
  deps nuevas; el write path existente queda byte-idéntico (merge es opt-in).
- **Límites declarados (honestidad del contrato):**
  1. **Skew de relojes de pared:** un reloj adelantado puede invertir el orden causal real — v1 lo
     acepta como límite; el upgrade (vector clock) lo corrige.
  2. **Empate de tiempo:** el desempate por bytes es arbitrario (no semántico) pero estable y
     convergente; el conflicto queda señalado para que el llamador decida (p.ej. cuarentena).
  3. **Bookkeeping local:** `version` y `created_at_ms` (first-seen) son store-locales y no
     convergen byte a byte entre réplicas; lo convergente es `(payload, metadata, updated_at_ms)`
     del ganador declarado (el reloj almacenado es el **max** del write set — R1).
  4. **put concurrente vs merge sobre la misma clave:** fuera de cobertura v1 (usar una ruta por
     clave; el merge lock serializa merge-vs-merge, no merge-vs-put).
  5. **Detección acotada:** solo el empate temporal es detectable sin vector clock; una escritura
     "vieja pero con timestamp mayor" se comporta como update LWW declarado.
  6. **Sin transporte:** el event-log/sync entre dispositivos y los bindings (Python/WASM/MCP/
     HTTP) de `merge_record` quedan como FIND (siguiente iteración de federación).
  7. **Alcance de campos:** el fingerprint de conflicto es solo `payload + metadata`; diferencias
     en los demás campos no se reconcilian a tiempo igual (un merge `Unchanged` no escribe nada),
     y en un merge que escribe el registro entrante completo reemplaza al almacenado (campos
     derivados incluidos).
  8. **Lock por handle:** `merge_lock` es por instancia `Embedded` (mismo alcance conocido de
     `supersede_lock`, REVIEW-13); dos handles sobre el mismo `StorageEngine` (p.ej. vía
     `from_engine`) no comparten la serialización — una ruta por handle.
- **Relaciones:** WSM-15 (exclusión OPFS) es complementario, no solapado; wal-shipping sigue
  siendo replicación unidireccional sin merge (sin cambios); `MEMORY_INTERCHANGE_FORMAT.md`
  documenta la ruta de merge como la política multi-escritor declarada.
