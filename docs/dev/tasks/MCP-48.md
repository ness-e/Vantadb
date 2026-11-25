---
title: "MCP-48 - El proxy MCP re-elige writer en vez de morir con él"
kind: task
status: in-progress
description: "Si el writer HTTP cae, el proxy stdio se queda sin tools hasta reiniciar. Re-elección acotada (backoff) con promoción a writer bajo el lock fs2 y cleanup de discovery sólo por PID demostrablemente muerto."
---

# MCP-48 — El proxy re-elige writer (no muere con él)

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §9.b (ola MCP/MEM)
> **Ruta:** `vanta-worker` · **Carril:** R · **Appetite:** 4-6h
> **Dependencias:** ninguna (`MEM-72` toca otro crate; `MCP-43` es dependencia cruzada, ver §Incógnitas)

## Contexto

El modo proxy de `vantadb-mcp` (MCP-35) nace de un `DatabaseBusy`: el segundo
proceso no consigue el lock fs2, lee `.vanta.server.json`, valida PID + `/api/v2/health`
y se queda hablando HTTP contra ese writer **para siempre**
(`vantadb-mcp/src/server.rs:1023-1029`, `run_proxy_stdio_server`).

`ProxyHandle` se clona por request y nunca se re-resuelve. Si el writer muere:

| Paso | Hoy |
|---|---|
| writer muere | su `WriterGuard` borra `.vanta.server.json` (si `pid==self`) |
| proxy recibe `tools/call` | `proxy_tools_call` → `reqwest` connection refused |
| respuesta al cliente | error `-32603` por cada request, indefinidamente |
| salida del estado | sólo reiniciando el proceso (el cliente MCP no puede) |

Observado en producción el 2026-11-23: la sesión se queda **sin tools hasta
reiniciar**. El patrón es el mismo que el de cualquier leader election con lock
file: el líder muere y los seguidores no vuelven a mirar el lock.

Segundo defecto, en el mismo path: `cleanup_stale`
(`vantadb-mcp/src/proxy.rs:180`) **borra el discovery file sin mirar el PID**.
Se llama cuando `try_proxy` falló, y `try_proxy` falla también por un health
probe que expira a 500 ms con el writer vivo. Borrar el fichero de un writer
vivo rompe el discovery de todos los demás clientes. Hoy no llega a crear dos
writers porque la autoridad real es el lock fs2, no el fichero — pero el
`cleanup` no debe depender de esa suerte.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `vantadb-mcp/src/proxy.rs` (1-257) | `ServerInfo`, `is_pid_alive`, `ProxyHandle::try_connect`/`proxy_mcp_call`, `try_proxy`, `cleanup_stale`, 4 tests | es el archivo donde vive el defecto y todos sus call sites |
| `vantadb-mcp/src/server.rs` (1-1238) | `WriterGuard`, `write_discovery_file`, `spawn_writer_http`, `mcp_proxy_handler`, `run_stdio_server`, `dispatch_request_proxy`, `serve_lines_proxy`, `run_proxy_stdio_server`, `run_stdio_server_auto` | destino del cambio de rol y de la elección |
| `vantadb-mcp/src/lib.rs` (1-73) | re-exports de `proxy`/`server` (API pública del crate, `publish = false`) | qué cambia de firma |
| `vantadb-mcp/Cargo.toml` | features de `vantadb` (`server`, `fjall`, `fs2`, `sysinfo`), `tempfile` dev-dep | confirma que promoción a writer es posible sin features nuevos |
| `vantadb-mcp/tests/mcp_fallback_proxy.rs` (1-132) | 3 tests E2E del MCP-35 | patrón de test y contrato previo que no se debe romper |
| `src/storage/engine/init.rs:340-417` | adquisición del lock fs2 (`try_lock_exclusive`, retry, `DatabaseBusy`) | **la autoridad**: es lo que impide dos writers |
| `src/storage/engine/tests/read_only.rs:160-190` | lock tomado ⇒ `DatabaseBusy` | evidencia de que promover con el lock tomado falla cerrado |
| `.opencode/rules/server-mcp.md` (R-1..R-3), `.opencode/rules/concurrency-async.md` (R-1, R-2, R-3, R-7, R-8) | semáforo + `spawn_blocking`, nunca guard síncrono sobre `.await`, no bloquear el runtime | restricciones de la re-elección |
| `docs/api/MCP.md:92-102, 110-166` | Troubleshooting + tabla de códigos | dónde documentar y qué código usar (`-32001` BUSY retriable) |

**Referencias hacia dentro (callers)**

- `crate::proxy::try_proxy` → `vantadb-mcp/src/server.rs:1023` y 5 sitios de test. Firma intacta; la re-elección la usa.
- `cleanup_stale` → `server.rs:1033` (+ tests). Pasa a devolver `bool` (borrado sí/no); los callers que ignoran el retorno siguen compilando.
- `ProxyHandle` → `dispatch_request_proxy` y `tests/mcp_fallback_proxy.rs` (3 usos). **Sin cambios de firma**.
- `run_proxy_stdio_server` → exportado en `lib.rs:55`, llamado en `server.rs:1029`. Cambia su parámetro a `Arc<WriterProxy>`.
- `StorageEngine::open_with_config` → sólo se llama desde el path de promoción nuevo (misma firma que ya usa `run_stdio_server_auto:1004`).

**Referencias hacia fuera (depends)**

- `sysinfo` (liveness de PID), `reqwest` (health + forward), `fs2` vía `StorageEngine` (lock = autoridad), `axum` vía `spawn_writer_http` (writer HTTP al promover).

**Veredicto de impacto**

Blast radius acotado a `vantadb-mcp` (proceso que usan **todos** los clientes MCP).
El riesgo no es el diff: es el rol. Dos writers sobre la misma base = corrupción.
Por eso el criterio de limpieza es **PID demostrablemente muerto** y la promoción
pasa **siempre** por `StorageEngine::open_with_config` (lock fs2), nunca por
borrar un fichero y asumir que el lock está libre.

## Incógnitas / dependencia cruzada

- **`MCP-43` (scheduler de `vanta-memory` dentro de `--mcp` cuando el proceso es
  el writer)**: si el proceso se autopromueve a writer en caliente, el scheduler
  de consolidación debe arrancar también en ese momento. **No se resuelve aquí**
  (fuera de scope): MCP-48 deja `WriterProxy::role()` observable para que MCP-43
  enganche el arranque del scheduler cuando el rol pasa a `writer-promoted`.
- **Falso positivo documentado del cleanup**: PID reciclado por otro proceso. Un
  PID vivo que ya no es el writer bloquea la limpieza del fichero → el fichero se
  queda, pero el path de apertura lo **sobrescribe** al promover. Coste: un
  fichero obsoleto hasta la siguiente promoción, nunca un segundo writer.
- **Falso negativo (peor caso evitado)**: writer vivo pero health lento → PID vivo
  ⇒ **no** se borra nada (antes se borraba). El cliente recibe `-32001` retriable
  en vez de perder el discovery.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | `cleanup_stale` conservador (PID-only) + tests unitarios | ✅ |
| 2 | `WriterProxy`: handle cacheado, clasificación Ok/WriterError/Unreachable, re-elección con backoff acotado, promoción bajo lock, contadores | ✅ |
| 3 | `server.rs`: dispatch proxy contra `WriterProxy`, log periódico de rol/reconexiones | ✅ |
| 4 | Integración: writer vivo → proxy → matar writer → re-elección sin reiniciar; y no-doble-writer con lock tomado | ✅ |
| 5 | Docs + verify de contrato + review | ✅ |

### Hallazgo durante la implementación (rendimiento, no del plan)

`is_pid_alive` usaba `System::new_all()` + `refresh_all()`: enumerar todos los
procesos cuesta ~3,5 s en Windows, y la re-elección lo llama hasta 3 veces por
operación → el contrato "reintento acotado" tardaba 11 s. Corregido a refresh
**dirigido** (`refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]))`).
Medido: la operación fallida pasó de 11,17 s a <0,3 s (test
`live_writer_lock_blocks_promotion`, aserción `< 5 s`).
También se eliminó la copia muerta (`#[allow(dead_code)]`) en `server.rs`: una
sola implementación, re-exportada.


## Contrato mecánico

```
cargo nextest run --profile audit -p vantadb-mcp -E 'test(proxy) or test(writer) or test(discovery)' --build-jobs 2
```

## Spec

Decisión por ítem, con evidencia:

| Decisión | Elegido | Por qué (evidencia) |
|---|---|---|
| Criterio de writer muerto | **PID inexistente** (sysinfo) — nunca timeout ni heartbeat | `src/storage/engine/init.rs:390-413`: el lock fs2 ya resuelve la exclusividad; el fichero es sólo un directorio de servicios. Un timeout agresivo borra el discovery de un writer lento (bug real de hoy, `proxy.rs:180`). |
| ¿Quién es la autoridad? | **El lock fs2**, siempre | `init.rs:368` `try_lock_exclusive` + `tests/read_only.rs:185` (`DatabaseBusy` con lock tomado). La promoción abre storage: si otro writer vive, falla cerrado ⇒ imposible duplicar. |
| Re-elección | Redescubrir (health+PID) → si falla, promover a writer | Cubre los dos desenlaces del contrato (reconectar / re-elegir) sin reiniciar y sin inventar un coordinador externo. |
| Promoción | `StorageEngine::open_with_config` + `spawn_writer_http` | Reusa el 100 % del writer MCP-35 (router, discovery, `WriterGuard`) ⇒ un solo path de escritura, sin lógica duplicada. |
| Reintento | 3 intentos, sleeps 50/150 ms con `tokio::time::sleep`, techo ~200 ms + probes | Acotado y asíncrono: `concurrency-async.md` R-2/R-3 (nunca dormir bloqueando el runtime, nunca loop infinito). |
| Clasificación del fallo | 4 salidas: `Ok` / `WriterError` (el writer respondió) / `NotSent` (solo `is_connect()`: nunca llegó) / `Unknown` (timeout, 5xx, body ilegible) | Un error de validación del writer **no** es una muerte: re-elegir por cada `-32009` multiplicaría la latencia y podría promover sin necesidad. Y un timeout **no** prueba que la escritura no ocurrió: replayarla duplicaría mutaciones (hallazgo 2 del review). |
| Replay | sólo `NotSent` se reintenta tras re-elegir | `Unknown` devuelve `-32603` + `data.retriable:false` + hint: el cliente decide, no nosotros por él. |
| Cleanup | sólo PID demostrablemente muerto; **un fichero ilegible no se toca** | Un fichero a medio escribir (lectura parcial de un writer vivo) no es evidencia de muerte; borrarlo dejaba al writer vivo sin anuncio para siempre (hallazgo 1). Además `write_discovery_file` publica atómico (tmp + rename). |
| Promoción | sin pre-chequeo de "existe el fichero"; siempre intenta el lock | Un fichero que no se puede borrar (sharing violation, PID reciclado) deshabilitaba la promoción para siempre (hallazgo 4). El lock es la autoridad; el costo es ≤ `file_lock_timeout_ms`. |
| Single-flight | `tokio::sync::Mutex` alrededor de probe+promote | Dos requests contra un writer muerto no deben promover a la vez; el perdedor adopta el writer del ganador (hallazgo 7). |
| Estado observable | `tracing` (info al reconectar/promover, warn al perder) + contadores `reconnects()`/`resolve_failures()` + log periódico con rol | El contrato pide estado observable sin reiniciar; el log de 30 s ya existe en `run_stdio_server` y se replica en el proxy. |

## Review (P2-01, ronda 1 — `vanta-review`, contexto fresco)

**Veredicto inicial: `changes-required`** (8 hallazgos; 2 high, 2 medium, 4 low). Ninguno
cuestionaba el invariante central —el lock fs2 como autoridad, imposible duplicar writer—;
todos eran del camino de error. Estado tras la corrección:

| # | Hallazgo | Sev | Resolución |
|---|---|---|---|
| 1 | `cleanup_stale` borraba el fichero de un writer **vivo** si la lectura era parcial (escritura no atómica) | high | Escritura atómica (tmp + `sync_all` + `rename`) **y** ilegible ⇒ no se borra. Tests: `stale_discovery_cleanup_is_pid_only`, `unparseable_discovery_is_left_alone` |
| 2 | Un timeout re-ejecutaba `tools/call` (mutación duplicada) | high | `NotSent` vs `Unknown`: sólo se reintenta lo que provablemente no llegó. Test: `connect_failure_is_not_sent` |
| 3 | 4xx (auth/rate-limit) clasificado como "writer muerto" ⇒ `-32001 retriable` | medium | 4xx ⇒ `WriterError` con `retriable:false`; 5xx ⇒ `Unknown`. Test: `writer_refusal_is_not_a_death` |
| 4 | La promoción se deshabilitaba para siempre si el fichero no se podía borrar | medium | Sin pre-chequeo: siempre se intenta el lock. `live_writer_lock_blocks_promotion` ahora afirma el `DatabaseBusy` |
| 5 | El writer promovido no se recuperaba si su propio listener moría | low | `promote()` re-lanza `spawn_writer_http` sobre el engine retenido |
| 6 | Límites de la promoción (clave de cifrado, `read_only`) no documentados | low | Fila "Promotion limits" en `docs/api/MCP.md` |
| 7 | Promoción concurrente sin single-flight | low | `resolve_gate` + re-check del handle |
| 8 | `std::fs`/sysinfo en la tarea async | low | Los tres accesos van por `spawn_blocking` (R-1/R-7) |
| N1 | El guard del listener reiniciado borraba el fichero que acababa de escribir (identidad sólo por pid) | medium | `WriterGuard` compara **pid + puerto** (Drop y señal). Test: `writer_guard_drop_respects_port` |

## Verificación

- `cargo nextest run --profile audit -p vantadb-mcp -E 'test(proxy) or test(writer) or test(discovery)' --build-jobs 2` → **15/15 ✅**
- `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅ · `cargo fmt -p vantadb-mcp -- --check` ✅
- MCP-35 sin regresión: `cargo nextest run -p vantadb-mcp --test mcp_fallback_proxy` → **3/3 ✅**
- Test de integración `vantadb-mcp/tests/mcp_proxy_reconnect.rs`:
  - `proxy_re_elects_writer_without_restart` — writer vivo → proxy → writer muerto → la siguiente escritura promueve, va a storage real y el estado (rol, contador, discovery) es observable.
  - `live_writer_lock_blocks_promotion` — con el lock tomado por un writer vivo: `-32001` retriable, rol sin cambiar, sin doble writer (`open` → `DatabaseBusy`), y reconexión cuando el puerto vuelve.
  - `writer_refusal_is_not_a_death` — un 401 del writer no dispara re-elección ni se marca retriable.
  - `stale_discovery_cleanup_is_pid_only` — PID vivo ⇒ no se toca; PID muerto ⇒ se borra; ilegible ⇒ no se toca.

