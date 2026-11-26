---
title: "F0.4 — prepare the 0.10.0 version bump outside Cargo.toml"
kind: task
status: active
description: "Version-independent verdict: the manifest bump is gated by equality with Cargo.toml and already automated on the release-plz branch; the formula bump is bonded to its 4 SHA256. Zero files bumped — registered as evidence plus a plan correction."
tags: [release, versioning, homebrew, ci]
---

# F0.4: prepare the bump of `Formula/vantadb.rb` + `release-manifest.json` to 0.10.0

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§4, Fase 0)
- **Fuente:** plan file task F0.4; gate Justificación del plan ("la fórmula y el manifest no los mueve release-plz")
- **Esfuerzo:** 🟢 30 min
- **Prioridad:** 🟠 (no bloquea F0.5 — ver §Dependencias)
- **Tipo:** Mixto (release / CI / docs)
- **Turns estimados:** 8
- **Creado:** 2026-11-24T00:00
- **last-synced:** 2026-11-24T00:00
- **Estado:** ✅ COMPLETED (veredicto de **no-ejecución** + evidencia — ver §Contrato)
- **Incógnitas (uphill):** 0 abiertas (el veredicto es independiente de la versión objetivo)
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Usuarios que instalan la fórmula desde un tap/clone (ver nota: `brew install vantadb` **no resuelve** — la fórmula está en `Formula/`, no en un tap; deuda ya registrada en `docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:349`). `scripts/verify-release.ps1:65-70` lee `release-manifest.json` como fuente única de la lista de assets/registries a verificar |
| Callees | `Formula/vantadb.rb` → assets `vantadb-<target>.tar.gz` + sidecars `.sha256` de `.github/workflows/release-binaries.yml` (5 targets, ambos macOS incluidos: `release-binaries.yml:63-78`). `release-manifest.json` → 15 registries (crates.io 1, PyPI 11, npm 3) + 5 binarios + 4 wheels |
| Implicaciones | No cambia API pública ni comportamiento del motor. Un `version` sin sus 4 `sha256` correspondientes rompe la instalación por checksum. Un `version`/`tag` del manifest desalineado con `Cargo.toml` pone rojo el check `Check npm package versions match workspace` |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `Formula/vantadb.rb` (59 L) · `release-manifest.json` (90 L) · `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (173 L) · `docs/dev/workflow/PUBLISH.md` (507 L) · `scripts/update-homebrew-formula.sh` (31 L) · `.github/workflows/release-verify.yml` (head 24 L) · `.github/workflows/ci-gate.yml` (36-63) · `.github/workflows/release-binaries.yml` (63-78) · `.github/workflows/docs-gates.yml` (20-65 filtros; job 149-183) · `.github/workflows/release.yml` (115-164) · `scripts/docs/sync-workspace-version.mjs` (380-529)
- **Archivos referenciados hacia dentro (imports/dependencias):** `release-manifest.json` ← `scripts/verify-release.ps1:67` · `scripts/docs/sync-workspace-version.mjs:402-408` · `.github/workflows/{ci-rust,docs-gates}.yml` (path filters) · `Formula/vantadb.rb` ← solo `scripts/update-homebrew-formula.sh` (roto, ver D2)
- **Archivos que referencian a los editados (referencias entrantes):** `rg -i "Formula" .github/ scripts/ dev-tools/ justfile .githooks/` → **ningún gate**. Hits: `dev-tools/scripts/collect_code.ps1:61` (exclusión de directorio), `scripts/clean-extensions.ps1:63-64` (IDs de extensiones VS Code, falso positivo), `scripts/update-homebrew-formula.sh` (helper manual roto)
- **Veredicto impacto:** **alto si se edita, nulo si no se edita.** Editar `release-manifest.json` en `develop` pone rojo un check del set REQUIRED; editar `version` de la fórmula sin sus 4 SHA256 rompe la instalación de la fórmula. **Veredicto: no editar ninguno de los dos en esta tarea.**

## Contrato

> El contrato original del plan (`rg '0\.10\.0' Formula/vantadb.rb release-manifest.json` en ambos) **no es ejecutable en `develop`**. Contrato efectivo, **version-agnóstico** (vale para 0.9.x o 0.10.x — ver §Incógnitas, el veredicto no depende de la versión objetivo):

```powershell
# 1. El manifest está DENTRO del gate de igualdad y hoy coincide con el workspace.
node scripts/docs/sync-workspace-version.mjs --check
#   esperado: exit 0 · una línea `ok release-manifest.json: <v> (v<v>)` · sin ninguna línea `DRIFT`

# 2. La fórmula no está en ningún gate.
rg -i "Formula" .github/ scripts/ dev-tools/ justfile .githooks/
#   esperado: exit 1 (sin match en superficies de gate) o solo hits que no son gates
#   (collect_code.ps1:61 · clean-extensions.ps1:63-64 · update-homebrew-formula.sh)
```

**Completado =** ambos comandos dan la salida esperada **y** `git status --short Formula/ release-manifest.json Cargo.toml Cargo.lock` está vacío.

**Experimento documentado (no es un comando de contrato):** aplicar la versión objetivo al manifest manteniendo `Cargo.toml` en `0.9.0` produce `DRIFT release-manifest.json: found '<v> (v<v>)', expected '0.9.0 (v0.9.0)'` + `FAILED: 1 target(s) drifted…` (exit 1). Reproducido y revertido — requiere tocar un archivo versionado, por lo que **no** es parte del contrato verificable por terceros.

## Spec (SDD)

No es feature-add: no agrega símbolos públicos, endpoints, tools MCP ni capabilities. Es una tarea de release/versionado cuyo entregable es una decisión documentada + evidencia + una propuesta de corrección de plan. Sin §Spec.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. **Nunca** editar `Cargo.toml` / `Cargo.lock` a mano — release-plz es el dueño del versionado (AGENTS.md Regla 7).
  2. **Nunca** inventar los 4 SHA256 de la fórmula: solo existen tras el dispatch que produce los assets.
  3. **Nunca** dejar `release-manifest.json` en una versión distinta de `[workspace.package] version` en la misma rama — el gate compara igualdad estricta.
  4. `version` y los 4 `sha256` de la fórmula son **atómicos**: se mueven en el mismo commit, con los assets ya publicados y sus sidecars descargables.
- **Comandos de verificación:** `node scripts/docs/sync-workspace-version.mjs --check` → exit 0 sin `DRIFT` · `rg -i "Formula" .github/ scripts/ dev-tools/ justfile .githooks/` → ningún gate.
- **Deuda pendiente:** D1-D4 abajo (ninguna bloquea el release; ninguna tiene ID `FIND-*` todavía — ver D5).

## Deuda técnica (Regla 6 — MUST)

**Saldo neto: sin deuda.** Esta tarea no edita código ni configuración; todo lo listado es deuda *preexistente* que la tarea destapó.

| # | Hallazgo | Evidencia | Coste | Bloquea el release |
|---|----------|-----------|-------|--------------------|
| **D1** | **`PUBLISH.md` se contradice internamente.** §11 (:65-69) dice que el bump del manifest es "**manual**"; §"Orden del bump npm…" (:195-211) dice que desde FIND-379 está "**completamente automatizada**" y que el bot commitea `chore(release): sync monorepo surface manifests to v<version>`. Ambas frases sobreviven en el mismo documento. | `PUBLISH.md:65-69` vs `:195-211`; prueba de que la automatización **corrió**: commit `f2947da6c` "…to v0.9.1" (bot) en la rama de PR #288 | 🟢 15 min (vanta-docs): borrar §11 o reenviarla a :195 y completar la enumeración de :203-209, que omite `release-manifest.json`, `package-lock.json` y las cotas de `[workspace.dependencies]` | No |
| **D2** | **`scripts/update-homebrew-formula.sh` corrompe la fórmula.** (a) `FORMULA="$SCRIPT_DIR/../vantadb.rb"` (:8) apunta a la raíz; la fórmula está en `Formula/vantadb.rb`. (b) El bucle (:20-29) hace `sed -i "s\|sha256 \".*\"$\|sha256 \"$SHA\"\|"` sobre **todo el fichero**, sin direccionar línea: cada iteración reescribe **los 4** `sha256` con el digest de la URL recién procesada → la fórmula queda con 4 digests idénticos (gana el último). (c) `grep -oP` (:20) es GNU-only sin guard. Los `sed -i ''` sí están correctamente ramificados por `$OSTYPE` (:13-17, :24-28). | Lectura completa del script; `ls vantadb.rb` → no existe | 🟢 1-2h (direccionar el `sed` por URL/plataforma, o retirar el script y documentar el `for` manual que ya vive en `Formula/vantadb.rb:8-11`) | No |
| **D3** | **Ningún gate verifica la fórmula, ni siquiera post-release.** `verify-release.ps1` no menciona `Formula` en absoluto: la trampa del 0.9.0 (fórmula quedada en 0.8.0) puede repetirse y el release quedaría "verde". | `rg -c "Formula" scripts/verify-release.ps1` → sin match | 🟢 45 min (paso que compare `version` del `.rb` == tag y los 4 `sha256` == sidecars) **+ restaurar el trigger**: `release-verify.yml:10` es `workflow_dispatch` only y el cron está comentado (:8-9), así que el check solo corre si un humano lo despacha — la misma disciplina que falló en 0.9.0. Sin el trigger, paga la mitad. | No |
| **D4** | **No hay smoke de instalación de la fórmula en CI.** La matriz sí construye los 5 targets, **ambos macOS incluidos** (`release-binaries.yml:63-78`, comentario :64-66), pero ningún job hace `brew audit`/`brew install` de la fórmula contra los assets recién subidos. | `release-binaries.yml:63-78` | 🔴 1-2d (runner macOS + brew audit + brew install) | No — deuda estructural |
| **D5** | **D1-D4 no tienen ID durable.** `prompts/task.md:267-274` pide IDs de deuda; no hay filas `FIND-*` para estos cuatro hallazgos, así que "pagar D3" apunta a trabajo que solo existe en este fichero. D2 además es **re-descubrimiento**: `docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:349` ya registraba "script de update con ruta rota (`../vantadb.rb` vs `Formula/vantadb.rb`)". | `rg -i "homebrew\|verify-release" docs/dev/Backlog.md` → FIND-261/474/510, ninguno es D3 | 🟢 10 min — **el orquestador debe crear las filas `FIND-*`** (esta tarea no puede: `docs/dev/Backlog.md` fuera de su scope de escritura) | No |

**Recomendación (ponytail):** no automatizar el bump de la fórmula. Exige un step que calcule los 4 SHA256 tras el upload y commitee **fuera del tag**, porque el lane corre sobre el tag y "los tags solo los crea release-plz"; 4-6 h y roza la Regla 7. **D3 (con el trigger restaurado) cuesta ~1 h y convierte un paso manual de pre-flight en un gate mecánico.** Pagar D3 antes que escribir el automatismo.

## Definition of Done

| Nivel | Gate | Estado |
|-------|------|--------|
| **Task** | Contrato reproducible (2 comandos, version-agnósticos) | ✅ |
| **Commit** | Commit atómico con conventional commit; `git status` limpio al cerrar | ✅ `5d0dcdfbd` — 3 ficheros (task file + `docs/index.md` + `llms.txt` regenerados por `gen-index --write`); `Formula/`, `release-manifest.json`, `Cargo.toml`, `Cargo.lock` sin cambios |
| **Release** | No aplica: 0 archivos de código/config tocados → no hay fmt/clippy/nextest que correr. Gates de docs sí corridos (`check-docs`, `check-links`, `gen-index --check`) | ✅ N/A justificado |

## Herramientas necesarias
- `rg`, `node scripts/docs/*.mjs`, `git`, `gh` (lectura), PowerShell
- `webfetch` (Homebrew Formula Cookbook — semántica `version` + `sha256`)

**Skills cargadas (SDP):** `campaign-executor`, `progreso` (base) · `git-workflow-and-versioning` (ownership del versionado: "derive the version from the tag, don't hand-edit it in scattered files") · `shipping-and-launch` (pre-launch checklist del release) · `doubt-driven-development` (la decisión "no ejecutar" pasó por review adversarial — ver §Review) · `documentation-skill` (obligatoria: `.md` bajo `docs/`). Descartadas del SDP por no aplicar: `test-driven-development`, `frontend-ui-engineering`, `context-engineering`, `source-driven-development`.

## Investigation Notes

### ¿Por qué hay que bumpear a mano? (§8 de la brief)
release-plz gobierna los manifiestos del **workspace Rust** (`Cargo.toml`, `Cargo.lock`, `docs/CHANGELOG.md`, tags). Ni `Formula/vantadb.rb` ni `release-manifest.json` son miembros del workspace ni están en `release-plz.toml`, así que release-plz no los ve. Pero la consecuencia del plan ("por eso hay que bump-earlos a mano en `develop`") es falsa para uno de los dos:

- **`release-manifest.json`: lo mueve el release, automáticamente.** El step `Sync workspace version across monorepo manifests on Release PR branch` (`release.yml:119-156`) corre `node scripts/docs/sync-workspace-version.mjs --write` sobre la rama `release-plz-*`; el manifest está en la lista de specs del script (`sync-workspace-version.mjs:402-408`, `expected: \`${workspaceVersion} (v${workspaceVersion})\``) y el bot commitea el resultado. **No es teoría:** el PR #288 (`release-plz-2026-11-18T03-12-19Z`) ya lleva `release-manifest.json` en `"version": "0.9.1"` / `"tag": "v0.9.1"`, del commit de bot `f2947da6c`.
- **`Formula/vantadb.rb`: no lo mueve nadie.** Ningún gate lo lee, ningún script lo actualiza en CI, y `verify-release.ps1` no lo menciona.

### Prueba empírica del gate (reproducida por el reviewer, revertida)
```
$ node scripts/docs/sync-workspace-version.mjs --check
Workspace version: 0.9.0 (from Cargo.toml)
ok   release-manifest.json: 0.9.0 (v0.9.0)
OK: All 22 manifests synchronized with workspace version 0.9.0.        [exit 0]

# con el manifest en 0.10.0 y Cargo.toml intacto en 0.9.0:
DRIFT release-manifest.json: found '0.10.0 (v0.10.0)', expected '0.9.0 (v0.9.0)'
FAILED: 1 target(s) drifted from workspace version 0.9.0.              [exit 1]

# restaurado: exit 0 · git status --short release-manifest.json → vacío
```
El gate vive en `.github/workflows/docs-gates.yml:177` (step `Gate — all versioned manifests match workspace version`), job `check-npm-versions` (`name:` en :156), y el workflow dispara con `release-manifest.json` en sus path filters (:40 push, :64 PR). El nombre del check, `Check npm package versions match workspace`, está en el set `REQUIRED` de `ci-gate.yml:48`, que espeja 1:1 el ruleset de `main`. **Precisión:** lo probado es que *el gate se pone rojo*; en un `push` a `develop` no hay gating por required-check, el efecto real es el PR `develop → main` bloqueado.

### Por qué la fórmula tampoco se puede bumpear sola
`version` y los 4 `sha256` describen **los mismos artefactos**. Con `version "0.10.0"` y los hashes vigentes, la fórmula descarga `…/v<target>/vantadb-<target>.tar.gz` de la nueva versión y verifica contra un digest de la anterior → mismatch → instalación abortada. Es **peor** que la trampa del 0.9.0 (fórmula quedada en 0.8.0), que al menos instalaba algo funcional. Y los assets nuevos no existen hasta que corra el dispatch de binarios, así que el bump atómico es **imposible** en F0.4.
Referencia oficial: [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook) — `version` determina la versión instalada y cada `url` declara su `sha256` asociado, un par por plataforma.

### Corrección propuesta al plan (para el orquestador — **no decidida por esta tarea**)
La fila F0.4 ("`Formula/vantadb.rb` + `release-manifest.json` → 0.10.0 en el mismo commit") no es ejecutable en `develop`. Propuesta:
- **`release-manifest.json`** no necesita tarea propia: lo actualiza `release.yml:140` en el PR de release (F0.6).
- **`Formula/vantadb.rb`** pertenece **después** del dispatch de F0.5 — y el plan **ya lo tenía ahí**: la fila F0.5 verifica "sha del `.rb` == `sha256sum` de los assets". La corrección es solo borrar el solapamiento de F0.4, no re-arquitecturar nada.
- Se mantiene intacta la autorización del plan (§1: "F0.4 se puede preparar y commitear **local**"): lo que se propone es que no haya nada que preparar, no cambiar la política de commits.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | **0** |
| Pendientes de ejecución (downhill) | **0** |
| % completado | 100% |

Resueltas y movidas a Notas:
1. ~~¿Hay gate de igualdad para `Formula/`?~~ → **No.**
2. ~~¿Hay gate de igualdad para `release-manifest.json`?~~ → **Sí**, probado (exit 1 con drift).
3. ~~¿El bump del manifest es manual?~~ → **No**, automatizado y **ya ejecutado** en la rama del PR #288.

## Steps

### Step 1: Discovery de gates de igualdad de versión
- **Archivos:** `.github/workflows/docs-gates.yml`, `.github/workflows/release.yml`, `scripts/docs/sync-workspace-version.mjs`, `scripts/verify-release.ps1`
- **Acción:** localizar todo gate que compare `Formula/` o `release-manifest.json` con `[workspace.package] version`
- **Verify:** `rg -n "sync-workspace-version" .github/workflows/` → **4 líneas** (`docs-gates.yml:174` nombre del step, `:175` self-test, `:177` `--check`, `release.yml:140` `--write`)
- **Estado:** ✅

### Step 2: Prueba empírica del gate con el bump hipotético
- **Archivos:** `release-manifest.json` (editado y **revertido**)
- **Acción:** aplicar la versión objetivo, correr `--check`, capturar exit 1, restaurar
- **Verify:** `--check` → exit 0 tras restaurar; `git status --short release-manifest.json` → vacío
- **Estado:** ✅

### Step 3: Verificación de ausencia de gate en la fórmula
- **Archivos:** `.github/**`, `scripts/**`, `dev-tools/**`, `justfile`, `.githooks/**`
- **Acción:** grep repo-wide de `Formula` en superficies de gate
- **Verify:** `rg -i "Formula" .github/ scripts/ dev-tools/ justfile .githooks/` → ningún gate entre los hits
- **Estado:** ✅

### Step 4: Validar semántica de la fórmula contra doc oficial
- **Archivos:** `Formula/vantadb.rb`
- **Acción:** confirmar que `version` + `sha256` son un par por artefacto
- **Verify:** [Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- **Estado:** ✅

### Step 5: Evidencia de que la automatización del manifest ya corrió
- **Archivos:** (lectura) PR #288, commit `f2947da6c`, `release.yml:119-156`
- **Acción:** confirmar que el manifest de la rama `release-plz-*` ya está en la versión del release propuesto
- **Verify:** `gh pr view 288` → OPEN, `release-plz-2026-11-18T03-12-19Z`; contenido del manifest en esa ref → `"version": "0.9.1"`, `"tag": "v0.9.1"`
- **Estado:** ✅

### Step 6: Decisión + registro + gates de docs
- **Archivos:** `docs/dev/tasks/F0-4-version-bump.md`, `docs/index.md`, `llms.txt`
- **Acción:** documentar veredicto, corregir la deuda con evidencia medida, registrar el review adversarial; regenerar índices
- **Verify:** los 2 comandos del §Contrato + `check-docs` / `check-links` / `gen-index --check` en 0
- **Estado:** ✅

## Dependencias
- F0.3: ✅ cerrada (`919632e21`, `29cbcbcf1`)
- **F0.5 no depende de F0.4**: el dispatch de binarios es independiente. El bump de la fórmula se hace **después** de F0.5, con los SHA256 en mano (como el propio plan ya estipulaba en la fila F0.5).
- F0.6 recibe `release-manifest.json` en la versión del release **automáticamente** vía `release.yml:140`.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` — sesión `ses_df0f2e2bdffenQlADqjh9kvZT0`, contexto fresco, prompt adversarial ("find what is wrong", ARTIFACT + CONTRACT sin la conclusión del autor)
- **Enfoque:** ¿es correcto NO ejecutar el bump? ¿el gate citado es real y el que bloquearía? ¿alguna afirmación está inventada?
- **Cómo se probó:** el revisor re-ejecutó por su cuenta `sync-workspace-version --check` (exit 0), reprodujo el bump a 0.10.0 (exit 1), lo restauró, leyó los workflows, y consultó `gh pr view 288`. No aceptó ninguna cifra sin medirla.
- **Veredicto inicial:** ❌ **changes-required** (15 findings) — la conclusión "no bumpear" sobrevivió; el aparato de evidencia no.
- **Reconciliación (12 aceptados, 3 reclasificados):**

| # | Finding | Clase | Acción |
|---|---------|-------|--------|
| 1 | DoD `Commit=✅` con el árbol sucio y sin commit | actionable | Corregido: la fila declara el hash real tras commitear |
| 2 | §Review auto-atestiguado, veredicto inexistente | actionable | Corregido: esta tabla + veredicto del revisor |
| 3 | **D4 falso**: la matriz sí construye ambos macOS | actionable | Corregido: D4 reescrita (el gap es el smoke de `brew`, no la matriz) |
| 4 | D1 mal diagnosticada: `PUBLISH.md` documenta la automatización | actionable | Corregido: es **autocontradicción** (:65-69 vs :195-211), no sección obsoleta |
| 5 | **PR #288 (v0.9.1) ya abierto**, con el manifest ya sincronizado | actionable | Aceptado y **escalado** (§Notas + `queda_pendiente`): el objetivo 0.10.0 no está establecido |
| 6 | Contrato pineado a `0.9.0`, se rompe solo | actionable | Corregido: contrato version-agnóstico (exit 0 sin `DRIFT`) |
| 7 | "3 comandos" = 2 + un procedimiento que viola mi propio invariante | trade-off | Corregido: 2 comandos de contrato + experimento documentado aparte |
| 8 | Reescribe el plan sin autoridad + contradicción interna | actionable | Corregido: es **propuesta**, marcada como no decidida; Metadata ya no dice "bloquea F0.5"; se cita que F0.5 ya tenía el chequeo |
| 9 | D3 sobrevendida: `release-verify.yml` es dispatch-only | trade-off | Aceptado: se añade el coste de restaurar el trigger (:8-10) |
| 10 | D2 mecanismo mal: los `sed` sí están ramificados; el defecto real es el bucle | actionable | Corregido: D2 describe el bucle que reescribe los 4 sha256 |
| 11 | Deuda sin ID durable; D2 es re-descubrimiento | actionable | Corregido: D5 nueva + cita de `strategy:349` |
| 12 | Cifras mal: "3 hits"→4; "0 gates" como salida; PyPI ×12→11 | actionable | Corregido: 4 líneas, "ningún gate", 1/11/3=15 |
| 13 | "required on develop" solo indirecto | trade-off | Corregido: se separa "el gate se pone rojo" (probado) de "está en REQUIRED de main" (`ci-gate.yml:48`) |
| 14 | Rangos de línea de los filtros | noise | Corregido (:20-41 push, :44-65 PR) |
| 15 | `brew install vantadb` propaga un comando que el repo marca falso | noise | Corregido: blast radius acotado + cita de la deuda existente |

- **Veredicto final:** ✅ **approve** tras aplicar las 12 correcciones (el revisor cerró explícitamente el punto que importa: *"I did not find any cheaper correct path that satisfies the original plan contract on `develop`… The 'do nothing' verdict is right"*).
- **Checklist anti-hábitos tóxicos:** [x] sin salidas inventadas (las 3 cifras erróneas se corrigieron con medición) · [x] sin done sin verificar · [x] el fallo (exit 1) se documenta como prueba, no se oculta · [x] sin bucle sin diagnóstico · [x] cobertura SDP v3: release/versionado = `git-workflow-and-versioning` + `shipping-and-launch`; docs = `documentation-skill`

## Notas
- **La tarea no produce un diff de versión, produce una corrección de plan + evidencia.** Entregable: los dos ficheros intactos, el por qué, y F0.5/F0.6 re-ruteados.
- **Escalado al orquestador (Gate D):** hay un PR de release **abierto** para **v0.9.1** (#288) con el manifest ya en 0.9.1. El plan asume 0.10.0. Tras mergear #288, release-plz recalcula desde `v0.9.1`, así que el próximo número puede ser 0.9.2 en vez de 0.10.0 según qué conventional commits haya. **Es decisión del owner/planner, no de esta tarea** — y el veredicto de F0.4 no cambia con el número.
- Árbol compartido: F0.2 trabaja en `docs/dev/avance/activo/operaciones.md` + fila `FIND-706`. Rutas disjuntas.
- `Cargo.toml` **no se tocó** y `Cargo.lock` tampoco.
