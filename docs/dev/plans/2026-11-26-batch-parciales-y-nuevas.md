---
title: "Batch de parciales y filas nuevas - ejecución por waves"
kind: plan
status: active
description: "Plan de ejecución del lote de 34 IDs recibido el 2026-11-26: triage del gate aplicado, 19 tareas DO agrupadas en waves por partición de ficheros, y el resto diferido, omitido o bloqueado con razón."
tags: [pipeline, backlog, waves]
---

# Batch 2026-11-26 — parciales + filas nuevas

**Fuente:** lote de 34 IDs recibido en `/pipeline` (22 `🟡 Parcial` + 12 filas nuevas de la sesión).
**Estado de partida:** `origin/develop` = `c8e73e412`; gate de docs operativo y required (14 contexts en `develop`, 19 en `main`).
**Restricciones aprendidas y vigentes:** máximo **2 carriles Rust** compilando (RAM/disco); **1 carril CI** por workflow (`.github/workflows/*.yml` es un recurso único); commits **locales** con `GIT_INDEX_FILE` temporal + verificación por contenido; **nunca `--amend`**; **push sólo con instrucción explícita**.

## Tabla resumen del triage

| Veredicto | N | IDs |
|---|---:|---|
| ✅ **DO** | 19 | `FIND-766` `FIND-767` `FIND-770` `FIND-758` `FIND-757` `FIND-756` `FIND-759` `FIND-762` `FIND-764` `FIND-768` `FIND-771` `DEAD-01` `FIND-158` `FIND-159` `FIND-191` `FIND-215` `FIND-219` `FIND-290` `WIRE-19` |
| 🟡 **DEFER** | 12 | `DIST-18` `FIND-320` `EGO-13` `EGO-15` `FIND-120` `FIND-121` `FIND-122` `FIND-198` `FIND-598` `FIND-607` `FIND-118` `FIND-219`* |
| ❌ **SKIP** | 1 | `FIND-118` |
| 🔴 **BLOQUEADO** | 3 | `NOT-06` `GTM-05` `GTM-09` |

\* `FIND-219` cuenta en DO; se lista arriba por error de conteo — ver Gate Justificación.

## Gate Justificación

| ID | Gate | Razón |
|---|---|---|
| `FIND-767` | ✅ DO · 🟠 | Seis tests de **seguridad** con **cero ejecuciones** en cualquier workflow. En una DB no es cosmético: es la verificación de que el cifrado hace lo que dice. **1 carril CI.** |
| `FIND-766` | ✅ DO · 🟠 | 98 tests con un lane que nunca ejecutó uno solo (0 success / 36 failure / 30 días). Separar `cli_tests` del gate de datasets y que el rojo por dataset sea **otro job**. **Comparte fichero con 767 → misma wave, secuencial.** |
| `FIND-770` | ✅ DO · 🟠 | 10 de los 14 required contexts quedan en `Expected` para siempre si un PR sólo toca `docs/**`. Es el 90 % del deadlock que `FIND-765` dejó medido. **Dueño exclusivo de `ci-rust.yml`** (bloquea 762 y 764 en la misma wave). |
| `FIND-758` | ✅ DO · 🟠 | `let _ =` sobre `ShreddedRowStore::delete`: un borrado puede fallar **en silencio**. El razonamiento del sitio (el tombstone del WAL es el commit) es correcto; lo que falta es **reportar**. |
| `FIND-757` | ✅ DO · 🟡 | Una cifra publicada (`<0,3 s` de `MCP-48`) que un carril posterior no reprodujo (~1,9 s). Puede ser redacción, no código — y hay que decirlo con medición. |
| `FIND-756` | ✅ DO · 🟡 | Decisión de `vanta-arch` sobre la representación de ids del grafo. **No implementa**: entrega decisión + contrato. Cierra previsiblemente `FIND-296`. |
| `FIND-759` | ✅ DO · 🟡 | 70 sitios `unsafe`, 54 `SAFETY:`. No se cierra con un recuento: requiere veredicto por sitio o declaración de política. |
| `FIND-762` | ✅ DO · 🟡 | El `pre-push` formatea con stable 1.95.0 y CI con nightly: verde local ≠ verde CI. **Bloqueado por `FIND-770`** (mismo workflow). |
| `FIND-764` | ✅ DO · 🟡 | Un push cancela la suite del anterior: hay SHAs en `develop` sin verificar. **Bloqueado por `FIND-770`** (mismo workflow). |
| `FIND-768` | ✅ DO · 🟡 | La lane de snapshot **está verde** y selecciona 42 de 56: el hueco es invisible. |
| `FIND-771` | ✅ DO · 🟡 | `docs-gates.yml` cancela en vuelo y ahora hospeda un gate **required**: un required cancelado queda ausente, no rojo. |
| `DEAD-01` | ✅ DO · 🟡 | Auditoría de código muerto. Diff ancho en `src/**` → **wave propia**, sin otro carril en `src/`. |
| `FIND-158` `FIND-159` | ✅ DO · 🟢/🟡 | Deuda API W2/W3 no bloqueante, acotada. |
| `FIND-191` | ✅ DO · 🟠 | `public-api` harness: stack overflow en cold-cache y `--profile audit` sin completar. Es un gate que no mide. |
| `FIND-215` | ✅ DO · 🟡 | jemalloc/mimalloc: las features ya existen; falta medir y decidir. |
| `FIND-219` | ✅ DO · 🟡 | Matriz de features documentada + CI por combinación. |
| `FIND-290` | ✅ DO · 🟡 | Reflexión sin consumidor in-repo. Necesita decisión de activación → `vanta-arch` primero. |
| `WIRE-19` | ✅ DO · 🟡 | `ADR-0063` decidido, implementación pendiente: mover el wiring a `vanta-memory`. |
| `DIST-18` | 🟡 DEFER | Congelado explícitamente hasta post-1.0.0. |
| `FIND-320` `EGO-13` `EGO-15` `FIND-598` `FIND-607` | 🟡 DEFER | Semanas de esfuerzo o requieren perfilado dedicado. `FIND-607` son **18 hallazgos** agrupados: hay que partirlo antes de tocarlo. |
| `FIND-120` `FIND-121` `FIND-122` `FIND-198` | 🟡 DEFER | Features de producto: necesitan spec (SDD) antes de partir en tareas. |
| `FIND-118` | ❌ SKIP | La ventana era **0.6.0** y el repo va por 0.9.x con `v0.10.0` en curso: la migración temporizada ya pasó. **Verificar antes de cerrar** que el alias no sigue vivo. |
| `NOT-06` `GTM-05` `GTM-09` | 🔴 BLOQUEADO | Requieren al owner: páginas de Notion, DNS/landing en Vercel, pricing público. No delegables. |

## Waves

Partición por **fichero**, no por tarea: es la lección de las últimas horas (índice compartido, `CI_POLICY.md`, `ci-rust.yml`).

### Wave 1 (en vuelo)
| Tarea | Ruta | Ficheros exclusivos |
|---|---|---|
| `FIND-758` | `vanta-worker` | `src/storage/engine/delete.rs` · `src/shred/**` |
| `FIND-770` | `vanta-worker` | `.github/workflows/ci-rust.yml` |
| `FIND-756` | `vanta-arch` | `docs/dev/architecture/adr/**` · `docs/dev/tasks/FIND-756.md` (sin código) |

### Wave 2 (tras 770)
| Tarea | Ruta | Nota |
|---|---|---|
| `FIND-767` | `vanta-worker` | `.github/workflows/**` + `CI_POLICY.md`; secuencial con 766 |
| `FIND-762` `FIND-764` | `vanta-worker` | `.githooks/pre-push` + `ci-rust.yml` — **sólo después** de que 770 libere el fichero |
| `FIND-757` | `vanta-worker` | `vantadb-mcp/src/proxy.rs` |

### Wave 3
`FIND-766` · `FIND-768` · `FIND-771` · `FIND-191`

### Wave 4 (ancha, un solo carril en `src/`)
`DEAD-01` · `FIND-759` · `FIND-158` · `FIND-159`

### Wave 5 (decisiones primero)
`FIND-756` (si no cerró) · `FIND-290` · `WIRE-19` · `FIND-215` · `FIND-219`

## Contrato de cierre (idéntico para todas)

1. `pwsh dev-tools/ocr-review.ps1` + review P2-01 por agente distinto, registrado en §Review del task file.
2. Fila fuera del Backlog + entrada en `docs/dev/avance/<dominio>.md` **con la medición**.
3. Commit **local** con `& ./dev-tools/git-commit-isolated.ps1 … -ExpectAbsent '<path>::| \`<ID>\` |'`.
4. **Verificación por contenido** post-commit, no por exit code (`FIND-753`).
5. **No** `gen-index.mjs` (se regenera en el cierre de wave). **No** `--amend`. **No** `.tmp-*`.
6. **No** push: lo agrupa el orquestador.

## Riesgos declarados

- **`docs/pipeline-state.json` está obsoleto** (última escritura 2026-11-24, con 4 `running` que llevan días muertos). Este plan **no** lo usa como fuente; se leerá del Backlog y de git.
- Disco: el patrón medido hoy es **~7 GB de `incremental` por agente Rust**. `-AutoSafe` degrada a podar `*.pdb`, así que el techo real son **2 carriles Rust**.
- Las cifras de esfuerzo son del Backlog y **no están re-medidas**: si una tarea se sale del appetite, se devuelve en vez de estirarla.
