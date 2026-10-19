---
title: "FIND-383 — Unificación In-Process vanta-cli y vantadb-server (Eliminación de Subproceso Externo)"
kind: task
description: "Eliminar Command::new('vantadb-server') en el CLI y cablear el servidor HTTP y MCP directamente in-process, garantizando sincronización total y distribución de binario único"
status: pending
owners: ["Principal Systems Engineer"]
tags: ["cli", "server", "mcp", "architecture", "unification", "in-process"]
---

# FIND-383: Unificación In-Process `vanta-cli` y `vantadb-server` (`src/cli_handlers/server.rs`, `vantadb-server/`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Unificación de Herramientas de Despliegue)
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Tipo:** Rust
- **Turns estimados:** 10
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes de ejecución (downhill):** 3 steps de ejecución restantes

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | Usuarios y scripts ejecutando `vanta server start` o `vanta server --mcp` |
| **Callees** | `vantadb_mcp::run_stdio_server_auto`, `crate::cli_server::run`, `vanta-memory` |
| **Implicaciones** | Resuelve el fallo histórico `FIND-82` donde el comando CLI fallaba si el binario `vantadb-server` no estaba en el `$PATH`. Simplifica la distribución de artefactos a un binario único universal (`vanta`). Mantiene `vantadb-server` como shim de compatibilidad hacia atrás. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `src/cli_handlers/server.rs:310-380`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml`.
- **Archivos referenciados hacia dentro:** `vantadb_mcp`, `crate::cli_server`, `tokio`.
- **Archivos que referencian a los editados:** Puntos de entrada de CLI en `src/bin/vanta-cli.rs`.
- **Veredicto impacto:** Arquitectura limpia sin deuda técnica residual; cero dependencias de subprocesos externos.

---

## Contrato

`vanta server --mcp y vanta server start ejecutan sus bucles asíncronos en el mismo proceso sin invocar Command::new("vantadb-server")`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Punto de entrada del binario único | A: Binario `vanta` generado desde el crate downstream `vantadb-server` (con feature `cli`) vs B: Duplicar código en ambos crates | Opción A (binario downstream con feature `cli` que reúne todo) | ✅ decidido-por-evidencia |
| 2 | Redirección de stdout en modo MCP | Asegurar que `tracing-subscriber` envíe logs estrictamente a `std::io::stderr` | Logs a stderr obligatorio en modo MCP | ✅ decidido-por-evidencia |
| 3 | Retrocompatibilidad de `vantadb-server` | Mantener `vantadb-server` como shim delgado que invoca la misma rutina in-process | Shim delgado retrocompatible | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** En modo `--mcp`, bajo ninguna circunstancia se puede imprimir texto que no sea JSON-RPC válido en `stdout`. Cualquier mensaje de log, banner o telemetría debe dirigirse exclusivamente a `stderr`.
- **Comandos de verificación:** `cargo check -p vantadb-server` y `cargo test test_server_cli_unification`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo check -p vantadb-server
  evidencia:
    - claim: vanta server --mcp ejecuta in-process y no busca binarios externos en el PATH
      evidencia: src/cli_handlers/server.rs:cmd_server_mcp
      confianza: alta
  artefactos:
    - src/cli_handlers/server.rs
    - vantadb-server/src/main.rs
  invariantes: Ninguna escritura de texto no estructurado en stdout durante el modo MCP
  deuda: ninguna
  queda_pendiente: Verificar empaquetado de release de binarios en CI
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: remoción de spawn frágil de subproceso externo documentado en `FIND-82`).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Invocación in-process implementada en el CLI y compilación limpia | ⬜ PENDING |
| **Commit** | Atomic commit `refactor(cli): unify server and mcp execution in-process` | ⬜ PENDING |
| **Release** | Distribución simplificada de un único binario universal para releases de GitHub | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo check`
- `cargo nextest`
- **Skills cargadas (SDP):** `server-mcp`, `doubt-driven-development` (revisión de protocolo stdio).

---

## Investigation Notes

En `src/cli_handlers/server.rs:329`:
```rust
let mut child = Command::new("vantadb-server")
    .arg("--mcp")
    .spawn()
    .map_err(|e| VantaError::Io(...))?;
```
El servidor HTTP ya corría in-process (`src/cli_server::run`), pero el servidor MCP requería que `vantadb-server` estuviera instalado en una ruta del sistema operativo. Si el usuario instalaba VantaDB mediante `cargo install --bin vanta-cli`, el comando `--mcp` fallaba inmediatamente.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Arquitectura downstream acordada |
| **Pendientes de ejecución (downhill)** | 3 | 1) Refactor en `server.rs`; 2) Enrutamiento en `vantadb-server`; 3) Pruebas de stdio |
| **% completado** | 10% | Diseño cerrado; implementación pendiente |

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Elimina riesgo de ejecución de binarios arbitrarios mediante suplantación de `vantadb-server` en el `$PATH`.
- [x] **PERFORMANCE** — Elimina el tiempo de arranque de un subproceso hijo del sistema operativo (~30-50ms ahorrados en cada sesión MCP).

---

## Steps

### Step 1: Modificar `src/cli_handlers/server.rs`
- **Archivos:** `src/cli_handlers/server.rs`
- **Acción:** Reemplazar `Command::new("vantadb-server")` por la llamada asíncrona in-process directa `vantadb_mcp::run_stdio_server_auto()`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Sincronizar Flags de CLI y Redirección de Logs
- **Archivos:** `src/cli_handlers/server.rs`
- **Acción:** Asegurar que si `--mcp` está activo, el suscriptor de tracing configure el escritor hacia `std::io::stderr`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Convertir `vantadb-server/src/main.rs` en Shim Delgado
- **Archivos:** `vantadb-server/src/main.rs`
- **Acción:** Delegar el inicio a la misma función unificada de ejecución in-process para preservar compatibilidad con scripts existentes.
- **Verify:** `cargo check -p vantadb-server`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Fase 1 completada.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Host Systems Reviewer)
- **Enfoque:** Pureza de stdio para clientes MCP (Claude Desktop, VS Code, Cursor) y arranque in-process.
- **Cómo se probó:** Verificación de tipos y handshake simulado.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Cero `.unwrap()` en producción.
  - [x] No romper compatibilidad de flags CLI existentes.
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

Esta consolidación elimina definitivamente la discordancia entre el CLI y el servidor, asegurando que las 79+ herramientas de MCP compartan exactamente el mismo motor y memoria que el runtime principal.
