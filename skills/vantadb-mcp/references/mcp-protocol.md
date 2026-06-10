# MCP Protocol Reference

## Overview

The Model Context Protocol (MCP) is a standardized protocol for AI agents to interact with external systems through a JSON-RPC interface over stdio.

## Protocol Version

Current version: `2024-11-05`

## Connection

MCP servers communicate over stdio using JSON-RPC 2.0:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": { ... }
}
```

## Methods

### initialize

Initialize the MCP connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "client-name",
      "version": "1.0.0"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": {
      "name": "server-name",
      "version": "1.0.0"
    },
    "capabilities": {
      "tools": {},
      "resources": {},
      "prompts": {}
    }
  }
}
```

### tools/list

List available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "tool-name",
        "description": "Tool description",
        "inputSchema": {
          "type": "object",
          "properties": { ... },
          "required": [ ... ]
        }
      }
    ]
  }
}
```

### tools/call

Execute a tool.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "tool-name",
    "arguments": { ... }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Result text"
      }
    ]
  }
}
```

### resources/list

List available resources.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "resources": [
      {
        "uri": "resource-uri",
        "name": "Resource name",
        "description": "Resource description",
        "mimeType": "application/json"
      }
    ]
  }
}
```

### resources/read

Read a resource.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "resources/read",
  "params": {
    "uri": "resource-uri"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "contents": [
      {
        "uri": "resource-uri",
        "mimeType": "application/json",
        "text": "Resource content"
      }
    ]
  }
}
```

### prompts/list

List available prompts.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "prompts/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "prompts": [
      {
        "name": "prompt-name",
        "description": "Prompt description",
        "arguments": [
          {
            "name": "arg-name",
            "description": "Argument description",
            "required": true
          }
        ]
      }
    ]
  }
}
```

### prompts/get

Get a specific prompt.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "prompts/get",
  "params": {
    "name": "prompt-name",
    "arguments": { ... }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "description": "Prompt description",
    "messages": [
      {
        "role": "user",
        "content": {
          "type": "text",
          "text": "Prompt text"
        }
      }
    ]
  }
}
```

## Error Handling

Errors follow JSON-RPC 2.0 format:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

Common error codes:
- `-32600`: Invalid Request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Capabilities

Servers declare their capabilities in the initialize response:

- `tools`: Tool execution support
- `resources`: Resource access support
- `prompts`: Prompt template support

## More Information

Official MCP documentation: https://modelcontextprotocol.io
