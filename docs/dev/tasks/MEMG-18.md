---
title: "TASK MEMG-18: Multimodalidad — decisión + spec (extensión transversal vs 9ª dimensión)"
kind: task
description: "Decisión documentada (extensión de modalidad transversal, no 9ª dimensión) + spec mínima anclada en payload/vector/MemoryExportLine + trigger refinado con evidencia 2026; sin implementación"
---

# TASK MEMG-18: Multimodalidad — decisión + spec (extensión transversal vs 9ª dimensión)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 62, Wave F5 — Frontera & estrategia; decisión/spec docs-only)
- **Fuente:** Backlog `MEMG-18` (`docs/dev/Backlog.md:157`) + plan Task 62 (L1781-1807, bloque F0 expandido) + decisión owner 2026-09-14 (`docs/dev/backlog-futuro.md:28` FUT-15 / `docs/dev/Backlog.md:520`) + validación externa 2026-09-30 (`Backlog.md:157`) + `docs/dev/research/memg-14-marco-2.0.md` §2.2 (b)
- **Esfuerzo:** 🟡 1-2d | **Appetite:** max 2d
- **Prioridad:** 🟡 (plan) / 🟡 P2 (Backlog)
- **Tipo:** Research/spec (docs-only — cero código; cero símbolos públicos)
- **Turns estimados:** 6-10 (una sesión de sub-agente)
- **Creado:** 2026-10-06 | **last-synced:** 2026-10-06
- **Estado:** ⏳ IN PROGRESS (reservada como taskId `62` en el campaign server)
- **Incógnitas (uphill):** 0 — downhill directo (decisión → spec+trigger). Resueltas en DISCOVERY: (a) la decisión registrada 2026-09-14 es de **alcance** (fuera de v1.0) + trigger, no de framing → la decisión de este run (transversal vs 9ª dim) se documenta consistente (Gate D no disparado); (b) superficies de anclaje verificadas en código (`MemoryInput`/`MemoryRecord`/`MemoryExportLine` + `Ingestor` + `MemoryType`)
- **Pendientes (downhill):** 4 steps (1-4)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `62`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs (sin código):** master plan Task 62 (estado), Backlog `:157` (fila MEMG-18 — se elimina al cierre, Trigger 1 progreso), `backlog-futuro.md:28` + `Backlog.md:520` (FUT-15 — pointer al trigger refinado), `memg-14-marco-2.0.md` §2.2 (consumidor F5 — pointer "entregada"), avance `operaciones.md` (registro de cierre). |
| Callees | Fuentes citadas (sin modificar): `src/sdk/types/record.rs` (anclas de spec), `src/wiki/ingestors.rs`, `vanta-memory/src/core/abstractions/types.rs`, docs del marco (`memg-14-marco-2.0.md`, `NOTION-SYNC-2026-09-24.md`), URLs externas fetch-verificadas. |
| Implicaciones | **Aditivo docs-only:** `docs/dev/research/memg-18-multimodalidad.md` (nuevo; `kind: research`) + `docs/dev/tasks/MEMG-18.md` (nuevo, este archivo) + ediciones puntuales en `backlog-futuro.md`/`Backlog.md` (pointer FUT-15) + `memg-14-marco-2.0.md` (pointer) + eliminación de fila `Backlog.md:157` + registro `operaciones.md`. `gen-index --write` regenera `docs/index.md` + `llms.txt`. Sin código, sin wire, sin deps, sin locks. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-06, worktree sobre HEAD `84bbc276`; WIP MEMG-15 en vuelo desplaza líneas de `record.rs`/`types.rs` — re-verificar post-commit MEMG-15).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/dev/backlog-futuro.md` (:1-52 — FUT-15 + revisiones; formato canónico 10-col).
  - `docs/dev/Backlog.md` (:80-174 DELTA P0-P3 + `:520` FUT-15 mirror; fila `:157` MEMG-18; fila `:842` MGR-25).
  - `docs/dev/strategy/NOTION-SYNC-2026-09-24.md` (:120-144 — §13-14 syncs + "multimodal candidata" :138).
  - `docs/dev/research/memg-14-marco-2.0.md` (:1-209 — completo; §2.2 (b) multimodalidad ámbito candidato #3; §2.3 ejes; §2.6/2.7 límites).
  - `docs/dev/tasks/MEMG-14.md` (:1-187 — formato canónico de task file de la campaña).
  - `docs/dev/avance/activo/operaciones.md` (:460-488 — registro MEMG-14; formato de cierre F5).
  - `src/sdk/types/record.rs` (:100-279, :559-655 — `MemoryInput` :178, `MemoryRecord` :252, `MemoryExportLine` :562; payload String + vector + metadata).
  - `src/sdk/types.rs` (:150 — `MemoryMetadata = Fields`).
  - `src/wiki/ingestors.rs` (grep — trait `Ingestor` :50, `default_ingestors` :125, `scan_ingestable_sources` :147 — MGR-25).
  - `vanta-memory/src/core/abstractions/types.rs` (:15-59 — `MemoryType` :25: 7 tipos L1; sin tipo multimodal).
  - `docs/dev/tasks/MEMG-08.md` (grep — precedente MGR-25: metadata.source plana {file,page,chunk}).
  - `.opencode/task-system/prompts/{pipeline-full,recitation-template,question-gates}.md` (contrato de ejecución + gates).
- **Archivos referenciados hacia dentro (imports/deps):** n/a (docs; sin imports). El research-doc referenciará hacia afuera con enlaces relativos (documentation-skill §1).
- **Referencias entrantes (grep `MEMG-18|multimodal` HEAD):** `Backlog.md:157` (fila), `backlog-futuro.md:28` (FUT-15), `Backlog.md:520` (FUT-15 mirror), `memg-14-marco-2.0.md:103` (consumidor §2.2), master plan L1781-1807 + L2766 (Task 62 — **prohibido editar**), `docs/user/operations/EXPERIMENTAL_FEATURES.md:95` (multimodal en Freeze list — sin cambio), `ROADMAP-v0.7.md:67` (mención informativa), `docs/dev/research/validacion/01:72` (competidor), `archive/*` (histórico, sin cambio).
- **Veredicto impacto:** **BAJO (aditivo docs-only)** — 1 doc nuevo + 1 task file + 3 ediciones puntuales (pointer) + 1 fila Backlog eliminada al cierre + 1 registro avance. Sin cambios en código, contratos, wire, deps ni locks. Riesgos del pre-mortem mitigados: (1) sin reabrir alcance v1.0 — contrato = decisión+spec, cero código; (2) decisión consistente con la registrada 2026-09-14 (Gate D evaluado, no disparado — ver §Spec); (3) spec anclada en superficies verificadas (payload/vector/`MemoryExportLine`/`Ingestor`).

## Contrato

"decisión documentada (extensión de modalidad transversal vs 9ª dimensión) + spec mínima (qué toca: contenido multimodal en episódica/semántica + embeddings multi-modal) + trigger de reevaluación refinado (hoy: MGR-25 + caso de uso) con la evidencia 2026; sin implementación." (plan Task 62 L1790).

**Verificación del contrato (cierre):** research-doc existe con decisión (opciones consideradas + fundamento + alcance v1.0 intacto) + spec mínima (superficies ancladas con refs de código) + trigger refinado (con evidencia 2026 fetch-verificada) + gates docs 0 (`check-links` + `check-docs` + `gen-index --check`) + `validate-docs-coverage.ps1` 0 gaps.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado.** (a) Blast radius ≤ 6 archivos, docs-only, sin hot path/API pública; (b) sin símbolos públicos nuevos (cero código); (c) contrato sancionado por el plan F0 (Task 62, Gate Result ✅ DO) — sin ambigüedad nueva; (d) **la decisión registrada 2026-09-14 es de alcance (fuera de v1.0) + trigger, no de framing**: la decisión de este run (transversal vs 9ª dim) se documenta consistente con el registro (alcance intacto) → no difiere → sin question al owner (pre-mortem #2 del plan). Si la evidencia hubiera sugerido reabrir alcance → BLOQUEO, no decisión unilateral.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Framing de multimodalidad | A) **Extensión de modalidad transversal** (modalidad = eje de forma del contenido; payload+vector+metadata; sin dim nueva) / B) 9ª dimensión explícita (sensorial) (contra: mezcla ejes — D1-D4 cognitivas, D5-D8 ingeniería; "sensory" en la evidencia = procesamiento perceptual, no función persistente) / C) almacén sensorial dedicado (ya descartado 2026-09-14) | ✅ **A** — decidido-por-evidencia: marco núcleo+extensiones (MEMG-14 §2.2 ubica multimodalidad como ámbito candidato, no dimensión); materialización industrial = Resource Memory + embeddings multimodales (MIRIX 2507.07957), no dimensión cognitiva |
| 2 | Hogar del doc | A) **research-doc nuevo `docs/dev/research/memg-18-multimodalidad.md` (kind=research)** / B) sección en `memg-14-marco-2.0.md` (mezcla F5) / C) `docs/dev/strategy/` (no es hogar de investigaciones) | ✅ **A** — patrón `memg-14-*.md`/`mgr-*.md`; `kind: research` → ruta canónica (documentation-skill §2.1) |
| 3 | Alcance del doc | A) **decisión + spec mínima + trigger refinado + evidencia 2026** (contrato completo) / B) solo decisión (deja spec/trigger sin entregable) | ✅ **A** — contrato L1790 enumera los 3 entregables |
| 4 | Registro FUT-15 | A) **pointer al doc en `backlog-futuro.md:28` + `Backlog.md:520`** (el registro conserva su trigger original + refinamiento trazable) / B) reescribir el trigger en las filas (pierde el registro owner 2026-09-14) | ✅ **A** — no se re-litiga el registro; el refinamiento vive en el doc y las filas apuntan |
| 5 | ADR | A) **research-doc (sin ADR)** — la decisión no está sancionada para implementación y Regla 5 (forcing function: el ADR lo articula el humano) / B) ADR nuevo (excede: no hay decisión de implementación que registrar) | ✅ **A** — patrón MEMG-14 (marco → research-doc) |
| 6 | Verificación de fuentes | A) **fetch de cada URL citada en DISCOVERY (TSYS-13); solo se citan las resueltas** / B) citar y marcar todo `[a verificar]` | ✅ **A** — red disponible; 3/3 arXiv resueltas (2602.06052 v4 TMLR / 2507.07957 MIRIX / 2512.13564 v2) |

## Invariantes de dominio (handoff — MUST)

1. **El alcance v1.0 no se reabre** (pre-mortem #1 del plan): el contrato es decisión+spec, cero código. Multimodal sigue fuera de v1.0 con trigger (registro owner 2026-09-14 intacto).
2. **La decisión documentada es consistente con el registro** — cualquier cambio de alcance futuro es del owner (Gate D/no unilateral).
3. **Spec anclada en superficies reales** (`src/sdk/types/record.rs` :178/:252/:562; `src/wiki/ingestors.rs:50`; `vanta-memory/src/core/abstractions/types.rs:25`) — sin inventar APIs; refs `[a verificar]` = 0.
4. **Regla 11:** todo claim con fuente reproducible (repo path o URL fetch-verificada); MIRIX +35%/−99.9% citado del abstract fetch-verificado.
5. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno (MEMG-15: `docs/api/**`, `src/**`, `Backlog.md` dirty) no se stagea; commit con **pathspec** y staging por hunks si `Backlog.md` sigue dirty (coordinación MEMG-15).
6. **Frontera de este run:** cero implementación; cero símbolos públicos; el doc NO es normativo para código hasta que el trigger dispare.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código, sin `unsafe`, sin deps. El run **elimina** deuda de trazabilidad: la decisión multimodal deja de ser un trigger sin spec y queda auditable desde el repo (decisión + spec + trigger refinado). Deuda declarada: ninguna nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Research-doc con decisión (opciones + fundamento + alcance intacto) + spec mínima (superficies ancladas) + trigger refinado (evidencia 2026) verificables 1:1 contra el contrato L1790; fuentes fetch-verificadas; gates docs 0; task file completo (Impacto Regla 0 + Review P2-01) |
| **Commit** | Commit atómico conventional `docs(research):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin changelog) |

## Herramientas necesarias

- `webfetch` (verificación TSYS-13 de URLs citadas) + `codegraph_codegraph_explore`/grep/read (anclas de código) + `campaign_*` (estado/scope/verify)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre) + fork `vanta-review` (P2-01)

**Skills cargadas (SDP v3):** `campaign-executor` · `progreso` · `ponytail` (base auto) · `writing-guidelines` (base type docs) · `documentation-and-adrs` (keyword docs) · `documentation-skill` (obligatoria docs/**) · `source-driven-development` (verificación de fuentes) · `spec-driven-development` (lifecycle DEFINE — spec mínima) · `coordinated-web-search` (router obligatorio de búsqueda web). SDP phase=DEFINE devolvió además `writing-plans`/`interview-me`/`idea-refine` — excluidas: contrato sancionado por el plan F0 (sin ambigüedad que interviewar; sin plan multi-paso nuevo). `test-driven-development`/`systematic-debugging` excluidas: sin lógica ni bug.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — N/A: docs-only, sin trust boundary nuevo, sin input de usuario, sin deps, sin red en producto (los fetches de verificación son de DISCOVERY, solo lectura).
- [ ] **PERFORMANCE** — N/A: sin hot paths, sin código; Regla 9 no dispara (sin claim de optimización).

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato)

- **Archivos:** `docs/dev/tasks/MEMG-18.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (FUT-15, Backlog:157, memg-14 §2.2, superficies de código); resolución de la decisión registrada (alcance, no framing) + Gate D evaluado; verificación TSYS-13 de las URLs clave (3/3); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + `campaign_validate_scope` OK (advisory)
- **Evidencia:** ✅ este archivo; URLs verificadas en §Notas (títulos registrados)
- **Estado:** ✅ COMPLETED

### Step 2 — Research-doc: decisión + spec + trigger

- **Archivos:** `docs/dev/research/memg-18-multimodalidad.md` (nuevo)
- **Acción:** redactar el research-doc: §1 decisión (opciones A/B/C + fundamento + alcance v1.0); §2 evidencia 2026 fetch-verificada; §3 spec mínima (ingesta/storage/embeddings/recuperación/export/gobernanza/evaluación + lo que NO toca); §4 trigger refinado; §5 límites; §6 fuentes.
- **Verify:** `node scripts/docs/check-docs.mjs` (frontmatter/kind OK)
- **Evidencia:** ✅ doc creado (`docs/dev/research/memg-18-multimodalidad.md`): §0 resumen + §1 decisión (A/B/C + fundamento 4 puntos + alcance v1.0 intacto) + §2 evidencia 2026 (2.1 surveys / 2.2 competidores / 2.3 embeddings — 9 fuentes fetch-verificadas) + §3 spec mínima (3.1-3.8, anclas `record.rs:178/252/562`, `types.rs:150`, `ingestors.rs:50`) + §4 trigger refinado (3 condiciones) + §5 límites + §6 fuentes. `check-docs`: GATING all clear (kind/path OK); `check-links`: 0 broken. **Profundización multi-fuente (regla owner 2026-10-06):** +6 fuentes de competidores/stack (Mem0/Cognee/Letta + jina-clip-v2/Voyage) — tabla en §Notas.
- **Estado:** ✅ COMPLETED

### Step 3 — Pointer FUT-15 + consumidor F5 + gates docs

- **Archivos:** `docs/dev/backlog-futuro.md` (:28), `docs/dev/Backlog.md` (:520), `docs/dev/research/memg-14-marco-2.0.md` (§2.2), `docs/index.md` + `llms.txt` (generados)
- **Acción:** pointer al doc (trigger refinado) en las filas FUT-15; pointer "entregada" en memg-14 §2.2; `gen-index --write`.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` exit 0
- **Evidencia:** ✅ pointers aplicados (`backlog-futuro.md:28` + `Backlog.md:520` FUT-15 → doc; `memg-14-marco-2.0.md` §2.2 item 3 → "entregadas" + link); `gen-index --write` regeneró 6 índices (delta propio: `docs/index.md` 1557→1559 docs / Research 68→69; `llms.txt` 458→459 páginas + 1100 task files; `docs/api/index.md` regenerado — pertenece al blast radius de MEMG-15, no se stagea); `gen-index --check` exit 0; `check-links` 0 broken; `check-docs` GATING all clear; `validate-docs-coverage.ps1` 0 gaps.
- **Estado:** ✅ COMPLETED

### Step 4 — Verify full + OCR + Review P2-01 + commit local + cierre campaign

- **Archivos:** `docs/dev/tasks/MEMG-18.md` (§Review + RESULTADO), `docs/dev/Backlog.md` (eliminar fila MEMG-18), `docs/dev/avance/activo/operaciones.md` (registro de cierre)
- **Acción:** `pwsh scripts/validate-docs-coverage.ps1` · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`) → Critical/High bloquean; Medium → FIND. Clasificar tier HARD-02: paths `docs/**` → **Fast**. Fork `vanta-review` (fresh context) para P2-01. Commit **LOCAL** `docs(research):` con pathspec (+ staging por hunks si `Backlog.md` sigue dirty por MEMG-15); cierre `campaign_update_task_state(completed, taskId:"62")` con recitation + payload `review`; Trigger 1 progreso (fila Backlog eliminada + registro avance).
- **Verify:** veredicto registrado + `git show --stat HEAD` limitado a archivos propios + `git status` sin WIP ajeno stageado
- **Evidencia:** ⬜
- **Estado:** ⬜ PENDING

## Dependencias

- ✅ Evidencia base: decisión owner 2026-09-14 (FUT-15) · validación externa 2026-09-30 (`Backlog.md:157`; arXiv 2602.06052/2507.07957/2512.13564) · marco 2.0 (`memg-14-marco-2.0.md` §2.2) · MGR-25 parcial (`src/wiki/ingestors.rs`; FIND-298).
- Consumidores: MEMG-14(b) (ámbitos candidatos — pointer), FUT-15 (trigger refinado), MEMG-19 (descartes — dominio vecino).
- nextTask: lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (HARD-02: paths `docs/**`; verify mecánico + veredicto; sin adversarial completo). Sesión reviewer: `ses_ef09584d6ffeDoCq87U0ZNDreX`. Ronda 1 (review completo): ✅ **APPROVE** (0 Required; 1 Optional + 3 NIT). Delta re-review (fixes + profundización multi-fuente): ✅ **APPROVE** final (0 Required; 1 Optional + 1 NIT — aplicados).
- **Enfoque:** contrato L1790 1:1 (decisión/spec/trigger/sin-implementación/sin-reapertura); decisión vs registro 2026-09-14; anclas de código; evidencia multi-fuente (8 fetches live del reviewer); gates 4/4; scope docs-only.
- **Cómo se probó:** reviewer re-corrió `check-links` (exit 0) · `check-docs` (exit 0) · `gen-index --check` (exit 0) · `validate-docs-coverage` (0 gaps); re-fetch live de las 9 fuentes externas (arXiv 3/3 + Mem0/Cognee/Letta/jina/Voyage); verificación de anclas (`record.rs:178/252/562`, `types.rs:150`, `ingestors.rs:50`, `MemoryType:25`); lectura completa de ambos artefactos; `git diff` scope check (src/ sin hits de modal).
- **Hallazgos + disposición:**
  1. [OPTIONAL r1] O-1 — anclas etiquetadas "HEAD 84bbc276" eran del worktree (WIP MEMG-15 desplaza líneas) → **re-etiquetadas** "worktree 2026-10-06 + re-verificar post-commit MEMG-15" (doc §3/§6 + task file).
  2. [NIT r1] N-1 — tercera condición del trigger (embedder) sin aclarar → **aclarado**: condiciones owner = (1)+(2); (3) = prerrequisito técnico declarado.
  3. [NIT r1] N-2 — RESULTADO §7 desincronizado (provisional) → **actualizado en el cierre**.
  4. [NIT r1] N-3 — cita del contrato sin "(hoy:" → **corregida** (match plan L1790).
  5. [OPTIONAL delta] O-2 — cita textual de Cognee no localizable en la fuente citada → **reemplazada** por citas verbatim verificadas ("turns audio and images into the text it then processes"; "both are ordinary text…").
  6. [NIT delta] N-2(d) — quote de jina-clip-v2 estaba en el release post, no en la ficha del modelo → **parafraseada** (sin comillas).
- **Veredicto:** ✅ **APPROVE** (ronda 1 + delta final) — contrato 1:1, decisión sin reapertura, evidencia multi-fuente verificada, gates verdes; listo para commit local (Step 4).

## Context Save Point

- **Última acción:** Steps 1-3 ✅ — DISCOVERY + task file + research-doc + pointers FUT-15/memg-14 + gates docs 4/4 verdes (`check-links` 0 broken · `check-docs` clear · `gen-index --check` exit 0 · `validate-docs-coverage` 0 gaps) + OCR delegation N/A (diff docs-only; reviewables = WIP ajeno MEMG-15).
- **Próximo paso:** Step 4 — review P2-01 (fork `vanta-review`) → commit local `docs(research):` → cierre campaign taskId `62` + Trigger 1 progreso.
- **Estado del worktree:** HEAD `84bbc276`; WIP ajeno (MEMG-15: `docs/api/**`, `src/**`, `Backlog.md` dirty con remoción fila MEMG-15 + FIND-309..311, `opencode.jsonc`, master plan, `dev-tools/heavy-test-lock.ps1`, `docs/dev/tasks/MEMG-15.md`) NO se stagea; prohibido tocar `docs/pipeline-state.json`.

## Notas

- **Fuentes verificadas y fechadas (multi-fuente — regla owner 2026-10-06; todas fetch-verificadas el 2026-10-06, TSYS-13):**

  | URL | Fecha | Claim que respalda |
  |---|---|---|
  | `arxiv.org/abs/2602.06052` | v4, 2026-08-04 (TMLR) | Mecanismo cognitivo incluye "sensory"; sustrato paramétrico/externo — evidencia de que la modalidad es capacidad, no dimensión |
  | `arxiv.org/abs/2507.07957` | 2025-07-10 | MIRIX: 6 tipos incl. Resource Memory; +35% ScreenshotVQA vs RAG, −99.9% storage — implementación de referencia transversal |
  | `arxiv.org/abs/2512.13564` | v2, 2026-01-13 | "Multimodal memory" = frontera emergente (dirección futura) — respalda "fuera de v1.0 con trigger" |
  | `docs.mem0.ai/open-source/features/multimodal-support` | live 2026-10-06 | Mem0: facts extraídos de imágenes → "stored as standard memories" (transversal, sin tipo nuevo) |
  | `docs.cognee.ai/guides/multimedia-audio-image-processing` | live 2026-10-06 | Cognee: audio/imagen → texto ("turns audio and images into the text it then processes") → mismo pipeline (transversal convert-to-text) |
  | `docs.letta.com/v1-sdk/messages/image-inputs` | live 2026-10-06 | Letta: imágenes = contenido de mensaje para LLM multimodal (capa mensaje, no memoria) |
  | `docs.letta.com/v1-sdk/memory/archival-memory` | live 2026-10-06 | Letta archival memory = store semántico de texto (sin dimensión multimodal) |
  | `jina.ai/models/jina-clip-v2/` | live 2026-10-06 | Embedder texto+imagen conjunto, Matryoshka 1024→64 — disponibilidad para §3.3 |
  | `www.mongodb.com/docs/voyageai/models/multimodal-embeddings/` | live 2026-10-06 | Shared vector space texto/imagen/video; dims 256–2048 — invariante de espacio por colección |

  (Las 3 arXiv también re-verificadas por el reviewer P2-01 en vivo — 3/3 OK.)
- **Coordinación MEMG-15 (en vuelo):** `docs/dev/Backlog.md` dirty (remoción fila MEMG-15 + FIND-309..311). Al cierre: releer fresco; staging por hunks para commitear SOLO la remoción de MEMG-18 + pointer FUT-15 si sigue dirty (o full file si ya commiteó).
- **Precedente workers↔Notion:** `mgr-23-24:11` — workers no operan Notion; sin sync en esta tarea (el doc es el entregable; el draft de sync de MEMG-14 §5 es el canal).
- **NOTICED BUT NOT TOUCHING:** `EXPERIMENTAL_FEATURES.md:95` (multimodal en Freeze list — sigue vigente, sin cambio); `ROADMAP-v0.7.md:67` (mención informativa); `docs/dev/research/validacion/01:72` (competidor); archive/* (histórico); `docs/api/MEMORY_INTERCHANGE_FORMAT.md` (WIP MEMG-15 — no se toca).

## RESULTADO §7 (contrato de retorno — pipeline-full)

> Provisional — se actualiza en Step 4 (cierre).

```
RESULTADO: 🟡 INCOMPLETO
STEPS_OK: 1/4 total steps
PROXIMO_STEP: Step 2 — redactar docs/dev/research/memg-18-multimodalidad.md
COMMIT_HASH: ninguno
ARCHIVOS: docs/dev/tasks/MEMG-18.md (nuevo)
VERIFY_CONTRATO: no-corrido (Step 1: task file creado; contrato se verifica en Step 4)
BLOQUEO: ninguno
GATES_EVALUADOS: P:no(plan sanciona Task 62) D:no(docs-only; sin símbolos; decisión consistente) V:no C:no
SKILLS_CARGADAS: campaign-executor, progreso, ponytail (base) · writing-guidelines · documentation-and-adrs · documentation-skill · source-driven-development · spec-driven-development · coordinated-web-search
```
