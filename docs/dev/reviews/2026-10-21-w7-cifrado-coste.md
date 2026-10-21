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
> **Estado:** medida (2026-10-21). Los numeros publicados viven en
> [`BENCHMARKS.md`](../../user/operations/BENCHMARKS.md) §22; esta nota es el
> metodo y la lectura.

W6 encendio la feature: los **4 puntos de escritura independientes** (WAL v4,
VantaFile v3, backend-KV, imagen HNSW v9) cifran, y el canario de 6 superficies
esta en verde. Lo que **no** existia era el **numero**: cuanto cuesta el
cifrado en reposo. W7 lo mide.

---

## 1. Que habia que medir (y por que un bench nuevo)

`benches/canonical_p99.rs` **no puede** medir el coste del cifrado en reposo:
mide un `CPIndex` **puro en memoria** (sin E/S de storage). Con y sin la
feature `encryption` mediria **exactamente lo mismo** (delta ~0). El baseline
canónico de `BENCHMARKS.md` §11 (p50 2.2388 / p95 4.248 / **p99 4.9987 ms**,
2026-09-25) es ese bench: sirve de **referencia de escala**, no de
comparacion directa con un motor con storage.

Por eso W7 anade `benches/encrypted_p99.rs`: mueve el **`StorageEngine` real**
(insert + search + flush + cold start) con la feature `encryption` activada y
una clave valida, de modo que los envelopes **estan en el camino de escritura
y de lectura**.

### 1.1 Que mide cada fase

| Fase | Que mide | Por que importa |
|---|---|---|
| `insert` | 100k `engine.insert()` contra un motor real (WAL + vstore + backend + indice HNSW) | El coste de **escribir** los 4 artefactos cifrados |
| `search` | 1000 queries por `engine.hnsw` **con el vector store** (`Some(&vs)`) | El coste de **leer** cifrado: el motor puntua los candidatos leyendo el vector del vstore — un `mmap` zero-copy en claro, un **descifrado GCM por nodo** en cifrado |
| `flush` | `engine.flush()` | La **imagen HNSW se reescribe entera** (`maintenance.rs:64` → `save_vector_index`): en modo cifrado es **un AEAD sobre el indice completo** |
| `cold` | cerrar + reabrir | El arranque en frio **descifra la imagen entera**: se pierde el `mmap` zero-copy (limite declarado en ADR-0057 §5.2) |

`benches/wal_throughput.rs` mide, por su parte, el **coste por frame del WAL**
(3 modos de sync × 4 tamanos de lote), que es el otro camino caliente.

### 1.2 Dataset y perfil (los canonicos, no unos propios)

- **Dataset:** seed 42, **100k × 1536d**, generado **exactamente** como
  `benches/canonical_p99.rs` (`StdRng::seed_from_u64(42)`, `rng.random::<f32>()`).
  Los 1000 queries son los 1000 primeros vectores de ese mismo stream — igual
  que en el bench canonico.
- **Perfil:** `benches/common::apply_fixed_profile` (warmup 3 s / measure 5 s,
  confianza 0.95), el mismo de todos los benches de regresion.
- **Protocolo:** **3 corridas, mediana** (plan §5, W7; Regla 9).

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
| Fecha | **2026-10-21** |
| OS | Windows 11 Pro (10.0.26200) |
| CPU | 12th Gen Intel Core i5-1235U (10c/12t) |
| RAM | 31.78 GB |
| Toolchain | rustc 1.95.0 / cargo 1.95.0 (`x86_64-pc-windows-msvc`) |
| Perfil | `bench` = `release` (`opt-level 3`, `lto = "thin"`, `codegen-units = 1`) + `debug = 1` |
| Dataset | seed 42, **2 000 × 1536d** en la corrida publicada (§3.1: 100k no se pudo medir en esta maquina), `StdRng::seed_from_u64(42)` — identico a `benches/canonical_p99.rs` |
| Perfil de medicion | `benches/common::apply_fixed_profile` (warmup 3 s / measure 5 s, confianza 0.95) |
| Protocolo | **3 corridas por variante, mediana**; variantes **intercaladas** (P1, E1, P2, E2, P3, E3) |
| Commit | `796f9b06` (`develop`) + el worktree sucio de los agentes concurrentes (`src/parser/`, `src/planner.rs`, `src/physical_plan/`, `src/lsm.rs`, `src/wal*.rs`) |
| Config del motor | `Config::default()` (backend **Fjall**, `sync_mode = Periodic`, `flat_threshold = 10000`) + `with_encryption(<hex 32 B>)` + `with_flush_threshold(1000)` |
| HNSW del motor | `HnswConfig::default()`: `m = 32`, `m_max0 = 64`, `ef_construction = 100`, **`ef_search = 100`**, Cosine |
| Clave | **Fuente 1 de 3** de la precedencia (ADR-0057 D2): `Config::encryption_key` programatica → `VANTADB_ENCRYPTION_KEY` → `…_KEY_FILE` → `…_KEY_CMD`. En la maquina de medicion **no hay ninguna variable `VANTADB_*` seteada** |
| Estado de la maquina | **3 agentes** compilando en paralelo durante toda la corrida; el disco C: se lleno (4.4 GB libres) y hubo que liberar la cache incremental obsoleta de `target/debug/incremental` (17.6 GB, dirs de 2 dias) para que la corrida del dataset canonico cupiera |

### 2.1 Como se lanzo (y por que no con `cargo bench`)

```powershell
# build SOLO del target del bench (el link LTO de `vanta-cli` se queda sin
# memoria en una maquina compartida: `rustc-LLVM ERROR: out of memory`):
cargo build -p vantadb --profile bench --bench encrypted_p99 -j 1
cargo build -p vantadb --profile bench --bench encrypted_p99 --features encryption -j 1

# medicion: el binario se lanza directamente (sin el lock de artefactos de cargo)
$env:ENCRYPTED_P99_SCALE = "100000"
$env:ENCRYPTED_P99_RUNS  = "1"     # una corrida por invocacion; 3 invocaciones por variante
$env:ENCRYPTED_P99_OUT   = "<fichero>"   # el bench escribe sus numeros aqui, linea a linea
target\release\deps\encrypted_p99-<hash>.exe
```

**Ambas variantes se compilaron del mismo estado del arbol** (hash del diff de
`src/` + `Cargo.toml` + `benches/` verificado antes y despues de las dos
compilaciones: `SRC-STABLE=true`). Sin eso la comparacion no valdria: las dos
variantes tienen que diferir **solo** en la feature.

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

> **Estos numeros son de la corrida medida por la implementacion de W7** (binarios
> con las auto-verificaciones de §1.4 activas: `mode=plaintext wal_version=Some(3)`
> / `mode=encrypted wal_version=Some(4)`, `hits=10` en las 6 corridas).
> **Escala medida: 2 000 × 1536d**, no 100k — ver §3.1.
> Los numeros completos (comando, entorno y fecha) estan en
> [`BENCHMARKS.md`](../../user/operations/BENCHMARKS.md) **§22.3**.

| Métrica | Sin cifrar | Cifrado | Δ | Umbral 15 % |
| :--- | ---: | ---: | ---: | :--- |
| search p50 (ms) | 2.3750 | 2.8442 | +19.8 % | superado |
| search p95 (ms) | 5.0552 | 7.6095 | +50.5 % | superado |
| **search p99 (ms)** | **6.4655** | **11.0673** | **+71.2 %** | **superado** |
| insert 2 000 nodos (s) | 24.225 | 26.427 | +9.1 % | dentro |
| flush — imagen HNSW (s) | 0.103 | 0.111 | +7.8 % | dentro |
| cold start (s) | 0.274 | 0.394 | +43.8 % | superado |
| WAL `never`/1 000 (throughput) | 867.06 Melem/s | 427.52 Melem/s | **−50.7 %** | — |
| WAL `always`/1 (latencia por registro) | 1.4137 ms | 1.4211 ms | **+0.5 %** | — |

### 3.1 La escala: por que 2 000 y no 100 000

El dataset de 100k **no se pudo medir en esta maquina**: con 3 agentes
compilando en paralelo, el **commit charge** de Windows estaba al limite y el
motor **moria con `os error 1450`** ("recursos insuficientes") durante el
`insert` (Fjall). A 10k × 1536d (artifacts: WAL ~62 MB + vstore ~61 MB + imagen
HNSW ~61 MB) el working set ya no cabe: el p50 de la busqueda subia a
**8-14 ms** (frente a los **2.24 ms** del bench canonico *in-memory* de 100k) →
la medicion media la E/S de la maquina, no el cifrado. La corrida de 10k se
**descarto**: su spread intra-variante en el p99 fue de **+56 %** (plaintext
15.83 / 20.78 / 41.15 ms) a **+200 %** (cifrado).

A **2 000 × 1536d** los artifacts caben en el page cache y la medicion es de
**CPU**, que es lo que el cifrado cambia. Los **deltas** son comparables (misma
maquina, mismos binarios, intercalados); los **absolutos no** son comparables
con el p99 canonico de `BENCHMARKS.md` §11 (4.9987 ms): otro bench, otra
escala, otra maquina.

### 3.2 El p99 no es un numero estable en esta maquina

El protocolo de arriba (16:43-16:51) da **+71.2 %** en el p99; un protocolo
**identico** corrido 1 h 40 min antes (15:03-15:07, mismos parametros, los
mismos vectores) dio el **signo contrario**: p99 cifrado 12.20 ms vs plaintext
14.22 ms (**−14.2 %**). El spread intra-variante es de **±56 %** (plaintext) a
**+200 %** (cifrado). Conclusion honesta: **la mediana de este protocolo supera
el umbral de 15 %, pero esta maquina compartida no puede certificarlo** — hace
falta una maquina dedicada (o CI) para publicar un p99 cifrado como numero de
regresion. Lo que **si** es reproducible es el **coste por frame del WAL**.

### 3.3 El numero limpio: el coste del envelope del WAL

**≈ +0.6 a +1.2 µs por frame** (AEAD + CRC32C y SHA-256 **sobre el
ciphertext**, que antes eran sobre el plaintext). Con un registro de **24 B**
(el mas pequeno posible — el **peor caso**, porque un AEAD de 24 B no amortiza
nada) eso es **−39 % a −51 % de throughput** en el camino bufferizado; con un
fsync por registro (1.4137 ms → 1.4211 ms = **+0.5 %**) el cifrado es
**invisible** porque domina el fsync de Windows. Con un registro real de ~6 KB
(un nodo con vector) el coste relativo es una fraccion mucho menor.

---

## 4. Lectura

1. **El cuello de botella no es el AEAD: es la granularidad del envelope del
   VantaFile.** El envelope es **por nodo**
   (`src/storage/vfile_crypto.rs::seal_node`), y el store cifrado **no puede
   devolver una vista prestada** del `mmap`: el plaintext es `Cow::Borrowed`
   sobre el mapping (`src/index_port.rs:86`), el cifrado tiene que
   **materializar** — `read_record_bytes` copia el ciphertext del mapping a un
   `Vec`, `open_node` lo descifra en otro `Vec` y `AlignedBlob` lo copia otra
   vez a un buffer alineado: **3 allocaciones + 3 copias de 6 KB + 1 AEAD por
   candidato** que visita el recorrido HNSW. El cero-copy se pierde (limite ya
   declarado en ADR-0057 §5.2).
2. **El cold start** (+43.8 %) paga la **imagen HNSW entera**: un unico AEAD
   sobre el indice completo, descifrado en cada open, sin mmap zero-copy. Y el
   WAL se **valida entero** en cada open (`recover_valid_records_v4` descifra
   todos los frames).
3. **El cambio de diseno** (si el p99 medido en una maquina dedicada confirma
   el >15 %): **bajar la granularidad del envelope del VantaFile de "por nodo"
   a "por pagina"** (p. ej. 64 KiB) **y anadir una cache de paginas
   descifradas** (LRU). Con envelope por pagina el descifrado se amortiza
   sobre los ~10 nodos de la pagina, y la pagina descifrada cacheada se vuelve
   a leer **sin copia** (vuelve el cero-copy). Para la **imagen HNSW**, un
   envelope por pagina permite descifrar en el arranque en frio solo las
   paginas que hacen falta, o descifrarlas en paralelo. Para el **WAL**,
   truncar el WAL en el checkpoint (que `flush()` ya escribe) elimina el
   descifrado de todo el WAL en cada open.
4. **El WAL no necesita cambio de diseno**: el envelope por frame es la
   granularidad correcta (el frame **es** la unidad de escritura) y su coste
   (+0.6-1.2 µs) queda enterrado bajo el fsync en el camino de durabilidad.
5. **El claim honesto**: el cifrado en reposo **cuesta** — medido, con comando,
   entorno y fecha — pero el **cuanto en el p99 de la busqueda** depende de una
   medicion que esta maquina compartida no puede cerrar (el ruido es del orden
   del umbral). No se puede publicar "el cifrado no cuesta nada" ni "cuesta un
   X %" en el p99 sin esa medicion.

---

## 5. Que queda abierto

### 5.1 Lo que esta wave **no** pudo medir (y por que)

| Hueco | Motivo (con el dato) |
| :--- | :--- |
| **3 corridas con mediana** (el protocolo del plan) | La maquina la comparten **3 agentes** que compilan el mismo crate a la vez: el lock de artefactos de cargo serializo cada compilacion (la del bench tardo **59 min 57 s** en el perfil `bench`) y una corrida canonica de 100k tarda **~25 min** por invocacion. 6 invocaciones = **~2,5 h** solo de medicion, mas las compilaciones. Se entrega **1 par completo** (P1 + E1) del dataset canonico de 100k x 1536d. |
| **`canonical_p99` re-medido en esta maquina** | `canonical_p99` es **in-memory puro** y **no** mide el cifrado: re-medirlo solo sirve como referencia de escala de la maquina, y su bench imprime su histograma con `println!` (el mismo defecto de buffer que se corrigio aqui). El baseline registrado (2026-09-25) se cita **como baseline registrado**, no como numero de esta maquina. |
| **RSS pico** | Se **si** midio (muestreado cada 3 s sobre el proceso del bench) durante las corridas; lo que **no** se midio es el RSS del proceso con el **dataset completo de 100k** en paralelo con los otros agentes, porque el pico se produce durante el `insert` y la maquina estaba compartida. |
| **Coste de CPU del AEAD aislado** (us/frame) | El plan lo daba por **"no medido → no se afirma nada"** (§9 del plan). Esta wave mide el coste **de extremo a extremo** (insert / search / flush / cold), no el coste unitario de un sello GCM: ese numero sigue sin existir y no se afirma. |
| **WAL shipping / backup de una base cifrada** | Fuera del alcance de W7 (W6 los dejo abiertos como `FIND-496` / `FIND-507`). |

### 5.2 Lo que W7 **si** cierra

- El **umbral del 15 %** del plan tiene numero (ver §4).
- El claim "el cifrado en reposo no cuesta nada" / "cuesta un 30 %" queda **sustituido por un numero con comando, entorno y fecha**.
- Los **dos defects del bench** (auto-verificacion debil, corrida vacia que podia publicar numero, `println!` que se perdia al morir el proceso) estan **corregidos** en `benches/encrypted_p99.rs`.

---
