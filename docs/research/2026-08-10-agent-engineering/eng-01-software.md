# Ingeniería de Software End-to-End: Prácticas y Referencias

> Investigación sobre ingeniería de software: del requisito al mantenimiento. Basado en fuentes primarias y blogs de ingeniería de referencia (Google, Netflix, Uber, Shopify, Stripe), literatura canónica (Clean Code, Refactoring, Fowler, Kent Beck, 12-Factor, DORA, SRE) y mejores prácticas de la industria de 2025-2026.
>
> Fecha: 2026-08-10 · Idioma: español (términos técnicos en inglés) · Área: ingeniería de software

---

## 1. Ciclo de vida de una feature (end-to-end)

El ciclo de vida completo de una feature, desde que se concibe hasta que se mantiene, tiene las siguientes fases:

| Fase | Objetivo | Entregable clave |
|---|---|---|
| Discovery | Validar el problema antes de construir | PRD / problem statement |
| Requirements / Spec | Definir alcance y criterios de aceptación | Spec + acceptance criteria |
| Design | Elegir la estrategia de implementación | Design doc / RFC |
| Implementation | Escribir el código con TDD | Código + tests |
| Testing | Verificar en múltiples niveles | Pirámide de tests en CI |
| Review | Validar calidad y diseño por pares | PR aprobado (LGTM) |
| Deployment | Liberar de forma segura e incremental | Release / rollout |
| Monitoring | Confirmar que funciona en producción | Dashboards + alertas |
| Post-release | Aprender de incidentes y medir impacto | Postmortem + métricas |

### Agile y planning iterativo
- Un feature pasa por fases **iterativas**, no cascada estricta: definir → implementar → verificar → liberar en ciclos cortos.
- Cada milestone genera un deliverable concreto (requisitos claros antes de diseño, diseño antes de código).
- El `release cycle` incluye feature flags, rollout canary, monitoreo y plan de rollback: publicar no es el final, es el punto medio.
- "Talk first, code later": escribir el enfoque antes de codificar reduce rework y alinea equipos multi-función.

**Fuentes:**
- https://jellyfish.co/blog/sdlc-best-practices
- https://docs.gitscrum.com/en/best-practices/managing-feature-development-cycles
- https://blog.pragmaticengineer.com/talk-first-code-later/

---

## 2. Arquitectura limpia, SOLID, DDD y boundaries

### Principios SOLID
1. **S**ingle Responsibility: una clase/módulo debe tener una única razón para cambiar.
2. **O**pen/Closed: abierto a extensión, cerrado a modificación.
3. **L**iskov Substitution: las subclases deben poder sustituir a su base sin romper invariantes.
4. **I**nterface Segregation: interfaces pequeñas y específicas por cliente.
5. **D**ependency Inversion: depender de abstracciones, no de concreciones (el detalle depende de la política, nunca al revés).

### De SOLID a capas (estructura recomendada)
```
Domain (entidades + reglas de negocio, sin dependencias)
  ↓
Application (use cases / servicios de aplicación)
  ↓
Infrastructure (repos, clientes HTTP, DB, integraciones)
  ↓
Web / Interface (controllers, presenters, frameworks)
```
La regla de dependencia apunta hacia adentro: las capas internas no conocen nada de las externas.

### Las 4 características de una buena arquitectura (Uncle Bob)
1. **Independiente del framework**.
2. **Testable** sin infraestructura externa (la regla del negocio se testea sin DB ni HTTP).
3. **Independiente de la UI**.
4. **Independiente de la base de datos**.

### DDD essentials (nivel pragmático)
- **Bounded context**: cada contexto tiene su propio modelo de dominio, lenguaje ubicuo y límites explícitos.
- **Aggregate**: clúster de entidades con invariantes transaccionales; solo se accede por su raíz.
- **Domain service** para lógica que no pertenece a ninguna entidad.
- **Eventos de dominio** para desacoplar efectos dentro del mismo contexto.

### Boundaries sin ceremonia (version pragmática / DDD lite)
- No toda la arquitectura necesita ser hexágono + DDD puro el día uno; hay un espectro:
  - **Layered** (default): capas simples por dependencia.
  - **Hexagonal / Ports & Adapters**: la lógica de negocio no depende de nada externo.
  - **Vertical slice**: cada feature es una rebanada completa (request → response), evita la dispersión por capas técnicas.
  - **DDD lite**: bounded contexts + aggregates sin la ceremonia completa de eventsourcing/CQRS.
- El objetivo es **evitar acoplamiento accidental**, no la perfección formal. Un boundary mal dibujado es peor que ninguno.
- Caso real: Shopify usa un **modular monolith** de 2.8M líneas de Ruby; aplicó boundaries entre componentes (Packwerk) para poder swap-deletear el motor de impuestos sin romper el resto. Extractó solo 2 servicios (storefront read-only y credit-card vaulting).

**Fuentes:**
- https://youngju.dev/post/2022-06-22-clean-architecture
- https://developersvoice.com/blog/architecture/pragmatic-domain-boundaries
- https://martinfowler.com/bliki/MonolithFirst.html
- https://shopify.engineering/deconstructing-monolith-designing-software-maximizes-developer-productivity

---

## 3. Código de calidad: escribir software legible y mantenible

Principios de Clean Code (Robert C. Martin) y buenas prácticas modernas de naming:

### Naming
- Los nombres deben **revelar la intención**: `isActive`, `calculateTotal`, no `flag`, `doStuff`.
- Una variable/función hace una cosa → un nombre. Si necesitas "and" en el nombre, separa responsabilidades.
- Filosofía: el código se lee 10x más de lo que se escribe; la legibilidad es una feature, no un lujo.

### Funciones
- **Una función = una tarea**: funciones pequeñas con un solo propósito y un solo nivel de abstracción.
- Evita profundidad de indentación excesiva (early returns, guard clauses).
- Los parámetros importan: pocos, con nombres claros; evita flags booleanos que cambian el comportamiento.

### Anti-patrones que eliminar
- **"Later equals never"**: los TODO ("lo arreglo luego") no se ejecutan; el código postergado se convierte en el estándar.
- Código clever/ingenioso por encima de claro: si el equipo tarda en entenderlo, es un bug futuro.
- Búsqueda manual de datos (loops + condiciones) donde hay una función estándar del lenguaje (e.g. `find`, `filter`, `map`).

### Legibilidad general
- Los cambios pequeños y bien aislados son más fáciles de revisar y menos propensos a errores que los cambios masivos.
- El estándar "production-grade" exige manejo de errores, logging accionable y observabilidad, no solo que "funcione".

**Fuentes:**
- https://besthub.dev/articles/clean-code-principles-and-best-practices
- https://martinfowler.com/bliki/WriteReadableCode.html
- https://blog.pragmaticengineer.com/readable-code/

---

## 4. TDD y la pirámide de testing

### TDD (Test-Driven Development, Kent Beck)
El ciclo fundamental de 3 pasos (Red-Green-Refactor):
1. **Red**: escribe un test que falla (expresa el comportamiento deseado).
2. **Green**: escribe el mínimo código que haga pasar el test.
3. **Refactor**: elimina duplicación y mejora diseño, con los tests como red de seguridad.

**Reglas de TDD:**
- No escribas código de producción sin un test fallando primero.
- Escribe solo el código necesario para pasar el test (YAGNI en su máxima expresión).
- Corre la suite con frecuencia: feedback en segundos, no al final.
- Kent Beck: "Dos reglas: escribe el test una línea antes del fallo, escribe el código una línea antes de pasar".

### Pirámide de testing (~70/20/10)
1. **Unit tests (70%)**: rápidos, aislados, testean una unidad de comportamiento. La base de la pirámide.
2. **Integration tests (20%)**: verifican interacción entre componentes (DB real o testcontainers, API, colas).
3. **E2E (10%)**: flujos completos de usuario; lentos y frágiles, pocos pero críticos.

### Niveles adicionales de test
- **Test doubles**: stubs, mocks y fakes para aislar la unidad, pero **no abuses de mocks**: de-más mocks == tests que mienten.
- **Property-based testing**: definir invariantes y dejar que la herramienta genere casos (entradas aleatorias). Complementa los casos de ejemplo.
- **Fuzzing**: entrada aleatoria/adversarial para cazar crashes y violaciones de invariantes; imprescindible en parsers, validadores y código con input no confiable.
- **Mutation testing**: muta el código (cambia `<` por `>`, borra una condición) y verifica que los tests lo detecten. Métricas: **mutation score ≥ 70%**; un test que pasa con el código mutado es un test muerto.

### Números de referencia
- **Cobertura**: 80% como mínimo razonable; más allá indica returns decrecientes. La cobertura mide qué se ejecutó, **no** que esté verificado: la mutation testing es la métrica de calidad real.
- Los dos sombreros de Fowler: cuando refactorizas, debes tener **tests antes**; refactorizar sin tests es reescribir.

**Fuentes:**
- https://martinfowler.com/bliki/TestDrivenDevelopment.html
- https://codecademy.com/article/tdd-red-green-refactor
- https://tdd.mooc.fi/1-tdd
- https://circleci.com/blog/testing-pyramid
- https://iamraghuveer.com/posts/test-strategies-unit-vs-integration-vs-e2e
- https://yrkan.com/blog/mutation-testing-coverage

---

## 5. Refactoring seguro (Martin Fowler)

Refactoring = "cambiar la estructura interna sin cambiar el comportamiento observable". No agregar features; reorganizar código para preservar comportamiento.

### Los dos sombreros (The Two Hats)
- **Adding functionality**: agrega features.
- **Refactoring**: reestructura sin agregar nada.
- Nunca mezcles los dos sombreros en el mismo commit: confunde la revisión y complica el rollback.

### Loop de refactoring seguro (4 pasos)
1. **Detectar el smell** (code smell): método largo, duplicación, feature envy, god object, shotgun surgery, speculative generality.
2. **Seleccionar la técnica**: Extract Method, Extract Class, Inline, Introduce Parameter Object, Replace Conditional with Polymorphism, Rename...
3. **Aplicar en pasos pequeños** con tests entre cada paso. Cada transformación debe preservar el comportamiento y estar validada por la suite.
4. **Verificar**: corre los tests tras cada transformación; si falla, el último cambio es el culpable y el revert es barato.

### Catálogo de smells (Refactoring Guru)
- **Bloaters**: Long Method, Large Class, Primitive Obsession, Long Parameter List, Data Clumps.
- **Object-Orientation Abusers**: Switch Statements, Temporary Field, Refused Bequest, Alternative Classes with Different Interfaces.
- **Change Preventers**: Divergent Change, Shotgun Surgery, Parallel Inheritance Hierarchies.
- **Dispensables**: Comments (como muletilla), Duplicate Code, Lazy Class, Data Class, Dead Code, **Speculative Generality** (generics para un futuro hipotético = deuda innecesaria).
- **Couplers**: Feature Envy, Inappropriate Intimacy, Message Chains, Middle Man.

### Design Stamina Hypothesis (Fowler/Shopify)
- Al inicio de un sistema, "no architecture" permite máxima velocidad. El diseño paga en la madurez: cuando la velocidad de añadir features cae, invertir en diseño.
- **Monolith-first**: empieza con un monolith modular, extrae servicios solo cuando el rendimiento o la seguridad lo exijan. Diseñar microservicios sin experiencia de dominio es la apuesta más arriesgada.

**Fuentes:**
- https://alybadawy.com/articles/architectural-and-design-patterns/refactoring-techniques
- https://refactoring.guru/refactoring/catalog
- https://martinfowler.com/bliki/DesignStaminaHypothesis.html
- https://github.com/cskwork/clean-code-skill

---

## 6. Code review efectivo

### Guías oficiales de Google (Referencia: eng-practices)
- **Justificar el enfoque**: "¿Por qué hacemos este cambio?" y "¿Cómo debería mirarse este CL?".
- **Escribir buenas descripciones de CL**: resumen breve + contexto + cómo se verificó.
- **CLs pequeños**: idealmente < 400 líneas; los CLs pequeños se revisan mejor y se detectan más bugs por token. Nada justifica un CL de 5000 líneas.
- **Terminología**: CL (ChangeList), LGTM (Looks Good To Me), "nit" = detalle menor.

### Lo que aprecia un reviewer
- El autor debe probar el código y **explicar cómo lo probó** en la descripción.
- Separar refactors de features (dos sombreros) para revisarlos con el frame correcto.
- Responder a todos los comentarios; si no estás de acuerdo, explica, no ignores.
- Todas las revisiones se hacen en un solo pase: el autor revisa antes de enviar, el reviewer revisa el diff completo.

### AI code review (2025-2026)
- Herramientas: CodeRabbit, GitHub Copilot Reviews, Cursor Review, Graphite, uReview (Uber analiza ~90% de sus ~65K cambios semanales con GenAI).
- Rol correcto: **primer pase** para detectar bugs obvios, estilo, cobertura y seguridad básica; **nunca reemplaza** la revisión humana, especialmente en auth, billing, seguridad y cambios de datos.
- El AI acelera el feedback loop; el humano aporta contexto de negocio y diseño que la IA no tiene.

### Cultura
- El objetivo de la review no es ganar discusiones, es que el código mejore y el equipo aprenda.
- Da feedback accionable ("cambia X por Y porque..." en lugar de "esto está feo") y sin humillaciones públicas.

**Fuentes:**
- https://google.github.io/eng-practices/
- https://github.com/google/eng-practices
- https://zglg.work/en/ai/guides/ai-code-review-tools
- https://www.uber.com/blog/ureview/
- https://blog.pragmaticengineer.com/good-code-reviews-better-code-reviews/

---

## 7. Quality gates, CI/CD y shift-left

### Quality gates
Son **umbrales de aprobación/fallo** en el pipeline que bloquean el merge/deploy si no se cumplen:
- Linters y formatters (falta chequear en CI: no "lo formateo localmente", se falla si no está formateado).
- Static analysis (SonarQube, clippy, mypy, ESLint estricto).
- Coverage mínimo (80%+).
- Tests unitarios/integration/E2E pasando.
- Checks de seguridad (dependency scanning, SAST).
- En proyecto de alta madurez: mutation score y análisis de complejidad como gates.

### Shift-left
- **"Pocos bugs en producción porque los encuentras baratos en desarrollo"**: los bugs cuestan 10-100x más caros si se detectan en producción vs desarrollo (IBM research: factor ~100-200x).
- Mover la verificación lo antes posible: pre-commit hooks (formato + lint + tests rápidos), CI por PR, tests antes del merge.
- Meta: **cada PR debe ser verificable en CI antes de la review humana** — esta es la definición de CI.

### Pipeline moderno
1. Pre-commit (hooks locales).
2. CI en PR (build + lint + static analysis + tests + coverage gate).
3. Merge → build de release.
4. CD: deploy automatizado con canary/feature flags.
5. Verificación post-deploy automatizada (smoke tests en producción).

**Fuentes:**
- https://bfotool.com/interview/cicd/quality-gates
- https://autemos.com/en/blogs/quality-gates-cicd
- https://yrkan.com/blog/shift-left-testing-early-detection
- https://martinfowler.com/articles/is-quality-worth-cost.html
- https://newsletter.pragmaticengineer.com/p/cicd-with-robert-erez

---

## 8. Documentación viva: ADRs, Design Docs, Working Backwards

### ADR (Architecture Decision Record)
Registro ligero de cada decisión de arquitectura:
- **Contexto**: el problema y las fuerzas en juego.
- **Decision**: qué se decidió y por qué.
- **Consequences**: implicaciones positivas y negativas, trade-offs conocidos.
- **Alternativas consideradas**: las opciones descartadas y por qué.
Regla práctica: 1 ADR por decisión significativa, se mantienen en el repo junto al código.

### Google Design Docs
> "El trabajo del ingeniero no es producir código, sino resolver problemas; el texto no estructurado puede ser la mejor herramienta temprana."
- Propósito: consenso organizacional, identificación temprana de problemas (cuando el cambio aún es barato), consideración de cross-cutting concerns, escalar el conocimiento de senior engineers, memoria organizacional.
- Anatomía típica:
  1. **Goals / Problem statement** (objetivos medibles: "reducir P95 de 2.1s a <1s").
  2. **Context and scope**.
  3. **Architecture / system design** (con diagrama, componentes, failure modes).
  4. **APIs / contracts** (backwards compatibility).
  5. **Alternatives considered** (con trade-offs).
  6. **Risks & mitigations**, **security/privacy/i18n/storage** si aplica.
  7. **RFC phase**: tag reviewers, preguntas abiertas, deadline.
- Lifecycle: creación e iteración rápida → review (múltiples rondas) → implementación → mantenimiento. Actualiza si el sistema aún no shippeó; si ya shippeó, las enmiendas se añaden como docs nuevos enlazados (modelo "constitución con amendment").
- Los design docs actúan como "code review antes de escribir código" (todo diseñador lo presenta a seniors).

### Amazon Working Backwards (PR/FAQ)
1. Escribe el **press release** (1-2 páginas) de cómo se contará el producto al mundo.
2. Escribe el **FAQ** con las preguntas difíciles que el equipo debe contestar.
3. Sin PR/FAQ claro → el feature no debería construirse. Define el resultado de éxito (cliente) antes de la implementación.

### Casos empresariales de planning
- **Uber**: evolucionó de DUCK/Google Docs (2013) → RFCs por mailing list (400+/semana) → Engineering Review Docs (ERDs) en tooling propio con approvers y tiered templates (light para cambios de equipo, heavy para cambios org-wide). Blindaron el proceso por "criticality" de la decisión.
- **Stripe**: procesos de planning y review de API muy estrictos; las APIs son contratos de larga vida revisados con rigor.

**Fuentes:**
- https://www.industrialempathy.com/posts/design-docs-at-google
- https://abseil.io/resources/swe-book/html/ch10.html
- https://docs.gitscrum.com/en/best-practices/documenting-architectural-decisions
- https://larksuite.com/en_us/blog/amazon-working-backwards
- https://ideaplan.io/frameworks/working-backwards
- https://newsletter.pragmaticengineer.com/p/rfcs-and-design-docs
- https://blog.pragmaticengineer.com/rfcs-and-design-docs/

---

## 9. Deuda técnica y productividad de equipo

### Boy Scout Rule (Uncle Bob, Clean Code)
> "Deja el campamento más limpio de como lo encontraste."
- Siempre que toques un archivo, deja al menos una mejora pequeña (renombrar una variable, extraer un método); sin proyecto paralelo de "pagar deuda", se paga de forma incremental.
- Cada nueva feature debe **mejorar** el diseño del código existente, no degradarlo (Shopify observó exactamente este cambio cultural).

### Gestión pragmática de la deuda
- **Registra la deuda** (issue/backlog con etiqueta `tech-debt`), no la dejes en comentarios: una deuda sin registro no genera trabajo.
- **Límite por PR**: un budget de deuda declarada (máx. N % del PR) o cada PR de feature incluye su propia limpieza.
- La deuda estratégica es necesaria cuando el costo de hacerlo bien hoy > costo de pagarlo luego. Lo malo es la deuda **no reconocida**.
- Design Stamina (Fowler): la velocidad para añadir features cae con el tiempo sin diseño; el momento de invertir en diseño es cuando esa curva se desploma.

### DORA metrics (el estándar de velocidad + estabilidad)
| Métrica | Qué mide | Ideales (elite) |
|---|---|---|
| **Deployment frequency** | Qué tan seguido depliegas | On-demand (múltiples por día) |
| **Lead time for changes** | Del commit al deploy en producción | < 1 día |
| **Change failure rate** | % de deploys que causan fallo/corrección | < 15% (elite ~5-10%) |
| **Mean Time to Recovery (MTTR)** | Tiempo en recuperarse de un incidente | < 1 hora |

- **Nunca** usar DORA para rankear individuos; son métricas de sistema/equipo.
- Reliability se añadió como 5ª dimensión (SRE). La velocidad duradera exige estabilidad.
- **Trunk-based development**: el mechanism real que correlaciona con DORA elite es el *batch size* — ramas de 1-2 días y merges frecuentes (estudio con 33.000 profesionales). Menos divergencia = menos conflictos = menos fix time.

### Productividad sin micromedición
- Stripe: mide con input (docs de diseño, reviews de API) y output (features liberadas con adopción), no con lines of code ni commits.
- El staff engineering se mide por impacto (sistemas mejorados, velocidad del equipo), no por tickets cerrados.

**Fuentes:**
- https://ctoframework.com/tech/development/boy-scout-rule
- https://viprasol.com/blog/engineering-metrics-dora
- https://koalr.com/blog/trunk-based-development-dora
- https://martinfowler.com/bliki/DesignStaminaHypothesis.html
- https://newsletter.pragmaticengineer.com/p/stripe
- https://shopify.engineering/shopify-monolith

---

## 10. Cultura de equipos de ingeniería (referencias grandes)

| Empresa | Práctica distintiva |
|---|---|
| **Netflix** | Sin perf reviews formales: feedback continuo + "Keeper Test" + 360 anual. "Sunshining" (admitir errores públicamente para aprendizaje colectivo). Freedom & responsibility + radical candor ("no brilliant jerks"). Deploys constantes basados en datos. |
| **Uber** | ERDs (antes RFCs/DUCKs): planning formal que escala de pocos a miles de ingenieros, tiered por criticality. uReview: GenAI code review en ~90% de ~65K cambios semanales. |
| **Shopify** | Modular monolith de 2.8M líneas Ruby con boundaries forzados por Packwerk en compile-time; extrajo solo 2 servicios (storefront read-only y credit-card vaulting). |
| **Stripe** | Revisión estricta de APIs (contratos de larga vida) y productividad medida por outcomes, no métricas vanity. |
| **Google** | Design docs obligatorios con templates que fuerzan security/privacy/i18n/storage reviews; g3doc (docs junto al código); eng-practices como estándar de review; SWE book. |
| **Apple / otros** | MDN Web Docs = referencia normativa del web platform. O'Reilly/Manning publican los libros canónicos (Clean Code, Refactoring, SWE at Google, Staff Engineer's Path, The Software Engineer's Guidebook). |

### Fuentes
- https://newsletter.pragmaticengineer.com/p/netflix
- https://launchdarkly.com/blog/secrets-of-netflixs-engineering-culture
- https://www.performyard.com/articles/netflix-company-culture
- https://eng.uber.com/
- https://shopify.engineering
- https://developer.mozilla.org (MDN)
- https://abseil.io/resources/swe-book/html/ch10.html

---

## 11. 12-Factor App (cloud-native best practices)

Metodología de Adam Wiggins (Heroku, 2011) para aplicaciones SaaS portables, escalables y deployables en cualquier cloud. Sigue vigente en 2026 como base del desarrollo cloud-native.

| # | Factor | Regla |
|---|---|---|
| I | **Codebase** | Un codebase en control de versiones, muchos deploys (dev/staging/prod difieren solo por config) |
| II | **Dependencies** | Declarar explícitamente (manifest: package.json, requirements.txt, Cargo.toml) y aislar; nunca confiar en paquetes globals del sistema |
| III | **Config** | Config en environment variables, nunca en el código ("works on my machine" = anti-pattern) |
| IV | **Backing services** | Tratar DBs, caches, colas como recursos attachables via locator URI, no como parte del código |
| V | **Build, Release, Run** | Separar estrictamente las 3 etapas; el release es inmutable (codebase + config) |
| VI | **Processes** | Procesos stateless y share-nothing; el estado vive en el backing service |
| VII | **Port binding** | Self-contained: exponerse por puerto, no embebido en un servidor external |
| VIII | **Concurrency** | Escalar por proceso (horizontal), no por threads gigantes |
| IX | **Disposability** | Startup rápido + graceful shutdown ante señales de terminación |
| X | **Dev/Prod parity** | Mantener dev/staging/prod lo más parecidos posible (mismas deps y tools) |
| XI | **Logs** | Logs como event streams a stdout; el platform los enruta (no archivos manageados por la app) |
| XII | **Admin processes** | Tareas admin (migrations, backfill) como one-off processes con el mismo codebase+config |

**Los 5 factores que más importan en la práctica** (orden de impacto/seguridad):
1. Config en env vars (Factor III) — el error #1 de seguridad y deploy.
2. Procesos stateless (Factor VI) — base del scaling horizontal.
3. Dependencias declaradas (Factor II) — builds reproducibles.
4. Logs a stdout (Factor XI) — compatible con todas las plataformas.
5. Dev/Prod parity (Factor X) — mata el "works on my machine".

**Anti-patterns**: multi-repo por app, hardcodear secrets, logs en archivos locales, tasks admin dentro del serve.

**Fuentes:**
- https://12factor.net (referencia original)
- https://www.redhat.com/en/blog/12-factor-app
- https://en.wikipedia.org/wiki/Twelve-Factor_App_methodology
- https://codelit.io/blog/twelve-factor-app-explained

---

## 12. CheckList del ingeniero de software (resumen accionable)

### Fase DEFINE (Definir)
- [ ] Requisitos claros, escritos y con criterios de aceptación testables.
- [ ] Problema validado: ¿es el real? ¿por qué ahora? ¿cómo mediremos el éxito?
- [ ] Requisitos ambiguos → preguntas, no suposiciones.
- [ ] Non-goals explícitas (qué NO hace el feature).

### Fase DESIGN (Diseñar)
- [ ] Design doc o RFC escrito para proyectos de tamaño no trivial (goals medibles, arquitectura, APIs, alternativas consideradas, riesgos).
- [ ] Los boundaries entre capas/contextos están definidos (qué puede llamar a qué).
- [ ] Se respetaron principios SOLID y DDD essentials si aplica (sin over-engineering del día 1).
- [ ] ADR registrado para decisiones de arquitectura significativas.
- [ ] Feedback de pares sobre el diseño ANTES de escribir código ("talk first, code later").

### Fase IMPLEMENT (Implementar)
- [ ] TDD: Red → Green → Refactor.
- [ ] Nombres que revelan intención; funciones de una sola responsabilidad.
- [ ] Sin "clever code": lo más legible gana.
- [ ] Config en env vars, nunca secrets en el código (12-Factor III).
- [ ] Sin deuda no registrada: si hay shortcut, está explicitado en el PR o en un ADR/ticket.

### Fase TEST (Probar)
- [ ] Unit tests para la lógica de negocio sin infraestructura externa.
- [ ] Integration tests con deps reales (DB testcontainers, etc.).
- [ ] E2E de los flujos críticos (10%).
- [ ] Edge cases cubiertos (vacíos, nulos, límites, unicode, concurrencia).
- [ ] Property-based en lógica de alto valor; fuzzing en parsers/input no confiable.
- [ ] Mutation testing ≥ 70% en el código nuevo.
- [ ] Cobertura ≥ 80% en módulos críticos (coverage ≠ verificación).

### Fase REVIEW (Revisar)
- [ ] CL pequeño (<400 líneas), descripción de calidad: enfoque + cómo se probó.
- [ ] Code review por un humano además de AI review (auth/billing/security/data siempre humanos).
- [ ] Responder a todos los comentarios; explicar en desacuerdos.
- [ ] QA pre-merge: CI verde con linters + static analysis + tests + gates.
- [ ] Shift-left: errores detectados baratos, en dev/PR, no en prod.

### Fase RELEASE (Publicar)
- [ ] Feature flags / rollout incremental (canary) para features de riesgo.
- [ ] Rollback plan listo antes de deploy.
- [ ] Gestión de release: tag + changelog; versionado semántico respetado.
- [ ] Deploys frecuentes y pequeños (TBD: ramas de 1-2 días = batch size pequeño).

### Fase MAINTAIN (Mantener)
- [ ] Observabilidad: logs a stdout, métricas, alertas accionables en producción.
- [ ] Lección de cualquier incidente: postmortem blameless documentado y publicado.
- [ ] Deuda técnica registrada en backlog con etiqueta, no en comentarios.
- [ ] Boy scout rule: cada archivo que toco quedó un poco mejor.
- [ ] Docs/ADRs actualizados si el sistema aún no shippeó; enmiendas enlazadas si ya vive.
- [ ] DORA: medir deployment frequency, lead time, change failure rate, MTTR.

---

> Todas las URL citadas por sección están al final de cada una. Referencias canónicas: Google eng-practices, Fowler/Martin (Clean Code, Refactoring), 12factor.net, DORA, MDN, refactoring.guru, blogs de Stripe/Netflix/Uber/Shopify.