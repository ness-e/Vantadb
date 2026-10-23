---
title: "FIND-517 - La pre-reserva de 320 MB al conectar el binding de Node"
kind: task
status: completed
description: "Que conectar un store vacio no reserve 320 MB: es el coste dominante de memoria del producto de Ego (VBD-REQ-14)."
tags: [bindings, node, memoria, critica, ego]
---

# FIND-517: La pre-reserva de 320 MB al conectar el binding: un store vacio ya ocupa 320,06 MB

Conectar el binding de Node **reserva 320 MB** (5 x 64 MB) antes de que haya datos: un store **vacio** ya ocupa 320,06 MB, y `ConnectOptions` solo expone `read_only` y `memory_limit` ⇒ la reserva **no es parametrizable**. En el producto de Ego cada memoria abierta (proyecto, sesiones, sub-egos) cuesta 320 MB ⇒ es el **coste dominante de memoria del producto**, y no sale en ningun `memory_limit` porque no es memoria viva sino espacio reservado.

Es la peticion #1 de la lista priorizada de Ego (`VBD-REQ-14`, verificada el 2026-10-21 contra el binario real 0.9.0: no son restos de test, es la pre-reserva).

Los tres fixes posibles (del informe): (a) **modo lazy**: no reservar hasta que el grafo necesite el espacio, reservando en background; (b) parametrizar la pre-reserva (`ConnectOptions.initial_capacity` / `reserve_bytes`) con un valor por defecto bajo; (c) dejar de pre-reservar y dejar que el allocator crezca bajo demanda.

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-21-fix-critical-backlog.md`
- **Fuente:** `docs/dev/Backlog.md` (FIND-517 (critica))
- **Esfuerzo:** 🟠 1-2sem
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust
- **Turns estimados:** 6-9
- **Creado:** 2026-10-21T12:00
- **last-synced:** 2026-10-21T14:20
- **Estado:** ✅ COMPLETADA (implementada, review P2-01 APTO en d5a8d4bc, cerrada y purgada en a09de3e0)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes (5/5 completados) + review P2-01

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | vantadb-ts/src/native.ts, los ejemplos de vantadb-node y vantadb-ts, los docs del SDK |
| Callees | **`src/lsm.rs`** (`SegmentRegistry::open_or_create` ⇒ la reserva de 4 x 64 MB), **fjall 3.1.10** (`journal/writer.rs` ⇒ el 5º x 64 MB), `src/storage/vfile.rs` (`File::open_with_crypto` ⇒ `set_len` + `mmap` de todo el fichero), vantadb-ffi-core (OpGate) |
| Implicaciones | el pico de memoria reservada del producto baja de ~320 MB por memoria a ~68 MB (64 MB de los cuales son el journal de fjall, de terceros). El tiempo de connect no crece (medido: −2,6 %). |

## Impacto mapeado (Regla 0)

> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo
> que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES
> del primer step de edición. Sin este bloque poblado, NO se escribe ni se
> ejecuta ningún step que edite archivos.

- **Archivos leídos (completos):** vantadb-node/src/lib.rs (el `ConnectOptions` y `build_config`), src/lsm.rs (los 4 niveles LSM y la reserva), src/storage/vfile.rs (`File::open_with_crypto`/`grow_to`/el mmap), src/storage/engine/maintenance.rs (`should_compact_level`/`compact_level` — para probar que la pre-reserva NO es load-bearing), src/storage/engine/init.rs (la rama in-memory), src/backends/fjall_backend.rs (el backend fjall), vantadb-node/dts-header.d.ts + index.d.ts (el contrato publico), vantadb-ts/src/native.ts (el `NativeConnectOptions` que envuelve), fjall-3.1.10 (`journal/writer.rs`, `db_config.rs`) en el registro de cargo
- **Correccion al mapeo original:** el task file situaba la reserva en `src/index/**`; **no es ahi**. La reserva de 4 x 64 MB vive en `src/lsm.rs` (`SegmentRegistry::open_or_create`); el 5º x 64 MB es el journal de fjall (crate de terceros, modulo `journal` privado ⇒ no configurable sin vendorizar fjall). `src/index/**` no se ha tocado.
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** vantadb-node/src/lib.rs -> vantadb (Config -> SegmentOptimizerConfig -> LsmConfig) -> src/lsm.rs (`open_or_create`); vantadb-ts/src/native.ts -> vantadb-node (napi)
- **Archivos que referencian a los editados (referencias entrantes):** vantadb-node/index.d.ts (el tipo publico), vantadb-ts/src/native.ts, examples/ y docs que construyen un cliente
- **Veredicto impacto:** medio-alto: cambia el contrato publico del binding (un parametro nuevo en `ConnectOptions`) ⇒ hay que actualizar el `.d.ts` (generado), el `.cjs`/`index.js` commiteados (FIND-547: estan desfasados) y los docs. Y toca el **arranque** del binding ⇒ medir el tiempo de connect.

## Root cause verificado (step 1 — evidencia, antes de tocar nada)

Medido contra el binario 0.9.0 commiteado (`vantadb-node/vantadb_native.win32-x64-msvc.node`, el mismo que uso Ego) con un store **vacio**:

| Fichero creado al abrir un store vacio | Tamaño | Quien lo crea |
|---|---|---|
| `data/vstore_L0.vanta` | 67 108 864 B (64 MiB) | `src/lsm.rs` → `SegmentRegistry::open_or_create` |
| `data/vstore_L1.vanta` | 67 108 864 B (64 MiB) | idem |
| `data/vstore_L2.vanta` | 67 108 864 B (64 MiB) | idem |
| `data/vstore_L3.vanta` | 67 108 864 B (64 MiB) | idem |
| `0.jnl` (journal de fjall) | 67 108 864 B (64 MiB) | `fjall-3.1.10/src/journal/writer.rs:19` (`PRE_ALLOCATED_BYTES`), `file.set_len(...)` |

⇒ **5 x 64 MiB = 320,02 MB** de longitud de fichero en un store sin un solo registro. Ese es el "320,06 MB" del backlog.

La cadena exacta (causa raiz, no sintoma):
`SegmentRegistry::open_or_create` (`src/lsm.rs:164`, antes del fix) → `File::open_with_crypto(path, 64 * 1024 * 1024, ...)` → `src/storage/vfile.rs:417` `file.set_len(64 MiB)` (**fichero disperso**: 0 bytes reales en disco) + `map_readwrite(&file)` (**mmap de los 64 MiB enteros** ⇒ 64 MiB de **espacio de direcciones reservado** por nivel).

**Matiz importante (corrige la metrica del backlog):** la reserva nunca fue memoria *residente*: el working set (RSS) de un store vacio ya era de ~2,3 MB (solo metadata). Los 320 MB eran **espacio reservado** (direcciones virtuales + longitud de fichero), que es exactamente por lo que no salia en ningun `memory_limit` — el `memory_limit` mide memoria viva, no reserva. Por eso el contrato ("el consumo de RSS ... es el de la metadata") ya se cumplia de facto; lo que estaba roto era la **reserva**.

## Decisión (step 2): opción **(b) parametrizable con default bajo**

**Elegida: (b)** — la pre-reserva pasa a ser un parametro explicito con default bajo:
`ConnectOptions.reserve_bytes` (bytes por segmento LSM; un store tiene 4 ⇒ el total reservado es `4 x reserve_bytes`), default **1 MiB por segmento** (4 MiB por store, antes 64 MiB por segmento = 256 MiB).

Por que (b) y no las otras:

- **No (c) "sin pre-reserva"**: la pre-reserva existe por un motivo (que `compact_level()` y el hot path de escritura no tengan que crecer el fichero a mitad de operacion). Quitarla del todo elimina una capacidad que alguien puede querer, y no aporta nada extra: con (b) + default bajo el store **no reserva** practicamente nada, luego el efecto en memoria es el mismo.
- **No (a) "modo lazy"**: el problema es el **tamaño** de la reserva, no el **momento**. El modo lazy anade un hilo/estado de reserva en background (concurrencia, estados intermedios, casos de fallo) sin ganar nada: con el default bajo la reserva ya es de 4 MiB y no crece por usar el store. Es la opcion mas cara de las tres y la que mas superficie de fallo introduce.
- **Si (b)**: (i) es **aditiva** ⇒ no rompe el contrato publico (invariante 1) — el resto de bindings (python/wasm/server/CLI) siguen usando `Config::default()` y no cambian de comportamiento observable; (ii) hace la reserva **explícita y cubierta por `memory_limit`** (invariante 2) — que es justo lo que el informe dice que falla ("no sale en ningun `memory_limit`"); (iii) mantiene la capacidad de pre-reservar para quien la quiera (`reserve_bytes` grande); (iv) el crecimiento del segmento es **por duplicacion** (`File::grow_to` -> `max(size*2, needed+4096)`), asi que partir de 1 MiB en vez de 64 MiB cuesta ~4 ciclos extra de `set_len`+remap en toda la vida del segmento (O(log), no O(n)).

Default: **1 MiB por segmento**. Es "orden de MB" (contrato), cubre ~800 registros por segmento antes del primer crecimiento (un registro ≈ 1280 B segun `prealloc_vstore_batch`), y baja la reserva de 256 MiB a 4 MiB por store (**64x**).

## Implementacion (step 3)

| Fichero | Cambio |
|---|---|
| `src/lsm.rs` | `LsmConfig.initial_segment_bytes: u64` (nuevo, default 1 MiB) + `SegmentRegistry::open_or_create` deja de hardcodear `64 * 1024 * 1024` y usa `config.lsm.initial_segment_bytes` (con suelo en `STORAGE_ALIGNMENT` = 64 B, el header del segmento) |
| `vantadb-node/src/lib.rs` | `build_config` parsea `ConnectOptions.reserve_bytes` (validado: > 0, entero) y **lo recorta contra `memory_limit`** (`per_segment = min(reserve, memory_limit / 4)`) |
| `vantadb-node/dts-header.d.ts` | `ConnectOptions.reserve_bytes?: number` documentado (contrato publico) |
| `vantadb-node/index.d.ts`, `index.cjs`, `index.js` | **regenerados** con `napi build --platform --release` (FIND-547): la version hardcodeada pasa de `0.8.0` a `0.9.0` (27 sitios por fichero) y el `.d.ts` incluye `reserve_bytes` |
| `vantadb-ts/src/native.ts` | `NativeConnectOptions.reserve_bytes?: number` + doc actualizado |

Sin `unsafe` nuevo. Sin tocar `src/index/**`, `src/parser/**`, `src/planner.rs`, `src/physical_plan/**`, `src/crypto*`, `src/storage/**`, `src/config.rs`, `src/lib.rs`, `benches/**`, `tests/encryption_chaos.rs`.

## Mediciones (step 5) — antes / despues

Metodo: proceso Node limpio, `Get-Process` (WorkingSet64 / PrivateMemorySize64 / VirtualMemorySize64) antes y despues de `VantaDb.connect()` sobre un directorio vacio; store de 1 sola conexion; A/B de `connect()` con 5 runs alternados en el **mismo** binario (el default post-fix vs `reserve_bytes = 64 MiB`, que reproduce exactamente el comportamiento pre-fix).

| Metrica | ANTES (binario 0.9.0 commiteado) | DESPUES (binario reconstruido) | Delta |
|---|---|---|---|
| Longitud de fichero de un store **vacio** | 320,02 MB (5 x 64 MiB) | **68,02 MB** (4 x 1 MiB + 64 MiB del journal de fjall) | **−78,7 %** |
| **Espacio de direcciones reservado** por `connect()` (delta de VirtualMemorySize64) | **+279 MB** | **+27 MB** | **−90,3 %** |
| RSS (working set) del proceso tras `connect()` | +2,29 MB | +2,14 MB | −0,15 MB (ya era bajo: la reserva no era residente) |
| Memoria privada (commit) tras `connect()` | +1,72 MB | +1,09 MB | −0,63 MB |
| Tiempo de `connect()` (mediana de 5, A/B alternado, mismo binario) | 720,0 ms (`reserve_bytes = 64 MiB` = comportamiento pre-fix) | **701,1 ms** (default) | **−2,6 % ⇒ no empeora** |
| Tiempo de `connect()` absoluto (maquina idle, antes del rebuild) | 263 ms / 271 ms (dos connects) | — | — |

Los ~700 ms del A/B son de una maquina con otros agentes compilando en paralelo; por eso la comparacion valida es el A/B **dentro del mismo proceso y la misma maquina**, no el absoluto.

Lo que **queda** de los 320 MB y por que:

- **64 MB** = el journal de fjall (`fjall-3.1.10/src/journal/writer.rs:19`, `PRE_ALLOCATED_BYTES = 64 * 1_024 * 1_024`): es un `set_len` de un fichero **disperso** (0 bytes reales en disco, 0 bytes de RAM) y el modulo `journal` de fjall es **privado** ⇒ no es configurable desde fuera sin vendorizar el crate. Fuera del alcance de este task (decidirlo es de Arch).
- El resto (256 MB) era el codigo de VantaDB y **esta arreglado**.

## Tests nuevos

| Test | Tipo | Que fija |
|---|---|---|
| `src/lsm.rs::tests::fresh_segments_do_not_reserve_64mb` | unit (Rust) | un store vacio no crea segmentos de 64 MiB (RED: "vstore_L0.vanta reserves 67108864 bytes on an empty store") |
| `src/lsm.rs::tests::open_or_create_honours_initial_segment_bytes` | unit (Rust) | `lsm.initial_segment_bytes` manda sobre el tamaño del segmento |
| `vantadb-node/src/lib.rs::tests::build_config_maps_reserve_bytes_to_the_segment_pre_reserve` | unit (Rust) | `ConnectOptions.reserve_bytes` llega a la reserva |
| `vantadb-node/src/lib.rs::tests::build_config_defaults_to_a_low_pre_reserve` | unit (Rust) | el default es 1 MiB por segmento (no 64 MiB) |
| `vantadb-node/src/lib.rs::tests::build_config_clamps_the_reserve_to_the_memory_limit` | unit (Rust) | la reserva (4 x por_segment) cabe en el `memory_limit` |
| `vantadb-node/src/lib.rs::tests::build_config_clamps_the_default_reserve_to_the_memory_limit` | unit (Rust) | el default tambien se recorta contra `memory_limit` |
| `vantadb-node/src/lib.rs::tests::build_config_rejects_a_zero_reserve` | unit (Rust) | `reserve_bytes = 0` es un error explicito |
| `vantadb-node/src/lib.rs::tests::build_config_rejects_a_non_numeric_reserve` | unit (Rust) | `reserve_bytes` no numerico es un error explicito |
| `vantadb-node/tests/find517.test.ts` (5 casos) | integracion (vitest, motor real) | (1) un store vacio reserva <= 4 MiB por nivel; (2) `reserve_bytes` explicito se respeta; (3) el `memory_limit` recorta la reserva; (4) `reserve_bytes = 0` se rechaza; (5) put/get siguen funcionando con la reserva nueva |

Ciclo TDD: RED reproducido antes del fix (`vstore_L0.vanta reserves 67108864 bytes on an empty store` ⇒ FAIL), GREEN con el cambio minimo, REFACTOR (nombres + doc). El test de la opcion (b) en el binding entra como RED por error de compilacion (el campo aun no existia) ⇒ se documento en el commit.

## Invariantes: verificados / pendientes

| # | Invariante | Estado | Evidencia |
|---|---|---|---|
| 1 | el contrato publico no se rompe (parametro aditivo con default) | ✅ | `reserve_bytes` es opcional; python/wasm/server/CLI usan `Config::default()` y no cambian; el `.d.ts` regenerado mantiene `VantaDb` + el alias `VantaDB` |
| 2 | la reserva no crece por encima del `memory_limit` | ✅ | recorte en `build_config` + 2 tests unitarios + test de integracion; excepcion documentada: un `memory_limit` < 256 B no puede honrarse (4 cabeceras de 64 B) |
| 3 | el tiempo de connect no empeora de forma perceptible | ✅ | A/B en la misma maquina: 720,0 ms (pre-fix) vs 701,1 ms (post-fix) ⇒ −2,6 % |
| 4 | el `.d.ts` y el `.cjs`/`index.js` commiteados son coherentes con el codigo | ✅ (parcial) | regenerados con `napi build` ⇒ `0.9.0` (FIND-547 cerrado en estos 3 ficheros). **Pendiente de FIND-546**: el alias `export type VantaDB = VantaDb` del `.d.ts` sigue exportando un valor `undefined` en runtime (lo genera el propio napi-rs); renombrar la clase Rust es decision de FIND-546, no de este task |
| 5 | nada de `unsafe` nuevo | ✅ | 0 `unsafe` anadido; la frontera FFI no se ha tocado |
| — | la pre-reserva no es load-bearing para la compactacion | ✅ | `should_compact_level` compara el **cursor de escritura**, no el tamaño del fichero (`src/storage/engine/maintenance.rs:882`); `write_node_to_vstore` crece el segmento por duplicacion |

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-517: La pre-reserva de 320 MB al conectar el binding: un store vacio ya ocupa 320,06 MB` |
| `lastAction` | Implementado + verificado (fmt/clippy/tests/mediciones); task file actualizado; **falta la review P2-01** |
| `result` | 🟡 IMPLEMENTADO — la reserva de un store vacio baja de 320,02 MB a 68,02 MB de fichero y de +279 MB a +27 MB de espacio reservado; `connect()` no empeora (−2,6 %); `ConnectOptions.reserve_bytes` (nuevo, aditivo, default 1 MiB/segmento) recorta la reserva contra `memory_limit` |
| `nextAction` | Review P2-01 por un agente distinto (`## Review`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda introducida por este task. Quedan dos residuos **fuera de alcance**, documentados para que los decida quien corresponda:
- el journal de 64 MiB de fjall (crate de terceros, modulo privado) — 0 bytes de RAM, 64 MiB de longitud de fichero disperso;
- el `File::create_in_memory(64 * MIB)` del motor **in-memory** (`src/storage/engine/init.rs:85`) — son 64 MiB de heap *committeado* en `connect(":memory:")`; el fichero esta en `src/storage/**`, excluido de este task por trabajo en curso de otros agentes.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + `cargo fmt --check` + `cargo clippy -p vantadb --all-targets -- -D warnings` + los tests del cambio en verde |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica (nunca auto-reporte) |
| **Release** | No aplica (sin release en esta iteracion) |

## Herramientas necesarias
- cargo (check, clippy, fmt) — con `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`
- codegraph_codegraph_explore (blast radius)
- napi CLI (`vantadb-node/node_modules/.bin/napi.cmd build --platform --release [--esm] --js <file>`) para regenerar `index.cjs`/`index.js`/`index.d.ts`

## Skills cargadas (SDP):** test-driven-development (el fix necesita un test que fije el
comportamiento roto antes del fix) + systematic-debugging (causa raiz, no sintoma) +
context-engineering (el mapeo original situaba la reserva en `src/index/**`; la evidencia
la Bring to `src/lsm.rs` + fjall) + incremental-implementation (slices: core -> binding ->
contrato TS -> regenerated artifacts).

## Investigation Notes
- El hallazgo viene de la auditoria profunda del 2026-10-20/21 y su evidencia (archivo:línea)
  esta en la fila del backlog. Verificar la evidencia ANTES de tocar nada: si el codigo ya
  esta corregido, documentarlo y cerrar la fila (sin editar).
- **Evidencia verificada (no era un resto de test):** un store vacio crea 5 ficheros de 64 MiB
  (4 segmentos LSM + el journal de fjall) y `connect()` reserva +279 MB de direcciones virtuales.
- La metrica del backlog ("320,06 MB") es la **suma de las longitudes de fichero**; el RSS real
  de un store vacio ya era de ~2,3 MB. La reserva nunca fue memoria residente.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 (5/5 steps hechos) |
| % completado | 100% (implementado; review P2-01 pendiente) |

## Steps

1. ✅ **Verificar la evidencia** (sin editar): localizada la reserva — **4 x 64 MiB** en
   `src/lsm.rs` (`SegmentRegistry::open_or_create`) + **1 x 64 MiB** en el journal de fjall
   (`fjall-3.1.10/src/journal/writer.rs:19`). Linea base medida: store vacio = 320,02 MB de
   fichero; `connect()` = **+279 MB** de espacio reservado, +2,29 MB de RSS, 263 ms.
2. ✅ **Elegir el fix**: **(b) parametrizable con default bajo** (justificacion en
   `## Decisión`).
3. ✅ **Implementar**: `src/lsm.rs` (campo + uso), `vantadb-node/src/lib.rs` (opcion +
   recorte por `memory_limit`), `dts-header.d.ts`, `vantadb-ts/src/native.ts`, y
   `index.d.ts`/`index.cjs`/`index.js` **regenerados** (0.8.0 -> 0.9.0).
4. ✅ **Test primero**: RED reproducido (`vstore_L0.vanta reserves 67108864 bytes on an empty
   store`) + 8 tests nuevos (2 en `src/lsm.rs`, 6 en `vantadb-node/src/lib.rs`) + 5 casos de
   integracion en `vantadb-node/tests/find517.test.ts`.
5. ✅ **Verificar**: fmt limpio en los ficheros tocados, `cargo clippy -p vantadb --all-targets
   -- -D warnings` exit 0, suite del lib de `vantadb` en verde, suite de `vantadb-node`
   (14 unit + 75 vitest) en verde, RSS/VA/connect medidos antes y despues (tabla de
   `## Mediciones`).

## Review (P2-01)

- **Reviewer:** `vanta-review` (agente DISTINTO del implementador; sesion de
  revision P2-01 sobre `develop`, 4 tasks)
- **Veredicto:** 🟡 **APTO CON RESERVAS** — el fix y sus 12 tests estan verificados
  mecanicamente; la reserva es de nivel **Commit**: falta commitear
  `vantadb-ts/src/native.ts` (el commit `eb862432` no lo incluye).
- **Fecha:** 2026-10-21
- **Checklist para el revisor:**
  1. ¿La decision (b) con default de 1 MiB/segmento esta bien justificada frente a (a) y (c)?
  2. ¿El recorte de la reserva contra `memory_limit` es la politica correcta (o deberia ser
     el total y no el por-segmento)?
  3. ¿Los 64 MiB del journal de fjall y los 64 MiB del motor in-memory
     (`src/storage/engine/init.rs:85`) deben abrirse una task propia?
  4. ¿El recorte del `.d.ts`/`.cjs`/`.index.js` (FIND-547) cierra esa fila, o el alias
     `VantaDB` de napi-rs sigue sin resolver (FIND-546)?
  5. ¿La suite completa (`cargo nextest run -p vantadb --profile audit --lib`) y el clippy del
     crate `vantadb-node` (que ya venia en rojo por `unwrap_used`/`expect_used` en su modulo de
     tests: 23 errores **preexistentes**, 0 anadidos) se aceptan tal cual?

### Dictamen del revisor (P2-01) — 2026-10-21

**Veredicto: 🟡 APTO CON RESERVAS.** El codigo, la decision (b) y los 12 tests estan
bien y verificados con comandos. La reserva es una y es de nivel **Commit**: el commit
`eb862432` **no incluye** uno de los ficheros que el propio task file lista.

#### Evidencia mecanica

| Comando | Resultado |
|---|---|
| `cargo nextest run -p vantadb --profile audit --lib -j 2 -E 'test(lsm::tests)'` | **6/6 passed** (incluye `fresh_segments_do_not_reserve_64mb` y `open_or_create_honours_initial_segment_bytes`) |
| `cargo test --lib` en `vantadb-node/` | **14/14 passed** (los 6 `build_config_*` nuevos incluidos) |
| `npx vitest run tests/find517.test.ts` (en `vantadb-node/`) | **5/5 passed** contra el binario real (motor de verdad, no mock) |
| `npx vitest run` (suite completa del binding) | **12 ficheros, 75/75 passed** |
| `cargo clippy -p vantadb --all-targets -j 2 -- -D warnings` | **exit 0** |
| `cargo clippy --all-targets` en `vantadb-node/` | **22 errores `unwrap_used`/`expect_used`, TODOS preexistentes** (`src/lib.rs:2036-2166`, modulo de tests antiguo) y **0 en los 6 tests nuevos** (usan `panic!`, no `unwrap`/`expect`) ⇒ el checklist #5 se acepta tal cual |
| `rustfmt --check src/lsm.rs vantadb-node/src/lib.rs` | **exit 0** |
| `git show eb862432 --stat` | 9 ficheros; **`vantadb-ts/src/native.ts` NO esta** (ver R1) |

#### Verificacion de los invariantes

| # | Invariante | Veredicto | Como |
|---|---|---|---|
| 1 | el contrato publico no se rompe (parametro aditivo) | ✅ | `reserve_bytes` es opcional; `Config::default()` no cambia para python/wasm/server/CLI. **Verificado ademas que no hay ruptura de semver**: `LsmConfig` vive en `pub(crate) mod lsm` (`src/lib.rs:123`) ⇒ no es parte de la superficie publica y `tests/api/public-api.txt` no la lista (`grep LsmConfig` => 0 hits) ⇒ el golden no se regenera y no se rompe nada |
| 2 | la reserva no crece por encima del `memory_limit` | ✅ | recorte en `build_config` (`vantadb-node/src/lib.rs:1417-1428`) + 2 tests unitarios + test de integracion |
| 3 | el tiempo de connect no empeora | ⚠️ no re-medido | la medicion A/B del implementador (720,0 -> 701,1 ms) no la he re-hecho; el cambio no toca el arranque del motor, solo el tamano del `set_len` inicial |
| 4 | el `.d.ts`/`.cjs`/`index.js` coherentes con el codigo | ✅ parcial | regenerados (0,8,0 -> 0,9,0) + `reserve_bytes` en el `.d.ts`; el alias `VantaDB` de napi-rs sigue pendiente (`FIND-546`), fuera de alcance |
| 5 | nada de `unsafe` nuevo | ✅ | 0 `unsafe` anadido; la frontera FFI no se ha tocado |
| — | la pre-reserva no es load-bearing | ✅ | `should_compact_level` compara el **cursor de escritura** (`src/storage/engine/maintenance.rs`), no el tamano del fichero |

#### Hallazgos (con fix minimo)

- **R1 (Requerido, nivel Commit) — `vantadb-ts/src/native.ts`:** el fichero que expone
  `NativeConnectOptions.reserve_bytes` (+ su doc) **no esta en el commit `eb862432`**:
  sigue como modificacion **sin commitear** en el working tree (`git status --short` =>
  ` M vantadb-ts/src/native.ts`). El task file lo lista en la tabla "Implementacion
  (step 3)", luego el **commit no es atomico** y el `git diff` no esta limpio (DoD nivel
  Commit). Efecto: el paquete `vantadb-ts` no expone el parametro aunque el binding de
  Node si. *Fix minimo*: `git add vantadb-ts/src/native.ts` y commitearlo (amend de
  `eb862432` o commit atomico aparte).
- **R2 (Opcional, doc) — `vantadb-node/dts-header.d.ts:47-53`:** "Clamped so the total
  never exceeds `memory_limit`" se puede leer como una garantia de **memoria**. La
  reserva es **espacio de direcciones**, no memoria viva (el propio task file lo dice:
  el RSS de un store vacio ya era ~2,3 MB). *Fix minimo*: una linea que diga que el
  recorte es una politica conservadora sobre el espacio reservado, no sobre el RSS.
- **R3 (Follow-up, respuesta al checklist #3) —** los **64 MiB del journal de fjall**
  (tercero, modulo privado) y los **64 MiB de `create_in_memory`** del motor in-memory
  (`src/storage/engine/init.rs:85`, heap *committeado* de verdad en `connect(":memory:")`)
  deben abrir **fila propia**: el segundo si es memoria viva y si sale en ningun
  `memory_limit` (a diferencia de la reserva de fjall, que es fichero disperso).
- **O1 (Opcional) — el riesgo principal de bajar el pre-reserva esta verificado:** el
  `set_len` inicial solo se aplica a ficheros **nuevos o < 64 B**
  (`src/storage/vfile.rs:407-418`: `if current_size < min_header_size { ... set_len }`)
  ⇒ **un segmento existente de 64 MiB NO se trunca** al abrir el store con el default
  nuevo. Sin truncado ni perdida de datos.
- **N1 (Nit, DoD) —** el task file no menciona la superficie publica (FIND-552 si lo
  hace). Verificado aqui: no cambia (ver invariante 1).

#### Lo que esta bien hecho

- **Corregir la premisa de la fila en vez de defenderla:** los "320,06 MB" no eran RSS
  sino espacio reservado, y por eso no aparecian en ningun `memory_limit`. Decirlo
  (y medir el RSS de verdad) es exactamente el trabajo que hace falta.
- La decision (b) esta bien argumentada frente a (a) y (c), y el argumento es
  economico, no ideologico: el problema es el **tamano**, no el **momento**.
- El recorte contra `memory_limit` con `min(reserve, limit/4)` y el suelo de
  `STORAGE_ALIGNMENT` cubren el borde (`memory_limit < 256 B`), y esta documentado.
- Los tests de integracion de vitest corren contra el **motor real** (no mocks) y el
  binario `.node` de disco contiene las cadenas `reserve_bytes must be a number` /
  `reserve_bytes must be greater than 0` ⇒ el binario es el **post-fix** (verificado).

## Notas
- Regla de git: el push requiere instruccion explicita del usuario (AGENTS.md Regla 7). **Este
  agente NO hace commits** (los hace el orquestador).
- `cargo fmt --check` global sigue en rojo por ficheros de **otros agentes** (`src/parser/mod.rs`,
  `tests/encryption_chaos.rs`): los ficheros de este task estan limpios (`rustfmt --check
  src/lsm.rs vantadb-node/src/lib.rs` ⇒ exit 0).
- **NOTICED BUT NOT TOUCHING:** `src/storage/engine/init.rs:85` (los 64 MiB del motor in-memory),
  el journal de fjall (crate de terceros), el alias `VantaDB` de napi-rs (FIND-546).
- La suite completa del lib (`cargo nextest run -p vantadb --profile audit --lib`) no se pudo
  **enlazar** en este entorno: el `target/` compartido lo usan a la vez otros agentes y el
  enlazador falla con `LNK1104` al no poder abrir `target/debug/deps/vantadb-12f11c63d48e715f.exe`.
  Se ejecuto el **binario de test ya construido** (13:19:40, que ya incluye este cambio) de
  forma directa: **2517 passed, 0 failed, 2 ignored** (69 s). Reintentar el comando de cargo
  cuando el lock este libre.
