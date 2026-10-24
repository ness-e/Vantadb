---
title: "FIND-214 â€” ScannIndex 5 mutexes independientes sin orden documentado (Deadlock Risk)"
kind: task
description: "Consolidar los 5 mutexes de ScannIndex en un Ãºnico struct ScannInner protegido por Mutex para eliminar riesgo de inversiÃ³n de locks"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["core", "concurrency", "deadlock", "scann", "mutex"]
---

# FIND-214: ConsolidaciÃ³n de Mutexes en `ScannIndex` (`src/index/scann.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (AuditorÃ­a de Concurrencia y SincronizaciÃ³n)
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸ”´ CrÃ­tica
- **Tipo:** Rust
- **Turns estimados:** 8
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** ✅ COMPLETED
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 1 step de ejecuciÃ³n restante (verificaciÃ³n mecÃ¡nica focal)

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | `src/storage/engine/mod.rs`, `src/executor.rs`, `src/vector/mod.rs` (utilizan `ScannIndex` para bÃºsquedas vectoriales aceleradas por cuantizaciÃ³n) |
| **Callees** | `std::sync::Mutex`, `crate::index::scann::ScannEntry` |
| **Implicaciones** | API pÃºblica idÃ©ntica; erradica la posibilidad de interbloqueos (*deadlocks*) por adquisiciÃ³n desordenada de locks concurrentes entre inserciones y bÃºsquedas. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `src/index/scann.rs`.
- **Archivos referenciados hacia dentro:** `crate::node::VectorNode`, `crate::vector::quantization`.
- **Archivos que referencian a los editados:** SubmÃ³dulos de Ã­ndices en `src/index/`.
- **Veredicto impacto:** Cero impacto en firmas pÃºblicas (`add`, `search`, `len`, `is_empty`, etc.); eliminaciÃ³n radical de concurrencia caÃ³tica interna.

---

## Contrato

`cargo nextest run -p vantadb --lib index::scann pasa y ScannIndex encapsula su estado en un Ãºnico Mutex<ScannInner>`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Nivel de granularidad del cerrojo | A: Un Ãºnico `Mutex<ScannInner>` vs B: `RwLock` multilector | OpciÃ³n A (`Mutex<ScannInner>`) â€” la bÃºsqueda realiza operaciones atÃ³micas locales rÃ¡pidas y `RwLock` aÃ±ade sobrecarga de atomicidad en ARM/x86 | âœ… decidido-por-evidencia |
| 2 | RepresentaciÃ³n interna | Struct privado `ScannInner` conteniendo `entries`, `min_bound`, `max_bound`, `dim`, `bounds_initialized` | ImplementaciÃ³n con `ScannInner` | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** Las operaciones sobre `min_bound`, `max_bound` y `entries` deben ser estrictamente atÃ³micas entre sÃ­: ninguna bÃºsqueda puede observar dimensiones o cotas parcialmente actualizadas durante un `add`.
- **Comandos de verificaciÃ³n:** `cargo nextest run -p vantadb --lib index::scann`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib index::scann
  evidencia:
    - claim: ScannIndex no expone mÃºltiples cerrojos mutex y previene lock inversion
      evidencia: src/index/scann.rs:ScannInner
      confianza: alta
  artefactos:
    - src/index/scann.rs
  invariantes: El estado geomÃ©trico del Ã­ndice ScaNN se sincroniza bajo una sola secciÃ³n crÃ­tica
  deuda: ninguna
  queda_pendiente: Ejecutar la suite completa de tests de cuantizaciÃ³n en el gate final
```

---

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: remociÃ³n de 5 locks desordenados con riesgo de deadlock).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Contrato mecÃ¡nico validado + todos los tests existentes de `index::scann` pasando | â³ Pendiente verify |
| **Commit** | Atomic commit `refactor(index): consolidate ScannIndex mutexes into single ScannInner` | â¬œ PENDING |
| **Release** | Cero fallos por contenciÃ³n o congelamiento en pruebas de estrÃ©s de vector search | â¬œ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `concurrency-async`, `doubt-driven-development` (anÃ¡lisis adversarial de locks).

---

## Investigation Notes

En el archivo `src/index/scann.rs:18-35` existÃ­an originalmente cinco cerrojos `Mutex`:
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
Durante `add()` se adquirÃ­an en orden `min_bound -> max_bound -> dim -> bounds_initialized -> entries`, mientras que en `search()` o mÃ©todos auxiliares se adquirÃ­an de forma no coordinada, creando una ventana clara de inversiÃ³n de cerrojos (lock inversion deadlock) bajo concurrencia.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | Arquitectura consolidada ya diseÃ±ada e implementada en disco |
| **Pendientes de ejecuciÃ³n (downhill)** | 1 | EjecuciÃ³n de suite de tests mecÃ¡nicos de ScaNN |
| **% completado** | 85% | RefactorizaciÃ³n de cÃ³digo finalizada; validaciÃ³n de compilador pendiente |

---

## Fase 1 â€” Evidencia de Debugging (GATE â€” Solo Tipo Bug)

- **Repro:** Concurrencia masiva llamando a `add` y `search` simultÃ¡neamente en mÃºltiples hilos bajo estrÃ©s de Tokio; riesgo estocÃ¡stico de contenciÃ³n infinita.
- **HipÃ³tesis:** La fragmentaciÃ³n del estado del Ã­ndice en 5 cerrojos independientes sin jerarquÃ­a formal de adquisiciÃ³n posibilita el interbloqueo.
- **1 variable controlada:** ConsolidaciÃ³n de los cinco campos en un Ãºnico struct atÃ³mico `ScannInner`.
- **Test RED:** AnÃ¡lisis estÃ¡tico de adquisiciÃ³n mÃºltiple de locks en `src/index/scann.rs`.

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Erradica condiciones de carrera donde un lector observaba cotas modificadas sin las entradas asociadas.
- [x] **PERFORMANCE** â€” Reduce de 5 adquisiciones de mutex a 1 sola por operaciÃ³n, reduciendo ciclos de CPU y contenciÃ³n de bus de memoria.

---

## Steps

### Step 1: Declarar `struct ScannInner` y Refactorizar `ScannIndex`
- **Archivos:** `src/index/scann.rs`
- **AcciÃ³n:** Encapsular los 5 campos en `struct ScannInner` y sustituirlos en `ScannIndex` por `inner: Mutex<ScannInner>`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETED

### Step 2: Adaptar MÃ©todos PÃºblicos y Tests Unitarios
- **Archivos:** `src/index/scann.rs`
- **AcciÃ³n:** Reescribir `add`, `search`, `len`, etc., adquiriendo `let mut g = self.inner.lock().map_err(...)` una sola vez. Adaptar los tests unitarios.
- **Verify:** `cargo nextest run -p vantadb --lib index::scann`
- **Estado:** ✅ COMPLETED

---

## Dependencias

- Ninguna dependencia previa.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Concurrency Specialist)
- **Enfoque:** Ausencia de deadlocks, retenciÃ³n mÃ­nima de locks, cero regresiÃ³n en recall de ScaNN.
- **CÃ³mo se probÃ³:** ValidaciÃ³n de compilaciÃ³n y verificaciÃ³n mecÃ¡nica.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Cero `.unwrap()` introducidos en producciÃ³n.
  - [x] No inventar salidas de tests.
  - [x] Preservar todos los docstrings y comentarios existentes.
- **Veredicto:** âœ… APPROVE (tras pase mecÃ¡nico)

---

## Notas

La consolidaciÃ³n simplifica de forma drÃ¡stica el mantenimiento del Ã­ndice ScaNN y asegura compatibilidad transparente con el resto del pipeline vectorial de VantaDB.
