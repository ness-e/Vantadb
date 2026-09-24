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
