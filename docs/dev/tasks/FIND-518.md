---
title: "FIND-518 - El WHERE entero de IQL se pierde si la primera condicion no parsea"
kind: task
status: active
description: "Corregir el parser: un WHERE mal formado debe ser un error de parseo, no un escaneo sin filtro. Incluye FIND-519 (el trailing input se descarta en silencio)."
tags: [parser, iql, critica, correctness]
---

# FIND-518: El WHERE entero se pierde si la primera condicion no parsea

El parser de IQL envuelve la clausula `WHERE` en `opt(...)`: en `nom`, un `opt` ante `Err::Error` **restaura el input** y devuelve `None`. Con `FROM Doc WHERE n =` (una condicion mal formada) el resultado es `FROM Doc` **sin filtro y sin error**: la query se ejecuta como un escaneo completo y `PhysicalScan::open` materializa todos los nodos en RAM antes de filtrar (viola `memory-budget.md` R-3/R-4). Su hermana `FIND-519` es el mismo patron: el *trailing input* de cualquier statement se descarta en silencio (HTTP, MCP y CLI), asi que `WHERE a = 1, b = 2` aplica un filtro y **pierde el otro**, `min >= 0.88` degrada el brazo vectorial a match lexico, `PROFILE bogus` corre sin perfil e `INSERT ... LINK ...` inserta el nodo sin arista.

**El fix estructural de ambas es el mismo:** que `reject_trailing_input` sea estricto para todas las statements (tolerando solo `;`) y que el `WHERE` propague el error cuando su keyword ya se consumio. Ese unico cambio cierra `FIND-518`, `FIND-519` y parte de `FIND-525`/`FIND-526`.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-518 (critica) + FIND-519 (alta, su hermana))
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 6-8
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T12:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | src/executor.rs, src/server/handlers.rs, vantadb-mcp/src/handlers/tools.rs, src/bin/vanta-cli.rs |
| Callees | src/parser/grammar.rs, src/parser/lexer.rs |
| Implicaciones | el contrato de error cambia: un query mal formado pasa a fallar en vez de ejecutarse sin filtro. Es el comportamiento que la doc ya promete. |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** src/parser/grammar.rs (832 lineas), src/parser/lexer.rs (180), src/parser/mod.rs, src/executor.rs (la parte de parseo)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** src/parser/mod.rs -> grammar.rs + lexer.rs; grammar.rs -> lexer.rs (tokens); executor.rs -> parser::parse_statement
- **Archivos que referencian a los editados (referencias entrantes):** src/executor.rs (parse_statement), src/server/handlers.rs, vantadb-mcp/src/handlers/tools.rs (query_iql), src/sdk (IQL)
- **Veredicto impacto:** alto: cambia el contrato observable del parser (un trailing pasa a ser error). Hay que enumerar los llamadores que hoy dependen del descarte silencioso y actualizarlos (CLI/REPL con `;`, MCP, HTTP).

## Contrato
"`FROM Doc WHERE n =` devuelve **error de parseo** (no un escaneo sin filtro) y `FROM Doc WHERE a = 1, b = 2` tambien; `FROM Doc` (sin WHERE) sigue funcionando; `FROM Doc;` (con `;`) sigue funcionando; el resto del trailing input (`FROM Doc garbage`, `PROFILE bogus`, `INSERT ... LINK ...`) devuelve error. Tests: hay un test que fija cada caso."

## Spec (SDD)

N/A — no es feature-add ni symbolos publicos nuevos: es la correccion de un
comportamiento existente que ya tiene contrato documentado (el del backlog).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** 1) un statement **valido** no puede pasar a fallar (la suite de 2493 tests del lib debe seguir verde); 2) el `;` final (REPL/CLI) se sigue tolerando; 3) los limites de longitud de query no cambian; 4) nada de panics: todo fallo de parseo es un `Error` con mensaje util.
- **Comandos de verificación:** `cargo nextest run -p vantadb --profile audit --lib parser`
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-518: El WHERE entero se pierde si la primera condicion no parsea` |
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
| Pendientes de ejecución (downhill) | 4 |
| % completado | 0% |

## Steps

1. **Verificar la evidencia** (sin editar): confirmar que `opt(...)` alrededor del
   `WHERE` y el `reject_trailing_input` no estricto se comportan como dice la fila (leer
   `grammar.rs:196-269` y `:737-753`).
2. **Test primero**: escribir los tests que fijan el comportamiento roto (un `WHERE` mal
   formado, un trailing, una coma en vez de `AND`, `PROFILE bogus`) y verlos **fallar**.
3. **Fix**: (a) el `WHERE` propaga el error cuando su keyword ya se consumio (`Err::Failure`
   en vez de `opt`); (b) `reject_trailing_input` estricto para todas las statements
   (tolerando solo `;`). Ajustar los llamadores que dependian del descarte silencioso.
4. **Verificar**: `cargo fmt --check`, `cargo clippy -p vantadb --all-targets -- -D warnings`,
   la suite del lib completa, y los tests nuevos en verde.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador; la tarea NO se marca
  COMPLETED sin este review registrado aqui)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7).
