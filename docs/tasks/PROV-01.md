# PROV-01: Fix compile openai — añadir exclude_superseded: false

## Metadata
- **Plan file:** docs/plans/2026-08-25-research-providers-quickwins.md
- **Fuente:** docs/Backlog.md P45 PROV-01 (INV-providers-01 H-01) — list() construye VantaMemoryListOptions sin campo exclude_superseded (E0063)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 3
- **Creado:** 2026-08-26T23:20
- **last-synced:** 2026-08-26T23:20
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `providers/openai/src/python.rs` (pyclass VantaDBOpenAI::list, ::search), `providers/litellm/src/python.rs`, `providers/ollama/src/python.rs` (patrón idéntico — verificar consistencia), `src/sdk/types.rs` (definición VantaMemoryListOptions / VantaMemorySearchRequest) |
| Callees | `src/sdk/types.rs:214-232` (VantaMemoryListOptions), `src/sdk/types.rs` VantaMemorySearchRequest, `vantadb::sdk::VantaEmbedded::list/search` |
| Implicaciones | contrato no rompe — campo ya existente con Default=false, fix es añadir campo faltante. No cambia comportamiento público (superseded sigue visible). No afecta performance/memoria. No requiere migración. Tests existentes providers deben re-verificarse. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `providers/openai/src/python.rs` (349L), `providers/openai/Cargo.toml` (23L), `src/sdk/types.rs:214-232` (VantaMemoryListOptions)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `providers/openai/src/python.rs` → `vantadb::config::VantaConfig`, `vantadb::error::VantaError`, `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord, VantaMemorySearchRequest, VantaValue}`, `pyo3`, `std::collections::HashMap`, `std::time::{SystemTime, UNIX_EPOCH}`
- **Archivos que referencian a los editados (referencias entrantes):** grep `vantadb-openai` → `providers/openai/Cargo.toml`, `providers/openai/tests/test_openai.py`, `providers/litellm`/`providers/ollama` (patrón hermano); grep `VantaMemoryListOptions` → 48 callers en `src/sdk/mod.rs`, `src/cli_handlers/`, `src/storage/`, tests
- **Veredicto impacto:** bajo — cambio mecánico de 1 campo en 2 structs literals (search + list). Sin callers externos afectados. Riesgo solo si se omite campo en otro provider hermano (litellm/ollama ya verificados — ambos ya tienen exclude_superseded).

## Contrato
`cargo check --manifest-path providers/openai/Cargo.toml` exit 0

## Spec (SDD — obligatoria si Phase 1b detectó feature-add/símbolos públicos)

No aplica — fix mecánico sin símbolos públicos nuevos. No agrega `pub fn`/endpoint/binding nuevo.

| # | Decisión | Opciones | Default recomendado | Resuelto |
|---|----------|----------|---------------------|----------|
| 1 | Valor de exclude_superseded | false (visible) / true (ocultar) | false (preserva comportamiento actual, Default) | ✅ decidido-por-evidencia (src/sdk/types.rs:231 default false, commit 2754c783 ya usa false) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** VantaMemoryListOptions y VantaMemorySearchRequest deben construirse con todos los campos obligatorios; superseded records siguen visibles por default (exclude_superseded=false); no cambiar semántica de search/list.
- **Comandos de verificación:** `cargo check --manifest-path providers/openai/Cargo.toml` (contrato), `cargo fmt --check` (determinismo)
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Fix compile openai — añadir exclude_superseded: false |
| `lastAction` | Discovery completado, task file creado, verify cargo check pendiente |
| `result` | PARTIAL (work in progress) |
| `nextAction` | Step 1 verify: cargo check --manifest-path providers/openai/Cargo.toml |
| `contract` | Contrato + Invariantes + evidencia |
| `nextTask` | PROV-06 (según plan Wave 1) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — fix reemplaza construcción incompleta por construcción completa (0 líneas netas nuevas vs deuda). No introduce deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 + campo exclude_superseded presente en list() y search() |
| **Commit** | Commit atómico feat: PROV-01, conventional commit (si aplica), git diff limpio — NO commitea lead |
| **Release** | No aplica (fix no release-able solo) |

## Herramientas necesarias
- cargo check (verify contrato)
- cargo fmt --check
- codegraph_explore (blast radius)

**Skills cargadas (SDP):**
- `source-driven-development` — verificar docs oficiales si API referencia ambigua (detectado por campaign_load_skills)
- `ponytail` (full) — ladder: existe > stdlib > dep > mínimo código; fix de 1 línea, sin abstracciones
- `systematic-debugging` — bug fix E0063 requiere Iron Law (root cause antes de fix) — Lifecycle VERIFY
- `incremental-implementation` — slice vertical delgado (≤100 líneas, verify tras cada edición) — Lifecycle BUILD
- `doubt-driven-development` — stakes producción (compile break) requiere verificación adversarial en contexto fresco — Lifecycle BUILD
- `campaign-executor` — núcleo task system PLAN/ACT/VERIFY
- `progreso` — migración Backlog → docs/avance al cierre
- `code-review-and-quality` — gate pre-commit 5 ejes (no aplica edición nueva, pero verifica diff existente)
- SDP: keywords grep `cargo|compile|rust|provider` → sin candidatos adicionales más allá de los 8 listados

## Investigation Notes
- codegraph_explore `VantaMemoryListOptions exclude_superseded` → 48 callers, fix ya aplicado en commit 2754c783 (search: exclude_superseded: false + search_profile: None en línea 201-202; list: exclude_superseded: false en línea 322). Build verifica: cargo check pasa.
- Web research no requerida — API es interna (src/sdk/types.rs:231 default false), no hay ambigüedad externa.
- Line numbers del contrato original (296-302) apuntan a `get()` en versión actual; E0063 real estaba en `list()` (322) y `search()` (201). Ambos ya corregidos.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado (añadir campo = false), evidencia de commit previo |
| Pendientes de ejecución (downhill) | 1 — Step 1 verify |
| % completado | 80% (discovery + task file done, falta verify mecánico) |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** `cargo check --manifest-path providers/openai/Cargo.toml` → E0063 missing field `exclude_superseded` in initializer of `VantaMemoryListOptions` (y `VantaMemorySearchRequest`) contra core actual (src/sdk/types.rs:214-232 añade campo sin Default en literal).
- **Hipótesis:** VantaMemoryListOptions añadió `exclude_superseded: bool` (ADR-028) sin actualizar providers/openai literal; Rust exige campo en struct literal salvo `..Default::default()`.
- **1 variable controlada:** añadir `exclude_superseded: false` en `VantaMemoryListOptions` literal (y `VantaMemorySearchRequest`) — sin tocar otro campo.
- **Test RED:** cargo check falla antes del fix (evidencia histórica INV-providers-01 score 4.0/10, Backlog PROV-01); GREEN tras fix — verificado en commit 2754c783 y re-verificado ahora.

**Gate:** sección poblada antes de fix — fix ya aplicado en 2754c783, hoy solo reverificación.

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — No toca trust boundaries (input usuario, auth, FFI más allá de thin wrapper existente). No agrega dependencias. No requiere security-and-hardening. Justificación: campo bool interno, sin input externo.
- [x] **PERFORMANCE** — No toca hot path (vector/HNSW/engine). No requiere performance-optimization ni baseline. Justificación: literal struct, 0 impacto.

## Steps

### Step 1: Verify compile openai (contrato) ✅
- **Archivos:** `providers/openai/src/python.rs:188-203` (search), `providers/openai/src/python.rs:316-323` (list), `providers/openai/Cargo.toml`
- **Acción:** Verificar que `exclude_superseded: false` está presente en ambos literales y que `cargo check --manifest-path providers/openai/Cargo.toml` exit 0. Si falta, añadirlo (1 línea). Si ya está, solo verificar y marcar ✅.
- **Verify:** `cargo check --manifest-path providers/openai/Cargo.toml` → exit 0 (1.02s) ✅; `Select-String exclude_superseded: false` → 2 matches (L201 search, L322 list) ✅; `cargo fmt --check` exit 0 ✅; `cargo clippy --manifest-path providers/openai/Cargo.toml -- -D warnings` exit 0 ✅
- **Estado:** ✅ COMPLETED (2026-08-26T23:20 — sin edición, ya estaba en 2754c783)

## Dependencias
- Ninguna (Wave 1 Task 1 — sin bloqueos)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (contexto fresco degradado — no hay subagente disponible en esta invocación; alternativa vanta-review no invocable por límite de sesión)
- **Enfoque:** ¿approach correcto? ¿alternativa `..Default::default()` mejor? — Veredicto: explicit `exclude_superseded: false` es correcto. `..Default::default()` oculta intención y rompería si Default cambia; explicit es ponytail-rung 6 (1 línea) y deja grep auditable. Alternativa descartada.
- **Cómo se probó:** `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 + grep 2 matches + clippy -D warnings exit 0 — evidencia mecánica, no auto-reporte.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos no ejecutados
  - [x] No saltarse clarificación por "ya sé qué quiere"
  - [x] No declarar done sin verificar contrato
  - [x] No ignorar fallos
  - [x] No hacer un solo intento de búsqueda y darlo por saturado
  - [x] No copiar sin citar
  - [x] No reintentar en bucle sin diagnóstico
  - [x] No dejar huérfanos los pasos
  - [x] No degradar chequeo de errores
  - [x] No gastar presupuesto infinito
- **Veredicto:** ✅ approve (degraded — single-context, cross-model skip anunciado: non-interactive context)

## Notas
- Fix ya merged en 2754c783 (2026-08-26) con Wave 1 completo (PROV-01/03/06/07/08). Este task file documenta re-verificación y cierre formal por pipeline. No se requiere edición si verify pasa.
- Ponytail: skipped abstracción `..Default::default()` que oculta qué campos se setean; explicit `exclude_superseded: false` es más corto y más claro. Add `..Default` cuando struct crezca >8 campos.
- Verify full ejecutado (sin commit por instrucción `NO commitees — lead commitea`): cargo check 1.02s exit 0, grep 2 matches, fmt exit 0, clippy exit 0 (5.82s). nextest/workspace no requerido por contrato (provider fuera de workspace) — archivado como WAVE-1 batch.

## Context Save Point
- **Último step:** Step 1 ✅ COMPLETED — contrato `cargo check --manifest-path providers/openai/Cargo.toml` exit 0 verificado
- **Próximo:** ninguno — task COMPLETED, handoff a PROV-06
- **Archivos tocados:** `.opencode/skills/campaign-executor/tasks/PROV-01.md` (nuevo task file); `providers/openai/src/python.rs` ya corregido en 2754c783 (no tocado en esta invocación)
- **Verify pendiente:** ninguno
