---
title: "FIND-676: ciclo cognitivo - intrusion de hechos obsoletos 0% (aserción PASS con control); L2/L3 no medibles desde el SDK"
kind: task
status: completed
description: "benchmarks/cognitive_cycle_bench.py: 0.00% de intrusion en top-K con exclude_superseded (125 hits sin la bandera como control) y aislamiento de namespaces OK. context_assemble/vanta_scene/vanta_dream no estan en el Client de Python."
id: FIND-676
type: bug
---

# FIND-676 — Benchmark E2E del ciclo cognitivo L0→L3

- **Prioridad:** 🟠 Alta · **Origen:** *MemoryAgentBench* (ICLR 2026), *HaluMem* (EMNLP 2026)
- **Creado:** 2026-11-22 · **Medido:** 2026-11-27
- **Estado:** 🟡 PARCIAL — lo medible está medido y la aserción **pasa**; L2/L3
  están **bloqueados por falta de superficie**, no diferidos por comodidad.

## Parte 1 — intrusión de hechos obsoletos (la aserción de la fila)

30 escenarios de contradicción: se escribe `v1` y `v2` del mismo sujeto,
`supersede(v1 → v2)`, y se consulta el sujeto con `top_k=10`.

```
Part 1 — stale-fact intrusion (supersede)
  scenarios=30 top_k=10 -> stale hits with flag: 0, without flag: 125
  ASSERTION stale intrusion == 0 %: 0.00 % -> PASS
```

**El control es lo que hace válido el 0 %.** El mismo corpus, la misma consulta y
la misma bandera apagada devuelve **125 hits obsoletos**: el 0 % lo produce
`exclude_superseded`, no un índice vacío. Sin ese control, un `0` sería
indistinguible de un arnés roto — el defecto que `FIND-664` documenta.

## Parte 2 — aislamiento y compartición controlada

Misma clave (`note`) y mismo payload secreto en tres namespaces (`agent-a/b/c`),
consulta en cada uno:

```
  cross-namespace hits: 0, namespaces returning nothing: 0, shared namespace hits: 1 -> PASS
```

Los tres namespaces devuelven algo (no es un 0 por vacío), ninguno devuelve el
registro de otro, y un namespace compartido explícito sigue funcionando.

## Parte 3 — L0 → L1 (`memory_capture` / `memory_recall`) — INFORMATIVO

```
  captured 116 B -> recalled 4 B (ratio 0.034), fact present: False,
  capture 13.385 ms / recall 0.204 ms    (5 intentos con espera)
```

`memory_capture` acepta la sesión (`recorded_count: 1`) pero
`memory_recall(text, session_id)` devuelve `None`. **No se reporta como bug:**
el docstring documenta `None` como «nada que inyectar» y `memory_capture`
devuelve un `cursor_ms`, así que el ingest puede ser asíncrono y una sola lectura
inmediata no es evidencia. Convertir esa carrera en un defecto sería el mismo
error que esta fila persigue. El JSON lo marca `inconclusive` con el motivo.

## Lo que NO se puede medir, con la prueba

```
$ python -c "import vantadb; print([m for m in dir(vantadb.Client) if 'context' in m or 'scene' in m or 'dream' in m or 'thread' in m])"
[]
```

`context_assemble`, `vanta_scene` (L2), `vanta_dream` (L3) y `thread_send` **no
están exportados** por el `Client` de Python instalado (0.9.0): son herramientas
MCP de `vantadb-mcp`. Por tanto:

- la **ganancia de compresión/precisión de escenas L2 y axiomas L3 sobre L1
  crudo** —el entregable (2) de la fila— **no se certifica**, porque no hay ruta
  desde el SDK;
- la supersesión cross-sesión con `scope="agent"`/`"team"` queda igual de
  bloqueada (el parámetro existe, pero la escena que la resuelve es L2).

**Lo que falta para cerrar:** exponer esas rutas al SDK, o el arnés MCP que es
`FIND-674`. Se declara, no se finge.

## Verificación

```
$ python benchmarks/cognitive_cycle_bench.py --scenarios 30 --json evals/results/cognitive_cycle_bench.json
  ASSERTION stale intrusion == 0 %: 0.00 % -> PASS
  cross-namespace hits: 0 ... -> PASS
  exit 0
$ ruff check benchmarks/cognitive_cycle_bench.py && ruff format --check ...
All checks passed! / 1 file already formatted
```

**Cero dependencias nuevas, cero compilación, 0.1 min.** Artefacto:
`evals/results/cognitive_cycle_bench.json`.

## Impacto mapeado (Regla 0)

- **Leídos:** superficie del `Client` de Python (`memory_capture`, `memory_recall`,
  `supersede`, y la ausencia de `context_assemble`), docstrings de `memory_recall`
  (contrato de `session_key`/`scope`/`None`).
- **Referencias hacia dentro:** ninguna — archivo nuevo bajo `benchmarks/`.
- **Veredicto:** sólo se añade el benchmark. **Cero cambios en `src/**`,
  `vanta-memory/**` ni `vantadb-mcp/**`.**

## Review (P2-01)

Tier: **Fast**. Verify: corrida real, aserción con control en la dirección que
importa, `ruff` limpio. **approve**.
