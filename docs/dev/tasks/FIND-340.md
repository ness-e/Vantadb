---
title: "FIND-340 — Preparación release 0.8.0 SIN push"
kind: task
status: active
description: "Backup verificado + verify-release v0.8.0 con reporte + pins DIST-08 alineados + ARCHITECTURE refresh FIND-322. Solo-preparar, sin push."
tags: [vantadb, release, find-340]
---

# FIND-340 — Preparación release 0.8.0 SIN push

> Origen: Backlog §PROC-01 (solo-preparar) + §DIST-08 + §FIND-322 · Wave 1 · Next: EGO-03.
> Regla: `docs/dev/workflow/RULES.md` §8 (backups) + `PUBLISH.md` (release completo = publicado + verificado).
> `docs/dev/FASE-A.md`: solo lectura (su ejecución es humana).
> Skill docs: `documentation-skill` cargada antes de editar `.md` (EN para QUICKSTART/ARCHITECTURE, frontmatter intacto).
> SDP: mandated base (documentation-skill + progreso lectura + ponytail full); keywords release/backup/pins/architecture/docs → sin candidatos extra en SKILLS-MANIFEST (codebase-memory/codegraph innecesarios: blast radius mínimo scripts + 2 docs, verificado por grep).
> SKILLS_CARGADAS: documentation-skill (edición .md), progreso (lectura, Trigger 2: ID no registrado en avance), ponytail full (diff mínimo, sin abstracciones).
> PROHIBIDO: push/tags/PRs, versiones/CHANGELOG (release-plz), `vantadb-python/**` (WIP ajeno, 26 files dirty intactos), stash, `.opencode/**`, `docs/index.md` y `llms.txt` (NO gen-index), `src/**`, `tests/**`, `vantadb-node/**`.

## Objetivo

Dejar la release 0.8.0 **preparada y verificada localmente, sin push**: backup verificado + verify-release ejecutado con reporte de gaps + pins QUICKSTART/docs-demo alineados a 0.8.0 live + ARCHITECTURE.md refrescado v0.6.1→v0.8.0 (scoped, sin reescritura).

## Contrato

- Backup verificado exit 0 (`git bundle verify` fail-closed; si ≠0 → STOP total, R2 GitLab).
- `scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` ejecutado con reporte + gaps informados.
- Pins alineados (0.6.1→0.8.0 live) en QUICKSTART/docs-demo.
- ARCHITECTURE.md actualizado (WAL layout, record variants, HTTP ownership, retrieval model, límites re-verificados + frontmatter).
- `git diff --check` exit 0 + `check-links`/`check-docs` sin nuevos issues.

## Spec (decisiones)

| # | Decisión | Opción elegida | Alternativa rechazada + por qué |
|---|----------|----------------|--------------------------------|
| 1 | Destino backup | Default `$HOME\VantaDB-Backups` (fuera del repo) | Volumen externo: no existe (solo C: en este runner) — warning same-volume aceptado y registrado como gap |
| 2 | Pin target DIST-08 | `0.8.0` = `[workspace.package] version` (Cargo.toml:803) + live en 4 registries (verify-release ALL GREEN lo confirma) | `0.7.0`: vigente al redactar DIST-08 pero superado por el tren 0.8.0 |
| 3 | Piso demo `>=0.8.0` | Consistente con QUICKSTART (`==0.8.0`) y avance ci-cd (0.6.1/0.7.0/0.8.0 ✓ en PyPI) | `>=0.6.1,<0.9.0` (patrón adapters): rango extra sin valor para una demo |
| 4 | FAQ.md en scope | Incluido como "docs demo" (pins de instalación user-facing, diferidos por API-10 a decisión owner → esta tarea es esa decisión) | Dejar stale: contradice el contrato "pins alineados" |
| 5 | Blog/clients históricos | NO tocados (claude_code_mcp_memory.md, UPGRADE.md §0.6.1, CHANGELOG) = registro histórico | Reescribir historia: destruye trazabilidad |
| 6 | ARCHITECTURE scope | Solo: frontmatter + marcador versión + variantes txn WAL v2 (Begin/Prepare/Commit/Abort) | Reescritura: prohibida por FIND-322 (scoped, no reescritura) |

## Impacto mapeado (Regla 0)

- Archivos leídos completos: `scripts/git-backup.ps1`, `scripts/verify-release.ps1`, `docs/user/QUICKSTART.md` (verificado ya-0.8.0 por API-10, sin edits), `docs/dev/architecture/ARCHITECTURE.md`, `docs/user/FAQ.md:21-38`, `docs/user/operations/MCP_REGISTRY.md:39`, `examples/demo/{README.md,requirements.txt,demo.py}` (sección pins), `src/wal.rs:109-152` + `src/binary_header.rs` (solo lectura verificadora, sin edición), `docs/dev/Backlog.md` (§PROC-01/§DIST-08/§FIND-322), `docs/dev/workflow/RULES.md` §8.
- Referencias hacia dentro: pins → registries live (evidencia: verify-release exit 0); ARCHITECTURE claims → `src/wal.rs` (variantes), `src/binary_header.rs` + `src/wal.rs:158-163` (VWAL/20B), `src/cli_handlers/server.rs` + `src/lib.rs:72` (feature `server`).
- Referencias entrantes: `docs/index.md` enlaza ARCHITECTURE (índice generado, prohibido — no se regenera en esta tarea); QUICKSTART enlazado desde README (sin cambios de links).
- Veredicto: impacto doc-only + scripts solo-ejecutados (cero ediciones en scripts); ningún símbolo `pub` nuevo; ningún hot path; Gate D no disparado (blast radius 6 archivos docs/demo, cambio mecánico de pins + refresh scoped).

## Steps

- [x] S1 — Backup: `pwsh scripts/git-backup.ps1` → exit 0, bundle `vantadb-20261015-1747.bundle` (161.4 MB, 42 refs, 18 commits locales vs origin/develop), `git bundle verify` OK. Gap: mismo volumen (solo C: existe) — R2 parcial.
- [x] S2 — `scripts/verify-release.ps1 -Tag v0.8.0 -Smoke` → exit 0 ALL GREEN (14 assets GitHub + crates.io/PyPI/npm×2 live + smoke `vanta-cli 0.8.0` con sha256). Sin gaps: la release 0.8.0 existe y está completa.
- [x] S3 — DIST-08 pins →0.8.0: `docs/user/FAQ.md:23,32`, `docs/user/operations/MCP_REGISTRY.md:39`, `examples/demo/requirements.txt:1`, `examples/demo/README.md:20,27`, `examples/demo/demo.py:17`. QUICKSTART ya-0.8.0 (API-10, verificado sin cambios).
- [x] S4 — FIND-322 ARCHITECTURE scoped: frontmatter (desc→v0.8.0, retiradas `type`/`last_reviewed` inválidas por schema) + marcador v0.6.1→v0.8.0 + variantes txn WAL v2. WAL layout (VWAL/20B), HTTP ownership (`server` feature), retrieval v1 y límites re-verificados contra develop sin cambios (ya correctos).
- [x] S5 — VERIFY: `git diff --check` exit 0 + `check-links`/`check-docs` sin nuevos issues (5 broken links + 2 missing-frontmatter pre-existentes en archivos no tocados).

## Decisiones registradas

- NOTICED BUT NOT TOUCHING: `vantadb-python/**` (26 files dirty, WIP ajeno) · `docs/user/blog/claude_code_mcp_memory.md:28,34` + `UPGRADE.md` §0.6.1 + `CHANGELOG.md` (histórico) · `examples/langgraph_dev_to_prod/README.md:34` (describe pins de `integrations/`, dueño FIND-200 al publicar) · `docs/user/tutorials/04-hybrid-search-basics.md:185` (texto de ejemplo, no pin) · `check-links` 5 rotos + `check-docs` 2 sin-frontmatter (pre-existentes, candidatos a filas FIND) · `docs/index.md` (describe ARCHITECTURE como v0.6.1 — índice generado, lo regenera el orquestador con gen-index).
- Deuda: backup same-volume (sin disco externo en runner) · `gen-index --write` no corrido (prohibido tocar `docs/index.md`/`llms.txt` — el orquestador lo corre al cerrar).

## Context Save Point

Trabajo en worktree (sin commit, sin stage — lo commitea el orquestador): `docs/user/FAQ.md` (2 pins), `docs/user/operations/MCP_REGISTRY.md` (1 sync), `examples/demo/{requirements.txt,README.md,demo.py}` (4 pins), `docs/dev/architecture/ARCHITECTURE.md` (frontmatter + versión + txn variants), este task file (nuevo).
Evidencia: bundle `C:\Users\Eros\VantaDB-Backups\vantadb-20261015-1747.bundle` (verify OK) · verify-release ALL GREEN exit 0 (output en sesión) · `git diff --check` exit 0.
PROXIMO_STEP: review + commit (lo organiza el orquestador).

## Review (Gate P2-01)

- **Revisor:** vanta-review, sesión distinta al implementador (veredicto 2026-10-15, 4/5 re-ejecutado; verify-release network-gated no re-ejecutado, declarado).
- **Veredicto:** ✅ APPROVE FIND-340 — sin tags/ramas nuevas, pins 0.6.1→0, ARCHITECTURE scoped + variantes WAL corroboradas, bundle verify exit 0, checks sin nuevos issues. 1 Optional (log de verify-release a futuro) + 1 deuda (docs/index.md stale → gen-index al cierre).
