---
title: "AUDIT-05: Perfil Cognitivo McpProfile::Brain y Reparación de Instaladores"
kind: task
status: completed
created_at: "2026-10-31T06:00:00Z"
updated_at: "2026-10-31T06:00:00Z"
owner: "vanta-lead"
category: "mcp-distribution"
tags: [mcp, brain-profile, embed-local, installer, distribution]
description: "Perfil Cognitivo McpProfile::Brain (12 herramientas + vanta_dispatch + protocolo L0→L3) y reparación de instaladores/binarios con embed-local."
---

# AUDIT-05: Perfil Cognitivo McpProfile::Brain y Reparación de Instaladores

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-31-audit-remediation-master.md`
- **Fuente:** Auditoría v0.9.0 / Post-mortem release 0.9.0
- **Esfuerzo:** 🟡 2d
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (MCP) + Scripts de distribución
- **Turns estimados:** 15-20
- **Creado:** 2026-10-31T06:00:00Z
- **last-synced:** 2026-10-31T06:00:00Z
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 4

## Objetivo

Implementar el perfil cognitivo de alta densidad `McpProfile::Brain` en `vantadb-mcp`, diseñado específicamente para asistentes agénticos en IDEs (Claude Code, Cursor, OpenCode). Este perfil expone exactamente 12 herramientas esenciales estructuradas para el protocolo de memoria L0→L3 (L0: captura de hechos crudos, L1: deduplicación y extracción, L2: síntesis de escenas, L3: persona/insights permanentes), integrando el meta-despachador unificado `vanta_dispatch` para reducir el consumo de context window. En paralelo, reparar y validar los scripts de instalación de binarios precompilados (`scripts/install.sh`, `scripts/vanta-install.ps1`) para asegurar que empaqueten y activen la feature `embed-local` sin requerir dependencias dinámicas externas de ONNX Runtime.

## Archivos en scope

- `vantadb-mcp/src/profile.rs`: definición de perfiles MCP y registro de `McpProfile::Brain`.
- `vantadb-mcp/src/tools/**`: implementación de las 12 tools especializadas y de `vanta_dispatch`.
- `vantadb-mcp/src/lib.rs`: orquestación de servidor stdio/SSE y selección de perfil en arranque.
- `scripts/install.sh`: instalador Unix (Linux/macOS).
- `scripts/vanta-install.ps1`: instalador PowerShell para Windows.
- `.github/workflows/release.yml`: workflow de compilación y empaquetado de artefactos con `embed-local`.

## Blast Radius

| Dirección | Módulos |
|---|---|
| Callers | Clientes MCP (Claude Code, Cursor, OpenCode), usuarios de instalación en una línea |
| Callees | `vantadb-mcp`, `vanta-memory` (capas L0..L3), ONNX Runtime estático / `fastembed` |
| Implicaciones | La superficie MCP se compacta facilitando el razonamiento del LLM; los instaladores no fallan por bibliotecas C faltantes. |

## Criterios de aceptación

1. `McpProfile::Brain` compila y expone únicamente las 12 herramientas aprobadas más `vanta_dispatch`.
2. Las 12 herramientas del perfil cubren el ciclo completo:
   - Captura y consulta rápida: `vanta_recall`, `vanta_remember`, `vanta_search`, `vanta_forget`.
   - Navegación y grafo: `vanta_graph_query`, `vanta_nodes_related`.
   - Gobernanza y estado: `vanta_scopes_list`, `vanta_session_checkpoint`, `vanta_stats`.
   - Razonamiento L0→L3: `vanta_scene_synthesize`, `vanta_persona_reflect`, `vanta_export`.
3. `vanta_dispatch` permite invocar sub-acciones internas enviando un payload estructurado JSON.
4. Los instaladores instalan binarios estáticamente enlazados con soporte `embed-local` probado en un entorno limpio sin dependencias adicionales de sistema.

## Contrato

```bash
cargo test -p vantadb-mcp test_profile_brain_tools_manifest
pwsh scripts/vanta-install.ps1 -DryRun
bash scripts/install.sh --dry-run
```

## Steps

### Step 1: Formalizar `McpProfile::Brain` en `profile.rs`
- Definir la variante `Brain` en el enum `McpProfile`.
- Configurar la lista blanca de herramientas y prompts expuestos para esta variante.

### Step 2: Implementar meta-tool `vanta_dispatch`
- Crear el handler que recibe `{ "target": "tool_name", "params": { ... } }`.
- Validar esquemas y retornar respuestas estructuradas al cliente MCP.

### Step 3: Implementar tools cognitivas L0→L3
- Conectar `vanta_scene_synthesize` y `vanta_persona_reflect` con los pipelines correspondientes en `vanta-memory`.
- Conectar `vanta_forget` según la especificación de `AUDIT-03`.

### Step 4: Actualizar scripts de instalación y workflows de release
- Modificar `scripts/install.sh` y `scripts/vanta-install.ps1` para descargar artefactos con `embed-local`.
- Garantizar que los hashes `.sha256` se verifiquen obligatoriamente durante el bootstrap.

## Invariantes de dominio

- Los nombres de las herramientas del perfil `Brain` deben ser inmutables para evitar que prompts de clientes MCP queden obsoletos.
- La ejecución local con `embed-local` no puede abrir sockets no solicitados ni transmitir telemetría sin consentimiento explícito.

