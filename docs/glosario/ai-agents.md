---
type: glossary-entry
status: stable
tags: [vantadb, glosario, ia, agentes, caso-de-uso]
last_refined: 2026-06
links: "[[README.md]]"
---
#AI Agents

##Definition

**AI agents** are autonomous systems based on language models that can perceive their environment, make decisions and execute actions to achieve specific objectives, maintaining state and context over time.

## Key Features

| Característica | Descripción |
|---------------|-------------|
| **Autonomía** | Operan sin intervención humana constante |
| **Persistencia** | Mantienen estado entre sesiones |
| **Contexto** | Recuerdan interacciones previas |
| **Herramientas** | Usan APIs y funciones externas |
| **Razonamiento** | Planifican y ejecutan multi-step |

## Architecture of a Memory Agent

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

## Types of Memory in Agents

### 1. Episodic Memory

Memories of specific interactions:

```python
db.put(
    key=f"conversation_{timestamp}",
    text="Usuario pidió resumen del proyecto",
    vector=embed("resumen proyecto"),
    metadata={"type": "episodic", "session": "abc123"}
)
```

### 2. Semantic Memory

General knowledge and facts:

```python
db.put(
    key="fact_python_version",
    text="Python 3.12 fue lanzado en octubre 2023",
    vector=embed("python versión lanzamiento"),
    metadata={"type": "semantic", "category": "programming"}
)
```

### 3. Procedural Memory

How to do things:

```python
db.put(
    key="procedure_deploy",
    text="1. Run tests 2. Build image 3. Push to registry 4. Deploy",
    vector=embed("deploy procedure steps"),
    metadata={"type": "procedural", "domain": "devops"}
)
```

## Popular Frameworks

###LangChain

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

###CrewAI

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

## Usage Patterns with VantaDB

### 1. Persistent Context

```python
# Antes de cada interacción
context = db.search(
    vector=embed(user_message),
    top_k=5,
    filter={"session": current_session}
)

# Inject context at prompt
prompt = f"""
Previous context:
{format_memories(context)}

Current message: {user_message}
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

# Apply preference in future responses
prefs = db.search(vector=embed("user preferences"), top_k=3)
response_style = "concise" if has_brevity_pref(prefs) else "detailed"
```

### 3. Reflection and Self-improvement

```python
# Después de cada tarea
db.put(
    key=f"reflection_{task_id}",
    text=f"Tarea: {task}. Resultado: {outcome}. Lección: {lesson}",
    vector=embed(f"{task} {outcome} {lesson}"),
    metadata={"type": "reflection", "success": was_successful}
)

# Before similar tasks
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

## See Also

- [[rag]] — Retrieval-Augmented Generation
- [[graphrag]] — Rich context with graphs
- [[mcp]] — Agent communication protocol
- [[embedded]] — Local-first architecture

---

*VantaDB is the persistent memory layer designed specifically for autonomous AI agents.*

