# AGT-04: Limpieza .opencode/opencode-loop/ — corrupt/tmp + rotación

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 12)
- **Fuente:** docs/Backlog.md (colateral batch)
- **Esfuerzo:** 🟢 30m
- **Prioridad:** 🟢
- **Tipo:** Maintenance / infra task-system
- **Turns estimados:** 3
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ❌ FAILED (WIP huérfano — plan `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` no existe (batch cerrado/archivado); cerrado 2026-08-25 por MOD-15 executor para desbloquear one-task-at-a-time. Reversible: re-abrir si el batch revive.)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 2 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno — `.opencode/opencode-loop/` es runtime del plugin opencode-loop (gitignored, 0 tracked); ningún código del repo lo referencia |
| Callees | El plugin global `~/.config/opencode/plugins/opencode-loop.ts` (genera los archivos: `writeState` → `.tmp`, `readState` → `.corrupt-*`); script `dev-tools/clean-opencode-loop.ps1` (nuevo, commiteable) |
| Implicaciones | Borrado de runtime basura + GC automático en el plugin → no se re-acumulan; sesiones vivas `ses_*.json` NO se tocan; `loop.log` y `goals/` intactos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** plugin global `C:\Users\Eros\.config\opencode\plugins\opencode-loop.ts` (1995 líneas completas — mecanismo: `writeState` L383-395 escribe `target.json.<pid>.<ts>.tmp` + rename; si el proceso muere entre write y rename queda el `.tmp`; `readState` L367-381 copia el archivo corrupto a `target.json.corrupt-<ts>`); `.gitignore:189` (`.opencode/opencode-loop/` ignorado); `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` Task 12; conteo completo del dir (1357 archivos: 1260 `ses_*.json` vivos + 88 corrupt + 8 tmp + loop.log)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `.opencode/opencode-loop/` no es importado por nada del repo (gitignored runtime dir); el plugin lo crea vía `STATE_DIR = ".opencode/opencode-loop"`
- **Archivos que referencian a los editados (referencias entrantes):** grep `opencode-loop` → solo `.opencode/skills/campaign-executor/SKILL.md` Apéndice B (documenta el plugin) y `review-deep/loop-prompt.md` (lo usa como loop externo). Ninguno depende del contenido del dir
- **Veredicto impacto:** BAJO — borrar 96 archivos runtime huérfanos (32 bytes cada uno, generados por crashes del plugin); editar el plugin global agrega GC idempotente sin cambiar comportamiento de sesiones; crear script nuevo en `dev-tools/` no rompe nada

## Contrato
"corrupt/tmp eliminados; rotación agregada al loop server" — verify:
1. `Get-ChildItem .opencode/opencode-loop -File | Where-Object { $_.Name -match '\.corrupt-[\d]+$|\.tmp$' } | Measure-Object` → **0 residuales** (antes: 96)
2. Conteo de sesiones vivas intacto: `ses_*.json` = 1260 (no decrementado)
3. Plugin `opencode-loop.ts` contiene GC (función de cleanup de `*.corrupt-*`/`*.tmp` en init)
4. Script `dev-tools/clean-opencode-loop.ps1` existe y es idempotente (correr 2x → 0 errores)

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** NO borrar `ses_*.json` vivos (1260) ni `loop.log` ni `goals/`; el GC del plugin SOLO matchea `*.corrupt-*` y `*.tmp`; no cambiar semántica de writeState/readState (son el mecanismo atómico correcto — solo falta limpiar los residuos); no tocar otros plugins/config global
- **Comandos de verificación:** conteo corrupt/tmp antes vs después (0 residuales); `ses_*.json` = 1260 antes y después; `Get-Content ~/.config/opencode/plugins/opencode-loop.ts | Select-String "cleanupLoopStateDir"` → presente; `dev-tools/clean-opencode-loop.ps1` corre sin error
- **Deuda pendiente:** el plugin es global y no-commiteable — el parche vive en `~/.config/opencode/plugins/`; si `oc plugin install` lo sobreescribe, el GC se pierde (script `dev-tools/` queda como fallback manual documentado)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (borrado de runtime + script 25 líneas). Deuda pre-existente: el plugin no limpiaba residuos de crash — se paga con el GC (saldo 0 o negativo).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: 0 corrupt/tmp residuales; sesiones vivas intactas; GC en plugin; script en dev-tools/ |
| **Commit** | Lead verifica mecánico y commitea (sub-agente NO commitea — regla del plan): archivo `dev-tools/clean-opencode-loop.ps1` + task file |
| **Release** | N/A — tarea de mantenimiento sin release (justificado en Notas) |

## Herramientas necesarias
- PowerShell 7 (Get-ChildItem/Measure-Object)
- Editor de texto para el plugin global (fuera del repo)

## Investigation Notes
- Contéo exacto PASO 0: 1357 archivos totales = 1260 `ses_*.json` vivos + 88 `*.corrupt-*` + 8 `*.tmp` + 1 `loop.log` (verificado con regex `\.corrupt-[\d]+$` y `\.tmp$`).
- Patrón `.tmp` = `ses_XXX.json.<pid>.<ts>.tmp` (writeState L387). Patrón `.corrupt-*` = `ses_XXX.json.corrupt-<ts>` (readState L376). Ambos 32 bytes (JSON `{"version":4,"jobs":[]}` truncado o vacío).
- Los `.tmp` de un crash del plugin quedan porque `writeState` renombra temp→target; si el proceso muere antes del rename, el `finally` con `fs.rm` no corre.
- El dir está gitignored → borrar es solo filesystem; el artefacto commiteable es el script de cleanup.
- El plugin SÍ es editable (TS plano en config global) → se aplica GC al init (ponytail: 1 función + 1 llamada), NO solo documentar.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — mecanismo del plugin leído completo, patrones confirmados |
| Pendientes de ejecución (downhill) | 2 — (1) borrar 96 archivos; (2) GC plugin + script |
| % completado | 20% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: borrado de archivos runtime propios del task-system (no input de usuario, no trust boundary nuevo); el GC solo matchea patrones `.corrupt-*`/`.tmp` en un dir gitignored conocido
- [ ] **PERFORMANCE** — NO aplica: no toca hot paths del engine; GC en init es 1 readdir + rm por archivo

## Steps

### Step 1: Borrar 96 archivos corrupt/tmp de `.opencode/opencode-loop/`
- **Archivos:** `.opencode/opencode-loop/*.corrupt-*` (88) + `*.tmp` (8) — SOLO esos patrones
- **Acción:** `Remove-Item` con filtro `Name -match '\.corrupt-[\d]+$|\.tmp$'` (nunca `ses_*.json` sin sufijo)
- **Verify:** conteo residual = 0; `ses_*.json` = 1260 (subió a 1261 por sesión activa del loop); `loop.log` + `goals/` intactos
- **Estado:** ✅ COMPLETED — 96 borrados (88 corrupt + 8 tmp), residuales 0

### Step 2: GC automático en plugin opencode-loop + script de cleanup en repo
- **Archivos:** `C:\Users\Eros\.config\opencode\plugins\opencode-loop.ts` (init L1990) + `dev-tools/clean-opencode-loop.ps1` (nuevo)
- **Acción:** agregar `cleanupLoopStateDir(stateDir(directory))` al init del plugin (borra `*.corrupt-*` + `*.tmp` del stateDir); script PS1 idempotente con mismo filtro + uso documentado en header
- **Verify:** plugin contiene la función (L407) + llamada (L1990); script corre 2x sin error; conteo residual 0 tras correr script
- **Estado:** ✅ COMPLETED — GC en plugin (cleanupLoopStateDir L407, init L1990); script dry-run + apply x2 → 0 residuales, 1261 sesiones vivas intactas

## Dependencias
- Ninguna — tarea independiente (Wave 3)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review / lead (verifica mecánico al commitear — regla del plan: sub-agentes NO commitean)
- **Enfoque:** ¿el GC borra solo corrupt/tmp y nunca sesiones vivas? ¿el script es idempotente?
- **Cómo se probó:** conteo antes/después con regex exacta; script corrido 2x
- **Checklist anti-hábitos tóxicos:** N/A para revisión del lead — evidencia reproducible con 1 comando
- **Veredicto:** pendiente del lead

## Notas
- Regla 0 cumplida: plugin leído completo (1995L), refs grepeadas (SKILL.md Apéndice B + loop-prompt.md), veredicto BAJO.
- El parche al plugin es global (fuera del repo, no-commiteable) — se documenta en task file para que el lead sepa que el GC del server vive en `~/.config/opencode/plugins/`.
- Release N/A: mantenimiento de infra sin release.