---
title: "HIG-03 — Limpieza de dead_code en src/ y archivo de Backlog en avance/"
kind: task
description: "Auditoría de #[allow(dead_code)] en core y migración de tareas completadas a docs/dev/avance/"
status: completed
---

# Task `HIG-03` + `Progreso`: Higiene de `#[allow(dead_code)]` y Archivo de Backlog

## Scope (Allowed Files)
- `src/graph.rs`
- `src/memory/consolidation.rs`
- `src/memory/lifecycle.rs`
- `src/server/handlers.rs`
- `src/storage/engine/node_ops.rs`
- `src/vector/pq.rs`
- `docs/dev/Backlog.md`
- `docs/dev/avance/activo/ci-cd.md`
- `docs/dev/avance/activo/core-engine.md`
- `docs/dev/avance/activo/bindings.md`
- `docs/dev/avance/auditoria/seguridad.md`
- `docs/dev/reviews/2026-10-18-full-system-audit.md`

## Acceptance Criteria
1. Unnecessary `#[allow(dead_code)]` attributes in `src/` are removed or, where required by feature-gating/schema compatibility, accompanied by an explicit `// Reason:` justification comment.
2. Completed rows from `2026-10-15`, `FIND-345..380`, `DX-04..09`, and `HIG-03` are archived into `docs/dev/avance/` and removed from `docs/dev/Backlog.md` per `.opencode/skills/progreso/SKILL.md`.

## Verification Commands
- `cargo clippy --workspace --all-targets -- -D warnings`
- `node scripts/docs/check-docs.mjs`
