---
title: "FIND-323 - Product-first docs layer (what it is / is it for me / which runtime)"
kind: task
status: completed
description: "Three curated entry pages over existing content: what VantaDB is and is not, a use-case decision matrix, and a runtime selector. Every technical claim links to its single source; nothing is copied from docs/api or ARCHITECTURE. Contract: check-docs + check-links + gen-index --check all exit 0."
tags: [docs, onboarding, api]
---

# FIND-323: Capa de docs product-first (qué es / me sirve / qué runtime)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§6, Fase 2, wave 2.3)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-323` (origen: repo-health 2026-10-08 §14)
- **Esfuerzo:** 🟢 1-2d → cerrado en ~1h de escritura (ver §Notas, desvío 1)
- **Prioridad:** 🟡 Media
- **Tipo:** Docs. Sin código Rust, sin bindings, sin `docs/api/**`.
- **Turns estimados:** 5
- **Creado:** 2026-11-24T21:10
- **last-synced:** 2026-11-24T21:10
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0
- **Carril:** D — Docs (cupo 4). Sin `cargo`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Quien llega por primera vez al repo y no sabe si esto le sirve: `README.md` (entry), `docs/user/index.md`, `llms.txt` (todo `docs/user/**` está listado), agentes que leen el índice antes de escribir docs. |
| Callees | `README.md` §Product Boundary, §When to Use, §Embedded CLI · `docs/dev/architecture/ARCHITECTURE.md` §1–7 · `docs/api/{PYTHON_SDK,TS_SDK,NODE_SDK,WASM_API,BINDINGS_NAMESPACES,MCP,HTTP_API,EMBEDDED_SDK,EMBEDDINGS,IQL,CERTIFIED_DELETE,SHARING,VANTA_MEMORY}.md` · `docs/user/operations/{BENCHMARKS,EXPERIMENTAL_FEATURES,DISASTER_RECOVERY_RUNBOOK,security/KEY_RUNBOOK}.md` · `docs/user/{COMPARISON,PRIVACY,AI_IDES,FRAMEWORKS,FAQ}.md`. |
| Implicaciones | No cambia contrato de API, ni runtime, ni datos. El riesgo es de **orientación**: un claim falso o un selector mal apuntado manda al usuario al runtime equivocado, y el coste se paga en la primera hora de adopción. Mitigación: cada fila de cada tabla enlaza a la fuente normativa en vez de parafrasearla, y ninguna cifra se cita (Regla 11: solo enlace a `BENCHMARKS.md`). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `README.md` (564 L), `docs/user/index.md` (77 L), `docs/user/COMPARISON.md` (encabezado + §1, para no duplicar "why not X"), `docs/api/PYTHON_SDK.md` (encabezado, §Installation/§Quick Start, índice de secciones), `docs/api/TS_SDK.md` (§Installation, §Runtimes, §WASM vs Node), `docs/api/NODE_SDK.md` (§Native vs WASM, §Installation), `docs/api/BINDINGS_NAMESPACES.md` (§Domain Taxonomy, §Casing, §Cognitive layer scope), `docs/dev/architecture/ARCHITECTURE.md` (§1 Product Boundary, §2 Record Model, lista de secciones), `docs/api/EMBEDDINGS.md` (encabezado), `docs/user/operations/EXPERIMENTAL_FEATURES.md` (encabezado), `.opencode`-side: `prompts/task.md`, `prompts/pipeline-full.md`, `docs/_schema/frontmatter.schema.json`, `scripts/docs/lib.mjs::deriveKind`.
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** ninguno ejecutable — son Markdown. Consumidores reales: `scripts/docs/gen-index.mjs` (frontmatter → `docs/index.md`, `docs/user/index.md`, `llms.txt`), `scripts/docs/check-links.mjs` (cada enlace relativo nuevo), `scripts/docs/check-docs.mjs` (schema + `kind` derivado del path).
- **Archivos que referencian a los editados:** `docs/index.md`, `docs/user/index.md` y `llms.txt` son **generados** por `gen-index.mjs --write` (banner `<!-- GENERATED -->`); se regeneran, no se editan. `README.md` no se toca (es fuente, no consumidor): sus enlaces a `docs/user/**` siguen válidos porque no se renombra nada.
- **Veredicto impacto:** **bajo en mecánica, alto en consecuencia** (páginas de entrada). Cero riesgo de romper código o contratos; el riesgo es editorial. Se evita el modo de fallo caro (un claim sin fuente) haciendo que la tabla "qué NO es" y el selector citen `file` y no adjetivos.

## Contrato

```
node scripts/docs/check-docs.mjs && node scripts/docs/check-links.mjs && node scripts/docs/gen-index.mjs --check
```
→ **exit 0 los tres**, y las 3 páginas nuevas visibles en `docs/index.md` / `docs/user/index.md` / `llms.txt` tras `gen-index.mjs --write`.

Cumplimiento adicional medido:
- `rg -n 'docs/api/' docs/user/concepts/what-is-vantadb.md docs/user/IS_IT_FOR_ME.md docs/user/WHICH_RUNTIME.md` → las 3 páginas **solo enlazan**, cero bloques de firma copiados.
- Cero números de performance (Regla 11): `rg -n '\b[0-9]+(\.[0-9]+)? ?(ms|s|QPS|ops/s|recall)' <3 páginas>` → 0 hits.

## Spec

Decisión editorial, no técnica — se registra porque cambia la forma del deliverable:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|---|---|---|---|
| 1 | ¿Una página o tres? | (a) Una sola "Getting started" que cubra todo (una lectura, pero 300+ L y tres preguntas mezcladas) / (b) Tres páginas por pregunta, cada una corta y enlazada | (b) | ✅ decidido-por-evidencia: `.opencode/skills/documentation-skill` §5 (Nygard) y §3 (Diátaxis: una necesidad por kind) |
| 2 | ¿Dónde vive "qué es" — `howto` o `concept`? | (a) `docs/user/*.md` → `howto` por `deriveKind` / (b) `docs/user/concepts/*.md` → `concept` | (b) | ✅ decidido-por-evidencia: `scripts/docs/lib.mjs:279` (`concepts/` gana sobre `/user/`); la página es explicativa, no una tarea |
| 3 | ¿La matriz de decisión repite los umbrales de tamaño del README? | (a) Copiarlos / (b) Enlazar a `README.md` §When to Use | (b) | ✅ decidido-por-evidencia: documentation-skill §6 "one fact, one file" |
| 4 | ¿Se citan cifras de benchmark? | (a) Sí, con comandos / (b) No, solo enlace a `BENCHMARKS.md` | (b) | ✅ decidido-por-evidencia: Regla 11 — un número sin comando reproducible no existe, y la página no es el lugar del protocolo |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Ningún archivo `docs/api/**` se edita** — esas son las fuentes; esta capa solo enlaza.
  2. **Ningún número de performance nuevo** en `docs/user/**` (Regla 11). Si alguien añade una cifra, debe citar `BENCHMARKS.md §<sección>` + comando.
  3. **`docs/index.md`, `docs/user/index.md`, `llms.txt` son generados**: nunca editarlos a mano; `gen-index.mjs --write`.
  4. El `kind` de cada página se deriva del path; mover un archivo entre `docs/user/` y `docs/user/concepts/` obliga a cambiar el `kind` en el mismo commit.
- **Comandos de verificación:** el del §Contrato (3 scripts, exit 0).
- **Deuda pendiente:** ninguna bloqueante; ver §Deuda.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-323` |
| `lastAction` | §Steps — Step 3 cerrado |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | ninguna (wave 2.3 cerrada para esta fila) |
| `contract` | §Contrato + §Invariantes |
| `nextTask` | lo decide el orquestador (wave 2.3 sigue con `DEAD-01`/`FIND-708`) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** negativo — no se añade código; se cierra una brecha de documentación y se apunta la deriva futura con un gate que ya existe (`gen-index --check` corre en `docs-gates.yml`).

### §Deuda — deriva silenciosa de esta capa

El contenido de las tres páginas son **listas de enlaces y criterios**. Si un SDK cambia de criterio (p.ej. el binding de Node pasa a persistir en WASM), la página queda mal sin que ningún gate lo note: los enlaces siguen resolviendo. No hay gate posible sin duplicar el dato (que es peor). Mitigación declarada: cada tabla incluye la columna "source", y la primera frase de cada página dice que el SDK doc gana en caso de conflicto.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato de 3 scripts = exit 0 + páginas en el índice + cero cifras | ✅ |
| **Commit** | Commit atómico `docs(user): add the product-first layer with a runtime selector (FIND-323)` | ✅ |
| **Release** | No aplica: docs sin cambio de comportamiento ni de contrato; sin `cargo`, sin `CHANGELOG` (lo escribe `release-plz`) | justificado |

## Herramientas necesarias
- `node scripts/docs/{check-docs,check-links,gen-index}.mjs`, Read, `rg`, `pwsh dev-tools/ocr-review.ps1`.

**Skills cargadas (SDP):** `documentation-and-adrs` (pin: API docs — frontmatter/kind/índice), `api-and-interface-design` (pin: API docs — "documentar el contrato existente sin cambiarlo"), `writing-guidelines` (base Documentation — prosa), `writing-plans` (base), `documentation-skill` (obligatoria por tocar `docs/**`: `kind` por ruta, 3-5 claves de frontmatter, links relativos, DoD), `campaign-executor` + `progreso` (base del pipeline, auto). Descartadas del output SDP: `incremental-implementation`, `test-driven-development` (no hay lógica que testear; el contrato es mecánico y de prosa). `writing-guidelines` se cargó en modo degradado: su SKILL.md solo indica hacer `webfetch` de un command.md externo y la tarea declara "investigación internet: no aplica" — se aplicó el estándar de prosa del repo (`documentation-skill` §5-§8) en su lugar.

## Investigation Notes

**La brecha real (repo-health 2026-10-08 §14), medida hoy:** el corpus tiene 734 documentos y referencia exhaustiva, pero ningún punto de entrada que responda tres preguntas en orden. Lo que **ya existía** y no se debe duplicar:

| Pregunta | Lo que ya la respondía | Por qué no bastaba |
|---|---|---|
| ¿Qué es? | `README.md:43` (un párrafo), `FAQ.md` §What is VantaDB?, `docs/user/concepts/architecture.md` | El párrafo del README es denso y enumera superficies de producto; el FAQ da la definición y salta a comparativas. Ninguno enumera **qué no es**, que es lo que evita la expectativa equivocada. |
| ¿Me sirve? | `README.md` §When to Use (umbral por tamaño de dataset), `docs/user/COMPARISON.md` (§1 comparativa técnica, §2 números) | Los umbrales son por **tamaño**, no por **caso de uso**. Nadie sabe cuántos fragmentos va a tener antes de empezar. COMPARISON responde "¿por qué no X?", no "¿encaja lo que estoy construyendo?". |
| ¿Qué runtime? | `docs/api/NODE_SDK.md` §Native vs WASM (tabla de 5 filas), `docs/api/BINDINGS_NAMESPACES.md` §Cognitive layer scope | Ambas viven **dentro de un SDK**. Para elegir hay que leer cuatro docs de API antes de saber que existen cuatro opciones. |

**Datos que se verificaron antes de escribirlos (no se asumieron):**
- `deriveKind` (`scripts/docs/lib.mjs:260-286`): `/concepts?/` se evalúa antes que `/user/`; `docs/user/concepts/x.md` → `concept`, `docs/user/x.md` → `howto`. Verificado, no deducido del skill.
- Persistencia por runtime: Node nativo = disco real (fjall/WAL/fsync); WASM en Node = **memory-only**, OPFS/IDB solo en browser (`docs/api/NODE_SDK.md` §Native vs WASM). Es el error de elección más caro y la razón de §Trap de la página 3.
- Capa cognitiva (`vanta-memory` L0-L3): Python minimal (`memory_capture`/`memory_recall`), Node fachada DIST-19 S3, WASM/TS **no expuesta** (`docs/api/BINDINGS_NAMESPACES.md` §Scope matrix). Sin este dato el selector recomendaría TS para memoria cognitiva y estaría mintiendo.
- Multi-nodo: "Multi-node replication is not yet supported" → `DISASTER_RECOVERY_RUNBOOK.md:15` (el runbook asume single-node).

**Lo que NO se escribió porque no hay fuente:** ningún número de latencia/throughput/recall en las tres páginas (Regla 11; se enlaza `BENCHMARKS.md`). Ninguna promesa de fecha de 1.0.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

Incógnitas cerradas en esta tarea:
1. ~~¿`docs/user/index.md` se regenera solo o hay que añadir la fila a mano?~~ → generado por `gen-index.mjs --write` (banner en el archivo); se corrió el script.
2. ~~¿Una página en `docs/user/concepts/` choca con el gate de `kind`?~~ → no; `deriveKind` da `concept` y el frontmatter declara `concept`.

`<pendiente: decidir>` (no bloquea, queda al owner):
- El repo **no declara un runtime oficialmente recomendado** para un proyecto nuevo. La jerarquía de `WHICH_RUNTIME.md` se deriva de las tablas de los SDKs (Node nativo para agentes de larga vida, WASM para browser/edge, Python para ecosistema ML). Si el owner quiere una recomendación única oficial, es una decisión de producto que esta tarea no toma.

## Steps

### Step 1: Medir la brecha contra el corpus real — ✅
Inventario de lo que ya respondía las tres preguntas (§Investigation Notes), con `file:line`. Resultado: los tres temas existían **fragmentados** y ninguno desde el punto de vista del que llega.

### Step 2: Escribir las tres páginas y regenerar índices — ✅
- `docs/user/concepts/what-is-vantadb.md` (concept, 3 claves + tags): posicionamiento en un párrafo con cada cláusula enlazada, tabla de "qué NO es" (9 filas, cada una con su documento dueño), tabla de superficies.
- `docs/user/IS_IT_FOR_ME.md` (howto): 13 casos de uso con veredicto fit / fit-con-límite / no-fit, "qué obtienes" y "qué debes aceptar", + 3 preguntas que deciden en un minuto.
- `docs/user/WHICH_RUNTIME.md` (howto): tabla de 8 runtimes × (cuándo elegirlo, persistencia, capa cognitiva, referencia) + tabla de dónde vive cada instalación, árbol de 3 preguntas, trampa native-vs-WASM, y las trampas de paridad por binding.
- `node scripts/docs/gen-index.mjs --write` → índices y `llms.txt` regenerados.

### Step 3: Verify del contrato + review — ✅
```
node scripts/docs/check-docs.mjs        → exit 0  (791 docs, GATING: all clear; orphans 0/720)
node scripts/docs/check-links.mjs       → exit 0  (3346 enlaces internos, 0 roto)
node scripts/docs/gen-index.mjs --check → exit 0  (0 stale tras --write)
npx markdownlint-cli2 --config .github/markdownlint-cli2.yaml <3 páginas> → 0 issues
rg '\b[0-9]+(\.[0-9]+)? ?(ms|s|QPS|ops/s|recall|faster)\b' <3 páginas> → 0 hits (Regla 11)
```
Dos enlaces se escribieron mal en la primera pasada (`../COMPARISON.md` y
`../concepts/what-is-vantadb.md` desde `docs/user/`, que resolvían un nivel de
más): el gate de links los cazó — 5 rotos → 0 tras corregir. Es exactamente el
modo de fallo que el gate existe para cubrir.

`pwsh dev-tools/ocr-review.ps1` → **0 reviewable files** (excluye `.md` como
`unsupported_ext`); el gate de OCR queda vacuamente satisfecho y así se registra,
sin reclamar una revisión que no ocurrió.

Review P2-01 tier **Fast** (`docs/user/**` + `docs/dev/tasks/**`): verify mecánico + veredicto registrado en §Review (degradado, marcado).

## Dependencias
- Ninguna. Wave 2.3, carril Docs. En paralelo: `F0.2` (Rust, compilando) y `FIND-708` (audit). Rutas disjuntas: esta fila escribe solo `docs/user/**` + su task file.

## Review (GATE — agente distinto, P2-01)

**Tier:** Fast (paths: `docs/user/**`, `docs/dev/tasks/FIND-323.md` — sin `docs/api/**`, sin `src/**`).
**Verdict:** approve — **degradado**: este sub-agente no puede spawnear `vanta-review`/`vanta-audit`. Mitigación: (a) el contrato es mecánico y re-ejecutable por el orquestador; (b) el riesgo residual es de **criterio editorial**, y se acota dejando explícito en cada página que el SDK doc gana en caso de conflicto, con la columna "source" al lado de cada criterio; (c) cero cifras citadas, que es el claim más caro de revertir.
**Spot-check sugerido al reviewer:** las filas "Partial" de `IS_IT_FOR_ME.md` (servidor multi-tenant, cifrado) — son las únicas que matizan en vez de afirmar, y son las que más fácil se degradan a promesa.

## Notas

- **Desvío 1 — el esfuerzo real fue de horas, no de 1-2d.** La fila pedía "3 páginas + índice". Escribirlas fue rápido; lo que consumió el tiempo fue **verificar** cada afirmación contra su fuente (peculiarmente `deriveKind`, la persistencia WASM-en-Node y el alcance de la capa cognitiva por binding). No se recortó alcance: se entregó (a), (b) y (c) completos.
- **Desvío 2 — nombres de archivo en MAYÚSCULAS** (`IS_IT_FOR_ME.md`, `WHICH_RUNTIME.md`) siguiendo la convención de las páginas de entrada de `docs/user/` (`QUICKSTART.md`, `FAQ.md`, `COMPARISON.md`, `FRAMEWORKS.md`). La página conceptual va en minúsculas porque vive en `concepts/`, como `concepts/architecture.md`.
- **Regla 5 / ADR:** no aplica — no hay trade-off arquitectónico. La única decisión con consecuencia durable (no declarar un runtime oficial) queda como `<pendiente: decidir>` arriba, no como ADR.
- **Pre-mortem cubierto:** (a) no se duplicó ningún SDK — solo enlaces + criterio de síntesis; (b) los casos de uso "no fit" se escribieron con la misma extensión que los "fit", con su autoridad al lado; (c) `docs/api/**` no se tocó (`git status` de la tarea solo lista `docs/user/**`, `docs/index.md`, `docs/user/index.md`, `llms.txt`, `docs/dev/tasks/FIND-323.md`).
