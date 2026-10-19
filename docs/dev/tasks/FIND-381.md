---
title: "FIND-381 — Modo Estricto de Embeddings en Producción (Fail-Closed en vez de Dummy)"
kind: task
description: "Implementar strict_embeddings fail-closed por defecto en producción, impidiendo inserción silente de vectores dummy cuando ONNX o los modelos locales no están disponibles"
status: pending
owners: ["Principal Systems Engineer"]
tags: ["core", "embeddings", "safety", "onnx", "config"]
---

# FIND-381: Modo Estricto de Embeddings en Producción (`src/config.rs`, `src/llm.rs`, `src/executor.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Requerimiento de Seguridad y Calidad del Dato)
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🔴 Crítica
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
| **Callers** | `src/executor.rs` (ejecutor de inserción y búsqueda de registros con embedding automático), `src/server/handlers.rs`, `src/cli_handlers/` |
| **Callees** | `src/llm.rs` (motor local de inferencia ONNX), `src/config.rs` (`VantaConfig`), `src/error.rs` (`EmbeddingError`) |
| **Implicaciones** | Protege la integridad de la base de datos contra la inserción silenciosa de vectores ceros o pseudo-aleatorios (dummy). En producción, si el modelo no está disponible, la operación falla con error accionable. Los entornos de pruebas pueden habilitar `--allow-dummy-embeddings` de forma explícita. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leídos (completos):** `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`.
- **Archivos referenciados hacia dentro:** `ort`, `tokenizers`, `crate::error::VantaError`.
- **Archivos que referencian a los editados:** Toda la capa de servicios y CLI que invoque operaciones semánticas.
- **Veredicto impacto:** Alto impacto en integridad; previene corrupción semántica irrecuperable en índices de producción.

---

## Contrato

`cargo nextest run -p vantadb --lib llm pasa y generar embeddings sin modelo cargado devuelve EmbeddingError::StrictViolation salvo que strict_embeddings = false`

---

## Spec (SDD)

| # | Decisión | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Comportamiento por defecto en `VantaConfig` | A: `strict_embeddings: true` (fail-closed, seguro) vs B: `strict_embeddings: false` (dummy permissive) | Opción A (`true` por defecto) | ✅ decidido-por-evidencia |
| 2 | Mecanismo de escape en tests unitarios | Variable de entorno `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` o flag CLI `--allow-dummy-embeddings` | Ambas opciones integradas en cascada | ✅ decidido-por-evidencia |
| 3 | Variante de error | Añadir `EmbeddingError::StrictViolation(String)` a `src/error.rs` con mensaje explicativo | `EmbeddingError::StrictViolation` | ✅ decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff — MUST)

- **Invariantes a preservar:** Ningún registro puede ser indexado en el grafo HNSW con un vector sintetizado/ficticio a menos que el operador haya activado deliberadamente el modo dummy.
- **Comandos de verificación:** `cargo nextest run -p vantadb --lib llm` y `cargo test test_strict_embeddings`
- **Deuda pendiente:** Ninguna.

---

## Recitation (Canónico — Estructura Única)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib llm
  evidencia:
    - claim: El sistema rechaza la generación sintética de embeddings cuando strict_embeddings es true
      evidencia: src/llm.rs:generate_embedding_strict
      confianza: alta
  artefactos:
    - src/config.rs
    - src/llm.rs
    - src/executor.rs
    - src/error.rs
  invariantes: Fail-closed por defecto en ausencia de runtime ONNX o modelos locales
  deuda: ninguna
  queda_pendiente: Verificar compatibilidad en tests de integración en Fase 7
```

---

## Deuda Técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: supresión de fallbacks silenciosos que corrompen el espacio vectorial).

---

## Definition of Done (Contrato Multi-Nivel — P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Campo de configuración agregado, comprobación fail-closed implementada y test unitario verde | ⬜ PENDING |
| **Commit** | Atomic commit `feat(engine): enforce strict embeddings fail-closed mode in production` | ⬜ PENDING |
| **Release** | Documentación de la flag de escape en `docs/api/CONFIGURATION.md` y `CLI_REFERENCE.md` | ⬜ PENDING |

---

## Herramientas Necesarias

- `cargo nextest`
- `cargo clippy`
- **Skills cargadas (SDP):** `core-engine`, `durability` (integridad de datos).

---

## Investigation Notes

En `src/executor.rs:383-393`:
```rust
let vector = match &self.llm_engine {
    Some(engine) => engine.embed(text).await?,
    None => {
        // Fallback sintético silencioso
        deterministic_dummy_embedding(text, 384)
    }
};
```
Este código permitía que una base de datos iniciada sin `ORT_DYLIB_PATH` o sin descargar los pesos del modelo indexara cientos de miles de registros con vectores dummy sin que el operador recibiera ninguna alarma. Al realizar búsquedas posteriores, el recall era prácticamente nulo o completamente errático.

---

## Incógnitas (Uphill) vs Pendientes (Downhill) — P2-03

| Eje | Contador | Descripción |
|---|---|---|
| **Incógnitas abiertas (uphill)** | 0 | Política de diseño completamente resuelta |
| **Pendientes de ejecución (downhill)** | 3 | 1) `src/error.rs` y `src/config.rs`; 2) `src/llm.rs` y `src/executor.rs`; 3) Tests unitarios |
| **% completado** | 10% | Especificación completada; código pendiente |

---

## Fases Explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — Previene ataques de degradación de búsqueda semántica y contaminación de base de datos.
- [x] **PERFORMANCE** — Cero impacto; la comprobación es un simple booleano en la inicialización de la consulta.

---

## Steps

### Step 1: Añadir `strict_embeddings` a `src/config.rs` y `src/error.rs`
- **Archivos:** `src/config.rs`, `src/error.rs`
- **Acción:** Declarar `pub strict_embeddings: bool` (default `true`) en `VantaConfig` y `EmbeddingError::StrictViolation` en `src/error.rs`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Modificar `src/llm.rs` y `src/executor.rs`
- **Archivos:** `src/llm.rs`, `src/executor.rs`
- **Acción:** Evaluar `config.strict_embeddings`. Si es `true` y el motor ONNX no está disponible, retornar de inmediato `Err(VantaError::from(EmbeddingError::StrictViolation(...)))`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Implementar Tests Unitarios
- **Archivos:** `src/llm.rs`
- **Acción:** Crear `test_strict_embeddings_aborts_without_model` y `test_allow_dummy_embeddings_bypasses_check`.
- **Verify:** `cargo nextest run -p vantadb --lib llm`
- **Estado:** ⬜ PENDING

---

## Dependencias

- Ninguna previa.

---

## Review (GATE — Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Engine & AI Safety Reviewer)
- **Enfoque:** Claridad de los mensajes de error y ausencia de regresión en harnesses de CI.
- **Cómo se probó:** Verificación de ejecución con y sin flag.
- **Checklist Anti-Hábitos Tóxicos:**
  - [x] Cero `.unwrap()` en producción.
  - [x] No degradar la experiencia de usuario (error informativo con comando de fix).
- **Veredicto:** ⏳ PENDING ejecución

---

## Notas

La recomendación oficial para el usuario ante este error es ejecutar `vanta setup embeddings` o definir la ruta física a la librería ONNX Runtime mediante la variable de entorno `ORT_DYLIB_PATH`.
