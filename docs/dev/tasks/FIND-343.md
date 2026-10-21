---
title: "FIND-343 - Borrado fisico incompleto: el payload sigue legible en el WAL rotado y el journal"
kind: task
status: active
description: "Delimitar y corregir la retencion del payload de un registro borrado: el WAL rotado y el journal de Fjall lo conservan."
tags: [durabilidad, wal, privacidad, critica]
---

# FIND-343: Borrado fisico incompleto: el payload sigue legible en el WAL rotado y en el journal Fjall

Tras un `delete` (logico), el **payload** sigue siendo recuperable: en el **WAL rotado** (`vanta.wal.<ts>`, los shards rotados por auto-rotate) y en el **journal de Fjall** (el backend). Es decir: un registro borrado no ha borrado sus bytes, solo ha marcado su clave como borrada. Un atacante con acceso al disco (o un `strings` sobre los ficheros) recupera el contenido.

El alcance de W6 (cifrado en reposo) **cambia el marco**: con la feature `encryption` activada, el WAL y el backend estan cifrados ⇒ el payload ya no esta en claro ⇒ el problema pasa a ser (a) el **cifrado en reposo no esta activo** por defecto (la feature es opt-in) y (b) la **retencion** del material cifrado: rotar el WAL no borra las DEKs antiguas, y un ciphertext de un registro borrado sigue descifrable con la clave vigente.

Ojo: el cifrado **no** es la remezcla del borrado fisico (un `delete` fisico real debe compactar/reescribir los segmentos), y ademas el borrado fisico tiene un coste que hay que medir (Regla 9). Esta tarea es de **durabilidad y privacidad**, y su alcance hay que delimitar antes de codificar.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-343 (critica))
- **Esfuerzo:** 🔴 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 8-12
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T12:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 6 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/storage/engine/ops.rs, src/storage/engine/txn.rs, src/wal_shipping.rs, src/storage/archive.rs |
| Callees | src/wal.rs, src/wal_sharded.rs, src/backends/**, src/storage/archive.rs |
| Implicaciones | el borrado fisico tiene un coste (reescribir segmentos) y un riesgo (perder datos si el crash ocurre a mitad). El cifrado en reposo ya mitiga la lectura en claro, pero no borra el material cifrado. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/wal.rs (auto-rotate y retencion de segmentos), src/wal_sharded.rs, src/storage/engine/delete.rs (borrado logico), src/backends/ (el journal de Fjall), src/storage/archive.rs (compactacion)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** wal.rs -> storage::vfile_crypto + crypto (el envelope); backends/ -> journal de Fjall; delete.rs -> backend + wal
- **Archivos que referencian a los editados (referencias entrantes):** src/storage/engine/ops.rs, src/storage/engine/txn.rs, src/wal_shipping.rs (lee los segmentos), src/storage/archive.rs
- **Veredicto impacto:** alto: es el unico hallazgo de privacidad de datos en el backlog. Toca el hot path del WAL y el formato en disco ⇒ cualquier cambio de formato necesita version (WAL v4 ya existe) y el shipper (`FIND-496`) lo tiene que poder leer.

## Contrato
"Tras `delete` + `vacuum`/compactacion, el payload **no** es recuperable ni del WAL rotado ni del journal: un `strings`/busqueda de bytes del payload en el directorio de datos **no** lo encuentra (con la feature `encryption` activada: el contenido esta cifrado y el registro borrado no se puede descifrar sin la DEK vigente)."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) **no se puede perder datos**: una compactacion interrumpida por un crash deja la base **recuperable** (es el requisito de durabilidad: WAL + replay); 2) el WAL rotado no se puede borrar antes de que su contenido este en un segmento vigente (o en un snapshot); 3) el formato en disco versionado (WAL v4) ⇒ un cambio necesita subir de version y el lector debe seguir leyendo v3; 4) el shipper (`FIND-496`) no puede romperse; 5) el borrado fisico **no** puede degradar el rendimiento del camino caliente sin medirlo (Regla 9).
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib wal`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-343: Borrado fisico incompleto: el payload sigue legible en el WAL rotado y en el journal Fjall` |
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
| Pendientes de ejecución (downhill) | 6 |
| % completado | 0% |

## Steps

1. **Delimitar el alcance ANTES de codificar** (esta tarea es ambigua a proposito):
   medir **donde** queda el payload tras un delete (grep del payload en el WAL rotado, en los
   shards, en el journal de Fjall y en los snapshots) y con que feature activada. El resultado
   de la medicion es parte del entregable.
2. **Test primero** (que falle): tras `delete` + compactacion, el payload no aparece en el
   directorio de datos (con `encryption` activada y sin ella).
3. **Fix**: segun la medicion, (a) purge del WAL rotado tras la compactacion (y no antes),
   (b) reescritura/purge del journal, (c) o la garantia de que con `encryption` el material
   cifrado de un registro borrado no es recuperable (rotacion de DEK por borrado ⇒ coste).
4. **Versionar** el formato si cambia (WAL v4 -> v5) y **verificar que el lector viejo sigue
   leyendo** el formato anterior.
5. **Medir** el coste del borrado fisico (Regla 9) y anotarlo.
6. **Verificar**: fmt, clippy, la suite del lib completa, y el test nuevo en verde.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
