---
title: "FIND-739: `live_writer_lock_blocks_promotion` mide el reloj en vez del mecanismo de reintento"
kind: task
status: active
description: "La aserción `elapsed < 5s` del test de MCP-48 mezclaba dos relojes: el presupuesto de reintento (3 sondas + backoff) y la paciencia de lock que el test nunca configuraba (1000 ms por defecto). Ahora afirma el mecanismo (una resolución, cero elecciones, `DatabaseBusy`, rol y handle intactos) y deja el reloj como red gruesa de 10 s"
---

# FIND-739: `live_writer_lock_blocks_promotion` mide el reloj, no el mecanismo

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (residual del run; la fila vive solo en `Backlog.md:784`)
- **Fuente:** `docs/dev/Backlog.md:784` · detectado por `MCP-44` (`2fb20c728`)
- **Esfuerzo:** 🟢 1-2h
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (test-only)
- **Creado:** 2026-11-26
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno: `live_writer_lock_blocks_promotion` es un test de integración, no tiene callers |
| Callees | `WriterProxy::new` → `call` → `resolve_with_backoff` (`vantadb-mcp/src/proxy.rs:522`) → `resolve_once` (`:544`) → `try_proxy`/`cleanup_stale` → `promote` (`:575`) → `StorageEngine::open_with_config` → `acquire_db_lock` (`src/storage/engine/init.rs:318`) → `file_lock_timeout_ms` |
| Implicaciones | Ninguna sobre producto. El diff es un test + docs. El contrato del producto (`la promoción se acota y falla cerrado`) queda **fijado con más fuerza** que antes: ahora también se assertea `resolve_failures()` y `connected()` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/tests/mcp_proxy_reconnect.rs` (373 L) · `vantadb-mcp/src/proxy.rs` (864 L) · `src/storage/engine/init.rs:300-417` (`acquire_db_lock`) · `src/config.rs:470-520` (`PoolCfg`/`StorageCfg`) · `docs/dev/tasks/FIND-724.md` (patrón de fix y formato de task file) · `docs/dev/avance/activo/bindings.md:1679-1688` (§`MCP-48`) · `docs/dev/avance/activo/core-engine.md:1667-1674` (§`FIND-724`)
- **Referencias hacia dentro:** helpers del propio crate de test (`write_discovery`, `spawn_fake_writer`, `tools_call`); `vantadb::config::Config`; `vantadb::storage::StorageEngine::open_with_config`; `vantadb::Error::DatabaseBusy`
- **Referencias entrantes:** `rg FIND-739` → `docs/dev/Backlog.md`, `docs/dev/avance/activo/bindings.md` (`MCP-48` deuda observada), este task file. Ningún `.rs` de producción referencia el test
- **Veredicto impacto:** bajo — test-only; no toca `vantadb-mcp/src/**`, ni `src/**`, ni firmas públicas, ni wire format

## Contrato

```
cargo nextest run --profile audit -p vantadb-mcp \
  -E 'test(live_writer_lock_blocks_promotion)' --build-jobs 2
```
pasa, **más 20 corridas seguidas sin un fallo** — bajo carga (6 jobs quemando CPU en paralelo). Prohibido: quitar la cota temporal por completo (perdería la protección de la clase `System::new_all()` → 11,17 s de `MCP-48`), usar `sleep`, `#[ignore]`, retries, o tocar `vantadb-mcp/src/**` (si hiciera falta un contador nuevo en producción, es otro ticket).

## Diagnóstico (root cause)

El test afirmaba **dos relojes distintos** bajo una sola aserción:

| Reloj | Origen | Coste medido (esta máquina) |
|---|---|---|
| Presupuesto de reintento | `RESOLVE_ATTEMPTS = 3` sondas + backoff 50/150 ms (`proxy.rs:46-48`) | ~1,9 s — cada sonda paga **dos** `is_pid_alive` (`try_proxy` + `cleanup_stale`) a ~300 ms cada uno en Windows |
| Paciencia de lock de la promoción | `Config::file_lock_timeout_ms`, **1000 ms por defecto**; el test pasaba `promote_config: None` → `Config::default()` (`proxy.rs:604`) | ~1,0 s |

La aserción `elapsed < 5 s` cubría la suma de ambos **más la carga ambiente**: `MCP-44` la midió en **5,05 s** bajo carga — 50 ms de margen. No era flake de producto: era una aserción que medía la máquina.

## Spec (SDD — Phase 1b)

No aplica: el diff **no** agrega símbolos públicos, endpoints, tools MCP ni métodos de binding. Es un fix test-only (`vantadb-mcp/tests/mcp_proxy_reconnect.rs` + este task file). Los observables que usa —`resolve_failures()`, `reconnects()`, `role()`, `connected()`— ya existen desde `MCP-48`.

## Invariantes de dominio (handoff — MUST)

1. **Promover sobre un writer vivo sigue siendo imposible por construcción:** quien manda es el lock `fs2`, no el discovery file. Un segundo `open` con el lock tomado debe seguir devolviendo `DatabaseBusy`.
2. **El reintento sigue acotado:** 3 sondas + backoff + **una** promoción. La red temporal de 10 s existe para que una regresión de esa cota (la clase `System::new_all()`, 11,17 s) no pase en verde.
3. **Cero cambios en `vantadb-mcp/src/**` y `src/**`**: el commit toca un test y docs.
4. **El discovery file de un PID vivo no se borra** (lo ilegible tampoco): invariante de `cleanup_stale` que el test sigue cubriendo en `stale_discovery_cleanup_is_pid_only`.

## Deuda técnica (Regla 6)

**Saldo neto:** negativo — se elimina la aserción de reloj como contrato y se ganan dos observables (`resolve_failures() == 1`, `!connected()`) sin añadir código de producto.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato cumplido: 20/20 corridas bajo carga + archivo de test completo verde |
| **Commit** | Commit atómico `test(mcp): …`, diff limpio (test + task file + avance + backlog), verificación mecánica |
| **Release** | N/A — no toca producto ni packaging |

## Steps

### Step 1: Pinchar la paciencia de lock y afirmar el mecanismo
- **Archivos:** `vantadb-mcp/tests/mcp_proxy_reconnect.rs`
- **Acción:** `WriterProxy::new(&path, None, Some(Config { file_lock_timeout_ms: 50, ..Default }))` (50 ms sigue cubriendo la escalera 5/10/20/40 ms de `acquire_db_lock`, así que el camino `DatabaseBusy` se ejerce de verdad). Añadidas: `resolve_failures() == 1`, `!connected()`, `DatabaseBusy` con la **misma** config (sin pagar 1000 ms de paciencia en el chequeo). Reloj: `5 s → 10 s` como red, con el número medido al lado.
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp -E 'test(live_writer_lock_blocks_promotion)' --build-jobs 2`
- **Estado:** ✅ DONE

### Step 2: Medir el reparto del reloj (antes de elegir el número)
- **Acción:** instrumentación temporal del `elapsed` y corridas con/sin carga.
- **Resultado:** con `file_lock_timeout_ms: 50` → **1.931 / 1.944 / 1.971 / 2.001 / 2.061 / 2.098 ms** (6 corridas con 6 jobs quemando CPU). El delta contra el default (1000 ms) confirma el reparto: ~1,0 s de paciencia de lock + ~1,9 s de sondas (cada sonda paga dos `is_pid_alive` a ~300 ms en Windows). Instrumentación **eliminada** del código final.
- **Estado:** ✅ DONE

### Step 3: 20 corridas consecutivas bajo carga
- **Verify:** bucle de 20 con 6 `Start-Job` quemando CPU.
- **Resultado:** **20/20 sin fallo de test** — 19 ejecutadas verdes + 1 corrida (`run 3`) que **no llegó a ejecutar**: `vantadb-mcp/tests/l0_l3_cycle_e2e.rs` (otro carril, a medio escribir) rompió el build con `error[E0425] cannot find type Arc`; al compilar de nuevo, el mismo filtro siguió verde. Sumadas 4 corridas extra de confirmación (`run 21-24`): **23 ejecuciones verdes, 0 fallos**.
- **Estado:** ✅ DONE

### Step 4: Falsabilidad (mutante de escenario)
- **Acción:** apuntar el `StorageEngine` del setup a `{path}-nolock` (el escritor deja de retener el lock) y correr el filtro; revertir después.
- **Resultado:** el test **cae** con `unreachable writer must surface an error, not a promotion: Object {...}` — la llamada **promueve** en vez de fallar cerrado. Muere en la aserción pre-existente de BUSY (`expect_err`), no en las nuevas: el mutante prueba que el **escenario** es falsable, y el complemento de las aserciones nuevas (rol `writer-promoted`, `reconnects()==1`) lo fija `proxy_re_elects_writer_without_restart` en el mismo archivo. No se sobrevende: no hay contador por sonda que mutar (ver §Incógnitas).
- **Estado:** ✅ DONE (revertido; `git diff` del archivo = solo el cambio intencional)

### Step 5: Archivo de test completo + cierre
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp --test mcp_proxy_reconnect --build-jobs 2` → **4 tests run: 4 passed** [6.438 s] · `rustfmt --edition 2021 --check` → exit 0 · `cargo clippy -p vantadb-mcp --tests -- -D warnings` → exit 0 · `check-docs` → `GATING: all clear` · `check-links` → 0 markdown roto
- **Estado:** ✅ DONE — commits locales `f097c5f78` (test) + `81e7143ac` (cierre docs)

## Resultado

- **20/20 corridas del bucle sin un fallo de test** (+4 extra = 23 ejecuciones verdes, 0 fallos), con 6 jobs de CPU en paralelo. Detalle y la única corrida que no ejecutó (build ajeno) en §Step 3.
- **Mecanismo elegido:** (a) conteo de resoluciones (`resolve_failures() == 1`, el observable que dejó `MCP-48`) + (b) estado final (rol `proxy`, `reconnects() == 0`, `connected() == false`, `DatabaseBusy` en un segundo `open`) + pinche del reloj que el test no controlaba (`file_lock_timeout_ms: 50` en vez del default 1000 ms).
- **Por qué conserva la cota:** la red sigue ahí, pero gruesa y **justificada por medición** — `5 s → 10 s`. Medido 1,93-2,10 s con carga (×~5 de margen); el peor número *con carga* de esta clase es 5,05 s (`MCP-44`, que además pagaba los 1000 ms que este test ya no paga) y la regresión que la red protege es 11,17 s. Sin la red, la clase `System::new_all()` quedaría en verde.

## Dependencias
- `MCP-48` ✅ (`d38975eb9`: introdujo `reconnects()`/`resolve_failures()`) · `MCP-44` ✅ (`2fb20c728`: midió el fallo) · `FIND-724` ✅ (patrón señal-vs-reloj). Carril R (1 de 2 slots).

## Incógnitas / deuda observada y no tocada
- **`is_pid_alive` cuesta ~300 ms por llamada en Windows** (refresco dirigido por PID), y cada sonda paga **dos** (`try_proxy` + `cleanup_stale`) ⇒ una resolución completa son ~1,9 s. El `<0,3 s` que anota `MCP-48` en `avance/activo/bindings.md:1684` describe el camino de reconexión que acierta en la primera sonda, no esta resolución de 3 sondas con PID vivo; conviene medirlo antes de citarlo como número de este camino. Reducirlo es cambio de producción (`vantadb-mcp/src/**`, prohibido en este ticket) → **derivada a registrar por el owner**.
- Los 1000 ms de paciencia por defecto siguen siendo el coste del caso patológico documentado en `promote()` (`proxy.rs:570-574`). El test ya no los paga porque no es lo que verifica.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (tier fast: el diff toca tests + docs, ningún path de la tabla adversarial)
- **Enfoque:** ¿el test sigue afirmando el invariante o se volvió siempre-verde? ¿la red de 10 s conserva la protección de 11,17 s?
- **Veredicto:** ver §Review abajo / entrada de `avance`
