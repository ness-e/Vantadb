# OLD-11: CLI/TUI Interactivo

**Fuente:** Backlog Phase 9 (Old Docs Rescue)  
**Estado:** ⚠️ Parcial (CLI completo 46 tests, TUI no implementado)  
**Effort:** 🟡 1-2 sem  
**Dependencias:** Ninguna. Proyecto aparte (puede ser crate separado)

## Gate
✅ DO — CLI completo existe (`src/cli.rs`, `src/cli_server.rs`, `src/cli/`). TUI spec de 1106 líneas existe. Falta implementar TUI.

## Objetivo
Implementar TUI interactivo para VantaDB usando `ratatui` + `crossterm`. 
- Modo monitor: watch queries, WAL status, memory usage en tiempo real
- Modo REPL: query interactiva con output formateado
- Modo dashboard: vista general del engine

## Archivos

| Archivo | Qué hacer |
|---------|-----------|
| `src/cli/tui.rs` (new) | Entry point del TUI: bucle principal con ratatui |
| `src/cli/tui_dashboard.rs` (new) | Panel de dashboard: stats engine, WAL, memoria |
| `src/cli/tui_monitor.rs` (new) | Monitor de queries en tiempo real |
| `src/cli/tui_repl.rs` (new) | REPL interactivo con historial |
| `src/cli/mod.rs` | Agregar subcomando `vantadb tui` |
| `Cargo.toml` | Agregar dep: `ratatui`, `crossterm` (feature-gated) |

## Pasos

### 1. Leer spec TUI existente
Buscar spec de 1106 líneas en `docs/` o `spec/`.

### 2. Agregar dependencias
```toml
# feature "tui"
ratatui = { version = "0.28", optional = true }
crossterm = { version = "0.28", optional = true }
```

### 3. Implementar TUI loop básico
```rust
// src/cli/tui.rs
pub fn run_tui(engine: Arc<Engine>) -> Result<()> {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(|frame| { /* render */ })?;
        // handle input
    }
}
```

### 4. Subcomando CLI
En `src/cli.rs` o `src/main.rs`:
```rust
Command::Tui => run_tui(engine)?,
```

### 5. Tests
- Test que `vantadb tui` arranca con engine vacío
- Test que dashboard renderiza stats básicas
- Test que REPL acepta query y muestra resultado

### 6. Verificación
```bash
cargo check --features tui -p vantadb
cargo nextest run --features tui -p vantadb -- cli::tui
```

### 7. Progreso
- Marcar OLD-11 ✅ en Backlog.md
- Agregar entry en progreso/README.md
- Auto-commit
