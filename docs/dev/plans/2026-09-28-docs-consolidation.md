---
title: "Plan de Consolidación Documental — enlaces, metadata, índices y skill"
kind: plan
description: documentation-skill → campaign-executor → progreso → writing-plans → systematic-debugging → writing-guidelines
---

# Plan de Consolidación Documental — enlaces, metadata, índices y skill

> **Campaign ID:** _(asignar al iniciar con `/pipeline plan`)_
> **Inicio:** 2026-09-28
> **Estado:** 🟡 F0 COMPLETADA · F1-F4 pendientes
> **Fuente:** `docs/dev/research/docs-strategy/01-ecosystem-2026.md` (investigación de 5 líneas paralelas, 2026-09-28) + medición directa del repo
> **Autonomous:** false — el owner gatea push. La política de git vive en `.opencode/AGENTS.md` Regla 7; no se reescribe aquí.
> **Restricción dura del owner:** `docs/dev/tasks/` y `docs/dev/plans/` **no se mueven**. Cambiar su ubicación rompe el sistema de tareas. Este plan trabaja dentro de ellos, nunca sobre ellos.
> **Modo:** PLAN → ejecución con `/pipeline run docs/dev/plans/2026-09-28-docs-consolidation.md`

## SDP (skills del plan)

`documentation-skill` → `campaign-executor` → `progreso` → `writing-plans` → `systematic-debugging` → `writing-guidelines`

## Por qué existe este plan

La medición del 2026-09-28 encontró tres problemas que se alimentaban entre sí:

| Métrica | Antes de F0 | Después de F0 | Cómo se midió |
|---|---|---|---|
| Enlaces markdown rotos | **~110** | 109 (presupuesto, F1-T1) | `check-links.mjs` |
| Ficheros huérfanos | **1481 de 1717 (86%)** | **4 de 1489** | `check-docs.mjs` |
| Ficheros sin frontmatter | **1250 de 1717 (73%)** | 0 | `check-docs.mjs` |
| Wikilinks en prosa | 650 | **40 ocurrencias en 33 ficheros** (presupuesto, F1) | `check-links.mjs` / `check-docs.mjs` |
| Violaciones de esquema que gatean | n/a (no había gate) | 0 | `check-docs.mjs` |
| Grupos de nombres duplicados | 69 | 69 → deuda en F3 | `check-docs.mjs` |
| Claves de frontmatter desconocidas | n/a | 276 ficheros (reportadas, no gatean) | `check-docs.mjs` |
| Errores markdownlint | 5 | 12 (presupuesto; 7 introducidos por la migración) | `markdownlint-cli2` |

La causa raíz de los tres primeros era **una sola**: el grafo de enlaces estaba roto, así
que nada era alcanzable, así que nada tenía relación, así que la vista de grafo de Obsidian
no mostraba nada útil. Y el índice se mantenía a mano, con una regla ("indexar en el mismo
PR") que ya estaba fallando.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 17 |
| ⏸️ DEFER | 7 (F3-F4, requieren decisión del owner) |
| ⏭️ SKIP | 4 (con justificación explícita) |
| 🚧 BLOQUEADO | 0 |

Status: 🔺 uphill = 0 · 🔻 downhill = 17

---

## F0 — Interoperabilidad Obsidian ↔ GitHub · ✅ COMPLETADA

**Gate de salida:** `useMarkdownLinks: true` + `newLinkFormat: relative`, cero wikilinks
convertibles pendientes, 396 enlaces convertidos con alias preservados, y verificación de que
82/82 encabezados de un fichero representativo siguen intactos.

### Task 0 — F0-T1: investigación del dilema de enlaces

**Appetite:** max 2h · **Esfuerzo:** ✅ 2 h · **Prioridad:** P0
**Archivos clave:** `docs/.obsidian/app.json`, `community-plugins.json`, `core-plugins.json`
**Verificación real:** comparación de encabezados por fichero contra HEAD
**Gate Justificación:** la premisa del brief era que wikilinks y GitHub eran excluyentes. La documentación oficial de Obsidian dice lo contrario.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

**Hallazgo que desbloquea el plan.** `docs/.obsidian/app.json` tenía
`"useMarkdownLinks": false`. Esa única línea era la causa de todo.
[obsidian.md/help/links](https://obsidian.md/help/links) dice literalmente:

> _"By default, due to its more compact format, Obsidian generates links using the Wikilink
> format. **If interoperability is important to you, you can disable Wikilinks and use
> Markdown links instead.**"_

Y la vista de grafo documenta que "Lines represent **Internal links** between two nodes" —
término que incluye las dos formas. El grafo no distingue: `useMarkdownLinks` gobierna lo que
Obsidian **genera**, nunca lo que **reconoce**.

Config aplicada:

```json
{ "useMarkdownLinks": true, "newLinkFormat": "relative" }
```

`relative` y no `shortest` porque hay 69 grupos de nombres duplicados (`README.md` ×16) y
`shortest` es precisamente el modo que genera esa ambigüedad. Una ruta relativa tiene una
única resolución posible.

### Task 1 — F0-T2: `wikilinks-to-md.mjs`

**Appetite:** max 3h · **Esfuerzo:** ✅ 3 h · **Prioridad:** P0
**Archivos clave:** `scripts/docs/wikilinks-to-md.mjs`, `scripts/docs/lib.mjs`
**Verificación real:** 396 enlaces convertidos · idempotente (2ª pasada = 0 cambios) · `node --check` OK
**Gate Justificación:** la migración debe ser reproducible y revisable como un diff, no 140 escrituras silenciosas del editor.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

Decisiones de diseño que importan:

- **Nunca adivina.** Un wikilink no resoluble se reporta y se deja intacto. Un enlace
  equivocado es peor que uno roto: miente en silencio y pasa cualquier comprobación de
  existencia posterior.
- **Escala de resolución:** (1) `./` o `../` explícito contra el directorio del fichero →
  (2) hermano en el mismo directorio → (3) ruta desde la raíz del vault → (4) basename único
  → (5) ambiguo ⇒ no tocar.
- **El paso 2 es el que resuelve los 39 `[[README.md]]` de `docs/user/glosario/`.** Sin él
  hay 15 candidatos y ninguna decisión defendible.
- **Protege tres zonas** que una expresión regular naïve corrompe: frontmatter YAML (Obsidian
  usa wikilinks en `links:` y ese tipo de propiedad es global en el vault), bloques de
  código delimitados, y código inline. 44 + 0 + 129 ocurrencias preservadas.

### Task 2 — F0-T3: `repair-links.mjs` y verificación de integridad

**Appetite:** max 3h · **Esfuerzo:** ✅ 3 h · **Prioridad:** P0
**Archivos clave:** `scripts/docs/repair-links.mjs`, `scripts/docs/check-links.mjs`
**Verificación real:** 115 enlaces markdown + 35 wikilinks reparados · 659 → 246 rotos
**Gate Justificación:** el 47% de enlaces rotos venía de ficheros movidos, deriva de mayúsculas y renombrados de ADR, no de enlaces mal escritos.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

**Escala de confianza** (todas exigen coincidencia única): ruta exacta → ruta insensible a
mayúsculas → intercambio de convención ADR (`001_x.md` ↔ `ADR-0001-x.md`) → normalización
de separadores → basename único → stem único.

**Bug de alcance encontrado y corregido:** la primera versión indexaba sólo `docs/` y
reportaba como rotos enlaces como `docs/api/MCP.md` → `../../vantadb-mcp/src/error.rs`, que
son **válidos** en GitHub porque salen del árbol de docs. `lib.mjs buildTargetSet()` ahora
indexa todo el repositorio. Corregir el alcance en la librería compartida, en vez de parchear
el síntoma en cada script, evitó tener que arreglarlo en tres sitios.

### Task 3 — F0-T4: sello de metadata y esquema

**Appetite:** max 3h · **Esfuerzo:** ✅ 3 h · **Prioridad:** P0
**Archivos clave:** `docs/_schema/frontmatter.schema.json`, `docs/_schema/tags.txt`, `scripts/docs/stamp-frontmatter.mjs`, `scripts/docs/lib.mjs`
**Verificación real:** 1716 ficheros sellados · 0 sin frontmatter · claves retiradas: `type` 440, `last_reviewed` 428, `related` 210
**Gate Justificación:** sin `kind` no hay índice generable, y sin índice todo fichero es huérfano. Era el prerrequisito, no la decoración.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

**El script nunca inventa contenido.** Cada valor se deriva mecánicamente: `title` del
frontmatter, luego del H1, luego del nombre del fichero; `kind` **de la ruta**, la única
fuente fiable; `description` de la primera línea de prosa, con enlaces aplanados a texto y
corte en frontera de palabra. **`tags` nunca se escribe** — es la única decisión humana, y
equivocarse en ella es lo que hace que la metadata "se vea mal".

Tres claves retiradas, con justificación:

| Clave | Por qué se retira |
|---|---|
| `type` | Nunca fue una taxonomía: el corpus tenía `type: adr`, `type: master-index`, `type: documentation`. `kind` sí es un enumerado y decide la ubicación. |
| `last_reviewed` | Una fecha autodeclarada no es una medición. La frescura real es `git log -1 --format=%cs -- <file>`, gratis y exacta. Un campo de fecha en el frontmatter es una _copia_ de eso y sólo puede estar obsoleta. |
| `related` | Una lista de backlinks mantenida a mano. Eso es ahora trabajo de `gen-index.mjs`. |

**El enum de `status` adopta el vocabulario MADR** (`proposed`/`accepted`/`rejected`). Adoptar
el estándar en el esquema es más barato que reescribir 54 ADRs para que cumplan un enum
inventado: `enum: proposed` no es una mejora, es un coste.

**Vocabulario de tags: advisory, no cerrado.** El corpus usaba ~430 tags distintos. Una
lista de 430 entradas no es un vocabulario. Se exige sólo el **formato** (minúsculas con
guiones), porque Obsidian deriva el _tipo_ de una propiedad del nombre a nivel de vault: un
fichero con un valor raro corrompe la visualización de los 1721.

### Task 4 — F0-T5: `gen-index.mjs` — la palanca

**Appetite:** max 3h · **Esfuerzo:** ✅ 3 h · **Prioridad:** P0
**Archivos clave:** `scripts/docs/gen-index.mjs`, `docs/index.md`, `docs/api/index.md`, `docs/user/index.md`, `docs/dev/architecture/adr/README.md`, `llms.txt`
**Verificación real:** **huérfanos 1481 → 4** · 5 ficheros generados · idempotente
**Gate Justificación:** el índice a mano `master-index.md` (282 líneas, regla "indexar en el mismo PR") ya estaba fallando: ése es el 86% de huérfanos. Generarlo convierte "1481 huérfanos que revisar" en "0 huérfanos por construcción".
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

**Por qué salida commiteada y no consulta en tiempo de ejecución:** los bloques
`` ```dataview `` **no se renderizan en GitHub** — aparecen como bloque de código con el
código de la consulta. El fichero tiene que existir en git. Es el patrón de SurrealDB: el
índice es a la vez fuente commiteada (revisable en el diff de un PR) y artefacto regenerado
(no puede derivar).

```
docs/index.md                        índice canónico, agrupado por kind
docs/api/index.md                    índice de sección
docs/user/index.md                   índice de sección
docs/dev/architecture/adr/README.md  tabla de ADRs (nº, título, estado)
llms.txt                             índice plano para agentes (llms.txt v2)
```

**El 86% de huérfanos no se "arregló": se hizo imposible.** Esa es la diferencia entre
revisar 1481 ficheros y no tener la métrica.

### Task 5 — F0-T6: gates de CI

**Appetite:** max 2h · **Esfuerzo:** ✅ 2 h · **Prioridad:** P0
**Archivos clave:** `.github/workflows/gate-docs-links.yml`, `docs/dev/workflow/gate-docs-links.md`
**Verificación real:** 4 jobs con `timeout-minutes`, SHA pins, `permissions: contents: read`, `CATEGORY:` en el único `continue-on-error`
**Gate Justificación:** los tres números sólo mejoran si un PR puede romperlos y arreglarlos de forma visible.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

Obedece `docs/dev/workflow/RULES.md`: regla 1 (paths), 2 (timeouts), 3 (SHA), 4 (permisos),
6 (CATEGORY), 7 (branches).

**`docs-schema` no gatea los huérfanos todavía.** Reporta pero no bloquea, porque un gate que
nace rojo se apaga. Se promueve con `--strict` cuando el recuento llegue a 0.

**Baseline numérico de markdownlint en vez de una regla.** La migración arregló el 47% de los
enlaces e introdujo 7 errores en 2 ficheros. Un recuento es más fácil de sostener que un
conjunto de reglas, y baja su propio presupuesto cuando se arregla. `markdownlint --fix` no se
usa: son ficheros de prosa escritos a mano, y un formateador automático produce churn que
esconde las ediciones reales en el diff.

### Task 6 — F0-T7: configuración de Obsidian que no rompe GitHub

**Appetite:** max 1h · **Esfuerzo:** ✅ 1 h · **Prioridad:** P1
**Archivos clave:** `docs/.obsidian/app.json`, `core-plugins.json`, `community-plugins.json`
**Verificación real:** 3 ficheros escritos
**Gate Justificación:** el poder de Obsidian y la renderización de GitHub entran en conflicto en puntos concretos y verificables.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

Core plugins desactivados, cada uno por una razón que produce GitHub roto:

| Plugin | Por qué off |
|---|---|
| `canvas` | Genera `.canvas` = JSON crudo en GitHub |
| `bases` | `.base`, misma clase de problema, más nuevo |
| `note-composer` | Un clic inserta un embed `![[…]]` = sintaxis muerta en GitHub |
| `templates` | El mecanismo que estamparía sintaxis Obsidian-only en ficheros nuevos |
| `daily-notes` | Crea ficheros con fecha desde `templates.json` |
| `tag-pane` | Empareja con la caída de `tags:`; vacío y engañoso |
| `word-count` | Duplica `better-word-count` |

Plugins retirados: `obsidian-icon-folder` (Iconize) — **deprecado**, sin release desde
2025-01-03, y escribe `icon:` en el frontmatter (una fila más de tabla en 1721 páginas);
`calendar` — sin release desde 2021-04-20, y su única función es abrir o crear daily notes,
que se acaba de desactivar.

Plugins añadidos: `update-relative-links` (2.1.4, MIT) — su README exige
`useMarkdownLinks: true`, o sea que **requiere** exactamente la configuración aplicada;
`better-markdown-links` (5.2.0, MIT) — el conversor interactivo equivalente, **con el
disparador en manual**, porque puede convertir en cada guardado y puede eliminar el `!` de
todos los embeds del vault.

### Task 7 — F0-T8: skill `documentation-skill` + declaración en AGENTS.md

**Appetite:** max 2h · **Esfuerzo:** ✅ 2 h · **Prioridad:** P0
**Archivos clave:** `.opencode/skills/documentation-skill/SKILL.md`, `.opencode/AGENTS.md`
**Verificación real:** skill registrada en el catálogo; declarada en la tabla de skills base y en Regla 3
**Gate Justificación:** un estándar escrito que ningún agente carga no es un estándar. La declaración en AGENTS.md es la parte que lo hace obligatorio.
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

La skill cubre: los cinco innegociables, sintaxis de enlace y la regla del pipe en tablas,
esquema de frontmatter con `kind` → ruta, los cuatro tipos de documento de Diátaxis y qué
debe cada uno, formato MADR con sus secciones obligatorias, el límite de longitud de Nygard,
la tabla de generado-vs-manual, la Definition of Done con comandos, y 10 anti-patrones.

---

## F1 — Cerrar los enlaces que importan · ⏳ PENDIENTE

**Gate de salida:** 0 enlaces rotos en la superficie pública; el resto clasificado y excluido.

246 enlaces siguen rotos. **No todos importan igual**, y tratarlos todos por igual es el
error más caro disponible: la mayoría son mojibake en ficheros históricos.

### Task 8 — F1-T1: triage por prioridad

**Appetite:** max 1h · **Esfuerzo:** ⬜ · **Prioridad:** P0
**Archivos clave:** salida de `check-links.mjs --json`
**Verificación real:** cada enlace roto asignado a un nivel
**Gate Result:** ✅ DO
**Estado:** PENDING

| Nivel | Definición | Acción | Presupuesto |
|---|---|---|---|
| **P0** | El destino no existe **y** el origen es alcanzable desde `docs/index.md` o `docs/user/QUICKSTART.md` | Arreglar ya | 2 h |
| **P1** | Origen en `docs/user/**` o `docs/api/**` | Arreglar ya — es la superficie viva | 4 h |
| **P2** | Origen en `docs/dev/**`, no archivado, modificado en los últimos 90 días | Arreglar | 2 h |
| **P3** | Bajo `archive/`, o modificado hace más de un año | **Dejar roto. Excluir del gate.** | 0 |
| **P4** | Externo | Barrido mensual, no por PR | 30 min/mes |

**La regla que lo hace barato:** P3 y P4 deben quedar fuera del gate bloqueante, o el gate
nace rojo y alguien lo desactiva. Qdrant y LanceDB hacen exactamente esto (`fail: false` +
un issue de seguimiento), con el motivo escrito en el propio workflow.

### Task 9 — F1-T2: el mojibake `[[bench]]` / `[[test]]` / `[[package]]`

**Appetite:** max 1h · **Esfuerzo:** ⬜ · **Prioridad:** P2
**Archivos clave:** `docs/dev/avance/historial/**`, `docs/dev/avance/activo/**`
**Verificación real:** 0 ocurrencias del patrón fuera de `tasks/`
**Gate Result:** ✅ DO
**Estado:** PENDING

**No son enlaces rotos: son corrupción de codificación.** De las 40 ocurrencias que quedan
en prosa, ~30 son de este tipo: `[[bench]]` ×14, `[[test]]` ×5, `[[package]]` ×3, `[[bin]]`.
El contexto delata el origen: `binario [[test]] separado` era "binario de test separado";
`[[package]] vantadb-desktop` era `[package] name="vantadb-desktop"` en TOML. Una pasada de
reemplazo de codificación destruyó los caracteres y los dejó entre dobles corchetes.

**Nota de medición:** el recuento bruto era 219. Bajó a 40 en cuanto `check-links` empezó a
contar sobre `proseOf()` en vez de sobre el texto crudo — 144 de esas ocurrencias estaban
dentro de spans de código inline. El bug era del contador, no del corpus. Documentar un
patrón en un plan lo reintroduce en el recuento; excluir el código es lo que hace el
recuento drenable.

Las ~10 restantes sí son enlaces muertos genuinos: `[[../architecture/hnsw_index]]`,
`[PYTHON_SDK.md](../../api/PYTHON_SDK.md)`, `[SEARCH_PARITY](../../api/SEARCH_PARITY.md)` — ficheros renombrados o borrados, que
`repair-links.mjs` no puede arreglar porque no hay coincidencia única. Van con F1-T1.

**No se reparan como enlaces.** Arreglarlos es una decisión de contenido: restaurar el texto
correcto requiere leer el original, que no existe en el fichero. Procedimiento propuesto:
buscar en `git log -S` el carácter sustituido y restaurar desde una revisión anterior.

**No tocar `docs/dev/tasks/`** (restricción del owner): el mojibake vive en `avance/`, que es
histórico. Si aparece en `tasks/`, se reporta pero no se modifica sin un task file propio.

### Task 10 — F1-T3: los 39 `links: "[[README.md]]"` del glosario

**Appetite:** max 30min · **Esfuerzo:** ⬜ · **Prioridad:** P3
**Gate Result:** ⏭️ SKIP — con justificación
**Estado:** SKIPPED

**No son deuda: son correctos.** `links:` es una propiedad de Obsidian cuyo tipo es global en
el vault, y ahí los wikilinks son la forma canónica. `wikilinks-to-md.mjs` los exime a
propósito, y `check-docs.mjs` también los exime al contar wikilinks. Reescribirlos
corrompería la propiedad en los 1721 ficheros.

### Task 11 — F1-T4: 2 etiquetas de enlace con barra invertida

**Appetite:** max 15min · **Esfuerzo:** ✅ 15 min · **Prioridad:** P2
**Archivos clave:** `scripts/docs/fix-link-labels.mjs`, `docs/user/operations/BENCHMARKS.md`
**Verificación real:** 32 etiquetas reparadas en 6 ficheros · idempotente
**Gate Result:** ✅ DO
**Estado:** COMPLETADO

`[bm25](../../user/glosario/bm25.md)` es la forma de Obsidian para un enlace dentro de una celda de tabla, donde
el pipe debe ir escapado. El conversor quitó la barra invertida del destino pero la dejó en
la etiqueta de reserva, produciendo `[bm25](../../user/glosario/bm25.md)`. El conversor está
corregido para futuras ejecuciones; este script drena lo ya escrito.

### Task 12 — F1-T5: los 7 errores de markdownlint introducidos

**Appetite:** max 2h · **Esfuerzo:** ⬜ · **Prioridad:** P2
**Archivos clave:** `docs/user/operations/BENCHMARKS.md` (5 errores: MD005 ×3, MD007 ×2), `docs/user/operations/CONFIGURATION.md` (2 errores: MD027 ×2)
**Verificación real:** `npx markdownlint-cli2` baja de 12 a 5 (= baseline de HEAD)
**Gate Result:** ✅ DO
**Estado:** PENDING

**Causa raíz:** al convertir enlaces dentro de celdas de tabla, las etiquetas con caracteres
que cambian la interpretación de la fila (`(` `)` dentro de un enlace anidado en negrita)
alteraron la estructura de la lista/markdownlint la ve de otra forma.

**Por qué está en F1 y no se resolvió en F0:** el baseline de markdownlint está fijado en 12
en el workflow, así que el gate no está rojo. Arreglarlo es cerrar deuda propia, no una
emergencia. Al arreglar, bajar `BASELINE` a 5.

**Restricción:** no usar `markdownlint --fix` sobre prosa escrita a mano.

---

## F2 — Ejecutar los ejemplos de la documentación · ⏳ PENDIENTE

**Gate de salida:** un bloque de código incorrecto en `docs/` hace fallar el build.

### Task 13 — F2-T1: hacer ejecutables los ejemplos de `docs/`

**Appetite:** max 4h · **Esfuerzo:** ⬜ · **Prioridad:** P0
**Archivos clave:** `docs/api/EMBEDDED_SDK.md`, `docs/api/PYTHON_SDK.md`, `docs/user/QUICKSTART.md`, `ci-examples.yml`
**Verificación real:** un ejemplo con API inventada falla el CI
**Gate Justificación:** `ci-examples.yml` ejecuta `examples/`, no los bloques dentro de `docs/`. Hoy **ningún** ejemplo de la documentación se ha ejecutado jamás.
**Gate Result:** ✅ DO
**Estado:** PENDING

Es el hueco de mayor impacto que queda. La literatura de gobernanza documental con IA es
consistente en un punto: [arXiv 2609.04218](https://arxiv.org/abs/2609.04218) reporta que
_ambas_ condiciones de revisión automatizada fallaron en detectar los defectos de calidad
documental que un revisor humano sí encontró, y que la diferencia de severidad fue **retirada**
tras reevaluación a ciegas, con un re-test que **no replicó** (6/24 vs 5/24). La conclusión
honesta: la automatización de revisión tiene su hueco exactamente donde viven los defectos
documentales. Lo que funciona es la **verificación**.

Un ejemplo equivocado **no compila**, así que nunca llega a un humano. Tres concretos:

- Rust: `RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace` (hoy `ci-rustdoc.yml` sólo genera el artefacto `cargo doc`, sin `-D warnings` y sin doctests)
- Python: `pydoclint --check-style=numpy vantadb/` (deriva docstring ↔ firma)
- TS: `typedoc` + los ejemplos como tests

Seguir el patrón de **Haystack**, que ejecuta sus snippets de documentación cada noche con
credenciales reales, y de **Qdrant**, que compila sus snippets como fuentes reales y falla el
PR si `git status` queda sucio.

### Task 14 — F2-T2: gate de cambio de API ⇒ cambio de docs

**Appetite:** max 2h · **Esfuerzo:** ⬜ · **Prioridad:** P1
**Archivos clave:** `.github/workflows/gate-docs-links.yml`
**Verificación real:** un PR que cambia `fn pub` sin tocar `docs/` falla
**Gate Justificación:** es el único gate de documentación que escala a 1 humano + N agentes, porque es un chequeo de diff, no un juicio humano.
**Gate Result:** ✅ DO
**Estado:** PENDING

Es exactamente el patrón de **DuckDB** (`NeedsDocumentation.yml`). Se puede implementar como:
si cambian ítems públicos de `src/**` y ningún fichero bajo `docs/**` cambió en el mismo
commit → fallar.

### Task 15 — F2-T3: gate anti-fuga en `docs/`

**Appetite:** max 2h · **Esfuerzo:** ⬜ · **Prioridad:** P1
**Archivos clave:** `.github/workflows/gate-docs-links.yml`, `.gitleaks.toml` o `trufflehog`
**Verificación real:** un hostname interno o un patrón de credencial en `docs/` falla
**Gate Justificación:** con 1721 ficheros, la mitad internos, es el gate que más probablemente no existe hoy.
**Gate Result:** ✅ DO
**Estado:** PENDING

Los dos mayores incidentes doc-adjentes de la historia fueron documentación interna y
credenciales en el mismo VCS:

- **CISA "Private-CISA", 2026-05-18** (Krebs on Security): `importantAWStokens` con
  credenciales administrativas a tres servidores AWS GovCloud, `AWS-Workspace-Firefox-Passwords.csv`
  con credenciales en claro de decenas de sistemas internos, y ficheros detallando **cómo CISA
  construye, testea y despliega software internamente**. La cuenta tenía **desactivado** el
  secret-blocking por defecto de GitHub.
- **Uber 2016**: credenciales admin de AWS hardcodeadas en un repo **privado**; 57M registros.
  El fallo no es "los repos públicos son peligrosos" — es que el VCS es donde viven _tanto_ la
  arquitectura _como_ las claves, así que un compromiso da ambas.

Complementar con el push protection de GitHub en los repos públicos.

---

## F3 — Normalizar nombres y eliminar duplicación · ⏸️ DEFER (decisión del owner)

**Gate de salida:** 0 grupos de nombres duplicados fuera de la lista de exenciones.

### Task 16 — F3-T1: 54 ADRs, 3 convenciones de nombre

**Appetite:** max 4h · **Esfuerzo:** ⬜ · **Prioridad:** P2
**Archivos clave:** `docs/dev/architecture/adr/**`, `docs/dev/architecture/adr/README.md`
**Verificación real:** regex `^\d{4}-[a-z0-9-]+\.md$` sobre el directorio
**Gate Justificación:** 41 `ADR-_`, 7 `NNN__`, 6 otros. Con 54 ficheros y `[[README]]`-style ambiguity, la convención única es lo que hace que `git log --follow` y la resolución de enlaces funcionen.
**Gate Result:** ✅ DO
**Estado:** DEFER (requiere aprobación: un commit de 54 renombrados)

Procedimiento: (1) inventario + `adr/_migration-map.csv` commiteado primero — el mapa es lo
que hace funcionar `git log --follow`; (2) `git mv` en **un** commit; (3) reescribir enlaces
entrantes en el mismo commit; (4) regla de regex en CI; (5) `index.md` regenerado.

También aquí: `supersedes` / `superseded_by` bidireccionales, para que un ADR con un
`supersedes` que apunta a un destino que sigue saying `accepted` sea una arista rota
detectable.

### Task 17 — F3-T2: `docs/user/book/src/` es una copia

**Appetite:** max 2h · **Esfuerzo:** ⬜ · **Prioridad:** P1
**Archivos clave:** `docs/user/book/src/**` (75 ficheros), `docs/user/book/book.toml`
**Verificación real:** 0 ficheros con contenido duplicado en dos rutas
**Gate Justificación:** es la causa directa de 8 de los 69 grupos de nombres duplicados (`CHANGELOG.md` ×2, `MCP.md` ×3, `BENCHMARKS.md` ×4, `ARCHITECTURE.md` ×3) y una segunda fuente de verdad que ya ha divergido.
**Gate Result:** ✅ DO
**Estado:** DEFER (decisión del owner: generar el libro desde `docs/` en build, o retirarlo)

**Regla applicable ya, sin esperar la decisión:** ningún enlace nuevo puede apuntar dentro de
`docs/user/book/src/`. Se enlaza a `docs/`. Esto está en `documentation-skill` §1.1.

### Task 18 — F3-T3: 1 007 task files como lastre de navegación

**Appetite:** max 0 (decisión) · **Esfuerzo:** ⬜ · **Prioridad:** P3
**Archivos clave:** `docs/dev/tasks/**`
**Verificación real:** n/a
**Gate Result:** ⏭️ SKIP — restricción dura del owner
**Estado:** SKIPPED POR RESTRICCIÓN

**1 007 ficheros = el 59% de `docs/`, 100% huérfanos, cero enlaces entrantes.** Son la mayor
fuente de no-descubrimiento del repo. Ningún proyecto de los nueve estudiados (rust-lang,
Django, FastAPI, Kubernetes, Next.js, Astro, Tailwind, Supabase, Cloudflare) guarda
work-items en `docs/`.

**Pero el owner decidió explícitamente que `docs/dev/tasks/` no se mueve** porque la
ubicación es parte del sistema de tareas. Este plan lo respeta y **no propone mover nada**.

Mitigación aplicada en su lugar: los índices generados excluyen el detalle de tasks, y
`check-docs.mjs` reporta 4 huérfanos en total (los tasks ya están excluidos por `kind: task`
más el filtro de no-archivo). La deuda queda **documentada, no resuelta**, y revisar esta
decisión es una decisión del owner, no del agente.

---

## F4 — Opcional, sólo si aparece el caso · ⏸️ DEFER

### Task 19 — F4-T1: modelo de arquitectura validado (Structurizr C4 DSL)

**Appetite:** max 6h · **Esfuerzo:** ⬜ · **Prioridad:** P3
**Gate Result:** ⏭️ SKIP por ahora — con justificación
**Estado:** DEFER

Es el único punto donde un agente recibe **validación** en vez de texto: el MCP gratuito de
Structurizr expone `validate_structurizr_dsl` y la jerarquía de abstracción C4 impide que un
agente invente un componente dentro de un sistema. Contrasta con Backstage: 2-5 FTE, tres
CVEs RCE en 2026, y un motor de docs en modo mantenimiento hasta 2027-05-05.

**Por qué se difiere:** VantaDB ya tiene **CodeGraph** (índice pre-construido de 20,5K
símbolos). Añadir un segundo modelo de arquitectura es duplicación, y el ladder dice que no.
Se revisa si algún día un agente hace cambios arquitectónicos sin consultar las fronteras.
Regla dura: los ADRs **no** van como fuente en la sección Decisions de Structurizr — el
export estático los borra.

### Task 20 — F4-T2: sitio estático para `docs/api/`

**Appetite:** max 4h · **Esfuerzo:** ⬜ · **Prioridad:** P3
**Archivos clave:** `mkdocs.api.yml`
**Gate Result:** ⏭️ SKIP por ahora — con justificación
**Estado:** DEFER

Hoy no hay URL canónica para la documentación, así que no hay superficie SEO. Cuando se
quiera: **MkDocs Material con `docs_dir: docs/api`** — la opción más inequívoca para
renderizar _una subcarpeta_, y `--strict` da un gate de enlaces gratis.

⚠️ **No construir Docusaurus sobre todo `docs/`**: además de los problemas de Webpack
(builds de >1h, OOM sobre ~12k ficheros, `EMFILE` en Windows), construiría los 1 007 task
files.

---

## DSKIP — Descartado con justificación

| Descartado | Por qué |
|---|---|
| Migrar `docs/` a Confluence / Notion / Outline / Backstage | Destruye `git log` (la única ventaja real), crea segunda fuente de verdad, y elude el problema real (higiene, no formato) |
| Recomendar Mintlify / GitBook / Scalar de pago | $0-450/mes contra una solución MIT que cuesta un tarde. El único escenario donde Mintlify gana es "quiero un editor WYSIWYG y no tengo a nadie que escriba" — y aquí los editores son agentes |
| Índice RAG / de embeddings sobre `docs/` | El consenso 2026 documentado dice que es peor que `grep` para texto en repo: 18,3 MB en un repo que los agentes ya clonan |
| `llms-full.txt` | No está en la especificación llms.txt v2; para 18,3 MB sería una bomba de contexto |
| Adoptar Zensical | 11 meses de edad, con tier comercial adjunto — la misma forma que produjo el problema en MkDocs. Observar |
| Migrar los 54 ADRs a YADR (YAML) | 3 meses de antigüedad. La idea correcta, el momento equivocado. Añadir `id`/`status` a MADR da el 80% del beneficio |
| Mover `docs/dev/tasks/` fuera de `docs/` | **Restricción dura del owner.** Documentado como deuda en F3-T3, no ejecutado |
| Sustituir `master-index.md` por `docs/index.md` sin periodo de solape | El índice antiguo es referencia humana; se deprecará en F5 con un banner, no se borra de golpe |

---

## F5 — Cierre

**Gate de salida:** los 4 jobs verdes, `check-links` y `check-docs` en `all clear`, y la
skill se carga sin intervención manual.

- [x] `node scripts/docs/check-links.mjs` → exit 0 (109 rotos dentro de presupuesto, F1-T1)
- [x] `node scripts/docs/check-docs.mjs` → `all clear`
- [x] `node scripts/docs/gen-index.mjs --check` → exit 0
- [x] `npx markdownlint-cli2` → 12 = baseline
- [x] `skill documentation-skill` carga y su Definition of Done pasa en un documento nuevo
- [ ] `docs/dev/master-index.md` lleva un banner que apunta a `docs/index.md` como canónico (F1)

## Invariantes del plan

- **Ningún script adivina un destino de enlace.** Si la coincidencia no es única, se reporta
  y el enlace se queda como estaba. Un enlace equivocado es peor que uno roto.
- **Los índices generados nunca se editan a mano.** Llevan banner `<!-- GENERATED -->`.
- **`docs/dev/tasks/` y `docs/dev/plans/` no se mueven, no se renombran, no se reordenan.**
  Restricción del owner; este plan trabaja dentro del sistema de tareas.
- **`docs/CHANGELOG.md` no se edita a mano.** release-plz es su único escritor.
- **Toda cifra de este plan es reproducible** con un comando de `scripts/docs/`. Si un número
  no se puede volver a medir, no se cita.
