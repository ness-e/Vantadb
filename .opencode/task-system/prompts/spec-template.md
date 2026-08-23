> **PLANTILLA — Mini-Spec (spec-driven guiado, fuente única)**
> Cuándo: toda tarea **feature-add o lógica nueva** (Gate P/Gate D de
> `question-gates.md`) ANTES de aprobar DO y ANTES del task file.
> Las decisiones abiertas (⚠️) se resuelven con el usuario vía `question` tool —
> una ronda, opciones concretas + default recomendado. Sin spec completa → no hay ACT.

# Spec: <ID> — <título>

- **Tipo:** feature-add | lógica nueva | refactor-comportamental
- **Origen:** backlog | usuario | discovery
- **Fecha:**

## 1. Problema
Qué problema real resuelve, en ≤5 líneas. Evidencia (issue, queja, benchmark, caso de uso).

## 2. Criterio de aceptación
Comandos/comportamientos observables que prueban que está resuelto (se convierten en el `Contrato` del task file):
1. `<comando mecánico>` pasa
2. <comportamiento específico verificable>

## 3. Alcance
- **Incluye:** ...
- **NO incluye:** ... (explícito — el anti-scope-creep se define acá)

## 4. Diseño propuesto
≤10 líneas: archivos a tocar, firmas nuevas/cambios de firma, flujo de datos.

## 5. Decisiones abiertas (⚠️ → question al usuario)

> **Profundidad mínima por fila:** ≥2 alternativas **REALES** (enfoques
> materialmente distintos, no variantes cosméticas) + costo/tradeoff de una
> línea por opción. Si solo existe un camino viable, NO inventar opciones de
> relleno: registrar el camino único con su evidencia (`ref: archivo:línea`
> o doc oficial) y marcarlo ✅ decidido-por-evidencia.

| # | Decisión | Opciones (+tradeoff) | Default recomendado |
|---|----------|----------------------|---------------------|
| 1 | ej: API sync o async | sync (simple, bloquea caller) / async (throughput, complejidad) | sync |

> Cada fila se pregunta con `question` antes de cerrar la spec. La respuesta
> queda registrada acá (columna **Resuelto**) y en la recitation.

## 6. Riesgos y blast radius esperado
Top 3 riesgos (alimentan el Risk Register del plan) + archivos estimados.

---
**Estado de la spec:** ⬜ borrador → 🔄 preguntas enviadas → ✅ confirmada por el usuario

Al confirmarse: la spec se pega en la sección `## Spec` del task file y el
`Contrato` se copia de §2. Gate mecánico: un task file feature-add sin `## Spec`
llena NO pasa a ACT (ver pipeline-full.md §Discovery).
