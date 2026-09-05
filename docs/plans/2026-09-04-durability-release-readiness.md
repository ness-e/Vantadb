# Plan de Ejecución: Durability + Release-Readiness (2026-09-04)

> **Campaign ID:** a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
> **Inicio:** 2026-09-04
> **Estado:** ⏳ IN PROGRESS (Wave 0: 3/3 ✅ — FIND-63 `a7285969`, GOV-TK3 `b3be4176`, MEM-63 verificado ya-en-HEAD `6058cc84`)
> **Fuente:** docs/Backlog.md (105 activas, verificado contra código el 2026-09-04)
> **Autonomous:** false
> **Decisiones del usuario (Gate P + spec, 2026-09-04):** set DO 12 ✅ · publish real NO (solo dry-run+checklist) · FUT-12 entra como SPEC (opt-in only, group-commit, ≥10× + ventana declarada) · PRX-01 entra (trigger header+ruta, degraded 3 fallos/5 éxitos)

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 12 | 89 | 2 | 2 |

- **SKIP:** REVIEW-10 (fila stale — `src/cli_server.rs` tiene 13 líneas, split ya hecho en `src/server/` con 10 módulos; limpiar fila al ejecutar), FIND-53 (fila dice ✅ Completada; limpiar fila al ejecutar).
- **BLOQUEADO:** AUD-042 (upstream tantivy 0.27.0 no publicada), GOV-TK2 (requiere decisión de release — va por `/ship`, no por este plan).
- **DEFER (89):** FUT-02..11/13/14 (roadmap sin trigger), PRX-02..13 (gateway milestone — solo PRX-01 entra), UX-02..19 (necesita input visual owner), DESKTOP-40..45 (plataforma/manual), TS-09..13, WSM-02/03/09/11/14, WEB-02/03/05/08/09, INTG-01/02, PROV-01..12, MOD-05/22/23/24, FIND-11/17/20/21/47/48/49/50/60, MEM-66/68/69/70, SRV-01/06/07, GOV-TK5/8, BND-07/12/13, PERF-BENCH-01, MKT-18f (requiere tokens PyPI humanos + PRs upstream con identidad), MKT-18i (BLOQUEADO upstream AnythingLLM), BLOG-CTA (contenido editorial), OLD-01, CI-01, TBH-06, ISSUE-TS-001, MCP-41, DISC-03, DEC-02 (ICEBOX, vigente).

## Waves (FAIL_MODE=parallel, MAX_CONCURRENT=3, archivos disjuntos por wave)

- **Wave 0:** FIND-63 + MEM-63 + GOV-TK3
- **Wave 1:** FIND-62 + GOV-TK7 + STABLE-06
- **Wave 2:** STABLE-04 + BND-08 + FUT-12-spec
- **Wave 3:** BND-09 + MCP-34b + PRX-01 (BND-09 corre solo si BND-08 verificó pipeline; si no, queda BLOQUEADO con evidencia)

### Task 1: FIND-62 — commit_transaction bajo insert_lock + test de interleaving
- **Archivos clave:** `src/storage/engine/txn.rs:119-213`, `src/storage/engine/maintenance.rs:47-93` (flush ERR-010)
- **Gate Justificación:** premisa verificada hoy (0 refs a insert_lock en txn.rs; insert/delete/flush sí lo toman). Race real flush-vs-commit → record invisible en recovery. 🔴 correctness/durabilidad.
- **Contrato:** nuevo test `commit_flush_interleaving` (flush concurrente durante commit → todos los records visibles post-recovery) verde + suite `storage` verde + `cargo clippy -D warnings` + `cargo fmt --check` limpios. Pre-mortem obligatorio: verificar que ningún caller mantiene el guard al llamar commit_transaction (lock no-reentrante → deadlock). Si el fix introduce riesgo de deadlock no resoluble en el appetite → cerrar como BLOQUEADO con evidencia, sin código a medias.
- **Task file:** `tasks/FIND-62.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `fix(storage): commit_transaction bajo insert_lock + test interleaving (FIND-62)`

### Task 2: FIND-63 — rama explícita SyncMode::Never (o documentar que no existe)
- **Archivos clave:** `src/wal.rs:376-389` (`maybe_sync`), `src/config.rs` (enum SyncMode: Always/Periodic/Never)
- **Gate Justificación:** verificado hoy — `maybe_sync` solo ramifica `Always`; variante `Never` existe pero sin brazo (con threshold None fsyncea igual que Periodic). 🟢 <1h.
- **Contrato:** `rg "Never" src/wal.rs` muestra brazo explícito con semántica testeada, O doc que declara "Never no desactiva fsync en esta versión" + test que lo fija; suite `wal` verde + clippy/fmt limpios.
- **Task file:** `tasks/FIND-63.md`
- **Estado:** ✅ COMPLETO (2026-09-05, commit `a7285969` — match exhaustivo Always|Never|Periodic + test RED→GREEN; suite wal 63/63)
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `fix(wal): rama explícita SyncMode::Never (FIND-63)`

### Task 3: MEM-63 — doc stale auto_recall + embeddings auto-on
- **Archivos clave:** `vanta-memory/src/core/hooks/auto_recall.rs:69-73`, Cargo features
- **Gate Justificación:** doc dice que embeddings "degradan hasta wirear" pero MEM-47 ya implementó el hook; fix doc + auto-on con provider configurado (chars-fallback solo sin provider). 🟢 quick-win.
- **Contrato:** doc actualizada + test que verifica auto-on con provider y fallback sin provider + suite `vanta-memory` verde.
- **Task file:** `tasks/MEM-63.md`
- **Estado:** ✅ COMPLETO (2026-09-05, retry fresco: contrato verificado sin edits — fuente ya en HEAD vía `6058cc84`; suite 328/328 + l1_dedup 9/9 ambas features + fmt/clippy/doc OK; sin commit nuevo en main repo porque no hay diff)
- **Ruta:** vanta-docs
- **Branch:** develop
- **Commit:** `docs(memory): auto_recall doc + auto-on embeddings (MEM-63)`

### Task 4: GOV-TK3 — drift yaml↔real ×3
- **Archivos clave:** gramática IQL case del yaml vs parser UPPERCASE; `GraphTraversalBody` (roots numéricos + max_depth requerido); search en DB fresca requiere rebuild-index previo
- **Gate Justificación:** drift doc↔código verificado en auditoría; fix acotado docs (+ behavior si corresponde). 🟠.
- **Contrato:** los 3 drifts resueltos (doc corregida o código alineado, uno por uno con evidencia) + suite afectada verde.
- **Task file:** `tasks/GOV-TK3.md`
- **Estado:** ✅ COMPLETO (2026-09-05, commit `b3be4176` — doc-fix ×3 + parity 5/5; Backlog/avance sync pendiente del orquestador)
- **Ruta:** vanta-docs
- **Branch:** develop
- **Commit:** `docs(api): drift yaml-real ×3 (GOV-TK3)`

### Task 5: GOV-TK7 — put_batch metadatas solo-str
- **Archivos clave:** doc-tutorial vs API de `put_batch`, coercion de metadatas
- **Gate Justificación:** inconsistencia doc↔API 🟡 pequeña. Mini-decisión (alinear doc o ampliar coercion) se resuelve en DISCOVERY vía question-gates, sin nueva ronda.
- **Contrato:** doc y API coinciden (una dirección, documentada) + test de coercion verde.
- **Task file:** `tasks/GOV-TK7.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `fix(api): put_batch metadata coercion alineada (GOV-TK7)`

### Task 6: STABLE-06 — gate npm TS como Fast Gate
- **Archivos clave:** `vantadb-ts/package.json`, `vantadb-ts/src/`, `.github/workflows/release-npm-61.yml`
- **Gate Justificación:** validación 🟠 para promoción default-members. DISCOVERY primero: verificar claim "264 tests / 26s" contra disco; si difiere, re-escalar con evidencia.
- **Contrato:** `npm ci && npm run build && npx vitest run` verde + `npx eslint .` 0 + `npm pack` incluye `engines` + tiempo medido en CI limpio (<5 min o justificado como Heavy).
- **Task file:** `tasks/STABLE-06.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(ts): gate npm Fast Gate medido (STABLE-06)`

### Task 7: STABLE-04 — validar vantadb-mcp (gates 1-6)
- **Archivos clave:** `vantadb-mcp/src/`, `vantadb-mcp/tests/` (11 archivos; DISCOVERY verifica claim "72 mcp_tests")
- **Gate Justificación:** validación 🔴 para promoción; gates 1-6 + `test-mcp.py` 37 checks vs skill.
- **Contrato:** gates 1-6 del contrato P47 pasados con números reportados (check/fmt/clippy/deny/nextest/docs-coverage) + `test-mcp.py` 37/37.
- **Task file:** `tasks/STABLE-04.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(mcp): validación gates 1-6 (STABLE-04)`

### Task 8: BND-08 — pipeline npm napi-rs end-to-end en dry-run (SIN publicar)
- **Archivos clave:** `.github/workflows/release-npm-node.yml` (job Publish existe, OIDC), `vantadb-node/package.json` (0.5.0, npm 404 verificado hoy)
- **Gate Justificación:** el pipeline existe pero el paquete nunca se publicó; scope aprobado: solo dry-run + checklist, publish real = decisión humana posterior.
- **Contrato:** `npm pack` + prepublish artifacts OK + `npm publish --dry-run` verde + checklist de release escrita (`docs/plans/artifacts/bnd-08-publish-checklist.md`). PROHIBIDO publicar.
- **Task file:** `tasks/BND-08.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(node): prepublish verificado dry-run + checklist (BND-08)`

### Task 9: FUT-12-spec — spec de WAL fsync-batching (SOLO SPEC, sin código)
- **Archivos clave:** `src/wal.rs:305-389` (`append`, `batch_append` existentes), `src/config.rs` (SyncMode), nueva spec en `docs/architecture/adr/` o `docs/plans/artifacts/fut-12-spec.md`
- **Gate Justificación:** decisiones del usuario ya tomadas (opt-in only, group-commit, ≥10× + ventana declarada). La spec es el deliverable.
- **Contrato:** spec escrita con objetivo/diseño group-commit/ACEPTACIÓN (≥10× batch + ventana declarada y testeable)/límites (default intacto) + registrada para futura implementación. Cero código productivo.
- **Task file:** `tasks/FUT-12-spec.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-arch
- **Branch:** develop
- **Commit:** `docs(adr): spec WAL fsync-batching opt-in (FUT-12-spec)`

### Task 10: BND-09 — targets linux musl en napi (corre solo si BND-08 verificó pipeline)
- **Archivos clave:** `vantadb-node/package.json` (napi.targets), CI (matrix musl aarch64/x86_64)
- **Gate Justificación:** 🟢 Docker/Alpine sin cobertura; desbloqueada porque el pipeline existe (verificado hoy).
- **Contrato:** targets musl presentes en config + matriz CI los incluye; si el build cross falla por toolchain, documentar requisito y cerrar parcial con evidencia. Si BND-08 no verificó pipeline → cerrar como BLOQUEADO con evidencia, sin tocar nada.
- **Task file:** `tasks/BND-09.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(node): targets linux musl (BND-09)`

### Task 11: MCP-34b — tool `snapshot_restore(name)` (con stop-condition)
- **Archivos clave:** `src/storage/engine/mod.rs:637-796` (`create_snapshot` + `snapshot_restore` + failpoint EXISTEN — verificado hoy), `docs/research/res02-backup-restore.md` §3, `vantadb-mcp/src/handlers/tools.rs`
- **Gate Justificación:** 🟢 wrapper sobre API pública existente. Stop-condition: Step 1 verifica S1 (quiesce+flush en create_snapshot) + tests de restore; si falta → cerrar como BLOQUEADO con evidencia, cero código.
- **Contrato:** tool funciona E2E (validación identifier + confirmación destructiva explícita) + tests MCP verdes, O fila BLOQUEADO documentada.
- **Task file:** `tasks/MCP-34b.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `feat(mcp): snapshot_restore tool (MCP-34b)`

### Task 12: PRX-01 — wiring proxy (advance + classifier + mem-commands + degraded)
- **Archivos clave:** `vanta-proxy/src/server.rs`, `session.rs`, `session/claude_code.rs`, `mem_command.rs`, `rate_limit.rs`
- **Gate Justificación:** 🟡 cablear código ya construido; decisiones tomadas (trigger header+ruta, degraded 3 fallos/5 éxitos).
- **Contrato:** (1) `SessionStore::advance()` dispara por header Y ruta dedicada + test; (2) `classify_cc_request` consume routing Main/Fork/Sidequery + test; (3) `mem:sync`/`create-skill` ejecutan pipeline real (no stub) + test; (4) `set_degraded(true)` tras 3 upstream 429/5xx consecutivos, sale con 5 éxitos + test. Suite `vanta-proxy` verde + clippy/fmt.
- **Task file:** `tasks/PRX-01.md`
- **Estado:** ⬜ PENDING
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `feat(proxy): wiring advance+classifier+mem-commands+degraded (PRX-01)`

## Dependencias entre Tasks
Wave 0 → Wave 1 → Wave 2 → Wave 3 (arriba). BND-09 gated por BND-08. STABLE-09 excluido a propósito (requiere STABLE-05/07 fuera del plan). Limpieza de filas al cerrar: REVIEW-10, FIND-53 (stale) → backlog-history con nota.

=== RECITATION FIND-63 ===
Campaign ID: a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
Objetivo activo: FIND-63: rama explicita SyncMode::Never en maybe_sync
Estado: completed
Última acción: GREEN+CIERRE: match exhaustivo Always|Never|Periodic, test RED->PASS, suite wal 63/63, fmt+clippy limpios, commit a7285969 solo src/wal.rs (+62/-9)
Resultado: OK
Próxima acción: ninguno (siguiente tarea del plan la decide el orquestador: MEM-63 / GOV-TK3, Wave 0)
Contrato: verificacion: cargo nextest run -p vantadb --lib wal (63/63 OK) + cargo fmt --check -p vantadb OK + cargo clippy -p vantadb --all-targets -- -D warnings OK + pre-commit hook OK; evidencia: rg Never src/wal.rs -> lineas 229/380/382/868/895/898/903 | claim: Never nunca auto-syncea, sync() manual intacto | evidencia: wal::tests::test_sync_mode_never_skips_auto_sync PASS | confianza: alta; artefactos: src/wal.rs, .opencode/skills/campaign-executor/tasks/FIND-63.md; invariantes: Periodic/Always sin cambio de comportamiento; ningun caller actual usaba Never salvo path config-file hot-reload; deuda: ninguna; queda_pendiente: orquestador actualiza Task 2 a completada en docs/plans/2026-09-04-durability-release-readiness.md (no tocado: untracked de otra sesion)
Próxima tarea si completa: MEM-63
=== END RECITATION ===

=== RECITATION GOV-TK3 ===
Campaign ID: a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
Objetivo activo: GOV-TK3: 3 drifts yaml-real resueltos (doc-fix) + suite verde
Estado: completed
Última acción: S1-S7 + commit b3be4176 docs(api): drift yaml-real ×3 (GOV-TK3) + plan file Task4->COMPLETO (sin stagear) + 2 lessons
Resultado: OK
Próxima acción: ninguno (orquestador: Backlog/avance sync + push + proxima Wave 0/1)
Contrato: verificacion: cargo test -p vantadb --test openapi_yaml_parity 5/5 + parser lib 117/117 + fmt/clippy -D warnings + docs-coverage 0 gaps + hooks pre-commit ok | evidencia: claim drift1 -> openapi.yaml UPPERCASE + IQL.md TYPE/.. + test case-sensitivity (alta) | claim drift2 -> 5 bodies por endpoint + test_http_shapes + 0 refs GraphTraversalBody (alta) | claim drift3 -> notas startup-only+sintoma+remedio + test anti-cli_server (alta) | artefactos: b3be4176 (4 files), tasks/GOV-TK3.md | invariantes: 0 codigo productivo; ajenos intactos (.opencode, web/, assets, Backlog, avance, plan file sin stagear) | deuda: targets integracion no compilan en entorno (pre-existente); full nextest excluido con motivo | queda_pendiente: orquestador sync Backlog fila GOV-TK3 + avance + push
Próxima tarea si completa: ninguno
=== END RECITATION ===

=== RECITATION 3 ===
Campaign ID: a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
Objetivo activo: MEM-63 doc stale auto_recall + embeddings auto-on (Task 3 Wave 0)
Estado: in-progress
Última acción: Re-verificado contrato sin edits de fuente (ya en HEAD); actualizados plan+task+backlog+avance
Resultado: ✅
Próxima acción: git add solo 4 archivos + commit docs(memory) MEM-63, luego task 3 completed
Contrato: verificacion: cargo test -p vanta-memory --lib 328 OK + l1_dedup 9/9 ambas features + fmt/clippy/doc OK; evidencia: claim doc sin degradan→grep 0 en vanta-memory/ (alta); claim auto-on con provider→default_wires_local_provider_when_feature_on verde con embed-local (alta); claim fallback sin provider→default_stays_keyword_only_without_feature verde (alta); artefactos: docs/plans/2026-09-04-durability-release-readiness.md, .opencode/skills/campaign-executor/tasks/MEM-63.md, docs/Backlog.md, docs/avance/activo/vanta-memory.md; invariantes: no tocar web/assets/.opencode ajenos, no re-escribir historia 6058cc84; deuda: cargo doc 60 warnings redundant_explicit_links + sin deny(missing_docs) en vanta-memory; queda_pendiente: commit docs(memory) + marcar task 3 completed
Próxima tarea si completa: GOV-TK3
=== END RECITATION ===

=== RECITATION MEM-63 ===
Campaign ID: a6f16be4-a2a2-44eb-bfdb-1a84a4b573cf
Objetivo activo: MEM-63 doc stale auto_recall + embeddings auto-on (Task 3 Wave 0, RETRY fresco)
Estado: completed
Última acción: Re-verificacion fresca sin edits; Task 3 marcada COMPLETO en plan file (sin stagear)
Resultado: OK
Próxima acción: ninguno (orquestador: Wave 1)
Contrato: verificacion: cargo test -p vanta-memory --lib 328/328 + embed-local l1_dedup 9/9 + fmt/clippy/doc OK; evidencia: claim doc sin degradan->grep 0 vanta-memory/ (alta); claim auto-on->default_wires_local_provider_when_feature_on PASS (alta); claim fallback->default_stays_keyword_only_without_feature PASS (alta); artefactos: auto_recall.rs + l1_dedup.rs (HEAD via 6058cc84), tasks/MEM-63.md (submodule 7e05f8a), plan Task 3 COMPLETO; invariantes: 0 edits fuente, ajenos sin tocar; deuda: cargo doc 60 warnings + sin deny(missing_docs) (pre-existentes); queda_pendiente: ninguno
Próxima tarea si completa: ninguno
=== END RECITATION ===
