# FND-11: No mergear código IA sin poder explicarlo — Regla 10 (AI Guardian)

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md (wave P20b, Backlog.md:499)
- **Fuente:** Backlog.md:499 (P20b)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴
- **Tipo:** Docs (`.opencode/AGENTS.md`)
- **Turns estimados:** 5
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3

> **NUMERACIÓN DE REGLA — coordinación anti-colisión:** FND-10 (otra wave) reclama explícitamente **"Regla 9"** (Backlog.md:498: "Regla 9 — No optimizar sin medir"). Esta tarea usa **Regla 10** para no colisionar. La sección actual termina en Regla 8; Regla 9 queda reservada para FND-10.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/AGENTS.md` (sección "VantaDB Development Protocol & AI Guardian Rules", Reglas 1-8 existentes) |
| Callees | ninguno (docs pura, sin imports) |
| Implicaciones | ningún contrato de código; agrega gate de proceso para PRs con código IA |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (554L — sección AI Guardian Rules con Reglas 1-8 en L387-553; sección "Ritual de Inicio" L192-219; "Conventional Commits" L496-518 de Regla 7)
- **Archivos referenciados hacia dentro:** ninguno (AGENTS.md no importa archivos)
- **Archivos que referencian a los editados (referencias entrantes):** grep `.opencode/AGENTS.md` → 99 matches: `AGENTS.md` raíz (L3/5/29), `.opencode/agents/vanta-research.md:65`, `.opencode/references/*.md` (punteros "Movido desde"), `.opencode/rules/README.md`, docs/plans/*, docs/progreso/*. Ninguno depende del texto exacto de las reglas — cambios aditivos seguros.
- **Veredicto impacto:** bajo — adición de regla + referencia; no modifica texto existente.

## Contrato
"`grep .opencode/AGENTS.md` contiene 'Regla 10' con la regla AI Guardian (incapacidad de explicar línea por línea = señal de qué estudiar; 'el desarrollo dicta el syllabus') Y una referencia a esa regla en el workflow de PR / sección Conventional Commits (Regla 7)"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no tocar `docs/Backlog.md`, `AUD-024.md`, `verify-log.jsonl`, `_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`, `.opencode/agents/*`, `references/task-system.md`; NO git add/commit (lead commitea); NO usar `campaign_update_task_state`; no re-numerar reglas existentes; dejar Regla 9 libre para FND-10.
- **Comandos de verificación:** `grep -n "Regla 10" .opencode/AGENTS.md` + `grep -n "línea por línea" .opencode/AGENTS.md` + `grep -n "dicta el syllabus" .opencode/AGENTS.md`
- **Deuda pendiente:** ninguna

## Deuda técnica (Regla 6 — MUST)
Sin deuda — cambio docs aditivo.

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato grep pasa (Regla 10 + ref PR).
- **Commit:** lo hace el lead por batch (instrucción del orquestador) — nivel commit delegado.
- **Release:** no aplica (docs, sin release).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [ ] **SECURITY** — no aplica: edición de docs de proceso, sin trust boundaries ni dependencias.
- [ ] **PERFORMANCE** — no aplica: sin código.

## Steps

### Step 1: Agregar Regla 10 al final de la sección AI Guardian Rules
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** después de la última línea de Regla 8 ("El mismo contexto que implementó no puede auto-auditarse (P2-01)."), agregar sección "### Regla 10: No Mergear Código IA sin Poder Explicarlo (AI Guardian)" con tabla estilo Regla 1/2 (Si el autor no puede... / Debes responder...): incapacidad de explicar decisión no trivial línea por línea = señal de qué estudiar esa semana; el desarrollo dicta el syllabus.
- **Verify:** `grep -n "Regla 10" .opencode/AGENTS.md` → match en sección AI Guardian; `grep -n "dicta el syllabus" .opencode/AGENTS.md` → match
- **Estado:** ⬜ PENDING

### Step 2: Referenciar Regla 10 en workflow de PR (sección Conventional Commits, Regla 7)
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** agregar línea "**Gate de explicabilidad:** antes de mergear cualquier PR con código generado por IA, el autor debe poder explicar cada decisión no trivial línea por línea — ver Regla 10 (AI Guardian)." después del bloque "Reglas estrictas" de "#### Conventional Commits (obligatorio para release-plz)" (no existe sección Workflow/PR dedicada; Conventional Commits/PR es la sección canónica — verificado).
- **Verify:** `grep -n "Gate de explicabilidad" .opencode/AGENTS.md` → match; la línea referencia "Regla 10"
- **Estado:** ⬜ PENDING

### Step 3: Verificación mecánica del contrato
- **Archivos:** ninguno
- **Acción:** correr los 3 greps del contrato.
- **Verify:** contrato pasa (Regla 10 + línea por línea + dicta el syllabus + Gate de explicabilidad)
- **Estado:** ⬜ PENDING

## Review (GATE — agente distinto, P2-01)
- **Revisor:** no-corrido — wave orquestada por el usuario con instrucciones explícitas; verificación mecánica (grep) + contrato definido por el orquestador. Justificado en Notas.
- **Veredicto:** pendiente de orquestador

## Notas
- Decisión de numeración: **Regla 10** (Regla 9 reservada para FND-10, Backlog.md:498).
- Referencia PR: no existe sección "Workflow/PR" en AGENTS.md (verificado en lectura completa L1-554); la sección Conventional Commits de Regla 7 es el punto canónico.
- Estilo de la regla: tabla "Si el autor no puede... / Debes responder..." consistente con Reglas 1/2.