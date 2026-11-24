---
title: "FIND-707: el stamper de frontmatter no puede volver a dañar el corpus"
kind: task
status: in-progress
description: "El parser compartido no tolera BOM ni el escritor el CRLF: una pasada inyecta un segundo bloque, deriva el title del nombre de fichero y añade una línea en blanco antes del H1"
---

# FIND-707: el stamper de frontmatter no puede volver a dañar el corpus

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — Task 5
- **Fuente:** fila `FIND-707` de `docs/dev/Backlog.md` (origen: gate GATES Docs 2026-11-23, run `45253777483`) + commit `c54165cff`
- **Esfuerzo:** 🟢 2h
- **Prioridad:** 🟠 Alta (el corpus se reparó a mano; la herramienta sigue rota)
- **Tipo:** Docs / tooling de gates (Node ESM, sin dependencias)
- **Turns estimados:** 8
- **Creado:** 2026-11-24T02:10
- **last-synced:** 2026-11-24T03:05
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `parseFrontmatter` (`scripts/docs/lib.mjs`): `check-docs.mjs:97`, `check-links.mjs:166`, `gen-index.mjs:80`, `check-doc-examples.mjs:118`, `stamp-frontmatter.mjs` → los 4 jobs de `docs-gates` en CI |
| Callees | `parseYaml` (`split(/\r?\n/)`, ya tolerante a CRLF), `serialiseFrontmatter`, `firstH1`, `firstProseLine`, `deriveKind`, `isArchive`, `listDocs`/`readDoc` |
| Implicaciones | El contrato de retorno de `parseFrontmatter` **no cambia de forma** (`{raw, data, body, hadFrontmatter, end, nl}`); cambia el valor de `body` (sin BOM) y `end` sigue indexando el texto ORIGINAL. `data` idéntico ⇒ `check-docs` (kind por ruta) y `gen-index` no cambian de veredicto. Sin impacto en Rust, sin migración de datos, sin re-indexación |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/docs/lib.mjs`, `scripts/docs/stamp-frontmatter.mjs`, `scripts/docs/check-npm-versions.mjs` (patrón `--self-test`), `scripts/docs/wikilinks-to-md.mjs` (patrón `--self-test`), `.opencode/task-system/prompts/task.md`, `.opencode/skills/documentation-skill/SKILL.md`
- **Archivos referenciados hacia dentro (imports):** `lib.mjs` no importa nada del repo; `stamp-frontmatter.mjs` importa `listDocs`, `parseFrontmatter`, `serialiseFrontmatter`, `deriveKind`, `firstH1`, `firstProseLine`, `readDoc`, `writeDoc`, `isArchive`
- **Archivos que referencian a los editados (referencias entrantes):** `grep -n "parseFrontmatter" scripts/docs/*.mjs` → 5 consumidores (arriba). `grep -rn "stamp-frontmatter" docs/ .github/ justfile` → docs de gates (`gate-docs-links.md`, `documentation-skill`) y CI (ningún workflow lo ejecuta; es una herramienta de backfill manual)
- **Veredicto impacto:** medio-acotado — el cambio vive en un helper compartido por 4 gates, así que se verifican los 4 (todos exit 0) y no toca ningún path del core Rust

## Contrato

```
node scripts/docs/stamp-frontmatter.mjs --self-test
node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/check-secrets.mjs && node scripts/docs/gen-index.mjs --check
node scripts/docs/stamp-frontmatter.mjs --write      # no-op sobre el corpus limpio
git status --porcelain -- docs                        # idéntico antes/después (este predicate SÍ ve untracked)
git diff --quiet -- docs ':(exclude)docs/dev/plans/2026-11-23-release-v0.10.0-gates.md' ':(exclude)docs/pipeline-state.json'   # post-commit
```

Todo exit 0 / sin salida. Dos matices medidos, no teóricos:

- `git diff --quiet` es **ciego a ficheros untracked**: en el run de review el stamper escribió `docs/dev/tasks/FIND-485.md` (del agente concurrente) y el diff acotado no lo habría visto. Por eso la prueba de idempotencia es el par `--write` + `git status --porcelain` idéntico (más hash de los 777 documentos), no el `git diff` solo.
- El `git diff` acotado se corre **después** del commit: antes, `docs/index.md` está sucio por la regeneración legítima del índice (el task file nuevo entra en el índice), que no la escribe el stamper. Las dos exclusiones restantes son ficheros sucios por trabajo concurrente (`FIND-485` / task-system), no documentos del corpus.

## Spec (decisiones técnicas)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde normalizar el BOM | A) `readDoc` (acotado, deja `parseFrontmatter` frágil para los otros 4 consumidores que leen por otras vías) / B) dentro de `parseFrontmatter` (punto único, cura a los 5 consumidores) | ✅ B — evidencia: el daño se originó en `hadFrontmatter:false` falso, y los 5 consumidores comparten ese valor; `readDoc` no lo arregla para `check-doc-examples.extractBlocks(src)` |
| 2 | ¿Re-emitir el BOM al escribir? | A) conservarlo (byte-idéntico) / B) descartarlo (normaliza el fichero) | ✅ B — corpus con 0 BOM tras la reparación manual (`node -e` sobre los 775 docs: `bom: 0`), así que descartarlo no produce ningún diff hoy y el BOM deja de viajar al `body` |
| 3 | Newline del bloque al recomponer | A) siempre LF (fichero mixto en los 199 docs CRLF) / B) el newline del propio fichero | ✅ B — una línea por `---` con el `\r\n` del fichero; idempotente y sin mezclar EOL |
| 4 | Cómo se prueba | A) aserciones sobre el regex / B) `stampDocument()` puro ejercitado por `--self-test` | ✅ B — el pre-mortem del plan lo exige: un test del regex queda verde mientras el escritor corrompe ficheros |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) ninguna pasada puede dejar más de un bloque de frontmatter por documento; (2) exactamente una línea en blanco entre el `---` de cierre y el cuerpo; (3) `title` sale del `title` de frontmatter, si no del H1, y solo si no hay H1 del nombre de fichero; (4) `body` no se reescribe más allá de las líneas en blanco iniciales; (5) `kind` sigue derivándose de la ruta.
- **Comandos de verificación:** `node scripts/docs/stamp-frontmatter.mjs --self-test` (5 casos, exit 0) + los 4 gates de docs + idempotencia acotada.
- **Deuda pendiente:** `segment()` (`lib.mjs`) sigue siendo BOM-ciego (`src.startsWith("---\n")`); hoy no daña nada porque el corpus tiene 0 BOM y `check-docs` lo detectaría, pero un BOM futuro haría que `wikilinks-to-md` reescribiese frontmatter. Fuera de alcance de este fix (ver Notas).

## Fase 1 — Evidencia de Debugging

- **Repro:** con el parser de `HEAD`, los 3 casos dañan (script `red-repro.mjs`, mismo predicado que el self-test):

  ```
  DAMAGED  BOM + frontmatter — 3 "---" lines
  DAMAGED  BOM + H1, no frontmatter — title "FIND 591"
  DAMAGED  CRLF — 2 blank line(s) before the H1
  HEAD damage reproduced in 3/3 cases.
  ```

- **Hipótesis:** causa raíz doble. (a) `parseFrontmatter` prueba `/^---\r?\n/` sobre el texto crudo: con `\uFEFF---` no matchea → `hadFrontmatter:false` → el stamper considera el documento "sin frontmatter" e inyecta otro bloque; el mismo BOM hace que `^#` no matchee el H1 (`\uFEFF# Title`) → `title` cae al nombre de fichero. (b) La recomposición `serialiseFrontmatter(next) + "\n" + fm.body.replace(/^\n+/,"")` no consume `\r\n`, así que el `\r\n` inicial del body sobrevive y cada pasada añade una línea en blanco más.
- **1 variable controlada:** por intento — primero el parser (BOM), después el escritor (newline), cada uno con su caso de self-test.
- **Test RED:** `--self-test` falla 3/5 contra `HEAD` (evidencia arriba) y pasa 6/6 con el fix; el caso 4 (sin H1 → nombre de fichero) y el 5 (LF simple) fijan el comportamiento que NO debía cambiar, y el 6 cubre el cierre `---` en EOF (mismo género: el parser decía "no hay frontmatter" y el stamper apilaba un bloque encima).
- **Mutantes (comprobados):** quitar el strip de BOM → exit 1 (`3 "---" lines`, title `FIND 591`) · volver a `/^\n+/` → exit 1 (`2 blank line(s)`) · forzar el bloque a LF en fichero CRLF → exit 1 (newline del bloque) · escribir el bloque en LF desde `compose` → exit 1. Ningún mutante de los 4 sobrevive.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** negativo — no se añade deuda (el `stampDocument` extraído es el mismo código, ahora puro y testeable) y se cierra la clase de bug que obligó a reparar 33 ficheros a mano.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | self-test 6/6 + 4 gates de docs exit 0 + idempotencia (`--write` no-op: `git status --porcelain -- docs` idéntico y `git diff --quiet` acotado exit 0 post-commit) |
| **Commit** | atómico (2 scripts + task file + estado del plan), conventional `fix(docs): …`, verificación mecánica pegada abajo |
| **Release** | N/A — herramienta local, sin impacto en artefactos publicados |

## Herramientas necesarias
- `node` (gates de docs; sin cargo — `target/` lo usa `FIND-485`)
- `git diff --quiet` acotado para la prueba de idempotencia

**Skills cargadas (SDP):** `documentation-skill` (Regla 3, corpus/frontmatter) · `test-driven-development` (bug con repro RED primero) · `systematic-debugging` (Iron Law: causa raíz antes del fix) · `incremental-implementation` (2 slices: parser, escritor) · `source-driven-development` (patrón `--self-test` del repo, sin APIs externas) · `context-engineering` (jerarquía de contexto) · `writing-guidelines` / `writing-plans` / `campaign-executor` / `progreso` (base SDP tipo Documentation)

## Investigation Notes
- Web research innecesario: el comportamiento es del propio repo (`docs/_schema/frontmatter.schema.json`, `scripts/docs/*.mjs`).
- `.gitattributes` fija `* text=auto eol=lf` (`git ls-files --eol` → `i/lf w/crlf`): 199 documentos están CRLF en el working tree porque el checkout de Windows los materializa así. Escribir el bloque con el newline del propio fichero no produce diff (git normaliza a LF en el índice), y escribirlo con LF mezclaría EOL en el working tree.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Context Save Point

- **Estado del árbol:** `scripts/docs/lib.mjs`, `scripts/docs/stamp-frontmatter.mjs`, `docs/dev/tasks/FIND-707.md`, `docs/index.md` y `llms.txt` commiteados juntos. Sucios y **ajenos**: `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (lo lleva el orquestador), `docs/pipeline-state.json` (task-system), `docs/dev/tasks/FIND-485.md` (worker paralelo) y `src/**` (`FIND-485`).
- **Si vuelve a aparecer un BOM:** el fix del parser ya lo absorbe; el único punto BOM-ciego que queda es `segment()` (ver Notas) y `check-docs` avisaría antes de que dañe nada.
- **Si el índice aparece sucio:** es `gen-index.mjs --write` recogiendo documentos nuevos de otros agentes, no el stamper (`--write` del stamper deja `unchanged: 777` y no toca ficheros).

## Steps

### Step 1: Reproducir el daño de `HEAD` (RED)
- **Archivos:** `scripts/docs/lib.mjs`, `scripts/docs/stamp-frontmatter.mjs`
- **Acción:** correr los 3 casos de daño contra el parser de `HEAD` y contra la recomposición de `HEAD`
- **Verify:** `node $TEMP/opencode/red-repro.mjs` → `HEAD damage reproduced in 3/3 cases`, exit 1
- **Estado:** ✅

### Step 2: `parseFrontmatter` tolerante a BOM (parser, cura a 5 consumidores)
- **Archivos:** `scripts/docs/lib.mjs:138`
- **Acción:** descartar el BOM antes del regex; `end` se devuelve sumando el BOM para seguir indexando el texto original; docstring con el daño del 2026-11-23
- **Verify:** `node scripts/docs/stamp-frontmatter.mjs --self-test`
- **Estado:** ✅

### Step 3: `stampDocument()` puro + recomposición newline-aware
- **Archivos:** `scripts/docs/stamp-frontmatter.mjs`
- **Acción:** extraer la transformación por documento a una función pura (el loop solo agrega stats y escribe); `compose()` usa el newline del fichero y consume `/^(?:\r?\n)+/`
- **Verify:** `node scripts/docs/stamp-frontmatter.mjs` (dry run) → `unchanged: 775`
- **Estado:** ✅

### Step 4: `--self-test` con los 3 casos de daño + 3 de no-regresión
- **Archivos:** `scripts/docs/stamp-frontmatter.mjs`
- **Acción:** 6 casos que afirman el contrato real (1 bloque, 1 línea en blanco, EOL del bloque, title del H1, título del fichero solo sin H1, cierre en EOF) + segunda pasada no-op
- **Verify:** `node scripts/docs/stamp-frontmatter.mjs --self-test` → 6/6 ok, exit 0
- **Estado:** ✅

### Step 5: gates del área + idempotencia
- **Archivos:** `docs/index.md`, `llms.txt` (regenerados por `gen-index.mjs --write`, nunca a mano)
- **Acción:** los 4 gates de docs + `--write` sobre el corpus limpio comparando estado del working tree y hash de los documentos
- **Verify:** los 4 gates exit 0 (con `docs/dev/tasks/FIND-485.md` transitoriamente fuera del árbol, porque es de otro agente); `status_identical=True`; `git diff --quiet` acotado exit 0 post-commit
- **Estado:** ✅

### Step 6: review P2-01 por agente distinto
- **Archivos:** `docs/dev/tasks/FIND-707.md` (§Review)
- **Acción:** review adversarial del diff por `vanta-review` (contexto distinto) + OCR delegation
- **Verify:** veredicto registrado en §Review, con los 3 ítems accionables cerrados
- **Estado:** ✅

## Dependencias
- Ninguna. Wave 0. `FIND-485` corre en paralelo sobre `src/executor.rs` — no se toca ni se commitea nada suyo.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_df2144581ffeY0BdKegcaO7H6q`, contexto distinto al implementador)
- **Tier:** Fast (`scripts/docs/**`, `docs/dev/tasks/**`) → verify mecánico + spot-check del revisor (re-corrió el contrato completo, no aceptó auto-reporte)
- **Enfoque:** ¿el fix está en el punto único (parser) y no parchea síntoma por caller? Confirmado: revisó los 4 consumidores uno por uno y verificó que `end` indexa el texto original (`end + bom`) y que nadie reconstruye el body con `text.slice(fm.end)`; concluyó que `readDoc` no habría servido (`extractBlocks(src)` recibe un string crudo). También grepeó `parseFrontmatter` repo-wide: los demás matches son código vendorizado con parser propio, el blast radius se queda en `scripts/docs/`.
- **Cómo se probó:** veredicto sobre comandos ejecutados, no sobre el informe del implementador (`--self-test` exit 0; 4 gates exit 0/0/0/0; mutantes en `%TEMP%`). Dictamen inicial: **❌ cambios requeridos — solo contrato/DoD, código aprobado tal cual**; 3 ítems accionables:
  1. El contrato #3 no daba exit 0 y el task file afirmaba que sí (faltaba `docs/index.md`, sucio por la regeneración del índice). → contrato reescrito y re-medido post-commit.
  2. `git diff --quiet` es ciego a untracked (su propio `--write` estampó `FIND-485.md` sin que el predicado lo viera). → la prueba de idempotencia pasa a `git status --porcelain` idéntico + hash de los documentos.
  3. Un mutante que escribía el bloque en LF sobre fichero CRLF pasaba 5/5. → añadida la aserción del newline del bloque (y el mutante ahora falla).
  Low/Nit aceptados como documentados: BOM no re-emitido (decisión 2, corpus con 0 BOM), colapso de ≥2 líneas en blanco a 1 (0 reescrituras hoy por el guard `rebuilt === text`), y `fences()` contando HRs → reemplazado por el predicado `hadFrontmatter && el body no empieza con un delimitador`.
- **Veredicto:** ✅ approve (código aprobado en la primera pasada; los 3 ítems de contrato/cobertura cerrados y re-verificados: 6/6 casos, 4 mutantes en rojo)

## Notas
- **NOTICED BUT NOT TOUCHING:** `segment()` (`scripts/docs/lib.mjs:460`) hace `src.startsWith("---\n")` sin tolerar BOM. Mismo género de bug, distinto escritor (`wikilinks-to-md.mjs`). No se toca: (a) fuera del contrato de esta tarea, (b) hoy el corpus tiene 0 BOM y `check-docs` gritaría antes de que `wikilinks-to-md` corrompa nada. Si reaparece un BOM, el fix es simétrico: prefijo `{text: bom, safe: false}` para que `segmentsRoundTrip` siga cerrando.
- **NOTICED BUT NOT TOUCHING:** el `--self-test` no está cableado en `docs-gates.yml`, así que hoy solo corre si alguien lo invoca. Cablearlo es una línea en el job de self-tests del workflow (donde ya viven `check-npm-versions --self-test`, `sync-workspace-version --self-test` y `wikilinks-to-md --self-test`), pero `.github/**` está fuera del alcance autorizado de esta tarea. Queda para el owner de CI.
- **NOTICED BUT NOT TOUCHING:** `scripts/docs/lib.mjs` y `stamp-frontmatter.mjs` ya incumplían `prettier --check` en `HEAD` (junto con `gen-index.mjs`, `gen-surface-manifest.mjs`, `sync-workspace-version.mjs`: 5 de los scripts del área). No se reformatean: `prettier --write` reescribiría ~70 líneas ajenas al cambio y enterraría el fix. Ningún workflow de CI corre prettier (solo el hook local opcional).
- **Carrera con `FIND-485`:** ese worker deja `docs/dev/tasks/FIND-485.md` sin commitear. `gen-index.mjs --write` es un artefacto del árbol COMPLETO, así que el índice se regeneró con el corpus propio (724 docs) y su entry queda a cargo de su propio cierre. Efecto medido: `gen-index --check` → exit 0 sin su fichero, exit 1 mientras esté untracked. No se commitea nada suyo.
- `stampDocument` queda sin `export` a propósito: no tiene consumidores fuera del fichero, y exportarlo invitaría a testear una copia en vez del run real.
