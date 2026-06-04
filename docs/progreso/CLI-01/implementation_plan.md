# Plan de Implementación: CLI-01 (Modernización de la Interfaz de Línea de Comandos)

## Resumen

Este plan implementa una CLI moderna y profesional para VantaDB utilizando:
- **clap v4** - Parsing de argumentos con derive macros
- **indicatif** - Spinners y barras de progreso
- **console** - Estilo y colores de terminal
- **clap_complete** - Autocompletado para shells

## Objetivos

1. ✅ Reemplazar el parser manual de argumentos con clap
2. ✅ Implementar todos los comandos del plan original
3. ✅ Agregar UX premium con spinners y formato visual
4. ✅ Generar autocompletado para bash, zsh, fish, PowerShell

## Arquitectura

### Estructura del CLI

```rust
#[derive(Parser)]
struct Cli {
    #[arg(short, long, env = "VANTA_DB", default_value = "./db")]
    db: String,
    
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    Put { namespace, key, payload, vector },
    Get { namespace, key },
    List { namespace, limit },
    RebuildIndex,
    AuditIndex { namespace, json, deep },
    RepairTextIndex,
    Export { namespace, out },
    Import { input },
    Query { query, limit },
    Status,
    Completions { shell },
}
```

### Flujo de Operaciones

```
┌─────────────────────────────────────────────────────────────┐
│                       CLI Entry Point                        │
├─────────────────────────────────────────────────────────────┤
│  1. Parse arguments with clap                               │
│  2. Open StorageEngine with VantaConfig                     │
│  3. Execute command handler                                 │
│  4. Display results with console styling                    │
└─────────────────────────────────────────────────────────────┘
```

## Comandos Implementados

### 1. `put` - Almacenar registros
```bash
vanta-cli put --namespace <ns> --key <key> --payload <text> [--vector <f32,f32,...>]
```

### 2. `get` - Recuperar registros
```bash
vanta-cli get --namespace <ns> --key <key>
```

### 3. `list` - Listar registros
```bash
vanta-cli list --namespace <ns> [--limit <n>]
```

### 4. `rebuild-index` - Reconstruir índices
```bash
vanta-cli rebuild-index
```

### 5. `audit-index` - Auditar índice
```bash
vanta-cli audit-index [--namespace <ns>] [--json] [--deep]
```

### 6. `repair-text-index` - Reparar índice
```bash
vanta-cli repair-text-index
```

### 7. `export` - Exportar a JSON
```bash
vanta-cli export [--namespace <ns>] --out <file>
```

### 8. `import` - Importar desde JSON
```bash
vanta-cli import --in <file>
```

### 9. `query` - Ejecutar consultas
```bash
vanta-cli query "<query>" [--limit <n>]
```

### 10. `status` - Dashboard de estado
```bash
vanta-cli status
```

### 11. `completions` - Generar autocompletado
```bash
vanta-cli completions --shell <bash|zsh|fish|powershell>
```

## UX Premium

### Spinners de Progreso
- Usados en operaciones largas (rebuild, import)
- Animación de 10 frames con estilo cyan
- Mensajes de estado actualizables

### Formato Tabular
- Bordes Unicode para tablas
- Alineación de columnas
- Truncamiento inteligente de texto largo

### Dashboard de Estado
```
╔═══════════════════════════════════════════════════════════╗
║               VantaDB Status Dashboard                    ║
╠═══════════════════════════════════════════════════════════╣
║  📁 Database Information                                  ║
║     Path:           ./db                                  ║
║     Backend:        Fjall                                 ║
║     Read-only:      No                                    ║
║  💾 Storage Statistics                                    ║
║     HNSW Nodes:     1234                                  ║
║     Cache entries:  56                                    ║
║     Logical size:   42 MB                                 ║
║     Physical RSS:   38 MB                                 ║
║  ⚡ Performance Metrics                                   ║
║     Startup time:   125 ms                                ║
║     WAL replay:     45 ms (120 records)                   ║
║     ANN rebuild:    890 ms                                ║
╚═══════════════════════════════════════════════════════════╝
```

## Dependencias

```toml
[dependencies]
clap = { version = "4.4", features = ["derive", "env"], optional = true }
clap_complete = { version = "4.4", optional = true }
indicatif = { version = "0.17", optional = true }
console = { version = "0.15", optional = true }

[features]
cli = ["dep:indicatif", "dep:console", "dep:clap", "dep:clap_complete"]
```

## Integración con StorageEngine

### Apertura de Base de Datos
```rust
fn open_database(path: &str, read_only: bool) -> Result<StorageEngine> {
    let config = VantaConfig {
        read_only,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config))
}
```

### Generación de Node ID
```rust
fn memory_node_id(namespace: &str, key: &str) -> u64 {
    let mut hasher = twox_hash::XxHash64::default();
    hasher.write(namespace.as_bytes());
    hasher.write(b"\0");
    hasher.write(key.as_bytes());
    hasher.finish()
}
```

### Campos de Registro
```rust
const FIELD_NAMESPACE: &str = "__vanta_namespace";
const FIELD_KEY: &str = "__vanta_key";
const FIELD_PAYLOAD: &str = "__vanta_payload";
const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
const FIELD_VERSION: &str = "__vanta_version";
```

## Próximos Pasos

1. **Verificar compilación**: `cargo check --features cli --bin vanta-cli`
2. **Ejecutar pruebas manuales** con los comandos de ejemplo
3. **Validar autocompletado** en diferentes shells
4. **Documentar uso** en README.md

## Notas Técnicas

- El CLI usa `StorageEngine` directamente para operaciones básicas
- Para operaciones avanzadas (export/import con SDK), se requiere `VantaEmbedded`
- Las consultas IQL usan el `Executor` con `execute_hybrid()`
- Las métricas se obtienen de `operational_metrics_snapshot()`
