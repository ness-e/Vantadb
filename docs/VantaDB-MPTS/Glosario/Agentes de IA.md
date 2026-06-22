---
type: glossary-entry
status: stable
tags: [vantadb, glosario, ia, agentes, caso-de-uso]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# Agentes de IA

## Definición

**Agentes de IA** son sistemas autónomos basados en modelos de lenguaje que pueden percibir su entorno, tomar decisiones y ejecutar acciones para alcanzar objetivos específicos, manteniendo estado y contexto a lo largo del tiempo.

## Características Clave

| Característica | Descripción |
|---------------|-------------|
| **Autonomía** | Operan sin intervención humana constante |
| **Persistencia** | Mantienen estado entre sesiones |
| **Contexto** | Recuerdan interacciones previas |
| **Herramientas** | Usan APIs y funciones externas |
| **Razonamiento** | Planifican y ejecutan multi-step |

## Arquitectura de un Agente con Memoria

```
┌─────────────────────────────────────────────────────────────┐
│                      Agente de IA                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   LLM       │  │  Planner    │  │   Tool Executor     │ │
│  │ (Reasoning) │  │  (Strategy) │  │   (Action)          │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                    │            │
│         └────────────────┼────────────────────┘            │
│                          │                                  │
│                          ▼                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Memory Layer (VantaDB)                   │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────┐  │  │
│  │  │ Episodic   │  │ Semantic   │  │ Procedural     │  │  │
│  │  │ Memory     │  │ Memory     │  │ Memory         │  │  │
│  │  └────────────┘  └────────────┘  └────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Tipos de Memoria en Agentes

### 1. Memoria Episódica

Recuerdos de interacciones específicas:

```python
db.put(
    key=f"conversation_{timestamp}",
    text="Usuario pidió resumen del proyecto",
    vector=embed("resumen proyecto"),
    metadata={"type": "episodic", "session": "abc123"}
)
```

### 2. Memoria Semántica

Conocimiento general y hechos:

```python
db.put(
    key="fact_python_version",
    text="Python 3.12 fue lanzado en octubre 2023",
    vector=embed("python versión lanzamiento"),
    metadata={"type": "semantic", "category": "programming"}
)
```

### 3. Memoria Procedural

Cómo hacer cosas:

```python
db.put(
    key="procedure_deploy",
    text="1. Run tests 2. Build image 3. Push to registry 4. Deploy",
    vector=embed("deploy procedure steps"),
    metadata={"type": "procedural", "domain": "devops"}
)
```

## Frameworks Populares

### LangChain

```python
from langchain.agents import initialize_agent
from langchain_vantadb import VantaDBMemory

memory = VantaDBMemory(path="./agent_memory")
agent = initialize_agent(
    tools=tools,
    llm=llm,
    memory=memory,
    agent="conversational-react-description"
)
```

### CrewAI

```python
from crewai import Agent
from vantadb import VantaEmbedded

db = VantaEmbedded("./crew_memory")

researcher = Agent(
    role="Researcher",
    memory_backend=db,
    long_term_memory=True
)
```

### AutoGen

```python
from autogen import AssistantAgent
from vantadb_autogen import VantaDBMemoryManager

memory = VantaDBMemoryManager("./autogen_memory")
assistant = AssistantAgent(
    name="assistant",
    memory_manager=memory
)
```

## Patrones de Uso con VantaDB

### 1. Contexto Persistente

```python
# Antes de cada interacción
context = db.search(
    vector=embed(user_message),
    top_k=5,
    filter={"session": current_session}
)

# Inyectar contexto en prompt
prompt = f"""
Contexto previo:
{format_memories(context)}

Mensaje actual: {user_message}
"""
```

### 2. Aprendizaje de Preferencias

```python
# Detectar preferencia del usuario
if "prefiero respuestas cortas" in user_message:
    db.put(
        key="user_preference_brevity",
        text="Usuario prefiere respuestas concisas",
        vector=embed("preferencia brevedad"),
        metadata={"type": "preference", "confidence": 0.95}
    )

# Aplicar preferencia en futuras respuestas
prefs = db.search(vector=embed("preferencias usuario"), top_k=3)
response_style = "concise" if has_brevity_pref(prefs) else "detailed"
```

### 3. Reflexión y Auto-mejora

```python
# Después de cada tarea
db.put(
    key=f"reflection_{task_id}",
    text=f"Tarea: {task}. Resultado: {outcome}. Lección: {lesson}",
    vector=embed(f"{task} {outcome} {lesson}"),
    metadata={"type": "reflection", "success": was_successful}
)

# Antes de tareas similares
past_reflections = db.search(
    vector=embed(new_task),
    filter={"type": "reflection"},
    top_k=3
)
```

## Métricas de Éxito

| Métrica | Sin Memoria | Con VantaDB |
|---------|-------------|-------------|
| **Coherencia conversacional** | Baja | Alta |
| **Repetición de preguntas** | Frecuente | Rara |
| **Personalización** | Ninguna | Adaptativa |
| **Aprendizaje** | Ninguno | Continuo |

## Véase También

- [RAG](RAG.md) — Retrieval-Augmented Generation
- [GraphRAG](GraphRAG.md) — Contexto enriquecido con grafos
- [MCP](MCP.md) — Protocolo de comunicación con agentes
- [Embebido](Embebido.md) — Arquitectura local-first

---

*VantaDB es la capa de memoria persistente diseñada específicamente para agentes de IA autónomos.*

