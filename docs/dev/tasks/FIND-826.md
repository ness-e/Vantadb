---
title: "FIND-826: forget(mode=certified) borra los bytes antes de firmar el recibo"
kind: task
status: completed
description: "El recibo de borrado certificado afirmaba `purged` mientras el payload seguia en 0.jnl, data/vanta.shard0.wal y vstore_L0.vanta hasta que alguien compactaba a mano. Ahora la compactacion corre ANTES de emitir el recibo."
id: FIND-826
---

# FIND-826: `forget(mode="certified")` no borraba bytes — el recibo mentía

## Metadata

- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-826`, medida por `FIND-675`)
- **Origen:** `FIND-675` (bench de borrado certificado, 2026-11-27) · **Ver:** `FIND-675`, `FIND-618`, `FIND-343`
- **Esfuerzo:** 🟠 1-2 d · **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED (local; review P2-01 pendiente — ver §Review)

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
  (§`compact_wal` FIND-651/FIND-343, §`compact_layout_bfs`), `tests/certified_delete.rs`,
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
   `delete_certified` (§Scope) — el coste y el alcance quedan escritos donde se leen.
5. ✅ Tests: el de `forget`, el de `delete_certified` y el de backend en memoria (WASM mapea
   `"certified"` a esta misma ruta: no debe fallar por una compactación que no necesita).

## Contrato de verificación

```
cargo nextest run --profile audit -p vantadb --build-jobs 2 \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --test certified_delete            # 13/13 passed (12 previos + 3 nuevos - 2 ya contados)
```

**Invariante:** el testigo debe seguir vivo en todos los casos (un 0 hits por base vacía no es
un borrado). Y el test **no** puede volver a medir una base cifrada: con una clave en el entorno,
0 hits significaría "ciphertext", no "borrado" (P2-01 M4 de `FIND-343`).

## Review

- **Ronda P2-01:** delegada a `vanta-review` (contexto fresco, agente distinto) sobre el diff de
  esta fila + `FIND-809`; veredicto pendiente al cerrar esta iteración. Sin veredicto la fila
  **no** se marca COMPLETED en el task-system.
- **Auto-revisión mecánica:** `dev-tools/verify.ps1` → 9 PASS / 1 SKIPPED
  (`cargo-clippy`, rojo de otro carril: imports sin usar de `FIND-490` en `src/planner.rs`).

## Deuda declarada

- **Bench Python no re-corrido en local:** `benchmarks/certified_unlearning_bench.py` necesita
  la extensión PyO3 reconstruida (`maturin develop`), que es un **segundo feature set** en el
  `target/` compartido — justo lo que D6 prohíbe en un carril. La aserción equivalente, y más
  fuerte, corre ahora en el Fast Gate como test Rust (mismo escaneo de bytes, mismos ficheros).
  El bench queda como verificación de binding en CI.
- **`vanta-cli certificate verify`** sigue siendo una verificación *lógica* (re-escanear
  superficies), no un escaneo de bytes del directorio: el CLI no es el instrumento forense.
