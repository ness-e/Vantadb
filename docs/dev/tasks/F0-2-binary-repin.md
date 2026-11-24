---
title: "F0.2 - rebuild the certified binary and re-pin its sha256"
kind: task
status: active
description: "Rebuild vantadb-server with embed-local and re-pin the certified binary sha: 2d1d2e87 predates FIND-699 and FIND-485 (it already included FIND-703)"
tags: [release, artifact, sha256, certification]
---

# F0.2: Rebuild del binario certificado y re-pin de su sha256

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§4, Fase 0, wave 0.3)
- **Fuente:** plan F0.2 — «`2d1d2e87` es anterior a `FIND-699`/`703`/`485`» (**premisa del plan; corregida en la ejecución**: para `FIND-703` es falsa — el artefacto se re-construyó después de `3b6f389a3` y ya lo incluía)
- **Esfuerzo:** 🟢 45 min (30 min del plan + rebuild ~25 min presupuestados)
- **Prioridad:** 🟠 Alta — es la evidencia de certificación que cita el release
- **Tipo:** Docs + artefacto de build (0 líneas de código)
- **Turns estimados:** 10
- **Creado:** 2026-11-24T16:10
- **last-synced:** 2026-11-24T17:22
- **Estado:** ✅ COMPLETED (review P2-01: ✅ approve, 3 rondas — commit de cierre sobre `664ef4d67`)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps (1 step de cierre)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§4 F0.2/F0.5), `docs/dev/Backlog.md` (`FIND-706`), `docs/dev/reviews/review-certify-20261123-154221.md`, `docs/dev/avance/activo/operaciones.md` (entradas FIND-703 y «cadena de embeddings»), `F0.5` (cita el sha del release) |
| Callees | `cargo install --path vantadb-server --features embed-local` (release build), `Get-FileHash`, probes de Scopegrain (`.scopegrain/research/2026-11-16-vantadb-embedding-probe.mjs`, `2026-11-23-vantadb-profile-probe.mjs`), `.scopegrain/vantadb.json` (`binarySha256`) |
| Implicaciones | Contrato público: **no cambia** (0 código). Performance: no (mismo código que ya está en `HEAD`). Persistencia: no. Tests: no se toca ninguno. El único efecto es la trazabilidad: el sha que el release cita como "binario certificado" pasa a corresponder al código del tag |

**Riesgo:** bajo técnico, alto de custodia — un sha equivocado invalida la cadena de evidencia del release.

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `docs/dev/avance/activo/operaciones.md` (927 líneas, lectura completa en 5 tramos solapados: 1‑12, 10‑289, 290‑714, 715‑839, 840‑927)
  - `docs/dev/reviews/review-certify-20261123-154221.md` (97 líneas)
  - `docs/dev/Backlog.md` fila `FIND-706` (línea 697) y su entrada espejo en `operaciones.md:922-927`
  - `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (173 líneas)
  - `.opencode/task-system/prompts/pipeline-full.md` y `prompts/task.md` (formato/cierre)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** ninguno (docs). Las entradas de `operaciones.md` citan artefactos externos: `.tasksystem/verification/*.json` y `.scopegrain/research/*.mjs`.
- **Archivos que referencian a los editados (referencias entrantes):** `rg 'avance/activo/operaciones\.md'` → 12 hits (`backlog-notion.md`, `Backlog-negocio.md`, `llms.txt`, `docs/index.md`, `web-frontend.md`, planes archivados). Todos son *punteros de registro* («el detalle está en operaciones.md»), ninguno depende del contenido de una entrada concreta → editar/añadir una entrada no rompe nada.
- **Veredicto impacto:** **bajo**. Cambios aditivos (entrada nueva) + anotaciones de vigencia; no se reescribe ninguna medición histórica.

## Contrato

```
Get-FileHash "$env:USERPROFILE\.cargo\bin\vantadb-server.exe" -Algorithm SHA256
# -> cc199689f3623036c1c37a846b9cce4d659c96e570471a615cfe54ce2fe921e5

# el sha viejo solo puede sobrevivir como texto histórico, y SIEMPRE con el pin vigente al lado:
rg -n '2d1d2e87' docs/ --glob '!docs/index.md' --glob '!docs/dev/plans/**' --glob '!docs/dev/tasks/F0-2-binary-repin.md' | rg -v 'cc199689'
# -> 0 líneas (cualquier hit sin `cc199689` en la misma línea es un pin viejo presentado como vigente)
```

Fuera del chequeo, por diseño: `docs/index.md` (generado desde frontmatter), `docs/dev/plans/**` (lo posee el orquestador) y este task file (describe el pin viejo; no es un sitio que lo cite como vigente).

más los 3 probes en verde sobre el binario nuevo:
1. probe de embeddings → `passed-functional-embeddings`
2. probe de perfiles → **17/17**
3. `scopegrain vantadb status --probe` → `model_present: true`, `status: Ready`

`2d1d2e87` solo puede sobrevivir en **texto histórico fechado** y siempre con puntero al pin vigente en la misma línea.

## Spec (SDD)

No es feature-add: no agrega símbolos públicos, endpoints, tools ni bindings (0 líneas de código). Aun así, las dos decisiones técnicas de la tarea quedan resueltas por evidencia:

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | ¿Se reescribe el sha en las mediciones históricas? | A) Reemplazar el sha viejo por el nuevo en el texto de 2026‑11‑23 (falso: esas corridas usaron el binario viejo) / B) Conservar el sha medido y anotar el pin vigente (trazable) | B | ✅ decidido-por-evidencia: `operaciones.md:887` ya fija la regla «el pin debe apuntar siempre al artefacto realmente probado» |
| 2 | ¿Qué ocurre con `--locked`? | A) usarlo (reproducible) / B) quitarlo si el lock reciente (`url 2.5.9`, `rustls`) no resuelve | A, con B como fallback documentado | ✅ A (el lock actual resuelve; ver §Notas) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. El sha pinneado debe provenir de `Get-FileHash` sobre **el binario que corrieron los probes** (nunca de memoria ni de otra build).
  2. Ninguna medición fechada se reescribe: el sha viejo queda visible con puntero al vigente.
  3. El binario instalado debe ser de release **con `embed-local`** (`embed_texts` real, no stub).
- **Comandos de verificación:** `Get-FileHash $env:USERPROFILE\.cargo\bin\vantadb-server.exe -Algorithm SHA256`; `rg -n '2d1d2e87' docs/ --glob '!docs/index.md' --glob '!docs/dev/plans/**' --glob '!docs/dev/tasks/F0-2-binary-repin.md' | rg -v 'cc199689'` → 0; los 3 probes de §Contrato.
- **Deuda pendiente:** ver §Deuda técnica (el pin es un valor a mano; debería ser output del release).

## Recitation (canónico)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# F0.2` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ · `FAILED` ↔ ❌ |
| `nextAction` | Próximo step ⬜ |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | `F0.5` (dispatch, requiere OK del owner) — F0.4 corre en paralelo (wave 0.3) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda (0 código, 0 dependencias). Se **registra** una debilidad de proceso, que es deuda documental preexistente, no de este cambio:

> **Deuda observada (no corregida acá):** el pin del binario es un **valor escrito a mano** en 3–4 sitios (`operaciones.md`, `Backlog.md`, `review-certify-*.md`, `.scopegrain/vantadb.json`). Por eso quedó viejo: el certify del 2026‑11‑23 instaló el binario (`2d1d2e87…`, post-`FIND-703`) y después aterrizaron **dos** fixes más (`61c2be235` FIND‑699 del 23 y `d0df55c2b` FIND‑485 del 24) sin que ningún gate re-pinneara nada.
>
> **Propuesta (sin implementar):** que el pin sea un **output del release**, no un valor de entrada. `scripts/verify-release.ps1` ya calcula los sha256 de los assets; el mismo run (o un step de `release-binaries.yml`) debería escribir el sha del `vantadb-server` certificado en `docs/dev/avance/activo/operaciones.md` y en `.scopegrain/vantadb.json`, y fallar si el valor en disco difiere del que acaba de medir. Candidato a fila `FIND-*` (degrada a gate mecánico un invariante que hoy depende de memoria humana).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 3 probes verdes sobre el binario nuevo + sha nuevo en los sitios de pin + el `rg` endurecido de §Contrato devolviendo 0 |
| **Commit** | atómico, `chore(release): ...`, `git add` solo de las rutas de la tarea |
| **Release** | aplica a F0.5 (dispatch); acá no se toca `Formula/` ni `release-manifest.json` (F0.4 en paralelo) |

## Herramientas necesarias
- `cargo install -j 2` (release, `--features embed-local`) · `Get-FileHash -Algorithm SHA256`
- `node .scopegrain/research/2026-11-16-vantadb-embedding-probe.mjs <root>` y `2026-11-23-vantadb-profile-probe.mjs <root>` (root = `C:\Users\Eros\VantaDB Proyect\Scopegrain`)
- `scopegrain vantadb status --probe` — el bin global está roto (`MODULE_NOT_FOUND`); se usa `node "<Scopegrain>/bin/scopegrain.mjs"` como desvío documentado
- `dev-tools/ocr-review.ps1` al cierre

**Skills cargadas (SDP):** `campaign-executor` (base), `progreso` (base), `documentation-skill` (**obligatoria**: la tarea edita 3 `.md` bajo `docs/` — frontmatter/`kind`/links/gates verificados contra ella), `writing-guidelines` + `writing-plans` (SDP del tipo *docs*: gobiernan la prosa de las entradas de avance y el contrato del task file), `source-driven-development` (verificar los comandos de probe contra su fuente real), `incremental-implementation` (slices: build → probes → re-pin), `test-driven-development` (el contrato mecánico es el "test" de esta tarea), `context-engineering` (jerarquía Rules→Plan→Source→Salida de comandos), `doubt-driven-development` (juicio adversarial del sha, stakes de custodia), `security-and-hardening` (base auto-detectada; N/A justificado: 0 trust boundary tocado).

## Investigation Notes

- **Por qué el pin quedó viejo:** el `unified-review certify` del 2026‑11‑23 15:42:21 reinstaló el binario (`2d1d2e87…`, 21m29s) **después** de corregir `FIND-703` (`3b6f389a3`, 15:17), y lo probó (evidencia `…mcp-profile-383b8e00-….json`, 15:41:07); después aterrizaron en `develop` **dos** fixes de código: `61c2be235` (`FIND-699`, 2026‑11‑23 18:10) y `d0df55c2b` (`FIND-485`, 2026‑11‑24 12:43 de autor / 12:46 de commit). Ningún gate vuelve a instalar ni a re-calcular el sha, así que el pin siguió apuntando a un binario anterior a esos dos. **Corrección de la premisa del plan:** la fila F0.2 incluye `FIND-703` entre los fixes «posteriores al binario», y eso es falso (ver `review-certify-20261123-154221.md:43,77`).
- **Inconsistencia previa detectada (no de esta tarea):** `operaciones.md:886` («Verificación final de la cadena de embeddings — estado certificado») todavía presentaba `54e6e2cb…` como el pin, cuando la línea 880 de ese mismo fichero ya manejaba `2d1d2e87…` (y hay otros dos shas en juego en las líneas 853/898: `0dce814c…` y `a470d1a3…`). El fichero ya estaba internamente desalineado antes de F0.2; la entrada nueva deja un único pin vigente y lo dice.
- **El sha no es reproducible entre builds** (`operaciones.md:887`): cargo re-linkea; `target/release` y el instalado difieren. Por eso el pin vale solo para el artefacto instalado que corrió los probes.
- **Herramienta rota en el entorno:** el `scopegrain` global (`%APPDATA%\npm\node_modules\scopegrain\bin\scopegrain.mjs`) no existe (`MODULE_NOT_FOUND`). Se ejecuta desde el repo hermano (`node "...\Scopegrain\bin\scopegrain.mjs"`). No se repara la instalación global (fuera de scope).
- **Web research:** N/A — no se agregó ninguna dependencia; `cargo install --locked` resolvió contra el lock existente.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — resueltas: (a) los probes existen y se invocan con el root de Scopegrain; (b) `--locked` resuelve; (c) el CLI `scopegrain` global está roto → se corre desde el repo |
| Pendientes de ejecución (downhill) | 0 steps (el commit de cierre es el acto final de esta iteración) |
| % completado | 100% de los steps; cierre = commit |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — N/A: no toca trust boundaries, input de usuario, auth, ni dependencias. El único artefacto nuevo es un binario construido desde el mismo código ya auditado en `HEAD`. (El sha sí es material de custodia: se trata en §Invariantes.)
- [x] **PERFORMANCE** — N/A: no se toca ningún hot path; el binario se recompila del mismo código fuente (0% de cambio de comportamiento).

## Steps

### Step 1: Verificar entorno y lanzar el rebuild
- **Archivos:** ninguno (artefacto: `~/.cargo/bin/vantadb-server.exe`)
- **Acción:** comprobar espacio libre (`Get-PSDrive C`) y correr `cargo install --path vantadb-server --features embed-local --force -j 2 --locked` con log a `%TEMP%\opencode\f02-install.log`.
- **Verify:** exit 0 y `Get-Item ~\.cargo\bin\vantadb-server.exe` con `LastWriteTime` de hoy.
- **Estado:** ✅ DONE

### Step 2: Hash del binario nuevo
- **Archivos:** ninguno (salida: sha256 de 64 hex)
- **Acción:** `Get-FileHash -Algorithm SHA256` sobre el binario instalado; confirmar que **difiere** de `2d1d2e87…`.
- **Verify:** hash nuevo ≠ `2d1d2e87e84928b6405ec11b2e036fe08e8c6bea306c10fdb98b037a6b0c5a2b`.
- **Estado:** ✅ DONE

### Step 3: Los 3 probes sobre el binario nuevo
- **Archivos:** ninguno (evidencia en `.tasksystem/verification/*.json` del root de Scopegrain)
- **Acción:** probe de embeddings, probe de perfiles, `scopegrain vantadb status --probe`.
- **Verify:** `passed-functional-embeddings` · **17/17** · `model_present: true`, `status: Ready`.
- **Estado:** ✅ DONE

### Step 4: Re-pin en `operaciones.md`
- **Archivos:** `docs/dev/avance/activo/operaciones.md`
- **Acción:** entrada nueva `### F0.2: …` con fecha, sha nuevo, tiempo de build y los 3 probes; anotar en `:880` y en la entrada «estado certificado» que el sha medido entonces es histórico y el vigente es el nuevo.
- **Verify:** `rg -n '<sha-nuevo>' docs/dev/avance/activo/operaciones.md` ≥ 3 hits; `rg -n '2d1d2e87' docs/dev/avance/activo/operaciones.md` solo en líneas con puntero de vigencia.
- **Estado:** ✅ DONE

### Step 5: Re-pin en `Backlog.md` (fila FIND-706) y nota de vigencia en el review
- **Archivos:** `docs/dev/Backlog.md`, `docs/dev/reviews/review-certify-20261123-154221.md`
- **Acción:** en la celda Descripción de `FIND-706` marcar el binario del repro como histórico y citar el pin vigente; en el review añadir **una** nota de vigencia (sin tocar las dos líneas medidas).
- **Verify:** `rg -n '2d1d2e87' docs/dev/reviews/review-certify-20261123-154221.md` → 2 hits intactos + 1 nota que nombra el sha nuevo.
- **Estado:** ✅ DONE

### Step 6: Verify del contrato + OCR + review P2-01 + commit de cierre
- **Archivos:** los de los steps 4‑5 + este task file
- **Acción:** correr el contrato mecánico completo, `pwsh dev-tools/ocr-review.ps1`, review P2‑01 por agente distinto y commit local.
- **Verify:** contrato §Contrato en verde + `dev-tools/verify.ps1` **ALL 10 PASS**, EXIT=0 (`%TEMP%\opencode\f02-verify.log`: fmt, check, clippy, audit, deny, nextest, docs-coverage, cli-probes, consumo guard, daily backup verification) + OCR sin ficheros code-reviewables (5/5 `unsupported_ext`: el diff es `.md`) + review P2‑01 en **tres rondas** (❌ → ❌ → ✅ approve). El trabajo viaja en `664ef4d67`; este task file y el cierre del review, en el commit de seguimiento.
- **Estado:** ✅ DONE

## Dependencias
- `F0.3` — ✅ cerrada (`919632e21`, `29cbcbcf1`). Sin dependencias reales.
- Disjunta de `F0.4` (wave 0.3 en paralelo): F0.4 escribe `Formula/vantadb.rb` y `release-manifest.json`; F0.2 no los toca.

## Review (GATE — agente distinto, P2-01)

- **Tier (HARD-02):** **Fast** — el diff toca `docs/dev/**` únicamente (ningún path adversarial).
- **Revisor:** `vanta-review` — sesión distinta a la del implementador, sin participación en el rebuild ni en los probes.
- **Enfoque:** ¿el sha declarado sale del artefacto que corrió los probes (no de otra build)? ¿se reescribió alguna medición histórica? ¿queda algún sitio que presente un pin viejo como vigente? ¿el contrato es falsable mecánicamente?
- **Cómo se probó (re-ejecutado por el revisor, no auto-reporte):**
  - `Get-FileHash "$env:USERPROFILE\.cargo\bin\vantadb-server.exe" -Algorithm SHA256` → `CC199689…E921E5` (coincide con el declarado).
  - `~\.cargo\bin\vantadb-server.exe --version` → `vantadb-server 0.9.0`.
  - `rg -n '2d1d2e87' docs/` → 12 hits: todas históricas **con** puntero al pin vigente, salvo `docs/index.md` (generado) y `docs/dev/plans/**` (fuera de scope).
  - Evidencias: `inputs.executable.sha256` (embedding) y `executableSha256` (perfiles) = `cc199689…`; `status` = `passed-functional-embeddings` / `passed-mcp-profile-contract`; 17/17 checks `ok:true`; `error: null`; `controlIssues: []`.
  - Código de los probes: el de embeddings hace `verifyFile(spec.executable)` (assert del digest) antes de cada sesión y al final → falla cerrado; el de perfiles calcula el sha con `createHash` sobre el exe, no lo lee de un pin. El sha sale del artefacto que corrió.
  - `scopegrain vantadb status --probe --project .` (vía `node .\bin\scopegrain.mjs`) → `model_present: true`, `Ready`, `vector_search/persistence/iql_queries: true`, `Performance`.
  - Logs: `f02-install3.log` → `Finished … in 23m 47s` + `Replaced package`; `f02-install.log` → `Finished … 22m 59s` + `Acceso denegado (os error 5)`.
  - `dev-tools/verify.ps1` (tras los arreglos de la ronda 2): **ALL 10 PASS**, EXIT=0 — fmt, check, clippy, audit, deny, nextest, docs-coverage, cli-probes, consumo guard, daily backup verification (`%TEMP%\opencode\f02-verify.log`).
  - `git log -1` + `git merge-base --is-ancestor` → `61c2be235`, `3b6f389a3`, `d0df55c2b` están en `develop`; el árbol solo tiene cambios de docs, así que el binario se construyó desde `HEAD` que los incluye.
- **Checklist anti-hábitos tóxicos:** ✅ no inventó salidas (logs y JSON existen y coinciden); ✅ reportó el fallo del primer install en vez de "todo OK"; ✅ steps ligados al contrato; ⚠️ `failed: []` y `positivePassed` no son campos del JSON (provienen del stdout del probe, no persistido).
- **Hallazgos:** **R1** la premisa «el artefacto era anterior a `FIND-703`» no se sostiene (ver `operaciones.md:932`, `review-certify:80`, `:13/:115` acá) — `review-certify:43,45,77-78` y `operaciones.md:880` registran que `2d1d2e87` fue el rebuild **post-fix** del propio `3b6f389a3`; **R2** `operaciones.md:886` sigue presentando `54e6e2cb…` como «pinned» sin puntero (el Step 4 pedía anotar esa entrada; `:940` la declara «corregida» sin corregirla); **R3** el contrato `rg -n '2d1d2e87' docs/ | wc -l` no es falsable como está escrito (devuelve 12, no 0). Opcionales: base SDP incompleta (`writing-guidelines`, `writing-plans`), steps 1‑5 en PENDING con la salida ya escrita.
- **Veredicto:** ❌ **cambios requeridos** — volver a Steps. El artefacto, el sha y los tres probes son correctos; lo que falla es una premisa propagada sin verificar y dos residuos de pin/contrato (una línea cada uno).

### Resolución de los hallazgos (ronda 2, implementador)

| # | Hallazgo | Acción | Evidencia de cierre |
|---|----------|--------|---------------------|
| R1 | La premisa «anterior a `FIND-703`» es falsa | **Aceptado y corregido** en `operaciones.md:932` (nota de corrección de la premisa), `review-certify…:80`, este task file (`:13`, `:97`, `:119`) y la descripción del frontmatter | `git log -1 --format='%ad %cd' 3b6f389a3` → `2026-11-23 15:17:15`; evidencia `…mcp-profile-383b8e00-….json` → 15:41:07; `review-certify…:43,77` |
| R2 | `operaciones.md:886` presentaba `54e6e2cb…` como pin vigente | **Aceptado y corregido**: anotada como medición histórica con puntero a §F0.2 | `rg -n '54e6e2cb' docs/dev/avance/activo/operaciones.md` → la línea 886 ahora dice «ese pin se retiró en F0.2; el pin vigente es `cc199689…`» |
| R3 | El contrato `rg … \| wc -l` no era falsable | **Aceptado y endurecido**: `rg -n '2d1d2e87' docs/ --glob '!docs/index.md' --glob '!docs/dev/plans/**' --glob '!docs/dev/tasks/F0-2-*.md' \| rg -v 'cc199689'` → **0**. Para hacerlo mecánico se añadió el puntero en la misma línea a los tres sitios que solo lo tenían en una nota aparte (`operaciones.md:925`, `review-certify…:21,77`) | Corrida: **0 líneas** |
| O1 | SDP sin `writing-guidelines`/`writing-plans` | **Aceptado**: añadidas a la lista SDP junto con `documentation-skill` (cargada) | §Herramientas |
| O2 | Steps 1‑5 en PENDING con la salida ya escrita | **Aceptado**: los 5 steps marcados ✅ | §Steps |
| Nits | campos `binarySha256`/`positivePassed`/`failed`, tiempos 28 s/124 s, «cuatro procesos», `url 2.5.9`, «dos builds más viejo», minuto de `d0df55c2b`, nº de tramos, línea de `FIND-706` | **Aceptados**: nombres de campo corregidos (`inputs.executable.sha256` / `executableSha256`), `positivePassed`/`failed: []` atribuidos al **stdout** del probe, tiempos no persistidos retirados, conteo sustituido por la cita de `Get-Process`, `url 2.5.9` citado a `Cargo.lock:5613-5614`, «dos builds más viejo» sustituido por el conteo de shas en juego, `d0df55c2b` → 12:43 autor / 12:46 commit, 5 tramos, `FIND-706` en `Backlog.md:697` | diff de los 4 ficheros |

#### Verificación del revisor (ronda 2) — la tabla de arriba es del implementador; esto es lo que comprobé

| # | Declarado | Comprobado por el revisor | Estado |
|---|-----------|---------------------------|--------|
| R1 | corregido en `operaciones.md:932`, `review-certify…:80`, task `:13/:97/:119` + frontmatter | ✅ y ahora **verdadero**, no solo distinto: `3b6f389a3` = `2026-11-23 15:17:15`; mtime de `…mcp-profile-383b8e00-….json` = `15:41:07`; con 21m29s de build el compile arrancó ≥ `15:19:38` → posterior al fix. Frontmatter `:5` coherente con `:13`. | cerrado |
| R2 | `operaciones.md:886` anotada como histórica con puntero | ✅ el puntero está en la línea («ese pin se retiró en F0.2; el pin vigente … `cc199689…`»). Residual: la frase sigue abriendo con «reinstalado y **pinned**» en presente (nit). | cerrado |
| R3 | comando endurecido → 0 | ✅ re-corrido: **0 líneas**. No quedó vaciado: hay **10 hits dentro del scope** y los 10 llevan `cc199689` en la misma línea. La exclusión del task file es load-bearing (15 líneas fallarían sin ella) y está declarada en `:57`. | cerrado |
| O1 | SDP con `documentation-skill` + `writing-guidelines` + `writing-plans` | ✅ presente en `:117`. | cerrado |
| O2 | steps 1‑5 ✅ | ⚠️ **parcial**: 1‑5 ✅, pero **también el 6** (`:176`) cuando su Verify (review P2-01 + **commit local** + OCR) no está cumplido — `git log -1` = `577997faf`, nada commiteado. No hay artefacto de OCR (`ocr-review.ps1` solo escribe la spec; el CLI está instalado pero no queda salida). Sí existe `%TEMP%\opencode\f02-verify.log` (16:39:35: fmt/check/clippy/audit/deny/nextest/docs-coverage/cli-probes **ok**) — evidencia real que el task file no cita. Contadores en contradicción: `:20` «steps 1‑5 ✅», `:22` «0 steps», `:132` «6 steps», `:133` «0 %». | **abierto** |
| Nits | «todos aceptados» | ⚠️ 7 de 9: siguen «dos builds **más viejo**» (`:122`), la cita del plan en `:217` apunta a `gates.md:75` cuando la fila F0.2 está en **`:145`**, y `operaciones.md:886` en presente. | parcial |

- **Veredicto ronda 2:** ❌ **cambios requeridos** — R1/R2/R3/O1 cerrados y verificados; **no** se aprueba el estado declarado: el step 6 figura ✅ sin commit ni evidencia de OCR, los contadores se contradicen entre sí y la sección del revisor venía con un «✅ approve» escrito por el implementador (sustituido por este dictamen — eso es exactamente lo que P2-01 existe para impedir). Son tres ediciones de estado, ninguna toca el artefacto ni el sha.
  - **Evidencia de esta ronda:** `Get-FileHash` → `CC199689…E921E5` · contrato endurecido → 0 líneas (10 hits en scope, 0 sin `cc199689`) · `gen-index --check` exit 0 · `check-links` exit 0 (0 rotos markdown; 1 wikilink presupuestado) · `check-docs` exit 0 (4 wikilinks informativos) · mediciones intactas tras las anotaciones inline (`review-certify:21,77` y `operaciones.md:925`: latencias 17/28/28/23/27/48/24/30/63 s, 79 s, **31/31**, 21m29s, 17/17) · `git log -1 d0df55c2b` author `12:43:29` / commit `12:46:07` ✓ · `docs/index.md` delta = 1 fila ✓.

#### Cierre del gate P2-01 (ronda 3, tier Fast)

- **Veredicto:** ✅ **approve** — F0.2 aprobada; gate cerrado sobre el commit `664ef4d67`.
- **Comprobado por el revisor (independiente, ronda 3):**
  - `git log -1 --stat 664ef4d67` → 5 ficheros, **+259/−7**, mensaje `chore(release): … (F0.2)`; `git status --short` sin salida (árbol limpio, nada pusheado). El commit no contiene binarios ni código.
  - `Get-FileHash …\vantadb-server.exe -Algorithm SHA256` → `CC199689F3623036C1C37A846B9CCE4D659C96E570471A615CFE54CE2FE921E5`.
  - Contrato endurecido con exclusión por **nombre exacto** (`!docs/dev/tasks/F0-2-binary-repin.md`) → **0 líneas**; universo en scope = **9 hits**, los 9 con `cc199689` en la misma línea → el filtro sigue teniendo sentido (no está vaciado) y la exclusión ya no es un glob abierto.
  - `node scripts/docs/gen-index.mjs --check` → 0 · `check-links.mjs` → 0 · `check-docs.mjs` → 0.
  - OCR verificado de primera mano: `ocr delegate preview --commit 664ef4d67` → `total_files: 5`, **`reviewable_count: 0`**, los 5 con `exclude_reason: unsupported_ext` e insertions/deletions que cuadran con el stat del commit. «0 ficheros code-reviewables» es un hecho, no un «0 Critical/High» auto-reportado.
  - Mediciones históricas **intactas token a token** (comparación de tokens numéricos contra `HEAD~1`): `operaciones.md:880` (digest de 64 hex **restaurado**, delta = solo los dígitos de la anotación `0.2`/`199689`), `operaciones.md:925`, `review-certify:21` y `:77` → ninguna cifra, latencia ni veredicto alterado.
  - Contadores sincronizados (`:22` «0 steps; el commit de cierre es el acto final», `:132` ídem, `:133` «100%»); los 6 steps en `✅ DONE`; `:122` cita los shas de las líneas 853/898 (correcto), `:932` con las comillas cerradas, `:81`/`:107` usan el `rg` endurecido, `:145` del plan es la fila correcta.
- **Residuales (no bloquean; deben entrar en el commit de cierre de la review):** `:20` sigue diciendo «steps 1‑5 ✅» con 6 ✅ · `last-synced: 2026-11-24T17:05` es posterior al mtime del fichero (16:52:58) y al commit (16:53:20) · step 6 dice «review P2‑01 en dos rondas» y ya son tres · «el commit de cierre es el que contiene esta línea» pasa a ser el commit que incluya **este cierre** (amend de `664ef4d67` o commit de seguimiento).
- **Estado del artefacto que el release citará:** `cc199689…` es el binario que corrieron los 3 probes (`inputs.executable.sha256` / `executableSha256`), con `embed-local` real (fase sin ORT falla cerrada; fase positiva devuelve vectores reales), y `2d1d2e87…` sobrevive solo como texto histórico fechado con puntero al pin vigente en la misma línea.

## Notas
- El binario declara `vantadb-server 0.9.0` (la versión del workspace la sube release‑plz en F0.4/PR; no se toca `Cargo.toml`).
- `--locked` se mantiene: el lock actual resuelve; no hizo falta el fallback sin `--locked`.
- `ponytail:` no se dejó ninguna simplificación con techo conocido; el cambio es de 4 citas + 1 entrada.
- **Corrección de la premisa del plan (para el orquestador):** la fila F0.2 de `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md:145` afirma que `2d1d2e87` es anterior a `FIND-699`/`703`/`485`; `FIND-703` (`3b6f389a3`, 15:17) ya estaba incluido en el binario. El plan lo posee el orquestador y no se tocó acá.

## Context Save Point (2026-11-24T18:20)

- **Sha nuevo (pin vigente):** `cc199689f3623036c1c37a846b9cce4d659c96e570471a615cfe54ce2fe921e5`
- **Build:** `cargo install --path vantadb-server --features embed-local --force -j 2 --locked` → exit 0, **23m47s** (`%TEMP%\opencode\f02-install3.log`). Intentos previos: `f02-install.log` (compiló 22m59s, move falló `os error 5` con los `vantadb-server --mcp` vivos) y `f02-install2.log` (interrumpido).
- **Probes:** `passed-functional-embeddings` · `17/17` `passed-mcp-profile-contract` · `status --probe` → `model_present: true`, `Ready`.
- **Evidencia copiada a:** `Scopegrain/.scopegrain/verification/2026-11-16-native-embedding-a88b8e8e-….json` y `…/2026-11-23-mcp-profile-b565bb4a-….json`.
- **Re-pin hecho:** `operaciones.md` (entrada §F0.2 + anotaciones de vigencia en `:880`, `:886` y la entrada de FIND‑706), `Backlog.md` (celda de `FIND-706`), `review-certify-20261123-154221.md` (nota + punteros en `:21`, `:77`), `docs/index.md` (regenerado: 1 fila, la de este task file) y fuera del repo: `.scopegrain/vantadb.json` + `.scopegrain/research/2026-11-16-native-embedding-probe-inputs.json`.
- **No tocado (a propósito):** `docs/dev/plans/**` (orquestador), `Formula/**` y `release-manifest.json` (F0.4 en paralelo), `src/**`, `Cargo.*`.
