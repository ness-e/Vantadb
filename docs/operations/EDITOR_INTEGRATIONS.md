---
title: VantaDB Editor Integrations
type: operations
status: active
tags: [vantadb, operations]
last_reviewed: 2026-07-01
aliases: []
---

# VantaDB Editor Integrations

This guide explains how to integrate VantaDB with popular code editors and AI-assisted development environments using the Model Context Protocol (MCP).

## Overview

VantaDB provides a complete MCP server implementation that enables AI assistants in code editors to access persistent memory, perform hybrid vector and text search, and maintain context across sessions.

## Supported Editors

### Cursor

Cursor has native MCP support for AI assistants.

#### Configuration

Add to your Cursor configuration file (`~/.cursor/config.json`):

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "~/.vantadb"],
      "env": {
        "VANTADB_PATH": "~/.vantadb"
      }
    }
  }
}
```

#### Usage

Once configured, Cursor's AI assistant can:
- Store and retrieve project context
- Search through code documentation
- Maintain conversation history
- Access operational metrics

### VS Code

VS Code requires an MCP-compatible extension.

#### Prerequisites

1. Install an MCP-compatible extension (e.g., "MCP Client" or similar)
2. Ensure VantaDB server is installed

#### Configuration

Configure the MCP client extension:

```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "${workspaceFolder}/.vantadb"],
      "cwd": "${workspaceFolder}"
    }
  }
}
```

#### Workspace-Specific Setup

For workspace-specific VantaDB instances:

```json
{
  "mcpServers": {
    "vantadb-workspace": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "${workspaceFolder}/.vantadb"],
      "cwd": "${workspaceFolder}"
    }
  }
}
```

### OpenCode

OpenCode supports MCP through its AI assistant features.

#### Configuration

```json
{
  "mcp": {
    "servers": {
      "vantadb": {
        "command": "vantadb-server",
        "args": ["--mcp", "--path", "~/.vantadb"],
        "enabled": true
      }
    }
  }
}
```

### OpenClaw

> **⚠️ DRAFT** — Esta sección está pendiente de verificación contra el código real. OpenClaw no es un editor verificado.

OpenClaw integrates with MCP for AI-powered development.

#### Configuration

```yaml
mcp:
  servers:
    vantadb:
      command: vantadb-server
      args:
        - --mcp
        - --path
        - ~/.vantadb
```

### Devin

Devin (AI-powered IDE) supports MCP for persistent memory.

#### Configuration

```json
{
  "ai": {
    "memory": {
      "backend": "vantadb",
      "mcp": {
        "command": "vantadb-server",
        "args": ["--mcp", "--path", "~/.vantadb"]
      }
    }
  }
}
```

### Antigravity

> **⚠️ DRAFT** — Esta sección está pendiente de verificación contra el código real. "Antigravity" no es un editor conocido.

Antigravity editor with AI features supports MCP.

#### Configuration

```toml
[mcp]
[mcp.servers](../api/MCP.md)
name = "vantadb"
command = "vantadb-server"
args = ["--mcp", "--path", "~/.vantadb"]
```

## Common Configuration Patterns

### Project-Level Memory

Configure VantaDB to store memory per project:

```json
{
  "mcpServers": {
    "vantadb-project": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "${workspaceFolder}/.vantadb"],
      "cwd": "${workspaceFolder}",
      "env": {
        "VANTADB_NAMESPACE": "project-${workspaceFolderBasename}"
      }
    }
  }
}
```

### Global Memory

Use a single global VantaDB instance across all projects:

```json
{
  "mcpServers": {
    "vantadb-global": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "~/.vantadb/global"],
      "env": {
        "VANTADB_NAMESPACE": "global"
      }
    }
  }
}
```

### Multi-Workspace Setup

Configure separate VantaDB instances for different workspaces:

```json
{
  "mcpServers": {
    "vantadb-workspace-1": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "${workspaceFolder}/.vantadb"],
      "cwd": "${workspaceFolder}",
      "condition": "workspaceFolder =~ /project1/"
    },
    "vantadb-workspace-2": {
      "command": "vantadb-server",
      "args": ["--mcp", "--path", "${workspaceFolder}/.vantadb"],
      "cwd": "${workspaceFolder}",
      "condition": "workspaceFolder =~ /project2/"
    }
  }
}
```

## Available MCP Tools

When VantaDB is connected via MCP, the following tools are available to AI assistants:

### Memory Operations

- `memory_put` - Inserts or updates a memory record with payload, vector, and optional metadata
- `memory_get` - Retrieves a memory record by namespace and key
- `memory_delete` - Deletes a memory record by namespace and key
- `memory_list` - Lists memory records with optional pagination and metadata filters
- `memory_list_namespaces` - Lists all available namespaces

### Search Operations

- `search_memory` - Hybrid vector + text search with optional filters, distance metric, and explain
- `search_semantic` - Raw semantic vector search directly in the HNSW index

### Graph Operations

- `query_iql` - Executes an IQL statement (read structures, insert/mutate nodes providing semantic context)
- `get_node_neighbors` - Inspect neighbors or lineage of a node
- `inject_context` - Injects external state connecting it to a specific thread for subsequent consolidation
- `read_axioms` - Returns the active Devil's Advocate Axioms (Iron Axioms)

### Collection Operations

- `collection_stats` - Returns statistics for a namespace: record count, byte size, vector index info, creation time
- `collection_list` - Lists all collections with metadata including record count, vector index status, and creation time
- `collection_delete` - Deletes an entire namespace/collection and all its records (requires `confirm: "yes"`)

### Resources

- `metrics://` - Operational metrics (memory usage, HNSW stats, storage info)
- `memory://{namespace}/{key}` - Individual memory record
- `namespace://{namespace}` - Namespace contents

## Example Workflows

### Code Review Assistant

AI assistant can:
1. Store code review comments in VantaDB
2. Search for similar issues across projects
3. Maintain review history with metadata
4. Retrieve relevant context for new reviews

### Documentation Search

AI assistant can:
1. Index project documentation
2. Perform semantic search for relevant docs
3. Maintain versioned documentation history
4. Cross-reference related documentation

### Context-Aware Coding

AI assistant can:
1. Store project-specific preferences
2. Maintain coding style guidelines
3. Track architectural decisions
4. Retrieve relevant context for code generation

## Troubleshooting

### Connection Issues

**Problem**: Editor cannot connect to VantaDB MCP server

**Solutions**:
1. Verify VantaDB server is installed: `vantadb-server --version`
2. Check the command path in configuration
3. Ensure the database path is writable
4. Check editor logs for connection errors

### Permission Errors

**Problem**: VantaDB cannot write to database path

**Solutions**:
1. Ensure the directory exists and is writable
2. Check file system permissions
3. Use a different path with proper permissions
4. Run editor with appropriate user permissions

### Memory Issues

**Problem**: VantaDB consuming too much memory

**Solutions**:
1. Configure memory limit in VantaConfig
2. Use namespace isolation to limit scope
3. Implement periodic cleanup of old memories
4. Adjust HNSW parameters for memory efficiency

## Performance Optimization

### Index Configuration

Optimize HNSW index for your workload:

```json
{
  "hnsw": {
    "m": 16,
    "ef_construction": 200,
    "ef_search": 50
  }
}
```

### Memory Limits

Configure appropriate memory limits:

```json
{
  "memory_limit_bytes": 512000000,
  "max_blocking_threads": 4
}
```

### Namespace Strategy

Use namespaces strategically:
- Per-project namespaces for project-specific memory
- Global namespace for shared preferences
- Session namespaces for temporary context

## Security Considerations

### Namespace Isolation

Use separate namespaces for different contexts to prevent cross-contamination.

### Read-Only Mode

For production deployments, consider read-only mode for AI assistants:

```json
{
  "read_only": true
}
```

### Access Control

Implement access control at the editor level:
- Restrict which projects can access global memory
- Use different VantaDB instances for different security levels
- Audit memory access logs

## Advanced Configuration

### Custom Tokenizer

> **⚠️ DRAFT** — Esta sección está pendiente de verificación contra el código real. No existe una API `advanced_tokenizer` en el MCP server.

Configure advanced tokenizer for better text search:

```json
{
  "advanced_tokenizer": {
    "language": "english",
    "stemming": true,
    "stopwords": true
  }
}
```

### Custom Metrics

> **⚠️ DRAFT** — Esta sección está pendiente de verificación contra el código real. No existen las opciones `enable_hnsw_stats`, `enable_storage_stats`, `enable_query_stats` en el MCP server real.

Configure custom operational metrics:

```json
{
  "metrics": {
    "enable_hnsw_stats": true,
    "enable_storage_stats": true,
    "enable_query_stats": true
  }
}
```

## Migration Guide

### From Other Memory Backends

Migrating from other memory backends to VantaDB:

1. Export existing memories
2. Transform to VantaDB format
3. Import using `memory_put` tool
4. Verify search functionality
5. Update editor configuration

### Version Upgrades

When upgrading VantaDB:
1. Backup existing database
2. Update VantaDB server
3. Test MCP connectivity
4. Verify existing memories
5. Update configuration if needed

## Support

For issues with editor integrations:
- GitHub Issues: https://github.com/your-org/vantadb/issues
- Documentation: https://docs.vantadb.io
- MCP Protocol: https://modelcontextprotocol.io

## Contributing

To add support for additional editors:
1. Test MCP compatibility
2. Document configuration steps
3. Provide example configurations
4. Submit a pull request with documentation
