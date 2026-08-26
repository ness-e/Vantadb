---
title: "Plan de Acción Unificado — Validación de 30 reviews (2026-08-26)"
type: plan
status: active
date: 2026-08-26
scope: Consolidación de docs/reviews/* (30 archivos) — solo pendientes VIGENTES accionables
method: lectura completa + verificación mecánica contra código/registries/backlog de cada claim (git log, grep, Test-Path, npm/PyPI/DNS/discord)
verdict: los hallazgos críticos de 2026-08-25 ya se resolvieron en su mayoría; quedan ~20 pendientes vigentes (1 P0, 7 alta, 6 media, 6 baja)
---

# Plan de Acción Unificado — VantaDB

> **Fecha de validación:** 2026-08-26 · **Branch:** develop @ `c141c1ce` + working tree (62 cambios sin commit)
> **Fuente:** consolidación de los 30 archivos de `docs/reviews/`. Cada fila indica el reporte origen y fue **verificada hoy contra el código/repo/registries**.
> **Regla:** solo se listan pendientes VIGENTES. Los hallazgos ya resueltos por commits posteriores se documentan en la sección 3 (cerrados) para no re-ticketear.

---

## 1. Pendientes vigentes priorizados

### 🔴 P0 — Distribución (bloquea adopción)

| ID propuesto | Tarea | Reporte origen | Evidencia de vigencia hoy | Esfuerzo |
|---|---|---|---|---|
| **DIST-01** | **Publicar `vantadb-node` en npm** (pipeline napi-rs, 5 targets + musl). Sigue E404 en registry. | research-vantadb-node H-01 / Backlog **BND-08** (ya existe la fila) | `npm view vantadb-node` → E404 confirmado hoy | 🔴 1-2d (requiere token npm owner) |
| **DIST-02** | **Publicar los 9 adapters Python en PyPI** (langchain/llamaindex/dspy/haystack/crewai/letta/mem0/ollama/openai) | research-integrations H-01 / **MKT-18f** | PyPI 404; `integrations/*/pyproject.toml` publicable; working tree con fixes sin commit | 🔴 medio día por paquete |
| **DIST-03** | **Cortar release 0.6.0** — triage semver del `[Unreleased]` (~650+ líneas) + release-plz | auditoria-doc SYNC-02 (D5 diferido) | tag sigue en `v0.5.0`; CHANGELOG `[Unreleased]` sin corte; workspace en 0.5.0 | 🔴 2-4h |

> Nota: DIST-01/02/03 ya están registrados en Backlog (BND-08, MKT-18f, y release diferido en D5). No duplicar filas — ejecutar los existentes.

### 🟠 Alta — Público / reputacional / funcional

| ID | Tarea | Reporte origen | Evidencia de vigencia hoy | Esfuerzo |
|---|---|---|---|---|
| **PUB-01** | **Resolver dominio `vantadb.dev` sin DNS** — decide registrar dominio o migrar contactos (security@/cla@/enterprise@) a GitHub Advisories | auditoria-raiz finding 3 | `Invoke-WebRequest https://vantadb.dev` → "Host desconocido" hoy | decisión owner |
| **PUB-02** | **Corregir tutoriales con `graph_bfs` roto** — `docs/tutorials/migration-from-lancedb.md:290` y `03-migrating-from-chromadb.md:196` usan firma `(roots, max_depth)` pero la real es `(roots, max_depth, direction)` | auditoria-doc V2.9 | Verificado hoy: `src/sdk/graph.rs:50` y `vantadb-python/src/lib.rs:1937` exigen 3 args; los snippets usan 2 | 🟢 <30 min |
| **PUB-03** | **`docs/api/MCP.md` link roto + drift de SKILL** — enlaza `skills/vantadb-mcp/SKILL.md` que no existe; los 2 SKILL.md tienen hashes distintos (0480… vs 1CE5…) | mcp-research FIND-24b | Confirmado hoy: `docs/skills/` no existe; hashes MD5 difieren | 🟢 30 min |
| **MCP-01** | **Negociación `protocolVersion`** — `initialize.rs:11` hardcodea `2024-11-05`; anunciar/eco 2025-06-18 | mcp-research P0-A / **MCP-36** | Confirmado hoy: string hardcodeado en código | 🟢 horas |
| **MCP-02** | **Tool annotations** (readOnly/destructive/idempotent/openWorld) en las ~65-75 tools — hoy 0 | mcp-research P0-C / **MCP-38** | `rg -c annotations vantadb-mcp/src/` = 0 | 🟡 1 día |
| **MCP-03** | **Perfiles de tool surface** (env `VANTADB_MCP_PROFILE`) para caber en caps de Cursor (~40) | mcp-research P0-B / **MCP-37** | ~65 `"name":` en tools.rs supera cap de Cursor | 🟡 >1 sem |
| **WEB-01** | **Decidir alcance del "dashboard embebido"** — el registro web declara UI que NO existe en `web/src` (vive en desktop). Decidir: construir, corregir registro, o desktop-only | research-web-prod H-01 | `rg -il dashboard web/src` = 0 matches hoy | decisión producto |

### 🟡 Media

| ID | Tarea | Reporte origen | Evidencia de vigencia hoy | Esfuerzo |
|---|---|---|---|---|
| **SERV-01** | **MCP-35: fallback HTTP proxy multi-instancia** — 2ª sesión muere por lock exclusivo | mcp-research P1-H / **MCP-35** | Fila Backlog `MCP-35` sigue Pendiente; incidente 2026-08-25 | 🔴 2-4d |
| **SRV-01** | **Anti path-traversal en `validate_identifier`** (no bloquea `/ \ . ..`) para `snapshot_restore` | mcp-research P1-D / **MCP-34** | `snapshot_restore` ya existe (29d21cba); falta sanitización de path | 🟢 líneas |
| **PROV-01** | **Unificar superficie de providers Rust `src/llm.rs` vs `providers/` Python** (ADR pendiente, H-14) | research-providers H-14 | Dos superficies sin contrato común; decisión de arquitectura pendiente | decisión/ADR |
| **TS-01** | **Corregir tipos de grafo ficticios** — `GraphBfsResult{visited,levels,path}` no corresponde al wire format | research-vantadb-ts H-01 / **MOD-22 / TS-01** | Confirmado hoy: `vantadb-ts/src/types.ts:208-212` aún tiene el shape ficticio | 🟡 |
| **REPORTS-01** | **Resolver contradicción northstar vs pipeline-evals** — uno reporta 100% primer-intento, el otro 0.0% + 1 regresión, del mismo evento 2026-08-11 | auditoria-doc V2.5 | Confirmado hoy: ambos stale (23/08), valores mutuamente excluyentes | 🟡 |
| **PROV-02** | **Validar/medir IVF clones per-candidate** (AUD-045) — hot-path alloc por candidato | audit-full-031011 Phase 3 / **AUD-045** | Fila Backlog Pendiente | 🟡 |

### 🟢 Baja / higiene

| ID | Tarea | Reporte origen | Evidencia de vigencia hoy | Esfuerzo |
|---|---|---|---|---|
| **CFG-01** | **CONFIGURATION.md: ejemplo `rate_limit_rpm: 100`** en :160 contradice default real 600 (:44 ya corregido) | auditoria-doc V2.2 | Confirmado hoy: :44=600, :160=100 | 🟢 1 min |
| **PY-01** | **PYTHON_RELEASE_POLICY.md: línea 16-17 aún dice "does not publish to PyPI"** — falso (0.5.0 publicado) | auditoria-doc V2.3 / V2.10 | Confirmado hoy: texto de negación persiste | 🟢 10 min |
| **NODE-01** | **`index.d.ts` con `any`** en put/search/list + tests: 8 tests para ~30 métodos | research-vantadb-node H-05/H-06 | Working tree ya agrega `tests/api.test.ts` + bench (untracked) | 🟡 |
| **WASM-01** | **Paridad de límites/score entre transports** (MAX_K 1k vs node 10k; score vs distance) | research-vantadb-wasm H-11/H-14/H-15 | Límites divergentes confirmados en código | 🟡 |
| **WEB-02** | **Commit de `web/e2e/flujo-critico.spec.ts`** — hoy existe pero untracked; sin gate | research-web-prod H-05 | `git status` muestra `?? web/e2e/` | 🟢 |
| **DESK-01** | **Desktop versión `0.1.0` hardcodeada** fuera del versionado release-plz; decisión signing/auto-update/i18n | research-desktop-prod H-11/H-06/H-08/H-10 | `desktop/package.json` sigue `0.1.0` | 🟡/decisión |

---

## 2. Verificación dinámica pendiente (medir, no afirmar)

| ID | Acción | Reporte origen |
|---|---|---|
| **MEAS-01** | Re-medir Lighthouse post-WDA-05 (−7,615 líneas) — claim "perf 96/95" sin re-medición | research-web-prod H-06 / web-design-audit |
| **MEAS-02** | Baseline de recursos del app desktop (startup/RAM) — hoy solo estimación de plataforma (Regla 11) | research-desktop-prod H-15 |
| **MEAS-03** | Benchmarks reproducibles publicados: node (vs WASM), TS/WASM path, adapters, providers | research-* (PY-02/TS-11/PERF-BENCH-01) |

---

## 3. Cerrado / superado (NO re-ticketear — verificado resuelto hoy)

Estos hallazgos de los reportes ya fueron implementados por commits posteriores a su redacción. Se documentan para evitar duplicación.

| Hallazgo del reporte | Estado real hoy | Commit/evidencia |
|---|---|---|
| AUD-043 clippy `_ns` (audit-full-010607) | ✅ RESUELTO — `let options_for = move \|_ns: String\|` | `src/cli_server.rs:1423` |
| AUD-044 mmap shim write-back | ✅ Completada en Backlog | — |
| AUD-046 truncation fan-out señal | ✅ RESUELTO — `truncated_namespaces` expuesto | `e835446a`; `cli_server.rs:3912` |
| AUD-047 layer.rs duplicación métrica | ✅ Completada en Backlog | — |
| nextest filter `python_sdk_boundary` inefectivo (SYNC-01) | ✅ RESUELTO — binario real registrado | `Cargo.toml:432`; `67384785` |
| h2 RUSTSEC-2026-0258 (review-full-200850) | ✅ RESUELTO — `h2 0.4.18` en lock | Cargo.lock |
| Dependabot sin pip (review-full-200850) | ✅ RESUELTO — ecosystem `pip` agregado | `.github/dependabot.yml` |
| CSP desktop `null` (research-desktop H-01) | ✅ RESUELTO — CSP mínima configurada | `tauri.conf.json`; `a7ed0d22` |
| Desktop sparse_vector rename (H-04) + F1/F2 + palette | ✅ RESUELTO | `a7ed0d22` |
| providers openai no compila (PROV-01) | ✅ RESUELTO — `exclude_superseded` presente | `providers/openai/src/python.rs:201`; `2754c783` |
| providers stubs .pyi stale (H-03) | ✅ RESUELTO | `2754c783` |
| MCP-34b snapshot_restore ausente | ✅ RESUELTO — tool con confirmación destructiva | `29d21cba`; `tools.rs:477` |
| vantadb-node engines/os/cpu + README (H-02/H-03) | ✅ RESUELTO | `8056dd5e` |
| vantadb-ts vitest gate + smoke-pack + async `_native` (TS-02/05/06/07/08) | ✅ RESUELTO | `c141c1ce` |
| case_studies ficticios (V2.1) | ✅ RESUELTO — archivados con disclaimer | `docs/archive/case-studies-unverified/` |
| TDAM-VANTADB vacía (GOV-03) | ✅ RESUELTO — eliminada | dir no existe |
| ADR-026 fuera de adr/ (GOV-02) | ✅ RESUELTO — movido | `docs/architecture/adr/ADR-026-*` |
| master-index congelado (IDX-01) | ✅ RESUELTO — regenerado 24/08, indexa VANTA_MEMORY/avance/research | `docs/master-index.md` |
| avance/activo sin vanta-memory/proxy (V2.7) | ✅ RESUELTO — dominios creados | `docs/avance/activo/{vanta-memory,vanta-proxy,context-engine}.md` |
| AGENTS skills count 111 vs 193 | ✅ RESUELTO | `.opencode/AGENTS.md:22` = "193 skills (162+31)" |
| Regla 2 "7 instancias" vs 8 | ✅ RESUELTO | `.opencode/AGENTS.md:456` = "8 instancias" |
| ERR-011 WAL shard truncation silenciosa | ✅ RESUELTO — replay truncado superficial | `86490dd2` |
| Docker/Dependabot/release-plz versiones (review-full-0727) | ✅ superado (posteriores audits PASS) | — |
| rate_limit CONFIGURATION :44 | ✅ ya corregido (default 600) | resto en CFG-01 (:160) |

---

## 4. Método de validación usado (2026-08-26)

- **Lectura completa** de los 30 archivos de `docs/reviews/`.
- **Verificación mecánica por claim**: `git log` (commits recientes confirman fixes), `rg`/grep en código, `Test-Path` de rutas citadas, consulta a registries (`npm view`, PyPI), DNS (`vantadb.dev`), Discord API (invite `g8nqB3NtXt` → guild válida).
- **Deduplicación contra `docs/Backlog.md`**: cada fila propuesta verifica que no exista ya (MCP-34/35/36/37/38, BND-08, MKT-18f, MOD-22, TS-01, AUD-045 ya están; no se recrean).
- **Firma de `graph_bfs`** verificada contra `src/sdk/graph.rs:50` y `vantadb-python/src/lib.rs:1937` (3 args).

## 5. Notas

- **Working tree sucio (62 cambios)** sin commit: incluye fixes de integrations (crewai/langchain/etc.), vantadb-node (tests/bench/dts), vantadb-wasm (opfs/idb/worker), web (e2e + layouts). Conviene commitearlos y relanzar `/audit full` para cobertura completa (la última audit completa fue 2026-08-25 03:10 PASS 7.3/10).
- Los hallazgos de `errors-found.md` (ERR-*) y `stabilization-report.md` son históricos (2026-08-08 / 07-18) y ya están trackeados en el Backlog P15/P13; no requieren re-acción salvo lo señalado.
- `command-system-audit` (consolidación /audit→unified-review, /build→/pipeline) es una **propuesta de proceso** sin ejecutar — no es deuda de producto; decidir si aplicar.
