# Ejecución de Tareas Paso a Paso y Loop de Trabajo

**Fecha:** 2026-08-10 · **Lote:** 2026-08-10-agent-engineering · **Agente:** agent-02
**Idioma:** Español (términos técnicos en inglés)
**Estado:** Completo, 28+ fuentes citadas
**Tema:** Cómo debe un agente de IA ejecutar tareas, proyectos, planes e investigaciones — patrones, framework del loop, técnicas de investigación, y protocolo operativo paso a paso.

---

## 1. Resumen Ejecutivo

- Los agentes más efectivos de producción no ejecutan "prompts gigantes": usan un **loop estructurado** (plan → act → observe → verify → repeat) con **acceso a comandos/herramientas reales** (bash, editor, buscador, tests) que les dan **ground truth del entorno**, no solo autoreporte del modelo.
- El patrón arquitectónico de referencia es **ReAct** (Reasoning + Acting): intercalar `Thought → Action → Observation` y repetir hasta que el objetivo se cumpla o se agoten los recursos. Su variantes compuestas: plan-and-execute, reflexion, self-ask, tree-of-thoughts, evaluator-optimizer.
- **Casi todos los fallos de agentes son de parada o de verificación**: parada temprana (declara victoria antes de tiempo), parada tardía (loop infinite), false completion (reporta hecho lo que no hizo). La causa raíz es la dependencia del auto-reporte. La fix: **verificación externa contra ground truth** (tests, salidas de comandos, contenido de archivos, evaluadores/jueces LLM).
- La **clarificación de requisitos** al inicio evita construir lo equivocado: técnicas como *interview-me* (una pregunta a la vez), *spec-driven* y *brainstorming* antes de codificar.
- El **Definition of Done** debe escribirse como criterios **verificables por un tercero** (números, paths, comportamientos), no como adjetivos ("funciona bien").
- La **descomposición en pasos atómicos** (cada uno con entrada, salida y check de pass/fail) es la base de tareas multi-paso estables; el *backward chaining* desde el objetivo hacia atrás garantiza que ningún paso quede huerfano.
- Las tareas de **investigación** son un caso especial: requieren iteración de búsqueda (broadening/narrowing), verificación de fuentes (triage de calidad), saturación como criterio de parada y **citación como ancla** de cada claim (claimer sin cita = alucinación).
- El **manejo de errores** debe ser explícito y clasificado (transitorios / permanentes / ambiguos): idempotencia, retry con backoff+jitter, circuit breaker, escalado automático con límites de presupuesto, y dead-letter o escalation humana.
- **Checkpointing y resumibilidad**: persistir estado entre pasos/sesiones (state muy streams, git, artefactos) permite retomar tras fallos sin recomenzar.
- **Métricas honestas**: en caso de fallo, reportar las métricas exactas observadas (steps, error code) en vez de inventar; evaluar con staged evaluation y tests de regresión.

---

## 2. Archivo de Fuentes (URLs)

### 2.1 Referencias núcleo (framework / loop)

| # | Fuente | URL |
|---|--------|-----|
| 1 | Anthropic — Building Effective Agents (dic 2024) | https://www.anthropic.com/research/building-effective-agents |
| 2 | Anthropic — Continue with Claude Code (harness) | https://docs.anthropic.com/en/docs/claude-code/continuation |
| 3 | Anthropic — Effective harnesses para long-running agents | https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents |
| 4 | Anthropic — Harness Design para long-running apps | https://www.anthropic.com/engineering/long-running-agent-best-practices |
| 5 | Claude Code Best Practices | https://www.anthropic.com/engineering/claude-code-best-practices |
| 6 | Claude Agent SDK (Python/TS) | https://docs.anthropic.com/en/docs/agents-and-tools/agent-sdk/overview |
| 7 | arXiv — OpenHands Software Agent SDK (2511.03690) | https://arxiv.org/html/2511.03690v1 |
| 8 | OpenHands (formerly OpenDevin) — ICLR 2025 | https://arxiv.org/abs/2407.16741 |
| 9 | ReAct pattern explainer — AI-TLDR.dev | https://ai-tl-dr.dev/learn/ai-agents/agent-fundamentals/react-agent-pattern/ |
| 10 | Agent Patterns (ReAct, plan-and-execute) — Agent Patterns ReadTheDocs | https://agent-patterns.readthedocs.io/en/latest/patterns/react.html |
| 11 | ReAct: Synergizing Reasoning and Acting (Yao et al.) | https://arxiv.org/abs/2210.03629 |

### 2.2 Definition of Done y descomposición

| # | Fuente | URL |
|---|--------|-----|
| 12 | Addy Osmani — Defining Done (agent skills) | https://github.com/addyosmani/agent-skills/docs/defining-done.md |
| 13 | Acceptance Criteria: todo lo que necesitas saber | https://www.iagocavalcante.com/acceptance-criteria |
| 14 | OpenAI — Atomic task decomposition | https://www.coveo.com/blog/atomic-task-decomposition/ (OpenAI cookbook referencia) |
| 15 | Addy Osmani — Task decomposition (agent skills) | https://github.com/addyosmani/agent-skills/docs/task-decomposition.md |
| 16 | Defining Done — AI Agentic Engineering Academy | https://www.aiengineeringacademy.com/blog/ai-agent-key-principles-defining-done |
| 17 | AIAA — Principles for Success in AI Agent Engineering (task earliness) | https://github.com/AIAA-Engineering/AIAA/blob/main/docs/Principles_for_Success_in_AI_Agent_Engineering.md |

### 2.3 Investigación y deep research

| # | Fuente | URL |
|---|--------|-----|
| 18 | Deep Research Bench (arXiv 2506.06287) | https://arxiv.org/abs/2506.06287 |
| 19 | Anthropic — Multi-agent research system | https://www.anthropic.com/engineering/multi-agent-research-system |
| 20 | Google — Deep Research agent (Gemini API) | https://ai.google.dev/gemini-api/docs/deep-research |
| 21 | Webcite — Deep research: cómo basar cada claim en evidencia | https://webcite.ai/blog/deep-research-verification |
| 22 | SkillsMP — scholar-deep-research skill (8 fases, saturación) | https://skillsmp.com/creators/agents365-ai/365-skills/plugins-scholar-deep-research |
| 23 | Anthropic — How to build a great research agent | https://www.anthropic.com/engineering/building-a-great-research-agent |
| 24 | Harvard Library — Evaluating web sources | https://library.harvard.edu/research-support/quick-research-guides/evaluating-sources |
| 25 | USC — SIFT method (Stop, Investigate, Find, Trace) | https://libguides.usc.edu/evaluatingwebsites |

### 2.4 Manejo de errores y resiliencia

| # | Fuente | URL |
|---|--------|-----|
| 26 | Error handling en agentic systems (retry, circuit breaker, dead-letter) | https://openhermit.org.github.io/ai-agent-engineering/error-handling.html |
| 27 | AI Agent Failures — Error Handling Patterns | https://aiagentfailures.substack.com/p/error-handling-in-ai-agents-part-i |
| 28 | Superwise — Error handling para agentic AI | https://superwise.ai/blog/error-handling-for-agentic-ai |
| 29 | Preporato — NCP-AAI error handling patterns | https://preporato.com/blog/error-handling-resilience-patterns-agentic-ai-systems |
| 30 | Classification of failures (transient/permanent/ambiguous) | https://learn.microsoft.com/en-us/azure/architecture/best-practices/transient-faults |

### 2.5 Fallos de agentes / loops halucinados / verificación

| # | Fuente | URL |
|---|--------|-----|
| 31 | Vectara — Awesome Agent Failures (taxonomía de modos de fallo) | https://github.com/vectara/awesome-agent-failures |
| 32 | WalkingLabs/harness-engineering — lecture 09: por qué los agentes declaran victoria pronto | https://github.com/walkinglabs/harness-engineering/blob/main/docs/en/lectures/lecture-09-why-agents-declare-victory-too-early/index.md |
| 33 | Understanding Code Agent Behaviour (arXiv 2511.00197) | https://arxiv.org/abs/2511.00197 |
| 34 | Self-correction: inseguro sin grounding externo (LLM-as-judge) | https://zylos.ai/en/research/2026-04-10-llm-as-judge-production-agent-verification-2026 |
| 35 | Hallucination mitigation (inconsistency detection, SIFT) | https://www.myengineeringpath.com/2025/10/hallucinations-mitigation-in-ai.html |

### 2.6 State / memoria / loops de ingeniería

| # | Fuente | URL |
|---|--------|-----|
| 36 | LangGraph — State machines, checkpointing, human-in-the-loop | https://langchain-ai.github.io/langgraph/concepts/low_level/ |
| 37 | Addy Osmani — Loop Engineering (ADDYOSMANI/agent-skills) | https://github.com/addyosmani/agent-skills/blob/main/docs/loop-engineering.md |
| 38 | Memory blindness / fallos silenciosos en agentes | https://www.brightwave.com/blog/2025/02/11/agent-memory-silent-failures |

### 2.7 Clarificación de requisitos

| # | Fuente | URL |
|---|--------|-----|
| 39 | Addy Osmani — Interview-me skill (una pregunta a la vez) | https://github.com/addyosmani/agent-skills/blob/main/skills/interview-me/SKILL.md |
| 40 | PromptForge — Deep research best practices (brief completeness) | https://promptforge.ai/blog/deep-research-best-practices |

---

## 3. El Loop de Trabajo del Agente

### 3.1 El patrón ReAct: la base de todo

Todo agente efectivo implementa una variante de **ReAct** (Reasoning + Acting). El loop canónico:

```
1. **Thought**  — razona sobre el estado actual y el objetivo
2. **Action**   — elige y ejecuta una herramienta (bash, read, edit, websearch, test)
3. **Observation** — interpreta la salida real del entorno (no suposiciones)
4. **Repeat**   — hasta cumplir el objetivo o el límite
```

- El **Observation debe venir del entorno**, nunca inventarse. Si el agente no recibe salida real de una herramienta, está alucinando el estado del mundo.
- **Iteración observable**: cada mini-loop genera un costo; el diseño del harness controla cuántas iteraciones se permiten (presupuesto de steps/tokens/costo).
- La **parada es parte del loop**: stop cuando (a) objetivo verificado, (b) límite de steps/costo alcanzado, (c) fallo irrecuperable diagnosticado, (d) se requiere intervención humana.

### 3.2 Variantes y composición

| Patrón | Descripción | Cuándo |
|--------|-------------|--------|
| **ReAct** | Thought→Action→Observation entrelazados | Tareas interactivas con herramientas |
| **Plan-and-execute** | Planificar todo primero, luego ejecutar | Tareas con plan claro y ejecución larga |
| **Reflexion** | Re-evaluar acciones pasadas en memoria verbal y corregir | Depuración iterativa |
| **Self-ask** | Preguntas intermedias para dividir el problema | Multi-hop QA |
| **Tree-of-thoughts** | Explorar múltiples ramas de razonamiento en paralelo | Búsqueda de soluciones alternativas |
| **Evaluator-optimizer** | Un LLM genera, otro juzga; loop hasta aprobar | Generación de texto/código con juicio externo |

### 3.3 El harness: el agente no decide todo solo

Los sistemas de producción separan **el modelo (agente)** del **harness** (infraestructura que lo rodea):

- **Herramientas**: bash, editor, búsqueda, tests — la fuente de ground truth.
- **Contexto**: el harness decide qué entra al contexto del modelo (qué archivos, qué logs, qué doc del repo).
- **Control de recursos**: presupuesto de pasos, tokens, tiempo; límites de comandos peligrosos.
- **Estado**: archivos, git, artifactos intermedios; la persistencia entre sesiones.
- **Verificación**: gates que corren después de cada acción (linters, tests) y bloquean si fallan.

> **Lección central (Anthropic)**: los agentes más confiables no dependen de "que el modelo recuerde". Dependen de un harness que les **de instrumentos reales** y **valide sus salidas**.

---

## 4. Clarificación de Requisitos (Antes de Ejecutar)

### 4.1 Por qué es crítica

La causa nº1 de agentes que "no hacen lo que se pidió" es ejecutar sobre una instrucción ambigua. Antes de escribir una sola línea de código o una sola búsqueda: **aclarar**.

### 4.2 Técnica *interview-me* (una pregunta a la vez)

- Hacer **una pregunta por turno**, la más bloqueante primero.
- No asumir nada que se pueda preguntar en una línea.
- Criterio de salida: ~95% de confianza en la intención real (para quién, por qué, cuándo, con qué recursos, qué éxito significa).
- Detonante: pedidos vagos ("arregla esto", "mejor déjalo bien", "optimiza"), build de features sin "para quién", o cuando el agente se sorprende rellenando requisitos en silencio.

### 4.3 Técnicas *spec* y *brainstorming*

- **Spec-driven**: escribir una especificación de 1 página (objetivo, alcance, no-objetivos, criterios de aceptación, riesgos) y que el usuario la confirme.
- **Brainstorming**: si los requisitos son creativos/ambiguos, explorar intención antes de diseñar la solución.
- **Reformulación (echoing)**: repetir al usuario lo entendido con tus propias palabras para atrapar malentendidos temprano.

### 4.4 Contrato de salida de la fase de clarificación

- Lista de requisitos **priorizada** (MVP vs nice-to-have).
- **Acceptance criteria** en formato verificable (ver §5).
- Restricciones explícitas: stack permitido, librerías, límites de presupuesto, tiempo.
- Acordar **Definition of Done**.

---

## 5. Definition of Done y Acceptance Criteria

### 5.1 La regla de oro: verificable por un tercero

> "Definition of Done ≠ Definition of Done según el agente." Una tarea está "done" cuando un **juicio externo** (otro agente, un script, el usuario) puede comprobar el criterio sin confiar en el autoreporte.

- **Mal**: "el código queda limpio", "la app se ve bien", "el informe es completo".
- **Bien**: "los `cargo test` pasan todos", "la respuesta HTTP es 200 en `/health`", "el hallazgo X cita la fuente [12]", "el archivo `CHANGELOG.md` contiene la entrada v1.3.0".

### 5.2 Formato de acceptance criteria

Cada criterio debe incluir:
1. **Acción o escenario detonante** (qué se hace para probarlo).
2. **Resultado esperado concreto y mensurable** (número, estado, path, salida).
3. **Condición negativa** (qué NO debe pasar).

### 5.3 Patrón "triple-uso" y números

- Los criterios se escriben para: guiar la ejecución, verificar al final, y servir de regresión futura.
- Cualquier criterio sin **métrica numérica** es sospechoso. Si no se puede poner número, falta refinamiento del requisito.

### 5.4 Tres niveles de *done* (AI Agentic Engineering Academy)

1. **Done del paso**: el paso atómico cumplió su check de pass/fail.
2. **Done de la tarea**: todos los pasos aprobados + criteria de aceptación cumplidos.
3. **Done del proyecto**: integración completa, verificación final, documentación y entrega.

(Equivalente a: subtarea → fase → entregable.)

---

## 6. Descomposición de Tareas (Task Decomposition)

### 6.1 Definición de paso atómico

Un paso atómico es la unidad de trabajo del agente: tiene **entrada**, **salida** y **check de pass/fail**. Si no se puede formular como "si ejecuto X y obtengo Y", no es atómico.

### 6.2 Backward chaining (encadenado inverso)

- Empezar desde el **objetivo final** y preguntar: "¿qué se necesita para lograrlo?"; repetir hacia atrás hasta tener pasos que ya se saben hacer.
- Garantiza que **ningún paso quede sin conexión al objetivo** (sin pasos órfanos de adorno).
- El resultado es un **DAG/dependencias**: pasos independientes en paralelo; pasos dependientes en secuencia.

### 6.3 Reglas de pasos buenos

- **Acotados** (bounded): un paso abarca una sola responsabilidad.
- **Testeables individualmente**: cada paso deja un artefacto verificable (archivo, salida, estado).
- **Ordenables**: se sabe cómo encadenarlos para llegar al objetivo.
- **Rollback-able**: si el paso falla, se identifica el checkpoint al que volver.

### 6.4 Plan de ≥3 pasos → escribir el plan

Un cambio que toca >1 archivo o tiene >3 pasos se escribe como **plan** (archivo o lista) antes de empezar: *writing-plans / planning-and-task-breakdown*. El plan se vuelve la lista de todos del agente.

---

## 7. Métodos de Investigación Correcta para Agentes

### 7.1 La investigación no es "buscar y copiar"

Es un **proceso iterativo de búsqueda con verificación**: formular query → sentir resultados → triage de relevancia → leer → **identificar gaps** → buscar de nuevo (más específico o más amplio).

### 7.2 Broadening vs Narrowing

- **Narrowing**: si hay suficiente relevancia, acotar la query (fechas, dominios, términos técnicos) para llegar a datos precisos.
- **Broadening**: si hay insuficientes resultados o poca relevancia, generalizar (otras fuentes, otros idiomas, sin filtros de fecha).
- La iteración de búsqueda se detiene por **saturación** (una ronda más no añade fuentes nuevas), no por agotamiento.

### 7.3 Triage de fuentes (evaluación en 4 pasos — Harvard + SIFT)

- **Stop** — pausa antes de creer.
- **Investigate** — ¿quién publica? ¿autoridad/conocimiento del tema? ¿afiliación?
- **Find** — buscar la cobertura original/better sourcing de la misma afirmación.
- **Trace** — rastrear claims hasta el **primary source**, no parar en agregadores o resúmenes de terceros.

Criterios concretos de calidad de fuente:
- Origen (papers, documentación oficial, sitios reputados vs blogs sin autor).
- Fecha (¿vigencia del contenido?).
- Corroboración (¿la misma afirmación en ≥2 fuentes independientes?).
- Precisión del claim (¿habla realmente de lo que dice?).
- Sesgo (¿editorializa donde no hay evidencia?).

### 7.4 Las citas son anclas, no decoración

- **Todo claim no trivial lleva `[id]` apuntando a su fuente.**
- Un claim sin cita = **sospechoso de alucinación**; se descarta o se marca como supuesto.
- En informes: cada hallazgo con cita + enlace; los supuestos propios se separan de la evidencia.

### 7.5 Workflow de investigación en 8 fases (scholar-deep-research)

```
Phase 0: Scope        → descomponer la pregunta, elegir arquetipo, inicializar estado
Phase 1: Discovery    → búsqueda multi-fuente, dedupe
Phase 2: Triage       → rankear por relevancia/calidad, top-N para lectura profunda
Phase 3: Deep read    → extraer evidencia por fuente
Phase 4: Chasing      → seguir el grafo de citas (forward + backward)
Phase 5: Synthesis    → agrupar por tema, mapear tensiones
Phase 6: Self-critique → revisión adversarial, encontrar gaps
Phase 7: Report       → entrega con citas y apéndice de crítica
```

### 7.6 Saturation como criterio de parada (no exhaustion)

- El criterio de parada correcto en investigación: **saturar la fuente**, no tocar todos los papers posibles.
- Regla práctica: una fase termina cuando una ronda nueva de búsqueda añade **<20% de fuentes nuevas** o ninguna fuente sobre umbral de citas/relevancia.
- Evaluar saturación **por fuente** (cada fuente consultada 1 sola vez indica gap de profundidad), nunca asumir global.

### 7.7 Multi-agente de investigación (patrón Anthropic)

- Un **agente líder de investigación** descompone la pregunta y delega en **subagentes de búsqueda con prompts específicos** (query, perspectiva, qué traer de vuelta).
- Cada subagente aporta: resumen + citas + profundidad; el líder sintetiza, detecta conflictos y **refina la pregunta**.
- Evaluación: **LLM-as-judge** con criterios explícitos (cobertura, precisión, balance, citas) en lugar de solo autoreporte.

### 7.8 Prevención de alucinaciones en research

- **Inconsistency detection**: comparar respuestas del modelo con el contenido retrieved; si el claim no está en la fuente, descartar.
- **SIFT antes de citar**: validación de fuente previa a usarla como evidencia.
- **Grounded generation**: forzar que el output cite IDs de los documentos realmente leídos.
- **Verificación de citas**: los webcrawlers del modelo resuelven los URLs citados; cita rota → evidencia inválida.

---

## 8. Manejo de Errores y Resiliencia

### 8.1 Clasificar el fallo primero

| Clase | Señal | Estrategia |
|-------|-------|------------|
| **Transitorio** | timeout, rate limit, 5xx, conexión | Retry con **backoff exponencial + jitter** (cap de intentos) |
| **Permanente** | 4xx lógico, validación falla, archivo no existe | No reintentar; diagnosticar y escalar |
| **Ambiguo** | error parcial, respuesta incompleta sin código claro | Asumir transitorio, reintentar tras comprobar el estado, escalar si persiste |

### 8.2 Patrones de resiliencia

- **Idempotencia**: las operaciones críticas (write, deploy, send) deben poder repetirse sin duplicar efecto. Es la base de todos los retry.
- **Circuit breaker**: tras N fallos consecutivos, cortar llamadas al recurso por un período, evitar cascada.
- **Dead-letter queue**: los mensajes/tareas que agotaron retries van a una cola de inspección humana, no se pierden.
- **Escalación humana (human-in-the-loop)**: gates de aprobación para acciones de alto impacto (borrar, publicar, pagar, semear datos masivos).
- **Fallback controlado**: si el camino A (herramienta/modelo) falla, usar camino B con degradación explícita (resultado parcial marcado como tal), nunca "resultado inventado".

### 8.3 Reporte honesto de fallos

- Si el agente falla: reportar **métricas exactas** (código de error, steps usados, qué data se verificó).
- Si el agente se corrige: **registrar el proceso de corrección** (que falló, cómo diagnosticó, qué cambió) — es la fuente de aprendizaje (Reflexion).
- **Nunca ocultar un fallo parcial bajo un "todo ok"**: la transparencia ES parte del contrato.

---

## 9. Fallos Clásicos de Agent-Loops (y su arreglo)

### 9.1 Taxonomía de quebraderos de loop (Vectara)

| Modo de fallo | Descripción | Arreglo |
|---------------|-------------|---------|
| **Parada temprana / victoria prematura** | Declara done sin verificar criterios | Verificación externa obligatoria antes de parar |
| **Loop infinito / parada tardía** | Repite sin progreso hasta agotar presupuesto | Presupuesto duro de steps; detección de "sin progreso" |
| **False completion** | Reporta que hizo X cuando no | Ground truth del entorno (read el resultado, no asumir) |
| **Reintento ciego** | Mismo fallo N veces sin diagnóstico | Clasificar fallo; cambiar de estrategia tras 1-2 reintentos |
| **Confabulación de salidas de herramientas** | Inventa la salida de un comando que no corrió | Harness que inyecta salidas reales; el agente nunca inventa la salida |

### 9.2 Por qué los agentes declaran victoria temprano (harness-engineering, lecture 09)

- Causas: **presión de presupuesto** (steps limitados incentivan parar), **autoreporte sin contraste**, criterios de done vagos, ausencia de verificación automática de salida.
- Arreglo: gates de verificación **antes** de permitir "done"; definir done con evidencia observable; permitir al agente gastar presupuesto extra si la verificación lo exige.

### 9.3 Hallar "sin progreso"

- **Heurística**: si el agente repite el mismo action con la misma salida ≥2 veces, no está progresando; el harness debe cortar y pedir replanteo.
- La observación es la señal: solo el entorno puede decir si hubo progreso.

---

## 10. Memoria, Estado y Continuidad (para tareas largas)

### 10.1 La memoria del agente no es infinita

- Los contextos tienen límite; una tarea larga requiere **persistir estado en artefactos**: archivos, planes, todo-lists, notas, git commits, checkpoints.
- **Checkpointing**: guardar estado tras cada fase para poder **reanudar** (no recomenzar) tras un fallo o nueva sesión.

### 10.2 Artefactos recomendados para tareas largas / multi-sesión

| Artefacto | Contenido |
|-----------|-----------|
| `PLAN.md` / plan file | Pasos, dependencias, estado (pending/in-progress/done) |
| `TASKS.md` / todo list | Lista de todos con checkboxes y criterio de done |
| `STATE.md` | Estado actual (dónde estoy, qué falta, qué se asume) |
| Notas de investigación | Claims + citas + gaps abiertos |
| `CHANGELOG.md` | Historial de cambios verificado (release-plz en este repo ver §AGENTS) |
| Log de decisiones (ADR) | Por qué se eligió cada opción relevante |

### 10.3 Memoria silenciosa / memoria-ciega

- **Memory blindness**: archivar demasiado agresivo esconde datos críticos incluso del propio agente; compensar con: todo-list visible, resúmenes por fase, y paging de cold-storage con índice buscable.
- **Fallo silencioso**: una operación de memoria que "funciona" pero guardó mal (truncó, sobrescribió) no lanza error; mitigar verificando que lo guardado = entrada (round-trip check).

---

## 11. Verificación: Cómo el Agente Sabe que Terminó

### 11.1 Auto-reporte vs ground truth

- La revisión-apropiación intrínseca del modelo ("revisa tu trabajo") sin contraste externo **degrada** la calidad (hallazgos de LLM-as-judge): el mismo modelo confirma su propio error.
- La verificación confiable exige **una fuente de verdad distinta al modelo**:
  1. **Tests deterministas** (unit/integration): la mejor fuente.
  2. **Salidas de herramientas** (compiler, curl, git status): corroboración inmediata.
  3. **Retrieval aumentado**: comparar claims contra los documentos de dónde salieron.
  4. **Juez LLM distinto** (evaluator-optmizer): para output libre, con rúbrica.
- Anclar siempre: "pasé los tests" se demuestra con el log del comando, no con la palabra.

### 11.2 Mínimo verificable por tarea

- Tras cualquier cambio: correr el **comando de lint/typecheck/tests** que exista en el repo (en VantaDB: `cargo fmt --check`, `cargo clippy`, `cargo test`; ver `.opencode/AGENTS.md`) y **reportar el output real**.
- En investigación: cada claim con su `[id]` resuelto; los URLs citados deben existir (el crawler los valida).

### 11.3 Gates de verificación en el harness

```
[accion del agente] → [gate] → [avanza]
   falla el gate → bloquea → el agente debe corregir y reintentar
```
Los gates son la diferencia entre "agente que dice que terminó" y "agente que terminó". 

---

## 12. Hábitos Tóxicos a Evitar (checklist del agente)

- [ ] **No inventar salidas** de comandos/herramientas que no se ejecutaron.
- [ ] **No saltarse la clarificación** por "ya sé qué quiere".
- [ ] **No declarar done sin verificar** contra los acceptance criteria.
- [ ] **No ignorar fallos** ni reportar "todo OK" cuando hubo fallo parcial.
- [ ] **No hacer un solo intento de búsqueda** y darlo por saturado.
- [ ] **No copiar sin citar** ni presentar supuestos propios como evidencia.
- [ ] **No reintentar en bucle** sin diagnóstico.
- [ ] **No dejar huérfanos los pasos**: cada paso conectado al objetivo.
- [ ] **No degradar el chequeo de errores** en paths de dinero/seguridad.
- [ ] **No gastar presupuesto infinito**; paradas explícitas.

---

## 13. Protocolo Paso a Paso para Ejecutar UNA Tarea Correctamente

> Aplica a cualquier tarea; para tareas de investigación ver el Protocolo de Investigación (§14).

### Fase 0 — Recepción y Clarificación
**Input**: solicitud del usuario.
**Acciones**:
1. Leer la solicitud completa; identificar requisitos explícitos, implícitos y faltantes.
2. Si hay ambigüedad bloqueante: **interview-me** (una pregunta a la vez, no asumir).
3. Confirmar restricciones (stack, tiempo, presupuesto, alcance).
4. Escribir para confirmación: objetivo + non-goals + acceptance criteria acordados.
**Output**: spec acordado con criteria de aceptación.

### Fase 1 — Planificación y Descomposición
**Input**: spec acordado (Fase 0).
**Acciones**:
1. Descomponer en **pasos atómicos** con entrada/salida/check (backward chaining).
2. Si >3 pasos o >1 archivo: escribir **PLAN** con dependencias y estado.
3. Definir el **Definition of Done** verificable de la tarea completa.
4. Definir presupuesto (steps/tiempo) y criterios de parada temprana.
**Output**: plan + todo list + criteria de done.

### Fase 2 — Ejecución con Loop ReAct
**Input**: plan (Fase 1).
**Acciones**:
1. Por cada paso: `Thought → Action → Observation` usando herramientas reales.
2. Registrar salida real de cada herramienta; nunca inventar.
3. Marcar el paso done solo si su **check de pass/fail** pasó (con evidencia).
4. Si un paso falla: clasificar → reintentar (backoff) → cambiar estrategia → escalar.
5. Verificar progreso: si dos intentos iguales sin cambio, replantear.
**Output**: pasos completados con evidencia + log de errores/correcciones.

### Fase 3 — Verificación de Done
**Input**: trabajo ejecutado (Fase 2).
**Acciones**:
1. Correr checks externos: lint/typecheck/tests; reportar output real.
2. Verificar **cada acceptance criterion** contra evidencia, no contra la palabra del modelo.
3. Corrección de errores detectados: volver a Fase 2 para el paso afectado.
4. **Solo entonces** declarar done (y solo si el criterio de done es observable).
**Output**: verificación aprobada + evidencia (logs, tests, diffs).

### Fase 4 — Documentación y Handoff
**Input**: tarea verificada (Fase 3).
**Acciones**:
1. Actualizar todos los artefactos: PLAN (done), STATE, changelog, notes.
2. Si hay supuestos/decision: registrar en ADR o notas.
3. Redactar resumen de entrega: qué se hizo, evidencia, qué quedó fuera, próximos pasos.
**Output**: entrega documentada y resumible.

### Fase 5 — Revisión Final (si aplica)
- Confirmar que nada se comprometió sin verificación; que los criteria de done se cumplen; y que la documentación permite a un tercero retomar el trabajo.

---

## 14. Protocolo de Investigación Correcta para Agentes

> Sub-proceso §13 para tareas de investigación (recopilar, sintetizar y citar conocimiento externo).

### Fase A — Scope y Descomposición de la Pregunta
1. Reformular la pregunta de investigación en **sub-preguntas** explícitas.
2. Elegir el **arquetipo de investigación** (exploratoria / comparativa / estado del arte / troubleshooting).
3. Definir **incluye/excluye** de fuentes y el alcance temporal.
4. Inicializar el **estado** (formato: pregunta, fuentes consultadas por tema, citas, gaps).
**Output**: plan de investigación + sub-preguntas + criterios de saturación.

### Fase B — Discovery iterativo con Broadening/Narrowing
1. Buscar en **múltiples fuentes paralelas** (websearch, webfetch, papers, repo docs).
2. Ronda 1: queries amplias → colapsar duplicados → triage por relevancia.
3. Si pocos resultados: **broadening** (generalizar, otros idiomas/fuentes).
4. Si suficientes: **narrowing** (fechas, dominios, términos específicos).
5. Iterar hasta **saturación** de la fuente en juego.
**Output**: pool de candidatos + log de búsquedas.

### Fase C — Triage y Lectura Profunda
1. Rankear fuentes por calidad (Harvard/SIFT: autoridad, fecha, corroboración).
2. Seleccionar **top-N** para lectura profunda; registrar descartes y por qué.
3. Leer de verdad (deep read), extrayendo claims concretos.
**Output**: top-N leído + notas de evidencia por fuente.

### Fase D — Chasing del grafo de citas
1. De cada fuente clave: seguir citas **backward** (referencias que usa) y **forward** (quien la cita).
2. Evaluar la nueva fuente igual (calidad, relevancia).
3. Cerrar cuando se satura el grafo (las referencias se repiten).
**Output**: grafo de citas cerrado + fuentes primarias alcanzadas.

### Fase E — Síntesis
1. Agrupar hallazgos por **tema** (no por fuente).
2. **Mapear tensiones**: dónde las fuentes discrepan; registrar ambas con citas.
3. Separar **evidencia citada** de **interpretación propia** (marcada como tal).
**Output**: síntesis por tema + tensiones explícitas.

### Fase F — Self-critique y Verificación de Citas
1. Revisión adversarial: ¿qué falta? ¿quién discrepa? ¿qué supuesto no está cubierto?
2. Verificar que **cada claim tiene `[id]`** a una fuente realmente consultada.
3. Resolver los URLs citados (citas rotas = evidencia inválida).
4. Reflexion: anotar qué se aprendió y qué falló en el proceso.
**Output**: borrador autocriticado + citas verificadas.

### Fase G — Reporte
1. Estructura: resumen ejecutivo → hallazgos por tema (con citas `[id]` + URLs) → tensiones → gaps → protocolo aplicado.
2. Entregar con **nota de incertidumbre** y límites.
3. Cumplir los criterios de aceptación de Scope (Fase A) antes de cerrar.
**Output**: informe final + archivo de estado actualizado (STATE).

---

## 15. Integración con el Repositorio (VantaDB)

Para ejecutar cualquier tarea en este repo, el agente debe operar bajo las reglas de `.opencode/AGENTS.md`:
- Uso de **CodeGraph** para explorar el código antes de editarlo.
- **Skills del lote de investigación** para generación de reportes.
- **Release Workflow Regla 7**: conventional commits (feat/fix/docs test/refactor/perf/ci/chore), nunca tocar versiones/tags manualmente (release-plz).
- Verificación de cadena de Rust: `cargo fmt --check`, `cargo clippy`, `cargo test` — y reportar el output real.
- Este documento es un **artifact de investigación (doc)**: los cambios derivados de él en código siguen `feat:`/`fix:` con PR a `develop`, no a `main`.

---

*Fin del documento. Generado por agent-02 con investigación basada en fuentes citadas en §2.*