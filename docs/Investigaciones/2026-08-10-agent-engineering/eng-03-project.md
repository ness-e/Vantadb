# ENG-03 — ENTREGA DE PROYECTOS, PLANIFICACIÓN, RIESGO Y META-TRABAJO

**Fecha:** 2026-08-10
**Ángulo:** Cómo se lleva un plan/proyecto de principio a fin (inception → closure), para un solo dev autónomo (humano o agente).
**Tipo:** Investigación (sin cambios de código).
**Método de investigación:** Fetch directo de fuentes primarias (DORA, Basecamp/Shape Up, Google SRE Book, Martin Fowler, Agile Alliance, Mountain Goat, Spotify Engineering) + búsquedas agregadas (MetaSearch con Bing/mwmbl/arxiv/crossref/StackOverflow; Wikipedia; CircleCI State of Software Delivery) + literatura consolidada de la industria (PMI/PMBOK, Software Engineering at Google, HBR, Kanban, indie engineering).

---

## 0. RESULTADO EJECUTIVO (TL;DR)

Un proyecto correcto no es una lista de archivos por tocar: es un **artefacto de decisión** con etapas explícitas, una secuencia de problemas ordenada de mayor riesgo a menor, barreras de calidad objetivables y un canal de aprendizaje que cierra el loop. La evidencia DORA insiste en que **velocidad y estabilidad NO son trade-offs opuestos** (los "elite" ganan en las dos), y que lo medible se mide por *flow de entrega* (lead time, deployment frequency, recovery time, change fail rate), no por horas trabajadas. Para un solo dev, el framework que emerge fusionando Shape Up (appetite, bets, circuit breaker), Scrum/XP (DoD, retrospectiva, CI/simple design), Kanban (WIP limits, colas) y PMBOK (risk register, WBS) es:

1. **Shaping primero, ejecución después** — problema acotado, con appetite de tiempo, sin rabbit holes.
2. **Slice end-to-end pequeño y shippable** (*working in small batches*) como unidad mínima de progreso.
3. **Definition of Done por nivel** (tarea / PR / feature / release) escrita y verificable.
4. **Gates y stop conditions explícitos** (circuit breaker): un proyecto que no cabe en su appetite se corta, no se extiende.
5. **Live artifact tracking** (plan file / scope map / hill chart): el documento, no la memoria, es la fuente de verdad.
6. **Continuous learning loop**: blameless post-mortem + retrospectiva corta + ajuste de proceso medido.

---

## 1. CICLO DE VIDA DE UN PROYECTO (PMI + ágil, fusionado)

### 1.1 Fases clásicas (PMI/PMBOK — esenciales)

PMBOK define 5 grupos de procesos que se ejecutan en espiral, no en línea recta:

| Fase | Salida mínima | Gate de salida |
|------|---------------|----------------|
| **Initiation** | Business case, project charter, stakeholders, objetivo SMART, scope preliminar | Charter: problema + por qué + criterio de éxito |
| **Planning** | WBS, schedule, presupuesto, risk register, plan de comunicación, Definition of Done | Baseline de tiempo/alcance/riesgo aceptado |
| **Execution** | Entregables incrementales, coordinación, status tracking | Feature/PR/release cumplen DoD |
| **Monitoring & Control** | Medición (DORA), variance analysis, change control, replanificación | Evidencia de progreso, no opinión |
| **Closure** | Handoff, documentación, lessons learned, post-mortem, deuda registrada | Nada "en el aire" sin owner |

Fuente: PMI / PMBOK Guide (knowledge areas; principios de valor, calidad en la entrega, liderazgo adaptativo en 6.ª/7.ª ed.). Versión lazy para un dev: ningun arte del PMBOK es **obligatorio**, pero el **risk register**, la **WBS** y el **integration management** (cómo encaja lo que haces con lo que ya existe) se pagan solos.

### 1.2 El equivalente del "plan file" en la industria

- **GitHub Projects / Jira**: EPIC → Story → Task, con estados y DoD por columna.
- **Shape Up**: Pitch (1-2 páginas) → Scopes (partes que se construyen/integran/finish solas) → Hill chart.
- **RFC/Design Docs (Google/Stripe/Shopify)**: el plan ES un documento que se revisa antes de commitear arquitectura.
- **Modelo "single source of truth"**: todo el estado vive en un artefacto versionado (plan file en el repo), donde el agente/humano escribe: objetivo activo, contrato de validación y próximo paso.

### 1.3 Inception bien hecha (lo que separa proyectos muertos de vivos)

Antes de escribir una línea de código, debe existir respuesta verificable a:

- ¿Este trabajo **tiene que existir**? (rung 1 de la escalera YAGNI — Fowler: solo 1/3 de features mejoran la métrica que apuntan; Kohavi et al. en MS).
- ¿Quién es el usuario y cuál es su *baseline*? (Shape Up: "what customers do without the thing we're building").
- ¿Cuál es el **appetite** (tiempo que VAMOS a invertir), en oposición al estimate (cuánto creemos que cuesta)?
- ¿Cuál es el criterio de éxito medible (acceptance criteria) y cuál el criterio de **abandono** (stop condition)?

---

## 2. DESCOMPOSICIÓN Y ESTIMACIÓN

### 2.1 Work Breakdown Structure (WBS)

- PMI: descomponer el entregable hasta **work packages** donde se puede asignar, estimar y verificar. Regla empírica: cada package = 1 slice de valor = independiente = testeable.
- En software un work package NO es "la feature entera": es el **thin vertical slice** (UI + lógica + datos) mínimo que produce algo usable (Shape Up cap 11, "Get One Piece Done").
- Anti-pattern: **iceberg** — backend enorme invisible bajo una UI pequeña; marcarlo explícito en el plan.

### 2.2 Atomic tasks

Qué hace que una tarea sea **atomic** (y por tanto planificable):

- **Un solo resultado verificable** (el test/PR/commit cambia una cosa).
- **Un solo owner** en un solo contexto, sin interrupciones.
- **DoD propio** (condiciones para cerrarla).
- **Estimable en rango corto** (<= 1-2 días de trabajo real; más grande → partir).
- **Visible en el plan** (si no está trackeado, no existe).

### 2.3 Estimación

| Técnica | Cómo | Cuándo funciona |
|---------|------|-----------------|
| **Relative sizing / story points** | Comparar contra un baseline conocido (1,2,3,5,8,13 Fibonacci) | Backlog de un mismo contexto |
| **Planning Poker (Mike Cohn)** | Votos privados simultáneos + discusión dirigida por extremos | Consenso; mata anclaje y presión social. Jørgensen (Simula): "los más competentes deben estimar" |
| **T-shirt sizing (S/M/L/XL)** | Categorías gruesas sin falsa precisión | Backlog nuevo o creativo (alto unknown) |
| **Time-box / appetite (Shape Up)** | Fijas el TIEMPO y dejas que varíe el ALCANCE | La mejor para features; fuerza trade-offs |
| **Reference class forecasting** | Comparar con proyectos pasados similares del propio repo | Calibración honesta (Kahneman: overconfidence fuera de la propia clase) |
| **Three-point (PERT: o/m/p)** | Optimista / más probable / pesimista | Solo con suficiente histórico |

Reglas de oro (Mountain Goat + Jørgensen):
- **Los que harán el trabajo estiman** (no PMs ajenos al contexto).
- **Estimaciones independientes antes de discutir** (mejora precisión).
- Estimación **relativa, calibrada con histórico real** (velocity/throughput propio), no intuición.
- Nunca oficies un promedio como fecha; usa distribución, y si solo hay un número, que sea prudente (sesgo de optimismo).

### 2.4 Alinear esfuerzo / dependencias / riesgo

El orden de ejecución se decide con tres criterios en tensión:
1. **Riesgo técnico alto primero** (el desconocido que puede matar la feature: spike ahora, no al final).
2. **Dependencias de dominio** (lo que destraba a otros).
3. **Valor de negocio primero** (feedback real del usuario en el slice inicial).

Nunca "primero lo fácil y lo difícil al final": el trabajo fácil es justo el que NO necesita planificación temprana.

---

## 3. GESTIÓN DE DEPENDENCIAS Y SECUENCIAS DE TAREAS

- **Modela el trabajo como DAG**: nodo = atomic task, arista = "B necesita a A". Las tareas sin aristas entrantes pendientes son paralelizables por buffers/waves.
- **Critical path (CPM/PERT)**: la ruta más larga de dependencias seriales define la fecha mínima. Optimizar es acortar el critical path, no "hacer más en general". En un solo dev el critical path suele ser él mismo → el cuello de botella es el **context switching** (evítalo con WIP bajo en foco).
- **Waves / partial ordering**: agrupa en olas reales (wave 1: foundations, wave 2: features) solo cuando hay dependencias reales; de lo contrario no serialices artificialmente.
- **Blockers formales**: si una tarea depende de una decisión ajena (design doc, approval, API externa), es un **blocker** — se marca, se data, se escala, no se "rodea".
- DORA "working in small batches" (https://dora.dev/capabilities/working-in-small-batches/): batches pequeños → menor lead time, menor rollout risk, recuperación más fácil. Es la entrada a casi todas las métricas.

---

## 4. GESTIÓN DE RIESGO (PMI + SRE + Shape Up)

### 4.1 Risk register (PMI)

Tabla viva en el plan file. Columnas mínimas:

| ID | Riesgo | Prob (1-5) | Impacto (1-5) | Score (P×I) | Respuesta | Trigger / due | Estado |
|----|--------|-----------|---------------|-------------|-----------|---------------|--------|
| R1 | API externa cambia contrato | 3 | 4 | 12 | Mitigate: contract tests + pin version | antes de feature | open |

Técnicas de respuesta: **avoid** (no hacer), **mitigate** (reducir P o I), **transfer** (delegar/asegurar), **accept** (explícito, con trigger de escalación). Para un solo dev: máx. 5-8 riesgos activos; los que no tienen acción de mitigación se borran (el registro no es un museo).

### 4.2 Pre-mortem (la técnica con mejor ROI cognitivo)

Método de Gary Klein ("Performing a Project Premortem", HBR 2007; popularizado por Kahneman en *Thinking, Fast and Slow*): **asume por un momento que el proyecto YA fracasó** y escribe la historia de por qué. Desbloquea el sesgo de optimismo y la información que se calla "por no quejarse". Un solo dev: hazte el pre-mortem antes del commitment fuerte del plan — 10 minutos contra 3 semanas de reinicio.

### 4.3 Rabbit holes y detección temprana (Shape Up)

- **Rabbit hole** = parte demasiado desconocida/compleja/abierta para apostar por un ciclo. Se **quita del scope** en shaping.
- "Present to technical experts": el shaping lo revisa un conocedor del sistema antes de convertirse en bet.
- **Declare out of bounds / No-gos**: qué NO vamos a hacer en este proyecto, explícito en el pitch.
- El **circuit breaker** es la regla de riesgo maestra: *un proyecto que no envía en su ciclo se CANCELA por defecto, no se extiende* (extender solo con razón de negocio objetiva). Es gestión de riesgo y de scope a la vez.

### 4.4 Cuándo pausar / escalar / parar (stop conditions)

- **Pausar**: falta información que cambia alcance o appetite (blocker real, no desgana).
- **Escalar**: el riesgo aceptado cruza su trigger (sube P o I), o el bloqueado depende de un tercero.
- **Parar (sunk cost NO)**: el círculo vicioso clásico es invertir más porque ya se invirtió. Stop conditions escritas ANTES:
  - El slice crítico no logra sus acceptance criteria en X iteraciones.
  - El appetite se duplicó sin re-negociar entre partes.
  - Apareció una alternativa de menor riesgo con igual valor.
  - El pre/post-mortem prevé un impacto mayor que el beneficio (ROI del proyecto vs ROI de la próxima apuesta).
- SRE (Google): definir **postmortem triggers ANTES** de que ocurran (outage visible, data loss, rollback, recovery-time sobre umbral) — misma lógica: los umbrales se deciden en frío, no en el fragor.

---

## 5. DEFINICIÓN DE DONE (POR NIVEL)

### 5.1 DoD por nivel (patrón Agile Alliance: contrato explícito y visible)

| Nivel | DoD mínimo (ejemplo genérico) | Gate de calidad |
|-------|-------------------------------|-----------------|
| **Task** | Implementada + commit + test que la cubre + docs del cambio | test green; CI green |
| **PR / Change** | Tests pasan, lint/clippy clean, review (humana o agent) hecha, sin debug code | CI + review + link a issue/changelog |
| **Feature / Scope** | Integration end-to-end probada, edge cases del baseline, sin regresiones, observabilidad si aplica, docs de usuario | DoD de feature + smoke en entorno real |
| **Release** | Versionado correcto (tag/CHANGELOG, no a mano), release notes, rollback plan, monitoring | DoD de feature + release gate (QA en bordes) |
| **Project** | Objetivos del pitch cumplidos, deuda registrada, handoff escrito, post-mortem si aplica | Criterios de éxito del charter |

Reglas de la fuente Agile Alliance (https://agilealliance.org/glossary/definition-of-done/):
- El DoD se **muestra de forma visible** — no vale "shared understanding" en la cabeza.
- Si no se cumple al cerrar el sprint/ciclo, **NO cuenta para el velocity** (nada de "casi done").
- Cada feature puede tener un DoD específico adicional al general.
- **Definition of Ready** (complemento): condiciones para que una tarea ENTRE al trabajo (acceptance criteria claros, dependencias resueltas) — la puerta de entrada del kanban pull.
- Beneficios observados: guía la estimación/pre-diseño, limita rework, elimina conflicto entre equipo y stakeholder.

### 5.2 Acceptance criteria efectivas

- Formato **INVEST**: Independiente, Negociable, Valioso, Estimable, Small, Testeable.
- **Given / When / Then** (Gherkin) cuando hay lógica de negocio; para casos técnicos escribir criterios como checks de test (p. ej. "sin pérdida de datos en corte de energía" se convierte en test).
- Criticarlos en el pre-mortem: el criterio que no se puede verificar en una iteración es un riesgo, no un requisito.

### 5.3 Gates por tipo de trabajo

- **Feature nueva**: DoD + slice de prueba + manual smoke ("QA is for the edges", Shape Up).
- **Bug fix**: test de regresión que reproduce el bug; fix en la función compartida (root cause, no symptom).
- **Refactor**: comportamiento idéntico (suite como red de seguridad — SelfTestingCode de Fowler).
- **Infra / release**: runbook + rollback validado + alerta de monitoring configurada ANTES de pushear.
- **Idea / R&D**: gate de timebox (spike), no DoD de producción.

---

## 6. COMUNICACIÓN Y REPORTING DE PROGRESO

### 6.1 Teoría de reporting (qué NO muestra el %)

Shape Up cap 13 ("Estimates don't show uncertainty"): el % de tareas completadas miente porque las tareas **imaginadas** se convierten en **descubiertas** al trabajar. El buen indicador no es "40% hecho" sino la posición en el **hill chart**: *uphill* (quedan desconocidos por resolver) vs *downhill* (solo ejecución) vs *done*.

### 6.2 Status update eficiente (formato status / blocker / next)

- **Qué se completó** (con evidencia: test/PR/artifact), **qué está en curso**, **qué bloquea** (con dueño y fecha).
- Los blockers se **reportan al instante**, no se acumulan a la reunión.
- En un repo de agentes: el status vive en el plan file (live artifact); el reporte humano es solo el diff desde la última vez (¿qué cambió? ¿qué riesgo se movió? ¿qué pido?).

### 6.3 Handoffs limpios y artefactos vivos

- **Contexto transferible**: un handoff deja a quien sigue en capacidad de continuar SIN preguntar al anterior: estado actual, decisiones con rationale, invariantes de dominio, comandos de verificación, deuda pendiente.
- **Artefactos vivos**: DESIGN.md / plan file / README del módulo actualizados EN CADA cambio. DORA 2021 y 2023: la documentación de alta calidad **amplifica** el impacto de las capacidades técnicas; DORA 2023: user-centricity predice +40% de performance.
- Regla: *toda tarea se cierra con su documentación al día* — si la escribes al final del proyecto, está desactualizada desde el día 1.

---

## 7. METODOLOGÍAS — QUÉ RETENER PARA UN SOLO DEV AUTÓNOMO

| Metodología | Idea central | Qué retener (dev solo) |
|-------------|--------------|------------------------|
| **Agile Manifesto** | Individuos > procesos; software funcionando > documentación; responder al cambio > seguir un plan | Los valores, no los ceremoniales |
| **Scrum** | Sprints timeboxed, DoD, retrospectiva, roles | Timebox + DoD + retrospectiva, sin ceremonies infladas |
| **XP** | TDD, pair programming, Simple Design, CI, YAGNI | TDD (SelfTestingCode), Simple Design, CI, YAGNI; el par lo sustituye la review (humana o de agente) |
| **Kanban** | Pull, WIP limits, colas cortas, optimizar flow no "velocidad aparente" | WIP=1-2 por foco, visualizar el trabajo, anti-context-switching |
| **Waterfall (traje)** | Fases secuenciales con gates | NO en ejecución de software; SÍ como esqueleto de entrega cuando el resultado debe cerrarse formalmente |
| **Lean / TPS** | Eliminar muda (desperdicio), jidoka (calidad en la fuente) | Cortar WIP muerto; errores detectados en origen (test temprano) |
| **Shape Up (Basecamp/37signals)** | Shaping → betting → building; appetite fijo, scope variable; circuit breaker | El mejor modelo completo para un dev solo: pitch con no-gos, hill chart, cancelar en vez de extender |
| **Getting Real / It Doesn't Have to Be Crazy** | Pequeño por defecto; constraints como feature | Menos feature, más foco; el límite ES el diseño |

Síntesis: **Kanban para el tablero, XP para la calidad técnica, Shape Up para el ciclo de entregable, de Scrum solo la retrospectiva, de PMBOK solo risk+wbs.** Cada pieza responde a una pregunta real; ningún ceremony de adorno.

---

## 8. DORA Y ENGINEERING EFFECTIVENESS (cómo transforma la ejecución diaria)

### 8.1 Las métricas (modelo actual de 5, dora.dev)

**Throughput**
- **Change lead time**: commit → producción.
- **Deployment frequency**: deploys por período.
- **Failed deployment recovery time** (evolución del MTTR): tiempo de recuperar un deploy fallido.

**Instability**
- **Change fail rate**: ratio de deploys que requieren intervención inmediata (rollback/hotfix).
- **Deployment rework rate**: deploys no planificados a consecuencia de un incidente.

Hallazgos core (https://dora.dev/research/): velocidad y estabilidad **correlacionan positivamente**; los "Elite" (clúster definido desde 2018) ganan en las cinco. Señales élite históricas: deploys bajo demanda, lead times de horas o menos, change fail rate bajo, recovery en horas. DORA 2024: platform engineering y user-centricity son los drivers; DORA 2025: **AI actúa como amplifier, pero el mayor retorno viene de los sistemas sociotécnicos subyacentes** — un agente sobre un proceso roto acelera el desorden.

### 8.2 Cómo aterrizar DORA en el día a día de UN dev

- **Cada change es pequeño y mergeable diariamente** → mejora lead time y deployment frequency.
- **Rollback-first**: si un deploy falla, recuperar (revert/rollback) es el objetivo #1; el fix definitivo puede esperar.
- **main verde permanente**: si `main` está roto TODO está en pausa; el pipeline roto es el incidente #1 de un proyecto individual.
- Buenas métricas son leading Y lagging, y sirven para **conversaciones sobre fricción**, no para forzar metas (Goodhart: fijar "todos deben deployar n veces/día" induce trampa).
- Pitfalls DORA a evitar: metric como goal, comparaciones entre apps dispares, siloed ownership, medir sin mejorar.

### 8.3 Medir lo correcto en un entregable de agente

| Qué medir | Cómo | Qué evita |
|-----------|------|-----------|
| Throughput real (tareas cerradas con DoD) | Emit diario desde el plan file | Ilusión de avance |
| Lead time del slice (idea → shippable) | Timestamps del plan | WIP eterno |
| Change fail / rework (cerrado que se reabrió) | Reabiertos en tracker | "feo pero cerrado" |
| Recovery (tiempo hasta volver a verde) | Incident log | Deuda en producción |

---

## 9. POST-MORTEM Y MEJORA CONTINUA

### 9.1 Blameless post-mortem (Google SRE Book, cap. 15)

- **Propósito**: documentar, entender causa raíz y — clave — definir *preventive actions* accionables. No es castigo.
- **Regla de oro**: asumir que todos hicieron lo correcto con la información que tenían. Se arreglan sistemas y procesos, no personas ("You can't fix people, but you can fix systems").
- **Estructura mínima**: timeline, impacto, acciones de mitigación, root cause(s), follow-up actions con owner y fecha, lecciones.
- **Postmortem triggers definidos antes**: outage visible, data loss, rollback intervenido, recovery-time sobre umbral, fallo de monitoring.
- **Unreviewed postmortem = inexistente**; compártelo con todo el que pueda aprender.
- Etsy abrió **Morgue** (github.com/etsy/morgue) como gestor de postmortems de referencia.
- Para un solo dev: "post-mortem de 10 minutos" tras incidentes o entregas dolorosas — 3 secciones: *qué pasó, por qué, qué cambio para que no se repita*.

### 9.2 Retrospectiva (Scrum) como loop de proceso

- Frecuencia proporcional al riesgo: semanal si hay foco intenso; por ciclo/entregable si no.
- Formato liviano que funciona solo: **Start / Stop / Continue** + **UNA acción de cambio** (no tres).
- La acción debe ser **medida** en la siguiente retro (¿mejoró lead time? ¿fail rate?).

### 9.3 Learning loop completo (imprescindible en un agente autónomo)

1. Incidente o entrega → datos (logs, diffs, reabiertos).
2. Post-mortem / retro → causa raíz (sin blame).
3. Acción de proceso única y medible.
4. Verificación en el siguiente período.
5. **Persistir la lección** (lessons learned / memory file) — no volver a aprender.

---

## 10. CULTURA DE RFC / DESIGN DOCS (cómo la industria decide con documentos)

### 10.1 Modelos de la industria

- **Google — Software Engineering at Google** (abseil.io/resources/swe-book): "Engineering" = programming integrated over time + personas; el diseño documentado (design docs con owners y estado "proposed/accepted/deprecated") y la code review son los mecanismos de decisión y diseminación de conocimiento.
- **Stripe — engineering blog**: cultura de RFC/design docs con dueño explícito, sección "Open Questions", y decisión documentada; decisiones reversibles se toman rápido, las difíciles se documentan.
- **Shopify — engineering blog**: "engineering with shared context"; ADRs (Architecture Decision Records) pequeños y vivos sobre el código.
- **Spotify (engineering culture)** — autonomy + alignment: squads autónomos con misión clara, decidir cerca del trabajo; la documentación conecta la autonomía con la alineación: https://engineering.atspotify.com/2014/03/spotify-engineering-culture-part-1/
- **Amazon practice**: "six-page narrative memos" antes de decidir — el documento fuerza pensamiento completo (eco conceptual en cualquier design doc previo al código).

### 10.2 Plantilla mínima de design doc útil

1. **Contexto/Problem statement** (baseline del usuario, Shape Up).
2. **Goals / Non-goals** (criterio de éxito + no-gos explícitos).
3. **Opciones consideradas** con trade-offs (no solo la elegida — documenta el "por qué no").
4. **Decisión y rationale**.
5. **Open questions / riesgos** (rabbit holes).
6. **Rollout / rollback / verificación**.
7. Owner + fecha de revisión.

Regla práctica: si una decisión de arquitectura tarda >30 min de discusión (o tendría diferentes caminos de ejecución), **merece design doc**. Si es trivial, un comentario en el PR basta — documentar sin necesidad es costo puro.

---

## 11. ENFOQUE "ONE PERSON BAND" / INDIE ENGINEERING

### 11.1 Cómo un dev solo ejecuta proyectos de calidad sin equipo

- **Slice pequeño y shippability constante**: la unidad de éxito no es "avance", es "algo envía". (Getting Real / Shape Up "Done means deployed", cap 10.)
- **Priorización brutal**: rank por valor × riesgo / costo; y la pregunta de Basecamp: *¿es este el problema adecuado? ¿es correcto el appetite? ¿es la solución atractiva? ¿es ahora?* (preguntas de la betting table, Shape Up cap 9).
- **Un solo proyecto a la vez** como bet; los bugs y pedidos sueltos viven en cola aparte, NO interrumpen el bet.
- **Limitar el sistema de trabajo**: WIP visualizado, cola corta, cool-down entre apuestas para ad-hoc (Shape Up).
- **Ritmo sostenible**: la soledad cognitiva amplifica el burn-out de WIP alto; el timebox es también preservación de energía (37signals "It Doesn't Have to Be Crazy at Work"; investigación: felicidad correlaciona con productividad y retención — literatura sobre Happiness & productivity en SE).
- **Externalizar la segunda opinión**: code review hecha por un peer/agente, pre-mortem contigo mismo, y publicar el plan para que alguien lo cuestione (combate sesgos identificados en *Cognitive Biases in Software Engineering*: overconfidence, anchoring, planning fallacy).
- **Simplicidad por defecto** (Yagni de Fowler): costo de build + costo de delay + costo de carry + costo de repair para cualquier feature especulativa.

### 11.2 Diferencia clave: humano con equipo vs dev solo

- En equipo la planificación es coordinación; solo, la planificación es **memoria externa y compromiso consigo mismo** → el plan file es impositivo, no opcional.
- El dev solo no tiene quién lo contradiga: los gates (DoD, pre-mortem, review de agente, métricas) sustituyen la fricción social saludable que en un equipo la proveen los pares.

---

## 12. SÍNTESIS DE FUENTES (verificación y enlaces)

Fuentes primarias consultadas directamente:
1. DORA — "DORA's software delivery performance metrics" guide: https://dora.dev/guides/dora-metrics-four-keys/
2. DORA — Research program (hallazgos anuales 2014-2025): https://dora.dev/research/
3. DORA — Working in small batches: https://dora.dev/capabilities/working-in-small-batches/
4. Basecamp — Shape Up (Ryan Singer), libro completo: https://basecamp.com/shapeup
5. Google SRE Book — "Postmortem Culture: Learning from Failure" (cap. 15): https://sre.google/sre-book/postmortem-culture/
6. Martin Fowler — Yagni: https://martinfowler.com/bliki/Yagni.html
7. Agile Alliance — Glossary "Definition of Done": https://agilealliance.org/glossary/definition-of-done/
8. Mountain Goat Software (Mike Cohn) — Planning Poker: https://www.mountaingoatsoftware.com/agile/planning-poker
9. Spotify Engineering — "Spotify engineering culture (part 1)": https://engineering.atspotify.com/2014/03/spotify-engineering-culture-part-1/
10. CircleCI — 2024 State of Software Delivery: https://circleci.com/resources/2024-state-of-software-delivery/

Fuentes agregadas / de referencia (búsquedas, snippets y literatura consolidada):
11. PMI — Project Management Body of Knowledge (PMBOK), process groups y risk/WBS.
12. Google — "Software Engineering at Google" (abseil.io/resources/swe-book/).
13. Klein, G. — "Performing a Project Premortem", Harvard Business Review, 2007.
14. Jørgensen, M. — review de expert estimation (Simula Research Lab).
15. Kahneman, D. — Thinking, Fast and Slow (planning fallacy, reference class forecasting).
16. Agile Manifesto y 12 principios: https://agilemanifesto.org
17. Scrum Guide / scrum.org (Definition of Done, Sprint, Retrospective).
18. Anderson / Kanban (WIP limits, pull) y Lean/TPS (muda, jidoka).
19. Wikipedia — Peer_Microservices/Platform engineering; Continuous integration (snippets de búsqueda).
20. arXiv:1707.03869 — "Cognitive Biases in Software Engineering" (mapping study).
21. arXiv:2406.07737 — "The Future of AI-Driven Software Engineering".
22. arXiv:1904.08239 — "Happiness and the productivity of software engineers".
23. Ko, A. et al. — "Rethinking Productivity in Software Engineering" (Springer, 2019), cuatro lentes de productividad.
24. Dingsoyr & Dyba — "Team effectiveness in software development" (CHASE 2012).
25. StackOverflow #385213 — discussion sobre métricas de proceso.
26. Etsy — Morgue (gestor de postmortems): https://github.com/etsy/morgue
27. 37signals — Getting Real / It Doesn't Have to Be Crazy at Work.
28. patio11 / Indie Hackers — literatura de indie engineering y one-person products.

---

## CHECKLIST POR FASE DE UN PROYECTO CORRECTO (INICIAR → PLANIFICAR → EJECUTAR → CERRAR)

### Fase A — INICIAR (Inception)
- [ ] Problema escrito en una frase; usuario y *baseline* definidos (qué hace hoy sin esto).
- [ ] Criterio de éxito medible (acceptance criteria de alto nivel) y NO-goals explícitos.
- [ ] **Appetite de tiempo fijado** (no estimación); riesgo si el appetite falta.
- [ ] **Pre-mortem** realizado: escribir la historia del fracaso y sus causas.
- [ ] Alternatives consideradas (1 línea cada una por qué no) + decisión + rationale.
- [ ] Scope preliminar: sin rabbit holes declarados fuera de los límites.
- [ ] Stop conditions / circuit breaker escritos (cuándo se CANCELA, no solo cuándo se termina).
- [ ] Stakeholders/handoff target: quién recibe esto y qué necesita.

### Fase B — PLANIFICAR (Planning)
- [ ] **WBS**: trabajo descompuesto en work packages / scopes / atomic tasks verificables.
- [ ] Cada task: owner, DoD propio, estimación relativa corta (<= 2 días) o timebox.
- [ ] **DAG / secuencia**: dependencias explícitas; critical path identificado.
- [ ] Orden de ejecución: riesgo alto → dependencias → valor.
- [ ] **Risk register**: 5-8 riesgos, mitigación y trigger por cada uno.
- [ ] Definition of Done por nivel (task/PR/feature/release) escrita y visible en el plan.
- [ ] Criterios del proyecto escritos en el plan file (objetivo activo, contrato de validación, próximo paso).
- [ ] Baseline de métricas DORA del período (leading/lagging para revisar luego).
- [ ] Design doc (solo si la decisión de arquitectura lo merece) con owner y fecha.

### Fase C — EJECUTAR (Execution)
- [ ] **Small batches**: earliest shippable slice primero; integrar y terminar un scope antes del siguiente.
- [ ] WIP limitado (1-2 por foco); un solo bet activo.
- [ ] Cada PR: CI verde, tests, review, DoD del nivel cumplido.
- [ ] `main` verde permanente (un pipeline roto se arregla ANTES de otra cosa).
- [ ] **Live artifact**: plan file actualizado al cerrar cada tarea (estado, decisiones, riesgo, next).
- [ ] Blockers reportados al instante (con dueño y fecha), nunca acumulados.
- [ ] Pregunta periódica al mapa: ¿scope sigue == appetite? ¿algún rabbit hole emergió? (scope hammering).
- [ ] Si un scope no cabe: **recortar alcance, no calidad** ; si el proyecto no cabe: circuit breaker.

### Fase D — CERRAR (Closure / Release)
- [ ] Release versionado formalmente (tag/CHANGELOG/release notes); nunca versiones a mano.
- [ ] Rollback plan + monitoring verificado ANTES de pushear.
- [ ] Deuda técnica y "ponytail:" simplificaciones registradas con su techo conocido y cuándo pagarlas.
- [ ] Handoff escrito: estado, decisiones con rationale, invariantes, comandos de verificación, pendientes por prioridad.
- [ ] Documentación/artefactos vivos al día (README/DESIGN.md/plan file).
- [ ] **Post-mortem o lección** (según incidente/entrega): qué pasó, por qué, qué cambio.
- [ ] Retrospectiva: una acción de cambio medible para el siguiente proyecto.
- [ ] Métricas DORA del período comparadas contra el baseline; ajuste de proceso.
- [ ] Lessons learned persistidas (memory file) — no re-aprender la misma lección.