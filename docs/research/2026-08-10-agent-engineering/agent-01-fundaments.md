# AGENT-01 — Fundamentos y Mejores Prácticas de Agentes Efectivos

> **Rama de investigación**: Fundamentos y mejores prácticas de agentes efectivos (patrones probados de diseño de agentes)
> **Fecha**: 2026-08-10
> **Idioma**: Español (términos técnicos en inglés)
> **Objetivo**: Consolidar qué debe hacer un agente (y un sub-agente) correctamente para cumplir una tarea, proyecto, plan o investigación. NO modifica código.

---

## Índice

1. [Definición operativa](#1-definición-operativa)
2. [Fuentes clave (mapa de evidencia)](#2-fuentes-clave-mapa-de-evidencia)
3. [Área 1 — Patrones fundamentales de agentes](#3-área-1--patrones-fundamentales-de-agentes)
4. [Área 2 — Diseño de tools / function calling](#4-área-2--diseño-de-tools--function-calling)
5. [Área 3 — Planificación dentro del agente](#5-área-3--planificación-dentro-del-agente)
6. [Área 4 — Evaluación de agentes (evals)](#6-área-4--evaluación-de-agentes-evals)
7. [Área 5 — Guardrails y seguridad](#7-área-5--guardrails-y-seguridad)
8. [Área 6 — Contexto y memoria](#8-área-6--contexto-y-memoria)
9. [Área 7 — Auto-corrección y reflexión](#9-área-7--auto-corrección-y-reflexión)
10. [Área 8 — Qué enseña cada gran actor](#10-área-8--qué-enseña-cada-gran-actor)
11. [Checklist: lo que un agente correcto DEBE hacer](#11-checklist-lo-que-un-agente-correcto-debe-hacer)

---

## 1. Definición operativa

- **Agente (Anthropic, 2025)**: un LLM (modelo) que usa **tools de forma autónoma en un loop** para resolver una tarea — el modelo dirige el flujo, los pasos y las herramientas.
- **Workflow**: sistemas orquestados por código predefinido donde el LLM solo ejecuta pasos acotados.
- **Diferencia clave**: en un *workflow* los caminos están predefinidos; en un *agente* el LLM **decide dinámicamente** los caminos y usa herramientas para interactuar con el entorno.
- **Valor del agente**: el modelo obtiene **ground truth del entorno en cada paso** (no solo de su conocimiento entrenado) y corrige el rumbo con evidencia real.
- Fuente: https://www.anthropic.com/engineering/building-effective-agents

---

## 2. Fuentes clave (mapa de evidencia)

| # | Fuente | Tema | URL |
|---|--------|------|-----|
| 1 | Anthropic — Building Effective Agents (dic 2024) | Patrones de workflows/agentes | https://www.anthropic.com/engineering/building-effective-agents |
| 2 | Anthropic — Writing Effective Tools for Agents (sep 2025) | Diseño de tools | https://www.anthropic.com/engineering/writing-tools-for-agents |
| 3 | Anthropic — Effective Context Engineering (sep 2025) | Contexto y memoria | https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents |
| 4 | Anthropic — Demystifying Evals for AI Agents (ene 2026) | Evaluación | https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents |
| 5 | Anthropic — Effective Guardrails for AI Agents (feb 2026) | Guardrails | https://www.anthropic.com/engineering/effective-guardrails-for-ai-agents |
| 6 | Anthropic — Effective Harnesses for Long-Running Agents (nov 2025) | Agentes long-running | https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents |
| 7 | Anthropic — Claude Code: Best Practices for Agentic Coding (2025) | Harness/agente de código | https://www.anthropic.com/engineering/claude-code-best-practices |
| 8 | Simon Willison — Definition of agent (sep 2025) | LLMs autonomously using tools in a loop | https://simonwillison.net/2025/Sep/18/agents/ |
| 9 | OpenAI — A Practical Guide to Building Agents (2025) | Diseño y orquestación | https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/ |
| 10 | Google / Kaggle — Introduction to Agents whitepaper (nov 2025) | LLM+tools+context: ejes del agente | https://howaiworks.ai/blog/kaggle-google-introduction-to-agents-whitepaper-2025 |
| 11 | Google — Agent Quality (ago 2025) | Evals en 3 pilares | https://ai.google.dev/resources/agents-quality |
| 12 | DeepMind — AlphaEvolve (may 2025) | Agente de código + evaluador automático | https://deepmind.google/blog/alphaevolve-a-gemini-powered-coding-agent-for-designing-advanced-algorithms/ |
| 13 | LangChain — Improving Deep Agents (feb 2026) | Harness engineering, Top 5 Terminal Bench | https://blog.langchain.com/improving-deep-agents-with-harness-engineering/ |
| 14 | Cognition — Multi-Agents: What's Actually Working (abr 2026) | Orquestación multi-agente | https://cognition.com/blog/multi-agents-working/ |
| 15 | Trek (Vercel) — Hacking the North Star (ago 2025) | Agent benchmark gaps | https://trek.io/blog/hacking-the-north-star |
| 16 | HumanLayer et al. — Evolving Agent Research (2025) | Rutas de investigación de agentes | https://www.humanlayer.dev/ ... /evolving-agent-research-why-your-agent-fails/ |
| 17 | ReAct: Synergizing Reasoning and Acting in Language Models (Yao et al., 2022) | Ilustra la mecánica del loop | https://arxiv.org/abs/2210.03629 |
| 18 | Tree of Thoughts (Yao et al., 2023) | Planificación/búsqueda | https://arxiv.org/abs/2305.10601 |
| 19 | Self-Refine (Madaan et al., 2023) | Auto-corrección | https://arxiv.org/abs/2303.17651 |
| 20 | Reflexion (Shinn et al., 2023) | Retrospección con memoria verbal | https://arxiv.org/abs/2303.11366 |
| 21 | CRITIC (Gou et al., 2024) | Auto-corrección con tools externas | https://arxiv.org/abs/2305.11738 |
| 22 | LATS: Language Agent Tree Search (Zhou et al., 2023) | Búsqueda+reflexión | https://arxiv.org/abs/2310.04406 |
| 23 | Huang et al. — LLMs Cannot Self-Correct Reasoning Yet | Crítica a la auto-corrección pura | https://arxiv.org/abs/2310.01798 |

---

## 3. Área 1 — Patrones fundamentales de agentes

### 3.1 Patrones de *workflow* (flujos controlados por código)

| Patrón | Dónde funciona | Riesgos |
|--------|---------------|---------|
| **Prompt chaining** | Tareas que pueden descomponerse en pasos secuenciales fijos | Si un paso falla, arrastra al resto |
| **Routing** | Clasificar inputs y dirigir a un handler especializado | Necesita categorías bien distintas |
| **Parallelization** | Subtareas independientes (seccionar, votar, evaluar) | Coordinación de resultados |
| **Orchestrator–workers** | Tareas complejas donde no se sabe de antemano cuántos subtarea | El orquestador es punto único de fallo |
| **Evaluator–optimizer** | Un generador + un evaluador que iteren | Coste de doble llamada |

Fuente: #1 (Anthropic Building Effective Agents).

### 3.2 Cuándo usar un *agente* (no un workflow)

- La complejidad no se puede predecir de antemano.
- Se necesitan **decisiones abiertas** informadas por el entorno (no clasificar lists de opciones).
- Los errores pueden corregirse con herramientas (retry, buscar docs, ejecutar) — sobre todo en **respuestas a preguntas abiertas, análisis de código, control de herramientas o autocompletado de archivos**.
- Regla de oro: resistencia y escalabilidad a costa de **previsibilidad y coste** (más tokens, más latencia, decisiones no deterministas).

### 3.3 Elementos de un sistema de agente (harness)

Según #3 y #6, los componentes son:
1. **Model** (LLM que razona y decide).
2. **Harness** — el código que envuelve al modelo: instrucciones/`system prompt`, guardrails, herramientas disponibles, parsing de respuestas, loop.
3. **Tools** — superficie de acción sobre el entorno.
4. **Context** — lo que el modelo "ve" (página actual del archivo, historial, resumen de sesión).
5. **Environment** — dónde operan las tools (CLI, navegador, sandbox).

### 3.4 El loop del agente (ilustrado por ReAct, #17)

1. **Reason** → pensar qué se necesita.
2. **Act** → invocar una tool y obtener resultado real.
3. **Observe** → registrar el resultado en contexto.
4. **Repeat** hasta que el objetivo esté verificado → **finalizar** con un resultado sintetizado.

---

## 4. Área 2 — Diseño de tools / function calling

Fuente principal: #2 (Anthropic Writing Effective Tools for Agents). Principios:

### 4.1 Elegir las tools correctas

- **Identificar las reales**: definir qué tareas el agente DEBE realizar; no añadir tools por especulación.
- **Nombrar explícitamente las tools que NO debe habilitar** cuando aplica (reducción de superficie de error).
- Elegir un **grado de síntesis adecuado**: a veces conviene exponer primitivas de bajo nivel (1 archivo/1 función), a veces una tool de alto nivel (navegador entero, test runner).
- **Namespacing**: herramientas con nombres/objetivos solapados confunden al modelo; agrupar por dominio.

### 4.2 Firma y descripción de tools

- **Nombres representativos** del efecto real (no abreviaturas crípticas).
- **Parámetros fuertemente tipados y descriptivos** — una firma débil produce llamadas malformadas repetidas.
- **Prompts de descripción con ejemplos**. Ejemplo convertido (Anthropic) de débil → fuerte:
  - Débil: `"Use this tool for document processing."`
  - Fuerte: `"This tool extracts the text content from DOCX files and returns it as plain text. Useful for reading Word documents without formatting. Returns a transcript of the document's visible paragraphs and table entries. Note: embedded images and hyperlink targets are NOT included in the output."`

### 4.3 El protocolo de tools mal escritas

- Herramientas que fallan → rellaman en bucle.
- **Errores útiles**: el error debe describir *qué* pasó y *cómo* corregirlo (incluir `code`, `error_type`, `args_validated`, hint). Toot `return_on_error=True` para que el modelo vea el error sin excepción.
- **Resultados ricos en contexto**: devolver metadatos (ej. dossier de un personaje: herramientas usadas, autoestima, prompt) para que el agente "sepa lo que hizo".
- **Eficiencia de tokens**: embeddings grandes devuelven vectores — los resultados se leen en el contexto; devolver sólo lo necesario.

### 4.4 Reglas para el flujo tool loop

- Devolver el resultado **dentro de la conversación** (el modelo lo observa).
- Guardar estados intermedios (working memory) en archivos/notas estructuradas, **no** en el contexto sin límite.
- `stop_turn_every_tool_result` evita que el modelo haga varias tools a la vez arrastrando errores de batch.

---

## 5. Área 3 — Planificación dentro del agente

### 5.1 Patrones de planificación

1. **Plan–execute** (pipeline): el agente produce un plan iterativo con objetivos (*goals*), tareas (*tasks*), hitos (*milestones*), prioridades (*priorities*), restricciones (*constraints*). Re-planifica al completar hitos.
2. **Tree of Thoughts (#18)**: busca sobre un árbol de estados de razonamiento con evaluación/poda; más caro pero mucho más resiliente en tareas de planificación complejas.
3. **LATS (#22)**: combina Monte Carlo Tree Search + Reflexion — planifica, ejecuta, observa y actualiza por retroalimentación del entorno.
4. **Re-planning**: cuando el entorno no coincide con el plan (tool fails, contexto cambia), el agente debe **descartar el plan obsoleto** y generar uno nuevo (no insistir).

### 5.2 Gestión de la "cola de trabajo" en contexto

- Mantener un **TODO visible y actualizado** en cada respuesta (el modelo debe "saber" cuántos pasos van y cuántos faltan).
- Actuar **sobre un estado persistente** (variables de sesión, archivos) en lugar de re-derivar todo desde el contexto que crece.
- Cuando el contexto crece, **mover lo completado a un resumen/compaction** y mantener en el frente sólo el plan activo.

### 5.3 Errores típicos de planificación del agente

- Plan genérico que no se reconcilia con la realidad del entorno.
- No **verificar** el resultado antes de marcarlo "completo".
- Insistir en un plan con herramientas que fallan → requiere **no-progress detection** (§7).

---

## 6. Área 4 — Evaluación de agentes (evals)

Fuente principal: #4 (Anthropic Demystifying Evals). También #11 (Google Agent Quality) y #15 (Trek).

### 6.1 Vocabulario común

- **Task / problem**: unidad de evaluación (ej. "actualiza el CHANGELOG").
- **Trial**: ejecución única sobre una task.
- **Grader / check**: criterios verificables (assertions) sobre el resultado.
- **Transcript / trace / trajectory**: registro paso a paso del loop (tool calls, outputs).
- **Eval set**: colección de tasks + graders que representan "lo que importa".

### 6.2 Pasos para construir evals

1. **Elegir ocurrencias reales** (no casos sintéticos genéricos).
2. **Escribir problemas claros y acotados** (una gold path de referencia).
3. **Escribir el grader** (lo que cuenta como éxito) — cuanto más objetivo y con assertions, mejor.
4. **Correr múltiples trials** por task (no determinismo).
5. **Volver al ciclo**: los fracasos de evals se convierten en nuevas tareas de evals (evals evolutivos).

### 6.3 Dimensiones de calidad (Google Agent Quality, #11)

- **Prompt quality** (entrada/respuesta por turns).
- **Tool quality** (calls válidas, fallos de tool, outputs).
- **Performance quality** (counterfactuals, razonamiento intermedio).
- Se pueden puntuar con **trace-based spoofed evals** y **LLM-as-a-judge**.

### 6.4 Lo que muestran los benchmarks (#15)

- Trek: los problemas reales de agentes a menudo no se capturan en los benchmarks de agentes estándar → los problemas difieren del **determinismo de tareas reales**.
- LangChain (#13): la adición de **traces y verificación** (harness engineering) produjo un salto de Top 30 → **Top 5 en Terminal-Bench 2.0**, sin cambiar el modelo.
- Conclusión: **evaluar el loop completo** (tools + contexto + harness) y no only el LLM.

---

## 7. Área 5 — Guardrails y seguridad

Fuente principal: #5 (Anthropic Effective Guardrails). Otras: #10.

### 7.1 Tipos de guardrails

| Tipo | Ejemplo | Mitiga |
|------|---------|--------|
| **Input guardrails** | validación/limitaciones de entrada del prompt, redacción automática | prompt injection, jailbreaks |
| **Tool guardrails** | permisos explícitos por tool (aprobar/denegar), sandboxing | acciones dañinas |
| **Context guardrails** | límites de contexto, verificación de edad de la información | alucinaciones sobre datos viejos |
| **Output guardrails** | validación de formato, checks post-proceso | salidas malformadas |
| **Sandboxing** | ejecutar tools en contenedor/VM aislada con permisos mínimos | escapes y daño al host |

### 7.2 Enfoque estructural "behind a boundary" (#5)

- **Enfoque por políticas**: mínimo privilegio, granularidad de permisos (archivos, red, ejecución).
- **Trigger on dissent**: preguntar al usuario sólo cuando se cruza un límite definido (evita fatiga de prompts de permiso).
- **Model guardrails**: instrucciones del modelo para permanecer en contexto.
- **Ciclo continuo**: redactar/refinar guardrails con evals.

### 7.3 Escalado de permisos (datos de Anthropic)

- Con **permission prompts**: humanos aprueban ~85% de acciones → fatiga.
- Con **rule-based policy** + sandboxing: bloqueo automático/solicitud solo al superar límite → tiempo de desarrollo por solicitud **cae 10–20×**, ~84% menos moratorias, estable a lo largo del tiempo.
- Regla: **sandbox por defecto, permisos explícitos y revisables**, y auditar/logs de todas las acciones.

### 7.4 Seguridad del contexto

- **Data provenance**: etiquetar información con origen y fecha, para que el agente distinga lo verificado de lo no verificado.
- **Límite de edad del contexto**: marcar cuándo la información es antigua.
- **Alucinación con tools**: verificar con el entorno antes de afirmar.

---

## 8. Área 6 — Contexto y memoria

Fuente principal: #3 (Anthropic Effective Context Engineering).

### 8.1 Conceptos centrales

- **Context rot**: a medida que crecen los tokens, la capacidad del modelo para recuperar información **decae** (especialmente en medio de la ventana).
- **Attention budget**: el "presupuesto" de atención útil es menor que la ventana total; no llenar el contexto con relleno.
- **Attention cliff**: una zona razonable de atención es empleada en los extremos; en el medio el recall es peor.

### 8.2 Técnicas de context engineering (router/strategies)

- **Single context ceiling**: mantener un único contexto bajo el "techo" — si crece demasiado, **compaction** (resumir lo viejo) o pasar a **multi-context** (mover a archivos).
- **Structured note-taking**: el agente redacta resúmenes/planos y los relee en vez de mantener todo en la ventana.
- **Multi-agent sub-architectures**: separar agentes con contextos cortos y task-specific.
- **Runtime retrieval**: sacar del contexto los detalles que deben extraerse bajo demanda (archivos, docs, tools).
- **Contexcoloc = más contexto no siempre mejor**: ser selectivo en qué se incluye.

### 8.3 Compactación / resumen

- Al resumir el historial: **verificar que el resumen conserve la información clave del plan y sus estados**; los compactions mal hechos pierden intención (fuente comunitaria: TianPan, 2025).
- Guardar en **archivos persistentes** los estados intermedios y planes, para poder reconstruir el contexto de la próxima sesión (harness long-running, #6).

---

## 9. Área 7 — Auto-corrección y reflexión

### 9.1 Patrones

- **Self-Refine (#19)**: generar → obtener feedback (del mismo modelo o evaluador) → **refinar** en loop. Iterativo, no paralelo.
- **Reflexion (#20)**: el agente almacena una **memoria verbal** (auto-comentario) de la trayectoria y errores, y la consulta en la siguiente tentativa (equivale a "lecciones aprendidas" en disco).
- **LATS (#22)**: búsqueda en árbol + reflexión al fallar.
- **CRITIC (#21)**: usar **tools externas** (librerías, validadors, sandbox) como veto para corregir salidas; no depender solo del propio juicio.

### 9.2 Advertencia importante (#23)

- **Los LLM no se auto-corrigen bien sin herramientas externas**: en razonamiento puro, la auto-corrección rara vez mejora y a menudo degrada (falta de feedback del entorno).
- Implicación: la auto-corrección efectiva requiere **validación con el entorno** (ejecutar, comprobar, leer la salida) — no "suponer" que el output es correcto.

### 9.3 Receta de reflexión aplicada a un loop de trabajo

1. Tras cada hito: **verificar** (test, leer salida, diff).
2. Si falla: **registrar la causa** en una nota persistente o trayectoria.
3. **Actualizar el plan**; nunca re-ejecutar a ciegas la misma acción que falló.
4. Cerrar con **resumen de resultados** (DONE) si el objetivo está verificado.

---

## 10. Área 8 — Qué enseña cada gran actor

| Actor | Lección principal |
|-------|-------------------|
| **Anthropic (#1, #2, #3, #6, #7)** | Harness engineering (context, tools, guardrails) > elegir el modelo; evals continuos; sandbox por defecto; trabajo single-threaded con archivos. |
| **OpenAI (#9)** | Fundamentos de diseño: elegir el modelo según latencia/coste, definir tools claras, configurar instrucciones, y entender orquestación (single vs multi vs decentralized); evitar over-engineered multi-agent sin necesidad. |
| **Google/Kaggle (#10)** | El agente = **modelo + tools + contexto**; guiar la tarea con instrucciones/system prompt; el contexto se construye (trajectory, memory, working memory, ephemeral). |
| **Google DeepMind (#12)** | AlphaEvolve: agente de código autónomo apoyado en **evaluadores automáticos** (metric/tools) que proveen el "objetivo" que guía la búsqueda del agente. |
| **LangChain (#13)** | **Harness engineering mueve métricas**: system prompt + tool choice + ejecución/flujos + verificación mueven un agente de Top 30 a Top 5 en un benchmark estándar, con el mismo modelo. |
| **Cognition (#14)** | **Multi-agentes: no por defecto**. Los swarms paralelos de escritores fallan; el patrón que funciona es "mucho juicio, un solo autor". Managers demasiado prescriptivos fallan; los agentes asumen estado compartido que no existe. |
| **HumanLayer et al. (#16)** | Los agentes más citados en industria fallan por: calidad del contexto, tools mal diseñadas, evals insuficientes y falta de loops de retroalimentación. |

---

## 11. Checklist: lo que un agente correcto DEBE hacer

Clasificado por fase. Verdadero check-list accionable para tarea, proyecto, plan o investigación.

### Fase A — Inicio (setup del agente)

- [ ] **Leer** la tarea completa y confirmar su alcance antes de actuar (brainstorming/plan only si requisitos poco claros).
- [ ] **Definir el harness**: system prompt claro, tools habilitadas y deshabilitadas, guardrails y sandbox.
- [ ] **Reunir contexto** en target points (archivos, docs, esquemas) en lugar de relleno; no laboral con contexto no utilizado.
- [ ] **Preparar evals**: criterios de éxito verificables (grader/assertions) y casos muestra del mundo real.
- [ ] **Elegir modelo** según latencia/coste vs calidad de razonamiento.
- [ ] Decidir si un workflow (rutas fijas) vale, antes de montar un agente o multi-agentes.

### Fase B — Ejecución (en loop)

- [ ] **Revisar** qué se ha hecho y qué se está haciendo; mantener un TODO visible.
- [ ] **Buscar/leer** la evidencia del entorno (ground truth) antes de afirmar.
- [ ] **Actuar con tools** y **observar** el resultado (no asumir que "funcionó").
- [ ] **Escribir pasos/estados intermedios** en archivos persistentes (working memory) en vez de solo en la conversación.
- [ ] **Plantear y replantear** el plan según los resultados reales (plan–execute, re-planning; LATS/ToT si la tarea lo exige).
- [ ] **Guardar el contexto bajo control**: compaction/structured-note cuando crece; respetando el attention budget.
- [ ] **Registrar lecciones** (Reflexion) tras cada fallo: causa → nota persistente → próxima tentativa.
- [ ] Corregir fallos **con validación externa** (executar/test/leer output) — no auto-corrección "supuesta".
- [ ] No entrar en buttocks: detectar **no-progress** (misma acción fallida repetida) y cambiar de estrategia.

### Fase C — Verificación

- [ ] **Comprobar resultados con el entorno** (tests, diff, desploy) — no "parece correcto".
- [ ] Evaluar con el **grader/assertions** predefinido; correr ≥2 trials si no determinista.
- [ ] Revisar **traces/transcript** del loop si una trial falla (¿tool call válida? ¿contexto correcto?).
- [ ] Verificar que los **resultados responden la tarea original**, no una próxima interpretación.
- [ ] **Auditar logs y permisos** (guardrails) de la sesión.

### Fase D — Cierre (handoff)

- [ ] **Sintetizar un resumen final** claro de qué se hizo y qué falta (DONE/resumen).
- [ ] **Dejar artefactos para la próxima sesión**: plan actualizado, estado, resumen, notas persistentes.
- [ ] Actualizar **evals evolutivos** con los casos que fallaron.
- [ ] Comunicar **límites** de lo completado y riesgos/decisiones pendientes.
- [ ] Confirmar que el entregable está en el lugar correcto (archivo/PR/doc) y con enlaces/líneas referenciadas.

---

*Fin del documento. Bases de fuentes en §2; los enlaces permiten verificación directa.*