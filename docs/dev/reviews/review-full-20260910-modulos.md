---
title: "Revisión por módulo — 19 módulos — 2026-09-10"
type: review
status: archived
tags: [vantadb, review]
last_reviewed: 2026-09-15
aliases: []
related: []
---

# Revisión por módulo — 19 módulos — 2026-09-10

> **Modo:** 19 subagentes `vanta-review` (solo lectura) en waves ×3. Rama `develop`.
> **Metodología:** `unified-review` (fan-out/fan-in) + verificación mecánica por módulo.
> **Quality Gate:** ⚠️ CONDICIONAL — 0 critical, 6 High (FIND-63..68), resto medium/low.
> **Backlog sync:** creados FIND-63..81.

## Scoreboard

| Módulo | Score | Estado | Hallazgos (C/H/M/L) |
|---|---|---|---|
| benches | 7 | 🟡 | 0/1/3/3 |
| benchmarks | 7 | 🟡 | 0/0/5/4 |
| desktop | 7.5 | 🟡 | 0/1/2/2 |
| embeddings | 7 | 🟡 | 0/1/3/2 |
| examples | 8 | ✅ | 0/0/3/2 |
| Formula | 6 | 🟡 | 0/3/2/2 |
| fuzz | 7 | 🟡 | 0/0/4/3 |
| integrations | 7 | 🟡 | 1/2/3/1 |
| providers | 8 | ✅ | 0/0/2/2 |
| skills | 7 | 🟡 | 0/1/2/2 |
| vanta-memory | 8 | 🟡 | 0/1/3/3 |
| vanta-proxy | 7 | 🟡 | 0/1/4/3 |
| vantadb-mcp | 8 | 🟡 | 0/0/2/1 |
| vantadb-node | 8 | 🟡 | 0/0/2/3 |
| vantadb-python | 8.5 | ✅ | 0/0/3/3 |
| vantadb-server | 8 | ✅ | 0/0/2/2 |
| vantadb-ts | 8 | ✅ | 0/0/2/4 |
| vantadb-wasm | 7 | 🟡 | 0/1/3/1 |
| docs | 7 | 🟡 | 0/3/2/0 |
| **TOTAL** | **~7.4** | — | **1/14/51/42** |

## Informes por módulo (consolidado)

### benches (7/10) — Rust/criterion, 23 harnesses, compilan (`check --benches` 0)
Funciona; nightly los corre (9). Gaps: `ingestion_concurrent` se saltea en silencio sin `--features async-ingestion` (→ FIND-70); 17/23 sin mención en BENCHMARKS.md; sin `benches/README.md`; OOM en Windows sin `-j 2`; `list_window` sin entrada `[[bench]]`. Config: añadir flag al nightly + bloque bench + doc.

### benchmarks (7/10) — Python, 6/6 compilan, smoke BENCH-01 OK
`competitive_bench` no corre aquí (deps + 1GB datasets); `batch_vs_sequential` sin CLI (→ FIND-72); requirements sin pins; Chroma WinError32 registrado sin fix; baselines placeholder honestos. Sin CI para 5/8 scripts. Config: datasets bajo demanda, nada de secrets.

### desktop (7.5/10) — tsc + cargo check verdes
**FIND-63:** vitest 73/86 — 13 fails `localStorage` bajo Node (fix: jsdom/setup en vitest.config). Sidecar MCP dual, deep-link completo, 0 `window.confirm`/`console`, a11y canvas PASS. i18n `tt()` pervasive + 1 literal duplicado menor. Updater sin configurar = wontfix declarado. E2E 8 specs.

### embeddings (7/10) — lazy opt-in bien diseñado
`download --check` + `verify --check` + dry-run OK; sanity numérico real. **FIND-71:** pesos 3-4× por patterns duplicados; `verify_model` dummy (confuso vs `sanity_embed` real); lock 3/9; sizes README stale. `cargo check --features embed-local` bloqueado por OOM ambiental (no código). Integración: `src/llm.rs` + `l1_writer`/`l1_dedup` auto-on + desktop, todo default-off correcto.

### examples (8/10 ✅) — 14 ejemplos + notebook, todo compila
`cargo check --examples` 0 + `py_compile` 10/10 + APIs cruzadas vigentes + CI `ci-examples-12.yml` los corre. Gaps (→ FIND-74): sin `examples/README.md` índice, requirements 0.4 vs 0.5.0, TS examples fuera del árbol, QUICKSTART no enlaza.

### Formula (6/10) — fórmula real, README roto
Canales por componente mapeados (crates/bins/PyPI/npm activos; Docker manual; Scoop/apt inexistentes). **FIND-66:** README desincronizado ×3 (mcp, ARM64, --head) + script `update-homebrew-formula.sh` apunta mal + fórmula huérfana en docs. Versiones 0.5.0 sincronizadas; OIDC sin secretos.

### fuzz (7/10) — 4 targets activos, compilan, CI scheduled + gate PR
Sin corpus commiteado ni upload de crashes (→ FIND-80); docs `fuzz-40.md` omite ci-gate/fuzz-pr; sin targets HTTP/JSON/WASM; comentario bincode→postcard drift + assert WAL débil. Sanitizer default ASan.

### integrations (7/10) — 9 adapters, APIs vigentes, CI los testea
**Crítico:** fallback dspy roto sin framework (TypeError → FIND-69). Fixtures crewai/letta frágiles (disco lleno env). 7/9 deps sin upper-bound. No publicados en PyPI (release por tag). Docs 9/9 README desiguales, sin índice central. INTG-01/02 implementadas, tasks por cerrar.

### providers (8/10 ✅) — 3/3 `cargo check` verdes, superficie idéntica activa
`embed_batch` PROV-11, jerarquía errores MOD-20, tests mock. Gaps (→ FIND-73): `.pyi` omiten `key`, `ollama/README` stale, locks re-resueltos ensucian checkout, flake litellm 1 vez. Colisión nominal con `integrations/*` documentada. PROV-12 pendiente.

### skills (7/10) — 2/2 SKILL.md íntegros, referencias resuelven
Manifest 194 vs 195 SKILL.md en disco (+1); duplicados divergentes `vantadb*` (raíz vs `.opencode/`); `ponytail` referenciada sin path canónico. 63 skills a11y, obsoletas marcadas. Sin uso de registro en opencode.json necesario.

### vanta-memory (8/10) — check/test/clippy/fmt verdes (336/0)
MEM-66/68/69, MCP-41, MEM-63 activas con tests; 0 unsafe; degradación P4 consistente. **FIND-64:** CI paths no matchean `vanta-memory/` (bypass posible). Deuda declarada: wiring worker, tool 77, números MEM-70.

### vanta-proxy (7/10) — pipeline ordenado verificado, 12 features opt-in activas
Suite 273/1: **FIND-65** TTL test overflow (solo test, prod seguro). Deny advisories pre-existentes (rsa/SRV-06). Sin doc dedicada + `config.toml` mínimo (→ FIND-68). Deuda: SSE response_usage, thinking simétrico, LRU O(n) acotado.

### vantadb-mcp (8/10) — 79 tools = docs, check/clippy/fmt + lib/mcp/wiki suites verdes
Full suite >timeout (limitación). Solo comentarios stale 76→79 (→ FIND-77). P25/P26 cerradas; MCP-41-tool-77 premisa stale. Perfiles, budgets, loopback-only correctos.

### vantadb-node (8/10) — 35/35 métodos activos, tests 35/35, pack incluye *.node
E404 (no publicado, checklist TS-12 lista). OIDC + matrix 7 en workflow. Gaps (→ FIND-78): link review roto, nota engines, solo 1 prebuild local, sin tags `node-v*`.

### vantadb-python (8.5/10 ✅) — 139 passed, 52 métodos activos, stubs con anti-drift
`__array_interface__` owned-copy, jerarquía errores fina, dual `put_batch` intencional. Gaps: matriz CI 3.11/3.13 vs classifiers 3.11-14, `probe_lock_db` 268MB en checkout, README dev-paths stale.

### vantadb-server (8/10 ✅) — wrapper mínimo, 0 unwrap, HTTP_API.md par con código
45 tests (full >timeout, parcial lib+cli 2/2). JWT SRV-06 cableado. Gaps (→ FIND-81): `vanta_certification.json` legacy, `vantadb_data/` 335MB, sin README.

### vantadb-ts (8/10 ✅) — build/tsc/eslint 0, 280/280, publicado 0.5.0 == local == core
Gaps (→ FIND-79): export/import/reindex sin tests, `importRecords` débil, `db.wiki={}` intencional, `./native` no en exports, `vantadb-node` sin publicar.

### vantadb-wasm (7/10) — check host 0, toolchain presente, 51 `pub fn`
**FIND-75:** README bundle stale (1.35 vs 1.58MB), IQL parcial documentado, DTO diverge, input zero-copy pendiente, `.d.ts` src vs pkg. Tests browser-only (CI best-effort). `db.wiki` vacío por D43.

### docs (7/10) — 1458 .md, índice maestro + mdbook + markdownlint OK, fresca hoy
**FIND-67** QUICKSTART stale, **FIND-68** sin PROXY.md, **FIND-76** jwt_secret gap + link roto HTTP_API:600. 27 ADRs vigentes, CHANGELOG al día, BENCHMARKS Regla 11 con fuentes. 37 TODOs mayormente filas Backlog.

## Patrones cross-cutting
1. **Artefactos locales en árbol** (probe_lock_db 268MB, vantadb_data 335MB, data_bench 520MB, coverage/): todo gitignored pero ensucia DX y rompe tests por disco.
2. **Baselines placeholder honestos** (criterion/python): refrescar con nightly/dispatch.
3. **Stale-docs**: versiones, conteos (76→79), bundle sizes, QUICKSTART — automatizar o fechar.
4. **rustc 1.95 Windows**: crash paralelo + E0786 pagefile + link OOM — usar `-j 2` siempre (ya lesson).
5. **validate-docs-coverage roto** (parse L58): bloquea el gate docs; solo jwt_secret real pendiente.

## Recomendaciones priorizadas
1. **(High, esta iteración)** FIND-63 (vitest desktop), FIND-64 (CI paths memory), FIND-65 (TTL test), FIND-66 (Formula README), FIND-67 (QUICKSTART), FIND-68 (PROXY.md).
2. **(Medium, backlog)** FIND-69..81 según tabla (dspy, benches flag, embeddings, requirements, pyi, examples índice, wasm README, jwt_secret+link, comentarios MCP, node README, TS tests, fuzz corpus, server higiene).
3. **(Proceso)** Re-correr full suites con timeout ≥10min (mcp, server) para cerrar los 🟡 parciales.

---
*Teams de revisión: 19 subagentes vanta-review read-only (waves ×3) + fan-in orquestador. Backlog sync: FIND-63..88.

## Apendice A — Hallazgos completos por modulo (detalle verificable)

> Efecto correctivo 2026-09-10: la consolidacion resumia; abajo esta TODO lo reportado por los 19 agentes con evidencia file:linea + comando.

### benches (7) — H: ingestion_concurrent skip silencioso (FIND-70)
- M: 17/23 sin mencion en BENCHMARKS.md (6 del nightly). M: sin benches/README.md. M: list_window sin bloque [[bench]] + requiere --features server. L: 1 unreachable! defensivo; OOM Windows sin -j 2; 157 unwrap bajo allow ponytail (aceptado).
- Comandos: `cargo check --benches --jobs 2` exit 0 (46.55s); list_window resuelve por autodiscovery.

### benchmarks (7) — M x5: batch sin CLI (cuelga >120s), competitive inejecutable aqui, requirements sin pins, Chroma WinError32, baselines placeholder honestos (FIND-72)
- L: update_markdown path viejo en docstring, README omite 4 scripts, import deprecado x3.
- Comandos: py_compile 6/6 exit 0; smoke BENCH-01 OK; embed dummy exit 0.

### desktop (7.5) — H: vitest 73/86, 13 fails localStorage Node (FIND-63)
- M: MemoryLens literal duplicado; TODO consolidate-core. L: default sin puerto.
- Bien: tsc 0, cargo check 0, 0 window.confirm/console, CSP, a11y PASS, sidecar dual, deep-link, i18n pervasive, 8 specs E2E.

### embeddings (7) — H: pesos 3-4x por ALLOW_PATTERNS duplicados (FIND-71)
- M: verify_model dummy; threshold EN tautologico; .cache duplica GB. L: sizes README stale, verify.log parcial.
- Comandos: py_compile 0, download --check OK, verify --check PASS, dry-run OK. embed-local bloqueado por OOM ambiental (no codigo).

### examples (8 APROBADO) — M: requirements 0.4 vs 0.5.0; L: Rust sin cleanup, __pycache__ local (FIND-74)
- Todo compila + APIs vigentes + CI ci-examples-12. Falta indice README + enlace QUICKSTART + decision TS.

### Formula (6) — H x3 README desincronizado: mcp listado no instalado, ARM64 Planned ya servido, --head sin stanza (FIND-66)
- M: update-homebrew-formula.sh apunta mal; formula huerfana en docs. L: test minimo, versiones intencionales.
- Packaging mapeado: crates/bins/PyPI/npm/SBOM activos; Homebrew unico sin automatizar; sin Ruby en runner.

### fuzz (7) — M: sin corpus commiteado ni upload crashes; docs fuzz-40 stale; sin targets HTTP/JSON/WASM (FIND-80)
- L: comentario bincode drift, assert WAL debil, sanitizer default. Comandos: fuzz list 4/4, check bins exit 0.

### integrations (7) — C: fallback dspy TypeError sin framework (FIND-69)
- H: 7/9 deps sin upper-bound; fixtures fragiles (subdir + disco 23MB). M: dist/ sin decidir; NotImplementedError por diseno. L: sin secrets.
- Tests: langchain 27, openai/ollama 9+9 mocks; crewai/letta bloqueo disco env; resto skip sin SDK. No publicado PyPI.
- Fila nueva: pineos + fixtures + dist/ + PyPI decision (FIND-84).

### providers (8 APROBADO) — M: .pyi omiten key, ollama/README stale (FIND-73)
- L: locks re-resueltos, flake litellm 1 vez. 3/3 check verdes, superficie identica activa, ERR-PY-01 cerrado. PROV-12 pendiente.

### skills — H: manifest 194 vs 195 en disco + duplicados divergentes + ponytail sin path (ver informe; fila nueva FIND-83 cubre drift C1/H1/H2/H3 + M1 assets)
- Hallazgo cruzado: binario instalado sirve 56 vs 79 documentados (C1: test-mcp.py no aserta conteo, FIND-82); contradiccion MCP-27/29; api-ref omite 8 tools; ~ sin expandir; 0 secrets.

### vanta-memory (8) — H: CI paths no matchean el crate (FIND-64)
- M: MEM-69 wiring, tool 77, MEM-70 numeros (FIND-86). L: docs-coverage manual, sin README.
- Comandos: check/test/clippy/fmt verdes (336/0). 0 unsafe, degradacion P4.

### vanta-proxy (7) — H: TTL test overflow solo-test (FIND-65)
- M: deny advisories pre-existentes; hot path limpio; sin doc dedicada ni config.toml completo (FIND-68).
- Pipeline 12 gates verificado; 12 features opt-in activas; suite 273/1. Deuda: SSE response_usage + simetrico (FIND-88).

### vantadb-mcp (8) — M: comentarios 76-79 stale (FIND-77)
- 79 tools = docs; check/clippy/fmt + lib/mcp/wiki suites verdes; full suite >timeout (re-correr >=10min).

### vantadb-node (8) — M: link review roto + nota engines (FIND-78)
- 35/35 metodos activos, tests 35/35, pack OK, E404 coherente, OIDC listo; 1 prebuild local; sin tags node-v*.

### vantadb-python (8.5 APROBADO) — M: matriz CI 3.11/3.13 vs classifiers + probe_lock_db 268MB + firma put_batch_raw (FIND-85)
- 139 passed, 52 metodos activos, stubs anti-drift, UAF owned-copy, 0 unsafe.

### vantadb-server (8 APROBADO) — M/L: vanta_certification.json + vantadb_data 335MB + sin README (FIND-81)
- Wrapper minimo, 0 unwrap, HTTP_API.md par, JWT cableado, 45 tests (full >timeout parcial).

### vantadb-ts (8 APROBADO) — M: export/import/reindex sin tests + importRecords debil (FIND-79)
- build/tsc/eslint 0, 280/280, publicado 0.5.0 == local == core. Fila nueva: ./native exports + nota wiki (FIND-87).

### vantadb-wasm (7) — H: README bundle stale 1.35 vs 1.58MB (FIND-75)
- M: IQL parcial documentado, DTO diverge, input zero-copy pendiente. L: .d.ts src vs pkg. Check host 0, toolchain presente, tests browser-only.

### docs (7) — H: QUICKSTART stale (FIND-67), sin PROXY.md (FIND-68)
- M: jwt_secret gap + link roto HTTP_API:600 (FIND-76). 1458 md, indice + mdbook + markdownlint OK, 27 ADRs, CHANGELOG al dia, BENCHMARKS Regla 11 con fuentes. Informe completo original pegado por el usuario en el turno de correccion.

## Apendice B — Comandos globales ejecutados por los agentes
`git branch --show-current` (develop x19) · `cargo check/test/clippy/fmt -p <crate> --tests -j 2` · `python -m py_compile` · `npx tsc --noEmit` + `vitest run` + `eslint` (ts/desktop) · `npm pack --dry-run` (node) · `cargo fuzz list` · `cargo check --manifest-path` (providers/fuzz) · `Test-Path` + `Select-String` + `Get-ChildItem` en todo. Reporte registrado en `docs/reports/INDEX.md`.*
