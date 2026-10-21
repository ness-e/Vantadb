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
- **Estado:** ⬜ PENDING
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
| `lastAction` | (se actualiza en cada step) |
| `result` | ⬜ PENDING |
| `nextAction` | Step 1 |
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
| Incógnitas abiertas (uphill) | 2 (el reparto de copias y el tope de dimension) |
| Pendientes de ejecución (downhill) | 6 |
| % completado | 0% |

## Steps

1. **Medir el reparto de copias** (no suponerlo): contar las copias del payload de un nodo en el
   path cifrado (perfil o contador temporal) y **verificar** cuales sobran. Dejar el numero en el
   informe.
2. **Test primero**: un test que fije que el resultado de una busqueda (ids, orden y scores) **no
   cambia** con el fix, y otro que fije que la cache **no** devuelve un buffer caducado (que el
   consumidor pueda usar el vector despues de que la cache lo haya reemplazado).
3. **Fix**: (a) `read_header` cifrado que **no** copie el payload entero (leer solo la cabecera del
   blob descifrado, o cachear la cabecera por pagina); (b) `read_full_vector` con `align_to::<f32>()`
   en vez del bucle `chunks_exact(4)`; (c) cache de nodos descifrados con cap derivado del
   `memory_limit`, invalidada por escritura, y con el `Arc` poseido por el consumidor.
4. **Verificar**: fmt, clippy `-D warnings`, la suite del lib completa, los tests nuevos, y el
   golden `tests/api/public-api.txt` **sin regenerar** (la API publica no cambia).
5. **Re-medir**: `cargo bench -p vantadb --bench encrypted_p99` (variante cifrada) con la maquina en
   silencio y la escala mas alta que quepa ⇒ comparar el delta con la medicion de W7 y **publicar
   el numero** en `docs/user/operations/BENCHMARKS.md` §22 y en la nota de W7, con comando,
   entorno y fecha.
6. **Actualizar `FIND-598`** con el resultado y **anadir la entrada de avance**.

## Review (P2-01)

- **Reviewer:** (pendiente — agente DISTINTO del implementador)
- **Veredicto:** (pendiente)
- **Fecha:** (pendiente)

## Notas

- `src/storage/vfile.rs` y `src/index_port.rs` son de la feature de cifrado: **no los toques sin
  coordinar** con las otras tasks de cifrado (`FIND-585`..`FIND-591`), que estan sobre los mismos
  ficheros.
- Si al final hay que tocar el trait publico `VectorStoreRef` (o el `.d.ts` de los bindings), eso
  **ya no es el Paso 0**: es el envelope por pagina ⇒ para, documenta y escala la decision al owner.
