# PROV-11: Embed batching/async

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-code.md
- **Fuente:** plan file Task 9 (Wave2)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢 Baja
- **Tipo:** Rust PyO3 providers (openai/ollama/litellm) — feature-add aditiva
- **Turns estimados:** 6
- **Creado:** 2026-09-10
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 (diseño cerrado en Spec; AsyncClient nativo = DEFER documentado)
- **Pendientes (downhill):** 3 slices

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `integrations/openai|ollama` `vectorstore.py` (`_embed_many` — Python puro, no llama a estos crates); tests `providers/*/tests/test_*.py` (`test_embed_mocked`) |
| Callees | `providers/shared_py.rs` (`common::err_to_py`, `parse_distance_metric`); `vantadb::sdk::VantaEmbedded`; Python clients (`openai.OpenAI`, `ollama.Client`, `litellm.embedding`) |
| Implicaciones | Métodos nuevos aditivos (`embed_batch`); `embed()` sync intacto (sin regresión); sin nuevas deps (sin tokio/pyo3-asyncio); async = patrón `asyncio.to_thread` documentado+testeado, AsyncClient nativo DEFER |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** providers/shared_py.rs (272L), providers/openai/src/python.rs (335L), providers/ollama/src/python.rs (332L), providers/litellm/src/python.rs (335L), providers/openai|ollama|litellm/Cargo.toml (23L c/u), providers/openai|ollama|litellm/vantadb_*.pyi (56L c/u), providers/openai|ollama|litellm/tests/test_*.py
- **Archivos referenciados hacia dentro:** python.rs → `#[path="../../shared_py.rs"] mod common` + `vantadb::config::VantaConfig` + `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions}` + pyo3; shared_py.rs → pyo3 + `vantadb::error::VantaError` + `vantadb::sdk::{VantaMemoryRecord, VantaMemorySearchRequest, VantaValue}`
- **Archivos que referencian a los editados:** `integrations/openai|ollama/vantadb_*/vectorstore.py` (clases homónimas Python puras — sin import de estos crates); `providers/*/tests/test_*.py` (`test_embed_mocked` ×3); `.pyi` stubs (firma `embed`); ningún caller Rust externo (crates standalone con `[workspace]` propio)
- **Veredicto impacto:** bajo — 1 helper compartido + 1 método nuevo por crate (aditivo, `embed()` sin tocar salvo extraer chunk interno); 3 `.pyi` + 3 tests Python extendidos; sin cambios a WAL/vector/storage/engine; sin breaking (stop condition plan: breaking → DEFER, no dispara)

## Contrato

`cargo check` ×3 crates 0 warnings/errors + test batch/async ✅ + sin regresión sync (`embed()` firma y comportamiento intactos)

## Spec (SDD — feature-add: método público nuevo ×3)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive chunking | A: `shared_py.rs` (`common::batch_slices`, precedente PROV-01 dedup) / B: duplicado en cada python.rs (~15L ×3, drift futuro) | A | ✅ decidido-por-evidencia (shared_py.rs existe para eso) |
| 2 | Forma de batch | A: `embed_batch(texts, batch_size=100)` chunked sync aditivo / B: `batch_size` en `embed()` (rompe firma sync → viola pre-mortem) | A | ✅ plan pre-mortem (aditivo, no breaking) |
| 3 | Async | A: patrón `asyncio.to_thread(embed_batch)` documentado+testeado, sin deps / B: `pyo3-asyncio`+tokio nativo (deps nuevas, GIL+runtime, excede appetite 1d) | A | ✅ ponytail ladder (existe>stdlib>mínimo); AsyncClient nativo (ollama.AsyncClient/litellm.aembedding) = follow-up DEFER con servicio vivo |
| 4 | Validación batch_size | A: `batch_size<1` → ValueError (precedente PROV-07 distance_metric) / B: clamp silencioso (oculta bug caller) | A | ✅ api-and-interface-design (validate at boundaries) |
| 5 | Vacío | A: `[]` → `[]` sin llamar al cliente (precedente `_embed_many` integrations) / B: llamar igual (1 request inútil) | A | ✅ precedente integrations `if not texts: return []` |
| 6 | Medición volumen | A: test cuenta llamadas por chunk (250 textos/bs=100 → 3 calls, orden preservado) / B: bench live (requiere keys/servicio, fuera appetite) | A | ✅ plan pre-mortem (medir sin bench live) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `embed()` firma/cuerpo-observable intacto (1 call, mismo orden, mismos errores); `store/search/get/delete/list` sin tocar; `unwrap/expect` prohibido en prod (solo tests); sin nuevas dependencias; `.pyi` en sync con Rust; no stagear `Cargo.lock` providers (churn precedente 09-10) ni WIP ajeno (PRX-06/SRV-06, opencode.jsonc, .opencode, Investigacion-plan.md)
- **Comandos de verificación:** `cargo check` en providers/{openai,ollama,litellm} (0) + `cargo test` en los 3 (0 failed) + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check`
- **Deuda pendiente:** AsyncClient nativo (ollama.AsyncClient / litellm.aembedding / openai.AsyncOpenAI) con servicio vivo → follow-up; wiring `embed_batch` en `integrations/*/vectorstore.py::_embed_many` → follow-up

## Recitation (canónico — estructura única)

Ver pipeline-full.md §3 (se sincroniza vía campaign_update_task_state).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (código aditivo puro; `ponytail:` ceilings documentados donde aplique, no deuda).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | contrato ✅ + fmt/clippy/test ×3 + tests batch/async nuevos pasan + sync sin regresión |
| **Commit** | atómico solo-propios en develop, `feat: PROV-11 — ...`, hooks verdes |
| **Release** | N/A (no release; develop) |

## Herramientas necesarias

- cargo check/test/clippy/fmt (stack VantaDB, bash directa por bug campaign_verify_cmd exit -1)
- codegraph_explore (blast radius — hecho)

**Skills cargadas (SDP):** test-driven-development (lógica nueva RED→GREEN) · incremental-implementation (3 slices verticales) · context-engineering (context pack por slice) · source-driven-development (PyO3 0.29 signatures verificadas en código existente, sin fetch externo: patrón `#[pyo3(signature=...)]` precedente en archivos) · api-and-interface-design (método público nuevo aditivo + validate at boundaries) · base auto-MCP: campaign-executor, progreso. SDP v2 keywords: providers/openai/ollama/litellm/embed. `frontend-ui-engineering` sugerida por scoring pero descartada: sin UI web. `doubt-driven-development` conservada como lectura (stakes bajos, sin trust boundary nuevo — embed ya cruza a red vía clientes Python existentes).

## Investigation Notes

- `embed()` actual ×3: 1 sola llamada API con `input=texts` completo (openai: `client.embeddings.create`; ollama: `client.embed`; litellm: `litellm.embedding`). Sin chunking → volúmenes grandes = 1 request gigante o error de límite.
- Backlog PROV-11: "aprovechar AsyncClient ollama / aembedding litellm cuando haya volumenes grandes. Origen: INV-providers-01 H-12".
- Plan verificación real: crates standalone existen y compilan (confirmado 2026-09-10: check ×3 verde baseline esta sesión).
- Gate D: sin `question` — diseño implícito aprobado por owner (plan Task 9: "aditivo, no breaking" + contrato + stop condition) + blast radius bajo (7 archivos, sin hot path, sin callers Rust externos).
- Gate P: plan 19 DO ya aprobado por owner vía `question` 2026-09-10 (plan §Gate P).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 3 (slice1 helper compartido / slice2 openai+ollama / slice3 litellm+pyi+tests+verify) |
| % completado | 15% (discovery + task file) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — no aplica nuevo trust boundary: `embed_batch` reusa los mismos clientes Python y credenciales que `embed()` (red ya existente, sin inputs nuevos, sin auth nueva). No se carga security-and-hardening (sin boundary nuevo).
- [x] **PERFORMANCE** — aplica parcial: es la feature (throughput). Medición = test cuenta-chunks (250/100→3 calls, orden) + preservación 1-call para volúmenes ≤ batch_size (sin regresión de latencia en caso común). Bench live con servicio real = DEFER (requiere keys/red, fuera appetite). Sin hot path Rust tocado (GIL-bound Python I/O).

## Steps

### Step 1 (slice 1): helper compartido `batch_slices` + `validate_batch_size` + tests Rust
- **Archivos:** `providers/shared_py.rs`
- **Acción:** RED: tests `batch_slices_chunks_and_preserves_order` + `validate_batch_size_rejects_zero` (fallan: no existen). GREEN: `pub(super) fn validate_batch_size(batch_size: usize) -> Result<usize, String>` + `pub(super) fn batch_slices<T: Clone>(items: &[T], batch_size: usize) -> Vec<Vec<T>>` (mínimo, sin GIL, puro)
- **Verify:** `cargo test` en 1 crate (p. ej. openai) + clippy + fmt
- **Estado:** ✅ COMPLETED (RED E0425 confirmado → GREEN 6/6; clippy dead-code intermedio resuelto en slice 2 al consumir helpers)

### Step 2 (slice 2): `embed_batch` en openai + ollama (refactor chunk interno)
- **Archivos:** `providers/openai/src/python.rs`, `providers/ollama/src/python.rs`
- **Acción:** extraer `embed_chunk(&self, py, texts: &[String])` del cuerpo actual de `embed()` (sin cambio observable); añadir `#[pyo3(signature = (texts, batch_size = 100))] fn embed_batch` que valida (ValueError), `[]`→`[]`, loop chunks + concat; Rust include_str sanity test por crate (firma + ValueError path, precedente PROV-07/10)
- **Verify:** `cargo check` + `cargo test` ×2 crates + clippy + fmt
- **Estado:** ✅ COMPLETED (free fn `*_embed_chunk`: `&[String]` no es arg `#[pymethods]` válido E0277 → 1 bloque pymethods + free fn; PyO3 0.29 rechaza 2 bloques E0119)

### Step 3 (slice 3): `embed_batch` en litellm + `.pyi` ×3 + tests Python batch/async + verify full
- **Archivos:** `providers/litellm/src/python.rs`, `providers/{openai,ollama,litellm}/vantadb_*.pyi`, `providers/{openai,ollama,litellm}/tests/test_*.py`
- **Acción:** mismo patrón slice 2 en litellm; `.pyi`: `def embed_batch(self, texts: list[str], batch_size: int = 100) -> list[list[float]]: ...`; tests Python: batch chunking con cliente mockeado (N textos/bs → N calls, orden) + `batch_size=0` → ValueError + `[]` → `[]` + async `asyncio.to_thread(embed_batch)` preserva orden (×3 providers con su fake respectivo)
- **Verify:** contrato full: `cargo check` ×3 (0) + `cargo test` ×3 (0 failed) + clippy + fmt
- **Estado:** ✅ COMPLETED (Rust 7/7 ×3 + pytest 18+17+19 con .venv 3.11 maturin develop; sync intacto: todos los tests pre-existentes verdes)

## Dependencias
- Ninguna (standalone; Wave2 paralelo PRX-06/SRV-06 disjuntos — sin colisión)
- PROV-01 ✅ (shared_py existe), PROV-07/10 (precedentes test-patterns)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente (vanta-review post-commit o orquestador)
- **Enfoque:** pendiente
