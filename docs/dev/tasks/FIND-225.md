---
title: "TASK FIND-225: Triage de las 48 alertas CodeQL (6 critical + 42 high)"
kind: task
description: "Dismiss justificado de 48 alertas code-scanning (6 critical salt fixtures + 34 cleartext-logging en tests + 8 docs tooling) - contrato: open=0"
---

# FIND-225: Triage de las 48 alertas CodeQL (6 critical + 42 high)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 8)
- **Fuente:** `docs/dev/Backlog.md` (FIND-225) — deuda de seguridad visible en la pestaña Security del repo
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Tipo:** Mixto (security triage vía API GitHub + tests Rust + scripts docs `.mjs`)
- **Turns estimados:** 15-30
- **Creado:** 2026-10-02T22:30
- **last-synced:** 2026-10-02T23:10
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE — reviewer_context `ses_f0077097affeld9ywnDHp0aSOr`)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 1 step (review P2-01; commit local ya hecho: `c8c3e0fe`)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | GitHub code-scanning (CodeQL 2.27.1, refs/heads/main) — 48 alertas open; consumidor: pestaña Security del repo + gates de seguridad de la org |
| Callees | `gh api` (REST code-scanning alert update); archivos fuente citados por las alertas (tests + scripts docs) — NO se editan |
| Implicaciones | Ninguna sobre código de producción: 48/48 alertas en `tests/` (classifications `[test]`) o `scripts/docs/` (classifications `[documentation]`); 0 en `src/`. Dismissals = estado de API, sin cambios de archivos; sin impacto en performance/API/serialización. Contrato = 0 open con comentario por resolución |

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN. Aplica a los 11 archivos citados por las alertas (solo lectura + verificación; NO se modifican) y al task file (creado).

- **Archivos leídos (completos):** no se edita ningún archivo del repo; se leyeron los tramos citados por las 48 alertas:
  `vanta-memory/tests/dreaming.rs` (L295-354 + ocurrencias `with_run_id_salt`), `vantadb-mcp/tests/dream_tests.rs` (L109-126, L316, L358, L404), `vantadb-mcp/tests/mcp_tests.rs` (L655-690, L2075-2135, L6270-6307), `vantadb-mcp/tests/ver04_governance.rs` (tramos citados), `vantadb-mcp/tests/demo_ai_ides.rs` (tramos citados), `tests/certified_delete.rs` (L79-109), `scripts/docs/gen-index.mjs` (L129-150, L209-224), `scripts/docs/lib.mjs` (L189-200), `scripts/docs/check-structure.mjs` (L29-45), `scripts/docs/attribute-structure.mjs` (L17-31), `scripts/docs/repair-orphan-blocks.mjs` (L15-33); `vanta-memory/src/core/dream/mod.rs` (L1059-1068, `generate_run_id`), `vanta-memory/src/core/dream/mod.rs:147` (`with_run_id_salt`), `vantadb-mcp/src/dreams.rs:105` (schema tool).
- **Archivos referenciados hacia dentro (imports/dependencias):** los tests Rust dependen de `vanta-memory`/`vantadb` (fixtures propios); `scripts/docs/*.mjs` son standalone Node ESM invocados por `.github/workflows/gate-docs-links.yml` (L78 `gen-index.mjs --check`, L112 `check-structure.mjs --budget=40`, L115 `attribute-structure.mjs`).
- **Archivos que referencian a los editados (referencias entrantes):** n/a — no se edita código. `run_id_salt` se consume en `vanta-memory/src/core/dream/mod.rs:1018` y se expone como argumento opcional de la tool MCP (`vantadb-mcp/src/dreams.rs:105`), documentado "for deterministic run ids (tests)".
- **Veredicto impacto:** **bajo** — solo estado de alertas vía API GitHub. Ningún archivo del repo se modifica; ningún contrato público cambia; cero riesgo para el motor.

## Contrato
`gh api '/repos/ness-e/Vantadb/code-scanning/alerts?state=open' --jq 'length'` **= 0** — cada resolución con `dismissed_comment` que cite el motivo (used-in-tests / false-positive con archivo:línea / fixed con commit).

## Spec (SDD)

No aplica — Phase 1b: la solución no agrega símbolos/contratos públicos nuevos (triage de alertas vía API; 0 archivos editados). `N/A` justificado por evidencia: blast radius = 11 archivos test/docs citados, ninguno modificado.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Cero archivos de producción (`src/`) tocados por este triage — verificado: 48/48 alertas en `tests/` o `scripts/docs/`.
  2. Ninguna alerta se dismissea sin revisar su línea fuente real + taint path (pre-mortem #2). Evidencia por alerta en §Investigation Notes.
  3. `dismissed_reason` del enum válido de GitHub: `false positive` | `won't fix` | `used in tests` | `mitigated` (espacios, no guiones).
  4. `dismissed_comment` ≤ 280 chars (límite de la API).
  5. `opencode.jsonc` NO se toca (WIP de otra sesión).
- **Comandos de verificación:** `gh api '/repos/ness-e/Vantadb/code-scanning/alerts?state=open' --jq 'length'` → esperado `0` (contrato). Verificación adicional por grupo: `state=dismissed` + `dismissed_reason` por alerta.
- **Deuda pendiente:** ninguna si el contrato llega a 0. Si CodeQL re-detecta en el próximo push a `main` (re-evaluación), las alertas re-abiertas re-ingresan como FIND nuevo (pre-mortem #1).

## Recitation (canónico)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-225` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | FIND-226 |

## Deuda técnica (Regla 6)

**Sin deuda** — no se introduce código nuevo; solo estado de alertas vía API. Deuda *removida*: 48 alertas de seguridad que ensuciaban la pestaña Security (incl. 6 critical).

## Definition of Done (contrato multi-nivel)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato `alerts?state=open` = 0 + comentario por resolución; ninguna alerta dismisseada sin revisión individual (evidencia en §Investigation Notes) |
| **Commit** | n/a — dismissals son API (`gh api PATCH`), 0 archivos modificados; el único artefacto versionado es este task file |
| **Release** | n/a — no toca release; no aplica changelog/semver |

## Herramientas necesarias
- `gh api --paginate` (listar alertas), `gh api -X PATCH` (dismiss)
- `campaign_verify_cmd` (verificación del contrato)
- `webfetch` / docs.github.com (validación del endpoint — hecho)
- codegraph/CBM no aplica (archivos test/docs, sin símbolos de producción citados)

**Skills cargadas (SDP):** campaign-executor · progreso · test-driven-development (pin) · systematic-debugging (pin) · source-driven-development · security-and-hardening · incremental-implementation · context-engineering — SDP v3, fase BUILD. Cargadas: security-and-hardening, systematic-debugging, source-driven-development (+ base auto).

## Investigation Notes

### Inventario real (2026-10-02, `gh api .../code-scanning/alerts?state=open&per_page=100`)
48 alertas, todas `tool=CodeQL 2.27.1`, `ref=refs/heads/main`. Ninguna en `src/`.

| Grupo | rule.id | Severidad | Count | Paths |
|---|---|---|---|---|
| A | `rust/hard-coded-cryptographic-value` | critical | 6 | `vanta-memory/tests/dreaming.rs` (4), `vantadb-mcp/tests/dream_tests.rs` (2) |
| B | `rust/cleartext-logging` | high | 34 | `vantadb-mcp/tests/mcp_tests.rs` (20), `ver04_governance.rs` (7), `tests/certified_delete.rs` (4), `demo_ai_ides.rs` (3) |
| C | `js/incomplete-sanitization` | high | 5 | `scripts/docs/gen-index.mjs` (4), `scripts/docs/lib.mjs` (1) |
| D | `js/bad-tag-filter` | high | 3 | `scripts/docs/attribute-structure.mjs`, `check-structure.mjs`, `repair-orphan-blocks.mjs` (1 c/u) |

**Alert numbers:** A={184,185,186,187,188,189} · B={150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183} · C={145,146,147,148,149} · D={142,143,144}.

### Revisión individual por grupo (pre-mortem #2 — ninguna dismisseada sin leer la línea)

- **A — critical "hard-coded value used as a salt" (6/6 son fixtures de test):** las 6 líneas citadas son `with_run_id_salt("test-*")` / `seed_dated_dream_run(..., "test-promote-*")`. El valor se consume en `generate_run_id` (`vanta-memory/src/core/dream/mod.rs:1059-1068`), que lo hashea con `DefaultHasher` (SipHash, **no criptográfico**) junto a `session_id`/`now_ms` para producir un run id determinista. El schema MCP lo documenta "Optional salt for deterministic run ids (tests)" (`vantadb-mcp/src/dreams.rs:105`). No es un secreto ni material criptográfico; no hay valor hardcodeado en producción.
- **B — "cleartext-logging" (34/34 son mensajes de fallo de assert en tests):** el span señalado es SIEMPRE el argumento de formato de `assert!`/`assert_eq!` (p.ej. `assert_eq!(out["content"][0]["text"], "[]", "{out}")` en `mcp_tests.rs:6294`, `"all surfaces must report 0 residues: {cert}"` en `mcp_tests.rs:685`). CodeQL modela el panic message (stderr) como "log file" y rastrea el taint desde `session_key` de los argumentos JSON de la tool (valores fake de test: `"dream-agent-1"`, `"sess-a"`) hasta el mensaje. Datos 100% test-locales; no hay logging de producción.
- **C — `js/incomplete-sanitization` (5):** `.replace(/\|/g, '\\|')` en `gen-index.mjs:140,142,216,217` (escape de pipes para celdas de tabla Markdown del índice generado) y `.replace(/"/g, '\\"')` en `lib.mjs:199` (quoting de un escalar YAML en frontmatter generado). El input son títulos/descripciones de docs del propio repo. No hay sink de seguridad (ni HTML, ni SQL, ni shell): es formato Markdown/YAML. Clasificación GitHub: `[documentation]`.
- **D — `js/bad-tag-filter` (3):** regex `/^\s*(<!--|-->)/` / `^<!--|^\s*-->` que clasifican prefijos de línea al escanear Markdown (`attribute-structure.mjs:25`, `check-structure.mjs:36`, `repair-orphan-blocks.mjs:23`). `--!>` no es terminador válido de comentario HTML/Markdown (es sintaxis XML/legacy); ninguna línea se mis-parsea. Sin sink de seguridad.

### Validación web del endpoint PATCH (regla VantaDB de validación — hecha ANTES de ejecutar)
Fuente oficial: https://docs.github.com/en/rest/code-scanning/code-scanning?apiVersion=2022-11-28 (§"Update a code scanning alert").
- Path: `PATCH /repos/{owner}/{repo}/code-scanning/alerts/{alert_number}`.
- `state`: `open` | `dismissed`; **"You must provide `dismissed_reason` when you set the state to `dismissed`"**.
- `dismissed_reason` enum: `false positive`, `won't fix`, `used in tests`, `mitigated`, `null` (con espacios — confirmado; NO usar `used-in-tests`/`false-positive`).
- `dismissed_comment`: string; maxLength **280**.
- Respuesta 200 = alerta actualizada (`state=dismissed`, `dismissed_reason`, `dismissed_comment`, `dismissed_at/by`).

### Decisiones de dismissal (matriz)

| Grupo | reason | Fundamento |
|---|---|---|
| A (6) | `used in tests` | fixtures deterministas de test; valor no criptográfico (DefaultHasher) |
| B (34) | `used in tests` | diagnóstico de assert en tests; datos test-locales |
| C (5) | `false positive` | escape de formato Markdown/YAML en tooling de docs; sin sink ni input no confiable |
| D (3) | `false positive` | marcador de comentario para linter de estructura Markdown de docs |

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — inventario y revisión de los 48 completa; endpoint validado contra docs; contrato verificado |
| Pendientes de ejecución (downhill) | 1 — review P2-01 por agente distinto (leaf no puede spawnear) + commit local |
| % completado | 95% |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — ES la tarea (triage de seguridad). Threat model: trust boundary = API GitHub (auth `gh` CLI, scope `security_events`); activos = postura de seguridad visible del repo; STRIDE: Tampering (PATCH requiere el token del owner), Information disclosure (los comentarios no deben incluir secretos → no los incluyen). Checklist security-and-hardening aplicada: sin secrets en comentarios, sin cambios de auth/deps, sin input no confiable en sinks. Gate: contraste contra docs oficiales ✅.
- [ ] **PERFORMANCE** — no aplica: 0 archivos de código tocados, 0 hot paths. Justificado.

## Steps

### Step 1: Inventario + agrupación de las 48 alertas
- **Archivos:** n/a (API)
- **Acción:** `gh api .../code-scanning/alerts?state=open&per_page=100` → agrupar por rule.id + path; verificar severidades y que ninguna esté en `src/`.
- **Verify:** `48` total; 6 critical (A) + 42 high (B+C+D); 0 en `src/`.
- **Estado:** ✅ DONE

### Step 2: Revisión individual de los 48 (línea fuente + taint path)
- **Archivos:** los 11 citados (solo lectura)
- **Acción:** extraer línea/columnas de cada alerta y leer el contexto; verificar que A/B son test-only y C/D no tienen sink de seguridad; documentar en §Investigation Notes.
- **Verify:** 48/48 líneas leídas; `generate_run_id` no criptográfico confirmado (`DefaultHasher`); `session_key` fixtures confirmados en tests.
- **Estado:** ✅ DONE

### Step 3: Validación del endpoint PATCH contra docs oficiales
- **Archivos:** n/a (web)
- **Acción:** `webfetch` docs.github.com §Update a code scanning alert → confirmar body/enum/límites.
- **Verify:** enum con espacios (`used in tests`, `false positive`), `dismissed_reason` obligatorio con `state=dismissed`, `dismissed_comment` ≤280.
- **Estado:** ✅ DONE

### Step 4: Ejecutar las 48 resoluciones vía `gh api -X PATCH`
- **Archivos:** n/a (API)
- **Acción:** 4 grupos con `dismissed_reason` + `dismissed_comment` citando motivo y archivo:línea; assert de longitud ≤280 antes de enviar; verificar respuesta 200 por alerta.
- **Verify:** 48/48 respuestas con `state=dismissed` y `dismissed_reason` correcto.
- **Resultado:** ✅ 48/48 PATCH 200. Smoke test #189 (A) + lote de 47: A=5, B=34, C=5, D=3. Reasons: 40× `used in tests` + 8× `false positive`. `dismissed_by=ness-e`. Comentarios 218/222/236/259 chars (≤280).
- **Estado:** ✅ DONE

### Step 5: Verify del contrato + evidencia de cierre
- **Archivos:** `docs/dev/tasks/FIND-225.md`
- **Acción:** `gh api .../alerts?state=open --jq 'length'` → 0; registrar output real; OCR delegation review; DoD 3 niveles; commit local del task file.
- **Verify:** contrato = `0` (via `campaign_verify_cmd`).
- **Resultado:** ✅ contrato verde — `campaign_verify_cmd` `{passed:true, exitCode:0, stdout:"0"}`. Verificación individual de los 48 números: todos `state=dismissed` + reason + comment no vacío. Gates docs: `check-docs` 0, `check-links` 0, `markdownlint-cli2` 0 issues, `validate-docs-coverage.ps1` 0 gaps. Fix aplicado: frontmatter agregado al task file (era GATING `missingFrontmatter=1`).
- **Estado:** ✅ DONE

### Step 6: Review P2-01 (agente distinto) + commit local
- **Archivos:** `docs/dev/tasks/FIND-225.md`
- **Acción:** el sub-agente es leaf (sin tool `task`) → NO puede spawnear `vanta-review`. Evidencia completa dejada en §Review para que el orquestador corra el review. **Commit local hecho:** `c8c3e0fe docs(tasks): FIND-225 - triage de las 48 alertas CodeQL (48 dismissals justificados, contrato open=0)` (solo el task file; sin push).
- **Verify:** veredicto registrado en §Review por reviewer distinto (`reviewer_context ≠ author_context`) → luego `campaign_update_task_state(completed)`.
- **Estado:** ✅ DONE (review APPROVE registrado en §Review — reviewer_context `ses_f0077097affeld9ywnDHp0aSOr`)

## Dependencias
- Ninguna (Wave 0). Dependientes: FIND-226 (nextTask).

## Review (GATE — agente distinto, P2-01)

> **Estado: ✅ APPROVE (P2-01, 2026-10-03) — reviewer fresco `vanta-review` (`reviewer_context` `ses_f0077097affeld9ywnDHp0aSOr` ≠ autor `ses_f008ab62dffeenZGImv7g0jqWm`). Evidencia re-ejecutada: contrato `open=0`; spot-check 9 alertas (4 grupos) `dismissed` con reason+comment; inventario 142–189 sin gaps; 40× `used in tests` + 8× `false positive`; 0/48 en `src/`. Observaciones: nit self-referencial del commit (se resuelve con esta edición); dismissals históricos de sept en `src/` revisados como diligencia (candidato FIND futuro: #137 `won't fix` PBKDF2 legacy); con umbral >30 usar `per_page=100`. Evidencia completa en el RESULTADO del reviewer (sesión citada).**

- **Revisor requerido:** `vanta-review` o `vanta-audit` (agente distinto al implementador; `reviewer_context ≠ author_context`).
- **Tier (HARD-02):** **Fast** — diff = `docs/dev/tasks/FIND-225.md` (task file), 0 paths adversariales (`docs/api/**`, `src/**`, etc. no tocados). Gate = verify fast mecánico + veredicto registrado (spot-check), sin adversarial completo.
- **Enfoque a revisar:**
  1. ¿El triage clasificó correctamente cada alerta? Riesgo principal: dismiss de algo real (pre-mortem #2). Contra-evidencia: 48/48 líneas fuente leídas y documentadas (4 grupos, `## Investigation Notes`), `classifications` de GitHub = `[test]` (34×) / `[documentation]` (8×) / `[test]` (6×), todas `refs/heads/main`, 0 en `src/`.
  2. ¿Los `dismissed_reason` son correctos? `used in tests` (fixtures deterministas de test, no criptográficos — `DefaultHasher`; mensajes de assert) y `false positive` (tooling de docs sin sink ni input no confiable).
  3. ¿El contrato se verificó de forma independiente? `campaign_verify_cmd` ejecutó `gh api .../state=open --jq 'length'` → `stdout="0"`, `passed:true`. Inventario post-hoc: 48/48 alertas con `state=dismissed` + reason + comment.
- **Cómo se probó (evidencia reproducible):**
  - Inventario: `gh api '/repos/ness-e/Vantadb/code-scanning/alerts?state=open&per_page=100'` (48 antes; 0 después).
  - Por alerta: `gh api '/repos/ness-e/Vantadb/code-scanning/alerts/<n>'` → `state`, `dismissed_reason`, `dismissed_comment`.
  - Comentarios citan archivo:línea (ej. `#184` cita `vanta-memory/src/core/dream/mod.rs:1059`; `#145` cita `gen-index.mjs:140,142,216,217`; `#142` cita los 3 `.mjs`; `#150` cita `mcp_tests.rs:685`).
  - OCR delegation: `pwsh dev-tools/ocr-review.ps1 -Format json` → el selector de OCR **excluye `.md` (`unsupported_ext`)**; único archivo reviewable del workspace = `docs/pipeline-state.json` (WIP de otra sesión, no de esta tarea). `ocr delegate rule docs/dev/tasks/FIND-225.md` devuelve el Rule Group default (Correctness/Security/Performance/Maintainability/Tests) — genérico para Markdown. Sustituto mecánico: gates docs ✅ + markdownlint ✅. Sin hallazgos Critical/High.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas — todos los outputs de `gh`/`campaign_verify_cmd` son reales y citados.
  - [x] No saltarse la clarificación — sin decisiones ambiguas; contrato del plan explícito.
  - [x] No declarar done sin verificar contra acceptance criteria — contrato verificado mecánicamente (`0`).
  - [x] No ignorar fallos — el gate `missingFrontmatter` falló y se corrigió (frontmatter agregado) antes de cerrar.
  - [x] No hacer una sola búsqueda — revisión individual de las 48 líneas + taint paths.
  - [x] No copiar sin citar — cada dismissal cita `archivo:línea` y motivo.
  - [x] No reintentar en bucle — sin reintentos necesarios.
  - [x] Pasos conectados al objetivo — inventario → revisión → validación → ejecución → verify.
  - [x] No degradar chequeos en paths de seguridad — 48/48 revisadas, ninguna dismisseada a ciegas.
  - [x] Presupuesto acotado — ~30 tool calls, dentro de límites.
  - [x] Cobertura SDP — skills cargadas cubren el dominio (security-and-hardening + source-driven-development + systematic-debugging).
- **Veredicto:** ✅ **APPROVE** — registrado 2026-10-03; cierre mecánico `campaign_update_task_state(completed)` ejecutado por el orquestador con payload review fresh (`ses_f0077097affeld9ywnDHp0aSOr`). Observaciones del reviewer: (1) nit del commit self-referencial (resuelto al commitear esta edición); (2) dismissals históricos en `src/` (21–24 sept) revisados como diligencia — candidato FIND futuro (#137 `won't fix` PBKDF2 legacy); (3) para contratos con umbral >30 usar `per_page=100`.

## Notas
- Plan pre-mortem #2 respetado: se revisaron las 48 líneas antes de dismissear. Ninguna resultó alerta "real de producción" → 0 fixes de código necesarios, 0 filas FIND nuevas.
- `opencode.jsonc` (WIP ajeno) y `docs/pipeline-state.json` NO se tocan ni commitean.
- Riesgo de re-apertura (pre-mortem #1): GitHub no re-abre una alerta dismisseada mientras el código no cambie; si un push futuro modifica estas líneas, CodeQL puede re-detectar → re-ingresa como FIND nuevo.
- **Evidencia de ejecución (2026-10-03T02:00Z):** 48 PATCHes OK (1 smoke #189 + 47 en lote); contrato `open=0` verificado por `campaign_verify_cmd` (`passed:true, stdout:"0"`); verificación individual 48/48 (`state=dismissed` + reason + comment presente). Comentarios: 218-259 chars (límite 280).
- **Incidente menor resuelto:** `check-docs.mjs` GATING `missingFrontmatter=1` por el task file nuevo → corregido agregando frontmatter (`title`/`kind: task`/`description`, patrón de PROC-02/PROC-03/AUD-038). Gates docs re-verificados en verde.
- **Pendiente estructural:** review P2-01 (leaf sin tool `task`). Evidencia completa en §Review para el orquestador.
