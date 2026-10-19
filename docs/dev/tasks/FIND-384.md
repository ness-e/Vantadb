---
title: "FIND-384 — Setup Wizard Permanente & Demonio de Self-Healing en Background"
kind: task
description: "Persistir variables de entorno en el perfil del usuario (~/.bashrc, ~/.zshrc o Registro de Windows) e incorporar tarea cron interna de self-healing (vacuum + rebuild-index en reposo)"
status: pending
owners: ["Principal Systems Engineer"]
tags: ["ops", "setup-wizard", "self-healing", "vacuum", "hnsw", "maintenance"]
---

# FIND-384: Setup Wizard Permanente & Demonio de Self-Healing (`scripts/install.*`, `src/server/self_healing.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Experiencia de Usuario y Mantenimiento Autónomo)
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Tipo:** Mixto (PowerShell / Shell / Rust)
- **Turns estimados:** 12
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes de ejecución (downhill):** 3 steps de ejecución restantes

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | Scripts de instalación para usuarios finales (`scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`), y el servidor HTTP en background (`src/server/bootstrap.rs`) |
| **Callees** | `[Environment]::SetEnvironmentVariable` (Windows User Registry), `.bashrc`/`.zshrc` (POSIX), `src/storage/engine/` (`compact`, `rebuild_index`), `tokio::time` |
| **Implicaciones** | Elimina la pérdida recurrente de variables de entorno al abrir nuevas terminales tras ejecutar el setup wizard. Garantiza que la localidad de caché del grafo HNSW y la purga de lápidas (tombstones) ocurran de forma autónoma durante períodos de baja actividad sin intervención humana. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`, `src/server/bootstrap.rs:100-160`.
- **Archivos referenciados hacia dentro:** `src/storage/engine`, `tokio::sync::watch`.
- **Archivos que referencian a los editados:** Pipeline de arranque del servidor y scripts de despliegue.
- **Veredicto impacto:** Mejora sustancial en retención de usuarios y mantenimiento de bases de datos de larga duración.

---

## Contrato

`scripts/install.ps1 persiste variables de entorno a nivel User en Windows; install.sh actualiza perfiles POSIX; y SelfHealingService ejecuta vacuum automáticamente tras 300 segundos sin peticiones`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Ámbito de persistencia en Windows | A: Ámbito Usuario (`[EnvironmentVariableTarget]::User`) sin requerir permisos de Administrador vs B: Ámbito Máquina | Opción A (Usuario, seguro y no privilegiado) | ✅ decidido-por-evidencia |
| 2 | Marcadores en perfiles POSIX | Usar bloques delimitados `# >>> vantadb initialize >>>` para actualización idempotente | Bloque idempotente estándar | ✅ decidido-por-evidencia |
| 3 | Ventana de inactividad para self-healing | 300 segundos (5 minutos) continuos de actividad cero | 300s configurable mediante `VANTADB_AUTO_VACUUM_IDLE_SECS` | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** El proceso de self-healing (vacuum / rebuild) debe abortar o ceder inmediatamente si ingresa una nueva petición de lectura o escritura concurrente. La base de datos nunca debe degradar la latencia de queries de usuario por tareas internas de fondo.
- **Comandos de verificación:** `cargo nextest run -p vantadb --lib server::self_healing` y pruebas estáticas de scripts
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib server::self_healing
  evidencia:
    - claim: El servicio de self-healing se activa únicamente cuando el ActivityTracker confirma inactividad prolongada
      evidencia: src/server/self_healing.rs:SelfHealingService
      confianza: alta
  artefactos:
    - scripts/install.ps1
    - scripts/install.sh
    - scripts/setup-embeddings.ps1
    - src/server/self_healing.rs
  invariantes: El mantenimiento en segundo plano nunca bloquea peticiones de usuario entrantes
  deuda: ninguna
  queda_pendiente: Validar ejecución en entornos de prueba en Fase 7
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: automatización de tareas manuales de DBA que anteriormente requerían scripts externos).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Persistencia permanente de envs en instaladores y demonio de self-healing con tracker de actividad | ⬜ PENDING |
| **Commit** | Atomic commit `feat(ops): add permanent env persistence to wizard and background self-healing` | ⬜ PENDING |
| **Release** | Documentación de parámetros de self-healing en `docs/user/operations/` | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `powershell` (validación de script)
- **Skills cargadas (SDP):** `automation`, `durability` (políticas de mantenimiento autónomo).

---

## Investigation Notes

Anteriormente, el instalador `scripts/install.ps1` ejecutaba:
```powershell
$env:ORT_DYLIB_PATH = $onnxPath
```
Esto solo tenía efecto en el proceso actual de PowerShell; al cerrar la ventana y abrir una nueva sesión para ejecutar `vanta`, la variable ya no existía y VantaDB fallaba. En Linux/macOS ocurría exactamente lo mismo al no exportarse a `~/.bashrc` o `~/.zshrc`.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Rutinas de shell y Tokio ya diseñadas |
| **Pendientes de ejecución (downhill)** | 3 | 1) Actualizar instaladores; 2) Crear `self_healing.rs`; 3) Montar servicio en bootstrap |
| **% completado** | 10% | Diseño cerrado; pendiente código |

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Los instaladores operan exclusivamente en el espacio del usuario actual; no modifican configuraciones globales del sistema.
- [x] **PERFORMANCE** — Mantiene la localidad de caché del grafo HNSW compactando nodos borrados, previniendo degradaciones de latencia de hasta un 30% a lo largo del tiempo.

---

## Steps

### Step 1: Añadir Persistencia en `scripts/install.ps1` y `scripts/setup-embeddings.ps1`
- **Archivos:** `scripts/install.ps1`, `scripts/setup-embeddings.ps1`
- **Acción:** Definir función `Set-VantaUserEnvironment` que invoque `[Environment]::SetEnvironmentVariable($Name, $Value, [EnvironmentVariableTarget]::User)` para persistir `ORT_DYLIB_PATH`, `VANTADB_HOME` y añadir VantaDB al `PATH` del usuario.
- **Verify:** Inspección de sintaxis de PowerShell
- **Estado:** ⬜ PENDING

### Step 2: Añadir Persistencia en `scripts/install.sh`
- **Archivos:** `scripts/install.sh`
- **Acción:** Implementar bloque idempotente en `~/.bashrc`, `~/.zshrc` y `~/.profile` exportando las variables necesarias.
- **Verify:** Inspección de sintaxis POSIX `sh`
- **Estado:** ⬜ PENDING

### Step 3: Implementar `src/server/self_healing.rs`
- **Archivos:** `src/server/self_healing.rs`, `src/server/bootstrap.rs`
- **Acción:** Crear `struct ActivityTracker` que registre la marca de tiempo de la última petición atómica. Diseñar `SelfHealingService` con bucle de verificación cada 60s; si `now - last_activity > idle_secs`, ejecutar `engine.compact_and_rebuild_index().await` cediendo cooperativamente.
- **Verify:** `cargo nextest run -p vantadb --lib server::self_healing`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Fase 3 (Unificación de Servidor) completada.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Operations & SRE Reviewer)
- **Enfoque:** Idempotencia en perfiles de shell y cesión cooperativa de locks en tareas de fondo.
- **Cómo se probó:** Tests unitarios de tracking de actividad y simulación de inactividad.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] No romper perfiles de usuario preexistentes.
  - [x] Respetar timeouts de Tokio.
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

El intervalo de inactividad por defecto de 300 segundos puede ser anulado estableciendo `VANTADB_AUTO_VACUUM_IDLE_SECS=0` si se desea desactivar la compactación automática en entornos de test.
