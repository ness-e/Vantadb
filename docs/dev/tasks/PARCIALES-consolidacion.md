---
title: "Consolidación de las 18 filas parciales: veredicto, cita corregida y puntero de verificación"
kind: task
status: completed
description: "Las 18 filas 🟡 Parcial del Backlog quedaron actualizadas contra HEAD: 4 siguen vivas, 11 son parciales reales (con lo hecho y lo que queda enumerado y con línea), 2 cambian de contrato —una a gate de versión, otra a pendiente— y 1 queda como requiere-owner; cada fila lleva la cita corregida y el puntero a su verificación."
id: PARCIALES-consolidacion
tags: [vantadb, backlog, verificacion, consolidacion]
---

# Consolidación de las 18 filas parciales

**Alcance:** consolidación documental. Sin código, sin compilar, sin tocar `src/**`.

**Fuentes (no se editan):** [PARCIALES-verificacion-A.md](./PARCIALES-verificacion-A.md)
(9 filas, medida en `12e6d7c`) y [PARCIALES-verificacion-B.md](./PARCIALES-verificacion-B.md)
(9 filas, medida en `7aaa6c239`).

**HEAD de la consolidación:** `653af5daf` (2026-11-27). Entre `7aaa6c239` y `HEAD`
solo cambiaron documentos (`Backlog.md`, `avance/activo/context-engine.md`,
`tasks/FIND-777.md`, los dos `PARCIALES-verificacion-*`, `docs/index.md`,
`llms.txt`): **ningún `src/**` ni crate**, así que las citas de fuente medidas por
los dos barridos siguen vigentes y se aplicaron tal cual.

## Qué se hizo en cada fila

1. El `Estado` refleja el veredicto medido, no la marca anterior.
2. La cita `archivo:línea` se corrigió contra `HEAD` (6 de las 9 del barrido A
   estaban desplazadas; `FIND-122` citaba un archivo que nunca contuvo lo que
   decía).
3. Cada fila lleva `Verificación: docs/dev/tasks/PARCIALES-verificacion-<A|B>.md`
   — sin eso el veredicto no es re-ejecutable y en dos semanas la fila vuelve a
   mentir (clase `FIND-811`).
4. Se retiró el marcador `— marcado por sweep sin evidencia, requiere verificación`
   de las 18 filas: ya se verificaron.

## Las 18, con su estado nuevo

| ID | Veredicto (fuente) | Estado en el Backlog | Corrección de cita aplicada |
|---|---|---|---|
| `FIND-120` | PARCIAL-REAL (A) | `🟡 Parcial` — hecho: registro multi-formato (`42fb7c86a`). Queda: scan gitignore-aware, chunking por símbolo, escena repo-map, hook incremental, oferta en el wizard | `sources.rs:82` → `:39` (el filtro `ext == "md"`); `code.rs` → `:39` (8 tools), `:326-327` (stub), `:13-16` (BFS-1); hooks 50/50 → 39 asserts; `setup-embeddings.ps1` → `scripts/setup-embeddings.ps1` |
| `FIND-121` | PARCIAL-REAL (A) | `🟡 Parcial` — hecho: slice 1 `embed-cache` (`FIND-706`). Queda: cache de recall + proxy-context | `cache.rs:21-34` precisa (`:21` doc + `:22-33` struct); `volatile_cache_entries` → `metrics/core/mod.rs:679`; se documenta que `EmbedCache` existe y **rechaza el TTL** |
| `FIND-122` | CAMBIADA (A) | `🟡 Pendiente (contrato corregido)` — el storage híbrido y `search_profile` **ya existen**; falta el router automático | `index/search/layer.rs` (nunca tuvo fusión) → `planner.rs`, `physical_plan/scan.rs`, `search_profile.rs:20-30`; `SearchProfileMode:35-43`, `Config:50`, pesos `:73,80,87`; `api/scores.rs:17` se mantiene |
| `FIND-158` | PARCIAL-REAL (A) | `🟡 Parcial` — hecho: (d) reporta `truncated_namespaces`. Queda: paginar `versions`, decidir naming, corregir `next_cursor` de salida | `handlers.rs:286-289`; `router.rs:277-278` + `openapi.yaml:697,732`; (c) vive en `vantadb-node/index.d.ts:148` vs `:120` (**no** en `desktop/`); `list_records.rs:107,116,126`; `handlers.rs:449-453` |
| `FIND-159` | PARCIAL-REAL (A) | `🟡 Parcial` — hecho: nada nuevo; (a) sigue diferido. Queda: `data.code` en `internal_error` **y** `invalid_request`, tipar 41 tools | "38" era el perfil `dev` (`MCP.md:276`); el universo real es **41** extendidas sobre 96 definidas / 90 listadas (`MCP.md:971`); `error.rs:82-84`, `:21-27`, `:124-125`, `:87-89` |
| `FIND-290` | PARCIAL-REAL (A) | `🟡 Parcial` — (a) **cerrado** (`trust_aware` es la perilla). Queda: (b) promotion de lecciones, (c) wiring del pase | `lib.rs:45,1164` exactas; se añade el matiz `if owned.trust_aware` (`:1163`), `reflection/mod.rs:245`, `:6`, `:351` |
| `EGO-13` | PARCIAL-REAL (A) | `🟡 Parcial` — hecho: Python zero-copy. Queda: **solo** la mitad napi + bench (Regla 9) | `vantadb-py/` no existe → `vantadb-python/`; `EGO-03.md:113` purgado por `cb22b4636`; `lib.rs:988,1008`, `types.rs:19-38`, `convert.rs:198,213`, `Cargo.toml:15`; `get_f32_vec:1929-1960` |
| `EGO-15` | PARCIAL-REAL (A) | `🟡 Parcial` — el cap ya es el default. Queda: exponerlo en **cada binding**, decidir el seed, `max_total_nodes` | `seed.rs:25` → `:31` (`8f3b7039d`, dos días **antes** de la re-verificación que citó `:25`); el setter **sí** existe en el SDK (`pipeline.rs:96,101`; `public-api.txt:3151,3153`); `graphrag_search` `lib.rs:1013-1019` |
| `DEAD-01` | VIVA (A) | `🟡 Pendiente` — no hay barrido repo-wide; 81 `#[allow(dead_code)]` en 33 archivos; `dead_code` fuera de `deny` (`Cargo.toml:761-762`) | el "barrido acotado (DX-13 Slice 4)" **no tiene traza**: `DX-13` es "Regeneración de completions de CLI" (`avance/activo/bindings.md:1249`) y la fila se cita a sí misma |
| `FIND-198` | VIVA (B) | `🟡 Pendiente` — la disposición está escrita; falta el runner opt-in (nunca en CI) | `evals/data/README.md:33-41`; cero BEAM en `evals/**/*.py`; dueño `vanta-tuner` |
| `FIND-215` | PARCIAL-REAL (B) | `🟡 Parcial` — **el "falta doc de cuándo usar cuál" quedó refutado** (`docs/feature-matrix.md:55,75-76`, `5deca8c78`). Queda **solo**: medición canonical_p99 + decisión de `default` | `Cargo.toml:164-165,194-202` → `:168-169` (features) y `:200-210` (deps); `:156` = `default` sin allocator |
| `FIND-320` | VIVA (B) | `🟡 Pendiente` — piezas de recall existen, `retrieve()` no. **Corrección de inventario:** `MGR-06`/`MGR-18` están `Pendiente`, no son piezas existentes | `auto_recall.rs:391,402,424`, trust gate `:184`, opt-in `lib.rs:1161-1176`; abstención real en `vantadb-mcp/src/validation.rs:433`; `Backlog.md:331,359` |
| `FIND-598` | VIVA (B) | `🟡 Pendiente` — Paso 0 hecho (mitigación), gate del p99 **sigue superado**; falta el envelope por página (breaking de `VectorStoreRef`) | `vfile.rs:115,1255,1340,1451-1461,198-205`; test del reparto `:2223-2250` (`encrypted_read_path_copies_are_pinned`, `assert copies == 5` en `:2248-2249` — B citó `:2085-2119`, que es el helper `legacy_read`); `index_port.rs:86-88`; `neighbors.rs:57`; el diseño sigue sin implementar (`docs/index.md:446`). **El veredicto y el puntero van en la celda `Relaciones`**: la Descripción mide 7,6 KB (números certificados) y no se reescribió |
| `WIRE-19` | PARCIAL-REAL (B) | `🟡 Parcial` — hecho: 3 de 9 filas + `WIRE-20` cerrada. Quedan `WIRE-21/22/23` y la enmienda del ADR D4 | shims `scheduler.rs:82,85,94` → `:63,66,75`; degradación P4 `:97-100` → `vantadb-server/src/scheduler.rs:75-81` (`:79`, shim `ingest_runner_factory`) + `vantadb-mcp/memory_scheduler.rs:98-103` (`:101`) — **B la citó en `vanta-memory/services/scheduler.rs:78-81`, que es el cierre de `parse_interval_ms`**; `_opt` ya existe (`vanta-memory/services/scheduler.rs:92`) |
| `DIST-18` | CAMBIADA (B) | `⏸️ Bloqueada: versión 1.0.0` — **no es trabajo a medias**, es un gate de versión; se retira del recuento de parciales | `vanta-proxy/Cargo.toml:7` `publish = false`; fuera de `default-members` (`Cargo.toml:797,805,812`); workspace 0.9.0 (`:820`); 404 en crates.io (2026-11-27) |
| `GTM-05` | PARCIAL-REAL (B) | `🟡 Parcial - requiere al owner: DNS` — spec y contrato de UI en repo; el dominio **no resuelve** | `VANTADB_DEV_LANDING_AND_ENTERPRISE_SPEC.md:137-160`; `BND-07` en `Backlog-negocio.md:66` |
| `GTM-09` | PARCIAL-REAL (B) | `🟡 Parcial - requiere al owner: deploy + precio` — el one-pager ya lleva el precio; falta el deploy con DNS | `OEM_ONE_PAGER.md:86` ("Starting at $750 / month"); `/enterprise` en el spec `:78-80`; `Backlog-negocio.md:96` |
| `NOT-06` | NO-VERIFICABLE-DESDE-EL-REPO (B) | `🟡 Pendiente - requiere al owner: walk de las 71` — en vivo hay 8 secciones y 38 hijas; "71/71" no es afirmable | el fichero de evidencia citado no existe en el repo; el plan archivado afirma 100 % (`plans/archive/2026-11-03-master-execution-plan.md:617-634`) |

**Contable (18 = 4 VIVA + 11 PARCIAL-REAL + 2 CAMBIADA + 1 NO-VERIFICABLE):**
**11 `🟡 Parcial`** (los 11 PARCIAL-REAL), **6 `🟡 Pendiente`** (los 4 VIVA +
`FIND-122`, cuya CAMBIADA deja sólo el router automático por hacer, + `NOT-06`,
que es NO-VERIFICABLE y requiere al owner), **1 `⏸️ Bloqueada`** (`DIST-18`).
Ninguna fila se retiró: **0 REFUTADAS** en estas 18.

## Las 10 decisiones del owner (propuestas por el barrido B)

Ninguna se tomó en esta consolidación: son decisiones de producto, contrato o
versión. Cada una dice qué desbloquea.

| # | Tema | Decisión propuesta | Desbloquea |
|---|---|---|---|
| 1 | **DNS de `vantadb.dev`** | Registrar/delegar el dominio y apuntarlo a Vercel | `GTM-05`, `GTM-09`, `BND-07` — un solo bloqueo para tres filas |
| 2 | **Cuenta Vercel y pasarelas** | Mirar el dashboard (deploy activo, dominio); mantener pasarelas fuera hasta que exista checkout real (el spec define waitlist sin pago) | `GTM-05` |
| 3 | **Precio público `$750/mes`** | Publicarlo tal cual en `/enterprise` (`start at $750`, escalando a ~$1,000): ya está escrito y coincide con `Backlog-negocio.md:96`; ocultarlo obliga a dos discursos | `GTM-09` |
| 4 | **`NOT-06`: criterio de aceptación** | Exigir el walk de las 71 páginas con el DoD del propio plan (0 vacías, 0 refs `v0.6.1`) y guardar el informe en `docs/dev/reviews/`; hoy apunta a un fichero inexistente | `NOT-06` |
| 5 | **`DIST-18`** | No hay decisión pendiente hoy: reetiquetada como bloqueada por 1.0.0 y retirada del recuento de parciales. El día del 1.0.0 sí hará falta autorización explícita de publish | — |
| 6 | **`FIND-215`: allocator en `[features] default`** | **No**: los binarios publicados ya eligen por OS (`release-binaries.yml:117-122`) y meterlo en `default` obliga a cada consumidor de la librería a compilar mimalloc/jemalloc. Falta la medición (Regla 9), no la política | `FIND-215` |
| 7 | **`FIND-198`: runner BEAM opt-in** | Sí, con la disposición ya escrita (`evals/data/README.md:33-41`): nunca en CI, dataset fuera del repo. Falta decidir quién lo posee (`vanta-tuner`, según la propia nota) | `FIND-198` |
| 8 | **`FIND-320`: ADR del contrato `retrieve()`** | ADR corto que fije el wire shape (trust + confidence + freshness + provenance + conflict + abstain) y el opt-in byte-idéntico por default (precedente MEMG-21). Sin eso el implementador inventa el contrato | `FIND-320` |
| 9 | **`FIND-598`: breaking de `VectorStoreRef`** | Asumirlo post-1.0.0: el envelope por página devuelve el zero-copy pero cambia un trait público, y en caliente el p99 (+30,7 %) tampoco cierra el gate | `FIND-598` |
| 10 | **`WIRE-19`: ADR D4 (loop OFF sin variable vs ON a 5 000 ms)** | Enmendar el ADR a lo enviado (ON 5 000 en el host MCP) en vez de revertir: desactivar una capacidad ya enviada no es un refactor, y `WIRE-21/22/23` cuelgan de esa decisión | `WIRE-19`, `WIRE-21`, `WIRE-22`, `WIRE-23` |

## Contradicciones encontradas

Regla aplicada: **gana `HEAD`**, y se dice.

1. **`WIRE-19` (fila vs `HEAD`)** — la fila, escrita el mismo 2026-11-27, citaba
   los shims en `vantadb-server/src/scheduler.rs:82,85,94` y la degradación P4 en
   `:97-100`. `HEAD` da los shims en `:63,66,75` y la degradación P4 en
   `vantadb-server/src/scheduler.rs:75-81` (`:79`, dentro del shim
   `ingest_runner_factory`) + `vantadb-mcp/memory_scheduler.rs:98-103` (`:101`).
   Causa: `WIRE-20` se cerró ese mismo día (`ddf64df2c`, `858c8361c`) y la madre
   envejeció en horas. Ganó `HEAD`. **Nota:** el barrido B citó esa segunda copia
   en `vanta-memory/src/services/scheduler.rs:78-81`, que en `HEAD` es el cierre
   del brazo `Err` de `parse_interval_ms` (no la degradación); la fila quedó
   corregida contra `HEAD` y lo declara. B no se edita (es fuente).
2. **`FIND-215` (fila vs `HEAD`)** — citaba `Cargo.toml:164-165,194-202`; lo
   medido es `:168-169` (features) y `:200-210` (deps). Además la fila pedía
   "doc de cuándo usar cuál": existe desde `5deca8c78` (`docs/feature-matrix.md`).
   Ganó `HEAD`; el "falta" queda reducido a medición + default.
3. **`EGO-15` (marca de re-verificación vs `HEAD`)** — la fila dice
   "re-verificado 2026-11-22" y cita `seed.rs:25`; `8f3b7039d` (2026-11-20) ya
   había empujado ese `embedded.search` a `:31`, **dos días antes**. La marca de
   re-verificación era falsa. Ganó `HEAD`.
4. **`EGO-15`, segunda afirmación (fila vs API pública)** — "ninguna superficie
   pública expone `max_degree_per_node`/`min_edge_weight`" contradice
   `pipeline.rs:96,101` y `tests/api/public-api.txt:3151,3153`. Ganó `HEAD`; el
   error iba en la dirección conservadora (exageraba lo que falta) y podía llevar
   a crear un setter que ya existe.
5. **`FIND-320` (fila vs Backlog)** — la fila llama "piezas que existen" a
   `MGR-06` (conflicto) y `MGR-18` (abstención); ambas siguen `Pendiente`
   (`Backlog.md:331,359`). Ganó el Backlog.
6. **`FIND-121` (prescripción vs código)** — la fila propone "LRU+TTL" y pide
   verificar que no exista cache texto→vector. `EmbedCache` (`src/llm.rs:230-262`)
   la entregó `FIND-706` **68 días después** y **rechaza el TTL por diseño**
   (`:234-237`): `embed` es determinista en `(model, prefix, text)`. La
   prescripción contradice el código; se anota en la fila y se reescribe el alcance.
7. **`FIND-598` (contradicción interna de la fila)** — la misma celda dice
   "**Lo que queda pendiente**: (a) la re-medicion post-Paso 0" y, más abajo,
   "**Re-medicion post-Paso 0 (2026-10-21/22):** el gate sigue **SUPERADO**". El
   residual (a) está hecho; lo que falta es el envelope por página. No se
   reescribió la celda entera (7,6 KB, fuerte en números certificados): se
   corrigieron estado y veredicto, y la contradicción queda declarada acá.
8. **`NOT-06` (plan archivado vs repo)** — el plan del 2026-11-03 declara
   "100 % COMPLETADO EN VIVO" (`:617-634`) y la fila cita un fichero de evidencia
   que **no existe** en el repo. En vivo hay 8 secciones y 38 hijas. Ganó el repo:
   NO-VERIFICABLE, y "verificado" no significa nada hasta el walk.
9. **`DIST-18` (título vs estado real)** — la fila se llama "Publicar
   `vanta-proxy`" con estado parcial; el crate tiene `publish = false` y no existe
   en crates.io. No hay trabajo a medias que retomar: hay un gate de versión.
10. **`FIND-120` (cita vs `HEAD`)** — `sources.rs:82` "hoy solo .md": en `HEAD` el
    filtro está en `:39` y `:82` ya no lo contiene; el registro multi-formato
    (`42fb7c86a`) además desmiente el ".md-only" actual. Ganó `HEAD`.
11. **`DEAD-01` (fila como evidencia de sí misma)** — "barrido acotado a
    cli/cli_handlers/bin (DX-13 Slice 4)" no aparece en ningún documento salvo la
    fila; `DX-13` en el registro de avance es "Regeneración de completions de
    CLI" (`avance/activo/bindings.md:1249`). No hay traza del barrido declarado
    hecho, pero el defecto (falta de barrido repo-wide) es real: la fila pasa a
    `🟡 Pendiente`.

## Conteo de filas (guarda anti-vaciado)

| Métrica | Antes | Después |
|---|---:|---:|
| Líneas del archivo | 963 | 963 |
| Filas de tabla con ID (`` ^\| `ID` ``) | 281 | 281 |
| `git diff --numstat` | — | **18 inserciones / 18 borrados** |
| Hunks con línea de fila | — | exactamente las 18 líneas objetivo (89, 112, 115-116, 140, 176-180, 199, 205, 238, 555, 602, 612-613, 694) |

> El enunciado de la tarea esperaba **275** filas. Medido en `HEAD`: **281**
> filas con ID más la fila `FASE-A` (sin sufijo numérico) = **282** celdas
> iniciales con backtick. La cifra del enunciado no corresponde a ninguna métrica
> reproducible sobre este archivo; la invariante que importa —**el mismo número
> antes y después**— se cumple (281/281), y ningún otro carril aparece en el diff.

## No tocado, a propósito

- **Encabezado de totales del Backlog** (`> **Total open items:** … **40 🟡 — 36
  `🟡 Parcial` + 4 `🟡 Pendiente`** …`): sigue diciendo "294 filas" cuando hay 281.
  Corregir solo la parte `🟡` dejaría el resto falso igual; es un saneamiento
  aparte y esa línea la tocan otros carriles. `NOTICED BUT NOT TOUCHING`.
- `docs/dev/plans/**`, `docs/index.md`, `llms.txt`: prohibidos en esta tarea (no
  se corrió `gen-index`).
- Los dos `PARCIALES-verificacion-*.md`: son la fuente, no se editan.
- `FIND-777`: cerrada y fuera del Backlog.
- `src/**`, `tests/**`, `vanta-memory/**`, `vantadb-mcp/**`, `.github/**`,
  `dev-tools/**`, `.opencode/**`: otros carriles escriben ahí (`FIND-798` compilando).

## Verificación

- `git diff --numstat -- docs/dev/Backlog.md` → `18 18` (solo las 18 filas).
- `git diff -U0` → hunks únicamente en las 18 líneas objetivo.
- Conteo de filas con ID: 281 antes y después; 963 líneas antes y después.
- `Verificación:` presente 9× (A) + 9× (B) = 18 filas.
- Cero caracteres de reemplazo (`U+FFFD`) y `ñ`/`ó` contados antes/después sin
  pérdida (sin mojibake en la escritura UTF-8).
- `node scripts/docs/check-links.mjs` → 0 enlaces rotos nuevos (el único roto del
  repo es previo, en `avance/historial/snapshot-2026-08-03.md`).
- `node scripts/docs/check-docs.mjs` → `GATING: all clear` (951 documentos).
- `npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml <3 archivos>` →
  `0 issues in 0 files`: **este gate no comprobó nada** (las tres rutas están en
  `ignores` de la config del repo). Se declara por transparencia, no como evidencia.
- `pwsh dev-tools/ocr-review.ps1 -Format json` → 2 grupos de reglas (JSON y Rust);
  **ninguno aplica** a este cambio (solo markdown). Los `.rs` y `.json` que
  aparecen en el diff son de otros carriles.
- Sin compilación de Rust (disco y carriles ajenos).

## Review

**Review P2-01 (contexto fresco, `vanta-review`, sesión `ses_de238a19cffehoFYUiboWix9yh`).**

- **Revisor:** `vanta-review` (subagente, contexto distinto al autor).
- **Enfoque:** (1) ningún veredicto inventado — las 18 etiquetas contra las dos
  fuentes; (2) ~25 citas `archivo:línea` abiertas al azar contra `HEAD`;
  (3) alcance del diff (18/18, cero filas ajenas); (4) punteros y estados;
  (5) 4 contradicciones al azar; (6) `FIND-777` y las dos fuentes intactas;
  (7) premisa H-07 re-ejecutada (`git diff --name-only 7aaa6c239..HEAD` = 7
  paths, todos documentos, 0 de código).
- **Veredicto ronda 1:** `changes-required` — contenido correcto, **3 defectos de
  cita/número** (la capa que esta tarea existe para arreglar) + falta esta sección.
  - **R1 (cita falsa):** la degradación P4 no está en
    `vanta-memory/src/services/scheduler.rs:78-81` (eso es el cierre del brazo
    `Err` de `parse_interval_ms`); las dos copias reales son
    `vantadb-server/src/scheduler.rs:75-81` (`:79`) y
    `vantadb-mcp/src/memory_scheduler.rs:98-103` (`:101`). Era un defecto
    **heredado de B**; se corrigió en la fila y en la tabla de arriba, y se
    declaró que B no se edita.
  - **R2:** el titular del avance decía "5 seguían vivas" contra el "4 VIVA" del
    propio bloque → corregido a **4**.
  - **R3:** el `description` del frontmatter decía "5 pasan a Pendiente porque
    seguían vivas"; en realidad quedan **6** en `🟡 Pendiente` (4 VIVA +
    `FIND-122` CAMBIADA + `NOT-06` NO-VERIFICABLE) → frontmatter reescrito.
  - **R4:** esta sección no existía y el task file ya se declaraba `completed` →
    agregada.
  - **Opcional O1 aplicado:** el test del reparto de `FIND-598` es
    `vfile.rs:2223-2250` (`encrypted_read_path_copies_are_pinned`), no
    `:2085-2119` (helper `legacy_read` pre-Paso 0). Mismo patrón "cita el vecino".
  - **Opcional O2 (no aplicado, ajeno):** `docs/index.md` está modificado en el
    árbol por el carril de `FIND-798` (su fila entra en el mismo diff) y ya
    publica esta tarea como `completed`. No se tocó: `gen-index` está prohibido
    aquí; queda declarado.
- **Verificado en verde por el revisor:** 18/18 etiquetas coinciden con A/B;
  estados vs veredicto sin contradicción; punteros 9+9 sin cruces; hunks e IDs
  exactos; `U+FFFD = 0` en los tres archivos; las 4 contradicciones muestreadas
  son reales (`WIRE-19 :82,85,94→:63,66,75`, `EGO-15 seed.rs:25→:31`,
  `FIND-320` MGR-06/MGR-18 en `Backlog.md:331,359`, `FIND-215`
  `Cargo.toml:168-169/200-210/156`).
- **Estado:** `changes-required` ronda 1 **aplicado**; **ronda 2: `approve`**
  (misma sesión de review, sobre este texto), con 3 nits no bloqueantes. Dos se
  aplicaron después del approve: el assert vive en `:2248-2249` (el `assert_eq!`
  abre en 2248 y el literal en 2249) y el bullet de markdownlint ahora declara
  que **no comprobó nada** (rutas en `ignores`); el tercero (agrupación del
  bullet de review en el avance) es cosmético y se deja.

