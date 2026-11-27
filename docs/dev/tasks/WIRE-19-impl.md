---
title: "WIRE-19-impl: contrato ejecutable — una sola regla de config de ingest para los tres caminos del host"
kind: task
status: active
description: "Implementación de ADR-0063: la superficie FIND-112 se muda a vanta-memory (7 filas del ADR) MÁS las 2 copias que MCP-43 dejó al aterrizar antes (memory_scheduler.rs, threads.rs). Tabla origen→destino, deprecación, contrato de una sola fábrica."
tags: [vantadb, vanta-memory, mcp, server, crate-boundary, contract-first, executable]
---

# WIRE-19-impl — el wiring del host, en filas ejecutables

Decision: [ADR-0063](../architecture/adr/ADR-0063-host-wiring-home-vanta-memory.md) (`status: accepted`).
Analysis: [WIRE-19](./WIRE-19.md) (`status: complete` — spec cerrada).
Implementation: **parcial** — `FIND-783` (`85c972227`) aterrizó las filas 1, 5 y 9 sin anotarlo, y
**desvió** las filas 4/8 a un constructor distinto del que este fichero prescribe (§8.4, medido).
Estado pieza por pieza y partición en §0.1.

**Este fichero no re-decide nada**, con **una** excepción declarada y acotada en §8.0:
`MCP-43` aterrizó antes que esta tarea y dejó dos copias nuevas; el alcance crece de 7 a
9 filas, y una de ellas toca un default que ya está en producción. Eso **no** es una
decisión del implementador: §8.0 la fija con recomendación y marca qué necesita
confirmación del owner antes del merge.

---

## §0 — Qué significa "hecho"

Una sola frase falsable: **hay exactamente un sitio en el árbol que resuelve el TOML de
ingest y una sola fábrica de runner, y los tres caminos del host la consumen.**

**Hoy hay tres caminos, no dos** — este es el hallazgo que cambia la tarea. **Estado a
2026-11-27: la tabla describe el árbol ANTES de `FIND-783`, conservada como registro del
hallazgo; el estado vivo está en §0.1.**

| # | Camino | Resuelve el TOML | Fábrica de runner | Intervalo |
|---|---|---|---|---|
| 1 | `--http` (`vantadb-server/src/main.rs:88-95`) | ~~`scheduler.rs:91-96`~~ → **hoy** `runner_config.rs:143` vía el shim `scheduler.rs:85` | ~~`scheduler.rs:108-115`~~ → **hoy** `scheduler.rs:94-101` (sigue en el server) | `scheduler.rs:67-73`, default **60 000** (`:45`) |
| 2 | `--mcp`, tool `wiki_ingest` (`vantadb-mcp/src/wiki.rs:292`) | ~~`wiki.rs:388-395`~~ → ✅ **borrada**; `from_storage(storage)` | `build_ingest_runner` inline (`:294`) | — |
| 3 | `--mcp`, **scheduler de producción** (`vantadb-mcp/src/memory_scheduler.rs:92-104`) | ~~`threads.rs:362-369`~~ → ✅ **borrada**; `from_storage(storage)` (`:97`) | `build_ingest_runner` inline (`:98-104`) | `memory_scheduler.rs:68-83`, default **5 000** |

Los tres diferían en algo. En backend de disco los caminos 1 y 2/3 coincidían **por
accidente** (`data_dir == <root>/data`, `src/storage/engine/init.rs:459`); en
`BackendKind::InMemory` `data_dir` es `PathBuf::new()` (`init.rs:425-428`), así que los
caminos 2 y 3 resolvían un `vanta-ingest.toml` **relativo al CWD** y el 1 el del
storage. `FIND-783` cerró eso con un constructor (`from_storage`) que no depende de
`data_dir` existiendo (§8.4). El comentario que lo admitía ya no existe en el árbol:

> *"Duplicated because that helper is private; both must stay in sync (same host config,
> one meaning)."* — `threads.rs:357-361` **en `HEAD~`**; `rg` sobre `HEAD` devuelve **0**.

Es exactamente la clase de drift de `FIND-704`, documentada como intención.

**Lo que esta tarea NO arregla, y hay que decirlo:** un MCP en modo **proxy** (el
proceso que pierde el writer y cae a `run_proxy_stdio_server`,
`vantadb-mcp/src/server.rs:981,1078-1109`) **no ve la cola del writer** — la cola
(`LocalStateBackend`) es in-process y vive en el otro proceso. Compartir la fábrica de
runner no cambia eso: en proxy no debe haber scheduler en absoluto. Es la guarda de
`MCP-43`, no de `WIRE-19` (§4.3).

---

## §0.1 — Estado medido contra `HEAD` (2026-11-27, tras `85c972227`)

Medido con `rg`/lectura, **sin compilar**. Este bloque es el que manda: si contradice una fila de
§2, la fila está desactualizada.

| Pieza | Estado | Dónde vive hoy |
|---|---|---|
| `resolve_ingest_toml_path` (privado) | ✅ **movida** | `vanta-memory/src/ingest/runner_config.rs:126` |
| `ingest_toml_path` (`pub fn`) | ✅ **movida, 1 sola definición** | `vanta-memory/src/ingest/runner_config.rs:143`; el server la re-exporta (`vantadb-server/src/scheduler.rs:82`) |
| `IngestRunnerCfg::from_storage_path` | ✅ **existe, y sí se usa** | `runner_config.rs:278`. Ningún host la llama **directamente**: `main.rs:89` entra por el shim `scheduler.rs:85` → `:86`. **No la borres por "muerta"** (ver §8.4) |
| `IngestRunnerCfg::from_storage` (**no está en el ADR**) | ✅ **añadida por `FIND-783`** | `runner_config.rs:295`; la usan los 3 hosts |
| `IngestRunnerCfg::runner_factory` | ❌ **no existe** | la lógica del P4 sigue en el `fn` libre `vantadb-server/src/scheduler.rs:94-101`, **y duplicada** en `vantadb-mcp/src/memory_scheduler.rs:98-104` |
| `DEFAULT_INTERVAL_MS` | ❌ **no movida** | `vantadb-server/src/scheduler.rs:45` (y el default del host, `memory_scheduler.rs:44`) |
| `parse_interval_ms` | ❌ **no movida** | `vantadb-server/src/scheduler.rs:49` |
| `scheduler_interval_ms_from_env_opt` | ❌ **no existe en ningún sitio** | — |
| `scheduler_interval_ms_from_env` (reescrita como wrapper de `_opt`) | ❌ **no reescrita** | `vantadb-server/src/scheduler.rs:67` |
| Re-export de los 6 nombres | 🟡 **1 `pub use` + 2 shims; los 3 del intervalo siguen definidos en el server** | `pub use`: `ingest_toml_path` (`scheduler.rs:82`); shims: `ingest_runner_cfg` (`:85`), `ingest_runner_factory` (`:94`). Los tres del intervalo no pueden re-exportarse hasta mover la pieza (`WIRE-20`/`WIRE-22`) |
| `wire_memory` en `run_stdio_server` + guardas | 🟡 **la composición no es `wire_memory`; las guardas sí están, el one-shot no** | el MCP compone con `memory_scheduler::start` (`vantadb-mcp/src/server.rs:1064,1120`; `proxy.rs:659,712`), no con `wire_memory` (0 usos fuera de `vantadb-server`). `read_only` ✅ — pero sobre el **clon pre-open** (`server.rs:1056` → guarda en `:1063,1119`), no sobre `storage.config().read_only` como literalmente pide `ADR-0063:364-365`; equivalente en la práctica porque `cfg_with_path` es el `Config` con que se abrió, pero no es la misma expresión. Host one-shot ❌ (`memory_scheduler.rs:69-70` ⇒ 5 000 ms con la variable **sin definir** ⇒ loop **ON**; el ADR D4 pide OFF, ver §8.0) |
| Superficie `pub` añadida en `vanta-memory` | 🟡 **+3 de +7** (y una de más) | `ingest_toml_path`, `from_storage_path`, `from_storage` ⇒ el diff real de `cargo public-api` será **+8**, no +7: `from_storage` no estaba en el ADR |

Métricas de cierre que ya están en verde (las mismas de §4.2/§6, re-medidas):

```text
rg -n 'fn ingest_toml_path' --glob '*.rs'             → 1 definición (+1 test en :479)
rg -n 'join\("vanta-ingest.toml"\)' --glob '*.rs'      → 1  (esperado 0: el `join` ahora vive DENTRO del dueño)
rg -n 'env::var\("VANTADB_INGEST_CONFIG"\)' --glob '*.rs' → 1
rg -n 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' --glob '*.rs' → 2  (server :69 + memory_scheduler :69) ⇒ R-5 sigue violada
rg -n 'fn ingest_runner_cfg\(' --glob '*.rs'           → 1  (el shim del server)
```

> **`join("vanta-ingest.toml")` pasó de "0 esperado" a "1 aceptado":** §4.2 pedía 0 porque el
> objetivo era que ningún *host* construyera la ruta. Ahora la construye el dueño, una vez. La
> métrica correcta sigue siendo `0` **fuera de** `vanta-memory/src/ingest/runner_config.rs`.

---

## §1 — Alcance de escritura

| Fichero | Qué cambia | Fila |
|---|---|---|
| `vanta-memory/src/ingest/runner_config.rs` | **+5 items** (`resolve_ingest_toml_path` privado, `ingest_toml_path`, `IngestRunnerCfg::from_storage_path`, `IngestRunnerCfg::from_storage`, `IngestRunnerCfg::runner_factory`) + 3 tests movidos. **Hecho: los 4 primeros (`FIND-783`); falta `runner_factory`** | 1 |
| `vanta-memory/src/services/scheduler.rs` | **+3 items** (`DEFAULT_INTERVAL_MS`, `parse_interval_ms`, `scheduler_interval_ms_from_env_opt`) y `scheduler_interval_ms_from_env` reescrita como wrapper + 1 test movido | 2 |
| `vantadb-server/src/scheduler.rs` | **borrar** `:44-108` (const, parser, `_from_env`, `resolve`, el `pub use` de `:82`, el shim `ingest_runner_cfg` `:85-87` y `ingest_runner_factory` `:94-101`) y `mod tests` `:139-186` salvo los 2 tests de la fábrica; **añadir** §3.1; recortar imports. ⚠️ **Medido en `HEAD`: `wire_memory` empieza en `:110`, NO en `:117`** — un `:44-115` literal se come sus 6 primeras líneas | 3 |
| `vantadb-server/src/main.rs` | `:88-94` → `from_storage_path` + `cfg.runner_factory()` | 4 |
| `vantadb-mcp/src/wiki.rs` | ✅ **hecho** (`FIND-783`): copia local (`:385-395`) borrada; `:292` llama `from_storage(storage)` (no `from_storage_path` — §8.4) | 5 |
| `docs/user/operations/CONFIGURATION.md` | `:136` y `:137`: el fichero dueño de cada variable — **`:137` ya es falso en `HEAD`** (el dueño es `runner_config.rs:145` desde `FIND-783`) | 6, 7 |
| `vantadb-mcp/src/memory_scheduler.rs` | **borrar el `env::var`** (`:69`) y la fábrica inline (`:98-104`); delegar en la superficie compartida (la mitad de la fábrica ya usa `from_storage`, `:97`) | **8** |
| `vantadb-mcp/src/threads.rs` | ✅ **hecho** (`FIND-783`): `ingest_runner_cfg` borrada (`:362-369`), `:355` usa `from_storage` | **9** |
| `vantadb-server/tests/scheduler_e2e.rs` | **+1 test** que nombra los seis nombres viejos (§4.1) | — |

**No se toca:** `vantadb-mcp/Cargo.toml` (ver §8.1), `vantadb-server/Cargo.toml`
(`llm-driver` se queda — §6), `vanta-proxy/**`, `src/**`, `docs/dev/Backlog.md`, el
`ADR-0063` (§8.0 explica qué enmienda hace falta y por qué no la puede hacer el
implementador), `CHANGELOG.md` de la raíz (§3.3).

---

## §2 — Tabla de migración

Las **7 primeras filas son las del ADR**, sin cambios (salvo las correcciones de
snippet que marcó la review). Las **filas 8 y 9 no están en el ADR**: existen porque
`MCP-43` aterrizó antes (§8.0). Van marcadas.

### 2.1 Filas 1-7 (tabla del ADR-0063)

| # | Origen (fichero:líneas) | Destino | Qué se mueve / añade | Effort | Si se olvida |
|---|---|---|---|---|---|
| **1** | `vantadb-server/src/scheduler.rs:76-85` (privado) · `:87-96` · `:103-115` · tests `:161-192` (3) | `vanta-memory/src/ingest/runner_config.rs`, junto a `from_env_toml` (`:227`) | `resolve_ingest_toml_path` **verbatim y privado**; `pub fn ingest_toml_path` **verbatim**; `IngestRunnerCfg::from_storage_path` (**nuevo**, delegación pura: `from_env_toml(&ingest_toml_path(p))`); `IngestRunnerCfg::runner_factory` (**nuevo**, ver §2.2) + los 3 tests | 2 h | nada de los pasos 2-5 compila |
| **2** | `vantadb-server/src/scheduler.rs:44-46` · `:48-65` · `:67-74` · test `:152-159` | `vanta-memory/src/services/scheduler.rs`, **por encima** del primer `#[cfg(feature = "http-server")]` | `DEFAULT_INTERVAL_MS`, `parse_interval_ms` **verbatim**; `scheduler_interval_ms_from_env_opt() -> Option<u64>` (**nuevo**, único read site del env); `scheduler_interval_ms_from_env()` → `_opt().unwrap_or(DEFAULT_INTERVAL_MS)`. **Reescribir el doc de `DEFAULT_INTERVAL_MS`**: hoy cita `src/config.rs`, y desde este crate debe citar `vantadb/src/config.rs` (R-4) | 1 h | el parser de intervalo no tiene hogar sin gate; el doc cita una ruta que no existe en el crate |
| **3** | `vantadb-server/src/scheduler.rs:44-108` + `mod tests` `:139-186` | mismo fichero, reducido | **Borrar** esos rangos; conservar el doc de módulo (L1-31, re-escrito: "el host compone; la superficie vive en `vanta_memory`") y `wire_memory` (**`:110-132` en `HEAD`**); **añadir** §3.1. Imports finales — ver §2.3, **no** es la lista que parecía | 1 h | definiciones duplicadas ⇒ `cargo check` falla; un shim que falte rompe `scheduler::*` en silencio |
| **4** | `vantadb-server/src/main.rs:88-94` | mismo fichero | `scheduler_interval_ms_from_env()` sin cambios; sustituir el par cfg+factory por `from_storage_path` + `cfg.runner_factory()` — ver §2.4 | 0.25 h | el hogar canónico queda invisible ⇒ la copia vuelve |
| **5** | `vantadb-mcp/src/wiki.rs:385-395` · `:291` | mismo fichero | **Borrar** `fn ingest_toml_path`; `:291` → `IngestRunnerCfg::from_storage(storage)` — **no** `from_storage_path(&storage.config().storage_path)` (§8.4: ese constructor reintroduce el bug al revés). **`data_dir()` no es el parámetro**: ya es `<root>/data`, un nivel de más | 0.5 h | la divergencia `InMemory`/CWD sobrevive en el tool |
| **6** | `docs/user/operations/CONFIGURATION.md:136` | mismo fichero | `VANTADB_SCHEDULER_INTERVAL_MS`: → `(vanta-memory/src/services/scheduler.rs)` | 0.15 h | R-4: ruta citada que ya no contiene la variable |
| **7** | `docs/user/operations/CONFIGURATION.md:137` | mismo fichero | `VANTADB_INGEST_CONFIG`: → `(vanta-memory/src/ingest/runner_config.rs)` | 0.15 h | R-4 en la fila de config |

### 2.2 Filas 8-9 (**no están en el ADR** — las dejó `MCP-43` al aterrizar antes)

| # | Origen (fichero:líneas) | Destino | Qué se cambia | Effort | Si se olvida |
|---|---|---|---|---|---|
| **8** | `vantadb-mcp/src/memory_scheduler.rs:64-83` (`interval_ms_from_env`) · `:92-101` (`start`) | mismo fichero, delegando en `vanta-memory` | **Intervalo**: borrar el `std::env::var` de `:69`. Deja el nombre como wrapper de delegación pura: `fn interval_ms_from_env() -> u64 { vanta_memory::services::scheduler::scheduler_interval_ms_from_env_opt().unwrap_or(DEFAULT_INTERVAL_MS) }`. **`DEFAULT_INTERVAL_MS = 5_000` (`:44`) se queda** como default del host (ver §8.0). **Fábrica**: sustituir `let cfg = crate::threads::ingest_runner_cfg(storage);` + el closure `match build_ingest_runner(&cfg)` por `let factory = IngestRunnerCfg::from_storage(storage).runner_factory();` (§8.4 — **no** `from_storage_path(&storage.config().storage_path)`) y pasar `factory` a `start_with` (que ya acepta `Fn() -> Option<R>`). Imports: `build_ingest_runner` y `ConcreteRunner` (`:34`) probablemente quedan sin uso ⇒ **que lo diga `cargo check`**, no el ojo. El test `interval_env_is_one_knob…` (`:305-328`) **se queda tal cual**: sigue ejercitando el wrapper | 1 h | el camino de **producción** del `--mcp` sigue resolviendo por `data_dir()`, es decir el bug que la tarea existe para cerrar sigue vivo donde más importa |
| **9** | `vantadb-mcp/src/threads.rs:357-369` (`ingest_runner_cfg`) · `:354` | mismo fichero | ✅ **HECHO por `FIND-783`** (`85c972227`): `pub(crate) fn ingest_runner_cfg` borrada; `note_runner_state_once` llama `IngestRunnerCfg::from_storage(storage).build_runner()` (`threads.rs:355`). El `use` de `:23` se queda. Queda **una** constatación, no trabajo: la fila decía `from_storage_path(...)` y la implementación usa `from_storage` (§8.4) | 0 h | — |

**Total: ~6 h de edición + ~2.5 h de verificación (§5) ≈ 1.25-1.5 días.** El ADR estimó
1 día para 7 filas; las dos extra son ~1.5 h.

### 2.3 La forma exacta de `runner_factory` (aquí se rompe si se copia mal)

El original toma `cfg` **por valor** (`scheduler.rs:108-115`). Como método toma
`&self`, así que **tiene que clonar antes de mover**:

```rust
// vanta-memory/src/ingest/runner_config.rs
impl IngestRunnerCfg {
    /// Runner factory from this config: `build_runner` + the P4 degradation
    /// (`Some(ConcreteRunner::None)` → `None`), moved verbatim from the host.
    pub fn runner_factory(&self) -> impl Fn() -> Option<ConcreteRunner> + Send + Sync + 'static {
        let cfg = self.clone();               // REQUIRED: the closure is `move` and owns it
        move || match build_ingest_runner(&cfg) {
            Some(ConcreteRunner::None) => None,
            other => other,
        }
    }
}
```

Sin el `let cfg = self.clone();` no compila (`cannot move out of *self`). `IngestRunnerCfg`
deriva `Clone` (`runner_config.rs:65`).

**Imports finales de `vantadb-server/src/scheduler.rs` (paso 3), corregidos:** `Arc`,
`Duration`, `LlmRunner`, `HttpCaptureBridge`, `spawn_memory_scheduler`,
`LocalStateBackend`, `SystemClock`, `ServerHooks`, `Embedded`. **Fuera:** `Path`,
`PathBuf`, `build_ingest_runner`. **Ojo:** `IngestRunnerCfg` **no** va en la lista de
`use` — lo aporta el `pub use` de §3.1, y un `use` adicional sería **E0252** (defined
multiple times), no un warning. `ConcreteRunner` sólo entra si el shim usa el nombre
corto; el shim de §3.1 usa la ruta completa, así que **tampoco**.

### 2.4 Los dos call sites, escritos

```rust
// vantadb-server/src/main.rs:88-94  (HTTP)
let interval_ms = vantadb_server::scheduler::scheduler_interval_ms_from_env();
let cfg = vantadb_server::scheduler::IngestRunnerCfg::from_storage_path(&storage_path);
vantadb_server::scheduler::wire_memory(hooks, db, interval_ms, cfg.runner_factory());
```

> El `pub use` de §3.1 vive **dentro del módulo `scheduler`**: no introduce nada en el
> scope de `main.rs`, que no tiene ningún `use` propio. Por eso la ruta completa — o
> añadir `use vantadb_server::scheduler::IngestRunnerCfg;`. Un `IngestRunnerCfg::` a
> secas **no compila**.

```rust
// vantadb-mcp/src/wiki.rs:291  (tool wiki_ingest)
let runner_cfg = IngestRunnerCfg::from_storage(storage);   // §8.4: the live engine, not config().storage_path
let pipeline = runner_cfg.pipeline_config();
let runner = build_ingest_runner(&runner_cfg);      // :293 se queda
```

---

## §3 — Qué queda deprecado, y cómo (la respuesta corta: **nada**)

### 3.1 `vantadb-server`: seis nombres preservados, cero `#[deprecated]`

`vantadb-server` es `publish = false` (`Cargo.toml:7`), su lib **no es un contrato** —
pero `vantadb-pro` (repo aparte, fuera del workspace por open-core) consume
`vantadb_server::scheduler::*` y **no se puede inspeccionar desde acá**. La mudanza es
**aditiva**, y el bloque vive **dentro de `vantadb-server/src/scheduler.rs`** (no en
`lib.rs`: el módulo es un fichero, y un `pub use` en la raíz publicaría
`vantadb_server::ingest_runner_cfg`, **no** `::scheduler::…`):

```rust
// vantadb-server/src/scheduler.rs — cuerpo del módulo, no lib.rs
pub use vanta_memory::ingest::runner_config::{ingest_toml_path, IngestRunnerCfg};
pub use vanta_memory::services::scheduler::{
    parse_interval_ms, scheduler_interval_ms_from_env, DEFAULT_INTERVAL_MS,
};

/// Load the ingest runner config from env + TOML (FIND-112 surface).
pub fn ingest_runner_cfg(storage_path: &str) -> IngestRunnerCfg {
    IngestRunnerCfg::from_storage_path(storage_path)
}

/// Production runner factory, built from `cfg`; a degraded config maps to `None` (P4).
pub fn ingest_runner_factory(
    cfg: IngestRunnerCfg,
) -> impl Fn() -> Option<vanta_memory::ingest::runner_config::ConcreteRunner> + Send + Sync + 'static {
    cfg.runner_factory()
}
```

| Nombre viejo | Cómo sobrevive |
|---|---|
| `DEFAULT_INTERVAL_MS` | `pub use` |
| `parse_interval_ms` | `pub use` |
| `scheduler_interval_ms_from_env` | `pub use` |
| `ingest_toml_path` | `pub use` |
| `ingest_runner_cfg` | **shim** — era `fn`, ahora `IngestRunnerCfg::from_storage_path` |
| `ingest_runner_factory` | **shim** — era `fn(cfg)`, ahora método |

Los shims llevan `///`: el crate es `#![warn(missing_docs)]`
(`vantadb-server/src/lib.rs:1`) y clippy corre con `-D warnings`.

**Por qué sin `#[deprecated]`:** un aviso de deprecación sólo sirve si existe una ruta de
migración visible para quien lo recibe. El único consumidor fuera del árbol es
`vantadb-pro`, que no podemos leer ni actualizar en este repo; marcar los nombres
generaría warnings inaccionables en un crate que no podemos arreglar. Si algún día
`vantadb-pro` se puede inspeccionar, ese es el momento del ciclo de deprecación.

### 3.2 Lo que se borra de verdad (no se depreca)

| Qué | Por qué es seguro |
|---|---|
| `resolve_ingest_toml_path` en `vantadb-server` | **privado**, nunca fue superficie. Se **mueve** |
| `fn ingest_toml_path` en `vantadb-mcp/src/wiki.rs` | **privado**, y es la copia que causa la divergencia. Borrarlo *es* el fix |
| `fn ingest_runner_cfg` en `vantadb-mcp/src/threads.rs` | **`pub(crate)`**, 2 callers, ambos actualizados por las filas 8-9 |
| `interval_ms_from_env`'s `env::var` en `memory_scheduler.rs` | el **nombre** sobrevive como wrapper; lo que muere es la lectura del env |
| `mod tests` de `vantadb-server/src/scheduler.rs` | los 4 tests se mueven con su código. No se pierde cobertura: se reubica |

### 3.3 CHANGELOG: no se toca en esta tarea

- `vantadb-server` es `publish = false` ⇒ sin entrada.
- `vanta-memory` **es publicable** (`Cargo.toml:2-3`, sin `publish = false`; DIST-01) y
  gana **+8 items `pub`** ⇒ bajo semver 0.x, **minor**. (El ADR y la versión previa de esta
  fila decían `+7`; el octavo es `from_storage`, que `FIND-783` añadió por la razón medida en
  §8.4.) La entrada le corresponde a
  `[Unreleased] ### Added` de `CHANGELOG.md` (raíz, `:14-16`) y la pone el carril de
  release cuando corte versión (release-plz en pausa por DIST-01).
- Si se quiere adelantar es **una línea**, pero eso **amplía el write-scope** de esta
  tarea: se decide aparte.

**Resumen:** `+8` aditivos en `vanta-memory`, `6/6` nombres preservados en
`vantadb-server` sin marcar, `4` privados borrados/movidos, `0` entradas de CHANGELOG.

---

## §4 — Contrato de verificación: una sola fábrica para todos los caminos

### 4.1 Los seis nombres, ejercitados (la promesa de D3)

`vantadb-server/tests/scheduler_e2e.rs` hoy sólo importa `wire_memory` (`:28`), así que
**un tidy-up futuro borraría un `pub use` y nada se enteraría**. Añade:

```rust
#[test]
fn scheduler_module_keeps_the_six_find112_names() {
    use vantadb_server::scheduler::{
        ingest_runner_cfg, ingest_runner_factory, ingest_toml_path, parse_interval_ms,
        scheduler_interval_ms_from_env, DEFAULT_INTERVAL_MS, IngestRunnerCfg,
    };
    let _ms: u64 = scheduler_interval_ms_from_env();
    let _ = (
        DEFAULT_INTERVAL_MS,
        parse_interval_ms(None),
        _ms,
        ingest_toml_path("/tmp/x"),
        ingest_runner_cfg("/tmp/x"),
        ingest_runner_factory(IngestRunnerCfg::defaults()),
    );
}
```

Dos detalles que rompen este test si se escriben "a ojo": `IngestRunnerCfg` **no
implementa `Default`** (deriva `Debug, Clone` en `runner_config.rs:65`; el idioma es
`IngestRunnerCfg::defaults()` en `:116`), y **nada** de `>= 0` sobre un `u64` (dispara
`clippy::absurd_extreme_comparisons`, deny-by-default, y el paso 6 queda rojo con
`-D warnings`). Es un test de **compilación**: su valor es que borrar un re-export deja
de compilar.

### 4.2 Un solo read site — medido sobre `env::var`, no sobre menciones

Los comandos anteriores contaban **menciones** (comentarios, logs y tests nombran las
variables), así que nunca podían dar 1. Los correctos apuntan a la lectura y a la
construcción de la ruta:

```bash
rg -n 'env::var\("VANTADB_INGEST_CONFIG"\)' --glob '*.rs'          # exactamente 1
rg -n 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' --glob '*.rs'  # exactamente 1
rg -n 'fn ingest_toml_path' --glob '*.rs'                          # exactamente 1
rg -n 'fn ingest_runner_cfg\(' --glob '*.rs'                       # exactamente 1 (el shim del server)
rg -n 'join\("vanta-ingest.toml"\)' --glob '*.rs'                  # exactamente 0
```

Estado **hoy**, para contraste (medido): `VANTADB_INGEST_CONFIG` se lee en **3** sitios
(`scheduler.rs:93`, `wiki.rs:389`, `threads.rs:363`); `VANTADB_SCHEDULER_INTERVAL_MS` en
**2** (`scheduler.rs:70`, `memory_scheduler.rs:69`); `fn ingest_toml_path` **2**
(`scheduler.rs:91`, `wiki.rs:388`); `fn ingest_runner_cfg(` **2** (`scheduler.rs:99`,
`threads.rs:362`); `join("vanta-ingest.toml")` **2** (`wiki.rs:394`, `threads.rs:367`).

> **No uses `rg 'data_dir\(\)'` como métrica.** Hay 5 usos legítimos en `vantadb-mcp/src`
> que **no** son la resolución del TOML: `server.rs:194,198` (derivar la raíz del storage)
> y `handlers/tools.rs:3282` (padre del `data_dir`). La métrica correcta es la
> construcción de la ruta (`join("vanta-ingest.toml")`), no el accesor.

> **No uses `rg 'fn ingest_runner_cfg'` sin el paréntesis.** Casa también con cuatro
> nombres de test en `vanta-memory/src/ingest/runner_config.rs:358,371,383,392`
> (`ingest_runner_cfg_defaults_to_local_p4`, …) y te dará un falso rojo.

R-5 prohíbe **el segundo read site**, no un default documentado por host: el `Option` de
`scheduler_interval_ms_from_env_opt` es la primitiva, y cada host pone su default con
`unwrap_or` (§8.0).

### 4.3 La fábrica **es** la misma — y lo que eso no implica

| Afirmación | Cómo se verifica | Estado tras esta tarea |
|---|---|---|
| los tres caminos resuelven el TOML con la **misma** regla | §4.2 (una sola `ingest_toml_path`) | ✅ |
| los tres construyen el runner con la **misma** fábrica | §4.2 + filas 4 (`from_storage_path(&storage_path)`, sólo `&str`) y 5/8 (`from_storage(storage)`, engine vivo) usando `.runner_factory()` — §8.4 explica por qué **no** son el mismo constructor | ⬜ falta `runner_factory` |
| `InMemory`: los tres dan la raíz declarada, no CWD | test de §4.4 — **ya existe y está verde** (`FIND-783` lo dejó RED→GREEN; ver el avance de `vanta-memory`). Vuelve a rojo si alguien cambia `from_storage` por `from_storage_path(&config.storage_path)` | ✅ |
| los tests de ingest del MCP siguen degradando igual (P4) | `cargo nextest run -p vantadb-mcp -E 'test(ingest_local_no_model_degrades) or test(ingest_ollama_down_degrades) or test(ingest_openai_no_key_degrades)'` | ✅ debe seguir verde |
| el scheduler del `--mcp` sigue arrancando igual | `memory_scheduler`'s `interval_env_is_one_knob…` (`:305-328`) + el test de la ola (`:330+`) | ✅ sin cambios de comportamiento |
| **un MCP en proxy ve la cola del writer** | — | ❌ **NO**. La cola es in-process: el proxy (`server.rs:981,1078-1109`) vive en otro proceso. Compartir la fábrica no cambia eso; la respuesta es la guarda de `MCP-43`, no esta mudanza |

Esa última fila está aquí a propósito: es la lectura equivocada que un "unificado ✅"
invita. `WIRE-19` unifica **configuración**, no **estado**.

### 4.4 El test que mata la divergencia

En `vantadb-mcp`, sobre `BackendKind::InMemory`, con `VANTADB_INGEST_CONFIG` sin definir:

```rust
// El operador escribe UN vanta-ingest.toml. El host que lo honra no puede depender
// del CWD del proceso, ni del default de `config.storage_path` (§8.4).
let cfg = IngestRunnerCfg::from_storage(&storage);
```

Forma concreta: la ruta efectiva debe terminar en `data/vanta-ingest.toml` **y** empezar por
la raíz declarada — nunca ser relativa al CWD. Hoy el camino 2 (`wiki.rs`) y el 3
(`memory_scheduler.rs`) resuelven con `from_storage`, que con `InMemory` cae a
`config.storage_path` (donde el operador **sí** lo declaró) ⇒ **ya verde**, y volverían a
rojo si alguien los cambia a `from_storage_path(&storage.config().storage_path)`.
Es la aserción que mide el fix; sin ella el cambio se cierra "en el papel".

---

## §5 — Orden de los pasos (el árbol compila en cada uno)

Las 9 filas aterrizan en **un solo commit/PR** (si no, el árbol queda con dos read sites
del mismo env dentro de la misma rama). El orden mantiene `cargo check` verde **después
de cada paso**.

| # | Paso | Filas | Effort | Parada falsable |
|---|---|---|---|---|
| **1** | **Aditivo**: mover a `vanta-memory/src/ingest/runner_config.rs` (`resolve` privado, `ingest_toml_path`, `from_storage_path`, `runner_factory` con el `clone()`) + 3 tests | 1 | 2 h | `cargo check -p vanta-memory` + `cargo test -p vanta-memory --lib ingest` verdes. **El server sigue con su copia**: dos read sites es aceptable *dentro* del PR |
| **2** | **Aditivo**: mover a `vanta-memory/src/services/scheduler.rs` (const + `parse_interval_ms` + `_opt`), `from_env` como wrapper, mover el test. Todo **por encima** del primer `#[cfg(feature = "http-server")]` | 2 | 1 h | `cargo check -p vanta-memory --no-default-features` verde ⇒ son ungated |
| **3** | **Borrar + shims en el server** (§3.1), imports de §2.3 | 3 | 1 h | `cargo check -p vantadb-server` verde **y** `rg -c 'fn ingest_toml_path' --glob '*.rs'` = 1 |
| **4** | **main.rs** al API nueva (§2.4) | 4 | 0.25 h | `cargo check -p vantadb-server` verde |
| **5** | ~~**mcp wiki**: borrar la copia, `from_storage_path`~~ | 5 | ✅ **0 h** | ✅ **hecho** por `FIND-783`: `wiki.rs:292` usa `from_storage`. `join("vanta-ingest.toml")` en `vantadb-mcp` = **0** |
| **6** | **mcp scheduler de producción** (§2.2 fila 8) | 8 | 1 h | `cargo check -p vantadb-mcp`; el test `interval_env_is_one_knob…` **sigue verde sin tocarlo**; `rg -c 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' --glob '*.rs'` = **1** |
| **7** | **mcp threads** (§2.2 fila 9) | 9 | 0.5 h | `rg -c 'join\("vanta-ingest.toml"\)' --glob '*.rs'` = **0**; `rg -c 'env::var\("VANTADB_INGEST_CONFIG"\)' --glob '*.rs'` = **1** |
| **8** | **Tests de verificación**: §4.1 + §4.4 | — | 1.5 h | ambos verdes; §4.4 **rojo si se revierte la fila 5 o la 8** |
| **9** | **Docs**: filas 6 y 7 | 6,7 | 0.3 h | `node scripts/docs/check-docs.mjs && node scripts/docs/check-links.mjs` exit 0 |
| **10** | **Cierre**: suite + public-api + machete | — | 1 h | §6 entero |

**Por qué aditivo primero (1-2 antes de 3):** los pasos 3-7 **borran**. Si el borrado va
primero, el árbol no compila hasta el final y no hay punto de retorno parcial — es el
"if missed: nothing compiles in steps 2-5" del ADR.

**Por qué el server (3) antes que el MCP (5-7):** el MCP pasa a consumir items de
`vanta-memory`; si el `pub use` del server aún no existe, el paso 3 falla por ambigüedad
y el diagnóstico se mezcla con los cambios del MCP. Además la fila 3 es el único riesgo
real (los seis nombres).

**Por qué 5 → 6 → 7 y no los tres a la vez:** cada uno tiene su propia parada
(`data_dir` de 1 a 0). Si se hacen juntos, un fallo no dice cuál de las tres copias
quedó viva.

---

## §6 — Verificación completa

```bash
cargo check -p vanta-memory -p vantadb-server -p vantadb-mcp
cargo check -p vanta-memory --no-default-features
cargo nextest run --profile audit -p vanta-memory -E 'test(ingest) or test(interval)'
cargo nextest run --profile audit -p vantadb-mcp
cargo nextest run --profile audit -p vantadb-server
cargo clippy -p vanta-memory -p vantadb-server -p vantadb-mcp --all-targets -- -D warnings
cargo machete
cargo public-api -p vanta-memory                          # diff: exactamente +8, 0 eliminados (§3.3)
```

No-regresión medible (los read sites, no las menciones — §4.2):

```bash
rg -n 'env::var\("VANTADB_INGEST_CONFIG"\)' --glob '*.rs'          # 1
rg -n 'env::var\("VANTADB_SCHEDULER_INTERVAL_MS"\)' --glob '*.rs'  # 1
rg -n 'fn ingest_toml_path' --glob '*.rs'                          # 1
rg -n 'fn ingest_runner_cfg\(' --glob '*.rs'                       # 1
rg -n 'join\("vanta-ingest.toml"\)' --glob '*.rs'                  # 0
git status --short src/                                            # vacío
```

**Sobre `llm-driver` en `vantadb-server/Cargo.toml:37`:** **se queda**, aunque el server
deje de nombrar `ConcreteRunner`. Es lo que hace que el runner construido *dentro de*
`vanta-memory` sea un cliente HTTP real y no `NotConfigured`
(`docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:159`). Verifícalo con
`cargo machete` antes de borrarlo — no a ojo.

---

## §7 — Trampas (las que rompen en silencio)

1. **El `pub use` en `lib.rs`.** `vantadb-server/src/lib.rs` declara `pub mod scheduler;`
   (módulo-fichero). Un `pub use` en `lib.rs` publica `vantadb_server::ingest_runner_cfg`,
   **no** `::scheduler::…`, y rompe al consumidor para el que existe. Va en el cuerpo de
   `scheduler.rs`. (H1 del review del ADR.)
2. **Son seis, no cinco.** Cinco `pub fn` + un `const`. `ingest_runner_factory` es el que
   se olvidó en la ronda 2 (H-A). Cuéntalos: 4 `pub use` + 2 shims.
3. **`IngestRunnerCfg` dos veces en el módulo** ⇒ **E0252**, no warning (§2.3).
4. **`IngestRunnerCfg::` a secas en `main.rs`** ⇒ no compila: el `pub use` no está en su
   scope (§2.4).
5. **`runner_factory` sin `clone()`** ⇒ no compila (§2.3).
6. **`data_dir()` en vez de `storage_path`** ⇒ reintroduce el bug con una sola copia.
   `data_dir()` ya incluye `data/`.
7. **Items movidos por debajo de un `#[cfg(feature = "http-server")]`** en
   `services/scheduler.rs` ⇒ `vantadb-mcp` no los ve y el fallo parece de Cargo, no de
   posición.
8. **Un segundo `env::var("VANTADB_SCHEDULER_INTERVAL_MS")`** para "saber si está
   definido" ⇒ viola R-5 y es exactamente lo que H-B prohibió. Usa
   `scheduler_interval_ms_from_env_opt()`.
9. **Borrar `DEFAULT_INTERVAL_MS = 5_000` de `memory_scheduler.rs`.** Es el default **del
   host**, no una copia del del server (5 000 ≠ 60 000, y está justificado en `:38-43`).
   Lo que se mueve es el **parser**, no la política (§8.0).
10. **`Path`/`PathBuf` sin recortar** en `scheduler.rs` ⇒ warning ⇒ clippy rojo.

---

## §8 — Discrepancias medidas contra `HEAD` (2026-11-26)

### 8.0 **`MCP-43` aterrizó antes que esta tarea** — la premisa de secuencia del ADR murió

Esto es lo que cambia el alcance, y es el hallazgo que hay que leer antes de aprobar.

`vantadb-mcp/src/memory_scheduler.rs` **existe, está trackeado y está en `HEAD`** (664
líneas). Su cabecera lo dice: `//! MCP-43: the --mcp writer hosts the vanta-memory
scheduler`. Y **no** usó la superficie que este ADR define:

| El ADR decidió | `MCP-43` implementó |
|---|---|
| `vantadb-mcp` alcanza la superficie "con `llm-driver` solo" (D5, `ADR:410-418`) | `vantadb-mcp/Cargo.toml:31` ya lleva `["llm-driver", "http-server"]` |
| `MCP-43` "must not start until this ADR is approved" (§Status, `ADR:15-18`) | arrancó igual, con parser y fábrica propios |
| el loop del `--mcp` sólo arranca si `VANTADB_SCHEDULER_INTERVAL_MS` está **explícitamente definida** y ≠ 0 (D4, `ADR:368-391`) | **sin definir ⇒ 5 000 ms ⇒ loop ON** (`memory_scheduler.rs:69-70`) |
| `scheduler_interval_ms_from_env_opt()` es la primitiva con la que `MCP-43` expresará "sin definir" (D4) | no la usa: lee el env a mano |

Consecuencias fácticas: (i) el ADR-0063 ya **no** desbloquea `MCP-43` — su valor ahora es
reparar el drift; (ii) la resolución del TOML existe **tres** veces, no dos, y la tercera
—`threads.rs:367`— es el camino de **producción** del `--mcp` y sigue usando `data_dir()`;
(iii) el mismo proceso (`vantadb-server --mcp`) tiene dos defaults distintos para la misma
variable (5 000 y 60 000), que es el *semantic fork* que D4 quiso evitar, sólo que entre
hosts en vez de entre parsers.

**Esto no invalida la decisión de ADR-0063** (mover la superficie sigue siendo correcto, y
ahora más: una regla en lugar de tres). Invalida **su premisa de secuencia**.

#### La única decisión que este task file no puede tomar solo

**¿Qué significa "sin definir" para el scheduler del `--mcp`?**

| Opción | Comportamiento | Valoración |
|---|---|---|
| **A (recomendada)** — preservar lo enviado: `_opt().unwrap_or(5_000)` | el loop sigue arrancando por defecto en el `--mcp` | **Sin regresión para nadie.** El default es política del host, y R-5 prohíbe el segundo *read site*, no un default por host. Requiere una línea de enmienda en ADR-0063 D4 |
| B — literal de D4: sin definir ⇒ **off** | el loop del `--mcp` deja de arrancar salvo que el operador defina la variable | **Desactiva en silencio una capacidad enviada** (`MCP-43` existe para que el turno anterior sea recallable en el siguiente). No hay medición de quién se apoya en el default. Y el motivo de D4 — el host one-shot `vanta-cli mcp-call` — no se verificó contra el código enviado |
| C — no tocar `memory_scheduler.rs`; recortar el DoD a "HTTP + `wiki_ingest`" y abrir deuda | menos diff | deja el **camino de producción** del `--mcp` resolviendo por `data_dir()`: el bug que el ADR existe para cerrar sigue vivo donde más importa. Y el DoD de §0 pasa a ser falso |

**Este contrato ejecuta la opción A.** Si el owner prefiere B, el cambio es: `match
scheduler_interval_ms_from_env_opt() { None | Some(0) => off, Some(ms) => ms }` en
`start` (y el test de `:305-328` cambia de expectativa). Si prefiere C, hay que reescribir
§0 y §4.2 y abrir la fila de deuda.

**Enmienda necesaria en `ADR-0063` (la hace `vanta-arch`/owner, no el implementador):**
una línea en §Status del tipo *"MCP-43 landed ahead of this ADR with its own interval/env
parser and a third TOML resolution; those are folded into WIRE-19's scope, and the MCP
host keeps its 5 s default on unset (D4 corrected)."* Sin esa línea, el ADR y el árbol
se contradicen y el próximo lector re-litiga esto.

### 8.1 `vantadb-mcp` ya tiene `http-server` (declarado, no un defecto)

`vantadb-mcp/Cargo.toml:31` es `vanta-memory = { workspace = true, features =
["llm-driver", "http-server"] }`. El ADR (D5) decía "con `llm-driver` solo" y que
`http-server` "pertenece a `MCP-43`". **No hay que editar el manifiesto** y no se debe
interpretar la feature presente como evidencia de que `MCP-43` "ya se hizo según el ADR":
se hizo, pero por su cuenta (§8.0).

### 8.2 Las líneas del bloque a borrar: **estaban** donde el ADR decía, y `FIND-783` las movió

Medido en `HEAD` (2026-11-27): `vantadb-server/src/scheduler.rs` tiene **182 líneas**, no 192 —
`FIND-783` ya borró `resolve_ingest_toml_path` y reemplazó `ingest_toml_path` por un `pub use`.
Mapa vivo, que es el que un implementador necesita:

| Rango `HEAD` | Qué es | Destino |
|---|---|---|
| `:43-45` | `DEFAULT_INTERVAL_MS` + doc (cita `src/config.rs`) | → `vanta-memory/src/services/scheduler.rs` |
| `:47-64` | `parse_interval_ms` | → idem |
| `:66-73` | `scheduler_interval_ms_from_env` (lee el env en `:69`) | → idem, como wrapper de `_opt` |
| `:75-82` | doc + `pub use ... ingest_toml_path` | **se queda** (es el re-export) |
| `:84-87` | shim `ingest_runner_cfg` | **se queda**; su cuerpo pasa a `from_storage_path` → ya lo es |
| `:89-101` | `ingest_runner_factory` | **se muda como método** (`runner_factory`); el `fn` queda como shim |
| **`:110-132`** | `wire_memory` | **se queda** (ADR D2: composición, no configuración) |
| `:134-182` | `mod tests` (4 tests) | 3 se mueven con su código; los 2 de la fábrica se quedan hasta que exista el método |

⚠️ **Corregido por review:** el rango `:44-115` que decían §1/§2.1/§8.2 en la versión previa
**cortaba las 6 primeras líneas de `wire_memory`** en `HEAD` (empieza en `:110`, no en `:117`).
Era el error más caro del fichero: una instrucción de borrado que se lleva código que el ADR
dice conservar.

### 8.3 `storage.config()` existe; `main.rs` no cambia de firma

`src/storage/engine/mod.rs:505` (`pub fn config(&self) -> &Config`) está vivo, así que
`&storage.config().storage_path` compila en `wiki.rs`, `memory_scheduler.rs` y
`threads.rs`. `main.rs:85` ya guarda `let storage_path = config.storage_path.clone();` —
el call site nuevo lo reusa sin tocar el closure.

---

### 8.4 **Medido: el constructor que este fichero prescribía reintroduce el bug al revés**

Las filas 4/5/8 y §2.4/§4.4 pedían `IngestRunnerCfg::from_storage_path(&storage.config().storage_path)`.
`FIND-783` **no** lo hizo y añadió `IngestRunnerCfg::from_storage(&StorageEngine)`
(`runner_config.rs:295-304`). Verificado leyendo el código, no heredado:

| Afirmación | Evidencia | Veredicto |
|---|---|---|
| `StorageEngine::open(path)` **no** escribe `path` en `config.storage_path` | `src/storage/engine/init.rs:24-26` → `open_with_config(path, None)` → `init.rs:29-31`: `let config = config.unwrap_or_default();`. `path` sólo llega a `Self::storage_root(path)` (`init.rs:313-316`), que devuelve un `PathBuf` **sin** tocar `config` | ✅ **cierta** |
| Ese default no es la ruta del operador | `src/config.rs:1442-1447`: `storage_path = env VANTADB_STORAGE_PATH` o el literal **`"vantadb_data"`** | ✅ **cierta** ⇒ `from_storage_path(&config.storage_path)` resolvería `./vantadb_data/data/vanta-ingest.toml` **relativo al CWD** |
| Ya había precedente escrito | `vantadb-mcp/src/server.rs:190-193` deriva la raíz de `data_dir()` con el mismo razonamiento, por la misma razón (discovery file del proxy) | ✅ |
| `data_dir()` es `<raíz>/data` para todo backend de disco, y vacío sólo en `InMemory` | `init.rs:419-470`: `init_storage` es genérica por `BackendRegistry` y hace `base_path.join("data")` (`:459`); el early-return de `InMemory` devuelve `PathBuf::new()` (`:425-428`) | ✅ ⇒ `data_dir().parent()` **es** la raíz declarada para fjall/rocksdb/vfile/lo que registre el registry |
| En los dos hosts reales ambos constructores coinciden | `vantadb-server/src/main.rs:85` pasa `config.storage_path` (que ahí **sí** es la raíz declarada) y `vantadb-mcp/src/server.rs:1050-1057` abre con `Some(cfg_with_path)`, cuyo `storage_path` se fija en `:1051-1053` | ✅ ⇒ `from_storage` es mejora estricta, **no** cambio de comportamiento en producción |

**Contrato corregido:** las filas 5/8/9, §2.4 (wiki) y §4.4 usan ya `from_storage`.
`from_storage_path` **se queda**: es el constructor correcto para quien sólo tiene un `&str`
(el hook del HTTP, `main.rs:88-95`) y el que ejercita el shim `ingest_runner_cfg`.
El diff de `cargo public-api -p vanta-memory` será **+8, no +7** (§3.3): `from_storage` es un
item `pub` que el ADR-0063 no previó.

**Lo que sólo se puede comprobar compilando — y no lo hice (disco, y `FIND-773` compilando):**

1. Que `cargo public-api -p vanta-memory` dé exactamente `+8 / -0` (el conteo de arriba es por
   `rg`, no por el snapshot).
2. Que `IngestRunnerCfg` siga derivando `Clone` donde §2.3 lo necesita para `runner_factory`
   (el `derive` existe; su línea cambió con la mudanza, `runner_config.rs:60-70` aprox.).
3. Que ningún backend registrado en `BackendRegistry` devuelva un `data_dir` cuyo `parent()` no
   sea la raíz — la lectura cubre `init_storage`, no cada factory.

---

## §9 — ORDEN RECOMENDADO (entre los dos contratos de esta entrega)

**`FIND-756-impl.md` va primero.** Este fichero **no** va primero, aunque en la versión
anterior de la recomendación sí lo hacía: `MCP-43` aterrizó (§8.0), así que `WIRE-19` ya
**no desbloquea nada** — sólo repara el drift que `MCP-43` dejó.

| Criterio | WIRE-19 | FIND-756 |
|---|---|---|
| Ficheros en común | **0** | **0** |
| Decisiones abiertas | **1** (§8.0, opción A/B/C — merge gate, no del implementador) | **0** |
| Effort | ~1.25-1.5 d (9 filas) | ~8.25 h (6 sitios, 13 aserciones) |
| Desbloquea | **nada** (`MCP-43` ya está en `HEAD`) | `FIND-296`/`MEMG-03` parcial |
| Riesgo si espera | bajo: el drift ya existe y ya está en producción | bajo |

**Razón 1 — cero conflicto, así que ordena el valor.** `WIRE-19` toca
`vanta-memory/src/{ingest,services}` + `vantadb-server` + `vantadb-mcp`; `FIND-756` toca
`vanta-memory/src/core/*` + `tests/lineage_graph.rs`. Ni un fichero compartido.

**Razón 2 — uno está listo y el otro no.** `FIND-756-impl` no tiene ninguna decisión
pendiente: cada paso es mecánico, cada aserción es falsable. `WIRE-19-impl` tiene §8.0,
que cambia el comportamiento de un host **ya enviado** y que no se decide en un task
file. Empezar por el que no bloquea en una decisión externa es el uso correcto del tiempo.

**Razón 3 — el baseline de `public-api`.** `WIRE-19` mueve la superficie publicable
(`+8`). Conviene que ese diff se mida y se acepte **antes** de tocar los internos del
mismo crate, para no atribuir a `FIND-756` (que no añade ni un item `pub`) un diff raro
de `cargo public-api`. Es el único motivo por el que `WIRE-19` preferiría ir primero — y
no pesa más que las razones 1 y 2.

**Contra-argumento, para que el owner pueda invertir el orden:** el drift de `WIRE-19`
está **vivo y creciendo** (cada host nuevo copia otra vez; ya pasó con `MCP-43`), y el
camino de producción del `--mcp` resuelve el TOML mal en `InMemory` **hoy**. Si lo que
pesa es "el que más se degrada si espera", `WIRE-19` gana. No hay dependencia técnica
entre ambos: invertir no rompe nada, sólo cambia qué se arregla primero.
