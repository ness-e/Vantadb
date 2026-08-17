# TIR-08 — Saturación <20% + Broadening/Narrowing + jitter en retry

**Fecha:** 2026-08-17 · **Tarea:** TIR-08 (read-only) · **Fuente:** backlog P18

## 1. Inventario de los 3 criterios

| # | Criterio | Dónde vive hoy | En prompts (task-system) |
|---|----------|----------------|--------------------------|
| 1 | **Saturación <20% como stop**: "una fase termina cuando una ronda nueva de búsqueda añade <20% de fuentes nuevas o ninguna fuente sobre umbral de citas/relevancia"; evaluar por fuente | `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md:275-279` (§7.6); también `:240` (§7.2), `:476` (§14 Fase A), `:483` (§14 Fase B) | ❌ ausente. Único 20% en prompts = Context Budget (`iter-loop-tools.md:372`) — es otra cosa, no saturación de research |
| 2 | **Broadening/Narrowing como re-enfoque**: narrowing si hay suficiente relevancia; broadening si insuficientes resultados o poca relevancia | `agent-02-task-execution.md:236-240` (§7.2); `:478-483` (§14 Fase B) | ❌ ausente. Estado RESEARCH permite websearch (`iter-loop-tools.md:134`) pero sin regla de re-enfoque |
| 3 | **Jitter en retry**: "Retry con backoff exponencial + jitter (cap de intentos)" | `agent-02-task-execution.md:302` (§8.1) | ❌ ausente. RULES.md:453-458 define backoff **determinista sin jitter**: `Start-Sleep -Seconds [Math]::Pow(2, $retryCount)` |

**Hallazgo clave:** los 3 criterios viven **solo en la investigación** (documento histórico del lote 2026-08-10, fuera del path de carga de agentes). Ninguno está en los prompts operativos (`iter-loop-tools.md`, `research-agent.md`, `task.md` — grep de `saturaci|20%|broadening|narrowing|jitter|backoff` sobre `.opencode/task-system/prompts/` solo matchea `iter-loop-tools.md:372`, falso positivo de Context Budget).

**Conflicto latente:** la investigación §8.1 dice "+ jitter"; RULES.md:453 lo implementa determinista. Los prompts no resuelven la contradicción porque no mencionan backoff numérico.

## 2. Análisis: formalizar-en-prompts vs guías tácitas

### A favor de formalizar
- **Invisibles hoy en ejecución**: `agent-02-task-execution.md` no está en el load path de skills/prompts del pipeline (campaign_load_skills no lo carga). Un agente corriendo una research task (estado RESEARCH / deep research) nunca ve los criterios → la saturación como stop es letra muerta. Es exactamente el fallo "parada tardía / loop infinito" de §9.1 del propio doc.
- **Enforceability**: el task-system se basa en "contrato verificable" y "verificación mecánica, nunca auto-reporte" (`iter-loop-tools.md:377-378`). El 20% es una regla mecánica y medible (¿la ronda añadió fuentes nuevas? sí/no) — encaja con la cultura del repo (tabla ❌ Vago vs ✅ Verificable).
- **Costo mínimo**: son ~6 líneas en un solo archivo (research-agent.md, 28 líneas hoy).

### En contra de formalizar
- **Ruido en hot path**: `iter-loop-tools.md` ya tiene 384 líneas y se carga en CADA iteración de cualquier tarea; meter criterios de research allí encarece el contexto de tareas no-research.
- **Costo de mantenimiento**: formalizar crea dos fuentes de verdad (prompt + investigación) que pueden desincronizarse.
- **Alternativa tácita**: la investigación es la referencia; el contrato explícito de cada task file ya funciona como stop para tareas acotadas (como esta TIR-08).

### Veredicto intermedio
Los criterios aplican solo a **research abierta** (Fase B de §14), no a toda iteración. Por eso el hogar correcto es `research-agent.md` (prompt dedicado de research, se carga solo cuando hay research) — no `iter-loop-tools.md` (evita el ruido en el 20% del contexto de cada iteración) ni `task.md` (es template de definición, no comportamiento runtime).

## 3. Recomendación

**IMPLEMENTAR (parcial)** — formalizar los criterios 1 y 2 en `research-agent.md` como bloque compacto (~6 líneas: stop por saturación <20% por fuente + broadening/narrowing como regla de re-enfoque, citando `agent-02-task-execution.md` como referencia). **WONTFIT** para el criterio 3 (jitter): mantener el backoff determinista `2^retry` de RULES.md:453 — es reproducible y testeable (valores exactos en tests), y el thundering-herd que el jitter mitiga no aplica a un pipeline de tareas single-agent. La contradicción con §8.1 se resuelve documentando en RULES.md que el jitter es deliberadamente descartado (decisión, no omisión).

**Justificación:** los criterios 1 y 2 son operativamente muertos hoy (fuera del load path) y su formalización cuesta ~6 líneas en un archivo de 28 — el cambio mínimo que los vuelve enforceables. El criterio 3 ya está implementado de forma correcta para este pipeline (determinista); agregar jitter sería ruido sin beneficio medible.

### Alternativas consideradas
- **Deferir**: razonable si la research abierta es rara (hoy las tasks de research llevan contrato explícito). Se descarta porque el sistema ya tiene estado RESEARCH + workflow research JSON: el modo existe, solo le faltan las reglas de parada/re-enfoque.
- **Formalizar en iter-loop-tools.md**: se descarta por ruido en el hot path de todas las tareas.
- **WONTFIT total**: se descarta — dejaría la saturación como letra muerta y el conflicto de jitter sin resolver.

## Verificación
- Contrato TIR-08: ✅ archivo existe, sección "Recomendación" con opción explícita (IMPLEMENTAR).
- NO se editaron prompts. NO se commiteó (el lead commitea).