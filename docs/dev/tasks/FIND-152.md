---
title: "FIND-152: `unwrap` en `shred` y `wasm` — clasificación por superficie"
kind: task
status: active
description: "El ticket medía 43+49 `unwrap` como 'contexto prod'. Medido: los 92 son test-only, 0 en producción en ambas superficies. Cierra por clasificación + elimina el único panic real de prod en wasm (`worker.rs:237` `unreachable!()` → `Err`). Deriva FIND-758: el purgado de shred traga el fallo con `let _ =`."
---

# FIND-152: `unwrap` en `shred` y `wasm` — clasificar, no convertir

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (Fase 2 / §6.c)
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-152`, origen baseline semgrep/ast-grep `FIND-149`, 2026-09-24)
- **Esfuerzo:** 🟡 2-3h
- **Prioridad:** 🟡 Media
- **Tipo:** audit + fix puntual WASM
- **Creado:** 2026-11-26
- **Estado:** ✅ COMPLETED
- **SDP:** `security-and-hardening` (pin: trust boundary) · `incremental-implementation` · `test-driven-development` · `context-engineering` · `doubt-driven-development` · `source-driven-development` · base (`campaign-executor`, `progreso`)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `OpfsWorkerProxy::send` tiene un call-site (`worker.rs`, camino `connect_worker`). El cambio no altera la firma ni el tipo de error: `Err(JsValue)` en vez de `panic!` |
| Callees | `try_send` → `Promise`/`MessageChannel` (js-sys). Sin cambios |
| Implicaciones | Ninguna sobre comportamiento observable: el `unreachable!()` era **inalcanzable por construcción** (ver §Clasificación). Se cambia la clase de fallo de `abort` a `Err(JsValue)` en el host JS |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/shred/mod.rs` (1217 L) · `vantadb-wasm/src/worker.rs:160-238` + cabecera · `src/storage/engine/delete.rs:1-90` · `vantadb-wasm/Cargo.toml:46-62` · `.opencode/rules/js-ecosystem.md` · `docs/dev/tasks/FIND-739.md` (patrón de task file) · `docs/dev/avance/activo/bindings.md` (cola: 3 entradas de formato)
- **Referencias hacia dentro:** `mod worker` es `#[cfg(feature = "opfs")]` (`lib.rs:99-100`); `OpfsWorkerProxy` sólo lo construye el camino `connect_worker`. `ShreddedRowStore` lo consumen `src/sdk/api/memory.rs` (put), `src/storage/engine/delete.rs` (delete/delete_batch/purge_permanent) y `src/storage/engine/tests/engine.rs`
- **Referencias entrantes:** `rg FIND-152` → `docs/dev/Backlog.md`, `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md:213`. Ningún `.rs` referencia el ID
- **Veredicto impacto:** bajo en `worker.rs` (1 línea, rama demostrada inalcanzable). **Nulo en `src/shred/mod.rs`**: 0 líneas de producción con `unwrap`.

## Contrato

```
rg -n '\.unwrap\(\)' src/shred/mod.rs vantadb-wasm/src/lib.rs   # antes y después: números pegados
cargo nextest run --profile audit -p vantadb -E 'test(shred)' --build-jobs 2
cargo check -p vantadb-wasm --target wasm32-unknown-unknown --features opfs
```

más el conteo por clase (a)/(b)/(c) y los que quedan con su justificación.

**Prohibido:** `let _ =` sobre operaciones de borrado · añadir `Result` en cascada · tocar `src/storage/**` (propiedad de Arch/Engine — el hallazgo derivado se registra, no se arregla acá).

## Clasificación medida (2026-11-26)

Conteo mecánico, `rg -n '\.unwrap\(\)'`, con la frontera de test determinada por los límites de módulo:

| Superficie | `unwrap()` total | **prod** | test | Frontera test |
|---|---|---|---|---|
| `src/shred/mod.rs` | **43** | **0** | 43 | `#[cfg(test)] mod tests` desde `:488` (`#[allow(clippy::unwrap_used, clippy::expect_used)]` en `:489`) |
| `vantadb-wasm/src/lib.rs` | **49** | **0** | 49 | primer módulo de test en `:2269` (`core02_graph_persist_tests`) |
| `vantadb-wasm/src/lib.rs` (`.expect(`) | 42 | **0** | 42 | idem |
| `vantadb-wasm/src/{opfs,idb,worker}.rs` | 0 | 0 | — | sin `unwrap`/`expect` |
| **Total** | **92 + 42 expect** | **0** | 134 | |

Las cifras del ticket (43 / 49) son **exactas**; lo que no se sostiene es su encuadre *"en contexto prod"*. El baseline de 2026-09-24 (`wasm` 14 prod / 35 test) ya no aplica: 14 prod pasaron a 0.

### Clases

| Clase | N | Disposición |
|---|---|---|
| **(a)** infalible por construcción / no-producción | **92** | Los 92 `unwrap()` son de test. El módulo `tests` de `shred` ya lleva `#[allow(clippy::unwrap_used, clippy::expect_used)]`: un `unwrap` en un test es el idioma correcto (el fallo ES el reporte). **No se convierten**: `expect("razón")` en un test no compra nada y 92 líneas de ruido. Quedan con justificación documentada, que es exactamente lo que pide el contrato |
| **(b)** frontera de error real | **0 `unwrap` + 1 `unreachable!`** | `worker.rs:237` — ver abajo |
| **(c)** invariante que habría que demostrar | **0** | No hay `unwrap` productivo que exija cambiar firmas |

### El único panic real de producción: `worker.rs:237`

```rust
for attempt in 0..=MAX_RETRIES {          // MAX_RETRIES = 2 → 0,1,2
    match self.try_send(req).await {
        Ok(resp) => return Ok(resp),
        Err(e) if attempt < MAX_RETRIES && is_retryable(&e) => { /* backoff */ }
        Err(e) => return Err(e),          // catch-all sin guarda
    }
}
unreachable!()                            // inalcanzable por construcción
```

En la iteración final `attempt == MAX_RETRIES`, la guarda de reintento es falsa y el brazo `Err(e)` devuelve: el bucle no puede caer. **Es infalible por construcción (a), pero con el peor fallo posible en wasm**: `unreachable!()` compila a `panic!`, y un panic en un módulo wasm **aborta el host** en vez de llegar al llamador JS como excepción. La alternativa sin panic es trivial (`Err(JsValue::from_str(..))`), así que se aplica la rama "alternativa sin panic si es trivial" de la clase (a).

## Invariantes de dominio (handoff — MUST)

1. **Cero `unwrap`/`expect`/`panic!`/`unreachable!` en código de producción** en `src/shred/mod.rs` y en la región prod de `vantadb-wasm/src/**`. Este es el invariante que la fila protegía, y ahora es medible: `rg` sobre la región prod = 0.
2. **Un fallo de borrado nunca se silencia.** El único `let _ =` del camino de borrado de shred (`src/storage/engine/delete.rs:54`) es deuda **registrada** (FIND-758), no un patrón a replicar.
3. **`MAX_RETRIES`/`is_retryable` no cambian:** el reintento sigue siendo 3 intentos y sólo sobre el timeout tipado (`VantaWorkerTimeout`). El cambio toca sólo la rama post-bucle.
4. **`src/storage/**` no se toca en este ticket** (Arch/Engine). El hallazgo se reporta con evidencia y fix propuesto.

## Deuda técnica (Regla 6)

**Saldo neto: negativo.** Se elimina un panic de producción (1 `unreachable!()` → `Err`) sin añadir código de producción; los 92 `unwrap` de test quedan con allow explícito y justificación. La deuda que **no** se paga acá queda como fila propia (FIND-758) por límite de propiedad de fichero, no por pereza.

## Verificación

| Check | Comando | Resultado |
|---|---|---|
| Conteo antes/después | `rg -n '\.unwrap\(\)' src/shred/mod.rs vantadb-wasm/src/lib.rs` | **43 / 49 antes · 43 / 49 después** — el fix no era un `unwrap`, y por eso el contrato literal del ticket no podía verlo |
| Familia de panic en prod | `rg 'panic!\|unreachable!\|todo!\|unimplemented!\|\.unwrap\(\)\|\.expect\('` sobre `src/shred/mod.rs:1-487` y `vantadb-wasm/src/{lib.rs:1-2268,opfs,idb,worker}.rs` | **0** (antes: 1 en `worker.rs:237`) |
| Tests de shred | `cargo nextest run --profile audit -p vantadb -E 'test(shred)' --build-jobs 2` | **35/35 ✅** |
| Compila en el target real | `cargo check -p vantadb-wasm --target wasm32-unknown-unknown --features opfs` | **✅ 29,9 s** (sin `--features opfs` el módulo `worker` ni se compila: el gate no mediría) |
| Formato | `cargo fmt -p vantadb-wasm -- --check` | **✅ exit 0** |
| OCR review | `pwsh dev-tools/ocr-review.ps1` | ver §Review |
| `.gitignore` / temporales | — | sin `.tmp-*` en el repo (los scripts auxiliares viven en `%TEMP%\opencode\`) |

**Clase (b) medida en `shred`:** 0. El decodificador ya propaga (`take`/`take_array` + `corrupt_row`, SH-02) y `delete`/`delete_op` devuelven `Result`. Ningún `unwrap` quedó sin convertir porque no había ninguno en producción.

**No corridos, con razón:**
- `cargo clippy --target wasm32-unknown-unknown` — **roto por `FIND-721`** (`drop_non_drop`, `src/wal.rs:1347`), fuera de este ticket. El gate aplicable hoy es `cargo check`.
- Suite completa `--workspace` — pertenece al verify de batch del orquestador (§3.c del plan); acá se corrió el filtro acotado del contrato.

## Review P2-01

Tier **adversarial** (el diff toca `vantadb-wasm/src/**`, binding: aunque `src/storage/**` no se toca, el cambio es en una frontera de FFI/panic). Agente distinto: `vanta-review`, una ronda. Dictamen en §Cierre.

## Nota de proceso (árbol compartido + disco)

- Al commitear, el hook `pre-commit` corre `cargo fmt --check` y `clippy` **sobre el árbol entero** y falló sobre ficheros que **no son míos** (`vanta-memory/src/core/conversation/consolidation.rs`, `vantadb-mcp/src/memory_scheduler.rs` — carriles ajenos en vuelo, `FIND-745` y la ola MCP/MEM). El propio hook imprime la salida: *"if one fails on a file you did not stage, use: `SKIP=cargo-fmt,cargo-clippy git commit`"*. Se usó **eso**, no `--no-verify`: mis gates de área sí corrieron (`cargo fmt -p vantadb-wasm -- --check` exit 0; `cargo check` wasm32+`opfs` ✅) y el hook siguió ejecutando sus gates restantes.
- El primer intento de commit **no aterrizó** y dejó el disco en **0 bytes** (el hook reconstruye el workspace). Mitigación del plan: `pwsh dev-tools/target-cleanup.ps1 -AutoSafe` → **11,06 GB liberados** (1,73 → 12,79 GB). El script avisa que sigue por debajo de su umbral de 15 GB: es la deuda de `FIND-192`, no de esta tarea.
- Commit con `GIT_INDEX_FILE` temporal (`%TEMP%\opencode\idx-152`) y **pathspec explícito** de 5 rutas: el árbol tenía 7 ficheros ajenos modificados y `web/*.png` sin trackear. Sin esto, `git add -A` habría horneado trabajo de otro carril (la clase de `FIND-753`).

## Deuda derivada

`FIND-758` — el purgado de shred se traga el fallo con `let _ =` (`src/storage/engine/delete.rs:54`). Registrada en `docs/dev/Backlog.md`; no se arregla acá por propiedad de fichero (Arch/Engine).
