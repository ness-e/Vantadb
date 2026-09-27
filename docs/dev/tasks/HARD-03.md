# HARD-03: Continuidad local + release trains

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 3, Fase F0)
- **Fuente:** plan file Task 3 (`:98-120`) + **decisión owner 2026-09-26: push a `develop` SOLO al completar el plan** (master-roadmap §Política de commits/push `:39-44`) + R2 (lección GitLab 2017: pérdida de datos por falta de backups)
- **Esfuerzo:** 🟢 0.5-1d · **Prioridad:** 🔴 · **Tipo:** Mixto (script + docs)
- **Ruta:** vanta-lead
- **Turns estimados:** 8-12
- **Creado:** 2026-09-26T19:53 · **last-synced:** 2026-09-26T19:53
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas — debe permanecer en 0 para ✅
- **Pendientes (downhill):** 5 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/dev/workflow/RULES.md` (fuente de reglas durables del área workflow) ← `docs/dev/workflow/README.md`/`TRIGGERS.md`/`CI_POLICY.md`; `CONTRIBUTING.md:99-125` (Branch & PR Flow) ← contributors; `.opencode/AGENTS.md` (flujo mínimo + Regla 7 §Política de git) ← todo el harness |
| Callees | `git bundle create/verify` (git nativo, sin deps nuevas), `docs/dev/plans/2026-09-26-master-roadmap.md` (§Política — referencia), `scripts/` (convención de scripts pwsh del repo) |
| Implicaciones | No toca código de producto ni CI. Crea un script NUEVO + regla nueva + referencia. Sin cambio de comportamiento de runtime. El script NO ejecuta `git push` (Regla 7). Mitigación directa del riesgo real: 26+ commits locales sin push (medido en plan `:103`). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `docs/dev/workflow/RULES.md` (182 L), `CONTRIBUTING.md` (297 L), `.opencode/AGENTS.md` (Regla 7 §Política de git + flujo mínimo — en contexto de sesión), master-roadmap §Política (`:39-44`) + Task 3 (`:98-120`)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `scripts/` existentes (patrón de scripts pwsh: `validate-docs-coverage.ps1`, etc.); `git` CLI (nativo); `.gitignore` (si un bundle cayera dentro del repo — NO debe)
- **Archivos que referencian a los editados (referencias entrantes):** `docs/dev/workflow/RULES.md` es citado por `CI_POLICY.md:175-182` y el header de `README.md`/`TRIGGERS.md`; `CONTRIBUTING.md` es entry point de contribución (README); `.opencode/AGENTS.md` Regla 7 es citada por `CONTRIBUTING.md:112` (política de git)
- **Veredicto impacto:** bajo — aditivo (script nuevo + sección nueva + referencias); única zona sensible: wording de política en `.opencode/AGENTS.md` (Gate H + stop condition) y evitar duplicar la política (fuente única = RULES.md, pre-mortem F3).

## Contrato
"`pwsh scripts/git-backup.ps1` crea bundle con timestamp y `git bundle verify` exit 0 Y política de commits/push/trenes documentada en `docs/dev/workflow/RULES.md` Y `CONTRIBUTING.md` referencia el script"

## Spec (SDD — Phase 1b: NO feature-add — no agrega símbolos/contratos públicos; decisiones documentadas)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Formato del backup | A) `git bundle create <ts>.bundle` + `git bundle verify` (un archivo, verificable, restaurable con `git clone/fetch`) / B) `git clone --mirror` (dir más pesado, sin verify nativo) | A | ✅ decidido-por-evidencia: contrato del plan `:106` + R2 |
| 2 | Destino del bundle | A) parametrizable (`-Dest`), default fuera del repo; validar misma-unidad → warning explícito (pre-mortem F1) / B) ruta fija en C:\ | A | ✅ decidido-por-evidencia: pre-mortem F1 del plan (`:111`) |
| 3 | Fuente única de la política | A) `RULES.md` (nueva regla con Must/Must-not/Why/Verify, formato propio) + 1 línea de referencia desde AGENTS.md / B) duplicar política en AGENTS.md | A | ✅ decidido-por-evidencia: pre-mortem F3 (`:111`) + stop condition (`:112`) |
| 4 | Release trains | A) 0.8.0 tras F3 (release-plz) + 1.0 con exit criteria de HARD-01 / B) otro esquema | A | ✅ decidido-por-evidencia: plan `:120` + master-roadmap gates por fase (`:29-37`) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** **push SOLO con instrucción explícita del owner** (AGENTS.md Regla 7 §Política de git) — el script es 100% local y JAMÁS pushea; no duplicar la política (RULES.md = fuente única, AGENTS.md solo referencia); no commitear bundles (verificar `.gitignore`/destino); release-plz sigue siendo el único versionador (los "release trains" son fechas/hitos, no versiones a mano).
- **Comandos de verificación:** `pwsh scripts/git-backup.ps1` (exit 0 + bundle con timestamp) + `git bundle verify <bundle>` (exit 0) + `pwsh scripts/validate-docs-coverage.ps1` (docs) + `/harness` (SI se toca `.opencode/`).
- **Deuda pendiente:** ninguna.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-03: …` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | HARD-04 |

```
=== RECITATION ===
Objetivo activo: HARD-03 — continuidad local + release trains
Estado: pending
Última acción: task file creado (Plan: master-roadmap Task 3; decisión owner push al final)
Resultado: —
Próxima acción: Step 1 — `scripts/git-backup.ps1` core (create + verify + exit codes)
Contrato: ver ## Contrato
Invariantes: script local (nunca push); RULES.md fuente única; bundles no commiteados
Deuda: ninguna al abrir
Próxima tarea si completa: HARD-04
last-synced: 2026-09-26
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — aditivo (script + doc + referencia); sin deps nuevas (git nativo), sin suppressions, sin stubs.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (3 condiciones) + ejecución real del script con evidencia (bundle verificado que contiene los commits locales) |
| **Commit** | Atómico(s) (script · docs), conventional (`feat:`/`docs:`/`chore:`), sin push (Regla 7 — este task NO es checkpoint de push) |
| **Release** | N/A (herramienta local + política); los release trains se ejecutan en F3 (0.8.0) y al cierre 1.0 vía release-plz. Justificado. |

## Herramientas necesarias

- `git` (bundle create/verify/list-heads — nativo), pwsh 7 (scripts del repo), `pwsh scripts/validate-docs-coverage.ps1`, `/harness` (Gate H si toca `.opencode/`), `campaign_verify_cmd`, codegraph_explore
- Leer ANTES de editar: `.opencode/rules/release-ci.md` (política release/CI), `.opencode/AGENTS.md` §Regla 7 (política git — no duplicar), `.opencode/references/floor-guard.md` no aplica (sin código Rust)

**Skills cargadas (SDP):** campaign-executor (base task-system) · writing-plans (SDP base del task — plan/política) · documentation-and-adrs (regla durable en RULES.md + referencia en CONTRIBUTING) · git-workflow-and-versioning (política git/release trains — skill F0 del plan) · incremental-implementation (slices: script → robustez → docs) · test-driven-development (case check del script: destino inválido, retención, verify) · source-driven-development (semántica `git bundle` — docs oficiales git). Descartadas: frontend-ui-engineering (sin `web/`), doubt-driven-development (esfuerzo 🟢 / riesgo bajo).

## Investigation Notes

- `RULES.md` tiene formato propio "Must / Must not / Por qué / Verify" por regla → la política de continuidad entra como **regla nueva** (no como prosa suelta). Header declara scope `.github/workflows/ + docs/dev/workflow/* + CI_POLICY.md` — la regla de continuidad es del área workflow (commit/push/release trains), sin tocar CI.
- `CONTRIBUTING.md:99-125` (Branch & PR Flow + nota agent-assisted `:108-112`) es el punto natural para la referencia al script (1-3 líneas, sin duplicar la política).
- `.opencode/AGENTS.md` Regla 7 ya contiene la política de push/commits/tags; el plan la cita como referencia. **Stop condition:** si Gate H rechaza el cambio en AGENTS.md → dejar solo script + RULES.md (plan `:112`).
- `.opencode/` es repo separado (ignorado por VantaDB git): si se edita AGENTS.md, el commit del harness es aparte y requiere `/harness` verde.
- Riesgo real medido (plan `:103`): 26+ commits locales sin push (`git rev-list --count origin/develop..develop`); no existe script de backup ni ritual. `git bundle` es la mitigación estándar (restaurar: `git clone <bundle>` / `git fetch <bundle> <ref>`).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — El bundle contiene la historia completa del repo (incluye `.git`): el destino debe ser privado (unidad externa/sincronizada privada); NO commitear bundles ni subirlos a servicios públicos; si el destino cae dentro del árbol del repo → verificar `.gitignore` o abortar con mensaje. Sin secrets nuevos ni auth.
- [x] **PERFORMANCE** — N/A (herramienta local, fuera de hot path). El script debe ser rápido (<1min esperado) y la retención evita crecimiento sin límite.

## Steps

### Step 1: `scripts/git-backup.ps1` — core (create + verify + exit codes)
- **Archivos:** `scripts/git-backup.ps1` (nuevo)
- **Acción:** script con params (`-Dest` default configurable, `-Keep` futuro); `git bundle create <dest>/vantadb-<yyyyMMdd-HHmm>.bundle` con refs acordadas (`--all` o `develop main tags` — validar contra docs git); `git bundle verify` fail-closed; salida legible (ruta + refs + count de commits locales vs `origin/develop`). Sin `git push` (Regla 7).
- **Verify:** `pwsh scripts/git-backup.ps1 -Dest <tmp>` exit 0 + `git bundle verify <bundle>` exit 0 + caso destino inválido → exit ≠ 0 con mensaje claro
- **Estado:** ⬜ PENDING

### Step 2: Robustez — destino + retención
- **Archivos:** `scripts/git-backup.ps1`
- **Acción:** validar destino (inexistente → crear o error claro; misma unidad que el repo → warning explícito pre-mortem F1, no bloqueante); retención `-Keep` (default 7) con prune de bundles viejos; resumen final (bundle path, tamaño, commits incluidos).
- **Verify:** 2 corridas → 2 bundles con timestamp distinto; prune respeta `-Keep`; warning misma-unidad dispara con `-Dest` en la unidad del repo
- **Estado:** ⬜ PENDING

### Step 3: RULES.md — regla de continuidad + release trains
- **Archivos:** `docs/dev/workflow/RULES.md`
- **Acción:** nueva regla en formato del archivo (Must/Must-not/Por qué/Verify): push solo al final del plan/instrucción owner (ref AGENTS.md Regla 7), bundles como mitigación (comando exacto + verify), release trains 0.8.0 post-F3 (release-plz) y 1.0 con exit criteria de HARD-01 (referenciar COMPATIBILITY).
- **Verify:** regla con check mecánico (`pwsh scripts/git-backup.ps1` + `git bundle verify` exit 0) + `pwsh scripts/validate-docs-coverage.ps1` verde
- **Estado:** ⬜ PENDING

### Step 4: CONTRIBUTING.md referencia el script (+ AGENTS.md solo si aplica)
- **Archivos:** `CONTRIBUTING.md` (§Branch & PR Flow `:99-125`), `.opencode/AGENTS.md` (OPCIONAL: 1 línea de referencia — stop condition)
- **Acción:** agregar 1-3 líneas en CONTRIBUTING (ritual de backup + link al script y a la regla de RULES.md); si se toca `.opencode/AGENTS.md`, SOLO una línea de referencia (sin duplicar política) → **Gate H**; si Gate H rechaza → dejar solo script + RULES.md (stop condition del plan).
- **Verify:** links resuelven (script y RULES.md) + `/harness` verde SI se tocó `.opencode/`
- **Estado:** ⬜ PENDING

### Step 5: Cierre (contrato + evidencia real + commit local)
- **Archivos:** — (todo el set)
- **Acción:** ejecutar el contrato completo; correr el script en modo real (destino externo/sincronizado) y conservar el bundle como evidencia de continuidad; recitation; commit local (nunca push); learnings vía `campaign_memory_write`.
- **Verify:** Contrato (3 condiciones) + bundle real verificado con ≥26 commits locales (`git bundle verify` + count documentado en Notas)
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna (F0). Coordina con **HARD-02** (mismo repo harness para Gate H) y con §Política del master-roadmap (push al final del plan — HARD-03 NO es checkpoint de push). Siguiente: HARD-04.

## Review (GATE — agente distinto, P2-01)

> Placeholder. Lo completa un agente DISTINTO al implementador al ejecutar la tarea (vanta-review/vanta-audit; fallback sin subagentes: doubt-driven-development degradado + escalado al owner). Sin review registrado, NO se marca ✅ COMPLETED.

- **Revisor:** ⬜ (designar al ejecutar — distinto del implementador)
- **Enfoque:** ¿el script es restaurable de verdad (verify + prueba de fetch)? ¿la política quedó en fuente única (sin duplicar AGENTS.md)? ¿el ritual es ejecutable (1 línea en el flujo mínimo)?
- **Cómo se probó:** ⬜ (pegar: salida del script, `git bundle verify` exit 0, caso destino inválido, prune)
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⬜ pendiente

## Notas
- Decisión owner registrada: **push a `develop` solo al completar el plan** con todo validado/verificado; bundles = mitigación de pérdida (R2: lección GitLab 2017). HARD-03 NO introduce checkpoint de push.
- Release trains documentados (no ejecutados acá): **0.8.0 tras F3** (release-plz) y **1.0** con exit criteria de HARD-01. Nunca versión/tag/CHANGELOG a mano.
- Si Gate H rechaza wording en `.opencode/AGENTS.md` → stop condition: solo script + RULES.md.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) como input del reviewer al cierre (Critical/High bloquean).
