---
title: "Envelope por pagina en el VantaFile cifrado (FIND-598) + orden de ataque de FIND-585..591 + brief de FIND-559"
kind: review
status: active
description: "Diseno, sin implementar, del paso del envelope del VantaFile cifrado de por nodo a por pagina con cache LRU de paginas descifradas; el orden de ataque de los 7 hallazgos de cifrado que siguen abiertos; y el brief de decision de la clausula ROLE para el owner."
tags: [vantadb, review, encryption, at-rest, adr-0057, find-598, find-585, find-586, find-587, find-588, find-589, find-590, find-591, find-559, vfile, lru, wal]
created: "2026-10-21"
---

# Envelope por pagina en el VantaFile cifrado — y el resto del frente de cifrado abierto

> **Estado:** diseno. **No hay codigo.** Este documento decide (o propone decidir)
> tres cosas:
>
> 1. **FIND-598** — el p99 del motor cifrado empeora +71,2 % y la causa es la
>    granularidad del envelope del VantaFile (por nodo). Disena el paso a
>    **por pagina (64 KiB) + cache LRU de paginas descifradas**.
> 2. **FIND-585..591** — siete hallazgos de cifrado abiertos. Propone el **orden
>    de ataque**, con el criterio, el riesgo, los tests que lo fijan y que
>    pasos pueden ir en paralelo.
> 3. **FIND-559** — la clausula `ROLE` poda todo nodo sin `_owner_role` y nadie
>    escribe ese campo. Brief de decision para el owner.
>
> **Metodo:** lectura del codigo y de los documentos citados. **No se compilo,
> no se ejecuto `cargo`, no se lanzo ningun test ni bench.** Todo lo que no
> pude resolver leyendo esta marcado como **"por verificar"** y listado en
> §8. Los numeros de rendimiento que se citan son los **medidos** por W7
> (`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md`), no estimaciones mias.
>
> **Alcance de la escritura:** este es el unico fichero que este trabajo toca.

---

## 1. Resumen ejecutivo

**El veredicto tiene cinco partes.**

**1.1 — El problema de FIND-598 esta bien diagnosticado, pero la causa que W7
identifico ("la granularidad del envelope") es una de dos, y la otra es mas
barata de arreglar y mas probable que sea la dominante.**

La medicion de W7 (`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md:227-236`)
atribuye el +71,2 % del p99 a que "el store cifrado no puede devolver una vista
prestada del mmap" y cuenta **3 alocaciones + 3 copias de 6 KB + 1 AEAD por
candidato**. Leyendo el codigo, hay **4** copias, no 3, y la cuarta esta en el
camino que la medicion **no nombra**: `read_header` sobre un segmento cifrado
copia el payload **entero** para extraer 64 bytes de cabecera
(`src/storage/vfile.rs:913-940`, la linea 938). Y el `read_full_vector` cifrado
reconstruye el vector **byte a byte** en un bucle `chunks_exact(4)` +
`f32::from_le_bytes` (`src/storage/vfile.rs:1061-1065`) — no es un `memcpy`, es
un bucle interpretado de 1536 iteraciones por candidato.

⇒ **Paso 0 obligatorio (sin cambio de formato):** eliminar la 4ª copia, usar
`align_to::<f32>()` en vez del bucle, y sustituir la cache de **una** entrada
por una **LRU de nodos descifrados** con vista prestada. Son ~40 lineas, no
tocan los bytes del segmento, y atacan directamente lo que la medicion
describe. **Si eso cierra el gap, el cambio de formato de disco no hace falta**
(YAGNI). Si no lo cierra, el diseno de pagina de §3 sigue siendo el camino, pero
ya con el dato de cuanto queda.

**1.2 — El diseno de "envelope por pagina" que se pide es correcto en su
intencion y mas caro de lo que la peticion sugiere.** Tres hechos que cambian
el coste y que estan todos en el codigo:

- El envelope por pagina **no se puede hacer sin romper el trait publico
  `VectorStoreRef`** (`src/index_port.rs:73`, metodo `read_full_vector` en
  `:86`): para devolver una vista prestada de una pagina cacheada, la
  pagina tiene que vivir en un `Arc` **que el consumidor posea**, y el tipo de
  retorno actual (`Cow<'_, [f32]>` atado a `&self`) no puede expresar eso.
  Es un **breaking change de API**, no solo un cambio de formato.
- El envelope por pagina **choca con la mutacion in situ** que el formato v3
  hace hoy (`set_tombstone` re-sella el nodo en su offset,
  `src/storage/vfile.rs:667-707`, invocado durante el replay del WAL en
  `src/storage/engine/init.rs:723`). Con paginas, esa mutacion obliga a
  **re-escribir una pagina del medio del fichero**, rompiendo la propiedad
  append-only del segmento.
- El envelope por pagina **multiplica el radio de impacto de una escritura
  rota**: hoy se pierde **1 nodo**; con paginas, se pierden **~10** (una
  pagina). El WAL lo repara, pero el diagnostico empeora.

Ninguna de las tres es un bloqueo: son el precio de la decision, y hay que
pagarlos explicitamente.

**1.3 — El WAL no cambia. Confirmado.** Su envelope **ya es por frame**
(`src/wal.rs:473-508`), el frame **es** la unidad de escritura, y el
formato v4 ya esta publicado. **Truncar el WAL en el checkpoint NO es seguro
como truncado in situ** (rompe el `slot` del AAD y el genesis de la cadena de
hashes): lo seguro es **rotar**, que ya existe (`src/wal.rs:1086-1139`). Ver
§4.

**1.4 — El orden de ataque de los 7 hallazgos** (§5) tiene una forma clara:
**primero los que no tocan el disco** (FIND-588, FIND-589 — son de superficie
de configuracion/CLI y se pueden hacer en paralelo entre si), **despues los
que dependen del registro** (FIND-585, FIND-587, FIND-590 — todos tocan
`src/storage/engine/init.rs` y `src/crypto.rs`), **FIND-586 en medio** (es el
unico que obliga a **subir la version del formato**, y por lo tanto tiene que
ir **antes** que cualquier otro cambio de formato, incluido el de FIND-598),
y **FIND-591 al final** (es el guard que debe cubrir todo lo anterior).

**1.5 — El brief de FIND-559** (§6) recomienda **mantener el fail-closed** (la
poda) y arreglar el **silencio**, no la semantica: un `ROLE` que podo el 100 %
de las filas debe decirlo, no devolver un conjunto vacio sin explicacion.

---

## 2. Problema 1 — el layout actual (la evidencia)

### 2.1 Como es hoy el segmento v3

**El numero de version y donde vive.** El formato del VantaFile tiene **dos**
versiones, y la separacion es deliberada:

| Constante | Valor | Significado | Fichero:linea |
|---|---|---|---|
| `VFILE_VERSION` | `2` | El formato **en claro**. Un segmento escrito sin clave conserva su layout byte a byte, para siempre. | `src/storage/vfile.rs:52` |
| `VFILE_ENCRYPTED_VERSION` | `3` | El formato **cifrado**. | `src/storage/vfile.rs:70` |
| `VFILE_MAX_READABLE_VERSION` | `2` o `3` segun la feature | El build sin `encryption` rechaza un v3 con un mensaje explicito, nunca "corrupto". | `src/storage/vfile.rs:76-80` |

El marcador de version vive en la **cabecera en claro que ya existia**
(`Header.format_version`, leida en `src/storage/vfile.rs:445`), de modo que el
lector decide **por los bytes del disco**, nunca por la configuracion
(ADR-0057 D1, `docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md:152-180`).

**Como se versiona hoy (la respuesta a "que lector entiende el viejo").** Hay
**dos** caminos de lectura y el v3 los tiene los dos:

- `src/storage/vfile.rs:462-476` — un v3 **sin** clave ⇒ `Error::EncryptionKeyRequired`
  (o `IncompatibleFormat` en un build sin la feature). Nunca "corrupto".
- `src/storage/vfile.rs:477` — `header.validate_compat(*b"VFLE", VFILE_MAX_READABLE_VERSION, ...)`
  acepta cualquier version <= la maxima legible.
- `src/storage/vfile.rs:436-444` — un fichero **nuevo** (sin magic `VFLE`) se
  inicializa con la version que dicta el contexto: v3 con clave, v2 sin ella.

**El layout del registro v3** (el "envelope por nodo"):

```
v3, registro de nodo @ offset `off` (alineado a 64 B):

  [u32 blob_len][12B nonce][ct ‖ 16B tag]

  plaintext = DiskNodeHeader (64 B) ‖ vector payload
```

Evidencia:

- El envoltorio se construye en `src/storage/vfile_crypto.rs:168-188`
  (`seal_node`): `blob_len` (u32 LE), el nonce, y el blob AEAD.
- El largo del prefijo: `RECORD_PREFIX_LEN = 4 + 12 = 16`
  (`src/storage/vfile_crypto.rs:350`), `TAG_LEN = 16` (`:347`),
  `NONCE_LEN = 12` (`:344`).
- El **plaintext** del sobre es `DiskNodeHeader ‖ payload`
  (`src/storage/vfile.rs:942-965`): el `File::seal_node` **reescribe**
  `header.vector_offset = 64` (`:960`, la constante `ENCRYPTED_PAYLOAD_OFFSET`
  en `:89`) porque dentro del sobre el payload ya no es una direccion de
  fichero, es un offset *dentro del blob*.
- El **AAD** ata el sobre a su **offset en bytes dentro del segmento**
  (`src/storage/vfile_crypto.rs:170-177` al sellar, `:214-221` al abrir):
  `AadBuilder::new(Artifact::VantaFile, VFILE_ENCRYPTED_VERSION).epoch(..).slot(offset).plaintext_len(..)`.
  **No lleva `identifier`**: por eso un nodo de `vstore_L0.vanta`@X autentica
  en `vstore_L1.vanta`@X (FIND-586).
- El **paso** de un nodo: prefijo (16) + cabecera (64) + payload + tag (16),
  redondeado a 64 B (`src/storage/vfile.rs:649-657`, `node_stride`).

**Como se escribe hoy.** Un unico primitivo, `write_node_to_vstore`
(`src/storage/ops.rs:70`), que bifurca por `vstore.is_encrypted()`
(`src/storage/ops.rs:71-74`) hacia `write_node_to_vstore_encrypted`
(`src/storage/ops.rs:286-330`): materializa el payload (`materialize_payload`,
`src/storage/ops.rs:255-278` — la degradacion `MmapFull → Full` que ADR-0057
§5.2 declara), calcula el largo del registro, crece el fichero, y llama a
`File::write_node` (`src/storage/vfile.rs:894-911`), que sella y copia el
sobre al `mmap`.

**Como se lee hoy — el camino caliente.** El motor puntua **cada candidato** que
visita el recorrido HNSW leyendo su vector del vstore
(`src/index/search/layer.rs:84` y `:257`). De ahi la cadena:

```
read_header(offset)          src/storage/vfile.rs:581-609
  └─ read_node_encrypted      src/storage/vfile.rs:913-940
       └─ decrypt_blob        src/storage/vfile.rs:967-993   (cache de UNA entrada)
            ├─ read_record_bytes  :995-1022   -> COPIA 1: el ciphertext a un Vec
            ├─ open_node          src/storage/vfile_crypto.rs:198-227 -> COPIA 2: el plaintext a un Vec
            └─ AlignedBlob::from_bytes  src/storage/vfile.rs:141-152 -> COPIA 3: a un Vec<u64> alineado
       └─ payload: bytes[64..].to_vec()   src/storage/vfile.rs:938 -> COPIA 4 (¡no contabilizada por W7!)

read_full_vector(offset)     src/storage/vfile.rs:1046-1072
  └─ chunks_exact(4) + f32::from_le_bytes  :1061-1065 -> COPIA 5, y no es un memcpy: es un bucle
```

⇒ **Por candidato: 5 alocaciones, 5 copias de ~6 KB, 1 AEAD y 1536 iteraciones
de bucle.** El `"Cow::Borrowed"` del camino en claro
(`src/index_port.rs:86`, `full_vector_from_bytes` en `:98-114`, que usa
`align_to::<f32>()` — el patron correcto) **no es alcanzable** en el camino
cifrado.

### 2.2 Lo que la medicion de W7 midio (y lo que no)

`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md:173-182`:

| Metrica | Claro | Cifrado | Delta |
|---|---:|---:|---:|
| search p50 | 2,3750 ms | 2,8442 ms | +19,8 % |
| search p95 | 5,0552 ms | 7,6095 ms | +50,5 % |
| **search p99** | **6,4655 ms** | **11,0673 ms** | **+71,2 %** |
| insert 2 000 nodos | 24,225 s | 26,427 s | +9,1 % |
| flush (imagen HNSW) | 0,103 s | 0,111 s | +7,8 % |
| cold start | 0,274 s | 0,394 s | +43,8 % |

La propia nota es honesta sobre el ruido: un protocolo identico 1 h 40 min antes
dio el **signo contrario** (−14,2 %) y el spread intra-variante es de ±56 % a
+200 % (`§3.2` de la nota). **El gate del 15 % no queda certificado** y hace
falta una maquina dedicada. ⇒ **Regla 9: ninguna conclusion de rendimiento sin
medir; este documento no afirma que el envelope por pagina baje el p99, afirma
que elimina una fuente concreta de trabajo por candidato y que eso se puede
medir.**

### 2.3 Lo que el formato v3 no puede hacer

1. **Devolver una vista prestada del plaintext** — el plaintext vive en un
   `Vec` owned (`src/storage/vfile.rs:987`) y el consumidor lo recibe como
   `Cow::Owned` (`:1065`).
2. **Reutilizar el trabajo entre candidatos** — la cache de descifrado es de
   **una sola entrada** (`src/storage/vfile.rs:290-295`, `ArcSwap<Option<Arc<DecryptedNode>>>`),
   invalidada en cada escritura (`:909`). Sirve para el patron
   "lee la cabecera y luego el vector **del mismo nodo**", no para el recorrido
   HNSW, que visita nodos distintos.
3. **Exprimir la identidad del fichero en el AAD** — FIND-586.

---

## 3. Problema 1 — el diseno del envelope por pagina

### 3.1 La forma del formato (propuesta)

**Versionado.** `VFILE_ENCRYPTED_VERSION` pasa de `3` a `4` y se introduce
`VFILE_ENCRYPTED_PAGE_VERSION: u16 = 4` (`src/storage/vfile.rs:70` y siguientes).
El **plaintext no se toca**: `VFILE_VERSION` sigue en `2`
(`src/storage/vfile.rs:52`) y un segmento sin clave conserva su layout byte a
byte (ADR-0057 D1 regla 1). `VFILE_MAX_READABLE_VERSION`
(`src/storage/vfile.rs:76-80`) pasa a `4` **solo** con la feature `encryption`;
un build sin ella rechaza el v4 con `IncompatibleFormat`
(`src/storage/vfile.rs:449-459`).

**El lector del viejo.** Un v3 **sigue siendo legible** por el build actual: el
`format_version` de la cabecera lo dice y el lector bifurca por el **marcador**
(el patron `is_encrypted()` ya existe, `src/storage/vfile.rs:640-642`). Esto es
**obligatorio** porque el v3 ya salio en el arbol (W6, 2026-10-21) y puede
haber bases cifradas en disco.

> **Versionado del WAL, para que quede simetrico:** el WAL ya tiene su propio
> par (`WAL_FORMAT_VERSION = 3` / `WAL_ENCRYPTED_FORMAT_VERSION = 4`,
> `src/wal.rs:26` y `:45`) con el mismo patron "el marcador vive en la cabecera
> en claro". El VantaFile lo replica.

**Layout de la pagina.** El segmento v4 es:

```
[0 .. 16)     Header (binary_header::Header, magic "VFLE", format_version = 4)
[16 .. 24)    write cursor (u64 LE)
[24 .. 64)    reservado (ceros)
[64 ..)       area de paginas

pagina p (indice p), que ocupa los bytes [64 + p*65536, 64 + (p+1)*65536):

  [0 .. 4)     u32 plaintext_len   (bytes de plaintext utiles en la pagina)
  [4 .. 16)    12B nonce
  [16 .. 16+ct_len)  ct ‖ 16B tag
  resto        relleno a cero hasta 65536
```

**La propiedad clave: el plaintext de una pagina es byte a byte el layout
plano (v2) de la region que reemplaza.** Es decir, dentro del plaintext de la
pagina hay la misma secuencia de `[DiskNodeHeader 64B][payload]` que en un
segmento plano, con `header.vector_offset` siguiendo siendo una **direccion
absoluta de fichero**. El lector descifra la pagina y aplica **los mismos
lectores que ya existen**: `DiskNodeHeader::read_from_bytes` y el payload en
`vector_offset − base_de_la_pagina`.

Esto tiene tres ventajas decisivas:

1. **El `read_header` central** (`src/storage/vfile.rs:581-609`), que es el
   guard de alineacion de 4 B (INV-024 M-1) y el unico sitio donde se aplica
   (`src/storage/vfile.rs:567-575`), **sigue siendo el unico guard**: no se
   duplica la validacion en el camino cifrado. Regla "validar en bordes,
   confiar internamente".
2. Los **lectores de vector** (`src/index_port.rs:98-114`,
   `src/storage/engine/get.rs:74-91`) siguen funcionando sobre `&[u8]`: no
   hay que tocar ni el motor ni el indice para *leer*.
3. La **compaction** (`src/storage/archive.rs:160-266`) puede reescribir el
   segmento **por paginas** sin cambiar la semantica de los registros.

**El AAD de la pagina** (usando el `AadBuilder` que ya existe,
`src/crypto/registry.rs:173-233`):

```
AadBuilder::new(Artifact::VantaFile, VFILE_ENCRYPTED_PAGE_VERSION)
    .epoch(nonce.epoch())
    .slot(page_index)              // el indice de la pagina dentro del segmento
    .plaintext_len(plaintext_len)
    .identifier(segment_file_name)  // "vstore_L0.vanta"
    .build()
```

> **El `identifier` es la novedad que cierra la mitad del VantaFile de
> FIND-586** (`src/storage/vfile_crypto.rs:170-177` y `:214-221` hoy no lo
> llevan): con el nombre del segmento en el AAD, un sobre de `vstore_L0.vanta`
> no autentica en `vstore_L1.vanta`. Los cuatro segmentos L0..L3 comparten DEK
> y espacio de nonces (`src/storage/engine/init.rs:60-80` resuelve **un**
> contexto y lo clona a cada `File`), asi que hoy el unico atrinchero es el
> offset. **El envelope por pagina y el fix de FIND-586 son el mismo cambio de
> formato ⇒ no se pueden hacer por separado sin hacer el trabajo dos veces.**

**Restriccion de tamano.** El registro de un nodo (64 B de cabecera + payload
alineado) **no puede cruzar** el limite de una pagina: si no cabe en el hueco
restante, el escritor **rellena** el resto de la pagina con ceros y abre la
siguiente. Con el vector canonico de 1536 d (6144 B de payload) el registro
mide 64 + 6144 = 6208 B ⇒ caben **10 nodos** por pagina de 64 KiB
(62 080 B) ⇒ el desperdicio medio es de ~5,3 % del espacio.
**Un nodo cuyo registro no quepa en una pagina (payload > ~64 KiB) no tiene
representacion.** Esto hay que decidirlo explicitamente:

- **Opcion recomendada:** la pagina se dimensiona para que quepa el nodo mas
  grande admisible, y por encima de un limite se devuelve un error tipado
  (`Error::Validation`), no un panic.
- **Extension posible:** la cabecera de pagina lleva un `u32 page_count` y un
  nodo gigante ocupa varias paginas consecutivas selladas en un solo sobre.
  Mas complejo; solo si el producto admite vectores de >16k dimenes.

> **Por verificar:** no he encontrado en el repo un **tope de dimension de
> vector** documentado. `MAX_PERSISTED_NODE_BYTES = 128 MiB`
> (`src/storage/ops.rs:33`) es un tope de *deserializacion*, no de tamano de
> nodo. Hay que fijar el limite antes de fijar el tamano de pagina.

### 3.2 El modelo de escritura — y las "dos paginas"

Este es el punto donde la peticion y el codigo discrepan, asi que en §3.2 se
presentan los **dos** modelos con su coste.

**Modelo A — paginas append-only, con la cola en RAM (recomendado).**

El escritor mantiene una **pagina abierta** (buffer de plaintext en RAM) mas
un indice de la siguiente pagina. Al escribir un nodo:

1. Si no hay pagina abierta, se abre una en el indice `next_page`.
2. Si el registro no cabe en el hueco: se **sella** la pagina abierta (1 AEAD)
   y se escribe al fichero; se abre la siguiente. **Aqui estan las "dos
   paginas"**: la que se sella (la del final) y la que recibe el nodo.
3. El nodo se anade al buffer; su **offset de fichero** es
   `base_de_la_pagina + hueco`, calculable por adelantado ⇒ el HNSW lo puede
   guardar sin esperar al sellado.

*Coste*: la escritura al disco es **amortizada** (una pagina por ~10 nodos) ⇒
el volumen escrito por nodo es el mismo que hoy (~6 KB) e incluso mejor (1 AEAD
por ~10 nodos en vez de 1 por nodo).

*Consecuencias que hay que pagar*:

- **La cola no esta en disco hasta que se sella.** El `flush()` tiene que
  **sellar la pagina abierta ANTES de `save_vector_index()`**
  (`src/storage/engine/maintenance.rs:64`), porque la imagen HNSW puede
  referenciar offsets de la pagina abierta. Esto es una **restriccion de
  orden nueva**, al lado de la que ya existe por ERR-010
  (`src/storage/engine/maintenance.rs:66-74`).
- **El snapshot tiene que sellar tambien.** `create_snapshot` →
  `mirror_data_dir` (`src/storage/engine/mod.rs:717`, `:543`) copia el
  `data_dir`: si la cola no esta sellada, **el snapshot pierde la cola**. Es un
  hallazgo de este diseno, no del codigo actual, y hay que anadirlo al plan.
- **La lectura en el proceso** debe consultar la cola en RAM antes que el
  fichero. Como la cola se muta bajo `&mut self` y se lee bajo `&self`, y el
  motor serializa con `RwLock<File>` (`src/storage/engine/init.rs:86`), no hay
  carrera: la cola puede ser un campo plano de `File`.

**Modelo B — sellar en cada escritura (read-modify-write de la pagina).**

Cada escritura de nodo descifra la pagina que lo contiene, lo modifica, lo
re-sella y lo reescribe.

*Coste*: **1 AEAD + 64 KiB escritos por nodo insertado** ⇒ ~10× de
amplificacion de escritura en el vstore (6,4 GB en vez de 640 MB cada 100k
nodos de 1536 d). A cambio: la cola siempre esta en disco ⇒ sin restricciones
de orden con el HNSW, sin problema de snapshot, y `set_tombstone` funciona
naturalmente.

*Veredicto*: **Modelo A**, porque la amplificacion de escritura de B es una
regresion garantizada en el camino de insert (que hoy esta dentro del umbral:
+9,1 %) y porque el WAL ya da la durability. **El coste de A es el acoplamiento
del sellado con `flush()` y con el snapshot**, que son dos puntos concretos y
testeables.

### 3.3 Atomicidad: el WAL sigue siendo la unidad de escritura

**Confirmado: el WAL no cambia.** Su envelope es **por frame**
(`src/wal.rs:473-508`: `WalCrypto::frame` construye
`[u32 ct_len][u32 crc32c(ct)][12B nonce][ct ‖ 16B tag][32B prev_hash][32B record_hash]`),
el frame **es** la unidad de escritura, y el formato v4 ya esta publicado y
leido por `recover_valid_records_v4` (`src/wal.rs:1676-1699`) y por el replay
(`src/storage/engine/init.rs:603-614`). **El envelope por frame ya es la
granularidad correcta** (ADR-0057 D1, fila WAL). **No cambia.**

**La cadena de razonamiento de la atomicidad, con evidencia:**

1. **El WAL es la fuente de verdad y el vstore es derivado.** El replay del
   WAL **reescribe** el nodo en el vstore: `replay_write_node` →
   `write_node_to_vstore` (`src/storage/engine/ops.rs:213`). Es decir, el
   vstore se puede reconstruir desde el WAL.
2. **Por tanto una escritura de pagina rota no pierde datos**: el WAL
   re-aplica el nodo (en un offset nuevo, porque el vstore es append-only) y
   actualiza el HNSW en memoria (`src/storage/engine/init.rs:696-704`).
3. **Lo que cambia es el radio de impacto y el diagnostico**: hoy una escritura
   rota pierde **1 nodo**; con paginas pierde **~10** (la pagina entera).
   El error que sale es el mismo: el tag AEAD falla ⇒
   `Error::EncryptionKeyRejected` ("wrong key or tampered",
   `src/storage/vfile_crypto.rs:222-226`; ADR-0057 D6 dice explicitamente que
   un solo tag no puede distinguir "clave equivocada" de "manipulado").
   ⇒ **Declarar en el ADR que el error de una pagina no autenticada no
   significa "clave equivocada" sino "la pagina esta rota o ha sido
   manipulada"**, y que el radio de impacto es una pagina.
4. **El WAL no se toca ⇒ la unidad de escritura sigue siendo el frame del
   WAL**, y el vstore sigue siendo un store derivado, append-only, reparable
   por replay. La atomicidad que ADR-0057 D5 declara ("el envelope es la
   unidad que se escribe") se cumple **con mas fuerza**: ahora la unidad
   escrita es una pagina, que es todavia mas grande que el frame.

**Lo que hay que tocar para que el punto 3 sea cierto (y son los mismos puntos
del §3.2):**

- `flush()` (`src/storage/engine/maintenance.rs:35-92`): sellar la pagina
  abierta **antes** de `save_vector_index()` (`:64`) y antes de escribir
  `checkpoint_seq` (`:84-88`).
- `compact_wal` (`src/storage/engine/maintenance.rs:126-142`): idem (llama a
  `flush()` primero, `:130`).
- `create_snapshot` (`src/storage/engine/mod.rs:717`): sellar antes de copiar.
- `compact_layout_encrypted` (`src/storage/archive.rs:160-266`): reescribir
  **por paginas** en vez de por nodos (el recorrido BFS del grafo HNSW ya da el
  orden fisico: `traverse_graph`, `src/storage/archive.rs:268-269` ⇒ **la
  pagina compacted tiene la localidad del grafo**, que es justo lo que hace
  que la cache de paginas funcione bien).
- `replace_backing_file` (`src/storage/vfile.rs:541-565`): ya invalida la cache
  de descifrado (`:563`) ⇒ con paginas, invalida la cache **completa**.

### 3.4 La cache de paginas descifradas

**Donde vive.** **Por `File`** (o sea, por segmento), dentro del proceso — no
global. Razon: el `File` ya es el dueno del cache de descifrado actual
(`src/storage/vfile.rs:290-295`) y es donde se puede invalidar con precision
(`write_node` invalida hoy, `:909`; `replace_backing_file` invalida hoy,
`:563`). Una cache global por base exigiria una clave con la ruta del segmento
y un reloj comun; no aporta nada aqui. **Por base** (en el sentido de "no
comparte estado entre bases") ✓ — cada `File` pertenece a una base y a un
segmento.

**Su clave.** El **indice de pagina dentro del segmento**. La identidad del
segmento es implicita (la cache es de ese `File`). Si en el futuro se
quiere una cache global, la clave pasa a ser `(ruta canonica del segmento,
indice de pagina)`.

**Su cap.** Debe ser un tope **declarado** (regla 4 de
`.opencode/rules/memory-budget.md`: "toda estructura residente nueva lleva un
cap declarado", con politica documentada al alcanzarlo) y **relacionado con
`memory_limit`** (`Config::memory_limit: Option<u64>`, `src/config.rs:733`;
el parser acepta `64KiB`, `500MB`, `2GB`, `src/config.rs:1025`).

Propuesta:

```
paginas = clamp(memory_limit / PAGE_SIZE / 8, 16, 4096)
```

- `memory_limit = 4 GiB` ⇒ 4096 paginas ⇒ **256 MiB** de plaintext en RAM.
- `memory_limit = None` ⇒ el default (16 paginas = 1 MiB) — conservative: sin
  limite declarado, el motor no reserva memoria que el operador no pidio.
- El `/8` es el margen: la cache de paginas no puede ser el unico consumidor de
  la memoria (el HNSW es 100 % RAM residente, `.opencode/rules/memory-budget.md:14`).

**Politica al alcanzar el cap: eviccion LRU, nunca rechazo de una lectura.**
Un acierto de cache siempre es "hit"; un fallo se descifra y se inserta,
sustituyendo la victima menos usada. La lectura **nunca** falla por falta de
cache.

**Implementacion (sin dependencias nuevas).** `lru = "0.18"` ya es dependencia
(`Cargo.toml:77`) y `arc-swap = "1.7"` tambien (`Cargo.toml:79`).
⚠️ **`lru::LruCache::get` necesita `&mut self`** ⇒ con `parking_lot::Mutex`
el `get` devuelve una referencia atada al guard, que **no puede escapar de la
funcion** ⇒ el `Cow::Borrowed` no puede salir de la cache. **Por eso el tipo
de retorno tiene que cambiar** (ver §3.5).

**Invalidacion — cuando se invalida una pagina:**

| Evento | Efecto | Fichero:linea |
|---|---|---|
| Se escribe un nodo en la pagina P | Se invalida **P** (su plaintext cambio) | `src/storage/vfile.rs:894-911` (el `write_node` de hoy ya invalida la cache de una entrada, `:909`) |
| Se sella la pagina abierta | Se inserta la version nueva en la cache (es la mas caliente) | §3.2 |
| Se re-sella el fichero (`replace_backing_file`) | Se invalida **toda** la cache del segmento | `src/storage/vfile.rs:541-565` (ya lo hace hoy en `:563`) |
| Crece el fichero (`grow_to` → `remap_mut`) | **No** invalida: el contenido de las paginas ya selladas no cambia; solo cambia el mapping | `src/storage/vfile.rs:754-789`, `:519-538` |
| Se compacta el segmento | Se invalida **toda** la cache del segmento (los offsets cambian) | `src/storage/archive.rs:262` (`replace_backing_file`) |

### 3.5 El aliasing: el buffer no puede caducar mientras el consumidor lo mira

Este es el punto que la peticion marca como **"esto es lo importante"**, y es
donde el diseno se topa con la API.

**El requisito.** El consumidor (el motor, en
`src/index/search/layer.rs:84` y `:257`) recibe el vector y lo usa **en el
mismo bloque**. Si la cache puede expulsar la pagina mientras el consumidor la
mira, el buffer tiene que seguir vivo. La solucion estandar: la cache guarda
`Arc<PageBuffer>` y el **consumidor** clonea el `Arc` ⇒ mientras el consumidor
lo tenga, la pagina esta viva, **aunque la cache la haya expulsado**. La
expulsion solo suelta la referencia de la cache.

**El problema: el tipo de retorno actual no puede expresar eso.**

```rust
// src/index_port.rs:86
fn read_full_vector(&self, offset: u64, header: &DiskNodeHeader) -> Option<Cow<'_, [f32]>>;
```

- El `'_` esta atado a `&self`. Una pagina cacheada vive en un `Arc` que la
  **cache** posee; para devolver `Cow::Borrowed` el metodo tendria que
  devolver una referencia a un buffer que el cache puede soltar ⇒ **aliasing
  inseguro** (use-after-free logico).
- Y no se puede "arreglar" por dentro: con `lru::LruCache` (que exige
  `&mut` para `get`) bajo un `parking_lot::Mutex`, la referencia devuelta
  esta atada al guard, que es local ⇒ no puede salir con el lifetime de
  `&self`.

⇒ **El zero-copy con cache obliga a cambiar el tipo de retorno del trait
publico `VectorStoreRef`.** Esto es un **breaking change de API**, no un
detalle de implementacion:

```rust
// Propuesta: un valor que POSEE la pagina (Arc) y expone la vista prestada.
pub enum NodeVector<'a> {
    /// Prestado del mapping (segmento en claro) — zero-copy, como hoy.
    Borrowed(&'a [f32]),
    /// Prestado de una pagina descifrada cacheada: el valor POSEE el Arc.
    Cached {
        page: Arc<PageBuffer>,
        range: Range<usize>,
    },
    /// Materializado (degradacion explicita).
    Owned(Vec<f32>),
}

impl NodeVector<'_> {
    /// La vista del vector. En `Cached`, el `Arc` del propio valor mantiene
    /// viva la pagina mientras la vista exista.
    pub fn as_f32(&self) -> &[f32] { /* ... */ }
}
```

Con `Cached`, el `Arc` viaja **dentro del valor de retorno** ⇒ el buffer no
puede caducar mientras el consumidor lo mira ✓, y la vista `&[f32]` que se
presta desde el `Arc` tiene el lifetime del valor de retorno ⇒ Rust lo
comprueba. **Es la unica forma segura de devolver una vista prestada de una
cache.**

**Radio de impacto del breaking change:**

- `src/index_port.rs:73` — el trait `pub trait VectorStoreRef: Send + Sync`
  (**no** esta sellado ⇒ es API publica: un implementor externo posible).
- `src/index_port.rs:86` — el metodo a cambiar.
- `src/storage/vfile.rs:1029-1073` — el `impl` de `File`.
- `src/index/flat.rs:542-553` — el `impl` de `TestVectorStore` (tests).
- `src/index/search/layer.rs:84`, `:257` — los dos call sites.
- `tests/api/public-api.txt:3719-3722` — **el snapshot de superficie publica
  lista el trait con `mmap_bytes` y `read_header` pero NO con
  `read_full_vector`** ⇒ **el gate de superficie publica es ciego a este
  metodo**. Un cambio de su firma pasaria el gate sin avisar. Hay que
  arreglar el gate (o anadir el metodo al snapshot a mano) **en el mismo PR**.
  *Por verificar:* por que la herramienta (`cargo public-api --simplified`,
  `tests/api/public_api.rs:20`) omite los metodos con cuerpo por defecto — no
  lo heinvestigado; el hecho de que la linea no esta en el snapshot si esta
  verificado.

**Semver:** el crate es pre-1.0 pero el changelog exige `feat!:` /
`BREAKING CHANGE:` ⇒ **major**. Y los bindings (`vantadb-python`,
`vantadb-node`) **no** implementan este trait (es interno al core Rust), asi
que el radio de impacto real se queda en el crate `vantadb`.

**Alternativa sin breaking change:** dejar `read_full_vector` como esta y
devolver `Cow::Owned` (una copia) ⇒ se pierde el zero-copy ⇒ **el objetivo del
trabajo se cae**. Por eso el breaking change es inevitable si se quiere el
zero-copy con cache.

### 3.6 El riesgo de seguridad: el plaintext en la memoria del proceso

**El limite ya declarado.** `SECURITY.md:33`:

> The **master key lives in the process memory** while the database is open:
> this protects the data at rest, **not** against an attacker who can read the
> process's memory (or who has the key).

Y ADR-0057 §5.3 punto 4 (`docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md:571-572`):
"No protection against an attacker who has the process's memory or the key
itself. The master key lives in process memory while the database is open."

**Lo que hay que anadir.** El envelope por pagina introduce una clase nueva de
material sensible en RAM: **vectores en claro**, no solo la master key. Hoy el
plaintext en RAM es transitorio (un `Vec` por candidato, liberado al rato);
con la cache de paginas, **el plaintext de hasta 256 MiB vive en la memoria
del proceso de forma estable y consultable**.

**Texto exacto a anadir en `SECURITY.md`** (una viñeta nueva en la lista
"What "encrypted at rest" does NOT cover", `SECURITY.md:28-40`; colocar
**despues** de la viñeta de la linea 33, para que los dos limites de "el
proceso tiene el material" queden juntos):

```markdown
  - The **decrypted pages of the vector store live in the process memory**: an encrypted
    segment is decrypted **one page at a time** (64 KiB), and the decrypted pages are kept
    in a bounded in-memory cache (the cap is derived from `memory_limit`) so that a search
    does not have to decrypt again. While the database is open, those pages hold
    **plaintext vectors** in RAM. This does not move the "at rest" boundary: it protects
    the data on disk, **not** against an attacker who can read the process's memory (or who
    has the key).
```

**Y el texto a cambiar en `SECURITY.md:27`** (la afirmacion de las cuatro
superficies), porque hoy dice "the **VantaFile** segments (format v3, **per
node**)" ⇒ con el formato v4 pasa a "format v4, **per page** (64 KiB)":

```markdown
the **VantaFile** segments (format v4, per page of 64 KiB, with the decrypted pages cached
in a bounded in-memory cache)
```

**Y ADR-0057 §5.2 hay que enmendarlo**: hoy dice "The mmap zero-copy tier is
forfeited for encrypted stores" (`:533-544`). Con la cache de paginas, el
zero-copy **vuelve** para las paginas cacheadas (la vista prestada sale del
buffer descifrado, no del mapping). El texto nuevo tendria que decir que el
zero-copy se **recupera** a cambio de un coste en RAM y de un radio de impacto
mayor en una escritura rota. **Esto es un cambio de contrato documentado ⇒
ADR nuevo (no un parche del ADR-0057).**

### 3.7 Lo que NO cambia

| Artefacto | Envelope hoy | ¿Cambia? | Por que |
|---|---|---|---|
| **WAL** | por **frame** (`src/wal.rs:473-508`) | **NO** | El frame **es** la unidad de escritura; el formato v4 ya esta publicado y leido (`src/wal.rs:1676-1699`). El coste medido (+0,6 a +1,2 µs por frame) queda enterrado bajo el fsync (`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md:213-221`). |
| **backend-KV** | por **valor** (`[12B nonce][ct‖16B tag]`, ADR-0057 D1 fila 4) | **NO** | **Su cache seria el mismo problema?** No: el backend KV no tiene un modo de acceso aleatorio sobre un mapping. Se accede por clave, y el valor se descifra **en el sitio de la lectura** (`src/backends/kv_crypto.rs:401-413`, el iterador de `scan_prefix` descifra cada valor). El "coste" es 1 AEAD por valor, y el valor **se necesita entero**. Una cache de valores descifrados tendria el mismo problema de **secretos en RAM** pero no el de "no se puede devolver una vista prestada": un valor del backend no se usa como `&[f32]` en un hot path comparable, y el indice de texto y el de espacios de nombres **exigen** el valor en claro para operar (ADR-0057 §5.3.1). ⇒ **La cache del backend no aporta; ademas el texto indexado esta en claro de todos modos (SECURITY.md:30).** |
| **imagen HNSW** | un **unico** AEAD sobre el indice entero (`src/index/serialize/crypto.rs:202-232`) | **NO (pero con matiz)** | El envelope por pagina **si** le serviria: el cold start (+43,8 %) paga descifrar la imagen entera; con paginas, se descifrarian solo las que hacen falta (o en paralelo). **PERO** la imagen se reescribe **entera** en cada flush (`src/storage/engine/maintenance.rs:64` → `save_vector_index`, `:144-158`) ⇒ es un artefacto de **escritura completa**, no de acceso aleatorio. Ponerlo por pagina no amortiza nada en la escritura (habria que re-sellar toda pagina en cada flush) y solo (parcialmente) partiria el cold start. **Decision recomendada: dejar la imagen como un sobre unico** y, si el cold start molesta, atacarlo por otro lado (ver §4). |

### 3.8 Alternativa A — el paso 0, sin cambio de formato

**Esta es la alternativa que hay que medir ANTES de tocar el disco**, y es la
que la escalera de YAGNI impone.

**Que cambia (todo dentro de `src/storage/vfile.rs`, sin tocar el formato):**

1. **`read_header` cifrado que no copia el payload.**
   `src/storage/vfile.rs:913-940`: hoy construye un `NodeRecord` con
   `payload: bytes[header_size..].to_vec()` (`:938`) **solo** para extraer la
   cabecera de 64 bytes. El fix: una funcion aparte que parsea la cabecera
   desde el blob cacheado **sin materializar el payload**. Ahorra 1
   alocacion + 1 copia de 6 KB **por candidato**.
2. **`read_full_vector` cifrado que usa `align_to::<f32>()` en vez del bucle.**
   `src/storage/vfile.rs:1061-1065`: hoy reconstruye el vector byte a byte
   (`chunks_exact(4)` + `f32::from_le_bytes`) en un `Vec<f32>` propio. El
   `AlignedBlob` esta alineado a 8 B (`src/storage/vfile.rs:119-171`), asi que
   `align_to::<f32>()` es valido y da **`Cow::Borrowed` sin copia** — el mismo
   patron que ya usa el camino en claro (`src/index_port.rs:105-113`). Ahorra
   1 alocacion + 1 copia + 1536 iteraciones de bucle **por candidato**.
3. **Cache de mas de una entrada.** `src/storage/vfile.rs:290-295` (una sola
   entrada) → una LRU de N nodos descifrados (`Arc<AlignedBlob>`), con la
   misma invalidacion que ya existe (`:909`, `:563`). Con el patron de
   lectura real (la cabecera y el vector **del mismo nodo**, mas las
   re-visitas del recorrido HNSW), el hit rate sube.
4. **Evitar la copia del ciphertext.** `read_record_bytes`
   (`src/storage/vfile.rs:995-1022`) copia el ciphertext a un `Vec` para
   pasarselo a `open_node` (`src/storage/vfile_crypto.rs:198`). Si
   `Cipher::decrypt_with_nonce` aceptara `&[u8]` de entrada y escribiera en un
   buffer de salida, esa copia desaparece. Es un cambio en
   `src/crypto.rs` (la API C-1 del ADR-0057 D8 ya existe; lo que falta es una
   variante "decrypt into").

**Que NO arregla.** El **AEAD por candidato** sigue siendo 1: la
amortizacion del descifrado (10 nodos por AEAD) **solo** la da el envelope por
pagina. Este es el punto en el que el diseno por pagina gana.

**Por que es el paso 0 y no un extra:**

- Es ~40 lineas, **no cambia un byte del disco**, no cambia ninguna API
  publica, y no se puede romper la compatibilidad.
- Ataca **exactamente** lo que la medicion describe ("el store cifrado
  materializa 3 veces los 6 KB por candidato"), y ademas una copia que la
  medicion no conte.
- Si con esto el p99 baja del +71,2 % al umbral del 15 %, **el cambio de
  formato de disco no hace falta** ⇒ se ahorra un breaking change de API
  (§3.5), un cambio de formato en disco, el acoplamiento del sellado con
  `flush()` y el snapshot (§3.2), y el radio de impacto de una escritura rota.
- Regla 9 de `.opencode/AGENTS.md`: **sin medicion no se afirma**. Este
  documento no afirma que el paso 0 cierre el gap; afirma que es el primer
  paso, que es barato, y que su resultado decide si el resto hace falta.

### 3.9 Plan de pasos (FIND-598)

Cada paso: que hace, que fichero toca, que riesgo tiene, que test lo fija.

| # | Paso | Ficheros | Riesgo | Test que lo fija |
|---|---|---|---|---|
| **0** | **Medir el estado actual en una maquina dedicada** (el gate del 15 % no esta certificado: W7 §3.2). Sin este numero no se puede juzgar nada. | (ninguno: es un bench) | Ninguno | `benches/encrypted_p99.rs`, protocolo de W7 §2.1, 3 corridas intercaladas |
| **1** | **Paso 0 sin cambio de formato**: (a) `read_header` cifrado sin copiar el payload; (b) `read_full_vector` con `align_to`; (c) cache LRU de nodos descifrados; (d) `decrypt into` sin copiar el ciphertext | `src/storage/vfile.rs` (`:913-940`, `:1046-1072`, `:290-295`, `:995-1022`), `src/storage/vfile_crypto.rs`, `src/crypto.rs` | **Bajo**: no toca el disco ni la API publica. Riesgo real: el `align_to` sobre el blob alineado (ya cubierto por el invariante de `AlignedBlob`, `src/storage/vfile.rs:119-171`) | Los tests existentes de `src/storage/vfile_crypto.rs` (round-trip, corrupto, clave incorrecta) + un test nuevo: "el vector devuelto por `read_full_vector` en un segmento cifrado es `Cow::Borrowed`" |
| **2** | **Medir de nuevo** con el mismo protocolo | (ninguno) | Ninguno | Mismo bench. **Gate: si el p99 delta <= 15 %, el paso 3+ se cancela y FIND-598 se cierra con el paso 1** |
| **3** | **ADR del envelope por pagina** (formato v4, AAD con `identifier`, radio de impacto, limite de RAM) + actualizar ADR-0057 §5.2 y §5.3 | `docs/dev/architecture/adr/NNN_*.md` | — | Revision del owner (Regla 5: el ADR lo escribe el humano) |
| **4** | **FIND-586 antes que nada** (ata la identidad del fichero al AAD en los 4 artefactos) | `src/storage/vfile_crypto.rs:170-177`, `:214-221`; `src/wal.rs:480-484`, `:1346-1350`; `src/wal_sharded.rs:390-393`; `src/index/serialize/crypto.rs:214-222`, `:252-260` | **Medio**: cambiar el AAD invalida los sobres existentes ⇒ hay que **subir la version del formato** (v3→v4 del VantaFile, v4→v5 del WAL, v9→v10 de la imagen) **y decidir que pasa con las bases cifradas v3/v4 ya escritas**. ADR-0057 decia "los formatos v3/v4 son nuevos y sin publicar, se puede subir de version sin coste de migracion" — **eso ya no es cierto**: W6 los encendio el 2026-10-21 | Test de la revision P2-01: un blob de L0@X en L1@X **falla** la autenticacion (el test que el backlog ya describe) |
| **5** | **El envelope por pagina, en el mismo PR que el paso 4** (comparten el AAD y el bump de version) | `src/storage/vfile.rs`, `src/storage/vfile_crypto.rs`, `src/storage/ops.rs`, `src/storage/archive.rs` | **Alto**: cambio de formato en disco + rotura del trait publico | Round-trip v4; leer un v3 **sin** cambios (compat); compactacion round-trip; "una pagina de 64 KiB con 10 nodos hace 1 AEAD" |
| **6** | **La cache de paginas + el breaking change del trait** | `src/index_port.rs:86`, `src/storage/vfile.rs:1029-1073`, `src/index/search/layer.rs:84`,`:257`, `src/index/flat.rs:542` | **Alto**: breaking change de API + `Arc` en el camino caliente | Test de superficie publica (con el gate arreglado, §3.5); test de que el buffer sobrevive a la eviction |
| **7** | **El modelo de escritura A**: pagina abierta en RAM, sellado en `flush()`, `compact_wal()` y `create_snapshot()` | `src/storage/vfile.rs`, `src/storage/engine/maintenance.rs:35-92`, `:126-142`, `src/storage/engine/mod.rs:717` | **Alto**: el acoplamiento del sellado con el flush y el snapshot es donde se puede perder una cola | Test: "un snapshot tomado con la cola abierta contiene la cola"; test: "tras `flush()`, el fichero en disco contiene todos los nodos que el HNSW referencia" |
| **8** | **Miri con la feature `encryption`** (FIND-591, §5) | `.github/workflows/ci-rust.yml:679-689` | Bajo | El job de Miri pasa con `--features encryption` |
| **9** | **Medir** el p99 final en la maquina dedicada | (ninguno) | Ninguno | El gate del 15 % de ADR-0057 §6 |

> **Nota sobre el orden 4 → 5:** FIND-586 (ata el fichero al AAD) y el envelope
> por pagina (ata la pagina al AAD) **tocan el mismo codigo y el mismo
> versionado**. Separarlos es hacer el trabajo dos veces. ⇒ **FIND-586 y el
> envelope por pagina son un solo PR de formato**, con el orden dentro del PR:
> primero el `identifier` (que es un cambio pequeno y testeable), luego el
> envelope por pagina.

---

## 4. El WAL y el cold start

### 4.1 El problema

El cold start cifrado mide **+43,8 %** (`docs/dev/reviews/2026-10-21-w7-cifrado-coste.md:180`).
La nota atribuye dos causas (`§4.2` de la nota): la **imagen HNSW entera** (un
unico AEAD sobre el indice completo) y que **el WAL se valida entero en cada
open** (`recover_valid_records_v4` descifra todos los frames).

### 4.2 "Truncar el WAL en el checkpoint" — ¿es seguro?

**La pregunta es si `flush()` (que ya escribe `checkpoint_seq`,
`src/storage/engine/maintenance.rs:84-88`) puede **truncar** el WAL.

**Respuesta: NO como truncado in situ. SI como rotacion.** Y la razon es
concreta, en el codigo:

1. **El `slot` del AAD del WAL es el ordinal del frame dentro del segmento.**
   `src/wal.rs:480-484` (`.slot(slot)`) y `src/wal.rs:971`
   (`let mut slot = self.record_count;`): el ordinal **empieza en 0 en cada
   segmento** y **se reinicia** al rotar (`src/wal.rs:1132`:
   `self.record_count = 0`).
2. **La cadena de hashes tambien es por segmento.** El `chain_head` arranca en
   `CHAIN_GENESIS` (`src/wal.rs:763`, `:1136`) y el reader lo recupera del
   ultimo frame (`src/wal.rs:820-827`).
3. ⇒ **Truncar los primeros N frames del fichero desplaza los ordinales y
   rompe las dos cosas a la vez**: el frame que queda con ordinal `k` pasaria
   a tener ordinal `k−N` ⇒ su AAD es distinto ⇒ **no autentica**. Y la cadena
   de hashes dejaria de arrancar en el genesis.

**Lo que ya existe y es correcto: la rotacion.** `WalWriter::rotate`
(`src/wal.rs:1054`) y `try_auto_rotate` (`src/wal.rs:1086-1139`) archivan el
segmento (`vanta.wal.<timestamp>`, `:1100-1112`), crean uno nuevo en la ruta
original y **reinician el ordinal y la cadena** (`:1132`, `:1136`). El
`compact_wal` del motor (`src/storage/engine/maintenance.rs:126-142`) hace
exactamente eso: `flush()` → `rotate_all()` → **`checkpoint_seq = 0`**
(`:136-139`).

**Que habria que tocar para que `flush()` rota (y no solo `compact_wal`):**

| Punto | Fichero:linea | Que hay que hacer |
|---|---|---|
| La rotacion del WAL al hacer flush | `src/storage/engine/maintenance.rs:35-92` | `flush()` deberia rotar el WAL cuando supere un umbral (hoy solo rota a los 256 MiB, `src/wal.rs:843`, `max_segment_size`, y por `try_auto_rotate`) |
| **`checkpoint_seq` a 0 al rotar** | `src/storage/engine/maintenance.rs:136-139` | Ya lo hace `compact_wal`. Si `flush()` rota, tiene que hacer lo mismo: si no, el replay saltaria records equivocados. El calculo del skip por shard esta en `src/wal_sharded.rs:547-559` y se usa en `src/storage/engine/init.rs:585-586` |
| **Los shards** | `src/wal_sharded.rs:641` (`rotate_all`) | Rotar **todos** los shards a la vez: el skip se calcula con el `checkpoint_seq` global repartido por shard (`src/wal_sharded.rs:557-559`), asi que una rotacion parcial dejaria los shards desalineados |
| **Los archivos archivados** | `src/wal.rs:1193-1194` | Los `.wal.<ts>` **no se abren en el arranque** (solo `vanta.wal` y los shards) ⇒ no se descifran. Pero **acumulan**: hay que decidir quien los borra (el `rotate_all` los archiva; hay una funcion que los lista, `src/wal_sharded.rs:844`) |
| **La cola de replay** | `src/storage/engine/init.rs:545-548`, `:598-631` | Con el WAL rotado, el `checkpoint_seq` es 0 ⇒ el replay aplica **todos** los frames del segmento vivo ⇒ correcto, pero **el replay sigue descifrando todos los frames del segmento vivo**. El ahorro viene de que el segmento vivo es **pequeno** (lo que se escribio desde el ultimo flush), no de no descifrar |

**Conclusion honesta sobre el cold start.** Con la rotacion en `flush()`:

- El **cold start** deja de pagar "todo el WAL historico" y paga "el WAL desde
  el ultimo flush" ⇒ la mejora es **proporcional a la frecuencia de flush**.
- El **`recover_valid_records_v4`** (`src/wal.rs:1676-1699`) **sigue descifrando
  todos los frames del segmento vivo** (es la validacion de integridad: cada
  frame se descifra para verificar el tag). Eso no se puede evitar sin perder la
  deteccion de corrupcion. ⇒ **el cold start mejora, no desaparece**.
- El otro factor del cold start (la **imagen HNSW**, un unico AEAD sobre el
  indice entero) **no lo toca ninguna de las dos acciones**: por §3.7, la imagen
  se queda con un sobre unico.

> **Por verificar:** cuanto del +43,8 % del cold start es el WAL y cuanto es la
> imagen HNSW. La nota de W7 los menciona juntos sin separarlos. **No se puede
> decidir el presupuesto de trabajo del cold start sin ese reparto**, y
> separarlo exige un bench nuevo (o instrumentar el arranque). Marcar como
> **incognita abierta**.

### 4.3 Lo que el WAL **no** necesita

- **No cambia el envelope** (por frame, ya es la granularidad correcta).
- **No cambia el formato** por el cold start: el v4 ya esta publicado.
- **No necesita una cache**: los frames se leen una vez (secuencial) y el
  coste unitario (+0,6 a +1,2 µs) queda enterrado bajo el fsync.

---

## 5. Problema 2 — el orden de ataque de FIND-585..591

### 5.1 El criterio del orden

El orden no es "por severidad" (FIND-585 es Critica, FIND-591 es Alta). El
criterio es **que habilita que**:

1. **Lo que no toca el disco, primero** — son cambios de superficie (config,
   CLI) que no dependen de nada y no pueden romper el formato.
2. **Lo que depende del registro, en bloque** — FIND-585, FIND-587 y FIND-590
   dependen todos de la misma decision por proceso (`src/crypto.rs:365-404`)
   y del mismo predicado "base nueva" (`src/storage/engine/init.rs:1050-1077`).
   Tocarlos por separado es tocar el mismo codigo tres veces.
3. **FIND-586 antes de cualquier cambio de formato** — es el unico que obliga a
   **subir la version del WAL (v4→v5), del VantaFile (v3→v4) y de la imagen
   (v9→v10)**. Si el envelope por pagina de FIND-598 landara antes, habria dos
   bumps de version seguidos en los mismos ficheros.
4. **FIND-591 al final** — es el **guard** (Miri con la feature `encryption`) que
   debe cubrir todo lo anterior. Correrlo antes es correrlo dos veces.

### 5.2 El orden propuesto

```
   Fase 0  (paralelo, sin dependencias)
   ├── A  FIND-588  la master key se imprime (Config/StorageCfg derivan Debug)
   └── B  FIND-589  la master key via argv (--new-key)

   Fase 1  (secuencial: bloque unico de "la decision por proceso")
   └── C  FIND-585 + FIND-590 + FIND-587  (el bloque de la decision y el registro)
             C1  FIND-590  la decision fail-open  (debe ir PRIMERO: es la guarda)
             C2  FIND-585  el modo mixto silencioso multi-base
             C3  FIND-587  la rotacion contra una base viva

   Fase 2  (paralelo con la Fase 3, ficheros distintos)
   └── D  FIND-586  atar la identidad del fichero al AAD  (bump de formato)

   Fase 3  (depende de D)
   └── E  FIND-598  envelope por pagina + cache  (el mismo bump de formato)

   Fase 4  (guard de todo lo anterior)
   └── F  FIND-591  Miri con la feature `encryption` + align_to en vez de from_raw_parts
```

### 5.3 Detalle por paso

#### A — FIND-588 — la master key se imprime

- **Que rompe.** `Config` deriva `Debug` (`src/config.rs:702-703`) y
  `encryption_key` es un `String` sin zeroizar (`src/config.rs:958`; el mismo
  campo en `StorageCfg`, `src/config.rs:247`). Cualquier
  `tracing::info!("{:?}", config)`, un `dbg!` o un panic con `{:?}` imprime la
  master key en hex. Ademas hay **>=3 copias en heap** que nunca se borran: el
  `String` no pasa por `Zeroizing` (`src/config.rs:480` clona el campo;
  `src/storage/engine/init.rs:180` clona el `Config` entero otra vez).
  `ServerCfg` tiene la misma clase de fallo (`src/config.rs:264-265`,
  `api_key` `:268`, `jwt_secret` `:270`).
- **Ficheros que toca.** `src/config.rs` (los `Debug` manuales + el tipo del
  campo), `src/storage/engine/init.rs:180` (el clon).
- **Riesgo.** **Bajo**. Es un cambio de `Debug` (superficie observable, pero
  el unico consumidor de un `Debug` de `Config` es el diagnostico).
  ⚠️ **Hyrum's Law**: el `Debug` de `Config` **es** la superficie que un
  operador usa para volcar el estado ⇒ cambiarlo a "redactado" es un cambio
  de comportamiento observable, y alguien puede depender de ver la clave (por
  debugging). Se documenta en el changelog.
- **Tests que lo fijan.** `format!("{:?}", config)` **no** contiene la clave
  (el test que el backlog ya describe); `format!("{:?}", storage_cfg)` tampoco;
  `format!("{:?}", server_cfg)` tampoco.
- **Paralelismo.** **En paralelo con B** (toca `src/cli.rs` y
  `src/cli_handlers/encryption.rs`).
  ⚠️ `src/config.rs` **no** se toca en B ⇒ no hay colision.

#### B — FIND-589 — la master key via argv

- **Que rompe.** `--new-key <hex>` (`src/cli.rs:519`) es legible en
  `/proc/<pid>/cmdline` para cualquier usuario local y queda en el historial
  del shell. Ademas `cmd_encryption_rotate` **solo consulta el entorno**
  (`src/cli_handlers/encryption.rs:47-54`), asi que un operador con la clave en
  un fichero de configuracion no puede rotar, y la precedencia D2 (env → file
  → cmd) se pierde.
- **Ficheros que toca.** `src/cli.rs:515-524`, `src/cli_handlers/encryption.rs:47-83`.
- **Riesgo.** **Medio**: quitar `--new-key` es un **breaking change de CLI**
  (`feat!:`). El propio codigo ya ofrece la alternativa correcta
  (`--new-key-file`, `src/cli.rs:521-523`) y ya avisa en el doc del flag
  (`:516-518`).
- **Tests que lo fijan.** `--new-key` no existe (o esta deprecado con aviso);
  la precedencia de las 3 fuentes es la del ADR-0057 D2.
- **Paralelismo.** **En paralelo con A** (ficheros disjuntos). ⚠️ **C3** (FIND-587) **toca
  `src/cli_handlers/encryption.rs`** ⇒ **B y C3 no pueden ir en paralelo**: B
  reescribe la resolucion de la clave nueva, C3 reescribe la del registro.
  **B antes que C3**, o se fusionan.

#### C1 — FIND-590 — la decision de modo es fail-open

- **Que rompe.** La regla es "si **ningun** artefacto dice 'cifrado' ⇒ plaintext"
  (`src/storage/engine/init.rs:1010-1042`, `detect_encryption_mode`). Un atacante
  con escritura en `data/` borra `encryption.registry` + los ficheros de datos
  ⇒ el motor decide Plaintext ⇒ **escribe en claro** a partir de ahi.
  El lado fail-closed **si** esta bien: un artefacto que diga "cifrado" da
  `EncryptionKeyRequired` **antes** de abrir el backend
  (`src/storage/engine/init.rs:836-840`).
- **Ficheros que toca.** `src/storage/engine/init.rs:1010-1042`,
  `src/backends/kv_crypto.rs:447-455`.
- **Riesgo.** **Medio-alto**: cambiar la regla de decision puede **romper la
  apertura de bases legitimas**. El fix correcto (el que propone el backlog) es
  condicional: "si el backend **tiene datos** (`!is_empty`) y el registro falta
  ⇒ `EncryptionKeyRequired`", no "si el registro falta ⇒ error".
- **Tests que lo fijan.** "borrar el registro de una base con datos **no** la
  convierte en plaintext" (el test que el backlog ya describe); "una base
  vacia sin registro con una clave se cifra desde el primer byte" (el
  comportamiento actual, `src/storage/engine/init.rs:849-853`).
- **Paralelismo.** **Secuencial dentro de la Fase 1**: C1 define la regla que
  C2 implementa para el caso multi-base.

#### C2 — FIND-585 — el modo mixto silencioso multi-base

- **Que rompe.** Dos predicados distintos de "base nueva": el motor usa
  `is_new_database` ("hay ficheros en la raiz?", `src/storage/engine/init.rs:1050-1077`)
  y el backend usa `is_empty` ("hay valores?", `src/backends/kv_crypto.rs:447-455`).
  Hoy coinciden **solo** porque el motor registra su decision antes de abrir el
  backend (`src/storage/engine/init.rs:853`) y el backend corta por `KeyNotSet`.
  Ese corte desaparece cuando `process_decision()` es ambiguo
  (`src/crypto.rs:396-404`: con **mas de una base** en el proceso devuelve
  `None` ⇒ "las fuentes deciden") ⇒ el backend resuelve la clave por su cuenta,
  ve la base vacia y **cifra el backend KV in situ** ⇒ migracion implicita
  parcial (prohibida por ADR-0057 D7) + modo mixto + **sin aviso**.
- **Ficheros que toca.** `src/storage/engine/init.rs:849-853`, `:1050-1077`;
  `src/crypto.rs:365-404`; `src/backends/kv_crypto.rs:428-485`.
- **Riesgo.** **Alto**: el arreglo de raiz (un `EncryptionContext` tipado
  pasado del motor a los 4 artefactos, en vez de la **tabla global mutable**
  `PROCESS_KEYS`) es un **refactor de plumbing** que toca los 4 caminos de
  escritura. Es el hallazgo con mas superficie de la lista.
  ⚠️ **Coste de la tabla global** (`src/crypto.rs:365-376`): un `HashMap`
  global con `Mutex` por proceso ⇒ es un punto de contention y de
  no-determinismo. **Alternativa recomendada en el ADR**: el `EncryptionContext`
  tipado (que el backlog ya menciona en FIND-593 punto 2).
- **Tests que lo fijan.** "con dos bases en el proceso, la segunda **falla**
  con un error explicito (no se cifra a medias)"; "base plaintext vacia + clave
  ⇒ el backend **no** se cifra por su cuenta".
- **Paralelismo.** **Secuencial**: C1 (la regla) → C2 (la aplicacion).
  ⚠️ C2 toca `src/backends/kv_crypto.rs`, **igual que C1** ⇒ no paralelos.

#### C3 — FIND-587 — la rotacion no comprueba que la base este cerrada

- **Que rompe.** `vanta-cli encryption rotate` documenta que la base debe estar
  cerrada (`src/cli_handlers/encryption.rs:12-14`) pero **nada lo exige**. Si
  la base esta abierta en otro proceso, ese proceso mantiene el registro viejo
  en memoria y su proximo `persist()` lo **reescribe con el wrapping viejo**
  ⇒ **la rotacion se deshace sola**. Ademas el `.bak` se copia con
  `std::fs::copy` (`src/crypto/registry.rs:672-680`), **sin `fsync`** ⇒ el
  backup puede no estar en disco.
- **La revision de caos ya lo confirmo** (fila del backlog, 2026-10-21):
  `rotation_while_the_database_is_open_leaves_a_coherent_registry` — la
  rotacion **no se rechaza** contra una base viva; lo que el caos si garantiza
  es que el estado queda **coherente** (nunca hay DEKs que su KEK no pueda
  desenvolver, la clave vieja se rechaza, ningun dato se pierde).
- **Ficheros que toca.** `src/cli_handlers/encryption.rs:27-115`,
  `src/crypto/registry.rs:669-715`.
- **Riesgo.** **Medio**: tomar el lock exclusivo de `.vanta.lock` puede
  **chocar** con un motor vivo ⇒ hay que **devisar** el mensaje de error (no
  hay deadlock: el lock tiene que ser tentativo).
- **Tests que lo fijan.** "rotar con la base abierta en otro proceso **falla**
  con un error explicito" (el test de caos ya existe ⇒ hay que invertir su
  expectativa); "el `.bak` esta en disco (fsync) antes de que la rotacion
  devuelva `rotated: true`".
- **Paralelismo.** **Despues de B** (comparten `src/cli_handlers/encryption.rs`).

#### D — FIND-586 — el AAD ata el offset, no el fichero

- **Que rompe.** ADR-0057 D4 dice literalmente "un ciphertext movido de sitio
  **falla la autenticacion**" (`docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md:301-319`),
  pero el `slot` del VantaFile es el offset **dentro del segmento**
  (`src/storage/vfile_crypto.rs:170-177`, `:214-221`) y no hay `identifier`.
  Los 4 segmentos L0..L3 **comparten DEK y espacio de nonces**
  (`src/storage/engine/init.rs:60-80` resuelve un contexto y lo clona; el
  comentario de `src/storage/vfile_crypto.rs:23-25` lo dice). ⇒ un nodo de
  `vstore_L0.vanta`@X sustituye al de `vstore_L1.vanta`@X ⇒ el HNSW resuelve
  **el vector de otro nodo** ⇒ **resultados de busqueda silenciosamente
  incorrectos**. En el WAL, el `slot` es el ordinal intra-fichero y **todos los
  shards comparten DEK** (`src/wal_sharded.rs:390-393`) ⇒ lo mismo; la cadena
  de hashes **no lo detiene** (es SHA-256 **sin clave** sobre el ciphertext,
  recalculable por quien pueda escribir el fichero) y el replay no la valida.
- **Ficheros que toca.** `src/storage/vfile_crypto.rs:170-177`, `:214-221`;
  `src/wal.rs:480-484`, `:1346-1350`; `src/wal_sharded.rs:390-413`;
  `src/index/serialize/crypto.rs:214-222`, `:252-260`.
- **Riesgo.** **Alto (de formato)**: cambiar el AAD **invalida todos los sobres
  existentes** ⇒ hay que subir la version de los 3 formatos (VantaFile v3→v4,
  WAL v4→v5, imagen v9→v10) y **decidir que pasa con las bases cifradas ya
  escritas**. ⚠️ El backlog (fila `FIND-586`, `docs/dev/Backlog.md`) decia
  "los formatos v3/v4 son nuevos y sin publicar, se puede subir de version sin
  coste de migracion" — **eso ya no es cierto** desde W6 (2026-10-21):
  los formatos ya se escriben en disco. ⇒ **Decision del owner necesaria**
  (aceptar que las bases cifradas v3/v4 exijan export/import, o mantener un
  reader dual).
  ⇒ **Por eso D va antes que E**: comparten el bump y el AAD.
- **Tests que lo fijan.** "un blob de L0@X en L1@X **falla** la
  autenticacion"; "un frame de un shard en otro shard **falla**" (ambos tests
  ya estan descritos en el backlog).
- **Paralelismo.** Con A/B/C **si**: los ficheros son disjuntos
  (`vfile_crypto.rs`, `wal.rs`, `wal_sharded.rs`, `index/serialize/crypto.rs`
  vs `config.rs`, `cli.rs`, `init.rs`, `crypto.rs`).
  ⚠️ `src/wal.rs` **no** lo tocan A/B/C ⇒ sin colision.

#### E — FIND-598 — el envelope por pagina

- Ver §3. Depende de D (mismo bump de version, mismo AAD).
- **Paralelismo.** Con la Fase 1 **si** (ficheros distintos salvo
  `src/storage/vfile.rs` / `src/storage/vfile_crypto.rs`, que D tambien toca).

#### F — FIND-591 — Miri no corre la feature `encryption`

- **Que rompe.** (a) Todo el codigo `#[cfg(feature = "encryption")]` (el path de
  descifrado, el `AlignedBlob`, la variante `NodeBytes::Blob`) queda **fuera
  del detector de UB**: el job de Miri corre
  `--no-default-features --features "cli,fjall,memmap2,fs2"`
  (`.github/workflows/ci-rust.yml:683`) ⇒ **la feature `encryption` nunca se
  ejecuta bajo Miri**. Ademas el job de tree-borrows (`:695-701`) corre **el
  mismo** set de features ⇒ tampoco.
  (b) El invariante de alineacion de los `from_raw_parts` sobre `&[f32]` se
  sostiene hoy por **dos razones distintas y ninguna exigida en el sitio**: el
  guard central de `read_header` (que **no aplica a la rama cifrada**) y el
  `AlignedBlob` alineado a 8 B. El `debug_assert`
  (`src/storage/engine/get.rs:85`) **no corre en release**.
- **Ficheros que toca.** `.github/workflows/ci-rust.yml:679-701`;
  `src/storage/engine/get.rs:88-89`, `:596-601`, `:623-628`;
  `src/storage/engine/ops.rs:316-321`; `src/storage/archive.rs:382-387`,
  `:412-417`.
- **Riesgo.** **Bajo** para (a) (un step mas en el CI) y **medio** para (b):
  sustituir `from_raw_parts` por `align_to::<f32>()` cambia la semantica de los
  bordes (`align_to` devuelve el prefijo/sufio como `&[u8]` y hay que
  comprobar `words.len() == vector_len` — el patron que ya usa el repo en
  `src/index_port.rs:105-113` y `src/index/serialize/bytes.rs:146`).
- **Tests que lo fijan.** El job de Miri incluye la feature; los 4 sitios pasan
  sin `from_raw_parts` sobre `f32`.
  ⚠️ **Regla 2 (tolerancia cero a flaky)**: el step de tree-borrows ya es
  `continue-on-error: true` con `# CATEGORY: BEST-EFFORT`
  (`.github/workflows/ci-rust.yml:693-697`) ⇒ **no se toca**; el step nuevo
  tiene que ser **required**.
- **Paralelismo.** Con todo lo demas **si** (`.github/workflows/` no lo toca
  nadie mas), **pero** su valor depende de que el codigo bajo prueba ya este
  ⇒ en la practica va **al final**.

### 5.4 Resumen de paralelismo

| Pareja | ¿Paralelo? | Por que |
|---|---|---|
| A (FIND-588) ∥ B (FIND-589) | **Si** | `src/config.rs` vs `src/cli.rs` + `src/cli_handlers/encryption.rs` |
| A ∥ C1/C2/C3 | **Si** | `src/config.rs` vs `src/storage/engine/init.rs`, `src/crypto.rs`, `src/backends/kv_crypto.rs`, `src/cli_handlers/encryption.rs` |
| B ∥ C3 | **NO** | Los dos tocan `src/cli_handlers/encryption.rs` |
| C1 → C2 → C3 | **Secuencial** | Mismo predicado ("base nueva") y mismo fichero (`init.rs`, `kv_crypto.rs`) |
| D ∥ A/B/C | **Si** | Ficheros disjuntos |
| E | **Despues de D** | Mismo bump de version y mismo AAD |
| F | **Al final** | Es el guard |

---

## 6. FIND-559 — brief de decision para el owner

### 6.1 El problema, con evidencia

- La clausula `ROLE` se parsea en `src/parser/grammar.rs:267`
  (`opt(tuple((ws(tag("ROLE")), ws(string_literal))))`) y se propaga como
  `enforce_role` (`src/query.rs:544`, `:628`).
- La poda esta en `src/executor.rs:672-684`: si el nodo **no** tiene
  `relational["_owner_role"]` (la constante `FIELD_OWNER_ROLE`,
  `src/physical_plan/project.rs:12`) igual al rol pedido, **se descarta**
  (`continue`, `:682`) — salvo que el rol pedido sea `"admin"` (`:681`).
- **Nadie escribe ese campo**: el unico `grep _owner_role` en `src/` es un
  benchmark (`benches/high_density.rs:113`); los nodos de memoria lo reciben
  solo si el usuario lo pone a mano en la `metadata`
  (`src/sdk/serialization/mod.rs:621-623`).
- La proyeccion ya no lo borra: `SYSTEM_METADATA_FIELDS` incluye
  `FIELD_OWNER_ROLE` (`src/physical_plan/project.rs:28-34`), y el test
  `role_prune_still_applies_when_the_query_projects_fields`
  (`src/executor.rs:987-1001`) lo fija ⇒ **la parte de FIND-552 ya esta
  resuelta**.
- El documento **ya describe** el comportamiento fail-closed
  (`docs/api/IQL.md:119`).

⇒ El problema **ya no es semantico**: es que un `ROLE "user"` sobre una base
sin etiquetar devuelve **0 filas sin error ni aviso** — el patron de
`FIND-413` (resultado vacio silencioso).

### 6.2 Las opciones

#### Opcion A — fail-closed (poda) — **el comportamiento actual**

Un nodo sin `_owner_role` **se descarta** para todo rol distinto de `admin`.

- **Seguridad**: la mas fuerte. Un nodo sin rol es invisible para un rol
  restringido. El filtro **nunca amplia** el conjunto visible.
- **UX**: mala sin diagnostico. `ROLE "user"` devuelve 0 filas y el usuario no
  sabe si es que no hay datos, si el rol no existe, o si nadie etiqueto la base.
  **Es el unico problema real.**
- **Compatibilidad**: **cero cambios**. El texto de `docs/api/IQL.md:119`
  ya lo dice.
- **Coste**: un `ROLE` sobre una base sin etiquetar es inutilizable ⇒ el
  operador tiene que etiquetar (`metadata._owner_role`) o quitar la clausula.

#### Opcion B — fail-open (un nodo sin rol **pasa**)

- **Seguridad**: **debilita** el filtro. Un nodo sin rol es visible para
  **todos** los roles, incluido un "reader" que deberia ver solo lo suyo. En
  una base donde el etiquetado es parcial (unos registros con rol, otros sin),
  los sin etiquetar se filtran **hacia dentro**: es una **ampliacion**.
- **UX**: buena. `ROLE` se comporta como un filtro y no como una puerta.
- **Compatibilidad**: **cambio de comportamiento** para los despliegues que hoy
  dependen (quizas sin saberlo) de la poda. En el unico caso documentado
  (`docs/api/IQL.md:119`) el cambio es visible.
- **Coste**: hay que cambiar la poda (`src/executor.rs:672-684`), reescribir el
  texto de `docs/api/IQL.md:119`, y **revisar si algun otro sitio depende de
  la poda** (p. ej. el prune de `type` para los `SemanticSummary` de baja
  confianza, que sigue el mismo patron — `src/physical_plan/mod.rs:585-604`).

#### Opcion C — error explicito (fail-loud)

`ROLE` sobre una base cuyos registros **no** llevan `_owner_role` falla con un
error tipado en vez de devolver un conjunto vacio.

- **Seguridad**: la mas fuerte en diagnostico; el conjunto visible no cambia
  respecto de A (la poda sigue).
- **UX**: ruidosa pero **honesta**: el usuario sabe que su consulta no se puede
  evaluar. Obliga a etiquetar o a quitar la clausula.
- **Compatibilidad**: **rompe** toda consulta con `ROLE` sobre una base sin
  etiquetar (que hoy devuelve 0 filas sin error). Como no hay ruta de
  migracion de datos (la misma politica "no in-place migration" de ADR-0057
  D7), el operador tendria que **reexportar y reimportar** etiquetando.
- **Coste**: la mas alta de las tres, y el fallo es **falso positivo** en el
  caso legitimo de "esta base no usa roles y la clausula solo se usa para filtrar
  por un rol que nadie tiene" (un `ROLE` mal escrito, por ejemplo).

### 6.3 La recomendacion

**Opcion A (fail-closed, la de hoy) + arreglar el SILENCIO, no la semantica.**

El argumento, en orden de peso:

1. **La semantica fail-closed es la correcta para un filtro de seguridad.**
   Cambiarla a fail-open (B) **amplia** el conjunto visible de toda base
   desplegada ⇒ es una regresion de seguridad, no una mejora de UX. Y la
   Opcion C rompe el uso legitimo (un `ROLE` mal escrito, o una base que no
   usa roles) con un error que no se puede distinguir de un problema real.
2. **El problema de `FIND-559` no es "poda", es "silencio".** El backlog lo
   titula *"no es un bypass (fail-closed) pero si un **resultado vacio
   silencioso**"* ⇒ la fix que pide el propio backlog ("documentar
   `_owner_role` ... y tratar 'nodo sin `_owner_role`' como *pasa* (o como
   error explicito), no como *se descarta*") es **una de las dos opciones
   semanticas** — pero ninguna de las dos arregla el silencio: B y C cambian
   lo que se ve, no que se avise.
3. **El arreglo barato y correcto es un diagnostico**: contar los nodos podados
   en el executor (`src/executor.rs:672-684` ya recorre todos los candidatos)
   y, si se podo el **100 %** de las filas escaneadas con un rol distinto de
   `admin`, emitir **un aviso por consulta**:
   `"ROLE 'user' podo 412 de 412 registros: ninguno lleva el campo
   '_owner_role' — etiquetalos en la metadata o quita la clausula"`.
   Convierte el conjunto vacio mudo en un vacio que explica. **Cuesta un
   contador.**
4. **El texto de `docs/api/IQL.md:119` ya dice la verdad** ⇒ la accion de
   documentacion (que el backlog pide) **ya esta hecha**; lo que falta es
   *enfatizar* la consecuencia y *darle el remedio*.

**Lo que el owner tiene que decidir (una eleccion, no tres):**

> **¿Que debe pasar con un nodo que no lleva `_owner_role` cuando la consulta
> lleva `ROLE "<rol>"`?**
>
> - **(A) se poda** (fail-closed, hoy) — recomendado;
> - **(B) pasa** (fail-open);
> - **(C) es un error explicito** (fail-loud).

La recomendacion es **(A)**, mas el diagnostico del punto 3.

### 6.4 El texto exacto para `docs/api/IQL.md` (las 2 variantes)

**Variante A — la recomendada (fail-closed + diagnostico).** Reemplaza la
linea 119 de `docs/api/IQL.md`:

```markdown
| `ROLE "<role>"` | RBAC owner role filter. The record must carry an `_owner_role` field equal to the role. A record **without** that field is **pruned**: the filter is **fail-closed**, so it never widens the visible set. A database whose records were never tagged with a role therefore returns **no rows** for any `ROLE` clause except `"admin"` — this is the intended behaviour, not an error. When the clause prunes **every** scanned record, the engine emits a diagnostic naming the missing `_owner_role` field; tag the records (it is part of the record's `metadata`) or drop the clause. The field survives `FETCH` projection. |
```

Y **una nota nueva** debajo de la tabla de componentes (despues de la linea
122), para que el comportamiento se vea tambien sin leer la tabla:

```markdown
> **`ROLE` is fail-closed.** A record that does not carry `_owner_role` is
> **pruned** for every role except `"admin"`. On a database whose records were
> never tagged, `ROLE "user"` returns **no rows** — the filter is a gate, not
> a filter. Tag the records with `_owner_role` in their `metadata`, or drop the
> clause.
```

**Variante B — la alternativa (fail-open).** Reemplaza la linea 119:

```markdown
| `ROLE "<role>"` | RBAC owner role filter. The record must carry an `_owner_role` field equal to the role. A record **without** that field is **kept** (it is not pruned): the filter is **fail-open**, so it never hides a record it cannot classify. On a database whose records were never tagged with a role, `ROLE "user"` therefore returns **every** record. Use `ROLE "admin"` to restrict the result set. The field survives `FETCH` projection. |
```

Y la nota:

```markdown
> **`ROLE` is fail-open.** A record that does not carry `_owner_role` is
> **not pruned** — the clause filters the records it can classify and leaves
> the rest visible. On a database whose records were never tagged,
> `ROLE "user"` returns every record. To restrict the result set, use
> `ROLE "admin"` or tag the records with `_owner_role` in their `metadata`.
```

**En los dos casos**, si el owner elige C (error explicito), el texto es el de
A **sustituyendo** la frase "returns **no rows**" por "fails with
`Error::Iql` naming the missing `_owner_role` field", y anadiendo a la nota:
"the clause **cannot be evaluated** on a database whose records carry no
`_owner_role`; the query fails instead of returning an empty result."

---

## 7. Lo que este documento **no** decide

1. **Si el envelope por pagina se hace.** Depende del **paso 2** del plan
   (§3.9): si el paso 0 (sin cambio de formato) baja el p99 al umbral del 15 %,
   **no hace falta**. La decision la toma el dato, no este documento.
2. **El destino de las bases cifradas v3/v4 ya escritas** cuando FIND-586 suba
   las versiones (lector dual vs export/import). **Es decision del owner.**
3. **El limite de tamano de nodo** y, por tanto, el tamano de pagina (§3.1).
4. **Cuanto del cold start es el WAL y cuanto es la imagen HNSW** (§4.2) —
   hace falta un bench que los separe.
5. **Si el `ROLE` recurre a la Opcion A, B o C** (§6.3) — es del owner.

---

## 8. Incognitas (marcadas explicitamente, no inventadas)

| # | Incognita | Por que no se pudo resolver leyendo | Como se resuelve |
|---|---|---|---|
| 1 | **Cuanto del +71,2 % del p99 son las copias y cuanto es el AEAD** | No hay un bench que separe el AEAD del resto. W7 midio de extremo a extremo y su propio protocolo no es certificable (`§3.2` de la nota) | Un bench nuevo que mida el AEAD aislado (micro) + el de las copias, en la maquina dedicada |
| 2 | **Tope de dimension de vector** (y por tanto el tamano de pagina) | No encontre un tope documentado; `MAX_PERSISTED_NODE_BYTES = 128 MiB` (`src/storage/ops.rs:33`) es un tope de deserializacion | Buscar el limite en el producto (SDK / API) y fijarlo en el ADR |
| 3 | **Por que `cargo public-api` omite `read_full_vector`** del snapshot (`tests/api/public-api.txt:3719-3722`) | No inspeccione la herramienta; solo verifique que la linea **no esta** | Reproducir con la version de `cargo-public-api` que usa el repo y decidir si el gate se arregla o el metodo se anade a mano |
| 4 | **Cuanto del cold start es el WAL** | La nota de W7 los menciona juntos sin separarlos | Instrumentar el arranque (tiempo de `recover_valid_records_v4` vs tiempo de `open_image`) |
| 5 | **Si el `vector_offset` absoluto del layout de pagina rompe el lector** (dentro de la pagina, el payload esta en `vector_offset − base_de_la_pagina`) | Es una consecuencia del diseno propuesto, no del codigo actual | Resolverlo en el diseno: el lector de pagina resta `base_de_la_pagina`; test de round-trip con un nodo no en el inicio de la pagina |
| 6 | **Si el motor tiene un solo escritor por segmento** (para el modelo A: la pagina abierta en RAM) | El `File` esta detras de un `RwLock` (`src/storage/engine/init.rs:86`), pero no he auditado todos los caminos de escritura al vstore | Auditar los call sites de `write_node_to_vstore` (6 en `src/storage/engine/`) |
| 7 | **El comportamiento de `vanta-cli verify` y del backup con el formato v4** | `FIND-593` punto 4 ya dice que `verify` presenta como "tamper evidence" lo que es CRC + SHA-256 sin clave | Decidir con el resto del bloque D/E |

---

## 9. Referencias

- `docs/dev/reviews/2026-10-21-w7-cifrado-coste.md` — la medicion (W7).
- `docs/dev/architecture/adr/ADR-0057-encryption-at-rest.md` — el diseno del
  cifrado en reposo (D1 granularidad, D2 jerarquia de claves, D3 nonces,
  D4 AAD, D5 locks/atomicidad, D6 errores, D7 activacion, §5.2 el coste del
  zero-copy, §5.3 los limites declarados, §6 el gate del 15 %).
- `docs/dev/plans/2026-10-20-wiring-encryption-feature.md` — el plan W0..W7.
- `docs/dev/Backlog.md` — FIND-496, FIND-500, FIND-507, FIND-552, FIND-559,
  FIND-585..593, FIND-598.
- `docs/api/IQL.md` — la superficie de `ROLE` (lineas 99, 116, 119, 209).
- `SECURITY.md` — las Security Practices y la lista de limites (lineas 25-48).
- `.opencode/rules/memory-budget.md` — el cap declarado de las estructuras
  residentes (regla 4) y el budget de RAM (reglas 1-3).
