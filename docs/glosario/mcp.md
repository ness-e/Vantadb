---
type: glossary-entry
status: stable
tags: [vantadb, glosario, protocolo, ia, agentes]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# MCP (Model Context Protocol)

## Definición

**MCP** (Model Context Protocol) es un protocolo estándar abierto que permite a modelos de lenguaje (LLMs) y agentes de IA interactuar con herramientas y fuentes de datos externas de manera estructurada y segura.

## Especificación

- **Protocolo:** JSON-RPC 2.0
- **Transporte:** stdio o HTTP con Server-Sent Events (SSE)
- **Versión actual:** 2025-11-25

## Arquitectura

```
┌─────────────────────────────────────────────────────┐
│                    AI Assistant                       │
│  (Cursor, Claude Code, Windsurf, etc.)              │
└──────────────────────┬──────────────────────────────┘
                       │ JSON-RPC 2.0
                       ▼
┌─────────────────────────────────────────────────────┐
│                    MCP Server                         │
│  (vantadb-server --mcp)                              │
├─────────────────────────────────────────────────────┤
│  Resources: DB schema, statistics                    │
│  Tools: search, get, put, delete                     │
│  Prompts: Templates for common patterns              │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│                    VantaDB Core                       │
│  (Embedded engine)                                   │
└─────────────────────────────────────────────────────┘
```

## Primitivas MCP

### Resources (Lectura)

Exponen datos de solo lectura al LLM:

```json
{
  "resources": [
    {
      "uri": "vantadb://schema",
      "name": "Database Schema",
      "description": "Current namespace structure"
    },
    {
      "uri": "vantadb://stats",
      "name": "Database Statistics",
      "description": "Record counts, memory usage"
    }
  ]
}
```

### Tools (Acciones)

Funciones que el LLM puede invocar:

```json
{
  "tools": [
    {
      "name": "search_memory",
      "description": "Search for relevant memories",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": {"type": "string"},
          "top_k": {"type": "integer", "default": 10}
        }
      }
    },
    {
      "name": "store_memory",
      "description": "Store a new memory",
      "inputSchema": {
        "type": "object",
        "properties": {
          "key": {"type": "string"},
          "content": {"type": "string"},
          "vector": {"type": "array", "items": {"type": "number"}}
        }
      }
    }
  ]
}
```

### Prompts (Templates)

Plantillas reutilizables:

```json
{
  "prompts": [
    {
      "name": "rag_context",
      "description": "Generate RAG context prompt",
      "arguments": [
        {"name": "query", "required": true}
      ]
    }
  ]
}
```

## Configuración en IDEs

### Cursor

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vantadb-server",
      "args": ["--mcp", "--port", "3000"]
    }
  }
}
```

### Claude Code

```json
{
  "mcpServers": {
    "vantadb": {
      "type": "sse",
      "url": "http://localhost:3000/sse"
    }
  }
}
```

## Flujo de Interacción

```
1. AI Assistant → MCP Server: initialize
2. MCP Server → AI Assistant: capabilities
3. AI Assistant → MCP Server: tools/list
4. MCP Server → AI Assistant: available tools
5. AI Assistant → MCP Server: tools/call (search_memory)
6. MCP Server → VantaDB: search()
7. VantaDB → MCP Server: results
8. MCP Server → AI Assistant: formatted results
```

## Seguridad

### Modo Read-Only por Defecto

```rust
pub struct McpServer {
    db: VantaEmbedded,
    read_only: bool,  // Default: true
}
```

### Validación de Inputs

```rust
fn validate_tool_call(tool: &str, params: &Value) -> Result<()> {
    match tool {
        "search_memory" => validate_search_params(params),
        "store_memory" => {
            if read_only {
                return Err(McpError::ReadOnly);
            }
            validate_store_params(params)
        },
        _ => Err(McpError::UnknownTool),
    }
}
```

## Casos de Uso

### 1. Memoria de Proyecto en IDE

El IDE usa VantaDB como memoria persistente del proyecto:
- Recuerda decisiones de diseño
- Mantiene contexto entre sesiones
- Busca código relacionado semánticamente

### 2. Asistente de Código

```
Usuario: "¿Cómo funciona la autenticación en este proyecto?"
LLM → MCP: search_memory("autenticación", top_k=5)
MCP → VantaDB: hybrid_search(...)
VantaDB → MCP: [relevant docs]
LLM: "La autenticación usa JWT con..."
```

### 3. Knowledge Base Local

```
Usuario: "Resume las reuniones de esta semana"
LLM → MCP: search_memory("reuniones semana", filter={date: "this_week"})
MCP → VantaDB: filtered_search(...)
LLM: "Esta semana se discutieron 3 temas principales..."
```

## Véase También

- [RAG](RAG.md) — Caso de uso principal
- [GraphRAG](GraphRAG.md) — Búsqueda con contexto relacional
- [Agentes de IA](Agentes de IA.md) — Consumidores del protocolo

### Documentación de Implementación Relacionada
- [[../api/MCP|MCP API Integration]]

---

*MCP permite que VantaDB sea la capa de memoria estándar para herramientas de desarrollo con IA.*

