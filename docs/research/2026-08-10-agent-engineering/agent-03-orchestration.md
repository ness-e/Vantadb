# Agente-03 — Orquestación y Colaboración Multi-Agente (Orquestador + Sub-agentes)

> **Investigación:** 2026-08-10-agent-engineering
> **Ángulo asignado:** orquestación y colaboración multi-agente — cómo un orquestador delega y controla sub-agentes correctamente
> **Alcance:** patrón orchestrator-workers, cuándo delegar, prompts de sub-agente, validación de outputs, recuperación, paralelismo, estado/durabilidad, roles, anti-patrones, evaluación
> **Método:** 15+ fuentes primarias consultadas vía websearch/webfetch/metasearchmcp/argus; 8 leídas completas
> **Idioma:** español; términos técnicos en inglés

---

## 1. Resumen ejecutivo

La orquestación multi-agente (patrón **orchestrator-workers**) es el mecanismo por el
cual un agente líder (orchestrator/lead) delega trabajo a sub-agentes (workers), consume
sus resultados y sintetiza la respuesta final. Es el patrón más caro de los agentic
systems y, cuando el problema lo justifica, el que más calidad aporta: los sub-agentes
son *scaling de test-time compute* con contexto aislado, no un truco de arquitectura.

Hallazgos más duros (Anthropic opera **Claude Research** multi-agente en producción;
OpenAI expone el patrón en su Agents SDK):

- Un sistema con **Opus 4 como lead y Sonnet 4 como sub-agentes superó al agente único
  Opus 4 en +90.2%** en el eval interno de investigación de Anthropic.
- **Tres factores explican 95% de la varianza** de calidad en BrowseComp: uso de tokens
  (80%), número de tool calls y elección de modelo. Los multi-agent "gastan los tokens
  suficientes": cada sub-agente suma capacidad de razonamiento en su propio contexto.
- Costo: agentes usan **~4× tokens** que chats; multi-agente **~15×**. Solo es viable
  cuando el valor de la tarea paga el sobrecosto.
- Contexto = recurso finito (**context rot / attention budget**). El sub-agente es la
  respuesta estructural al desbordamiento: contexto limpio por tarea, devuelve solo
  **1.000–2.000 tokens** condensados.
- Principal fuente de fallos de orquestación: **prompt de delegación pobre** → duplicación
  de trabajo, huecos, división de labor inexistente. Anthropic corrigió esto enseñando al
  orquestador a delegar y calibrando el esfuerzo a la complejidad de la query.

Este documento entrega el **contrato orquestador ↔ sub-agente** (§12) con el formato de
`RESULTADO` recomendado.

---

## 2. Patrón orchestrator-workers y evidencia de calidad

### 2.1 Qué es

Definición de referencia (Anthropic, "Building effective agents", dic 2024): un LLM
central **descompone dinámicamente** la tarea, delega subtareas a workers LLM y
**sintetiza** sus resultados. A diferencia de `parallelization`, los subtasks **no están
predefinidos**: los decide el orquestador según el input concreto (en coding, el nº de
archivos y el tipo de cambio dependen de la tarea).

```
INPUT → [Lead: planifica, descompone, asigna]
            ├─ subagent₁ (tarea A) ──► resumen condensado
            ├─ subagent₂ (tarea B) ──► resumen condensado
            └─ subagent₃ (tarea C) ──► resumen condensado
            ▼
       [Lead: valida, decide si más investigación, sintetiza] → OUTPUT
```

Fuente: <https://www.anthropic.com/engineering/building-effective-agents>

### 2.2 Evidencia de que los sub-agentes mejoran calidad

| Factor | Evidencia | Fuente |
|---|---|---|
| Mejora en eval de investigación | Multi-agente Opus4 + Sonnet4 vs agente único Opus4: **+90.2%** | Anthropic multi-agent research system |
| Varianza en BrowseComp | uso de tokens 80%; tool calls + modelo ≈ 95% combinado | Anthropic multi-agent research system |
| Test-time compute | sub-agentes = más compute en inferencia = mejor en tareas difíciles | arXiv 2506.12928 |
| Goal success enterprise | multi-agente mejora hasta **+70%** vs agente único; **90%** end-to-end | arXiv 2412.05449 (AWS Bedrock) |
| Payload referencing | **+23%** en tareas code-intensive (pasar referencias, no texto) | arXiv 2412.05449 |
| Paralelismo | lead con 3–5 sub-agentes en paralelo + sub-agentes con 3+ tools en paralelo → tiempo **−90%** | Anthropic multi-agent research system |

### 2.3 Por qué funcionan: *capacity* y separación de concerns

Argumento de Anthropic: "los sistemas multi-agente funcionan principalmente porque
ayudan a gastar suficientes tokens para resolver el problema". Cada sub-agente actúa
como **filtro inteligente**: explora con su propio context window y comprime lo más
importante para el lead. Aportan además **separation of concerns** — tools, prompts y
trayectorias de exploración independientes — lo que reduce path dependency y permite
investigaciones paralelas profundas.

Sistemas que comparten el mismo contexto o con muchas dependencias entre agentes **no
son buen fit** (p.ej. la mayoría de coding actual). El fit ideal: tareas de alto valor,
paralelizables, con info que excede context windows individuales, e interfaz con
múltiples tools.

---

## 3. Cuándo y cómo delegar

### 3.1 Cuándo SÍ delegar (delegation triggers)

1. **Contexto excedido** — la tarea supera el context window o degrada por context rot.
   El sub-agente trabaja con contexto limpio y devuelve resumen condensado. Mecanismo
   estructural recomendado por Anthropic para long-horizon tasks.
2. **Aislamiento de investigación** — múltiples direcciones independientes en paralelo
   (queries breadth-first): cada sub-agente explora un "beam" sin contaminar al lead
   ni a los demás.
3. **Especialización** — tools, prompts o modelos distintos por dominio (código, PDFs,
   web, finanzas). Recomendación explícita del SDK de OpenAI: *specialized agents over
   general-purpose agents*.
4. **Fidelidad / compresión** — outputs estructurados (código, reportes, visualizaciones)
   los produce mejor el sub-agente especializado que un coordinador general. Anthropic:
   así se evita el "game of telephone".
5. **Evaluación independiente** — evaluar con otro sub-agente crítico (LLM-as-judge,
   evaluator-optimizer, revisión de vulnerabilidades) en vez del mismo modelo.
6. **División de modelos/costo** — routing de tareas fáciles a modelos baratos y
   difíciles a modelos capaces (ej. Haiku vs Sonnet).

### 3.2 Cuándo NO delegar

| Señal | Motivo |
|---|---|
| La tarea cabe en 1 LLM call optimizado | Agrega latencia y costo sin ganancia. Regla general de Anthropic: empezar por lo más simple; solo añadir complejidad si mejora resultados |
| Dependencias densas entre sub-agentes | La coordinación real-time entre agentes aún es débil |
| Dominio donde todos comparten el mismo contexto | Estado global síncrono no encaja en el modelo de workers aislados |
| Subtareas predecibles y fijas | Usar workflow con código (prompt chaining / parallelization) en vez de orquesta autonómica |
| Contexto extensible con compaction o note-taking | Compaction preserva el flujo conversacional mejor que saltar a multi-agente |
| Valor económico bajo de la salida | ~15× tokens; si la salida no paga el costo, es mala inversión |

### 3.3 Topologías de delegación (OpenAI Agents SDK)

| Topología | Mecanismo | Cuándo |
|---|---|---|
| **Agents as tools** (`Agent.as_tool()`) | El manager conserva el control y llama especialistas como tools; los outputs vuelven al manager | Un agente posee la respuesta final, combina outputs de varios especialistas o aplica guardrails en un punto único |
| **Handoffs** | Un agente triage enruta y el especialista **se vuelve el agente activo** del turno | El routing es parte del flujo y el especialista debe responder con su propio prompt |

Regla práctica: *agents as tools* para subtareas acotadas que no deben tomar la
conversación; *handoffs* cuando el routing mismo es el trabajo. Combinables: un triage
puede hacer handoff a un especialista que a su vez llama otros agentes como tools.
Ambas se pueden ejecutar en paralelo vía código (`asyncio.gather` en Python).

Fuente: <https://openai.github.io/openai-agents-python/multi_agent/>

### 3.4 Modos de coordinación (AWS Bedrock / survey MAGNET)

- **Coordination mode**: comunicación inter-agentes + **payload referencing** (referencias
  a artefactos en lugar de copiar contenido) — clave en tareas code-intensive.
- **Routing mode**: forwarding de mensajes con **bypass del orquestador** cuando no
  hace falta orquestación → reduce latencia.
- MAGNET (survey de sistemas multi-agente LLM) estructura el workflow en 5 componentes:
  **profile, perception, self-action, mutual interaction, evolution** — cada agente tiene
  un perfil/rol, percibe, actúa, interactúa, y el sistema evoluciona.
  - <https://arxiv.org/abs/2412.05449> · <https://link.springer.com/article/10.1007/s44336-024-00009-2>

---

## 4. Redacción de prompts de sub-agente efectivos

Lección directa de Claude Research (Anthropic): "Teach the orchestrator how to delegate"
y "Scale effort to query complexity". El prompt del sub-agente es el **contrato de
tarea** que el orquestador construye al crear al worker.

### 4.1 Componentes mínimos que debe entregar el orquestador al crear el sub-agente

1. **Objetivo (objective)** — qué resultado concreto. Vago = duplicación o huecos.
   Ejemplo de fallo real de Anthropic: dos sub-agentes investigando la misma cadena de
   suministro 2025 mientras un tercero exploraba la crisis de chips 2021.
2. **Alcance / task boundaries** — qué cubrir y qué NO, para establecer division of
   labor explícita entre workers.
3. **Formato de salida requerido** — estructura del return (ver §5 y §12); crítica
   para la verificabilidad y la síntesis posterior.
4. **Tools y fuentes a usar** — qué tools/sources, en qué orden (primero breadth,
   luego depth), y preferencia de fuentes primarias sobre secundarias.
5. **Restricciones y esfuerzo** — reglas explícitas: nº máximo de tool calls, cuándo
   parar (stop conditions), presupuesto de esfuerzo según complejidad de la query.
6. **Contexto de entrada** — solo lo necesario: el objetivo, referencias a artefactos,
   seeds de búsqueda. NO regurgitar el contexto completo del orquestador.

### 4.2 Heurísticas de Anthropic para prompt de delegación

| Heurística | Práctica |
|---|---|
| Empezar ancho, luego estrecho | Queries cortas y broad primero; afinar progresivamente (evita la "tyranny of long specific queries") |
| Calibrar esfuerzo a complejidad | Fact-finding = 1 agente 3–10 tool calls; comparaciones = 2–4 sub-agentes 10–15 calls; investigación compleja = 10+ sub-agentes con alcances divididos |
| Pensamiento guiado (extended thinking) | El lead usa thinking para planificar, elegir tools, decidir nº de sub-agentes y definir cada rol; sub-agentes usan interleaved thinking tras tool results |
| Heurísticas, no reglas rígidas | Prompt frameworks de colaboración > instrucciones estrictas; mejoras con evals |
| Guardrails anti-spiral | Evitar que el agente "siga y siga": criterios de suficiencia para terminar |

### 4.3 Prioridad del contexto: contexto limpio > contexto masivo

Anthropic (context engineering): el objetivo es el **menor conjunto posible de tokens de
alta señal**. En delegación esto significa: el sub-agente recibe un contexto pequeño y
enfocado (lo que necesita para su subtarea), no el historial del orquestador. El
orquestador hace *just-in-time retrieval*: mantiene referencias ligeras (paths, queries,
links) y carga datos al contexto solo cuando hace falta.

Fuentes:
- <https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents>
- <https://www.anthropic.com/engineering/multi-agent-research-system>

## 5. Consumo y validación del output de sub-agentes

### 5.1 Estructura de retorno

El output del sub-agente debe ser **estructurado y consumible por el orquestador**, no
prosa libre. Recomendado (síntesis de Anthropic + OpenAI + CrewAI):

| Elemento | Por qué |
|---|---|
| Resultado sintetizado (1.000–2.000 tokens) | El sub-agente puede gastar decenas de miles explorando, pero al lead solo le llega la compresión |
| Formato declarado (JSON / secciones) | Facilita parseo, validación programática y parcial (structured outputs) |
| Referencias a artefactos en filesystem | Externos grandes (código, reportes) se guardan en archivos; el sub-agente pasa "reference" no copia — evita el *game of telephone* y ahorra tokens |
| Fuentes / evidencia de cada claim | Habilita verificabilidad y citación |
| Estado / confianza | Éxito, parcial, fallo; flag explícito cuando la tarea no se completó |

Regla de Anthropic: **"Subagent output to a filesystem"** — para outputs estructurados
(código, reportes, visualizaciones) el sub-agente escribe a un artifact system y pasa
referencias de vuelta al coordinador. Mejora fidelidad y reduce el token overhead de
copiar outputs grandes por el historial.

### 5.2 Verificabilidad

- **Evidencia por claim**: cada hallazgo con su fuente (URL / doc / tool result). El
  sistema de Research usa un **CitationAgent** dedicado que procesa documentos y
  resultados para localizar citas exactas.
- **Fact-check programático** cuando el eval tiene respuesta objetiva (LLM judge
  comprueba la respuesta correcta, p.ej. "lista las 3 farmas con mayor R&D").
- **End-state evaluation**: en agentes que mutan estado a lo largo de muchos turnos,
  juzgar el estado final correcto, no el proceso. Evaluación por checkpoints discretos
  para flujos complejos.
- **Trazabilidad de decisiones**: production tracing de patrones de decisión y estructura
  de interacción (sin leer el contenido de conversaciones, por privacidad).

### 5.3 Outputs vacíos / incompletos / fabricados

| Síntoma | Mitigación |
|---|---|
| Vacío (0 fuentes, nada encontrado) | Validar si el sub-agente buscó bien (query respiró? tools fallaron?); pedirle re-búsqueda con estrategia distinta; marcarlo `retry` |
| Incompleto (respuesta parcial) | Consumir lo parcial como progreso, no como definitivo; lanzar sub-agente complementario o ampliar alcance |
| Fabricado / hallucinado | Exigir fuentes por claim; LLM-as-judge chequea "¿las citas coinciden con las afirmaciones?"; human eval detecta sesgos no cubiertos por automatización (p.ej. preferir SEO-content-farms sobre fuentes autoritativas) |
| Inconsistente entre sub-agentes | Cruzar hallazgos; marcar conflictos para validación humana; devolver al worker con feedback específico |

Regla de oro de Anthropic para evals: empieza a medir **inmediatamente con muestras
pequeñas** (~20 queries reales) — con effect sizes 30%→80%, pocos casos bastan para ver
cambios. LLM-as-judge con un solo prompt y rúbrica (0.0–1.0 + pass/fail) es lo más
consistente. La review humana sigue siendo imprescindible (edge cases que los evals
pierden).

---

## 6. Recuperación de sub-agentes (resiliencia)

Anthropic es explícito: *"Agents are stateful and errors compound"*. Un fallo menor en
software tradicional es un bloqueo en un agente. **No reiniciar desde cero**: reinicios
son caros y frustrantes.

### 6.1 Estrategias en orden de barato → caro

1. **Retry determinista** — reintentar la tool call fallida con backoff; la mayoría de
   fallos de tool son transitorios (timeout, rate limit).
2. **Dejar que el agente se adapte** — informarle que la tool falló y que ajuste. "Dejar
   que el modelo maneje errores con gracia funciona sorprendentemente bien".
3. **Checkpoints y resume** — estado persistido; reanudar desde donde estaba el agente,
   no desde el inicio. Anthropic combina adaptabilidad del modelo + salvaguardas
   deterministas (retry logic + regular checkpoints).
4. **Fresh context / new sub-agent** — si el contexto se corrompió o agotó, spawnear un
   sub-agente con contexto limpio, pasándole una nota/plan de estado (handoff con
   resumen) para mantener continuidad.
5. **Cambiar estrategia** — si el sub-agente insiste en un camino sin éxito, el lead lo
   redirige: nueva estrategia de búsqueda, otras tools, otro modelo.
6. **Escalar a humano (human-in-the-loop)** — checkpoints para confirmación, o handoff
   al humano cuando hay bloqueos, ambigüedad de alto riesgo o desacuerdo entre
   sub-agentes. AutoGen lo modela como modo conversable con humanos; crewAI deja la
   delegación explícita (delegation disabled by default).

### 6.2 Protección contra bucles y espirales

- **Stop conditions**: límites de iteraciones, max tool calls por query, criterios de
  suficiencia ("¿ya tengo los datos? → terminá").
- Early ants: sub-agentes que spawn-eaban **50 sub-agentes** para queries simples,
  buscaban fuentes inexistentes, o se distraían con updates excesivos. Se corrigió con
  prompt engineering (esfuerzo calibrado) y guardrails.

Fuentes: <https://www.anthropic.com/engineering/multi-agent-research-system> ·
<https://arxiv.org/abs/2308.08155> (AutoGen)

---

## 7. Coordinación y paralelismo

### 7.1 Patrones topológicos

| Patrón | Comunicación | Ejemplo |
|---|---|---|
| Supervisor (single lead) | Lead centraliza routing; workers no se hablan | Anthropic Research, CrewAI hierarchical, supervisor de LangGraph |
| Swarm (peer-to-peer handoff) | Agentes de igual nivel se pasan el control con handoffs | OpenAI Swarm / handoffs del SDK |
| Network (multiparty graph) | Grafos de agentes con enrutado por estado | LangGraph StateGraph multi-supervisor |

Tradeoff supervisor vs swarm (focused.io): supervisor = mejor enrutado/accuracy, menor
latencia por hot-path de decisión única pero cuello de botella central; swarm = más
flexible y fresco por turno, peor cuando el estado global debe persistir. La decisión
depende de si el routing debe ser global (supervisor) o local por turno (swarm).

### 7.2 Fork / join y waves

- **Fork**: el lead descompone y despacha sub-agentes (en paralelo idealmente).
- **Join**: espera resultados; decide si más investigación (nueva wave de sub-agentes,
  refinar estrategia) o sintetiza y sale del loop.
- **Waves**: en Research, el lead "ejecuta síncronamente: espera cada conjunto de
  sub-agentes antes de continuar. Crea **cuellos de botella** (no puede dirigir a
  sub-agentes en vuelo, se bloquea por el más lento). Asincronía permitiría más
  paralelismo pero a cambio de problemas de coordinación de resultados, consistencia de
  estado y propagación de errores.
- **DAG de dependencias**: cuando las subtareas no son independientes, orquestar por
  grafo (LangGraph). Workers con dependencias no deben ir en el mismo wave.

### 7.3 Límites de concurrencia

- Antes de cualquier orquestación multi-agente: revisar rate limits del proveedor,
  presupuesto de tokens y costo. crewAI permite `max_rpm` y `max_iterations` por crew.
- **Merge de resultados**: el lead debe consolidar outputs de distinta calidad/granularidad;
  cruzar solapamientos (duplicación) y unir huecos. La síntesis es trabajo del lead,
  no del user.

### 7.4 Dos niveles de paralelismo (Anthropic)

1. Lead spawn-ea **3–5 sub-agentes en paralelo** (no serial).
2. Cada sub-agente hace **3+ tool calls en paralelo**.
Junto sumaron ~90% de reducción en tiempo de investigación para queries complejas.

---

## 8. Estado y durabilidad entre sub-agentes

| Mecanismo | Descripción | Fuente |
|---|---|---|
| **Artefactos en filesystem** | Sub-agentes escriben outputs a archivos persistentes y pasan referencias (path) al lead; evita re-copiar contenido | Anthropic multi-agent |
| **Memoria estructurada (note-taking)** | El agente mantiene notas/plan persistidos fuera del contexto (NOTES.md, plan en memory); se releen tras resets de contexto | Anthropic context engineering |
| **Compaction** | Cuando el contexto se acerca al límite, resumir historial y continuar con el resumen + últimos archivos; preserva decisiones arquitectónicas, bugs pendientes, detalles | Anthropic context engineering |
| **Handoff con estado** | Antes de pasar la tarea, escribir resumen de estado + pendientes; el nuevo agente continua con continuidad | Anthropic / LangGraph |
| **Correlation / trace IDs** | Trazas de decisión y estructura de interacción para debugging de no-determinismo; monitorear patrones, no contenido | Anthropic multi-agent |
| **Sesiones durables** | Persistencia de conversación/estado entre runs (checkpointing, sessions en SDK de OpenAI, LangGraph persistence) | OpenAI / LangGraph |
| **Rainbow deploys** | No actualizar todos los agentes a la vez: siempre hay agentes a mitad del flujo; migrar tráfico gradualmente manteniendo versiones viejas | Anthropic multi-agent |

La carga de trabajo larga de más de un turno requiere que el plan vivo esté en memoria
externa: si el context window supera 200K y se trunca, el plan se conserva y se
recupera. Los sub-agentes con contexto fresco hacen handoff con el resumen del estado.

Fuentes:
- <https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents>
- <https://www.anthropic.com/engineering/multi-agent-research-system>
- <https://docs.langchain.com/> (LangGraph: state + persistence)

## 9. Roles y especialización

### 9.1 Cómo se asignan roles

En la práctica, los frameworks definen el rol de cada agente con un **perfil explícito**:
- **CrewAI / AutoGen**: `role`, `goal`, `backstory` (CrewAI) o system-message de agente
  (AutoGen). El rol define tools disponibles, estilo y responsabilidades.
- **Anthropic Research**: la división es por función — `LeadResearcher` (planifica,
  descompone, sintetiza), `Subagents` de investigación acotada, `CitationAgent`
  (ubicar citas exactas). Roles derivados de la estrategia de la tarea, no fijos.
- **MAGNET**: cada agente tiene un *profile* (identidad/rol) que condiciona su
  perception y self-action.
- **LangGraph**: roles son grafos/supervisores definidos por estado y tools.

### 9.2 Qué capabilities debe tener cada rol

| Rol | Capabilities mínimas | Capabilities a NO darle |
|---|---|---|
| Orquestador/lead | Tools de investigación broad; escritura de artefactos; planificación; delegación; evaluación de suficiencia | Tools destructivas del sub-agente (escribir en el dominio del worker); contexto saturado de detalles innecesarios |
| Worker/sub-agente | Solo las tools de su dominio; prompts especializados; formato de salida riguroso | Tools de otros workers (evita pisar alcances); arms de escalada (a veces necesarios para reportar fallo) |
| Evaluador/judge | Rúbrica, criterios, capacidad de verificación | Posición de voto en su propia tarea |
| Citador | Tools de asignación de citas a documentos | — |

### 9.3 Principio para dividir el trabajo

- **Frontera por información**, no por texto: separar dominios que no comparten estado.
- **Un worker = un alcance**, boundaries explícitas contra overlap (division of labor).
- **Specialized agents > general-purpose** (OpenAI SDK): tener sub-agentes que destacan
  en una tarea en vez de uno todo-terreno.

---

## 10. Anti-patrones de orquestación

| # | Anti-patrón | Síntoma | Corrección |
|---|---|---|---|
| 1 | **Delegación vaga** | Workers duplican trabajo o dejan huecos (caso real 2021/2025 chips) | Prompt de delegación con objective + boundaries + output format |
| 2 | **Esfuerzo descalibrado** | 50 sub-agentes para una query trivial; research infinita | Escalas de esfuerzo embebidas en prompt (1 agente 3–10 calls → 10+ para complejas) |
| 3 | **Dilución de contexto del lead** | Lead recibe decenas de miles de tokens por sub-agente | Condensación 1–2K tokens por worker; referencias a artefactos; no re-copiar |
| 4 | **Game of telephone** | Output se degrada al pasar por varios agentes | Sub-agent output a filesystem; pasar referencias |
| 5 | **Orquestador sin síntesis** | El lead junta outputs sin cruzar, sin verificar, sin detectar solapamientos | Merge explícito con detección de duplicados y huecos |
| 6 | **Sub-agentes que no verifican** | Citan afirmaciones sin evidencia; nudges de autoridad no comprobados | Evidencia por claim + CitationAgent + LLM-as-judge |
| 7 | **Orquestador que pierde el hilo** | Contexto truncado → plan olvidado → deriva | Plan en memoria externa; handoff de estado; compaction |
| 8 | **Sin stop conditions** | Bucles infinitos, gasto descontrolado | max iterations, max tool calls, criterios de suficiencia |
| 9 | **OCR de abstracciones / frameworks** | Depender de framework sin conocer el código debajo | Frameworks para empezar; entender prompts y respuestas subyacentes |
| 10 | **Over-delegación** | Delegar lo que un solo call resuelve | Regla: solo añadir complejidad si mejora resultados medidos |
| 11 | **Sub-agentes que mutan estado no rastreado** | Estados corrompidos entre waves | Checkpoints + end-state eval + correlación de traces |
| 12 | **Salida no estructurada** | Lead no puede parsear ni validar la respuesta del worker | Formato de RESULTADO obligatorio (§12) |

Los anti-patrones 1–3 son los que Anthropic documentó explícitamente en sus errores
iniciales de Research y corrigió vía prompt engineering.

---

## 11. Evaluación de sistemas multi-agente

- **No evaluar pasos, evaluar resultados**: multi-agent es no-determinista; distintos
  paths válidos llegan al mismo resultado. Evaluar outcome + proceso razonable, no seguir
  pasos prefijados.
- **Evals que importan** (rúbrica de Anthropic): factual accuracy, citation accuracy,
  completeness, source quality, tool efficiency.
- **LLM-as-judge**: un solo prompt que devuelve scores 0.0–1.0 y pass/fail = más
  consistente que múltiples judges. Compatible con check de respuesta correcta cuando
  hay ground truth.
- **Human eval**: detecta lo que los evals automatizados pierden (hallucination en
  queries raras, sesgo de selección de fuentes, fallos de sistema).
- **Empezar chico ya mismo**: 20 casos de uso reales << “800 casos”. Con effect sizes
  grandes, pocos casos revelan regresiones.
- **Observabilidad**: tracing de decisiones y patrones de interacción (sin mirar el
  contenido de chats) para diagnosticar "¿por qué el agente no encontró lo obvio?". En
  sistemas agénticos los errores **se compounen**: un paso que falla envía al agente a
  otra trayectoria.

---

## 12. Contrato orquestador ↔ sub-agente

Formato recomendado para operar orquestación con sub-agentes de forma verificable.

### 12.1 Deberes del orquestador (al delegar)

1. Definir el **objetivo** de la subtarea en 1–2 frases, sin ambigüedad.
2. Entregar **contexto mínimo de entrada**: objetivo + referencias (paths/queries/links) —
   jamás volcar todo su contexto.
3. Fijar **alcance y boundaries**: qué queda fuera; a qué no debe tocar.
4. Especificar **formato de RESULTADO** exigido (abajo).
5. Señalar **tools y fuentes** preferidas y heurísticas (breadth→depth, fuentes primarias).
6. Fijar **restricciones de esfuerzo**: max tool calls, stop conditions, presupuesto.
7. Definir **criterio de éxito** para que el worker sepa cuándo parar.
8. Proveer **ruta de artefactos**: dónde debe persistir outputs grandes en filesystem.
9. Establecer **escalada**: qué hacer ante fallo/bloqueo (retry → adaptar → reportar).

### 12.2 Deberes del sub-agente (al responder)

1. Cumplir el formato de RESULTADO al pie de la letra.
2. **Evidencia por claim** (fuente/URL/tool result para cada hallazgo).
3. Reportar **estado real**: éxito, parcial, fallo — sin maquillar trabajo incompleto.
4. Si no encontró nada: distinguir "no existe/busqué bien" de "no pude buscar".
5. Persistir outputs grandes en filesystem y devolver solo la referencia + resumen.
6. No salir del alcance asignado; si el alcance es insuficiente, reportarlo (no
   expandirlo unilateralmente).
7. Reportar supuestos, incertidumbre y fuentes no verificadas de forma explícita.

### 12.3 Plantilla de `RESULTADO` recomendada

```text
## RESULTADO
- status: OK | PARTIAL | FAILED | RETRY        # estado real, nunca fabricado
- objective: <echo de la subtarea asignada>    # autocontenido, verificable
- resumen: <1.000–2.000 tokens max>            # síntesis, no dump de contexto
- hallazgos:
  - claim: <afirmación concreta>
    evidencia: <URL | file path | tool result> # obligatoria por claim
    confianza: alta | media | baja             # con razón de la incertidumbre
- artefactos:
  - <path a output persistido en filesystem>   # outputs grandes NO en el mensaje
- fuentes_consultadas: <n>  primarias: <n>  secundarias: <n>
- pendiente_adicional: <lo que el orquestador debería delegar a otro worker o validar>
- advertencias:
  - <supuesto asumido>
  - <hecho no verificado>
  - <indicación de que el scope era insuficiente, si aplica>
```

El orquestador, al recibir esto: valida `status` y `objective`; chequea que cada
`claim` tenga `evidencia`; cruza `hallazgos` entre workers (duplicados → merge, huecos →
nuevo worker); y decide `ITERATE (otra wave) | SYNTHESIZE | ESCALAR_A_HUMANO`.

### 12.4 Contrato para el fallo (qué hace cada lado si algo falla)

| Escenario | Orquestador | Sub-agente |
|---|---|---|
| Tool call falla | Espera/retry; manda a adaptar | Reporta el error, prueba alternativa |
| Output vacío | Replanifica búsqueda; reenvía con otra estrategia | Explica qué buscó y qué bloqueó el resultado |
| Output parcial | Consume lo parcial + lanza complementario | Marca `PARTIAL` con lo cubierto y lo que falta |
| Sospecha de fabricación | LLM-as-judge sobre evidencia; humano si persiste | — |
| Bucle/alcance | Termina, redirige, escala | Respeta stop conditions y reporta suficiencia |

---

## 13. Fuentes

1. Anthropic — *Building effective agents* (dic 2024): <https://www.anthropic.com/engineering/building-effective-agents>
2. Anthropic — *How we built our multi-agent research system* (jun 2025): <https://www.anthropic.com/engineering/multi-agent-research-system>
3. Anthropic — *Effective context engineering for AI agents* (sep 2025): <https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents>
4. OpenAI Agents SDK — *Agent orchestration*: <https://openai.github.io/openai-agents-python/multi_agent/>
5. OpenAI Agents SDK (Python) — GitHub: <https://github.com/openai/openai-agents-python>
6. OpenAI — *ChatGPT Agent System Card* (deep research y operator combinados): <https://deploymentsafety.openai.com/chatgpt-agent/introduction>
7. AWS (Wu et al.) — *Towards Effective GenAI Multi-Agent Collaboration: Design and Evaluation for Enterprise Applications* (arXiv 2412.05449): <https://arxiv.org/abs/2412.05449>
8. Li et al. — *A survey on LLM-based multi-agent systems: workflow, infrastructure, and challenges* (MAGNET, Springer 2024): <https://link.springer.com/article/10.1007/s44336-024-00009-2>
9. Wu et al. — *AutoGen: Enabling Next-Gen LLM Applications via Multi-Agent Conversation* (arXiv 2308.08155): <https://arxiv.org/abs/2308.08155>
10. CrewAI — *Hierarchical Process*: <https://docs.crewai.com/en/learn/hierarchical-process>
11. CrewAI — *Crews* (manager agent, process hierarchical): <https://docs.crewai.com/concepts/crews>
12. LangChain — LangGraph multi-agent (supervisor, handoffs, state): <https://docs.langchain.com/> · <https://github.com/langchain-ai/langgraph>
13. DeepWiki/LangGraph-101 — *Multi-Agent Patterns* (supervisor vs handoff descentralizado): <https://deepwiki.com/langchain-ai/langgraph-101/6-multi-agent-patterns>
14. focused.io — *Multi-Agent Orchestration in LangGraph: Supervisor vs Swarm, Tradeoffs*: <https://focused.io/lab/multi-agent-orchestration-in-langgraph-supervisor-vs-swarm-tradeoffs-and-architecture>
15. tutorialslogic — *LangGraph Multi-Agent Graphs: Supervisor, Specialist, Handoff*: <https://www.tutorialslogic.com/langgraph/multi-agent-graphs>
16. machinelearningplus — *LangGraph Multi-Agent: Supervisor, Swarm & Network*: <https://machinelearningplus.com/gen-ai/langgraph-multi-agent-systems-supervisor-swarm-network/>
17. *Scaling Test-time Compute for LLM Agents* (arXiv 2506.12928): <https://arxiv.org/abs/2506.12928>
18. turion.ai — *Framework Deep Dive: CrewAI — Role-Based Multi-Agent Orchestration*: <https://turion.ai/blog/framework-deep-dive-crewai/>
19. Anthropic Cookbook — patrones multi-agente (prompts del sistema Research): <https://platform.claude.com/cookbook/patterns-agents-basic-workflows>

---

## 14. Conclusión aplicada (auto-lección del investigador)

- **Delegar ≠ descargar**: delegar sin objective, boundaries y formato de salida es la
  principal fuente de ruido. Cada sub-agente que lanzo debe nacer con el contrato §12.
- **El contexto es el recurso**: mi rol como orquestador es comprimir; los 1–2K tokens
  de resultado por worker son la unidad de moneda, no el dump.
- **Verificar, no confiar**: evidencia por claim y estados reales (`PARTIAL`/`FAILED`)
  protegen la síntesis de la fabricación.
- **Paralelizar con límites**: waves de 3–5, tool calls concurrentes, stop conditions;
  asíncrono solo cuando Merge/estado estén resueltos.
- **Recuperación barata**: checkpoints + resume + fresh-context handoff antes que
  reiniciar; humano al final de la escalera, no al principio.