---
title: "API-08: W7 vanta-memory — API Rust estable (NO exponer, Gate P core-only)"
kind: task
description: "cargo test -p vanta-memory verde Y degradado sin llm-driver verificado (test :209-218 pasa) Y D37/D21/MEM-16 con benchmark o DEFER fundado Y 0 símbolos nuevos en bindings\""
---

# API-08: W7 vanta-memory — API Rust estable (NO exponer, Gate P core-only)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-24-api-ejecucion.md` §Task 8 · investigación `docs/dev/tasks/API-STD-12.md` (+ síntesis `API-STD-15` eje vanta-memory, W7 `API-STD-18`)
- **Fuente:** Backlog Phase 51 (fila `API-08`, `docs/dev/Backlog.md:877`)
- **Esfuerzo:** 🟡 2-3d (tool estimate: 15-30 turns)
- **Prioridad:** 🟡
- **Tipo:** Mixto (Rust crate rustdoc + `docs/api/`)
- **Turns estimados:** 15-30
- **Creado:** 2026-09-26T00:53
- **last-synced:** 2026-09-26T01:30
- **Estado:** ✅ COMPLETED (2026-09-26) — contrato 4/4; Steps 0-3 ✅; review P2-01 ronda 1 ❌ → R1/O1/Nit aplicados → ronda 2 ✅ APPROVE; commit local = lead (política owner)
- **Incógnitas (uphill):** 0 abiertas — D37/D21/MEM-16 (+ scoring MEM-48) resueltas por evidencia (ya pagadas en código; ver §Spec)
- **Pendientes (downhill):** 0/4 steps ✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Consumidores in-process del crate (NO bindings): `vantadb-mcp/Cargo.toml:14` (`features=["llm-driver"]`), `vanta-proxy/Cargo.toml:30` (`llm-driver`), `desktop/src-tauri/Cargo.toml:46` (`default-features=false`). Ninguno lee la doc del crate; los cambios de este task (rustdoc + `docs/api/`) no cambian su compilación. |
| Callees | `vantadb` (core, `default-features=false`); feature-gated: `reqwest` (`llm-driver`), `tiktoken-rs` (`precise-tokens`), `vantadb/remote-inference` (`embeddings`), `vantadb/embed-local` (`embed-local`). |
| Implicaciones | Doc-only: 0 cambios de comportamiento, 0 símbolos nuevos (públicos o de binding), 0 migración de datos. El único archivo de código tocado (`src/lib.rs`) es **solo doc-comment** — recompila el crate, no altera la API. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/lib.rs` (57L) ✅ · `vanta-memory/src/adapters/standalone/llm_runner.rs` (251L) ✅ · `vanta-memory/src/context_engine/token_estimator.rs` (325L) ✅ · `vanta-memory/src/core/hooks/auto_recall.rs` (635L) ✅ · `vanta-memory/Cargo.toml` (69L) ✅ · `docs/api/VANTA_MEMORY.md` (112L) ✅ · `vanta-memory/tests/llm_runner_contract.rs` (122L) ✅ · `vanta-memory/tests/generation_log.rs:1-130` ✅ · `docs/dev/tasks/API-STD-12.md` ✅
- **Archivos leídos (secciones relevantes):** `vanta-memory/src/services/pipeline_worker.rs:600-799` (MEM-43 `run_context_assembly` post-L3) · `vanta-memory/src/core/record/l1_writer.rs:58-91` (hooks `core_embedding_hook`/`local_embedding_hook`) · `vanta-memory/src/core/record/l1_dedup.rs:47-91,518` (auto-on MEM-63) · `vanta-memory/tests/e2e_flow.rs:330-449` (D19/MEM-43 + MEM-37 shared budget) · `vanta-memory/tests/semantic_recall.rs` (listado) · `docs/dev/architecture/adr/ADR-0029-vanta-memory-context-engine.md:59-187` (D21 enmienda) · `docs/dev/architecture/adr/ADR-0053-adr-029-review-guide.md:29-42,356-364`
- **Archivos referenciados hacia dentro (imports):** `src/lib.rs` no importa nada (doc + `pub mod`); `llm_runner.rs` → `crate::core::abstractions::{LlmError, LlmRunParams, LlmRunner}`; `token_estimator.rs` → `context_engine::types`; `auto_recall.rs` → `vantadb::sdk::Embedded` + core record/persona/scene.
- **Archivos que referencian a los editados (referencias entrantes):** `rg VANTA_MEMORY.md` → ADR-0029:158 (cita "superficies públicas F5"), `docs/dev/master-index.md:77`, API-STD-01/12/17, plan ejecución/estandarización, avance `activo/vanta-memory.md:56`, MEM-38. Ninguna referencia depende de las líneas que cambian (solo de la existencia de la página); sin enlaces rotos.
- **Veredicto impacto:** **bajo** — doc-comment + referencia API: sin cambios de contrato, sin símbolos, sin consumers afectados. El riesgo real es la deriva doc↔código, que este task corrige (las deudas documentadas como abiertas ya están pagadas o parcialmente pagadas en código).

## Contrato

"`cargo test -p vanta-memory` verde Y degradado sin `llm-driver` verificado (test `:209-218` pasa) Y D37/D21/MEM-16 con benchmark o DEFER fundado Y 0 símbolos nuevos en bindings"

Mapeo al estado real del repo (2026-09-26):
1. `cargo test --target-dir target/session-api01 -p vanta-memory` → **verificado** GREEN (2 unit — lib 342 + bin vanta-seed 0 — + 25 targets de integración (24 con tests) + 1 doc-test = 28 targets, 0 failed, EXIT=0; log `target/session-api01/api08-test-default.log`).
2. Test de degradación: API-STD-12 lo citó como `:209-218`; **hoy vive en `llm_runner.rs:237-250`** (`llm_free_mode_reports_not_configured`) porque WIRE-11 (`84cb2d19`) insertó `llm_driver_fails_loud_on_unreachable_endpoint` en `:207-235`. Focused run → 1 passed/0 failed.
3. D37/D21/MEM-16 → resueltas por evidencia (pagadas por MEM-46/47/63, BND-03 y MEM-43) + scoring de compresión reclasificado (MEM-48, parcial); residual DEFER fundado (ver §Spec + §Investigation Notes).
4. `rg "vanta[_-]memory"` en `vantadb-python vantadb-ts vantadb-node vantadb-wasm` → **0 matches** (baseline y post-cambio).

## Spec (SDD — decisiones por evidencia, no feature-add)

> Gate D check: la solución **NO agrega símbolos públicos nuevos** (`pub fn`/método/tool/endpoint/binding) — solo doc-comment + `docs/api/`. No dispara Gate D/spec-first de feature-add; la tabla se llena igual porque el contrato exige decisión explícita sobre D37/D21/MEM-16.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | D37 (recall/dedup keyword-overlap "hasta embeddings") | A: benchmark + implementar embeddings en API-08 / B: constatar pago existente + DEFER residual fundado | **B** — ✅ decidido-por-evidencia: ya pagada por MEM-46 `e22b496a` + MEM-47 `f32e4d51` + MEM-63 `6058cc84` (`auto_recall.rs:11-21,336-451` dual-pool RRF; `l1_dedup.rs:72,91` auto-on con `embed-local`). Residual: sin provider adjunto degrada a keyword (diseño P4, `RecallMode::effective` — `auto_recall.rs:87-98`). DEFER con trigger: host adjunta provider / demanda de embeddings hosted. Regla 9 no aplica (no hay optimización nueva). |
| 2 | D21 (`TokenEstimator chars/3`) | A: benchmark de calibración ahora / B: DEFER fundado con enmienda existente | **B** — ✅ decidido-por-evidencia: enmienda ADR-0029 + BND-03 `784b27b9` implementó `precise-tokens` opt-in (cl100k exacto, `token_estimator.rs:51-61`) con golden tests (`:299-324`, verificado GREEN hoy). Default chars/3 aceptado (±20%; CJK ~2×, `guia-revision:38-42`). Trigger de revisión: drift >15% o demanda CJK. |
| 3 | MEM-16 (context engine ↔ pipeline worker) | A: wirear ahora (ya estaba pendiente de decisión) / B: constatar MEM-43 + DEFER | **B** — ✅ decidido-por-evidencia: **ya cableado** por MEM-43 `a0bcb112` (`pipeline_worker.rs:663-755` fase post-L3, budget compartido; e2e `d19_worker_assembles_context_post_l3_with_compression_active`, `e2e_flow.rs:397+`). Doc decía "pendiente de decisión" (deriva). Sin residual. |
| 4 | Exponer vanta-memory en bindings | A: exponer "de paso" / B: NO (Gate P core-only) | **B** — ✅ decidido por Gate P (`API-STD-15:30`): "core-only + API Rust estable; exponer post-release con demanda". Prohibición explícita del task; D42/D43 en `BINDINGS_NAMESPACES.md:14,25-27`. |
| 5 | Forma de "API Rust estable" | A: cambios de código/attributes (`#[non_exhaustive]`, etc.) / B: contrato de estabilidad documentado + verificación mecánica | **B** — ✅ decidido-por-evidencia: la API ya es estable (pub modules L0–L3 sin deuda de diseño abierta); el gap real es doc-deriva (3 deudas) + ausencia de declaración de scope/estabilidad. Añadir attributes sin consumidor = churn (ponytail: `#[non_exhaustive]` se agrega cuando haya enum que crezca; no hay señal). |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** crate **core-only** — 0 símbolos en bindings (Py/TS/Node/WASM) y 0 binding nuevo (Gate P, D42/D43); degradación P4 intacta (sin `llm-driver` → `NotConfigured` → store-all/heurística, nunca bloquea ni pierde datos); `precise-tokens` y `embed-local` siguen opt-in (default lean); no tocar `src/cli.rs`/`src/cli_handlers/**` (API-07 en curso), `vantadb-mcp/**`, `vanta-proxy/**`, `src/server/**`, `src/parser/**`; no mezclar wiki/skills (F3) — no tocar `ingest/`/`core/skill/`.
- **Comandos de verificación:** `cargo test --target-dir target/session-api01 -p vanta-memory` · `cargo test --target-dir target/session-api01 -p vanta-memory --lib llm_free_mode_reports_not_configured` · `cargo test --target-dir target/session-api01 -p vanta-memory --features precise-tokens --lib precise_tokens_match_known_cl100k_golden_values` · `rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm` (0) · `cargo fmt -p vanta-memory --check` · `cargo clippy --target-dir target/session-api01 -p vanta-memory --all-targets -- -D warnings` · `pwsh scripts/validate-docs-coverage.ps1`
- **Deuda pendiente:** residual D37 (sin provider → keyword; trigger en doc) · rustdoc pre-existente: 60 warnings `redundant_explicit_links` + sin `deny(missing_docs)` (MEM-63, no introducidos aquí) · fetcher HTTPS/git D30/D36 (excluido del contrato).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|---------------------------|
| `activeGoal` | Encabezado `# API-08: W7 vanta-memory — API Rust estable (NO exponer, Gate P core-only)` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | API-09 (W8 cierre — depende de API-01..08) |

    contract:
      verificacion: cargo test --target-dir target/session-api01 -p vanta-memory → 28 targets, 0 failed, EXIT=0 (log api08-test-final.log) | cargo test … --lib llm_free_mode_reports_not_configured → 1 passed (llm_runner.rs:237-250) | cargo test … --features precise-tokens --lib precise_tokens_match_known_cl100k_golden_values → 1 passed | rg "vanta[_-]memory" vantadb-python vantadb-ts vantadb-node vantadb-wasm → exit 1 (0 matches) | pwsh scripts/validate-docs-coverage.ps1 → 0 gaps | cargo fmt -p vanta-memory --check → 0 | cargo clippy --target-dir target/session-api01 -p vanta-memory --all-targets -- -D warnings → 0 | campaign_verify_cmd 4/4 passed
      evidencia:
        - claim: "cargo test -p vanta-memory verde en default features"
          evidencia: target/session-api01/api08-test-default.log + api08-test-final.log (28 targets: lib 342 + bin 0 + 25 integración (24 con tests) + doc-test 1; 0 failed; EXIT=0)
          confianza: alta
        - claim: "degradado sin llm-driver verificado (test API-STD-12 :209-218 → hoy :237-250)"
          evidencia: vanta-memory/src/adapters/standalone/llm_runner.rs:237-250 + focused run 1 passed + arqueología git del revisor (84cb2d19~1)
          confianza: alta
        - claim: "D37/D21/MEM-16 con benchmark o DEFER fundado — resueltas por evidencia"
          evidencia: MEM-46 e22b496a + MEM-47 f32e4d51 + MEM-63 6058cc84 (auto_recall.rs:336-451; l1_dedup.rs:63-91); BND-03 784b27b9 (token_estimator.rs:51-61,299-324); MEM-43 a0bcb112 (pipeline_worker.rs:673-755) + MEM-48 4fbaa4a3 (compressor.rs:59-115) — re-verificado por el revisor claim por claim
          confianza: alta
        - claim: "0 símbolos nuevos en bindings"
          evidencia: rg "vanta[_-]memory" en bindings → 0 matches (baseline + post-cambio; diff sin código)
          confianza: alta
        - claim: "review P2-01 agente distinto"
          evidencia: docs/dev/tasks/API-08.md §Review (ronda 1 ❌ R1/O1 → fixes → ronda 2 ✅ APPROVE, sesión ses_f23e51bc6ffeeKMMEmlG7z2aZx)
          confianza: alta
      artefactos:
        - docs/dev/tasks/API-08.md
        - docs/api/VANTA_MEMORY.md (truth-up 2026-09-26)
        - target/session-api01/api08-test-default.log|api08-test-final.log|api08-test-precise-tokens.log|api08-review.diff|api08-ocr-review.json
      invariantes: crate core-only — 0 símbolos en bindings (Gate P/D42/D43); degradación P4 intacta; precise-tokens/embed-local opt-in; no tocar cli/mcp/proxy/server/parser ni ingest/skill; no commit/push por worker
      deuda: residual D37 (sin provider → keyword; trigger en doc) + residual scoring (offload-entry scores; trigger en doc); rustdoc warnings pre-existentes fuera de scope
      queda_pendiente: commit local del lead (`docs(memory): API-08 — …`; archivos: docs/api/VANTA_MEMORY.md, vanta-memory/src/lib.rs, docs/dev/tasks/API-08.md) + skill progreso + bookkeeping plan/Backlog/avance; push solo con instrucción del owner

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** ≤0 — el PR paga la deuda de **documentación desincronizada** (4 ítems del §Deudas declarados abiertos y ya pagados/parcialmente pagados en código: D37/D21/MEM-16 + scoring de compresión MEM-48) y fija el scope/estabilidad del crate. No introduce código nuevo (0 símbolos, 0 deps, 0 cambios de comportamiento). No hay deuda nueva que compensar.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato 4/4 verificado mecánicamente (suite default + focused degrade + precise-tokens golden + grep bindings 0) + doc truth-up + rustdoc estable |
| **Commit** | Atómico, conventional (`docs(memory): API-08 — …`), verificación mecánica; commit = **lead** (esta sesión NO commitea — política owner 2026-09-25) |
| **Release** | `dev-tools/verify.ps1` en el commit del lead (Regla 1); docs mismo-PR (Regla 3); release-plz decide versión/tag (Regla 7) |

## Herramientas necesarias
- Terminal: `cargo` con `--target-dir target/session-api01` (MCP server lockea `target/debug/vanta-cli.exe`), `rg`, `pwsh`
- `codegraph_explore` (blast radius), `campaign_verify_cmd` por step, `vanta-review` (P2-01), `dev-tools/ocr-review.ps1` (advisory)

**Skills cargadas (SDP):** source-driven-development (contrastar doc vs código real: feature gates, firmas, commits) · incremental-implementation (doc truth-up como slice único, ≤100 líneas) · test-driven-development (sin lógica nueva → N/Ajustificado; verificación mecánica del contrato) · doubt-driven-development (re-check adversarial de los claims "ya pagada") · ponytail full (docs-only, diff mínimo, sin attributes cosméticos) · performance-optimization **descartada** (sin claim cuantificado → Regla 9 no aplica). `campaign_discover_skills_v2` devolvió también base (writing-guidelines/writing-plans/campaign-executor/progreso) + api-and-interface-design (aplicado como glosario de estabilidad); `frontend-ui-engineering` descartada (no toca `web/`).

## Investigation Notes

### Estado real de las 3 deudas (re-verificado en código, 2026-09-26)
| Deuda (API-STD-12) | Cita investigación | Estado real HOY | Evidencia |
|---|---|---|---|
| D37 keyword-overlap "hasta embeddings" | `VANTA_MEMORY.md:69,101-102` | **PAGADA** (MEM-46 `e22b496a` + MEM-47 `f32e4d51` + MEM-63 `6058cc84`): dual-pool cosine+keyword con RRF; auto-on `local_embedding_hook()` con `embed-local`; sin provider → keyword (diseño P4) | `auto_recall.rs:11-21,87-98,336-451`; `l1_writer.rs:58-91`; `l1_dedup.rs:72,91`; `tests/semantic_recall.rs` 5/5 |
| D21 `chars/3` | `VANTA_MEMORY.md:102` | **PAGADA** (enmienda ADR-0029 + BND-03 `784b27b9`): `precise-tokens` opt-in cl100k exacto + golden tests; default chars/3 aceptado (±20%, CJK ~2×) | `token_estimator.rs:7-11,51-61,296-324`; golden run 1/1 GREEN; `guia-revision:38-42` |
| MEM-16 context↔worker | `VANTA_MEMORY.md:104` | **PAGADA** (MEM-43 `a0bcb112`): fase post-L3 con budget compartido + report persistido | `pipeline_worker.rs:663-799`; `e2e_flow.rs:397+` 6/6 |

### Deuda de docs detectada (lo que este task corrige)
- `VANTA_MEMORY.md:69` afirmaba que embedding/hybrid "degradan hasta que el core exponga embeddings" — falso desde MEM-47/63.
- `VANTA_MEMORY.md:101-104` listaba D37/D21/MEM-16 como deudas abiertas — desactualizado.
- `VANTA_MEMORY.md:44` firma del trait `LlmRunner` sin `&` en `params` (el trait real es `fn run(&self, params: &LlmRunParams)` — `llm_runner.rs:93`) — corregir.
- `VANTA_MEMORY.md` no declaraba scope/estabilidad (Gate P core-only) ni el contrato de degradación con referencias a tests.

### Línea `:209-218` del contrato (deriva de números)
API-STD-12 (2026-09-24) citó el test de degradación en `:209-218`. Hoy `:207-235` es el test WIRE-11 (`llm_driver_fails_loud_on_unreachable_endpoint`, `84cb2d19`) y el test de degradación default quedó en **`:237-250`** (`llm_free_mode_reports_not_configured`). Se documenta el mapeo para que el próximo agente no busque en la línea equivocada.

### Consumidores del crate (no bindings)
`vantadb-mcp` (llm-driver), `vanta-proxy` (llm-driver), `desktop/src-tauri` (default-features=false). Ninguna superficie Py/TS/Node/WASM referencia `vanta-memory` (grep 0) — consistente con D42/D43 y el contrato "0 símbolos nuevos en bindings".

### NOTICED BUT NOT TOUCHING (scope discipline)
- `VANTA_MEMORY.md` no cubre módulos post-MEM-38 (`dream/`, `approval`, `ingest/auto_sync`, `conversation_hook`, `generation_log`) — expansión de cobertura fuera del contrato de estabilización → **FIND-160** registrada en `docs/dev/Backlog.md` (discovery 2026-09-26).
- Rustdoc pre-existente: 60 warnings `redundant_explicit_links` + ausencia de `deny(missing_docs)` (MEM-63) — no se toca (sin relación con el contrato; limpieza masiva sería scope creep).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — D37/D21/MEM-16 (+ scoring MEM-48) resueltas por evidencia en §Spec/§Investigation Notes |
| Pendientes de ejecución (downhill) | 0 (Steps 0-3 ✅) |
| % completado | 100% (4/4 steps · contrato 4/4 · review ronda 2 ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no toca trust boundaries: sin deps nuevas, sin código ejecutable nuevo, sin inputs de usuario. El crate mantiene `publish = false` y 0 exposición en bindings (superficie de ataque sin cambios).
- [x] **PERFORMANCE** — no toca hot paths ni reclama performance. Regla 9 no aplica: no hay optimización en este task (las deudas D37/D21/MEM-16 se constatan pagadas; su residual queda en DEFER fundado con trigger). No se publican números sin fuente reproducible (Regla 11).

## Steps

### Step 0: Discovery + evidencia baseline (contrato) — ✅ DONE 2026-09-26
- **Archivos:** `vanta-memory/src/**`, `docs/api/VANTA_MEMORY.md`, bindings (solo lectura/grep)
- **Acción:** auto-tipo → `docs`/estabilización (Gate D no disparado: 0 símbolos nuevos); SDP v2; codegraph blast radius (`TokenEstimator` 13 callers, `LlmRunner` implementors ×8 sin exposición); baseline mecánico del contrato; lectura Regla 0 completa (ver §Impacto mapeado)
- **Verify:** suite default GREEN (`target/session-api01/api08-test-default.log`, EXIT=0) + focused degrade 1 passed + precise-tokens golden 1 passed + grep bindings 0 + `codegraph_explore` ejecutado
- **Estado:** ✅ DONE 2026-09-26 — 2 unit (lib 342 + bin 0) + 25 integración (24 con tests) + 1 doc-test = 28 targets, 0 failed; `llm_free_mode_reports_not_configured` 1/1; `precise_tokens_match_known_cl100k_golden_values` 1/1; `rg vanta[_-]memory` bindings = 0; blast radius bajo (doc-only).

### Step 1: Doc truth-up — `docs/api/VANTA_MEMORY.md` (stability + flags + degradación + deudas)
- **Archivos:** `docs/api/VANTA_MEMORY.md`
- **Acción:** (a) sección "Scope & stability (Gate P / D42 / D43)" — core-only, `publish=false`, no bindings, 0 símbolos por diseño; (b) tabla de feature flags (`llm-driver`/`embeddings`/`embed-local`/`precise-tokens`/`mock`/`fjall`/`http-server`, default off); (c) sección "Degradation contract (P4)" con refs a tests reales (`:237-250` + por capa); (d) corregir firma `LlmRunner` (`&LlmRunParams`); (e) reescribir §Deudas: D37/D21/MEM-16 → resueltas con evidencia; residual D37 + scoring heurístico + fetcher → DEFER con trigger; (f) `last_reviewed: 2026-09-26`. ≤100 líneas netas.
- **Verify:** `rg "hasta que el core exponga embeddings|pendiente de decisión" docs/api/VANTA_MEMORY.md` → 0 · `rg "Gate P|llm-driver|precise-tokens" docs/api/VANTA_MEMORY.md` ≥3 · `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps
- **Estado:** ✅ DONE 2026-09-26 — doc reescrita: §Scope & stability (Gate P) + §Feature flags (7 features) + §Degradation contract (refs a tests) + firma `LlmRunner` con `&` + `RecallConfig { scope }` corregido + §Debts resueltas/DEFER con commits; stale phrases 0; anchors 11; docs-coverage 0 gaps (EXIT=0). **Ronda 1 review:** R1 aplicado — scoring heurístico reclasificado a parcialmente pagado por MEM-48 (`4fbaa4a3`) con residual + trigger (`compressor.rs:7-9`); O1 aplicado (precisión gate keyword en Recall).

### Step 2: Rustdoc stability — `vanta-memory/src/lib.rs` (doc-comment, 0 símbolos)
- **Archivos:** `vanta-memory/src/lib.rs:15-24`
- **Acción:** párrafo `//! ## Stability`: core-only por decisión (Gate P, D42/D43), crate interno `publish=false` sin re-export en bindings, superficie estable = doc `docs/api/VANTA_MEMORY.md`, exposición futura = nuevos bindings + demanda (post-release)
- **Verify:** `cargo test --target-dir target/session-api01 -p vanta-memory --doc` → 1 passed · `cargo fmt -p vanta-memory --check` → 0 · `cargo clippy --target-dir target/session-api01 -p vanta-memory --all-targets -- -D warnings` → 0
- **Estado:** ✅ DONE 2026-09-26 — `//! ## Stability` (core-only Gate P/D42/D43, `publish=false`, sin re-export en bindings, doc canónica) + path del plan archivado corregido; doc-test 1/1; fmt 0; clippy `-D warnings` 0 (44s incremental).

### Step 3: Verify final + cierre (review P2-01, recitation)
- **Archivos:** task file (+ artefactos de verificación en `target/session-api01/`)
- **Acción:** re-run contrato 4/4 vía `campaign_verify_cmd` (suite default + focused degrade + grep bindings + docs-coverage), fmt/clippy scoped, OCR advisory (`pwsh dev-tools/ocr-review.ps1 -Format json`), review `vanta-review` (subagente fresco) → registrar veredicto en §Review, sync task file, recitation final
- **Verify:** `campaign_verify_cmd` 4/4 + veredicto review registrado
- **Estado:** ✅ DONE 2026-09-26 — contrato 4/4 vía `campaign_verify_cmd` (suite `passed=true` 342/0; focused degrade `passed=true` 1/0; bindings `exitCode=1` esperado/0 matches; docs-coverage `passed=true` 0 gaps) + fmt/clippy scoped 0 + OCR advisory (`api08-ocr-review.json`; `lib.rs` doc-only en rule group Rust) + review P2-01 ronda 1 ❌ (R1/O1/Nit) → fixes → **ronda 2 ✅ APPROVE**; task file sincronizado. Sin commit (lo hace el lead).

## Dependencias
- API-01 ✅ (tipos/error; sin dependencia de código nueva)
- Paralela con API-07 (disjunta: CLI vs vanta-memory). Desbloquea: API-09 (cierre W8; `validate-docs-coverage` + VERSIONING).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (subagente, contexto fresco, sesión `ses_f23e51bc6ffeeKMMEmlG7z2aZx`) — 2 rondas
- **Enfoque:** ¿la resolución de D37/D21/MEM-16 (y las deudas restantes) por evidencia es correcta vs benchmark? ¿el doc truth-up refleja el código real? ¿el scope core-only se respeta (0 símbolos en bindings)? ¿el approach docs-only es adecuado para una estabilización 🟡?
- **Cómo se probó (no auto-reporte):** el revisor re-ejecutó las verificaciones del contrato con `--target-dir target/session-api01` (suite 28 targets/0 failed; focused degrade 1/1; golden precise-tokens 1/1; grep bindings 0) + `fmt --check`, `clippy -D warnings`, `validate-docs-coverage` (0 gaps); verificó por arqueología git el mapeo `:209-218 → :237-250` (`git show 84cb2d19~1`); contrastó una a una las claims "ya pagada" contra código (MEM-46/47/63, BND-03, MEM-43) y las tablas de feature flags/degradación; inspeccionó el diff completo (sin código nuevo, 0 `pub` items).
- **Checklist anti-hábitos tóxicos** (contrato §12 agent-02-task-execution) — evaluado por el revisor:
  - [x] No inventar salidas de comandos → re-ejecutó todo fresh (rondas 1 y 2).
  - [x] No saltarse la clarificación por "ya sé qué quiere" → decisiones por evidencia Gate P/ADR-029.
  - [x] No declarar done sin verificar acceptance criteria → contrato 4/4 re-verificado.
  - [x] No ignorar fallos → ronda 1 🔴 R1 (deriva MEM-48) atendido, no silenciado.
  - [x] No hacer un solo intento de búsqueda → spot-checks multi-fuente (código + commits + changelog + avance).
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia → **excepción ronda 1**: el ítem "Heuristic compression scoring" se heredó del doc viejo sin re-verificar (R1, corregido en ronda 2).
  - [x] No reintentar en bucle sin diagnóstico → 2 rondas, fixes quirúrgicos.
  - [x] No dejar huérfanos los pasos → Steps 0-3 conectados al contrato.
  - [x] No degradar chequeos en paths de dinero/seguridad → no aplica (docs-only).
  - [x] No gastar presupuesto infinito → paradas explícitas por ronda.
- **Ronda 1 (2026-09-26): ❌ cambios requeridos** — contrato 4/4 PASS; R1 (blocker): ítem "Heuristic compression scoring" desactualizado (MEM-48 `4fbaa4a3` ya consume scores L1 reales — `compressor.rs:59-115`, `pipeline_worker.rs:735-753`) + heading "trigger-gated" incoherente; O1 (Low): precisión de prosa en Recall; Nit: conteos del task file.
- **Fixes aplicados:** R1 (reclasificado a "partially paid by MEM-48" con residual + trigger `compressor.rs:7-9`), O1 (gate keyword explícito), Nit (conteos 28 targets corregidos).
- **Ronda 2 (2026-09-26): ✅ APPROVE** — R1 verificado claim por claim (incl. trigger no cumplido: `OffloadEntry.score` existe pero los writers lo dejan `None` → residual genuino, `offload/types.rs:35-38`); O1 exacto; numeración 1-5 coherente; checks mecánicos fresh verdes. "Task DoD: PASS · Commit/Release pendientes del lead (política owner)".
- **Veredicto:** ✅ **APPROVE** (ronda 2)

## Notas
- **No commit / no push:** política owner 2026-09-25 — el commit local lo hace el lead; push solo con instrucción explícita. `skill progreso` + bookkeeping plan/Backlog/avance = lead al cierre (patrón API-04/05/06).
- **Entorno:** `--target-dir target/session-api01` obligatorio (MCP server lockea `target/debug/vanta-cli.exe`); disco C: ~13 GB libres al inicio; `--jobs 2` por política de sesiones anteriores.
- **Estrategia:** slice único doc-only (truth-up + stability) — no hay lógica nueva que red/verify; el valor está en verificaciones mecánicas + doc fiel (Regla 3) + decisión fundada de las deudas.
- **Idioma del doc:** `VANTA_MEMORY.md` era mixto ES legado; las secciones nuevas/reescritas quedan en EN (Doc Language Split: `docs/api/` = English); traducción completa del resto = fuera de scope (candidato para API-09/docs pass).
- **OCR advisory:** `dev-tools/ocr-review.ps1` corre en modo workspace (incluye WIP de API-07); evidencia usable = `vanta-memory/src/lib.rs` (doc-only, rule group Rust: sin typos/código/unsafe). Scoping del script al changeset = mejora futura del tool, no de este task (O2 del review).
- **Pre-mortem del plan (F1-F3):** F1 (binding nuevo "de paso") → §Spec #4 + grep de cierre; F2 (`NotConfigured` fatal) → verificado no-fatal (suite default + focused; tests por capa); F3 (wiki/skills mezclados) → invariante de scope (no se toca `ingest/`/`core/skill/`).
- **Riesgos del plan cubiertos:** scope explosion → DEFER D42 no re-abierto (Gate P); "sin benchmark" → Regla 9 no aplica, DEFER fundado con triggers documentados.
- **Lección (ronda 1):** en truth-up docs, todo ítem heredado se re-verifica contra código — el ítem "compression scoring" venía stale del doc anterior y el reviewer lo cazó; los ítems resueltos se marcan con commit + evidencia (no "upgrade futuro").
