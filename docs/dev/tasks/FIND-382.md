---
title: "FIND-382 — Auto-Recuperación de Locks Huérfanos con Sonda de PID Vivo"
kind: task
description: "Implementar recuperación automática de .vanta.lock cuando el proceso anterior ya no existe en el sistema operativo (OpenProcess en Windows / kill(pid, 0) en Unix) y corregir diagnósticos"
status: pending
owners: ["Principal Systems Engineer"]
tags: ["core", "storage", "locks", "process", "resilience", "windows", "unix"]
---

# FIND-382: Auto-Recuperación de Locks Huérfanos con Sonda de PID Vivo (`src/storage/lock.rs`, `src/server/diagnostics.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Resiliencia Operativa y Auto-Recuperación)
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Tipo:** Rust
- **Turns estimados:** 10
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 3 steps de ejecución restantes

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | `src/storage/engine/init.rs`, `src/server/bootstrap.rs`, `src/server/diagnostics.rs` |
| **Callees** | APIs del SO: `windows-sys` (`OpenProcess`, `GetExitCodeProcess`) en Windows; `libc::kill(pid, 0)` en Unix; `fs2::FileExt` |
| **Implicaciones** | Permite que demonios de VantaDB en contenedores o servicios del sistema arranquen automáticamente tras un fallo de alimentación o terminación abrupta (SIGKILL) sin requerir borrado manual de `.vanta.lock`. Corrige falsos positivos en `vanta doctor`. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/lock.rs` (nuevo o existente), `src/server/diagnostics.rs:220-255`.
- **Archivos referenciados hacia dentro:** `std::fs`, `serde_json`, APIs de plataforma condicionales (`#[cfg(windows)]`, `#[cfg(unix)]`).
- **Archivos que referencian a los editados:** Inicialización del motor de almacenamiento y diagnóstico del servidor.
- **Veredicto impacto:** Mejora drástica de la disponibilidad del sistema en entornos no supervisados; elimina intervenciones manuales de soporte.

---

## Contrato

`cargo nextest run -p vantadb --lib storage::lock pasa y un lockfile con un PID inexistente es adquirido y reclamado sin error de cerrojo ocupado`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Formato de metadatos en el archivo de lock | A: JSON estructurado (`pid`, `timestamp`, `hostname`, `process_name`) vs B: Texto plano con solo PID | Opción A (JSON estructurado) para validación cruzada y trazabilidad | ✅ decidido-por-evidencia |
| 2 | Sonda de proceso vivo en Windows | `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)` | `OpenProcess` estándar con `CloseHandle` seguro | ✅ decidido-por-evidencia |
| 3 | Sonda de proceso vivo en Unix | `libc::kill(pid, 0)` evaluando `ESRCH` | Sonda de señal 0 estándar POSIX | ✅ decidido-por-evidencia |
| 4 | Protección contra PID Recycling | Validar que el nombre del ejecutable corresponda a `vantadb` si el PID existe | Validación de timestamp y proceso | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** Bajo ninguna circunstancia dos procesos en ejecución activa pueden compartir o adquirir concurrentemente el mismo directorio de base de datos. Si el titular sigue vivo en la tabla del SO, el intento de apertura debe fallar categóricamente con `StorageError::DatabaseLocked`.
- **Comandos de verificación:** `cargo nextest run -p vantadb --lib storage::lock`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib storage::lock
  evidencia:
    - claim: Si el PID registrado en .vanta.lock no existe en el sistema operativo, el motor reclama el cerrojo limpiamente
      evidencia: src/storage/lock.rs:test_orphan_lock_recovery
      confianza: alta
  artefactos:
    - src/storage/lock.rs
    - src/server/diagnostics.rs
  invariantes: Ningún proceso vivo puede ser despojado de su cerrojo activo
  deuda: ninguna
  queda_pendiente: Verificar compatibilidad en entornos CI Linux y Windows
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: corrección del bug de diagnóstico `diagnostics.rs:237` que marcaba erróneamente bases de datos sanas como bloqueadas).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Módulo `src/storage/lock.rs` implementado con sonda multi-plataforma y test de recuperación pasando | ⬜ PENDING |
| **Commit** | Atomic commit `feat(storage): auto-recover orphaned locks using live pid probing` | ⬜ PENDING |
| **Release** | Resiliencia garantizada en reinicios automáticos tras cortes de suministro o fallos del kernel | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `core-engine`, `durability` (gestión de cerrojos y crash consistency).

---

## Investigation Notes

En `src/server/diagnostics.rs:237`:
```rust
let lock = path.join(".vanta.lock");
if !lock.is_file() {
    report.healthy = false;
    report.messages.push("Lock file missing or invalid".into());
}
```
Esto causaba dos problemas graves:
1. Si `.vanta.lock` existía, reportaba como "ocupado", asumiendo que cualquier presencia física del archivo implicaba un proceso bloqueándolo.
2. Si un proceso terminaba con `kill -9`, el descriptor de kernel era liberado por el SO, pero el archivo permanecía en disco. Al reiniciar, el motor o las herramientas de diagnóstico fallaban al interpretar que el archivo físico significaba conflicto.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Mecanismos de sondeo de SO claramente delimitados |
| **Pendientes de ejecución (downhill)** | 3 | 1) `src/storage/lock.rs`; 2) Integración en `init.rs`; 3) Corrección en `diagnostics.rs` |
| **% completado** | 10% | Especificación completa; implementación pendiente |

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Evita escalación indebida de privilegios; utiliza `PROCESS_QUERY_LIMITED_INFORMATION` para no requerir permisos de depuración en Windows.
- [x] **PERFORMANCE** — Sonda ejecutada exclusivamente en el arranque inicial del motor, cero impacto en runtime.

---

## Steps

### Step 1: Implementar `src/storage/lock.rs`
- **Archivos:** `src/storage/lock.rs`
- **Acción:** Definir `struct LockMetadata { pid: u32, timestamp: u64, process_name: String }`. Implementar función `is_process_alive(pid: u32) -> bool` para Windows y Unix.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Implementar Lógica de Auto-Recuperación y `Drop`
- **Archivos:** `src/storage/lock.rs`
- **Acción:** Al intentar adquirir el lock con `fs2::FileExt::try_lock_exclusive`, si falla, leer metadatos JSON. Si el PID está muerto, romper el lock con advertencia estructurada `tracing::warn!` y sobreescribir con los nuevos metadatos. Implementar `Drop` para remover el archivo al salir limpiamente.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Corregir `src/server/diagnostics.rs:237`
- **Archivos:** `src/server/diagnostics.rs`
- **Acción:** Verificar el estado del cerrojo llamando a la sonda de PID de `src/storage/lock.rs` en lugar de la comprobación estática `!lock.is_file()`.
- **Verify:** `cargo nextest run -p vantadb --lib server::diagnostics`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Depende conceptualmente de la finalización de la Fase 1.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Durability & OS Systems Reviewer)
- **Enfoque:** Ausencia de fugas de handles de SO y comprobación contra PID recycling.
- **Cómo se probó:** Simulación de lock huérfano con PID reservado y muerto.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Cero `.unwrap()` en llamadas a APIs de SO.
  - [x] Cerrar handles de Windows con `CloseHandle` en bloques de guarda o RAII.
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

En Windows, `PROCESS_QUERY_LIMITED_INFORMATION` (disponible desde Windows Vista / Server 2008) es suficiente para determinar si un proceso está vivo mediante `GetExitCodeProcess`, sin requerir permisos de administrador.
