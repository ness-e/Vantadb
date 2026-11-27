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

## D6 — Un solo feature set para los carriles (decision del owner, 2026-11-27)

**Todos los carriles compilan con el set del gate:** `cli,fjall,memmap2,fs2,roaring`.

Los tests de `encryption`, `wal-shipping` y `--all-features` pasan a **CI** (`Tests (embeddings features)` y las lanes de features).

**Por qué:** la auditoría de recursos midió **3.727 fingerprints** de `vantadb`, **226** `libvantadb*.rlib` de ~110 MB y **9** builds de `librocksdb-sys` (~4 GB cada uno), porque cada carril compilaba con un set distinto y no compartían ni un objeto.

**Coste aceptado explícitamente:** un bug que sólo aparezca con `encryption` o `wal-shipping` **deja de verse en local** y sólo lo caza CI. El owner lo aceptó para ganar 30-40 % de tiempo de build.

**Excepción:** el carril que trabaje **en** cifrado puede usar `encryption,wal-shipping` **declarándolo** en su task file. Es la única segunda variante admitida.

## D7 — Disco y compilación (aplicado 2026-11-27)

Cinco cambios, todos reversibles, todos medidos antes de aplicarse:

| # | Cambio | Dónde | Ganancia medida | Coste declarado |
|---|---|---|---|---|
| R1 | `CARGO_INCREMENTAL = "0"` | `.cargo/config.toml` | El incremental llegaba a **14,43 GB** creciendo 1,64 GB/25 min con 843 `dep-graph.bin` | Cada edición recompila el crate; **medido: 2,9 min** un build completo de tests de `vanta-memory` |
| R3 | `test-threads = 2` en el perfil `audit` | `.config/nextest.toml` | Commit estaba al **80,7 %**; clase `os error 1455` | **Cambia el perfil de medición**: ya no expone flakes de contención que 12 hilos veían |
| R4 | El fast gate ya no compila el bench en release | `dev-tools/verify.ps1` | Era el paso más caro (`lto=thin`, `codegen-units=1`, `opt-level=3`) | Deja de garantizar que el bench **enlace**; eso pasa al nightly (precedente HARD-02) |
| R5 | `lld-link` en vez del `link.exe` de MSVC | `.cargo/config.toml` | MSVC medía 380-392 MB por link; LLD 22.1.2 ya estaba instalado sin usar | Diferencias de semántica en PDB; **verificado con un link real: exit 0** |
| R8 | Terminar los procesos ociosos | — | 3 `codebase-memory-mcp` quemaban **~1 núcleo permanente** (51 h-núcleo acumuladas) | **Se perdió la herramienta MCP** `codebase-memory-mcp` en esa sesión; declarado |

**No aplicado: sccache.** Necesita 15 GB de caché y hay que esperar a un momento sin dos builds vivos. Queda como la siguiente palanca.

## Lo que estas decisiones NO cambian
