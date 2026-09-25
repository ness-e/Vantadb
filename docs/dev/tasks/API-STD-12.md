# Task API-STD-12 — INDIVIDUAL (11/11) vanta-memory

> **Plan:** `docs/dev/plans/2026-09-24-api-estandarizacion.md`
> **Estado:** ✅ DONE (inline, sin subagentes)
> **Fecha:** 2026-09-24

## 1. Objetivo + contrato

Ficha individual vanta-memory: funcionamiento + uso + código + veredicto.

## 2. Funcionamiento

Crate LLM-driven memoria agentes: L0 captura idempotente, L1 extracción/dedup 1-call JSON, L2 escenas, L3 persona, recall con scope, context engine, offload, wiki. **LLM opcional (P4)**: sin `llm-driver` degrada a store-all + dedup heurístico sin perder datos.

## 3. Uso (ejemplo mínimo)

```rust
// sin feature llm-driver: todo flujo degrada a equivalente sin-LLM
let report = pipeline.record_turn(session, text)?; // store-all si NotConfigured
```

## 4. Código (re-verificado 2026-09-24, grep directo)

- **V1 Degradación por diseño CONFIRMADA:** doc `:9-11` (`llm_runner.rs`, "callers degrade"), `#[cfg(not(llm-driver))] → Err(NotConfigured)` (`:106-111`), test `default features must degrade` (`:209-218`), lib `:17`.
- **V2 Core-only CONFIRMADO:** `conversation`/`skills` reservados, L0–L3 sin binding (`BINDINGS_NAMESPACES.md:25-27,250-259`, D42/D43).
- **V3 Deudas CONFIRMADAS:** keyword-overlap hasta embeddings D37 (`VANTA_MEMORY.md:69,101-102`), `TokenEstimator chars/3` D21 (`:102`), context↔worker MEM-16 (`:104`); F1-F3 en `EMBEDDED_SDK.md`.

## 5. Veredicto + implicaciones

3/3 (V1 = diseño sano). Propuestas 15: mantener P4 como estándar; exponer L0–L3 (scope grande → probable DEFER D42) o API Rust core-only estable; pagar D37/D21/MEM-16 con benchmark (Regla 9).

## 6. DoD

- [x] Contrato ✅ · Task file sync · Recitation: vanta-memory ficha completa (11/11 ✅)

## Context Save Point

API-STD-12 DONE — F1 completa (11/11). Next: API-STD-13 (conjunto+orquestador).
