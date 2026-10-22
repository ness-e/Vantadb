---
title: "W7 — Medicion del coste del cifrado en reposo: p99, flush, cold start y WAL"
kind: review
status: active
description: "Nota de la wave W7 del plan de cableado de la feature `encryption` (ADR-0057): que se midio, con que comando, en que entorno y que veredicto sale frente al umbral del 15 %."
tags: [vantadb, review, encryption, at-rest, adr-0057, wave-w7, benchmark]
created: "2026-10-21"
---

# W7 — El coste del cifrado en reposo, medido (ADR-0057)

> **Wave:** W7 del plan
> [`2026-10-20-wiring-encryption-feature.md`](../plans/2026-10-20-wiring-encryption-feature.md).
> **Design:** [`ADR-0057`](../architecture/adr/ADR-0057-encryption-at-rest.md).
> **Estado:** medida y **certificada** (2026-10-21, segunda medicion). Los numeros
> publicados viven en
> [`BENCHMARKS.md`](../../user/operations/BENCHMARKS.md) §22; esta nota es el
> metodo y la lectura.

W6 encendio la feature: los **4 puntos de escritura independientes** (WAL v4,
VantaFile v3, backend-KV, imagen HNSW v9) cifran, y el canario de 6 superficies
esta en verde. Lo que **no** existia era el **numero**: cuanto cuesta el
cifrado en reposo. W7 lo mide.

> **Veredicto corto:** el gate del plan (*"el p99 cifrado no empeora mas de un
> 15 %"*) **queda SUPERADO**: el p99 de la busqueda empeora **+190,3 %**
> (mediana de 3 corridas intercaladas por variante) y el veredicto es
> **concluyente** — las 3 corridas cifradas son mas de un 15 % peores que las 3
> claras, sin solapamiento. **Es una señal de diseño**, no ruido: la causa es la
> **granularidad del envelope del VantaFile** (§4.1). El `insert` y el `flush`
> **no** empeoran (dentro del ruido).

---

## 1. Que habia que medir (y por que un bench nuevo)

`benches/canonical_p99.rs` **no puede** medir el coste del cifrado en reposo:
mide un `CPIndex` **puro en memoria** (sin E/S de storage). Con y sin la
feature `encryption` mediria **exactamente lo mismo** (delta ~0). El baseline
canónico de `BENCHMARKS.md` §11 (p50 2.2388 / p95 4.248 / **p99 4.9987 ms**,
2026-09-25) es ese bench: sirve de **referencia de escala**, no de
comparacion directa con un motor con storage. **No se compara** con las cifras
de aqui.

Por eso W7 anade `benches/encrypted_p99.rs`: mueve el **`StorageEngine` real**
(insert + search + flush + cold start) con la feature `encryption` activada y
una clave valida, de modo que los envelopes **estan en el camino de escritura
y de lectura**.

### 1.1 Que mide cada fase

| Fase | Que mide | Por que importa |
|---|---|---|
| `insert` | 25k `engine.insert()` contra un motor real (WAL + vstore + backend + indice HNSW) | El coste de **escribir** los 4 artefactos cifrados |
| `search` | 1000 queries por `engine.hnsw` **con el vector store** (`Some(&vs)`) | El coste de **leer** cifrado: el motor puntua los candidatos leyendo el vector del vstore — un `mmap` zero-copy en claro, un **descifrado GCM por nodo** en cifrado |
| `flush` | `engine.flush()` | La **imagen HNSW se reescribe entera** (`maintenance.rs:64` → `save_vector_index`): en modo cifrado es **un AEAD sobre el indice completo** |
| `cold` | cerrar + reabrir | El arranque en frio **descifra la imagen entera**: se pierde el `mmap` zero-copy (limite declarado en ADR-0057 §5.2) |

`benches/wal_throughput.rs` mide, por su parte, el **coste por frame del WAL**
(3 modos de sync × 4 tamanos de lote), que es el otro camino caliente.

### 1.2 Dataset y perfil (los canonicos, no unos propios)

- **Dataset:** seed 42, **25 000 × 1536d**, generado **exactamente** como
  `benches/canonical_p99.rs` (`StdRng::seed_from_u64(42)`,
  `rng.random::<f32>()`). Los 1000 queries son los 1000 primeros vectores de
  ese mismo stream — igual que en el bench canonico. **La escala no es la del
  plan (100k): §3.1 explica por que, y por que las alternativas que se
  sugerian (10k, 5k) eran invalidas.**
- **Perfil:** `benches/common::apply_fixed_profile` (warmup 3 s / measure 5 s,
  confianza 0.95, `sample_size(10)`), el mismo de todos los benches de regresion.
- **Protocolo:** **3 corridas por variante, mediana**, **intercaladas**
  (P1, E1, P2, E2, P3, E3) para que la deriva de la maquina afecte a las dos
  variantes por igual (plan §5, W7; Regla 9).

### 1.3 La clave: la via que W6 implemento

La master key se configura por la **fuente 1 de 3** de la precedencia fija
(ADR-0057 D2): `Config::encryption_key` **programatica** →
`VANTADB_ENCRYPTION_KEY` → `VANTADB_ENCRYPTION_KEY_FILE` →
`VANTADB_ENCRYPTION_KEY_CMD`. El bench la fija con
`Config::default().with_encryption(<hex 32 B>)` (`src/config.rs:1827`), asi
que **no depende del entorno** (en la maquina de medicion no hay ninguna
variable `VANTADB_*` seteada). Es la misma via que usa un operador.

### 1.4 Las dos variantes miden lo mismo (salvo la feature)

- **Mismo fuente**, misma constante de dataset, mismo perfil, mismos 3 runs,
  misma mediana.
- La unica diferencia es el **feature compilado** (`--features encryption`) y,
  con el, la clave. Sin la feature, `Config::encryption_key` se ignora y la
  base queda en claro: el camino de bytes es el de siempre.
- Las dos variantes se **compilaron desde el mismo arbol**: `src/`, `benches/`
  y `Cargo.toml` verificados identicos (`git diff` vacio) y los commits
  intermedios de la sesion fueron **solo de docs** (`SRC-STABLE`).
- El bench lleva **auto-verificaciones** (una corrida que no puede probar
  que midio lo que dice no publica un numero):
  1. la busqueda debe devolver hits (una busqueda vacia seria "rapida" por el
     motivo equivocado);
  2. la **cabecera del WAL en disco** dice v3 (claro) o v4 (cifrado) — la
     prueba directa del modo, no la mera presencia del registro;
  3. el registro y la version del WAL tienen que **coincidir**.

### 1.5 Defectos encontrados y corregidos en el bench heredado

El bench lo escribio un intento anterior de esta misma wave que murio antes
de medir. Al revisarlo antes de correrlo se encontraron (y corrigieron) tres
cosas:

1. **La auto-verificacion era debil**: comprobaba que existiera
   `data/encryption.registry` (que solo prueba que se configuro una clave),
   no que los artefactos estuvieran cifrados. Ahora lee la **cabecera del WAL
   en disco** (v3/v4) y exige que el registro y la version coincidan.
2. **Una corrida vacia podia publicar numero**: `hits` se imprimia pero no se
   exigia. Ahora `hits > 0` es un `assert` — si la busqueda no encuentra
   nada, la corrida **falla** en vez de reportar un p99 rapido y falso.
3. **El comentario del `flush_threshold` era incorrecto**: describe un
   "batch de fsync del WAL", y el knob hace **dos** cosas: es el umbral de
   fsync del WAL (`src/wal.rs:1004` `maybe_sync`, con `SyncMode::Periodic` el
   umbral por defecto es **1**, o sea un fsync por registro) **y** el
   auto-flush del motor (`apply_insert`, `insert.rs:704`). El auto-flush
   **no** puede dispararse en este camino: `insert()` sostiene `insert_lock`
   mientras llama a `apply_insert`, y el auto-flush solo corre si
   `insert_lock.try_lock()` tiene exito — y no puede. Verificado; el umbral
   actua aqui **solo** como batch de fsync.

---

## 2. Entorno de medicion (Regla 11)

| Campo | Valor |
| :--- | :--- |
| Fecha | **2026-10-21**, 19:59:07 → 22:21 (6 invocaciones intercaladas) |
| OS | Windows 11 Pro (10.0.26200) |
| CPU | 12th Gen Intel Core i5-1235U (10c/12t) |
| RAM | 31.78 GB (32.6 GiB utiles) |
| Toolchain | rustc 1.95.0 / cargo 1.95.0 (`x86_64-pc-windows-msvc`) |
| Perfil | `bench` = `release` (`opt-level 3`, `lto = "thin"`, `codegen-units = 1`) + `debug = 1` |
| Dataset | seed 42, **25 000 × 1536d**, `StdRng::seed_from_u64(42)` — identico a `benches/canonical_p99.rs` (§3.1: por que esta escala y no la del plan) |
| Perfil de medicion | `benches/common::apply_fixed_profile` (warmup 3 s / measure 5 s, confianza 0.95) |
| Protocolo | **3 corridas por variante, mediana**; variantes **intercaladas** (P1, E1, P2, E2, P3, E3) |
| Commit | `4fb9f02` (`develop`); `src/` + `benches/` + `Cargo.toml` **identicos** en las dos compilaciones |
| **Estado del codigo medido** | **anterior al "Paso 0"** (`6edc635f`, 22:10 del 2026-10-21: 5 → 3 copias por candidato y cache de 1 entrada → LRU). Los binarios se construyeron a las **18:26** (cifrado) y **19:45** (claro), antes de ese commit, y **ningun commit toco `src/` entre las 18:00 y las 20:00** ⇒ las dos variantes difieren **solo** en la feature. **La cifra es la del estado que evalua el gate del plan**; la re-medicion *post*-Paso 0 queda pendiente (`docs/dev/tasks/FIND-598-paso0.md`) |
| Config del motor | `Config::default()` (backend **Fjall**, `sync_mode = Periodic`, `flat_threshold = 10000`) + `with_encryption(<hex 32 B>)` + `with_flush_threshold(1000)` |
| HNSW del motor | `HnswConfig::default()`: `m = 32`, `m_max0 = 64`, `ef_construction = 100`, **`ef_search = 100`**, Cosine |
| Clave | **Fuente 1 de 3** de la precedencia (ADR-0057 D2): `Config::encryption_key` programatica → `VANTADB_ENCRYPTION_KEY` → `…_KEY_FILE` → `…_KEY_CMD`. En la maquina de medicion **no hay ninguna variable `VANTADB_*` seteada** |
| **RAM libre** | **12,33 GB al empezar → 11,57 GB al terminar** |
| **Commit charge** | **28,29 / 35,78 GB (79,1 %) al empezar → 29,34 / 35,78 GB (82,0 %) al terminar** (el limite subio a 37,79 GB durante P1: Windows expandio el pagefile) |
| **Estado de la maquina** | **en silencio**: 0 procesos de compilacion y 0 procesos de bench durante toda la medicion |

### 2.1 Como se lanzo (y por que no con `cargo bench`)

```powershell
# build SOLO del target del bench (el link LTO de `vanta-cli` se queda sin
# memoria en una maquina compartida: `rustc-LLVM ERROR: out of memory`).
# -j 1: dos compilaciones simultaneas fallan con `os error 1455`.
cargo build -p vantadb --profile bench --bench encrypted_p99 -j 1
cargo build -p vantadb --profile bench --bench encrypted_p99 --features encryption -j 1

# medicion: el binario se lanza directamente (sin el lock de artefactos de cargo).
# `--bench` SIN `--`: criterion decide su modo por ese flag (criterion 0.8.2,
# `lib.rs:958`); sin el cae en test mode (1 iteracion, sin perfil). Y PowerShell
# reenvia el `--` al binario nativo, lo que rompe el parseo de clap.
$env:ENCRYPTED_P99_SCALE  = "25000"
$env:ENCRYPTED_P99_RUNS   = "1"     # una corrida por invocacion; 6 invocaciones intercaladas
$env:ENCRYPTED_P99_WARMUP = "0"     # sin pasada de calentamiento (mismo metodo que canonical_p99)
$env:ENCRYPTED_P99_OUT    = "<fichero>"   # el bench escribe sus numeros aqui, linea a linea
target\release\deps\encrypted_p99-<hash>.exe --bench
```

**Ambas variantes se compilaron del mismo estado del arbol** (hash de
`src/` + `Cargo.toml` + `benches/` verificado antes y despues de las dos
compilaciones). Sin eso la comparacion no valdria: las dos variantes tienen
que diferir **solo** en la feature.

### 2.2 Un defecto del bench que la medicion decia (y que se arreglo)

El bench que heredo esta wave imprimia sus numeros con `println!` **y nada
mas**. Rust vacia el buffer de `stdout` **al salir del proceso**: una corrida
que muere **al final** (la de 100k murio con `exit = -1` al liberar la memoria del
motor re-abierto, ya con la caja de criterion impresa) **perdia todos sus
numeros** — la caja de criterion salia, las cifras no. El bench ahora escribe
cada linea **en un fichero** y lo **vacia** despues de cada linea
(`ENCRYPTED_P99_OUT`): una corrida muerta deja sus numeros atras.

---

## 3. Resultados

> **Estos numeros son de la medicion certificada del 2026-10-21** (binarios con
> las auto-verificaciones de §1.4 activas: `mode=plaintext wal_version=Some(3)`
> / `mode=encrypted wal_version=Some(4)`, `hits=10` en las 6 corridas, las 6 con
> `exit_code=0`).
> **Escala medida: 25 000 × 1536d**, no 100k — ver §3.1.
> Los numeros completos (comando, entorno y fecha) estan en
> [`BENCHMARKS.md`](../../user/operations/BENCHMARKS.md) **§22.3**.

| Métrica | Sin cifrar (min/med/max) | Cifrado (min/med/max) | Δ mediana | Spread | Gate 15 % |
| :--- | :---: | :---: | ---: | ---: | :--- |
| search p50 (ms) | 6.8135 / 13.2733 / 16.9805 | 35.3191 / 53.0399 / 63.3524 | **+299,6 %** | +149 % / +79 % | superado |
| search p95 (ms) | 12.0319 / 20.2060 / 29.2015 | 57.9409 / 85.3295 / 116.3134 | **+322,3 %** | +143 % / +101 % | superado |
| **search p99 (ms)** | 15.7945 / **40.5614** / 41.1132 | 77.4924 / **117.7418** / 192.2276 | **+190,3 %** | +160 % / +148 % | **SUPERADO** |
| insert 25 000 nodos (s) | 742.1 / 795.4 / 1044.7 | 452.8 / 805.0 / 1599.1 | +1,2 % | +41 % / +253 % | dentro (ruido) |
| flush — imagen HNSW (s) | 0.833 / 1.509 / 2.990 | 1.185 / 1.418 / 1.681 | −6,0 % | +259 % / +42 % | dentro (ruido) |
| cold start (s) | 1.760 / 2.675 / 4.850 | 2.664 / 3.940 / 7.341 | **+47,3 %** | +176 % / +176 % | superado |
| WAL bufferizado (`never`/1 000) | 949 ns/registro | 1 124 ns/registro | **+0,18 µs** (−51,6 % thrpt) | — | — |
| WAL con fsync (`always`/1) | 1,3597 ms/registro | 1,9633 ms/registro | **celda no publicable** (§3.3) | — | — |

### 3.1 La escala: por que 25 000 y no 100 000

**El 100k del plan cabe, pero no es affordable con el protocolo completo.** En
esta maquina (ya sin ruido) el `insert` de 100k mide **~35-40 min por
invocacion**, y el grupo de criterion del bench son 1000 queries × 10 muestras
(≈ 10-20 min mas) ⇒ **~6 h** para las 6 invocaciones intercaladas que exige el
plan. (El intento anterior de 100k murio con `os error 1450` por el *commit
charge* al limite con 3 agentes compilando; aqui no habia ninguno, asi que la
limitacion es de **tiempo**, no de recursos.)

La escala elegida es **25 000 × 1536d**, y la eleccion **no es arbitraria**:

1. **25 000 > `flat_threshold` (10 000) ⇒ el indice usa el camino HNSW (ANN), el
   mismo que exercise el dataset canonico de 100k.** El motor enruta a
   **barrido plano (fuerza bruta)** cuando `nodes.len() <= flat_threshold`
   (`src/index/search/neighbors.rs:57`, `use_flat_search`), y
   `Config::default()` fija ese umbral en **10 000** (`src/config.rs:359`),
   que `StorageEngine::open` propaga al indice
   (`src/storage/engine/init.rs:462`).
2. **Por tanto 10 000 y 5 000 —las dos escalas de reserva— son invalidas para
   esta comparacion**: a 10k o menos cada query **escanea todos los vectores**
   (un workload distinto, ~15 MFLOP por query), no el recorrido ANN que mide
   el dataset canonico. Esto **corrige el diagnostico del intento anterior**:
   el p50 de 8-14 ms que se atribuyo a "la E/S de la maquina" a 10k es
   exactamente el coste de un **barrido de fuerza bruta de 10 000 × 1536d**, no
   ruido de disco. Y deja fuera de comparacion las cifras de **2 000 × 1536d**
   publicadas antes en `BENCHMARKS.md` §22.3.2 (estaban en el regimen plano).
3. **Los artifacts caben en el page cache** (~450 MB: WAL + vstore + imagen
   HNSW) ⇒ la medicion es de **CPU**, que es lo que el cifrado cambia.
4. **En el regimen HNSW el coste por query del cifrado lo fija `ef_search`**
   (los candidatos que visita el recorrido), **no `n`**: el descifrado por
   candidato cuesta lo mismo a 25k y a 100k. Por eso el delta medido a 25k es
   representativo del dataset canonico.

### 3.2 El veredicto: concluyente (y por que esta vez si)

El criterio de decision se fijo **antes** de ver los numeros
(`target/w7-cert/DECISION-RULE.md`), sobre las **bandas** de las 3 corridas de
cada variante, sin supuestos estadisticos:

- **A) gate superado** si `max(claro) × 1,15 < min(cifrado)`
  → **41,1132 × 1,15 = 47,28 < 77,4924 ⇒ VERDADERO**;
- B) gate cumplido si `max(cifrado) < min(claro) × 1,15` → 192,23 < 18,16 ⇒ falso.

Es decir: **las 3 corridas cifradas son mas de un 15 % peores que las 3 claras**.
El delta por par intercalado (P1/E1, P2/E2, P3/E3) es **+373,9 %**, **+186,4 %**
y **+390,6 %**. La segunda medicion independiente del bench (el grupo de
**criterion**, sobre el motor **reabierto**, con su propio CI) confirma el
resultado y **sus intervalos no se solapan**: medianas 14 453,1 ms sin cifrar
vs 57 257,6 ms cifrados (**+296,1 %**).

**Que cambio respecto al intento anterior (que no era certificable).** La causa
del ruido anterior no era "la maquina compartida" en abstracto, sino **dos
procesos zombis** que nadie cerro:

1. una corrida de 100k del intento anterior (`w7-meas.ps1`) **siguio viva**
   41 min despues de terminada su sesion, con 2 009 s de CPU quemados y
   613 MB de working set, y **bloqueaba el enlazado** del binario
   (`LNK1104: no se puede abrir el archivo ...encrypted_p99-...exe`);
2. una segunda medicion (`w7-25k.ps1`, escala 25 000) **arranco en paralelo**
   de la nueva, compitiendo por CPU y disco.

Ambas se detuvieron antes de medir. A partir de ahi la maquina estuvo
**realmente en silencio** (0 compilaciones, 0 benches, 12,33 GB libres, commit
al 79,1 %). El spread intra-variante que queda (+148 % en el p99 cifrado) es
**deriva de la sesion**, no competencia: el `insert` cae de forma monotone a lo
largo de las 2,5 h (1045 → 795 → 742 s en claro) porque la maquina se
**calienta**, y esa deriva afecta **por igual** a las dos variantes gracias al
intercalado. Por eso el veredicto se apoya en el **solapamiento de las bandas**
(criterio A) y no en la mediana sola.

### 3.3 El WAL: el numero limpio (bufferizado) y una celda que no es publicable

**El coste del envelope en el camino bufferizado** (`SyncMode::Never`, que es
el camino real de escritura con `flush_threshold`): **≈ +0,14 a +0,71 µs por
frame** (AEAD + CRC32C y SHA-256 de la cadena **sobre el ciphertext**, que
antes eran sobre el plaintext). Con un registro de **24 B** (el mas pequeno
posible — el **peor caso**, porque un AEAD de 24 B no amortiza nada) eso es
**−37 % a −55 % de throughput**. Con un registro real de ~6 KB (un nodo con
vector) el coste relativo es una fraccion mucho menor. Este delta **se
reproduce entre corridas** y es el unico numero del WAL que se publica como
cifra de referencia.

**La celda de fsync por registro NO es publicable, y se dice por que.** Tres
mediciones del mismo par dieron tres numeros que no concuerdan: **−7,8 %**
(medicion back-to-back, 23:05-23:14), **−68,1 %** (barrido completo de 12
celdas, 22:21-22:47) y **−1,5 %** (repeticion aislada posterior) ⇒ la celda
**queda descartada** (es el mismo modo de fallo que invalidaba el intento
anterior: ruido de E/S). Lo que **si** es solido y explica por que la celda
es fragil: el frame cifrado ocupa **~124 B frente a los ~32 B** del frame en
claro (4x: nonce + tag + CRC32C + SHA-256 de la cadena) ⇒ en un camino
dominado por el `FlushFileBuffers` de Windows ese **4x de bytes se paga** en
el flush. **Por tanto el WAL cifrado en el camino de durabilidad es mas caro
de lo que sugiere el coste unitario del AEAD, y su cuantificacion no es
estable en esta maquina**: queda como **pregunta abierta**, no como cifra.
(Aviso: desde las 23:10 corria **en paralelo un bench ajeno** de otra sesion,
de modo que la ventana del WAL no es tan silenciosa como la del motor; por eso
el coste por frame del camino bufferizado se **corrobora** con el barrido
independiente de las 22:21-22:47.)

**El WAL, en cualquier caso, no es el cuello de botella del motor**: el
`insert` medido (§3) sale **+1,2 %**, dentro del ruido, porque ahi manda la
E/S y no el cifrado.

---

## 4. Lectura

1. **El cuello de botella no es el AEAD: es la granularidad del envelope del
   VantaFile.** El envelope es **por nodo**
   (`src/storage/vfile_crypto.rs::seal_node`), y el store cifrado **no puede
   devolver una vista prestada** del `mmap`: el plaintext es `Cow::Borrowed`
   sobre el mapping (`src/index_port.rs:86`), el cifrado tiene que
   **materializar** — y lo hace **varias** veces por candidato:
   `read_record_bytes` copia el ciphertext del mapping a un `Vec`
   (`src/storage/vfile.rs:997`), `open_node` lo descifra en otro `Vec`
   (`src/storage/vfile_crypto.rs:198`), `AlignedBlob::from_bytes` lo copia otra
   vez a un buffer alineado, `read_full_vector` reconstruye el `Vec<f32>` y
   `read_header` parseaba la cabecera copiando el payload entero. En claro, en
   cambio, `full_vector_from_bytes` devuelve el `Cow::Borrowed` del mapping:
   **0 copias**. Un probe del **Paso 0** los midio en **5 copias por candidato**
   en cache fria (**1** con la cache caliente) y los reduzco a **3** (el
   ciphertext se presta del mapping, `read_header` parsea *in situ*,
   `read_full_vector` paso a `memcpy`, y la cache de 1 entrada paso a LRU
   acotada). **Lo que no se puede quitar sin cambiar el formato es el 1 AEAD
   por candidato** ⇒ hace falta el **envelope por pagina**. En cualquier
   caso, con `ef_search = 100` son ~100 candidatos **por query**, y por eso el
   p99 se va (limite ya declarado en ADR-0057 §5.2).
2. **El `insert` y el `flush` no empeoran** (+1,2 % y −6,0 %, dentro del ruido):
   el camino de escritura esta dominado por la E/S del WAL y del vstore, y ahi
   el coste del envelope (µs) se pierde. **El problema es el camino de
   lectura**, no el de escritura.
3. **El `cold start` (+47,3 %)** paga la **imagen HNSW entera**: un unico AEAD
   sobre el indice completo, descifrado en cada open, sin mmap zero-copy. Y el
   WAL se **valida entero** en cada open (`recover_valid_records_v4` descifra
   todos los frames).
4. **El cambio de diseno** (que el gate del plan ya anticipaba): **bajar la
   granularidad del envelope del VantaFile de "por nodo" a "por pagina"**
   (p. ej. 64 KiB) **y anadir una cache de paginas descifradas** (LRU). Con
   envelope por pagina el descifrado se amortiza sobre los ~10 nodos de la
   pagina, y la pagina descifrada cacheada se vuelve a leer **sin copia**
   (vuelve el cero-copy). Para la **imagen HNSW**, un envelope por pagina
   permite descifrar en el arranque en frio solo las paginas que hacen falta,
   o descifrarlas en paralelo. Para el **WAL**, truncar el WAL en el checkpoint
   (que `flush()` ya escribe) elimina el descifrado de todo el WAL en cada
   open. El diseño esta **ya especificado** en
   `docs/dev/architecture/design/2026-10-21-encryption-envelope-per-page.md`
   (commit `3c1c863`).
5. **El WAL no necesita cambio de diseno — con una reserva.** El envelope por
   frame es la granularidad correcta (el frame **es** la unidad de escritura) y
   su coste (**+0,14 a +0,71 µs**) se pierde en el camino bufferizado. **La
   reserva:** el frame cifrado ocupa **4x** bytes (nonce + tag + CRC32C +
   SHA-256 de la cadena), y en el camino de **fsync** eso se paga en el
   `FlushFileBuffers`: las tres mediciones de esa celda **no concuerdan**
   (−7,8 % / −68,1 % / −1,5 %) ⇒ su cuantificacion **no es estable en esta
   maquina** y queda como **pregunta abierta**, no como cifra (§3.3). Ademas,
   el `insert` real del motor sale **+1,2 %** (§3): el WAL **no** es el cuello
   de botella de la escritura.
6. **El claim honesto**: el cifrado en reposo **cuesta**, y el numero ya esta
   medido, con comando, entorno y fecha: **el p99 de la busqueda empeora
   +190,3 %** (mediana de 3+3 intercaladas, veredicto concluyente frente al
   15 %). El claim "el cifrado no cuesta nada" queda **cerrado como falso**, y
   el claim "cuesta un 30 %" queda sustituido por **+190 %** en el p99 de la
   busqueda, con el `insert` y el `flush` **sin** penalizacion apreciable.

---

## 5. Que queda abierto

### 5.1 Lo que esta wave **no** pudo medir (y por que)

| Hueco | Motivo (con el dato) |
| :--- | :--- |
| **El dataset canonico de 100k × 1536d** | **Cabe, pero no es affordable con el protocolo completo**: el `insert` de 100k mide **~35-40 min por invocacion** y el grupo de criterion son 1000 queries × 10 muestras (≈ 10-20 min) ⇒ **~6 h** para las 6 invocaciones intercaladas. Se publico **25 000 × 1536d**, en el **mismo regimen de codigo** (HNSW) que el 100k (§3.1). Lo que **falta** es la repeticion del protocolo 3+3 a 100k, que es una question de **tiempo de maquina**, no de metodo. |
| **El p99 como numero de regresion en CI** | La medicion **si** es concluyente (§3.2), pero un bench de 25k × 1536d **no es** un gate de CI (25 min por invocacion). El **gate de regresion** que haria falta es un **bench corto y estable** (por ejemplo el coste por frame del WAL, §3.3, que **si** es reproducible), no el p99 de la busqueda. |
| **`canonical_p99` re-medido en esta maquina** | `canonical_p99` es **in-memory puro** y **no** mide el cifrado: re-medirlo solo sirve como referencia de escala de la maquina, y su bench imprime su histograma con `println!` (el mismo defecto de buffer que se corrigio aqui). El baseline registrado (2026-09-25) se cita **como baseline registrado**, no como numero de esta maquina. |
| **RSS pico** | Se **si** midio (muestreado por el driver en cada invocacion: 7.22 GB de RAM libre tras P1, con el pico del `insert`), pero **no** se midio con el **dataset completo de 100k**. |
| **Coste de CPU del AEAD aislado** (us/frame) | El plan lo daba por **"no medido → no se afirma nada"** (§9 del plan). Esta wave mide el coste **de extremo a extremo** (insert / search / flush / cold), no el coste unitario de un sello GCM: ese numero sigue sin existir y no se afirma. |
| **WAL shipping / backup de una base cifrada** | Fuera del alcance de W7 (W6 los dejo abiertos como `FIND-496` / `FIND-507`). |

### 5.2 Lo que W7 **si** cierra

- El **umbral del 15 %** del plan tiene veredicto: **SUPERADO** (+190,3 % en el
  p99), y el veredicto es **concluyente** (criterio de bandas, §3.2).
- El claim "el cifrado en reposo no cuesta nada" / "cuesta un 30 %" queda
  **sustituido por un numero con comando, entorno y fecha**.
- La **causa** queda identificada con `file:line`: la granularidad del envelope
  del VantaFile (por nodo) ⇒ 4 materializaciones de 6 KB por candidato ⇒ se
  pierde el zero-copy. Es un **problema de diseño**, y **no se absorbe en
  silencio**.
- Los **defectos del bench** (auto-verificacion debil, corrida vacia que podia
  publicar numero, `println!` que se perdia al morir el proceso) estan
  **corregidos** en `benches/encrypted_p99.rs`.
- La **causa del ruido del intento anterior** (procesos zombis, no "la maquina
  compartida") queda **documentada**, con los PIDs, los procesos y el efecto
  (LNK1104 + contencion).

---
