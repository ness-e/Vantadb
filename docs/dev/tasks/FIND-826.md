---
title: "FIND-826: forget(mode=certified) borra los bytes antes de firmar el recibo - IMPLEMENTADO, delta abierto"
kind: task
status: in-progress
description: "La compactacion se ejecuta ahora ANTES del recibo (medido: 0 bytes en el escaneo crudo, testigo vivo). El review P2-01 encontro el techo: el motor trata como no-fatal un fallo de compactacion del backend y el certificado atesta el vector desde una constante, asi que la certificacion byte-level bajo concurrencia sigue abierta."
id: FIND-826
---

# FIND-826: `forget(mode="certified")` no borraba bytes — el recibo mentía

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-826`, medida por `FIND-675`)
- **Origen:** `FIND-675` (bench de borrado certificado, 2026-11-27) · **Ver:** `FIND-675`, `FIND-618`, `FIND-343`
- **Esfuerzo:** 🟠 1-2 d · **Creado:** 2026-11-27
- **Estado:** 🟡 **PARCIAL** — implementado y medido en secuencial; **delta abierto** (ver §Delta). Por D4 la fila **no** se cierra.

## Premisa (re-medida en HEAD, no re-derivada)

`certified` era **byte-idéntico a `hard`**: tras `forget(certified)` + `flush()` + `close()`
el marcador del payload seguía en `0.jnl` y `data/vanta.shard0.wal`, y el vector en
`vstore_L0.vanta`. Sólo tras `compact_wal()` + `compact_layout()` el escaneo daba 0.

Medición propia sobre `HEAD` (test nuevo, sin cambios de código), con testigo vivo:

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test certified_delete -E 'test(forget_certified)'
→ panicked: certified forget left the payload on disk in
  ["…\\0.jnl", "…\\data\\vanta.shard0.wal"]
→ Summary [1.113s] 2 tests run: 1 passed, 1 failed
```

## Contrato elegido: (a) ejecuta la compactación antes del recibo

Las dos opciones del contrato eran (a) ejecutar la compactación antes de emitir el recibo, o
(b) declarar en el recibo que el borrado físico es diferido y nombrar el comando.

**Se elige (a)**, y la razón es de fondo, no de conveniencia:

1. **La fila pide `certified_assertion: PASS` justo después del `forget`.** La opción (b) exige
   mover la aserción después de una compactación explícita: el bench pasaría a medir el comando
   de mantenimiento, no el borrado certificado. Quedaría verde sin que `certified` signifique
   nada distinto de `hard` + un pie de página.
2. **GDPR Art. 17 se juzga con un escaneo de bytes, no con un campo `status`.** Un recibo que
   declara "diferido" es documentación de una obligación incumplida. `certified` es la ruta
   legal: tiene que ser el modo caro, y serlo **antes** de firmar.
3. **La ruta ya existía**: `compact_wal()` (que rota y redacta los segmentos, `FIND-343`) +
   `compact_layout()` son exactamente las dos llamadas que el bench necesitaba para llegar a 0.
   No se inventó mecanismo nuevo.

Coste aceptado y declarado: un `forget(certified)` sobre una base persistente es una operación
de clase mantenimiento (proporcional a la base, no al conjunto borrado). Es el precio de un
recibo que aguanta un `grep` en crudo. Una base en memoria no tiene bytes que reescribir.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/sdk/api/memory.rs` (§`delete_certified`, `forget`,
  `delete_inner`, `purge_lock`), `src/attestation.rs` (§`build_certificate`, `declared_limits`),
  `src/sdk/api/admin.rs` (`compact_wal`, `compact_layout`), `src/storage/engine/maintenance.rs`
  (§`compact_wal` FIND-651/FIND-343, §`compact_layout_bfs`), `src/backends/fjall_backend.rs`
  (§`Compactable::compact`, que el review pide), `tests/certified_delete.rs`,
  `benchmarks/certified_unlearning_bench.py`, `docs/dev/tasks/FIND-343.md` (patrón de canario).
- **Referencias entrantes:** `vanta-cli delete --attest` → `cmd_delete_certified` →
  `Embedded::delete_certified` (cubierta por el mismo fix); PyO3/WASM/Node/MCP →
  `Embedded::forget` (cubiertos). Firma pública sin cambios
  (`tests/api/public-api.txt` intacto: `delete_certified(&self,&str,&str)`, `forget(&self,&str,ForgetTarget,ForgetMode)`).
- **Veredicto:** cambio confinado a `src/sdk/api/memory.rs`; sin tocar `src/storage/**` ni
  `src/wal*` (fuera de zona). El único efecto lateral es coste/latencia en las dos entradas
  certificadas, declarado en el rustdoc de `forget` y de `delete_certified`.

## Steps

1. ✅ Test RED: escaneo de bytes recursivo (payload + bytes f32 del vector **tal como se
   almacena** + testigo que debe sobrevivir) tras `forget(certified)`+`flush`+`close`.
   Falla en HEAD en `0.jnl` + `vanta.shard0.wal`.
2. ✅ Fix: `forget`'s brazo `Certified` = `delete_inner` por clave → **una** compactación
   (`compact_wal` + `compact_layout`) si algo se borró → recién entonces los certificados,
   vía el helper privado `purge_certificate`.
3. ✅ `delete_certified` (superficie de una clave, la que usa el CLI) también compacta antes de
   atestar: el hueco no se mueve de puerta.
4. ✅ Rustdoc de `forget` (§`Certified` erases bytes before attesting them) y de
   `delete_certified` (§Physical erase) — con el **techo declarado**.
5. ✅ Tests: el de `forget`, el de `delete_certified` y el de backend en memoria (WASM mapea
   `"certified"` a esta misma ruta: no debe fallar por una compactación que no necesita).
6. ✅ Review P2-01 (ronda 1, `vanta-review`): **changes-required** → aplicado lo que cae en
   zona; el resto pasa a §Delta (por D4 la fila queda abierta).

## Contrato de verificación

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test certified_delete            # 13/13 passed (perfil audit: test-threads = 2)
```

**Honestidad sobre el runner (review M2):** con `cargo test` en paralelo por defecto (12 hilos)
el **mismo** binario deja 3 de 13 en rojo, cada uno con el payload vivo en su propio `0.jnl`, y
pasan todos con `--test-threads=1`. Reproducido por el ejecutor de esta fila:

```
cargo test -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test certified_delete
→ test result: FAILED. 10 passed; 3 failed   (6.79 s)
   delete_certified_erases_payload_bytes_too                       → 0.jnl
   forget_certified_erases_payload_and_vector_bytes_with_witness_alive → 0.jnl
   physical_erasure_no_payload_bytes_after_full_maintenance        → ¡el test PREEXISTENTE de FIND-343!
```

**El tercer fallo es la prueba de propiedad del defecto:** el canario de `FIND-343`
(`tests/certified_delete.rs:252`, vacuum + merge + `compact_wal` + flush + close) falla con el
mismo síntoma bajo 12 hilos, y esa ruta **no la tocó esta fila**. El que omite la reescritura es
el motor bajo concurrencia (D1), no el `forget` certificado. No es el test el que falla por
ruido: es la compactación la que se omite en silencio. El gate del repo usa el perfil `audit`
(`test-threads = 2`, D7/R3); la certificación de este binario bajo 12 hilos es evidencia de D1.
**No se debilita el test** (ni `#[serial]` ni `#[ignore]`): un test que sólo pasa a 2 hilos
documenta un techo, y esconderlo sería el defecto que `FIND-664` describe.

**Invariante:** el testigo debe seguir vivo en todos los casos (un 0 hits por base vacía no es
un borrado). Y el test **no** puede volver a medir una base cifrada: con una clave en el entorno,
0 hits significaría "ciphertext", no "borrado" (P2-01 M4 de `FIND-343`).

## Review

### P2-01 ronda 1 (`vanta-review`, contexto fresco, agente distinto) — **changes-required**

Confirmado sano: el orden `delete → compactar → certificar`, el `compact once` en `forget`, el
testigo vivo, y que `compact_wal`/`compact_layout` no toman `purge_lock` (sin inversión de orden).

| # | Sev | Hallazgo | Estado |
|---|---|---|---|
| C1 | CRITICAL | `src/storage/engine/maintenance.rs:182-191` degrada a `warn!` el fallo de compactación del backend ("non-fatal"), así que el recibo puede firmarse con los bytes vivos. Evidencia del review: `cargo test --test certified_delete` (12 hilos) → 3/13 en rojo, cada uno con el hit en su `0.jnl`; verde con `--test-threads=1` | **DELTA D1** (fuera de zona: `src/storage/**`) |
| C2 | CRITICAL | `src/attestation.rs:257-266`: la superficie `vector_store` es un `residues: 0` **hard-codeado** con evidencia "physical bytes remain until segment compaction (declared limit)"; el certificado no re-mide el vstore tras compactar | **DELTA D2** (fuera de zona: `src/attestation.rs`) |
| H1 | HIGH | `vanta-memory/src/core/record/cascade_forget.rs:375,409,462,500,528`: 5 bucles llaman `delete_certified` por clave ⇒ un cascade forget es O(N·DB) con N reescrituras completas | **DELTA D3** (fuera de zona: `vanta-memory/**`) |
| H2 | HIGH | `delete_certified` no atómico: si la compactación falla, el registro ya está borrado y la llamada devuelve `Err` sin certificado; el reintento da `not_found` | **APLICADO**: `compact_purged()` devuelve un error que dice explícitamente "el registro ya está borrado lógicamente; falló la reescritura física; re-corré `compact_wal()`+`compact_layout()`" |
| M1 | MEDIUM | el §Scope de `delete_certified` decía "for a byte-scan receipt use `forget`" mientras el cuerpo ya compactaba — doc autocontradictoria | **APLICADO**: §Physical erase reescrita con lo que el código hace + el techo declarado |
| M2 | MEDIUM | este task file afirmaba `verify.ps1` verde sin matizar que el contrato citado está rojo con el runner por defecto | **APLICADO**: §Contrato de verificación |
| L1 | LOW | `ensure_edge` perdió `#[tracing::instrument]` en el split de `FIND-809` | **APLICADO** en `FIND-809` |

## Delta abierto (lo que la fila NO cierra — por D4)

**D1 — la compactación del backend es best-effort y el SDK no puede distinguir "hecha" de
"omitida".** `compact_wal` (`src/storage/engine/maintenance.rs:176-191`) envuelve
`Compactable::compact()` en un `match` que loguea y sigue. En secuencial reescribe
(`FjallBackend::compact`, `src/backends/fjall_backend.rs:490`) y el escaneo da 0; bajo carga
concurrente la reescritura se omite/falla y **el recibo sale igual**. Medido en esta fila:
`cargo test --test certified_delete` (12 hilos) → 10 passed / 3 failed, y uno de los tres es el
canario **preexistente** de `FIND-343` ⇒ el defecto es del motor, no del path certificado (que
sólo lo hereda). Contrato para cerrarlo: `compact_wal` devuelve el resultado de la compactación
del backend (o un `MaintenanceReport { backend_compacted: bool }`) y el path certificado **falla
cerrado** cuando no se reescribió. Toca `src/storage/engine/**` + `src/backends/**` → dueño:
Arch/Engine. **Verificación:** `cargo test -p vantadb --test certified_delete` (12 hilos, que es
donde hoy se ve) → 13/13.

**D2 — el certificado no mide bytes.** `vector_store` es una constante con `residues: 0`
(`src/attestation.rs:257-266`) y `verify_purge_certificate` re-escanea la misma constante.
Contrato: la superficie `vector_store` (y `wal`) se calcula tras la compactación, o el
certificado declara `action: "not-assessed"` con el motivo. Dueño del archivo: attestation
compartido (no es zona de esta fila). **Verificación:** un test que fabrique un residuo conocido
en el vstore y exija que el certificado **no** lo llame 0.

**D3 — `cascade_forget` compacta N veces.** Debe hacer **un** `forget` con el conjunto completo
(que ya compacta una vez) en vez de N `delete_certified`. Dueño: `vanta-memory/**`.
**Verificación:** un test de cascade con 5 claves que cuente 1 compactación, no 5.

**No cerrada la fila, aunque el defecto original esté corregido en secuencial.** El documento
sigue siendo la prueba: mientras D1 y D2 estén abiertos, un recibo `purged` bajo concurrencia
no está instrumentado para ser verdad.
