# MCP Profiles

OpenCode no soporta filtrado nativo de MCP servers por agente.  
Este sistema permite alternar entre perfiles manualmente.

## Perfiles disponibles

| Perfil | MCPs activos | Para qué |
|--------|-------------|----------|
| **core** | codegraph, cargo-mcp, rust-analyzer-mcp, metasearchmcp, argus, campaign | Tareas Rust, backend, ingeniería |
| **design** | codegraph, pencil, playwright, campaign | Diseño UI/visual, frontend |
| **full** | Todos (el default) | Desarrollo general |
| **social** | codegraph, discord, metasearchmcp, argus, campaign | Discord, web scraping |

## Cómo cambiar de perfil

```powershell
# Ver perfil actual
.opencode/mcp-profiles/switch-profile.ps1 -Status

# Cambiar a perfil core (deshabilita pencil, discord, lottie)
.opencode/mcp-profiles/switch-profile.ps1 -Profile core

# Cambiar a perfil design (habilita pencil, deshabilita cargo-mcp)
.opencode/mcp-profiles/switch-profile.ps1 -Profile design

# Volver a full (todo habilitado)
.opencode/mcp-profiles/switch-profile.ps1 -Profile full

# Ver qué perfiles existen
.opencode/mcp-profiles/switch-profile.ps1 -List
```

## Notas

- El cambio es inmediato — solo modifica `enabled: true/false` en `opencode.jsonc`
- OpenCode debe reiniciarse (o recargar MCPs) para que el cambio surta efecto
- Los perfiles no borran config, solo alternan `enabled`
