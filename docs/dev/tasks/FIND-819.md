---
title: "FIND-819 — la matriz daba por stale el doc que ya estaba corregido"
kind: task
status: completed
description: "`docs/feature-matrix.md:31-37` daba por stale tres puntos que `FIND-808` ya había corregido en `FEATURES.md`: la matriz narraba el fix del documento que la cita como si siguiera roto. Se reescribe el párrafo (7 líneas, sin mover anclas) para afirmar los hechos medidos en `Cargo.toml:156` y `vantadb-python/Cargo.toml:25` sin narrar la historia de otro fichero."
tags: [vantadb, cargo, features, docs]
---

# FIND-819 — la fuente de verdad quedó stale porque el doc que la cita se corrigió

> **Carril:** documentación, **sin compilar Rust**. Escritura: este task file ·
> `docs/feature-matrix.md` · `docs/dev/avance/activo/docs.md`.
> **Prohibido y respetado:** `docs/dev/architecture/FEATURES.md` (recién cerrado por `FIND-808`) ·
> `docs/dev/Backlog.md` (`FIND-818` editándolo fila a fila) · `Cargo.toml` · `Cargo.lock` ·
> `.github/**` · `src/**` (`FIND-798` compilando) · `vanta-memory/**` · `vantadb-mcp/**` ·
> `dev-tools/**` · `.opencode/**` · `docs/dev/plans/**` · `docs/index.md`.

## La premisa heredada era falsa en su forma, cierta en su sustancia

La fila decía «la matriz describe como stale tres puntos». Medido contra `HEAD`: la matriz
**ya afirmaba los hechos correctos** (cinco flags, `default + rayon`). Lo que había quedado
falso es la **atribución**: las tres frases que declaraban a `FEATURES.md:32`/`:51` como
stale, y a su workflow como inexistente, dejaron de ser ciertas en el instante en que
`FIND-808` cerró. Tres afirmaciones sobre otro fichero, no tres datos propios.

No se corrige «de memoria»: cada dato se releyó de su fuente primaria.

## Medición contra `HEAD` (rama `develop`, 2026-11-27)

| Punto | Matriz decía (`:31-37`) | Medido | Ahora dice |
|---|---|---|---|
| (a) `default` | cinco flags ✅ **pero** «despite the stale aggregate row in `FEATURES.md:32`» | `FEATURES.md:32` dice cinco y cita el mismo `Cargo.toml:156`: **no está stale** | cinco flags, fuente `Cargo.toml:156`, sin mención a otro doc |
| (b) wheel | «does **not** pass `default-features = false` … `default + rayon` ✅ **pero** «Same class of drift at `FEATURES.md:51`» | `FEATURES.md:51` ya declara `default` encendido + `rayon`: **no hay drift** | `vantadb-python/Cargo.toml:25` citado por línea; neto `default + rayon` |
| (c) workflow | «it also cites a `ci-rust-10.yml` that no longer exists» | `rg 'ci-rust-10' docs/dev/architecture/FEATURES.md` → **0** desde `FIND-808` | la cadena desaparece del fichero; queda la regla «workflows por el **filename real**, nunca por un basename renombrado» |

Comandos crudos:

```
$c = Get-Content Cargo.toml; $c[155]
default = ["cli", "fjall", "memmap2", "fs2", "roaring"]                  # línea 156, cinco

(Get-Content vantadb-python/Cargo.toml)[24]
vantadb = { workspace = true, features = ["fjall", "memmap2", "rayon"] } # sin default-features = false

rg -c 'ci-rust-10' docs/dev/architecture/FEATURES.md    # exit 1 → 0 matches
rg -c 'ci-rust-10' docs/feature-matrix.md               # exit 1 → 0 matches (tras el edit)
```

## Impacto mapeado (Regla 0)

- **Leídos completos:** `docs/feature-matrix.md` (123 líneas) · `docs/dev/architecture/FEATURES.md`
  (110 líneas, solo lectura) · `docs/dev/tasks/FIND-808.md` (cabecera + medición) ·
  `Cargo.toml:150-205` · `vantadb-python/Cargo.toml:15-35`.
- **Referencias hacia dentro de la matriz:** ninguna cita por **línea** desde fuera salvo
  `docs/dev/Backlog.md:205` (`FIND-215`) → `feature-matrix.md:55,75-76`. `FEATURES.md` la cita
  por **sección** (`:32`→§1-2, `:41`→§4, `:66`→§2, `:92`→§4). Ambas formas se preservaron.
- **Referencias salientes que se tocan:** `FEATURES.md:32` y `FEATURES.md:51` dejan de citarse
  (eran las frases falsas). `feature-matrix.md:79`→`FEATURES.md:48` y `:80`→`FEATURES.md:60`
  **no se tocaron** y siguen válidas (medidas contra `HEAD`).
- **Veredicto:** impacto bajo y contenido: 7 líneas reescritas por 7, cero desplazamiento.

## El truco del carril: 7 líneas por 7 líneas

Un párrafo más corto habría movido `:55` y `:75-76` — las dos citas por línea que sí existen
fuera del fichero — y todas las secciones que `FEATURES.md` referencia. El reemplazo mantiene
**exactamente 7 líneas** (`:31-37`), así que §2 sigue abriendo en `:42` y §3 en `:65`.

Es la misma lección de `FIND-808` por el otro lado: allá hubo que *restaurar* tres anclas
porque el doc se comprimió; acá se preservan por construcción.

## Steps

| # | Step | Estado |
|---|---|---|
| 1 | Medir los tres puntos contra `HEAD` (no contra la fila) | ✅ |
| 2 | Reescribir `:31-37` — 7 líneas, una por punto, sin narrar otro fichero | ✅ |
| 3 | Verificar anclas **después** del edit (`:55`, `:75-76`, `## N` headings) | ✅ |
| 4 | Contract mecánico: `check-links` · `check-docs` · `rg ci-rust-10` · listas de flags | ✅ |
| 5 | Avance + task file (el commit lo hace el orquestador, no este step) | ✅ |

## Verificación

| Check | Resultado |
|---|---|
| `node scripts/docs/check-links.mjs` | **exit 0** (0 broken markdown) |
| `node scripts/docs/check-docs.mjs` | **exit 0** (`GATING: all clear`) |
| `rg -c 'ci-rust-10' docs/feature-matrix.md` | **exit 1 → 0** |
| Lista matriz `:31` vs `Cargo.toml:156` | `cli, fjall, memmap2, fs2, roaring` ≡ `["cli", "fjall", "memmap2", "fs2", "roaring"]` |
| Ancla `:55` (citada por `FIND-215`) | sigue siendo la fila 5 `release-binary` |
| Anclas `:75-76` (citadas por `FIND-215`) | siguen siendo las filas de precedencia de allocator |
| Headings `## 1..6` | `:20/:42/:65/:88/:98/:114` — **sin cambio** |
| Encoding | UTF-8 sin BOM, LF; `§` y `²` intactos (`node` codepoint check) |
| Rust | **no compilado** (carril ajeno construyendo) |

## Review P2-01 (una ronda, `vanta-review`, contexto fresco)

Veredicto ronda 1: **changes-required**, 0 Blocking / 2 Major / 3 Minor. Los dos Major eran
míos y del mismo tipo que el ticket:

1. La regla nueva decía «workflows por archivo, **nunca por línea**» — y la propia página la
   falsifica en `:55` (`release-binaries.yml:120-127`) y `:107` (`ci-rust.yml:972,975`), con
   19 citas `file:line` en total. Peor: diagnosticaba mal la causa. Lo que falló no fue un
   número de línea, fue un **basename renombrado**. Reescrita a «filename real vs basename
   renombrado» — nombra la causa, no un síntoma vecino.
2. El Step 5 se marcaba ✅ incluyendo un commit que no existía. El commit lo hace el
   orquestador; ahora lo dice.

Minors aplicados: se borró la lista negativa de cuatro flags (copia literal de
`FEATURES.md:32`, redundante con `:24` y `:57`) y los dos conteos volátiles salieron de la
prosa. El Nit 6 (`:107`) quedó como deuda (3): pre-existente y fuera del párrafo de esta tarea.

Veredicto ronda 2: **approve**, 0 Blocking. Verificado por el revisor leyendo del disco
(un solo hunk `-31,7 +31,7`, 123 líneas, las tres anclas y los seis headings sin mover).

## Deuda declarada

1. `.opencode/AGENTS.md:466` **sigue afirmando nueve defaults** (`cli + arrow + fjall + roaring +
   advanced-tokenizer + memmap2 + fs2 + sysinfo + rayon`). Es la tercera copia de la misma lista y
   la que todo agente lee primero. Fuera del scope de esta tarea (`.opencode/**` prohibido):
   necesita fila propia.
2. `rg 'ci-rust-10' docs/` global → más de 100 ocurrencias en 37 ficheros de otros dueños
   (`Backlog.md`, `docs/index.md` generado, `TEST_MAP.md`, `master-index.md`, `avance/**`).
   El contrato de esta tarea se cumple en el fichero que posee; el resto es `FIND-399`.
