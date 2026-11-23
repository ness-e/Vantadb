---
title: Roadmap v0.7 → v0.8.0 → v1.0 — Gobernanza del ciclo de vida
kind: concept
status: active
description: "Re-baseline 2026-09-30: 0.7.0 publicado; gobernanza manual + migración única = corte 0.8.0 (ADR-046); v1.0 = automática"
tags: [vantadb, roadmap, governance]
---

# Roadmap v0.7 → v0.8.0 → v1.0 — Gobernanza del ciclo de vida

> Fuente: Notion Propuesta §4 (gobernanza manual → automática). **Re-baseline 2026-09-30:**
> **0.7.0 publicado 2026-09-25** (crates.io/PyPI/npm); la **migración única de esquema + gobernanza
> manual** quedan en el corte **0.8.0** (ADR-046, preparado localmente — push/release = lane owner);
> **v1.0** = gobernanza automática. Este archivo registra el alcance acordado; no es plan de
> ejecución (eso vive en `docs/dev/Backlog.md` §DELTA + `docs/dev/plans/2026-09-26-master-roadmap.md`).

## Estado real (2026-09-30)

- **0.7.0 (2026-09-25):** publicado — crates.io `vantadb` 0.7.0 · PyPI `vantadb-py` 0.7.0 · npm `vantadb`/`vantadb-wasm` 0.7.0 (incluye API-01..09 + WIRE-01..11 + estabilización).
- **0.8.0 (preparado):** migración única de esquema v2 (bitemporal + confianza + cuarentena) — **SCH-01..08 ✅** (F3), **VER-01..08 ✅** (F4/F5), **ICP-01..03 ✅** (F5); guía `docs/user/operations/UPGRADE.md` §0.8.0 + marcador breaking `b9296909` listos. Push/release = lane owner (PROC-01).
- **v1.0:** gobernanza automática.

## Gobernanza manual (corte 0.8.0 — plan original "v0.7")

Capacidades con docs + tests de integración, operadas por el usuario:

- `mark_duplicate` — **[PROPUESTA]** (MGR-05 + MEMG-09; `WIRE-05` ✅ base de entity linking).
- `detect_conflicts` — **[PROPUESTA]** (MGR-06 + MEMG-01).
- `extract_skills` — **[PROPUESTA]** (MGR-07; `skill_extract` solo-candidatos [REAL], FIND-111).
- Base ya real: TTL/supersession/namespaces + **schema v2** (SCH-02/04/05 ✅) + **AS OF** (SCH-03 ✅) + cuarentena/abstención (SCH-05 ✅).

> Nota 2026-09-30: la gobernanza manual quedó **parcialmente cubierta** por el esquema v2 + cuarentena + importadores (VER-05); las 3 capacidades nominales siguen [PROPUESTA] y se trackean en MGR/MEMG (DELTA).

## v1.0 — Gobernanza automática

- `auto_resolve_entities` (con fusión reversible) — **[PROPUESTA]** (MGR-05).
- `resolve_conflict` / promoción sin intervención — **[PROPUESTA]** (MGR-06/08).
- `consolidate(ns)` automático — **[PROPUESTA]** (MGR-08 + VER-07 ✅ base).

## Regla de marcas (de Propuesta)

Toda capacidad lleva marca: `[REAL]` (corre hoy, con evidencia en código), `[PARCIAL]`, `[PROPUESTA]`. Nunca presentar PROPUESTA como REAL (ver backlog-notion N-02: `auto_resolve_entities`/`detect_conflicts`/`extract_skills` correctamente marcados [PROPUESTA] a 2026-09-24).

## Migración única de esquema (decisión owner 2026-09-24 · ADR-046 firmado)

**MGR-10 (bitemporalidad) + MGR-12 (scores asserted/derived + `last_validated`) + MGR-13 (cuarentena) entran en UN solo breaking change de schema en el corte 0.8.0** (ventana única: nadie usa los paquetes aún).

- Pre-requisito duro: Cierre MGR de los 3 research-docs — ✅ (`docs/dev/research/mgr-10-bitemporalidad.md` / `mgr-12-confianza.md` / `mgr-13-cuarentena.md`).
- Filas: **P53 SCH-01..08 — ✅ ejecutados**; `SCH-09` (edge bitemporal) pendiente. Alcance 0.8.0 = primitivas + superficies; derivación completa/grounding con jueces y automatización total quedan v1.0.
- Referencias: ADR-046 · Zep/Graphiti (arXiv 2501.13956) · Snodgrass · `UPGRADE.md` §0.8.0.

## Verificabilidad (categoría propietaria — ejecutada)

Hueco exclusivo vs 17 sistemas de memoria: **memoria verificable y gobernable**. Filas P52 — **✅ VER-01..09**:

- **VER-01** tamper-evident (hash-chain WAL + `vanta-cli verify`) · **VER-02** borrado certificado · **VER-03** redacción persistida + namespaces cifrados · **VER-04** governance de inyección.
- **VER-05** importadores Mem0/Zep/Letta · **VER-06** export file-native Markdown · **VER-07** dreams dry-run + promote real · **VER-08/VER-09** harness propio + head-to-head.
- Residuales trackeados: FIND-189/194, MEMG-17 (rollback + firmas + erasure).

## Tracks ICP (decisión owner 2026-09-24)

Cada track con métrica propia: **AI-IDEs vía MCP** (sesiones con put+search en 7d) · **local-LLM/privacidad** (0 PII en store) · **frameworks** (installs/semana de adapters). El núcleo (motor de memoria embebido gobernado) es único; los tracks son puertas de entrada. Filas: **P54 ICP-01..03 — ✅ ejecutados**; one-pagers en `docs/user/{AI_IDES,PRIVACY,FRAMEWORKS}.md`; publicación PyPI adapters = lane owner (MKT-20 (ex MKT-18f)).

## Añadido 2026-09-30 — marco 2.0 + adopción core

- **MEMG-11..13:** adopción del motor core por `vanta-memory` · semántica v2 write-side · superficies core restantes.
- **MEMG-14..23:** validación externa (≈100 fuentes) — portabilidad, sharing multi-agente, rollback/verificabilidad/erasure, multimodal, prospectiva, checkpoints, scoring L1, procedencia por campo, evaluación "mejora con experiencia".
- Ver `docs/dev/Backlog.md` §DELTA (55 ítems — fuente del próximo plan).
