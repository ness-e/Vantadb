---
title: "DEF-06 — README↔BENCHMARKS reconciliados (claims)"
kind: task
---

# DEF-06: README↔BENCHMARKS reconciliados (claims)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 46, Fase F5 — wave F5.2, co-batch con ICP-03)
- **Fuente:** plan Task 46 · `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` §2/§5 · VER-08 (`0405eeda`, §19)
- **Esfuerzo:** 🟡 2d · **Prioridad:** 🔴 · **Tipo:** docs + una corrida de bench (sin cambio de código)
- **Ruta:** vanta-docs (+ vanta-tuner: corrida de regen)
- **Creado:** 2026-09-30 · **last-synced:** 2026-09-30
- **Branch:** develop · **Commit:** lo hace el LEAD (wave F5.2: sub-agentes NO commitean, NO self-review)
- **Estado:** ✅ TRABAJO COMPLETO (steps 1–7 ✅) — cierre = LEAD (commit local + re-review P2-01). Este sub-agente no commitea ni se auto-revisa (mandato wave F5.2)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 ejecutables por este agente; handoff: commit LEAD + re-review + filas FIND + `COMPARISON.md` (§Findings)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `README.md` (público, badge/docs entry) · `README_ES.md` (gemelo ES) · `docs/user/COMPARISON.md` (§2 cita §2 FROZEN — región de ICP-03, solo lectura) · `docs/dev/FASE-A.md` (cita el bench) · blog/docs que citan BENCHMARKS |
| Callees | `benchmarks/vantadb_local_bench.py` → bindings `vantadb` 0.7.0 (venv, release) → `src/` core (fase PUT/rebuild/search); `src/config.rs:1121-1135` (claim backend) |
| Implicaciones | 100% docs + una corrida local. No se toca `src/`, `benches/`, workflows, ni `perf-bench.yml`. El único archivo compartido con co-batch es `docs/user/COMPARISON.md` → NO se edita (región ICP-03); su staleness queda como deuda documentada |

## Impacto mapeado (Regla 0)

> Gate cumplido antes de la primera edición (archivos a **editar** leídos completos; creados sin referencias entrantes aún).

- **Archivos leídos (completos):** `README.md` (435L) · `README_ES.md` (§Benchmarks :330-409 + barrido) · `docs/user/operations/BENCHMARKS.md` (1228L — §1/§2/§5/§8/§11/§19/§Planificado en foco) · `src/config.rs` (:1090-1159, claim backend) · `benchmarks/vantadb_local_bench.py` (257L) · `benchmarks/update_markdown.py` (100L) · `docs/user/COMPARISON.md` (194L) · `docs/dev/research/mgr-19-benchmarks-baseline-suites.md` (82L) · `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 46 + recitations) · `docs/dev/tasks/VER-08.md` (293L) · `.github/workflows/perf-bench.yml` (110L) · `.config/nextest.toml` (referencia) · `.github/workflows/heavy-certification.yml` (verificado: existe; el sufijo `-50` está stale)
- **Referencias hacia dentro (lo que citan los archivos que toco):** `README.md` → `docs/user/operations/BENCHMARKS.md` (§1/§5/§2), `assets/benchmark-sift1m.svg`, scripts `install.sh/ps1`. `BENCHMARKS.md` → `tests/certification/stress_protocol.rs`, `benches/canonical_p99.rs`, `benchmarks/embed_bench.py`, `benchmarks/vantadb_local_bench.py`, `heavy-certification.yml`. `README_ES.md` → mismos destinos.
- **Referencias entrantes (grep):** `rg "BENCHMARKS.md" README* docs/user docs/dev/FASE-A.md` → README/README_ES/COMPARISON/FASE-A/FAQ; `rg "vanta_benchmark_report"` → bench script + update_markdown + README* + COMPARISON + nocturnal_suite + collect_code + research; `rg "heavy-certification-50"` → BENCHMARKS.md:23 stale (workflow renombrado a `heavy-certification.yml` en FIND-142) + docs históricos (no se tocan).
- **Veredicto impacto:** **bajo** — docs-only + una corrida local. Riesgos: (a) claim SSOT roto si README cita números nuevos sin que §2 los contenga (mitigado: §2 primero, README después); (b) región co-batch ICP-03 (COMPARISON.md) — no se toca; (c) `update_markdown.py` divergente (español) — no se toca.

## Contrato

> **Verbatim del plan (Task 46, F5):**

"claim corregido: README declara 'Fjall default; RocksDB opt-in por feature/env' consistente con `config.rs` (0 menciones de 'fallback' de storage) Y §2 regenerada: corrida `benchmarks/vantadb_local_bench.py` con bindings release + hardware documentado (Regla 11) → tabla reemplazada, banner FROZEN retirado, README puede volver a citar SDK-scope Y hardware normalizado: cada sección numérica declara su entorno y README cita solo lo citable Y 0 claims no reproducibles en README/BENCHMARKS (rg de números sin comando/fuente = 0; lo no medido queda como 'en progreso') Y `validate-docs-coverage` verde"

**Contexto verificado del bloque (no re-derivado):** claim falso vigente `README.md:201` ("Fjall (default) or RocksDB fallback") vs `src/config.rs:1121-1135` (solo `rocksdb|memory`, default Fjall, warning — sin fallback). §2 FROZEN (`BENCHMARKS.md:52-59`) con markers `BENCHMARK_METRICS_START/END`. `docs/BENCHMARKS.md` NO existe; la fuente canónica es `docs/user/operations/BENCHMARKS.md`. VER-08 ✅ (`0405eeda`) → §19 con protocolo/hardware documentado (estilo Regla 11) reutilizado como formato del env de §2. Remanente: claim :201 + regen §2 + hardware normalizado + retiro FROZEN.

## Spec (SDD — decisiones)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Mecánica de regen §2 | correr script + editar tabla a mano (inglés, mismo estilo, markers intactos) vs `update_markdown.py` | ✅ **Manual**. `update_markdown.py` emite headers en **español** y dropea los links de glosario (divergiría de la política `docs/user/operations/` = EN + del estilo actual); en CI es non-fatal y su input (`benchmark_results.json`) no coincide con lo que produce el workflow. Se documenta como deuda (FIND propuesto) |
| 2 | Alcance README_ES | arreglar vs dejarlo | ✅ **Arreglar**: `README_ES.md:352-360` aún cita el artefacto gitignored con los números 2.0/3.1 ms que MGR-19 declaró no citables para `README.md` (violación Regla 11 viva + pre-mortem F3 "más sitios sin rastrillar"). No está owned por otro task (grep plans activos = 0) |
| 3 | `COMPARISON.md` (§2.2 + nota "Removed") quedará stale post-regen | editar vs no tocar | ✅ **No tocar** — región co-batch ICP-03 ("solo LEER", mandato wave F5.2). Deuda documentada para LEAD (§Deuda) |
| 4 | Fuente de §2 post-regen | corrida local documentada vs serie CI 2026-08-12 | ✅ **Local documentada** (única regenerable hoy; CI corre shape 1000 y no commitea tabla). Serie CI 2026-08-12 → nota de retiro (vive en historia git); no se elige "el número más lindo" |
| 5 | Shape de la corrida | `--size 10000 --dim 128 --queries 1000` (comando documentado §3/README) | ✅ Fijo (no se inventa shape nuevo) |
| 6 | Hardware normalizado | §2 env table estilo §19 + fix §1 (workflow renombrado) + verificar §5/§8 | ✅ §2 gana tabla de entorno (Fecha/OS/CPU/RAM/Runtime/versión/shape); §1 cita `heavy-certification.yml` (el `-50.yml` no existe — renombrado FIND-142); §5/§8 ya declaran entorno (solo verificación) |
| 7 | `README.md:401` / `README_ES.md:401` "for CI tracking" | reformular | ✅ El JSON es gitignored y CI no lo commitea (perf-bench: `contents: read`, update non-fatal) → reformular a "local, regenerable" |
| 8 | Números en README | citar §2 con números vs solo puntero | ✅ Puntero + números clave de §1 (tabla existente) + frase explícita de que SDK-scope vive en §2 regenerada (con shape/entorno), sin mezclar scopes |

## Invariantes de dominio (handoff — MUST)

1. **No tocar** `src/**`, `benches/**`, `.github/workflows/**` (incl. `perf-bench.yml`), `CONSTRAINTS.md`, `desktop/**`, `opencode.jsonc`, `Backlog.md`, plan file.
2. **Región co-batch F5.2:** `docs/user/*` de ICP-03 (`COMPARISON.md`, one-pager nuevo) e `integrations/**` → solo lectura.
3. **Regla 11:** todo número cita comando + shape + entorno; no mezclar scopes (§1 Rust vs §2 SDK/PyO3/GIL vs canonical_p99 vs §19 harness).
4. La tabla §2 debe quedar **dentro** de `<!-- BENCHMARK_METRICS_START -->` / `<!-- BENCHMARK_METRICS_END -->`.
5. La corrida se publica **tal cual** (una corrida local; el script no tiene seed → varianza run-to-run declarada).
6. Reportería honesta: si la corrida local difiere fuerte de la serie CI congelada, se publica con etiqueta de entorno y nota de retiro de la serie vieja (no se elige el número más lindo).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR: sin deuda de código** — no se toca código. Deuda de docs detectada (no pagada acá por co-batch/mandato):
- `docs/user/COMPARISON.md` §2.2 + nota "Removed (2026-09-29)" citan §2 FROZEN → actualización quirúrgica post-regen (LEAD/ICP-03).
- `benchmarks/update_markdown.py` emite tabla en español y dropea links → candidato a FIND (alinear a EN o retirar del workflow).
- `benchmarks/vantadb_local_bench.py` no tiene seed ni metadata en el reporte JSON (fecha/commit/HW) → candidato a FIND (ya señalado en review `docs/dev/reviews/modulos/benchmarks.md`).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato §Contrato verificado por cláusula (ver §Verificación por cláusula) |
| **Commit** | Lo ejecuta el LEAD (wave F5.2). Changeset atómico listo + commit sugerido: `docs(DEF-06): README↔BENCHMARKS claims reconciled + §2 SDK regen` |
| **Release** | No aplica (docs, sin semver surface) |

## Herramientas necesarias → Skills (SDP v3, phase=BUILD)

| Skill | Fase | Justificación |
|-------|------|---------------|
| documentation-skill | BUILD | **obligatoria** (edita `docs/user/**`, READMEs, links/frontmatter/gates) |
| writing-guidelines | BUILD | revisión de prosa de claims (voz/tono/honestidad) |
| source-driven-development | BUILD | verificación de claims contra código/fuentes (config.rs, workflows, script) |
| doubt-driven-development | BUILD | claims de performance = verificación adversarial de números y scopes |

**SKILLS_CARGADAS:** campaign-executor, progreso (auto vía MCP) · documentation-skill, writing-guidelines, source-driven-development, doubt-driven-development, documentation-and-adrs.

## Steps

### Step 1: DISCOVERY (contrato + claim + §2 + fuentes) ✅
- **Archivos:** plan Task 46, MGR-19, VER-08.md, README(.es), BENCHMARKS.md, config.rs, bench script, update_markdown.py, perf-bench.yml, COMPARISON.md
- **Acción:** contrato verbatim + re-verificación claim `config.rs:1121-1135` + mapeo de §2/markers/estilo + inventario de claims numéricos README + decisión de alcance (README_ES sí; COMPARISON no)
- **Verify:** claim falso confirmado (`README.md:201`); `docs/BENCHMARKS.md` inexistente; workflow stale `heavy-certification-50.yml` detectado en `BENCHMARKS.md:23`; gates docs baseline ✅ (exit 0 ×4) ✅

### Step 2: Bindings release + smoke del harness ✅
- **Acción:** `maturin develop --release` (venv, `CARGO_BUILD_JOBS=2`; primer intento ✓ compiló pero copy bloqueado por proceso del co-batch → retry OK, cached 0.35s) + smoke `--size 50 --dim 16 --queries 5`
- **Verify:** `vantadb-py-0.7.0` editable instalado; smoke exit 0 (123.64 rec/s; HNSW p50 0.172 ms @50r) ✅

### Step 3: Corrida §2 (10k×128×1000) ✅
- **Comando:** `.venv/Scripts/python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000 --output benchmarks/vanta_benchmark_report.json`
- **Resultado (corrida única, 2026-09-30):** PUT p50 13.510 / p95 18.153 / p99 19.454 ms · 75.02 rec/s · rebuild 6.00 s (1,667 ops/s) · HNSW p50 2.372 / p95 3.016 / p99 3.329 ms · hybrid p50 5.593 / p95 6.931 / p99 12.025 ms
- **Hallazgo (doubt-check post-run):** la fila BM25 del script mide un **match vacío** en este corpus — las queries `token_N keyword_N` devuelven **0 hits** (verificado reabriendo la DB: `synthetic` → 10 hits; `token_0` / `keyword_0` → 0). Decisión: excluir la fila con nota † (precedente MGR-19) en vez de publicar 0.0082 ms como latencia léxica. La serie CI retirada mostraba 115.334 ms — no comparable (otra era/hardware)
- **Verify:** JSON (`benchmarks/vanta_benchmark_report.json`, gitignored) + stdout reproducidos arriba ✅

### Step 4: README.md — claim :201 + §Benchmarks post-regen ✅
- **Archivos:** `README.md`
- **Acción:** claim storage → "Fjall (default); RocksDB opt-in (Cargo feature `rocksdb` + `VANTADB_BACKEND=rocksdb`), in-memory via `VANTADB_BACKEND=memory`" (contra `config.rs:1121-1135` + `backend.rs:128-135`); §Benchmarks: "frozen/pending" → corrida local 2026-09-30; footnote cita §2 con números SDK (13.5 / 2.4 / 5.6 ms) + nota BM25; "realistic workloads" → "synthetic, single-threaded"; ":401 CI tracking" → local regenerable
- **Verify:** `rg "fallback" README.md` → 0 ✅

### Step 5: BENCHMARKS.md — §2 regenerada + FROZEN retirado + §1 fix + §8 env + §Planificado ✅
- **Archivos:** `docs/user/operations/BENCHMARKS.md`
- **Acción:** §2 reemplazada dentro de markers (4 filas nuevas + nota † BM25 + Environment table + Retired series CI 2026-08-12); banner FROZEN retirado; §1: workflow renombrado `heavy-certification-50.yml` → `heavy-certification.yml` (renames FIND-142) + env CI + comando reproduce; §8: línea Entorno (dummy determinístico, seed 42 + hash); §Planificado: DEF-06 → ENTREGADO
- **Verify:** `rg "FROZEN"` BENCHMARKS → solo la línea histórica de §Planificado; markers intactos (1×START + 1×END) ✅

### Step 6: README_ES.md — espejo de claims ✅
- **Archivos:** `README_ES.md`
- **Acción:** espejo de los 4 cambios EN (claim; tabla §1; footnote §2; "CI tracking" → local). El ES citaba el artefacto gitignored con números 2.0/3.1 ms prohibidos por MGR-19 (Regla 11 viva)
- **Verify:** `rg "fallback|frozen" README_ES.md` → 0 ✅

### Step 7: Gates + verificación por cláusula + RESULTADO ✅
- **Verify (mecánico):** check-links exit 0 (45/58 budget, sin links nuevos rotos) · check-docs exit 0 · gen-index --check exit 0 · validate-docs-coverage 0 gaps · markdownlint-cli2 sobre BENCHMARKS + task file + README → 0 issues (README_ES:290 MD028 preexistente = FIND-172; fuera del scope CI `docs/**`) ✅

## Verificación por cláusula (contrato verbatim → evidencia)

| # | Cláusula | Evidencia | Estado |
|---|----------|-----------|--------|
| 1 | claim corregido (`Fjall default; RocksDB opt-in por feature/env`; 0 "fallback" de storage) | `README.md:201` + `README_ES.md:200` vs `src/config.rs:1121-1135` + `src/backend.rs:128-135`; `rg "fallback" README.md README_ES.md` → 0 hits | ✅ |
| 2 | §2 regenerada (bindings release + hardware documentado) → tabla reemplazada + FROZEN retirado + README puede citar SDK-scope | `BENCHMARKS.md:50-89` (tabla nueva + Environment + Retired series); markers :54/:61; `README.md:360` cita §2 con números SDK | ✅ |
| 3 | hardware normalizado (cada sección numérica declara entorno; README cita solo lo citable) | §1 `BENCHMARKS.md:22-25` (CI ubuntu-latest + reproduce) · §2 env table :67-77 · §5 (Ryzen, ya presente) · §8 :202 (dummy, nueva); README cita §1/§2/§5 con fuentes | ✅ |
| 4 | 0 claims no reproducibles en README/BENCHMARKS | barrido `rg` de números/ms/qps en README+ES: cada número con fuente inline (§1/§2/§5) o nota; §1/§2/§5/§8 con comando/fuente; BM25 excluido con nota (no publicado) | ✅ |
| 5 | `validate-docs-coverage` verde | `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps (exit 0) | ✅ |

## Context Save Point (para LEAD — changeset del commit)

**Archivos editados por DEF-06 (solo estos):**
- `README.md` (5 cambios: claim :201 + BENCH-01 intro + §Benchmarks intro + footnote §2 + running-suite footer)
- `README_ES.md` (espejo de los 4 grupos de cambios)
- `docs/user/operations/BENCHMARKS.md` (4 bloques: §1 note/reproduce + §2 regen + §8 env + §Planificado)
- `docs/dev/tasks/DEF-06.md` (nuevo)

**⚠️ Co-batch F5.2 — NO atribuir a DEF-06 (presentes en el worktree, de ICP-03):** `docs/user/COMPARISON.md` (§9 añadida), `docs/user/FRAMEWORKS.md` (nuevo), `docs/dev/tasks/ICP-03.md` (nuevo), `.github/workflows/ci-frameworks-demo.yml` (nuevo), `examples/langgraph_dev_to_prod/**` (nuevo), y los generados `docs/index.md` / `docs/user/index.md` / `llms.txt` (regenerados con el co-batch; `gen-index --check` ✅ con ambos cambios presentes).
**Gitignored (NO commit):** `benchmarks/vanta_benchmark_report.json`, `benchmarks/data_bench_db/`, logs `$TEMP/opencode/de06-*`.
**Commit sugerido (LEAD):** `docs(DEF-06): README↔BENCHMARKS claims reconciled + §2 SDK regen (F5)`
**No tocado (región co-batch):** `docs/user/COMPARISON.md` (§2.2/nota "Removed" quedan stale → ver §Findings), one-pager ICP-03, `integrations/**`.

## Findings / disposiciones (para LEAD — Backlog prohibido en este agente)

| ID propuesto | Descripción | Dueño sugerido |
|---|---|---|
| FIND-* (a crear) | `docs/user/COMPARISON.md:47-56` (nota "Removed") + `:74` ("FROZEN pre-SIMD … regen pending") quedan **stale** tras la regen §2 → actualización quirúrgica (2 bloques; NO se tocó por ser región co-batch ICP-03, que agregó su §9 al mismo archivo) | LEAD / ICP-03 |
| FIND-* (a crear) | `benchmarks/update_markdown.py` emite tabla en español + sin links de glosario y su input (`benchmark_results.json`) no coincide con el output del workflow perf-bench (step non-fatal) → alinear a EN o retirar del workflow | vanta-lead |
| FIND-* (a crear) | `benchmarks/vantadb_local_bench.py`: dataset sin seed + reporte JSON sin metadata (fecha/commit/HW) + fila BM25 degenerada en local (queries `token_N keyword_N` → 0 hits; ¿tokenizer/analyzer?) → caracterizar antes de republicar una fila léxica | vanta-tuner / vanta-docs |
| FIND-* (a crear, sweep) | refs stale `heavy-certification-50` fuera de BENCHMARKS (p.ej. `CONTRIBUTING.md:210`, `.config/nextest.toml:3`) — renames FIND-142 no rastrillaron todo; `docs/dev/workflow/heavy-certification-50.md` SÍ existe (nombre histórico del doc) | vanta-lead |

## Review (GATE — agente distinto, P2-01)

- **Ronda 1:** ✅ **APPROVE** — 0 Critical/Required. **1 Optional aplicado:** nota de método `Throughput` en §2 (`qps = 1000/p50_ms`, indicativo; aplica también a la serie retirada). **Nits NO aplicados** (justificados): §5 sin comando inline = head histórico cubierto por stop-condition; `File` vs `VantaFile` = preexistente.
- Re-verify post-fix: check-links + check-docs + validate-docs-coverage + markdownlint verdes (ver §Fixes).
- Este sub-agente no se auto-revisa (mandato wave F5.2).

## Fixes review round 1 (2026-09-30)

| # | Fix | Evidencia (comando → resultado) |
|---|-----|-------------------------------|
| Optional | Nota de método de la columna Throughput en §2 (`qps = 1000/p50_ms` indicativo; ingestión vía JSON; rebuild derivado; mismo método en "Retired series") | `BENCHMARKS.md:67` (párrafo **Throughput method**) · `node scripts/docs/check-docs.mjs` → exit 0 · `node scripts/docs/check-links.mjs` → exit 0 · `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps · `npx markdownlint-cli2 BENCHMARKS.md` → 0 issues |

## Notas

- Wave F5.2 co-batch: `docs/user/*` de ICP-03 y `integrations/**` = solo lectura; `Backlog.md` y plan file prohibidos para este agente.
- La serie CI 2026-08-12 congelada queda retirada en §2 (nota + historia git); el README deja de declarar "frozen/pending".
- El smoke y la corrida full usan el comando documentado (§3 BENCHMARKS + README §Running the Local Benchmark Suite).
