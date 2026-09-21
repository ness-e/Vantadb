---
title: "ADR-027: Fase 4 completada — cierre de deuda REST, WASM/OPFS backbone y reconciliación documental"
type: adr
status: accepted
tags: [vantadb, architecture, adr, fase4, rest, wasm, opfs, reconciliation]
created: 2026-08-20
last_reviewed: 2026-08-20
---

# ADR-027: Fase 4 completada — cierre de deuda REST, WASM/OPFS backbone y reconciliación documental

## Context

El plan `2026-08-19-vanta-studio-fase4.md` (18 tareas, waves W0-W4) cerró la
deuda documental/REST de la Fase 3 y construyó el backbone WASM/OPFS + 3
diferenciadores del research. Las decisiones D13 (alcance F4), D14
(reconciliación documental) y D15 (auth del dashboard) se aprobaron al planear;
este ADR registra el cierre real y el estado final del contrato.

## Decision

### D13 — Alcance Fase 4: cerrado en su totalidad

| Sub-fase | Entregable | Cierre |
|----------|-----------|--------|
| F4.0 | Reconciliación documental (registro canónico) | ✅ DOC-01..04 (lead, wave W0): Backlog P26, task files, plan, changelog reconciliados |
| F4.1 | Deuda REST | ✅ REST-01..06: rate limiter sin escape, `/api/v2/metrics` JSON, graph_v2 (u128-safe), paginación cursor, namespace_stats, IQL server |
| F4.2 | WASM/OPFS backbone | ✅ WASM-01..04: cuota verificada, backend wasm, persistencia OPFS con reload, drag&drop import |
| F4.3 | 3 diferenciadores del research | ✅ FEAT-01..03: slider de pesos híbridos (RRF weighted client-side), superficie Índices/salud real, consolidación asistida + supersession durable (ADR-028) |

### D14 — Reconciliación documental: ejecutada

Backlog P26: filas obsoletas (18/19) reemplazadas por las tareas reales de la
fase; task files stale recreados en `.opencode/skills/campaign-executor/tasks/`;
plan F1 renombrado al archivo canónico `2026-08-19-vanta-studio-fase4.md`;
Fase 0 (auth, bind 127.0.0.1, refuerzo de invariantes) registrada en progreso.

### D15 — Auth del dashboard: mantenida local-first

Se mantiene **sin auth** la consola web/WASM: bind 127.0.0.1, sin token para
loopback. MEM-05 (auth 3 capas) sigue siendo workstream aparte si algún día se
expone fuera de loopback. Confirmado por los E2E de VER-01: ambos modos
(dashboard servido + consola standalone) operan contra el server local.

### Deuda REST: cerrada (verificada por VER-01 E2E)

- **REST-01:** ráfaga sin 429 con el rate limiter default (600 rpm, burst completo).
- **REST-02:** `/api/v2/metrics` devuelve snapshot JSON operacional (no Prometheus text).
- **REST-03:** graph_v2 con ids u128-safe en string (`VantaGraphTraversalResult`) —
  roundtrip verificado con root > u64::MAX (18446744073709551617).
- **REST-04:** paginación `{records, next_cursor}` en search.
- **REST-06:** IQL completo vía `/api/v2/query` (SELECT + INSERT + roundtrip graph).
- Gap conocido (documentado, NO en contrato): `QueryResponse.node_id` del endpoint
  IQL legacy serializa `u128` como número JSON → JS pierde precisión > 2^53.
  Los endpoints v2 del console (search/graph_v2) ya usan `u128_serde`/id-string.

### WASM/OPFS backbone: cerrado

`connect_persistent` + `save()` contra OPFS (secure context 127.0.0.1), CRUD
persistente con reload real, import drag&drop de `.vdbdump`/JSONL/CSV.
Limitación documentada: el bundle Playwright no expone `navigator.storage`
(getDirectory no disponible — WASM-01 midió la cuota por otra vía).

## Consequences

- **Pros**
  - Deuda documental y REST de F3 cerrada con verificación E2E automatizada
    (`desktop/scripts/selfcheck-web-e2e.ts` ampliado a REST-02/03/06).
  - La consola corre en ambos modos (server + standalone WASM) — el cliente
    pierde la dependencia de un server para el caso local.
  - Los 3 diferenciadores del research (pesos, índices/salud, consolidación)
    están en superficie real, no mock.
- **Cons**
  - El endpoint IQL legacy (`/api/v2/query`) mantiene `node_id` numérico en el
    wire (gap u128>2^53) — no se rompió nada porque los clientes v2 no lo usan,
    pero es una trampa si alguien lee node_id del body de un INSERT.
  - Auth del dashboard sigue diferida a MEM-05 (aceptado: local-first).

## Alternatives Considered

- Arreglar `QueryResponse.node_id` (u128_serde) en VER-01: rechazado — es un
  cambio de wire en un endpoint legacy fuera del contrato de la fase; el
  roundtrip del E2E usa el id string exacto del INSERT. Se trackea como follow-up.
- Auth en VER-01: rechazado — D15/D12 vigentes; MEM-05 ya existe como workstream.

## References

- Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (D13:13, D14:14, D15:15, VER-01 Task 18:188).
- E2E: `desktop/scripts/selfcheck-web-e2e.ts` (REST-02/03/06), `desktop/scripts/selfcheck-wasm-e2e.ts`.
- ADR-028: supersession durable (core decay, FEAT-03b).