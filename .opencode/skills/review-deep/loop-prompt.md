# Review Deep — Loop Prompt

> **Modo:** Revisión profunda de UN módulo por invocación.
> El loop externo (opencode-loop) itera sobre los módulos en orden.
>
> **Anchor:** `.opencode/skills/review-deep/SKILL.md` — metodología completa.
> **Referencia módulos:** SKILL.md → Orden de Iteración de Módulos (Waves 0-6).

## Entrada

Variables de entorno / argumento del loop:
- `MODULE` = nombre del módulo (ej: `vantadb-sdk`, `vantadb-vector`, `vantadb-python`)
- `DEPTH` = `full` | `quick` (default: `full`)
  - `full`: Fases 1-7 completas
  - `quick`: Fase 1 + 2 + 3a + 6 (sin research ni competitor)

## Contrato

```
CONTRATO: "Se revisó ${MODULE} completo. Hallazgos documentados en Backlog.md.
           Skills cargadas: [lista]. Reporte generado."
```

---

## FASE 0: Tool Lock-in

Antes de empezar, fijar las tools disponibles según la fase actual:

| Fase | Tools visibles | Tools ocultas |
|------|---------------|---------------|
| F1-F3 (análisis) | codegraph_explore, Read, Grep, Glob, bash (solo rust-analyzer, cargo-mcp) | Edit, Write, search_web, browser |
| F4-F5 (research) | search_web, browser, extract_content, Read | Edit, Write, codegraph_explore |
| F6 (triage) | Edit, Write, Read (Backlog.md) | codegraph_explore, search_web, browser |
| F7 (reporte) | Write, Read | codegraph_explore, search_web, Edit |

Esto evita el "context suicide" donde el modelo gasta 67.6% de tokens en tool
outputs irrelevantes. ~5 tools visibles en vez de 30+.

## QUALITY GATES

Dos gates obligatorios que verifican completitud antes de pasar a la siguiente
fase. Si un gate falla, VOLVER a la fase anterior — no seguir.

### Gate 1 (entre F3→F4): Integridad de hallazgos
- Cada hallazgo tiene archivo:línea exacto
- Cada hallazgo tiene tipo (LOGIC/PATTERN/ARCH/CODE/ERROR/...) y severidad
- Si no → volver a F3, completar los que falten

### Gate 2 (entre F5→F6): Research completa
- Cada hallazgo Medium+ tiene al menos 1 URL de referencia
- URLs verificadas (no placeholder ni "TODO")
- Si no → volver a F4, investigar los pendientes

---

## FASE 0: Skills

```bash
skill progreso
skill ponytail-audit
skill code-review-and-quality
skill doubt-driven-development
skill code-simplification
skill security-and-hardening
skill performance-optimization
skill api-and-interface-design
skill source-driven-development
```

Si DEPTH=full, cargar también:
```
skill writing-plans
skill brainstorming
```

---

## FASE 1: Structural Mapping

```bash
codegraph_explore "vantadb::${MODULE} symbols classes functions structs enums"
codegraph_explore "vantadb::${MODULE} callers dependants"
codegraph_explore "vantadb::${MODULE} callees dependencies imports"
```

Registrar:
- API Surface (pub fn, pub struct, pub enum)
- CALLERS (módulos que dependen de este)
- CALLEES (de qué depende este módulo)
- File sizes (archivos grandes >500L)
- Tests existentes que cubren el módulo

---

## FASE 2: Static Analysis

Ejecutar los comandos que correspondan según el tipo de módulo.

### Rust (vantadb-*, adaptadores, server, mcp, wasm)

```bash
cargo check -p ${CRATE} 2>&1
cargo clippy -p ${CRATE} --all-targets --all-features -- -D warnings 2>&1
cargo fmt --check 2>&1
```

Registrar: errores de compilación, warnings de clippy, issues de formato.

### Python SDK

```bash
target/audit-venv/Scripts/python -m mypy vantadb-python/ 2>&1 | tail -20
target/audit-venv/Scripts/python -m ruff check vantadb-python/ 2>&1 | tail -20
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/ -v 2>&1 | tail -30
```

### TypeScript SDK

```bash
cd vantadb-ts && npx tsc --noEmit 2>&1
cd vantadb-ts && npx eslint . --ext .ts 2>&1
cd vantadb-ts && npx vitest run 2>&1
```

---

## FASE 3: Deep Code Review

### 3a. Pattern Scanning

```bash
# Contar patrones en el módulo
codegraph_explore "vantadb::${MODULE} expect unwrap unsafe"
codegraph_explore "vantadb::${MODULE} todo unimplemented unreachable"
codegraph_explore "vantadb::${MODULE} allow transmute clone lock block"
```

Para cada patrón, verificar:
- `expect`/`unwrap`: ¿mensaje descriptivo? ¿precondición documentada?
- `unsafe`: ¿SAFETY comment presente? ¿invariant documentado?
- `#[allow]`: ¿justificación? ¿se puede eliminar?
- `clone()`: ¿necesario vs Copy type? ¿hot path?
- `.lock()`: ¿orden consistente? ¿deadlock possible?

### 3b-3g: Review Checklists

Aplicar las listas de verificación de FASE 3 del SKILL.md según
corresponda al tipo de módulo.

Para cada hallazgo encontrado, asignar:
- Tipo (LOGIC/PATTERN/ARCH/CODE/ERROR/MISSING/FEATURE/ALGO/SECURITY/PERF)
- Severidad tentativa (Critical/High/Medium/Low/Info)
- Archivo:línea exacto

---

## FASE 4: Web Research (solo DEPTH=full)

Para cada hallazgo no trivial (Medium+), investigar:

```bash
MetaSearchMCP.search_web("<issue específico>")
Argus.extract_content("<url resultado>")
```

Criterios para investigar:
- Patrones de Rust que no se usan en el proyecto
- APIs/librerías externas
- Decisiones técnicas con múltiples enfoques
- Errores/deprecation de dependencias
- Mejoras de performance documentadas

Registrar: URL, solución recomendada, alternativas.

---

## FASE 5: Competitor Comparison (solo DEPTH=full)

Comparar el módulo contra competidores relevantes:

- **Vector/index:** Chroma, Pinecone, Qdrant, Milvus, LanceDB
- **Storage:** RocksDB, SQLite, Sled
- **SDK/bindings:** LangChain, LlamaIndex, Haystack

Para cada feature del módulo: ¿quién más lo tiene? ¿mejor/peor?

---

## FASE 6: Triage → Backlog.md

Gate de evaluación para CADA hallazgo:

```
Hallazgo: [issue]
Archivo: [path:line]
Severidad tentativa: [Critical/High/Medium/Low/Info]

Gate:
[ ] ¿Issue real? (sí/no)
[ ] ¿Severidad correcta?
[ ] ¿Impacto real en usuarios? (sí/no)
[ ] ¿Esfuerzo? (XS/S/M/L/XL)
[ ] ¿Workaround?
[ ] ¿Relacionado con otro hallazgo?

Resultado:
[ ] 🟢 FIX AHORA (esfuerzo XS, <30min, mismo archivo)
    → Arreglar inmediatamente, commit junto con la revisión
    → Anotar en reporte: "arreglado en ruta"
[ ] ✅ BACKLOG → agregar a Backlog.md con ID DRV-NNN
[ ] ❌ DESCARTAR → razón: [falso positivo / no aplica / duplicado]
```

### Formato para Backlog.md

```markdown
| `DRV-001` | **Módulo: unsafe sin SAFETY docs** — 3 bloques unsafe en src/foo.rs sin SAFETY comments que documenten invariantes | src/foo.rs:42,55,78 | 🟢 XS | 🟡 | ❌ |
```

El campo de estado debe ser:
- 🔴 si Critical
- 🟡 si High
- 🔵 si Medium
- ⚪ si Low
- ℹ️ si Info

---

## FASE 6b: Scorecard Registration (darwin-godel pattern)

Tras el triage y antes del reporte, registrar scorecard:

```bash
# Crear directorio de iteraciones si no existe
mkdir -p .opencode/skills/review-deep/tmp

# Generar scorecard como JSON (valores reales del módulo revisado)
cat > .opencode/skills/review-deep/tmp/${MODULE}-$(date +%s).json << 'SCORECARD'
{
  "module": "${MODULE}",
  "duration": "${DURATION}",
  "findings": {
    "critical": ${CRIT},
    "high": ${HIGH},
    "medium": ${MED},
    "low": ${LOW},
    "info": ${INFO}
  },
  "fixed_now": ${FIXED},
  "to_backlog": ${BACKLOG},
  "discarded": ${DISCARDED},
  "research_urls": ${URLS}
}
SCORECARD
```

Si hay scorecard previo para el mismo módulo, comparar deltas.

---

## FASE 7: Reporte + Yield

### Reporte del módulo

```markdown
### Módulo: ${MODULE}

| Métrica | Valor |
|---------|-------|
| API Surface | N pub items |
| Archivos | N |
| Líneas | N |
| Tests | N |
| Hallazgos totales | N |
| Fixeados ahora | N |
| Al backlog | N (DRV-NNN..DRV-MMM) |
| Descartados | N |
| Web research | N URLs |
| Competitor gaps | N |
```

### Yield

```bash
opencode_loop_goal_progress summary:"${MODULE} revisado: N hallazgos, M al backlog"
  next:"Siguiente módulo: ${NEXT_MODULE}"
```

Si era el último módulo:

```bash
opencode_loop_goal_complete summary:"Review Deep completo: N módulos, M hallazgos totales"
  evidence:"Backlog.md actualizado con DRV-NNN... reporte generado"
```
