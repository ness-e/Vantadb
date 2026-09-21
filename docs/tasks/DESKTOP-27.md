# DESKTOP-27: Docs + ADR Vanta Studio — README desktop + ARCHITECTURE.md modelo real (transporte pluggable)

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETED

## Blast Radius
Callers: docs/desktop/ (nuevo), docs/architecture/adr/
Callees: ADR-026, ADR-027, ADR-028 (ya existen)
Implicaciones: Documentación completa del modelo Studio para usuarios y desarrolladores

## Spec
N/A — documentación con contrato mecánico

## Contrato
`ls docs/desktop/README.md docs/desktop/ARCHITECTURE.md` existen; ADR del modelo Studio revisado por vanta-arch; guía cubre nativo + server + wasm

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Crear README desktop ✅
- **Archivos:** `docs/desktop/README.md` (nuevo)
- **Acción:** Documentar: instalación, modo transporte pluggable (nativo embebida / HTTP `/api/v2/*` / WASM-OPFS), comandos Tauri expuestos, desarrollo local, troubleshooting
- **Verify:** Archivo existe y es legible

### Step 2: Crear ARCHITECTURE.md del modelo real ✅
- **Archivos:** `docs/desktop/ARCHITECTURE.md` (nuevo)
- **Acción:** Documentar arquitectura: `ConnectionManager` (registry + active_id), transportes (NativeConnection, ServerConnection, WasmConnection), `ConnectionSelector` eliminado (ADMIN-03), path lock via NativeConnection, shutdown_all lifecycle. Referenciar ADR-026/027/028 — NO duplicar
- **Verify:** Archivo existe y es consistente con código

### Step 3: Guía de usuario por modo de transporte ✅
- **Archivos:** `docs/desktop/GUIDE.md` (nuevo)
- **Acción:** Guías separadas: (1) Nativo embebido (default, máximo rendimiento), (2) Server HTTP (remoto, multi-usuario, auth Bearer), (3) WASM-OPFS (standalone, offline, demo). Capturas de pantalla opcionales
- **Verify:** Guía cubre nativo + server + wasm

### Step 4: Revisión vanta-arch del ADR modelo Studio ✅
- **Resultado:** APPROVE (ses_fcdaa5aa2fferCAypE3ZKSuN4f). 7/7 claims verificados contra código real; detalle menor incorporado en ARCHITECTURE.md (fallback de active_id tras remove()).
- **Archivos:** `docs/architecture/adr/` (existentes)
- **Acción:** Solicitar revisión a vanta-arch del modelo transporte pluggable. Incorporar feedback
- **Verify:** ADR del modelo Studio revisado por vanta-arch

## Dependencias
- ADR-026/027/028 ya existen (verificados en backlog)

## Notas
- DoD: ADR del modelo Studio revisado por vanta-arch; guía cubre nativo + server + wasm
- No duplicar ADRs existentes — referenciarlos