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
- **SDP:** `security-and-hardening` (pin: trust boundary) · `incremental-implementation` · `test-driven-development` · `context-engineering` · `doubt-driven-development` · `source-driven-development` · `documentation-skill` (obligatoria por `AGENTS.md` Regla 3: el entregable son 4 `.md` bajo `docs/`) · base (`campaign-executor`, `progreso`)

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

### Los otros dos entregables de la fila (encontrados por el review, no por mí)

El texto de la fila en el Backlog no era sólo los `unwrap`: decía también **"`+ 14 unchecked-ops ast-grep + 67 unsafe-review pointers`. Disposar antes del endurecimiento a ERROR (fila por fila)"**. Borrar la fila sin tocarlos los dejaba sin dueño fuera del único tracker vivo. Medidos:

| Ítem | Medición (2026-11-26) | Clase / disposición |
|---|---|---|
| **unchecked-ops** (`*_unchecked`) | `rg -c '_unchecked' src/**/*.rs` → **16**, de los cuales **15 en producción**: `src/index/distance/kernels.rs` **12** + `src/index/distance/mapper.rs` **3** (el 16.º es `src/index/graph/tests.rs`, test) | **(a) infalible por construcción, justificado `// SAFETY:`** — los dos ficheros llevan **14** comentarios `// SAFETY:` (12 + 2, uno cubre dos call-sites) y el invariante es el mismo en todos: `chunks_exact(N)` garantiza `chunk.len() == N`. **No se convierten**: `get_unchecked` no tiene alternativa segura de coste equivalente en un kernel SIMD, y la alternativa (`get`) es lo que el kernel existe para evitar. El ticket decía 14 y hoy hay 15: el delta es una instancia nueva, no una stale |
| **unsafe-review pointers** | `rg -n 'unsafe\s*(\{\|fn \|impl )' src/ -g '!**/tests/**' -g '!**/*tests.rs'` → **70** sitios · `rg -c '// SAFETY:'` sobre el mismo scope → **54** comentarios | **(c) invariante que hay que demostrar sitio por sitio** — no es un conteo que se cierre leyendo: hay que emitir veredicto por sitio. Registrado como **`FIND-759`**, no cerrado acá |

- **Por qué el unsafe no se cierra en esta tarea:** un recuento no prueba nada. Los 70 sitios menos los 54 `SAFETY` **no** equivalen a "16 sin justificar" (un comentario puede cubrir dos bloques contiguos, y varios `unsafe fn` heredan el invariante del caller). Emitir ese veredicto es una revisión de seguridad por sitio, que es el objeto de `vanta-audit` — y por eso la fila derivada conserva el contrato, en vez de heredar un número que nadie va a poder reproducir.

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
| OCR review | `pwsh dev-tools/ocr-review.ps1` | rule group `project / {vantadb-wasm/**,…}`: el cambio **es** la remediación de su regla "`unwrap()`/`panic!`/`unimplemented!` en producción donde el fallo es propagable" |
| unchecked-ops heredados de la fila | `rg -c '_unchecked' src/**/*.rs` | **16** (15 prod + 1 test), 14 `// SAFETY:` en los dos ficheros → clase (a) |
| unsafe heredados de la fila | `rg 'unsafe\s*(\{\|fn \|impl )' src/ -g '!**/tests/**'` · `rg -c '// SAFETY:'` | **70** sitios · **54** comentarios → `FIND-759` (veredicto por sitio, no por recuento) |
| Gates de docs | `node scripts/docs/check-links.mjs` · `check-docs.mjs` · `markdownlint-cli2 --config .github/…` | **exit 0** · **exit 0**, orphans **0** · **0 ficheros** (mis 4 rutas están en el `ignores` del config, por diseño) |
| `.gitignore` / temporales | — | sin `.tmp-*` en el repo (los scripts auxiliares viven en `%TEMP%\opencode\`) |

**Clase (b) medida en `shred`:** 0. El decodificador ya propaga (`take`/`take_array` + `corrupt_row`, SH-02) y `delete`/`delete_op` devuelven `Result`. Ningún `unwrap` quedó sin convertir porque no había ninguno en producción.

**No corridos, con razón:**
- `cargo clippy --target wasm32-unknown-unknown` — **roto por `FIND-721`** (`drop_non_drop`, `src/wal.rs:1347`), fuera de este ticket. El gate aplicable hoy es `cargo check`.
- Suite completa `--workspace` — pertenece al verify de batch del orquestador (§3.c del plan); acá se corrió el filtro acotado del contrato.

## Review P2-01

Tier **adversarial** (el diff toca `vantadb-wasm/src/**`, binding: una frontera de panic que cruza a JS).

- **Revisor:** `vanta-review`, contexto fresco, **una ronda** (sesión distinta de la que implementó).
- **Enfoque:** (C1) si la frontera producción/test de los 92 `unwrap` estaba bien establecida, cruzada por dos criterios independientes; (C2) si la aritmética del bucle de reintento hace realmente inalcanzable la rama y si convertir un panic en `Err` cambia algo para los 5 consumidores; (C3) si abstenerse de arreglar `delete.rs:54` era correcto; (C4) si cerrar la fila dejaba trabajo vivo; (C5) frontmatter, enlaces y `description`.
- **Cómo se probó:** re-ejecutó `rg` de los tres conteos y del alcance `unsafe`/`unchecked`; leyó `send`/`try_send`/`MAX_RETRIES`/`is_retryable` y los 5 call-sites públicos hasta `connect_worker` (`lib.rs:677`) y `worker_read/write/delete` (`lib.rs:718/730/742`), verificando que **ningún `let _ =`** traga el error nuevo; re-corrió `cargo check` wasm32+`opfs` (0 warnings) y `check-links`/`check-docs`; leyó `take()`/`take_array` de `shred` línea por línea.
- **Veredicto ronda 1:** 🔴 **changes-required** — 3 bloqueos, ninguno en el fix de código:
  1. **[Alto]** la fila borrada arrastraba 2 de 3 entregables (`14 unchecked-ops` + `67 unsafe-review pointers`) y quedaban sin dueño → **aplicado**: medidos (15 `_unchecked` productivos con `SAFETY`; 70 sitios `unsafe` / 54 `SAFETY`) y el unsafe pasa a `FIND-759`.
  2. **[Medio]** el gate remitía a un `§Cierre` inexistente → **aplicado**: este bloque.
  3. **[Medio]** SDP sin `documentation-skill` siendo una tarea 80 % documental → **aplicado**: agregada.
- **Nit aplicado:** la rama devolvía un `JsValue::from_str` mientras todo el archivo usa `js_sys::Error`; ahora `js_sys::Error::new(..).into()` para que el lado JS vea **una sola clase de error**.
- **Nit no aplicado (declarado):** el contrato literal del ticket (`rg '\.unwrap\(\)'`) no puede medir este fix; el gate real es el `rg` de la familia de panic. Se deja escrito en §Verificación en vez de reescribir el contrato del plan.
- **Ronda 2:** no ejecutada (una ronda por contrato). Los 3 bloqueos y el nit quedan **aplicados en el commit de cierre**, verificados por `cargo check` wasm32+`opfs` y `cargo fmt --check`.

## Nota de proceso (árbol compartido + disco)

- Al commitear, el hook `pre-commit` corre `cargo fmt --check` y `clippy` **sobre el árbol entero** y falló sobre ficheros que **no son míos** (`vanta-memory/src/core/conversation/consolidation.rs`, `vantadb-mcp/src/memory_scheduler.rs` — carriles ajenos en vuelo, `FIND-745` y la ola MCP/MEM). El propio hook imprime la salida: *"if one fails on a file you did not stage, use: `SKIP=cargo-fmt,cargo-clippy git commit`"*. Se usó **eso**, no `--no-verify`: mis gates de área sí corrieron (`cargo fmt -p vantadb-wasm -- --check` exit 0; `cargo check` wasm32+`opfs` ✅) y el hook siguió ejecutando sus gates restantes.
- El primer intento de commit **no aterrizó** y dejó el disco en **0 bytes** (el hook reconstruye el workspace). Mitigación del plan: `pwsh dev-tools/target-cleanup.ps1 -AutoSafe` → **11,06 GB liberados** (1,73 → 12,79 GB). El script avisa que sigue por debajo de su umbral de 15 GB: es la deuda de `FIND-192`, no de esta tarea.
- Commit con `GIT_INDEX_FILE` temporal (`%TEMP%\opencode\idx-152`) y **pathspec explícito** de 5 rutas: el árbol tenía 7 ficheros ajenos modificados y `web/*.png` sin trackear. Sin esto, `git add -A` habría horneado trabajo de otro carril (la clase de `FIND-753`).

## Deuda derivada

- **`FIND-758`** — el purgado de shred se traga el fallo con `let _ =` (`src/storage/engine/delete.rs:54`). No se arregla acá por propiedad de fichero (Arch/Engine).
- **`FIND-759`** — inventario y veredicto por sitio de los 70 `unsafe` de `src/**` (54 con `// SAFETY:`). No se cierra con un recuento; lo que heredó de `FIND-152` es el contrato, no el número.
