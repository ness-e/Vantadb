# TIR-01 — Compaction de contexto runtime: ¿resumen incremental por fase?

> **Tipo:** Investigación/Decisión (read-only) · **Fecha:** 2026-08-17
> **Fuente:** backlog P18 (línea 448) · **Task file:** `.opencode/skills/campaign-executor/tasks/TIR-01.md`
> **Alcance:** NO modifica prompts ni código. Solo análisis + recomendación para el lead.

---

## 1. Estado actual del harness (mecanismos existentes)

| Mecanismo | Dónde | Qué es | Cuándo ocurre |
|---|---|---|---|
| **Recitation block** | `RULES.md` Rule 3/18 | Bloque al final del plan file; única cosa que persiste entre iteraciones | Cada iteración |
| **Context Save Point** | `iter-loop-tools.md:291-292` | Note-taking manual al final del task file: Fecha, Branch, CI pendiente, Decisiones, Problemas conocidos, Próxima tarea | Al final de cada tarea |
| **Task file steps** | `tasks/<ID>.md` | Steps ✅/⬜ + contrato + notas — estado durable | Continuo |
| **Memoria** | `campaign_memory_read/write` (TSYS-15) | Lecciones/decisiones por línea `- fecha \| tema \| decisión \| ref` | Ad-hoc |
| **SARL escalera retry** | `subagent-recovery.md` §2 | RESUME (misma sesión) → RETRY (fresco + digest ~200 tokens) → STRATEGY → ESCALATE | Falla de sub-agente |
| **MoM ladder** | `iter-loop-tools.md:171-182` | Escalón 2 = "contexto fresco + resumen ~200 tokens" | 2ª falla mismo error |

**Hallazgo clave:** el harness actual NO tiene compaction runtime automático. Es **reset + reconstrucción desde archivos** (Rule 5: disposable agents; Rule 11: sesión cerrada por tarea; Rule 18: "el recitation block ES nuestro mecanismo de condensación"). Esto corresponde al patrón *multi-context / structured note-taking* de `agent-01-fundaments.md` §8.2, **no** al *compaction* de §8.3.

---

## 2. Análisis de alternativas

### (a) Resumen incremental por fase
Escribir un resumen estructurado en el task file al cierre de CADA fase/step (no solo al final), tipo: `Fase X → steps ✅ / decisiones tomadas / problemas abiertos / próxima acción`.

| Pro | Contra |
|---|---|
| Alinea con §8.3: "verificar que el resumen conserve la información clave del plan y sus estados" | **Riesgo documentado:** "los compactions mal hechos pierden intención" (TianPan, citado en §8.3) — un resumen por fase mal escrito es peor que el estado durable actual |
| Mejora el digest del RETRY (SARL Nivel 2 / MoM escalón 2): en vez de "~200 tokens" ad-hoc, se lee el resumen real del task file | Costo de tokens por fase + disciplina manual (el harness ya sufre note-taking inconsistente: SARL §3.3 verifica "si el sub-agente escribió el Context Save Point" — a menudo no lo hace) |
| Cierra la brecha real para tareas largas (15-30 turns como TIR-01) donde el contexto de sesión crece | Requiere tocar prompts del task-system (fuera del alcance de esta tarea; decisión del lead) |

### (b) Multi-turn compaction (claim original del backlog P18)
Compactar el historial DENTRO de la sesión (summarize + rewrite del contexto, patrón OpenHands/Anthropic context editing).

| Pro | Contra |
|---|---|
| Es lo que el claim original prometía ("compaction de contexto runtime") | **OpenCode no expone edición del historial de conversación** — el mecanismo no es implementable en este harness sin hackear el cliente |
| Reduce context rot (§8.1) en sesiones muy largas | Complejidad de harness (triggers, formato, state machine) para un caso que el diseño actual ya resuelve con reset + recitation |
| | Contradice Rule 5 (disposable agents) y Rule 18 ("no necesita lógica extra") — diseño deliberado |

### (c) Estado actual (Context Save Point manual + escalera retry)
El harness conserva: goals activos, steps ✅/⬜, recitation, decisiones de memoria, git (diffs/commits) — exactamente la tabla "qué se guarda / qué se descarta" de Rule 12. Descarta a propósito: tool outputs cerrados, logs, mensajes de éxito.

**Gap real:** el digest del RETRY es "~200 tokens" sin estructura — depende del sub-agente que lo escribe en caliente. El Context Save Point al final de la tarea no sirve para retomar en medio (se escribe al cierre, no por fase).

---

## 3. Comparación contra el claim original

| Claim P18 ("multi-turn compaction") | Realidad actual | Brecha |
|---|---|---|
| Compactar contexto runtime automáticamente | Recitation + Context Save Point + reset por iteración | El harness hace **reset + reconstrucción**, no compactación. Brecha real solo en: (1) tareas largas multi-turn dentro de una sesión, (2) calidad del digest de RETRY |
| Conservar intención entre turns | Conserva intención vía task file + recitation + memoria | Suficiente para steps atómicos (~100 líneas); insuficiente solo si el agente no escribió los artefactos |

---

## 4. Recomendación

### **DEFERIR** — no implementar multi-turn compaction; no implementar resumen por fase como mecanismo nuevo ahora.

**Justificación:**

1. **El claim original (compaction runtime automático) es WONTFIT técnicamente**: OpenCode no permite editar el historial de conversación dentro de la sesión, y Rule 5/18 del harness ya resuelven el contexto creciente con reset limpio + recitation. Automatizar compaction dentro de la sesión sería añadir complejidad de harness contra el diseño deliberado de "disposable agents".

2. **El resumen incremental por fase tiene ROI marginal hoy**: la evidencia (§8.3, TianPan) documenta que los compactions mal hechos **pierden intención** — y el harness ya sufre note-taking manual inconsistente (SARL §3.3 verifica que el sub-agente *sí* escribió el Context Save Point porque a menudo no lo hace). Un mecanismo nuevo que depende de la misma disciplina manual no cierra la brecha.

3. **Lo que SÍ vale la pena es un micro-cambio de prompt, no un mecanismo nuevo** (decisión del lead, fuera del alcance de TIR-01):
   - Extender el Context Save Point de `iter-loop-tools.md:291` para que se escriba **por fase** (no solo al final): `Fase → steps ✅ / decisiones / problemas / próxima acción`.
   - Que SARL Nivel 2 (RETRY) y MoM escalón 2 lean el **Context Save Point del task file** como digest, en vez de "~200 tokens" ad-hoc.
   - Costo: ~5 líneas en 2 prompts. Beneficio: digest estructurado real en vez de resumen improvisado.

4. **Cuándo reabrir**: si aparecen tareas de >30 turns con fallas recurrentes de intención (re-trabajo por contexto perdido), re-evaluar con evidencia de 3+ incidentes. Hasta entonces, WONTFIT.

**Pendiente para el lead:** registrar la decisión en `campaign_memory_write(file="decisions")` y, si aprueba el micro-cambio, crearlo como tarea de implementación de prompts (TIR-01 es read-only).

---

*Fin del documento. Fuentes: `agent-01-fundaments.md` §5.2/§8.1-8.3/§9, `iter-loop-tools.md:171-182,291-292`, `subagent-recovery.md` §2-3, `campaign-executor/RULES.md` Rule 5/11/12/18.*