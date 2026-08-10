# ENG-02 · Ingeniería de sistemas, resolución de problemas complejos y debugging sistemático

- **Fecha:** 2026-08-10
- **Ámbito:** Investigación web (15+ fuentes) sobre pensamiento sistémico, metodologías de resolución de problemas, debugging sistemático, SRE, observabilidad, ingeniería de requisitos, trade-offs y quick-fix vs root-fix.
- **Propósito:** Base de conocimiento para agentes (y humanos) que deben diagnosticar fallos, diseñar soluciones y comunicar decisiones complejas de forma estructurada.
- **Idioma:** Español; términos técnicos en inglés (field terms).

---

## 1. Pensamiento sistémico (Systems Thinking)

### 1.1 Fundamentos (Donella Meadows)
El pensamiento sistémico trata el sistema como un todo interconectado, no como partes aisladas. Fuente: Meadows / *Thinking in Systems* (PDF de referencia pública).

Conceptos núcleo:

| Concepto (EN)        | Traducción / definición                                             |
|----------------------|---------------------------------------------------------------------|
| Element / node       | Componente básico (persona, servicio, dato).                         |
| Feedback loop        | Bucle causal: refuerzo (amplifica) o balance (estabiliza).           |
| Delay               | Retardo entre acción y efecto — causa de oscilaciones.               |
| Stock & flow         | Inventarios (stocks) y flujos de entrada/salida.                     |
| Leverage point       | Punto de apalancamiento: el lugar donde un pequeño cambio genera gran efecto. |
| Emergent behavior    | Comportamiento del todo que no existe en ninguna parte individual.   |

Reglas prácticas (Meadows):
1. Análisis del sistema completo — nadie tiene la "máquina" completa en la cabeza; el sistema es mayor que la suma de sus partes.
2. Los **retardos** explican por qué las intervenciones "no funcionan": el efecto llega tarde, no está ausente.
3. Los **leverage points** suelen estar en la estructura (reglas del juego), no en los parámetros.
4. El sistema que produce sus propios problemas no se arregla solo cambiando las piezas; a veces hay que cambiar la estructura o el objetivo.

Fuentes:
- BMF Tech, "Systems Thinking basics" — https://bmf-tech.com/posts/systems-thinking-basics/
- Meadows, *Thinking in Systems* (2008) — https://research.fit.edu/.../Meadows-2008.-Thinking-in-Systems.pdf

### 1.2 Marco Cynefin (David Snowden)
Cynefin clasifica los contextos para elegir la estrategia correcta. Usado en decisión y troubleshooting.

Cuadrantes:

| Contexto   | Característica                  | Estrategia dominante      | Riesgo típico                |
|-----------|---------------------------------|---------------------------|------------------------------|
| Clear      | Causa-efecto obvios            | *Best practice*: Sense → Categorize → Respond | Exceso de simplificación |
| Complicated| Causa-efecto explicables (experto) | *Good practice*: Sense → Analyze → Respond  | Análisis-parálisis        |
| Complex    | Causa-efecto solo en retrospectiva | *Emergent*: Probe → Sense → Respond       | Forzar plan rígido        |
| Chaotic    | Sin relación causal discernible  | *Novel*: Act → Sense → Respond             | Paralización / panik      |
| (Confusion) | No sabemos en qué cuadrante estamos | Entrar a uno de los anteriores por prueba | Saltar a favoritos       |

Aplicación al debugging: identificar si el fallo es *Complicated* (hipótesis y análisis) o *Complex* (experimentación incremental: prove → sense → respond). Intentar técnicas de "Complicated" en un problema "Complex" genera parálisis y fixes equivocados.

Fuente: Toolshero, "Cynefin Framework" — https://www.toolshero.com/decision-making/cynefin-framework/

### 1.3 First principles thinking
Descomponer el problema hasta sus verdades fundamentales y construir desde ahí, en vez de razonar por analogía ("siempre lo hicimos así").

- **Analogía** → rápida, hereda supuestos y sesgos.
- **First principles** → caro pero descubre supuestos falsos.
- **Elon Musk method**: (1) identificar supuestos, (2) descomponer en elementos básicos, (3) reconstruir desde cero.
- **Beneficio en bugs**: separar "esto debe ser así" de "esto solo es así por legado". Los fixes "por analogía" replican el error.

Fuente: BreakDecisions, "First Principles vs Analogies" — https://breakdecisions.com/articles/first-principles-vs-analogies

---

## 2. Metodologías estructuradas de resolución de problemas

### 2.1 Marco genérico de 7 pasos (ingeniería)
Fuentes: LibreTexts (Seven-Step Design Framework), StructX (7-Step Problem Solving), ASEE (Seven C's).

Pasos universales:
1. **Define the problem** — declaración de problema observable, medible, sin presuponer causa.
2. **Gather information** — datos, logs, contexto, historia (constraints y requirements).
3. **Generate alternatives** — múltiples soluciones candidatas; evitar la primera idea.
4. **Evaluate / analyze** — trade-offs, riesgos, coste, factibilidad.
5. **Select & plan** — elegir y escribir el plan de implementación.
6. **Implement** — ejecutar el cambio; una sola variable a la vez.
7. **Evaluate / verify** — comprobar el resultado contra la definición inicial; iterar.

Regla de oro: **saltarse el paso 1 (define)** es la causa #1 de solucionar el problema equivocado. En debugging, "el problema" es la divergencia entre comportamiento esperado y observado, no el síntoma.

Fuentes:
- LibreTexts — https://eng.libretexts.org/.../7.03%3A_Seven-Step_Design_Framework
- StructX — https://www.structx.com/Article002-7_Step_Problem_Solving.html
- ASEE, "The Seven C's of Solving Engineering Problems" — https://peer.asee.org/board-96-the-seven-c-s-of-solving-engineering-problems.pdf

### 2.2 DMAIC (Six Sigma)
Marco de mejora de procesos de calidad, usado también para resolver problemas recurrentes:
- **Define** — problema, alcance, objetivos, customers.
- **Measure** — métricas de línea base (baseline).
- **Analyze** — identificar causas raíz con datos.
- **Improve** — implementar soluciones validadas.
- **Control** — sostener: monitoreo, estandarización, control de regresión.

Aplicación: cuando un fallo reaparece o un sistema degrada sistemáticamente, DMAIC obliga a medir antes de "mejorar". El paso *Control* es el que falta en la mayoría de fixes: sin monitor, el bug vuelve.

Fuente: PMC, "DMAIC" — https://pmc.ncbi.nlm.nih.gov/articles/PMC10229001

### 2.3 Kepner-Tregoe (problem analysis)
Método de diagnóstico de desviaciones usado en plantas y TI:
- **Desviación** = lo que debería ser vs lo que es.
- Preguntas distintivas (IS / IS NOT):
  - ¿Qué objeto/entidad se desvía? → **What**
  - ¿Dónde se observa exactamente? → **Where**
  - ¿Cuándo empezó / cuándo no? → **When**
  - ¿Hasta qué punto / extensión? → **Extent**
- Comparar el espacio IS vs IS NOT para aislar **factores distintivos** (unique characteristics), que son los candidatos a causa.
- Luego: posibles causes → verificar contra datos → causa más probable.

Este método es la base formal del "bisect mental" de localización de fallos en sistemas distribuidos.

### 2.4 MECE + Issue Trees (McKinsey)
- **MECE** = Mutually Exclusive, Collectively Exhaustive. Particiones del problema sin solapamiento ni huecos.
- **Issue tree / hypothesis tree**: descomposición del problema en ramas, cada rama probada con datos.
- **Hypothesis-driven problem solving**: 70% del tiempo en las ramas más probables; el análisis confirma o descarta hipótesis, no recopila todo por recopilar.
- Errores comunes: ramas no MECE (solapadas), profundidad asimétrica, confundir causa con efecto.

Aplicación: construir un árbol de hipótesis del fallo (¿dónde puede estar? red, código, datos, config, deploy) y descartar ramas por evidencia.

Fuente: Deckary, "McKinsey problem solving frameworks" — https://deckary.com/blog/problem-solving-frameworks

---

## 3. Debugging sistemático

### 3.1 Mentalidad (Debugging Mindset) y metodología de 4 fases
Fuentes: Devon O'Dell (talks de Google) y skill de referencia *Systematic Debugging* (obra/superpowers).

Mentalidad:
- Fixed vs growth mindset aplicado al debugging: los errores son señal de un hueco de información, no de fracaso personal.
- El bug es un rompecabezas: mantener calma, curiosidad, y sistema; evitar "shotgun debugging" (cambiar cosas al azar).
- Debugging = método científico aplicado: reproducir → observar → hipótesis → predecir → testear.

Fases (referencia Systematic Debugging):
1. **Understand the bug** (Phase 1):
   - Leer el mensaje de error completo.
   - Reproducir de forma **determinística** (loop de repro rojo).
   - Identificar cambios recientes (git, deploys).
   - Reunir evidencia: logs, estado, flujo de datos.
   - Aislar a un componente / tramo de código.
   - Articular hipótesis de causa raíz **antes** de tocar nada.
   - NO pasar a la fase 2 sin entender el porqué.
2. **Pattern analysis** (Phase 2):
   - Minimizar la reproducción: quitar inputs/config/pasos UNO A UNO hasta el repro mínimo (load-bearing elements).
   - Buscar ejemplos que *funcionan* similares al código roto (delta positivo).
   - Diferenciar por límites: separar inputs, timeouts, edge cases.
3. **Fix & verify** (Phase 3):
   - Escribir un test de regresión que falle primero (RED), una vez que existe repro mínimo.
   - Implementar UN único fix, sin refactors "de paso".
   - Verificar: test específico pasa + suite completa sin regresiones.
4. **Root cause / reflect** (Phase 4):
   - **Rule of Three**: si 3 fixes fallan, STOP → cuestionar la arquitectura. Cada fix que revela acoplamiento nuevo en un sitio distinto = señal de problema estructural.
   - Preguntarse qué permite que este bug exista (tests ausentes, validación, observabilidad).

Tabla: racionalizaciones comunes vs realidad

| Excusa                                       | Realidad                                                    |
|----------------------------------------------|-------------------------------------------------------------|
| "Es simple, no necesito proceso"             | Simple no significa sin causa raíz; el proceso es rápido.  |
| "Emergencia, no hay tiempo"                  | El método sistemático es MÁS rápido que el ensayo-error.   |
| "Pruebo esto primero y luego investigo"      | El primer fix fija el patrón; hazlo bien desde el inicio.  |
| "El test lo escribo cuando funcione"         | Un fix sin test no se sostiene.                            |
| "Varios fixes a la vez ahorran tiempo"       | No puedes aislar cuál funcionó y crea bugs nuevos.         |
| "Veo el problema, lo arreglo"                | Ver el síntoma ≠ entender la causa raíz.                   |

Fuentes:
- DEV, "How to Develop a Debugging Mindset (Devon O'Dell)" — https://dev.to/anaveecodes/how-to-develop-a-debugging-mindset-1h04
- Hermes Agent, "Systematic Debugging (4-phase)" — https://hermes-agent.nousresearch.com/docs/user-guide/skills/bundled/software-development/software-development-systematic-debugging

### 3.2 Técnicas de depuración concretas

#### git bisect
Búsqueda binaria sobre el historial de commits para localizar el commit que introdujo el bug.
- Práctica: `git bisect start`, marcar known-good y known-bad, `git bisect run <script de test>`.
- Con cientos de commits: ~**10 pasos** para converger (log₂ n). Automatizable totalmente con `git bisect run`.
- Requiere: repro determinista que devuelva exit code ≠ 0 en el malo.
- Trampas: commits que no compilan rompen la bisección (marcar `skip`); bugs de interacción (solo aparecen con la combinación de dos commits).

Fuentes:
- TrainWithSky, "Git bisect debugging" — https://devops.trainwithsky.com/blog/git/git-bisect-debugging
- Takao Blog — https://takao.blog/en/web/git-bisect-debug-regression/
- How2.sh — https://how2.sh/posts/how-to-debug-with-git-bisect-binary-search/

#### Minimizar reproducción (binary reduction)
Igual que git bisect pero sobre inputs/config/código:
- Comentar/eliminar la mitad del flujo y ver si el fallo persiste (divide and conquer).
- Reducir inputs hasta el caso mínimo.
- Automatizar el repro: script que falle a voluntad vale más que diez descripciones verbales.

#### Bucle de hipótesis (hypothesis-driven loop)
Observar → hipótesis → predecir (outcome esperado) → testear (experimento) → actualizar (aceptar/descartar). Nunca cambiar más de una variable por experimento; si cambias dos y falla, no sabes cuál lo rompió.

Fuentes adicionales:
- Buglyst, "Systematic Debugging Methodology" — https://buglyst.com/blog/the-debugging-mindset
- HackerNoon, "The Debugging Mindset" — https://hackernoon.com/the-debugging-mindset-building-resilience-and-problem-solving-skills-in-development

---

## 4. SRE: sitios confiables, SLI/SLO, error budgets y postmortems

### 4.1 SLI / SLO / Error Budget (Google SRE)
- **SLI** (Service Level Indicator): métrica real (latencia p99, disponibilidad, tasa de error).
- **SLO** (Service Level Objective): umbral objetivo sobre el SLI (p.ej. 99.9%).
- **Error budget**: 100% − SLO. El "presupuesto de error" que se puede gastar en experimentos, deploys arriesgados y mejoras.
- Relación con debugging: una caída de SLO activa el incidente; el error budget dice si "arriesgar" es aceptable. Los SLO deben reflejar **resultados de usuario**, no métricas técnicas sueltas.

### 4.2 Incident response
Fases de respuesta a incidentes en equipos SRE:
1. **Detection / alerting** — ante la seña, no esperar al usuario.
2. **Mitigation** — restaurar servicio PRIMERO (rollback, feature-flag off, scale-out). El debugging en caliente daña la experiencia: debuggear *después* de mitigar.
3. **Debugging / root cause analysis (RCA)** — como actividad *post-incidente* idealmente, con datos completos.
4. **Postmortem blameless** — describir qué pasó y por qué, sin culpa; identificar acciónate items.
5. **Prevention / follow-up** — controls, tests, monitors que cierran el loop.

> Principio: **mitigar primero, root-cause después.** No preguntes "por qué" mientras el servicio está caído.

### 4.3 Postmortem blameless (Google SRE)
- Foco en el **sistema y el proceso**, nunca en la persona ("blameless").
- Elementos típicos: timeline, impacto, causa, lo que funcionó, lo que falló, acciones correctivas con responsables.
- Cultura: el resultado es un **documento de aprendizaje**, no un expediente de culpabilidad. Las personas cometen errores; el sistema debe hacer difícil que el error cause daño.
- Importante: asignar owners y follow-ups; un postmortem sin acción correctiva es teatro.

Fuentes SRE:
- SRE Workbook (Google) — https://sre.google/workbook/index/
- SRE Book, "Postmortem Culture" — https://sre.google/sre-book/postmortem-culture
- Tomoda Hinata, "Incident response / runbook / oncall" — https://tomodahinata.com/en/blog/incident-response-runbook-postmortem-oncall-sre-guide

### 4.4 Observabilidad: tres pilares + eventos
- **Metrics**: valores numéricos agregados (Prometheus). Rápidos y baratos de consultar; sin relaciones entre servicios.
- **Logs**: registros con timestamp y contexto; explican la secuencia de eventos pero pueden ser ruidosos.
- **Traces**: flujo de una petición end-to-end (spans con trace ID); un trace es un árbol/DAG de spans. Mejor herramienta para RCA en microservicios.
- **Events**: cambios de estado externos (deploy, flag toggle, restart) que correlacionan con los anteriores.
- **OpenTelemetry** es el estándar unificado de instrumentación.
- Correlación: **unified observability** (logs+metrics+traces en una sola vista) reduce MTTR drásticamente (40-60% en muchos equipos).
- Matiz clave: **visibilidad ≠ observabilidad**. Visibilidad muestra el estado esperado; observabilidad permite *preguntas nuevas* a sistemas nunca antes vistos (p.ej. high-cardinality queries).
- **MTTD / MTTR**: time-to-detect y time-to-resolve; métricas operativas núcleo.

Aplicación a debugging: si el sistema no tiene trazas de extremo a extremo, el debug en microservicios es adivinanza. El fallo se rastrea por trace ID, no por logs dispersos.

Fuentes:
- Atatus, "Logs vs Metrics vs Traces" — https://www.atatus.com/blog/logging-traces-metrics-observability
- StrongDM, "What is Observability?" — https://www.strongdm.com/observability
- Honeycomb, "How Observability Helps Incident Response" — https://www.honeycomb.io/resources/getting-started/observability-helps-incident-response
- Xurrent, "RCA Guide for SREs" — https://www.xurrent.com/blog/root-cause-analysis-guide-sre

---

## 5. Análisis de causa raíz: técnicas formales (RCA)

### 5.1 5 Whys
- Preguntar "¿por qué?" repetidamente (5 veces típico) hasta llegar a la causa de proceso/sistema.
- Fortaleza: rápido, profundo, sin herramientas.
- Debilidad: dependiente de quién lo hace ("5 Whys" distintos dan causas distintas); tiende a una única cadena cuando los fallos suelen tener múltiples causas.

### 5.2 Ishikawa / Fishbone diagram
- Categoriza causas en "huesos" (típicamente: Method, Machine, Material, Measurement, People, Environment — 6M).
- Fortaleza: explora causas **múltiples y simultáneas**, no una sola cadena.
- Uso en TI: adaptable (código, infra, datos, proceso, personas, dependencias externas).

### 5.3 Análisis de eventos / DAG de causas
- Construir un **grafo acíclico dirigido (DAG)** de eventos: cada síntoma es hijo de una o varias causas.
- Los fallos complejos tienen **múltiples contribuyentes**; el DAG muestra combinaciones de eventos que disparan el fallo.
- Relacionado con el paper de Google "Debugging incidents in Google's distributed systems": la causa de incidentes grandes casi siempre es **multifactorial** → buscar causa única (single cause) es un sesgo peligroso.

### 5.4 Comparativa de técnicas

| Técnica          | Alcance        | Mejor para                                  | Riesgo                      |
|------------------|----------------|---------------------------------------------|-----------------------------|
| 5 Whys           | Cadena única   | Fallos simples con causa lineal             | Anchura falsa (una sola causa) |
| Ishikawa         | Múltiples causas | Fallos con factores concurrentes           | Mucha información sin prioridad |
| DAG de eventos   | Complejidad    | Incidentes distribuidos / multi-causa       | Costoso de construir        |
| Kepner-Tregoe    | Desviaciones   | Fallos reproducibles con "IS/IS NOT"        | Rigidez temporal            |

Fuentes:
- iSixSigma, "Ishikawa diagrams and 5 Whys" — https://www.isixsigma.com/cause-effect/root-cause-analysis-ishikawa-diagrams-and-the-5-whys/
- ADHDCode, "RCA techniques: 5 Whys / Fishbone" — https://adhdecode.com/debugging-distributed/post-mortem-analysis/root-cause-analysis-techniques-five-whys-fishbone/
- ACM DL, "Debugging incidents in Google's distributed systems" — https://dl.acm.org/doi/10.1145/3397880

---

## 6. Quick fix vs root fix y deuda técnica

### 6.1 El dilema
- **Quick fix / workaround**: restaura el servicio rápido (mitigation) pero deja la causa instalada.
- **Root fix**: corrige la causa; más caro y arriesgado en el corto plazo.
- La regla operativa: **mitiga con quick fix, conviértelo en tarea, y haz el root fix con dueño y fecha**. El error es el quick fix que nadie agenda y se convierte en permanente.

### 6.2 Deuda técnica
- Metáfora del "técnico que pega la grieta sin arreglar la tubería": el quick fix silencioso genera interés compuesto.
- Refactor estrategy vs benda: los fixes de parche repetidos en el mismo módulo = señal de deuda que ya merece refactor estructural (coincide con la Rule of Three del debugging).
- Priorización por cantidad: si un módulo tiene alta tasa de fallos y fixes repetidos, su refactor es más barato que seguir parcheando.

Fuentes:
- SysDesai, "Quick patch vs root fix" — https://www.sysdesai.com/news/5sIM1BzbhQc4
- TechWell, "Quick fixes and root cause analysis" — https://techwell.com/techwell-insights/2020/07/physical-metaphor-quick-fixes-and-root-cause-analysis
- Kyligence, "RCA quick-start guide" — https://kyligence.io/blog/root-cause-analysis-a-quick-start-guide
- SystemDesignLLD Crash Course, "Technical debt & refactoring strategy" — https://github.com/Sunchit/SystemDesignLLDCrashCourse/blob/main/05-senior-engineer-design-thinking/06-technical-debt-and-refactoring-strategy.md

---

## 7. Diseño de soluciones: trade-offs, ADR y comunicación

### 7.1 Arquitectura de decisión: ADR / RFC
- **ADR** (Architecture Decision Record): documento corto que registra una decisión arquitectónica en contexto y su justificación. Formato: Context / Decision / Consequences (y opcional Alternatives).
- Estándares: ADR GitHub org — https://github.com/architecture-decision-record/architecture-decision-record
- **RFC** (Request for Comments): colaborativo, usado en demanda abierta; ejemplo de proceso en la entrada de Lukas Niessen.
- Valor: las decisiones viejas con contexto evitan re-litigios; "por qué se hizo así" queda en el repo, no en la memoria.
- **Trade-off analysis**: evaluar alternativas explícitamente (coste/beneficio/riesgo) ANTES de decidir; documentar lo que se descartó y por qué.

### 7.2 Pre-mortem vs post-mortem
- **Post-mortem**: ¿qué pasó? (reactivo).
- **Pre-mortem** ("el proyecto ya fracasó; ¿por qué?"): imaginar el fracaso ANTES de ejecutar para exponer riesgos y debilidades latentes. Es baratísimo y altamente efectivo para evitar sorpresas.
- Uso: antes de grandes refactors, deploys o arquitecturas nuevas.

### 7.3 Comunicación de problemas / Soluciones (SCQA — Barbara Minto)
- **S**ituation — contexto que todos comparten.
- **C**omplication — el cambio que rompe el statu quo.
- **Q**uestion — pregunta implícita que el lector hará.
- **A**nswer — tu recomendación primero (conclusion-first).
- Principio de la pirámide de Minto: la respuesta/conclusión va al inicio; el detalle después, agrupado y ordenado.
- Aplicación: reports de bug, PR, incident postmortems, y análisis para ejecutivos.

Fuente: Management Consulted, "SCQA Framework" — https://managementconsulted.com/scqa-framework/

---

## 8. Investigación técnica: spikes y codebases desconocidas

### 8.1 Spikes (GitLab philosophy)
- **Spike**: período de tiempo acotado (días máximo) para investigar una incertidumbre técnica y reducir riesgo, sin entregar código de producción.
- Objetivos: validar viabilidad, estimar esfuerzo, descubrir trampas, decidir entre alternativas.
- Entregable: documentación del hallazgo + recomendación; NO "código de prueba" que se filtra a producción.
- Regla: acotar el tiempo; si no converge, escalar (más contexto, más senior, otra alternativa).

Fuente: GitLab Handbook, "Technical Spikes" — https://handbook.gitlab.com/handbook/engineering/development/growth/technical_spikes/

### 8.2 Acercarse a una codebase desconocida
Estrategias de exploración efectiva:
1. Lectura de docs/README y código "core" primero, no todo el árbol.
2. Trazar los flujos de datos de punta a punta (entry → handler → storage).
3. Buscar TODO/FIXME/`ponytail:` y tests para entender intención.
4. Usar tools de código-estructura (index de símbolos, call graph) en vez de leer millones de líneas.
5. Antes de editar: entender el **blast radius** (quién llama a lo que voy a tocar).
6. Hipótesis de comportamiento frente al código real; confirmar con pruebas o ejecuciones.
7. Documentar los hallazgos en notas o ADR para la próxima vez.

Fuente: CoderCops, "Technical due diligence / codebase assessment" — https://blog.codercops.com/blog/technical-due-diligence-codebase-assessment-2026

---

## 9. Fuentes consultadas (resumen)

| # | Fuente | Tema | URL |
|---|--------|------|-----|
| 1 | Meadows, *Thinking in Systems* | Systems thinking | https://research.fit.edu/.../Meadows-2008.-Thinking-in-Systems.pdf |
| 2 | BMF Tech | Systems thinking basics | https://bmf-tech.com/posts/systems-thinking-basics/ |
| 3 | Toolshero | Cynefin framework | https://www.toolshero.com/decision-making/cynefin-framework/ |
| 4 | BreakDecisions | First principles vs analogies | https://breakdecisions.com/articles/first-principles-vs-analogies |
| 5 | LibreTexts | Seven-Step Design Framework | https://eng.libretexts.org/.../7.03%3A_Seven-Step_Design_Framework |
| 6 | StructX | 7-Step Problem Solving | https://www.structx.com/Article002-7_Step_Problem_Solving.html |
| 7 | ASEE | Seven C's of engineering problems | https://peer.asee.org/board-96-the-seven-c-s-of-solving-engineering-problems.pdf |
| 8 | PMC | DMAIC | https://pmc.ncbi.nlm.nih.gov/articles/PMC10229001 |
| 9 | Deckary | McKinsey frameworks (MECE, issue trees) | https://deckary.com/blog/problem-solving-frameworks |
| 10 | DEV (O'Dell) | Debugging mindset | https://dev.to/anaveecodes/how-to-develop-a-debugging-mindset-1h04 |
| 11 | Hermes Agent | Systematic Debugging 4-phase | https://hermes-agent.nousresearch.com/docs/user-guide/skills/bundled/software-development/software-development-systematic-debugging |
| 12 | Buglyst | Systematic debugging methodology | https://buglyst.com/blog/the-debugging-mindset |
| 13 | HackerNoon | Debugging mindset & resilience | https://hackernoon.com/the-debugging-mindset-building-resilience-and-problem-solving-skills-in-development |
| 14 | TrainWithSky | git bisect | https://devops.trainwithsky.com/blog/git/git-bisect-debugging |
| 15 | Takao Blog | git bisect | https://takao.blog/en/web/git-bisect-debug-regression/ |
| 16 | How2.sh | git bisect binary search | https://how2.sh/posts/how-to-debug-with-git-bisect-binary-search/ |
| 17 | SRE Workbook | SRE practices | https://sre.google/workbook/index/ |
| 18 | SRE Book | Postmortem culture | https://sre.google/sre-book/postmortem-culture |
| 19 | Tomoda Hinata | Incident response / runbook | https://tomodahinata.com/en/blog/incident-response-runbook-postmortem-oncall-sre-guide |
| 20 | ACM DL | Debugging incidents at Google | https://dl.acm.org/doi/10.1145/3397880 |
| 21 | Atatus | Logs vs Metrics vs Traces | https://www.atatus.com/blog/logging-traces-metrics-observability |
| 22 | StrongDM | What is observability | https://www.strongdm.com/observability |
| 23 | Honeycomb | Observability & incident response | https://www.honeycomb.io/resources/getting-started/observability-helps-incident-response |
| 24 | Xurrent | RCA guide for SREs | https://www.xurrent.com/blog/root-cause-analysis-guide-sre |
| 25 | iSixSigma | Ishikawa & 5 Whys | https://www.isixsigma.com/cause-effect/root-cause-analysis-ishikawa-diagrams-and-the-5-whys/ |
| 26 | ADHDCode | 5 Whys / Fishbone | https://adhdecode.com/debugging-distributed/post-mortem-analysis/root-cause-analysis-techniques-five-whys-fishbone/ |
| 27 | SysDesai | Quick patch vs root fix | https://www.sysdesai.com/news/5sIM1BzbhQc4 |
| 28 | TechWell | Quick fixes & RCA | https://techwell.com/techwell-insights/2020/07/physical-metaphor-quick-fixes-and-root-cause-analysis |
| 29 | ADR GitHub | Architecture Decision Records | https://github.com/architecture-decision-record/architecture-decision-record |
| 30 | Lukas Niessen | RFC/ADR decision process | https://www.lukasniessen.com/blog/149-architecture-decision-process/ |
| 31 | Management Consulted | SCQA (Minto) | https://managementconsulted.com/scqa-framework/ |
| 32 | GitLab Handbook | Technical spikes | https://handbook.gitlab.com/handbook/engineering/development/growth/technical_spikes/ |
| 33 | CoderCops | Codebase assessment | https://blog.codercops.com/blog/technical-due-diligence-codebase-assessment-2026 |
| 34 | INCOSE (O'Reilly) | Systems engineering handbook | https://www.oreilly.com/library/view/incose-systems-engineering/9781119814290 |
| 35 | Kyligence | RCA quick-start | https://kyligence.io/blog/root-cause-analysis-a-quick-start-guide |
| 36 | SystemDesignLLDCrashCourse | Technical debt | https://github.com/Sunchit/SystemDesignLLDCrashCourse/blob/main/05-senior-engineer-design-thinking/06-technical-debt-and-refactoring-strategy.md |

---

## 10. Protocolo de debug sistemático

Protocolo con pasos numerados para aplicar ante CUALQUIER bug. Ejecutar en orden; no saltarse fases sin registrar por qué.

### Fase 0 — Contención (solo si hay impacto en producción)
1. Si el problema afecta a usuarios: **mitigar primero** (rollback, feature flag off, restart, scale-out). No debuggear en caliente.
2. Registrar el incidente (timeline, impacto) aunque todavía no haya causa.
3. Entrar al protocolo solo cuando el sistema esté estable.

### Fase 1 — Comprender el problema
4. Leer el mensaje de error completo (no solo la primera línea) y el stack trace entero.
5. Escribir la **declaración del problema**: comportamiento esperado vs comportamiento observado (IS / IS NOT), sin presuponer causa.
6. Clasificar el contexto con Cynefin: ¿Complicated (análisis experto) o Complex (experimentación)? Esto decide la estrategia.
7. Reunir evidencia: logs, métricas, trazas (trace ID), eventos de deploy/config, tiempo exacto.
8. Identificar **cambios recientes** (git log, deploys, feature flags) — la mayoría de bugs entran con un cambio.

### Fase 2 — Reproducir
9. Construir un **repro mínimo determinista**: script/test que falle a voluntad (repro rojo).
   - Si es flaky: aumentar la tasa de reproducción (ruidar 100x, paralelizar, estresar, inyectar sleeps). Un flake al 1% casi no es debuggable; al 50% sí.
   - Agotar posibilidades: fuzz básico, bisection de inputs.
10. Si el repro se puede automatizar con exit code ≠ 0 → apto para `git bisect run`.

### Fase 3 — Analizar patrones
11. **Minimizar la reproducción**: quitar inputs/config/pasos UNO A UNO; conservar solo lo load-bearing.
12. Buscar código **similar que funcione** (mismo codebase) y diferencialo (delta positivo).
13. Construir el **árbol de hipótesis MECE** del fallo (red, código, datos, config, infra, externo) y descartar ramas por evidencia.
14. Aplicar **Kepner-Tregoe**: diferencias IS vs IS NOT → factores distintivos → causas más probables.
15. Si hay historial: `git bisect` para localizar el commit culpable.
16. Cuando haya sospecha de causas múltiples: trazar **DAG de eventos** (evitar el sesgo de causa única).

### Fase 4 — Hipótesis, testeo y root cause
17. Convertir la mejor causa en hipótesis con **predicción falsable**: "si la causa es X, entonces pasar el input Y produce Z".
18. Testear UNA variable a la vez. Si cambias dos y falla, no sabes cuál la rompió.
19. Al confirmar, definir la causa raíz (no el síntoma) y su contribución.
20. **Regla de Three**: si ya probaste 3 fixes que no funcionan → STOP y cuestiona la arquitectura. Cada fix que revela acoplamiento nuevo = señal estructural.

### Fase 5 — Corregir y verificar
21. Escribir primero el **test de regresión** que apuntale el fallo (RED), usando el repro mínimo.
22. Aplicar el **fix raíz** (no el workaround) con el cambio más pequeño posible; sin refactors de paso.
23. Verificar: test específico en verde + suite completa sin regresiones.
24. Si el fix es un parche inevitable, registrar **deuda técnica** con dueño y fecha en el backlog; nunca parche silencioso permanente.

### Fase 6 — Cerrar el loop
25. Documentar: causa raíz, fix, cómo se detectó, qué impidió que se detectara antes.
26. Postmortem breve (blameless): timeline, impacto, lecciones, action items con owner.
27. Actualizar monitorización/alertas si el bug pasó desapercibido (inversión en el "por qué no se detectó antes").
28. Prevenir recurrencia: coordenar con tests, validación de entrada o guards donde corresponda.

---

## 11. Protocolo de análisis de problemas complejos

Para problemas difíciles, difusos o estratégicos (no bugs puntuales): arquitectura, decisiones de diseño, fallos recurrentes, "qué hacer cuando nadie sabe".

### Paso 1 — Definir y enmarcar (Problem framing)
1. Escribir el problema en UNA frase con impacto observable y medible (quién, qué, cuándo, cuánto).
2. Separar problema de síntoma: preguntar "¿qué cambió cuando empezó?" y "¿qué se rompe si NO se resuelve?".
3. Identificar constraints y requirements (restricciones duras, soft, no negociables).
4. Elegir lente: Systems thinking (¿es un sistema con feedback?) o First principles (¿hay supuestos falsos heredados?).
5. Situar en Cynefin: si es Complex, planear experimentos (probe → sense → respond); si Complicated, análisis experto.

### Paso 2 — Descomponer sin perder el todo
6. Construir el **issue tree MECE** del problema (2-5 ramas de alto nivel, sin solapes ni huecos).
7. Mapear el **sistema** relevante: elementos, conexiones, stocks/flows, feedback loops, delays.
8. Identificar **leverage points**: dónde un cambio pequeño produce mayor efecto (a menudo en reglas/estructura, no en parámetros).
9. Buscar causas con técnicas formales según complejidad:
   - Lineal → 5 Whys.
   - Múltiples factores → Ishikawa.
   - Distribuido/multicausa → DAG de eventos.
   - Desviación reproducible → Kepner-Tregoe (IS/IS NOT).

### Paso 3 — Hipótesis y evidencia
10. Ordenar hipótesis por **impacto × probabilidad** (priorizar ramas del árbol).
11. Recolectar datos solo donde discrimina (baseline, tendencias, comparativas).
12. Probar hipótesis con experimentos de bajo coste (spikes, prototipos, A/B, simulaciones).
13. Documentar lo que se descarta y por qué — evita re-visitas.

### Paso 4 — Generar y evaluar alternativas
14. Generar MÚLTIPLES soluciones antes de decidir (evitar el anclaje a la primera idea).
15. Evaluar con matriz de trade-offs: coste, riesgo, tiempo, mantenibilidad, deuda técnica que introduce.
16. Hacer **pre-mortem**: "¿cómo podría fracasar esta solución?" — lista de riesgos latentes.
17. Si es decisión arquitectónica: escribir **ADR** (Context / Decision / Consequences / Alternatives).

### Paso 5 — Decidir y comunicar
18. Elegir la solución con criterio explícito (score de la matriz, riesgo aceptado).
19. Comunicar con **SCQA / pirámide de Minto**: conclusión primero, contexto después, agrupado.
20. Formalizar el plan con pasos numerados, owner y deadlines (evita que el quick fix se vuelva permanente).

### Paso 6 — Ejecutar y verificar
21. Implementar incrementalmente (un componente a la vez) con señales de verificación en cada paso.
22. Probar contra la definición del Paso 1 (¿resuelve el problema medible?).
23. Medir post-implementación (baseline vs después); documentar el resultado.

### Paso 7 — Sostener y aprender
24. Convertir la solución en estándar/proceso si aplica (playbook, runbook, ADR, checklist).
25. Control (DMAIC): monitorear para detectar regresión antes de que el problema vuelva.
26. Revisar periódicamente: si los fixes se repiten en la misma zona → refactor estructural (deuda técnica madura).
27. Registrar lecciones en la base de conocimiento (postmortem o nota técnica) con enlace a este protocolo.

---

## 12. Glosario rápido (EN → ES)

| EN | ES |
|----|----|
| Root cause analysis (RCA) | Análisis de causa raíz |
| Postmortem (blameless) | Reunión post-incidente sin culpa |
| Error budget | Presupuesto de error |
| SLO / SLI | Objetivo / Indicador de nivel de servicio |
| Blast radius | Radio de impacto de un cambio |
| Leverage point | Punto de apalancamiento |
| Reproducibility | Reproducibilidad (repro) |
| Hipótesis falsable | Predicción que puede demostrarse falsa |
| Trade-off | Compensación entre opciones |
| Technical debt | Deuda técnica |
| Spike | Investigación técnica acotada en el tiempo |
| Quick fix / workaround | Parche temporal / atajo |
| Observations → hypothesis → test | Observar → hipótesis → probar |