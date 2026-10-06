---
title: "TASK MEMG-14: Marco 2.0 — núcleo + extensiones (reformulación del marco 8/6/10/PI)"
kind: task
description: "Marco reformulado 'núcleo + extensiones' (7 sub-decisiones a-g), respaldo de dims/áreas/ámbitos en repo, validaciones externas y cadencia anual; fuentes fetch-verificadas; draft sync Notion"
---

# TASK MEMG-14: Marco 2.0 — núcleo + extensiones (reformulación del marco 8/6/10/PI)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 60, Wave F5 — Frontera & estrategia; research/spec)
- **Fuente:** Backlog `MEMG-14` (`docs/dev/Backlog.md:140`) + plan Task 60 (L1725-1751, bloque F0 expandido) + `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` §14 + marco núcleo en repo (`SPEC.md:9`, `Backlog.md:776`) + `docs/dev/research/validacion/` (8 docs)
- **Esfuerzo:** 🟡 2-3d | **Appetite:** max 3d
- **Prioridad:** 🟡 (plan) / 🟠 P1 (Backlog)
- **Tipo:** Research/spec (docs-only — cero código; cero símbolos públicos)
- **Turns estimados:** 8-14 (una sesión de sub-agente)
- **Creado:** 2026-10-05 | **last-synced:** 2026-10-05
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `60` en el campaign server)
- **Incógnitas (uphill):** 0 — downhill directo (reformular → respaldar → sync/revisión). La incógnita "¿el marco está en el repo?" se resolvió en DISCOVERY: **núcleo sí** (`SPEC.md:9` + `Backlog.md:776`: D1-D8/C1-C6/AM1-AM10), detalle de cada dim/ámbito vive en Notion (page IDs citados)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `60`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs (sin código):** master plan Task 60 (estado), `NOTION-SYNC-2026-09-24.md` §14 (nota "sync completo queda en MEMG-14(e)" → pointer), Backlog `:140` (fila MEMG-14 — se elimina al cierre, Trigger 1 progreso), `docs/dev/strategy/ROADMAP-v0.7.md:67` (mención informativa, sin cambio), avance (registro de cierre). F5 depende del vocabulario: MEMG-15/18/19 (Tasks 61-63) consumen la reformulación. |
| Callees | Fuentes citadas (sin modificar): `SPEC.md`, `docs/dev/Backlog.md`, `NOTION-SYNC-2026-09-24.md` §14, `docs/dev/research/{mgr-10,mgr-12,mgr-23-24,validacion/00}`, `ADR-0054`, `backlog-futuro.md` (FUT-15), URLs externas fetch-verificadas. |
| Implicaciones | **Aditivo docs-only:** `docs/dev/research/memg-14-marco-2.0.md` (nuevo; `kind: research`) + `docs/dev/tasks/MEMG-14.md` (nuevo, este archivo) + edición append-only en `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` §14 + eliminación de fila `docs/dev/Backlog.md` + registro `docs/dev/avance/activo/operaciones.md`. `gen-index --write` regenera `docs/index.md` + `llms.txt` (delta = el doc nuevo). Sin código, sin wire, sin deps, sin locks. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-05, HEAD `64eee3f7`).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (:1-144 — completo; §13-14 = syncs ejecutados + pendiente MEMG-14(e)).
  - `docs/dev/Backlog.md` (:80-174 DELTA P0-P3 + `:740-829` Phase 49 (marco 8/6/10: `:773-776`) + grep `MEMG-14` = fila `:140` + `:120` + ROADMAP `:67`).
  - `SPEC.md` (:1-153 — completo; `:9` = marco en repo: 6 áreas + 8 dims con nombres).
  - `docs/dev/architecture/adr/ADR-0054-scheduler-host-vantadb-server.md` (:1-203 — cadencia L0→L3: host del planificador).
  - `docs/dev/research/mgr-23-24-memoria-proyecto.md` (:1-60 — track PI: taxonomía de lo memorable; "Notion no accesible a workers — el resumen del Backlog manda").
  - `docs/dev/research/validacion/00-SINTESIS-EJECUTIVA2.md` (:1-80 — síntesis 2026-08-25, docs 01-08; contenido producto/mercado).
  - `docs/dev/tasks/MEMG-13.md` (:1-189 — formato canónico de task file de la campaña).
  - `docs/dev/avance/activo/vanta-memory.md` (:160-204 — registros MEMG-11..21; formato de cierre).
  - `.opencode/skills/progreso/SKILL.md` (:1-219 — Trigger 1: eliminar fila Backlog + registrar avance por dominio).
  - `docs/dev/backlog-notion.md` (:1-86 — registry Notion: N-11 re-validación recurrente, N-18..N-32).
  - `docs/dev/backlog-futuro.md` (FUT-15 — multimodal fuera de alcance v1.0 con triggers).
- **Archivos referenciados hacia dentro (imports/deps):** n/a (docs; sin imports). El research-doc referenciará hacia afuera con enlaces relativos (documentation-skill §1).
- **Referencias entrantes (grep `MEMG-14|Marco 2.0` HEAD):** `Backlog.md:140` (fila), `NOTION-SYNC-2026-09-24.md:143-144` (pendiente), `docs/dev/strategy/ROADMAP-v0.7.md:67` (mención), master plan L1725-1751 + L1788 + L2895 (Task 60 — **prohibido editar**), `docs/dev/avance/activo/operaciones.md:408` (registro Sync #2).
- **Veredicto impacto:** **BAJO (aditivo docs-only)** — 1 doc nuevo + 1 edición append-only + 1 fila Backlog eliminada al cierre + 1 registro avance. Sin cambios en código, contratos, wire, deps ni locks. Riesgos del pre-mortem mitigados: (1) sin re-litigación — el doc reformula las 7 sub-decisiones tal como están sancionadas en `Backlog.md:140`; (2) drift Notion↔repo — IDs de página citados + fecha + commits; (3) fuentes no fetch-verificadas — **11/11 URLs verificadas en DISCOVERY** (abajo).

## Contrato

"Marco reformulado ('núcleo + extensiones') con las 7 sub-decisiones del backlog: (a) áreas 7/8 + meta-área observabilidad, (b) ámbitos +4 y ampliaciones, (c) ejes + cadencia L0→L3, (d) corrección 'cerrada en seis' + separación cognitivas/ingeniería, (e) sync Notion + respaldo al repo + registro de validaciones + cadencia anual, (f) confianza calibrada como extensión + paramétrica fuera de alcance, (g) lista 'no elevar' + nota bitemporal; revisión documentada; respaldo en `docs/` con enlaces a evidencia." (plan Task 60 L1734).

**Verificación del contrato (cierre):** research-doc existe con las 7 sub-decisiones (a-g) mapeables 1:1 contra `Backlog.md:140` + respaldo del marco núcleo (D1-D8/C1-C6/AM1-AM10/PI con anclas repo + IDs Notion) + registro de validaciones (§3) + cadencia (§4) + draft de sync (§5) + gates docs 0 (`check-links` + `check-docs` + `gen-index --check`) + `validate-docs-coverage.ps1` 0 gaps.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): pre-respondido por el plan F0** — Task 60 sanciona el contrato completo (7 sub-decisiones + respaldo + draft sync; Gate Result ✅ DO) y el stop fija el corte (>3d → (d)+(e) + FIND del resto). Sin símbolos públicos nuevos (docs-only); sin ambigüedad de contrato (las 7 decisiones están enumeradas en `Backlog.md:140`). Precedente de campaña: MEMG-13 (docs+spec, "Gate D: pre-respondido por el plan F0").

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Hogar del respaldo | A) **research-doc nuevo en `docs/dev/research/` (kind=research)** / B) sección en `NOTION-SYNC` (mezcla draft con registro) / C) `docs/dev/strategy/` (no es el hogar de investigaciones) | ✅ **A** — decidido-por-evidencia: patrón `mgr-*.md` (research docs del programa MGR); `kind: research` → ruta canónica `docs/research/` (documentation-skill §2.1); NOTION-SYNC queda como registro de syncs |
| 2 | Nombre | A) **`memg-14-marco-2.0.md`** (patrón `<ID>-<tema>`) / B) `marco-2.0-nucleo-extensiones.md` (sin ID, no trazable) | ✅ **A** — patrón `mgr-10-bitemporalidad.md`, `mgr-23-24-*.md` |
| 3 | Alcance del doc | A) **7 sub-decisiones + respaldo núcleo + registro validaciones + cadencia + draft sync** (todo el contrato) / B) solo reformulación (deja (e) sin entregable) | ✅ **A** — contrato L1734 enumera (a)-(g) completos |
| 4 | Contenido no verificable (Notion) | A) **citar IDs de página + fecha + marcar `[a verificar — Notion]` el detalle no presente en repo** / B) fetch Notion desde el worker (fuera de política: workers no fetchean Notion — `pipeline-full.md` 0c-context; precedente `mgr-23-24:11`) | ✅ **A** — el respaldo ancla a lo que SÍ está en repo (SPEC.md:9, Backlog:776) y marca el resto |
| 5 | Sync Notion | A) **draft append-ready en §5 (páginas + texto) + pendiente aplicación (lane owner; Sync #2 requirió aprobación owner via question)** / B) ejecutar el sync desde el worker (sin aprobación; fuera de lane) | ✅ **A** — patrón `NOTION-SYNC` ("contenido listo para pegar… ejecuta el owner/agente") |
| 6 | Verificación de fuentes | A) **fetch de cada URL citada en DISCOVERY (TSYS-13); solo se citan las resueltas** / B) citar y marcar todo `[a verificar]` | ✅ **A** — red disponible; 11/11 resueltas (ver §Notas); sin citas pendientes |
| 7 | Estructura de las 7 decisiones | A) **una subsección por letra (a)-(g): decisión + evidencia + estado** / B) prosa continua (no verificable 1:1) | ✅ **A** — permite el check mecánico del contrato (7/7 contra Backlog:140) |

## Invariantes de dominio (handoff — MUST)

1. **Scope = reformular las 7 sub-decisiones, NO rediseñar el marco** (pre-mortem #1 del plan). Si algo excede, va a FIND.
2. **Lo no verificable desde el repo se marca** `[a verificar — Notion: <página>]` — nunca se presenta como verificado (Regla 11).
3. **IDs de página Notion citados** (patrón NOTION-SYNC) + fecha del sync (`2026-09-30`) + commits (`076d2bb1`/`fc3ec237`).
4. **Regla 11:** todo claim con fuente reproducible (repo path o URL fetch-verificada); los números citados llevan su ancla (p. ej. ECE 0.0003 → `NOTION-SYNC:57` + VER-08).
5. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno no se stagea; commit con **pathspec**.
6. **Frontera de este run:** cero implementación (docs); el marco como taxonomía no genera código ni símbolos.
7. **Sync Notion = draft** — la aplicación queda en lane owner (política de workers; Sync #2 precedente).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código, sin `unsafe`, sin deps, sin abstracciones. El run **elimina** deuda de trazabilidad (el marco deja de vivir solo en Notion: respaldo + validaciones + cadencia quedan auditables desde el repo). Deuda declarada: ninguna nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Research-doc con (a)-(g) verificables 1:1 + respaldo núcleo + registro validaciones + cadencia + draft sync; fuentes fetch-verificadas; gates docs 0; task file completo (Impacto Regla 0 + Review P2-01) |
| **Commit** | Commit atómico conventional `docs(research):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin changelog) |

## Herramientas necesarias

- `webfetch`/`fetchWebContent` (verificación TSYS-13 de URLs citadas) + grep/read (docs — `codebase-memory-mcp` desconectado en la sesión, fallback documentado)
- `campaign_verify_cmd` (gates docs + validate-docs-coverage) + `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` + `pwsh scripts/validate-docs-coverage.ps1`

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `writing-guidelines` · `writing-plans` (base type docs) · `documentation-and-adrs` (keyword docs) · `documentation-skill` (obligatoria docs/**) · `source-driven-development` (verificación de fuentes). SDP phase=DEFINE devolvió además `spec-driven-development`/`interview-me`/`idea-refine` — excluidas: contrato ya sancionado por el plan F0 (sin ambigüedad que interviewar). `test-driven-development`/`systematic-debugging` excluidas: sin lógica ni bug.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — N/A: docs-only, sin trust boundary nuevo, sin input de usuario, sin deps, sin red en producto (los fetches de verificación son de DISCOVERY, solo lectura).
- [ ] **PERFORMANCE** — N/A: sin hot paths, sin código; Regla 9 no dispara (sin claim de optimización).

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato)

- **Archivos:** `docs/dev/tasks/MEMG-14.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (NOTION-SYNC §14, Backlog:140/776, SPEC.md:9); resolución de la incógnita "marco en repo" (núcleo sí / detalle Notion); verificación TSYS-13 de las URLs clave (11/11); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + `campaign_validate_scope` OK (advisory)
- **Evidencia:** ✅ este archivo; URLs verificadas en §Notas (títulos registrados)
- **Estado:** ✅ COMPLETED

### Step 2 — Research-doc: reformulación (a-g) + respaldo del marco núcleo

- **Archivos:** `docs/dev/research/memg-14-marco-2.0.md` (nuevo)
- **Acción:** redactar el research-doc: §1 marco núcleo (D1-D8/C1-C6/AM1-AM10/PI con anclas repo + IDs Notion + marcas); §2 las 7 sub-decisiones (a)-(g) con decisión/evidencia/estado; §3 registro de validaciones externas de apuestas propias; §4 cadencia anual; §6 límites.
- **Verify:** `node scripts/docs/check-docs.mjs` (frontmatter/kind OK)
- **Evidencia:** ✅ doc creado (7 subsecciones (a)-(g) mapeables 1:1 a `Backlog.md:140`; respaldo núcleo con anclas `SPEC.md:9`/`Backlog.md:776` + IDs Notion + marcas `[a verificar — Notion]`; §3 registro de validaciones; §4 cadencia; §6 límites). `check-docs`: GATING all clear (kind/path OK); `check-links`: 0 broken markdown.
- **Estado:** ✅ COMPLETED

### Step 3 — Draft de sync Notion + cross-links + gates docs

- **Archivos:** `docs/dev/research/memg-14-marco-2.0.md` (§5 draft), `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (§14 pointer), `docs/index.md` + `llms.txt` (generados)
- **Acción:** §5 con páginas + IDs + texto append-ready (incluye la nota pendiente de `Seguridad de la memoria` — MEMG-17, §14:143); actualizar el "Estado" de §14 con el pointer; `gen-index --write`.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0
- **Evidencia:** ✅ §5 draft (6 páginas: áreas/dims/ámbitos/PI/Propuesta/Seguridad) marcado DRAFT — pendiente aplicación lane owner; §14 actualizado (pointer al research-doc + nota MEMG-17 incluida en draft); `gen-index --write` regeneró 6 índices (delta real: `docs/index.md` + `llms.txt`; doc indexado en `docs/index.md:247` + `llms.txt:316`); `gen-index --check` limpio; `check-links` 0 broken; `check-docs` GATING all clear; `validate-docs-coverage.ps1` 0 gaps.
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-14.md` (§Review + RESULTADO), `docs/dev/Backlog.md` (eliminar fila MEMG-14), `docs/dev/avance/activo/operaciones.md` (registro de cierre)
- **Acción:** `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → Critical/High bloquean; Medium → FIND. Clasificar tier HARD-02: paths `docs/**` → **Fast** (verify mecánico + veredicto §Review). Fork `vanta-review` (fresh context) para P2-01. Commit **LOCAL** `docs(research):` con pathspec; cierre `campaign_update_task_state(completed, taskId:"60")` con recitation + payload `review`; Trigger 1 progreso (fila Backlog eliminada + registro avance).
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** ✅ **Gates (working tree):** `check-links` 0 broken (3025 links) · `check-docs` GATING all clear · `gen-index --check` limpio al momento del write (posteriormente stale por WIP ajeno: MEMG-15 editó `docs/api/MEMORY_INTERCHANGE_FORMAT.md`; R-1 declarado) · `validate-docs-coverage.ps1` 0 gaps. **OCR delegation:** `pwsh dev-tools/ocr-review.ps1 -Format json` → 2 archivos code reviewables (`src/sdk/serialization/impl_export.rs`, `dev-tools/heavy-test-lock.ps1`) = **WIP ajeno** (fuera de pathspec; no se tocan); diff propio docs-only excluido por diseño (`unsupported_ext`) → 0 hallazgos aplicables. **Review P2-01:** vanta-review tier Fast — ronda 1 🔴 changes-required (R-1/R-2) → fixes → ronda 2 ✅ APPROVE (sesión `ses_ef0bf832cffefQ7hc1vjWRs2Qh`). **Commit:** `21939b6f` (5 archivos propios; pre-commit hook OK). **Trigger 1 progreso:** fila `MEMG-14` eliminada de `Backlog.md` (diff 1 deletion) + registro en `operaciones.md`. **Campaign:** `completed` taskId `60` con payload review.
- **Estado:** ✅ COMPLETED

## Dependencias

- ✅ Evidencia base: Sync #2 (`NOTION-SYNC §14`, commits `076d2bb1`/`fc3ec237`) · marco núcleo en repo (`SPEC.md:9`, `Backlog.md:776`) · `mgr-10`/`mgr-12`/`mgr-23-24` (research docs relacionados) · `ADR-0054` (cadencia L0→L3).
- Consumidores (F5): MEMG-15 (Task 61 — portabilidad usa (a)/(e)), MEMG-18 (Task 62 — multimodal usa (b)/(f)), MEMG-19 (Task 63 — descartes usan (b)/(g)).
- nextTask: MEMG-15 (Task 61) — lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: paths `docs/**`; verify mecánico + veredicto; sin adversarial completo). Sesión reviewer: `ses_ef0bf832cffefQ7hc1vjWRs2Qh`. Ronda 1: 🔴 changes-required (2 Required) → fixes aplicados → ronda 2: ✅ **APPROVE**.
- **Enfoque:** contrato 7/7 vs `Backlog.md:140` + plan L1734; respaldo §1 vs `SPEC.md:9`/`Backlog.md:776`; marcas de honestidad `[a verificar — Notion]`; fuentes (3/11 re-fetch en vivo por el reviewer); scope docs-only; gates 4/4; DoD.
- **Cómo se probó:** el reviewer re-corrió `check-links` (0 broken, 3025 links) · `check-docs` (GATING all clear) · `gen-index --check` (exit 0) · `validate-docs-coverage.ps1` (0 gaps); lectura completa de ambos artefactos; `Select-String` de anclas; `git diff --numstat` de generados; verificación en vivo de arXiv 2604.16548 / W3C CG / DAMA.
- **Hallazgos + disposición:**
  1. [REQUIRED r1] R-1 — `docs/index.md`/`llms.txt` incluyen `MEMG-15.md` (untracked, sesión paralela) → **transitorio declarado** en §Notas + fix pre-push (secuenciar; commits LOCAL).
  2. [REQUIRED r1] R-2 — ancla "cascadas multiagente" apuntaba a NOTION-SYNC (0 matches) → **re-anclada** a `SPEC.md:9`; re-verificado.
  3. [OPTIONAL r1] O-1 "10/10"→"11/11" (4 spots) · O-2 RESULTADO/§-numeración → **corregidos**.
  4. [NIT r1] N-1 alias D8 ("meta-memoria") → nota agregada · N-2 path completo `docs/dev/strategy/ROADMAP-v0.7.md` → **corregido**. [NIT r2] NIT-1..3 (numeración § residual + redacción R-1) → **aplicados en Step 4** (cosméticos, pre-autorizados por el reviewer).
- **Veredicto:** ✅ **APPROVE** (ronda 2) — contrato 7/7, gates verdes, fixes verificados; listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-4 ✅ (research-doc + gates docs + OCR + review P2-01 APPROVE ronda 2 + commit `21939b6f` + fila Backlog eliminada + registro avance + cierre campaign taskId `60`).
- **Próximo paso:** ninguno (tarea cerrada). Handoff: aplicar el draft de sync §5 (lane owner) + R-1 pre-push (secuenciar `MEMG-15.md` → regenerar índice si aplica).
- **Estado del worktree:** HEAD `21939b6f` (+ commit de cierre docs(avance)); WIP ajeno (`opencode.jsonc`, master plan, `src/{attestation.rs,cli_handlers/data.rs,sdk/**,tests/**}`, `dev-tools/heavy-test-lock.ps1`, `docs/dev/tasks/MEMG-15.md`, `docs/api/MEMORY_INTERCHANGE_FORMAT.md`) NO se stagea; prohibido tocar `docs/pipeline-state.json`.

## Notas

- **URLs fetch-verificadas en DISCOVERY (TSYS-13, 11/11 OK):** `arxiv.org/abs/2604.16548` ("A Survey on Long-Term Memory Security in LLM Agents…" — confirma 6 fases: Write, Store, Retrieve, Execute, **Share & Propagate**, **Forget & Rollback**) · `w3.org/community/ai-agent-memory-interop/` (CG charter v1.0: cell portable, ML-DSA-65, sharing contracts con revocación, erasure GDPR Art.17) · `dama.org` DMBOK (2ª ed. con "data integration & interoperability") · `arxiv.org/abs/2602.06052` · `2507.07957` (MIRIX) · `2512.13564` · `2607.12385` (PM-Bench) · `2606.23459` (TriggerBench) · `2602.23944` (MemEmo) · `2507.05257` (MemoryAgentBench) · `2607.27773` (ChronoMem). Sin citas `[a verificar — sin red]`.
- **`codebase-memory-mcp` desconectado** ("Not connected") → discovery docs por grep/read (fallback sancionado: "fallback solo configs/docs/no indexado"). No bloquea: tarea docs-only.
- **Precedente workers↔Notion:** `mgr-23-24:11` — "Notion track PI (no accesible a workers — el resumen del Backlog manda)"; `pipeline-full.md` 0c-context. El sync queda como draft (§5).
- **R-1 (review ronda 1) — transitorio de índice declarado:** `docs/index.md` + `llms.txt` regenerados incluyen la fila de `docs/dev/tasks/MEMG-15.md` (untracked — sesión paralela MEMG-15 en vuelo). El commit es **LOCAL**; **antes del push (PROC-01):** verificar que `MEMG-15.md` esté commiteado — si ya lo está, el índice actual es consistente sin cambios; si sigue untracked, la acción operativa es **secuenciar** (su sesión commitea `MEMG-15.md` antes del push; re-correr `gen-index --write` solo tiene efecto cuando el árbol no contiene archivos que no se van a commitear — si nunca se commiteara, retirarlo del árbol y regenerar). No se toca el WIP ajeno.
- **WIP ajeno:** master plan + `opencode.jsonc` + `src/{attestation.rs,cli_handlers/data.rs,sdk/**,tests/**}` (sesión MEMG-15) + `dev-tools/heavy-test-lock.ps1` + `docs/dev/tasks/MEMG-15.md` modificados/untracked en el árbol por otros — NO se stagean. Commit con pathspec.
- **NOTICED BUT NOT TOUCHING:** `docs/dev/strategy/ROADMAP-v0.7.md:67` menciona MEMG-14..23 (informativo, sin cambio); N-11 (`backlog-notion.md`) es la re-validación recurrente de cobertura dims — el research-doc define la cadencia anual sin pisarla; `docs/dev/research/validacion/` son docs de producto/mercado 2026-08-25 (no del marco — el marco vive en Notion + Backlog).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 4/4 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: 21939b6f (docs research, LOCAL — sin push) + commit de cierre docs(avance)
ARCHIVOS: docs/dev/research/memg-14-marco-2.0.md (nuevo), docs/dev/tasks/MEMG-14.md (nuevo), docs/dev/strategy/NOTION-SYNC-2026-09-24.md, docs/index.md + llms.txt (generados), docs/dev/Backlog.md (fila removida), docs/dev/avance/activo/operaciones.md (registro)
VERIFY_CONTRATO: pasa — research-doc (7/7 sub-decisiones + respaldo + registro + cadencia + draft) + gates docs 4/4 exit 0 en working tree (check-links 0 / gating clear / gen-index / coverage 0 gaps); R-1 (transitorio de índice por WIP ajeno MEMG-15) declarado como deuda pre-push
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan sanciona (Task 60 + stop) · D:no (pre-respondido plan F0; docs-only) · V:no (verde; R-2 = ancla corregida, no umbral) · C:no (WIP ajeno no stageado; R-1 declarado)
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · writing-guidelines, writing-plans (base type docs) · documentation-and-adrs · documentation-skill · source-driven-development
```
