---
title: "TASK STRAT-06: Research licencias (Khoj/Jan/Reor/OpenWebUI/Letta) + ACV OSS DBs"
kind: task
description: "Research de licencias comparadas (grupo local-first AI + OSS DBs) y ACV/bands de OSS DBs con fuente por número o marcado modelado; implicaciones open-core/OEM; 32 fuentes fetch-verificadas 2026-10-06"
---

# TASK STRAT-06: Research licencias (Khoj/Jan/Reor/OpenWebUI/Letta) + ACV OSS DBs

- **Fecha:** 2026-10-06 · **Tipo:** research (docs-only — cero código, cero símbolos públicos) · **Cero implementación** (el contrato es un research doc)
- **Contrato (plan Task 66 L1895):** "research doc con licencias comparadas (Khoj/Jan/Reor/OpenWebUI/Letta + OSS DBs relevantes) + ACV/bands de OSS DBs con fuente por número (o marcado "modelado") + implicaciones para el modelo open-core; fuentes citadas (Regla 11)."
- **Origen:** plan `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` Task 66 (L1886-1912, bloque F0 expandido) + Backlog `docs/dev/Backlog.md:168` (fila STRAT-06, removida al cierre — Trigger 1 progreso) + precedentes `docs/dev/research/validacion/01`, `02`, `03`, `08`, `00-SINTESIS-EJECUTIVA2.md` + `docs/dev/Backlog.md:173` (nota DELTA: "los bands de revenue son modelado, no datos").
- **Alcance:** licencias comparadas del grupo local-first AI (Khoj/Jan/Reor/OpenWebUI/Letta) + OSS DBs relevantes, ACV/bands con separación estricta **dato verificado vs modelado**, e implicaciones para el modelo open-core/OEM de VantaDB. **No** es plan GTM ni re-litiga la decisión Apache-2.0 + CLA (la cita). **No** toca código.

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 66, bloque F0)
- **Fuente:** Backlog `:168` (fila STRAT-06) + plan Task 66 L1886-1912 + `Backlog.md:173` (nota DELTA modelado)
- **Esfuerzo:** 🟡 2d (research) | **Appetite:** max 3d ✓
- **Prioridad:** 🟢 (plan) / 🔵 P3 (Backlog)
- **Tipo:** Research (docs-only)
- **Turns estimados:** 8-12 (una sesión de sub-agente)
- **Creado:** 2026-10-06 | **last-synced:** 2026-10-06
- **Estado:** ✅ COMPLETED (2026-10-06 — review P2-01 vanta-review delta APPROVE; gates docs 0; commits locales `docs(research):` + `docs(avance):`; campaign taskId `66` para el cierre — instrucción del orquestador; `validate_scope`/`analyze_task` resuelven por texto `STRAT-06`, el numérico no resuelve en esos parsers — precedente STRAT-05/MEMG-19)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY: (a) licencias del grupo local-first → fetch-verificadas 2026-10-06 (Khoj AGPL-3.0; Jan Apache-2.0; Reor AGPL-3.0 archivado; OpenWebUI custom BSD-3+branding; Letta Apache-2.0); (b) ACV → datos públicos con fuente (ClickHouse/Supabase/MongoDB/Elastic/Neo4j/Redis) + bands modeladas marcadas; (c) revenue no público → columna `tipo` (dato/reportado/modelado) por fila
- **Pendientes (downhill):** 3 steps (1-3)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `66`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Lectores de docs (sin código):** master plan Task 66 (estado — **prohibido editar**), `Backlog.md:168` (fila STRAT-06 — se elimina al cierre, Trigger 1 progreso), `Backlog.md:173` (nota DELTA — se cita), decisiones open-core citadas (`00-SINTESIS-EJECUTIVA2.md` Q3, `.opencode/rules/open-core-licensing.md` — intactas), avance de dominio (registro de cierre). |
| Callees | Fuentes citadas (sin modificar): `validacion/01` (Letta Apache-2.0 $20; pricing competidores), `validacion/02` (modelos negocio OSS DBs), `validacion/03` (tabla SSPL/dual licensing), `validacion/08` ($79 desktop, Lemon Squeezy), `00-SINTESIS-EJECUTIVA2.md` (Apache-2.0 hoy + CLA ligero); 32 URLs externas fetch-verificadas (2026-10-06). |
| Implicaciones | **Aditivo docs-only:** `docs/dev/research/strat-06-licencias-acv-oss.md` (nuevo; `kind: research`) + `docs/dev/tasks/STRAT-06.md` (nuevo, este archivo) + fila `Backlog.md:168` eliminada al cierre + registro `avance` al cierre. Sin código, sin wire, sin deps, sin locks, sin migración. |

## Impacto mapeado (Regla 0)

> Gate previo a la primera edición — poblado en DISCOVERY (2026-10-06, HEAD `06b183ef`, branch `develop`; WIP ajeno: `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` + `opencode.jsonc` modificados y `dev-tools/heavy-test-lock.ps1` untracked — **no se tocan ni se stagean**).

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 66 L1886-1912 + reglas de ejecución L2080-2095 — regla investigación profunda owner 2026-10-06).
  - `docs/dev/research/validacion/01-competidores-memoria-agentes-ia.md` (completo — Letta `:141`; tabla consolidada; patrones de mercado).
  - `docs/dev/research/validacion/02-bases-datos-embebidas-modelos-negocio.md` (completo — modelos de negocio OSS DBs; tabla comparativa; lecciones).
  - `docs/dev/research/validacion/03-licenciamiento-y-monetizacion-open-core.md` (completo — tabla de licencias SSPL/BSL/FSL/dual; casos Elastic/Redis/MongoDB/HashiCorp).
  - `docs/dev/research/validacion/08-monetizacion-local-first-sin-capital.md` (completo — MoR, Desktop Pro $79, OEM $500-2k, Apache-2.0 + CLA).
  - `docs/dev/research/validacion/00-SINTESIS-EJECUTIVA2.md` (completo — decisión Q3: Apache-2.0 hoy + CLA ligero; fase 2 escalera).
  - `docs/dev/Backlog.md` (:160-189 — fila STRAT-06 `:168` + nota DELTA `:173`).
  - `.opencode/rules/open-core-licensing.md` (completo — 4 reglas del modelo open-core vigente).
  - `.opencode/task-system/prompts/task.md` + `pipeline-full.md` (formato canónico), `docs/dev/tasks/STRAT-05.md` + `docs/dev/research/strat-05-object-storage.md` (formato de precedente F0 aprobado).
- **Archivos referenciados hacia dentro (imports/deps):** n/a (docs; sin imports). El research-doc referenciará hacia afuera con enlaces relativos (documentation-skill §1).
- **Referencias entrantes (grep `STRAT-06|Khoj|Jan|Reor|OpenWebUI` HEAD `06b183ef`):** `Backlog.md:161` (nota DELTA) y `:168` (fila), master plan L1886-1912 (**prohibido editar**), `validacion/00-SINTESIS-EJECUTIVA2.md` (cita Apache-2.0 + CLA). Research previo de **producto** (no de licencias/ACV): `human-facing-db-ui/03-ai-memory-graphs/RESEARCH.md:52,170,174` (Khoj — 4/10) y `human-facing-db-ui/02-desktop-db-tools/RESEARCH.md:46,113` (Open WebUI); Letta en `02-desktop-db-tools/RESEARCH.md:45,113,114` y `validacion/01`; Jan/Reor sin research de producto. La frontera de este run (licencias comparadas + ACV) no estaba cubierta — sin duplicación. Sin referencias de código.
- **Veredicto impacto:** **BAJO (aditivo docs-only)** — 1 research doc + 1 task file + 1 fila Backlog eliminada al cierre + 1 registro avance. Sin cambios en código, contratos, wire, deps ni locks. Pre-mortem mitigado: (1) revenue no público → columna `tipo` dato/reportado/modelado por fila, cero números inventados; (2) scope → research, no plan GTM; (3) staleness → fecha de verificación por fila.

## Contrato

"research doc con licencias comparadas (Khoj/Jan/Reor/OpenWebUI/Letta + OSS DBs relevantes) + ACV/bands de OSS DBs con fuente por número (o marcado "modelado") + implicaciones para el modelo open-core; fuentes citadas (Regla 11)." (plan Task 66 L1895).

**Verificación del contrato (cierre):** research-doc existe con (a) **licencias comparadas** de los 5 productos nombrados (fetch-verificadas 2026-10-06, con fecha por fila) + OSS DBs relevantes; (b) **ACV/bands de OSS DBs** con fuente por número y columna `tipo` (dato/reportado/modelado) — sin números sin fuente; (c) **implicaciones para el modelo open-core** (citando la decisión Apache-2.0 + CLA sin re-litigarla); (d) fuentes citadas con fecha (Regla 11); gates docs 0 (`check-links` + `check-docs` + `gen-index --check`) + `validate-docs-coverage.ps1` 0 gaps; cero código tocado.

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado.** (a) Blast radius ≤4 archivos propios, docs-only, sin hot path/API pública; (b) sin símbolos públicos nuevos (cero código); (c) contrato sancionado por el plan F0 (Task 66, Gate Result ✅ DO) — sin ambigüedad nueva; (d) la decisión Apache-2.0 + CLA ya está tomada y registrada — este run **la fundamenta con evidencia externa**, no la re-litiga.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Hogar del research doc | A) **`docs/dev/research/strat-06-licencias-acv-oss.md` (kind=research)** / B) sección en `validacion/03` (contra: `03` es 2026-08-25 y su scope es monetización, no ACV/licencias del grupo local-first) / C) `docs/dev/strategy/` (no es hogar de investigaciones) | ✅ **A** — patrón `strat-05-*`/`memg-*`; `kind: research` → ruta canónica (documentation-skill §2.1) |
| 2 | Separación dato/modelado | A) **Columna `tipo` por fila (dato / reportado / modelado) + fecha de verificación por fila** / B) prosa con aclaraciones | ✅ **A** — pre-mortem #1 del plan; nota DELTA `Backlog.md:173` ("bands de revenue son modelado, no datos"); Regla 11 |
| 3 | Fuentes de ACV | A) **Públicas con fuente: earnings (MongoDB/Elastic), prensa oficial (Neo4j/Redis), estimaciones de research firm citadas como tales (Sacra/Dealroom)** / B) solo oficiales (contra: deja fuera ClickHouse/Supabase) / C) estimaciones sin etiqueta (contra: viola pre-mortem) | ✅ **A** — cada número con fuente + fecha + nivel de confianza; "reportado/estimado" explícito |
| 4 | Bands de ACV | A) **Derivadas de los datos públicos (definiciones de cohorte ≥$100k/≥$1M como ancla) y marcadas `modelado (derivado)`** / B) bands "de la industria" sin derivación (contra: no auditable) | ✅ **A** — bandas self-serve/SMB/mid/enterprise/top con la derivación visible |
| 5 | Alcance del grupo local-first | A) **Los 5 nombrados en el contrato (Khoj/Jan/Reor/OpenWebUI/Letta) + OSS DBs relevantes del precedente (Turso/DuckDB/Chroma/Qdrant/Weaviate/MongoDB/Elastic/Redis/Neo4j/ClickHouse/Supabase)** / B) ampliar a todo el ecosistema AI-local (contra: scope creep) | ✅ **A** — contrato 1:1; el resto queda citado vía precedentes |
| 6 | Verificación de fuentes (TSYS-13) | A) **Fetch de cada URL citada en DISCOVERY; solo se citan las resueltas, con fecha** / B) citar y marcar `[a verificar]` | ✅ **A** — red disponible; 32/32 URLs resueltas (fetch 2026-10-06); 2 fuentes secundarias marcadas como tales (agenticindex→confirmada por primaria; billiondollarpitchdecks→confianza media) |
| 7 | Tono / alcance | A) **No normativo; research puro; las implicaciones citan la decisión vigente (Apache-2.0 + CLA) y proponen gatillos, no cambios** / B) proponer cambio de licencia | ✅ **A** — contrato "research, no plan"; la decisión de licencia ya existe (Q3 síntesis + regla open-core) y no se re-litiga |

## Invariantes de dominio (handoff — MUST)

1. **Decisión vigente intacta:** el core `vantadb` permanece Apache-2.0 + CLA ligero (decisión Q3 `00-SINTESIS-EJECUTIVA2.md` + `.opencode/rules/open-core-licensing.md`); este run la **fundamenta con evidencia externa**, no la modifica ni la propone cambiar.
2. **Cero números sin fuente:** todo número del research doc lleva fuente + fecha + tipo (dato/reportado/modelado). Los "bands" derivados se marcan `modelado (derivado)`.
3. **Cero código:** sin `pub fn`/tipos/bindings; sin deps; sin cambios en `Cargo.toml`/`deny.toml`.
4. **No re-litigar:** `validacion/03` (FSL) y la corrección de `validacion/08` (Apache-2.0 hoy) se citan; el research no reabre esa decisión.
5. **No tocar:** master plan, `opencode.jsonc`, `docs/pipeline-state.json` (prohibidos). WIP ajeno no se stagea; commit con **pathspec**.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** ≤0 — sin código, sin `unsafe`, sin deps. El run **elimina** deuda de decisión: las decisiones open-core/OEM dejan de estar "sin base citada" (Gate Justificación del plan) y quedan con licencias comparadas + ACV con fuente. Deuda declarada: ninguna nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Research-doc con (a) licencias comparadas de los 5 + OSS DBs (fecha por fila) + (b) ACV/bands con fuente por número o marcado modelado + (c) implicaciones open-core/OEM; verificable 1:1 contra el contrato L1895; gates docs 0; task file completo (Impacto Regla 0 + Spec + Review P2-01) |
| **Commit** | Commit atómico conventional `docs(research):` + pathspec solo de archivos propios (sin WIP ajeno) + verificación mecánica (nunca auto-reporte); **LOCAL** (⛔ nunca push) |
| **Release** | n/a (docs; sin changelog) |

## Herramientas necesarias

- `webfetch` (fetch-verificación TSYS-13 — GitHub API para licencias, páginas oficiales para pricing/ARR) + skill `coordinated-web-search` (router obligatorio) + rg/read (evidencia interna) + `campaign_*` (estado/scope/verify)
- Gates docs: `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write` + `pwsh scripts/validate-docs-coverage.ps1`
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre) + fork `vanta-review` (P2-01)

**Skills cargadas (SDP v3):** base auto (`campaign-executor` · `progreso` · `ponytail`) · `writing-guidelines` + `writing-plans` (base type Documentation) · `documentation-skill` (obligatoria `docs/**`) · `documentation-and-adrs` (keyword-mapped — research doc/decisiones) · `source-driven-development` (verificación de fuentes) · `coordinated-web-search` (router obligatorio — regla investigación profunda owner 2026-10-06). Excluidas con justificación: `incremental-implementation`/`test-driven-development`/`context-engineering` (lifecycle BUILD — docs-only, sin lógica ni tests), `systematic-debugging` (no hay bug), `doubt-driven-development` (el review P2-01 lo ejecuta `vanta-review` en contexto fresco).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — N/A en este run (docs-only, sin trust boundary nuevo, sin input de usuario, sin deps; los fetches son de DISCOVERY, solo lectura). El research **documenta** la relación licencia/marca/OEM como material estratégico, no operativo.
- [ ] **PERFORMANCE** — N/A: sin hot paths, sin código; Regla 9 no dispara (sin claim de optimización).

## Steps

### Step 1 — DISCOVERY + task file (Regla 0 + Spec + contrato + investigación profunda)

- **Archivos:** `docs/dev/tasks/STRAT-06.md` (nuevo, este archivo)
- **Acción:** evidencia anclada (plan Task 66, precedentes `validacion/01/02/03/08/00`, `Backlog.md:168,:173`, regla open-core); Gate D evaluado; SDP v3; investigación profunda multi-fuente (32 fuentes fetch-verificadas 2026-10-06 — regla owner); formato canónico con Impacto Regla 0 + Spec + DoD.
- **Verify:** task file existe + `campaign_validate_scope` OK (advisory)
- **Evidencia:** ✅ este archivo; fuentes verificadas en §Notas
- **Estado:** ✅ COMPLETED

### Step 2 — Research-doc: licencias comparadas + ACV/bands + implicaciones

- **Archivos:** `docs/dev/research/strat-06-licencias-acv-oss.md` (nuevo, `kind: research`)
- **Acción:** doc en español con: §0 resumen ejecutivo (tabla de respuestas); §1 licencias comparadas (1.1 grupo local-first AI Khoj/Jan/Reor/OpenWebUI/Letta; 1.2 OSS DBs relevantes; 1.3 lectura para VantaDB); §2 ACV/bands (2.1 datos verificados con fuente+fecha por número; 2.2 bands `modelado (derivado)`; 2.3 implicaciones open-core/OEM); §3 fuentes (tabla con fecha); §4 lagunas.
- **Verify:** doc existe; `rg -c "modelado"` > 0 y `rg -c "Khoj|OpenWebUI"` > 0; secciones completas; gates docs corren al cierre.
- **Estado:** ✅ COMPLETED (2026-10-06 — doc escrito; `rg -c "modelado"`=12 · `rg -c "Khoj|OpenWebUI"`=17; `check-links` exit 0 · `check-docs` GATING clear · `gen-index --check` exit 0 · `validate-docs-coverage` 0 gaps)

### Step 3 — Verificación, review P2-01 y cierre

- **Archivos:** este task file (sync de estados) + índices regenerados si difieren
- **Acción:** gates docs 0 (`check-links` + `check-docs` + `gen-index --write`/`--check`) + `validate-docs-coverage.ps1`; OCR delegation (`ocr-review.ps1 -Format json`); review P2-01 (fork `vanta-review` — contexto fresco, adversarial); commit **LOCAL** `docs(research):` con pathspec; cierre campaign `taskId: "66"` con payload `review` (HARD-07); `skill progreso` (Trigger 1 — elimina fila Backlog `:168` + registro avance).
- **Verify:** gates 0 + veredicto review registrado + commit local + campaign `completed` (`updated:true`).
- **Estado:** ✅ COMPLETED (2026-10-06 — gates docs 4/4 + OCR N/A (diff docs-only) + review P2-01 vanta-review r1 changes-required → fixes → delta r2 APPROVE; commits locales `docs(research):` + `docs(avance):`; campaign taskId `66` → completed)

## Dependencias

- Task 64 (STRAT-04, WASM threads — en vuelo, otra área): **sin dependencia mutua** (áreas disjuntas: WASM vs research estratégico; releído fresco; pathspec en el commit).
- Task 65 (STRAT-05, object storage): ✅ COMPLETED — precedente de formato (task file + research doc + cierre).
- Consumidores aguas abajo: decisiones open-core/OEM (`00-SINTESIS-EJECUTIVA2.md` Q3/Q4; `.opencode/rules/open-core-licensing.md`) — este research les da base citada; no se tocan sus archivos.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador (`vanta-review` — fork en contexto fresco). Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** `vanta-review` — P2-01, contexto fresco (no participó de la implementación); tier **Fast** (paths del diff: `docs/dev/**` → verify mecánico + veredicto registrado).
- **Enfoque:** contraste de licencias/ACV contra fuentes re-fetcheadas + separación dato/reportado/modelado + scope (research, no GTM) + higiene de commit (WIP ajeno) + checklist anti-hábitos.
- **Cómo se probó (ronda 1):** 13 fetches externos re-ejecutados (GitHub API ×5: khoj/jan/reor/open-webui/letta; LICENSE ×3: jan/open-webui/weaviate; `app.khoj.dev` sunset; `docs.openwebui.com/license` + `/enterprise`; `docs.letta.com/pricing`; `jan.ai`; SEC 10-Q MDB; archivo Neo4j) → 11/11 spot-checks coinciden con el doc, 0 contradicciones; gates re-corridos (`check-links`/`check-docs`/`gen-index --check` exit 0); `git status`/`git diff`/`git grep` sobre HEAD `06b183ef`.
- **Hallazgos ronda 1 + disposición:**
  1. **[High → resuelto] F1:** `docs/index.md` regenerado contenía la fila de `STRAT-04.md` (WIP ajeno untracked) → `docs/index.md` **excluido del pathspec** (única opción limpia — hand-editar el índice rompería `gen-index --check`; `llms.txt` verificado sin STRAT-04); deuda transitoria declarada (regeneración conjunta al cierre de STRAT-04).
  2. **[Medium → resuelto] F2:** "1M+ downloads" de Jan stale y sin fuente → "6.8M+ (jan.ai, 2026-10-06)" + fuente §4 #4.
  3. **[Medium → resuelto] F3:** claim "0 hits en docs/" falso → corregido con evidencia precisa (research de producto previo: Khoj `03:52,170,174`; Open WebUI `02:46,113`; Letta `02:45,113,114` + `validacion/01`; la frontera licencias/ACV no estaba cubierta).
  4. **[Low → resuelto] F4:** fila Redis → `dato (ARR) + reportado (conteos)`.
  5. **[Low → resuelto] F5:** §4 con URLs completas; **sub-claim del reviewer rechazado con evidencia** (permalink Neo4j resuelve — re-fetcheado 2026-10-06; sus 404 eran slugs inventados); #23 anotado como corroboración de serie; §2.1 MongoDB cita solo el 10-Q.
- **Checklist anti-hábitos tóxicos:** ✅ — 11/11 spot-checks coinciden (sin salidas inventadas); 0 claims sin fuente tras fixes; steps conectados al contrato (sin huérfanos); SDP cubierto (§Herramientas); alternativas evaluadas (Spec #1-7).
- **Veredicto:** ✅ **APPROVE (delta r2)** — vanta-review (contexto fresco, P2-01). 5/5 hallazgos ronda 1 resueltos (F5 permalink Neo4j re-verificado fetch 2026-10-06; F1 `docs/index.md` fuera del pathspec + `llms.txt` verificado limpio de STRAT-04); contrato 1:1; gates docs 0 re-verificados. Nits residuales (L155 "1M+"→"6.8M+"; L51 Letta→`02:45,113,114`) corregidos en este mismo sync. Commit autorizado con pathspec `docs/dev/research/strat-06-licencias-acv-oss.md` + `docs/dev/tasks/STRAT-06.md` + `llms.txt` (sin `docs/index.md`, deuda transitoria declarada).

## Notas

### Investigación profunda — hallazgos clave (32 fuentes fetch-verificadas 2026-10-06)

**Licencias del grupo local-first AI (5 nombrados):**

| Producto | Licencia | Estado | ⭐ | Modelo |
|---|---|---|---|---|
| Khoj | **AGPL-3.0** | Activo (push 2026-08-02); **Khoj Cloud sunset 2026-04-15** | 37.6k | Self-host only; pivote a Open Paper + Pipali |
| Jan (Menlo Research) | **Apache-2.0** (LICENSE, © 2025 Menlo Research) | Activo | 44.8k | App desktop free; 6.8M+ downloads; sin pricing público |
| Reor | **AGPL-3.0** | **Archivado** (último push 2025-05-13) | 8.5k | Proyecto descontinuado |
| Open WebUI | **"Open WebUI License"** (BSD-3 + branding clause v0.6.6, 2025-04-19; no-OSI) | Activo | 154k | Free con marca; **enterprise license** para white-label (>50 usuarios); CLA; sponsors |
| Letta | **Apache-2.0** | Activo | 25.0k | Free (3 agentes) / **$20/mes Pro** / usage-based / Teams |

**OSS DBs relevantes (licencia motor):** ClickHouse Apache-2.0 · Qdrant Apache-2.0 · Turso MIT · Neo4j GPL-3.0 (+comercial) · Weaviate **split: BSD-3 + `wl/` enterprise license-key** · MongoDB SSPL · Elastic AGPL-3.0+ELv2+SSPL · Redis AGPL-3.0 (tri) · Supabase Apache-2.0 (fechas por fila: 2026-10-06 fresh o 2026-08-25 vía precedente).

**ACV/ARR verificado (fuente por número):**

| Empresa | Dato | Fecha | Fuente |
|---|---|---|---|
| ClickHouse | **ARR >$350M** (+40% desde may-2026; $160M fin-2025) | ago-2026 | dealroom.co + sacra.com [reportado/estimado] |
| Supabase | **$70M ARR** sep-2025 → ~$170M may-2026 | 2025-2026 | sacra.com + devgraphiq [estimado] |
| MongoDB | **2,999 clientes ≥$100k ARR** (de 70,600); **402 ≥$1M**; NER 122% | jul-2026 | SEC 10-Q / earnings [dato] |
| Elastic | **1,800+ clientes ≥$100k ACV**; NER ~112% | ago-2026 | earnings Q1 FY27 [dato] |
| Neo4j | **>$200M ARR** (oficial) | nov-2024 | neo4j.com press release [dato] |
| Redis | **>$300M ARR** (oficial); 12,000+ pagos; **50+ >$1M** | ene-2026 | GlobeNewswire/Yahoo + sacra [dato] |
| Turso | $15M+ raised (Series A) | — | billiondollarpitchdecks [confianza media] |

**Bands (modelado — derivado de los datos de arriba):** self-serve $0-1k · SMB $1k-10k · mid-market $10k-100k · enterprise $100k+ (definición de cohorte MongoDB/Elastic) · top $1M+ (MongoDB 402; Redis 50+/12k = 0.4%).

**Implicaciones open-core/OEM (borrador — detalle en el research doc):** (1) el segmento local-first usa AGPL (Khoj/Reor) o Apache (Jan/Letta); VantaDB Apache-2.0+CLA está alineado con la mitad permissiva; (2) OpenWebUI = precedente de "proteger marca, no código" (branding clause + enterprise license) compatible con Apache-2.0; (3) Khoj Cloud sunset valida la fase 1 sin nube y advierte del coste de sync/integraciones cloud-first; (4) los OSS DBs $100M-350M+ ARR monetizan cohortes enterprise ≥$100k → para VantaDB el salto de ACV requiere fase 2 (cloud) o SKU OEM/dual-license (Neo4j $200M con GPL+comercial = ancla del modelo dual).

### Disposición del review P2-01 (ronda 1 → fixes aplicados → delta)

- **F1 (High — higiene de commit):** `docs/index.md` regenerado incluye la fila de `STRAT-04.md` (WIP ajeno untracked, otra tarea en vuelo) → **`docs/index.md` se EXCLUYE del pathspec** del commit (se commitea `llms.txt`, limpio de STRAT-04; verificado por el reviewer). La regeneración conjunta queda para el cierre de STRAT-04 (precedente DESKTOP-44). Deuda transitoria declarada.
- **F2 (Medium — número sin fuente):** "1M+ downloads" de Jan → corregido a **6.8M+ downloads (jan.ai, fetch 2026-10-06)** + fila de fuente §4 #4.
- **F3 (Medium — claim de verificación falso):** "Khoj/Jan/Reor/OpenWebUI = 0 hits en docs/" corregido con la evidencia precisa: existe research previo de **producto** (Khoj en `03-ai-memory-graphs`; Open WebUI en `02-desktop-db-tools`), no de licencias/ACV — la frontera del run se mantiene.
- **F4 (Low):** fila Redis §2.1 → `dato (ARR) + reportado (conteos)`.
- **F5 (Low):** §4 con URLs completas (permalinks) para #17-#29; **sub-claim rechazado con evidencia**: el permalink de Neo4j (`neo4j.com/press-releases/neo4j-revenue-milestone-2024/`) **resuelve** (fetch 2026-10-06 → press release completo nov-2024); #23 anotado como corroboración de serie (2,900 Q1 FY27) y §2.1 MongoDB cita solo el 10-Q (2,999).

### Fuentes fetch-verificadas (2026-10-06 — TSYS-13)

GitHub API (licencias+estado): `khoj-ai/khoj` · `janhq/jan` (+LICENSE raw) · `reorproject/reor` · `open-webui/open-webui` (+LICENSE raw) · `letta-ai/letta` · `ClickHouse/ClickHouse` · `qdrant/qdrant` · `weaviate/weaviate` (+LICENSE raw) · `tursodatabase/turso` · `neo4j/neo4j`.
Páginas oficiales: `app.khoj.dev` (sunset notice) · `docs.openwebui.com/license` + `/enterprise` · `docs.letta.com/pricing` · `khoj.dev` + `blog.khoj.dev`.
ACV: `dealroom.co` + `sacra.com` (ClickHouse) · `sacra.com` + `devgraphiq.com` + `techstartups.com` (Supabase) · `stocktitan.net` + `fool.com` + `finance.yahoo.com` (MongoDB) · `stockstofind.com` + `stocktitan.net` + `panabee.com` (Elastic) · `neo4j.com/press-releases` · `finance.yahoo.com` + `sacra.com` (Redis) · `billiondollarpitchdecks.com` (Turso).
Precedentes internos (fechados 2026-08-25): `validacion/01`, `02`, `03`, `08`, `00-SINTESIS-EJECUTIVA2.md`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas en DISCOVERY (licencias fetch-verificadas; ACV con fuente; separación dato/modelado) |
| Pendientes de ejecución (downhill) | 0 (steps 1-3; 3/3 ✅) |
| % completado | 100% |
