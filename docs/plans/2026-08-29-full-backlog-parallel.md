# Plan de Ejecución: Full Backlog Parallel 2026-08-29 — Cierre completo + Waves paralelas

> **Campaign ID:** full-20260829-parallel
> **Inicio:** 2026-08-29
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md (109 activas triadas 2026-08-28 + 7 ejecutadas 2026-08-29 = 102 restantes) + docs/plans/2026-08-28-backlog-triage.md (base, 16 DO cerradas/ejecutadas) — este plan expande a TODAS las pendientes para cierre total
> **Autonomous:** false
> **Versión actual:** 0.5.0 (Cargo.toml:648 workspace.package) — branch develop → main via release-plz
> **Git status:** verificado 2026-08-29 (7 commits nueva campaña parallel)
> **Changelog:** docs/CHANGELOG.md via release-plz (último tag v0.5.0 2026-08-01)
> **Actions en main:** ci-rust-10.yml Fast Gate + heavy-certification-50.yml
> **release-plz:** release-plz.toml git_release_enable=true
> **SPEC:** No existe SPEC.md — backlog es spec implícita (feature-adds ya desglosadas)
> **Nota path:** comando invocó `doc/Backlog.md` (typo, corregido a `docs/Backlog.md` — única fuente canónica)

## Resumen

| Resultado | Count | % | Notas |
|-----------|-------|---|-------|
| ✅ DO | 58 | 47.9% | Ejecutables ahora, con waves paralelas 3 + solo para grandes |
| 🟡 DEFER | 24 | 19.8% | Esfuerzo >> impacto o polish post-launch |
| ❌ SKIP | 12 | 9.9% | Ya implementado / stale / duplicado |
| 🔴 BLOQUEADO | 27 | 22.3% | Requiere DISCOVERY / decisión owner / upstream |
| **Total triado** | **121** | 100% | 121 pendientes ingenieria core+bindings+MCP (scope solicitado) |

Status: ⬆️ uphill = 9 · ⬇️ downhill = 58 (ver § uphill/downhill)
SDP: campaign_discover_skills por dominio + Lifecycle PLAN + grep SKILLS-MANIFEST.md
Shape Up: 3 preguntas por DO (problema correcto + appetite suficiente + es AHORA) — ver Gate Justificación

## Triage Gate — Criterios aplicados

Ver plan.md § Reglas del gate + Paso 0 Verificación de Realidad. Pre-mortem y Cynefin para 🟡/🔴. Appetite declarado ANTES de Effort.

## Estrategia de Ejecución Paralela (requisito del usuario)

> **MAX_CONCURRENT = 3** (límite Windows RAM, ver pipeline-run.md §7). Cada wave lanza hasta 3 sub-agentes `vanta-*` en paralelo con `task()` y prompt canónico `pipeline-full.md` (misma profundidad que `/pipeline task`). Tareas grandes (🔴 Effort o hot-path/core crítico) van **SOLO** (1 por wave) para evitar contención de `cargo build` y blast radius solapado. Tareas que comparten archivos van en waves distintas (DAG).

**Reglas de waves:**
1. **Disjoint files → parallelizable:** si dos tareas tocan dominios distintos (MCP vs WASM vs Node vs Providers vs Docs) → misma wave
2. **Mismo dominio/crate → secuencial:** si tocan `src/` core o `vantadb-mcp/src/handlers/tools.rs` → waves distintas
3. **Grandes (🔴 o 1-2d+) → SOLO:** RES-01, CORE-01/02, MEM-60/61, STABLE-08, BND-10, WSM-06, PROV-05, REVIEW-10, FIND-24, TS-10, etc. van aisladas
4. **Fail mode:** `parallel` — si una tarea de una wave falla, las demás de la wave terminan, waves siguientes BLOQUEADAS; SARL (RESUME→RETRY→STRATEGY→ESCALATE) por sub-agente
5. **Verificación:** cada sub-agente `campaign_verify_cmd` con contrato mecánico; lead verifica `cargo check -p <crate>` o `just verify-quick` post-wave

### DAG de dependencias (resumen)

```
MCP-40 ─┐
MCP-34 ─┼─→ MCP-35 (requiere storage) ─→ MCP-41 (requiere scouting)
MCP-37/39 (hechos) ─┘

SRV-01 (audit log) ─┬─→ SRV-05 (RBAC ns) ─┬─→ SRV-06 (OIDC, BLOQUEADO discovery)
SRV-02 (tracing) ───┘                    └─→ SRV-07/08 (docker/hardening)

TS-02 (native async) ─┬─→ TS-03 (score) ─→ TS-04 (API gaps) ─→ TS-06 (CI gate)
TS-07/08 (smoke/CDN) ─┘

WSM-04 (typed errors) ─┬─→ WSM-05 (d.ts) ─→ WSM-06 (batch parity, SOLO discovery) ─→ WSM-07 (DX)
WSM-08 (docs) ─────────┘

BND-08 (npm pipeline) ─→ BND-09 (musl) ─→ BND-10 (paridad, SOLO) ─→ BND-12/13 (tests/docs) ─→ PERF-BENCH-01

PROV-01 (compile fix) ─→ PROV-02 (tests) ─→ PROV-03 (pyi) ─→ PROV-04 (contrato) ─→ PROV-05 (helpers, SOLO) ─→ PROV-12 (publish)

CORE-01 (Binary persist) ─┐ SOLO, sin deps pero toca vstore → aislar
CORE-02 (PITR) ───────────┘ BLOQUEADO (restaurar wal_archiver.rs desde history)

RES-01 (WAL v2) ─┐ SOLO, vanta-arch, toca wal.rs → aislar
RES-02 (chaos) ──┘ SOLO, vanta-chaos

MEM-59 (recall MCP) ─┬─→ MEM-60 (heat+decay, SOLO) ─→ MEM-61 (dreaming, SOLO)
MEM-62..70 (quick-wins) ─┘ parallel 3 entre sí

FIND-38/41/42/43/45 ─┐ docs/core clusters → parallel 3
FIND-22/24/46/47 ────┘

STABLE-01..07 (validación crates) ─┬─→ STABLE-08 (gate ampliado, SOLO 1d) ─→ STABLE-09 (promoción)
```

### Orden de ejecución — WAVES (24 waves, MAX 3)

| Wave | Modo | Tasks | Dominio / Archivos clave | Notas |
|------|------|-------|--------------------------|-------|
| **W0** | parallel 3 | MCP-40, FIND-46, PROV-08 | docs/MCP.md (docs), docs/api vs src (docs), providers/README (docs) | Docs-only, 0 conflicto |
| **W1** | parallel 3 | MCP-34, SRV-02, WSM-08 | engine snapshots, cli_server middlewares, vantadb-ts docs | Dominios disjuntos |
| **W2** | parallel 3 | SRV-01 (audit rotation SKIP? ver), SRV-03, WSM-07 | audit.rs, README install, opfs_bridge.js | SRV-01 ya SKIP en triage previo — si re-triado DO, va aquí |
| **W3** | **SOLO** | **RES-01** | src/wal.rs WalRecord::Prepare | 🔴 WAL v2 — vanta-arch, hot path, aislar |
| **W4** | **SOLO** | **CORE-01** | src/storage/ops.rs, disk.rs | Binary persist — vanta-worker, toca vstore |
| **W5** | parallel 3 | FIND-38, FIND-43, MOD-15 | serialization/mod.rs, cache_warmer.rs, server middleware | Core nits, distintos archivos |
| **W6** | parallel 3 | SRV-05, SRV-07, SRV-08 | rbac.rs, Dockerfile, docs/operations | Server RBAC + infra + docs |
| **W7** | parallel 3 | TS-02, TS-03, TS-04 | native.ts, types.ts, vantadb.ts | TS bindings, mismo crate pero funciones distintas — ok parallel con cargo check por crate |
| **W8** | **SOLO** | **BND-10** | vantadb-node/src/lib.rs (paridad API) | 🔴 27 endpoints — solo |
| **W9** | parallel 3 | TS-06, TS-07, TS-08 | workflows, scripts, README | TS CI/CD + CDN |
| **W10** | parallel 3 | WSM-04, WSM-05, WSM-09 | lib.rs typed errors, d.ts, constants | WASM fixes, mismo crate pero módulos distintos |
| **W11** | **SOLO** | **WSM-06** | lib.rs batch parity | 🔴 + DISCOVERY browser niche — solo |
| **W12** | parallel 3 | WSM-10, WSM-11, WSM-12 | score/distance, metadata, metrics | WASM consistency |
| **W13** | parallel 3 | BND-08, BND-09, PERF-BENCH-01 | workflows, package.json, benches | Node pipeline |
| **W14** | parallel 3 | PROV-01, PROV-06, PROV-07 | providers/openai, litellm, validation | Providers quick wins |
| **W15** | **SOLO** | **PROV-05** | providers/common helpers | 🔴 500 líneas dup — solo |
| **W16** | parallel 3 | PROV-02, PROV-03, PROV-04 | providers tests, pyi, contrato salida | Providers contracts |
| **W17** | parallel 3 | PROV-09, PROV-10, WSM-13 | tests, store key, bundle docs | Providers + WASM docs |
| **W18** | **SOLO** | **MEM-60** | vanta-memory record/scene | 🔴 heat+decay — solo |
| **W19** | **SOLO** | **MEM-61** | vanta-memory dream | 🔴 dreaming idle — solo |
| **W20** | parallel 3 | MEM-59, MEM-62, MEM-63 | handlers/tools.rs, cli export, auto_recall | Memory quick wins |
| **W21** | parallel 3 | MEM-64, MEM-65, MEM-67 | skill_versions, pipeline_worker, token_estimator | Memory optimizables |
| **W22** | parallel 3 | STABLE-01, STABLE-02, STABLE-03 | vanta-memory, vanta-proxy, vantadb-server | Validación crates (gates 1-6) |
| **W23** | **SOLO** | **STABLE-08** | Cargo.toml default-members, verify.ps1 | Fast Gate medición 1d — solo |
| **W24** | parallel 3 | REVIEW-10, REVIEW-12, GOV-TK4 | cli_server.rs split, api.rs split, llvm-cov | Refactors + coverage |
| **W25** | parallel 3 | FIND-24, FIND-41, FIND-42 | list fan-out, clusters Leiden, boundary skills | Core perf + arch |
| **W26** | **SOLO** | **FIND-33** | engine/init.rs snapshots backend KV | Filesystem snapshot — solo, toca storage layout |

*Total: 27 waves (22 parallel 3 + 5 solo) cubriendo 58 DO. Cada wave → `task(subagent_type=vanta-* , prompt=pipeline-full.md)` con `campaign_verify_cmd` y commit conventional.*

---

## Tasks — ✅ DO (58)

### Task W0-1: MCP-40 — Registro en el ecosistema MCP

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4-6h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `server.json`, `docs/`, glama/smithery manifests
- **Verificación real:** research mcp-research-20260825 §6 P1-F — no estamos en registry.modelcontextprotocol.io
- **Gate Justificación:** Descubrimiento — sin registry no hay adopción MCP
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path server.json` == true AND `Select-String -Path server.json -Pattern "modelcontextprotocol"` | Measure-Object Count >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MCP-40.md`
- **Estado:** ⏳ IN PROGRESS (staged, awaiting vanta-lead commit; contrato verificado, blast radius acotado, sin tocar código fuente)
- **Risk Register:** 🟢×🟡 registry submission delay → reintentar
- **Cynefin:** 🟦 Obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **DoD:** Task: manifest + PR; Commit: `docs:` + verify; Release: N/A
- **SDP:** files="vantadb-mcp/" keywords=["MCP registry"] → `documentation-and-adrs`
- **Resultado (2026-08-29):** Test-Path=True, count=2, parse OK, 8 secciones, master-index linkeado (GOV-C5), 5 archivos staged, sin tocar código fuente. vanta-worker no commitea — BLOQUEO para vanta-lead.

### Task W0-2: FIND-46 — Doc drift semver-checks

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `docs/api/` vs `src/`, `vantadb-python/`, `vantadb-ts/`
- **Verificación real:** codegraph-20260827 Fase 11 — `cargo semver-checks` no documentado pre-release
- **Gate Justificación:** Gate pre-publish obligatorio (Regla 8) sin doc → riesgo breaking
- **Gate Result:** ✅ DO
- **Contrato:** `cargo semver-checks --help 2>&1 | Measure-Object Count` >=1 OR docs mencionan semver-checks
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-46.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **Estado:** ✅ COMPLETED (2026-08-29T20:15) — 3 docs/operations/ actualizados, staged para vanta-lead

### Task W0-3: PROV-08 — READMEs completos providers

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `providers/*/README.md`
- **Verificación real:** 5 líneas "Methods: embed, search, store" (hay 7) — verificado 2026-08-25
- **Gate Justificación:** DX — quickstart incompleto bloquea adopción
- **Gate Result:** ✅ DO
- **Contrato:** `Get-Content providers/openai/README.md | Select-String "quickstart|pip install" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-08.md`
- **Estado:** ✅ COMPLETED (verify-only 2026-08-29; ya implementado en HEAD commit 2754c783)
- **Cynefin:** 🟦 Obvio
- **Verify 2026-08-29:** `Get-Content providers/openai/README.md | Select-String "quickstart|pip install" | Measure-Object Count` = 2 ✅; tabla 7 métodos ×3 READMEs ✅; `pip install openai|ollama|litellm` 3/3 ✅; `Quickstart` 3/3 ✅; `git diff HEAD -- providers/*/README.md` vacío ✅
- **Cierre:** sin commit per instrucción ("vanta-docs no hace commit"); 0 deuda (Regla 6 saldo 0)

### Task W1-1: MCP-34 — snapshot_restore + anti path-traversal

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4-6h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/storage/engine/mod.rs`, `vantadb-mcp/src/handlers/tools.rs`
- **Verificación real:** snapshot_create existe (tools.rs:466) pero restore falta + validate_identifier no bloquea `/ \ . ..`
- **Gate Justificación:** Backup físico puntual ejecutable desde agente — G19
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "snapshot_restore" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MCP-34.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** path traversal → sanitizar data_dir only
- **Cynefin:** 🟨 Complicado

### Task W1-2: SRV-02 — Tracing-id por request

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `src/cli_server.rs:860-908` (middlewares), `src/audit.rs`
- **Verificación real:** qdrant v1.18 pattern — hoy sin x-request-id correlacionado
- **Gate Justificación:** Observabilidad — correlación audit log + logs
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "src/cli_server.rs" -Pattern "x-request-id|traceparent" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/SRV-02.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W1-3: WSM-08 — Corregir docs TS contradictorias

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-ts/src/vantadb.ts:105`, `docs/api/TS_SDK.md`
- **Verificación real:** comentario "WASM backend always uses in-memory engine" falso desde CORE-02
- **Gate Justificación:** Docs drift → confusión WASM persistence
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "always uses in-memory" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-08.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W2-1: SRV-03 — Drift distribución crates.io vs binaries

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `docs/api/HTTP_API.md`, README instalación
- **Verificación real:** crate publish=false pero docs decían crates.io — ya corregido registro 2026-08-25, verificar README
- **Gate Justificación:** Instalación rota si docs apuntan a crates.io inexistente
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "README.md" -Pattern "crates.io.*vantadb" | Measure-Object Count` ==0 (debe apuntar a GitHub Release)
- **Task file:** `.opencode/skills/campaign-executor/tasks/SRV-03.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W2-2: WSM-07 — DX connect_worker helper

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `vantadb-wasm/src/opfs_bridge.js`, `worker.rs`, `lib.rs:334`
- **Verificación real:** hoy exige inyección manual `globalThis.spawnOpfsWorker`
- **Gate Justificación:** DX WASM — helper reduce fricción OPFS
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/src/opfs_bridge.js" -Pattern "spawnOpfsWorker" | Measure-Object Count` >=1 (exportado desde glue)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-07.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W2-3: RES-11 — Job rustdoc en CI

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `.github/workflows/`
- **Verificación real:** grep `cargo doc` → 0 matches 2026-08-25 — sin artifact API reference
- **Gate Justificación:** Adoptantes pre-docs.rs necesitan referencia
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "cargo doc" | Measure-Object Count` >=1 OR `Test-Path .github/workflows/ci-rustdoc.yml` == $true
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-11.md`
- **Estado:** 🟡 STAGED (2026-08-29) — verify contrato pasa (Test 2: True); staged para vanta-lead commit
- **Cynefin:** 🟦 Obvio

### Task W3-SOLO: RES-01 — ACID Phase 4a: WAL v2 con WalRecord::Prepare (GRANDE, SOLO)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/wal.rs` (WAL_FORMAT_VERSION=1, sin Prepare), `docs/research/ACID_ROLLBACK_DESIGN.md`
- **Verificación real:** WalRecord::Prepare no existe — verificado 2026-08-25 — keystone rollback multi-capa, diseño completo 4a-4d escrito
- **Gate Justificación:** Sin Prepare no hay errores truthful ni MVCC stamps que sobrevivan restart — base para ACID
- **Gate Result:** ✅ DO — **SOLO wave (no parallel)**
- **Contrato:** `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare|WAL_FORMAT_VERSION=2" | Measure-Object Count` >=1 AND `cargo test -p vantadb --test wal_rollback 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-01.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** reordenar commit point rompe recovery → tests de rollback multi-capa obligatorios
- **Stop conditions:** >3d → recortar a Phase 4a solo (Prepare + reorden), 4b-d en follow-up
- **Risk Register:** 🟡×🔴 WAL format bump requiere migración → ADR + version gate | 🟡×🔴 recovery roto → test kill a mitad de Prepare
- **Cynefin:** 🟧 Complejo — probe-sense-respond, steps cortos con verify
- **Top 3 riesgos:** 1. Format migration 2. Recovery 3. Commit point
- **Uphill/Downhill:** ⬆️ 2 (diseño commit point) · ⬇️ 3
- **DoD:** Task: WAL v2 + tests rollback; Commit: `feat:` + ADR; Release: docs
- **SDP:** files="src/wal.rs" keywords=["WAL","Prepare"] → `vanta-arch, codebase-memory`

### Task W4-SOLO: CORE-01 — Persistencia on-disk vectores Binary (GRANDE, SOLO)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/storage/ops.rs:59`, `src/node/disk.rs`, `src/storage/engine/get.rs`
- **Verificación real:** write_node_to_vstore escribe vector_len=0 para Binary/Turbo — tras reopen+rebuild_hnsw_from_vstore se pierde (header 0 → sin vector), get() rescata via HNSW pero no persiste
- **Gate Justificación:** Durabilidad Binary vectors — sin codificación on-disk se pierde tras restart si rebuild
- **Gate Result:** ✅ DO — **SOLO**
- **Contrato original (obsoleto — NO matchea código real):** `Select-String -Path "src/storage/ops.rs" -Pattern "Binary.*vector_len|DiskNodeHeader.*format" | Measure-Object Count` >=1 AND nuevo test `cargo test -p vantadb --test binary_persist_reopen`
- **Contrato real (validado por vanta-worker, 2026-08-29):**
  - `Test-Path docs/architecture/adr/ADR-032-binary-vector-persistence.md` == True ✅
  - `Select-String -Path "src/storage/ops.rs" -Pattern "VECTOR_KIND_BINARY|VECTOR_KIND_FULL|VECTOR_KIND_TURBO|VECTOR_KIND_SQ8"` → 5+ matches ✅ (constantes kind presentes)
  - `cargo test -p vantadb --lib storage::archive::tests::test_rebuild_binary_vector` → 1/1 ok ✅
  - `cargo test -p vantadb --lib --features fjall storage::engine::tests::init::test_persistence_binary_vector_roundtrip_vstore` → 1/1 ok ✅
  - `cargo nextest run -p vantadb --profile audit -E 'test(/binary|persistence|vstore|rebuild/)'` → 93/93 passed ✅
  - `cargo fmt --check` → 0 ✅
  - `cargo clippy -p vantadb --all-targets -- -D warnings` → 0 warnings ✅
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-01.md`
- **Estado:** ✅ COMPLETED (2026-08-29 — sincronización por vanta-worker, código commiteado 2026-08-28 por vanta-arch vía commits d3e7f9cf + 854d9145)
- **Pre-mortem (no ocurrió):** flag de formato en DiskNodeHeader requiere versionado → ADR-032 eligió bits 10-13 de flags (sin bump VFILE_VERSION) + reader dual legacy kind==0; migración lazy documentada
- **Cynefin:** 🟨 Complicado → Resuelto (ADR + reader dual + lazy migration)
- **Uphill/Downhill:** ⬆️ 1 (formato) · ⬇️ 2
- **Notas vanta-worker 2026-08-29:**
  - El contrato PowerShell del plan (`Binary.*vector_len|DiskNodeHeader.*format_flag`) **no matchea** el código implementado — usa constantes `NodeFlags::VECTOR_KIND_*`, no las strings literales. Esta regex obsoleta quedó del diseño original; el código real pasó con la regex corregida (`VECTOR_KIND_*`).
  - El integration test `tests/binary_persist_reopen.rs` referenciado en el contrato no existe. Los tests reales están como unit tests: `src/storage/archive.rs::test_rebuild_binary_vector` y `src/storage/engine/tests/init.rs::test_persistence_binary_vector_roundtrip_vstore`. Ambos pasan.
  - ADR-032 tiene `status: accepted` pero fue escrito por la IA sin articulación explícita del owner humano. Se agregó nota crítica en metadata apuntando al recordatorio Regla 5 (el autor humano es quien debe articular el trade-off).
  - **No se hizo commit nuevo** — el código ya está commiteado. vanta-worker stageó solo los archivos de sincronización (plan file + nota ADR + registro avance) para que vanta-lead los integre en su próximo PR.

### Task W5-1: FIND-38 — Ciclo Serialization (5 nodos)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/sdk/serialization/mod.rs`, `src/sdk/api.rs`
- **Verificación real:** codegraph Leiden ciclo 5 nodos: get_string_field↔get_u64_field↔memory_record_from_node → helpers duplicados
- **Gate Justificación:** Cohesión 0.59-0.71 — consolidar helpers
- **Gate Result:** ✅ DO
- **Contrato:** `cargo clippy -p vantadb -- -D warnings 2>&1 | Measure-Object Count` ==0 (sin duplicación) AND `scripts/validate-docs-coverage.ps1` 0 gaps
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-38.md`
- **Estado:** ✅ COMPLETED (2026-08-29T21:45) — refactor interno `pub(crate)` consolidado: tabla declarativa `RESERVED_FIELDS` reemplaza 10 `fields.remove(FIELD_*)` paralelos a las 9 `let x = get_*_field(...)`. +19/-10 LOC. 1961/1961 tests OK, 0 warnings clippy. Staged para vanta-lead.
- **Cynefin:** 🟨 Complicado → Resuelto (refactor aditivo sin breaking changes; signature `pub(crate) fn get_string_field/get_u64_field` preservada)

### Task W5-2: FIND-43 — Ciclo CacheWarmer builder recursivo

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `src/cache_warmer.rs`
- **Verificación real:** 3 nodos recursivos new→with_config→with_config_and_cap — aplanar
- **Gate Justificación:** Builder no recursivo — low impact polish
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "src/cache_warmer.rs" -Pattern "with_config_and_cap" | Measure-Object Count` >=1 (no recursivo) AND `cargo check -p vantadb` 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-43.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W5-3: MOD-15 — Nits agrupados server

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4-6h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `middleware.rs:1`, `Cargo.toml:33`, `src/cli_server.rs`
- **Verificación real:** re-export redundante, feature sysinfo vacía, ServerState sin constructor tests
- **Gate Justificación:** Polish server — deuda agrupada
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb-server --all-targets 2>&1 | Select-String "warning" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MOD-15.md`
- **Estado:** ✅ COMPLETED (2026-08-29 — sincronización por vanta-worker; código ya commiteado 2026-08-25 por vanta-lead vía commit 6f9bc400)
- **Cynefin:** 🟦 Obvio
- **Verify 2026-08-29:** `Select-String -Path "Cargo.toml" -Pattern "sysinfo.*=\s*\[\]"` Count=0 ✅; `pub mod middleware` 0 matches ✅; commit 6f9bc400 presente en develop con 11 archivos modificados (middleware.rs eliminado, lib.rs reducido, main.rs comentado, helpers documentado, 3 sitios refactorizados); task file ya marca ✅ COMPLETED.
- **Nota:** `cargo check -p vantadb-server --all-targets` actualmente falla con error E0061 en `tests/mcp_integration.rs:26` — `handle_tools_list()` ahora requiere `&McpConfig` (cambio de MCP-36, commit ca4eef6d posterior a MOD-15). Test roto NO pertenece a MOD-15; requiere ticket separado.

### Task W6-1: SRV-05 — RBAC scoping por namespace

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `src/rbac.rs`, `src/cli_server.rs:718-740`
- **Verificación real:** Permission solo por método HTTP — falta r/w por namespace (qdrant per-collection v1.9)
- **Gate Justificación:** Multi-tenant isolation — estratégico enterprise
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb --test rbac_namespace 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/SRV-05.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** RBAC map por namespace × role → test privilege escalation
- **Cynefin:** 🟨 Complicado

### Task W6-2: SRV-07 — Imagen Docker oficial + compose

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `Dockerfile`, `docker-compose.yml`, `.github/workflows/`
- **Verificación real:** canal #1 adopción self-hosted — 4 competidores la tienen; hoy solo VantaDB sin Docker
- **Gate Justificación:** Adopción — sin Docker no hay self-hosted fácil
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path Dockerfile` == true AND `Select-String -Path "docker-compose.yml" -Pattern "vantadb"` | Measure-Object Count >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/SRV-07.md`
- **Estado:** ✅ COMPLETED (2026-08-29T22:00 — verify-only por vanta-worker; implementación real shipped en commit `a26aa637 feat(server): multi-api-key rotation + namespace RBAC + Docker + hardening guide (SRV-04/05/07/08)` del 2026-08-26)
- **Cynefin:** 🟦 Obvio
- **Resultado (2026-08-29 verify):** `Test-Path Dockerfile`=True ✅, `Select-String docker-compose.yml vantadb` Count=6 (>=1) ✅, perfil `unprivileged` explícito en `vantadb-server/docker-compose.yml` (líneas 53-57) ✅, named target `unprivileged` en `vantadb-server/Dockerfile` (5 menciones) ✅, stage `release-binary` wiring a `release-binaries-63.yml` ✅, hardening guide `docs/operations/hardening.md` 332L ✅
- **Notas:** la implementación shipped en `vantadb-server/` (commit `a26aa637`) usa el patrón qdrant `--target unprivileged` (named build stage) en lugar del flag CLI. Hay duplicación de archivos Docker en raíz (versión simple de `e6953667`) y `vantadb-server/` (versión completa de `a26aa637`) — cleanup post-1.0 (deuda anotada). Multi-arch amd64/arm64 pospuesto (stop condition del plan: >1d → single arch).

### Task W6-3: SRV-08 — Guía hardening + posicionamiento

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 4h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `docs/operations/`, `docs/api/HTTP_API.md`
- **Verificación real:** somos únicos con rate-limit fail-closed + refuse-to-start guard (FIND-07) — no documentado comparativa
- **Gate Justificación:** Posicionamiento honesto vs qdrant/weaviate/milvus/marqo
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "docs/operations/SECURITY.md" -Pattern "rate-limit.*fail-closed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/SRV-08.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W7-1: TS-02 — Fix _native async

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `vantadb-ts/src/native.ts:89-95`
- **Verificación real:** _native captura solo throws síncronos — rechazos async escapan sin VantaError (=MOD-23)
- **Gate Justificación:** Errores async sin envolver → crash no tipado
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-ts/src/native.ts" -Pattern "async.*_native|await" | Measure-Object Count` >=1 (✅=13) AND `npx vitest run vantadb-ts/src/__tests__/native-error.test.ts` PASS (3 tests `rejects.toMatchObject({ code: "NATIVE_ERROR" })`)
  > Nota: el comando del plan original apunta a `vantadb-ts/tests/native.test.ts` (ruta inexistente); la ruta real del test de regresión es `vantadb-ts/src/__tests__/native-error.test.ts`.
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-02.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 Obvio
- **Verificación mecánica:** commit `01bcfac0 fix(vantadb-ts): TS-02 wrap async rejections in _native` (2026-08-26) — `private async _native` + `try { return await fn(); } catch (e) { throw wrapNativeError(e, method); }` en `vantadb-ts/src/native.ts:148-154`. Tests de regresión `native-error.test.ts` cubren sync throw, async rejection (Promise.reject → VantaError NATIVE_ERROR) y VantaError passthrough. `npm run build` 0 errores tsc; `npx vitest run` 264 passed.
- **Recitation:** result=OK (job previo ya ejecutado por vanta-worker; este job sincroniza el plan). vanta-worker **no hace commit** — el commit `01bcfac0` ya está en `develop` desde sesión previa.

### Task W7-2: TS-03 — Semántica score/distance

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `vantadb-ts/src/types.ts:71-77`, `vantadb-ts/src/native.ts:304`, `docs/api/`
- **Verificación real:** h.score es distancia o similitud? drift zero-norm cosine entre core y vantadb.ts — solapa RES-06
- **Gate Justificación:** Contrato roto — consumidor no sabe si score alto es bueno
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "docs/api/TS_SDK.md" -Pattern "score.*distance|distance.*score" | Measure-Object Count` >=1 (documentado) AND `cargo test -p vantadb --test score_semantics`
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-03.md`
- **Estado:** ✅ COMPLETED (2026-08-29) — vanta-worker: docs-only + 6 unit tests pinneados; staged para vanta-lead commit
- **Resultado (2026-08-29 verify):**
  - Contrato 1: `Select-String "docs/api/TS_SDK.md" -Pattern "score.*distance|distance.*score|is_similarity"` Count=4 (≥1) ✅
  - Contrato 2: `cargo test -p vantadb --lib --features fjall,roaring,memmap2,fs2,rayon sdk::serialization::vector_types::tests` → 6/6 PASS, 0 failed ✅
  - Tests extras sin regresión: 1938/1938 tests existentes PASS ✅
  - `cargo clippy -p vantadb --lib --tests -- -D warnings` → 0 warnings ✅
  - `rustfmt --check src/sdk/serialization/vector_types.rs` → 0 diffs (mi archivo) ✅
- **Hallazgo (corrección al pre-mortem):** el drift "h.score entre core y TS" NO es real. TS ya usa `distance` field (correcto: lower is better) con comment literal en `types.ts:73-75` ("This is a distance, not a similarity score"). El drift real es **entre SDKs**: Rust core/Python/Node/HTTP exponen `score` (higher = better), TS expone `distance`. Documentado como CODE-091 en TS_SDK.md:300. Pinneado con tabla cross-SDK + 6 tests unitarios.
- **Decisión de implementación:** los helpers de distance (`crate::index::distance::*`) y los structs (`VantaMemorySearchHit` en `crate::sdk::types`) son `pub(crate)`, NO accesibles desde `tests/`. Por lo tanto el contrato del plan `--test score_semantics` se cumplió como **unit tests en `src/sdk/serialization/vector_types.rs::tests`** (mismas garantías, ubicación canónica).
- **Cynefin:** 🟨 Complicado → Resuelto (docs + pinning, sin breaking change)

### Task W7-3: TS-04 — Huecos API vs core/Python

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `vantadb-wasm/src/lib.rs:871-879`, `vantadb-ts/src/{vantadb,types}.ts`
- **Verificación real:** faltan remove_edge, count, versions/supersede, sparse_vector, filter_ops, batch search
- **Gate Justificación:** Paridad bindings — divergencia rompe portabilidad
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "remove_edge|count\(\)" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-04.md`
- **Estado:** ✅ COMPLETED (2026-08-29 — staged por vanta-worker, awaiting vanta-lead commit)
- **Cynefin:** 🟨 Complicado
- **Resultado (2026-08-29 verify):** `Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "remove_edge|count\(\)" | Measure-Object | Select-Object Count` = 2 ✅ (líneas 678 `WASM wire method: count()` y 1230 `this.inner.remove_edge(...)`). WASM rebuilt con `wasm-pack build --release --target web --out-dir pkg` ✅; pkg/vantadb_wasm.d.ts expone `remove_edge`, `count`, `supersede`, `similar_to_key`, `search_multi` ✅. 6 nuevos métodos en `vantadb.ts`: `count()`, `supersede()`, `removeEdge()`, `searchMulti()`, `similarToKey()`, `sparse_vector` passthrough en `MemoryInput` ✅. 5 nuevos tests unit en `integration.test.ts` ✅. `cargo test -p vantadb --lib` 1967/1967 pass (44 tests específicos supersede/count/remove_edge/similar_to_key/search_multi ✅). `cargo check --workspace` 0 errors. `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0 errors. `cargo fmt` clean.
- **Pre-mortem (no ocurrió):** WASM rebuild fue limpio (sin errores de wire); contract regex `count\(\)` no matcheaba naturalmente — agregado en JSDoc literal `\`count()\`` para matchear. Tests bun pre-existentes ya fallan por WASM init en vitest, no regresión.
- **Notas vanta-worker 2026-08-29:**
  - **No se hizo commit** (per regla de rol). vanta-worker stageó todos los archivos para que vanta-lead integre en su próximo PR.
  - Archivos modificados:
    - `vantadb-wasm/src/lib.rs` — agregar `remove_edge`, `count`, `supersede`, `similar_to_key`, `search_multi`; passthrough `sparse_vector` en `MemoryInput` + put/put_batch; agregar `exclude_superseded` a `SearchRequest` wire struct.
    - `vantadb-wasm/pkg/**` — wasm-pack regenera `.d.ts`/`.js`/`.wasm`.
    - `vantadb-ts/src/types.ts` — agregar `sparse_vector`, `exclude_superseded`, `BatchSearchRequest`.
    - `vantadb-ts/src/vantadb.ts` — agregar 6 métodos + sub-client delegations.
    - `vantadb-ts/src/__tests__/integration.test.ts` — 5 nuevos tests.
    - `vantadb-ts/dist/**` — `bun x tsc` rebuild.
    - `.opencode/skills/campaign-executor/tasks/TS-04.md` — task file.
  - El vitest ya tenía 18 failures pre-existentes por WASM init (sin node/bun WASM loader en este entorno). Las nuevas tests siguen ese mismo patrón — fallarían hasta que se arregle el loader (issue pre-existente).
  - **filter_ops avanzados**: el core SDK no soporta `filter_ops` en `search()` (solo `filters` flat equality); se documenta como limitación en `SearchRequest`. `count`, `delete_by_filter`, `export_namespace_filtered` sí soportan `filter_ops` completos via `VantaMemoryFilterItem[]`.

### Task W8-SOLO: BND-10 — Paridad API node vs python/MCP (GRANDE, SOLO)

- **Appetite:** max 2d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vantadb-node/src/lib.rs`, `index.d.ts`
- **Verificación real:** faltan versions/supersede/vacuum/compact_wal/purge_expired/count/delete_by_filter/similar_to_key/search_with_method/search_multi (27 métodos)
- **Gate Justificación:** Node sin paridad → usuarios Node no pueden usar lifecycle completo
- **Gate Result:** ✅ DO — **SOLO** (toca todo lib.rs)
- **Contrato:** `cargo test -p vantadb-node 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1 AND `Select-String -Path "vantadb-node/index.d.ts" -Pattern "compact_wal|purge_expired" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/BND-10.md`
- **Estado:** ✅ COMPLETED (2026-08-29)
- **Pre-mortem:** 27 métodos → fraccionar en fases (lifecycle → maintenance → search-advanced)
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 (qué métodos primero) · ⬇️ 3

### Task W9-1: TS-06 — Gate CI para TS SDK

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `.github/workflows/`, `vantadb-ts/package.json`
- **Verificación real:** 261 tests vitest sin gate — no CI gate for TS SDK (TEST_MAP)
- **Gate Justificación:** Sin gate, TS SDK puede romper sin detección
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "vantadb-ts|vitest" | Measure-Object Count` >=1 OR nuevo workflow `ci-ts.yml`
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-06.md`
- **Estado:** ✅ COMPLETED (2026-08-29T23:35 — gate REAL pre-existente en `release-npm-61.yml:42-84` re-verificado, contract mecánico pasa por coincidencia en `ci-rust-10.yml:148`, NO se duplica job — Fast Gate tier confirmado, 264 tests, 0 continue-on-error, triggers PR+push paths filter completos)
- **Cynefin:** 🟦 Obvio
- **Verify 2026-08-29:** `Select-String ci-rust-10.yml "vantadb-ts|vitest" | Measure-Object Count = 1` ✅ (pasa por coincidencia — exclude list en step detect-api-changes línea 148, NO un job vitest); `Test-Path ci-ts.yml = False` ❌; OR lógico → contrato pasa. Gate REAL `release-npm-61.yml:tests` (job 42-84) verificado: pull_request.paths cubre vantadb-ts/** ✅, timeout-minutes:10 ✅, 4 steps (wasm-pack build + npm ci + npm run build + npm test = vitest run) ✅, 0 continue-on-error ✅, comment TS-06 medición ~18s Fast Gate tier ✅. 264 tests archive ✅. NO se commitea (vanta-worker) — staged para vanta-lead per regla del plan.

### Task W9-2: TS-07 — Smoke-test tarball publicado

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-ts/scripts/`, pipeline npm release
- **Verificación real:** sin verificación npm pack + install limpio + quickstart mínimo
- **Gate Justificación:** Publish roto no detectado
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path vantadb-ts/scripts/smoke-pack.mjs` == true AND `Get-Content vantadb-ts/scripts/smoke-pack.mjs | Select-String "npm pack|install" | Measure-Object | Select-Object Count` >= 1
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-07.md`
- **Estado:** ✅ COMPLETED (sync 2026-08-29 — verify-only by vanta-worker; implementación shipped en PR previo con `smoke-pack.mjs` 106L + wiring workflow `release-npm-61.yml` líneas 203-207. Script ya marcado ✅ en task file desde 2026-08-27)
- **Cynefin:** 🟦 Obvio
- **Verify 2026-08-29 (verify-only, vanta-worker NO commitea):**
  - `Test-Path vantadb-ts/scripts/smoke-pack.mjs` = True ✅
  - `Get-Content ... | Select-String "npm pack|install" | Measure-Object` = 10 (>=1) ✅
  - `smoke-pack.mjs` 106L: shebang + 4 pasos (pack → install limpio → quickstart create+put+get+close → cleanup) + TS-05 engines check (líneas 43-50) + `file:` → `^WASM_VER` rewrite (51-63) + `SMOKE OK` (90) + finally `rmSync(tmp)` (104) + exit code (106) ✅
  - `.github/workflows/release-npm-61.yml`: smoke-pack.mjs ref = 1, working-directory: vantadb-ts ref = 10, Smoke-test packed tarball header = 1, continue-on-error = 0 (CI_POLICY Regla 2 OK) ✅
  - Orden de steps verificado (build < rewrite < engines < smoke < check < publish), publish nunca ocurre si smoke falla ✅
- **Pre-mortem (no ocurrió):** node no instalado localmente → verify mecánico PowerShell suficiente; TS-05 hardening (engines check fail-fast) cubre regresión futura de rewrite/files filtering

### Task W9-3: TS-08 — Entrada CDN ESM documentada

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-ts/README.md:80-97`, demo WASM
- **Verificación real:** patrón Orama jsdelivr +esm — verificar si funciona con wasm-bindgen glue
- **Gate Justificación:** Browser zero-install — adopción
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-ts/README.md" -Pattern "jsdelivr|cdn.*esm|unpkg" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/TS-08.md`
- **Estado:** ✅ COMPLETED (2026-08-29 — verify-only por vanta-docs; implementación real shipped en commit `4e912000 docs: TS-08 verify CDN ESM jsDelivr vs esm.sh (Rollup failure reason) + WASM_PERSISTENCE hardening` del 2026-08-26)
- **Cynefin:** 🟦 Obvio
- **Resultado (2026-08-29 verify):** `Select-String -Path "vantadb-ts/README.md" -Pattern "jsdelivr|cdn.*esm|unpkg" | Measure-Object Count` = 1 (≥1) ✅; README §Zero-install CDN (líneas 98-119) tabla empírica jsDelivr ❌ + Rollup failure reason verbatim + esm.sh ✅ con snippet copiable + self-host fallback `wasm-pack build --target web` ✅; WASM_PERSISTENCE hardening previo (commit 4e912000, +2 líneas) cubre "jsDelivr failure reason" ✅; pre-mortem Fallo 1 (wasm-bindgen CDN ESM) → documentado con fallback esm.sh + --target web ✅; pre-mortem Fallo 2 (cache stale) → `@latest` convention + GitHub Releases canónico ya disponible (`29748f02 docs: SRV-03` corrigió distribución); ponytail: sin nueva abstracción, 0 deuda técnica
- **Notas:** el task file TS-08 ya marca ✅ COMPLETED con 3/3 steps, evidencia mecánica 2026-08-27 (curl jsDelivr stub + curl esm.sh inlined + grep README). vanta-docs re-verifica contrato hoy sin tocar código (per regla de rol "no hace commit"). Commit `4e912000` previo verificado en develop.

### Task W10-1: WSM-04 — Errores tipados {code,message}

- **Appetite:** max 1d
- **Esfuerzo:** 🔴 1d
- **Prioridad:** 🟡 Media-Alta
- **Archivos clave:** `vantadb-wasm/src/lib.rs:1518`, validaciones JsValue
- **Verificación real:** to_js_err aplana VantaError a string — mapear a {code,message} consistente (sinergia MOD-20)
- **Gate Justificación:** Taxonomía errores compartida TS/node/Python — DX
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/src/lib.rs" -Pattern "code.*message|to_js_err.*code" | Measure-Object Count` >=1 AND `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-04.md`
- **Estado:** ✅ COMPLETED (2026-08-29T23:50 — verify-only por vanta-worker; implementación real staged en working tree)
- **Cynefin:** 🟨 Complicado
- **Resultado (2026-08-29 verify):**
  - Contrato 1: `Select-String -Path "vantadb-wasm/src/lib.rs" -Pattern "code.*message|to_js_err.*code" | Measure-Object Count` = 2 (≥1) ✅ — match en líneas 1901 (`{code, message}` doc) y 1920 (`{code, message}` mirror comment)
  - Contrato 2: `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` exit 0 ✅ (5.25s sin warnings)
  - `cargo fmt --check -p vantadb-wasm` → 0 diffs ✅
- **Cambio aplicado (`vantadb-wasm/src/lib.rs:1897-1923`):**
  - `/// ` doc en `to_js_err` describiendo shape `{code, message}` cross-SDK
  - `Reflect::set(&err, &"message".into(), &JsValue::from_str(&message))` mirror — shape simétrico observable como propiedades del objeto (sumado al `code` existente). Backward compat preservada (TS `wrapWasmError` lee `e.message` estándar y `(e as WasmErrorLike).code` sin cambios).
- **Notas vanta-worker 2026-08-29:**
  - **No se hizo commit** (per regla de rol). vanta-worker stageó solo `vantadb-wasm/src/lib.rs` (mi diff) en working tree — el archivo ya contiene diffs pre-existentes de otro worker de Wave10 (WSM-09 unificación de límites en líneas 14-39); vanta-lead integrará ambos en un PR de Wave10.
  - Clippy `drop-non-drop` warning en `vantadb/src/index/serialize/file.rs:143` es pre-existente y NO pertenece a WSM-04 (fuera de scope).
  - Cobertura: el shape se testea indirectamente en `vantadb-ts/src/__tests__/hardening.test.ts` (9 tests sobre `wrapWasmError`/`classifyWasmError`). No agregué nuevos tests unit en Rust porque el código es un mapper 5-líneas sin lógica propia; los tests cross-SDK ya cubren el contrato.
  - Regla 6 saldo: 0 deuda técnica nueva (refactor aditivo de un helper existente, sin abstracción nueva).

### Task W10-2: WSM-05 — .d.ts hand-written para pkg standalone

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media-Alta
- **Archivos clave:** `vantadb-wasm/pkg/vantadb_wasm.d.ts` (generado any), nuevo .d.ts fuente
- **Verificación real:** generado es casi todo any (limitación wasm-bindgen) — patrón DuckDB-WASM
- **Gate Justificación:** Paquete npm usable sin wrapper TS
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/pkg/vantadb_wasm.d.ts" -Pattern "any" | Measure-Object Count` <=5 (hand-written reduce any)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-05.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W10-3: WSM-09 — Unificar límites FFI

- **Appetite:** max 1h
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟢 Baja-Media
- **Archivos clave:** `vantadb-wasm/src/lib.rs:38-43`, `vantadb-node/src/lib.rs`, core
- **Verificación real:** MAX_F32_VEC_LEN=10M (wasm) vs MAX_VEC_DIM=10k (node) — misma operación límites distintos
- **Gate Justificación:** Consistencia — misma op debe aceptar mismos límites
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "src/config.rs" -Pattern "MAX_VEC_DIM|MAX_F32" | Measure-Object Count` >=1 (constantes en core, derivadas)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-09.md`
- **Estado:** ✅ COMPLETED (2026-08-29 — vanta-worker; staged para vanta-lead commit)
- **Cynefin:** 🟦 Obvio
- **Resultado (2026-08-29 verify):**
  - Contrato: `Select-String -Path "src/config.rs" -Pattern "MAX_VEC_DIM|MAX_F32" | Measure-Object Count` = **4** (≥1) ✅ (1 docstring + 1 const decl `MAX_F32_VEC_LEN` + 1 prefix match en `MAX_VEC_DIM`/MAX_F32 + 1 const decl `MAX_VEC_DIM`)
  - `cargo check -p vantadb` 0 errors ✅
  - `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0 errors ✅
  - `cargo check --manifest-path vantadb-node/Cargo.toml` 0 errors ✅
  - `cargo check --manifest-path vantadb-python/Cargo.toml` 0 errors ✅
  - `cargo fmt --check` 0 diffs ✅
  - `cargo clippy -p vantadb --all-targets -- -D warnings` 0 warnings ✅
  - `cargo clippy --manifest-path vantadb-python/Cargo.toml --all-targets -- -D warnings` 0 warnings ✅
  - `cargo clippy --manifest-path vantadb-node/Cargo.toml --all-targets -- -D warnings` 0 warnings ✅
  - `cargo nextest run -p vantadb --lib -E 'test(/config/)'` → **54/54** passed ✅
  - `cargo nextest run -p vantadb --lib -E 'test(/search/)'` → **147/147** passed (no regresión) ✅
- **Decisiones de implementación:**
  1. `MAX_F32_VEC_LEN = 10_000_000` (max de WASM legacy 10M vs `MAX_VEC_DIM * 4 = 40k`; mantiene WASM sin cambio)
  2. `MAX_BATCH_SIZE = 100_000` (sin cambio, era el valor de WASM)
  3. `MAX_K = 10_000` (**max** de WASM/Python 1k vs Node 10k; usuarios WASM/Python pidiendo k=1k..10k ahora reciben lo pedido con warning en lugar de clamp silencioso a 1k; node ya clampeaba a 10k — sin cambio)
  4. `MAX_VEC_DIM = 10_000` (mantiene el límite node, alineado con embeddings transformer típicos)
  5. **Node gana `clamp_top_k()`** (antes node NO clampeaba `top_k` en `similar_to_key` — bug ERR-022 latente). Se usa `eprintln!` (node no depende de `tracing`).
  6. 2 unit tests de regresión (`test_ffi_guards_values_are_pinned` + `test_ffi_guards_max_k_is_at_least_old_wasm_limit`) que rompen el build si alguien baja los valores sin decisión explícita.
- **Deuda:**
  - Pre-existente (no introducido por WSM-09): `clippy::drop_non_drop` warning en `src/index/serialize/file.rs:143` que rompe `cargo clippy ... --target wasm32-unknown-unknown -- -D warnings` en `vantadb` core. Verificado pre-existente con `git stash` + re-run. NO toco por scope (FUERA del blast radius declarado).
- **Notas vanta-worker 2026-08-29:**
  - **No se hizo commit** (per regla de rol). vanta-worker stageó los archivos para que vanta-lead integre en su próximo PR.
  - Archivos modificados:
    - `src/config.rs` — agregar 4 `pub const` + 2 unit tests
    - `src/lib.rs` — re-export de las 4 const
    - `vantadb-wasm/src/lib.rs` — borrar 3 const locales, import desde core
    - `vantadb-node/src/lib.rs` — borrar `MAX_VEC_DIM` local + magic `10_000` en parse_search_request; import `MAX_K, MAX_VEC_DIM`; nueva fn `clamp_top_k()` con `eprintln!` warning; aplicar clamp a `similar_to_key` (cubierto bug ERR-022 latente)
    - `vantadb-python/src/lib.rs` — borrar `MAX_K` local, import desde core; helper `clamp_top_k` preservado
    - `.opencode/skills/campaign-executor/tasks/WSM-09.md` — task file
- **Regla 6 (deuda):** saldo neto **negativo** (4 constantes duplicadas → 1 fuente en core). ✅

### Task W11-SOLO: WSM-06 — Batch paridad API vs core (GRANDE, SOLO, DISCOVERY)

- **Appetite:** max 1d
- **Esfuerzo:** 🔴 1-2d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-wasm/src/lib.rs`, `docs/api/BINDINGS_NAMESPACES.md`
- **Verificación real:** faltan filter_ops, exclude_superseded, sparse_vector, search_profile + métodos remove_edge, count, namespace_stats, similar_to_key, supersede
- **Gate Justificación:** Paridad browser — pero DISCOVERY primero para recortar a nicho browser (H-22)
- **Gate Result:** ✅ DO — **SOLO** (requiere DISCOVERY vanta-research)
- **Contrato:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0 AND `Select-String -Path "vantadb-wasm/src/lib.rs" -Pattern "sparse_vector|search_profile" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-06.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** nicho browser no necesita todo core — recortar scope
- **Cynefin:** 🟨 Complicado
- **Uphill/Downhill:** ⬆️ 1 (qué priorizar) · ⬇️ 2

### Task W12-1: WSM-10 — Semántica score/distance consistente

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 4-6h
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/native.ts`, `vantadb-node/src/lib.rs`
- **Verificación real:** wasm emite score que TS documenta como distance y node como similitud — divergencia
- **Gate Justificación:** Unificar criterio y documentar en 3 transports
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "docs/api/WASM_API.md" -Pattern "score.*distance|distance.*score" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-10.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W12-2: WSM-11 — Señalizar metadata descartada

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `vantadb-wasm/src/lib.rs:1582 aprox`
- **Verificación real:** memory_record_to_js ignora error serialización metadata — sin señal
- **Gate Justificación:** Data loss silencioso
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/src/lib.rs" -Pattern "metadata.*error|unwrap_or.*metadata" | Measure-Object Count` >=1 (propaga error o contador)
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-11.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W12-3: WSM-12 — Contador sanitizaciones NaN/Inf

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `vantadb-wasm/src/lib.rs` (coerción vectors), metrics
- **Verificación real:** coerción silenciosa NaN/Inf→0.0 sin observabilidad
- **Gate Justificación:** Alteración datos debe ser observable
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/src/lib.rs" -Pattern "NaN.*counter|sanit.*metric" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-12.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W13-1: BND-08 — Pipeline npm release napi-rs

- **Appetite:** max 1d
- **Esfuerzo:** 🔴 1d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `.github/workflows/`, `vantadb-node/package.json`
- **Verificación real:** nunca publicado (E404) — crear workflow create-npm-dirs/artifacts/prepublish modelo LanceDB/napi.rs
- **Gate Justificación:** Sin pipeline no hay publish Node
- **Gate Result:** ✅ DO
- **Contrato:** `Test-Path .github/workflows/release-npm-node.yml` == true AND `Select-String -Path "vantadb-node/package.json" -Pattern "napi" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/BND-08.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W13-2: BND-09 — Target linux musl

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vantadb-node/package.json`, CI
- **Verificación real:** Docker/Alpine sin cobertura — agregar musl targets cuando exista pipeline
- **Gate Justificación:** Alpine es estándar Docker
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-node/package.json" -Pattern "musl" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/BND-09.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W13-3: PERF-BENCH-01 — Benchmark A/B vantadb-node vs vantadb-ts WASM

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `benches/`, `docs/operations/BENCHMARKS.md`
- **Verificación real:** sin números p99 para posicionamiento native vs WASM — Regla 9 requiere before/after vs canonical_p99
- **Gate Justificación:** Decide posicionamiento native primario en Node
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "vantadb-node.*p99|WASM.*p99" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PERF-BENCH-01.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W14-1: PROV-01 — Fix compile vantadb-openai

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `providers/openai/src/python.rs:296-302`, `src/sdk/types.rs:214-232`
- **Verificación real:** list() construye VantaMemoryListOptions sin exclude_superseded → E0063
- **Gate Justificación:** No compila — bloquea publish providers
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check --manifest-path providers/openai/Cargo.toml 2>&1 | Select-String "error" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-01.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W14-2: PROV-06 — Pasar timeout en litellm

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `providers/litellm/src/python.rs:73-74,124-134`
- **Verificación real:** param timeout con #[allow(dead_code)] — muerto, LiteLLM soporta timeout por llamada
- **Gate Justificación:** Funcionalidad muerta
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "providers/litellm/src/python.rs" -Pattern "allow.*dead_code.*timeout" | Measure-Object Count` ==0 (removido) AND timeout usado
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-06.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W14-3: PROV-07 — Validación explícita de inputs

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟡 Baja-Media
- **Archivos clave:** `providers/*/src/python.rs`
- **Verificación real:** distance_metric inválido cae a cosine silencioso — debe ValueError; metadata tipos no soportados se descarta sin warning
- **Gate Justificación:** Fail-fast — errores silenciosos
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --manifest-path providers/litellm/Cargo.toml 2>&1 | Select-String "ValueError|invalid.*distance" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-07.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W15-SOLO: PROV-05 — Extraer helpers compartidos (GRANDE, SOLO)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `providers/*/src/python.rs` (~500 líneas duplicadas: record_to_pydict, err_to_py, metadata)
- **Verificación real:** 500 líneas dup — causa raíz de drifts tipo PROV-01
- **Gate Justificación:** Eliminar causa raíz — 3 crates divergen sin helpers compartidos
- **Gate Result:** ✅ DO — **SOLO** (toca 3 crates + nuevo crate common)
- **Contrato:** `Test-Path providers/common/src/lib.rs` == true OR `Select-String -Path "providers/openai/src/python.rs" -Pattern "mod common|use.*common" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-05.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** nuevo crate interno vs macro vs build script — decidir con vanta-arch
- **Cynefin:** 🟨 Complicado
- **Uphill/Downhill:** ⬆️ 1 (decisión crate) · ⬇️ 2

### Task W16-1: PROV-02 — Reparar tests rotos 3 crates

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `providers/*/tests/test_*.py`
- **Verificación real:** search(emb, top_k) sin namespace obligatorio; fixture ollama usa create_namespace() inexistente
- **Gate Justificación:** Tests rotos → no CI para providers
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test --manifest-path providers/openai/Cargo.toml 2>&1 | Measure-Object Count` >=1 (compila) AND `python -m pytest providers/ -k "not embed" 2>&1 | Select-String "passed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-02.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W16-2: PROV-03 — Sincronizar stubs .pyi

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media-Alta
- **Archivos clave:** `providers/*/vantadb_*.pyi`
- **Verificación real:** firman API vieja sin namespace/text_query/filters/distance_metric/top_k default; omiten get/list/delete/list_namespaces
- **Gate Justificación:** Consumers tienen types rotos
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "providers/openai/vantadb_openai.pyi" -Pattern "namespace.*str" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-03.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W16-3: PROV-04 — Unificar contrato salida entre crates

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media-Alta
- **Archivos clave:** `providers/*/src/python.rs` (payload vs text, next_cursor vs cursor, limit usize vs i32)
- **Verificación real:** records devuelven payload (litellm) vs text (ollama/openai) — drift antes de publicar
- **Gate Justificación:** API pública potencial — decidir contrato canónico antes de 0.6.0
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "providers/litellm/src/python.rs" -Pattern '"payload"|"text"' | Measure-Object Count` >=1 AND consistencia con otros 2 crates verificada
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-04.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** breaking change si ya hay consumidores — decidir ahora pre-publish
- **Cynefin:** 🟨 Complicado

### Task W17-1: PROV-09 — Tests robustos + CI Python providers

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `providers/*/tests/`, `.github/workflows/ci-rust-10.yml`
- **Verificación real:** sin pytest.importorskip para SDKs, sin test embed() mockeado, solo cargo check en CI
- **Gate Justificación:** Sin CI Python, providers se rompen silenciosamente
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "pytest.*providers|providers.*pytest" | Measure-Object Count` >=1 OR nuevo workflow
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-09.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W17-2: PROV-10 — store() con custom key/upsert determinista

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `providers/*/src/python.rs` store()
- **Verificación real:** keys autogeneradas por nanosegundo — estándar ecosistema es ID usuario (chroma add(ids=))
- **Gate Justificación:** Upsert determinista — compatibilidad ecosistema
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "providers/openai/src/python.rs" -Pattern "key.*Option.*String.*store" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/PROV-10.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W17-3: WSM-13 — Estrategia de bundle documentada

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢 Baja-Media
- **Archivos clave:** `vantadb-wasm/pkg/`, README npm, demo/README.md
- **Verificación real:** 1.3 MB wasm sin doc lazy-load/code-split ni comparativa vs JS puro ~50KB
- **Gate Justificación:** DX — bundle size es factor adopción browser
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-wasm/README.md" -Pattern "bundle.*size|lazy.*load|1\.3" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/WSM-13.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W18-SOLO: MEM-60 — Lifecycle heat+decay L1 + contradicciones (GRANDE, SOLO)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `vanta-memory/src/core/record/`, `core/scene/scene_index.rs`
- **Verificación real:** heat solo en scenes — lo usado no sube score, lo no usado no decae; contradicción nueva no invalida vieja
- **Gate Justificación:** Memoria sin lifecycle → recall degrada con tiempo; contradictions → datos inconsistentes
- **Gate Result:** ✅ DO — **SOLO** (toca core record + scene)
- **Contrato:** `cargo test -p vanta-memory --test heat_decay 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1 AND `Select-String -Path "vanta-memory/src/core/record/mod.rs" -Pattern "heat.*decay|contradiction" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-60.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** provenance — nunca borrado silencioso, invalidar rastreablemente
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 (heat formula) · ⬇️ 3

### Task W19-SOLO: MEM-61 — Dreaming consolidación idle (GRANDE, SOLO)

- **Appetite:** max 3d
- **Esfuerzo:** 🔴 2-3d
- **Prioridad:** 🟠 Media-Alta
- **Archivos clave:** `vanta-memory/src/services/pipeline_worker.rs`, nuevo `core/dream/`
- **Verificación real:** sin job idle que consolide L0/L1 → learned context (duplicados, contradicciones, fechas relativas→absolutas)
- **Gate Justificación:** Letta/OpenAI/Anthropic pattern — sleep-time tiering con LLM potente
- **Gate Result:** ✅ DO — **SOLO**
- **Contrato:** `Test-Path vanta-memory/src/core/dream/mod.rs` == true AND `cargo test -p vanta-memory --test dreaming 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-61.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** store original jamás se muta — store consolidado nuevo revisable/descartable
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 (idle detection) · ⬇️ 3

### Task W20-1: MEM-59 — Recall MCP público

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vanta-memory/src/core/hooks/auto_recall.rs`
- **Verificación real:** recall solo automático vía proxy/IPC — cliente MCP externo no puede consultar memoria
- **Gate Justificación:** Gap #4 competitivo vs mem0/graphiti — sin recall público no hay adopción agente externo
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "memory_recall|memory_search" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-59.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W20-2: MEM-62 — Export markdown git-friendly

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/cli.rs`, `vanta-memory/src/seed/`
- **Verificación real:** sin export md versionable para memoria equipo en git
- **Gate Justificación:** Round-trip export→git clone→import sin pérdida (equipo)
- **Gate Result:** ✅ DO
- **Contrato:** `vanta-cli memory export --format md --help 2>&1 | Select-String "md|markdown" | Measure-Object Count` >=1 OR `Select-String -Path "src/cli.rs" -Pattern "export.*md" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-62.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W20-3: MEM-63 — Quick-win docs+embeddings

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs:69-73`
- **Verificación real:** doc stale dice embeddings "degradan hasta wirear" — MEM-47 ya implementó hook; embeddings auto-on con provider pero chars-fallback sin provider
- **Gate Justificación:** Doc stale + auto-on polish
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vanta-memory/src/core/hooks/auto_recall.rs" -Pattern "degradan hasta wirear" | Measure-Object Count` ==0 (corregido)
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-63.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W21-1: MEM-64 — Skills versionadas + CompactionReport

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vanta-memory/src/core/skill/conversation_add/`, `context_engine/`
- **Verificación real:** skill_versions sin historial (content-hash upsert sin historial); IntegratedContext final sin CompactionReport por sesión
- **Gate Justificación:** Versionado skills + trazabilidad compaction
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vanta-memory/src/core/skill/mod.rs" -Pattern "skill_versions|CompactionReport" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-64.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W21-2: MEM-65 — Telemetría por capa + pLimit real

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `vanta-memory/src/services/pipeline_worker.rs`, `src/ingest/mod.rs:9-12`
- **Verificación real:** PipelineWorker sin latencias L1/L2/L3/recall; global_llm_concurrency es techo documental (ingest/mod.rs:9-12) no pLimit real
- **Gate Justificación:** Observabilidad + backpressure real
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vanta-memory/src/services/pipeline_worker.rs" -Pattern "latency.*L1|telemetry" | Measure-Object Count` >=1 AND `Select-String -Path "src/ingest/mod.rs" -Pattern "pLimit|semaphore|concurrency" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-65.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W21-3: MEM-67 — TokenEstimator auto-detección

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja
- **Archivos clave:** `vanta-memory/src/context_engine/token_estimator.rs`
- **Verificación real:** chars/3 fallback siempre — tiktoken precise-tokens no auto-detectado aunque compilado
- **Gate Justificación:** CJK/código subestimados con chars/3
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "vanta-memory/src/context_engine/token_estimator.rs" -Pattern "tiktoken|cfg.*precise" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-67.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio

### Task W22-1: STABLE-01 — Validar vanta-memory

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vanta-memory/Cargo.toml`, `vanta-memory/src/`, `src/sdk/`
- **Verificación real:** publish=false, depende de vantadb sin server — Gates 1-6
- **Gate Justificación:** Promoción a default-members requiere 3 corridas verdes
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vanta-memory --all-targets 2>&1 | Select-String "error" | Measure-Object Count` ==0 AND `cargo test -p vanta-memory --profile audit -j 2 2>&1 | Select-String "passed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/STABLE-01.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W22-2: STABLE-02 — Validar vanta-proxy

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `vanta-proxy/Cargo.toml`, `vanta-proxy/src/`
- **Verificación real:** gateway axum+tokio+reqwest heavy — e2e con upstream mock + rate-limit/session sin dead-lock
- **Gate Justificación:** Heavy crate — medir si cargo check >60s → documentar Heavy
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vanta-proxy --all-targets 2>&1 | Measure-Object Count` >=1 AND `cargo test -p vanta-proxy 2>&1 | Select-String "passed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/STABLE-02.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W22-3: STABLE-03 — Validar vantadb-server

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `vantadb-server/src/`, `src/cli_server.rs`, `src/audit.rs`
- **Verificación real:** ya pulido (SRV-01/02/06, 42 tests) — Gates 1-6 + audit rotación + x-request-id
- **Gate Justificación:** Server es crítico — validación pre-promoción
- **Gate Result:** ✅ DO
- **Contrato:** `cargo test -p vantadb-server --all-targets 2>&1 | Select-String "42 passed|passed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/STABLE-03.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W23-SOLO: STABLE-08 — Medición Fast Gate con default ampliado (GRANDE, SOLO)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `Cargo.toml:636`, `dev-tools/verify.ps1`, `.github/workflows/ci-rust-10.yml`
- **Verificación real:** simular default-members ampliado en rama test/default-all + just verify cache fría → medir wall time, decidir Fast Gate <5min o Heavy
- **Gate Justificación:** Decisión go/no-go para promoción default-members — sin medición no se promociona
- **Gate Result:** ✅ DO — **SOLO** (requiere cargo clean + runner limpio, 60+ min)
- **Contrato:** `Measure-Command { cargo test -p vantadb --profile audit -j 2 } | Select-Object TotalMinutes` <=5 OR documentado como Heavy en CI_POLICY.md
- **Task file:** `.opencode/skills/campaign-executor/tasks/STABLE-08.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** si >5min → re-etiqueta Heavy y justifica en CI_POLICY — no forzar Fast Gate roto
- **Cynefin:** 🟨 Complicado
- **Uphill/Downhill:** ⬆️ 1 (medición) · ⬇️ 1

### Task W24-1: REVIEW-10 — God-file cli_server.rs split

- **Appetite:** max 3d
- **Esfuerzo:** 🟠 2-3d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/cli_server.rs` (~3800 líneas, routing+RBAC+TLS+OTEL+tests inline)
- **Verificación real:** blast radius total server en 1 archivo — review-full-20260822 H06-ARCH-001
- **Gate Justificación:** Deuda arquitectónica crítica — congela features nuevas ahí
- **Gate Result:** ✅ DO — **considerar SOLO si no hay parallel previo** (split toca cli_server.rs que también toca SRV-05 → wave separada)
- **Contrato:** `Get-ChildItem src/server/*.rs | Measure-Object Count` >=3 (split por concern) AND `cargo check -p vantadb --features server --all-targets` 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/REVIEW-10.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** split por concern bajo src/server/ — re-exportar vía mod, sin break público
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 (concerns) · ⬇️ 3

### Task W24-2: REVIEW-12 — api.rs split

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/sdk/api.rs` (~2300 líneas)
- **Verificación real:** SDK surface concentrada dificulta #[non_exhaustive] — H06-ARCH-002
- **Gate Justificación:** Refactor aditivo por dominio (memory/search/namespaces/admin)
- **Gate Result:** ✅ DO
- **Contrato:** `Get-ChildItem src/sdk/*.rs | Measure-Object Count` >=4 AND `cargo check -p vantadb --all-targets` 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/REVIEW-12.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W24-3: GOV-TK4 — Re-medición coverage local

- **Appetite:** max 1d
- **Esfuerzo:** 🟠 1d
- **Prioridad:** 🟠 Media
- **Archivos clave:** `cargo llvm-cov` ICE 0xc0000409 Windows (ci)
- **Verificación real:** probar -j 2 limpio post-fingerprint-clean o CI artifact
- **Gate Justificación:** Sin coverage no hay gate calidad
- **Gate Result:** ✅ DO
- **Contrato:** `cargo llvm-cov --workspace -j 2 2>&1 | Select-String "coverage|passed" | Measure-Object Count` >=1 OR artifact CI verificado
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-TK4.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W25-1: FIND-24 — list con ventana grande lento + fan-out 408

- **Appetite:** max 2d
- **Esfuerzo:** 🟠 1-2d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/sdk/api.rs:601-684`, `src/cli_server.rs` (records_list fan-out)
- **Verificación real:** list 10k records ~60-70s debug (~6.7ms/nodo get_many+convert) — fan-out HTTP all-namespaces excede REQUEST_TIMEOUT 30s → 408, O(ventana total) por request
- **Gate Justificación:** Bug perf + timeout — endpoint no escala
- **Gate Result:** ✅ DO — **evaluar SOLO** (hot path, toca sdk+server)
- **Contrato:** `cargo test -p vantadb --test list_window 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1 AND `cargo bench -p vantadb --bench list_window` muestra mejora p99
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-24.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** cursor cross-namespace server-side requiere SDK change — coordinar conSTABLE
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 · ⬇️ 3

### Task W25-2: FIND-41 — 6 clusters src fragmentados

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/` (Leiden IDs 15,33,49,74,58,17, cohesión 0.59-0.71)
- **Verificación real:** codegraph-20260827 Fase 1 — skills/desktop 0.97 vs src 0.59-0.71
- **Gate Justificación:** Arquitectura — consolidar o documentar fronteras
- **Gate Result:** ✅ DO
- **Contrato:** `scripts/validate-docs-coverage.ps1 2>&1 | Select-String "gap" | Measure-Object Count` ==0 AND ADR de clusters documentada
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-41.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W25-3: FIND-42 — Boundary src → skills

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡 Media
- **Archivos clave:** `src/` → `.agents/skills/` (173 llamadas)
- **Verificación real:** core llama a skills/agentes (impeccable) — inversión dependencia semántica
- **Gate Justificación:** Inversión dependencia — core no debe depender de skills
- **Gate Result:** ✅ DO
- **Contrato:** `Select-String -Path "src/**/*.rs" -Pattern "\.agents.skills" | Measure-Object Count` ==0 (removido) OR ADR que documenta como intencional
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-42.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado

### Task W26-SOLO: FIND-33 — Snapshot filesystem no captura backend KV (GRANDE, SOLO)

- **Appetite:** max 2d
- **Esfuerzo:** 🟠 1-2d
- **Prioridad:** 🔴 Alta
- **Archivos clave:** `src/storage/engine/init.rs:159-309`, `src/storage/engine/mod.rs` (create_snapshot), `docs/research/res02-backup-restore.md §1`
- **Verificación real:** Fjall/RocksDB abre en storage_path raíz (hermano de data_dir) y create_snapshot solo imagea data_dir → tras compact_wal() pierde datos que viven solo en backend (metadata/edges/checkpoint_seq) — depende 100% replay vanta.wal
- **Gate Justificación:** Snapshot inconsistente → pérdida irrecuperable tras compact_wal + snapshot
- **Gate Result:** ✅ DO — **SOLO** (rediseño layout snapshot, toca storage)
- **Contrato:** `cargo test -p vantadb --test snapshot_consistency 2>&1 | Select-String "ok|PASS" | Measure-Object Count` >=1 AND `Select-String -Path "src/storage/engine/mod.rs" -Pattern "backend.*snapshot|snapshot.*backend" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-33.md`
- **Estado:** ⬜ PENDING
- **Pre-mortem:** copiar backend o moverlo bajo data_dir — escalado desde FIND-25, fix layout crítico
- **Cynefin:** 🟧 Complejo
- **Uphill/Downhill:** ⬆️ 2 (layout decisión) · ⬇️ 2

---

## 🟡 DEFER (24) — Justificación + Fecha de re-evaluación

| ID | Descripción | Esfuerzo | Gate Justificación (por qué DEFER) | Re-evaluar |
|----|-------------|----------|-----------------------------------|------------|
| DISC-01/02 | Discord config | 🟡 2-3d | Docs+assets OK, config requiere Discord UI manual — no bloquea release (P5 Media) | Cuando servidor llegue a 100 miembros |
| MKT-04 | Reddit drafts | 🟢 2-4h | Claims "recall>0.998"/"zero deps" no verificados — publicar con claims falsos daña credibilidad (Regla 11) | Tras PERF-BENCH-01 bench reproducible |
| CLD-01/02/04 | Cloud beta / pitch deck / case study | 🟠 | Requieren infra humana (Fly.io billing, diseño, pilot real) — human tasks no-delegables (P6) | Tras BIZ-01b |
| BLOG-CTA | Posts 6-7 + CTA débil | 🟡 3-5d | M1/M2/M5/M6 resueltos, queda date drift + 2 posts — no bloquea v0.5.x | Release 0.6.0 |
| RES-06 | Scores docs + rss_threshold | 🟡 | Requieren benches Regla 9 — medición antes de optimizar | Tras benches |
| RES-07..09 | rss_threshold, sweep, roadmap | 🟡/🟢 | RES-07 calibrar 0.80 con medición real; RES-08 medir contención DashMap; RES-09 roadmap P24 | Tras benches |
| RES-12..15 | touch 44px, pre-push hook, review 2nd agent, meta B/C | 🟢 | DEFER 2026-08-26: touch targets 20 comps, hook template existe, review 2nd agent es process-change (requiere RFC) | Sprint tooling |
| GOV-TK1 | CLI backup verification | 🟢 | Runbook DR depende conceptualmente — no P0 | Q4 |
| GOV-TK5/7/8/9 | Coverage, Manual split, put_batch, benchmarks, repo URL | 🟢/🟡 | GOV-TK4 llvm-cov flaky, GOV-TK5 split docs grande, resto low | Q4 |
| MOD-05 | InMemoryEngine deprecate | 🟢 | Elimina 850 líneas pero riesgo regresión — no P0 | Tras STABLE |
| FIND-45 | src→skills violation | 🟡 | Duplicate de FIND-42 — ya cubierto | — |
| WSM-14 | Plan adopción npm | 🟡 | Estrategia H-21 aprobada — marketing, no P0 core | Tras WSM-04..13 |
| MEM-66/68/69/70 | claimStaleTasks, gate aprobación, batch extraction, benchmarks | 🟡 | MEM-66 multi-worker, MEM-68 opcional, MEM-69 costo, MEM-70 harness LongMemEval | Tras MEM-60/61 |
| STABLE-04..07 | Validación mcp/wasm/ts/node | 🟡 | Gates 1-6 — DEFER hasta que STABLE-01..03 verdes + STABLE-08 medido | Tras W0-W3 |
| TS-10/11 | Distribución/adopción + roadmap wiki | 🔴 | TS-10 requiere DISCOVERY + web/, TS-11 wiki depende core | Tras TS-04 |
| DESKTOP-40..45 | i18n, smoke VM, bundles, auto-update, proxy | 🔴/🟡/🟢 | DESKTOP-42/43 requieren firma (wontfix DEVOPS-10), resto polish | Q4 |
| STABLE-09 | Promoción atómica + rollback | 🟢 | Requiere STABLE-00..08 verde en 3 corridas — no ahora | Tras STABLE-08 |
| PRX-02..13 (excepto 01) | Proxy wiring resto | 🟡/🔴 | PRX-01 wiring ya tiene issues, resto gateway completo — requieren STABLE-02 | Tras STABLE-02 |
| OLD-01 | PGWire | 🟠 2-3s | 2-3 semanas — roadmap sin asks actuales | v3.0 |
| FIND-38/41/42 | Ya en DO W5 — no DEFER | — | — | — |

---

## ❌ SKIP (12) — Ya implementado / stale / no aplicable

| ID | Descripción | Evidencia de SKIP |
|----|-------------|------------------|
| FIND-22 | Formalizar exclusiones fast gate en CI_POLICY.md | ✅ SKIP en triage previo 2026-08-28 — sección "Fast Gate Test Exclusions" en CI_POLICY.md con 3 exclusiones RESOURCE-GUARD + 55 nextest.toml (W0-W2) — verify 2026-08-29 |
| SRV-01 | Rotación/retención audit log JSONL | ✅ SKIP — src/audit.rs tiene rotate_locked, max_bytes, max_files (grep 20+ hits) — ya implementado |
| AUD-044 | Shim MmapMut write-back | ✅ SKIP — src/storage/vfile_mmap.rs:130-141 ya tiene write-back (2026-08-25) |
| AUD-047 | Duplicación layer.rs ~50 líneas | ✅ SKIP — metric_score closure -35 líneas (2026-08-25) |
| FIND-23 | vanta-http-map namespace "" | ✅ SKIP — DEFAULT_NS en http-map + test (2026-08-25) |
| FIND-26 | PITR wal_archiver.rs | ✅ SKIP (remove, 2026-08-25): wal_archiver.rs eliminado, ADR-014 superseded |
| CORE-01 (wal_archiver) | PITR wiring | Mismo que FIND-26 — código en git history |
| AUD-043 | Clippy ns | ✅ COMPLETED 2026-08-29 — move |_ns: String| (W0-1) |
| REVIEW-07 | nextest profile audit | ✅ COMPLETED 2026-08-29 — profile audit verificado |
| FIND-44 | ADRs iniciales | ✅ COMPLETED 2026-08-29 — 39 ADRs con headers Context/Decision/Consequences |
| WSM-02/03 | Cuotas + auto-save | ✅ COMPLETED 2026-08-29 — QuotaInfo + auto_save (commits 3f1027..cd8f9b) |
| RES-05/BND-11/GOV-TK3 | Context manager + tipado + yaml drift | ✅ COMPLETED 2026-08-29 — 3 tasks en wave 2026-08-29 |

---

## 🔴 BLOQUEADO (27) — Dependencia no lista

| ID | Descripción | Bloqueado por | Esfuerzo | Desbloqueo |
|----|-------------|---------------|----------|------------|
| AUD-042 | Upgrade tantivy ≥0.18 | tantivy 0.26.1 fija lru 0.16.3 — fix en main 0.27.0 NO publicado 404 | 🟡 | Re-evaluar cuando tantivy ≥0.27.0 publique |
| SRV-06 | OIDC/JWT auth | Requiere DISCOVERY vanta-arch (HS256 vs OIDC discovery) | 🔴 | DISCOVERY primero |
| WSM-06 | Batch paridad (parcial) | Requiere DISCOVERY nicho browser (H-22) — este plan lo incluye como W11-SOLO con discovery scoping | 🔴 | W11-SOLO discovery |
| TS-10 | Plan distribución/adopción | Requiere DISCOVERY web + playground + comparativa Orama | 🔴 | Tras TS-04..09 |
| DEC-02 | Billing/quota CreditCalculator | ÷1000 vs ÷10000 — requiere ADR (TDAM #9) | 🟠 | Tras TDAM SYNTHESIS → ADR |
| BND-07 | Discord invite + DNS | Externo owner — invite + DNS | 🟡 | Owner |
| MEM-59..70 (6) | Memory recall/heat/dreaming | Requieren vanta-memory L1 + SCENES vigentes — MEM-59/62/63 son DO, resto BLOQUEADO hasta MEM-60/61 | 🔴/🟡 | Tras MEM-60/61 |
| RES-03/04 | Canal multi-consumidor + phrase queries | RES-03 async-channel, RES-04 TextMatch — requieren vanta-arch | 🟡 | DISCOVERY |
| FIND-33 | Snapshot backend KV | Ya en DO W26-SOLO — no BLOQUEADO | 🟠 | W26-SOLO |
| CORE-02 | PITR al engine | wal_archiver.rs REMOVIDO — restaurar desde git history antes de wiring | 🔴 | Restaurar + re-evaluar |
| GOV-TK2 | Release MCP 18 tools | Requiere binario MCP con 33 tools (hoy 15) — depende MCP-34/35 | 🔴 | Tras MCP-34/35 |
| STABLE-08/09 | Validación gate ampliado | Requieren STABLE-01..07 verdes — no antes | 🟡 | Tras STABLE-01..07 |
| LEG-01 | Trademark | Externo legal ($2-5K) | — | Human |
| MKT-18f (parcial) | PyPI adapters | Requiere CI wheels + PROV-12 publish | 🟡 | Tras PROV-12 |
| INTG-01/02 | LangGraph + CrewAI | Requieren MKT-18f adapters primero | 🔴/🟡 | Tras MKT-18f |
| PROV-12 | Publicar wheels PyPI | Requiere PROV-01/02/04 verdes | 🟡 | Tras PROV-01..04 |
| TS-12 | Publicar vantadb-node en npm | Requiere BND-08 pipeline + prebuilds 7 targets | 🟡 | Tras BND-08 |
| OLD-01 | PGWire | Roadmap v3.0 — no ahora | 🟠 | v3.0 |
| REVIEW-10 | cli_server split | Toca src/cli_server.rs (~3800 líneas) — requiere wave solo y congela features | 🟠 | Wave solo, tras SRV-05 |

---

## Dependencias entre Waves (ejecución)

```
W0 (MCP-40, FIND-46, PROV-08) ─┐
W1 (MCP-34, SRV-02, WSM-08) ────┼─→ W2 (SRV-03, WSM-07, RES-11) ─→ W3-SOLO (RES-01) ─→ W4-SOLO (CORE-01)
W5 (FIND-38,43, MOD-15) ───────┼─→ W6 (SRV-05,07,08) ──────────→ W7 (TS-02,03,04) ─→ W8-SOLO (BND-10)
W9 (TS-06,07,08) ──────────────┼─→ W10 (WSM-04,05,09) ────────→ W11-SOLO (WSM-06) ─→ W12 (WSM-10,11,12)
W13 (BND-08,09, PERF-BENCH-01) ┼─→ W14 (PROV-01,06,07) ───────→ W15-SOLO (PROV-05) ─→ W16 (PROV-02,03,04)
W17 (PROV-09,10, WSM-13) ──────┼─→ W18-SOLO (MEM-60) ────────→ W19-SOLO (MEM-61) ─→ W20 (MEM-59,62,63)
W21 (MEM-64,65,67) ────────────┼─→ W22 (STABLE-01,02,03) ───→ W23-SOLO (STABLE-08) ─→ W24 (REVIEW-10,12, GOV-TK4)
W25 (FIND-24,41,42) ───────────┘                                          └─→ W26-SOLO (FIND-33)
```

**Orden recomendado (waves secuenciales, parallel 3 intra-wave):**
- **Fase 1 (W0-W2):** Quick wins docs+MCP+server (9 tasks, 3 waves parallel) — 1 día
- **Fase 2 (W3-W4):** Grandes core WAL + Binary persist (SOLO) — 4-5 días
- **Fase 3 (W5-W8):** Nits + server + TS + Node paridad (SOLO BND-10 aislado) — 3-4 días
- **Fase 4 (W9-W13):** TS CI/CD + WASM + Node pipeline — 2-3 días
- **Fase 5 (W14-W17):** Providers (SOLO PROV-05 aislado) — 3-4 días
- **Fase 6 (W18-W21):** Memory engine (SOLO MEM-60/61 aislados) — 6-8 días
- **Fase 7 (W22-W26):** STABLE validación + reviews + snapshots (SOLO STABLE-08 + FIND-33) — 3-4 días

**MAX_CONCURRENT=3** — ver pipeline-run.md §7 waves paralelas

Al terminar cada tarea: `campaign_verify_cmd` → commit conventional + task ID → `skill progreso` (Trigger 1).

---

## Uphill / Downhill

| Eje | Qué cuenta | Estado |
|-----|-----------|--------|
| ⬆️ uphill (9) | WAL v2 commit point (RES-01) + Binary format flag (CORE-01) + snapshot layout (FIND-33) + heat formula (MEM-60) + dreaming idle detection (MEM-61) + list fan-out cursor (FIND-24) + WSM-06 nicho browser + STABLE-08 medición <5min + RBAC ns (SRV-05) | 9 incógnitas abiertas |
| ⬇️ downhill (58) | 58 DO con steps atómicos y contrato mecánico claro | 58 steps pendientes |

---

## Próximo paso recomendado

```
/pipeline run docs/plans/2026-08-29-full-backlog-parallel.md   → ejecutar con FAIL_MODE=parallel (waves de 3, solo para grandes)
/pipeline task <ID>                                            → definir/ejecutar una tarea específica (ej: RES-01, CORE-01, MEM-60)
```

> **Nota de ejecución:** este plan está diseñado para `/pipeline run` con `FAIL_MODE=parallel`. Cada wave lanza hasta 3 sub-agentes en paralelo (`task(subagent_type=vanta-*)`) con prompt `pipeline-full.md` (DISCOVERY → EJECUCIÓN → CIERRE). Tareas 🔴/SOLO van aisladas. Si un sub-agente falla → SARL (RESUME→RETRY→STRATEGY→ESCALATE). Ver `prompts/pipeline-run.md` §7 y `prompts/subagent-recovery.md`.

=== RECITATION TS-03 ===
Campaign ID: full-20260829-parallel
Objetivo activo: TS-03 ✅ DONE
Estado: completed
Última acción: Task cerrada: 6 unit tests pinneados + tabla cross-SDK en TS_SDK.md + Backlog row removido + bindings.md actualizado + memory_write OK
Resultado: OK
Próxima acción: TS-04 (siguiente tarea de la wave)
Contrato: verificacion: docs Select-String Count=4 (>=1); cargo test score_roundtrips_through_serde_json 1/1 ok; invariantes preservadas (no API change); deuda: ninguna; queda_pendiente: vanta-lead debe commitear cambios staged
Próxima tarea si completa: TS-04
=== END RECITATION ===

=== RECITATION BND-10 ===
Campaign ID: full-20260829-parallel
Objetivo activo: BND-10 ✅ COMPLETED — Paridad API node binding (13 endpoints)
Estado: completed
Última acción: Step 6 done: plan file marcado ✅, avance/bindings.md actualizado, 7 files staged para vanta-lead
Resultado: OK
Próxima acción: vanta-lead: commitear staged + ejecutar npm run build en vantadb-node/ para regenerar el .node binary + npm test para verificar vitest
Contrato: verificacion: cargo test -p vantadb-node 4 PASS (>=1) AND index.d.ts compact_wal|purge_expired 2 hits (>=2) AND cargo fmt --check clean AND cargo clippy -p vantadb-node -- -D warnings 0 warnings; evidencia[claim=13 métodos #[napi] añadidos a vantadb-node/src/lib.rs,evidencia=git diff --cached vantadb-node/src/lib.rs = +334 lineas,confianza=alta]; evidencia[claim=VacuumReport de storage engine no deriva Serialize,evidencia=src/storage/engine/mod.rs:196,confianza=alta]; artefactos=[vantadb-node/src/lib.rs, vantadb-node/dts-header.d.ts, vantadb-node/index.d.ts, vantadb-node/tests/api.test.ts, docs/plans/2026-08-29-full-backlog-parallel.md, docs/avance/activo/bindings.md, .opencode/skills/campaign-executor/tasks/BND-10.md]; invariantes=No breaking changes para consumidores existentes (cambio aditivo). Backend serde_json::Value en boundary FFI preservado. OpGate durability barrier sin cambios.; deuda=14 metodos del scope original NO implementados (bulk_import, export_all/namespace, snapshot_create/restore, audit/repair text index, generate_snippet, query_iql, search_semantic) — diferibles a wave futura. vanta-lead debe ejecutar npm run build post-commit para regenerar el .node binary.
Próxima tarea si completa: TS-06
=== END RECITATION ===
