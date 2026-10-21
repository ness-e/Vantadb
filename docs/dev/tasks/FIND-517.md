---
title: "FIND-517 - La pre-reserva de 320 MB al conectar el binding de Node"
kind: task
status: active
description: "Que conectar un store vacio no reserve 320 MB: es el coste dominante de memoria del producto de Ego (VDB-REQ-14)."
tags: [bindings, node, memoria, critica, ego]
---

# FIND-517: La pre-reserva de 320 MB al conectar el binding: un store vacio ya ocupa 320,06 MB

Conectar el binding de Node **reserva 320 MB** (5 x 64 MB) antes de que haya datos: un store **vacio** ya ocupa 320,06 MB, y `ConnectOptions` solo expone `read_only` y `memory_limit` ⇒ la reserva **no es parametrizable**. En el producto de Ego cada memoria abierta (proyecto, sesiones, sub-egos) cuesta 320 MB ⇒ es el **coste dominante de memoria del producto**, y no sale en ningun `memory_limit` porque no es memoria viva sino espacio reservado.

Es la peticion #1 de la lista priorizada de Ego (`VBD-REQ-14`, verificada el 2026-10-21 contra el binario real 0.9.0: no son restos de test, es la pre-reserva).

Los tres fixes posibles (del informe): (a) **modo lazy**: no reservar hasta que el grafo necesite el espacio, reservando en background; (b) parametrizar la pre-reserva (`ConnectOptions.initial_capacity` / `reserve_bytes`) con un valor por defecto bajo; (c) dejar de pre-reservar y dejar que el allocator crezca bajo demanda.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-517 (critica))
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 6-9
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T12:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 5 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | vantadb-ts/src/native.ts, los ejemplos de vantadb-node y vantadb-ts, los docs del SDK |
| Callees | src/index/** (la reserva de 5 x 64 MB), vantadb-ffi-core (OpGate) |
| Implicaciones | el pico de memoria del producto baja de ~320 MB por memoria a lo que el grafo use de verdad. El tiempo de connect no debe crecer (medir). |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** vantadb-node/src/lib.rs (el ConnectOptions y la reserva: donde se reserva y cuanto), el index del indice (la reserva en el lado Rust), vantadb-node/index.d.ts (el contrato publico), vantadb-ts/src/native.ts (el ConnectOptions que envuelve)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** vantadb-node/src/lib.rs -> vantadb (Embedded) -> src/index (la reserva del grafo); vantadb-ts/src/native.ts -> vantadb-node (napi)
- **Archivos que referencian a los editados (referencias entrantes):** vantadb-node/index.d.ts (el tipo publico), vantadb-ts/src/native.ts, examples/ y docs que construyen un cliente
- **Veredicto impacto:** medio-alto: cambia el contrato publico del binding (un parametro nuevo en `ConnectOptions`) ⇒ hay que actualizar el `.d.ts` (generado), el `.cjs`/`index.js` commiteados (FIND-547: estan desfasados) y los docs. Y toca el **arranque** del binding ⇒ medir el tiempo de connect.

## Contrato
"Un store **vacio** ya no ocupa 320 MB: el consumo de RSS tras `connect()` de un store vacio es el de la metadata (orden de MB, no de cientos) y el `memory_limit` del `ConnectOptions` **cubre** la reserva (o la reserva no existe como memoria reservada). El pico de memoria de un uso real de Ego baja del orden de los cientos de MB, y el tiempo de `connect()` no empeora."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) el **contrato publico** del binding no puede romperse sin bumping la version (un parametro nuevo es aditivo y con default); 2) la reserva no puede crecer por encima del `memory_limit` del usuario; 3) el **tiempo de connect** no puede empeorar de forma perceptible (medir); 4) el `.d.ts` y el `.cjs`/`index.js` commiteados deben seguir siendo consistentes con el codigo (FIND-547: hoy estan desfasados); 5) nada de `unsafe` nuevo (la frontera FFI es memoria-segura por construccion, FIND-504).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib node  (o el target de node si no hay)`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-517: La pre-reserva de 320 MB al conectar el binding: un store vacio ya ocupa 320,06 MB` |
| `lastAction` | (se actualiza en cada step) |
| `result` | ⬜ PENDING |
| `nextAction` | Step 1 (ver `## Steps`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) |
| **Release** | No aplica (sin release en esta iteracion) |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
security-and-hardening (cuando el hallazgo es de integridad de datos).

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 |
| % completado | 0% |

## Steps

1. **Verificar la evidencia** (sin editar): localizar la reserva de 5 x 64 MB (el
   `ConnectOptions` y el `Connect` de `vantadb-node/src/lib.rs`) y **medir** el RSS de un store
   vacio (el numero del backlog: 320,06 MB) para tener la linea base.
2. **Elegir el fix** (decision de producto, pero con un default defendible): (a) lazy, (b)
   parametrizable con default bajo, o (c) sin pre-reserva. Documentar la eleccion y su
   justificacion en el task file.
3. **Implementar** con el `.d.ts` regenerado y el `.cjs`/`index.js` coherentes (FIND-547).
4. **Test primero**: un test que verifique que un store vacio **no** reserva 320 MB (o que la "
   "reserva respeta el `memory_limit`).
5. **Verificar**: fmt, clippy, la suite del lib completa, el RSS medido antes/despues y el tiempo "
   "de connect.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
