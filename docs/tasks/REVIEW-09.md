# REVIEW-09: Bug lógico cache_warmer — latch `saturated` monotónico mata el aprendizaje

## Metadata
- **Plan file:** docs/plans/2026-08-23-backlog-triage.md (NO editar — Task 7)
- **Fuente:** plan Task 7 · review-full-20260822 H09-CODE-001
- **Esfuerzo:** 🟢 · **Prioridad:** 🟠 · **Tipo:** Rust core
- **Turns estimados:** 5-10
- **Creado:** 2026-08-23T00:00
- **last-synced:** 2026-08-23T01:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 (mecanismo confirmado en DISCOVERY)
- **Pendientes (downhill):** ver Steps

## Impacto mapeado (Regla 0)

> GATE ANTES DE CUALQUIER EDICIÓN — poblado en DISCOVERY.

- **Archivos leídos (completos):** `src/cache_warmer.rs` (1–414: header/imports 1–29, struct+record_co_access 30–142 vía codegraph verbatim, decay/suggest/metrics/clear 140–263, tests 264–414)
- **Referencias hacia dentro:** std sync atomics/RwLock/HashMap; `crate::metrics::record_cache_warmer_metrics`; `crate::index::CPIndex` (solo `hnsw_top_layer_ids`, fuera del cambio)
- **Referencias entrantes (`saturated`:)** escrituras en `:144` (set) y `:254` (`clear()`); lectura única en `:117`. Callers externos de `CacheWarmer`: `src/storage/engine/get.rs` (vía `suggest_warm_ids`/`prefetch_related`) — no tocan `decay()` ni `saturated`. `pair_count` reconciliado solo en `decay():200`.
- **Veredicto impacto:** **bajo** — fix confinado a 1 archivo, sin API pública (`CacheWarmer` es `pub(crate)`), sin cambios de formato/persistencia. Comportamiento nuevo: el latch puede bajar; el cap de memoria sigue garantizado (satura al volver a cruzar `max_pairs`).

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)

- **Repro:** unit test determinístico: cap=2 → `[1,2]`+`[3,4]` satura → `[9,10]` no se aprende (refresh-only) → `decay()` elimina ambos pares (counts 1→0) → tabla queda 0 << max_pairs pero `saturated` sigue `true`.
- **Hipótesis (causa raíz):** el latch es set-once — `record_co_access:143-145` hace `saturated.store(true)` cuando `pair_count >= max_pairs` pero NADIE lo resetea; `decay()` (:189–201) reduce la tabla y reconcilia `pair_count` (:199–200) sin tocar `saturated`. En servers long-running, ciclos de decay vacían la tabla y el warmer queda en modo refresh-only para siempre.
- **1 variable controlada:** agregar reset condicional del latch SOLO dentro de `decay()` cuando `post-decay total < max_pairs`. Ningún otro cambio.
- **Test RED:** `test_decay_below_cap_resets_saturation_and_learning_resumes` — verificado FALLANDO antes del fix (assert `!saturated` tras decay bajo cap).

## Contrato

"test: simular ciclos — warm hasta saturar → decay reduce tabla bajo `max_pairs` → latch reseteado cuando post-decay total < max_pairs → warmer VUELVE a aprender pares nuevos y re-satura al cruzar el umbral. Sin thrashing: decays que dejan post-total ≥ cap NO resetean (una sola transición por cruce). `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo nextest run -p vantadb cache_warmer` verde."

## Invariantes de dominio (handoff — MUST)

- **Invariantes:** (1) bound de memoria intacto — nunca aprender por encima del cap sin pasar por saturación; (2) una sola transición de latch por cruce de umbral (reset solo en `decay()`, cada ≥1000 eventos); (3) `clear()` sigue reseteando todo; (4) sin unwraps nuevos; (5) sin API pública nueva/modificada.
- **Comandos de verificación:** `cargo nextest run -p vantadb cache_warmer` (módulo completo) · verify full: fmt + clippy workspace + nextest audit workspace.
- **Deuda pendiente:** ninguna.

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — N/A: lógica interna de contadores, sin trust boundaries, input, auth, ni dependencias nuevas.
- [x] **PERFORMANCE** — N/A como optimización: el único costo añadido es un `AtomicBool::store` condicional dentro de `decay()` (corre ≤1× por 1000 eventos). Sin claim de perf → Regla 9 no aplica. Cap de memoria preservado.

## Deuda técnica (Regla 6)

Sin deuda nueva (saldo neto 0).

## Steps

### Step 1: DISCOVERY + task file
- **Archivos:** este task file
- **Acción:** leer código, confirmar mecanismo latch set-once + decay sin reset, mapear blast radius
- **Verify:** mecanismo citado con líneas exactas (:143–145, :189–201)
- **Estado:** ✅ DONE

### Step 2: RED — tests de regresión que reproducen el bug
- **Archivos:** `src/cache_warmer.rs` (mod tests)
- **Acción:** agregar `test_decay_below_cap_resets_saturation_and_learning_resumes` (ciclo completo: saturar → decay bajo cap → reset → reaprender → re-saturar) y `test_no_thrash_latch_persists_while_post_decay_total_at_cap` (histéresis mecánica: decays con post-total ≥ cap mantienen latch; reset único al caer bajo cap)
- **Verify:** `cargo nextest run -p vantadb cache_warmer` → los 2 tests nuevos FALLAN (RED confirma el bug)
- **Estado:** ✅ DONE (RED verificado: fallo en :426 y :468; 9 existentes PASS)

### Step 3: GREEN — reset del latch en `decay()`
- **Archivos:** `src/cache_warmer.rs:186-201` (+ docstring :104-106)
- **Acción:** en `decay()`, tras reconciliar `pair_count`: `if total < self.max_pairs { self.saturated.store(false, Relaxed) }`; actualizar doc comments para documentar que el decay levanta el cap y el aprendizaje se reanuda
- **Verify:** `cargo nextest run -p vantadb cache_warmer` → suite módulo completa PASS
- **Estado:** ✅ DONE (11/11 PASS)

### Step 4: Verify full + commit
- **Acción:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2` (vía campaign_verify_cmd) → commit conventional
- **Verify:** los 3 comandos exit 0
- **Estado:** ✅ DONE (fmt ✅ · clippy ✅ · nextest audit workspace 2714/2714 ✅ · commit `8b8924b3` con pre-commit hooks verdes)

### Step 5: Gate P2-01 review + hallazgo de docs
- **Acción:** review adversarial por vanta-review (contexto fresco) → ❌ changes-required por doc deshonesta del campo `saturated` (:49-50 decía "monotonic"); corregida a comportamiento post-fix; re-verificado contrato (fmt + módulo 11/11)
- **Verify:** re-run `cargo nextest run -p vantadb cache_warmer` = 11/11 PASS tras la corrección
- **Estado:** ✅ DONE

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sub-agente `ses_fcf48fec6ffeAuUiOahukhpCnV`, contexto fresco)
- **Enfoque:** approve-condicional — reset en `decay()` es el lugar correcto (0 callers externos de `decay()`); bound AUDIT-04 intacto (re-satura al cruzar umbral); histéresis extra = YAGNI (transiciones ≤1×/1000 eventos). Hallazgo 🔴: doc del campo `saturated` (:49-51) seguía diciendo "monotonic" — vector de regresión. Corregido.
- **Cómo se probó:** el reviewer re-ejecutó independientemente `cargo fmt --check` ✅ y `cargo nextest run -p vantadb cache_warmer` = 11/11 PASS. Verificó anti-vacuo de tests y semántica de borde (`total == cap` mantiene latch vía `<` estricto).
- **Checklist anti-hábitos tóxicos:** ✅ sin unwraps nuevos, sin API pública cambiada, sin degradación de chequeo de errores, evidencia reproducida por tercer contexto.
- **Veredicto:** ✅ approve (tras corrección del hallazgo 1; hallazgos 🟡/🟢 registrados como notas, sin bloqueo)

## Notas
- Histéresis: no se necesita banda extra — el reset solo ocurre dentro de `decay()` (≤1× por 1000 eventos), así que la cadencia natural amortigua cualquier oscilación. La condición `total < max_pairs` da exactamente una transición por cruce.
- Overshoot intra-call pre-existente (un solo `record_co_access` con N ids puede insertar N(N-1)/2 pares antes del check) se mantiene igual que hoy — fuera de scope.
- **Review P2-01 (hallazgo 🟡, no bloqueante):** derivar saturación de `pair_count.load() >= max_pairs` en vez del `AtomicBool` eliminaría el estado duplicado que habilitó esta clase de bug — candidato a refactor futuro/Backlog.
- `campaign_verify_cmd` (MCP) inusable en esta sesión: schema sin parámetro `planFile` + server con 2 planes activos en 24h → "Ambiguous active plan". Verify full ejecutado con bash directo (mismos comandos, exit codes reales).
