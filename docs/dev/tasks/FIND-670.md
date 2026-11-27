---
title: "FIND-670: la opcion (b) del cifrado ya esta implementada y medida, y el p99 sigue +43.2% - el camino que queda es (a)"
kind: task
status: completed
description: "Mapa del mecanismo con lineas y fechas: la cache LRU + zeroize (opcion b) existe desde 2026-10-21 y la remedicion post-Paso-0 de BENCHMARKS 22.4 sigue dando p99 +43.2% contra el objetivo <15%."
id: FIND-670
type: bug
---

# FIND-670 — p99 de cifrado `+30.7 %` a `+59.2 %` contra un objetivo de `<15 %`

- **Prioridad:** 🟠 Alta · **Origen:** auditoría 59 benchmarks 2026-11-22 (#17)
- **Creado:** 2026-11-22 · **Medición cerrada:** 2026-11-27
- **Estado:** 🟡 **CONTRATO CORREGIDO, sin número nuevo.** La opción (b) de la fila
  ya está en el árbol y ya está medida; el `cargo bench` no pudo correr en esta
  sesión y **no se inventa un número**.

## Lo que la fila pide y lo que ya existe

La fila ofrecía dos caminos:

- **(a)** vectores cuantizados SQ8/F16 en memoria para el paseo del grafo, con
  desencriptación AEAD **exacta solo en el re-ranking final** (`top_ef`); o
- **(b)** caché LRU de vectores desencriptados gobernada por `memory_limit_bytes`,
  con `zeroize` en eviction y drop.

**(b) está implementado, y no es de esta sesión:**

| Pieza | Dónde | Desde |
|---|---|---|
| Envelope AEAD **por nodo** (`seal_node`) | `src/storage/vfile.rs:1254` | W3 (`5a9f095e9`, 2026-10-20) |
| `DecryptCache` = `lru::LruCache<u64, Arc<AlignedBlob>>` | `src/storage/vfile.rs:213` | **`991d00c76`, 2026-10-21** ("5 copias por candidato reducidas a 3, y a 1 con caché caliente") |
| Presupuesto = `memory_limit_bytes`, `limit/16` clampeado a `[1 MiB, 256 MiB]` | `src/storage/vfile.rs:185-196` | ídem |
| `Zeroize` en drop del plaintext (`AlignedBlob`) | `src/storage/vfile.rs:410-426` | `e2a05ed94`, 2026-10-23 (FIND-618) |
| Decrypt por candidato en la lectura del vector | `read_full_vector` → `decrypt_blob` (`vfile.rs:1473`, `:1344`) | W3 |

## Lo que la medición registrada dice — y por qué es la base correcta

`docs/user/operations/BENCHMARKS.md` **§22.4** se titula literalmente «la
re-medición **post-Paso 0**» y es del **2026-10-21/22**, es decir **después** de
`991d00c76` (2026-10-21). Es la medición con la caché ya puesta:

| Métrica | Sin cifrar (med) | Cifrado (med) | Δ mediana | Umbral 15 % |
|---|---:|---:|---:|---|
| **search p50** | 6.5512 ms | 9.8823 ms | **+50,8 %** | superado |
| **search p95** | 11.5981 ms | 15.9925 ms | **+37,9 %** | superado |
| **search p99** | 16.3662 ms | 23.4357 ms | **+43,2 %** | **SUPERADO** |
| `insert` 25 000 nodos | 392.8 s | 435.4 s | +10,8 % | dentro |
| `flush` | 0.733 s | 0.804 s | +9,7 % | dentro |
| `cold start` | 1.433 s | 1.771 s | +23,6 % | superado |

Con el criterio fijado antes de ver los números (`max(claro) × 1.15 < min(cifrado)`
→ 16.7699 × 1.15 = **19.2854 < 22.0183**), §22.4.2 concluye **«GATE SUPERADO»**
en el sentido de *el presupuesto está violado*, con los tres pares intercalados
apuntando al mismo signo (+31,3 %, +58,2 %, +85,6 %).

**Conclusión que la fila no tenía:** la opción (b) **ya está aplicada y medida, y
el objetivo sigue violado**. No es que falte implementar la caché: es que una
caché no elimina el decrypt por candidato cuando el paseo del grafo visita más
nodos distintos que entradas caben en el presupuesto, ni cuando el `top_ef` de
re-ranking vuelve a tocar el ciphertext. El camino que queda es **(a)**.

## Por qué no hay número nuevo (y por qué eso es el entregable correcto)

`cargo bench -p vantadb --bench encrypted_p99 [--features encryption]` necesita un
build **release** del crate. En el momento de esta tarea otro carril tiene
`src/wal.rs`, `src/storage/vfile.rs` y `src/storage/vfile_crypto.rs` en edición y
la lib **no compila**:

```
error[E0425]: cannot find type `Path` in this scope  --> src\storage\vfile_crypto.rs:377
error[E0382]: borrow of moved value: `path`          --> src\wal.rs:920
error: could not compile `vantadb` (lib) due to 2 previous errors
```

Compilar `--features encryption` contra ese árbol mide un estado intermedio de
otro carril, no el de la rama. **No se publica un número que no se midió.**

## Contrato corregido (lo que queda, en orden)

1. **(a) SQ8/F16 en memoria para el paseo.** Mantener una representación
   cuantizada del vector en RAM (participa del paseo y del cálculo de distancia
   aproximada) y desencriptar AEAD **exacto sólo en el re-ranking de `top_ef`**.
   Es lo único que hace que el coste por candidato deje de ser un AEAD.
2. **Presupuesto medido, no asumido:** `decrypt_cache_budget()` = `limit/16`,
   clamp `[1 MiB, 256 MiB]`. Con 25 000 × 100d el corpus de vectores son ~10 MB;
   si el presupuesto efectivo no cubre el working set del paseo, la caché no
   puede ganar por diseño. Hay que **registrar la tasa de aciertos** por corrida
   (contador en `DecryptCache`) para que el próximo número explique el suyo.
3. **Re-medir §22 sobre GloVe-100 (25 000 × 100d)** con el árbol limpio, el mismo
   método de §22.4 (3+3 intercaladas, criterio fijado antes) y publicar p50/p95/p99.
   **Vara:** p99 dentro de `<15 %`, o el número real declarado sin llamarlo mejora.

## Verificación de esta tarea (mecánica, sin bench)

- `git log -S "struct DecryptCache" -- src/storage/vfile.rs` → la caché entra en
  `991d00c76` (2026-10-21), **antes** de la medición de §22.4 (2026-10-21/22).
- `grep` de las cinco piezas de (b) en `src/storage/vfile.rs` (tabla de arriba,
  con línea y hash).
- `cargo check -p vantadb` → **falla** por WIP ajeno (transcrito arriba), lo que
  documenta por qué no hay corrida nueva.

## Impacto mapeado (Regla 0)

- **Leído:** `benches/encrypted_p99.rs` (cabecera + doc del método),
  `src/storage/vfile.rs` (§cache 126-230, `read_full_vector` 1445-1490,
  `decrypt_blob` 1344), `docs/user/operations/BENCHMARKS.md` §22.4 completa.
- **Referencias hacia dentro:** `benches/encrypted_p99.rs` (consumidor),
  `BENCHMARKS.md` §22 (fuente de los números).
- **Veredicto:** **cero cambios de código** en esta fila. `src/storage/vfile.rs`
  es además el archivo que otro carril está editando ahora: ni se toca.

## Review (P2-01)

Tier: **Fast** (sólo documentación). Verify: hashes y líneas citadas verificables
con `git log -S` / `grep`; el fallo de compilación transcrito. **approve**.
