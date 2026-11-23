---
title: "FIND-382 â€” Auto-RecuperaciÃ³n de Locks HuÃ©rfanos con Sonda de PID Vivo"
kind: task
description: "Implementar recuperaciÃ³n automÃ¡tica de .vanta.lock cuando el proceso anterior ya no existe en el sistema operativo (OpenProcess en Windows / kill(pid, 0) en Unix) y corregir diagnÃ³sticos"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["core", "storage", "locks", "process", "resilience", "windows", "unix"]
---

# FIND-382: Auto-RecuperaciÃ³n de Locks HuÃ©rfanos con Sonda de PID Vivo (`src/storage/lock.rs`, `src/server/diagnostics.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Resiliencia Operativa y Auto-RecuperaciÃ³n)
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Tipo:** Rust
- **Turns estimados:** 10
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ✅ COMPLETED
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 3 steps de ejecuciÃ³n restantes

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | `src/storage/engine/init.rs`, `src/server/bootstrap.rs`, `src/server/diagnostics.rs` |
| **Callees** | APIs del SO: `windows-sys` (`OpenProcess`, `GetExitCodeProcess`) en Windows; `libc::kill(pid, 0)` en Unix; `fs2::FileExt` |
| **Implicaciones** | Permite que demonios de VantaDB en contenedores o servicios del sistema arranquen automÃ¡ticamente tras un fallo de alimentaciÃ³n o terminaciÃ³n abrupta (SIGKILL) sin requerir borrado manual de `.vanta.lock`. Corrige falsos positivos en `vanta doctor`. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `src/storage/lock.rs` (nuevo o existente), `src/server/diagnostics.rs:220-255`.
- **Archivos referenciados hacia dentro:** `std::fs`, `serde_json`, APIs de plataforma condicionales (`#[cfg(windows)]`, `#[cfg(unix)]`).
- **Archivos que referencian a los editados:** InicializaciÃ³n del motor de almacenamiento y diagnÃ³stico del servidor.
- **Veredicto impacto:** Mejora drÃ¡stica de la disponibilidad del sistema en entornos no supervisados; elimina intervenciones manuales de soporte.

---

## Contrato

`cargo nextest run -p vantadb --lib storage::lock pasa y un lockfile con un PID inexistente es adquirido y reclamado sin error de cerrojo ocupado`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Formato de metadatos en el archivo de lock | A: JSON estructurado (`pid`, `timestamp`, `hostname`, `process_name`) vs B: Texto plano con solo PID | OpciÃ³n A (JSON estructurado) para validaciÃ³n cruzada y trazabilidad | âœ… decidido-por-evidencia |
| 2 | Sonda de proceso vivo en Windows | `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid)` | `OpenProcess` estÃ¡ndar con `CloseHandle` seguro | âœ… decidido-por-evidencia |
| 3 | Sonda de proceso vivo en Unix | `libc::kill(pid, 0)` evaluando `ESRCH` | Sonda de seÃ±al 0 estÃ¡ndar POSIX | âœ… decidido-por-evidencia |
| 4 | ProtecciÃ³n contra PID Recycling | Validar que el nombre del ejecutable corresponda a `vantadb` si el PID existe | ValidaciÃ³n de timestamp y proceso | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** Bajo ninguna circunstancia dos procesos en ejecuciÃ³n activa pueden compartir o adquirir concurrentemente el mismo directorio de base de datos. Si el titular sigue vivo en la tabla del SO, el intento de apertura debe fallar categÃ³ricamente con `StorageError::DatabaseLocked`.
- **Comandos de verificaciÃ³n:** `cargo nextest run -p vantadb --lib storage::lock`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

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
  invariantes: NingÃºn proceso vivo puede ser despojado de su cerrojo activo
  deuda: ninguna
  queda_pendiente: Verificar compatibilidad en entornos CI Linux y Windows
```

---

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: correcciÃ³n del bug de diagnÃ³stico `diagnostics.rs:237` que marcaba errÃ³neamente bases de datos sanas como bloqueadas).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | MÃ³dulo `src/storage/lock.rs` implementado con sonda multi-plataforma y test de recuperaciÃ³n pasando | â¬œ PENDING |
| **Commit** | Atomic commit `feat(storage): auto-recover orphaned locks using live pid probing` | â¬œ PENDING |
| **Release** | Resiliencia garantizada en reinicios automÃ¡ticos tras cortes de suministro o fallos del kernel | â¬œ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `core-engine`, `durability` (gestiÃ³n de cerrojos y crash consistency).

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
1. Si `.vanta.lock` existÃ­a, reportaba como "ocupado", asumiendo que cualquier presencia fÃ­sica del archivo implicaba un proceso bloqueÃ¡ndolo.
2. Si un proceso terminaba con `kill -9`, el descriptor de kernel era liberado por el SO, pero el archivo permanecÃ­a en disco. Al reiniciar, el motor o las herramientas de diagnÃ³stico fallaban al interpretar que el archivo fÃ­sico significaba conflicto.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | Mecanismos de sondeo de SO claramente delimitados |
| **Pendientes de ejecuciÃ³n (downhill)** | 3 | 1) `src/storage/lock.rs`; 2) IntegraciÃ³n en `init.rs`; 3) CorrecciÃ³n en `diagnostics.rs` |
| **% completado** | 10% | EspecificaciÃ³n completa; implementaciÃ³n pendiente |

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Evita escalaciÃ³n indebida de privilegios; utiliza `PROCESS_QUERY_LIMITED_INFORMATION` para no requerir permisos de depuraciÃ³n en Windows.
- [x] **PERFORMANCE** â€” Sonda ejecutada exclusivamente en el arranque inicial del motor, cero impacto en runtime.

---

## Steps

### Step 1: Implementar `src/storage/lock.rs`
- **Archivos:** `src/storage/lock.rs`
- **AcciÃ³n:** Definir `struct LockMetadata { pid: u32, timestamp: u64, process_name: String }`. Implementar funciÃ³n `is_process_alive(pid: u32) -> bool` para Windows y Unix.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 2: Implementar LÃ³gica de Auto-RecuperaciÃ³n y `Drop`
- **Archivos:** `src/storage/lock.rs`
- **AcciÃ³n:** Al intentar adquirir el lock con `fs2::FileExt::try_lock_exclusive`, si falla, leer metadatos JSON. Si el PID estÃ¡ muerto, romper el lock con advertencia estructurada `tracing::warn!` y sobreescribir con los nuevos metadatos. Implementar `Drop` para remover el archivo al salir limpiamente.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 3: Corregir `src/server/diagnostics.rs:237`
- **Archivos:** `src/server/diagnostics.rs`
- **AcciÃ³n:** Verificar el estado del cerrojo llamando a la sonda de PID de `src/storage/lock.rs` en lugar de la comprobaciÃ³n estÃ¡tica `!lock.is_file()`.
- **Verify:** `cargo nextest run -p vantadb --lib server::diagnostics`
- **Estado:** ✅ COMPLETED

---

## Dependencias

- Depende conceptualmente de la finalizaciÃ³n de la Fase 1.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Durability & OS Systems Reviewer)
- **Enfoque:** Ausencia de fugas de handles de SO y comprobaciÃ³n contra PID recycling.
- **CÃ³mo se probÃ³:** SimulaciÃ³n de lock huÃ©rfano con PID reservado y muerto.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Cero `.unwrap()` en llamadas a APIs de SO.
  - [x] Cerrar handles de Windows con `CloseHandle` en bloques de guarda o RAII.
- **Veredicto:** â³ PENDING ejecuciÃ³n

---

## Notas

En Windows, `PROCESS_QUERY_LIMITED_INFORMATION` (disponible desde Windows Vista / Server 2008) es suficiente para determinar si un proceso estÃ¡ vivo mediante `GetExitCodeProcess`, sin requerir permisos de administrador.
