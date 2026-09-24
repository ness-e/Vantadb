# Task API-STD-09 — INDIVIDUAL (8/11) IQL

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual IQL: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Lenguaje query del core (nom: `grammar.rs`+`lexer.rs`): CRUD nodos, traversals, vector/hybrid search, `INSERT MESSAGE`, `PROFILE`. Lo consumen SDK (`query`), HTTP, MCP (`query_iql`), CLI.

## 3. Uso (ejemplos mínimos)

```
FROM docs WHERE payload CONTAINS "hello" FETCH 10
INSERT NODE#123 { payload: "hi" }
```

## 4. Código (re-verificado 2026-09-24, grep directo)

- **Q1 7 vs 6 CONFIRMADO:** `parse_select` existe (`grammar.rs:349`, usado `:316`, dispatch `:439` con comentario "Must be before parse_query") — docs `IQL.md:16-26` lo omiten.
- **Q2 Orden-desambiguación CONFIRMADO:** `parse_insert_message` antes de `parse_insert` (`:434` "to prevent shadowing"); `parse_select` antes de `parse_query` (`:439`).
- **Q3 `PROFILE` sin gate CONFIRMADO:** tag en `:117` (`grammar.rs`).
- **Q4 Sin versión CONFIRMADO:** `IQL_VERSION|language version|LANGUAGE_VERSION` = 0 matches en `src/`.
- **Q5 Bugs lexer/parser** (auditoría previa, para repro en 16): `==` (`:47-56` orden `=` primero + test `mod.rs:170-176`); int→`Float(42.0)` (`lexer.rs:138-144`); keywords solo UPPERCASE (`lexer.rs:41-73`); solo `"` (`:85-122`).

## 5. Veredicto + implicaciones

5/5 confirmados. Propuestas 15: `IQL_VERSION` (gatear `PROFILE`/futuro), 1 forma de lectura o las 3 documentadas con defaults idénticos, case-insensitive o prohibido explícito, `"`+`'` o error claro, `==` + int literales, AST JSON plano para bindings (bloque usuario §5). Blast radius: parser + `sdk/api.rs` + tests `mod.rs` + YAML ejemplo + MCP `query_iql`.

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: IQL ficha completa

## Context Save Point

API-STD-09 DONE. Next: API-STD-10 (CLI). Deuda: ninguna. WIP: ninguno.
