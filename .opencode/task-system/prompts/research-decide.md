# INV-DECIDE — Sala de decisión global: convertir los resultados de investigación en acciones

> **DISPARADOR:** `/research synthesis` — ejecutar SOLO después de
> `docs/reviews/research-bindings-synthesis-<fecha>.md` y los reportes
> `research-<modulo>-<fecha>.md` en `docs/reviews/`.
> **Principios de formato** heredados de `prompts/question-gates.md` (fuente única):
> opciones concretas + default recomendado (`(Recomendado)` primero), nunca pregunta
> abierta sin opciones, registrar cada decisión en trace/recitation.

## Objetivo
Convertir los hallazgos de las investigaciones en DECISIONES del usuario — qué
aplicar, mejorar, agregar, optimizar o descartar — y materializar cada decisión
en tareas ejecutables del sistema (Backlog/plan).

## Fase 1 — Consolidación (sin preguntar nada todavía)

Leé y consolidá EN UNA SOLA TABLA MAESTRA todos los hallazgos de:
1. Los reportes `docs/reviews/research-<modulo>-<fecha>.md` (apéndices H-NN)
2. La síntesis `docs/reviews/research-bindings-synthesis-<fecha>.md`
3. Las filas FIND-\* ya derivadas al Backlog por esas investigaciones

Clasificá cada hallazgo en UNA categoría de acción:

| Categoría | Significado | Ejemplo |
|-----------|-------------|---------|
| **APLICAR** | Quick win <1 día, listo para ejecutar ya | "agregar engines field a package.json" |
| **MEJORAR** | Existe pero deficiente (score ≤6 en alguna dimensión) | "docs sin ejemplos por runtime" |
| **AGREGAR** | Feature/capacidad que falta (gap P0/P1) | "snapshot_restore no existe en ningún binding" |
| **OPTIMIZAR** | Funciona pero con costo medible (perf/bundle/memoria) | "WASM bundle 2.3MB sin lazy-load" |
| **ESTRATEGIA** | Decisión de dirección, no tarea puntual | "¿competir en browser-only o ceder a Orama?" |
| **DESCARTAR/WONTFIX** | No vale el costo — proponer archivar | "soporte Deno <1.0" |

Por cada ítem calculá: **impacto** (alto/medio/bajo) × **esfuerzo** (🟢🟡🔴) ×
**dimensión ISO afectada**, usando lo que dicen los reportes (no inventes).

Si hay >25 ítems, pre-seleccioná: TODOS los P0/P1, top-10 por ratio impacto/esfuerzo,
y los ESTRATEGIA. El resto va a un anexo "sin pregunta — van directo a Backlog como
P2/P3 salvo objeción".

## Fase 2 — Rondas de `question` (batched, máx 4 rondas)

Usá la tool `question` con MÚLTIPLES preguntas por llamada. Cada pregunta:
- Agrupada por tema, NO una por hallazgo (agregá alternativas mutuamente excluyentes
  cuando compitan: "Opción A vs B vs ambas").
- Con `(Recomendado)` primero, derivado de los scores de la síntesis.
- Permitiendo `multiple: true` cuando las opciones no sean excluyentes.

### Ronda 1 — Veredicto y estrategia (la más importante)
Presentá primero EL VEREDICTO consolidado de la síntesis (scores por módulo,
ranking de competidores, posición relativa de VantaDB). Preguntá:
- Q1: ¿Cuál es la apuesta estratégica prioritaria?
- Q2: ¿Qué dimensión global es inaceptable hoy? (multiple)
- Q3: ¿Hay hallazgos que contradigan la hoja de ruta actual? → re-priorizar Backlog?

### Ronda 2 — Quick wins APLICAR ya
- Q4: Quick wins (<1 día, alto impacto): ¿cuáles aplicamos ahora? (multiple)
- Q5: ¿Vía plan batch (`/pipeline plan`) o tareas sueltas (`/pipeline task`)?

### Ronda 3 — AGREGAR features faltantes
- Q6: Gaps P0 (por módulo): ¿cuáles entran al Backlog como fase nueva?
- Q7: Gaps P1: ¿entrar, diferir, o WONTFIX?
- Q8: Gaps de PARIDAD entre módulos: ¿paridad completa como política o caso por caso?

### Ronda 4 — MEJORAR/OPTIMIZAR + descartes
- Q9: Mejorables: ¿cuáles suben de prioridad? (score actual vs potencial)
- Q10: Optimizables: ¿cuáles exigen benchmark before/after antes de decidir
  (Regla 9) y cuáles se aprueban directo?
- Q11: Descartes propuestos: ¿confirmás WONTFIX/archivo?

**Regla de corte:** si después de la Ronda 2 el usuario dice "todo lo recomendado",
saltá las rondas restantes aplicando los defaults recomendados y listalos como
decisiones tomadas por-default-explícito (registradas igual).

## Fase 3 — Materializar decisiones

Por CADA decisión del usuario:
1. **Tarea ejecutable** en `docs/Backlog.md`: ID canónico según módulo destino
   (ver tabla de prefijos en `.opencode/references/research-modules.md`; hallazgos
   genéricos = FIND-\*), contrato verificable ("hecho = comando que lo prueba"),
   esfuerzo, prioridad resultante, archivos clave.
2. **WONTFIX/DESCARTES** → `docs/avance/decisiones/wontfix.md` con el motivo.
3. **Estrategia** → `campaign_memory_write(file="decisions", ...)`; trade-off
   arquitectónico → sugerir ADR (lo redacta el autor humano — Regla 5).
4. **Quick wins aprobados** → plan file nuevo listo para `/pipeline run`, con
   waves (independientes en Wave 0).
5. Emití `campaign_emit_event(event="inv-decide.completed", data={decisiones: N,
   quick_wins: [...], wontfix: [...]})`.

## Output final

```markdown
## Decisiones registradas
| # | Decisión | Categoría | Destino | ID tarea/plan |
|---|----------|-----------|---------|---------------|

## Por-default-explícito (aplicados sin pregunta)
...

## Rechazados/WONTFIX
... (con motivo)

## Próximo paso recomendado
/pipeline plan docs/plans/<nuevo-plan>.md   (si hubo quick wins)
```

RESULTADO final con SKILLS_CARGADAS y GATES_EVALUADOS.
