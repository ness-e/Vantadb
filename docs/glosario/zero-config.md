---
type: glossary-entry
status: stable
tags: [concept, ux, developer-experience, zero-config]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Zero Configuration, No Configuration, Zero Config]
description: "Design principle where the software works correctly immediately after installation, without requiring configuration files or manual setup steps"
---
#Zero-Config

##Definition

**Zero-Config** is a design principle where software works correctly **immediately after installation**, without requiring configuration files, environment variables, external services, or manual setup steps.

## Characteristics of a Zero-Config System

| Característica | Descripción |
|---------------|-------------|
| **Defaults inteligentes** | Valores por defecto que funcionan para el 90% de los casos |
| **Auto-detección** | Detecta hardware, SO y recursos disponibles |
| **Sin archivos .conf** | No requiere editar YAML, TOML, JSON de configuración |
| **Sin dependencias externas** | No necesita Redis, PostgreSQL, ni servicios adicionales |
| **Funciona out-of-the-box** | `pip install` → `import` → funciona |

##Why it Matters in VantaDB

VantaDB competes against alternatives that require complex infrastructure:

### Setup Comparison

**Pinecone (Cloud Vector DB):**
```bash
1. Create an account on pinecone.io
2. Verify email
3. Create project
4. Generate API key
5. Configure environment variables
6. pip install pinecone-client
7. Initialize client with API key
8. Create index (wait provisioning)
9. Start using
```

**VantaDB (Zero-Config):**
```python
pip install vantadb-py
```

```python
from vantadb import VantaEmbedded

db = VantaEmbedded("./my_memory")
db.put("doc1", vector=[0.1, 0.2, ...], text="Hello world")
results = db.search(vector=[0.1, 0.2, ...], top_k=10)
```

**That's all.** No accounts, no API keys, no provisioning.

## Technical Implementation in VantaDB

| Aspecto | Decisión Zero-Config |
|---------|---------------------|
| **Backend** | [[fjall]] por defecto (no requiere instalación de C++) |
| **Índice vectorial** | [[hnsw]] con parámetros auto-tuneados según dataset size |
| **Tokenizador** | BM25 con defaults razonables (lowercase, sin stopwords) |
| **Persistencia** | Directorio local, sin configuración de conexión |
| **Concurrencia** | [[file-locking]] automático al abrir |
| **Memoria** | Detección automática de RAM disponible |

## Example: Complete Zero-Config Experience

```python
# Instalación
# pip install vantadb-py

from vantadb import VantaEmbedded

#1. Create instance (without configuration)
db = VantaEmbedded("./agent_memory")

#2. Save memory (without prior schema)
db.put(
    key="conversation_001",
    vector=[0.12, -0.34, 0.56, ...], # 384 dimensions
    text="The user prefers concise answers",
    metadata={
        "timestamp": "2026-06-12T10:30:00Z",
        "user_id": "user_123",
        "confidence": 0.95
    }
)

#3. Search (without setting indexes)
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    top_k=5,
    filter={"user_id": "user_123"}
)

#4. Everything works. No configuration.
```

## Zero-Config Trade-offs

| Ventaja | Costo |
|---------|-------|
| Onboarding instantáneo | Menos control fino para power users |
| Sin errores de configuración | Defaults pueden no ser óptimos para casos edge |
| Ideal para prototipos | Producción puede requerir tuning posterior |

### Escape Hatch: Configuration When Needed

VantaDB allows advanced configuration **when necessary**, but does not require it:

```python
# Zero-config (default)
db = VantaEmbedded("./data")

# Advanced settings (optional)
db = VantaEmbedded(
    "./data",
    config={
        "hnsw": {"M": 32, "ef_construction": 400},
        "wal": {"sync_mode": "always"},
        "memory_limit_mb": 4096
    }
)
```

##Zero-Config vs "Easy-Config"

| Enfoque | Ejemplo | Problema |
|---------|---------|----------|
| **Zero-Config** | SQLite, VantaDB | Ninguno |
| **Easy-Config** | MongoDB (defaults razonables) | Aún requiere `mongod` corriendo |
| **Config-Heavy** | PostgreSQL, Elasticsearch | Requiere DBA o DevOps |

## See Also

- [[embedded]] — Habilita zero-config al no requerir servidor
- [[local-first]] — Filosofía compatible con zero-config
- [[fjall]] — Backend que no requiere instalación de dependencias C++

---

*Zero-config is not design laziness, it is respect for the developer's time.*

