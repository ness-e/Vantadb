---
title: Roadmap v0.7 (governance manual) → v1.0 (governance automática)
type: strategy
status: active
tags: [vantadb, roadmap, governance]
last_reviewed: 2026-09-24
aliases: []
---

# Roadmap v0.7 → v1.0 — Gobernanza del ciclo de vida

> Fuente: Notion Propuesta §4 (v0.7 manual, v1.0 automática). Estado del repo:
> v0.6.1 (Early Access). Este archivo registra el alcance acordado; no es
> plan de ejecución (eso vive en `docs/dev/Backlog.md` cuando se creen filas).

## v0.7 — Governance manual

Capacidades con docs + tests de integración, operadas por el usuario:

- `mark_duplicate` — marcar duplicados a mano.
- `detect_conflicts` — detectar contradicciones entre memorias.
- `extract_skills` — extracción de skills con revisión humana.
- Base existente: `skill_extract` solo-candidatos (FIND-111, real),
  TTL/supersession/namespaces (reales).

## v1.0 — Governance automática

- `auto_resolve_entities` (con fusión reversible).
- Resolución y promoción sin intervención.
- `consolidate(ns)` automático (hoy [PROPUESTA]).

## Regla de marcas (de Propuesta)

Toda capacidad lleva marca: `[REAL]` (corre hoy, con evidencia en código),
`[PARCIAL]`, `[PROPUESTA]`. Nunca presentar PROPUESTA como REAL
(ver backlog-notion N-02: `auto_resolve_entities`/`detect_conflicts`/
`extract_skills` correctamente marcados [PROPUESTA] a 2026-09-24).

## Migración única de esquema (decisión owner 2026-09-24)

Decisión D2 del plan post-investigación (`docs/dev/plans/2026-09-24-post-investigacion-integral.md`): **MGR-10 (bitemporalidad) + MGR-12 (scores asserted/derived + `last_validated`) + MGR-13 (cuarentena) entran en UN solo breaking change de schema en 0.7.0** (ventana única: nadie usa los paquetes aún).

- Pre-requisito duro: Cierre MGR de los 3 research-docs (P49) antes de tocar `src/sdk/types/record.rs`.
- Filas: **P53 SCH-01..08**. Alcance 0.7.0 = primitivas + superficies; derivación completa/grounding con jueces y automatización total quedan v1.0.
- Referencias: Zep/Graphiti (arXiv 2501.13956), Snodgrass (valid/transaction time).

## Verificabilidad (v0.7 — categoría propietaria)

Hueco exclusivo verificado vs 17 sistemas de memoria: **memoria verificable y gobernable**. Filas P52:

- **VER-01** tamper-evident (hash-chain sobre WAL + `vanta-cli verify`) · **VER-02** borrado certificado (attestation de purga) · **VER-03** redacción persistida + namespaces cifrados · **VER-04** governance de inyección (presupuesto + audit log).
- Paridad de producto: **VER-07** dreams dry-run + promote real · **VER-05** importadores Mem0/Zep/Letta · **VER-06** export file-native Markdown · **VER-08/VER-09** harness propio + head-to-head publicado.

## Tracks ICP (decisión owner 2026-09-24)

Los 3 tracks se desarrollan a profundidad, cada uno con métrica propia: **AI-IDEs vía MCP** (métrica: sesiones con put+search en 7d) · **devs local-LLM/privacidad** (métrica: 0 PII en store) · **frameworks** (métrica: installs/semana de adapters). El núcleo (motor de memoria embebido gobernado) es único; los tracks son puertas de entrada. Filas: **P54 ICP-01..03**; one-pagers vía BIZ-05 (desbloqueada).
