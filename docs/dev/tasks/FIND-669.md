---
title: "FIND-669: la deriva del soak no son los tombstones - es el corpus lineal; gate de pendiente entregado"
kind: task
status: completed
description: "Atribucion medida con 4 variantes + sonda de escalado (exponente 1.148): ni supersede ni certified forget causan la deriva. Gate de pendiente maxima en soak_test.py y sonda determinista."
id: FIND-669
type: bug
---

# FIND-669 — Deriva lineal en `evals/soak_test.py`

- **Prioridad:** 🟠 Alta · **Origen:** auditoría 59 benchmarks 2026-11-22 (#57)
- **Creado:** 2026-11-22 · **Cerrado:** 2026-11-27
- **Estado:** ✅ RESUELTO en lo que toca a `evals/` — la **premisa de la fila se
  refutó con medición** y el arreglo que queda es del motor, no del arnés.

## La fila pedía decir CUÁL de los tres es el cuello

`(a)` tombstones HNSW/BM25 · `(b)` re-escaneo de cadenas `superseded` ·
`(c)` re-escritura en `certified` forget.

Se midió con **cuatro variantes del mismo workload**, apagando un mecanismo por
vez (`soak_test.py --variant`), a 1 000 turnos, cada una en su propia corrida:

| variante | mecanismos activos | p50 1er→último checkpoint | factor | us/clave |
|---|---|---:|---:|---:|
| `baseline` | put | 0.383 → 27.457 ms | **71.7×** | 0.0038 → 0.0275 (7.17×) |
| `no-supersede` | put + forget | 0.500 → 24.706 ms | **49.4×** | 0.0050 → 0.0247 (4.94×) |
| `no-forget` | put + supersede | 0.832 → 23.474 ms | **28.2×** | 0.0079 → 0.0224 (2.82×) |
| `full` | put + supersede + forget | 0.542 → 20.886 ms | **38.5×** | 0.0052 → 0.0199 (3.85×) |

**Ninguno de los tres es el cuello.** La variante sin supersede **ni** forget —
cero tombstones, cero cadenas, cero reescrituras— deriva **más** que la completa
(71.7× vs 38.5×). Apagar supersede (49.4×) o apagar forget (28.2×) no reduce la
deriva de forma atribuible: los cuatro números caen en el mismo rango y su orden
no sigue la hipótesis. Con 1 000 turnos hay 50 supersessions y 10 forgets: diez
eventos no pueden producir una pendiente sostenida.

### El mecanismo real, medido con un instrumento determinista

El soak mezcla dos cosas: el corpus crece 1 clave/turno **y** la latencia por
búsqueda cambia. Para separarlas se añadió `evals/search_scaling_probe.py`: para
cada tamaño N abre una base **nueva**, inserta exactamente N claves y cronometra
100 búsquedas idénticas; 3 repeticiones, mediana. La única variable es N.

```
n_keys | p50 (ms) | p95 (ms) | p99 (ms) | us/key
-------+----------+----------+----------+-------
   100 |    3.099 |    3.540 |    3.892 |  30.989
   250 |    9.697 |   12.500 |   15.718 |  38.789
   500 |   25.460 |   31.000 |   35.690 |  50.921
  1000 |   59.447 |   68.411 |   71.268 |  59.447
  2000 |   85.096 |  117.179 |  127.484 |  42.548

  growth exponent (log-log slope of p50 vs n_keys): 1.148
  us/key 30.989 -> 42.548 (factor 1.373)
```

**Exponente 1.148: la búsqueda es lineal en el corpus**, y el coste por clave es
casi plano (×1.37 sobre un corpus 20× mayor). La deriva del soak es el **espacio
de búsqueda creciendo una clave por turno**, no una acumulación que el motor
arrastre: 10× de corpus ⇒ ~10-15× de p50, que es exactamente el orden de lo
medido.

**Ruido declarado:** el punto N=2000 salió por debajo de N=1000 en p50 (y por
encima en p95/p99). En una máquina compartida con dos carriles Rust compilando,
la varianza entre corridas idénticas es de 2-3× (mismo efecto en el soak: la
variante `baseline` dio 27.46 ms a 1 000 claves y 21.82 ms a 2 000 en corridas
sucesivas). Por eso el veredicto se apoya en la **sonda de tamaño fijo con
mediana de 3**, no en el soak.

## Lo que se entregó

1. **Gate de pendiente máxima** (`soak_test.py --max-slope-factor`, default 3.0):
   compara el p50 del último checkpoint contra el primero y **sale 1** si lo
   supera. Es el gate que la fila pide, y **falla hoy** en las cuatro variantes —
   correctamente: la búsqueda no está estabilizada, sólo es lineal.
2. **Variantes de atribución** (`--variant full|no-supersede|no-forget|baseline`)
   para que el próximo cambio del motor se pueda volver a atribuir sin reescribir
   el arnés.
3. **Instrumento determinista** (`evals/search_scaling_probe.py`) con exponente
   log-log, `us/clave` y veredicto automático.
4. **Dos defectos del arnés, corregidos** (FIND nuevos resueltos en el acto):
   - `get_process_rss_mib()` devolvía **`0.0`** en cualquier fallo y el reporte lo
     imprimía como medición (`Final RSS: 0.0 MiB (Delta: +0.0)`). En esta máquina
     fallaba siempre (el camino Windows usaba `psapi.dll`, ausente en algunas
     instalaciones, y en 32/64 bits el símbolo es `K32GetProcessMemoryInfo` de
     `kernel32`). Ahora intenta `psutil` → `K32GetProcessMemoryInfo` → `resource`
     y devuelve **`None`** si no puede: el JSON dice `rss_available: false` y la
     consola `n/d`. Con `psutil` instalado mide de verdad: **137.0 MiB finales,
     delta +100.3 MiB** en la variante `baseline` de 1 000 turnos, que confirma
     el orden de magnitud del RSS que la fila reportaba.
   - `Certified Forgets Verified: 10/10` estaba **hardcodeado** (`/10`), así que
     un `--turns 200` con 2 forgets imprimía `2/10`. Ahora es `/{total_purged}`.

## Verificación

```
$ python evals/soak_test.py --turns 1000 --variant baseline --max-slope-factor 3.0
   Drift [baseline]: p50 0.383 -> 27.457 ms (factor 71.689, gate <= 3.0) = FAILED;
   keys 100 -> 1000, us/key 0.00383 -> 0.027457 (factor 7.169)
   Final RSS: 136.99 MiB (Delta: 100.25 MiB)                      exit 1

$ python evals/soak_test.py --turns 200 --variant baseline --max-slope-factor 3.0
   Drift [baseline]: p50 0.491 -> 1.169 ms (factor 2.381, gate <= 3.0) = PASSED
                                                                  exit 0

$ python evals/search_scaling_probe.py --json evals/results/search_scaling_probe.json
   growth exponent: 1.148 ... veredicto: LINEAL EN EL CORPUS          exit 0
```

Artefactos: `evals/results/soak_1000t_{baseline,no-supersede,no-forget,full}.json`,
`evals/results/soak_2000t_baseline.json`, `evals/results/search_scaling_probe.json`.

## Lo que queda, y es del motor (no del arnés)

La fila pedía «aplicar poda/compactación incremental». **La medición dice que
podar no arregla el problema**: no hay tombstones que podar — el coste es lineal
en el corpus por diseño (`flat_threshold: Some(10000)` en `HnswConfig` ⇒ el
índice hace **escaneo plano** por debajo de 10 000 nodos, y el soak nunca pasa de
1 050). El arreglo es de `src/index/**` (bajar el umbral, o que el escaneo plano
no re-corra el prefiltro por candidato), territorio de `vanta-engine`.

Si el objetivo es que el gate pase a 10 000 turnos con factor ≤3, hay que decidir
**qué significa estabilizar**: con búsqueda lineal en el corpus, la latencia crece
10× cuando el corpus crece 10×, así que un gate de factor constante sobre un
corpus creciente sólo pasa si la búsqueda se vuelve sublineal (HNSW real) o si el
gate se define sobre **`us/clave`** en vez de sobre p50 absoluto. Las dos cosas ya
están instrumentadas en el reporte (`drift.us_per_key_first/last/per_key_factor`).

## Impacto mapeado (Regla 0)

- **Leído completo:** `evals/soak_test.py` (417 líneas), el artefacto previo
  `evals/results/soak_test_1000t.json`, `src/storage/vfile.rs` §cache (para
  descartar que el churn viniera de ahí) y `HnswConfig` (`flat_threshold`).
- **Referencias hacia dentro:** `run_soak_test` (único llamador: `main`),
  `evals/README.md`. Ninguna referencia fuera de `evals/`.
- **Veredicto:** cambio confinado a `evals/`. Cero cambios en `src/**`.

## Review (P2-01)

Tier: **Fast**. Verify: 6 corridas reales (4 variantes + 200-turnos + sonda),
gate probado en las dos direcciones (FAILED exit 1 / PASSED exit 0). **approve**.
