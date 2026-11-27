---
title: Avance — Context Engine
kind: review
status: active
tags: [vantadb, avance, context-engine, compression, budget, recall, mmd]
---

# Avance — Context Engine

> Registro consolidado del trabajo completado sobre el context engine (F5): ensamblado de contexto con cascada de compresión LLM-free, estimación de tokens, MMD persistente y wiring productivo al pipeline worker. **IDs originales conservados.** Catch-up por campaña (no commit-por-commit). El crate fuente es `vanta-memory/` — su registro general vive en `vanta-memory.md`.

## Cobertura rápida

- **P29:** assemble con cascada mild/aggressive, token estimator chars/3 + emergency truncate, MMD persistente dedup pair-safe, assemble_with_recall como coordinador único de budget.
- **P31:** wiring productivo como fase post-L3 del pipeline con flag de config y budget compartido.

---

## Campaña P29 — Vanta Context Engine (F5)

### MEM-22/23/24 + MEM-37: Ensamblado, compresión y presupuesto — catch-up por campaña
- **Fecha:** 2026-08-20 → 2026-08-21
- **MEM-22** (`4d1363ec`): context engine assemble con cascada de compresión mild/aggressive — LLM-free, cursor-aware.
- **MEM-23** (`8de35359`): token estimator chars/3 + emergency truncate + `CompactionReport`.
- **MEM-24** (`ddc5671f`): MMD persistente sobre TaskMemory META — dedup por fingerprint, injector pair-safe.
- **MEM-37** (`ae7fe30b`): `assemble_with_recall` como coordinador único de budget (cursor boundary + test e2e).
- **Resultado:** ✅ Parte de la campaña 9/9 (cierre `00f18662`). Las piezas de memoria no-ensamblado viven en `vanta-memory.md` § P29.
- **Ids:** `MEM-22`, `MEM-23`, `MEM-24`, `MEM-37`

---

## Campaña P31 — Cierre Final

### MEM-43: Wiring productivo al pipeline worker
- **Fecha:** 2026-08-21
- **Resultado:** ✅ Commit `a0bcb112` — el context engine se ejecuta como fase post-L3 del pipeline de `vanta-memory`: flag de config para activarlo, presupuesto de tokens compartido entre compresión e inyección. Registro completo en `vanta-memory.md` § P31.
- **Id:** `MEM-43`

---

## Task-system - donde vive el estado de una tarea (FIND-774)

### FIND-774: el selector lee el frontmatter y el store deja de mentir
- **Fecha:** 2026-11-26
- **Decisión:** el **plan file es el único store de estado**; `docs/dev/tasks/<ID>.md` es la spec, no un segundo store. **No se construye registry** - un índice es una caché más que consultar y su fallo es silencioso, que es la causa raíz de los ~21 `Task not found` de la sesión.
- **Selector:** `findPlanFile` lee `status:` del frontmatter como criterio **primario** (`active`, o sin frontmatter); el mtime sólo desempata entre selectables. Con >1 activo tocado en 24h el error **nombra los ficheros**. Sin selectables → `null`.
- **Escritor:** cuando el ID no tiene fila en el plan pero sí task file, el early-return nombra el fichero (`taskFileFound`) en vez de decir `Task not found`. `updated:false`/`taskFound:false` intactos (FIND-775).
- **Verificación:** `node --test .opencode/task-system/mcp/*.test.mjs` → 62/62 antes, **78/78** después (+16, `plan-store.test.mjs`). Reproducción real: el repo tenía 1 plan `active` + 2 `superseded` en 24h y el selector tiraba `Ambiguous active plan`; ahora devuelve el activo.
- **Escala medida:** de **126** task files en `docs/dev/tasks/`, **108** no tienen fila en el plan activo (86 %): el `Task not found` del escritor era estructuralmente falso para la mayoría de IDs, no un borde.
- **Review P2-01:** `vanta-review` en contexto fresco → cambios requeridos y aplicados (verify_cmd ya no salta el budget en silencio, `taskId` validado contra traversal, un solo vocabulario de status, BOM, mensaje `NO_ACTIVE_PLAN` con los ficheros).
- **Id:** `FIND-774`

---

## Task-system - la skill que cierra las tareas sale del catalogo global (FIND-777)

### FIND-777: `progreso` vendorizada en `.opencode/skills/` (decision del owner)

- **Fecha:** 2026-11-27
- **Decision:** la skill que **cierra el estado de una tarea** vivia en
  `~/.agents/skills/progreso/` — fuera del repo, fuera de `.opencode/`, y
  git-ignorada (`.agents/` esta en `.git/info/exclude:9`). El owner decidio
  **vendorizarla**, no mantenerla global: `.opencode/skills/progreso/SKILL.md`
  (9 716 B, copia **byte-identica**, SHA-256 verificado) pasa a ser la copia
  **canonica del repo**. La global **no se borra** — otros proyectos del owner
  la usan; el orden de discovery (project > global) hace que la local gane
  siempre, asi que no hay dos versiones compitiendo dentro de este repo.
- **Rutas:** el archivo **no tenia ninguna referencia absoluta** (`~`, `C:\Users`,
  `/home/`, `.agents` → 0 hits). Su unico defecto de ruta era **relativo**: el
  enlace `../../references/definition-of-done.md` resolvia a
  `~/.agents/references/`, que **no existe** (`Test-Path` → `False`). La skill
  fue escrita para vivir en `.opencode/skills/` y se publico fuera: vendorizarla
  **repara el enlace** (`Test-Path` → `True`) sin editar una sola linea.
- **Registro:** `skills-index.json` regenerado con
  `build-skills-index.mjs` → `progreso` pasa de `source:"global"` /
  `path:"~/.agents/skills/..."` a `source:"project"` /
  `path:".opencode/skills/progreso/SKILL.md"`; el generador reporta
  `.opencode/skills: 4 adoptadas` y `~/.agents/skills: 4 en sombra (override)`.
  Ademas: `skills/INDEX.md` (3 → 4 locales), `SKILLS-MANIFEST.md` (nota de
  vendorizacion) y `references/definition-of-done.md:136` (citaba la ruta de
  fuera como **fuente**). Ninguno de los 44 archivos que invocan
  `skill progreso` necesito edicion: resuelven **por nombre**.
- **Prueba mecanica:** la skill no tiene scripts propios; su Trigger 1.F invoca
  dos del repo. Ambos corridos desde el repo: `scripts/check-avance-coverage.ps1`
  → exit 0, **1034/1034 IDs cubiertos (100.0 %)**, y
  `scripts/validate-docs-coverage.ps1` → exit 0, **0 gaps**.
- **Δ medido:** referencias a `~/.agents/skills/progreso` en `.opencode/`:
  **2 → 0** (eran `skills-index.json:9561` y `definition-of-done.md:136`).
  Fila del Backlog eliminada. Quedan **2 menciones historicas** que no son
  dependencia viva y se conservan a proposito: el informe de auditoria fechado
  `docs/dev/reviews/2026-11-26-harness-hardening.md:414` (evidencia congelada —
  reescribirlo seria falsificar el registro) y este mismo task file, que
  documenta el antes/despues.
- **Premisa re-medida:** la fila afirmaba "se olvido **4 veces** (`status: active`
  en tareas cerradas)". Es una **anecdota de auditoria sin comando reproducible**.
  El unico detector disponible —task file `status: active` sin fila en el
  Backlog— da **8**, pero tiene falso positivo estructural conocido (`FIND-774`:
  108 de 126 task files no tienen fila en el plan activo). La anecdota **ni se
  confirma ni se refuta**; lo medible es la dependencia de la ruta de fuera.
- **Id:** `FIND-777`
