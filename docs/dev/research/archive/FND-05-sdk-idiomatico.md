# FND-05 — SDK idiomático (no wrapper 1:1 de Rust)

> **Fecha:** 2026-08-16 · **Plan:** 2026-08-16-wave-p20-tsys.md · **Prio:** 🟡
> **Tipo:** análisis + prototipo (NO rewrite) · **Ámbito:** `vantadb-python/`, `vantadb-ts/`

## Resumen ejecutivo

Ambos SDK ya NO son un reflejo crudo del core Rust — hay trabajo idiomático real hecho
(objetos tipados en Python, `VantaError` con codes en TS, wrapper async en Python,
backend async nativo en TS). Pero quedan gaps concretos, el más serio en **Python**
(stub drift que hace que `connect`/`__version__`/`get_memory`/`list_memory`/`put`
mientan a los type checkers) y **fricción de metadata tagged-union en TS**
(`{ String: "en" }` en vez de `"en"`).

Veredicto: no hay rewrite a la vista. Hay (a) fixes de stubs, (b) añadir context
manager a `VantaDB`, (c) añadir `Symbol.asyncDispose` a los backends TS,
(d) normalizar `VantaMetadata` — en ese orden de prioridad.

---

## 1. Estado actual por SDK

### 1.1 Python (`vantadb-python`)

| Superficie | Estado | Idiómatico |
|---|---|---|
| `VantaDB` (sync, PyO3) | `close()` existe; **sin** `__enter__/__exit__` | ❌ falta CM |
| `AsyncVantaDB` (wrapper py) | `async with` completo (`__aenter__`/`__aexit__`), `asyncio.to_thread` + semáforo | ✅ |
| `SearchRequest` (dataclass) | `namespace`, `query_vector`, filtros, top_k... `asdict()` | ✅ |
| `VantaMemoryRecord`/`VantaSearchHit`/`VantaListResult` | pyclass con getters tipados + `__getitem__`/`__iter__` | ✅ |
| Stubs `.pyi` | `vantadb_py.pyi` (183L) + `__init__.pyi` (294L) | ⚠️ con drift (ver gaps) |
| Type hints | `list[float]`, `int | None`, `dict | None`... | ⚠️ sin TypedDict/overloads |
| Vectores | `Any` (acepta `list[float]` o numpy) | ⚠️ aceptable, pierde typing |
| `put_batch` | dual API 9 parámetros | ❌ (deuda P2-5 ya conocida) |

**Buenas noticias:** `AsyncVantaDB` es exactamente el patrón idiomático que el
backlog sospechaba que faltaba: `async with AsyncVantaDB(path) as db:` cierra solo
(vía `to_thread` para liberar GIL durante el close Rust). Eso ya está hecho y tipado
en `__init__.pyi:153-262`.

### 1.2 TypeScript (`vantadb-ts`)

| Superficie | Estado | Idiómatico |
|---|---|---|
| `VantaDB` (WASM, sync) | factories `connect/create/open`, `close()` idempotente, `_assertOpen` | ✅ para main-thread |
| `NativeVantaDB` (napi-rs, async) | `static async connect`, métodos `Promise<T>`, `async close()` | ✅ |
| `VantaError` | class con `code`, `details`, `toJSON()` + `wrapWasmError`/`wrapNativeError` | ✅ |
| Type guards | `isMemoryRecord`, `isSearchHit`, `isVantaMetadata`, `isValidVector`, `validateVector` | ✅ |
| `VantaMetadata` | `Record<string, VantaValue>` con tagged-union `{String: string}...` | ❌ fricción |
| Tipos numéricos | `created_at_ms: string \| number`, `version`, `node_id` | ⚠️ unions amplias |
| Resource management | `close()` manual; sin `Symbol.dispose`/`Symbol.asyncDispose` | ❌ falta `await using` |
| `QueryResult` | `Read?` / `Write?` opcionales | ⚠️ no es unión discriminada |

**Buenas noticias:** el SDK TS ya es moderno — ESM, `strict`, type guards exportadas,
`VantaError` con códigos (mejor que chromadb y libsql), backend nativo async real.
El README (pulido por FND-18) ya muestra `metadata: { lang: { String: "en" } }`, o sea
que la fricción de `VantaValue` está documentada pero sigue presente.

---

## 2. Análisis idiomático — contraste con referentes

### 2.1 Python

**Context manager.** El referente stdlib es `sqlite3.Connection` como context manager.
Fuente oficial: docs.python.org — *"Connection objects can be used as context managers
that automatically commit or rollback transactions"* — **no cierra la conexión**. Ese
gotcha está bien documentado (blog.rtwilson.com: "assumed it would close... it turns
out that's not the case"). Para VantaDB el CM correcto es el **contrario**: `close()`
es la barrera de durabilidad (drena operaciones in-flight, hace flush), así que
`__exit__` debe llamar a `close()`, no a un commit/rollback. Modelo: `open()`/`socket`
(CM que cierra) en vez del CM de transacción de sqlite3. `AsyncVantaDB` ya lo hace así
(`__aexit__` → `to_thread(close)`).

**Type hints modernos.** El estándar 2026: `TypedDict` para hits/metadata, `@overload`
para APIs con modos (el caso `put_batch` grita overloads), `Protocol`/`Buffer` para
vectores numpy en vez de `Any`. chromadb acepta `metadata` como plain `dict[str, Any]`
sin TypedDict — así que el nivel de VantaDB no está mal, pero TypedDict para el shape
de `VantaSearchHit`/filtros daría autocomplete gratis.

### 2.2 TypeScript

**Async.** El referente es libsql (`@libsql/client`): `client.close()` en try/finally o
`Symbol.asyncDispose` con `await using`. TS 5.2+ (el proyecto usa `^5.7.0`) soporta
`using`/`await using` nativo — fuente oficial: devblogs.microsoft.com (Announcing
TypeScript 5.2, "using Declarations and Explicit Resource Management"). Requiere
`lib: ["esnext.disposable"]` o polyfill de los símbolos. El tsconfig actual
(`target: ES2022`) no lo tiene → gap de configuración además de API.

**Tipos.** El tagged-union `VantaValue = { String: string } | { Int: number }...` es el
enum Rust serializado 1:1 (serde) — NO es cómo se escribe metadata en JS. duckdb-node,
libsql y chromadb JS usan `Record<string, unknown>` / JSON values planos. El wrapper
tiene `guards.ts` que ya valida el shape — el trabajo de normalización (`"en"` → 
`{String: "en"}`) podría ir en una capa de conversión en vez de exigirlo al usuario.

---

## 3. Gaps concretos

### 3.1 Python

| # | Gap | Ubicación | Severidad |
|---|---|---|---|
| PY-1 | **Stub drift `connect`**: `__init__.pyi:293` declara `connect()` pero `__init__.py` NO lo importa del módulo nativo (importa solo VantaDB, VantaListResult, VantaMemoryRecord, VantaSearchHit, VantaVector, `__version__`). `from vantadb_py import connect` → ImportError en runtime; mypy lo da por válido. El pyfunction existe (`lib.rs:1942`) pero no se re-exporta. | `vantadb_py/__init__.pyi:293`, `vantadb_py/__init__.py:12` | 🔴 rompe-uso (type checker miente) |
| PY-2 | **Stub drift `__version__`**: stubs declaran `def __version__() -> str` (función) pero lib.rs:1962 lo agrega como atributo string. `__version__()` → TypeError en runtime. | `vantadb_py/__init__.pyi:294`, `vantadb_py/vantadb_py.pyi:183`, `src/lib.rs:1962` | 🔴 rompe-uso |
| PY-3 | **Stub drift tipos de retorno**: `get_memory` stub dice `dict \| None` (`__init__.pyi:66`) pero runtime devuelve `VantaMemoryRecord \| None` (`lib.rs:793`); `list_memory` stub dice `dict` (`__init__.pyi:79-85`) pero runtime devuelve `VantaListResult` (`lib.rs:880`); `put` stub dice `dict` (`__init__.pyi:37-45`) pero runtime devuelve `VantaPyMemoryRecord`. IDE sugiere `.payload` (funciona) pero el type checker lo rechaza. | `vantadb_py/__init__.pyi:37,66,79`, `src/lib.rs:793,880` | 🟡 fricción (stub miente) |
| PY-4 | **Sin context manager en `VantaDB` sync**: `close()` existe pero `with VantaDB(path) as db:` no funciona (0 matches de `__enter__/__exit__` en `src/`). Usuarios deben recordar `close()` o `try/finally` manual — riesgo de DB sin cerrar (WAL sin flush). | `vantadb-python/src/lib.rs` (pyclass VantaDB), `vantadb_py/vantadb_py.pyi:28-118` | 🟡 fricción + riesgo recurso |
| PY-5 | **`put_batch` dual API sin overloads**: 9 parámetros, dos modos de llamada (entries posicional vs keys/vectors/payloads...). Stub sin `@overload` → no hay autocomplete de modo. Deuda P2-5 ya registrada (AGENTS.md Regla 6). | `vantadb_py/__init__.pyi:46-56`, `src/lib.rs` | 🟡 fricción (deuda conocida) |
| PY-6 | **Sin TypedDict**: `metadata: dict`, `filters: dict`, hit shapes sin TypedDict → sin autocomplete de campos. | `vantadb_py/__init__.pyi:44,71-77` | 🟢 cosmético |
| PY-7 | **Duplicación de stubs**: `vantadb_py.pyi` (183L) y `__init__.pyi` (294L) duplican la clase VantaDB completa — riesgo de drift (ya difieren: AsyncVantaDB/connect solo en `__init__.pyi`). | `vantadb_py/vantadb_py.pyi`, `vantadb_py/__init__.pyi` | 🟢 cosmético |

### 3.2 TypeScript

| # | Gap | Ubicación | Severidad |
|---|---|---|---|
| TS-1 | **`VantaMetadata` tagged-union = enum Rust 1:1**: usuario debe escribir `{ lang: { String: "en" } }` (README.ts:45). Idiomático JS es `{ lang: "en" }` con `Record<string, unknown>` o un tipo `MetadataValue = string \| number \| boolean \| null \| ...`. Fricción en TODA escritura de metadata. | `vantadb-ts/src/types.ts:1-12`, `README.md:45` | 🟡 fricción (uso común) |
| TS-2 | **Sin `Symbol.asyncDispose` / `await using`**: `NativeVantaDB.async close()` existe (`native.ts:136`) pero la clase no implementa `[Symbol.asyncDispose]` → no hay `await using db = await NativeVantaDB.connect()`. Faltaría además `lib: ["esnext.disposable"]` en tsconfig (TS 5.7 lo soporta; el target ES2022 es correcto). | `vantadb-ts/src/native.ts:80-145`, `tsconfig.json:3-12` | 🟡 fricción + config |
| TS-3 | **README engaña con sync/async**: README.ts:10 usa `await db.put(...)` sobre `VantaDB.create()` que es **sync** (vantadb.ts:213). El `await` es no-op — funciona pero confunde (el lector cree que put es async). | `vantadb-ts/README.md:10`, `src/vantadb.ts:213` | 🟢 cosmético |
| TS-4 | **Unions amplias `string \| number`**: `created_at_ms`, `version`, `node_id` — defensivas por JS number-safety pero el consumidor no sabe cuál recibirá. Un `bigint` normalizado o conversión en el wrapper lo resolvería. | `vantadb-ts/src/types.ts:28-36` | 🟢 cosmético |
| TS-5 | **`QueryResult` con campos opcionales** en vez de unión discriminada estricta (`{kind:"Read", nodes} \| {kind:"Write",...}`). | `vantadb-ts/src/types.ts:110-114` | 🟢 cosmético |

---

## 4. Prototipos idiomáticos

Dos archivos de ejemplo en `docs/examples/` (NO modifican el SDK):

### 4.1 Python — `with VantaDB(path) as db:` (FND-05 gap PY-4)

`docs/examples/fnd05_python_context_manager.py`

Subclase de `VantaDB` que añade `__enter__`/`__exit__` llamando `close()` — el patrón
`open()`/`socket` (CM que libera el recurso), que es el correcto para VantaDB porque
`close()` es la barrera de durabilidad (a diferencia de sqlite3 cuyo CM hace
commit/rollback y NO cierra). Incluye el mismo patrón para `AsyncVantaDB` (ya existe,
se demuestra). Comentario del cambio que haría falta en `src/lib.rs` (agregar
`fn __enter__`/`fn __exit__` al pyclass) como recomendación de implementación.

### 4.2 TypeScript — `await using db` (FND-05 gap TS-2)

`docs/examples/fnd05-ts-async-dispose.ts`

Wrapper `DisposableVantaDB` sobre `NativeVantaDB` que implementa
`[Symbol.asyncDispose]` → permite `await using db = await connectVantaDB(path)` con
disposición automática (TS 5.2+, fuente: devblogs.microsoft.com/typescript/
announcing-typescript-5-2/). Documenta el cambio de tsconfig necesario
(`lib: ["esnext.disposable"]`). También muestra el fallback `try/finally` actual.

---

## 5. Recomendación de implementación (priorizada)

**Prioridad 1 — Fix de stubs Python (PY-1, PY-2, PY-3).** Barato (editar 2 `.pyi`),
elimina errores de runtime que los type checkers no detectan. `connect` → importarlo
en `__init__.py` o quitar del stub; `__version__` → atributo no función; retornos
`VantaMemoryRecord`/`VantaListResult`. Test: `python -c "import vantadb_py; print(vantadb_py.connect); print(vantadb_py.__version__)"`.

**Prioridad 2 — Context manager en `VantaDB` (PY-4).** Agregar `__enter__`/`__exit__`
al pyclass en `src/lib.rs` (~10 líneas, sin unsafe, GIL-safe: `__exit__` llama a
`close()` existente). Doble beneficio: sync y como base de AsyncVantaDB. Actualizar
README quickstart a `with VantaDB(...) as db:`.

**Prioridad 3 — `Symbol.asyncDispose` en TS (TS-2).** Agregar
`[Symbol.asyncDispose](): Promise<void> { return this.close(); }` a `NativeVantaDB`
(y `Symbol.dispose` a `VantaDB` WASM sync). Cambio en tsconfig:
`lib: ["ES2022", "ESNext.Disposable"]`. Actualizar README a `await using`.

**Prioridad 4 — Normalizar `VantaMetadata` (TS-1).** Capa de conversión
`normalizeMetadata(input: Record<string, unknown>): VantaMetadata` en el wrapper +
tipo de entrada amigable (`JsonMetadata = Record<string, string | number | boolean | null | JsonMetadata | ...>`).
Requiere coordinar con el core (el engine recibe el tagged-union). **Este es el único
gap que toca el core Rust** → requiere spec previa (FND-06 boundary) antes de tocar.

**Prioridad 5 — TypedDict + overloads (PY-6, PY-5).** TypedDict para
`VantaSearchHit`-like shapes y `@overload` para `put_batch` (paga deuda P2-5).

**Diferidos:** TS-3 (README await cosmético — fix con Prioridad 3), TS-4/TS-5
(cosméticos, union discriminada en QueryResult al tocar TS-1), PY-7 (dedupe de stubs
— consolidar en `__init__.pyi` como fuente única).

**No hacer (por ahora):** zero-copy numpy arrays (FND-04 ya cubre), async nativo
Rust-side en PyO3 (el wrapper `to_thread` es suficiente y correcto), rewrite de la
API de graph methods.

---

## 6. Fuentes

- sqlite3 Connection as context manager (commit/rollback, **no** close): https://docs.python.org/3/library/sqlite3.html (verificado 2026-08-16)
- TypeScript 5.2 `using`/`await using`, `Symbol.asyncDispose`, `AsyncDisposable`, lib `esnext.disposable`: https://devblogs.microsoft.com/typescript/announcing-typescript-5-2/ (verificado 2026-08-16)
- chromadb Python API (metadata as plain dict): https://docs.trychroma.com/ (referencia de contraste)
- libsql TypeScript client (async, close()): https://docs.turso.tech/sdk/ts (referencia de contraste)
- sqlite3 CM gotcha (no cierra): https://blog.rtwilson.com/a-python-sqlite3-context-manager-gotcha/ (referencia)

## 7. Anexo — evidencia de verificación

- `grep __enter__|__exit__|__aenter__|__aexit__|asyncio|async src/*.rs` → **0 matches** (confirma PY-4: sync VantaDB sin CM; AsyncVantaDB vive en `__init__.py`)
- `vantadb_py/__init__.py:12` importa SOLO `VantaDB, VantaListResult, VantaMemoryRecord, VantaSearchHit, VantaVector, __version__` — sin `connect` (confirma PY-1)
- `src/lib.rs:1962` `m.add("__version__", metadata::reported_version().into_owned())` — atributo string (confirma PY-2)
- `src/lib.rs:793` `PyResult<Option<VantaPyMemoryRecord>>`; `src/lib.rs:880` `PyResult<VantaPyListResult>` (confirma PY-3)
- `vantadb-ts/src/native.ts:136` `async close()`; `vantadb-ts/src/vantadb.ts:157` `close()` sync — ninguna clase implementa `[Symbol.asyncDispose]` (confirma TS-2)
- `vantadb-ts/tsconfig.json` — `target: ES2022`, sin `lib` explícita → sin `ESNext.Disposable` (confirma TS-2 config)