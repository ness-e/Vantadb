---
title: "Batch de parciales y filas nuevas - ejecución completa por waves"
kind: plan
status: active
description: "Plan de ejecución de los 33 IDs recibidos + 2 del plan del release el 2026-11-26. Todas las tareas entran al plan y se ejecutan; se agrupan en waves por partición de ficheros. Los prerequisitos externos (credenciales, autorización de publish) se declaran por tarea, no se usan para descartarla."
tags: [pipeline, backlog, waves]
---

# Batch 2026-11-26 — 33 tareas, ejecución completa

**Fuente:** lote de 34 IDs recibido en `/pipeline` (22 `🟡 Parcial` + 12 filas nuevas de la sesión).
**Estado de partida:** `origin/develop` = `c8e73e412`; gate de docs operativo y **required** (14 contexts en `develop`, 19 en `main`).
**Regla del lote:** **todas las tareas se ejecutan y se quedan en este plan.** No hay `SKIP` ni `DEFER` por criterio del orquestador. Lo que se declare son **prerequisitos** (algo que hay que tener o que un tercero debe autorizar), que se resuelven o se esperan — no descartan la tarea.

**Restricciones operativas aprendidas (no son triage):**
- **2 carriles Rust** compilando a la vez (RAM + ~7 GB de `incremental` por agente).
- **1 carril por fichero de workflow** (`.github/workflows/*.yml` es recurso único: `ci-rust.yml` lo edita una tarea, no dos).
- **1 solo carril en `src/**`** cuando la tarea es un barrido ancho (`DEAD-01`, `FIND-759`).
- Commits **locales**, `GIT_INDEX_FILE` temporal, verificación **por contenido**, nunca `--amend`, push sólo con instrucción explícita.

## Las 35 tareas y su wave

| # | ID | Wave | Carril | Ficheros / zona exclusiva | Prerequisito declarado |
|---:|---|---|---|---|---|
| 1 | `FIND-758` | **W1** | R | `src/storage/engine/delete.rs` · `src/shred/**` | — |
| 2 | `FIND-770` | **W1** | CI | `.github/workflows/ci-rust.yml` | — |
| 3 | `FIND-756` | **W1** | A | `docs/dev/architecture/adr/**` · task file | decisión de `vanta-arch` |
| 4 | `FIND-767` | W2 | CI | `.github/workflows/certification.yml` + `CI_POLICY.md` | tras W1 (`FIND-770` libera `ci-rust.yml`) |
| 5 | `FIND-766` | W2 | CI | `.github/workflows/certification.yml` | después de `FIND-767` (mismo fichero) |
| 6 | `FIND-757` | W2 | R | `vantadb-mcp/src/proxy.rs` | — |
| 7 | `FIND-762` | W3 | CI | `.githooks/pre-push` · `ci-rust.yml` | tras W1 |
| 8 | `FIND-764` | W3 | CI | `.github/workflows/ci-rust.yml` | tras `FIND-762` (mismo fichero) |
| 9 | `FIND-771` | W3 | CI | `.github/workflows/docs-gates.yml` | — |
| 10 | `FIND-768` | W3 | CI | `.github/workflows/snapshot-certification.yml` | — |
| 11 | `FIND-191` | W4 | R | `tests/api/**` · `tests/public_api*` | — |
| 12 | `FIND-215` | W4 | R | `Cargo.toml` (features) · medir | — |
| 13 | `FIND-219` | W4 | C | `docs/**` (matriz) + `.github/workflows/**` | tras W3 (workflows libres) |
| 14 | `DEAD-01` | W5 | R | `src/**` (barrido, carril único) | — |
| 15 | `FIND-759` | W5 | R | `src/**` (`unsafe`, carril único) | tras `DEAD-01` |
| 16 | `FIND-158` | W5 | R | deuda API W2 (ver task file) | — |
| 17 | `FIND-159` | W5 | R | deuda API W3 (ver task file) | — |
| 18 | `FIND-290` | W6 | A→R | `vanta-memory` reflexión + `src/**` | decisión de `vanta-arch` primero |
| 19 | `WIRE-19` | W6 | R | `vanta-memory` + `vantadb-mcp` (según `ADR-0063`) | — |
| 20 | `FIND-320` | W6 | A | contrato `retrieve()` trust-aware | decisión de `vanta-arch` (ADR) |
| 21 | `EGO-13` | W6 | R | ingesta zero-copy (`PyBuffer`/`Float32Array`) | — |
| 22 | `EGO-15` | W6 | R | `graphragSearch` (`max_degree_per_node`) | — |
| 23 | `FIND-598` | W4 | T | perfilado p99 cifrado | herramienta de perfilado + ventana de medida |
| 24 | `FIND-607` | W6 | R | 18 hallazgos P2 — **partir antes de ejecutar** | — |
| 25 | `FIND-120` | W7 | — | onboarding auto-index | **spec SDD** antes de partir |
| 26 | `FIND-121` | W7 | — | cache de embeddings con TTL | **spec SDD** |
| 27 | `FIND-122` | W7 | — | router determinista de retrieval | **spec SDD** |
| 28 | `FIND-198` | W7 | R | runner BEAM opcional | decisión de tamaño/dependencia |
| 29 | `FIND-118` | W1 | C | alias legacy `vantadb_py` | **verificar estado real primero**: si la ventana 0.6.0 ya lo ejecutó, se cierra con evidencia |
| 30 | `DIST-18` | W8 | L | publicar `vanta-proxy` | **secuencia**: post-1.0.0 + autorización de publish del owner |
| 31 | `NOT-06` | W8 | D | 71 páginas de Notion | **credenciales/permiso del workspace del owner** |
| 32 | `GTM-05` | W8 | D | landing `vantadb.dev` | **DNS/dominio + cuenta Vercel del owner** |
| 33 | `GTM-09` | W8 | D | `/enterprise` + one-pager | **precio público**: decisión del owner |
| 34 | `FIND-689` | **W9** | L | guardrail de transferencia `ness-e/Vantadb` → `NegentMind/vantadb` | **Trusted Publishing OIDC en crates.io** + decisión de transferencia del owner |
| 35 | `FIND-704` | **W9** | A→R | `src/config.rs` (tipo `ModelLocation`) + `src/llm.rs` · `src/embedding_health.rs` | decisión de `vanta-arch` sobre strict vs lenient |

> Las filas 30–33 **se ejecutan hasta donde no requieran el clic del owner**: contenido, copy, estructura de página, plantilla de Notion, `vercel.json`, borradores listos para publicar. Lo que queda pendiente se lista explícitamente como "espera al owner" y **la tarea sigue en el plan**.

## Contrato de cierre (idéntico para las 33)

1. `pwsh dev-tools/ocr-review.ps1` + review P2-01 por agente distinto, registrado en `§Review` del task file.
2. Fila fuera del `Backlog.md` + entrada en `docs/dev/avance/<dominio>.md` **con la medición**, no con la intención.
3. Commit **local**: `& ./dev-tools/git-commit-isolated.ps1 -Message "…" -Paths … -ExpectAbsent '<path>::| \`<ID>\` |'`.
4. **Verificación por contenido** post-commit (exit code no basta: `FIND-753`).
5. **No** `gen-index.mjs` por tarea (se regenera al cerrar cada wave). **No** `--amend`. **No** `.tmp-*`.
6. **No** push: lo agrupa el orquestador al cerrar wave, y sólo con instrucción explícita.

## Riesgos declarados

- **`docs/pipeline-state.json` está obsoleto** (última escritura 2026-11-24, con 4 `running` muertos). No se usa como fuente: la verdad es el Backlog + git.
- Esfuerzos del Backlog **no re-medidos**: si una tarea se sale del appetite, se reporta y se parte — no se estira ni se abandona.
- **`FIND-607` son 18 hallazgos** en un solo ID: se parte en sub-tareas antes de ejecutar, o el cierre mentirá sobre lo hecho.

## Residual incorporado (2026-11-26): 15 filas abiertas que no estaban en el plan

El owner detectó que **15 filas abiertas no figuraban en el plan**: todas nacieron de esta sesión
(derivadas de cierres) y se quedaron fuera al crearlo. **Regla: toda fila abierta va en el plan**, o no se ejecuta nunca.

| Wave | Filas | Zona / partición |
|---|---|---|
| **W10** | `FIND-787` · `FIND-789` · `FIND-788` · `FIND-791` | `.github/workflows/**` (una por fichero) + `dev-tools/` + Backlog |
| **W11** | `FIND-782` · `FIND-783` | `vantadb-mcp/src/proxy.rs` · `vanta-memory/src/services/scheduler.rs` + `vantadb-mcp/src/threads.rs` |
| **W12** | `FIND-774` · `FIND-775` · `FIND-777` · `FIND-781` | `.opencode/task-system/mcp/**` (parser y campaign) + `.opencode/` prompts |
| **W13** | `FIND-772` · `FIND-773` · `FIND-779` · `FIND-790` | `vanta-memory/src/core/**` · `src/sdk/api/graph.rs` · `dev-tools/check-harness-contract.mjs` |
| **W14** | `FIND-737` | `.github/workflows/ci-rust.yml` (lane TSan) + `-Zbuild-std` |

**Partición:** un agente por wave; dentro de una wave, ficheros disjuntos. Las waves 10-13 son
**disjuntas entre sí**, así que pueden correr en paralelo hasta el cupo de recursos
(2 Rust compilando · 1 agente por fichero de workflow · disco).
