---
title: "FIND-496: el shipper de WAL no puede leer segmentos cifrados"
kind: task
status: in-progress
description: "WalShipper::ship_once abre los segmentos rotados con WalReader::open (sin clave): sobre un WAL cifrado falla con EncryptionKeyRequired a mitad de la replicacion, despues de haber enviado parte de los lotes. Contrato elegido: el shipping declara que no soporta bases cifradas y falla al inicio del ciclo, antes de cualquier POST, con un mensaje accionable."
---

# FIND-496: el shipper no puede leer segmentos v5 (cifrados)

## Metadata
- **Fuente:** `docs/dev/Backlog.md` (fila `FIND-496`)
- **Origen:** auditoría profunda v0.9.0 (2026-10-20), hueco detectado al implementar la wave W2
- **Esfuerzo:** 1h · **Prioridad:** 🟠 Alta · **Dep:** `FIND-407`
- **Creado / cerrado:** 2026-11-27
- **Estado:** ✅ COMPLETED

## Premisa (medida)

`src/wal_shipping.rs:126` abre cada segmento descubierto con `WalReader::open(segment_path)` — la variante **sin clave**. Con la wave W2 el WAL cifrado produce segmentos v5 (v4 al escribirse la fila), y `WalReader::open` los rechaza con `EncryptionKeyRequired` (`src/wal.rs:2147`). Consecuencias medidas en el código:

1. El error sale de `ship_once` **después** de haber enviado los lotes de los segmentos anteriores de la lista (el bucle `for segment_path in &segments` ya hizo POSTs) ⇒ replicación a medias.
2. El mensaje (`artifact: wal`) no dice qué hacer: el operador ve un error de clave en el shipper.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/wal_shipping.rs` (410 líneas), `src/wal.rs` (`WalReader::{open,open_with_key}`, `is_encrypted`, `segment_version`).
- **Referencias hacia dentro:** `ship_once` es llamado por `run_loop` (único call site) y por tests. `discover_segments` es interno.
- **Referencias entrantes:** `WalShipper` lo usa el binario cuando `wal-shipping` está activo; ningún test de integración lo construye.
- **Veredicto impacto:** **bajo**. Se añade una comprobación al inicio de `ship_once`; no cambia la firma pública ni el formato de nada.

## Spec

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | ¿Pasar la clave y re-cifrar en destino, o declarar no soportado? | (a) pasar la clave al reader y re-cifrar en destino — el shipper envía **records en JSON** (no frames), así que "re-cifrar" significa cifrar el HTTP body, y el destino necesitaría la DEK ⇒ **material de clave por la red**, explícitamente descartado por la fila · (b) **declarar no soportado y fallar al inicio** con mensaje claro | ✅ (b). (a) no es implementable sin violar la restricción de la fila; además el shipper no tiene contexto de cifrado en su API pública y añadirlo es una feature, no un fix. |
| 2 | ¿Dónde se detecta? | (a) en `run_loop` — el proceso sigue vivo y el error se loguea cada ciclo · (b) **al inicio de `ship_once`, sobre la lista de segmentos descubiertos, antes de cualquier POST** · (c) por segmento, al abrirlo | ✅ (b): es el punto en el que ya se conoce el conjunto a enviar y todavía no se envió nada. (c) es el bug actual. |
| 3 | ¿Cómo se detecta sin la clave? | (a) intentar abrir y mapear el error · (b) **leer el `format_version` del header cleartext** (`WalHeader`, 20 B) | ✅ (b): el marker de cifrado vive en el header en claro (ADR-0057 D1); no hace falta clave ni abrir el frame. |
| 4 | ¿Qué código de error? | (a) `Error::EncryptionKeyRequired` (el que sale hoy) · (b) `Error::wal_error` con el detalle accionable · (c) `Error::EncryptionUnsupported` | ✅ (b): (a) es exactamente lo que confunde; (c) es un variant que hoy no se construye en ningún sitio (`FIND-593` #3, fila ajena) y usarlo aquí sería cambiarlo de dueño. |

## Steps

1. ✅ Test RED `shipper_refuses_an_encrypted_wal_before_shipping_anything` (segmento rotado cifrado ⇒ `ship_once` falla nombrando el cifrado y **sin** escribir marker, es decir sin haber enviado nada).
2. ✅ `wal::segment_version` pasa a `pub(crate)`; `ship_once` comprueba la lista descubierta antes del bucle de envío.
3. ✅ GREEN + gates.

## Contrato de verificación

```
cargo nextest list --profile audit -p vantadb --build-jobs 2 --features encryption,wal-shipping \
  -E 'test(shipper) or test(wal_shipping)'    # > 0
cargo nextest run  ... mismo filtro ... --no-tests fail
```

**Invariante que no se puede romper:** una base **sin** clave sigue replicando igual (los segmentos v3 no llevan marker y el camino no cambia).

## Deuda declarada

- **Replicación de bases cifradas: no existe.** Este fix la declara imposible, no la implementa. El camino real (si se quiere) es enviar los frames **cifrados** tal cual y que el destino los escriba con su propia DEK — requiere que el destino comparta la DEK ⇒ clave por la red, que la fila prohíbe. Queda registrado como límite en `docs/user/operations/SECURITY.md` § Encryption at Rest, no como tarea implícita.

## Context Save Point

- **Hecho:** `src/wal_shipping.rs` (+ test), `src/wal.rs` (visibilidad de `segment_version`), este task file, `SECURITY.md` (límite), `Backlog.md` (retirada), `avance/activo/core-engine.md`.
