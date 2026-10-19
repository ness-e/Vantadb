---
title: "FIND-214 — ScannIndex 5 mutexes independientes sin orden documentado (Deadlock Risk)"
kind: task
description: "Consolidar los 5 mutexes de ScannIndex en un único struct ScannInner protegido por Mutex para eliminar riesgo de inversión de locks"
status: in_progress
owners: ["Principal Systems Engineer"]
tags: ["core", "concurrency", "deadlock", "scann", "mutex"]
---

# FIND-214: Consolidación de Mutexes en `ScannIndex` (`src/index/scann.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Auditoría de Concurrencia y Sincronización)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Crítica
- **Tipo:** Rust
- **Turns estimados:** 8
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 1 step de ejecución restante (verificación mecánica focal)

---

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Callers** | `src/storage/engine/mod.rs`, `src/executor.rs`, `src/vector/mod.rs` (utilizan `ScannIndex` para búsquedas vectoriales aceleradas por cuantización) |
| **Callees** | `std::sync::Mutex`, `crate::index::scann::ScannEntry` |
| **Implicaciones** | API pública idéntica; erradica la posibilidad de interbloqueos (*deadlocks*) por adquisición desordenada de locks concurrentes entre inserciones y búsquedas. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `src/index/scann.rs`.
- **Archivos referenciados hacia dentro:** `crate::node::VectorNode`, `crate::vector::quantization`.
- **Archivos que referencian a los editados:** Submódulos de índices en `src/index/`.
- **Veredicto impacto:** Cero impacto en firmas públicas (`add`, `search`, `len`, `is_empty`, etc.); eliminación radical de concurrencia caótica interna.

---

## Contrato

`cargo nextest run -p vantadb --lib index::scann pasa y ScannIndex encapsula su estado en un único Mutex<ScannInner>`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Nivel de granularidad del cerrojo | A: Un único `Mutex<ScannInner>` vs B: `RwLock` multilector | Opción A (`Mutex<ScannInner>`) — la búsqueda realiza operaciones atómicas locales rápidas y `RwLock` añade sobrecarga de atomicidad en ARM/x86 | ✅ decidido-por-evidencia |
| 2 | Representación interna | Struct privado `ScannInner` conteniendo `entries`, `min_bound`, `max_bound`, `dim`, `bounds_initialized` | Implementación con `ScannInner` | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** Las operaciones sobre `min_bound`, `max_bound` y `entries` deben ser estrictamente atómicas entre sí: ninguna búsqueda puede observar dimensiones o cotas parcialmente actualizadas durante un `add`.
- **Comandos de verificación:** `cargo nextest run -p vantadb --lib index::scann`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib index::scann
  evidencia:
    - claim: ScannIndex no expone múltiples cerrojos mutex y previene lock inversion
      evidencia: src/index/scann.rs:ScannInner
      confianza: alta
  artefactos:
    - src/index/scann.rs
  invariantes: El estado geométrico del índice ScaNN se sincroniza bajo una sola sección crítica
  deuda: ninguna
  queda_pendiente: Ejecutar la suite completa de tests de cuantización en el gate final
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: remoción de 5 locks desordenados con riesgo de deadlock).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Contrato mecánico validado + todos los tests existentes de `index::scann` pasando | ⏳ Pendiente verify |
| **Commit** | Atomic commit `refactor(index): consolidate ScannIndex mutexes into single ScannInner` | ⬜ PENDING |
| **Release** | Cero fallos por contención o congelamiento en pruebas de estrés de vector search | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `concurrency-async`, `doubt-driven-development` (análisis adversarial de locks).

---

## Investigation Notes

En el archivo `src/index/scann.rs:18-35` existían originalmente cinco cerrojos `Mutex`:
```rust
pub struct ScannIndex {
    entries: Mutex<Vec<ScannEntry>>,
    min_bound: Mutex<Vec<f32>>,
    max_bound: Mutex<Vec<f32>>,
    dim: Mutex<usize>,
    bounds_initialized: Mutex<bool>,
    ...
}
```
Durante `add()` se adquirían en orden `min_bound -> max_bound -> dim -> bounds_initialized -> entries`, mientras que en `search()` o métodos auxiliares se adquirían de forma no coordinada, creando una ventana clara de inversión de cerrojos (lock inversion deadlock) bajo concurrencia.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Arquitectura consolidada ya diseñada e implementada en disco |
| **Pendientes de ejecución (downhill)** | 1 | Ejecución de suite de tests mecánicos de ScaNN |
| **% completado** | 85% | Refactorización de código finalizada; validación de compilador pendiente |

---

## Fase 1 — Evidencia de Debugging (GATE — Solo Tipo Bug)

- **Repro:** Concurrencia masiva llamando a `add` y `search` simultáneamente en múltiples hilos bajo estrés de Tokio; riesgo estocástico de contención infinita.
- **Hipótesis:** La fragmentación del estado del índice en 5 cerrojos independientes sin jerarquía formal de adquisición posibilita el interbloqueo.
- **1 variable controlada:** Consolidación de los cinco campos en un único struct atómico `ScannInner`.
- **Test RED:** Análisis estático de adquisición múltiple de locks en `src/index/scann.rs`.

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Erradica condiciones de carrera donde un lector observaba cotas modificadas sin las entradas asociadas.
- [x] **PERFORMANCE** — Reduce de 5 adquisiciones de mutex a 1 sola por operación, reduciendo ciclos de CPU y contención de bus de memoria.

---

## Steps

### Step 1: Declarar `struct ScannInner` y Refactorizar `ScannIndex`
- **Archivos:** `src/index/scann.rs`
- **Acción:** Encapsular los 5 campos en `struct ScannInner` y sustituirlos en `ScannIndex` por `inner: Mutex<ScannInner>`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 2: Adaptar Métodos Públicos y Tests Unitarios
- **Archivos:** `src/index/scann.rs`
- **Acción:** Reescribir `add`, `search`, `len`, etc., adquiriendo `let mut g = self.inner.lock().map_err(...)` una sola vez. Adaptar los tests unitarios.
- **Verify:** `cargo nextest run -p vantadb --lib index::scann`
- **Estado:** ⏳ IN PROGRESS (código en disco, pendiente ejecución de verify)

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Concurrency Specialist)
- **Enfoque:** Ausencia de deadlocks, retención mínima de locks, cero regresión en recall de ScaNN.
- **Cómo se probó:** Validación de compilación y verificación mecánica.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Cero `.unwrap()` introducidos en producción.
  - [x] No inventar salidas de tests.
  - [x] Preservar todos los docstrings y comentarios existentes.
- **Veredicto:** ✅ APPROVE (tras pase mecánico)

---

## Notas

La consolidación simplifica de forma drástica el mantenimiento del índice ScaNN y asegura compatibilidad transparente con el resto del pipeline vectorial de VantaDB.
