# HARD-03: Continuidad local + release trains

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 3, Fase F0)
- **Fuente:** plan file Task 3 (`:98-120`) + **decisión owner 2026-09-26: push a `develop` SOLO al completar el plan** (master-roadmap §Política de commits/push `:39-44`) + R2 (lección GitLab 2017: pérdida de datos por falta de backups)
- **Esfuerzo:** 🟢 0.5-1d · **Prioridad:** 🔴 · **Tipo:** Mixto (script + docs)
- **Ruta:** vanta-lead
- **Turns estimados:** 8-12
- **Creado:** 2026-09-26T19:53 · **last-synced:** 2026-09-27T02:05
- **Estado:** ✅ COMPLETED (2026-09-27 — commit local delegado al LEAD)
- **Incógnitas (uphill):** 0 abiertas — debe permanecer en 0 para ✅
- **Pendientes (downhill):** 0 steps

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

- **Invariantes a preservar:** **push SOLO con instrucción explícita del owner** (AGENTS.md Regla 7 §Política de git) — el script es 100% local y JAMÁS pushea; no duplicar la política (continuidad/trains = solo RULES.md §8; push/commit = solo AGENTS.md Regla 7 §Política de git, referenciada desde ambos); no commitear bundles (verificar `.gitignore`/destino); release-plz sigue siendo el único versionador (los "release trains" son fechas/hitos, no versiones a mano).
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

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · `test-driven-development` · **PINNED (policy):** `ci-cd-and-automation`, `git-workflow-and-versioning`

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
| Pendientes de ejecución (downhill) | 0 steps |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — El bundle contiene la historia completa del repo (incluye `.git`): el destino debe ser privado (unidad externa/sincronizada privada); NO commitear bundles ni subirlos a servicios públicos; si el destino cae dentro del árbol del repo → verificar `.gitignore` o abortar con mensaje. Sin secrets nuevos ni auth.
- [x] **PERFORMANCE** — N/A (herramienta local, fuera de hot path). El script debe ser rápido (<1min esperado) y la retención evita crecimiento sin límite.

## Steps

### Step 1: `scripts/git-backup.ps1` — core (create + verify + exit codes)
- **Archivos:** `scripts/git-backup.ps1` (nuevo)
- **Acción:** script con params (`-Dest` default configurable, `-Keep` futuro); `git bundle create <dest>/vantadb-<yyyyMMdd-HHmm>.bundle` con refs acordadas (`--all` o `develop main tags` — validar contra docs git); `git bundle verify` fail-closed; salida legible (ruta + refs + count de commits locales vs `origin/develop`). Sin `git push` (Regla 7).
- **Verify:** ✅ 2026-09-27 — run real a temp: exit 0; bundle 155.3 MB; `git bundle verify` exit 0 (28 refs, HEAD=develop tip); destino Z: → exit 1 con mensaje limpio; destino dentro del repo sin .gitignore → exit 1 sin crear dir; colisión de minuto → exit 1 (no sobreescribe)
- **Estado:** ✅ DONE

### Step 2: Robustez — destino + retención
- **Archivos:** `scripts/git-backup.ps1`
- **Acción:** validar destino (inexistente → crear o error claro; misma unidad que el repo → warning explícito pre-mortem F1, no bloqueante); retención `-Keep` (default 7) con prune de bundles viejos; resumen final (bundle path, tamaño, commits incluidos).
- **Verify:** ✅ 2026-09-27 — 2 corridas → `vantadb-20260927-0028` + `-0030` (timestamps distintos); `-Keep 1` → pruned 0030+0028, queda 1 + verify exit 0; warning misma-unidad disparó (C:); restore real: `git clone <bundle>` exit 0 → HEAD `50106b4e`
- **Estado:** ✅ DONE

### Step 3: RULES.md — regla de continuidad + release trains
- **Archivos:** `docs/dev/workflow/RULES.md`
- **Acción:** nueva regla en formato del archivo (Must/Must-not/Por qué/Verify): push solo al final del plan/instrucción owner (ref AGENTS.md Regla 7), bundles como mitigación (comando exacto + verify), release trains 0.8.0 post-F3 (release-plz) y 1.0 con exit criteria de HARD-01 (referenciar COMPATIBILITY).
- **Verify:** ✅ 2026-09-27 — regla 8 insertada (`RULES.md:175-191`) con check mecánico; `pwsh scripts/validate-docs-coverage.ps1` exit 0 (0 gaps, verify_cmd 18.6s)
- **Estado:** ✅ DONE

### Step 4: CONTRIBUTING.md referencia el script (+ AGENTS.md solo si aplica)
- **Archivos:** `CONTRIBUTING.md` (§Branch & PR Flow `:99-125`), `.opencode/AGENTS.md` (OPCIONAL: 1 línea de referencia — stop condition)
- **Acción:** agregar 1-3 líneas en CONTRIBUTING (ritual de backup + link al script y a la regla de RULES.md); si se toca `.opencode/AGENTS.md`, SOLO una línea de referencia (sin duplicar política) → **Gate H**; si Gate H rechaza → dejar solo script + RULES.md (stop condition del plan).
- **Verify:** ✅ 2026-09-27 — CONTRIBUTING.md blockquote (link `docs/dev/workflow/RULES.md`); `.opencode/AGENTS.md` 1 línea en Flujo mínimo → **Gate H ✅ APROBADO** (vanta-harness, sesión `ses_f1ea9677fffeHG7DG7alY3o6CB`; corrió smoke real del script: exit 0)
- **Estado:** ✅ DONE

### Step 5: Cierre (contrato + evidencia real + commit local)
- **Archivos:** — (todo el set)
- **Acción:** ejecutar el contrato completo; correr el script en modo real (destino externo/sincronizado) y conservar el bundle como evidencia de continuidad; recitation; commit local (nunca push); learnings vía `campaign_memory_write`.
- **Verify:** ✅ 2026-09-27 — contrato completo (3 condiciones); bundle real `C:\Users\Eros\VantaDB-Backups\vantadb-20260927-0127.bundle` (verify exit 0, 31 commits locales); review P2-01 ✅; commit local → **delegado al LEAD** (instrucción de orquestación: el sub-agente no commitea)
- **Estado:** ✅ DONE

## Dependencias
- Ninguna (F0). Coordina con **HARD-02** (mismo repo harness para Gate H) y con §Política del master-roadmap (push al final del plan — HARD-03 NO es checkpoint de push). Siguiente: HARD-04.

## Review (GATE — agente distinto, P2-01)

> Placeholder. Lo completa un agente DISTINTO al implementador al ejecutar la tarea (vanta-review/vanta-audit; fallback sin subagentes: doubt-driven-development degradado + escalado al owner). Sin review registrado, NO se marca ✅ COMPLETED.

- **Revisor:** ✅ `vanta-review` (sesión `ses_f1e975170ffemcSiW19TunrzZt`, contexto fresco — P2-01) + Gate H `vanta-harness` (sesión `ses_f1ea9677fffeHG7DG7alY3o6CB`)
- **Enfoque:** restaurabilidad real (verify + clone), fuente única de política, ritual ejecutable — respondidos con evidencia reproducida por el revisor
- **Cómo se probó:** exit 0 + `git bundle verify` exit 0 (28 refs, HEAD `50106b4e`, 31 commits locales); clon real desde bundle → `rev-list --count 40b1dd..develop` = 31 (franja no pusheada restaurada); `Z:\` → exit 1; inside-repo → exit 1 sin dir; colisión → exit 1; `-Keep 1` → prune a 1; `validate-docs-coverage` exit 0 (0 gaps); harness corrió smoke propio (exit 0)
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar):
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere" (decisión owner registrada).
  - [x] No declarar done sin verificar contra los acceptance criteria (steps 1-4 DONE con verify; Step 5 en curso al momento del review).
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial (fallos negativos documentados: exit 1).
  - [x] No hacer un solo intento de búsqueda y darlo por saturado — N/A (tarea script/docs; verificación mecánica múltiple).
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia (Fuentes en `RULES.md:16`; script:5-6; evidencia reproducida).
  - [x] No reintentar en bucle sin diagnóstico — N/A (sin reintentos).
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo (5/5 al contrato).
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad (fail-closed verify; guard inside-repo; never-overwrite; jamás push).
  - [x] No gastar presupuesto infinito; paradas explícitas (retención `-Keep`; exits explícitos).
  - (verificado 10/10 por `vanta-review`, con evidencia per-ítem)
- **Veredicto:** ✅ APROBADO (vanta-review: 0 Critical / 0 Required; 1 Optional wording → corregido en Invariantes; 1 Optional owner-action `-Dest` externo; 4 nits aceptados — ver Notas)

## Notas
- Decisión owner registrada: **push a `develop` solo al completar el plan** con todo validado/verificado; bundles = mitigación de pérdida (R2: lección GitLab 2017). HARD-03 NO introduce checkpoint de push.
- Release trains documentados (no ejecutados acá): **0.8.0 tras F3** (release-plz) y **1.0** con exit criteria de HARD-01. Nunca versión/tag/CHANGELOG a mano.
- Si Gate H rechaza wording en `.opencode/AGENTS.md` → stop condition: solo script + RULES.md.
- OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) como input del reviewer al cierre (Critical/High bloquean).
- **Review P2-01 ✅** (`vanta-review` `ses_f1e975170ffemcSiW19TunrzZt`): contrato reproducido en contexto fresco (clon desde bundle restaura la franja: `rev-list --count 40b1dd..develop` = 31); 0 Critical / 0 Required. Optional-1 (wording) → corregido en Invariantes. Optional-2 (owner action): definir carpeta externa/sincronizada y fijar `-Dest` en `.opencode/AGENTS.md:9`. Nits aceptados: bundle parcial en fallo de create/verify no se auto-borra (se poda por retención); `HEAD` redundante bajo `--all` (inofensivo; habilita checkout en clone).
- **Gate H ✅** (`vanta-harness` `ses_f1ea9677fffeHG7DG7alY3o6CB`, read-only): aprobó la línea de `.opencode/AGENTS.md`; corrió smoke real propio (exit 0).
- **Commit:** delegado al LEAD (instrucción de orquestación pipeline-run: el sub-agente no commitea). Commit local VantaDB: `scripts/git-backup.ps1` + `RULES.md` + `CONTRIBUTING.md` + `HARD-03.md`; `.opencode/AGENTS.md` va en commit aparte del repo harness (Gate H ya verde).
