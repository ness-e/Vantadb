# PRX-09: Semantic caching — exact primero (slice 1)

## Metadata

- **Plan file:** docs/plans/2026-09-09-backlog.md (Task 5)
- **Fuente:** plan Task 5 + Gate Justificación gateway completo
- **Esfuerzo:** 🟠 2d (slice 1 exact ≤3d appetite)
- **Prioridad:** 🟠 Media-Alta
- **Tipo:** Rust (vanta-proxy)
- **Turns estimados:** 12
- **Creado:** 2026-09-09
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps — S2✅ S3✅ S4✅ S5✅

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `server.rs` `process_inner` (único caller nuevo); tests `tests/*.rs` (5 literales `ProxyConfig`) |
| Callees | `config.rs` (CacheConfig), `memory_tools.rs` (`announces` pub(crate) ya usado en server.rs:333), `serde`, `std::collections` |
| Implicaciones | contrato HTTP no cambia (hit devuelve mismo status/body); forward/inject intactos; `ProxyConfig` gana campo con `#[serde(default)]` → TOML viejo compatible; 5 literales de test requieren 1 línea c/u |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-proxy/src/inject.rs` (531L), `vanta-proxy/src/server.rs` (583L), `vanta-proxy/src/config.rs` (213L), `vanta-proxy/src/lib.rs`, `vanta-proxy/src/forward.rs`, `vanta-proxy/src/handlers/openai.rs`, `vanta-proxy/tests/pipeline.rs`, `vanta-proxy/Cargo.toml`, `.opencode/rules/api-contract.md`, `.opencode/rules/server-mcp.md`, `.opencode/rules/concurrency-async.md` (R-1..R-8)
- **Archivos referenciados hacia dentro:** cache.rs nuevo → `config::CacheConfig`, `memory_tools::announces` (pub(crate)); server.rs → `cache::{ExactCache, CachedEntry}`; config.rs → nada nuevo
- **Archivos que referencian a los editados:** `tests/pipeline.rs`, `proxy_wire.rs`, `prx01_wiring.rs`, `prx05_aux.rs`, `tool_loop.rs` construyen `ProxyConfig{}` literal (rompen al agregar campo → fix 1 línea c/u); `config.toml` ejemplo (opcional, defaults cubren)
- **Veredicto impacto:** medio-bajo — aditivo puro (módulo nuevo + campo config con default + hook en session-path); verbatim early-returns (sidequery/sin-sesión) y tool-loop intactos; streaming SSE nunca bufferizado (bypass por content-length ausente)

## Contrato

`cargo test -p vanta-proxy` 0 failed + test cache-hit exact byte-a-byte ✅ + sin regresión PRX-04 (prefijo estable) + `cargo clippy -p vanta-proxy --all-targets -- -D warnings` 0

## Spec (SDD — feature-add: módulo `pub` nuevo)

Gate D evaluado: símbolos `pub` nuevos (`ExactCache`, `CacheConfig`, `CachedEntry`) **pre-aprobados por el plan** (Task 5 Archivos clave: `cache.rs (nuevo)`; Gate Result ✅ DO). Sin `question` adicional.

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Clave del cache | A) body post-inyección completo como clave (0 deps, exactitud total) / B) hash sha256 (requiere crate `sha2` nueva) | A | ✅ decidido-por-evidencia (plan: sin nuevas deps; memoria acotada por max_entries) |
| 2 | Qué se cachea | A) solo 2xx JSON no-SSE con content-length ≤4MB / B) todo incl. SSE (rompe streaming, riesgo pérdida body) | A | ✅ decidido-por-evidencia (server.rs tool-loop ya distingue SSE por content-type; forward.rs streams sin buffer) |
| 3 | Concurrencia | A) `std::sync::Mutex` con secciones 100% sync sin `.await` / B) `tokio::sync::Mutex` (requiere feature `sync` en Cargo.toml) | A | ✅ decidido-por-evidencia (concurrency-async R-2 permite guard sync sin await; tokio proxy sin feature sync) |
| 4 | Default | A) `enabled=false` opt-in / B) enabled=true | A | ✅ safe-defaults (incremental Rule 4; evita cambio de comportamiento sorpresivo) |
| 5 | Invalidación | A) implícita (clave incluye body inyectado → cambio memoria = miss) + `invalidate_all()` explícito / B) TTL por entrada | A | ✅ decidido-por-evidencia (PRX-04 despeja: prefijo estable; TTL = DEFER slice 2) |
| 6 | Evicción | A) FIFO acotado (`VecDeque`, O(1)) / B) LRU (crate `lru` nueva) | A | ✅ ponytail (3 líneas vs dep; techo conocido) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** PRX-04 prefijo estable byte-a-byte (re-inject = None o idéntico); D34 auth siempre antes que cache (nunca servir hit sin autenticar); streaming SSE jamás bufferizado; `unwrap`/`expect` prohibidos en prod (workspace deny); WIP ajeno intacto (opencode.jsonc, .opencode, Investigacion-plan.md, Backlog.md con edición ajena)
- **Comandos de verificación:** `cargo test -p vanta-proxy` (0 failed) · `cargo clippy -p vanta-proxy --all-targets -- -D warnings` (0) · `cargo fmt --check` (0)
- **Deuda pendiente:** semántico por embeddings (slice 2, DEFER por pre-mortem); TTL/evicción LRU (DEFER); `skill progreso` + sync plan Task 5 los hace el orquestador (Backlog.md con edición ajena)

## Deuda técnica (Regla 6 — MUST)

**Saldo neto:** 0 — FIFO O(1) en vez de LRU es simplificación con techo conocido (`ponytail:` tag en código), no deuda nueva sin pago: no se introduce `unsafe`, `clone` en hot path (solo en buffer ≤4MB ya asignado), ni dependencias.

## Definition of Done

- Task: contrato ✅ + fmt/clippy/nextest ✅ + tests nuevos pasan
- Commit: atómico, `feat: PRX-09 ...`, solo archivos propios (NO stagear WIP ajeno)
- Release: N/A (no se pushea; push = vanta-lead)

## Herramientas necesarias

- cargo check/clippy/fmt/nextest (terminal)
- codegraph_explore (blast radius — usado)

**Skills cargadas (SDP):** incremental-implementation (slices verticales) · test-driven-development (RED→GREEN, Prove-It) · context-engineering (jerarquía Rules→Plan→Source) · source-driven-development (base tipo proxy) · systematic-debugging (fallos verify) · api-and-interface-design (módulo `pub` nuevo). Descartada: frontend-ui-engineering (sin web/), doubt-driven-development (stakes medios, cache opt-in default-off).

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — evaluado: cache después de auth D34 (hit solo tras `authenticate` OK); `x-vanta-user-key` nunca en clave ni log; solo 2xx cacheados (no se cachea 401/502); session-key queda dentro del body inyectado (aislamiento por clave). Sin `security-and-hardening` (sin input parsing nuevo, sin deps).
- [x] **PERFORMANCE** — evaluado: NO es hot path de motor (proxy I/O-bound, no `vector/`/`engine.rs`); lookup O(1) HashMap; buffer acotado 4MB con bypass por content-length; sin baseline requerido (Regla 9 aplica a hot paths del motor). Sin `performance-optimization`.

## Steps

### Step 1: Task file + IN PROGRESS

- **Archivos:** `docs/tasks/PRX-09.md`
- **Acción:** crear task file (Spec + Regla 0) y marcar in-progress
- **Verify:** existe + `campaign_update_task_state`
- **Estado:** ✅ COMPLETED

### Step 2: RED — test cache-hit exact + combinado PRX-04

- **Archivos:** `vanta-proxy/tests/prx09_cache.rs` (nuevo)
- **Acción:** 3 tests: `exact_hit_byte_identical_upstream_once` (mock cuenta hits; 2× mismo request con sesión → 1 hit upstream + bodies byte-idénticos); `memory_change_invalidates` (cambio persona → miss, upstream 2); `prx04_prefix_stable_with_cache` (re-inject estable sobre body cacheado)
- **Verify:** `cargo test -p vanta-proxy --test prx09_cache` falla (módulo `cache` inexistente = RED)
- **Estado:** ✅ COMPLETED (RED confirmado: E0432/E0560/E0433)

### Step 3: GREEN — cache.rs + config

- **Archivos:** `vanta-proxy/src/cache.rs` (nuevo, ~120L), `lib.rs` (+1 mod), `config.rs` (+CacheConfig)
- **Acción:** `ExactCache` (HashMap+VecDeque FIFO, std Mutex externo), `CachedEntry{status,content_type,body}`, `is_cacheable_request` (sin `stream:true`), `CacheConfig{enabled:false,max_entries:128}`
- **Verify:** `cargo check -p vanta-proxy` + RED pasa a GREEN
- **Estado:** ✅ COMPLETED

### Step 4: Wiring session-path + literales

- **Archivos:** `server.rs` (AppState.cache + lookup tras inject + store tras forward con gate content-length), 5 tests literales (+`cache: Default::default()`)
- **Acción:** lookup solo en session-path post-inject (tras auth ✅); store solo 2xx JSON con content-length ≤4MB; SSE/chunked bypass
- **Verify:** `cargo test -p vanta-proxy` 0 failed
- **Estado:** ✅ COMPLETED

### Step 5: Verify full + commit + cierre

- **Archivos:** —
- **Acción:** fmt + clippy + nextest; self-review 3 ejes; commit solo archivos propios; memory_write lesson; update-task-state completed
- **Verify:** contrato completo ✅
- **Estado:** ✅ COMPLETED

## Dependencias

- PRX-05 ✅ a1855dd6 (necesario: base handlers estable)
- PRX-04 ✅ (necesario: prefijo estable para clave)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** pendiente (orquestador asigna vanta-audit/vanta-review; implementador = worker actual)
- **Enfoque:** —
- **Cómo se probó:** —
- **Veredicto:** ⬜ pendiente

## Notas

- Slice 1 = solo exacto (stop condition plan: appetite >3d → shippear exact + DEFER semántico). Semántico slice 2.
- No re-derivar contexto del plan: gate/wave/cynefin ya verificados (plan Task 5).
- `campaign_verify_cmd` con bug exit -1 conocido → bash directa si falla.
- SDP keywords: vanta-proxy/src/cache/inject (base + lifecycle BUILD, manifest sin candidatos extra).
- **Verify real S5:** `cargo test -p vanta-proxy` 142 passed 0 failed (109 lib + 5 suites + 3 prx09 nuevos) · `cargo fmt --check` 0 · `cargo clippy --all-targets -- -D warnings` 0 · PRX-04 intacto (12 tests inject + pipeline/tool_loop verdes).
- **Debug S4:** 2º request miss → causa raíz `forward.rs` elimina `content-length` (streaming-safe) → gate de longitud imposible → fix: gate por content-type + cap 4MB en colecta (502 tipado si upstream miente longitud).
- **Self-review 3 ejes:** correctitud (clave post-inject, hit byte-idéntico, auth previo, bypass SSE/stream/errores) · simplicidad (0 deps nuevas, FIFO O(1), Mutex sync sin await) · consistencia (patrones pipeline.rs/mem_command).
- **Scope:** 10 archivos propios (2 nuevos); WIP ajeno intacto, no stageado.
