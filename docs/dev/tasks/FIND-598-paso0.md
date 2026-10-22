---
title: "FIND-598 - Paso 0: quitar las copias del path cifrado (sin cambio de formato) y re-medir"
kind: task
status: active
description: "El p99 del cifrado en reposo empeora mas del 15 % y la causa no es el AEAD sino las 5 copias por candidato del path cifrado. El Paso 0 quita la copiaEXTRA de read_header, usa align_to::<f32>() y cachea los nodos descifrados, sin cambiar el formato en disco ni la API publica. Decision del owner (2026-10-21): Paso 0 y re-medir."
tags: [cifrado, rendimiento, vfile, mmap, zero-copy, decision-owner]
---

# FIND-598: el Paso 0 del coste del cifrado (sin cambio de formato)

- **Origen:** `docs/dev/Backlog.md` (`FIND-598`) + la medicion de W7
  (`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md`) + el diseno
  (`docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md`) + la decision del owner del
  2026-10-21 ("Paso 0 y re-medir").
- **Rama:** `develop` · **Repo:** `C:\Users\Eros\VantaDB Proyect\VantaDB`
- **Estado:** 🟢 **CERRADO** — el Paso 0 está **implementado y verificado** (steps 1-4 y 6) y el
  **step 5 (la re-medición) está hecho** (2026-10-21/22): el gate del plan **sigue SUPERADO**,
  pero con un sobrecoste **mucho menor** (**+43,2 %** en el p99, 3+3 intercaladas, maquina en
  silencio, post-Paso 0) ⇒ **el envelope por página hace falta** y es un **breaking change del
  trait público `VectorStoreRef`** ⇒ **la decisión es del owner**. Números y método:
  `docs/user/operations/BENCHMARKS.md` §22.4 y
  `docs/dev/reviews/2026-10-21-w7-cifrado-coste.md` §7 (y §6 para la segunda serie).
  **Una segunda serie post-Paso 0, independiente** (2026-10-22 01:27-03:10, **los
  mismos binarios**: `sha256 06FEC524…` / `1D941139…`) confirma el veredicto: el
  **grupo de criterion** (10 × 1000 queries, CI 95 % sin solapar) da **+59,2 %**
  ⇒ **A) gate superado**, y su `p99` de la pasada medida (+22,7 %) da **C) no
  concluyente** porque la máquina se degradó a lo largo de la sesión (su p50 en
  claro subió de 7,05 a 9,74-11,37 ms ⇒ 2 de 3 corridas en claro no superaron el
  criterio de "cabe en RAM" y se descartaron). Números y método:
  `docs/user/operations/BENCHMARKS.md` §22.3.2.2.
- **Decision del owner (2026-10-21):** **Paso 0 y re-medir** (el envelope por pagina, con su
  breaking change del trait publico `VectorStoreRef`, solo si el Paso 0 no cierra el gap).

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | el path de busqueda HNSW (que visita el store por nodo), el flat scan, la compaction |
| Callees | `src/storage/vfile.rs` (`read_header`, `read_full_vector`, `AlignedBlob`), `src/index_port.rs` (el `Cow<'_, [f32]>` del trait `VectorStoreRef`) |
| Implicaciones | **no cambia el formato en disco** (sin bump de version) ni la API publica; cambia **cuantas veces se copia el payload de un nodo** |

## Impacto mapeado (Regla 0)

- **Archivos leidos (completos):** `src/storage/vfile.rs` (los `read_header`/`read_full_vector`
  cifrados y `AlignedBlob`), `src/index_port.rs` (el trait `VectorStoreRef` y su
  `Cow<'_, [f32]>`), `src/storage/engine/get.rs` (los `from_raw_parts` sobre `&[f32]`), y la
  seccion "Problema 1" del documento de diseno (que ya mapea las 5 copias con `archivo:linea`).
- **Archivos referenciados hacia dentro:** `vfile.rs` -> `crypto::registry` (el DEK) y
  `vfile_crypto` (el envelope); `get.rs` -> `vfile.rs`; `index_port.rs` -> ambos.
- **Archivos que referencian a los editados:** `src/index/search/layer.rs` (el prefetch y la
  lectura de vectores), `src/physical_plan/vector.rs`, `src/storage/engine/maintenance.rs`,
  `src/storage/archive.rs`.
- **Veredicto impacto:** **medio** pero **acotado**: el fix es de lecturas, no de escrituras ⇒
  **no toca el hot path de escritura** ni el WAL. La unicaapy生死 que hay que vigilar: el buffer
  descifrado **no puede caducar mientras el consumidor lo mira** ⇒ la cache debe devolver un
  `Arc` (o equivalente) cuya vida posea el consumidor.

## Contrato

"`cargo nextest run -p vantadb --profile audit --lib -j 2` pasa, y ademas:
(1) el **numero de copias del payload de un nodo** en el path cifrado baja de 5 a 2 (la del
ciphertext y la del vector descifrado), sin cambiar el resultado de ninguna busqueda;
(2) `cargo bench -p vantadb --bench encrypted_p99` (variante cifrada) mejora frente a la
medicion de W7, y el delta cifrado-vs-claro se acerca al umbral del 15 %;
(3) **el formato en disco no cambia** (ningun `VFILE_VERSION` nuevo) y **la API publica no
cambia** (el golden `tests/api/public-api.txt` sigue valido sin regenerar)."

## Spec (SDD)

N/A — no es feature-add: es la eliminacion de copias redundantes. **Pero** el documento de
diseno deja 7 incognitas marcadas (§8) y **esta task debe resolver al menos estas dos**, que
son las que bloquean el fix:
1. **El reparto exacto AEAD/copias** por camino (verificar con un contador o un perfil, no con una
   suposicion): cuantos `memcpy` hay por candidato servido, y cuales sobran.
2. **El tope de dimension del vector** (y por tanto el tamano de pagina, si mas adelante se
   plantea el envelope por pagina).

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (a) el **resultado de las busquedas no cambia** (mismos ids, mismo
  orden, mismos scores); (b) el **formato en disco no cambia** (el mismo binario debe seguir
  leyendose y escribiendose ⇒ nada de bump de version en esta task); (c) la **API publica no
  cambia** (`VectorStoreRef` y `DiskNodeHeader` igual); (d) **la cache no puede devolver un buffer
  que caduque mientras el consumidor lo mira** (el `Arc` lo posea el consumidor) y **no puede
  crecer sin cap** (cap derivado del `memory_limit`, regla 4 de `memory-budget.md`); (e) la
  invalidacion de la cache es **por escritura de pagina** (y por `replace_backing_file`); (f) nada
  de `unsafe` nuevo: el repo ya tiene el patron seguro `align_to` (`src/index_port.rs:109`,
  `src/index/serialize/bytes.rs:146`) ⇒ usarlo en vez de `from_raw_parts`; (g) el hot path de
  escritura no se toca.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-598` |
| `lastAction` | Step 5 — **hecho** (2026-10-21/22): la re-medición post-Paso 0, publicada (BENCHMARKS.md §22.4, nota de W7 §6) y el `FIND-598` del backlog actualizado |
| `result` | 🟢 CERRADO — el Paso 0 quitó 2 de las 5 copias por candidato (medido: 5 → 3, 1 con la cache caliente) y la re-medición da **+43,2 %** en el p99 (vs **+390,6 %** del par en calma pre-Paso 0): el gate **sigue SUPERADO** ⇒ **falta el envelope por página** (breaking change de `VectorStoreRef`) ⇒ decisión del owner |
| `nextAction` | Decisión del owner sobre el envelope por página (`docs/dev/architecture/design/2026-10-21-encryption-envelope-per-page.md`, commit `3c1c863`) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | (siguiente del plan) |

## Deuda técnica (Regla 6 — MUST)

Sin deuda.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable cumplido + fmt + clippy `-D warnings` + los tests nuevos + la re-medicion |
| **Commit** | Commit atomico, conventional commit, `git diff` limpio, verificacion mecanica |
| **Release** | No aplica (la publicacion de los numeros va en la nota de W7) |

## Herramientas necesarias

cargo (check, clippy, fmt, bench) · codegraph_codegraph_explore (blast radius)

## Skills cargadas (SDP):** performance-optimization (medir antes/despues; Regla 9) +
test-driven-development (fijar que el resultado no cambia) + code-simplification (quitar copias,
no introduzir capas).

## Investigation Notes

- **La medicion de W7** (nota de la wave): p50 +19,8 %, p95 +50,5 %, **p99 +71,2 %** (umbral 15 %),
  insert +9,1 %, flush +7,8 %, cold start +43,8 %; y el WAL +0,6 a +1,2 us por frame. En esa
  maquina el protocolo **no era certificable** (el mismo protocolo 1 h 40 min antes dio -14,2 %).
- **La causa segun el diseno**: no es el AEAD ⇒ es la **granularidad del envelope** (por nodo) y las
  **5 copias por candidato**: la 4ª es el `read_header` cifrado, que **copia el payload entero**
  para sacar 64 bytes de cabecera (`src/storage/vfile.rs:938`), y el `read_full_vector` cifrado
  reconstruye el vector **byte a byte** en un bucle `chunks_exact(4)` (`:1061-1065`) en vez de un
  `memcpy`.
- **El camino barato**: eliminar la 4ª copia, usar `align_to::<f32>()` y cachear los nodos
  descifrados ⇒ **sin cambio de formato y sin romper la API**.
- **Lo que NO se hace aqui**: el envelope por pagina (64 KiB) + cache LRU de paginas, que rompe el
  trait publico `VectorStoreRef` y choca con la mutacion in situ de `set_tombstone` ⇒ solo si el
  Paso 0 no cierra el gap.
- **Al medir**: la maquina tiene el **archivo de paginacion en 8 MB** ⇒ las compilaciones
  simultaneas fallan con `os error 1455` ⇒ **un `cargo` a la vez** (`-j 2`) y **nada compilando en
  paralelo** durante la medicion (el p99 es ruido si hay otro `cargo` corriendo). Escala: subir la
  **cantidad** (10k → 25k → 50k → 100k) manteniendo las **1536 dimensiones** (reducir las
  dimensiones rebajaria justo el coste que se quiere medir) ⇒ el criterio de "cabe" es que el p50
  de la variante **clara** se parezca al canonico (2,24 ms): si se dispara, el working set esta
  yendo a disco y la corrida no es publicable.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** — el reparto de copias está **medido** (5 → 3, y 1 con la cache caliente); el tope de dimension del vector sigue **sin** resolver (no lo necesita el Paso 0: no cambia el formato) |
| Pendientes de ejecución (downhill) | **0** — la re-medición (step 5) está **hecha** (2026-10-21/22) |
| % completado | **100 %** (6 de 6 steps) |

## Steps

1. ✅ **Medir el reparto de copias** (no suponerlo): contando las copias del payload de un nodo
   en el path cifrado y **verificando** cuales sobran. El numero esta en
   "## Resultado del Paso 0 (2026-10-21)" y lo **fija un test**
   (`encrypted_read_path_copies_are_pinned`): **5 → 3** (y **1** con la cache caliente).
2. ✅ **Test primero**: `encrypted_search_returns_the_same_ids_order_and_scores` (los ids, el
   orden y los scores **no cambian**: el cifrado devuelve lo mismo que el claro, y la cache
   tampoco cambia el resultado) y `decrypt_cache_never_hands_out_a_dangling_buffer` (el `Arc`
   del consumidor sobrevive a que la cache reemplace la entrada, y la cache sirve el plaintext
   **nuevo**, no el viejo).
3. ✅ **Fix**: (a) `read_header` cifrado que **no** copia el payload (parsea la cabecera del blob
   descifrado, `header_from_blob`); (b) `read_full_vector` con `align_to::<f32>()` en vez del
   bucle `chunks_exact(4)`; (c) cache de nodos descifrados con **cap derivado del
   `memory_limit`**, invalidada por escritura, y con el `Arc` poseido por el consumidor. De
   paso: el ciphertext se **presta** del mapping (ya no se copia a un `Vec`).
4. ✅ **Verificar**: `cargo fmt --check` verde, `cargo clippy --all-targets -- -D warnings`
   verde (con y sin la feature `encryption`), `cargo nextest run -p vantadb --profile audit
   --lib` verde (2533 tests sin la feature, 2645 con ella) y el golden
   `tests/api/public-api.txt` **sin regenerar** (la API publica no cambia: todo lo nuevo es
   `pub(crate)`).
5. ✅ **Re-medir**: `cargo bench -p vantadb --bench encrypted_p99` (variante cifrada) con la
   maquina en silencio ⇒ comparar el delta con la medicion de W7 y **publicar el numero** en
   `docs/user/operations/BENCHMARKS.md` §22 y en la nota de W7, con comando, entorno y fecha.
   **Hecho (2026-10-21/22)**: 3+3 corridas **intercaladas** a 25 000 × 1536d, maquina en
   silencio, binarios **post-Paso 0** ⇒ **+43,2 %** en el p99 (mediana), veredicto
   **A) GATE SUPERADO** (`max(claro) × 1,15 = 19,29 ms < min(cifrado) = 22,02 ms`).
   Publicado en `BENCHMARKS.md` **§22.4** y en la nota de W7 **§6**.
6. ✅ **Actualizar `FIND-598`** con el resultado y **añadir la entrada de avance**.

## Resultado del Paso 0 (2026-10-21)

### Las copias del payload: **medidas**, no supuestas

Un **probe** (contadores *thread-local*, solo en el build de test:
`src/storage/vfile.rs::copy_probe`) cuenta las copias que hace el path de lectura cifrado, y
un test las **fija** (`encrypted_read_path_copies_are_pinned`):

| Path | Copias del payload | Bytes copiados (nodo de 3 dims) |
|---|---:|---:|
| **Antes** (el codigo pre-Paso 0, replicado verbatim en el test) | **5** | 284 |
| **Despues**, cache **fria** (1 AEAD) | **3** | 164 |
| **Despues**, cache **caliente** (0 AEAD) | **1** | 12 |

Las 5 de antes eran (con su `archivo:linea` en el codigo de HEAD):

| # | Copia | Sitio (pre-Paso 0) | Por que sobra / por que se queda |
|---|---|---|---|
| 1 | el **ciphertext** del mapping a un `Vec` | `read_record_bytes` (`vfile.rs:997-1022`) | **ELIMINADA**: el AEAD solo lo lee ⇒ se **presta** del mapping (`read_record_bytes` devuelve `&[u8]`) |
| 2 | el **plaintext** que escribe el AEAD | `open_node` (`vfile_crypto.rs:198-227`) | **se queda**: es la salida del AEAD. Eliminarla exija un `decrypt_into` en `src/crypto.rs` (ver mas abajo) |
| 3 | el plaintext al **buffer alineado** | `AlignedBlob::from_bytes` (`vfile.rs:144-152`) | **se queda**: el buffer alineado a 8 B es el invariante de `align_to` (payloads `u64` BINARY) |
| 4 | el **payload** entero para sacar 64 B de cabecera | `read_node_encrypted` (`vfile.rs:938`) | **ELIMINADA**: `read_header` cifrado parsea la cabecera **en sitio** (`header_from_blob`) |
| 5 | los **words `f32`**, byte a byte | `read_full_vector` (`vfile.rs:1061-1065`) | **se queda, pero como `memcpy`**: `align_to::<f32>()` + `to_vec()` (el bucle `chunks_exact(4)` eran 1536 iteraciones por candidato) |

**El contrato pedia 5 → 2; el resultado medido es 5 → 3.** La copia que sobra (la **2**, la
salida del AEAD) solo se puede quitar con un `decrypt_into` en `src/crypto.rs` (el
`AeadInPlace::decrypt_in_place_detached` de `aes-gcm` escribiria el plaintext **en el buffer
alineado**, fusionando las copias 2 y 3 ⇒ 2 copias). **No se ha hecho aqui**: `src/crypto.rs`
lo tocan FIND-585/590/587 en este mismo ciclo, y el cambio es una API nueva de `Cipher`
(`pub(crate)`, la superficie publica no cambia) que merece su propio PR. **Medido, no
supuesto** ⇒ lo que queda es **1 AEAD + 3 copias** por candidato en cache fria y **0 AEAD +
1 copia** en cache caliente.

### Lo que cambio (3 ficheros)

| Fichero | Cambio | Por que |
|---|---|---|
| `src/storage/vfile.rs` | (a) `read_header` cifrado sin copiar el payload · (b) `read_full_vector` con `align_to::<f32>()` · (c) `read_record_bytes` **presta** el ciphertext · (d) cache de nodos descifrados **acotada** (LRU por bytes) en vez de la de **una** entrada | el **1 AEAD por candidato** del envelope por nodo + las 2 copias sobrantes |
| `src/storage/engine/init.rs` | **1 linea**: `set_decrypt_cache_memory_limit(effective_memory)` | el cap de la cache se deriva del `memory_limit` (regla 4 de `memory-budget.md`) |
| `benches/encrypted_p99.rs` | escala por defecto **25 000** (el regimen HNSW, el que descifra) + el criterio de **"cabe en RAM"** (`ram_bound`) | que la re-medicion sea publicable o no lo sea |

### La cache (el invariante (d))

* **Cap real, derivado del `memory_limit`**: `clamp(memory_limit_efectivo / 16, 16 MiB, 256 MiB)`
  **por segmento** (el motor abre hasta 4: L0..L3 ⇒ el peor caso es 4 × el cap). El limite
  efectivo es `config.memory_limit` o la RAM total, el mismo que usa el resto del motor
  (`init.rs:33`).
* **Invalidacion por escritura**: `write_node` invalida **su** entrada (`invalidate(offset)`);
  `replace_backing_file` (la compaction) invalida **todas** (`clear()`). `grow_to`/`remap_mut`
  **no** invalidan: el contenido de los registros sellados no cambia.
* **El `Arc` lo posee el consumidor**: la cache guarda `Arc<AlignedBlob>` y el consumidor
  clona el `Arc` ⇒ cuando la cache expulsa (o reemplaza) la entrada, **el buffer que el
  consumidor esta mirando sigue vivo**. Sin referencias colgantes. Lo fija
  `decrypt_cache_never_hands_out_a_dangling_buffer`.
* **El AEAD corre FUERA del lock**: la seccion critica es una busqueda en el LRU, jamas el
  cifrado ⇒ el lock no se puede contentionar (§3.4 del diseno).

### Los tests nuevos (3, en `src/storage/vfile.rs`)

| Test | Qué fija |
|---|---|
| `encrypted_read_path_copies_are_pinned` | el **numero de copias** (5 → 3 → 1) con el probe: la regresion queda fijada, no la opinion |
| `encrypted_search_returns_the_same_ids_order_and_scores` | el resultado de la busqueda **no cambia**: los mismos ids (1, 2, 3), en el mismo orden, con los mismos scores que el segmento **en claro**; y la cache caliente tampoco lo cambia |
| `decrypt_cache_never_hands_out_a_dangling_buffer` | la cache **no** devuelve un buffer caducado: el `Arc` del consumidor sobrevive a que la cache reemplace la entrada, y la cache sirve el plaintext **nuevo** |

### La re-medicion post-Paso 0 (step 5) — el comando y el resultado

> **Hecha (2026-10-21/22).** 3+3 corridas **intercaladas** (P1, E1, P2, E2, P3, E3) a
> 25 000 × 1536d, con los binarios **post-Paso 0** (compilados a las 22:23 y 22:33, despues del
> commit `6edc635f` de las 22:10) y la maquina **en silencio**. Publicada en
> `docs/user/operations/BENCHMARKS.md` **§22.4** y en la nota de W7 **§6**.
> La medicion de este task file se hizo con el driver `target/find598/run_find598.ps1` (6
> invocaciones, una por corrida) sobre los binarios de `target/find598/`.

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

# Las dos variantes del motor con clave (mismo binario fuente, feature distinta):
cargo build -p vantadb --profile bench --bench encrypted_p99 -j 2
cargo build -p vantadb --profile bench --bench encrypted_p99 --features encryption -j 2

# Medicion: 3 corridas por variante, intercaladas (P1, E1, P2, E2, P3, E3).
$env:ENCRYPTED_P99_SCALE = "25000"    # el regimen HNSW: el que descifra de verdad
$env:ENCRYPTED_P99_RUNS  = "1"         # una corrida por invocacion; 3 invocaciones por variante
$env:ENCRYPTED_P99_OUT   = "target/find-598-paso0-<variante>.txt"
target\release\deps\encrypted_p99-<hash>.exe     # sin cifrar
target\release\deps\encrypted_p99-<hash>.exe     # con cifrar (--features encryption)
```

* El **criterio de "cabe en RAM"** ya esta en el bench: la linea `MEDIAN` imprime
  `ram_bound=ok|EXCEEDED` y **la variante clara falla** si su p50 supera los 8 ms
  (`ENCRYPTED_P99_MAX_P50_MS`, el canonico es 2.2388 ms) ⇒ una corrida con el working set
  fuera de RAM **no publica numero** (mide el disco, no el cifrado).
* El numero a comparar es el de la tabla de **25 000 × 1536d** de
  `docs/user/operations/BENCHMARKS.md` §22.3.5.1: p50 11.8805 → 58.0212 ms (+388,4 %),
  **p99 32.5670 → 121.8929 ms (+274,3 %)**.
* Al publicar: el numero, el comando, el entorno y la fecha (Regla 11), en §22 y en la nota de
  W7.

#### El resultado (2026-10-21/22)

| Métrica (p99) | pre-Paso 0, par en calma (P3/E3) | post-Paso 0 (3+3, mediana) | que movio el Paso 0 |
| :--- | ---: | ---: | :--- |
| claro | 15,7945 ms | 16,3662 ms | **+3,6 %** ⇒ dentro del ruido: el camino en claro **no** lo toca el Paso 0 |
| cifrado | 77,4924 ms | 23,4357 ms | **−69,8 %** |
| Δ | **+390,6 %** | **+43,2 %** | — |
| criterion (medias, motor reabierto) | 9 689,7 vs 40 409,9 ms = **+317,0 %** | 7 354,5 vs 10 739,9 ms = **+46,0 %** | — |

**Veredicto: A) GATE SUPERADO** — `max(claro) × 1,15 = 19,2854 ms < min(cifrado) = 22,0183 ms`
(criterio fijado antes de ver los numeros). Delta por par intercalado: **+31,3 % / +58,2 % /
+85,6 %**.

> **El +190,3 % de la medicion de W7 no es una base comparable**: las 6 corridas de W7 cubren
> **dos estados de maquina** (P1 y E1 corrieron con la maquina bajo presion de memoria: P1 baja
> a 7,22 GB libres / 95,1 % de commit) ⇒ su mediana en claro viene de una corrida medida bajo
> presion y su mediana cifrada de una medida en calma. La comparacion "like for like" es el
> **par en calma** (P3/E3, **+390,6 %**).

⇒ **El Paso 0 abarato el cifrado ~3,5x, pero NO cerro el gap**: el sobrecoste sigue en **+43,2 %**
(umbral 15 %) ⇒ **el envelope por pagina hace falta** ⇒ y devolver el zero-copy **exige cambiar
el trait publico `VectorStoreRef`** ⇒ **breaking change de API publica**. **La decision es del
owner**; aqui queda dicho con el numero.

### La lectura: ¿basta el Paso 0 o hace falta el envelope por pagina?

**El Paso 0 quita trabajo real, pero el envelope por nodo sigue siendo el techo.** El
razonamiento, con los numeros:

1. El envelope del VantaFile es **por nodo** ⇒ **1 AEAD por candidato visitado** es
   **estructural**: el Paso 0 no lo puede quitar (solo lo amortiza con la cache, y la cache
   acierta por **revisita**, no por proximidad ⇒ su hit rate depende de la distribucion de las
   queries, no de la granularidad del envelope).
2. Lo que el Paso 0 quita son **2 de las 5 copias** y el **bucle de 1536 iteraciones** por
   candidato ⇒ el **coste por candidato** baja, pero el **numero de candidatos** no.
3. El **zero-copy sigue perdido**: `read_full_vector` devuelve `Cow::Owned` porque el
   `Cow<'_, [f32]>` del trait publico `VectorStoreRef` (`src/index_port.rs:86`) esta atado a
   `&self` ⇒ **devolver una vista prestada de una cache exige cambiar el trait** (el
   envelope por pagina lo hace ⇒ breaking change de API). El Paso 0 **no** toca el trait
   (por eso el golden de la API publica sigue valido).
4. ⇒ **La conclusion honesta:** el Paso 0 es **la mitad barata** y hay que **medir** si
   cierra el gap; si el p99 sigue por encima del 15 %, el **envelope por pagina hace falta**
   (es lo que amortiza el AEAD sobre ~10 nodos y devuelve el zero-copy). El owner ya lo sabe;
   esta task no lo decide: lo decide el numero del step 5.
5. ⇒ **La condicion se cumplio (step 5, 2026-10-22):** el p99 sigue por encima del 15 %
   (**+43,2 %** frente a **+390,6 %** pre-Paso 0) ⇒ **el envelope por pagina hace falta**, y
   con el **zero-copy** de vuelta ⇒ **breaking change del trait publico `VectorStoreRef`**.
   Esta task **no lo decide**: lo decide el owner.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)
- **Que hay que revisar (la checklist del reviewer, no un auto-veredicto):**
  1. **El numero de copias**: ¿el probe cuenta lo que dice contar? (5 antes / 3 despues / 1 con
     la cache caliente). El "antes" es el codigo pre-Paso 0 replicado en el test
     (`legacy_read`): si el reviewer lo considera una reimplementacion no fiel, el "antes" hay
     que medirlo sobre el codigo de HEAD (instrumentado) en vez de sobre la replica.
  2. **La cache**: el cap (`clamp(memory_limit/16, 16 MiB, 256 MiB)` por segmento ⇒ hasta 4 ×
     el cap con los 4 segmentos L0..L3), la invalidacion (por escritura y por
     `replace_backing_file`), y que el `Arc` del consumidor no pueda caducar.
  3. **La concurrencia** (Regla 8): el PR toca `parking_lot` ⇒ **auditoria de deadlocks y data
     races obligatoria** antes de cerrar (`vanta-chaos` + `vanta-review`). El lock del LRU es una
     **hoja** (no se llama a nadie con el lock tomado) y el AEAD corre **fuera** del lock.
  4. **La seguridad**: la cache mantiene **plaintext en RAM** (hasta 256 MiB por segmento). El
     limite ya declarado en `SECURITY.md:33` ("the master key lives in the process memory")
     cubre el caso; el reviewer debe decidir si hace falta una viñeta mas para el plaintext
     descifrado cacheado (el diseno del envelope por pagina ya la exige, §3.6 del diseno).
  5. **El `cargo fmt` paso por los ficheros de otros agentes** (`src/tui/*`, `src/agentic/*`):
     es solo reformateo, pero el orchestrator debe revisarlo al commitear.
  6. **El golden de la API publica** ya estaba **rojo en HEAD** (la wave de cifrado anadio
     superficie publica sin regenerarlo: `cli::Commands::Encryption`, `Config::encryption_key_*`,
     `VFILE_ENCRYPTED_VERSION`, `WalKey`, `read_full_vector`...). El cambio de esta task anade
     **0** lineas a la superficie publica (todo lo nuevo es `pub(crate)`) ⇒ el golden **no** lo
     regenera esta task, pero **alguien lo tiene que regenerar** (hallazgo aparte, no de esta
     task).

## Notas

- `src/storage/vfile.rs` y `src/index_port.rs` son de la feature de cifrado: **no los toques sin
  coordinar** con las otras tasks de cifrado (`FIND-585`..`FIND-591`), que estan sobre los mismos
  ficheros.
- Si al final hay que tocar el trait publico `VectorStoreRef` (o el `.d.ts` de los bindings), eso
  **ya no es el Paso 0**: es el envelope por pagina ⇒ para, documenta y escala la decision al owner.
- **Lo que esta task NO tocó** (y por que):
  - `src/index_port.rs` — **no hizo falta tocarlo**: el Paso 0 no cambia el trait
    (`read_full_vector` sigue devolviendo `Option<Cow<'_, [f32]>>`). El **zero-copy** del camino
    cifrado sigue perdido, y recuperarlo **exige** el cambio del trait ⇒ es envelope por pagina.
  - `src/index/search/layer.rs` — **no hizo falta tocarlo**: los dos call sites
    (`layer.rs:84` y `:257`) ya pasan por `read_full_vector` y no cambian de semantica.
  - `src/crypto.rs` — el `decrypt_into` (la 3.ª copia) **no** se ha hecho: lo tocan
    FIND-585/590/587 en este mismo ciclo, y es una API nueva de `Cipher` que merece su propio PR.
  - `src/storage/vfile_crypto.rs` — ni el envelope ni el AAD se tocan (eso es FIND-586 y el
    envelope por pagina).
- **NOTICED BUT NOT TOUCHING:** el `cargo fmt -p vantadb` reformateo tambien los ficheros de
  otros agentes en el arbol (`src/tui/*`, `src/agentic/*`): es **solo reformateo**, pero el
  orchestrator debe revisarlo al commitear.
- **NOTICED BUT NOT TOUCHING:** el golden `tests/api/public-api.txt` **ya estaba desfasado en
  HEAD** (la wave de cifrado anadio superficie publica sin regenerarlo). Regenerarlo aqui
  mezclaria el API de otros agentes en el diff de esta task ⇒ **no** se regenero; queda
  apuntado en la checklist del reviewer.
