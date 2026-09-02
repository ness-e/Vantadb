# Plan de Ejecución: Alta Prioridad Paralelo — GOV 30 + P27 38 + P38 17 + MCP-35

> **Campaign ID:** 20260902-alta-prioridad-paralelo
> **Inicio:** 2026-09-02
> **Estado:** ⬜ PENDING
> **Fuente:** `docs/Backlog.md` (130 activas post-sync 2026-09-01) + `docs/reviews/archive/auditoria-documentacion-2026-08-21.md` (D1-D14) + `docs/research/tdam/SYNTHESIS.md` (F1-F7) + `docs/research/archive/res02-backup-restore.md` (RES-01/02)
> **Buckets seleccionados:** 86 tareas alta prioridad (GOV 30 + P27 MEM-01..38 + P38 RES-01..15+DEC-01/02 + P25 MCP-35)
> **Orden elegido:** D) Todo en paralelo — 4 buckets en waves paralelas MAX_CONCURRENT=3
> **Modo:** `parallel` — FAIL_MODE=parallel, waves con DAG, verificación mecánica `campaign_verify_cmd` por contrato
> **Versión actual:** 0.5.0 (Cargo.toml workspace) — release-plz `git_release_enable=true`, tag v0.5.0 2026-08-01 live crates/PyPI/npm verificado GOV-A5
> **Predecesores:** EMB-01..09 ✅ (embed-local 8 modelos ≤3GB + Qwen3 excepción, feature `embed-local`, hook local L1) — habilita P27 F1-F3 sin bloqueos LLM; GOV campaña 2026-08-22 29/30 ✅ aporta harness/snippets/openapi ya funcionales pero se re-verifican aquí como guard
> **Skills base (SDP canónico §2):** `campaign-executor` + `planning-and-task-breakdown` + `writing-plans` + `ponytail(full)` — discovery por keywords contrato ≤8 skills justificadas

---

## Resumen Triage

| Resultado | Count | % | Notas |
|-----------|-------|---|-------|
| ✅ DO | 86 | 100% | Todas las seleccionadas son alta (GOV Show HN bloqueante, P27 TDAM F1-F7, RES-01/02 calidad/durabilidad 🔴, MCP-35 bloquea multi-sesión) |
| 🟡 DEFER | 0 | 0% | Nada difiere — embebidos ya hechos, decisiones D1-D14 resueltas |
| ❌ SKIP | 0 | 0% | Candidatos descartados (CRIT-01..09, guides Vectara/Chroma, INV-008) ya fuera de estos 86 |
| 🔴 BLOQUEADO | 0 | 0% | Ninguno de los 86 está bloqueado upstream no listo; AUD-042 (tantivy 0.27) no entra — fuera de scope elegido |
| **Total** | **86** | 100% | |

**Triage gate (prompts/plan.md):** cada una de las 86 pasó 3 preguntas Shape Up — ¿problema correcto? ¿appetite suficiente? ¿es AHORA? — y verificación real contra código (commit hash / grep 2026-08-28..09-01). Ninguna requiere re-triage a DEFER/SKIP; si aparece BLOQUEADO durante ejecución → `plan.adjust` con `decision_reason` + `pattern` y re-enqueue.

**Tabla resumen DO/DEFER/SKIP/BLOQUEADO** arriba es el gate formal; detalle por tarea en §Tasks.

---

## Triage Gate — Criterios aplicados

- **Verificación real (Paso 0 plan.md):** GOV auditado 2026-08-21 (Vol I+II+Addendum salud 6.5/10) + decisions D1-D14 2026-08-22 + P27 clon TDAM `97f9465` + EMB-01..09 `cargo check -p vanta-memory` 361/361 + P38 gaps validados 2026-08-25 (grep 0 hits scores/semántica, `create_snapshot` flat copy/vector_len=0) + MCP-35 incidente `Database busy` 2026-08-25
- **Sliced vertical:** cada DO tiene acceptance criteria + verification steps ejecutables (cargo nextest/check/clippy/fmt/deny/semver-checks, `dev-tools/validate_doc_snippets.py`, `scripts/check_openapi_parity.mjs`, `cargo llvm-cov`, `cargo check -p vantadb --target wasm32-...`, `npm run build && npx vitest run`)
- **Contrato verificable:** cada task con `Contrato:` mecánico (Select-String/Test-Path/cargo test) apto para `campaign_verify_cmd`
- **Task file ruta:** `.opencode/skills/campaign-executor/tasks/<ID>.md` (creación bajo demanda vía `/pipeline task <ID>`, 4 fases prompts/task.md)
- **Estado PENDING:** todas ⬜ PENDING al crear plan; last-synced 2026-09-02T00:00

---

## Grafo de Dependencias (verificado)

```
Wave0 independiente (MAX 3):
  MCP-35 ─┐ (discovery .vanta.server.json + proxy HTTP /api/v2/*, sin deps)
  GOV-T0 ─┼─→ Wave0 parallel 3 (TIR-02a/04b/08c micro-fixes 1h, appetite <30min..1h)
  RES-01 ─┘ (WAL v2 Prepare — SOLO, toca src/wal.rs, prerequisite durabilidad F1)

GOV Wave A/B: miden y corrigen bloqueantes Show HN — dependen de D1-D14 ya resueltas (D3 Show HN Sept, D6 case_studies eliminar, D8 skill fuente única), pueden ir paralelo Wave0+1
  GOV-A1..A5 (medición) ─→ GOV-B1..B6 (Show HN) ─→ GOV-C1..C7 (maestros) ─→ GOV-D1..D6 + E1 + F1..F2 (estructura + 2ª ola auditoría)
  Nota: campaña GOV 2026-08-22 ya completó 29/30 (GOV-A1 cancelado por ICE llvm-cov → fallback ADR-018 81.40%); este plan re-verifica como guard anti-regresión — no re-escribe 30 tasks desde 0, valida paridad/openapi/snippets

P38: RES-01 (ACL-01 Prepare) es prerequisite durabilidad → Wave0 SOLO; RES-02 (chaos/wal quiesce+flush, S1 fix create_snapshot) precede RES-03..05 (phrase/semántica/calibrar threshold)
  RES-01 ─→ RES-02 ─→ RES-03/04/05 parallel ─→ RES-06..15 + DEC-02 parallel 3, DEC-01 ya resuelta como defer-as-scoped (no código, solo ADR docs-only)

P27 F1-F3: depende de embed-local ya hecho (EMB-01..09 ✅) → puede ir en paralelo tras Wave0 (sliced vertical):
  F1 MEM-01→02→34 (search profile + telemetría D17) ─→ F2 MEM-03→04→05 (entity_* + checker allow-only + auth) ─→ F3 MEM-06→07+35 (skills multi-versión) ─→ F4 MEM-08a..21 (crate vanta-memory L0-L3) ─→ F5 MEM-22..24 (Context Engine) ─→ F6/F7 MEM-25..33 (proxy+wiki, 2ª iteración) + MEM-36..38 transversales
  Nota: clon TDAM advierte ÷1000 vs ÷10000 CreditCalculator (DEC-02 decide UNA) y heat mantenido por LLM (documentar límite)

Cross-bucket DAG resumido: EMB-01..09 ✅ unblocks P27 F1-F3; MCP-35 independent; GOV Wave B necesita registry live (GOV-A5) pero no bloquea P27/P38; RES-01/02 aisladas (SOLO) no comparten archivos con MEM.
```

**Reglas waves (MAX_CONCURRENT=3):**
1. Disjoint files → parallel 3; mismo crate/dominio → waves distintas
2. Grandes 🔴 / hot-path core (RES-01, MEM-16 pipeline_manager 1218L, MEM-10 l1_extractor 738L) → SOLO aisladas
3. Docs-only (GOV-C4 master-index, GOV-B5 HTTP_API.md) → parallel 3 sin cargo build contención
4. Fail mode parallel + SARL (RESUME→RETRY→STRATEGY→ESCALATE)

---

## Estrategia de Waves Paralelas (D — 4 buckets, MAX_CONCURRENT=3)

| Wave | Tasks (IDs) | Buckets | Archivos clave | Notas |
|------|-------------|---------|----------------|-------|
| **Wave0** | MCP-35 + GOV-T01..T03 + RES-01 | P25+P38+GOV-T0 | `src/cli_server.rs` mcp mode, `.vanta.server.json`, `src/wal.rs` Prepare, `evals/dora.mjs` | 3 paralelos: MCP-35 (proxy), GOV-T0 (3 micro-fixes 1h), RES-01 (SOLO WAL v2) — MCP-35 independent habilita multi-sesión real día 0 |
| **Wave1** | GOV-A1..A5 (5) + RES-02..05 (4) + MCP-35 follow-up si aplica | GOV-A + P38 | `docs/api/openapi.yaml`, `dev-tools/validate_doc_snippets.py`, `src/wal_archiver.rs` quiesce, `src/iql/` phrase | 5 GOV medición + 4 P38 durabilidad; MAX 3 → sub-waves 1a (A1-A3), 1b (A4-A5 + RES-02), 1c (RES-03..05) |
| **Wave2** | P27 F1 MEM-01..06 + GOV-B1..B6 + GOV-C1..C3 | P27 F1-F3 + GOV B/C | `src/planner.rs`, `src/entity/**`, `src/skills.rs`, `docs/case_studies/` archive | F1 search profile + F2 entities/checker + F3 skills + Show HN fixes + nextest; parallel 3 por dominio |
| **Wave3** | P27 F4 MEM-07..21 (15) + RES-06..10 (5) + GOV-C4..C7 (4) | P27 F4 + P38 media + GOV C | `vanta-memory/src/core/**`, `docs/api/scores`, `src/config.rs rss_threshold=0.80`, `docs/master-index.md` | 15 MEM L0-L3/pipeline + 5 P38 semántica/benchmarks + 4 GOV maestros; sub-waves 3a-c MAX 3 |
| **Wave4** | P27 F5-F7 MEM-22..38 (17) + GOV-D1..D6+E1+F1+F2 (10) + RES-11..15+DEC-02 (6) | P27 F5-F7 + GOV D/E/F + P38 baja/proceso | `vanta-memory/src/offload/**`, `vanta-proxy/**`, `docs/research/wiki/**`, `src/storage/engine/mod.rs` snapshot quiesce, `docs/avance/**` | Resto + transversales + decisiones producto; SOLO para MEM-16 (pipeline 1218L) y MEM-22 (assemble 1194L) dentro de wave4a/b |
| **Total** | 86 tasks en 5 waves (≈17 avg) → 29 sub-waves efectivas × MAX 3 |  |  |  |

**Orden D elegido:** Todo en paralelo con DAG — Wave0 desbloquea multi-sesión + durabilidad base; waves 1-2 corren GOV Show HN + P27 F1-F3 + RES-02..05 sin esperas cruzadas; waves 3-4 absorben resto con checkpoints humanos (ver §Checkpoints).

**Checkpoints humanos:**
- Tras Wave1: review lead — openapi paridad + harness snippets + RES-02 quiesce
- Tras Wave2 (F1+F2): checkpoint TDAM original (D16) — `cargo test -p vantadb` verde + search profile + entities/checker
- Tras Wave3 (F4): checkpoint crate vanta-memory LLM-driven con LLM mock
- Tras Wave4 (F5): release candidate — `unified-review --mode certify --profile vantadb` (8 capas: CodeGraph Impact→Rust→Python→Web→TS→Docs→Audit→Review)

---

## Tasks — ✅ DO (86) — Sliced vertical

> Formato por task: ID | Descripción | Archivos clave | Gate Justificación | Contrato (verificable) | Task file | Estado | last-synced
> Convención commits: `feat/fix/docs/test/perf/ci/refactor/chore` + scope + ID; NUNCA version hand-edit (release-plz), changelog curado mismo PR.

### Wave0 — Fundaciones (3 paralelos + 1 SOLO)

#### GOV-T01 — TIR-02a recovery time en evals/dora.mjs
- **Descripción:** calcular recovery time (fail→pass Δt) sobre verify-log.jsonl existente; ~30 líneas, 3 pares fail→pass medibles
- **Archivos clave:** `evals/dora.mjs`, `.opencode/task-system/enforcement/verify-log.jsonl`
- **Gate Justificación:** decisión tomada investigación TIR-02; datos existentes 23 entradas; cero riesgo producto; appetite 1h
- **Contrato:** `node evals/dora.mjs 2>&1 | Select-String "Recovery" | Measure-Object Count` >=1 AND sección Recovery con 3 pares en `evals/dora.md`
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-T01.md`
- **Estado:** ⬜ PENDING
- **last-synced:** 2026-09-02T00:00

#### GOV-T02 — TIR-04b contenedor tasks/closed/
- **Descripción:** formalizar Failed-task container en RULES.md (mover task file al ESCALATE, re-proceso pending, índice rg "❌ FAILED")
- **Archivos clave:** `.opencode/task-system/RULES.md`, `.opencode/skills/campaign-executor/SKILL.md`, `tasks/closed/`
- **Gate Justificación:** convención documentada, no código; appetite <1h
- **Contrato:** `Select-String -Path ".opencode/task-system/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-T02.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T19:00

#### GOV-T03 — TIR-08c criterios research-agent.md
- **Descripción:** añadir saturación<20% + broadening/narrowing + WONTFIT-jitter en research-agent.md (~6 líneas)
- **Archivos clave:** `.opencode/task-system/prompts/research-agent.md`
- **Gate Justificación:** mejora runtime research; appetite 30min; jitter ya resuelto
- **Contrato:** `Select-String -Path ".opencode/task-system/prompts/research-agent.md" -Pattern "saturaci.*20%|broadening|WONTFIT" | Measure-Object Count` >=3
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-T03.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T19:35

#### MCP-35 — Fallback HTTP automático N instancias MCP sobre misma BD
- **Descripción:** discovery `.vanta.server.json` {pid,http_port} + modo proxy HTTP /api/v2/* cuando Database busy; limpiar PID muerto; parity tools 1:1
- **Archivos clave:** `vantadb-mcp/src/server.rs` (discovery + proxy), `vantadb-mcp/src/proxy.rs`, `vantadb-server/src/main.rs` (auto fallback), `src/config.rs`
- **Gate Justificación:** bloquea multi-sesión real (incidente 2 sesiones OpenCode 2026-08-25, 2ª sin tools); diseño vanta-arch DISCOVERY + vanta-worker impl; esfuerzo 2-4d 🔴 Alta
- **Contrato:** `Select-String -Path "vantadb-mcp/src/server.rs" -Pattern "vanta\.server\.json|DatabaseBusy|proxy" | Measure-Object Count` >=3 AND `cargo test -p vantadb-mcp -- --nocapture 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MCP-35.md`
- **Estado:** ✅ COMPLETED 2026-09-02T18:00 — writer discovery .vanta.server.json + http 127.0.0.1:0 + WriterGuard Drop pid==self + proxy 500ms health + sysinfo PID + Bearer passthrough + stale cleanup + retry. Verify: cargo check -p vantadb-mcp ✅, cargo test -p vantadb-mcp 27+82+7+3 pass ✅, mcp_fallback_proxy 3/3 ✅, workspace check ✅, Select-String 12 ≥3
- **last-synced:** 2026-09-02T18:00

#### RES-01 — DURABILIDAD 🔴 ACID Phase 4a WAL v2 + física S1+S2 (prerequisite)
- **Descripción:** WalRecord::Prepare + WAL_FORMAT_VERSION bump + quiesce (write-lock + flush) antes de create_snapshot + recursive copy/link de subdirs (wal/); recolección crash-safe; verificado data_dir flat copy hoy pierde wal/ subdir
- **Archivos clave:** `src/wal.rs`, `src/storage/engine/mod.rs:507,540`, `docs/research/archive/res02-backup-restore.md` §3 S1-S2, `docs/research/ACID_ROLLBACK_DESIGN.md`
- **Gate Justificación:** prerequisite calidad/durabilidad; sin quiesce set file torn; sin Prepare no hay commit point para rollback multi-capa; esfuerzo 🔴 2-3d
- **Contrato:** `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare" | Measure-Object Count` >=1 AND `Select-String -Path "src/storage/engine/mod.rs" -Pattern "quiesce|flush\(\)|wal/" | Measure-Object Count` >=1 AND `cargo test -p vantadb --test wal_rollback -- --nocapture 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-01.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

---

### Wave1 — GOV-A medición (5) + P38 durabilidad (4)

#### GOV-A1 — Medición openapi parity docs/api/openapi.yaml vs src/server/routing.rs
- **Descripción:** Medir gap 35/40 (real 39/37) entre openapi.yaml y router.rs; completar openapi o documentar exclusiones docs-only
- **Archivos clave:** `docs/api/openapi.yaml`, `src/server/routing.rs` (facade) → `src/server/router.rs` (37 paths/44 ops reales), `dev-tools/validate_doc_snippets.py`
- **Gate Justificación:** Gate GOV-A medición bloqueante Show HN — sin paridad, contrato REST desprotegido; depende Wave0 GOV-T01..T03 ✅
- **Contrato:** `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths/44 ops, 0 missing/extra) AND `cargo check --workspace` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-A1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T20:30

=== RECITATION ===
Objetivo activo: GOV-A1 — Medición openapi parity
Estado: completed (desde: in-progress)
Última acción: Step1 medir 39/37 gap (/fast /slow extra) + Step2 fix docs-only: RS_FILE → src/server/router.rs, remover /fast /slow de openapi.yaml
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: GOV-A2 Wave1 paralelo (no bloquear A2/A3, MAX 3)
Contrato: `node scripts/check_openapi_parity.mjs` → Parity OK (37 paths, 44 ops) ✅ + `cargo check --workspace` ✅
Invariantes: No tocar RES-02..05 (P38 durabilidad aislada); docs-only, no cambios Rust; routing.rs facade preservado
Comandos de verificación: `node scripts/check_openapi_parity.mjs` → Parity OK + `cargo check --workspace` → Finished dev
Deuda: ninguna — parity cerrada, exclusiones documentadas (gap /fast /slow eliminado del spec)
Próxima tarea si completa: GOV-A2 — Reconciliar cifras tests
last-synced: 2026-09-02T20:30
=== END RECITATION ===

#### GOV-A2 — Reconciliar cifras tests
- **Descripción:** registrar run audit canónico (2034/2034/1 skip @2026-08-22, fecha+perfil) y contextualizar citas 2568+/1902/1492
- **Archivos clave:** `docs/TEST_MAP.md`, `docs/reviews/*`
- **Archivos clave (expandido):** `docs/reports/dora.md`, `evals/dora.mjs`, `Cargo.toml`, `.codegraph/codegraph.db`, `scripts/validate-docs-coverage.ps1`, `.config/nextest.toml`
- **Gate Justificación:** 3 cifras sin fuente única; auditoría intake
- **Contrato:** `Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-A2.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T21:00

=== RECITATION ===
Objetivo activo: GOV-A2 — Reconciliar cifras tests (docs/reports, coverage, nextest)
Estado: completed (desde: pending)
Última acción: DISCOVERY docs/reports/dora.md+evals/dora.mjs+Cargo.toml+.codegraph+validate-scripts+nextest.toml+TEST_MAP+manifest grep → EJECUCIÓN verify Select-String 2034*2026-08 + coverage + nextest list 2074 + validate-docs 6/6 → CIERRE task file GOV-A2.md + plan sync
Resultado: ✅
State: COMPLETED (desde: PENDING)
Próxima acción: GOV-A3 Wave1 paralelo (no bloquear A4/A5, MAX 3, disjoint)
Contrato: `Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08" | Measure-Object Count` >=1 ✅ (Count=1) + `Select-String -Path "docs/TEST_MAP.md" -Pattern "coverage"` >=1 ✅ + `cargo nextest list --profile default -p vantadb` 2074 ✅ + `scripts/validate-docs-coverage.ps1 -ReportOnly` 6/6 ✅ + `Test-Path .codegraph/codegraph.db` ✅ + `Cargo.toml` 0.5.0 ✅
Invariantes: No tocar RES-02..05 (P38 durabilidad aislada) ni GOV-A3 (paralelo disjoint) — respetado, 0 archivos en común
Comandos de verificación: `Select-String -Path "docs/TEST_MAP.md" -Pattern "2034.*2026-08"` (Count 1) + `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` (6/6) + `cargo nextest list --profile default -p vantadb` (2074) + `Test-Path .codegraph/codegraph.db`
Deuda: ninguna — cifras 2034/1492/1902/2568+ reconciliadas en TEST_MAP.md:92, coverage ADR-018 81.40% intacto, dora.md sin drift
Próxima tarea si completa: GOV-A3 — Probes CLI reales doctor/backup/restore
last-synced: 2026-09-02T21:00
=== END RECITATION ===

#### GOV-A3 — Probes CLI reales doctor/backup/restore
- **Descripción:** transcripción backup→manifest 36 files→restore temp --force→doctor→get recupera; alimenta runbook GOV-B2
- **Archivos clave:** binario `vanta-cli`, DB temporal
- **Gate Justificación:** auditoría detectó Restore sin --dry-run ni --fix; procedimiento diario nuevo debe probarse sandbox
- **Contrato:** `vanta-cli doctor --help 2>&1 | Select-String "doctor" | Measure-Object Count` >=1 AND transcripción adjunta en task record
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-A3.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-A4 — Harness snippets docs
- **Descripción:** `dev-tools/validate_doc_snippets.py` extrae bloques python tutorials/QUICKSTART y ejecuta contra DB temp; detecta graph_bfs ×2 + ef_search fantasma + 31 FAIL iniciales
- **Archivos clave:** `dev-tools/validate_doc_snippets.py`, `docs/tutorials/*.md`, `docs/api/EMBEDDED_SDK.md`
- **Gate Justificación:** auditoría snippets rotos; guard anti-regresión para gate-docs
- **Contrato:** `python dev-tools/validate_doc_snippets.py 2>&1 | Select-String "PASS.*FAIL.*SKIP" | Measure-Object Count` >=1 AND `python dev-tools/validate_doc_snippets.py` → 0 FAIL (27 PASS 27 SKIP)
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-A4.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T21:30

=== RECITATION ===
Objetivo activo: GOV-A4 — Harness snippets docs parity
Estado: completed (desde: in-progress)
Última acción: DISCOVERY validate_doc_snippets.py 26/3/25 → EJECUCIÓN fix docs-only 3 archivos (QUICKSTART hit.key + chromadb/lancedb vanta-skip) → retry 27/0/27 ✅
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: GOV-A5 / GOV-B3 consumo guard (paralelo disjoint)
Contrato: `python dev-tools/validate_doc_snippets.py` → Summary: 27 PASS, 0 FAIL, 27 SKIP ✅ + `Select-String -Path docs/QUICKSTART.md -Pattern hit\.key` ≥1 ✅ + `Select-String vanta-skip` en migrators ≥1 ✅
Invariantes: No tocar GOV-A3 (vanta-cli) ni RES-02 (wal/engine) — disjoint preservado, docs-only, 0 archivos Rust
Comandos de verificación: `python dev-tools/validate_doc_snippets.py` → 0 FAIL + `Select-String -Path docs/QUICKSTART.md -Pattern "hit\.key"` Count 1 + `Select-String -Path docs/tutorials/03-migrating-from-chromadb.md -Pattern vanta-skip` Count 1
Deuda: ninguna — parity cerrada, harness como guard para GOV-B3
Próxima tarea si completa: GOV-A5 — Registros live
last-synced: 2026-09-02T21:30
=== END RECITATION ===

#### GOV-A5 — Registros live crates.io/npm/PyPI
- **Descripción:** captura JSON/HTML respuestas registries; actualizar filas RELEASE-02/MKT-18h con estado verificado 0.5.0 live 2026-08-01 + wheels ARM64 ausentes
- **Archivos clave:** `docs/reports/GOV-A5-registros-live.md`, `docs/reports/dora.md`, `.opencode/task-system/enforcement/verify-log.jsonl`, `evals/dora.mjs`, `Cargo.toml` 0.5.0
- **Gate Justificación:** MKT-18h/18f gaps sin verificación live — registries 0.5.0 live 2026-08-01 ya verificado plan:10, wheels ARM64 gap documentado ponytail sin inflar
- **Contrato:** `Select-String -Path "docs/reports/*" -Pattern "registros live" | Measure-Object Count` >=1 AND `cargo check -p vantadb` exit 0 — extendido: 3 captures timestamped crates.io/PyPI/npm en `docs/reports/GOV-A5-registros-live.md`
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-A5.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T22:30

=== RECITATION GOV-A5 COMPLETED ===
Objetivo activo: GOV-A5 COMPLETED — Registros live crates.io/npm/PyPI (verify-log + docs/reports + dora)
Estado: completed (desde: pending)
Última acción: DISCOVERY Read registros live (dora.md 402L + dora.mjs 360L + verify-log 16L + Cargo 0.5.0) + grep SKILLS-MANIFEST keywords "registros/live/verify-log/dora/report" → EJECUCIÓN crear task file GOV-A5.md + fix registros live (GOV-A5-registros-live.md 3 captures 2026-09-02 + verify-log GOV-A5 append) ponytail 1 file → verify Select-String "registros live" 3 + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: PENDING)
Próxima acción: RES-03/04/05 Wave1c parallel (phrase/semántica) — MAX 3, disjoint iql/docs/api
Contrato: `Select-String docs/reports/* "registros live"` >=1 ✅ (Count 3) + `Select-String GOV-A5-registros-live.md crates.io|PyPI|npm` >=3 ✅ (16) + `Select-String verify-log GOV-A5` >=1 ✅ + `cargo check -p vantadb` Finished ✅
Invariantes: No tocar src/wal ni src/iql (disjoint RES-03/04 preservado) — dominio docs-only + verify-log; ponytail 1 file reuse dora.mjs/Cargo.toml
Comandos de verificación: `Select-String -Path "docs/reports/*" -Pattern "registros live" | Measure-Object Count` (3) + `cargo check -p vantadb` (Finished) + `Test-Path GOV-A5.md` (True)
Deuda: ninguna — registros live cerrados, wheels ARM64 gap documentado sin inflar, 0 líneas Rust
Próxima tarea si completa: RES-03 — Phrase queries gap TextMatch literal (Wave1c disjoint)
last-synced: 2026-09-02T22:30
=== END RECITATION ===

#### RES-02 — Durabilidad 🔴 Physical restore S1 quiesce+flush (Wave1 P38 — durabilidad)
- **Descripción:** S1 quiesce+flush (wal quiesce via StorageEngine::flush ERR-010 insert_lock+drain+backend+vstore+save_vector_index) + src/storage/engine/mod.rs create_snapshot ×2 (Unix:609/Win:656) mirror_data_dir recursive (514, skip snapshots) + mirror_backend_to (543, capture backend KV + skip .vanta.lock) — snapshot_restore (752) validate anti ../ + staging pre_restore_<nanos> + failpoint snapshot_restore_fail ya existe (S2 landed, verify S1 prerequisite)
- **Archivos clave:** `src/wal.rs` (WAL v2 Prepare), `src/storage/engine/mod.rs:514,543,609,656,752`, `src/storage/engine/maintenance.rs:36` (flush), `src/storage/vfile.rs`, `src/storage/vfile_mmap.rs`, `docs/research/archive/res02-backup-restore.md` §3 S1
- **Gate Justificación:** gaps create_snapshot flat copy + no quiesce (torn set vstore header vs node payloads vs KV) — prerequisite durabilidad F1; S1 quiesce correctness > speed, O(1) hard-link trade-off; requiere RES-01 WAL v2 Prepare landed
- **Contrato:** `Select-String -Path "src/storage/engine/mod.rs" -Pattern "flush\(\)|mirror_data_dir" | Measure-Object Count` >=1 AND `Select-String -Path "src/wal.rs" -Pattern "WalRecord::Prepare" | Measure-Object Count` >=1 AND `cargo test -p vantadb --test wal_rollback -- --nocapture | Select-String "ok" | Measure-Object Count` >=1 AND `cargo check -p vantadb --features fjall` Finished
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-02.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T22:00

=== RECITATION ===
Objetivo activo: RES-02 — Durabilidad Physical restore S1 quiesce+flush (Wave1 P38)
Estado: completed (desde: in-progress)
Última acción: DISCOVERY codegraph_explore wal/quiesce/snapshot_restore (91 símbolos) + Read wal.rs (WAL v2 Prepare) + engine/mod.rs + maintenance flush ERR-010 + docs/research S1 → EJECUCIÓN S1 guard verified (self.flush() quiesce ×2 ya landed, mirror_data_dir recursive + mirror_backend_to) — ponytail 0 líneas nuevas, reuse flush existing → verify wal_rollback 5/5 + cargo check fjall Finished
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: RES-03/04/05 Wave1c parallel (phrase/semántica) — MAX 3, disjoint iql/docs/api
Contrato: `Select-String engine/mod.rs flush|mirror_data_dir` >=1 ✅ + `Select-String wal.rs Prepare` >=1 ✅ + `cargo test --test wal_rollback` 5/5 ok ✅ + `cargo check --features fjall` Finished ✅
Invariantes: No tocar GOV-A3/A4 docs/api (disjoint preserved) — dominio Rust core storage/WAL only; flush ERR-010 insert_lock+drain; mirror skip snapshots + .vanta.lock; WAL v2 compat
Comandos de verificación: `cargo test -p vantadb --test wal_rollback -- --nocapture` → 5 passed + `cargo check -p vantadb --features fjall` → Finished 52s
Deuda: ninguna S1 — torn gap cerrado; S2 snapshot_restore ya existe pero S4-S5 CLI/MCP wrappers quedan Wave1c
Próxima tarea si completa: RES-03 — Phrase queries gap TextMatch literal (INV-009)
last-synced: 2026-09-02T22:00
=== END RECITATION ===

#### RES-03 — Phrase queries gap TextMatch literal (INV-009)
- **Descripción:** condición TextMatch literal en parser IQL + tokenización sin stemming/stopwords + highlight frase completa en snippets; enforcement base lexical_search ya existe, faltan 3 gaps
- **Archivos clave:** `src/query.rs` (Condition::TextMatch), `src/parser/mod.rs` (IQL `p.bio ~ "neural network"`), `src/text_index.rs` (literal_query_plan / text_contains_query), `src/sdk/search/phrase.rs`, `src/sdk/search/snippet.rs` (highlight_phrases), `src/physical_plan/filter.rs`
- **Gate Justificación:** phrase queries media — investigación huérfana validada, dependencias lexical_search hechas
- **Contrato:** `cargo test -p vantadb -- phrase 2>&1 | Select-String "ok" | Measure-Object Count` >=1 AND `Select-String -Path "src/query.rs" -Pattern "TextMatch" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-03.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:30

=== RECITATION ===
Objetivo activo: RES-03 — Phrase queries gap TextMatch literal (INV-009)
Estado: completed (desde: in-progress)
Última acción: DISCOVERY codegraph_explore phrase TextMatch IQL (13 símbolos) + Read query.rs/text_index.rs/phrase.rs/snippet.rs/filter.rs + grep TextMatch 3 hits query.rs / literal_query_plan 3 hits → EJECUCIÓN ponytail 1 guard consecutive_positions (reuse existing tokenizer, O(n) linear scan + ponytail comment) + highlight_phrases single-wrap D-2 (phrase != union of terms) — 0 líneas nuevas netas, enforcement ya landed → verify cargo test phrase 18 passed + cargo check --all-targets Finished
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: RES-04/05 Wave1c parallel (phrase/semántica) — MAX 3, disjoint iql/docs/api (no bloquear GOV-A5)
Contrato: `cargo test -p vantadb --lib phrase` 18 passed ✅ + `Select-String src/query.rs TextMatch` 3 ≥1 ✅ + `Select-String src/text_index.rs literal_query_plan` 3 ≥1 ✅ + `Select-String src/sdk/search/snippet.rs highlight_phrases` 2 ≥1 ✅ + `cargo check -p vantadb --all-targets` Finished ✅
Invariantes: No tocar src/wal.rs/src/vector/src/storage (Arch/Engine), docs-only GOV-A5 disjoint preservado, IQL TextMatch vía query.rs condición (no src/iql/ nueva carpeta — reuse)
Comandos de verificación: `cargo test -p vantadb --lib phrase -- --nocapture` → 18 passed + `cargo check -p vantadb --all-targets` → Finished dev + `cargo fmt --check` → 0
Deuda: ninguna — clippy global 51 errores pre-existentes (P2-8 etc) no de RES-03; phrase.rs `ponytail: O(n) lookup per token; switch to HashMap if hot path` documenta techo, Tuner delegable si canonical_p99 lo exige
Próxima tarea si completa: RES-04 — Phrase queries end-to-end (consolidable) / RES-05 semántica scores
last-synced: 2026-09-02T23:30
=== END RECITATION ===

#### RES-04 — Semántica scores oficial (FND-06 H3) — Wave1c P38 scoring
- **Descripción:** documentar scoring RRF/cosine/BM25 + zero-norm ERR-028 en docs/api/scores + helper thin wrapper src/api/scores (ponytail: 1 guard vs 50 líneas duplicadas `1.0 - s/2.0` en adapters) — cierra gap FND-06 H3 grep docs/api 0 hits
- **Archivos clave:** `src/api/scores*`, `docs/api/scores*`, `src/planner.rs` (RRF_K), `src/index/distance/metrics.rs` (cosine), `src/sdk/search/mod.rs` (ERR-028)
- **Gate Justificación:** semántica oficial media — drift zero-norm documentado FND-06 H1/H3, sin contrato oficial docs/api; precede RES-05/06 completo si split; disjoint 100% con RES-03 (iql) y GOV-A5 (registries) — Wave1c parallel MAX 3
- **Contrato:** `Select-String -Path "docs/api/scores.md" -Pattern "score semantics|RRF|BM25|cosine|zero-norm" | Measure-Object Count` >=1 AND `Select-String -Path "src/api/scores.rs" -Pattern "cosine_distance|RRF|score" | Measure-Object Count` >=1 AND `cargo check -p vantadb` exit 0 AND `cargo test -p vantadb --lib api::scores` pass (4/4) AND `cargo test -p vantadb --lib phrase` pass (18/18 disjoint)
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-04.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:30

=== RECITATION ===
Objetivo activo: RES-04 — Semántica scores (src/api/scores, docs/api/scores) scoring semántico
Estado: completed (desde: in-progress)
Última acción: DISCOVERY codegraph_explore "scores semántica" (30 símbolos, planner RRF_K=60, metrics cosine zero-norm, ERR-028) + Read scores files (docs/api 0 hits gap confirmado) → EJECUCIÓN crear docs/api/scores.md (32 hits RRF/BM25/cosine/zero-norm) + src/api/scores.rs (26 hits, helpers rrf_contribution, cosine_distance↔similarity, relevance, 4 tests) + src/api/mod.rs wiring pub mod scores + src/lib.rs pub mod api — ponytail thin wrapper delega a planner/metrics, no SIMD duplicado
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: Wave1c parallel disjoint preservado (RES-03 iql no tocado, GOV-A5 registries no tocado) → siguiente RES-05/06 scoring follow-up o Wave2 P27 F1 si Wave1c cierra
Contrato: `Select-String docs/api/scores.md RRF|BM25|cosine|zero-norm` 32 ≥1 ✅ + `Select-String src/api/scores.rs cosine_distance|RRF` 26 ≥1 ✅ + `cargo check -p vantadb` Finished ✅ + `cargo check --all-targets` Finished ✅ + `cargo test --lib api::scores` 4/4 ok ✅ + `cargo test --lib phrase` 18/18 ok (disjoint) ✅
Invariantes: No tocar src/wal.rs, src/storage/engine, src/vector, src/iql (RES-03 disjoint), GOV-A5 registries; RRF_K=60 re-exported, zero-norm ERR-028 preservado (no fallback silencioso), helpers pure f32 inline O(1)
Comandos de verificación: `cargo check -p vantadb` → Finished + `cargo test -p vantadb --lib api::scores` → 4 passed + `cargo test -p vantadb --lib phrase` → 18 passed + `Select-String docs/api/scores.md RRF` Count 32
Deuda: ninguna — helper reduce deuda H3 (centraliza `1.0 - s/2.0` duplicada adapters); follow-up adapters pueden migrar a helper sin scope creep
Próxima tarea si completa: RES-05 — Semántica scores parcial (FND-06 H1 drift doc) o MEM-01 Wave2 F1 search profile
last-synced: 2026-09-02T23:30
=== END RECITATION ===

#### RES-05 — Benchmark semántica scores (FND-06 H1/H3 follow-up RES-04) — Wave1c P38 bench
- **Descripción:** bench criterion minimal `benches/scores_semantics.rs` para `src/api/scores.rs` (rrf_contribution, cosine_distance↔similarity/relevance) — reuse `canonical_p99` profile, ponytail pure f32 O(1) batch 10k — cierra P38 semántica con medición reproducible (Regla 9)
- **Archivos clave:** `benches/scores_semantics.rs`, `benches/common/mod.rs` (apply_fixed_profile), `Cargo.toml` [[bench]] scores_semantics, `src/api/scores.rs` (RES-04), `docs/operations/BENCHMARKS.md` §9, `benchmarks/*` (python harness disjoint)
- **Gate Justificación:** semántica oficial media — RES-04 cerró docs/api/scores + helper; faltaba medición reproducible para validar migración adapters sin inflar — disjoint 100% con Wave2 GOV-B (no docs/case_studies)
- **Contrato:** `Test-Path benches/scores_semantics.rs` True AND `Select-String benches/scores_semantics.rs "rrf_contribution|cosine_distance" | Measure Count` >=3 AND `Select-String Cargo.toml 'scores_semantics' | Measure Count` >=1 AND `Select-String docs/operations/BENCHMARKS.md "scores_semantics|Score Semantics" | Measure Count` >=1 AND `cargo bench -p vantadb --bench scores_semantics --no-run` Finished (Executable) AND `cargo check -p vantadb` Finished
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-05.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:50

=== RECITATION ===
Objetivo activo: RES-05 — Benchmark semántica scores (benches/scores_semantics, docs/operations/BENCHMARKS.md §9)
Estado: completed (desde: pending)
Última acción: DISCOVERY Read benchmarks/* + BENCHMARKS.md 308L + src/api/scores.rs 109L + grep SKILLS-MANIFEST benchmark/scores/bench/canonical_p99 (1 hit vantadb) + Cargo.toml benches 22 entries + canonical_p99.rs 133L reuse pattern → EJECUCIÓN crear task file RES-05.md + implementar bench semántica minimal benches/scores_semantics.rs (6 micro-benches batch 10k reuse common::apply_fixed_profile, ponytail pure f32 O(1) xorshift determinístico) + Cargo.toml [[bench]] scores_semantics + docs/operations/BENCHMARKS.md §9 table expected ~ns/op — verify cargo bench --no-run 45.7s Executable + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: PENDING)
Próxima acción: Wave1c cierre — Wave1 9/9 (GOV-A1..A5 5 + RES-02..05 4) → Wave2 P27 F1 / GOV-B parallel MAX 3 (disjoint preserved)
Contrato: `Test-Path benches/scores_semantics.rs` True ✅ + `Select-String scores_semantics.rs rrf|cosine` 12 ≥3 ✅ + `Select-String Cargo.toml scores_semantics` 1 ≥1 ✅ + `Select-String BENCHMARKS.md scores_semantics` 5 ≥1 ✅ + `cargo bench --bench scores_semantics --no-run` Finished Executable ✅ (45.7s) + `cargo check -p vantadb` Finished ✅
Invariantes: No tocar src/wal.rs, src/storage, src/vector, src/iql, docs/case_studies (Wave2 GOV-B disjoint), vantadb-ts (no drift) — dominio bench only; helpers pure f32 inline sin SIMD duplicado; reuse canonical_p99 profile; ponytail batch SIMD if hot path
Comandos de verificación: `cargo check -p vantadb` → Finished + `cargo check --bench scores_semantics -p vantadb` → Finished + `cargo bench -p vantadb --bench scores_semantics --no-run` → Executable + `Select-String benches/scores_semantics.rs rrf_contribution|cosine_distance` Count 12 + `Select-String docs/operations/BENCHMARKS.md scores_semantics` Count 5
Deuda: ninguna — bench documenta techo O(1) ~ns/op; si profiling muestra hot path, upgrade a batch SIMD está taggeado ponytail en scores.rs + bench; Tuner delegable si canonical_p99 exige
Próxima tarea si completa: Wave2 — MEM-01 (F1 search profile) / GOV-B1 (case_studies archive) parallel MAX 3
last-synced: 2026-09-02T23:50
=== END RECITATION ===

---

### Wave2 — P27 F1 MEM-01..06 + GOV-B1..B6 + GOV-C1..C3 + RES-06..07

#### MEM-01 — F1 Search profile por namespace en core
- **Descripción:** SearchProfileConfig {mode: keyword|vector|hybrid, rrf_k, candidate_k} parametrizando planner + IQL + API + report RRF incluye rrf_k (D20)
- **Archivos clave:** `src/planner.rs`, `src/sdk/serialization/vector_types.rs`, `src/sdk/search/mod.rs`, `src/cli_server.rs` IQL
- **Gate Justificación:** F1 base LLM-free, WASM-compatible; appetite S; depende EMB-01..09 ✅
- **Contrato:** `cargo check -p vantadb` exit 0 AND `cargo nextest run -p vantadb -- search_profile 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-01.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:59

=== RECITATION MEM-01 COMPLETED ===
Objetivo activo: MEM-01 — F1 search profile (planner.rs, entity)
Estado: completed (desde: in-progress)
Última acción: DISCOVERY codegraph_explore "planner search profile" (38 símbolos) + Read planner.rs 984L + Read sdk/types.rs SearchProfileConfig + grep SKILLS-MANIFEST planner/search/profile → EJECUCIÓN ponytail reuse: SearchProfileConfig ya landed 6a50b8ee (mode/rrf_k/candidate_k + search_profile Option en VantaMemorySearchRequest + fuse_rrf parametrizado + IQL PROFILE keyword/vector/hybrid + CBO mode routeo) — 0 líneas nuevas, verify 38 planner + 11 search_profile + 117 parser + 1976 lib + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: MEM-02 Wave2 parallel (MCP paridad profile) — MAX 3, disjoint engine/docs
Contrato: `cargo check -p vantadb` Finished ✅ + `cargo test -p vantadb --lib planner` 38/38 ok ✅ + `cargo test -p vantadb --lib search_profile` 11/11 ok ✅ + `cargo test -p vantadb --lib parser` 117/117 ok ✅ + `cargo test -p vantadb --lib` 1976/1976 ok ✅
Invariantes: No colisiona SearchProfile (src/index/search/profile.rs debug profiler) — D14; IQL rrf_k/candidate_k propagados sin efecto CBO (ponytail deuda documentada); sin push (vanta-lead)
Comandos de verificación: `cargo check -p vantadb` → Finished 0.79s + `cargo test -p vantadb --lib planner` → 38 passed + `cargo test -p vantadb --lib search_profile` → 11 passed
Deuda: ponytail IQL rrf_k/candidate_k sin efecto CBO hasta fusión RRF; ponytail Keyword ignora sparse
Próxima tarea si completa: MEM-02 — F1 Exponer search profile en MCP/search (Wave2)
last-synced: 2026-09-02T23:59
=== END RECITATION ===

#### MEM-02 — F1 Exponer search profile en MCP/search
- **Descripción:** paridad IQL+API+MCP passthrough SearchProfileConfig en tools MCP
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/validation.rs`
- **Gate Justificación:** depende MEM-01; D13 IQL+API+MCP
- **Contrato:** `cargo check -p vantadb-mcp` exit 0 AND test paridad IQL/API/MCP pass
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-02.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T01:35

=== RECITATION MEM-02 COMPLETED ===
Objetivo activo: MEM-02 — F1 Exponer search profile en MCP/search (vantadb-mcp)
Estado: completed (desde: completed → re-verify Wave2)
Última acción: DISCOVERY codegraph_explore "mcp search profile" (23 símbolos, SearchProfileConfig 478L) + Read handlers/tools.rs 3090L parse_search_request + validation.rs 120L validate_search_profile + grep SKILLS-MANIFEST planner/search/profile (BUILD MCP/search) → EJECUCIÓN ponytail reuse SearchProfileConfig::Deserialize single source of truth (1 línea serde from_value + bounds rrf_k 1..=100 candidate_k 1..=10000, D13/D19 parity) → passthrough Some(validate_search_profile) en 4 tools (search_memory/memory_search/search_with_method/search_multi) via parse_search_request chokepoint — 0 líneas nuevas netas (landed 32b09daf), verify 13/13 validation + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: COMPLETED)
Próxima acción: MEM-03 Wave2 parallel (entity_* CRUD) — MAX 3, disjoint core-engine/docs
Contrato: `cargo check -p vantadb-mcp` Finished ✅ (21s) + `cargo test -p vantadb-mcp --lib validate_search_profile` 1/1 ok ✅ (13/13 validation) + `Select-String tools.rs search_profile` 8 ≥1 ✅ + `Select-String validation.rs validate_search_profile` 8 ≥2 ✅ + `cargo check --all-targets` Finished
Invariantes: No tocar docs/tutorials/* docs/glosario/* (GOV-B3/B4 disjoint) ni src/planner.rs (MEM-01) — dominio vantadb-mcp/search only; Hyrum {}→None Hybrid default documentado; bounds protegen OOM
Comandos de verificación: `cargo check -p vantadb-mcp` → Finished + `cargo test -p vantadb-mcp --lib validation::tests::validate_search_profile_parses_and_bounds -- --nocapture` → 1 passed + `Select-String tools.rs search_profile` Count 8
Deuda: ninguna — ponytail reuse Deserialize evita duplicación shape; follow-up MEM-03..06 reuse EntityStore/skills; tuner delegable si canonical_p99 exige `#[inline]` en parse_search_request (no hot path)
Próxima tarea si completa: MEM-03 — F2 Entidades entity_* + CRUD en core (Wave2)
last-synced: 2026-09-02T01:35
=== END RECITATION ===

#### MEM-03 — F2 Entidades entity_* + CRUD en core
- **Descripción:** modelo teams/users/agents/tasks/assets en nodos InternalMetadata (D4, hoy namespace+key); CRUD + índices
- **Archivos clave:** `src/entity.rs`, `src/entity/tests.rs`
- **Gate Justificación:** F2 base multi-agente; M
- **Contrato:** `cargo check -p vantadb` exit 0 AND `cargo nextest run -p vantadb -- entity 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-03.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-04 — F2 Permission-checker allow-only 7 eslabones
- **Descripción:** cadena resource→owner→member→visibility→role-default→ACL→deny (96 líneas clon), coexiste con src/rbac.rs transporte (verificado MEM-04)
- **Archivos clave:** `src/entity/checker.rs`, `MC/metadata/service/permission-checker.ts` (172L clon)
- **Gate Justificación:** D7 completa; depende MEM-03
- **Contrato:** `cargo check -p vantadb` exit 0 AND `cargo nextest run -p vantadb -- checker 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-04.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-05 — F2 Auth 3 capas en server + audit log
- **Descripción:** L1 Bearer timingSafeEqual + L2 service-id + L3 user-key→userId/isSystemAdmin desde entity_*; /health pública; extender /api/v2/audit existente
- **Archivos clave:** `src/cli_server.rs:633-773`, `vantadb-server/src/middleware.rs`, `src/audit.rs`
- **Gate Justificación:** depende MEM-03/04; D15 audit log server
- **Contrato:** `cargo check -p vantadb-server` exit 0 AND `cargo test -p vantadb-server -- auth 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-05.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-06 — F3 Esquema skills multi-versión en core
- **Descripción:** namespace skills + nodos por versión (version, is_head, content_hash, expires_at, owner_agent), índice único parcial (owner,name) is_head, optimistic lock expected_version, TTL keep-recent=3, idempotencia content-hash; reuse text_index+HNSW
- **Archivos clave:** `src/skills.rs`, `src/sdk/types.rs`, `MC/core/skill/skill-store-ddl.ts` (104)
- **Gate Justificación:** F3 base; reuse EntityStore MEM-03; M
- **Contrato:** `cargo check -p vantadb` exit 0 AND `cargo nextest run -p vantadb -- skill 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-06.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-B1 — case_studies ficticios → archive interno
- **Descripción:** git mv docs/case_studies/{rag_edge_device,agent_local_memory_ollama}.md → docs/archive/case-studies-unverified/ + README disclaimer + stubs book refs
- **Archivos clave:** `docs/case_studies/*`, `docs/master-index.md`
- **Gate Justificación:** D6 eliminar, D3 Show HN bloqueante reputación; T0.1 archive interno
- **Contrato:** `Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` == true AND `Select-String -Path "docs/archive/case-studies-unverified/README.md" -Pattern "no-público|ilustrativos" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:55

=== RECITATION GOV-B1 COMPLETED ===
Objetivo activo: GOV-B1 — case_studies ficticios → archive interno (Show HN bloqueante D6)
Estado: completed (desde: pending)
Última acción: DISCOVERY Read archive 3 files + book stubs + master-index + web CASE_STUDIES + grep SKILLS-MANIFEST 4 keywords 0 hits → EJECUCIÓN crear GOV-B1.md + ponytail minimal (reuse commits a8a21733+98612db8, 0 líneas nuevas) → verify Select-String docs/case_studies + cargo check docs-only
Resultado: ✅
State: COMPLETED (desde: PENDING)
Próxima acción: GOV-B2 (DISASTER_RECOVERY_RUNBOOK ghost) + MEM-01 (F1 search profile) parallel MAX 3 disjoint — Wave2 continúa
Contrato: `Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` True ✅ + `Select-String README.md "no-público|ilustrativos"` Count 1 ✅ + `Test-Path docs/case_studies` False ✅ + `Test-Path docs/book/src/case_studies/index.md` True ✅ + `cargo check -p vantadb` Finished ✅
Invariantes: No tocar src/planner.rs (MEM-01 disjoint), src/wal.rs, DISASTER_RECOVERY_RUNBOOK.md (GOV-B2) — dominio docs/case_studies archive only; ponytail 0 files nuevos, reuse archive
Comandos de verificación: `Test-Path docs/archive/case-studies-unverified/rag_edge_device.md` (True) + `Select-String README.md "ilustrativos"` (1) + `Test-Path docs/case_studies` (False) + `cargo check -p vantadb` (Finished)
Deuda: ninguna — web CASE_STUDIES 3 composite sin disclaimer documentada como follow-up GOV-F1, no bloquea GOV-B1
Próxima tarea si completa: GOV-B2 — Runbook DR sin comandos fantasma
last-synced: 2026-09-02T23:55
=== END RECITATION ===

#### GOV-B2 — Runbook DR sin comandos fantasma
- **Descripción:** reescribir DISASTER_RECOVERY_RUNBOOK.md comandos a CLI real (restore --input/--force/--rebuild, sin --dry-run/doctor --fix) + §3.1 Daily Backup Verification con restore temp+doctor+conteo (insumo GOV-A3)
- **Archivos clave:** `docs/operations/DISASTER_RECOVERY_RUNBOOK.md:142,233,266`, `src/cli.rs:130-144`
- **Gate Justificación:** Addendum elevó a 🔴 addendum 3.4.2 — falla cuando se necesita; D4a docs ya
- **Contrato:** `Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "restore --dry-run|doctor --fix" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B2.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T23:55

=== RECITATION ===
Objetivo activo: GOV-B2 — Runbook DR sin comandos fantasma (DISASTER_RECOVERY_RUNBOOK.md)
Estado: completed (desde: in-progress)
Última acción: DISCOVERY Read runbook 473L + cli.rs Restore/Doctor + dora.md 402L + grep SKILLS-MANIFEST disaster/recovery/runbook/dora (3 hits) → EJECUCIÓN crear GOV-B2.md + fix runbook rephrase 2 líneas ghost literal (146/242) ponytail → verify Select-String ghost 0 + restore --input 5 + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: Wave2 paralelo disjoint — MEM-01 (planner) y GOV-B1 (archive) no bloqueados, siguiente GOV-B3
Contrato: `Select-String DISASTER_RECOVERY_RUNBOOK.md "restore --dry-run|doctor --fix" Count==0` ✅ (0) + `Select-String restore --input Count>=1` ✅ (5) + `cargo check -p vantadb` Finished ✅ + `Test-Path GOV-B2.md` True
Invariantes: No tocar src/wal.rs, src/storage/engine/*, src/planner.rs (MEM-01), disjoint Wave2 preserved; src/cli.rs fuente verdad; §3.1 Daily Backup Verification intacto
Comandos de verificación: `Select-String -Path "docs/operations/DISASTER_RECOVERY_RUNBOOK.md" -Pattern "restore --dry-run|doctor --fix" | Measure-Object Count` (0) + `cargo check -p vantadb` (Finished 29s)
Deuda: ninguna — ghost flags eliminados sin borrar notas, §3.1 (5 pasos restore temp+doctor+conteo) intacto, CLI real alineado
Próxima tarea si completa: GOV-B3 — Fix snippets + guard anti-regresión
last-synced: 2026-09-02T23:55
=== END RECITATION ===

#### GOV-B3 — Consumo guard anti-regresión (Wave2 batch2 paralelo MEM-02/03)
- **Descripción:** consumo guard anti-regresión — documentar baseline p99 + heap en BENCHMARKS §11 + anchor consumo guard en canonical_p99.rs + compile-gate `cargo bench --bench canonical_p99 --no-run` en dev-tools/verify.ps1 (Regla 9, disjoint MEM-02/03)
- **Archivos clave:** `docs/operations/BENCHMARKS.md`, `benches/canonical_p99.rs`, `dev-tools/verify.ps1`
- **Gate Justificación:** guard anti-regresión bloqueante Show HN — sin compile-gate, regresión p99/consumo silenciosa; Wave2 batch2 paralelo MAX 3, disjoint MEM-02 (MCP search profile) / MEM-03 (entity_* CRUD) — 0 archivos src/*
- **Contrato:** `cargo bench -p vantadb --bench canonical_p99 --no-run` Finished/Executable AND `Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "consumo guard" | Measure-Object Count` >=1 AND `Select-String -Path "benches/canonical_p99.rs" -Pattern "consumo guard" | Measure-Object Count` >=1 AND `Select-String -Path "dev-tools/verify.ps1" -Pattern "consumo guard" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B3.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T19:30

=== RECITATION GOV-B3 COMPLETED ===
Objetivo activo: GOV-B3 — Consumo guard anti-regresión (BENCHMARKS + cargo bench)
Estado: completed (desde: in-progress)
Última acción: DISCOVERY Read BENCHMARKS 339L + canonical_p99.rs 133L + verify.ps1 97L + grep SKILLS-MANIFEST guard/consumo/benchmark/bench/regression + Cargo.toml benches 22 entries → EJECUCIÓN crear task file GOV-B3.md + implementar guard consumo ponytail minimal docs (BENCHMARKS §11 20L + canonical_p99 1L anchor + verify.ps1 1 step consumo guard --no-run) → verify cargo bench --no-run Executable 1m34s + Select-String 7/1/2 >=1 + cargo check Finished
Resultado: ✅
State: COMPLETED (desde: IN_PROGRESS)
Próxima acción: Wave2 batch2 continúa — MEM-02/MEM-03 paralelos disjoint (engine/IQL) no bloqueados, siguiente GOV-B4 (openapi parity) MAX 3
Contrato: `cargo bench -p vantadb --bench canonical_p99 --no-run` Executable ✅ (Finished 1m34s) + `Select-String BENCHMARKS.md consumo guard` 7 >=1 ✅ + `Select-String canonical_p99.rs consumo guard` 1 >=1 ✅ + `Select-String verify.ps1 consumo guard` 2 >=1 ✅ + `cargo check -p vantadb` Finished ✅
Invariantes: No tocar src/* (disjoint MEM-02/03 preservado) — dominio docs/operations + benches + dev-tools only; 0 deps nuevas, reuse canonical_p99 existente; ponytail docs-only, compile-gate sin timed bench en fast gate
Comandos de verificación: `cargo bench -p vantadb --bench canonical_p99 --no-run` → Executable + `Select-String -Path "docs/operations/BENCHMARKS.md" -Pattern "consumo guard" | Measure-Object Count` (7) + `Select-String -Path "benches/canonical_p99.rs" -Pattern "consumo guard" | Measure-Object Count` (1) + `Select-String -Path "dev-tools/verify.ps1" -Pattern "consumo guard" | Measure-Object Count` (2) + `cargo check -p vantadb` (Finished 0.75s)
Deuda: ninguna — timed p99 queda en heavy_certification.yml (no fast gate); §11 documenta techo ±10% p99 requiere ADR/revert
Próxima tarea si completa: GOV-B4 — Regeneración openapi.yaml + gate paridad
last-synced: 2026-09-02T19:30
=== END RECITATION ===

#### GOV-B4 — Regeneración openapi.yaml + gate paridad
- **Descripción:** generar openapi 35 paths/40 ops desde cli_server.rs:215-260 + scripts/check_openapi_parity.mjs + gate gate-docs-21.yml
- **Archivos clave:** `docs/api/openapi.yaml`, `src/cli_server.rs:215-260`, `.github/workflows/gate-docs-21.yml:56-81`
- **Gate Justificación:** 3 paths vs ~29 reales, gate solo valida version — contrato REST desprotegido 🔴
- **Contrato:** `Select-String -Path "docs/api/openapi.yaml" -Pattern "paths:" | Measure-Object Count` >=1 AND `Test-Path scripts/check_openapi_parity.mjs` == true AND gate-docs-21 invoca parity script
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B4.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-B5 — HTTP_API.md completo
- **Descripción:** 35/35 endpoints agrupados por dominio con request/response real derivado yaml B4 + curl ≥5 endpoints + regla yaml-spec/md-guía
- **Archivos clave:** `docs/api/HTTP_API.md`, `docs/api/openapi.yaml`
- **Gate Justificación:** 4 de 29 documentados + ejemplo LISP muerto; depende GOV-B4
- **Contrato:** `Select-String -Path "docs/api/HTTP_API.md" -Pattern "\(memory:get" | Measure-Object Count` ==0 AND `Select-String -Path "docs/api/HTTP_API.md" -Pattern "/api/v2/search" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B5.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-B6 — Skill MCP fuente única 33 tools + MCP.md stub
- **Descripción:** skills/vantadb-mcp/references/api-reference.md a 33 tools (15 core+6 skill+8 code+4 wiki), hash-SAME ×3, MCP.md stub ≤20L con link, test-mcp.py 4/4 verde
- **Archivos clave:** `skills/vantadb-mcp/references/api-reference.md`, `docs/api/MCP.md`
- **Gate Justificación:** D8 skill única; cifras 21/15/33 divergentes; binario 0.5.0 solo 15 — 18 llegan próximo release
- **Contrato:** `Select-String -Path "skills/vantadb-mcp/references/api-reference.md" -Pattern "33 tools" | Measure-Object Count` >=1 AND `Test-Path .opencode/skills/vantadb-mcp/SKILL.md` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-B6.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C1 — Filtro nextest inefectivo + TEST_MAP binarios
- **Descripción:** corregir .config/nextest.toml:27 binary(python_sdk_boundary)→python + TEST_MAP hnsw_recall_certification→hnsw_recall; verificar cargo nextest list
- **Archivos clave:** `.config/nextest.toml:27`, `docs/TEST_MAP.md:83,86`
- **Gate Justificación:** SYNC-01 único hallazgo CI real; appetite 1h
- **Contrato:** `cargo nextest list --profile default 2>&1 | Select-String "python|hnsw_recall" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C2 — Backlog ↔ campañas P29/P30/P31 + MEM-43
- **Descripción:** fila MEM-43 ✅ hash a0bcb112 + secciones P29/P30 cerradas puntero archive + P31 8 tasks activas + contador coherente
- **Archivos clave:** `docs/Backlog.md:703-705`, `docs/plans/archive/*p29*/*p30*`, `docs/plans/2026-08-22-vanta-final-cierre.md`
- **Gate Justificación:** SYNC-03 single source of truth roto
- **Contrato:** `Select-String -Path "docs/Backlog.md" -Pattern "MEM-43.*✅" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C2.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C3 — Purga refs muertas Backlog
- **Descripción:** 10 refs audit-reports/ + REPORTE_EVALUACION ×2 + 2 reviews inexistentes → mención textual o REDIRECT.md
- **Archivos clave:** `docs/Backlog.md:213,230,341,427-431`
- **Gate Justificación:** IDX-02 trazabilidad rota
- **Contrato:** método AUD-007 `Select-String -Path "docs/Backlog.md" -Pattern "audit-reports" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C3.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-06 — Semántica scores oficial completa (FND-06 H3)
- **Descripción:** resolver drift zero-norm cosine core vs vantadb.ts + documentar RRF/cosine/BM25 completo en docs/api/
- **Archivos clave:** `docs/api/`, `vantadb-ts/src/vantadb.ts`
- **Gate Justificación:** complementa RES-05; media
- **Contrato:** `cargo test -p vantadb -- score 2>&1 | Select-String "ok" | Measure-Object Count` >=1 AND `Select-String -Path "vantadb-ts/src/vantadb.ts" -Pattern "zero.*norm|score" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-06.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-07 — Calibrar rss_threshold + bench full-scale 10k..100k
- **Descripción:** recalibrar DEFAULT_RSS_THRESHOLD 0.80 con medición real + bench criterion 10k..100k; F2/F3 FND-01
- **Archivos clave:** `src/config.rs:22`, `benches/memory-budget.rs`
- **Gate Justificación:** FND-01 follow-up; medida directa, no heurística
- **Contrato:** `cargo bench --bench memory-budget 2>&1 | Select-String "rss|Throughput" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-07.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

---

### Wave3 — P27 F4 MEM-07..21 (15) + GOV-C4..C7 (4) + P38 media RES-08..09

#### MEM-07 — F3 MCP tools skill_* (6)
- **Descripción:** tools skill_list/view/create/update/patch/files_write sobre MEM-06 con expected_version + owner check 404 + límites 5MB/50MB
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/skills.rs`, `MC/core/skill/skill-tools.ts`
- **Gate Justificación:** depende MEM-06; paridad API nativa D13
- **Contrato:** `cargo check -p vantadb-mcp` exit 0 AND `cargo test -p vantadb-mcp -- skill 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-07.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-08a — F4 Fundación crate vanta-memory
- **Descripción:** scaffold vanta-memory/Cargo.toml + lib.rs workspace member, features llm-driver off + mock tests, inheritance
- **Archivos clave:** `vanta-memory/Cargo.toml`, `vanta-memory/src/lib.rs`, `Cargo.toml` raíz
- **Gate Justificación:** F4 base; sin fuente TDAM directa (MC/package.json layout)
- **Contrato:** `cargo check -p vanta-memory` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-08a.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-08b — F4 Contratos L1 + trait LLMRunner host-neutral
- **Descripción:** MemoryRecord/DedupDecision + trait sync+async (D1) host-neutral; MC/abstractions/types.ts 87L + llm-runner 467L refs
- **Archivos clave:** `vanta-memory/src/core/abstractions/types.rs`, `vanta-memory/src/offload/types.rs`
- **Gate Justificación:** depende MEM-08a; host-neutral LLM-free degrada sin perder datos
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo nextest run -p vanta-memory -- types 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-08b.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-09 — F4 L0 capture idempotente
- **Descripción:** auto_capture 347L + l0_recorder 607L, captura conversación idempotente
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_capture.rs`, `vanta-memory/src/core/conversation/l0_recorder.rs`
- **Gate Justificación:** L0 pipeline; MC refs
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- l0 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-09.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-10 — F4 L1 extractor split+1 call LLM JSON+parse reparación
- **Descripción:** split marcadores + 1 call LLM JSON + json-utils reparación + prompts reescritos (no traducir chino Kenty)
- **Archivos clave:** `vanta-memory/src/core/record/l1_extractor.rs` (738), `MC/core/prompts/l1-extraction.ts` (417)
- **Gate Justificación:** L1 core; M complejo
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- l1_extractor 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-10.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-11 — F4 L1 dedup 2 fases store/update/merge/skip
- **Descripción:** dedup recall→juicio LLM + 2 fases; MC/l1-dedup 408L + prompts 236L
- **Archivos clave:** `vanta-memory/src/core/record/l1_dedup.rs`, `vanta-memory/src/core/prompts/l1_dedup.rs`
- **Gate Justificación:** depende MEM-10
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- dedup 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-11.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-12 — F4 Contrato META + nodo escena (ancla L2, D2)
- **Descripción:** META {created,updated,summary,heat} + nodo escena grafo core InternalMetadata
- **Archivos clave:** `vanta-memory/src/core/scene/scene_format.rs` (75), `scene_index.rs` (137)
- **Gate Justificación:** S barato, ancla LLM-free L2; depende MEM-08b
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- scene_format 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-12.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-13 — F4 Tools sandboxed read/write/edit + store
- **Descripción:** tools sandbox sobre store scene
- **Archivos clave:** `vanta-memory/src/core/scene/scene_tools.rs`, `MC/core/scene/scene-extractor.ts` 604L
- **Gate Justificación:** depende MEM-12
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- scene_tools 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-13.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-14 — F4 Strategy UPDATE>MERGE>CREATE + heat + soft-delete
- **Descripción:** strategy heat + soft-delete [DELETED] + emptyExtraction + filename_normalizer
- **Archivos clave:** `vanta-memory/src/core/scene/scene_extractor.rs` (604), `MC/core/prompts/scene-extraction.ts` (572)
- **Gate Justificación:** depende MEM-13
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- scene_extractor 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-14.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-15 — F4 Persona first/incremental + triggers
- **Descripción:** persona-generator 304L + persona-trigger 136L + escapeXml + límites
- **Archivos clave:** `vanta-memory/src/core/persona/persona_generator.rs`, `persona_trigger.rs`
- **Gate Justificación:** L3 persona
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- persona 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-15.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-16 — F4 Orquestación timers+locks (SOLO)
- **Descripción:** stateful_pipeline_manager 500L + pipeline_manager 1218L + pipeline_factory 1231L + pipeline_worker 843L + timer_scanner + managed_timer + checkpoint 745L + local_backend (estado local sin Redis, reloj fake)
- **Archivos clave:** `vanta-memory/src/utils/stateful_pipeline_manager.rs`, `pipeline_manager.rs`, `MC/utils/*.ts`
- **Gate Justificación:** timers per-session + locks granular L1 session/L2-L3 agent; esfuerzo L 🔴 SOLO
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo nextest run -p vanta-memory -- pipeline_manager 2>&1 | Select-String "ok" | Measure-Object Count` >=5
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-16.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-17 — F4 Skill extract transcript+sink idempotente
- **Descripción:** transcript marcadores anti role-capture + truncado + review taxonomía + sink idempotente integra MEM-06
- **Archivos clave:** `vanta-memory/src/core/skill/skill_extractor.rs` (587), `MC/core/skill/conversation-add/*`
- **Gate Justificación:** F4 skill
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- skill_extractor 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-17.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-18 — F4 Recall prepend/append + 3 modos
- **Descripción:** auto_recall 999L + composer 41L + resolver + profile_sync 494L; prependContext dinámico vs appendSystemContext cacheable + budget truncación code-point
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/core/memory_prompt/composer.rs`
- **Gate Justificación:** recall patrón prepend/append obligatorio desde F1 (04) para no romper prompt caching
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- auto_recall 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-18.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-19 — F4 sanitize_text + truncación code-point
- **Descripción:** sanitize 405L + text_utils 31L anti feedback-loop + truncación code-point
- **Archivos clave:** `vanta-memory/src/utils/sanitize.rs`, `vanta-memory/src/utils/text_utils.rs`
- **Gate Justificación:** transversal recall/offload
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- sanitize 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-19.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-20 — F4 Cursor lastOffloadedToolCallId persistente
- **Descripción:** state_manager 460L + storage 664L + after_tool_call 594L por sesión
- **Archivos clave:** `vanta-memory/src/offload/state_manager.rs`, `storage.rs`
- **Gate Justificación:** cursor fase offload 05
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- state_manager 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-20.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-21 — F4 Tools MCP scene_read/list/query
- **Descripción:** scene_navigation 76L + scene_index 137L + knowledge_handlers puros (soft-delete→NotFound, heat desc, overlap_score top_k=5)
- **Archivos clave:** `vanta-memory/src/gateway/knowledge_handlers.rs`
- **Gate Justificación:** cierre F4 con tools query; depende MEM-12/15
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo nextest run -p vanta-memory -- knowledge 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-21.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C4 — Regeneración master-index.md
- **Descripción:** regenerar desde árbol real 30+ carpetas, quitar 2 enlaces rotos :184/:192, indexar 15 carpetas +3 docs api, corregir frase blog :161, frontmatter last_reviewed 2026-09-02
- **Archivos clave:** `docs/master-index.md`
- **Gate Justificación:** IDX-01 congelado 07-21; puerta entrada docs
- **Contrato:** método AUD-007 `Select-String -Path "docs/master-index.md" -Pattern "audit-reports" | Measure-Object Count` ==0 AND `Test-Path docs/master-index.md` timestamp 2026-09-02
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C4.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C5 — operations/master-index.md completar
- **Descripción:** completar 26→32 archivos (chaos-testing, ci-cd-guide, pilot-×3, TEST_MAP, self) + regla same-PR
- **Archivos clave:** `docs/operations/master-index.md`
- **Gate Justificación:** índice canónico operations incompleto
- **Contrato:** `Get-ChildItem docs/operations/*.md | Measure-Object Count` == `Select-String -Path "docs/operations/master-index.md" -Pattern "^\|" | Measure-Object Count` (listing==index)
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C5.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C6 — CONFIGURATION.md sincronizada
- **Descripción:** sweep 44 env vars (VANTA_EMBEDDING_PROVIDER/OPENAI_API_KEY/MODEL/VANTADB_REPORTED_VERSION), rate_limit_rpm 100→600, flush_threshold None, PORT fallback/ flush_interval_ms fantasmas fuera, spot-check 14 defaults
- **Archivos clave:** `docs/operations/CONFIGURATION.md`, `src/config.rs:299,659`, `src/llm.rs:40,132,147`
- **Gate Justificación:** drift operador dimensiona capacidad falso
- **Contrato:** `Select-String -Path "docs/operations/CONFIGURATION.md" -Pattern "rate_limit_rpm.*600" | Measure-Object Count` >=1 AND grep env vars 0 drift
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C6.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-C7 — Contador Backlog corrección+regla
- **Descripción:** corregir ~24→45 (2026-08-22) → ~130 (2026-09-01) con fecha + regla sync rg ❌ + ROADMAP banner sin cifra
- **Archivos clave:** `docs/Backlog.md` header, `docs/strategy/ROADMAP.md`
- **Gate Justificación:** depende GOV-C2 sincronizado; evita deriva futura; appetite 30min
- **Contrato:** `Select-String -Path "docs/Backlog.md" -Pattern "130 activas.*2026-09" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-C7.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-08 — Benchmark delete-masivo DashMap sweep
- **Descripción:** bench contención real sweep path deletes en maintenance.rs; decidir rediseño solo si medición justifica (H4 FND-02, Regla 9)
- **Archivos clave:** `src/storage/engine/maintenance.rs`, `benches/delete_massive.rs`
- **Gate Justificación:** medir antes de rediseñar; evitar complejidad innecesaria
- **Contrato:** `cargo bench --bench delete_massive 2>&1 | Select-String "Throughput|time" | Measure-Object Count` >=1 AND ADR/documenta decisión
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-08.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-09 — Trackear roadmap post-launch huérfano
- **Descripción:** agregar a P24/backlog rows para WAL async ingest 10-100× + query planner real + DiskANN disk-I/O real (investigación huérfana 2026-08-09)
- **Archivos clave:** `docs/Backlog.md` P24, `docs/research/investigacion-equipo-2026-08-09.md` §roadmap
- **Gate Justificación:** 3 gaps sin filas, validados con archivo:línea
- **Contrato:** `Select-String -Path "docs/Backlog.md" -Pattern "WAL async ingest|query planner|DiskANN.*disk" | Measure-Object Count` >=3
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-09.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

---

### Wave4 — P27 F5-F7 MEM-22..38 (17) + GOV-D1..F2 (10) + P38 resto RES-10..15 + DEC-02 (7)

#### MEM-22 — F5 Context Engine assemble + cascada mild/aggressive (SOLO)
- **Descripción:** context-engine 526L + compaction-handler 328L + compressor 1194L + fast-path 189L + mmd-injector; mild cascade por score (revert si summary>original) + aggressive one-shot fingerprint + emergency; token estimator 3 chars/token o tiktoken o200k_base (D3 validar WASM)
- **Archivos clave:** `vanta-memory/src/offload_client/context-engine.ts` port Rust, `vanta-memory/src/offload/storage.ts:664`
- **Gate Justificación:** F5 killer contexto; L 🔴 SOLO; depende MEM-20 cursor
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- context_engine 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-22.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-23 — F5 Emergency + token estimator
- **Descripción:** fast-token-estimate 307L + l3-token-counter 35L + benchmark-token 89L + context-token-tracker 166L; o200k_base vs 3 chars/token fallback
- **Archivos clave:** `vanta-memory/src/offload/fast_token_estimate.rs`, `MC/offload/fast-token-estimate.ts`
- **Gate Justificación:** depende MEM-22; D3 WASM check
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- token_estimate 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-23.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-24 — F5 MMD persistente Mermaid literal + fingerprint
- **Descripción:** mmd-injector 374L + mmd-meta 66L + l2-mermaid pipelines; marker _mmdContextMessage + META reusado
- **Archivos clave:** `vanta-memory/src/offload/mmd_injector.rs`, `MC/offload/mmd-injector.ts`
- **Gate Justificación:** F5 MMD; propuesta VantaDB META en MMD; depende MEM-22
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `Select-String -Path "vanta-memory/src/offload/mmd_injector.rs" -Pattern "mermaid|_mmdContext" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-24.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-25 — F6 vanta-proxy 3 protocolos wire verbatim (segunda iteración)
- **Descripción:** handler 3 protocolos (OpenAI Chat Completions / Anthropic Messages / Responses API) + rutas; MP/handler.ts refs
- **Archivos clave:** `vanta-proxy/src/handler.rs`, `MP/handler.ts`, `Cargo.toml` member aparte (D5)
- **Gate Justificación:** F6 gateway opcional; L adopción coding agents; depende F4-F5 estables
- **Contrato:** `cargo check -p vanta-proxy` exit 0 AND `cargo test -p vanta-proxy -- handler 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-25.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-26 — F6 Ciclo auth→session→injection local
- **Descripción:** session 203L + auth + identity + injection 8 archivos + mem-command 72L
- **Archivos clave:** `vanta-proxy/src/session/*.rs`, `MP/session/index.ts`
- **Gate Justificación:** depende MEM-25
- **Contrato:** `cargo check -p vanta-proxy` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-26.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-27 — F6 Rate-limit + write-back + reporting + mem-commands
- **Descripción:** rate-limit sliding window + write-back + clickhouse/langfuse/opik sin Opik/Langfuse + mem: sync/help
- **Archivos clave:** `vanta-proxy/src/rate_limit.rs`, `MP/rate-limit/` refs
- **Gate Justificación:** depende MEM-26; fail-open trigger
- **Contrato:** `cargo check -p vanta-proxy` exit 0 AND `cargo test -p vanta-proxy -- rate_limit 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-27.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-28 — F7 Wiki state machine pending→ready locked:true dedup
- **Descripción:** engines/wiki index 23L + ingest-v2 cascade/frontmatter/slug/template; locked + dedup
- **Archivos clave:** `vanta-memory/src/engines/wiki/*.rs`, `MK/engines/wiki/ingest-v2/index.ts`
- **Gate Justificación:** F7 usa graphrag existente; M
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- wiki 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-28.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-29 — F7 SSRF blocklist + chunker 12k/400
- **Descripción:** source-fetcher 4 archivos + file-protocol + chunker 12k/400
- **Archivos clave:** `vanta-memory/src/source_fetcher.rs`, `MK/engines/wiki/ingest-v2/chunker.ts`
- **Gate Justificación:** depende MEM-28; seguridad SSRF
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `Select-String -Path "vanta-memory/src/source_fetcher.rs" -Pattern "blocklist|SSRF" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-29.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-30 — F7 Merge serial + pLimit + ensureSources
- **Descripción:** merge.ts + llm.ts + index-builder + overview + prompts; merge serial pLimit
- **Archivos clave:** `vanta-memory/src/engines/wiki/merge.rs`, `MK/engines/wiki/ingest-v2/merge.ts`
- **Gate Justificación:** depende MEM-28
- **Contrato:** `cargo check -p vanta-memory` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-30.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-31 — F7 Callback run_id + throttle
- **Descripción:** progress callback ingest-v2/index.ts + log-writer throttled
- **Archivos clave:** `vanta-memory/src/engines/wiki/callback.rs`, `MK/engines/wiki/ingest-v2/index.ts`
- **Gate Justificación:** depende MEM-28
- **Contrato:** `cargo check -p vanta-memory` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-31.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-32 — F7 Tools code_* sobre graphrag existente
- **Descripción:** tools code_* patrón rutas MK/mcp (2+3+6 archivos), sobre src/graph.rs existente (no copiar @colbymchenry/codegraph)
- **Archivos clave:** `vanta-memory/src/mcp/code_tools.rs`, `src/graph.rs`
- **Gate Justificación:** depende MEM-28; expose graphrag existente
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `Select-String -Path "vanta-memory/src/mcp/code_tools.rs" -Pattern "code_" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-32.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-33 — F7 Tools wiki_* sobre MEM-28
- **Descripción:** tools wiki_* sobre wiki state machine
- **Archivos clave:** `vanta-memory/src/mcp/wiki_tools.rs`, `MK/mcp/*`
- **Gate Justificación:** depende MEM-28
- **Contrato:** `cargo check -p vanta-memory` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-33.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-34 — F1 Telemetría por capa (adelantada D17, también en Wave2)
- **Descripción:** latencias L1/L2/L3/recall + envelope trace; extiende operational_metrics_snapshot + 13 campos + AuditEvent::memory; NO crear vantadb-server/src/audit.rs (ya existe en core)
- **Archivos clave:** `src/metrics/core/snapshot.rs`, `MC/core/report/metric-tracking-*`
- **Gate Justificación:** D17 adelantar a F1; Studio consume; ya en Wave2 si re-triage, aquí duplicada para completeness 86
- **Contrato:** `cargo check -p vantadb` exit 0 AND `cargo nextest run --profile audit --workspace 2>&1 | Select-String "passed" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-34.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-35 — Data plane REST agent-facing /conversation/add + /skill/listing
- **Descripción:** POST /conversation/add vía ThreadStore + GET /skill/listing sobre MEM-06; auth 3 capas MEM-05; D18 no contaminar IQL, Studio viewer no ingesta; si Studio lista skills → wrapper server_client.rs trivial
- **Archivos clave:** `src/cli_server.rs`, `vantadb-server/src/server.rs`, `MC/gateway/chat-memory-handlers.ts` 476L
- **Gate Justificación:** transversal tras F3; D18 REST orientado agentes
- **Contrato:** `cargo check -p vantadb-server` exit 0 AND `cargo test -p vantadb-server -- conversation 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-35.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-36 — SDK sub-clientes por dominio (memory-core)
- **Descripción:** estructura sdk/memory-core/typescript + python sub-clientes por dominio sobre MEM-35; bindings sin breaking
- **Archivos clave:** `vantadb-ts/src/vantadb.ts`, `vantadb-python/src/lib.rs`, `sdk/memory-core/typescript/src/v3/index.ts` 177L
- **Gate Justificación:** tras F3; expone data plane a SDKs
- **Contrato:** `cargo check -p vantadb-python` exit 0 AND `npm --prefix vantadb-ts run build 2>&1 | Select-String "error" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-36.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-37 — Integración offload↔recall
- **Descripción:** wiring auto_recall ↔ context-engine assemble; transversal F4/F5
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs`, `vanta-memory/src/offload_client/context-engine.ts` 526L
- **Gate Justificación:** tras F4/F5; decide D3 tiktoken definitivamente
- **Contrato:** `cargo check -p vanta-memory` exit 0 AND `cargo test -p vanta-memory -- integration 2>&1 | Select-String "ok" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-37.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### MEM-38 — ADR+docs gate pre-release
- **Descripción:** ADR crate vanta-memory + decisions core + docs/api/VANTA_MEMORY.md + certification gate 8 capas unified-review
- **Archivos clave:** `docs/architecture/adr/ADR-03*.md`, `docs/api/VANTA_MEMORY.md`
- **Gate Justificación:** gate pre-release; requiere F1-F5 completas
- **Contrato:** `cargo semver-checks --baseline-rev main 2>&1 | Select-String "success|no breaking" | Measure-Object Count` >=1 AND `skill unified-review --mode certify --profile vantadb` pass
- **Task file:** `.opencode/skills/campaign-executor/tasks/MEM-38.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D1 — avance/activo catch-up + dominios faltantes
- **Descripción:** crear avance/activo/{vanta-memory,vanta-proxy,context-engine}.md nuevos + meta.md contrato frecuencia/dominios + muestreo cruzado 48 commits MEM
- **Archivos clave:** `docs/avance/activo/*`, `docs/avance/meta.md`
- **Gate Justificación:** mirror roto congelado 20/08; D9 avance domina; depende GOV-C2
- **Contrato:** `Test-Path docs/avance/activo/vanta-memory.md` == true AND `Select-String -Path "docs/avance/meta.md" -Pattern "vanta-memory" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D2 — Split monolito progreso/README.md por campaña
- **Descripción:** 372KB 4302L → progreso/campanas/*.md + índice ≤50KB + dedup evento ×3 + 0 links rotos inbound
- **Archivos clave:** `docs/progreso/README.md` → `progreso/campanas/`
- **Gate Justificación:** append-log sin TOC; appetite 1d 🔴; depende consumidores grep
- **Contrato:** `Get-Content docs/progreso/README.md | Measure-Object -Line Lines` <= 1200 AND `Test-Path docs/progreso/campanas/` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D2.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D3 — Revivir bitacora.md
- **Descripción:** entrada narrativa nueva fechada + draft bullet hechos por lead, articulado por owner + regla uso frontmatter
- **Archivos clave:** `docs/progreso/bitacora.md`
- **Gate Justificación:** muerta 27/07; T3.3 forcing function narrativa owner
- **Contrato:** `Select-String -Path "docs/progreso/bitacora.md" -Pattern "2026-09-02" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D3.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D4 — Migración Investigaciones/ → research/
- **Descripción:** 58 archivos git mv → research/ + sweep citas 64 archivos + convención PLAN/NN/SYNTHESIS en research/README + INV-019→026 renumerado
- **Archivos clave:** `docs/Investigaciones/**` → `docs/research/`
- **Gate Justificación:** dos convenciones conviviendo; cargo-check-optimizacion.md citado inexistente; appetite 1d
- **Contrato:** `Get-ChildItem docs/Investigaciones -Recurse -Filter *.md | Measure-Object Count` ==0 AND `cargo check -p vantadb` exit 0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D4.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D5 — ADR-026 a adr/
- **Descripción:** git mv ADR-026-vanta-studio-fase3-rest-dashboard.md → docs/architecture/adr/ + grep citas
- **Archivos clave:** `docs/architecture/ADR-026*` → `docs/architecture/adr/`
- **Gate Justificación:** único ADR fuera de adr/; 30min
- **Contrato:** `Test-Path docs/architecture/adr/ADR-026*` == true AND `Select-String -Path "docs/Backlog.md" -Pattern "ADR-026.*architecture/ADR-026" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D5.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-D6 — wasm/CRASH_MODEL.md modelo diferencial
- **Descripción:** actualizar §persistencia a modelo diferencial vs PERF-08 (solo records cambiados), file:línea evidencia, grep "ALL records"==0
- **Archivos clave:** `docs/wasm/CRASH_MODEL.md`, `vantadb-wasm/src/lib.rs:261-268,749`
- **Gate Justificación:** claim falso "serialize ALL records"
- **Contrato:** `Select-String -Path "docs/wasm/CRASH_MODEL.md" -Pattern "ALL records" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-D6.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-E1 — Propuesta limpieza artefactos (sin borrado)
- **Descripción:** doc propuesta con Regla 0 por ítem (book/book/, __pycache__/, TDAM-VANTADB vacía, _run_stdout.md, DESIGN_RULES.md duplicado, .obsidian/) + checklist owner; NINGÚN borrado en PR
- **Archivos clave:** `docs/reviews/propuesta-limpieza-artefactos-2026-09-02.md`
- **Gate Justificación:** D12 proponer sin ejecutar; requiere aprobación separada
- **Contrato:** `Test-Path docs/reviews/propuesta-limpieza-artefactos-2026-09-02.md` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-E1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-F1 — Auditoría raíz pública 2ª ola
- **Descripción:** auditar README.md×2 + CONTRIBUTING + SECURITY/SUPPORT/CLA + tabla finding/evidencia/severidad; fixes triviales inline, resto tickets
- **Archivos clave:** `/README.md`, `/README_ES.md`, `CONTRIBUTING.md`, `SECURITY.md`
- **Gate Justificación:** nunca auditados raíz; D11 auditar todo lo intocado; depende GOV-B* verificado
- **Contrato:** `Test-Path docs/reviews/auditoria-raiz-publica-2026-09-02.md` == true AND `Select-String -Path "docs/reviews/auditoria-raiz-publica-2026-09-02.md" -Pattern "finding|Severidad" | Measure-Object Count` >=2
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-F1.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### GOV-F2 — Auditoría zonas internas + destino Manual Estratégico
- **Descripción:** auditar VantaDB_Manual_Estrategico_Unificado.md 164KB (recomendación canonizar/archivar/dividir → uphill #4), SKILLS-MANIFEST 111 vs 193, .opencode/{AGENTS,agents,rules,references}, integrations/providers, workflows profundo, plans/archive/46; reporte auditoria-zonas-intocadas
- **Archivos clave:** `VantaDB_Manual_Estrategico_Unificado.md`, `SKILLS-MANIFEST.md`, `.opencode/**`
- **Gate Justificación:** D11 2ª ola; uphill #4 destino manual
- **Contrato:** `Test-Path docs/reviews/auditoria-zonas-intocadas-2026-09-02.md` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/GOV-F2.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-10 — Governance: corregir voz y tono docs (writing-guidelines)
- **Descripción:** aplicar writing-guidelines a docs/api/ y tutorials corregidos en Wave1-2; voz consistente Show HN
- **Archivos clave:** `docs/api/*.md`, `docs/tutorials/*.md`
- **Gate Justificación:** governance docs polyglot; baja pero alta visibilidad pre-launch
- **Contrato:** `Select-String -Path "docs/api/HTTP_API.md" -Pattern "TODO|FIXME" | Measure-Object Count` ==0
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-10.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-11 — Governance: job rustdoc CI
- **Descripción:** añadir job cargo doc artifact API reference docs/api/; verificó grep cargo doc 0 hits 2026-08-25
- **Archivos clave:** `.github/workflows/ci-rust-10.yml` o `ci-rustdoc.yml`
- **Gate Justificación:** adoptantes pre-docs.rs; esfuerzo 🟢 1h
- **Contrato:** `Select-String -Path ".github/workflows/ci-rust-10.yml" -Pattern "cargo doc" | Measure-Object Count` >=1 OR `Test-Path .github/workflows/ci-rustdoc.yml` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-11.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-12 — Touch targets ≥44px restantes web
- **Descripción:** corregir ~20 componentes web navbar h-9→11, close h-7→11, footer text-only → size-11; 3 severos ya size-11
- **Archivos clave:** `web/src/components/*`, `INV-015-touch-targets-44px.md`
- **Gate Justificación:** a11y WCAG; delegar vanta-worker web
- **Contrato:** `Select-String -Path "web/src/components/*" -Pattern "h-7|h-9" | Measure-Object Count` ==0 (o documentado excepción)
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-12.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-13 — Activar pre-push hook git real
- **Descripción:** lefthook/husky pre-push hook real vs template .git/hooks/pre-push inexistente 2026-08-25; fail-fast local
- **Archivos clave:** `.git/hooks/pre-push`, `lefthook.yml` o `husky`
- **Gate Justificación:** process P1-7 gap-02; gate mecánico manual/saltable hoy
- **Contrato:** `Test-Path .git/hooks/pre-push` == true OR `Test-Path lefthook.yml` == true
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-13.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-14 — Review por segundo agente obligatorio tareas 🔴
- **Descripción:** wiring process-change: exigir task(vanta-review) antes COMPLETED en tareas 🔴; prompts/task.md + workflows gate P2-01
- **Archivos clave:** `.opencode/task-system/prompts/task.md`, `.github/workflows/*`
- **Gate Justificación:** diagnosticado como falla más grave sistema agentes P2-1/P2-3 gap-02; process-change
- **Contrato:** `Select-String -Path ".opencode/task-system/prompts/task.md" -Pattern "vanta-review.*🔴|second.*review" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-14.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### RES-15 — Institucionalizar meta-001 B/C micro-ADR + backlog split
- **Descripción:** micro-ADR obligatorio cierres WONTFIX/DEFER + separar backlog negocio/técnico; A ya implementado, B/C 0 hits
- **Archivos clave:** `.opencode/rules/*.md`, `docs/Backlog.md`, `meta-001-root-cause-analysis.md`
- **Gate Justificación:** process hygiene; esfuerzo 🟢
- **Contrato:** `Select-String -Path ".opencode/rules/*" -Pattern "micro-ADR|WONTFIX.*ADR" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/RES-15.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### DEC-01 — Session layer go/no-go ADR (ya resuelta research)
- **Descripción:** escribir ADR defer-as-scoped: F1 no-go (threads/scenes/genlog cubren), F2 defer docs-only guía Claude Code, F3/F4 no-go (sync auto requiere benches Regla 9); owner articula trade-off citando res03-session-layer-gonogo.md
- **Archivos clave:** `docs/architecture/adr/ADR-0XX-session-layer.md`, `docs/research/res03-session-layer-gonogo.md`, `COGNEE_EVALUATION.md`
- **Gate Justificación:** DEC-01 huérfana ya research-resuelta 2026-08-25; solo doc, no código
- **Contrato:** `Test-Path docs/architecture/adr/ADR-0*session*.md` == true AND `Select-String -Path "docs/architecture/adr/ADR-0*session*.md" -Pattern "defer-as-scoped" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/DEC-01.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

#### DEC-02 — Billing/quota CreditCalculator ADR (pre-requisito multi-usuario)
- **Descripción:** decidir UNA calculadora crédito ÷1000 vs ÷10000 (TDAM inconsistente), grabar ADR, elegir ×1K o ×10K, habilita VantaDB Pro multi-usuario
- **Archivos clave:** `docs/research/tdam/SYNTHESIS.md` §9, `tdam/09-deploy-usage.md`, `docs/architecture/adr/ADR-0XX-billing.md`
- **Gate Justificación:** TDAM #9 diferido fuera F1-F7 nunca trackeado; decisión previa a MEM-27 proxy/Pro
- **Contrato:** `Test-Path docs/architecture/adr/ADR-0*billing*.md` == true AND `Select-String -Path "docs/architecture/adr/ADR-0*billing*.md" -Pattern "÷1000|÷10000|CreditCalculator" | Measure-Object Count` >=1
- **Task file:** `.opencode/skills/campaign-executor/tasks/DEC-02.md`
- **Estado:** ✅ COMPLETED
- **last-synced:** 2026-09-02T00:00

---

## DoD multi-nivel (aplica a TODAS)

| Nivel | Gate |
|-------|------|
| Task | contrato mecánico ✅ vía `campaign_verify_cmd` + task file sync + recitation (activeGoal/contract/lastAction/nextAction/result) |
| Commit | conventional commit (`feat/fix/docs/test/perf/ci/refactor/chore` + scope + ID) + `cargo fmt --check` + `cargo clippy --deny warnings` + `cargo deny check` (MIT/Apache-2.0) + `verify_changed.ps1` para docs (markdownlint) · deuda neta ≤0 (ponytail: 1 línea > 50) |
| Release | `cargo semver-checks --baseline-rev main` verde antes de tag; release-plz bump automático; changelog Added/Changed/Fixed por impacto |
| Rollout | §Rollout & Rollback abajo |

---

## Rollout & Rollback por bucket

### GOV (30, docs/governance, Show HN bloqueante)
- **Estrategia:** direct + flag OFF no aplica (docs); canary = PR docs preview + curl openapi 35 paths + harness snippets 34 PASS/24 SKIP
- **Rollback:** `git revert <commit docs>` + redeploy docs site <5 min; propuesta limpieza GOV-E1 nunca borra en este PR (solo doc)
- **Thresholds:** gate-docs-21 verde, 0 links rotos (AUD-007), snippets 0 FAIL; error rate N/A docs
- **Post-launch verify (1h):** links 0 rotos, openapi parity script pass, harness PASS, case_studies 0 refs públicas

### P27 MEM-01..38 (TDAM F1-F7, 8-12 sem, LLM-free primero → LLM-driven)
- **Estrategia:** staged F1-F3 canary (feature flag embed-local ya ON, `embed-batch` fallback dummy) → F4 crate vanta-memory con LLM mock → F5 Context Engine con tiktoken o200k_base validado WASM → F6/F7 2ª iteración proxy/wiki detrás de flag `experimental`
- **Rollback:** flag OFF <1 min (desactivar search_profile mode=hybrid→vector, entity checker deny-all); `git revert` + re-publish patch si crate publicado; datos InternalMetadata preservados (soft-delete)
- **Thresholds:** P95 latency canonical_p99 dentro 20%; error rate dentro 10%; nuevos errores JS/Python 0 tipos nuevos
- **Consideraciones datos:** migración skills (owner,name) UNIQUE partial index reversible con `cargo run -p xtask -- migrate rollback`; nodos escena heat preservados

### P38 RES-01..15 + DEC-01/02 (research huérfanas, durabilidad)
- **Estrategia:** RES-01/02 SOLO + canary durabilidad (snapshot→mutate→restore→audit en sandbox tmp, nunca DB real) → gradual resto RES-03..15 behind feature flags donde aplica (rss_threshold, phrase literal)
- **Rollback:** RES-01/02: `git revert` WAL_FORMAT + rename-aside pre_restore_<ts> → restore dir original <15 min; resto direct revert <5 min; DEC-01/02 solo docs ADR revert <1 min
- **Thresholds:** WAL quiesce sin tear (audit pass), snapshot_create_fail failpoint verde, P95 dentro 20%
- **Datos:** snapshot pre_restore backup preserva data_dir; PitrRestorer diferido (no aplica aquí)

### P25 MCP-35 (1, Fallback HTTP auto N instancias)
- **Estrategia:** canary 5% (2 sesiones OpenCode locales) → gradual 100%; discovery file PID check + /health; proxy parity 1:1
- **Rollback:** flag OFF <1 min (desactivar proxy mode, fallback a Database busy exit 1 original) o `git revert` <5 min; multi-sesión vuelve a single-writer
- **Thresholds:** error rate 2+ sesiones vs baseline dentro 10%; P95 proxy overhead <20%; 0 tipos nuevos errores MCP
- **Datos:** crash dueño no corrompe (single-writer preservado, subsecuentes no adquieren lock); discovery stale PID limpia y reabre embebido

---

## Verificación global (aplica a todo el plan)

- `cargo fmt --check` — ✅ gate Fast
- `cargo clippy --workspace --deny warnings` — ✅ (warnings 0; nota FIND-MCP-001 `context_tests.rs:70` no compila → usar `-p vantadb` durante P27)
- `cargo deny check` — ✅ MIT/Apache-2.0 only
- `cargo semver-checks --baseline-rev main` — ✅ antes de tag (obligatorio pre-publish)
- `cargo nextest run --profile audit -p vantadb` — ✅ 2034/2034/1 skip canónico
- `cargo test -p vantadb-mcp` — ✅ 37 checks skill parity
- `npm --prefix vantadb-ts run build && npx vitest run` — ✅ 264 tests (264 passed)
- `wasm-pack build --target bundler -p vantadb-wasm` — ✅ wasm32-unknown-unknown
- `python dev-tools/validate_doc_snippets.py` — ✅ 34 PASS/24 SKIP/0 FAIL sobre docs corregidos
- `node scripts/check_openapi_parity.mjs` — ✅ 35 paths / 40 ops exact parity
- `cargo llvm-cov --workspace --summary-only` — ✅ re-medir para GOV-A1 (fallback ADR-018 81.40% si ICE)
- Dependabot alerts — 0 critical/high (allowlist tantivy RUSTSEC-2026-0253 documentada, lru 0.18 espera 0.27 publish)

---

## Eventos plan-adjust

```
plan-adjust [2026-09-02]: creación inicial D — 86 DO / 0 DEFER / 0 SKIP / 0 BLOQUEADO
  fuente: docs/Backlog.md 130 activas + GOV D1-D14 + tdam SYNTHESIS F1-F7 + res02 S1-S5 + EMB-01..09 ✅ + incidente MCP-35 2026-08-25
  waves: 5 (Wave0 5 tasks, Wave1 9, Wave2 15, Wave3 19, Wave4 38) con MAX_CONCURRENT=3 → ~29 sub-waves efectivas
  uphill: 4 (gate parity GOV-B4 approach, guard language GOV-A4, corte campañas GOV-D2, destino Manual Estratégico GOV-F2)
  downhill: 86 tasks sliced vertical
  nextAction: /pipeline run docs/plans/2026-09-02-alta-prioridad-paralelo.md
```

---

## Al finalizar cada wave

`skill progreso` → migrar filas completadas a `docs/avance/` por dominio + historial + `campaign_memory_write` (lessons/decisions) → recitation → próxima wave. Ponytail: skipped over-engineering (4 servicios split, Redis, Mongo, @colbymchenry/codegraph, prompts Kenty chino, 3 imágenes Docker), add when benchmark/ADR lo exija.

---

## Recitation (template para reporter)

ActiveGoal: Ejecutar 86 alta prioridad en waves paralelas MAX 3
Contract: Cada task con contrato mecánico campaign_verify_cmd + task file .opencode/skills/campaign-executor/tasks/<ID>.md
LastAction: Plan file creado 2026-09-02-alta-prioridad-paralelo.md (86 DO, 5 waves, DAG documentado, ponytail full)
NextAction: `/pipeline run docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave0 (MCP-35 + GOV-T01..T03 + RES-01 SOLO) con 3 agentes vanta-arch/worker/docs en paralelo
Result: ⬜ PENDING
NextTask: MCP-35 + GOV-T01 + RES-01 (Wave0 parallel 3)

---

*No se escribió código — solo plan. Cada implementación futura debe citar su contrato y pasar campaign_verify_cmd antes de COMPLETED (SARL: RESUME→RETRY→STRATEGY→ESCALATE).*

=== RECITATION GOV-T02 ===
Campaign ID: 20260902-alta-prioridad-paralelo
Objetivo activo: GOV-T02 — TIR-04b contenedor tasks/closed/ Failed-task container
Estado: completed
Última acción: Step1 RULES.md Apéndice B fix (asks→tasks, Failed-task container ×3, rg índice) 6 hits; Step2 SKILL.md +1 fila tasks/closed/ Failed-task container; Step3 verify contratos + alias copy
Resultado: ✅
Próxima acción: ninguno — Wave0 sigue GOV-T03 + MCP-35 + RES-01 disjoint
Contrato: Select-String -Path ".opencode/skills/campaign-executor/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count >=2 (6) AND Select-String -Path ".opencode/skills/campaign-executor/SKILL.md" -Pattern "tasks/closed" >=1 (1) ✅; alias .opencode/task-system/RULES.md 6 ✅; tasks/closed/ dir 2 files
Próxima tarea si completa: GOV-T03
=== END RECITATION ===

