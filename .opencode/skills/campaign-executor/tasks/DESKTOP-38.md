# DESKTOP-38: Dashboard PROXY — visualizar TurnReports, sesiones, cola write-back, rate-limit

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ⬜ PENDING

## Blast Radius
Callers: desktop/src/components/proxy/* (nuevos), desktop/src/components/layout/WorkspaceShell.tsx
Callees: vanta-proxy/src/server.rs (requiere endpoint metrics/snapshot server-side), desktop/src/vanta.ts (REST proxy)
Implicaciones: Panel operativo con datos vivos de una sesión proxy real

## Spec
N/A — feature UI con contrato mecánico

## Contrato
`cd desktop && npm run build`; panel operativo con datos vivos de una sesión proxy real

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: Verificar/extender vanta-proxy endpoint metrics
- **Archivos:** `vanta-proxy/src/server.rs`, `vanta-proxy/src/metrics.rs` (nuevo o extendido)
- **Acción:** Verificar qué expone `/health` hoy. Requerir endpoint `/metrics` o `/snapshot` que devuelva: TurnReports (protocolo/modelo/status/duración), sesiones activas (team→agent→task con TTL), cola write-back pendiente, rate-limit hits. Coordinar con owner de vanta-proxy
- **Verify:** `cargo check -p vanta-proxy`; endpoint responde JSON con estructura esperada

### Step 2: Crear componentes Dashboard PROXY
- **Archivos:** `desktop/src/components/proxy/ProxyDashboard.tsx` (nuevo), `desktop/src/components/proxy/TurnReportsPanel.tsx`, `desktop/src/components/proxy/SessionsPanel.tsx`, `desktop/src/components/proxy/WriteBackPanel.tsx`, `desktop/src/components/proxy/RateLimitPanel.tsx`
- **Acción:** UI consumiendo REST del proxy (no bridge nativo — proxy corre como proceso aparte). Paneles: (1) TurnReports: tabla protocolo/modelo/status/duración, (2) Sesiones: árbol team→agent→task con TTL, (3) Write-back: cola pendiente con estado, (4) Rate-limit: hits + config. Polling configurable (5-10s). Design system Studio
- **Verify:** `cd desktop && npm run build`

### Step 3: Integrar en WorkspaceShell
- **Archivos:** `desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/components/layout/Sidebar.tsx`
- **Acción:** Añadir "PROXY" como lente en sidebar (condicional: solo si proxy configurado). Al click: carga ProxyDashboard
- **Verify:** Navegación funcional

### Step 4: Test con sesión proxy real
- **Archivos:** `desktop/src/components/proxy/*.test.tsx` (extender DESKTOP-26)
- **Acción:** Levantar `vanta-proxy` real con sesión activa. Verificar: datos vivos en paneles, polling actualiza, rate-limit hits visibles
- **Verify:** Test manual con proxy real funcional

## Dependencias
- vanta-proxy endpoint metrics/snapshot (requiere trabajo en crate `vanta-proxy` previo) — BLOQUEANTE
- DESKTOP-28 (design system unificado) — complementaria

## Notas
- DoD: panel operativo con datos vivos de una sesión proxy real
- ⚠️ Pre-requisito: vanta-proxy solo expone `/health` hoy — requiere endpoint metrics/snapshot server-side primero
- UI consume REST del proxy (no bridge nativo)
- Esfuerzo 🟡, Prio 🔵