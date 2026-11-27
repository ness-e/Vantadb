---
title: "FIND-782: 77 % del coste de re-elegir writer es el timeout de health x3 - se acota la fase de connect de la sonda"
kind: task
status: completed
description: "La resolución que falla cerrado paga 3 × 500 ms de timeout de health porque un puerto loopback cerrado tarda ~2 s en rechazar el connect. Se acota la fase de connect de la SONDA (150 ms) dejando intactos los 500 ms de respuesta (writer lento y vivo) y el cliente de las llamadas. Medido: 1,84 s → 0,78 s por tool call."
---

# FIND-782: el coste de re-elegir writer es la fase de `connect`, no la comprobación de PID

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (wave de cierre)
- **Fuente:** `docs/dev/Backlog.md` §`Media 2026-11-26 - La re-eleccion de writer gasta ~2,7 s por tool call…`
- **Origen:** `FIND-757` (medición por tramos) · **Ver:** `MCP-48`, `FIND-739`
- **Esfuerzo:** 🟢 2-3h · **Prioridad:** 🟡 Media · **Tipo:** Rust (perf) + tests
- **Creado:** 2026-11-26 · **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `WriterProxy::connect` (`proxy.rs:412`) · `WriterProxy::resolve_once` (`:548`) · `try_proxy` re-exportado (`lib.rs:54`) |
| Callees (tocados) | `ProxyHandle::try_connect` (`proxy.rs:116`) — dos `reqwest::Client` en vez de uno: **sonda** (connect acotado) y **llamada** (sin tocar) |
| Propiedad ajena | `vantadb-mcp/src/threads.rs` y `vanta-memory/src/services/scheduler.rs` son de `FIND-783` → **no se tocan** (0 hits en el diff) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/proxy.rs` (864 L) · `vantadb-mcp/tests/mcp_proxy_reconnect.rs` (456 L) · `vantadb-mcp/tests/mcp_fallback_proxy.rs` (132 L) · `vantadb-mcp/Cargo.toml` (58 L) · `.opencode/rules/server-mcp.md` (28 L) · `docs/dev/tasks/FIND-757.md` (§Diagnóstico) · `docs/dev/Backlog.md` (fila `FIND-782`) · `docs/dev/avance/activo/bindings.md` (§`MCP-48`, §`FIND-739`)
- **Referencias hacia dentro:** `try_connect` sólo lo llama `try_proxy` (`:296`). El `client` que devuelve el handle es el que usa `call_classified` (`:226`); `for_local_port` (`:185`) construye el suyo para los listeners propios.
- **Referencias entrantes:** `rg 'try_proxy\(|try_connect'` → 10 hits, todos listados arriba. Ningún otro crate construye un `ProxyHandle`.
- **Veredicto impacto:** medio-bajo, confinado a `vantadb-mcp/src/proxy.rs` + tests del área. Lo delicado no es el `connect_timeout` en sí: es **a qué cliente se le pone**. Ver §Decisión.

## Contrato

```
cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2 \
  -E 'test(proxy) or test(writer) or test(reelect) or test(reconnect) or test(stale) or test(discovery) or test(closed_port_probe) or test(connect_deadline_is_still_not_sent)'
```

…verde (**21/21**: los 18 del área + los 3 nuevos), más la medición antes/después del escenario "writer ausente" (N≥5, mediana) y la política de no-replay de `MCP-48` intacta: `Unknown` no replaya, `NotSent` sí, 4xx sigue siendo writer vivo.

> El contrato lleva `test(closed_port_probe)` explícito porque nextest matchea **nombre de función**, no binario: `test(find782)` devuelve **0 tests**. Sin esa cláusula el gate local era verde con el fix revertido (review P2-01, R2).

## Diagnóstico (premisa confirmada, no re-derivada)

`FIND-757` midió el reparto: **≈77 %** del reloj es el timeout de health (500 ms × 3), **≈9 %** `is_pid_alive`. Su causa raíz: `connect` a un puerto loopback **cerrado** no falla rápido. Confirmado en esta máquina, 5/5:

```
closed-port connect: 2191, 2037, 2019, 2024, 2024 ms   (mediana 2037 ms)
→ err WSAECONNREFUSED, es decir: rechaza, pero a los ~2 s
```

⇒ cada sonda gasta sus 500 ms completos en la fase de `connect` sin que la petición HTTP llegue a existir, y `RESOLVE_ATTEMPTS = 3` lo paga tres veces.

## Decisión: acotar `connect` **de la sonda**, y sólo de la sonda

| Palanca | Descartada por |
|---|---|
| Bajar los 500 ms de health a secas | pre-mortem (b): un writer **lento pero vivo** (payload grande, GC, disco) pierde su presupuesto de respuesta. El reloj medido de una sonda que **acierta** es 142-358 ms: el margen ya es 1,4x, no sobra |
| Deduplicar `is_pid_alive` | pre-mortem (a) y `FIND-757`: **84 ms de 1.950 (4,3 %)** y ensancha una API. No es la palanca |
| Bajar `RESOLVE_ATTEMPTS` | cambia la **paciencia antes de promover** — política, no coste (lo dice `bindings.md` §`FIND-739`). Fuera de contrato |
| `connect_timeout` en el cliente **de las llamadas** | no hacerlo **en este ticket**, pero no por la razón que escribí primero (review R1). Un `connect_timeout` **sí** sigue siendo replay-safe: el vencimiento lo emite el conector, `reqwest::Error::is_connect()` recorre la cadena y devuelve `true` ⇒ `NotSent` (lo pina `proxy::tests::connect_deadline_is_still_not_sent`). El motivo real es otro: cambia el **timing** del camino de replay. Hoy un writer muerto se manifiesta como rechazo lento y la llamada se replaya contra el writer re-elegido; con deadline, la misma llamada re-elege ~1,8 s antes y, sobre un writer **vivo con handshake lento**, reportaría un BUSY retriable espurio en vez de aterrizar. Es un trade que se mide aparte, no un regalo para meter de contrabando acá |
| Bajar el **timeout total** (de petición) en el cliente de llamadas | comería el presupuesto de respuesta del writer lento — es la misma palanca que bajar los 500 ms, con otro nombre |

**Aplicado:** `HEALTH_CONNECT_TIMEOUT_MS = 150` en el cliente que **sólo** hace las dos lecturas de descubrimiento (`/api/v2/health`, `/_admin/config`). El `ProxyHandle.client` (el de `POST /api/v2/mcp/proxy`) queda **byte-idéntico**. Lo que se acota es la fase que en loopback o es instantánea (listener vivo: kernel acepta en µs) o no ocurre nunca (puerto cerrado) — nunca la fase de respuesta, que sigue teniendo `REQUEST_TIMEOUT_MS = 500` (pinneado por `alive_but_silent_writer_keeps_the_reply_budget`).

## Steps

1. ✅ Medir `connect` crudo a puerto cerrado (5/5) — premisa confirmada.
2. ✅ Test RED `find782_probe_deadline::closed_port_probe_fails_on_the_connect_deadline` (mediana 525 ms > techo 400 ms).
3. ✅ Implementar el cliente de sonda con `connect_timeout` (2 clientes en `try_connect`; el de llamadas, intacto).
4. ✅ GREEN (mediana 168 ms) + medición antes/después del camino completo por tool call.
5. ✅ Review P2-01 (ronda 1, `vanta-review`, contexto fresco): 0 Blocking, 4 Required, 8 Optional → R1-R4 cerrados + O1/O2/O3/O5/O6 aplicados (ver §Review).
6. ✅ Verify (`nextest` contrato 21/21, `--lib` 62/62, `fmt`, `clippy -D warnings`) + commit local.

## Verify

```
cargo nextest run --profile audit -p vantadb-mcp --build-jobs 2 \
  -E 'test(proxy) or test(writer) or test(reelect) or test(reconnect) or test(stale) or test(discovery) or test(closed_port_probe) or test(connect_deadline_is_still_not_sent)'
→ 21 tests run: 21 passed
cargo nextest run --profile audit -p vantadb-mcp --lib --build-jobs 2
→ 62 tests run: 62 passed        (incluye connect_failure_is_not_sent, connect_deadline_is_still_not_sent)
cargo fmt -p vantadb-mcp --check → limpio
cargo clippy -p vantadb-mcp --all-targets --all-features -- -D warnings → limpio
```

**Poder de detección de los 3 tests nuevos (no vacuo, comprobado por mutación):** poner el `timeout` del cliente de sonda a 150 ms (la "simplificación" de fusionar los dos clientes) hace **FAIL** `alive_but_silent_writer_keeps_the_reply_budget` con `190.1ms < 400ms`; revertirlo lo vuelve verde. Y quitar `.connect_timeout(...)` hace FAIL `closed_port_probe_fails_on_the_connect_deadline` (~525 ms > 400 ms).

## Medición (escenario "writer ausente": PID anunciado **vivo** + puerto cerrado + lock fs2 tomado por otro writer)

`file_lock_timeout_ms: 50` (como `FIND-739`), N=5 por columna, mediana. Harness temporal (medido y borrado; el test permanente es `find782_probe_deadline`).

| | Antes | Después | Δ |
|---|---|---|---|
| `try_proxy` (una sonda) | **526 ms** `[526,5 · 525,6 · 518,7 · 528,3 · 526,4]` | **178 ms** `[180,1 · 167,4 · 183,1 · 178,6 · 177,1]` | **−66 %** |
| **tool call completa** (3 sondas + backoff 150 ms + `cleanup_stale` + promoción `DatabaseBusy`) | **1.841 ms** `[1852 · 1841 · 1835 · 1843 · 1823]` | **784 ms** `[788 · 778 · 801 · 781 · 784]` | **−57 %** |

El resto de los 784 ms es política, no coste oculto: 3 × 150 ms de deadline de connect + 150 ms de backoff (`RESOLVE_ATTEMPTS`/`BACKOFF_BASE_MS`) + `cleanup_stale` (~30-70 ms) + la paciencia de lock de la promoción (50 ms fijados por el test; 1000 ms por defecto en producción).

**Escenario que el test permanente fija** (una sonda, N=5): mediana **525 ms → 168 ms**, con el techo en 400 ms. Medición post-review: `[208,9 · 173,0 · 178,8 · 182,0 · 174,7] ms`, mediana **178,8 ms**.

## Deuda declarada

- Con el PID **realmente muerto** el camino ya era barato (corta antes del health) y **sigue sin medirse** — heredado de `FIND-757`, no lo toca este cambio.
- El `connect` de la fase de **llamada** sigue costando ~2 s cuando el writer muere con un handle instalado. **No** es una regresión de replay (un deadline ahí también sería `NotSent`, probado), pero tampoco es un cambio gratis: mueve el timing del camino de replay y necesita su propia medición. Queda como palanca abierta, con el motivo correcto escrito — no como "no lo toques".
- Los 3 × 150 ms restantes son la política de reintento (`RESOLVE_ATTEMPTS`), deliberadamente intacta.

## Review

- **Review P2-01 (ronda 1, agente distinto: `vanta-review`, contexto fresco): `changes-required` → atendido; 0 Blocking, 4 Required, 8 Optional.**
- **Enfoque:** el reviewer re-midió la premisa por su cuenta (`connect` crudo a 5 puertos cerrados: `2050 · 2038 · 2032 · 2013 · 2016 ms`) y re-corrió el contrato; leyó la fuente de `reqwest 0.12.28` / `hyper-util 0.1.20` del `Cargo.lock` para atacar la afirmación central.
- **R1 (el mecanismo citado era falso):** yo había escrito que un `connect_timeout` en el cliente de llamadas podía llegar como `Unknown` y romper el no-replay. **Falso con las versiones pinneadas**: el vencimiento lo emite el conector (`connect.rs:904-910`), `hyper-util` lo mapea a `ErrorKind::Connect` (`legacy/client.rs:514-517`) y `is_connect()` recorre la cadena ⇒ `NotSent`. Cerrado **convirtiéndolo en evidencia**: `proxy::tests::connect_deadline_is_still_not_sent` (10 líneas) lo pina, y el comentario/deuda/`bindings.md` se reescribieron con el motivo verdadero (timing del camino de replay, no política).
- **R2 (el contrato no ejecutaba el test nuevo):** `test(find782)` → **0 tests** (nextest matchea nombre de función); con el fix revertido el gate local quedaba verde. Añadido `test(closed_port_probe)` al contrato → 21/21.
- **R3 (faltaba el test del invariante "los 500 ms de respuesta no se tocan"):** añadido `alive_but_silent_writer_keeps_the_reply_budget` (listener que nunca hace `accept`; el kernel completa el handshake ⇒ respuesta muda ⇒ debe gastar el presupuesto de 500 ms, no el deadline). **Su poder se comprobó por mutación** (ver §Verify).
- **R4 (task file sin `## Review` y test sin trackear):** esta sección + el test entra en el commit.
- **Optional aplicados:** O1 techo 350 → 400 ms · O2 nota explícita de que el test de puerto cerrado es Windows-shaped (en Linux un RST inmediato haría pasar la forma vieja) · O3 `HEALTH_TIMEOUT_MS` → `REQUEST_TIMEOUT_MS` (se usa también en los clientes de llamada, donde el nombre engañaba) · O4 comentario del net de `mcp_proxy_reconnect` (pre-fix vs post-fix, ~13x) · O5 `tracing::debug!` con el error de la sonda (la clase de fallo nueva ya es diagnosticable) · O6 el test compara también `http_port`.
- **Optional no aplicados (declarados):** O7 TOCTOU `bind`→`drop`→`connect` — patrón preexistente (`proxy.rs:843`), falla ruidosamente; O8 discrepancias menores de prosa en el task file (corregidas las de la descripción y §Contrato/§Verify).
- **Cómo se probó:** contrato 21/21 · `--lib` 62/62 · mutación del cliente de sonda → `alive_but_silent` FAIL con número (190,1 ms) · `fmt --check` · `clippy -D warnings`.
- **Veredicto:** **approve** tras aplicar R1-R4.
