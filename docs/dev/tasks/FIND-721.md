---
title: "FIND-721: `clippy --target wasm32` falla (`drop_non_drop`) y ninguna lane lo ejerce"
kind: task
status: active
description: "Arreglar los 3 lints que rompen `clippy -p vantadb-wasm --target wasm32` (dos `drop_non_drop` y un `expect_used`) y proponer la lane de CI que los ejerce"
---

# FIND-721: `clippy --target wasm32` falla (`drop_non_drop`) y ninguna lane lo ejerce

## Metadata
- **Plan file:** docs/dev/plans/2026-11-23-release-v0.10.0-gates.md (§6.b)
- **Fuente:** plan file §6.b — fila `FIND-721`
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Rust (core + utils) + Mixto (task file)
- **Turns estimados:** 6
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T12:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas (la contradicción contrato/scope se resolvió por Gate D: "arreglar los 3")
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `write_redacted_segment` ← `WalWriter::redact_erased` (src/wal.rs:1387) ← `wal_sharded.rs:801` (`compact_wal`); `write_atomic_with_tmp` (src/utils/fs.rs:50) ← `write_atomic`/`write_atomic_unique` (snapshot + vfile restore); `insert.rs:1091` es el rollback de `batch_insert_with_opts` |
| Callees | `std::fs::File` (host: handle real; wasm32: tipo sin `Drop`), `parking_lot::FairMutex::try_lock`, `swap_segment` → `sync_parent_dir` |
| Implicaciones | Sin cambio de contrato público ni de formato wire. En host el `drop(file)` **se conserva** (Windows rechaza el `rename`/`remove_file` con el handle abierto). En wasm32 el `File` no tiene descriptor: dejarlo caer al final del scope es equivalente. `insert.rs`: el `try_lock()` de wasm32 pasa de `expect` (panic, prohibido por `lint clippy::expect_used = "deny"`) a `Option` vivo — mismo guard cuando se adquiere, sin panic en el path de rollback. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/utils/fs.rs` (182 L, completo). `src/wal.rs`: `write_redacted_segment` 1265-1367 + `swap_segment` 1369-1375 + `WalWriter::redact_erased` 1377-1438 + relación con `wal_sharded.rs:801` — extraídos verbatim vía `codegraph_codegraph_explore`; el resto del archivo (3766 L) es lectura WAL no relacionada y no se edita.
- **Archivos referenciados hacia dentro:** `src/wal.rs` → `crate::utils::fs::sync_parent_dir`; `src/utils/fs.rs` → `web_time`, `std::fs`; `src/storage/engine/insert.rs:1101` → `self.insert_lock` (`FairMutex<()>`, `FairMutexGuard`), `self.acquire_insert_lock` (misma semántica `try_lock` en wasm32, `mod.rs:1490`).
- **Archivos que referencian a los editados:** `src/wal_sharded.rs` (`redact_erased`), `src/wal_shipping.rs`, `src/lib.rs` (`WalWriter`); `write_atomic*` desde snapshot/vfile; `insert.rs` desde `Engine::put`/`put_batch`. Ninguno depende de que exista una sentencia `drop(...)` concreta.
- **Veredicto impacto:** **bajo**. Sentencias `drop` y un `expect` sobre un `Option` local; ninguna firma, tipo ni garantía de durabilidad cambia. Riesgo real = que el `cfg` erre el target (pre-mortem a) → se verifica host **y** wasm32.

## Contrato

```
cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings   # exit 0
cargo check  -p vantadb-wasm --target wasm32-unknown-unknown                  # exit 0
cargo check  -p vantadb                                                       # exit 0 (host intacto)
```
`rustup target list --installed` incluye `wasm32-unknown-unknown` (verificado).

## Spec (SDD)

No es feature-add: no agrega símbolos públicos, endpoints, tools ni métodos de binding. Decisión única (Gate D, respondida por el owner):

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Alcance del fix para que el contrato (exit 0) pase | A) los 3 errores (contrato verde, 2 archivos fuera de §2) / B) solo `wal.rs` (contrato rojo, 2 FIND) / C) `wal.rs`+`fs.rs` (rojo por 1) | A | ✅ pregunta Gate D → **A** |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) host (Windows) debe seguir cerrando el handle antes de `rename`/`remove_file` en `write_redacted_segment` y `write_atomic_with_tmp`; (2) el rollback de `batch_insert_with_opts` sigue corriendo bajo `insert_lock` cuando el lock está libre; (3) ningún `#[allow(clippy::…)]` tapa el lint.
- **Comandos de verificación:** los 3 del §Contrato + `cargo nextest run -p vantadb -E 'test(write_atomic)'` + `cargo nextest run -p vantadb -E 'test(redact)'`.
- **Deuda pendiente:** lane de CI (ver §Deuda).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado |
| `lastAction` | Último step ✅ |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | Próximo step ⬜ |
| `contract` | §Contrato + §Invariantes + evidencia |
| `nextTask` | siguiente fila del plan (§6.b) |

## Deuda técnica (Regla 6 — MUST)

**Sin deuda.** El cambio **reduce** deuda: elimina 3 violaciones de lint que ninguna lane veía.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| Task | contrato (3 comandos exit 0) + tests `redact`/`write_atomic` verdes |
| Commit | atómico, `fix(wal): …`, solo los 3 archivos + este task file |
| Release | N/A (sin cambio de API ni de wire) |

## Herramientas necesarias
- `cargo clippy/check --target wasm32-unknown-unknown`, `cargo nextest -p vantadb`
- `codegraph_codegraph_explore` (blast radius de `write_redacted_segment` / `write_atomic_with_tmp`)
- `pwsh dev-tools/ocr-review.ps1` (cierre)

**Skills cargadas (SDP):** `campaign_discover_skills_v2(phase=BUILD, taskType=bug-fix, keywords=[wasm32, clippy, drop, cfg target])` → `incremental-implementation` (slice único, repo siempre compilable), `doubt-driven-development` (revisión adversarial de un `cfg` en durabilidad), `source-driven-development` (semántica de `Drop`/`File` en wasm32 verificada contra la doc del lint), `security-and-hardening` + `deprecation-and-migration` (policy pins de SDP v3; el segundo es match por keyword "storage/schema" y no aplica — se registra el falso positivo), `campaign-executor`/`progreso`/`ponytail` ya activos.

## Investigation Notes

- **`clippy::drop_non_drop`** (rust-clippy): dispara cuando `needs_drop::<T>() == false`; el `drop` "solo extiende lifetimes contenidos". En `wasm32-unknown-unknown` el `std::fs::File` de std es el stub `unsupported::fs` — **no tiene impl de `Drop`** (no hay fd que cerrar), por eso el lint lo marca ahí y no en host.
  Fuente: <https://rust-lang.github.io/rust-clippy/master/index.html#drop_non_drop> · <https://doc.rust-lang.org/std/fs/struct.File.html> (plataformas soportadas).
- El `drop(file)` es **funcional en host**: el comentario de `redact_erased` (src/wal.rs:1409-1412) documenta que el `rename` requiere el destino cerrado (Windows rechaza reemplazar un archivo abierto), y `write_atomic_with_tmp` hace `rename` justo después del `drop`. Por eso se `cfg`-gatea, no se borra.
- `insert.rs:1101`: `try_lock()` devuelve `Option<FairMutexGuard>` (ver `mod.rs:1498`). El `expect` viola `workspace.lints.clippy.expect_used = "deny"` (Cargo.toml:778); la rama solo existe en `cfg(target_arch = "wasm32")`, por eso `Clippy Lints` (host) nunca la vio.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 4 |
| % completado | 0% |

## Fase 1 — Evidencia de Debugging (GATE — solo tipo Bug)

- **Repro:** `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` → `error: could not compile `vantadb` (lib) due to 3 previous errors`. Salida completa: `C:\Users\Eros\AppData\Local\Temp\opencode\find721-clippy.txt`.
- **Hipótesis:** el target wasm32 nunca pasó por clippy (no existe lane); los 3 lints (`drop_non_drop` ×2, `expect_used` ×1) son de código `cfg`-exclusivo de wasm32 o de `drop` sobre un tipo sin `Drop` en ese target.
- **1 variable controlada:** el `cfg`/binding del `drop` y del `try_lock`; nada más.
- **Test RED:** el propio comando de clippy (exit 1, 3 errores) — verificado ANTES del fix. Regresión estructural = la lane de CI (§Deuda).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — toca durabilidad (WAL/snapshot), no input de usuario ni dependencias. El fix no relaja ninguna garantía: el fsync y el `rename` atómico quedan iguales; el único cambio es *dónde* se libera el handle según la plataforma.
- [x] **PERFORMANCE** — N/A: no hay hot path (el `redact` corre solo en `compact_wal`; `write_atomic` en snapshot/restore). Sin cambio de complejidad.

## Steps

### Step 1: `wal.rs` — hacer explícito el `drop` en wasm32
- **Archivos:** `src/wal.rs`
- **Acción:** `#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]` sobre el `drop(file)` de `write_redacted_segment` + comentario de por qué (sin `allow`). El predicate es exacto: `wasm32-unknown-unknown` (sin fd, `File` sin `Drop`) pierde el `drop`; `wasip1/2` y `wasm32-unknown-emscripten` (`target_os` distinto) lo conservan.
- **Verify:** `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` (queda 2 errores)
- **Estado:** ✅ DONE

### Step 2: `utils/fs.rs` — mismo fix en `write_atomic_with_tmp`
- **Archivos:** `src/utils/fs.rs`
- **Acción:** idéntico `cfg` sobre `drop(file)` (justo antes del `rename`).
- **Verify:** mismo comando (queda 1 error)
- **Estado:** ✅ DONE

### Step 3: `engine/insert.rs` — `try_lock` sin `expect` en el rollback wasm32
- **Archivos:** `src/storage/engine/insert.rs`
- **Acción:** `let _rollback_guard = self.insert_lock.try_lock();` (Option vivo) + comentario; comportamiento idéntico cuando el lock está libre.
- **Verify:** mismo comando → exit 0
- **Estado:** ✅ DONE

### Step 4: verificación host + tests + cierre
- **Archivos:** —
- **Acción:** `cargo check -p vantadb` (host), `cargo check -p vantadb-wasm --target wasm32-unknown-unknown`, `rustfmt --check` de los 3 archivos, nextest filtrado `redact`/`write_atomic`, OCR review, review P2-01.
- **Verify:** los 3 comandos del §Contrato en exit 0
- **Estado:** ✅ DONE — evidencia: clippy wasm32 `exit 0` (0 warnings, 11m09s con lock de `target/`), `cargo check -p vantadb-wasm --target wasm32-unknown-unknown` `exit 0`, `cargo check -p vantadb` host `exit 0`, `cargo nextest run -p vantadb -E 'test(redact) or test(write_atomic)'` → **11/11 passed**, `rustfmt --check` de los 3 archivos limpio, `node scripts/docs/check-docs.mjs` → `GATING: all clear`.

## Context Save Point

- 2026-11-24: los 3 lints cerrados y verificados en ambos targets. Falta solo el veredicto del review P2-01 y el commit local.
- 2026-11-24 (cierre): review P2-01 en 2 rondas → **approve**. Commit local **`5b0c67397`** (3 fuentes + este task file, sin push).
- **Incidente durante la tarea:** otro agente del worktree compartido commiteó mientras mis cambios estaban *unstaged* y el hook `pre-commit` los stasheó/revió sin restaurarlos (ventana documentada como `FIND-726`); `git diff` volvió vacío para los 3 archivos. Se detectó por el fallo de clippy en la línea 1347 ya arreglada, se re-aplicó todo y se re-verificó el contrato **completo sobre el árbol final**. Lección de proceso: en árbol compartido, `git add` + commit inmediato tras el verify, no dejar trabajo unstaged.

## Deuda

> **Lane de CI propuesta (no implementada acá — `.github/**` es de FIND-714/FIND-725):**
> En `.github/workflows/ci-rust.yml`, dentro del job `clippy` (el que corre `cargo clippy --workspace --all-targets --all-features -- -D warnings` en host), agregar como **step previo** (después de `dtolnay/rust-toolchain`, que debe llevar `targets: wasm32-unknown-unknown`):
>
> ```yaml
>       - name: Add wasm32 target for the core crate lint
>         run: rustup target add wasm32-unknown-unknown
>       - name: Clippy (core, wasm32 target)
>         run: cargo clippy -p vantadb --target wasm32-unknown-unknown -- -D warnings
> ```
>
> Por qué `-p vantadb` y no `-p vantadb-wasm`: `vantadb-wasm` arrastra `vantadb` igual, pero `-p vantadb` no exige `wasm-bindgen`/`web-sys` y el fallo se reporta en el crate que lo tiene. Coste medido: compila el core para wasm32 una vez (~2-3 min en CI, cacheado por sccache). Alternativa más barata si el coste molesta: `cargo check -p vantadb --target wasm32-unknown-unknown` en el Fast Gate y el `clippy` completo en la lane nocturna.

## Dependencias
- Ninguna. Carril R (comparte `target/` con FIND-724).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (adversarial: `src/wal*.rs` matchea el tier adversarial).
- **Enfoque:** ¿el `cfg` deja el WAL sin fsync en wasm? ¿y el host sin cerrar el handle? ¿algún `allow` escondido?
- **Cómo se probó:** los 3 comandos del contrato + nextest filtrado.
- **Veredicto:** ✅ **APPROVE** (ronda 2, sobre `HEAD 5b0c67397`).
  - **Ronda 1 — `cambios requeridos`** (1 Required, solo doc): el doc de `batch_insert_with_opts` (`insert.rs:918-920`) seguía prometiendo `try_lock().expect` en wasm32, un panic que el diff eliminó. Aplicado; también se aplicaron los 2 Optional (predicate exacto `not(all(wasm32, target_os="unknown"))` y `debug_assert!` que recupera el diagnóstico sin `expect`).
  - **Ronda 2 — `approve`.** El revisor re-ejecutó el contrato sobre los bytes de HEAD: clippy wasm32 `EXIT=0` (recompiló `vantadb`, 1m04s — el predicate no se volvió siempre-verdadero), `cargo check -p vantadb` `EXIT=0`, nextest 11/11, `rustfmt --check` limpio, `git diff | grep '^+.*allow'` → 0 hits. Sin Critical ni Required abiertos.
  - **Nit de historial (no reescrito):** el subject `fix(wal): make the wasm32 drop explicit` describe la solución inversa a la final (en `wasm32-unknown-unknown` el `drop` se **omite**). Se conserva por ser el mensaje fijado por el owner en el task.
- **Hallazgos (con evidencia):**
  - **Required - `src/storage/engine/insert.rs:918-920`:** el doc de `batch_insert_with_opts` afirma "MUST NOT be called while holding `insert_lock` ... would deadlock (or panic via `try_lock().expect` on wasm32)". Tras el diff no hay `expect` ni panic: la rama wasm32 guarda el `Option` y, si el lock estuviera tomado, el rollback corre bajo el guard del caller (mismo thread). El contrato ya no es "panic" sino "se ejecuta igual": dejarlo escrito documenta un fallo que ya no puede ocurrir, en la unica funcion cuyo contrato de no-reentrancia cambio. Fix: reemplazar el parentesis por el comportamiento real. Grep `try_lock().expect` en `src/` + `docs/` -> 1 hit (esta linea).
  - **Optional - `src/utils/fs.rs:70-71`:** el predicate `not(target_arch = "wasm32")` es mas ancho que la razon real. `wasm32-wasip1/2` tambien son `target_arch = "wasm32"` pero SI tienen `File` real con `Drop` (ahi `drop_non_drop` no dispararia). No rompe nada: renombrar sobre un archivo abierto es legal en POSIX/WASI y esos targets no estan soportados (`rustup target list --installed` solo trae `wasm32-unknown-unknown`). Predicate exacto si se quiere: `#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]`.
  - **Optional - alternativa sin `cfg` (evaluada, no requerida):** en `wal.rs` el handle ya muere al final de la funcion, *antes* del `swap_segment` del caller (1410 -> 1433), asi que un bloque `{ let mut file = ...; }` con `bytes_after` calculado adentro haria el cierre implicito en todos los targets y borraria el `cfg`. En `fs.rs` no es equivalente (el `rename` de la linea 78 quedaria con el handle en scope en wasm32), pero es inocuo por lo dicho arriba. Se registra como brainstorm: el `cfg` de 4 lineas es mas barato de auditar que mover codigo en una ruta de durabilidad.
- **Lo que quedo bien:** `cfg` sin `#[allow]` (`git diff | grep '^+.*allow'` -> 0 hits; el `allow(clippy::expect_used)` de `insert.rs:1212` es preexistente y de `#[cfg(test)] mod t32_tests`); el `fsync` NO se gateo (`wal.rs:1342` y `fs.rs:66` intactos); e `insert.rs` no degrada: en wasm32 el lock esta libre de forma determinista al llegar al rollback (`_guard` de `insert.rs:981` muere con el closure en `1089`), y en el caso patologico (contrato violado) el rollback corre bajo el guard del caller, que es el mismo thread -- nunca queda sin proteccion.
- **Evidencia propia (re-ejecutada por el revisor, no auto-reporte):** `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` -> EXIT=0 (fingerprint fresco, sin recompilar: los fuentes son los mismos que se lintearon); `cargo check -p vantadb` (host) -> EXIT=0 (recompilo el crate, 52.58s); `rustfmt --edition 2021 --check` de los 3 archivos -> EXIT=0; `cargo nextest run -p vantadb --lib -E 'test(redact) or test(write_atomic)'` -> **11/11 passed**, incluido `utils::fs::tests::test_write_atomic_replaces_file_and_removes_tmp` (el `rename` sobre Windows justo despues del `drop`). Nota: el `drop` de `wal.rs` tampoco es load-bearing respecto al orden -- el handle muere al final de la funcion antes del `swap_segment` del caller, asi que el `cfg` no puede retrasar el cierre mas alla del rename en ningun target.
- **DoD Commit:** el arbol tiene 13 archivos modificados de otras tareas (`docs/api/*`, `src/parser/mod.rs`, `src/query.rs`, `vantadb-ts/*`, `vantadb-wasm/src/lib.rs`, ...). El commit de FIND-721 debe ser `git add` explicito de los 3 archivos + este task file, nunca `-a`, o el gate de commit atomico falla.

## Notas
- Gate D disparado: el contrato (exit 0) era inalcanzable con el scope de escritura de §2 (3 errores, 1 solo archivo permitido). Owner eligió arreglar los 3.
