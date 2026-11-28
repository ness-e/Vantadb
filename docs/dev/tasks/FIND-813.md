---
title: "FIND-813 — una rama es estado global del árbol de trabajo"
kind: task
status: completed
description: "Un carril hizo `git checkout -b test/find787-failpath` y cuatro commits de tres carriles aterrizaron ahí. La regla queda escrita en AGENTS.md Regla 7: rama propia = worktree propio, ningún prompt pide checkout, CI en rama se crea desde el remoto, y el orquestador declara y re-verifica la rama antes/después de cada wave."
id: FIND-813
tags: [vantadb, harness, git, worktree, paralelismo]
---

# FIND-813 — una rama es estado global del árbol de trabajo

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-813`)
- **Origen:** prueba del fail-path del guard (2026-11-27) · **Ver:** `FIND-792`, `FIND-780`, `FIND-787`
- **Esfuerzo:** 🟢 30-60 min (regla; la implementación en código no estaba en el contrato)
- **Estado:** ✅ COMPLETED (local, sin push — Regla 7)

## Premisa (re-medida en HEAD)

El árbol sigue en `develop` y ningún prompt mueve el HEAD:

```
git branch --show-current                                  → develop
rg -n 'git checkout|git switch' .opencode/task-system/prompts
  → subagent-recovery.md:62: "- Nunca `git checkout`/`reset --hard` sobre el trabajo del sub-agente."
  → 1 hit, y es una PROHIBICIÓN: hoy ningún prompt pide cambiar de rama.
```

Lo que faltaba no era el estado del árbol sino la **regla**: el incidente (cuatro commits de tres
carriles en `test/find787-failpath`) no violó ninguna norma escrita porque la norma no existía. El
`Backlog.md` compartido tiene su fila (`FIND-792`) y el índice compartido tiene la suya (`FIND-753`,
índice privado del helper); el HEAD no estaba en esa lista.

## Contrato entregado (regla, no código)

`.opencode/AGENTS.md` → **Regla 7 §Ramas y árbol de trabajo** (§ nueva, tras «Política de git
asistido por agentes»):

| Regla | Obligación |
|---|---|
| R1 | Un carril que necesite rama propia **necesita worktree propio** (`git worktree add`); no corre en paralelo con nadie sobre el árbol principal. |
| R2 | **Ningún prompt puede pedir `checkout`/`switch`** — ni crear ni cambiar. Si aparece, el review lo rechaza. |
| R3 | Para probar CI en una rama: **crearla desde el remoto** (`git push origin develop:refs/heads/test/…`), sin tocar el árbol. |
| R4 | El orquestador **declara** la rama (`git branch --show-current`) antes de despachar una wave y **la verifica al cerrar**. |

La regla lleva su verificación mecánica en el mismo bloque (las dos órdenes de arriba), para que el
review tenga un comando y no una promesa.

## Steps

1. ✅ Cargar contexto: `AGENTS.md` Regla 7, `FIND-792`/`FIND-753` (las otras dos caras del mismo
   estado global), `subagent-recovery.md:62`.
2. ✅ Medir que hoy no hay prompt que pida `checkout`/`switch` (arriba).
3. ✅ Escribir R1-R4 + verificación en Regla 7.
4. ✅ Gate del harness: 9 checks + `--self-test` 89/89 (sin tocar el validador).

## Contrato de verificación

```
node dev-tools/check-harness-contract.mjs --root .    → harness contract holds (9 checks)
node dev-tools/check-harness-contract.mjs --self-test → 89/89 self-test fixtures passed
git branch --show-current                             → develop
```

**Deuda declarada (fuera de alcance):** la regla **no tiene gate mecánico** propio todavía — la
verifica el review, no un check. Lo natural es una fixture en `check-harness-contract.mjs` que
falle si un prompt de `.opencode/task-system/prompts` contiene `git checkout`/`switch` sin ser una
prohibición; pide tocar el validador (89/89 + fixtures que fallen contra `HEAD`) y esta fila era de
contrato, no de herramienta.

## Review

- **Ronda P2-01:** tier **Fast** (`docs/dev/**`, `.opencode/**`, sin código): verify mecánico
  (`check-harness-contract` 9/9 + 89/89) y spot-check del orquestador.
- **Scope:** solo `.opencode/AGENTS.md`; código del core, workflows y `dev-tools/check-harness-contract.mjs` intactos.
