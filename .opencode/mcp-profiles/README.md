# MCP Profiles

OpenCode no soporta filtrado nativo de MCP servers por agente.  
Este sistema permite alternar entre perfiles manualmente.

> **⚠️ Alcance:** Los perfiles solo controlan MCPs configurados en el proyecto (`opencode.jsonc`).  
> MCPs del config global (`~/.config/opencode/opencode.json`) como cargo-mcp, rust-analyzer-mcp,  
> pencil y playwright NO son afectados por estos perfiles.

## Perfiles disponibles

| Perfil | MCPs activos | Para qué |
|--------|-------------|----------|
| **core** | codegraph, metasearchmcp, argus, campaign | Tareas Rust, backend, ingeniería |
| **design** | codegraph, campaign | Diseño UI/visual, frontend |
| **full** | codegraph, discord, metasearchmcp, argus, campaign, lottiefiles-creator (default) | Desarrollo general |

## Cómo cambiar de perfil

```powershell
# Ver perfil actual
.opencode/mcp-profiles/switch-profile.ps1 -Status

# Cambiar a perfil core (deshabilita discord, lottie)
.opencode/mcp-profiles/switch-profile.ps1 -Profile core

# Cambiar a perfil design (solo codegraph + campaign)
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
