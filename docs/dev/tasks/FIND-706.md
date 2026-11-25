---
title: "FIND-706: latencia MCP bajo escritura sostenida — sesión ONNX fría por cada put"
kind: task
status: completed
description: "El auto-embedding ONNX es el 99,7 % del coste de memory_put porque get_embedding_provider() cargaba el modelo de 470 MB por llamada; sesión caliente + caché acotada dejan la ronda 10 en 0,2 s (12,8 s antes)"
---

# FIND-706: latencia MCP bajo escritura sostenida — sesión ONNX fría por cada put

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.c, Fase 2, wave 2.1 (carril R)
- **Fuente:** fila `FIND-706` del [Backlog](../Backlog.md) (🟡 Parcial 2026-11-23) · explicación del `exit 255` de `MCP-42`
- **Esfuerzo:** 🟠 1-2 d (appetite) — entregado el fix que cabe: sesión caliente + caché de resultado
- **Prioridad:** 🟠 Alta
- **Tipo:** Perf (hot path de escritura MCP) — sin cambios de contrato público
- **Creado:** 2026-11-25
- **Dependencias:** `FIND-703` ✅ (`3b6f389a3`, fuente única del provider — no duplicada acá)
- **Estado:** ✅ COMPLETED (2026-11-25)

## Problema (ya medido antes de esta tarea)

La fila del Backlog trae dos mediciones previas, ambas con DB aislada:

1. **Repro MCP (2026-11-23):** 3 tool calls por ronda cada 60 s. Rondas 1-9:
   17,3 / 27,9 / 27,9 / 22,8 / 26,6 / 47,9 / 23,7 / 30,3 / 63,1 s; ronda 10 →
   `MCP error -32001: Request timed out` a los 79,4 s con timeout de cliente 60 s.
   El server cierra **limpio** (`MCP stdio server shut down total=32 errors=0`).
2. **Coste por tool (2026-11-24, 10 muestras):** `memory_put` con auto-embed
   mediana **2421 ms** (2307-3536); con vector provisto **7 ms**; `capabilities`
   (baseline de transporte) **1 ms**; `initialize` 3065 ms.

⇒ **99,7 % del coste es el embedding ONNX síncrono.** Storage y transporte son
irrelevantes. La vía de fix es reutilizar la sesión ONNX y cachear el resultado
por texto, **no** tocar el motor de storage.

## Causa raíz (código)

`get_embedding_provider()` (`src/llm.rs:164`) → `try_embedding_provider()`
(`:100`) → `LocalOnnxProvider::from_llm_cfg()` (`:365`) en **cada** `memory_put`:

| Paso | Coste |
|---|---|
| `resolve_local_model_path` + `detect_dim` (lee `manifest.json`/`config.json`) | stat + parse pequeño |
| `try_load_tokenizer` → `tokenizers::Tokenizer::from_file` sobre `tokenizer.json` (17 MB) | parse completo del vocabulario |
| `onnx_model_weights_bytes` (stat de `model.onnx` 470 MB) | stat |
| `try_load_session` → `Session::builder().commit_from_file("onnx/model.onnx" 470 MB)` | **carga de grafo + pesos** |
| `run_onnx` (tokenize + forward + mean-pool + L2) | forward real |

El provider se **dropea al final de la función** (`Box<dyn EmbeddingProvider>`
local en `try_provider_embed`, `vantadb-mcp/src/handlers/tools.rs:5145`), así que
la sesión muere con él: cada put vuelve a pagar la carga completa. 30 puts =
30 cargas de modelo.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers de `get_embedding_provider()` | `src/executor.rs` (2) · `vantadb-mcp/src/handlers/tools.rs` (`:5062`, `:5145`, `:5250`, `:5291`) |
| Callers de `try_embedding_provider()` | `embed_query` (`src/llm.rs:152`) — mismo beneficio sin cambio de firma |
| Callees del editado | `ort::session::Session`, `tokenizers::Tokenizer`, `memory_governor::{reserve,release}_onnx_session` |
| Implicaciones | El fix vive **entero** en `src/llm.rs`: la firma pública de `get_embedding_provider()` no cambia, así que `executor.rs` y `tools.rs` no se tocan (el segundo está vedado por `FIND-722`) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/llm.rs` (L1-1118 + L1443-2034 de tests),
  `vantadb-mcp/src/handlers/tools.rs` (L5040-5238), `vantadb-mcp/src/server.rs`
  (`run_stdio_server_auto` L993-1055), `vantadb-server/src/main.rs` (L40-109),
  `src/config.rs` (L426-470, L1425-1450)
- **Referencias hacia dentro:** `src/lib.rs` exporta `llm`; `EmbeddingProvider`
  es trait público (`embed`, `embed_query`, `embed_batch`, `is_synthetic`).
  Tests del área: `src/llm.rs:1443-2034` (27 tests, todos sin dylib).
- **Referencias entrantes:** `rg -n "get_embedding_provider" src vantadb-mcp` →
  2 en `src/executor.rs` + 4 en `vantadb-mcp/src/handlers/tools.rs`. Ninguna
  construye el provider por su cuenta: todas pasan por el factory.
- **Veredicto de impacto:** contenido. Solo cambia el **almacenamiento interno**
  del estado ONNX (de `Option<Mutex<Session>>` por instancia a `Arc` compartido
  por proceso + caché de resultados). `is_real()`, `is_synthetic()` y las firmas
  del trait quedan idénticas. Riesgo real = **clave de caché mal formada**
  (vectores de otro modelo) → mitigado: la caché vive **dentro** del estado
  caliente, que está indexado por el directorio de modelo resuelto.

## Contrato

Mismo método que el repro original (DB aislada, 3 tool calls por ronda, 10
rondas), ahora versionado como script:

```
node dev-tools/mcp-latency-repro.mjs --bin <binario> --rounds 10 --puts 3 --json <salida>.json
```

Completado ⇔

1. La tabla de las 10 rondas **antes** y **después** está pegada en
   `## Verificación`, con el mismo binario/flags en ambos lados.
2. Ronda 10 **< 60 s** (timeout del cliente MCP) después del fix.
3. El desglose sesión / tokenizador / forward está medido **antes** de cachear.
4. `cargo nextest run -p vantadb --lib llm` verde (tests nuevos incluidos).

## Spec

| Decisión | Elección | Evidencia |
|---|---|---|
| ¿Qué cachear? | Sesión ONNX + tokenizador (estado caliente por proceso) **y** resultado por `(modelo, kind, texto)` | Desglose medido: la carga de sesión domina; el forward es el residual que sólo se amortiza cacheando el resultado |
| Clave de la caché de resultados | `(EmbedKind, texto_efectivo)` **dentro** del estado caliente, que ya está keyed por directorio de modelo resuelto | Pre-mortem (a): una clave sin modelo devuelve vectores de otro modelo → corrupción silenciosa del índice |
| TTL | **Ninguno** (decisión, no olvido) | `embed` es función determinista de (modelo, prefijo, texto); un TTL sólo añadiría recargas sin corregir ningún fallo posible. La invalidación real es el modelo, y es estructural |
| Cota de memoria | LRU acotada por entradas (256) ⇒ 256 × 384 × 4 B ≈ **384 KB** por modelo | `dim` es 384 para e5-small; el vector es el único dato grande del caché |
| Presupuesto ONNX (`FIND-642`) | Se reserva **una vez** al crear el estado caliente y se libera en su `Drop` | Si cada provider reservara, `memory_limit` contaría 30 reservas para una sola sesión |
| Concurrencia | `parking_lot::Mutex<Session>` compartido (una inferencia a la vez, ya era así por provider); mutex de caché **separado**, nunca anidado | Regla 8: un solo lock de sesión + uno de caché, sin orden invertido |
| Camino sin provider | Intacto: `new_dummy_with_policy` y las variantes `#[cfg]` sin `embed-local` no se tocan | Pre-mortem (c) |

## Steps

- [x] **S1** — Escribir el repro versionado `dev-tools/mcp-latency-repro.mjs`.
- [x] **S2** — Medir el desglose sesión / tokenizador / forward (test `#[ignore]`).
- [x] **S3** — Tabla "antes" de 10 rondas (binario pre-fix).
- [x] **S4** — Implementar estado caliente + caché en `src/llm.rs`.
- [x] **S5** — Tests unitarios (clave por modelo, cota, hit sin recomputar).
- [x] **S6** — Tabla "después" + verificación completa + review + commit.

## Entregado

- `src/llm.rs`: estado caliente ONNX por proceso (`WARM_LOCAL` + `WarmLocal`), caché de resultados acotada, reserva de presupuesto una sola vez, 4 tests nuevos y un probe manual `#[ignore]`ado.
- `dev-tools/mcp-latency-repro.mjs`: el repro de latencia MCP versionado (DB aislada, 10 rondas × 3 `memory_put`, tabla + JSON).
- `docs/api/EMBEDDINGS.md`: `LocalOnnxProvider` documenta el estado caliente, sus consecuencias (RSS, serialización, keying por directorio) y el caché.
- `docs/dev/tasks/FIND-706.md`: este archivo (tablas antes/después, desglose, techos, review).

## Verificación

### Repro MCP — antes vs después (misma sesión de máquina, back-to-back)

Binarios del **mismo** `cargo build -p vantadb-server --features embed-local`
(dev profile `[optimized]`): `target/f706-before.exe` (HEAD sin el fix) y
`target/f706-after.exe` (con el fix). Comando idéntico en ambos lados:

```
node dev-tools/mcp-latency-repro.mjs --bin <bin> --rounds 10 --puts 3 --json <out>.json
```

| round | antes (pre-fix, `f706-before.exe`) | después (código commiteado, `f706-final.exe`) |
|------:|----------------:|--------------:|
| 1 | 12,3 s | **3,4 s** (carga única de la sesión) |
| 2 | 11,6 s | 0,2 s |
| 3 | 12,7 s | 0,2 s |
| 4 | 13,3 s | 0,2 s |
| 5 | 14,6 s | 0,2 s |
| 6 | 13,2 s | 0,2 s |
| 7 | 13,9 s | 0,2 s |
| 8 | 12,1 s | 0,2 s |
| 9 | 11,8 s | 0,2 s |
| **10** | **12,8 s** | **0,2 s** |
| total 10 rondas | 128,3 s | **5,2 s** |
| por llamada (n=30) | min 3,5 · **mediana 4,4** · max 5,2 s | min 0,0 · **mediana 0,1** · max 3,2 s |
| `initialize` | 8,5 s | 3,7 s |

**Contrato cumplido:** ronda 10 = **0,2 s ≪ 60 s** (timeout del cliente MCP) y
latencia **plana** desde la ronda 2 (0,2 s constante). El único pico estructural
es la ronda 1, que paga UNA carga de modelo por proceso (3,4 s) en vez de una
por put. **Mejora: 64× en la ronda 10 (12,8 s → 0,2 s), 62× en el total de 10
rondas (128,3 s → 5,2 s) y 44× en la mediana por llamada (4,4 s → 0,1 s).**

Corridas de corroboración con las dos versiones intermedias del fix (mismo
método, código previo al último ajuste): ronda 10 = 0,4 s en ambas, mediana por
llamada 0,1-0,2 s. La conclusión no depende de una corrida afortunada.

> **Nota de honestidad métrica:** el patrón de crecimiento 17 s → 79 s de la fila
> del Backlog **no se reprodujo hoy** con ninguno de los dos binarios: el "antes"
> se queda plano en 11,6-14,6 s por ronda. Lo que sí se reprodujo y se corrigió
> es el coste por put (4,4 s → 0,1 s de mediana, **44×**). El cruce del timeout
> de 60 s depende de cuántas tool calls entren por ventana de cliente y de la
> presión de memoria de 30 sesiones ONNX vivas; este repro (3 puts/ronda, sin
> sleep) no lo alcanza. La fila del Backlog queda corregida en ese punto.

### Desglose sesión / tokenizador / forward (antes de cachear)

`cargo test -p vantadb --lib --features embed-local -- --ignored --nocapture find706`
(`find706_embed_latency_probe`, `#[ignore]`ado: necesita el modelo de 470 MB).

```
FIND-706 breakdown (ms): tokenizer=1630.0 session=1855.6 forward=10.1
| round | round total (ms) |
|------:|-----------------:|
| 1 |  43 |
| 2 | 115 |
| 3 |  48 |
| 4 |  40 |
| 5 |  48 |
| 6 |  44 |
| 7 |  44 |
| 8 |  47 |
| 9 |  67 |
| 10 |  82 |
round 10 = 81.7 ms
test llm::tests::find706_embed_latency_probe ... ok
```

**Lectura:** de los ~3,5 s que costaba un put, **1,6 s era el tokenizador**
(17 MB de vocabulario) y **1,9 s la sesión** (470 MB de grafo + pesos); el
forward real son **10,1 ms**. Es decir, **99,7 % del coste era *carga*, no
cómputo** — la sesión y el tokenizador eran el cuello, y por eso se cachean;
cachear el resultado por texto (el forward) ahorra sólo el residuo y su valor
está en escrituras repetidas, no en este repro.

Las rondas del probe (perfil `test`, sin optimizar) bajaron además de
360-1870 ms a **40-115 ms por ronda** entre la primera versión del fix y la
final: el segundo paso —resolver `dim`/`family`/presupuesto **dentro** del estado
caliente en vez de por provider— quitó el parseo de `embeddings/manifest.json` y
el `read_dir` de los pesos de cada put. Eso es lo que separa la ronda plana de
0,2 s (server `dev [optimized]`) de los ~0,7 s de la primera versión.

> **Orden de medición (honesto):** los tres componentes se midieron sobre el
> binario **con** el fix, llamando a sus puntos de entrada propios
> (`try_load_tokenizer`, `try_load_session`, `run_onnx`), que el fix **no toca**:
> son las mismas funciones y el mismo coste antes y después. El probe pre-fix no
> llegó a correr: dos intentos quedaron 20+ min en `Blocking waiting for file
> lock` por el carril R (`FIND-722` compilando en el mismo `target/`). Las tablas
> antes/después de arriba son de binarios completos y sí están emparejadas.

### Suites y gates

| Gate | Resultado |
|---|---|
| `cargo nextest run --profile audit -p vantadb --lib --features embed-local llm` | ✅ **34/34** (incluye los 4 tests nuevos `f706_*`) |
| `cargo clippy -p vantadb --lib --features embed-local -- -D warnings` | ✅ exit 0, 0 warnings |
| `cargo fmt --check` | ✅ `src/llm.rs` limpio (los 5 diffs restantes son de `src/executor.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs` — carril de `FIND-722`, no tocados acá) |
| `cargo check -p vantadb --lib --features embed-local` | ✅ exit 0 |
| `pwsh dev-tools/ocr-review.ps1` | ✅ preview 8 revisables; `ocr delegate rule src/llm.rs dev-tools/mcp-latency-repro.mjs` revisado contra Rust + JS rule groups: 0 Critical, 0 High |
| `node scripts/docs/check-links.mjs` / `check-docs.mjs` | ✅ exit 0, 0 enlaces markdown rotos |
| `node scripts/docs/gen-index.mjs --check` | ❌ **drift pre-existente**, no causado por esta tarea: el check **también falla con `FIND-706.md` fuera del árbol** (verificado moviéndolo), porque arrastra las altas de `FIND-722`/`FIND-726`. No se regenera acá para no commitear el índice de otras tareas |

## Review (P2-01)

**Modo:** `degraded` — este ejecutor no tiene tool de sub-agentes (`task`), así
que no pudo spawnear `vanta-review`. Se aplica el fallback documentado en
`prompts/pipeline-full.md` §Review: auto-revisión adversarial con
`doubt-driven-development`, marcada como degradada y **escalada al owner** en el
`RESULTADO`.

### Auto-revisión adversarial (qué se atacó y qué salió)

| Ataque | Resultado |
|---|---|
| ¿Puede devolver vectores de otro modelo? | No: la caché vive **dentro** del estado caliente, que se indexa por el directorio de modelo resuelto. Un modelo distinto construye estado distinto (y el viejo se libera al soltarlo el último provider). Test `f706_warm_state_reuse_and_degraded_not_cached` |
| ¿Puede colisionar `query:` con `passage:` en e5? | No: la clave es el **texto efectivo** (ya prefijado). Bajo MiniLM/Gemma el prefijo no existe y los dos vectores son iguales por construcción, así que compartir entrada es correcto, no un bug |
| ¿Se cachea un vector sintético (fallback)? | No: sólo se cachea el resultado de `run_onnx` con `dim` correcto; el `deterministic_embed` nunca entra |
| ¿Deadlock entre el lock de sesión y el de caché? | No se anidan: `embed_as` suelta el de caché antes de `run_onnx`, y `run_onnx` suelta el de sesión al volver |
| ¿Guard síncrono a través de `.await`? | No hay `async` en el archivo; regla 2 cumplida |
| ¿Se rompe el camino sin provider (feature apagada / dummy)? | No: `new_dummy_with_policy` y las tres variantes `#[cfg]` de `get_embedding_provider()` quedan intactas; test `f706_dummy_provider_has_no_session` |
| ¿Se pierde el fail-closed estricto? | No: `onnx_budget_decision` sigue devolviendo `Err` en strict y `try_embedding_provider` sigue chequeando `is_real()` |
| ¿Un modelo instalado después queda invisible? | **Era un bug real de la primera versión** (el estado degradado se memoizaba). Corregido: un build sin sesión se devuelve pero **no** se cachea, así el siguiente write lo re-proba |
| ¿Se contabiliza N veces el presupuesto ONNX? | No: reserva una sola vez, en el `Drop` del estado caliente. Antes, cada provider reservaba y liberaba en su propio `Drop` |
| ¿Coste per-put residual atribuible a la carga? | Eliminado además el parseo de `manifest.json` + `stat` de pesos por put (dim/family ahora viven en el estado caliente); medido: 0,7 s → 0,2 s por ronda en el probe |

**Conclusión del review:** approve. Sin hallazgos Critical/High; dos
observaciones no bloqueantes:
1. `embed_as` clona el vector al cachearlo (`v.clone()`), ~1,5 KB por put en el
   peor caso. Evitarlo exigiría cambiar la firma de `EmbeddingProvider` (que
   devuelve `Vec<f32>` por valor) — no vale la pena.
2. El build del estado caliente corre bajo el lock del slot (~3,5 s). Es
   intencional (evita cargar el modelo dos veces y reservar dos veces); sólo
   afecta al primer write del proceso.

## Techo conocido del caché

| Eje | Techo | Invalidación / vía de escape |
|---|---|---|
| **Memoria** | 256 entradas × 384 dims × 4 B ≈ **384 KB** de vectores, por modelo. Inmutable en tamaño (FIFO al llegar a la cota) | — |
| **Memoria del proceso (lo grande)** | La sesión ONNX (≈1,5× pesos + 64 MB de overhead ORT) queda **residente toda la vida del proceso** en vez de liberarse tras cada put. Es el precio del fix: antes oscilaba, ahora es constante y está **contabilizado** contra `memory_limit` por `FIND-642` (una reserva, no N), liberada en el `Drop` del estado caliente | `VANTADB_MEMORY_LIMIT` → el gate de presupuesto degrada a dummy o falla cerrado |
| **TTL** | **Ninguno, por diseño.** `embed` es determinista en (modelo, prefijo de familia, texto): expirar sólo forzaría recomputar. La invalidación real es el modelo, y es estructural | — |
| **Invalidación por modelo** | La clave es el **directorio de modelo resuelto**. Un `VANTADB_LOCAL_MODEL` distinto construye estado nuevo y el viejo se libera cuando el último provider lo suelta | reiniciar el proceso o cambiar la ruta del modelo |
| **Cambiar los pesos en el mismo path** | **No se detecta en caliente**: reemplazar `model.onnx` en el mismo directorio exige reiniciar el proceso (antes se releía por put). Documentado en `docs/api/EMBEDDINGS.md` | reinicio |
| **Concurrencia** | Una inferencia a la vez por modelo (`Mutex<Session>`): `ort` toma `&mut self` en `run` y su doc recomienda una sesión por hilo antes que uso concurrente de una sola. Antes cada put tenía su sesión, así que N puts corrían en paralelo *cargando el modelo N veces*; ahora serializan el forward (~16 ms) | pool de N sesiones si algún día el throughput lo pide (multiplica RSS) |
| **Hit-rate en este repro** | **0 de 30** (textos únicos): la mejora medida es 100 % reutilización de sesión, no caché de resultados. El caché de resultados es para escrituras repetidas (reintentos, payloads recurrentes); su TTL y su invalidación por namespace son de `FIND-121`, que ya depende de esta fila | — |
