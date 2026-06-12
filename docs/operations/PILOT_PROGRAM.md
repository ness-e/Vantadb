# Programa de Pilotos Beta y Guía de Onboarding

Este documento detalla la estrategia de captación (outreach), el proceso de integración rápida (onboarding) y el formulario de feedback técnico para el **Programa de Pilotos de VantaDB**.

---

## 🎯 1. Estrategia de Captación y Comunidades Objetivo

Buscamos de 3 a 5 desarrolladores que construyan **agentes de IA locales** (local-first) y experimenten problemas con la durabilidad de memoria (caídas de datos con FAISS o Chroma en memoria) o fricciones de compilación con extensiones C++.

| Canal | Comunidad | Propósito del Reclutamiento |
|---|---|---|
| **Reddit** | `r/LocalLLaMA` | Desarrolladores construyendo sistemas RAG locales y agentes con Ollama. |
| **Reddit** | `r/rust` | Ingenieros de sistemas interesados en el rendimiento de bases de datos y bindings de PyO3. |
| **Discord** | Servidor de Ollama (`#projects`) | Builders de IA corriendo modelos locales sobre hardware de consumo. |
| **Discord** | LlamaIndex / LangChain | Desarrolladores integrando almacenes vectoriales locales. |

---

## 🛠️ 2. Guía de Onboarding y Configuración Rápida (Ollama)

Esta guía te permite integrar VantaDB como el motor de memoria semántica de un agente de IA en menos de 15 minutos.

### Prerrequisitos
Asegúrate de tener **Ollama** ejecutándose localmente y descarga los modelos requeridos:
```bash
ollama pull nomic-embed-text
ollama pull llama3
```

### Instalación de dependencias
```bash
pip install vantadb-py ollama psutil
```

### Script de Integración (`agent_memory_loop.py`):
```python
import os
import ollama
import vantadb_py

# 1. Inicializar base de datos local
DB_PATH = "./agent_durable_memory"
db = vantadb_py.VantaDB(DB_PATH, distance_metric="cosine")
NAMESPACE = "agent_memories"

def get_local_embedding(text: str) -> list[float]:
    """Genera un vector de 768 dimensiones con el modelo de Ollama."""
    response = ollama.embeddings(model="nomic-embed-text", prompt=text)
    return response["embedding"]

def remember_interaction(key: str, topic: str, content: str):
    """Guarda una interacción conversacional de forma persistente."""
    print(f"\n[Escribiendo en WAL] Key: {key} | Topic: {topic}")
    vector = get_local_embedding(content)
    
    db.put(
        namespace=NAMESPACE,
        key=key,
        vector=vector,
        payload={
            "topic": topic,
            "text": content
        }
    )
    db.flush() # Forzar persistencia física en el disco (fsync)

def query_agent_memory(query_text: str, top_k: int = 2):
    """Ejecuta una búsqueda híbrida nativa (Vectorial HNSW + Léxica BM25) con fusión RRF."""
    print(f"\n[Búsqueda Híbrida] Consulta: '{query_text}'")
    query_vector = get_local_embedding(query_text)
    
    results = db.search_memory(
        namespace=NAMESPACE,
        query_vector=query_vector,
        text_query=query_text,
        top_k=top_k
    )
    return results

if __name__ == "__main__":
    remember_interaction(
        key="mem_01",
        topic="Arquitectura del Motor",
        content="VantaDB usa archivos de diseño de página mapeados en memoria (MMap) compactados secuencialmente en orden BFS para reducir page faults."
    )
    remember_interaction(
        key="mem_02",
        topic="GIL Python",
        content="El wrapper de Python (PyO3) de VantaDB libera el GIL usando allow_threads en las búsquedas para concurrencia real de hilos."
    )

    print("\n[Compresión] Reconstruyendo índice vectorial con layout BFS...")
    db.rebuild_index()

    # Buscar utilizando palabras clave y similitud semántica simultánea
    search_results = query_agent_memory("PyO3 liberar GIL", top_k=2)

    for i, res in enumerate(search_results):
        print(f"Rank {i+1} | Score: {res.score:.4f} | Key: {res.key}")
        print(f"  Topic: {res.payload['topic']}")
        print(f"  Content: {res.payload['text']}\n")

    db.close()
```

---

## 📋 3. Cuestionario de Feedback para Pilotos

Una vez integrado, por favor comparte este cuestionario diligenciado:

1. **Entorno de Desarrollo:**
   - Sistema Operativo (e.g., Windows 11, macOS M2, Ubuntu):
   - CPU (e.g., 8-core Intel i7):
   - Tipo de almacenamiento (e.g., NVMe SSD, SATA SSD):
2. **Métricas de Rendimiento:**
   - Latencia de Ingesta media por `put` (ms):
   - Tiempo de reconstrucción del índice (`rebuild_index`):
   - Latencia de búsqueda (p50 y p95):
3. **Preguntas Cualitativas:**
   - ¿Instaló la rueda de Python a la primera sin advertencias del compilador?
   - ¿La búsqueda híbrida con RRF cubrió tu intención de búsqueda semántica y léxica?
   - ¿Encontraste algún bug, bloqueo de archivos o consumo inusual de memoria?
