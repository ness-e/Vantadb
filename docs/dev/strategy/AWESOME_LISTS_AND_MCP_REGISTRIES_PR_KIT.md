---
title: "Kit Operativo de Pull Requests: 4 Awesome Lists y 5 Registros MCP (GTM-04 + GTM-10)"
kind: concept
status: active
description: "Guía operativa paso a paso con los bloques exactos de Markdown, JSON y metadatos listos para copiar y pegar al enviar PRs a las 4 Awesome Lists y publicar en los 5 Registros MCP oficiales tras la certificación de FASE-A."
tags: [vantadb, gtm, mcp, awesome-lists, pull-requests, registries, marketing, distribution]
---

# Kit Operativo de PRs: 4 Awesome Lists y 5 Registros MCP (`GTM-04` + `GTM-10`)

> **Propósito:** Proveer al mantenedor de VantaDB (`ness-e`) una guía de ejecución táctica libre de ambigüedades con **títulos, descripciones, snippets Markdown de una línea y payloads JSON/YAML listos para copiar y pegar**.  
> **Alineación:** Respuestas al cuestionario maestro `Q28 (Opción B: Escalonado Orgánico)` + Backlog `GTM-04` (Manifiestos MCP) y `GTM-10` (PRs Awesome Lists).  
> **Condición de Disparo:** Se ejecuta una vez superado el Stranger Test de `FASE-A`, con el GIF demo VHS incrustado en el `README.md` y CI en verde en la rama principal.

---

## 📋 Resumen de Inventario y Calendario Escalonado

Para maximizar la tasa de aprobación de los mantenedores y evitar penalizaciones algorítmicas por spam concentrado, los envíos se distribuyen a lo largo de 7 días hábiles según la siguiente matriz:

| Día | Objetivo | Tipo | Repositorio / Plataforma Destino | Fichero en Repo Local |
| :---: | :--- | :--- | :--- | :--- |
| **Día 1** | Registro MCP Oficial | Manifiesto Oficial | `modelcontextprotocol/registry` | `server.json` (raíz) |
| **Día 1** | Glama.ai Registry | Directorio Web | `glama.ai/mcp` | `glama.json` (raíz) |
| **Día 2** | Smithery Marketplace | Marketplace CLI | `smithery.ai` | `smithery.yaml` (raíz) |
| **Día 3** | Awesome MCP Servers | Curated List | `punkpeye/awesome-mcp-servers` & `wong2/awesome-mcp-servers` | Sección 1 de esta guía |
| **Día 4** | PulseMCP & MCP.so | Directorio Web | `pulsemcp.com` & `mcp.so` | Sección 2 de esta guía |
| **Día 5** | Awesome AI Agents | Curated List | `e2b-dev/awesome-ai-agents` & `kyrolabs/awesome-agents` | Sección 1 de esta guía |
| **Día 6** | Cursor Directory | Extensiones IDE | `cursor.directory` | Sección 2 de esta guía |
| **Día 7** | Awesome Rust | Curated List | `rust-unofficial/awesome-rust` | Sección 1 de esta guía |
| **Día 8** | Awesome Local AI | Curated List | `janhq/awesome-local-ai` | Sección 1 de esta guía |

---

## 🚀 Parte 1: Las 4 Awesome Lists (`GTM-10`)

---

### 1. Lista 1: Awesome MCP Servers (`punkpeye` y `wong2`)

#### A. Repositorio: `punkpeye/awesome-mcp-servers`
* **URL:** `https://github.com/punkpeye/awesome-mcp-servers`
* **Sección destino:** `### Knowledge & Memory` (o `### Databases`)
* **Ubicación en lista:** Orden alfabético por nombre (`VantaDB` bajo la letra V).
* **Convención de emojis del repositorio:** `🦀` = Escrito en Rust; `🏠` = Ejecución local / Local-first; `☁️` = Nube.

##### Bloque Markdown exacto (1 línea):
```markdown
- [ness-e/Vantadb](https://github.com/ness-e/Vantadb) 🦀 🏠 - Embedded, local-first long-term memory and hybrid retrieval engine (BM25 + HNSW) in Rust for AI agents with tiered memory (L0–L3) and token-budgeted tool profiles.
```

##### Título del Pull Request:
```text
Add VantaDB (embedded agent memory & hybrid search engine) to Knowledge & Memory
```

##### Cuerpo del Pull Request:
```markdown
### Summary of Changes

Adds **VantaDB** to the `Knowledge & Memory` section.

- **Repository:** https://github.com/ness-e/Vantadb
- **Description:** Embedded, local-first long-term memory engine and hybrid vector database written in Rust (Apache-2.0). Provides sub-2ms recall, tiered memory pipelines (L0-L3), local ONNX embeddings (EmbeddingGemma 2 / E5-small), and token-budgeted MCP profiles (`brain` profile with 15 core tools to eliminate context bloat).
- **Transport:** stdio
- **Language / Runtime:** Rust (`🦀`), runs 100% locally (`🏠`).

Thank you for curating this list!
```

---

#### B. Repositorio: `wong2/awesome-mcp-servers`
* **URL:** `https://github.com/wong2/awesome-mcp-servers`
* **Sección destino:** `### Memory & Knowledge` (o `### Storage & Database`)

##### Bloque Markdown exacto (1 línea):
```markdown
- [VantaDB](https://github.com/ness-e/Vantadb) - Embedded, local-first long-term memory engine and hybrid search (BM25 + HNSW) in Rust for AI coding agents. Zero Docker, sub-2ms recall, and token-budgeted MCP profiles.
```

##### Título del Pull Request:
```text
Add VantaDB to Memory & Knowledge
```

##### Cuerpo del Pull Request:
```markdown
### What is this server?

[VantaDB](https://github.com/ness-e/Vantadb) is an embedded, zero-daemon long-term memory engine for AI agents (Cursor, Claude Code, Windsurf, OpenCode).

- Written in 100% pure Rust (Apache-2.0).
- Uses BM25 lexical + topological HNSW vector retrieval fused via Reciprocal Rank Fusion (RRF).
- Features a compact `brain` MCP profile (15 essential tools) that cuts schema token consumption by ~85% compared to monolithic agent servers.
- Fully local and offline (built-in ONNX embeddings support).

Tested and verified via stdio JSON-RPC.
```

---

### 2. Lista 2: Awesome AI Agents (`e2b-dev` y `kyrolabs`)

#### A. Repositorio: `e2b-dev/awesome-ai-agents`
* **URL:** `https://github.com/e2b-dev/awesome-ai-agents`
* **Sección destino:** `### Memory & Storage` (o `### Infrastructure & Tooling`)

##### Bloque Markdown exacto (1 línea):
```markdown
- [VantaDB](https://github.com/ness-e/Vantadb) - Embedded, local-first long-term memory and hybrid retrieval engine in Rust for AI agents. Zero Docker, sub-2ms recall, and tiered cognitive memory (L0–L3).
```

##### Título del Pull Request:
```text
Add VantaDB to Memory & Storage infrastructure
```

##### Cuerpo del Pull Request:
```markdown
### Description

This PR adds **VantaDB** to the `Memory & Storage` category.

VantaDB is an open-source (Apache-2.0), embedded long-term memory engine designed specifically for local AI agents. Rather than running multi-gigabyte Docker containers with Postgres and pgvector, VantaDB provides an in-process / stdio engine in a single Rust binary:
- **Sub-2ms hybrid retrieval:** Reciprocal Rank Fusion of BM25 + HNSW on disk.
- **Cognitive tiered pipeline:** L0 (session), L1 (scenes/episodes), L2 (consolidated semantic knowledge graph), and L3 (sync).
- **FastGate pre-LLM filter:** Drops >85% of redundant agent writes deterministically without calling LLM APIs.
- **Drop-in compatibility:** Python SDK with Mem0-compatible adapter, Node.js native bindings, TS/WASM, and MCP server.

License: Apache-2.0  
Website: https://vantadb.dev
```

---

#### B. Repositorio: `kyrolabs/awesome-agents`
* **URL:** `https://github.com/kyrolabs/awesome-agents`
* **Sección destino:** `### Agent Memory & Context`

##### Bloque Markdown exacto (1 línea):
```markdown
- [VantaDB](https://github.com/ness-e/Vantadb) - Local-first embedded memory and hybrid vector retrieval engine for AI agents with tiered episodic/semantic memory and zero cloud overhead.
```

##### Título del Pull Request:
```text
Add VantaDB to Agent Memory & Context
```

##### Cuerpo del Pull Request:
```markdown
Adds VantaDB (https://github.com/ness-e/Vantadb) to the Agent Memory & Context section.

VantaDB is an embedded database tailored for persistent agent state and memory governance, featuring local ONNX embeddings, hybrid BM25+HNSW search, and full GDPR compliance primitives.
```

---

### 3. Lista 3: Awesome Rust (`rust-unofficial/awesome-rust`)

* **URL:** `https://github.com/rust-unofficial/awesome-rust`
* **Sección destino:** `Database` -> `Embedded databases` (o `Applications` -> `Database`)
* **Reglas y Criterios Obligatorios de `awesome-rust` (Checklist de Cumplimiento):**
  - [x] **Licencia:** 100% código abierto bajo licencia Apache-2.0 permisiva.
  - [x] **Puntuación y Madurez:** Repositorio con CI en verde, tests unitarios e integrados completos.
  - [x] **Formato estricto:** Separador `—` (em-dash con espacios) o `-`, sin adjetivos subjetivos ("the best", "fastest ever"), terminando con un punto final (`.`).
  - [x] **Ordenación:** Estrictamente alfabética por el texto del enlace en su subsección.

##### Bloque Markdown exacto (1 línea):
```markdown
* [VantaDB](https://github.com/ness-e/Vantadb) — Embedded, local-first long-term memory engine and hybrid retrieval database (BM25 + HNSW) for AI agents.
```

##### Título del Pull Request:
```text
Add VantaDB to Database / Embedded databases
```

##### Cuerpo del Pull Request:
```markdown
### Summary

This PR adds **VantaDB** to the `Database` -> `Embedded databases` section.

- **Project:** [VantaDB](https://github.com/ness-e/Vantadb)
- **Description:** An embedded, local-first database combining BM25 full-text indexing, topological HNSW vector retrieval (RRF fusion), bitemporal graph storage, and write-ahead log recovery in Rust.
- **License:** Apache-2.0
- **Alphabetical order:** Verified and inserted in alphabetical order within the section.
- **Formatting:** Follows the repository's formatting guidelines (em-dash separator, ending with a period).

Thank you for reviewing!
```

---

### 4. Lista 4: Awesome Local AI (`janhq/awesome-local-ai`)

* **URL:** `https://github.com/janhq/awesome-local-ai`
* **Sección destino:** `### Tools` -> subsección `Agent Memory` (o `### Agents`)
* **Enfoque de valor:** Resalta la independencia de la nube, 0 tokens desperdiciados y compatibilidad con inferencia local (Ollama / Llama.cpp / ONNX local).

##### Bloque Markdown exacto (1 línea):
```markdown
- [VantaDB](https://github.com/ness-e/Vantadb) - Embedded, local-first long-term memory engine and hybrid search database for local AI agents (zero Docker, sub-2ms recall, 100% offline).
```

##### Título del Pull Request:
```text
Add VantaDB (embedded local-first agent memory) to Tools
```

##### Cuerpo del Pull Request:
```markdown
### Contribution Summary

Adds **VantaDB** to the `Tools / Agent Memory` section.

- **Repository:** https://github.com/ness-e/Vantadb
- **Highlights for Local AI:**
  - 100% offline, zero cloud telemetry or dependencies.
  - Runs in-process or as a lightweight stdio MCP server for local agents (Jan, OpenCode, Claude Code, Cursor).
  - Built-in local ONNX embedding pipelines (`multilingual-e5-small` and `EmbeddingGemma 2`).
  - Eliminates context window bloat by storing agent history locally and retrieving relevant memories in < 2ms using BM25 + HNSW.
- **License:** Apache-2.0
```

---

## 🛠️ Parte 2: Los 5 Registros MCP Oficiales (`GTM-04`)

---

### 1. Registro 1: Official MCP Registry (`modelcontextprotocol/registry`)

El protocolo oficial de Anthropic / MCP mantiene un registro centralizado basado en esquemas JSON versionados. VantaDB ya cuenta con su manifiesto canónico `server.json` en la raíz del repositorio.

#### A. Ubicación del Manifiesto Local:
* `server.json` (raíz del repositorio, versión de esquema `2025-12-11`).

#### B. Payload JSON Exacto de `server.json`:
```json
{
  "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
  "name": "io.github.ness-e/vantadb",
  "title": "VantaDB MCP",
  "description": "MCP server for VantaDB: durable local memory and hybrid vector retrieval for AI agents.",
  "version": "0.10.0",
  "websiteUrl": "https://github.com/ness-e/Vantadb/blob/main/docs/api/MCP.md",
  "repository": {
    "url": "https://github.com/ness-e/Vantadb",
    "source": "github"
  },
  "_meta": {
    "io.modelcontextprotocol.registry/publisher-provided": {
      "tool": "vantadb-docs-bot",
      "build_info": {
        "timestamp": "2026-11-07",
        "schema": "2025-12-11",
        "submission_state": "submitted"
      }
    }
  }
}
```

#### C. Procedimiento de Publicación vía Pull Request:
1. Hacer fork de `https://github.com/modelcontextprotocol/registry`.
2. Crear rama: `git checkout -b add-vantadb-server`.
3. Crear el directorio de namespace: `servers/io.github.ness-e/vantadb/`.
4. Copiar el archivo `server.json` en esa ruta.
5. Abrir Pull Request:
   * **Título:** `feat: Add io.github.ness-e/vantadb`
   * **Cuerpo:**
     ```markdown
     Submitting VantaDB MCP server manifest.

     - **Namespace:** `io.github.ness-e/vantadb`
     - **Repository:** https://github.com/ness-e/Vantadb
     - **Verification:** Repository owned by GitHub user `ness-e`.
     - **Transport:** stdio
     - **Schema:** 2025-12-11
     ```

---

### 2. Registro 2: Glama.ai MCP Directory (`glama.ai/mcp`)

Glama indexa servidores MCP verificando la autoría mediante el archivo `glama.json` en la raíz del repositorio oficial.

#### A. Ubicación del Archivo Local:
* `glama.json` (raíz del repositorio).

#### B. Contenido de `glama.json`:
```json
{
  "$schema": "https://glama.ai/mcp/schemas/server.json",
  "maintainers": [
    "ness-e"
  ]
}
```

#### C. Procedimiento de Verificación en Glama:
1. Iniciar sesión con la cuenta de GitHub `ness-e` en `https://glama.ai/mcp`.
2. Navegar a **Submit Server** (`https://glama.ai/mcp/servers/new`).
3. Pegar la URL del repositorio: `https://github.com/ness-e/Vantadb`.
4. Glama leerá automáticamente el archivo `glama.json` y comprobará que `maintainers` contiene `"ness-e"`.
5. La indexación y generación de insignia (badge) es instantánea.

---

### 3. Registro 3: Smithery.ai Marketplace (`smithery.ai`)

Smithery es el registro y gestor de paquetes para despliegues automatizados de herramientas MCP en clientes como Claude Desktop y Cursor.

#### A. Ubicación del Archivo Local:
* `smithery.yaml` (raíz del repositorio).

#### B. Contenido de `smithery.yaml`:
```yaml
# Smithery configuration file: https://smithery.ai/docs/config#smitheryyaml
startCommand:
  type: stdio
configSchema:
  type: object
  properties:
    profile:
      type: string
      description: "VantaDB MCP operating profile: 'brain' (default 15 core cognitive memory tools), 'coder' (codebase and project memory), or 'minimal' (basic key-value and search)"
      default: "brain"
      enum:
        - "brain"
        - "coder"
        - "minimal"
    dbPath:
      type: string
      description: "Custom directory path for persistent database storage (defaults to ~/.vantadb)"
commandFunction: |-
  (config) => ({
    command: 'vanta-cli',
    args: [
      'server',
      '--mcp',
      '--profile',
      config.profile || 'brain',
      ...(config.dbPath ? ['--db', config.dbPath] : [])
    ],
    env: {
      VANTADB_MCP_PROFILE: config.profile || 'brain'
    }
  })
exampleConfig:
  profile: "brain"
```

#### C. Procedimiento de Indexación:
1. Acceder a `https://smithery.ai`.
2. Conectar GitHub y autorizar el repositorio `ness-e/Vantadb`.
3. O ejecutar desde la terminal del desarrollador:
   ```bash
   npx @smithery/cli publish
   ```
4. Smithery parsea `smithery.yaml` y habilita la instalación en un clic: `npx -y @smithery/cli install @ness-e/vantadb --client cursor`.

---

### 4. Registro 4: Cursor Directory (`cursor.directory`)

`cursor.directory` es el directorio más consultado por usuarios de Cursor para reglas `.cursorrules` y servidores MCP compatibles.

#### Formulario de Envío en `https://cursor.directory/mcp`:

| Campo | Valor a Introducir |
| :--- | :--- |
| **Server Name** | `VantaDB` |
| **Short Tagline** | `Embedded, local-first long-term memory & hybrid search engine for AI agents` |
| **Category** | `Database` / `Memory & Context` / `Developer Tools` |
| **GitHub Repository** | `https://github.com/ness-e/Vantadb` |
| **Website URL** | `https://vantadb.dev` |
| **Command** | `vanta-cli` |
| **Args** | `server`, `--mcp`, `--profile`, `brain` |
| **Environment Variables** | `VANTADB_MCP_PROFILE=brain` |
| **One-Click Config Snippet** | *(Ver bloque JSON inferior)* |

##### Configuración JSON para Cursor (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "vantadb": {
      "command": "vanta-cli",
      "args": ["server", "--mcp", "--profile", "brain"],
      "env": {
        "VANTADB_MCP_PROFILE": "brain"
      }
    }
  }
}
```

##### Descripción en Markdown para el Formulario:
```markdown
VantaDB is an embedded, local-first database and long-term memory engine written in Rust for Cursor agents.

### Why use VantaDB with Cursor?
- **Zero Token Bloat (`brain` profile):** Advertises only 15 compact cognitive tools (~2.4k tokens), keeping your context window clear for code.
- **Multi-Instance Concurrency:** Run multiple Cursor windows without database file lock crashes (`DatabaseBusy`).
- **Hybrid Retrieval:** BM25 lexical + HNSW vector search fused via RRF (< 1.5ms latency).
- **Persistent Bugfix Learning:** Records resolved compiler errors and architectural decisions so your agent doesn't repeat past mistakes.

### 30-Second Quickstart:
1. Install CLI: `irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex` (Windows) or `curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh` (macOS/Linux)
2. Run: `vanta-cli mcp init --profile brain`
```

---

### 5. Registro 5: PulseMCP y MCP.so

Directorios de descubrimiento comunitario en tiempo real de servidores MCP y herramientas interoperables.

#### A. PulseMCP (`https://pulsemcp.com/submit`):
* **Project Name:** VantaDB
* **One-line summary:** Zero-daemon, local-first long-term cognitive memory and hybrid vector retrieval engine in Rust.
* **Repository URL:** `https://github.com/ness-e/Vantadb`
* **Transport Type:** `stdio`
* **License:** `Apache-2.0`
* **Tags:** `memory`, `vector-database`, `rust`, `hybrid-search`, `bm25`, `hnsw`, `local-ai`, `cursor`, `claude`

#### B. MCP.so (`https://mcp.so/submit`):
* **Server Title:** VantaDB Cognitive Memory
* **Package/Command:** `vanta-cli server --mcp --profile brain`
* **Primary Category:** `Knowledge & Memory`
* **Docs URL:** `https://github.com/ness-e/Vantadb/blob/main/docs/api/MCP.md`
* **Demo URL:** `https://vantadb.dev`

---

## 🛡️ FMEA: Gestión de Riesgos y Modos de Fallo en Envíos

| Modo de Fallo | Probabilidad | Severidad | Mitigación Inmediata |
| :--- | :---: | :---: | :--- |
| **Rechazo por orden alfabético en Awesome List** | Media | Baja | Verificar con `Ctrl+F` la posición de la letra `V` antes de hacer commit en el fork. |
| **Linters de CI fallando en el repo upstream** | Media | Media | Comprobar trailing spaces, enlaces caídos y formato em-dash (`—`) antes de abrir el PR. |
| **Mantenedor upstream pide estrellas mínimas** | Baja | Media | Posponer el envío de esa lista específica hasta completar los Días 1 y 2 en Reddit / Show HN (`>150 stars`). |
| **Smithery CLI falla compilación o target** | Baja | Baja | `smithery.yaml` invoca `vanta-cli` binario precompilado o instalado globalmente; no requiere build de Node. |
| **Retraso en revisión manual de `modelcontextprotocol/registry`** | Alta | Baja | Normal en el registro oficial (1 a 7 días hábiles); no bloquea ningún otro canal ni el uso del servidor. |

---

## 📌 Checklist de Pre-Flight para el Propietario

Antes de pulsar "Create Pull Request":
1. [ ] ¿Está la rama `main` de `ness-e/Vantadb` al día y con CI pasando?
2. [ ] ¿Está incrustado el GIF `docs/assets/demo.gif` en `README.md` y `README_ES.md`?
3. [ ] ¿Se han subido los archivos `glama.json` y `smithery.yaml` a la raíz del repositorio?
4. [ ] ¿Se ha creado una rama con nombre descriptivo en cada fork (ej: `feat/add-vantadb`) en lugar de trabajar sobre `main`?
5. [ ] ¿Se respetó el orden alfabético de la sección correspondiente en cada lista?
