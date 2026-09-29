---
title: "Plan de Ejecución: Master Roadmap VantaDB — de 0.7.x a 1.0 (fuente única)"
kind: plan
description: "SDP: campaign-executor · progreso · writing-plans · planning-and-task-breakdown · api-and-interface-design · spec-driven-development · documentation-and-adrs · writing-guidelines"
---

# Plan de Ejecución: Master Roadmap VantaDB — de 0.7.x a 1.0 (fuente única)

> **Campaign ID:** ed20beae-edf6-42f5-b41f-e8519830d6cb
> **Inicio:** 2026-09-26
> **Estado:** ⏳ EN PROGRESO (34/50 — F0 ✅ · F1 ✅ · F2 ✅ · **F3 ✅ 11/11**; siguiente: F4 — bloques primero) | **Gate F3: ✅ local** (migración determinista verde + corte 0.8.0 preparado: guía + marcador R1 `b9296909` + R2; release real = lane owner post-push)
> **Fuente:** `docs/dev/Backlog.md` (P52–P59 + FIND-*) + planes absorbidos (`2026-09-24-post-investigacion-integral.md` W2–W7, `2026-09-24-estabilizacion-pendiente.md`, `2026-09-24-harness-gaps.md`, `2026-09-24-sesion-continuidad.md`, `2026-09-20-estabilizacion-total.md`) + `docs/dev/strategy/` (13 docs) + Notion "VantaDB Docs" (~28 subpáginas) + investigación de riesgos 2026-09-26 (4 sub-agentes R1–R4, multi-fuente)
> **Autonomous:** false — el owner gatea push/merge/release. **Push a `develop`: al completar el plan con todo validado/verificado** (decisión owner 2026-09-26, ver §Política de commits/push)
> **Modo:** PLAN → ejecución con `/pipeline run docs/dev/plans/2026-09-26-master-roadmap.md`
> **Orden:** fases F0→F6 ESTRICTAS (no paralelizar entre fases); dentro de cada fase hasta 3 tasks en paralelo. ⚠️ Caveat RAM (lección histórica): con verifies pesados el efectivo puede ser 1.
> **Nota:** este plan ABSORBE los 5 planes activos anteriores (archivados a `docs/dev/plans/archive/`). Única fuente de plan.

## SDP (skills del plan — campaign_discover_skills_v2 phase=PLAN)

SDP: `campaign-executor` · `progreso` · `writing-plans` · `planning-and-task-breakdown` · `api-and-interface-design` · `spec-driven-development` · `documentation-and-adrs` · `writing-guidelines`
(Adicionales por fase: ver §Herramientas, skills y MCP.)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 50 |
| 🟡 DEFER | cola restante del Backlog (ver §Cola restante — disposición) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 (deps internas por fase; ninguna externa al plan) |

Status: ⬆️ uphill = 5 (F2–F6 con bloques esenciales que se COMPLETAN al nivel F0/F1 al iniciar cada fase — REGLA en §F2–F6) · ⬇️ downhill = 49 (F0/F1 con detalle completo; HARD-08 ✅ pre-run)

## Gates por fase

| Fase | Contenido | Gate de salida |
|------|-----------|----------------|
| **F0** | Hardening + quick wins (HARD-01..07) | verify verde + bundles + rails de breaking + gates tuneados + deuda quick cerrada |
| **F1** | Seguridad/frontera (WIRE-10, DEF-01..08) | 0 críticos seguridad + frontera de producto declarada y verificable |
| **F2** | Cableado (WIRE-02..08) | loop proxy E2E verificable + paridad bindings |
| **F3** | Esquema (MGR-10/12/13 → SCH-01..08) | migración determinista verde + corte **0.8.0** (release-plz) |
| **F4** | Verificabilidad (VER-*) | features de categoría "verificable" con tests + attestations |
| **F5** | ICP + Harness (MKT-18f, ICP-01..03, VER-08, DEF-06) | harness publicado (dataset commiteado) + adapters PyPI |
| **F6** | Anuncio (VER-09, EXE-01, N-17) | `/audit certify` + `/ship` GO + anuncio (owner) |

## Política de commits/push (decisión owner 2026-09-26)

- **Cada tarea = 1 commit local** (conventional commit + task ID). Nunca push por tarea.
- **Push a `develop`:** SOLO al completar el plan con todo validado/verificado (decisión owner). Mitigación de pérdida: backups `git bundle` automáticos (HARD-03).
- **PR a main:** solo desde `develop` + OK owner. **Release:** release-plz (nunca tags/versión/CHANGELOG a mano).
- **Gate H:** todo cambio en `.opencode/` (HARD-02/03/05/07) requiere `/harness` verde antes del commit.

---

## Tasks

### Task 1: HARD-01 — Rails de breaking changes (semver-checks + public-api + docs de migración)
- **Fase:** F0

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `release-plz.toml` · `docs/api/VERSIONING.md` · `docs/api/DEPRECATIONS.md` (nuevo) · `docs/api/COMPATIBILITY.md` (nuevo) · `docs/user/operations/UPGRADE.md` (existente) · `Cargo.toml` (metadata `cargo-semver-checks` lints) · `tests/api/public_api.rs` + `public-api.txt` (nuevo snapshot) · `.github/workflows/ci-rust.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — `semver_check = true` ya activo (`release-plz.toml`); `VERSIONING.md` lista 11 superficies (API-09, commit `032cbd0f`); `ci-rust.yml:92-128` ya tiene job `semver-checks` pero **main-only** → gap real: gate en PR + snapshot `cargo-public-api` + `DEPRECATIONS.md`/`COMPATIBILITY.md` (R1: semver-checks solo ve Rust → falta gate por superficie).
- **Gate Justificación:** rails baratos con 0 usuarios (R1: DuckDB/Polars/Qdrant/Milvus + cargo-semver-checks/cargo-public-api); cierra la desventaja "breaking by design" antes de que haya consumidores.
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo semver-checks check-release` exit 0 (o findings triados y documentados) Y `cargo test -p vantadb --test public_api` verde con snapshot commiteado Y `DEPRECATIONS.md` + `COMPATIBILITY.md` + `UPGRADE.md` existen con 0 links rotos (validate-docs-coverage verde)"
- **Task file:** `docs/dev/tasks/HARD-01.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟨 complicado — configurar lints semver sin ruido + snapshot tooling + plantilla de migración.
- **Top 3 riesgos:** 1. snapshot `public-api` ruidoso → CI rojo permanente · 2. `cargo-semver-checks` lento en fast gate · 3. docs de migración vacías (checklist decorativo).
- **Pre-mortem:** F1: snapshot sin exclude-list → falsos positivos constantes; F2: gate no instalado en CI → skip silencioso; F3: `UPGRADE.md` duplica CHANGELOG (una fuente: CHANGELOG es release notes; UPGRADE es guía de migración).
- **Stop conditions:** gate >5min en fast → mover a nightly (coordina HARD-02); appetite >3d → entregar snapshot + DEPRECATIONS y diferir COMPATIBILITY.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Snapshot ruidoso | `exclude` acotado + `assert_eq_or_update` | CI rojo 2 corridas seguidas |
  | 🟢×🔴 | Gate skip silencioso | verificar presencia en CI con `--version` + step obligatorio | PR sin step |
  | 🟡×🟡 | Docs vacías | checklist 1.0 con exit criteria verificables (R1 rec. #7) | review P2-01 |
- **Uphill/Downhill:** ⬆️ 1 (formato final del snapshot) / ⬇️ 6 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** consume R1 (rec. #2/#3/#4/#5/#6/#7); `feat!`→MINOR automático ya lo hace release-plz en 0.x.

### Task 2: HARD-02 — Tuning de gates (coverage reporte, review risk-tiered, nightly, release dry-run)
- **Fase:** F0

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H para partes `.opencode/`)
- **Archivos clave:** `dev-tools/verify.ps1` · `.config/nextest.toml` · `.github/workflows/ci-rust.yml` (+ nuevo nightly) · `docs/dev/operations/CI_POLICY.md` · `.opencode/task-system/prompts/pipeline-full.md` (tiering de review) · `CONSTRAINTS.md` (quality bar — actualizar con decisión owner)
- **Verificación real:** ✅ CÓDIGO-REAL — `verify.ps1` corre coverage bloqueante (`--fail-under-lines 60`, reparado en API-09); fast gate actual con 11 pasos; sin nightly de certificación pesada; review P2-01 sin tiering. Decisión owner 2026-09-26: aplicar **(a)+(b)+(c) completo** (coverage→reporte+presupuesto, review risk-tiered, nightly).
- **Gate Justificación:** R1 (Google eng: coverage lossy; fast<5min vs nightly) + decisión owner explícita; reduce costo de mantenimiento sin perder señal.
- **Gate Result:** ✅ DO
- **Contrato:** "coverage ya no bloquea (reporte + presupuesto por directorio documentado en CI_POLICY) Y review risk-tiered documentado y activo en prompts (solo diffs `docs/api|sdk|parser|storage|wire` → adversarial; resto verify fast) Y workflow nightly de certificación pesada existe Y fast gate medido <5min"
- **Task file:** `docs/dev/tasks/HARD-02.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟨 complicado — cambiar gates sin debilitar el floor (`floor-guard.ps1`) + coordinar prompts.
- **Top 3 riesgos:** 1. debilitar el quality bar (CONSTRAINTS.md) · 2. nightly que nadie mira · 3. tiering de review que deje pasar breaking.
- **Pre-mortem:** F1: quitar coverage bloqueante sin presupuesto → deriva silenciosa; F2: nightly sin notificación → rojo crónico; F3: tiering ambiguo → reviews degradados.
- **Stop conditions:** floor-guard rojo → revertir coverage a bloqueante; appetite >3d → solo (b)+(c).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Quality bar debilitado | actualizar CONSTRAINTS.md con la decisión + floor-guard verde | floor-guard rojo |
  | 🟡×🟡 | Nightly ignorado | notificación en fallo (issue automático) + badge | 2 semanas sin revisar |
  | 🟢×🟡 | Tiering laxo | lista de paths explícita + test de la regla | review P2-01 |
- **Uphill/Downhill:** ⬆️ 1 (notificación nightly) / ⬇️ 6 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** Gate H obligatorio (`.opencode/`); FIND-134..147 (CI) y FIND-152 (harness residual) se evalúan/cierran acá si siguen vigentes.

### Task 3: HARD-03 — Continuidad local + release trains
- **Fase:** F0

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5-1d · **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `scripts/git-backup.ps1` (nuevo: `git bundle create` + `verify`) · `docs/dev/workflow/RULES.md` (política) · `CONTRIBUTING.md` · `docs/dev/plans/2026-09-26-master-roadmap.md` (§Política — referencia)
- **Verificación real:** ✅ CÓDIGO-REAL — 26+ commits locales sin push (medido `git rev-list --count origin/develop..develop`); sin script de backup; política de push dispersa (AGENTS.md Regla 7 + flujo mínimo). Decisión owner: push al final del plan; bundles como mitigación (R2: lección GitLab 2017).
- **Gate Justificación:** riesgo real de pérdida local sin CI visible; mitigación barata y sin cambiar el control del owner.
- **Gate Result:** ✅ DO
- **Contrato:** "`pwsh scripts/git-backup.ps1` crea bundle con timestamp y `git bundle verify` exit 0 Y política de commits/push/trenes documentada en `docs/dev/workflow/RULES.md` Y `CONTRIBUTING.md` referencia el script"
- **Task file:** `docs/dev/tasks/HARD-03.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 obvio — script + doc.
- **Top 3 riesgos:** 1. bundle a misma unidad del disco → 2. backup nunca ejecutado (sin ritual) → 3. política contradictoria con AGENTS.md.
- **Pre-mortem:** F1: backup en C: (mismo disco) → definir unidad externa o carpeta sincronizada; F2: sin recordatorio → agregar al flujo mínimo (Gate H); F3: duplicar política → RULES.md como fuente única, AGENTS.md referencia.
- **Stop conditions:** si Gate H rechaza el cambio en AGENTS.md → dejar solo script + RULES.md.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Pérdida de 26+ commits | bundle diario + verify | sin backup >7d |
  | 🟢×🟡 | Ritual olvidado | integrar al flujo mínimo (1 línea) | 2 semanas |
  | 🟢×🟢 | Duplicación de política | RULES.md fuente única | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** release trains: 0.8.0 tras F3 (release-plz); 1.0 con exit criteria de HARD-01. NO checkpoint push (decisión owner).

### Task 4: HARD-04 — vanta-memory: spec de fachada + triggers de exposición
- **Fase:** F0

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🟡
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/api/VANTA_MEMORY.md` · `docs/dev/Backlog.md` (FIND-160 co-referencia)
- **Verificación real:** ✅ CÓDIGO-REAL — crate `publish=false`, 423 ítems pub, 9 módulos (R3); ya consumible vía MCP (`memory_recall`/`memory_search`), proxy y desktop; 0 símbolos en bindings (Gate P core-only). Gap: sin spec de fachada ni triggers documentados.
- **Gate Justificación:** R3 — mantener core-only (costo 0) + prep sin código; triggers medibles evitan exponer por entusiasmo.
- **Gate Result:** ✅ DO
- **Contrato:** "`docs/api/VANTA_MEMORY.md` incluye §Facade (capture/recall/seed/ingest, firma conceptual + degradación) Y §Exposure triggers T1–T4 (≥5 pedidos externos / adapter ICP-03 bloqueado / 2 releases sin breaking / stranger-tests confirman demanda local-first) Y validate-docs-coverage verde Y 0 símbolos nuevos en bindings"
- **Task file:** `docs/dev/tasks/HARD-04.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 obvio — doc.
- **Top 3 riesgos:** 1. prometer superficie no estable · 2. triggers no medibles · 3. re-abrir Gate P sin evidencia.
- **Pre-mortem:** F1: facade como contrato firme → marcar "candidate, not published"; F2: triggers vagos → fechas/umbrales numéricos; F3: scope creep a bindings → PROHIBIDO (Gate P).
- **Stop conditions:** si aparece señal de exposición en el mismo PR → BLOQUEO (core-only).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | Facade como promesa | etiqueta "candidate" + revisión en 1.0 | review P2-01 |
  | 🟢×🟢 | Triggers decorativos | tabla medible con fuentes | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** consume R3; FIND-160 (cobertura módulos) se paga en HARD-06.

### Task 5: HARD-05 — Entorno local blindado (regla `-p`, required-features, target dir)
- **Fase:** F0

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H)
- **Archivos clave:** `.opencode/AGENTS.md` (regla dura) · `Cargo.toml` (`[[test]] required-features`) · `docs/dev/references/troubleshooting.md` (entrada del incidente) · `dev-tools/verify.ps1` (ya scoped — verificar)
- **Verificación real:** ✅ CÓDIGO-REAL — incidente 2026-09-26: `cargo test` sin `-p` desde raíz → default-members unifican features (`vantadb-server` pide `server`) → test arranca HTTP real y cuelga :8080 (mitigado: `#[cfg(not(feature = "server"))]` en `tests/cli_tests.rs`, commit `f6c395ef`); verify.ps1 ya usa `-p vantadb`; nextest filters calificados (BND-06).
- **Gate Justificación:** R2 — riesgo vivo en invocaciones ad-hoc; regla + defensa dura eliminan la clase de fallo.
- **Gate Result:** ✅ DO
- **Contrato:** "`AGENTS.md` prohíbe `cargo test`/`nextest` sin `-p` desde la raíz Y targets feature-gated tienen `required-features` (sin skip silencioso) Y suite unificada (`cargo test --test cli_tests`) completa sin hang Y `troubleshooting.md` documenta síntoma/causa/fix"
- **Task file:** `docs/dev/tasks/HARD-05.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 obvio — regla + atributos.
- **Top 3 riesgos:** 1. required-features que oculten tests válidos · 2. regla sin enforcement · 3. Gate H rechaza wording.
- **Pre-mortem:** F1: required-features mal puesto → tests desaparecen; F2: nadie lee la regla → ponerla en flujo mínimo; F3: doc sin comando exacto → incluir repro.
- **Stop conditions:** si un target requiere feature no-default → evaluar caso por caso (no forzar).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Tests ocultos | revisar conteos before/after por target | diff de count |
  | 🟢×🟡 | Regla ignorada | flujo mínimo + troubleshooting | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 4 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** Gate H; coordina con HARD-02 (mismo verify).

### Task 6: HARD-06 — Deuda quick wins (FIND-162, FIND-160, FIND-154, decisión FIND-161)
- **Fase:** F0

- **Appetite:** max 2d · **Esfuerzo:** 🟡 1d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ docs para FIND-160)
- **Archivos clave:** `scripts/validate-docs-coverage.ps1` (FIND-162) · `docs/api/VANTA_MEMORY.md` (FIND-160) · `.github/workflows/perf-bench.yml` (FIND-154) · `docs/api/ERROR_HANDLING.md` + `docs/api/HTTP_API.md` (FIND-161) · `docs/dev/Backlog.md` (filas) · `docs/dev/tasks/MEM-55*` (bookkeeping)
- **Verificación real:** ✅ CÓDIGO-REAL — FIND-162: sección 3 del script inerte tras rename `VantaError`→`Error` (`validate-docs-coverage.ps1:102` vs `src/error.rs:137`); FIND-160: módulos sin doc (dream/gateway/ingest/services); FIND-154: gate perf falso positivo por varianza (rojo en main, verde en develop, dispatch `36101773914` vs `36094025761`); FIND-161: envelope real `{success:false, error|data, code(,hint)}` documentado pero RFC 9457 pendiente de decisión; FIND-153 ya resuelto (`114f55f0`) → solo cierre.
- **Gate Justificación:** baratos (15min-1d) y desbloquean el "CI verde" que FASE-A necesita (R3: FIND-154 contamina el checklist).
- **Gate Result:** ✅ DO
- **Contrato:** "FIND-162: check reparado o retirado con motivo + `rg VantaError scripts/` = 0 (o excepción documentada) Y FIND-160: secciones por módulo O exclusión motivada Y FIND-154: gate ya no falsea (tolerancia/quarantine documentada) Y FIND-161: decisión registrada (ADR o decision-memory) + docs coherentes; filas del Backlog actualizadas/cerradas"
- **Task file:** `docs/dev/tasks/HARD-06.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 obvio — fixes acotados.
- **Top 3 riesgos:** 1. FIND-154 sin repro estable → fix a ciegas · 2. FIND-161 decisión que toca superficies · 3. mezclar 4 fixes en un commit.
- **Pre-mortem:** F1: perf gate con varianza irreducible → quarantine + issue; F2: RFC 9457 adoptado a medias → decisión binaria documentada; F3: commit no atómico → 1 commit por FIND (4 commits, 1 task).
- **Stop conditions:** FIND-154 si el fix requiere reescribir el gate → DEFER con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | FIND-154 reincide | quarantine + métrica de varianza | 2 falsos positivos más |
  | 🟢×🟡 | FIND-161 ambiguo | opciones + decisión owner si hay tradeoff | Gate P |
- **Uphill/Downhill:** ⬆️ 0 (FIND-161 resuelto: owner eligió B) / ⬇️ 0 — steps 5/5 ✅ (verify_changed OK)
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** FIND-153 se evalúa junto a FIND-154; MEM-55 (task file huérfano) → catalogar o cerrar.

### Task 7: HARD-07 — Review gate mecanizado (reviewer_context ≠ author_context)
- **Fase:** F0

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H; revisión vanta-harness)
- **Archivos clave:** `.opencode/task-system/state-tools.mjs` (invariante REVIEW→ACCEPT) · `.opencode/task-system/prompts/pipeline-full.md` · `.opencode/task-system/prompts/question-gates.md` · `docs/dev/workflow/RULES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — incidente API-09: §Review registró review degradado (mismo contexto) habiendo subagentes frescos; se corrigió manualmente con review fresco (sesión `ses_f2329896bffeh83Kc70QBccoVe`). Gap: la degradación no bloquea el ACCEPT.
- **Gate Justificación:** R2 (Anthropic best practices + self-preference bias arXiv 2404.13076 + CriticGPT): sin enforcement mecánico, el gate se degrada en silencio.
- **Gate Result:** ✅ DO
- **Contrato:** "REVIEW→ACCEPT exige `reviewer_context ≠ author_context` (o waiver registrado que BLOQUEA el ACCEPT) Y simulación de review degradado NO permite ACCEPT (caso de prueba en el task file) Y Gate H verde"
- **Task file:** `docs/dev/tasks/HARD-07.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟨 complicado — tocar la state machine sin romper el flujo.
- **Top 3 riesgos:** 1. romper el state machine (ACCEPT bloqueado de más) · 2. waiver que se vuelva costumbre · 3. regresión en tareas ya completadas.
- **Pre-mortem:** F1: invariante demasiado estricto → probar con tareas históricas; F2: waiver sin registro → waiver = decisión owner registrada; F3: sin test → simulación obligatoria.
- **Stop conditions:** si rompe el flujo de >2 comandos → revertir y rediseñar.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | State machine roto | caso de prueba + Gate H | ACCEPT bloqueado en tarea válida |
  | 🟢×🟡 | Waivers crónicos | waiver = owner + registro | 2 waivers seguidos |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 4 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** complementa HARD-02 (política) — este mecaniza el invariante.

### Task 8: WIRE-10 — Distribución P0 (`install.sh` macOS, Colab a `Client`, hooks sin `pwsh`)
- **Fase:** F1

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `scripts/install.sh` · `examples/colab/vantadb_quickstart.ipynb` · hooks (claude/codex/cursor/opencode templates) · `src/cli*.rs` (`mcp-call` superficie)
- **Verificación real:** ✅ CÓDIGO-REAL — implementada 2026-09-25 (commit canónico `8e55e853`, steps 5/5 ✅) + cierre formal 2026-09-27 (verify.ps1 ALL 10 PASS + P2-01 fresco APPROVE + FIND-169). El "⏳ IN PROGRESS / 5 steps pendientes" previo era metadata stale — los Steps del task file mandan.
- **Gate Justificación:** P0 de distribución (roadmap W2/A2); sin esto, macOS/Colab/hooks rotos.
- **Gate Result:** ✅ DO
- **Contrato:** ver `docs/dev/tasks/WIRE-10.md` §Contrato (leer al ejecutar; el task file es fuente de verdad del contrato fino)
- **Task file:** `docs/dev/tasks/WIRE-10.md` (existente)
- **Estado:** ✅ COMPLETED (2026-09-27 — cierre F1a; commit `8e55e853`; §Review con ronda fresca; FIND-169) · **Branch:** develop · **Commit:** `8e55e853`
- **Cynefin:** 🟨 complicado — multiplataforma (shell/notebook/hooks).
- **Top 3 riesgos:** 1. checksum macOS varía por arch · 2. Colab sin red/API 3. hooks con paths Windows.
- **Pre-mortem:** F1: install.sh sin verificación de checksum → fallback; F2: notebook con API removida → usar `Client` (ya planificado); F3: hooks asumen `pwsh` → portar a sh.
- **Stop conditions:** si macOS no verificable en local → smoke documentado + CI matrix.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Verificación macOS limitada | CI matrix + script portable | 2 fallos en matrix |
  | 🟢×🟡 | Hooks rompen agentes | templates por agente + smoke | reporte de agente |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 5 steps (del task file)
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** task file referencia el plan viejo → actualizado a este master.

### Task 9: DEF-01 — Decisión única de producto + alineación SPEC/README/VISION
- **Fase:** F1

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs (+ Gate P owner)
- **Archivos clave:** `SPEC.md` / `docs/SPEC.md` (según exista) · `README.md` · `VISION.md` · `docs/dev/strategy/` (fuente)
- **Verificación real:** ✅ CÓDIGO-REAL — conflicto detectado en investigación (R4: README "RAG/edge" vs SPEC; P55 fila DEF-01). Jerarquía propuesta: un núcleo ("SQLite para agentes") + 3 puertas ICP.
- **Gate Justificación:** bloquea coherencia de DEF-05/DEF-07 y los one-pagers ICP; decisión de producto → Gate P con owner.
- **Gate Result:** ✅ DO
- **Contrato:** "SPEC/README/VISION declaran la MISMA jerarquía (1 núcleo + 3 puertas) sin contradicciones Y Gate P registrado (owner) Y validate-docs-coverage verde"
- **Task file:** `docs/dev/tasks/DEF-01.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟨 complicado — decisión de producto con tradeoffs.
- **Top 3 riesgos:** 1. decisión sin owner → Gate P obligatorio · 2. tocar README técnico en español/inglés mal · 3. scope creep a marketing.
- **Pre-mortem:** F1: proponer jerarquía sin pregunta → bloquear hasta respuesta; F2: duplicar estrategia → referenciar `strategy/`.
- **Stop conditions:** owner no disponible → dejar propuesta + BLOQUEO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Decisión sin owner | Gate P (question) antes de editar | sin respuesta |
  | 🟢×🟡 | Drift docs | mismo-PR Regla 3 | review P2-01 |
- **Uphill/Downhill:** ⬆️ 1 (decisión owner) / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** base para DEF-05.

### Task 10: DEF-02 — `EXPERIMENTAL_FEATURES.md` regenerado a 0.7.0 + categorías labs
- **Fase:** F1

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/EXPERIMENTAL_FEATURES.md` · código/features referenciados (verificación por fila)
- **Verificación real:** ✅ CÓDIGO-REAL — fixes inmediatos de 4 claims falsos ya aplicados 2026-09-24; falta versionar a 0.7.0 + verificar cada fila contra código + categorías labs (proxy/desktop/web/memory).
- **Gate Justificación:** claims honestos (Regla 11) + base de DEF-07.
- **Gate Result:** ✅ DO
- **Contrato:** "cada fila verificada contra código/feature real (evidencia file:line o comando) Y versión 0.7.0 declarada Y categorías labs explícitas Y 0 claims sin respaldo (Regla 11)"
- **Task file:** `docs/dev/tasks/DEF-02.md`
- **Estado:** ✅ COMPLETED
- **Cynefin:** 🟦 obvio — barrido de verificación.
- **Top 3 riesgos:** 1. fila sin feature real · 2. benchmark sin comando reproducible · 3. omitir una feature.
- **Pre-mortem:** F1: claims sin bench → citar `BENCHMARKS.md` o quitar; F2: feature renombrada → grep/feature check.
- **Stop conditions:** >3 filas irreconciliables → FIND + nota.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Claims sin respaldo | Regla 11: comando o quitar | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 4 steps
- **DoD task:** contrato ✅ · task file sync · recitation

### Task 11: DEF-03 — Frontera verificable en CI (`validate-frontier`)
- **Fase:** F1

- **Appetite:** max 2d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🟠
- **Ruta:** vanta-lead
- **Archivos clave:** `scripts/validate-docs-coverage.ps1` (extender) o `scripts/validate-frontier.ps1` (nuevo) · `.github/workflows/*` (step) · `docs/EXPERIMENTAL_FEATURES.md` (frontera declarada)
- **Verificación real:** ✅ CÓDIGO-REAL — validate-docs-coverage existe (0 gaps hoy); gap: no valida la frontera contra features Cargo/rutas reales (P55).
- **Gate Justificación:** convierte "frontera declarada" en gate mecánico (cierra drift día-1, R1).
- **Gate Result:** ✅ DO
- **Contrato:** "script valida cada fila Production-facing contra feature/ruta real (exit ≠0 si no existe) Y corre en CI Y verde en main actual"
- **Task file:** `docs/dev/tasks/DEF-03.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commits `8a884ba6` ci + `b82f90cb` cierre; re-review fresco P2-01 OK)
- **Cynefin:** 🟦 obvio — script + step.
- **Top 3 riesgos:** 1. falsos positivos por rutas condicionales · 2. script lento · 3. acoplarse a docs frágiles.
- **Pre-mortem:** F1: rutas feature-gated → mapa feature→path explícito; F2: script >1min → solo en fast gate si <10s, si no nightly.
- **Stop conditions:** si >30% falsos positivos → rediseñar como reporte.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Falsos positivos | mapa explícito + dry-run | 2 corridas rojas falsas |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation

### Task 12: DEF-04 — Naming freeze 0.7.0→1.0 (ADR + política de alias)
- **Fase:** F1

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs (+ firma owner — ADR Regla 5)
- **Archivos clave:** `docs/dev/architecture/adr/NNN_naming_freeze.md` (nuevo) · `docs/api/VERSIONING.md` · `docs/api/DEPRECATIONS.md` (coordina HARD-01)
- **Verificación real:** ✅ CÓDIGO-REAL — 9 artefactos a congelar (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory); API-01..09 completadas (base).
- **Gate Justificación:** congelar nombres AHORA (pre-usuarios) es gratis; después cuesta migraciones (R1).
- **Gate Result:** ✅ DO
- **Contrato:** "ADR con los 9 artefactos congelados + política de alias/deprecación (alias con fecha de remoción, nunca rename silencioso) Y firma owner (Regla 5) Y referencia desde VERSIONING.md"
- **Task file:** `docs/dev/tasks/DEF-04.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — firma owner Gate P; commit `d45deba5`; review fresco OK)
- **Cynefin:** 🟨 complicado — decisión con tradeoffs de naming.
- **Top 3 riesgos:** 1. ADR redactado por IA (Regla 5: lo articula el owner) · 2. alias sin fecha · 3. olvidar un artefacto.
- **Pre-mortem:** F1: IA escribe el ADR completo → entregar evidencia + owner articula; F2: alias eternos → fecha de remoción obligatoria.
- **Stop conditions:** owner no firma → queda como propuesta.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Rename post-1.0 | freeze ahora + DEPRECATIONS | post-release |
- **Uphill/Downhill:** ⬆️ 1 (firma) / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation

### Task 13: DEF-05 — North-star + success criteria (SPEC/VISION)
- **Fase:** F1

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs
- **Archivos clave:** `SPEC.md`/`VISION.md` · `docs/dev/strategy/ROADMAP.md` (métrica) · proxy (medición)
- **Verificación real:** ✅ CÓDIGO-REAL — North Star propuesta en P55: "agentes activos que recuperan una memoria con éxito en ventana de 7 días" (medible en proxy: sesiones MCP con put+search la misma semana); falta formalizar en SPEC/VISION + guardrails.
- **Gate Justificación:** toda métrica de F5 (ICP) y F6 (anuncio) cuelga de esto.
- **Gate Result:** ✅ DO
- **Contrato:** "SPEC/VISION declaran North Star + guardrails (0 hallazgo crítico seguridad, latencia) Y la métrica es medible con el proxy actual (comando/consulta documentada)"
- **Task file:** `docs/dev/tasks/DEF-05.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `58c9903c`; review fresco OK)
- **Cynefin:** 🟦 obvio — definición con evidencia.
- **Top 3 riesgos:** 1. métrica no medible hoy · 2. guardrails decorativos · 3. contradicción con DEF-01.
- **Pre-mortem:** F1: métrica sin query → documentar SQL/comando proxy; F2: sin baseline → registrar valor actual 0.
- **Stop conditions:** sin capacidad de medición → DEFER parcial (declarar y diferir).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Métrica no medible | verificar en proxy antes de declarar | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation

### Task 14: DEF-07 — Presupuesto de alcance: core-promise vs labs
- **Fase:** F1

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/dev/strategy/` · `SPEC.md` · `docs/EXPERIMENTAL_FEATURES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — P55: categorizar proxy/desktop/web/memory (labs vs core-promise) + decidir qué consume el presupuesto "1 comando, 0 config".
- **Gate Justificación:** evita que labs consuman el presupuesto de la promesa central (R4: DEF-07 pendiente).
- **Gate Result:** ✅ DO
- **Contrato:** "tabla categorizada (core-promise vs labs) por superficie Y regla de inversión documentada Y referencia desde SPEC"
- **Task file:** `docs/dev/tasks/DEF-07.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `fd50ea22`; review fresco OK)
- **Cynefin:** 🟦 obvio — clasificación.
- **Top 3 riesgos:** 1. clasificar por gusto · 2. contradecir DEF-01 · 3. no tocar roadmap.
- **Pre-mortem:** F1: sin criterio → usar North Star (DEF-05) como criterio; F2: sin consecuencias → regla de inversión explícita.
- **Stop conditions:** —
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟢×🟡 | Clasificación arbitraria | criterio = North Star | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 2 steps
- **DoD task:** contrato ✅ · task file sync · recitation

### Task 15: DEF-08 — Install SLO + telemetría opt-in + fallback visible
- **Fase:** F1

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠
- **Ruta:** vanta-docs (+ worker si toca código de fallback)
- **Archivos clave:** `docs/user/QUICKSTART.md` · `docs/dev/strategy/DISTRIBUTION.md` · (si aplica) código de fallback ORT/modelo
- **Verificación real:** ✅ CÓDIGO-REAL — P55: definir SLO de instalación (tiempo máx al primer recall, tasa de éxito), telemetría opt-in privacidad-first y fallback visible cuando ORT/modelo no cargan.
- **Gate Justificación:** FASE-A (stranger tests) mide exactamente esto; sin SLO no hay criterio de éxito del anuncio.
- **Gate Result:** ✅ DO
- **Contrato:** "SLO definido (tiempo al primer recall + tasa de éxito) con método de medición Y telemetría opt-in especificada (privacy-first, default off) Y fallback visible especificado (mensaje + docs)"
- **Task file:** `docs/dev/tasks/DEF-08.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `1a696766`; review fresco OK)
- **Cynefin:** 🟦 obvio — definición + doc.
- **Top 3 riesgos:** 1. telemetría que roce privacidad · 2. SLO no medible en FASE-A · 3. fallback sin UX.
- **Pre-mortem:** F1: telemetría default-on → PROHIBIDO (opt-in); F2: SLO vago → números.
- **Stop conditions:** si requiere código >1d → solo spec + FIND.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Privacidad telemetría | opt-in + revisión security | review P2-01 |
- **Uphill/Downhill:** ⬆️ 0 / ⬇️ 3 steps
- **DoD task:** contrato ✅ · task file sync · recitation

---

## F2–F6 — Bloques esenciales (se COMPLETAN al nivel F0/F1 al iniciar cada fase — REGLA)

> **REGLA DE COMPLETADO (obligatoria — el bloque esencial es una SEMILLA, no el contrato final):**
> al alcanzar cada fase F2–F6, **antes de claimear/ejecutar cualquier tarea de esa fase**, el orquestador DEBE completar cada bloque al MISMO nivel que F0/F1 — todos los campos, sin excepción:
>
> 1. **Paso 0 — Verificación de Realidad** (`prompts/plan.md`): archivos/símbolos existen, gap real confirmado, blast radius (`codegraph_explore` + `codebase-memory-mcp`).
> 2. **Shape Up** (¿problema correcto? ¿scope cabe en el appetite? ¿es ahora?) + **Pre-mortem** (2-3 modos de fallo) + **Stop conditions** (appetite/rabbit hole).
> 3. **Cynefin** + **Top 3 riesgos** + **Risk Register** (Prob×Impacto + respuesta + trigger).
> 4. **Contrato** verbatim verificable + **Uphill/Downhill** + **DoD multi-nivel**.
> 5. **Task file** creado/refinado en el DISCOVERY de la tarea (flujo canónico `/pipeline run` → `pipeline-full.md`).
>
> **Campos finales del bloque = idénticos a F0/F1:** Appetite · Esfuerzo · Prioridad · Ruta · Archivos clave · Verificación real · Gate Justificación · Gate Result · Contrato · Task file · Estado · Branch · Commit · Cynefin · Top 3 riesgos · Pre-mortem · Stop conditions · Risk Register · Uphill/Downhill · DoD task · Iteraciones · Notas.
> **Gate de fase:** una fase F2–F6 NO arranca hasta que TODOS sus bloques estén completados a este nivel (verificable con `rg` de campos por bloque en este plan). Registrar cada completado como `plan-adjust` en §Notas.

### Task 16: WIRE-02 — MCP: enforce de perfil en `tools/call` + default `agent` + fusión 85→~65 + fix doc `MCP.md:265`
- **Fase:** F2

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (enforce en `handle_tools_call` + fusión en `handle_tools_list`) · `vantadb-mcp/src/config.rs` (`McpProfile`: nuevo default `agent`) · `vantadb-mcp/tests/mcp_tests.rs` (meta-tests de perfil + smoke `tools/list`) · `docs/api/MCP.md` (tabla de perfiles, error de `tools/call`, conteos)
- **Verificación real:** ✅ CÓDIGO-REAL (Dep API-04 ✅ — aliases canonicalizados, MCP.md:536) — el perfil se aplica SOLO al listing: `profile_allowed_tools` (tools.rs:1023) se consume en `handle_tools_list` (tools.rs:1008-1017); `handle_tools_call` (tools.rs:1204) entra directo al `match` sin gate y su fall-through emite `Tool not found: {name}` pelado (tools.rs:2969) vs el error documentado `… (not in profile <profile>)` (MCP.md:265 — el `:254` del seed quedó desplazado). Default `Full` (config.rs:16-17,110) con **85 listadas** (meta-test mcp_tests.rs:4667; tabla MCP.md:198,231) + 2 aliases dispatchables no listados (tools.rs:27; MCP.md:214-223) → re-baseline del seed: 85 listadas → ~65 (el "87" incluía los 2 aliases; config.rs:11,14 stale). Redundancia dura: `code_*`×8 (tools.rs:1120-1128) ≈ `graph_traverse` con otro formato + triple solape de introspección `code_status` ≈ `operational_metrics()` (tools.rs:2081) ≈ `capabilities` (tools.rs:808) — el informe recomienda 87→~65 y default `agent` ~45 (VantaDB-Informe-Analisis-Completo.md:199,203,428). Blast radius: `handle_tools_call` ← `dispatch_request`/`mcp_proxy_handler` (vantadb-mcp/src/server.rs); `McpConfig` 75 callers (codegraph).
- **Gate Justificación:** el default `full` ≈ 23K tokens de overhead por cliente (informe:23) y el perfil es un claim documentado que HOY no se cumple (tool fuera de perfil sigue llamable) → enforce + default `agent` + fusión reduce el overhead y cierra la brecha doc↔código antes del anuncio (F6).
- **Gate Result:** ✅ DO
- **Contrato:** "`tools/call` de tool fuera de perfil → error real `Tool not found: <name> (not in profile <profile>)` (test) Y default `agent` con smoke `tools/list` ≤45 Y superficie fusionada con conteo real documentado en `MCP.md` (tabla + error, consistente con `handle_tools_list` y sus meta-tests) Y aliases API-04 siguen dispatchables"
- **Task file:** `docs/dev/tasks/WIRE-02.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `80f91bd8`; review fresco 2 rondas OK; 239/239)
- **Cynefin:** 🟨 complicado — gate de dispatch + cambio de default compatible + fusión sin romper aliases.
- **Top 3 riesgos:** 1. default `agent` rompe clientes existentes · 2. fusión de tools rompe dispatch/aliases · 3. enforcement bloquea smokes que llamaban tools fuera de perfil.
- **Pre-mortem:** F1: default agent sin escape → mantener `full` opt-in documentado (`VANTADB_MCP_PROFILE=full`) + notas de release; F2: fusión sin alias → retener dispatch-only aliases (precedente API-04) + meta-test listing↔dispatch; F3: doc y código divergen → `validate-docs-coverage` + meta-test de conteos.
- **Stop conditions:** si el enforcement rompe >2 smokes de clientes o la fusión a ~65 no cierra sin breaking de dispatch → entregar enforce + default + doc y diferir la fusión (anotar en Backlog).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | Cambio de default rompe compat | `full` dispatchable opt-in + notas de migración | review P2-01 |
  | 🟡×🟠 | Fusión rompe dispatch | aliases dispatch-only + meta-test listing vs dispatch | test rojo |
  | 🟢×🟡 | Doc diverge del código | validate-docs-coverage + meta-test de conteos | gate-docs |
- **Uphill/Downhill:** ⬆️ 0 (enforce + default + fusión no-breaking resueltos; delta a ~65 diferido → FIND-181) / ⬇️ 5 steps (5/5 ✅)
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 1 · **Notas:** seed decía `87→~65` y `MCP.md:254` — re-baselined post-API-04 (85 listadas; error hoy en `MCP.md:265`); consume informe §6-7. **Implementación 2026-09-27:** enforce real del perfil en `tools/call` (error documentado, test), default `agent` (37 ≤45, smoke), fusión no-breaking 85→79 (−6 `code_*` absorbed dispatch-only, precedente API-04), aliases API-04 dispatchables (test); meta-tests de conteos 79/36/20/37 + `validate-docs-coverage` exit 0; delta a ~65 → FIND-181 (stop condition aplicada).

### Task 17: WIRE-03 — `query_sparse` + text-only en 3 bindings + filtros avanzados (`$and`/`$or`, range/datetime)
- **Fase:** F2

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/src/lib.rs` + `vantadb-python/vantadb_py/{__init__,vantadb_py}.pyi` (stubs) · `vantadb-node/src/lib.rs` + `vantadb-node/index.d.ts` · `vantadb-ts/src/{types.ts,guards.ts,native.ts,vantadb.ts}` · `src/sdk/serialization/vector_types.rs` + `src/sdk/types/record.rs` (contrato core) · tests por binding (`vantadb-python/tests/`, `vantadb-node/tests/`, `vantadb-ts/src/__tests__/`)
- **Verificación real:** ✅ CÓDIGO-REAL (Dep API-02 ✅) — el core soporta sparse en los paths de search (`MemorySearchRequest.query_sparse`, vector_types.rs:15-19; hybrid.rs:36-39; debug_ops.rs:216-374; explain.rs:28-180) pero los bindings lo hardcodean `None`: Py lib.rs:1240,1331,2255,2404; Node `parse_search_request` lib.rs:838. TS no lo expone en `SearchRequest` (types.ts:141-155) y `buildSearchRequestBase` rechaza `query_vector: []` ("must be a non-empty array", guards.ts:219-224) → text-only imposible por la puerta principal. Filtros: Py convierte dict→metadata legacy por igualdad (`py_dict_to_metadata`, lib.rs:1241) sin `FilterOp`; el core es AND-only plano (`MemoryFilter = Vec<MemoryFilterItem>`, record.rs:54-55) con ops Gt/Lt/Gte/Lte + `Value::DateTime` ya disponibles (record.rs:11-25). Blast radius: `buildSearchRequestBase` compartido por los paths nativo y wasm de TS (native.ts:331-340; vantadb.ts:580-596); stubs Py sujetos a `test_stub_drift.py`.
- **Gate Justificación:** `query_sparse` existe en core (y en MCP/HTTP con su shape) y NO en ningún binding — la paridad Py/TS/Node es requisito del gate de fase F2 ("paridad bindings") y alimenta la matriz de capacidades de ICP-03 (F5).
- **Gate Result:** ✅ DO
- **Contrato:** "`query_sparse` expuesto en Py/TS/Node con test de roundtrip Y text-only (`query_vector: []` = solo-BM25) por la puerta principal en los 3 Y filtros avanzados (`$and`/`$or`, range/datetime) equivalentes py↔js con tests Y stubs/d.ts sincronizados (`test_stub_drift` verde)"
- **Task file:** `docs/dev/tasks/WIRE-03.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `81095b88`; review fresco 2 rondas OK; py 159 · node 41 · ts 323)
- **Cynefin:** 🟨 complicado — paridad cross-binding + validación compartida en TS.
- **Top 3 riesgos:** 1. cambiar la validación TS rompe flujos existentes (contrato "non-empty") · 2. semántica OR/range divergente entre bindings · 3. stubs `.pyi`/`.d.ts` desincronizados.
- **Pre-mortem:** F1: TS acepta `[]` sin distinguir intención → exigir `text_query` presente cuando el vector está vacío (error claro); F2: `$or` implementado distinto en py vs js → definir la forma canónica en el core y que los 3 bindings la serialicen; F3: drift de stubs → `test_stub_drift.py` + check de d.ts en CI.
- **Stop conditions:** si `$and`/`$or` exige cambiar el modelo core más allá del slice → dividir: primero sparse+text-only (mínimo vendible) y OR/range diferido con FIND; rabbit hole: reescribir validaciones de TS → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Validación TS rompe clientes | rechazar solo si vector vacío Y sin `text_query` | test d5a-validation |
  | 🟡×🟡 | Divergencia OR/range | forma canónica única desde core + matriz de paridad | review P2-01 |
  | 🟢×🟡 | Stub drift | `test_stub_drift.py` + check d.ts | CI binding |
- **Uphill/Downhill:** ⬆️ 1 (forma canónica de filtros en 3 bindings) / ⬇️ 7 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** seed "matriz de paridad 3 bindings con tests de sparse y text-only" (Backlog:968) se mantiene; WASM fuera de scope — coordinar `SearchRequestInput` (wasm d.ts) por la vía TS.

### Task 18: WIRE-04 — TTL superficie completa (server HTTP + default por colección + sweeper)
- **Fase:** F2

- **Appetite:** max 2d · **Esfuerzo:** 🟢 1-2d · **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `src/server/handlers.rs` + `src/server/router.rs` (superficie HTTP; `vantadb-server/` es entrypoint delgado, lib.rs:3-7) · `src/sdk/api/memory.rs` (`put_one`/`purge_expired`) · `src/gc.rs` (sweeper) · `src/sdk/types/record.rs` (`ttl_ms`) · config de colección/namespace (default TTL — nuevo) · `docs/api/HTTP_API.md` · tests (inline en `src/gc.rs` y `src/sdk/api.rs`)
- **Verificación real:** ✅ CÓDIGO-REAL (⚠️ Dep MGR-09 🆕 PENDIENTE — Backlog:815, research de retención; el slice TTL no la requiere; su spec informa default/decay futuro) — premisa del seed AJUSTADA: el param HTTP YA existe — `records_put`/`records_put_batch` toman `MemoryInput` (handlers.rs:250,261) que incluye `ttl_ms` (record.rs:74-77; doc HTTP_API.md:228) y `put_one` lo resuelve a `expires_at_ms` (memory.rs:88); `get` filtra expirados (memory.rs:342). Gaps reales: (1) NO existe default TTL por colección (`rg default_ttl|collection_ttl|ttl_default` = 0 hits — Qdrant/Weaviate/Milvus tampoco lo tienen: diferenciador, Backlog:969); (2) NO hay sweeper background de memoria — `purge_expired` es manual (memory.rs:587-698; endpoint router.rs:197 + handlers.rs:1151) y SÍ purga índices derivados + text index (memory.rs:689-698), pero `GcWorker::sweep` (gc.rs:34) solo se registra para threads (thread.rs:205,252) y no se invoca en producción (solo tests; comentario gc.rs:33 "runs in a tokio::spawn loop" sin spawn real). Blast radius: `MemoryInput` compartido HTTP/SDK/MCP (put path); `purge_expired` ← maintenance endpoint.
- **Gate Justificación:** la retención hoy depende de llamadas manuales a `purge_expired`; exponer default + agendarla hace que la promesa TTL no se pudra (costo barato: la purga física completa ya existe) y alimenta la narrativa privacy/local-first de ICP-02 (F5).
- **Gate Result:** ✅ DO
- **Contrato:** "TTL HTTP verificado con test E2E (put `ttl_ms` → expira → `get` None / `purge_expired` ≥1) Y default TTL por colección implementado (config + fallback en put, solo writes nuevos) y documentado en `HTTP_API.md` Y sweeper background invocado que purga memoria/índices con test de expiración física (nodo ausente del índice, no solo filtrado en lectura)"
- **Task file:** `docs/dev/tasks/WIRE-04.md`
- **Estado:** ✅ COMPLETED (2026-09-27 — commit `54261e96`; review fresco 2 rondas OK; TTL 3/3 + snapshot verde)
- **Cynefin:** 🟨 complicado — scheduler async + defaults de retención sin degradar close.
- **Top 3 riesgos:** 1. sweeper que interfiere con close/shutdown · 2. default por colección cambia semántica de puts existentes · 3. purga física que deja índices inconsistentes.
- **Pre-mortem:** F1: sweeper sin cancelación → shutdown token + test de close/join; F2: default retroactivo → solo writes nuevos (sin backfill) y opt-in por colección; F3: purga que no limpia HNSW/text → reutilizar `purge_expired` (ya purga derivados) + test que verifica índice post-purga.
- **Stop conditions:** si el sweeper exige refactor del lifecycle (engine + tokio) >1d → entregar default + E2E y dejar el sweep como FIND con diseño; rabbit hole: GC distribuido → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Sweeper vs close | shutdown token + test de drain | test de close |
  | 🟡×🟡 | Default cambia semántica | solo writes nuevos + opt-in por colección | review P2-01 |
  | 🟢×🟠 | Índices inconsistentes | reutilizar `purge_expired` + test post-purga | test expiración |
- **Uphill/Downhill:** ⬆️ 1 (sweeper sin degradar shutdown) / ⬇️ 5 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** ajuste verificado vs seed ("TTL no en HTTP" era stale — el param ya está, falta default+sweeper); MGR-09 anotada como dep de la parte semántica (`audit_vigency`/decay), no del slice.

### Task 19: WIRE-05 — Entity linking determinista + boost multi-señal en RRF (Fellegi-Sunter + embeddings)
- **Fase:** F2

- **Appetite:** max 4d · **Esfuerzo:** 🟡 3-4d · **Prioridad:** 🔴
- **Ruta:** vanta-engine
- **Archivos clave:** `src/entity/{mod.rs,checker.rs,scene.rs}` + `src/entity/linking.rs` (nuevo: matching/canonicalización) · `src/sdk/search/fusion.rs` (boost opt-in en `fuse_rrf*`) · `src/sdk/search/{hybrid.rs,explain.rs,mod.rs}` (callers) · `src/graph.rs` (relaciones) · tests (`src/entity/tests.rs`, `src/sdk/search/tests.rs`)
- **Verificación real:** ✅ CÓDIGO-REAL (⚠️ Dep MGR-05 🆕 PENDIENTE — Backlog:811, research+spec 3-4d; el slice determinista no requiere el LLM-juez, que queda como paso 2/v1.0) — NO existe linking/ER: `src/entity/` = `EntityStore` genérico + `checker` (permisos) + `scene` (mod.rs:1-34); `rg mark_duplicate|auto_resolve_entities|fellegi` = 0 hits. La fusión RRF existe y es determinista (`fuse_rrf` fusion.rs:72-91; `fuse_rrf_many` :97; tie-break por key/node_id :71) y NO conoce entidades → el boost es extensión aditiva; blast radius 12 callers: `hybrid_search` (hybrid.rs), `search_impl` (mod.rs), `explain_memory_search` (explain.rs) + tests (trace_path). Forma requerida: opt-in, reversible, con proveniencia (Backlog:970); la medición del boost queda para el harness VER-08 (F5 — dep cross-fase anotada).
- **Gate Justificación:** paridad Mem0 sin LLM (matching multi-señal determinista, ms, coste 0) refuerza el diferenciador "determinista y auditable" del anuncio; el RRF ya está en producción y el boost aditivo mantiene riesgo bajo.
- **Gate Result:** ✅ DO
- **Contrato:** "matching multi-señal determinista (Fellegi-Sunter + embeddings, sin LLM-juez) con tests (mismos inputs → mismo score) Y boost de entidades en `fuse_rrf*` opt-in, reversible y con proveniencia Y suites `sdk::search`/`entity` verdes (boost OFF byte-idéntico al ranking actual)"
- **Task file:** `docs/dev/tasks/WIRE-05.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `3b2521dd`; review fresco OK)
- **Cynefin:** 🟨 complicado — matching multi-señal determinista + integración en fusión existente.
- **Top 3 riesgos:** 1. falsos positivos de matching (merge indebido) · 2. boost que rompe determinismo/orden del RRF · 3. MGR-05 no llega y el diseño se sesga.
- **Pre-mortem:** F1: matching agresivo sin umbral → Fellegi-Sunter con umbral conservador y `mark_duplicate` manual como paso 1; F2: boost no reversible → proveniencia por hit + test OFF byte-idéntico; F3: dep pendiente → mini-discovery con informe + estrategia como fuente determinista; MGR-05 solo para el LLM-juez.
- **Stop conditions:** si el matching exige LLM en el camino crítico → recortar a Fellegi-Sunter + embeddings; si el boost no puede ser byte-idéntico con OFF → rediseñar antes de tocar `fuse_rrf`.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Merge indebido (falso positivo) | umbral conservador + reversible + proveniencia | 2 merges erróneos en tests |
  | 🟡×🟠 | Rompe determinismo RRF | OFF byte-idéntico (test) + ON orden estable | test determinismo |
  | 🟢×🟡 | Dep MGR-05 ausente | mini-discovery determinista; LLM-juez fuera de scope | discovery |
- **Uphill/Downhill:** ⬆️ 2 (diseño de matching sin MGR-05 + reversibilidad) / ⬇️ 6 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** medición de boost en harness = VER-08 (F5); seed citaba `src/sdk/search/fusion.rs` — verificado, más `hybrid.rs`/`explain.rs` como callers.

### Task 20: WIRE-06 — Batching productizado (`insert_lock` → segmentos appendables)
- **Fase:** F2

- **Appetite:** max 1sem · **Esfuerzo:** 🔴 1sem · **Prioridad:** 🔴
- **Ruta:** vanta-engine
- **Archivos clave:** `src/storage/engine/insert.rs` (path de inserción + auto-flush) · `src/wal.rs` (fsync/group-commit) · `src/config.rs` (`insert_lock_timeout_ms`) · `benches/ingestion_concurrent.rs` (infra A/B find61) + `benches/wal_throughput.rs` · `docs/dev/architecture/adr/ADR-0038-wal-fsync-batching-opt-in.md` (spec) · `docs/user/operations/BENCHMARKS.md` §13 (before/after)
- **Verificación real:** ✅ CÓDIGO-REAL (Dep FUT-12-spec ✅ — spec cerrada: `docs/dev/tasks/FUT-12-spec.md` + ADR-0038 Proposed spec-only, CHANGELOG:1650) — techo confirmado: `insert_lock` global (insert.rs:208 "WAL + queue" bajo un guard; delete.rs:42,217; timeout config.rs:238) + convoy lock+HNSW: `Never* p1/w4 −37%` CON fsync apagado (core-engine.md:555) y fsync ≈0.16ms ≈1.5% del op (ibíd.) → el término dominante es lock+serial, no fsync. Prototipo medido: N=32 **1016 ops/s / 9.1×** vs 111.5 §13 (BENCHMARKS.md:561,612,644; gate ≥2× cumple) — cerrado SIN slice con bench-infra A/B como legado (core-engine.md:557): la productización es este task. `ingestion.rs:42` documenta el serial path (single worker por el lock). Blast radius: insert path global (SDK/MCP/server/ingestion escriben por el mismo lock).
- **Gate Justificación:** la ingesta batch es el claim de performance con la evidencia más fuerte del repo (9.1× prototipado, gate A/B listo) y el techo es arquitectónico (lock global) — productizarlo con Regla 9 (before/after) convierte un prototipo en capacidad medida antes del head-to-head (VER-09, F6).
- **Gate Result:** ✅ DO
- **Contrato:** "≥5× ingesta sostenida vs baseline (bench before/after commiteado, Regla 9) Y p99 sin regresión >15% Y tests de integridad (WAL/durabilidad/reopen, batch atómico) verdes Y bench nightly activo con gate anti-regresión"
- **Task file:** `docs/dev/tasks/WIRE-06.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `5e8e8879`; review adversarial OK; pareada 5.92×)
- **Cynefin:** 🟨 complicado — segmentos appendables + compactación background + política de durabilidad.
- **Top 3 riesgos:** 1. durabilidad degradada por batch (ventana de pérdida) · 2. p99 >15% por compactación background · 3. segmentos que fragmentan el índice/HNSW.
- **Pre-mortem:** F1: group-commit sin política explícita → ADR-0038 manda (opt-in, ventana declarada); F2: compactación que compite con writers → background con presupuesto + métrica; F3: bench que no reproduce → usar harness find61 existente (BATCH=400, DIM=16, perfil bench) como control.
- **Stop conditions:** si la pata segmentos es irreducible a 1sem → entregar group-commit/fsync-batching opt-in (ADR-0038) con ≥5× y diferir appendables con diseño + FIND; rabbit hole: reescribir HNSW → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Ventana de pérdida (durabilidad) | ADR-0038: política de sync explícita + opt-in | review P2-01 |
  | 🟡×🟠 | p99 regresa | presupuesto de compactación + bench nightly | 2 corridas rojas |
  | 🟢×🟡 | Fragmentación de índice | compactación + tests de integridad post-reopen | test reopen |
- **Uphill/Downhill:** ⬆️ 3 (segmentos + política de durabilidad + compactación) / ⬇️ 7 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** consume FUT-12-spec + ADR-0038 (spec-only) y el bench find61 como infra A/B; bench nightly coordina con el gate perf existente (no tocar `perf-bench.yml` sin dueño).

### Task 21: WIRE-07 — Refactors: crate `ffi-core` (OpGate×3), trait-split storage↔index, desacoplar `server→cli`
- **Fase:** F2

- **Appetite:** max 1sem · **Esfuerzo:** 🔴 1sem · **Prioridad:** 🟠
- **Ruta:** vanta-arch
- **Archivos clave:** `vantadb-ffi-core/` (nuevo crate: OpGate + clamps MAX_* + mapeo de errores) · `vantadb-node/src/lib.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs` (consumidores) · `Cargo.toml` (feature `server = ["cli", …]` → desacople) · `docs/dev/architecture/BOUNDARIES.md` (§3/§5 — bookkeeping) · `src/index_port.rs` + `src/index/port_impl.rs` (evidencia F3X)
- **Verificación real:** ✅ CÓDIGO-REAL — (a) OpGate triplicado literal: node lib.rs:661/670/732, py lib.rs:97/106/170, wasm lib.rs:435/444/511; guards MAX_* ya centralizados en core (config.rs:40-56) pero los clamps se duplican (node lib.rs:34-42; py lib.rs:41-48) — el informe propone `vantadb-ffi-core` (2-3d; strategy:153). (b) **trait-split YA EJECUTADO**: F3X-impl ✅ (commit `13f0f729`, ADR-0042 accepted; `src/index_port.rs` + `src/index/port_impl.rs`) — los imports storage↔index restantes son test-only (archive.rs:204-209/464-470; index/core.rs:151-156) → la pata del seed pasa a BOOKKEEPING: `BOUNDARIES.md` §3 (L117-160) y §5 ítem 5 (L205-206) siguen listando el ciclo como deuda abierta y hay que actualizarlos con la evidencia. (c) acople `server→cli` vigente: `Cargo.toml:177-185` (`server = ["cli", …]` arrastra clap/indicatif/anyhow al binario HTTP; strategy:68). Re-estimación: 🔴 1sem (era 1-2sem; trait-split ya pagado).
- **Gate Justificación:** refactors de frontera habilitan F2/F3 sin cambiar comportamiento — ffi-core elimina la triplicación con UNA semántica de drain (riesgo real de divergencia node/py/wasm); el desacople server→cli reduce el binario HTTP y la clase de builds silenciosos con backend equivocado (strategy:68); el bookkeeping cierra el gate §5 de BOUNDARIES con evidencia ya pagada.
- **Gate Result:** ✅ DO
- **Contrato:** "crate `vantadb-ffi-core` consumido por node/py/wasm (`rg 'struct OpGate'` = 1 en el workspace) Y `server` sin feature `cli` (grafo sin arrastre; `cargo check` por-crate verde: vantadb, vantadb-node, vantadb-python, vantadb-wasm, vantadb-server) Y `BOUNDARIES.md` §3/§5 actualizados con evidencia F3X `13f0f729` Y suites verdes sin cambio de comportamiento"
- **Task file:** `docs/dev/tasks/WIRE-07.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commits `ce50876e`..`8f4a0c6d` A-F; review adversarial OK)
- **Cynefin:** 🟨 complicado — frontera de crate + feature-split con suites cross-crate.
- **Top 3 riesgos:** 1. ffi-core con dependencias que contaminan bindings · 2. desacople que rompe el surface `cli_server` público · 3. refactor con cambio de comportamiento inadvertido.
- **Pre-mortem:** F1: crate FFI con deps pesadas → leaf std-only (perfil `index_port.rs`) + `cargo tree` check; F2: surface roto → preservar re-exports (`src/server/mod.rs:16,34` guarda `crate::cli_server::*`) + semver-checks; F3: regresión silenciosa → un commit por refactor (OS del Backlog) para bisectar.
- **Stop conditions:** si `server→cli` no se desacopla sin mover código compartido → DEFER parcial documentado (ffi-core primero; FIND para server→cli); rabbit hole: mudanzas cosméticas de módulos (BND-08: layout ≠ architecture) → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | FFI core contamina bindings | leaf std-only + cargo tree review | review P2-01 |
  | 🟡×🟡 | Surface `cli_server` roto | re-exports preservados + semver-checks | CI semver |
  | 🟢×🟡 | Regresión silenciosa | 1 commit por refactor + suites por crate | verify_changed |
- **Uphill/Downhill:** ⬆️ 2 (frontera ffi-core + feature-split) / ⬇️ 6 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** ajuste verificado: trait-split ya pagado (`13f0f729`, ADR-0042) → solo bookkeeping en BOUNDARIES; re-estimación 1-2sem→1sem; "deuda P2" del seed = triplicación FFI + feature-coupling.

### Task 22: WIRE-08 — Range/group_by + cursor con resume + RRF en CBO + rewriting + MMR
- **Fase:** F2

- **Appetite:** max 5d · **Esfuerzo:** 🟡 3-5d · **Prioridad:** 🟠
- **Ruta:** vanta-engine
- **Archivos clave:** `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest`) · `src/sdk/search/{mod,fusion,vector,lexical}.rs` · `src/planner.rs` (RRF como operador del CBO) · `src/search_profile.rs` (`RRF_K`=60) · `src/sdk/types/record.rs` (cursor de list como precedente) · tests (`src/sdk/search/tests.rs`, planner) · `docs/api/` (paridad Milvus/Qdrant)
- **Verificación real:** ✅ CÓDIGO-REAL (Dep API-06 ✅; ⚠️ Dep MGR-16 🆕 PENDIENTE — Backlog:842, spec de RRF-CBO/rewriting/MMR; consumirla o DEFER anotado por pata) — `MemorySearchRequest` (vector_types.rs:10-38) NO tiene cursor/range/group_by; `rg group_by|range_filter|radius|mmr` en `src/` = 0 código (solo comentarios de scores.rs:37,59 sobre adapters que duplican relevancia MMR) → gaps reales. Cursor existe SOLO en list (`MemoryListOptions.cursor`, record.rs:155; namespaces.rs:47-60) → "extender el de list" es viable como patrón. RRF hoy SOLO SDK: helpers en `sdk/search/fusion.rs` (movidos adrede para romper `sdk→planner`, fusion.rs:1-8) y el CBO no fusiona ("solo el path SDK lo usa", planner.rs:180-181 — la línea `planner.rs:181` del seed, verificada). MMR client-side ya existe en adapters (langchain vectorstore.py:149-244; llamaindex :238) → centralizar en core. Blast radius: `MemorySearchRequest` lo consumen SDK/server/MCP/bindings; el CBO (`planner.rs:190-208`) queda intacto si el operador es aditivo.
- **Gate Justificación:** radius/range/group_by y cursor con resume son las capacidades que las tablas comparativas marcan ausentes (paridad Milvus/Qdrant); RRF-CBO cierra la divergencia IQL↔SDK y MMR centralizado elimina la duplicación en adapters — todo antes del head-to-head (VER-09, F6) que las mide.
- **Gate Result:** ✅ DO
- **Contrato:** "range/radius + group_by + cursor con resume en search implementados con tests (resume estable con writes intercalados y page-full/last-page) Y paridad Milvus/Qdrant documentada en tabla capacidad×capacidad Y RRF-CBO/rewriting/MMR implementadas o DEFER explícito por pata en el task file (motivo + dueño) Y suites search/planner verdes"
- **Task file:** `docs/dev/tasks/WIRE-08.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `8ac202a1`; review adversarial 2 rondas OK)
- **Cynefin:** 🟨 complicado — operadores nuevos en planner + semántica de cursor/MMR.
- **Top 3 riesgos:** 1. cursor no-resumible (estado no serializable) · 2. RRF en CBO que rompe queries IQL existentes · 3. scope creep (5 patas en 3-5d).
- **Pre-mortem:** F1: cursor opaco no estable entre requests → snapshot de plan + offset determinista (test de resume con writes intercalados); F2: RRF-CBO aditivo → flag/profile + paridad IQL↔SDK test; F3: 5 patas no cierran → orden por valor (range/group_by → cursor → MMR → rewriting → RRF-CBO) con DEFER posible por pata.
- **Stop conditions:** si una pata excede y compromete el resto → DEFER explícito por pata (motivo/dueño en task file); rabbit hole: reescribir el planner → NO (extensión por operador registrado, planner.rs:169-173).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Cursor no resumible | snapshot de plan + test con writes intercalados | test resume |
  | 🟡×🔴 | RRF-CBO rompe IQL | flag/profile + paridad IQL↔SDK | suite planner roja |
  | 🟡×🟡 | Scope creep 5 patas | orden por valor + DEFER por pata | review P2-01 |
- **Uphill/Downhill:** ⬆️ 2 (RRF en CBO + opciones de planner) / ⬇️ 8 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** dep MGR-16 pendiente → DEFER por pata permitido; MMR adapter-side existe como referencia de semántica (langchain/llamaindex).

### Task 23: MGR-10 — research-doc: bitemporalidad (dim 5)
- **Fase:** F3
- **Dep:** — (Backlog:839 citaba MGR-01; fuera del plan — research lane post-1.0, L769)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-arch
- **Archivos clave:** `docs/dev/research/mgr-10-bitemporalidad.md` (nuevo — research-doc; convención de `docs/dev/research/mgr-19-benchmarks-baseline-suites.md`) · `src/sdk/types/record.rs` (`MemoryRecord`: `created_at_ms`/`updated_at_ms` :112-115, `superseded_by`/`superseded_at_ms` :129-137; `MemoryInput` :59-78) · `src/sdk/version_history.rs` (historial por versión :1-14; backing de `get_version`/`versions` :219-229) · `src/schema.rs` (`CURRENT_SCHEMA_VERSION` :11 + header `.vanta.schema` :165-179) · `src/sdk/serialization/mod.rs` (`EXPORT_SCHEMA_VERSION` :35; export/import :498-529) · `docs/dev/architecture/adr/ADR-0028-core-decay-supersession.md` (precedente de semántica) · `docs/dev/Backlog.md` (fila MGR-10 :839)
- **Verificación real:** ✅ CÓDIGO-REAL — **gap confirmado**: 0 hits de dominio de `valid_at`/`valid_from`/`bitemporal`/`as_of` en `src/` (rg; solo comentarios "point-in-time" de snapshots/métricas: `src/storage/engine/txn.rs:227`, `src/backends/rocksdb_backend.rs:350`, `src/metrics/core/mod.rs:622`). **Infra base verificada**: (1) *transaction-time parcial* — snapshots post-commit por versión en `BackendPartition::Versions` (`src/sdk/version_history.rs:1-14`) con API pública `get_version`/`versions` (`src/sdk/api/memory.rs:409-432`, VS-CORE-07) y cap de retención (`version_history_limit`); (2) invalidación key→key vía `superseded_by`/`superseded_at_ms` (ADR-0028, `record.rs:129-137`) — NO es ventana de validez; (3) schema header versionado con TooOld/TooNew (`src/schema.rs:11,84-98`) + comando `migrate` (`src/cli_handlers/migrate.rs:175-279`); (4) export/import con `schema_version` propio (`serialization/mod.rs:35,498-529`). **Formato oculto a migrar**: mirror postcard `SnapshotRecord` (`version_history.rs:76-90`, From :112-148) omite `superseded_*` — la migración lo alcanza. Edge sin validez temporal (Backlog:946 → SCH-09). Blast radius: `MemoryRecord` consumido por SDK/server/MCP/bindings + export.
- **Gate Justificación:** es la dim 5 del corte F3: define el modelo temporal del que dependen el ADR de SCH-01 y el schema v2 de SCH-02; sin diseño valid vs transaction no hay queries auditables "qué era verdad en T" (evidence-before-belief, Backlog:839) ni base para SCH-03.
- **Gate Result:** ✅ DO
- **Contrato:** "research-doc cerrado con modelo valid-time vs transaction-time, tradeoffs (append-only vs invalidación + storage del historial) y plan de migración/backfill determinista, listo para SCH-01"
- **Task file:** `docs/dev/tasks/MGR-10.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `b6614e1e`; review fresco OK)
- **Cynefin:** 🟨 complicado — modelo temporal de dos ejes + tradeoffs de storage + migración única.
- **Top 3 riesgos:** 1. modelo sobredimensionado (bitemporal completo Snodgrass en 0.8.0) · 2. backfill no determinista del historial · 3. edge-bitemporal (SCH-09) fuera del diseño → segundo breaking.
- **Pre-mortem:** F1: research-doc que cuela v1.0 (índices temporales/MVCC) → sección explícita "0.8.0 vs v1.0" + Cierre MGR; F2: diseño conceptual desconectado del storage real → tradeoffs mapeados a `version_history`/KV/export con comandos verificables; F3: se redacta sin el insumo dim 5 → citar fuente (Backlog MGR-10 + Notion dim 5) y refs externas (Snodgrass; Zep/Graphiti arXiv 2501.13956).
- **Stop conditions:** si el diseño exige rediseño del engine o segundo breaking sobre edges → acotar a records (valid/invalid + time-travel por key) y diferir con FIND; rabbit hole: implementar MVCC/índices temporales → NO (research-doc).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Modelo sobredimensionado | sección "0.8.0 vs v1.0" + revisión en SCH-01 | review P2-01 |
  | 🟡×🔴 | Backfill no determinista | plan de migración determinista + validación SCH-02/SCH-06 | review SCH-02 |
  | 🟢×🟠 | Edge-bitemporal fuera del diseño | SCH-09 dentro de la migración única (Backlog:946) o FIND | discovery SCH-01 |
- **Uphill/Downhill:** ⬆️ 1 (modelo valid/transaction acotado a la migración única) / ⬇️ 4 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog MGR-10 = 🔴 4-5d (investigación + ADR, :839) → acotado a research-doc 2-3d (el ADR formal se consolida en SCH-01, Task 26). Fuente: P49 dim 5 (Backlog:814). Task file se crea en DISCOVERY.

### Task 24: MGR-12 — research-doc: confianza (dim 6)
- **Fase:** F3
- **Dep:** — (Backlog:846 citaba MGR-01; fuera del plan — research lane post-1.0, L769)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-arch
- **Archivos clave:** `docs/dev/research/mgr-12-confianza.md` (nuevo) · `src/sdk/types/record.rs` (destino del esquema: `MemoryRecord` :102-138 sin score/clase de confianza) · `vantadb-mcp/src/axioms.rs` (Iron Axioms hardcodeadas :22-30) · `src/node/unified.rs` (`confidence_score: f32` :41-42; default `0.5` :92) · `src/config.rs` + `src/eviction.rs` (`eviction_weight_confidence` :285 / :33-42) · `src/executor.rs` (filtro low-confidence `<0.4` :218-234) · `src/utils/confidence_metrics.rs` (`OriginCollisionTracker` :8-63) · `docs/dev/Backlog.md` (fila MGR-12 :846)
- **Verificación real:** ✅ CÓDIGO-REAL — **gap confirmado**: `MemoryRecord` NO tiene score/clase de confianza (`asserted`/`derived`) ni `last_validated` (`record.rs:102-138`); `rg 'asserted|derived'` en `vantadb-mcp/src/axioms.rs` = 0 — axioms.rs son 4 Iron Axioms hardcodeadas (JSON de reglas, :22-30), no jerarquía de procedencia. **Infra base verificada**: confianza ya existe a nivel NODO — `UnifiedNode.confidence_score: f32` (:41-42; default `0.5` :92; persistida en disk header `src/node/disk.rs:18-19`), pondera eviction (`eviction_weight_confidence=2.0`, `src/config.rs:285`; `src/eviction.rs:33-42`), filtra `SemanticSummary` <0.4 en executor (:218-234) y se restaura a nodo en `src/sdk/api/graph.rs:231`; multi-agente: `OriginCollisionTracker` (`src/utils/confidence_metrics.rs:8-63`). Gap: jerarquía asserted/derived + reglas de derivación + calibración POR REGISTRO (hoy: metadata libre como escape).
- **Gate Justificación:** es la dim 6 del corte F3: SCH-04 (0.8.0) consume esta spec para scores consumibles y SCH-05 para trust-aware retrieval/abstención; el nodo ya puntúa confianza (eviction/executor) → el diseño debe unificar record↔nodo sin divergencia antes del corte.
- **Gate Result:** ✅ DO
- **Contrato:** "research-doc cerrado con modelo de confianza asserted/derived (scores por registro + reglas de derivación + `last_validated` + calibración básica), tradeoffs y mapeo al `confidence_score` de nodo, listo para SCH-01"
- **Task file:** `docs/dev/tasks/MGR-12.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `50df4efd`; review fresco OK)
- **Cynefin:** 🟨 complicado — taxonomía de procedencia + calibración + unificación record↔nodo.
- **Top 3 riesgos:** 1. scores no calibrados (números decorativos) · 2. doble fuente de confianza (record vs nodo) divergente · 3. scope creep a jueces/grounding (v1.0).
- **Pre-mortem:** F1: "asserted vs derived" sin definición operacional → taxonomía con ejemplos + reglas de derivación verificables; F2: grounding con jueces LLM dentro del slice → excluido explícito (derivación completa = v1.0, Backlog:934); F3: record y nodo divergen → semántica canónica en record + mapeo explícito (`graph.rs:231`).
- **Stop conditions:** si exige calibración empírica → spec con fórmula + calibración diferida a VER-08 (F5, dep cross-fase); rabbit hole: FACTS/FaithJudge → NO (v1.0).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Scores no calibrados | semántica definida + límites documentados + calibración VER-08 | review P2-01 |
  | 🟡×🟡 | Record vs nodo divergentes | canónico en record + mapeo a `confidence_score` (graph.rs:231) | discovery SCH-02 |
  | 🟢×🟠 | Scope creep a jueces | excluido del slice + FIND con dueño | review SCH-01 |
- **Uphill/Downhill:** ⬆️ 1 (taxonomía assert/derive operacional sin jueces) / ⬇️ 4 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog MGR-12 = 🔴 4d (investigación + esquema + ADR, :846) → acotado a research-doc 2-3d; esquema final en ADR SCH-01, implementación SCH-02/SCH-04. Consumidores: SCH-04 (scores), SCH-05 (trust-aware), MGR-18 (post-1.0, Backlog:867).

### Task 25: MGR-13 — research-doc: cuarentena
- **Fase:** F3
- **Dep:** — (Backlog:847 citaba MGR-04 + MGR-12; MGR-04 fuera del plan → threat model propio; MGR-12 co-batch en F3)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-arch
- **Archivos clave:** `docs/dev/research/mgr-13-cuarentena.md` (nuevo) · `vanta-memory/src/core/dream/mod.rs` (consolidación: `promote` stub :24; `DreamRun` :153-181; originals nunca replaced :179) · `src/server/middleware.rs` (superficie de escritura HTTP :26-81) · `src/wal.rs` (precedente léxico `quarantine_corrupt_tail` :611-642 — NO es estado de contenido) · `src/sdk/types/record.rs` (destino del estado :102-138) · `docs/dev/Backlog.md` (fila MGR-13 :847)
- **Verificación real:** ✅ CÓDIGO-REAL — **gap confirmado**: no existe cuarentena de registros — `rg 'quarantine|quarantined'` en `src/` = solo salvage de WAL corrupto (`src/wal.rs:611-642`; `src/wal_sharded.rs:259,287-313`: trunca a prefijo coherente + respalda el tail), semántica de almacenamiento, NO estado de contenido; `MemoryRecord` sin field de estado/trust (:102-138); `rg 'tainted'` en src = 0 (los `trusted_proxies` de `config.rs:246` son de red, no de contenido). **Superficies de escritura a threat-modelar**: API HTTP (auth middleware `src/server/middleware.rs:26-81`), dream/consolidación (`vanta-memory/src/core/dream/mod.rs` — escribe `dream/<s>/<run_id>`, promote stub :24), import JSONL (`src/sdk/serialization/mod.rs:498-529`). Threat refs write-time: AgentPoison (NeurIPS 2024) / MINJA (arXiv 2503.03704) / OWASP ASI06.
- **Gate Justificación:** dim 6/AM10 del corte F3: SCH-05 (0.8.0) implementa cuarentena+abstención y su test de contención exige esta spec; sin threat model write-time el estado `quarantined` no tiene criterios de entrada ni transiciones de salida.
- **Gate Result:** ✅ DO
- **Contrato:** "research-doc cerrado con estados + transiciones (entrada/promoción/expiración con dueño y trigger) + threat model write-time por superficie (API/dream/import), listo para SCH-01"
- **Task file:** `docs/dev/tasks/MGR-13.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `72f29720`; review fresco OK)
- **Cynefin:** 🟨 complicado — máquina de estados + criterios de promoción + threat model de poisoning.
- **Top 3 riesgos:** 1. cuarentena que aísle contenido legítimo (falsos positivos) · 2. estados sin transición de salida (cuarentena permanente) · 3. threat model que no cubre vectores write-time (dream/import).
- **Pre-mortem:** F1: criterios opacos → tabla estados×transiciones con señales medibles + promoción revisable; F2: poison vía consolidación/import no cubierto → threat model POR superficie de escritura (no solo API); F3: dep Backlog MGR-04 ausente → clase de confianza mínima desde MGR-12 + deuda anotada a v1.0/SCH-05.
- **Stop conditions:** si el diseño exige motor de políticas (ABAC/namespaces trusted) → recortar a estado `quarantined` + gates simples y FIND; rabbit hole: hash-chain/PROV-O de auditoría → NO (VER-01 lo cubre sobre WAL, F4).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Falsos positivos aíslan contenido válido | criterios explícitos + promoción revisable + test contención (SCH-05) | review P2-01 |
  | 🟡×🟠 | Deep-poisoning vía dream/import | threat model por superficie write-time | discovery |
  | 🟢×🟡 | Dep MGR-04 ausente | slice mínimo con clase de confianza MGR-12 + FIND | review SCH-01 |
- **Uphill/Downhill:** ⬆️ 1 (threat model propio sin MGR-04) / ⬇️ 4 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog MGR-13 (🟡 3d, :847) se mantiene 2-3d; hash-chain de auditoría queda en VER-01 (F4) — citar, no duplicar; abstención se implementa en SCH-05.

### Task 26: SCH-01 — Plan único + ADR de migración (dims 5-6; alcance 0.8.0 vs v1.0)
- **Fase:** F3
- **Dep:** MGR-10/12/13 (research-docs + Cierre MGR — pre-requisito duro: Backlog:938 "exigir Cierre MGR de los 3 research-docs antes de tocar `record.rs`")

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-arch
- **Archivos clave:** `docs/dev/architecture/adr/ADR-0046-schema-v2-migracion-unica.md` (nuevo; último ADR = ADR-0045 → slot libre) · `docs/dev/research/mgr-10/12/13-*.md` (insumos) · `src/sdk/types/record.rs` (campos destino) · `src/schema.rs` + `src/cli_handlers/migrate.rs` (versionado + plan de migración) · `src/sdk/serialization/mod.rs` (compat export/import :35,498-529) · `docs/api/VERSIONING.md` (política de breaking :16-34) · `docs/dev/plans/2026-09-26-master-roadmap.md` (gate F3 :35)
- **Verificación real:** ✅ CÓDIGO-REAL — **deps abiertas**: `docs/dev/research/` sin material de dims 5/6 (solo `mgr-19-*`, `product-definition-gap-*`, `PLAN-ACCION-*`) y task files MGR-10/12/13 inexistentes (`docs/dev/tasks/` solo MGR-19.md) → SCH-01 NO inicia hasta los 3 Cierres MGR. **Rails listos (HARD-01 ✅)**: `VERSIONING.md:16-27` (0.x: MINOR = frontera de breaking; `feat!:`/`BREAKING CHANGE:` obligatorio), `DEPRECATIONS.md` + `COMPATIBILITY.md` existen, `release-plz.toml` con `semver_check` (Task 1). **Precedentes**: ADR-0028 (supersession), ADR-0044 (acumulado breaking 0.6.0), ADR-0045 (último → slot ADR-0046). **Re-baseline de alcance**: 0.7.0 YA released (`docs/CHANGELOG.md:10` — `[0.7.0] - 2026-09-25`) → el corte de la migración es **0.8.0** (plan :35 + Task 33 SCH-08). "Breaking ilimitado pre-lanzamiento" (Backlog:934).
- **Gate Justificación:** pre-requisito duro de SCH-02..08 (Backlog:938); consolida dims 5-6 en UNA decisión + UNA migración (evita breakings múltiples); sin ADR aceptado + revisión owner no se toca `record.rs`; el corte 0.8.0 ES esta migración.
- **Gate Result:** ✅ DO
- **Contrato:** "ADR de migración aceptado (campos, semántica valid vs transaction, alcance 0.8.0 vs v1.0, plan de migración/backfill, compat export/import) + plan de implementación único con comandos + revisión owner registrada"
- **Task file:** `docs/dev/tasks/SCH-01.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `f24d8b55`; ADR-0046 firmado por owner; review fresco OK)
- **Cynefin:** 🟨 complicado — consolidar 3 investigaciones en una decisión + alcance + compat.
- **Top 3 riesgos:** 1. insumos divergentes/incompletos → ADR ambiguo · 2. segundo breaking accidental (edges SCH-09) · 3. alcance 0.8.0 inflado con v1.0 (derivación/jueces).
- **Pre-mortem:** F1: ADR sin comandos de migración → DoD exige plan con comandos (Backlog:938); F2: specs contradictorias → tabla de reconciliación campo×spec en el ADR; F3: "revisión owner" sin registro → registrarla en el task file (carril owner, §Carril owner L750-763).
- **Stop conditions:** si MGR-10/12/13 no cierran o divergen irresolublemente → NO iniciar SCH-01 (bloquea F3 — escalar a owner); rabbit hole: resolver dims restantes (D7/D8) → NO (post-1.0, L769).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Insumos divergentes | tabla de reconciliación campo×spec + revisión owner | discovery SCH-01 |
  | 🟡×🟠 | Segundo breaking (edges) | SCH-09 dentro de la migración única (Backlog:946) o FIND | review ADR |
  | 🟡×🟡 | Alcance inflado con v1.0 | sección explícita 0.8.0 vs v1.0 + gate F3 | review P2-01 |
- **Uphill/Downhill:** ⬆️ 2 (consolidación dims 5-6 + compat export/import) / ⬇️ 5 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** re-baseline verificado: el seed decía "alcance 0.7.0 vs v1.0" — 0.7.0 ya shipped (CHANGELOG:10); corte = 0.8.0 (plan :35, Task 33). ADR en slot libre 046. Consumir Backlog:934 (decisión owner de migración única) + :938 (DoD).

### Task 27: SCH-02 — Schema v2 (bitemporal + confidence + quarantined + backfill)
- **Fase:** F3
- **Dep:** SCH-01 (ADR aceptado — no tocar `record.rs` antes)

- **Appetite:** max 5d · **Esfuerzo:** 🔴 3-5d · **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `src/sdk/types/record.rs` (`MemoryRecord` :102-138 + `MemoryInput` :59-78 + `MemoryListOptions` :142-160 + `MemoryExportLine` :229-261) · `src/sdk/version_history.rs` (mirror `SnapshotRecord` :76-90 / From :112-148 — no listado en el seed, entra a la migración) · `src/sdk/serialization/mod.rs` (export: `EXPORT_SCHEMA_VERSION` :35, `record_from_export_line` :521-529) + `impl_export.rs` · `src/schema.rs` (`CURRENT_SCHEMA_VERSION` :11) + `src/cli_handlers/migrate.rs` (:175-279) · `src/wal.rs` (postcard forward-compat :22,109) · tests: `tests/core/snapshot_certification.rs:257` · `tests/sdk_serialization.rs:296` · `tests/durability_recovery.rs:90` · `tests/text_index_recovery.rs:785`
- **Verificación real:** ✅ CÓDIGO-REAL — **mapa del schema v1** (13 campos, `record.rs:102-138`) y **4 formatos de persistencia a migrar**: (1) KV/WAL postcard (`src/wal.rs:22,109` — forward-compat por versión); (2) mirror de snapshots `SnapshotRecord` (`version_history.rs:76-90`) que **omite `superseded_*`** (From :112-148 los resetea a `None`) → todo campo nuevo exige mirror + roundtrip propios; (3) export JSONL `EXPORT_SCHEMA_VERSION=1` (`serialization/mod.rs:35`) con `record_from_export_line` que RECHAZA ≠1 (:521-529); (4) header `.vanta.schema` `CURRENT_SCHEMA_VERSION=1` con TooNew/TooOld (`src/schema.rs:11,84-98`) + comando `migrate` existente (`cli_handlers/migrate.rs:175-279`). **Backfill definido** (Backlog:939): `valid_at=created_at`, `invalid_at=superseded_at` si existe. **Infra de verificación existente**: roundtrip export/import (`tests/sdk_serialization.rs:296`; `snapshot_certification.rs:257-286`), reopen (`durability_recovery.rs:90`; `text_index_recovery.rs:785`); índices derivados con schema propio + rebuild-on-mismatch (`serialization/mod.rs:36` `DERIVED_INDEX_SCHEMA_VERSION`; `impl_index.rs:52`) → no requieren migración in-place.
- **Gate Justificación:** el corte 0.8.0 ES esta migración (Backlog:934); el gate F3 exige "migración determinista verde" (plan :35) — backfill + roundtrip + reopen son ese gate; sin v2 no existen SCH-03..07.
- **Gate Result:** ✅ DO
- **Contrato:** "schema v2 implementado (`valid_at`/`invalid_at` + confianza asserted/derived + `quarantined`, todo `#[serde(default)]` compatible v1) Y migración v1→v2 determinista (misma DB → mismo resultado) con backfill (`valid_at=created_at`, `invalid_at=superseded_at` si existe) Y roundtrip export/import verde (v1 sigue importable) Y reopen/durabilidad verdes"
- **Task file:** `docs/dev/tasks/SCH-02.md`
- **Estado:** ✅ COMPLETED (2026-09-28 — commit `7af34366`; review adversarial 2 rondas OK)
- **Cynefin:** 🟨 complicado — campos nuevos × 4 formatos de persistencia + backfill determinista.
- **Top 3 riesgos:** 1. crash mid-migration → DB inconsistente · 2. formatos anidados desincronizados (snapshot mirror/export/WAL) · 3. DB v1 real que no reabre post-migración.
- **Pre-mortem:** F1: campo sin `#[serde(default)]` rompe DBs v1 → default en TODO campo nuevo + test de lectura v1; F2: backfill no idempotente → migración con versión explícita + doble corrida determinista (SCH-06); F3: mirror postcard olvidado → checklist por formato (record/WAL/snapshot/export) + roundtrip por formato.
- **Stop conditions:** si un formato anidado exige rediseño >1d extra → migrar record+export con defaults y diferir snapshots históricos con FIND documentado; rabbit hole: migrar índices derivados in-place → NO (rebuild-on-mismatch ya existe, `impl_index.rs:52`).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Crash mid-migration | migración idempotente + failpoints en SCH-06 | test chaos rojo |
  | 🟡×🟠 | Formatos anidados desincronizados | checklist por formato + roundtrip tests | test reopen rojo |
  | 🟢×🟡 | Backfill no determinista | mismos inputs → mismo output (doble corrida) | review SCH-06 |
- **Uphill/Downhill:** ⬆️ 2 (determinismo + 4 formatos) / ⬇️ 6 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog:939 lista `record.rs` + `serialization/`; verificado: agregar `version_history.rs` (mirror `SnapshotRecord`) y `schema.rs`/`migrate.rs`. Consumir ADR SCH-01 verbatim; export v1 debe seguir importable (compat).

### Task 28: SCH-03 — Queries `AS OF`/point-in-time + filtros `valid_at` + `exclude_superseded`
- **Fase:** F3
- **Dep:** SCH-02 (`valid_at`/`invalid_at` disponibles en el record)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-engine
- **Archivos clave:** `src/parser/` (`grammar.rs`/`lexer.rs` + bump `IQL_VERSION`; patrón gateado `mod.rs:11-28`) · `src/executor.rs` (`execute_hybrid` :153) · `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest` :98-140) · `src/sdk/search/page.rs` (assembly final + hash de cursor :157,304-330) · `src/sdk/types/record.rs` (`MemoryListOptions` :142-160) · `src/sdk/version_history.rs` + `src/sdk/api/memory.rs` (lectura histórica `get_version`/`versions` :409-432) · tests (`src/sdk/search/tests.rs`, inline parser/executor)
- **Verificación real:** ✅ CÓDIGO-REAL — **re-baseline del seed**: `exclude_superseded` YA EXISTE (ADR-0028) — search (`MemorySearchRequest.exclude_superseded` `vector_types.rs:120-123`; aplicado en assembly `page.rs:317-319` "no index change"), list (`record.rs:156-159`; filtrado `src/sdk/api/namespaces.rs:164-169`) y tests (`src/sdk/api.rs:897,936` — hide + default keep); hasheado para cursor (`page.rs:157`). **Gaps reales**: (1) `AS OF`/`valid_at` = 0 hits en `src/parser/` y en dominio `src/`; (2) sin parámetros temporales en `MemorySearchRequest`/`MemoryListOptions`; (3) falta extender `exclude_superseded` a la semántica `invalid_at` nueva. **Precedentes**: lectura histórica por key ya existe (`get_version`/`versions`, `memory.rs:402-432`, VS-CORE-07); filtros `MemoryFilter` AND-only con `Gt/Lt/Gte/Lte` + `Value::DateTime` (`record.rs:11-55`); cláusulas version-gateadas (`PROFILE`, `parser/mod.rs:11-28`); cursor resume con selectores (WIRE-08, `page.rs:304-330`).
- **Gate Justificación:** DoD del Backlog: "el spec decía X en fecha del incidente" devuelve el estado histórico correcto (:940, evidence-before-belief); consume schema v2 y cierra la dim 5 en superficie de query antes del corte 0.8.0.
- **Gate Result:** ✅ DO
- **Contrato:** "`AS OF` operable en IQL (con `IQL_VERSION` bumpeado + feature-detect) y params equivalentes en search/list devuelven el estado histórico correcto con tests deterministas Y filtros por ventana de validez sobre v2 (`valid_at`/`invalid_at`) Y `exclude_superseded` extendido a la semántica nueva Y cursor estable con resume sobre los params nuevos"
- **Task file:** `docs/dev/tasks/SCH-03.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `932b1211` wave F3.3a; review adversarial + delta OK)
- **Cynefin:** 🟨 complicado — semántica temporal en IQL + params + determinismo de cursor.
- **Top 3 riesgos:** 1. semántica `AS OF` ambigua (valid vs transaction — la fija el ADR SCH-01) · 2. filtros temporales fuera del pipeline → rompen top_k/cursor · 3. cláusula nueva que rompe queries v1.
- **Pre-mortem:** F1: `AS OF` leyendo el eje equivocado → semántica exacta del ADR + tests de referencia con T conocido; F2: filtrado post-hoc → integrar junto a `exclude_superseded` en el assembly (`page.rs:317`) + hash de request; F3: gramática rota para v1 → bump `IQL_VERSION` + `iql_supports` (patrón PROFILE).
- **Stop conditions:** si `AS OF` completo exige rediseño del pipeline de search → entregar filtros `valid_at` + `exclude_superseded` extendido y `AS OF` en get/list por key (`version_history`), con FIND para search; rabbit hole: MVCC temporal en índices → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Semántica AS OF ambigua | definida en ADR SCH-01 + tests de referencia | review P2-01 |
  | 🟡×🟠 | Filtros rompen top_k/cursor | integración en assembly + hash de request (`page.rs:157`) | test resume |
  | 🟢×🟠 | Queries v1 rotas | bump IQL_VERSION + feature-detect | parser tests |
- **Uphill/Downhill:** ⬆️ 2 (semántica + cursor con params nuevos) / ⬇️ 5 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** re-baseline verificado vs seed: `exclude_superseded` ya implementado (ADR-0028) → el slice real es `AS OF`/`valid_at` + extensión; SCH-07 expone en bindings/server/MCP (aquí core+IQL); `docs/api/IQL.md` hoy sin `AS OF` (verificado).

### Task 29: SCH-04 — Scores asserted/derived consumibles (slice 0.8.0)
- **Fase:** F3
- **Dep:** SCH-02 (campos de confianza por registro definidos por ADR SCH-01)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-engine
- **Archivos clave:** `src/sdk/types/record.rs` (`MemoryRecord` :102-138 — campos v2 del ADR; `MemoryListOptions` :142-160) · `src/sdk/serialization/vector_types.rs` (`MemorySearchHit` :195-202) · `src/sdk/search/page.rs` (assembly `run_search_page` :257-363; fingerprint de cursor :137-166 si se agrega filtro) · `vantadb-python/src/types.rs` (getters `VantaPySearchHit` :334-377; stub `.pyi` con `test_stub_drift`) · `vantadb-node/index.d.ts` (:64-67, :147-152) · `vantadb-ts/src/types.ts` (:61-70, :173-183) · `vantadb-wasm/src/lib.rs` (:1182-1203, :2233) · `vantadb-mcp/src/handlers/tools.rs` (recall :1824-1828; search hits por serde) · `src/server/handlers.rs` (`SearchPageV2` :524-529) + `docs/api/openapi.yaml` · `docs/api/scores.md` (:61-73) · `docs/api/EMBEDDED_SDK.md` (:540-543) · `docs/api/HTTP_API.md` (:203,:257)
- **Verificación real:** ✅ CÓDIGO-REAL — **gap confirmado**: `MemoryRecord` sin confianza por registro (13 campos, `record.rs:103-138`; `rg 'confidence'` en `src/sdk/search/` = 0 hits — el ranking/assembly no la conoce: RRF `fusion.rs:72-131`, assembly `page.rs:317-330`); `MemorySearchHit` = `record + score + explanation` (`vector_types.rs:195-202`, score :199). **Precedente node-level ya consumible**: `NodeRecord.confidence_score` viaja en superficies graph (`graph_types.rs:68-69,129`; restauración `graph.rs:231`; TS `types.ts:207`) + eviction (`config.rs:285`/`eviction.rs:33-42`) + executor filtra `SemanticSummary` <0.4 (`executor.rs:219-242`). **Consumidores a cubrir**: Py `VantaPySearchHit` (score :334-337; patrón de campo nuevo `superseded_by`/`superseded_at_ms` :369-377); Node `MemorySearchHit` (index.d.ts:147-152; wire napi genérico `lib.rs:106-123`); TS `SearchHit` (types.ts:173-183); WASM `search_hit_to_js` :1182-1203 + `memory_record_to_js` :2233; MCP recall `{content, score, type}` (tools.rs:1824-1828; hits de search serializan vía `text_content_hits_with_budget`, `validation.rs:425`); HTTP `SearchPageV2 { records: Vec<MemorySearchHit> }` (handlers.rs:524-529; doc HTTP_API.md:203,257). **Re-baseline del título**: el seed decía "slice 0.7.0" — STALE: 0.7.0 shipped 2026-09-25 (`docs/CHANGELOG.md:10`); el corte es **0.8.0** (plan :35, Task 33), igual que SCH-01. Blast radius: `MemorySearchHit` 54 callers (mmr/explain/tests; snapshots `tests/query_result_*.rs` + `tests/snapshots/*.snap`); `MemoryRecord` export/WAL/snapshot mirror (SCH-02).
- **Gate Justificación:** el corte F3 (dim 6) no cierra con la confianza solo en storage: el nodo ya la consume (eviction/filtro) y ya la expone en superficies graph — el record v2 sin superficie consumible queda como columna muerta y el circuito MGR-12→SCH-01→SCH-02 no se cierra antes de 0.8.0; la propagación en hits es además el insumo de UI/consola y de la matriz de paridad de ICP-03 (F5).
- **Gate Result:** ✅ DO
- **Contrato:** "`confidence` por registro (valor + clase asserted|derived + `last_validated`, campos fijados por ADR SCH-01) visible en search/get/list del SDK core y propagado en HTTP/MCP vía serde `#[serde(default)]` (sin breaking) Y expuesto en los 4 bindings (Py/TS/Node/WASM con stubs/d.ts sincronizados) Y consultable (filtro opt-in por umbral/clase junto a `range`/`exclude_superseded`, hasheado en el fingerprint de cursor) Y el ranking/UI lo propaga (hits + snapshot deliberado; el orden por defecto NO cambia) Y límites de calibración documentados en `docs/api/scores.md` con derivación completa + grounding con jueces diferida a v1.0 (FIND con dueño) — todo con tests (roundtrip serde + getters + snapshot)"
- **Task file:** `docs/dev/tasks/SCH-04.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `932b1211` wave F3.3a; review adversarial + fixes OK)
- **Cynefin:** 🟨 complicado — wire cross-binding de campos nuevos + consulta sin romper determinismo del cursor ni el orden del ranking.
- **Top 3 riesgos:** 1. scores sin calibración (números decorativos) · 2. doble fuente record↔nodo divergente (`confidence_score` de nodo vs record v2) · 3. snapshot/wire cross-binding roto (54 callers de `MemorySearchHit` + `.snap`).
- **Pre-mortem:** F1: "asserted/derived" sin definición operacional → taxonomía MGR-12 con ejemplos + límites explícitos en `scores.md`; F2: record y nodo divergen → mapeo canónico explícito (precedente `graph.rs:231`) + test de consistencia; F3: drift de wire/snapshot → regeneración deliberada de snapshots + `sdk_serialization` + stub drift verde.
- **Stop conditions:** si la calibración exige datos empíricos → exponer con defaults documentados y diferir calibración a VER-08 (F5, dep cross-fase); si el filtro nuevo complica el cursor/fingerprint → exponer sin filtro y anotar FIND; rabbit hole: jueces/grounding/FACTS → NO (v1.0, stop condition de MGR-12).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Scores no calibrados | límites documentados + calibración diferida VER-08 | review P2-01 |
  | 🟡×🟡 | Record vs nodo divergen | mapeo canónico + test (precedente `graph.rs:231`) | discovery SCH-02 |
  | 🟢×🔴 | Wire cross-binding roto | `sdk_serialization` + stubs/d.ts + snapshots deliberados | gate bindings |
- **Uphill/Downhill:** ⬆️ 1 (wire de campos nuevos en 4 bindings + snapshots) / ⬇️ 5 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** re-baseline verificado del título (seed "slice 0.7.0" → **0.8.0**; 0.7.0 shipped `CHANGELOG.md:10`). Consume MGR-12 (Task 24) + Backlog:941; el nodo ya puntúa confianza (`unified.rs:41-42`, default 0.5 :92) — la unificación record↔nodo y los nombres finales de campo los fija ADR SCH-01. "Derivación completa + grounding con jueces = v1.0" queda como FIND con dueño (stop condition).

### Task 30: SCH-05 — Cuarentena + abstención + trust-aware retrieval (threat model write-time)
- **Fase:** F3
- **Dep:** SCH-02 (estado `quarantined` en el record) + insumo MGR-13 (threat model + estados/transiciones — Task 25; MGR-04 fuera del plan → clase mínima vía MGR-12/SCH-01)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `src/sdk/types/record.rs` (estado `quarantined` v2 + `MemoryListOptions` :142-160) · `src/sdk/serialization/vector_types.rs` (`MemorySearchRequest` :100-145 — include opt-in + umbral de abstención) · `src/sdk/search/page.rs` (filtro por defecto junto a `exclude_superseded` :317-330; fingerprint :137-166) · `src/sdk/api/memory.rs` (get/list/search paths) · `vanta-memory/src/core/hooks/auto_recall.rs` (`perform_auto_recall` :198 — nunca inyectar cuarentenado) · `vantadb-mcp/src/handlers/tools.rs` (`inject_context` :585/:2025; recall :1818-1841; writes lista :30) · `src/server/handlers.rs` + `src/server/middleware.rs` (:42 ingress) · `src/sdk/serialization/mod.rs` (import :522-531) · `vanta-memory/src/core/dream/mod.rs` (promote :24-27) · docs: `docs/api/EMBEDDED_SDK.md` + `docs/api/MCP.md` (estados/abstención) · tests: `tests/memory_api.rs` + nuevo `tests/quarantine_containment.rs` (destino)
- **Verificación real:** ✅ CÓDIGO-REAL — **gap confirmado**: no existe cuarentena de contenido (`rg 'quarantine|quarantined'` en `src/` = solo WAL salvage `src/wal.rs:611-642` — semántica de storage, NO de contenido); `rg 'abstain|abstention'` en `src/` = **0 hits** (único hit del repo: investigación de benchmarks `docs/dev/strategy/VantaDB-Analisis-Arquitectura-Producto-Competencia.md:280`); sin params de trust/clase en `MemorySearchRequest` (:100-145) ni `MemoryListOptions` (:142-160); el único gate de baja confianza es node-level y aislado (`filter_low_confidence_summaries` <0.4, `executor.rs:219-242`). **Superficies write-time a threat-modelar** (insumo MGR-13): HTTP ingress (`middleware.rs:42` auth_middleware + `handlers.rs` records_put), dream (`vanta-memory/src/core/dream/mod.rs` — promote stub :24-27; namespace separado `dream/<session>/<run_id>` :11), import JSONL (`serialization/mod.rs:522-531` rechaza `schema_version` ≠ actual), MCP writes (`tools.rs:30` lista ~37 readOnlyHint=false: memory_put/import/dream_consolidate...). **Inyección a proteger** (DoD "no se inyecta por defecto"): `perform_auto_recall` (`vanta-memory/src/core/hooks/auto_recall.rs:198`), MCP `inject_context` (tools.rs:585/2025), `memory_recall` (tools.rs:1818-1841). Blast radius: `MemorySearchRequest` compartido SDK/HTTP/MCP/bindings; record consumido por export/WAL/snapshots (SCH-02).
- **Gate Justificación:** dim 6/AM10 del corte: la inyección de memoria a prompts es el camino write-then-inject real (AgentPoison/MINJA); un estado `quarantined` sin operación (default-exclude + transiciones) sería columna muerta y sin abstención el sistema "responde" con confianza baja sin señalarlo — el slice MGR-13 no cierra sin esto antes de 0.8.0.
- **Gate Result:** ✅ DO
- **Contrato:** "cuarentena operativa: contenido `quarantined` EXCLUIDO por defecto de search/list/retrieval (include opt-in; `auto_recall`/`inject_context` nunca inyectan cuarentenado) Y transiciones con dueño+trigger (entrada write-time, promoción/expiración) Y abstención selectiva: con umbral de confianza configurado, una consulta sin candidatos suficientes devuelve señal `abstained` explícita en el wire (nunca resultados silenciosamente degradados; default OFF) Y retrieval trust-aware respeta la clase asserted/derived (semántica ADR SCH-01) Y test de contención verde (dudoso no inyectado por defecto) + threat model de MGR-13 citado por superficie"
- **Task file:** `docs/dev/tasks/SCH-05.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `83d65518`; review adversarial 2 rondas: ❌→fixes→delta ✅)
- **Cynefin:** 🟨 complicado — máquina de estados + abstención con semántica honesta + gates write-time sin motor de políticas.
- **Top 3 riesgos:** 1. falsos positivos aíslan contenido legítimo (default-exclude) · 2. abstención que rompe clientes (vacíos silenciosos) · 3. deep-poisoning vía dream/import no cubierto por el threat model.
- **Pre-mortem:** F1: criterios de cuarentena opacos → tabla estados×transiciones con señales medibles + promoción revisable (MGR-13); F2: abstención implementada como "menos resultados" sin señal → `abstained` explícito en el wire + test de contrato; F3: import/dream evaden el default-exclude → test de contención por superficie de entrada (no solo API).
- **Stop conditions:** si los gates de promoción exigen motor de políticas (ABAC/namespaces trusted MGR-04) → recortar a estado + default-exclude + abstención simple y FIND; rabbit hole: hash-chain/PROV-O de auditoría → NO (VER-01 lo cubre sobre WAL, F4).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Falsos positivos aíslan contenido válido | criterios explícitos MGR-13 + promoción revisable + include opt-in | review P2-01 |
  | 🟡×🟠 | Abstención rompe clientes | señal `abstained` explícita + default OFF (solo con umbral configurado) | test de contrato |
  | 🟢×🟠 | Poison vía dream/import | threat model por superficie write-time + contención por entrada | discovery |
- **Uphill/Downhill:** ⬆️ 2 (abstención con señal honesta + gates write-time) / ⬇️ 6 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** consume MGR-13 (Task 25: threat model + estados/transiciones) + clase de MGR-12/SCH-01; MGR-04 (namespaces trusted/tainted) fuera del plan → clase mínima y deuda a v1.0 anotada. DoD Backlog:942: "test de contención (contenido dudoso no se inyecta por defecto) + doc". Hash-chain de auditoría cita a VER-01 (F4), no se duplica.

### Task 31: SCH-06 — Tests: migración determinista, time-travel, roundtrip export/import, chaos
- **Fase:** F3
- **Dep:** SCH-02..05 (schema v2 + queries AS OF + cuarentena/abstención)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-chaos
- **Archivos clave:** nuevos (destinos; nombre final en DISCOVERY): `tests/schema_v2_migration.rs` · `tests/time_travel.rs` · `tests/chaos_migration.rs` · extender: `tests/schema_evolution.rs` (:29) · `tests/core/snapshot_certification.rs` (:257-286 schema_version=1; :1529 `snapshot_restore_fail`) · `tests/sdk_serialization.rs` (:296) · `tests/memory_export_import.rs` (:38-50) · `tests/durability_recovery.rs` (:381-416 `snapshot_serialize_fail`) · `tests/wal_rollback.rs` (:165-217 recovery) · `tests/storage/chaos_integrity.rs` (:127) · `tests/storage/crash_injection.rs` (:1-40 `crash_helper` SIGKILL) · `tests/quarantine_containment.rs` (SCH-05) · `fuzz/fuzz_targets/fuzz_parser.rs` (si `AS OF` toca el parser) · `.config/nextest.toml` (:76 audit, :100-103 chaos) · `.github/workflows/chaos.yml` · `Cargo.toml` (:158 `failpoints`; :397-399 required-features)
- **Verificación real:** ✅ CÓDIGO-REAL — **infra de tests verificada**: failpoints feature (`Cargo.toml:158`, dep `fail` :134) + chaos ya en CI (`.github/workflows/chaos.yml` job "Chaos Tests (failpoints)"; `tests/storage/chaos_integrity.rs:127`; perfil `chaos` `.config/nextest.toml:100-103`) + crash-injection real por subproceso (`tests/storage/crash_injection.rs:1-40`: build `crash_helper` → SIGKILL/TerminateProcess → cold-start) + failpoints de snapshot/WAL (`durability_recovery.rs:381-416`; `snapshot_certification.rs:1529`) + harness `VantaSession`/`TerminalReporter` (`tests/common/mod.rs`) + roundtrip export/import (`memory_export_import.rs:38-50`) + proptest/fuzz (`tests/proptest_serialization_roundtrip.rs`, `tests/proptest_wal_roundtrip.rs`, `fuzz/fuzz_targets/fuzz_{wal,parser,node_deserialize,archive}.rs`). **Gaps**: la suite fija schema v1 (`snapshot_certification.rs:280-286` assert schema_version=1; `sdk_serialization.rs:296`; `tests/common/mod.rs:278,330`) → determinismo de migración, time-travel y bordes (TTL+quarantine, supersede+invalid) no existen; heavy binaries están fuera del filtro default de nextest (correr scoped `-p vantadb` + profiles `audit`/`chaos`, regla HARD-05).
- **Gate Justificación:** el gate F3 exige "migración determinista verde" (plan :35) — esta suite ES ese gate; DoD Backlog:943 (suite + bordes) y los failpoints ya existen → costo marginal bajo; sin esto el corte 0.8.0 no tiene evidencia de recuperación ante crash mid-migration ni de que `AS OF` devuelve el estado correcto.
- **Gate Result:** ✅ DO
- **Contrato:** "suite verde: migración v1→v2 determinista (misma DB → mismo resultado; doble corrida byte-idéntica sobre copia) Y time-travel `AS OF` con fechas de referencia (eje valid/transaction según ADR SCH-01) Y roundtrip export/import v1↔v2 (v1 sigue importable; campos nuevos sobreviven ida y vuelta) Y chaos: crash durante migración → recuperación íntegra (failpoint + reopen + integridad de índices) Y bordes (TTL+quarantine, supersede+invalid) — todo corriendo en CI vía chaos.yml + nextest scoped"
- **Task file:** `docs/dev/tasks/SCH-06.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `b90c494b`; review fast ✅; H1→FIND-186)
- **Cynefin:** 🟨 complicado — determinismo byte-exacto + failpoints de migración + semántica temporal correcta en los tests.
- **Top 3 riesgos:** 1. determinismo frágil (timestamps/orden de records) · 2. failpoints flaky/caros en CI · 3. time-travel mal especificado → tests que fijan la semántica equivocada.
- **Pre-mortem:** F1: migración no determinista por reloj/orden → reloj inyectado + doble corrida byte-idéntica (copia de DB v1 → v2 ×2); F2: no existe failpoint en el runner de migración → instrumentar con `fail` + abort-antes-de-swap (precedente `snapshot_restore_fail`, `snapshot_certification.rs:1529`); F3: `AS OF` testeado contra el eje equivocado → fechas de referencia fijadas por el ADR + revisión cruzada con SCH-03.
- **Stop conditions:** si instrumentar failpoints en la migración exige tocar el engine >1d → cobertura chaos vía crash-injection existente (SIGKILL mid-migración) + FIND para failpoint dedicado; rabbit hole: fuzz targets nuevos de migración → NO (proptest/fuzz existentes primero).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Determinismo frágil | doble corrida byte-idéntica + reloj inyectado | test doble corrida |
  | 🟡×🟠 | Failpoints flaky en CI | reusar chaos.yml + perfil `chaos` + timeouts | 2 corridas rojas seguidas |
  | 🟢×🟠 | Time-travel mal especificado | fechas de referencia del ADR + review SCH-03 | review SCH-03 |
- **Uphill/Downhill:** ⬆️ 2 (determinismo byte-exacto + failpoint de migración) / ⬇️ 6 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog:943 ("OS: failpoints existentes" — verificado: `Cargo.toml:158`, chaos.yml, perfiles nextest). Correr scoped `-p vantadb` + `--profile audit/chaos` (HARD-05: nunca nextest sin `-p` desde raíz). Nombres de archivos nuevos = destino propuesto (los task files SCH-0x no existen aún; se crean en DISCOVERY).

### Task 32: SCH-07 — Superficies: bindings/server/MCP/IQL + docs/api mismo-PR
- **Fase:** F3
- **Dep:** SCH-02..06 (schema v2 completo + suite verde)

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-worker
- **Archivos clave:** `vantadb-python/src/{lib.rs,types.rs}` + `vantadb-python/vantadb_py/vantadb_py.pyi` (search :1162; IQL `query`/`query_structured` :1755-1789; getters :334-377) · `vantadb-node/{src/lib.rs,index.d.ts}` (:64-67, :147-152; wire napi genérico :106-123) · `vantadb-ts/src/{types.ts,guards.ts,vantadb.ts}` (:61-70, :173-183; IQL :1140) · `vantadb-wasm/src/lib.rs` (:1182-1203, :2233) · `src/server/{handlers.rs,router.rs}` (:505-555) + `docs/api/openapi.yaml` · `vantadb-mcp/src/handlers/tools.rs` (parse :3243; `query_iql` :273/:1688) + `vantadb-mcp/tests/mcp_tests.rs` (meta-tests de conteos WIRE-02) · `src/parser/{mod.rs,grammar.rs}` (:17/:26; :95/:120) · docs: `docs/api/{PYTHON_SDK,TS_SDK,NODE_SDK,WASM_API,HTTP_API,MCP,IQL,EMBEDDED_SDK,scores}.md` + `docs/api/BINDINGS_NAMESPACES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — **superficies y puntos de extensión verificados**: Py (`lib.rs:1162` search con `exclude_superseded`+`query_sparse`; IQL `query`/`query_structured` :1755-1789; getters `types.rs` — patrón de campo nuevo :369-377; stubs con drift gate `vantadb-python/tests/test_stub_drift.py` — existe); Node (`index.d.ts:64-67` MemoryRecord + :147-152 MemorySearchHit; wire napi genérico `Value` `lib.rs:106-123`); TS (`types.ts:61-70` MemoryRecord, :173-183 SearchHit; IQL `vantadb.ts:1140`; guards `guards.ts`); WASM (`memory_record_to_js` :2233; `search_hit_to_js` :1182-1203); HTTP (`SearchPageV2` handlers.rs:524-529; `records_search` :531-555; parity `tests/api/openapi_yaml_parity.rs`); MCP (`parse_search_request` tools.rs:3243; meta-tests de conteos en `mcp_tests.rs` — WIRE-02); IQL (`IQL_VERSION=1` parser/mod.rs:17; `iql_supports` :26; gramática :95/:120; tool `query_iql` tools.rs:273/1688). **Gates de docs/paridad**: `scripts/validate-docs-coverage.ps1` (Regla 3 mecánica, CI_POLICY.md:28,34) + `sdk_serialization` (wire cross-binding, VERSIONING.md:76-77) + `public_api` snapshot (VERSIONING.md:82-84) + matriz de las 11 superficies (VERSIONING.md:29-48). **Gap**: ninguna binding conoce campos v2; `AS OF` (SCH-03) queda core+IQL hasta esta task (Notas SCH-03: "SCH-07 expone en bindings/server/MCP").
- **Gate Justificación:** con 11 superficies contractuales (VERSIONING.md:29-48), un schema v2 visible solo en core es un breaking silencioso para consumidores multi-lenguaje; Regla 3 mecánica (`validate-docs-coverage`) evita el drift día 1 y la matriz de paridad alimenta ICP-01..03 (F5); los cambios de wire son aditivos (`#[serde(default)]`) pero deben marcarse como frontera MINOR del corte.
- **Gate Result:** ✅ DO
- **Contrato:** "los campos v2 (bitemporal + confianza + quarantined) y los params de query (AS OF/valid_at + abstención) cruzan Py/TS/Node/WASM + HTTP (+`openapi.yaml`) + MCP + IQL con los MISMOS nombres de wire Y matriz de paridad verde (`openapi_yaml_parity` + `sdk_serialization` + stub drift `.pyi` + d.ts + meta-tests MCP de conteos) Y `docs/api/` actualizadas en el mismo PR (Regla 3: `validate-docs-coverage` exit 0) Y `public_api` snapshot regenerado deliberadamente (sin cambios Rust no intencionales)"
- **Task file:** `docs/dev/tasks/SCH-07.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `aa111979`; review adversarial + delta ✅; docs 0 gaps) · **Branch:** develop · **Commit:**
- **Cynefin:** 🟨 complicado — paridad de 8 superficies + openapi + docs en el mismo PR sin romper contratos existentes.
- **Top 3 riesgos:** 1. drift de nombres/shape entre superficies · 2. breaking accidental no marcado (semver-checks vs baseline 0.7.0) · 3. docs/api desincronizadas → gate rojo.
- **Pre-mortem:** F1: cada binding inventa su forma → wire canónico desde core (serde) + matriz en `BINDINGS_NAMESPACES.md` + tests por binding; F2: WASM/desktop exceden el appetite → stop condition: core+3 bindings+HTTP+MCP+IQL primero y FIND para el resto; F3: docs a medias → `validate-docs-coverage` en el mismo PR + checklist por doc del listado de Archivos clave.
- **Stop conditions:** si WASM (browser) o desktop no cierran dentro del appetite → entregar Py/TS/Node + HTTP + MCP + IQL y diferir WASM/desktop con FIND documentado (paridad parcial declarada, nunca silenciosa); rabbit hole: refactor de tipos compartidos entre bindings → NO.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟠 | Drift entre superficies | wire canónico + matriz de paridad + tests por binding | gate bindings |
  | 🟡×🔴 | Breaking no marcado | `feat!` + semver-checks + snapshot deliberado | semver job |
  | 🟢×🟡 | Docs desincronizadas | Regla 3 mecánica `validate-docs-coverage` mismo PR | gate-docs |
- **Uphill/Downhill:** ⬆️ 2 (paridad de 4 bindings + openapi + docs mismo-PR) / ⬇️ 7 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** Backlog:944; coordina con API-01..09 (mismo sistema de tipos); precedente WIRE-03 (stubs 7/7, `test_stub_drift.py`) y WIRE-02 (meta-tests de conteos MCP). WASM/desktop pueden diferirse por stop condition (FIND) — la promesa del contrato se evalúa por superficie declarada, no por intención.

### Task 33: SCH-08 — Corte 0.8.0: migration guide + CHANGELOG + release notes
- **Fase:** F3
- **Dep:** SCH-07 (superficies + docs sincronizadas)

- **Appetite:** max 2d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/user/operations/UPGRADE.md` (nueva sección "Upgrading to 0.8.0" sobre el placeholder :68-71; patrón :45-66) · `docs/CHANGELOG.md` (entrada `[0.8.0]` generada por release-plz) · `docs/api/COMPATIBILITY.md` (§ Pre-release deltas → cierre 0.8.0; rails :17-30) · `docs/api/VERSIONING.md` (:16-27 contrato 0.x; :113-119 release mechanics) · `docs/api/DEPRECATIONS.md` (si aplica) · `release-plz.toml` (changelog_path/git_release_enable) · `.github/workflows/release.yml` (jobs release-plz) · PR release-plz vigente (owner lane)
- **Verificación real:** ✅ CÓDIGO-REAL — **infra de release verificada**: `docs/CHANGELOG.md:8-14` (`## [Unreleased]` vacío; `[0.7.0] - 2026-09-25` :10 — el corte activo es **0.8.0**, plan :35); `release-plz.toml` (`changelog_path = "docs/CHANGELOG.md"`, `git_release_enable = true` → release notes automáticas del GitHub release, `release_always = false` (HIG-01), `semver_check = true`); `.github/workflows/release.yml` jobs `release-plz-release` + `release-plz-pr` (action pin v0.5.139 :38/:67). **Rails HARD-01 ✅**: `VERSIONING.md:16-27` (0.x: MINOR = frontera de breaking; `feat!`/`BREAKING CHANGE` obligatorio) + `COMPATIBILITY.md:17-30` + `DEPRECATIONS.md` + `UPGRADE.md`. **Ancla de la guía**: `UPGRADE.md:68-71` ya declara "Pending breaking changes for the next MINOR (0.8.0, in development) ... lands in this section with the 0.8.0 release" — el trabajo es rellenar esa sección con la migración v1→v2 real. **Owner lane**: `RELEASE_PLZ_TOKEN` + PR #228 (release-plz) en plan :880-884; "nunca manual" = Regla 7 (release-plz único versionador). **Re-baseline verificado**: Backlog:945 decía "tag 0.7.0" — STALE (0.7.0 shipped 2026-09-25); el corte de esta fase es 0.8.0. Gap: no existe guía de migración v1→v2 propia (`rg -i migrat docs` = guías de otras DBs `docs/user/tutorials/migrating-*` + `SQLITE_MIGRATION_GUIDE.md`, ninguna de VantaDB).
- **Gate Justificación:** cierre de fase F3: el corte 0.8.0 ES la migración única (Backlog:934, decisión owner); sin guía publicada + CHANGELOG con breaking marcados, los consumidores 0.7.0 se rompen sin aviso (la migración v1→v2 toca el formato); release-plz es el único versionador permitido (Regla 7 / plan :44) y las release notes quedan revisadas antes del merge.
- **Gate Result:** ✅ DO
- **Contrato:** "UPGRADE.md §'Upgrading to 0.8.0' publicado (campos v2 + semántica valid/transaction + `AS OF` + pasos de migración/backfill + backup pre-upgrade) Y entrada `[0.8.0]` en `docs/CHANGELOG.md` con breaking changes marcados (`feat!`/`BREAKING CHANGE`) Y release notes del GitHub release (release-plz) revisadas Y corte ejecutado SOLO vía release-plz (Release PR merge — nunca tag/versión/CHANGELOG a mano, Regla 7), con verificación post-release de artefactos (crates/wheels/npm)"
- **Task file:** `docs/dev/tasks/SCH-08.md` (a crear en DISCOVERY)
- **Estado:** ✅ COMPLETED (2026-09-29 — commit `b9296909` mensaje R1; review ❌→fixes→delta ✅; R2 aplicado) · **Branch:** develop · **Commit:**
- **Cynefin:** 🟨 complicado — release-plz como único versionador + guía de migración real + artefactos multi-ecosistema.
- **Top 3 riesgos:** 1. Release PR sin la migración (bump/CHANGELOG incorrecto o sin breaking notes) · 2. guía incompleta → usuarios 0.7.0 rotos · 3. artefactos post-release fallan (wheels/npm/crates).
- **Pre-mortem:** F1: ningún commit del tramo usa `feat!` → release-plz propone PATCH → auditar commits + `BREAKING CHANGE` footer antes del PR y verificar el diff del Release PR; F2: guía publicada sin probar el upgrade → smoke de upgrade real (DB v1 0.7.0 → 0.8.0) registrado en el task file; F3: merge sin token/release roto → queda en carril owner (bloqueo registrado, no silencioso; artifacts re-verificados con release-wheels/release-npm).
- **Stop conditions:** si el Release PR no refleja 0.8.0 correctamente (versión o notes) → NO mergear; corregir commits y regenerar (nunca editar el CHANGELOG a mano); si el token owner no está disponible → publicar guía + notes revisadas y dejar el merge en carril owner con bloqueo anotado; rabbit hole: escribir release notes a mano fuera del CHANGELOG → NO (skill `release-notes-one-pager` solo como artefacto de anuncio, nunca como fuente).
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🔴 | Release PR sin breaking notes | auditar commits `feat!` + revisar diff del PR release-plz | review del PR |
  | 🟡×🟠 | Guía incompleta | checklist campos+AS OF+backup sobre :68-71 + smoke de upgrade | review P2-01 |
  | 🟢×🟠 | Artefactos post-release fallan | verificación post-merge (release-wheels/release-npm) + RELEASE_PLZ_TOKEN owner | post-release |
- **Uphill/Downhill:** ⬆️ 1 (Release PR correcto con breaking notes + guía real) / ⬇️ 4 steps
- **DoD task:** contrato ⬜ · task file sync · recitation · **Iteraciones:** 0 · **Notas:** re-baseline verificado (Backlog:945 decía "tag 0.7.0" — STALE; corte = 0.8.0, `CHANGELOG.md:10` + plan :35). Owner lane: RELEASE_PLZ_TOKEN + PR #228 (plan :880-884; owner decide merge/close). Regla 7 / plan :44: "Release: release-plz (nunca tags/versión/CHANGELOG a mano)". La sección nueva de UPGRADE.md es el entregable de guía; las release notes salen del GitHub release de release-plz (revisadas).

### Task 34: VER-07 — Dreams: dry-run + diff report + `promote_dream_run` real
- **Fase:** F4
- **Dep:** — · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "dry-run + diff + promote (ADD/UPDATE/DELETE/NOOP) con tests" · **Task file:** `docs/dev/tasks/VER-07.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 35: VER-01 — Tamper-evident: hash-chain en WAL + `vanta-cli verify`
- **Fase:** F4
- **Dep:** — · 🔴 3-5d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "hash-chain + comando verify + test de manipulación detectada" · **Task file:** `docs/dev/tasks/VER-01.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 36: VER-05 — Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio
- **Fase:** F4
- **Dep:** — · 🟢 1-2d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "importadores con roundtrip + formato documentado" · **Task file:** `docs/dev/tasks/VER-05.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 37: VER-06 — Export file-native Markdown + `rebuild_index` (git-friendly)
- **Fase:** F4
- **Dep:** — · 🟢 1-2d · 🟡 · **Ruta:** vanta-worker · **Contrato:** "export MD + rebuild_index + roundtrip verde" · **Task file:** `docs/dev/tasks/VER-06.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 38: VER-02 — Borrado certificado (delete-path shred→GC→WAL + attestation)
- **Fase:** F4
- **Dep:** SCH-02 · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "attestation de purga verificable + tests" · **Task file:** `docs/dev/tasks/VER-02.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 39: VER-03 — Redacción-on-write persistida + namespaces cifrados
- **Fase:** F4
- **Dep:** WIRE-01 ✅ · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "redacción persistida + envelope por namespace + test PII=0" · **Task file:** `docs/dev/tasks/VER-03.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 40: VER-04 — Governance de inyección (presupuesto + ACLs + audit log)
- **Fase:** F4
- **Dep:** WIRE-01 ✅ · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "presupuesto/ACLs enforced + audit log de inyección" · **Task file:** `docs/dev/tasks/VER-04.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 41: MKT-18f — Publicar 9 adapters PyPI (owner-assisted)
- **Fase:** F5
- **Dep:** — · 🟡 (owner) · 🔴 · **Ruta:** vanta-lead (packaging; publish = owner) · **Contrato:** "9 adapters publicados o disposición documentada por adapter" · **Task file:** `docs/dev/tasks/MKT-18f.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 42: ICP-01 — Track AI-IDEs (MCP): one-pager + repo-map/watcher + viewer + hooks + demo CI
- **Fase:** F5
- **Dep:** MGR-22 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI verde + métrica North Star instrumentada" · **Task file:** `docs/dev/tasks/ICP-01.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 43: ICP-02 — Track local-LLM/privacidad: one-pager + redacción/cifrado/forget + auditoría PII + demo CI
- **Fase:** F5
- **Dep:** VER-02/03/04 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI + 0 PII en auditoría" · **Task file:** `docs/dev/tasks/ICP-02.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 44: ICP-03 — Track frameworks: one-pager + adapters PyPI (MKT-18f) + importadores (VER-05) + demo CI
- **Fase:** F5
- **Dep:** MKT-18f, VER-05 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI + instalación de adapters verde" · **Task file:** `docs/dev/tasks/ICP-03.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 45: VER-08 — Harness propio: canonical_p99 + LoCoMo/LongMemEval-S/BEAM-subset + p99-CI
- **Fase:** F5
- **Dep:** — · 🟡 3-5d · 🔴 · **Ruta:** vanta-tuner · **Contrato:** "harness publicado (dataset commiteado) + p99 en CI + write-quality/abstención" · **Task file:** `docs/dev/tasks/VER-08.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 46: DEF-06 — README↔BENCHMARKS reconciliados (claims)
- **Fase:** F5
- **Dep:** VER-08 · 🟡 2d · 🔴 · **Ruta:** vanta-docs · **Contrato:** "claim RocksDB corregido + §2 regenerado + números citables (Regla 11)" · **Task file:** `docs/dev/tasks/DEF-06.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 47: VER-09 — Head-to-head Mem0/Zep/Letta con protocolo publicado (absorbe EXE-02)
- **Fase:** F6
- **Dep:** VER-08 · 🔴 1-2sem · 🟠 · **Ruta:** vanta-tuner · **Contrato:** "reporte win/loss con protocolo + dataset reproducible" · **Task file:** `docs/dev/tasks/VER-09.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 48: EXE-01 — Demos CI (3 casos: memory/verify/governance)
- **Fase:** F6
- **Dep:** — · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "3 demos ejecutables en CI + output documentado" · **Task file:** `docs/dev/tasks/EXE-01.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

### Task 49: N-17 — Notion sync (drafts NOTION-SYNC-2026-09-24.md → páginas)
- **Fase:** F6
- **Dep:** — · 🟢 0.5d · 🟡 · **Ruta:** vanta-lead (Notion MCP) · **Contrato:** "10 páginas sincronizadas (0.7.0 vigente, no stale) + draft aplicado" · **Task file:** `docs/dev/tasks/N-17.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**

---

### Task 50: HARD-08 — SDP v3: mejoras de búsqueda/lectura/revisión/selección de skills (PRE-RUN ✅)
- **Fase:** F0

- **Appetite:** max 2d · **Esfuerzo:** 🟡 1d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H vanta-harness)
- **Archivos clave:** `.opencode/task-system/mcp/sdp-v3.mjs` (nuevo) · `.opencode/task-system/mcp/campaign-server.mjs` · `.opencode/task-system/scripts/{build-skills-index,sdp-selftest,record-skill-outcome,smoke-mcp-sdp}.mjs` · `.opencode/task-system/skills-index.json` · `.opencode/references/skills-engineering.md` · prompts `task.md`/`pipeline-full.md`
- **Verificación real:** ✅ CÓDIGO-REAL — implementado y verificado E2E (selftest 15/15 · smoke v2+v1 OK · índice 195 skills/279 ratings) con Gate H ✅ (3 rondas: ❌→❌→✅)
- **Gate Justificación:** decisión owner "todas las mejoras" (2026-09-27) tras auditoría de 7 gaps del SDP v2 (ES↔EN, morfología, drift spec↔impl, familias, deprecated, sin índice/pins/cuotas/outcomes)
- **Gate Result:** ✅ DO (ejecutado pre-run)
- **Contrato:** "selftest 15/15 Y smoke E2E OK (sdpVersion=v3 + v1 operativo) Y índice 195 Y Gate H ✅ Y nombre/contrato v2 preservados"
- **Task file:** `docs/dev/tasks/HARD-08.md`
- **Estado:** ✅ COMPLETED (2026-09-27, pre-run — commits `.opencode` `903a029` + `e97c093`) · **Branch:** develop · **Commit:** `903a029` (.opencode, local)
- **Notas:** los sub-agentes del run usarán v3 **tras restart de OpenCode** (MCP live). Owner action registrada en el task file.

## Carril owner (paralelo — sin pipeline)

| Ítem | Qué | Por qué | Cómo |
|------|-----|---------|------|
| **RELEASE_PLZ_TOKEN** | Setear el PAT fine-grained como secret | Cascadas npm/PyPI/wheels/SBOM automáticas (sin dispatch manual) | `gh secret set RELEASE_PLZ_TOKEN --repo ness-e/Vantadb` (pegar el token ahí, NUNCA en chat) |
| **Push final** | `git push origin develop` al completar el plan (validado/verificado) | Decisión owner 2026-09-26 | Al cerrar F0-F6 (o cuando el owner decida) |
| **FASE-A** | 5 installs + stranger tests (`docs/dev/FASE-A.md`) | Gate del anuncio (pausado hasta verde) | Owner-side (conseguir personas + fichas) |
| **R-05** | Smoke usuario real (venv limpio pip/npm) | Opcional; valida artefactos publicados | Ver `estabilizacion-pendiente.md:41` (referencia 0.6.1) |
| **PRs #228/#224** | Decidir al momento del release | #228 = release-plz (se actualiza a 0.8.0); #224 = dependabot (croaring) | Owner decide merge/close |
| **BIZ-04/10/11/12/13** | ToS/Privacy/Refund, Lemon Squeezy, Binance Pay, Payoneer→Airtm, invoice | Carril negocio (bloquea cobros) | `docs/dev/strategy/GO_TO_MARKET.md` |
| **LEG-01** | Dictamen de marca antes de filing | Riesgo legal | Owner |
| **HIG-02** | Firma del waiver ADR-041 (firma formal diferida) | Cierra deuda API-01 | Owner firma |
| **DEC-01** | ADR pendiente de escribir (P38 ✅) | Regla 5 (lo articula el owner) | Owner |
| **MCP refresh** | Rebuild `target/debug` + restart (frescura total) | Binario local puede ser pre-API-04 | Cerrar OpenCode → rebuild → reabrir |

## Cola restante del Backlog (disposición — no en F0-F6)

| Grupo | Filas | Disposición |
|-------|-------|-------------|
| Investigación gobernanza | P49 MGR-01..25 (resto, fuera de MGR-10/12/13) | Post-1.0 (research lane) |
| Docs subpáginas | P50 EXE-04..10 (EXE-03 = gate FASE-A owner) | F6/después (depende de anuncio) |
| Web/labs | P34 UX-*, P43 WEB-09, SHOW-02/03 | Post-0.8.0 (labs web) |
| Bindings extra | P41 TS-10/11/13, P42 WSM-14 | Con F2/F3 (lane bindings) |
| Desktop/Studio | P46 DESKTOP-41/43/44, STU-01..03 | Labs (post-1.0) |
| Providers | P45 PROV-12 | Post-1.0 |
| Varios | P9 OLD-01 (PGWire), P16 CI-01, GOV-TK5, MKT-18i (blocked upstream), P5 DISC-03 (ICEBOX) | Post-1.0 / ICEBOX / bloqueado (sin acción) |
| Futuro | P24 FUT-02..15 | Registro futuro (post-1.0) |
| FINDs varios | FIND-118, 120-122 | Triage en F1/F2 si aplica |
| FINDs CI | FIND-134..147 (~14) | HARD-02 evalúa/cierra vigentes |
| FINDs deuda | FIND-152 → HARD-02 · FIND-153/154 → HARD-06 · FIND-155 → F1 (dashboard 401) · FIND-156 → F2 · FIND-157 → F2/core · FIND-158/159 → post-0.8.0 · FIND-160/161/162 → HARD-06 | Ruteadas |
| Huérfanos | MEM-55 (task file sin fila) → HARD-06 catalogar/cerrar · FIND-98 → owner (retiro/reinstalación) | HARD-06 / owner |

## Herramientas, skills y MCP (referencia para ejecución)

**MCP:** `codegraph_explore` (blast radius — siempre primero) · `codebase-memory-mcp` (architecture/impact/semantic) · `campaign_*` (task system: next_task, verify_cmd, update_task_state, discover_skills_v2, memory_write) · web: `coordinated-web-search` + MetaSearchMCP/Argus/firecrawl/webfetch · `vantadb` (memoria: recall/search) · `notion` (N-17) · Playwright (si UI) · `gh` CLI (PRs/CI).

**Skills base:** `campaign-executor` · `progreso` · `ponytail(full)` (siempre).
**Skills por fase:** F0: `ci-cd-and-automation`, `git-workflow-and-versioning`, `documentation-and-adrs`, `deprecation-and-migration` · F1: `security-and-hardening`, `api-and-interface-design` · F2: `rust-write-tests`, `performance-optimization`, `source-driven-development` · F3: `database-design`, `database-schema-designer`, `test-driven-development` · F4: `security-and-hardening`, `observability-and-instrumentation`, `rust-write-tests` · F5: `release-notes-one-pager`, `performance-optimization` · F6: `shipping-and-launch`, `unified-review`.
**Comandos clave:** `dev-tools/verify.ps1` / `verify_changed.ps1` · `cargo nextest --profile audit -p <crate>` (NUNCA sin `-p` desde raíz — HARD-05) · `cargo semver-checks` · `cargo public-api` · `dev-tools/ocr-review.ps1` · `scripts/validate-docs-coverage.ps1` · `/harness` (Gate H) · `/audit certify` · `/ship`.

## Residuales absorbidos → ruteo

| Residual (de planes absorbidos) | Ruteo |
|--------------------------------|-------|
| FIND-152 (harness L7 warn) | HARD-02 |
| FIND-153 + FIND-154 (gate perf) | HARD-06 |
| FIND-160/161/162 | HARD-06 |
| MEM-55 (scheduler host huérfano) | HARD-06 |
| EST-12 (FASE-A/R-05) | Owner lane |
| HIG-02/HIG-03 (P58) | HIG-02 → owner; HIG-03 → post-0.8.0 |
| DEC-01 (ADR) | Owner lane |
| Propuesta "próximo arco" (gap R4) | **Este plan la entrega** |
| N-17 (Notion sync) | Task 49 |

## Notas / plan-adjust

- **plan-adjust [2026-09-26]:** creación del Master Roadmap — absorción de 5 planes activos (archivados) + investigación de riesgos R1–R4 (sub-agentes, multi-fuente) + 6 decisiones owner vía `question` (HARD atómicas · profundidad F0-F1 · task files HARD-*/F0-F1 · absorción+archivo · push al final · gates (a)+(b)+(c)).
- **plan-adjust [2026-09-26]:** ⬆️ uphill antes: propuesta de próximo arco sin entregar + roadmap disperso (5 planes + strategy + Notion). ⬆️ uphill después: 5 incógnitas (F2-F6 esenciales). ⬇️ downhill: 44 tasks listas.
- WIRE-10 task file: `Plan file:` actualizado a este master (era el plan viejo archivado).
- **plan-adjust [2026-09-27]:** HARD-08 (SDP v3) ejecutado **pre-run** (decisión owner "todas las mejoras"); Gate H ✅ 3 rondas (vanta-harness); harness commiteado (`903a029` + `e97c093`).
  - ⬆️ uphill antes: 7 gaps SDP v2 (ES↔EN, morfología, drift spec↔impl, sin índice/pins/cuotas/outcomes) → **después: 0** (implementados).
  - ⬇️ downhill antes/después: 50 → 49 (HARD-08 ✅).
  - ⚠️ Owner action **resuelta:** `opencode reload` respawnea MCP servers sin restart del server (verificado: campaign live sirve v3 con morfología+aliases).
  - **Pre-run pass (R2) ✅:** 14/14 task files F0/F1 con SDP v3 actualizado (pins+aliases) — commit local.
- **plan-adjust [2026-09-27b]:** **Pre-run hardening del harness (Gate H ✅ APPROVE, 3 rondas).** 4 auditorías profundas (vanta-harness) + 30+ fixes: gates (verify retry/refund, budget 40, validate_scope canónico), claim wave-aware (salta ⏳, WIP=3), seguridad (inyección workDir/git, PIPED_SHELL blocking, sandbox deadlock), observabilidad (eval por tarea+filtro plan, skills SDP en verify-log), parsers (id canónico `HARD-01`, formato compacto), plan normalizado (50 Estados escribibles + Fase ×50), hooks alineados, checkpoint. Tests: **49/49** + parity 10/40/40/5/120 + probes E2E en vivo. ⬆️ antes: 13 hallazgos 🔴/🟠 + 9 seguridad → **después: 0 bloqueantes**. ⬇️ downhill: 49.
- **plan-adjust [2026-09-27c]:** **F1 COMPLETA (8/8).** DEF-03 (`8a884ba6`+`b82f90cb`) · DEF-04 (`d45deba5` — ADR-0045 firmado por owner, Gate P) · DEF-05 (`58c9903c`) · DEF-07 (`fd50ea22`) · DEF-08 (`1a696766`) + cierres `44e3394e`/`3dc52f16` + FIND-169..175. Fix de harness: MCP codegraph/CBM nativos para sub-agentes (`543f917` + `3389234e`, Gate H). ⬆️ antes: 3 en progreso F1b → después: 0. ⬇️ downhill: 34.
- **plan-adjust [2026-09-27d]:** **Bloques F2 completados al nivel F0/F1 (REGLA L412-422)** — Tasks 16-22 (WIRE-02..08), 23 campos c/u, Paso 0 verificado (codegraph/CBM): WIRE-02 re-baseline 85 listadas (`MCP.md:265`); WIRE-04 re-scope (TTL HTTP ya existe → default por colección + sweeper); WIRE-07 trait-split ya ejecutado → bookkeeping + OpGate/server→cli; deps MGR-05/09/16 anotadas. Gate de fase F2 ✅ → wave F2 lanzada (WIRE-03 ‖ WIRE-04; WIRE-02 en cola). Commit `0b6f9822`.
- **plan-adjust [2026-09-28e]:** **Bloques F3 completados al nivel F0/F1 (REGLA L412-422)** — Tasks 23-33 (MGR-10/12/13 + SCH-01..08), 23 campos c/u, Paso 0 verificado (codegraph/CBM): SCH-01 re-baseline alcance 0.8.0 (0.7.0 shipped); SCH-03 `exclude_superseded` YA existe (gaps reales = AS OF/valid_at); SCH-04 re-baseline slice 0.8.0; SCH-08 corte 0.8.0 (Backlog stale corregido); deps fuera del plan anotadas (MGR-01/04). Gate de fase F3 ✅ → wave F3.1 lanzada (MGR-10 ‖ MGR-12 ‖ MGR-13).
- **plan-adjust [2026-09-28f]:** **Owner ratificó los 16 defaults de diseño F3** (question 2026-09-28, opción "Aprobar defaults y avanzar"): MGR-10 (6) + MGR-12 (4) + MGR-13 (6). SCH-01 los consolida en ADR-0046; el ADR pasa por revisión owner explícita (Regla 5). Cierre MGR ✅ = pre-req duro SCH-01 cumplido. Wave F3.2 lanzada (SCH-01).
- **plan-adjust [2026-09-28g]:** **F3.2 cerrada (28/50)** — SCH-01 (ADR-0046 firmado por owner) + SCH-02 (schema v2 + migración determinista + backfill; `7af34366`; review adversarial 2 rondas ✅ 2403/2403). Wave F3.3 lista: SCH-03 ‖ SCH-04 ‖ SCH-05 (desbloqueadas).
- **plan-adjust [2026-09-29a]:** **F3.3a cerrada (30/50)** — SCH-03 (AS OF/valid_at/cursor) + SCH-04 (confidence consumible + filtros): **commit combinado `932b1211`** por changesets entrelazados (`vector_types.rs`/`page.rs`, wave co-batch) — documentado. Reviews adversariales frescos ✅ + fixes post-review (rechazo AS OF duplicado + delta; MCP schema min/max). FIND nuevo: `campaign_get_next_task` no desambigua con `planFile` si hay 2 planes activos (workaround: claim vía `update_task_state`). F3.3b lanzada: SCH-05.
- **plan-adjust [2026-09-29b]:** **F3.3b cerrada (31/50)** — SCH-05 cuarentena+abstención (`83d65518`): default-exclude + gates de inyección (l1_reader/axioms) + transiciones T1-T5 con audit + sticky raw transport (F4a) + abstención explícita. Review adversarial 2 rondas (❌ F1-F6/N → batch → delta ✅). Docs/api diferidas a SCH-07 (2 gaps declarados). Nota: la sesión de docs commiteó su consolidación (`d23e1224` + `86c55e01`) — WIP ajeno masivo cerrado.
- **plan-adjust [2026-09-29c]:** **F3.4 cerrada (32/50)** — SCH-06 suite determinista/time-travel/roundtrip/chaos (`b90c494b`; review fast ✅; CI exacto 2/2). FIND-186 registrado (`70196101`: WAL shard-group crash → salvage). Índices regenerados (`e23bc11e`). Nota: sesión docs commiteó consolidación completa (`d23e1224`/`86c55e01`/`10577ec0`). Wave F3.5 lanzada: SCH-07 (recoge docs/api diferidas de SCH-03/04/05 + 2 gaps).
- **plan-adjust [2026-09-29d]:** **F3.5 cerrada (33/50)** — SCH-07 superficies (`aa111979`): 8 superficies con wire unificado + docs/api mismo-PR (2 gaps cerrados, 0 gaps) + public_api +4 auditado; review adversarial + delta ✅. FIND-187/188 registrados (abstención page-shaped por binding; quarantine ops/import args en HTTP/MCP/CLI). semver-checks: 9 fallas del corte acumulado → SCH-08 (`feat!`). Wave F3.6 lanzada: SCH-08.
- **plan-adjust [2026-09-29e]:** **F3 COMPLETA (34/50) — GATE F3 ✅ local.** SCH-08 (`b9296909`): guía Upgrading-to-0.8.0 + auditoría release-plz (R1: commit marcador `feat(schema)!` + footer BREAKING CHANGE — garantiza 0.8.0 + breaking notes; R2: `[changelog] header` — preserva frontmatter, aplicado) + release notes draft + handoff owner (7 pasos; PR #228 vigente NO mergear). Review ❌→fixes→delta ✅. Release real = lane owner (push bloqueado por política). Siguiente: bloques F4 → wave F4.

## Recitation

```
=== RECITATION MASTER-ROADMAP ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: F3 — Bitemporalidad/confianza/cuarentena + schema v2 (MGR/SCH)
Estado: in-progress
Última acción: **F3 COMPLETA (34/50)** — SCH-08 corte 0.8.0 (`b9296909` marcador R1 + R2); gate F3 ✅ local; siguiente: bloques F4
Resultado: OK (F0 7/7 · F1 8/8 · F2 7/7 · **F3 11/11**)
Próxima acción: completar bloques F4 (REGLA) → wave F4 (VER-07/VER-01/VER-05/VER-06/VER-02/VER-03/VER-04)
Contrato: —
Próxima tarea si completa: HARD-01
=== END RECITATION ===
```

**Ejecución:**
```
/pipeline run docs/dev/plans/2026-09-26-master-roadmap.md
```

=== RECITATION HARD-02 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-02 — tuning de gates (coverage reporte+presupuesto, review risk-tiered, nightly, release dry-run)
Estado: completed
Última acción: Decisión owner (c) implementada y verificada: coverage report+budget movido al nightly (dev-tools/coverage-budget.ps1 + nightly.yml job coverage-budget, artifact); verify.ps1 con -IncludeCoverage default OFF → ALL 10 PASS, 209.5s warm <5min (era 509.4s); contrato 4/4; Gate H APROBADO; P2-01 APROBADO (3 rondas); FIND-163 resuelto; floor-guard exit 0; actionlint 0.
Resultado: OK
Próxima acción: Commit local del changeset por el lead (VantaDB + .opencode por separado; NO push sin OK del owner). Siguiente tarea: HARD-04
Contrato: 4/4 ✅ — (1) coverage no bloquea + reporte+presupuesto documentado (coverage-budget.ps1 6 buckets baseline−1.0pt, enforce en nightly; CI_POLICY §Coverage) ✅; (2) review risk-tiered en prompts (globs docs/api|sdk|parser|storage|wire+node/text_index; diff mixto=adversarial; Gate H verde) ✅; (3) nightly.yml existe (subset diario 05:00 + coverage-budget + notify auto-issue + release dry-run cargo publish --dry-run) ✅; (4) fast gate medido 209.5s warm <5min ✅. Evidencia: hard02-verify-r3.log (ALL 10 PASS), floor-guard 0, actionlint 0, publish dry-run exit 0 (59.8s), docs-coverage 0 gaps. Invariantes: ADR-0018 intacto; floor-guard verde; budget falla en violación explícita. Deuda tracked: RULES.md '27 files'→HARD-03; calibración ubuntu del primer nightly; gitleaks historia 197 (diff-scoped por diseño).
Próxima tarea si completa: HARD-04
=== END RECITATION ===

=== RECITATION HARD-01 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-01 — rails de breaking changes (fix ronda 1 de review aplicado)
Estado: in-progress
Última acción: Fix ronda 1: UPGRADE.md:69 ../api/ -> ../../api/; COMPATIBILITY.md anchor #pre-10-stability-contract + claim cold-run real (149.7s/391.1s); link-check changeset 58 relativos -> 0 rotos, 0 anchors malos; validate-docs-coverage ✅ post-fix; §Review ronda 1 registrada
Resultado: OK
Próxima acción: Ronda 2 de review (vanta-review, sesion fresca) sobre el diff post-fix; luego LEAD: commit local 3 bloques (sin push)
Contrato: verificacion: check-release exit 100 raw x2 (findings triados+documentados en COMPATIBILITY) | public_api ✅ MCP (snapshot 8480 L, hash 7BA24E…5DBC) | validate-docs-coverage ✅ 0 gaps post-fix | link-check 4 docs: 58 relativos -> 0 rotos + 0 anchors (62 totales) | fmt ✅ clippy --test public_api ✅ · evidencia: ronda 1 completa en task file §Review (ses_f1e5eaf82ffeiJrZgx6UrRp3iK); fixes verificados por grep + checker inline · artefactos: tests/api/public_api.rs + public-api.txt, Cargo.toml/lock, ci-rust.yml, docs/api/DEPRECATIONS.md + COMPATIBILITY.md, VERSIONING.md, UPGRADE.md, task file · invariantes: release-plz unico versionador; 11 superficies intactas; sin push; archivos HARD-02/03 intactos · deuda: exit100 esperado hasta 0.8.0; nightly flotante; normalizacion exit codes MCP · queda_pendiente: ronda 2 review + commit LEAD + OCR delegate rule (advisory)
Próxima tarea si completa: HARD-04
=== END RECITATION ===

=== RECITATION HARD-03 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-03 — continuidad local + release trains (bundle git + política RULES.md §8 + referencia CONTRIBUTING + línea Gate H)
Estado: completed
Última acción: 5/5 steps ✅: script git-backup.ps1 (smoke real exit 0 + verify + clone restore 31 commits + negativos Z:/inside-repo/colisión + prune -Keep); RULES.md §8 (175-191); CONTRIBUTING.md:114-117; .opencode/AGENTS.md:9 (Gate H ✅); reviews ✅ (vanta-review P2-01 APROBADO 0C/0R + vanta-harness); bundle real conservado $HOME\VantaDB-Backups\vantadb-20260927-0127.bundle; learnings registrados
Resultado: OK
Próxima acción: LEAD: commit local VantaDB (scripts/git-backup.ps1 + RULES.md + CONTRIBUTING.md + HARD-03.md) + commit aparte repo .opencode (AGENTS.md:9, Gate H ya verde). Próxima tarea: HARD-04
Contrato: Contrato ✅ 3/3: (1) bundle timestamped + verify exit 0 — real: $HOME\VantaDB-Backups\vantadb-20260927-0127.bundle (28 refs, HEAD 50106b4e, 31 commits locales; clon desde bundle restaura la franja: rev-list --count 40b1dd..develop=31, reproducido por vanta-review); (2) política en RULES.md §8 (+last_reviewed+Fuentes); (3) CONTRIBUTING.md:114-117 referencia script+RULES.md §8. Verificacion: pwsh scripts/git-backup.ps1 ✅ exit 0; git bundle verify ✅ exit 0; validate-docs-coverage ✅ exit 0 (verify_cmd 18.6s). Evidencia: Gate H ✅ (vanta-harness ses_f1ea9677fffeHG7DG7alY3o6CB, smoke propio exit 0); P2-01 ✅ (ses_f1e975170ffemcSiW19TunrzZt, clone+31 commits reproducidos); OCR advisory exit 0 (reglas por grupo, sin bloqueantes). Invariantes: script 100% local (jamás push, Regla 7); single-source (push=Regla 7, continuidad=§8); bundles nunca commiteados. Deuda: commit local delegado al LEAD (instrucción de orquestación); owner action: fijar -Dest externo/sincronizado en AGENTS.md:9 cuando exista la carpeta.
Próxima tarea si completa: HARD-04
=== END RECITATION ===

=== RECITATION HARD-04 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-04: vanta-memory — spec de fachada + triggers de exposición (core-only, Gate P)
Estado: completed
Última acción: 4/4 steps ✅: evidence pack 4/4 ops (refs file:line) + baseline rg bindings (exit 1); §Facade+§Exposure triggers T1–T4 integrados en docs/api/VANTA_MEMORY.md:34,74 (candidate, not published; umbral+fuente+dueño); drift IngestResult→IngestReport; gates: validate-docs-coverage exit 0 (0 gaps), markdownlint 0 issues, OCR advisory exit 0 (.md excluidos → 0 C/H aplicables), rg cierre exit 1; review P2-01 ronda 1 DEGRADADA (doubt-driven self) approve + escalado; commit local delegado al LEAD.
Resultado: OK
Próxima acción: LEAD: commit local docs: de docs/api/VANTA_MEMORY.md + docs/dev/tasks/HARD-04.md (sin push) + ratificar review con vanta-review; próxima tarea HARD-06.
Contrato: verificacion: rg vanta[_-]memory vantadb-python vantadb-ts vantadb-node vantadb-wasm → exit 1 / 0 matches (campaign_verify_cmd HARD-04, x2 pre+post) · pwsh scripts/validate-docs-coverage.ps1 → exit 0 (0 gaps, 2.8s) · npx markdownlint-cli2 docs/api/VANTA_MEMORY.md docs/dev/tasks/HARD-04.md → 0 issues. evidencia: §Facade+§Exposure triggers → VANTA_MEMORY.md:34,74 (2 hits; pre-mortem F1 etiqueta candidate / F2 umbrales medibles / F3 cero bindings); firmas 4/4: auto_capture.rs:84, auto_recall.rs:198, seed/mod.rs:76, ingest/worker.rs:35; OCR spec: 5 reviewables = HARD-02 (fuera de alcance), .md excluded (unsupported_ext). artefactos: docs/api/VANTA_MEMORY.md (+63/−1), docs/dev/tasks/HARD-04.md, %TEMP%\hard04-ocr.json. invariantes: vanta-memory core-only (publish=false; 0 símbolos en bindings — re-verificado al cierre); facade etiquetada 'candidate, not published'; página = referencia manual; FIND-160 sigue abierto para HARD-06; sub-agente sin commit/push. deuda: review P2-01 ronda 1 degradada (sin subagente vanta-review en runtime — escalada al owner/orquestador); commit local pendiente (LEAD). queda_pendiente: LEAD commit + vanta-review ratificación + skill progreso; WIP de HARD-02/05 intacto.
Próxima tarea si completa: HARD-06
=== END RECITATION ===

=== RECITATION HARD-05 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-05 — Entorno local blindado (regla -p, required-features, target dir): regla dura en AGENTS.md, required-features sin skip silencioso, suite unificada sin hang, troubleshooting documentado
Estado: completed
Última acción: 4/4 steps ✅. Regla -p en AGENTS.md (Regla 1 L484 + §Test Suite L430 + coherencia L163/L487/L641; tier de cierre nombrado tras Gate H ronda 1) y test-suite.md scoped (L4/L18); required-features=['rayon'] en stress_protocol (Cargo.toml:476; nextest 2297==2297; --no-run OK); troubleshooting.md sección :8080 (síntoma/causa/fix/comandos); Gate H ronda 2 ✅ + P2-01 ✅ + OCR sin Critical/High + FIND-164 registrada + verify_changed ALL 4 PASS
Resultado: OK
Próxima acción: Handoff al LEAD: commit local (root: Cargo.toml, docs/dev/references/troubleshooting.md, docs/dev/tasks/HARD-05.md, docs/dev/Backlog.md; .opencode repo separado: AGENTS.md, references/test-suite.md) — sin push. Próxima tarea: HARD-06
Contrato: verificacion: (1) rg regla → AGENTS.md:430/484 ✅; (2) required-features + nextest list before=2297/after=2297 + --no-run exit 0 + check exit 0 ✅; (3) cargo test -p vantadb --test cli_tests → 88 passed; cargo test --test cli_tests (raíz, timeout guard 1800s) → 87 passed sin hang (δ=strategate server) ✅; (4) rg 8080/f6c395ef/unificación en troubleshooting.md → 5 hits ✅. evidencia: outputs pegados en task file Steps 1-4 + §Review; artefactos: Cargo.toml:476, .opencode/AGENTS.md:430,484, .opencode/references/test-suite.md:4,18, docs/dev/references/troubleshooting.md:22-43, docs/dev/Backlog.md:262 (FIND-164); invariantes: cli_tests 88/88 canónico, conteo nextest sin regresión, Git policy (sin commit/push del ejecutor); deuda: FIND-164 (bare -p en 5 archivos del harness, fuera de blast radius); queda_pendiente: commit del LEAD + archivar en plan master
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION HARD-06 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-06 — Deuda quick wins (FIND-162, FIND-160, FIND-154, decisión FIND-161 + MEM-55)
Estado: completed
Última acción: Steps 5/5 ✅ + verify completo (verify_changed ALL 4 PASS; validate-docs exit 0; self-test 2/2; YAML OK). 4 fixes + bookkeeping listos en working tree. PENDIENTE SOLO LEAD: review P2-01 (vanta-review, diff docs/api/**) + commit por bloques (mapping en RESULTADO §7; NOTA: VANTA_MEMORY.md/Backlog.md mezclan cambios de HARD-04/HARD-02 sin commitear) + push NO (requiere owner).
Resultado: OK
Próxima acción: LEAD: review P2-01 adversarial + commit por bloques + flip final si aplica. Siguiente task del plan: WIRE-10 (F1) — HARD-06 era la última de F0.
Contrato: verificacion: pwsh scripts/validate-docs-coverage.ps1 ✅ exit 0 (§3 no trivial) · python benchmarks/compare_baseline.py --self-test ✅ 2/2 · pwsh dev-tools/verify_changed.ps1 ✅ ALL 4 PASS · YAML parse ✅ · rg VantaError scripts/validate-docs-coverage.ps1 = 0 ✅ (json = excepción documentada). evidencia: (1) §3 no trivial — 'src/error.rs (Error) — 35 items ok' + negativo FooBarBazXYZ=false; (2) ruido +108.5% no bloquea / regresión +40% bloquea — self-test 2/2 + smoke CLI; (3) decisión FIND-161=B (RFC 9457) — decisions.md L135 + HTTP_API.md:787-793 + ERROR_HANDLING.md:422-426; (4) FIND-153 resuelta — gh run 36094025761 success; (5) 5 módulos documentados — VANTA_MEMORY.md §Operational modules. artefactos: scripts/validate-docs-coverage.ps1, benchmarks/compare_baseline.py, .github/workflows/perf-bench.yml, docs/api/VANTA_MEMORY.md, docs/api/HTTP_API.md, docs/api/ERROR_HANDLING.md, docs/dev/Backlog.md, docs/dev/avance/activo/ci-cd.md, docs/dev/tasks/complete/MEM-55.md, docs/dev/tasks/HARD-06.md, docs/dev/plans/2026-09-26-master-roadmap.md. invariantes: fast gate verde; perf gate sigue bloqueando regresiones reales (self-test); anti_stutter_map.json NO tocado; quality bar no debilitado. deuda: ninguna nueva (FIND-165 = impl RFC 9457 ruteada). queda_pendiente: LEAD — review P2-01 + commit por bloques.
Próxima tarea si completa: WIRE-10 (docs/dev/tasks/WIRE-10.md)
=== END RECITATION ===

=== RECITATION HARD-07 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: HARD-07 — Review gate mecanizado (reviewer_context ≠ author_context)
Estado: completed
Última acción: 4/4 steps ✅: (1) REVIEW_ACCEPT_RULES+validateReviewAccept en config/state-tools.mjs + re-export C0-unified; (2) enforcement en updateTaskStateCore (schema recitation.review, fail-fast reviewBlocked, recordReviewWaiver, guard findTaskById) + fixtures; (3) prosa pipeline-full/pipeline-run/question-gates/RULES §9(+§Cierre, restart note); (4) review-gate.test.mjs T1-T5 + E2E MCP. Gate H r1 ❌→r2 ✅; P2-01 r1 ❌→r2 ✅ APPROVE.
Resultado: OK
Próxima acción: LEAD: commit local selectivo (sin push) + skill progreso + plan file sync. No quedan steps.
Contrato: verificacion: review-gate 5/5 ✅ + bun test 54/54 ✅ + state-persistence 15/15 ✅ + parity exit 0 ✅ + smoke OK ✅ + verify_changed ALL4 ✅ + Gate H ✅ + P2-01 ✅ | evidencia: T2 degraded-no-waiver {updated:false,reviewBlocked:true}; T1/T4 fresh updated:true; T5 numeric-id → recitation7:true; E2E stdio bloqueo OK; probes r2 de ambos revisores | artefactos: .opencode/task-system/{config/state-tools.mjs, C0-unified.mjs, mcp/campaign-server.mjs, mcp/review-gate.test.mjs, mcp/state-persistence.test.mjs, prompts/pipeline-full.md, prompts/pipeline-run.md, prompts/question-gates.md}, docs/dev/workflow/RULES.md §9, docs/dev/tasks/HARD-07.md, docs/dev/Backlog.md (FIND-166/167/168) | invariantes: C0 parity 10 estados intacto; 49 pre-existentes verdes; WIP/claim intactos; waiver=owner registrado (decisions+trace) | deuda: FIND-166/167/168 (Baja, registradas); enforcement activa al reiniciar MCP | queda_pendiente: commit selectivo del LEAD (pipeline-run.md fue scope addendum por regla Gate H medium — disclosed en el task file)
Próxima tarea si completa: WIRE-10
=== END RECITATION ===

=== RECITATION DEF-02 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: 
Estado: completed
Última acción: 
Resultado: 
Próxima acción: 
Contrato: 
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION DEF-01 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: 
Estado: completed
Última acción: 
Resultado: 
Próxima acción: 
Contrato: 
Próxima tarea si completa: 
=== END RECITATION ===

=== RECITATION WIRE-10 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-10 — distribución P0 (install.sh macOS, Colab a Client, hooks sin pwsh) — cierre completado
Estado: completed
Última acción: Reconciliación (5/5 steps ya ✅ desde 2026-09-25; plan stale) + verify scoped completo (fmt/clippy/nextest 11/11/hooks 65/65/notebook e2e 8/8/install harness 8/8) + verify.ps1 ALL 10 PASS + P2-01 fresco APPROVE (ses_f1c06efc4ffeLk9FF3gTqyO9Hw) + task file/plan sincronizados + FIND-169
Resultado: OK
Próxima acción: Ninguno — tarea cerrada; LEAD: commit local del cierre (docs/dev/tasks/WIRE-10.md + plan §Task 8 + Backlog FIND-169) sin push
Contrato: verificacion: verify.ps1 ALL 10 PASS (CARGO_TARGET_DIR=target/session-api01; lock binario target/debug por 8 sesiones MCP live) + install harness pass=8/8 + notebook NOTEBOOK OK executed=8 + hooks PASS=65 FAIL=0 + nextest 11/11 + validate-docs-coverage exit 0; P2-01 fresco APPROVE
evidencia:
  - claim: install.sh macOS portable (sha256sum→shasum fallback) + fail-closed verificado con mocks (GNU/macOS/ninguna/mismatch)
    evidencia: target/session-api01/wire10/step1-verify.ps1 → pass=8 fail=0 (re-ejecutado por reviewer)
    confianza: alta
  - claim: notebook Colab migrado a Client/search y ejecuta e2e (8 celdas)
    evidencia: pwsh target/session-api01/wire10/run-nb.ps1 → NOTEBOOK OK executed=8 skipped=1
    confianza: alta
  - claim: hooks inyectan contexto real sin pwsh (templates + plugin opencode) y tests 65/65
    evidencia: skills/vantadb-mcp/assets/hooks/tests/test-hooks.ps1 → PASS=65 FAIL=0; smoke live mcp-call memory_recall/memory_put OK sin huérfanos; ps1 intacto
    confianza: alta
  - claim: mcp-call one-shot implementado con tests y fixes del review previo (floor_char_boundary, reaps, exit-2)
    evidencia: cargo nextest run -p vantadb --lib cli_handlers::mcp_call --target-dir target/session-api01 → 11 tests run: 11 passed
    confianza: alta
  - claim: P2-01 fresco (HARD-07) — revisión independiente del changeset (steps 5/5 ✅ desde 2026-09-25; plan decía stale)
    evidencia: vanta-review sesión ses_f1c06efc4ffeLk9FF3gTqyO9Hw → APPROVE; Medium no bloqueante → FIND-169
    confianza: alta
artefactos: scripts/install.sh, examples/colab/vantadb_quickstart.ipynb, src/cli_handlers/mcp_call.rs, src/cli.rs, src/bin/vanta-cli.rs, skills/vantadb-mcp/assets/hooks/**, docs/dev/tasks/WIRE-10.md, docs/dev/plans/2026-09-26-master-roadmap.md (§Task 8), docs/dev/Backlog.md (FIND-169), target/session-api01/wire10/** (harnesses)
invariantes: install.sh Linux intacto; narrativa notebook intacta; vanta-mcp-local.ps1 intacto; secretos solo env de sesión
deuda: FIND-169 (Medium pre-existente no bloqueante, registrado en Backlog con repro)
queda_pendiente: commit local del cierre (task file/plan/Backlog) → LEAD; sin push
Próxima tarea si completa: DEF-03
=== END RECITATION ===

=== RECITATION DEF-05 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-05 — North-star + success criteria (SPEC/VISION)
Estado: completed
Última acción: Steps 1–4 ✅ + review fresco P2-01 APPROVE + nit corregido; contrato verificado (coverage exit 0)
Resultado: OK
Próxima acción: LEAD: commit local docs: (SPEC + VISION + task file)
Contrato: North Star byte-idéntica SPEC/VISION + guardrails con medición + validate-docs-coverage exit 0
Próxima tarea si completa: DEF-07
=== END RECITATION ===

=== RECITATION DEF-03 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-03 — Frontera verificable en CI (validate-frontier)
Estado: completed
Última acción: Steps 1–3 + fixes F1/F2/F3 + re-review fresco APPROVE; contrato verificado (exit 0)
Resultado: OK
Próxima acción: LEAD: commit ci: (script + workflow) + docs(tasks) cierre
Contrato: exit 0 repo actual; exit ≠0 perturbado (6 RED); job check-frontier en gate-docs.yml
Próxima tarea si completa: DEF-04
=== END RECITATION ===

=== RECITATION DEF-04 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-04 — Naming freeze 0.7.0→1.0 (ADR + política de alias)
Estado: completed
Última acción: Firma owner Gate P (D1–D3) + review fresco P2-01 APPROVE; contrato verificado (coverage exit 0)
Resultado: OK
Próxima acción: LEAD: commit docs: (ADR-0045 + VERSIONING + task file)
Contrato: ADR accepted + 9/9 artefactos + política de alias + firma owner + ref VERSIONING
Próxima tarea si completa: DEF-05
=== END RECITATION ===

=== RECITATION DEF-08 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-08 — Install SLO + telemetría opt-in + fallback visible
Estado: completed
Última acción: Steps 1-4 ✅ + review fresco APPROVE + fixes O1/O2/O3/N1/N2
Resultado: OK
Próxima acción: LEAD: commit docs: install SLO + specs + cierre F1
Contrato: SLO + telemetría privacy-first + fallback visible especificados (rg anchors + coverage exit 0)
Próxima tarea si completa: WIRE-02 (F2)
=== END RECITATION ===

=== RECITATION 14 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-07 — Presupuesto de alcance: core-promise vs labs
Estado: in-progress
Última acción: Steps 1-3 ✅: inventario 13/13 por North Star; sección 'Scope Budget — core-promise vs labs' + regla admisión→promoción→freeze + watchlist en EXPERIMENTAL_FEATURES.md; SPEC §Frontera referencia §Scope Budget; task file cerrado (steps/recitation/review/CSP)
Resultado: OK
Próxima acción: LEAD: review fresco P2-01 (2-3 filas al azar + fronteras desktop/server) + commit 'docs: scope budget core-promise vs labs (DEF-07)' + progreso al cierre de wave
Contrato: Verificado: rg -n core-promise → hits en SPEC.md y EXPERIMENTAL_FEATURES.md ✅ · pwsh scripts/validate-docs-coverage.ps1 exit 0 ✅ · pwsh scripts/validate-frontier.ps1 exit 0 ✅ · npx markdownlint-cli2 (2 docs + task file) ✅ · diff docs = SPEC + EXPERIMENTAL (2 archivos); commit = LEAD
Próxima tarea si completa: DEF-08 (Task 15)
=== END RECITATION ===

=== RECITATION DEF-07 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: DEF-07 — Presupuesto de alcance: core-promise vs labs
Estado: completed
Última acción: Steps 1-3 ✅ + review fresco APPROVE + fixes; verify rg core-promise ✅
Resultado: OK
Próxima acción: LEAD: commit docs: scope budget + cierre F1
Contrato: tabla 13/13 + regla de inversión + ref SPEC (rg ×2 + coverage + frontier verdes)
Próxima tarea si completa: DEF-08
=== END RECITATION ===

=== RECITATION WIRE-03 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-03 — query_sparse + text-only en 3 bindings + filtros avanzados
Estado: completed
Última acción: Steps 6/6 ✅ + review fresco (❌ R1-R3 → fixes → ✅ delta APPROVE) + gates re-verdes (py 159 · node 41 · ts 323+tsc · core 216)
Resultado: OK
Próxima acción: LEAD: commit feat(bindings) + cierre parcial F2
Contrato: query_sparse Py/TS/Node + text-only (guard) + filtros $op/DateTime py↔js + stubs sync (7/7) + WASM fail-loud
Próxima tarea si completa: WIRE-05
=== END RECITATION ===

=== RECITATION WIRE-04 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-04 — TTL superficie completa (default por colección + sweeper)
Estado: completed
Última acción: Steps 1-9 ✅ + review fresco (❌ docs → fixes → ✅ delta APPROVE) + snapshot regen + public_api verde + coverage 0 gaps
Resultado: OK
Próxima acción: LEAD: commit feat(ttl) (staged; espera hook fmt libre de WIRE-02/03) + cierre wave F2
Contrato: E2E TTL (404 lazy + purga física) + default por colección (no backfill) + sweeper físico (nodo+scalar) — 3/3
Próxima tarea si completa: WIRE-05
=== END RECITATION ===

=== RECITATION WIRE-02 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-02 — MCP: enforce de perfil + default agent + fusión 85→79 + doc
Estado: completed
Última acción: Steps 5/5 ✅ + review fresco (❌ 6 meta-tests → fixes → ✅ delta APPROVE) + 239/239 + commit 80f91bd8
Resultado: OK
Próxima acción: LEAD: cierre parcial F2 (plan sync)
Contrato: enforce -32601 exacto + default agent 37≤45 + fusión 79 no-breaking + aliases dispatchables (4/4)
Próxima tarea si completa: WIRE-05
=== END RECITATION ===

=== RECITATION WIRE-07 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-07 — Refactors: ffi-core (OpGate×3) + server↔cli + bookkeeping
Estado: completed
Última acción: Steps 6/6 ✅ + review adversarial (❌ F1 evidencia pyd → fix LEAD → ✅ delta APPROVE); 6 slices A-F
Resultado: OK
Próxima acción: LEAD: commits por slices A-F + cierre wave F2b
Contrato: OpGate único + hoja 0-deps + server sin cli (unit-graph) + BOUNDARIES F3X + suites verdes
Próxima tarea si completa: WIRE-08
=== END RECITATION ===

=== RECITATION WIRE-05 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-05 — Entity linking determinista + boost RRF
Estado: completed
Última acción: Steps 6/6 ✅ + review fresco APPROVE + Optionals aplicados (non_exhaustive/wording); 112/112 área
Resultado: OK
Próxima acción: LEAD: commit feat(entity) (staged) + cierre wave F2b; R-1 snapshot al cierre
Contrato: F-S determinista + boost opt-in/reversible/proveniencia + OFF byte-idéntico + suites verdes
Próxima tarea si completa: WIRE-06
=== END RECITATION ===

=== RECITATION WIRE-06 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-06 — Batching productizado (group-commit opt-in)
Estado: completed
Última acción: Steps 7/7 ✅ + review adversarial APPROVE (rerun pareado min 5.93×) + fixes (alternación por muestra + clamp sanitized + dead-code cfg)
Resultado: OK
Próxima acción: LEAD: commit perf(storage) + cierre wave F2b
Contrato: ≥5× (pareada 5.92× mediana) + p99 por construcción + integridad 220/220 + nightly gate
Próxima tarea si completa: WIRE-07
=== END RECITATION ===

=== RECITATION WIRE-08 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: WIRE-08 — Range/group_by + cursor + RRF-CBO + MMR
Estado: completed
Última acción: Steps 8/8 ✅ + review adversarial (❌ C1/R1/R2 → fixes → ✅ delta APPROVE); 2369/2369
Resultado: OK
Próxima acción: LEAD: commit feat(search) + cierre F2 (23/50) + FINDs 183/184/185
Contrato: range/group_by + cursor best-effort documentado/testeado + MMR + RRF-CBO + paridad Milvus/Qdrant + rewriting DEFER
Próxima tarea si completa: MGR-10 (F3)
=== END RECITATION ===

=== RECITATION MGR-13 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: MGR-13 — research-doc: cuarentena
Estado: completed
Última acción: Steps 4/4 ✅ + review fresco APPROVE + fixes textuales; doc §0-§8 + threat model
Resultado: OK
Próxima acción: LEAD: commit docs (hecho) → ronda consolidada → SCH-01
Contrato: estados+transiciones (dueño+trigger) + threat model write-time por superficie
Próxima tarea si completa: SCH-01
=== END RECITATION ===

=== RECITATION MGR-10 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: MGR-10 — research-doc: bitemporalidad (dim 5)
Estado: completed
Última acción: Steps 4/4 ✅ + review fresco APPROVE + nits aplicados; doc §0-§7 + Cierre MGR
Resultado: OK
Próxima acción: LEAD: commit docs (hecho) → ronda consolidada de preguntas owner → SCH-01
Contrato: modelo valid/transaction + tradeoffs + plan de migración determinista (4 formatos)
Próxima tarea si completa: SCH-01
=== END RECITATION ===

=== RECITATION MGR-12 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: MGR-12 — research-doc: confianza (dim 6)
Estado: completed
Última acción: Steps 4/4 ✅ + review fresco APPROVE + nits 5-8 aplicados; doc §0-§10 + Cierre MGR
Resultado: OK
Próxima acción: LEAD: commit docs (hecho) → ronda consolidada de preguntas owner → SCH-01
Contrato: modelo asserted/derived (V1-V5 + min(padres)×0.9) + last_validated + mapeo canónico + calibración diferida VER-08
Próxima tarea si completa: SCH-01
=== END RECITATION ===

=== RECITATION SCH-01 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-01 — Plan único + ADR de migración (ADR-0046, 0.8.0)
Estado: completed
Última acción: Steps 4/5 ✅ + review fresco APPROVE + fixes + FIRMA OWNER (set completo); ADR-0046 accepted + commit
Resultado: OK
Próxima acción: LEAD: claim SCH-02 (schema v2) — desbloqueada
Contrato: ADR aceptado (campos + semántica + alcance 0.8.0 + plan migración + compat) + revisión owner ✅
Próxima tarea si completa: SCH-02
=== END RECITATION ===

=== RECITATION SCH-02 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-02 — Schema v2 (bitemporal + confidence + quarantined + backfill)
Estado: completed
Última acción: Steps 9/9 ✅ — schema v2 + migración determinista + fixes R1/R2/R3; review adversarial 2 rondas ✅; commit 7af34366
Resultado: OK
Próxima acción: Wave F3.3: claim SCH-03/04/05 (desbloqueadas por schema v2)
Contrato: schema v2 #[serde(default)] v1-compat + migración determinista + backfill + roundtrip v1/v2 + reopen (2403/2403)
Próxima tarea si completa: SCH-03
=== END RECITATION ===

=== RECITATION SCH-04 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-04 — Scores asserted/derived consumibles (slice 0.8.0)
Estado: completed
Última acción: 8/8 ✅ + review APPROVE + fixes post-review (MCP schema min/max + citas); commit 932b1211 (wave F3.3a)
Resultado: OK
Próxima acción: Wave F3.3b: SCH-05
Contrato: confianza visible SDK/HTTP/MCP + 4 bindings (stubs/d.ts) + filtro min_confidence opt-in (fingerprint/cursor) sin cambio de ranking
Próxima tarea si completa: SCH-05
=== END RECITATION ===

=== RECITATION SCH-03 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-03 — Queries AS OF/point-in-time + filtros valid_at + exclude_superseded
Estado: completed
Última acción: 8/8 ✅ + review APPROVE + Optional-2 fix + delta ✅; commit 932b1211 (wave F3.3a)
Resultado: OK
Próxima acción: Wave F3.3b: claim SCH-05
Contrato: AS OF IQL (IQL_VERSION 2 + feature-detect + duplicado rechazado) + as_of_ms/valid_window search/list + exclude_superseded extendido + cursor fingerprint (2438/2438)
Próxima tarea si completa: SCH-05
=== END RECITATION ===

=== RECITATION SCH-05 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-05 — Cuarentena + abstención + trust-aware retrieval
Estado: completed
Última acción: 7/7 ✅ + review adversarial 2 rondas (❌ F1-F6/N → batch → delta ✅); commit 83d65518
Resultado: OK
Próxima acción: Wave F3.4: claim SCH-06 (chaos/tests)
Contrato: default-exclude + gates de inyección + transiciones T1-T5 con audit + sticky raw transport + abstención explícita (2464/2464 · containment 24/24 · MCP 245)
Próxima tarea si completa: SCH-06
=== END RECITATION ===

=== RECITATION SCH-06 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-06 — Tests: migración determinista, time-travel, roundtrip, chaos
Estado: completed
Última acción: 7/7 ✅ + review fast APPROVE; suites nuevas + wiring CI; FIND-186 registrado
Resultado: OK
Próxima acción: Wave F3.5: claim SCH-07 (superficies + docs/api)
Contrato: migración determinista byte-idéntica + time-travel + roundtrip v1↔v2 + chaos mid-migración + bordes (CI exacto 2/2 · 2475/2475)
Próxima tarea si completa: SCH-07
=== END RECITATION ===

=== RECITATION SCH-07 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-07 — Superficies: bindings/server/MCP/IQL + docs/api mismo-PR
Estado: completed
Última acción: 7/7 ✅ + review adversarial APPROVE + batch (Opt 1/2/4 + nit) + delta ✅
Resultado: OK
Próxima acción: Wave F3.6: claim SCH-08 (corte 0.8.0)
Contrato: campos v2 + params + abstención cruzan 8 superficies (mismos nombres) + docs/api 0 gaps + public_api +4 auditado
Próxima tarea si completa: SCH-08
=== END RECITATION ===

=== RECITATION SCH-08 ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: SCH-08 — Corte 0.8.0: migration guide + CHANGELOG + release notes
Estado: completed
Última acción: 7/8 ✅ (parte local) + review ❌→fixes→delta ✅; commit b9296909 (mensaje R1) + R2 aplicado; gate F3 ✅ local
Resultado: OK
Próxima acción: Bloques F4 (REGLA) → wave F4; release real = lane owner (push)
Contrato: UPGRADE.md §0.8.0 + auditoría release-plz (R1/R2) + release notes draft + handoff owner; CHANGELOG nunca a mano
Próxima tarea si completa: F4
=== END RECITATION ===
