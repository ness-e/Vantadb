---
title: "FIND-607: verificación de las nueve hijas — cuatro premisas falsas"
kind: task
status: completed
description: "Cuatro de las nueve hijas de FIND-607 describen código ya borrado, dos están mal descritas y tres siguen vivas: tabla con veredicto, evidencia en HEAD y commit que fecha el error."
---

# FIND-607: verificación de las nueve hijas

`FIND-607` era una fila sola con 18 hallazgos P2 y la afirmación
*"re-verificado 2026-11-22"*. Esa re-verificación era **falsa**: de las nueve
hijas partidas el 2026-11-27, **cuatro** describen código que ya no existe
(`AG-04`, `SH-03`, `SH-04`, `UT-01`), **dos** están mal descritas (`UT-03`,
`UT-04`) y **tres** siguen vivas (`AG-03`, `AG-05`, `TU-04`).

Medición pura: sin compilar, sin tocar `src/**`. La evidencia es lectura de
`HEAD` + `git log -S` / `git show`.

## Tabla de veredictos

| ID | Afirmación de la fila | Veredicto | Evidencia en `HEAD` | Commit que fecha el error |
|---|---|---|---|---|
| `FIND-799` | `purge_expired_threads` descarta ids cuando la lectura falla (`get(...).ok().flatten()`) | **REFUTADA** | `src/agentic/thread.rs:352-374`: los dos sitios que traducen `get` a pertenencia al índice propagan con `?`; el comentario de `:362` lo declara | `647f4f27c` (`NOT-02`, 2026-11-03) — **24 días antes** de la fila |
| `FIND-798` | `create` deja nodo huérfano si falla la escritura del índice; reintentar duplica | **VIVA** | `src/agentic/thread.rs:255-256`: `insert` y luego `add_thread_id_locked`, sin compensación; `:448-455` puede fallar en `load_thread_ids`/`save_thread_ids`. El lock de `:225` (que sí añadió `647f4f27c`) serializa llamadores concurrentes, **no revierte el insert** | — |
| `FIND-800` | `GcWorker` muerto (`Embedded` pasa `gc = None`) y `send_message` desliza el TTL | **VIVA** | `src/sdk/builder.rs:371` y `:379` pasan `None`; `src/agentic/thread.rs:297-306` renueva `expires_at` con cada mensaje. `GcWorker` sólo se instancia en `src/gc.rs` (tests): ningún caller de producción lo usa | — |
| `FIND-801` | Un `put` shreddeado fallido apaga el fast path del namespace para siempre; metadata vacía nunca recibe fila | **REFUTADA** | No hay consumidor: `filter_verdict` y `has_omitted_fields` llevan `#[allow(dead_code)] // consumer lands with the read wiring` (`src/shred/mod.rs:231,248`); `put` es `#[cfg(test)]` (`:340`) y la escritura viva es `put_op` dentro del lote (`:361-370`). Sin consumidor no hay estado de namespace que apagar. La mitad de la **metadata vacía** es **latente, no falsa**: `encode_fields` devuelve `None` (`:294-297`) y el batch purga la fila vieja al quedarse sin metadata (`memory.rs:1749-1761`), pero sin lector no hay defecto medible | `928df0970` (2026-10-31) borró el consumidor (`lexical.rs`); `94855d647` (2026-11-19) y `5a45a1f8c` (2026-11-22) cerraron el resto — **27 días antes** de la fila |
| `FIND-802` | N+1 en el fast path shreddeado: una lectura al backend por nodo, con doc que promete "an order of magnitude faster" | **REFUTADA** | `src/sdk/search/lexical.rs:176-190`: `bitset_from_filters` proyecta namespace+filtros con hashes en memoria (`O(filters.len())`) y **no toca el backend**. La única lectura de filas shreddeadas que queda es `src/attestation.rs:209,317,574` (una por nodo, por diseño), y ningún docstring promete ya ese orden de magnitud | `928df0970` (2026-10-31) — **27 días antes** de la fila |
| `FIND-803` | El "ban" de un agente se revierte con su siguiente colisión | **REFUTADA** | `src/utils/confidence_metrics.rs:28-29` declara `slashed: HashSet<String>`; `:46-63` mantiene la confianza en `0.0` para un vetado; `is_slashed` (`:99-106`) consulta el set, y el levantamiento es explícito (`rehabilitate_origin`, `:108-117`) | `5fa148005` (`UT-01`, 2026-11-03) — **24 días antes** de la fila |
| `FIND-804` | `is_duplicate` + `mark_processed` no atómicos, con locks distintos | **CAMBIADA** | El primitivo atómico **ya existe**: `check_and_mark` (`:151-172`) hace check+set bajo **una** adquisición del lock de escritura, y producción lo usa (`src/sdk/api/memory.rs:993`). Lo que queda: el docstring (`:24-35`) sigue enseñando la composición racy, y el par `u128` no tiene forma atómica. **"Locks distintos" era falso**: ambos usan `bit_array` — la carrera son **dos adquisiciones del mismo lock**, no dos locks | `0488c7dc8` (2026-11-03) añadió `check_and_mark` — 24 días antes de la fila |
| `FIND-805` | `new(capacity_hint)` reserva sin tope (`abort` con `usize::MAX/2`) y no impone el hint | **CAMBIADA** | El abort está cerrado: `:15` define `MAX_BLOOM_BITS` (64 MB) y `:47-57` acota `optimal_bits` con `.min(MAX_BLOOM_BITS)`; el test `test_max_bloom_bits_bound` (`:299-303`) **asserta la cota**. El "descarta registros en silencio" también era falso: el único consumidor de producción (`src/sdk/api/memory.rs:993`) usa `check_and_mark` y ante un falso positivo re-verifica exacto (`:997-1011`) e **inserta igual** sin coincidencia (`:1053-1057`); `is_duplicate`, el patrón que sí descartaría, no tiene llamador fuera del módulo. Queda sólo que la saturación no tiene **contador agregado**, aunque cada put loguea `bloom_hit` (`memory.rs:1015-1030`) — severidad rebajada a Baja | `0488c7dc8` (2026-11-03) — 24 días antes de la fila |
| `FIND-806` | El modo Monitor no muestra nada y su mensaje miente (`refresh()` no-op) | **VIVA** | `src/tui/monitor.rs:33-37`: `refresh` sólo tiene comentarios; `:65-69` promete que las consultas "run in REPL (mode 3) or via the API" se ven ahí. `MonitorData::queries` (`:21`) no recibe **ningún** `push` en todo el repo (`src/tui/mod.rs:78,122` sólo lo construye y refresca) | — |

## Qué se hizo con las filas

- **Retiradas** (`FIND-801`, `FIND-802`, `FIND-803`): la fila y su sección salen
  del Backlog; el registro queda aquí y en `docs/dev/avance/activo/core-engine.md`.
- **Reescritas** (`FIND-804`, `FIND-805`): conservan sólo el defecto que sigue en
  `HEAD`, con la línea actual y sin la afirmación refutada. `FIND-805` bajó de
  Alta a Baja: lo que queda es un contador que falta, no una pérdida de datos.
- **Intactas** (`FIND-798`, `FIND-800`, `FIND-806`): el defecto está como se
  describe. Sólo se corrigió la **cita de línea**, que estaba desplazada
  (`thread.rs:209-212` es hoy el constructor, `:311-317` es `get`): el defecto
  vive en `:255-256` y `:297-306` respectivamente.

## El patrón, para la próxima sesión

Las cuatro refutadas comparten firma: **el audit es del 2026-10-21 y la fila se
escribió el 2026-11-27**, sin releer el código entre medio. Los **tres** commits
que cierran las **cuatro** refutadas son todos del 2026-10-31 / 2026-11-03 — es
decir, las filas nacieron citando código borrado semanas antes, igual que
`FIND-799`.

Comandos que lo demuestran (repetibles):

```powershell
git log --all --oneline -S 'ShreddedRowStore::get' -- src/sdk/search/lexical.rs   # 928df0970
git show 928df0970 -- src/sdk/search/lexical.rs                                    # borra el fast path con break
git log --all --oneline -S 'slashed: HashSet' -- src/utils/confidence_metrics.rs   # 5fa148005
git log --all --oneline -S 'MAX_BLOOM_BITS' -- src/utils/duplicate_prevention.rs   # 0488c7dc8
git log --all --oneline -S 'check_and_mark' -- src/utils/duplicate_prevention.rs   # 0488c7dc8
```

Regla que se deduce: **una fila citada no se re-verifica leyendo la fila; se
re-verifica abriendo la línea que cita**. Y para toda refutada, buscar el commit
con `-S` — sin fecha no se distingue "nunca fue cierta" de "envejeció bien".

## Review

- **Revisor:** `vanta-review` (contexto fresco, ronda 1, sobre el commit
  `1e9cc6429`).
- **Veredicto:** `changes-required` — no por una retirada injustificada (las tres
  se verificaron línea por línea), sino porque la reescritura de `FIND-805`
  reintrodujo una consecuencia falsa ("el consumidor descarta registros", que es
  justo el error que este documento viene a corregir).
- **Cómo se probó:** re-ejecución de la premisa en `HEAD` para cada ID
  (`src/sdk/search/lexical.rs`, `src/shred/mod.rs`,
  `src/utils/confidence_metrics.rs`, `src/utils/duplicate_prevention.rs`,
  `src/sdk/api/memory.rs`, `src/tui/monitor.rs`), `git show 928df0970 --
  src/sdk/search/lexical.rs`, `git show -s --format='%h %ad %s'` de los cuatro
  SHA citados, `node scripts/docs/check-links.mjs` (exit 0) y
  `node scripts/docs/check-docs.mjs` (exit 0).
- **Aplicado en ronda 2:** consecuencia de `FIND-805` corregida (un falso
  positivo cuesta dos lecturas, no un registro; `is_duplicate` no tiene llamador
  de producción), severidad Alta→Baja y `bloom_hit` sí queda en el log por put;
  conteo de cierres corregido a "tres commits / cuatro refutadas"; citas de línea
  de `FIND-798` y `FIND-800` alineadas al `HEAD`; esta sección añadida.
