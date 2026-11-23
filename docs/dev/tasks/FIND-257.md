---
id: FIND-257
title: "Sanitizar saltos CRLF en scripts/docs/check-api-docs.mjs"
type: fix
state: IN_PROGRESS
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
absorbe: FIND-267
---

# FIND-257 — Normalizar CRLF en el parser de superficie de API

## Objetivo

`extractRust` itera lineas con `text.split("\n")` y ancla el header de bloque
`impl` en `\{[ \t]*$`. En Windows los ficheros del worktree llegan con CRLF;
el `\r` residual rompe el match de TODOS los bloques `impl` -> el gate reporta
126 remociones fantasma (`rust impl *`) con el arbol limpio.

## Contrato

```
node scripts/docs/check-api-docs.mjs --self-test && node scripts/docs/check-api-docs.mjs --changed HEAD..WORKTREE
```

**Estado ANTES (medido):** `--changed HEAD..WORKTREE` -> exit 1,
`GATE FAILED: 0 added, 126 removed` (todas `rust impl`, fantasma).
`--self-test` -> 17/17 exit 0 (fixtures LF: no detecta el defecto).

## Impacto mapeado (Regla 0)

| Archivo | Rol | Leido |
|---|---|---|
| `scripts/docs/check-api-docs.mjs` | split("\n") l.358 + regex `\{[ \t]*$` l.363 (extractRust) | SI (completo) |
| `scripts/docs/lib.mjs` | helpers compartidos; sus parsers ya usan `\r?\n` (l.139-155, 294, 334, 495) | SI (completo) |

**Referencias entrantes:** workflows `docs-gates.yml`/`ci-rust.yml` invocan el
script (PROHIBIDOS en esta tarea, FIND-392/FIND-468 en paralelo). lib.mjs NO
tiene el bug: `parseYaml`/`firstProseLine`/`loadTagVocabulary`/`segment` ya
normalizan `\r?\n`.

**Veredicto:** blast radius = 1 archivo (gate de paridad API docs). Extractores
python (trimEnd) y ts (trim) ya toleran `\r`; solo el loop de `impl` en
extractRust es vulnerable.

## Plan de fix

1. RED: anadir caso CRLF al `--self-test` (fixture `src/engine.rs` con `\r\n`)
   que falle antes del arreglo.
2. GREEN: normalizar `stripRust(modFile).replace(/\r\n/g, "\n")` en extractRust.
3. Verificar contrato: self-test exit 0 + `--changed HEAD..WORKTREE` exit 0.

## SDP

SKILLS: systematic-debugging, ci-cd-and-automation

## Steps

- [x] MEDIR estado ANTES (126 remociones fantasma, exit 1) — hecho
- [x] RED: caso CRLF en self-test que falla pre-fix — hecho
- [x] GREEN: normalizar EOL en extractRust — hecho
- [x] VERIFY contrato completo — hecho
- [x] Commit local — hecho
