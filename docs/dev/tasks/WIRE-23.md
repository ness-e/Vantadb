---
title: "WIRE-23: el --mcp delega el intervalo en la superficie compartida"
kind: task
status: completed
description: "memory_scheduler.rs dejaba de tener parser propio: interval_ms_from_env es ahora _opt().unwrap_or(5000) sobre el unico read site canonico. Basura => 60000 (resuelto por el canonico), no 5000: la enmienda de la fila."
id: WIRE-23
---

# WIRE-23: el `--mcp` mantiene su propio parser de intervalo

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `WIRE-23`, enmendada 2026-11-27)
- **Origen:** `WIRE-19` (partida 2026-11-27) · **Ver:** `WIRE-19`, `WIRE-20`, `WIRE-21`, `MCP-43`, `ADR-0063`
- **Esfuerzo:** 🟢 0.5 h · **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED

## Premisa (re-medida en HEAD)

`vantadb-mcp/src/memory_scheduler.rs:69` leía `VANTADB_SCHEDULER_INTERVAL_MS` con su propio
`match std::env::var(..)` + `parse` + `warn`: **segundo** read site del árbol, contra R-5
(`ADR-0063`: `scheduler_interval_ms_from_env_opt` es el único).

```
rg -c 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' vantadb-mcp/src   # antes: 1 · después: 0
```

## La decisión de la enmienda: basura ⇒ 60 000, no 5 000

El contrato original prometía *basura ⇒ 5 000*. Es imposible: `_opt` resuelve un valor
*presente pero no parseable* a su propio `DEFAULT_INTERVAL_MS` (**60 000**) y devuelve
`Some(60_000)`, indistinguible de un valor declarado. Distinguirlos exigiría releer el env
— el segundo read site que esta fila elimina.

| Entrada | `_opt()` | `interval_ms_from_env()` (MCP) |
|---|---|---|
| sin definir | `None` | **5 000** (default del host, se queda) |
| `"0"` | `Some(0)` | `0` (loop OFF) |
| `" 1500 "` | `Some(1500)` | `1500` (trim en el canónico) |
| `"not-a-number"` | `Some(60_000)` | **60 000** (R-5: warn + default canónico) |

Coherente con `_opt` y **documentado** en el doc comment de `interval_ms_from_env`, que es lo
que pedía la recomendación de la fila (§Deuda de `WIRE-20`).

## Cambio

- `interval_ms_from_env()` = `scheduler_interval_ms_from_env_opt().unwrap_or(DEFAULT_INTERVAL_MS)`.
- Se borra el `match` + el `warn` duplicado (el canónico ya avisa, con su propio mensaje).
- `DEFAULT_INTERVAL_MS = 5_000` (`:44`) **se queda**: es el default del *host*, no una copia
  del server (5 000 ≠ 60 000, justificado en `:38-43`, `WIRE-19-impl` §7.9).
- El bloque de `runner_factory` (`build_ingest_runner` en el closure P4) **no** se toca: es
  `WIRE-21`, sigue abierta y fuera de esta fila.

## Verificación

```
cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2 --lib memory_scheduler::tests::interval
→ 2 tests run: 2 passed          (RED antes del fix: left 5000 · right 60000)
cargo clippy -p vantadb-mcp --all-targets -- -D warnings   → limpio
rg 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' vantadb-mcp/src → 0 (el literal sobrevive solo en doc + log + `set_var` de los tests)
```

## Deuda declarada

- **ADR-0063 D4** sigue pidiendo loop **OFF** sin la variable; el host arranca **ON** a 5 000
  (opción A, `WIRE-19-impl` §8.0). Enmienda del ADR: arch/owner, no del implementador.
- `WIRE-21` (`runner_factory` en `IngestRunnerCfg`) y `WIRE-22` (`vantadb-server/*` + docs)
  siguen abiertas; esta fila sólo cierra **la mitad del intervalo** del host MCP.
