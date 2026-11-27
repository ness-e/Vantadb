---
title: "FIND-783: una sola resolución del vanta-ingest.toml, y el camino --mcp deja data_dir()"
kind: task
status: active
description: "Tres copias de la resolución del vanta-ingest.toml (server, mcp wiki, mcp threads). MCP-43 aterrizó antes de ADR-0063 y la del camino de producción (--mcp) usa data_dir(), que resuelve a CWD con InMemory. Una resolución en vanta-memory, tres llamadas, test de la propiedad CWD."
tags: [vantadb, vanta-memory, mcp, server, config, bug-fix]
---

# FIND-783 — una sola resolución del `vanta-ingest.toml`

Origen: contrato de `WIRE-19` (2026-11-26), verificado por `vanta-arch` al
contrastar [ADR-0063](../architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md)
contra el código. Detalle ejecutable de la mudanza completa (9 filas):
[WIRE-19-impl](./WIRE-19-impl.md). **Esta tarea es el corte de la resolución de
ruta** (filas 1, 3-parcial, 5, 8-parcial, 9 de ese contrato), no la mudanza
entera del intervalo ni de la fábrica.

## Contrato

1. **Una** resolución de TOML, y las tres llamadas a ella. La de producción
   (`--mcp`, scheduler) **no** puede usar `data_dir()`: el contrato de
   `WIRE-19`/`ADR-0063` exige que `--mcp` resuelva desde la config declarada, no
   desde el CWD.
2. No cambiar semántica de defaults en silencio: las discrepancias medidas se
   deciden y se documentan abajo.
3. Contrato mecánico: `cargo nextest run --profile audit -p vantadb-mcp -p
   vanta-memory --build-jobs 2 -E 'test(mcp_init) or test(ingest) or
   test(config) or test(scheduler)'` **con aserción de población** (`cargo
   nextest list` con el mismo filtro > 0 — un `-E` que matchea 0 tests sale
   exit 0, `FIND-794`).
4. Test nuevo que pruebe **la propiedad**: con `--mcp` y un CWD distinto del
   `data_dir`, la config se resuelve igual (escenario `InMemory`/CWD).

## Impacto mapeado (Regla 0)

| Qué | Medición |
|---|---|
| Copias de la resolución (construcción de la ruta) | **3**: `vantadb-server/src/scheduler.rs:83` (`<storage_path>/data/…`, regla del ADR), `vantadb-mcp/src/wiki.rs:394` (`data_dir()/…`), `vantadb-mcp/src/threads.rs:367` (`data_dir()/…`, camino de **producción** del `--mcp`) |
| Lecturas del env | **3**: `env::var("VANTADB_INGEST_CONFIG")` en `scheduler.rs:93`, `wiki.rs:389`, `threads.rs:363` |
| Definiciones `fn ingest_toml_path` | **2**: `scheduler.rs:91` (`pub`), `wiki.rs:388` (privada) |
| Callers de `threads::ingest_runner_cfg` | **2**: `memory_scheduler.rs:94` (producción), `threads.rs:354` (`note_runner_state_once`) |
| Callers de `wiki::ingest_toml_path` | **1**: `wiki.rs:291` (tool `wiki_ingest`) |
| Callers de `scheduler::ingest_toml_path` / `ingest_runner_cfg` | `main.rs:89` (`--http`); `ingest_toml_path` no tiene caller directo fuera del módulo (`ingest_runner_cfg` lo usa) |
| Semántica `data_dir()` | `<storage_path>/data` con backend de disco (`src/storage/engine/init.rs:459`), `PathBuf::new()` con `BackendKind::InMemory` (`init.rs:425-428`) ⇒ `join` sobre vacío = **relativo al CWD** |
| `config().storage_path` en el camino `--mcp` | Autoritativo: `vantadb-server/src/main.rs:72` pasa `Some(config)` a `run_stdio_server_auto`, y `config.storage_path` es la raíz. **Pero** `StorageEngine::open(path)` no escribe `path` en config (`init.rs:29`, default `vantadb_data`; ver `vantadb-mcp/src/server.rs:190-193`) ⇒ no se puede usar a ciegas |
| Referencias entrantes | `memory_scheduler::start` (producción), `threads::note_runner_state_once`, tool `wiki_ingest`, `vantadb-server::scheduler::*` (consumido por `main.rs` y `vantadb-pro`, fuera del workspace) |
| Referencias salientes | `IngestRunnerCfg::from_env_toml` (ya centralizado, `vanta-memory/src/ingest/runner_config.rs:227`) |
| Veredicto | Superficie pública afectada: **cero** eliminaciones (`vantadb-server` conserva sus seis nombres vía `pub use`/shims). Riesgo concentrado en la regla de ruta; nada de WAL/storage/index. |

## Decisiones medidas (contrato #2)

| Discrepancia | Medición | Decisión |
|---|---|---|
| Valor del override: ¿trim? | `scheduler.rs:79` hace `map(str::trim).filter(...)` y devuelve el valor **trimeado**; `wiki.rs:389-392` y `threads.rs:363-367` solo trimean para el *blank-check* y devuelven el valor **crudo** | **Trim** (2 de 3 copias no trimean, pero el ADR fija "non-blank after `trim()`" y el server es la regla que el ADR describe; un path con espacios en los extremos es un typo de operador). Sin cambio de default: blank ⇒ fallback en las tres |
| Ruta del fallback | server: `<storage_path>/data/vanta-ingest.toml`; mcp: `data_dir()/vanta-ingest.toml` | **`<storage_path>/data/…`** (ADR-0063). Idéntica byte a byte en backend de disco; distinta en `InMemory` (que es el bug) |
| Raíz del storage desde un engine vivo | `data_dir()` es autoritativo con disco; `config().storage_path` lo es con `InMemory` (no hay `data_dir`) | `data_dir().parent()` si no está vacío, si no `config().storage_path` — el idioma que ya existe en `vantadb-mcp/src/server.rs:190-202`, para **no** reintroducir el bug al revés (usar `config().storage_path` a secas cambiaría la ruta en backends abiertos con `StorageEngine::open(path)`) |

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | `vanta-memory/src/ingest/runner_config.rs`: `resolve_ingest_toml_path` (privado) + `pub fn ingest_toml_path` + `IngestRunnerCfg::from_storage_path` + `::from_storage` + tests movidos | ✅ |
| 2 | RED: test de la propiedad en `vantadb-mcp/src/threads.rs` (`InMemory` + CWD distinto) | ✅ |
| 3 | `vantadb-mcp/src/wiki.rs`: borrar la copia, llamar la compartida | ✅ |
| 4 | `vantadb-mcp/src/threads.rs` + `memory_scheduler.rs`: borrar la copia, llamar la compartida | ✅ |
| 5 | `vantadb-server/src/scheduler.rs`: borrar la copia, `pub use` de `vanta-memory` (nombres preservados) | ✅ |
| 6 | Verify: fmt + clippy + nextest del contrato (con población) + las 4 métricas `rg` | ✅ |
| 7 | Review P2-01 + OCR + commit local + fila de Backlog + avance | ✅ |

## Review (P2-01)

- **Tier:** Fast (ningún path del diff está en la tabla adversarial; todos son `vanta-memory/src/ingest/**`, `vantadb-mcp/src/**`, `vantadb-server/src/scheduler.rs`, docs).
- **Reviewer:** `vanta-review`, contexto fresco, 1 ronda → **approve**, 0 Blocking.
- **Medium aplicado:** el test del server llamaba el wrapper que lee env (`VANTADB_INGEST_CONFIG` exportado lo habría puesto rojo por una razón ajena a la regla). Reemplazado por un test de **superficie** (los seis nombres se siguen resolviendo; no lee env).
- **Lows declarados, no pagados:** (a) el test de la propiedad no distingue "solo declarado" de "declarado-o-CWD" — no hay decoy en el CWD porque escribirlo en la raíz del repo es un side effect global con tests en paralelo; el RED ya midió que el camino CWD lee **nada** (`12000` vs `1234`); (b) `Config::in_memory()` deja la raíz en `:memory:` ⇒ `:memory:/data/vanta-ingest.toml`, una ruta sin fichero posible (mismo efecto que hoy: defaults; declarado abajo); (c) `to_string_lossy` es inocuo porque la raíz viene de un `&str` (nunca no-UTF8), pero no lo demuestra el tipo; (d) `vantadb-mcp/src/server.rs:190-202` deriva la raíz con el mismo idioma — debería usar `from_storage` (fuera del alcance de esta fila).
- **Evidencia del reviewer:** re-corrió las tres métricas de cierre; verificó `init.rs:313-315,425-428,459` (parent de `data_dir` = raíz declarada, sin canonicalizar) y `server.rs:1050-1057` (`--mcp` siempre abre con `Some(cfg)` ⇒ `data_dir` autoritativo en disco); confirmó los seis nombres vivos en `vantadb_server::scheduler` y cero `unwrap`/`expect` en código no-test.

## Deuda declarada

| Qué | Por qué no se paga acá |
|---|---|
| `:memory:` como raíz | `Config::in_memory()` no tiene raíz declarada; la regla del ADR produce una ruta sin fichero (defaults, igual que antes). Una guarda sería una regla nueva, no una mudanza |
| Raíz derivada dos veces (`vantadb-mcp/src/server.rs:190-202`) | Es el *path* del discovery file, no el TOML de ingest; tocarlo cambia el arranque del proxy |
| Fábrica de runner e intervalo siguen en el host | Es `WIRE-19-impl` filas 4, 8 y 2; esta fila es sólo la resolución de ruta |
| `vantadb-server/src/scheduler.rs` fuera del allowlist del prompt | El contrato de la fila pide **una** resolución consumida por **los tres** sitios y la métrica `join(...)` = 1; sin tocar ese fichero quedaban 2. No está en la lista de PROHIBIDO y `WIRE-19-impl` §2.1 fila 3 ya prescribe exactamente este cambio (borrar el privado, `pub use`, seis nombres preservados) |

## Verificación

```bash
cargo nextest list --profile audit -p vantadb-mcp -p vanta-memory --build-jobs 2 \
  -E 'test(mcp_init) or test(ingest) or test(config) or test(scheduler)'   # > 0
rg -n 'join\("vanta-ingest\.toml"\)' --glob '*.rs'                        # 1 (era 3)
rg -n 'fn ingest_toml_path\(' --glob '*.rs'                               # 1 (era 2)
rg -n 'env::var\("VANTADB_INGEST_CONFIG"\)' --glob '*.rs'                 # 1 (era 3)
```

**Población medida:** `cargo nextest list` con el filtro del contrato → **46**
(era 44 antes de los dos tests nuevos). El run del contrato arranca **46 tests**
(no 0: `FIND-794`), `46/46` verdes.

**Suites completas (red de regresión, el filtro salta 923 tests):**
`-p vantadb-mcp -p vanta-memory -p vantadb-server` → **980/981**. La única roja
es `vantadb-server::git_commit_declared::version_declares_the_head_commit`, que
compara el commit horneado en el binario con `HEAD`: otro carril commiteó
(`4a1c224a5` → `159640800`) **durante** la corrida. No toca ningún path de este
diff.

**Claves ajenas en el worktree:** `docs/dev/plans/…budget.json`,
`docs/dev/tasks/H-02-H-07.md`, `docs/index.md`, `llms.txt` — de otros carriles,
fuera del commit (FIND-792).
