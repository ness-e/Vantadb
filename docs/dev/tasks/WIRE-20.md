---
title: "WIRE-20: el intervalo del scheduler se resuelve en su dueño (vanta-memory)"
kind: task
status: complete
description: "DEFAULT_INTERVAL_MS, parse_interval_ms y las dos variantes de entorno salen de vantadb-server a vanta-memory/src/services/scheduler.rs (ADR-0063). scheduler_interval_ms_from_env_opt nace como primitiva de lectura única del env y from_env pasa a ser su unwrap_or; el server re-exporta los cuatro nombres."
id: WIRE-20
---

# WIRE-20: el intervalo del scheduler se resuelve en su dueño

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §9.b (ola MCP/MEM).
- **Fuente:** fila `WIRE-20` de [docs/dev/Backlog.md](../Backlog.md) (partida de `WIRE-19`, 2026-11-27).
- **Madre:** [WIRE-19](WIRE-19.md) (decisión) + [WIRE-19-impl](WIRE-19-impl.md) §4.2 / §5 / §7.7 (partición y orden).
- **Esfuerzo real:** 🟢 ~50 min (estimado 1 h).
- **Tipo:** refactor de hogar + un símbolo público nuevo (`_opt`).
- **Carril:** Rust (`--build-jobs 2`, disco 19 GB libres).
- **Creado:** 2026-11-27.

## Blast Radius

| Dirección | Módulos |
|---|---|
| **Escritura (esta tarea)** | `vanta-memory/src/services/scheduler.rs` (las cuatro piezas + 2 tests) · `vantadb-server/src/scheduler.rs` (borrado del bloque + re-export + 1 bullet de doc de módulo) |
| **Referencias entrantes medidas (`rg`)** | `vantadb-server/src/main.rs:88` → `scheduler_interval_ms_from_env()` (sigue igual, vía re-export) · `vantadb-server/src/scheduler.rs` `mod tests` → `DEFAULT_INTERVAL_MS`, `parse_interval_ms` (las tres aserciones siguen compilando vía re-export) · `vantadb-pro` (repo aparte, no verificable: por eso el re-export) |
| **No toqué** | `vantadb-mcp/**` (`WIRE-23`) · `vanta-memory/src/ingest/runner_config.rs` (`WIRE-21`) · `vantadb-server/src/main.rs` y `tests/scheduler_e2e.rs` (`WIRE-22`) |
| **Implicación de seguridad** | Ninguna nueva: la única variable leída sigue siendo `VANTADB_SCHEDULER_INTERVAL_MS` (un intervalo, no un secreto). El módulo de `vanta-memory` deja de poder afirmar "never reads env" — el doc se corrigió en el mismo cambio |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-server/src/scheduler.rs` (182 L) · `vanta-memory/src/services/scheduler.rs` (396 L) · `vanta-memory/Cargo.toml` · `vantadb-mcp/src/memory_scheduler.rs` (L60-110, L305-330) · `vanta-memory/src/services/mod.rs` · `docs/dev/tasks/WIRE-19.md` · fila `WIRE-20` del Backlog · ADR-0063 §Migration surface fila 2 + §H-B.
- **Referencias hacia dentro:** los cuatro nombres eran `pub` en `vantadb_server::scheduler` (lib publicada en el workspace) → `vantadb-pro` los consume fuera del repo; `vanta-memory` es publicable (DIST-01) así que los cuatro items nuevos son superficie semver. Ningún item necesita feature gate.
- **Referencias salientes:** el bloque movido no depende de nada del server (solo `std::env`, `tracing`); `vanta-memory` ya tiene ambos. `wire_memory` se queda (composición sobre `ServerHooks`, ADR-0063 D2).
- **Veredicto:** **bajo en corrección, medio en compatibilidad.** El riesgo real es romper a quien importa `vantadb_server::scheduler::<name>`; se neutraliza con 4 `pub use` (compilación probada con `cargo check -p vantadb-server --all-targets`, que incluye `main.rs` y el `mod tests`).

## Spec

| Decisión | Elección | Evidencia |
|---|---|---|
| Dónde vive el bloque | `vanta-memory/src/services/scheduler.rs`, **ungated** — fuera de cualquier `#[cfg(feature = ...)]` (los dos `use` gateados no lo condicionan; probado con `--no-default-features`) | ADR-0063 §Migration surface fila 2; `WIRE-19-impl` §7.7 (el MCP no ve lo gateado) |
| ¿Módulo nuevo? | No | ADR-0063 "place by owner" |
| `_opt` con el env **sin definir** | `None` (único caso `None`) | ADR-0063:247 — "`None` = unset" |
| `_opt` con el env **presente y roto** | `Some(DEFAULT_INTERVAL_MS)` (vía `parse_interval_ms`, que ya avisa) | R-5: `warn!` + default, nunca panic; el ADR acota `None` a "unset", así que "presente y roto" no es `None` |
| `from_env` | `_opt().unwrap_or(DEFAULT_INTERVAL_MS)` — **no** lee el env | ADR-0063:248; R-5 una sola lectura |
| Default del intervalo | `60_000`, byte-idéntico | Medido: `vantadb-server/src/scheduler.rs:45` en `HEAD`; aserción explícita en el test nuevo |
| Test existente del server (`:140-146`) | No se toca: sigue ejerciendo la **misma** función (ahora por re-export), no una copia | Fila `WIRE-20`: reubicación del `mod tests` = `WIRE-22`; al ser re-export no hay duplicación |

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Discovery: madre + ADR + fila, medir el delta (`rg`) y localizar cada lector de los cuatro nombres | ✅ |
| 2 | RED: medir que `scheduler_interval_ms_from_env_opt` **no existe** en `HEAD` (`rg` = 0 hits) — el test de la propiedad no compila antes del cambio | ✅ |
| 3 | GREEN-1: mover `DEFAULT_INTERVAL_MS` + `parse_interval_ms` **verbatim** y escribir `_opt` + `from_env` como wrapper, ungated, en el dueño | ✅ |
| 4 | GREEN-2: borrar el bloque del server y re-exportar los cuatro nombres; corregir el doc de módulo del dueño ("never reads env") y la ruta del default, que pasa a `src/config.rs` de la crate raíz (R-4: el prefijo `vantadb/src/...` está prohibido) | ✅ |
| 5 | Tests de la propiedad (`_opt` Some/None/no-parseable; `from_env` = `_opt().unwrap_or(DEFAULT)`; default byte-idéntico) | ✅ |
| 6 | Contrato mecánico: `list` → población, `run` con **el mismo** filtro | ✅ |
| 7 | `cargo check -p vanta-memory --no-default-features` + clippy `-D warnings` + fmt | ✅ |
| 8 | Docs (este task file + avance + fila del Backlog) y review P2-01 | ✅ |

## Contrato mecánico

| Check | Comando | Antes → Después |
|---|---|---|
| Las cuatro piezas en el dueño | `rg -n 'pub const DEFAULT_INTERVAL_MS\|fn parse_interval_ms\|fn scheduler_interval_ms_from_env' --glob '*.rs'` | definiciones en `vantadb-server/src/scheduler.rs:45,49,67` y `_opt` **0 hits** → `vanta-memory/src/services/scheduler.rs:62,66,92,101` (las únicas; `auto_sync.rs:36` es otro concepto, sin tocar) |
| Read site del env (métrica de la fila) | `rg -n 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' --glob '*.rs'` | **2** (`vantadb-server:69` + `vantadb-mcp:69`) → **2** (`vanta-memory:92` + `vantadb-mcp:69`) — **server 1 → 0**; el absoluto **1** exige `WIRE-23` (el read site del MCP es de esa fila y está prohibido acá, ver §Resultado) |
| Población del filtro antes de confiar en el run (FIND-794) | `cargo nextest list --profile audit -p vanta-memory -p vantadb-server --build-jobs 2 -E 'test(schedul) or test(interval) or test(ingest)'` | **33** → **35** (los dos tests nuevos) |
| El `run`, mismo filtro | `cargo nextest run --profile audit -p vanta-memory -p vantadb-server --build-jobs 2 -E 'test(schedul) or test(interval) or test(ingest)'` | — → **35/35 passed** |
| Feature set mínimo (gate de la fila) | `cargo check -p vanta-memory --no-default-features` | — → ✅ |
| Superficie del server intacta | `cargo check -p vantadb-server --all-targets` | — → ✅ (incluye `main.rs` y `mod tests`) |
| Lint / formato | `cargo clippy -p vanta-memory -p vantadb-server --all-targets -- -D warnings` · `cargo fmt --check` | — → ✅ / ✅ |

## Review (P2-01)

Tier **Fast** (paths: `vanta-memory/src/**`, `vantadb-server/src/**`, `docs/dev/**` — ninguno de la tabla adversarial: no hay `src/storage/**`, `src/wal*`, `src/sdk/**`, `src/parser/**`).

| Campo | Valor |
|---|---|
| Reviewer | `vanta-review` (contexto fresco, sesión `ses_de3842ee…`; re-ejecutó el contrato por su cuenta) |
| Ronda | 1 |
| Veredicto ronda 1 | 🔴 **CAMBIOS REQUERIDOS** — 1 Blocking, 1 High, 2 Medium, 3 Low |
| Código | ✅ aprobado por el reviewer: movimiento byte-idéntico, re-export cubre a todos los lectores reales, server de verdad en 0 read sites, gate de población respetado |
| Resolución B1 | La fila `WIRE-20` del Backlog había sido **revertida** por un commit ajeno (`FIND-794`) que reescribió el fichero desde una copia stale (clase `FIND-753`). Re-aplicada, `0 1` verificado antes del commit |
| Resolución H1 | `vantadb/src/config.rs` **no existe** y `api-contract.md:31` (R-4) prohíbe ese prefijo explícitamente → los tres sitios citan ahora el `src/config.rs` **de la crate raíz**. El ADR-0063:457 prescribe el string incorrecto: **erratum del owner** (fuera del write-scope de esta fila) |
| Resolución M2 | El doc afirmaba "the single read site" árbol-wide cuando hoy hay 2 (el del `--mcp` es `WIRE-23`) → acotado a "in this crate", con el pendiente nombrado. Era la misma clase de invariante falso que la tarea nació a corregir |
| Resolución M1 | **No es defecto de esta implementación** (el ADR fija `None = unset` y `parse_interval_ms` se mueve verbatim); lo que queda roto es la fila `WIRE-23`, que promete basura→5 000 y un test "sin tocar". Declarado como deuda de esa fila, sin cambio de semántica acá |
| Resolución L1 | `"por encima del primer #[cfg]"` era falso al pie de la letra (el primer `#[cfg]` gatea un `use`) → reformulado a **ungated**, que es la propiedad que importa y está probada con `--no-default-features` |
| Pendiente para el orchestrator (L2/L3) | (a) `WIRE-19`/`WIRE-22` conservan premisa stale ("el bloque sigue en el server", "borrar las definiciones") — hay que actualizarlas antes de despacharlas; (b) `_opt` se re-exporta **sin consumidor que lo nombre**: el test de superficie de `WIRE-22` debe incluirlo o un tidy-up lo borra en silencio; (c) `CONFIGURATION.md:136` ya no es cierta, y es de `WIRE-22` |

## Resultado

Las cuatro piezas viven en `vanta-memory/src/services/scheduler.rs` con `_opt` como primitiva y `from_env` como su `unwrap_or`; el server re-exporta los cuatro nombres y no lee el env del intervalo.

**Desviación declarada del gate literal "el env debe quedar en 1":** el segundo read site está en `vantadb-mcp/src/memory_scheduler.rs:69`, que esta tarea tiene **prohibido** tocar (`WIRE-23`). Medido: absolute 2 → 2, con el server 1 → 0. El `1` de la fila es el punto de convergencia **al final de la ola**, no de este slice (la propia fila lo dice: "no se mide el conteo final en esta fila").

**Consecuencia anotada para `WIRE-23` (no la arreglo acá):** `_opt` devuelve `Some(DEFAULT_INTERVAL_MS = 60 000)` cuando la variable está **presente y no parsea** (el ADR acota `None` a "unset"). Si el host `--mcp` pasa a `_opt().unwrap_or(5_000)`, un valor basura resolverá 60 000, no 5 000, y su test `interval_env_is_one_knob…` (que esa fila declara intocable) asertará distinto. Es la única línea que `WIRE-23` tendrá que mirar de este slice. El reviewer confirmó que la alternativa (`None` también para no-parseable) **contradiría el ADR por escrito** y perdería el `warn!` de R-5: se resuelve en `WIRE-23`/ADR, no acá.

**Deuda declarada al cerrar (ninguna es de esta tarea):** (a) `ADR-0063:457` prescribe `vantadb/src/config.rs`, prefijo que R-4 prohíbe → erratum del owner; (b) la fila `WIRE-23` promete basura→5 000 con `_opt` actual → enmendarla antes de despacharla; (c) `WIRE-19`/`WIRE-22` conservan premisa stale (el bloque ya no está en el server) → actualizarlas; (d) `CONFIGURATION.md:136` pasa a nombrar `vanta-memory/src/services/scheduler.rs` (fila de `WIRE-22`); (e) el test de superficie de `WIRE-22` debe nombrar `_opt`, o el `pub use` se puede borrar sin que nada falle.

## Referencias

- [ADR-0063](../architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md) — §Migration surface fila 2, §H-B.
- [WIRE-19](WIRE-19.md) · [WIRE-19-impl](WIRE-19-impl.md) §4.2 / §5 / §7.7 / §8.0.
- `.opencode/rules/server-mcp.md` · `.opencode/rules/core-engine.md` (R-5) · `.opencode/rules/api-contract.md` (R-4 rutas reales).
