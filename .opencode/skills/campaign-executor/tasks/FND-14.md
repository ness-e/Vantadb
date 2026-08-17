# FND-14: Ritual de inicio — validación de feature stack

## Metadata
- **Plan file:** docs/plans/2026-08-16-wave-p20-tsys.md (wave P20b, Backlog.md:502)
- **Fuente:** Backlog.md:502 (P20b)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Docs (`.opencode/AGENTS.md`)
- **Estado:** ✅ COMPLETO (verificado en AGENTS.md §Ritual de Inicio, wave p20-tsys)
- **Turns estimados:** 3
- **Creado:** 2026-08-16
- **last-synced:** 2026-08-16
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 3

> **NUMERACIÓN:** no agrega regla — agrega paso al Ritual de Inicio de Sesión (L192-219). Sin colisión.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/AGENTS.md` — sección "## Ritual de Inicio de Sesión (MUST DO)" (L192-219, 4 pasos + bloque "Al finalizar") |
| Callees | ninguno (docs pura) |
| Implicaciones | el ritual de sesión pasa a verificar que el feature set mínimo compila; sin cambio de código |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (554L — Ritual de Inicio completo en L192-219: pasos 1-4 + bloque finalizar)
- **Archivos referenciados hacia dentro:** ninguno
- **Archivos que referencian a los editados (referencias entrantes):** misma lista que FND-11 (99 matches, ninguno depende del texto exacto del ritual)
- **Veredicto impacto:** bajo — adición de un paso (5) al ritual; no modifica los pasos existentes.

## Contrato
"`grep .opencode/AGENTS.md` contiene 'cargo check --no-default-features --features fjall' DENTRO de la sección 'Ritual de Inicio de Sesión'"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** no tocar `docs/Backlog.md`, `AUD-024.md`, `verify-log.jsonl`, `_vanta-cli.ps1`, `docs/plans/2026-08-16-wave-p20-tsys.md`, `.opencode/agents/*`, `references/task-system.md`; NO git add/commit; NO usar `campaign_update_task_state`; no modificar los 4 pasos existentes del ritual.
- **Comandos de verificación:** `grep -n "cargo check --no-default-features --features fjall" .opencode/AGENTS.md` → match dentro de L192-219
- **Deuda pendiente:** ninguna

## Deuda técnica (Regla 6 — MUST)
Sin deuda — cambio docs aditivo.

## Definition of Done (contrato multi-nivel — P2-08)
- **Task:** contrato grep pasa (paso en el ritual).
- **Commit:** lo hace el lead por batch (instrucción del orquestador).
- **Release:** no aplica (docs).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [ ] **SECURITY** — no aplica: docs de proceso.
- [ ] **PERFORMANCE** — no aplica.

## Steps

### Step 1: Agregar paso 5 de validación de feature stack al Ritual de Inicio
- **Archivos:** `.opencode/AGENTS.md`
- **Acción:** después del paso 4 ("Verificar entorno rápido") y antes del bloque "Al **finalizar** la sesión:", agregar paso 5 "Validar feature stack (si la sesión toca Rust)" con `cargo check --no-default-features --features fjall` (feature set mínimo compila). Comando literal dado por el orquestador (Backlog.md:502).
- **Verify:** `grep -n "cargo check --no-default-features --features fjall" .opencode/AGENTS.md` → match entre L192-219
- **Estado:** ⬜ PENDING

### Step 2: Verificación mecánica del contrato
- **Archivos:** ninguno
- **Acción:** correr grep del contrato + confirmar que el match está en la sección Ritual (L192-219).
- **Verify:** contrato pasa
- **Estado:** ⬜ PENDING

## Review (GATE — agente distinto, P2-01)
- **Revisor:** no-corrido — wave orquestada por el usuario con instrucciones explícitas; verificación mecánica (grep) + contrato definido por el orquestador. Justificado en Notas.
- **Veredicto:** pendiente de orquestador

## Notas
- El Ritual de Inicio existe (L192-219) con 4 pasos; el paso nuevo es el 5.
- Comando literal `cargo check --no-default-features --features fjall` — el orquestador lo define (consistente con FND-03 "feature set mínimo compila"). No se ejecuta cargo (contrato = grep).