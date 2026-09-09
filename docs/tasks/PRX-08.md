# PRX-08 — Higiene y ceilings documentados

- **Plan:** `docs/plans/2026-09-08-backlog.md` (Task 8, Wave1)
- **Appetite:** max 1d · **Esfuerzo:** 🟢 4-6h · **Wave:** Wave1 (disjunto BND-08 ✅ df1baa2e / MEM-66)
- **Branch:** develop · **Commit:** conventional con task ID
- **Contrato:** `cargo test -p vanta-proxy` 0 failed + `cargo clippy -p vanta-proxy -- -D warnings` 0 + cada sub-item con antes/después en notas
- **SDP:** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (frontend-ui-engineering descartada: sin web/ en scope)
- **Tipo:** proxy/refactor (higiene, sin cambio de protocolo, sin símbolos `pub` nuevos salvo 1 método validate puro)
- **Estado:** ⏳ IN PROGRESS

## Gate D

Blast radius 6 archivos en 1 crate; refactor higiene sin símbolos públicos nuevos (solo `pub(crate)` + 1 `pub fn` puro de validación, sin cambio de wire). Contrato mecánico del plan. → Gate D no disparado, sin `question`.

## Spec (6 sub-items de Backlog.md:517)

| # | Sub-item | Archivo | Antes | Después |
|---|----------|---------|-------|---------|
| S1 | auth O(1) HashMap | `auth.rs` | `resolve_user_key` scan lineal `entity_list` 10k/request | índice cache-aside `HashMap<user_key, UserIdentity>`; scan 1 vez, luego O(1) |
| S2 | upstream no autorreferencial | `config.rs` + hook `server.rs::from_engine` | default `http://127.0.0.1:8096` = self-loop silencioso (puerto = DEFAULT_PORT) | `points_at_self(port)` + fail-fast `Config` en `from_engine` |
| S3 | LRU evict sesiones | `session.rs` | `HashMap` unbounded (`ponytail:` ceiling comentado) | cap `MAX_SESSIONS=10_000`, evict oldest `updated_at_ms` |
| S4 | evict buckets rate-limit | `rate_limit.rs` | `buckets: HashMap` unbounded | cap `MAX_BUCKETS=10_000`, evict oldest-front + prune expirados |
| S5 | writeback incremental | `writeback.rs` | `persist()` full-file rewrite por fallo (`ponytail:` ceiling) | append-only JSONL por enqueue; rewrite solo en flush con resto |
| S6 | tools mixtas sin colgados | `memory_tools.rs` + `server.rs` loop | `append_exchange` reenvía ALL tool_calls con results solo nuestros → 400 upstream / colgados | filtra assistant a calls ejecutadas + `warn` de dropeadas; replay verbatim si 0 nuestras |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `auth.rs` (212L), `session.rs` (440L), `rate_limit.rs` (421L), `writeback.rs` (324L), `config.rs` (149L), `memory_tools.rs` (369L), `forward.rs` (154L), `server.rs` §1-120 + §300-412, `lib.rs`.
- **Hacia dentro (usan mis archivos):** `server.rs` usa los 6 (AuthDb::open/new, SessionStore::new, RateLimiter::new, WriteBack::new, memory_tools::{announces,extract,execute,append_exchange}, config); tests `pipeline/prx01/proxy_wire/tool_loop` construyen `ProxyConfig` explícito (mock URL puerto random); `desktop/.../ProxyDashboard.tsx` lee `SessionSnapshot` wire (solo S3 aditivo, wire intacto).
- **Hacia fuera (mis archivos usan):** `vantadb::{entity,storage,config,sdk}`, `vanta-memory::core::hooks`, axum/reqwest/serde_json/tokio. Sin tocar.
- **Entrantes tests:** `store.sessions.lock()` directo en session tests (no cambio tipo campo); `WriteBack::new/pending_count/pending_labels/flush` usados en capture/memory_tools tests (firmas intactas); `RateLimiter::new/check` (firmas intactas); `resolve_user_key/authenticate` (firmas intactas + `invalidate()` nuevo).
- **Veredicto:** impacto LOCAL por slice, firmas públicas intactas, wire intacto. Riesgo mayor: S1 orden (first-wins preservado con `or_insert`) + S2 default-config ahora falla `from_engine` (intencional, documentado). Sin `lru` crate nueva (stdlib + evict manual; evita riesgo registry/AUD-042).

## Steps (1 slice = 1 sub-item, ~100 líneas, check por slice)

- [x] S1 auth O(1): RED test + GREEN índice cache-aside + `invalidate()` → `cargo test -p vanta-proxy auth::` 4/4 ✅
- [x] S2 self-loop: RED tests `points_at_self` + GREEN + hook `from_engine` → config:: 3/3 ✅
- [x] S3 session cap: RED test evict + GREEN `MAX_SESSIONS` + evict en ensure/advance → session:: 22/22 ✅
- [x] S4 bucket cap: RED test + GREEN evict en `check` → rate_limit:: 14/14 ✅
- [x] S5 append-only: RED test (2 líneas) + GREEN `persist_append` + flush rewrite → writeback:: 7/7 ✅
- [x] S6 mixed tools: RED test + GREEN filtro en `append_exchange` (ambos protocolos) → memory_tools:: 7/7 ✅
- [x] Verify full contrato + commit solo paths propios + recitation

## Baseline (antes, 2026-09-09)

- `cargo test -p vanta-proxy`: 86 lib + 5 pipeline + 10 proxy_wire + 5 prx01 + 5 tool_loop = **111 passed / 0 failed** ✅
- `cargo clippy -p vanta-proxy -- -D warnings`: ❌ **bloqueado por ajeno** — `vanta-memory` lib `field pending is never read` (WIP MEM-66 sin commitear, `local_backend.rs` +3). Pre-existente, fuera de scope, NO tocar.
- Deuda ajena en worktree (NO stagear): `M .opencode`, `M opencode.jsonc`, `M vanta-memory/...`, `M vanta-memory/tests/...`, `?? Investigacion-plan.md`, `?? docs/plans/2026-09-08-backlog.md`, `?? docs/tasks/MEM-66.md`.

## Notas (antes/después por sub-item — completar al cerrar)

- **S1 auth O(1)** (`auth.rs`): antes `resolve_user_key` scan `entity_list`(10k)/request con `ct_eq` por candidato; después snapshot `HashMap` en primer miss + O(1) warm, first-wins preservado, `invalidate()` explícito (proxy nunca escribe `user` → exacto en steady state). `ct_eq` + su test eliminados (timing-oracle void con cache; HashMap no filtra por posición). Ceiling: snapshot per-process — escritores externos requieren `invalidate`/restart.
- **S2 self-loop** (`config.rs`, `server.rs`): antes default `http://127.0.0.1:8096` == `DEFAULT_PORT` → loop silencioso hasta timeout 600s; después `UpstreamConfig::points_at_self(port)` (loopback+port, puro) + fail-fast `Config` en `from_engine`. Tests existentes usan mock URL puerto random → intactos. Ceiling: solo detecta loopback+port exacto (un alias DNS al propio host no se detecta).
- **S3 session cap** (`session.rs`): antes `HashMap` unbounded (`ponytail:`); después `MAX_SESSIONS=10_000` + evict oldest-`updated_at_ms` post-insert en ensure/advance (scan O(n) solo al overflow). Wire `SessionSnapshot` intacto.
- **S4 bucket cap** (`rate_limit.rs`): antes `buckets` unbounded; después `MAX_BUCKETS=10_000` + evict (expirados/vacíos primero, luego oldest-front) pre-insert. Sin `lru` crate nueva (stdlib; evita riesgo registry AUD-042).
- **S5 append-only** (`writeback.rs`): antes `persist()` rewrite full-doc `{"pending":[...]}` por fallo; después `persist_append()` 1 línea JSONL `{"label":...}` por enqueue + `persist_rewrite` compactado solo en flush-con-resto. Formato viejo sin lector (audit-only) → seguro. Ceiling: sin fsync por línea (best-effort, como antes).
- **S6 mixed tools** (`memory_tools.rs`): antes echo ALL tool_calls + results solo nuestros → 400 upstream/colgados; después echo solo ejecutadas (OpenAI + Anthropic blocks) + `warn` por dropeada. `server.rs` sin cambio (0 nuestras → replay verbatim ya existía). Ceiling: client tools se dropean con warn (servir ambas requiere cambio protocolo, fuera de scope).
- **Verify final (2026-09-09):** test 118/118 (93 lib + 5+10+5+5) ✅ · fmt ✅ · check ✅ · clippy contrato ❌ **bloqueado ajeno**: `vanta-memory` lib `field pending is never read` (WIP MEM-66 sin commitear) + `campaign_verify_cmd` bug exit -1 (stdout vacío) → bash directa como fallback.
