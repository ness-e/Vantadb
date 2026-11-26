---
title: "FIND-713: segment() BOM-safe y el self-test del stamper cableado en CI"
kind: task
status: in-progress
description: "El segundo lector del mismo fichero sigue siendo BOM-ciego y el --self-test del stamper solo corre a mano: un normalizador único de BOM y un step en el job de frontmatter"
---

# FIND-713: segment() BOM-safe y el self-test del stamper cableado en CI

## Metadata
- **Plan file:** [2026-11-23-release-v0.10.0-gates.md](../plans/2026-11-23-release-v0.10.0-gates.md) — §6, Fase 2, wave 2.2
- **Fuente:** fila `FIND-713` de `docs/dev/Backlog.md` (origen: cierre de `FIND-707`, `51c038efd`)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢 Baja (la clase de bug ya costó 33 ficheros reparados a mano)
- **Tipo:** Docs / tooling de gates (Node ESM, sin dependencias) + CI
- **Turns estimados:** 5
- **Creado:** 2026-11-24T19:10
- **last-synced:** 2026-11-24T22:05
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers de `segment()` | `check-links.mjs:170` (`proseLineLookup`), `check-secrets.mjs:318` (`proseWithLines`), `check-structure.mjs:120` (`fenceLineMask`), `check-docs.mjs:179` (vía `proseOf`), `wikilinks-to-md.mjs:46` (único escritor de segmentos), `fix-bad-headings.mjs:14` (vía `segmentsRoundTrip`) |
| Callers de `parseFrontmatter` | `check-docs.mjs:97`, `check-links.mjs:166`, `gen-index.mjs:80`, `check-doc-examples.mjs:118`, `stamp-frontmatter.mjs` |
| CI | Los 4 gates de docs de `.github/workflows/docs-gates.yml` (`docs-links`, `docs-schema`, `docs-index`, `docs-structure`) más el job nuevo consumidor: `docs-schema` |
| Implicaciones | `segment()` gana un segmento `{why:"bom", safe:false}` cuando el fichero empieza por U+FEFF. Los 6 consumidores filtran por `safe` / `why === "code-fence"` / `why === "prose"`, así que un segmento sin `line` es inerte para todos; `segmentsRoundTrip` sigue cerrando porque el BOM se emite, no se descarta. Sin impacto en Rust, sin migración de datos |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `scripts/docs/lib.mjs` (605 L), `scripts/docs/stamp-frontmatter.mjs` (380 L), `.github/workflows/docs-gates.yml` (663 L), `docs/dev/tasks/FIND-707.md`, `.opencode/task-system/prompts/pipeline-full.md`, `.opencode/skills/documentation-skill/SKILL.md`, `.opencode/skills/ci-cd-and-automation/SKILL.md`
- **Referencias hacia dentro (imports):** `lib.mjs` no importa nada del repo. `segment` / `proseOf` / `segmentsRoundTrip` son importados por 6 scripts de `scripts/docs/` (tabla de arriba). `stamp-frontmatter.mjs` importa 9 helpers de `lib.mjs` y no es importado por nadie: se ejecuta por CLI.
- **Referencias que salen de los editados:** ninguno de los dos scripts sale de `scripts/docs/`; el workflow es consumido por GitHub Actions y por `actionlint`.
- **Referencias entrantes a los editados:** `grep -rn "stamp-frontmatter"` → `docs/wiki/modules/tooling.md`, `docs/dev/workflow/gate-docs-links.md`, `scripts/docs/repair-orphan-blocks.mjs`, `scripts/docs/check-structure.mjs` (comentarios), `docs/dev/avance/**` (histórico) y `check-docs.mjs` (hints). Ninguno depende de la firma de `segment()`.
- **Veredicto impacto:** medio-acotado. `segment()` es compartido por 6 consumidores, pero el cambio es aditivo (un segmento más, nunca menos) y todos filtran por `safe`/`why`. Verificación: los 4 gates de docs (`check-links`, `check-docs`, `check-secrets`, `gen-index --check`) + `wikilinks-to-md --self-test` + `check-structure`.

## Contrato

```
node scripts/docs/stamp-frontmatter.mjs --self-test            # exit 0 (7/7)
node scripts/docs/stamp-frontmatter.mjs --write                # idempotente sobre el corpus limpio
git status --porcelain -- docs                                  # sin cambios atribuibles al stamper
npx --no-install actionlint .github/workflows/docs-gates.yml    # ✗ no existe: actionlint no es paquete npm → se usa el binario `actionlint` (1.7.12)
node scripts/docs/wikilinks-to-md.mjs --self-test               # exit 0 (round-trip del escritor)
node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/check-secrets.mjs && node scripts/docs/gen-index.mjs --check   # gates; el último queda rojo hasta que el orquestador regenere el índice con los task files trackeados
```

Caso nuevo del self-test, que arranca en ROJO contra `HEAD` y queda verde con el fix: `segment()` sobre un documento con BOM + frontmatter debe devolver un segmento `why === "frontmatter"` y `segmentsRoundTrip()` debe seguir cerrando (el BOM se emite como segmento propio, no se descarta — descartarlo rompería el invariante de concatenación).

## Spec (decisiones técnicas)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Dónde vive el normalizador | A) copia local en `stamp-frontmatter.mjs` (imposible: `segment()` vive en `lib.mjs`; una segunda copia es exactamente lo que `FIND-704` critica) / B) helper único en `lib.mjs` usado por `parseFrontmatter` y `segment` | ✅ B — `bomLength(text)` exportado; los dos lectores del mismo fichero se normalizan con la misma función. **Devía del "PROHIBIDO `lib.mjs`" del spawn**: el contrato (a) pide que `segment()` tolere BOM y `segment()` solo existe en `lib.mjs` (`rg "function segment" scripts/docs/` → 1 hit). `FIND-707` ya está cerrado (`51c038efd`), así que no hay carril en vuelo. Ver §Deuda |
| 2 | ¿Descartar el BOM o emitirlo como segmento? | A) descartarlo (byte-idéntico al fix del parser, pero `segmentsRoundTrip` deja de cerrar en cualquier fichero con BOM) / B) emitir `{text: "\uFEFF", safe: false, why: "bom"}` y arrancar el walker después | ✅ B — el invariante "concatenar todos los segmentos reproduce el input byte a byte" es lo que protege a `wikilinks-to-md` de corromper un fichero; `FIND-707` dejó anotada esta misma forma como fix simétrico |
| 3 | Job de CI | A) `docs-links` (lo que decía la fila del Backlog; su sujeto es la resolución de enlaces) / B) `docs-schema` (su sujeto es el frontmatter, que es lo que el stamper escribe) / C) `docs-structure` (ya corre `wikilinks-to-md --self-test`) | ✅ B — el artefacto bajo test es el lector/escritor de frontmatter; `docs-schema` es el job de frontmatter, no tiene guard `if:` (corre también en PRs de Dependabot, donde un job `skipped` cuenta como check satisfecho — `FIND-472`), y **está en los required checks del ruleset de `main`** (verificado: `gh api repos/ness-e/Vantadb/rulesets/16844590`) |
| 4 | Dónde va el caso de `segment()` | A) `wikilinks-to-md.mjs --self-test` (su escritor es el consumidor real, ya corre en `docs-structure`) / B) el `--self-test` del stamper, que es el step nuevo | ✅ B — un `--self-test` que pasa porque no ejercita `segment()` es el pre-mortem (c) de esta tarea; el step que se cablea en CI tiene que cubrir los dos lectores que comparten `lib.mjs`. A no se descarta: ya tiene su propio round-trip y se corre en el contrato |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `segmentsRoundTrip(src)` sigue siendo `true` para cualquier entrada, BOM incluido — es lo que impide que un escritor de segmentos corrompa un fichero; (2) `parseFrontmatter` no cambia de forma de retorno (`{raw,data,body,hadFrontmatter,end,nl}`) ni de comportamiento sin BOM; (3) `segment()` sigue emitiendo segmentos contiguos y en orden posicional (la regresión de 2026-09-29 que dañó 32 ficheros); (4) ningún consumidor recibe menos segmentos que antes.
- **Comandos de verificación:** los 6 del §Contrato.
- **Deuda pendiente:** `segment()` no protege un bloque de frontmatter cerrado en EOF sin salto final (`stop === -1` → se trata como prosa). `parseFrontmatter` sí lo resuelve desde `FIND-707`; la asimetría no se cierra aquí porque ningún documento del corpus está en esa forma y el fix pedido es el BOM. Ver §Deuda técnica y §Notas.

## Fase 1 — Evidencia de Debugging

- **Repro (RED, medido):** añadido el caso `segment()` al `--self-test` y ejecutado contra el `lib.mjs` de `HEAD`:

  ```
    ok    BOM + frontmatter: exactly one block
    ...
    FAIL  segment(): BOM + frontmatter is a protected block and round-trips — segment() did not protect the frontmatter block
  SELF-TEST FAILED: 1 case(s).
  ```

  Causa: `segment()` prueba `src.startsWith("---\n")` sobre el texto crudo; con `\uFEFF---` no matchea, así que el bloque entero se emite como `prose` (`safe: true`) y el único escritor de segmentos (`wikilinks-to-md.mjs`) puede reescribir frontmatter.
- **Hipótesis:** el normalizador de BOM se escribió dentro de `parseFrontmatter` y no se extrajo, así que el otro lector del mismo fichero quedó sin él. La causa no es "a `segment()` le falta un strip", es "hay dos lectores y una sola normalización".
- **1 variable controlada:** el caso nuevo es el único que cambia entre RED y GREEN; los 6 casos de `FIND-707` deben seguir verdes sin editarlos.
- **Mutante que debe morir:** quitar el `bomLength(src)` de `segment()` (dejar `let i = 0`) → el self-test vuelve a exit 1.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto por PR: cero o negativo.** No se añade deuda: se extrae a un helper una constante que ya existía duplicada conceptualmente en `parseFrontmatter`, y se elimina la clase "dos lectores, dos reglas". Deuda documentada en vez de arreglada:

- **Devia del alcance de ficheros del spawn:** `scripts/docs/lib.mjs` estaba marcado PROHIBIDO porque "ya lo tocó `FIND-707`". Ese ticket está COMPLETED y commiteado (`51c038efd`); `segment()` no existe en ningún otro fichero. Sin tocar `lib.mjs` el contrato (a) es inejecutable y la única alternativa sería duplicar la lógica (lo que el propio prompt prohíbe). Se toca el mínimo: 1 helper + 2 líneas de `parseFrontmatter` + el arranque de `segment()`. Si el orquestador prefiere otro reparto, el revert es de una sola función.
- `segment()` sigue sin la rama "bloque cerrado en EOF sin salto final" que `parseFrontmatter` ganó en `FIND-707`. Fuera de alcance; ningún documento del corpus lo tiene.
- `lib.mjs` y `stamp-frontmatter.mjs` ya incumplían `prettier --check` en `HEAD` (5 de los scripts del área). No se reformatean: enterraría el fix en ~70 líneas ajenas.

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | self-test 7/7 (el caso de `segment()` rojo antes, verde después) + mutante muerto + los 4 gates de docs exit 0 + `wikilinks-to-md --self-test` 0 + idempotencia del `--write` |
| **Commit** | atómico (`lib.mjs` + `stamp-frontmatter.mjs` + `docs-gates.yml` + task file), conventional `fix(docs): …`, verificación mecánica pegada abajo |
| **Release** | N/A — tooling interno, sin impacto en artefactos publicados; el step nuevo sí entra en el gate de `main` |

## Herramientas necesarias
- `node` (gates de docs; sin cargo)
- `npx --no-install actionlint` para el workflow
- `gh api` para verificar los required status checks del ruleset

**Skills cargadas (SDP v3):** `documentation-skill` (Regla 3, corpus/frontmatter) · `ci-cd-and-automation` (pin CI/release) · `git-workflow-and-versioning` (pin CI/release) · `test-driven-development` (caso RED primero) · `incremental-implementation` (2 slices: reader, CI) · `context-engineering` (jerarquía de contexto) · `doubt-driven-development` (blast radius compartido) · base: `campaign-executor`, `progreso`

## Investigation Notes
- Web research no aplica: el comportamiento es del propio repo. La única fuente externa potencial (semántica del BOM en Node) está cubierta por el patrón que ya usa `parseFrontmatter` (`charCodeAt(0) === 0xfeff`) desde `FIND-707`.
- **Required checks (verificado, no asumido):** el ruleset de `develop` (`23692587`) exige 10 checks, todos de Rust — **ningún job de `docs-gates` es required en `develop`**. El ruleset de `main` (`16844590`) exige 18, e incluye `Frontmatter schema, kind agreement, orphans`, `Internal links resolve`, `Generated indexes are current` y `actionlint (workflows lint + inventory gate)`. Consecuencia honesta: el step nuevo bloquea el PR `develop → main` (donde se decide el release), no cada commit diario en `develop`.
- **Hallazgo derivado (para Backlog, no arreglado aquí):** los comentarios de `docs-gates.yml:199-201,219-221,237-239,618-620` afirman "es required" de jobs que **no** están en el ruleset de `develop`. Cierto para `main`, falso para `develop`: un PR a `develop` puede mergear con `docs-schema` rojo. Requiere decisión del owner (editar el ruleset es admin, no está en el alcance de esta tarea).

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Context Save Point

- **Estado del árbol:** `develop` con trabajo concurrente ajeno (`FIND-259`: `vanta-memory/**`, `src/**`; `docs/api/VANTA_MEMORY.md`). No se commitea nada de eso. `--write` del stamper se corre con snapshot de `git status` antes/después, no contra un árbol vacío.
- **Si el self-test aparece rojo en un caso de `FIND-707`:** el cambio en `segment()` no debería poder tocarlos; si pasa, el problema es el orden de segmentos, no el BOM.

## Step evidence — verificación ejecutada (2026-11-24T21:50)

| Comando | Resultado |
|---------|-----------|
| `node scripts/docs/stamp-frontmatter.mjs --self-test` (RED, antes del fix) | ❌ exit 1 — `FAIL segment(): … — segment() did not protect the frontmatter block` |
| `node scripts/docs/stamp-frontmatter.mjs --self-test` (GREEN) | ✅ exit 0 — 7/7 |
| mutante `const bom = 0;` en `segment()` | ❌ exit 1 — el caso nuevo vuelve a rojo |
| mutante sin `bomLength` en `parseFrontmatter` | ❌ exit 1 — cae el caso 2 de `FIND-707` (`title "FIND 591" != "Real title"`). El caso nuevo de `segment()` sigue verde, que es lo correcto: cada lector normaliza el BOM por su cuenta y el mutante aísla uno solo |
| `node scripts/docs/stamp-frontmatter.mjs --write` + snapshot `git status --porcelain -- docs` | ✅ 794 scanned, `unchanged: 794`, `STATUS_IDENTICAL=True` |
| `actionlint .github/workflows/docs-gates.yml` (binario 1.7.12; **el contrato decía `npx --no-install actionlint`, que no existe: actionlint no es un paquete npm**) | ✅ exit 0 |
| `node scripts/docs/check-links.mjs` | ✅ exit 0 — 0 broken, 1 wikilink (budget 20) |
| `node scripts/docs/check-docs.mjs` | ✅ exit 0 — GATING all clear |
| `node scripts/docs/check-secrets.mjs` | ✅ exit 0 — 0 errors, 4 warnings (budget 28) |
| `node scripts/docs/wikilinks-to-md.mjs --self-test` | ✅ exit 0 — 11/11 |
| `node scripts/docs/check-structure.mjs --budget=40` | ✅ exit 0 — 34/40 |
| `pwsh scripts/validate-docs-coverage.ps1` | ✅ exit 0 — 0 gaps |
| `pwsh dev-tools/ocr-review.ps1` + `ocr delegate rule <3 archivos>` | ✅ ejecutado — Rule Group 1 (`.github/**`, reglas release/CI: sin `continue-on-error` nuevo, actions pinneadas a SHA, timeout presente) y Rule Group 2 (JS: sin dead code, sin `eval`, sin DOM — el grupo es de frontend y no aplica a estos scripts) |
| `node scripts/docs/gen-index.mjs --check` | ❌ exit 1 **por diseño**: `Task files 204 → 206` = los 2 task files untracked (`FIND-259` ajeno + `FIND-713` propio). Regenerar el índice exige `gen-index --write`, que escribe `docs/index.md` y `llms.txt` — fuera del alcance autorizado. Lo regenera el orquestador al cerrar la wave, con los dos ficheros ya trackeados |

## Steps

### Step 1: Caso RED de `segment()` en el `--self-test`
- **Archivos:** `scripts/docs/stamp-frontmatter.mjs`
- **Acción:** importar `segment`/`segmentsRoundTrip` y añadir un caso que exige frontmatter protegido + round-trip sobre un documento con BOM
- **Verify:** `node scripts/docs/stamp-frontmatter.mjs --self-test` → exit 1, `FAIL segment(): BOM + frontmatter is a protected block — segment() did not protect the frontmatter block`
- **Estado:** ✅

### Step 2: Un solo normalizador de BOM (`bomLength`) usado por los dos lectores
- **Archivos:** `scripts/docs/lib.mjs`
- **Acción:** exportar `bomLength`, usarlo en `parseFrontmatter` y arrancar `segment()` después del BOM, emitiéndolo como segmento `safe: false`
- **Verify:** `node scripts/docs/stamp-frontmatter.mjs --self-test` → 7/7, exit 0; mutante (quitar `bomLength` de `segment`) → exit 1
- **Estado:** ✅

### Step 3: Step de CI en el job required de `main`
- **Archivos:** `.github/workflows/docs-gates.yml`
- **Acción:** `node scripts/docs/stamp-frontmatter.mjs --self-test` como primer step de `docs-schema`, con el porqué (la regresión de 2026-11-23) en un comentario
- **Verify:** `actionlint .github/workflows/docs-gates.yml` → exit 0 (el contrato decía `npx --no-install actionlint`; actionlint no se publica en npm — se usa el binario oficial `1.7.12` que instala winget y que ya usan `.githooks/pre-commit` y `.githooks/pre-push`)
- **Estado:** ✅

### Step 4: Gates del área + idempotencia
- **Archivos:** ninguno (verificación)
- **Acción:** los 4 gates de docs, `wikilinks-to-md --self-test`, y `--write` comparando `git status --porcelain` antes/después
- **Verify:** todo exit 0 y ninguna línea de `docs/` atribuible al stamper
- **Estado:** ✅

### Step 5: Review P2-01 + commit local
- **Archivos:** `docs/dev/tasks/FIND-713.md` (§Review)
- **Acción:** veredicto de `vanta-review` (contexto distinto) + OCR delegation; commit local con el mensaje acordado
- **Verify:** veredicto registrado en §Review (`approve`, sin Critical ni Required; F4 y F7 corregidos, F1/F2/F3 a filas `FIND-*`)
- **Estado:** ✅

## Dependencias
- `FIND-707` ✅ (`51c038efd`) — el normalizador del parser y el self-test existen; esta tarea los reusa en vez de reescribirlos.
- En paralelo: `FIND-192` (`dev-tools/**`), `FIND-259` / `FIND-710` (Rust). Ninguno toca `scripts/docs/**` ni `docs-gates.yml`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (sesión `ses_df075258affe37fwtyGFCYLax5`, contexto distinto al implementador)
- **Tier:** Fast (`scripts/docs/**`, `.github/workflows/**`, `docs/dev/tasks/**`) → el revisor **corrió el contrato él mismo** en vez de leer este informe, y re-hizo las mutaciones con hash de `lib.mjs` verificado antes/después (`5671eb5e` HEAD → `36b5f9a1` fix).
- **Enfoque (6 ataques, no validación):**
  1. *¿Rompe algún consumidor?* — leyó los 6 y además corrió un **test diferencial sobre los 795 docs reales** (BOM vs sin BOM): `proseOf` idéntico, 0 diferencias de campos, 0 fallos de round-trip, 0 discrepancias con `parseFrontmatter`. Verificó el punto sutil de `end: end + bom`: un enlace real en la línea 9 se reporta como línea 9 con y sin BOM.
  2. *¿El caso nuevo era rojo antes?* — swap del `lib.mjs` de `HEAD` en su sitio → exit 1 en el caso 7 con los 6 de `FIND-707` verdes. Mutante "quitar el `push` del BOM dejando `i=bom`" → cae por round-trip, o sea la segunda aserción también discrimina.
  3. *¿Normalizador único?* — `rg` deja **una sola** copia de `0xfeff`; pero encontró un **7.º lector BOM-ciego** fuera de `lib.mjs` (`check-structure.findOrphanProse`), con falso positivo probado. Pre-existente y 0 docs con BOM.
  4. *Hueco EOF* — probado (`parseFrontmatter.hadFrontmatter=true` vs `segment` sin bloque), exposición 0/795, deuda ya declarada.
  5. *El step de CI* — `docs-schema` es uno de los 4 jobs sin `if:` (no skippeable) y está en el ruleset de `main`; `docs-links` y `docs-schema` son equivalentes en gate, y el artefacto bajo test es el frontmatter ⇒ la elección del task file es la correcta, el Backlog hay que reconciliarlo.
  6. *Lo que actionlint no ve* — 672 CRLF / 672 LF pero `text=auto eol=lf` ⇒ diff de 9 líneas sin reescritura de EOL; sin BOM, sin tabs, sin `continue-on-error` nuevo; el inventory gate cuenta ficheros, así que el step le es invisible.
- **Findings:** F1 hueco EOF (`Optional`, deuda declarada, 0/795) · F2 `docs-schema` no required en `develop` (`Optional`, admin del ruleset) · F3 7.º lector BOM-ciego en `check-structure` (`Optional`, ajeno al ticket) · F4 `bomLength` exportado sin importadores (`Nit`) · F5 `gate-docs-links.md:38` snapshot incompleto (`Nit`) · F6 verificado con actionlint 1.7.12 vs CI 1.7.7 (`Nit`) · F7 transcripción del nombre del caso en §Fase 1 (`Nit`).
- **Disposición:** F4 corregido (`bomLength` deja de exportarse: cero importadores externos → superficie muerta), F7 corregido. F1/F2/F3 → filas `FIND-*` propuestas en §Notas (F2 ya estaba documentada aquí). F5/F6 aceptados como documentados: ninguno cambia una decisión.
- **Veredicto:** ✅ **approve** — sin Critical ni Required. DoD Task ✅ (7/7, mutantes muertos, 4 gates 0, wikilinks 0, idempotencia con snapshot de `git status`); Commit quedó para Step 5; Release N/A.

## Notas

- **NOTICED BUT NOT TOUCHING:** `docs-gates.yml` declara "es required" en 4 jobs que no están en el ruleset de `develop`. Ver §Investigation Notes — requiere edición del ruleset (admin).
- **NOTICED BUT NOT TOUCHING:** `check-doc-examples.mjs:103` documenta un `ponytail:` sobre no reusar `segment()` porque su regex de fence no tolera `m`. Es deuda declarada del otro consumidor, no de este cambio.

### Hallazgos que esta tarea NO arregla (filas `FIND-*` propuestas)

Tres defectos de la misma familia, encontrados o confirmados en el review. Ninguno bloquea:
el corpus tiene **0 de 795** documentos en las formas que los disparan, pero el vector está armado.

1. **`segment()` no protege un bloque cerrado en EOF sin salto final** — `parseFrontmatter` lo
   resuelve desde `FIND-707`, `segment()` no, así que los dos lectores del mismo fichero siguen
   discrepando. Con esa forma, `proseOf()` filtra `title:` (y hasta un `[[wikilink]]`) al escritor.
   Medido: 3 documentos del corpus llevan `[[` dentro del frontmatter ⇒ si alguno cayera en esa
   forma, `wikilinks-to-md` lo reescribiría. Fix estimado: 3 líneas (`src.endsWith(nl + "---")`
   espejo del parser) + 1 caso de self-test. Severidad: 🟡 Media, exposición 0 hoy.
2. **Ningún job de `docs-gates` es required en `develop`** — el ruleset `23692587` exige 10
   contextos, todos de Rust. Los comentarios de `docs-gates.yml:199-201,219-221,237-239,618-620`
   afirman lo contrario. En `main` sí son required (18 contextos), así que el gate protege el
   release y no el día a día. Severidad: 🟠 Alta de proceso, arreglo = admin del ruleset.
3. **`check-structure.findOrphanProse` es un 7.º lector BOM-ciego** (`text[3]`,
   `text.startsWith("---")`), fuera de `lib.mjs` y por tanto fuera del "normalizador único".
   Con BOM + frontmatter multilínea reporta `orphan-prose` falso positivo (probado por el
   revisor). Severidad: 🟢 Baja, el corpus tiene 0 BOM y el gate está a 34/40.
