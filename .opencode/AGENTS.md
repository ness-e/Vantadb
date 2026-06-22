# Directivas del Proyecto VantaDB (Rust Core)

## Flujo de Progreso

Este proyecto VantaDB usa un skill de progreso para mantener el historial unificado:
`./.opencode/skills/progreso/SKILL.md`

- **Al iniciar una nueva tarea:** carga el skill `progreso` y sigue sus triggers.
- **Al completar una tarea:** aplica el skill `progreso` (Trigger 1) ANTES de cualquier mensaje de resumen.
- **Backlog maestro:** `C:\Users\Eros\Obsidian\Eros\Backlog.md`
- **Changelog:** `C:\Users\Eros\Obsidian\Eros\Changelog.md`

## Comportamiento General

- Preserva siempre el contenido de `./docs/progreso/README.md` — es el historial inmutable del proyecto.
- No sobrescribas archivos de planificación sin antes haber consolidado la tarea anterior en el historial unificado.

---

## Skills del Sistema Disponibles para Rust

Estos skills están disponibles globalmente y deben cargarse cuando el caso de uso lo requiera. NO se cargan automáticamente.

### Skills Base de Datos y Esquemas
- **`prisma-expert`**: Schema design, migrations, query optimization, relations modeling. Cargar cuando se trabaje con prisma-client-rust o esquemas de DB.
- **`prisma`**: Type-safe database operations y schema design con Prisma ORM.
- **`database-schema-designer`**: Diseño de esquemas SQL/NoSQL — normalización, indexing, migraciones.
- **`supabase-postgres-best-practices`**: Optimización Postgres, queries, schema design.

### Skills Calidad y Debugging
- **`systematic-debugging`**: Debugging metódico — usar ANTE de proponer fixes ante cualquier bug o test failure.
- **`pr-feedback-quality-gate`**: Tracking de PR feedback, resolución de merge conflicts, validación de fixes.

### Skills Diseño de APIs
- **`api-design-principles`**: Diseño de APIs REST y GraphQL — usar al diseñar nuevas APIs Rust (axum, actix, tonic).

---

## Herramientas CLI Esenciales para Rust

Instalables bajo demanda. El agente puede proponer su instalación cuando el caso de uso lo requiera:

```bash
# === CALIDAD DE CÓDIGO ===
cargo install cargo-machete       # Detectar dependencias no usadas
cargo install cargo-audit         # Auditoría de vulnerabilidades (RustSec DB)
cargo install cargo-deny          # Licencias, bans, advisories
cargo install cargo-outdated      # Mostrar dependencias desactualizadas
cargo install cargo-tree          # Árbol de dependencias visual
cargo install cargo-llvm-cov      # Cobertura de código (LLVM)

# === TESTING ===
cargo install cargo-nextest       # Test runner paralelo más rápido
cargo install cargo-watch         # Rebuild/test automático en save
cargo install bacon               # Background check con feedback visual

# === MACROS & METADATA ===
cargo install cargo-expand        # Expandir macros para debugging
cargo install cargo-edit          # add/rm/upgrade deps desde CLI
cargo install cargo-modules       # Visualizar estructura de módulos

# === MCP SERVERS ===
cargo install cargo-mcp           # MCP server para comandos Cargo
cargo install rust-mcp-server     # MCP server completo para Rust
cargo install rust-analyzer-mcp   # MCP server para rust-analyzer LSP
```

---

## MCP Servers para Rust (Configuración Referencial)

Estos MCP servers están disponibles para conectar asistentes AI al toolchain Rust. Se configuran en `opencode.json` bajo `mcpServers` cuando se necesiten.

### 1. cargo-mcp
- **Qué hace**: Ejecuta `cargo check`, `clippy`, `test`, `build`, `fmt`, `add`, `remove`, `bench`, `run`
- **Instalación**: `cargo install cargo-mcp`
- **Config**:
```json
{
  "cargo-mcp": {
    "command": "cargo-mcp",
    "args": ["serve"]
  }
}
```

### 2. rust-analyzer-mcp (zeenix)
- **Qué hace**: Integración LSP completa — hover, goto def, references, completions, diagnostics, rename, format
- **Instalación**: `cargo install rust-analyzer-mcp`
- **Requiere**: `rustup component add rust-analyzer`
- **Config**:
```json
{
  "rust-analyzer-mcp": {
    "command": "rust-analyzer-mcp"
  }
}
```

### 3. rust-mcp-server
- **Qué hace**: Bridge completo — build, test, deps, clippy, doc, project management, dependency management
- **Instalación**: `cargo install rust-mcp-server`
- **Config**:
```json
{
  "rust-mcp": {
    "command": "rust-mcp-server"
  }
}
```

### Notas sobre MCPs
- Son GRATUITOS y corren 100% local (no requieren API keys externas).
- Las tool definitions consumen ~500-1000 tokens c/u. Activar solo las necesarias.
- El agente sugerirá activar un MCP server específico cuando detecte el caso de uso.

---

## Skills Locales del Proyecto

Estos skills están en `./skills/` y `./.opencode/skills/`:

- **`vantadb-mcp`** (`./skills/vantadb-mcp/SKILL.md`): Integración MCP de VantaDB para memoria persistente AI.
- **`vantadb`** (`./skills/vantadb/SKILL.md`): Guía experta de VantaDB — core operations, hybrid search, SDK Python/Rust, integraciones LangChain/LlamaIndex.
- **`progreso`** (`./.opencode/skills/progreso/SKILL.md`): Historial unificado de progreso del proyecto.
