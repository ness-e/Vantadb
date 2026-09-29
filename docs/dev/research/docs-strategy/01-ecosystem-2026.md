---
title: Estrategia de Documentación — Investigación Exhaustiva del Ecosistema 2026
kind: research
status: active
description: El modelo que ya tienes es el correcto. No lo migres
tags: [vantadb, docs, research, "2026"]
date: "2026-09-28"
scope: [documentacion, estrategia, tooling, ADRs, llms-txt, docs-as-code]
method: "5 líneas de investigación web paralela + auditoría local del repo"
---

# Estrategia de Documentación — Investigación del Ecosistema 2026

> **Fecha de verificación: 2026-09-28.** Todos los precios, fechas de release, estrellas y
> códigos HTTP de este informe fueron comprobados ese día. Lo que no se pudo verificar está
> marcado explícitamente. No se ha usado conocimiento previo ("de memoria") para ninguna
> afirmación sobre el estado de una herramienta.

---

## 0. Veredicto en una página

**El modelo que ya tienes es el correcto. No lo migres.**

Los cinco informes convergen en lo mismo, y es la conclusión más importante de este documento:

| Alternativa propuesta en el brief | Veredicto 2026 | Evidencia |
|---|---|---|
| **Mintlify** como portal externo | ❌ **No.** $450/mes Pro, y **self-host es solo Enterprise**. 6 de 7 sitios del benchmark que usan Mintlify no tienen ningún CI de docs. | [mintlify.com/pricing](https://www.mintlify.com/pricing) |
| **Swimm** para documentación interna | ❌ **No.** Siguen vivos pero sin ronda de financiación declarada desde nov-2021 ($27.6M Serie A, total $33.3M). Y es *code-coupled*, ortogonal a ADRs. | [swimm.io](https://swimm.io/), [Insight Partners](https://www.insightpartners.com/ideas/swimm-raises-27-6-million-to-repair-developers-love-hate-relationship-with-documentation/) |
| **Docusaurus** (docs-as-code React) | ⚠️ **Solo para `docs/api/`.** Webpack: builds de >1h reportados, OOM sobre ~12k archivos, `EMFILE` en Windows (tu plataforma). 4/25 pares lo usan. | [discussion #10895](https://github.com/facebook/docusaurus/discussions/10895), [issue #4765](https://github.com/facebook/docusaurus/issues/4765) (abierto desde 2021) |
| **Starlight** (Astro) | ⚠️ Rápido, pero **no tiene versionado nativo** y VantaDB es pre-1.0 multiversión. | [HN 44074626](https://news.ycombinator.com/item?id=44074626) |
| **Backstage** (IDP interno) | ❌ **No.** 28% de los adoptantes están en "maintenance mode" (encuesta DX, n=180). TechDocs está atado a MkDocs (EOL 2027-05-05) y arrastra **tres CVEs RCE en 2026**. | [DX survey](https://newsletter.getdx.com/p/backstage-and-the-developer-portal-market), [CVE-2026-25153](https://www.sentinelone.com/vulnerability-database/cve-2026-25153/) |
| **Confluence / Outline / Notion** | ❌ **No.** Segunda fuente de verdad, coste por asiento, y Confluence Data Center tiene EOL 2029-03-28. | [atlassian.com EOL](https://www.atlassian.com/licensing/data-center-end-of-life) |
| **MkDocs Material** | ❌ **No como dependencia nueva.** Modo mantenimiento, **EOL 2027-05-05**. Chromium: propiedadFilters→ `CVE-2026-25153`. | [squidfunk#8523](https://github.com/squidfunk/mkdocs-material/issues/8523) |
| **GitBook** | ⚠️ **Gratis y sale a cuenta** (incluye llms.txt + MCP server en plan Free), pero es lock-in total (sin self-host) y el CLI npm `gitbook` es de 2018. | [gitbook.com/pricing](https://www.gitbook.com/pricing) |
| **DeepWiki** | ⚠️ **Como lector de dependencias, sí. Como fuente de verdad propia, no.** Sin export, índice propietario, no indexa repos privados. | [cognition.com](https://cognition.com/blog/deepwiki-mcp-server) |

**Lo que sí es estándar de facto en 2026: `llms.txt` + docs legibles por agente.** En el benchmark de 32
dominios de infra/IA: **18/18** proyectos comerciales de vector/graph/AI-infra lo sirven; **0/9** de
OSS académico/legacy. Ya lo tienes en la raíz — pero está incompleto (§5.1).

**El hueco de mercado real y medible:** en 25+ proyectos de bases de datos e infraestructura
revisados, **0 de 25 mantienen ADRs en git**. Ese es el espacio limpio de VantaDB. Y tú ya
tienes 54.

---

## 1. Lo que ya tienes (auditoría local, 2026-09-28)

Antes de recomendar nada, esto es lo que existe en el repo:

| Capa | Estado verificado | Nota |
|---|---|---|
| **Vault** | 1 715 `.md`, 18,3 MB en `docs/` | GitHub-compatible + Obsidian |
| **CI de docs** | `.github/workflows/gate-docs.yml` | 4 jobs: markdownlint-cli2, frontmatter `title:`, paridad OpenAPI/router + versiones, `validate-frontier.ps1` |
| **CODEOWNERS** | `/docs/ @ness-e` | ✅ Ya existe |
| **`llms.txt`** | Raíz, 110 líneas | ⚠️ Versionado 0.5.0 (el repo ya tiene 0.6.0 en `target/`), 4 URLs, apunta a `github.com/blob` (HTML, no raw) |
| **API ref desde código** | `ci-rustdoc.yml` → artefacto `cargo doc` | ⚠️ No es `cargo test --doc`, no es `-D warnings` |
| **Ejemplos ejecutables** | `ci-examples.yml` → `cargo run --example *`, `python examples/**` | ✅ Pero ejecuta `examples/`, **no los bloques de código dentro de `docs/`** |
| **Changelog** | `docs/CHANGELOG.md`, 175 KB, release-plz | ✅ Consistente con los pares (LlamaIndex: 686 KB) |
| **ADRs** | 54 archivos en `docs/dev/architecture/adr/` | ⚠️ **3 convenciones de nombre simultáneas**: 41 `ADR-*`, 7 `NNN_*`, 6 otros |
| **Task files** | 912 en la raíz de `docs/dev/tasks/` + 95 en subcarpetas | ⚠️ El 57% de todo `docs/` |
| **Índice** | `docs/dev/master-index.md`, 282 líneas, regla manual "índexar en el mismo PR" | ⚠️ Depende de disciplina humana |
| **Sitio público** | ❌ **No existe.** Sin mkdocs/docusaurus/astro/hugo | ⚠️ No hay URL canónica ni superficie SEO |
| **Link checker** | ❌ No hay lychee, ni vale, ni cspell, ni typos | — |
| **Gate anti-fuga** | ❌ No hay escaneo de secretos/hostnames en `docs/` | — |

**Conclusión de la auditoría: ya tienes el 70% de la respuesta correcta.** El trabajo real
no es elegir herramientas: es cerrar 6 huecos mecánicos. Todos están en §5.

---

## 2. Mercado externo verificado (2026-09-28)

### 2.1 Generadores docs-as-code

| Herramienta | Licencia | Mantenimiento verificado | llms.txt nativo | Escala a 1 715 archivos | Veredicto |
|---|---|---|---|---|---|
| **Hugo** + Docsy | Apache-2.0 | 0.167.0, push 2026-09-28 | Manual (plantilla) | ✅ El único que documenta escala grande | Si necesitas un 2º sitio, este |
| **Starlight** (Astro) | MIT | 0.42.4, push 2026-09-28 | Plugin (`starlight-llms-txt` 0.12.0) | ✅ Rápido | ⚠️ **Sin versionado nativo** |
| **VitePress** | MIT | 1.6.4 (`latest`; v2 sigue alpha) | Plugin | ✅ | ⚠️ v2 alpha; nunca hará wikilinks |
| **Docusaurus** | MIT | 3.10.2, push 2026-09-25 | 3 forks de plugin, ninguno first-party | ⚠️ OOM >12k, >1h builds, `EMFILE` en Windows | Solo para `docs/api/` |
| **fumadocs** (Next.js) | MIT | 13,2k★, push 2026-09-27 | Sí, integrado | ✅ | El mejor playground OpenAPI, pero **tú construyes el versionado** |
| **MkDocs Material** | MIT | **MODO MANTENIMIENTO, EOL 2027-05-05** | `mkdocs-llmstxt` 0.5.0 | ✅ (`docs_dir:` por subcarpeta, sin ambigüedad) | Congelado, no como dependencia |
| **Zensical** (sucesor Rust de MkDocs Material) | MIT | 5,8k★, push 2026-09-28 | Manual | ✅ | ⚠️ 11 meses, con tier comercial en nov-2026. **Observar, no comprometer** |
| **mdBook** | MIT | 0.5.4 | Manual | ✅ | El único con `mdbook test` nativo |
| **Nextra 4** | MIT | 4.6.1, último release 2025-12 | Manual | ✅ | Nextra 4 rompió Pages Router |
| **Quartz** | MIT | 5.0.0 | Manual | ✅ | ⚠️ **Renombra/slugifica archivos** → rompe el árbol compatible con GitHub ([#1859](https://github.com/jackyzha0/quartz/issues/1859), [#2432](https://github.com/jackyzha0/quartz/issues/2432)) |
| **Antora** | MPL-2.0 | Vive en **GitLab**, no GitHub | Manual | ✅ | ❌ Exige contenido en repos separados |
| **docsify** | MIT | 5.0.0 | Manual | ❌ 1 715 XHR en runtime, **cero validación en build** | Descartado |
| **Slate** | — | **`archived: true`** | — | — | Muerto |
| **Docute** | MIT | Último push **2023-09-25** | — | — | Muerto |
| **Dendron** | Apache-2.0 | Empresa **cerrada** | — | — | Muerto |
| **GitBook CLI** | — | npm `gitbook` = **2.6.9 de 2018** | — | — | El producto es SaaS; el CLI murió |

### 2.2 SaaS de portal (precios verificados en la página de precios)

| Producto | Gratis | Pagado | Self-host | MCP | Nota crítica |
|---|---|---|---|---|---|
| **GitBook** | $0 — **incluye `llms.txt`, `llms-full.txt`, `.md` y MCP server automático** | $65 / $249 por sitio + **$12/usuario** | **Nunca** | ✅ todos los planes | Mejor gratis del mercado. Doble facturación: 6 personas = $321/mes |
| **Mintlify** | $0 — MCP server incluido, dominio propio | **$450/mes** Pro (10 000 créditos; 25 créditos/respuesta → ~400 preguntas/mes) | **Solo Enterprise** | ✅ | ~$66,7Mlevantados. La variabilidad de coste por *lector* es el problema |
| **ReadMe** | $0 — 1 proyecto, 1 versión, LLMs.txt + MCP | $250/mes + $150 Ask AI | No | ✅ | Lo útil es Pro-only |
| **Scalar** | $0 — 3 APIs, SDKs ≤25 endpoints | $150 / $600 + SDKs extra | ✅ Core MIT | Parcial | 16,2k★. Core MIT = el único lock-in bajo |
| **Fern** | $0 — 10 miembros, 1 sitio, 1 000 páginas | **Solo Enterprise** (retiró Hobby y Team) | Sí (ofrecido) | No | El hueco de precio se ensanchó |
| **Bump.sh** | No hay free real | $50 / $120 | No | Sí | Cobra las MCP tools como unidad (5 en Basic, 50 en Pro) |
| **Speakeasy** | Tier free | ~$250/mes (cifras de terceros) | Solo generador OSS | ✅ | AGPL desde ~2025 |
| **Stoplight** → **SmartBear** | 1 usuario | $44 / $113 / $362 | Solo Elements | No | Rebranding = señal de dirección |
| **DeepWiki** | Repos públicos | Incluido en Devin Enterprise | No | ✅ `mcp.deepwiki.com` | ⚠️ **Sin export, no indexa repos privados** |

### 2.3 La evidencia honesta sobre `llms.txt`

Hay que separar tres cosas que se confunden constantemente:

| Métrica | Cifra | Fuente | Qué significa |
|---|---|---|---|
| Adopción (top-10k del web) | **5,6%** (jun-2026), desde 1,04% (jul-2025) | HTTP Archive BigQuery (Casey Burridge) | Crece, pero es una especificación **informal sin cuerpo de estándares** |
| Adopción (documentación de infra/IA) | **18/18** | Benchmark propio §4 | Ya es **caso normal entre pares**, no diferenciación |
| **Tráfico que genera** | **97% de los 137 210 ficheros válidos con cero peticiones** en mayo-2026 | Ahrefs, vía [angeo.dev](https://angeo.dev/llms-txt-v2-spec-and-evidence/) | ⚠️ **No cambia el ranking** |
| Correlación con citas de IA | **Ninguna.** Adopción plana por banda de tráfico; quitar la variable *mejoró* la precisión del modelo XGBoost | SE Ranking, 300k dominios, 2025-11-07 (patrocinado por vendedor) | No loystic |
| Google | Lighthouse 13.3 audita `llms.txt` pero lo marca **N/A si falta**; John Mueller: "no ayuda ni perjudica a Search" | [Chrome Lighthouse](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt), Search Off the Record | El auditor lo marca, el buscador lo ignora |
| Uso en inferencia | **Ningún proveedor lo ha confirmado públicamente** | — | ⚠️ La evidencia fuerte no existe |

**Correcciones a dos mitos que circulan:**
- `llms-full.txt` **no está en la especificación** — es una convención de Mintlify (nov-2024).
- **No existe borrador W3C.** `w3c/strategy#506` (abierto 2025-04-27) sigue abierto y
  explícitamente "no introduce ningún formato nuevo".

**Veredicto:** publícalo porque cuesta ~0 y porque **los propios labs lo publican**
(`platform.claude.com/llms.txt` verificado: 639 páginas EN + 261 × 10 locales). Pero no
financies ninguna decisión con ello, y no persigas rankings con él.

**Lo que v2 (2026-08-10) sí añadió y es gratis:**
- `llms.txt` puede vivir en **cualquier path** (`/docs/llms.txt` cubre `/docs/`) — relevante
  porque VantaDB puede tener la doc bajo subpath.
- Markdown limpio en la **misma URL** vía `.md` (`page.md` o `page.html.md`).
- Descubrimiento por `rel="alternate" type="text/markdown"` y `rel="describedby"`,
  expresable como cabecera HTTP `Link:` — **sin tocar las páginas**.
- Se eliminó la semántica mecánica de `## Optional`.

---

## 3. Mercado interno verificado (2026-09-28)

### 3.1 Comparativa

| Herramienta | Licencia | Coste | MCP para agentes | Portabilidad | Mantenimiento 2026 | Veredicto n=1-3 |
|---|---|---|---|---|---|---|
| **Markdown en git + `AGENTS.md`** | MIT | $0 | ✅ GitHub MCP oficial (56 tools) | Perfecta | — | ✅ **Ya lo tienes. Es el estándar de facto** |
| **Obsidian** | Gratis para trabajo desde **2025-02-20** | $0 app | ❌ sin MCP first-party (community) | Perfecta (ficheros planos) | Activo; 10 000+ orgs | ✅ Editor humano; **git es el sistema de registro** |
| **Backstage** + TechDocs | Apache-2.0 | $0 licencia, **2-5 FTE** | `mcp-actions-backend` | Catálogo sí, plugins no | 34,5k★ | ❌ 28% en "maintenance mode"; **3 CVEs RCE 2026**; TechDocs atado a MkDocs (EOL 2027) |
| **Docmost** | AGPL-3.0 + **Business/Enterprise** | Self-host gratis, **MCP de pago** | ✅ 20+ tools, OAuth read/write | Import/export | 21k+★ | ⚠️ El mejor MCP self-host, pero **pagas para que agentes lean ficheros que ya tienes** |
| **BookStack** | MIT | $0 | ❌ sin MCP | Export MD/HTML | Activo | ⚠️ El más barato y aburrido; falta MCP |
| **Outline** | **BSL 1.1** (no OSI) | Self-host gratis | API + community MCP | MD in/out ✅ | Muy activo, ~40k★ | ⚠️ BSL = *lock-in smell* para ti |
| **Wiki.js** | AGPL-3.0 | $0 | ❌ sin MCP | MD/HTML | Activo | ⚠️ Más pesado que BookStack, mismo hueco |
| **Confluence + Rovo** | Propietario | $5,42–$10,44/usuario/mes | ✅ Rovo MCP oficial (GA 2026-02-04) | Export HTML/MD (**con pérdida**) | **Data Center EOL 2029-03-28** | ❌ DB muerta en 3 años, y es *la* plataforma con rottenness documentada por Atlassian |
| **Notion** | Propietario, sin self-host | $10–$20/usuario | ⚠️ el `notion-mcp-server` OSS **está archivado**; solo remoto | Buenas (snapshot con pérdida) | Activo | ❌ API en la nube por lectura |
| **Swimm** | Propietario | Contact sales | `/ask Swimm` | MD en repo | ⚠️ Vivo, **sin financiación declarada desde nov-2021** | ❌ Code-coupled = ortogonal a ADRs |
| **CodeSee Maps** | — | — | — | — | ☠️ **MUERTO** (adquirido por GitKraken 2024-05-14) | ⚠️ Patrón de cautela: no hagas load-bearing del artefacto de un vendor |
| **Quartz** | MIT | $0 | — | — | Activo | ⚠️ Renderizador *read-only* del vault, si quieres vista humana |
| **DeepWiki** | Propietario | Free público | ✅ | ❌ **sin export** | Cognition | ✅ Para leer *dependencias*; ❌ para tu propio repo privado |
| **Structurizr + C4 DSL** | MIT | $0 (+ servidor opcional) | ✅ **9 tools incl. `validate_structurizr_dsl`** | DSL+JSON en git ✅ | v2026.09.19 | ✅ **El único donde el agente recibe *validación*, no texto** |
| **YADR** (MADR→YAML) | OSS | $0 | ✅ parseable por máquina | YAML | **Nuevo, jun-2026** | ⚠️ La idea correcta, 3 meses de edad |
| **adr-tools** | MIT (bash) | $0 | ❌ | Ficheros | Antiguo | ❌ Genera `0001-title.md` — **exactamente tu problema** |
| **log4brains** | MIT | $0 | ❌ | Ficheros | Activo | ⚠️ ADRs + C4 en un sitio; dependencia Node |
| **adr-log** | MIT | $0 | — | — | ☠️ **Listado como "no mantenido" por la propia comunidad ADR** | ❌ |

### 3.2 El caso contra y a favor de "un solo vault en Obsidian leído por agentes"

**A favor (gana, y por mucho):**
1. **Neutralidad de formato.** 1 715 ficheros de texto plano: `grep`, `git blame`, `git log`
   con cronología de decisiones gratis, diffs, backup del SO. El dato sobrevive a **todas**
   las herramientas de §3.1. Compara: Confluence (read-only en 2029), Notion (export con
   pérdida), Outline (BSL), Docmost (AGPL + MCP de pago).
2. **La industria se estandarizó *hacia* tu modelo, no en contra.** OpenAI liberó
   `AGENTS.md` en agosto-2025; se donó a la **Agentic AI Foundation bajo la Linux Foundation**
   en diciembre-2025 con Anthropic y Block. GitHub publicó MCP oficial con 56 tools. Ambos
   asumen que el agente lee **ficheros en un repo**.
3. **El argumento narrativo más creíble de 2026** viene de un ingeniero que gastó meses
   construyendo un IDP Backstage, lo tiró y lo reconstruyó en días: *"ADRs siempre importarán.
   El contexto de lógica de negocio siempre importará. Pero explicar cómo funciona el código…
   de eso se encarga el código."* — [flos.life](https://flos.life/vibe-docs-why-i-stopped-writing-documentation/)
4. **Context7 lo demuestra por contraste:** existe *porque* los agentes alucinan APIs de
   librerías de terceros. La respuesta es texto derivado del código, con versión. Eso ya es
   tu repo, para tu propio código.

**En contra (también válido):**
1. **El problema ya no es el formato, es la escala.** 1 715 ficheros / 18,3 MB, de los cuales
   1 007 son tareas. El modo de fallo no es podredumbre: es **no-descubrimiento**.
   `master-index.md` es hoy un punto único de fallo que ningún agente mantiene bien. Confluence
   con 1 715 páginas es igual de indescoverible, con peor buscador.
2. **Git no da garantía de frescura.** Una página obsoleta produce una respuesta
   *confiada y plausibly incorrecta*, mucho peor que el honesto "no sé" de una página ausente:
   el fallo es asimétrico y nadie lo detecta hasta que un cliente actúa sobre ello.
   Tus dos convenciones de nombre de ADR son literalmente *deuda de versión*: dos respuestas
   con apariencia autoritativa a "¿qué decidimos?".

**Veredicto: el modelo es correcto, la higiene no.** Migrar 1 715 ficheros a cualquier
herramienta de §3.1 costaría semanas, destruiría el `git log` (tu única ventaja real) y te
dejaría en un problema de descubrimiento *peor*, dentro de una herramienta con vendor.
Lo que falta no es un contenedor mejor: son **tres garantías mecánicas** (gate de enlaces,
enum de estado/supersesión en ADRs, índice regenerado por CI).

### 3.3 Formato de ADR: MADR 4.0.0

| Formato | Licencia | Estado | Por qué sí/no |
|---|---|---|---|
| **MADR** | **MIT OR CC0-1.0** | **v4.0.0**, 2,5k★, 4 variantes de plantilla | ✅ **Secciones obligatorias = lo que hace que un ADR sea un ADR**: Contexto → Opciones consideradas → Resultado. El campo *decision drivers* + *pros/contra* es lo que impide que un agente re-proonga la opción rechazada |
| Nygard | — | Antigua | ❌ Sin lista de opciones consideradas → el agente reaprende lo descartado |
| Y-Statement | — | — | ⚠️ Complemento de 1 línea, no reemplazo |
| YADR (MADR→YAML) | OSS | jun-2026 | ⚠️ Machine-parseable, demasiado nuevo para 54 ADRs |
| ISO/IEC/IEEE 42010 | — | — | ❌ 9 ítems, pesado |

Tu `docs/README.md` ya usa front-matter consistente (`title`, `type`, `status`,
`last_reviewed`, `tags`). **Añade `id`, `status` como enum, `supersedes`/`superseded_by` y
`scope`.** Eso hace tus 54 ADRs parseables por máquina sin migrar a YADR.

Checklist de validación (de [Zimmermann, "Ten Common Mistakes in ADRs", 2026-09-12](https://ozimmer.ch/practices/2026/09/12/ADRMistakes.html)):
contexto infraespecificado · criterios inferidos desde la opción preferida · **opciones
consideradas no registradas** ("una decisión que no identifica alternativas y elige una de
ellas no es una decisión") · sin consecuencias negativas.

---

## 4. Benchmark de pares: qué hacen realmente 25+ proyectos de BD/infra

SSG determinado por `<meta name="generator">` del HTML servido, fingerprint de assets, y
config del repo. `llms.txt` = código HTTP observado + `Content-Type` + primeros 60 bytes.

| Proyecto | Dominio | SSG | llms.txt | MCP producto | ADRs | AGENTS.md | Versionado | CI de docs | Generador ref API | Changelog |
|---|---|---|---|---|---|---|---|---|---|---|
| **Qdrant** | qdrant.tech | **Hugo 0.160.1** | ✅ 200 (177 KB, **plantilla Hugo** que no puede derivar del nav) | ✅ `mcp-server-qdrant` | ❌ | ✅ | single latest | ✅ lychee + snippets compilados **typechequeados** | ✅ ReRed ← utoipa (36 versiones) | GH Releases |
| **Mem0** | docs.mem0.ai | Mintlify | ✅ 200 **check-in + CI two-way diff** | ✅ `mem0-mcp` | ❌ | ✅ 7 pares | single | ✅ **el mejor gate**: `check-llms-txt-coverage.py` bidireccional, check requerido | ✅ OpenAPI→Mintlify | — |
| **LangChain** | docs.langchain.com | Mintlify | ✅ **índice anidado `/_llms/` con conteo de páginas** | ✅ | ❌ | ✅ **38 KB AGENTS.md** | single (por lenguaje) | ✅ links + Vale + **refresh diario de OpenAPI desde el servicio vivo** | ✅ diario desde servicio vivo | — |
| **Milvus** | milvus.io/docs | Next.js custom | ✅ 200 (82 KB) | ❌ 404 | ❌ | ✅ | ✅ **10 árboles de versión** | ⚠️ markdown-check nocturno, **no bloquea** | ⚠️ MDX curado a mano | — |
| **SurrealDB** | surrealdb.com | Vike | ✅ **check-in + regenerado en prebuild** | ✅ `surrealmcp` | ❌ | ✅ 9 CLAUDE + 2 AGENTS | single | ✅ `reindex-docs.yml` (el mejor CI leído) | ⚠️ solo `check-html-links` | — |
| **Chroma** | docs.trychroma.com | Mintlify | ✅ 200 | ✅ | ❌ | ✅ | single | ❌ **NINGUNO** en 2 653 ficheros | ✅ | — |
| **LanceDB** | docs.lancedb.com | Mintlify | ✅ 200 | ❌ 404 | ❌ | ✅ 3 | single | ✅ lychee diario `fail:false` | ✅ legacy mkdocs-swagger | — |
| **Weaviate** | docs.weaviate.io | **Docusaurus 3.9.2** | ✅ 200 | ✅ in-product | ❌ | ❌ AGENTS, ✅ CLAUDE | single | ❌ ninguno en 6 565 ficheros | ✅ **Scalar** ← OpenAPI | on-site |
| **Pinecone** | docs.pinecone.io | Mintlify | ✅ 200 | ✅ | n/a | ? | single | ? | ? | ? |
| **ClickHouse** | clickhouse.com/docs | **Mintlify en vivo; Docusaurus en repo** | ✅ 200 (102 KB) | ✅ | ❌ | ❌ | single | ✅ **Vale real**, líneas cambiadas, pin en `.mise.toml` | ✅ **el más profundo**: `autogenerate-settings` + Redocly | ✅ generado en build |
| **DuckDB** | duckdb.org/docs | Jekyll | ✅ 200 (3 KB, a mano) | — | ❌ | ✅ | `/docs/current/` | ✅ **`NeedsDocumentation.yml`: cambio de API ⇒ tocar docs, o falla** | C-docstrings in-tree | generado |
| **DataFusion** | datafusion.apache.org | Sphinx 9 + myst | ✅ **check-in** en repo | — | ❌ | ✅ | single | ✅ **el más minucioso**: build real path-filtered en PR | Sphinx + maturin | stub de 972 B |
| **Neo4j** | neo4j.com/docs | **WordPress (un CMS, no SSG)** | ✅ 200 | ✅ `neo4j/mcp` | ❌ | ⚠️ 1 par | `/current/` | ❌ | WordPress | on-site |
| **Memgraph** | docs.memgraph.com | Next.js | ✅ 200 | ❌ | ❌ | ❌ | single | ❌ | — | — |
| **Vespa** | docs.vespa.ai | Jekyll + Denali | ✅ 200 | ❌ | ❌ | ❌ | single | ✅ link-checker L/M/V, allowlist | ⚠️ ships reference *code* | — |
| **FalkorDB** | docs.falkordb.com | Mintlify | ✅ 200 | ✅ | ❌ | ❌ AGENTS, ✅ CLAUDE | single | ❌ | — | — |
| **Haystack** | docs.haystack.deepset.ai | Docusaurus 3.10.2 | ⚠️ **200 pero es un dump de 2,68 MB de sus propios AGENTS.md** | ❌ | ❌ | ✅ 7 pares | ✅ **mejor política pre-1.0** (versiona <1.0, no ≥1.0) | ✅ **ejecuta los snippets con API keys reales, nightly** | ✅ plugin generate-llms-txt | reno |
| **Zep/Graphiti** | help.getzep.com | **Fern** | ✅ 200 | ✅ in-repo | ❌ | ✅ | single | ⚠️ lint.yml | Fern | — |
| **LlamaIndex** | docs.llamaindex.ai | Astro 6.4.8 | ⚠️ **200 pero `text/html`, 247 KB de shell SPA** | ❌ | ❌ | ❌ | single | ⚠️ | ⚠️ dos stacks | ✅ **686 KB** autogenerado |
| **Polars** | docs.pola.rs | MkDocs + Material | ❌ 404 | — | ❌ | ❌ | single | ✅ **el más granular**: link-check bloqueante + `ruff-action` sobre `docs/source` (**linta el Python de los bloques**) | — | release-drafter |
| **txtai** | neuml.github.io/txtai | MkDocs Material | n/a — **txtai.ai NXDOMAIN, txtai.io en venta** | ❌ | ❌ | ❌ | ❌ **`mkdocs gh-deploy --force` en cada push** | ✅ deploy-only | ✅ griffe/mkdocstrings | — |
| **ArangoDB** | docs.arangodb.com | Hugo 0.164.0 | ⚠️ **200 pero `text/html`, 16 KB** | — | ? | ? | ✅ `/stable/` + `/3.12/` | ? | — | — |
| **FAISS** | faiss.ai | Sphinx | ❌ 404 | — | ❌ | ❌ | ❌ | ⚠️ deploy, no verificación | ✅ autodoc C++ | — |
| **pgvector** | (solo README) | **ninguno** | n/a | — | ❌ | ❌ | n/a | ❌ **cero workflows** | ❌ | ❌ |
| **Kuzu** | kuzudb.github.io/docs | Astro 5.5.6 en GH Pages | ❌ 404 | — | ❌ | ❌ | ❌ | ⚠️ workflow huérfano | ? | — |
| **SQLite / FoundationDB / Arrow / AGE** | varios | varios | ❌ 404 / n-a | — | ❌ | ✅ SQLite y FDB | evergreen / `/master/` | ❌ | ✅ C-docstrings | in-tree |

### 4.1 Patrones que emergen

**(a) `llms.txt` no tiene que ver con la maturity del proyecto.** Trackea **quién consume la doc
con un LLM**. DuckDB y DataFusion son viejos y lo tienen; Marqo y Turbopuffer son nuevos y no.
Las empresas comerciales de API tienen *support-engineer-in-the-loop* con asistentes; la
investigación OSS embebida tiene humanos.

**(b) El elección de SSG no es la palanca.** 11 sitios sin Mintlify tienen `llms.txt` limpio
(Qdrant/Hugo, DataFusion/Sphinx, DuckDB/Jekyll, SurrealDB/Vike, Neo4j/WordPress). Mintlify
solo **elimina la decisión**.

**(c) La escalera de madurez de CI de docs:**

1. **Nada** — Chroma, Weaviate, Memgraph, Letta, SQLite, FoundationDB, Kuzu, FalkorDB, Arrow.
   **Nueve de los mayores no tienen ningún workflow de docs.** Chroma es la prueba más dura: 2 653
   ficheros, Mintlify, `llms.txt` vivo, y ni un workflow.
2. **Solo deploy** — prueba que el sitio compila; no prueba nada del contenido.
3. **Link checking** — el gate real más común. Tres sabores:
   - **Vespa**: L/M/V 03:00, allowlist.
   - **LanceDB / Qdrant**: lychee diario/semanal, `fail: false`, resultados a **un issue de
     seguimiento**. El comentario explica por qué: bloquear PRs por links externos "trade a
     lot of false failures for very little signal".
   - **Polars**: link-check **bloqueante** + `ruff-action` sobre el código de los bloques.
4. **Prose linting** — solo 2: **LangChain** y **ClickHouse** (Vale sobre líneas cambiadas,
   motor pineado en `.mise.toml`: *"Bump the pin there, not here"*).
5. **Corrección del contenido doc** — el nivel raro y valioso: **Qdrant** compila sus snippets
   como fuentes reales y **falla el PR si `git status` queda sucio**; **Haystack** *ejecuta* los
   snippets con API keys reales cada noche; **LangChain** refresca el OpenAPI desde el servicio
   vivo cada día; **DuckDB** falla el PR si cambian la API sin tocar docs.
6. **Índice de agente aplicado como contrato** — **exactamente 1 proyecto: Mem0.**
7. **Frescura de la referencia contra el servicio vivo** — **exactamente 1: LangChain.**

**Nadie ejecuta `docusaurus build --strict` ni `mkdocs build --strict` como check requerido en
ningún sitio que pude encontrar.** El patrón es: build-on-PR, links, prosa y muestras de
código como preocupación separada.

**(d) Documentación versionada en BD pre-1.0 — cuatro estrategias, y la "obvia" no es la más común:**

1. **Árboles de versión numerados en el repo de contenido** — **Milvus** es el único
   practicante real: `v0.x/` … `v3.0.x/`, cada uno con su `package.json`. Coste: 10 árboles
   paralelos, repo de 55 226 ficheros.
2. **Versionado en el SSG con política "inestable → congelar"** — **Haystack** es la mejor
   respuesta a tu pregunta: `versioned_docs/version-3.2/` nativas, página `/versions`,
   `promote_unstable_docs.yml` que dispara **solo** en tags `v[0-9]+.[0-9]+.0` y
   **excluye explícitamente 1.x**. Versionan duro por debajo de 1.0 y deliberadamente no
   versionan en 1.0+. **Es el único sitio que codifica una *política* sobre cuándo un cambio
   rompiente recibe una serie de docs.**
3. **Versión en el path, alias "current"** — ArangoDB (`/stable/`, `/3.12/`), Neo4j
   (`/cypher-manual/current/`). Lo más barato que funciona.
4. **Un solo "latest", sin versiones** — la mayoría. Y **txtai hace la versión activamente
   dañina**: `gh-deploy --force` en cada push, así que cada URL de doc queda silenciosamente
   mal en el siguiente cambio rompiente, sin redirect ni aviso.

**Lectura honesta: solo Milvus y Haystack resuelven realmente docs versionadas para una BD
pre-1.0. La mediana hace trampa.** Para VantaDB, la política de Haystack es la plantilla.

### 4.2 El hueco no ocupado, puntuado

| | Doc legible por agente | Modelo de arquitectura validado | Referencia generada desde código | ADRs en git |
|---|---|---|---|---|
| **Qdrant** | ✅ output `.md` *autoral* + llms.txt templado + skills **medidas** | ❌ | ✅ snippets compilados y typechequeados | ❌ |
| **Mem0** | ✅ aplicado en CI, diff bidireccional | ❌ | ❌ | ❌ |
| **LangChain** | ✅ índice anidado + 7 MB full | ⚠️ | ✅ refresco diario desde servicio vivo | ❌ |
| **Milvus** | ✅ 82 KB llms.txt + skills en 4 harnesses | ✅ `docs/agent_guides/` | ⚠️ | ❌ |
| **ClickHouse** | ✅ | ❌ | ✅ el más profundo | ❌ |
| **Qdrant/… (resto)** | parcial | ❌ | parcial | ❌ |
| **VantaDB hoy** | ⚠️ llms.txt mínimo, sin `.md` en URL | ✅ **CodeGraph pre-indexado** | ⚠️ `cargo doc` artefacto, no gate | ✅ **54 ADRs** |

**Nadie hace las cuatro.** La columna de ADRs es **0 de 25+** — la apertura más limpia.

**El espacio no ocupado, enunciado con precisión:** *documentación legible por agentes
aplicada en CI como Mem0 la hace, sobre un modelo de arquitectura generado y validado contra
un grafo de código como el CodeGraph de VantaDB, con una referencia generada desde el código
que no puede derivar, más ADRs en git que registren las decisiones que un agente tomó.*
Cualquier **dos** de esas ya es top-decile. Las cuatro no tiene ocupante. Y **la primera ya es
checkbox**, así quelidera con la tercera y la cuarta en lo que publiques.

### 4.3 Lo que hacen los 5 mejores, en detalle

**Qdrant** — el único que trata la salida para agente como **artefacto de build de primera
clase con su propia plantilla**. Cada página se renderiza dos veces: `single.markdown.md`
emite `<url>/index.md` junto al HTML, con punteros al catálogo de skills. Su `AGENTS.md` es
explícito en que esto **no es una conversión del HTML**: los shortcodes tienen variantes
`.markdown.md` autorales (`code-snippet`, `include`, `prompt`) para que la vista de agente sea
*escrita*, no *raspada*. `llms.txt` es una plantilla Hugo que camina `.Site.Sections` y tira
`.Description` — **por lo tanto no puede derivar**. Además: defensa contra prompt-injection en
el build (los prompts se sacan del cuerpo inline porque "reach the agent-facing `index.md`,
where an instruction addressed to an agent can displace the question that agent was actually
asked"), y `qdrant/skills` con `SCORING.md` de 28 KB que define
`lift(model) = mean_score(with-skill) − mean_score(no-skill)`. **El único par que mide si su
documentación para agentes funciona.**

**Mem0** — `docs-llms-txt-check.yml` corre `check-llms-txt-coverage.py` y hace un **diff
bidireccional**: páginas en `docs/**/*.mdx` sin entrada en `llms.txt` **y** entradas que
apuntan a páginas inexistentes **ambos fallan**. Es un check requerido por `ci-gate.yml`. Su
`docs/AGENTS.md` declara el invariante en tres ítems y acopla dos reglas: *"Code samples must
be runnable. If a sample calls a public SDK method, it has to match the real signature"* y
*"Any change to a public SDK signature has to update the matching page here in the same PR."*

**LangChain** — el `llms.txt` más inteligente: 3 KB que apunta a **índices anidados `/_llms/`
con conteo de páginas** ("AGENT DEVELOPMENT LIFECYCLE (1478 pages)"), con instrucción
explícita de seguir recursivamente. CI: link-check, Vale sobre líneas cambiadas con motor
pineado en `.mise.toml`, y un cron diario que re-fetch-ea el spec vivo y abre PR. **La
referencia de API no puede pudrir en silencio contra el servicio.**

**SurrealDB** — `llms.txt` es **a la vez fichero check-in** (`public/llms.txt`, 99 KB, aparece
en diffs de PR y es revisable) **y artefacto de build** (`prebuild` lo regenera).
Revisable *y* regenerable *y* incapaz de derivar. Su `reindex-docs.yml` es el CI mejor
escrito: documenta por qué un push squash y uno directo disparan ambos; por qué el indexador
tira del archivo del repo en vez de rastrear el sitio publicado ("does not wait for the Vercel
deployment… a brand-new page can therefore appear in search results shortly before its URL is
live"); y pinea `permissions: contents: read` con el comentario de que `{}` "would be tempting
but sets every scope to none".

**Haystack** — dos cosas individualmente más valiosas que nada más en el set: su **política de
versionado** y la **ejecución nocturna de los snippets doc con API keys reales**
(`docs-website-test-docs-snippets.yml`, 03:17 UTC).

### 4.4 Los 5 peores, y el fallo concreto

**Kuzu — abandono total, y es una graph DB embebida pre-1.0, tu par exacto.** El README abre:
*"We are archiving the KuzuDB project here…"*. `kuzu.dev`, `kuzudb.com`,
`docs.kuzudb.com`: **NXDOMAIN**. PyPI sigue sirviendo v0.11.3 apuntando al repo lápida. 5 839
ficheros, **sin `docs/`, sin ADRs, sin `AGENTS.md`, sin `CLAUDE.md`**. El fallo no es
descuido: es que nada de la documentación fue un activo del que alguien rindiera cuentas.

**txtai — el dominio canónico es una página aparcada.** `txtai.ai` → **NXDOMAIN**.
`txtai.io` → **HTTP 200, `<title>TxtAi.io for sale | Spaceship.com</title>`**. Compuesto:
`docs.yml` son 470 bytes con `mkdocs gh-deploy --force` en cada push.

**FAISS — no hay directorio `docs/` en el repo.** 1 180 ficheros, `docs/` no está entre ellos.
`faiss.ai/llms.txt` → 404. Para la librería ANN más citada de la historia, la superficie doc
es un README a mano.

**pgvector — 169 ficheros y nada más.** Sin `docs/`, sin `.github/workflows/` en absoluto, sin
`CONTRIBUTING.md`, sin `CHANGELOG.md`.

**ArangoDB y LlamaIndex — el soft-200, que es peor que un 404.** ArangoDB sirve HTML de 16 KB
bajo la ruta del índice de agente; LlamaIndex, 247 KB de shell SPA. Un cliente automático que
solo mira el status code ingiere un cuarto de megabyte de HTML como índice documental.
**Sirve `text/plain` o `text/markdown` y valida contenido, no status.** Nadie lo ha hecho.

### 4.5 Fallos que un chequeo por status code no atrapa

Esto es la lección más transferible del benchmark:

| Sitio | Status | `Content-Type` | Primeros bytes del cuerpo |
|---|---|---|---|
| ArangoDB `llms.txt` | **200** | `text/html` | `<!doctype html>…<title>Arango Documentation</title>` |
| LlamaIndex `llms.txt` | **200** | `text/html` | `<html class="astro-avtbfcnz">` |
| Haystack `llms.txt` | **200** | `text/plain` | `// File: AGENTS / ## docs-website/docs/ Guidelines` (2,68 MB de sus propios AGENTS.md) |

Ninguna de las tres es un `llms.txt`. **Ninguna falla donde un 404 sí fallaría.**

---

## 5. Los 6 huecos de VantaDB, con su corrección

Cada uno es un diff pequeño contra algo que ya existe.

### 5.1 `llms.txt` está incompleto y desactualizado

**Estado:** 110 líneas, 4 URLs, versión `0.5.0` (el repo tiene 0.6.0 en `target/`), apunta a
`github.com/blob/...` (HTML renderizado, no raw) sin negociación de contenido, sin `.md` en la
misma URL, sin `Link:` header.

**Corrección (comparar con Qdrant/Mem0/SurrealDB):**
1. Check-in en `docs/llms.txt` **y** regenerado por script en build → *revisable y no derivable*
   (patrón SurrealDB).
2. URLs `raw.githubusercontent.com/...` (texto plano, no HTML).
3. Puntos a `.md` en la misma URL de cada página + cabecera `Link: rel="alternate" type="text/markdown"`
   (llms.txt v2 §1, no requiere tocar páginas).
4. Diff bidireccional en CI (patrón Mem0): página sin entrada **y** entrada sin página fallan.
5. Validación de `Content-Type` — la lección de ArangoDB/LlamaIndex/Haystack.
6. **Excluir `CHANGELOG.md` (175 KB) y `docs/dev/tasks/` (1 007 ficheros)** del índice.
7. Correr la versión desde `Cargo.toml` en build, no a mano.

**No hacer:** `llms-full.txt` (no está en la spec; para 18,3 MB sería una bomba de contexto).

### 5.2 Los bloques de código de `docs/` no se ejecutan

**Estado:** `ci-examples.yml` ejecuta `examples/` (`cargo run --example basic`, `python
examples/python/*.py`). `ci-rustdoc.yml` genera un artefacto `cargo doc` — **no** es
`cargo test --doc`, no es `-D warnings`. **Ningún bloque de código dentro de `docs/` se
ejecuta jamás.**

Esto es el hueco de mayor impacto. La literatura de gobernanza de docs con IA es consistente
en un punto: **arXiv 2609.04218** (Kwon, v1 2026-07-01, v3 2026-09-15) reporta que *ambas*
condiciones (revisión estructurada y prompt desestructurado) **fallaron en detectar los
defectos de calidad documental que un revisor humano QA sí encontró**; y la revisión de
severidad fue **retirada** tras reevaluación a ciegas, con un re-test que **no replicó**
(6/24 vs 5/24). La conclusión honesta: **la automatización de revisión tiene su hueco
exactamente donde viven los defectos documentales.** Lo que funciona es la *verificación*.

Ejemplos ejecutables (Qdrant, Haystack, Polars) son la respuesta estructural a "los agentes
escriben ejemplos de API plausibles pero equivocados": un ejemplo equivocado **no compila**, así
que nunca llega a un humano.

**Corrección:**
```bash
# 1. Rust: el gate de una línea de mayor ROI de todo este informe
RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" cargo doc --no-deps --workspace
RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace
# 2. Python: pydoclint (0.10.1, 2026-09-28) — docstring ↔ firma
pydoclint --check-style=numpy vantadb/
# 3. TS: typedoc + examples como tests
```
Más: un extractor de los bloques ```` ```python ```` / ```` ```rust ```` de `docs/` hacia un
`docs-examples/` que los importe y ejecute — el patrón Haystack.

### 5.3 Sin gate de link-checking

**Estado:** cero. `gate-docs.yml` hace markdownlint + frontmatter + paridad de versión. No
verifica ni un enlace en 1 715 ficheros.

`lychee` es el detector de rotura más barato que existe: Apache-2.0, Rust, 3 956★, último
push 2026-09-28. Y es el **único** que hace `include_wikilinks`, `include_fragments = "full"`
y `offline` — exactamente lo que necesita un vault con `[[wikilinks]]` y enlaces relativos
`[x](path.md)`.

```toml
# lychee.toml — el archivo que hace funcionar tu vault de doble estilo de enlace
no_progress = true
accept = ["200", "429"]              # 429 = rate-limited, no roto
include_wikilinks = true             # valida también [[wikilinks]]
include_fragments = "full"           # valida #anchors
offline = true                       # solo ficheros locales → cero 429
fallback_extensions = ["md", "html"]
index_files = ["README.md", "index.md"]
exclude_path = ["docs/CHANGELOG\\.md$", "^docs/dev/tasks/", "^docs/dist/"]
```
```yaml
# añadir a gate-docs.yml
      - uses: lycheeverse/lychee-action@v2
        with:
          args: --config lychee.toml --no-progress 'docs/**/*.md'
          fail: true
```

**Empieza con `fail: false` + un issue de seguimiento** (patrón Qdrant/LanceDB: el link-check
externo es ruidoso y bloquear PRs por él compra pocos falsos positivos), y sube a bloqueante
cuando la base esté limpia.

### 5.4 54 ADRs, 3 convenciones de nombre, 0 estado machine-readable

**Estado:** 41 `ADR-*.md`, 7 `NNN_*.md`, 6 otros. Sin campo `status` enforceable, sin
supersesión. Un agente que cite un ADR obsoleto no tiene forma de saberlo — *"Agents won't
determine which ADRs are stale"* es exactamente tu riesgo, reportado por un practicante en
r/SoftwareArchitecture en 2026.

**Corrección (una tarde, un commit, ~100 líneas de CI):**
1. Inventario + `adr/_migration-map.csv` (el mapa es lo que hace funcionar `git log --follow`).
2. Renombrar en **un commit** con `git mv`; corregir enlaces entrantes en el mismo commit.
3. Nombre canónico: `NNNN-slug.md` (4 dígitos, cero-padding, guion, sin prefijo) — elimina las
   tres convenciones y `ls` ordena numéricamente.
4. Front matter por ADR: `id`, `title`, `status` (enum `proposed|accepted|superseded|deprecated`),
   `date`, `owners`, `supersedes`, `superseded_by`, `scope`.
5. CI: (a) regex de nombre canónico, falla si aparece `ADR-*` o `NNN_*`; (b) ≥2 opciones bajo
   *Considered options*; (c) `supersedes`/`superseded_by` **bidireccionales y consistentes**.
6. `index.md` regenerado por CI desde el front matter, nunca a mano.

**No adoptar** `adr-tools` (genera `0001-title.md` — exactamente tu problema), ni `log4brains`
(dependencia Node para algo ya navegable como ficheros), ni `adr-log` (la propia comunidad ADR
lo lista como no mantenido).

**Y no pongas los ADRs como fuente en Structurizr**: el export estático los borra.

### 5.5 1 007 task files dentro del árbol documental

**Estado:** 912 en la raíz de `docs/dev/tasks/` + 93 en `complete/` + 2 en `closed/`. Es el
**57% de todo `docs/`** y el mayor pasivo de grafo de enlaces del repo.

**Corrección:** los archivos de tarea son *work items*, no documentación. No necesitan estar en
el mismo árbol que la referencia de API. Mover `docs/dev/tasks/` fuera de `docs/` (o a
`tasks/` en la raíz) elimina de un plumazo la mitad del problema de descubrimiento. Si deben
quedarse: índice generado por CI, y **prohibido** que la prosa de `docs/` enlace dentro.

El argumento a favor de borrar es real: *"documentation that competes with actual work always
loses"* — y `git log` ya es el archivo. Mover, no borrar: el historial vive en git.

### 5.6 Sin gate anti-fuga y sin superficie pública canónica

**Estado:** (a) no hay escaneo de secretos/hostnames/nombres de clientes en `docs/`; (b) no
existe sitio de docs, luego no hay URL canónica.

**(a) Fuga.** Los dos mayores incidentes doc-adjentes de la historia fueron documentación
interna + credenciales en el mismo VCS:
- **CISA "Private-CISA", 2026-05-18** ([Krebs on Security](https://krebsonsecurity.com)):
  `importantAWStokens` con credenciales administrativas a tres servidores AWS GovCloud,
  `AWS-Workspace-Firefox-Passwords.csv` con credenciales en claro de decenas de sistemas
  internos, y **ficheros detallando cómo CISA construye, testea y despliega software
  internamente**. La cuenta tenía **desactivado el secret-blocking por defecto** de GitHub.
- **Uber 2016**: credenciales admin de AWS hardcodeadas en un repo **privado**; 57M registros.
  El fallo no es "los repos públicos son peligrosos" — es que el VCS es donde viven *tanto* la
  arquitectura *como* las claves, así que un compromiso da ambas.

Con 1 715 ficheros de los que la mitad son internos, es el gate que más probablemente no
tienes. Es un fichero de
config regex. Pulsa también el push protection de GitHub en los repos públicos.

**(b) Superficie pública.** Sin sitio, no hay URL canónica para enlazar ni para que los labs
lo citem. El coste es ~$0 (hosting estático). Y hay una razón concreta más allá del SEO:
**GitHub no está en el top-50 de dominios más citados en ChatGPT** ([Ahrefs, 2026-09-02],
metodología consumer-US/all-topic, que *no* habla de queries de desarrollo — úsala con cuidado,
pero no es un buen sustituto de un dominio propio).

**SSG recomendado para `docs/api/`:** **MkDocs Material con `docs_dir: docs/api`** — la
opción más inequívoca para renderizar *una subcarpeta* como sitio propio, la más tolerante con
Markdown escrito a mano, con `--strict` como gate de enlaces gratis. Está en modo
mantenimiento (EOL 2027-05-05) pero como *renderer congelado* está bien; **no lo conviertas en
la base de una cadena de herramientas nueva**. Si en el futuro quieres un segundo sitio, **Hugo**
(Docsy, Apache-2.0) es el único que documenta escala grande y se mantiene rápido al crecer.

**No construyas Docusaurus sobre `docs/` completo** — además de los problemas de Webpack, te
construiría los 1 007 task files.

---

## 6. Toolchain: qué instalar y qué descartar

### 6.1 Instalar (ordenado por ratio impacto/esfuerzo)

| Herramienta | Versión verificada | Por qué esta |
|---|---|---|
| **lychee** | 0.24.2 (crate 2026-05-01), `lychee-action@v2.9.0` | Único con wikilinks + fragments + offline. El mayor ratio de la lista |
| **typos-cli** | 1.50.3 (2026-09-25) | Un binario, sin config para el caso común. ⚠️ el crate `typos` de crates.io es **otro, obsoleto, 0.10.44** |
| **cspell** | 10.3.5 (2026-09-27) | Mejor encaje para términos inventados (`vantadb`, `PyO3`, `HNSW`, `RRF`, `warp`, `Fjall`) |
| **pydoclint** | 0.10.1 (2026-09-28) | El gate de docs Python de mayor valor: deriva docstring ↔ firma |
| **cargo-semver-checks** | 0.50.0 | Gate de merge sobre la superficie pública |
| **vacuum** | 0.30.6 (2026-09-15) | Lint OpenAPI, binario Go, cero config. Sustituto de Spectral |
| **oasdiff** | 1.33.0-rc.1 | Gate duro de cambio rompiente |
| **remark-obsidian** | 1.13.0 (2026-09-28) | La única herramienta viva que convierte `[[x]]` ↔ `[x](x.md)` en ambos sentidos |
| **dprint** | 0.57.4 + plugin-markdown 0.24.0 | Determinista, más barato que prettier. ⚠️ los plugins **no son paquetes npm** |
| **mise** | push 2026-09-28 | Pinea todas las herramientas de docs en un fichero |
| **check-jsonschema** | 0.38.2 (2026-09-23) | Deriva config ↔ doc |
| **mkdocs-material** | 9.7.7 | Solo como renderer de `docs/api/` |

### 6.2 Descartar explícitamente (con evidencia de fecha)

| Descartar | Último release | Antigüedad a 2026-09-28 |
|---|---|---|
| MkDocs Material (como dependencia) | EOL anunciado | **2027-05-05** |
| `slatedocs/slate` | — | **`archived: true`** |
| `docusaurus-plugin-llms-txt` | 2024-12-30 | 1,9 años |
| `write-good` | 2021-02-16 | 5,6 años |
| `remark-snippet` | 2021-08-05 | 5,1 años |
| `snowstorm` | 2020-06-06 | 6,3 años |
| `mdbook-wikilink` | 2021-08-27 | 5,1 años |
| `git-chglog` | 2023-02-15 | 3,6 años |
| `dredd` | 2021-11-16 | 4,9 años (→ `schemathesis` 4.28.0, 2026-09-22) |
| `pydocstyle` | 2023-01-17 | 3,7 años (→ pydoclint) |
| `pydoc-markdown` | 2023-06-26 | 3,3 años (→ griffe 2.3.0 + mkdocstrings 1.0.6) |
| `docstr-coverage`, `interlink`, `docs-coverage` | — | **Sin repo vivo localizable** |
| `swagger-cli`, `auto-changelog`, `version-sync` | — | **404 en GitHub / PyPI** |
| `markitdown`, `docling`, `unstructured` | activos | ❌ **El desperdicio más común**: convierten PDF/Office → MD. Tu fuente *ya es* Markdown |
| **RAG / índice de embeddings sobre `docs/`** | — | ❌ Ver abajo |
| `Vale` (paquetes de estilo completos) | 3.23.0 | ⚠️ ~4 000 alertas el día 1 en 1 715 ficheros. Solo `Vale.Spelling` + vocabulario, o nada |

**Sobre el índice vectorial:** hay consenso documentado en 2026 en contra, y lo marco porque
es el error más caro disponible aquí:
- [arXiv 2605.15184, "Is Grep All You Need?"](https://arxiv.org/abs/2605.15184)
- [LlamaIndex: "Is grep all you need?"](https://www.llamaindex.ai/blog/is-grep-all-you-need-lexical-vs-sematic-search-for-agents) — "for a codebase, a docs folder, a handful of markdown notes, lexical search is fast, predictable, and gives agents exactly the right thing"
- [jxnl.co: "Why I Stopped Using RAG for Coding Agents"](https://jxnl.co/writing/2025/09/11/why-i-stopped-using-rag-for-coding-agents-and-you-should-too/) — "unnecessary and potentially counterproductive"

18,3 MB de texto plano, 1 715 ficheros, en un repo que los agentes ya clonan. `rg` sobre eso
son milisegundos, siempre fresco, cero infra, cero podredumbre. Un índice vectorial sería
estrictamente peor en todos los ejes salvo *fuzzy synonym matching*.

### 6.3 Orden de los jobs (rápido → lento), ~3 min en un PR de docs

| # | Job | Comando | Runtime | Bloquea |
|---|---|---|---|---|
| 1 | `docs-links` | `lychee --config lychee.toml --no-progress 'docs/**/*.md'` | 30-60 s | sí |
| 2 | `docs-lint` | `markdownlint-cli2 'docs/**/*.md'` *(ya existe)* | 10-20 s | sí |
| 3 | `docs-terms` | `typos docs/` o `cspell` | 15-30 s | sí (baseline) |
| 4 | `docs-format` | `dprint check docs/**/*.md` | 10-15 s | sí |
| 5 | `doc-examples` | `cargo test --doc` + runner de ejemplos Python/TS de `docs/` | 2-4 min | sí |
| 6 | `api-breaking` | `oasdiff breaking base.json openapi.yaml` | 90 s | sí |
| 7 | `doc-tests` | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` | 60-90 s | sí |
| 8 | `agent-docs` | `python tools/check_llms_txt.py` (diff bidireccional) | 20 s | sí |
| 9 | `nightly` | `mkdocs build --strict` + link-externos | 4-6 min | nocturno, no bloquea |

Todos con `paths: ['docs/**']` para no gravar PRs de solo-código.

---

## 7. Decisión: tres stacks

### Stack A — "Cortar y hacer cumplir" ★ recomendado

**`docs/` en git · ADRs con front matter y enum de estado · un validador en CI · lychee + typos
· `AGENTS.md` como entrada de agente · sin renderer, o `quartz` read-only**

- **Elige esto si** tienes una tarde y presupuesto cero. Arregla ~90% de los problemas reales
  con coste ≈ 0 y lock-in ≈ 0.
- **Coste:** ~$0, ~100 líneas de CI, 2 dev-deps.
- **Lock-in:** ninguno. El "export" es un no-op porque no hay formato.
- **Legible por agentes:** máximo. Los ficheros en git *son* la interfaz; `AGENTS.md` es
  estándar Linux Foundation; el MCP oficial de GitHub (56 tools) los lee nativamente.
- **Lo que NO arregla:** no reduce los 1 715 ficheros. Eso requiere **mover/borrar** (§5.5), y
  borrar es la única palanca que funciona ahí.
- **Lo que te pierdes:** ninguna UI humana. Si nadie más que los agentes lee esto, es una victoria.

### Stack B — "Cortar y hacer cumplir + un modelo validado"

**Stack A, más `docs/dev/architecture/workspace.dsl` (Structurizr C4 DSL, MIT) validado en CI y
consultado por el MCP gratuito de Structurizr (`mcp.structurizr.com/mcp`)**

- **Elige esto si** los agentes están a punto de hacer cambios arquitectónicos sin consultar tus
  límites. VantaDB tiene 6 bindings y un core vector/grafo con fronteras reales.
- **Por qué es cualitativamente distinto:** el agente no *lee prosa sobre* la arquitectura,
  **recibe `validate` / `parse` / `inspect` atrás**. Structurizr **exige la jerarquía de
  abstracción C4**, así que un agente no puede inventar un componente dentro de un sistema.
  El propio análisis de Structurizr sobre IA
  ([docs.structurizr.com/ai](https://docs.structurizr.com/ai)) rechaza tanto las herramientas
  de diagrama por UI (Visio, draw.io) como *diagrams-as-code* (PlantUML, Mermaid, D2) para
  workflows de IA, precisamente porque "don't understand the C4 model and don't respect its
  rules". **Es decir: tus diagramas Mermaid actuales son la opción más débil, no la segura.**
- **Coste:** ~$0. Un `.dsl`, un paso de CI, sin ecosistema que aprender. Contraste: Backstage,
  2-5 FTE, tres CVEs RCE, y un motor de docs en modo mantenimiento hasta 2027-05-05.
- **Honest cost:** un DSL más. Y regla dura: **no pongas los ADRs como fuente en la sección
  Decisions de Structurizr** — el export estático los borra. Modela fronteras; las decisiones
  van en MADR.
- ⚠️ No verificado: **el subpath `docs/dev/architecture/adr` ya existe**, así que modela el
  core, no cada ADR.

### Stack C — "Añade UI humana, git sigue canónico"

**Stack A o B, más un sitio *read-mostly* generado desde `docs/api/` (MkDocs Material) a
`docs.vantadb.io`, y opcionalmente Docmost si necesitas un editor Confluence-grade**

- **Elige esto si** hay inversores, colaboradores, o una segunda persona que no vaya a
  instalar Obsidian ni leer Markdown crudo. Si no, este stack es coste puro.
- **Lock-in:** bajo **solo si el sitio se genera desde git y nunca se edita a mano.** El
  momento en que alguien edite en Docmost y no en git, tienes dos fuentes de verdad y el
  problema de podredumbre de Confluence. Impón *generate-only* o no lo hagas.
- **No recomendado:** Confluence (DC EOL 2029-03-28), Notion (sin self-host; el MCP OSS está
  archivado), Outline (BSL 1.1), Backstage+TechDocs (motor en modo mantenimiento, 3 CVEs
  RCE), Swimm (vivo pero sin financiación desde nov-2021, y code-coupled).

### Independentemente del stack, tres cosas

1. **Sirve `.md` limpio en la URL de cada página** + cabecera `Link:` (llms.txt v2 §1) —
   gratis, y es lo que hacen Mintlify, Qdrant y SurrealDB.
2. **Regístrate en Context7** ([upstash/context7](https://github.com/upstash/context7), 62,5k★,
   push 2026-09-28) — es el mecanismo por el que un agente *encuentra* tu documentación de
   terceros. Ortogonal a todo lo demás.
3. **Gate de build-time.** A 175 KB de changelog y 1 715 ficheros, los agentes producirán
   referencias cruzadas rotas; una referencia de API generada sin validar es *peor que ninguna*,
   porque parece completa.

---

## 8. Lo que deliberadamente NO se hizo

| No hecho | Por qué |
|---|---|
| Migrar `docs/` a Confluence/Notion/Outline/Backstage | Destruye `git log` (tu ventaja real), crea segunda fuente de verdad, y elude el problema real (higiene, no formato) |
| Recomendar Mintlify/GitBook/Scalar de pago | $0–$450/mes contra una solución MIT que cuesta un afternoon. El único escenario donde Mintlify gana es "quiero un editor WYSIWYG y no tengo a nadie que lo escriba" — y tus editores son agentes |
| Construir un sitio sobre todo `docs/` | Te construiría los 1 007 task files por Docusaurus; con Hugo/MkDocs es un desperdicio de build |
| Recomendar un índice RAG/embeddings sobre `docs/` | El consenso 2026 documentado dice que es peor que `rg` para texto en repo |
| `llms-full.txt` | No está en la spec; para 18,3 MB es una bomba de contexto |
| Adopting `Zensical` | 11 meses de edad, con tier comercial adjunto — la misma forma que produjo el problema en MkDocs. Observar |
| Adoptar YADR (YAML) | 3 meses. La idea correcta, el momento equivocado para 54 ADRs. Añadir `id`/`status` a MADR da el 80% |
| Una métrica de "docs quality" | Infalsable por construcción. Ojo: el post de Ahrefs de "top-50 dominios más citados" está, literalmente, *"generated and automatically kept up to date every month by Agent A"* — un vendor usando un agente para publicar números recurrentes es el patrón en el que desconfiar |
| Una plataforma de IDP | El 89% de share de Backstage está medido entre empresas *ya están persiguiendo* un IDP. Irrelevante para n=1-3 |

---

## 9. Lo no verificado (declarado, no adivinado)

- **Rate limit de la API de GitHub** en varias fases del benchmark: los conteos de estrellas y
  fechas de push vienen de raspar las páginas de repo, no de la API. Trátalos como ±margen.
- **VitePress v2** sigue en alpha (`latest` pineado a 1.6.4 de 2025-08-05). Pínalo.
- **`docling` vs `markitdown`**: la cifra de "27× más lento" viene de un post de un tercero, no
  de un benchmark revisado por pares. Irrelevante en tu caso (no conviertes PDF).
- **Tiempos de build a 1 700 ficheros**: extrapolados de mediciones de terceros a 100 páginas.
  Los OOM/memoria de Docusaurus están citados y son de primera mano; los *timings* no.
- **La conclusión "0 de 25 tienen ADRs"** es **basada en rutas de árbol**, no en búsqueda de
  código (GitHub code search requiere auth). El regex no tuvo control positivo validado
  (kubernetes y grafana también dieron 0, lo cual es correcto para ambos pero no prueba que el
  patrón matchee). Trátalo como alta confianza, no como probado.
- **Zensical Studio** lanza 2026-11-05 con precios de Pro/Team sin publicar. No diseñes alrededor.
- **Precios de Redocly** por módulo están detrás de una calculadora interactiva.
- **Speakeasy**: los límites de tier están ofuscados por el reframe a facturación por petición MCP.
- **Northflank** solo publica precios de cómputo, no un SKU de developer portal distinto.
- **No hay ROI publicado y creíble** de docs-as-code. Lo que circula es content marketing de
  vendors (pushpen, slite, repowise, ekline, datadef — todos vendiendo el arreglo del problema
  que describen). **No lo he citado como evidencia.**
- **Atribución a "el SRE Book de Google dice docs-as-a-product"**: verificado por fetching ambos
  TOCs. El SRE Book (34 capítulos) y el SRE Workbook (21) **no tienen capítulo de
  documentación**. Esta atribución popular es infalsificable en las fuentes que se le atribuyen.
  Trátala como `[FOLKLORE]`.
- La afirmación "GitHub está fuertemente rastreado por GPTBot/ClaudeBot y los subdominios de
  docs no" circulaba en el brief: **no encontré datos por dominio en ninguna dirección.**

---

## 10. Fuentes

**Primarias:** [llmstxt.org v2](https://llmstxt.org/) (mod. 2026-08-10) ·
[mintlify.com/pricing](https://www.mintlify.com/pricing) · [gitbook.com/pricing](https://www.gitbook.com/pricing) ·
[readme.com/pricing](https://readme.com/pricing) · [scalar.com/pricing](https://scalar.com/pricing) ·
[atlassian.com Data Center EOL](https://www.atlassian.com/licensing/data-center-end-of-life) ·
[obsidian.md/blog/free-for-work](https://obsidian.md/blog/free-for-work/) ·
[adr.github.io/adr-templates](https://adr.github.io/adr-templates/) ·
[adr.github.io/adr-tooling](https://adr.github.io/adr-tooling/) ·
[diataxis.fr/news](https://diataxis.fr/news/) · [docs.structurizr.com/ai/mcp](https://docs.structurizr.com/ai/mcp) ·
[docs.structurizr.com/ai](https://docs.structurizr.com/ai) · [zensical.org/upcoming-changes](https://zensical.org/upcoming-changes/) ·
[developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt) ·
[developers.google.com/search/docs/essentials/spam-policies](https://developers.google.com/search/docs/essentials/spam-policies) ·
[Linux Foundation: Agentic AI Foundation](https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation) ·
[Obsidian: `github.com` top-50 más citados — Ahrefs 2026-09-02](https://ahrefs.com) · `survey Docusaurus #10895` ·
`issue #4765` · `issue #3652` · `issue #10556` · `discussion #11664` · `issue #11923` ·
`squidfunk/mkdocs-material#8523` · `mkdocs discussions #4010` ·
`fpgmaas.com/blog/collapse-of-mkdocs` · `backstage#32815` · `GHSA-6jr7-99pf-8vgf` ·
`CVE-2026-25153` / `CVE-2026-29186` / `CVE-2026-88064` ·
`flos.life/vibe-docs-why-i-stopped-writing-documentation` · `flos.life/why-backstage-might-not-be-worth-it` ·
`newsletter.getdx.com/p/backstage-and-the-developer-portal-market` ·
`notion.com/guides/mcp/hosting-open-source-mcp` · `cognition.com/blog/deepwiki-mcp-server` ·
`krebsonsecurity.com` (CISA, 2026-05-18) · `zimmermann ADRMistakes 2026-09-12` ·
`catio.tech/blog/architecture-decision-record` · `arXiv 2605.15184` · `arXiv 2609.04218` ·
`arXiv 2608.26232` (GROUP 2027) · `angeo.dev` (análisis Ahrefs de llms.txt) ·
`SE Ranking 300k dominios (2025-11-07)` · `Cloudflare AI crawler report (2025-08-29)` ·
`HN 44074626` · `News Engineer's LLMs.txt death` (analysis) ·
`Codex / Mem0 / Qdrant / LangChain / SurrealDB / Milvus / Haystack repos (leídos in-tree)`.
