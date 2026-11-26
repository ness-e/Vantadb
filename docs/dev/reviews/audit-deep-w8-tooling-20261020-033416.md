---
title: "Deep audit W8 - Tooling and periphery"
kind: review
status: active
description: "Auditoria read-only de dev-tools/, scripts/, examples/, Formula/, integrations/, providers/, skills/, .github/workflows/ y artefactos del repo: portabilidad, idempotencia, exit codes, duplicados y gates de CI."
tags: [audit, reviews, tooling]
---

# Deep audit W8 - Tooling y periferia

> **Wave:** W8 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W8)
> **Ejecucion:** subagentes `vanta-audit` + `vanta-review` no disponibles (sin tool `task`).

## Scope

**Cubierto y medido:**

| Area | Conteo real | Nota |
|---|---|---|
| `dev-tools/` | **20** ficheros | El brief de la campana decia 23 - medido 20 |
| `scripts/` | **27** ficheros | Coincide con el brief |
| `examples/` | 8 subdirs (`agent_memory_cli`, `colab`, `demo`, `langgraph_dev_to_prod`, `python`, `rag_pdf_chat`, `rust`) + `README.md` | Coincide |
| `Formula/` | `vantadb.rb` + `README.md` | - |
| `.github/workflows/` | **20** workflows | - |
| `integrations/` | 9 adapters + `vantadb_shared` | - |
| `providers/` | 3 providers + `shared_py.rs` | - |
| `skills/` | `vantadb` (2) + `vantadb-mcp` (24) | - |
| `e2e/` | 5 ficheros | - |
| `fuzz/` | 4 targets + corpus | Fuera del workspace |
| `datasets/` | SIFT1M + GloVe-100 + longmemeval | - |

## Metodologia

- Inventario con `Get-ChildItem` y contraste con el brief.
- Analisis de los 20 workflows: triggers, `continue-on-error`, invocaciones de `cargo`, uso de secrets.
- Verificacion de estado de versionado (`git ls-files` vs `git check-ignore`) de 20 artefactos del repo.
- Lectura de `Formula/vantadb.rb` y de `docs/dev/workflow/PUBLISH.md` (creado el 2026-10-20).

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-390` | High | **11 `continue-on-error: true` sin el tag `# CATEGORY:` que la propia Regla 2 exige** | `.github/workflows/certification.yml:534`, `.github/workflows/ci-rust.yml:360`, `.github/workflows/ci-rust.yml:454`, `.github/workflows/ci-rust.yml:650`, `.github/workflows/ci-rust.yml:693`, `.github/workflows/ci-rust.yml:782`, `.github/workflows/platform-ops.yml:221`, `.github/workflows/release-adapters.yml:167`, `.github/workflows/release-npm-node.yml:244`, `.github/workflows/release-providers.yml:267`, `.github/workflows/release-wheels.yml:197` | Medicion: `Select-String -Path .github/workflows/*.yml -Pattern '^\s*continue-on-error:\s*true'` = **38 ocurrencias** en 9 workflows (`nightly.yml` 23, `ci-rust.yml` 6, `docs-gates.yml` 3, y 1 en cada uno de los otros 6). De las 38, **11 NO llevan el tag `# CATEGORY:`** en la misma linea (las otras 27 si). `AGENTS.md:505` (Regla 2): "Cualquier nueva exencion requiere justificacion + CATEGORY tag." La mayoria de las 11 sin tag estan en **workflows de release** (`release-adapters`, `release-npm-node`, `release-providers`, `release-wheels`) y en `platform-ops.yml` (que contiene el analisis de seguridad CodeQL) - es decir, los canales donde un fallo silencioso se traduce en un artefacto publicado sin verificar. | Anadir `# CATEGORY: <EXPERIMENTAL|BEST-EFFORT|NON-CRITICAL|INFORMATIONAL>` a las 11 lineas, o eliminarlas. Prioridad: las 4 de workflows de release. Anadir un gate en `docs-gates.yml` que falle si encuentra `continue-on-error: true` sin `CATEGORY:` en la misma linea. | 2h |
| `FIND-391` | Medium | `AGENTS.md` declara "8 instancias" de `continue-on-error`; hay 38 | `AGENTS.md:505` | `AGENTS.md:505` -> "estrictamente prohibido sugerir o escribir `continue-on-error: true` en cualquier GitHub Action nuevo o existente (**existen 8 instancias**, todas con `# CATEGORY:` explicita)". Conteo real: **38** ocurrencias, de las cuales **11 no tienen** el tag que la misma frase afirma que todas tienen. El numero "8" no coincide con ninguna medicion posible. | Corregir a "38 (11 pendientes de tag)" y enlazar `FIND-390`. | 15min |
| `FIND-388` | Medium | **La formula de Homebrew sirve la version 0.8.0 cuando el crate esta en 0.9.0** | `Formula/vantadb.rb:12` | `Formula/vantadb.rb:12` -> `version "0.8.0"`, con los sha256 de la release v0.8.0 (comentario en lineas 5-10: "SHA256s verificados 2026-10-16 (FIND-342): fetched from the v0.8.0 release tarball sidecars"; "On each new release, refresh the version above and the SHA256s from: curl -sL .../v$VERSION/..."). `Cargo.toml:805` -> `version = "0.9.0"`. Consecuencia: `brew install vantadb` instala 0.8.0, un minor por detras, y el propio fichero dice que hay que actualizarlo en cada release - no se actualizo. La release 0.9.0 se publico el 2026-10-20 (postmortem), es decir, hoy. | Actualizar `version` a 0.9.0 y los 4 sha256 desde los sidecars `v0.9.0`. Anadir el refresh al checklist de `docs/dev/workflow/PUBLISH.md`. | 1h |
| `FIND-404` | Medium | `fuzz/Cargo.lock` esta gitignored: el harness de fuzzing no es reproducible | `.gitignore` (regla que excluye `fuzz/Cargo.lock`), `fuzz/Cargo.toml` | `git check-ignore -v fuzz/Cargo.lock` -> ignored. `git ls-files fuzz/` -> 0 ficheros trackeados. El crate `fuzz/` esta **fuera del workspace** por diseño (`Cargo.toml:784-786`: "cargo-fuzz requires nightly + Linux sanitizers"), y **sus 4 targets** (`fuzz_parser.rs`, `fuzz_wal.rs`, `fuzz_node_deserialize.rs`, `fuzz_archive.rs`) se ejecutan en `certification.yml:751,798,852` con `cargo +nightly fuzz run --target x86_64-unknown-linux-gnu`. Sin `Cargo.lock` versionado, las versiones de `libfuzzer-sys`/`arbitrary` de cada corrida pueden diferir: un crash reproducible ayer puede no reproducirse hoy, y al reves. | Trackear `fuzz/Cargo.lock` (es lo que hacen los crates con `cargo-fuzz` en CI) y anadir un `cargo +nightly fuzz` con `-max_total_time` fijo. | 1h |
| `W8-05` | Info | Inventario real: `dev-tools/` 20 (no 23), `scripts/` 27, `examples/` 8 | `dev-tools/`, `scripts/`, `examples/` | `dev-tools/`: `audit-all.ps1`, `build-wasm-types.mjs`, `chaos_loop.ps1`, `check-agents-refs.ps1`, `clean-opencode-loop.ps1`, `coverage-budget.ps1`, `fix-wasm-pkg-files.mjs`, `floor-guard.ps1`, `gate-common.ps1`, `heavy-test-lock.ps1`, `nocturnal_suite.ps1`, `ocr-review.ps1`, `parity-compare.mjs`, `run-all-tests.ps1`, `setup_venv.ps1`, `setup_venv.sh`, `target-cleanup.ps1`, `validate_doc_snippets.py`, `verify_changed.ps1`, `verify.ps1`. `scripts/`: 27 (4 pares con equivalente `.ps1`/`.sh`: `install`, `download_benchmark_datasets`, `verify_datasets`, `setup_venv`). El brief de la campana decia 23 scripts en `dev-tools/`; el numero real es 20. La diferencia probablemente es la version del brief, no una perdida en el repo. | Sin accion. Corregir el conteo en el brief de la campana para que la proxima revision no lo tome como faltante. | - |
| `W8-06` | Info | Todos los artefactos locales del repo estan gitignored (higiene OK) | `.gitignore` | Verificado uno por uno: `vector_index.bin` (ignored), `ci-fail-big.log` / `ci-fail-big2.log` (ignored), `dev.pid` (ignored), `.coverage` (ignored), `argus_budgets.db` (ignored), `.env.tokens` (ignored), `.env.local` (ignored), `benchmarks/vanta_benchmark_report.json` (ignored), `benchmarks/beir_recall_report.json` (ignored), `evals/memory_bench_report.json` y los dos `memory_harness_report*.json` (ignored), `embeddings/verify.log` (ignored), `fuzz/Cargo.lock` (ignored), `.vantadb-mcp/` (ignored, `.gitignore:205`), `embeddings/models/` (ignored, `.gitignore:238`). **Trackeados y correctos**: `benchmarks/criterion_baseline.json`, `benchmarks/python_baseline.json`, `benchmarks/wasm_threads_results*.json`, `embeddings/sanity_report.json`. Excepcion: `fuzz/Cargo.lock` (ver `FIND-404`). | Sin accion (salvo FIND-404). | - |
| `W8-07` | Info | `dev-tools/` cubre las 5 etapas del ciclo (verify, gate, coverage, chaos, nocturnal) con scripts nomeados | `dev-tools/verify.ps1`, `dev-tools/verify_changed.ps1`, `dev-tools/gate-common.ps1`, `dev-tools/floor-guard.ps1`, `dev-tools/coverage-budget.ps1`, `dev-tools/chaos_loop.ps1`, `dev-tools/nocturnal_suite.ps1`, `dev-tools/heavy-test-lock.ps1`, `dev-tools/audit-all.ps1`, `dev-tools/ocr-review.ps1`, `dev-tools/check-agents-refs.ps1`, `dev-tools/parity-compare.mjs`, `dev-tools/validate_doc_snippets.py` | El set cubre: verificacion (2 scripts, rapido+completo), gate comun reutilizable, quality floor (`floor-guard.ps1`, referenciado por `AGENTS.md:41` "Floor checks: `dev-tools/floor-guard.ps1`"), presupuesto de cobertura, caos, suite nocturna, lock de tests pesados, auditoria global, revision OCR (advisory, con API key), refs de agents, paridad de SDK y snippets de doc. `dev-tools/validate_doc_snippets.py` esta **en el path de CI** de `docs-gates.yml` (verificado). `dev-tools/parity-compare.mjs` es un entry point del grafo. | Sin accion. Es la parte mejor resuelta del tooling. | - |
| `W8-08` | Info | `scripts/` tiene 4 pares bash/PowerShell con contrato declarado identico | `scripts/{install,download_benchmark_datasets,verify_datasets}.{sh,ps1}`, `scripts/setup_venv.{ps1,sh}` | `scripts/verify_datasets.ps1:1` -> "Pre-test gate for heavy-certification workflow (Windows)" y `scripts/verify_datasets.sh:2` -> "(Linux)". El gate se inyecta como pre-step en `certification.yml` (job `other-heavy`). El prework de datos (`download_benchmark_datasets.*`) y la instalacion (`install.*`, `setup_venv.*`) tienen el mismo patron. | Sin accion. | - |
| `W8-09` | Info | 20 workflows con pins SHA y `permissions` declaradas | `.github/workflows/*.yml` | 20 workflows: `certification`, `ci-fleet`, `ci-gate`, `ci-rust`, `demo-acceptance`, `docs-gates`, `electron-node`, `nightly`, `perf-bench`, `platform-ops`, `publish-mcp-registry`, `release-adapters`, `release-binaries`, `release-npm-61`, `release-npm-node`, `release-providers`, `release-sbom`, `release-verify`, `release-wheels`, `release`. Todos los `uses:` de actions externas van pinneados por SHA con el comentario de version (ej. `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`). `release.yml` usa `secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN`. | Sin accion. | - |
| `W8-10` | Low | `benchmarks/update_markdown.py` puede escribir en `docs/` sin que el gate lo detecte | `benchmarks/update_markdown.py`, `docs/user/operations/BENCHMARKS.md:53` | `BENCHMARKS.md` contiene los markers `<!-- BENCHMARK_METRICS_START -->` / `<!-- BENCHMARK_METRICS_END -->` (lineas 53-58) que sugieren inyeccion automatica de la tabla. Si el generador escribe en `docs/`, el unico control es el diff del PR - `check-docs.mjs` valida frontmatter y `kind`, no contenido generado. | Verificar el destino de escritura del script y, si escribe en `docs/`, anadir su salida al allow-list de artefactos generados de `gen-index.mjs`. | 30min |

## Veredicto

**PASS CON OBSERVACIONES**

El tooling es la parte mas compacta y mejor organizada del sistema: 20 scripts en `dev-tools/`
(cubriendo verify, gate, floor, coverage, chaos, nocturnal, OCR, paridad y snippets), 27 en
`scripts/` con 4 pares bash/PowerShell de contrato identico, 20 workflows con todos los `uses:`
pinneados por SHA, y una higiene de artefactos impecable (cada `.json` de resultados, cada `.log` y
cada `.env` esta gitignored, con solo 4 artefactos de benchmark deliberadamente versionados). Tres
defectos: (1) **11 `continue-on-error` sin el tag obligatorio**, cuatro de ellos en los canales de
publicacion, donde un fallo silencioso se convierte en un artefacto publicado sin verificar - es
exactamente la clase de fallo que el postmortem de 0.9.0 documenta; (2) la formula de Homebrew
sirviendo 0.8.0 el mismo dia que se publica 0.9.0; y (3) `fuzz/Cargo.lock` sin versionar, que hace
irreproducibles las corridas de fuzzing que si se ejecutan en CI.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-390` | High | `.github/workflows/release-wheels.yml:197` |
| `FIND-391` | Medium | `AGENTS.md:505` |
| `FIND-388` | Medium | `Formula/vantadb.rb:12` |
| `FIND-404` | Medium | `fuzz/Cargo.toml:1` |
| `W8-05` | Info | `dev-tools/verify.ps1:1` |
| `W8-06` | Info | `.gitignore:238` |
| `W8-07` | Info | `dev-tools/gate-common.ps1:1` |
| `W8-08` | Info | `scripts/verify_datasets.ps1:1` |
| `W8-09` | Info | `.github/workflows/ci-rust.yml:1` |
| `W8-10` | Low | `benchmarks/update_markdown.py:1` |
