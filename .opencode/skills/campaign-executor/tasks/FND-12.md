# FND-12: ADRs como forcing function — reforzar Regla 5 (autor humano, IA solo evidencia)

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md (wave P20b, Backlog.md:500)
- **Fuente:** Backlog.md:500 (P20b)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Docs (`.opencode/AGENTS.md`)
- **Turns estimados:** 5
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3

> **NUMERACIÓN:** no agrega regla nueva — refuerza la Regla 5 existente (L435-446). Sin colisión de numeración.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/AGENTS.md` — sección "### Regla 5: Memoria de Decisiones Arquitectónicas" (L435-446) |
| Callees | ninguno (docs pura) |
| Implicaciones | refuerza la política de ADRs; sin cambio de código; los ADRs futuros deberán ser articulados por el humano |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (554L — Regla 5 completa en L435-446 con tabla Acción/Formato + nota "Validación web")
- **Archivos referenciados hacia dentro:** `docs/architecture/adr/` (formato `NNN_titulo_breve.md`, plantilla `docs/_templates/adr.md`) — se mencionan, no se tocan
- **Archivos que referencian a los editados (referencias entrantes):** misma lista que FND-11 (99 matches, ninguno depende del texto exacto de Regla 5)
- **Veredicto impacto:** bajo — adición de párrafo de refuerzo + ejemplo dentro de Regla 5; no modifica texto existente.

## Contrato
"`grep .opencode/AGENTS.md` contiene en Regla 5 la política 'el ADR lo escribe el autor humano articulando el trade-off; la IA solo aporta evidencia' (forcing function) Y el ejemplo de formato con Contexto/Decisión/Consecuencias + quién articula"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no tocar `docs/Backlog.md`, `AUD-024.md`, `verify-log.jsonl`, `_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`, `.opencode/agents/*`, `references/task-system.md`; NO git add/commit; NO usar `campaign_update_task_state`; no re-numerar reglas; no modificar el texto existente de Regla 5 (solo adición).
- **Comandos de verificación:** `grep -n "autor humano" .opencode/AGENTS.md` + `grep -n "Contexto\|Decisión\|Consecuencias" .opencode/AGENTS.md` (en Regla 5)
- **Deuda pendiente:** ninguna

## Deuda técnica (Regla 6 — MUST)
Sin deuda — cambio docs aditivo.

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato grep pasa (refuerzo + ejemplo en Regla 5).
- **Commit:** lo hace el lead por batch (instrucción del orquestador).
- **Release:** no aplica (docs).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [ ] **SECURITY** — no aplica: docs de proceso.
- [ ] **PERFORMANCE** — no aplica.

## Steps

### Step 1: Reforzar Regla 5 con forcing function
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** dentro de Regla 5 (después de la tabla Acción/Formato o del párrafo "Esto previene..."), agregar párrafo: el ADR lo escribe el **autor humano** articulando el trade-off con sus propias palabras; la IA solo aporta evidencia (datos, comparativas, riesgos). Si la IA redacta el ADR por el autor, pierde su función: el ejercicio de articulación ES la decisión.
- **Verify:** `grep -n "autor humano" .opencode/AGENTS.md` → match dentro de Regla 5 (L435-446)
- **Estado:** ⬜ PENDING

### Step 2: Agregar ejemplo breve del formato (Contexto/Decisión/Consecuencias + quién articula)
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** agregar tabla mínima con 3 filas (Contexto / Decisión / Consecuencias), columna "Quién lo articula" = humano (IA solo aporta datos en Consecuencias).
- **Verify:** `grep -n "Contexto" .opencode/AGENTS.md` y `grep -n "Decisión" .opencode/AGENTS.md` y `grep -n "Consecuencias" .opencode/AGENTS.md` → 3 matches en Regla 5
- **Estado:** ⬜ PENDING

### Step 3: Verificación mecánica del contrato
- **Archivos:** ninguno
- **Acción:** correr greps del contrato.
- **Verify:** contrato pasa
- **Estado:** ⬜ PENDING

## Review (GATE — agente distinto, P2-01)
- **Revisor:** no-corrido — wave orquestada por el usuario con instrucciones explícitas; verificación mecánica (grep) + contrato definido por el orquestador. Justificado en Notas.
- **Veredicto:** pendiente de orquestador

## Notas
- El refuerzo se agrega DENTRO de Regla 5 (no nueva regla) — evita colisión con FND-10 (Regla 9) y FND-11 (Regla 10).
- Ejemplo de formato = tabla Contexto/Decisión/Consecuencias con columna de quién articula, consistente con la plantilla `docs/_templates/adr.md`.