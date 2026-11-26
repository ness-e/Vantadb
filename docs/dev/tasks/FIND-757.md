---
title: "FIND-757: el `<0,3 s` de `MCP-48` mide una sonda, no la resolución — y `is_pid_alive` cuesta 24-34 ms, no 300"
kind: task
status: active
description: "Las dos cifras eran ciertas midiendo cosas distintas: `is_pid_alive` aislado cuesta 24-34 ms (no 300), una sonda que acierta 142-358 ms y la resolución completa 1,90-2,40 s, dominada por el timeout de health de 500 ms pagado 3 veces — no por la comprobación de PID (~9 %). Desenlace (ii): se corrige la redacción, no el código"
---

# FIND-757: el `<0,3 s` de `MCP-48` mide una sonda, no la resolución

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md` (fila 6, W2, carril R)
- **Fuente:** `docs/dev/Backlog.md` §`Media 2026-11-26 - El <0,3 s de MCP-48 no se reproduce`
- **Esfuerzo:** 🟢 2-3h
- **Prioridad:** 🟡 Media
- **Tipo:** Rust (medición) + docs
- **Creado:** 2026-11-26
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno: el diff es un task file + las frases de `bindings.md` + la fila del Backlog + un **comentario** en el test. Cero código de producción. |
| Callees (medidos, no modificados) | `vantadb-mcp::is_pid_alive` (`proxy.rs:69`) · `try_proxy` (`:284`) → `ProxyHandle::try_connect` (`:116`) → health `GET /api/v2/health` con timeout 500 ms (`:127`) · `cleanup_stale` (`:321`) · `WriterProxy::call` → `resolve_with_backoff` (`:522`) → `resolve_once` (`:544`) → `promote` (`:575`) |
| Implicaciones | Ninguna sobre producto: no se toca `vantadb-mcp/src/**`. Se corrige una atribución de coste publicada (el reparto real del reloj) y se deja la medición reproducible en este task file. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/proxy.rs` (864 L) · `vantadb-mcp/tests/mcp_proxy_reconnect.rs` (450 L) · `vantadb-mcp/Cargo.toml` (58 L) · `docs/dev/tasks/FIND-739.md` (125 L) · `docs/dev/avance/activo/bindings.md:1679-1688` (§`MCP-48`) y `:1736-1749` (§`FIND-739`) · `docs/dev/Backlog.md:800-808`
- **Referencias hacia dentro:** `lib.rs:54` re-exporta `is_pid_alive`/`try_proxy`/`cleanup_stale` (API del crate, la usada por la medición). `is_pid_alive` sólo se llama desde `try_connect` (`:119`) y `cleanup_stale` (`:333`).
- **Referencias entrantes:** `rg FIND-757` → `docs/dev/Backlog.md`, plan `2026-11-26-batch-parciales-y-nuevas.md`, este task file. Ningún `.rs` referencia el ID.
- **Veredicto impacto:** bajo — docs + un **comentario** en el test. La instrumentación de medición entró y salió del árbol de trabajo; lo único que queda en `vantadb-mcp/tests/mcp_proxy_reconnect.rs` es la corrección del comentario que afirmaba "~300 ms" (sin cambio de lógica ni de aserciones), y `vantadb-mcp/src/**` sigue intacto.

## Contrato

```
cargo nextest run --profile audit -p vantadb-mcp \
  -E 'test(live_writer_lock_blocks_promotion)' --build-jobs 2
```
sigue verde. Más: **las dos mediciones por separado**, con las cifras pegadas (abajo). Si el desenlace hubiera sido (i) —código patológicamente lento— el commit habría incluido `vantadb-mcp/src/proxy.rs` con medición antes/después y 20 corridas del filtro, como `FIND-739`. **No** es el caso.

## Diagnóstico (medición antes de decidir)

### (a) `is_pid_alive` aislado — 3 procesos, `N=20` con PID vivo + `N=10` con PID muerto

| Proceso | 1ª llamada (frío) | Caliente, PID vivo (min/mediana/max) | Caliente, PID muerto (mediana) |
|---|---|---|---|
| 1 | 34,1 ms | 18,0 / 26,6 / 53,3 ms (media 28,6) | 31,4 ms |
| 2 | 27,6 ms | 17,1 / 23,8 / 38,5 ms (media 23,8) | 21,5 ms |
| 3 (bajo carga) | 50,2 ms | 24,1 / 32,8 / 67,6 ms (media 34,1) | 31,2 ms |

⇒ **~24-34 ms típico**, máximo observado 68 ms. **No** los ~300 ms que atribuyó `FIND-739` (que nunca midió esta llamada aislada: dividió 1,9 s entre 6).

### (b) Resolución completa de 3 sondas, punta a punta, `file_lock_timeout_ms: 50` como en `FIND-739`

| Corrida | `elapsed` |
|---|---|
| 1 | 1,904 s |
| 2 | 1,946 s |
| 3 | 1,955 s |
| 4 | 2,015 s |
| 5 | 2,046 s |
| 6 | 2,071 s |
| 7 | 2,135 s |
| 8 | 2,151 s |
| 9 | 2,396 s |

⇒ **1,90-2,40 s**, compatible con lo que midió `FIND-739` (1,93-2,10 s): la cifra se reproduce. **Procedencia:** harness temporal `#[ignore]` **que replica la config del test** (misma `Config { file_lock_timeout_ms: 50 }`, mismo discovery de PID vivo + puerto cerrado, misma llamada a `proxy.call`) — el test real no imprime `elapsed` cuando pasa (`:342-346`), así que su `junit.xml` no puede confirmar la cifra interna. **Carga:** las corridas 1-6 se tomaron en una ventana más tranquila; 7-9 con otros carriles de este checkout compilando en paralelo (de ahí el 2,396 s). Ese contraste es real y **subestima** el ruido: el mismo test tardó **9,1 s** (corrida del review) y **15,4 s** (`junit.xml` 13:24:46) de duración total en la misma máquina, 4-7× la ventana de la serie.

### Dónde va el tiempo (instrumentación de la sonda, `N=3` por corrida)

| Tramo | Coste medido |
|---|---|
| `try_proxy` **fallando** (PID vivo, puerto cerrado) | **530-800 ms** ⇒ de ahí ~500 ms son el **timeout de health** |
| `cleanup_stale` | 23-77 ms ⇒ un `is_pid_alive` |
| `try_proxy` **acertando** (PID vivo + health 200 + `/_admin/config`) | **142-358 ms** (1ª de 3: 358 ms; luego 142 / 164) |
| `GET /api/v2/health` crudo a un puerto cerrado (timeout 500 ms) | **510-525 ms** (el timeout, no un rechazo) |
| backoff entre sondas | 50 + 100 = 150 ms |
| promoción con `DatabaseBusy` (paciencia 50 ms) | ~50 ms |
| `reqwest::Client::build()` | 0,03-2,4 ms (no es el coste) |
| `TcpStream::connect` crudo a puerto loopback cerrado | **~2,04 s** (¡no rechaza!) |

**Reparto aproximado del 1,95 s** (cuotas indicativas, no suman exacto: el GET al health varía 510-525 ms por sonda y la promoción tiene jitter): 3 × ~500 ms de health timeout ≈ **1,5 s (≈77 %)** + 6 × ~28 ms de `is_pid_alive` ≈ **0,17 s (≈9 %)** + 150 ms de backoff + ~50 ms de lock. La conclusión **no depende de la división**: la sostienen las mediciones directas por sonda (530-800 ms fallando) y el GET crudo (510-525 ms). La causa del timeout es que **en esta máquina un puerto loopback cerrado no devuelve RST**: el `connect` crudo tarda ~2,0 s, así que la sonda siempre agota sus 500 ms.

**Precondición del escenario (importa):** los 1,90-2,40 s son con el **PID anunciado vivo y su puerto cerrado** — lo que el test fija a propósito (`tests/mcp_proxy_reconnect.rs:234-239`, "a live PID at a port that refuses connections"). Con el PID **realmente muerto**, `try_connect` corta **antes** del health request (`proxy.rs:119-125`) y `cleanup_stale` borra el fichero en la sonda 1, así que las sondas 2-3 ni pagan PID: ese caso corre mucho más rápido y **no se midió aquí** (queda como hueco declarado, no como cifra estimada).

## Desenlace: **(ii) la documentación estaba mal** — el código no

- **`<0,3 s`** describe lo que `MCP-48` de verdad arregló: la **comprobación de PID** pasó de `System::new_all()` a refresco dirigido (24-34 ms). Lo que medí de la **sonda que acierta** son **142-358 ms** (3 muestras: 358 la primera con otros carriles compilando, luego 142 / 164): **compatible con** `<0,3 s`, **no** una reproducción de esa cota — dos de tres quedan por debajo de 300 ms y una no. Lo que sí queda demostrado es que citaba `live_writer_lock_blocks_promotion`, que mide **otra cosa**.
- **1,9 s** describe la **resolución completa que falla cerrado** con el PID anunciado **vivo** y el puerto cerrado (3 sondas + promoción bloqueada). También cierto, y es lo que ese test mide hoy.
- **No se optimiza** porque no hay patología que optimizar en `is_pid_alive`: deduplicar la comprobación (2× → 1× por sonda, posible porque `cleanup_stale` re-verifica el mismo PID que `try_connect` acaba de ver vivo) ahorraría **3 × ~28 ms ≈ 84 ms sobre 1.950 ms (4,3 %)** y ensancharía la API de `cleanup_stale`; el 81 % restante es el timeout de health, que es **presupuesto intencional** de una sonda ("escritor lento" ≠ "escritor muerto", invariante de `MCP-48`), no un defecto. Tocarlo es decisión de `vanta-arch`, no de este ticket.

## Spec (SDD — Phase 1b)

No aplica: el diff **no** agrega símbolos públicos, endpoints, tools MCP ni métodos de binding. Es corrección de una frase de avance + task file + fila de Backlog + un comentario de test, respaldada por la medición de arriba.

## Invariantes de dominio (handoff — MUST)

1. **`cleanup_stale` sigue siendo sólo-por-PID**: un fichero de un PID vivo (o ilegible) no se borra. No se tocó.
2. **La re-elección sigue acotada** (3 sondas + backoff + una promoción) y el test conserva sus aserciones de mecanismo; el contrato se re-corrió después de medir.
3. **Cero cambios en `vantadb-mcp/src/**` y `src/**`**: la instrumentación de medición del test es temporal; lo que entra al commit es un **comentario** (sin aserciones ni lógica) y sigue prohibido tocar `proxy.rs`.
4. **La anotación de `FIND-739` no se borra**: es la que detectó la discrepancia. Se corrige su **aritmética** (el reparto del reloj), no su hallazgo.

## Deuda técnica (Regla 6)

**Saldo neto:** negativo — se elimina una atribución de coste falsa de la documentación pública y se deja el reparto medido en su lugar, sin añadir código. La deuda que **queda** (declarada, no tocada): el timeout de health de 500 ms se paga 3 veces en una resolución que va a fallar cerrado (~1,5 s de espera en el hot path del MCP cuando el discovery anuncia un puerto muerto). Es una decisión de arquitectura (bajar `RESOLVE_ATTEMPTS` o el timeout de health), derivada al owner.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Las dos mediciones pegadas (a) y (b) + desenlace declarado + contrato verde |
| **Commit** | Commit atómico `docs(mcp): …`, diff = `bindings.md` + task file + Backlog (fila fuera) + el **comentario** del test |
| **Release** | N/A — no toca producto ni packaging |

## Steps

### Step 1: Medir `is_pid_alive` aislado (frío y caliente) y la resolución completa
- **Archivos:** `vantadb-mcp/tests/mcp_proxy_reconnect.rs` (harness `#[ignore]`, temporal)
- **Acción:** N=20 con PID vivo + N=10 con PID muerto, 1ª llamada como frío; resolución de 3 sondas con `file_lock_timeout_ms: 50` replicando `live_writer_lock_blocks_promotion`.
- **Resultado:** `is_pid_alive` 24-34 ms (no 300); resolución 1,90-2,40 s en 9 corridas (reproduce a `FIND-739`). Tablas arriba.
- **Estado:** ✅ DONE

### Step 2: Partir el reloj de la sonda (¿de quién es el 1,9 s?)
- **Acción:** instrumentar `try_proxy` y `cleanup_stale` por separado, el `Client::build`, un `GET` crudo al puerto muerto y un `TcpStream::connect` crudo; medir también la sonda que acierta contra un writer falso vivo.
- **Resultado:** 530-800 ms por sonda fallida, de los que ~500 ms son el timeout de health; `is_pid_alive` 25-30 ms; `Client::build` <2 ms; `connect` crudo a puerto cerrado ~2,04 s (no rechaza). Reparto (cuotas indicativas): ≈77 % health timeout / ≈9 % PID.
- **Estado:** ✅ DONE

### Step 3: Elegir desenlace y corregir la redacción
- **Archivos:** `docs/dev/avance/activo/bindings.md` (§`MCP-48` y anotación §`FIND-739`)
- **Acción:** la frase de `MCP-48` pasa a decir qué mide cada número (PID/comprobación vs resolución completa) con las cifras medidas y sin citar el test equivocado; la anotación de `FIND-739` se conserva y se le corrige la atribución (~300 ms × 6 → timeout de health × 3).
- **Estado:** ✅ DONE

### Step 4: Quitar la instrumentación, corregir el comentario del test y re-correr el contrato
- **Archivos:** `vantadb-mcp/tests/mcp_proxy_reconnect.rs`
- **Acción:** eliminar el harness `#[ignore]` (revert a HEAD) y corregir el comentario de la red gruesa (`:330-341`), que afirmaba *"each probe pays two `is_pid_alive`, ~300 ms per Windows targeted refresh"* — el número refutado, dentro de código que un carril futuro copiaría. El comentario pasa a: escenario (PID vivo + puerto cerrado), 1,90-2,40 s en 9 corridas, timeout de health ~77 % vs `is_pid_alive` ~9 % (24-34 ms medidos), y el ~21 s marcado como **estimación**. Sin cambio de lógica ni de aserciones.
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp -E 'test(live_writer_lock_blocks_promotion)' --build-jobs 2` → **1 passed** (7,144 s; build 14m04s en cola tras `FIND-758`) · `rustfmt --edition 2021 --check` → exit 0
- **Estado:** ✅ DONE

### Step 5: Cierre (fila fuera del Backlog + review)
- **Acción:** fila `FIND-757` fuera de `docs/dev/Backlog.md` (y su sección, que queda vacía), entrada en `bindings.md`, commit aislado con `git-commit-isolated.ps1`.
- **Estado:** ✅ DONE

## Resultado

- **Desenlace (ii):** la documentación estaba mal, no el código. `<0,3 s` y `1,9 s` miden **cosas distintas** — la comprobación de PID / una sonda que acierta, vs la resolución completa que falla cerrado con el PID vivo y el puerto cerrado — y el defecto era que `MCP-48` citaba como venue un test que mide lo segundo. De la primera familia lo medido es **compatible** con `<0,3 s` (142-358 ms), no una reproducción de esa cota.
- **La atribución de `FIND-739` también era falsa:** `is_pid_alive` no cuesta ~300 ms (cuesta 24-34 ms) y 6 comprobaciones son ≈0,17 s, no 1,8 s. El dueño del reloj es el **timeout de health de 500 ms pagado 3 veces** (~1,5 s ≈ 77 %), porque en esta máquina un puerto loopback cerrado **no rechaza** la conexión (un `connect` crudo tarda ~2,04 s).
- **No se tocó el código:** deduplicar `is_pid_alive` ahorraría ~84 ms (4,3 %, y ~9 % en el caso PID-muerto) y ensancharía una API; el ≈77 % es presupuesto intencional de la sonda.

## Dependencias
- `MCP-48` ✅ (`d38975eb9`: introdujo `reconnects()`/`resolve_failures()` y el refresco dirigido por PID) · `FIND-739` ✅ (midió la resolución y anotó la discrepancia). **Carril R, 2º de 2** (`FIND-758` ocupa el otro, en `src/`).

## Incógnitas / deuda observada y no tocada
- **El timeout de health se paga 3 veces** (~1,5 s) en la resolución que falla cerrado. Reducirlo (o `RESOLVE_ATTEMPTS`) cambia el contrato de "paciencia antes de promover" → decisión de `vanta-arch`, no de este ticket. **No se le creó fila propia a este carril:** el contrato de `FIND-757` dice `Backlog.md` **solo mi fila** (crear una fila nueva habría colisionado además con la asignación de IDs que otros tres carriles están haciendo en el mismo árbol). Queda como `queda_pendiente` para el orquestador, con el texto listo: *"la resolución que falla cerrado paga 3 × 500 ms de health timeout (~1,5 s, 77 %); en producción se suma la paciencia de lock por defecto (1000 ms, `src/config.rs:683`) ⇒ ~2,7 s por tool call en el bucle del agente — decidir si el presupuesto de sonda (timeout y/o `RESOLVE_ATTEMPTS`) baja."*
- **La cifra de producción no se midió:** todas las corridas usan `file_lock_timeout_ms: 50`. Con el default (**1000 ms**, `src/config.rs:683`) la resolución fallida suma ≈ +1 s sobre 1,90-2,40 s. Queda declarado como hueco, no como número.
- **El caso "PID realmente muerto" no se midió** (ver §Precondición del escenario): ahí `try_connect` corta antes del health y `cleanup_stale` borra el fichero en la sonda 1, así que es un orden de magnitud más rápido. No se publica cifra porque no se midió.
- **Residuos de la atribución falsa que quedan FUERA de mi scope de escritura** (declarados, no silenciados): `docs/dev/tasks/MCP-48.md:109-110` (*"la operación fallida pasó de 11,17 s a <0,3 s"*, misma venue falsa, y explícitamente sobre la operación **fallida**) y `docs/dev/tasks/FIND-739.md:115` (*"`is_pid_alive` cuesta ~300 ms … ⇒ ~1,9 s"*). Este ticket sólo puede escribir `bindings.md`, este task file y su fila del Backlog; corregir las fuentes queda para el orquestador/owner.
- **`connect` a loopback cerrado tarda ~2 s en Windows** en esta máquina (medido crudo, 2 muestras: 2044 / 2034 ms) — un RST no llega, así que cualquier sonda con timeout menor siempre lo agota. Vale para todo el ecosistema MCP/HC, no sólo este path.
- **El número de `MCP-48` para la clase de regresión (~21 s = 6 × 3,5 s)** no se re-midió: `System::new_all()` ya no está en el código y no se puede medir sin reintroducirlo. Marcado como **estimación** en `bindings.md:1684`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (contexto fresco — `ses_de71a79eeffePO02kM5C1HMSYE`; tier **fast**: docs + un comentario de test, ningún path de la tabla adversarial)
- **Enfoque:** ¿la separación (a)/(b) sostiene el veredicto o hay números derivados? ¿la frase corregida dice qué mide cada cifra? ¿se preservó la anotación de `FIND-739`? ¿(ii) es defendible frente a (i)? ¿la fila del Backlog salió de verdad? ¿el diff toca algo prohibido?
- **Cómo se probó:** re-corrió el contrato por su cuenta **2 veces** (**1 passed, 9,097 s** y **1 passed, 7,185 s**; builds de 25m33s y 33m56s en cola tras `FIND-758`) · `check-docs`/`check-links` exit 0 · leyó `proxy.rs:119-125` (corte por PID muerto), `:127` (timeout 500 ms), `:321-353` (`cleanup_stale`), `:522-540` (3 sondas) y `src/config.rs:683` (paciencia 1000 ms por defecto) para juzgar la precondición del escenario. Sus dos corridas (7,2 s / 9,1 s de duración total del test) confirman el punto de O6: **nextest no observa los 1,90-2,40 s internos** en ninguna corrida — de ahí que la cita de venue en `bindings.md` se haya reescrito a "N=9 en un harness que replica la config", y que el régimen de carga de la ventana de medición quede declarado como más tranquilo que el de hoy.
- **Veredicto ronda 1:** 🔴 `changes-required` — 0 Critical, **4 Required**:
  - **R1 — la fila del Backlog seguía en el árbol, y el cierre la declaraba fuera.** Causa raíz: un commit concurrente la **resucitó** (`9cac002ac docs(backlog): restore the FIND-757 row the FIND-767 commit dropped by accident`) porque mi borrado había viajado dentro del commit de `FIND-767` (`e6c7130cd`) — la clase `FIND-753` (copia stale del fichero compartido), no una omisión mía. **Aplicado:** fila + su sección (cabecera, tabla, fila, blanco) borradas otra vez con aserciones de contenido línea a línea, `Select-String → 0`, y el commit entra con path-scoping.
  - **R2 — la atribución falsa sobrevive en `docs/dev/tasks/MCP-48.md:109-110` y `docs/dev/tasks/FIND-739.md:115`.** **No aplicado: fuera de mi scope de escritura** (el contrato fija `bindings.md`, este task file y mi fila del Backlog). Declarado como residuo con `file:line` en §Incógnitas; corregir las fuentes es del orquestador.
  - **R3 — la frase publicada no nombraba la precondición** (PID anunciado **vivo** + puerto cerrado) y así se leía como "el mecanismo cuesta 1,9 s siempre". **Aplicado:** precondición explícita en `bindings.md:1684` y `:1746`, con el corte por PID muerto citado por línea. **No** se publica el número del caso PID-muerto: no se midió, y estimarlo sería repetir el pecado del ticket.
  - **R4 — el gate estaba vacío con `✅ COMPLETED`.** Aplicado: esta sección.
- **Optional aplicados (4 de 7):** **O1** el `~300 ms` seguía en un comentario del test → corregido, con escenario y cuotas; **O2** `bindings.md:1739` ahora tiene pointer a la corrección; **O3** cuotas recalculadas a ≈77 % / ≈9 % y marcadas como indicativas (no suman exacto); **O4** "era cierto" suavizado a "compatible con, no una reproducción de la cota"; **O7** el `~21 s` marcado como **estimación** en ambas entradas.
- **Optional no aplicados (con motivo):** **O5** (cifra de producción con la paciencia de 1000 ms) → declarada como hueco, no medida; **O6** (procedencia) → cubierto: el task file dice que las 9 corridas salen de un harness `#[ignore]` que replica la config del test, y por qué el `junit.xml` no puede confirmarlas.
- **Pendiente del orquestador (fuera de "solo mi fila"):** abrir la fila del lever de arquitectura (timeout de health ×3 / `RESOLVE_ATTEMPTS`, ~2,7 s por tool call en producción) con el texto listo en §Incógnitas.
- **Sin segunda ronda formal** (declarado, no fabricado).
