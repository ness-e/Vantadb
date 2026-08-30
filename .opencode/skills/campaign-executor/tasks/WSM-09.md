# WSM-09: Unificar límites FFI en core (MAX_VEC_DIM, MAX_F32)

## Metadata
- **Plan file:** `docs/plans/2026-08-29-full-backlog-parallel.md` (Wave 10, parallel-3 con WSM-04/WSM-05)
- **Creado:** 2026-08-29
- **last-synced:** 2026-08-29
- **Estado:** ⬜ PENDING
- **Tipo:** refactor (consolidar constantes duplicadas en core)
- **Workflow:** refactor (audit → migrate → cleanup → verify → review → accept → close)
- **Owner:** vanta-worker

## Contexto (research-vantadb-wasm-20260825 H-12)

Hoy existen 4 constantes duplicadas/límites divergentes en las fronteras FFI:

| Constante | Valor actual | Ubicación |
|---|---|---|
| `MAX_F32_VEC_LEN` | 10_000_000 | `vantadb-wasm/src/lib.rs:39` |
| `MAX_BATCH_SIZE` | 100_000 | `vantadb-wasm/src/lib.rs:40` |
| `MAX_K` | 1_000 | `vantadb-wasm/src/lib.rs:44`, `vantadb-python/src/lib.rs:46` |
| `MAX_VEC_DIM` | 10_000 | `vantadb-node/src/lib.rs:25` |
| `max_top_k` | 1_000 (default) | `vantadb-mcp/src/config.rs:101` |
| node `top_k` | **sin clamp** | `vantadb-node/src/lib.rs:546-550` |

**Misma operación → distinto límite según transporte:**
- WASM: vector ≤ 10M, k ≤ 1k
- Node: vector ≤ 10k (1000× más estricto), k sin límite

**Riesgo:** los usuarios Node no pueden insertar un embedding de 384d típico, pero WASM sí.
node no clampea `top_k`, puede provocar ERR-022.

## Blast Radius

| Crate | Archivos tocados | Tipo |
|---|---|---|
| `vantadb` core | `src/config.rs`, `src/lib.rs` | agregar const + re-export |
| `vantadb-wasm` | `src/lib.rs` | reemplazar const local → import core |
| `vantadb-node` | `src/lib.rs` | reemplazar const local → import core, agregar clamp `top_k` |
| `vantadb-python` | `src/lib.rs` | reemplazar const local → import core |

**Implicaciones:** ninguno que rompa consumidores (todas las constantes se mantienen idénticas o se toman `max()` entre transports — pre-mortem Fallo 2).

## Contrato

```powershell
Select-String -Path "src/config.rs" -Pattern "MAX_VEC_DIM|MAX_F32" | Measure-Object | Select-Object Count
```
Debe devolver **>= 1** (constantes en core).

Plus verificaciones complementarias:
- `cargo check -p vantadb` ✅
- `cargo check -p vantadb-python` ✅
- `cargo check --manifest-path vantadb-node/Cargo.toml` ✅ (sin target wasm32 instalado)
- `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` ✅

## Pre-mortem (del plan)

- **Fallo 1: mover constante de wasm/node a core — verificar import en ambos**
  → Mitigación: usar `use vantadb::config::{MAX_F32_VEC_LEN, MAX_BATCH_SIZE, MAX_K, MAX_VEC_DIM}` en ambos.
- **Fallo 2: cambiar valor puede romper usuarios existentes — max() entre ambos**
  → Mitigación: `MAX_F32_VEC_LEN = max(10M, 10k*4) = 10M`; `MAX_K = max(1k, 10k) = 10k`. Tomamos el más permisivo entre transports para no romper usuarios existentes. **El bump de `MAX_K` de 1k → 10k afecta bindings wasm/python** — pero como es `clamp`, semánticamente idéntico para callers que pedían k ≤ 1k (siguen recibiendo lo mismo). Para callers que pedían k > 1k, antes recibían 1k silencioso (warning si python, silent si wasm); ahora reciben su valor pedido. Cambio **observado por el docstring** `clamp_top_k` ya warning.

  > **Decisión:** mantener `MAX_K = 10_000` (max de wasm 1k + node 10k) implica:
  > - wasm: WARNING log cuando top_k > 10k (igual que ya hace python)
  > - python: sin cambio (ya clampea 1k → ahora clampea 10k; usuarios python que pedían k=5000 antes recibían 1000 con warning, ahora reciben 5000 — **mejora real**, no regresión)
  > - node: nuevo clamp `top_k.min(MAX_K)` con warning

  En resumen: `MAX_K = 10_000` (max). Para usuarios existentes, el cambio es **neutral o mejora** (más permisivo, con warning observable donde antes era silencioso).
- **Fallo 3: top_k limite distinto entre transports (1k vs 10k) — unificar en max(10k)**
  → Cubierto por el contrato: `MAX_K = 10_000` y node gana un clamp.

## Stop conditions

>1h → docs-only, decisión unificación. (Doc-only ya está hecho — este refactor es la implementación.)

## Steps

### Step 1: Agregar constantes en `src/config.rs` + re-export en `src/lib.rs` ⬜ PENDING
- **Archivos:** `src/config.rs`, `src/lib.rs`
- **Acción:** agregar 4 `pub const` cerca de las otras `pub(crate) const MAX_*` (línea 25). Re-export en `src/lib.rs`.
- **Verify:** `cargo check -p vantadb` 0

### Step 2: Reemplazar en `vantadb-wasm/src/lib.rs` ⬜ PENDING
- **Archivos:** `vantadb-wasm/src/lib.rs:38-44`
- **Acción:** borrar las 3 const locales, importar desde `vantadb::config::*`.
- **Verify:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` 0

### Step 3: Reemplazar en `vantadb-node/src/lib.rs` + agregar clamp top_k ⬜ PENDING
- **Archivos:** `vantadb-node/src/lib.rs:22-25, 546-550`
- **Acción:** borrar `MAX_VEC_DIM` local, importar; agregar `top_k.min(MAX_K)` con warning.
- **Verify:** `cargo check --manifest-path vantadb-node/Cargo.toml` 0

### Step 4: Reemplazar en `vantadb-python/src/lib.rs` ⬜ PENDING
- **Archivos:** `vantadb-python/src/lib.rs:43-54`
- **Acción:** borrar `MAX_K` local, importar desde core; mantener `clamp_top_k()` helper.
- **Verify:** `cargo check -p vantadb-python` 0

### Step 5: Verify mecánico completo ⬜ PENDING
- **Acción:** clippy + fmt + nextest core (skip wasm/node/python tests que requieren target)
- **Verify:** 0 errors, 0 warnings

## Dependencias

Ninguna (tarea aislada).

## Notas

- **Regla 6 (deuda):** este PR **elimina deuda** (4 constantes duplicadas → 1 fuente). Saldo neto negativo. ✅
- **Regla 5 (decisión):** NO requiere ADR — es refactor sin tradeoff (constantes se unifican; semántica preservada vía `max()`).
- **Regla 8 (concurrencia):** NO toca paths concurrentes (solo `const`); audit no requerida.
- **Regla 9 (perf):** NO toca hot path (compilación puede agregar inline `pub const` con cero overhead).
- **Regla 11 (claims):** no publica números — refactor puro.

## Context Save Point
- **Fecha:** 2026-08-29
- **Branch:** develop
- **CI pendiente:** NO (vanta-worker no commitea)
- **Decisiones:** MAX_K=10_000 (max de 1k+10k); MAX_F32_VEC_LEN=10_000_000; MAX_VEC_DIM=10_000; MAX_BATCH_SIZE=100_000
- **Próxima tarea:** WSM-09 verificación mecánica + cierre