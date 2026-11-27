---
title: "FIND-586: el AAD ata el ciphertext a su offset/ordinal, no a su fichero"
kind: task
status: in-progress
description: "El AAD no incluye la identidad del fichero: un registro copiado de L0@X a L1@X, o un frame del shard0@N al shard1@N, produce un AAD identico y el tag sigue autenticando. Fix: identifier = nombre del segmento/shard + bump de los formatos cifrados (VantaFile v3->v4, WAL v4->v5)."
---

# FIND-586: el AAD no ata el ciphertext a su fichero

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (la fila deriva de `FIND-500`; la review P2-01 + el audit de seguridad del 2026-10-21 la abrieron como fila propia — **no** existía una fila `FIND-586` en el Backlog cuando se abrió el lote, ver P2-01 L9)
- **Origen:** revision P2-01 (vanta-review, M-1) + audit de seguridad (vanta-audit, A-1) del 2026-10-21
- **Esfuerzo:** 1-2 semanas (la fila) · recortado a lo que cabe en el lote · **Prioridad:** 🟠 Alta
- **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED
- **Dep:** `FIND-500`, `FIND-498`

## Premisa (medida)

`ADR-0057 D4` dice literalmente *"un ciphertext movido de sitio falla la autenticación"*. Con `slot` intra-fichero, mover un registro entre **ficheros hermanos** al mismo offset produce un AAD idéntico:

```
AAD = artifact_id(2B) ‖ format_version(2B) ‖ epoch(8B) ‖ slot(8B) ‖ plaintext_len(8B)
```

L0..L3 comparten **un** `VFileCrypto` (una DEK, un espacio de nonces, un epoch) ⇒ un nodo de `vstore_L0.vanta@X` autentica en `vstore_L1.vanta@X` si el vector tiene la misma longitud. El HNSW guarda `(segmento, offset)` ⇒ resuelve **el vector de otro nodo** sin error.

En el WAL, todos los shards comparten `WalCrypto` (misma DEK, mismo epoch); un frame del shard 0 en el ordinal N autentica en el shard 1 en el ordinal N. La cadena VER-01 no lo detiene: es SHA-256 **sin clave** sobre el ciphertext (recalculable por quien pueda escribir el fichero) y el replay no la valida.

Medición RED (ambos tests fallan en `HEAD`): ver §Contrato de verificación.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/storage/vfile_crypto.rs`, `src/wal.rs` (§consts, `WalCrypto`, `WalWriter::append`/`batch_append_encrypted`, `WalReader`, `read_encrypted_frame`, `recover_valid_records_v4`), `src/storage/vfile.rs` (§header, `open_with_mode`, `seal_node`, `decrypt_blob`), `src/crypto/registry.rs` (`AadBuilder`), `src/wal_sharded.rs` (§`new_impl`).
- **Referencias hacia dentro:** `vfile_crypto::seal_node`/`open_node` tienen 3 call sites de producción (`vfile.rs:1299`, `vfile.rs:1323`, `vfile.rs:2104`) y 1 en `storage/archive.rs:229` (vía `File::seal_node`); `WalCrypto::frame` tiene 2 (`append`, `batch_append_encrypted`); `read_encrypted_frame` tiene 2 (`recover_valid_records_v4`, `next_record_encrypted`).
- **Referencias entrantes:** `AadBuilder` lo usan 4 módulos: `vfile_crypto.rs`, `wal.rs`, `backends/kv_crypto.rs` (ya lleva `identifier`), `index/serialize/crypto.rs` (ya lleva el path de la imagen). **Sólo los dos primeros cambian.**
- **Veredicto impacto:** **alto y contenido**. Cambia el formato en disco de dos artefactos; por eso el bump de versión es parte del contrato, no un efecto colateral.

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Qué identifica al fichero? | (a) el path completo — ata el registro a una ruta absoluta: mover el data_dir rompe la base · (b) **el nombre del fichero** (segmento / shard) · (c) un UUID persistido por segmento | ✅ (b): es la granularidad que el ataque necesita (hermanos en el mismo directorio), sobrevive a mover el `data_dir`, y no añade un fichero de metadatos nuevo. |
| 2 | ¿Cómo se deriva para el WAL, que se **renombra** al rotar? | (a) `file_name()` — un segmento rotado (`vanta.wal.<ts>`) dejaría de descifrar · (b) **el stem antes de `.wal`** (`vanta.shard0`), estable a través de la rotación · (c) el inodo | ✅ (b): mismo criterio que `scratch_path`; estable en rotación, distinto por shard. |
| 3 | ¿Migramos los formatos viejos? | (a) leer v3/v4 con un AAD sin identidad (compatibilidad) — deja el agujero vivo y crea dos AAD por versión (footgun) · (b) **rechazarlos con `IncompatibleFormat`** y bump · (c) reescribir la base al abrir | ✅ (b): los formatos cifrados son **nuevos y sin publicar** (`encryption` no está en `default`, `0.10.0` sin tag). Sin datos reales no hay migración que hacer; el rechazo es explícito, nunca "corrupt". |
| 4 | ¿Se sube también la versión del índice HNSW? | (a) sí, por simetría · (b) **no**: su AAD ya lleva el path de la imagen (`index/serialize/crypto.rs:107`) | ✅ (b): no tiene el defecto; tocarlo sería ruido. |

## Steps

1. ✅ Tests RED: `vfile_v3_node_blob_cannot_be_moved_to_another_segment` (un `VFileCrypto` compartido por L0 y L1, copia del registro) y `wal_v4_frame_cannot_be_moved_to_another_shard` (dos shards con el mismo `WalCrypto` clonado, graft del frame).
2. ✅ `AadBuilder::identifier` con el nombre del segmento en `vfile_crypto::{seal_node,open_node}`; `File` lo deriva de su `path`.
3. ✅ `WalCrypto::frame` + `read_encrypted_frame` reciben el identificador; `WalWriter`/`WalReader` lo derivan de su path con `wal_identifier`.
4. ✅ Bump `VFILE_ENCRYPTED_VERSION` 3→4 y `WAL_ENCRYPTED_FORMAT_VERSION` 4→5; los layouts previos se rechazan explícitamente.
5. ✅ GREEN + gates (`fmt`, `clippy`, `nextest` con el mismo filtro, `rustdoc`).

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 --features encryption \
  -E 'test(vfile_v3_node_blob_cannot_be_moved_to_another_segment) or test(wal_v4_frame_cannot_be_moved_to_another_shard)'   # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail                                                                   # pasa
cargo nextest run  --profile audit -p vantadb --features encryption -E 'test(vfile) or test(crypto) or test(wal)'          # sin regresiones
```

**Invariante que no se puede romper:** un registro legítimo sigue autenticando en **su** segmento y en **su** offset (los tests de roundtrip y de "movido de offset" lo pinchan); y el WAL rotado sigue siendo legible (el identificador se deriva del stem, no del nombre con timestamp).

## Deuda declarada

- **`FIND-498` no se cierra aquí:** atar el nombre del fichero **permite** sacar el AEAD del lock del escritor (el `slot` ya no necesita el ordinal bajo lock para garantizar la ligadura al fichero), pero mover el cifrado fuera del lock es un cambio de concurrencia con medición propia. Queda medido y listado, no implementado.
- **Bases cifradas pre-0.10.0:** hay que reescribirlas (borrar `data/` y recargar desde la fuente canónica). No existe camino de migración y no se va a inventar: el formato no se publicó.

## Context Save Point

- **Hecho:** `src/storage/vfile_crypto.rs`, `src/storage/vfile.rs`, `src/wal.rs`, este task file, `docs/user/operations/SECURITY.md` (§ Encryption at Rest: v4/v5 + la ligadura al fichero), `docs/dev/Backlog.md` (retirada de la fila), `docs/dev/avance/activo/core-engine.md`.
- **No hacer:** leer un v3/v4 cifrado "por compatibilidad" — reintroduce el agujero y dos formas de AAD por artefacto.

## Review

### P2-01 ronda 1 (contexto fresco, `vanta-review`, agente distinto) - **changes-required**, aplicado

VERDICT: changes-required con **0 CRITICAL / 0 HIGH / 7 MEDIUM / 9 LOW**. Confirmado sano: cobertura total del AAD (los 4 constructores pasan `identifier`), identidad estable en rotacion y shards, compaction cifrada coherente, rechazo en el open de vfile/WAL, FIND-496 bien acotado y FIND-507 en el orden correcto.

Aplicado en la misma ronda: **M1** (`verify_wal_file` rechaza el v4 legacy y reporta la version real), **M2** (`migrate`/`check_integrity` dicen "refused / rewrite", no "newer"), **M3** (§Diseno de FIND-513 reescrito con lo que el codigo tiene de verdad), **M4** (la medicion de 343 limpia la clave y asserta base plana), **M5** (`encryption_block` sin fail-open y sin segundo recorrido), **M6** (el Step 5 falso de 507 corregido), **M7** (el pin de rotacion, commiteado) y los LOW de prosa (L1, L6, L7, L8, L9).

No aplicado y declarado: **L2** (un hunk ajeno de `FIND-669` horneado en el commit de cripto: ruido de carril, no enmendable sin `--amend`), **L3** (fila `FIND-500` colgando: es de otra fila, fuera de las 5 del lote) y **L4** (la deuda de crypto-shredding queda en el §Deuda de 343 en vez de como fila nueva: los IDs los asigna el orquestador).

