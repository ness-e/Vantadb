---
title: "FIND-159: deuda API-04 - el fallo envuelto deja de ser anonimo, y la cifra que era de otro perfil"
kind: task
status: in-progress
description: "internal_error e invalid_request delegaban en plain(), que fija data:Null, y to_json omite data cuando es null: un fallo envuelto llegaba al cliente LLM sin nada mas que -32603/-32600. Ahora ambos llevan data.code canonico + retriable. El '38 tools' era el perfil dev, no las extendidas (41 definidas / 96 totales). Schemars sigue diferido."
id: FIND-159
---

# FIND-159 - deuda API-04: un fallo envuelto tiene que seguir siendo ramificable

> **Carril:** `vantadb-mcp/src/error.rs` + tests + `docs/api/**`. Escritura:
> `vantadb-mcp/src/error.rs` · `vantadb-mcp/tests/mcp_tests.rs` · `docs/api/ERROR_HANDLING.md` ·
> `docs/api/MCP.md` · este task file · `docs/dev/Backlog.md` (solo la fila `FIND-159`) ·
> `docs/dev/avance/activo/bindings.md`.
> **Prohibido y respetado:** `src/text_index.rs`, `src/storage/engine/get.rs` (carril 513/812) ·
> `vantadb-mcp/src/memory_scheduler.rs` y `vantadb-{python,node,wasm}/src/**`
> (carril WIRE-23/EGO-15) · `src/sdk/api/**` · `vanta-memory/**` · `.github/**` ·
> `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md`.

## Premisa re-medida en HEAD (2026-11-27)

| Item | Estado medido | Evidencia |
| --- | --- | --- |
| (a) `schemars` como fuente unica del schema de las tools | **VIVA como DEFER** | 0 hits de `schemars` en `vantadb-mcp/` y en el workspace: no es un dato sin verificar, es una dependencia que no existe |
| (b) "38 tools extendidas" | **CAMBIADA — la cifra era de otro perfil** | `38` es el perfil `dev` (`docs/api/MCP.md:288`, pinnado por `vantadb-mcp/tests/mcp_tests.rs:5476`). Las extendidas son **41 definidas / 35 listadas** (`MCP.md:583`), sobre **96 definidas / 90 listadas** (`MCP.md:234,971`) |
| (c) `internal_error` sin `data.code` | **VIVA, exacta** | `error.rs:82-84` delega en `plain` (`:21-27`) → `data: Null` → `to_json` (`:123-129`) **omite** `data`. Gemelo que la fila omitia: `invalid_request` (`:86-89`) |

Contraste que hace la fila demostrable: `not_found` (`:60-67`) y `resource_limit` (`:72-79`)
llevan `code` + `retriable`; `from_domain` (`:108-114`) siempre emite `data.code`.

**Y una promesa del doc que hoy es falsa:** `docs/api/MCP.md:116` dice que un escritor con
resultado desconocido devuelve "-32603 con `data.retriable: false`" — hoy ese `data` no existe.

## Decisiones (una por item, con su razon)

### (c) `data.code` en `internal_error` e `invalid_request` — **implementado**

| Factory | JSON-RPC | `data.code` | `retriable` |
| --- | --- | --- | --- |
| `internal_error` | `-32603` | `VANTADB_INTERNAL_ERROR` | `false` |
| `invalid_request` | `-32600` | `VANTADB_INVALID_ARGUMENT` | `false` |

- **`VANTADB_INTERNAL_ERROR` es un codigo nuevo (el 11.º canónico) y se declara como tal** en
  `ERROR_HANDLING.md` §1.1: no lo produce `Error::code()`, es el nombre que el transporte MCP
  le da a `-32603` cuando el fallo no viene de un `Error` del core (panic, semaforo cerrado,
  cancelacion, serializacion). Sin el, la unica salida era `VANTADB_WASM_ERROR` (el catch-all
  del core para `Generic`), que es un nombre falso para un servidor MCP.
- **`VANTADB_INVALID_ARGUMENT`** ya existe en §1.1 como "el llamador paso un argumento
  malformado" — un JSON-RPC que no es un Request valido es exactamente eso. No se inventa nada.
- **`retriable: false`** en ambos: replica la clasificacion del core (§1.1, ❌ para los dos
  codigos) y es el valor que `MCP.md:116` ya prometia. Un `-32603` que en realidad era un
  timeout sigue siendo un defecto de clasificacion de su call site, no de esta factory (ver
  observaciones).
- **No se tocan las otras tres factories** (`parse_error`, `invalid_params`,
  `method_not_found`). Tienen un test de compatibilidad explicito —
  `err_mcp_01_std_factories_keep_null_data` (`mcp_tests.rs:6116-6124`) — que declara que las
  rechazos de protocolo no llevan `data` a menos que se fije. Se respeta ese contrato: los dos
  que cambian son los dos donde el payload es lo unico que distingue la causa.

### (b) la cifra — **se fija el universo donde se lee**

Los `38` de `MCP.md:276`, `vantadb-mcp/src/config.rs:13,34` y `mcp_tests.rs:5476` son
**correctos**: son el perfil `dev`. Lo que estaba mal era atribuir ese numero a las tools
extendidas en las filas de plan (prohibidas en este carril). Lo que se corrige aca:
`MCP.md` gana la nota que impide la relectura ("`38` = perfil `dev`; las extendidas son 41
definidas / 35 listadas; el total definido es 96"), y la tabla de perfiles deja de ser la
unica fuente de ese numero.

### (a) schemars single-source — **DEFER declarado, con su razon**

Requiere wire structs + parseo tipado sobre 41 herramientas de 7 familias; es multi-semana y
no compra nada que (c) no haya comprado ya (un cliente puede ramificar por `data.code` sin que
el schema sea generado). Queda escrito en `MCP.md` §Coverage, donde se lee el estado real, para
que no vuelva a entrar como "pendiente sin razon".

## Impacto mapeado (Regla 0)

- **Archivos leidos completos:** `vantadb-mcp/src/error.rs` (146 L),
  `vantadb-mcp/tests/mcp_tests.rs:6016-6145`, `docs/api/ERROR_HANDLING.md:25-144` y `:360-449`,
  `docs/api/MCP.md:105-215` y `:262-292` y `:560-570` y `:965-975`,
  `vantadb-mcp/src/validation.rs:520-527` (`error_content_mcp`), `.opencode/rules/server-mcp.md`.
- **Referencias hacia dentro:** `internal_error` tiene ~25 call sites
  (`vantadb-mcp/src/{code,skills}.rs`, `handlers/tools.rs`, `server.rs`); `invalid_request`
  tiene 2 (`server.rs:576,932`). Todos reciben el `data` nuevo sin cambiar su firma.
  `error_content_mcp` serializa `to_json()` → el contenido `isError` de esas tools gana el
  campo; ningun test compara el texto exacto de un `internal_error` (verificado: 0 hits de
  `-32603` en `vantadb-mcp/tests/` salvo el mapeo de `from_domain`).
- **Referencias hacia fuera:** ningun cliente publicado depende de que `data` falte; el campo
  es aditivo y lo documenta `docs/api/MCP.md` §Envelope.
- **Veredicto:** riesgo bajo y contenido — aditivo en el wire, sin cambio de firmas, sin
  `unsafe`, sin concurrencia. Los paths tocados (`vantadb-mcp/src/**`, `docs/api/**`) caen en
  el tier adversarial de review P2-01.

## Steps

| # | Step | Estado |
| --- | --- | --- |
| 1 | Re-medir los tres items: `schemars` (0 hits), la cifra (38 = perfil `dev`), el envelope (`plain` → `Null` → omitido) | ✅ |
| 2 | RED: test que exige `data.code` en `internal_error` e `invalid_request` | ✅ |
| 3 | GREEN: las dos factories emiten `code` + `retriable`; `plain` sigue siendo el camino de las tres de protocolo | ✅ |
| 4 | Docs: §1.1 con el 11.º codigo, §6.1/§6.3/§6.4 de `ERROR_HANDLING.md`, intro + tablas de `MCP.md` | ✅ |
| 5 | (b) nota del universo real de tools + (a) DEFER declarado en `MCP.md` | ✅ |
| 6 | Verificacion mecanica + commit, Backlog y avance | ✅ |

## Verificacion

| Comando | Resultado |
| --- | --- |
| `cargo nextest list -p vantadb-mcp --test mcp_tests --ignore-default-filter -E 'test(err_mcp_01)'` | **población 10** (el filtro desnudo sobre el perfil `default` daba **0**: `.config/nextest.toml:64` excluye `binary(mcp_tests)`; hace falta `--ignore-default-filter`, como la fase `05-mcp` de `certification.yml:576`) |
| `cargo nextest run … -E 'test(err_mcp_01)' --no-tests fail --build-jobs 2` | **10/10 passed** (incluye el test nuevo y el de compat endurecido) |
| **RED del test nuevo** (probado revirtiendo las dos factories a `plain()` y restaurando por hash: `0DF22F19…` antes y despues) | `left: Null` / `right: "VANTADB_INTERNAL_ERROR"` en `mcp_tests.rs:6146` — el `data` ausente que describe la fila |
| `pwsh scripts/validate-docs-coverage.ps1` | **0 gaps** (sigue midiendo las 55 core tools) |
| `cargo fmt --check` | sin diff en mis archivos (los diffs son de `impl_rebuild.rs`, `storage/engine/get.rs`, `text_index.rs`, carriles ajenos) |

## Review P2-01

Tier **adversarial** (`docs/api/**` + `vantadb-mcp/src/**`). Ronda 1 por `vanta-review` en
contexto fresco sobre los dos cambiosets: **`changes-required`**, con 1 **Blocking** y 5 Required
— **ninguno de ellos en este changeset** (el Blocking era el `next_cursor` de `FIND-158`; ver
allí). Lo que el review confirmó de esta fila, re-midiendo:

- `38` es el perfil `dev` y **ningún** sitio de `docs/api/**` lo atribuye a las extendidas tras
  la nota nueva (`MCP.md:288` es la fila `dev`).
- Los números 96 / 90 / 55 / 41 / 35 cuadran con el código: `55 + 41 = 96` y `49 + 6 + 35 = 90`,
  con `dev_tools.len() == 38` pinnado en `mcp_tests.rs:5476`.
- El gate documental **no se debilitó**: `validate-docs-coverage.ps1` sigue en 0 gaps y sigue
  midiendo las **55** core tools.
- `VANTADB_INTERNAL_ERROR` como 11.º código se sostuvo: es el único nombre verdadero
  (`WASM_ERROR` en un servidor MCP sería falso; `IO_ERROR` es otra familia) y §1.1 lo marca como
  "`code()` never returns it".
- Lo que faltaba **no era el nombre, era la prosa**: R2/R3/R4 eran frases absolutas del doc que
  este cambio volvió falsas o que ya lo eran (`invalid_params` vs `validation()`, "Every …
  carries `data.code`" contra el path proxy, `retriable: false` contra §6.4). Se corrigieron en
  `ERROR_HANDLING.md` y `MCP.md` (ver los R-notes en `FIND-158.md`, mismo review).
- **O3 aplicado:** el test de compat pasa de `j["data"].is_null()` a `get("data").is_none()`,
  que es lo que el nombre del test promete.

## Observaciones fuera de carril (NOTICED BUT NOT TOUCHING)

- `vantadb-mcp/src/server.rs:719,743` mapean un **timeout** del runtime a
  `McpError::internal_error("Request timed out")`, que ahora saldra con
  `retriable: false` cuando semanticamente es `VANTADB_TIMEOUT` (✅ retriable). No se toca:
  pediria una factory `timeout()` nueva y cambiar el call site; `server.rs` esta fuera del
  alcance declarado de esta fila (el carril solo toca `src/error.rs` de `vantadb-mcp`).
- `vantadb-mcp/src/proxy.rs:306,335` construyen `-32603` con `json!` a mano y
  siguen sin `data`; `:319,795` emiten `data` con `retriable` pero **sin**
  `data.code`. Mismo motivo: fuera del alcance declarado (el carril sólo toca
  `src/error.rs` de `vantadb-mcp`), y se nombran en `docs/api/MCP.md` como la
  excepción conocida del envelope.
