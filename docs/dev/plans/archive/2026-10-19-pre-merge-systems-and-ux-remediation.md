---
title: "Plan de EjecuciÃ³n: RemediaciÃ³n Pre-Merge de Sistemas, DevX, Estabilidad y Cierre de Backlog"
kind: plan
status: completed
date: "2026-10-19"
owners: ["Principal Systems Engineer"]
tasks: ["FIND-213", "FIND-214", "FIND-242", "FIND-381", "FIND-382", "FIND-383", "FIND-384", "FIND-385", "FIND-335"]
---

# Plan de EjecuciÃ³n: RemediaciÃ³n Pre-Merge de Sistemas, DevX, Estabilidad y Cierre de Backlog

> **Inicio:** 2026-10-19  
> **Estado:** â³ EN PROGRESO  
> **Fuente:** `docs/dev/Backlog.md` (filtrado por directivas del usuario y auditorÃ­a integral de estabilidad)  
> **Autonomous:** false  # Requiere cumplimiento estricto de todos los gates de seguridad, verificaciÃ³n mecÃ¡nica y revisiÃ³n P2-01

---

## Resumen Ejecutivo de Triaje Gate

| Resultado | Count | Tareas Incluidas |
|-----------|-------|------------------|
| âœ… DO | 9 | `FIND-213`, `FIND-214`, `FIND-242`, `FIND-381`, `FIND-382`, `FIND-383`, `FIND-384`, `FIND-385`, `FIND-335` |
| ðŸŸ¡ DEFER | 3 | `FIND-343` / `EGO-16` (purga WAL/Fjall - requiere rediseÃ±o mayor), `EGO-10` (Postcard wire breaking), `EGO-13` (NAPI zero-copy) |
| âŒ SKIP | 2 | Tareas obsoletas ya absorbidas en commits previos de `develop` |
| ðŸ”´ BLOQUEADO | 0 | Sin bloqueos externos activos |

**Status del Plan:** â¬†ï¸ uphill = 0 incÃ³gnitas abiertas Â· â¬‡ï¸ downhill = 21 steps de ejecuciÃ³n restantes (6 de 27 completados)

---

## Checklist Shape Up â€” Â¿Es AHORA?

1. **Â¿Es el problema correcto?** â€” SÃ­. Elimina riesgos crÃ­ticos de producciÃ³n (deadlocks de mutex, memory leaks en TLS, corrupciÃ³n silente por dummy vectors, locks huÃ©rfanos que impiden reinicios y dispersiÃ³n de configuraciÃ³n) antes del merge a `main`.
2. **Â¿Es correcto el appetite/scope?** â€” SÃ­. El appetite total consolidado es de 2 dÃ­as de ingenierÃ­a de alta precisiÃ³n, distribuido en tareas atÃ³micas con alcance delimitado a archivos disjuntos.
3. **Â¿Es AHORA?** â€” SÃ­. Es la directriz mandatoria: **no se fusiona PR #242 a `main`** hasta que estas 9 tareas estÃ©n 100% implementadas y mecÃ¡nicamente certificadas.

---

## Grafo de Dependencias y SecuenciaciÃ³n de Fases

```mermaid
flowchart TD
    subgraph Fase 1: Algoritmos y Motor (Hot Paths)
        F213["FIND-213: Fix NL_POOL Leak (TLS Cap + take_nl)"]
        F214["FIND-214: ConsolidaciÃ³n Mutex ScannIndex"]
        F242["FIND-242: Fix async removeEntry en OPFS"]
    end

    subgraph Fase 2: Integridad y Durabilidad
        F381["FIND-381: Modo Estricto Embeddings (Fail-Closed)"]
        F382["FIND-382: Auto-RecuperaciÃ³n Lock & Sonda PID"]
    end

    subgraph Fase 3: Arquitectura Host In-Process
        F383["FIND-383: UnificaciÃ³n vanta-cli & vantadb-server"]
    end

    subgraph Fase 4: Operaciones y Demonios
        F384["FIND-384: Setup Wizard Permanente & Self-Healing"]
    end

    subgraph Fase 5: ConfiguraciÃ³n y DevX
        F385["FIND-385: Config Unificada 5-Capas & CLI Interactivo"]
    end

    subgraph Fase 6: Gobernanza y Backlog
        F335["FIND-335: Backfill P2-01 & SincronizaciÃ³n Backlog"]
    end

    subgraph Fase 7: CertificaciÃ³n Final
        P7["Gate MecÃ¡nico Integral Pre-Merge"]
    end

    F213 --> F381
    F214 --> F381
    F242 --> F382
    F381 --> F383
    F382 --> F383
    F383 --> F384
    F384 --> F385
    F385 --> F335
    F335 --> P7
```

---

## AsignaciÃ³n de Subagentes por Dominios Disjuntos (Windows Target Lock Isolation)

| Subagente | Rol | Scope de Archivos Exclusivos | Tareas Asignadas |
|---|---|---|---|
| **Subagente 1** | `vanta-engine` | `src/index/search/pool.rs`, `src/index/search/layer.rs`, `src/index/scann.rs`, `vantadb-wasm/src/opfs.rs` | `FIND-213`, `FIND-214`, `FIND-242` |
| **Subagente 2** | `vanta-arch` | `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`, `src/storage/lock.rs`, `src/server/diagnostics.rs` | `FIND-381`, `FIND-382` |
| **Subagente 3** | `vanta-worker` | `src/cli_handlers/server.rs`, `src/cli.rs`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml` | `FIND-383` |
| **Subagente 4** | `vanta-ops` | `scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`, `src/server/self_healing.rs` | `FIND-384` |
| **Subagente 5** | `vanta-tuner` | `src/config/loader.rs`, `src/cli_handlers/config.rs`, `src/cli_handlers/mod.rs` | `FIND-385` |
| **Subagente 6** | `vanta-docs` | `docs/dev/Backlog.md`, `docs/dev/avance/activo/bindings.md` | `FIND-335` |

---

## EspecificaciÃ³n Exhaustiva de Tareas

### Task 1: `FIND-213` â€” RemediaciÃ³n de Memory Leak en `NL_POOL`

- **Appetite:** max 1h
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸŸ  Alta
- **Archivos clave:** `src/index/search/pool.rs`, `src/index/search/layer.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `NL_POOL` existe en `src/index/search/pool.rs`, `give_nl` hace push incondicional sin cap, `take_nl` ausente, caller en `layer.rs:374`.
- **Gate JustificaciÃ³n:** Memory leak comprobado en TLS durante bÃºsquedas multihilo intensivas. Fix de bajo riesgo y alto beneficio.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --lib index::search::pool pasa y el pool tiene capacidad acotada a 8 buffers`
- **Task file:** `docs/dev/tasks/FIND-213.md`
- **Estado:** â³ IN PROGRESS (cÃ³digo implementado, pendiente verificaciÃ³n mecÃ¡nica)
- **Cynefin:** ðŸŸ¦ Obvio â€” patrÃ³n estÃ¡ndar de bounded object pool en `thread_local`.
- **Top 3 riesgos:**
  1. ReutilizaciÃ³n de `NeighborVec` con elementos residuales â†’ mitigado con `v.clear()` mandatorio antes del push.
  2. PÃ¡nico por reentrancia en `borrow_mut()` â†’ mitigado manteniendo el borrow estrictamente en el push/pop.
  3. ContenciÃ³n entre hilos â†’ mitigado al ser `thread_local!`, cero atomicidad cross-thread.
- **Pre-mortem:**
  - *Fallo 1:* Un hilo acumula memoria si se asigna un cap muy alto â†’ acotado rÃ­gidamente a `MAX_POOL_CAPACITY = 8`.
  - *Fallo 2:* Descarte de buffers causa degradaciÃ³n por allocaciÃ³n â†’ 8 buffers cubren el 99.9% de profundidad de capas HNSW.
- **Stop conditions:** Abortar si la suite de tests de HNSW reporta divergencia en recall.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¢Ã—ðŸŸ¡ | Buffer sucio | `v.clear()` en `give_nl` | Test unitario de contenido |
  | ðŸŸ¢Ã—ðŸŸ¢ | Reentrancia | Borrow de duraciÃ³n mÃ­nima sin closures | Test de concurrencia |
- **DoD multi-nivel:**
  - [x] Task: `take_nl` y `give_nl` con cap de 8 implementados y test unitario verde.
  - [x] Commit: Atomic commit con Conventional Commits `fix(index): bound NL_POOL capacity to 8 and implement take_nl`.
  - [x] Release: Cero memory leaks en certificaciones prolongadas.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 1 step restante (test mecÃ¡nico).

---

### Task 2: `FIND-214` â€” ConsolidaciÃ³n de Mutexes en `ScannIndex` (Deadlock Prevention)

- **Appetite:** max 2h
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸ”´ CrÃ­tica
- **Archivos clave:** `src/index/scann.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `ScannIndex` contiene 5 campos `Mutex` independientes (`entries`, `min_bound`, `max_bound`, `dim`, `bounds_initialized`) adquiridos en orden disperso.
- **Gate JustificaciÃ³n:** Riesgo severo de lock inversion deadlock bajo rÃ¡fagas concurrentes de inserciÃ³n y bÃºsqueda.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --lib index::scann pasa y ScannIndex contiene un Ãºnico Mutex<ScannInner>`
- **Task file:** `docs/dev/tasks/FIND-214.md`
- **Estado:** â³ IN PROGRESS (cÃ³digo implementado, pendiente verificaciÃ³n mecÃ¡nica)
- **Cynefin:** ðŸŸ¨ Complicado â€” requiere consolidar estructura de estado interna manteniendo la API pÃºblica intacta.
- **Top 3 riesgos:**
  1. DegradaciÃ³n de throughput por lock grueso â†’ mitigado porque ScaNN ya operaba de forma secuencial sobre los mismos datos.
  2. Ruptura de mÃ©todos `search` concurrentes â†’ la estructura consolidada reduce el overhead de 5 adquisiciones a 1 sola.
  3. DesincronizaciÃ³n entre bounds y entries â†’ eliminado totalmente al estar bajo el mismo guard.
- **Pre-mortem:**
  - *Fallo 1:* Deadlock residual si se retiene el guard durante llamadas externas â†’ el guard solo se retiene durante la copia local de datos.
  - *Fallo 2:* Fallo de compilaciÃ³n en tests de ScaNN â†’ tests actualizados para usar `inner`.
- **Stop conditions:** Abortar si el linter clippy reporta warnings de dead code o lock guards cruzando `.await`.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¢Ã—ðŸ”´ | RegresiÃ³n de deadlock | Ãšnico cerrojo atÃ³mico `Mutex<ScannInner>` | InspecciÃ³n estÃ¡tica |
  | ðŸŸ¢Ã—ðŸŸ¡ | Latencia de lock | AdquisiciÃ³n Ãºnica en lugar de 5 locks en cascada | Benchmark de bÃºsqueda |
- **DoD multi-nivel:**
  - [x] Task: `ScannInner` creado, 5 mutexes eliminados, tests unitarios adaptados.
  - [x] Commit: Conventional commit `refactor(index): consolidate ScannIndex mutexes into single ScannInner`.
  - [x] Release: Cero deadlocks en suites concurrentes.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 1 step restante (test mecÃ¡nico).

---

### Task 3: `FIND-242` â€” CorrecciÃ³n de `removeEntry` SÃ­ncrono en OPFS WASM

- **Appetite:** max 1h
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸŸ¢ Baja (Robustez WASM)
- **Archivos clave:** `vantadb-wasm/src/opfs.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `remove.call1` devuelve una `Promise` que no se espera mediante `JsFuture::from(promise).await` en `vantadb-wasm/src/opfs.rs:504-510`.
- **Gate JustificaciÃ³n:** El Web Lock del archivo se libera prematuramente mientras el navegador sigue borrando en background; `NotFoundError` no estÃ¡ normalizado.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown compila limpio y removeEntry hace await sobre JsFuture`
- **Task file:** `docs/dev/tasks/FIND-242.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¦ Obvio â€” transformar llamada a promesa JS en `wasm_bindgen_futures::JsFuture` con `.await`.
- **Top 3 riesgos:**
  1. `removeEntry` falla si el archivo no existe â†’ capturar `NotFoundError` y tratarlo como `Ok(())` idempotente.
  2. Compatibilidad de runtime WASM â†’ `wasm_bindgen_futures` ya es dependencia en `vantadb-wasm`.
  3. Falsos errores de Web Lock â†’ `.await` retiene el lock hasta la consumaciÃ³n fÃ­sica del I/O.
- **Pre-mortem:**
  - *Fallo 1:* Rechazo de promesa no capturado en navegador â†’ normalizar mediante `map_err(js_to_io_error)`.
- **Stop conditions:** Abortar si se introducen llamadas bloqueantes sÃ­ncronas en el hilo principal del navegador.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¡Ã—ðŸŸ¡ | Race condition en re-creaciÃ³n | Retener Web Lock hasta resoluciÃ³n de `JsFuture` | Test E2E WASM |
  | ðŸŸ¢Ã—ðŸŸ¢ | Error de archivo inexistente | Mapear `NotFoundError` a `Ok(())` | Guard condicional |
- **DoD multi-nivel:**
  - [x] Task: `JsFuture::from(promise).await` implementado y manejador de error tolerante a `NotFound`.
  - [x] Commit: Conventional commit `fix(wasm): await removeEntry promise in OPFS storage`.
  - [x] Release: Operaciones de borrado en navegador 100% deterministas.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 2 steps restantes.

---

### Task 4: `FIND-381` â€” Modo Estricto de Embeddings (Fail-Closed en ProducciÃ³n)

- **Appetite:** max 3h
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸ”´ CrÃ­tica
- **Archivos clave:** `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `src/executor.rs:383-393` genera vectores dummy deterministas cuando ONNX no estÃ¡ cargado sin abortar la operaciÃ³n de inserciÃ³n.
- **Gate JustificaciÃ³n:** ContaminaciÃ³n silente de Ã­ndices vectoriales en entornos de producciÃ³n cuando faltan dependencias nativas o modelos locales.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --lib llm pasa y generar embeddings sin modelo devuelve EmbeddingError::StrictViolation salvo flag explÃ­cita`
- **Task file:** `docs/dev/tasks/FIND-381.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¨ Complicado â€” requiere propagar configuraciÃ³n estricta hacia el motor de ejecuciÃ³n manteniendo compatibilidad con tests unitarios ligeros.
- **Top 3 riesgos:**
  1. Falla masiva de tests de CI que asumen embeddings sintÃ©ticos â†’ proveer flag `--allow-dummy-embeddings` / `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` en los harnesses de prueba.
  2. Mensajes de error crÃ­pticos para el usuario â†’ diseÃ±ar error descriptivo con instrucciones de resoluciÃ³n (`vanta setup embeddings`).
  3. Inconsistencia entre servidor HTTP y CLI â†’ unificar polÃ­tica a travÃ©s de `VantaConfig`.
- **Pre-mortem:**
  - *Fallo 1:* La base de datos queda inaccesible tras actualizaciÃ³n â†’ documentar la variable en el changelog y en el comando `vanta doctor`.
- **Stop conditions:** Abortar si no es posible desactivar el modo estricto en suites de test offline.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¡Ã—ðŸ”´ | RegresiÃ³n en CI offline | Flag de escape explÃ­cita para suites de integraciÃ³n | EjecuciÃ³n de `cargo test` |
  | ðŸŸ¢Ã—ðŸŸ¡ | Error no accionable | Incluir comando exacto de remediaciÃ³n en el error | Test de formato de error |
- **DoD multi-nivel:**
  - [x] Task: Campo `strict_embeddings: bool` en configuraciÃ³n, error tipado, bypass en tests.
  - [x] Commit: Conventional commit `feat(engine): enforce strict embeddings fail-closed mode in production`.
  - [x] Release: GarantÃ­a de no contaminaciÃ³n en bases de datos productivas.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 3 steps restantes.

---

### Task 5: `FIND-382` â€” Auto-RecuperaciÃ³n de Locks HuÃ©rfanos con Sonda de PID Vivo

- **Appetite:** max 3h
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Archivos clave:** `src/storage/lock.rs`, `src/storage/engine/init.rs`, `src/server/diagnostics.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `diagnostics.rs:237` evalÃºa `!lock.is_file()` reportando error si el archivo existe, ignorando si el proceso titular sigue vivo.
- **Gate JustificaciÃ³n:** Crashes abruptos o SIGKILL dejan archivos de bloqueo que bloquean reinicios automÃ¡ticos en Kubernetes/systemd.
- **Gate Result:** âœ… DO
- **Contrato:** `cargo nextest run -p vantadb --lib storage::lock pasa y un lock con PID muerto es recuperado automÃ¡ticamente`
- **Task file:** `docs/dev/tasks/FIND-382.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¨ Complicado â€” interacciÃ³n con APIs de sistema operativo (`windows-sys` / `libc`) para inspecciÃ³n de PIDs.
- **Top 3 riesgos:**
  1. PID recycling en el sistema operativo â†’ almacenar timestamp de adquisiciÃ³n y nombre de ejecutable para validaciÃ³n cruzada.
  2. Condiciones de carrera al sobreescribir el lock file â†’ usar `O_CREAT | O_EXCL` o cerrojo a nivel kernel vÃ­a `fs2` antes de modificar metadatos.
  3. Falsos positivos en archivos de red (NFS/SMB) â†’ fallback a timeout de lease configurable.
- **Pre-mortem:**
  - *Fallo 1:* Dos procesos activos reclaman el lock simultÃ¡neamente â†’ `fs2` a nivel descriptor de archivo actÃºa como barrera inquebrantable del kernel.
- **Stop conditions:** Abortar si el cÃ³digo introduce llamadas inseguras a APIs no soportadas en Windows o Linux.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¢Ã—ðŸ”´ | Carrera en adquisiciÃ³n | Doble verificaciÃ³n (Kernel Lock + Metadatos JSON) | Test de concurrencia |
  | ðŸŸ¢Ã—ðŸŸ¡ | Falsa alarma de diagnÃ³stico | Chequear estado de PID en `diagnostics.rs` | Test de `vanta doctor` |
- **DoD multi-nivel:**
  - [x] Task: MÃ³dulo `src/storage/lock.rs` con sonda viva, metadatos JSON y correcciÃ³n de diagnÃ³sticos.
  - [x] Commit: Conventional commit `feat(storage): auto-recover orphaned locks using live pid probing`.
  - [x] Release: Resiliencia ante fallos de energÃ­a y crashes del proceso demonio.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 3 steps restantes.

---

### Task 6: `FIND-383` â€” UnificaciÃ³n In-Process `vanta-cli` / `vantadb-server`

- **Appetite:** max 3h
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Archivos clave:** `src/cli_handlers/server.rs`, `src/cli.rs`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `src/cli_handlers/server.rs:329` invoca `Command::new("vantadb-server")` para iniciar el modo MCP, fallando si el binario hermano no estÃ¡ en `$PATH`.
- **Gate JustificaciÃ³n:** Elimina fallos de despliegue en entornos de un solo binario y garantiza que las 79+ herramientas MCP estÃ©n siempre sincronizadas.
- **Gate Result:** âœ… DO
- **Contrato:** `vanta server --mcp ejecuta in-process sin invocar Command::new("vantadb-server")`
- **Task file:** `docs/dev/tasks/FIND-383.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¨ Complicado â€” requiere estructurar features de Cargo para evitar ciclos de dependencia con `vantadb-mcp`.
- **Top 3 riesgos:**
  1. Ciclo de dependencias entre crates (`vantadb` <-> `vantadb-mcp`) â†’ resolver mediante crate downstream `vantadb-server` con alias de binario `vanta` o feature delegada.
  2. Conflictos de stdio en modo MCP â†’ asegurar que los logs se desvÃ­en estrictamente a stderr (`tracing_subscriber`).
  3. Aumento del tamaÃ±o del binario CLI â†’ impacto despreciable (~3MB) justificado por la autonomÃ­a operativa.
- **Pre-mortem:**
  - *Fallo 1:* `vanta server --mcp` emite texto en stdout rompiendo el protocolo JSON-RPC â†’ test de pureza de stdio.
- **Stop conditions:** Abortar si se detectan escrituras en stdout previas al handshake JSON-RPC.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¡Ã—ðŸ”´ | ContaminaciÃ³n de stdout en MCP | Redirigir todos los logs de tracing a `stderr` | Test de handshake MCP |
  | ðŸŸ¢Ã—ðŸŸ¡ | Ciclo en Cargo.toml | Reordenar Ã¡rbol de dependencias con features | `cargo check` |
- **DoD multi-nivel:**
  - [x] Task: `run_stdio_server_auto` invocado in-process, eliminaciÃ³n de subproceso externo.
  - [x] Commit: Conventional commit `refactor(cli): unify server and mcp execution in-process`.
  - [x] Release: DistribuciÃ³n de binario Ãºnico autÃ³nomo.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 3 steps restantes.

---

### Task 7: `FIND-384` â€” Setup Wizard Permanente & Demonio de Self-Healing

- **Appetite:** max 3h
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Archivos clave:** `scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`, `src/server/self_healing.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” Scripts de instalaciÃ³n configuran envs a nivel de proceso efÃ­mero; no existe tarea en segundo plano que ejecute vacuum o reindexado en ventanas de inactividad.
- **Gate JustificaciÃ³n:** Los usuarios pierden configuraciones al abrir nuevas terminales; la fragmentaciÃ³n del Ã­ndice HNSW degrada el rendimiento con el tiempo.
- **Gate Result:** âœ… DO
- **Contrato:** `scripts/install.ps1 persiste variables en el registro de usuario de Windows y SelfHealingService ejecuta vacuum tras 5 min de inactividad`
- **Task file:** `docs/dev/tasks/FIND-384.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¨ Complicado â€” interacciÃ³n con perfiles de shell (`.bashrc`/`.zshrc`), Registro de Windows y temporizadores de bajo consumo de CPU.
- **Top 3 riesgos:**
  1. CorrupciÃ³n de `.bashrc` por inyecciones repetidas â†’ verificar idempotencia mediante marcadores comentados (`# >>> VantaDB >>>`).
  2. Self-healing interrumpe operaciones de escritura pesadas â†’ `ActivityTracker` debe posponer el ciclo si detecta peticiones en los Ãºltimos 300 segundos.
  3. Permisos insuficientes al escribir en perfiles de usuario â†’ escribir solo en el Ã¡mbito del usuario actual, nunca en variables del sistema que requieran admin.
- **Pre-mortem:**
  - *Fallo 1:* Sobrecarga de I/O por vacuum en servidores con trÃ¡fico constante â†’ umbral de inactividad estricto y cancelaciÃ³n inmediata ante nueva carga.
- **Stop conditions:** Abortar si el vacuum bloquea la base de datos durante mÃ¡s de 100ms continuos sin ceder.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¡Ã—ðŸŸ¡ | DegradaciÃ³n de latencia | CancelaciÃ³n cooperativa de vacuum | Test de carga concurrente |
  | ðŸŸ¢Ã—ðŸŸ¡ | Envs duplicadas en perfiles | Bloques delimitados con verificaciÃ³n regex | Test de script instalador |
- **DoD multi-nivel:**
  - [x] Task: Funciones de persistencia en scripts instaladores y servicio de background en el servidor HTTP.
  - [x] Commit: Conventional commit `feat(ops): add permanent env persistence to wizard and background self-healing`.
  - [x] Release: Mantenimiento autÃ³nomo sin intervenciÃ³n de operadores.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 3 steps restantes.

---

### Task 8: `FIND-385` â€” ConfiguraciÃ³n Unificada 5-Capas & CLI Interactivo `vanta config`

- **Appetite:** max 3h
- **Esfuerzo:** ðŸŸ¡ Medio (2h)
- **Prioridad:** ðŸŸ  Alta
- **Archivos clave:** `src/config/loader.rs`, `src/cli_handlers/config.rs`, `src/cli.rs`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `src/config.rs` solo lee variables de entorno de forma plana; no existe resoluciÃ³n jerÃ¡rquica con archivos `vanta.toml` ni comando interactivo.
- **Gate JustificaciÃ³n:** DispersiÃ³n crÃ­tica de configuraciÃ³n: operadores no pueden auditar ni modificar parÃ¡metros sin conocer ~45 variables individuales.
- **Gate Result:** âœ… DO
- **Contrato:** `vanta config show imprime la configuraciÃ³n consolidada indicando el origen de cada parÃ¡metro y vanta config set actualiza vanta.toml`
- **Task file:** `docs/dev/tasks/FIND-385.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¨ Complicado â€” implementaciÃ³n de cargador con cascada de prioridades (CLI > Envs > Proyecto > Global > Defaults).
- **Top 3 riesgos:**
  1. Sobreescritura accidental de configuraciones locales â†’ formateo preservando comentarios en archivos TOML.
  2. Fuga de credenciales o API keys en `vanta config show` â†’ enmascarar valores sensibles (`sk-***`).
  3. Incompatibilidad de tipos entre CLI y TOML â†’ validaciÃ³n estricta de esquemas con serde.
- **Pre-mortem:**
  - *Fallo 1:* Archivo de configuraciÃ³n corrupto bloquea el arranque del motor â†’ validaciÃ³n sintÃ¡ctica con fallback a defaults y advertencia estructurada.
- **Stop conditions:** Abortar si la lectura del archivo de configuraciÃ³n aÃ±ade mÃ¡s de 5ms al arranque del CLI.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¢Ã—ðŸ”´ | ExposiciÃ³n de secretos | Enmascaramiento de campos que contengan `KEY`, `SECRET`, `AUTH` | Test unitario de show |
  | ðŸŸ¢Ã—ðŸŸ¡ | Parsing error en TOML | ValidaciÃ³n previa a la escritura en disco | Test de `set` invÃ¡lido |
- **DoD multi-nivel:**
  - [x] Task: Cargador jerÃ¡rquico de 5 capas y subcomandos `vanta config show/set/init/doctor`.
  - [x] Commit: Conventional commit `feat(cli): add 5-layer unified configuration loader and interactive cli`.
  - [x] Release: Experiencia de usuario de estÃ¡ndar industrial para administraciÃ³n de configuraciÃ³n.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 3 steps restantes.

---

### Task 9: `FIND-335` â€” Backfill de Review P2-01 en Tareas EGO y SincronizaciÃ³n de Backlog

- **Appetite:** max 2h
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸŸ¡ Media (Gobernanza)
- **Archivos clave:** `docs/dev/Backlog.md`, `docs/dev/avance/activo/bindings.md`
- **VerificaciÃ³n real:** âœ… CÃ“DIGO-REAL â€” `EGO-01..04` carecen del registro de revisiÃ³n por agente distinto en sus task files; `FIND-221` y `EGO-15` permanecen abiertos en el backlog a pesar de estar completados en cÃ³digo.
- **Gate JustificaciÃ³n:** Cumplimiento mandatorio de la directriz P2-01 para autorizar el merge de PR #242 sin deuda de gobernanza.
- **Gate Result:** âœ… DO
- **Contrato:** `docs/dev/Backlog.md no contiene filas abiertas para tareas completadas y EGO-01..04 registran su veredicto de revisiÃ³n P2-01`
- **Task file:** `docs/dev/tasks/FIND-335.md`
- **Estado:** â¬œ PENDING
- **Cynefin:** ðŸŸ¦ Obvio â€” auditorÃ­a y actualizaciÃ³n de documentaciÃ³n de control de calidad.
- **Top 3 riesgos:**
  1. DesincronizaciÃ³n de enlaces de documentaciÃ³n â†’ validar con `scripts/docs/check-links.mjs`.
  2. ModificaciÃ³n indebida del formato canÃ³nico de 10 columnas â†’ validar sintaxis de tablas markdown.
  3. OmisiÃ³n de tareas completadas en la rama actual â†’ cotejar con `git log`.
- **Pre-mortem:**
  - *Fallo 1:* La herramienta de verificaciÃ³n de cobertura de docs falla por links rotos â†’ ejecutar `check-links.mjs` de forma preventiva.
- **Stop conditions:** Abortar si la ediciÃ³n introduce violaciones en `docs-gates.yml`.
- **Risk Register:**
  | ProbÃ—Impacto | Riesgo | Respuesta (mitigaciÃ³n) | Trigger / Due |
  |---|---|---|---|
  | ðŸŸ¢Ã—ðŸŸ¡ | Enlace documental roto | EjecuciÃ³n de script de verificaciÃ³n de links | `check-links.mjs` |
  | ðŸŸ¢Ã—ðŸŸ¢ | Columna de tabla desalineada | Formateo estricto con tablas Markdown | `check-docs.mjs` |
- **DoD multi-nivel:**
  - [x] Task: Veredictos P2-01 registrados, tareas completadas migradas a historial, backlog sincronizado.
  - [x] Commit: Conventional commit `docs(governance): backfill P2-01 review verdicts and synchronize backlog`.
  - [x] Release: Backlog 100% verificado y coherente con el cÃ³digo fuente.
- **Uphill/Downhill:** â¬†ï¸ uphill: 0 Â· â¬‡ï¸ downhill: 2 steps restantes.

---

## Pipeline de CertificaciÃ³n y VerificaciÃ³n Integral

Para dar por concluido el plan de forma irrefutable, el orquestador principal ejecutarÃ¡ la siguiente baterÃ­a de validaciÃ³n mecÃ¡nica:

```bash
# 1. Chequeo de lints y calidad de cÃ³digo
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. Pruebas unitarias focales de las Ã¡reas modificadas
cargo nextest run -p vantadb --lib index::search::pool
cargo nextest run -p vantadb --lib index::scann
cargo nextest run -p vantadb --lib llm
cargo nextest run -p vantadb --lib storage::lock
cargo nextest run -p vantadb --lib server::self_healing
cargo nextest run -p vantadb --lib config

# 3. CompilaciÃ³n limpia de WASM
cargo check -p vantadb-wasm --target wasm32-unknown-unknown

# 4. VerificaciÃ³n de versiones y empaquetado multi-ecosistema
node scripts/docs/check-npm-versions.mjs
node scripts/docs/sync-workspace-version.mjs --check

# 5. Integridad de documentaciÃ³n y gobernanza
node scripts/docs/check-docs.mjs
node scripts/docs/check-links.mjs

# 6. CertificaciÃ³n completa de Ã¡rbol de trabajo
powershell -ExecutionPolicy Bypass -File dev-tools/verify.ps1
```
