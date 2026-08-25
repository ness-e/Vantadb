---
description: "Investigación profunda por módulo con registro de configuración y decisiones HITL por hallazgo. Uso: /research <módulo> | /research synthesis | /research (listar)"
---

> **ENTRY POINT — Research Command**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/research`.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
>
> **Registro de módulos (fuente única de datos):** `.opencode/references/research-modules.md`
> **Consolidación 2026-08-25:** este comando NO reemplaza a ningún otro — opera la
> investigación INV-\* que antes no tenía entry point.

Cargá las skills `progreso`, `ponytail` (full), `source-driven-development`, `coordinated-web-search`.

Entrada: $ARGUMENTS

## Router

| Invocación | Acción |
|------------|--------|
| `/research <módulo>` | Flujo por-módulo: Fases V→R→D (abajo) |
| `/research synthesis` | Sala de decisión GLOBAL: cargá y ejecutá `prompts/research-decide.md` |
| `/research` (sin argumento) | Listar módulos del registro + estado de reportes existentes en `docs/reviews/` |

---

## Flujo por-módulo (`/research <módulo>`)

### Fase V — Validar módulo (Gate de registro)

Leé el registro `.opencode/references/research-modules.md`. El argumento
`<módulo>` puede venir con o sin backticks; normalizá a minúsculas.

1. **¿Está en la tabla del registro Y existe el directorio?**
   → ✅ Continuar a Fase R con los datos de su fila.
2. **Está en la tabla pero el directorio NO existe en el repo:**
   → Informarlo y preguntar vía `question`: "El módulo `<X>` está registrado pero
   no existe en disco. ¿Investigar igual (puede haberse movido), archivar la fila,
   o abortar?" — Registrar lo decidido.
3. **NO está en la tabla pero el directorio SÍ existe:**
   → Gate de registro (obligatorio antes de investigar). Preguntá vía `question`
   los campos faltantes, UNA ronda:
   - **Tipo** (binding/SDK/server/adapters/otro — con ejemplos de la tabla)
   - **Ecosistema** (npm/PyPI/crates.io/repo)
   - **Usuarios objetivo** (opción "descripción libre" permitida acá — es dato nuevo)
   - **Competidores mínimos** (sugerí 3-5 detectados por el dominio si podés)
   - **Nota específica** (deudas/checkpoints conocidos)
   Con las respuestas: agregá la fila al registro (Edit) y continuá a Fase R.
4. **Ni en la tabla ni existe el directorio:**
   → Comunicar que el módulo no existe: "❌ `<X>` no está registrado ni existe en
   el repo. Módulos disponibles: <lista>". Ofrecer vía `question`: registrar como
   módulo externo/nuevo (flujo 3 sin directorio) o abortar.

### Fase R — Investigación (`prompts/research-module.md`)

1. Cargá `.opencode/task-system/prompts/research-module.md`.
2. Sustituí TODOS los placeholders con los datos de la fila del registro:
   `{{MODULO}}` · `{{TIPO}}` · `{{ECOSISTEMA}}` · `{{USUARIOS}}` ·
   `{{COMPETIDORES}}` · `{{COMPETIDOR_PRINCIPAL}}` · `{{DOC_API}}` ·
   `{{USUARIOS_DETALLE}}` (= columna Usuarios expandida).
3. Ejecutá el proceso completo de investigación (fuentes internet + interno).
4. El informe DEBE incluir el **Apéndice de hallazgos H-NN**: todo hallazgo con ID,
   categoría sugerida, severidad, esfuerzo, file:line. Sin ese apéndice el informe
   está incompleto (es la entrada de la Fase D).

### Fase D — Decisiones por hallazgo (HITL, cero pérdida de datos)

> Principios canónicos: `prompts/question-gates.md`. Cada hallazgo del apéndice
> H-NN recibe UNA decisión explícita o entra por default-explícito registrado.

1. Extraé del informe la lista completa de hallazgos H-01..H-NN.
2. Agrupal por categoría sugerida en rondas de **máx 5 hallazgos por pregunta**,
   usando `question` con `multiple: true`:
   - Cada opción lista: `H-NN — <título corto> (severidad, esfuerzo 🟢🟡🔴)`.
   - Pregunta tipo: *"Hallazgos APLICAR (<n>): ¿cuáles ejecutamos ahora?"*
   - Default recomendado = la sugerencia del apéndice.
3. Hallazgos ESTRATEGIA → pregunta individual dedicada (no agrupados).
4. **Ningún H-NN puede quedar sin destino:** si una ronda no menciona un hallazgo,
   va a una pregunta final de barrido ("¿Qué hacemos con los restantes?") o entra
   como default-explícito registrado.
5. Materialización (idéntica a `prompts/research-decide.md` Fase 3):
   - APLICAR/MEJORAR/AGREGAR/OPTIMIZAR → fila en Backlog (prefijo según módulo;
     genéricos FIND-\*) con contrato verificable.
   - DESCARTAR → `docs/avance/decisiones/wontfix.md` con motivo.
   - Quick wins aprobados → plan file con waves, listo para `/pipeline run`.
   - Estrategia → `campaign_memory_write(decisions)` + sugerir ADR.
6. Emití `campaign_emit_event(event="inv-research.completed", data={modulo, hallazgos: NN, decisiones: NN})`.

## Output final

```markdown
## Investigación: <módulo>
Informe: docs/reviews/research-<módulo>-<fecha>.md (score global X.X/10)
Hallazgos: NN (aplicar N · mejorar N · agregar N · optimizar N · estrategia N · descartar N)

## Decisiones registradas
| H-NN | Decisión | Destino (Backlog ID / wontfix / plan) |
...

## Próximo paso recomendado
/pipeline run docs/plans/<plan>.md        (si hubo quick wins)
/research <siguiente-módulo>              (continuar investigación)
/research synthesis                       (cuando estén los 7 módulos)
```

RESULTADO con SKILLS_CARGADAS y GATES_EVALUADOS.
