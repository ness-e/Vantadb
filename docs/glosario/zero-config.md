---
type: glosario-entry
status: stable
tags: [concepto, ux, developer-experience, zero-config]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Zero Configuration, Sin Configuración, Cero Config]
description: "Principio de diseño donde el software funciona correctamente inmediatamente después de la instalación, sin requerir archivos de configuración ni pasos de setup manual"
---

# Zero-Config

## Definición

**Zero-Config** (cero configuración) es un principio de diseño donde el software funciona correctamente **inmediatamente después de la instalación**, sin requerir archivos de configuración, variables de entorno, servicios externos ni pasos de setup manual.

## Características de un Sistema Zero-Config

| Característica | Descripción |
|---------------|-------------|
| **Defaults inteligentes** | Valores por defecto que funcionan para el 90% de los casos |
| **Auto-detección** | Detecta hardware, SO y recursos disponibles |
| **Sin archivos .conf** | No requiere editar YAML, TOML, JSON de configuración |
| **Sin dependencias externas** | No necesita Redis, PostgreSQL, ni servicios adicionales |
| **Funciona out-of-the-box** | `pip install` → `import` → funciona |

## Por Qué Importa en VantaDB

VantaDB compite contra alternativas que requieren infraestructura compleja:

### Comparación de Setup

**Pinecone (Cloud Vector DB):**
```bash
1. Crear cuenta en pinecone.io
2. Verificar email
3. Crear proyecto
4. Generar API key
5. Configurar variables de entorno
6. pip install pinecone-client
7. Inicializar cliente con API key
8. Crear índice (esperar provisioning)
9. Empezar a usar
```

**VantaDB (Zero-Config):**
```python
pip install vantadb-py
```

```python
from vantadb import VantaEmbedded

db = VantaEmbedded("./mi_memoria")
db.put("doc1", vector=[0.1, 0.2, ...], text="Hola mundo")
results = db.search(vector=[0.1, 0.2, ...], top_k=10)
```

**Eso es todo.** Sin cuentas, sin API keys, sin provisioning.

## Implementación Técnica en VantaDB

| Aspecto | Decisión Zero-Config |
|---------|---------------------|
| **Backend** | [Fjall](Fjall.md) por defecto (no requiere instalación de C++) |
| **Índice vectorial** | [HNSW](HNSW.md) con parámetros auto-tuneados según dataset size |
| **Tokenizador** | BM25 con defaults razonables (lowercase, sin stopwords) |
| **Persistencia** | Directorio local, sin configuración de conexión |
| **Concurrencia** | [File Locking](File Locking.md) automático al abrir |
| **Memoria** | Detección automática de RAM disponible |

## Ejemplo: Experiencia Zero-Config Completa

```python
# Instalación
# pip install vantadb-py

from vantadb import VantaEmbedded

# 1. Crear instancia (sin configuración)
db = VantaEmbedded("./agent_memory")

# 2. Guardar memoria (sin schema previo)
db.put(
    key="conversation_001",
    vector=[0.12, -0.34, 0.56, ...],  # 384 dimensiones
    text="El usuario prefiere respuestas concisas",
    metadata={
        "timestamp": "2026-06-12T10:30:00Z",
        "user_id": "user_123",
        "confidence": 0.95
    }
)

# 3. Buscar (sin configurar índices)
results = db.search(
    vector=[0.11, -0.33, 0.55, ...],
    top_k=5,
    filter={"user_id": "user_123"}
)

# 4. Todo funciona. Sin configuración.
```

## Trade-offs del Zero-Config

| Ventaja | Costo |
|---------|-------|
| Onboarding instantáneo | Menos control fino para power users |
| Sin errores de configuración | Defaults pueden no ser óptimos para casos edge |
| Ideal para prototipos | Producción puede requerir tuning posterior |

### Escape Hatch: Configuración Cuando se Necesita

VantaDB permite configuración avanzada **cuando es necesaria**, pero no la exige:

```python
# Zero-config (default)
db = VantaEmbedded("./data")

# Configuración avanzada (opcional)
db = VantaEmbedded(
    "./data",
    config={
        "hnsw": {"M": 32, "ef_construction": 400},
        "wal": {"sync_mode": "always"},
        "memory_limit_mb": 4096
    }
)
```

## Zero-Config vs "Easy-Config"

| Enfoque | Ejemplo | Problema |
|---------|---------|----------|
| **Zero-Config** | SQLite, VantaDB | Ninguno |
| **Easy-Config** | MongoDB (defaults razonables) | Aún requiere `mongod` corriendo |
| **Config-Heavy** | PostgreSQL, Elasticsearch | Requiere DBA o DevOps |

## Véase También

- [Embebido](Embebido.md) — Habilita zero-config al no requerir servidor
- [Local-First](Local-First.md) — Filosofía compatible con zero-config
- [Fjall](Fjall.md) — Backend que no requiere instalación de dependencias C++

---

*Zero-config no es pereza de diseño, es respeto por el tiempo del desarrollador.*

