---
title: Auditoría — Dependencias
kind: review
status: active
tags: [vantadb, avance, dependencies, deny, dependabot, advisories, cargo]
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
- Ver detalle en `docs/dev/Backlog.md` fase SEC y `docs/historial/backlog-history.md` (removidos).

## Commit de P8 para dependabot

- P8-03: dependabot semver-minor/patch (`target-branch: develop`, monkeys: `cargo`/`npm`/`pip`, auto-label `dependencies`/`auto-merge`, `allow-dep` auto-merge con `review_count: 0` únicamente cuando build/test pasan). ✅
- P8-04: cron semanal `cargo-deny check advisories` (fail on vuln). ✅

## Convenio | Contrato

- `.github/dependabot.yml` — config en repo.
- `.github/workflows/ci-security-weekly.yml` — cron advisory check.
- Releases: `cargo semver-checks` gate (v1-lead).
- ERR-006 (deny.toml RUSTSEC-2024-0436 limpio) — migrado 2026-08-12 (ver docs/progreso/README.md)

- ERR-007 (multiple-versions ban-skip documentado) - migrado 2026-08-12 (ver docs/progreso/README.md)

### AST-007 (deny triage 2026-09-11)
- RUSTSEC-2023-0071 (rsa 0.9.10/Marvin via jsonwebtoken 11.0.0): triaged con evidencia HS256-only (src/server/jwt.rs:9,64-67; sin from_rsa_* en src/); ignore con owner vanta-lead + expiry 2027-01-01. Sin patch disponible segun advisory.
- RUSTSEC-2026-0253 (lru): ignore removido - advisory-not-detected (lru directo 0.18.4 >= 0.18.2; 0.16.4 solo transitiva via tantivy 0.26.1).
- Verificacion: cargo deny check exit 0 (advisories/bans/licenses/sources ok).

### FIND-96 (audit-quick 2026-09-16)
- RUSTSEC-2026-0285 (rustls 0.23.43, TLS 1.3 cross-nivel): `cargo update -p rustls` 0.23.43→0.23.45 (+ webpki/aws-lc); audit limpio de este ID; `check --workspace --tests` 2m12s ✅.

### FIND-97 (audit-quick 2026-09-16)
- RUSTSEC-2023-0071 (rsa/Marvin): análisis profundo (cadena única `cargo tree`; HS256-only; backends `rust_crypto`/`aws_lc_rs` verificados en fuente jsonwebtoken 11) → opción A elegida por owner: espejado del triage AST-007 a `.cargo/audit.toml` (owner/expiry 2027-01-01); swap a `aws_lc_rs` descartado (compilación C para riesgo inalcanzable). `cargo audit` exit 0 ✅.

### Nota lru 0.16.4 (solo documentación, sin acción — audit-quick 2026-09-16)
- `cargo audit` muestra warning unsound RUSTSEC-2026-0253 sobre lru 0.16.4, que entra solo vía tantivy 0.26.1 (cadena única verificada con `cargo tree`; direct lru 0.18.4 ya ≥ parche).
- No hay fix desde nuestro código: se resuelve cuando tantivy publique versión sin lru 0.16. Tensión conocida: la allowlist se removió en AST-007 por advisory-not-detected y el warning reapareció vía transitiva — re-evaluar al triagear FIND-97 o ante bump de tantivy. No bloquea gates (deny exit 0, audit lo marca allowed).

### FIND-652: Migración `chrono` → `jiff` con serialización RFC3339 byte-idéntica (`RUSTSEC-2026-0543`)
- **Fecha:** 2026-11-18
- **Objetivo:** Reemplazar `chrono::DateTime<Utc>` por `jiff::Timestamp` en `vantadb` (`Value::{DateTime, ListDateTime}`, `FieldValue::{DateTime, ListDateTime}`), `vanta-memory`, `vantadb-python` (`pyo3` feature `jiff-02`) y `providers/shared_py.rs`, preservando compatibilidad 100% byte-idéntica en disco y wire (RFC3339 `SecondsFormat::AutoSi` y `postcard`) para retirar `chrono` del grafo de dependencias default.
- **Resultado:** ✅ Módulo `src/sdk/serialization/tsfmt.rs` reproduce byte a byte el formato `AutoSi` de `chrono 0.4.45` (0/3/6/9 dígitos fraccionarios, años de 4/5 dígitos, `+00:00`/`Z`), re-exportado públicamente como `vantadb::sdk::tsfmt` y consumido también por `providers/{openai,ollama,litellm}`; snapshot `tests/api/public-api.txt` actualizado y notas de migración en `docs/user/operations/UPGRADE.md`. Suite 591/591 serialización/compat + 721/721 `vanta-memory` + clippy de los 3 providers verde; review P2-01 `vanta-review 3a13775b` + `reviewer-fast` (`#21`) APPROVE y auditoría `#20` PASS.
- **Commit:** `f17caf906`, `aadef5a73` (`5fdb7faac`, `22e442ec4`)


### FIND-644: Dependabot alerts en paquetes publicados (triaje 2026-11-19)

- **Contexto:** GitHub reportaba ~25 alertas en `main` (4 high, 17 moderate, 4 low); FIND-644 registró 31 el 2026-11-09 (15 de `feature-freez/desktop`, que solo existe en `main` — `develop` ya no contiene ese árbol y el merge de lanzamiento las retira). Barrido local hecho con `npm audit` (4 lockfiles tracked), `cargo audit` (root + vantadb-node + providers) y consulta OSV por cada manifiesto pip escaneado.

- **Remediadas (npm — `vantadb-ts/package-lock.json`, 13 moderadas → 0):**
  - `vite` 6.4.4 → 8.3.8 (GHSA-8g5g-mxqh-grgj; el audit-fix deduplicó el peer de vitest/vite-plugin-wasm a la línea 8, ya permitida).
  - `eslint` 10.7.0 → 10.15.0, `@eslint/config-array` 0.23.5 → 0.23.6, `typescript-eslint` 8.64.0 → 8.73.0 (arrastran `minimatch ^11` → 11.0.9; cierra GHSA-g3hv-75wg-m249 en esas ramas).
  - Cadena `c8@12` → `test-exclude@8` → `{glob 13.0.6, minimatch 10.2.5}`: upstream no publicó fix (test-exclude 8.0.0 es la última y sigue vulnerable; la única "fix" de npm era downgrade c8→9.1.0). Resuelta con overrides acotados en `vantadb-ts/package.json` (`test-exclude.{glob:^14.0.2, minimatch:^11.0.1}`) — glob 14.0.6/minimatch 11.0.9 verificados compatibles con la API que usa test-exclude (`await glob()`, `glob.sync()`, `minimatch()`); smoke `npx c8` + `test-exclude.globSync()` OK. Cierra GHSA-3g3m-2fjr-6pw4 + resto de GHSA-g3hv-75wg-m249.
  - `vantadb-node`, `benchmarks`, `examples/typescript`: 0 vulns en entrada (la campaña previa ya subió vite/@vitest/mocker/source-map-js).

- **Remediadas (pip — floors en pyproject.toml; verificadas limpias contra OSV):**
  - `integrations/mem0`: `mem0ai>=2.0.18,<3` (antes `>=0.1.0,<1`). Cubre GHSA-7587-9684-52r6/CVE-2026-115324 (SQLi Databricks, crítica, fix 1.0.8), GHSA-5324-3w58-qpp2/CVE-2026-116575 (fuga cross-tenant de session_scope, fix 2.0.18) y GHSA-xqxw-r767-67m7/CVE-2026-7597 (pickle en faiss.py, fix 2.0.0b2). La única superficie consumida es `mem0.vector_stores.base.VectorStoreBase` — 33/33 tests verdes contra mem0ai 2.2.1 instalado.
  - `providers/litellm`: `litellm>=1.105.0,<2` (antes `>=1.0,<2`). ~44 advisories cubren <1.105 (último fix 1.96.2 + GHSA-7hp6-4w63-5g45 privesc → 1.100.4/1.101.3/1.102.2/1.103.1/1.104.0rc2); 1.105.0 es la primera estable sin advisories abiertas en OSV.
  - `integrations/llamaindex`: `llama-index-core>=0.13.0,<1` — 14 advisories en 0.12.x; GHSA-cr7q-2w66-hjcm/PYSEC-2026-1562 solo fijan en 0.13.0.
  - `integrations/langchain`: `langchain-core>=1.3.3,<2` (antes `>=0.3.81,<1`) + `langgraph-checkpoint>=4.1.1,<5` (antes `>=2,<5`). 8 advisories en langchain-core 0.3.x fijan solo en 1.2.x/1.3.3; 6 en checkpoint fijan en 3.0.0/4.0.0/4.1.1. Compat verificada: imports del adaptador resuelven en langchain-core 1.6.2 + checkpoint 4.2.0; kwarg `allowed_objects` de FIND-649 existe en 1.x; checkpoint 4.1.1 solo exige `langchain-core>=0.2.38`. 47/47 tests verdes.
  - `integrations/dspy`: `dspy>=3.0.4,<4` — GHSA-vvw2-h478-xwr3/PYSEC-2026-1318 afectan ≤3.0.3 sin fix 2.x. El adaptador solo toca `dspy.Prediction` (con fallback graceful de ImportError).
  - `integrations/haystack`: `haystack-ai>=2.3.1,<3` — GHSA-hx9v-6r9f-w677/PYSEC-2026-1447 fix 2.3.1.
  - `integrations/test_pins.py`: 19/19 — todas las deps de framework conservan techo `<`.

- **Excepciones documentadas (no remediales hoy):**
  - `chromadb` en `benchmarks/requirements.txt` (`>=0.4.22`): 4 advisories (GHSA-2wm9-hf6c-p5cr, GHSA-36p7-vc44-83pf, PYSEC-2026-3813/3814) cubren `0.4.17`–`1.5.9` — **1.5.9 es la última publicada; no existe release parcheada**. Dependencia opcional del competitive_bench, no se distribuye con el producto. Re-evaluar cuando chromadb publique >1.5.9.
  - `heapless 0.7.17` — 6 advisories *unsound* (RUSTSEC-2026-0577..0582, publicadas 2026-11-18). Transitiva vía `postcard`; no hay fix en la línea 0.7 (`cargo update -p heapless --dry-run`: 0 paquetes) y la migración a heapless 0.9 llega con postcard 2.0 (ya trackeado en `.cargo/audit.toml`/deny.toml vía RUSTSEC-2023-0089). Warnings permitidos, no vulnerabilidades con CVE.
  - `wide 1.6.0` yanked — RESUELTO: `cargo update -p wide` → 1.7.1 (+ `safe_arch` 1.1.0→1.2.0) en `vantadb-node` y `providers/{ollama,openai,litellm}` Cargo.lock; el lock raíz ya estaba en 1.7.1. Era warning de yank, no advisory.
  - `feature-freez/desktop`: sus alertas viven solo en `main`; `develop` no contiene el árbol — cierre implícito al mergear la rama de lanzamiento.
  - GitHub Actions (SHA-pinned): sin forma de enumerar las alertas exactas sin API; hay ramas dependabot de versiones pendientes (download-artifact 8.0.2, setup-node 7.1.0, upload-artifact 7.0.2, dtolnay) que cubren los bumps rutinarios.

- **Verificación:** `npm audit` vantadb-ts 0 vulns (de 13); vitest 380/380; pytest mem0 33/33, langchain 47/47, test_pins 19/19; `cargo audit` 0 vulnerabilidades en root + vantadb-node + providers/{ollama,openai,litellm} (9 warnings permitidos — heapless unsound + preexistentes); `cargo check -p vantadb --all-targets -j 2` ✅ (6m07s).
