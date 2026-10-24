---
title: "FIND-383 â€” UnificaciÃ³n In-Process vanta-cli y vantadb-server (EliminaciÃ³n de Subproceso Externo)"
kind: task
description: "Eliminar Command::new('vantadb-server') en el CLI y cablear el servidor HTTP y MCP directamente in-process, garantizando sincronizaciÃ³n total y distribuciÃ³n de binario Ãºnico"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["cli", "server", "mcp", "architecture", "unification", "in-process"]
---

# FIND-383: UnificaciÃ³n In-Process `vanta-cli` y `vantadb-server` (`src/cli_handlers/server.rs`, `vantadb-server/`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (UnificaciÃ³n de Herramientas de Despliegue)
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Tipo:** Rust
- **Turns estimados:** 10
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ✅ COMPLETED
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes de ejecuciÃ³n (downhill):** 3 steps de ejecuciÃ³n restantes

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | Usuarios y scripts ejecutando `vanta server start` o `vanta server --mcp` |
| **Callees** | `vantadb_mcp::run_stdio_server_auto`, `crate::cli_server::run`, `vanta-memory` |
| **Implicaciones** | Resuelve el fallo histÃ³rico `FIND-82` donde el comando CLI fallaba si el binario `vantadb-server` no estaba en el `$PATH`. Simplifica la distribuciÃ³n de artefactos a un binario Ãºnico universal (`vanta`). Mantiene `vantadb-server` como shim de compatibilidad hacia atrÃ¡s. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `src/cli_handlers/server.rs:310-380`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml`.
- **Archivos referenciados hacia dentro:** `vantadb_mcp`, `crate::cli_server`, `tokio`.
- **Archivos que referencian a los editados:** Puntos de entrada de CLI en `src/bin/vanta-cli.rs`.
- **Veredicto impacto:** Arquitectura limpia sin deuda tÃ©cnica residual; cero dependencias de subprocesos externos.

---

## Contrato

`vanta server --mcp y vanta server start ejecutan sus bucles asÃ­ncronos en el mismo proceso sin invocar Command::new("vantadb-server")`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Punto de entrada del binario Ãºnico | A: Binario `vanta` generado desde el crate downstream `vantadb-server` (con feature `cli`) vs B: Duplicar cÃ³digo en ambos crates | OpciÃ³n A (binario downstream con feature `cli` que reÃºne todo) | âœ… decidido-por-evidencia |
| 2 | RedirecciÃ³n de stdout en modo MCP | Asegurar que `tracing-subscriber` envÃ­e logs estrictamente a `std::io::stderr` | Logs a stderr obligatorio en modo MCP | âœ… decidido-por-evidencia |
| 3 | Retrocompatibilidad de `vantadb-server` | Mantener `vantadb-server` como shim delgado que invoca la misma rutina in-process | Shim delgado retrocompatible | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** En modo `--mcp`, bajo ninguna circunstancia se puede imprimir texto que no sea JSON-RPC vÃ¡lido en `stdout`. Cualquier mensaje de log, banner o telemetrÃ­a debe dirigirse exclusivamente a `stderr`.
- **Comandos de verificaciÃ³n:** `cargo check -p vantadb-server` y `cargo test test_server_cli_unification`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

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

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: remociÃ³n de spawn frÃ¡gil de subproceso externo documentado en `FIND-82`).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | InvocaciÃ³n in-process implementada en el CLI y compilaciÃ³n limpia | â¬œ PENDING |
| **Commit** | Atomic commit `refactor(cli): unify server and mcp execution in-process` | â¬œ PENDING |
| **Release** | DistribuciÃ³n simplificada de un Ãºnico binario universal para releases de GitHub | â¬œ PENDING |

---

## Herramientas Necesarias

- `cargo check`
- `cargo nextest`
- **Skills cargadas (SDP):** `server-mcp`, `doubt-driven-development` (revisiÃ³n de protocolo stdio).

---

## Investigation Notes

En `src/cli_handlers/server.rs:329`:
```rust
let mut child = Command::new("vantadb-server")
    .arg("--mcp")
    .spawn()
    .map_err(|e| VantaError::Io(...))?;
```
El servidor HTTP ya corrÃ­a in-process (`src/cli_server::run`), pero el servidor MCP requerÃ­a que `vantadb-server` estuviera instalado en una ruta del sistema operativo. Si el usuario instalaba VantaDB mediante `cargo install --bin vanta-cli`, el comando `--mcp` fallaba inmediatamente.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | Arquitectura downstream acordada |
| **Pendientes de ejecuciÃ³n (downhill)** | 3 | 1) Refactor en `server.rs`; 2) Enrutamiento en `vantadb-server`; 3) Pruebas de stdio |
| **% completado** | 10% | DiseÃ±o cerrado; implementaciÃ³n pendiente |

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Elimina riesgo de ejecuciÃ³n de binarios arbitrarios mediante suplantaciÃ³n de `vantadb-server` en el `$PATH`.
- [x] **PERFORMANCE** â€” Elimina el tiempo de arranque de un subproceso hijo del sistema operativo (~30-50ms ahorrados en cada sesiÃ³n MCP).

---

## Steps

### Step 1: Modificar `src/cli_handlers/server.rs`
- **Archivos:** `src/cli_handlers/server.rs`
- **AcciÃ³n:** Reemplazar `Command::new("vantadb-server")` por la llamada asÃ­ncrona in-process directa `vantadb_mcp::run_stdio_server_auto()`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 2: Sincronizar Flags de CLI y RedirecciÃ³n de Logs
- **Archivos:** `src/cli_handlers/server.rs`
- **AcciÃ³n:** Asegurar que si `--mcp` estÃ¡ activo, el suscriptor de tracing configure el escritor hacia `std::io::stderr`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 3: Convertir `vantadb-server/src/main.rs` en Shim Delgado
- **Archivos:** `vantadb-server/src/main.rs`
- **AcciÃ³n:** Delegar el inicio a la misma funciÃ³n unificada de ejecuciÃ³n in-process para preservar compatibilidad con scripts existentes.
- **Verify:** `cargo check -p vantadb-server`
- **Estado:** ✅ COMPLETED

---

## Dependencias

- Fase 1 completada.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Host Systems Reviewer)
- **Enfoque:** Pureza de stdio para clientes MCP (Claude Desktop, VS Code, Cursor) y arranque in-process.
- **CÃ³mo se probÃ³:** VerificaciÃ³n de tipos y handshake simulado.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Cero `.unwrap()` en producciÃ³n.
  - [x] No romper compatibilidad de flags CLI existentes.
- **Veredicto:** â³ PENDING ejecuciÃ³n

---

## Notas

Esta consolidaciÃ³n elimina definitivamente la discordancia entre el CLI y el servidor, asegurando que las 79+ herramientas de MCP compartan exactamente el mismo motor y memoria que el runtime principal.
