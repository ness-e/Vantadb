---
title: "Plan de Ejecución: Remediación Pre-Merge de Sistemas, DevX, Estabilidad y Cierre de Backlog"
kind: plan
status: in_progress
date: "2026-10-19"
owners: ["Principal Systems Engineer"]
tasks: ["FIND-213", "FIND-214", "FIND-242", "FIND-381", "FIND-382", "FIND-383", "FIND-384", "FIND-385", "FIND-335"]
---

# Plan de Ejecución: Remediación Pre-Merge de Sistemas, DevX, Estabilidad y Cierre de Backlog

> **Inicio:** 2026-10-19  
> **Estado:** ⏳ EN PROGRESO  
> **Fuente:** `docs/dev/Backlog.md` (filtrado por directivas del usuario y auditoría integral de estabilidad)  
> **Autonomous:** false  # Requiere cumplimiento estricto de todos los gates de seguridad, verificación mecánica y revisión P2-01

---

## Resumen Ejecutivo de Triaje Gate

| Resultado | Count | Tareas Incluidas |
|-----------|-------|------------------|
| ✅ DO | 9 | `FIND-213`, `FIND-214`, `FIND-242`, `FIND-381`, `FIND-382`, `FIND-383`, `FIND-384`, `FIND-385`, `FIND-335` |
| 🟡 DEFER | 3 | `FIND-343` / `EGO-16` (purga WAL/Fjall - requiere rediseño mayor), `EGO-10` (Postcard wire breaking), `EGO-13` (NAPI zero-copy) |
| ❌ SKIP | 2 | Tareas obsoletas ya absorbidas en commits previos de `develop` |
| 🔴 BLOQUEADO | 0 | Sin bloqueos externos activos |

**Status del Plan:** ⬆️ uphill = 0 incógnitas abiertas · ⬇️ downhill = 21 steps de ejecución restantes (6 de 27 completados)

---

## Checklist Shape Up — ¿Es AHORA?

1. **¿Es el problema correcto?** — Sí. Elimina riesgos críticos de producción (deadlocks de mutex, memory leaks en TLS, corrupción silente por dummy vectors, locks huérfanos que impiden reinicios y dispersión de configuración) antes del merge a `main`.
2. **¿Es correcto el appetite/scope?** — Sí. El appetite total consolidado es de 2 días de ingeniería de alta precisión, distribuido en tareas atómicas con alcance delimitado a archivos disjuntos.
3. **¿Es AHORA?** — Sí. Es la directriz mandatoria: **no se fusiona PR #242 a `main`** hasta que estas 9 tareas estén 100% implementadas y mecánicamente certificadas.

---

## Grafo de Dependencias y Secuenciación de Fases

```mermaid
flowchart TD
    subgraph Fase 1: Algoritmos y Motor (Hot Paths)
        F213["FIND-213: Fix NL_POOL Leak (TLS Cap + take_nl)"]
        F214["FIND-214: Consolidación Mutex ScannIndex"]
        F242["FIND-242: Fix async removeEntry en OPFS"]
    end

    subgraph Fase 2: Integridad y Durabilidad
        F381["FIND-381: Modo Estricto Embeddings (Fail-Closed)"]
        F382["FIND-382: Auto-Recuperación Lock & Sonda PID"]
    end

    subgraph Fase 3: Arquitectura Host In-Process
        F383["FIND-383: Unificación vanta-cli & vantadb-server"]
    end

    subgraph Fase 4: Operaciones y Demonios
        F384["FIND-384: Setup Wizard Permanente & Self-Healing"]
    end

    subgraph Fase 5: Configuración y DevX
        F385["FIND-385: Config Unificada 5-Capas & CLI Interactivo"]
    end

    subgraph Fase 6: Gobernanza y Backlog
        F335["FIND-335: Backfill P2-01 & Sincronización Backlog"]
    end

    subgraph Fase 7: Certificación Final
        P7["Gate Mecánico Integral Pre-Merge"]
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

## Asignación de Subagentes por Dominios Disjuntos (Windows Target Lock Isolation)

| Subagente | Rol | Scope de Archivos Exclusivos | Tareas Asignadas |
|---|---|---|---|
| **Subagente 1** | `vanta-engine` | `src/index/search/pool.rs`, `src/index/search/layer.rs`, `src/index/scann.rs`, `vantadb-wasm/src/opfs.rs` | `FIND-213`, `FIND-214`, `FIND-242` |
| **Subagente 2** | `vanta-arch` | `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`, `src/storage/lock.rs`, `src/server/diagnostics.rs` | `FIND-381`, `FIND-382` |
| **Subagente 3** | `vanta-worker` | `src/cli_handlers/server.rs`, `src/cli.rs`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml` | `FIND-383` |
| **Subagente 4** | `vanta-ops` | `scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`, `src/server/self_healing.rs` | `FIND-384` |
| **Subagente 5** | `vanta-tuner` | `src/config/loader.rs`, `src/cli_handlers/config.rs`, `src/cli_handlers/mod.rs` | `FIND-385` |
| **Subagente 6** | `vanta-docs` | `docs/dev/Backlog.md`, `docs/dev/avance/activo/bindings.md` | `FIND-335` |

---

## Especificación Exhaustiva de Tareas

### Task 1: `FIND-213` — Remediación de Memory Leak en `NL_POOL`

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠 Alta
- **Archivos clave:** `src/index/search/pool.rs`, `src/index/search/layer.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `NL_POOL` existe en `src/index/search/pool.rs`, `give_nl` hace push incondicional sin cap, `take_nl` ausente, caller en `layer.rs:374`.
- **Gate Justificación:** Memory leak comprobado en TLS durante búsquedas multihilo intensivas. Fix de bajo riesgo y alto beneficio.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --lib index::search::pool pasa y el pool tiene capacidad acotada a 8 buffers`
- **Task file:** `docs/dev/tasks/FIND-213.md`
- **Estado:** ⏳ IN PROGRESS (código implementado, pendiente verificación mecánica)
- **Cynefin:** 🟦 Obvio — patrón estándar de bounded object pool en `thread_local`.
- **Top 3 riesgos:**
  1. Reutilización de `NeighborVec` con elementos residuales → mitigado con `v.clear()` mandatorio antes del push.
  2. Pánico por reentrancia en `borrow_mut()` → mitigado manteniendo el borrow estrictamente en el push/pop.
  3. Contención entre hilos → mitigado al ser `thread_local!`, cero atomicidad cross-thread.
- **Pre-mortem:**
  - *Fallo 1:* Un hilo acumula memoria si se asigna un cap muy alto → acotado rígidamente a `MAX_POOL_CAPACITY = 8`.
  - *Fallo 2:* Descarte de buffers causa degradación por allocación → 8 buffers cubren el 99.9% de profundidad de capas HNSW.
- **Stop conditions:** Abortar si la suite de tests de HNSW reporta divergencia en recall.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟢×🟡 | Buffer sucio | `v.clear()` en `give_nl` | Test unitario de contenido |
  | 🟢×🟢 | Reentrancia | Borrow de duración mínima sin closures | Test de concurrencia |
- **DoD multi-nivel:**
  - [x] Task: `take_nl` y `give_nl` con cap de 8 implementados y test unitario verde.
  - [ ] Commit: Atomic commit con Conventional Commits `fix(index): bound NL_POOL capacity to 8 and implement take_nl`.
  - [ ] Release: Cero memory leaks en certificaciones prolongadas.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 1 step restante (test mecánico).

---

### Task 2: `FIND-214` — Consolidación de Mutexes en `ScannIndex` (Deadlock Prevention)

- **Appetite:** max 2h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🔴 Crítica
- **Archivos clave:** `src/index/scann.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `ScannIndex` contiene 5 campos `Mutex` independientes (`entries`, `min_bound`, `max_bound`, `dim`, `bounds_initialized`) adquiridos en orden disperso.
- **Gate Justificación:** Riesgo severo de lock inversion deadlock bajo ráfagas concurrentes de inserción y búsqueda.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --lib index::scann pasa y ScannIndex contiene un único Mutex<ScannInner>`
- **Task file:** `docs/dev/tasks/FIND-214.md`
- **Estado:** ⏳ IN PROGRESS (código implementado, pendiente verificación mecánica)
- **Cynefin:** 🟨 Complicado — requiere consolidar estructura de estado interna manteniendo la API pública intacta.
- **Top 3 riesgos:**
  1. Degradación de throughput por lock grueso → mitigado porque ScaNN ya operaba de forma secuencial sobre los mismos datos.
  2. Ruptura de métodos `search` concurrentes → la estructura consolidada reduce el overhead de 5 adquisiciones a 1 sola.
  3. Desincronización entre bounds y entries → eliminado totalmente al estar bajo el mismo guard.
- **Pre-mortem:**
  - *Fallo 1:* Deadlock residual si se retiene el guard durante llamadas externas → el guard solo se retiene durante la copia local de datos.
  - *Fallo 2:* Fallo de compilación en tests de ScaNN → tests actualizados para usar `inner`.
- **Stop conditions:** Abortar si el linter clippy reporta warnings de dead code o lock guards cruzando `.await`.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟢×🔴 | Regresión de deadlock | Único cerrojo atómico `Mutex<ScannInner>` | Inspección estática |
  | 🟢×🟡 | Latencia de lock | Adquisición única en lugar de 5 locks en cascada | Benchmark de búsqueda |
- **DoD multi-nivel:**
  - [x] Task: `ScannInner` creado, 5 mutexes eliminados, tests unitarios adaptados.
  - [ ] Commit: Conventional commit `refactor(index): consolidate ScannIndex mutexes into single ScannInner`.
  - [ ] Release: Cero deadlocks en suites concurrentes.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 1 step restante (test mecánico).

---

### Task 3: `FIND-242` — Corrección de `removeEntry` Síncrono en OPFS WASM

- **Appetite:** max 1h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja (Robustez WASM)
- **Archivos clave:** `vantadb-wasm/src/opfs.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `remove.call1` devuelve una `Promise` que no se espera mediante `JsFuture::from(promise).await` en `vantadb-wasm/src/opfs.rs:504-510`.
- **Gate Justificación:** El Web Lock del archivo se libera prematuramente mientras el navegador sigue borrando en background; `NotFoundError` no está normalizado.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check -p vantadb-wasm --target wasm32-unknown-unknown compila limpio y removeEntry hace await sobre JsFuture`
- **Task file:** `docs/dev/tasks/FIND-242.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio — transformar llamada a promesa JS en `wasm_bindgen_futures::JsFuture` con `.await`.
- **Top 3 riesgos:**
  1. `removeEntry` falla si el archivo no existe → capturar `NotFoundError` y tratarlo como `Ok(())` idempotente.
  2. Compatibilidad de runtime WASM → `wasm_bindgen_futures` ya es dependencia en `vantadb-wasm`.
  3. Falsos errores de Web Lock → `.await` retiene el lock hasta la consumación física del I/O.
- **Pre-mortem:**
  - *Fallo 1:* Rechazo de promesa no capturado en navegador → normalizar mediante `map_err(js_to_io_error)`.
- **Stop conditions:** Abortar si se introducen llamadas bloqueantes síncronas en el hilo principal del navegador.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟡×🟡 | Race condition en re-creación | Retener Web Lock hasta resolución de `JsFuture` | Test E2E WASM |
  | 🟢×🟢 | Error de archivo inexistente | Mapear `NotFoundError` a `Ok(())` | Guard condicional |
- **DoD multi-nivel:**
  - [ ] Task: `JsFuture::from(promise).await` implementado y manejador de error tolerante a `NotFound`.
  - [ ] Commit: Conventional commit `fix(wasm): await removeEntry promise in OPFS storage`.
  - [ ] Release: Operaciones de borrado en navegador 100% deterministas.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 2 steps restantes.

---

### Task 4: `FIND-381` — Modo Estricto de Embeddings (Fail-Closed en Producción)

- **Appetite:** max 3h
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🔴 Crítica
- **Archivos clave:** `src/config.rs`, `src/llm.rs`, `src/executor.rs`, `src/error.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `src/executor.rs:383-393` genera vectores dummy deterministas cuando ONNX no está cargado sin abortar la operación de inserción.
- **Gate Justificación:** Contaminación silente de índices vectoriales en entornos de producción cuando faltan dependencias nativas o modelos locales.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --lib llm pasa y generar embeddings sin modelo devuelve EmbeddingError::StrictViolation salvo flag explícita`
- **Task file:** `docs/dev/tasks/FIND-381.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado — requiere propagar configuración estricta hacia el motor de ejecución manteniendo compatibilidad con tests unitarios ligeros.
- **Top 3 riesgos:**
  1. Falla masiva de tests de CI que asumen embeddings sintéticos → proveer flag `--allow-dummy-embeddings` / `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` en los harnesses de prueba.
  2. Mensajes de error crípticos para el usuario → diseñar error descriptivo con instrucciones de resolución (`vanta setup embeddings`).
  3. Inconsistencia entre servidor HTTP y CLI → unificar política a través de `VantaConfig`.
- **Pre-mortem:**
  - *Fallo 1:* La base de datos queda inaccesible tras actualización → documentar la variable en el changelog y en el comando `vanta doctor`.
- **Stop conditions:** Abortar si no es posible desactivar el modo estricto en suites de test offline.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟡×🔴 | Regresión en CI offline | Flag de escape explícita para suites de integración | Ejecución de `cargo test` |
  | 🟢×🟡 | Error no accionable | Incluir comando exacto de remediación en el error | Test de formato de error |
- **DoD multi-nivel:**
  - [ ] Task: Campo `strict_embeddings: bool` en configuración, error tipado, bypass en tests.
  - [ ] Commit: Conventional commit `feat(engine): enforce strict embeddings fail-closed mode in production`.
  - [ ] Release: Garantía de no contaminación en bases de datos productivas.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 3 steps restantes.

---

### Task 5: `FIND-382` — Auto-Recuperación de Locks Huérfanos con Sonda de PID Vivo

- **Appetite:** max 3h
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Archivos clave:** `src/storage/lock.rs`, `src/storage/engine/init.rs`, `src/server/diagnostics.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `diagnostics.rs:237` evalúa `!lock.is_file()` reportando error si el archivo existe, ignorando si el proceso titular sigue vivo.
- **Gate Justificación:** Crashes abruptos o SIGKILL dejan archivos de bloqueo que bloquean reinicios automáticos en Kubernetes/systemd.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo nextest run -p vantadb --lib storage::lock pasa y un lock con PID muerto es recuperado automáticamente`
- **Task file:** `docs/dev/tasks/FIND-382.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado — interacción con APIs de sistema operativo (`windows-sys` / `libc`) para inspección de PIDs.
- **Top 3 riesgos:**
  1. PID recycling en el sistema operativo → almacenar timestamp de adquisición y nombre de ejecutable para validación cruzada.
  2. Condiciones de carrera al sobreescribir el lock file → usar `O_CREAT | O_EXCL` o cerrojo a nivel kernel vía `fs2` antes de modificar metadatos.
  3. Falsos positivos en archivos de red (NFS/SMB) → fallback a timeout de lease configurable.
- **Pre-mortem:**
  - *Fallo 1:* Dos procesos activos reclaman el lock simultáneamente → `fs2` a nivel descriptor de archivo actúa como barrera inquebrantable del kernel.
- **Stop conditions:** Abortar si el código introduce llamadas inseguras a APIs no soportadas en Windows o Linux.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟢×🔴 | Carrera en adquisición | Doble verificación (Kernel Lock + Metadatos JSON) | Test de concurrencia |
  | 🟢×🟡 | Falsa alarma de diagnóstico | Chequear estado de PID en `diagnostics.rs` | Test de `vanta doctor` |
- **DoD multi-nivel:**
  - [ ] Task: Módulo `src/storage/lock.rs` con sonda viva, metadatos JSON y corrección de diagnósticos.
  - [ ] Commit: Conventional commit `feat(storage): auto-recover orphaned locks using live pid probing`.
  - [ ] Release: Resiliencia ante fallos de energía y crashes del proceso demonio.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 3 steps restantes.

---

### Task 6: `FIND-383` — Unificación In-Process `vanta-cli` / `vantadb-server`

- **Appetite:** max 3h
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Archivos clave:** `src/cli_handlers/server.rs`, `src/cli.rs`, `vantadb-server/src/main.rs`, `vantadb-server/Cargo.toml`
- **Verificación real:** ✅ CÓDIGO-REAL — `src/cli_handlers/server.rs:329` invoca `Command::new("vantadb-server")` para iniciar el modo MCP, fallando si el binario hermano no está en `$PATH`.
- **Gate Justificación:** Elimina fallos de despliegue en entornos de un solo binario y garantiza que las 79+ herramientas MCP estén siempre sincronizadas.
- **Gate Result:** ✅ DO
- **Contrato:** `vanta server --mcp ejecuta in-process sin invocar Command::new("vantadb-server")`
- **Task file:** `docs/dev/tasks/FIND-383.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado — requiere estructurar features de Cargo para evitar ciclos de dependencia con `vantadb-mcp`.
- **Top 3 riesgos:**
  1. Ciclo de dependencias entre crates (`vantadb` <-> `vantadb-mcp`) → resolver mediante crate downstream `vantadb-server` con alias de binario `vanta` o feature delegada.
  2. Conflictos de stdio en modo MCP → asegurar que los logs se desvíen estrictamente a stderr (`tracing_subscriber`).
  3. Aumento del tamaño del binario CLI → impacto despreciable (~3MB) justificado por la autonomía operativa.
- **Pre-mortem:**
  - *Fallo 1:* `vanta server --mcp` emite texto en stdout rompiendo el protocolo JSON-RPC → test de pureza de stdio.
- **Stop conditions:** Abortar si se detectan escrituras en stdout previas al handshake JSON-RPC.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟡×🔴 | Contaminación de stdout en MCP | Redirigir todos los logs de tracing a `stderr` | Test de handshake MCP |
  | 🟢×🟡 | Ciclo en Cargo.toml | Reordenar árbol de dependencias con features | `cargo check` |
- **DoD multi-nivel:**
  - [ ] Task: `run_stdio_server_auto` invocado in-process, eliminación de subproceso externo.
  - [ ] Commit: Conventional commit `refactor(cli): unify server and mcp execution in-process`.
  - [ ] Release: Distribución de binario único autónomo.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 3 steps restantes.

---

### Task 7: `FIND-384` — Setup Wizard Permanente & Demonio de Self-Healing

- **Appetite:** max 3h
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Archivos clave:** `scripts/install.ps1`, `scripts/install.sh`, `scripts/setup-embeddings.ps1`, `src/server/self_healing.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — Scripts de instalación configuran envs a nivel de proceso efímero; no existe tarea en segundo plano que ejecute vacuum o reindexado en ventanas de inactividad.
- **Gate Justificación:** Los usuarios pierden configuraciones al abrir nuevas terminales; la fragmentación del índice HNSW degrada el rendimiento con el tiempo.
- **Gate Result:** ✅ DO
- **Contrato:** `scripts/install.ps1 persiste variables en el registro de usuario de Windows y SelfHealingService ejecuta vacuum tras 5 min de inactividad`
- **Task file:** `docs/dev/tasks/FIND-384.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado — interacción con perfiles de shell (`.bashrc`/`.zshrc`), Registro de Windows y temporizadores de bajo consumo de CPU.
- **Top 3 riesgos:**
  1. Corrupción de `.bashrc` por inyecciones repetidas → verificar idempotencia mediante marcadores comentados (`# >>> VantaDB >>>`).
  2. Self-healing interrumpe operaciones de escritura pesadas → `ActivityTracker` debe posponer el ciclo si detecta peticiones en los últimos 300 segundos.
  3. Permisos insuficientes al escribir en perfiles de usuario → escribir solo en el ámbito del usuario actual, nunca en variables del sistema que requieran admin.
- **Pre-mortem:**
  - *Fallo 1:* Sobrecarga de I/O por vacuum en servidores con tráfico constante → umbral de inactividad estricto y cancelación inmediata ante nueva carga.
- **Stop conditions:** Abortar si el vacuum bloquea la base de datos durante más de 100ms continuos sin ceder.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟡×🟡 | Degradación de latencia | Cancelación cooperativa de vacuum | Test de carga concurrente |
  | 🟢×🟡 | Envs duplicadas en perfiles | Bloques delimitados con verificación regex | Test de script instalador |
- **DoD multi-nivel:**
  - [ ] Task: Funciones de persistencia en scripts instaladores y servicio de background en el servidor HTTP.
  - [ ] Commit: Conventional commit `feat(ops): add permanent env persistence to wizard and background self-healing`.
  - [ ] Release: Mantenimiento autónomo sin intervención de operadores.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 3 steps restantes.

---

### Task 8: `FIND-385` — Configuración Unificada 5-Capas & CLI Interactivo `vanta config`

- **Appetite:** max 3h
- **Esfuerzo:** 🟡 Medio (2h)
- **Prioridad:** 🟠 Alta
- **Archivos clave:** `src/config/loader.rs`, `src/cli_handlers/config.rs`, `src/cli.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `src/config.rs` solo lee variables de entorno de forma plana; no existe resolución jerárquica con archivos `vanta.toml` ni comando interactivo.
- **Gate Justificación:** Dispersión crítica de configuración: operadores no pueden auditar ni modificar parámetros sin conocer ~45 variables individuales.
- **Gate Result:** ✅ DO
- **Contrato:** `vanta config show imprime la configuración consolidada indicando el origen de cada parámetro y vanta config set actualiza vanta.toml`
- **Task file:** `docs/dev/tasks/FIND-385.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟨 Complicado — implementación de cargador con cascada de prioridades (CLI > Envs > Proyecto > Global > Defaults).
- **Top 3 riesgos:**
  1. Sobreescritura accidental de configuraciones locales → formateo preservando comentarios en archivos TOML.
  2. Fuga de credenciales o API keys en `vanta config show` → enmascarar valores sensibles (`sk-***`).
  3. Incompatibilidad de tipos entre CLI y TOML → validación estricta de esquemas con serde.
- **Pre-mortem:**
  - *Fallo 1:* Archivo de configuración corrupto bloquea el arranque del motor → validación sintáctica con fallback a defaults y advertencia estructurada.
- **Stop conditions:** Abortar si la lectura del archivo de configuración añade más de 5ms al arranque del CLI.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟢×🔴 | Exposición de secretos | Enmascaramiento de campos que contengan `KEY`, `SECRET`, `AUTH` | Test unitario de show |
  | 🟢×🟡 | Parsing error en TOML | Validación previa a la escritura en disco | Test de `set` inválido |
- **DoD multi-nivel:**
  - [ ] Task: Cargador jerárquico de 5 capas y subcomandos `vanta config show/set/init/doctor`.
  - [ ] Commit: Conventional commit `feat(cli): add 5-layer unified configuration loader and interactive cli`.
  - [ ] Release: Experiencia de usuario de estándar industrial para administración de configuración.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 3 steps restantes.

---

### Task 9: `FIND-335` — Backfill de Review P2-01 en Tareas EGO y Sincronización de Backlog

- **Appetite:** max 2h
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 Media (Gobernanza)
- **Archivos clave:** `docs/dev/Backlog.md`, `docs/dev/avance/activo/bindings.md`
- **Verificación real:** ✅ CÓDIGO-REAL — `EGO-01..04` carecen del registro de revisión por agente distinto en sus task files; `FIND-221` y `EGO-15` permanecen abiertos en el backlog a pesar de estar completados en código.
- **Gate Justificación:** Cumplimiento mandatorio de la directriz P2-01 para autorizar el merge de PR #242 sin deuda de gobernanza.
- **Gate Result:** ✅ DO
- **Contrato:** `docs/dev/Backlog.md no contiene filas abiertas para tareas completadas y EGO-01..04 registran su veredicto de revisión P2-01`
- **Task file:** `docs/dev/tasks/FIND-335.md`
- **Estado:** ⬜ PENDING
- **Cynefin:** 🟦 Obvio — auditoría y actualización de documentación de control de calidad.
- **Top 3 riesgos:**
  1. Desincronización de enlaces de documentación → validar con `scripts/docs/check-links.mjs`.
  2. Modificación indebida del formato canónico de 10 columnas → validar sintaxis de tablas markdown.
  3. Omisión de tareas completadas en la rama actual → cotejar con `git log`.
- **Pre-mortem:**
  - *Fallo 1:* La herramienta de verificación de cobertura de docs falla por links rotos → ejecutar `check-links.mjs` de forma preventiva.
- **Stop conditions:** Abortar si la edición introduce violaciones en `docs-gates.yml`.
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta (mitigación) | Trigger / Due |
  |---|---|---|---|
  | 🟢×🟡 | Enlace documental roto | Ejecución de script de verificación de links | `check-links.mjs` |
  | 🟢×🟢 | Columna de tabla desalineada | Formateo estricto con tablas Markdown | `check-docs.mjs` |
- **DoD multi-nivel:**
  - [ ] Task: Veredictos P2-01 registrados, tareas completadas migradas a historial, backlog sincronizado.
  - [ ] Commit: Conventional commit `docs(governance): backfill P2-01 review verdicts and synchronize backlog`.
  - [ ] Release: Backlog 100% verificado y coherente con el código fuente.
- **Uphill/Downhill:** ⬆️ uphill: 0 · ⬇️ downhill: 2 steps restantes.

---

## Pipeline de Certificación y Verificación Integral

Para dar por concluido el plan de forma irrefutable, el orquestador principal ejecutará la siguiente batería de validación mecánica:

```bash
# 1. Chequeo de lints y calidad de código
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. Pruebas unitarias focales de las áreas modificadas
cargo nextest run -p vantadb --lib index::search::pool
cargo nextest run -p vantadb --lib index::scann
cargo nextest run -p vantadb --lib llm
cargo nextest run -p vantadb --lib storage::lock
cargo nextest run -p vantadb --lib server::self_healing
cargo nextest run -p vantadb --lib config

# 3. Compilación limpia de WASM
cargo check -p vantadb-wasm --target wasm32-unknown-unknown

# 4. Verificación de versiones y empaquetado multi-ecosistema
node scripts/docs/check-npm-versions.mjs
node scripts/docs/sync-workspace-version.mjs --check

# 5. Integridad de documentación y gobernanza
node scripts/docs/check-docs.mjs
node scripts/docs/check-links.mjs

# 6. Certificación completa de árbol de trabajo
powershell -ExecutionPolicy Bypass -File dev-tools/verify.ps1
```
