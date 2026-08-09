---
title: "Auditoría — Dependencias"
type: audit-log
status: active
tags: [vantadb, avance, dependencies, deny, dependabot, advisories, cargo]
last_reviewed: 2026-08-07
aliases: []
---

# Auditoría — Dependencias

> Registro consolidado de la gestión de dependencias: cargo-deny, Dependabot, advisories, licencias. IDs originales conservados (SEC-*, CODE-*, P8-*).

## Política

- `cargo-deny` debe pasar **antes de cualquier release** — solo licencias MIT/Apache-2.0 (Regla 7 AGENTS.md).
- Dependabot: PRs de dependencias revisados; auto-merge solo para semver-minor/patch con label `dependencies`/`auto-merge` (P8-03).
- Cron **weekly security check** con `cargo-deny check advisories` (P8-04).

## Actividad registro

| ID | Tarea | Resultado |
|---|---|---|
| SEC-01 | FFI audit de dependencias con unsafe | ✅ |
| SEC-02 | Supply chain: dependencias con `as?` | ✅ |
| CODE-056 | Duplicate reqwest 0.12+0.13 (unificar una sola versión) | ✅ |
| CODE-058 | Ignored advisories sin rationale | ✅ |
| CODE-051 | deny.toml stale (ignore desactualizado) | ✅ batch `5652a9f` |
| CODE-067 | u128 migrate (dependency metadata) | ✅ |

## Uso de dependencias (workspace)

- Workspace `Cargo.toml` con `[workspace.dependencies]` para dependencias compartidas; NO versiones duplicadas por crate.
- Cualquier dependencia nueva → revision en `SEC-02` (supply chain) y `cargo-deny check`.

## Advisories conocidos

- Estado: watch en CI semanal (`ci-security-weekly.yml`).
- Ver detalle en `docs/Backlog.md` fase SEC y `docs/historial/backlog-history.md` (removidos).

## Commit de P8 para dependabot

- P8-03: dependabot semver-minor/patch (`target-branch: develop`, monkeys: `cargo`/`npm`/`pip`, auto-label `dependencies`/`auto-merge`, `allow-dep` auto-merge con `review_count: 0` únicamente cuando build/test pasan). ✅
- P8-04: cron semanal `cargo-deny check advisories` (fail on vuln). ✅

## Convenio | Contrato

- `.github/dependabot.yml` — config en repo.
- `.github/workflows/ci-security-weekly.yml` — cron advisory check.
- Releases: `cargo semver-checks` gate (v1-lead).