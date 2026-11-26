---
title: "FIND-732 — los tests dejan GB de bases Fjall en %TEMP% cuando el proceso muere sin Drop"
kind: task
status: completed
description: "15,35 GB en un día y tres gates abortados por StorageFull: un test que panica limpia, uno que muere sin ejecutar Drop (slow-timeout, abort) no. Recolector con inactividad + sonda de lock en verify.ps1 y pre-push."
---

# FIND-732 — el residuo de los tests es una DB temporal que nadie cierra

> **Plan:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` §6
> **Ruta:** `vanta-worker` · **Carril:** C · **Appetite:** 2-3h
> **Dependencias:** ninguna. Toca `.githooks/pre-push` **de forma aditiva** sobre lo que dejó `FIND-729`.

## Contrato

```
# medir antes
(Get-ChildItem $env:TEMP -Directory -Filter '.fjall_compaction_tmp_*' | Measure-Object).Count
# correr una suite que falle a propósito (test con panic / con abort)
# medir después y demostrar que NO queda residuo (o que el recolector lo borra)
```

Más un self-check que falla si el recolector no limpia lo huérfano o si toca lo
que no es suyo.

## Diagnóstico (medido, no asumido)

El síntoma declarado — *"los tests limpian al terminar bien, pero no al fallar"* —
era **falso a medias**. Lo que distingue los dos casos no es "bien" vs "fallar":
es **`Drop` vs no `Drop`**.

| Caso | ¿Corre `TempDir::drop`? | Residuo medido |
|---|---|---|
| 22 tests de fjall que **pasan** (`cargo nextest run -p vantadb --lib -E 'test(fjall)'`) | sí | **0** entradas en `%TEMP%` |
| un test que **panica** (`panic!("intentional")` tras abrir la DB) | sí (unwind) | **0** entradas |
| un test que **aborta** (`std::process::abort()`, lo que hace nextest al vencer `slow-timeout`) | **no** | `.tmpVHQREI` · **68,1 MB** · `lock=True` |

```
BEFORE tmp_dirs=0
FAIL [1.720s] find732_panic_leaves_nothing        -> 0 residuo
ABORT[2.751s] find732_abort_leaks_the_temp_db     -> 0xc0000409 (sin destructores)
AFTER  tmp_dirs=1  .tmpVHQREI  68,1 MB  lock=True
```

68,1 MB es el **techo** de una DB que alcanzó a pre-asignar el journal de fjall
(64 MiB): el promedio real del día fue 15,35 GB / 1.887 entradas ≈ **8,1 MB**,
porque la mayoría murió antes de escribir el journal completo. 1.887 entradas
con un techo de 68 MB serían 128 GB — el promedio, no el techo, es el número que
explica el 15,35 GB.

### Las dos fuentes de residuo

| Residuo | Quién lo crea | Por qué queda |
|---|---|---|
| `<TEMP>\.tmpXXXXXX` | `tempfile::TempDir` — prefijo por defecto `.tmp` (`tempfile-3.27.0/src/lib.rs:231`) | la limpieza vive en `Drop` (`src/dir/mod.rs:499-501`) y **`drop` ignora el error**. Upstream lo dice: *"`TempDir` and `NamedTempFile` will fail if their destructors don't run"* (`src/lib.rs:16-24`). nextest mata el proceso al vencer `slow-timeout = { period = "60s", terminate-after = 3 }` (`.config/nextest.toml`) |
| `<TEMP>\.fjall_compaction_tmp_<pid>` | `FjallBackend::compact()` — escribe la reescritura como **hermano** del data dir (`src/backends/fjall_backend.rs:512-523`) | el tmp sólo se borra al **inicio de la siguiente** compactación sobre el mismo path. Si el test muere a mitad, no se borra nunca; y si el data dir cuelga directo de `%TEMP%`, el residuo también |

### Hipótesis del ticket, resueltas

| Hipótesis | Veredicto |
|---|---|
| (a) tests que abren `Embedded` sobre un `tempdir` y no lo borran al morir por panic | **parcial**: con `panic` sí se borra (unwind); el leak es por muerte **abrupta** y por `drop` que ignora errores en Windows |
| (b) fjall deja su `compaction_tmp` si el proceso termina sin `Drop` | **confirmada**, pero el nombre lo genera VantaDB (`fjall_backend.rs:522`), no fjall |
| (c) algún test escribe en un path fijo en vez de un tempdir | **confirmada como amplificador**: `src/schema.rs:260` (`<TEMP>\vanta_schema_test`) y los `vanta_test_wal_*` / `vantadb_ser_test_*` ponen su DB directamente bajo `%TEMP%`, así que el tmp de compactación nace ahí |

## Opciones evaluadas

| Opción | Veredicto |
|---|---|
| **(a)** `tempfile::TempDir` como guard RAII | **Es lo que ya hay.** No cubre `abort`/SIGKILL: no hay destructor. Descartada como fix |
| **(b)** guard + `panic::set_hook` | **Rechazada**: el caso que filtra es justamente el que no ejecuta hooks ni destructores |
| **(c)** recolector por PID muerto + sonda de lock | **Elegida.** `compact()` mete el PID en el nombre; y toda DB de fjall tiene un `lock` abierto, así que la **liveness** se puede probar, no adivinar. El review P2-01 (F1) mostró que el lock **solo** no basta: hay ventanas vivas sin lock, así que la versión entregada exige además inactividad (`-MinAgeMinutes`, 10 min) |
| (d) redirigir `TEMP` de toda la suite bajo `target/` (perfil de nextest) | **Rechazada**: no arregla nada, sólo mueve el problema dentro de `target/`, no cubre `cargo test`, y un `TEMP` relativo rompe a quien asuma ruta absoluta |

**Ponytail:** no se toca ni un test. El 95% del residuo son DBs con `lock`
propio, y el lock da una prueba de vida exacta — no hace falta ninguna
heurística de mtime ni instrumentar 200 ficheros de test.

## Solución

`dev-tools/temp-collector.ps1`, con el mismo patrón de guardas que
`target-cleanup.ps1` (`Assert-UnderTemp` rechaza `%TEMP%` mismo, hermanos tipo
`Temp2\` y escapes con `..`):

1. `<TEMP>\.fjall_compaction_tmp_<pid>` → se borra si su `lock` está libre, o si
   el PID ya no existe.
2. `<TEMP>\.tmp*`, `<TEMP>\vanta*`, `<TEMP>\vantadb*` → se borra **sólo** si
   contiene un `lock` de fjall (≤4 niveles) **libre**, o si está vacío (0 bytes).
   Un `.tmp*` no vacío sin `lock` no se toca.
3. **Dos** pruebas de abandono, ambas obligatorias: inactividad del árbol
   (`-MinAgeMinutes`, 10 por defecto — cubre las tres ventanas con la DB viva y
   el lock libre: `compact()` cierra el tmp antes del swap, un test que cierra y
   reabre, y un `.tmp*` ajeno recién creado) y la sonda de lock (fjall mantiene
   abierto `<db>\lock`, `fjall-3.1.12/src/file.rs:11`, así que abrirlo con
   `FileShare.None` falla mientras siga abierto).

Se invoca solo desde `dev-tools/verify.ps1` (justo antes del paso de nextest) y
desde `.githooks/pre-push` (bloque aditivo antes del gate Rust — así un
`verify.ps1` que muere en `fmt`/`clippy` no deja residuo sin reclamar), y en
ambos casos es **advisory**: nunca cambia el veredicto del gate.

## Impacto mapeado (Regla 0)

**Leídos completos antes de editar**

| Archivo | Qué se leyó | Por qué |
|---|---|---|
| `dev-tools/target-cleanup.ps1` (326 L) | `Get-TargetPrefix`, `Assert-UnderTarget`, `-SelfCheck`, exit codes, guarda de builds en vuelo | es el patrón de guardas y de self-check que se replica |
| `.githooks/pre-push` (183 L) | bloques por área, `winpath()`, gate Rust, comentarios de `FIND-729` | dónde encaja una llamada que no debe bloquear |
| `dev-tools/verify.ps1` (100+ L) | `run()`, bootstrap MSVC, paso de nextest | punto de inserción antes de la suite |
| `src/backends/fjall_backend.rs:461-600` | `Compactable::compact` paso a paso (tmp hermano del data dir, swap con `ptr::read/write`) | prueba de que el `.fjall_compaction_tmp_<pid>` es nuestro |
| `tempfile-3.27.0/src/lib.rs:16-32,231` + `src/dir/mod.rs:499-501` | prefijo `.tmp`, `Drop` que ignora errores | causa raíz del `.tmpXXXXXX` |
| `fjall-3.1.12/src/file.rs:11`, `src/locked_file.rs` | `LOCK_FILE = "lock"`, `try_lock` | la sonda de vida se apoya en un invariante de la dependencia, no en una suposición |
| `docs/dev/references/troubleshooting.md` § disco y § git hooks | formato de sección | dónde va el síntoma |

**Referencias hacia dentro**

- `temp-collector.ps1` lo llaman `verify.ps1` y `pre-push`; ninguno depende de
  su exit code (`|| true` / `try{}catch{}`).
- No toca código Rust: cero impacto en la suite, en el engine o en los bindings.

**Referencias hacia afuera**

- `%TEMP%` (entorno), `Get-Process` (liveness), `FileShare.None` (lock probe).

**Veredicto:** blast radius = los dos gates que lo invocan, y ambos lo llaman de
forma tolerante a fallos. Reversible con un `git revert`.

## Steps

1. ✅ Diagnóstico medido — suite en verde deja 0; `abort` deja 68,1 MB con `lock`.
2. ✅ RED — repro descartable (`tests/find732_scratch.rs`, borrado) que fija el número antes del fix.
3. ✅ GREEN — `dev-tools/temp-collector.ps1` + self-check + recolección real del residuo medido.
4. ✅ Wire — `verify.ps1` (1 llamada advisory) + `pre-push` (bloque aditivo).
5. ✅ Troubleshooting § disco.
6. ✅ Review P2-01 (`vanta-review`, **changes-required**) → F1/F2 corregidos y re-medidos; F3-F12 aplicados o documentados. Commit local.

## Verificación (contrato)

**Antes de tocar nada** (el owner ya había borrado a mano los 15,35 GB):

```
%TEMP%: fjall=0  .tmp=0
```

**Control — 22 tests de fjall en verde, nada se filtra:**

```
cargo nextest run -p vantadb --lib -E 'test(fjall)' --build-jobs 2
Summary [5.387s] 22 tests run: 22 passed
AFTER: {"fjall":0,"tmp":0}
```

**RED — el mismo contenido, muriendo sin destructores:**

```
BEFORE tmp_dirs=0
FAIL  [1.874s] find732_panic_leaves_nothing        (panic a propósito, Drop corre)
ABORT [3.854s] find732_abort_leaks_the_temp_db     - 0xc0000409
AFTER tmp_dirs=1  .tmpiwIDZx  68,1 MB  lock=True
```

**GREEN — el recolector lo borra (re-medido con la versión post-review):**

```
pwsh ... -MinAgeMinutes 0        # sin suites en vuelo
      68,1 MB  .tmpiwIDZx  [DB fjall con lock libre, sin actividad hace 0 min]
recolector: 1/1 carpeta(s), 68,1 MB liberados.
AFTER_COLLECT tmp_dirs=0
```

Con el default (`-MinAgeMinutes 10`) la misma carpeta **no** se toca: es la
guarda de inactividad que cierra las ventanas vivas (review F1), y se ve en el
self-check (`recién escrita: la guarda de inactividad la protege`).

**Bonus medido en la misma corrida** — el barrido por defecto encontró residuo
real que el diagnóstico no había catalogado, de otra familia de tests:

```
      68,0 MB  C:\...\Temp\vanta-cog-ckpt-5296          [sin actividad hace 69 min]
      68,0 MB  C:\...\Temp\vanta-cog-ckpt-restore-5296  [sin actividad hace 69 min]
recolector: 2/2 carpeta(s), 136,0 MB liberados.
```

**Self-check (falla si el recolector no limpia el caso real, o si limpia lo ajeno):**

```
pwsh -NoProfile -File dev-tools/temp-collector.ps1 -SelfCheck
SELFCHECK: PASS (19/19 asserts)     # rechaza %TEMP%, C:\WINDOWS, ..\Windows, Temp2\;
                                    # acepta .fjall_compaction_tmp_*;
                                    # lock libre vs. abierto;
                                    # EL CASO REAL: `.tmp*` + lock libre → recolectada y borrada;
                                    # inactividad protege lo recién escrito;
                                    # `.tmp*` ajeno intacto; PID muerto; dir cualquiera no clasifica
```

**Gate de sintaxis de lo que se tocó** (un `verify.ps1` roto bloquea todo el repo):

```
Parser::ParseFile dev-tools/verify.ps1          -> OK
Parser::ParseFile dev-tools/temp-collector.ps1  -> OK
bash -n .githooks/pre-push                      -> exit 0
pwsh dev-tools/target-cleanup.ps1 -SelfCheck    -> SELFCHECK: PASS (16/16)
node scripts/docs/check-docs.mjs                -> exit 0
node scripts/docs/check-links.mjs               -> 0 broken markdown (1 wikilink preexistente)
```

El `tests/find732_scratch.rs` que produjo el RED **no se commitea** (aborta un
proceso a propósito); queda documentado acá y se borró tras medir.

## Review P2-01

- **Revisor:** `vanta-review`, contexto fresco (no participó de la implementación).
- **Enfoque:** seguridad del borrado (¿puede tocar una DB viva o algo ajeno?),
  validez del self-check, tolerancia a fallo de los dos call sites, veracidad de
  la doc y fidelidad de la evidencia.
- **Cómo se probó:** self-check reproducido (exit 0), `-DryRun` sobre `%TEMP%`
  real, tres experimentos propios de seguridad (junction suelto, junction
  anidado, handle con share `ReadWrite|Delete` + `Remove-Item`), `bash -n` del
  hook, `check-docs`/`check-links`, y lectura con números de línea de
  `fjall_backend.rs`, `.config/nextest.toml`, `Cargo.lock`, `target-cleanup.ps1`
  y las fuentes vendorizadas de `tempfile`/`fjall`.
- **Veredicto:** **changes-required** → corregido y re-medido en el mismo commit.

| Hallazgo | Sev. | Disposición |
|---|---|---|
| **F1** el lock era la **única** prueba de vida: hay tres ventanas con la DB viva y el lock libre (`compact()` cierra el tmp antes del swap, `open→drop→open`, `.tmp*` ajeno recién creado), y además `Remove-Item` **sí** borra a través de un handle abierto con share `ReadWrite\|Delete` | Required | **Corregido.** Segunda prueba obligatoria de abandono: `-MinAgeMinutes` (10) sobre la **última escritura de todo el árbol** (un solo recorrido calcula lock, tamaño y mtime — también mejora el coste de F12). Las tres ventanas quedan cubiertas y el self-check lo aserta (`recién escrita … la guarda de inactividad la protege`). La guarda de procesos de `target-cleanup.ps1:244-249` no servía acá: los tests no son `cargo`/`rustc` |
| **F2** el self-check **nunca** ejercitaba la rama `.tmp*` + lock libre — el 95% del residuo | Required | **Corregido.** Escenario `.tmp<hex>\data\lock` (+ `0.jnl`) con asserts de "recién escrita → protegida", "vieja → recolectada", "borrada", "fuera de la lista". Si mañana `Get-LeakKind` se rompe para `.tmp*`, el self-check cae |
| **F3** el cálculo "68,1 MB × 1.887 ≈ 15,35 GB" daba 128,5 GB | Optional | **Corregido.** El número correcto es el promedio (8,1 MB) y 68,1 MB es el techo |
| **F4** la doc decía `PASS (15/15)`; eran 16 | Optional | **Corregido** (y ahora 19/19 en los dos ficheros) |
| **F5** "the top of `verify.ps1`" (está en el paso previo a nextest) y exit code `1` mal descrito | Optional | **Corregido** en troubleshooting: "just before the `nextest` step", y un rechazo durante el barrido se reporta y se salta |
| **F6** doble invocación por push (el hook llama y `verify.ps1` vuelve a llamar) | Optional | **Aceptado con motivo**: si `verify.ps1` muere en `fmt`/`clippy` antes de su llamada, el barrido del hook es la única pasada. Es idempotente y cuesta ~0,7 s |
| **F7** los paths fijos (`vanta_schema_test`, `vantadb_ser_test_*`, `vanta_test_*`) no se reclamaban | Optional | **Corregido.** Prefijos `vanta*`/`vantadb*` clasificados (sólo **directorios**, con la misma doble guarda). En la primera corrida real encontró 136 MB de `vanta-cog-ckpt-*` de otra familia de tests |
| **F8** `docs/dev/tasks/FIND-732.md` huérfano (índice sin regenerar) | Optional | **Corregido.** `gen-index.mjs --write` en el mismo commit |
| **F9** citaba `fjall-3.1.10`; el lock resuelve 3.1.12 | Nit | **Corregido** en los tres sitios (la línea del `LOCK_FILE` es la misma) |
| **F10** step 6 ✅ y `§Review` sin dictamen antes de que existiera el review | Nit | **Corregido**: esta sección + steps sincronizados |
| **F11** un `.tmp*` vacío se borraba sin probar que fuera nuestro | Nit | **Corregido por diseño**: la guarda de inactividad lo cubre, y "vacío = 0 bytes = pérdida nula" queda en la cabecera del script |
| **F12** coste no medido con 1.887 directorios | Optional | **Mitigado**: un solo recorrido por candidato (antes eran tres); medido 0,72 s con 28 dirs. A 1.887 queda por medir |

## Deuda / límites conocidos

- `ponytail:` un `.tmp*` **no vacío sin `lock`** (kill de un test que no abre
  fjall) no se recolecta: no se puede probar que sea nuestro. Es ruido de KB, no
  de GB. Si algún día importa, basta que el helper de ese test cree su tempdir
  con prefijo `vantadb-` y añadir la regla.
- El coste del barrido con miles de candidatos no está medido (review F12): hoy
  son 0,7 s con decenas. Si un día hay miles de DBs reales, conviene probar
  layouts conocidos (`<d>\lock`, `<d>\*\lock`) antes del recorrido genérico.
- El recolector corre en `verify.ps1`/`pre-push`. Una suite lanzada a mano con
  `cargo nextest run ...` no pasa por ninguno de los dos; el residuo se reclama
  en el siguiente gate. Para forzarlo: `pwsh dev-tools/temp-collector.ps1`.
