---
title: "FIND-381 â€” Modo Estricto de Embeddings en ProducciÃ³n (Fail-Closed en vez de Dummy)"
kind: task
description: "Implementar strict_embeddings fail-closed por defecto en producciÃ³n, impidiendo inserciÃ³n silente de vectores dummy cuando ONNX o los modelos locales no estÃ¡n disponibles"
status: completed
owners: ["Principal Systems Engineer"]
tags: ["core", "embeddings", "safety", "onnx", "config"]
---

# FIND-381: Modo Estricto de Embeddings en ProducciÃ³n (`src/config.rs`, `src/llm.rs`, `src/executor.rs`)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-19-pre-merge-systems-and-ux-remediation.md`
- **Fuente:** `docs/dev/Backlog.md` (Requerimiento de Seguridad y Calidad del Dato)
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸ”´ CrÃ­tica
- **Tipo:** Rust
- **Turns estimados:** 10
- **Creado:** 2026-10-19T02:00
- **last-synced:** 2026-10-19T03:00
- **Estado:** â¬œ PENDING
- **IncÃ³gnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 3 steps de ejecuciÃ³n restantes

---

## Blast Radius

| DirecciÃ³n | MÃ³dulos |
|---|---|
| **Callers** | `src/executor.rs` (ejecutor de inserciÃ³n y bÃºsqueda de registros con embedding automÃ¡tico), `src/server/handlers.rs`, `src/cli_handlers/` |
| **Callees** | `src/llm.rs` (motor local de inferencia ONNX), `src/config.rs` (`VantaConfig`), `src/error.rs` (`EmbeddingError`) |
| **Implicaciones** | Protege la integridad de la base de datos contra la inserciÃ³n silenciosa de vectores ceros o pseudo-aleatorios (dummy). En producciÃ³n, si el modelo no estÃ¡ disponible, la operaciÃ³n falla con error accionable. Los entornos de pruebas pueden habilitar `--allow-dummy-embeddings` de forma explÃ­cita. |

---

## Impacto Mapeado (Regla 0)

- **Archivos leÃ­dos (completos):** `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`.
- **Archivos referenciados hacia dentro:** `ort`, `tokenizers`, `crate::error::VantaError`.
- **Archivos que referencian a los editados:** Toda la capa de servicios y CLI que invoque operaciones semÃ¡nticas.
- **Veredicto impacto:** Alto impacto en integridad; previene corrupciÃ³n semÃ¡ntica irrecuperable en Ã­ndices de producciÃ³n.

---

## Contrato

`cargo nextest run -p vantadb --lib llm pasa y generar embeddings sin modelo cargado devuelve EmbeddingError::StrictViolation salvo que strict_embeddings = false`

---

## Spec (SDD)

| # | DecisiÃ³n | Opciones (+tradeoff) | Default Recomendado | Resuelto |
|---|---|---|---|---|
| 1 | Comportamiento por defecto en `VantaConfig` | A: `strict_embeddings: true` (fail-closed, seguro) vs B: `strict_embeddings: false` (dummy permissive) | OpciÃ³n A (`true` por defecto) | âœ… decidido-por-evidencia |
| 2 | Mecanismo de escape en tests unitarios | Variable de entorno `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` o flag CLI `--allow-dummy-embeddings` | Ambas opciones integradas en cascada | âœ… decidido-por-evidencia |
| 3 | Variante de error | AÃ±adir `EmbeddingError::StrictViolation(String)` a `src/error.rs` con mensaje explicativo | `EmbeddingError::StrictViolation` | âœ… decidido-por-evidencia |

---

## Invariantes de Dominio (Handoff â€” MUST)

- **Invariantes a preservar:** NingÃºn registro puede ser indexado en el grafo HNSW con un vector sintetizado/ficticio a menos que el operador haya activado deliberadamente el modo dummy.
- **Comandos de verificaciÃ³n:** `cargo nextest run -p vantadb --lib llm` y `cargo test test_strict_embeddings`
- **Deuda pendiente:** Ninguna.

---

## Recitation (CanÃ³nico â€” Estructura Ãšnica)

```yaml
contract:
  verificacion: cargo nextest run -p vantadb --lib llm
  evidencia:
    - claim: El sistema rechaza la generaciÃ³n sintÃ©tica de embeddings cuando strict_embeddings es true
      evidencia: src/llm.rs:generate_embedding_strict
      confianza: alta
  artefactos:
    - src/config.rs
    - src/llm.rs
    - src/executor.rs
    - src/error.rs
  invariantes: Fail-closed por defecto en ausencia de runtime ONNX o modelos locales
  deuda: ninguna
  queda_pendiente: Verificar compatibilidad en tests de integraciÃ³n en Fase 7
```

---

## Deuda TÃ©cnica (Regla 6 â€” MUST)

**Saldo neto de deuda por PR:** Sin deuda (deuda eliminada: supresiÃ³n de fallbacks silenciosos que corrompen el espacio vectorial).

---

## Definition of Done (Contrato Multi-Nivel â€” P2-08)

| Nivel | Gate | Estado |
|---|---|---|
| **Task** | Campo de configuraciÃ³n agregado, comprobaciÃ³n fail-closed implementada y test unitario verde | â¬œ PENDING |
| **Commit** | Atomic commit `feat(engine): enforce strict embeddings fail-closed mode in production` | â¬œ PENDING |
| **Release** | DocumentaciÃ³n de la flag de escape en `docs/api/CONFIGURATION.md` y `CLI_REFERENCE.md` | â¬œ PENDING |

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
        // Fallback sintÃ©tico silencioso
        deterministic_dummy_embedding(text, 384)
    }
};
```
Este cÃ³digo permitÃ­a que una base de datos iniciada sin `ORT_DYLIB_PATH` o sin descargar los pesos del modelo indexara cientos de miles de registros con vectores dummy sin que el operador recibiera ninguna alarma. Al realizar bÃºsquedas posteriores, el recall era prÃ¡cticamente nulo o completamente errÃ¡tico.

---

## IncÃ³gnitas (Uphill) vs Pendientes (Downhill) â€” P2-03

| Eje | Contador | DescripciÃ³n |
|---|---|---|
| **IncÃ³gnitas abiertas (uphill)** | 0 | PolÃ­tica de diseÃ±o completamente resuelta |
| **Pendientes de ejecuciÃ³n (downhill)** | 3 | 1) `src/error.rs` y `src/config.rs`; 2) `src/llm.rs` y `src/executor.rs`; 3) Tests unitarios |
| **% completado** | 10% | EspecificaciÃ³n completada; cÃ³digo pendiente |

---

## Fases ExplÃ­citas â€” SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** â€” Previene ataques de degradaciÃ³n de bÃºsqueda semÃ¡ntica y contaminaciÃ³n de base de datos.
- [x] **PERFORMANCE** â€” Cero impacto; la comprobaciÃ³n es un simple booleano en la inicializaciÃ³n de la consulta.

---

## Steps

### Step 1: AÃ±adir `strict_embeddings` a `src/config.rs` y `src/error.rs`
- **Archivos:** `src/config.rs`, `src/error.rs`
- **AcciÃ³n:** Declarar `pub strict_embeddings: bool` (default `true`) en `VantaConfig` y `EmbeddingError::StrictViolation` en `src/error.rs`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 2: Modificar `src/llm.rs` y `src/executor.rs`
- **Archivos:** `src/llm.rs`, `src/executor.rs`
- **AcciÃ³n:** Evaluar `config.strict_embeddings`. Si es `true` y el motor ONNX no estÃ¡ disponible, retornar de inmediato `Err(VantaError::from(EmbeddingError::StrictViolation(...)))`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 3: Implementar Tests Unitarios
- **Archivos:** `src/llm.rs`
- **AcciÃ³n:** Crear `test_strict_embeddings_aborts_without_model` y `test_allow_dummy_embeddings_bypasses_check`.
- **Verify:** `cargo nextest run -p vantadb --lib llm`
- **Estado:** â¬œ PENDING

---

## Dependencias

- Ninguna previa.

---

## Review (GATE â€” Agente Distinto, P2-01)

- **Revisor:** `vanta-audit` (Engine & AI Safety Reviewer)
- **Enfoque:** Claridad de los mensajes de error y ausencia de regresiÃ³n en harnesses de CI.
- **CÃ³mo se probÃ³:** VerificaciÃ³n de ejecuciÃ³n con y sin flag.
- **Checklist Anti-HÃ¡bitos TÃ³xicos:**
  - [x] Cero `.unwrap()` en producciÃ³n.
  - [x] No degradar la experiencia de usuario (error informativo con comando de fix).
- **Veredicto:** â³ PENDING ejecuciÃ³n

---

## Notas

La recomendaciÃ³n oficial para el usuario ante este error es ejecutar `vanta setup embeddings` o definir la ruta fÃ­sica a la librerÃ­a ONNX Runtime mediante la variable de entorno `ORT_DYLIB_PATH`.
