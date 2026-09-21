# MEM-63: Quick-win docs+embeddings — auto-recall doc fixed + embeddings auto-on

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Wave 20-3) + re-verificado bajo `docs/plans/2026-09-04-durability-release-readiness.md` (Task 3, Wave 0)
- **Creado:** 2026-08-30T18:45
- **last-synced:** 2026-09-05T00:00 (re-verificación plan 2026-09-04)
- **Estado:** ✅ COMPLETO (código ya en HEAD vía `6058cc84`; re-verificado 2026-09-05, cierre docs+plan+backlog+avance)

## Blast Radius
- **Callers:** `L1DedupConfig::default()` consumed by:
  - `vanta-memory/src/services/conversation_hook.rs:103` (HttpCaptureBridge::run_bridge_pass)
  - `vanta-memory/src/core/record/l1_dedup.rs` (5 internal tests)
  - `vanta-memory/tests/l1_dedup.rs` (5 integration tests)
- **Callees:** `local_embedding_hook()` (l1_writer.rs:69-76) — `#[cfg(feature = "embed-local")]` gated, deterministic 384-d fallback.
- **Doc-only:** `vanta-memory/src/core/hooks/auto_recall.rs:11-17, 62-74, 77-88` — rustdoc strings (no behavior change).
- **Implicaciones:** additive constructor + doc text edits. No new public API removal. No new dependency. No new unsafe. No storage or schema change. No `unsafe` to add.

## Contrato
1. `Select-String -Path "vanta-memory/src/core/hooks/auto_recall.rs" -Pattern "degradan hasta wirear" | Measure-Object | Select-Object Count` == 0
2. `cargo check -p vanta-memory` exit 0
3. (bonus) embeddings wired by default when `embed-local` compiled → records persist vectors out-of-the-box, not require explicit `.with_local_provider()`.

## Herramientas
- terminal (cargo), Read/Edit

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `vanta-memory/src/core/hooks/auto_recall.rs` (625L)
  - `vanta-memory/src/core/record/l1_dedup.rs` (492L)
  - `vanta-memory/src/core/record/l1_writer.rs` (lines 40-100)
  - `vanta-memory/src/services/conversation_hook.rs` (107L)
  - `vanta-memory/Cargo.toml` (65L)
  - `src/llm.rs` (834L — factory + providers)
- **Referencias hacia dentro:**
  - `L1DedupConfig::default()` → `batch_dedup()`, `recall_candidate_matches()`, `pipeline_worker` MemoryTaskHandler
  - `local_embedding_hook()` → `with_local_provider()` (already wired via cfg)
- **Referencias entrantes:**
  - `HttpCaptureBridge::run_bridge_pass` (conversation_hook.rs:103) — sole non-test call site
  - 5 unit + 5 integration tests using `default()`
- **Veredicto:** safe additive change. Changing `Default::default()` to call `with_local_provider()` semantics is one cfg-gated branch. Existing tests don't assert on `embed.is_some()` except via the `with_local_provider` chain (already covered).

## Decisiones de diseño (verificadas contra el core)
1. **Doc fix (low risk):** replace "Embedding cosine similarity. Degrades to [`RecallMode::Keyword`] when no embedding hook is supplied; with `embed-local` feature, `L1DedupConfig::with_local_provider()` wires `LocalOnnxProvider` automatically." → "Embedding cosine similarity. With the `embed-local` feature compiled **and** a working `LocalOnnxProvider` available (the auto-on path), [`L1DedupConfig::default`] wires `local_embedding_hook()` automatically; without the feature the call falls back to keyword-overlap. Either way, the public contract is preserved: records without a vector still pass the keyword gate (D38 dual-pool)."
2. **`L1DedupConfig::default()` (low risk):** when `embed-local` is compiled, wire `local_embedding_hook()`. Equivalent to a one-line `self.embed = Some(local_embedding_hook()); self` under `#[cfg(feature = "embed-local")]`. Without the feature, keep `embed: None` (current behavior).
3. **No breaking change to `with_local_provider()`** — the method stays idempotent (re-applies the hook). Pre-existing tests at l1_dedup.rs:457-484 cover this path.
4. **Test added:** `default_wires_local_provider_when_feature_on()` and `default_stays_keyword_only_without_feature()` (mirrors existing test pattern).

## FASE SECURITY
- No trust boundary crossed. `local_embedding_hook()` is gated by feature flag and uses the core factory (which is well-tested). Provider failures degrade to `None` per P4 (best-effort, never blocks).
- No new dependency. No FFI. No auth/session. No storage change.
- Checklist security-and-hardening: N/A (no input from trust boundary in this change).

## FASE PERFORMANCE
- No hot path touched. Auto-wiring happens once at config construction. Per-record embed remains O(dim) on vector writes.
- No benchmark needed (additive; no throughput or latency claim).

## Steps

### Step 1: Doc fix in `auto_recall.rs`
- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs`
- **Acción:** rewrite the module-level rustdoc (lines 11-17) and the `RecallMode` enum doc (lines 62-74) so they correctly describe the auto-on path: `embed-local` feature + provider configured → embeddings turn on by default (D38 dual-pool keyword fallback preserved); without the feature → keyword-only path (legacy).
- **Verify:** `Select-String -Path "vanta-memory/src/core/hooks/auto_recall.rs" -Pattern "degradan hasta wirear" | Measure-Object | Select-Object Count` == 0 ✅; `Select-String -Path "vanta-memory/src/core/hooks/auto_recall.rs" -Pattern "MEM-63 auto-on|wires.*automatically|auto-on" | Measure-Object | Select-Object Count` = 3 (new wording present) ✅
- **Estado:** ✅ COMPLETO

### Step 2: Auto-on in `L1DedupConfig::default()`
- **Archivos:** `vanta-memory/src/core/record/l1_dedup.rs`
- **Acción:** change `Default::default()` to set `embed: Some(local_embedding_hook())` when `embed-local` is compiled, else `embed: None` (current behavior). Update the doc on `L1DedupConfig.embed` field to mention auto-on semantics.
- **Verify:** `cargo check -p vanta-memory` exit 0 ✅
- **Estado:** ✅ COMPLETO

### Step 3: Tests for auto-on
- **Archivos:** `vanta-memory/src/core/record/l1_dedup.rs` (test module, after line 506)
- **Acción:** add `default_wires_local_provider_when_feature_on()` and `default_stays_keyword_only_without_feature()` tests (mirrors existing `with_local_provider_wires_384d_dummy_vectors` and `with_local_provider_without_feature_stays_keyword_only`).
- **Verify:** `cargo test -p vanta-memory --lib core::record::l1_dedup` → 9/9 PASS ✅
- **Estado:** ✅ COMPLETO

### Step 4: Verify full (cierre)
- **Acción:** `cargo fmt --check -p vanta-memory`, `cargo check -p vanta-memory`, `cargo clippy -p vanta-memory --all-targets -- -D warnings`, `cargo nextest run -p vanta-memory --lib -E 'test(/l1_dedup/)'`, plus contract regex check.
- **Verify:** all exit 0 ✅
  - `cargo fmt --check -p vanta-memory` → no diffs ✅
  - `cargo check -p vanta-memory` → 0 errors ✅
  - `cargo clippy -p vanta-memory --all-targets -- -D warnings` → 0 warnings ✅
  - `cargo test -p vanta-memory --lib core::record::l1_dedup` → 9/9 passed, 0 failed ✅
  - contract regex `Select-String -Path auto_recall.rs -Pattern "degradan hasta wirear" | Measure-Object Count` = 0 ✅
- **Estado:** ✅ COMPLETO

## Notas
- **vanta-docs no hace commit** (regla de rol). El commit `docs: MEM-63 — Auto-recall doc fixed + embeddings auto-on` queda staged para vanta-lead integrar en su próximo PR.
- El pre-commit hook (`.githooks/pre-commit`) corre `cargo fmt --all -- --check` contra TODO el working tree, y falla por `src/cli_handlers/export_md.rs` + `src/cli_handlers/mod.rs` (trabajo en progreso de otra sesión, NO blast radius de MEM-63). Por regla de rol vanta-docs no puede `--no-verify` el commit.
- Pre-mortem mitigations:
  - Fallo 1 (doc fixed sin verificar) → grep post-fix en Step 1 verify ✅.
  - Fallo 2 (embeddings auto-on requiere feature flag) → cfg-gated `#[cfg(feature = "embed-local")]` en `default()`; tests cubren ambos branches.

## Context Save Point
- **Fecha:** 2026-08-30T18:45
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** auto-on vía cfg-gated default; sin nueva API pública.
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** MEM-63 cierre (commit `docs: MEM-63 — Auto-recall doc fixed + embeddings auto-on`).

## Re-verificación 2026-09-05 (plan 2026-09-04-durability-release-readiness, Task 3 Wave 0)
- **SDP:** `campaign_discover_skills` BUILD → campaign-executor, progreso, ponytail, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering. Cargadas (rol vanta-docs): documentation-and-adrs, writing-guidelines, test-driven-development, writing-plans, progreso. `SKILLS_CARGADAS: documentation-and-adrs, writing-guidelines, test-driven-development, writing-plans, progreso, campaign-executor, ponytail`
- **Hallazgo:** el trabajo MEM-63 ya está en HEAD pero entró bajo commit `6058cc84` (`docs: FIND-41 — ADR clusters...`) cuyo mensaje dice "Sin codigo fuente modificado" — falso, el commit tocó `auto_recall.rs` + `l1_dedup.rs` (47+24L). Deuda de trazabilidad, no de código: no se re-escribe historia, se documenta aquí.
- **Contrato 2026-09-04 (verificado, sin edits de fuente necesarias):**
  1. `grep "degradan hasta wirear" vanta-memory/` → 0 matches ✅
  2. `cargo test -p vanta-memory --lib` → 328 passed, 0 failed ✅
  3. `cargo test -p vanta-memory --lib core::record::l1_dedup` (default, sin feature) → 9 passed incl. `default_stays_keyword_only_without_feature` (chars/keyword-fallback solo sin provider) ✅
  4. `cargo test -p vanta-memory --features embed-local --lib core::record::l1_dedup` → 9 passed incl. `default_wires_local_provider_when_feature_on` (auto-on con provider, 384-d) ✅
  5. `cargo fmt --check -p vanta-memory` → limpio ✅; `cargo clippy -p vanta-memory --all-targets` → 0 warnings ✅; `cargo test -p vanta-memory --doc` → 1 passed ✅; `cargo doc --no-deps -p vanta-memory` → exit 0 (60 warnings pre-existentes `rustdoc::redundant_explicit_links`, deuda no bloqueante) ✅
- **Deuda:** `vanta-memory` sin `#![deny/warn(missing_docs)]` (grep 0) — enforcement manual vía review; `cargo doc` 60 warnings pre-existentes.
- **Cierre:** sin diff de fuente (ya en HEAD); este commit solo registra plan+task+backlog+avance con mensaje `docs(memory): auto_recall doc + auto-on embeddings (MEM-63)`.