---
title: ADR-041 Error Variant Renames
type: architecture
status: active
tags: [vantadb, adr]
last_reviewed: 2026-09-23
aliases: []
---

# ADR-041: Renames del enum `Error` sin stutter (major directo)

> **Estado:** Aceptado (decisión humana vía `/pipeline run` cleanCA, 2026-09-12) · **Tarea:** D4b (diseño en `docs/dev/tasks/D4b.md`)

## Contexto

El enum `Error` (`src/error.rs`) repite el sufijo `Error` en 13 variantes (`IoError`,
`BackendError`, `ValidationError`…) dentro de un tipo que ya se llama `Error`
(stuttering N2, guía §2.1/Apéndice V.3). El diseño D4b mapeó afectados: ~100 call-sites
Rust, espejos Python (`shared_py.rs`, `convert.rs`), `from_core` en desktop, ~15 tests,
3 docs API. El wire (`code()`/`Display`) no se toca en ningún caso.

## Decisión

**Major directo:** renombrar las 13 variantes de una vez
(`WalError→Wal`, `SerializationError→Serialization`, `IoError→Io`,
`IqlParseError→IqlParse`, `ValidationError→Validation`, `IqlError→Iql`,
`CliError→Cli`, `SearchError→Search`, `RuntimeError→Runtime`, `RestoreError→Restore`,
`BackupError→Backup`, `BackendError→Backend`, `SchemaError→Schema`), documentado como
breaking en el changelog. Sin aliases de compatibilidad.

## Consecuencias

- **Costos:** ~100 call-sites Rust se actualizan en el mismo cambio; consumidores externos
  del crate migran en el major; `cargo semver-checks` debe correrse antes de merge.
- **Riesgos:** bajo — wire invariante, cambio mecánico verificable por compilación.
- **Deuda evitada:** no se crean aliases deprecated (cero deuda de compatibilidad).
- **Alternativas descartadas:** aliases deprecated (deuda temporal), solo-internos
  (deja stutter público remanente).
