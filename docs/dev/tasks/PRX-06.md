# PRX-06: Task-aware routing por tier

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-code.md
- **Fuente:** plan file Task 7 (Wave2)
- **Esfuerzo:** 🟠 2d
- **Prioridad:** 🟠 Media-Alta
- **Tipo:** Rust (vanta-proxy)
- **Turns estimados:** 20
- **Creado:** 2026-09-10
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (diseño cerrado en Spec; Responses SSE shape = tolerante+fail-open)
- **Pendientes (downhill):** 3 slices

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `server.rs` (process/process_inner/forward_raw/forward_with_tool_loop), 12 sitios `ProxyConfig {}` literales en tests |
| Callees | `session/claude_code.rs` (classify_cc_request), `inject.rs` (Protocol), `sse_intercept.rs`, `memory_tools.rs`, `forward.rs` (failover) |
| Implicaciones | TOML legacy compat (#[serde(default)]); routing default OFF (transparent proxy intacto); failover preservado (reorder-first); Responses SSE parse tolerante (unknown events skip, fail-open replay) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** vanta-proxy/src/server.rs (760L), vanta-proxy/src/config.rs (373L), vanta-proxy/src/session/claude_code.rs (285L), vanta-proxy/src/memory_tools.rs (437L), vanta-proxy/src/lib.rs, vanta-proxy/tests/tool_loop.rs (370L), vanta-proxy/src/sse_intercept.rs (head 120L), vanta-proxy/src/inject.rs (head 80L)
- **Archivos referenciados hacia dentro:** server.rs → auth/cache/capture/config/cost/forward/handlers/inject/mem_command/memory_tools/rate_limit/report/session/sse_intercept/writeback; config.rs → error; claude_code.rs → (solo serde_json)
- **Archivos que referencian a los editados:** `ProxyConfig` 17 callers (server.rs, config.rs, 9 suites tests); `classify_cc_request` 1 caller real (server.rs:76 is_cc_sidequery) + tests; tool-loop gate en server.rs:483; `upstreams_resolved` en forward paths
- **Veredicto impacto:** medio — nuevo módulo aditivo + 1 campo config (rompe 12 literales test: fix mecánico 1 línea c/u) + firmas privadas process_inner/forward_raw/forward_with_tool_loop (sin callers externos); sin cambios a WAL/vector/storage; sin endpoints nuevos

## Contrato

`cargo test -p vanta-proxy` 0 failed + test tier→modelo/upstream ✅ + `/v1/responses` en tool-loop ✅ + `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0

## Spec (SDD — feature-add: nuevo módulo pub + capability nueva)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive mapeo tier→modelo/upstream | A: `[routing]` en config.rs (precedente PRX-02 upstreams, PRX-03 cost) / B: archivo TOML separado (más plumbing, sin precedente) | A | ✅ decidido-por-evidencia (config.rs:17-41 patrón establecido) |
| 2 | Semántica upstream por tier | A: índice en lista resolved + reorder-first (failover intacto) / B: lista separada por tier (duplica failover, rompe compat) | A | ✅ decidido-por-evidencia (forward_with_failover en forward.rs:216) |
| 3 | Modo default | A: shadow/log-only (safe default, stop condition plan) / B: enforce directo (riesgo mala clasificación en prod) | A shadow | ✅ plan pre-mortem (override por key; calidad<baseline→shadow+DEFER) |
| 4 | Override por key | A: `bypass_keys: Vec<String>` user_ids que saltan routing / B: por-key tier fijo (más knobs, sin necesidad probada) | A | ✅ plan pre-mortem explícito |
| 5 | Responses en tool-loop | A: gate + `responses_message` accumulator + append a `input` array (string-input→replay, ponytail ceiling) / B: solo gate sin parse (loop entra pero nunca ejecuta — contrato a medias) | A | ✅ contrato exige loop real |
| 6 | Alcance tier | A: solo Anthropic (CC classifier solo vale ahí) / B: heurística para OpenAI (inventar clasificación — prohibido) | A | ✅ claude_code.rs:57 firma Anthropic-only |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** proxy transparente por default (routing.enabled=false → byte-identical); failover PRX-02 intacto en todos los paths; D34 fail-closed intacto (routing post-auth); Responses no-loop → replay verbatim (fail-open); TOML legacy sin `[routing]` parsea igual; `unwrap/expect` prohibido en prod (solo tests)
- **Comandos de verificación:** `cargo test -p vanta-proxy --tests -j 2` (0 failed) + `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` (0) + `cargo fmt --check`
- **Deuda pendiente:** wiring `record_response_usage` SSE (heredada PRX-03); Responses SSE shape real vs sintético (parser tolerante, validación manual futura)

## Recitation (canónico — estructura única)

Ver pipeline-full.md §3 (se sincroniza vía campaign_update_task_state).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (módulo aditivo puro; `ponytail:` ceilings documentados, no deuda).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | contrato ✅ + fmt/clippy/nextest + tests nuevos pasan |
| **Commit** | atómico solo-propios en develop, `feat: PRX-06 — ...`, hooks verdes |
| **Release** | N/A (no release; develop) |

## Herramientas necesarias

- cargo test/clippy/fmt (stack VantaDB)
- codegraph_explore (blast radius — hecho)

**Skills cargadas (SDP):** test-driven-development (lógica nueva RED→GREEN) · incremental-implementation (3 slices verticales) · context-engineering (context pack por slice) · api-and-interface-design (nueva config pública `[routing]`) · base auto-MCP: campaign-executor, source-driven-development, progreso. SDP v2 keywords: vanta-proxy/src/server/config. `frontend-ui-engineering` y `doubt-driven-development` sugeridas por scoring pero descartadas: sin UI web ni stakes de seguridad (routing post-auth, fail-open).

## Investigation Notes

- Clasificador CC existe huérfano: `classify_cc_request` (Main/Fork/Sidequery) solo usado para Sidequery-bypass (server.rs:76); distinción Main/Fork sin mapear → PRX-06 la mapea a tiers.
- Tool-loop gate (server.rs:483) excluye Responses aunque parse (`openai_message`) y `append_exchange` ya contemplan `Protocol::Responses` → contrato lo incluye.
- Responses `append_exchange` actual: `get_mut("messages")` → None en bodies Responses (`input`, no `messages`) → no-op. Slice 3 lo corrige para `input` array.
- Gate D: sin `question` — diseño implícito aprobado por owner (plan Task 7: "usa clasificador existente", "override por key", "modo shadow + DEFER enforce") + precedente config-extension PRX-02/03.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas | 0 |
| Pendientes de ejecución | 3 (slice1 routing.rs+config / slice2 server wiring+test / slice3 responses loop+test) |
| % completado | 10% (discovery + task file) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — aplica parcial: routing corre POST-auth (D34 intacto), bypass_keys son user_ids ya autenticados, model rewrite solo campo `model` (sin inyección de headers/upstream URL — índices, no URLs). Sin inputs de red nuevos. No se carga security-and-hardening (sin trust boundary nuevo; fail-open).
- [x] **PERFORMANCE** — no aplica: path no-hot (1 classify + 1 rewrite por request, O(n) mensajes ya recorrido por classifier); sin benchmark (sin hot path tocado). Justificado.

## Steps

### Step 1 (slice 1): routing.rs + config `[routing]` + RED
- **Archivos:** `vanta-proxy/tests/prx06_routing.rs` (nuevo, RED primero), `vanta-proxy/src/routing.rs` (nuevo), `vanta-proxy/src/lib.rs`, `vanta-proxy/src/config.rs`
- **Acción:** RED: test tier→modelo/upstream + TOML compat (falla: módulo no existe). GREEN: `Tier`, `TierRoutingConfig` (enabled/shadow/model overrides/upstream idx/bypass_keys), `tier_of`, `resolve`, `apply_model_override`; campo `routing` en ProxyConfig; fix literales test exhaustivos
- **Verify:** `cargo test -p vanta-proxy --test prx06_routing` + clippy + fmt
- **Estado:** ✅ COMPLETED (10/10 + clippy 0 + fmt OK; +fix literales 9 tests + cfg_with)

### Step 2 (slice 2): server wiring enforce/shadow + test e2e tier
- **Archivos:** `vanta-proxy/src/server.rs`, `vanta-proxy/tests/prx06_routing.rs` (extender)
- **Acción:** `ResolvedRoute` en process() (classify post-auth identity para bypass) → thread upstreams a forward_raw/forward_with_tool_loop (reorder-first); model rewrite pre-inject; test e2e: fork→modelo barato + upstream alternativo observado en mock
- **Verify:** `cargo test -p vanta-proxy --test prx06_routing` + suite proxy_wire/pipeline verdes
- **Estado:** ✅ COMPLETED (11/11 incl. e2e fork→B+haiku; suite full 0 failed; clippy 0; fmt OK)

### Step 3 (slice 3): `/v1/responses` en tool-loop + test
- **Archivos:** `vanta-proxy/src/sse_intercept.rs` (`responses_message`), `vanta-proxy/src/server.rs` (gate), `vanta-proxy/src/memory_tools.rs` (append `input` array), `vanta-proxy/tests/prx06_routing.rs` o tool_loop.rs (test responses loop)
- **Acción:** gate incluye Responses; accumulator eventos `response.output_item.added`/`function_call_arguments.delta`/`output_text.delta`; append function_call[_output] a `input` array (string→replay); test e2e responses con memory tool → 2 forwards
- **Verify:** contrato full `cargo test -p vanta-proxy` 0 failed + clippy 0
- **Estado:** ✅ COMPLETED (12/12 incl. e2e responses loop; suite full 0 failed; clippy 0; fmt OK)

## Dependencias
- PRX-02 ✅ (failover a preservar), PRX-03 ✅ (cost wiring convive), PRX-09 slice1 ✅ (cache pre-tool-loop convive)
- Wave2 paralelo SRV-06/PROV-11 disjuntos (core-server/providers) — sin colisión

## Review (GATE — agente distinto, P2-01)
- **Revisor:** pendiente (orquestador: vanta-review post-commit)
- **Enfoque:** approach config-extension + reorder-first + Responses tolerant parse
- **Cómo se probó:** suite full 0 failed (lib 134 + 11 suites), e2e tier + responses, clippy/fmt 0
- **Veredicto:** pendiente

## Notas
- `ponytail:` reorder-first en vez de listas por tier (failover gratis); bypass_keys en vez de per-key tiers; string-input Responses → replay (append solo array).
- Deuda ajena NO tocar: M opencode.jsonc + M .opencode + ?? Investigacion-plan.md.
