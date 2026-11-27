---
title: "Decisiones del owner 2026-11-27 - autonomia, formatos, ruleset, tareas largas y version"
kind: task
status: active
description: "Las cinco decisiones que desbloquean la automatizacion: push autonomo, cambios de formato pre-1.0, ruleset aplicable, tareas largas con diseno+parcial, y v0.10.0 publica todo."
id: DECISIONES-2026-11-27
---

# Decisiones del owner — 2026-11-27

**Vinculantes para todos los carriles hasta que el owner las cambie.** Respondidas por el owner a las cinco preguntas que bloqueaban la automatización.

## D1 — Push autónomo a `develop`

El orquestador **empuja a `develop` sin pedir permiso** cuando el gate local pase. Un push por wave, esperando su gate.

**Sigue prohibido sin OK explícito:** tags, publishes, merge a `main`, y cualquier acción destructiva.

## D2 — Cambios incompatibles de formato: PERMITIDOS

Estamos en `0.9.x` y los formatos cifrados son nuevos **sin publicar**. Cambiar el formato **cuesta cero ahora**; después cuesta una migración.

**Afecta directamente a:** `FIND-586` (identidad del fichero en el AAD), `FIND-513` (cifrar la clave del posting), `FIND-343` (purga física).

**Sigue exigido:** un test que falle antes del cambio, y **verificar que no hay fixtures viejos** en `tests/`.

## D3 — Ruleset de `develop`: aplicable

El orquestador **puede añadir o ajustar checks obligatorios**, con **backup previo** del JSON y **verificación posterior**. Avisar de cada cambio.

**Motivo:** sin esto, arreglos como `FIND-815` quedan escritos pero dormidos.

## D4 — Tareas que no caben: diseño + parte implementada

No se abandona nada. `FIND-586` (1-2 semanas) y `FIND-343` (3-5 días) se entregan con el **diseño completo** y la **parte que quepa**, y la fila queda con el contrato exacto de lo que falta.

**Lo que NO se hace:** marcar cerrada una fila cuya implementación quedó a medias. Se queda abierta con el delta escrito.

## D5 — Versión: `v0.10.0` publica **todo**

**`DIST-18` queda desbloqueada.** El proxy **sí** se publica en `v0.10.0`; no se espera a 1.0.0. La decisión de congelación de octubre queda **anulada**.

El release sigue necesitando la **firma del owner**: tag y publishes no se ejecutan por automatización.

## Consecuencias operativas

| Antes | Ahora |
|---|---|
| El orquestador pedía permiso por cada push | Empuja solo, un push por wave |
| `FIND-513` sólo podía documentar el límite | Puede cambiar la clave del posting |
| `FIND-815` quedaba propuesto | Se aplica con backup y aviso |
| `FIND-586` era "no cabe" | Diseño + parcial, con contrato del delta |
| `DIST-18` bloqueada hasta 1.0.0 | Se publica en `v0.10.0` |

## Lo que estas decisiones NO cambian

- Un gate que falla **no se salta** (`SKIP` declarado sólo por WIP ajeno, nunca por prisa).
- Un `FIND` nuevo **se resuelve en el acto**.
- Nada se marca cerrado sin su verificación.
- Los dos cuellos reales siguen siendo el **disco** y **una compilación Rust a la vez**.
