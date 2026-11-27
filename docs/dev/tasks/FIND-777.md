---
title: "FIND-777: vendorizar la skill `progreso` en `.opencode/skills/` y cortar su dependencia de `~/.agents/skills/`"
kind: task
status: stable
description: "La skill `progreso` —la que cierra el estado de las tareas— vive en `~/.agents/skills/`, fuera del repo y sin versionar. Decision del owner: vendorizarla en `.opencode/skills/progreso/`, adaptar sus rutas, registrarla en el indice de skills y probar mecanicamente que corre desde el repo."
---

# FIND-777: vendorizar la skill `progreso`

## Metadata

- **Fuente:** fila `FIND-777` de `docs/dev/Backlog.md` (hallazgo F12 de la auditoria del harness, 2026-11-26)
- **Tipo:** harness / `.opencode/` — sin codigo de producto (no se compila Rust)
- **Creado:** 2026-11-27
- **Estado:** ✅ COMPLETED
- **Decision del owner (ya tomada):** **vendorizar** la skill dentro de `.opencode/`. No dejarla global, no "quitar la dependencia".

## Problema (medido, no derivado)

La skill que **cierra el estado de una tarea** (`skill progreso`, invocada por
`pipeline-full.md:196,217,223`) vivia fuera del repo:

| Hecho | Medicion |
|---|---|
| Existe solo en el catalogo global | `C:\Users\Eros\.agents\skills\progreso\SKILL.md` (9 716 B, 1 archivo) |
| Ausente del repo | `Test-Path .opencode/skills/progreso` → `False` |
| No versionada | `.agents/` esta en `.git/info/exclude:9` |
| Su unico enlace relativo esta **roto** donde vive | `../../references/definition-of-done.md` → `~/.agents/references/` (`Test-Path` → `False`) |

El enlace roto es la prueba de que la skill fue **escrita para vivir en
`.opencode/skills/`** y se publico desde ahi: vendorizarla lo repara
(`.opencode/skills/progreso/../../references/` = `.opencode/references/`).

## Premisa de la fila — re-medida

La fila afirma "se olvido **4 veces** (`status: active` en tareas cerradas)".
Ese numero es una **anecdota de auditoria, sin comando reproducible**. El unico
detector que puedo correr es "task file `status: active` sin fila en el Backlog",
y da **8** — pero ese detector tiene un falso positivo estructural conocido
(`FIND-774`: 108 de 126 task files no tienen fila en el plan activo, 86 %).
Conclusion honesta: la anecdota **ni se confirma ni se refuta** con un comando;
lo que si es medible es la dependencia de la ruta de fuera (contrato 6).

**Correccion del review (P2-01):** el contrato original de la fila pedia ademas
"un check detecta task files `active` cuya fila ya no esta en el Backlog". Ese
check **ya existe** — es `dev-tools/check-closure.ps1 -All` (regla (c)), y hoy
esta disparando. Lo que **no** se entrego aca es el resto del contrato de la
fila (que el cierre completo —fila fuera + avance + commit + `status:
completed`— ocurra desde **una sola invocacion**): eso queda diferido a
`H3-08` (`docs/dev/Backlog.md` §Gate de cierre de tarea), que ya existia.
Esta tarea entrega **solo la decision de politica del owner** (vendorizar); la
primera version de esta fila borro la fila del Backlog sin nombrar el remanente,
lo que borraba el criterio de aceptacion. Queda nombrado aca.

## Contrato

1. Copiar `~/.agents/skills/progreso/` a `.opencode/skills/progreso/`.
2. Adaptar toda referencia absoluta / de fuera del repo (`~`, `C:\Users`, `/home/`, `.agents`).
3. No duplicar: un solo sitio canonico, declarado.
4. Registrarla para que el pipeline y los prompts la encuentren.
5. Probar mecanicamente que corre desde el repo.
6. **Δ:** referencias a `~/.agents/skills/progreso` (`rg` en `.opencode` + `docs`) → **0** vivas.

## Impacto mapeado (Regla 0)

### Archivos leidos completos

| Archivo | Por que |
|---|---|
| `~/.agents/skills/progreso/SKILL.md` (219 L) | Es el artefacto a vendorizar; unico archivo del skill |
| `.opencode/skills/INDEX.md` (37 L) | Enumera que vive en `.opencode/skills/` — queda desactualizado al pasar de 3 a 4 |
| `.opencode/task-system/scripts/build-skills-index.mjs` | Generador del indice: define prioridad `.opencode/skills` > `.agents/skills` > `~/.agents/skills` y el campo `path` |
| `.opencode/task-system/skills-index.json` (§`progreso`) | Entrada `source:"global"` / `path:"~/.agents/skills/progreso/SKILL.md"` |
| `.opencode/SKILLS-MANIFEST.md` (tabla Pipeline/Core) | Registro curado; fila `progreso` en L486 |
| `.opencode/references/definition-of-done.md:136` | Cita la ruta de fuera como **fuente** — referencia viva |
| `docs/dev/Backlog.md` (fila FIND-777, L834) | Fila a cerrar |
| `docs/dev/tasks/FIND-774.md` | Precedente de formato + el dato de 108/126 |

### Referencias hacia dentro (quien depende de la skill)

`rg -l 'progreso' .opencode` → 44 archivos. Los que la **invocan por nombre**
(`skill progreso`) y por tanto solo necesitan que el runtime la resuelvan:
`task-system/prompts/pipeline-full.md:14,196,217,223`, `pipeline-run.md`,
`plan.md`, `task.md`, `commands/{pipeline,status,backlog,audit,harness,research,codeGraph}.md`,
`agents/{vanta-lead,vanta-docs,vanta-research}.md`,
`task-system/{C0-unified.mjs,RULES.md}`, `task-system/workflows/*.json`,
`references/{skills-engineering,definition-of-done,backlog-format,REFERENCE-SYNTHESIS}.md`.
**Ninguno necesita edicion**: `skill <nombre>` resuelve por nombre y la copia del
proyecto gana sobre la global (orden de discovery: project > global).

### Referencias hacia afuera (de que depende la skill)

- `scripts/check-avance-coverage.ps1` — existe ✅
- `scripts/validate-docs-coverage.ps1` — existe ✅
- `.opencode/references/definition-of-done.md` via `../../references/` — existe ✅ **solo** desde `.opencode/skills/progreso/`
- `docs/dev/backlog/avance/plans/reports` — arbol del repo ✅
- `campaign_memory_write` (MCP) — runtime, no ruta

### Veredicto de impacto

**Contenido, no destructivo.** Copia aditiva de 1 archivo (9 716 B). No borra la
global (el owner la usa en otros proyectos). Riesgo acotado a: (a) indice
desactualizado → se regenera; (b) enumeraciones "3 skills custom" en docs →
se actualizan **las que estan en el scope de escritura**; `AGENTS.md` queda
fuera de scope y se anota como NOTICED-BUT-NOT-TOUCHING.

**Ambiguedad resuelta:** "movela" (contrato) vs "no la borres" (⚠️). Gana la
instruccion explicita: **copiar**, y declarar `.opencode/skills/progreso/` como
canonica **para este repo**; la global sobrevive para los demas proyectos. No
hay dos versiones divergiendo *dentro* de este repo porque el orden de
resolucion (project > global) hace que la local gane siempre.

## Steps

- [x] **S1** — copiar `~/.agents/skills/progreso/SKILL.md` → `.opencode/skills/progreso/SKILL.md` (SHA-256 origen == destino)
- [x] **S2** — adaptar rutas: **0 referencias absolutas** que corregir; el unico defecto era el enlace relativo `../../references/`, **reparado por la ubicacion** (`Test-Path` False → True)
- [x] **S3** — registrar: `skills-index.json` regenerado (`source:"project"`), `.opencode/skills/INDEX.md` (3 → 4), `SKILLS-MANIFEST.md`, `definition-of-done.md:136`
- [x] **S4** — prueba mecanica: `check-avance-coverage.ps1` exit 0 (1034/1034) + `validate-docs-coverage.ps1` exit 0 (0 gaps)
- [x] **S5** — Δ: `.opencode` 2 → **0**; fila del Backlog fuera; entrada en `docs/dev/avance/activo/context-engine.md`
- [x] **S6** — cierre: ocr-review, review P2-01, commit x2 (`.opencode` + VantaDB)

## Verificacion (mecanica)

| Check | Comando | Resultado |
|---|---|---|
| Copia fiel | `Get-FileHash` origen vs destino | `7954B937…31FC48` == `7954B937…31FC48` ✅ |
| Origen intacto | `(Get-Item ~/.agents/skills/progreso/SKILL.md).Length` | 9 716 B (no borrada, no editada) ✅ |
| Rutas de fuera | `rg '~[/\\]\|C:\\Users\|/home/\|\.agents' .opencode/skills/progreso/SKILL.md` | 0 hits ✅ |
| Enlace relativo | `Test-Path .opencode/skills/progreso/../../references/definition-of-done.md` | `True` (era `False`) ✅ |
| Enlace roto en origen | `Test-Path ~/.agents/skills/progreso/../../references/...` | `False` (prueba de que fue escrita para `.opencode/skills/`) |
| Indice | `bun build-skills-index.mjs` | `source:"project"`, `.opencode/skills: 4 adoptadas` ✅ |
| Override | idem | `~/.agents/skills: 4 en sombra (override)` ✅ |
| Script de la skill (Trigger 1.F) | `pwsh scripts/check-avance-coverage.ps1` | exit 0 · **1034/1034 IDs (100.0 %)** ✅ |
| Script de la skill (Trigger 1.F) | `pwsh scripts/validate-docs-coverage.ps1` | exit 0 · **0 gaps** ✅ |
| Δ ruta de fuera (`.opencode`) | `rg -c '\.agents[/\\]skills[/\\]progreso' .opencode` | `skills-index.json:9561` + `definition-of-done.md:136` → **0** ✅ |
| Backlog | `rg -c 'FIND-777' docs/dev/Backlog.md` | 0 (fila eliminada, 6 deletions / 0 additions) ✅ |

## Como se invoca desde el repo

```
skill progreso
```

Mismo call-site que antes — los 44 archivos que la nombran
(`pipeline-full.md:14,196,217,223`, `pipeline-run.md`, `commands/*.md`,
`agents/*.md`) **no cambian una linea**: el runtime resuelve por nombre y la
copia del proyecto gana sobre la global. Editar el archivo:
`.opencode/skills/progreso/SKILL.md`.

## Incidente durante la ejecucion (declarado)

**Dos, ambos declarados.**

**(a) Backlog vaciado por regex.** El primer intento de eliminar la fila del Backlog uso `[IO.File]::Replace` con `(?s)` y **vacio el archivo a 0 bytes**. Detectado en la misma corrida y **restaurado desde git** (`git checkout -- docs/dev/Backlog.md` → 349 033 B, identico a `HEAD`, `git status` limpio). El segundo intento fue **por lineas con dry run y aserciones** (span verificado = 5 lineas, 1 sola fila `FIND-777`), y el diff final es **6 deletions / 0 additions**. Leccion: en un `.md` de 340 KB con acentos, no se usa regex dotall para borrar una fila; se indexa por linea.

**(b) El helper reporto `FAILED` con el commit ya creado.** En el commit de
`.opencode`, una de las cinco `-ExpectPresent` se tipeo con parentesis
(`vendorizada en el repo (FIND-777)`) cuando el texto real usa guion largo
(`vendorizada en el repo — FIND-777`). El helper **creo el commit `a840490`** y
despues fallo la verificacion (`exit 1`), reportando `NOT COMMITTED` — es la
clase de `FIND-780` (commit por plumbing, verificacion despues). Se **verifico
por contenido contra el blob**, no por el exit code: `git show --stat a840490`
= exactamente las 5 rutas, 0 archivos de `memory/` (WIP ajeno intacto), y
`git cat-file -p a840490:references/definition-of-done.md` contiene el texto
esperado. El commit es bueno; lo que fallo fue mi string de expectativa.

## Review (GATE — agente distinto, P2-01)

> Primera ronda: **🔴 changes-required**. Segunda ronda tras aplicar las
> correcciones: **✅ approve**.

- **Revisor:** `vanta-review` (contexto fresco, sesión `ses_de2511210ffedDaZECcpAQZHzf`; nunca la misma sesión del implementador).
- **Premisa (H-07):** **confirmada** — el revisor re-ejecutó por su cuenta la medición que sostiene la fila: `rg -n '\.agents[/\\]skills[/\\]progreso' .opencode` → **0 matches (exit 1)**, y `Get-FileHash` de ambos `SKILL.md` → `7954B937…31FC48` idéntico. También re-corrió `build-skills-index.mjs` para probar que el índice es reproducible (solo cambió `generated`) y verificó el `sha1(content)[0:12]` del campo `hash`.
- **Enfoque:** correcto, y con la alternativa evaluada —el revisor consideró symlinkar `.opencode/skills/progreso` → global y lo **rechazó con razón** (no versiona, rompe por máquina). Vendorizar + junction `.agents/skills` preserva un único punto de edición.
- **Cómo se probó:** hashes SHA-256 de ambas copias; `Test-Path` del enlace relativo en las dos ubicaciones (`False` → `True`); `rg` del path de fuera en `.opencode` y `docs`; `git diff --numstat` del Backlog (6/0) + bytes sin BOM/CRLF ni caracteres de reemplazo; diff semántico del índice (`322 = 322`, deep-changed = `['progreso']`); y los dos scripts del Trigger 1.F (exit 0 ambos).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos — todas las tablas de evidencia son de corridas reales.
  - [x] No saltarse la clarificación — "muévela" vs "no la borres" se surfacó y se resolvió con la ⚠️ explícita del owner (copiar, no borrar).
  - [ ] No declarar done sin verificar — **incumplido en la primera ronda**: S6 estaba `[x]` sin ningún commit ni sección Review. Corregido: S6 destildado hasta que existan los dos commits.
  - [x] No ignorar fallos — el incidente del Backlog (regex dotall → 0 bytes) está declarado, con la restauración desde git y el rehacer por líneas.
  - [x] No copiar sin citar — el dato de 108/126 task files cita `FIND-774`; la tabla de dominios cita `SKILL.md`.
  - [x] No dejar huérfanos los pasos — el remanente del contrato de la fila ahora **nombra** `H3-08` y `check-closure.ps1 -All`.
  - [x] No degradar chequeos en paths sensibles — no hay paths de dinero/seguridad; no se relajó ningún gate (nada de `--no-verify`).
  - [x] SKILLS_CARGADAS cubre el dominio — `incremental-implementation` + `documentation-skill` + `context-engineering` + `ponytail`.
- **Veredicto:** ✅ **approve** (ronda 2), con los siguientes hallazgos **aceptados como deuda declarada** (no bloquean):

| # | Hallazgo del revisor | Tratamiento |
|---|---|---|
| M4 | Enumeraciones "solo 3 custom" obsoletas dentro de `.opencode/` | **Corregidas las de `SKILLS-MANIFEST.md`** (L3, L10, L604 — dentro del scope de escritura). Las de `task-system/prompts/*.md` y `references/skills-engineering.md` quedan **NOTICED BUT NOT TOUCHING**: fuera del scope declarado por el owner; sin rotura funcional porque la junction hace que la tabla siga aterrizando en la copia del proyecto. |
| M5 | Contrato completo de la fila no entregado | Nombrado arriba: detector ya existe (`check-closure.ps1 -All`); remanente → `H3-08`. |
| M6 | Nada detecta drift entre la copia del proyecto y la global | **Mitigado sin código**: comando de verificación de drift documentado en `.opencode/skills/INDEX.md`. Un assert automático en `build-skills-index.mjs` queda como deuda (fuera de scope). |
| M7 | `status: stable` en una tarea que cierra | Aceptado: el gate lee `**Estado:**`, no el frontmatter; vocabulario dominante respetado en el campo que sí se lee. |
| L1 | Nit de legibilidad `9561`:1 | Corregido a `skills-index.json:9561`. |

- **Commits:** `.opencode` `a840490` · VantaDB: este mismo commit.

## Invariantes

- No borrar ni editar `~/.agents/skills/progreso/` (otros proyectos del owner la usan).
- No tocar `src/**`, `.github/**`, `dev-tools/**`, `docs/dev/plans/**` — hay un carril de CI en vuelo.
- No commitear WIP ajeno (`budget.json`, `H-02-H-07.md`, docs de verificacion ajenos).
- Un solo sitio canonico: sin copia parcial en `.opencode/`.
