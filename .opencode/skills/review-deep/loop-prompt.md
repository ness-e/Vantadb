# Review Deep — Loop Prompt

> **Modo:** Revisión profunda de UN módulo por invocación.
> El loop externo (opencode-loop) itera sobre los módulos en orden.
>
> **Anchor:** `.opencode/skills/review-deep/SKILL.md` — metodología completa (FASES 1-7, skills, checklists).
> **Este archivo:** solo configuración del loop, gates, y yield. No duplica fases.

## Entrada

- `MODULE` = nombre del módulo (ej: `vantadb-sdk`)
- `DEPTH` = `full` | `quick` (default: `full`)

## Contrato

```
CONTRATO: "Se revisó ${MODULE} completo. Hallazgos documentados en Backlog.md."
```

---

## FASE 0: Tool Lock-in (best-effort)

Ideal: limitar tools visibles según fase para reducir ruido (de lo contrario hay ~30+ tools). En la práctica esto depende de las capabilities del runtime del agente. Aplicar como guía, no como restricción infranqueable.

| Fase | Tools ideales |
|------|---------------|
| F1-F3 (análisis) | codegraph_explore, Read, Grep, Glob, bash (rust-analyzer, cargo-mcp) |
| F4-F5 (research) | metasearchmcp_search_web, argus_extract_content, Read |
| F6 (triage) | Edit, Write, Read (Backlog.md) |
| F7 (reporte) | Write, Read |

## QUALITY GATES

### Gate 1 (entre F3→F4): Integridad de hallazgos
- Cada hallazgo tiene archivo:línea exacto
- Cada hallazgo tiene tipo + severidad
- Si no → volver a F3

### Gate 2 (entre F5→F6): Research completa
- Cada hallazgo Medium+ tiene ≥1 URL de referencia
- URLs verificadas (no placeholder)
- Si no → volver a F4

---

## FASE 1-7

Ejecutar según SKILL.md. Cargar skills de FASE 0 del SKILL.md según tipo de módulo.

---

## FASE 6b: Scorecard Registration

Tras el triage, registrar scorecard:

```powershell
# Crear directorio de iteraciones
New-Item -ItemType Directory -Force -Path ".opencode/skills/review-deep/tmp" | Out-Null

# Generar scorecard como JSON
$scorecard = @{
    module    = $env:MODULE
    duration  = $env:DURATION
    findings  = @{
        critical = [int]$env:CRIT
        high     = [int]$env:HIGH
        medium   = [int]$env:MED
        low      = [int]$env:LOW
        info     = [int]$env:INFO
    }
    fixed_now     = [int]$env:FIXED
    to_backlog    = [int]$env:BACKLOG
    discarded     = [int]$env:DISCARDED
    research_urls = [int]$env:URLS
}
$scorecard | ConvertTo-Json | Out-File -Encoding utf8 ".opencode/skills/review-deep/tmp/$($env:MODULE)-$(Get-Date -Format 'yyyyMMddHHmmss').json"
```

Si hay scorecard previo, comparar deltas.

---

## FASE 7: Reporte + Yield

### Yield

```
opencode_loop_goal_progress summary:"${MODULE} revisado: N hallazgos, M al backlog"
  next:"Siguiente módulo: ${NEXT_MODULE}"
```

Si era el último módulo:

```
opencode_loop_goal_complete summary:"Review Deep completo: N módulos, M hallazgos totales"
  evidence:"Backlog.md actualizado, reporte generado"
```
