# ConnectomeDB — Documentation & Developer Experience Strategy

---

## 1. Documentation Site

### Plataforma: **mdBook** (Rust-native)
```
¿Por qué NO Docusaurus?
  - Docusaurus requiere Node.js/React → contradicción para un proyecto "zero-dependency Rust"
  - mdBook es el estándar de la comunidad Rust (usado por The Rust Book, Tokio, Axum)
  - Deploys estáticos en GitHub Pages sin CI complejo

¿Por qué NO MkDocs?
  - Python dependency → misma contradicción
  - mdBook tiene mejor rendering de código Rust
```

### Estructura del docs site:
```
docs/
├── book.toml                    # mdBook config
├── src/
│   ├── SUMMARY.md               # Table of Contents
│   ├── introduction.md          # What is ConnectomeDB
│   │
│   ├── getting-started/
│   │   ├── installation.md      # cargo, docker, binary
│   │   ├── quickstart.md        # First insert + query in 2min
│   │   ├── configuration.md     # Env vars, ports, LLM setup
│   │   └── docker.md            # Docker compose with Ollama
│   │
│   ├── iql-reference/
│   │   ├── overview.md          # IQL philosophy and syntax
│   │   ├── queries.md           # FROM, WHERE, FETCH, RANK BY
│   │   ├── mutations.md         # INSERT, UPDATE, DELETE, RELATE
│   │   ├── vector-search.md     # ~ operator, HNSW, TEMPERATURE
│   │   ├── graph-traversal.md   # SIGUE, depth, edge labels
│   │   ├── conversational.md    # INSERT MESSAGE, threads
│   │   └── rbac.md              # ROLE, owner_role, permissions
│   │
│   ├── architecture/
│   │   ├── unified-node.md      # UnifiedNode struct explained
│   │   ├── storage-engine.md    # RocksDB + WAL + zero-copy
│   │   ├── hnsw-index.md        # HNSW implementation details
│   │   ├── query-pipeline.md    # Parser → AST → LogicalPlan → Executor
│   │   ├── graph-engine.md      # BFS, edge weights, traversal
│   │   └── llm-bridge.md        # Ollama integration, auto-embedding
│   │
│   ├── integrations/
│   │   ├── ollama.md            # Native LLM setup
│   │   ├── langchain.md         # Python SDK + LangChain
│   │   ├── rest-api.md          # HTTP endpoints reference
│   │   └── prometheus.md        # Metrics & monitoring
│   │
│   ├── guides/
│   │   ├── rag-agent.md         # Build a RAG agent tutorial
│   │   ├── recommendation.md   # Recommendation engine tutorial
│   │   ├── knowledge-base.md    # Enterprise KB tutorial
│   │   └── migration.md         # From Postgres/Neo4j/Pinecone
│   │
│   └── reference/
│       ├── cli.md               # CLI commands reference
│       ├── config.md            # All config options
│       ├── errors.md            # Error codes and troubleshooting
│       └── changelog.md         # Version history
```

### Deploy: GitHub Pages
```yaml
# .github/workflows/docs.yml
name: Deploy Docs
on:
  push:
    branches: [main]
    paths: ['docs/**']
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install mdbook
      - run: cd docs && mdbook build
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book
```

---

## 2. API Reference

### Estrategia dual:

**Capa 1 — `cargo doc` automático:**
```bash
# Genera docs de toda la API Rust pública
cargo doc --no-deps --open

# Se ejecuta automáticamente en CI y se publica en:
# https://connectomedb.dev/rustdoc/
```

**Capa 2 — REST API Reference (OpenAPI/Swagger):**
```yaml
# Generado a partir de los handlers de Axum en src/server.rs
# Herramienta: utoipa (crate Rust para generar OpenAPI desde annotations)

endpoints:
  POST /api/v1/query:
    description: Execute IQL statement (read or write)
    body: { "query": "FROM Persona WHERE nombre = \"Eros\"" }
    response: { "nodes": [...], "time_ms": 4.2 }

  GET /api/v1/health:
    description: Server health check
    response: { "status": "ok", "uptime_s": 3600, "nodes_count": 50000 }

  GET /metrics:
    description: Prometheus metrics endpoint
    response: text/plain (prometheus exposition format)
```

---

## 3. DB Visualizer (Web UI)

### MVP: Panel web estático servido por el mismo Axum server

```
Arquitectura:
  ConnectomeDB Server (Axum)
    ├── /api/v1/query     → Motor IQL
    ├── /api/v1/health    → Status
    ├── /metrics           → Prometheus
    └── /ui/               → Archivos estáticos del visualizador
         ├── index.html
         ├── graph.js      → Vis.js / D3.js force-directed graph
         ├── vectors.js    → Plotly.js 2D/3D scatter plot (UMAP projected)
         └── query.js      → IQL editor con syntax highlighting
```

### 3 Paneles del Visualizador:

**Panel 1 — Graph Explorer:**
```
Librería: vis-network (vis.js)
Features:
  - Nodos como círculos con labels
  - Arcos coloreados por tipo de relación
  - Click en nodo → panel lateral con todos los fields
  - BFS animation cuando se ejecuta SIGUE
  - Filtro por TYPE, edge label
```

**Panel 2 — Vector Space:**
```
Librería: Plotly.js (3D scatter)
Features:
  - UMAP projection de vectores a 2D/3D
  - Coloreado por TYPE del nodo
  - Hover para ver metadata de cada punto
  - Highlight de K-nearest neighbors al hacer click
  - Toggle: mostrar/ocultar clusters
```

**Panel 3 — Query Editor:**
```
Librería: CodeMirror 6
Features:
  - Syntax highlighting custom para IQL
  - Autocompletado de tipos y campos
  - Execution plan visual (AST tree)
  - Result table con exportar a CSV/JSON
  - Query history con timestamps
```

---

## 4. Query & AST Visualizer

### Execution Pipeline Interactivo:
```
IQL Input                    Visual Output
─────────                    ─────────────
FROM Persona                 ┌──────────┐
SIGUE 1..3 "amigo" Amigo     │   SCAN   │ → Persona
WHERE bio ~ "rust", 0.8      │  Persona │
FETCH nombre                 └────┬─────┘
                                  │
                             ┌────▼─────┐
                             │ TRAVERSE │ → depth 1..3, "amigo"
                             │   BFS    │
                             └────┬─────┘
                                  │
                             ┌────▼──────────┐
                             │ VECTOR SEARCH │ → bio ~ "rust" (HNSW)
                             │  HNSW k=5    │
                             └────┬──────────┘
                                  │
                             ┌────▼─────┐
                             │ PROJECT  │ → nombre
                             │  FETCH   │
                             └──────────┘
```

### Implementación: Mermaid.js en el web UI
```javascript
// Genera diagrama Mermaid dinámicamente desde LogicalPlan JSON
function planToMermaid(plan) {
  let mmd = "graph TD\n";
  plan.operators.forEach((op, i) => {
    mmd += `  op${i}["${op.type}: ${op.detail}"] --> op${i+1}\n`;
  });
  return mmd;
}
```

---

## 5. Online Playground

### Fase 1 (Mes 3): Embeddable WASM Playground
```
Tecnología: ConnectomeDB compilado a WebAssembly (wasm32-unknown-unknown)
Hosting: GitHub Pages estáticas
Limitaciones: 
  - Sin RocksDB (in-memory only, BTreeMap backend)
  - Sin Ollama (vectores pre-computados)
  - Dataset demo de 1000 nodos precargado

Experiencia:
  1. Usuario abre connectomedb.dev/playground
  2. Editor IQL a la izquierda
  3. Resultados + graph viz a la derecha
  4. Queries de ejemplo clickeables
  5. "Try it locally" CTA → Docker one-liner
```

### Fase 2 (Mes 6): Replit-like con Backend Real
```
Tecnología: Fly.io ephemeral VMs
  - Cada sesión: VM efímera con ConnectomeDB + Ollama preinstalado
  - TTL: 30 minutos por sesión
  - Costo: ~$0.003 por sesión (spot instances ARM)
```

---

## 6. CLI Mejorado (`connectomedb shell`)

### Estado actual:
```
src/bin/connectomedb-cli.rs — REPL básico con rustyline
```

### Mejoras propuestas:
```
Prioridad Alta:
  ✅ Syntax highlighting (colored crate + regex patterns para IQL)
  ✅ Autocompletado de keywords (FROM, WHERE, SIGUE, INSERT, etc.)
  ✅ Output formateado en tabla (tabled crate)
  ✅ Timing de cada query (elapsed time en footer)
  ✅ Multi-line input (detectar query incompleta)

Prioridad Media:
  ◻️ .help → lista de comandos
  ◻️ .schema → muestra todos los TYPES detectados
  ◻️ .stats → count de nodos, edges, vectores
  ◻️ .export <file.json> → dump de resultados
  ◻️ .plan <query> → muestra LogicalPlan sin ejecutar (EXPLAIN)

Prioridad Baja:
  ◻️ .import <file.csv> → bulk insert desde CSV
  ◻️ .benchmark <query> <N> → ejecuta N veces y reporta p50/p99
  ◻️ Themes de color (dark/light/solarized)
```
