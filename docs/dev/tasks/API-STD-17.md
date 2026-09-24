# Task API-STD-17 — Docs / referencias / actualizaciones

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Auditar deriva doc↔código + owners + updates. Contrato: coverage anotado + tabla deriva con dueño + checklist updates.

## 2. Tabla deriva doc→código (dueño propuesto)

| Deriva | Dueño |
|---|---|
| `BINDINGS_NAMESPACES:215` supersede Python-only (falso) + `:75` search Py (falso) + `:70-73` ausencias obsoletas | owner: binding-lead (reescribir tras 18-W1) |
| IQL 6-vs-7 (`IQL.md:16-26` vs `grammar.rs:349`) | owner: parser (fijar en 18-W5) |
| YAML vs router/handlers (6 drifts H5) | owner: server (YAML owner único desde 18-W2) |
| `MCP.md` tools count vs `opencode.jsonc:87` (15 vs 87, FIND-NEW-02) | owner: mcp (re-listar tras 18-W3) |
| `TS_SDK.md` distance lower-is-better (falso hoy) | owner: ts (fijar en 18-W1) |
| `vanta-memory` sin coverage script (`VANTA_MEMORY.md:14`) | owner: memory (referencia manual) |
| Contrato plan 01: 19 = 18 md + 1 yaml (corregido) | owner: docs (este plan) |

## 3. Checklist updates (verificado 2026-09-24)

- `scripts/validate-docs-coverage.ps1`: existe (`True`) — correr en 18 por wave (nota: no cubre vanta-memory).
- Dependabot: `.github/dependabot.yml` existe (`True`).
- Workflows: 27. `secrets.*` solo `GITHUB_TOKEN` automático (release/desktop) + opcionales OCR/opencode — **cero tokens de publish** ✅ OIDC tokenless vigente.
- release-plz: bump+CHANGELOG+tags automáticos; prohibido manual (Regla 7) — aplica a las 18-W waves (`feat!:` cada breaking).
- Doc Language Split: normativa nueva en EN (`docs/api/`), ES solo planning — este plan y task files en ES ok.

## 4. DoD

- [x] Contrato ✅ · Task file sync · Recitation: deriva con dueños + updates verificados

## Context Save Point

API-STD-17 DONE. Next: API-STD-18 (implementación). Deuda: 7 derivas con dueño. WIP: ninguno.
