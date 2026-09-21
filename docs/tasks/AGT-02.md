# AGT-02: Verificar stats CodeGraph en AGENTS.md

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 10)
- **Fuente:** docs/Backlog.md:871
- **Esfuerzo:** 🟢 30m
- **Prioridad:** 🟢
- **Tipo:** Docs
- **Turns estimados:** 4
- **Creado:** 2026-08-25
- **last-synced:** 2026-08-25
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno — `.opencode/AGENTS.md` es leído por agentes al inicio de sesión; ningún archivo depende del valor numérico exacto |
| Callees | Ninguno — solo actualiza un literal de texto |
| Implicaciones | Cambio cosmético de exactitud documental; no rompe contratos, API, ni tests. El índice CodeGraph NO se modifica |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/AGENTS.md` (contenido completo inyectado en contexto de sesión; sección § CodeGraph releída con Read líneas 62-75); `.codegraph/codegraph.db` (schema + conteos vía sqlite read-only); `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md`; `docs/Backlog.md:871`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `.opencode/AGENTS.md` no importa archivos — es markdown de instrucciones
- **Archivos que referencian a los editados (referencias entrantes):** grep de "7.3K|24.7K" → solo `.opencode/AGENTS.md:67` (§ CodeGraph, SCOPE) y `.opencode/AGENTS.md:368` (§ MCP Servers, FUERA DE SCOPE — se documenta como deuda). `docs/Backlog.md` y el plan file describen la tarea, no dependen del número. El resto de refs a `.opencode/AGENTS.md` (CONTRIBUTING.md, VANTADB-OPERATING-MANUAL.md, skills) no citan los números
- **Veredicto impacto:** BAJO — editar un literal en 1 línea de markdown; nada se rompe. El dato stale es cosmético (drift documental)

## Contrato
"Números de CodeGraph verificados/actualizados en AGENTS.md § CodeGraph" — verify:
1. `codegraph status` (CLI, index up to date) → nodes=20.496, edges=71.446 (separador europeo = 20,496 / 71,446)
2. Query directa sqlite read-only: `SELECT COUNT(*) FROM nodes` = 20496; `SELECT COUNT(*) FROM edges` = 71446 (fuente primaria)
3. Grep del número actualizado en `.opencode/AGENTS.md` coincide con la fuente

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** solo se edita la línea 67 (§ CodeGraph) de `.opencode/AGENTS.md`; NO se tocan otras secciones del archivo (en particular línea 368 § MCP Servers — queda como deuda para decisión del lead); NO se modifica el índice `.codegraph/codegraph.db`
- **Comandos de verificación:** `codegraph status` → nodes 20.496 / edges 71.446; `python cg_stats.py` (sqlite ro) → nodes 20496 / edges 71446; `grep -n "20.5K\|71.4K" .opencode/AGENTS.md` → línea 67
- **Deuda pendiente:** línea 368 de `.opencode/AGENTS.md` (§ MCP Servers Disponibles) conserva "7.3K símbolos" stale — fuera de scope de esta tarea; lead decide si aprobar micro-fix o dejar (los números del index cambian con cada indexación, considerar quitar en vez de re-verificar)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (edit de 1 literal, 0 código). Deuda pre-existente documentada: literal stale en línea 368.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable: números en AGENTS.md coinciden con `codegraph status` + sqlite directo |
| **Commit** | Lead verifica mecánico y commitea (sub-agente NO commitea — regla del plan) |
| **Release** | N/A — tarea docs sin release (justificado en Notas) |

## Herramientas necesarias
- codegraph CLI (`codegraph status`)
- Python 3.11 stdlib sqlite3 (read-only)

## Investigation Notes
- El intento previo falló por quoting de shell (backslash-quote en PowerShell rompió el one-liner de Python). Solución: script temp en `C:\Users\Eros\AppData\Local\Temp\opencode\cg_stats.py` con URI `file:...?mode=ro` — evitando el quoting inline.
- `codegraph status` usa separador europeo de miles (1.075 / 20.496 / 71.446) → 1,075 files / 20,496 nodes / 71,446 edges.
- DB 111MB con WAL activo (7.3MB) — la lectura vía CLI y vía sqlite directa coinciden (20,496 / 71,446), doble fuente.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — números verificados con 2 fuentes independientes |
| Pendientes de ejecución (downhill) | 0 — step completado y verificado |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica: edición de un literal en markdown de docs, sin trust boundaries, input de usuario, ni dependencias.
- [ ] **PERFORMANCE** — NO aplica: no toca hot paths, índices ni código.

## Steps

### Step 1: Editar `.opencode/AGENTS.md` § CodeGraph con números reales
- **Archivos:** `.opencode/AGENTS.md` (línea 67)
- **Acción:** reemplazar "(7.3K símbolos, 24.7K edges)" por "(20.5K símbolos, 71.4K edges — verificado 2026-08-25 vía `codegraph status`)"
- **Verify:** `rg -n "20\.5K|71\.4K|7\.3K|24\.7K" .opencode/AGENTS.md` → línea 67 actualizada, línea 368 (fuera de scope) intacta; `codegraph status` → Nodes 20.496 / Edges 71.446; `python cg_stats.py` (sqlite ro) → nodes 20496 / edges 71446
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna — tarea independiente (Wave 3)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review / lead (verifica mecánico al commitear — regla del plan: sub-agentes NO commitean)
- **Enfoque:** ¿los números actualizados provienen de la fuente real del index y no de inventiva?
- **Cómo se probó:** doble fuente (CLI `codegraph status` + query sqlite directa read-only) con resultados idénticos; comando documentado en Contrato
- **Checklist anti-hábitos tóxicos:** N/A para revisión del lead — evidencia reproducible con 2 comandos
- **Veredicto:** pendiente del lead

## Notas
- Scope estricto: solo § CodeGraph (instrucción explícita del usuario: "NO toques otras secciones del archivo"). La línea 368 (§ MCP Servers) tiene el mismo número stale → documentada en Deuda pendiente, NO editada.
- Formato K preservado del estilo existente (7.3K → 20.5K; 24.7K → 71.4K).
- Se agrega fecha + comando de verificación para trazabilidad anti-drift (era la info que faltaba cuando se escribieron 7.3K/24.7K).
- Release N/A: tarea docs sin release.