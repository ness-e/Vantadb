---
title: "ADR-029: vanta-memory — LLM-driven context engine (D21-D23)"
type: adr
status: draft
tags: [vantadb, architecture, adr, vanta-memory]
created: 2026-08-21
last_reviewed: 2026-08-21
---

# ADR-029: vanta-memory — LLM-driven context engine (D21-D23)

> ⏳ **BORRADOR — pendiente articulación humana** (Regla 5 / D41). Guía de revisión
> con evidencia y preguntas socráticas por decisión (D21-D36):
> [`guia-revision-ADR-029-y-D24-D37.md`](./guia-revision-ADR-029-y-D24-D37.md)

> **Borrador técnico para revisión del autor — editar con tus palabras antes de aprobar.**
> Este documento fue redactado por la IA como evidencia técnica de la campaña F5
> (plan P29, `docs/plans/2026-08-21-vanta-context-engine.md`). Per Regla 5
> (forcing function), el Contexto/Decisión/Consecuencias finales los articula el
> autor humano; este borrador aporta los datos, alternativas evaluadas y deuda
> asumida para ese ejercicio.

## Context

El crate `vanta-memory` (F4-F5) porta el pipeline de memoria de TDAM a Rust:
L1 extracción/dedup, L2 scenes con heat, L3 persona, y en F5 el **context
engine** — compresión de historial de chat LLM-free, MMD (task memory activa),
recall híbrido cross-session, generation log de provenance, y GC.

Tres decisiones quedaron abiertas durante la implementación y se resolvieron en
código sin ADR (deuda arrastrada desde P27):

| ID | Decisión | Dónde vive |
|----|----------|------------|
| D21 | Token estimation por `chars / 3`, sin tiktoken | `context_engine/token_estimator.rs::TokenEstimator` |
| D22 | Recall scope híbrido `session \| agent \| team`, default `agent` | `core/hooks/auto_recall.rs::RecallScope` |
| D23 | Formato META del MMD `{created, updated, summary, heat}` | `context_engine/mmd.rs::SceneMeta` / `TaskMemory` |

Además, el crate adoptó un modelo **LLM-driven opcional**: el pipeline depende
del trait host-neutral `LlmRunner` y nunca de un runtime concreto; sin la
feature `llm-driver`, todo path dependiente de LLM degrada a su equivalente
LLM-free (compresión local, store-all, dedup heurístico) y nada bloquea.

## Decision

*(borrador técnico — el autor articula la decisión final)*

### Modelo LLM-driven con trait host-neutral sync

`LlmRunner` es un trait **síncrono** (`fn run(&self, params) -> Result<String,
LlmError>`) + conveniencia `complete_json<T>` que extrae JSON de output con
fences/prosa. Implementaciones incluidas:

- `StandaloneLlmRunner` — HTTP OpenAI-compatible directo (feature `llm-driver`)
- `OpenClawLlmRunner` — delega a un host via trait `OpenClawHost`
- `MockLlmRunner` — scripted, determinista (feature `mock`, tests)

**Alternativa descartada:** trait async nativo. Se eligió sync base porque el
pipeline corre embedded/sync hoy; existe `AsyncLlmRunner` opt-in bajo
`llm-driver` para que la capa server (MEM-16/35) adapte vía
`spawn_blocking`. El crate no incluye executor propio.

**Error contract:** `LlmError::NotConfigured` es la señal canónica de modo
LLM-free — los callers degradan (nunca bloquean); un fallo de LLM jamás pierde
datos (Principio 4).

### D21 — Estimador chars/3 sin tiktoken

`TokenEstimator { chars_per_token: 3 }`: tokens ≈ `chars / 3`, aplicado sobre
role-line + content (paridad TDAM `extractLlmVisibleText`).

**Alternativa descartada:** tiktoken/BPE real. Costos: dependencia nueva pesada,
vocabulario versionado por modelo, y acoplamiento del crate genérico a un
tokenizer específico. El estimador solo decide *cuándo comprimir* y *cuánto*;
un error sistemático de ±20% solo mueve el punto de disparo, no corrompe datos.
La compresión siempre deja margen (corta hasta quedar bajo budget, no exacto).

**Trade-off conocido:** 3 chars/token subestima tokens de CJK (~1.5 chars/token
real → CJK ocupa ~2× lo estimado). Documentado como techo; upgrade path:
`chars_per_token` configurable ya expuesto, o estimador por-script más adelante.

### D22 — recall_scope híbrido session | agent | team, default agent

`RecallScope` controla el alcance cross-session del pool L1:

| Scope | Visible |
|-------|---------|
| `Session` | solo la sesión actual (comportamiento pre-MEM-40) |
| `Agent` *(default)* | sesión actual + otras sesiones del mismo `agent_id` |
| `Team` | sesión actual + todas las sesiones del mismo `team_id` |

Default `agent` replica el comportamiento de-facto de TDAM (memoria acumula
across sessions de un agente) sin su leak cross-agent. Reglas de visibilidad:
los records de la propia sesión siempre visibles; records legacy sin metadata
`agent_id`/`team_id` permanecen session-only (no desaparecen al ampliar scope).

**Implementación:** full scan de namespaces `l1/*` excluyendo la propia —
O(#sesiones + #records) por recall, aceptable a cientos de sesiones. Upgrade
path documentado: índice sessions-per-agent.

### D23 — MMD formato META

La task memory activa (`TaskMemory`) lleva contrato META `{created, updated,
summary, heat}` + body ≤ 4000 chars (`MAX_MMD_CONTENT_CHARS`, guard ~1300
tokens). Persistida en namespace `mmd/<session>/active`; historial en
`mmd/<session>/history` con keys FNV-1a(content+updated) idempotentes.

Semántica heat (convención compartida con `SceneMeta` y `SceneNode` core):
CREATE = 1, UPDATE = old + 1. El store MMD es CRUD tonto — nunca muta META;
la semántica (preservar `created`, bump `updated`) vive en la estrategia.

**Trade-off:** `summary` es placeholder descriptivo hasta L1 (el offload real
de resúmenes semánticos llega con L1); dedup por fingerprint `{len}:{primeros
64 chars}` — colisiones teóricas aceptadas para contenido idéntico-prefix.

### Trade-offs transversales asumidos (F5)

1. **Compresión heurística vs score LLM:** `score_message` usa replaceability
   por rol (ToolResult=6 > ToolCall=5 > Assistant=4 > User=2) + bonus edad,
   sustituyendo al score L1 del LLM. Determinista y LLM-free; upgrade post-L1:
   consumir scores reales.
2. **Stub `[compacted N chars]`** sin semántica recuperable — refs re-leen a
   demanda (offload). Aceptado; el summary semántico es trabajo de L1.
3. **Aggressive con prefijo protegido:** si el prefijo protegido (cursor)
   solo ya excede el budget, `assemble` devuelve over-budget antes que violar
   la garantía del cursor — degrada a emergency operando solo sobre la región
   compactable.
4. **Recall keyword-overlap sin vectores:** `RecallMode::{Embedding, Hybrid}`
   existen en la API pero degradan a `Keyword` hasta cablear el índice vector
   de VantaDB en este crate. Techo explícito (`RecallMode::effective()`).
5. **Pair-guard:** unidades atómicas (`build_units`) garantizan que un par
   tool_call/tool_result jamás se parte en ninguna pasada.

## Consequences

*(borrador técnico — el autor evalúa costos/riesgos)*

### Pros
- Cero dependencias de tokenizer/LLM runtime en el build default; el crate
  compila y funciona 100% LLM-free.
- Contrato de errores tipado y `#[non_exhaustive]`; degradación por capa en
  lugar de fallo en cascada.
- Superficie pública mínima y testeable con mocks deterministas (suite 430/430).

### Cons / deuda asumida (documentada, con upgrade path)
| Deuda | Techo | Upgrade path |
|-------|-------|--------------|
| chars/3 subestima CJK | budgets ~2× off para texto CJK | estimator per-script / tiktoken opt-in |
| keyword-overlap sin embeddings | recall léxico puro | cablear vector index de VantaDB |
| scan O(#sessions) en scoped recall | cientos de sesiones | índice sessions-per-agent |
| aggressive→emergency con prefijo protegido puede devolver over-budget | garantía cursor > budget estricto | caller decide (drop cursor / raise budget) |
| MMD `summary` placeholder | sin resumen semántico hasta L1 | offload real con L1 |
| stub `[compacted N chars]` no recuperable in-place | refs re-leen a demanda | summary semántico post-L1 |

### Nota mecánica
`scripts/validate-docs-coverage.ps1` hoy NO escanea `vanta-memory` (solo
src/sdk, config, error, cli, python bindings, MCP tools): la cobertura de docs
de este crate no está enforceada por CI. La referencia canónica de las
superficies públicas F5 vive en `docs/api/VANTA_MEMORY.md`.
