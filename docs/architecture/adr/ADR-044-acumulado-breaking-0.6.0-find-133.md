---
title: "ADR-044: Acumulado breaking develop hacia 0.6.0 — veredicto FIND-133 (sin revert)"
type: adr
status: proposed
tags: [vantadb, architecture, adr, semver, release]
created: 2026-09-19
last_reviewed: 2026-09-19
---

# ADR-044: Acumulado breaking develop hacia 0.6.0 — veredicto FIND-133 (sin revert)

## Context

El job Semver Checks (`cargo semver-checks -p vantadb` vs crates.io 0.5.0) está rojo con
21 categorías / ~100 ítems (FIND-133; baseline local en `Temp/opencode/semver-clean.txt`).
El triage exhaustivo (`docs/tasks/FIND-133.md`, 14 grupos con evidencia código/commits)
concluye que **el 100% es evolución intencional de develop**: de-prefix SDK `Vanta*`
(AST-002/AST-010, `refactor!:`), split de tipos (FIND-49), SearchProfile per-request (MEM-01),
métricas L1/L2/L3/recall (MEM-34), routing flat/IVF/HNSW (OLD-21), remoción PITR muerta
(FIND-26), split config-datos (ADR-043), CacheLayer (C2S3b), rediseño agentic threads,
contrato providers (ADR-033), CLI features, graceful shutdown WAL. Cero accidentales.

El propio CI ya contempla este estado (`ci-rust-10.yml:97-101`: en develop la comparación
es "ruido" mientras se acumula pre-release hacia 0.6.0; el job solo corre en main/PR-a-main).
`cargo-semver-checks --release-type` deriva del número de versión: el verde llega con el
bump 0.5.0→0.6.0. Precedentes: ADR-041 y ADR-042 (major directo documentado, sin aliases),
AST-010 ("rename directo, 0 usuarios"), CHANGELOG [Unreleased] con entradas "BREAKING (0.x)".

## Decision

No revertir ningún breaking. La versión se bumpa **solo vía release-plz** (los commits
`refactor!:`/`feat!:` ya en historia + `semver_check=true` producen el minor 0.6.0 en main);
bump manual prohibido. No se añaden aliases de compatibilidad (precedente AST-010/ADR-041:
deuda de compatibilidad rechazada). El rojo-en-develop se acepta hasta el release 0.6.0.

## Consequences

- Pros: features de develop intactas; cero churn en archivos hot; mecanismo estándar 0.x
  (minor = breaking) en vez de gate artesanal; consumidores externos migran una vez en 0.6.0.
- Cons: `cargo semver-checks -p vantadb` seguirá rojo en develop y en la PR develop→main
  hasta el bump post-merge; quien exija verde-ya debe pedir revert masivo destructivo
  (desaconsejado) — ver Gate C en `docs/tasks/FIND-133.md`.
- Deuda: ninguna nueva; el ADR cierra Regla 5 para los ~100 ítems (veredicto + evidencia
  en el task file, no solo en este ADR).
