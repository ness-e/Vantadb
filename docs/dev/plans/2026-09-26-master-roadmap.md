# Plan de Ejecución: Master Roadmap VantaDB — de 0.7.x a 1.0 (fuente única)

> **Campaign ID:** ed20beae-edf6-42f5-b41f-e8519830d6cb
> **Inicio:** 2026-09-26
> **Estado:** ⏳ EN PROGRESO (0/49 — plan recién creado)
> **Fuente:** `docs/dev/Backlog.md` (P52–P59 + FIND-*) + planes absorbidos (`2026-09-24-post-investigacion-integral.md` W2–W7, `2026-09-24-estabilizacion-pendiente.md`, `2026-09-24-harness-gaps.md`, `2026-09-24-sesion-continuidad.md`, `2026-09-20-estabilizacion-total.md`) + `docs/dev/strategy/` (13 docs) + Notion "VantaDB Docs" (~28 subpáginas) + investigación de riesgos 2026-09-26 (4 sub-agentes R1–R4, multi-fuente)
> **Autonomous:** false — el owner gatea push/merge/release. **Push a `develop`: al completar el plan con todo validado/verificado** (decisión owner 2026-09-26, ver §Política de commits/push)
> **Modo:** PLAN → ejecución con `/pipeline run docs/dev/plans/2026-09-26-master-roadmap.md`
> **Nota:** este plan ABSORBE los 5 planes activos anteriores (archivados a `docs/dev/plans/archive/`). Única fuente de plan.

## SDP (skills del plan — campaign_discover_skills_v2 phase=PLAN)

SDP: `campaign-executor` · `progreso` · `writing-plans` · `planning-and-task-breakdown` · `api-and-interface-design` · `spec-driven-development` · `documentation-and-adrs` · `writing-guidelines`
(Adicionales por fase: ver §Herramientas, skills y MCP.)

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 49 |
| 🟡 DEFER | cola restante del Backlog (ver §Cola restante — disposición) |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 (deps internas por fase; ninguna externa al plan) |

Status: ⬆️ uphill = 5 (F2–F6 con bloques esenciales que se COMPLETAN al nivel F0/F1 al iniciar cada fase — REGLA en §F2–F6) · ⬇️ downhill = 44 (F0/F1 con detalle completo, listos para ejecutar)

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

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `release-plz.toml` · `docs/api/VERSIONING.md` · `docs/api/DEPRECATIONS.md` (nuevo) · `docs/api/COMPATIBILITY.md` (nuevo) · `docs/user/operations/UPGRADE.md` (existente) · `Cargo.toml` (metadata `cargo-semver-checks` lints) · `tests/api/public_api.rs` + `public-api.txt` (nuevo snapshot) · `.github/workflows/ci-rust.yml`
- **Verificación real:** ✅ CÓDIGO-REAL — `semver_check = true` ya activo (`release-plz.toml`); `VERSIONING.md` lista 11 superficies (API-09, commit `032cbd0f`); `ci-rust.yml:92-128` ya tiene job `semver-checks` pero **main-only** → gap real: gate en PR + snapshot `cargo-public-api` + `DEPRECATIONS.md`/`COMPATIBILITY.md` (R1: semver-checks solo ve Rust → falta gate por superficie).
- **Gate Justificación:** rails baratos con 0 usuarios (R1: DuckDB/Polars/Qdrant/Milvus + cargo-semver-checks/cargo-public-api); cierra la desventaja "breaking by design" antes de que haya consumidores.
- **Gate Result:** ✅ DO
- **Contrato:** "`cargo semver-checks check-release` exit 0 (o findings triados y documentados) Y `cargo test -p vantadb --test public_api` verde con snapshot commiteado Y `DEPRECATIONS.md` + `COMPATIBILITY.md` + `UPGRADE.md` existen con 0 links rotos (validate-docs-coverage verde)"
- **Task file:** `docs/dev/tasks/HARD-01.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H para partes `.opencode/`)
- **Archivos clave:** `dev-tools/verify.ps1` · `.config/nextest.toml` · `.github/workflows/ci-rust.yml` (+ nuevo nightly) · `docs/dev/operations/CI_POLICY.md` · `.opencode/task-system/prompts/pipeline-full.md` (tiering de review) · `CONSTRAINTS.md` (quality bar — actualizar con decisión owner)
- **Verificación real:** ✅ CÓDIGO-REAL — `verify.ps1` corre coverage bloqueante (`--fail-under-lines 60`, reparado en API-09); fast gate actual con 11 pasos; sin nightly de certificación pesada; review P2-01 sin tiering. Decisión owner 2026-09-26: aplicar **(a)+(b)+(c) completo** (coverage→reporte+presupuesto, review risk-tiered, nightly).
- **Gate Justificación:** R1 (Google eng: coverage lossy; fast<5min vs nightly) + decisión owner explícita; reduce costo de mantenimiento sin perder señal.
- **Gate Result:** ✅ DO
- **Contrato:** "coverage ya no bloquea (reporte + presupuesto por directorio documentado en CI_POLICY) Y review risk-tiered documentado y activo en prompts (solo diffs `docs/api|sdk|parser|storage|wire` → adversarial; resto verify fast) Y workflow nightly de certificación pesada existe Y fast gate medido <5min"
- **Task file:** `docs/dev/tasks/HARD-02.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5-1d · **Prioridad:** 🔴
- **Ruta:** vanta-lead
- **Archivos clave:** `scripts/git-backup.ps1` (nuevo: `git bundle create` + `verify`) · `docs/dev/workflow/RULES.md` (política) · `CONTRIBUTING.md` · `docs/dev/plans/2026-09-26-master-roadmap.md` (§Política — referencia)
- **Verificación real:** ✅ CÓDIGO-REAL — 26+ commits locales sin push (medido `git rev-list --count origin/develop..develop`); sin script de backup; política de push dispersa (AGENTS.md Regla 7 + flujo mínimo). Decisión owner: push al final del plan; bundles como mitigación (R2: lección GitLab 2017).
- **Gate Justificación:** riesgo real de pérdida local sin CI visible; mitigación barata y sin cambiar el control del owner.
- **Gate Result:** ✅ DO
- **Contrato:** "`pwsh scripts/git-backup.ps1` crea bundle con timestamp y `git bundle verify` exit 0 Y política de commits/push/trenes documentada en `docs/dev/workflow/RULES.md` Y `CONTRIBUTING.md` referencia el script"
- **Task file:** `docs/dev/tasks/HARD-03.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🟡
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/api/VANTA_MEMORY.md` · `docs/dev/Backlog.md` (FIND-160 co-referencia)
- **Verificación real:** ✅ CÓDIGO-REAL — crate `publish=false`, 423 ítems pub, 9 módulos (R3); ya consumible vía MCP (`memory_recall`/`memory_search`), proxy y desktop; 0 símbolos en bindings (Gate P core-only). Gap: sin spec de fachada ni triggers documentados.
- **Gate Justificación:** R3 — mantener core-only (costo 0) + prep sin código; triggers medibles evitan exponer por entusiasmo.
- **Gate Result:** ✅ DO
- **Contrato:** "`docs/api/VANTA_MEMORY.md` incluye §Facade (capture/recall/seed/ingest, firma conceptual + degradación) Y §Exposure triggers T1–T4 (≥5 pedidos externos / adapter ICP-03 bloqueado / 2 releases sin breaking / stranger-tests confirman demanda local-first) Y validate-docs-coverage verde Y 0 símbolos nuevos en bindings"
- **Task file:** `docs/dev/tasks/HARD-04.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H)
- **Archivos clave:** `.opencode/AGENTS.md` (regla dura) · `Cargo.toml` (`[[test]] required-features`) · `docs/dev/references/troubleshooting.md` (entrada del incidente) · `dev-tools/verify.ps1` (ya scoped — verificar)
- **Verificación real:** ✅ CÓDIGO-REAL — incidente 2026-09-26: `cargo test` sin `-p` desde raíz → default-members unifican features (`vantadb-server` pide `server`) → test arranca HTTP real y cuelga :8080 (mitigado: `#[cfg(not(feature = "server"))]` en `tests/cli_tests.rs`, commit `f6c395ef`); verify.ps1 ya usa `-p vantadb`; nextest filters calificados (BND-06).
- **Gate Justificación:** R2 — riesgo vivo en invocaciones ad-hoc; regla + defensa dura eliminan la clase de fallo.
- **Gate Result:** ✅ DO
- **Contrato:** "`AGENTS.md` prohíbe `cargo test`/`nextest` sin `-p` desde la raíz Y targets feature-gated tienen `required-features` (sin skip silencioso) Y suite unificada (`cargo test --test cli_tests`) completa sin hang Y `troubleshooting.md` documenta síntoma/causa/fix"
- **Task file:** `docs/dev/tasks/HARD-05.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 2d · **Esfuerzo:** 🟡 1d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ docs para FIND-160)
- **Archivos clave:** `scripts/validate-docs-coverage.ps1` (FIND-162) · `docs/api/VANTA_MEMORY.md` (FIND-160) · `.github/workflows/perf-bench.yml` (FIND-154) · `docs/api/ERROR_HANDLING.md` + `docs/api/HTTP_API.md` (FIND-161) · `docs/dev/Backlog.md` (filas) · `docs/dev/tasks/MEM-55*` (bookkeeping)
- **Verificación real:** ✅ CÓDIGO-REAL — FIND-162: sección 3 del script inerte tras rename `VantaError`→`Error` (`validate-docs-coverage.ps1:102` vs `src/error.rs:137`); FIND-160: módulos sin doc (dream/gateway/ingest/services); FIND-154: gate perf falso positivo por varianza (rojo en main, verde en develop, dispatch `36101773914` vs `36094025761`); FIND-161: envelope real `{success:false, error|data, code(,hint)}` documentado pero RFC 9457 pendiente de decisión; FIND-153 ya resuelto (`114f55f0`) → solo cierre.
- **Gate Justificación:** baratos (15min-1d) y desbloquean el "CI verde" que FASE-A necesita (R3: FIND-154 contamina el checklist).
- **Gate Result:** ✅ DO
- **Contrato:** "FIND-162: check reparado o retirado con motivo + `rg VantaError scripts/` = 0 (o excepción documentada) Y FIND-160: secciones por módulo O exclusión motivada Y FIND-154: gate ya no falsea (tolerancia/quarantine documentada) Y FIND-161: decisión registrada (ADR o decision-memory) + docs coherentes; filas del Backlog actualizadas/cerradas"
- **Task file:** `docs/dev/tasks/HARD-06.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
- **Cynefin:** 🟦 obvio — fixes acotados.
- **Top 3 riesgos:** 1. FIND-154 sin repro estable → fix a ciegas · 2. FIND-161 decisión que toca superficies · 3. mezclar 4 fixes en un commit.
- **Pre-mortem:** F1: perf gate con varianza irreducible → quarantine + issue; F2: RFC 9457 adoptado a medias → decisión binaria documentada; F3: commit no atómico → 1 commit por FIND (4 commits, 1 task).
- **Stop conditions:** FIND-154 si el fix requiere reescribir el gate → DEFER con evidencia.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |--------------|--------|------------------------|---------------|
  | 🟡×🟡 | FIND-154 reincide | quarantine + métrica de varianza | 2 falsos positivos más |
  | 🟢×🟡 | FIND-161 ambiguo | opciones + decisión owner si hay tradeoff | Gate P |
- **Uphill/Downhill:** ⬆️ 1 (FIND-161) / ⬇️ 5 steps
- **DoD task:** contrato ✅ · task file sync · recitation · **Notas:** FIND-153 se evalúa junto a FIND-154; MEM-55 (task file huérfano) → catalogar o cerrar.

### Task 7: HARD-07 — Review gate mecanizado (reviewer_context ≠ author_context)

- **Appetite:** max 1d · **Esfuerzo:** 🟢 0.5d · **Prioridad:** 🔴
- **Ruta:** vanta-lead (+ Gate H; revisión vanta-harness)
- **Archivos clave:** `.opencode/task-system/state-tools.mjs` (invariante REVIEW→ACCEPT) · `.opencode/task-system/prompts/pipeline-full.md` · `.opencode/task-system/prompts/question-gates.md` · `docs/dev/workflow/RULES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — incidente API-09: §Review registró review degradado (mismo contexto) habiendo subagentes frescos; se corrigió manualmente con review fresco (sesión `ses_f2329896bffeh83Kc70QBccoVe`). Gap: la degradación no bloquea el ACCEPT.
- **Gate Justificación:** R2 (Anthropic best practices + self-preference bias arXiv 2404.13076 + CriticGPT): sin enforcement mecánico, el gate se degrada en silencio.
- **Gate Result:** ✅ DO
- **Contrato:** "REVIEW→ACCEPT exige `reviewer_context ≠ author_context` (o waiver registrado que BLOQUEA el ACCEPT) Y simulación de review degradado NO permite ACCEPT (caso de prueba en el task file) Y Gate H verde"
- **Task file:** `docs/dev/tasks/HARD-07.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🔴
- **Ruta:** vanta-worker
- **Archivos clave:** `scripts/install.sh` · `examples/colab/vantadb_quickstart.ipynb` · hooks (claude/codex/cursor/opencode templates) · `src/cli*.rs` (`mcp-call` superficie)
- **Verificación real:** ✅ CÓDIGO-REAL — task file EXISTE y está ⏳ IN PROGRESS (`docs/dev/tasks/WIRE-10.md`, 138 líneas, 5 steps pendientes, Spec P/D resuelta 2026-09-25). Continuar desde el primer step ⬜ PENDING — NO re-hacer steps ✅.
- **Gate Justificación:** P0 de distribución (roadmap W2/A2); sin esto, macOS/Colab/hooks rotos.
- **Gate Result:** ✅ DO
- **Contrato:** ver `docs/dev/tasks/WIRE-10.md` §Contrato (leer al ejecutar; el task file es fuente de verdad del contrato fino)
- **Task file:** `docs/dev/tasks/WIRE-10.md` (existente)
- **Estado:** ⏳ IN PROGRESS · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs (+ Gate P owner)
- **Archivos clave:** `SPEC.md` / `docs/SPEC.md` (según exista) · `README.md` · `VISION.md` · `docs/dev/strategy/` (fuente)
- **Verificación real:** ✅ CÓDIGO-REAL — conflicto detectado en investigación (R4: README "RAG/edge" vs SPEC; P55 fila DEF-01). Jerarquía propuesta: un núcleo ("SQLite para agentes") + 3 puertas ICP.
- **Gate Justificación:** bloquea coherencia de DEF-05/DEF-07 y los one-pagers ICP; decisión de producto → Gate P con owner.
- **Gate Result:** ✅ DO
- **Contrato:** "SPEC/README/VISION declaran la MISMA jerarquía (1 núcleo + 3 puertas) sin contradicciones Y Gate P registrado (owner) Y validate-docs-coverage verde"
- **Task file:** `docs/dev/tasks/DEF-01.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 3d · **Esfuerzo:** 🟡 2-3d · **Prioridad:** 🟠
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/EXPERIMENTAL_FEATURES.md` · código/features referenciados (verificación por fila)
- **Verificación real:** ✅ CÓDIGO-REAL — fixes inmediatos de 4 claims falsos ya aplicados 2026-09-24; falta versionar a 0.7.0 + verificar cada fila contra código + categorías labs (proxy/desktop/web/memory).
- **Gate Justificación:** claims honestos (Regla 11) + base de DEF-07.
- **Gate Result:** ✅ DO
- **Contrato:** "cada fila verificada contra código/feature real (evidencia file:line o comando) Y versión 0.7.0 declarada Y categorías labs explícitas Y 0 claims sin respaldo (Regla 11)"
- **Task file:** `docs/dev/tasks/DEF-02.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 2d · **Esfuerzo:** 🟡 2d · **Prioridad:** 🟠
- **Ruta:** vanta-lead
- **Archivos clave:** `scripts/validate-docs-coverage.ps1` (extender) o `scripts/validate-frontier.ps1` (nuevo) · `.github/workflows/*` (step) · `docs/EXPERIMENTAL_FEATURES.md` (frontera declarada)
- **Verificación real:** ✅ CÓDIGO-REAL — validate-docs-coverage existe (0 gaps hoy); gap: no valida la frontera contra features Cargo/rutas reales (P55).
- **Gate Justificación:** convierte "frontera declarada" en gate mecánico (cierra drift día-1, R1).
- **Gate Result:** ✅ DO
- **Contrato:** "script valida cada fila Production-facing contra feature/ruta real (exit ≠0 si no existe) Y corre en CI Y verde en main actual"
- **Task file:** `docs/dev/tasks/DEF-03.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs (+ firma owner — ADR Regla 5)
- **Archivos clave:** `docs/dev/architecture/adr/NNN_naming_freeze.md` (nuevo) · `docs/api/VERSIONING.md` · `docs/api/DEPRECATIONS.md` (coordina HARD-01)
- **Verificación real:** ✅ CÓDIGO-REAL — 9 artefactos a congelar (vantadb, vantadb-py, vantadb-node, vantadb-ts, vantadb-server, vantadb-mcp, vanta-cli, vanta-proxy, vanta-memory); API-01..09 completadas (base).
- **Gate Justificación:** congelar nombres AHORA (pre-usuarios) es gratis; después cuesta migraciones (R1).
- **Gate Result:** ✅ DO
- **Contrato:** "ADR con los 9 artefactos congelados + política de alias/deprecación (alias con fecha de remoción, nunca rename silencioso) Y firma owner (Regla 5) Y referencia desde VERSIONING.md"
- **Task file:** `docs/dev/tasks/DEF-04.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🔴
- **Ruta:** vanta-docs
- **Archivos clave:** `SPEC.md`/`VISION.md` · `docs/dev/strategy/ROADMAP.md` (métrica) · proxy (medición)
- **Verificación real:** ✅ CÓDIGO-REAL — North Star propuesta en P55: "agentes activos que recuperan una memoria con éxito en ventana de 7 días" (medible en proxy: sesiones MCP con put+search la misma semana); falta formalizar en SPEC/VISION + guardrails.
- **Gate Justificación:** toda métrica de F5 (ICP) y F6 (anuncio) cuelga de esto.
- **Gate Result:** ✅ DO
- **Contrato:** "SPEC/VISION declaran North Star + guardrails (0 hallazgo crítico seguridad, latencia) Y la métrica es medible con el proxy actual (comando/consulta documentada)"
- **Task file:** `docs/dev/tasks/DEF-05.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠
- **Ruta:** vanta-docs
- **Archivos clave:** `docs/dev/strategy/` · `SPEC.md` · `docs/EXPERIMENTAL_FEATURES.md`
- **Verificación real:** ✅ CÓDIGO-REAL — P55: categorizar proxy/desktop/web/memory (labs vs core-promise) + decidir qué consume el presupuesto "1 comando, 0 config".
- **Gate Justificación:** evita que labs consuman el presupuesto de la promesa central (R4: DEF-07 pendiente).
- **Gate Result:** ✅ DO
- **Contrato:** "tabla categorizada (core-promise vs labs) por superficie Y regla de inversión documentada Y referencia desde SPEC"
- **Task file:** `docs/dev/tasks/DEF-07.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

- **Appetite:** max 1d · **Esfuerzo:** 🟢 1d · **Prioridad:** 🟠
- **Ruta:** vanta-docs (+ worker si toca código de fallback)
- **Archivos clave:** `docs/user/QUICKSTART.md` · `docs/dev/strategy/DISTRIBUTION.md` · (si aplica) código de fallback ORT/modelo
- **Verificación real:** ✅ CÓDIGO-REAL — P55: definir SLO de instalación (tiempo máx al primer recall, tasa de éxito), telemetría opt-in privacidad-first y fallback visible cuando ORT/modelo no cargan.
- **Gate Justificación:** FASE-A (stranger tests) mide exactamente esto; sin SLO no hay criterio de éxito del anuncio.
- **Gate Result:** ✅ DO
- **Contrato:** "SLO definido (tiempo al primer recall + tasa de éxito) con método de medición Y telemetría opt-in especificada (privacy-first, default off) Y fallback visible especificado (mensaje + docs)"
- **Task file:** `docs/dev/tasks/DEF-08.md`
- **Estado:** ⬜ PENDING · **Branch:** develop · **Commit:**
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

### Task 16: WIRE-02 — MCP: enforce de perfil en `tools/call` + default `agent` + fusión 87→~65 + fix doc `MCP.md:254`
- **Dep:** API-04 ✅ · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "perfil enforced (test) + default agent + conteo tools fusionado documentado + `MCP.md:254` corregido" · **Task file:** `docs/dev/tasks/WIRE-02.md` · ⬜ PENDING

### Task 17: WIRE-03 — `query_sparse` + text-only en 3 bindings + filtros avanzados (`$and`/`$or`, range/datetime)
- **Dep:** API-02 ✅ · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "query_sparse en Py/TS/Node con tests + filtros avanzados verdes" · **Task file:** `docs/dev/tasks/WIRE-03.md` · ⬜ PENDING

### Task 18: WIRE-04 — TTL superficie completa (server HTTP + default por colección + sweeper)
- **Dep:** MGR-09 · 🟢 1-2d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "TTL en HTTP + default por colección + sweeper al índice con test de expiración" · **Task file:** `docs/dev/tasks/WIRE-04.md` · ⬜ PENDING

### Task 19: WIRE-05 — Entity linking determinista + boost multi-señal en RRF (Fellegi-Sunter + embeddings)
- **Dep:** MGR-05 · 🟡 3-4d · 🔴 · **Ruta:** vanta-engine · **Contrato:** "matching multi-señal con benchmark + fusión reversible con proveniencia" · **Task file:** `docs/dev/tasks/WIRE-05.md` · ⬜ PENDING

### Task 20: WIRE-06 — Batching productizado (`insert_lock` → segmentos appendables)
- **Dep:** FUT-12-spec · 🔴 1sem · 🔴 · **Ruta:** vanta-engine · **Contrato:** "batching con throughput medido (Regla 9: before/after) + tests de integridad" · **Task file:** `docs/dev/tasks/WIRE-06.md` · ⬜ PENDING

### Task 21: WIRE-07 — Refactors: crate `ffi-core` (OpGate×3), trait-split storage↔index, desacoplar `server→cli`
- **Dep:** — · 🔴 1-2sem · 🟠 · **Ruta:** vanta-arch · **Contrato:** "refactor sin cambio de comportamiento (tests verdes) + deuda P2 pagada" · **Task file:** `docs/dev/tasks/WIRE-07.md` · ⬜ PENDING

### Task 22: WIRE-08 — Range/group_by + cursor con resume + RRF en CBO + rewriting + MMR
- **Dep:** API-06 ✅ · 🟡 3-5d · 🟠 · **Ruta:** vanta-engine · **Contrato:** "paridad Milvus/Qdrant documentada + tests de cursor/range" · **Task file:** `docs/dev/tasks/WIRE-08.md` · ⬜ PENDING

### Task 23: MGR-10 — research-doc: bitemporalidad (dim 5)
- **Dep:** — · 🟡 2-3d · 🔴 · **Ruta:** vanta-arch · **Contrato:** "research-doc cerrado (diseño + tradeoffs + migración) listo para SCH-01" · **Task file:** `docs/dev/tasks/MGR-10.md` · ⬜ PENDING

### Task 24: MGR-12 — research-doc: confianza (dim 6)
- **Dep:** — · 🟡 2-3d · 🔴 · **Ruta:** vanta-arch · **Contrato:** "research-doc cerrado (modelo de confianza asserted/derived)" · **Task file:** `docs/dev/tasks/MGR-12.md` · ⬜ PENDING

### Task 25: MGR-13 — research-doc: cuarentena
- **Dep:** — · 🟡 2-3d · 🔴 · **Ruta:** vanta-arch · **Contrato:** "research-doc cerrado (estados + transiciones + threat model)" · **Task file:** `docs/dev/tasks/MGR-13.md` · ⬜ PENDING

### Task 26: SCH-01 — Plan único + ADR de migración (dims 5-6; alcance 0.7.0 vs v1.0)
- **Dep:** MGR-10/12/13 · 🟡 2-3d · 🔴 · **Ruta:** vanta-arch · **Contrato:** "plan+ADR con alcance explícito + revisión owner" · **Task file:** `docs/dev/tasks/SCH-01.md` · ⬜ PENDING

### Task 27: SCH-02 — Schema v2 (bitemporal + confidence + quarantined + backfill)
- **Dep:** SCH-01 · 🔴 3-5d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "migración determinista + backfill + roundtrip verde" · **Task file:** `docs/dev/tasks/SCH-02.md` · ⬜ PENDING

### Task 28: SCH-03 — Queries `AS OF`/point-in-time + filtros `valid_at` + `exclude_superseded`
- **Dep:** SCH-02 · 🟡 2-3d · 🔴 · **Ruta:** vanta-engine · **Contrato:** "time-travel query con tests deterministas" · **Task file:** `docs/dev/tasks/SCH-03.md` · ⬜ PENDING

### Task 29: SCH-04 — Scores asserted/derived consumibles (slice 0.7.0)
- **Dep:** SCH-02 · 🟡 2-3d · 🟠 · **Ruta:** vanta-engine · **Contrato:** "scores en ranking/UI + derivación completa diferida a v1.0 documentada" · **Task file:** `docs/dev/tasks/SCH-04.md` · ⬜ PENDING

### Task 30: SCH-05 — Cuarentena + abstención + trust-aware retrieval (threat model write-time)
- **Dep:** SCH-02 · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "cuarentena operativa + abstención con tests + threat model citado" · **Task file:** `docs/dev/tasks/SCH-05.md` · ⬜ PENDING

### Task 31: SCH-06 — Tests: migración determinista, time-travel, roundtrip export/import, chaos
- **Dep:** SCH-02..05 · 🟡 2-3d · 🔴 · **Ruta:** vanta-chaos · **Contrato:** "suite de migración/chaos verde + crash-recovery" · **Task file:** `docs/dev/tasks/SCH-06.md` · ⬜ PENDING

### Task 32: SCH-07 — Superficies: bindings/server/MCP/IQL + docs/api mismo-PR
- **Dep:** SCH-02..06 · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "4 bindings + server + MCP + IQL exponen schema v2 + docs sync (Regla 3)" · **Task file:** `docs/dev/tasks/SCH-07.md` · ⬜ PENDING

### Task 33: SCH-08 — Corte 0.8.0: migration guide + CHANGELOG + release notes
- **Dep:** SCH-07 · 🟢 1d · 🔴 · **Ruta:** vanta-docs · **Contrato:** "migration guide publicado + release notes + corte vía release-plz (nunca manual)" · **Task file:** `docs/dev/tasks/SCH-08.md` · ⬜ PENDING

### Task 34: VER-07 — Dreams: dry-run + diff report + `promote_dream_run` real
- **Dep:** — · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "dry-run + diff + promote (ADD/UPDATE/DELETE/NOOP) con tests" · **Task file:** `docs/dev/tasks/VER-07.md` · ⬜ PENDING

### Task 35: VER-01 — Tamper-evident: hash-chain en WAL + `vanta-cli verify`
- **Dep:** — · 🔴 3-5d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "hash-chain + comando verify + test de manipulación detectada" · **Task file:** `docs/dev/tasks/VER-01.md` · ⬜ PENDING

### Task 36: VER-05 — Importadores Mem0/Zep/Letta→VantaDB + formato de intercambio
- **Dep:** — · 🟢 1-2d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "importadores con roundtrip + formato documentado" · **Task file:** `docs/dev/tasks/VER-05.md` · ⬜ PENDING

### Task 37: VER-06 — Export file-native Markdown + `rebuild_index` (git-friendly)
- **Dep:** — · 🟢 1-2d · 🟡 · **Ruta:** vanta-worker · **Contrato:** "export MD + rebuild_index + roundtrip verde" · **Task file:** `docs/dev/tasks/VER-06.md` · ⬜ PENDING

### Task 38: VER-02 — Borrado certificado (delete-path shred→GC→WAL + attestation)
- **Dep:** SCH-02 · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "attestation de purga verificable + tests" · **Task file:** `docs/dev/tasks/VER-02.md` · ⬜ PENDING

### Task 39: VER-03 — Redacción-on-write persistida + namespaces cifrados
- **Dep:** WIRE-01 ✅ · 🟡 2-3d · 🔴 · **Ruta:** vanta-worker · **Contrato:** "redacción persistida + envelope por namespace + test PII=0" · **Task file:** `docs/dev/tasks/VER-03.md` · ⬜ PENDING

### Task 40: VER-04 — Governance de inyección (presupuesto + ACLs + audit log)
- **Dep:** WIRE-01 ✅ · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "presupuesto/ACLs enforced + audit log de inyección" · **Task file:** `docs/dev/tasks/VER-04.md` · ⬜ PENDING

### Task 41: MKT-18f — Publicar 9 adapters PyPI (owner-assisted)
- **Dep:** — · 🟡 (owner) · 🔴 · **Ruta:** vanta-lead (packaging; publish = owner) · **Contrato:** "9 adapters publicados o disposición documentada por adapter" · **Task file:** `docs/dev/tasks/MKT-18f.md` · ⬜ PENDING

### Task 42: ICP-01 — Track AI-IDEs (MCP): one-pager + repo-map/watcher + viewer + hooks + demo CI
- **Dep:** MGR-22 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI verde + métrica North Star instrumentada" · **Task file:** `docs/dev/tasks/ICP-01.md` · ⬜ PENDING

### Task 43: ICP-02 — Track local-LLM/privacidad: one-pager + redacción/cifrado/forget + auditoría PII + demo CI
- **Dep:** VER-02/03/04 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI + 0 PII en auditoría" · **Task file:** `docs/dev/tasks/ICP-02.md` · ⬜ PENDING

### Task 44: ICP-03 — Track frameworks: one-pager + adapters PyPI (MKT-18f) + importadores (VER-05) + demo CI
- **Dep:** MKT-18f, VER-05 · 🟡 1sem · 🟠 · **Ruta:** vanta-docs · **Contrato:** "one-pager + demo CI + instalación de adapters verde" · **Task file:** `docs/dev/tasks/ICP-03.md` · ⬜ PENDING

### Task 45: VER-08 — Harness propio: canonical_p99 + LoCoMo/LongMemEval-S/BEAM-subset + p99-CI
- **Dep:** — · 🟡 3-5d · 🔴 · **Ruta:** vanta-tuner · **Contrato:** "harness publicado (dataset commiteado) + p99 en CI + write-quality/abstención" · **Task file:** `docs/dev/tasks/VER-08.md` · ⬜ PENDING

### Task 46: DEF-06 — README↔BENCHMARKS reconciliados (claims)
- **Dep:** VER-08 · 🟡 2d · 🔴 · **Ruta:** vanta-docs · **Contrato:** "claim RocksDB corregido + §2 regenerado + números citables (Regla 11)" · **Task file:** `docs/dev/tasks/DEF-06.md` · ⬜ PENDING

### Task 47: VER-09 — Head-to-head Mem0/Zep/Letta con protocolo publicado (absorbe EXE-02)
- **Dep:** VER-08 · 🔴 1-2sem · 🟠 · **Ruta:** vanta-tuner · **Contrato:** "reporte win/loss con protocolo + dataset reproducible" · **Task file:** `docs/dev/tasks/VER-09.md` · ⬜ PENDING

### Task 48: EXE-01 — Demos CI (3 casos: memory/verify/governance)
- **Dep:** — · 🟡 2-3d · 🟠 · **Ruta:** vanta-worker · **Contrato:** "3 demos ejecutables en CI + output documentado" · **Task file:** `docs/dev/tasks/EXE-01.md` · ⬜ PENDING

### Task 49: N-17 — Notion sync (drafts NOTION-SYNC-2026-09-24.md → páginas)
- **Dep:** — · 🟢 0.5d · 🟡 · **Ruta:** vanta-lead (Notion MCP) · **Contrato:** "10 páginas sincronizadas (0.7.0 vigente, no stale) + draft aplicado" · **Task file:** `docs/dev/tasks/N-17.md` · ⬜ PENDING

---

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

## Recitation

```
=== RECITATION MASTER-ROADMAP ===
Campaign ID: ed20beae-edf6-42f5-b41f-e8519830d6cb
Objetivo activo: F0 — Hardening + quick wins (HARD-01..07)
Estado: pending
Última acción: plan creado (absorción de 5 planes + investigación R1-R4 + decisiones owner)
Resultado: —
Próxima acción: /pipeline run docs/dev/plans/2026-09-26-master-roadmap.md
Contrato: —
Próxima tarea si completa: HARD-01
=== END RECITATION ===
```

**Ejecución:**
```
/pipeline run docs/dev/plans/2026-09-26-master-roadmap.md
```
