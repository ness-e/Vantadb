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
### ERR-006 (deny.toml RUSTSEC-2024-0436 limpio) — migrado 2026-08-12 (ver docs/progreso/README.md)

### ERR-007 (multiple-versions ban-skip documentado) - migrado 2026-08-12 (ver docs/progreso/README.md)

### AST-007 (deny triage 2026-09-11)
- RUSTSEC-2023-0071 (rsa 0.9.10/Marvin via jsonwebtoken 11.0.0): triaged con evidencia HS256-only (src/server/jwt.rs:9,64-67; sin from_rsa_* en src/); ignore con owner vanta-lead + expiry 2027-01-01. Sin patch disponible segun advisory.
- RUSTSEC-2026-0253 (lru): ignore removido - advisory-not-detected (lru directo 0.18.4 >= 0.18.2; 0.16.4 solo transitiva via tantivy 0.26.1).
- Verificacion: cargo deny check exit 0 (advisories/bans/licenses/sources ok).

### FIND-96 (audit-quick 2026-09-16)
- RUSTSEC-2026-0285 (rustls 0.23.43, TLS 1.3 cross-nivel): `cargo update -p rustls` 0.23.43→0.23.45 (+ webpki/aws-lc); audit limpio de este ID; `check --workspace --tests` 2m12s ✅.

### Nota lru 0.16.4 (solo documentación, sin acción — audit-quick 2026-09-16)
- `cargo audit` muestra warning unsound RUSTSEC-2026-0253 sobre lru 0.16.4, que entra solo vía tantivy 0.26.1 (cadena única verificada con `cargo tree`; direct lru 0.18.4 ya ≥ parche).
- No hay fix desde nuestro código: se resuelve cuando tantivy publique versión sin lru 0.16. Tensión conocida: la allowlist se removió en AST-007 por advisory-not-detected y el warning reapareció vía transitiva — re-evaluar al triagear FIND-97 o ante bump de tantivy. No bloquea gates (deny exit 0, audit lo marca allowed).
