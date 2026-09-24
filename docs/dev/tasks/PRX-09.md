# PRX-09: Semantic caching — exact primero (slice 1)

## Metadata

- **Plan file:** docs/dev/plans/2026-09-09-backlog.md (Task 5)
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

- **Archivos:** `docs/dev/tasks/PRX-09.md`
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

## Slice 2 — Semantic caching + TTL + LRU (plan 2026-09-10-code Task 13)

- **Estado:** ⏳ IN PROGRESS
- **SDP:** incremental-implementation · test-driven-development · context-engineering · source-driven-development · api-and-interface-design (+ campaign-executor/progreso base). Descartadas: frontend-ui-engineering (sin web/), doubt-driven-development (stakes medios, opt-in), security-and-hardening (sin input parsing nuevo, D34 intacto), performance-optimization (proxy I/O-bound, n≤max_entries).
- **Context cargado:** AGENTS.md Reglas 0/1/4/6/9 · plan Task 13 (no re-derivar) · cache.rs 223L completo · config.rs CacheConfig · server.rs hook 5b/5c + maybe_store · prx09_cache.rs · inject.rs patrón `value.get_mut("messages")` · Cargo.toml vanta-proxy (sin lru dep; lock trae lru 0.16.4/0.18.4 transitivo).
- **Diseño (aditivo, 0 deps nuevas):**
  - `CachedEntry` pub INTACTA (Hyrum: no romper API pública slice 1).
  - `TimedEntry{entry, inserted_at: Instant, prompt: String}` interna; `lookup(&mut self)` con touch LRU + expiración lazy; `lookup_similar()` tras exact-miss en server.rs.
  - `CacheConfig += ttl_secs: u64 (0=sin expiración) + semantic_enabled: bool (false) + similarity_threshold: f32 (0.90)` — todo `#[serde(default)]`, TOML viejo compatible.
  - Similitud = coseno sobre TF de tokens normalizados (lowercase, alfanumérico); techo conocido `ponytail:` — upgrade a embeddings cuando exista embed-local (verificado: NO existe en workspace; stop-condition no dispara porque near-dups sí hitean en fixtures).
  - LRU hand-rolled (touch O(n), n≤max_entries=128 default) en vez de crate `lru` (versiones 0.16/0.18 en lock divergen; hand-roll = 0 riesgo) — `ponytail:` tag.
- **Slices:** S6 RED → S7 GREEN cache.rs+config → S8 wiring server.rs + literales → S9 verify+commit.

### Step 6: RED slice 2 — TTL/LRU/similitud (debe FALLAR: API inexistente)

- **Archivos:** `vanta-proxy/src/cache.rs` (tests), `vanta-proxy/tests/prx09_cache.rs` (+1 integración)
- **Acción:** tests `expired_fn`, `lru_touch_refreshes_recency`, `similar_hit_near_duplicate`, `similar_miss_different_prompt`, `similar_threshold_configurable`, `similar_disabled_by_default`, `extract_prompt_openai_anthropic`; integración `semantic_hit_replays_without_second_upstream_hit`
- **Verify:** `cargo test -p vanta-proxy --lib cache` falla (E0432/E0599 — API nueva inexistente = RED correcto)
- **Estado:** ✅ COMPLETED (RED confirmado: E0425 `TTL_DISABLED` + E0560 `ttl_secs`/`semantic_enabled`/`similarity_threshold`)

### Step 7: GREEN — TimedEntry + TTL + LRU + similitud + config

- **Archivos:** `vanta-proxy/src/cache.rs` (~+150L), `vanta-proxy/src/config.rs` (+3 campos + Default)
- **Acción:** mínima para pasar RED; `lookup(&mut self)`; `store` extrae prompt best-effort (sin cambio de firma); FIFO-test slice 1 sigue verde (sin lookups previos no hay touch)
- **Verify:** `cargo test -p vanta-proxy --lib` + `cargo check -p vanta-proxy`
- **Estado:** ✅ COMPLETED (GREEN: 14/14 lib-cache incl. 7 nuevos; fifo slice-1 verde; `let mut guard` en server.rs incluido)

### Step 8: Wiring server.rs + literales CacheConfig

- **Archivos:** `vanta-proxy/src/server.rs` (cache_lookup → exact + similar; `let mut guard`), `vanta-proxy/tests/prx09_cache.rs` (2 literales `..Default::default()`-safe), `vanta-proxy/src/cache.rs` tests (2 literales)
- **Acción:** `lookup_similar` tras exact-miss solo si semantic_enabled; parse-fail → skip (fail-open)
- **Verify:** `cargo test -p vanta-proxy` 0 failed (incl. regresión exact/PRX-04)
- **Estado:** ✅ COMPLETED (wiring `cache_lookup_similar` tras exact-miss; 5 literales `..Default::default()`; memory_change verde — template distingue contexto)

### Step 9: Verify full + commit + cierre

- **Archivos:** —
- **Acción:** fmt + clippy + nextest scoped; commit solo-propios en develop; lesson; update-task-state completed; plan inline (sin stagear plan — compartido Wave4)
- **Verify:** contrato Task 13 ✅
- **Estado:** ✅ COMPLETED
- **Verify real S9:** `cargo test -p vanta-proxy --tests -j 2` → 216 passed 0 failed (lib 145 incl. 12 cache + 12 suites incl. prx09 4/4 con `semantic_hit`) · `cargo fmt --check` 0 · `cargo clippy --all-targets -- -D warnings` 0 (1 fix propio `question_mark`) · sin regresión exact/PRX-04/PRX-12/tool_loop.
- **Debug S9:** `lookup(&self)`→`&mut self` exigió `let mut guard` en server.rs; `let-chains` + `as_str_mut` inexistente en edition actual → reescritura sin let-chain; `is_none_or` mueve `best` → `match &best`.
- **Scope:** 5 archivos propios (cache.rs/config.rs/server.rs/prx09_cache.rs/PRX-09.md); WIP ajeno intacto, no stageado.

## Slice 3 — Embeddings reales (PRX-09-embeddings, 2026-09-10)

- **Estado:** ⏳ IN PROGRESS
- **Contrato:** similitud por embeddings reales + test hit semántico ✅ + sin regresión exact/PRX-04 + `cargo test -p vanta-proxy` 0 failed + `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0
- **SDP:** test-driven-development (RED paráfrasis que lo léxico NO captura) · performance-optimization (medir latencia lookup embed vs léxico) · api-and-interface-design (trait `EmbedProvider` mínimo + stub offline) · doubt-driven-development (stakes: no romper hits exactos verdes) · incremental-implementation · context-engineering (+ campaign-executor/progreso/ponytail base). Descartada: frontend-ui-engineering (sin web/).
- **Impacto mapeado (Regla 0):**
  - **Archivos leídos (completos):** `vanta-proxy/src/cache.rs` (622L), `vanta-proxy/tests/prx09_cache.rs` (335L), `vanta-proxy/src/server.rs` (hook 5b/5c L424-481, SOLO lectura), `vanta-proxy/Cargo.toml` (sin ort/tokenizers), `Cargo.toml` raíz (`embed-local = dep:ort+tokenizers`, solo core), `vanta-memory/src/core/record/l1_writer.rs` (`EmbedFn`, `local_embedding_hook`, fail-open P4), `vanta-memory/src/core/record/l1_dedup.rs` (`recall_candidate_matches` + threshold configurable MEM-69).
  - **Archivos referenciados hacia dentro:** `cache.rs` → `config::CacheConfig` (sin cambios), `std::sync::Arc` (nuevo import); tests → `cache::{EmbedProvider, ...}` nuevo.
  - **Archivos que referencian a los editados:** `server.rs` (`cache_lookup_similar` ya existe — NO se edita, PRX-11-slice2 paralelo lo owns); `prx09_cache.rs` integración existente (no se toca — regresión intacta).
  - **Veredicto impacto:** bajo — aditivo puro en `cache.rs` (trait + campo `Option` + builder + test-only fake); `new()`/`store`/`lookup` firmas INTACTAS (Hyrum); `Cargo.toml`/`Cargo.lock` sin cambios (0 deps nuevas); `config.rs` sin cambios (threshold existente reutilizado); `server.rs` NO tocado.
- **Decisión A/B (evidencia):**
  - A) adapter trait + stub offline + provider HTTP Ollama opcional — ELEGIDA.
  - B) provider HTTP configurable solo — descartada como única vía (tests quedarían sin red; stub necesario igual).
  - embed-local (ort, modelo 691MB — verificado `l1_writer.rs:64-67`) — descartado para proxy: desproporcionado para cache de wire; `reqwest/blocking` YA es dep de vanta-proxy → provider Ollama `/api/embed` con 0 crates nuevos, 0 peso binario, CI offline verde (fake determinístico en tests, fail-open `None` en prod).
- **Diseño (aditivo):** `pub trait EmbedProvider: Send+Sync { fn embed(&self, text: &str) -> Option<Vec<f32>>; }` · `ExactCache::with_embedder(Arc<dyn EmbedProvider>)` (builder, `new()` intacto) · `TimedEntry += prompt_vec: Option<Vec<f32>>` (precompute en `store` vía `self.embedder`) · `lookup_similar`: embed-path primero si embedder presente (cosine f32 + mismo template-gate + mismo `similarity_threshold`), fallback léxico si embed falla/ausente · `OllamaEmbedProvider{base_url,model}` (blocking reqwest, `from_env`, fail-open) · fake semántico SOLO en `#[cfg(test)]` (buckets por keywords).
- **Deuda/resto:** wiring server-side (`with_embedder` en `AppState`) = follow-up (server.rs owned por PRX-11) → nota al orquestador para re-registrar fila PRX-09 en Backlog si aplica. `campaign_update_task_state` inservible (sin plan file: "No plan file found") → estado vive en este task file + RESULTADO.

### Step 10: RED — test paráfrasis (debe FALLAR: trait inexistente)

- **Archivos:** `vanta-proxy/src/cache.rs` (tests)
- **Acción:** fake `BucketEmbed` + tests `embed_hit_paraphrase_lexical_miss` (paráfrasis deploy/release: coseno léxico ~0.45 < 0.9 MISS, embed-coseno 1.0 HIT), `embed_miss_unrelated`, `embed_failure_falls_back_to_lexical`, `embed_latency_budget`
- **Verify:** `cargo test -p vanta-proxy --lib cache` falla (E0432/E0599 — `EmbedProvider`/`with_embedder` inexistentes = RED correcto)
- **Estado:** ✅ COMPLETED (RED confirmado: E0432 `no EmbedProvider in cache`)

### Step 11: GREEN — trait + embed-path + Ollama provider

- **Archivos:** `vanta-proxy/src/cache.rs`
- **Acción:** mínima para pasar RED; sin `unwrap`/`expect` (workspace deny); fallback léxico fail-open
- **Verify:** `cargo test -p vanta-proxy --lib cache` GREEN + exact/slice-2 tests intactos
- **Estado:** ✅ COMPLETED (GREEN: 18/18 lib-cache incl. 4 embed nuevos; Debug manual estilo `L1DedupConfig`; refactor: scan léxico extraído a `lookup_similar_lexical` sin cambio de comportamiento)

### Step 12: Integración cache + recalibración threshold + medición latencia

- **Archivos:** `vanta-proxy/src/cache.rs` (docs + threshold reuse justificado)
- **Acción:** confirmar mismo `similarity_threshold` para embed-coseno (vectores normalizados, 0.90 conservador); eprintln latencias embed vs léxico (evidencia, no assert flaky salvo budget holgado)
- **Verify:** `cargo test -p vanta-proxy --tests -j 2` 0 failed
- **Estado:** ✅ COMPLETED (lib 155 ✅ · prx09 4/4 ✅ · 13 suites ajenas al WIP paralelas ✅ · prx11_translate FALLA por WIP ajeno PRX-11-slice2 mid-slice — no mi blast radius; latencia embed 50×/128 entries = 818.7µs ≈16µs/lookup, fake en-memoria, techo documentado)

### Step 13: Verify full + commit + cierre

- **Archivos:** —
- **Acción:** fmt + clippy all-targets/all-features + nextest scoped; commit solo-propios en develop (`feat: PRX-09-embeddings ...`); lesson; RESULTADO + nota resto al orquestador
- **Verify:** contrato slice 3 ✅
- **Estado:** ✅ COMPLETED
- **Verify real S13:** lib 155 ✅ · `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0 · `cargo fmt -p vanta-proxy --check` limpio en propios (diff restante = prx11 ajeno) · prx09 4/4 + 13 suites ✅ · commit `15157d39` (solo-propios: cache.rs + PRX-09.md) · sin regresión exact/PRX-04.
- **Debug S13:** `Arc<dyn EmbedProvider>` rompe `#[derive(Debug)]` → Debug manual estilo `L1DedupConfig` (`embedder.is_some()`); `cargo test --tests` global bloqueado por WIP ajeno prx11_translate (símbolos inexistentes mid-slice) → verificado por suites individuales.
- **Scope:** 2 archivos propios; WIP ajeno intacto, no stageado.

## Notas

## Slice 4 — Wiring server-side `with_embedder` (PRX-09-wiring, 2026-09-10)

- **Estado:** ✅ COMPLETED
- **Contrato:** `OllamaEmbedProvider::from_env()` cableado en construcción del cache + gate config + test wiring (con embedder / sin embedder) ✅ + `cargo test -p vanta-proxy` 0 failed + clippy 0
- **SDP:** incremental-implementation (wiring mínimo: construcción + gate) · test-driven-development (RED E0599 → GREEN) · api-and-interface-design (gate `semantic_enabled` con serde default pre-existente → TOML legacy intacto, 0 campos nuevos) · doubt-driven-development (default-off: test explícito sin embedder) · context-engineering (+ campaign-executor/progreso/ponytail base). Descartada: frontend-ui-engineering (sin web/).
- **Impacto mapeado (Regla 0):**
  - **Archivos leídos (completos):** `vanta-proxy/src/server.rs` (`from_engine` L105-153, hook 5b/5c — SOLO lectura salvo 1 hunk), `vanta-proxy/src/cache.rs` (`with_embedder`, `OllamaEmbedProvider`, tests embed), `vanta-proxy/src/config.rs` (`CacheConfig`, `#[serde(default)]` intacto), `vanta-proxy/tests/prx09_cache.rs` (`state_for`, `state_for_semantic`).
  - **Veredicto impacto:** mínimo — 1 hunk en `from_engine` (gate + builder), 1 accessor aditivo `has_embedder()`, fix de corrección en `OllamaEmbedProvider` (worker thread), 2 tests wiring; `config.rs`/`translate.rs` NO tocados (PRX-11-slice3 corre después, sin WIP suyo en árbol — verificado `git status` limpio en `vanta-proxy/`); threshold 0.90 + TTL + FIFO/LRU intactos; fallback léxico incondicional.
- **Diseño:** gate SOLO en `config.cache.semantic_enabled` (0 campos config nuevos — scope discipline); `from_env()` no hace I/O hasta el primer `embed`; todo fallo embed → `None` → léxico (fail-open, hits solo se agregan).
- **Deuda/resto:** ninguna — wiring cierra el follow-up de slice 3.

### Step 14: RED — tests wiring (debe FALLAR: `has_embedder` inexistente)

- **Archivos:** `vanta-proxy/tests/prx09_cache.rs` (+2 tests, reusan `state_for`/`state_for_semantic`, 0 literales nuevos)
- **Acción:** `wiring_attaches_embedder_when_semantic_enabled` (URL dummy — `from_engine` sin I/O) + `wiring_no_embedder_by_default` (default-off, doubt-driven)
- **Verify:** `cargo test -p vanta-proxy --test prx09_cache wiring_ -j 2` falla (E0599 ×2 = RED correcto)
- **Estado:** ✅ COMPLETED

### Step 15: GREEN — accessor + wiring + fix worker-thread

- **Archivos:** `vanta-proxy/src/cache.rs` (`has_embedder()` + doc `with_embedder` actualizado), `vanta-proxy/src/server.rs` (gate `semantic_enabled` → `with_embedder(OllamaEmbedProvider::from_env())`)
- **Debug S15 (sistemático, no retry ciego):** GREEN inicial hizo fallar `semantic_hit_replays...` — causa raíz: `reqwest::blocking::Client` crea un tokio Runtime interno; construido + dropeado dentro del contexto async del test → panic `Cannot drop a runtime...` en teardown (en prod paniquearía al apagar el server + bloquearía el executor). Fix: cliente mudado a worker thread dedicado (creación + drop fuera de async, timeout 10s, `recv` fail-open) — `cache.rs` local, 0 deps nuevas, alternativa `ureq` descartada (churn Cargo/lock). Debug manual `OllamaEmbedProvider` (base_url/model visibles, estilo `ExactCache`) para silenciar `dead_code` bajo `-D warnings`.
- **Verify:** prx09 6/6 ✅
- **Estado:** ✅ COMPLETED

### Step 16: Verify full + commit + cierre

- **Acción:** nextest scoped `--tests -j 2` + clippy all-targets/all-features + fmt + `git status` pre-stage; commit solo-propios en develop (`feat: PRX-09-wiring ...`); lesson; RESULTADO
- **Verify real S16:** lib 155 ✅ · integración 14 suites ✅ (prx09 6/6 incl. 2 wiring nuevos; prx11_translate 16/16 — sin WIP ajeno) · `cargo clippy -p vanta-proxy --all-targets --all-features -- -D warnings` 0 · `cargo fmt -p vanta-proxy --check` 0 · sin regresión exact/PRX-04/semántico-léxico.
- **Scope:** 4 archivos propios (cache.rs/server.rs/prx09_cache.rs/PRX-09.md); WIP ajeno intacto, no stageado (`M .opencode`, `M desktop/...Cargo.lock`, `M opencode.jsonc`, `?? Investigacion-plan.md`).
- **Estado:** ✅ COMPLETED

- Slice 1 = solo exacto (stop condition plan: appetite >3d → shippear exact + DEFER semántico). Semántico slice 2.
- No re-derivar contexto del plan: gate/wave/cynefin ya verificados (plan Task 5).
- `campaign_verify_cmd` con bug exit -1 conocido → bash directa si falla.
- SDP keywords: vanta-proxy/src/cache/inject (base + lifecycle BUILD, manifest sin candidatos extra).
- **Verify real S5:** `cargo test -p vanta-proxy` 142 passed 0 failed (109 lib + 5 suites + 3 prx09 nuevos) · `cargo fmt --check` 0 · `cargo clippy --all-targets -- -D warnings` 0 · PRX-04 intacto (12 tests inject + pipeline/tool_loop verdes).
- **Debug S4:** 2º request miss → causa raíz `forward.rs` elimina `content-length` (streaming-safe) → gate de longitud imposible → fix: gate por content-type + cap 4MB en colecta (502 tipado si upstream miente longitud).
- **Self-review 3 ejes:** correctitud (clave post-inject, hit byte-idéntico, auth previo, bypass SSE/stream/errores) · simplicidad (0 deps nuevas, FIFO O(1), Mutex sync sin await) · consistencia (patrones pipeline.rs/mem_command).
- **Scope:** 10 archivos propios (2 nuevos); WIP ajeno intacto, no stageado.
