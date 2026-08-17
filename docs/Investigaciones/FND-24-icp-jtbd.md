# FND-24 — ICP + JTBD con evidencia de usuarios reales

> **Tarea:** FND-24 (Backlog.md:518, P20d, prio 🟢)
> **Fecha:** 2026-08-16
> **Estado:** investigación completada — veredicto: **sin evidencia de usuarios reales; ICP/JTBD definidos como hipótesis con plan de validación**
> **Rol:** vanta-docs

---

## 1. Resumen ejecutivo

La pregunta de FND-24 es doble:

1. **¿Para quién es VantaDB?** ¿dev de chatbot local o de edge computing?
2. **¿Qué job cumple?** ¿por qué elegir VantaDB sobre SQLite + extensión vectorial?

**Veredicto honesto:** el repositorio no contiene evidencia de *usuarios reales externos* usando el producto. Los únicos claims sobre el "usuario objetivo" provienen de documentos de posicionamiento escritos por el creador (README, VISION, GO_TO_MARKET, SHOW_HN_PREP, PILOT_PROGRAM) — es decir, **hipótesis del autor, no datos de mercado**. Las métricas disponibles indican adopción ~cero:

| Métrica | Valor | Fuente |
|---|---|---|
| GitHub stars | ⭐ 2 | `docs/Investigaciones/investigacion-equipo-2026-08-09.md:119` |
| GitHub forks | 0 | `investigacion-equipo-2026-08-09.md:119` |
| crates.io descargas | 32 | `investigacion-equipo-2026-08-09.md:120` |
| crates.io dependents | **0** | `investigacion-equipo-2026-08-09.md:120` |
| Show HN | No ocurrió aún (planificado sept 2026) | `docs/strategy/SHOW_HN_PREP.md`, `VantaDB_Manual_Estrategico_Unificado.md:612` |

> **Implicación operativa:** todo lo que sigue son **hipótesis** que el Show HN + pilot program deben confirmar o descartar. No hay entrevistas, testimonios, ni issues de usuarios que citar. No se inventa evidencia donde no existe (regla explícita de FND-24).

---

## 2. Evidencia dura de no-adopción (lo único verificable hoy)

| Claim | Evidencia (fuente local) | Confianza |
|---|---|---|
| VantaDB está publicado pero sin distribución | GitHub `@ness-e/Vantadb` ⭐ 2, 0 forks, 1381 commits (`investigacion-equipo-2026-08-09.md:119`) | alta |
| Adopción en crates.io ~cero | 32 descargas, **0 dependents** (`investigacion-equipo-2026-08-09.md:120`) | alta |
| "NADIE ha encontrado el proyecto orgánicamente" | Diagnóstico externo: SEO inexistente, contenido inexistente, comunidad inexistente (`VantaDB_Manual_Estrategico_Unificado.md:1043`) | alta |
| El producto NO ha sido validado con usuarios reales | "¿Ya tienes algún usuario real (aunque sea uno)?… o el producto ha sido validado hasta ahora solo internamente" (`VantaDB_Manual_Estrategico_Unificado.md:772`) | alta |
| El Show HN es la primera oportunidad real de exposición | "Show HN es tu primera oportunidad real de exposición" (`VantaDB_Manual_Estrategico_Unificado.md:1043`) | alta |

**Conclusión:** no existe evidencia de usuarios reales en el repo. Cualquier definición de ICP/JTBD debe tratarse como **hipótesis de diseño**, no como hallazgo de mercado.

---

## 3. ICP (Ideal Customer Profile) — HIPÓTESIS

Derivado de los documentos de posicionamiento. Cada perfil se etiqueta con su fuente; **ninguno tiene evidencia de usuarios reales**.

### 3.1 ICP primario hipotético: AI Agent Developer local-first

| Atributo | Valor | Fuente |
|---|---|---|
| **Rol** | ML/AI engineer construyendo agentes autónomos | `docs/vision/VISION.md:54` |
| **Stack** | Python + LangChain/LlamaIndex/CrewAI/LangGraph + Ollama (LLM local) | `VISION.md:56`, `GO_TO_MARKET.md:166` |
| **Dolor** | "Mi agente olvida todo entre sesiones"; ChromaDB pierde datos en crash; PostgresSaver caro en prod | `VISION.md:61-64`, `GO_TO_MARKET.md:167` |
| **Alternativa actual** | ChromaDB (memoria) + SQLite (metadata); InMemorySaver dev / PostgresSaver prod | `GO_TO_MARKET.md:167-172` |
| **Por qué VantaDB (hipótesis)** | Memoria persistente WAL + hybrid search nativo en un pip install, sin servidor | `README.md:34`, `SHOW_HN_PREP.md:27` |
| **Peso de mercado** | 🟠 HIGH (vertical 2 de GTM) | `GO_TO_MARKET.md:159-172` |

### 3.2 ICP secundario hipotético: Local LLM Stack dev (privacy-first)

| Atributo | Valor | Fuente |
|---|---|---|
| **Rol** | Devs/enthusiasts corriendo LLMs 100% locales | `GO_TO_MARKET.md:146` |
| **Stack** | Ollama + AnythingLLM + vector DB | `GO_TO_MARKET.md:151` |
| **Dolor** | LanceDB default no tiene BM25/graphs; no pueden mandar datos a cloud | `GO_TO_MARKET.md:152` |
| **Alternativa actual** | LanceDB (default de AnythingLLM) | `GO_TO_MARKET.md:152` |
| **Peso de mercado** | 🟠 HIGH (vertical 1 de GTM) | `GO_TO_MARKET.md:144-157` |

### 3.3 ICP terciario hipotético: AI-IDE Tooling dev

| Atributo | Valor | Fuente |
|---|---|---|
| **Rol** | Devs usando Claude Code / Cursor / Windsurf / Cline | `GO_TO_MARKET.md:176` |
| **Dolor** | IDE pierde contexto entre sesiones; CLAUDE.md no resuelve historial/búsqueda/aislamiento | `GO_TO_MARKET.md:181-182` |
| **Alternativa actual** | CLAUDE.md + claude-mem (SQLite, 89K★) | `GO_TO_MARKET.md:181` |
| **Por qué VantaDB (hipótesis)** | MCP server ya implementado; hybrid search + per-project isolation | `GO_TO_MARKET.md:183-184, 189` |
| **Peso de mercado** | 🟠 HIGH (vertical 3) — canal de distribución estratégico vía MCP | `GO_TO_MARKET.md:174-189` |

### 3.4 Early Adopter hipotético (pilot program — el más cercano a evidencia operativa)

El único documento que describe criterios *seleccionables* de perfil es el PILOT_PROGRAM:

| Criterio | Valor | Fuente |
|---|---|---|
| Equipos construyendo **local-first AI agents** con ≥1 de: memoria durable, fricción de compilación C++, latencia <50ms, requisito local-first | `docs/operations/PILOT_PROGRAM.md:38-43` | hipótesis |
| Must-have: proyecto activo con embeddings/RAG + disponibilidad para feedback | `PILOT_PROGRAM.md:47-48` | hipótesis |
| Nice-to-have: ya probó Chroma/FAISS/LanceDB/Qdrant; Windows/ARM macOS; multi-usuario | `PILOT_PROGRAM.md:53-55` | hipótesis |

> Este es el **único ICP con criterios falsables** y con infraestructura de reclutamiento lista (`PILOT_PROGRAM.md` §5). Debería ser la base del plan de validación (§6).

### 3.5 Perfil de entrada implícito del Show HN

El claim del contexto de la tarea —"un dev de Show HN que quiere probar con un comando"— se corresponde con:

| Señal | Fuente |
|---|---|
| Onboarding one-liner: `pip install vantadb-py` | `README.md:60-67` |
| Quickstart de 5 minutos con CRUD + hybrid search | `README.md:83-123` |
| CLI one-line install (`curl ... \| sh` / `irm ... \| iex`) | `README.md:238-265` |
| "Time-to-first-query under 2 minutes" (target, hoy ~3 min) | `VISION.md:46, 222` |
| Show HN draft pide feedback sobre "how you manage local memory in your agent pipelines" | `SHOW_HN_PREP.md:77` |

**Hipótesis:** el primer usuario real de VantaDB será un dev individual (no un equipo) que llega del Show HN, instala con un comando, y evalúa en <15 min si resuelve memoria + hybrid search sin servidor. **No hay evidencia que lo confirme aún.**

---

## 4. JTBD (Jobs-to-be-Done) — HIPÓTESIS

Jobs funcionales, emocionales y sociales. Todos son hipótesis del autor (etiqueta `hipótesis: sin evidencia de usuario` salvo indicación).

### 4.1 Jobs funcionales

| # | Job | Contexto ("cuando…") | Alternativa actual que "contrata" | Fuente |
|---|---|---|---|---|
| J1 | **Persistir la memoria de un agente entre sesiones** sin perderla en crash/restart | Cuando construyo un agente local que debe recordar conversaciones, prefiero que sobreviva a reinicios | In-memory FAISS/Chroma (pierde datos); SQLite+vector (fricción C++) | `SHOW_HN_PREP.md:30-34`, `why_i_built.md:21-23` |
| J2 | **Buscar contexto híbrido (semántico + keyword exacta)** en un solo call, sin fusionar yo dos índices | Cuando el agente necesita encontrar memorias por significado Y por términos exactos (nombres, IDs, API keys) | SQLite FTS5 + vector extension con join manual; Chroma sin BM25 | `why_i_built.md:36`, `SHOW_HN_PREP.md:92` |
| J3 | **Obtener persistencia + búsqueda sin servidor ni infraestructura** (local-first, offline-capable) | Cuando mi app de agente corre en el dispositivo/edge y no puedo depender de cloud | Cloud VDB (Pinecone/Qdrant): latencia 100-200ms, egress, no offline | `why_i_built.md:44-48`, `SHOW_HN_PREP.md:32` |
| J4 | **Instalar y arrancar en un comando** sin compilar C++ ni configurar servicios | Cuando quiero probar rápido un motor embebido en Python | `pip install sqlite-vec` (binarios C++ por OS); Docker/Qdrant | `SHOW_HN_PREP.md:93`, `README.md:60` |
| J5 | **Actualizar doc + embedding + metadata atómicamente** (transaccionalidad multi-modelo) | Cuando quiero coherencia entre el record canónico y sus índices derivados | Multi-DB (Pinecone + Postgres + Neo4j) sin txn atómica | `VISION.md:26-33` |

### 4.2 Jobs emocionales

| # | Job | Fuente |
|---|---|---|
| E1 | **Sentir que mis datos no salen de mi máquina** (privacidad por diseño) | `VISION.md:57`, `GO_TO_MARKET.md:146`, `why_i_built.md:47` |
| E2 | **Sentir confianza en la durabilidad** ("no pierdo memoria si el proceso crashea") | `SHOW_HN_PREP.md:137-145` (WAL + chaos testing como respuesta al escepticismo) |
| E3 | **Evitar la ansiedad de elegir/operar infraestructura** (zero-config, un solo motor) | `VISION.md:31` ("Operational complexity: managing 3-4 databases"), `README.md:34` |

### 4.3 Jobs sociales

| # | Job | Fuente |
|---|---|---|
| S1 | **Usar una herramienta con credibilidad técnica** en la comunidad (Rust, benchmarks reproducibles, Apache-2.0) para poder defender la elección ante pares | `SHOW_HN_PREP.md:77` (pide feedback de arquitectura), `FND-13` (benchmarks honestos) — hipótesis |
| S2 | **Estar en la ola local-first / privacy-first** frente a la narrativa cloud-only | `why_i_built.md:19`, `GO_TO_MARKET.md:142` — hipótesis |

---

## 5. Tabla de evidencia claim → fuente

| Claim | Fuente (local, verificada por lectura) | Etiqueta |
|---|---|---|
| VantaDB es para AI agents, RAG local y edge apps | `README.md:34` | `hipótesis: claim de posicionamiento del creador` |
| ICP primario = AI Agent Developer (LangChain/LlamaIndex/CrewAI) | `docs/vision/VISION.md:52-58` | `hipótesis: claim del creador` |
| Pains citados ("olvida entre sesiones", "ChromaDB perdió datos") | `VISION.md:61-64` | `hipótesis: sin evidencia de usuario — pains presumidos` |
| 3 verticales de mercado (Local LLM, Agentic, IDE) | `docs/strategy/GO_TO_MARKET.md:140-191` | `hipótesis: análisis del creador, sin datos de usuario` |
| Audiencia objetivo del Show HN = devs de agentes locales | `docs/strategy/SHOW_HN_PREP.md:23-34` | `hipótesis: audiencia asumida, HN no ocurrió` |
| Early adopter profile seleccionable | `docs/operations/PILOT_PROGRAM.md:34-61` | `hipótesis: criterios del creador, programa sin participantes confirmados` |
| Perfil de reportador de issues (Python/CLI/server + OS) | `.github/ISSUE_TEMPLATE/bug_report.yml` | `hipótesis: estructura, no datos` |
| **No hay usuarios reales: 2 stars / 0 forks / 32 descargas / 0 dependents** | `docs/Investigaciones/investigacion-equipo-2026-08-09.md:119-124` | `evidencia: dato verificado del repo (2026-08-09)` |
| "NADIE ha encontrado el proyecto orgánicamente" | `VantaDB_Manual_Estrategico_Unificado.md:1043` | `evidencia: diagnóstico externo documentado` |
| Show HN = primera oportunidad real de exposición | `VantaDB_Manual_Estrategico_Unificado.md:1043` | `evidencia: diagnóstico externo documentado` |

---

## 6. Hipótesis sin validar + cómo validarlas

> Regla de FND-24: no inventar usuarios. Estas son las hipótesis pendientes y el plan concreto para convertirlas en evidencia.

| # | Hipótesis sin validar | Método de validación | Instrumento existente | Dueño |
|---|---|---|---|---|
| H1 | El primer usuario será un dev individual que llega del Show HN y prueba con 1 comando | Analizar el thread de Show HN (sept 2026): quiénes comentan, qué prueban, qué piden; medir conversión `pip install` → primera query | `SHOW_HN_PREP.md` (draft + Q&A defensiva) | lead |
| H2 | El dolor dominante es "memoria entre sesiones + hybrid search sin servidor" (J1+J2+J3) | Entrevistas Mom Test (20+) con devs de agentes locales; codificar pains en palabras del usuario | Plantilla de email frío + guía C11 (`VantaDB_Manual_Estrategico_Unificado.md:1018-1024`) | lead |
| H3 | El ICP real es Agentic Frameworks (LangGraph/CrewAI) y NO edge computing | Pilot program con 3-5 design partners; cada uno genera feedback estructurado (3 puntos: qué funcionó/qué se rompió/qué falta) | `PILOT_PROGRAM.md` §7 (cuestionario), §5 (reclutamiento) | lead |
| H4 | SQLite+vector es la alternativa real a batir (no cloud VDB) | Preguntar en entrevistas "¿qué usás hoy?"; medir menciones de sqlite-vec/sqlite-vss vs Pinecone/Chroma | Cuestionario pilot §3 (qualitative) | lead |
| H5 | Early adopters son equipos con problemas de durabilidad/compilación/latencia | Los design partners del pilot que matcheen los criterios de `PILOT_PROGRAM.md:38-43` | `PILOT_PROGRAM.md` KPIs (retention ≥80%, NPS ≥30, 2+ testimonios) | lead |
| H6 | Onboarding <15 min es decisivo | Analytics de docs: % que completa quickstart, % que activa hybrid search, % que abandona en install | `docs/Investigaciones/VantaDB-28-07-2026.md:474-475` (métricas propuestas) | lead |

### Orden recomendado de ejecución (post-Show HN)

1. **Día del Show HN:** capturar el thread completo (comentarios, preguntas, críticas) → primera evidencia real de demanda.
2. **Semana 1-2 post-HN:** entrevistas Mom Test con 5-8 interesados del thread (el cold email template ya existe).
3. **Semana 2-4:** cerrar 3-5 design partners con el PILOT_PROGRAM (criterios ya definidos) y ejecutar cohorte de 8 semanas.
4. **Semana 4-8:** con los exit reports (1 página cada uno) + analytics de docs, actualizar este documento: promover hipótesis confirmadas a evidencia, descartar las refutadas, y decidir el ICP/JTBD definitivo (FND-23 pattern: decisión con evidencia, no intuición).

**Métricas de no-éxito (cuándo pivotar):** si tras 50 outreach no hay 3 design partners → el mensaje/ICP está mal (`VantaDB_Manual_Estrategico_Unificado.md:994`); si <10% de los visitantes del quickstart llegan a una búsqueda híbrida → el onboarding falla (H6).

---

## 7. Veredicto y deuda

- **Veredicto:** FND-24 NO puede entregar "ICP + JTBD con evidencia de usuarios reales" porque la evidencia no existe aún. Se entrega la definición de ICP (4 perfiles hipotéticos) + JTBD (5 funcionales, 3 emocionales, 2 sociales) con tabla claim→fuente y plan de validación accionable. Cumple el contrato en su forma permitida: "todas las filas marcadas hipótesis con plan de validación".
- **Deuda:** este documento debe actualizarse post-Show HN y post-pilot (semana 4-8) con los datos reales; la actualización es candidata a nueva tarea FND (o reapertura de FND-24).
- **Dependencia:** FND-23 (decisión default-on/opt-in con telemetría) comparte la necesidad de evidencia de adopción; H2/H3 alimentan ambas.

## 8. Referencias

- `docs/Backlog.md:518` — FND-24 (fuente de la tarea)
- `docs/Investigaciones/investigacion-equipo-2026-08-09.md` — estado de adopción real
- `docs/strategy/SHOW_HN_PREP.md` — draft Show HN + Q&A defensiva
- `docs/strategy/GO_TO_MARKET.md` — 3 verticales GTM
- `docs/vision/VISION.md` — ICP/UVP/claims
- `docs/operations/PILOT_PROGRAM.md` — programa de pilot (instrumento de validación)
- `VantaDB_Manual_Estrategico_Unificado.md` — plan C11 (design partners), diagnóstico de adopción