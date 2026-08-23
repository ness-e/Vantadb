---
description: Start spec-driven development — write a structured specification before writing code
---

Invoke the spec-driven-development skill.
If requirements are ambiguous, also invoke the interview-me skill to extract what the user actually needs.

## Flujo automático — la IA pregunta con la tool `question`

> Principios de formato heredados de `prompts/question-gates.md` (fuente única):
> opciones concretas + default recomendado (`(Recomendado)` primero), máximo
> 1 ronda, nunca pregunta abierta sin contexto, nunca asumir GO.

### Paso 1 — Contexto primero (no preguntes lo que el código ya responde)

Antes de hacer UNA sola pregunta, derivá todo lo posible del repositorio:

1. `codegraph_explore` del área afectada (estructura, patrones, stack real)
2. Docs existentes: README, `docs/api/`, `docs/architecture/adr/`, specs previas
3. Decisiones anteriores: `campaign_memory_read(file="decisions")`
4. **Validación web** — si una decisión involucra elegir librería/API/tecnología
   externa: validar contra docs oficiales (`websearch`/`webfetch`) ANTES de
   proponerla como opción (Regla 0 de AGENTS.md). Un tradeoff citado vale más
   que un adjetivo.

Cada dato derivable del contexto entra directo al spec como **decisión tomada**
con su evidencia citada (`ref: archivo:línea`). Esto minimiza las preguntas.

### Paso 2 — Tabla de decisiones abiertas

Listá SOLO lo que el contexto no puede responder. Formato y **profundidad
mínima** según `prompts/spec-template.md` §5: ≥2 alternativas reales con su
tradeoff de una línea; si hay un solo camino viable, registrarlo con evidencia
(✅ decidido-por-evidencia), nunca opciones de relleno.

| # | Decisión | Opciones (+tradeoff) | Default recomendado |
|---|----------|----------------------|---------------------|
| 1 | ej: auth por sesión o JWT | session (simple, revocable) / JWT (stateless, revocación compleja) | session |

Si no hay decisiones abiertas (todo era derivable), saltá directo al Paso 4.

### Paso 3 — UNA ronda de `question` (batch)

Envialas TODAS en una sola llamada `question` (soporta múltiples preguntas),
cada una con sus opciones y el `(Recomendado)` primero. Reglas:

- Nunca preguntas abiertas sin opciones ("¿qué querés?" ❌)
- Si el harness no expone `question`: reporte estructurado con la tabla del
  Paso 2 y STOP esperando input — nunca asumir respuestas
- Las respuestas quedan registradas en la columna **Resuelto** de la tabla
  y en la recitation / `campaign_memory_write(file="decisions", ...)`

### Paso 4 — Escribir `SPEC.md`

Generá el spec cubriendo las seis áreas core: objective, commands, project
structure, code style, testing strategy, boundaries — usando contexto (Paso 1)
+ respuestas (Paso 3).

Guardar en `SPEC.md` (raíz) o `docs/SPEC.md`. Considerá crear
`docs/architecture/adr/` para decisiones arquitectónicas.
Después de escribir el spec, registrar la decisión en memoria:
- `campaign_memory_write(file="decisions", entry="Spec: {nombre} — {decisión clave}")`

**Decision gates antes de escribir:**
- ¿Hay specs, ADRs o design docs previos? Leerlos primero (Paso 1).
- ¿El objetivo es claro para escribir acceptance criteria testeables? Si no,
  es una fila más de la tabla del Paso 2.
- ¿El tech stack necesita validación? Ejecutar `/audit quick` tras escribir el spec.

## Output format

```markdown
# Spec: [Project/Feature Name]

## Objective
[1-2 lines]

## Target Users
[who, what problem]

## Core Features
- [ ] Feature 1: [AC: what "done" looks like]

## Tech Stack
[language, framework, deps, constraints]

## Boundaries
- Always: [patterns to follow]
- Ask first: [decisions requiring approval]
- Never: [prohibited patterns]

## Testing Strategy
[how to verify each feature]

## Project Structure
[file tree or module layout if known]
```
