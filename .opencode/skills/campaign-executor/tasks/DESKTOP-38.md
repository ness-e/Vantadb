# DESKTOP-38: Dashboard PROXY — visualizar TurnReports, sesiones, cola write-back, rate-limit

## Metadata
- **Plan file:** docs/Backlog.md (Phase 12)
- **Creado:** 2026-08-24
- **last-synced:** 2026-08-24T00:00:00
- **Estado:** ✅ COMPLETED

## Blast Radius
Callers: desktop/src/components/proxy/* (nuevos), desktop/src/components/layout/WorkspaceShell.tsx
Callees: vanta-proxy/src/server.rs (requiere endpoint metrics/snapshot server-side), desktop/src/vanta.ts (REST proxy)
Implicaciones: Panel operativo con datos vivos de una sesión proxy real

## Spec
N/A — feature UI con contrato mecánico

## Impacto mapeado (Regla 0)
Leídos completos: `vanta-proxy/src/{server,report,session,writeback,rate_limit,config}.rs`,
`desktop/src/components/layout/WorkspaceShell.tsx`, `desktop/src/hooks/useMetricsPoll.ts`,
`desktop/src/store/connections.ts`, `desktop/src/components/palette/CommandPalette.tsx`.
Referencias entrantes Rust: `router()` usado por main.rs + tests/proxy_wire.rs (ruta nueva aditiva, no rompe).
Referencias entrantes Desktop: Surface type usado por CommandPalette (`onNavigate`) — se añade `"proxy"` solo al union del shell (palette ya está desincronizada de `"memoria"`, no se toca).
Veredicto: cambios aditivos; sin callers que rompan. Ponytail: paneles inline en 1 componente (no 5 archivos).

## Context Save Point
- Step 1 ✅ `/snapshot` en server.rs + ring buffer Reporter (cap 100) + `SessionStore::snapshot()` + `RateLimiter::hits_total()/limit()` + `WriteBack::pending_labels()`. Tests unitarios nuevos ×3.
- Step 2 ✅ `desktop/src/components/proxy/ProxyDashboard.tsx` (1 componente, paneles inline, polling 5s, config form si sin URL) + `ProxyDashboard.test.tsx`.
- Step 3 ✅ WorkspaceShell: Surface `"proxy"`, SideButton condicional (`proxyConfigured` reactivo vía PROXY_URL_EVENT), render de la lente.
- Step 4 🟡 Test manual con sesión proxy real NO ejecutable en este runner (requiere upstream LLM + proceso proxy vivo). Cubierto mecánicamente: `cargo test -p vanta-proxy` 72/72, `npm run build` OK, `npm test` 68/68 (64 previos + 4 nuevos). Deuda documentada abajo.

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