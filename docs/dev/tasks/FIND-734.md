---
title: "FIND-734: la clase de ruido TSan cambió de firma y el suppression file no la cubre"
kind: task
status: active
description: "Run 45668284667: 822 tests medidos, 243 reportes de una sola clase (std sin instrumentar). Ninguna suppression es efectiva y estrecha → se retira la lane TSan con CATEGORY y receta de retorno"
tags: [ci, tsan, gates]
---

# FIND-734: la clase de ruido TSan cambió de firma y el suppression file no la cubre

## Metadata

- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6.b / §9.b
- **Fuente:** fila `FIND-734` derivada de `FIND-714` (el contrato de la lane TSan sigue abierto)
- **Esfuerzo:** 🟡 2-3 h · **Prioridad:** 🟡 Media (no bloquea el merge; sí decide si un run verde es honesto)
- **Tipo:** CI/CD — sin código de producto
- **Carril:** C
- **Dependencias:** `FIND-714` ✅ (`4ca972ddb`, suppression + aserción) · `FIND-730` ✅ (`98e752079`, el verificador que ahora sí verifica)
- **Derivada:** `FIND-737` (spike `-Zbuild-std` que devuelve la señal) — texto de la fila en §"Receta de retorno"
- **Creado:** 2026-11-25

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Nadie: el job `sanitizer-tsan` no era check requerido (`continue-on-error`), así que retirarlo no puede bloquear un merge. `docs-gates.yml` cuenta *workflow files*, no jobs — el inventory gate no se ve afectado |
| Callees | El job consumía `dev-tools/tsan-suppressions.txt` (vía `TSAN_OPTIONS.suppressions`), `llvm-symbolizer`, `.config/nextest.toml` y el `-E` filterset. Sin job, los cuatro quedan sin ese consumidor |
| Implicaciones | Se pierde la única señal automatizada de data races. El riesgo real de la tarea es **el opuesto**: dejar un gate 100 % ruidoso que normaliza el rojo. La receta de retorno queda documentada para que la señal vuelva con fundamento, no con otra suppression |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o en la extensión necesaria):** `.github/workflows/ci-rust.yml` L14-58 (los dos bloques `paths:` con la entrada de suppressions) y L1025-1245 (job `sanitizer-tsan` entero: header con los comentarios de `FIND-650`/`FIND-714`/`FIND-227`, `continue-on-error` con su `# CATEGORY:`, los 4 steps y la aserción); `dev-tools/tsan-suppressions.txt` (27 L); `docs/dev/operations/CI_POLICY.md` L45-145 (tabla de jobs + §TSan completa); `src/index/distance/kernels.rs` L1-40 y L260-280 + `src/index/distance/metrics.rs` L1-20 (para decidir si el reporte 42 es un race real).
- **Referencias hacia dentro:** `dev-tools/tsan-suppressions.txt` solo lo consumía ese job; `CI_POLICY.md` lo citaba en la tabla de jobs y en §TSan; ningún script ni hook lo importa. `.github/workflows/ci-rust.yml` es check del ruleset de `develop`/`main`, pero el job retirado no era required.
- **Referencias hacia fuera del job:** `TSAN_OPTIONS` + `llvm-symbolizer` + `.config/nextest.toml` + el `-E` filterset + `nextest --partition hash` + el step de aserción de `FIND-730`. Todo deja de tener consumidor salvo `.config/nextest.toml` (lo usan los demás jobs).
- **Veredicto de impacto:** **bajo en corrección, alto en señal.** No se toca código, ni el resto de lanes, ni la aserción de `FIND-730` (se retira con el job que la contenía, porque sin job no hay partición que verificar).

## Diagnóstico — medido, no derivado

Run `45668284667`, job `139404898566` (partición 1/4, sha `b7a5d2eab`). `FIND-714` hizo su trabajo:
la lane **mide**. `822 tests run (selected == executed)`, sin abortos, la aserción de `FIND-730`
parsea el número. Lo que no cambió es el veredicto:

```
partition 1/4: 822 tests run (selected == executed)
race reports: 243
suppression hits: 0
```

`suppression hits: 0` **no es un problema de formato**: las dos entradas de
`dev-tools/tsan-suppressions.txt` (`race:std::sync::mpmc`, `race:CompletedTest`) describen una
implementación del harness que este nightly ya no usa. El reporte de hoy es otro tipo:

```
Write of size 8 at 0x724400000020 by main thread:
  #0 free
  #1 <alloc::rcs::arc::Arc<std::sync::poison::mutex::Mutex<core::option::Option<test::run_test::{closure#0}>>> as Drop>::drop  alloc/src/rcs/arc.rs:3030
  #2 core::ptr::drop_glue::<Arc<Mutex<Option<test::run_test::{closure#0}>>>>
  #3 test::run_test                                    library/test/src/lib.rs:687
Previous read of size 8 at 0x724400000020 by thread T1:
  #0 memcpy
  #1 core::profiling::compiler_move::<test::run_test::{closure#0}, 288>
  #2 <core::option::Option<test::run_test::{closure#0}>>::unwrap
  #3 test::run_test::{closure#1}                       library/test/src/lib.rs:677
```

Baseline `06485fe8e`, job `139058583998`: `821 … 258 failed`, `race reports: 258`, misma clase →
crónico, no regresión.

### Los 243 reportes, clasificados (no muestreados)

Extraídos del log completo del job y agrupados por firma:

| N | Firma | Qué es |
|---|---|---|
| 176 | `Arc<Mutex<Option<test::run_test::{closure#0}>>>` (drop vs `compiler_move`) | plumbing de eventos de libtest, protegido por un `std::sync::Mutex` |
| 58 | `free`/`dealloc_nonnull` vs `Atomic<usize>::fetch_sub` sobre `Arc<flume::Hook<fjall::worker_pool::WorkerMessage>>` | refcount de `Arc` en el teardown de fjall/flume, todo dentro de `std` |
| 8 | `blocking::pool` de tokio: 7 de shutdown (`ThreadManagementState::begin_shutdown`) y 1 de spawn (`VecDeque::push_back_mut`, warn #37) | un `std::sync::Mutex` haciendo su trabajo, invisible |
| 1 | init de `OnceLock<DistanceKernels>` (`kernels.rs:275`, warn #42) | el `Once` basado en futex |

### Por qué el ruido reaparece con otra firma

La causa es la misma de `FIND-227` y no cambia entre nightlies: **`std` no está instrumentado**
(no hay `-Zbuild-std`, llega precompilado). TSan aprende sincronización por dos vías —
interceptores libc (`pthread_mutex_lock`) e instrumentación de atómicos en compilación — y la
sincronización de Rust no llega por ninguna: `std::sync::Mutex`, `Once`/`OnceLock` y los refcounts
de `Arc` son futex + atómicos **dentro de `std`**. El log lo dice literalmente: la pila pasa por
`std::sys::sync::once::futex::Once::call` — un futex, no un `pthread_mutex`, así que el interceptor
no dispara. Lo que cambia entre nightlies es **qué estructura interna de `libtest`/`std` queda en
la pila**, no la causa: por eso una lista de firmas es una lista viva que se rompe cada pocas
semanas.

Rust Unstable Book, [`sanitizer`](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html):
*"Using it without instrumenting all the program code can lead to false positive reports"* y
*"It is strongly recommended to combine sanitizers with recompiled and instrumented standard
library, for example using cargo `-Zbuild-std`"*.

## Decisión — opción (b): retirar la lane

La tarea pedía, en orden: (1) ampliar el suppression file con la firma nueva, (2) verificar que se
aplica, y (3) si no basta, retirar el job con `CATEGORY`. El paso (1) es viable —`race:` matchea
contra nombres de función **y** de fichero de la pila reportada y de la de acceso previo, como
substring con `*` implícito, ver
[ThreadSanitizerSuppressions](https://github.com/google/sanitizers/wiki/ThreadSanitizerSuppressions)
y `IsSuppressed` en
[`tsan_suppressions.cpp`](https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/tsan/rtl/tsan_suppressions.cpp)—
pero **no basta**, y el motivo no es el formato: es el alcance.

Entre los 243 reportes hay uno que **no** tiene ni una sola pila de harness ni de dependencia:

```
Read of size 8 at 0x5555577e3420 by thread T4:
  #0 vantadb::index::distance::kernels::f32_dot_and_norm_b_sq  src/index/distance/kernels.rs:275
  #1 vantadb::index::distance::metrics::f32_l2_norm            src/index/distance/metrics.rs:14
  #2 vantadb::index::core::tests::concurrent_insert_preserves_hnsw_invariants::{closure#0}
  …
Previous write of size 8 at 0x5555577e3420 by thread T3:
  #0 <core::mem::maybe_uninit::MaybeUninit<DistanceKernels>>::write
  #1 <std::sync::once_lock::OnceLock<DistanceKernels>>::initialize   library/std/src/sync/once_lock.rs:574
  #2 <std::sync::once::Once>::call_once_force                       library/std/src/sync/once.rs:227
  #3 <std::sys::sync::once::futex::Once>::call                      library/std/src/sys/sync/once/futex.rs:183
  …
```

`select_kernels()` es `KERNELS.get_or_init(..)` (`src/index/distance/kernels.rs:31-34`): código
correcto, y el acquire ocurre dentro del `Once` no instrumentado. Es el **mismo defecto** que los
otros 242, alcanzado sin harness y sin dependencias.

De ahí la conclusión:

1. Un token estrecho para el warn #42 (el 42.º `WARNING` del log de esa partición) **existe**, pero
   tiene que nombrar un tipo del motor: `OnceLock<vantadb::index::distance::kernels::DistanceKernels>`
   (o `MaybeUninit<`). Es decir: el gate se vuelve verde apuntando a `vantadb::index::distance`, el
   hot path de la Regla 9.
2. Las entradas que cubrirían los otros 67 (`fjall`, tokio, `flume`) nombran la cola de escritura de
   fjall y el pool de tokio: justo donde un race real sí sería grave.
3. Pero el argumento decisivo **no es** que sea imposible: es que caduca. Las 2 entradas de
   `FIND-714` se cargaron en este mismo run (`Using suppressions: … (2 entries)`) y dieron
   `suppression hits: 0` en ~24 h, porque libtest cambió su canal de eventos. Cualquier lista viva de
   internals de `libtest`/`fjall`/`tokio` se mantiene contra cada nightly, y para #42 además nombra
   tipos del motor. Un verde así no es señal: es un compromiso de mantenimiento que nadie firmó.

Con eso, el estado actual (cuatro jobs rojos en cada run, exentos por `continue-on-error`) es
exactamente lo que la Regla 2 prohíbe: un rojo crónico que nadie mira. La salida honesta es
retirarla y dejar la receta de retorno con el prerrequisito correcto (`-Zbuild-std`), no con otra
lista de firmas.

**Lo que este retiro NO autoriza a concluir.** 67 de los 243 reportes llevan frames `vantadb::` (la
cadena de drop de `FjallBackend`/`StorageEngine`, `gc::spawn_memory_ttl_sweeper`, cuerpos de test),
aunque el par en conflicto esté siempre en `std`. Un double-free real en el teardown de `FjallInner`
imprimiría la misma forma y TSan no podría distinguirlo. La frase correcta es "el instrumento no
puede adjudicar", nunca "el motor está limpio".

**La disciplina del fichero se cumplió hasta el final.** Las dos entradas que existían sí pasaban
el criterio `0 hits en src/ y tests/` (`rg -c 'run_test|Arc<Mutex<Option' src/ tests/` → el único
match es el nombre de fichero `dry_run_test.json` en `src/cli_handlers/mcp_init.rs:473`). No es que
el criterio estuviera mal: es que este defecto no tiene token propio **que no sea del motor**.

## Lo que se retira y dónde queda la receta

### Coste medido de la receta (`-Zbuild-std`) — no estimado

Antes de darla por viable, medida. Sobre `rustc 1.101.0-nightly (c36f14571 2026-10-01)`, host
Windows, `CARGO_TARGET_DIR` limpio (sin sccache), compilando `std` instrumentado para
`x86_64-unknown-linux-gnu`:

```
rustup component add rust-src --toolchain nightly   → "component rust-src is up to date" (ya estaba)
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly build -Zbuild-std --target x86_64-unknown-linux-gnu
  → ERROR: linker `cc` not found        (3.24 min, 164 MB de artefactos)
```

Lectura honesta de ese número:

- **Lo que sí mide:** 3.2 min de compilación de `std` (core/alloc/proc_macro/std_detect/…) con
  instrumentación TSan. Ese es el coste **añadido** sobre hoy, donde `std` viene precompilado (0 min).
- **Lo que NO mide:** el enlace (falla en Windows por no haber linker Linux; son segundos), ni el
  grafo de dependencias de `-p vantadb` (arrow/fjall/tokio…), ni el crate. Es decir, el build por
  pierna = std (3.2 min) + deps + crate.
- **Lo que NO se midió:** si las 4 particiones caben en el step de 50 min con build-std. Eso exige un
  push real y es el criterio de cierre de `FIND-737`; acá no se afirma que quepa.

| Qué | Antes | Ahora |
|---|---|---|
| Job `sanitizer-tsan` | 4 particiones `nextest` con `-Zsanitizer=thread`, `--no-fail-fast`, step de aserción (`FIND-730`) y `continue-on-error` con `CATEGORY: NON-CRITICAL` | **eliminado** de `.github/workflows/ci-rust.yml` (el fichero termina en el job `sanitizer-asan`) |
| `dev-tools/tsan-suppressions.txt` | 2 entradas libtest (`race:std::sync::mpmc`, `race:CompletedTest`) + 19 L de criterio | **borrado**. Una lista sin consumidor invita justo al diagnóstico equivocado de hoy (`Matched: 0` leído como "formato roto") |
| `paths:` de `push`/`pull_request` | incluían `dev-tools/tsan-suppressions.txt` | entradas y sus dos comentarios eliminados |
| `docs/dev/operations/CI_POLICY.md` | fila de job + §TSan (contrato y alcance del suppression) | fila eliminada; §TSan reescrita como **nota de retiro**: medición, causa raíz, por qué una suppression no puede, y receta de retorno con `-Zbuild-std` |
| Receta de retorno | — | `CI_POLICY.md` §"Bringing the lane back": `rust-src` + `-Zbuild-std` + `continue-on-error` ausente + sin suppression file + job programado, nunca en cada PR |

El estado del repo queda: **cero jobs TSan**, y el conocimiento de por qué (con números) en el
documento de política, que es donde vive el contrato de CI.

## Verify

Verificación **en un worktree temporal sobre `HEAD` + este cambio**, no en el árbol compartido: el
árbol de trabajo tiene WIP de otros carriles, así que `gen-index --check` ahí siempre difiere por
razones ajenas. El worktree reproduce exactamente lo que verá CI tras el commit.

| Comando | Resultado |
|---|---|
| `rg -n 'sanitizer-tsan\|tsan-suppressions' .github/` | **0 hits** — job, `paths:` y comentarios limpios |
| `git diff --numstat -- .github/workflows/ci-rust.yml` | `0 227` — **solo borrado**, cero líneas añadidas (así que ningún `continue-on-error` nuevo, ningún pin tocado, ningún `\|\| true`) |
| `actionlint .github/workflows/ci-rust.yml` | **exit 0** |
| `rg -c 'run_test\|Arc<Mutex<Option' src/ tests/` | **1 hit, y es el nombre de fichero** `dry_run_test.json` (`src/cli_handlers/mcp_init.rs:473`): la firma del harness no existe en nuestro código |
| `node scripts/docs/gen-index.mjs --check` (worktree) | **exit 0** — 768 documentos, índice consistente con `HEAD` + este cambio |
| `node scripts/docs/check-links.mjs` (worktree) | **exit 0** — 0 enlaces markdown rotos |
| `node scripts/docs/check-docs.mjs` (worktree) | **exit 0** — `GATING: all clear`, orphans 0 |
| `git ls-files --eol .github/workflows/ci-rust.yml docs/dev/operations/CI_POLICY.md` | `i/lf w/lf`, sin BOM |
| `rg -n 'tsan-suppressions'` (excluyendo `tasks/`, `avance/`, `plans/`) | **1 hit, y es esta nota de retiro** (`CI_POLICY.md:85`) — 0 referencias vivas al fichero borrado |
| `pwsh dev-tools/ocr-review.ps1` | el preview marca `.md`/`.yml` como `excluded: unsupported_ext`; el único dato útil es la forma del diff (`ci-rust.yml +0/-227`, borrado puro). Rule Group cargado con `ocr delegate rule .github/workflows/ci-rust.yml` (`project / {.github/**}`): revisadas sus reglas de seguridad/corrección — **`needs:` roto** es el riesgo real de retirar un job y `rg 'sanitizer-tsan' .github/` = 0 lo cierra; permisos, pins, secrets, `pull_request_target` y `shell` no cambian (diff de solo borrado). Sin Critical/High |

**Nota sobre los índices generados.** `gen-index.mjs --write` se corrió en un worktree temporal
sobre `HEAD` + `FIND-734.md`, no en este árbol: el árbol compartido tiene WIP de otros carriles
(`FIND-733`, `MCP-48`, `MEM-72`, `ADR-0064`) y regenerar aquí habría metido sus filas —con enlaces a
ficheros todavía sin commitear— en este commit. Efecto colateral correcto: `HEAD` ya listaba las
filas de `MCP-48` y `MEM-72`, cuyos ficheros **no** están en git, así que el índice bajó de 231 a
230 filas de task + `FIND-734`. Eso es una corrección, no una regresión: `gen-index --check` falla
en un checkout limpio de `HEAD` por esas dos filas colgantes, y cada carril las recupera al correr
`gen-index --write` cuando commitea su task file.

## Steps

| # | Step | Estado |
|---|------|--------|
| 1 | Clasificar los 243 reportes del run `45668284667` por firma (176/58/9) y confirmar que `suppression hits: 0` es firma nueva, no formato | ✅ |
| 2 | Leer `kernels.rs`/`metrics.rs` y confirmar que el reporte 42 (`OnceLock`) es código correcto, no un race del motor | ✅ |
| 3 | Validar la sintaxis real de `suppressions` contra la wiki y el código de TSan (`IsSuppressed`, las dos pilas, substring) | ✅ |
| 4 | Retirar el job `sanitizer-tsan` + sus dos entradas de `paths:` de `ci-rust.yml` | ✅ |
| 5 | Borrar `dev-tools/tsan-suppressions.txt` (sin consumidor) | ✅ |
| 6 | Reescribir `CI_POLICY.md` §TSan como nota de retiro con medición, causa raíz y receta `-Zbuild-std`; quitar la fila de la tabla de jobs | ✅ |
| 7 | Verify mecánico + OCR + review P2-01 + commit local | ✅ |

## Lo que esta tarea NO hace

- **No** toca `src/**` ni `tests/**`: el warn #42 se diagnostica por lectura; si alguna vez se
  quiere *verificar* que `OnceLock` publica bien, eso es un test nuevo, no este cambio.
- **No** toca la lane ASan (`sanitizer-asan`), el `-E` filterset, `.config/nextest.toml` ni la
  aserción de `FIND-730` (se va con el job que la contenía).
- **No** introduce ningún `-Zbuild-std`: es la receta de retorno, y validarlo en CI exige un push
  real. Va como fila propia (`FIND-737`), no como cambio a ciegas.
- **No** edita `docs/dev/workflow/ci-rust-10.md` ni `docs/dev/avance/activo/ci-cd.md` (ver §Deuda).
- **No** hace push (política de git: el push es acto explícito del owner).

## Deuda

- **`docs/dev/workflow/ci-rust-10.md` queda stale en cuatro sitios**, no uno: `:43` (fila de tabla
  con la lane viva), `:50` ("TSan salta …"), `:66` ("Data races (TSan, continue-on-error …)") y
  `:72` ("ASan y TSan corren en paralelo … Ambos suprimen el error de ABI mismatch … para evitar el
  overhead de `-Z build-std`"). La última es la más dañina: presenta la omisión de `-Zbuild-std` como
  una optimización deliberada, que es justo la causa raíz que esta tarea identifica. Ese documento ya
  venía desactualizado en otras filas (`wasm-test`, `osv`, `machete`), así que tocarlo a medias sería
  media verdad; fuera del allowlist declarado.
- **`docs/dev/avance/activo/ci-cd.md:908,983`** describen `dev-tools/tsan-suppressions.txt` como
  vigente y su `suppression hits: 0` como deuda abierta. El registro canónico se corrige con el
  `progreso` de cierre, que esta tarea no corre (`avance/**` está fuera de su scope): conviene que la
  entrada de cierre lo nombre para que nadie lo lea como un fichero existente.
- **La señal de TSan no existe hasta que haya `-Zbuild-std`.** No es deuda nueva: es el estado
  honesto, y su retorno está registrado como `FIND-737` (fila de Backlog, no solo prosa).
- **`Miri` sigue siendo la única herramienta de UB en CI.** No sustituye a TSan. La cobertura de
  concurrencia mientras la lane está retirada es explícita: `vanta-chaos` (stress/deadlock, Regla 8),
  los tests de concurrencia del core (`memory_concurrency`, tests `concurrent_*`/`*_concurrency_*`) y
  `FIND-328`; ninguna de las tres *ve* un data race — se apoyan en que un race se manifieste.

## De qué cubre el hueco (mientras la lane está retirada)

Retirar TSan deja de tener un detector de data races *estático respecto al timing*. Lo que queda, con
su nombre y su límite:

| Señal | Qué detecta | Qué **no** detecta |
|---|---|---|
| `vanta-chaos` (stress + deadlock, Regla 8) | deadlocks, corrupción bajo carga, quiescencia de WAL/snapshots | un data race que no cambie un observable |
| `memory_concurrency` + tests `concurrent_*` / `*_concurrency_*` del core | invariantes de HNSW/índices bajo escritura concurrente; UAF/panic si el race se manifiesta | races silenciosos (lectura rasgada que "casi siempre" está bien) |
| `FIND-328` (mutation testing acotado) | tests que no matan mutantes — incluidos los de concurrencia | races que ningún test ejercita |
| `miri` (job `miri`) | UB en los tests etiquetados `miri` | concurrencia real: Miri no ejecuta `rayon`/`tokio` multi-hilo |

Conclusión honesta: **queda un hueco**, y es el de los races que no se manifiestan como fallo. Está
cubierto por nombre y no por omisión: el día que `FIND-737` cierre, el hueco se cierra con el
instrumento correcto en vez de con una lista de firmas.

## Review (P2-01)

- **Reviewer:** `vanta-review`, contexto fresco (sesión `ses_debe2ffc2ffenWEFWaTaZ7SXdU`), sin acceso
  a la sesión que implementó. Tier Fast (`.github/workflows/**`, `docs/dev/**`, `dev-tools/**`).
- **Veredicto: ✅ `approve`.** Cero Critical, cero Required. El reviewer **re-derivó** la evidencia en
  vez de creerla: sacó los 243 reportes del log con `gh api` y le salió 176/58/8/1 exacto, confirmó
  `library/test/src/lib.rs:687` ×176 y `compiler_move` ×176, comparó el warn #42 frame a frame,
  verificó `select_kernels()` = `KERNELS.get_or_init(..)` en `kernels.rs:31-34`, y comprobó el
  baseline `06485fe8e` (821/258 → crónico). Re-ejecutó el contrato: `numstat 0/227`, `actionlint` 0,
  `rg` en `.github/` 0 hits, y el único hit de `run_test` es `dry_run_test.json`
  (`src/cli_handlers/mcp_init.rs:473`). Regeneró el índice sobre `HEAD` + `FIND-734.md` y obtuvo los
  mismos hashes que el fichero staged (`--check` exit 0), confirmando que la metodología del worktree
  es real y no una excusa. Verificó además que `docs-gates.yml:667-672` cuenta *workflow files*, no
  jobs, así que retirar el job no roza el inventory gate.
- **Lo que corrigió la revisión (una ronda, 5 Optional + nits):**
  - **O1 (la importante) — "none of them is a race in VantaDB code" afirmaba de más.** 67 de los 243
    reportes sí llevan frames `vantadb::` en la cadena de drop. El par en conflicto está siempre en
    `std`, pero un double-free real en el teardown de `FjallInner` imprimiría la misma forma. Toda
    afirmación sobre "no hay races" pasó a "TSan no puede adjudicarlo", en `CI_POLICY.md` y acá.
  - **O2 — "`race reports: 0` es inalcanzable" era literalmente falso.** Existe un token estrecho
    para #42 (`OnceLock<vantadb::index::distance::kernels::DistanceKernels>`). El argumento se
    reemplazó por el verdadero: **caducidad** (2 entradas cargadas, `suppression hits: 0` en ~24 h) y
    que ese token nombra un tipo del motor.
  - **O3 — la deuda de `ci-rust-10.md` estaba subdeclarada**: son cuatro sitios (`:43`, `:50`, `:66`,
    `:72`), no uno; y `:72` presenta la omisión de `-Zbuild-std` como optimización.
  - **O4 — el spike de `-Zbuild-std` no estaba registrado como fila.** Nace `FIND-737` (deuda en
    prosa = nunca), con el criterio de cierre escrito.
  - **O5 + nits** — tokio eran 7 shutdown + 1 spawn (no "8 shutdown"); el bloque del warn #42 omitía
    frames sin elipsis; `rg 'tsan-suppressions'` devuelve 1 hit (esta nota), no 0; `Arc<flume::Hook<…>>`
    lleva el segundo parámetro (`flume::signal::SyncSignal`); `<Once>::call` con corchetes; y la cita
    del Unstable Book cortaba antes de "functionality".
- **Alternativas que el reviewer evaluó y descartó** (con el argumento): `-Zbuild-std` dentro de esta
  tarea (es *la* correcta pero exige push + medición para no cambiar un gate roto por otro no
  verificado — por eso va a `FIND-737`); suprimir las 3 familias y dejar #42 (la lista murió en 24 h);
  dejar la lane roja sin `continue-on-error` (243 reportes deterministas que nadie miraría en una
  semana); y lane programada sin build-std (mueve el ruido de sitio).
- **Su reserva, textual:** *"la única razón por la que no aprobaría esto sería O1 — no porque la
  decisión sea incorrecta, sino porque la frase que sobrevive dice 'no hay races' cuando lo que el
  instrumento permite decir es 'no puedo ver si los hay'"*. Corregido en ambos documentos.
